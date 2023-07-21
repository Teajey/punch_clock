use chrono::Utc;

use crate::{error::Result, record::Record, time::human_readable_duration};

pub fn run(record: &mut Record<Utc>) -> Result<()> {
    let since = record.clock_out()?;

    println!("Clocked out after {}", human_readable_duration(&since)?);

    Ok(())
}
