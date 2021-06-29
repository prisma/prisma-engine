use crate::common::*;
use datamodel::{ast::Span, diagnostics::*, render_datamodel_to_string, IndexDefinition, IndexType};

#[test]
fn basic_unique_index_must_work() {
    let dml = r#"
    model User {
        id        Int    @id
        firstName String
        lastName  String

        @@unique([firstName,lastName])
    }
    "#;

    let schema = parse(dml);
    let user_model = schema.assert_has_model("User");
    user_model.assert_has_index(IndexDefinition {
        name: None,
        fields: vec!["firstName".to_string(), "lastName".to_string()],
        tpe: IndexType::Unique,
    });
}

#[test]
fn multiple_unnamed_arguments_must_error() {
    let dml = r#"
    model User {
        id        Int    @id
        firstName String
        lastName  String

        @@unique(firstName,lastName)
    }
    "#;

    let errors = parse_error(dml);
    errors.assert_is(DatamodelError::new_attribute_validation_error("You provided multiple unnamed arguments. This is not possible. Did you forget the brackets? Did you mean `[firstName, lastName]`?", "unique", Span::new(108, 134)));
}

#[test]
fn multi_field_unique_indexes_on_relation_fields_must_error_and_give_nice_error_on_inline_side() {
    let dml = r#"
    model User {
        id               Int @id
        identificationId Int

        identification Identification @relation(fields: [identificationId], references:[id])

        @@unique([identification])
    }

    model Identification {
        id Int @id
    }
    "#;

    let errors = parse_error(dml);
    errors.assert_is(DatamodelError::new_model_validation_error("The unique index definition refers to the relation fields identification. Index definitions must reference only scalar fields. Did you mean `@@unique([identificationId])`?", "User",Span::new(185, 209)));
}

#[test]
fn multi_field_unique_indexes_on_relation_fields_must_error_and_give_nice_error_on_non_inline_side() {
    let dml = r#"
    model User {
        id               Int @id
        identificationId Int
        identification   Identification @relation(fields: [identificationId], references:[id])
    }

    model Identification {
        id   Int @id
        user User
        @@unique([user])
    }
    "#;

    let errors = parse_error(dml);
    // in this case the error can't give a suggestion
    errors.assert_is(DatamodelError::new_model_validation_error("The unique index definition refers to the relation fields user. Index definitions must reference only scalar fields.", "Identification",Span::new(258, 272)));
}

#[test]
fn single_field_unique_on_relation_fields_must_error_nicely_with_one_underlying_fields() {
    let dml = r#"
    model User {
        id               Int @id
        identificationId Int

        identification Identification @relation(fields: [identificationId], references:[id]) @unique
    }

    model Identification {
        id Int @id
    }
    "#;

    let errors = parse_error(dml);
    errors.assert_is(DatamodelError::new_attribute_validation_error("The field `identification` is a relation field and cannot be marked with `unique`. Only scalar fields can be made unique. Did you mean to put it on `identificationId`?", "unique", Span::new(175, 181)));
}

#[test]
fn single_field_unique_on_relation_fields_must_error_nicely_with_many_underlying_fields() {
    let dml = r#"
    model User {
        id                Int @id
        identificationId1 Int
        identificationId2 Int

        identification Identification @relation(fields: [identificationId1, identificationId2], references:[id]) @unique
    }

    model Identification {
        id1 Int
        id2 Int
        @@id([id1, id2])
    }
    "#;

    let errors = parse_error(dml);
    errors.assert_is(DatamodelError::new_attribute_validation_error("The field `identification` is a relation field and cannot be marked with `unique`. Only scalar fields can be made unique. Did you mean to provide `@@unique([identificationId1, identificationId2])`?", "unique", Span::new(227, 233)));
}

#[test]
fn single_field_unique_on_enum_field_must_work() {
    let dml = r#"
    model User {
        id        Int    @id
        role      Role   @unique
    }

    enum Role {
        Admin
        Member
    }
    "#;

    let schema = parse(dml);
    schema
        .assert_has_model("User")
        .assert_has_scalar_field("role")
        .assert_is_unique(true);
}

#[test]
fn the_name_argument_must_work() {
    let dml = r#"
    model User {
        id        Int    @id
        firstName String
        lastName  String

        @@unique([firstName,lastName], name: "MyIndexName")
    }
    "#;

    let schema = parse(dml);
    let user_model = schema.assert_has_model("User");
    user_model.assert_has_index(IndexDefinition {
        name: Some("MyIndexName".to_string()),
        fields: vec!["firstName".to_string(), "lastName".to_string()],
        tpe: IndexType::Unique,
    });
}

#[test]
fn multiple_unique_must_work() {
    let dml = r#"
    model User {
        id        Int    @id
        firstName String
        lastName  String

        @@unique([firstName,lastName])
        @@unique([firstName,lastName], name: "MyIndexName")
    }
    "#;

    let schema = parse(dml);
    let user_model = schema.assert_has_model("User");

    user_model.assert_has_index(IndexDefinition {
        name: None,
        fields: vec!["firstName".to_string(), "lastName".to_string()],
        tpe: IndexType::Unique,
    });

    user_model.assert_has_index(IndexDefinition {
        name: Some("MyIndexName".to_string()),
        fields: vec!["firstName".to_string(), "lastName".to_string()],
        tpe: IndexType::Unique,
    });
}

#[test]
fn dbgenerated_default_errors_must_not_cascade_into_other_errors() {
    let dml = r#"
    datasource ds {
        provider = "mysql"
        url = "mysql://"
    }

    model User {
        id        Int    @id
        role      Bytes
        role2     Bytes @ds.VarBinary(40) @default(dbgenerated(""))

        @@unique([role2, role])
    }
    "#;

    let error = datamodel::parse_schema(dml).map(drop).unwrap_err();
    let expectation = expect![[r#"
        [1;91merror[0m: [1mError parsing attribute "@default": dbgenerated() takes either no argument, or a single nonempty string argument.[0m
          [1;94m-->[0m  [4mschema.prisma:10[0m
        [1;94m   | [0m
        [1;94m 9 | [0m        role      Bytes
        [1;94m10 | [0m        role2     Bytes @ds.VarBinary(40) @[1;91mdefault(dbgenerated(""))[0m
        [1;94m   | [0m
    "#]];
    expectation.assert_eq(&error)
}

#[test]
fn multi_field_unique_on_native_type_fields_fields_must_work() {
    let dml = r#"
    datasource ds {
        provider = "mysql"
        url = "mysql://"
    }

    model User {
        id        Int    @id
        role      Bytes
        role2     Bytes @ds.VarBinary(40)

        @@unique([role2, role])
    }
    "#;
    parse(dml);
}

#[test]
fn multi_field_unique_indexes_on_enum_fields_must_work() {
    let dml = r#"
    model User {
        id        Int    @id
        role      Role

        @@unique([role])
    }

    enum Role {
        Admin
        Member
    }
    "#;

    let schema = parse(dml);
    let user_model = schema.assert_has_model("User");
    user_model.assert_has_index(IndexDefinition {
        name: None,
        fields: vec!["role".to_string()],
        tpe: IndexType::Unique,
    });
}

#[test]
fn must_error_when_unknown_fields_are_used() {
    let dml = r#"
    model User {
        id Int @id

        @@unique([foo,bar])
    }
    "#;

    let errors = parse_error(dml);

    errors.assert_is(DatamodelError::new_model_validation_error(
        "The unique index definition refers to the unknown fields foo, bar.",
        "User",
        Span::new(48, 65),
    ));
}

#[test]
fn must_error_when_using_the_same_field_multiple_times() {
    let dml = r#"
    model User {
        id    Int    @id
        email String @unique

        @@unique([email, email])
    }
    "#;

    let errors = parse_error(dml);

    errors.assert_is(DatamodelError::new_model_validation_error(
        "The unique index definition refers to the fields email multiple times.",
        "User",
        Span::new(83, 105),
    ));
}

#[test]
fn unique_attributes_must_serialize_to_valid_dml() {
    let dml = r#"
        model User {
            id        Int    @id
            firstName String
            lastName  String

            @@unique([firstName,lastName], name: "customName")
        }
    "#;
    let schema = parse(dml);

    assert!(datamodel::parse_datamodel(&render_datamodel_to_string(&schema)).is_ok());
}

#[test]
fn stringified_field_names_in_unique_return_nice_error() {
    let dm = r#"
        model User {
            id        Int    @id
            firstName String
            lastName  String

            @@unique(["firstName", "lastName"])
        }
    "#;

    let err = parse_error(dm);

    err.assert_is(DatamodelError::TypeMismatchError {
        expected_type: "constant literal".into(),
        received_type: "string".into(),
        raw: "firstName".into(),
        span: Span::new(136, 147),
    });
}
