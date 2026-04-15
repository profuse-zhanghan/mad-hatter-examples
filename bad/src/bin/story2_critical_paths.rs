//! Story 2: Critical Paths — Order Processing Pipeline
//!
//! A typical Rust developer processes a batch of orders from JSON files.
//! Four classic error-handling anti-patterns:
//!
//! 1. `anyhow::Result` + `?` — type erasure, caller can't distinguish
//!    "file not found" from "JSON parse error"
//! 2. `.unwrap()` — one bad write crashes the entire pipeline
//! 3. `eprintln!` — pretend-handling, no structured logging, no trace ID
//! 4. `let _ =` — silent swallow, disk full and nobody knows
//!
//! In the `good/` version you'll rewrite this with Mad Hatter's Critical
//! three-exit pattern: propagate / suppress / suppress_with.

use anyhow::Result;
use serde::Deserialize;
use std::fs;

#[derive(Deserialize)]
struct Order {
    id: u64,
    customer: String,
    amount: f64,
}

fn main() -> Result<()> {
    // ── Step 1: Read order list ─────────────────────────────────
    // anyhow `?` erases the error — caller can't tell whether the file
    // was missing or the JSON was malformed.
    let data = fs::read_to_string("orders/pending.json")?;
    let orders: Vec<Order> = serde_json::from_str(&data)?;

    println!("Processing {} orders...", orders.len());

    // ── Step 2: Process each order ──────────────────────────────
    let mut success_count: u32 = 0;
    let mut fail_count: u32 = 0;

    for order in &orders {
        // Validation — eprintln pretends to handle the error
        if order.amount <= 0.0 {
            eprintln!(
                "Warning: skipping order {} for {}: invalid amount {}",
                order.id, order.customer, order.amount
            );
            fail_count += 1;
            continue;
        }

        // Write result file — individual failure should NOT crash the
        // whole pipeline, but this code either unwraps or uses `?` which
        // does exactly that.
        let result_json = format!(
            r#"{{"id":{},"customer":"{}","status":"completed","amount":{}}}"#,
            order.id, order.customer, order.amount
        );

        match fs::write(
            format!("orders/results/{}.json", order.id),
            &result_json,
        ) {
            Ok(_) => success_count += 1,
            Err(e) => {
                // eprintln is not structured logging — no trace ID,
                // no level, grep-hostile in production
                eprintln!("Error: order {} failed: {}", order.id, e);
                fail_count += 1;
            }
        }
    }

    // ── Step 3: Write summary report ────────────────────────────
    // .unwrap() — if this write fails the whole process panics
    let summary = format!(
        "Batch complete. Success: {}, Failed: {}",
        success_count, fail_count
    );
    fs::write("orders/summary.txt", &summary).unwrap();

    // ── Step 4: Send notification ───────────────────────────────
    // `let _ =` silently swallows the error — if disk is full or
    // permissions are wrong, nobody will ever know
    let _ = fs::write("notifications/batch-done.txt", &summary);

    // ── Step 5: Cleanup temp directory ──────────────────────────
    // Another silent swallow
    let _ = fs::remove_dir_all("orders/temp");

    println!("{}", summary);
    Ok(())
}