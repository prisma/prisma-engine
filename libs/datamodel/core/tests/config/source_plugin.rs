use crate::common::*;
use datamodel::{ast::Span, common::ScalarType, configuration::*, error::DatamodelError};
use datamodel_connector::{Connector, ExampleConnector};
use pretty_assertions::assert_eq;

//##########################
// Directive implementation
//##########################

//struct CustomDirective {
//    base_type: ScalarType,
//}
//
//impl DirectiveValidator<dml::Field> for CustomDirective {
//    fn directive_name(&self) -> &'static str {
//        &"mapToInt"
//    }
//    fn validate_and_apply(&self, _args: &mut Arguments, obj: &mut dml::Field) -> Result<(), DatamodelError> {
//        obj.field_type = dml::FieldType::Base(self.base_type);
//        return Ok(());
//    }
//
//    fn serialize(
//        &self,
//        _obj: &dml::Field,
//        _datamodel: &dml::Datamodel,
//    ) -> Result<Vec<datamodel::ast::Directive>, DatamodelError> {
//        Ok(Vec::new())
//    }
//}

//##########################
// Definition Boilerplate
//##########################

const CONNECTOR_NAME: &str = "customDemoSource";

struct CustomDbDefinition {}

impl CustomDbDefinition {
    pub fn new() -> CustomDbDefinition {
        CustomDbDefinition {}
    }
}

impl SourceDefinition for CustomDbDefinition {
    fn connector_type(&self) -> &'static str {
        CONNECTOR_NAME
    }

    fn create(
        &self,
        name: &str,
        url: StringFromEnvVar,
        documentation: &Option<String>,
    ) -> Result<Box<dyn Source + Send + Sync>, DatamodelError> {
        Ok(Box::new(CustomDb {
            name: String::from(name),
            url,
            _base_type: ScalarType::Int,
            documentation: documentation.clone(),
        }))
    }
}

//##########################
// Source Boilerplate
//##########################

struct CustomDb {
    name: String,
    url: StringFromEnvVar,
    _base_type: ScalarType,
    documentation: Option<String>,
}

impl Source for CustomDb {
    fn connector_type(&self) -> &str {
        CONNECTOR_NAME
    }
    fn name(&self) -> &String {
        &self.name
    }

    fn url(&self) -> &StringFromEnvVar {
        &self.url
    }
    fn set_url(&mut self, url: &str) {
        self.url = StringFromEnvVar {
            from_env_var: None,
            value: url.to_string(),
        };
    }

    fn documentation(&self) -> &Option<String> {
        &self.documentation
    }

    fn connector(&self) -> Box<dyn Connector> {
        Box::new(ExampleConnector::empty())
    }
}

//##########################
// Unit Test
//##########################

// TODO: decide whether we still need this
#[ignore]
#[test]
fn custom_plugin() {
    std::env::set_var("URL_CUSTOM_1", "https://localhost");
    let schema = parse_with_plugins(DATAMODEL, vec![Box::new(CustomDbDefinition::new())]);

    let user_model = schema.assert_has_model("User");

    user_model
        .assert_has_field("firstName")
        .assert_base_type(&ScalarType::Int);
    user_model
        .assert_has_field("lastName")
        .assert_base_type(&ScalarType::Int);
    user_model
        .assert_has_field("email")
        .assert_base_type(&ScalarType::String);

    let post_model = schema.assert_has_model("Post");

    post_model
        .assert_has_field("comments")
        .assert_base_type(&ScalarType::Int);
    post_model.assert_has_field("likes").assert_base_type(&ScalarType::Int);
}

const DATAMODEL: &str = r#"
datasource custom_1 {
    provider = "customDemoSource"
    url = env("URL_CUSTOM_1")
}

datasource custom_2 {
    provider = "customDemoSource"
    url = "https://localhost"
}


model User {
    id Int @id
    firstName String @custom_1.mapToInt
    lastName String @custom_1.mapToInt
    email String
}

model Post {
    id Int @id
    likes String @custom_2.mapToInt
    comments Int
}
"#;

#[test]
fn serialize_sources_to_dmmf() {
    std::env::set_var("URL_CUSTOM_1", "https://localhost");
    let config =
        datamodel::parse_configuration_with_sources(DATAMODEL, vec![Box::new(CustomDbDefinition::new())]).unwrap();
    let rendered = datamodel::json::mcf::render_sources_to_json(&config.datasources);

    let expected = r#"[
  {
    "name": "custom_1",
    "connectorType": "customDemoSource",
    "url": {
        "fromEnvVar": "URL_CUSTOM_1",
        "value": "https://localhost"       
    }
  },
  {
    "name": "custom_2",
    "connectorType": "customDemoSource",
    "url": {
        "fromEnvVar": null,
        "value": "https://localhost"      
    }
  }
]"#;

    println!("{}", rendered);

    assert_eq_json(&rendered, expected);
}

#[test]
fn must_forbid_env_functions_in_provider_field() {
    let schema = r#"
        datasource ds {
            provider = env("DB_PROVIDER")
            url = env("DB_URL")
        }
    "#;
    std::env::set_var("DB_PROVIDER", "postgresql");
    std::env::set_var("DB_URL", "https://localhost");
    let config = datamodel::parse_configuration_with_sources(schema, vec![]);
    assert!(config.is_err());
    let errors = config.err().expect("This must error");
    errors.assert_is(DatamodelError::new_functional_evaluation_error(
        "A datasource must not use the env() function in the provider argument.",
        Span::new(9, 108),
    ));
}

fn assert_eq_json(a: &str, b: &str) {
    let json_a: serde_json::Value = serde_json::from_str(a).expect("The String a was not valid JSON.");
    let json_b: serde_json::Value = serde_json::from_str(b).expect("The String b was not valid JSON.");

    assert_eq!(json_a, json_b);
}
