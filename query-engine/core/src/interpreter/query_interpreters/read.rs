use super::*;
use crate::interpreter::query_interpreters::nested_pagination::NestedPagination;
use crate::{interpreter::InterpretationResult, query_ast::*, result_ast::*};
use connector::{self, ConnectionLike, ReadOperations};
use futures::future::{BoxFuture, FutureExt};
use prisma_models::ManyRecords;

pub fn execute<'a, 'b>(
    tx: &'a ConnectionLike<'a, 'b>,
    query: ReadQuery,
    parent_result: Option<&'a ManyRecords>,
) -> BoxFuture<'a, InterpretationResult<QueryResult>> {
    let fut = async move {
        match query {
            ReadQuery::RecordQuery(q) => read_one(tx, q).await,
            ReadQuery::ManyRecordsQuery(q) => read_many(tx, q).await,
            ReadQuery::RelatedRecordsQuery(q) => read_related(tx, q, parent_result).await,
            ReadQuery::AggregateRecordsQuery(q) => aggregate(tx, q).await,
        }
    };

    fut.boxed()
}

/// Queries a single record.
fn read_one<'conn, 'tx>(
    tx: &'conn ConnectionLike<'conn, 'tx>,
    query: RecordQuery,
) -> BoxFuture<'conn, InterpretationResult<QueryResult>> {
    let fut = async move {
        let model = query.model;
        let model_id = model.primary_identifier();
        let filter = query.filter.expect("Expected filter to be set for ReadOne query.");
        let scalars = tx.get_single_record(&model, &filter, &query.selected_fields).await?;

        match scalars {
            Some(record) => {
                let records: ManyRecords = record.into();
                let nested: Vec<QueryResult> = process_nested(tx, query.nested, Some(&records)).await?;

                Ok(QueryResult::RecordSelection(RecordSelection {
                    name: query.name,
                    fields: query.selection_order,
                    scalars: records,
                    nested,
                    model_id,
                    ..Default::default()
                }))
            }

            None => Ok(QueryResult::RecordSelection(RecordSelection {
                name: query.name,
                fields: query.selection_order,
                model_id,
                ..Default::default()
            })),
        }
    };

    fut.boxed()
}

/// Queries a set of records.
fn read_many<'a, 'b>(
    tx: &'a ConnectionLike<'a, 'b>,
    query: ManyRecordsQuery,
) -> BoxFuture<'a, InterpretationResult<QueryResult>> {
    let fut = async move {
        let scalars = tx
            .get_many_records(&query.model, query.args.clone(), &query.selected_fields)
            .await?;

        let model_id = query.model.primary_identifier();
        let nested: Vec<QueryResult> = process_nested(tx, query.nested, Some(&scalars)).await?;

        Ok(QueryResult::RecordSelection(RecordSelection {
            name: query.name,
            fields: query.selection_order,
            query_arguments: query.args,
            model_id,
            scalars,
            nested,
        }))
    };

    fut.boxed()
}

/// Queries related records for a set of parent IDs.
fn read_related<'a, 'b>(
    tx: &'a ConnectionLike<'a, 'b>,
    mut query: RelatedRecordsQuery,
    parent_result: Option<&'a ManyRecords>,
) -> BoxFuture<'a, InterpretationResult<QueryResult>> {
    let fut = async move {
        let relation = query.parent_field.relation();
        let is_m2m = relation.is_many_to_many();
        let paginator = NestedPagination::new_from_query_args(&query.args);

        query.args.first = None;
        query.args.skip = None;
        query.args.last = None;

        let scalars = if is_m2m {
            nested_read::m2m(tx, &query, parent_result, paginator).await?
        } else {
            nested_read::one2m(
                tx,
                &query.parent_field,
                query.parent_projections,
                parent_result,
                query.args.clone(),
                &query.selected_fields,
                paginator,
            )
            .await?
        };

        let model = query.parent_field.related_model();
        let model_id = model.primary_identifier();
        let nested: Vec<QueryResult> = process_nested(tx, query.nested, Some(&scalars)).await?;

        Ok(QueryResult::RecordSelection(RecordSelection {
            name: query.name,
            fields: query.selection_order,
            query_arguments: query.args,
            model_id,
            scalars,
            nested,
        }))
    };

    fut.boxed()
}

async fn aggregate<'a, 'b>(
    tx: &'a ConnectionLike<'a, 'b>,
    aggregate: AggregateRecordsQuery,
) -> InterpretationResult<QueryResult> {
    let mut results = vec![];

    for query in aggregate.queries {
        match query {
            AggregationQuery::Count(name, args) => {
                let result = tx.count_by_model(&aggregate.model, args).await?;
                results.push(AggregationQueryResult::Count(name, result));
            }
        }
    }

    Ok(QueryResult::RecordAggregation(RecordAggregation {
        fields: aggregate.selection_order,
        results,
    }))
}

fn process_nested<'a, 'b>(
    tx: &'a ConnectionLike<'a, 'b>,
    nested: Vec<ReadQuery>,
    parent_result: Option<&'a ManyRecords>,
) -> BoxFuture<'a, InterpretationResult<Vec<QueryResult>>> {
    let fut = async move {
        let mut results = Vec::with_capacity(nested.len());

        for query in nested {
            let result = execute(tx, query, parent_result).await?;
            results.push(result);
        }

        Ok(results)
    };

    fut.boxed()
}
