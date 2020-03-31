use crate::common::*;
use datamodel::ast::Span;
use datamodel::dml;
use datamodel::error::DatamodelError;

#[test]
fn relation_happy_path() {
    let dml = r#"
    model User {
        id Int @id
        firstName String
        posts Post[]
    }

    model Post {
        id Int @id
        text String
        userId Int
        user User @relation(fields: [userId], references: [id])
    }
    "#;

    let schema = parse(dml);
    let user_model = schema.assert_has_model("User");
    user_model
        .assert_has_field("posts")
        .assert_arity(&dml::FieldArity::List)
        .assert_relation_to("Post")
        .assert_relation_base_fields(&[])
        .assert_relation_to_fields(&[]);

    let post_model = schema.assert_has_model("Post");
    post_model
        .assert_has_field("user")
        .assert_arity(&dml::FieldArity::Required)
        .assert_relation_to("User")
        .assert_relation_base_fields(&["userId"])
        .assert_relation_to_fields(&["id"]);
}

#[test]
fn relation_must_error_when_base_field_does_not_exist() {
    let dml = r#"
    model User {
        id Int @id
        firstName String
        posts Post[]
    }

    model Post {
        id Int @id
        text String        
        user User @relation(fields: [userId], references: [id])
    }
    "#;

    let errors = parse_error(dml);
    errors.assert_is(DatamodelError::new_validation_error("The argument fields must refer only to existing fields. The following fields do not exist in this model: userId", Span::new(162, 217)));
}

#[test]
fn relation_must_error_when_base_field_is_not_scalar() {
    let dml = r#"
    model User {
        id Int @id
        firstName String
        posts Post[]
    }

    model Post {
        id Int @id
        text String
        userId Int
        otherId Int        
        
        user User @relation(fields: [other], references: [id])
        other OtherModel @relation(fields: [otherId], references: [id])
    }
    
    model OtherModel {
        id Int @id
        posts Post[]
    }
    "#;

    let errors = parse_error(dml);
    errors.assert_is(DatamodelError::new_validation_error("The argument fields must refer only to scalar fields. But it is referencing the following relation fields: other", Span::new(210, 264)));
}

#[test]
fn optional_relation_field_must_succeed_when_all_underlying_fields_are_optional() {
    let dml = r#"
    model User {
        id        Int     @id
        firstName String?
        lastName  String?
        posts     Post[]
    }

    model Post {
        id            Int     @id
        text          String
        userFirstName String?
        userLastName  String?
          
        user          User?   @relation(fields: [userFirstName, userLastName], references: [firstName, lastName])
    }
    "#;

    // must not crash
    let _ = parse(dml);
}

#[test]
fn optional_relation_field_must_error_when_one_underlying_field_is_required() {
    let dml = r#"
    model User {
        id        Int     @id
        firstName String
        lastName  String?
        posts     Post[]
    }

    model Post {
        id            Int     @id
        text          String
        userFirstName String
        userLastName  String?
          
        user          User?   @relation(fields: [userFirstName, userLastName], references: [firstName, lastName])
    }
    "#;

    let errors = parse_error(dml);
    errors.assert_is(DatamodelError::new_validation_error("The relation field `user` uses the scalar fields userFirstName, userLastName. At least one of those fields is required. Hence the relation field must be required as well.", Span::new(289, 394)));
}

#[test]
fn required_relation_field_must_succeed_when_at_least_one_underlying_fields_is_required() {
    let dml = r#"
    model User {
        id        Int     @id
        firstName String
        lastName  String?
        posts     Post[]
    }

    model Post {
        id            Int     @id
        text          String
        userFirstName String
        userLastName  String?
          
        user          User    @relation(fields: [userFirstName, userLastName], references: [firstName, lastName])
    }
    "#;

    // must not crash
    let _ = parse(dml);
}

#[test]
fn required_relation_field_must_error_when_all_underlying_fields_are_optional() {
    let dml = r#"
    model User {
        id        Int     @id
        firstName String?
        lastName  String?
        posts     Post[]
    }

    model Post {
        id            Int     @id
        text          String
        userFirstName String?
        userLastName  String?
          
        user          User    @relation(fields: [userFirstName, userLastName], references: [firstName, lastName])
    }
    "#;

    let errors = parse_error(dml);
    errors.assert_is(DatamodelError::new_validation_error("The relation field `user` uses the scalar fields userFirstName, userLastName. All those fields are optional. Hence the relation field must be optional as well.", Span::new(291, 396)));
}

#[test]
fn relation_must_error_when_referenced_field_does_not_exist() {
    let dml = r#"
    model User {
        id Int @id
        firstName String
        posts Post[]
    }

    model Post {
        id Int @id
        text String
        userId Int        
        user User @relation(fields: [userId], references: [fooBar])
    }
    "#;

    let errors = parse_error(dml);
    errors.assert_is(DatamodelError::new_validation_error("The argument `references` must refer only to existing fields in the related model `User`. The following fields do not exist in the related model: fooBar", Span::new(181, 240)));
}

#[test]
fn relation_must_error_when_referenced_field_is_not_scalar() {
    let dml = r#"
    model User {
        id Int @id
        firstName String
        posts Post[]
    }

    model Post {
        id Int @id
        text String
        userId Int        
        user User @relation(fields: [userId], references: [posts])
    }
    "#;

    let errors = parse_error(dml);
    errors.assert_is(DatamodelError::new_validation_error("The argument `references` must refer only to scalar fields in the related model `User`. But it is referencing the following relation fields: posts", Span::new(181, 239)));
}
