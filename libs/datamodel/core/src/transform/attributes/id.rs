use super::{super::helpers::*, AttributeValidator};
use crate::diagnostics::DatamodelError;
use crate::{ast, dml};

/// Prismas builtin `@primary` attribute.
pub struct IdAttributeValidator {}

impl AttributeValidator<dml::Field> for IdAttributeValidator {
    fn attribute_name(&self) -> &'static str {
        "id"
    }

    fn validate_and_apply(&self, args: &mut Arguments<'_>, obj: &mut dml::Field) -> Result<(), DatamodelError> {
        if let dml::Field::ScalarField(sf) = obj {
            sf.is_id = true;
            Ok(())
        } else {
            self.new_attribute_validation_error(
                &format!(
                    "The field `{}` is a relation field and cannot be marked with `@{}`. Only scalar fields can be declared as id.",
                    &obj.name(),
                    self.attribute_name()
                ),
                args.span(),
            )
        }
    }

    fn serialize(&self, field: &dml::Field, _datamodel: &dml::Datamodel) -> Vec<ast::Attribute> {
        if let dml::Field::ScalarField(sf) = field {
            if sf.is_id {
                return vec![ast::Attribute::new(self.attribute_name(), Vec::new())];
            }
        }

        vec![]
    }
}

pub struct ModelLevelIdAttributeValidator {}

impl AttributeValidator<dml::Model> for ModelLevelIdAttributeValidator {
    fn attribute_name(&self) -> &str {
        "id"
    }

    fn validate_and_apply(&self, args: &mut Arguments<'_>, obj: &mut dml::Model) -> Result<(), DatamodelError> {
        obj.id_fields = args.default_arg("fields")?.as_constant_array()?;

        Ok(())
    }

    fn serialize(&self, model: &dml::Model, _datamodel: &dml::Datamodel) -> Vec<ast::Attribute> {
        if !model.id_fields.is_empty() {
            let args = vec![ast::Argument::new_array(
                "",
                model
                    .id_fields
                    .iter()
                    .map(|f| ast::Expression::ConstantValue(f.to_string(), ast::Span::empty()))
                    .collect(),
            )];

            return vec![ast::Attribute::new(self.attribute_name(), args)];
        }

        vec![]
    }
}
