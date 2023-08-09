use std::{fmt::Display, iter::Sum, ops::RangeInclusive};

use chrono::{DateTime, Duration, TimeZone};

use crate::{
    error::{self, Result},
    range::Span,
    record::Entry,
};

#[derive(Clone)]
#[allow(clippy::module_name_repetitions)]
pub struct DateTimeRange<Tz: TimeZone>(RangeInclusive<DateTime<Tz>>);

impl<Tz: TimeZone> Span<Duration> for DateTimeRange<Tz>
where
    Tz::Offset: Copy,
{
    type Output = Duration;

    fn span(&self) -> Self::Output {
        *self.0.end() - *self.0.start()
    }
}

impl<Tz: TimeZone> DateTimeRange<Tz>
where
    Tz::Offset: Display,
{
    pub fn new(start: DateTime<Tz>, end: DateTime<Tz>) -> Result<Self> {
        if start >= end {
            return Err(error::Main::DateTimeRangeEndBeforeStart {
                start: start.to_rfc3339(),
                end: end.to_rfc3339(),
            });
        }
        Ok(Self(start..=end))
    }
}

impl<Tz: TimeZone> DateTimeRange<Tz> {
    pub fn into_bounds(self) -> (DateTime<Tz>, DateTime<Tz>) {
        self.0.into_inner()
    }

    pub fn start(&self) -> &DateTime<Tz> {
        self.0.start()
    }

    pub fn end(&self) -> &DateTime<Tz> {
        self.0.end()
    }
}

impl<Tz: TimeZone> From<Entry<Tz>> for DateTimeRange<Tz> {
    fn from(entry: Entry<Tz>) -> Self {
        let check_out = entry.check_in.clone() + entry.get_work_duration();

        Self(entry.check_in..=check_out)
    }
}

impl<Tz: TimeZone> Sum<DateTimeRange<Tz>> for Duration
where
    Tz::Offset: Copy,
{
    fn sum<I: Iterator<Item = DateTimeRange<Tz>>>(iter: I) -> Self {
        iter.fold(Duration::zero(), |duration, dtr| duration + dtr.span())
    }
}
