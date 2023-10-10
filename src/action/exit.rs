use chrono::Utc;

use crate::{error::Result, record::Record, time::human_readable_duration};

pub fn run(record: &mut Record<Utc>) -> Result<()> {
    let (clock_out_time, since) = record.clock_out()?;

    println!(
        "Clocked out on {} after {}",
        clock_out_time.with_timezone(&chrono::Local).format("%c"),
        human_readable_duration(&since)?
    );

    Ok(())
}
