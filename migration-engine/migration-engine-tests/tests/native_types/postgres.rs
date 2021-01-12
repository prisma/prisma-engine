use bigdecimal::BigDecimal;
use chrono::Utc;
use migration_engine_tests::sql::*;
use once_cell::sync::Lazy;
use quaint::{
    prelude::{Insert, Queryable},
    Value,
};
use sql_datamodel_connector::SqlDatamodelConnectors;
use std::{collections::HashMap, str::FromStr};

//how to handle aliases
// serial
// decimal

// do not castable ✓
// split castable into safe and risky ✓
// split seeds into risky succeeds ✓
// enable force in risky succeeds ✓
// adjust the differ ✓
// get this testfile to pass ✓
// cleanup
// think about removed/ignored aliases -> serial, decimal...
// get everything else to pass
// review
// merge / review without list->scalar / scalar -> list on monday
// setup separate test case for risky fails
// work on list/scalar scalar/list separately

static SAFE_CASTS: Lazy<Vec<(&str, Value, &[&str])>> = Lazy::new(|| {
    vec![
        (
            "SmallInt",
            Value::integer(u8::MAX),
            &[
                "SmallInt",
                "Integer",
                "BigInt",
                "Real",
                "DoublePrecision",
                "VarChar(53)",
                "Char(53)",
                "Text",
            ],
        ),
        (
            "Integer",
            Value::integer(i32::MAX),
            &[
                "Integer",
                "BigInt",
                "Real",
                "DoublePrecision",
                "VarChar(53)",
                "Char(53)",
                "Text",
            ],
        ),
        (
            "BigInt",
            Value::integer(i64::MAX),
            &["BigInt", "Real", "DoublePrecision", "VarChar(53)", "Char(53)", "Text"],
        ),
        (
            "Numeric(10,2)",
            Value::numeric(BigDecimal::from_str("12345678.90").unwrap()),
            &["Numeric(32,16)", "VarChar(53)", "Char(53)", "Text"],
        ),
        (
            "Numeric(2,0)",
            Value::numeric(BigDecimal::from_str("12").unwrap()),
            &[
                "SmallInt",
                "Integer",
                "BigInt",
                "Numeric(32,16)",
                "VarChar(53)",
                "Char(53)",
                "Text",
            ],
        ),
        (
            "Real",
            Value::float(f32::MIN),
            &["DoublePrecision", "VarChar(53)", "VarChar", "Char(53)", "Text"],
        ),
        (
            "DoublePrecision",
            Value::Double(Some(f64::MIN)),
            &["DoublePrecision", "Text", "VarChar", "Char(1000)"],
        ),
        ("VarChar", Value::text("fiver"), &["Text"]),
        ("VarChar(5)", Value::text("fiver"), &["VarChar(53)", "Char(53)", "Text"]),
        (
            "Char(1)", // same as Char
            Value::text("t"),
            &["VarChar(3)", "Char(3)", "Text"],
        ),
        ("Text", Value::text("true"), &["VarChar", "Text"]),
        ("ByteA", Value::bytes(b"DEAD".to_vec()), &["Text", "VarChar"]),
        (
            "Timestamp(3)",
            Value::datetime(Utc::now()),
            &[
                "VarChar(23)",
                "Char(23)",
                "Text",
                "Timestamp(1)",
                "Timestamptz(3)",
                "Date",
                "Time(3)",
            ],
        ),
        (
            "Timestamptz(3)",
            Value::datetime(Utc::now()),
            &[
                "VarChar(28)",
                "Char(53)",
                "Text",
                "Timestamp(1)",
                "Date",
                "Time(3)",
                "Timetz(3)",
            ],
        ),
        (
            "Date",
            Value::date(Utc::today().naive_utc()),
            &["VarChar(53)", "Char(28)", "Text", "Timestamp(3)", "Timestamptz(3)"],
        ),
        (
            "Time(3)",
            Value::time(Utc::now().naive_utc().time()),
            &["VarChar(14)", "Char(53)", "Text", "Timetz(3)"],
        ),
        (
            "Timetz(3)",
            Value::datetime(Utc::now()),
            &["VarChar(53)", "Char(19)", "Text", "Time(3)", "Timetz(6)"],
        ),
        ("Boolean", Value::boolean(false), &["VarChar", "Char(5)", "Text"]),
        (
            "Bit(1)", // same as Bit
            Value::text("0"),
            &["VarChar", "Char(1)", "Char(5)", "Text", "VarBit(10)"],
        ),
        (
            "Bit(10)",
            Value::text("0010101001"),
            &["VarChar(53)", "Char(53)", "Text", "VarBit(10)"],
        ),
        ("VarBit", Value::text("000101010101010010"), &["VarChar", "Text"]),
        ("VarBit(5)", Value::text("0010"), &["VarChar(53)", "Char(53)", "Text"]),
        (
            "Uuid",
            Value::text("75bf0037-a8b8-4512-beea-5a186f8abf1e"),
            &["VarChar(53)", "Char(53)", "Text"],
        ),
        ("Xml", Value::xml("[]"), &["VarChar", "Text"]),
        (
            "Json",
            Value::json(serde_json::json!({"foo": "bar"})),
            &["Text", "JsonB", "VarChar"],
        ),
        (
            "JsonB",
            Value::json(serde_json::json!({"foo": "bar"})),
            &["Text", "Json", "VarChar"],
        ),
    ]
});

static RISKY_CASTS: Lazy<Vec<(&str, Value, &[&str])>> = Lazy::new(|| {
    vec![
        (
            "SmallInt",
            Value::integer(2),
            &["Numeric(2,1)", "VarChar(3)", "Char(1)"],
        ),
        ("Integer", Value::integer(1), &["Numeric(2,1)", "VarChar(4)", "Char(1)"]),
        ("BigInt", Value::integer(2), &["Numeric(2,1)", "VarChar(17)", "Char(1)"]),
        (
            "Numeric(10,2)",
            Value::numeric(BigDecimal::from_str("1").unwrap()),
            &[
                "SmallInt",
                "Integer",
                "BigInt",
                "Real",
                "DoublePrecision",
                "VarChar(5)",
                "Char(5)",
            ],
        ),
        (
            "Numeric(5,0)",
            Value::numeric(BigDecimal::from_str("10").unwrap()),
            &["SmallInt", "VarChar(5)", "Char(5)", "Real", "DoublePrecision"],
        ),
        (
            "Real",
            Value::float(3 as f32),
            &[
                "SmallInt",
                "Integer",
                "BigInt",
                "Numeric(32,16)",
                "VarChar(10)",
                "Char(1)",
            ],
        ),
        (
            "DoublePrecision",
            Value::double(3 as f64),
            &[
                "SmallInt",
                "Integer",
                "BigInt",
                "Numeric(32,16)",
                "Real",
                "VarChar(5)",
                "Char(1)",
            ],
        ),
        ("VarChar(5)", Value::text("t"), &["VarChar(3)", "Char(1)"]),
        ("Text", Value::text("t"), &["VarChar(3)", "Char(1)"]),
        ("ByteA", Value::bytes(vec![1]), &["VarChar(4)", "Char(5)"]),
        ("VarBit(5)", Value::text("001"), &["Bit(3)"]),
        ("Xml", Value::xml("[]"), &["VarChar(100)", "Char(100)"]),
        (
            "Json",
            Value::json(serde_json::json!({"foo": "bar"})),
            &["VarChar(100)", "Char(100)"],
        ),
        (
            "JsonB",
            Value::json(serde_json::json!({"foo": "bar"})),
            &["VarChar(100)", "Char(100)"],
        ),
    ]
});

static NOT_CASTABLE: Lazy<Vec<(&str, Value, &[&str])>> = Lazy::new(|| {
    vec![
        (
            "SmallInt",
            Value::integer(u8::MAX),
            &[
                "ByteA",
                "Timestamp(3)",
                "Timestamptz(3)",
                "Date",
                "Time(3)",
                "Timetz(3)",
                "Boolean",
                "Bit(10)",
                "VarBit(10)",
                "Uuid",
                "Xml",
                "Json",
                "JsonB",
            ],
        ),
        (
            "Integer",
            Value::integer(i32::MAX),
            &[
                "ByteA",
                "Timestamp(3)",
                "Timestamptz(3)",
                "Date",
                "Time(3)",
                "Timetz(3)",
                "Boolean",
                "Bit(10)",
                "VarBit(10)",
                "Uuid",
                "Xml",
                "Json",
                "JsonB",
            ],
        ),
        (
            "BigInt",
            Value::integer(i64::MAX),
            &[
                "ByteA",
                "Timestamp(3)",
                "Timestamptz(3)",
                "Date",
                "Time(3)",
                "Timetz(3)",
                "Boolean",
                "Bit(10)",
                "VarBit(10)",
                "Uuid",
                "Xml",
                "Json",
                "JsonB",
            ],
        ),
        (
            "Numeric(10,2)",
            Value::numeric(BigDecimal::from_str("1").unwrap()),
            &[
                "ByteA",
                "Timestamp(3)",
                "Timestamptz(3)",
                "Date",
                "Time(3)",
                "Timetz(3)",
                "Boolean",
                "Bit(10)",
                "VarBit(10)",
                "Uuid",
                "Xml",
                "Json",
                "JsonB",
            ],
        ),
        (
            "Numeric(5,0)",
            Value::numeric(BigDecimal::from_str("1").unwrap()),
            &[
                "ByteA",
                "Timestamp(3)",
                "Timestamptz(3)",
                "Date",
                "Time(3)",
                "Timetz(3)",
                "Boolean",
                "Bit(10)",
                "VarBit(10)",
                "Uuid",
                "Xml",
                "Json",
                "JsonB",
            ],
        ),
        (
            "Real",
            Value::float(5.3),
            &[
                "Timestamp(3)",
                "Timestamptz(3)",
                "Date",
                "Time(3)",
                "Timetz(3)",
                "Boolean",
                "Bit(10)",
                "VarBit(10)",
                "Uuid",
                "Xml",
                "Json",
                "JsonB",
            ],
        ),
        (
            "DoublePrecision",
            Value::double(7.5),
            &[
                "ByteA",
                "Timestamp(3)",
                "Timestamptz(3)",
                "Date",
                "Time(3)",
                "Timetz(3)",
                "Boolean",
                "Bit(10)",
                "VarBit(10)",
                "Uuid",
                "Xml",
                "Json",
                "JsonB",
            ],
        ),
        (
            "VarChar(5)",
            Value::text("true"),
            &[
                "SmallInt",
                "Integer",
                "BigInt",
                "Numeric(32,16)",
                "Real",
                "DoublePrecision",
                "ByteA",
                "Timestamp(3)",
                "Timestamptz(3)",
                "Date",
                "Time(3)",
                "Timetz(3)",
                "Boolean",
                "Bit(10)",
                "VarBit(10)",
                "Uuid",
                "Xml",
                "Json",
                "JsonB",
            ],
        ),
        (
            "Char(5)",
            Value::text("true"),
            &[
                "SmallInt",
                "Integer",
                "BigInt",
                "Numeric(32,16)",
                "Real",
                "DoublePrecision",
                "ByteA",
                "Timestamp(3)",
                "Timestamptz(3)",
                "Date",
                "Time(3)",
                "Timetz(3)",
                "Boolean",
                "Bit(10)",
                "VarBit(10)",
                "Uuid",
                "Xml",
                "Json",
                "JsonB",
            ],
        ),
        (
            "Text",
            Value::text("true"),
            &[
                "SmallInt",
                "Integer",
                "BigInt",
                "Numeric(32,16)",
                "Real",
                "DoublePrecision",
                "ByteA",
                "Timestamp(3)",
                "Timestamptz(3)",
                "Date",
                "Time(3)",
                "Timetz(3)",
                "Boolean",
                "Bit(10)",
                "VarBit(10)",
                "Uuid",
                "Xml",
                "Json",
                "JsonB",
            ],
        ),
        (
            "ByteA",
            Value::bytes(vec![1]),
            &[
                "SmallInt",
                "Integer",
                "BigInt",
                "Numeric(32,16)",
                "Real",
                "DoublePrecision",
                "Timestamp(3)",
                "Timestamptz(3)",
                "Date",
                "Time(3)",
                "Timetz(3)",
                "Boolean",
                "Bit(10)",
                "VarBit(10)",
                "Uuid",
                "Xml",
                "Json",
                "JsonB",
            ],
        ),
        (
            "Timestamp(3)",
            Value::datetime(Utc::now()),
            &[
                "SmallInt",
                "Integer",
                "BigInt",
                "Numeric(32,16)",
                "Real",
                "DoublePrecision",
                "ByteA",
                "Boolean",
                "Bit(10)",
                "VarBit(10)",
                "Uuid",
                "Xml",
                "Json",
                "JsonB",
            ],
        ),
        (
            "Timestamptz(3)",
            Value::datetime(Utc::now()),
            &[
                "SmallInt",
                "Integer",
                "BigInt",
                "Numeric(32,16)",
                "Real",
                "DoublePrecision",
                "ByteA",
                "Boolean",
                "Bit(10)",
                "VarBit(10)",
                "Uuid",
                "Xml",
                "Json",
                "JsonB",
            ],
        ),
        (
            "Date",
            Value::date(Utc::today().naive_utc()),
            &[
                "SmallInt",
                "Integer",
                "BigInt",
                "Numeric(32,16)",
                "Real",
                "DoublePrecision",
                "ByteA",
                "Time(3)",
                "Timetz(3)",
                "Boolean",
                "Bit(10)",
                "VarBit(10)",
                "Uuid",
                "Xml",
                "Json",
                "JsonB",
            ],
        ),
        (
            "Time(3)",
            Value::time(Utc::now().naive_utc().time()),
            &[
                "SmallInt",
                "Integer",
                "BigInt",
                "Numeric(32,16)",
                "Real",
                "DoublePrecision",
                "ByteA",
                "Timestamp(3)",
                "Timestamptz(3)",
                "Date",
                "Boolean",
                "Bit(10)",
                "VarBit(10)",
                "Uuid",
                "Xml",
                "Json",
                "JsonB",
            ],
        ),
        (
            "Timetz(3)",
            Value::datetime(Utc::now()),
            &[
                "SmallInt",
                "Integer",
                "BigInt",
                "Numeric(32,16)",
                "Real",
                "DoublePrecision",
                "ByteA",
                "Timestamp(3)",
                "Timestamptz(3)",
                "Date",
                "Boolean",
                "Bit(10)",
                "VarBit(10)",
                "Uuid",
                "Xml",
                "Json",
                "JsonB",
            ],
        ),
        (
            "Boolean",
            Value::boolean(true),
            &[
                "SmallInt",
                "Integer",
                "BigInt",
                "Numeric(32,16)",
                "Real",
                "DoublePrecision",
                "ByteA",
                "Timestamp(3)",
                "Timestamptz(3)",
                "Date",
                "Time(3)",
                "Timetz(3)",
                "Bit(10)",
                "VarBit(10)",
                "Uuid",
                "Xml",
                "Json",
                "JsonB",
            ],
        ),
        (
            "Bit(10)",
            Value::text("0010101001"),
            &[
                "SmallInt",
                "Integer",
                "BigInt",
                "Numeric(32,16)",
                "Real",
                "DoublePrecision",
                "ByteA",
                "Timestamp(3)",
                "Timestamptz(3)",
                "Date",
                "Time(3)",
                "Timetz(3)",
                "Boolean",
                "Uuid",
                "Xml",
                "Json",
                "JsonB",
            ],
        ),
        (
            "VarBit(5)",
            Value::text("0010"),
            &[
                "SmallInt",
                "Integer",
                "BigInt",
                "Numeric(32,16)",
                "Real",
                "DoublePrecision",
                "ByteA",
                "Timestamp(3)",
                "Timestamptz(3)",
                "Date",
                "Time(3)",
                "Timetz(3)",
                "Boolean",
                "Bit(10)",
                "Uuid",
                "Xml",
                "Json",
                "JsonB",
            ],
        ),
        (
            "Uuid",
            Value::text("75bf0037-a8b8-4512-beea-5a186f8abf1e"),
            &[
                "SmallInt",
                "Integer",
                "BigInt",
                "Numeric(32,16)",
                "Real",
                "DoublePrecision",
                "ByteA",
                "Timestamp(3)",
                "Timestamptz(3)",
                "Date",
                "Time(3)",
                "Timetz(3)",
                "Boolean",
                "Bit(10)",
                "VarBit(10)",
                "Xml",
                "Json",
                "JsonB",
            ],
        ),
        (
            "Xml",
            Value::text("[]"),
            &[
                "SmallInt",
                "Integer",
                "BigInt",
                "Numeric(32,16)",
                "Real",
                "DoublePrecision",
                "ByteA",
                "Timestamp(3)",
                "Timestamptz(3)",
                "Date",
                "Time(3)",
                "Timetz(3)",
                "Boolean",
                "Bit(10)",
                "VarBit(10)",
                "Uuid",
                "Json",
                "JsonB",
            ],
        ),
        (
            "Json",
            Value::json(serde_json::json!({"foo": "bar"})),
            &[
                "SmallInt",
                "Integer",
                "BigInt",
                "Numeric(32,16)",
                "Real",
                "DoublePrecision",
                "ByteA",
                "Timestamp(3)",
                "Timestamptz(3)",
                "Date",
                "Time(3)",
                "Timetz(3)",
                "Boolean",
                "Bit(10)",
                "VarBit(10)",
                "Uuid",
                "Xml",
            ],
        ),
        (
            "JsonB",
            Value::json(serde_json::json!({"foo": "bar"})),
            &[
                "SmallInt",
                "Integer",
                "BigInt",
                "Numeric(32,16)",
                "Real",
                "DoublePrecision",
                "ByteA",
                "Timestamp(3)",
                "Timestamptz(3)",
                "Date",
                "Time(3)",
                "Timetz(3)",
                "Boolean",
                "Bit(10)",
                "VarBit(10)",
                "Uuid",
                "Xml",
            ],
        ),
    ]
});

static TYPE_MAPS: Lazy<HashMap<&'static str, &'static str>> = Lazy::new(|| {
    let mut maps = HashMap::new();

    maps.insert("SmallInt", "Int");
    maps.insert("Integer", "Int");
    maps.insert("BigInt", "BigInt");
    maps.insert("Numeric", "Decimal");
    maps.insert("Real", "Float");
    maps.insert("DoublePrecision", "Float");
    maps.insert("VarChar", "String");
    maps.insert("Char", "String");
    maps.insert("Text", "String");
    maps.insert("ByteA", "Bytes");
    maps.insert("Timestamp", "DateTime");
    maps.insert("Timestamptz", "DateTime");
    maps.insert("Date", "DateTime");
    maps.insert("Time", "DateTime");
    maps.insert("Timetz", "DateTime");
    maps.insert("Boolean", "Boolean");
    maps.insert("Bit", "String");
    maps.insert("VarBit", "String");
    maps.insert("Uuid", "String");
    maps.insert("Xml", "String");
    maps.insert("Json", "Json");
    maps.insert("JsonB", "Json");

    maps
});

fn prisma_type(native_type: &str) -> &str {
    let kind = native_type.split("(").next().unwrap();
    TYPE_MAPS.get(kind).unwrap()
}

#[test_each_connector(tags("postgres"), features("native_types"))]
async fn safe_casts_with_existing_data_should_work(api: &TestApi) -> TestResult {
    let connector = SqlDatamodelConnectors::postgres();

    for (from, seed, casts) in SAFE_CASTS.iter() {
        let mut previous_columns = "".to_string();
        let mut next_columns = "".to_string();
        let mut insert = Insert::single_into((api.schema_name(), "A"));
        let mut previous_assertions = vec![];
        let mut next_assertions = vec![];

        for (idx, to) in casts.iter().enumerate() {
            println!("From `{}` to `{}` with seed `{:?}`", from, to, seed);

            let column_name = format!("column_{}", idx);

            previous_columns.push_str(&format!(
                "{column_name}  {prisma_type}? @test_db.{native_type} \n",
                prisma_type = prisma_type(from),
                native_type = from,
                column_name = column_name
            ));

            next_columns.push_str(&format!(
                "{column_name}  {prisma_type}? @test_db.{native_type}\n",
                prisma_type = prisma_type(to),
                native_type = to,
                column_name = column_name
            ));

            insert = insert.value(column_name.clone(), seed.clone());

            previous_assertions.push((column_name.clone(), from.clone()));
            next_assertions.push((column_name, to).clone());
        }

        let dm1 = api.native_types_datamodel(format!(
            r#"
                model A {{
                    id Int @id @default(autoincrement()) @test_db.Integer
                    {columns}
                }}
                "#,
            columns = previous_columns
        ));

        api.schema_push(&dm1).send().await?.assert_green()?;

        //inserts
        api.database().insert(insert.into()).await?;

        //first assertions
        api.assert_schema().await?.assert_table("A", |table| {
            previous_assertions.iter().fold(
                table.assert_column_count(previous_assertions.len() + 1),
                |acc, (column_name, expected)| {
                    acc.and_then(|table| {
                        table.assert_column(column_name, |c| c.assert_native_type(expected, &connector))
                    })
                },
            )
        })?;

        let dm2 = api.native_types_datamodel(format!(
            r#"
                model A {{
                    id Int @id @default(autoincrement()) @test_db.Integer
                    {columns}
                }}
                "#,
            columns = next_columns
        ));

        api.schema_push(&dm2).send().await?.assert_green()?;

        //second assertions
        api.assert_schema().await?.assert_table("A", |table| {
            next_assertions.iter().fold(
                table.assert_column_count(next_assertions.len() + 1),
                |acc, (name, expected)| {
                    acc.and_then(|table| table.assert_column(name, |c| c.assert_native_type(expected, &connector)))
                },
            )
        })?;

        api.database()
            .raw_cmd(&format!("DROP TABLE \"{}\".\"A\"", api.schema_name()))
            .await?;
    }

    Ok(())
}

#[test_each_connector(tags("postgres"), features("native_types"))]
async fn risky_casts_with_existing_data_should_warn(api: &TestApi) -> TestResult {
    let connector = SqlDatamodelConnectors::postgres();

    for (from, seed, casts) in RISKY_CASTS.iter() {
        let mut previous_columns = "".to_string();
        let mut next_columns = "".to_string();
        let mut insert = Insert::single_into((api.schema_name(), "A"));
        let mut previous_assertions = vec![];
        let mut next_assertions = vec![];
        let mut warnings = vec![];

        for (idx, to) in casts.iter().enumerate() {
            println!("From `{}` to `{}` with seed `{:?}`", from, to, seed);

            let column_name = format!("column_{}", idx);

            previous_columns.push_str(&format!(
                "{column_name}  {prisma_type}? @test_db.{native_type} \n",
                prisma_type = prisma_type(from),
                native_type = from,
                column_name = column_name
            ));

            next_columns.push_str(&format!(
                "{column_name}  {prisma_type}? @test_db.{native_type}\n",
                prisma_type = prisma_type(to),
                native_type = to,
                column_name = column_name
            ));

            insert = insert.value(column_name.clone(), seed.clone());

            warnings.push( format!(
                "You are about to alter the column `{column_name}` on the `A` table, which contains 1 non-null values. The data in that column will be cast from `{from}` to `{to}`.",
               column_name = column_name,
                from = from,
                to = to,
            ).into());

            previous_assertions.push((column_name.clone(), from.clone()));
            next_assertions.push((column_name.clone(), to.clone()));
        }

        let dm1 = api.native_types_datamodel(format!(
            r#"
                model A {{
                    id Int @id @default(autoincrement()) @test_db.Integer
                    {columns}
                }}
                "#,
            columns = previous_columns
        ));

        api.schema_push(&dm1).send().await?.assert_green()?;

        //inserts
        api.database().insert(insert.into()).await?;

        //first assertions

        api.assert_schema().await?.assert_table("A", |table| {
            previous_assertions.iter().fold(
                table.assert_column_count(previous_assertions.len() + 1),
                |acc, (column_name, expected)| {
                    acc.and_then(|table| {
                        table.assert_column(column_name, |c| c.assert_native_type(expected, &connector))
                    })
                },
            )
        })?;

        let dm2 = api.native_types_datamodel(format!(
            r#"
                model A {{
                    id Int @id @default(autoincrement()) @test_db.Integer
                    {columns}
                }}
                "#,
            columns = next_columns
        ));

        api.schema_push(&dm2)
            .force(true)
            .send()
            .await?
            .assert_warnings(&warnings)?;

        //second assertions same as first
        api.assert_schema().await?.assert_table("A", |table| {
            next_assertions.iter().fold(
                table.assert_column_count(next_assertions.len() + 1),
                |acc, (column_name, expected)| {
                    acc.and_then(|table| {
                        table.assert_column(column_name, |c| c.assert_native_type(expected, &connector))
                    })
                },
            )
        })?;

        api.database()
            .raw_cmd(&format!("DROP TABLE \"{}\".\"A\"", api.schema_name()))
            .await?;
    }

    Ok(())
}

#[test_each_connector(tags("postgres"), features("native_types"))]
async fn not_castable_with_existing_data_should_warn(api: &TestApi) -> TestResult {
    let connector = SqlDatamodelConnectors::postgres();

    for (from, seed, casts) in NOT_CASTABLE.iter() {
        let mut previous_columns = "".to_string();
        let mut next_columns = "".to_string();
        let mut insert = Insert::single_into((api.schema_name(), "A"));
        let mut previous_assertions = vec![];
        let mut warnings = vec![];

        for (idx, to) in casts.iter().enumerate() {
            println!("From `{}` to `{}` with seed `{:?}`", from, to, seed);

            let column_name = format!("column_{}", idx);

            previous_columns.push_str(&format!(
                "{column_name}  {prisma_type}? @test_db.{native_type} \n",
                prisma_type = prisma_type(from),
                native_type = from,
                column_name = column_name
            ));

            next_columns.push_str(&format!(
                "{column_name}  {prisma_type}? @test_db.{native_type}\n",
                prisma_type = prisma_type(to),
                native_type = to,
                column_name = column_name
            ));

            insert = insert.value(column_name.clone(), seed.clone());

            //todo adjust to mention the to and from
            warnings.push(
                format!(
                    "The `{column_name}` column on the `A` table would be dropped and recreated. This will lead to data loss.",
                    column_name = column_name,
                    // from = from,
                    // to = to,
                )
                .into(),
            );

            previous_assertions.push((column_name, from).clone());
        }

        let dm1 = api.native_types_datamodel(format!(
            r#"
                model A {{
                    id Int @id @default(autoincrement()) @test_db.Integer
                    {columns}
                }}
                "#,
            columns = previous_columns
        ));

        api.schema_push(&dm1).send().await?.assert_green()?;

        //inserts
        api.database().insert(insert.into()).await?;

        //first assertions
        api.assert_schema().await?.assert_table("A", |table| {
            previous_assertions.iter().fold(
                table.assert_column_count(previous_assertions.len() + 1),
                |acc, (column_name, expected)| {
                    acc.and_then(|table| {
                        table.assert_column(column_name, |c| c.assert_native_type(expected, &connector))
                    })
                },
            )
        })?;

        let dm2 = api.native_types_datamodel(format!(
            r#"
                model A {{
                    id Int @id @default(autoincrement()) @test_db.Integer
                    {columns}
                }}
                "#,
            columns = next_columns
        ));

        // todo we could force here and then check that the db really returns not castable
        // then we would again need to have separate calls per mapping
        api.schema_push(&dm2).send().await?.assert_warnings(&warnings)?;

        //second assertions same as first
        api.assert_schema().await?.assert_table("A", |table| {
            previous_assertions.iter().fold(
                table.assert_column_count(previous_assertions.len() + 1),
                |acc, (column_name, expected)| {
                    acc.and_then(|table| {
                        table.assert_column(column_name, |c| c.assert_native_type(expected, &connector))
                    })
                },
            )
        })?;

        api.database()
            .raw_cmd(&format!("DROP TABLE \"{}\".\"A\"", api.schema_name()))
            .await?;
    }

    Ok(())
}

// lists

#[test_each_connector(tags("postgres"), features("native_types"))]
async fn safe_casts_with_existing_data_should_work(api: &TestApi) -> TestResult {
    let connector = SqlDatamodelConnectors::postgres();

    for (from, seed, casts) in SAFE_CASTS.iter() {
        let mut previous_columns = "".to_string();
        let mut next_columns = "".to_string();
        let mut insert = Insert::single_into((api.schema_name(), "A"));
        let mut previous_assertions = vec![];
        let mut next_assertions = vec![];

        for (idx, to) in casts.iter().enumerate() {
            println!("From `{}` to `{}` with seed `{:?}`", from, to, seed);

            let column_name = format!("column_{}", idx);

            previous_columns.push_str(&format!(
                "{column_name}  {prisma_type}? @test_db.{native_type} \n",
                prisma_type = prisma_type(from),
                native_type = from,
                column_name = column_name
            ));

            next_columns.push_str(&format!(
                "{column_name}  {prisma_type}? @test_db.{native_type}\n",
                prisma_type = prisma_type(to),
                native_type = to,
                column_name = column_name
            ));

            insert = insert.value(column_name.clone(), seed.clone());

            previous_assertions.push((column_name.clone(), from.clone()));
            next_assertions.push((column_name, to).clone());
        }

        let dm1 = api.native_types_datamodel(format!(
            r#"
                model A {{
                    id Int @id @default(autoincrement()) @test_db.Integer
                    {columns}
                }}
                "#,
            columns = previous_columns
        ));

        api.schema_push(&dm1).send().await?.assert_green()?;

        //inserts
        api.database().insert(insert.into()).await?;

        //first assertions
        api.assert_schema().await?.assert_table("A", |table| {
            previous_assertions.iter().fold(
                table.assert_column_count(previous_assertions.len() + 1),
                |acc, (column_name, expected)| {
                    acc.and_then(|table| {
                        table.assert_column(column_name, |c| c.assert_native_type(expected, &connector))
                    })
                },
            )
        })?;

        let dm2 = api.native_types_datamodel(format!(
            r#"
                model A {{
                    id Int @id @default(autoincrement()) @test_db.Integer
                    {columns}
                }}
                "#,
            columns = next_columns
        ));

        api.schema_push(&dm2).send().await?.assert_green()?;

        //second assertions
        api.assert_schema().await?.assert_table("A", |table| {
            next_assertions.iter().fold(
                table.assert_column_count(next_assertions.len() + 1),
                |acc, (name, expected)| {
                    acc.and_then(|table| table.assert_column(name, |c| c.assert_native_type(expected, &connector)))
                },
            )
        })?;

        api.database()
            .raw_cmd(&format!("DROP TABLE \"{}\".\"A\"", api.schema_name()))
            .await?;
    }

    Ok(())
}
