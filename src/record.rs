use std::{
    fmt::{Display, Write},
    fs,
};

use chrono::{DateTime, Duration, TimeZone, Utc};

use crate::error::{self, Result};

#[derive(Clone)]
pub struct Entry {
    pub check_in: DateTime<Utc>,
    work_time_millis: u32,
}

impl Entry {
    pub fn try_new(check_in: DateTime<Utc>, check_out: DateTime<Utc>) -> Result<Self> {
        if check_out < check_in {
            return Err(error::Main::CheckOutBeforeCheckIn);
        }

        let work_time_millis = check_out
            .signed_duration_since(check_in)
            .num_milliseconds()
            .try_into()?;

        Ok(Self {
            check_in,
            work_time_millis,
        })
    }

    fn from_tokens(check_in: &str, check_out: &str) -> Result<Self> {
        let check_in = DateTime::parse_from_rfc3339(check_in)?.with_timezone(&Utc);
        let check_out = DateTime::parse_from_rfc3339(check_out)?.with_timezone(&Utc);

        Self::try_new(check_in, check_out)
    }

    fn get_work_time(&self) -> Duration {
        Duration::milliseconds(self.work_time_millis.into())
    }

    pub fn get_check_out(&self) -> Result<DateTime<Utc>> {
        self.check_in
            .checked_add_signed(self.get_work_time())
            .ok_or_else(|| error::Main::DateTimeOverflow)
    }

    fn to_date_pair(self) -> (DateTime<Utc>, DateTime<Utc>) {
        let check_in = self.check_in;
        let check_out = self.check_in + self.get_work_time();

        (check_in, check_out)
    }
}

impl TryFrom<&str> for Entry {
    type Error = error::Main;

    fn try_from(value: &str) -> std::result::Result<Self, Self::Error> {
        let [check_in, check_out] = value.split(' ').collect::<Vec<_>>()[..] else {
            return Err(error::Main::EntryIncorrectNumberOfTokens);
        };

        Self::from_tokens(check_in, check_out)
    }
}

pub enum Latest<'a> {
    Entry(&'a Entry),
    Current(&'a DateTime<Utc>),
    None,
}

#[derive(Clone)]
pub struct Record {
    entries: Vec<Entry>,
    current_session: Option<DateTime<Utc>>,
}

impl Record {
    pub fn get_current_session(&self) -> Option<&DateTime<Utc>> {
        self.current_session.as_ref()
    }

    pub fn get_entries(&self) -> &[Entry] {
        &self.entries
    }

    pub fn get_latest(&self) -> Latest<'_> {
        match (&self.current_session, self.entries.last()) {
            (None, None) => Latest::None,
            (None, Some(last_entry)) => Latest::Entry(last_entry),
            (Some(current_session), _) => Latest::Current(current_session),
        }
    }

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

    pub fn serialize<Tz>(&self, timezone: &Tz) -> Result<String>
    where
        Tz: TimeZone,
        Tz::Offset: Display,
    {
        let mut buf = String::new();
        for entry in &self.entries {
            writeln!(
                buf,
                "{} {}",
                entry.check_in.with_timezone(timezone).to_rfc3339(),
                entry.get_check_out()?.with_timezone(timezone).to_rfc3339(),
            )?;
        }

        if let Some(current_session) = self.current_session {
            writeln!(
                buf,
                "{}",
                current_session.with_timezone(timezone).to_rfc3339()
            )?;
        }

        Ok(buf)
    }

    pub fn init() -> Result<()> {
        if Self::load()?.is_some() {
            return Err(error::Main::AlreadyInitialized);
        }

        fs::create_dir(".punch_clock")?;
        fs::write(".punch_clock/record", "")?;

        Ok(())
    }

    pub fn load() -> Result<Option<Self>> {
        if !std::path::Path::new(".punch_clock").exists() {
            return Ok(None);
        }

        let record = fs::read_to_string(".punch_clock/record")?
            .as_str()
            .try_into()?;

        Ok(Some(record))
    }

    fn to_vec(self) -> Result<Vec<Entry>> {
        let mut entries = self.entries;
        if let Some(current_session) = self.current_session {
            entries.push(Entry::try_new(current_session, Utc::now())?);
        }
        Ok(entries)
    }

    pub fn total_time(self) -> Result<Duration> {
        let seconds = self
            .to_vec()?
            .into_iter()
            .map(Entry::to_date_pair)
            .map(|(check_in, check_out)| check_out.signed_duration_since(check_in))
            .map(|duration| duration.num_seconds())
            .sum::<i64>();

        Ok(Duration::seconds(seconds))
    }
}

impl TryFrom<&str> for Record {
    type Error = error::Main;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        let mut current_session = None;

        let mut lines = value
            .split('\n')
            .filter(|entry| !entry.is_empty())
            .collect::<Vec<_>>();

        let last_line = lines.pop();

        let mut entries = lines
            .into_iter()
            .map(Entry::try_from)
            .collect::<Result<Vec<_>>>()?;

        if let Some(last_line) = last_line {
            match last_line.split(' ').collect::<Vec<_>>()[..] {
                [check_in, check_out] => {
                    entries.push(Entry::from_tokens(check_in, check_out)?);
                }
                [check_in] => {
                    current_session =
                        Some(DateTime::parse_from_rfc3339(check_in)?.with_timezone(&Utc));
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
