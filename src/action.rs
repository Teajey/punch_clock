mod dump;
mod enter;
mod exit;
mod status;

use crate::{
    app::{cli::Action, context},
    error::Result,
};

pub fn run(ctx: &context::Base, action: &Action) -> Result<()> {
    match action {
        Action::In => enter::run(ctx),
        Action::Out => exit::run(ctx),
        Action::Status => status::run(ctx),
        Action::Dump => dump::run(ctx),
    }
}
