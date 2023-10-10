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
    #[arg(short, long)]
    pub offset: Option<i32>,
}

#[derive(Subcommand)]
pub enum Action {
    In,
    Out,
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
}
