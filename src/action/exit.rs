use chrono::{Duration, Local, Utc};

use crate::{
    app::context::{self, Entry, Event},
    error::{self, Ago, Result},
};

pub fn run(ctx: &context::Base) -> Result<()> {
    let mut record = ctx.record.borrow_mut();

    let Some(Entry { date, event }) = record.0.last() else {
        return Err(error::Main::ExitingFirst);
    };

    let since = date.signed_duration_since(Utc::now());

    if since > Duration::zero() {
        return Err(error::Main::LastEntryInFuture(
            date.with_timezone(&Local),
            Ago(since),
        ));
    }

    if let Event::Out = event {
        return Err(error::Main::AlreadyCheckedOut(
            date.with_timezone(&Local),
            Ago(since),
        ));
    }

    record.0.push(Entry::new(Event::Out));

    Ok(())
}
