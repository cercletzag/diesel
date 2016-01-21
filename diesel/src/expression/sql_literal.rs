use backend::Backend;
use query_builder::*;
use std::marker::PhantomData;
use super::{Expression, SelectableExpression, NonAggregate};
use types::NativeSqlType;

#[derive(Debug, Clone)]
/// Available for when you truly cannot represent something using the expression
/// DSL. You will need to provide the type of the expression, in addition to the
/// SQL. The compiler will be unable to verify the correctness of this type.
pub struct SqlLiteral<ST> {
    sql: String,
    _marker: PhantomData<ST>,
}

impl<ST> SqlLiteral<ST> {
    pub fn new(sql: String) -> Self {
        SqlLiteral {
            sql: sql,
            _marker: PhantomData,
        }
    }
}

impl<ST: NativeSqlType> Expression for SqlLiteral<ST> {
    type SqlType = ST;
}

impl<ST, DB: Backend> QueryFragment<DB> for SqlLiteral<ST> {
    fn to_sql(&self, out: &mut DB::QueryBuilder) -> BuildQueryResult {
        out.push_sql(&self.sql);
        Ok(())
    }
}

impl<QS, ST: NativeSqlType> SelectableExpression<QS> for SqlLiteral<ST> {
}

impl<ST: NativeSqlType> NonAggregate for SqlLiteral<ST> {
}

pub fn sql<ST: NativeSqlType>(sql: &str) -> SqlLiteral<ST> {
    SqlLiteral::new(sql.into())
}
