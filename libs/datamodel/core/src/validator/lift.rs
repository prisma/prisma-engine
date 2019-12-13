use super::DirectiveBox;
use crate::{
    ast,
    common::value::ValueValidator,
    common::ScalarType,
    configuration, dml,
    error::{DatamodelError, ErrorCollection},
};
use datamodel_connector::{Connector, ExampleConnector};

/// Helper for lifting a datamodel.
///
/// When lifting, the
/// AST is converted to the real datamodel, and
/// additional semantics are attached.
pub struct LiftAstToDml {
    directives: DirectiveBox,
}

const USE_CONNECTORS_FOR_CUSTOM_TYPES: bool = false; // FEATURE FLAG

impl LiftAstToDml {
    /// Creates a new instance, with all builtin directives and
    /// the directives defined by the given sources registered.
    ///
    /// The directives defined by the given sources will be namespaced.
    pub fn with_sources(sources: &[Box<dyn configuration::Source + Send + Sync>]) -> LiftAstToDml {
        LiftAstToDml {
            directives: DirectiveBox::with_sources(sources),
        }
    }

    pub fn lift(&self, ast_schema: &ast::SchemaAst) -> Result<dml::Datamodel, ErrorCollection> {
        let mut schema = dml::Datamodel::new();
        let mut errors = ErrorCollection::new();

        for ast_obj in &ast_schema.tops {
            match ast_obj {
                ast::Top::Enum(en) => match self.lift_enum(&en) {
                    Ok(en) => schema.add_enum(en),
                    Err(mut err) => errors.append(&mut err),
                },
                ast::Top::Model(ty) => match self.lift_model(&ty, ast_schema) {
                    Ok(md) => schema.add_model(md),
                    Err(mut err) => errors.append(&mut err),
                },
                ast::Top::Source(_) => { /* Source blocks are explicitly ignored by the validator */ }
                ast::Top::Generator(_) => { /* Generator blocks are explicitly ignored by the validator */ }
                // TODO: For now, type blocks are never checked on their own.
                ast::Top::Type(_) => { /* Type blocks are inlined */ }
            }
        }

        if errors.has_errors() {
            Err(errors)
        } else {
            Ok(schema)
        }
    }

    /// Internal: Validates a model AST node and lifts it to a DML model.
    fn lift_model(&self, ast_model: &ast::Model, ast_schema: &ast::SchemaAst) -> Result<dml::Model, ErrorCollection> {
        let mut model = dml::Model::new(&ast_model.name.name);
        model.documentation = ast_model.documentation.clone().map(|comment| comment.text);

        let mut errors = ErrorCollection::new();

        for ast_field in &ast_model.fields {
            match self.lift_field(ast_field, ast_schema) {
                Ok(field) => model.add_field(field),
                Err(mut err) => errors.append(&mut err),
            }
        }

        if let Err(mut err) = self.directives.model.validate_and_apply(ast_model, &mut model) {
            errors.append(&mut err);
        }

        if errors.has_errors() {
            return Err(errors);
        }

        Ok(model)
    }

    /// Internal: Validates an enum AST node.
    fn lift_enum(&self, ast_enum: &ast::Enum) -> Result<dml::Enum, ErrorCollection> {
        let mut en = dml::Enum::new(
            &ast_enum.name.name,
            ast_enum.values.iter().map(|x| x.name.clone()).collect(),
        );
        en.documentation = ast_enum.documentation.clone().map(|comment| comment.text);

        let mut errors = ErrorCollection::new();

        if let Err(mut err) = self.directives.enm.validate_and_apply(ast_enum, &mut en) {
            errors.append(&mut err);
        }

        if errors.has_errors() {
            Err(errors)
        } else {
            Ok(en)
        }
    }

    /// Internal: Lift a field AST node to a DML field.
    fn lift_field(&self, ast_field: &ast::Field, ast_schema: &ast::SchemaAst) -> Result<dml::Field, ErrorCollection> {
        let mut errors = ErrorCollection::new();
        // If we cannot parse the field type, we exit right away.
        let (field_type, extra_attributes) = self.lift_field_type(&ast_field, ast_schema, &mut Vec::new())?;

        let mut field = dml::Field::new(&ast_field.name.name, field_type.clone());
        field.documentation = ast_field.documentation.clone().map(|comment| comment.text);
        field.arity = self.lift_field_arity(&ast_field.arity);

        if let Some(value) = &ast_field.default_value {
            let validator = ValueValidator::new(value)?;

            if let dml::FieldType::Base(base_type) = &field_type {
                match validator.as_type(*base_type) {
                    Ok(val) => field.default_value = Some(val),
                    Err(err) => errors.push(err),
                };
            } else {
                errors.push(DatamodelError::new_validation_error(
                    "Found default value for a non-scalar type.",
                    validator.span(),
                ))
            }
        }

        // We merge arttributes so we can fail on duplicates.
        let attributes = [&extra_attributes[..], &ast_field.directives[..]].concat();

        if let Err(mut err) = self.directives.field.validate_and_apply(&attributes, &mut field) {
            errors.append(&mut err);
        }

        if errors.has_errors() {
            Err(errors)
        } else {
            Ok(field)
        }
    }

    /// Internal: Lift a field's arity.
    fn lift_field_arity(&self, ast_field: &ast::FieldArity) -> dml::FieldArity {
        match ast_field {
            ast::FieldArity::Required => dml::FieldArity::Required,
            ast::FieldArity::Optional => dml::FieldArity::Optional,
            ast::FieldArity::List => dml::FieldArity::List,
        }
    }

    /// Internal: Lift a field's type.
    /// Auto resolves custom types and gathers directives, but without a stack overflow please.
    fn lift_field_type(
        &self,
        ast_field: &ast::Field,
        ast_schema: &ast::SchemaAst,
        checked_types: &mut Vec<String>,
    ) -> Result<(dml::FieldType, Vec<ast::Directive>), DatamodelError> {
        let type_name = &ast_field.field_type.name;

        if let Ok(scalar_type) = ScalarType::from_str(type_name) {
            if USE_CONNECTORS_FOR_CUSTOM_TYPES {
                let pg_connector = ExampleConnector::postgres();
                let args = vec![]; // TODO: figure out args
                let pg_type_specification = ast_field
                    .directives
                    .iter()
                    .find(|dir| dir.name.name.starts_with("pg.")) // we use find because there should be at max 1.
                    .map(|dir| dir.name.name.trim_start_matches("pg."));

                if let Some(x) = pg_type_specification.and_then(|ts| pg_connector.calculate_type(&ts, args)) {
                    let field_type = dml::FieldType::ConnectorSpecific(x);
                    Ok((field_type, vec![]))
                } else {
                    Ok((dml::FieldType::Base(scalar_type), vec![]))
                }
            } else {
                Ok((dml::FieldType::Base(scalar_type), vec![]))
            }
        } else if ast_schema.find_model(type_name).is_some() {
            Ok((dml::FieldType::Relation(dml::RelationInfo::new(type_name)), vec![]))
        } else if ast_schema.find_enum(type_name).is_some() {
            Ok((dml::FieldType::Enum(type_name.clone()), vec![]))
        } else {
            self.resolve_custom_type(ast_field, ast_schema, checked_types)
        }
    }

    fn resolve_custom_type(
        &self,
        ast_field: &ast::Field,
        ast_schema: &ast::SchemaAst,
        checked_types: &mut Vec<String>,
    ) -> Result<(dml::FieldType, Vec<ast::Directive>), DatamodelError> {
        let type_name = &ast_field.field_type.name;

        if checked_types.iter().any(|x| x == type_name) {
            // Recursive type.
            return Err(DatamodelError::new_validation_error(
                &format!(
                    "Recursive type definitions are not allowed. Recursive path was: {} -> {}",
                    checked_types.join(" -> "),
                    type_name
                ),
                ast_field.field_type.span,
            ));
        }

        if let Some(custom_type) = ast_schema.find_type_alias(&type_name) {
            checked_types.push(custom_type.name.name.clone());
            let (field_type, mut attrs) = self.lift_field_type(custom_type, ast_schema, checked_types)?;

            if let dml::FieldType::Relation(_) = field_type {
                return Err(DatamodelError::new_validation_error(
                    "Only scalar types can be used for defining custom types.",
                    custom_type.field_type.span,
                ));
            }

            attrs.append(&mut custom_type.directives.clone());
            Ok((field_type, attrs))
        } else if USE_CONNECTORS_FOR_CUSTOM_TYPES {
            let pg_connector = ExampleConnector::postgres();
            let args = vec![]; // TODO: figure out args

            if let Some(x) = pg_connector.calculate_type(&ast_field.field_type.name, args) {
                let field_type = dml::FieldType::ConnectorSpecific(x);
                Ok((field_type, vec![]))
            } else {
                Err(DatamodelError::new_type_not_found_error(
                    type_name,
                    ast_field.field_type.span,
                ))
            }
        } else {
            Err(DatamodelError::new_type_not_found_error(
                type_name,
                ast_field.field_type.span,
            ))
        }
    }
}
