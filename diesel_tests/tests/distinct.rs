use super::schema::*;
use diesel::*;

#[test]
fn simple_distinct() {
    use crate::schema::users::dsl::*;

    let connection = &mut connection();
    diesel::sql_query("INSERT INTO users (name) VALUES ('Sean'), ('Sean'), ('Tess')")
        .execute(connection)
        .unwrap();

    let source = users.select(name).distinct().order(name);
    let expected_data = vec!["Sean".to_string(), "Tess".to_string()];
    let data: Vec<String> = source.load(connection).unwrap();

    assert_eq!(expected_data, data);
}

#[cfg(feature = "postgres")]
#[test]
fn distinct_on() {
    use crate::schema::users::dsl::*;

    let connection = &mut connection();
    diesel::sql_query(
            "INSERT INTO users (name, hair_color) VALUES ('Sean', 'black'), ('Sean', NULL), ('Tess', NULL), ('Tess', NULL)",
        ).execute(connection)
        .unwrap();

    let source = users
        .select((name, hair_color))
        .order(name)
        .distinct_on(name);
    let mut expected_data = vec![
        ("Sean".to_string(), Some("black".to_string())),
        ("Tess".to_string(), None),
    ];
    let data: Vec<_> = source.load(connection).unwrap();

    assert_eq!(expected_data, data);

    let source = users
        .select((name, hair_color))
        .order(name.asc())
        .distinct_on(name);
    let data: Vec<_> = source.load(connection).unwrap();

    assert_eq!(expected_data, data);

    let source = users
        .select((name, hair_color))
        .order(name.desc())
        .distinct_on(name);
    let data: Vec<_> = source.load(connection).unwrap();

    expected_data.reverse();
    assert_eq!(expected_data, data);
}

#[cfg(feature = "postgres")]
#[test]
fn distinct_on_select_by() {
    use crate::schema::users::dsl::*;

    let connection = &mut connection();
    diesel::sql_query(
            "INSERT INTO users (name, hair_color) VALUES ('Sean', 'black'), ('Sean', NULL), ('Tess', NULL), ('Tess', NULL)",
        ).execute(connection)
        .unwrap();

    let source = users
        .select(NewUser::as_select())
        .order(name)
        .distinct_on(name);
    let expected_data = vec![
        NewUser::new("Sean", Some("black")),
        NewUser::new("Tess", None),
    ];
    let data: Vec<_> = source.load(connection).unwrap();

    assert_eq!(expected_data, data);
}

#[cfg(feature = "postgres")]
#[test]
fn distinct_on_select_order_by_two_columns() {
    use crate::schema::users::dsl::*;

    let connection = &mut connection();
    diesel::sql_query(
            "INSERT INTO users (name, hair_color) VALUES ('Sean', 'black'), ('Sean', 'aqua'), ('Tess', 'bronze'), ('Tess', 'champagne')",
        ).execute(connection)
        .unwrap();

    let source = users
        .select((name, hair_color))
        .order((name, hair_color.desc()))
        .distinct_on(name);
    let expected_data = vec![
        NewUser::new("Sean", Some("black")),
        NewUser::new("Tess", Some("champagne")),
    ];
    let data: Vec<_> = source.load(connection).unwrap();

    assert_eq!(expected_data, data);
}
