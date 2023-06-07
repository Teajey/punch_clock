use crate::{app::context, error::Result};

pub fn run(ctx: &context::Base) -> Result<()> {
    let mut record = ctx.record.borrow_mut();

    record.clock_in()?;

    Ok(())
}
