//! The commands exposed by the migration engine core are defined in this
//! module.

mod apply_migrations;
mod apply_script;
mod command;
mod create_migration;
mod debug_panic;
mod dev_diagnostic;
mod diagnose_migration_history;
mod evaluate_data_loss;
mod get_database_version;
mod list_migration_directories;
mod mark_migration_applied;
mod mark_migration_rolled_back;
mod plan_migration;
mod reset;
mod schema_push;

pub use apply_migrations::{ApplyMigrationsCommand, ApplyMigrationsInput, ApplyMigrationsOutput};
pub use apply_script::{ApplyScriptCommand, ApplyScriptInput, ApplyScriptOutput};
pub use command::MigrationCommand;
pub use create_migration::{CreateMigrationCommand, CreateMigrationInput, CreateMigrationOutput};
pub use debug_panic::DebugPanicCommand;
pub use dev_diagnostic::{DevAction, DevDiagnosticCommand, DevDiagnosticInput, DevDiagnosticOutput};
pub use diagnose_migration_history::{
    DiagnoseMigrationHistoryCommand, DiagnoseMigrationHistoryInput, DiagnoseMigrationHistoryOutput, DriftDiagnostic,
    HistoryDiagnostic,
};
pub use evaluate_data_loss::*;
pub use get_database_version::*;
pub use list_migration_directories::*;
pub use mark_migration_applied::{MarkMigrationAppliedCommand, MarkMigrationAppliedInput, MarkMigrationAppliedOutput};
pub use mark_migration_rolled_back::{
    MarkMigrationRolledBackCommand, MarkMigrationRolledBackInput, MarkMigrationRolledBackOutput,
};
pub use plan_migration::{PlanMigrationCommand, PlanMigrationInput, PlanMigrationOutput};
pub use reset::ResetCommand;
pub use schema_push::{SchemaPushCommand, SchemaPushInput, SchemaPushOutput};
