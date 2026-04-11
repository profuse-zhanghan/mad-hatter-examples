// Story 1: Ghost Status Code
//
// A user management system reads status from SQLite and checks access.
// Bug: "active" in DB vs "Active" in code — case mismatch, silent failure.

use rusqlite::Connection;

fn main() {
    let conn = Connection::open("users.db").unwrap();

    conn.execute(
        "CREATE TABLE IF NOT EXISTS users (id INTEGER PRIMARY KEY, name TEXT, status TEXT)",
        [],
    ).unwrap();

    conn.execute(
        "INSERT OR IGNORE INTO users (id, name, status) VALUES (1, 'alice', 'active')",
        [],
    ).unwrap();

    let status: String = conn.query_row(
        "SELECT status FROM users WHERE name = 'alice'",
        [],
        |row| row.get(0),
    ).unwrap();

    // Bug: DB stores "active" but we compare with "Active"
    if status == "Active" {
        println!("access granted");
    } else if status == "suspended" {
        println!("account suspended");
    } else {
        println!("unknown status: {}", status);
    }
}