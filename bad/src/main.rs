// Story 1: Ghost Status Code
// Two functions use string literals for the same concept,
// but with different casing — a silent runtime mismatch.

fn set_user_status() -> String {
    "active".to_string()
}

fn is_user_active(status: &str) -> bool {
    status == "Active" // Bug: capital A — never matches "active"
}

fn main() {
    let status = set_user_status();
    if is_user_active(&status) {
        println!("allowed");
    } else {
        println!("denied"); // always reaches here
    }
}