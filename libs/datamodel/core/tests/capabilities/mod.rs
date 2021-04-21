use crate::common::{parse, parse_error, ErrorAsserts};
use indoc::indoc;

#[test]
fn enums_must_only_be_supported_if_all_specified_provider_supports_them() {
    // Postgres and MySQL support enums but SQLite doesn't.
    test_enum_support("mysql", false);
    test_enum_support("postgres", false);
    test_enum_support("sqlite", true);
    test_enum_support("sqlserver", true);
}

fn test_enum_support(provider: &str, must_error: bool) {
    let dml = r#"
    model Todo {
      id     Int    @id
      status Status
    }

    enum Status {
      DONE
      NOT_DONE
    }
    "#;

    let error_msg =
        "Error validating: You defined the enum `Status`. But the current connector does not support enums.";
    test_capability_support(provider, must_error, dml, error_msg);
}

#[test]
fn scalar_lists_must_only_be_supported_if_all_specified_providers_support_them() {
    // Only Postgres supports scalar lists.
    test_scalar_list_support("postgres", false);

    for provider in &["sqlserver", "sqlite", "mysql"] {
        test_scalar_list_support(provider, true);
    }
}

fn test_scalar_list_support(provider: &str, must_error: bool) {
    let dml = r#"
    model Todo {
      id     Int      @id
      tags   String[]
    }
    "#;

    let error_msg = "Field \"tags\" in model \"Todo\" can\'t be a list. The current connector does not support lists of primitive types.";
    test_capability_support(provider, must_error, dml, error_msg);
}

#[test]
fn multiple_indexes_with_the_same_name_only_supported_if_all_specified_providers_support_them() {
    // Only MySQL supports multiple indexes with the same name.

    validate_uniqueness_of_names_of_indexes("mysql", false);

    for provider in &["sqlserver", "sqlite", "postgres"] {
        validate_uniqueness_of_names_of_indexes(provider, true);
    }
}

fn validate_uniqueness_of_names_of_indexes(provider: &str, must_error: bool) {
    let dml = r#"
     model User {
        id         Int @id
        neighborId Int

        @@index([id], name: "metaId")
     }

     model Post {
        id Int @id
        optionId Int

        @@index([id], name: "metaId")
     }
    "#;

    let error_msg= "The index name `metaId` is declared multiple times. With the current connector index names have to be globally unique.";

    test_capability_support(provider, must_error, dml, error_msg);
}

#[test]
fn json_must_only_be_supported_if_all_specified_providers_support_them() {
    // Only Postgres and MySQL support JSON.
    test_json_support("postgres", false);
    test_json_support("mysql", false);

    test_json_support("sqlite", true);
    test_json_support("sqlserver", true);
}

fn test_json_support(provider: &str, must_error: bool) {
    let dml = r#"
    model Todo {
      id     Int      @id
      json   Json
    }
    "#;

    let error_msg = "Error validating field `json` in model `Todo`: Field `json` in model `Todo` can\'t be of type Json. The current connector does not support the Json type.";
    test_capability_support(provider, must_error, dml, error_msg);
}

#[test]
fn relations_over_non_unique_criteria_must_only_be_supported_if_all_specified_providers_support_them() {
    // Only MySQL supports that.
    test_relations_over_non_unique_criteria_support("postgres", true);
    test_relations_over_non_unique_criteria_support("sqlite", true);

    test_relations_over_non_unique_criteria_support("mysql", false);
}

fn test_relations_over_non_unique_criteria_support(provider: &str, must_error: bool) {
    let dml = r#"
    model Todo {
      id           Int    @id
      assigneeName String
      assignee     User   @relation(fields: assigneeName, references: name)
    }

    model User {
      id   Int    @id
      name String
      todos Todo[]
    }
    "#;

    let error_msg = "Error validating: The argument `references` must refer to a unique criteria in the related model `User`. But it is referencing the following fields that are not a unique criteria: name";
    test_capability_support(provider, must_error, dml, error_msg);
}

#[test]
fn auto_increment_on_non_primary_columns_must_only_be_supported_if_all_specified_providers_support_them() {
    test_auto_increment_on_non_primary_columns("postgres", false);
    test_auto_increment_on_non_primary_columns("mysql", false);
    test_auto_increment_on_non_primary_columns("sqlite", true);
}

#[test]
fn enforcing_key_order() {
    let dml = indoc! {r#"
        model  Todo {
          id1  Int
          id2  Int
          cats Cat[]

          @@id([id1, id2])
        }

        model Cat {
          id    Int @id
          todo1 Int
          todo2 Int

          rel Todo @relation(fields: [todo1, todo2], references: [id2, id1])
        }
    "#};

    let error_msg = "Error validating: The argument `references` must refer to a unique criteria in the related model `Todo` using the same order of fields. Please check the ordering in the following fields: `id2, id1`.";

    test_capability_support("sqlserver", true, dml, error_msg);

    for provider in &["mysql", "sqlite", "postgres"] {
        test_capability_support(provider, false, dml, error_msg);
    }
}

fn test_auto_increment_on_non_primary_columns(provider: &str, must_error: bool) {
    let dml = r#"
    model Todo {
      id           Int    @id
      non_primary  Int    @default(autoincrement()) @unique
    }
    "#;

    let error_msg = "Error parsing attribute \"@default\": The `autoincrement()` default value is used on a non-id field even though the datasource does not support this.";

    test_capability_support(provider, must_error, dml, error_msg);
}

fn test_capability_support(provider: &str, must_error: bool, datamodel: &str, error_msg: &str) {
    let dml = format!(
        r#"
    datasource db {{
      provider = "{provider}"
      url = "{url}"
    }}

    {datamodel}
    "#,
        provider = provider,
        url = format!("{}://", provider),
        datamodel = datamodel,
    );

    if must_error {
        parse_error(&dml).assert_is_message(error_msg);
    } else {
        parse(&dml);
    }
}
