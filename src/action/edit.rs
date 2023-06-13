use std::fs;

use chrono::Local;

use crate::{
    app::context,
    error::{self, Result},
};

pub fn run(ctx: &context::Base, record: &context::Record) -> Result<context::Record> {
    let record_string = record.serialize(&Local)?;
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

    context::Record::try_from(edited_record_string.as_str())
}
