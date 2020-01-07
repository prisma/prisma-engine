use crate::{api::MigrationApi, commands::ResetCommand};
use datamodel::ast::{parser, SchemaAst};
use migration_connector::*;
use sql_migration_connector::SqlMigrationConnector;
use test_setup::*;

pub type TestResult = Result<(), anyhow::Error>;

pub fn parse(datamodel_string: &str) -> SchemaAst {
    parser::parse(datamodel_string).unwrap()
}

pub(super) async fn mysql_migration_connector(url_str: &str) -> SqlMigrationConnector {
    match SqlMigrationConnector::new(url_str).await {
        Ok(c) => c,
        Err(_) => {
            create_mysql_database(&url_str.parse().unwrap()).await.unwrap();
            SqlMigrationConnector::new(url_str).await.unwrap()
        }
    }
}

pub(super) async fn postgres_migration_connector(url_str: &str) -> SqlMigrationConnector {
    match SqlMigrationConnector::new(url_str).await {
        Ok(c) => c,
        Err(_) => {
            create_postgres_database(&url_str.parse().unwrap()).await.unwrap();
            SqlMigrationConnector::new(url_str).await.unwrap()
        }
    }
}

pub(super) async fn sqlite_migration_connector(db_name: &str) -> SqlMigrationConnector {
    SqlMigrationConnector::new(&sqlite_test_url(db_name)).await.unwrap()
}

pub async fn test_api<C, D>(connector: C) -> MigrationApi<C, D>
where
    C: MigrationConnector<DatabaseMigration = D>,
    D: DatabaseMigrationMarker + Send + Sync + 'static,
{
    let api = MigrationApi::new(connector).await.unwrap();

    api.handle_command::<ResetCommand>(&serde_json::Value::Null)
        .await
        .expect("Engine reset failed");

    api
}
