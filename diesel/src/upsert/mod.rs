//! Types and functions related to PG's and Sqlite's `ON CONFLICT` clause
//!
//! Upsert is currently supported by diesel for the following database systems:
//!
//! * PostgreSQL version 9.5 or newer
//! * Sqlite3 version 3.24.0 or newer
//!
//! See [the methods on `InsertStatement`](../query_builder/struct.InsertStatement.html#impl-2)
//! for usage examples.
use crate::query_builder::upsert::on_conflict_actions::Excluded;

mod on_conflict_extension;

pub use self::on_conflict_extension::*;
#[cfg(feature = "postgres")]
pub use crate::pg::query_builder::on_constraint::*;

/// Represents `excluded.column` in an `ON CONFLICT DO UPDATE` clause.
pub fn excluded<T>(excluded: T) -> Excluded<T> {
    Excluded::new(excluded)
}
