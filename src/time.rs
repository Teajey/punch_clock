use std::{
    fmt::{self, Display, Write},
    ops::RangeInclusive,
};

use chrono::{DateTime, Days, Duration, Local, NaiveDate, NaiveTime};

use crate::error::{self, Result};

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

fn combine_and_localize_date_and_time(date: NaiveDate, time: NaiveTime) -> DateTime<Local> {
    match date.and_time(time).and_local_timezone(Local) {
        chrono::LocalResult::Single(dt) => dt,
        chrono::LocalResult::Ambiguous(_, _) | chrono::LocalResult::None => {
            todo!("I'm not yet sure how to handle this error")
        }
    }
}

pub fn naive_date_into_local_datetime(date: NaiveDate) -> DateTime<Local> {
    combine_and_localize_date_and_time(date, NaiveTime::default())
}

pub fn naive_date_into_local_datetime_end(date: NaiveDate) -> Result<DateTime<Local>> {
    let dt = combine_and_localize_date_and_time(
        date,
        NaiveTime::from_hms_milli_opt(23, 59, 59, 999)
            .ok_or_else(|| error::Main::DateOutOfRange)?,
    );

    Ok(dt)
}

pub fn day_timespan(day: NaiveDate) -> Result<RangeInclusive<DateTime<Local>>> {
    let day_start = naive_date_into_local_datetime(day);

    let day_end = day_start
        .checked_add_days(Days::new(1))
        .ok_or(error::Main::DateOutOfRange)?
        .checked_sub_signed(Duration::milliseconds(1))
        .ok_or(error::Main::DateOutOfRange)?;

    Ok(day_start..=day_end)
}
