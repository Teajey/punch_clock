mod dump;
mod edit;
mod enter;
mod exit;
mod plugin;
mod stats;
mod status;
mod undo;

use std::{fs, io::Write as _};

use chrono::{Duration, NaiveTime, Utc};
use punch_clock_core::{
    context::Context,
    error::Result,
    record::{self, Record},
    script_hook,
    time::ContextTimeZone,
};

use crate::app::cli::{v2::Action, Day};

pub fn run<Tz: ContextTimeZone>(
    ctx: &Context<Tz>,
    action: &Action,
    mut record: Record<Utc>,
) -> Result<()> {
    match action {
        Action::In { comment } => {
            if !ctx.skip_hooks {
                script_hook::run("before-in")?;
            }

            enter::run(&mut record, comment.clone())?;

            fs::write(".punch_clock/record", record.serialize()?)?;

            if !ctx.skip_hooks {
                script_hook::run("in")?;
            }
        }
        Action::Out { comment } => {
            if !ctx.skip_hooks {
                script_hook::run("before-out")?;
            }

            exit::run(&mut record, comment.clone())?;

            fs::write(".punch_clock/record", record.serialize()?)?;

            if !ctx.skip_hooks {
                script_hook::run("out")?;
            }
        }
        Action::Status => status::run(&record)?,
        Action::Dump => {
            dump::run(&record.clone().with_timezone(&ctx.timezone))?;
        }
        Action::Edit => {
            let record = edit::run(ctx, record)?;
            fs::write(".punch_clock/record", record.serialize()?)?;
        }
        Action::Stats { day } => {
            let date = day.as_ref().map(|Day(date)| *date);
            stats::run(ctx, record.with_timezone(&ctx.timezone), date)?;
        }
        Action::Undo => {
            undo::run(&mut record)?;
            fs::write(".punch_clock/record", record.serialize()?)?;
        }
        Action::Day { date, resolution } => {
            let record = record.with_timezone(&ctx.timezone);
            let date = date.as_ref().map_or_else(
                || {
                    ctx.timezone
                        .now()
                        .date_naive()
                        .and_time(NaiveTime::default())
                        .and_local_timezone(ctx.timezone)
                        .unwrap() // *shudder* I think I can safely assume this won't fail
                },
                |d| {
                    d.0.and_time(NaiveTime::default())
                        .and_local_timezone(ctx.timezone)
                        .unwrap()
                },
            );
            let next_date = date + Duration::days(1);
            let total_datetime_ranges = record
                .clone()
                .try_into_cropped_datetime_ranges(ctx, date, next_date)?;
            let total_duration: chrono::Duration = total_datetime_ranges.iter().sum();
            println!(
                "Total time: {} hours, {} minutes",
                total_duration.num_hours(),
                total_duration.num_minutes() % 60
            );

            let mut hook = script_hook::Hook::name("day-stats")
                .env("PUNCH_CLOCK_DAY_TOTAL_DURATION", total_duration.to_string())
                .get_process();

            if let Some(mut stdin) = hook.take_stdin() {
                for e in total_datetime_ranges {
                    let start = e.start().to_rfc3339();
                    let end = e.end().to_rfc3339();
                    let val = serde_json::json!({
                        "start": start,
                        "end": end,
                    });
                    let _ = serde_json::to_writer(&mut stdin, &val);
                    let _ = stdin.write_all(b"\n");
                }
            }

            hook.run();

            let tr = record::display::time_range::time_range(
                &record,
                ctx.timezone.now(),
                date..=next_date,
                24 * resolution.as_hour_fraction(),
            )?;
            println!("{}", tr.print(6, "%R")?);
        }
        Action::Plugin { name, args } => {
            plugin::run(ctx, name, args, &record)?;
        }
    }

    Ok(())
}
