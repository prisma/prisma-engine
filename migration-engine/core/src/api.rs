mod error_rendering;
mod rpc;

pub use error_rendering::render_error;
pub use rpc::*;

use crate::{commands::*, migration_engine::MigrationEngine, CoreResult};
use migration_connector::*;
use tracing_futures::Instrument;

pub struct MigrationApi<C, D>
where
    C: MigrationConnector<DatabaseMigration = D>,
    D: DatabaseMigrationMarker + 'static,
{
    engine: MigrationEngine<C, D>,
}

impl<C, D> MigrationApi<C, D>
where
    C: MigrationConnector<DatabaseMigration = D>,
    D: DatabaseMigrationMarker + Send + Sync + 'static,
{
    pub async fn new(connector: C) -> CoreResult<Self> {
        let engine = MigrationEngine::new(connector).await?;

        Ok(Self { engine })
    }

    pub async fn handle_command<'a, E>(&'a self, input: &'a E::Input) -> CoreResult<E::Output>
    where
        E: MigrationCommand,
    {
        Ok(E::execute(input, &self.engine).await?)
    }

    pub fn connector(&self) -> &C {
        self.engine.connector()
    }
}

// This is here only to get rid of the generic type parameters due to neon not
// liking them in the exported class.
#[async_trait::async_trait]
pub trait GenericApi: Send + Sync + 'static {
    async fn version(&self, input: &serde_json::Value) -> CoreResult<String>;
    async fn apply_migration(&self, input: &ApplyMigrationInput) -> CoreResult<MigrationStepsResultOutput>;
    async fn apply_migrations(&self, input: &ApplyMigrationsInput) -> CoreResult<ApplyMigrationsOutput>;
    async fn calculate_database_steps(
        &self,
        input: &CalculateDatabaseStepsInput,
    ) -> CoreResult<MigrationStepsResultOutput>;
    async fn calculate_datamodel(&self, input: &CalculateDatamodelInput) -> CoreResult<CalculateDatamodelOutput>;
    async fn create_migration(&self, input: &CreateMigrationInput) -> CoreResult<CreateMigrationOutput>;
    async fn debug_panic(&self, input: &()) -> CoreResult<()>;
    async fn diagnose_migration_history(
        &self,
        input: &DiagnoseMigrationHistoryInput,
    ) -> CoreResult<DiagnoseMigrationHistoryOutput>;
    async fn infer_migration_steps(&self, input: &InferMigrationStepsInput) -> CoreResult<MigrationStepsResultOutput>;
    async fn initialize(&self, input: &InitializeInput) -> CoreResult<InitializeOutput>;
    async fn list_migrations(&self, input: &serde_json::Value) -> CoreResult<Vec<ListMigrationsOutput>>;
    async fn migration_progress(&self, input: &MigrationProgressInput) -> CoreResult<MigrationProgressOutput>;
    async fn plan_migration(&self, input: &PlanMigrationInput) -> CoreResult<PlanMigrationOutput>;
    async fn reset(&self, input: &serde_json::Value) -> CoreResult<serde_json::Value>;
    async fn schema_push(&self, input: &SchemaPushInput) -> CoreResult<SchemaPushOutput>;
    async fn unapply_migration(&self, input: &UnapplyMigrationInput) -> CoreResult<UnapplyMigrationOutput>;
    fn migration_persistence<'a>(&'a self) -> Box<dyn MigrationPersistence + 'a>;
    fn connector_type(&self) -> &'static str;

    fn render_error(&self, error: crate::error::Error) -> user_facing_errors::Error {
        error_rendering::render_error(error)
    }

    fn render_jsonrpc_error(&self, error: crate::error::Error) -> jsonrpc_core::error::Error {
        error_rendering::render_jsonrpc_error(error)
    }
}

#[async_trait::async_trait]
impl<C, D> GenericApi for MigrationApi<C, D>
where
    C: MigrationConnector<DatabaseMigration = D>,
    D: DatabaseMigrationMarker + Send + Sync + 'static,
{
    async fn version(&self, input: &serde_json::Value) -> CoreResult<String> {
        self.handle_command::<VersionCommand>(input)
            .instrument(tracing::info_span!("Version"))
            .await
    }

    async fn apply_migration(&self, input: &ApplyMigrationInput) -> CoreResult<MigrationStepsResultOutput> {
        self.handle_command::<ApplyMigrationCommand<'_>>(input)
            .instrument(tracing::info_span!(
                "ApplyMigration",
                migration_id = input.migration_id.as_str()
            ))
            .await
    }

    async fn apply_migrations(&self, input: &ApplyMigrationsInput) -> CoreResult<ApplyMigrationsOutput> {
        self.handle_command::<ApplyMigrationsCommand>(input)
            .instrument(tracing::info_span!("ApplyMigrations"))
            .await
    }

    async fn calculate_database_steps(
        &self,
        input: &CalculateDatabaseStepsInput,
    ) -> CoreResult<MigrationStepsResultOutput> {
        self.handle_command::<CalculateDatabaseStepsCommand<'_>>(input)
            .instrument(tracing::info_span!("CalculateDatabaseSteps"))
            .await
    }

    async fn calculate_datamodel(&self, input: &CalculateDatamodelInput) -> CoreResult<CalculateDatamodelOutput> {
        self.handle_command::<CalculateDatamodelCommand>(input)
            .instrument(tracing::info_span!("CalculateDatamodel"))
            .await
    }

    async fn create_migration(&self, input: &CreateMigrationInput) -> CoreResult<CreateMigrationOutput> {
        self.handle_command::<CreateMigrationCommand>(input)
            .instrument(tracing::info_span!(
                "CreateMigration",
                migration_name = input.migration_name.as_str()
            ))
            .await
    }

    async fn debug_panic(&self, input: &()) -> CoreResult<()> {
        self.handle_command::<DebugPanicCommand>(input)
            .instrument(tracing::info_span!("DebugPanic"))
            .await
    }

    async fn diagnose_migration_history(
        &self,
        input: &DiagnoseMigrationHistoryInput,
    ) -> CoreResult<DiagnoseMigrationHistoryOutput> {
        self.handle_command::<DiagnoseMigrationHistoryCommand>(input)
            .instrument(tracing::info_span!("DiagnoseMigrationHistory"))
            .await
    }

    async fn infer_migration_steps(&self, input: &InferMigrationStepsInput) -> CoreResult<MigrationStepsResultOutput> {
        self.handle_command::<InferMigrationStepsCommand<'_>>(input)
            .instrument(tracing::info_span!(
                "InferMigrationSteps",
                migration_id = input.migration_id.as_str()
            ))
            .await
    }

    async fn initialize(&self, input: &InitializeInput) -> CoreResult<InitializeOutput> {
        self.handle_command::<InitializeCommand>(input)
            .instrument(tracing::info_span!(
                "Initialize",
                migrations_directory_path = input.migrations_directory_path.as_str()
            ))
            .await
    }

    async fn list_migrations(&self, input: &serde_json::Value) -> CoreResult<Vec<ListMigrationsOutput>> {
        self.handle_command::<ListMigrationsCommand>(input)
            .instrument(tracing::info_span!("ListMigrations"))
            .await
    }

    async fn migration_progress(&self, input: &MigrationProgressInput) -> CoreResult<MigrationProgressOutput> {
        self.handle_command::<MigrationProgressCommand>(input)
            .instrument(tracing::info_span!(
                "MigrationProgress",
                migration_id = input.migration_id.as_str()
            ))
            .await
    }

    async fn plan_migration(&self, input: &PlanMigrationInput) -> CoreResult<PlanMigrationOutput> {
        self.handle_command::<PlanMigrationCommand>(input)
            .instrument(tracing::info_span!("PlanMigration"))
            .await
    }

    async fn reset(&self, input: &serde_json::Value) -> CoreResult<serde_json::Value> {
        self.handle_command::<ResetCommand>(input)
            .instrument(tracing::info_span!("Reset"))
            .await
    }

    async fn schema_push(&self, input: &SchemaPushInput) -> CoreResult<SchemaPushOutput> {
        self.handle_command::<SchemaPushCommand>(input)
            .instrument(tracing::info_span!("SchemaPush"))
            .await
    }

    async fn unapply_migration(&self, input: &UnapplyMigrationInput) -> CoreResult<UnapplyMigrationOutput> {
        self.handle_command::<UnapplyMigrationCommand<'_>>(input)
            .instrument(tracing::info_span!("UnapplyMigration"))
            .await
    }

    fn migration_persistence<'a>(&'a self) -> Box<dyn MigrationPersistence + 'a> {
        self.engine.connector().migration_persistence()
    }

    fn connector_type(&self) -> &'static str {
        self.engine.connector().connector_type()
    }
}
