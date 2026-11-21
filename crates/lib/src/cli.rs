use std::env;
use std::path::PathBuf;
use std::process::ExitCode;

use serde_json::Value;

use crate::context::{Context, Parts};
use crate::manifest::Plugin;
use crate::Result;

#[must_use]
pub fn run(plugin: &Plugin) -> ExitCode {
    let args: Vec<String> = env::args().skip(1).collect();

    match execute(plugin, &args) {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("noctalia: {error}");
            ExitCode::FAILURE
        }
    }
}

fn execute(plugin: &Plugin, args: &[String]) -> Result<()> {
    if args.first().is_some_and(|arg| arg == "emit") {
        let dir = PathBuf::from(args.get(1).ok_or("usage: emit <dir> <helper>")?);
        let helper = args.get(2).ok_or("usage: emit <dir> <helper>")?;
        plugin.emit(&dir, helper)?;
        return Ok(());
    }

    let parts = Parts {
        entry: args.first().cloned().ok_or("missing entry")?,
        event: args.get(1).cloned().ok_or("missing event")?,
        payload: decode(args.get(3)),
        config: decode(args.get(4)),
        state: decode(args.get(5)),
        data_dir: PathBuf::from(args.get(2).cloned().unwrap_or_default()),
    };

    let context = Context::new(parts, plugin.translations_value());

    for op in plugin.dispatch(&context)? {
        println!("{}", serde_json::to_string(&op)?);
    }

    Ok(())
}

fn decode(raw: Option<&String>) -> Value {
    raw.map_or_else(|| Value::Null, |value| {
        serde_json::from_str(value).unwrap_or_default()
    })
}
