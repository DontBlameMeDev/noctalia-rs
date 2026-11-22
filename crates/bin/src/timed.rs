mod duration;
mod launcher;
mod panel;
mod service;
mod timer;
mod widget;

use std::process::ExitCode;

use noctalia::run as run_cli;
use noctalia::{Context, Meta, Op, Plugin, Setting};
use serde_json::json;
use timer::{Store, Timer};

pub const PANEL_ID: &str = "dontblameme/timed:panel";
pub const STATE_KEY: &str = "timed.state";

const COLORS: &[(&str, &str)] = &[
    ("primary", "settings.color.primary"),
    ("secondary", "settings.color.secondary"),
    ("tertiary", "settings.color.tertiary"),
    ("error", "settings.color.error"),
    ("on_surface", "settings.color.on_surface"),
    ("on_surface_variant", "settings.color.on_surface_variant"),
];

const TRANSLATIONS: &[(&str, &str)] = &[
    ("notify.title", "Timed"),
    ("notify.body", "{duration} timer finished"),
    ("bar.empty", "No timers"),
    ("bar.timer", "{duration} timer"),
    ("bar.paused", "Paused - {time}"),
    ("panel.title", "Timed"),
    ("panel.empty", "No timers running"),
    ("panel.timer", "{duration} timer"),
    ("panel.ends_at", "Ends at {time}"),
    ("panel.paused_left", "Paused"),
    ("panel.pause", "Pause"),
    ("panel.resume", "Resume"),
    ("panel.stop", "Stop"),
    ("panel.stop_all", "Stop all"),
    ("panel.placeholder", "Duration, e.g. 1h5 or 5"),
    ("panel.start", "Start"),
    ("panel.invalid", "Invalid duration"),
    ("launcher.hint", "Try 1h5, 1y, 4s10h3m, or 5 for 5 minutes"),
    ("launcher.start", "Start timer {time}"),
    ("launcher.start_new", "Start a new timer"),
    ("launcher.timer", "{duration} timer"),
    ("launcher.message", "{duration} - {message}"),
    ("launcher.stop", "Stop"),
    ("launcher.pause", "Pause"),
    ("launcher.resume", "Resume"),
    ("launcher.no_matching", "No timers here"),
    ("launcher.ends_at", "Ends at {time} - {duration}"),
    ("launcher.paused_at", "Paused - {duration} left"),
    ("launcher.stop_all", "Stop all timers"),
    ("launcher.active_count", "{count} timers"),
    ("launcher.invalid", "Invalid duration"),
    ("settings.show_text.label", "Show time"),
    ("settings.show_text.description", "Show the remaining time next to the ring."),
    ("settings.show_count.label", "Show count"),
    ("settings.show_count.description", "Show the number of timers inside the ring."),
    ("settings.show_progress.label", "Show progress ring"),
    ("settings.show_progress.description", "Draw a progress ring for the nearest timer."),
    ("settings.use_urgency_colors.label", "Timer colors"),
    ("settings.use_urgency_colors.description", "Change color as the timer runs out."),
    ("settings.safe_color.label", "Safe color"),
    ("settings.safe_color.description", "Ring color while more than half remains."),
    ("settings.warning_color.label", "Warning color"),
    ("settings.warning_color.description", "Ring color when half or less remains."),
    ("settings.urgent_color.label", "Urgent color"),
    ("settings.urgent_color.description", "Ring color when a fifth or less remains."),
    ("settings.idle_color.label", "Idle color"),
    ("settings.idle_color.description", "Ring color when no timers are running."),
    ("settings.hide_when_idle.label", "Hide when idle"),
    ("settings.hide_when_idle.description", "Hide the widget when no timers are running."),
    ("settings.color.primary", "Primary"),
    ("settings.color.secondary", "Secondary"),
    ("settings.color.tertiary", "Tertiary"),
    ("settings.color.error", "Error"),
    ("settings.color.on_surface", "Text"),
    ("settings.color.on_surface_variant", "Muted text"),
];

pub fn publish(store: &Store) -> Op {
    Op::state(STATE_KEY, json!({ "timers": store.timers }))
}

pub fn timers_from_state(ctx: &Context) -> Vec<Timer> {
    ctx.state_value("timers").unwrap_or_default()
}

pub fn add_timer(ctx: &Context, spec: (&mut Store, u64, Option<String>)) -> bool {
    let (store, seconds, message) = spec;

    if seconds == 0 {
        return false;
    }

    store.seq = store.seq.saturating_add(1);
    let offset = i64::try_from(seconds)
        .unwrap_or(i64::MAX)
        .saturating_mul(1_000);

    store.timers.push(Timer {
        id: format!("{}-{}", ctx.now_ms, store.seq),
        seconds,
        label: duration::human(seconds),
        message,
        paused: false,
        ends_at: Some(ctx.now_ms.saturating_add(offset)),
        remaining_ms: None,
    });
    timer::sort(store);

    true
}

pub fn stop_by_id(store: &mut Store, id: &str) -> bool {
    let before = store.timers.len();
    store.timers.retain(|timer| timer.id != id);
    store.timers.len() != before
}

pub fn pause_by_id(ctx: &Context, store: &mut Store, id: &str) -> bool {
    let Some(timer) = store
        .timers
        .iter_mut()
        .find(|timer| timer.id == id && !timer.paused)
    else {
        return false;
    };

    let remaining = timer
        .ends_at
        .map_or(0, |end| end.saturating_sub(ctx.now_ms));
    if remaining <= 0 {
        return false;
    }

    timer.paused = true;
    timer.remaining_ms = Some(u64::try_from(remaining).unwrap_or(0));
    timer.ends_at = None;
    timer::sort(store);

    true
}

pub fn resume_by_id(ctx: &Context, store: &mut Store, id: &str) -> bool {
    let Some(timer) = store
        .timers
        .iter_mut()
        .find(|timer| timer.id == id && timer.paused)
    else {
        return false;
    };

    let remaining = timer.remaining_ms.unwrap_or(0);
    timer.paused = false;
    timer.ends_at = Some(
        ctx.now_ms
            .saturating_add(i64::try_from(remaining).unwrap_or(0)),
    );
    timer.remaining_ms = None;
    timer::sort(store);

    true
}

fn settings() -> Vec<Setting> {
    let colors: Vec<(&str, &str)> = COLORS.to_vec();

    vec![
        Setting::bool("show_text", "settings.show_text.label", true)
            .description("settings.show_text.description"),
        Setting::bool("show_count", "settings.show_count.label", true)
            .description("settings.show_count.description"),
        Setting::bool("show_progress", "settings.show_progress.label", true)
            .description("settings.show_progress.description"),
        Setting::bool("hide_when_idle", "settings.hide_when_idle.label", false)
            .description("settings.hide_when_idle.description"),
        Setting::bool("use_urgency_colors", "settings.use_urgency_colors.label", true)
            .description("settings.use_urgency_colors.description"),
        Setting::select(
            "safe_color",
            "settings.safe_color.label",
            ("tertiary", &colors),
        )
        .description("settings.safe_color.description"),
        Setting::select(
            "warning_color",
            "settings.warning_color.label",
            ("secondary", &colors),
        )
        .description("settings.warning_color.description")
        .visible_when("use_urgency_colors", &["true"]),
        Setting::select(
            "urgent_color",
            "settings.urgent_color.label",
            ("error", &colors),
        )
        .description("settings.urgent_color.description")
        .visible_when("use_urgency_colors", &["true"]),
        Setting::select(
            "idle_color",
            "settings.idle_color.label",
            ("primary", &colors),
        )
        .description("settings.idle_color.description"),
    ]
}

fn build_plugin() -> Plugin {
    let mut plugin = Plugin::new(
        Meta::new("dontblameme/timed", "Timed", "0.1.0")
            .author("DontBlameMe")
            .plugin_api(48)
            .description("Countdown timers written in Rust")
            .tags(&["launcher", "bar", "panel", "service", "countdown", "utility"])
            .icon("clock"),
    )
    .service("service", service::update)
    .widget("timed_indicator", (settings(), widget::update))
    .launcher("launcher", ("timer", "clock", launcher::update))
    .panel("panel", (380, 360, panel::update));

    for (key, value) in TRANSLATIONS {
        plugin = plugin.tr(key, value);
    }

    plugin
}

pub fn run() -> ExitCode {
    run_cli(&build_plugin())
}

