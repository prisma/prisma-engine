use crate::common::*;
use datamodel::ast::Span;
use datamodel::{ast, error::DatamodelError};

#[test]
fn shound_fail_on_directive_duplication() {
    let dml = r#"
    type ID = String @id @default(cuid())

    model Model {
        id ID @id
    }
    "#;

    let error = parse_error(dml);

    error.assert_is_at(
        0,
        DatamodelError::new_duplicate_directive_error("id", ast::Span::new(23, 25)),
    );
    error.assert_is_at(
        1,
        DatamodelError::new_duplicate_directive_error("id", ast::Span::new(77, 79)),
    );
}

#[test]
fn shound_fail_on_directive_duplication_recursive() {
    let dml = r#"
    type MyStringWithDefault = String @default(cuid())
    type ID = MyStringWithDefault @id

    model Model {
        id ID @default(cuid())
    }
    "#;

    let error = parse_error(dml);

    error.assert_is_at(
        0,
        DatamodelError::new_duplicate_directive_error("default", ast::Span::new(40, 47)),
    );
    error.assert_is_at(
        1,
        DatamodelError::new_duplicate_directive_error("default", ast::Span::new(128, 135)),
    );
}

#[test]
fn should_fail_on_endless_recursive_type_def() {
    let dml = r#"
    type MyString = ID
    type MyStringWithDefault = MyString
    type ID = MyStringWithDefault

    model Model {
        id ID 
    }
    "#;

    let error = parse_error(dml);

    error.assert_is(DatamodelError::new_validation_error(
        "Recursive type definitions are not allowed. Recursive path was: ID -> MyStringWithDefault -> MyString -> ID.",
        ast::Span::new(21, 23),
    ));
}

#[test]
fn shound_fail_on_unresolvable_type() {
    let dml = r#"
    type MyString = Hugo
    type MyStringWithDefault = MyString
    type ID = MyStringWithDefault

    model Model {
        id ID 
    }
    "#;

    let error = parse_error(dml);

    error.assert_is(DatamodelError::new_type_not_found_error("Hugo", ast::Span::new(21, 25)));
}

#[test]
fn should_fail_on_custom_related_types() {
    let dml = r#"
    type UserViaEmail = User @relation(references: email)
    type UniqueString = String @unique

    model User {
        id Int @id
        email UniqueString
        posts Post[]
    }

    model Post {
        id Int @id
        user UserViaEmail
    }
    "#;

    let error = parse_error(dml);

    error.assert_is(DatamodelError::new_validation_error(
        "Only scalar types can be used for defining custom types.",
        ast::Span::new(25, 29),
    ));
}

#[test]
fn should_fail_on_native_type_with_invalid_number_of_arguments() {
    let dml = r#"
        datasource pg {
          provider = "postgres"
          url = "postgresql://"
          previewFeatures = ["nativeTypes"]
        }

        model Blog {
            id     Int    @id
            bigInt Int    @pg.BigInt
            foobar String @pg.VarChar()
        }
    "#;

    let error = parse_error(dml);

    error.assert_is(DatamodelError::new_connector_error(
        "Native type VarChar takes 1 arguments, but received 0.",
        ast::Span::new(259, 271),
    ));
}

#[test]
fn should_fail_on_native_type_with_invalid_arguments() {
    let dml = r#"
        datasource pg {
          provider = "postgres"
          url = "postgresql://"
          previewFeatures = ["nativeTypes"]
        }

        model Blog {
            id     Int    @id
            bigInt Int    @pg.BigInt
            foobar String @pg.VarChar(a)
        }
    "#;

    let error = parse_error(dml);

    error.assert_is(DatamodelError::new_connector_error(
        "Native type VarChar takeks 1 arguments, but received 0.",
        ast::Span::new(259, 271),
    ));
}
