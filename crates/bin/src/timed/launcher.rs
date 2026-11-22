use noctalia::{Context, Op, Result};
use serde_json::{Value, json};

use super::duration;
use super::timer;
use super::{
    pause_by_id, publish, resume_by_id, stop_by_id, timers_from_state,
};

type ActionRows<'a> = (&'a str, &'a [timer::Timer], i64);
type TimerRow<'a> = (&'a str, &'a timer::Timer, i64);

pub fn update(ctx: &Context) -> Result<Vec<Op>> {
    match ctx.event.as_str() {
        "query" => {
            let text = ctx.payload_str("text").unwrap_or_default();
            let rows = query_rows(ctx, &text);

            Ok(vec![Op::LauncherResults { text, rows }])
        }
        "activate" => activate(ctx),
        _ => Ok(Vec::new()),
    }
}

fn activate(ctx: &Context) -> Result<Vec<Op>> {
    let id = ctx.payload_str("id").unwrap_or_default();
    let mut store = timer::load(ctx);

    if id == "stop-all" {
        store.timers.clear();
    } else if let Some((action, rest)) = id.split_once(':') {
        match action {
            "pause" => {
                pause_by_id(ctx, &mut store, rest);
            }
            "resume" => {
                resume_by_id(ctx, &mut store, rest);
            }
            "stop" => {
                stop_by_id(&mut store, rest);
            }
            "start" => start(&mut store, ctx.now_ms, rest),
            _ => {}
        }
    }

    timer::save(ctx, &store)?;

    Ok(vec![publish(&store)])
}

fn start(store: &mut timer::Store, now: i64, rest: &str) {
    let Some((seconds_text, message)) = rest.split_once(':') else {
        return;
    };
    let Ok(seconds) = seconds_text.parse::<u64>() else {
        return;
    };
    if seconds == 0 {
        return;
    }

    store.seq = store.seq.saturating_add(1);
    let offset = i64::try_from(seconds)
        .unwrap_or(i64::MAX)
        .saturating_mul(1_000);
    let text = if message.is_empty() {
        None
    } else {
        Some(message.to_owned())
    };

    store.timers.push(timer::Timer {
        id: format!("{now}-{}", store.seq),
        seconds,
        label: duration::human(seconds),
        message: text,
        paused: false,
        ends_at: Some(now.saturating_add(offset)),
        remaining_ms: None,
    });
    timer::sort(store);
}

fn query_rows(ctx: &Context, text: &str) -> Vec<Value> {
    let query = text.trim();
    let timers = timers_from_state(ctx);
    let now = ctx.now_ms;

    match query {
        "" => vec![row((
            "new",
            ctx.tr("launcher.start_new", &[]),
            ctx.tr("launcher.hint", &[]),
            "plus",
        ))],
        "pause" | "resume" | "stop" => action_rows(ctx, (query, &timers, now)),
        "stop all" => stop_all_rows(ctx, &timers),
        _ => duration_row(ctx, query),
    }
}

fn action_rows(ctx: &Context, spec: ActionRows<'_>) -> Vec<Value> {
    let (action, timers, now) = spec;
    let want_paused = action == "resume";
    let mut rows = timers
        .iter()
        .filter(|timer| action == "stop" || timer.paused == want_paused)
        .map(|timer| timer_row(ctx, (action, timer, now)))
        .collect::<Vec<_>>();

    if rows.is_empty() {
        rows.push(empty_row(ctx));
    }

    rows
}

fn stop_all_rows(ctx: &Context, timers: &[timer::Timer]) -> Vec<Value> {
    if timers.is_empty() {
        return vec![empty_row(ctx)];
    }

    vec![row((
        "stop-all",
        ctx.tr("launcher.stop_all", &[]),
        ctx.tr(
            "launcher.active_count",
            &[("count", timers.len().to_string())],
        ),
        "player-stop",
    ))]
}

fn duration_row(ctx: &Context, query: &str) -> Vec<Value> {
    let Some((seconds, message)) = duration::split(query) else {
        return vec![row((
            "invalid",
            ctx.tr("launcher.invalid", &[]),
            ctx.tr("launcher.hint", &[]),
            "alert-triangle",
        ))];
    };

    let human = duration::human(seconds);
    let subtitle = message.as_ref().map_or_else(
        || human.clone(),
        |text| {
            ctx.tr(
                "launcher.message",
                &[("duration", human.clone()), ("message", text.clone())],
            )
        },
    );
    let id = message.map_or_else(
        || format!("start:{seconds}:"),
        |text| format!("start:{seconds}:{text}"),
    );

    vec![row((
        &id,
        ctx.tr("launcher.start", &[("time", human)]),
        subtitle,
        "clock",
    ))]
}

fn timer_row(ctx: &Context, spec: TimerRow<'_>) -> Value {
    let (action, timer, now) = spec;
    let title = timer.message.clone().unwrap_or_else(|| {
        ctx.tr("launcher.timer", &[("duration", timer.label.clone())])
    });
    let left = duration::approx(timer.remaining_ms(now).div_ceil(1_000));
    let subtitle = if timer.paused {
        ctx.tr("launcher.paused_at", &[("duration", left)])
    } else {
        ctx.tr("launcher.ends_at", &[("duration", left)])
    };

    row((&format!("{action}:{}", timer.id), title, subtitle, action))
}

fn empty_row(ctx: &Context) -> Value {
    row((
        "empty",
        ctx.tr("launcher.no_matching", &[]),
        ctx.tr("launcher.hint", &[]),
        "clock",
    ))
}

fn row(parts: (&str, String, String, &str)) -> Value {
    let (id, title, subtitle, glyph) = parts;
    json!({ "id": id, "title": title, "subtitle": subtitle, "glyph": glyph })
}
