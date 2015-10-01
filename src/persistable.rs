use query_source::{Table, Column};
use types::{ValuesToSql, Nullable, NativeSqlType};

pub trait Insertable<T: Table> {
    type Columns: InsertableColumns<Table=T>;
    type Values: ValuesToSql<<Self::Columns as InsertableColumns>::SqlType> +
        AsBindParam<<Self::Columns as InsertableColumns>::SqlType>;

    fn columns() -> Self::Columns;

    fn values(self) -> Self::Values;
}

pub trait InsertableColumns {
    type Table: Table;
    type SqlType: NativeSqlType;

    fn names(&self) -> String;
}

impl<C: Column> InsertableColumns for C {
    type Table = <Self as Column>::Table;
    type SqlType = <Self as Column>::SqlType;

    fn names(&self) -> String {
        self.name()
    }
}

pub trait AsBindParam<T: NativeSqlType> {
    fn as_bind_param(&self, idx: &mut usize) -> String {
        let result = format!("${}", idx);
        *idx += 1;
        result
    }

    fn as_bind_param_for_insert(&self, idx: &mut usize) -> String;
}

impl<T, ST> AsBindParam<Nullable<ST>> for Option<T> where
    T: AsBindParam<ST>,
    ST: NativeSqlType,
{
    fn as_bind_param_for_insert(&self, idx: &mut usize) -> String {
        match self {
            &Some(ref value) => value.as_bind_param_for_insert(idx),
            &None => "DEFAULT".to_string(),
        }
    }
}
