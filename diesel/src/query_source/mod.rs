//! Types in this module are mostly internal and automatically generated. You
//! shouldn't need to interact with these types during normal usage, other than
//! the methods on [`Table`](/diesel/query_source/trait.Table.html)
#[doc(hidden)]
pub mod joins;

use backend::Backend;
use expression::{Expression, SelectableExpression, NonAggregate};
use query_builder::*;
use types::{FromSqlRow, HasSqlType};

pub use self::joins::JoinTo;
use self::joins::*;

/// Trait indicating that a record can be queried from the database. This trait
/// can be derived automatically using `diesel_codegen`. This trait can only be derived for
/// structs, not enums.
pub trait Queryable<ST, DB> where
    DB: Backend + HasSqlType<ST>,
{
    type Row: FromSqlRow<ST, DB>;

    fn build(row: Self::Row) -> Self;
}

#[doc(hidden)]
pub trait QuerySource {
    type FromClause;
    type DefaultSelection: SelectableExpression<Self>;

    fn from_clause(&self) -> Self::FromClause;
    fn default_selection(&self) -> Self::DefaultSelection;
}

/// A column on a database table. Types which implement this trait should have
/// been generated by the [`table!` macro](../macro.table.html).
pub trait Column: Expression {
    type Table: Table;

    fn name() -> &'static str;
}

/// A SQL database table. Types which implement this trait should have been
/// generated by the [`table!` macro](../macro.table.html).
pub trait Table: QuerySource + AsQuery + Sized {
    type PrimaryKey: SelectableExpression<Self> + NonAggregate;
    type AllColumns: SelectableExpression<Self> + NonAggregate;

    fn primary_key(&self) -> Self::PrimaryKey;
    fn all_columns() -> Self::AllColumns;

    fn inner_join<T>(self, other: T) -> Join<Self, T, Inner> where
        T: Table,
        Self: JoinTo<T, Inner>,
    {
        Join::new(self, other)
    }

    fn left_outer_join<T>(self, other: T) -> Join<Self, T, LeftOuter> where
        T: Table,
        Self: JoinTo<T, LeftOuter>,
    {
        Join::new(self, other)
    }
}
