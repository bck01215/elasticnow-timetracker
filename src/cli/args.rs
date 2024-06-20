use crate::cli::config::get_config_dir;
use ansi_term::Colour;
use clap::{Command, CommandFactory, Parser, Subcommand};
use clap_complete::{generate, Generator, Shell};
use dialoguer::{theme::ColorfulTheme, Select};
use std::io;
#[derive(Parser)]
#[command(name = "elasticnow", about = "ElasticNow time tracking CLI", version)]
pub struct Args {
    // If provided, outputs the completion file for given shell
    #[arg(long = "generate", value_enum)]
    generator: Option<Shell>,

    #[command(subcommand)]
    pub cmd: Option<Commands>,
}

#[derive(Subcommand, Debug, Clone)]
pub enum Commands {
    /// Run time tracking options utilizing ElasticNow and ServiceNow
    Timetrack {
        #[clap(short, long, conflicts_with = "search", action = clap::ArgAction::SetTrue)]
        /// Creates a new ticket instead of updating an existing one ( cannot be used with --search )
        new: bool,
        #[clap(short, long)]
        /// Comment for time tracking
        comment: String,
        #[clap(
            long,
            help = format!("Add time in the format of {} where 1 can be replaced with any number (hours must be less than 24)", Colour::Green.bold().paint("1h1m")))
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
    let args = Args::parse();
    if let Some(shell) = args.generator {
        let mut cmd = Args::command();
        print_completions(shell, &mut cmd);
        std::process::exit(0);
    }
    args
}

pub fn choose_options(mut options: Vec<String>) -> String {
    options.append(&mut vec!["New ticket".to_string(), "Cancel".to_string()]);
    let selection = Select::with_theme(&ColorfulTheme::default())
        .with_prompt("Please choose a ticket:")
        .default(0)
        .items(&options)
        .interact()
        .unwrap();

    options[selection].to_string()
}

pub fn write_short_description() -> String {
    dialoguer::Input::<String>::with_theme(&ColorfulTheme::default())
        .with_prompt("Short description:")
        .interact()
        .unwrap()
}

pub fn print_completions<G: Generator>(gen: G, cmd: &mut Command) {
    generate(gen, cmd, cmd.get_name().to_string(), &mut io::stdout());
}
