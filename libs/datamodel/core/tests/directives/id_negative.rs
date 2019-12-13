use crate::common::*;
use datamodel::{ast::Span, error::DatamodelError};

#[test]
fn id_should_error_if_the_field_is_not_required() {
    let dml = r#"
    model Model {
        id Int? @id
    }
    "#;

    let errors = parse_error(dml);

    errors.assert_is(DatamodelError::new_directive_validation_error(
        "Fields that are marked as id must be required.",
        "id",
        Span::new(36, 38),
    ));
}

#[test]
fn id_should_error_if_the_field_is_optional() {
    let dml = r#"
    model Model {
        id Int? @id
    }
    "#;

    let errors = parse_error(dml);

    errors.assert_is(DatamodelError::new_directive_validation_error(
        "Fields that are marked as id must be required.",
        "id",
        Span::new(36, 38),
    ));
}

#[test]
fn id_should_error_if_unique_and_id_are_specified() {
    let dml = r#"
    model Model {
        id Int @id @unique
    }
    "#;

    let errors = parse_error(dml);

    errors.assert_is(DatamodelError::new_directive_validation_error(
        "Fields that are marked as id should not have an additional @unique.",
        "unique",
        Span::new(39, 45),
    ));
}

#[test]
fn id_should_error_on_model_without_id() {
    let dml = r#"
    model Model {
        id String
    }
    "#;

    let errors = parse_error(dml);

    errors.assert_is(DatamodelError::new_model_validation_error(
        "Each model must have exactly one id criteria. Either mark a single field with `@id` or add a multi field id criterion with `@@id([])` to the model.",
        "Model",
        Span::new(5, 42),
    ));
}

#[test]
fn id_should_error_multiple_ids_are_provided() {
    let dml = r#"
    model Model {
        id         Int      @id
        internalId String   @id @default(uuid())
    }
    "#;

    let errors = parse_error(dml);

    errors.assert_is(DatamodelError::new_model_validation_error(
        "At most one field must be marked as the id field with the `@id` directive.",
        "Model",
        Span::new(5, 105),
    ));
}

#[test]
fn id_must_error_when_single_and_multi_field_id_is_used() {
    let dml = r#"
    model Model {
        id         Int      @id
        b          String
        
        @@id([id,b])
    }
    "#;

    let errors = parse_error(dml);

    errors.assert_is(DatamodelError::new_model_validation_error(
        "Each model must have exactly one id criteria. Either mark a single field with `@id` or add a multi field id criterion with `@@id([])` to the model.",
        "Model",
        Span::new(5, 112),
    ));
}

#[test]
fn id_must_error_when_multi_field_is_referring_to_undefined_fields() {
    let dml = r#"
    model Model {
      a String
      b String
      
      @@id([a,c])
    }
    "#;
    let errors = parse_error(dml);

    errors.assert_is(DatamodelError::new_model_validation_error(
        "The multi field id declaration refers to the unknown fields c.",
        "Model",
        Span::new(64, 73),
    ));
}

const ID_TYPE_ERROR: &str =
    "Invalid ID field. ID field must be one of: Int @id or Int @id @default(`Integer`|`autoincrement()`) for Int fields or String @id or String @id @default(`cuid()`|`uuid()`|`String`) for String fields.";

#[test]
fn id_should_error_if_the_id_field_is_not_of_valid_type() {
    let dml = r#"
    model Model {
        id DateTime @id
    }

    model Model2 {
        id Boolean @id
    }

    model Model3 {
        id Float @id
    }

    model Model4 {
        id Decimal @id
    }
    "#;

    let errors = parse_error(dml);

    errors.assert_is_at(
        0,
        DatamodelError::new_model_validation_error(ID_TYPE_ERROR, "Model", Span::new(27, 42)),
    );

    errors.assert_is_at(
        1,
        DatamodelError::new_model_validation_error(ID_TYPE_ERROR, "Model2", Span::new(77, 91)),
    );

    errors.assert_is_at(
        2,
        DatamodelError::new_model_validation_error(ID_TYPE_ERROR, "Model3", Span::new(126, 138)),
    );

    errors.assert_is_at(
        3,
        DatamodelError::new_model_validation_error(ID_TYPE_ERROR, "Model4", Span::new(173, 187)),
    );
}
