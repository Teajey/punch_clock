mod action;
mod app;
mod error;

use std::fs;

use clap::Parser;

fn main() {
    if let Err(err) = run() {
        println!("Error: {err}");
        std::process::exit(1);
    }
}

fn run() -> error::Result<()> {
    let cli = app::cli::Base::parse();

    if cli.init {
        app::context::init()?;
    }

    let Some(ctx) = app::context::load()? else {
        return Err(error::Main::Uninitialized);  
    };

    action::run(&ctx, &cli.action)?;

    fs::write(".punch_clock/record", ctx.record.borrow().to_string())?;
    
    Ok(())
}