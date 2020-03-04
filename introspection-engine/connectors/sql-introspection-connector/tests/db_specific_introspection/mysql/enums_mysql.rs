use crate::*;
use barrel::types;
use test_harness::*;

#[test_each_connector(tags("mysql"))]
async fn introspecting_a_table_enums_should_work(api: &TestApi) {
    api.barrel()
        .execute(|migration| {
            migration.create_table("Book", |t| {
                t.add_column("id", types::primary());
                t.inject_custom("color  ENUM ( 'black', 'white') Not Null");
                t.inject_custom("color2  ENUM ( 'black2', 'white2') Not Null");
            });
        })
        .await;

    let dm = r#"
        model Book {
            color   Book_color
            color2  Book_color2
            id      Int     @default(autoincrement()) @id
        }

        enum Book_color{
            black
            white
        }

        enum Book_color2{
            black2
            white2
        }
    "#;

    let result = dbg!(api.introspect().await);
    let result1 = dbg!(api.introspect().await);
    let result2 = dbg!(api.introspect().await);
    let result3 = dbg!(api.introspect().await);
    let result4 = dbg!(api.introspect().await);
    custom_assert(&result, dm);
    custom_assert(&result1, dm);
    custom_assert(&result2, dm);
    custom_assert(&result3, dm);
    custom_assert(&result4, dm);
}

#[test_each_connector(tags("mysql"))]
async fn introspecting_a_table_enums_should_return_alphabetically_even_when_in_different_order(api: &TestApi) {
    api.barrel()
        .execute(|migration| {
            migration.create_table("Book", |t| {
                t.add_column("id", types::primary());
                t.inject_custom("color2  ENUM ( 'black2', 'white2') Not Null");
                t.inject_custom("color  ENUM ( 'black', 'white') Not Null");
            });
        })
        .await;

    let dm = r#"
        model Book {
            color   Book_color
            color2  Book_color2
            id      Int     @default(autoincrement()) @id
        }

        enum Book_color{
            black
            white
        }

        enum Book_color2{
            black2
            white2
        }
    "#;

    let result = dbg!(api.introspect().await);
    let result1 = dbg!(api.introspect().await);
    let result2 = dbg!(api.introspect().await);
    let result3 = dbg!(api.introspect().await);
    let result4 = dbg!(api.introspect().await);
    custom_assert(&result, dm);
    custom_assert(&result1, dm);
    custom_assert(&result2, dm);
    custom_assert(&result3, dm);
    custom_assert(&result4, dm);
}

#[test_each_connector(tags("mysql"))]
async fn introspecting_a_table_with_enum_default_values_should_work(api: &TestApi) {
    api.barrel()
        .execute(|migration| {
            migration.create_table("Book", |t| {
                t.add_column("id", types::primary());
                t.inject_custom("color  Enum(\"black\", \"white\") Not Null default \"black\"");
            });
        })
        .await;

    let dm = r#"
        model Book {
            color   Book_color   @default(black)
            id      Int     @default(autoincrement()) @id
        }

        enum Book_color{
            black
            white
        }
    "#;

    let result = dbg!(api.introspect().await);
    custom_assert(&result, dm);
}
