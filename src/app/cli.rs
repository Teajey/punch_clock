use clap::{Args, Parser, Subcommand};

#[derive(Parser)]
#[command(author, version, about)]
pub struct Base {
    #[command(subcommand)]
    pub action: Action,
    #[arg(long)]
    pub init: bool,
}

#[derive(Args)]
pub struct Day {
    #[arg(long, short, requires = "month", requires = "year")]
    pub day: Option<u32>,
    #[arg(long, short, requires = "day", requires = "year")]
    pub month: Option<u32>,
    #[arg(long, short, requires = "day", requires = "month")]
    pub year: Option<i32>,
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
    Stats(Day),
}
