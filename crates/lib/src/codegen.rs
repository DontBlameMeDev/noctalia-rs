use std::fmt::Write as _;
use std::fs;
use std::io;
use std::path::Path;

use crate::bridge::SOURCE;
use crate::manifest::{
    Entry, EntryKind, NumberConfig, Plugin, Setting, SettingKind,
};

pub fn emit(plugin: &Plugin, dir: &Path, helper: &str) -> io::Result<()> {
    fs::create_dir_all(dir.join("lib"))?;
    fs::create_dir_all(dir.join("translations"))?;

    fs::write(dir.join("plugin.toml"), manifest(plugin))?;

    let translations =
        serde_json::to_string_pretty(&plugin.translations_value())
            .map_err(io::Error::other)?;
    fs::write(
        dir.join("translations/en.json"),
        format!("{translations}\n"),
    )?;

    let bridge = SOURCE
        .replace("{{HELPER}}", helper)
        .replace("{{PREFIX}}", &plugin.state_key());
    fs::write(dir.join("lib/bridge.luau"), bridge)?;

    for entry in &plugin.entries {
        let name = format!("{}.luau", entry.id);
        fs::write(dir.join(name), entry_script(entry))?;
    }

    Ok(())
}

fn quote(value: &str) -> String {
    format!("\"{}\"", value.replace('\\', "\\\\").replace('"', "\\\""))
}

fn toml_list(values: &[String]) -> String {
    let items = values.iter().map(|value| quote(value)).collect::<Vec<_>>();
    format!("[{}]", items.join(", "))
}

fn toml_map(values: &[(String, String)]) -> String {
    let items = values
        .iter()
        .map(|(key, value)| format!("{} = {}", quote(key), quote(value)))
        .collect::<Vec<_>>();
    format!("{{ {} }}", items.join(", "))
}

fn manifest(plugin: &Plugin) -> String {
    let meta = &plugin.meta;
    let mut out = format!(
        "id = {}\nname = {}\nversion = {}\nplugin_api = {}\nauthor = {}\nlicense = {}\n",
        quote(&meta.id),
        quote(&meta.name),
        quote(&meta.version),
        meta.plugin_api,
        quote(&meta.author),
        quote(&meta.license)
    );

    if !meta.description.is_empty() {
        let _ = writeln!(out, "description = {}", quote(&meta.description));
    }
    if !meta.tags.is_empty() {
        let _ = writeln!(out, "tags = {}", toml_list(&meta.tags));
    }
    if !meta.icon.is_empty() {
        let _ = writeln!(out, "icon = {}", quote(&meta.icon));
    }

    for entry in &plugin.entries {
        out.push('\n');
        write_entry(&mut out, entry);
    }

    out
}

fn write_entry(out: &mut String, entry: &Entry) {
    let script = format!("{}.luau", entry.id);

    match &entry.kind {
        EntryKind::Service => {
            let _ = write!(
                out,
                "[[service]]\nid = {}\nentry = {}\n",
                quote(&entry.id),
                quote(&script)
            );
        }
        EntryKind::Widget
        | EntryKind::Shortcut
        | EntryKind::DesktopWidget
        | EntryKind::Layer { .. } => {
            let table = match entry.kind {
                EntryKind::Widget => "widget",
                EntryKind::Shortcut => "shortcut",
                EntryKind::DesktopWidget => "desktop_widget",
                EntryKind::Layer { .. } => "layer",
                EntryKind::Service
                | EntryKind::LauncherProvider { .. }
                | EntryKind::Panel { .. } => {
                    unreachable!("matched entry kind")
                }
            };
            let _ = write!(
                out,
                "[[{table}]]\nid = {}\nentry = {}\n",
                quote(&entry.id),
                quote(&script)
            );

            if let EntryKind::Layer { config } = &entry.kind {
                let _ = writeln!(out, "layer = {}", quote(&config.layer));
                let _ =
                    writeln!(out, "anchors = {}", toml_list(&config.anchors));
                let _ = writeln!(out, "visible = {}", config.visible);
                let _ = writeln!(out, "keyboard = {}", quote(&config.keyboard));
                let _ = writeln!(out, "output = {}", quote(&config.output));
            }
            for setting in &entry.settings {
                write_setting(out, setting, table);
            }
        }
        EntryKind::LauncherProvider { prefix, glyph } => {
            let _ = write!(
                out,
                "[[launcher_provider]]\nid = {}\nentry = {}\nprefix = {}\nglyph = {}\ninclude_in_global_search = false\n",
                quote(&entry.id),
                quote(&script),
                quote(prefix),
                quote(glyph)
            );
        }
        EntryKind::Panel { width, height } => {
            let _ = write!(
                out,
                "[[panel]]\nid = {}\nentry = {}\nwidth = {}\nheight = {}\nplacement = \"floating\"\nposition = \"center\"\n",
                quote(&entry.id),
                quote(&script),
                width,
                height
            );
        }
    }
}

fn write_setting(out: &mut String, setting: &Setting, table: &str) {
    let _ = write!(
        out,
        "\n  [[{table}.setting]]\nkey = {}\nlabel_key = {}\n",
        quote(&setting.key),
        quote(&setting.label_key)
    );

    if let Some(key) = &setting.description_key {
        let _ = writeln!(out, "description_key = {}", quote(key));
    }
    if let Some((key, values)) = &setting.visible_when {
        let _ = writeln!(
            out,
            "visible_when = {{ key = {}, values = {} }}",
            quote(key),
            toml_list(values)
        );
    }

    write_setting_kind(out, setting, table);
}

fn write_setting_kind(out: &mut String, setting: &Setting, table: &str) {
    match &setting.kind {
        SettingKind::Bool(value) => {
            let _ = write!(out, "type = \"bool\"\ndefault = {value}\n");
        }
        SettingKind::String(value)
        | SettingKind::Folder(value)
        | SettingKind::Glyph(value)
        | SettingKind::Color(value) => {
            let ty = match setting.kind {
                SettingKind::String(_) => "string",
                SettingKind::Folder(_) => "folder",
                SettingKind::Glyph(_) => "glyph",
                SettingKind::Color(_) => "color",
                SettingKind::Bool(_)
                | SettingKind::StringList(_)
                | SettingKind::StringMap(_)
                | SettingKind::Int(_)
                | SettingKind::Double(_)
                | SettingKind::Select { .. }
                | SettingKind::File { .. } => {
                    unreachable!("matched setting kind")
                }
            };
            let _ =
                write!(out, "type = \"{ty}\"\ndefault = {}\n", quote(value));
        }
        SettingKind::StringList(values) => {
            let _ = write!(
                out,
                "type = \"string_list\"\ndefault = {}\n",
                toml_list(values)
            );
        }
        SettingKind::StringMap(values) => {
            let pairs = values
                .iter()
                .map(|(key, value)| (key.clone(), value.clone()))
                .collect::<Vec<_>>();
            let _ = write!(
                out,
                "type = \"string_map\"\ndefault = {}\n",
                toml_map(&pairs)
            );
        }
        SettingKind::Int(config) => write_number(out, config, false),
        SettingKind::Double(config) => write_number(out, config, true),
        SettingKind::Select { default, options } => {
            let _ = write!(
                out,
                "type = \"select\"\ndefault = {}\n",
                quote(default)
            );
            for (value, label_key) in options {
                let _ = write!(
                    out,
                    "\n  [[{table}.setting.options]]\nvalue = {}\nlabel_key = {}\n",
                    quote(value),
                    quote(label_key)
                );
            }
        }
        SettingKind::File {
            default,
            extensions,
        } => {
            let _ =
                write!(out, "type = \"file\"\ndefault = {}\n", quote(default));
            let _ = writeln!(out, "extensions = {}", toml_list(extensions));
        }
    }
}

fn write_number(out: &mut String, config: &NumberConfig, is_double: bool) {
    let (default, minimum, maximum) = *config;
    let ty = if is_double { "double" } else { "int" };
    let _ = write!(out, "type = \"{ty}\"\ndefault = {default}\n");
    if let Some(value) = minimum {
        let _ = writeln!(out, "min = {value}");
    }
    if let Some(value) = maximum {
        let _ = writeln!(out, "max = {value}");
    }
}

fn entry_script(entry: &Entry) -> String {
    let keys = entry
        .settings
        .iter()
        .map(|setting| quote(&setting.key))
        .collect::<Vec<_>>()
        .join(", ");

    let mut out = format!(
        "--!nonstrict\n\nlocal bridge = require(\"./lib/bridge.luau\")\nbridge.setup({}, {{ {} }})\n\n",
        quote(&entry.id),
        keys
    );

    match &entry.kind {
        EntryKind::Service => {
            out.push_str("function update() bridge.call(\"update\") end\n");
            out.push_str("function onIpc(event, payload) bridge.call(\"ipc\", { event = event, payload = payload }) end\n");
            out.push_str("function onResume() bridge.call(\"resume\") end\n");
            out.push_str("function onExit() bridge.call(\"exit\") end\n\n");
            out.push_str("bridge.call(\"init\")\n");
        }
        EntryKind::Widget => {
            out.push_str("function update() bridge.call(\"render\") end\n");
            out.push_str("function onClick() bridge.call(\"click\") end\n");
            out.push_str("function onExit() bridge.call(\"exit\") end\n");
            out.push_str(
                "bridge.watch_state(function() bridge.call(\"render\") end)\n",
            );
            out.push_str("bridge.call(\"render\")\n");
        }
        EntryKind::Shortcut => {
            out.push_str("function update() bridge.call(\"render\") end\n");
            out.push_str("function onClick() bridge.call(\"click\") end\n");
            out.push_str(
                "bridge.watch_state(function() bridge.call(\"render\") end)\n",
            );
        }
        EntryKind::LauncherProvider { .. } => {
            out.push_str("function onQuery(text) bridge.query(text) end\n");
            out.push_str("function onActivate(id) bridge.call(\"activate\", { id = id }) end\n");
            out.push_str(
                "bridge.watch_state(function() bridge.rerender_query() end)\n",
            );
        }
        EntryKind::DesktopWidget => {
            out.push_str("function update() bridge.call(\"render\") end\n");
            out.push_str("function onFrameTick(delta) bridge.call(\"frame\", { delta = delta }) end\n");
            out.push_str(
                "bridge.watch_state(function() bridge.call(\"render\") end)\n",
            );
            out.push_str("bridge.call(\"render\")\n");
        }
        EntryKind::Panel { .. } => {
            out.push_str("function onOpen() bridge.call(\"open\") end\n");
            out.push_str("function onClose() bridge.call(\"close\") end\n");
            out.push_str("function update() bridge.call(\"render\") end\n");
            out.push_str(
                "bridge.watch_state(function() bridge.call(\"render\") end)\n",
            );
            out.push_str("panel.setWantsSecondTicks(true)\n");
            out.push_str("bridge.call(\"render\")\n");
        }
        EntryKind::Layer { .. } => {
            out.push_str("function update() bridge.call(\"render\") end\n");
            out.push_str("function onFrameTick(delta) bridge.call(\"frame\", { delta = delta }) end\n");
            out.push_str("function onClick() bridge.call(\"click\") end\n");
            out.push_str(
                "bridge.watch_state(function() bridge.call(\"render\") end)\n",
            );
            out.push_str("bridge.call(\"render\")\n");
        }
    }

    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::manifest::{LayerConfig, Meta};
    use crate::{Context, Op};

    fn handler(ctx: &Context) -> crate::Result<Vec<Op>> {
        let _ = ctx.payload_str("ignored").ok_or("missing payload")?;
        Ok(Vec::new())
    }

    #[test]
    fn test_manifest_places_setting_keys_before_options() {
        let plugin = Plugin::new(Meta::new("me/hello", "Hello", "0.1.0"))
            .widget(
                "widget",
                (
                    vec![Setting::select(
                        "color",
                        "settings.color.label",
                        ("primary", &[("primary", "Primary")]),
                    )],
                    handler,
                ),
            );

        let manifest = manifest(&plugin);
        let label = manifest.find("label_key").expect("label key");
        let options = manifest
            .find("[[widget.setting.options]]")
            .expect("options");

        assert!(label < options);
    }

    #[test]
    fn test_manifest_supports_all_entry_kinds() {
        let plugin = Plugin::new(Meta::new("me/hello", "Hello", "0.1.0"))
            .service("service", handler)
            .widget("widget", (Vec::new(), handler))
            .shortcut("shortcut", handler)
            .launcher("launcher", ("timer", "clock", handler))
            .desktop_widget("desktop", handler)
            .panel("panel", (320, 240, handler))
            .layer("layer", (LayerConfig::new(), handler));

        let manifest = manifest(&plugin);

        for table in [
            "[[service]]",
            "[[widget]]",
            "[[shortcut]]",
            "[[launcher_provider]]",
            "[[desktop_widget]]",
            "[[panel]]",
            "[[layer]]",
        ] {
            assert!(manifest.contains(table), "missing {table}");
        }
    }

    #[test]
    fn test_manifest_supports_all_setting_types() {
        let settings = vec![
            Setting::bool("bool", "label.bool", true),
            Setting::string("string", "label.string", "value"),
            Setting::string_list("list", "label.list", &["a", "b"]),
            Setting::string_map("map", "label.map", &[("a", "b")]),
            Setting::int("int", "label.int", (1.0, Some(0.0), Some(5.0))),
            Setting::double("double", "label.double", (1.5, None, None)),
            Setting::select("select", "label.select", ("a", &[("a", "A")])),
            Setting::file("file", "label.file", ("x", &[".txt"])),
            Setting::folder("folder", "label.folder", "/tmp"),
            Setting::glyph("glyph", "label.glyph", "clock"),
            Setting::color("color", "label.color", "primary"),
        ];
        let plugin = Plugin::new(Meta::new("me/hello", "Hello", "0.1.0"))
            .widget("widget", (settings, handler));
        let manifest = manifest(&plugin);

        for ty in [
            "bool",
            "string",
            "string_list",
            "string_map",
            "int",
            "double",
            "select",
            "file",
            "folder",
            "glyph",
            "color",
        ] {
            assert!(
                manifest.contains(&format!("type = \"{ty}\"")),
                "missing {ty}"
            );
        }
    }

    #[test]
    fn test_entry_script_defines_layer_callbacks() {
        let plugin = Plugin::new(Meta::new("me/hello", "Hello", "0.1.0"))
            .layer("layer", (LayerConfig::new(), handler));
        let script = entry_script(&plugin.entries[0]);

        assert!(script.contains("onFrameTick"));
        assert!(script.contains("bridge.call(\"render\")"));
    }
}
