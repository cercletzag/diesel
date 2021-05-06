use crate::schema::*;
use diesel::result::Error;
use diesel::*;

#[test]
#[cfg(not(feature = "sqlite"))] // FIXME: This test is only valid when operating on a file and not :memory:
fn transaction_executes_fn_in_a_sql_transaction() {
    const TEST_NAME: &'static str = "transaction_executes_fn_in_a_sql_transaction";
    let mut conn1 = connection_without_transaction();
    let mut conn2 = connection_without_transaction();
    setup_test_table(&mut conn1, TEST_NAME);

    fn get_count(conn: &mut TestConnection) -> i64 {
        count_test_table(conn, TEST_NAME)
    }

    conn1
        .transaction::<_, Error, _>(|conn1| {
            assert_eq!(0, get_count(conn1));
            assert_eq!(0, get_count(&mut conn2));
            conn1.execute(&format!("INSERT INTO {} DEFAULT VALUES", TEST_NAME))?;
            assert_eq!(1, get_count(conn1));
            assert_eq!(0, get_count(&mut conn2));
            Ok(())
        })
        .unwrap();

    assert_eq!(1, get_count(&mut conn1));
    assert_eq!(1, get_count(&mut conn2));

    drop_test_table(&mut conn1, TEST_NAME);
}

#[test]
fn transaction_returns_the_returned_value() {
    let mut conn1 = connection_without_transaction();

    assert_eq!(Ok(1), conn1.transaction::<_, Error, _>(|_| Ok(1)));
}

#[test]
fn transaction_is_rolled_back_when_returned_an_error() {
    let mut connection = connection_without_transaction();
    let test_name = "transaction_is_rolled_back_when_returned_an_error";
    setup_test_table(&mut connection, test_name);

    let _ = connection.transaction::<(), _, _>(|connection| {
        connection
            .execute(&format!("INSERT INTO {} DEFAULT VALUES", test_name))
            .unwrap();
        Err(Error::RollbackTransaction)
    });
    assert_eq!(0, count_test_table(&mut connection, test_name));

    drop_test_table(&mut connection, test_name);
}

#[test]
fn transactions_can_be_nested() {
    let mut connection = connection_without_transaction();
    const TEST_NAME: &'static str = "transactions_can_be_nested";
    setup_test_table(&mut connection, TEST_NAME);
    fn get_count(connection: &mut TestConnection) -> i64 {
        count_test_table(connection, TEST_NAME)
    }

    let _ = connection.transaction::<(), _, _>(|connection| {
        connection
            .execute(&format!("INSERT INTO {} DEFAULT VALUES", TEST_NAME))
            .unwrap();
        assert_eq!(1, get_count(connection));
        let _ = connection.transaction::<(), _, _>(|connection| {
            connection
                .execute(&format!("INSERT INTO {} DEFAULT VALUES", TEST_NAME))
                .unwrap();
            assert_eq!(2, get_count(connection));
            Err(Error::RollbackTransaction)
        });
        assert_eq!(1, get_count(connection));
        let _ = connection.transaction::<(), Error, _>(|connection| {
            connection
                .execute(&format!("INSERT INTO {} DEFAULT VALUES", TEST_NAME))
                .unwrap();
            assert_eq!(2, get_count(connection));
            Ok(())
        });
        assert_eq!(2, get_count(connection));
        Err(Error::RollbackTransaction)
    });
    assert_eq!(0, get_count(&mut connection));

    drop_test_table(&mut connection, TEST_NAME);
}

#[test]
fn test_transaction_always_rolls_back() {
    let mut connection = connection_without_transaction();
    let test_name = "test_transaction_always_rolls_back";
    setup_test_table(&mut connection, test_name);

    let result = connection.test_transaction::<_, Error, _>(|connection| {
        connection.execute(&format!("INSERT INTO {} DEFAULT VALUES", test_name))?;
        assert_eq!(1, count_test_table(connection, test_name));
        Ok("success")
    });
    assert_eq!(0, count_test_table(&mut connection, test_name));
    assert_eq!("success", result);

    drop_test_table(&mut connection, test_name);
}

#[test]
#[should_panic(expected = "Transaction did not succeed")]
fn test_transaction_panics_on_error() {
    let mut connection = connection_without_transaction();
    connection.test_transaction::<(), _, _>(|_| Err(()));
}

fn setup_test_table(connection: &mut TestConnection, table_name: &str) {
    use crate::schema_dsl::*;
    create_table(table_name, (integer("id").primary_key().auto_increment(),))
        .execute(connection)
        .unwrap();
}

fn drop_test_table(connection: &mut TestConnection, table_name: &str) {
    connection
        .execute(&format!("DROP TABLE {}", table_name))
        .unwrap();
}

fn count_test_table(connection: &mut TestConnection, table_name: &str) -> i64 {
    use diesel::dsl::sql;
    select(sql::<sql_types::BigInt>(&format!(
        "COUNT(*) FROM {}",
        table_name
    )))
    .first(connection)
    .unwrap()
}

#[test]
#[cfg(feature = "postgres")]
fn regression_test_for_2123() {
    let mut conn = connection_without_transaction();
    // fail once
    let ret = conn.transaction(|conn| {
        let _ = conn.transaction(|conn| {
            // handling error
            match conn.execute("SELECT foo") {
                // do nothing
                Ok(_) => unreachable!("This query should fail"),
                // ignore the error
                Err(e) => eprintln!("error occurred: {}", e),
            };
            Ok::<_, Error>(())
        });

        conn.transaction(|conn| {
            let ret = conn.execute("SELECT 1");
            assert_eq!(Ok(1), ret);
            Ok::<_, Error>(())
        })
    });
    println!("{:?}", ret);
    // other transaction
    let ret = conn
        .build_transaction()
        .serializable()
        .run(|conn| conn.execute("SELECT 1"));
    // must be Ok(1), but get Err(AlreadyInTransaction)
    println!("{:?}", ret);
    assert_eq!(Ok(1), ret);
}
