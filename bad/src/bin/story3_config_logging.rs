//! Story 3: Application Configuration & Logging (without hat)
//!
//! A task scheduler service that reads config from env vars,
//! manages user settings in JSON, and logs to files.
//!
//! Problems a hat user would never have:
//! 1. Env vars scattered with inconsistent error handling (unwrap/expect/unwrap_or)
//! 2. JSON settings: manual serde, non-atomic write, crash = corrupt file
//! 3. Default templates: runtime parse of include_str!, fails at runtime not compile time
//! 4. Logging: no rotation, no retention, no crash hook, no structure

use std::env;
use std::fs;
use std::io::Write;
use std::path::PathBuf;

use serde::{Deserialize, Serialize};

// ── 1. Env var reading: scattered, inconsistent ─────────────────

fn load_env_config() -> (u16, String, String, String) {
    let port: u16 = env::var("SCHED_PORT")
        .unwrap_or_else(|_| "9090".to_string())
        .parse()
        .expect("SCHED_PORT must be a number");

    let api_key = env::var("SCHED_API_KEY")
        .expect("SCHED_API_KEY is required");

    let data_dir = env::var("SCHED_DATA_DIR")
        .unwrap_or_else(|_| "data".to_string());

    let log_level = env::var("SCHED_LOG_LEVEL")
        .unwrap_or_else(|_| "info".to_string());

    (port, api_key, data_dir, log_level)
}

// ── 2. JSON settings: manual, non-atomic ────────────────────────

#[derive(Serialize, Deserialize, Default)]
struct SchedulerSettings {
    max_retries: Option<u32>,
    timeout_secs: Option<u64>,
    notify_on_failure: Option<bool>,
    admin_email: Option<String>,
}

fn load_settings(path: &str) -> SchedulerSettings {
    match fs::read_to_string(path) {
        Ok(content) => serde_json::from_str(&content).unwrap_or_default(),
        Err(_) => SchedulerSettings::default(),
    }
}

fn save_settings(settings: &SchedulerSettings, path: &str) -> Result<(), String> {
    // NOT atomic — crash mid-write = corrupt file
    let json = serde_json::to_string_pretty(settings)
        .map_err(|e| format!("serialize failed: {}", e))?;
    fs::write(path, json)
        .map_err(|e| format!("write failed: {}", e))?;
    Ok(())
}

// ── 3. Built-in defaults: runtime parse ─────────────────────────

#[derive(Deserialize)]
struct TaskTemplate {
    name: String,
    description: String,
    cron: String,
}

fn load_default_templates() -> Vec<TaskTemplate> {
    // Fails at runtime, not compile time
    serde_json::from_str(include_str!("../../templates/defaults.json"))
        .expect("built-in templates must be valid JSON")
}

// ── 4. Primitive logging ────────────────────────────────────────

fn log_message(log_file: &mut fs::File, level: &str, msg: &str) {
    // No rotation, no retention, no structured format
    let _ = writeln!(log_file, "[{}] [{}] {}", "timestamp", level, msg);
}

// ── Main ────────────────────────────────────────────────────────

fn main() {
    let (port, _api_key, data_dir, _log_level) = load_env_config();

    // Settings path: manual string concatenation
    let settings_path = format!("{}/settings.json", data_dir);
    let settings = load_settings(&settings_path);

    let max_retries = settings.max_retries.unwrap_or(3);
    let timeout = settings.timeout_secs.unwrap_or(30);

    // Logging: manual file open
    let log_path = format!("{}/scheduler.log", data_dir);
    let mut log_file = fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(&log_path)
        .expect("cannot open log file");

    log_message(&mut log_file, "INFO",
        &format!("Scheduler starting on port {}", port));
    log_message(&mut log_file, "INFO",
        &format!("Max retries: {}, timeout: {}s", max_retries, timeout));

    // Load templates
    let templates = load_default_templates();
    log_message(&mut log_file, "INFO",
        &format!("Loaded {} task templates", templates.len()));

    // Save settings back (with defaults filled in)
    let updated = SchedulerSettings {
        max_retries: Some(max_retries),
        timeout_secs: Some(timeout),
        notify_on_failure: settings.notify_on_failure.or(Some(true)),
        admin_email: settings.admin_email,
    };
    if let Err(e) = save_settings(&updated, &settings_path) {
        eprintln!("Warning: failed to save settings: {}", e);
    }

    println!("Scheduler running on port {} with {} templates", port, templates.len());
}