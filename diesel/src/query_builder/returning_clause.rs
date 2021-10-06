use super::{AstPass, QueryFragment};
use crate::backend::Backend;
use crate::query_builder::QueryId;
use crate::result::QueryResult;

#[derive(Debug, Clone, Copy, QueryId)]
pub struct NoReturningClause;

impl<DB: Backend> QueryFragment<DB> for NoReturningClause {
    fn walk_ast(&self, _: AstPass<DB>) -> QueryResult<()> {
        Ok(())
    }
}

#[doc(hidden)]
#[derive(Debug, Clone, Copy, QueryId)]
pub struct ReturningClause<Expr>(pub Expr);

impl<Expr, DB> QueryFragment<DB> for ReturningClause<Expr>
where
    DB: Backend,
    Self: QueryFragment<DB, DB::ReturningClause>,
{
    fn walk_ast(&self, pass: AstPass<DB>) -> QueryResult<()> {
        <Self as QueryFragment<DB, DB::ReturningClause>>::walk_ast(self, pass)
    }
}

impl<Expr, DB>
    QueryFragment<DB, crate::backend::sql_dialect::returning_clause::PgLikeReturningClause>
    for ReturningClause<Expr>
where
    DB: Backend<
        ReturningClause = crate::backend::sql_dialect::returning_clause::PgLikeReturningClause,
    >,
    Expr: QueryFragment<DB>,
{
    fn walk_ast(&self, mut out: AstPass<DB>) -> QueryResult<()> {
        out.push_sql(" RETURNING ");
        self.0.walk_ast(out.reborrow())?;
        Ok(())
    }
}
