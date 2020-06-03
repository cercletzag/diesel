use crate::deserialize::{self, FromSql};
use crate::mysql::{Mysql, MysqlValue};
use crate::serialize::{self, IsNull, Output, ToSql};
use crate::sql_types;
use std::io::prelude::*;

impl FromSql<sql_types::Json, Mysql> for serde_json::Value {
    fn from_sql(value: Option<MysqlValue<'_>>) -> deserialize::Result<Self> {
        let value = not_none!(value);
        serde_json::from_slice(value.as_bytes()).map_err(|_| "Invalid Json".into())
    }
}

impl ToSql<sql_types::Json, Mysql> for serde_json::Value {
    fn to_sql<W: Write>(&self, out: &mut Output<W, Mysql>) -> serialize::Result {
        serde_json::to_writer(out, self)
            .map(|_| IsNull::No)
            .map_err(Into::into)
    }
}

#[test]
fn json_to_sql() {
    let mut bytes = Output::test();
    let test_json = serde_json::Value::Bool(true);
    ToSql::<sql_types::Json, Mysql>::to_sql(&test_json, &mut bytes).unwrap();
    assert_eq!(bytes, b"true");
}

#[test]
fn some_json_from_sql() {
    use crate::mysql::MysqlType;
    let input_json = b"true";
    let output_json: serde_json::Value = FromSql::<sql_types::Json, Mysql>::from_sql(Some(
        MysqlValue::new(input_json, MysqlType::Json),
    ))
    .unwrap();
    assert_eq!(output_json, serde_json::Value::Bool(true));
}

#[test]
fn bad_json_from_sql() {
    use crate::mysql::MysqlType;
    let uuid: Result<serde_json::Value, _> = FromSql::<sql_types::Json, Mysql>::from_sql(Some(
        MysqlValue::new(b"boom", MysqlType::Json),
    ));
    assert_eq!(uuid.unwrap_err().to_string(), "Invalid Json");
}

#[test]
fn no_json_from_sql() {
    let uuid: Result<serde_json::Value, _> = FromSql::<sql_types::Json, Mysql>::from_sql(None);
    assert_eq!(
        uuid.unwrap_err().to_string(),
        "Unexpected null for non-null column"
    );
}
