use serde::Serialize;
use serde_json::Value;

use crate::ui::UiNode;

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
    LauncherResults {
        text: String,
        rows: Vec<Value>,
    },
    RemoveFile {
        path: String,
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
}
