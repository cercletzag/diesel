mod array;
pub mod date_and_time;
pub mod floats;
mod integers;
mod primitives;
#[cfg(feature = "uuid")]
mod uuid;
#[cfg(feature = "serde_json")]
mod json;

/// PostgreSQL specific SQL types
///
/// Note: All types in this module can be accessed through `diesel::types`
pub mod sql_types {
    /// The OID SQL type. This is a PostgreSQL specific type.
    ///
    /// ### [`ToSql`](/diesel/types/trait.ToSql.html) impls
    ///
    /// - [`u32`][u32]
    ///
    /// ### [`FromSql`](/diesel/types/trait.FromSql.html) impls
    ///
    /// - [`u32`][u32]
    ///
    /// [u32]: https://doc.rust-lang.org/nightly/std/primitive.u32.html
    #[derive(Debug, Clone, Copy, Default)] pub struct Oid;

    /// The "timestamp with time zone" SQL type, which PostgreSQL abbreviates
    /// to `timestamptz`.
    ///
    /// ### [`ToSql`](/diesel/types/trait.ToSql.html) impls
    ///
    /// - [`PgTimestamp`][PgTimestamp]
    /// - [`chrono::NaiveDateTime`][NaiveDateTime] with `feature = "chrono"`
    /// - [`chrono::DateTime`][DateTime] with `feature = "chrono"`
    ///
    /// ### [`FromSql`](/diesel/types/trait.FromSql.html) impls
    ///
    /// - [`PgTimestamp`][PgTimestamp]
    /// - [`chrono::NaiveDateTime`][NaiveDateTime] with `feature = "chrono"`
    /// - [`chrono::DateTime`][DateTime] with `feature = "chrono"`
    ///
    /// [PgTimestamp]: /diesel/pg/data_types/struct.PgTimestamp.html
    /// [NaiveDateTime]: https://lifthrasiir.github.io/rust-chrono/chrono/naive/datetime/struct.NaiveDateTime.html
    /// [DateTime]: https://lifthrasiir.github.io/rust-chrono/chrono/datetime/struct.DateTime.html
    #[derive(Debug, Clone, Copy, Default)] pub struct Timestamptz;

    /// The Array SQL type. This wraps another type to represent a SQL array of
    /// that type. Multidimensional arrays are not supported, nor are arrays
    /// containing null.
    ///
    /// ### [`ToSql`](/diesel/types/trait.ToSql.html) impls
    ///
    /// - [`Vec<T>`][Vec] for any `T` which implements `ToSql<ST>`
    /// - [`&[T]`][slice] for any `T` which implements `ToSql<ST>`
    ///
    /// ### [`FromSql`](/diesel/types/trait.FromSql.html) impls
    ///
    /// - [`Vec<T>`][Vec] for any `T` which implements `ToSql<ST>`
    ///
    /// [Vec]: https://doc.rust-lang.org/nightly/std/vec/struct.Vec.html
    /// [slice]: https://doc.rust-lang.org/nightly/std/primitive.slice.html
    #[derive(Debug, Clone, Copy, Default)] pub struct Array<ST>(ST);

    /// Alias for SmallInt
    pub type SmallSerial = ::types::SmallInt;

    /// Alias for Integer
    pub type Serial = ::types::Integer;

    /// Alias for BigInt
    pub type BigSerial = ::types::BigInt;

    #[cfg(feature = "uuid")]
    /// The UUID SQL type. This type can only be used with `feature = "uuid"`
    ///
    /// ### [`ToSql`](/diesel/types/trait.ToSql.html) impls
    ///
    /// - [`uuid::Uuid`][Uuid]
    ///
    /// ### [`FromSql`](/diesel/types/trait.FromSql.html) impls
    ///
    /// - [`uuid::Uuid`][Uuid]
    ///
    /// [Vec]: https://doc.rust-lang.org/uuid/uuid/struct.Uuid.html
    #[derive(Debug, Clone, Copy, Default)] pub struct Uuid;

    /// Alias for `Binary`, to ensure `infer_schema!` works
    pub type Bytea = ::types::Binary;

    #[doc(hidden)]
    pub type Bpchar = ::types::VarChar;

    #[cfg(feature = "serde_json")]
    /// The JSON SQL type.  This type can only be used with `feature =
    /// "serde_json"`
    ///
    /// Normally you should prefer `Jsonb` instead, for the reasons
    /// discussed there.
    ///
    /// ### [`ToSql`](/diesel/types/trait.ToSql.html) impls
    ///
    /// - [`serde_json::Value`][Value]
    ///
    /// ### [`FromSql`](/diesel/types/trait.FromSql.html) impls
    ///
    /// - [`serde_json`][Value]
    ///
    /// [Value]: https://docs.serde.rs/serde_json/value/enum.Value.html
    #[derive(Debug, Clone, Copy, Default)] pub struct Json;

    #[cfg(feature = "serde_json")]
    /// The `jsonb` SQL type.  This type can only be used with `feature =
    /// "serde_json"`
    ///
    /// `jsonb` offers [several advantages][adv] over regular JSON:
    ///
    /// > There are two JSON data types: `json` and `jsonb`. They accept almost
    /// > identical sets of values as input. The major practical difference
    /// > is one of efficiency. The `json` data type stores an exact copy of
    /// > the input text, which processing functions must reparse on each
    /// > execution; while `jsonb` data is stored in a decomposed binary format
    /// > that makes it slightly slower to input due to added conversion
    /// > overhead, but significantly faster to process, since no reparsing
    /// > is needed. `jsonb` also supports indexing, which can be a significant
    /// > advantage.
    /// >
    /// > ...In general, most applications should prefer to store JSON data as
    /// > `jsonb`, unless there are quite specialized needs, such as legacy
    /// > assumptions about ordering of object keys.
    ///
    /// [adv]: https://www.postgresql.org/docs/9.6/static/datatype-json.html
    ///
    /// ### [`ToSql`](/diesel/types/trait.ToSql.html) impls
    ///
    /// - [`serde_json::Value`][Value]
    ///
    /// ### [`FromSql`](/diesel/types/trait.FromSql.html) impls
    ///
    /// - [`serde_json`][Value]
    ///
    /// [Value]: https://docs.serde.rs/serde_json/value/enum.Value.html
    #[derive(Debug, Clone, Copy, Default)] pub struct Jsonb;
}
