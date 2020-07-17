use crate::*;
use barrel::types;
use pretty_assertions::assert_eq;
use test_harness::*;

#[test_each_connector(tags("sqlite"))]
async fn introspecting_a_table_with_reserved_name_should_rename(api: &TestApi) {
    api.barrel()
        .execute(|migration| {
            migration.create_table("Transaction", |t| {
                t.add_column("id", types::primary());
            });
            // migration.create_table("Post", |t| {
            //     t.add_column("id", types::integer());
            //     t.inject_custom(
            //         "user_id INTEGER NOT NULL,
            //     FOREIGN KEY (`user_id`) REFERENCES `User`(`id`)",
            //     )
            // });
        })
        .await;

    // let dm = "model User {\n  id      Int    @default(autoincrement()) @id\n  // Post Post[]\n}\n\n// The underlying table does not contain a valid unique identifier and can therefore currently not be handled.\n// model Post {\n  // id      Int\n  // user_id Int\n  // User    User @relation(fields: [user_id], references: [id])\n// }\n";

    let result = dbg!(api.introspect().await);
    dbg!(result)
    // assert_eq!(&result, dm);
}
