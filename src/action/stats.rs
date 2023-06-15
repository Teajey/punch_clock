use chrono::Local;

use crate::{error::Result, record::Record, time::human_readable_duration};

pub fn run(record: Record<Local>) -> Result<()> {
    let record_duration = record.clone().total_time()?;
    println!("Total time: {}", human_readable_duration(&record_duration)?);
    let record_duration_today = record.todays_time()?;
    println!(
        "Total time today: {}",
        human_readable_duration(&record_duration_today)?
    );

    Ok(())
}
