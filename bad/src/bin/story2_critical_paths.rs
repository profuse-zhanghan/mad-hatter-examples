// Story 2: Critical Paths — Error Handling in a Data Import Tool
//
// Scenario: A CLI tool that imports user data from CSV files into a database.
// Steps: read config → scan directory → process each CSV → write summary → cleanup
//
// Problems a typical Rust developer creates:
// 1. anyhow::Result erases all error types — caller can't distinguish failures
// 2. .unwrap() on file reads — one bad file crashes the entire import
// 3. `let _ = ...` silently swallows cleanup errors — no logs, no trace
// 4. `?` propagates everything identically — config failure and data failure
//    are treated the same, but they shouldn't be

use anyhow::Result;
use std::fs;
use std::path::Path;

#[derive(serde::Deserialize)]
struct ImportConfig {
    input_dir: String,
    output_file: String,
    delimiter: char,
}

fn load_config(path: &str) -> Result<ImportConfig> {
    let text = fs::read_to_string(path)?;
    let config: ImportConfig = serde_json::from_str(&text)?;
    Ok(config)
}

fn process_csv(path: &Path, delimiter: char) -> Result<Vec<String>> {
    let content = fs::read_to_string(path)?;
    let records: Vec<String> = content
        .lines()
        .skip(1) // skip header
        .map(|line| {
            let fields: Vec<&str> = line.split(delimiter).collect();
            // Bug: unwrap on index — panics if CSV has wrong number of columns
            format!("{}:{}", fields[0], fields[1].trim())
        })
        .collect();
    Ok(records)
}

fn main() -> Result<()> {
    // Fatal if config missing, but anyhow erases this intent
    let config = load_config("import-config.json")?;

    let entries: Vec<_> = fs::read_dir(&config.input_dir)?
        .filter_map(|e| e.ok()) // silently skips unreadable entries
        .filter(|e| e.path().extension().map(|x| x == "csv").unwrap_or(false))
        .collect();

    let mut all_records = Vec::new();
    let mut failed_files = Vec::new();

    for entry in &entries {
        let path = entry.path();
        // Bug: unwrap — one malformed CSV kills the whole import
        match process_csv(&path, config.delimiter) {
            Ok(records) => all_records.extend(records),
            Err(e) => {
                // "Handling" the error by printing and continuing
                // But: no structured logging, no trace ID, error detail lost
                eprintln!("Warning: failed to process {}: {}", path.display(), e);
                failed_files.push(path.display().to_string());
            }
        }
    }

    // Write summary — non-atomic write (power failure = corrupted file)
    let summary = format!(
        "Imported {} records from {} files ({} failed)\n\nRecords:\n{}",
        all_records.len(),
        entries.len(),
        failed_files.len(),
        all_records.join("\n")
    );
    fs::write(&config.output_file, &summary)?;

    // Bug: silently swallow cleanup errors — no log, no trace
    let _ = fs::remove_file("import.lock");
    let _ = fs::remove_dir_all("temp_staging");

    println!(
        "Done! {} records imported, {} files failed",
        all_records.len(),
        failed_files.len()
    );
    Ok(())
}