mod action;
mod app;
mod fs;
mod plugins;
mod string;

use chrono::Utc;
use punch_clock_core::context::Context;
use punch_clock_core::error;
use punch_clock_core::record;

static GIT_REVISION: &str = env!("PUNCH_CLOCK_GIT_REVISION");
static LONG_VERSION: &str = env!("PUNCH_CLOCK_LONG_VERSION");

fn main() {
    if let Err(err) = run() {
        println!("Error: {err}");
        std::process::exit(1);
    }
}

fn run() -> error::Result<()> {
    std::env::set_current_dir(fs::file_location_in_path_by_prefix(".punch_clock")?)?;

    let cli = app::cli::v2::Base::parse();

    if cli.init {
        record::Record::init()?;
    }

    let Some(record) = record::Record::<Utc>::load()? else {
        return Err(error::Main::Uninitialized);
    };

    let action = cli.action.unwrap_or(app::cli::v2::Action::Status);

    if let Some(offset) = cli.offset {
        let ctx = Context::init_with_offset(cli.skip_hooks, offset)?;
        action::run(&ctx, &action, record)?;
    } else {
        let ctx = Context::init(cli.skip_hooks)?;
        action::run(&ctx, &action, record)?;
    }

    Ok(())
}
