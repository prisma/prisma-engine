use datamodel::{ast::Span, error::*, render_datamodel_to_string, IndexDefinition, IndexType};

use crate::common::*;

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
fn multi_field_unique_indexes_on_relation_fields_must_not_work() {
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
    errors.assert_is(DatamodelError::new_model_validation_error("The unique index definition refers to the relation fields identification. Index definitions must reference only scalar fields.", "User",Span::new(193, 217)));
}

#[test]
fn single_field_unique_on_relation_fields_must_not_work() {
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
    errors.assert_is(DatamodelError::new_directive_validation_error("The field `identification` is a relation field and cannot be marked with `unique`. Only scalar fields can be made unique.", "unique",Span::new(183, 189)));
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
fn unique_directives_must_serialize_to_valid_dml() {
    let dml = r#"
        model User {
            id        Int    @id
            firstName String
            lastName  String

            @@unique([firstName,lastName], name: "customName")
        }
    "#;
    let schema = parse(dml);

    assert!(datamodel::parse_datamodel(&render_datamodel_to_string(&schema).unwrap()).is_ok());
}
