use expression::{Expression, SelectableExpression, NonAggregate};
use query_builder::{QueryBuilder, BuildQueryResult};

pub struct Grouped<T>(pub T);

impl<T: Expression> Expression for Grouped<T> {
    type SqlType = T::SqlType;

    fn to_sql(&self, out: &mut QueryBuilder) -> BuildQueryResult {
        out.push_sql("(");
        try!(self.0.to_sql(out));
        out.push_sql(")");
        Ok(())
    }
}

impl<T, QS> SelectableExpression<QS> for Grouped<T> where
    T: SelectableExpression<QS>,
    Grouped<T>: Expression,
{
}

impl<T: NonAggregate> NonAggregate for Grouped<T> where
    Grouped<T>: Expression,
{
}
