//! Story 4: Type Safety & Error Handling (with hat)
//!
//! Improvements over bad/:
//! 1. FormatStruct — replaces manual Display/FromStr (UserId, UserRecord)
//! 2. MadError — replaces manual error boilerplate
//! 3. TryFrom/From — replaces bare `as` casts (E501)
//! 4. Named structs — replaces tuple returns (E506)
//! 5. Sealed<String> — prevents raw error leaking

use mad_hatter::{FormatStruct, MadError, Sealed};

// ── FormatStruct — compile-checked Display/FromStr/Serde ────

/// User ID: region + sequence. Format: "{region}_{sequence}"
/// Display: UserId { region: "US", sequence: 1001 } → "US_1001"
/// FromStr: "US_1001" → UserId { region: "US", sequence: 1001 }
#[derive(Debug, Clone, Default, FormatStruct)]
#[format("{region}_{sequence}")]
pub struct UserId {
    pub region: String,
    pub sequence: u32,
}

/// Full user record with prefix. Format: "{prefix}-{user_id}|{name}|{age}"
/// Nested FormatStruct: user_id field uses UserId's FromStr.
/// All separators ("-", "|", "_") live in format attributes — exempt from E101/E103.
#[derive(Debug, Clone, FormatStruct)]
#[format("{prefix}-{user_id}|{name}|{age}")]
pub struct UserRecord {
    pub prefix: String,
    pub user_id: UserId,
    pub name: String,
    pub age: u32,
}

// ── MadError — type-safe error handling ─────────────────────

#[derive(Debug, MadError)]
pub enum ExportError {
    #[humanize("record validation failed")]
    InvalidRecord { detail: Sealed<String> },
    #[humanize("export format error")]
    FormatError { detail: Sealed<String> },
}

// ── json_store! for validation config ───────────────────────
// Default 150 lives inside macro shell — exempt from E102.

mad_hatter::json_store!(ValidationConfig {
    max_valid_age: Option<u32> = 150,
});

// ── Named struct for stats — replaces (f64, u8, u8) tuple (E506) ──

pub struct AgeStats {
    pub average: f64,
    pub max: u32,
    pub min: u32,
}

// ── Functions ───────────────────────────────────────────────

/// Calculate age statistics with safe conversions.
/// f64::from() for widening, TryFrom for narrowing — no bare `as` (E501).
fn calculate_stats(ages: &[u32]) -> AgeStats {
    let sum: u32 = ages.iter().sum();
    let count = u32::try_from(ages.len()).unwrap_or(u32::MAX);
    let avg = f64::from(sum) / f64::from(count);

    let max = *ages.iter().max().unwrap_or(&0);
    let min = *ages.iter().min().unwrap_or(&0);

    AgeStats { average: avg, max, min }
}

/// Export records: parse with FormatStruct, validate, generate report.
/// Replaces manual split + field indices + bare `as` + tuple returns.
fn export_records(records: &[&str]) -> Result<String, ExportError> {
    let config = ValidationConfig { max_valid_age: None };
    let mut output = String::new();
    let mut ages = Vec::new();

    for &raw in records {
        // FormatStruct FromStr — replaces manual split('|') + parse
        let record: UserRecord = raw.parse()
            .map_err(|e| {
                let msg = format!("{}", e);
                ExportError::FormatError {
                    detail: Sealed::new(msg),
                }
            })?;

        // Validate age using json_store! default (150)
        let max_age = config.max_valid_age_or_default();
        if record.age > max_age {
            return Err(ExportError::InvalidRecord {
                detail: Sealed::new(format!("age {} exceeds maximum {}", record.age, max_age)),
            });
        }

        ages.push(record.age);
        // FormatStruct Display — no manual format!("USR-{}_{}", ...)
        output.push_str(&format!("  {} (age {})\n", record.user_id, record.age));
    }

    if !ages.is_empty() {
        let stats = calculate_stats(&ages);
        output.push_str(&format!(
            "\nStats: avg={:.1}, max={}, min={}\n",
            stats.average, stats.max, stats.min
        ));
    }

    Ok(output)
}

// ── Main ────────────────────────────────────────────────────

fn main() {
    // Test data from file — no hardcoded string literals in source
    let test_data = include_str!("../../test_data/story4_records.txt");
    let records: Vec<&str> = test_data.lines().filter(|l| !l.is_empty()).collect();

    match export_records(&records) {
        Ok(report) => {
            println!("=== Export Report ===");
            println!("{}", report);
        }
        Err(e) => {
            eprintln!("Export failed: {}", e);
        }
    }
}