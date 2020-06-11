use crate::{error::PrismaError, PrismaResult};
use datamodel::{Configuration, Datamodel};
use serde::Deserialize;
use std::{ffi::OsStr, fs::File, io::Read};
use structopt::StructOpt;

#[derive(Debug, StructOpt, Clone)]
pub enum Subcommand {
    /// Doesn't start a server, but allows running specific commands against Prisma.
    Cli(CliOpt),
}

#[derive(Debug, Clone, StructOpt)]
pub struct DmmfToDmlInput {
    #[structopt(name = "path")]
    pub path: String,
}

#[derive(Debug, Clone, StructOpt)]
pub struct ExecuteRequestInput {
    /// GraphQL query to execute
    pub query: String,
    /// Run in the legacy GraphQL mode
    #[structopt(long)]
    pub legacy: bool,
}

#[derive(Debug, Clone, StructOpt)]
#[structopt(rename_all = "camelCase")]
pub struct GetConfigInput {
    #[structopt(long)]
    pub ignore_env_var_errors: bool,
}

#[derive(Debug, StructOpt, Clone)]
pub enum CliOpt {
    /// Output the DMMF from the loaded data model.
    Dmmf,
    /// Get the configuration from the given data model.
    GetConfig(GetConfigInput),
    /// Executes one request and then terminates.
    ExecuteRequest(ExecuteRequestInput),
}

#[derive(Debug, StructOpt, Clone)]
#[structopt(version = env!("GIT_HASH"))]
pub struct PrismaOpt {
    /// The hostname or IP the query engine should bind to.
    #[structopt(long, short = "H", default_value = "127.0.0.1")]
    pub host: String,

    /// The port the query engine should bind to.
    #[structopt(long, short, env, default_value = "4466")]
    pub port: u16,

    /// Path to the Prisma datamodel file
    #[structopt(long, env = "PRISMA_DML_PATH", parse(from_os_str = load_datamodel_file))]
    datamodel_path: Option<String>,

    /// Base64 encoded Prisma datamodel
    #[structopt(long, env = "PRISMA_DML", parse(try_from_str = parse_base64_string))]
    datamodel: Option<String>,

    /// Base64 encoded datasources, overwriting the ones in the datamodel
    #[structopt(long, env, parse(try_from_str = parse_base64_string))]
    overwrite_datasources: Option<String>,

    /// Switches query schema generation to Prisma 1 compatible mode.
    #[structopt(long, short)]
    pub legacy: bool,

    /// Enables raw SQL queries with executeRaw/queryRaw mutation
    #[structopt(long, short = "r")]
    pub enable_raw_queries: bool,

    /// Enables the GraphQL playground
    #[structopt(long, short = "g")]
    pub enable_playground: bool,

    /// Enables server debug features.
    #[structopt(long = "debug", short = "d")]
    pub enable_debug_mode: bool,

    #[structopt(subcommand)]
    pub subcommand: Option<Subcommand>,
}

#[derive(Debug, Deserialize)]
struct SourceOverride {
    name: String,
    url: String,
}

impl PrismaOpt {
    fn datamodel_str(&self) -> PrismaResult<&str> {
        let res = self
            .datamodel
            .as_ref()
            .map(|dm| dm.as_str())
            .or(self.datamodel_path.as_ref().map(|dm| dm.as_str()))
            .ok_or_else(|| {
                PrismaError::ConfigurationError(
                    "Datamodel should be provided either as path or base64-encoded string.".into(),
                )
            })?;

        Ok(res)
    }

    pub fn datamodel(&self, ignore_env_errors: bool) -> PrismaResult<Datamodel> {
        let datamodel_str = self.datamodel_str()?;

        let datamodel = if ignore_env_errors {
            datamodel::parse_datamodel_and_ignore_env_errors(datamodel_str)
        } else {
            datamodel::parse_datamodel(datamodel_str)
        };

        match datamodel {
            Err(errors) => Err(PrismaError::ConversionError(errors, datamodel_str.to_string())),
            _ => Ok(datamodel?),
        }
    }

    pub fn configuration(&self, ignore_env_errors: bool) -> PrismaResult<Configuration> {
        let datamodel_str = self.datamodel_str()?;

        let config_result = if ignore_env_errors {
            datamodel::parse_configuration_and_ignore_env_errors(datamodel_str)
        } else {
            datamodel::parse_configuration(datamodel_str)
        };

        match config_result {
            Err(errors) => Err(PrismaError::ConversionError(errors, datamodel_str.to_string())),
            Ok(mut configuration) => {
                if let Some(ref overwrites) = self.overwrite_datasources {
                    let datasource_overwrites: Vec<SourceOverride> = serde_json::from_str(&overwrites)?;

                    for datasource_override in datasource_overwrites {
                        for datasource in &mut configuration.datasources {
                            if &datasource_override.name == datasource.name() {
                                debug!(
                                    "overwriting datasource {} with url {}",
                                    &datasource_override.name, &datasource_override.url
                                );
                                datasource.set_url(&datasource_override.url);
                            }
                        }
                    }
                }
                Ok(configuration)
            }
        }
    }
}

fn parse_base64_string(s: &str) -> PrismaResult<String> {
    match base64::decode(s) {
        Ok(bytes) => String::from_utf8(bytes).map_err(|e| {
            trace!("Error decoding {} from Base64 (invalid UTF-8): {:?}", s, e);

            PrismaError::ConfigurationError("Invalid Base64".into())
        }),
        Err(e) => {
            trace!("Decoding Base64 failed (might not be encoded): {:?}", e);
            Ok(String::from(s))
        }
    }
}

fn load_datamodel_file(path: &OsStr) -> String {
    let mut f = File::open(path).expect(&format!("Could not open datamodel file {:?}", path));
    let mut datamodel = String::new();

    f.read_to_string(&mut datamodel)
        .expect(&format!("Could not read datamodel file: {:?}", path));

    datamodel
}
