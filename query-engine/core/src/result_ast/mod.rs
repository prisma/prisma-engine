use connector::QueryArguments;
use prisma_models::{ManyRecords, ModelProjection, RecordProjection};

#[derive(Debug, Clone)]
pub enum QueryResult {
    Id(Option<RecordProjection>),
    Count(usize),
    RecordSelection(RecordSelection),
    Unit,
    Json(serde_json::Value),
}

// Todo: In theory, much of this info can go into the serializer as soon as the read results are resolved in a flat tree.
#[derive(Debug, Default, Clone)]
pub struct RecordSelection {
    /// Name of the query.
    pub name: String,

    /// Holds an ordered list of selected field names for each contained record.
    pub fields: Vec<String>,

    /// Scalar field results
    pub scalars: ManyRecords,

    /// Nested query results
    // Todo this is only here because reads are still resolved in one go
    pub nested: Vec<QueryResult>,

    /// Required for result processing
    pub query_arguments: QueryArguments,

    /// Model projection that can be used to retrieve the IDs of the contained records.
    pub model_id: ModelProjection,
}
