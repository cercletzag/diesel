extern crate libsqlite3_sys as ffi;

use super::bind_collector::{SqliteBindCollector, SqliteBindValue};
use super::raw::RawConnection;
use super::sqlite_value::OwnedSqliteValue;
use crate::connection::{MaybeCached, PrepareForCache};
use crate::query_builder::{QueryFragment, QueryId};
use crate::result::Error::DatabaseError;
use crate::result::*;
use crate::sqlite::{Sqlite, SqliteType};
use crate::util::OnceCell;
use std::ffi::{CStr, CString};
use std::io::{stderr, Write};
use std::mem::ManuallyDrop;
use std::os::raw as libc;
use std::ptr::{self, NonNull};

#[allow(missing_debug_implementations)]
pub(in crate::sqlite) struct Statement {
    inner_statement: NonNull<ffi::sqlite3_stmt>,
}

impl Statement {
    pub fn prepare(
        raw_connection: &RawConnection,
        sql: &str,
        is_cached: PrepareForCache,
    ) -> QueryResult<Self> {
        let mut stmt = ptr::null_mut();
        let mut unused_portion = ptr::null();
        let prepare_result = unsafe {
            ffi::sqlite3_prepare_v3(
                raw_connection.internal_connection.as_ptr(),
                CString::new(sql)?.as_ptr(),
                sql.len() as libc::c_int,
                if matches!(is_cached, PrepareForCache::Yes) {
                    ffi::SQLITE_PREPARE_PERSISTENT as u32
                } else {
                    0
                },
                &mut stmt,
                &mut unused_portion,
            )
        };

        ensure_sqlite_ok(prepare_result, raw_connection.internal_connection.as_ptr()).map(|_| {
            Statement {
                inner_statement: unsafe { NonNull::new_unchecked(stmt) },
            }
        })
    }

    // The caller of this function has to ensure that:
    // * Any buffer provided as `SqliteBindValue::BorrowedBinary`, `SqliteBindValue::Binary`
    // `SqliteBindValue::String` or `SqliteBindValue::BorrowedString` is valid
    // till either a new value is bound to the same paramter or the underlying
    // prepared statement is dropped.
    unsafe fn bind(
        &mut self,
        tpe: SqliteType,
        value: &SqliteBindValue<'_>,
        bind_index: i32,
    ) -> QueryResult<()> {
        let result = match (tpe, value) {
            (_, SqliteBindValue::Null) => {
                ffi::sqlite3_bind_null(self.inner_statement.as_ptr(), bind_index)
            }
            (SqliteType::Binary, SqliteBindValue::BorrowedBinary(bytes)) => ffi::sqlite3_bind_blob(
                self.inner_statement.as_ptr(),
                bind_index,
                bytes.as_ptr() as *const libc::c_void,
                bytes.len() as libc::c_int,
                ffi::SQLITE_STATIC(),
            ),
            (SqliteType::Binary, SqliteBindValue::Binary(bytes)) => ffi::sqlite3_bind_blob(
                self.inner_statement.as_ptr(),
                bind_index,
                bytes.as_ptr() as *const libc::c_void,
                bytes.len() as libc::c_int,
                ffi::SQLITE_STATIC(),
            ),
            (SqliteType::Text, SqliteBindValue::BorrowedString(bytes)) => ffi::sqlite3_bind_text(
                self.inner_statement.as_ptr(),
                bind_index,
                bytes.as_ptr() as *const libc::c_char,
                bytes.len() as libc::c_int,
                ffi::SQLITE_STATIC(),
            ),
            (SqliteType::Text, SqliteBindValue::String(bytes)) => ffi::sqlite3_bind_text(
                self.inner_statement.as_ptr(),
                bind_index,
                bytes.as_ptr() as *const libc::c_char,
                bytes.len() as libc::c_int,
                ffi::SQLITE_STATIC(),
            ),
            (SqliteType::Double, SqliteBindValue::F64(value)) => ffi::sqlite3_bind_double(
                self.inner_statement.as_ptr(),
                bind_index,
                *value as libc::c_double,
            ),
            (SqliteType::Integer, SqliteBindValue::I32(value)) => {
                ffi::sqlite3_bind_int(self.inner_statement.as_ptr(), bind_index, *value)
            }
            (SqliteType::Long, SqliteBindValue::I64(value)) => {
                ffi::sqlite3_bind_int64(self.inner_statement.as_ptr(), bind_index, *value)
            }
            (t, b) => {
                return Err(Error::SerializationError(
                    format!("Type missmatch: Expected {:?}, got {}", t, b).into(),
                ))
            }
        };
        ensure_sqlite_ok(result, self.raw_connection())
    }

    fn reset(&mut self) {
        unsafe { ffi::sqlite3_reset(self.inner_statement.as_ptr()) };
    }

    fn raw_connection(&self) -> *mut ffi::sqlite3 {
        unsafe { ffi::sqlite3_db_handle(self.inner_statement.as_ptr()) }
    }
}

pub(super) fn ensure_sqlite_ok(
    code: libc::c_int,
    raw_connection: *mut ffi::sqlite3,
) -> QueryResult<()> {
    if code == ffi::SQLITE_OK {
        Ok(())
    } else {
        Err(last_error(raw_connection))
    }
}

fn last_error(raw_connection: *mut ffi::sqlite3) -> Error {
    let error_message = last_error_message(raw_connection);
    let error_information = Box::new(error_message);
    let error_kind = match last_error_code(raw_connection) {
        ffi::SQLITE_CONSTRAINT_UNIQUE | ffi::SQLITE_CONSTRAINT_PRIMARYKEY => {
            DatabaseErrorKind::UniqueViolation
        }
        ffi::SQLITE_CONSTRAINT_FOREIGNKEY => DatabaseErrorKind::ForeignKeyViolation,
        ffi::SQLITE_CONSTRAINT_NOTNULL => DatabaseErrorKind::NotNullViolation,
        ffi::SQLITE_CONSTRAINT_CHECK => DatabaseErrorKind::CheckViolation,
        _ => DatabaseErrorKind::Unknown,
    };
    DatabaseError(error_kind, error_information)
}

fn last_error_message(conn: *mut ffi::sqlite3) -> String {
    let c_str = unsafe { CStr::from_ptr(ffi::sqlite3_errmsg(conn)) };
    c_str.to_string_lossy().into_owned()
}

fn last_error_code(conn: *mut ffi::sqlite3) -> libc::c_int {
    unsafe { ffi::sqlite3_extended_errcode(conn) }
}

impl Drop for Statement {
    fn drop(&mut self) {
        use std::thread::panicking;

        let raw_connection = self.raw_connection();
        let finalize_result = unsafe { ffi::sqlite3_finalize(self.inner_statement.as_ptr()) };
        if let Err(e) = ensure_sqlite_ok(finalize_result, raw_connection) {
            if panicking() {
                write!(
                    stderr(),
                    "Error finalizing SQLite prepared statement: {:?}",
                    e
                )
                .expect("Error writing to `stderr`");
            } else {
                panic!("Error finalizing SQLite prepared statement: {:?}", e);
            }
        }
    }
}

struct BoundStatement<'stmt, 'query> {
    statement: MaybeCached<'stmt, Statement>,
    // we need to store the query here to ensure noone does
    // drop it till the end ot the statement
    // We use a boxed queryfragment here just to erase the
    // generic type, we use ManuallyDrop to communicate
    // that this is a shared buffer
    query: ManuallyDrop<Box<dyn QueryFragment<Sqlite> + 'query>>,
    // we need to store any owned bind values speratly, as they are not
    // contained in the query itself. We use ManuallyDrop to
    // communicate that this is a shared buffer
    binds_to_free: ManuallyDrop<Vec<(i32, Option<SqliteBindValue<'static>>)>>,
}

impl<'stmt, 'query> BoundStatement<'stmt, 'query> {
    fn bind<T>(
        mut statement: MaybeCached<'stmt, Statement>,
        query: T,
    ) -> QueryResult<BoundStatement<'stmt, 'query>>
    where
        T: QueryFragment<Sqlite> + QueryId + 'query,
    {
        // Don't use a trait object here to prevent using a virtual function call
        // For sqlite this can introduce a measurable overhead
        let mut query = ManuallyDrop::new(Box::new(query));

        let mut bind_collector = SqliteBindCollector::new();
        query.collect_binds(&mut bind_collector, &mut ())?;
        let SqliteBindCollector { binds } = bind_collector;

        let binds_to_free = match Self::bind_buffers(binds, &mut statement) {
            Ok(value) => value,
            Err(e) => {
                unsafe {
                    // We return from this function afterwards and
                    // any buffer is already unbound by `bind_buffers`
                    // so it's safe to drop query now
                    ManuallyDrop::drop(&mut query);
                }
                return Err(e);
            }
        };

        Ok(Self {
            statement,
            binds_to_free,
            query: ManuallyDrop::new(
                // Cast to a trait object here, to erase the generic parameter T
                ManuallyDrop::into_inner(query) as Box<dyn QueryFragment<Sqlite> + 'query>,
            ),
        })
    }

    // This is a seperate function so that
    // not the whole construtor is generic over the query type T.
    // This hopefully prevents binary bloat.
    fn bind_buffers(
        binds: Vec<(SqliteBindValue<'_>, SqliteType)>,
        statement: &mut MaybeCached<'stmt, Statement>,
    ) -> QueryResult<ManuallyDrop<Vec<(i32, Option<SqliteBindValue<'static>>)>>> {
        let mut binds_to_free = ManuallyDrop::new(Vec::with_capacity(
            binds
                .iter()
                .filter(|&(b, _)| {
                    matches!(
                        b,
                        SqliteBindValue::BorrowedBinary(_)
                            | SqliteBindValue::BorrowedString(_)
                            | SqliteBindValue::String(_)
                            | SqliteBindValue::Binary(_)
                    )
                })
                .count(),
        ));
        for (bind_idx, (bind, tpe)) in (1..).zip(binds) {
            // It's safe to call bind here as:
            // * The type and value matches
            // * We ensure that corresponding buffers lives long enough below
            // * The statement is not used yet by `step` or anything else
            let res = unsafe { statement.bind(tpe, &bind, bind_idx) };

            if let Err(e) = res {
                Self::unbind_buffers(statement, &binds_to_free);
                unsafe {
                    // It's safe to drop binds_to_free here as
                    // we've already unbound the buffers
                    ManuallyDrop::drop(&mut binds_to_free);
                }
                return Err(e);
            }

            // We want to unbind the buffers later to ensure
            // that sqlite does not access uninitilized memory
            match bind {
                SqliteBindValue::BorrowedString(_) | SqliteBindValue::BorrowedBinary(_) => {
                    binds_to_free.push((bind_idx, None));
                }
                SqliteBindValue::Binary(b) => {
                    binds_to_free.push((bind_idx, Some(SqliteBindValue::Binary(b))));
                }
                SqliteBindValue::String(b) => {
                    binds_to_free.push((bind_idx, Some(SqliteBindValue::String(b))));
                }
                SqliteBindValue::I32(_)
                | SqliteBindValue::I64(_)
                | SqliteBindValue::F64(_)
                | SqliteBindValue::Null => {}
            }
        }
        Ok(binds_to_free)
    }

    fn unbind_buffers(
        stmt: &mut MaybeCached<'stmt, Statement>,
        binds_to_free: &[(i32, Option<SqliteBindValue<'static>>)],
    ) {
        for (idx, _buffer) in binds_to_free {
            unsafe {
                // It's always safe to bind null values, as there is no buffer that needs to outlife something
                stmt.bind(SqliteType::Text, &SqliteBindValue::Null, *idx)
                    .expect(
                        "Binding a null value should never fail. \
                             If you ever see this error message please open \
                             an issue at diesels issue tracker containing \
                             code how to trigger this message.",
                    );
            }
        }
    }
}

impl<'stmt, 'query> Drop for BoundStatement<'stmt, 'query> {
    fn drop(&mut self) {
        // First reset the statement, otherwise the bind calls
        // below will fails
        self.statement.reset();

        // Reset the binds that may point to memory that will be/needs to be freed
        Self::unbind_buffers(&mut self.statement, &self.binds_to_free);
        unsafe {
            // We unbound the corresponding buffers above, so it's fine to drop the
            // owned binds now
            ManuallyDrop::drop(&mut self.binds_to_free);
            // We've dropped everything that could reference the query
            // so it's safe to drop the query here
            ManuallyDrop::drop(&mut self.query);
        }
    }
}

#[allow(missing_debug_implementations)]
pub struct StatementUse<'stmt, 'query> {
    statement: BoundStatement<'stmt, 'query>,
    column_names: OnceCell<Vec<*const str>>,
    called_step_once: bool,
}

impl<'stmt, 'query> StatementUse<'stmt, 'query> {
    pub(super) fn bind<T>(
        statement: MaybeCached<'stmt, Statement>,
        query: T,
    ) -> QueryResult<StatementUse<'stmt, 'query>>
    where
        T: QueryFragment<Sqlite> + QueryId + 'query,
    {
        Ok(Self {
            statement: BoundStatement::bind(statement, query)?,
            column_names: OnceCell::new(),
            called_step_once: false,
        })
    }

    pub(in crate::sqlite::connection) fn run(self) -> QueryResult<()> {
        self.step().map(|_| ())
    }

    pub(in crate::sqlite::connection) fn step(self) -> QueryResult<Option<Self>> {
        let res = unsafe {
            match ffi::sqlite3_step(self.statement.statement.inner_statement.as_ptr()) {
                ffi::SQLITE_DONE => Ok(None),
                ffi::SQLITE_ROW => Ok(Some(())),
                _ => Err(last_error(self.statement.statement.raw_connection())),
            }
        }?;
        Ok(res.map(move |()| {
            if self.called_step_once {
                self
            } else {
                Self {
                    called_step_once: true,
                    column_names: OnceCell::new(),
                    ..self
                }
            }
        }))
    }

    // The returned string pointer is valid until either the prepared statement is
    // destroyed by sqlite3_finalize() or until the statement is automatically
    // reprepared by the first call to sqlite3_step() for a particular run or
    // until the next call to sqlite3_column_name() or sqlite3_column_name16()
    // on the same column.
    //
    // https://sqlite.org/c3ref/column_name.html
    //
    // Note: This function is marked as unsafe, as calling it can invalidate
    // other existing column name pointers on the same column. To prevent that,
    // it should maximally be called once per column at all.
    unsafe fn column_name(&self, idx: i32) -> *const str {
        let name = {
            let column_name =
                ffi::sqlite3_column_name(self.statement.statement.inner_statement.as_ptr(), idx);
            assert!(
                !column_name.is_null(),
                "The Sqlite documentation states that it only returns a \
                 null pointer here if we are in a OOM condition."
            );
            CStr::from_ptr(column_name)
        };
        name.to_str().expect(
            "The Sqlite documentation states that this is UTF8. \
             If you see this error message something has gone \
             horribliy wrong. Please open an issue at the \
             diesel repository.",
        ) as *const str
    }

    pub(super) fn column_count(&self) -> i32 {
        unsafe { ffi::sqlite3_column_count(self.statement.statement.inner_statement.as_ptr()) }
    }

    pub(super) fn index_for_column_name(&mut self, field_name: &str) -> Option<usize> {
        (0..self.column_count())
            .find(|idx| self.field_name(*idx) == Some(field_name))
            .map(|v| v as usize)
    }

    pub(super) fn field_name(&self, idx: i32) -> Option<&str> {
        let column_names = self.column_names.get_or_init(|| {
            let count = self.column_count();
            (0..count)
                .map(|idx| unsafe {
                    // By initializing the whole vec at once we ensure that
                    // we really call this only once.
                    self.column_name(idx)
                })
                .collect()
        });

        column_names
            .get(idx as usize)
            .and_then(|c| unsafe { c.as_ref() })
    }

    pub(super) fn copy_value(&self, idx: i32) -> Option<OwnedSqliteValue> {
        OwnedSqliteValue::copy_from_ptr(self.column_value(idx)?)
    }

    pub(super) fn column_value(&self, idx: i32) -> Option<NonNull<ffi::sqlite3_value>> {
        let ptr = unsafe {
            ffi::sqlite3_column_value(self.statement.statement.inner_statement.as_ptr(), idx)
        };
        NonNull::new(ptr)
    }
}
