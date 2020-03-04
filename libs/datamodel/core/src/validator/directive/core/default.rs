use crate::error::DatamodelError;
use crate::validator::directive::{Args, DirectiveValidator};
use crate::validator::LowerDmlToAst;
use crate::{ast, dml};

/// Prismas builtin `@default` directive.
pub struct DefaultDirectiveValidator {}

impl DirectiveValidator<dml::Field> for DefaultDirectiveValidator {
    fn directive_name(&self) -> &'static str {
        &"default"
    }

    fn validate_and_apply(&self, args: &mut Args, field: &mut dml::Field) -> Result<(), DatamodelError> {
        // If we allow list default values, we need to adjust the types below properly for that case.
        if field.arity == dml::FieldArity::List {
            return self.new_directive_validation_error("Cannot set a default value on list field.", args.span());
        }

        if let dml::FieldType::Base(scalar_type) = field.field_type {
            let dv = args
                .default_arg("value")?
                .as_default_value(scalar_type)
                .map_err(|e| self.wrap_in_directive_validation_error(&e))?;

            field.default_value = Some(dv);
        } else if let dml::FieldType::Enum(_) = &field.field_type {
            match args.default_arg("value")?.as_constant_literal() {
                // TODO: We should also check if this value is a valid enum value. For this we need the enums -.-
                Ok(value) => {
                    field.default_value = Some(dml::DefaultValue::Single(dml::ScalarValue::ConstantLiteral(value)))
                }
                Err(err) => return Err(self.wrap_in_directive_validation_error(&err)),
            }
        } else {
            return self.new_directive_validation_error("Cannot set a default value on a relation field.", args.span());
        }

        Ok(())
    }

    fn serialize(
        &self,
        field: &dml::Field,
        _datamodel: &dml::Datamodel,
    ) -> Result<Vec<ast::Directive>, DatamodelError> {
        if let Some(default_value) = &field.default_value {
            return Ok(vec![ast::Directive::new(
                self.directive_name(),
                vec![ast::Argument::new(
                    "",
                    LowerDmlToAst::lower_default_value(default_value.clone()),
                )],
            )]);
        }

        Ok(vec![])
    }
}
