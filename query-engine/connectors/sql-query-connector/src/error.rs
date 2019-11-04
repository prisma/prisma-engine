use connector_interface::error::*;
use failure::{Error, Fail};
use prisma_models::prelude::DomainError;
use std::string::FromUtf8Error;

#[derive(Debug, Fail)]
pub enum SqlError {
    #[fail(display = "Unique constraint failed: {}", field_name)]
    UniqueConstraintViolation { field_name: String },

    #[fail(display = "Null constraint failed: {}", field_name)]
    NullConstraintViolation { field_name: String },

    #[fail(display = "Record does not exist.")]
    RecordDoesNotExist,

    #[fail(display = "Column does not exist")]
    ColumnDoesNotExist,

    #[fail(display = "Error creating a database connection.")]
    ConnectionError(Error),

    #[fail(display = "Error querying the database: {}", _0)]
    QueryError(Error),

    #[fail(display = "The provided arguments are not supported.")]
    InvalidConnectionArguments,

    #[fail(display = "The column value was different from the model")]
    ColumnReadFailure(Error),

    #[fail(display = "Field cannot be null: {}", field)]
    FieldCannotBeNull { field: String },

    #[fail(display = "{}", _0)]
    DomainError(DomainError),

    #[fail(display = "Record not found: {}", _0)]
    RecordNotFoundForWhere(RecordFinderInfo),

    #[fail(
        display = "Violating a relation {} between {} and {}",
        relation_name, model_a_name, model_b_name
    )]
    RelationViolation {
        relation_name: String,
        model_a_name: String,
        model_b_name: String,
    },

    #[fail(
        display = "The relation {} has no record for the model {} connected to a record for the model {} on your write path.",
        relation_name, parent_name, child_name
    )]
    RecordsNotConnected {
        relation_name: String,
        parent_name: String,
        parent_where: Option<Box<RecordFinderInfo>>,
        child_name: String,
        child_where: Option<Box<RecordFinderInfo>>,
    },

    #[fail(display = "Conversion error: {}", _0)]
    ConversionError(Error),

    #[fail(display = "Database creation error: {}", _0)]
    DatabaseCreationError(&'static str),

    #[fail(display = "Database '{}' does not exist.", db_name)]
    DatabaseDoesNotExist { db_name: String },

    #[fail(display = "Access denied to database '{}'", db_name)]
    DatabaseAccessDenied { db_name: String },

    #[fail(display = "Authentication failed for user '{}'", user)]
    AuthenticationFailed { user: String },
}

impl From<tokio_postgres::error::Error> for SqlError {
    fn from(e: tokio_postgres::error::Error) -> Self {
        SqlError::ConnectionError(e.into())
    }
}

impl From<SqlError> for ConnectorError {
    fn from(sql: SqlError) -> Self {
        match sql {
            SqlError::UniqueConstraintViolation { field_name } => {
                ConnectorError::UniqueConstraintViolation { field_name }
            }
            SqlError::NullConstraintViolation { field_name } => ConnectorError::NullConstraintViolation { field_name },
            SqlError::DatabaseDoesNotExist { db_name } => ConnectorError::DatabaseDoesNotExist { db_name },
            SqlError::DatabaseAccessDenied { db_name } => ConnectorError::DatabaseAccessDenied { db_name },
            SqlError::AuthenticationFailed { user } => ConnectorError::AuthenticationFailed { user },
            SqlError::RecordDoesNotExist => ConnectorError::RecordDoesNotExist,
            SqlError::ColumnDoesNotExist => ConnectorError::ColumnDoesNotExist,
            SqlError::ConnectionError(e) => ConnectorError::ConnectionError(e),
            SqlError::InvalidConnectionArguments => ConnectorError::InvalidConnectionArguments,
            SqlError::ColumnReadFailure(e) => ConnectorError::ColumnReadFailure(e),
            SqlError::FieldCannotBeNull { field } => ConnectorError::FieldCannotBeNull { field },
            SqlError::DomainError(e) => ConnectorError::DomainError(e),
            SqlError::RecordNotFoundForWhere(info) => ConnectorError::RecordNotFoundForWhere(info),
            SqlError::RelationViolation {
                relation_name,
                model_a_name,
                model_b_name,
            } => ConnectorError::RelationViolation {
                relation_name,
                model_a_name,
                model_b_name,
            },
            SqlError::RecordsNotConnected {
                relation_name,
                parent_name,
                parent_where,
                child_name,
                child_where,
            } => ConnectorError::RecordsNotConnected {
                relation_name,
                parent_name,
                parent_where,
                child_name,
                child_where,
            },
            SqlError::ConversionError(e) => ConnectorError::ConversionError(e),
            SqlError::DatabaseCreationError(e) => ConnectorError::DatabaseCreationError(e),
            SqlError::QueryError(e) => ConnectorError::QueryError(e),
        }
    }
}

impl From<quaint::error::Error> for SqlError {
    fn from(e: quaint::error::Error) -> Self {
        match e {
            quaint::error::Error::QueryError(e) => Self::QueryError(e.into()),
            quaint::error::Error::IoError(e) => Self::ConnectionError(e.into()),
            quaint::error::Error::NotFound => Self::RecordDoesNotExist,
            quaint::error::Error::InvalidConnectionArguments => Self::InvalidConnectionArguments,

            quaint::error::Error::UniqueConstraintViolation { field_name } => {
                Self::UniqueConstraintViolation { field_name }
            }

            quaint::error::Error::NullConstraintViolation { field_name } => {
                Self::NullConstraintViolation { field_name }
            }

            quaint::error::Error::ConnectionError(e) => Self::ConnectionError(e.into()),
            quaint::error::Error::ColumnReadFailure(e) => Self::ColumnReadFailure(e.into()),
            quaint::error::Error::ColumnNotFound(_) => Self::ColumnDoesNotExist,

            e @ quaint::error::Error::ConversionError(_) => SqlError::ConversionError(e.into()),
            e @ quaint::error::Error::ResultIndexOutOfBounds { .. } => SqlError::QueryError(e.into()),
            e @ quaint::error::Error::ResultTypeMismatch { .. } => SqlError::QueryError(e.into()),
            e @ quaint::error::Error::DatabaseUrlIsInvalid { .. } => SqlError::ConnectionError(e.into()),
            e @ quaint::error::Error::DatabaseDoesNotExist { .. } => SqlError::ConnectionError(e.into()),
            e @ quaint::error::Error::AuthenticationFailed { .. } => SqlError::ConnectionError(e.into()),
            e @ quaint::error::Error::DatabaseAccessDenied { .. } => SqlError::ConnectionError(e.into()),
            e @ quaint::error::Error::DatabaseAlreadyExists { .. } => SqlError::ConnectionError(e.into()),
            e @ quaint::error::Error::ConnectTimeout { .. } => SqlError::ConnectionError(e.into()),
            e @ quaint::error::Error::Timeout => SqlError::ConnectionError(e.into()),
            e @ quaint::error::Error::TlsError { .. } => Self::ConnectionError(e.into()),
        }
    }
}

impl From<DomainError> for SqlError {
    fn from(e: DomainError) -> SqlError {
        SqlError::DomainError(e)
    }
}

impl From<serde_json::error::Error> for SqlError {
    fn from(e: serde_json::error::Error) -> SqlError {
        SqlError::ConversionError(e.into())
    }
}

impl From<url::ParseError> for SqlError {
    fn from(_: url::ParseError) -> SqlError {
        SqlError::DatabaseCreationError("Error parsing database connection string.")
    }
}

impl From<uuid::parser::ParseError> for SqlError {
    fn from(e: uuid::parser::ParseError) -> SqlError {
        SqlError::ColumnReadFailure(e.into())
    }
}

impl From<uuid::BytesError> for SqlError {
    fn from(e: uuid::BytesError) -> SqlError {
        SqlError::ColumnReadFailure(e.into())
    }
}

impl From<FromUtf8Error> for SqlError {
    fn from(e: FromUtf8Error) -> SqlError {
        SqlError::ColumnReadFailure(e.into())
    }
}
