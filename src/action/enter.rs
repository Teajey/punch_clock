use chrono::Utc;

use crate::{error::Result, record::Record};

pub fn run(record: &mut Record<Utc>) -> Result<()> {
    let clock_in_time = record.clock_in()?;

    println!("Clocked in at {}", clock_in_time.format("%c"));

    Ok(())
}
