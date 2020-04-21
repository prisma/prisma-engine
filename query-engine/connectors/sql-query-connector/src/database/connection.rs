use super::transaction::SqlConnectorTransaction;
use crate::{database::operations::*, QueryExt, SqlError};
use connector_interface::{
    self as connector, filter::Filter, Connection, QueryArguments, ReadOperations, RecordFilter, Transaction,
    WriteArgs, WriteOperations, IO,
};
use prisma_models::prelude::*;
use prisma_value::PrismaValue;
use quaint::{connector::TransactionCapable, prelude::ConnectionInfo};

pub struct SqlConnection<'a, C> {
    inner: C,
    connection_info: &'a ConnectionInfo,
}

impl<'a, C> SqlConnection<'a, C>
where
    C: QueryExt + Send + Sync + 'static,
{
    pub fn new(inner: C, connection_info: &'a ConnectionInfo) -> Self {
        Self { inner, connection_info }
    }

    async fn catch<O>(
        &self,
        fut: impl std::future::Future<Output = Result<O, SqlError>>,
    ) -> Result<O, connector_interface::error::ConnectorError> {
        match fut.await {
            Ok(o) => Ok(o),
            Err(err) => Err(err.into_connector_error(&self.connection_info)),
        }
    }
}

impl<'conninfo, C> Connection for SqlConnection<'conninfo, C>
where
    C: QueryExt + TransactionCapable + Send + Sync + 'static,
{
    fn start_transaction<'a>(&'a self) -> IO<'a, Box<dyn Transaction<'a> + 'a>> {
        let fut_tx = self.inner.start_transaction();
        let connection_info = self.connection_info;

        IO::new(self.catch(async move {
            let tx: quaint::connector::Transaction<'a> = fut_tx.await.map_err(SqlError::from)?;
            Ok(Box::new(SqlConnectorTransaction::new(tx, connection_info)) as Box<dyn Transaction<'a> + 'a>)
        }))
    }
}

impl<'a, C> ReadOperations for SqlConnection<'a, C>
where
    C: QueryExt + Send + Sync + 'static,
{
    fn get_single_record<'b>(
        &'b self,
        model: &'b ModelRef,
        filter: &'b Filter,
        selected_fields: &'b ModelProjection,
    ) -> connector::IO<'b, Option<SingleRecord>> {
        IO::new(self.catch(async move { read::get_single_record(&self.inner, model, filter, selected_fields).await }))
    }

    fn get_many_records<'b>(
        &'b self,
        model: &'b ModelRef,
        query_arguments: QueryArguments,
        selected_fields: &'b ModelProjection,
    ) -> connector::IO<'b, ManyRecords> {
        IO::new(
            self.catch(
                async move { read::get_many_records(&self.inner, model, query_arguments, selected_fields).await },
            ),
        )
    }

    fn get_related_m2m_record_ids<'b>(
        &'b self,
        from_field: &'b RelationFieldRef,
        from_record_ids: &'b [RecordProjection],
    ) -> connector::IO<'b, Vec<(RecordProjection, RecordProjection)>> {
        IO::new(
            self.catch(async move { read::get_related_m2m_record_ids(&self.inner, from_field, from_record_ids).await }),
        )
    }

    fn count_by_model<'b>(&'b self, model: &'b ModelRef, query_arguments: QueryArguments) -> connector::IO<'b, usize> {
        IO::new(self.catch(async move { read::count_by_model(&self.inner, model, query_arguments).await }))
    }
}

impl<'conn, C> WriteOperations for SqlConnection<'conn, C>
where
    C: QueryExt + Send + Sync + 'static,
{
    fn create_record<'a>(&'a self, model: &'a ModelRef, args: WriteArgs) -> connector::IO<RecordProjection> {
        IO::new(self.catch(async move { write::create_record(&self.inner, model, args).await }))
    }

    fn update_records<'a>(
        &'a self,
        model: &'a ModelRef,
        record_filter: RecordFilter,
        args: WriteArgs,
    ) -> connector::IO<Vec<RecordProjection>> {
        IO::new(self.catch(async move { write::update_records(&self.inner, model, record_filter, args).await }))
    }

    fn delete_records<'a>(&'a self, model: &'a ModelRef, record_filter: RecordFilter) -> connector::IO<usize> {
        IO::new(self.catch(async move { write::delete_records(&self.inner, model, record_filter).await }))
    }

    fn connect<'a>(
        &'a self,
        field: &'a RelationFieldRef,
        parent_id: &'a RecordProjection,
        child_ids: &'a [RecordProjection],
    ) -> connector::IO<()> {
        IO::new(self.catch(async move { write::connect(&self.inner, field, parent_id, child_ids).await }))
    }

    fn disconnect<'a>(
        &'a self,
        field: &'a RelationFieldRef,
        parent_id: &'a RecordProjection,
        child_ids: &'a [RecordProjection],
    ) -> connector::IO<()> {
        IO::new(self.catch(async move { write::disconnect(&self.inner, field, parent_id, child_ids).await }))
    }

    fn execute_raw<'a>(&'a self, query: String, parameters: Vec<PrismaValue>) -> connector::IO<serde_json::Value> {
        IO::new(self.catch(async move { write::execute_raw(&self.inner, query, parameters).await }))
    }
}
