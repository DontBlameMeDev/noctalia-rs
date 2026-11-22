use std::env;
use std::path::PathBuf;
use std::process::ExitCode;

use serde_json::Value;

use crate::Result;
use crate::context::{Context, Parts};
use crate::manifest::Plugin;

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
        let dir =
            PathBuf::from(args.get(1).ok_or("usage: emit <dir> <helper>")?);
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
        host: decode(args.get(6)),
        data_dir: PathBuf::from(args.get(2).cloned().unwrap_or_default()),
    };

    let context = Context::new(parts, plugin.translations_value());

    for op in plugin.dispatch(&context)? {
        println!("{}", serde_json::to_string(&op)?);
    }

    Ok(())
}

fn decode(raw: Option<&String>) -> Value {
    raw.map_or_else(
        || Value::Null,
        |value| serde_json::from_str(value).unwrap_or_default(),
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Context, Meta, Op};
    use std::fs;
    use std::process;

    fn plugin() -> Plugin {
        fn handler(ctx: &Context) -> crate::Result<Vec<Op>> {
            let _ = serde_json::to_value(&ctx.entry)?;
            Ok(vec![Op::log(ctx.event.clone())])
        }

        Plugin::new(Meta::new("me/hello", "Hello", "0.1.0"))
            .tr("notify.title", "Hello")
            .service("service", handler)
    }

    fn temp_dir(name: &str) -> PathBuf {
        let path =
            env::temp_dir().join(format!("noctalia-{name}-{}", process::id()));
        let _ = fs::remove_dir_all(&path);
        fs::create_dir_all(&path).expect("temp dir");
        path
    }

    #[test]
    fn test_decode_handles_missing_invalid_and_valid_json() {
        let valid = "{\"key\":1}".to_owned();
        let invalid = "not json".to_owned();

        assert_eq!(decode(None), Value::Null);
        assert_eq!(decode(Some(&invalid)), Value::Null);
        assert_eq!(decode(Some(&valid)), serde_json::json!({ "key": 1 }));
    }

    #[test]
    fn test_execute_emits_plugin_files() {
        let dir = temp_dir("emit");
        let path = dir.to_string_lossy().into_owned();
        let args = vec!["emit".to_owned(), path, "hello-helper".to_owned()];

        let result = execute(&plugin(), &args);

        assert!(result.is_ok());
        assert!(dir.join("plugin.toml").exists());
        assert!(dir.join("lib/bridge.luau").exists());
        assert!(dir.join("translations/en.json").exists());
        assert!(dir.join("service.luau").exists());
        fs::remove_dir_all(dir).expect("cleanup");
    }

    #[test]
    fn test_execute_reports_missing_arguments() {
        let args: Vec<String> = Vec::new();

        let error = execute(&plugin(), &args).expect_err("missing entry");

        assert_eq!(error.to_string(), "missing entry");
    }

    #[test]
    fn test_execute_dispatches_entry() {
        let args = vec![
            "service".to_owned(),
            "update".to_owned(),
            "/tmp".to_owned(),
            "{}".to_owned(),
            "{}".to_owned(),
            "{}".to_owned(),
            "{}".to_owned(),
        ];

        let result = execute(&plugin(), &args);

        assert!(result.is_ok());
    }
}
