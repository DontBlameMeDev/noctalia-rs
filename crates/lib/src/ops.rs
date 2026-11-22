use serde::Serialize;
use serde_json::Value;

use crate::ui::{UiNode, callback};

#[derive(Clone, Debug, Serialize)]
#[serde(tag = "op", rename_all = "snake_case")]
pub enum Op {
    Log {
        message: String,
    },
    Notify {
        title: String,
        body: String,
    },
    NotifyError {
        title: String,
        body: String,
    },
    SetInterval {
        ms: u64,
    },
    StateSet {
        key: String,
        value: Value,
    },
    TogglePanel {
        id: String,
    },
    OpenSettings,
    BarText {
        text: String,
    },
    BarColor {
        role: String,
    },
    BarVisible {
        visible: bool,
    },
    BarTooltip {
        value: Value,
    },
    BarImage {
        path: String,
        preserve: bool,
        width: u32,
        height: u32,
    },
    BarRender {
        tree: UiNode,
    },
    PanelRender {
        tree: UiNode,
    },
    DesktopWidgetRender {
        tree: UiNode,
    },
    LayerShow {
        id: String,
    },
    LayerHide {
        id: String,
    },
    LayerRender {
        tree: UiNode,
    },
    LayerConfigure {
        id: String,
        props: Value,
    },
    LauncherResults {
        text: String,
        rows: Vec<Value>,
    },
    RemoveFile {
        path: String,
    },
    HostCall {
        root: String,
        path: Vec<String>,
        args: Vec<Value>,
    },
}

impl Op {
    #[inline]
    #[must_use]
    pub fn log(message: impl Into<String>) -> Self {
        Self::Log {
            message: message.into(),
        }
    }

    #[inline]
    #[must_use]
    pub fn state(key: &str, value: Value) -> Self {
        Self::StateSet {
            key: key.to_owned(),
            value,
        }
    }

    #[inline]
    #[must_use]
    pub fn host_call(root: &str, path: &[&str], args: Vec<Value>) -> Self {
        Self::HostCall {
            root: root.to_owned(),
            path: path.iter().map(|item| (*item).to_owned()).collect(),
            args,
        }
    }

    #[inline]
    #[must_use]
    pub fn http(request: Value, callback_id: Option<&str>) -> Self {
        let args = match callback_id {
            Some(id) => vec![request, callback(id)],
            None => vec![request],
        };
        Self::host_call("noctalia", &["http"], args)
    }

    #[inline]
    #[must_use]
    pub fn run(command: Value, callback_id: Option<&str>) -> Self {
        let args = match callback_id {
            Some(id) => vec![command, callback(id)],
            None => vec![command],
        };
        Self::host_call("noctalia", &["runAsync"], args)
    }

    #[inline]
    #[must_use]
    pub fn copy(text: &str) -> Self {
        Self::host_call(
            "noctalia",
            &["copyToClipboard"],
            vec![Value::String(text.to_owned())],
        )
    }

    #[inline]
    #[must_use]
    pub fn open_path(path: &str) -> Self {
        Self::host_call(
            "noctalia",
            &["openPath"],
            vec![Value::String(path.to_owned())],
        )
    }

    #[inline]
    #[must_use]
    pub fn osd(icon: &str, value: &str, extra: Option<Value>) -> Self {
        let mut args = vec![
            Value::String(icon.to_owned()),
            Value::String(value.to_owned()),
        ];
        if let Some(options) = extra {
            args.push(options);
        }
        Self::host_call("noctalia", &["osd", "show"], args)
    }

    #[inline]
    #[must_use]
    pub fn set_wallpaper(path: &str) -> Self {
        Self::host_call(
            "noctalia",
            &["setWallpaper"],
            vec![Value::String(path.to_owned())],
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use serde_json::json;
    #[test]
    fn test_log_serializes_with_op_tag() {
        let value = serde_json::to_value(Op::log("hello")).expect("serialize");

        assert_eq!(value, json!({ "op": "log", "message": "hello" }));
    }

    #[test]
    fn test_state_serializes_value() {
        let value = serde_json::to_value(Op::state("key", json!([1, 2])))
            .expect("serialize");

        assert_eq!(
            value,
            json!({ "op": "state_set", "key": "key", "value": [1, 2] })
        );
    }

    #[test]
    fn test_host_call_serializes_path_and_args() {
        let value = serde_json::to_value(Op::host_call(
            "noctalia",
            &["state", "get"],
            vec![json!("key")],
        ))
        .expect("serialize");

        assert_eq!(
            value,
            json!({
                "op": "host_call",
                "root": "noctalia",
                "path": ["state", "get"],
                "args": ["key"]
            })
        );
    }

    #[test]
    fn test_http_embeds_callback_marker() {
        let value =
            serde_json::to_value(Op::http(json!({ "url": "x" }), Some("done")))
                .expect("serialize");

        assert_eq!(value["args"][1], json!({ "__host_callback": "done" }));
    }

    #[test]
    fn test_copy_builds_clipboard_call() {
        let value = serde_json::to_value(Op::copy("text")).expect("serialize");

        assert_eq!(value["path"], json!(["copyToClipboard"]));
        assert_eq!(value["args"], json!(["text"]));
    }

    #[test]
    fn test_run_uses_host_run_async() {
        let value = serde_json::to_value(Op::run(json!(["echo"]), None))
            .expect("serialize");

        assert_eq!(value["path"], json!(["runAsync"]));
        assert_eq!(value["args"], json!([["echo"]]));
    }

    #[test]
    fn test_open_path_and_wallpaper_use_expected_paths() {
        let open =
            serde_json::to_value(Op::open_path("/tmp")).expect("serialize");
        let wallpaper = serde_json::to_value(Op::set_wallpaper("/tmp/a.png"))
            .expect("serialize");

        assert_eq!(open["path"], json!(["openPath"]));
        assert_eq!(wallpaper["path"], json!(["setWallpaper"]));
    }

    #[test]
    fn test_osd_includes_optional_options() {
        let value = serde_json::to_value(Op::osd(
            "volume",
            "50",
            Some(json!({ "progress": 0.5 })),
        ))
        .expect("serialize");

        assert_eq!(value["path"], json!(["osd", "show"]));
        assert_eq!(value["args"][2]["progress"], 0.5);
    }

    #[test]
    fn test_http_without_callback_has_one_argument() {
        let value = serde_json::to_value(Op::http(json!({ "url": "x" }), None))
            .expect("serialize");

        assert_eq!(value["args"].as_array().expect("args").len(), 1);
    }
}
