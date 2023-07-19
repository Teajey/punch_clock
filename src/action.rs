mod dump;
mod edit;
mod enter;
mod exit;
mod stats;
mod status;

use std::fs;

use chrono::Utc;

use crate::{
    app::{
        cli::{Action, Day},
        context::Context,
    },
    error::Result,
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
            stats::run(ctx, record.clone().with_timezone(&ctx.timezone), date)?;
        }
        Action::Calendar {
            from: Day(from),
            to: Day(to),
            width,
        } => {
            record
                .with_timezone(&ctx.timezone)
                .paint_calendar(ctx, *from..=*to, *width)?;
        }
    };

    Ok(())
}
