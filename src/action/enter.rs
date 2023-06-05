use chrono::{Duration, Local, Utc};

use crate::{
    app::context::{self, Entry, Event},
    error::{self, Ago, Result},
};

pub fn run(ctx: &context::Base) -> Result<()> {
    let mut record = ctx.record.borrow_mut();

    if let Some(Entry { date, event }) = record.0.last() {
        let since = date.signed_duration_since(Utc::now());

        if since > Duration::zero() {
            return Err(error::Main::LastEntryInFuture(
                date.with_timezone(&Local),
                Ago(since),
            ));
        }

        if let Event::In = event {
            return Err(error::Main::AlreadyCheckedIn(
                date.with_timezone(&Local),
                Ago(since),
            ));
        }
    }

    record.0.push(Entry::new(Event::In));

    Ok(())
}
