use super::*;
use crate::{
    error::MongoError,
    queries::{aggregate, read, write},
};
use connector_interface::{ReadOperations, RelAggregationSelection, Transaction, WriteOperations};
use futures::Future;
use mongodb::Database;

/// Not really a transaction right now, just something to
/// satisfy the core interface until we figure something out.
pub struct MongoDbTransaction {
    /// Handle to a mongo database.
    pub(crate) database: Database,
}

impl MongoDbTransaction {
    pub(crate) fn new(database: Database) -> Self {
        Self { database }
    }

    async fn catch<O>(
        &self,
        fut: impl Future<Output = Result<O, MongoError>>,
    ) -> Result<O, connector_interface::error::ConnectorError> {
        match fut.await {
            Ok(o) => Ok(o),
            Err(err) => Err(err.into_connector_error()),
        }
    }
}

#[async_trait]
impl Transaction for MongoDbTransaction {
    async fn commit(&self) -> connector_interface::Result<()> {
        // Totally commited.
        Ok(())
    }

    async fn rollback(&self) -> connector_interface::Result<()> {
        // Totally rolled back.
        Ok(())
    }
}

#[async_trait]
impl WriteOperations for MongoDbTransaction {
    async fn create_record(
        &self,
        model: &ModelRef,
        args: connector_interface::WriteArgs,
    ) -> connector_interface::Result<RecordProjection> {
        self.catch(async move { write::create_record(&self.database, model, args).await })
            .await
    }

    async fn create_records(
        &self,
        model: &ModelRef,
        args: Vec<connector_interface::WriteArgs>,
        skip_duplicates: bool,
    ) -> connector_interface::Result<usize> {
        self.catch(async move { write::create_records(&self.database, model, args, skip_duplicates).await })
            .await
    }

    async fn update_records(
        &self,
        model: &ModelRef,
        record_filter: connector_interface::RecordFilter,
        args: connector_interface::WriteArgs,
    ) -> connector_interface::Result<Vec<RecordProjection>> {
        self.catch(async move { write::update_records(&self.database, model, record_filter, args).await })
            .await
    }

    async fn delete_records(
        &self,
        model: &ModelRef,
        record_filter: connector_interface::RecordFilter,
    ) -> connector_interface::Result<usize> {
        self.catch(async move { write::delete_records(&self.database, model, record_filter).await })
            .await
    }

    async fn m2m_connect(
        &self,
        field: &RelationFieldRef,
        parent_id: &RecordProjection,
        child_ids: &[RecordProjection],
    ) -> connector_interface::Result<()> {
        self.catch(async move { write::m2m_connect(&self.database, field, parent_id, child_ids).await })
            .await
    }

    async fn m2m_disconnect(
        &self,
        field: &RelationFieldRef,
        parent_id: &RecordProjection,
        child_ids: &[RecordProjection],
    ) -> connector_interface::Result<()> {
        self.catch(async move { write::m2m_disconnect(&self.database, field, parent_id, child_ids).await })
            .await
    }

    async fn execute_raw(
        &self,
        _query: String,
        _parameters: Vec<prisma_value::PrismaValue>,
    ) -> connector_interface::Result<usize> {
        Err(MongoError::Unsupported("Raw queries".to_owned()).into_connector_error())
    }

    async fn query_raw(
        &self,
        _query: String,
        _parameters: Vec<prisma_value::PrismaValue>,
    ) -> connector_interface::Result<serde_json::Value> {
        Err(MongoError::Unsupported("Raw queries".to_owned()).into_connector_error())
    }
}

#[async_trait]
impl ReadOperations for MongoDbTransaction {
    async fn get_single_record(
        &self,
        model: &ModelRef,
        filter: &connector_interface::Filter,
        selected_fields: &ModelProjection,
    ) -> connector_interface::Result<Option<SingleRecord>> {
        self.catch(async move { read::get_single_record(&self.database, model, filter, selected_fields).await })
            .await
    }

    async fn get_many_records(
        &self,
        model: &ModelRef,
        query_arguments: connector_interface::QueryArguments,
        selected_fields: &ModelProjection,
        aggregation_selections: &[RelAggregationSelection],
    ) -> connector_interface::Result<ManyRecords> {
        self.catch(async move {
            read::get_many_records(
                &self.database,
                model,
                query_arguments,
                selected_fields,
                aggregation_selections,
            )
            .await
        })
        .await
    }

    async fn get_related_m2m_record_ids(
        &self,
        from_field: &RelationFieldRef,
        from_record_ids: &[RecordProjection],
    ) -> connector_interface::Result<Vec<(RecordProjection, RecordProjection)>> {
        self.catch(async move { read::get_related_m2m_record_ids(&self.database, from_field, from_record_ids).await })
            .await
    }

    async fn aggregate_records(
        &self,
        model: &ModelRef,
        query_arguments: connector_interface::QueryArguments,
        selections: Vec<connector_interface::AggregationSelection>,
        group_by: Vec<ScalarFieldRef>,
        having: Option<connector_interface::Filter>,
    ) -> connector_interface::Result<Vec<connector_interface::AggregationRow>> {
        self.catch(async move {
            aggregate::aggregate(&self.database, model, query_arguments, selections, group_by, having).await
        })
        .await
    }
}
