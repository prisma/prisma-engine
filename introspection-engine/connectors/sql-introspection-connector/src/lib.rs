pub mod calculate_datamodel; // only exported to be able to unit test it
mod commenting_out_guardrails;
mod error;
mod misc_helpers;
mod prisma_1_defaults;
mod sanitize_datamodel_names;
mod schema_describer_loading;
mod version_checker;

use introspection_connector::{
    ConnectorError, ConnectorResult, DatabaseMetadata, IntrospectionConnector, IntrospectionResult,
};
use quaint::prelude::ConnectionInfo;
use sql_schema_describer::{SqlSchema, SqlSchemaDescriberBackend};
use std::future::Future;
use tracing_futures::Instrument;

pub use error::*;

pub type SqlIntrospectionResult<T> = core::result::Result<T, SqlError>;

pub struct SqlIntrospectionConnector {
    connection_info: ConnectionInfo,
    describer: Box<dyn SqlSchemaDescriberBackend>,
}

impl SqlIntrospectionConnector {
    pub async fn new(url: &str) -> ConnectorResult<SqlIntrospectionConnector> {
        let (describer, connection_info) = schema_describer_loading::load_describer(&url)
            .instrument(tracing::debug_span!("Loading describer"))
            .await
            .map_err(|error| {
                ConnectionInfo::from_url(url)
                    .map(|connection_info| error.into_connector_error(&connection_info))
                    .unwrap_or_else(|err| ConnectorError::url_parse_error(err, url))
            })?;

        tracing::debug!("SqlIntrospectionConnector initialized.");

        Ok(SqlIntrospectionConnector {
            describer,
            connection_info,
        })
    }

    async fn catch<O>(&self, fut: impl Future<Output = Result<O, SqlError>>) -> ConnectorResult<O> {
        fut.await
            .map_err(|sql_introspection_error| sql_introspection_error.into_connector_error(&self.connection_info))
    }

    async fn list_databases_internal(&self) -> SqlIntrospectionResult<Vec<String>> {
        Ok(self.describer.list_databases().await?)
    }

    async fn get_metadata_internal(&self) -> SqlIntrospectionResult<DatabaseMetadata> {
        let sql_metadata = self.describer.get_metadata(self.connection_info.schema_name()).await?;
        let db_metadate = DatabaseMetadata {
            table_count: sql_metadata.table_count,
            size_in_bytes: sql_metadata.size_in_bytes,
        };
        Ok(db_metadate)
    }

    async fn describe(&self) -> SqlIntrospectionResult<SqlSchema> {
        Ok(self.describer.describe(self.connection_info.schema_name()).await?)
    }
}

#[async_trait::async_trait]
impl IntrospectionConnector for SqlIntrospectionConnector {
    async fn list_databases(&self) -> ConnectorResult<Vec<String>> {
        Ok(self.catch(self.list_databases_internal()).await?)
    }

    async fn get_metadata(&self) -> ConnectorResult<DatabaseMetadata> {
        Ok(self.catch(self.get_metadata_internal()).await?)
    }

    async fn get_database_description(&self) -> ConnectorResult<String> {
        let sql_schema = self.catch(self.describe()).await?;
        tracing::debug!("SQL Schema Describer is done: {:?}", sql_schema);
        let description = serde_json::to_string(&sql_schema).unwrap();
        Ok(description)
    }

    async fn introspect(&self) -> ConnectorResult<IntrospectionResult> {
        let sql_schema = self.catch(self.describe()).await?;
        tracing::debug!("SQL Schema Describer is done: {:?}", sql_schema);

        let family = self.connection_info.sql_family();

        let introspection_result = calculate_datamodel::calculate_datamodel(&sql_schema, &family)
            .map_err(|sql_introspection_error| sql_introspection_error.into_connector_error(&self.connection_info))?;

        tracing::debug!("Calculating datamodel is done: {:?}", sql_schema);
        Ok(introspection_result)
    }
}

// async fn introspect(&self) -> ConnectorResult<IntrospectionResult> {
//     use std::panic;
//     let sql_schema = self.catch(self.describe()).await?;
//     tracing::debug!("SQL Schema Describer is done: {:?}", sql_schema);
//
//     let family = self.connection_info.sql_family();
//
//     let prev_hook = panic::take_hook();
//     panic::set_hook(Box::new(|_| {}));
//     let result = panic::catch_unwind(|| calculate_datamodel::calculate_datamodel(&sql_schema, &family).unwrap());
//     panic::set_hook(prev_hook);
//
//     pub fn extract_panic_message(panic_payload: &(dyn std::any::Any + Send + 'static)) -> Option<String> {
//         panic_payload
//             .downcast_ref::<&str>()
//             .map(|s| -> String { (*s).to_owned() })
//             .or_else(|| panic_payload.downcast_ref::<String>().map(|s| s.to_owned()))
//     }
//
//     match result {
//         Ok(res) => {
//             tracing::debug!("Calculating datamodel is done: {:?}", sql_schema);
//             Ok(res)
//         }
//         Err(introspection_error) => Err(ConnectorError::from_kind(ErrorKind::DatabaseSchemaInconsistent {
//             explanation: extract_panic_message(introspection_error.as_ref()).unwrap(),
//         })),
//     }
// }
