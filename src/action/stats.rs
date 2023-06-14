use crate::{error::Result, record::Record, time::human_readable_duration};

pub fn run(record: Record) -> Result<()> {
    let record_duration = record.total_time()?;
    println!("Total time: {}", human_readable_duration(&record_duration)?);

    Ok(())
}
