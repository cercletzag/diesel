use self::models::*;
use diesel::prelude::*;
use diesel_demo_step_3_mysql::*;

fn main() {
    use self::schema::posts::dsl::*;

    let mut connection = establish_connection();
    let results = posts
        .filter(published.eq(true))
        .limit(5)
        .load::<Post>(&mut connection)
        .expect("Error loading posts");

    println!("Displaying {} posts", results.len());
    for post in results {
        println!("{}", post.title);
        println!("-----------\n");
        println!("{}", post.body);
    }
}
