use barrel::Migration;
use pretty_assertions::assert_eq;
use quaint::{connector::Queryable};
use std::sync::Arc;
use test_setup::*;

pub(crate) fn custom_assert(left: &str, right: &str) {
    let parsed_expected = datamodel::parse_datamodel(&right).unwrap();
    let reformatted_expected =
        datamodel::render_datamodel_to_string(&parsed_expected).expect("Datamodel rendering failed");

    assert_eq!(left, reformatted_expected);
}

async fn run_full_sql(database: &Arc<dyn Queryable + Send + Sync>, full_sql: &str) {
    for sql in full_sql.split(";") {
        if sql != "" {
            database.query_raw(&sql, &[]).await.unwrap();
        }
    }
}

// barrel

pub struct BarrelMigrationExecutor {
    pub(super) database: Arc<dyn Queryable + Send + Sync>,
    pub(super) sql_variant: barrel::backend::SqlVariant,
}

impl BarrelMigrationExecutor {
    pub async fn execute<F>(&self, migration_fn: F)
    where
        F: FnMut(&mut Migration) -> (),
    {
        self.execute_with_schema(migration_fn, SCHEMA_NAME).await
    }

    pub async fn execute_with_schema<F>(&self, mut migration_fn: F, schema_name: &str)
    where
        F: FnMut(&mut Migration) -> (),
    {
        let mut migration = Migration::new().schema(schema_name);
        migration_fn(&mut migration);
        let full_sql = migration.make_from(self.sql_variant);
        run_full_sql(&self.database, &full_sql).await;
    }
}
