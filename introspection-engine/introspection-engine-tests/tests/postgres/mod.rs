use indoc::indoc;
use introspection_engine_tests::test_api::*;
use test_macros::test_each_connector;

#[test_each_connector(tags("postgres"))]
async fn sequences_should_work(api: &TestApi) -> crate::TestResult {
    api.barrel()
        .execute(move |migration| {
            migration.inject_custom("CREATE SEQUENCE \"first_Sequence\"");
            migration.inject_custom("CREATE SEQUENCE \"second_sequence\"");
            migration.inject_custom("CREATE SEQUENCE \"third_Sequence\"");

            migration.create_table("Test", move |t| {
                t.inject_custom("id Integer Primary Key");
                t.inject_custom("serial  Serial");
                t.inject_custom("first   BigInt Not Null Default nextval('\"first_Sequence\"'::regclass)");
                t.inject_custom("second  BigInt Default nextval('\"second_sequence\"')");
                t.inject_custom("third  BigInt Not Null Default nextval('third_Sequence'::text)");
            });
        })
        .await?;

    let dm = indoc! {r#"
        model Test {
          id     Int        @id
          serial Int        @default(autoincrement())
          first  BigInt     @default(autoincrement())
          second BigInt?    @default(autoincrement())
          third  BigInt     @default(autoincrement())
        }
    "#}
    .to_string();

    let result = api.re_introspect(&dm).await?;

    println!("EXPECTATION: \n {:#}", dm);
    println!("RESULT: \n {:#}", result);

    api.assert_eq_datamodels(&dm, &result);
    Ok(())
}
