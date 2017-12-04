use query_source::Table;

/// The `offset` method
///
/// This trait should not be relied on directly by most apps. Its behavior is
/// provided by [`QueryDsl`]. However, you may need a where clause on this trait
/// to call `offset` from generic code.
///
/// [`QueryDsl`]: ../trait.QueryDsl.html
pub trait OffsetDsl {
    type Output;

    /// See the trait documentation
    fn offset(self, offset: i64) -> Self::Output;
}

impl<T> OffsetDsl for T
where
    T: Table,
    T::Query: OffsetDsl,
{
    type Output = <T::Query as OffsetDsl>::Output;

    fn offset(self, offset: i64) -> Self::Output {
        self.as_query().offset(offset)
    }
}
