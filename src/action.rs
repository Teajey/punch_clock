mod dump;
mod edit;
mod enter;
mod exit;
mod stats;
mod status;

use std::fs;

use chrono::{Local, Utc};

use crate::{
    app::{cli::Action, context},
    error::Result,
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
        Action::Dump => dump::run(&record.clone().with_timezone(&Local))?,
        Action::Edit => {
            let record = edit::run(ctx, record)?;
            fs::write(".punch_clock/record", record.serialize()?)?;
        }
        Action::Stats => stats::run(record.clone().with_timezone(&Local))?,
    };

    Ok(())
}
