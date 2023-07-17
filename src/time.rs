use std::{
    fmt::{self, Display, Write},
    ops::RangeInclusive,
};

use chrono::{DateTime, Days, Duration};

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

pub fn day_timespan<Tz: UnfixedTimeZone>(
    day: chrono::NaiveDate,
) -> Result<RangeInclusive<DateTime<Tz>>> {
    let day_start = day.into_day_start();
    let day_end = day.into_day_end()?;

    Ok(day_start..=day_end)
}

pub trait UnfixedTimeZone: chrono::TimeZone<Offset = <Self as UnfixedTimeZone>::Offset> {
    type Offset: Copy;

    fn new() -> Self;
}

impl UnfixedTimeZone for chrono::Utc {
    type Offset = <Self as chrono::TimeZone>::Offset;

    fn new() -> Self {
        chrono::Utc
    }
}

impl UnfixedTimeZone for chrono::Local {
    type Offset = <Self as chrono::TimeZone>::Offset;

    fn new() -> Self {
        chrono::Local
    }
}

pub trait NaiveDateOperations {
    fn into_day_start<Tz: UnfixedTimeZone>(self) -> DateTime<Tz>;
    fn into_day_end<Tz: UnfixedTimeZone>(self) -> Result<DateTime<Tz>>;
}

impl NaiveDateOperations for chrono::NaiveDate {
    fn into_day_start<Tz: UnfixedTimeZone>(self) -> DateTime<Tz> {
        match self
            .and_time(chrono::NaiveTime::MIN)
            .and_local_timezone(Tz::new())
        {
            chrono::LocalResult::Single(dt) => dt,
            chrono::LocalResult::None | chrono::LocalResult::Ambiguous(_, _) => {
                todo!("I don't know how to handle this atm")
            }
        }
    }

    fn into_day_end<Tz: UnfixedTimeZone>(self) -> Result<DateTime<Tz>> {
        self.into_day_start()
            .checked_add_days(Days::new(1))
            .ok_or(error::Main::DateOutOfRange)?
            .checked_sub_signed(Duration::nanoseconds(1))
            .ok_or(error::Main::DateOutOfRange)
    }
}
