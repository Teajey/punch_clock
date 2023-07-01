use std::{
    collections::VecDeque,
    fmt::{Display, Write},
    fs,
};

use chrono::{DateTime, Days, Duration, FixedOffset, Local, NaiveDate, NaiveTime, TimeZone, Utc};

use crate::error::{self, Result};

#[derive(Clone)]
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

    fn sum_datetime_pairs(pairs: Vec<(DateTime<Tz>, DateTime<Tz>)>) -> Duration {
        let seconds = pairs
            .into_iter()
            .map(|(check_in, check_out)| check_out.signed_duration_since(check_in))
            .map(|duration| duration.num_seconds())
            .sum::<i64>();

        Duration::seconds(seconds)
    }
}

pub struct LocalIterator {
    entries: std::iter::Rev<std::vec::IntoIter<Entry<Local>>>,
    current_session: std::option::IntoIter<DateTime<Local>>,
}

impl LocalIterator {
    fn current_session_into_entry(&mut self) -> Option<Result<Entry<Local>>> {
        self.current_session
            .next()
            .map(|current_session| Entry::try_new(current_session, Local::now()))
    }
}

impl Iterator for LocalIterator {
    type Item = Result<Entry<Local>>;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(entry) = self.entries.next() {
            Some(Ok(entry))
        } else {
            self.current_session_into_entry()
        }
    }
}

impl DoubleEndedIterator for LocalIterator {
    fn next_back(&mut self) -> Option<Self::Item> {
        if let Some(current_session) = self.current_session_into_entry() {
            Some(current_session)
        } else {
            self.entries.next_back().map(Ok)
        }
    }
}

impl IntoIterator for Record<Local> {
    type Item = Result<Entry<Local>>;

    type IntoIter = LocalIterator;

    fn into_iter(self) -> Self::IntoIter {
        Self::IntoIter {
            entries: self.entries.into_iter().rev(),
            current_session: self.current_session.into_iter(),
        }
    }
}

impl Record<Local> {
    pub fn days_time(self, day: NaiveDate) -> Result<Duration> {
        let mut date_pairs = self
            .into_iter()
            .map(|entry_result| entry_result.map(Entry::into_date_pair))
            .filter(|entry| {
                entry.iter().any(|(check_in, check_out)| {
                    check_in.date_naive() == day || check_out.date_naive() == day
                })
            })
            .collect::<Result<VecDeque<_>>>()?;

        let Some((first_check_in, first_check_out)) = date_pairs.pop_back() else {
            return Ok(Duration::zero());
        };

        let first_check_in = if first_check_in.date_naive() == day {
            first_check_in
        } else {
            match day.and_time(NaiveTime::default()).and_local_timezone(Local) {
                chrono::LocalResult::None | chrono::LocalResult::Ambiguous(_, _) => {
                    panic!("If you see this error, send this to the developer: {day:?}")
                }
                chrono::LocalResult::Single(dt) => dt,
            }
        };

        date_pairs.push_back((first_check_in, first_check_out));

        let Some((last_check_in, last_check_out)) = date_pairs.pop_front() else {
            return Ok(Duration::zero());
        };

        let last_check_out = if last_check_out.date_naive() == day {
            last_check_out
        } else {
            match day.and_time(NaiveTime::default()).and_local_timezone(Local) {
                chrono::LocalResult::None | chrono::LocalResult::Ambiguous(_, _) => {
                    panic!("If you see this error, send this to the developer: {day:?}")
                }
                chrono::LocalResult::Single(dt) => {
                    let dt = dt
                        .checked_add_days(Days::new(1))
                        .ok_or_else(|| error::Main::DateOutOfRange)?;
                    let dt = dt
                        .checked_sub_signed(Duration::milliseconds(1))
                        .ok_or_else(|| error::Main::DateOutOfRange)?;
                    dt
                }
            }
        };

        date_pairs.push_front((last_check_in, last_check_out));

        Ok(Self::sum_datetime_pairs(date_pairs.into()))
    }

    pub fn todays_time(self) -> Result<Duration> {
        let now = Local::now();
        let today = now.date_naive();
        let mut date_pairs_today = self
            .into_iter()
            .rev()
            .map(|entry_result| entry_result.map(Entry::into_date_pair))
            .take_while(|entry| {
                entry
                    .iter()
                    .any(|(check_in, _)| check_in.date_naive() == today)
            })
            .collect::<Result<Vec<_>>>()?;

        let Some((first_check_in, first_check_out)) = date_pairs_today.pop() else {
            return Ok(Duration::zero());
        };

        let first_check_in = if first_check_in.date_naive() == today {
            first_check_in
        } else {
            match today
                .and_time(NaiveTime::default())
                .and_local_timezone(Local)
            {
                chrono::LocalResult::None | chrono::LocalResult::Ambiguous(_, _) => {
                    panic!("If you see this error, send this to the developer: {today:?}")
                }
                chrono::LocalResult::Single(dt) => dt,
            }
        };

        date_pairs_today.push((first_check_in, first_check_out));

        Ok(Self::sum_datetime_pairs(date_pairs_today))
    }

    pub fn total_time(self) -> Result<Duration> {
        let datetime_pairs = self
            .into_iter()
            .map(|entry_result| entry_result.map(Entry::into_date_pair))
            .collect::<Result<Vec<_>>>()?;

        Ok(Self::sum_datetime_pairs(datetime_pairs))
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
