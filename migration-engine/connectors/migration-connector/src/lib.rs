#![deny(rust_2018_idioms, unsafe_code, missing_docs)]

//! This crate defines the API exposed by the connectors to the migration engine core. The entry point for this API is the [MigrationConnector](trait.MigrationConnector.html) trait.

mod database_migration_inferrer;
mod database_migration_step_applier;
mod destructive_change_checker;
mod error;
mod migration_persistence;
mod migrations_directory;

pub use database_migration_inferrer::DatabaseMigrationInferrer;
pub use database_migration_step_applier::{DatabaseMigrationStepApplier, PrettyDatabaseMigrationStep};
pub use destructive_change_checker::{
    DestructiveChangeChecker, DestructiveChangeDiagnostics, MigrationWarning, UnexecutableMigration,
};
pub use error::ConnectorError;
pub use migration_persistence::{MigrationPersistence, MigrationRecord, PersistenceNotInitializedError, Timestamp};
pub use migrations_directory::{
    create_migration_directory, error_on_changed_provider, list_migrations, write_migration_lock_file,
    ListMigrationsError, MigrationDirectory,
};

use sha2::{Digest, Sha256};
use std::fmt::Debug;

/// The top-level trait for connectors. This is the abstraction the migration engine core relies on to
/// interface with different database backends.
#[async_trait::async_trait]
pub trait MigrationConnector: Send + Sync + 'static {
    /// The data structure containing the concrete migration steps for the connector. A migration is
    /// assumed to consist of multiple steps.
    ///
    /// For example, in the SQL connector, a step would represent an SQL statement like `CREATE TABLE`.
    type DatabaseMigration: DatabaseMigrationMarker + Send + Sync + 'static;

    /// If possible on the target connector, acquire an advisory lock, so multiple instances of migrate do not run concurrently.
    async fn acquire_lock(&self) -> ConnectorResult<()>;

    /// A string that should identify what database backend is being used. Note that this is not necessarily
    /// the connector name. The SQL connector for example can return "postgresql", "mysql" or "sqlite".
    fn connector_type(&self) -> &'static str;

    /// The version of the underlying database.
    async fn version(&self) -> ConnectorResult<String>;

    /// Create the database with the provided URL.
    async fn create_database(database_str: &str) -> ConnectorResult<String>;

    /// Drop all database state.
    async fn reset(&self) -> ConnectorResult<()>;

    /// Optionally check that the features implied by the provided datamodel are all compatible with
    /// the specific database version being used.
    fn check_database_version_compatibility(
        &self,
        _datamodel: &datamodel::dml::Datamodel,
    ) -> Option<user_facing_errors::common::DatabaseVersionIncompatibility> {
        None
    }

    /// See [MigrationPersistence](trait.MigrationPersistence.html).
    fn migration_persistence(&self) -> &dyn MigrationPersistence;

    /// See [DatabaseMigrationInferrer](trait.DatabaseMigrationInferrer.html).
    fn database_migration_inferrer(&self) -> &dyn DatabaseMigrationInferrer<Self::DatabaseMigration>;

    /// See [DatabaseMigrationStepApplier](trait.DatabaseMigrationStepApplier.html).
    fn database_migration_step_applier(&self) -> &dyn DatabaseMigrationStepApplier<Self::DatabaseMigration>;

    /// See [DestructiveChangeChecker](trait.DestructiveChangeChecker.html).
    fn destructive_change_checker(&self) -> &dyn DestructiveChangeChecker<Self::DatabaseMigration>;
}

/// Marker for the associated migration type for a connector.
pub trait DatabaseMigrationMarker: Debug + Send + Sync {
    /// The file extension to use for migration scripts.
    const FILE_EXTENSION: &'static str;

    /// Is the migration empty?
    fn is_empty(&self) -> bool;
}

/// Shorthand for a [Result](https://doc.rust-lang.org/std/result/enum.Result.html) where the error
/// variant is a [ConnectorError](/error/enum.ConnectorError.html).
pub type ConnectorResult<T> = Result<T, ConnectorError>;

/// Compute the checksum for a migration script, and return it formatted to be human-readable.
fn checksum(script: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(script.as_bytes());
    let checksum: [u8; 32] = hasher.finalize().into();
    checksum.format_checksum()
}

/// The length (in bytes, or equivalently ascii characters) of the checksum
/// strings.
const CHECKSUM_STR_LEN: usize = 64;

/// Format a checksum to a hexadecimal string. This is used to checksum
/// migration scripts with Sha256.
pub trait FormatChecksum {
    /// Format a checksum to a hexadecimal string.
    fn format_checksum(&self) -> String;
    /// Obsolete checksum method, should only be used for compatibility.
    fn format_checksum_old(&self) -> String;
}

impl FormatChecksum for [u8; 32] {
    fn format_checksum(&self) -> String {
        use std::fmt::Write as _;

        let mut checksum_string = String::with_capacity(32 * 2);

        for byte in self {
            write!(checksum_string, "{:02x}", byte).unwrap();
        }

        assert_eq!(checksum_string.len(), CHECKSUM_STR_LEN);

        checksum_string
    }

    // Due to an omission in a previous version of the migration engine,
    // some migrations tables will have old migrations with checksum strings
    // that have not been zero-padded.
    //
    // Corresponding issue:
    // https://github.com/prisma/prisma-engines/issues/1887
    fn format_checksum_old(&self) -> String {
        use std::fmt::Write as _;

        let mut checksum_string = String::with_capacity(32 * 2);

        for byte in self {
            write!(checksum_string, "{:x}", byte).unwrap();
        }

        checksum_string
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn format_checksum_does_not_strip_zeros() {
        assert_eq!(
            checksum("hello"),
            "2cf24dba5fb0a30e26e83b2ac5b9e29e1b161e5c1fa7425e73043362938b9824"
        );
        assert_eq!(checksum("abcd").len(), CHECKSUM_STR_LEN);
    }
}
