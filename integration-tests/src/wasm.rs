use std::path::Path;

pub enum Canister {
    SqliteDb,
}

impl Canister {
    pub fn as_path(&self) -> &'static Path {
        match self {
            Canister::SqliteDb => Path::new("../.artifact/sqlite_db.wasm.gz"),
        }
    }
}
