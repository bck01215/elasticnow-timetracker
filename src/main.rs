use ansi_term::Colour;
use elasticnow::cli::{self, config};
use tracing_subscriber::{fmt, prelude::*, EnvFilter};

fn main() {
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
            tracing::info!("Config: {:?}", config);
            tracing::info!("New: {:?}", new);
            tracing::info!("Comment: {:?}", comment);
            tracing::info!("Time Worked: {:?}", time_worked);
            tracing::info!("Search: {:?}", search);
            tracing::info!("Bin: {:?}", bin);
        }
        cli::args::Commands::Setup {
            id,
            instance,
            sn_instance,
            sn_username,
            sn_password,
        } => {
            cli::config::make_dir_if_none();
            let config = cli::config::Config {
                id,
                instance,
                sn_instance,
                sn_username,
                sn_password,
            };
            let resp = config.to_toml_file();
            if resp.is_err() {
                tracing::error!("Unable to create config file");
            }
        }
    }
}
