use super::super::helpers::*;
use crate::{
    ast::{self, Span},
    common::preview_features::GENERATOR,
    configuration::Generator,
    diagnostics::*,
    transform::ast_to_dml::common::parse_and_validate_preview_features,
    StringFromEnvVar,
};
use std::collections::HashMap;

const PROVIDER_KEY: &str = "provider";
const OUTPUT_KEY: &str = "output";
const BINARY_TARGETS_KEY: &str = "binaryTargets";
const EXPERIMENTAL_FEATURES_KEY: &str = "experimentalFeatures";
const PREVIEW_FEATURES_KEY: &str = "previewFeatures";
const FIRST_CLASS_PROPERTIES: &[&str] = &[
    PROVIDER_KEY,
    OUTPUT_KEY,
    BINARY_TARGETS_KEY,
    EXPERIMENTAL_FEATURES_KEY,
    PREVIEW_FEATURES_KEY,
];

/// Is responsible for loading and validating Generators defined in an AST.
pub struct GeneratorLoader {}

impl GeneratorLoader {
    pub fn load_generators_from_ast(ast_schema: &ast::SchemaAst) -> Result<ValidatedGenerators, Diagnostics> {
        let mut generators: Vec<Generator> = vec![];
        let mut diagnostics = Diagnostics::new();

        for gen in &ast_schema.generators() {
            match Self::lift_generator(&gen) {
                Ok(loaded_gen) => {
                    diagnostics.append_warning_vec(loaded_gen.warnings);
                    generators.push(loaded_gen.subject)
                }
                // Lift error.
                Err(err) => {
                    for e in err.errors {
                        match e {
                            DatamodelError::ArgumentNotFound { argument_name, span } => {
                                diagnostics.push_error(DatamodelError::new_generator_argument_not_found_error(
                                    argument_name.as_str(),
                                    gen.name.name.as_str(),
                                    span,
                                ));
                            }
                            _ => {
                                diagnostics.push_error(e);
                            }
                        }
                    }
                    diagnostics.append_warning_vec(err.warnings)
                }
            }
        }

        if diagnostics.has_errors() {
            Err(diagnostics)
        } else {
            Ok(ValidatedGenerators {
                subject: generators,
                warnings: diagnostics.warnings,
            })
        }
    }

    fn lift_generator(ast_generator: &ast::GeneratorConfig) -> Result<ValidatedGenerator, Diagnostics> {
        let args: HashMap<_, _> = ast_generator
            .properties
            .iter()
            .map(|arg| (arg.name.name.as_str(), ValueValidator::new(&arg.value)))
            .collect();
        let mut diagnostics = Diagnostics::new();

        let (from_env_var, value) = args
            .get(PROVIDER_KEY)
            .ok_or_else(|| DatamodelError::new_argument_not_found_error(PROVIDER_KEY, ast_generator.span))?
            .as_str_from_env()?;

        let provider = StringFromEnvVar {
            name: PROVIDER_KEY,
            from_env_var,
            value,
        };

        let output = if let Some(arg) = args.get(OUTPUT_KEY) {
            let (from_env_var, value) = arg.as_str_from_env()?;

            Some(StringFromEnvVar {
                name: OUTPUT_KEY,
                from_env_var,
                value,
            })
        } else {
            None
        };

        let mut properties: HashMap<String, String> = HashMap::new();

        let binary_targets = match args.get(BINARY_TARGETS_KEY) {
            Some(x) => x.as_array().to_str_vec()?,
            None => Vec::new(),
        };

        // for compatibility reasons we still accept the old experimental key
        let preview_features_arg = args
            .get(PREVIEW_FEATURES_KEY)
            .or_else(|| args.get(EXPERIMENTAL_FEATURES_KEY));

        let (raw_preview_features, span) = match preview_features_arg {
            Some(x) => (x.as_array().to_str_vec()?, x.span()),
            None => (Vec::new(), Span::empty()),
        };

        let preview_features = if !raw_preview_features.is_empty() {
            let (features, mut diag) = parse_and_validate_preview_features(raw_preview_features, &GENERATOR, span);
            diagnostics.append(&mut diag);

            if diagnostics.has_errors() {
                return Err(diagnostics);
            }

            features
        } else {
            vec![]
        };

        for prop in &ast_generator.properties {
            let is_first_class_prop = FIRST_CLASS_PROPERTIES.iter().any(|k| *k == prop.name.name);
            if is_first_class_prop {
                continue;
            }

            properties.insert(prop.name.name.clone(), prop.value.to_string());
        }

        Ok(ValidatedGenerator {
            subject: Generator {
                name: ast_generator.name.name.clone(),
                provider,
                output,
                binary_targets,
                preview_features,
                config: properties,
                documentation: ast_generator.documentation.clone().map(|comment| comment.text),
            },
            warnings: diagnostics.warnings,
        })
    }
}
