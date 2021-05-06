use crate::expression::Expression;
use crate::query_builder::upsert::into_conflict_clause::IntoConflictValueClause;
use crate::query_builder::upsert::on_conflict_actions::*;
use crate::query_builder::upsert::on_conflict_clause::*;
use crate::query_builder::upsert::on_conflict_target::*;
pub use crate::query_builder::upsert::on_conflict_target_decorations::DecoratableTarget;
use crate::query_builder::{AsChangeset, InsertStatement, UndecoratedInsertRecord};
use crate::query_source::QuerySource;
use crate::sql_types::BoolOrNullableBool;

impl<T, U, Op, Ret> InsertStatement<T, U, Op, Ret>
where
    U: UndecoratedInsertRecord<T> + IntoConflictValueClause,
{
    /// Adds `ON CONFLICT DO NOTHING` to the insert statement, without
    /// specifying any columns or constraints to restrict the conflict to.
    ///
    /// # Examples
    ///
    /// ### Single Record
    ///
    /// ```rust
    /// # include!("on_conflict_docs_setup.rs");
    /// #
    /// # #[cfg(any(feature = "sqlite", feature = "postgres"))]
    /// # fn main() {
    /// #     use self::users::dsl::*;
    /// #     let mut conn = establish_connection();
    /// #     #[cfg(feature = "postgres")]
    /// #     conn.execute("TRUNCATE TABLE users").unwrap();
    /// #     #[cfg(feature = "sqlite")]
    /// #     conn.execute("DELETE FROM users").unwrap();
    /// let user = User { id: 1, name: "Sean" };
    ///
    /// let inserted_row_count = diesel::insert_into(users)
    ///     .values(&user)
    ///     .on_conflict_do_nothing()
    ///     .execute(&mut conn);
    /// assert_eq!(Ok(1), inserted_row_count);
    ///
    /// let inserted_row_count = diesel::insert_into(users)
    ///     .values(&user)
    ///     .on_conflict_do_nothing()
    ///     .execute(&mut conn);
    /// assert_eq!(Ok(0), inserted_row_count);
    /// # }
    /// # #[cfg(feature = "mysql")]
    /// # fn main() {}
    /// ```
    ///
    /// ### Vec of Records
    ///
    /// ```rust
    /// # include!("on_conflict_docs_setup.rs");
    /// #
    /// # #[cfg(any(feature = "sqlite", feature = "postgres"))]
    /// # fn main() {
    /// #     use self::users::dsl::*;
    /// #     let mut conn = establish_connection();
    /// #     #[cfg(feature = "postgres")]
    /// #     conn.execute("TRUNCATE TABLE users").unwrap();
    /// # #[cfg(feature = "postgres")]
    /// let user = User { id: 1, name: "Sean" };
    ///
    /// # #[cfg(feature = "postgres")]
    /// let inserted_row_count = diesel::insert_into(users)
    ///     .values(&vec![user, user])
    ///     .on_conflict_do_nothing()
    ///     .execute(&mut conn);
    /// # #[cfg(feature = "postgres")]
    /// assert_eq!(Ok(1), inserted_row_count);
    /// # }
    /// # #[cfg(feature = "mysql")]
    /// # fn main() {}
    /// ```
    pub fn on_conflict_do_nothing(
        self,
    ) -> InsertStatement<T, OnConflictValues<U::ValueClause, NoConflictTarget, DoNothing>, Op, Ret>
    {
        self.replace_values(|values| OnConflictValues::do_nothing(values.into_value_clause()))
    }

    /// Adds an `ON CONFLICT` to the insert statement, if a conflict occurs
    /// for the given unique constraint.
    ///
    /// `Target` can be one of:
    ///
    /// - A column
    /// - A tuple of columns
    /// - [`on_constraint("constraint_name")`][`on_constraint`]
    ///
    /// # Examples
    ///
    /// ### Specifying a column as the target
    ///
    /// ```rust
    /// # include!("on_conflict_docs_setup.rs");
    /// #
    /// # #[cfg(any(feature = "sqlite", feature = "postgres"))]
    /// # fn main() {
    /// #     use self::users::dsl::*;
    /// #     let mut conn = establish_connection();
    /// #     #[cfg(feature = "postgres")]
    /// #     conn.execute("TRUNCATE TABLE users").unwrap();
    /// #     #[cfg(feature = "sqlite")]
    /// #     conn.execute("DELETE FROM users").unwrap();
    /// conn.execute("CREATE UNIQUE INDEX users_name ON users (name)").unwrap();
    /// let user = User { id: 1, name: "Sean" };
    /// let same_name_different_id = User { id: 2, name: "Sean" };
    /// let same_id_different_name = User { id: 1, name: "Pascal" };
    ///
    /// assert_eq!(Ok(1), diesel::insert_into(users).values(&user).execute(&mut conn));
    ///
    /// let inserted_row_count = diesel::insert_into(users)
    ///     .values(&same_name_different_id)
    ///     .on_conflict(name)
    ///     .do_nothing()
    ///     .execute(&mut conn);
    /// assert_eq!(Ok(0), inserted_row_count);
    ///
    /// let pk_conflict_result = diesel::insert_into(users)
    ///     .values(&same_id_different_name)
    ///     .on_conflict(name)
    ///     .do_nothing()
    ///     .execute(&mut conn);
    /// assert!(pk_conflict_result.is_err());
    /// # }
    /// # #[cfg(feature = "mysql")]
    /// # fn main() {}
    /// ```
    ///
    /// ### Specifying multiple columns as the target
    ///
    /// ```rust
    /// # include!("../doctest_setup.rs");
    /// #
    /// # table! {
    /// #     users {
    /// #         id -> Integer,
    /// #         name -> VarChar,
    /// #         hair_color -> VarChar,
    /// #     }
    /// # }
    /// #
    /// # #[derive(Clone, Copy, Insertable)]
    /// # #[table_name="users"]
    /// # struct User<'a> {
    /// #     id: i32,
    /// #     name: &'a str,
    /// #     hair_color: &'a str,
    /// # }
    /// #
    /// # #[cfg(any(feature = "sqlite", feature = "postgres"))]
    /// # fn main() {
    /// #     use self::users::dsl::*;
    /// use diesel::upsert::*;
    ///
    /// #     let mut conn = establish_connection();
    /// #     conn.execute("DROP TABLE users").unwrap();
    /// #     conn.execute("CREATE TABLE users (id SERIAL PRIMARY KEY, name TEXT, hair_color TEXT)").unwrap();
    /// conn.execute("CREATE UNIQUE INDEX users_name_hair_color ON users (name, hair_color)").unwrap();
    /// let user = User { id: 1, name: "Sean", hair_color: "black" };
    /// let same_name_different_hair_color = User { id: 2, name: "Sean", hair_color: "brown" };
    /// let same_name_same_hair_color = User { id: 3, name: "Sean", hair_color: "black" };
    ///
    /// assert_eq!(Ok(1), diesel::insert_into(users).values(&user).execute(&mut conn));
    ///
    /// let inserted_row_count = diesel::insert_into(users)
    ///     .values(&same_name_different_hair_color)
    ///     .on_conflict((name, hair_color))
    ///     .do_nothing()
    ///     .execute(&mut conn);
    /// assert_eq!(Ok(1), inserted_row_count);
    ///
    /// let inserted_row_count = diesel::insert_into(users)
    ///     .values(&same_name_same_hair_color)
    ///     .on_conflict((name, hair_color))
    ///     .do_nothing()
    ///     .execute(&mut conn);
    /// assert_eq!(Ok(0), inserted_row_count);
    /// # }
    /// # #[cfg(feature = "mysql")]
    /// # fn main() {}
    /// ```
    ///
    /// See the documentation for [`on_constraint`] and [`do_update`] for
    /// more examples.
    ///
    /// [`on_constraint`]: ../upsert/fn.on_constraint.html
    /// [`do_update`]: crate::upsert::IncompleteOnConflict::do_update()
    pub fn on_conflict<Target>(
        self,
        target: Target,
    ) -> IncompleteOnConflict<InsertStatement<T, U::ValueClause, Op, Ret>, ConflictTarget<Target>>
    where
        ConflictTarget<Target>: OnConflictTarget<T>,
    {
        IncompleteOnConflict {
            stmt: self.replace_values(IntoConflictValueClause::into_value_clause),
            target: ConflictTarget(target),
        }
    }
}

impl<Stmt, T, P> DecoratableTarget<P> for IncompleteOnConflict<Stmt, T>
where
    P: Expression,
    P::SqlType: BoolOrNullableBool,
    T: DecoratableTarget<P>,
{
    type FilterOutput = IncompleteOnConflict<Stmt, <T as DecoratableTarget<P>>::FilterOutput>;
    fn filter_target(self, predicate: P) -> Self::FilterOutput {
        IncompleteOnConflict {
            stmt: self.stmt,
            target: self.target.filter_target(predicate),
        }
    }
}

/// A partially constructed `ON CONFLICT` clause.
#[derive(Debug, Clone, Copy)]
pub struct IncompleteOnConflict<Stmt, Target> {
    stmt: Stmt,
    target: Target,
}

impl<T, U, Op, Ret, Target> IncompleteOnConflict<InsertStatement<T, U, Op, Ret>, Target> {
    /// Creates a query with `ON CONFLICT (target) DO NOTHING`
    ///
    /// If you want to do nothing when *any* constraint conflicts, use
    /// [`on_conflict_do_nothing`] instead. See [`on_conflict`] for usage
    /// examples.
    ///
    /// [`on_conflict_do_nothing`]: crate::query_builder::InsertStatement::on_conflict_do_nothing()
    /// [`on_conflict`]: crate::query_builder::InsertStatement::on_conflict()
    pub fn do_nothing(self) -> InsertStatement<T, OnConflictValues<U, Target, DoNothing>, Op, Ret> {
        let target = self.target;
        self.stmt
            .replace_values(|values| OnConflictValues::new(values, target, DoNothing))
    }
}

impl<Stmt, Target> IncompleteOnConflict<Stmt, Target> {
    /// Used to create a query in the form `ON CONFLICT (...) DO UPDATE ...`
    ///
    /// Call `.set` on the result of this function with the changes you want to
    /// apply. The argument to `set` can be anything that implements `AsChangeset`
    /// (e.g. anything you could pass to `set` on a normal update statement).
    ///
    /// Note: When inserting more than one row at a time, this query can still fail
    /// if the rows being inserted conflict with each other.
    ///
    /// # Examples
    ///
    /// ## Set specific value on conflict
    ///
    /// ```rust
    /// # include!("on_conflict_docs_setup.rs");
    /// #
    /// # #[cfg(any(feature = "sqlite", feature = "postgres"))]
    /// # fn main() {
    /// #     use self::users::dsl::*;
    /// #     let mut conn = establish_connection();
    /// #     #[cfg(feature = "postgres")]
    /// #     conn.execute("TRUNCATE TABLE users").unwrap();
    /// #     #[cfg(feature = "sqlite")]
    /// #     conn.execute("DELETE FROM users").unwrap();
    /// let user = User { id: 1, name: "Pascal" };
    /// let user2 = User { id: 1, name: "Sean" };
    ///
    /// assert_eq!(Ok(1), diesel::insert_into(users).values(&user).execute(&mut conn));
    ///
    /// let insert_count = diesel::insert_into(users)
    ///     .values(&user2)
    ///     .on_conflict(id)
    ///     .do_update()
    ///     .set(name.eq("I DONT KNOW ANYMORE"))
    ///     .execute(&mut conn);
    /// assert_eq!(Ok(1), insert_count);
    ///
    /// let users_in_db = users.load(&mut conn);
    /// assert_eq!(Ok(vec![(1, "I DONT KNOW ANYMORE".to_string())]), users_in_db);
    /// # }
    /// # #[cfg(feature = "mysql")]
    /// # fn main() {}
    /// ```
    ///
    /// ## Set `AsChangeset` struct on conflict
    ///
    /// ```rust
    /// # include!("on_conflict_docs_setup.rs");
    /// #
    /// # #[cfg(any(feature = "sqlite", feature = "postgres"))]
    /// # fn main() {
    /// #     use self::users::dsl::*;
    /// #     let mut conn = establish_connection();
    /// #     #[cfg(feature = "postgres")]
    /// #     conn.execute("TRUNCATE TABLE users").unwrap();
    /// #     #[cfg(feature = "sqlite")]
    /// #     conn.execute("DELETE FROM users").unwrap();
    /// let user = User { id: 1, name: "Pascal" };
    /// let user2 = User { id: 1, name: "Sean" };
    ///
    /// assert_eq!(Ok(1), diesel::insert_into(users).values(&user).execute(&mut conn));
    ///
    /// let insert_count = diesel::insert_into(users)
    ///     .values(&user2)
    ///     .on_conflict(id)
    ///     .do_update()
    ///     .set(&user2)
    ///     .execute(&mut conn);
    /// assert_eq!(Ok(1), insert_count);
    ///
    /// let users_in_db = users.load(&mut conn);
    /// assert_eq!(Ok(vec![(1, "Sean".to_string())]), users_in_db);
    /// # }
    /// # #[cfg(feature = "mysql")]
    /// # fn main() {}
    /// ```
    ///
    /// ## Use `excluded` to get the rejected value
    ///
    /// ```rust
    /// # include!("on_conflict_docs_setup.rs");
    /// #
    /// # #[cfg(any(feature = "sqlite", feature = "postgres"))]
    /// # fn main() {
    /// #     use self::users::dsl::*;
    /// use diesel::upsert::excluded;
    ///
    /// #     let mut conn = establish_connection();
    /// #     #[cfg(feature = "postgres")]
    /// #     conn.execute("TRUNCATE TABLE users").unwrap();
    /// let user = User { id: 1, name: "Pascal" };
    /// let user2 = User { id: 1, name: "Sean" };
    /// let user3 = User { id: 2, name: "Tess" };
    ///
    /// # #[cfg(feature = "postgres")]
    /// assert_eq!(Ok(1), diesel::insert_into(users).values(&user).execute(&mut conn));
    ///
    /// #[cfg(feature = "postgres")]
    /// let insert_count = diesel::insert_into(users)
    ///     .values(&vec![user2, user3])
    ///     .on_conflict(id)
    ///     .do_update()
    ///     .set(name.eq(excluded(name)))
    ///     .execute(&mut conn);
    /// # #[cfg(feature = "postgres")]
    /// assert_eq!(Ok(2), insert_count);
    ///
    /// # #[cfg(feature = "postgres")]
    /// let users_in_db = users.load(&mut conn);
    /// # #[cfg(feature = "postgres")]
    /// assert_eq!(Ok(vec![(1, "Sean".to_string()), (2, "Tess".to_string())]), users_in_db);
    /// # }
    /// # #[cfg(feature = "mysql")]
    /// # fn main() {}
    /// ```
    pub fn do_update(self) -> IncompleteDoUpdate<Stmt, Target> {
        IncompleteDoUpdate {
            stmt: self.stmt,
            target: self.target,
        }
    }
}

/// A partially constructed `ON CONFLICT DO UPDATE` clause.
#[derive(Debug, Clone, Copy)]
pub struct IncompleteDoUpdate<Stmt, Target> {
    stmt: Stmt,
    target: Target,
}

impl<T, U, Op, Ret, Target> IncompleteDoUpdate<InsertStatement<T, U, Op, Ret>, Target> {
    /// See [`do_update`] for usage examples.
    ///
    /// [`do_update`]: IncompleteOnConflict::do_update()
    pub fn set<Changes>(
        self,
        changes: Changes,
    ) -> InsertStatement<T, OnConflictValues<U, Target, DoUpdate<Changes::Changeset>>, Op, Ret>
    where
        T: QuerySource,
        Changes: AsChangeset<Target = T>,
    {
        let target = self.target;
        self.stmt.replace_values(|values| {
            OnConflictValues::new(values, target, DoUpdate::new(changes.as_changeset()))
        })
    }
}
