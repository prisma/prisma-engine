mod combined_connector;

pub mod connector_error;
pub mod helper;

pub use combined_connector::CombinedConnector;

use crate::connector_error::{ConnectorError, ConnectorErrorFactory, ErrorKind};
use dml::{
    field::Field, model::Model, native_type_constructor::NativeTypeConstructor,
    native_type_instance::NativeTypeInstance, scalars::ScalarType,
};

pub trait Connector: Send + Sync {
    fn name(&self) -> String;

    fn capabilities(&self) -> &Vec<ConnectorCapability>;

    fn has_capability(&self, capability: ConnectorCapability) -> bool {
        self.capabilities().contains(&capability)
    }

    fn validate_field(&self, field: &Field) -> Result<(), ConnectorError>;

    fn validate_model(&self, model: &Model) -> Result<(), ConnectorError>;

    /// Returns all available native type constructors available through this connector.
    /// Powers the auto completion of the vs code plugin.
    fn available_native_type_constructors(&self) -> &[NativeTypeConstructor];

    /// On each connector, each built-in Prisma scalar type (`Boolean`,
    /// `String`, `Float`, etc.) has a corresponding native type.
    fn default_native_type_for_scalar_type(
        &self,
        scalar_type: &ScalarType,
        temporary_native_types_on: bool,
    ) -> serde_json::Value;

    /// Same mapping as `default_native_type_for_scalar_type()`, but in the opposite direction.
    fn native_type_is_default_for_scalar_type(&self, native_type: serde_json::Value, scalar_type: &ScalarType) -> bool;

    fn find_native_type_constructor(&self, name: &str) -> Option<&NativeTypeConstructor> {
        self.available_native_type_constructors()
            .iter()
            .find(|constructor| constructor.name.as_str() == name)
    }

    /// This function is used during Schema parsing to calculate the concrete native type.
    /// This powers the use of native types for QE + ME.
    fn parse_native_type(&self, name: &str, args: Vec<String>) -> Result<NativeTypeInstance, ConnectorError>;

    /// This function is used in ME for error messages
    fn render_native_type(&self, native_type: serde_json::Value) -> String {
        let instance = self.introspect_native_type(native_type).unwrap();
        instance.render()
    }

    /// This function is used during introspection to turn an introspected native type into an instance that can be put into the Prisma schema.
    /// powers IE
    fn introspect_native_type(&self, native_type: serde_json::Value) -> Result<NativeTypeInstance, ConnectorError>;

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

    fn supports_non_id_auto_increment(&self) -> bool {
        self.has_capability(ConnectorCapability::AutoIncrementAllowedOnNonId)
    }

    fn supports_multiple_auto_increment(&self) -> bool {
        self.has_capability(ConnectorCapability::AutoIncrementMultipleAllowed)
    }

    fn supports_non_indexed_auto_increment(&self) -> bool {
        self.has_capability(ConnectorCapability::AutoIncrementNonIndexedAllowed)
    }

    fn allows_relation_fields_in_arbitrary_order(&self) -> bool {
        self.has_capability(ConnectorCapability::RelationFieldsInArbitraryOrder)
    }

    fn native_instance_error(&self, instance: NativeTypeInstance) -> ConnectorErrorFactory {
        ConnectorErrorFactory {
            connector: self.name(),
            native_type: instance.render(),
        }
    }

    fn native_str_error(&self, native_str: &str) -> ConnectorErrorFactory {
        ConnectorErrorFactory {
            connector: self.name(),
            native_type: native_str.to_string(),
        }
    }

    fn native_types_not_supported(&self) -> Result<NativeTypeInstance, ConnectorError> {
        Err(ConnectorError::from_kind(
            ErrorKind::ConnectorNotSupportedForNativeTypes {
                connector_name: self.name(),
            },
        ))
    }
}

/// Not all Databases are created equal. Hence connectors for our datasources support different capabilities.
/// These are used during schema validation. E.g. if a connector does not support enums an error will be raised.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ConnectorCapability {
    // start of General Schema Capabilities
    ScalarLists,
    RelationsOverNonUniqueCriteria,
    MultipleIndexesWithSameName,
    Enums,
    Json,
    AutoIncrementAllowedOnNonId,
    AutoIncrementMultipleAllowed,
    AutoIncrementNonIndexedAllowed,
    RelationFieldsInArbitraryOrder,
    // start of Query Engine Capabilities
    InsensitiveFilters,
    CreateMany,
    WritableAutoincField,
    CreateSkipDuplicates,
}

/// Contains all capabilities that the connector is able to serve.
#[derive(Debug)]
pub struct ConnectorCapabilities {
    capabilities: Vec<ConnectorCapability>,
}

impl ConnectorCapabilities {
    pub fn empty() -> Self {
        Self { capabilities: vec![] }
    }

    pub fn new(capabilities: Vec<ConnectorCapability>) -> Self {
        Self { capabilities }
    }

    pub fn contains(&self, capability: ConnectorCapability) -> bool {
        self.capabilities.contains(&capability)
    }
}
