mod day;

use clap::{Parser, Subcommand, ValueEnum};

pub use day::Day;

#[derive(Parser)]
#[command(author, version, about)]
pub struct Base {
    #[command(subcommand)]
    /// Default: status
    pub action: Option<Action>,
    #[arg(long)]
    /// Create a .punch_clock directory in the current working directory
    pub init: bool,
    /// Override punch_clock's current UTC offset
    #[arg(short, long)]
    pub offset: Option<i32>,
}

#[derive(Clone, ValueEnum)]
pub enum DayResolution {
    Hour = 1,
    HalfHour = 2,
    ThirdHour = 3,
    QuarterHour = 4,
    TenMinutes = 6,
    FiveMinutes = 12,
    TwoMinutes = 30,
    Minute = 60,
}

impl DayResolution {
    pub fn as_hour_fraction(&self) -> u16 {
        match self {
            DayResolution::Hour => 1,
            DayResolution::HalfHour => 2,
            DayResolution::ThirdHour => 3,
            DayResolution::QuarterHour => 4,
            DayResolution::TenMinutes => 6,
            DayResolution::FiveMinutes => 12,
            DayResolution::TwoMinutes => 30,
            DayResolution::Minute => 60,
        }
    }
}

#[derive(Subcommand)]
pub enum Action {
    /// Start a session
    In {
        /// Provide a comment associated with the start of this session
        comment: Option<String>,
    },
    /// End the current session
    Out {
        /// Provide a comment associated with the end of this session
        comment: Option<String>,
    },
    /// Check whether you're currently in a session
    Status,
    /// Print the record, formatted
    Dump,
    /// Open the record in your editor, in local time
    Edit,
    /// See some stats about your work hours
    Stats {
        /// For a particular day (YYYY-MM-DD)
        day: Option<day::Day>,
    },
    /// Print daily visualization of work hours (past 7 days by default)
    Calendar {
        /// YYYY-MM-DD
        from: Option<day::Day>,
        /// YYYY-MM-DD
        to: Option<day::Day>,
        /// Set the character width of the calendar
        #[arg(long, default_value_t = 48)]
        width: usize,
    },
    /// Remove the previous latest entry in the record
    Undo,
    /// Print visualization of a day's work hours (today by default)
    Day {
        /// YYYY-MM-DD
        date: Option<day::Day>,
        #[arg(short = 'r', long, value_enum, default_value_t = DayResolution::Hour)]
        resolution: DayResolution,
    },
}
