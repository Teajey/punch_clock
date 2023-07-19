use std::{
    fmt::{self, Display, Write},
    ops::RangeInclusive,
};

use chrono::{DateTime, Days, Duration, TimeZone};

use crate::{
    app::context::Context,
    error::{self, Result},
};

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

pub fn day_timespan<Tz: ContextTimeZone>(
    ctx: &Context<Tz>,
    day: chrono::NaiveDate,
) -> Result<RangeInclusive<DateTime<Tz>>> {
    let day_start = day.into_day_start(ctx)?;
    let day_end = day.into_day_end(ctx)?;

    Ok(day_start..=day_end)
}

pub trait NaiveDateOperations {
    fn into_day_start<Tz: ContextTimeZone>(self, ctx: &Context<Tz>) -> Result<DateTime<Tz>>;
    fn into_day_end<Tz: ContextTimeZone>(self, ctx: &Context<Tz>) -> Result<DateTime<Tz>>;
}

pub trait ContextTimeZone: TimeZone<Offset = <Self as ContextTimeZone>::Offset> + Copy {
    type Offset: Copy;

    fn now(&self) -> DateTime<Self>;
}

impl ContextTimeZone for chrono::Local {
    type Offset = <Self as TimeZone>::Offset;

    fn now(&self) -> DateTime<Self> {
        chrono::Local::now()
    }
}

impl ContextTimeZone for chrono::FixedOffset {
    type Offset = <Self as TimeZone>::Offset;

    fn now(&self) -> DateTime<Self> {
        chrono::Utc::now().with_timezone(self)
    }
}

impl NaiveDateOperations for chrono::NaiveDate {
    fn into_day_start<Tz: ContextTimeZone>(self, ctx: &Context<Tz>) -> Result<DateTime<Tz>> {
        match self
            .and_time(chrono::NaiveTime::MIN)
            .and_local_timezone(ctx.timezone)
        {
            chrono::LocalResult::Single(dt) => Ok(dt),
            chrono::LocalResult::None | chrono::LocalResult::Ambiguous(_, _) => {
                todo!("I don't know how to handle this atm")
            }
        }
    }

    fn into_day_end<Tz: ContextTimeZone>(self, ctx: &Context<Tz>) -> Result<DateTime<Tz>> {
        self.into_day_start(ctx)?
            .checked_add_days(Days::new(1))
            .ok_or(error::Main::DateOutOfRange)?
            .checked_sub_signed(Duration::nanoseconds(1))
            .ok_or(error::Main::DateOutOfRange)
    }
}
