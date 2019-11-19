use serde::Serialize;
use user_facing_error_macros::*;

#[derive(Debug, UserFacingError, Serialize)]
#[user_facing(
    code = "P1000",
    message = "\
Authentication failed against database server at `${database_host}`, the provided database credentials for `${database_user}` are not valid.

Please make sure to provide valid database credentials for the database server at `${database_host}`."
)]
pub struct IncorrectDatabaseCredentials {
    /// Database host URI
    pub database_user: String,

    /// Database user name
    pub database_host: String,
}

#[derive(Debug, UserFacingError, Serialize)]
#[user_facing(
    code = "P1001",
    message = "\
Can't reach database server at `${database_host}`:`${database_port}`

Please make sure your database server is running at `${database_host}`:`${database_port}`."
)]
pub struct DatabaseNotReachable {
    /// Database host URI
    pub database_host: String,

    /// Database port
    pub database_port: u16,
}

#[derive(Debug, UserFacingError, Serialize)]
#[user_facing(
    code = "P1002",
    message = "\
The database server at `${database_host}`:`${database_port}` was reached but timed out.

Please try again.

Please make sure your database server is running at `${database_host}`:`${database_port}`.
"
)]
pub struct DatabaseTimeout {
    /// Database host URI
    pub database_host: String,

    /// Database port
    pub database_port: String,
}

#[derive(Debug, UserFacingError, Serialize)]
#[user_facing(code = "P1003")]
pub enum DatabaseDoesNotExist {
    #[user_facing(
        message = "Database ${database_file_name} does not exist on the database server at ${database_file_path}"
    )]
    Sqlite {
        database_file_name: String,
        database_file_path: String,
    },
    #[user_facing(
        message = "Database `${database_name}.${database_schema_name}` does not exist on the database server at `${database_host}:${database_port}`."
    )]
    Postgres {
        database_name: String,
        database_schema_name: String,
        database_host: String,
        database_port: u16,
    },
    #[user_facing(
        message = "Database `${database_name}` does not exist on the database server at `${database_host}:${database_port}`."
    )]
    Mysql {
        database_name: String,
        database_host: String,
        database_port: u16,
    },
}

#[derive(Debug, UserFacingError, Serialize)]
#[user_facing(
    code = "P1004",
    message = "The downloaded/provided binary `${binary_path}` is not compiled for platform `${platform}`"
)]
pub struct IncompatibleBinary {
    /// Fully resolved path of the binary file
    binary_path: String,

    /// Identifiers for the currently identified execution environment, e.g. `native`, `windows`, `darwin` etc
    platform: String,
}

#[derive(Debug, UserFacingError, Serialize)]
#[user_facing(
    code = "P1005",
    message = "Failed to spawn the binary `${binary_path}` process for platform `${platform}`"
)]
pub struct UnableToStartTheQueryEngine {
    /// Fully resolved path of the binary file
    binary_path: String,

    /// Identifiers for the currently identified execution environment, e.g. `native`, `windows`, `darwin` etc
    platform: String,
}

#[derive(Debug, UserFacingError, Serialize)]
#[user_facing(
    code = "P1006",
    message = "\
Photon binary for current platform `${platform}` could not be found. Make sure to adjust the generator configuration in the schema.prisma file.

${generator_config}

Please run prisma2 generate for your changes to take effect.
"
)]
pub struct BinaryNotFound {
    /// Identifiers for the currently identified execution environment, e.g. `native`, `windows`, `darwin` etc
    platform: String,

    /// Details of how a generator can be added.
    generator_config: String,
}

#[derive(Debug, UserFacingError, Serialize)]
#[user_facing(
    code = "P1007",
    message = "Please try installing Prisma 2 CLI again with the `--unsafe-perm` option. <br /> Example: `npm i -g --unsafe-perm prisma2`"
)]
pub struct MissingWriteAccessToTheDownloadBinary;

#[derive(Debug, UserFacingError, Serialize)]
#[user_facing(code = "P1008", message = "Operations timed out after `${time}`")]
pub struct DatabaseOperationTimeout {
    /// Operation time in s or ms (if <1000ms)
    pub time: String,
}

#[derive(Debug, UserFacingError, Serialize)]
#[user_facing(
    code = "P1009",
    message = "Database `${database_name}` already exists on the database server at `${database_host}:${database_port}`"
)]
pub struct DatabaseAlreadyExists {
    /// Database name, append `database_schema_name` when applicable
    /// `database_schema_name`: Database schema name (For Postgres for example)
    pub database_name: String,

    /// Database host URI
    pub database_host: String,

    /// Database port
    pub database_port: u16,
}

#[derive(Debug, UserFacingError, Serialize)]
#[user_facing(
    code = "P1010",
    message = "User `${database_user}` was denied access on the database `${database_name}`"
)]
pub struct DatabaseAccessDenied {
    /// Database user name
    pub database_user: String,

    /// Database name, append `database_schema_name` when applicable
    /// `database_schema_name`: Database schema name (For Postgres for example)
    pub database_name: String,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::UserFacingError;

    #[test]
    fn database_does_not_exist_formats_properly() {
        let sqlite_err = DatabaseDoesNotExist::Sqlite {
            database_file_path: "/tmp/dev.db".into(),
            database_file_name: "dev.db".into(),
        };

        assert_eq!(
            sqlite_err.message(),
            "Database dev.db does not exist on the database server at /tmp/dev.db"
        );

        let mysql_err = DatabaseDoesNotExist::Mysql {
            database_name: "root".into(),
            database_host: "localhost".into(),
            database_port: 8888,
        };

        assert_eq!(
            mysql_err.message(),
            "Database `root` does not exist on the database server at `localhost:8888`."
        );
    }
}
