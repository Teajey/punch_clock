use chrono::Utc;

use crate::{app::context, error::Ago};

pub fn run(ctx: &context::Base) {
    match ctx.record.borrow().0.last() {
        Some(context::Entry { date, event }) => {
            let since = date.signed_duration_since(Utc::now());
            let ago = Ago(since);

            match event {
                context::Event::In => println!("Currently clocked in ({ago})"),
                context::Event::Out => println!("Currently clocked out ({ago})"),
            }
        }
        None => println!("No clock in/out records have been created."),
    }
}
