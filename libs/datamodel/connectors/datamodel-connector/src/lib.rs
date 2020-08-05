use crate::scalars::ScalarType;

pub mod error;
pub mod scalars;

mod builtin_connectors;
mod declarative_connector;

pub use builtin_connectors::BuiltinConnectors;
pub use declarative_connector::DeclarativeConnector;

pub trait Connector: Send + Sync {
    fn capabilities(&self) -> &Vec<ConnectorCapability>;

    fn has_capability(&self, capability: ConnectorCapability) -> bool {
        self.capabilities().contains(&capability)
    }

    fn calculate_type(&self, name: &str, args: Vec<i32>) -> Option<ScalarFieldType>;

    fn supports_scalar_lists(&self) -> bool {
        self.has_capability(ConnectorCapability::ScalarLists)
    }

    fn supports_multiple_indexes_with_same_name(&self) -> bool {
        self.has_capability(ConnectorCapability::MultipleIndexesWithSameName)
    }

    fn supports_relations_over_non_unique_criteria(&self) -> bool {
        self.has_capability(ConnectorCapability::RelationsOverNonUniqueCriteria)
    }

    fn supports_enums(&self) -> bool {
        self.has_capability(ConnectorCapability::Enums)
    }

    fn supports_json(&self) -> bool {
        self.has_capability(ConnectorCapability::Json)
    }

    fn supports_auto_increment(&self) -> bool {
        self.capabilities().into_iter().any(|c| match c {
            ConnectorCapability::AutoIncrement { .. } => true,
            _ => false,
        })
    }

    fn supports_non_id_auto_increment(&self) -> bool {
        self.capabilities().into_iter().any(|c| match c {
            ConnectorCapability::AutoIncrement {
                non_id_allowed: true, ..
            } => true,
            _ => false,
        })
    }

    fn supports_multiple_auto_increment(&self) -> bool {
        self.capabilities().into_iter().any(|c| match c {
            ConnectorCapability::AutoIncrement {
                multiple_allowed: true, ..
            } => true,
            _ => false,
        })
    }

    fn supports_non_indexed_auto_increment(&self) -> bool {
        self.capabilities().into_iter().any(|c| match c {
            ConnectorCapability::AutoIncrement {
                non_indexed_allowed: true,
                ..
            } => true,
            _ => false,
        })
    }
}

/// Not all Databases are created equal. Hence connectors for our datasources support different capabilities.
/// These are used during schema validation. E.g. if a connector does not support enums an error will be raised.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ConnectorCapability {
    ScalarLists,
    RelationsOverNonUniqueCriteria,
    MultipleIndexesWithSameName,
    Enums,
    Json,
    AutoIncrement {
        non_id_allowed: bool,
        multiple_allowed: bool,
        non_indexed_allowed: bool,
    },
}

#[derive(Debug, Clone, PartialEq)]
pub struct ScalarFieldType {
    name: String,
    prisma_type: scalars::ScalarType,
    datasource_type: String,
}

impl ScalarFieldType {
    pub fn new(name: &str, prisma_type: scalars::ScalarType, datasource_type: &str) -> Self {
        ScalarFieldType {
            name: name.to_string(),
            prisma_type,
            datasource_type: datasource_type.to_string(),
        }
    }

    pub fn prisma_type(&self) -> scalars::ScalarType {
        self.prisma_type
    }

    pub fn datasource_type(&self) -> &str {
        &self.datasource_type
    }
}
