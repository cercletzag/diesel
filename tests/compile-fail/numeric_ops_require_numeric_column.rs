#[macro_use]
extern crate yaqb;

use yaqb::*;

table! {
    users {
        id -> Serial,
        name -> VarChar,
    }
}

fn main() {
    use self::users::dsl::*;

    let _ = users.select(name + name);
    //~^ ERROR binary operation `+` cannot be applied to type `users::columns::name`
}
