diesel::table! {
    /// Representation of the `abc` table.
    ///
    /// (Automatically generated by Diesel.)
    abc (id) {
        /// The `id` column of the `abc` table.
        ///
        /// Its SQL type is `Integer`.
        ///
        /// (Automatically generated by Diesel.)
        id -> Integer,
    }
}

diesel::table! {
    /// Representation of the `def` table.
    ///
    /// (Automatically generated by Diesel.)
    def (id) {
        /// The `id` column of the `def` table.
        ///
        /// Its SQL type is `Integer`.
        ///
        /// (Automatically generated by Diesel.)
        id -> Integer,
    }
}

diesel::table! {
    /// Representation of the `ghi` table.
    ///
    /// (Automatically generated by Diesel.)
    ghi (id) {
        /// The `id` column of the `ghi` table.
        ///
        /// Its SQL type is `Integer`.
        ///
        /// (Automatically generated by Diesel.)
        id -> Integer,
    }
}

diesel::allow_tables_to_appear_in_same_query!(
    abc,
    def,
    ghi,
);
