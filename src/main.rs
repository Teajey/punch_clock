mod action;
mod fs;
mod time;
mod app;
mod error;
mod record;

use clap::Parser;
use chrono::{Utc, FixedOffset, Local};

fn main() {
    if let Err(err) = run() {
        println!("Error: {err}");
        std::process::exit(1);
    }
}

fn run() -> error::Result<()> {
    let cli = app::cli::Base::parse();

    std::env::set_current_dir(fs::file_location_in_path_by_prefix(".punch_clock")?)?;

    if cli.init {
        record::Record::init()?;
    }

    let Some(record) = record::Record::<Utc>::load()? else {
        return Err(error::Main::Uninitialized);  
    };

    if let Some(offset) = cli.offset {
        let ctx = app::Context::init(FixedOffset::east_opt(offset * 3600).ok_or_else(|| error::Main::TimezoneOutOfRange(offset))?)?;
        action::run(&ctx, &cli.action, record)?;
    } else {
        let ctx = app::Context::init(Local)?;
        action::run(&ctx, &cli.action, record)?;
    }


    Ok(())
}