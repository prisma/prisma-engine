use crate::error::DatamodelError;
use crate::validator::directive::{Args, DirectiveValidator};
use crate::{ast, dml};

/// Prismas builtin `@sequence` directive.
pub struct SequenceDirectiveValidator {}

impl DirectiveValidator<dml::Field> for SequenceDirectiveValidator {
    fn directive_name(&self) -> &'static str {
        &"sequence"
    }
    fn validate_and_apply(&self, args: &mut Args, obj: &mut dml::Field) -> Result<(), DatamodelError> {
        // TODO: Handle fields according to tests:
        // https://github.com/prisma/prisma/blob/master/server/servers/deploy/src/test/scala/com/prisma/deploy/migration/validation/SequenceDirectiveSpec.scala

        let mut seq = dml::Sequence {
            name: "".to_string(),
            allocation_size: 0,
            initial_value: 0,
        };

        match args.arg("name")?.as_str() {
            Ok(name) => seq.name = name,
            Err(err) => return Err(self.parser_error(&err)),
        }

        match args.arg("allocationSize")?.as_int() {
            Ok(allocation_size) => seq.allocation_size = allocation_size,
            Err(err) => return Err(self.parser_error(&err)),
        }

        match args.arg("initialValue")?.as_int() {
            Ok(initial_value) => seq.initial_value = initial_value,
            Err(err) => return Err(self.parser_error(&err)),
        }

        match &mut obj.id_info {
            Some(info) => info.sequence = Some(seq),
            None => {
                return self.error(
                    "An @sequence directive can only exist on a primary id field.",
                    args.span(),
                )
            }
        }

        Ok(())
    }

    fn serialize(
        &self,
        field: &dml::Field,
        _datamodel: &dml::Datamodel,
    ) -> Result<Vec<ast::Directive>, DatamodelError> {
        if let Some(id_info) = &field.id_info {
            if let Some(seq_info) = &id_info.sequence {
                let mut args = Vec::new();

                args.push(ast::Argument::new_string("name", &seq_info.name));
                args.push(ast::Argument::new(
                    "allocationSize",
                    dml::Value::Int(seq_info.allocation_size).into(),
                ));
                args.push(ast::Argument::new(
                    "initialValue",
                    dml::Value::Int(seq_info.initial_value).into(),
                ));

                return Ok(vec![ast::Directive::new(self.directive_name(), args)]);
            }
        }

        Ok(vec![])
    }
}
