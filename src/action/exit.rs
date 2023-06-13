use crate::{app::context, error::Result};

pub fn run(record: &mut context::Record) -> Result<()> {
    record.clock_out()?;

    Ok(())
}
