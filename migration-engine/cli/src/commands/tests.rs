use super::CliError;
use quaint::{prelude::*, single::Quaint};
use structopt::StructOpt;

async fn run(args: &[&str]) -> Result<String, CliError> {
    let cli = super::Cli::from_iter(std::iter::once(&"migration-engine-cli-test").chain(args.iter()));
    cli.run_inner().await
}

fn postgres_url(db: Option<&str>) -> String {
    postgres_url_with_scheme(db, "postgresql")
}

fn postgres_url_with_scheme(db: Option<&str>, scheme: &str) -> String {
    let original_url = test_setup::postgres_10_url(db.unwrap_or("postgres"));
    let mut parsed: url::Url = original_url.parse().unwrap();
    parsed.set_scheme(scheme).unwrap();
    parsed.to_string()
}

fn mysql_url(db: Option<&str>) -> String {
    test_setup::mysql_url(db.unwrap_or(""))
}

#[tokio::test]
async fn test_connecting_with_a_working_mysql_connection_string() {
    let result = run(&["--datasource", &mysql_url(None), "can-connect-to-database"])
        .await
        .unwrap();

    assert_eq!(result, "Connection successful");
}

#[tokio::test]
async fn test_connecting_with_a_non_working_mysql_connection_string() {
    let datasource = mysql_url(Some("this_does_not_exist"));
    let err = run(&["--datasource", &datasource, "can-connect-to-database"])
        .await
        .unwrap_err();

    assert_eq!("P1003", err.error_code().unwrap());
}

#[tokio::test]
async fn test_connecting_with_a_working_psql_connection_string() {
    let datasource = postgres_url(None);
    let result = run(&["--datasource", &datasource, "can-connect-to-database"])
        .await
        .unwrap();

    assert_eq!(result, "Connection successful");
}

#[tokio::test]
async fn test_connecting_with_a_working_psql_connection_string_with_postgres_scheme() {
    let result = run(&[
        "--datasource",
        &postgres_url_with_scheme(None, "postgres"),
        "can-connect-to-database",
    ])
    .await
    .unwrap();

    assert_eq!(result, "Connection successful");
}

#[tokio::test]
async fn test_connecting_with_a_non_working_psql_connection_string() {
    let datasource = postgres_url(Some("this_does_not_exist"));
    let err = run(&["--datasource", &datasource, "can-connect-to-database"])
        .await
        .unwrap_err();

    assert_eq!("P1003", err.error_code().unwrap());
}

#[tokio::test]
async fn test_create_mysql_database() {
    let url = mysql_url(Some("this_should_exist"));

    let res = run(&["--datasource", &url, "create-database"]).await;

    assert_eq!(
        "Database 'this_should_exist' created successfully.",
        res.as_ref().unwrap()
    );

    if let Ok(_) = res {
        let res = run(&["--datasource", &url, "can-connect-to-database"]).await;
        assert_eq!("Connection successful", res.as_ref().unwrap());

        {
            let uri = mysql_url(None);
            let conn = Quaint::new(&uri).await.unwrap();

            conn.execute_raw("DROP DATABASE `this_should_exist`", &[])
                .await
                .unwrap();
        }

        res.unwrap();
    } else {
        res.unwrap();
    }
}

#[tokio::test]
async fn test_create_psql_database() {
    let db_name = "this_should_exist";

    let _drop_database: () = {
        let url = postgres_url(None);

        let conn = Quaint::new(&url).await.unwrap();

        conn.execute_raw("DROP DATABASE IF EXISTS \"this_should_exist\"", &[])
            .await
            .unwrap();
    };

    let url = postgres_url(Some(db_name));

    let res = run(&["--datasource", &url, "create-database"]).await;

    assert_eq!(
        "Database 'this_should_exist' created successfully.",
        res.as_ref().unwrap()
    );

    let res = run(&["--datasource", &url, "can-connect-to-database"]).await;
    assert_eq!("Connection successful", res.as_ref().unwrap());

    res.unwrap();
}

#[tokio::test]
async fn test_create_sqlite_database() {
    let base_dir = tempfile::tempdir().unwrap();

    let sqlite_path = base_dir
        .path()
        .join("doesntexist/either")
        .join("test_create_sqlite_database.db");

    assert!(!sqlite_path.exists());

    let url = format!("file:{}", sqlite_path.to_string_lossy());

    let res = run(&["--datasource", &url, "create-database"]).await;
    assert_eq!("", res.as_ref().unwrap());

    assert!(sqlite_path.exists());
}

#[test]
fn test_fetch_db_name() {
    let url: url::Url = "postgresql://postgres:prisma@127.0.0.1:5432/pgres?schema=test_schema"
        .parse()
        .unwrap();
    let db_name = super::fetch_db_name(&url, "postgres");
    assert_eq!(db_name, "pgres");
}

#[test]
fn test_fetch_db_name_with_postgres_scheme() {
    let url: url::Url = "postgres://postgres:prisma@127.0.0.1:5432/pgres?schema=test_schema"
        .parse()
        .unwrap();
    let db_name = super::fetch_db_name(&url, "postgres");
    assert_eq!(db_name, "pgres");
}
