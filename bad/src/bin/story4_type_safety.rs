//! Story 4: Type Safety & Error Handling (without hat)
//!
//! A data export tool that reads user records, validates format,
//! and generates a summary report.
//!
//! Problems a hat user would never have:
//! 1. Manual Display/FromStr impl — verbose, error-prone, drifts from format
//! 2. Manual error types — boilerplate impl Display + Error, no structured erasing
//! 3. Bare `as` casts — silent integer truncation
//! 4. Tuple returns — unnamed fields, caller guesses which is which
//! 5. Raw error leaking — full internal errors exposed to callers

use std::fmt;
use std::str::FromStr;

// ── 1. Manual Display/FromStr — verbose and fragile ─────────────

/// User ID format: "USR-{region}_{sequence}"
#[derive(Debug, Clone)]
struct UserId {
    region: String,
    sequence: u32,
}

impl fmt::Display for UserId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "USR-{}_{}", self.region, self.sequence)
    }
}

impl FromStr for UserId {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let s = s.strip_prefix("USR-").ok_or("missing USR- prefix")?;
        let (region, seq_str) = s.split_once('_')
            .ok_or("missing _ separator")?;
        let sequence: u32 = seq_str.parse()
            .map_err(|_| "invalid sequence number")?;
        Ok(UserId {
            region: region.to_string(),
            sequence,
        })
    }
}

// ── 2. Manual error type — boilerplate ──────────────────────────

#[derive(Debug)]
enum ExportError {
    InvalidRecord(String),
    FormatError(String),
    IoError(String),
}

impl fmt::Display for ExportError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ExportError::InvalidRecord(msg) => write!(f, "Invalid record: {}", msg),
            ExportError::FormatError(msg) => write!(f, "Format error: {}", msg),
            // BUG: leaks internal details to caller
            ExportError::IoError(msg) => write!(f, "IO error: {}", msg),
        }
    }
}

impl std::error::Error for ExportError {}

// ── 3. Bare `as` cast — silent truncation ───────────────────────

fn calculate_stats(ages: &[u32]) -> (f64, u8, u8) {
    let sum: u32 = ages.iter().sum();
    let avg = sum as f64 / ages.len() as f64;  // E501: bare `as`

    let max = *ages.iter().max().unwrap_or(&0);
    let min = *ages.iter().min().unwrap_or(&0);

    // Silent truncation! If max age > 255, wraps around
    (avg, max as u8, min as u8)  // E501 + E506: tuple return
}

// ── 4. Tuple return — unnamed, fragile ──────────────────────────

fn validate_record(raw: &str) -> (bool, String, Vec<String>) {
    let mut errors = Vec::new();
    let mut display_name = String::new();

    let parts: Vec<&str> = raw.split('|').collect();
    if parts.len() < 3 {
        return (false, String::new(), vec!["too few fields".into()]);
    }

    // Validate user ID
    match UserId::from_str(parts[0]) {
        Ok(id) => display_name = id.to_string(),
        Err(e) => errors.push(format!("bad user id: {}", e)),
    }

    // Validate age
    match parts[2].parse::<u32>() {
        Ok(age) if age > 150 => errors.push("age too large".into()),
        Err(_) => errors.push("invalid age".into()),
        _ => {}
    }

    (errors.is_empty(), display_name, errors)
}

// ── 5. Raw error leaking ────────────────────────────────────────

fn export_records(records: &[&str]) -> Result<String, ExportError> {
    let mut output = String::new();
    let mut ages = Vec::new();

    for record in records {
        let (valid, name, errs) = validate_record(record);
        if !valid {
            // Leaks all internal validation details
            return Err(ExportError::InvalidRecord(
                format!("{}: {}", record, errs.join(", "))
            ));
        }

        let parts: Vec<&str> = record.split('|').collect();
        let age: u32 = parts[2].parse().unwrap();
        ages.push(age);

        output.push_str(&format!("  {} (age {})\n", name, age));
    }

    let (avg, max, min) = calculate_stats(&ages);
    output.push_str(&format!("\nStats: avg={:.1}, max={}, min={}\n", avg, max, min));

    Ok(output)
}

// ── Main ────────────────────────────────────────────────────────

fn main() {
    let records = vec![
        "USR-US_1001|Alice|30",
        "USR-EU_2002|Bob|25",
        "USR-AP_3003|Charlie|35",
    ];

    match export_records(&records) {
        Ok(report) => {
            println!("=== Export Report ===");
            println!("{}", report);
        }
        Err(e) => {
            // Full internal error exposed to user
            eprintln!("Export failed: {}", e);
        }
    }
}