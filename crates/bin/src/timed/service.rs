use noctalia::{Context, Op, Result};

use super::duration;
use super::timer;
use super::{add_timer, pause_by_id, publish, resume_by_id, stop_by_id};

pub fn update(ctx: &Context) -> Result<Vec<Op>> {
    let mut store = timer::load(ctx);
    let mut ops = Vec::new();

    match ctx.event.as_str() {
        "init" | "update" | "resume" => expire(ctx, &mut store, &mut ops),
        "ipc" => ipc(ctx, &mut store, &mut ops),
        _ => {}
    }

    timer::save(ctx, &store)?;
    ops.push(Op::SetInterval {
        ms: timer::next_interval_ms(&store, ctx.now_ms),
    });
    ops.push(publish(&store));

    Ok(ops)
}

fn expire(ctx: &Context, store: &mut timer::Store, ops: &mut Vec<Op>) {
    let before = store.timers.len();

    store.timers.retain(|timer| {
        if timer.paused || timer.ends_at.is_some_and(|end| end > ctx.now_ms) {
            return true;
        }

        let body = timer.message.clone().unwrap_or_else(|| {
            ctx.tr("notify.body", &[("duration", timer.label.clone())])
        });
        ops.push(Op::Notify {
            title: ctx.tr("notify.title", &[]),
            body,
        });

        false
    });

    let finished = before.saturating_sub(store.timers.len());
    if finished > 0 {
        ops.push(Op::log(format!("timed: {finished} finished")));
    }
}

fn ipc(ctx: &Context, store: &mut timer::Store, ops: &mut Vec<Op>) {
    let event = ctx.payload_str("event").unwrap_or_default();
    let payload = ctx.payload_str("payload").unwrap_or_default();

    match event.as_str() {
        "start" => {
            if let Some((seconds, message)) = duration::split(&payload) {
                add_timer(ctx, (store, seconds, message));
            }
        }
        "stop" => {
            stop_by_id(store, &payload);
        }
        "pause" => {
            pause_by_id(ctx, store, &payload);
        }
        "resume" => {
            resume_by_id(ctx, store, &payload);
        }
        "stop-all" => store.timers.clear(),
        "list" => list(ctx, store, ops),
        _ => {}
    }
}

fn list(ctx: &Context, store: &timer::Store, ops: &mut Vec<Op>) {
    for timer in &store.timers {
        let seconds = timer.remaining_ms(ctx.now_ms).div_ceil(1_000);
        let state = if timer.paused { "paused" } else { "running" };
        let message = format!(
            "timed: {} {} {} left",
            timer.id,
            state,
            duration::clock(seconds)
        );
        ops.push(Op::log(message));
    }
}
