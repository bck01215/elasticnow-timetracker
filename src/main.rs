use ansi_term::Colour;
use elasticnow::cli::{self, config};
use elasticnow::elasticnow::elasticnow::{ElasticNow, SearchResult};
use elasticnow::elasticnow::servicenow::ServiceNow;
use tracing_subscriber::{fmt, prelude::*, EnvFilter};

#[tokio::main]
async fn main() {
    tracing_subscriber::registry()
        .with(fmt::layer())
        .with(EnvFilter::from_env("ELASTICNOW_LOG_LEVEL"))
        .init();
    let args = cli::args::get_args();
    match args.cmd {
        Some(cli::args::Commands::Timetrack {
            new,
            comment,
            time_worked,
            search,
            bin,
        }) => {
            run_timetrack(new, comment, time_worked, search, bin).await;
        }
        Some(cli::args::Commands::Setup {
            id,
            instance,
            sn_instance,
            sn_username,
            sn_password,
            bin,
        }) => {
            run_setup(id, instance, sn_instance, sn_username, sn_password, bin).await;
        }
        _ => {
            std::process::exit(1);
        }
    }
}

async fn run_timetrack(
    new: bool,
    comment: String,
    time_worked: String,
    search: Option<String>,
    bin: Option<String>,
) {
    let config = config::Config::from_toml_file();
    if config.is_err() {
        tracing::error!(
            "Unable to load config file. Please run {} and try again.",
            Colour::Green.bold().paint("elasticnow setup")
        );
        std::process::exit(2);
    }
    let config = config.unwrap();

    tracing::debug!("New: {:?}", new);
    tracing::debug!("Comment: {:?}", comment);
    tracing::debug!("Time Worked: {:?}", time_worked);
    tracing::debug!("Search: {:?}", search);
    tracing::debug!("Bin: {:?}", bin);
    let sn_client = ServiceNow::new(
        &config.sn_username,
        &config.sn_password,
        &config.sn_instance,
    );

    let sys_id: String;
    let tkt_bin = bin.unwrap_or(config.bin.clone());
    if new {
        sys_id = new_ticket(&sn_client, &config).await;
    } else {
        let es_now_client = ElasticNow::new(&config.id, &config.instance);
        let keywords = search.clone().unwrap_or("".to_string());
        let tkt_options = search_tickets(es_now_client, &tkt_bin, &keywords).await;
        let tkt_options_string = search_results_to_string(&tkt_options);
        let item = cli::args::choose_options(tkt_options_string);
        tracing::debug!("Selected item: {}", &item);
        match &*item {
            "Cancel" => {
                std::process::exit(0);
            }
            "New ticket" => {
                sys_id = new_ticket(&sn_client, &config).await;
            }
            _ => {
                let tkt = get_search_result_from_input(&item, tkt_options);
                if tkt.is_none() {
                    tracing::error!("Unexpected error on input");
                    std::process::exit(2);
                }
                sys_id = tkt.unwrap().source.id;
            }
        }
    }
    tracing::debug!("Adding sys_id: {}", sys_id);

    let resp = sn_client
        .add_time_to_ticket(&sys_id, &time_worked, &comment)
        .await;
    if resp.is_err() {
        tracing::error!("Unable to add time to ticket: {:?}", resp.err());

        std::process::exit(2);
    }
    let time_worked_msg = ansi_term::Colour::Green.paint(time_worked);
    println!("Tracking {} of time", time_worked_msg);
    let ticket_url = ansi_term::Colour::Blue.paint(format!(
        "https://{}.service-now.com/task.do?sys_id={}",
        &config.sn_instance, sys_id
    ));
    println!("Link to ticket: {}", ticket_url);
    std::process::exit(0);
}

async fn run_setup(
    id: String,
    instance: String,
    sn_instance: String,
    sn_username: String,
    sn_password: String,
    bin: Option<String>,
) {
    cli::config::make_dir_if_none();
    let mut config = cli::config::Config {
        id,
        instance,
        sn_instance,
        sn_username,
        sn_password,
        bin: "".to_string(),
    };
    let sn_client = ServiceNow::new(
        &config.sn_username,
        &config.sn_password,
        &config.sn_instance,
    );
    if bin.is_none() {
        let user_group = sn_client.get_user_group(&config.sn_username).await;
        if user_group.is_err() {
            tracing::error!("Unable to get user group: {:?}", user_group.err());
            std::process::exit(2);
        }
        config.bin = user_group.unwrap();
    } else {
        config.bin = bin.unwrap();
    }
    let toml_resp = config.to_toml_file();
    if toml_resp.is_err() {
        tracing::error!("Unable to create config file: {:?}", toml_resp.err());
        std::process::exit(2);
    }
}

//Returns the sys_id of new ticket
async fn new_ticket(sn_client: &ServiceNow, config: &config::Config) -> String {
    let desc = cli::args::write_short_description();
    tracing::debug!("Creating new ticket: {}", &desc);
    let resp = sn_client.create_ticket(&config.bin, &desc).await;
    if resp.is_err() {
        tracing::error!("Unable to create ticket: {:?}", resp.err());
        std::process::exit(2);
    }
    let sys_id = resp.unwrap();
    tracing::info!(
        "Created ticket: https://{}.service-now.com/sc_req_item.do?sys_id={}",
        &config.sn_instance,
        sys_id
    );
    return sys_id;
}

async fn search_tickets(es_now_client: ElasticNow, bin: &str, keywords: &str) -> Vec<SearchResult> {
    let resp = es_now_client.get_keyword_tickets(keywords, bin).await;
    if resp.is_err() {
        tracing::error!("Unable to search tickets: {:?}", resp.err());
        std::process::exit(2);
    }
    resp.unwrap()
}

fn search_results_to_string(result: &Vec<SearchResult>) -> Vec<String> {
    let mut string_vec: Vec<String> = Vec::new();
    for r in result {
        string_vec.push(format!(
            "{}: {}",
            r.source.number, r.source.short_description
        ));
    }
    string_vec
}

fn get_search_result_from_input(input: &str, result: Vec<SearchResult>) -> Option<SearchResult> {
    for r in result {
        if input.starts_with(&r.source.number) {
            return Some(r);
        }
    }
    None
}
