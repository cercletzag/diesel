table! {
    /// Representation of the `self` table.
    ///
    /// (Automatically generated by Diesel.)
    #[sql_name = "self"]
    self_ (id) {
        /// The `id` column of the `self` table.
        ///
        /// Its SQL type is `Int4`.
        ///
        /// (Automatically generated by Diesel.)
        id -> Int4,
    }
}

table! {
    /// Representation of the `user-has::complex>>>role` table.
    ///
    /// (Automatically generated by Diesel.)
    #[sql_name = "user-has::complex>>>role"]
    user_has_complex_role (id) {
        /// The `user` column of the `user-has::complex>>>role` table.
        ///
        /// Its SQL type is `Int4`.
        ///
        /// (Automatically generated by Diesel.)
        user -> Int4,
        /// The `role` column of the `user-has::complex>>>role` table.
        ///
        /// Its SQL type is `Int4`.
        ///
        /// (Automatically generated by Diesel.)
        role -> Int4,
        /// The `id` column of the `user-has::complex>>>role` table.
        ///
        /// Its SQL type is `Int4`.
        ///
        /// (Automatically generated by Diesel.)
        id -> Int4,
        /// The `created at` column of the `user-has::complex>>>role` table.
        ///
        /// Its SQL type is `Timestamp`.
        ///
        /// (Automatically generated by Diesel.)
        #[sql_name = "created at"]
        created_at -> Timestamp,
        /// The `expiry date` column of the `user-has::complex>>>role` table.
        ///
        /// Its SQL type is `Nullable<Timestamp>`.
        ///
        /// (Automatically generated by Diesel.)
        #[sql_name = "expiry date"]
        expiry_date -> Nullable<Timestamp>,
    }
}

joinable!(user_has_complex_role -> self_ (user));

allow_tables_to_appear_in_same_query!(
    self_,
    user_has_complex_role,
);
