use chrono::Utc;

use crate::{error::Result, record::Record};

pub fn run(record: &mut Record<Utc>) -> Result<()> {
    let clock_in_time = record.clock_in()?;

    println!(
        "Clocked in on {}",
        clock_in_time.with_timezone(&chrono::Local).format("%c")
    );

    Ok(())
}
