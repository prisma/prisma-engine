use crate::*;
use datamodel::{dml, DefaultValue, WithDatabaseName};
use itertools::Itertools;

pub struct DatamodelConverter<'a> {
    datamodel: &'a dml::Datamodel,
    relations: Vec<TempRelationHolder>,
}

impl<'a> DatamodelConverter<'a> {
    pub fn convert_string(datamodel: String) -> InternalDataModelTemplate {
        let datamodel = datamodel::parse_datamodel(&datamodel).unwrap();
        Self::convert(&datamodel)
    }

    pub fn convert(datamodel: &dml::Datamodel) -> InternalDataModelTemplate {
        DatamodelConverter::new(datamodel).convert_internal()
    }

    fn new(datamodel: &dml::Datamodel) -> DatamodelConverter {
        DatamodelConverter {
            datamodel,
            relations: Self::calculate_relations(datamodel),
        }
    }

    fn convert_internal(&self) -> InternalDataModelTemplate {
        InternalDataModelTemplate {
            models: self.convert_models(),
            relations: self.convert_relations(),
            enums: self.convert_enums(),
            version: Some("v2".to_string()),
        }
    }

    fn convert_enums(&self) -> Vec<InternalEnum> {
        self.datamodel
            .enums()
            .map(|e| InternalEnum {
                name: e.name.clone(),
                values: self.convert_enum_values(e),
            })
            .collect()
    }

    fn convert_enum_values(&self, enm: &dml::Enum) -> Vec<InternalEnumValue> {
        enm.values()
            .map(|enum_value| InternalEnumValue {
                name: enum_value.name.clone(),
                database_name: enum_value.database_name.clone(),
            })
            .collect()
    }

    fn convert_models(&self) -> Vec<ModelTemplate> {
        self.datamodel
            .models()
            .map(|model| {
                let model = Self::sanitize_model(model.clone());

                ModelTemplate {
                    name: model.name.clone(),
                    is_embedded: model.is_embedded,
                    fields: self.convert_fields(&model),
                    manifestation: model.single_database_name().map(|s| s.to_owned()),
                    id_field_names: model.id_fields.clone(),
                    indexes: self.convert_indexes(&model),
                }
            })
            .collect()
    }

    fn convert_fields(&self, model: &dml::Model) -> Vec<FieldTemplate> {
        model
            .fields()
            .map(|field| match field.field_type {
                dml::FieldType::Relation(ref ri) => {
                    let relation = self
                        .relations
                        .iter()
                        .find(|r| r.is_for_model_and_field(model, field))
                        .unwrap_or_else(|| {
                            panic!(
                                "Did not find a relation for model {} and field {}",
                                model.name, field.name
                            )
                        });

                    FieldTemplate::Relation(RelationFieldTemplate {
                        name: field.name.clone(),
                        is_id: field.is_id,
                        is_required: field.is_required(),
                        is_list: field.is_list(),
                        is_unique: field.is_unique(),
                        is_auto_generated_int_id: field.is_auto_generated_int_id(),
                        data_source_fields: field.data_source_fields.clone(),
                        relation_name: relation.name(),
                        relation_side: relation.relation_side(field),
                        relation_info: ri.clone(),
                    })
                }
                _ => FieldTemplate::Scalar(ScalarFieldTemplate {
                    name: field.name.clone(),
                    type_identifier: field.type_identifier(),
                    is_required: field.is_required(),
                    is_list: field.is_list(),
                    is_unique: field.is_unique(),
                    is_id: field.is_id,
                    is_auto_generated_int_id: field.is_auto_generated_int_id(),
                    data_source_field: field
                        .data_source_fields
                        .clone()
                        .pop()
                        .expect("Expected exactly one data source field for ScalarFieldTemplate."),
                    behaviour: field.behaviour(),
                    internal_enum: field.internal_enum(self.datamodel),
                }),
            })
            .collect()
    }

    fn convert_relations(&self) -> Vec<RelationTemplate> {
        self.relations
            .iter()
            .map(|r| RelationTemplate {
                name: r.name(),
                model_a_on_delete: OnDelete::SetNull,
                model_b_on_delete: OnDelete::SetNull,
                manifestation: r.manifestation(),
                model_a_name: r.model_a.name.clone(),
                model_b_name: r.model_b.name.clone(),
            })
            .collect()
    }

    fn convert_indexes(&self, model: &dml::Model) -> Vec<IndexTemplate> {
        model
            .indices
            .iter()
            .map(|i| IndexTemplate {
                name: i.name.clone(),
                fields: i.fields.clone(),
                typ: match i.tpe {
                    dml::IndexType::Unique => IndexType::Unique,
                    dml::IndexType::Normal => IndexType::Normal,
                },
            })
            .collect()
    }

    pub fn calculate_relations(datamodel: &dml::Datamodel) -> Vec<TempRelationHolder> {
        let mut result = Vec::new();
        for model in datamodel.models() {
            for field in model.fields() {
                if let dml::FieldType::Relation(relation_info) = &field.field_type {
                    let dml::RelationInfo {
                        to, to_fields, name, ..
                    } = relation_info;

                    let related_model = datamodel
                        .find_model(&to)
                        .unwrap_or_else(|| panic!("Related model {} not found", to));

                    let related_field = related_model
                        .fields()
                        .find(|f| match f.field_type {
                            dml::FieldType::Relation(ref rel_info) => {
                                // TODO: i probably don't need to check the the `to`. The name of the relation should be enough. The parser must guarantee that the relation info is set right.
                                if model.name == related_model.name {
                                    // SELF RELATIONS
                                    rel_info.to == model.name && &rel_info.name == name && f.name != field.name
                                } else {
                                    // In a normal relation the related field could be named the same hence we omit the last condition from above.
                                    rel_info.to == model.name && &rel_info.name == name
                                }
                            }
                            _ => false,
                        })
                        .unwrap_or_else(|| {
                            panic!(
                                "Related model for model {} and field {} not found",
                                model.name, field.name
                            )
                        });

                    let related_field_info: &dml::RelationInfo = match &related_field.field_type {
                        dml::FieldType::Relation(info) => info,
                        _ => panic!("this was not a relation field"),
                    };

                    let (model_a, model_b, field_a, field_b, referenced_fields_a, referenced_fields_b) = match () {
                        _ if model.name < related_model.name => (
                            model.clone(),
                            related_model.clone(),
                            field.clone(),
                            related_field.clone(),
                            to_fields,
                            &related_field_info.to_fields,
                        ),
                        _ if related_model.name < model.name => (
                            related_model.clone(),
                            model.clone(),
                            related_field.clone(),
                            field.clone(),
                            &related_field_info.to_fields,
                            to_fields,
                        ),
                        // SELF RELATION CASE
                        _ => {
                            let (field_a, field_b) = if field.name < related_field.name {
                                (field.clone(), related_field.clone())
                            } else {
                                (related_field.clone(), field.clone())
                            };
                            (
                                model.clone(),
                                related_model.clone(),
                                field_a,
                                field_b,
                                to_fields,
                                &related_field_info.to_fields,
                            )
                        }
                    };
                    let inline_on_model_a = TempManifestationHolder::Inline {
                        in_table_of_model: model_a.name.clone(),
                        field: field_a.clone(),
                        referenced_fields: referenced_fields_a.clone(),
                    };
                    let inline_on_model_b = TempManifestationHolder::Inline {
                        in_table_of_model: model_b.name.clone(),
                        field: field_b.clone(),
                        referenced_fields: referenced_fields_b.clone(),
                    };
                    let inline_on_this_model = TempManifestationHolder::Inline {
                        in_table_of_model: model.name.clone(),
                        field: field.clone(),
                        referenced_fields: to_fields.clone(),
                    };
                    let inline_on_related_model = TempManifestationHolder::Inline {
                        in_table_of_model: related_model.name.clone(),
                        field: related_field.clone(),
                        referenced_fields: related_field_info.to_fields.clone(),
                    };

                    let manifestation = match (field_a.is_list(), field_b.is_list()) {
                        (true, true) => TempManifestationHolder::Table,
                        (false, true) => inline_on_model_a,
                        (true, false) => inline_on_model_b,
                        // TODO: to_fields is now a list, please fix this line.
                        (false, false) => match (to_fields.first(), &related_field_info.to_fields.first()) {
                            (Some(_), None) => inline_on_this_model,
                            (None, Some(_)) => inline_on_related_model,
                            (None, None) => {
                                if model_a.name < model_b.name {
                                    inline_on_model_a
                                } else {
                                    inline_on_model_b
                                }
                            }
                            (Some(_), Some(_)) => {
                                panic!("It's not allowed that both sides of a relation specify the inline policy. The field was {} on model {}. The related field was {} on model {}.", field.name, model.name, related_field.name, related_model.name)
                            }
                        },
                    };

                    result.push(TempRelationHolder {
                        name: name.clone(),
                        model_a,
                        model_b,
                        field_a,
                        field_b,
                        manifestation,
                    })
                }
            }
        }

        result.into_iter().unique_by(|rel| rel.name()).collect()
    }

    /// Normalizes the model for usage in the query core.
    fn sanitize_model(mut model: dml::Model) -> dml::Model {
        // Fold single-field unique indices into the fields (makes a single field unique).
        let (keep, transform): (Vec<_>, Vec<_>) = model.indices.into_iter().partition(|i| match i.tpe {
            dml::IndexType::Unique if i.fields.len() == 1 => false,
            _ => true,
        });

        model.indices = keep;

        for index in transform {
            if index.tpe == dml::IndexType::Unique {
                let field_name = index.fields.first().unwrap();

                model
                    .fields
                    .iter_mut()
                    .find(|f| &f.name == field_name)
                    .map(|f| f.is_unique = true);
            }
        }

        // Fold single-field @@id into the fields (makes a single field @id).
        if model.id_fields.len() == 1 {
            let field_name = model.id_fields.pop().unwrap();

            model
                .fields
                .iter_mut()
                .find(|f| f.name == field_name)
                .map(|f| f.is_id = true);
        }

        model
    }
}

#[derive(Debug, Clone)]
pub struct TempRelationHolder {
    pub name: String,
    pub model_a: dml::Model,
    pub model_b: dml::Model,
    pub field_a: dml::Field,
    pub field_b: dml::Field,
    pub manifestation: TempManifestationHolder,
}

#[derive(PartialEq, Debug, Clone)]
pub enum TempManifestationHolder {
    Inline {
        in_table_of_model: String,
        /// The relation field.
        field: dml::Field,
        /// The name of the (dml) fields referenced by the relation.
        referenced_fields: Vec<String>,
    },
    Table,
}

#[allow(unused)]
impl TempRelationHolder {
    fn name(&self) -> String {
        // TODO: must replicate behaviour of `generateRelationName` from `SchemaInferrer`
        match &self.name as &str {
            "" => format!("{}To{}", &self.model_a.name, &self.model_b.name),
            _ => self.name.clone(),
        }
    }

    pub fn table_name(&self) -> String {
        format!("_{}", self.name())
    }

    pub fn model_a_column(&self) -> String {
        "A".to_string()
    }

    pub fn model_b_column(&self) -> String {
        "B".to_string()
    }

    pub fn is_one_to_one(&self) -> bool {
        !self.field_a.is_list() && !self.field_b.is_list()
    }

    fn is_many_to_many(&self) -> bool {
        self.field_a.is_list() && self.field_b.is_list()
    }

    fn is_for_model_and_field(&self, model: &dml::Model, field: &dml::Field) -> bool {
        (&self.model_a == model && &self.field_a == field) || (&self.model_b == model && &self.field_b == field)
    }

    fn relation_side(&self, field: &dml::Field) -> RelationSide {
        if field == &self.field_a {
            RelationSide::A
        } else if field == &self.field_b {
            RelationSide::B
        } else {
            panic!("this field is not part of hte relations")
        }
    }

    fn manifestation(&self) -> RelationLinkManifestation {
        match &self.manifestation {
            // TODO: relation table columns must get renamed: lowercased type names instead of A and B
            TempManifestationHolder::Table => RelationLinkManifestation::RelationTable(RelationTable {
                table: self.table_name(),
                model_a_column: self.model_a_column(),
                model_b_column: self.model_b_column(),
            }),
            TempManifestationHolder::Inline { in_table_of_model, .. } => {
                RelationLinkManifestation::Inline(InlineRelation {
                    in_table_of_model_name: in_table_of_model.to_string(),
                })
            }
        }
    }
}

trait DatamodelFieldExtensions {
    fn type_identifier(&self) -> TypeIdentifier;
    fn is_required(&self) -> bool;
    fn is_list(&self) -> bool;
    fn is_unique(&self) -> bool;
    fn is_auto_generated_int_id(&self) -> bool;
    fn behaviour(&self) -> Option<FieldBehaviour>;
    fn final_db_name(&self) -> String;
    fn internal_enum(&self, datamodel: &dml::Datamodel) -> Option<InternalEnum>;
    fn internal_enum_value(&self, enum_value: &dml::EnumValue) -> InternalEnumValue;
    // fn default_value(&self) -> Option<dml::DefaultValue>; todo this is not applicable anymore
}

impl DatamodelFieldExtensions for dml::Field {
    fn type_identifier(&self) -> TypeIdentifier {
        match &self.field_type {
            dml::FieldType::Enum(x) => TypeIdentifier::Enum(x.clone()),
            dml::FieldType::Relation(_) => TypeIdentifier::String, // Todo: Unused
            dml::FieldType::Base(scalar) => match scalar {
                dml::ScalarType::Boolean => TypeIdentifier::Boolean,
                dml::ScalarType::DateTime => TypeIdentifier::DateTime,
                dml::ScalarType::Decimal => TypeIdentifier::Float,
                dml::ScalarType::Float => TypeIdentifier::Float,
                dml::ScalarType::Int => TypeIdentifier::Int,
                dml::ScalarType::String => TypeIdentifier::String,
            },
            dml::FieldType::ConnectorSpecific { .. } => {
                unimplemented!("Connector Specific types are not supported here yet")
            }
        }
    }

    fn is_required(&self) -> bool {
        self.arity == dml::FieldArity::Required
    }

    fn is_list(&self) -> bool {
        self.arity == dml::FieldArity::List
    }

    fn is_unique(&self) -> bool {
        self.is_unique
    }

    fn is_auto_generated_int_id(&self) -> bool {
        let is_autogenerated_id = match self.default_value {
            Some(DefaultValue::Expression(_)) if self.is_id => true,
            _ => false,
        };

        let is_an_int = self.type_identifier() == TypeIdentifier::Int;

        is_autogenerated_id && is_an_int
    }

    fn behaviour(&self) -> Option<FieldBehaviour> {
        if self.is_updated_at {
            Some(FieldBehaviour::UpdatedAt)
        } else {
            None
        }
    }

    fn final_db_name(&self) -> String {
        match self.database_names.first() {
            None => self.name.clone(),
            Some(x) => x.clone(),
        }
    }

    fn internal_enum(&self, datamodel: &dml::Datamodel) -> Option<InternalEnum> {
        match self.field_type {
            dml::FieldType::Enum(ref name) => {
                datamodel
                    .enums()
                    .find(|e| e.name == name.clone())
                    .map(|e| InternalEnum {
                        name: e.name.clone(),
                        values: e.values().map(|v| self.internal_enum_value(v)).collect(),
                    })
            }
            _ => None,
        }
    }

    fn internal_enum_value(&self, enum_value: &dml::EnumValue) -> InternalEnumValue {
        InternalEnumValue {
            name: enum_value.name.clone(),
            database_name: enum_value.database_name.clone(),
        }
    }

    // fn default_value(&self) -> Option<dml::DefaultValue> {
    //     self.default_value.clone()
    // }
}
