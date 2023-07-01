mod dump;
mod edit;
mod enter;
mod exit;
mod stats;
mod status;

use std::fs;

use chrono::{FixedOffset, Local, NaiveDate, Utc};

use crate::{
    app::{
        cli::{Action, Day},
        context,
    },
    error::{self, Result},
    record::Record,
};

pub fn run(ctx: &context::Base, action: &Action, mut record: Record<Utc>) -> Result<()> {
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
        Action::Dump { offset } => {
            if let Some(offset) = offset {
                let hour = 3600;
                let timezone = FixedOffset::east_opt(offset * hour)
                    .ok_or_else(|| error::Main::TimezoneOutOfRange(*offset))?;
                dump::run(&record.clone().with_timezone(&timezone))?;
            } else {
                dump::run(&record.clone().with_timezone(&Local))?;
            }
        }
        Action::Edit => {
            let record = edit::run(ctx, record)?;
            fs::write(".punch_clock/record", record.serialize()?)?;
        }
        Action::Stats(Day { day, month, year }) => {
            let date = match (day, month, year) {
                (Some(day), Some(month), Some(year)) => Some((day, month, year)),
                (None, None, None) => None,
                _ => unreachable!("Day, month, year arguments MUST be provided together"),
            };
            let date = date
                .map(|(day, month, year)| {
                    NaiveDate::from_ymd_opt(*year, *month, *day).ok_or(error::Main::DateOutOfRange)
                })
                .transpose()?;
            stats::run(record.clone().with_timezone(&Local), date)?;
        }
    };

    Ok(())
}
