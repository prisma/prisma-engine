use crate::{
    interpreter::{InterpretationResult, InterpreterError},
    query_ast::*,
    QueryResult,
};
use connector::{ConnectionLike, Filter, WriteOperations};
use prisma_value::PrismaValue;

pub async fn execute<'a, 'b>(
    tx: &'a ConnectionLike<'a, 'b>,
    write_query: WriteQuery,
) -> InterpretationResult<QueryResult> {
    match write_query {
        WriteQuery::CreateRecord(q) => create_one(tx, q).await,
        WriteQuery::UpdateRecord(q) => update_one(tx, q).await,
        WriteQuery::DeleteRecord(q) => delete_one(tx, q).await,
        WriteQuery::UpdateManyRecords(q) => update_many(tx, q).await,
        WriteQuery::DeleteManyRecords(q) => delete_many(tx, q).await,
        WriteQuery::ConnectRecords(q) => connect(tx, q).await,
        WriteQuery::DisconnectRecords(q) => disconnect(tx, q).await,
        WriteQuery::Raw { query, parameters } => execute_raw(tx, query, parameters).await,
    }
}

async fn execute_raw<'a, 'b>(
    tx: &'a ConnectionLike<'a, 'b>,
    query: String,
    parameters: Vec<PrismaValue>,
) -> InterpretationResult<QueryResult> {
    let res = tx.execute_raw(query, parameters).await?;
    Ok(QueryResult::Json(res))
}

async fn create_one<'a, 'b>(tx: &'a ConnectionLike<'a, 'b>, q: CreateRecord) -> InterpretationResult<QueryResult> {
    let res = tx.create_record(&q.model, q.args).await?;

    Ok(QueryResult::Id(Some(res)))
}

async fn update_one<'a, 'b>(tx: &'a ConnectionLike<'a, 'b>, q: UpdateRecord) -> InterpretationResult<QueryResult> {
    let mut res = tx.update_records(&q.model, Filter::from(q.where_), q.args).await?;

    Ok(QueryResult::Id(res.pop()))
}

async fn delete_one<'a, 'b>(tx: &'a ConnectionLike<'a, 'b>, q: DeleteRecord) -> InterpretationResult<QueryResult> {
    // We need to ensure that we have a record finder, else we delete everything (conversion to empty filter).
    let finder = match q.where_ {
        Some(f) => Ok(f),
        None => Err(InterpreterError::InterpretationError(
            "No record finder specified for delete record operation. Aborting.".to_owned(),
            None,
        )),
    }?;

    let res = tx.delete_records(&q.model, Filter::from(finder)).await?;

    Ok(QueryResult::Count(res))
}

async fn update_many<'a, 'b>(
    tx: &'a ConnectionLike<'a, 'b>,
    q: UpdateManyRecords,
) -> InterpretationResult<QueryResult> {
    let res = tx.update_records(&q.model, q.filter, q.args).await?;

    Ok(QueryResult::Count(res.len()))
}

async fn delete_many<'a, 'b>(
    tx: &'a ConnectionLike<'a, 'b>,
    q: DeleteManyRecords,
) -> InterpretationResult<QueryResult> {
    let res = tx.delete_records(&q.model, q.filter).await?;

    Ok(QueryResult::Count(res))
}

async fn connect<'a, 'b>(tx: &'a ConnectionLike<'a, 'b>, q: ConnectRecords) -> InterpretationResult<QueryResult> {
    tx.connect(
        &q.relation_field,
        &q.parent_id.expect("Expected parent record ID to be set for connect"),
        &q.child_ids,
    )
    .await?;

    Ok(QueryResult::Unit)
}

async fn disconnect<'a, 'b>(tx: &'a ConnectionLike<'a, 'b>, q: DisconnectRecords) -> InterpretationResult<QueryResult> {
    tx.disconnect(
        &q.relation_field,
        &q.parent_id.expect("Expected parent record ID to be set for disconnect"),
        &q.child_ids,
    )
    .await?;

    Ok(QueryResult::Unit)
}
