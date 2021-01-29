use migration_engine_tests::sql::*;
use sql_schema_describer::ColumnTypeFamily;

#[test_each_connector]
async fn bytes_columns_are_idempotent(api: &TestApi) -> TestResult {
    let dm = format!(
        r#"
        {datasource}

        generator client {{
            provider = "prisma-client-js"
            previewFeatures = ["nativeTypes"]
        }}

        model Cat {{
            id String @id
            chipData Bytes
        }}
    "#,
        datasource = api.datasource()
    );

    api.schema_push(&dm)
        .send()
        .await?
        .assert_green()?
        .assert_has_executed_steps()?;

    api.schema_push(&dm).send().await?.assert_green()?.assert_no_steps()?;

    Ok(())
}

#[test_each_connector]
async fn float_columns_are_idempotent(api: &TestApi) -> TestResult {
    let dm = r#"
        model Cat {
            id String @id
            meowFrequency Float
        }
    "#;

    api.schema_push(dm)
        .send()
        .await?
        .assert_green()?
        .assert_has_executed_steps()?;

    api.schema_push(dm).send().await?.assert_green()?.assert_no_steps()?;

    Ok(())
}

#[test_each_connector(features("native_types"))]
async fn decimal_columns_are_idempotent(api: &TestApi) -> TestResult {
    let dm = format!(
        r#"
        {datasource}

        generator client {{
            provider = "prisma-client-js"
            previewFeatures = ["nativeTypes"]
        }}

        model Cat {{
            id String @id
            meowFrequency Decimal
        }}
    "#,
        datasource = api.datasource()
    );

    api.schema_push(&dm)
        .send()
        .await?
        .assert_green()?
        .assert_has_executed_steps()?;

    api.schema_push(&dm).send().await?.assert_green()?.assert_no_steps()?;

    Ok(())
}

#[test_each_connector(features("native_types"))]
async fn float_to_decimal_works(api: &TestApi) -> TestResult {
    let dm1 = r#"
        model Cat {
            id String @id
            meowFrequency Float
        }
    "#;

    api.schema_push(dm1).send().await?.assert_green()?;

    api.assert_schema().await?.assert_table("Cat", |table| {
        table.assert_column("meowFrequency", |col| col.assert_type_family(ColumnTypeFamily::Float))
    })?;

    let dm2 = format!(
        r#"
        {datasource}

        generator client {{
            provider = "prisma-client-js"
            previewFeatures = ["nativeTypes"]
        }}

        model Cat {{
            id String @id
            meowFrequency Decimal
        }}
    "#,
        datasource = api.datasource()
    );

    api.schema_push(&dm2)
        .send()
        .await?
        .assert_green()?
        .assert_has_executed_steps()?;

    api.assert_schema().await?.assert_table("Cat", |table| {
        table.assert_column("meowFrequency", |col| col.assert_type_family(ColumnTypeFamily::Decimal))
    })?;

    Ok(())
}

#[test_each_connector(features("native_types"), log = "debug")]
async fn decimal_to_float_works(api: &TestApi) -> TestResult {
    let dm1 = format!(
        r#"
        {datasource}

        generator client {{
            provider = "prisma-client-js"
            previewFeatures = ["nativeTypes"]
        }}

        model Cat {{
            id String @id
            meowFrequency Decimal
        }}
    "#,
        datasource = api.datasource()
    );

    api.schema_push(&dm1).send().await?.assert_green()?;

    api.assert_schema().await?.assert_table("Cat", |table| {
        table.assert_column("meowFrequency", |col| col.assert_type_family(ColumnTypeFamily::Decimal))
    })?;

    let dm2 = r#"
        model Cat {
            id String @id
            meowFrequency Float
        }
    "#;

    api.schema_push(dm2)
        .send()
        .await?
        .assert_green()?
        .assert_has_executed_steps()?;

    api.assert_schema().await?.assert_table("Cat", |table| {
        table.assert_column("meowFrequency", |col| col.assert_type_family(ColumnTypeFamily::Float))
    })?;

    Ok(())
}

#[test_each_connector]
async fn bytes_to_string_works(api: &TestApi) -> TestResult {
    let dm1 = format!(
        r#"
        {datasource}

        generator client {{
            provider = "prisma-client-js"
            previewFeatures = ["nativeTypes"]
        }}

        model Cat {{
            id String @id
            meowData Bytes
        }}
    "#,
        datasource = api.datasource()
    );

    api.schema_push(&dm1).send().await?.assert_green()?;

    api.assert_schema().await?.assert_table("Cat", |table| {
        table.assert_column("meowData", |col| col.assert_type_is_bytes())
    })?;

    let dm2 = r#"
        model Cat {
            id String @id
            meowData String
        }
    "#;

    api.schema_push(dm2)
        .send()
        .await?
        .assert_green()?
        .assert_has_executed_steps()?;

    api.assert_schema().await?.assert_table("Cat", |table| {
        table.assert_column("meowData", |col| col.assert_type_is_string())
    })?;

    Ok(())
}

#[test_each_connector]
async fn string_to_bytes_works(api: &TestApi) -> TestResult {
    let dm1 = format!(
        r#"
    {datasource}

        generator client {{
            provider = "prisma-client-js"
            previewFeatures = ["nativeTypes"]
        }}

        model Cat {{
            id String @id
            meowData Bytes
        }}
    "#,
        datasource = api.datasource()
    );

    api.schema_push(&dm1).send().await?.assert_green()?;

    api.assert_schema().await?.assert_table("Cat", |table| {
        table.assert_column("meowData", |col| col.assert_type_is_bytes())
    })?;

    let dm2 = r#"
        model Cat {
            id String @id
            meowData String
        }
    "#;

    api.schema_push(dm2)
        .send()
        .await?
        .assert_green()?
        .assert_has_executed_steps()?;

    api.assert_schema().await?.assert_table("Cat", |table| {
        table.assert_column("meowData", |col| col.assert_type_is_string())
    })?;

    Ok(())
}

#[test_each_connector(capabilities("scalar_lists"))]
async fn decimal_to_decimal_array_works(api: &TestApi) -> TestResult {
    let dm1 = format!(
        r#"
            {datasource}

            generator client {{
            provider = "prisma-client-js"
            previewFeatures = ["nativeTypes"]
        }}

            model Test {{
                id       String    @id @default(cuid())
                decFloat Decimal
            }}
        "#,
        datasource = api.datasource()
    );

    api.schema_push(&dm1).send().await?.assert_green()?;

    api.assert_schema().await?.assert_table("Test", |table| {
        table.assert_column("decFloat", |col| col.assert_type_is_decimal()?.assert_is_required())
    })?;

    let dm2 = format!(
        r#"
            {datasource}

            generator client {{
            provider = "prisma-client-js"
            previewFeatures = ["nativeTypes"]
        }}

            model Test {{
                id       String    @id @default(cuid())
                decFloat Decimal[]
            }}
        "#,
        datasource = api.datasource()
    );

    api.schema_push(dm2)
        .send()
        .await?
        .assert_green()?
        .assert_has_executed_steps()?;

    api.assert_schema().await?.assert_table("Test", |table| {
        table.assert_column("decFloat", |col| col.assert_type_is_decimal()?.assert_is_list())
    })?;

    api.schema_push(&dm1).send().await?.assert_green()?;

    api.assert_schema().await?.assert_table("Test", |table| {
        table.assert_column("decFloat", |col| col.assert_type_is_decimal()?.assert_is_required())
    })?;

    Ok(())
}

#[test_each_connector(capabilities("scalar_lists"))]
async fn bytes_to_bytes_array_works(api: &TestApi) -> TestResult {
    let dm1 = format!(
        r#"
            {datasource}

            generator client {{
              provider = "prisma-client-js"
              previewFeatures = ["nativeTypes"]
            }}

            model Test {{
                id       String    @id @default(cuid())
                bytesCol Bytes
            }}
        "#,
        datasource = api.datasource()
    );

    api.schema_push(&dm1).send().await?.assert_green()?;

    api.assert_schema().await?.assert_table("Test", |table| {
        table.assert_column("bytesCol", |col| col.assert_type_is_bytes()?.assert_is_required())
    })?;

    let dm2 = format!(
        r#"
            {datasource}

        generator client {{
            provider = "prisma-client-js"
            previewFeatures = ["nativeTypes"]
        }}

            model Test {{
                id       String    @id @default(cuid())
                bytesCol Bytes[]
            }}
        "#,
        datasource = api.datasource()
    );

    api.schema_push(dm2)
        .send()
        .await?
        .assert_green()?
        .assert_has_executed_steps()?;

    api.assert_schema().await?.assert_table("Test", |table| {
        table.assert_column("bytesCol", |col| col.assert_type_is_bytes()?.assert_is_list())
    })?;

    api.schema_push(&dm1).send().await?.assert_green()?;

    api.assert_schema().await?.assert_table("Test", |table| {
        table.assert_column("bytesCol", |col| col.assert_type_is_bytes()?.assert_is_required())
    })?;

    Ok(())
}
