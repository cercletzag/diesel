use std::error::Error;
use std::io::Write;

use sqlite::{Sqlite, SqliteType};
use sqlite::connection::SqliteValue;
use types::{self, FromSql, HasSqlType, IsNull, ToSql, ToSqlOutput};

#[cfg(feature = "chrono")]
mod chrono;

impl HasSqlType<types::Date> for Sqlite {
    fn metadata(_: &()) -> SqliteType {
        SqliteType::Text
    }
}

impl HasSqlType<types::Time> for Sqlite {
    fn metadata(_: &()) -> SqliteType {
        SqliteType::Text
    }
}

impl HasSqlType<types::Timestamp> for Sqlite {
    fn metadata(_: &()) -> SqliteType {
        SqliteType::Text
    }
}

impl FromSql<types::Date, Sqlite> for String {
    fn from_sql(value: Option<&SqliteValue>) -> Result<Self, Box<Error + Send + Sync>> {
        FromSql::<types::Text, Sqlite>::from_sql(value)
    }
}

impl ToSql<types::Date, Sqlite> for str {
    fn to_sql<W: Write>(
        &self,
        out: &mut ToSqlOutput<W, Sqlite>,
    ) -> Result<IsNull, Box<Error + Send + Sync>> {
        ToSql::<types::Text, Sqlite>::to_sql(self, out)
    }
}

impl ToSql<types::Date, Sqlite> for String {
    fn to_sql<W: Write>(
        &self,
        out: &mut ToSqlOutput<W, Sqlite>,
    ) -> Result<IsNull, Box<Error + Send + Sync>> {
        <&str as ToSql<types::Date, Sqlite>>::to_sql(&&**self, out)
    }
}

impl FromSql<types::Time, Sqlite> for String {
    fn from_sql(value: Option<&SqliteValue>) -> Result<Self, Box<Error + Send + Sync>> {
        FromSql::<types::Text, Sqlite>::from_sql(value)
    }
}

impl ToSql<types::Time, Sqlite> for str {
    fn to_sql<W: Write>(
        &self,
        out: &mut ToSqlOutput<W, Sqlite>,
    ) -> Result<IsNull, Box<Error + Send + Sync>> {
        ToSql::<types::Text, Sqlite>::to_sql(self, out)
    }
}

impl ToSql<types::Time, Sqlite> for String {
    fn to_sql<W: Write>(
        &self,
        out: &mut ToSqlOutput<W, Sqlite>,
    ) -> Result<IsNull, Box<Error + Send + Sync>> {
        <&str as ToSql<types::Time, Sqlite>>::to_sql(&&**self, out)
    }
}

impl FromSql<types::Timestamp, Sqlite> for String {
    fn from_sql(value: Option<&SqliteValue>) -> Result<Self, Box<Error + Send + Sync>> {
        FromSql::<types::Text, Sqlite>::from_sql(value)
    }
}

impl ToSql<types::Timestamp, Sqlite> for str {
    fn to_sql<W: Write>(
        &self,
        out: &mut ToSqlOutput<W, Sqlite>,
    ) -> Result<IsNull, Box<Error + Send + Sync>> {
        ToSql::<types::Text, Sqlite>::to_sql(self, out)
    }
}

impl ToSql<types::Timestamp, Sqlite> for String {
    fn to_sql<W: Write>(
        &self,
        out: &mut ToSqlOutput<W, Sqlite>,
    ) -> Result<IsNull, Box<Error + Send + Sync>> {
        <&str as ToSql<types::Timestamp, Sqlite>>::to_sql(&&**self, out)
    }
}
