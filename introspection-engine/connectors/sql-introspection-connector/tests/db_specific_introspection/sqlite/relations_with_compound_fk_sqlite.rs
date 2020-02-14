use crate::*;
use barrel::types;
use test_harness::*;

#[test_each_connector(tags("sqlite"))]
#[test]
async fn compound_foreign_keys_should_work_for_one_to_one_relations(api: &TestApi) {
    let barrel = api.barrel();
    let _setup_schema = barrel
        .execute(|migration| {
            migration.create_table("User", |t| {
                t.add_column("id", types::primary());
                t.add_column("age", types::integer());
                t.inject_custom("CONSTRAINT user_unique UNIQUE(`id`, `age`)");
            });
            migration.create_table("Post", |t| {
                t.add_column("id", types::primary());
                t.add_column("user_id", types::integer().nullable(true));
                t.add_column("user_age", types::integer().nullable(true));
                t.inject_custom("FOREIGN KEY (`user_id`,`user_age`) REFERENCES `User`(`id`, `age`)");
                t.inject_custom("CONSTRAINT post_user_unique UNIQUE(`user_id`, `user_age`)");
            });
        })
        .await;

    let dm = r#"
            model User {
               age      Int
               id       Int                 @id @default(autoincrement())
               post     Post?

               @@unique([id, age], name: "sqlite_autoindex_User_1")
            }

            model Post {
                id      Int                 @id @default(autoincrement())
                user    User?               @map(["user_id", "user_age"]) @relation(references:[id, age])
            }

        "#;
    let result = dbg!(api.introspect().await);
    custom_assert(&result, dm);
}

#[test_each_connector(tags("sqlite"))]
#[test]
async fn compound_foreign_keys_should_work_for_required_one_to_one_relations(api: &TestApi) {
    let barrel = api.barrel();
    let _setup_schema = barrel
        .execute(|migration| {
            migration.create_table("User", |t| {
                t.add_column("id", types::primary());
                t.add_column("age", types::integer());
                t.inject_custom("CONSTRAINT user_unique UNIQUE(`id`, `age`)");
            });
            migration.create_table("Post", |t| {
                t.add_column("id", types::primary());
                t.add_column("user_id", types::integer());
                t.add_column("user_age", types::integer());
                t.inject_custom("FOREIGN KEY (`user_id`,`user_age`) REFERENCES `User`(`id`, `age`)");
                t.inject_custom("CONSTRAINT post_user_unique UNIQUE(`user_id`, `user_age`)");
            });
        })
        .await;

    let dm = r#"
            model User {
               age     Int
               id       Int                 @id @default(autoincrement())
               post     Post?

               @@unique([id, age], name: "sqlite_autoindex_User_1")
            }

            model Post {
                id      Int                 @id  @default(autoincrement())
                user    User                @map(["user_id", "user_age"]) @relation(references:[id, age])
            }

        "#;
    let result = dbg!(api.introspect().await);
    custom_assert(&result, dm);
}

#[test_each_connector(tags("sqlite"))]
#[test]
async fn compound_foreign_keys_should_work_for_one_to_many_relations(api: &TestApi) {
    let barrel = api.barrel();
    let _setup_schema = barrel
        .execute(|migration| {
            migration.create_table("User", |t| {
                t.add_column("id", types::primary());
                t.add_column("age", types::integer());
                t.inject_custom("CONSTRAINT user_unique UNIQUE(`id`, `age`)");
            });
            migration.create_table("Post", |t| {
                t.add_column("id", types::primary());
                t.add_column("user_id", types::integer().nullable(true));
                t.add_column("user_age", types::integer().nullable(true));
                t.inject_custom("FOREIGN KEY (`user_id`,`user_age`) REFERENCES `User`(`id`, `age`)");
            });
        })
        .await;

    let dm = r#"
            model User {
               age      Int
               id       Int                 @id @default(autoincrement())
               post     Post[]

               @@unique([id, age], name: "sqlite_autoindex_User_1")
            }

            model Post {
                id      Int                 @id @default(autoincrement())
                user    User?               @map(["user_id", "user_age"]) @relation(references:[id, age])
            }

        "#;
    let result = dbg!(api.introspect().await);
    custom_assert(&result, dm);
}

#[test_each_connector(tags("sqlite"))]
#[test]
async fn compound_foreign_keys_should_work_for_required_one_to_many_relations(api: &TestApi) {
    let barrel = api.barrel();
    let _setup_schema = barrel
        .execute(|migration| {
            migration.create_table("User", |t| {
                t.add_column("id", types::primary());
                t.add_column("age", types::integer());
                t.inject_custom("CONSTRAINT user_unique UNIQUE(`id`, `age`)");
            });
            migration.create_table("Post", |t| {
                t.add_column("id", types::primary());
                t.add_column("user_id", types::integer());
                t.add_column("user_age", types::integer());
                t.inject_custom("FOREIGN KEY (`user_id`,`user_age`) REFERENCES `User`(`id`, `age`)");
            });
        })
        .await;

    let dm = r#"
            model User {
               age      Int
               id       Int                 @id @default(autoincrement())
               post     Post[]

               @@unique([id, age], name: "sqlite_autoindex_User_1")
            }

            model Post {
                id      Int                 @id @default(autoincrement())
                user    User               @map(["user_id", "user_age"]) @relation(references:[id, age])
            }
        "#;
    let result = dbg!(api.introspect().await);
    custom_assert(&result, dm);
}

#[test_each_connector(tags("sqlite"))]
#[test]
async fn compound_foreign_keys_should_work_for_required_self_relations(api: &TestApi) {
    let barrel = api.barrel();
    let _setup_schema = barrel
        .execute(|migration| {
            migration.create_table("Person", |t| {
                t.add_column("id", types::primary());
                t.add_column("age", types::integer());
                t.add_column("partner_id", types::integer());
                t.add_column("partner_age", types::integer());
                t.inject_custom("FOREIGN KEY (`partner_id`,`partner_age`) REFERENCES `Person`(`id`, `age`)");
                t.inject_custom("CONSTRAINT `person_unique` UNIQUE (`id`, `age`)");
            });
        })
        .await;

    let dm = r#"
            model Person {
               age      Int
               id       Int         @id  @default(autoincrement())
               person   Person      @map(["partner_id", "partner_age"]) @relation("PersonToPerson_partner_id_partner_age", references: [id,age])
               other_person   Person[]    @relation("PersonToPerson_partner_id_partner_age")

               @@unique([id, age], name: "sqlite_autoindex_Person_1")
            }
        "#;
    let result = dbg!(api.introspect().await);
    custom_assert(&result, dm);
}

#[test_each_connector(tags("sqlite"))]
#[test]
async fn compound_foreign_keys_should_work_for_self_relations(api: &TestApi) {
    let barrel = api.barrel();
    let _setup_schema = barrel
        .execute(|migration| {
            migration.create_table("Person", |t| {
                t.add_column("id", types::primary());
                t.add_column("age", types::integer());
                t.add_column("partner_id", types::integer().nullable(true));
                t.add_column("partner_age", types::integer().nullable(true));
                t.inject_custom("FOREIGN KEY (`partner_id`,`partner_age`) REFERENCES `Person`(`id`, `age`)");
                t.inject_custom("CONSTRAINT `person_unique` UNIQUE (`id`, `age`)");
            });
        })
        .await;

    let dm = r#"
            model Person {
               age      Int
               id       Int         @id  @default(autoincrement())
               person   Person?     @map(["partner_id", "partner_age"]) @relation("PersonToPerson_partner_id_partner_age", references: [id, age])
               other_person   Person[]    @relation("PersonToPerson_partner_id_partner_age")

               @@unique([id, age], name: "sqlite_autoindex_Person_1")
            }
        "#;
    let result = dbg!(api.introspect().await);
    custom_assert(&result, dm);
}

#[test_each_connector(tags("sqlite"))]
#[test]
async fn compound_foreign_keys_should_work_with_defaults(api: &TestApi) {
    let barrel = api.barrel();
    let _setup_schema = barrel
        .execute(|migration| {
            migration.create_table("Person", |t| {
                t.add_column("id", types::primary());
                t.add_column("age", types::integer());
                t.add_column("partner_id", types::integer().default(0));
                t.add_column("partner_age", types::integer().default(0));
                t.inject_custom("FOREIGN KEY (`partner_id`,`partner_age`) REFERENCES `Person`(`id`, `age`)");
                t.inject_custom("CONSTRAINT `person_unique` UNIQUE (`id`, `age`)");
            });
        })
        .await;

    let dm = r#"
            model Person {
               age      Int
               id       Int         @id  @default(autoincrement())
               person   Person      @map(["partner_id", "partner_age"]) @relation("PersonToPerson_partner_id_partner_age", references: [id, age])
               other_person   Person[]    @relation("PersonToPerson_partner_id_partner_age")

               @@unique([id, age], name: "sqlite_autoindex_Person_1")
            }
        "#;
    let result = dbg!(api.introspect().await);
    custom_assert(&result, dm);
}

//todo decide on this,
// this can at most be a one:one relation, but with a more limited subset of available connections
// fetch this from indexes
// what about separate uniques? all @unique == @@unique ?? No! separate ones do not fully work since you can only connect to a subset of the @@unique case
// model.indexes contains a multi-field unique index that matches the colums exactly, then it is unique
// if there are separate uniques it probably should not become a relation
// what breaks by having an @@unique that refers to fields that do not have a representation on the model anymore due to the merged relation field?
//#[test_each_connector(tags("sqlite"))]
//#[test]
//async fn compound_foreign_keys_should_work_for_one_to_one_relations_with_separate_uniques(api: &TestApi) {
//    let barrel = api.barrel();
//    let _setup_schema = barrel
//        .execute(|migration| {
//            migration.create_table("User", |t| {
//                t.add_column("id", types::primary());
//                t.add_column("age", types::integer());
//                t.inject_custom("CONSTRAINT user_unique UNIQUE(`id`, `age`)");
//            });
//            migration.create_table("Post", |t| {
//                t.add_column("id", types::primary());
//                t.add_column("user_id", types::integer().unique(true));
//                t.add_column("user_age", types::integer().unique(true));
//                t.inject_custom("FOREIGN KEY (`user_id`,`user_age`) REFERENCES `User`(`id`, `age`)");
//            });
//        })
//        .await;
//
//    let dm = r#"
//            model Post {
//                id      Int                 @id
//                user    User                @map(["user_id", "user_age"]) @relation(references:[id, age])
//            }
//
//            model User {
//               age      Int
//               id       Int                 @id
//               post     Post?
//
//               @@unique([id, age], name: "sqlite_autoindex_User_1")
//            }
//        "#;
//    let result = dbg!(api.introspect().await);
//    custom_assert(&result, dm);
//}

// the fk indexes are created implicitly on mysql
#[test_each_connector(tags("sqlite"))]
#[test]
async fn compound_foreign_keys_should_work_for_one_to_many_relations_with_non_unique_index(api: &TestApi) {
    let barrel = api.barrel();
    let _setup_schema = barrel
        .execute(|migration| {
            migration.create_table("User", |t| {
                t.add_column("id", types::primary());
                t.add_column("age", types::integer());
                t.inject_custom("CONSTRAINT user_unique UNIQUE(`id`, `age`)");
            });
            migration.create_table("Post", |t| {
                t.add_column("id", types::primary());
                t.add_column("user_id", types::integer());
                t.add_column("user_age", types::integer());
                t.inject_custom("FOREIGN KEY (`user_id`,`user_age`) REFERENCES `User`(`id`, `age`)");
                t.add_index("test", types::index(vec!["user_id", "user_age"]));
            });
        })
        .await;

    let dm = r#"
            model User {
               age      Int
               id       Int                 @id @default(autoincrement())
               post     Post[]

               @@unique([id, age], name: "sqlite_autoindex_User_1")
            }

            model Post {
                id      Int                 @id @default(autoincrement())
                user    User                @map(["user_id", "user_age"]) @relation(references:[id, age])

                @@index([user], name: "test")
            }
        "#;
    let result = dbg!(api.introspect().await);
    custom_assert(&result, dm);
}
