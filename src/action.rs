mod dump;
mod edit;
mod enter;
mod exit;
mod status;

use crate::{
    app::{cli::Action, context},
    error::Result,
};

pub fn run(
    ctx: &context::Base,
    action: &Action,
    mut record: context::Record,
) -> Result<context::Record> {
    match action {
        Action::In => enter::run(&mut record)?,
        Action::Out => exit::run(&mut record)?,
        Action::Status => status::run(&record)?,
        Action::Dump => dump::run(&record)?,
        Action::Edit => return edit::run(ctx, &record),
    };

    Ok(record)
}
