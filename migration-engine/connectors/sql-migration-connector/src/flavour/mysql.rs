use super::SqlFlavour;
use crate::{
    connect,
    connection_wrapper::Connection,
    error::{quaint_error_to_connector_error, SystemDatabase},
    SqlMigrationConnector,
};
use datamodel::{walkers::walk_scalar_fields, Datamodel};
use enumflags2::BitFlags;
use indoc::indoc;
use migration_connector::{ConnectorError, ConnectorResult, MigrationDirectory, MigrationFeature};
use once_cell::sync::Lazy;
use quaint::connector::MysqlUrl;
use regex::RegexSet;
use sql_schema_describer::{DescriberErrorKind, SqlSchema, SqlSchemaDescriberBackend};
use std::sync::atomic::{AtomicU8, Ordering};
use url::Url;

const ADVISORY_LOCK_TIMEOUT: std::time::Duration = std::time::Duration::from_secs(10);

#[derive(Debug)]
pub(crate) struct MysqlFlavour {
    pub(super) url: MysqlUrl,
    /// See the [Circumstances] enum.
    pub(super) circumstances: AtomicU8,
    /// Relevant features enabled in the schema,
    pub(super) features: BitFlags<MigrationFeature>,
}

impl MysqlFlavour {
    pub(crate) fn is_mariadb(&self) -> bool {
        BitFlags::<Circumstances>::from_bits(self.circumstances.load(Ordering::Relaxed))
            .unwrap_or_default()
            .contains(Circumstances::IsMariadb)
    }

    pub(crate) fn is_mysql_5_6(&self) -> bool {
        BitFlags::<Circumstances>::from_bits(self.circumstances.load(Ordering::Relaxed))
            .unwrap_or_default()
            .contains(Circumstances::IsMysql56)
    }

    pub(crate) fn lower_cases_table_names(&self) -> bool {
        BitFlags::<Circumstances>::from_bits(self.circumstances.load(Ordering::Relaxed))
            .unwrap_or_default()
            .contains(Circumstances::LowerCasesTableNames)
    }

    async fn shadow_database_connection(
        &self,
        main_connection: &Connection,
        connector: &SqlMigrationConnector,
        temporary_database_name: Option<&str>,
    ) -> ConnectorResult<Connection> {
        if let Some(shadow_database_connection_string) = &connector.shadow_database_connection_string {
            let conn = crate::connect(shadow_database_connection_string).await?;
            let shadow_conninfo = conn.connection_info();
            let main_conninfo = main_connection.connection_info();

            if shadow_conninfo.host() == main_conninfo.host() && shadow_conninfo.dbname() == main_conninfo.dbname() {
                return Err(ConnectorError::from_msg("The shadow database you configured appears to be the same as as the main database. Please specify another shadow database.".into()));
            }

            tracing::info!(
                "Connecting to user-provided shadow database at {}",
                shadow_database_connection_string
            );

            if self.reset(&conn).await.is_err() {
                connector.best_effort_reset(&conn).await?;
            }

            return Ok(conn);
        }

        let database_name = temporary_database_name.unwrap();
        let create_database = format!("CREATE DATABASE `{}`", database_name);

        main_connection
            .raw_cmd(&create_database)
            .await
            .map_err(ConnectorError::from)
            .map_err(|err| err.into_shadow_db_creation_error())?;

        let mut temporary_database_url = self.url.url().clone();
        temporary_database_url.set_path(&format!("/{}", database_name));
        let temporary_database_url = temporary_database_url.to_string();

        tracing::debug!("Connecting to temporary database at {:?}", temporary_database_url);

        Ok(crate::connect(&temporary_database_url).await?)
    }
}

#[async_trait::async_trait]
impl SqlFlavour for MysqlFlavour {
    async fn acquire_lock(&self, connection: &Connection) -> ConnectorResult<()> {
        // https://dev.mysql.com/doc/refman/8.0/en/locking-functions.html
        let query = format!("SELECT GET_LOCK('prisma_migrate', {})", ADVISORY_LOCK_TIMEOUT.as_secs());
        Ok(connection.raw_cmd(&query).await?)
    }

    fn check_database_version_compatibility(
        &self,
        datamodel: &Datamodel,
    ) -> Option<user_facing_errors::common::DatabaseVersionIncompatibility> {
        if self.is_mysql_5_6() {
            let mut errors = Vec::new();

            check_datamodel_for_mysql_5_6(datamodel, &mut errors);

            if errors.is_empty() {
                return None;
            }

            let mut errors_string = String::with_capacity(errors.iter().map(|err| err.len() + 3).sum());

            for error in &errors {
                errors_string.push_str("- ");
                errors_string.push_str(error);
                errors_string.push('\n');
            }

            Some(user_facing_errors::common::DatabaseVersionIncompatibility {
                errors: errors_string,
                database_version: "MySQL 5.6".into(),
            })
        } else {
            None
        }
    }

    async fn create_database(&self, database_str: &str) -> ConnectorResult<String> {
        let mut url = Url::parse(database_str).map_err(|err| ConnectorError::url_parse_error(err, database_str))?;
        url.set_path("/mysql");

        let conn = connect(&url.to_string()).await?;
        let db_name = self.url.dbname();

        let query = format!(
            "CREATE DATABASE `{}` CHARACTER SET utf8mb4 COLLATE utf8mb4_unicode_ci;",
            db_name
        );

        conn.raw_cmd(&query).await?;

        Ok(db_name.to_owned())
    }

    async fn create_migrations_table(&self, connection: &Connection) -> ConnectorResult<()> {
        let sql = indoc! {r#"
            CREATE TABLE _prisma_migrations (
                id                      VARCHAR(36) PRIMARY KEY NOT NULL,
                checksum                VARCHAR(64) NOT NULL,
                finished_at             DATETIME(3),
                migration_name          VARCHAR(255) NOT NULL,
                logs                    TEXT,
                rolled_back_at          DATETIME(3),
                started_at              DATETIME(3) NOT NULL DEFAULT CURRENT_TIMESTAMP(3),
                applied_steps_count     INTEGER UNSIGNED NOT NULL DEFAULT 0
            ) DEFAULT CHARACTER SET utf8mb4 COLLATE utf8mb4_unicode_ci;
        "#};

        Ok(connection.raw_cmd(sql).await?)
    }

    async fn describe_schema<'a>(&'a self, connection: &Connection) -> ConnectorResult<SqlSchema> {
        sql_schema_describer::mysql::SqlSchemaDescriber::new(connection.quaint().clone())
            .describe(connection.connection_info().schema_name())
            .await
            .map_err(|err| match err.into_kind() {
                DescriberErrorKind::QuaintError(err) => {
                    quaint_error_to_connector_error(err, connection.connection_info())
                }
                DescriberErrorKind::CrossSchemaReference { .. } => {
                    unreachable!("No schemas in MySQL")
                }
            })
    }

    async fn drop_database(&self, database_url: &str) -> ConnectorResult<()> {
        let connection = connect(database_url).await?;
        let db_name = connection.connection_info().dbname().unwrap();

        connection.raw_cmd(&format!("DROP DATABASE `{}`", db_name)).await?;

        Ok(())
    }

    async fn drop_migrations_table(&self, connection: &Connection) -> ConnectorResult<()> {
        connection.raw_cmd("DROP TABLE _prisma_migrations").await?;

        Ok(())
    }

    async fn ensure_connection_validity(&self, connection: &Connection) -> ConnectorResult<()> {
        static MYSQL_SYSTEM_DATABASES: Lazy<regex::RegexSet> = Lazy::new(|| {
            RegexSet::new(&[
                "(?i)^mysql$",
                "(?i)^information_schema$",
                "(?i)^performance_schema$",
                "(?i)^sys$",
            ])
            .unwrap()
        });

        let db_name = connection.connection_info().schema_name();

        if MYSQL_SYSTEM_DATABASES.is_match(db_name) {
            return Err(SystemDatabase(db_name.to_owned()).into());
        }

        let version = connection.version().await?;
        let mut circumstances = BitFlags::<Circumstances>::default();

        if let Some(version) = version {
            if version.starts_with("5.6") {
                circumstances |= Circumstances::IsMysql56;
            }

            if version.contains("MariaDB") {
                circumstances |= Circumstances::IsMariadb;
            }
        }

        let result_set = connection.query_raw("SELECT @@lower_case_table_names", &[]).await?;

        if let Some(1) = result_set.into_single().ok().and_then(|row| {
            row.at(0)
                .and_then(|row| row.to_string().and_then(|s| s.parse().ok()).or_else(|| row.as_i64()))
        }) {
            // https://dev.mysql.com/doc/refman/8.0/en/identifier-case-sensitivity.html
            circumstances |= Circumstances::LowerCasesTableNames;
        }

        self.circumstances.store(circumstances.bits(), Ordering::Relaxed);

        Ok(())
    }

    async fn qe_setup(&self, database_str: &str) -> ConnectorResult<()> {
        let mut url = Url::parse(database_str).map_err(|err| ConnectorError::url_parse_error(err, database_str))?;
        url.set_path("/mysql");

        let conn = connect(&url.to_string()).await?;
        let db_name = self.url.dbname();

        let query = format!("DROP DATABASE IF EXISTS `{}`", db_name);
        conn.raw_cmd(&query).await?;

        let query = format!(
            "CREATE DATABASE `{}` CHARACTER SET utf8mb4 COLLATE utf8mb4_unicode_ci;",
            db_name
        );
        conn.raw_cmd(&query).await?;

        Ok(())
    }

    async fn reset(&self, connection: &Connection) -> ConnectorResult<()> {
        let db_name = connection.connection_info().dbname().unwrap();

        connection.raw_cmd(&format!("DROP DATABASE `{}`", db_name)).await?;
        connection.raw_cmd(&format!("CREATE DATABASE `{}`", db_name)).await?;
        connection.raw_cmd(&format!("USE `{}`", db_name)).await?;

        Ok(())
    }

    #[tracing::instrument(skip(self, migrations, connection, connector))]
    async fn sql_schema_from_migration_history(
        &self,
        migrations: &[MigrationDirectory],
        connection: &Connection,
        connector: &SqlMigrationConnector,
    ) -> ConnectorResult<SqlSchema> {
        let temporary_database_name = connector.temporary_database_name();

        let temp_database = self
            .shadow_database_connection(connection, connector, temporary_database_name.as_deref())
            .await?;

        // We go through the whole process without early return, then clean up
        // the temporary database, and only then return the result. This avoids
        // leaving shadow databases behind in case of e.g. faulty migrations.

        let sql_schema_result = (|| async {
            for migration in migrations {
                let script = migration.read_migration_script()?;

                tracing::debug!(
                    "Applying migration `{}` to temporary database.",
                    migration.migration_name()
                );

                temp_database
                    .raw_cmd(&script)
                    .await
                    .map_err(ConnectorError::from)
                    .map_err(|connector_error| {
                        connector_error.into_migration_does_not_apply_cleanly(migration.migration_name().to_owned())
                    })?;
            }

            self.describe_schema(&temp_database).await
        })()
        .await;

        if let Some(database_name) = temporary_database_name {
            let drop_database = format!("DROP DATABASE IF EXISTS `{}`", database_name);
            connection.raw_cmd(&drop_database).await?;
        }

        sql_schema_result
    }

    fn features(&self) -> BitFlags<MigrationFeature> {
        self.features
    }
}

#[derive(BitFlags, Debug, Clone, Copy, PartialEq)]
#[repr(u8)]
pub enum Circumstances {
    LowerCasesTableNames = 0b0001,
    IsMysql56 = 0b0010,
    IsMariadb = 0b0100,
}

fn check_datamodel_for_mysql_5_6(datamodel: &Datamodel, errors: &mut Vec<String>) {
    walk_scalar_fields(datamodel).for_each(|field| {
        if field.field_type().is_json() {
            errors.push(format!(
                "The `Json` data type used in {}.{} is not supported on MySQL 5.6.",
                field.model().name(),
                field.name()
            ))
        }
    });
}
