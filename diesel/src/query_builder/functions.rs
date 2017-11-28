use dsl::Select;
use expression::Expression;
use query_dsl::SelectDsl;
use super::delete_statement::DeleteStatement;
#[cfg(feature = "with-deprecated")]
use super::insert_statement::{DefaultValues, DeprecatedIncompleteInsertStatement};
use super::insert_statement::{Insert, Replace};
use super::{IncompleteInsertStatement, IncompleteUpdateStatement, IntoUpdateTarget,
            SelectStatement, SqlQuery};

/// Creates an `UPDATE` statement. Helpers for updating a single row can be
/// generated by deriving [`AsChangeset`](../trait.AsChangeset.html)
///
/// When a table is passed to `update`, every row in the table will be updated.
/// You can narrow this scope by calling [`filter`] on the table before passing it in,
/// which will result in `UPDATE your_table SET ... WHERE args_to_filter`.
///
/// Passing a type which implements `Identifiable` is the same as passing
/// `some_table.find(some_struct.id())`.
///
/// [`filter`]: ../diesel/query_builder/update_statement/struct.IncompleteUpdateStatement.html#method.filter
///
/// # Examples
///
/// ```rust
/// # #[macro_use] extern crate diesel;
/// # include!("../doctest_setup.rs");
/// #
/// # table! {
/// #     users {
/// #         id -> Integer,
/// #         name -> VarChar,
/// #     }
/// # }
/// #
/// # #[cfg(feature = "postgres")]
/// # fn main() {
/// #     use users::dsl::*;
/// #     let connection = establish_connection();
/// let updated_row = diesel::update(users.filter(id.eq(1)))
///     .set(name.eq("James"))
///     .get_result(&connection);
/// // On backends that support it, you can call `get_result` instead of `execute`
/// // to have `RETURNING *` automatically appended to the query. Alternatively, you
/// // can explicitly return an expression by using the `returning` method before
/// // getting the result.
/// assert_eq!(Ok((1, "James".to_string())), updated_row);
/// # }
/// # #[cfg(not(feature = "postgres"))]
/// # fn main() {}
/// ```
///
/// To update multiple columns, give [`set`] a tuple argument:
///
/// [`set`]: ../diesel/query_builder/struct.IncompleteUpdateStatement.html#method.set
///
/// ```rust
/// # #[macro_use] extern crate diesel;
/// # include!("../doctest_setup.rs");
/// #
/// # table! {
/// #     users {
/// #         id -> Integer,
/// #         name -> VarChar,
/// #         surname -> VarChar,
/// #     }
/// # }
/// #
/// # #[cfg(feature = "postgres")]
/// # fn main() {
/// # use users::dsl::*;
/// # let connection = establish_connection();
/// # connection.execute("DROP TABLE users").unwrap();
/// # connection.execute("CREATE TABLE users (
/// #     id SERIAL PRIMARY KEY,
/// #     name VARCHAR,
/// #     surname VARCHAR)").unwrap();
/// # connection.execute("INSERT INTO users(name, surname) VALUES('Sean', 'Griffin')").unwrap();
///
/// let updated_row = diesel::update(users.filter(id.eq(1)))
///     .set((name.eq("James"), surname.eq("Bond")))
///     .get_result(&connection);
///
/// assert_eq!(Ok((1, "James".to_string(), "Bond".to_string())), updated_row);
/// # }
/// # #[cfg(not(feature = "postgres"))]
/// # fn main() {}
/// ```
pub fn update<T: IntoUpdateTarget>(
    source: T,
) -> IncompleteUpdateStatement<T::Table, T::WhereClause> {
    IncompleteUpdateStatement::new(source.into_update_target())
}

/// Creates a `DELETE` statement.
///
/// When a table is passed to `delete`,
/// every row in the table will be deleted.
/// This scope can be narrowed by calling [`filter`]
/// on the table before it is passed in.
///
/// [`filter`]: ../diesel/query_builder/struct.DeleteStatement.html#method.filter
///
/// # Examples
///
/// ### Deleting a single record:
///
/// ```rust
/// # #[macro_use] extern crate diesel;
/// # include!("../doctest_setup.rs");
/// #
/// # table! {
/// #     users {
/// #         id -> Integer,
/// #         name -> VarChar,
/// #     }
/// # }
/// #
/// # fn main() {
/// #     delete();
/// # }
/// #
/// # fn delete() -> QueryResult<()> {
/// #     use users::dsl::*;
/// #     let connection = establish_connection();
/// #     let get_count = || users.count().first::<i64>(&connection);
/// let old_count = get_count();
/// try!(diesel::delete(users.filter(id.eq(1))).execute(&connection));
/// assert_eq!(old_count.map(|count| count - 1), get_count());
/// # Ok(())
/// # }
/// ```
///
/// ### Deleting a whole table:
///
/// ```rust
/// # #[macro_use] extern crate diesel;
/// # include!("../doctest_setup.rs");
/// #
/// # table! {
/// #     users {
/// #         id -> Integer,
/// #         name -> VarChar,
/// #     }
/// # }
/// #
/// # fn main() {
/// #     delete();
/// # }
/// #
/// # fn delete() -> QueryResult<()> {
/// #     use users::dsl::*;
/// #     let connection = establish_connection();
/// #     let get_count = || users.count().first::<i64>(&connection);
/// try!(diesel::delete(users).execute(&connection));
/// assert_eq!(Ok(0), get_count());
/// # Ok(())
/// # }
/// ```
pub fn delete<T: IntoUpdateTarget>(source: T) -> DeleteStatement<T::Table, T::WhereClause> {
    let target = source.into_update_target();
    DeleteStatement::new(target.table, target.where_clause)
}

/// Creates an `INSERT` statement.
///
/// Will add the given data to a table.
/// Backends that support the `RETURNING` clause, such as PostgreSQL,
/// can return the inserted rows by calling [`.get_results`] instead of [`.execute`].
///
/// [`.get_results`]: ../diesel/prelude/trait.LoadDsl.html#method.get_results
/// [`.execute`]: ../diesel/prelude/trait.ExecuteDsl.html#tymethod.execute
///
/// # Examples
///
/// ```rust
/// # #[macro_use] extern crate diesel;
/// # include!("../doctest_setup.rs");
/// #
/// # table! {
/// #     users {
/// #         id -> Integer,
/// #         name -> Text,
/// #     }
/// # }
/// #
/// # fn main() {
/// #     use self::users::dsl::*;
/// #     let connection = establish_connection();
/// let rows_inserted = diesel::insert(&name.eq("Sean"))
///     .into(users)
///     .execute(&connection);
///
/// assert_eq!(Ok(1), rows_inserted);
///
/// let new_users = vec![
///     name.eq("Tess"),
///     name.eq("Jim"),
/// ];
///
/// let rows_inserted = diesel::insert(&new_users)
///     .into(users)
///     .execute(&connection);
///
/// assert_eq!(Ok(2), rows_inserted);
/// # }
/// ```
///
/// ### Using a tuple for values
///
/// ```rust
/// # #[macro_use] extern crate diesel;
/// # include!("../doctest_setup.rs");
/// #
/// # table! {
/// #     users {
/// #         id -> Integer,
/// #         name -> Text,
/// #     }
/// # }
/// #
/// # fn main() {
/// #     use self::users::dsl::*;
/// #     let connection = establish_connection();
/// #     diesel::delete(users).execute(&connection).unwrap();
/// let new_user = (id.eq(1), name.eq("Sean"));
/// let rows_inserted = diesel::insert(&new_user)
///     .into(users)
///     .execute(&connection);
///
/// assert_eq!(Ok(1), rows_inserted);
///
/// let new_users = vec![
///     (id.eq(2), name.eq("Tess")),
///     (id.eq(2), name.eq("Jim")),
/// ];
///
/// let rows_inserted = diesel::insert(&new_users)
///     .into(users)
///     .execute(&connection);
///
/// // assert_eq!(Ok(2), rows_inserted);
/// # }
/// ```
///
/// ### Using struct for values
///
/// ```rust
/// # #[macro_use] extern crate diesel;
/// # include!("../doctest_setup.rs");
/// #
/// # table! {
/// #     users {
/// #         id -> Integer,
/// #         name -> Text,
/// #     }
/// # }
/// #
/// # fn main() {
/// #     use self::users::dsl::*;
/// #     let connection = establish_connection();
/// // Insert one record at a time
///
/// let new_user = NewUser { name: "Ruby Rhod".to_string() };
///
/// diesel::insert(&new_user)
///     .into(users)
///     .execute(&connection)
///     .unwrap();
///
/// // Insert many records
///
/// let new_users = vec![
///     NewUser { name: "Leeloo Multipass".to_string(), },
///     NewUser { name: "Korben Dallas".to_string(), },
/// ];
///
/// let inserted_names = diesel::insert(&new_users)
///     .into(users)
///     .execute(&connection)
///     .unwrap();
/// # }
/// ```
///
/// ### With return value
///
/// ```rust
/// # #[macro_use] extern crate diesel;
/// # include!("../doctest_setup.rs");
/// #
/// # table! {
/// #     users {
/// #         id -> Integer,
/// #         name -> Text,
/// #     }
/// # }
/// #
/// # #[cfg(feature = "postgres")]
/// # fn main() {
/// #     use self::users::dsl::*;
/// #     let connection = establish_connection();
/// // postgres only
/// let new_users = vec![
///     NewUser { name: "Diva Plavalaguna".to_string(), },
///     NewUser { name: "Father Vito Cornelius".to_string(), },
/// ];
///
/// let inserted_names = diesel::insert(&new_users)
///     .into(users)
///     .returning(name)
///     .get_results(&connection);
/// assert_eq!(Ok(vec!["Diva Plavalaguna".to_string(), "Father Vito Cornelius".to_string()]), inserted_names);
/// # }
/// # #[cfg(not(feature = "postgres"))]
/// # fn main() {}
/// ```
#[cfg(feature = "with-deprecated")]
#[deprecated(since = "0.99.0", note = "use `insert_into` instead")]
pub fn insert<T: ?Sized>(records: &T) -> DeprecatedIncompleteInsertStatement<&T, Insert> {
    DeprecatedIncompleteInsertStatement::new(records, Insert)
}

/// Creates an `INSERT` statement for the target table.
///
/// You may add data by calling [`values()`] or [`default_values()`]
/// as shown in the examples.
///
/// [`values()`]: ../diesel/query_builder/insert_statement/struct.IncompleteInsertStatement.html#method.values
/// [`default_values()`]: ../diesel/query_builder/insert_statement/struct.IncompleteInsertStatement.html#method.default_values
///
/// Backends that support the `RETURNING` clause, such as PostgreSQL,
/// can return the inserted rows by calling [`.get_results`] instead of [`.execute`].
///
/// [`.get_results`]: ../diesel/prelude/trait.LoadDsl.html#method.get_results
/// [`.execute`]: ../diesel/prelude/trait.ExecuteDsl.html#tymethod.execute
///
/// # Examples
///
/// ```rust
/// # #[macro_use] extern crate diesel;
/// # include!("../doctest_setup.rs");
/// #
/// # table! {
/// #     users {
/// #         id -> Integer,
/// #         name -> Text,
/// #     }
/// # }
/// #
/// # fn main() {
/// #     use users::dsl::*;
/// #     let connection = establish_connection();
/// let rows_inserted = diesel::insert_into(users)
///     .values(&name.eq("Sean"))
///     .execute(&connection);
///
/// assert_eq!(Ok(1), rows_inserted);
///
/// let new_users = vec![
///     name.eq("Tess"),
///     name.eq("Jim"),
/// ];
///
/// let rows_inserted = diesel::insert_into(users)
///     .values(&new_users)
///     .execute(&connection);
///
/// assert_eq!(Ok(2), rows_inserted);
/// # }
/// ```
///
/// ### Using a tuple for values
///
/// ```rust
/// # #[macro_use] extern crate diesel;
/// # include!("../doctest_setup.rs");
/// #
/// # table! {
/// #     users {
/// #         id -> Integer,
/// #         name -> Text,
/// #     }
/// # }
/// #
/// # fn main() {
/// #     use users::dsl::*;
/// #     let connection = establish_connection();
/// #     diesel::delete(users).execute(&connection).unwrap();
/// let new_user = (id.eq(1), name.eq("Sean"));
/// let rows_inserted = diesel::insert_into(users)
///     .values(&new_user)
///     .execute(&connection);
///
/// assert_eq!(Ok(1), rows_inserted);
///
/// let new_users = vec![
///     (id.eq(2), name.eq("Tess")),
///     (id.eq(2), name.eq("Jim")),
/// ];
///
/// let rows_inserted = diesel::insert_into(users)
///     .values(&new_users)
///     .execute(&connection);
///
/// // assert_eq!(Ok(2), rows_inserted);
/// # }
/// ```
///
/// ### Using struct for values
///
/// ```rust
/// # #[macro_use] extern crate diesel;
/// # include!("../doctest_setup.rs");
/// #
/// # table! {
/// #     users {
/// #         id -> Integer,
/// #         name -> Text,
/// #     }
/// # }
/// #
/// # fn main() {
/// #     use users::dsl::*;
/// #     let connection = establish_connection();
/// // Insert one record at a time
///
/// let new_user = NewUser { name: "Ruby Rhod".to_string() };
///
/// diesel::insert_into(users)
///     .values(&new_user)
///     .execute(&connection)
///     .unwrap();
///
/// // Insert many records
///
/// let new_users = vec![
///     NewUser { name: "Leeloo Multipass".to_string(), },
///     NewUser { name: "Korben Dallas".to_string(), },
/// ];
///
/// let inserted_names = diesel::insert_into(users)
///     .values(&new_users)
///     .execute(&connection)
///     .unwrap();
/// # }
/// ```
///
/// ### With return value
///
/// ```rust
/// # #[macro_use] extern crate diesel;
/// # include!("../doctest_setup.rs");
/// #
/// # table! {
/// #     users {
/// #         id -> Integer,
/// #         name -> Text,
/// #     }
/// # }
/// #
/// # #[cfg(feature = "postgres")]
/// # fn main() {
/// #     use users::dsl::*;
/// #     let connection = establish_connection();
/// // postgres only
/// let new_users = vec![
///     NewUser { name: "Diva Plavalaguna".to_string(), },
///     NewUser { name: "Father Vito Cornelius".to_string(), },
/// ];
///
/// let inserted_names = diesel::insert_into(users)
///     .values(&new_users)
///     .returning(name)
///     .get_results(&connection);
/// assert_eq!(Ok(vec!["Diva Plavalaguna".to_string(), "Father Vito Cornelius".to_string()]), inserted_names);
/// # }
/// # #[cfg(not(feature = "postgres"))]
/// # fn main() {}
/// ```
pub fn insert_into<T>(target: T) -> IncompleteInsertStatement<T, Insert> {
    IncompleteInsertStatement::new(target, Insert)
}

/// Creates a bare select statement, with no from clause. Primarily used for
/// testing diesel itself, but likely useful for third party crates as well. The
/// given expressions must be selectable from anywhere.
pub fn select<T>(expression: T) -> Select<SelectStatement<()>, T>
where
    T: Expression,
    SelectStatement<()>: SelectDsl<T>,
{
    SelectStatement::simple(()).select(expression)
}

/// Creates an insert statement with default values.
///
#[cfg(feature = "with-deprecated")]
#[deprecated(since = "0.99.0", note = "use `insert_into(table).default_values()` instead")]
#[allow(deprecated)]
pub fn insert_default_values() -> DeprecatedIncompleteInsertStatement<&'static DefaultValues, Insert> {
    static STATIC_DEFAULT_VALUES: &'static DefaultValues = &DefaultValues;
    insert(STATIC_DEFAULT_VALUES)
}

/// Creates a `REPLACE` statement.
///
/// If a constraint violation fails, the database will attempt to replace the
/// offending row instead. This function is only available with MySQL and
/// SQLite.
///
/// # Example
///
/// ```rust
/// # #[macro_use] extern crate diesel;
/// # include!("../doctest_setup.rs");
/// #
/// # table! {
/// #     users {
/// #         id -> Integer,
/// #         name -> VarChar,
/// #     }
/// # }
/// #
/// # #[derive(Insertable)]
/// # #[table_name="users"]
/// # struct User<'a> {
/// #     id: i32,
/// #     name: &'a str,
/// # }
/// #
/// # #[cfg(not(feature = "postgres"))]
/// # fn main() {
/// #     use users::dsl::*;
/// #     use diesel::{insert_into, replace_into};
/// #
/// #     let conn = establish_connection();
/// #     conn.execute("DELETE FROM users").unwrap();
/// replace_into(users)
///     .values(&vec![
///         User {
///             id: 1,
///             name: "Sean",
///         },
///         User {
///             id: 2,
///             name: "Tess",
///         },
///     ])
///     .execute(&conn)
///     .unwrap();
///
/// let new_user = User { id: 1, name: "Jim" };
/// replace_into(users)
///     .values(&new_user)
///     .execute(&conn)
///     .unwrap();
///
/// let names = users.select(name).order(id).load::<String>(&conn);
/// assert_eq!(Ok(vec!["Jim".into(), "Tess".into()]), names);
/// # }
/// # #[cfg(feature = "postgres")] fn main() {}
pub fn replace_into<T>(target: T) -> IncompleteInsertStatement<T, Replace> {
    IncompleteInsertStatement::new(target, Replace)
}

/// Construct a full SQL query using raw SQL.
///
/// This function exists for cases where a query needs to be written that is not
/// supported by the query builder. Unlike most queries in Diesel, `sql_query`
/// will deserialize its data by name, not by index. That means that you cannot
/// deserialize into a tuple, and structs which you deserialize from this
/// function will need to have `#[derive(QueryableByName)]`
///
/// # Safety
///
/// The implementation of `QueryableByName` will assume that columns with a
/// given name will have a certain type. The compiler will be unable to verify
/// that the given type is correct. If your query returns a column of an
/// unexpected type, the result may have the wrong value, or return an error.
///
/// # Example
///
/// ```rust
/// # #[macro_use] extern crate diesel;
/// # include!("../doctest_setup.rs");
/// #
/// # use schema::users;
/// #
/// # #[derive(QueryableByName, Debug, PartialEq)]
/// # #[table_name="users"]
/// # struct User {
/// #     id: i32,
/// #     name: String,
/// # }
/// #
/// # fn main() {
/// #     use diesel::sql_query;
/// #
/// #     let connection = establish_connection();
/// let users = sql_query("SELECT * FROM users ORDER BY id")
///     .load(&connection);
/// let expected_users = vec![
///     User { id: 1, name: "Sean".into() },
///     User { id: 2, name: "Tess".into() },
/// ];
/// assert_eq!(Ok(expected_users), users);
/// # }
/// ```
pub fn sql_query<T: Into<String>>(query: T) -> SqlQuery {
    SqlQuery::new(query.into())
}
