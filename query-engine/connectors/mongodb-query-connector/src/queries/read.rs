use super::*;
use crate::{filter::convert_filter, query_arguments::MongoQueryArgs};
use crate::{BsonTransform, IntoBson};
use connector_interface::{Filter, QueryArguments};
use mongodb::Database;
use mongodb::{bson::doc, options::FindOptions};
use prisma_models::*;

pub async fn get_single_record(
    database: &Database,
    model: &ModelRef,
    filter: &Filter,
    selected_fields: &ModelProjection,
) -> crate::Result<Option<SingleRecord>> {
    let coll = database.collection(model.db_name());
    let (filter, _) = convert_filter(filter.clone(), false)?.render();
    let find_options = FindOptions::builder()
        .projection(selected_fields.clone().into_bson()?.into_document()?)
        .build();

    let cursor = coll.find(Some(filter), Some(find_options)).await?;
    let docs = vacuum_cursor(cursor).await?;

    if docs.is_empty() {
        Ok(None)
    } else {
        let field_names: Vec<_> = selected_fields.db_names().collect();
        let doc = docs.into_iter().next().unwrap();
        let record = document_to_record(doc, &field_names)?;

        Ok(Some(SingleRecord { record, field_names }))
    }
}

// Checklist:
// - [x] OrderBy scalar.
// - [ ] OrderBy relation.
// - [x] Skip, take
// - [ ] Cursor
// - [x] Distinct select (inherently given from core).
pub async fn get_many_records(
    database: &Database,
    model: &ModelRef,
    query_arguments: QueryArguments,
    selected_fields: &ModelProjection,
) -> crate::Result<ManyRecords> {
    let coll = database.collection(model.db_name());
    let reverse_order = query_arguments.take.map(|t| t < 0).unwrap_or(false);
    let field_names: Vec<_> = selected_fields.db_names().collect();
    let mut records = ManyRecords::new(field_names.clone());

    if let Some(0) = query_arguments.take {
        return Ok(records);
    };

    let mongo_args = MongoQueryArgs::new(query_arguments)?.with_model_projection(selected_fields.clone())?;
    let cursor = mongo_args.find_documents(coll).await?;
    let docs = vacuum_cursor(cursor).await?;

    for doc in docs {
        let record = document_to_record(doc, &field_names)?;
        records.push(record)
    }

    if reverse_order {
        records.reverse();
    }

    Ok(records)
}

pub async fn get_related_m2m_record_ids(
    database: &Database,
    from_field: &RelationFieldRef,
    from_record_ids: &[RecordProjection],
) -> crate::Result<Vec<(RecordProjection, RecordProjection)>> {
    if from_record_ids.is_empty() {
        return Ok(vec![]);
    }

    let model = from_field.model();
    let coll = database.collection(model.db_name());

    let id_field = model.primary_identifier().scalar_fields().next().unwrap();
    let ids = from_record_ids
        .iter()
        .map(|p| (&id_field, p.values().next().unwrap()).into_bson())
        .collect::<crate::Result<Vec<_>>>()?;

    let filter = doc! { id_field.db_name(): { "$in": ids } };

    // Scalar field name where the relation ids list is on `model`.
    let relation_ids_field_name = from_field.relation_info.fields.get(0).unwrap();

    let find_options = FindOptions::builder()
        .projection(doc! { id_field.db_name(): 1, relation_ids_field_name: 1 })
        .build();

    let cursor = coll.find(filter, Some(find_options)).await?;
    let docs = vacuum_cursor(cursor).await?;

    let child_id_field = from_field
        .related_model()
        .primary_identifier()
        .scalar_fields()
        .next()
        .unwrap();

    let mut id_pairs = vec![];
    for mut doc in docs {
        let parent_id = value_from_bson(doc.remove(id_field.db_name()).unwrap())?;
        let child_ids: Vec<PrismaValue> = match value_from_bson(
            doc.remove(relation_ids_field_name)
                .unwrap_or_else(|| Bson::Array(vec![])),
        )? {
            PrismaValue::List(vals) => vals,
            val => vec![val],
        };

        let parent_projection = RecordProjection::from((id_field.clone(), parent_id));

        for child_id in child_ids {
            let child_projection = RecordProjection::from((child_id_field.clone(), child_id));
            id_pairs.push((parent_projection.clone(), child_projection));
        }
    }

    Ok(id_pairs)
}
