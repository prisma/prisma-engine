//! Prisma read query AST
use super::RecordFinderInjector;
use connector::{filter::RecordFinder, QueryArguments};
use prisma_models::prelude::*;
use std::fmt::Display;

#[derive(Debug, Clone)]
pub enum ReadQuery {
    RecordQuery(RecordQuery),
    ManyRecordsQuery(ManyRecordsQuery),
    RelatedRecordsQuery(RelatedRecordsQuery),
    AggregateRecordsQuery(AggregateRecordsQuery),
}

impl ReadQuery {
    pub fn name(&self) -> &str {
        match self {
            ReadQuery::RecordQuery(x) => &x.name,
            ReadQuery::ManyRecordsQuery(x) => &x.name,
            ReadQuery::RelatedRecordsQuery(x) => &x.name,
            ReadQuery::AggregateRecordsQuery(x) => &x.name,
        }
    }
}

impl RecordFinderInjector for ReadQuery {
    fn inject_record_finder(&mut self, rf: RecordFinder) {
        match self {
            Self::RecordQuery(ref mut rq) => rq.record_finder = Some(rf),
            _ => unimplemented!(),
        }
    }
}

impl Display for ReadQuery {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Self::RecordQuery(q) => write!(
                f,
                "RecordQuery(name: '{}', finder: {:?})",
                q.name,
                q.record_finder.as_ref().map(|finder| format!(
                    "{}, {} = {:?}",
                    finder.field.model().name,
                    finder.field.name,
                    finder.value
                ))
            ),
            Self::ManyRecordsQuery(q) => write!(f, "ManyRecordsQuery(name: '{}', model: {})", q.name, q.model.name),
            Self::RelatedRecordsQuery(q) => write!(
                f,
                "RelatedRecordsQuery(name: '{}', parent model: {}, parent relation field: {})",
                q.name,
                q.parent_field.model().name,
                q.parent_field.name
            ),
            Self::AggregateRecordsQuery(q) => write!(f, "AggregateRecordsQuery: {}", q.name),
        }
    }
}

#[derive(Debug, Default, Clone)]
pub struct RecordQuery {
    pub name: String,
    pub alias: Option<String>,
    pub record_finder: Option<RecordFinder>,
    pub selected_fields: SelectedFields,
    pub nested: Vec<ReadQuery>,
    pub selection_order: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct ManyRecordsQuery {
    pub name: String,
    pub alias: Option<String>,
    pub model: ModelRef,
    pub args: QueryArguments,
    pub selected_fields: SelectedFields,
    pub nested: Vec<ReadQuery>,
    pub selection_order: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct RelatedRecordsQuery {
    pub name: String,
    pub alias: Option<String>,
    pub parent_field: RelationFieldRef,
    pub parent_ids: Option<Vec<GraphqlId>>,
    pub args: QueryArguments,
    pub selected_fields: SelectedFields,
    pub nested: Vec<ReadQuery>,
    pub selection_order: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct AggregateRecordsQuery {
    pub name: String,
    pub alias: Option<String>,
    pub model: ModelRef,
}
