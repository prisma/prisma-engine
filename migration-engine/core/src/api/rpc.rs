use super::error_rendering::render_jsonrpc_error;
use crate::{CoreError, CoreResult, GenericApi};
use jsonrpc_core::{types::error::Error as JsonRpcError, IoHandler, Params};
use std::sync::Arc;

/// A JSON-RPC ready migration API.
pub struct RpcApi {
    io_handler: jsonrpc_core::IoHandler<()>,
    executor: Arc<Box<dyn GenericApi>>,
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum RpcCommand {
    ApplyMigrations,
    CreateMigration,
    DebugPanic,
    DevDiagnostic,
    DiagnoseMigrationHistory,
    EvaluateDataLoss,
    GetDatabaseVersion,
    ListMigrationDirectories,
    MarkMigrationApplied,
    MarkMigrationRolledBack,
    PlanMigration,
    Reset,
    SchemaPush,
}

impl RpcCommand {
    fn name(&self) -> &'static str {
        match self {
            RpcCommand::ApplyMigrations => "applyMigrations",
            RpcCommand::CreateMigration => "createMigration",
            RpcCommand::DebugPanic => "debugPanic",
            RpcCommand::DevDiagnostic => "devDiagnostic",
            RpcCommand::DiagnoseMigrationHistory => "diagnoseMigrationHistory",
            RpcCommand::EvaluateDataLoss => "evaluateDataLoss",
            RpcCommand::GetDatabaseVersion => "getDatabaseVersion",
            RpcCommand::ListMigrationDirectories => "listMigrationDirectories",
            RpcCommand::MarkMigrationApplied => "markMigrationApplied",
            RpcCommand::MarkMigrationRolledBack => "markMigrationRolledBack",
            RpcCommand::PlanMigration => "planMigration",
            RpcCommand::Reset => "reset",
            RpcCommand::SchemaPush => "schemaPush",
        }
    }
}

const AVAILABLE_COMMANDS: &[RpcCommand] = &[
    RpcCommand::ApplyMigrations,
    RpcCommand::CreateMigration,
    RpcCommand::DebugPanic,
    RpcCommand::DevDiagnostic,
    RpcCommand::DiagnoseMigrationHistory,
    RpcCommand::EvaluateDataLoss,
    RpcCommand::GetDatabaseVersion,
    RpcCommand::ListMigrationDirectories,
    RpcCommand::MarkMigrationApplied,
    RpcCommand::MarkMigrationRolledBack,
    RpcCommand::PlanMigration,
    RpcCommand::Reset,
    RpcCommand::SchemaPush,
];

impl RpcApi {
    /// Initialize a migration engine API. This entails starting a database connection.
    pub async fn new(datamodel: &str) -> CoreResult<Self> {
        let mut rpc_api = Self {
            io_handler: IoHandler::default(),
            executor: Arc::new(crate::migration_api(datamodel).await?),
        };

        for cmd in AVAILABLE_COMMANDS {
            rpc_api.add_command_handler(*cmd);
        }

        Ok(rpc_api)
    }

    /// The JSON-RPC IO handler. This is what you can plug onto a transport.
    pub fn io_handler(&self) -> &IoHandler {
        &self.io_handler
    }

    fn add_command_handler(&mut self, cmd: RpcCommand) {
        let executor = Arc::clone(&self.executor);

        self.io_handler.add_method(cmd.name(), move |params: Params| {
            let executor = Arc::clone(&executor);
            Box::pin(Self::create_handler(executor, cmd, params))
        });
    }

    async fn create_handler(
        executor: Arc<Box<dyn GenericApi>>,
        cmd: RpcCommand,
        params: Params,
    ) -> Result<serde_json::Value, JsonRpcError> {
        let result: Result<serde_json::Value, RunCommandError> = Self::run_command(&**executor, cmd, params).await;

        match result {
            Ok(result) => Ok(result),
            Err(RunCommandError::JsonRpcError(err)) => Err(err),
            Err(RunCommandError::CoreError(err)) => Err(render_jsonrpc_error(err)),
        }
    }

    async fn run_command(
        executor: &dyn GenericApi,
        cmd: RpcCommand,
        params: Params,
    ) -> Result<serde_json::Value, RunCommandError> {
        tracing::debug!(?cmd, "running the command");
        Ok(match cmd {
            RpcCommand::ApplyMigrations => render(executor.apply_migrations(&params.parse()?).await?),
            RpcCommand::CreateMigration => render(executor.create_migration(&params.parse()?).await?),
            RpcCommand::DevDiagnostic => render(executor.dev_diagnostic(&params.parse()?).await?),
            RpcCommand::DebugPanic => render(executor.debug_panic().await?),
            RpcCommand::DiagnoseMigrationHistory => {
                render(executor.diagnose_migration_history(&params.parse()?).await?)
            }
            RpcCommand::EvaluateDataLoss => render(executor.evaluate_data_loss(&params.parse()?).await?),
            RpcCommand::GetDatabaseVersion => render(executor.version().await?),
            RpcCommand::ListMigrationDirectories => {
                render(executor.list_migration_directories(&params.parse()?).await?)
            }
            RpcCommand::MarkMigrationApplied => render(executor.mark_migration_applied(&params.parse()?).await?),
            RpcCommand::MarkMigrationRolledBack => render(executor.mark_migration_rolled_back(&params.parse()?).await?),
            RpcCommand::PlanMigration => render(executor.plan_migration(&params.parse()?).await?),
            RpcCommand::Reset => render(executor.reset().await?),
            RpcCommand::SchemaPush => render(executor.schema_push(&params.parse()?).await?),
        })
    }
}

fn render(result: impl serde::Serialize) -> serde_json::Value {
    serde_json::to_value(result).expect("Rendering of RPC response failed")
}

enum RunCommandError {
    JsonRpcError(JsonRpcError),
    CoreError(CoreError),
}

impl From<JsonRpcError> for RunCommandError {
    fn from(e: JsonRpcError) -> Self {
        RunCommandError::JsonRpcError(e)
    }
}

impl From<CoreError> for RunCommandError {
    fn from(e: CoreError) -> Self {
        RunCommandError::CoreError(e)
    }
}
