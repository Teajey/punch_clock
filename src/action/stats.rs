use chrono::{Local, NaiveDate};

use crate::{error::Result, record::Record, time::human_readable_duration};

pub fn run(record: Record<Local>, date: Option<NaiveDate>) -> Result<()> {
    if let Some(date) = date {
        let record_duration = record.days_time(date)?;
        println!(
            "Total time for given day: {}",
            human_readable_duration(&record_duration)?
        );
    } else {
        let record_duration = record.clone().total_time()?;
        println!("Total time: {}", human_readable_duration(&record_duration)?);
        let record_duration_today = record.todays_time()?;
        println!(
            "Total time today: {}",
            human_readable_duration(&record_duration_today)?
        );
    }

    Ok(())
}
