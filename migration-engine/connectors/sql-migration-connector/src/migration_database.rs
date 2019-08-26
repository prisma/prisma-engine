use prisma_query::{
    ast::*,
    connector::{MysqlParams, PostgresParams, Queryable, ResultSet, SqliteParams},
    pool::{mysql::*, postgres::*, sqlite::*, PrismaConnectionManager},
};
use std::sync::Arc;
use std::{convert::TryFrom, ops::DerefMut, time::Duration};

pub trait MigrationDatabase: Send + Sync + 'static {
    fn execute(&self, db: &str, q: Query) -> prisma_query::Result<Option<Id>>;
    fn query(&self, db: &str, q: Query) -> prisma_query::Result<ResultSet>;
    fn query_raw(&self, db: &str, sql: &str, params: &[ParameterizedValue]) -> prisma_query::Result<ResultSet>;
    fn execute_raw(&self, db: &str, sql: &str, params: &[ParameterizedValue]) -> prisma_query::Result<u64>;
}

pub struct MigrationDatabaseWrapper {
    pub database: Arc<dyn MigrationDatabase + Send + Sync + 'static>,
}

impl database_introspection::IntrospectionConnection for MigrationDatabaseWrapper {
    fn query_raw(&self, sql: &str, schema: &str, params: &[ParameterizedValue]) -> prisma_query::Result<prisma_query::connector::ResultSet> {
        self.database.query_raw(schema, sql, params)
    }
}

type SqlitePool = r2d2::Pool<PrismaConnectionManager<SqliteConnectionManager>>;
type PostgresPool = r2d2::Pool<PrismaConnectionManager<PostgresManager>>;
type MysqlPool = r2d2::Pool<PrismaConnectionManager<MysqlConnectionManager>>;

pub struct Sqlite {
    pool: SqlitePool,
    pub(crate) file_path: String,
}

impl Sqlite {
    pub fn new(url: &str) -> prisma_query::Result<Self> {
        let params = SqliteParams::try_from(url)?;
        let file_path = params.file_path.to_str().unwrap().to_string();
        let manager = PrismaConnectionManager::sqlite(&file_path)?;

        let pool = r2d2::Pool::builder()
            .max_size(params.connection_limit)
            .test_on_check_out(false)
            .build(manager)?;

        Ok(Self { pool, file_path })
    }

    fn with_connection<F, T>(&self, db: &str, f: F) -> T
    where
        F: FnOnce(&mut dyn Queryable) -> T,
    {
        let mut conn = self.pool.get().unwrap();

        conn.execute_raw(
            "ATTACH DATABASE ? AS ?",
            &[
                ParameterizedValue::from(self.file_path.as_str()),
                ParameterizedValue::from(db),
            ],
        )
        .unwrap();

        let res = f(conn.deref_mut());

        conn.execute_raw("DETACH DATABASE ?", &[ParameterizedValue::from(db)])
            .unwrap();

        res
    }
}

impl MigrationDatabase for Sqlite {
    fn execute(&self, db: &str, q: Query) -> prisma_query::Result<Option<Id>> {
        self.with_connection(db, |conn| conn.execute(q))
    }

    fn query(&self, db: &str, q: Query) -> prisma_query::Result<ResultSet> {
        self.with_connection(db, |conn| conn.query(q))
    }

    fn query_raw(&self, db: &str, sql: &str, params: &[ParameterizedValue]) -> prisma_query::Result<ResultSet> {
        self.with_connection(db, |conn| conn.query_raw(sql, params))
    }

    fn execute_raw(&self, db: &str, sql: &str, params: &[ParameterizedValue]) -> prisma_query::Result<u64> {
        self.with_connection(db, |conn| conn.execute_raw(sql, params))
    }
}

pub struct PostgreSql {
    pool: PostgresPool,
}

impl PostgreSql {
    pub fn new(params: PostgresParams) -> prisma_query::Result<Self> {
        let manager = PrismaConnectionManager::postgres(params.config, Some(params.schema))?;

        let pool = r2d2::Pool::builder()
            .max_size(params.connection_limit)
            .connection_timeout(Duration::from_millis(1500))
            .test_on_check_out(false)
            .build(manager)?;

        Ok(Self { pool })
    }

    fn with_connection<F, T>(&self, f: F) -> T
    where
        F: FnOnce(&mut dyn Queryable) -> T,
    {
        let mut conn = self.pool.get().unwrap();
        f(conn.deref_mut())
    }
}

impl MigrationDatabase for PostgreSql {
    fn execute(&self, _: &str, q: Query) -> prisma_query::Result<Option<Id>> {
        self.with_connection(|conn| conn.execute(q))
    }

    fn query(&self, _: &str, q: Query) -> prisma_query::Result<ResultSet> {
        self.with_connection(|conn| conn.query(q))
    }

    fn query_raw(&self, _: &str, sql: &str, params: &[ParameterizedValue]) -> prisma_query::Result<ResultSet> {
        self.with_connection(|conn| conn.query_raw(sql, params))
    }

    fn execute_raw(&self, _: &str, sql: &str, params: &[ParameterizedValue]) -> prisma_query::Result<u64> {
        self.with_connection(|conn| conn.execute_raw(sql, params))
    }
}

pub struct Mysql {
    pool: MysqlPool,
}

impl Mysql {
    pub fn new(params: MysqlParams) -> prisma_query::Result<Self> {
        let manager = PrismaConnectionManager::mysql(params.config);

        let pool = r2d2::Pool::builder()
            .max_size(params.connection_limit)
            .test_on_check_out(false)
            .build(manager)?;

        Ok(Self { pool })
    }

    fn with_connection<F, T>(&self, f: F) -> T
    where
        F: FnOnce(&mut dyn Queryable) -> T,
    {
        let mut conn = self.pool.get().unwrap();
        f(conn.deref_mut())
    }
}

impl MigrationDatabase for Mysql {
    fn execute(&self, _: &str, q: Query) -> prisma_query::Result<Option<Id>> {
        self.with_connection(|conn| conn.execute(q))
    }

    fn query(&self, _: &str, q: Query) -> prisma_query::Result<ResultSet> {
        self.with_connection(|conn| conn.query(q))
    }

    fn query_raw(&self, _: &str, sql: &str, params: &[ParameterizedValue]) -> prisma_query::Result<ResultSet> {
        self.with_connection(|conn| conn.query_raw(sql, params))
    }

    fn execute_raw(&self, _: &str, sql: &str, params: &[ParameterizedValue]) -> prisma_query::Result<u64> {
        self.with_connection(|conn| conn.execute_raw(sql, params))
    }
}
