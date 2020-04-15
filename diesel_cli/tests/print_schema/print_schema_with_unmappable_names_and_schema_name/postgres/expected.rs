pub mod custom_schema {
    table! {
        /// Representation of the `custom_schema.self` table.
        ///
        /// (Automatically generated by Diesel.)
        #[sql_name = "custom_schema.self"]
        custom_schema.self_ (id) {
            /// The `id` column of the `custom_schema.self` table.
            ///
            /// Its SQL type is `Int4`.
            ///
            /// (Automatically generated by Diesel.)
            id -> Int4,
        }
    }

    table! {
        /// Representation of the `custom_schema.user-has::complex>>>role` table.
        ///
        /// (Automatically generated by Diesel.)
        #[sql_name = "custom_schema.user-has::complex>>>role"]
        custom_schema.user_has_complex_role (id) {
            /// The `user` column of the `custom_schema.user-has::complex>>>role` table.
            ///
            /// Its SQL type is `Int4`.
            ///
            /// (Automatically generated by Diesel.)
            user -> Int4,
            /// The `role` column of the `custom_schema.user-has::complex>>>role` table.
            ///
            /// Its SQL type is `Int4`.
            ///
            /// (Automatically generated by Diesel.)
            role -> Int4,
            /// The `id` column of the `custom_schema.user-has::complex>>>role` table.
            ///
            /// Its SQL type is `Int4`.
            ///
            /// (Automatically generated by Diesel.)
            id -> Int4,
            /// The `created at` column of the `custom_schema.user-has::complex>>>role` table.
            ///
            /// Its SQL type is `Timestamp`.
            ///
            /// (Automatically generated by Diesel.)
            #[sql_name = "created at"]
            created_at -> Timestamp,
            /// The `expiry date` column of the `custom_schema.user-has::complex>>>role` table.
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
}
