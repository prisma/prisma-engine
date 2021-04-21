use crate::IntoConnectorResult;
use migration_connector::DatabaseMigrationStepApplier;
use mongodb_migration::MongoDbMigrationStep;

use crate::{
    mongodb_migration::{self, MongoDbMigration},
    MongoDbMigrationConnector,
};

#[async_trait::async_trait]
impl DatabaseMigrationStepApplier<MongoDbMigration> for MongoDbMigrationConnector {
    async fn apply_migration(
        &self,
        database_migration: &MongoDbMigration,
    ) -> migration_connector::ConnectorResult<u32> {
        let db = self.client.database(&self.db_name);

        for step in database_migration.steps.iter() {
            match step {
                MongoDbMigrationStep::CreateCollection(name) => db
                    .create_collection(name.as_str(), None)
                    .await
                    .into_connector_result()?,
            }
        }

        Ok(database_migration.steps.len() as u32)
    }

    fn render_steps_pretty(
        &self,
        _database_migration: &MongoDbMigration,
    ) -> migration_connector::ConnectorResult<Vec<migration_connector::PrettyDatabaseMigrationStep>> {
        todo!()
    }

    fn render_script(
        &self,
        _database_migration: &MongoDbMigration,
        _diagnostics: &migration_connector::DestructiveChangeDiagnostics,
    ) -> String {
        todo!()
    }

    async fn apply_script(&self, _migration_name: &str, _script: &str) -> migration_connector::ConnectorResult<()> {
        todo!()
    }
}
