use crate::common::*;
use datamodel::ast::Span;
use datamodel::error::DatamodelError;

#[test]
fn nice_error_for_missing_model_keyword() {
    let dml = r#"
    User {
        id Int @id
    }
    "#;

    let error = parse_error(dml);

    error.assert_is(DatamodelError::new_validation_error(
        "This block is invalid. It does not start with any known Prisma schema keyword.",
        Span::new(5, 36),
    ));
}
#[test]
fn nice_error_for_missing_model_keyword_2() {
    let dml = r#"
    model User {
        id Int @id
    }
    Todo {
        id
    }
    "#;

    let error = parse_error(dml);

    error.assert_is(DatamodelError::new_validation_error(
        "This block is invalid. It does not start with any known Prisma schema keyword.",
        Span::new(47, 70),
    ));
}

#[test]
fn nice_error_on_incorrect_enum_field() {
    let dml = r#"
    enum Role {
        A-dmin
        User
    }
    "#;

    let error = parse_error(dml);

    error.assert_is(DatamodelError::new_validation_error(
        "The character `-` is not allowed in Enum Value names.",
        Span::new(25, 31),
    ));
}

#[test]
fn nice_error_missing_type() {
    let dml = r#"
    model User {
        id Int @id
        name
    }
    "#;

    let error = parse_error(dml);

    error.assert_is(DatamodelError::new_model_validation_error(
        "This field declaration is invalid. It is either missing a name or a type.",
        "User",
        Span::new(45, 50),
    ));
}

#[test]
fn nice_error_missing_directive_name() {
    let dml = r#"
    model User {
        id Int @id @
    }
    "#;

    let error = parse_error(dml);

    error.assert_is(DatamodelError::new_validation_error(
        "The name of a Directive must not be empty.",
        Span::new(38, 38),
    ));
}

// TODO: This case is not nice because the "{ }" belong to the declaration.
#[test]
fn nice_error_missing_braces() {
    let dml = r#"
    model User
        id Int @id
    "#;

    let error = parse_error(dml);

    error.assert_length(2);
    error.assert_is_at(
        0,
        DatamodelError::new_validation_error(
            "This line is invalid. It does not start with any known Prisma schema keyword.",
            Span::new(5, 16),
        ),
    );
    error.assert_is_at(
        1,
        DatamodelError::new_validation_error(
            "This line is invalid. It does not start with any known Prisma schema keyword.",
            Span::new(24, 35),
        ),
    );
}

#[test]
fn nice_error_broken_field_type_legacy_list() {
    let dml = r#"
    model User {
        id [Int] @id
    }"#;

    let error = parse_error(dml);

    error.assert_is(DatamodelError::new_legacy_parser_error(
        "To specify a list, please use `Type[]` instead of `[Type]`.",
        Span::new(29, 34),
    ));
}

#[test]
fn nice_error_broken_field_type_legacy_colon() {
    let dml = r#"
    model User {
        id: Int @id
    }"#;

    let error = parse_error(dml);

    error.assert_is(DatamodelError::new_legacy_parser_error(
        "Field declarations don't require a `:`.",
        Span::new(28, 29),
    ));
}

#[test]
fn nice_error_broken_field_type_legacy_required() {
    let dml = r#"
    model User {
        id Int! @id
    }"#;

    let error = parse_error(dml);

    error.assert_is(DatamodelError::new_legacy_parser_error(
        "Fields are required by default, `!` is no longer required.",
        Span::new(29, 33),
    ));
}

#[test]
fn nice_error_legacy_model_decl() {
    let dml = r#"
    type User {
        id Int @id
    }"#;

    let error = parse_error(dml);

    error.assert_is(DatamodelError::new_legacy_parser_error(
        "Model declarations have to be indicated with the `model` keyword.",
        Span::new(5, 9),
    ));
}

#[test]
fn optional_list_fields_must_error() {
    let dml = r#"
    model User {
        id Int @id
        names String[]?
    }"#;

    let error = parse_error(dml);

    error.assert_is(DatamodelError::new_legacy_parser_error(
        "Optional lists are not supported. Use either `Type[]` or `Type?`.",
        Span::new(51, 60),
    ));
}
