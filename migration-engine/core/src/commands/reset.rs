use crate::commands::command::*;
use crate::migration_engine::MigrationEngine;
use migration_connector::*;
use serde_json::json;

/// The `reset` command.
pub struct ResetCommand;

#[async_trait::async_trait]
impl<'a> MigrationCommand for ResetCommand {
    type Input = serde_json::Value;
    type Output = serde_json::Value;

    async fn execute<C, D>(_input: &Self::Input, engine: &MigrationEngine<C, D>) -> CommandResult<Self::Output>
    where
        C: MigrationConnector<DatabaseMigration = D>,
        D: DatabaseMigrationMarker + 'static,
    {
        engine.reset().await?;
        engine.init().await?;

        Ok(json!({}))
    }
}
