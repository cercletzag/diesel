use schema::*;
use diesel::*;

#[test]
fn one_to_many_returns_query_source_for_association() {
    let connection = connection_with_sean_and_tess_in_users_table();

    let sean = find_user_by_name("Sean", &connection);
    let tess = find_user_by_name("Tess", &connection);
    let seans_posts: Vec<Post> =  insert(&vec![
        sean.new_post("Hello", None), sean.new_post("World", None)
        ]).into(posts::table)
        .get_results(&connection)
        .unwrap().collect();
    let tess_posts: Vec<Post> = insert(&vec![
        tess.new_post("Hello 2", None), tess.new_post("World 2", None),
        ]).into(posts::table)
        .get_results(&connection)
        .unwrap().collect();

    let found_posts: Vec<_> = Post::belonging_to(&sean).load(&connection).unwrap().collect();
    assert_eq!(seans_posts, found_posts);

    let found_posts: Vec<_> = Post::belonging_to(&tess).load(&connection).unwrap().collect();
    assert_eq!(tess_posts, found_posts);
}

#[test]
fn association_where_struct_name_doesnt_match_table_name() {
    let connection = connection_with_sean_and_tess_in_users_table();

    let sean = find_user_by_name("Sean", &connection);
    let post: Post = insert(&sean.new_post("Hello", None)).into(posts::table)
        .get_result(&connection).unwrap();
    insert(&NewComment(post.id, "comment")).into(comments::table)
        .execute(&connection).unwrap();

    let comment_text = SpecialComment::belonging_to(&post).select(comments::text)
        .first::<String>(&connection);
    assert_eq!(Ok("comment".into()), comment_text);
}
