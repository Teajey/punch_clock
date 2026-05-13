use chrono::Utc;
use punch_clock_core::{error::Result, record::Record, time::human_readable_duration};

use crate::string::assert_no_newlines;

pub fn run(record: &mut Record<Utc>, comment: Option<String>) -> Result<()> {
    let comment = comment.map(assert_no_newlines).transpose()?;

    let (clock_out_time, since) = record.clock_out(comment)?;

    println!(
        "Clocking out on {} after {}",
        clock_out_time.with_timezone(&chrono::Local).format("%c"),
        human_readable_duration(&since)?
    );

    Ok(())
}
