use super::SqlSchemaCalculatorFlavour;
use crate::flavour::MssqlFlavour;
use datamodel::{
    walkers::{ModelWalker, ScalarFieldWalker},
    FieldArity, NativeTypeInstance, ScalarType,
};
use native_types::{MsSqlType, NativeType};
use sql_schema_describer::{ColumnArity, ColumnType, ColumnTypeFamily, ForeignKeyAction};

impl SqlSchemaCalculatorFlavour for MssqlFlavour {
    fn column_type_for_native_type(
        &self,
        field: &ScalarFieldWalker<'_>,
        _scalar_type: ScalarType,
        native_type_instance: &NativeTypeInstance,
    ) -> ColumnType {
        let mssql_type: MsSqlType = native_type_instance.deserialize_native_type();
        let data_type = "".to_string();
        let full_data_type = "".to_string();

        ColumnType {
            data_type,
            full_data_type,
            character_maximum_length: None,
            family: ColumnTypeFamily::String,
            arity: match field.arity() {
                FieldArity::Required => ColumnArity::Required,
                FieldArity::Optional => ColumnArity::Nullable,
                FieldArity::List => ColumnArity::List,
            },
            native_type: Some(mssql_type.to_json()),
        }
    }

    fn m2m_foreign_key_action(&self, model_a: &ModelWalker<'_>, model_b: &ModelWalker<'_>) -> ForeignKeyAction {
        // MSSQL will crash when creating a cyclic cascade
        if model_a.name() == model_b.name() {
            ForeignKeyAction::NoAction
        } else {
            ForeignKeyAction::Cascade
        }
    }
}
