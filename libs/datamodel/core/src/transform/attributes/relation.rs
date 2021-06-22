use super::{super::helpers::*, AttributeValidator};
use crate::common::RelationNames;
use crate::diagnostics::DatamodelError;
use crate::transform::attributes::field_array;
use crate::{ast, dml, Field};

/// Prismas builtin `@relation` attribute.
pub struct RelationAttributeValidator {}

impl AttributeValidator<dml::Field> for RelationAttributeValidator {
    fn attribute_name(&self) -> &'static str {
        &"relation"
    }

    fn validate_and_apply(&self, args: &mut Arguments<'_>, field: &mut dml::Field) -> Result<(), DatamodelError> {
        if let dml::Field::RelationField(rf) = field {
            if let Ok(name_arg) = args.default_arg("name") {
                let name = name_arg.as_str()?;

                if name.is_empty() {
                    return self
                        .new_attribute_validation_error("A relation cannot have an empty name.", name_arg.span());
                }

                rf.relation_info.name = name.to_owned();
            }

            if let Ok(related_fields) = args.arg("references") {
                rf.relation_info.references = related_fields.as_array().to_literal_vec()?;
            }

            if let Ok(base_fields) = args.arg("fields") {
                rf.relation_info.fields = base_fields.as_array().to_literal_vec()?;
            }

            if let Ok(map_arg) = args.default_arg("map") {
                let map = map_arg.as_str()?;

                if map.is_empty() {
                    return self.new_attribute_validation_error(
                        "A relation cannot have an empty map property.",
                        map_arg.span(),
                    );
                }

                rf.relation_info.fk_name = Some(map.to_owned());
            }

            Ok(())
        } else {
            self.new_attribute_validation_error("Invalid field type, not a relation.", args.span())
        }
    }

    fn serialize(&self, field: &dml::Field, datamodel: &dml::Datamodel) -> Vec<ast::Attribute> {
        if let dml::Field::RelationField(rf) = field {
            let mut args = Vec::new();

            let relation_info = &rf.relation_info;

            let parent_model = datamodel.find_model_by_relation_field_ref(rf).unwrap();

            let related_model = datamodel
                .find_model(&relation_info.to)
                .unwrap_or_else(|| panic!("Related model not found: {}.", relation_info.to));

            let mut all_related_ids = related_model.id_field_names();
            let has_default_name = relation_info.name
                == RelationNames::name_for_unambiguous_relation(&relation_info.to, &parent_model.name);

            if !relation_info.name.is_empty() && (!has_default_name || parent_model.name == related_model.name) {
                args.push(ast::Argument::new_string("", &relation_info.name));
            }

            let mut relation_fields = relation_info.references.clone();

            relation_fields.sort();
            all_related_ids.sort();

            if !relation_info.fields.is_empty() {
                args.push(ast::Argument::new_array("fields", field_array(&relation_info.fields)));
            }

            // if we are on the physical field
            if !relation_info.references.is_empty() {
                let is_many_to_many = match &field {
                    Field::RelationField(relation_field) => {
                        let (_, related_field) = datamodel.find_related_field(&relation_field).unwrap();
                        relation_field.arity.is_list() && related_field.arity.is_list()
                    }
                    _ => false,
                };

                if !is_many_to_many {
                    args.push(ast::Argument::new_array(
                        "references",
                        field_array(&relation_info.references),
                    ));
                }
            }

            if relation_info.on_delete != dml::OnDeleteStrategy::None {
                args.push(ast::Argument::new_constant(
                    "onDelete",
                    &relation_info.on_delete.to_string(),
                ));
            }

            if let Some(name) = &relation_info.fk_name {
                if !relation_info.fk_name_matches_default {
                    args.push(ast::Argument::new_string("map", name));
                }
            }

            if !args.is_empty() {
                return vec![ast::Attribute::new(self.attribute_name(), args)];
            }
        }

        vec![]
    }
}
