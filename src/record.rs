pub mod display;

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
    time::{range::DateTimeRange, ContextTimeZone, NaiveDateOperations},
};

// FIXME: I'm thinking Entry ought to just be completely replaced by DateTimeRange
#[derive(Clone)]
#[cfg_attr(test, derive(Debug, PartialEq, Eq))]
pub struct Entry<Tz: TimeZone> {
    pub check_in: DateTime<Tz>,
    work_time_millis: u32,
    pub in_comment: Option<String>,
    pub out_comment: Option<String>,
}

impl<Tz: TimeZone> Entry<Tz> {
    pub fn try_new(
        check_in: DateTime<Tz>,
        check_out: DateTime<Tz>,
        in_comment: Option<String>,
        out_comment: Option<String>,
    ) -> Result<Self> {
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
            in_comment,
            out_comment,
        })
    }

    pub fn with_timezone<Tz2: TimeZone>(self, tz: &Tz2) -> Entry<Tz2> {
        let Self {
            check_in,
            work_time_millis,
            in_comment,
            out_comment,
        } = self;
        Entry {
            check_in: check_in.with_timezone(tz),
            work_time_millis,
            in_comment,
            out_comment,
        }
    }

    pub fn get_work_duration(&self) -> Duration {
        Duration::milliseconds(self.work_time_millis.into())
    }

    pub fn get_check_out(&self) -> Result<DateTime<Tz>> {
        // FIXME: Why not just `self.check_in + self.get_work_duration()`?
        self.check_in
            .clone()
            .checked_add_signed(self.get_work_duration())
            .ok_or_else(|| error::Main::DateTimeOverflow)
    }
}

impl Entry<FixedOffset> {
    fn from_lines(check_in_line: &str, check_out_line: &str) -> Result<Entry<FixedOffset>> {
        let check_in_parts = split_sparse_tokens(check_in_line, ' ');
        let check_out_parts = split_sparse_tokens(check_out_line, ' ');

        let (check_in, in_comment) = match check_in_parts.as_slice() {
            [check_in] => (check_in, None),
            [check_in, comment @ ..] => (check_in, Some(comment.join(" "))),
            _ => {
                return Err(error::Main::EntryIncorrectNumberOfTokens);
            }
        };

        let (check_out, out_comment) = match check_out_parts.as_slice() {
            [check_out] => (check_out, None),
            [check_out, comment @ ..] => (check_out, Some(comment.join(" "))),
            _ => {
                return Err(error::Main::EntryIncorrectNumberOfTokens);
            }
        };

        Self::from_tokens(check_in, check_out, in_comment, out_comment)
    }

    fn from_tokens(
        check_in: &str,
        check_out: &str,
        in_comment: Option<String>,
        out_comment: Option<String>,
    ) -> Result<Entry<FixedOffset>> {
        let check_in = DateTime::parse_from_rfc3339(check_in)?;
        let check_out = DateTime::parse_from_rfc3339(check_out)?;

        Entry::try_new(check_in, check_out, in_comment, out_comment)
    }
}

fn split_sparse_tokens_str<'a>(line: &'a str, pat: &'static str) -> Vec<&'a str> {
    line.split(pat)
        .map(str::trim)
        .filter(|t| !t.is_empty())
        .collect()
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
        let lines = split_sparse_tokens(value, '\n');
        let [check_in_line, check_out_line] = lines.as_slice() else {
            return Err(error::Main::EntryIncorrectNumberOfLines);
        };
        let check_in_parts = split_sparse_tokens(check_in_line, ' ');
        let check_out_parts = split_sparse_tokens(check_out_line, ' ');

        let (check_in, in_comment) = match check_in_parts.as_slice() {
            [check_in] => (check_in, None),
            [check_in, comment @ ..] => (check_in, Some(comment.join(" "))),
            _ => {
                return Err(error::Main::EntryIncorrectNumberOfTokens);
            }
        };

        let (check_out, out_comment) = match check_out_parts.as_slice() {
            [check_out] => (check_out, None),
            [check_out, comment @ ..] => (check_out, Some(comment.join(" "))),
            _ => {
                return Err(error::Main::EntryIncorrectNumberOfTokens);
            }
        };

        Self::from_tokens(check_in, check_out, in_comment, out_comment)
    }
}

pub enum Latest<'a, Tz: TimeZone> {
    Entry(&'a Entry<Tz>),
    Current(&'a DateTime<Tz>, Option<&'a str>),
    None,
}

#[derive(Clone)]
#[cfg_attr(test, derive(Debug))]
pub struct Record<Tz: TimeZone> {
    entries: Vec<Entry<Tz>>,
    current_session: Option<(DateTime<Tz>, Option<String>)>,
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
            current_session: current_session.map(|(cs, comment)| (cs.with_timezone(tz), comment)),
        }
    }

    pub fn clone_last_datetime(&self) -> Result<Option<DateTime<Tz>>> {
        match self.get_latest() {
            Latest::Entry(entry) => Some(entry.get_check_out()).transpose(),
            Latest::Current(current, _) => Ok(Some(current.clone())),
            Latest::None => Ok(None),
        }
    }

    pub fn pop(&mut self) -> Option<(DateTime<Tz>, Option<String>)> {
        self.current_session.take().or_else(|| {
            self.entries.pop().map(|entry| {
                let out_comment = entry.out_comment.clone();
                let in_comment = entry.in_comment.clone();
                let (start, end) = DateTimeRange::from(entry).into_bounds();
                // Something about mutating self inside this closure feels very wrong...
                self.current_session = Some((start, in_comment));
                (end, out_comment)
            })
        })
    }

    pub fn get_current_session(&self) -> Option<&(DateTime<Tz>, Option<String>)> {
        self.current_session.as_ref()
    }

    pub fn get_entries(&self) -> &[Entry<Tz>] {
        &self.entries
    }

    pub fn get_latest(&self) -> Latest<'_, Tz> {
        match (&self.current_session, self.entries.last()) {
            (None, None) => Latest::None,
            (None, Some(last_entry)) => Latest::Entry(last_entry),
            (Some((current_session, in_comment)), _) => {
                Latest::Current(current_session, in_comment.as_deref())
            }
        }
    }

    pub fn serialize(&self) -> Result<String>
    where
        Tz::Offset: Display,
    {
        let mut buf = String::new();
        for entry @ Entry {
            check_in,
            work_time_millis: _,
            in_comment,
            out_comment,
        } in &self.entries
        {
            write!(buf, "{:<32}", check_in.to_rfc3339())?;
            if let Some(comment) = in_comment {
                write!(buf, " {comment}")?;
            }
            writeln!(buf)?;
            write!(buf, "{:<32}", entry.get_check_out()?.to_rfc3339())?;
            if let Some(comment) = out_comment {
                write!(buf, " {comment}")?;
            }
            writeln!(buf)?;
            writeln!(buf)?;
        }

        match self.current_session.clone() {
            Some((current_session, Some(in_comment))) => {
                writeln!(buf, "{:<32} {}", current_session.to_rfc3339(), in_comment)?;
            }
            Some((current_session, None)) => {
                writeln!(buf, "{:<32}", current_session.to_rfc3339())?;
            }
            _ => (),
        }

        Ok(buf)
    }
}

pub struct Iterator<Tz: TimeZone> {
    entries: std::iter::Rev<std::vec::IntoIter<Entry<Tz>>>,
    current_session: std::option::IntoIter<(DateTime<Tz>, Option<String>)>,
}

#[cfg_attr(test, derive(Debug, PartialEq, Eq))]
pub enum Item<Tz: TimeZone> {
    Entry(Entry<Tz>),
    CurrentSession(DateTime<Tz>, Option<String>),
}

impl<Tz: TimeZone> Item<Tz> {
    pub fn into_entry<F>(self, end: F) -> Result<Entry<Tz>>
    where
        F: FnOnce() -> DateTime<Tz>,
    {
        match self {
            Item::Entry(entry) => Ok(entry),
            Item::CurrentSession(current_session, in_comment) => {
                Entry::try_new(current_session, end(), in_comment, None)
            }
        }
    }
}

impl<Tz: TimeZone> std::iter::Iterator for Iterator<Tz> {
    type Item = Item<Tz>;

    fn next(&mut self) -> Option<Self::Item> {
        self.entries.next_back().map(Item::Entry).or_else(|| {
            self.current_session
                .next()
                .map(|(current_session, in_comment)| {
                    Item::CurrentSession(current_session, in_comment)
                })
        })
    }
}

impl<Tz: TimeZone> DoubleEndedIterator for Iterator<Tz> {
    fn next_back(&mut self) -> Option<Self::Item> {
        self.current_session
            .next_back()
            .map(|(current_session, in_comment)| Item::CurrentSession(current_session, in_comment))
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
        let datetime_ranges = self.try_into_cropped_datetime_ranges(
            ctx,
            day.into_day_start(ctx)?,
            day.into_day_end(ctx)?,
        )?;

        Ok(datetime_ranges.into_iter().sum())
    }

    pub fn todays_time(self, ctx: &context::Context<Tz>) -> Result<Duration> {
        let now = Local::now();
        let today = now.date_naive();

        let mut datetime_ranges_today = self
            .into_iter()
            .map(|item| {
                item.into_entry(|| ctx.timezone.now())
                    .map(DateTimeRange::from)
            })
            .rev()
            .take_while(|dtr| dtr.iter().any(|dtr| dtr.end().date_naive() == today))
            .collect::<Result<Vec<_>>>()?;

        let Some((first_check_in, first_check_out)) =
            datetime_ranges_today.pop().map(DateTimeRange::into_bounds)
        else {
            return Ok(Duration::zero());
        };

        let first_check_in = if first_check_in.date_naive() == today {
            first_check_in
        } else {
            today.into_day_start(ctx)?
        };

        datetime_ranges_today.push(DateTimeRange::new(first_check_in, first_check_out)?);

        Ok(datetime_ranges_today.into_iter().sum())
    }

    pub fn total_time(self, ctx: &context::Context<Tz>) -> Result<Duration> {
        let datetime_ranges = self
            .into_iter()
            .map(|entry| {
                entry
                    .into_entry(|| ctx.timezone.now())
                    .map(DateTimeRange::from)
            })
            .collect::<Result<Vec<_>>>()?;

        Ok(datetime_ranges.into_iter().sum())
    }

    pub fn current_session_time(&self, ctx: &Context<Tz>) -> Option<Duration> {
        self.current_session
            .as_ref()
            .map(|(sesh, _)| sesh.signed_duration_since(ctx.timezone.now()))
    }

    pub fn try_into_cropped_datetime_ranges(
        self,
        // FIXME: The entire context is not needed
        ctx: &Context<Tz>,
        start: DateTime<Tz>,
        end: DateTime<Tz>,
    ) -> Result<Vec<DateTimeRange<Tz>>> {
        if end <= start {
            return Err(error::Main::RangeStartPosition);
        }

        let mut datetime_ranges =
            self.into_iter()
                .map(|item| {
                    item.into_entry(|| ctx.timezone.now().min(end))
                        .map(DateTimeRange::from)
                })
                .filter(|entry| {
                    entry.iter().map(|dtr| dtr.clone().into_bounds()).any(
                        |(check_in, check_out)| {
                            (start <= check_in && check_in < end)
                                || (start <= check_out && check_out < end)
                        },
                    )
                })
                .collect::<Result<VecDeque<_>>>()?;

        if datetime_ranges.is_empty() {
            return Ok(vec![]);
        }

        let (first_check_in, first_check_out) = datetime_ranges
            .pop_front()
            .expect("datetime_ranges must be confirmed to have at least one element")
            .into_bounds();

        let first_check_in = if first_check_in < start {
            start
        } else {
            first_check_in
        };

        datetime_ranges.push_front(DateTimeRange::new(first_check_in, first_check_out)?);

        let (last_check_in, last_check_out) = datetime_ranges
            .pop_back()
            .expect("datetime_ranges must be confirmed to have at least one element")
            .into_bounds();

        let last_check_out = if last_check_out > end {
            end
        } else {
            last_check_out
        };

        datetime_ranges.push_back(DateTimeRange::new(last_check_in, last_check_out)?);

        Ok(datetime_ranges.into())
    }
}

impl Record<Utc> {
    pub fn clock_in(&mut self, comment: Option<String>) -> Result<DateTime<Utc>> {
        if self.current_session.is_some() {
            return Err(error::Main::AlreadyClockedIn);
        };

        let now = Utc::now();

        self.current_session = Some((now, comment));

        Ok(now)
    }

    pub fn clock_out(&mut self, out_comment: Option<String>) -> Result<(DateTime<Utc>, Duration)> {
        let Some((current_session, in_comment)) = self.current_session.as_ref() else {
            return Err(error::Main::NotClockedIn);
        };

        let now = Utc::now();
        let since = current_session.signed_duration_since(now);

        self.entries.push(Entry::try_new(
            *current_session,
            Utc::now(),
            in_comment.clone(),
            out_comment,
        )?);

        self.current_session = None;

        Ok((now, since))
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

        let mut paragraphs = split_sparse_tokens_str(value, "\n\n");

        let last_paragraph = paragraphs.pop();

        let mut entries = paragraphs
            .into_iter()
            .map(Entry::try_from)
            .collect::<Result<Vec<_>>>()?;

        if let Some(last_paragraph) = last_paragraph {
            match split_sparse_tokens(last_paragraph, '\n').as_slice() {
                [check_in_line, check_out_line] => {
                    entries.push(Entry::from_lines(check_in_line, check_out_line)?);
                }
                [session_line] => match split_sparse_tokens(session_line, ' ').as_slice() {
                    [check_in] => {
                        current_session = Some((DateTime::parse_from_rfc3339(check_in)?, None));
                    }
                    [check_in, in_comment @ ..] => {
                        current_session = Some((
                            DateTime::parse_from_rfc3339(check_in)?,
                            Some(in_comment.join(" ")),
                        ));
                    }
                    _ => return Err(error::Main::EntryIncorrectNumberOfTokens),
                },
                _ => {
                    return Err(error::Main::EntryIncorrectNumberOfLines);
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

    const RECORD_STR: &str = "2023-01-01T00:00:00+00:00       
2023-01-01T01:00:00+00:00       

2023-01-01T02:00:00+00:00       
2023-01-01T03:00:00+00:00        This is a comment!

2023-01-01T04:00:00+00:00       
";

    fn get_record() -> Record<FixedOffset> {
        Record::try_from(RECORD_STR).unwrap()
    }

    #[test]
    fn iterator() {
        let rec = get_record();
        let rec_vec = rec.into_iter().collect::<Vec<_>>();

        assert_eq!(
            vec![
                record::Item::Entry(Entry {
                    check_in: datetime_hm(0, 0),
                    work_time_millis: 3_600_000,
                    in_comment: None,
                    out_comment: None,
                }),
                record::Item::Entry(Entry {
                    check_in: datetime_hm(2, 0),
                    work_time_millis: 3_600_000,
                    in_comment: None,
                    out_comment: Some("This is a comment!".to_owned()),
                }),
                record::Item::CurrentSession(datetime_hm(4, 0), None),
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
                record::Item::CurrentSession(datetime_hm(4, 0), None),
                record::Item::Entry(Entry {
                    check_in: datetime_hm(2, 0),
                    work_time_millis: 3_600_000,
                    in_comment: None,
                    out_comment: Some("This is a comment!".to_owned()),
                }),
                record::Item::Entry(Entry {
                    check_in: datetime_hm(0, 0),
                    work_time_millis: 3_600_000,
                    in_comment: None,
                    out_comment: None,
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
        let rec_file = "2023-06-30T04:30:00.893153+00:00
2023-06-30T07:15:07.931153+00:00

2023-07-10T05:05:42.372091+00:00
2023-07-10T09:38:44.320091+00:00

2023-07-10T20:00:00+00:00       
2023-07-10T22:13:34.369+00:00   

2023-07-11T04:30:55.569838+00:00
2023-07-11T05:05:55.569838+00:00

2023-07-11T09:01:20.726248+00:00
2023-07-11T12:08:36.149248+00:00

2023-07-11T12:32:28.616529+00:00
2023-07-11T14:27:00.836529+00:00

2023-07-11T20:03:53.114039+00:00
2023-07-11T22:41:25.885039+00:00

2023-07-12T04:30:00+00:00       
2023-07-12T04:54:25.885+00:00   

2023-07-12T09:30:00+00:00       
2023-07-12T13:00:00+00:00       

2023-07-12T22:04:34.947469+00:00
2023-07-12T23:29:44.706469+00:00

2023-07-13T09:08:38.290767+00:00
2023-07-13T10:34:50.199767+00:00
";
        let rec = Record::try_from(rec_file)
            .unwrap()
            .with_timezone(&ctx.timezone);
        paint_day_range(&ctx, &rec, date_md(7, 10)..=date_md(7, 12), 24).unwrap();
    }

    #[test]
    fn read_write_read_integrity() {
        let rec = Record::try_from(RECORD_STR).unwrap();
        let written = rec.serialize().unwrap();

        assert_eq!(RECORD_STR, &written);
    }

    #[test]
    fn current_session_with_multiword_comment() {
        let timezone = FixedOffset::east_opt(0).unwrap();
        let rec_file = "2023-01-01T04:00:00.000000+00:00 Blah blah blah
";
        let rec = Record::try_from(rec_file).unwrap().with_timezone(&timezone);
        let rec_vec = rec.into_iter().rev().collect::<Vec<_>>();

        assert_eq!(
            vec![record::Item::CurrentSession(
                datetime_hm(4, 0),
                Some("Blah blah blah".to_string())
            ),],
            rec_vec
        );
    }

    #[test]
    fn one_entry_with_multiword_out_comment() {
        let timezone = FixedOffset::east_opt(0).unwrap();
        let rec_file = "2023-01-01T04:00:00.000000+00:00
2023-01-01T05:00:00.000000+00:00 Blah blah blah
";
        let rec = Record::try_from(rec_file).unwrap().with_timezone(&timezone);
        let rec_vec = rec.into_iter().rev().collect::<Vec<_>>();

        assert_eq!(
            vec![record::Item::Entry(Entry {
                check_in: datetime_hm(4, 0),
                work_time_millis: 3_600_000,
                in_comment: None,
                out_comment: Some("Blah blah blah".to_string())
            })],
            rec_vec
        );
    }

    #[test]
    fn one_entry_with_multiword_in_comment() {
        let timezone = FixedOffset::east_opt(0).unwrap();
        let rec_file = "2023-01-01T04:00:00.000000+00:00 Blah blah blah
2023-01-01T05:00:00.000000+00:00
";
        let rec = Record::try_from(rec_file).unwrap().with_timezone(&timezone);
        let rec_vec = rec.into_iter().rev().collect::<Vec<_>>();

        assert_eq!(
            vec![record::Item::Entry(Entry {
                check_in: datetime_hm(4, 0),
                work_time_millis: 3_600_000,
                in_comment: Some("Blah blah blah".to_string()),
                out_comment: None,
            })],
            rec_vec
        );
    }
}
