extern crate datamodel;
// use pretty_assertions::assert_eq;

// TODO: test `onDelete` back once `prisma migrate` is a thing
const DATAMODEL_STRING: &str = r#"model User {
  id        Int      @id
  createdAt DateTime
  email     String   @unique
  name      String?
  posts     Post[]   @relation("author")
  profile   Profile?

  @@map("user")
}

model Profile {
  id   Int    @id
  user User
  bio  String

  @@map("profile")
}

model Post {
  id         Int
  createdAt  DateTime
  updatedAt  DateTime
  title      String           @default("Default-Title")
  wasLiked   Boolean          @default(false)
  author     User             @relation("author")
  published  Boolean          @default(false)
  categories PostToCategory[]

  @@id([title, createdAt])
  @@map("post")
}

model Category {
  id    Int              @id
  name  String
  posts PostToCategory[]
  cat   CategoryEnum

  @@map("category")
}

model PostToCategory {
  id       Int      @id
  post     Post
  category Category

  @@map("post_to_category")
}

model A {
  id Int @id
  b  B
}

model B {
  id Int @id
  a  A
}

enum CategoryEnum {
  A
  B
  C
}"#;

// #[test]
// fn test_dmmf_roundtrip() {
//     let dml = datamodel::parse_datamodel(&DATAMODEL_STRING).unwrap();
//     let dmmf = datamodel::json::dmmf::render_to_dmmf(&dml);
//     let dml2 = datamodel::json::dmmf::parse_from_dmmf(&dmmf);
//     let rendered = datamodel::render_datamodel_to_string(&dml2).unwrap();
//
//     println!("{}", rendered);
//
//     assert_eq!(DATAMODEL_STRING, rendered);
// }

const DATAMODEL_STRING_WITH_FUNCTIONS: &str = r#"model User {
  id        Int      @id
  createdAt DateTime @default(now())
  someId    String   @default(cuid()) @unique
}"#;

// #[test]
// fn test_dmmf_roundtrip_with_functions() {
//     let dml = datamodel::parse_datamodel(&DATAMODEL_STRING_WITH_FUNCTIONS).unwrap();
//     let dmmf = datamodel::json::dmmf::render_to_dmmf(&dml);
//     let dml2 = datamodel::json::dmmf::parse_from_dmmf(&dmmf);
//     let rendered = datamodel::render_datamodel_to_string(&dml2).unwrap();
//
//     println!("{}", rendered);
//
//     assert_eq!(DATAMODEL_STRING_WITH_FUNCTIONS, rendered);
// }

const DATAMODEL_WITH_SOURCE: &str = r#"datasource pg1 {
  provider = "postgresql"
  url      = env("PG_URL")
}

model Author {
  id        Int      @id
  name      String?
  createdAt DateTime @default(now())
}"#;

// #[test]
// fn test_dmmf_roundtrip_with_sources() {
//     std::env::set_var("PG_URL", "https://localhost/postgres1");
//     let rendered = dmmf_roundtrip(DATAMODEL_WITH_SOURCE);
//
//     assert_eq!(DATAMODEL_WITH_SOURCE, rendered);
// }

const DATAMODEL_WITH_SOURCE_AND_COMMENTS: &str = r#"/// Super cool postgres source.
datasource pg1 {
  provider = "postgresql"
  url      = "https://localhost/postgres1"
}

/// My author model.
model Author {
  id        Int      @id
  /// Name of the author.
  name      String?
  createdAt DateTime @default(now())
}"#;

// #[test]
// fn test_dmmf_roundtrip_with_sources_and_comments() {
//     let rendered = dmmf_roundtrip(DATAMODEL_WITH_SOURCE_AND_COMMENTS);
//
//     assert_eq!(DATAMODEL_WITH_SOURCE_AND_COMMENTS, rendered);
// }

const DATAMODEL_WITH_GENERATOR: &str = r#"generator js {
  provider      = "javascript"
  output        = "./client"
  binaryTargets = ["a", "b"]
  extra_config  = "test"
}

generator foo {
  provider = "bar"
}

model Author {
  id        Int      @id
  name      String?
  createdAt DateTime @default(now())
}"#;

// #[test]
// fn test_dmmf_roundtrip_with_generator() {
//     let rendered = dmmf_roundtrip(DATAMODEL_WITH_GENERATOR);
//
//     assert_eq!(DATAMODEL_WITH_GENERATOR, rendered);
// }

const DMFF_WITHOUT_RELATION_NAME: &str = r#"
{
  "enums": [],
  "models": [
    {
      "name": "User",
      "isEmbedded": false,
      "dbName": null,
      "idFields": [],
      "fields": [
        {
          "name": "id",
          "kind": "scalar",
          "dbNames": [],
          "isList": false,
          "isRequired": true,
          "isUnique": false,
          "isId": true,
          "type": "Int",
          "isGenerated": false,
          "isUpdatedAt": false
        },
        {
          "name": "posts",
          "kind": "object",
          "dbNames": [],
          "isList": true,
          "isRequired": false,
          "isUnique": false,
          "isId": false,
          "type": "Post",
          "relationToFields": [],
          "relationOnDelete": "NONE",
          "isGenerated": false,
          "isUpdatedAt": false
        }
      ],
      "isGenerated": false
    },
    {
      "name": "Post",
      "isEmbedded": false,
      "dbName": null,
      "idFields": [],
      "fields": [
        {
          "name": "id",
          "kind": "scalar",
          "dbNames": [],
          "isList": false,
          "isRequired": true,
          "isUnique": false,
          "isId": true,
          "type": "Int",
          "isGenerated": false,
          "isUpdatedAt": false
        },
        {
          "name": "user",
          "kind": "object",
          "dbNames": [],
          "isList": false,
          "isRequired": true,
          "isUnique": false,
          "isId": false,
          "type": "User",
          "relationToFields": [
            "id"
          ],
          "relationOnDelete": "NONE",
          "isGenerated": false,
          "isUpdatedAt": false
        }
      ],
      "isGenerated": false
    }
  ]
}
"#;

const DML_WITHOUT_RELATION_NAME: &str = r#"model User {
  id    Int    @id
  posts Post[]
}

model Post {
  id   Int  @id
  user User
}"#;

// <<<<<<< HEAD
// #[test]
// fn should_serialize_dmmf_without_relation_name_correctly() {
//     let dml = datamodel::json::dmmf::parse_from_dmmf(DMFF_WITHOUT_RELATION_NAME);
//     let rendered = datamodel::render_datamodel_to_string(&dml).unwrap();
//
//     assert_eq!(DML_WITHOUT_RELATION_NAME, rendered);
// }

// fn dmmf_roundtrip(input: &str) -> String {
//     let dml = datamodel::parse_datamodel(input).unwrap();
//     let config = datamodel::parse_configuration(input).unwrap();
//
//     let dmmf = datamodel::json::dmmf::render_to_dmmf(&dml);
//     let mcf = datamodel::json::mcf::config_to_mcf_json(&config);
//
//     let dml2 = datamodel::json::dmmf::parse_from_dmmf(&dmmf);
//     let config = datamodel::json::mcf::config_from_mcf_json(&mcf);
//
//     let rendered = datamodel::render_datamodel_and_config_to_string(&dml2, &config).unwrap();
//
//     println!("{}", rendered);
//
//     rendered
// }
// =======
// #[test]
// #[ignore]
// fn should_serialize_dmmf_without_relation_name_correctly() {
//     let dml = datamodel::json::dmmf::parse_from_dmmf(DMFF_WITHOUT_RELATION_NAME);
//     let rendered = datamodel::render_datamodel_to_string(&dml).unwrap();
//
//     assert_eq!(DML_WITHOUT_RELATION_NAME, rendered);
// }
//
// fn dmmf_roundtrip(input: &str) -> String {
//     let dml = datamodel::parse_datamodel(input).unwrap();
//     let config = datamodel::parse_configuration(input).unwrap();
//
//     let dmmf = datamodel::json::dmmf::render_to_dmmf(&dml);
//     let mcf = datamodel::json::mcf::config_to_mcf_json(&config);
//
//     let dml2 = datamodel::json::dmmf::parse_from_dmmf(&dmmf);
//     let config = datamodel::json::mcf::config_from_mcf_json(&mcf);
//
//     let rendered = datamodel::render_datamodel_and_config_to_string(&dml2, &config).unwrap();
//
//     println!("{}", rendered);
//
//     rendered
// }
// >>>>>>> master
