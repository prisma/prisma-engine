use super::MigrationCommand;
use crate::{api::MigrationApi, parse_datamodel, CoreError, CoreResult};
use migration_connector::{DatabaseMigrationMarker, MigrationConnector};
use serde::{Deserialize, Serialize};
use std::path::Path;
use user_facing_errors::migration_engine::{MigrationNameTooLong, ProviderSwitchedError};

/// Create and potentially apply a new migration.
pub struct CreateMigrationCommand;

/// The input to the `createMigration` command.
#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct CreateMigrationInput {
    /// The filesystem path of the migrations directory to use.
    pub migrations_directory_path: String,
    /// The current prisma schema to use as a target for the generated migration.
    pub prisma_schema: String,
    /// The user-given name for the migration. This will be used in the migration directory.
    pub migration_name: String,
    /// If true, always generate a migration, but do not apply.
    pub draft: bool,
}

/// The output of the `createMigration` command.
#[derive(Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct CreateMigrationOutput {
    /// The name of the newly generated migration directory, if any.
    pub generated_migration_name: Option<String>,
}

#[async_trait::async_trait]
impl<'a> MigrationCommand for CreateMigrationCommand {
    type Input = CreateMigrationInput;

    type Output = CreateMigrationOutput;

    async fn execute<C: MigrationConnector>(input: &Self::Input, engine: &MigrationApi<C>) -> CoreResult<Self::Output> {
        let database_migration_inferrer = engine.connector().database_migration_inferrer();
        let applier = engine.connector().database_migration_step_applier();
        let checker = engine.connector().destructive_change_checker();

        if input.migration_name.len() > 200 {
            return Err(CoreError::user_facing(MigrationNameTooLong));
        }

        //check for provider switch
        let connector_type = engine.connector().connector_type();
        if matches!(
            migration_connector::match_provider_in_lock_file(&input.migrations_directory_path, connector_type),
            Some(false)
        ) {
            return Err(CoreError::user_facing(ProviderSwitchedError {
                provider: connector_type.into(),
            })
            .into());
        }

        // Infer the migration.
        let previous_migrations = migration_connector::list_migrations(&Path::new(&input.migrations_directory_path))?;
        let target_schema = parse_datamodel(&input.prisma_schema)?;

        let migration = database_migration_inferrer
            .infer_next_migration(&previous_migrations, &target_schema)
            .await?;

        if migration.is_empty() && !input.draft {
            return Ok(CreateMigrationOutput {
                generated_migration_name: None,
            });
        }

        let destructive_change_diagnostics = checker.pure_check(&migration);

        let migration_script = applier.render_script(&migration, &destructive_change_diagnostics);

        // Write the migration script to a file.
        let directory = migration_connector::create_migration_directory(
            &Path::new(&input.migrations_directory_path),
            &input.migration_name,
        )
        .map_err(|_| CoreError::Generic(anyhow::anyhow!("Failed to create a new migration directory.")))?;

        directory
            .write_migration_script(&migration_script, C::DatabaseMigration::FILE_EXTENSION)
            .map_err(|err| {
                CoreError::Generic(anyhow::Error::new(err).context(format!(
                    "Failed to write the migration script to `{:?}`",
                    directory.path(),
                )))
            })?;

        migration_connector::write_migration_lock_file(&input.migrations_directory_path, connector_type).map_err(
            |err| {
                CoreError::Generic(anyhow::Error::new(err).context(format!(
                    "Failed to write the migration lock file to `{:?}`",
                    &input.migrations_directory_path
                )))
            },
        )?;

        Ok(CreateMigrationOutput {
            generated_migration_name: Some(directory.migration_name().to_owned()),
        })
    }
}
