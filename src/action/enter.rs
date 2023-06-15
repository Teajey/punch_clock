use chrono::Utc;

use crate::{error::Result, record::Record};

pub fn run(record: &mut Record<Utc>) -> Result<()> {
    record.clock_in()?;

    Ok(())
}
