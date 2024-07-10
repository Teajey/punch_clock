use chrono::Utc;

use crate::{
    error::Result, record::Record, script_hook, string::assert_no_newlines,
    time::human_readable_duration,
};

pub fn run(record: &mut Record<Utc>, comment: Option<String>, skip_hooks: bool) -> Result<()> {
    let comment = comment.map(assert_no_newlines).transpose()?;

    let (clock_out_time, since) = record.clock_out(comment)?;

    if !skip_hooks {
        script_hook::run("out")?;
    }

    println!(
        "Clocked out on {} after {}",
        clock_out_time.with_timezone(&chrono::Local).format("%c"),
        human_readable_duration(&since)?
    );

    Ok(())
}
