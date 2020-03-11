use crate::{
    context::PrismaContext,
    request_handlers::{GraphQlBody, GraphQlRequestHandler, RequestHandler, SingleQuery},
    PrismaRequest, PrismaResponse,
};
use migration_connector::*;
use migration_core::{
    api::{GenericApi, MigrationApi},
    commands::{ApplyMigrationInput, InferMigrationStepsInput, ResetCommand},
};
use quaint::{
    ast::*,
    connector::ConnectionInfo,
    visitor::{self, Visitor},
};
use sql_migration_connector::SqlMigrationConnector;
use std::{collections::HashMap, sync::Arc};
use test_setup::*;

pub struct QueryEngine {
    context: Arc<PrismaContext>,
}

impl QueryEngine {
    pub fn new(ctx: PrismaContext) -> Self {
        QueryEngine { context: Arc::new(ctx) }
    }

    pub async fn request(&self, body: impl Into<SingleQuery>) -> serde_json::Value {
        let request = PrismaRequest {
            body: GraphQlBody::Single(body.into()),
            headers: HashMap::new(),
            path: String::new(),
        };

        match GraphQlRequestHandler.handle(request, &self.context).await {
            PrismaResponse::Single(response) => serde_json::to_value(response).unwrap(),
            _ => unreachable!(),
        }
    }
}

pub struct TestApi {
    connection_info: ConnectionInfo,
    migration_api: MigrationApi<sql_migration_connector::SqlMigrationConnector, sql_migration_connector::SqlMigration>,
    config: String,
    is_pgbouncer: bool,
}

impl TestApi {
    pub async fn create_engine(&self, datamodel: &str) -> anyhow::Result<QueryEngine> {
        let datamodel_string = format!("{}\n\n{}", self.config, datamodel);
        let migration_id = "test-cli-migration".to_owned();

        let infer_input = InferMigrationStepsInput {
            assume_applied_migrations: Some(Vec::new()),
            assume_to_be_applied: Some(Vec::new()),
            datamodel: datamodel_string.clone(),
            migration_id: migration_id.clone(),
        };

        self.migration_api.reset(&serde_json::Value::Null).await?;
        let result = self.migration_api.infer_migration_steps(&infer_input).await?;

        let apply_input = ApplyMigrationInput {
            force: Some(true),
            migration_id,
            steps: result.datamodel_steps,
        };

        self.migration_api.apply_migration(&apply_input).await?;

        let context = PrismaContext::builder()
            .enable_raw_queries(true)
            .datamodel(datamodel_string)
            .force_transactions(self.is_pgbouncer)
            .build()
            .await
            .unwrap();

        Ok(QueryEngine {
            context: Arc::new(context),
        })
    }

    pub fn connection_info(&self) -> &ConnectionInfo {
        &self.connection_info
    }

    pub fn to_sql_string<'a>(&'a self, query: impl Into<Query<'a>>) -> (String, Vec<ParameterizedValue>) {
        match self.connection_info() {
            ConnectionInfo::Postgres(..) => visitor::Postgres::build(query),
            ConnectionInfo::Mysql(..) => visitor::Mysql::build(query),
            ConnectionInfo::Sqlite { .. } => visitor::Sqlite::build(query),
        }
    }
}

pub async fn migration_api<C, D>(connector: C) -> MigrationApi<C, D>
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

pub async fn mysql_8_test_api(db_name: &str) -> TestApi {
    let url = mysql_8_url(db_name);
    let connection_info = ConnectionInfo::from_url(&url).unwrap();

    let connector = mysql_migration_connector(&url).await;
    let migration_api = migration_api(connector).await;

    let config = mysql_8_test_config(db_name);

    TestApi {
        connection_info,
        migration_api,
        config,
        is_pgbouncer: false,
    }
}

pub async fn mysql_test_api(db_name: &str) -> TestApi {
    let url = mysql_url(db_name);
    let connection_info = ConnectionInfo::from_url(&url).unwrap();

    let connector = mysql_migration_connector(&url).await;
    let migration_api = migration_api(connector).await;

    let config = mysql_test_config(db_name);

    TestApi {
        connection_info,
        migration_api,
        config,
        is_pgbouncer: false,
    }
}

pub async fn mysql_mariadb_test_api(db_name: &str) -> TestApi {
    let url = mariadb_url(db_name);
    let connection_info = ConnectionInfo::from_url(&url).unwrap();

    let connector = mysql_migration_connector(&url).await;
    let migration_api = migration_api(connector).await;

    let config = mariadb_test_config(db_name);

    TestApi {
        connection_info,
        migration_api,
        config,
        is_pgbouncer: false,
    }
}

pub async fn postgres9_test_api(db_name: &str) -> TestApi {
    let url = postgres_9_url(db_name);
    let connection_info = ConnectionInfo::from_url(&url).unwrap();

    let connector = postgres_migration_connector(&url).await;
    let migration_api = migration_api(connector).await;

    let config = postgres_9_test_config(db_name);

    TestApi {
        connection_info,
        migration_api,
        config,
        is_pgbouncer: false,
    }
}

pub async fn postgres_test_api(db_name: &str) -> TestApi {
    let url = postgres_10_url(db_name);
    let connection_info = ConnectionInfo::from_url(&url).unwrap();

    let connector = postgres_migration_connector(&url).await;
    let migration_api = migration_api(connector).await;

    let config = postgres_10_test_config(db_name);

    TestApi {
        connection_info,
        migration_api,
        config,
        is_pgbouncer: false,
    }
}

pub async fn postgres11_test_api(db_name: &str) -> TestApi {
    let url = postgres_11_url(db_name);
    let connection_info = ConnectionInfo::from_url(&url).unwrap();

    let connector = postgres_migration_connector(&url).await;
    let migration_api = migration_api(connector).await;

    let config = pgbouncer_test_config(db_name);

    TestApi {
        connection_info,
        migration_api,
        config,
        is_pgbouncer: true,
    }
}

pub async fn postgres12_test_api(db_name: &str) -> TestApi {
    let url = postgres_12_url(db_name);
    let connection_info = ConnectionInfo::from_url(&url).unwrap();

    let connector = postgres_migration_connector(&url).await;
    let migration_api = migration_api(connector).await;

    let config = postgres_12_test_config(db_name);

    TestApi {
        connection_info,
        migration_api,
        config,
        is_pgbouncer: false,
    }
}

pub async fn sqlite_test_api(db_name: &str) -> TestApi {
    let url = sqlite_test_url(db_name);
    let connection_info = ConnectionInfo::from_url(&url).unwrap();

    let connector = sqlite_migration_connector(db_name).await;
    let migration_api = migration_api(connector).await;

    let config = sqlite_test_config(db_name);

    TestApi {
        connection_info,
        migration_api,
        config,
        is_pgbouncer: false,
    }
}

pub(super) async fn mysql_migration_connector(url_str: &str) -> SqlMigrationConnector {
    match SqlMigrationConnector::new(url_str, "mysql").await {
        Ok(c) => c,
        Err(_) => {
            create_mysql_database(&url_str.parse().unwrap()).await.unwrap();
            SqlMigrationConnector::new(url_str, "mysql").await.unwrap()
        }
    }
}

pub(super) async fn postgres_migration_connector(url_str: &str) -> SqlMigrationConnector {
    match SqlMigrationConnector::new(url_str, "postgresql").await {
        Ok(c) => c,
        Err(_) => {
            create_postgres_database(&url_str.parse().unwrap()).await.unwrap();
            SqlMigrationConnector::new(url_str, "postgresql").await.unwrap()
        }
    }
}

pub(super) async fn sqlite_migration_connector(db_name: &str) -> SqlMigrationConnector {
    SqlMigrationConnector::new(&sqlite_test_url(db_name), "sqlite")
        .await
        .unwrap()
}
