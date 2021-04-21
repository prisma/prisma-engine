use super::{
    super::helpers::{ValueListValidator, ValueValidator},
    builtin_datasource_providers::{
        MongoDbDatasourceProvider, MsSqlDatasourceProvider, MySqlDatasourceProvider, PostgresDatasourceProvider,
        SqliteDatasourceProvider,
    },
    datasource_provider::DatasourceProvider,
};
use crate::configuration::StringFromEnvVar;
use crate::diagnostics::{DatamodelError, Diagnostics, ValidatedDatasource, ValidatedDatasources};
use crate::{ast, Datasource};
use std::collections::HashMap;

const PREVIEW_FEATURES_KEY: &str = "previewFeatures";
const SHADOW_DATABASE_URL_KEY: &str = "shadowDatabaseUrl";
const URL_KEY: &str = "url";

/// Is responsible for loading and validating Datasources defined in an AST.
pub struct DatasourceLoader {
    source_definitions: Vec<Box<dyn DatasourceProvider>>,
}

impl DatasourceLoader {
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        Self {
            source_definitions: get_builtin_datasource_providers(),
        }
    }

    /// Loads all datasources from the provided schema AST.
    /// - `ignore_datasource_urls`: datasource URLs are not parsed. They are replaced with dummy values.
    /// - `datasource_url_overrides`: datasource URLs are not parsed and overridden with the provided ones.
    pub fn load_datasources_from_ast(
        &self,
        ast_schema: &ast::SchemaAst,
        ignore_datasource_urls: bool,
        datasource_url_overrides: Vec<(String, String)>,
    ) -> Result<ValidatedDatasources, Diagnostics> {
        let mut sources = vec![];
        let mut diagnostics = Diagnostics::new();

        for src in &ast_schema.sources() {
            match self.lift_datasource(&src, ignore_datasource_urls, &datasource_url_overrides) {
                Ok(loaded_src) => {
                    diagnostics.append_warning_vec(loaded_src.warnings);
                    sources.push(loaded_src.subject)
                }
                // Lift error.
                Err(err) => {
                    for e in err.errors {
                        match e {
                            DatamodelError::ArgumentNotFound { argument_name, span } => {
                                diagnostics.push_error(DatamodelError::new_source_argument_not_found_error(
                                    argument_name.as_str(),
                                    src.name.name.as_str(),
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

        if sources.len() > 1 {
            for src in &ast_schema.sources() {
                diagnostics.push_error(DatamodelError::new_source_validation_error(
                    &"You defined more than one datasource. This is not allowed yet because support for multiple databases has not been implemented yet.".to_string(),
                    &src.name.name,
                    src.span,
                ));
            }
        }

        if diagnostics.has_errors() {
            Err(diagnostics)
        } else {
            Ok(ValidatedDatasources {
                subject: sources,
                warnings: diagnostics.warnings,
            })
        }
    }

    fn lift_datasource(
        &self,
        ast_source: &ast::SourceConfig,
        ignore_datasource_urls: bool,
        datasource_url_overrides: &[(String, String)],
    ) -> Result<ValidatedDatasource, Diagnostics> {
        let source_name = &ast_source.name.name;
        let args: HashMap<_, _> = ast_source
            .properties
            .iter()
            .map(|arg| (arg.name.name.as_str(), ValueValidator::new(&arg.value)))
            .collect();
        let diagnostics = Diagnostics::new();

        let provider_arg = args
            .get("provider")
            .ok_or_else(|| DatamodelError::new_argument_not_found_error("provider", ast_source.span))?;

        if provider_arg.is_from_env() {
            return Err(diagnostics.merge_error(DatamodelError::new_functional_evaluation_error(
                &"A datasource must not use the env() function in the provider argument.".to_string(),
                ast_source.span,
            )));
        }

        let provider = match provider_arg.as_string_literal() {
            Some(("", _)) => {
                return Err(diagnostics.merge_error(DatamodelError::new_source_validation_error(
                    "The provider argument in a datasource must not be empty",
                    source_name,
                    provider_arg.span(),
                )));
            }
            None => {
                return Err(diagnostics.merge_error(DatamodelError::new_source_validation_error(
                    "The provider argument in a datasource must be a string literal",
                    source_name,
                    provider_arg.span(),
                )));
            }
            Some((provider, _)) => provider,
        };

        let url_arg = args
            .get(URL_KEY)
            .ok_or_else(|| DatamodelError::new_argument_not_found_error(URL_KEY, ast_source.span))?;

        let override_url = datasource_url_overrides
            .iter()
            .find(|x| &x.0 == source_name)
            .map(|x| &x.1);

        let url = match (url_arg.as_str_from_env(), override_url) {
            (Err(err), _)
                if ignore_datasource_urls && err.description().contains("Expected a String value, but received") =>
            {
                return Err(diagnostics.merge_error(err));
            }
            (_, _) if ignore_datasource_urls => {
                // glorious hack. ask marcus
                StringFromEnvVar {
                    name: "url",
                    from_env_var: None,
                    value: format!("{}://", provider),
                }
            }
            (_, Some(url)) => {
                tracing::debug!("overwriting datasource `{}` with url '{}'", &source_name, &url);
                StringFromEnvVar {
                    name: "url",
                    from_env_var: None,
                    value: url.to_owned(),
                }
            }
            (Ok((env_var, url)), _) => StringFromEnvVar {
                name: "url",
                from_env_var: env_var,
                value: url.trim().to_owned(),
            },
            (Err(err), _) => {
                return Err(diagnostics.merge_error(err));
            }
        };

        validate_datasource_url(&url, source_name, &url_arg)?;

        let shadow_database_url_arg = args.get(SHADOW_DATABASE_URL_KEY);

        let shadow_database_url: Option<StringFromEnvVar> =
            if let Some(shadow_database_url_arg) = shadow_database_url_arg.as_ref() {
                let shadow_database_url = match shadow_database_url_arg.as_str_from_env() {
                    Err(err)
                        if ignore_datasource_urls
                            && err.description().contains("Expected a String value, but received") =>
                    {
                        return Err(diagnostics.merge_error(err));
                    }
                    _ if ignore_datasource_urls => {
                        // glorious hack. ask marcus
                        Some(StringFromEnvVar {
                            name: "shadow_database_url",
                            from_env_var: None,
                            value: format!("{}://", provider),
                        })
                    }

                    Ok((env_var, url)) => Some(StringFromEnvVar {
                        name: "shadow_database_url",
                        from_env_var: env_var,
                        value: url.trim().to_owned(),
                    })
                    .filter(|s| !s.value.is_empty()),

                    // We intentionally ignore the shadow database URL if it is defined in an env var that is missing.
                    Err(DatamodelError::EnvironmentFunctionalEvaluationError { .. }) => None,

                    Err(err) => {
                        return Err(diagnostics.merge_error(err));
                    }
                };

                // Temporarily disabled because of processing/hacks on URLs that make comparing the two URLs unreliable.
                // if url.value == shadow_database_url.value {
                //     return Err(
                //         diagnostics.merge_error(DatamodelError::new_shadow_database_is_same_as_main_url_error(
                //             source_name.clone(),
                //             shadow_database_url_arg.span(),
                //         )),
                //     );
                // }

                shadow_database_url
            } else {
                None
            };

        preview_features_guardrail(&args)?;

        let documentation = ast_source.documentation.as_ref().map(|comment| comment.text.clone());

        let datasource_provider = self.get_datasource_provider(&provider).ok_or_else(|| {
            diagnostics
                .clone()
                .merge_error(DatamodelError::new_datasource_provider_not_known_error(
                    provider,
                    provider_arg.span(),
                ))
        })?;

        // Validate the URL
        datasource_provider
            .validate_url(source_name, &url)
            .map_err(|err_msg| DatamodelError::new_source_validation_error(&err_msg, source_name, url_arg.span()))?;

        // Validate the shadow database URL
        if let (Some(shadow_database_url), Some(shadow_database_url_arg)) =
            (shadow_database_url.as_ref(), shadow_database_url_arg.as_ref())
        {
            datasource_provider
                .validate_shadow_database_url(source_name, shadow_database_url)
                .map_err(|err_msg| {
                    DatamodelError::new_source_validation_error(&err_msg, source_name, shadow_database_url_arg.span())
                })?;
        }

        Ok(ValidatedDatasource {
            subject: Datasource {
                name: source_name.to_string(),
                provider: provider.to_owned(),
                active_provider: datasource_provider.canonical_name().to_owned(),
                url,
                documentation,
                active_connector: datasource_provider.connector(),
                shadow_database_url,
            },
            warnings: diagnostics.warnings,
        })
    }

    fn get_datasource_provider(&self, provider: &str) -> Option<&dyn DatasourceProvider> {
        self.source_definitions
            .iter()
            .find(|sd| sd.is_provider(provider))
            .map(|b| b.as_ref())
    }
}

fn get_builtin_datasource_providers() -> Vec<Box<dyn DatasourceProvider>> {
    vec![
        Box::new(MySqlDatasourceProvider::new()),
        Box::new(PostgresDatasourceProvider::new()),
        Box::new(SqliteDatasourceProvider::new()),
        Box::new(MsSqlDatasourceProvider::new()),
        Box::new(MongoDbDatasourceProvider::new()),
    ]
}

fn preview_features_guardrail(args: &HashMap<&str, ValueValidator>) -> Result<(), DatamodelError> {
    args.get(PREVIEW_FEATURES_KEY)
        .map(|val| -> Result<_, _> { Ok((val.as_array().to_str_vec()?, val.span())) })
        .transpose()?
        .filter(|(feats, _span)| !feats.is_empty())
        .map(|(_, span)| {
            Err(DatamodelError::new_connector_error(
        "Preview features are only supported in the generator block. Please move this field to the generator block.",
        span,
    ))
        })
        .unwrap_or(Ok(()))
}

/// Validate that the `url` argument in the datasource block is not empty.
fn validate_datasource_url(
    url: &StringFromEnvVar,
    source_name: &str,
    url_arg: &ValueValidator,
) -> Result<(), DatamodelError> {
    if !url.value.is_empty() {
        return Ok(());
    }

    let suffix = match &url.from_env_var {
        Some(env_var_name) => format!(
            " The environment variable `{}` resolved to an empty string.",
            env_var_name
        ),
        None => "".to_owned(),
    };

    let msg = format!(
        "You must provide a nonempty URL for the datasource `{}`.{}",
        source_name, &suffix
    );

    Err(DatamodelError::new_source_validation_error(
        &msg,
        source_name,
        url_arg.span(),
    ))
}
