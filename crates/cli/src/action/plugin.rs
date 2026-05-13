use std::io::Write;
use std::{collections::HashMap, process::Command};

use anyhow::{anyhow, Context as _, Result};
use chrono::Utc;
use punch_clock_core::{context::Context, record::Record, time::ContextTimeZone};

use crate::plugins;

pub fn run<Tz: ContextTimeZone>(
    ctx: &Context<Tz>,
    name: &str,
    args: &[String],
    record: &Record<Utc>,
) -> Result<()> {
    let Some(path_var) = std::env::var("PATH").ok() else {
        return Ok(());
    };

    let mut plugin_map = HashMap::new();
    for plugin_dir in plugins::try_iter(&path_var).context("trying plugin iterator")? {
        for p in plugin_dir.context("reading a plugin directory")? {
            let (name, path) = p.context("reading plugin file")?;
            plugin_map.insert(name, path);
        }
    }

    let Some(plugin_path) = plugin_map.get(name) else {
        return Err(anyhow!(
            "attempted to run a plugin that wasn't mapped to a path"
        ));
    };

    let mut cmd = Command::new(plugin_path);
    cmd.args(args)
        .env("PUNCH_CLOCK_TIMEZONE", ctx.timezone.environment_value())
        .env("PUNCH_CLOCK_SKIP_HOOKS", ctx.skip_hooks.to_string());

    if let Some(offset) = ctx.offset {
        cmd.env("PUNCH_CLOCK_FIXED_UTC_OFFSET", offset.to_string());
    }

    let mut child = cmd
        .stdin(std::process::Stdio::piped())
        .spawn()
        .context("spawning plugin child process")?;

    if let Some(mut stdin) = child.stdin.take() {
        for e in record.get_entries() {
            let check_out = e
                .get_check_out()
                .map_or_else(|err| err.to_string(), |x| x.to_rfc3339());
            let val = serde_json::json!({
                "clock_in": e.check_in.to_rfc3339(),
                "clock_out": check_out,
                "in_comment": e.in_comment,
                "out_comment": e.out_comment,
            });
            let _ = serde_json::to_writer(&mut stdin, &val);
            let _ = stdin.write_all(b"\n");
        }

        if let Some((check_in, in_comment)) = record.get_current_session() {
            let val = serde_json::json!({
                "clock_in": check_in.to_rfc3339(),
                "in_comment": in_comment,
            });
            let _ = serde_json::to_writer(&mut stdin, &val);
            let _ = stdin.write_all(b"\n");
        }
    }

    let status = child.wait().context("waiting on plugin child process")?;

    if !status.success() {
        let exit_code = status.code().unwrap_or(-1);
        std::process::exit(exit_code);
    }

    Ok(())
}
