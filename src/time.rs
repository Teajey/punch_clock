use std::fmt::{self, Display, Write};

use chrono::{DateTime, Duration, Local, NaiveDate, NaiveTime};

use crate::error::Result;

pub fn human_readable_duration(duration: &Duration) -> Result<String, fmt::Error> {
    let mut buf = String::new();

    let days = duration.num_days().abs();
    let hours = duration.num_hours().abs();
    let minutes = duration.num_minutes().abs();

    if days > 0 {
        write!(
            buf,
            "{days} days, {} hours, {} minutes",
            hours % 24,
            minutes % 60
        )?;
    } else if hours > 0 {
        write!(buf, "{} hours, {} minutes", hours % 24, minutes % 60)?;
    } else {
        write!(buf, "{} minutes", minutes % 60)?;
    }

    Ok(buf)
}

#[derive(Debug)]
pub struct Ago(pub chrono::Duration);

impl Display for Ago {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", human_readable_duration(&self.0)?)?;
        if self.0 < Duration::zero() {
            write!(f, " ago")
        } else {
            write!(f, " from now")
        }
    }
}

pub fn naive_date_into_local_datetime(date: NaiveDate) -> Result<DateTime<Local>> {
    match date
        .and_time(NaiveTime::default())
        .and_local_timezone(Local)
    {
        chrono::LocalResult::Single(dt) => Ok(dt),
        chrono::LocalResult::Ambiguous(_, _) | chrono::LocalResult::None => {
            todo!("I'm not yet sure how to handle this error")
        }
    }
}
