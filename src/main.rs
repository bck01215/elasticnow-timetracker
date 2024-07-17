use ansi_term::Colour;
use elasticnow::cli::{self, args, config};
use elasticnow::elasticnow::elasticnow::ChooseOptions;
use elasticnow::elasticnow::elasticnow::{ElasticNow, SearchResult};
use elasticnow::elasticnow::servicenow::ServiceNow;
use elasticnow::elasticnow::servicenow_structs::TimeWorked;
use open::that;
use std::collections::HashMap;
use std::net::TcpListener;
use tiny_http::{Response, Server};
use tracing_subscriber::{fmt, prelude::*, EnvFilter};

struct ValueOption {
    value: String,
    display_value: String,
}

#[tokio::main]
async fn main() {
    tracing_subscriber::registry()
        .with(
            fmt::layer().event_format(
                tracing_subscriber::fmt::format()
                    .with_file(true)
                    .with_line_number(true),
            ),
        )
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
            no_tkt,
            all,
        }) => {
            run_timetrack(new, comment, time_worked, search, bin, no_tkt, all).await;
        }

        Some(cli::args::Commands::StdChg {
            search,
            bin,
            template_id,
        }) => {
            run_stdchg(search.unwrap_or_default(), bin, template_id).await;
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

        Some(cli::args::Commands::Report {
            user,
            mut since,
            until,
            top,
            today,
        }) => {
            if today {
                since = Some(args::get_today());
            }
            run_report(user, since, until, top).await;
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
    no_tkt: bool,
    all: bool,
) {
    let (mut config, sn_client) = check_config();
    tracing::debug!("New: {:?}", new);
    tracing::debug!("Comment: {:?}", comment);
    tracing::debug!("Time Worked: {:?}", time_worked);
    tracing::debug!("Search: {:?}", search);
    tracing::debug!("Bin: {:?}", bin);

    let tkt_bin = bin.unwrap_or(config.bin.clone());
    let mut sys_id: String = "".to_string();
    let resp: Result<(), Box<dyn std::error::Error>>;
    if no_tkt {
        let category = cli::args::choose_category();
        resp = sn_client
            .add_time_to_no_tkt(&category, &time_worked, &comment)
            .await;
    } else {
        if new {
            sys_id = new_ticket(&sn_client, &config).await;
        } else {
            let tkt_options_string: Vec<String>;
            let tkt_options: Vec<ValueOption>;
            if all {
                let tkt_options_res = sn_client.get_all_tickets_in_bin(&tkt_bin).await;
                if tkt_options_res.is_err() {
                    tracing::error!("Unable to get tickets: {:?}", tkt_options_res.err());
                    std::process::exit(2);
                }
                let tkt_options_generic = tkt_options_res.unwrap();
                tkt_options = generic_options_to_value_option(&tkt_options_generic);
                tkt_options_string = search_results_to_string(&tkt_options_generic);
            } else {
                let mut es_now_client = ElasticNow::new(&config.id, &config.instance);
                if es_now_client.check_auth().await.is_err() {
                    tracing::error!("Unable to authenticate to ElasticNow trying to log in");
                    let _cookie = get_cookie_from_browser(&config.instance);
                    config.set_new_id(&_cookie);
                    es_now_client = ElasticNow::new(&config.id, &config.instance);
                    let err = es_now_client.check_auth().await;
                    if err.is_err() {
                        tracing::error!("login attempt failed");
                        std::process::exit(1);
                    }
                }
                let keywords = search.clone().unwrap_or("".to_string());
                let tkt_options_generic = search_tickets(es_now_client, &tkt_bin, &keywords).await;
                tkt_options = generic_options_to_value_option(&tkt_options_generic);
                tkt_options_string = search_results_to_string(&tkt_options_generic);
            }
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
                    sys_id = tkt.unwrap().value;
                }
            }
        }
        tracing::debug!("Adding sys_id: {}", sys_id);

        resp = sn_client
            .add_time_to_ticket(&sys_id, &time_worked, &comment)
            .await;
    }

    if resp.is_err() {
        tracing::error!("Unable to add time to ticket: {:?}", resp.err());

        std::process::exit(2);
    }
    let time_worked_msg = ansi_term::Colour::Green.paint(time_worked);
    tracing::info!("Tracking {} of time", time_worked_msg);
    if !no_tkt {
        let ticket_url = ansi_term::Colour::Blue.paint(format!(
            "https://{}.service-now.com/task.do?sys_id={}",
            &config.sn_instance, sys_id
        ));
        tracing::info!("Link to ticket: {}", ticket_url);
    }
    std::process::exit(0);
}

async fn run_report(
    user: Option<String>,
    since: Option<String>,
    until: Option<String>,
    top: Option<usize>,
) {
    let (config, sn_client) = check_config();
    let user = user.unwrap_or(config.sn_username.clone());
    let since = since.unwrap_or(args::get_week_start());
    let until = until.unwrap_or(args::get_today());
    for date in vec![&since, &until] {
        let date_validate = args::range_format_validate(date);
        if date_validate.is_err() {
            tracing::error!("Invalid date format: {:?}", date_validate.err());
            std::process::exit(1);
        }
    }
    let tasks = sn_client.get_user_time_worked(&since, &until, &user).await;
    if tasks.is_err() {
        tracing::error!("Unable to get time worked: {:?}", tasks.err());
        std::process::exit(1);
    }
    let mut task_cat_time: HashMap<String, i64> = HashMap::new();
    let tasks = tasks.unwrap();
    let total = get_total(&tasks);
    let mut tasks_ids: HashMap<String, i64> = HashMap::new();
    for time_work in tasks {
        match time_work.task.as_ref() {
            "" => {
                let time_in_seconds: i64 = time_work.time_in_seconds.parse().unwrap_or_default();
                *task_cat_time
                    .entry(time_work.get_nice_name_category())
                    .or_insert(0) += time_in_seconds;
            }
            _ => {
                let time_in_seconds: i64 = time_work.time_in_seconds.parse().unwrap_or_default();
                *tasks_ids.entry(time_work.task).or_insert(0) += time_in_seconds;
            }
        }
    }
    let keys = tasks_ids.keys().cloned().collect::<Vec<String>>();
    let cost_centers = sn_client.get_tasks_cost_centers(&keys).await;
    if cost_centers.is_err() {
        tracing::error!("Unable to get cost centers: {:?}", cost_centers.err());
        std::process::exit(1);
    }
    let cost_centers = cost_centers.unwrap();
    for cost_center in cost_centers {
        let time = tasks_ids.get(&cost_center.task.value).unwrap_or(&0);
        *task_cat_time
            .entry(cost_center.cost_center.display_value)
            .or_insert(0) += time;
    }
    args::pretty_print_time_worked(task_cat_time, top.unwrap_or(10), total);
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

async fn run_stdchg(search: String, bin: Option<String>, template_id: Option<String>) {
    let (config, sn_client) = check_config();
    tracing::debug!("Search: {:?}", search);
    tracing::debug!("Bin: {:?}", bin);
    tracing::debug!("Template ID: {:?}", template_id);
    let bin = bin.unwrap_or(config.bin.clone());
    let template_sys_id: String;
    if template_id.is_none() {
        let std_changes_resp = sn_client.search_std_chg(&search).await;
        if std_changes_resp.is_err() {
            tracing::error!("Unable to search std chgs: {:?}", std_changes_resp.err());
            std::process::exit(2);
        }
        let std_changes = std_changes_resp.unwrap();
        if std_changes.is_empty() {
            tracing::error!("No std chgs found for search: {}", search);
            std::process::exit(1);
        }
        template_sys_id = cli::args::choose_chg_template(std_changes);
    } else {
        template_sys_id = template_id.unwrap();
    }
    tracing::debug!("Selected chg_id: {}", template_sys_id);
    let resp = sn_client
        .create_std_chg_from_template(&template_sys_id, &bin)
        .await;
    if resp.is_err() {
        tracing::error!("Unable to create std chg: {:?}", resp.err());
        std::process::exit(2);
    }
    let sys_id = resp.unwrap();
    tracing::info!("Created std chg: {}", sys_id);
    let ticket_url = ansi_term::Colour::Blue.paint(format!(
        "https://{}.service-now.com/change_request.do?sys_id={}",
        &config.sn_instance, sys_id
    ));
    tracing::info!("Link to CHG: {}", ticket_url);

    std::process::exit(0);
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

fn generic_options_to_value_option<T: ChooseOptions<T>>(result: &Vec<T>) -> Vec<ValueOption> {
    let mut value_vec: Vec<ValueOption> = Vec::new();
    for r in result {
        let val_opt = ValueOption {
            display_value: r.get_number(),
            value: r.get_id(),
        };
        value_vec.push(val_opt);
    }
    value_vec
}

fn search_results_to_string<T: ChooseOptions<T>>(result: &Vec<T>) -> Vec<String> {
    let mut string_vec: Vec<String> = Vec::new();
    for r in result {
        string_vec.push(r.get_debug_string());
    }
    string_vec
}

fn get_search_result_from_input(input: &str, result: Vec<ValueOption>) -> Option<ValueOption> {
    for r in result {
        if input.starts_with(&r.display_value) {
            return Some(r);
        }
    }
    None
}

fn check_config() -> (config::Config, ServiceNow) {
    let config = config::Config::from_toml_file();
    if config.is_err() {
        tracing::error!(
            "Unable to load config file. Please run {} and try again.",
            Colour::Green.bold().paint("elasticnow setup")
        );
        std::process::exit(2);
    }
    let config = config.unwrap();
    let sn_client = ServiceNow::new(
        &config.sn_username,
        &config.sn_password,
        &config.sn_instance,
    );
    (config, sn_client)
}

fn get_total(tasks: &Vec<TimeWorked>) -> i64 {
    tasks
        .iter()
        .map(|t| t.time_in_seconds.parse::<i64>().unwrap_or_default())
        .sum()
}

fn get_cookie_from_browser(elasticnow_url: &str) -> String {
    let mut chosen_port = 0;
    let mut server = None;
    for port in 8000..20000 {
        match TcpListener::bind(("0.0.0.0", port)) {
            Ok(listener) => {
                server = Some(Server::from_listener(listener, None).unwrap());
                chosen_port = port;
                println!("Server running on port {}", port);
                break;
            }
            Err(_) => {
                println!("Port {} is in use, trying next port...", port);
            }
        }
    }

    let server = match server {
        Some(s) => s,
        None => {
            tracing::error!(
                "Failed to bind to any port in the range 8000..20000 when attempting auth with elasticnow"
            );
            std::process::exit(1);
        }
    };

    // Define the login URL
    let login_url = format!("{}/cli/login/redirect/{}", elasticnow_url, chosen_port);

    // Open the browser for user to login
    match that(login_url) {
        Ok(_) => tracing::info!("Opened browser for to login to ElasticNow"),
        Err(e) => tracing::info!("Failed to open browser: {}", e),
    }

    // Set up the local server to capture the cookie
    let mut _id = "".to_string();
    for request in server.incoming_requests() {
        let response = Response::from_string("Login successful. You can close this window.");
        _id = request.url().to_string();
        _id.remove(0);
        request.respond(response).unwrap();
        break;
    }
    tracing::info!("Got cookie: {}", _id);
    _id.to_string()
}
