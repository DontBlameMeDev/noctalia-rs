use chrono::Utc;
use serde::Serialize;

use std::env;
use std::fs;
use std::path::Path;
use std::process::Command;

#[derive(Debug, Clone, Copy)]
pub struct SamplePlan {
    pub samples: usize,
    pub warmup: usize,
    pub build_samples: usize,
    pub build_warmup: usize,
}
#[derive(Debug, Serialize)]
pub struct Metadata {
    pub timestamp_utc: String,
    pub workspace: String,
    pub git_head: String,
    pub git_dirty: bool,
    pub rustc: String,
    pub cargo: String,
    pub os: String,
    pub cpu_model: String,
    pub logical_cpus: usize,
    pub cpu_max_freq_khz: String,
    pub memory_total_kib: String,
    pub profile: String,
    pub helper: String,
    pub samples: usize,
    pub warmup: usize,
    pub build_samples: usize,
    pub build_warmup: usize,
    pub percentile_method: &'static str,
    pub timer: &'static str,
}

impl Metadata {
    pub fn collect(workspace: &Path, helper: &Path, plan: SamplePlan) -> Self {
        Self {
            timestamp_utc: Utc::now().to_rfc3339(),
            workspace: workspace.display().to_string(),
            git_head: capture_in(
                workspace,
                "git",
                &["rev-parse", "--short", "HEAD"],
            ),
            git_dirty: !capture_in(
                workspace,
                "git",
                &["status", "--porcelain"],
            )
            .is_empty(),
            rustc: capture(
                env::var("RUSTC").as_deref().unwrap_or("rustc"),
                &["-V"],
            ),
            cargo: capture(
                env::var("CARGO").as_deref().unwrap_or("cargo"),
                &["-V"],
            ),
            os: capture("uname", &["-srm"]),
            cpu_model: cpu_model(),
            logical_cpus: std::thread::available_parallelism()
                .map_or(0, |value| value.get()),
            cpu_max_freq_khz: read_first_line(Path::new(
                "/sys/devices/system/cpu/cpu0/cpufreq/cpuinfo_max_freq",
            )),
            memory_total_kib: read_key("/proc/meminfo", "MemTotal:")
                .split_whitespace()
                .next()
                .unwrap_or("")
                .to_owned(),
            profile: "release".to_owned(),
            helper: helper.display().to_string(),
            samples: plan.samples,
            warmup: plan.warmup,
            build_samples: plan.build_samples,
            build_warmup: plan.build_warmup,
            percentile_method: "nearest rank",
            timer: "std::time::Instant",
        }
    }
}

fn capture(program: &str, args: &[&str]) -> String {
    Command::new(program)
        .args(args)
        .output()
        .ok()
        .filter(|output| output.status.success())
        .map_or_else(
            || "unavailable".to_owned(),
            |output| trim(&String::from_utf8_lossy(&output.stdout)),
        )
}

fn capture_in(dir: &Path, program: &str, args: &[&str]) -> String {
    Command::new(program)
        .current_dir(dir)
        .args(args)
        .output()
        .ok()
        .filter(|output| output.status.success())
        .map_or_else(
            || "unavailable".to_owned(),
            |output| trim(&String::from_utf8_lossy(&output.stdout)),
        )
}

fn cpu_model() -> String {
    let name = read_key("/proc/cpuinfo", "model name").replace('\t', " ");
    if name.is_empty() {
        capture("uname", &["-m"])
    } else {
        name
    }
}

fn read_key(path: &str, key: &str) -> String {
    fs::read_to_string(path).map_or_else(
        |_| String::new(),
        |text| {
            text.lines()
                .find_map(|line| line.strip_prefix(key))
                .map_or_else(String::new, |value| {
                    trim(value.trim_start_matches(':'))
                })
        },
    )
}

fn read_first_line(path: &Path) -> String {
    fs::read_to_string(path).map_or_else(
        |_| String::new(),
        |text| text.lines().next().map_or_else(String::new, trim),
    )
}

fn trim(value: &str) -> String {
    value.trim().to_owned()
}
