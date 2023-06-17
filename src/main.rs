mod action;
mod fs;
mod time;
mod app;
mod error;
mod record;

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

    let ctx = app::context::init()?;

    if cli.init {
        record::Record::init()?;
    }

    let Some(record) = record::Record::<Utc>::load()? else {
        return Err(error::Main::Uninitialized);  
    };

    action::run(&ctx, &cli.action, record)?;

    Ok(())
}