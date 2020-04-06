use super::common::*;
use crate::{
    ast, common::names::*, dml, dml::WithDatabaseName, error::ErrorCollection, DataSourceField, FieldArity,
    OnDeleteStrategy,
};

/// Helper for standardsing a datamodel.
///
/// When standardsing, datamodel will be made consistent.
/// Implicit back relation fields, relation names and `to_fields` will be generated.
pub struct Standardiser {}

impl Standardiser {
    /// Creates a new instance, with all builtin directives registered.
    pub fn new() -> Self {
        Standardiser {}
    }

    pub fn standardise(&self, ast_schema: &ast::SchemaAst, schema: &mut dml::Datamodel) -> Result<(), ErrorCollection> {
        self.add_missing_back_relations(ast_schema, schema)?;

        // This is intentionally disabled for now, since the generated types would surface in the
        // client schema.
        // self.add_missing_relation_tables(ast_schema, schema)?;

        self.set_relation_to_field_to_id_if_missing(schema);

        self.name_unnamed_relations(schema);

        self.populate_datasource_fields(schema);

        Ok(())
    }

    /// For any relations which are missing to_fields, sets them to the @id fields
    /// of the foreign model.
    fn set_relation_to_field_to_id_if_missing(&self, schema: &mut dml::Datamodel) {
        let schema_copy = schema.clone();

        // Iterate and mutate models.
        for model_idx in 0..schema.models.len() {
            let model = &mut schema.models[model_idx];
            let model_name = &model.name;

            for field_index in 0..model.fields.len() {
                let field = &mut model.fields[field_index];

                if let dml::FieldType::Relation(rel) = &mut field.field_type {
                    let related_model = schema_copy.find_model(&rel.to).expect(STATE_ERROR);
                    let related_field = related_model.related_field(model_name, &rel.name, &field.name).unwrap();
                    let related_model_name = &related_model.name;

                    let related_field_rel = if let dml::FieldType::Relation(rel) = &related_field.field_type {
                        rel
                    } else {
                        panic!(STATE_ERROR)
                    };

                    // If one of the fields has to_fields explicitly set by the user, we continue.
                    if !rel.to_fields.is_empty() || !related_field_rel.to_fields.is_empty() {
                        continue;
                    }

                    let embed_here = match (field.arity, related_field.arity) {
                        // many to many
                        (dml::FieldArity::List, dml::FieldArity::List) => true,
                        // one to many
                        (_, dml::FieldArity::List) => true,
                        // many to one
                        (dml::FieldArity::List, _) => false,
                        // one to one
                        (_, _) => match (model_name, related_model_name) {
                            (x, y) if x < y => true,
                            (x, y) if x > y => false,
                            // SELF RELATIONS
                            _ => field.name < related_field.name,
                        },
                    };

                    if embed_here {
                        rel.to_fields = related_model
                            .first_unique_criterion()
                            .iter()
                            .map(|f| f.name.to_owned())
                            .collect()
                    }
                }
            }
        }
    }

    /// Identifies and adds missing back relations. For 1:1 and 1:N relations.
    /// Explicit n:m relations are not touched, as they already have a back relation field.
    fn add_missing_back_relations(
        &self,
        ast_schema: &ast::SchemaAst,
        schema: &mut dml::Datamodel,
    ) -> Result<(), ErrorCollection> {
        let mut errors = ErrorCollection::new();

        let mut missing_back_relation_fields = Vec::new();
        for model in &schema.models {
            missing_back_relation_fields.append(&mut self.find_missing_back_relation_fields(&model, schema));
        }

        for missing_back_relation_field in missing_back_relation_fields {
            let model = schema
                .find_model(&missing_back_relation_field.model)
                .expect(STATE_ERROR);
            let field_name = missing_back_relation_field.field;

            if model.find_field(&field_name).is_some() {
                let source_model = schema
                    .find_model(&missing_back_relation_field.related_model)
                    .expect(STATE_ERROR);
                let source_field = source_model
                    .find_field(&missing_back_relation_field.related_field)
                    .expect(STATE_ERROR);
                errors.push(field_validation_error(
                                "Automatic related field generation would cause a naming conflict. Please add an explicit opposite relation field.",
                                &source_model,
                                &source_field,
                                &ast_schema,
                            ));
            } else {
                let model_mut = schema
                    .find_model_mut(&missing_back_relation_field.model)
                    .expect(STATE_ERROR);

                let mut back_relation_field = dml::Field::new(
                    &field_name,
                    dml::FieldType::Relation(missing_back_relation_field.relation_info),
                );

                back_relation_field.arity = missing_back_relation_field.arity;
                model_mut.add_field(back_relation_field);
            }
        }

        if errors.has_errors() {
            Err(errors)
        } else {
            Ok(())
        }
    }

    fn find_missing_back_relation_fields(
        &self,
        model: &dml::Model,
        schema: &dml::Datamodel,
    ) -> Vec<AddMissingBackRelationField> {
        let mut result = Vec::new();
        for field in model.fields() {
            if let dml::FieldType::Relation(rel) = &field.field_type {
                let mut back_field_exists = false;

                let related_model = schema.find_model(&rel.to).expect(STATE_ERROR);
                if related_model
                    .related_field(&model.name, &rel.name, &field.name)
                    .is_some()
                {
                    back_field_exists = true;
                }

                if !back_field_exists {
                    let relation_info = dml::RelationInfo {
                        to: model.name.clone(),
                        fields: vec![],
                        to_fields: vec![],
                        name: rel.name.clone(),
                        on_delete: OnDeleteStrategy::None,
                    };

                    let arity = if field.arity.is_singular() {
                        dml::FieldArity::List
                    } else {
                        dml::FieldArity::Optional
                    };

                    result.push(AddMissingBackRelationField {
                        model: rel.to.clone(),
                        field: model.name.camel_case(),
                        arity,
                        relation_info,
                        related_model: model.name.to_string(),
                        related_field: field.name.to_string(),
                    })
                }
            }
        }

        result
    }

    fn name_unnamed_relations(&self, datamodel: &mut dml::Datamodel) {
        let unnamed_relations = self.find_unnamed_relations(&datamodel);

        for (model_name, field_name, rel_info) in unnamed_relations {
            // Embedding side.
            let field = datamodel
                .find_model_mut(&model_name)
                .expect(STATE_ERROR)
                .find_field_mut(&field_name)
                .expect(STATE_ERROR);

            if let dml::FieldType::Relation(rel) = &mut field.field_type {
                rel.name = DefaultNames::relation_name(&model_name, &rel_info.to);
            } else {
                panic!("Tried to name a non-existing relation.");
            }

            // Foreign side.
            let field = datamodel
                .find_model_mut(&rel_info.to)
                .expect(STATE_ERROR)
                .related_field_mut(&model_name, &rel_info.name, &field_name)
                .expect(STATE_ERROR);

            if let dml::FieldType::Relation(rel) = &mut field.field_type {
                rel.name = DefaultNames::relation_name(&model_name, &rel_info.to);
            } else {
                panic!("Tried to name a non-existing relation.");
            }
        }
    }

    // Returns list of model name, field name and relation info.
    fn find_unnamed_relations(&self, datamodel: &dml::Datamodel) -> Vec<(String, String, dml::RelationInfo)> {
        let mut rels = Vec::new();

        for model in datamodel.models() {
            for field in model.fields() {
                if let dml::FieldType::Relation(rel) = &field.field_type {
                    let related_model = datamodel.find_model(&rel.to).expect(STATE_ERROR);
                    let related_field = related_model
                        .related_field(&model.name, &rel.name, &field.name)
                        .expect(STATE_ERROR);

                    if let dml::FieldType::Relation(related_rel) = &related_field.field_type {
                        if rel.name.is_empty()
                            && !rel.to_fields.is_empty()
                            // Tie is used to prevent duplicates on n:m relation.
                            && (related_rel.to_fields.is_empty() || tie(&model, &field, &related_model, &related_field))
                        {
                            rels.push((model.name.clone(), field.name.clone(), rel.clone()))
                        }
                    } else {
                        panic!(STATE_ERROR);
                    }
                }
            }
        }

        rels
    }

    fn populate_datasource_fields(&self, datamodel: &mut dml::Datamodel) {
        let mut datasource_fields_to_push: Vec<AddDatasourceField> = Vec::new();
        for model in datamodel.models() {
            for field in model.fields() {
                let datasource_fields = match &field.field_type {
                    dml::FieldType::Base(scalar_type, _) => {
                        self.get_datasource_fields_for_scalar_field(&field, &scalar_type)
                    }
                    dml::FieldType::Enum(_) => {
                        // TODO: why i do not need the enum name here? Seems fishy to ignore that.
                        self.get_datasource_fields_for_enum_field(&field)
                    }
                    dml::FieldType::Relation(rel_info) => {
                        //                        self.get_datasource_fields_for_relation_field(&field, &rel_info, &datamodel)
                        self.get_datasource_fields_for_relation_field_new(&model, &field, &rel_info)
                    }
                    dml::FieldType::ConnectorSpecific(_) => {
                        unimplemented!("ConnectorSpecific is not supported here as it will be removed soon.")
                    }
                    dml::FieldType::Unsupported(_) => panic!("These should always be commented out"),
                };
                datasource_fields.into_iter().for_each(|ds_field| {
                    datasource_fields_to_push.push(AddDatasourceField {
                        model: model.name.clone(),
                        field: field.name.clone(),
                        datasource_field: ds_field,
                    })
                });
            }
        }

        datasource_fields_to_push.into_iter().for_each(|add_ds_field| {
            let AddDatasourceField {
                model,
                field,
                datasource_field,
            } = add_ds_field;
            let field = datamodel
                .find_model_mut(&model)
                .unwrap()
                .find_field_mut(&field)
                .unwrap();
            field.data_source_fields.push(datasource_field);
        });
    }

    fn get_datasource_fields_for_scalar_field(
        &self,
        field: &dml::Field,
        scalar_type: &dml::ScalarType,
    ) -> Vec<DataSourceField> {
        let datasource_field = dml::DataSourceField {
            name: field.final_single_database_name().to_owned(),
            field_type: scalar_type.clone(),
            arity: field.arity,
            default_value: field.default_value.clone(),
        };
        vec![datasource_field]
    }

    fn get_datasource_fields_for_enum_field(&self, field: &dml::Field) -> Vec<DataSourceField> {
        let datasource_field = dml::DataSourceField {
            name: field.final_single_database_name().to_owned(),
            field_type: dml::ScalarType::String,
            arity: field.arity,
            default_value: field.default_value.clone(),
        };
        vec![datasource_field]
    }

    fn get_datasource_fields_for_relation_field_new(
        &self,
        model: &dml::Model,
        field: &dml::Field,
        rel_info: &dml::RelationInfo,
    ) -> Vec<DataSourceField> {
        rel_info
            .fields
            .iter()
            .map(|base_field| {
                let referenced_field = model.find_field(&base_field).expect(&format!(
                    "the field {} was not found in the model {}",
                    &base_field, &model.name
                ));

                match &referenced_field.field_type {
                    dml::FieldType::Base(scalar_type, _) => dml::DataSourceField {
                        name: referenced_field.final_single_database_name().to_owned(),
                        field_type: *scalar_type,
                        arity: field.arity,
                        default_value: None,
                    },
                    x => unimplemented!("This must be a scalar type: {:?}", x),
                }
            })
            .collect()
    }

    #[allow(unused)]
    fn get_datasource_fields_for_relation_field(
        &self,
        field: &dml::Field,
        rel_info: &dml::RelationInfo,
        datamodel: &dml::Datamodel,
    ) -> Vec<DataSourceField> {
        let final_db_names = self.final_db_names_for_relation_field(&field, &rel_info, &datamodel);
        // dbg!(&final_db_names);
        let to_fields_and_db_names = rel_info.to_fields.iter().zip(final_db_names.iter());

        to_fields_and_db_names
            .map(|(to_field, db_name)| {
                let related_model = datamodel.find_model(&rel_info.to).expect(STATE_ERROR);
                let referenced_field = related_model.find_field(&to_field).expect(STATE_ERROR);

                match &referenced_field.field_type {
                    dml::FieldType::Base(scalar_type, _) => {
                        let ds_field = dml::DataSourceField {
                            name: db_name.clone(),
                            field_type: *scalar_type,
                            arity: match field.arity {
                                // FIXME: superior hack. Talk to Marcus. This is a workaround for the behavior in row.rs for trait `ToSqlRow`
                                FieldArity::List => FieldArity::Optional,
                                x => x,
                            },
                            default_value: None,
                        };
                        vec![ds_field]
                    }
                    dml::FieldType::Relation(rel_info) => {
                        let mut x =
                            self.get_datasource_fields_for_relation_field(&referenced_field, &rel_info, &datamodel);
                        x.iter_mut().for_each(|ds_field| ds_field.name = db_name.to_owned());
                        x
                    }
                    x => unimplemented!("This must be a scalar type: {:?}", x),
                }
            })
            .flatten()
            .collect()
    }

    #[allow(unused)]
    fn final_db_names_for_relation_field(
        &self,
        field: &dml::Field,
        relation_info: &dml::RelationInfo,
        datamodel: &dml::Datamodel,
    ) -> Vec<String> {
        if field.database_names.len() == 0 {
            // TODO: this rule must be incorporated into psl-sql-conversion.md
            if relation_info.to_fields.len() == 1 {
                vec![field.final_single_database_name().to_owned()]
            } else {
                let related_model = datamodel.find_model(&relation_info.to).expect(STATE_ERROR);
                relation_info
                    .to_fields
                    .iter()
                    .map(|to_field| {
                        let referenced_field = related_model.find_field(&to_field).expect(STATE_ERROR);
                        // TODO: calling `final_single_database_name` means that this can not work for compound relation fields
                        format!(
                            "{}_{}",
                            field.final_single_database_name(),
                            referenced_field.final_single_database_name()
                        )
                    })
                    .collect()
            }
        } else {
            // This assertion verifies that the same number of names was used in @relation(references: [..]) and @map([..])
            // This must already verified by the parser. Just making sure this is the case.
            assert_eq!(relation_info.to_fields.len(), field.database_names.len());
            field.database_names.clone()
        }
    }
}

#[derive(Debug)]
struct AddMissingBackRelationField {
    model: String,
    field: String,
    arity: dml::FieldArity,
    relation_info: dml::RelationInfo,
    related_model: String,
    related_field: String,
}

#[derive(Debug)]
struct AddDatasourceField {
    model: String,
    field: String,
    datasource_field: DataSourceField,
}
