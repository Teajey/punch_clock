mod dump;
mod edit;
mod enter;
mod exit;
mod stats;
mod status;
mod undo;

use std::fs;

use chrono::{Duration, NaiveTime, Utc};

use crate::{
    app::{
        cli::{Action, Day},
        context::Context,
    },
    error::{self, Result},
    record::{self, Record},
    time::ContextTimeZone,
};

pub fn run<Tz: ContextTimeZone>(
    ctx: &Context<Tz>,
    action: &Action,
    mut record: Record<Utc>,
) -> Result<()> {
    match action {
        Action::In => {
            enter::run(&mut record)?;
            fs::write(".punch_clock/record", record.serialize()?)?;
        }
        Action::Out { comment } => {
            exit::run(&mut record, comment.as_deref().map(ToOwned::to_owned))?;
            fs::write(".punch_clock/record", record.serialize()?)?;
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
        Action::Calendar { from, to, width } => {
            let (from, to) = match (from.as_ref(), to.as_ref()) {
                (None, Some(_)) => unreachable!(),
                (None, None) => {
                    let to = chrono::Local::now().date_naive();
                    let from = to
                        .checked_sub_days(chrono::Days::new(6))
                        .ok_or(error::Main::DateOutOfRange)?;
                    (from, to)
                }
                (Some(to), None) => {
                    let to = to.0;
                    let from = to
                        .checked_sub_days(chrono::Days::new(6))
                        .ok_or(error::Main::DateOutOfRange)?;
                    (from, to)
                }
                (Some(from), Some(to)) => (from.0, to.0),
            };
            record
                .with_timezone(&ctx.timezone)
                .paint_calendar(ctx, from..=to, *width)?;
        }
        Action::Undo => {
            undo::run(&mut record)?;
            fs::write(".punch_clock/record", record.serialize()?)?;
        }
        Action::Day { date, scale } => {
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
            let total_duration: chrono::Duration = total_datetime_ranges.into_iter().sum();
            println!(
                "Total time: {} hours, {} minutes",
                total_duration.num_hours(),
                total_duration.num_minutes() % 60
            );

            let tr =
                record::display::time_range::time_range(&record, date..=next_date, 24 * scale)?;
            println!("{}", tr.print(6, "%R")?);
        }
    };

    Ok(())
}
