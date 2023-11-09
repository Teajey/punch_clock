use chrono::Utc;

use crate::{error::Result, record::Record, time::human_readable_duration};

pub fn run(record: &mut Record<Utc>, comment: Option<String>) -> Result<()> {
    let (clock_out_time, since) = record.clock_out(comment)?;

    println!(
        "Clocked out on {} after {}",
        clock_out_time.with_timezone(&chrono::Local).format("%c"),
        human_readable_duration(&since)?
    );

    Ok(())
}
