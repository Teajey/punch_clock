use chrono::NaiveDate;

use crate::{
    app::context,
    error::Result,
    record::Record,
    time::{human_readable_duration, ContextTimeZone},
};

pub fn run<Tz: ContextTimeZone>(
    ctx: &context::Context<Tz>,
    record: Record<Tz>,
    date: Option<NaiveDate>,
) -> Result<()> {
    if let Some(date) = date {
        let record_duration = record.days_time(ctx, date)?;
        println!(
            "Total time for given day: {}",
            human_readable_duration(&record_duration)?
        );
    } else {
        let record_duration = record.clone().total_time(ctx)?;
        println!("Total time: {}", human_readable_duration(&record_duration)?);
        let record_duration_today = record.todays_time(ctx)?;
        println!(
            "Total time today: {}",
            human_readable_duration(&record_duration_today)?
        );
    }

    Ok(())
}
