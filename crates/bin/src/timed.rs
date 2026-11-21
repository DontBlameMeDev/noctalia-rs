use std::process::ExitCode;

use noctalia::run as run_cli;
use noctalia::{Context, Meta, Op, Plugin, Result};
use serde_json::Value;

fn update(ctx: &Context) -> Result<Vec<Op>> {
    let timers: Vec<Value> = ctx.state_value("timers")?;

    Ok(vec![Op::log(format!("{} timers", timers.len()))])
}

pub fn run() -> ExitCode {
    let plugin = Plugin::new(Meta::new("dontblameme/timed", "Timed", "0.1.0"))
        .service("service", update);

    run_cli(&plugin)
}
