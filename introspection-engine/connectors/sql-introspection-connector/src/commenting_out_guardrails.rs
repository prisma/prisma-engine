use crate::warnings::{
    warning_enum_values_with_empty_names, warning_fields_with_empty_names, warning_models_without_identifier,
    warning_unsupported_types, EnumAndValue, Model, ModelAndField, ModelAndFieldAndType,
};
use datamodel::{Datamodel, FieldArity, FieldType};
use introspection_connector::Warning;

pub fn commenting_out_guardrails(datamodel: &mut Datamodel) -> Vec<Warning> {
    let mut models_without_identifiers = vec![];
    let mut fields_with_empty_names = vec![];
    let mut enum_values_with_empty_names = vec![];
    let mut unsupported_types = vec![];

    // find models with 1to1 relations
    let mut models_with_one_to_one_relation = vec![];
    for model in &datamodel.models {
        if model.relation_fields().any(|f| match f.arity {
            FieldArity::List => false,
            _ => {
                let other_field = datamodel.find_related_field_for_info(&f.relation_info);

                match other_field.arity {
                    FieldArity::Optional | FieldArity::Required => true,
                    FieldArity::List => false,
                }
            }
        }) {
            models_with_one_to_one_relation.push(model.name.clone())
        }
    }

    //todo more stuff to handle when commenting out. (Maybe it is easier to just work on supporting it.)
    // models with empty names?
    // also needs to follow the field references (relations, indexes, ids...)
    // also needs to drop usages of removed enum values

    // fields with an empty name
    for model in &mut datamodel.models {
        let model_name = model.name.clone();

        for field in model.scalar_fields_mut() {
            if field.name == "".to_string() {
                field.documentation = Some(
                    "This field was commented out because of an invalid name. Please provide a valid one that matches [a-zA-Z][a-zA-Z0-9_]*"
                        .to_string(),
                );
                field.name = field.database_name.as_ref().unwrap().to_string();
                field.is_commented_out = true;

                fields_with_empty_names.push(ModelAndField {
                    model: model_name.clone(),
                    field: field.name.clone(),
                })
            }
        }
    }

    //empty enum values
    for enm in &mut datamodel.enums {
        for enum_value in &mut enm.values {
            if let Some(name) = &enum_value.database_name {
                if enum_value.name == "".to_string() {
                    enum_value.name = name.clone();
                    enum_value.commented_out = true;
                    enum_values_with_empty_names.push(EnumAndValue {
                        enm: enm.name.clone(),
                        value: enum_value.name.clone(),
                    })
                }
            }
        }
    }

    // fields with unsupported as datatype
    for model in &mut datamodel.models {
        let model_name = model.name.clone();

        for field in model.scalar_fields_mut() {
            if let FieldType::Unsupported(tpe) = &field.field_type {
                field.is_commented_out = true;
                unsupported_types.push(ModelAndFieldAndType {
                    model: model_name.clone(),
                    field: field.name.clone(),
                    tpe: tpe.clone(),
                })
            }
        }
    }

    // use unsupported types to drop @@id / @@unique /@@index
    for mf in &unsupported_types {
        let model = datamodel.find_model_mut(&mf.model).unwrap();
        model.indices.retain(|i| !i.fields.contains(&mf.field));
        if model.id_fields.contains(&mf.field) {
            model.id_fields = vec![]
        };
    }

    // models without uniques / ids
    for model in &mut datamodel.models {
        if model.id_fields.is_empty()
            && !model
                .fields
                .iter()
                .any(|f| (f.is_id() || f.is_unique()) && !f.is_commented_out())
            && !model.indices.iter().any(|i| i.is_unique())
            && !models_with_one_to_one_relation.contains(&model.name)
        {
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
    for model_without_identifier in &models_without_identifiers {
        for model in &mut datamodel.models {
            model
                .fields
                .retain(|f| !f.points_to_model(model_without_identifier.model.as_ref()));
        }
    }

    let mut warnings = vec![];

    if !models_without_identifiers.is_empty() {
        warnings.push(warning_models_without_identifier(&models_without_identifiers))
    }

    if !fields_with_empty_names.is_empty() {
        warnings.push(warning_fields_with_empty_names(&fields_with_empty_names))
    }

    if !unsupported_types.is_empty() {
        warnings.push(warning_unsupported_types(&unsupported_types))
    }

    if !enum_values_with_empty_names.is_empty() {
        warnings.push(warning_enum_values_with_empty_names(&enum_values_with_empty_names))
    }

    warnings
}
