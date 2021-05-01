extern crate diesel;

use diesel::*;

table! {
    users {
        id -> Integer,
        name -> Text,
    }
}

table! {
    posts {
        id -> Integer,
        title -> Text,
        user_id -> Integer,
    }
}

joinable!(posts -> users(user_id));
allow_tables_to_appear_in_same_query!(users, posts);

fn main() {
    let conn = PgConnection::establish("").unwrap();

    users::table.select(users::name).having(users::id.gt(1)).load(&conn);

    users::table.into_boxed().having(users::id.gt(1)).load(&conn);

    users::table.select(users::name).group_by(users::id).having(posts::id.eq(42)).load(&conn);
}
