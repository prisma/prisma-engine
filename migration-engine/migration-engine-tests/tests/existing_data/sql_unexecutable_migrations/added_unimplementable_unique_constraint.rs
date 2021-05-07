use migration_engine_tests::sql::*;

#[test_connector]
async fn adding_a_unique_constraint_should_warn(api: &TestApi) -> TestResult {
    let dm1 = r#"
        model Test {
            id String @id
            name String
        }
    "#;

    api.schema_push(dm1).send().await?.assert_green()?;

    {
        api.insert("Test")
            .value("id", "abc")
            .value("name", "george")
            .result_raw()
            .await?;

        api.insert("Test")
            .value("id", "def")
            .value("name", "george")
            .result_raw()
            .await?;
    }

    let dm2 = r#"
        model Test {
            id String @id
            name String @unique
        }
    "#;

    api.schema_push(dm2)
        .force(false)
        .send()
        .await?
        .assert_warnings(&["A unique constraint covering the columns `[name]` on the table `Test` will be added. If there are existing duplicate values, this will fail.".into()])?;

    let rows = api.select("Test").column("id").column("name").send_debug().await?;

    assert_eq!(
        rows,
        &[
            &[r#"Text(Some("abc"))"#, r#"Text(Some("george"))"#],
            &[r#"Text(Some("def"))"#, r#"Text(Some("george"))"#]
        ]
    );

    Ok(())
}

#[test_connector(tags(Mysql, Postgres))]
async fn dropping_enum_values_should_warn(api: &TestApi) -> TestResult {
    let dm1 = r#"
        model Test {
            id String @id
            name Test_name
        }

        enum Test_name{
            george
            paul
            ringo
            john
        }
    "#;

    api.schema_push(dm1).send().await?.assert_green()?;

    {
        api.insert("Test")
            .value("id", "abc")
            .value("name", "george")
            .result_raw()
            .await?;

        api.insert("Test")
            .value("id", "def")
            .value("name", "george")
            .result_raw()
            .await?;
    }

    let dm2 = r#"
             model Test {
            id String @id
            name Test_name
        }

        enum Test_name{
            paul
            ringo
            john
        }
    "#;

    api.schema_push(dm2)
        .force(false)
        .send()
        .await?
        .assert_warnings(&["The values [george] on the enum `Test_name` will be removed. If these variants are still used in the database, this will fail.".into()])?;

    let rows = api.select("Test").column("id").column("name").send_debug().await?;

    if api.is_mysql() {
        assert_eq!(
            rows,
            &[
                &[r#"Text(Some("abc"))"#, r#"Text(Some("george"))"#],
                &[r#"Text(Some("def"))"#, r#"Text(Some("george"))"#]
            ]
        );
    } else {
        assert_eq!(
            rows,
            &[
                &[r#"Text(Some("abc"))"#, r#"Enum(Some("george"))"#],
                &[r#"Text(Some("def"))"#, r#"Enum(Some("george"))"#]
            ]
        );
    }

    Ok(())
}

#[test_connector]
async fn adding_a_unique_constraint_when_existing_data_respects_it_works(api: &TestApi) -> TestResult {
    let dm1 = r#"
        model Test {
            id String @id
            name String
        }
    "#;

    api.schema_push(dm1).send().await?.assert_green()?;

    api.insert("Test")
        .value("id", "abc")
        .value("name", "george")
        .result_raw()
        .await?;

    api.insert("Test")
        .value("id", "def")
        .value("name", "georgina")
        .result_raw()
        .await?;

    let dm2 = r#"
        model Test {
            id String @id
            name String @unique
        }
    "#;

    api.schema_push(dm2)
        .force(true)
        .send()
        .await?
        .assert_warnings(&["A unique constraint covering the columns `[name]` on the table `Test` will be added. If there are existing duplicate values, this will fail.".into()])?;

    let rows = api.select("Test").column("id").column("name").send_debug().await?;
    assert_eq!(
        rows,
        &[
            &[r#"Text(Some("abc"))"#, r#"Text(Some("george"))"#],
            &[r#"Text(Some("def"))"#, r#"Text(Some("georgina"))"#]
        ]
    );

    Ok(())
}
