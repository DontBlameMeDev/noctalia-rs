use noctalia::{ui, Context, Op, Result};
use serde_json::{json, Value};

use super::duration;
use super::timer;
use super::{add_timer, pause_by_id, publish, resume_by_id, stop_by_id, timers_from_state};

pub fn update(ctx: &Context) -> Result<Vec<Op>> {
    match ctx.event.as_str() {
        "action" => action(ctx),
        "close" => Ok(Vec::new()),
        _ => Ok(render(ctx)),
    }
}

fn action(ctx: &Context) -> Result<Vec<Op>> {
    let mut store = timer::load(ctx);
    let name = ctx.payload_str("name").unwrap_or_default();

    if let Some(id) = name.strip_prefix("toggle:") {
        let paused = store.timers.iter().find(|timer| timer.id == id).is_some_and(
            |timer| timer.paused,
        );
        if paused {
            resume_by_id(ctx, &mut store, id);
        } else {
            pause_by_id(ctx, &mut store, id);
        }
    } else if let Some(id) = name.strip_prefix("stop:") {
        stop_by_id(&mut store, id);
    } else {
        match name.as_str() {
            "stop_all" => store.timers.clear(),
            "draft" => set_draft(ctx, &mut store),
            "start" | "submit" => start(ctx, &mut store, &name),
            _ => {}
        }
    }

    timer::save(ctx, &store)?;

    Ok(vec![publish(&store)])
}

fn set_draft(ctx: &Context, store: &mut timer::Store) {
    if let Some(value) = arg_string(ctx, 0) {
        store.draft = value;
    }
}

fn start(ctx: &Context, store: &mut timer::Store, name: &str) {
    let raw = if name == "submit" {
        arg_string(ctx, 0)
    } else {
        Some(store.draft.clone())
    };

    let Some(text) = raw else {
        return;
    };
    let Some((seconds, message)) = duration::split(&text) else {
        return;
    };

    if add_timer(ctx, (store, seconds, message)) {
        store.draft.clear();
    }
}

fn arg_string(ctx: &Context, index: usize) -> Option<String> {
    match ctx.payload_arg(index)? {
        Value::String(value) => Some(value.clone()),
        Value::Number(value) => Some(value.to_string()),
        Value::Bool(_) | Value::Array(_) | Value::Object(_) | Value::Null => None,
    }
}

fn render(ctx: &Context) -> Vec<Op> {
    let store = timer::load(ctx);
    let timers = timers_from_state(ctx);
    let now = ctx.now_ms;

    let header = ui::row(
        json!({ "align": "center", "justify": "space_between", "gap": 8 }),
        header_children(ctx, timers.len()),
    );
    let body = body(ctx, &timers, now);
    let controls = ui::column(json!({ "gap": 6 }), vec![controls_row(ctx, &store.draft)]);

    let tree = ui::column(
        json!({ "gap": 12, "flexGrow": 1 }),
        vec![header, body, controls],
    );

    vec![Op::PanelRender { tree }]
}

fn header_children(ctx: &Context, count: usize) -> Vec<ui::UiNode> {
    let mut children = vec![ui::label(json!({
        "text": ctx.tr("panel.title", &[]),
        "fontSize": 18,
        "fontWeight": "bold"
    }))];

    if count > 1 {
        children.push(ui::button(json!({
            "text": ctx.tr("panel.stop_all", &[]),
            "variant": "ghost",
            "controlSize": "sm",
            "onClick": ui::call("stop_all")
        })));
    }

    children
}

fn body(ctx: &Context, timers: &[timer::Timer], now: i64) -> ui::UiNode {
    if timers.is_empty() {
        return ui::column(
            json!({ "flexGrow": 1, "justify": "center", "align": "center" }),
            vec![ui::label(json!({
                "text": ctx.tr("panel.empty", &[]),
                "color": "on_surface_variant"
            }))],
        );
    }

    let rows = timers
        .iter()
        .map(|timer| timer_row(ctx, timer, now))
        .collect();

    ui::scroll(json!({ "flexGrow": 1, "gap": 8 }), rows)
}

fn timer_row(ctx: &Context, timer: &timer::Timer, now: i64) -> ui::UiNode {
    let remaining = timer.remaining_ms(now);
    let seconds = remaining.div_ceil(1_000);
    let percent = timer.percent_remaining(now).min(100);
    let progress = f64::from(u32::try_from(percent).unwrap_or(0)) / 100.0;
    let role = match timer.state(now) {
        "urgent" => "error",
        "warning" => "secondary",
        _ => "tertiary",
    };
    let title = timer
        .message
        .clone()
        .unwrap_or_else(|| ctx.tr("panel.timer", &[("duration", timer.label.clone())]));
    let subtitle = if timer.paused {
        ctx.tr("panel.paused_left", &[])
    } else {
        ctx.tr("panel.ends_at", &[("time", duration::approx(seconds))])
    };
    let (glyph, tip, callback) = toggle_props(ctx, timer);

    ui::column(
        json!({ "gap": 8, "padding": 10, "fill": "surface_container/0.35", "radius": 10 }),
        vec![
            ui::row(
                json!({ "gap": 10, "align": "center" }),
                vec![
                    ui::column(
                        json!({ "gap": 2, "flexGrow": 1 }),
                        vec![
                            ui::label(json!({
                                "text": title,
                                "fontWeight": "semibold",
                                "maxLines": 1
                            })),
                            ui::label(json!({
                                "text": subtitle,
                                "fontSize": 12,
                                "color": "on_surface_variant",
                                "maxLines": 1
                            })),
                        ],
                    ),
                    ui::label(json!({
                        "text": duration::approx(seconds),
                        "fontSize": 18,
                        "fontWeight": "bold"
                    })),
                    ui::button(json!({
                        "glyph": glyph,
                        "variant": "ghost",
                        "controlSize": "sm",
                        "tooltip": tip,
                        "onClick": ui::call(&callback)
                    })),
                    ui::button(json!({
                        "glyph": "player-stop",
                        "variant": "ghost",
                        "controlSize": "sm",
                        "tooltip": ctx.tr("panel.stop", &[]),
                        "onClick": ui::call(&format!("stop:{}", timer.id))
                    })),
                ],
            ),
            ui::progress(json!({
                "height": 3,
                "radius": 2,
                "progress": progress,
                "fill": role,
                "track": "outline_variant"
            })),
        ],
    )
}

fn toggle_props(ctx: &Context, timer: &timer::Timer) -> (&'static str, String, String) {
    if timer.paused {
        (
            "player-play",
            ctx.tr("panel.resume", &[]),
            format!("toggle:{}", timer.id),
        )
    } else {
        (
            "player-pause",
            ctx.tr("panel.pause", &[]),
            format!("toggle:{}", timer.id),
        )
    }
}

fn controls_row(ctx: &Context, draft: &str) -> ui::UiNode {
    ui::row(
        json!({ "gap": 8, "align": "center" }),
        vec![
            ui::input(json!({
                "value": draft,
                "placeholder": ctx.tr("panel.placeholder", &[]),
                "flexGrow": 1,
                "focus": true,
                "onChange": ui::call("draft"),
                "onSubmit": ui::call("submit")
            })),
            ui::button(json!({
                "text": ctx.tr("panel.start", &[]),
                "variant": "primary",
                "onClick": ui::call("start")
            })),
        ],
    )
}
