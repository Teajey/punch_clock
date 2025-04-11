mod action;
mod app;
mod error;
mod fs;
mod range;
mod record;
mod script_hook;
mod string;
mod time;

use chrono::{FixedOffset, Local, Utc};
use clap::Parser;

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

    let action = cli.action.unwrap_or(app::cli::Action::Status);

    if let Some(offset) = cli.offset {
        let ctx = app::Context::init(
            FixedOffset::east_opt(offset * 3600).ok_or(error::Main::TimezoneOutOfRange(offset))?,
            cli.skip_hooks,
        )?;
        action::run(&ctx, &action, record)?;
    } else {
        let ctx = app::Context::init(Local, cli.skip_hooks)?;
        action::run(&ctx, &action, record)?;
    }

    Ok(())
}
