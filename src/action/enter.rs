use chrono::Utc;

use crate::{error::Result, record::Record, script_hook, string::assert_no_newlines};

pub fn run(record: &mut Record<Utc>, comment: Option<String>, skip_hooks: bool) -> Result<()> {
    let comment = comment.map(assert_no_newlines).transpose()?;

    let clock_in_time = record.clock_in(comment)?;

    if !skip_hooks {
        script_hook::run("in")?;
    }

    println!(
        "Clocked in on {}",
        clock_in_time.with_timezone(&chrono::Local).format("%c")
    );

    Ok(())
}
