use std::fs;

use chrono::{Local, Utc};

use crate::{
    app::context::Context,
    error::{self, Result},
    record::Record,
    time::ContextTimeZone,
};

pub fn run<Tz: ContextTimeZone>(ctx: &Context<Tz>, record: Record<Utc>) -> Result<Record<Utc>> {
    let record_string = record.with_timezone(&Local).serialize()?;
    let edit_path = ".punch_clock/EDIT_RECORD";

    fs::write(edit_path, record_string)?;

    if !std::process::Command::new(&ctx.editor_path)
        .arg(edit_path)
        .status()?
        .success()
    {
        return Err(error::Main::UnsuccessfulEditor);
    }

    let edited_record_string = fs::read_to_string(edit_path)?;

    Ok(Record::try_from(edited_record_string.as_str())?.with_timezone(&Utc))
}
