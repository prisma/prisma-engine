use crate::{error::quaint_error_to_connector_error, SqlMigrationConnector};
use migration_connector::{ConnectorResult, FormatChecksum, ImperativeMigrationsPersistence, MigrationRecord};
use quaint::ast::*;
use sha2::{Digest, Sha256};
use uuid::Uuid;

const IMPERATIVE_MIGRATIONS_TABLE_NAME: &str = "_prisma_migrations";

#[async_trait::async_trait]
impl ImperativeMigrationsPersistence for SqlMigrationConnector {
    async fn record_migration_started(&self, migration_name: &str, script: &str) -> ConnectorResult<String> {
        let conn = self.conn();
        let id = Uuid::new_v4().to_string();

        let mut hasher = Sha256::new();
        hasher.update(script.as_bytes());
        let checksum: [u8; 32] = hasher.finalize().into();
        let checksum_string = checksum.format_checksum();

        let insert = Insert::single_into((self.schema_name(), IMPERATIVE_MIGRATIONS_TABLE_NAME))
            .value("id", id.as_str())
            .value("checksum", checksum_string.as_str())
            // We need this line because MySQL can't default a text field to an empty string
            .value("logs", "")
            .value("migration_name", migration_name)
            .value("script", script);

        conn.execute(insert).await?;

        Ok(id)
    }

    async fn record_successful_step(&self, id: &str, logs: &str) -> ConnectorResult<()> {
        use quaint::ast::*;

        let update = Update::table((self.schema_name(), IMPERATIVE_MIGRATIONS_TABLE_NAME))
            .so_that(Column::from("id").equals(id))
            .set(
                "applied_steps_count",
                Expression::from(Column::from("applied_steps_count")) + Expression::from(1),
            )
            .set("logs", logs);

        self.conn().execute(update).await?;

        Ok(())
    }

    async fn record_failed_step(&self, id: &str, logs: &str) -> ConnectorResult<()> {
        let update = Update::table((self.schema_name(), IMPERATIVE_MIGRATIONS_TABLE_NAME))
            .so_that(Column::from("id").equals(id))
            .set("logs", logs);

        self.conn().execute(update).await?;

        Ok(())
    }

    async fn record_migration_finished(&self, id: &str) -> ConnectorResult<()> {
        let update = Update::table((self.schema_name(), IMPERATIVE_MIGRATIONS_TABLE_NAME))
            .so_that(Column::from("id").equals(id))
            .set("finished_at", chrono::Utc::now()); // TODO maybe use a database generated timestamp

        self.conn().execute(update).await?;

        Ok(())
    }

    async fn list_migrations(&self) -> ConnectorResult<Vec<MigrationRecord>> {
        self.flavour.create_imperative_migrations_table(self.conn()).await?;

        let select = Select::from_table((self.schema_name(), IMPERATIVE_MIGRATIONS_TABLE_NAME))
            .column("id")
            .column("checksum")
            .column("finished_at")
            .column("migration_name")
            .column("logs")
            .column("rolled_back_at")
            .column("started_at")
            .column("applied_steps_count")
            .column("script")
            .order_by("started_at".ascend());

        let result = self.conn().query(select).await?;

        let rows = quaint::serde::from_rows(result)
            .map_err(|err| quaint_error_to_connector_error(err, self.database_info().connection_info()))?;

        Ok(rows)
    }
}
