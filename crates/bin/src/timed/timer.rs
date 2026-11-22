use std::cmp::Ordering;
use std::fs;
use std::path::PathBuf;

use noctalia::{Context, Result};
use serde::{Deserialize, Serialize};

const MAX_INTERVAL_MS: i64 = 86_400_000;

#[inline]
fn quotient(value: u64, divisor: u64) -> u64 {
    value.checked_div(divisor).unwrap_or(0)
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Timer {
    pub id: String,
    pub seconds: u64,
    pub label: String,
    pub message: Option<String>,
    pub paused: bool,
    #[serde(rename = "endsAt", skip_serializing_if = "Option::is_none")]
    pub ends_at: Option<i64>,
    #[serde(rename = "remainingMs", skip_serializing_if = "Option::is_none")]
    pub remaining_ms: Option<u64>,
}

impl Timer {
    #[must_use]
    pub fn remaining_ms(&self, now: i64) -> u64 {
        if self.paused {
            return self.remaining_ms.unwrap_or(0);
        }

        self.ends_at
            .map_or(0, |end| u64::try_from(end.saturating_sub(now)).unwrap_or(0))
    }

    #[must_use]
    pub fn percent_remaining(&self, now: i64) -> u64 {
        if self.seconds == 0 {
            return 0;
        }

        quotient(
            self.remaining_ms(now).saturating_mul(100),
            self.seconds.saturating_mul(1_000),
        )
    }

    #[must_use]
    pub fn state(&self, now: i64) -> &'static str {
        if self.paused {
            return "paused";
        }

        match self.percent_remaining(now) {
            0..=20 => "urgent",
            21..=50 => "warning",
            _ => "safe",
        }
    }
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct Store {
    #[serde(default)]
    pub timers: Vec<Timer>,
    #[serde(default)]
    pub seq: u64,
    #[serde(default)]
    pub draft: String,
}

#[must_use]
pub fn load(ctx: &Context) -> Store {
    let Ok(raw) = fs::read_to_string(store_path(ctx)) else {
        return Store::default();
    };

    serde_json::from_str(&raw).unwrap_or_default()
}

pub fn save(ctx: &Context, store: &Store) -> Result<()> {
    let raw = serde_json::to_string(store)?;
    let temporary = ctx
        .data_dir
        .join(format!("timers.{}.tmp", ctx.now_ms.max(0)));

    fs::write(&temporary, raw)?;
    fs::rename(temporary, store_path(ctx))?;

    Ok(())
}

pub fn sort(store: &mut Store) {
    store.timers.sort_by(|left, right| match (left.paused, right.paused) {
        (false, true) => Ordering::Less,
        (true, false) => Ordering::Greater,
        (true, true) => left
            .remaining_ms
            .unwrap_or(0)
            .cmp(&right.remaining_ms.unwrap_or(0)),
        (false, false) => left.ends_at.unwrap_or(0).cmp(&right.ends_at.unwrap_or(0)),
    });
}

#[must_use]
pub fn next_interval_ms(store: &Store, now: i64) -> u64 {
    let end = store
        .timers
        .iter()
        .find(|timer| !timer.paused)
        .and_then(|timer| timer.ends_at);

    let Some(ends_at) = end else {
        return u64::try_from(MAX_INTERVAL_MS).unwrap_or(86_400_000);
    };

    let delay = ends_at.saturating_sub(now).saturating_add(50).clamp(100, MAX_INTERVAL_MS);
    u64::try_from(delay).unwrap_or(86_400_000)
}

fn store_path(ctx: &Context) -> PathBuf {
    ctx.data_dir.join("timers.json")
}

#[cfg(test)]
mod tests {
    use super::*;

    fn running_timer(seconds: u64) -> Timer {
        Timer {
            id: "one".to_owned(),
            seconds,
            label: "1m".to_owned(),
            message: None,
            paused: false,
            ends_at: Some(60_000),
            remaining_ms: None,
        }
    }

    #[test]
    fn test_remaining_ms_uses_end_time_for_running_timer() {
        let timer = running_timer(60);

        assert_eq!(timer.remaining_ms(10_000), 50_000);
        assert_eq!(timer.remaining_ms(70_000), 0);
    }

    #[test]
    fn test_state_uses_remaining_percentage() {
        let timer = running_timer(100);

        assert_eq!(timer.state(0), "safe");
        assert_eq!(timer.state(35_000), "warning");
        assert_eq!(timer.state(90_000), "urgent");
    }

    #[test]
    fn test_next_interval_ms_ignores_paused_timers() {
        let mut paused = running_timer(60);
        paused.paused = true;
        paused.ends_at = None;
        paused.remaining_ms = Some(60_000);
        let store = Store {
            timers: vec![paused],
            ..Store::default()
        };

        assert_eq!(next_interval_ms(&store, 0), 86_400_000);
    }

    #[test]
    fn test_next_interval_ms_follows_soonest_timer() {
        let store = Store {
            timers: vec![running_timer(60)],
            ..Store::default()
        };

        assert_eq!(next_interval_ms(&store, 0), 60_050);
    }
}
