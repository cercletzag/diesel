use schema::*;
use yaqb::*;

#[test]
fn filter_by_inequality() {
    use schema::users::dsl::*;

    let connection = connection_with_sean_and_tess_in_users_table();
    let sean = User::new(1, "Sean");
    let tess = User::new(2, "Tess");

    assert_eq!(vec![tess.clone()], connection.query_all(users.filter(name.ne("Sean"))).as_vec());
    assert_eq!(vec![sean.clone()], connection.query_all(users.filter(name.ne("Tess"))).as_vec());
    assert_eq!(vec![sean, tess], connection.query_all(users.filter(name.ne("Jim"))).as_vec());
}

#[test]
fn filter_by_gt() {
    use schema::users::dsl::*;

    let connection = connection_with_3_users();
    let tess = User::new(2, "Tess");
    let jim = User::new(3, "Jim");

    assert_eq!(vec![tess, jim.clone()],
        connection.query_all(users.filter(id.gt(1))).as_vec());
    assert_eq!(vec![jim], connection.query_all(users.filter(id.gt(2))).as_vec());
}

#[test]
fn filter_by_ge() {
    use schema::users::dsl::*;

    let connection = connection_with_3_users();
    let tess = User::new(2, "Tess");
    let jim = User::new(3, "Jim");

    assert_eq!(vec![tess, jim.clone()],
        connection.query_all(users.filter(id.ge(2))).as_vec());
    assert_eq!(vec![jim], connection.query_all(users.filter(id.ge(3))).as_vec());
}

#[test]
fn filter_by_lt() {
    use schema::users::dsl::*;

    let connection = connection_with_3_users();
    let sean = User::new(1, "Sean");
    let tess = User::new(2, "Tess");

    assert_eq!(vec![sean.clone(), tess],
        connection.query_all(users.filter(id.lt(3))).as_vec());
    assert_eq!(vec![sean], connection.query_all(users.filter(id.lt(2))).as_vec());
}

#[test]
fn filter_by_le() {
    use schema::users::dsl::*;

    let connection = connection_with_3_users();
    let sean = User::new(1, "Sean");
    let tess = User::new(2, "Tess");

    assert_eq!(vec![sean.clone(), tess],
        connection.query_all(users.filter(id.le(2))).as_vec());
    assert_eq!(vec![sean], connection.query_all(users.filter(id.le(1))).as_vec());
}

#[test]
fn filter_by_between() {
    use schema::users::dsl::*;

    let connection = connection_with_3_users();
    let sean = User::new(1, "Sean");
    let tess = User::new(2, "Tess");
    let jim = User::new(3, "Jim");

    assert_eq!(vec![sean, tess.clone(), jim.clone()],
        connection.query_all(users.filter(id.between(1..3))).as_vec());
    assert_eq!(vec![tess, jim],
        connection.query_all(users.filter(id.between(2..3))).as_vec());
}

#[test]
fn filter_by_like() {
    use schema::users::dsl::*;

    let connection = connection();
    setup_users_table(&connection);
    let data = vec![
        NewUser::new("Sean Griffin", None),
        NewUser::new("Tess Griffin", None),
        NewUser::new("Jim", None),
    ];
    connection.insert_without_return(&users, &data).unwrap();

    let sean = User::new(1, "Sean Griffin");
    let tess = User::new(2, "Tess Griffin");
    let jim = User::new(3, "Jim");
    assert_eq!(vec![sean, tess],
        connection.query_all(users.filter(name.like("%Griffin"))).as_vec());
    assert_eq!(vec![jim],
        connection.query_all(users.filter(name.not_like("%Griffin"))).as_vec());
}

#[test]
fn filter_by_any() {
    use schema::users::dsl::*;
    use yaqb::expression::dsl::any;

    let connection = connection_with_3_users();
    let sean = User::new(1, "Sean");
    let tess = User::new(2, "Tess");
    let jim = User::new(3, "Jim");

    let owned_names = vec!["Sean", "Tess"];
    let borrowed_names: &[&str] = &["Sean", "Jim"];
    assert_eq!(vec![sean.clone(), tess],
        connection.query_all(users.filter(name.eq(any(owned_names)))).as_vec());
    assert_eq!(vec![sean, jim],
        connection.query_all(users.filter(name.eq(any(borrowed_names)))).as_vec());
}

trait TestResultHelpers<U> {
    fn as_vec(self) -> Vec<U>;
}

impl<ST, U> TestResultHelpers<U> for Result<Cursor<ST, U>> where
    Cursor<ST, U>: Iterator<Item=U>,
{
    fn as_vec(self) -> Vec<U> {
        self.unwrap().collect()
    }
}

fn connection_with_3_users() -> Connection {
    let connection = connection();
    setup_users_table(&connection);
    let data = vec![
        NewUser::new("Sean", None),
        NewUser::new("Tess", None),
        NewUser::new("Jim", None),
    ];
    connection.insert_without_return(&users::table, &data).unwrap();
    connection
}
