use crate::{error::Result, record::Record};

pub fn run(record: &mut Record) -> Result<()> {
    record.clock_out()?;

    Ok(())
}
