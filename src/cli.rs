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
#[clap(
    about = "MOCO CLI - A command line interface for MOCO",
    long_about = "A terminal-based interface for interacting with the MOCO time tracking system, allowing you to manage activities, track time, and view reports."
)]
#[clap(styles = HELP_STYLES)]
pub struct Cli {
    #[clap(subcommand)]
    pub command: Commands,

    #[clap(long, help = "Enable debug logging for troubleshooting")]
    pub debug: bool,
}

#[derive(Debug, Subcommand)]
pub enum Commands {
    #[clap(
        about = "Log into MOCO",
        long_about = "Configure your MOCO credentials, including company name, API keys, and user identification."
    )]
    Login,
    #[clap(
        about = "List activities (Precedence as listed)",
        long_about = "View tracked activities for a specific day, week, or month.\nThe Precedence refers to the order in which the flag args are listed below displayed.\n'--backward' is ignored when using the '--date' flag."
    )]
    List {
        #[clap(long, help = "Show activities for a specific date (YYYY-MM-DD)")]
        date: Option<NaiveDate>,

        #[clap(long, help = "Show activities for a specific day of the year")]
        day: Option<u32>,

        #[clap(long, help = "Show activities for a specific week of the year")]
        week: Option<u32>,

        #[clap(long, help = "Show activities for a specific month of the year")]
        month: Option<u32>,

        #[clap(
            long,
            help = "Go back in time from now based on the day, week, or month specified"
        )]
        backward: bool,
    },
    #[clap(
        about = "Create a new activity",
        long_about = "Start a new time tracking entry by specifying project, task, and duration."
    )]
    New {
        #[clap(long, help = "The ID of the project")]
        project: Option<i64>,

        #[clap(long, help = "The ID of the task")]
        task: Option<i64>,

        #[clap(long, help = "The number of hours to log (if omitted, timer starts)")]
        hours: Option<f64>,

        #[clap(long, help = "The date for the activity (YYYY-MM-DD)")]
        date: Option<NaiveDate>,

        #[clap(long, help = "A description of the work performed")]
        description: Option<String>,
    },
    #[clap(
        about = "Edit an existing activity",
        long_about = "Modify the details of a previously recorded activity."
    )]
    Edit {
        #[clap(long, help = "The date of the activity to edit (YYYY-MM-DD)")]
        date: Option<NaiveDate>,

        #[clap(long, help = "The ID of the activity to edit")]
        activity: Option<i64>,
    },
    #[clap(
        about = "Delete an activity",
        long_about = "Permanently remove a time tracking entry."
    )]
    Rm {
        #[clap(long, help = "The ID of the activity to delete")]
        activity: Option<i64>,

        #[clap(long, help = "The date of the activity to delete (YYYY-MM-DD)")]
        date: Option<NaiveDate>,

        #[clap(
            long,
            help = "Delete activities in a loop which exits when no more activities are found"
        )]
        r#loop: bool,
    },
    #[clap(
        about = "Start or stop the activity timer",
        long_about = "Manage the live timer for an activity."
    )]
    Timer {
        #[clap(value_enum, help = "Start or stop the timer")]
        system: Timer,

        #[clap(long, help = "The ID of the activity to control")]
        activity: Option<i64>,
    },
    #[clap(
        about = "Show your overtime report",
        long_about = "View your current overtime balance or a detailed monthly report."
    )]
    Overtime {
        #[clap(long, help = "Show a detailed monthly breakdown of overtime")]
        monthly: bool,
    },
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum)]
pub enum Timer {
    Start,
    Stop,
}
