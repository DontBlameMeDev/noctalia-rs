use std::collections::BTreeMap;
use std::io;
use std::path::Path;

use serde_json::{Map, Value};

use crate::Result;
use crate::codegen;
use crate::context::Context;
use crate::ops::Op;

pub type Handler = fn(&Context) -> Result<Vec<Op>>;
type SelectConfig<'a> = (&'a str, &'a [(&'a str, &'a str)]);

#[derive(Clone, Debug)]
pub struct Meta {
    pub id: String,
    pub name: String,
    pub version: String,
    pub author: String,
    pub plugin_api: u32,
    pub license: String,
    pub description: String,
    pub tags: Vec<String>,
    pub icon: String,
}

impl Meta {
    #[inline]
    #[must_use]
    pub fn new(id: &str, name: &str, version: &str) -> Self {
        Self {
            id: id.to_owned(),
            name: name.to_owned(),
            version: version.to_owned(),
            author: String::new(),
            plugin_api: 24,
            license: "MIT".to_owned(),
            description: String::new(),
            tags: Vec::new(),
            icon: String::new(),
        }
    }

    #[inline]
    #[must_use]
    pub fn author(mut self, value: &str) -> Self {
        value.clone_into(&mut self.author);
        self
    }

    #[inline]
    #[must_use]
    pub const fn plugin_api(mut self, value: u32) -> Self {
        self.plugin_api = value;
        self
    }

    #[inline]
    #[must_use]
    pub fn description(mut self, value: &str) -> Self {
        value.clone_into(&mut self.description);
        self
    }

    #[inline]
    #[must_use]
    pub fn tags(mut self, values: &[&str]) -> Self {
        self.tags = values.iter().map(|value| (*value).to_owned()).collect();
        self
    }

    #[inline]
    #[must_use]
    pub fn icon(mut self, value: &str) -> Self {
        value.clone_into(&mut self.icon);
        self
    }
}

#[derive(Clone, Debug)]
pub enum EntryKind {
    Service,
    Widget,
    LauncherProvider {
        prefix: String,
        glyph: String,
    },
    Panel {
        width: u32,
        height: u32,
        placement: String,
        position: String,
        open_near_click: bool,
    },
}

#[derive(Clone, Debug)]
pub enum SettingKind {
    Bool(bool),
    String(String),
    Select {
        default: String,
        options: Vec<(String, String)>,
    },
}

#[derive(Clone, Debug)]
pub struct Setting {
    pub key: String,
    pub label_key: String,
    pub description_key: Option<String>,
    pub kind: SettingKind,
    pub visible_when: Option<(String, Vec<String>)>,
}

impl Setting {
    #[inline]
    #[must_use]
    pub fn bool(key: &str, label_key: &str, default: bool) -> Self {
        Self {
            key: key.to_owned(),
            label_key: label_key.to_owned(),
            description_key: None,
            kind: SettingKind::Bool(default),
            visible_when: None,
        }
    }

    #[inline]
    #[must_use]
    pub fn string(key: &str, label_key: &str, default: &str) -> Self {
        Self {
            key: key.to_owned(),
            label_key: label_key.to_owned(),
            description_key: None,
            kind: SettingKind::String(default.to_owned()),
            visible_when: None,
        }
    }

    #[inline]
    #[must_use]
    pub fn select(
        key: &str,
        label_key: &str,
        config: SelectConfig<'_>,
    ) -> Self {
        let (default, option_values) = config;
        let option_list = option_values
            .iter()
            .map(|(value, label)| ((*value).to_owned(), (*label).to_owned()))
            .collect();

        Self {
            key: key.to_owned(),
            label_key: label_key.to_owned(),
            description_key: None,
            kind: SettingKind::Select {
                default: default.to_owned(),
                options: option_list,
            },
            visible_when: None,
        }
    }

    #[inline]
    #[must_use]
    pub fn description(mut self, key: &str) -> Self {
        self.description_key = Some(key.to_owned());
        self
    }

    #[inline]
    #[must_use]
    pub fn visible_when(mut self, key: &str, values: &[&str]) -> Self {
        self.visible_when = Some((
            key.to_owned(),
            values.iter().map(|value| (*value).to_owned()).collect(),
        ));
        self
    }
}

#[derive(Clone)]
pub struct Entry {
    pub id: String,
    pub kind: EntryKind,
    pub settings: Vec<Setting>,
    pub handler: Handler,
}

pub struct Plugin {
    pub meta: Meta,
    pub entries: Vec<Entry>,
    pub translations: BTreeMap<String, String>,
}

impl Plugin {
    #[inline]
    #[must_use]
    pub const fn new(meta: Meta) -> Self {
        Self {
            meta,
            entries: Vec::new(),
            translations: BTreeMap::new(),
        }
    }

    #[inline]
    #[must_use]
    pub fn service(mut self, id: &str, handler: Handler) -> Self {
        self.entries.push(Entry {
            id: id.to_owned(),
            kind: EntryKind::Service,
            settings: Vec::new(),
            handler,
        });
        self
    }

    #[inline]
    #[must_use]
    pub fn widget(mut self, id: &str, spec: (Vec<Setting>, Handler)) -> Self {
        let (settings, handler) = spec;
        self.entries.push(Entry {
            id: id.to_owned(),
            kind: EntryKind::Widget,
            settings,
            handler,
        });
        self
    }

    #[inline]
    #[must_use]
    pub fn launcher(mut self, id: &str, spec: (&str, &str, Handler)) -> Self {
        let (prefix, glyph, handler) = spec;
        self.entries.push(Entry {
            id: id.to_owned(),
            kind: EntryKind::LauncherProvider {
                prefix: prefix.to_owned(),
                glyph: glyph.to_owned(),
            },
            settings: Vec::new(),
            handler,
        });
        self
    }

    #[inline]
    #[must_use]
    pub fn panel(mut self, id: &str, spec: (u32, u32, Handler)) -> Self {
        let (width, height, handler) = spec;
        self.entries.push(Entry {
            id: id.to_owned(),
            kind: EntryKind::Panel {
                width,
                height,
                placement: "floating".to_owned(),
                position: "center".to_owned(),
                open_near_click: false,
            },
            settings: Vec::new(),
            handler,
        });
        self
    }

    #[inline]
    #[must_use]
    pub fn tr(mut self, key: &str, value: &str) -> Self {
        self.translations.insert(key.to_owned(), value.to_owned());
        self
    }

    #[inline]
    #[must_use]
    pub fn state_key(&self) -> String {
        self.meta
            .id
            .split('/')
            .next_back()
            .unwrap_or(&self.meta.id)
            .to_owned()
    }

    #[must_use]
    pub fn translations_value(&self) -> Value {
        let mut root = Map::new();

        for (key, value) in &self.translations {
            let parts: Vec<&str> = key.split('.').collect();
            insert_translation(&mut root, &parts, value);
        }

        Value::Object(root)
    }

    pub fn dispatch(&self, ctx: &Context) -> Result<Vec<Op>> {
        let entry = self
            .entries
            .iter()
            .find(|entry| entry.id == ctx.entry)
            .ok_or_else(|| format!("unknown entry {}", ctx.entry))?;
        (entry.handler)(ctx)
    }

    #[inline]
    pub fn emit(&self, dir: &Path, helper: &str) -> io::Result<()> {
        codegen::emit(self, dir, helper)
    }
}

fn insert_translation(
    root: &mut Map<String, Value>,
    parts: &[&str],
    value: &str,
) {
    let Some((last, parents)) = parts.split_last() else {
        return;
    };

    let mut cursor = root;

    for part in parents {
        let next = cursor
            .entry((*part).to_owned())
            .or_insert_with(|| Value::Object(Map::new()));
        let Value::Object(map) = next else {
            return;
        };
        cursor = map;
    }

    cursor.insert((*last).to_owned(), Value::String(value.to_owned()));
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::context::Parts;

    fn empty_plugin() -> Plugin {
        Plugin::new(Meta::new("me/hello", "Hello", "0.1.0"))
    }

    #[test]
    fn test_state_key_uses_id_after_slash() {
        let plugin = empty_plugin();

        assert_eq!(plugin.state_key(), "hello");
    }

    #[test]
    fn test_translations_value_nests_dotted_keys() {
        let plugin = empty_plugin().tr("notify.title", "Done");

        assert_eq!(plugin.translations_value()["notify"]["title"], "Done");
    }

    #[test]
    fn test_dispatch_rejects_unknown_entry() {
        let parts = Parts {
            entry: "missing".to_owned(),
            event: "update".to_owned(),
            payload: Value::Null,
            config: Value::Null,
            state: Value::Null,
            data_dir: Path::new("/tmp").to_path_buf(),
        };
        let context = Context::new(parts, Value::Null);
        let plugin = empty_plugin();

        let error = plugin.dispatch(&context).expect_err("unknown entry");

        assert_eq!(error.to_string(), "unknown entry missing");
    }
}
