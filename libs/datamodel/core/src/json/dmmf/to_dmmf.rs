use super::*;
use crate::common::ScalarType;
use crate::{dml, IndexType};
use serde_json;

pub fn render_to_dmmf(schema: &dml::Datamodel) -> String {
    let dmmf = schema_to_dmmf(schema);
    serde_json::to_string_pretty(&dmmf).expect("Failed to render JSON")
}

pub fn render_to_dmmf_value(schema: &dml::Datamodel) -> serde_json::Value {
    let dmmf = schema_to_dmmf(schema);
    serde_json::to_value(&dmmf).expect("Failed to render JSON")
}

fn schema_to_dmmf(schema: &dml::Datamodel) -> Datamodel {
    let mut datamodel = Datamodel {
        models: vec![],
        enums: vec![],
    };

    for enum_model in schema.enums() {
        datamodel.enums.push(enum_to_dmmf(&enum_model));
    }

    for model in schema.models() {
        datamodel.models.push(model_to_dmmf(&model));
    }

    datamodel
}

fn enum_to_dmmf(en: &dml::Enum) -> Enum {
    let mut enm = Enum {
        name: en.name.clone(),
        values: vec![],
        db_name: en.database_name.clone(),
        documentation: en.documentation.clone(),
    };

    for enum_value in &en.values {
        enm.values.push(enum_value_to_dmmf(enum_value));
    }

    enm
}

fn enum_value_to_dmmf(en: &dml::EnumValue) -> EnumValue {
    EnumValue {
        name: en.name.clone(),
        db_name: en.database_name.clone(),
    }
}

fn model_to_dmmf(model: &dml::Model) -> Model {
    Model {
        name: model.name.clone(),
        db_name: model.database_name.clone(),
        is_embedded: model.is_embedded,
        fields: model.fields().map(&field_to_dmmf).collect(),
        is_generated: Some(model.is_generated),
        documentation: model.documentation.clone(),
        id_fields: model.id_fields.clone(),
        unique_fields: model
            .indices
            .iter()
            .filter_map(|i| {
                if i.tpe == IndexType::Unique {
                    Some(i.fields.clone())
                } else {
                    None
                }
            })
            .collect(),
    }
}

fn field_to_dmmf(field: &dml::Field) -> Field {
    Field {
        name: field.name.clone(),
        kind: get_field_kind(field),
        db_names: field.database_names.clone(),
        is_required: field.arity == dml::FieldArity::Required,
        is_list: field.arity == dml::FieldArity::List,
        is_id: field.is_id,
        default: default_value_to_serde(&field.default_value),
        is_unique: field.is_unique,
        relation_name: get_relation_name(field),
        relation_to_fields: get_relation_to_fields(field),
        relation_on_delete: get_relation_delete_strategy(field),
        field_type: get_field_type(field),
        is_generated: Some(field.is_generated),
        is_updated_at: Some(field.is_updated_at),
        documentation: field.documentation.clone(),
    }
}

fn get_field_kind(field: &dml::Field) -> String {
    match field.field_type {
        dml::FieldType::Relation(_) => String::from("object"),
        dml::FieldType::Enum(_) => String::from("enum"),
        dml::FieldType::Base(_) => String::from("scalar"),
        _ => unimplemented!("DMMF does not support field type {:?}", field.field_type),
    }
}

fn get_field_type(field: &dml::Field) -> String {
    match &field.field_type {
        dml::FieldType::Relation(relation_info) => relation_info.to.clone(),
        dml::FieldType::Enum(t) => t.clone(),
        dml::FieldType::Base(t) => type_to_string(t),
        dml::FieldType::ConnectorSpecific(sft) => type_to_string(&sft.prisma_type()),
    }
}

fn type_to_string(scalar: &ScalarType) -> String {
    scalar.to_string()
}

fn default_value_to_serde(dv_opt: &Option<dml::DefaultValue>) -> Option<serde_json::Value> {
    dv_opt.as_ref().map(|dv| match dv {
        dml::DefaultValue::Single(value) => value_to_serde(&value.clone()),
        dml::DefaultValue::Expression(vg) => function_to_serde(&vg.name, &vg.args),
    })
}

fn value_to_serde(value: &dml::ScalarValue) -> serde_json::Value {
    match value {
        dml::ScalarValue::Boolean(val) => serde_json::Value::Bool(*val),
        dml::ScalarValue::String(val) => serde_json::Value::String(val.clone()),
        dml::ScalarValue::ConstantLiteral(name) => serde_json::Value::String(name.clone()),
        dml::ScalarValue::Float(val) => serde_json::Value::Number(serde_json::Number::from_f64(*val as f64).unwrap()),
        dml::ScalarValue::Int(val) => serde_json::Value::Number(serde_json::Number::from_f64(*val as f64).unwrap()),
        dml::ScalarValue::Decimal(val) => serde_json::Value::Number(serde_json::Number::from_f64(*val as f64).unwrap()),
        dml::ScalarValue::DateTime(val) => serde_json::Value::String(val.to_rfc3339()),
    }
}

fn function_to_serde(name: &str, args: &Vec<dml::ScalarValue>) -> serde_json::Value {
    let func = Function {
        name: String::from(name),
        args: args.iter().map(|arg| value_to_serde(arg)).collect(),
    };

    serde_json::to_value(&func).expect("Failed to render function JSON")
}

fn get_relation_name(field: &dml::Field) -> Option<String> {
    match &field.field_type {
        dml::FieldType::Relation(relation_info) => Some(relation_info.name.clone()),
        _ => None,
    }
}

fn get_relation_to_fields(field: &dml::Field) -> Option<Vec<String>> {
    match &field.field_type {
        dml::FieldType::Relation(relation_info) => Some(relation_info.to_fields.clone()),
        _ => None,
    }
}

fn get_relation_delete_strategy(field: &dml::Field) -> Option<String> {
    match &field.field_type {
        dml::FieldType::Relation(relation_info) => Some(relation_info.on_delete.to_string()),
        _ => None,
    }
}
