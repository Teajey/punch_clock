mod action;
mod time;
mod app;
mod error;
mod record;

use std::fs;

use clap::Parser;
use chrono::Utc;

fn main() {
    if let Err(err) = run() {
        println!("Error: {err}");
        std::process::exit(1);
    }
}

fn run() -> error::Result<()> {
    let cli = app::cli::Base::parse();

    if cli.init {
        record::Record::init()?;
    }

    let Some(record) = record::Record::load()? else {
        return Err(error::Main::Uninitialized);  
    };

    let ctx = app::context::load()?;

    let record = action::run(&ctx, &cli.action, record)?;

    fs::write(".punch_clock/record", record.serialize(&Utc)?)?;
    
    Ok(())
}