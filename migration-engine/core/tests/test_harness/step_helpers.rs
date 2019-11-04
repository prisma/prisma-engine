use datamodel::ast::FieldArity;
use migration_connector::steps::*;

pub fn create_field_step(model: &str, field: &str, type_name: &str) -> MigrationStep {
    MigrationStep::CreateField(CreateField {
        model: model.to_string(),
        field: field.to_string(),
        tpe: type_name.to_owned(),
        arity: FieldArity::Required,
    })
}

pub fn delete_field_step(model: &str, field: &str) -> MigrationStep {
    MigrationStep::DeleteField(DeleteField {
        model: model.to_string(),
        field: field.to_string(),
    })
}

pub fn create_id_directive_step(model: &str, field: &str) -> MigrationStep {
    MigrationStep::CreateDirective(CreateDirective {
        locator: DirectiveLocation {
            directive: "id".to_owned(),
            location: DirectiveType::Field {
                model: model.to_owned(),
                field: field.to_owned(),
            },
            arguments: None,
        },
    })
}

pub fn create_model_step(model: &str) -> MigrationStep {
    MigrationStep::CreateModel(CreateModel {
        model: model.to_string(),
    })
}
