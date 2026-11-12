//! The local replica: the one layer that holds SQL. Migrations are plain `.sql` files applied in
//! order; reads build a `World`, writes are small functions that each run in one transaction.

pub mod codes;
mod load;
#[cfg(test)]
pub(crate) mod tests;
mod write;

pub use write::*;

use rusqlite::Connection;

const MIGRATIONS: [&str; 1] = [include_str!("../../migrations/0001_init.sql")];

pub struct Store {
    connection: Connection,
}

#[derive(Debug)]
pub struct StoreError(pub String);

impl From<rusqlite::Error> for StoreError {
    fn from(error: rusqlite::Error) -> Self {
        Self(error.to_string())
    }
}

impl std::fmt::Display for StoreError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}

impl std::error::Error for StoreError {}

pub type Result<T> = std::result::Result<T, StoreError>;

impl Store {
    /// Opens the database at `path`, or one in memory for `None`, and brings its schema up to date.
    pub fn open(path: Option<&str>) -> Result<Self> {
        let connection = match path {
            Some(path) => Connection::open(path)?,
            None => Connection::open_in_memory()?,
        };
        connection.execute_batch("PRAGMA foreign_keys = ON; PRAGMA journal_mode = WAL;")?;
        let store = Self { connection };
        store.migrate()?;
        Ok(store)
    }

    fn migrate(&self) -> Result<()> {
        let version: i64 = self
            .connection
            .query_row("PRAGMA user_version", [], |row| row.get(0))?;
        if version > MIGRATIONS.len() as i64 {
            return Err(StoreError(format!(
                "the database is at schema version {version}, newer than the {} this build knows",
                MIGRATIONS.len()
            )));
        }
        for (i, migration) in (0i64..).zip(MIGRATIONS).skip(version as usize) {
            let transaction = self.connection.unchecked_transaction()?;
            transaction.execute_batch(migration)?;
            transaction.pragma_update(None, "user_version", i + 1)?;
            transaction.commit()?;
        }
        Ok(())
    }

    /// Rows changed since the database opened: a change shows as a different count.
    pub fn changes(&self) -> u64 {
        self.connection.total_changes()
    }

    pub(crate) fn connection(&self) -> &Connection {
        &self.connection
    }
}
