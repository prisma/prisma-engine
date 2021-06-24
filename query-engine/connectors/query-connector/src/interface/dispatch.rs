use super::*;
use async_trait::async_trait;
use prisma_value::PrismaValue;

#[async_trait]
impl<'conn, 'tx> ReadOperations for ConnectionLike<'conn, 'tx> {
    async fn get_single_record(
        &mut self,
        model: &ModelRef,
        filter: &Filter,
        selected_fields: &ModelProjection,
        aggr_selections: &[RelAggregationSelection],
    ) -> crate::Result<Option<SingleRecord>> {
        match self {
            Self::Connection(c) => {
                c.get_single_record(model, filter, selected_fields, aggr_selections)
                    .await
            }
            Self::Transaction(tx) => {
                tx.get_single_record(model, filter, selected_fields, aggr_selections)
                    .await
            }
        }
    }

    async fn get_many_records(
        &mut self,
        model: &ModelRef,
        query_arguments: QueryArguments,
        selected_fields: &ModelProjection,
        aggregation_selections: &[RelAggregationSelection],
    ) -> crate::Result<ManyRecords> {
        match self {
            Self::Connection(c) => {
                c.get_many_records(model, query_arguments, selected_fields, aggregation_selections)
                    .await
            }
            Self::Transaction(tx) => {
                tx.get_many_records(model, query_arguments, selected_fields, aggregation_selections)
                    .await
            }
        }
    }

    async fn get_related_m2m_record_ids(
        &mut self,
        from_field: &RelationFieldRef,
        from_record_ids: &[RecordProjection],
    ) -> crate::Result<Vec<(RecordProjection, RecordProjection)>> {
        match self {
            Self::Connection(c) => c.get_related_m2m_record_ids(from_field, from_record_ids).await,
            Self::Transaction(tx) => tx.get_related_m2m_record_ids(from_field, from_record_ids).await,
        }
    }

    async fn aggregate_records(
        &mut self,
        model: &ModelRef,
        query_arguments: QueryArguments,
        selections: Vec<AggregationSelection>,
        group_by: Vec<ScalarFieldRef>,
        having: Option<Filter>,
    ) -> crate::Result<Vec<AggregationRow>> {
        match self {
            Self::Connection(c) => {
                c.aggregate_records(model, query_arguments, selections, group_by, having)
                    .await
            }
            Self::Transaction(tx) => {
                tx.aggregate_records(model, query_arguments, selections, group_by, having)
                    .await
            }
        }
    }
}

#[async_trait]
impl<'conn, 'tx> WriteOperations for ConnectionLike<'conn, 'tx> {
    async fn create_record(&mut self, model: &ModelRef, args: WriteArgs) -> crate::Result<RecordProjection> {
        match self {
            Self::Connection(c) => c.create_record(model, args).await,
            Self::Transaction(tx) => tx.create_record(model, args).await,
        }
    }

    async fn create_records(
        &mut self,
        model: &ModelRef,
        args: Vec<WriteArgs>,
        skip_duplicates: bool,
    ) -> crate::Result<usize> {
        match self {
            Self::Connection(c) => c.create_records(model, args, skip_duplicates).await,
            Self::Transaction(tx) => tx.create_records(model, args, skip_duplicates).await,
        }
    }

    async fn update_records(
        &mut self,
        model: &ModelRef,
        record_filter: RecordFilter,
        args: WriteArgs,
    ) -> crate::Result<Vec<RecordProjection>> {
        match self {
            Self::Connection(c) => c.update_records(model, record_filter, args).await,
            Self::Transaction(tx) => tx.update_records(model, record_filter, args).await,
        }
    }

    async fn delete_records(&mut self, model: &ModelRef, record_filter: RecordFilter) -> crate::Result<usize> {
        match self {
            Self::Connection(c) => c.delete_records(model, record_filter).await,
            Self::Transaction(tx) => tx.delete_records(model, record_filter).await,
        }
    }

    async fn m2m_connect(
        &mut self,
        field: &RelationFieldRef,
        parent_id: &RecordProjection,
        child_ids: &[RecordProjection],
    ) -> crate::Result<()> {
        match self {
            Self::Connection(c) => c.m2m_connect(field, parent_id, child_ids).await,
            Self::Transaction(tx) => tx.m2m_connect(field, parent_id, child_ids).await,
        }
    }

    async fn m2m_disconnect(
        &mut self,
        field: &RelationFieldRef,
        parent_id: &RecordProjection,
        child_ids: &[RecordProjection],
    ) -> crate::Result<()> {
        match self {
            Self::Connection(c) => c.m2m_disconnect(field, parent_id, child_ids).await,
            Self::Transaction(tx) => tx.m2m_disconnect(field, parent_id, child_ids).await,
        }
    }

    async fn query_raw(&mut self, query: String, parameters: Vec<PrismaValue>) -> crate::Result<serde_json::Value> {
        match self {
            Self::Connection(c) => c.query_raw(query, parameters).await,
            Self::Transaction(tx) => tx.query_raw(query, parameters).await,
        }
    }

    async fn execute_raw(&mut self, query: String, parameters: Vec<PrismaValue>) -> crate::Result<usize> {
        match self {
            Self::Connection(c) => c.execute_raw(query, parameters).await,
            Self::Transaction(tx) => tx.execute_raw(query, parameters).await,
        }
    }
}
