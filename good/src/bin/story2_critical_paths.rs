//! Story 2: Critical Paths — Order Processing Pipeline (Good Version)
//!
//! Rewrites bad/ with Mad Hatter's Critical three-exit pattern:
//! propagate (fatal) / suppress (recoverable) / suppress_with (fallback).

#[path = "../concept_map.rs"]
#[allow(dead_code)]
mod concept_map;
use concept_map::{ProcessOrders, ProcessOrdersFatal, ProcessOrdersSuppressible};

use mad_hatter::IntoCritical;
use serde::Deserialize;

mad_hatter::file_store!(OrderLayout, {
    base: "."
    dirs {
        results: "orders/results",
        temp: "orders/temp",
    }
    files {
        pending: "orders/pending.json",
        summary: "orders/summary.txt",
        batch_done: "notifications/batch-done.txt",
    }
});

#[derive(Deserialize)]
struct Order {
    id: u64,
    customer: String,
    amount_cents: i64,
}

#[mad_hatter::main(ProcessOrders)]
async fn main(scope: &mad_hatter::TxScope<ProcessOrders>) {
    let fs = scope.fs();
    let layout = OrderLayout::new();

    // ── Step 1: Read order list (fatal — can't continue without data) ──
    let data = fs.read_to_string(layout.pending())
        .propagate(scope, ProcessOrdersFatal::ReadOrders);
    let orders: Vec<Order> = serde_json::from_str(&data)
        .into_critical("parse_pending_orders")
        .propagate(scope, ProcessOrdersFatal::ParseOrders);

    println!("Processing {} orders...", orders.len());

    // ── Step 2: Process each order (suppress — skip bad ones) ──────────
    let mut success_count: u32 = 0;
    let mut fail_count: u32 = 0;

    for order in &orders {
        // Validation: invalid amount → skip (integer 0 is exempt from E102)
        if order.amount_cents <= 0 {
            fail_count += 1;
            continue;
        }

        let result_json = format!(
            r#"{{"id":{},"customer":"{}","status":"completed","amount_cents":{}}}"#,
            order.id, order.customer, order.amount_cents
        );

        let result_path = layout.results().join(format!("{}.json", order.id));
        let written = fs.write(&result_path, &result_json)
            .suppress(ProcessOrdersSuppressible::ProcessSingle, "write order result");

        if written.is_some() {
            success_count += 1;
        } else {
            fail_count += 1;
        }
    }

    // ── Step 3: Write summary report (fatal — must succeed) ────────────
    let summary = format!(
        "Batch complete. Success: {}, Failed: {}",
        success_count, fail_count
    );
    fs.write(layout.summary(), &summary)
        .propagate(scope, ProcessOrdersFatal::WriteSummary);

    // ── Step 4: Send notification (suppress — best effort) ─────────────
    fs.write(layout.batch_done(), &summary)
        .suppress(ProcessOrdersSuppressible::Notify, "send notification");

    // ── Step 5: Cleanup temp directory (suppress — best effort) ────────
    fs.remove_dir_all(layout.temp())
        .suppress(ProcessOrdersSuppressible::Cleanup, "cleanup temp");

    println!("{}", summary);
}