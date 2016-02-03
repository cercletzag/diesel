mod array;
pub mod date_and_time;
pub mod floats;
mod integers;
mod primitives;

#[doc(hidden)]
pub mod sql_types {
    #[derive(Clone, Copy, Default)] pub struct Oid;
    #[derive(Clone, Copy, Default)] pub struct Array<T>(T);
    pub type SmallSerial = ::types::SmallInt;
    pub type Serial = ::types::Integer;
    pub type BigSerial = ::types::BigInt;
}
