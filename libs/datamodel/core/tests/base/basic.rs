use crate::common::*;
use datamodel::common::ScalarType;

#[test]
fn parse_basic_model() {
    let dml = r#"
    model User {
        id Int @id
        firstName String
        lastName String
    }
    "#;

    let schema = parse(dml);
    let user_model = schema.assert_has_model("User");
    user_model.assert_is_embedded(false);
    user_model
        .assert_has_field("firstName")
        .assert_base_type(&ScalarType::String);
    user_model
        .assert_has_field("lastName")
        .assert_base_type(&ScalarType::String);
}

#[test]
fn parse_basic_enum() {
    let dml = r#"
    enum Roles {
        Admin
        User
        USER
        ADMIN
        ADMIN_USER
        Admin_User
        HHorse99
    }
    "#;

    let schema = parse(dml);
    let role_enum = schema.assert_has_enum("Roles");
    role_enum.assert_has_value("ADMIN");
    role_enum.assert_has_value("USER");
    role_enum.assert_has_value("User");
    role_enum.assert_has_value("Admin");
    role_enum.assert_has_value("ADMIN_USER");
    role_enum.assert_has_value("Admin_User");
    role_enum.assert_has_value("HHorse99");
}

#[test]
fn parse_comments() {
    let dml = r#"
    // The user model.
    model User {
        id Int @id
        // The first name.
        // Can be multi-line.
        firstName String
        lastName String
    }
    "#;

    let schema = parse(dml);
    let user_model = schema.assert_has_model("User");
    user_model.assert_with_documentation("The user model.");
    user_model
        .assert_has_field("firstName")
        .assert_with_documentation("The first name.\nCan be multi-line.");
}
