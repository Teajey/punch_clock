mod day;

use clap::{Parser, Subcommand, ValueEnum};

pub use day::Day;

#[derive(Parser)]
#[command(author, version, about)]
pub struct Base {
    #[command(subcommand)]
    pub action: Action,
    #[arg(long)]
    pub init: bool,
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
    In {
        comment: Option<String>,
    },
    Out {
        comment: Option<String>,
    },
    Status,
    Dump,
    Edit,
    Stats {
        day: Option<day::Day>,
    },
    Calendar {
        from: Option<day::Day>,
        to: Option<day::Day>,
        #[arg(long, default_value_t = 48)]
        width: usize,
    },
    Undo,
    Day {
        date: Option<day::Day>,
        #[arg(short = 'r', long, value_enum, default_value_t = DayResolution::Hour)]
        resolution: DayResolution,
    },
}
