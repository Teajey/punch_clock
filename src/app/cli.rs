use clap::{Parser, Subcommand};

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
    Stats,
}
