use barrel::{functions, types};
use indoc::indoc;
use introspection_engine_tests::{assert_eq_datamodels, test_api::*};
use quaint::prelude::Queryable;
use test_macros::test_each_connector;

#[test_each_connector]
async fn a_simple_table_with_gql_types(api: &TestApi) -> crate::TestResult {
    api.barrel()
        .execute_with_schema(
            |migration| {
                migration.create_table("Blog", move |t| {
                    t.add_column("bool", types::boolean());
                    t.add_column("float", types::float());
                    t.add_column("date", types::datetime());
                    t.add_column("id", types::primary());
                    t.add_column("int", types::integer());
                    t.add_column("string", types::text());
                });
            },
            api.schema_name(),
        )
        .await?;

    let dm = indoc! {r##"
        model Blog {
            bool    Boolean
            float   Float
            date    DateTime
            id      Int @id @default(autoincrement())
            int     Int
            string  String
        }
    "##};

    assert_eq_datamodels!(dm, &api.introspect().await?);

    Ok(())
}

#[test_each_connector]
async fn should_ignore_prisma_helper_tables(api: &TestApi) -> crate::TestResult {
    api.barrel()
        .execute_with_schema(
            |migration| {
                migration.create_table("Blog", move |t| {
                    t.add_column("id", types::primary());
                });

                migration.create_table("_RelayId", move |t| {
                    t.add_column("id", types::primary());
                    t.add_column("stablemodelidentifier", types::text());
                });

                migration.create_table("_Migration", move |t| {
                    t.add_column("revision", types::text());
                    t.add_column("name", types::text());
                    t.add_column("datamodel", types::text());
                    t.add_column("status", types::text());
                    t.add_column("applied", types::text());
                    t.add_column("rolled_back", types::text());
                    t.add_column("datamodel_steps", types::text());
                    t.add_column("database_migrations", types::text());
                    t.add_column("errors", types::text());
                    t.add_column("started_at", types::text());
                    t.add_column("finished_at", types::text());
                });

                migration.create_table("_prisma_migrations", move |t| {
                    t.add_column("id", types::primary());
                    t.add_column("checksum", types::text());
                    t.add_column("finished_at", types::text());
                    t.add_column("migration_name", types::text());
                    t.add_column("logs", types::text());
                    t.add_column("rolled_back_at", types::text());
                    t.add_column("started_at", types::text());
                    t.add_column("applied_steps_count", types::text());
                });
            },
            api.schema_name(),
        )
        .await?;

    let dm = indoc! {r##"
        model Blog {
            id      Int @id @default(autoincrement())
        }
    "##};

    assert_eq_datamodels!(dm, &api.introspect().await?);

    Ok(())
}

#[test_each_connector]
async fn a_table_with_compound_primary_keys(api: &TestApi) -> crate::TestResult {
    api.barrel()
        .execute_with_schema(
            |migration| {
                migration.create_table("Blog", |t| {
                    t.add_column("id", types::integer());
                    t.add_column("authorId", types::varchar(10));
                    t.set_primary_key(&["id", "authorId"]);
                });
            },
            api.schema_name(),
        )
        .await?;

    let dm = indoc! {r##"
        model Blog {
            id Int
            authorId String
            @@id([id, authorId])
        }
    "##};

    assert_eq_datamodels!(dm, &api.introspect().await?);

    Ok(())
}

#[test_each_connector]
async fn a_table_with_unique_index(api: &TestApi) -> crate::TestResult {
    api.barrel()
        .execute_with_schema(
            |migration| {
                migration.create_table("Blog", |t| {
                    t.add_column("id", types::primary());
                    t.add_column("authorId", types::r#char(10));
                    t.add_index("test", types::index(vec!["authorId"]).unique(true));
                });
            },
            api.schema_name(),
        )
        .await?;

    let dm = indoc! {r##"
        model Blog {
            id      Int @id @default(autoincrement())
            authorId String @unique
        }
    "##};

    assert_eq_datamodels!(dm, &api.introspect().await?);

    Ok(())
}

#[test_each_connector]
async fn a_table_with_multi_column_unique_index(api: &TestApi) -> crate::TestResult {
    api.barrel()
        .execute_with_schema(
            |migration| {
                migration.create_table("User", |t| {
                    t.add_column("id", types::primary());
                    t.add_column("firstname", types::varchar(10));
                    t.add_column("lastname", types::varchar(10));
                    t.add_index("test", types::index(vec!["firstname", "lastname"]).unique(true));
                });
            },
            api.schema_name(),
        )
        .await?;

    let dm = indoc! {r##"
        model User {
            id      Int @id @default(autoincrement())
            firstname String
            lastname String
            @@unique([firstname, lastname], name: "test")
        }
    "##};

    assert_eq_datamodels!(dm, &api.introspect().await?);

    Ok(())
}

#[test_each_connector]
async fn a_table_with_required_and_optional_columns(api: &TestApi) -> crate::TestResult {
    api.barrel()
        .execute_with_schema(
            |migration| {
                migration.create_table("User", |t| {
                    t.add_column("id", types::primary());
                    t.add_column("requiredname", types::varchar(255).nullable(false));
                    t.add_column("optionalname", types::varchar(255).nullable(true));
                });
            },
            api.schema_name(),
        )
        .await?;

    let dm = indoc! {r##"
        model User {
            id      Int @id @default(autoincrement())
            requiredname String
            optionalname String?
        }
    "##};

    assert_eq_datamodels!(dm, &api.introspect().await?);

    Ok(())
}

#[test_each_connector]
async fn a_table_with_default_values(api: &TestApi) -> crate::TestResult {
    api.barrel()
        .execute_with_schema(
            |migration| {
                migration.create_table("User", |t| {
                    t.add_column("a", types::text());
                    t.add_column("id", types::primary());
                    t.add_column("bool", types::boolean().default(false).nullable(false));
                    t.add_column("bool2", types::boolean().default(true).nullable(false));
                    t.add_column("float", types::float().default(5.3).nullable(false));
                    t.add_column("int", types::integer().default(5).nullable(false));
                    t.add_column("string", types::varchar(4).default("Test").nullable(false));
                });
            },
            api.schema_name(),
        )
        .await?;

    let dm = indoc! {r##"
        model User {
            a String
            id     Int     @id @default(autoincrement())
            bool   Boolean @default(false)
            bool2  Boolean @default(true)
            float  Float   @default(5.3)
            int    Int     @default(5)
            string String  @default("Test")
        }
    "##};

    assert_eq_datamodels!(dm, &api.introspect().await?);

    Ok(())
}

#[test_each_connector]
async fn a_table_with_a_non_unique_index(api: &TestApi) -> crate::TestResult {
    api.barrel()
        .execute_with_schema(
            |migration| {
                migration.create_table("User", |t| {
                    t.add_column("a", types::varchar(10));
                    t.add_column("id", types::primary());
                    t.add_index("test", types::index(vec!["a"]));
                });
            },
            api.schema_name(),
        )
        .await?;

    let dm = indoc! {r##"
        model User {
            a String
            id      Int @id @default(autoincrement())
            @@index([a], name: "test")
        }
    "##};

    assert_eq_datamodels!(dm, &api.introspect().await?);

    Ok(())
}

#[test_each_connector]
async fn a_table_with_a_multi_column_non_unique_index(api: &TestApi) -> crate::TestResult {
    api.barrel()
        .execute_with_schema(
            |migration| {
                migration.create_table("User", |t| {
                    t.add_column("a", types::varchar(10));
                    t.add_column("b", types::varchar(10));
                    t.add_column("id", types::primary());
                    t.add_index("test", types::index(vec!["a", "b"]));
                });
            },
            api.schema_name(),
        )
        .await?;

    let dm = indoc! { r##"
        model User {
            a  String
            b  String
            id Int @id @default(autoincrement())
            @@index([a,b], name: "test")
        }
    "##};

    assert_eq_datamodels!(dm, &api.introspect().await?);

    Ok(())
}

// SQLite does not have a serial type that's not a primary key.
#[test_each_connector(ignore("sqlite"))]
async fn a_table_with_non_id_autoincrement(api: &TestApi) -> crate::TestResult {
    api.barrel()
        .execute_with_schema(
            |migration| {
                migration.create_table("Test", |t| {
                    t.add_column("id", types::integer().primary(true));
                    t.add_column("authorId", types::serial().unique(true));
                });
            },
            api.schema_name(),
        )
        .await?;

    let dm = indoc! {r#"
        model Test {
            id       Int @id
            authorId Int @default(autoincrement()) @unique
        }
    "#};

    assert_eq_datamodels!(dm, &api.introspect().await?);

    Ok(())
}

#[test_each_connector]
async fn default_values(api: &TestApi) -> crate::TestResult {
    api.barrel()
        .execute_with_schema(
            |migration| {
                migration.create_table("Test", move |t| {
                    t.add_column("id", types::primary());
                    t.add_column(
                        "string_static_char",
                        types::custom("char(5)").default("test").nullable(true),
                    );
                    t.add_column(
                        "string_static_char_null",
                        types::r#char(5).default(types::null()).nullable(true),
                    );
                    t.add_column(
                        "string_static_varchar",
                        types::varchar(5).default("test").nullable(true),
                    );
                    t.add_column("int_static", types::integer().default(2).nullable(true));
                    t.add_column("float_static", types::float().default(1.43).nullable(true));
                    t.add_column("boolean_static", types::boolean().default(true).nullable(true));
                    t.add_column(
                        "datetime_now",
                        types::datetime().default(functions::current_timestamp()).nullable(true),
                    );
                });
            },
            api.schema_name(),
        )
        .await?;

    let dm = indoc! { r#"
        model Test {
            id                      Int       @id @default(autoincrement())
            string_static_char      String?   @default("test")
            string_static_char_null String?
            string_static_varchar   String?   @default("test")
            int_static              Int?      @default(2)
            float_static            Float?    @default(1.43)
            boolean_static          Boolean?  @default(true)
            datetime_now            DateTime? @default(now())
        }
    "#};

    assert_eq_datamodels!(dm, &api.introspect().await?);

    Ok(())
}

#[test_each_connector(tags("postgres"))]
async fn pg_default_value_as_dbgenerated(api: &TestApi) -> crate::TestResult {
    let sequence = "CREATE SEQUENCE test_seq START 1".to_string();
    api.database().execute_raw(&sequence, &[]).await?;

    api.barrel()
        .execute(|migration| {
            migration.create_table("Test", |t| {
                t.add_column("id", types::primary());
                t.inject_custom("string_function text Default E'  ' || '>' || ' '");
                t.inject_custom("int_serial Serial4");
                t.inject_custom("int_function Integer DEFAULT EXTRACT(year from TIMESTAMP '2001-02-16 20:38:40')");
                t.inject_custom("int_sequence Integer DEFAULT nextval('test_seq')");
                t.inject_custom("datetime_now TIMESTAMP DEFAULT NOW()");
                t.inject_custom("datetime_now_lc TIMESTAMP DEFAULT now()");
            });
        })
        .await?;

    let dm = indoc! {r#"
        model Test {
          id              Int       @id @default(autoincrement())
          string_function String?   @default(dbgenerated("(('  '::text || '>'::text) || ' '::text)"))
          int_serial      Int       @default(autoincrement())
          int_function    Int?      @default(dbgenerated("date_part('year'::text, '2001-02-16 20:38:40'::timestamp without time zone)"))
          int_sequence    Int?      @default(autoincrement())
          datetime_now    DateTime? @default(now())
          datetime_now_lc DateTime? @default(now())
          }
    "#};

    assert_eq_datamodels!(dm, &api.introspect().await?);

    Ok(())
}

//todo maybe need to split due to
// no function default values on mysql 5.7 and 8.0 -.-
// maria db allows this
#[test_each_connector(tags("mysql"))]
async fn my_default_value_as_dbgenerated(api: &TestApi) -> crate::TestResult {
    api.barrel()
        .execute(|migration| {
            migration.create_table("Test", |t| {
                t.add_column("id", types::primary());
                t.inject_custom("datetime_now TIMESTAMP NULL DEFAULT CURRENT_TIMESTAMP");
                t.inject_custom("datetime_now_lc TIMESTAMP NULL DEFAULT current_timestamp");
            });
        })
        .await?;

    let dm = indoc! {r#"
        model Test {
            id                      Int                 @id @default(autoincrement())
            datetime_now            DateTime?           @default(now())
            datetime_now_lc         DateTime?           @default(now())
        }
    "#};

    assert_eq_datamodels!(dm, &api.introspect().await?);

    Ok(())
}

#[test_each_connector(tags("mysql_8"))]
async fn a_table_with_an_index_that_contains_expressions_should_be_ignored(api: &TestApi) -> crate::TestResult {
    api.barrel()
        .execute_with_schema(
            |migration| {
                migration.create_table("Test", |t| {
                    t.add_column("id", types::integer().primary(true));
                    t.add_column("parentId", types::integer().nullable(true));
                    t.add_column("name", types::varchar(45).nullable(true));
                    t.inject_custom("UNIQUE KEY `SampleTableUniqueIndexName` (`name`,(ifnull(`parentId`,-(1))))");
                });
            },
            api.schema_name(),
        )
        .await?;

    let dm = indoc! {r#"
        model Test {
            id       Int     @id
            parentId Int?
            name     String?
        }
    "#};

    assert_eq_datamodels!(dm, &api.introspect().await?);

    Ok(())
}

#[test_each_connector(tags("postgres"))]
async fn default_values_on_lists_should_be_ignored(api: &TestApi) -> crate::TestResult {
    api.barrel()
        .execute(|migration| {
            migration.create_table("User", |t| {
                t.add_column("id", types::primary());
                t.inject_custom("ints integer[] DEFAULT array[]::integer[]");
                t.inject_custom("ints2 integer[] DEFAULT '{}'");
            });
        })
        .await?;

    let dm = indoc! {r#"
        datasource pg {
            provider = "postgres"
            url = "postgresql://localhost:5432"
        }

        model User {
            id      Int @id @default(autoincrement())
            ints    Int[]
            ints2   Int[]
        }
    "#};

    let result = format!(
        r#"
        datasource pg {{
            provider = "postgres"
            url = "postgresql://localhost:5432"
        }}

        {}
    "#,
        api.introspect().await?
    );

    assert_eq_datamodels!(dm, &result);

    Ok(())
}

// MySQL doesn't have partial indices.
#[test_each_connector(ignore("mysql"))]
async fn a_table_with_partial_indexes_should_ignore_them(api: &TestApi) -> crate::TestResult {
    api.barrel()
        .execute(move |migration| {
            migration.create_table("pages", move |t| {
                t.add_column("id", types::primary());
                t.add_column("staticId", types::integer().nullable(false));
                t.add_column("latest", types::integer().nullable(false));
                t.add_column("other", types::integer().nullable(false));
                t.add_index("full", types::index(vec!["other"]).unique(true));
                t.add_partial_index("partial", types::index(vec!["staticId"]).unique(true), "latest = 1");
            });
        })
        .await?;

    let dm = indoc! {r#"
        model pages {
            id       Int     @id @default(autoincrement())
            staticId Int
            latest   Int
            other    Int     @unique
        }
    "#};

    assert_eq_datamodels!(dm, &api.introspect().await?);

    Ok(())
}

#[test_each_connector(tags("postgres"))]
async fn introspecting_a_table_with_json_type_must_work(api: &TestApi) -> crate::TestResult {
    api.barrel()
        .execute(|migration| {
            migration.create_table("Blog", |t| {
                t.add_column("id", types::primary());
                t.add_column("json", types::json());
            });
        })
        .await?;

    let dm = indoc! {r#"
        datasource postgres {
            provider = "postgres"
            url = "postgresql://asdlj"
        }

        model Blog {
            id      Int @id @default(autoincrement())
            json    Json
        }
    "#};

    let expected = format!(
        r#"
        datasource postgres {{
            provider = "postgres"
            url = "postgresql://asdlj"
        }}

        {}
    "#,
        api.introspect().await?
    );

    assert_eq_datamodels!(dm, &expected);

    Ok(())
}

#[test_each_connector(tags("mariadb"))]
async fn different_default_values_should_work(api: &TestApi) -> crate::TestResult {
    api.barrel()
        .execute_with_schema(
            |migration| {
                migration.create_table("Blog", move |t| {
                    t.add_column("id", types::primary());
                    t.inject_custom("text Text Default \"one\"");
                    t.inject_custom(
                        "`tinytext_string` tinytext COLLATE utf8mb4_unicode_ci NOT NULL DEFAULT \"twelve\"",
                    );
                    t.inject_custom(
                        "`tinytext_number_string` tinytext COLLATE utf8mb4_unicode_ci NOT NULL DEFAULT \"1\"",
                    );
                    t.inject_custom("`tinytext_number` tinytext COLLATE utf8mb4_unicode_ci NOT NULL DEFAULT 10");
                    t.inject_custom("`tinytext_float` tinytext COLLATE utf8mb4_unicode_ci NOT NULL DEFAULT 1.0");
                    t.inject_custom("`tinytext_short` tinytext COLLATE utf8mb4_unicode_ci NOT NULL DEFAULT 1");
                });
            },
            api.schema_name(),
        )
        .await?;

    let dm = indoc! {r##"
        model Blog {
          id                     Int     @id @default(autoincrement())
          text                   String? @default("one")
          tinytext_string        String  @default("twelve")
          tinytext_number_string String  @default("1")
          tinytext_number        String  @default("10")
          tinytext_float         String  @default("1.0")
          tinytext_short         String  @default("1")
        }
    "##};

    assert_eq_datamodels!(dm, &api.introspect().await?);

    Ok(())
}

#[test_each_connector(ignore("sqlite"))]
async fn negative_default_values_should_work(api: &TestApi) -> crate::TestResult {
    api.barrel()
        .execute_with_schema(
            |migration| {
                migration.create_table("Blog", move |t| {
                    t.add_column("id", types::primary());
                    t.add_column("int", types::integer().default(1));
                    t.add_column("neg_int", types::integer().default(-1));
                    t.add_column("float", types::float().default(2.1));
                    t.add_column("neg_float", types::float().default(-2.1));
                    t.add_column("big_int", types::custom("bigint").default(3));
                    t.add_column("neg_big_int", types::custom("bigint").default(-3));
                });
            },
            api.schema_name(),
        )
        .await?;

    let dm = indoc! {r##"
        model Blog {
          id                     Int     @id @default(autoincrement())
          int                    Int     @default(1)
          neg_int                Int     @default(-1)
          float                  Float   @default(2.1)
          neg_float              Float   @default(-2.1)
          big_int                Int     @default(3)
          neg_big_int            Int     @default(-3)
        }
    "##};

    assert_eq_datamodels!(dm, &api.introspect().await?);

    Ok(())
}
