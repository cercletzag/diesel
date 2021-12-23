#[macro_use]
extern crate diesel;

table! {
    bar (id) {
        id -> Integer,
    }
}

table! {
    foo (bar_id) {
        bar_id -> Integer,
    }
}

#[derive(Identifiable)]
#[diesel(table_name = bar)]
struct Bar {
    id: i32,
}

#[derive(Identifiable)]
#[diesel(table_name = bar)]
struct Baz {
    id: i32,
}

#[derive(Associations)]
#[diesel(belongs_to)]
#[diesel(table_name = foo)]
struct Foo1 {
    bar_id: i32,
}

#[derive(Associations)]
#[diesel(belongs_to = "Bar")]
#[diesel(table_name = foo)]
struct Foo2 {
    bar_id: i32,
}

#[derive(Associations)]
#[diesel(belongs_to())]
#[diesel(table_name = foo)]
struct Foo3 {
    bar_id: i32,
}

#[derive(Associations)]
#[diesel(belongs_to(foreign_key = bar_id))]
#[diesel(table_name = foo)]
struct Foo4 {
    bar_id: i32,
}

#[derive(Associations)]
#[diesel(belongs_to(Bar = "bar_id"))]
#[diesel(table_name = foo)]
struct Foo5 {
    bar_id: i32,
}

#[derive(Associations)]
#[diesel(belongs_to(Bar, foreign_key))]
#[diesel(table_name = foo)]
struct Foo6 {
    bar_id: i32,
}

#[derive(Associations)]
#[diesel(belongs_to(Bar, foreign_key(bar_id)))]
#[diesel(table_name = foo)]
struct Foo7 {
    bar_id: i32,
}

#[derive(Associations)]
#[diesel(belongs_to(Bar, what))]
#[diesel(table_name = foo)]
struct Foo8 {
    bar_id: i32,
}

fn main() {}
