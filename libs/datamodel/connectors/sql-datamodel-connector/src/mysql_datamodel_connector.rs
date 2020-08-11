use datamodel_connector::error::ConnectorError;
use datamodel_connector::{Connector, ConnectorCapability, NativeTypeConstructor, NativeTypeInstance};
use native_types::NativeType;

const INT_TYPE_NAME: &str = "Int";
const SMALL_INT_TYPE_NAME: &str = "SmallInt";
const TINY_INT_TYPE_NAME: &str = "TinyInt";
const MEDIUM_INT_TYPE_NAME: &str = "MediumInt";
const BIG_INT_TYPE_NAME: &str = "BigInt";
const DECIMAL: &str = "Decimal";


pub struct MySqlDatamodelConnector {
    capabilities: Vec<ConnectorCapability>,
    constructors: Vec<NativeTypeConstructor>,
}

impl MySqlDatamodelConnector {
    pub fn new() -> MySqlDatamodelConnector {
        let capabilities = vec![
            ConnectorCapability::RelationsOverNonUniqueCriteria,
            ConnectorCapability::Enums,
            ConnectorCapability::Json,
            ConnectorCapability::MultipleIndexesWithSameName,
            ConnectorCapability::AutoIncrementAllowedOnNonId,
        ];

        let constructors: Vec<NativeTypeConstructor> = vec![];

        MySqlDatamodelConnector {
            capabilities,
            constructors,
        }
    }
}

impl Connector for MySqlDatamodelConnector {
    fn capabilities(&self) -> &Vec<ConnectorCapability> {
        &self.capabilities
    }

    fn available_native_type_constructors(&self) -> &Vec<NativeTypeConstructor> {
        &self.constructors
    }

    fn parse_native_type(&self, _name: &str, _args: Vec<u32>) -> Result<NativeTypeInstance, ConnectorError> {
        None
    }

    fn introspect_native_type(&self, _native_type: Box<dyn NativeType>) -> Result<NativeTypeInstance, ConnectorError> {
        None
    }
}
