//! Types in this module are mostly internal and automatically generated. You
//! shouldn't need to interact with these types during normal usage, other than
//! the methods on [`Table`](trait.Table.html)
#[doc(hidden)]
pub mod filter;
mod joins;

use expression::{Expression, SelectableExpression, NonAggregate};
use query_builder::*;
#[doc(hidden)]
pub use self::joins::{InnerJoinSource, LeftOuterJoinSource};
use types::{FromSqlRow, NativeSqlType};

pub use self::joins::JoinTo;

/// Trait indicating that a record can be queried from the database. This trait
/// can be derived automatically. See the [codegen
/// documentation](https://github.com/sgrif/diesel/tree/master/diesel_codegen#derivequeryable)
/// for more.
pub trait Queryable<ST: NativeSqlType> {
    type Row: FromSqlRow<ST>;

    fn build(row: Self::Row) -> Self;
}

#[doc(hidden)]
pub trait QuerySource: Sized {
    fn from_clause(&self, out: &mut QueryBuilder) -> BuildQueryResult;
}

/// A column on a database table. Types which implement this trait should have
/// been generated by the [`table!` macro](../macro.table!.html).
pub trait Column: Expression {
    type Table: Table;

    fn name() -> &'static str;
}

/// A SQL database table. Types which implement this trait should have been
/// generated by the [`table!` macro](../macro.table!.html).
pub trait Table: QuerySource + AsQuery + Sized {
    type PrimaryKey: Column<Table=Self> + Expression + NonAggregate + QueryFragment;
    type AllColumns: SelectableExpression<Self> + QueryFragment;

    fn name() -> &'static str;
    fn primary_key(&self) -> Self::PrimaryKey;
    fn all_columns() -> Self::AllColumns;

    fn inner_join<T>(self, other: T) -> InnerJoinSource<Self, T> where
        T: Table,
        Self: JoinTo<T>,
    {
        InnerJoinSource::new(self, other)
    }

    fn left_outer_join<T>(self, other: T) -> LeftOuterJoinSource<Self, T> where
        T: Table,
        Self: JoinTo<T>,
    {
        LeftOuterJoinSource::new(self, other)
    }
}

impl<T: Table> UpdateTarget for T {
    type Table = Self;

    fn where_clause(&self, _out: &mut QueryBuilder) -> BuildQueryResult {
        Ok(())
    }
}
