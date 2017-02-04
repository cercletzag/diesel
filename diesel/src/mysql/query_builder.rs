use super::backend::Mysql;
use query_builder::{QueryBuilder, BuildQueryResult};

#[allow(missing_debug_implementations)]
#[derive(Default)]
pub struct MysqlQueryBuilder {
    pub sql: String,
}

impl MysqlQueryBuilder {
    pub fn new() -> Self {
        MysqlQueryBuilder::default()
    }
}

impl QueryBuilder<Mysql> for MysqlQueryBuilder {
    fn push_sql(&mut self, sql: &str) {
        self.sql.push_str(sql);
    }

    fn push_identifier(&mut self, identifier: &str) -> BuildQueryResult {
        self.push_sql("`");
        self.push_sql(&identifier.replace("`", "``"));
        self.push_sql("`");
        Ok(())
    }

    fn push_bind_param(&mut self) {
        self.push_sql("?");
    }
}
