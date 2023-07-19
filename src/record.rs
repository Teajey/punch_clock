mod display;

use std::{
    collections::VecDeque,
    fmt::{Display, Write},
    fs,
    ops::RangeInclusive,
};

use chrono::{DateTime, Duration, FixedOffset, Local, NaiveDate, TimeZone, Utc};
use context::Context;

use crate::{
    app::context,
    error::{self, Result},
    time::{ContextTimeZone, NaiveDateOperations},
};

#[derive(Clone)]
#[cfg_attr(test, derive(Debug, PartialEq, Eq))]
pub struct Entry<Tz: TimeZone> {
    pub check_in: DateTime<Tz>,
    work_time_millis: u32,
}

impl<Tz: TimeZone> Entry<Tz> {
    pub fn try_new(check_in: DateTime<Tz>, check_out: DateTime<Tz>) -> Result<Self> {
        if check_out < check_in {
            return Err(error::Main::CheckOutBeforeCheckIn);
        }

        let work_time_millis = check_out
            .signed_duration_since(check_in.clone())
            .num_milliseconds()
            .try_into()?;

        Ok(Self {
            check_in,
            work_time_millis,
        })
    }

    pub fn with_timezone<Tz2: TimeZone>(self, tz: &Tz2) -> Entry<Tz2> {
        let Self {
            check_in,
            work_time_millis,
        } = self;
        Entry {
            check_in: check_in.with_timezone(tz),
            work_time_millis,
        }
    }

    fn get_work_time(&self) -> Duration {
        Duration::milliseconds(self.work_time_millis.into())
    }

    pub fn get_check_out(&self) -> Result<DateTime<Tz>> {
        self.check_in
            .clone()
            .checked_add_signed(self.get_work_time())
            .ok_or_else(|| error::Main::DateTimeOverflow)
    }

    fn into_date_pair(self) -> (DateTime<Tz>, DateTime<Tz>) {
        let check_out = self.check_in.clone() + self.get_work_time();

        (self.check_in, check_out)
    }
}

impl Entry<FixedOffset> {
    fn from_tokens(check_in: &str, check_out: &str) -> Result<Entry<FixedOffset>> {
        let check_in = DateTime::parse_from_rfc3339(check_in)?;
        let check_out = DateTime::parse_from_rfc3339(check_out)?;

        Entry::try_new(check_in, check_out)
    }
}

fn split_sparse_tokens(line: &str, pat: char) -> Vec<&str> {
    line.split(pat)
        .map(str::trim)
        .filter(|t| !t.is_empty())
        .collect()
}

impl TryFrom<&str> for Entry<FixedOffset> {
    type Error = error::Main;

    fn try_from(value: &str) -> std::result::Result<Self, Self::Error> {
        let [check_in, check_out] = split_sparse_tokens(value, ' ')[..] else {
            return Err(error::Main::EntryIncorrectNumberOfTokens);
        };

        Self::from_tokens(check_in, check_out)
    }
}

pub enum Latest<'a, Tz: TimeZone> {
    Entry(&'a Entry<Tz>),
    Current(&'a DateTime<Tz>),
    None,
}

#[derive(Clone)]
#[cfg_attr(test, derive(Debug))]
pub struct Record<Tz: TimeZone> {
    entries: Vec<Entry<Tz>>,
    current_session: Option<DateTime<Tz>>,
}

impl<Tz: TimeZone> Record<Tz> {
    pub fn with_timezone<Tz2: TimeZone>(self, tz: &Tz2) -> Record<Tz2> {
        let Self {
            entries,
            current_session,
        } = self;
        let entries = entries
            .into_iter()
            .map(|e| e.with_timezone(tz))
            .collect::<Vec<_>>();
        Record {
            entries,
            current_session: current_session.map(|cs| cs.with_timezone(tz)),
        }
    }

    pub fn get_current_session(&self) -> Option<&DateTime<Tz>> {
        self.current_session.as_ref()
    }

    pub fn get_entries(&self) -> &[Entry<Tz>] {
        &self.entries
    }

    pub fn get_latest(&self) -> Latest<'_, Tz> {
        match (&self.current_session, self.entries.last()) {
            (None, None) => Latest::None,
            (None, Some(last_entry)) => Latest::Entry(last_entry),
            (Some(current_session), _) => Latest::Current(current_session),
        }
    }

    pub fn serialize(&self) -> Result<String>
    where
        Tz::Offset: Display,
    {
        let mut buf = String::new();
        for entry in &self.entries {
            writeln!(
                buf,
                "{:<32} {:<32}",
                entry.check_in.to_rfc3339(),
                entry.get_check_out()?.to_rfc3339(),
            )?;
        }

        if let Some(current_session) = self.current_session.clone() {
            writeln!(buf, "{}", current_session.to_rfc3339())?;
        }

        Ok(buf)
    }

    pub fn sum_datetime_pairs(pairs: Vec<(DateTime<Tz>, DateTime<Tz>)>) -> Duration {
        let seconds = pairs
            .into_iter()
            .map(|(check_in, check_out)| check_out.signed_duration_since(check_in))
            .map(|duration| duration.num_seconds())
            .sum::<i64>();

        Duration::seconds(seconds)
    }
}

pub struct Iterator<Tz: TimeZone> {
    entries: std::iter::Rev<std::vec::IntoIter<Entry<Tz>>>,
    current_session: std::option::IntoIter<DateTime<Tz>>,
}

#[cfg_attr(test, derive(Debug, PartialEq, Eq))]
pub enum Item<Tz: TimeZone> {
    Entry(Entry<Tz>),
    CurrentSession(DateTime<Tz>),
}

impl<Tz: TimeZone> Item<Tz> {
    pub fn into_entry<F>(self, end: F) -> Result<Entry<Tz>>
    where
        F: FnOnce() -> DateTime<Tz>,
    {
        match self {
            Item::Entry(entry) => Ok(entry),
            Item::CurrentSession(current_session) => Entry::try_new(current_session, end()),
        }
    }
}

impl<Tz: TimeZone> std::iter::Iterator for Iterator<Tz> {
    type Item = Item<Tz>;

    fn next(&mut self) -> Option<Self::Item> {
        self.entries
            .next_back()
            .map(Item::Entry)
            .or_else(|| self.current_session.next().map(Item::CurrentSession))
    }
}

impl<Tz: TimeZone> DoubleEndedIterator for Iterator<Tz> {
    fn next_back(&mut self) -> Option<Self::Item> {
        self.current_session
            .next_back()
            .map(Item::CurrentSession)
            .or_else(|| self.entries.next().map(Item::Entry))
    }
}

impl<Tz: TimeZone> IntoIterator for Record<Tz> {
    type Item = Item<Tz>;

    type IntoIter = Iterator<Tz>;

    fn into_iter(self) -> Self::IntoIter {
        Self::IntoIter {
            entries: self.entries.into_iter().rev(),
            current_session: self.current_session.into_iter(),
        }
    }
}

impl<Tz: ContextTimeZone> Record<Tz> {
    pub fn paint_calendar(
        &self,
        ctx: &context::Context<Tz>,
        range: RangeInclusive<NaiveDate>,
        width: usize,
    ) -> Result<()> {
        display::paint_day_range(ctx, self, range, width)?;
        Ok(())
    }

    pub fn days_time(self, ctx: &context::Context<Tz>, day: NaiveDate) -> Result<Duration> {
        let date_pairs = self.try_into_cropped_datetime_pairs(
            ctx,
            day.into_day_start(ctx)?,
            day.into_day_end(ctx)?,
        )?;

        Ok(Self::sum_datetime_pairs(date_pairs))
    }

    pub fn todays_time(self, ctx: &context::Context<Tz>) -> Result<Duration> {
        let now = Local::now();
        let today = now.date_naive();

        let mut date_pairs_today = self
            .into_iter()
            .map(|item| {
                item.into_entry(|| ctx.timezone.now())
                    .map(Entry::into_date_pair)
            })
            .rev()
            .take_while(|entry| {
                entry
                    .iter()
                    .any(|(_, check_out)| check_out.date_naive() == today)
            })
            .collect::<Result<Vec<_>>>()?;

        let Some((first_check_in, first_check_out)) = date_pairs_today.pop() else {
            return Ok(Duration::zero());
        };

        let first_check_in = if first_check_in.date_naive() == today {
            first_check_in
        } else {
            today.into_day_start(ctx)?
        };

        date_pairs_today.push((first_check_in, first_check_out));

        Ok(Self::sum_datetime_pairs(date_pairs_today))
    }

    pub fn total_time(self, ctx: &context::Context<Tz>) -> Result<Duration> {
        let datetime_pairs = self
            .into_iter()
            .map(|entry| {
                entry
                    .into_entry(|| ctx.timezone.now())
                    .map(Entry::into_date_pair)
            })
            .collect::<Result<Vec<_>>>()?;

        Ok(Self::sum_datetime_pairs(datetime_pairs))
    }

    fn try_into_cropped_datetime_pairs(
        self,
        ctx: &Context<Tz>,
        start: DateTime<Tz>,
        end: DateTime<Tz>,
    ) -> Result<Vec<(DateTime<Tz>, DateTime<Tz>)>> {
        if end <= start {
            return Err(error::Main::RangeStartPosition);
        }

        let mut date_pairs = self
            .into_iter()
            .map(|item| {
                item.into_entry(|| ctx.timezone.now().min(end))
                    .map(Entry::into_date_pair)
            })
            .filter(|entry| {
                entry.iter().any(|(check_in, check_out)| {
                    (start <= *check_in && *check_in < end)
                        || (start <= *check_out && *check_out < end)
                })
            })
            .collect::<Result<VecDeque<_>>>()?;

        if date_pairs.is_empty() {
            return Ok(vec![]);
        }

        let (first_check_in, first_check_out) = date_pairs
            .pop_front()
            .expect("date_pairs must be confirmed to have at least one element");

        let first_check_in = if first_check_in < start {
            start
        } else {
            first_check_in
        };

        date_pairs.push_front((first_check_in, first_check_out));

        let (last_check_in, last_check_out) = date_pairs
            .pop_back()
            .expect("date_pairs must be confirmed to have at least one element");

        let last_check_out = if last_check_out > end {
            end
        } else {
            last_check_out
        };

        date_pairs.push_back((last_check_in, last_check_out));

        Ok(date_pairs.into())
    }
}

impl Record<Utc> {
    pub fn clock_in(&mut self) -> Result<()> {
        if self.current_session.is_some() {
            return Err(error::Main::AlreadyClockedIn);
        };

        self.current_session = Some(Utc::now());

        Ok(())
    }

    pub fn clock_out(&mut self) -> Result<()> {
        let Some(current_session) = self.current_session else {
            return Err(error::Main::NotClockedIn);
        };

        self.entries
            .push(Entry::try_new(current_session, Utc::now())?);

        self.current_session = None;

        Ok(())
    }

    pub fn load() -> Result<Option<Self>> {
        if !std::path::Path::new(".punch_clock").exists() {
            return Ok(None);
        }

        let record: Record<FixedOffset> = fs::read_to_string(".punch_clock/record")?
            .as_str()
            .try_into()?;

        let record = record.with_timezone(&Utc);

        Ok(Some(record))
    }

    pub fn init() -> Result<()> {
        if Self::load()?.is_some() {
            return Err(error::Main::AlreadyInitialized);
        }

        fs::create_dir(".punch_clock")?;
        fs::write(".punch_clock/record", "")?;

        Ok(())
    }
}

impl TryFrom<&str> for Record<FixedOffset> {
    type Error = error::Main;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        let mut current_session = None;

        let mut lines = split_sparse_tokens(value, '\n');

        let last_line = lines.pop();

        let mut entries = lines
            .into_iter()
            .map(Entry::try_from)
            .collect::<Result<Vec<_>>>()?;

        if let Some(last_line) = last_line {
            match split_sparse_tokens(last_line, ' ')[..] {
                [check_in, check_out] => {
                    entries.push(Entry::from_tokens(check_in, check_out)?);
                }
                [check_in] => {
                    current_session = Some(DateTime::parse_from_rfc3339(check_in)?);
                }
                _ => {
                    return Err(error::Main::EntryIncorrectNumberOfTokens);
                }
            }
        }

        Ok(Self {
            entries,
            current_session,
        })
    }
}

#[cfg(test)]
mod test {
    use chrono::{DateTime, FixedOffset, TimeZone};
    use pretty_assertions::assert_eq;

    use super::{display::paint_day_range, Record};
    use crate::{
        app::context,
        record::{self, Entry},
    };

    fn date_md(month: u32, day: u32) -> chrono::NaiveDate {
        chrono::NaiveDate::from_ymd_opt(2023, month, day).unwrap()
    }

    fn datetime_hm(hour: u32, min: u32) -> DateTime<FixedOffset> {
        FixedOffset::west_opt(0)
            .unwrap()
            .from_utc_datetime(&chrono::NaiveDateTime::new(
                chrono::NaiveDate::from_ymd_opt(2023, 1, 1).unwrap(),
                chrono::NaiveTime::from_hms_opt(hour, min, 0).unwrap(),
            ))
    }

    fn get_record() -> Record<FixedOffset> {
        let rec_file = "2023-01-01T00:00:00.000000+00:00 2023-01-01T01:00:00.000000+00:00
2023-01-01T02:00:00.000000+00:00 2023-01-01T03:00:00.000000+00:00
2023-01-01T04:00:00.000000+00:00";
        Record::try_from(rec_file).unwrap()
    }

    #[test]
    fn iterator() {
        let rec = get_record();
        let rec_vec = rec.into_iter().collect::<Vec<_>>();

        assert_eq!(
            vec![
                record::Item::Entry(Entry {
                    check_in: datetime_hm(0, 0),
                    work_time_millis: 3_600_000
                }),
                record::Item::Entry(Entry {
                    check_in: datetime_hm(2, 0),
                    work_time_millis: 3_600_000
                }),
                record::Item::CurrentSession(datetime_hm(4, 0)),
            ],
            rec_vec
        );
    }

    #[test]
    fn iterator_rev() {
        let rec = get_record();
        let rec_vec = rec.into_iter().rev().collect::<Vec<_>>();

        assert_eq!(
            vec![
                record::Item::CurrentSession(datetime_hm(4, 0)),
                record::Item::Entry(Entry {
                    check_in: datetime_hm(2, 0),
                    work_time_millis: 3_600_000
                }),
                record::Item::Entry(Entry {
                    check_in: datetime_hm(0, 0),
                    work_time_millis: 3_600_000
                }),
            ],
            rec_vec
        );
    }

    #[test]
    fn range_end_index_x_out_of_range_for_slice_of_length_y() {
        let ctx = context::Context {
            editor_path: String::new(),
            timezone: FixedOffset::east_opt(0).unwrap(),
        };
        let rec_file = "2023-07-10T05:05:42.372091+00:00 2023-07-10T09:38:44.320091+00:00
2023-07-10T20:00:00+00:00        2023-07-10T22:13:34.369+00:00";
        let rec = Record::try_from(rec_file)
            .unwrap()
            .with_timezone(&ctx.timezone);
        paint_day_range(&ctx, &rec, date_md(7, 9)..=date_md(7, 10), 48).unwrap();
    }

    #[test]
    fn range_end_index_x_out_of_range_for_slice_of_length_y_2() {
        let ctx = context::Context {
            editor_path: String::new(),
            timezone: FixedOffset::east_opt(0).unwrap(),
        };
        let rec_file = "2023-06-04T21:08:34.790590+00:00 2023-06-04T22:32:47.660590+00:00
2023-06-05T04:30:04.199633+00:00 2023-06-05T07:18:50.734633+00:00";
        let rec = Record::try_from(rec_file)
            .unwrap()
            .with_timezone(&ctx.timezone);
        paint_day_range(&ctx, &rec, date_md(6, 4)..=date_md(6, 5), 48).unwrap();
    }

    #[test]
    fn range_end_index_x_out_of_range_for_slice_of_length_y_3() {
        let ctx = context::Context {
            editor_path: String::new(),
            timezone: FixedOffset::east_opt(12 * 3600).unwrap(),
        };
        let rec_file = "2023-06-30T04:30:00.893153+00:00 2023-06-30T07:15:07.931153+00:00
2023-07-10T05:05:42.372091+00:00 2023-07-10T09:38:44.320091+00:00
2023-07-10T20:00:00+00:00        2023-07-10T22:13:34.369+00:00   
2023-07-11T04:30:55.569838+00:00 2023-07-11T05:05:55.569838+00:00
2023-07-11T09:01:20.726248+00:00 2023-07-11T12:08:36.149248+00:00
2023-07-11T12:32:28.616529+00:00 2023-07-11T14:27:00.836529+00:00
2023-07-11T20:03:53.114039+00:00 2023-07-11T22:41:25.885039+00:00
2023-07-12T04:30:00+00:00        2023-07-12T04:54:25.885+00:00   
2023-07-12T09:30:00+00:00        2023-07-12T13:00:00+00:00       
2023-07-12T22:04:34.947469+00:00 2023-07-12T23:29:44.706469+00:00
2023-07-13T09:08:38.290767+00:00 2023-07-13T10:34:50.199767+00:00";
        let rec = Record::try_from(rec_file)
            .unwrap()
            .with_timezone(&ctx.timezone);
        paint_day_range(&ctx, &rec, date_md(7, 10)..=date_md(7, 12), 24).unwrap();
    }
}
