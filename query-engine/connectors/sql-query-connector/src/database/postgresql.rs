use super::connection::SqlConnection;
use crate::{query_builder::ManyRelatedRecordsWithRowNumber, FromSource, QueryExt, SqlError};
use connector_interface::{Connection, Connector, IO};
use datamodel::Source;
use quaint::pool::{self, PostgresManager};
use tokio_resource_pool::{CheckOut, Pool};

pub struct PostgreSql {
    pool: Pool<PostgresManager>,
}

impl QueryExt for CheckOut<PostgresManager> {}

impl FromSource for PostgreSql {
    fn from_source(source: &dyn Source) -> crate::Result<Self> {
        let url = url::Url::parse(&source.url().value)?;
        let pool = pool::postgres(url)?;

        Ok(PostgreSql { pool })
    }
}

impl Connector for PostgreSql {
    fn get_connection<'a>(&'a self) -> IO<Box<dyn Connection + 'a>> {
        IO::new(async move {
            let conn = self.pool.check_out().await.map_err(SqlError::from)?;
            let conn = SqlConnection::<_, ManyRelatedRecordsWithRowNumber>::new(conn);

            Ok(Box::new(conn) as Box<dyn Connection>)
        })
    }
}
