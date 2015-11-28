use expression::Expression;
use expression::aliased::Aliased;
use query_builder::*;
use query_source::QuerySource;

pub trait WithDsl<'a, Expr> {
    type Output: AsQuery;

    fn with(self, expr: Aliased<'a, Expr>) -> Self::Output;
}

impl<'a, T, Expr> WithDsl<'a, Expr> for T where
    T: QuerySource + AsQuery,
    T::Query: WithDsl<'a, Expr>
{
    type Output = <T::Query as WithDsl<'a, Expr>>::Output;

    fn with(self, expr: Aliased<'a, Expr>) -> Self::Output {
        self.as_query().with(expr)
    }
}

#[doc(hidden)]
pub struct WithQuerySource<'a, Left, Right> {
    left: Left,
    right: Aliased<'a, Right>,
}

impl<'a, Left, Right> WithQuerySource<'a, Left, Right> {
    pub fn new(left: Left, right: Aliased<'a, Right>) -> Self {
        WithQuerySource {
            left: left,
            right: right,
        }
    }
}

impl<'a, Left, Right> QuerySource for WithQuerySource<'a, Left, Right> where
    Left: QuerySource,
    Aliased<'a, Right>: QuerySource + Expression,
{
    fn from_clause(&self, out: &mut QueryBuilder) -> BuildQueryResult {
        try!(self.left.from_clause(out));
        out.push_sql(", ");
        self.right.from_clause(out)
    }
}
