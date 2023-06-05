use std::{cell::RefCell, fmt::Display, fs};

use chrono::{DateTime, Utc};

use crate::error::{self, Result};

pub enum Event {
    In,
    Out,
}

impl Display for Event {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Event::In => write!(f, "in"),
            Event::Out => write!(f, "out"),
        }
    }
}

impl TryFrom<&str> for Event {
    type Error = error::Main;

    fn try_from(value: &str) -> std::result::Result<Self, Self::Error> {
        let event = match value {
            "in" => Self::In,
            "out" => Self::Out,
            invalid => return Err(error::Main::InvalidEventString(invalid.to_owned())),
        };

        Ok(event)
    }
}

pub struct Entry {
    pub date: DateTime<Utc>,
    pub event: Event,
}

impl Entry {
    pub fn new(event: Event) -> Self {
        Self {
            date: Utc::now(),
            event,
        }
    }
}

pub struct Record(pub Vec<Entry>);

impl TryFrom<&str> for Record {
    type Error = error::Main;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        let record = value
            .split('\n')
            .filter(|entry| !entry.is_empty())
            .map(|entry| {
                let [date, event] = entry.split(' ').collect::<Vec<_>>()[..] else {
                    return Err(error::Main::InvalidEntry);
                };

                let date = DateTime::parse_from_rfc3339(date)?.with_timezone(&Utc);
                let event = Event::try_from(event)?;

                Ok(Entry { date, event })
            })
            .collect::<Result<Vec<_>, _>>()?;

        Ok(Record(record))
    }
}

impl Display for Record {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for Entry { date, event } in &self.0 {
            writeln!(f, "{} {}", date.to_rfc3339(), event)?;
        }

        Ok(())
    }
}

pub struct Base {
    pub record: RefCell<Record>,
}

pub fn init() -> Result<()> {
    if load()?.is_some() {
        return Err(error::Main::AlreadyInitialized);
    }

    fs::create_dir(".punch_clock")?;
    fs::write(".punch_clock/record", "")?;

    Ok(())
}

pub fn load() -> Result<Option<Base>> {
    if !std::path::Path::new(".punch_clock").exists() {
        return Ok(None);
    }

    let record: Record = fs::read_to_string(".punch_clock/record")?
        .as_str()
        .try_into()?;

    Ok(Some(Base {
        record: RefCell::new(record),
    }))
}
