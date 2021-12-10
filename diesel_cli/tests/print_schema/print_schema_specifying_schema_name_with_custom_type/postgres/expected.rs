// @generated automatically by Diesel CLI.

pub mod custom_schema {
    /// A module containing custom SQL type definitions
    ///
    /// (Automatically generated by Diesel.)
    pub mod sql_types {
        /// The `custom_schema.my_enum` SQL type
        ///
        /// (Automatically generated by Diesel.)
        #[derive(diesel::sql_types::SqlType)]
        #[diesel(postgres_type(name = "my_enum", schema = "custom_schema"))]
        pub struct MyEnum;
    }

    diesel::table! {
        use diesel::sql_types::*;
        use super::sql_types::MyEnum;

        /// Representation of the `custom_schema.in_schema` table.
        ///
        /// (Automatically generated by Diesel.)
        custom_schema.in_schema (id) {
            /// The `id` column of the `custom_schema.in_schema` table.
            ///
            /// Its SQL type is `Int4`.
            ///
            /// (Automatically generated by Diesel.)
            id -> Int4,
            /// The `custom_type` column of the `custom_schema.in_schema` table.
            ///
            /// Its SQL type is `Nullable<MyEnum>`.
            ///
            /// (Automatically generated by Diesel.)
            custom_type -> Nullable<MyEnum>,
        }
    }
}
