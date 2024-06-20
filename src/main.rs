use ansi_term::Colour;
use elasticnow::cli::{self, config};
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
        cli::args::Commands::Timetrack {
            new,
            comment,
            time_worked,
            search,
            bin,
        } => {
            let config = config::Config::from_toml_file();
            if config.is_err() {
                tracing::error!(
                    "Unable to load config file. Please run {} and try again.",
                    Colour::Green.bold().paint("elasticnow setup")
                );
                std::process::exit(2);
            }
            tracing::info!("New: {:?}", new);
            tracing::info!("Comment: {:?}", comment);
            tracing::info!("Time Worked: {:?}", time_worked);
            tracing::info!("Search: {:?}", search);
            tracing::info!("Bin: {:?}", bin);
            let tkts = vec!["TKT2923290", "TKT2923291", "TKT2923292"];
            let item = cli::args::choose_options(tkts);
            tracing::info!("Selected item: {:?}", item);
        }
        cli::args::Commands::Setup {
            id,
            instance,
            sn_instance,
            sn_username,
            sn_password,
            bin,
        } => {
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
    }
}
