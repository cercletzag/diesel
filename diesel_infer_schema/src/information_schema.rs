use std::error::Error;

use diesel::*;
use diesel::backend::Backend;
use diesel::expression::NonAggregate;
use diesel::query_builder::{QueryId, QueryFragment};
use diesel::types::FromSql;
#[cfg(feature="postgres")]
use diesel::pg::Pg;
#[cfg(feature="mysql")]
use diesel::mysql::Mysql;

use table_data::TableData;
use super::data_structures::*;

pub trait UsesInformationSchema: Backend {
    type TypeColumn: SelectableExpression<
        self::information_schema::columns::table,
        types::Text,
    > + NonAggregate + QueryId + QueryFragment<Self>;

    fn type_column() -> Self::TypeColumn;
}

#[cfg(feature="postgres")]
impl UsesInformationSchema for Pg {
    type TypeColumn = self::information_schema::columns::udt_name;

    fn type_column() -> Self::TypeColumn {
        self::information_schema::columns::udt_name
    }
}

#[cfg(feature="mysql")]
impl UsesInformationSchema for Mysql {
    type TypeColumn = self::information_schema::columns::column_type;

    fn type_column() -> Self::TypeColumn {
        self::information_schema::columns::column_type
    }
}

mod information_schema {
    table! {
        information_schema.tables (table_schema, table_name) {
            table_schema -> VarChar,
            table_name -> VarChar,
            table_type -> VarChar,
        }
    }

    table! {
        information_schema.columns (table_schema, table_name, column_name) {
            table_schema -> VarChar,
            table_name -> VarChar,
            column_name -> VarChar,
            is_nullable -> VarChar,
            ordinal_position -> BigInt,
            udt_name -> VarChar,
            column_type -> VarChar,
        }
    }

    table! {
        information_schema.key_column_usage (table_schema, table_name, column_name, constraint_name) {
            table_schema -> VarChar,
            table_name -> VarChar,
            column_name -> VarChar,
            constraint_name -> VarChar,
            ordinal_position -> BigInt,
        }
    }

    table! {
        information_schema.table_constraints (table_schema, table_name, constraint_name) {
            table_schema -> VarChar,
            table_name -> VarChar,
            constraint_name -> VarChar,
            constraint_type -> VarChar,
        }
    }
}

pub fn determine_column_type(attr: &ColumnInformation) -> Result<ColumnType, Box<Error>> {
    let is_array = attr.type_name.starts_with('_');
    let tpe = if is_array {
        &attr.type_name[1..]
    } else {
        &attr.type_name
    };

    let tpe = if let Some(idx) = tpe.find('(') {
        &tpe[..idx]
    } else {
        tpe
    };

    Ok(ColumnType {
        path: vec!["diesel".into(), "types".into(), capitalize(tpe)],
        is_array: is_array,
        is_nullable: attr.nullable,
    })
}

fn capitalize(name: &str) -> String {
    name[..1].to_uppercase() + &name[1..]
}

pub fn get_table_data<Conn>(conn: &Conn, table: &TableData)
    -> QueryResult<Vec<ColumnInformation>> where
        Conn: Connection,
        Conn::Backend: UsesInformationSchema,
        String: FromSql<types::Text, Conn::Backend>,
{
    use self::information_schema::columns::dsl::*;

    let type_column = <Conn::Backend as UsesInformationSchema>::type_column();
    columns.select((column_name, type_column, is_nullable))
        .filter(table_name.eq(&table.name))
        .filter(table_schema.nullable().eq(&table.schema))
        .order(ordinal_position)
        .load(conn)
}

pub fn get_primary_keys<Conn>(conn: &Conn, table: &TableData)
    -> QueryResult<Vec<String>> where
        Conn: Connection,
        Conn::Backend: UsesInformationSchema,
        String: FromSql<types::Text, Conn::Backend>,
{
    use self::information_schema::table_constraints::{self, constraint_type};
    use self::information_schema::key_column_usage::dsl::*;

    let pk_query = table_constraints::table.select(table_constraints::constraint_name)
        .filter(constraint_type.eq("PRIMARY KEY"));

    key_column_usage.select(column_name)
        .filter(constraint_name.eq_any(pk_query))
        .filter(table_name.eq(&table.name))
        .filter(table_schema.nullable().eq(&table.schema))
        .order(ordinal_position)
        .load(conn)
}

pub fn load_table_names<Conn>(connection: &Conn, schema_name: Option<&str>)
    -> Result<Vec<TableData>, Box<Error>> where
        Conn: Connection,
        Conn::Backend: UsesInformationSchema,
        String: FromSql<types::Text, Conn::Backend>,
{
    use self::information_schema::tables::dsl::*;

    let schema_name = schema_name.unwrap_or("public");

    tables.select((table_name, table_schema))
        .filter(table_schema.eq(schema_name))
        .filter(table_name.not_like("\\_\\_%"))
        .filter(table_type.like("BASE TABLE"))
        .load(connection)
        .map_err(Into::into)
}

#[cfg(all(test, feature="postgres"))]
mod tests {
    extern crate dotenv;

    use super::*;
    use self::dotenv::dotenv;
    use diesel::pg::PgConnection;

    fn connection() -> PgConnection {
        let _ = dotenv();

        let connection_url = ::std::env::var("DATABASE_URL")
            .expect("DATABASE_URL must be set in order to run tests");
        let connection = PgConnection::establish(&connection_url).unwrap();
        connection.begin_test_transaction().unwrap();
        connection
    }

    #[test]
    fn skip_views() {
        let connection = connection();

        connection.execute("CREATE TABLE a_regular_table (id SERIAL PRIMARY KEY)").unwrap();
        connection.execute("CREATE VIEW a_view AS SELECT 42").unwrap();

        let table_names = load_table_names(&connection, None).unwrap();

        assert!(table_names.contains(&TableData::new("a_regular_table", "public")));
        assert!(!table_names.contains(&TableData::new("a_view", "public")));
    }

    #[test]
    fn load_table_names_loads_from_public_schema_if_none_given() {
        let connection = connection();

        connection.execute("CREATE TABLE load_table_names_loads_from_public_schema_if_none_given (id SERIAL PRIMARY KEY)")
            .unwrap();

        let table_names = load_table_names(&connection, None).unwrap();
        for TableData { schema, .. } in table_names {
            assert_eq!(Some("public".into()), schema);
        }
    }

    #[test]
    fn load_table_names_loads_from_custom_schema() {
        let connection = connection();

        connection.execute("CREATE SCHEMA test_schema").unwrap();
        connection.execute("CREATE TABLE test_schema.table_1 (id SERIAL PRIMARY KEY)").unwrap();

        let table_names = load_table_names(&connection, Some("test_schema")).unwrap();
        assert_eq!(vec![TableData::new("table_1", "test_schema")], table_names);

        connection.execute("CREATE TABLE test_schema.table_2 (id SERIAL PRIMARY KEY)").unwrap();

        let table_names = load_table_names(&connection, Some("test_schema")).unwrap();
        let expected = vec![
            TableData::new("table_1", "test_schema"),
            TableData::new("table_2", "test_schema"),
        ];
        assert_eq!(expected, table_names);

        connection.execute("CREATE SCHEMA other_test_schema").unwrap();
        connection.execute("CREATE TABLE other_test_schema.table_1 (id SERIAL PRIMARY KEY)").unwrap();

        let table_names = load_table_names(&connection, Some("test_schema")).unwrap();
        let expected = vec![
            TableData::new("table_1", "test_schema"),
            TableData::new("table_2", "test_schema"),
        ];
        assert_eq!(expected, table_names);
        let table_names = load_table_names(&connection, Some("other_test_schema")).unwrap();
        assert_eq!(vec![TableData::new("table_1", "other_test_schema")], table_names);
    }

    #[test]
    fn get_primary_keys_only_includes_primary_key() {
        let connection = connection();

        connection.execute("CREATE SCHEMA test_schema").unwrap();
        connection.execute("CREATE TABLE test_schema.table_1 (id SERIAL PRIMARY KEY, not_id INTEGER)").unwrap();
        connection.execute("CREATE TABLE test_schema.table_2 (id INTEGER, id2 INTEGER, not_id INTEGER, PRIMARY KEY (id, id2))").unwrap();

        let table_1 = TableData::new("table_1", "test_schema");
        let table_2 = TableData::new("table_2", "test_schema");
        assert_eq!(vec!["id".to_string()], get_primary_keys(&connection, &table_1).unwrap());
        assert_eq!(vec!["id".to_string(), "id2".to_string()], get_primary_keys(&connection, &table_2).unwrap());
    }

    #[test]
    fn get_table_data_loads_column_information() {
        let connection = connection();

        connection.execute("CREATE SCHEMA test_schema").unwrap();
        connection.execute("CREATE TABLE test_schema.table_1 (id SERIAL PRIMARY KEY, text_col VARCHAR, not_null TEXT NOT NULL)").unwrap();
        connection.execute("CREATE TABLE test_schema.table_2 (array_col VARCHAR[] NOT NULL)").unwrap();

        let table_1 = TableData::new("table_1", "test_schema");
        let table_2 = TableData::new("table_2", "test_schema");
        let id = ColumnInformation::new("id", "int4", false);
        let text_col = ColumnInformation::new("text_col", "varchar", true);
        let not_null = ColumnInformation::new("not_null", "text", false);
        let array_col = ColumnInformation::new("array_col", "_varchar", false);
        assert_eq!(Ok(vec![id, text_col, not_null]), get_table_data(&connection, &table_1));
        assert_eq!(Ok(vec![array_col]), get_table_data(&connection, &table_2));
    }
}
