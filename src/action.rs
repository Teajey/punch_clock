mod dump;
mod edit;
mod enter;
mod exit;
mod stats;
mod status;
mod undo;

use std::fs;

use chrono::Utc;

use crate::{
    app::{
        cli::{Action, Day},
        context::Context,
    },
    error::{self, Result},
    record::Record,
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
        Action::Out => {
            exit::run(&mut record)?;
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
    };

    Ok(())
}
