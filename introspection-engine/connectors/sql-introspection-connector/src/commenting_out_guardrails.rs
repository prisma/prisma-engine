use datamodel::{Datamodel, FieldArity, FieldType, RelationInfo};
use serde::Serialize;
use serde_json::Value;

#[derive(Serialize, Debug)]
struct Warning {
    code: i8,
    message: String,
    affected: Value,
}

#[derive(Serialize, Debug)]
struct Model {
    model: String,
}

#[derive(Serialize, Debug)]
struct ModelAndField {
    model: String,
    field: String,
}

#[derive(Serialize, Debug)]
struct ModelAndFieldType {
    model: String,
    field: String,
    tpe: String,
}

pub fn commenting_out_guardrails(datamodel: &mut Datamodel) -> Value {
    let mut models_without_identifiers = vec![];
    let mut fields_with_empty_names = vec![];
    let mut unsupported_types = vec![];

    let mut commented_model_names = vec![];

    // find models with 1to1 relations
    let mut models_with_one_to_one_relation = vec![];
    for model in &datamodel.models {
        if model.fields.iter().any(|f| match (&f.arity, &f.field_type) {
            (FieldArity::List, _) => false,
            (
                _,
                FieldType::Relation(RelationInfo {
                    to,
                    to_fields: _,
                    name: relation_name,
                    ..
                }),
            ) => {
                let other_model = datamodel.find_model(to).unwrap();
                let other_field = other_model
                    .fields
                    .iter()
                    .find(|f| match &f.field_type {
                        FieldType::Relation(RelationInfo {
                            to: other_to,
                            to_fields: _,
                            name: other_relation_name,
                            ..
                        }) if other_to == &model.name && relation_name == other_relation_name => true,
                        _ => false,
                    })
                    .unwrap();

                match other_field.arity {
                    FieldArity::Optional | FieldArity::Required => true,
                    FieldArity::List => false,
                }
            }
            (_, _) => false,
        }) {
            models_with_one_to_one_relation.push(model.name.clone())
        }
    }

    // models without uniques / ids
    for model in &mut datamodel.models {
        if model.id_fields.is_empty()
            && !model.fields.iter().any(|f| f.is_id || f.is_unique)
            && !model.indices.iter().any(|i| i.is_unique())
            && !models_with_one_to_one_relation.contains(&model.name)
        {
            commented_model_names.push(model.name.clone());
            model.is_commented_out = true;
            model.documentation = Some(
                "The underlying table does not contain a unique identifier and can therefore currently not be handled."
                    .to_string(),
            );
            models_without_identifiers.push(Model {
                model: model.name.clone(),
            })
        }
    }

    // remove their backrelations
    for name in &commented_model_names {
        for model in &mut datamodel.models {
            model.fields.retain(|f| !f.points_to_model(name));
        }
    }

    // fields with an empty name
    for model in &mut datamodel.models {
        for field in &mut model.fields {
            if field.name == "".to_string() {
                field.documentation = Some(
                    "This field was commented out because of an invalid name. Please provide a valid one that matches [a-zA-Z][a-zA-Z0-9_]*"
                        .to_string(),
                );
                field.name = field.database_names.first().unwrap().to_string();
                field.is_commented_out = true;

                fields_with_empty_names.push(ModelAndField {
                    model: model.name.clone(),
                    field: field.name.clone(),
                })
            }
        }
    }

    // fields with unsupported as datatype
    for model in &datamodel.models {
        for field in &model.fields {
            if let FieldType::Unsupported(tpe) = &field.field_type {
                unsupported_types.push(ModelAndFieldType {
                    model: model.name.clone(),
                    field: field.name.clone(),
                    tpe: tpe.clone(),
                })
            }
        }
    }

    let mut warnings = vec![];

    if !models_without_identifiers.is_empty() {
        warnings.push(Warning {
            code: 1,
            message: "These models do not have a unique identifier or id and are therefore commented out.".into(),
            affected: serde_json::to_value(&models_without_identifiers).unwrap(),
        })
    }

    if !fields_with_empty_names.is_empty() {
        warnings.push(Warning {
            code: 2,
            message: "The names of these fields are empty because we unsuccessfully tried to remap the column names."
                .into(),
            affected: serde_json::to_value(&fields_with_empty_names).unwrap(),
        })
    }

    if !unsupported_types.is_empty() {
        warnings.push(Warning {
            code: 3,
            message: "These fields were commented out because we currently do not support their types.".into(),
            affected: serde_json::to_value(&unsupported_types).unwrap(),
        })
    }

    serde_json::to_value(warnings).unwrap()
}
