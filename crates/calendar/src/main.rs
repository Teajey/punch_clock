mod display;

use anyhow::Context as _;
use anyhow::anyhow;
use chrono::Utc;
use clap::Parser;
use punch_clock_core::context::Context;
use punch_clock_core::day;
// use punch_clock_core::error;
use punch_clock_core::record;

fn main() {
    if let Err(err) = run() {
        println!("Error: {err}");
        std::process::exit(1);
    }
}

#[derive(Parser)]
#[command(author, about)]
struct Cli {
    /// YYYY-MM-DD
    from: Option<day::Day>,
    /// YYYY-MM-DD
    to: Option<day::Day>,
    /// Set the character width of the calendar
    #[arg(long, default_value_t = 48)]
    width: usize,
}

fn run() -> anyhow::Result<()> {
    let cli = Cli::parse();

    let record = record::Record::<Utc>::load_stdin().context("failed to load record")?;

    let skip_hooks = std::env::var("PUNCH_CLOCK_SKIP_HOOKS")
        .ok()
        .map(|x| x.parse::<bool>())
        .transpose()
        .ok()
        .flatten()
        .unwrap_or_default();

    let (from, to) = match (cli.from.as_ref(), cli.to.as_ref()) {
        (None, Some(_)) => unreachable!(),
        (None, None) => {
            let to = chrono::Local::now().date_naive();
            let from = to
                .checked_sub_days(chrono::Days::new(6))
                .ok_or_else(|| anyhow!("date out of range"))?;
            (from, to)
        }
        (Some(to), None) => {
            let to = to.0;
            let from = to
                .checked_sub_days(chrono::Days::new(6))
                .ok_or_else(|| anyhow!("date out of range"))?;
            (from, to)
        }
        (Some(from), Some(to)) => (from.0, to.0),
    };

    if let Some(offset) = std::env::var("PUNCH_CLOCK_FIXED_UTC_OFFSET")
        .ok()
        .map(|x| x.parse::<i32>())
        .transpose()
        .ok()
        .flatten()
    {
        let ctx = Context::init_with_offset(skip_hooks, offset)?;
        let record = record.with_timezone(&ctx.timezone);
        display::paint_day_range(&ctx, &record, from..=to, cli.width)?;
    } else {
        let ctx = Context::init(skip_hooks)?;
        let record = record.with_timezone(&ctx.timezone);
        display::paint_day_range(&ctx, &record, from..=to, cli.width)?;
    }

    Ok(())
}
