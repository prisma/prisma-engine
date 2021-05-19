use crate::{error::ApiError, logger::ChannelLogger};
use datamodel::{diagnostics::ValidatedConfiguration, Datamodel};
use napi::threadsafe_function::ThreadsafeFunction;
use opentelemetry::global;
use prisma_models::DatamodelConverter;
use query_core::{exec_loader, schema_builder, BuildMode, QueryExecutor, QuerySchema, QuerySchemaRenderer};
use request_handlers::{
    dmmf::{self, DataModelMetaFormat},
    GraphQLSchemaRenderer, GraphQlBody, GraphQlHandler, PrismaResponse,
};
use serde::{Deserialize, Deserializer, Serialize};
use std::{
    collections::{BTreeMap, HashMap},
    path::PathBuf,
    str::FromStr,
    sync::Arc,
};
use tokio::sync::RwLock;
use tracing::{metadata::LevelFilter, Level};
use tracing_futures::Instrument;
use tracing_opentelemetry::OpenTelemetrySpanExt;

/// The main engine, that can be cloned between threads when using JavaScript
/// promises.
#[derive(Clone)]
pub struct QueryEngine {
    inner: Arc<RwLock<Inner>>,
}

/// The state of the engine.
pub enum Inner {
    /// Not connected, holding all data to form a connection.
    Builder(EngineBuilder),
    /// A connected engine, holding all data to disconnect and form a new
    /// connection. Allows querying when on this state.
    Connected(ConnectedEngine),
}

/// Holding the information to reconnect the engine if needed.
#[derive(Debug, Clone)]
struct EngineDatamodel {
    datasource_overrides: Vec<(String, String)>,
    ast: Datamodel,
    raw: String,
}

/// Everything needed to connect to the database and have the core running.
pub struct EngineBuilder {
    datamodel: EngineDatamodel,
    config: ValidatedConfiguration,
    logging: LoggingOptions,
    config_dir: PathBuf,
}

#[derive(Clone)]
pub enum LoggingOptions {
    Builder {
        telemetry: TelemetryOptions,
        log_callback: ThreadsafeFunction<String>,
        log_level: LevelFilter,
    },
    Logger(ChannelLogger),
}

/// Internal structure for querying and reconnecting with the engine.
pub struct ConnectedEngine {
    datamodel: EngineDatamodel,
    config: serde_json::Value,
    query_schema: Arc<QuerySchema>,
    executor: crate::Executor,
    logger: ChannelLogger,
    config_dir: PathBuf,
}

/// Returned from the `serverInfo` method in javascript.
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ServerInfo {
    commit: String,
    version: String,
    primary_connector: Option<String>,
}

impl ConnectedEngine {
    /// The schema AST for Query Engine core.
    pub fn query_schema(&self) -> &Arc<QuerySchema> {
        &self.query_schema
    }

    /// The query executor.
    pub fn executor(&self) -> &(dyn QueryExecutor + Send + Sync) {
        &*self.executor
    }
}

/// Parameters defining the construction of an engine.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ConstructorOptions {
    datamodel: String,
    #[serde(deserialize_with = "deserialize_log_level")]
    log_level: LevelFilter,
    #[serde(default)]
    datasource_overrides: BTreeMap<String, String>,
    #[serde(default)]
    telemetry: TelemetryOptions,
    config_dir: PathBuf,
}

#[derive(Clone, Debug, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct TelemetryOptions {
    enabled: bool,
    endpoint: Option<String>,
}

fn deserialize_log_level<'de, D>(deserializer: D) -> Result<LevelFilter, D::Error>
where
    D: Deserializer<'de>,
{
    let buf = String::deserialize(deserializer)?;
    LevelFilter::from_str(&buf).map_err(serde::de::Error::custom)
}

impl QueryEngine {
    /// Parse a validated datamodel and configuration to allow connecting later on.
    ///
    /// Here we can't do anything runtime-specific, especially logging with
    /// telemetry which causes trouble if we have no runtime enabled...
    ///
    /// JavaScript constructors can't be async, so we manage a proper workflow
    /// with an event machine that can be either a builder or a connected
    /// engine.
    pub fn new(opts: ConstructorOptions, log_callback: ThreadsafeFunction<String>) -> crate::Result<Self> {
        set_panic_hook();

        let ConstructorOptions {
            datamodel,
            log_level,
            datasource_overrides,
            telemetry,
            config_dir,
        } = opts;

        let overrides: Vec<(_, _)> = datasource_overrides.into_iter().collect();
        let mut config = datamodel::parse_configuration_with_url_overrides(&datamodel, overrides.clone())
            .map_err(|errors| ApiError::conversion(errors, &datamodel))?;

        config.subject = config
            .subject
            .validate_that_one_datasource_is_provided()
            .map_err(|errors| ApiError::conversion(errors, &datamodel))?;

        config
            .subject
            .resolve_datasource_urls_from_env()
            .map_err(|errors| ApiError::conversion(errors, &datamodel))?;

        let ast = datamodel::parse_datamodel(&datamodel)
            .map_err(|errors| ApiError::conversion(errors, &datamodel))?
            .subject;

        let datamodel = EngineDatamodel {
            ast,
            raw: datamodel,
            datasource_overrides: overrides,
        };

        // Do not start logger yet. If the telemetry is enabled, it'll run tonic
        // grpc server that panics when it cannot find a runtime.
        let logging = LoggingOptions::Builder {
            telemetry,
            log_callback,
            log_level,
        };

        let builder = EngineBuilder {
            config,
            datamodel,
            logging,
            config_dir,
        };

        Ok(Self {
            inner: Arc::new(RwLock::new(Inner::Builder(builder))),
        })
    }

    /// Connect to the database, allow queries to be run.
    pub async fn connect(&self) -> crate::Result<()> {
        let mut inner = self.inner.write().await;

        match *inner {
            Inner::Builder(ref builder) => {
                // Either use the already initialized logger, or initialize one
                // now when we know we have started the Tokio in the N-API code.
                let logger = match builder.logging.clone() {
                    LoggingOptions::Builder {
                        telemetry,
                        log_callback,
                        log_level,
                    } => {
                        if telemetry.enabled {
                            // Talks OTLP and to the JavaScript callback. Requires a runtime.
                            ChannelLogger::new_with_telemetry(log_callback, telemetry.endpoint)
                        } else {
                            // Talks to the JavaScript callback. Doesn't require a runtime.
                            ChannelLogger::new(log_level, log_callback)
                        }
                    }
                    LoggingOptions::Logger(logger) => logger,
                };

                let engine = logger
                    .clone()
                    .with_logging(|| async move {
                        let template = DatamodelConverter::convert(&builder.datamodel.ast);

                        // We only support one data source at the moment, so
                        // take the first one (default not exposed yet).
                        let data_source = builder
                            .config
                            .subject
                            .datasources
                            .first()
                            .ok_or_else(|| ApiError::configuration("No valid data source found"))?;

                        let preview_features: Vec<_> = builder.config.subject.preview_features().cloned().collect();
                        let url = data_source
                            .load_url_with_config_dir(&builder.config_dir)
                            .map_err(|err| crate::error::ApiError::Conversion(err, builder.datamodel.raw.clone()))?;

                        let (db_name, executor) = exec_loader::load(&data_source, &preview_features, &url).await?;
                        let connector = executor.primary_connector();
                        connector.get_connection().await?;

                        // Build internal data model
                        let internal_data_model = template.build(db_name);

                        let query_schema = schema_builder::build(
                            internal_data_model,
                            BuildMode::Modern,
                            true, // enable raw queries
                            data_source.capabilities(),
                            preview_features,
                        );

                        let config = datamodel::json::mcf::config_to_mcf_json_value(&builder.config);

                        Ok(ConnectedEngine {
                            datamodel: builder.datamodel.clone(),
                            query_schema: Arc::new(query_schema),
                            logger,
                            executor,
                            config,
                            config_dir: builder.config_dir.clone(),
                        })
                    })
                    .await?;

                *inner = Inner::Connected(engine);

                Ok(())
            }
            Inner::Connected(_) => Err(ApiError::AlreadyConnected),
        }
    }

    /// Disconnect and drop the core. Can be reconnected later with `#connect`.
    pub async fn disconnect(&self) -> crate::Result<()> {
        let mut inner = self.inner.write().await;

        match *inner {
            Inner::Connected(ref engine) => {
                let config = datamodel::parse_configuration_with_url_overrides(
                    &engine.datamodel.raw,
                    engine.datamodel.datasource_overrides.clone(),
                )
                .map_err(|errors| ApiError::conversion(errors, &engine.datamodel.raw))?;

                let builder = EngineBuilder {
                    datamodel: engine.datamodel.clone(),
                    logging: LoggingOptions::Logger(engine.logger.clone()),
                    config,
                    config_dir: engine.config_dir.clone(),
                };

                *inner = Inner::Builder(builder);

                Ok(())
            }
            Inner::Builder(_) => Err(ApiError::NotConnected),
        }
    }

    /// If connected, sends a query to the core and returns the response.
    pub async fn query(&self, query: GraphQlBody, trace: HashMap<String, String>) -> crate::Result<PrismaResponse> {
        match *self.inner.read().await {
            Inner::Connected(ref engine) => {
                let span = tracing::span!(Level::INFO, "query");
                let cx = global::get_text_map_propagator(|propagator| propagator.extract(&trace));

                span.set_parent(cx);

                engine
                    .logger
                    .with_logging(|| {
                        async move {
                            let handler = GraphQlHandler::new(engine.executor(), engine.query_schema());
                            Ok(handler.handle(query).await)
                        }
                        .instrument(span)
                    })
                    .await
            }
            Inner::Builder(_) => Err(ApiError::NotConnected),
        }
    }

    /// Loads the query schema. Only available when connected.
    pub async fn sdl_schema(&self) -> crate::Result<String> {
        match *self.inner.read().await {
            Inner::Connected(ref engine) => Ok(GraphQLSchemaRenderer::render(engine.query_schema().clone())),
            Inner::Builder(_) => Err(ApiError::NotConnected),
        }
    }

    /// Loads the DMMF. Only available when connected.
    pub async fn dmmf(&self) -> crate::Result<DataModelMetaFormat> {
        match *self.inner.read().await {
            Inner::Connected(ref engine) => {
                let dmmf = dmmf::render_dmmf(&engine.datamodel.ast, engine.query_schema().clone());

                Ok(dmmf)
            }
            Inner::Builder(_) => Err(ApiError::NotConnected),
        }
    }

    /// Loads the configuration.
    pub async fn get_config(&self) -> crate::Result<serde_json::Value> {
        match *self.inner.read().await {
            Inner::Connected(ref engine) => Ok(engine.config.clone()),
            Inner::Builder(ref builder) => {
                let value = datamodel::json::mcf::config_to_mcf_json_value(&builder.config);
                Ok(value)
            }
        }
    }

    /// Info about the runnings server.
    pub async fn server_info(&self) -> crate::Result<ServerInfo> {
        match *self.inner.read().await {
            Inner::Connected(ref engine) => Ok(ServerInfo {
                commit: env!("GIT_HASH").into(),
                version: env!("CARGO_PKG_VERSION").into(),
                primary_connector: Some(engine.executor().primary_connector().name()),
            }),
            Inner::Builder(_) => Ok(ServerInfo {
                commit: env!("GIT_HASH").into(),
                version: env!("CARGO_PKG_VERSION").into(),
                primary_connector: None,
            }),
        }
    }
}

pub fn set_panic_hook() {
    let original_hook = std::panic::take_hook();

    std::panic::set_hook(Box::new(move |info| {
        let payload = info
            .payload()
            .downcast_ref::<String>()
            .map(Clone::clone)
            .unwrap_or_else(|| info.payload().downcast_ref::<&str>().unwrap().to_string());

        match info.location() {
            Some(location) => {
                tracing::event!(
                    tracing::Level::ERROR,
                    message = "PANIC",
                    reason = payload.as_str(),
                    file = location.file(),
                    line = location.line(),
                    column = location.column(),
                );
            }
            None => {
                tracing::event!(tracing::Level::ERROR, message = "PANIC", reason = payload.as_str());
            }
        }

        original_hook(info)
    }));
}
