use did::User;
use ic_cdk_macros::{post_upgrade, pre_upgrade, query, update};

#[update]
fn create() {
    let conn = ic_sqlite::CONN.lock().unwrap();
    if let Err(err) = conn.execute(
        "CREATE TABLE users (
            id   INTEGER PRIMARY KEY,
            name TEXT NOT NULL,
            age INTEGER
        )",
        [],
    ) {
        trap(format!("Failed to create table: {err}"));
    }
}

#[update]
fn insert_user(user: User) {
    let Ok(conn) = ic_sqlite::CONN.lock() else {
        trap("Failed top open database");
    };

    let mut stmt = match conn.prepare("INSERT INTO users (id, name, age) VALUES (?1, ?2, ?3);") {
        Ok(stmt) => stmt,
        Err(err) => {
            trap(format!("Failed to prepare statement: {err}"));
        }
    };

    if let Err(err) = stmt.execute((user.id, user.name, user.age)) {
        trap(format!("Failed to execute query: {err}"));
    }
}

#[query]
fn get_user(index: u64) -> User {
    let Ok(conn) = ic_sqlite::CONN.lock() else {
        trap("Failed top open database");
    };

    let Ok(mut stmt) = conn.prepare("SELECT * from users LIMIT 1 OFFSET ?1") else {
        trap("Failed to prepare statement");
    };
    let iter = match stmt.query_map((index,), |row| {
        Ok(User {
            id: row.get(0).unwrap(),
            name: row.get(1).unwrap(),
            age: row.get(2).unwrap(),
        })
    }) {
        Ok(e) => e,
        Err(err) => {
            trap(format!("Failed to query user: {err})"));
        }
    };
    if let Some(Ok(user)) = iter.into_iter().next() {
        user
    } else {
        trap("No such user");
    }
}

#[pre_upgrade]
fn pre_upgrade() {}

#[post_upgrade]
fn post_upgrade() {}

ic_cdk::export_candid!();

/// Utility functions to trap the canister.
///
/// The reason of this is that you cannot use [`panic!`] on canisters and you can't use
/// [`ic_cdk::trap`] in test units.
fn trap<S>(msg: S) -> !
where
    S: AsRef<str>,
{
    if cfg!(target_family = "wasm") {
        ic_cdk::trap(msg)
    } else {
        panic!("{}", msg.as_ref())
    }
}
