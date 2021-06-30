use migration_engine_tests::sync_test_api::*;
use sql_schema_describer::{ColumnTypeFamily, DefaultValue};

#[test_connector]
fn adding_an_id_field_of_type_int_with_autoincrement_works(api: TestApi) {
    let dm2 = r#"
        model Test {
            myId Int @id @default(autoincrement())
            text String
        }
    "#;

    api.schema_push(dm2).send_sync().assert_green_bang();
    api.assert_schema().assert_table("Test", |t| {
        t.assert_column("myId", |c| {
            if api.is_postgres() {
                c.assert_default(Some(DefaultValue::sequence("Test_myId_seq")))
            } else {
                c.assert_auto_increments()
            }
        })
    });
}

#[test_connector]
fn adding_multiple_optional_fields_to_an_existing_model_works(api: TestApi) {
    let dm1 = r#"
        model Cat {
            id Int @id
        }
    "#;

    api.schema_push(dm1).send_sync().assert_green_bang();

    let dm2 = r#"
        model Cat {
            id   Int @id
            name String?
            age  Int?
        }
    "#;

    api.schema_push(dm2).send_sync().assert_green_bang();

    api.assert_schema().assert_table("Cat", |table| {
        table
            .assert_column("name", |col| col.assert_is_nullable())
            .assert_column("age", |col| col.assert_is_nullable())
    });
}

#[test_connector]
fn a_model_can_be_removed(api: TestApi) {
    let directory = api.create_migrations_directory();

    let dm1 = r#"
        model User {
            id   Int     @id @default(autoincrement())
            name String?
            Post Post[]
        }

        model Post {
            id     Int    @id @default(autoincrement())
            title  String
            User   User   @relation(fields: [userId], references: [id])
            userId Int
        }
    "#;

    api.create_migration("initial", dm1, &directory).send_sync();

    let dm2 = r#"
        model User {
            id   Int     @id @default(autoincrement())
            name String?
        }
    "#;

    api.create_migration("second-migration", dm2, &directory).send_sync();

    api.apply_migrations(&directory)
        .send_sync()
        .assert_applied_migrations(&["initial", "second-migration"]);

    let output = api.diagnose_migration_history(&directory).send_sync().into_output();

    assert!(output.is_empty());
}

#[test_connector]
fn adding_a_scalar_field_must_work(api: TestApi) {
    let dm = format!(
        r#"
        {}

        model Test {{
            id          String @id @default(cuid())
            int         Int
            bigInt      BigInt
            float       Float
            boolean     Boolean
            string      String
            dateTime    DateTime
            decimal     Decimal
            bytes       Bytes
        }}
    "#,
        api.datasource_block(),
    );

    api.schema_push(&dm).send_sync().assert_green_bang();

    api.assert_schema().assert_table("Test", |table| {
        table
            .assert_columns_count(9)
            .assert_column("int", |c| {
                c.assert_is_required().assert_type_family(ColumnTypeFamily::Int)
            })
            .assert_column("bigInt", |c| {
                c.assert_is_required().assert_type_family(ColumnTypeFamily::BigInt)
            })
            .assert_column("float", |c| {
                c.assert_is_required().assert_type_family(ColumnTypeFamily::Float)
            })
            .assert_column("boolean", |c| {
                c.assert_is_required().assert_type_family(ColumnTypeFamily::Boolean)
            })
            .assert_column("string", |c| {
                c.assert_is_required().assert_type_family(ColumnTypeFamily::String)
            })
            .assert_column("dateTime", |c| {
                c.assert_is_required().assert_type_family(ColumnTypeFamily::DateTime)
            })
            .assert_column("decimal", |c| {
                c.assert_is_required().assert_type_family(ColumnTypeFamily::Decimal)
            })
            .assert_column("bytes", |c| {
                c.assert_is_required().assert_type_family(ColumnTypeFamily::Binary)
            })
    });

    // Check that the migration is idempotent.
    api.schema_push(dm).send_sync().assert_green_bang().assert_no_steps();
}

#[test_connector]
fn adding_an_optional_field_must_work(api: TestApi) {
    let dm2 = r#"
        model Test {
            id String @id @default(cuid())
            field String?
        }
    "#;

    api.schema_push(dm2).send_sync().assert_green_bang();
    api.assert_schema().assert_table("Test", |table| {
        table.assert_column("field", |column| column.assert_default(None).assert_is_nullable())
    });
}

#[test_connector]
fn adding_an_id_field_with_a_special_name_must_work(api: TestApi) {
    let dm2 = r#"
        model Test {
            specialName String @id @default(cuid())
        }
    "#;

    api.schema_push(dm2).send_sync().assert_green_bang();
    api.assert_schema()
        .assert_table("Test", |table| table.assert_has_column("specialName"));
}

#[test_connector(exclude(Sqlite))]
fn adding_an_id_field_of_type_int_must_work(api: TestApi) {
    let dm2 = r#"
        model Test {
            myId Int @id
            text String
        }
    "#;

    api.schema_push(dm2).send_sync().assert_green_bang();
    api.assert_schema()
        .assert_table("Test", |t| t.assert_column("myId", |c| c.assert_no_auto_increment()));
}

#[test_connector(tags(Sqlite))]
fn adding_an_id_field_of_type_int_must_work_for_sqlite(api: TestApi) {
    let dm2 = r#"
        model Test {
            myId Int @id
            text String
        }
    "#;

    api.schema_push(dm2).send_sync().assert_green_bang();

    api.assert_schema().assert_table("Test", |table| {
        table.assert_column("myId", |col| col.assert_auto_increments())
    });
}

#[test_connector]
fn removing_a_scalar_field_must_work(api: TestApi) {
    let dm1 = r#"
        model Test {
            id String @id @default(cuid())
            field String
        }
    "#;

    api.schema_push(dm1).send_sync().assert_green_bang();

    api.assert_schema()
        .assert_table("Test", |table| table.assert_columns_count(2).assert_has_column("field"));

    let dm2 = r#"
        model Test {
            id String @id @default(cuid())
        }
    "#;

    api.schema_push(dm2).send_sync().assert_green_bang();

    api.assert_schema()
        .assert_table("Test", |table| table.assert_column_count(1));
}

#[test_connector]
fn update_type_of_scalar_field_must_work(api: TestApi) {
    let dm1 = r#"
        model Test {
            id String @id @default(cuid())
            field String
        }
    "#;

    api.schema_push(dm1).send_sync().assert_green_bang();

    api.assert_schema().assert_table("Test", |table| {
        table.assert_column("field", |column| column.assert_type_is_string())
    });

    let dm2 = r#"
        model Test {
            id String @id @default(cuid())
            field Int
        }
    "#;

    api.schema_push(dm2).send_sync().assert_green_bang();

    api.assert_schema().assert_table("Test", |table| {
        table.assert_column("field", |column| column.assert_type_is_int())
    });
}

#[test_connector]
fn updating_db_name_of_a_scalar_field_must_work(api: TestApi) {
    let dm1 = r#"
        model A {
            id String @id @default(cuid())
            field String @map(name:"name1")
        }
    "#;

    api.schema_push(dm1).send_sync().assert_green_bang();
    api.assert_schema()
        .assert_table("A", |table| table.assert_has_column("name1"));

    let dm2 = r#"
        model A {
            id String @id @default(cuid())
            field String @map(name:"name2")
        }
    "#;

    api.schema_push(dm2).send_sync().assert_green_bang();
    api.assert_schema().assert_table("A", |t| {
        t.assert_columns_count(2)
            .assert_has_column("id")
            .assert_has_column("name2")
    });
}
