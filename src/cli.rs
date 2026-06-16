use chrono::NaiveDate;
use clap::builder::styling::{AnsiColor, Styles};
use clap::{Parser, Subcommand, ValueEnum};

/// Combines clap v3's classic green/yellow help coloring with clap v4's
/// bold/underline emphasis: colored *and* styled section headers.
const HELP_STYLES: Styles = Styles::styled()
    .header(AnsiColor::Yellow.on_default().bold().underline())
    .usage(AnsiColor::Green.on_default().bold().underline())
    .literal(AnsiColor::Green.on_default().bold())
    .placeholder(AnsiColor::Green.on_default())
    .valid(AnsiColor::Green.on_default().bold())
    .invalid(AnsiColor::Yellow.on_default().bold())
    .error(AnsiColor::Red.on_default().bold());

pub fn init() -> Cli {
    Cli::parse()
}

#[derive(Debug, Parser)]
#[clap(name = "mococli")]
#[clap(about = "MOCO CLI", long_about = None)]
#[clap(styles = HELP_STYLES)]
pub struct Cli {
    #[clap(subcommand)]
    pub command: Commands,

    #[clap(long)]
    pub debug: bool,
}

#[derive(Debug, Subcommand)]
pub enum Commands {
    #[clap(about = "Login into MOCO", long_about = None)]
    Login,
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

        #[clap(long)]
        description: Option<String>,
    },
    #[clap(about = "Edit activity", long_about = None)]
    Edit {
        #[clap(long)]
        date: Option<NaiveDate>,

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
        #[clap(value_enum)]
        system: Timer,

        #[clap(long)]
        activity: Option<i64>,
    },
    #[clap(about = "Show your current overtime report", long_about = None)]
    Overtime {
        #[clap(long)]
        monthly: bool,
    },
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum)]
pub enum Timer {
    Start,
    Stop,
}
