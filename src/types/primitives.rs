extern crate byteorder;

use super::{NativeSqlType, FromSql, Nullable};
use Queriable;
use row::Row;

use self::byteorder::{ReadBytesExt, BigEndian};
use std::error::Error;
use std::fmt;

macro_rules! primitive_impls {
    ($($Source:ident -> $Target:ty),+,) => {
        $(
            impl NativeSqlType for super::$Source {}

            impl Queriable<super::$Source> for $Target {
                type Row = Self;

                fn build(row: Self::Row) -> Self {
                    row
                }
            }
        )+
    }
}

primitive_impls! {
    Bool -> bool,

    SmallSerial -> i16,
    Serial -> i32,
    BigSerial -> i64,

    SmallInt -> i16,
    Integer -> i32,
    BigInt -> i64,

    Float -> f32,
    Double -> f64,

    VarChar -> String,

    Binary -> Vec<u8>,
}

impl FromSql<super::Bool> for bool {
    fn from_sql(bytes: Option<&[u8]>) -> Result<Self, Box<Error>> {
        match bytes {
            Some(bytes) => Ok(bytes[0] != 0),
            None => unexpected_null(),
        }
    }
}

impl FromSql<super::SmallInt> for i16 {
    fn from_sql(bytes: Option<&[u8]>) -> Result<Self, Box<Error>> {
        match bytes {
            Some(mut bytes) => bytes.read_i16::<BigEndian>().map_err(|e| Box::new(e) as Box<Error>),
            None => unexpected_null(),
        }
    }
}

impl FromSql<super::SmallSerial> for i16 {
    fn from_sql(bytes: Option<&[u8]>) -> Result<Self, Box<Error>> {
        <Self as FromSql<super::SmallInt>>::from_sql(bytes)
    }
}

impl FromSql<super::Integer> for i32 {
    fn from_sql(bytes: Option<&[u8]>) -> Result<Self, Box<Error>> {
        match bytes {
            Some(mut bytes) => bytes.read_i32::<BigEndian>().map_err(|e| Box::new(e) as Box<Error>),
            None => unexpected_null(),
        }
    }
}

impl FromSql<super::Serial> for i32 {
    fn from_sql(bytes: Option<&[u8]>) -> Result<Self, Box<Error>> {
        <Self as FromSql<super::Integer>>::from_sql(bytes)
    }
}

impl FromSql<super::BigInt> for i64 {
    fn from_sql(bytes: Option<&[u8]>) -> Result<Self, Box<Error>> {
        match bytes {
            Some(mut bytes) => bytes.read_i64::<BigEndian>().map_err(|e| Box::new(e) as Box<Error>),
            None => unexpected_null(),
        }
    }
}

impl FromSql<super::BigSerial> for i64 {
    fn from_sql(bytes: Option<&[u8]>) -> Result<Self, Box<Error>> {
        <Self as FromSql<super::BigInt>>::from_sql(bytes)
    }
}

impl FromSql<super::Float> for f32 {
    fn from_sql(bytes: Option<&[u8]>) -> Result<Self, Box<Error>> {
        match bytes {
            Some(mut bytes) => bytes.read_f32::<BigEndian>().map_err(|e| Box::new(e) as Box<Error>),
            None => unexpected_null(),
        }
    }
}

impl FromSql<super::Double> for f64 {
    fn from_sql(bytes: Option<&[u8]>) -> Result<Self, Box<Error>> {
        match bytes {
            Some(mut bytes) => bytes.read_f64::<BigEndian>().map_err(|e| Box::new(e) as Box<Error>),
            None => unexpected_null(),
        }
    }
}

impl FromSql<super::VarChar> for String {
    fn from_sql(bytes: Option<&[u8]>) -> Result<Self, Box<Error>> {
        match bytes {
            Some(bytes) => String::from_utf8(bytes.into()).map_err(|e| Box::new(e) as Box<Error>),
            None => unexpected_null(),
        }
    }
}

impl FromSql<super::Binary> for Vec<u8> {
    fn from_sql(bytes: Option<&[u8]>) -> Result<Self, Box<Error>> {
        match bytes {
            Some(bytes) => Ok(bytes.into()),
            None => unexpected_null(),
        }
    }
}

impl<T: NativeSqlType> NativeSqlType for Nullable<T> {}
impl<T, ST> FromSql<Nullable<ST>> for Option<T> where
    T: FromSql<ST>,
    ST: NativeSqlType,
{
    fn from_sql(bytes: Option<&[u8]>) -> Result<Self, Box<Error>> {
        match bytes {
            Some(_) => T::from_sql(bytes).map(Some),
            None => Ok(None)
        }
    }
}

impl<T, ST> Queriable<Nullable<ST>> for Option<T> where
    T: FromSql<ST> + Queriable<ST>,
    ST: NativeSqlType,
{
    type Row = Self;
    fn build(row: Self) -> Self {
        row
    }
}

#[derive(Debug)]
struct UnexpectedNullError {
    msg: String,
}

impl fmt::Display for UnexpectedNullError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.msg)
    }
}

impl Error for UnexpectedNullError {
    fn description(&self) -> &str {
        &self.msg
    }
}

fn unexpected_null<T>() -> Result<T, Box<Error>> {
    Err(Box::new(UnexpectedNullError {
        msg: "Unexpected null for non-null column".to_string(),
    }))
}
