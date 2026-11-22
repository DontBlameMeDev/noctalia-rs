use std::fs;
use std::path::PathBuf;

use noctalia::{Context, Op, Result};
use serde_json::{json, Value};

use super::duration;
use super::timers_from_state;
use super::timer::Timer;
use super::PANEL_ID;

const CIRCUMFERENCE_X100: u64 = 10_053;
type RoleSpec<'a> = (&'a str, &'a str, &'a str, &'a str, bool);
type RingSpec<'a> = (&'a Timer, usize, &'a str, bool);

#[inline]
fn quotient(value: u64, divisor: u64) -> u64 {
    value.checked_div(divisor).unwrap_or(0)
}

#[inline]
fn remainder(value: u64, divisor: u64) -> u64 {
    value.checked_rem(divisor).unwrap_or(0)
}

#[inline]
fn color_hex(role: &str) -> &'static str {
    match role {
        "primary" => "#CBA6F7",
        "secondary" => "#F9E2AF",
        "error" => "#F38BA8",
        "on_surface" => "#CDD6F4",
        "outline_variant" => "#4C4F69",
        _ => "#A6ADC8",
    }
}

pub fn update(ctx: &Context) -> Result<Vec<Op>> {
    match ctx.event.as_str() {
        "click" => Ok(vec![Op::TogglePanel {
            id: PANEL_ID.to_owned(),
        }]),
        "exit" => {
            let _ = fs::remove_file(ring_path(ctx));
            let _ = fs::remove_file(temporary_path(ctx));
            Ok(Vec::new())
        }
        _ => render(ctx),
    }
}

fn render(ctx: &Context) -> Result<Vec<Op>> {
    let timers = timers_from_state(ctx);

    let show_text = ctx.config_bool("show_text", true);
    let show_count = ctx.config_bool("show_count", true);
    let show_progress = ctx.config_bool("show_progress", true);
    let hide_when_idle = ctx.config_bool("hide_when_idle", false);
    let use_urgency = ctx.config_bool("use_urgency_colors", true);
    let safe = ctx.config_str("safe_color").unwrap_or_else(|| "tertiary".to_owned());
    let warning = ctx.config_str("warning_color").unwrap_or_else(|| "secondary".to_owned());
    let urgent = ctx.config_str("urgent_color").unwrap_or_else(|| "error".to_owned());
    let idle = ctx.config_str("idle_color").unwrap_or_else(|| "primary".to_owned());

    if hide_when_idle && timers.is_empty() {
        return Ok(vec![
            Op::BarVisible { visible: false },
            Op::SetInterval { ms: 86_400_000 },
        ]);
    }

    let nearest = timers.first();
    let now = ctx.now_ms;
    let count = timers.len();
    let state = nearest.map_or("idle", |timer| timer.state(now));
    let role = role_for(state, (&safe, &warning, &urgent, &idle, use_urgency));

    let mut ops = vec![Op::BarVisible { visible: true }];

    if show_progress && let Some(timer) = nearest {
        write_ring(ctx, (timer, count, role, show_count))?;
        ops.push(Op::BarImage {
            path: ring_path(ctx).to_string_lossy().into_owned(),
            preserve: true,
            width: 26,
            height: 26,
        });
    }

    let text = if show_text {
        nearest
            .map(|timer| duration::approx(timer.remaining_ms(now).div_ceil(1_000)))
            .unwrap_or_default()
    } else {
        String::new()
    };
    ops.push(Op::BarText { text });
    ops.push(Op::BarColor {
        role: "on_surface".to_owned(),
    });
    ops.push(Op::BarTooltip {
        value: json!(tooltip_rows(ctx, &timers, now)),
    });
    ops.push(Op::SetInterval {
        ms: if timers.iter().any(|timer| !timer.paused) {
            1_000
        } else {
            60_000
        },
    });

    Ok(ops)
}

fn role_for<'a>(state: &str, spec: RoleSpec<'a>) -> &'a str {
    let (safe, warning, urgent, idle, use_urgency) = spec;

    match state {
        "idle" => idle,
        "paused" => "on_surface_variant",
        "urgent" if use_urgency => urgent,
        "warning" if use_urgency => warning,
        _ => safe,
    }
}

fn tooltip_rows(ctx: &Context, timers: &[Timer], now: i64) -> Vec<Value> {
    let mut rows = Vec::new();

    for timer in timers {
        let left = duration::approx(timer.remaining_ms(now).div_ceil(1_000));
        let title = timer.message.clone().unwrap_or_else(|| {
            ctx.tr("bar.timer", &[("duration", timer.label.clone())])
        });
        let value = if timer.paused {
            ctx.tr("bar.paused", &[("time", left)])
        } else {
            left
        };
        rows.push(json!({ "key": title, "value": value }));
    }

    if rows.is_empty() {
        rows.push(json!({ "key": ctx.tr("bar.empty", &[]), "value": "" }));
    }

    rows
}

fn write_ring(ctx: &Context, spec: RingSpec<'_>) -> Result<()> {
    let (timer, count, role, show_count) = spec;
    let percent = timer.percent_remaining(ctx.now_ms).min(100);
    let offset_x100 = quotient(
        CIRCUMFERENCE_X100.saturating_mul(100_u64.saturating_sub(percent)),
        100,
    );
    let arc = color_hex(role);
    let count_text = if show_count {
        format!(
            "<text x=\"20\" y=\"26\" text-anchor=\"middle\" font-family=\"sans-serif\" font-size=\"19\" font-weight=\"bold\" fill=\"{arc}\">{count}</text>"
        )
    } else {
        String::new()
    };

    let svg = format!(
        "<svg xmlns=\"http://www.w3.org/2000/svg\" width=\"40\" height=\"40\" viewBox=\"0 0 40 40\">\
         <circle cx=\"20\" cy=\"20\" r=\"16\" fill=\"none\" stroke=\"{}\" stroke-width=\"4\"/>\
         <circle cx=\"20\" cy=\"20\" r=\"16\" fill=\"none\" stroke=\"{}\" stroke-width=\"4\" stroke-dasharray=\"100.53\" stroke-dashoffset=\"{}.{:02}\" stroke-linecap=\"round\" transform=\"rotate(-90 20 20)\"/>\
         {}</svg>",
        color_hex("outline_variant"),
        arc,
        quotient(offset_x100, 100),
        remainder(offset_x100, 100),
        count_text
    );

    fs::write(temporary_path(ctx), svg)?;
    fs::rename(temporary_path(ctx), ring_path(ctx))?;

    Ok(())
}

fn ring_path(ctx: &Context) -> PathBuf {
    ctx.data_dir.join("ring.svg")
}

fn temporary_path(ctx: &Context) -> PathBuf {
    ctx.data_dir.join("ring.svg.tmp")
}
