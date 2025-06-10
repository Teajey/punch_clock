use std::{fmt::Display, iter::Sum, ops::RangeInclusive};

use chrono::{DateTime, Duration, NaiveDate, TimeZone};

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

    pub fn days_covered(&self) -> Vec<NaiveDate> {
        let start = self.start().date_naive();
        let end = self.end().date_naive();
        let diff = end - start;
        let num_days = usize::try_from(diff.num_days() + 1)
            .expect("number of days is positive and fits in usize");
        start.iter_days().take(num_days).collect()
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

#[cfg(test)]
mod test {
    use std::str::FromStr;

    use super::*;

    #[test]
    fn days_covered() {
        let start = chrono::DateTime::<chrono::Utc>::from_str("2020-01-01T00:00:00.000Z").unwrap();
        let end = chrono::DateTime::<chrono::Utc>::from_str("2020-01-04T00:00:00.000Z").unwrap();
        let range = DateTimeRange::new(start, end).unwrap();
        let dc = range.days_covered();
        assert_eq!(
            vec![
                chrono::NaiveDate::from_ymd_opt(2020, 1, 1).unwrap(),
                chrono::NaiveDate::from_ymd_opt(2020, 1, 2).unwrap(),
                chrono::NaiveDate::from_ymd_opt(2020, 1, 3).unwrap(),
                chrono::NaiveDate::from_ymd_opt(2020, 1, 4).unwrap(),
            ],
            dc
        );
    }
}
