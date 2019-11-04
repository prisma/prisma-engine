use super::{
    command_helpers::run_infer_command,
    misc_helpers::{
        mysql_8_url, mysql_migration_connector, mysql_url, postgres_migration_connector, postgres_url,
        sqlite_migration_connector, test_api,
    },
    InferAndApplyOutput, SCHEMA_NAME,
};
use migration_connector::{MigrationPersistence, MigrationStep};
use migration_core::{
    api::GenericApi,
    commands::{ApplyMigrationInput, InferMigrationStepsInput},
};
use sql_connection::SyncSqlConnection;
use sql_migration_connector::SqlFamily;
use sql_schema_describer::*;
use std::sync::Arc;

/// A handle to all the context needed for end-to-end testing of the migration engine across
/// connectors.
pub struct TestApi {
    sql_family: SqlFamily,
    database: Arc<dyn SyncSqlConnection + Send + Sync + 'static>,
    api: Box<dyn GenericApi>,
}

impl TestApi {
    pub fn migration_persistence(&self) -> Arc<dyn MigrationPersistence> {
        self.api.migration_persistence()
    }

    pub fn apply_migration(&self, steps: Vec<MigrationStep>, migration_id: &str) -> InferAndApplyOutput {
        let input = ApplyMigrationInput {
            migration_id: migration_id.to_string(),
            steps,
            force: None,
        };

        let migration_output = self.api.apply_migration(&input).expect("ApplyMigration failed");

        assert!(
            migration_output.general_errors.is_empty(),
            format!(
                "ApplyMigration returned unexpected errors: {:?}",
                migration_output.general_errors
            )
        );

        InferAndApplyOutput {
            sql_schema: self.introspect_database(),
            migration_output,
        }
    }

    pub fn infer_and_apply(&self, datamodel: &str) -> InferAndApplyOutput {
        let migration_id = "the-migration-id";

        let input = InferMigrationStepsInput {
            migration_id: migration_id.to_string(),
            datamodel: datamodel.to_string(),
            assume_to_be_applied: Vec::new(),
        };

        let steps = run_infer_command(self.api.as_ref(), input).0.datamodel_steps;

        self.apply_migration(steps, migration_id)
    }

    fn introspect_database(&self) -> SqlSchema {
        let inspector: Box<dyn SqlSchemaDescriberBackend> = match self.api.connector_type() {
            "postgresql" => Box::new(sql_schema_describer::postgres::SqlSchemaDescriber::new(Arc::clone(
                &self.database,
            ))),
            "sqlite" => Box::new(sql_schema_describer::sqlite::SqlSchemaDescriber::new(Arc::clone(
                &self.database,
            ))),
            "mysql" => Box::new(sql_schema_describer::mysql::SqlSchemaDescriber::new(Arc::clone(
                &self.database,
            ))),
            _ => unimplemented!(),
        };

        let mut result = inspector
            .describe(&SCHEMA_NAME.to_string())
            .expect("Introspection failed");

        // the presence of the _Migration table makes assertions harder. Therefore remove it from the result.
        result.tables = result.tables.into_iter().filter(|t| t.name != "_Migration").collect();

        result
    }
}

pub fn mysql_8_test_api() -> TestApi {
    let connector = mysql_migration_connector(&mysql_8_url());

    TestApi {
        sql_family: SqlFamily::Mysql,
        database: Arc::clone(&connector.database),
        api: Box::new(test_api(connector)),
    }
}

pub fn mysql_test_api() -> TestApi {
    let connector = mysql_migration_connector(&mysql_url());

    TestApi {
        sql_family: SqlFamily::Mysql,
        database: Arc::clone(&connector.database),
        api: Box::new(test_api(connector)),
    }
}

pub fn postgres_test_api() -> TestApi {
    let connector = postgres_migration_connector(&postgres_url());

    TestApi {
        sql_family: SqlFamily::Postgres,
        database: Arc::clone(&connector.database),
        api: Box::new(test_api(connector)),
    }
}

pub fn sqlite_test_api() -> TestApi {
    let connector = sqlite_migration_connector();

    TestApi {
        sql_family: SqlFamily::Sqlite,
        database: Arc::clone(&connector.database),
        api: Box::new(test_api(connector)),
    }
}
