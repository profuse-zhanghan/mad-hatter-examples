#[path = "../concept_map.rs"]
mod concept_map;
use concept_map::{CheckUserAccess, CheckUserAccessFatal};

#[derive(mad_hatter::FormatEnum)]
#[format_enum(rename_all = "snake_case")]
enum UserStatus {
    Active,
    Suspended,
}

mad_hatter::dao!(UsersDb, schema = "../../schema_users.json",
    resolver = |tenant: &str| {
        mad_hatter::db::DsConfig::sqlite(
            format!("data/{}/users.db", tenant)
        )
    }
);

#[mad_hatter::main(CheckUserAccess)]
async fn main(scope: &mad_hatter::TxScope<CheckUserAccess>) {
    let mut db = scope.db().await;

    db.execute(mad_hatter::__sqlx::query(
        "INSERT OR IGNORE INTO users (id, name, status) VALUES (1, 'alice', 'active')"
    )).await.propagate(scope, CheckUserAccessFatal::DbInsertUser);

    let (status,): (UserStatus,) = db.fetch_one(
        mad_hatter::__sqlx::query_as("SELECT status FROM users WHERE name = 'alice'")
    ).await.propagate(scope, CheckUserAccessFatal::DbQuery);

    match status {
        UserStatus::Active => println!("access granted"),
        UserStatus::Suspended => println!("account suspended"),
    }
}