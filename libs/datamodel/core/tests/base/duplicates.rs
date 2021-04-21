use crate::common::{parse_error, ErrorAsserts};
use datamodel::{ast::Span, diagnostics::DatamodelError};

#[test]
fn fail_on_duplicate_models() {
    let dml = r#"
        model User {
            id Int @id
        }

        model User {
            id Int @id
        }
    "#;

    let errors = parse_error(dml);

    errors.assert_is(DatamodelError::new_duplicate_top_error(
        "User",
        "model",
        "model",
        Span::new(70, 74),
    ));
}

#[test]
fn fail_on_duplicate_models_with_map() {
    let dml = r#"
        model Customer {
            id Int @id

            @@map("User")
        }

        model User {
            id Int @id
        }
    "#;

    let errors = parse_error(dml);

    errors.assert_is(DatamodelError::new_duplicate_model_database_name_error(
        "User".into(),
        "Customer".into(),
        Span::new(95, 140),
    ));
}

// From issue: https://github.com/prisma/prisma/issues/1988
#[test]
fn fail_on_duplicate_models_with_relations() {
    let dml = r#"
    model Post {
      id Int @id
    }

    model Post {
      id Int @id
      categories Categories[]
    }

    model Categories {
      post Post @relation(fields:[postId], references: [id])
      postId Int
    }
    "#;

    let errors = parse_error(dml);

    errors.assert_is_at(
        0,
        DatamodelError::new_duplicate_top_error("Post", "model", "model", Span::new(52, 56)),
    );
}

#[test]
fn fail_on_model_enum_conflict() {
    let dml = r#"
    enum User {
        Admin
        Moderator
    }
    model User {
        id Int @id
    }
    "#;

    let errors = parse_error(dml);

    errors.assert_is(DatamodelError::new_duplicate_top_error(
        "User",
        "model",
        "enum",
        Span::new(65, 69),
    ));
}
#[test]
fn fail_on_model_type_conflict() {
    let dml = r#"
    type User = String
    model User {
        id Int @id
    }
    "#;

    let errors = parse_error(dml);

    errors.assert_is(DatamodelError::new_duplicate_top_error(
        "User",
        "model",
        "type",
        Span::new(34, 38),
    ));
}

#[test]
fn fail_on_enum_type_conflict() {
    let dml = r#"
    type User = String
    enum User {
        Admin
        Moderator
    }
    "#;

    let errors = parse_error(dml);

    errors.assert_is(DatamodelError::new_duplicate_top_error(
        "User",
        "enum",
        "type",
        Span::new(33, 37),
    ));
}

#[test]
fn fail_on_duplicate_field() {
    let dml = r#"
    model User {
        id Int @id
        firstName String
        firstName String
    }
    "#;

    let errors = parse_error(dml);

    errors.assert_is(DatamodelError::new_duplicate_field_error(
        "User",
        "firstName",
        Span::new(70, 79),
    ));
}

#[test]
fn fail_on_duplicate_field_with_map() {
    let dml = r#"
    model User {
        id Int @id
        firstName String
        otherName String @map("firstName")
    }
    "#;

    let errors = parse_error(dml);

    errors.assert_is(DatamodelError::new_duplicate_field_error(
        "User",
        "otherName",
        Span::new(70, 105),
    ));
}

#[test]
fn fail_on_duplicate_mapped_field_name() {
    let dml = r#"
    model User {
        id Int @id
        firstName String @map("thename")
        lastName String @map("thename")
    }
    "#;

    let errors = parse_error(dml);

    errors.assert_is(DatamodelError::new_duplicate_field_error(
        "User",
        "lastName",
        Span::new(86, 118),
    ));
}

#[test]
fn fail_on_duplicate_enum_value() {
    let dml = r#"
    enum Role {
        Admin
        Moderator
        Moderator
    }
    "#;

    let errors = parse_error(dml);

    errors.assert_is(DatamodelError::new_duplicate_enum_value_error(
        "Role",
        "Moderator",
        Span::new(57, 67),
    ));
}

#[test]
fn fail_on_reserved_name_for_enum() {
    let dml = r#"
    enum String {
        Admin
        Moderator
    }
    "#;

    let errors = parse_error(dml);

    errors.assert_is(DatamodelError::new_reserved_scalar_type_error(
        "String",
        Span::new(10, 16),
    ));
}

#[test]
fn fail_on_reserved_name_for_model() {
    let dml = r#"
    model DateTime {
        id Int @id
    }
    "#;

    let errors = parse_error(dml);

    errors.assert_is(DatamodelError::new_reserved_scalar_type_error(
        "DateTime",
        Span::new(11, 19),
    ));
}

#[test]
fn fail_on_reserved_name_fo_custom_type() {
    let dml = r#"
    type Int = String
    "#;

    let errors = parse_error(dml);

    errors.assert_is(DatamodelError::new_reserved_scalar_type_error("Int", Span::new(10, 13)));
}
