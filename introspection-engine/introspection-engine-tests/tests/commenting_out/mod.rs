use barrel::types;
use indoc::indoc;
use introspection_engine_tests::{assert_eq_json, test_api::*};
use pretty_assertions::assert_eq;
use serde_json::json;
use test_macros::test_each_connector;

#[test_each_connector]
async fn a_table_without_uniques_should_ignore(api: &TestApi) -> crate::TestResult {
    api.barrel()
        .execute(|migration| {
            migration.create_table("User", |t| {
                t.add_column("id", types::primary());
            });
            migration.create_table("Post", |t| {
                t.add_column("id", types::integer());
                t.add_column("user_id", types::integer().nullable(false));
                t.add_foreign_key(&["user_id"], "User", &["id"]);
            });
        })
        .await?;

    let dm = if api.sql_family().is_mysql() {
        indoc! {r#"
            /// The underlying table does not contain a valid unique identifier and can therefore currently not be handled by the Prisma Client.
            model Post {
              id      Int
              user_id Int
              User    User @relation(fields: [user_id], references: [id])

              @@index([user_id], name: "user_id")
              @@ignore
            }

            model User {
              id   Int    @id @default(autoincrement())
              Post Post[] @ignore
            }
        "#}
    } else {
        indoc! {r#"
            /// The underlying table does not contain a valid unique identifier and can therefore currently not be handled by the Prisma Client.
            model Post {
              id      Int
              user_id Int
              User    User @relation(fields: [user_id], references: [id])

              @@ignore
            }

            model User {
              id   Int    @id @default(autoincrement())
              Post Post[] @ignore
            }
        "#}
    };

    assert_eq!(dm, &api.introspect().await?);

    Ok(())
}

#[test_each_connector(tags("postgres"))]
async fn relations_between_ignored_models_should_not_have_field_level_ignores(api: &TestApi) -> crate::TestResult {
    api.barrel()
        .execute(|migration| {
            migration.create_table("User", |t| {
                t.inject_custom("id macaddr primary key not null");
            });
            migration.create_table("Post", |t| {
                t.inject_custom("id macaddr primary key not null");
                t.inject_custom("user_id macaddr not null");
                t.add_foreign_key(&["user_id"], "User", &["id"]);
            });
        })
        .await?;

    let dm = indoc! {r#"
            /// The underlying table does not contain a valid unique identifier and can therefore currently not be handled by the Prisma Client.
            model Post {
              id      Unsupported("macaddr") @id
              user_id Unsupported("macaddr")
              User    User                   @relation(fields: [user_id], references: [id])
            
              @@ignore
            }
            
            /// The underlying table does not contain a valid unique identifier and can therefore currently not be handled by the Prisma Client.
            model User {
              id   Unsupported("macaddr") @id
              Post Post[]
            
              @@ignore
            }
        "#};

    assert_eq!(dm, &api.introspect().await?);

    Ok(())
}

#[test_each_connector]
async fn a_table_without_required_uniques(api: &TestApi) -> crate::TestResult {
    api.barrel()
        .execute(|migration| {
            migration.create_table("Post", |t| {
                t.add_column("id", types::integer());
                t.add_column("opt_unique", types::integer().unique(true).nullable(true));
            });
        })
        .await?;

    let dm = indoc! {r#"
        /// The underlying table does not contain a valid unique identifier and can therefore currently not be handled by the Prisma Client.
        model Post {
          id         Int
          opt_unique Int? @unique

          @@ignore
        }
    "#};

    assert_eq!(dm, &api.introspect().await?);

    Ok(())
}

#[test_each_connector]
async fn a_table_without_fully_required_compound_unique(api: &TestApi) -> crate::TestResult {
    api.barrel()
        .execute(|migration| {
            migration.create_table("Post", |t| {
                t.add_column("id", types::integer());
                t.add_column("opt_unique", types::integer().nullable(true));
                t.add_column("req_unique", types::integer().nullable(false));

                t.add_constraint(
                    "sqlite_autoindex_Post_1",
                    types::unique_constraint(vec!["opt_unique", "req_unique"]),
                );
            });
        })
        .await?;

    let dm = indoc! {r#"
        /// The underlying table does not contain a valid unique identifier and can therefore currently not be handled by the Prisma Client.
        model Post {
          id         Int
          opt_unique Int?
          req_unique Int

          @@unique([opt_unique, req_unique], name: "sqlite_autoindex_Post_1")
          @@ignore
        }
    "#};

    assert_eq!(dm, &api.introspect().await?);

    Ok(())
}

#[test_each_connector(tags("postgres"))]
async fn unsupported_type_keeps_its_usages(api: &TestApi) -> crate::TestResult {
    api.barrel()
        .execute(|migration| {
            migration.create_table("Test", |t| {
                t.add_column("id", types::integer().unique(true));
                t.add_column("dummy", types::integer());
                t.add_column("broken", types::custom("macaddr"));
                t.add_index("unique", types::index(vec!["broken", "dummy"]).unique(true));
                t.add_index("non_unique", types::index(vec!["broken", "dummy"]).unique(false));
                t.set_primary_key(&["broken", "dummy"]);
            });
        })
        .await?;

    let expected = json!([{
        "code": 3,
        "message": "These fields are not supported by the Prisma Client, because Prisma currently does not support their types.",
        "affected": [
            {
                "model": "Test",
                "field": "broken",
                "tpe": "macaddr"
            }
        ]
    }]);

    assert_eq_json!(expected, api.introspection_warnings().await?);

    let dm = indoc! {r#"
        modelTest{
            id          Int     @unique
            dummy       Int
            broken Unsupported("macaddr")

            @@id([broken, dummy])
            @@unique([broken, dummy], name: "unique")
            @@index([broken, dummy], name: "non_unique")
        }
    "#};

    assert_eq!(dm.replace(" ", ""), api.introspect().await?.replace(" ", ""));

    Ok(())
}

#[test_each_connector(tags("postgres"))]
async fn a_table_with_only_an_unsupported_id(api: &TestApi) -> crate::TestResult {
    api.barrel()
        .execute(|migration| {
            migration.create_table("Test", |t| {
                t.add_column("dummy", types::integer());
                t.add_column(
                    "network_mac",
                    types::custom("macaddr").primary(true).default("08:00:2b:01:02:03"),
                );
            });
        })
        .await?;

    let expected = json!([
        {
            "code": 1,
            "message": "The following models were commented out as they do not have a valid unique identifier or id. This is currently not supported by the Prisma Client.",
            "affected": [{
                "model": "Test"
            }]
        },
        {
            "code": 3,
            "message": "These fields are not supported by the Prisma Client, because Prisma currently does not support their types.",
            "affected": [{
                "model": "Test",
                "field": "network_mac",
                "tpe": "macaddr"
            }]
        }
    ]);

    assert_eq_json!(expected, api.introspection_warnings().await?);

    let dm = indoc! {r#"
        /// The underlying table does not contain a valid unique identifier and can therefore currently not be handled by the Prisma Client.
        model Test {
          dummy       Int
          network_mac Unsupported("macaddr") @id @default(dbgenerated("'08:00:2b:01:02:03'::macaddr"))

          @@ignore
        }
    "#};

    assert_eq!(dm, &api.introspect().await?);

    Ok(())
}

#[test_each_connector(tags("postgres"))]
async fn a_table_with_unsupported_types_in_a_relation(api: &TestApi) -> crate::TestResult {
    api.barrel()
        .execute(|migration| {
            migration.create_table("User", |t| {
                t.add_column("id", types::primary());
                t.inject_custom("ip cidr not null unique");
            });
            migration.create_table("Post", |t| {
                t.add_column("id", types::primary());
                t.inject_custom("user_ip cidr not null ");
                t.add_foreign_key(&["user_ip"], "User", &["ip"]);
            });
        })
        .await?;

    let dm = indoc! {r#"
            model Post {
              id            Int                     @id @default(autoincrement())
              user_ip       Unsupported("cidr")
              User          User                    @relation(fields: [user_ip], references: [ip])
            }

            model User {
              id            Int                     @id @default(autoincrement())
              ip            Unsupported("cidr")  @unique
              Post          Post[]
            }
        "#};

    assert_eq!(dm.replace(" ", ""), api.introspect().await?.replace(" ", ""));

    Ok(())
}

#[test_each_connector]
async fn remapping_field_names_to_empty(api: &TestApi) -> crate::TestResult {
    api.barrel()
        .execute(|migration| {
            migration.create_table("User", |t| {
                t.add_column("1", types::text());
                t.add_column("last", types::primary());
            });
        })
        .await?;

    let dm = indoc! {r#"
        model User {
          // This field was commented out because of an invalid name. Please provide a valid one that matches [a-zA-Z][a-zA-Z0-9_]*
          // 1 String @map("1")
          last Int    @id @default(autoincrement())
        }
    "#};

    assert_eq!(dm, &api.introspect().await?);

    Ok(())
}

#[test_each_connector(tags("postgres"))]
async fn db_generated_values_should_add_comments(api: &TestApi) -> crate::TestResult {
    api.barrel()
        .execute_with_schema(
            |migration| {
                migration.create_table("Blog", move |t| {
                    t.add_column("id", types::primary());
                    t.inject_custom("number Integer Default 1");
                    t.inject_custom("bigger_number Integer DEFAULT sqrt(4)");
                    t.inject_custom("point Point DEFAULT Point(0, 0)");
                });
            },
            api.schema_name(),
        )
        .await?;

    let dm = indoc! {r##"
        model Blog {
          id            Int    @id @default(autoincrement())
          number        Int?   @default(1)
          bigger_number Int?   @default(dbgenerated("sqrt((4)::double precision)"))
          point      Unsupported("point")? @default(dbgenerated("point((0)::double precision, (0)::double precision)"))
        }
    "##};

    assert_eq!(dm.replace(" ", ""), api.introspect().await?.replace(" ", ""));

    Ok(())
}

#[test_each_connector(tags("postgres"))]
async fn commenting_out_a_table_without_columns(api: &TestApi) -> crate::TestResult {
    api.barrel()
        .execute(|migration| {
            migration.create_table("Test", |_t| {});
        })
        .await?;

    let expected = json!([{
        "code": 14,
        "message": "The following models were commented out as we could not retrieve columns for them. Please check your privileges.",
        "affected": [
            {
                "model": "Test"
            }
        ]
    }]);

    assert_eq_json!(expected, api.introspection_warnings().await?);

    let dm = indoc! {r#"
        // We could not retrieve columns for the underlying table. Either it has none or you are missing rights to see them. Please check your privileges.
        // model Test {
        // }
    "#};

    assert_eq!(dm, &api.introspect().await?);

    Ok(())
}

#[test_each_connector(tags("postgres"))]
async fn ignore_on_back_relation_field_if_pointing_to_ignored_model(api: &TestApi) -> crate::TestResult {
    api.barrel()
        .execute(|migration| {
            migration.create_table("User", |t| {
                t.add_column("id", types::primary());
                t.inject_custom("ip integer not null unique");
            });
            migration.create_table("Post", |t| {
                t.add_column("id", types::integer());
                t.inject_custom("user_ip integer not null ");
                t.add_foreign_key(&["user_ip"], "User", &["ip"]);
            });
        })
        .await?;

    let dm = indoc! {r#"
            ///The underlying table does not contain a valid unique identifier and can therefore currently not be handled by the Prisma Client.
            model Post {
                id      Int
                user_ip Int
                User    User @relation(fields: [user_ip], references: [ip])

                @@ignore
            }

            model User {
                id      Int  @id @default(autoincrement())
                ip      Int  @unique
                Post  Post[] @ignore
            }
        "#};

    assert_eq!(dm.replace(" ", ""), api.introspect().await?.replace(" ", ""));

    Ok(())
}
