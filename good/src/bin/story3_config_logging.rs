//! Story 3: Application Configuration & Logging (Good Version)
//!
//! Rewrites bad/ with Mad Hatter's declarative macros:
//! unified_params! (env + JSON config), json_store! (atomic settings),
//! log_service! (structured logging with rotation).

#[path = "../concept_map.rs"]
#[allow(dead_code)]
mod concept_map;
use concept_map::{BootstrapScheduler, BootstrapSchedulerFatal, BootstrapSchedulerSuppressible};

use mad_hatter::IntoCritical;
use serde::Deserialize;
use std::path::Path;

// ── 1. Configuration: env vars + JSON in one declaration ────────
mad_hatter::unified_params!(SchedulerParams, "SCHED", "{data_dir}/params.json" {
    #[required]
    api_key: String,

    #[readonly]
    port: u16 = 9090,

    #[readonly]
    data_dir: String = "data",

    log_level: String = "info",
});

// ── 2. Persistent settings: atomic writes, typed access ─────────
mad_hatter::json_store!(SchedulerSettings {
    max_retries: Option<u32> = 3,
    timeout_secs: Option<u64> = 30,
    notify_on_failure: Option<bool> = true,
    admin_email: Option<String>,
});

// ── 3. Logging: structured, with rotation + crash hook ──────────
mad_hatter::log_service! {
    SchedulerLog,
    tracing {
        time_format: "%Y-%m-%d %H:%M:%S",
        level: info,
    }
    crash {
        file: "crash.log",
    }
    rotate logs {
        file: "scheduler.log",
        max_files: 5,
    }
}

// ── 4. Built-in task templates ──────────────────────────────────
#[derive(Deserialize)]
struct TaskTemplate {
    #[allow(dead_code)]
    name: String,
    #[allow(dead_code)]
    description: String,
    #[allow(dead_code)]
    cron: String,
}

// ── 5. File layout for data directory ───────────────────────────
mad_hatter::file_store!(DataLayout, {
    base: "."
    dirs {
        data: "data",
    }
    files {
        settings: "data/settings.json",
    }
});

#[mad_hatter::main(BootstrapScheduler)]
async fn main(scope: &mad_hatter::TxScope<BootstrapScheduler>) {
    // Init params from env + JSON merge
    SchedulerParams::init();

    // Extract values before any await — RwLockReadGuard is not Send
    let (port, data_dir) = {
        let params = SchedulerParams::global();
        (params.port, params.data_dir.clone())
    };

    // Init logging infrastructure
    SchedulerLog::init(Path::new(&data_dir));

    let layout = DataLayout::new();

    // Load settings (falls back to defaults on missing/corrupt)
    let settings = SchedulerSettings::load(layout.settings());
    let retries = settings.max_retries_or_default();
    let timeout = settings.timeout_secs_or_default();

    tracing::info!("Scheduler starting on port {}", port);
    tracing::info!("Max retries: {}, timeout: {}s", retries, timeout);

    // Load built-in templates (compile-time embedded, runtime parsed)
    let templates: Vec<TaskTemplate> = serde_json::from_str(
        include_str!("../../templates/defaults.json")
    )
    .into_critical("parse_templates")
    .propagate(scope, BootstrapSchedulerFatal::ParseTemplates);

    tracing::info!("Loaded {} task templates", templates.len());

    // Save settings with defaults filled in (suppress — best effort)
    let updated = SchedulerSettings {
        max_retries: Some(retries),
        timeout_secs: Some(timeout),
        notify_on_failure: settings.notify_on_failure.or(Some(true)),
        admin_email: settings.admin_email,
    };
    updated.save(layout.settings())
        .into_critical("save_settings")
        .suppress(BootstrapSchedulerSuppressible::SaveSettings, "save settings");

    println!("Scheduler running on port {} with {} templates", port, templates.len());
}