use serde::Serialize;
use serde_json::{json, Value};

#[derive(Clone, Debug, Serialize)]
pub struct UiNode {
    kind: String,
    props: Value,
    children: Vec<Self>,
}

#[inline]
#[must_use]
pub fn node(kind: &str, props: Value, children: Vec<UiNode>) -> UiNode {
    UiNode {
        kind: kind.to_owned(),
        props,
        children,
    }
}

#[inline]
#[must_use]
pub fn column(props: Value, children: Vec<UiNode>) -> UiNode {
    node("column", props, children)
}

#[inline]
#[must_use]
pub fn row(props: Value, children: Vec<UiNode>) -> UiNode {
    node("row", props, children)
}

#[inline]
#[must_use]
pub fn scroll(props: Value, children: Vec<UiNode>) -> UiNode {
    node("scroll", props, children)
}

#[inline]
#[must_use]
pub fn label(props: Value) -> UiNode {
    node("label", props, Vec::new())
}

#[inline]
#[must_use]
pub fn button(props: Value) -> UiNode {
    node("button", props, Vec::new())
}

#[inline]
#[must_use]
pub fn input(props: Value) -> UiNode {
    node("input", props, Vec::new())
}

#[inline]
#[must_use]
pub fn progress(props: Value) -> UiNode {
    node("progress", props, Vec::new())
}

#[inline]
#[must_use]
pub fn call(name: &str) -> Value {
    json!({ "__call": name })
}
