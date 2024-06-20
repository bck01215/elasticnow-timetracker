use crate::cli::config::get_config_dir;
use ansi_term::Colour;
use clap::{Parser, Subcommand};
use dialoguer::{theme::ColorfulTheme, Select};
#[derive(Parser)]
#[command(author, version, about, long_about = None)]
pub struct Args {
    #[command(subcommand)]
    pub cmd: Commands,
}

#[derive(Subcommand, Debug, Clone)]
pub enum Commands {
    /// Run time tracking options utilizing ElasticNow and ServiceNow
    Timetrack {
        #[clap(short, long, conflicts_with = "search", action = clap::ArgAction::SetTrue)]
        /// Creates a new ticket instead of updating an existing one ( cannot be used with --search )
        new: Option<bool>,
        #[clap(short, long)]
        /// Comment for time tracking
        comment: Option<String>,
        #[clap(
            long,
            help = format!("Add time in the format of {} where 1 can be replaced with any number (hours must be less than 24)", Colour::Green.bold().paint("1h1m1s")))
        ]
        time_worked: String,
        #[clap(short, long, required_unless_present = "new")]
        /// Keyword search using ElasticNow (returns all tickets in bin by default)
        search: Option<String>,
        #[clap(short, long)]
        /// Override default bin for searching (defaults to user's assigned bin or override in config.toml)
        bin: Option<String>,
    },

    #[clap(about = format!("Create a new config file in {}", get_config_dir().display()))]
    Setup {
        #[clap(long, env = "ELASTICNOW_ID", hide_env_values = true)]
        /// The ElasticNow ID (retrieved from ElasticNow instance)
        id: String,
        #[clap(long, env = "ELASTICNOW_INSTANCE")]
        /// The ElasticNow instance
        instance: String,
        #[clap(long, env = "SN_INSTANCE")]
        /// The ServiceNow Instance (e.g. libertydev, liberty)
        sn_instance: String,
        #[clap(long, env = "SN_USERNAME")]
        /// The ServiceNow Username
        sn_username: String,
        #[clap(long, env = "SN_PASSWORD", hide_env_values = true)]
        /// The ServiceNow Password
        sn_password: String,
        /// Override default bin for searching (defaults to user's assigned bin)
        #[clap(short, long)]
        bin: Option<String>,
    },
}

pub fn get_args() -> Args {
    Args::parse()
}

pub fn choose_options(mut options: Vec<&str>) -> String {
    options.append(&mut vec!["New ticket", "Cancel"]);
    let selection = Select::with_theme(&ColorfulTheme::default())
        .with_prompt("Please choose a ticket:")
        .default(0)
        .items(&options)
        .interact()
        .unwrap();

    options[selection].to_string()
}
