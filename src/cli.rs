use chrono::NaiveDate;
use clap::{ArgEnum, Parser, Subcommand};

pub fn init() -> Cli {
    Cli::parse()
}

#[derive(Debug, Parser)]
#[clap(name = "mococp")]
#[clap(about = "Moco CLI", long_about = None)]
pub struct Cli {
    #[clap(subcommand)]
    pub command: Commands,

    #[clap(long)]
    pub debug: bool,
}

#[derive(Debug, Subcommand)]
pub enum Commands {
    #[clap(about = "Login into (Moco/Jira)", long_about = None)]
    Login {
        #[clap(arg_enum, default_value_t = Login::Moco)]
        system: Login,
    },
    #[clap(about = "List activities", long_about = None)]
    List {
        #[clap(long)]
        week: bool,

        #[clap(long)]
        month: bool,

        #[clap(long)]
        backward: Option<i64>,

        #[clap(long)]
        date: Option<NaiveDate>,
    },
    #[clap(about = "Create new activity", long_about = None)]
    New {
        #[clap(long)]
        project: Option<i64>,

        #[clap(long)]
        task: Option<i64>,

        #[clap(long)]
        hours: Option<f64>,

        #[clap(long)]
        date: Option<NaiveDate>,
    },
    #[clap(about = "Edit activity", long_about = None)]
    Edit {
        #[clap(long)]
        activity: Option<i64>,
    },
    #[clap(about = "Delete activity", long_about = None)]
    Rm {
        #[clap(long)]
        activity: Option<i64>,

        #[clap(long)]
        date: Option<NaiveDate>,
    },
    #[clap(about = "Start/Stop activity timer", long_about = None)]
    Timer {
        #[clap(arg_enum)]
        system: Timer,

        #[clap(long)]
        activity: Option<i64>,
    },
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ArgEnum)]
pub enum Login {
    Moco,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ArgEnum)]
pub enum Timer {
    Start,
    Stop,
}
