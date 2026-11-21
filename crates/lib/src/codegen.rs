use std::fmt::Write as _;
use std::fs;
use std::io;
use std::path::Path;

use crate::bridge::SOURCE;
use crate::manifest::{Entry, EntryKind, Plugin, Setting, SettingKind};

pub fn emit(plugin: &Plugin, dir: &Path, helper: &str) -> io::Result<()> {
    fs::create_dir_all(dir.join("lib"))?;
    fs::create_dir_all(dir.join("translations"))?;

    fs::write(dir.join("plugin.toml"), manifest(plugin))?;

    let translations =
        serde_json::to_string_pretty(&plugin.translations_value()).map_err(io::Error::other)?;
    fs::write(dir.join("translations/en.json"), format!("{translations}\n"))?;

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
        EntryKind::Widget => {
            let _ = write!(
                out,
                "[[widget]]\nid = {}\nentry = {}\n",
                quote(&entry.id),
                quote(&script)
            );
            for setting in &entry.settings {
                write_widget_setting(out, setting);
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
        EntryKind::Panel {
            width,
            height,
            placement,
            position,
            open_near_click,
        } => {
            let _ = write!(
                out,
                "[[panel]]\nid = {}\nentry = {}\nwidth = {}\nheight = {}\nplacement = {}\nposition = {}\nopen_near_click = {}\n",
                quote(&entry.id),
                quote(&script),
                width,
                height,
                quote(placement),
                quote(position),
                open_near_click
            );
        }
    }
}

fn write_widget_setting(out: &mut String, setting: &Setting) {
    let _ = write!(
        out,
        "\n  [[widget.setting]]\nkey = {}\n",
        quote(&setting.key)
    );

    match &setting.kind {
        SettingKind::Bool(value) => {
            let _ = write!(out, "type = \"bool\"\ndefault = {value}\n");
        }
        SettingKind::String(value) => {
            let _ = write!(out, "type = \"string\"\ndefault = {}\n", quote(value));
        }
        SettingKind::Select { default, options } => {
            let _ = write!(out, "type = \"select\"\ndefault = {}\n", quote(default));
            for (value, label_key) in options {
                let _ = write!(
                    out,
                    "\n  [[widget.setting.options]]\nvalue = {}\nlabel_key = {}\n",
                    quote(value),
                    quote(label_key)
                );
            }
        }
    }

    let _ = writeln!(out, "label_key = {}", quote(&setting.label_key));
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
            out.push_str(
                "function onIpc(event, payload) bridge.call(\"ipc\", { event = event, payload = payload }) end\n",
            );
            out.push_str("function onResume() bridge.call(\"resume\") end\n");
            out.push_str("function onExit() bridge.call(\"exit\") end\n\n");
            out.push_str("bridge.call(\"init\")\n");
        }
        EntryKind::Widget => {
            out.push_str("function update() bridge.call(\"render\") end\n");
            out.push_str("function onClick() bridge.call(\"click\") end\n");
            out.push_str("function onExit() bridge.call(\"exit\") end\n");
            out.push_str("bridge.watch_state(function() bridge.call(\"render\") end)\n");
            out.push_str("bridge.call(\"render\")\n");
        }
        EntryKind::LauncherProvider { .. } => {
            out.push_str("function onQuery(text) bridge.query(text) end\n");
            out.push_str("function onActivate(id) bridge.call(\"activate\", { id = id }) end\n");
            out.push_str("bridge.watch_state(function() bridge.rerender_query() end)\n");
        }
        EntryKind::Panel { .. } => {
            out.push_str("function onOpen() bridge.call(\"open\") end\n");
            out.push_str("function onClose() bridge.call(\"close\") end\n");
            out.push_str("function update() bridge.call(\"render\") end\n");
            out.push_str("bridge.watch_state(function() bridge.call(\"render\") end)\n");
            out.push_str("panel.setWantsSecondTicks(true)\n");
            out.push_str("bridge.call(\"render\")\n");
        }
    }

    out
}
