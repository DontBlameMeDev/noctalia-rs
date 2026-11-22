use std::collections::BTreeMap;
use std::io;
use std::marker::PhantomData;
use std::path::Path;

use serde_json::{Map, Value};

use crate::Result;
use crate::codegen;
use crate::context::Context;
use crate::ops::Op;

pub type Handler = fn(&Context) -> Result<Vec<Op>>;

type SelectConfig<'a> = (&'a str, &'a [(&'a str, &'a str)]);
pub type NumberConfig = (f64, Option<f64>, Option<f64>);
type FileConfig<'a> = (&'a str, &'a [&'a str]);

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
    pub dependencies: Vec<String>,
    pub deprecated: bool,
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
            plugin_api: 57,
            license: "MIT".to_owned(),
            description: String::new(),
            tags: Vec::new(),
            icon: String::new(),
            dependencies: Vec::new(),
            deprecated: false,
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

    #[inline]
    #[must_use]
    pub fn license(mut self, value: &str) -> Self {
        value.clone_into(&mut self.license);
        self
    }

    #[inline]
    #[must_use]
    pub fn dependencies(mut self, values: &[&str]) -> Self {
        self.dependencies =
            values.iter().map(|value| (*value).to_owned()).collect();
        self
    }

    #[inline]
    #[must_use]
    pub const fn deprecated(mut self, value: bool) -> Self {
        self.deprecated = value;
        self
    }
}

pub struct NeedId;
pub struct NeedName;
pub struct NeedVersion;
pub struct MetaReady;

pub struct MetaBuilder<S> {
    meta: Meta,
    marker: PhantomData<S>,
}

impl MetaBuilder<NeedId> {
    #[inline]
    #[must_use]
    pub fn new() -> Self {
        Self {
            meta: Meta::new("", "", ""),
            marker: PhantomData,
        }
    }

    #[inline]
    #[must_use]
    pub fn id(mut self, value: &str) -> MetaBuilder<NeedName> {
        value.clone_into(&mut self.meta.id);
        MetaBuilder {
            meta: self.meta,
            marker: PhantomData,
        }
    }
}

impl MetaBuilder<NeedName> {
    #[inline]
    #[must_use]
    pub fn name(mut self, value: &str) -> MetaBuilder<NeedVersion> {
        value.clone_into(&mut self.meta.name);
        MetaBuilder {
            meta: self.meta,
            marker: PhantomData,
        }
    }
}

impl MetaBuilder<NeedVersion> {
    #[inline]
    #[must_use]
    pub fn version(mut self, value: &str) -> MetaBuilder<MetaReady> {
        value.clone_into(&mut self.meta.version);
        MetaBuilder {
            meta: self.meta,
            marker: PhantomData,
        }
    }
}

impl<S> MetaBuilder<S> {
    #[inline]
    #[must_use]
    pub fn author(mut self, value: &str) -> Self {
        self.meta = self.meta.author(value);
        self
    }

    #[inline]
    #[must_use]
    pub fn api(mut self, value: u32) -> Self {
        self.meta = self.meta.plugin_api(value);
        self
    }

    #[inline]
    #[must_use]
    pub fn description(mut self, value: &str) -> Self {
        self.meta = self.meta.description(value);
        self
    }

    #[inline]
    #[must_use]
    pub fn tags(mut self, values: &[&str]) -> Self {
        self.meta = self.meta.tags(values);
        self
    }

    #[inline]
    #[must_use]
    pub fn icon(mut self, value: &str) -> Self {
        self.meta = self.meta.icon(value);
        self
    }

    #[inline]
    #[must_use]
    pub fn license(mut self, value: &str) -> Self {
        self.meta = self.meta.license(value);
        self
    }

    #[inline]
    #[must_use]
    pub fn dependencies(mut self, values: &[&str]) -> Self {
        self.meta = self.meta.dependencies(values);
        self
    }

    #[inline]
    #[must_use]
    pub fn deprecated(mut self, value: bool) -> Self {
        self.meta = self.meta.deprecated(value);
        self
    }
}

impl MetaBuilder<MetaReady> {
    #[inline]
    #[must_use]
    pub fn build(self) -> Meta {
        self.meta
    }
}

impl Default for MetaBuilder<NeedId> {
    #[inline]
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Clone, Debug)]
pub struct LayerConfig {
    pub layer: String,
    pub anchors: Vec<String>,
    pub visible: bool,
    pub keyboard: String,
    pub output: String,
    pub click_through: Option<bool>,
    pub exclusive_zone: Option<i32>,
    pub margins: BTreeMap<String, i32>,
    pub width: Option<u32>,
    pub height: Option<u32>,
}

impl Default for LayerConfig {
    #[inline]
    fn default() -> Self {
        Self {
            layer: "top".to_owned(),
            anchors: Vec::new(),
            visible: false,
            keyboard: "none".to_owned(),
            output: "all".to_owned(),
            click_through: None,
            exclusive_zone: None,
            margins: BTreeMap::new(),
            width: None,
            height: None,
        }
    }
}

impl LayerConfig {
    #[inline]
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    #[inline]
    #[must_use]
    pub fn layer(mut self, value: &str) -> Self {
        value.clone_into(&mut self.layer);
        self
    }

    #[inline]
    #[must_use]
    pub fn anchors(mut self, values: &[&str]) -> Self {
        self.anchors = values.iter().map(|value| (*value).to_owned()).collect();
        self
    }

    #[inline]
    #[must_use]
    pub const fn visible(mut self, value: bool) -> Self {
        self.visible = value;
        self
    }

    #[inline]
    #[must_use]
    pub fn keyboard(mut self, value: &str) -> Self {
        value.clone_into(&mut self.keyboard);
        self
    }

    #[inline]
    #[must_use]
    pub fn output(mut self, value: &str) -> Self {
        value.clone_into(&mut self.output);
        self
    }

    #[inline]
    #[must_use]
    pub const fn click_through(mut self, value: bool) -> Self {
        self.click_through = Some(value);
        self
    }

    #[inline]
    #[must_use]
    pub const fn exclusive_zone(mut self, value: i32) -> Self {
        self.exclusive_zone = Some(value);
        self
    }

    #[inline]
    #[must_use]
    pub fn margins(mut self, values: &[(&str, i32)]) -> Self {
        self.margins = values
            .iter()
            .map(|(key, value)| ((*key).to_owned(), *value))
            .collect();
        self
    }

    #[inline]
    #[must_use]
    pub const fn width(mut self, value: u32) -> Self {
        self.width = Some(value);
        self
    }

    #[inline]
    #[must_use]
    pub const fn height(mut self, value: u32) -> Self {
        self.height = Some(value);
        self
    }
}

#[derive(Clone, Debug)]
pub enum EntryKind {
    Service,
    Widget,
    Shortcut,
    LauncherProvider { prefix: String, glyph: String },
    DesktopWidget,
    Panel { width: u32, height: u32 },
    Layer { config: LayerConfig },
}
#[derive(Clone, Debug)]
pub struct LauncherCategory {
    pub label: String,
    pub glyph: String,
}

impl LauncherCategory {
    #[inline]
    #[must_use]
    pub fn new(label: &str, glyph: &str) -> Self {
        Self {
            label: label.to_owned(),
            glyph: glyph.to_owned(),
        }
    }
}

#[derive(Clone, Debug, Default)]
pub struct EntryConfig {
    pub actions: Vec<String>,
    pub hover_highlight: Option<bool>,
    pub placement: Option<String>,
    pub position: Option<String>,
    pub open_near_click: Option<bool>,
    pub dismiss_on_outside_click: Option<bool>,
    pub keyboard_focus: Option<bool>,
    pub max_height: Option<u32>,
    pub capture_keys: Option<bool>,
    pub persistent: Option<bool>,
    pub include_in_global_search: Option<bool>,
    pub debounce_ms: Option<u32>,
    pub category: Option<String>,
    pub script: Option<String>,
    pub action_map: BTreeMap<String, String>,
    pub keyboard_focus_text: Option<String>,
    pub capture_keys_list: Vec<String>,
    pub categories: Vec<LauncherCategory>,
    pub width_text: Option<String>,
    pub height_text: Option<String>,
    pub panel_layer: Option<String>,
}

impl EntryConfig {
    #[inline]
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    #[inline]
    #[must_use]
    pub fn actions(mut self, values: &[&str]) -> Self {
        self.actions = values.iter().map(|value| (*value).to_owned()).collect();
        self
    }

    #[inline]
    #[must_use]
    pub const fn hover_highlight(mut self, value: bool) -> Self {
        self.hover_highlight = Some(value);
        self
    }

    #[inline]
    #[must_use]
    pub fn placement(mut self, value: &str) -> Self {
        self.placement = Some(value.to_owned());
        self
    }

    #[inline]
    #[must_use]
    pub fn position(mut self, value: &str) -> Self {
        self.position = Some(value.to_owned());
        self
    }

    #[inline]
    #[must_use]
    pub const fn open_near_click(mut self, value: bool) -> Self {
        self.open_near_click = Some(value);
        self
    }

    #[inline]
    #[must_use]
    pub const fn dismiss_on_outside_click(mut self, value: bool) -> Self {
        self.dismiss_on_outside_click = Some(value);
        self
    }

    #[inline]
    #[must_use]
    pub const fn keyboard_focus(mut self, value: bool) -> Self {
        self.keyboard_focus = Some(value);
        self
    }

    #[inline]
    #[must_use]
    pub const fn max_height(mut self, value: u32) -> Self {
        self.max_height = Some(value);
        self
    }

    #[inline]
    #[must_use]
    pub const fn capture_keys(mut self, value: bool) -> Self {
        self.capture_keys = Some(value);
        self
    }

    #[inline]
    #[must_use]
    pub const fn persistent(mut self, value: bool) -> Self {
        self.persistent = Some(value);
        self
    }

    #[inline]
    #[must_use]
    pub const fn include_in_global_search(mut self, value: bool) -> Self {
        self.include_in_global_search = Some(value);
        self
    }

    #[inline]
    #[must_use]
    pub const fn debounce_ms(mut self, value: u32) -> Self {
        self.debounce_ms = Some(value);
        self
    }

    #[inline]
    #[must_use]
    pub fn category(mut self, value: &str) -> Self {
        self.category = Some(value.to_owned());
        self
    }

    #[inline]
    #[must_use]
    pub fn script(mut self, value: &str) -> Self {
        self.script = Some(value.to_owned());
        self
    }

    #[inline]
    #[must_use]
    pub fn action_map(mut self, values: &[(&str, &str)]) -> Self {
        self.action_map = values
            .iter()
            .map(|(key, value)| ((*key).to_owned(), (*value).to_owned()))
            .collect();
        self
    }

    #[inline]
    #[must_use]
    pub fn keyboard_focus_text(mut self, value: &str) -> Self {
        self.keyboard_focus_text = Some(value.to_owned());
        self
    }

    #[inline]
    #[must_use]
    pub fn capture_keys_list(mut self, values: &[&str]) -> Self {
        self.capture_keys_list =
            values.iter().map(|value| (*value).to_owned()).collect();
        self
    }

    #[inline]
    #[must_use]
    pub fn categories(mut self, values: Vec<LauncherCategory>) -> Self {
        self.categories = values;
        self
    }

    #[inline]
    #[must_use]
    pub fn width_text(mut self, value: &str) -> Self {
        self.width_text = Some(value.to_owned());
        self
    }

    #[inline]
    #[must_use]
    pub fn height_text(mut self, value: &str) -> Self {
        self.height_text = Some(value.to_owned());
        self
    }

    #[inline]
    #[must_use]
    pub fn panel_layer(mut self, value: &str) -> Self {
        self.panel_layer = Some(value.to_owned());
        self
    }
}

#[derive(Clone, Debug)]
pub enum SettingKind {
    Bool(bool),
    String(String),
    StringList(Vec<String>),
    StringMap(BTreeMap<String, String>),
    Int(NumberConfig),
    Double(NumberConfig),
    Select {
        default: String,
        options: Vec<(String, String)>,
    },
    File {
        default: String,
        extensions: Vec<String>,
    },
    Folder(String),
    Glyph(String),
    Color(String),
}

#[derive(Clone, Debug)]
pub struct Setting {
    pub key: String,
    pub label_key: String,
    pub description_key: Option<String>,
    pub kind: SettingKind,
    pub visible_when: Option<(String, Vec<String>)>,
    pub step: Option<f64>,
    pub advanced: Option<bool>,
}

impl Setting {
    #[inline]
    #[must_use]
    pub fn bool(key: &str, label_key: &str, default: bool) -> Self {
        Self::new(key, label_key, SettingKind::Bool(default))
    }

    #[inline]
    #[must_use]
    pub fn string(key: &str, label_key: &str, default: &str) -> Self {
        Self::new(key, label_key, SettingKind::String(default.to_owned()))
    }

    #[inline]
    #[must_use]
    pub fn string_list(key: &str, label_key: &str, values: &[&str]) -> Self {
        let value_list =
            values.iter().map(|value| (*value).to_owned()).collect();
        Self::new(key, label_key, SettingKind::StringList(value_list))
    }

    #[inline]
    #[must_use]
    pub fn string_map(
        key: &str,
        label_key: &str,
        values: &[(&str, &str)],
    ) -> Self {
        let map_entries = values
            .iter()
            .map(|(key, value)| ((*key).to_owned(), (*value).to_owned()))
            .collect();
        Self::new(key, label_key, SettingKind::StringMap(map_entries))
    }

    #[inline]
    #[must_use]
    pub fn int(key: &str, label_key: &str, config: NumberConfig) -> Self {
        Self::new(key, label_key, SettingKind::Int(config))
    }

    #[inline]
    #[must_use]
    pub fn double(key: &str, label_key: &str, config: NumberConfig) -> Self {
        Self::new(key, label_key, SettingKind::Double(config))
    }

    #[inline]
    #[must_use]
    pub fn select(
        key: &str,
        label_key: &str,
        config: SelectConfig<'_>,
    ) -> Self {
        let (default, option_values) = config;
        let options = option_values
            .iter()
            .map(|(value, label)| ((*value).to_owned(), (*label).to_owned()))
            .collect();
        Self::new(
            key,
            label_key,
            SettingKind::Select {
                default: default.to_owned(),
                options,
            },
        )
    }

    #[inline]
    #[must_use]
    pub fn file(key: &str, label_key: &str, config: FileConfig<'_>) -> Self {
        let (default, extensions) = config;
        Self::new(
            key,
            label_key,
            SettingKind::File {
                default: default.to_owned(),
                extensions: extensions
                    .iter()
                    .map(|value| (*value).to_owned())
                    .collect(),
            },
        )
    }

    #[inline]
    #[must_use]
    pub fn folder(key: &str, label_key: &str, default: &str) -> Self {
        Self::new(key, label_key, SettingKind::Folder(default.to_owned()))
    }

    #[inline]
    #[must_use]
    pub fn glyph(key: &str, label_key: &str, default: &str) -> Self {
        Self::new(key, label_key, SettingKind::Glyph(default.to_owned()))
    }

    #[inline]
    #[must_use]
    pub fn color(key: &str, label_key: &str, default: &str) -> Self {
        Self::new(key, label_key, SettingKind::Color(default.to_owned()))
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

    #[inline]
    #[must_use]
    pub const fn step(mut self, value: f64) -> Self {
        self.step = Some(value);
        self
    }

    #[inline]
    #[must_use]
    pub const fn advanced(mut self, value: bool) -> Self {
        self.advanced = Some(value);
        self
    }

    fn new(key: &str, label_key: &str, kind: SettingKind) -> Self {
        Self {
            key: key.to_owned(),
            label_key: label_key.to_owned(),
            description_key: None,
            kind,
            visible_when: None,
            step: None,
            advanced: None,
        }
    }
}

#[derive(Clone)]
pub struct Entry {
    pub id: String,
    pub kind: EntryKind,
    pub settings: Vec<Setting>,
    pub config: EntryConfig,
    pub handler: Handler,
}

impl Entry {
    #[inline]
    #[must_use]
    pub fn new(
        spec: (String, EntryKind, Vec<Setting>),
        handler: Handler,
    ) -> Self {
        let (id, kind, settings) = spec;
        Self::configured((id, kind, settings, EntryConfig::new()), handler)
    }

    #[inline]
    #[must_use]
    pub fn configured(
        spec: (String, EntryKind, Vec<Setting>, EntryConfig),
        handler: Handler,
    ) -> Self {
        let (id, kind, settings, config) = spec;
        Self {
            id,
            kind,
            settings,
            config,
            handler,
        }
    }
}

pub struct Plugin {
    pub meta: Meta,
    pub entries: Vec<Entry>,
    pub settings: Vec<Setting>,
    pub translations: BTreeMap<String, String>,
    pub assets: BTreeMap<String, Vec<u8>>,
    pub translation_files: BTreeMap<String, String>,
}

impl Plugin {
    #[inline]
    #[must_use]
    pub const fn new(meta: Meta) -> Self {
        Self {
            meta,
            entries: Vec::new(),
            settings: Vec::new(),
            translations: BTreeMap::new(),
            assets: BTreeMap::new(),
            translation_files: BTreeMap::new(),
        }
    }

    #[inline]
    #[must_use]
    pub fn entry(mut self, entry: Entry) -> Self {
        self.entries.push(entry);
        self
    }

    #[inline]
    #[must_use]
    pub fn service(mut self, id: &str, handler: Handler) -> Self {
        self.entries.push(Entry::new(
            (id.to_owned(), EntryKind::Service, Vec::new()),
            handler,
        ));
        self
    }

    #[inline]
    #[must_use]
    pub fn widget(mut self, id: &str, spec: (Vec<Setting>, Handler)) -> Self {
        let (settings, handler) = spec;
        self.entries.push(Entry::new(
            (id.to_owned(), EntryKind::Widget, settings),
            handler,
        ));
        self
    }

    #[inline]
    #[must_use]
    pub fn shortcut(mut self, id: &str, handler: Handler) -> Self {
        self.entries.push(Entry::new(
            (id.to_owned(), EntryKind::Shortcut, Vec::new()),
            handler,
        ));
        self
    }

    #[inline]
    #[must_use]
    pub fn launcher(mut self, id: &str, spec: (&str, &str, Handler)) -> Self {
        let (prefix, glyph, handler) = spec;
        self.entries.push(Entry::new(
            (
                id.to_owned(),
                EntryKind::LauncherProvider {
                    prefix: prefix.to_owned(),
                    glyph: glyph.to_owned(),
                },
                Vec::new(),
            ),
            handler,
        ));
        self
    }

    #[inline]
    #[must_use]
    pub fn desktop_widget(mut self, id: &str, handler: Handler) -> Self {
        self.entries.push(Entry::new(
            (id.to_owned(), EntryKind::DesktopWidget, Vec::new()),
            handler,
        ));
        self
    }

    #[inline]
    #[must_use]
    pub fn panel(mut self, id: &str, spec: (u32, u32, Handler)) -> Self {
        let (width, height, handler) = spec;
        self.entries.push(Entry::new(
            (
                id.to_owned(),
                EntryKind::Panel { width, height },
                Vec::new(),
            ),
            handler,
        ));
        self
    }

    #[inline]
    #[must_use]
    pub fn layer(mut self, id: &str, spec: (LayerConfig, Handler)) -> Self {
        let (config, handler) = spec;
        self.entries.push(Entry::new(
            (id.to_owned(), EntryKind::Layer { config }, Vec::new()),
            handler,
        ));
        self
    }

    #[inline]
    #[must_use]
    pub fn setting(mut self, setting: Setting) -> Self {
        self.settings.push(setting);
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
    pub fn translation_file(mut self, language: &str, content: &str) -> Self {
        self.translation_files
            .insert(language.to_owned(), content.to_owned());
        self
    }

    #[inline]
    #[must_use]
    pub fn asset(mut self, path: &str, content: &[u8]) -> Self {
        self.assets.insert(path.to_owned(), content.to_vec());
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
    use crate::Parts;

    fn empty_plugin() -> Plugin {
        Plugin::new(Meta::new("me/hello", "Hello", "0.1.0"))
    }

    #[test]
    fn test_meta_builder_requires_identity_fields() {
        let meta = MetaBuilder::new()
            .id("me/hello")
            .name("Hello")
            .version("0.1.0")
            .author("Author")
            .api(57)
            .build();

        assert_eq!(meta.id, "me/hello");
        assert_eq!(meta.plugin_api, 57);
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
    fn test_entry_new_uses_tuple_spec() {
        let entry = Entry::new(
            ("service".to_owned(), EntryKind::Service, Vec::new()),
            |_ctx| Ok(Vec::new()),
        );

        assert_eq!(entry.id, "service");
    }

    #[test]
    fn test_dispatch_rejects_unknown_entry() {
        let parts = Parts {
            entry: "missing".to_owned(),
            event: "update".to_owned(),
            payload: Value::Null,
            config: Value::Null,
            state: Value::Null,
            host: Value::Null,
            data_dir: Path::new("/tmp").to_path_buf(),
        };
        let context = Context::new(parts, Value::Null);
        let plugin = empty_plugin();

        let error = plugin.dispatch(&context).expect_err("unknown entry");

        assert_eq!(error.to_string(), "unknown entry missing");
    }
}
