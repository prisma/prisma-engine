use crate::{test_harness::*, BarrelMigrationExecutor};
use barrel::types;
use pretty_assertions::assert_eq;

//Fixme
// maybe slap db family in there as well? Have it as structured json?
// what about postgres versions??

#[test_each_connector(tags("mysql_5_6"))]
async fn database_version_for_mysql_5_6_should_work(api: &TestApi) {
    let barrel = api.barrel();
    setup(&barrel, api.db_name()).await;
    let result = dbg!(api.get_database_version().await);
    assert_eq!(result, "\"5.6.49\"".to_string());
}

#[test_each_connector(tags("mysql_8"))]
async fn database_version_for_mysql_5_7_should_work(api: &TestApi) {
    let barrel = api.barrel();
    setup(&barrel, api.db_name()).await;
    let result = dbg!(api.get_database_version().await);
    assert_eq!(result, "\"8.0.21\"".to_string());
}

#[test_each_connector(tags("mariadb"))]
async fn database_version_for_mariadb_should_work(api: &TestApi) {
    let barrel = api.barrel();
    setup(&barrel, api.db_name()).await;
    let result = dbg!(api.get_database_version().await);
    assert_eq!(result, "\"10.5.5-MariaDB-1:10.5.5+maria~focal\"".to_string());
}

#[test_each_connector(tags("postgres"))]
async fn database_version_for_postgres_should_work(api: &TestApi) {
    let barrel = api.barrel();
    setup(&barrel, api.db_name()).await;
    let result = dbg!(api.get_database_version().await);
    assert_eq!(result, "\"10.5.5-MariaDB-1:10.5.5+maria~focal\"".to_string());
}

#[test_each_connector(tags("sqlite"))]
async fn database_version_for_sqlite_should_work(api: &TestApi) {
    let barrel = api.barrel();
    setup(&barrel, api.db_name()).await;
    let result = dbg!(api.get_database_version().await);
    assert_eq!(result, "\"10.5.5-MariaDB-1:10.5.5+maria~focal\"".to_string());
}

// #[test_each_connector(tags("mssql_2019"))]
// async fn database_version_for_mssql_2019_should_work(api: &TestApi) {
//     let barrel = api.barrel();
//     setup(&barrel, api.db_name()).await;
//     let result = dbg!(api.get_database_version().await);
//     assert_eq!(result, "".to_string());
// }

async fn setup(barrel: &BarrelMigrationExecutor, db_name: &str) {
    barrel
        .execute_with_schema(
            |migration| {
                migration.create_table("Blog", |t| {
                    t.add_column("id", types::primary());
                });
            },
            db_name,
        )
        .await;
}
