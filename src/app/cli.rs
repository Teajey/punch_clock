mod day;

use clap::{Parser, Subcommand};

pub use day::Day;

#[derive(Parser)]
#[command(author, version, about)]
pub struct Base {
    #[command(subcommand)]
    pub action: Action,
    #[arg(long)]
    pub init: bool,
}

#[derive(Subcommand)]
pub enum Action {
    In,
    Out,
    Status,
    Dump {
        #[arg(short, long)]
        offset: Option<i32>,
    },
    Edit,
    Stats {
        day: Option<day::Day>,
    },
    Calendar {
        from: day::Day,
        to: day::Day,
        #[arg(long, default_value_t = 48)]
        width: usize,
    },
}
