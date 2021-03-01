use datamodel::parse_datamodel;
use indoc::indoc;
use pretty_assertions::assert_eq;

#[test]
fn back_relation_fields_must_be_added() {
    let input = r#"model Blog {
  id    Int     @id
  posts Post[]
}

model Post {
  id Int   @id
}

model Post2 {
  id     Int  @id
  blogId Int
  Blog   Blog @relation(fields: [blogId], references: [id])
}
"#;

    let expected = r#"model Blog {
  id    Int     @id
  posts Post[]
  Post2 Post2[]
}

model Post {
  id     Int   @id
  Blog   Blog? @relation(fields: [blogId], references: [id])
  blogId Int?
}

model Post2 {
  id     Int  @id
  blogId Int
  Blog   Blog @relation(fields: [blogId], references: [id])
}
"#;

    assert_reformat(input, expected);
}

#[test]
fn back_relation_fields_and_attribute_must_be_added_even_when_attribute_is_missing() {
    let input = r#"model User {
  id Int @id
  post Post
}

model Post {
  id Int @id
}
"#;

    let expected = r#"model User {
  id     Int  @id
  post   Post @relation(fields: [postId], references: [id])
  postId Int
}

model Post {
  id   Int    @id
  User User[]
}
"#;

    assert_reformat(input, expected);
}

#[test]
fn back_relation_fields_missing_attributes_should_not_add_attributes_multiple_times() {
    let input = r#"model User {
  id Int @id
  post Post
}

model Post {
  id Int @id
}

model Cat {
  id Int @id
  post Post
}
"#;

    let expected = r#"model User {
  id     Int  @id
  post   Post @relation(fields: [postId], references: [id])
  postId Int
}

model Post {
  id   Int    @id
  User User[]
  Cat  Cat[]
}

model Cat {
  id     Int  @id
  post   Post @relation(fields: [postId], references: [id])
  postId Int
}
"#;

    assert_reformat(input, expected);
}

#[test]
#[ignore]
fn back_relations_must_be_added_when_attribute_is_present_with_no_arguments() {
    let input = r#"model User {
  id Int @id
  post Post @relation
}

model Post {
  id Int @id
}
"#;

    let expected = r#"model User {
  id     Int  @id
  post   Post @relation(fields: [postId], references: [id])
  postId Int?
}

model Post {
  id   Int    @id
  User User[]
}
"#;

    assert_reformat(input, expected);
}

#[test]
#[ignore]
fn back_relations_must_be_added_when_attribute_is_present_with_only_one_argument() {
    let input = r#"model User {
  id Int @id
  post Post @relation(fields: [postId])
}

model Post {
  id Int @id
}
"#;

    let expected = r#"model User {
  id     Int  @id
  post   Post @relation(fields: [postId], references: [id])
  postId Int?
}

model Post {
  id   Int    @id
  User User[]
}
"#;

    assert_reformat(input, expected);
}

#[test]
#[ignore]
fn back_relations_must_be_added_when_attribute_is_present_with_both_arguments() {
    let input = r#"model User {
  id Int @id
  post Post @relation(fields: [postId], references: [id])
}

model Post {
  id Int @id
}
"#;

    let expected = r#"model User {
  id     Int  @id
  post   Post @relation(fields: [postId], references: [id])
  postId Int?
}

model Post {
  id   Int    @id
  User User[]
}
"#;

    assert_reformat(input, expected);
}

#[test]
fn scalar_field_and_attribute_must_be_added_even_when_attribute_is_missing_and_both_relation_fields_present() {
    let input = r#"model User {
  id Int @id
  post Post
}

model Post {
  id Int @id
  User User[]
}
"#;

    let expected = r#"model User {
  id     Int  @id
  post   Post @relation(fields: [postId], references: [id])
  postId Int
}

model Post {
  id   Int    @id
  User User[]
}
"#;

    assert_reformat(input, expected);
}

#[test]
fn scalar_field_and_attribute_must_be_added_even_when_attribute_is_missing_and_only_one_relation_field_present() {
    let input = r#"model User {
  id Int @id
}

model Post {
  id Int @id
  User User[]
}

model Cat {
  id Int @id
  post Post
}
"#;

    let expected = r#"model User {
  id     Int   @id
  Post   Post? @relation(fields: [postId], references: [id])
  postId Int?
}

model Post {
  id   Int    @id
  User User[]
  Cat  Cat[]
}

model Cat {
  id     Int  @id
  post   Post @relation(fields: [postId], references: [id])
  postId Int
}
"#;

    assert_reformat(input, expected);
}

#[test]
fn back_relations_must_be_added_even_when_attribute_is_missing_for_one_to_one() {
    let input = r#"model User {
  id     Int   @id
  Post   Post?
}

model Post {
  id   Int    @id
  User User
}
"#;
    let expected = r#"model User {
  id   Int   @id
  Post Post?
}

model Post {
  id     Int  @id
  User   User @relation(fields: [userId], references: [id])
  userId Int
}
"#;
    assert_reformat(input, expected);
}

#[test]
fn back_relations_and_attribute_must_be_added_even_when_attribute_is_missing_for_one_to_many() {
    let input = r#"model User {
  id     Int   @id
  Post   Post
}

model Post {
  id   Int    @id
  User User[]
}
"#;
    let expected = r#"model User {
  id     Int  @id
  Post   Post @relation(fields: [postId], references: [id])
  postId Int
}

model Post {
  id   Int    @id
  User User[]
}
"#;
    assert_reformat(input, expected);
}

#[test]
fn relation_attribute_must_not_be_added_for_many_to_many() {
    let input = r#"model User {
  id   Int    @id
  Post Post[]
}

model Post {
  id   Int    @id
  User User[]
}
"#;
    let expected = input;
    assert_reformat(input, expected);
}

#[test]
fn must_add_relation_attribute_to_an_existing_field() {
    let input = r#"
    model Blog {
      id    Int     @id
      posts Post[]
    }

    model Post {
      id     Int   @id
      Blog   Blog? @relation(fields: [blogId])
      blogId Int?
    }
    "#;

    let expected = r#"model Blog {
  id    Int    @id
  posts Post[]
}

model Post {
  id     Int   @id
  Blog   Blog? @relation(fields: [blogId], references: [id])
  blogId Int?
}
"#;
    assert_reformat(input, expected);
}

#[test]
fn forward_relation_fields_must_be_added() {
    let input = indoc! {r#"
        model PostableEntity {
            id String @id
        }
         
        model Post {
            id        String   @id
            postableEntities PostableEntity[]
        }
"#};

    let expected = indoc! {r#"
         model PostableEntity {
           id     String  @id
           Post   Post?   @relation(fields: [postId], references: [id])
           postId String?
         }
         
         model Post {
           id               String           @id
           postableEntities PostableEntity[]
         }
         
"#};

    assert_reformat(input, expected);
}

#[test]
fn must_add_back_relation_fields_for_given_list_field() {
    let input = indoc! {r#"
    model User {
        id Int @id
        posts Post[]
    }

    model Post {
        post_id Int @id
    }
    "#};

    let expected = indoc! {r#"
    model User {
      id    Int    @id
      posts Post[]
    }

    model Post {
      post_id Int   @id
      User    User? @relation(fields: [userId], references: [id])
      userId  Int?
    }
    "#};

    assert_reformat(input, expected);
}

#[test]
fn must_add_back_relation_fields_for_given_singular_field() {
    let input = indoc! {r#"
    model User {
        id     Int @id
        postId Int     
        post   Post @relation(fields: [postId], references: [post_id]) 
    }

    model Post {
        post_id Int @id
    }
    "#};

    let expected = indoc! {r#"
    model User {
      id     Int  @id
      postId Int
      post   Post @relation(fields: [postId], references: [post_id])
    }
    
    model Post {
      post_id Int    @id
      User    User[]
    }
    "#};

    assert_reformat(input, expected);
}

#[test]
fn must_add_back_relation_fields_for_self_relations() {
    let input = indoc! {r#"
    model Human {
        id    Int @id
        sonId Int?
        son   Human? @relation(fields: [sonId], references: [id]) 
    }
    "#};

    let expected = indoc! {r#"
    model Human {
      id    Int     @id
      sonId Int?
      son   Human?  @relation(fields: [sonId], references: [id])
      Human Human[] @relation("HumanToHuman")
    }
    "#};

    assert_reformat(input, expected);
}

#[test]
fn should_camel_case_back_relation_field_name() {
    let input = indoc! {r#"
    model OhWhatAUser {
        id Int @id
        posts Post[]
    }

    model Post {
        post_id Int @id
    }
    "#};

    let expected = indoc! {r#"
    model OhWhatAUser {
      id    Int    @id
      posts Post[]
    }
    
    model Post {
      post_id       Int          @id
      OhWhatAUser   OhWhatAUser? @relation(fields: [ohWhatAUserId], references: [id])
      ohWhatAUserId Int?
    }
    "#};

    assert_reformat(input, expected);
}

#[test]
//todo I dont like that mother and User field are both the same relation but only one side prints its relationname
fn add_backrelation_for_unambiguous_self_relations_in_presence_of_unrelated_other_relations() {
    let input = indoc! {r#"
        model User {
            id          Int @id
            motherId    Int
            mother      User @relation(fields: motherId, references: id)      
            subscribers Follower[]
        }

        model Follower {
            id        Int   @id
            following User[]
        }
    "#};

    let expected = indoc! {r#"
    model User {
      id          Int        @id
      motherId    Int
      mother      User       @relation(fields: motherId, references: id)
      subscribers Follower[]
      User        User[]     @relation("UserToUser")
    }
    
    model Follower {
      id        Int    @id
      following User[]
    }
    "#};

    assert_reformat(input, expected);
}

#[test]
fn must_succeed_when_fields_argument_is_missing_for_one_to_many() {
    let input = indoc! {r#"
    model User {
        id        Int @id
        firstName String
        posts     Post[]
    }

    model Post {
        id     Int     @id
        userId Int
        user   User    @relation(references: [id])
    }
    "#};

    let expected = indoc! {r#"
    model User {
      id        Int    @id
      firstName String
      posts     Post[]
    }
    
    model Post {
      id     Int  @id
      userId Int
      user   User @relation(references: [id], fields: [userId])
    }
    "#};

    assert_reformat(input, expected);
}

#[test]
fn must_add_referenced_fields_for_one_to_many_relations() {
    let input = indoc! {r#"
    model User {
        user_id Int    @id
        posts   Post[]
    }

    model Post {
        post_id Int    @id
        user    User
    }
    "#};

    let expected = indoc! {r#"
    model User {
      user_id Int    @id
      posts   Post[]
    }
    
    model Post {
      post_id     Int  @id
      user        User @relation(fields: [userUser_id], references: [user_id])
      userUser_id Int
    }
    "#};

    assert_reformat(input, expected);
}

#[test]
fn must_add_referenced_fields_for_one_to_many_relations_reverse() {
    let input = indoc! {r#"
    model User {
      user_id Int    @id
      post    Post
    }
    
    model Post {
      post_id Int    @id
      users   User[]
    }
    "#};

    let expected = indoc! {r#"
    model User {
      user_id     Int  @id
      post        Post @relation(fields: [postPost_id], references: [post_id])
      postPost_id Int
    }
    
    model Post {
      post_id Int    @id
      users   User[]
    }
    "#};

    assert_reformat(input, expected);
}

#[test]
fn must_add_referenced_fields_on_the_right_side_for_one_to_one_relations() {
    // the to fields are always added to model with the lower name in lexicographic order

    let input = indoc! {r#"
    model User1 {
      id         String @id @default(cuid())
      referenceA User2?
    }

    model User2 {
      id         String @id @default(cuid()) 
      referenceB User1?
    }

    model User3 {
      id         String @id @default(cuid()) 
      referenceB User4?
    }

    model User4 {
      id         String @id @default(cuid())
      referenceA User3?
    }
    "#};

    let expected = indoc! {r#"
    model User1 {
      id         String  @id @default(cuid())
      referenceA User2?  @relation(fields: [user2Id], references: [id])
      user2Id    String?
    }
    
    model User2 {
      id         String @id @default(cuid())
      referenceB User1?
    }
    
    model User3 {
      id         String  @id @default(cuid())
      referenceB User4?  @relation(fields: [user4Id], references: [id])
      user4Id    String?
    }
    
    model User4 {
      id         String @id @default(cuid())
      referenceA User3?
    }
    "#};

    assert_reformat(input, expected);
}

#[test]
fn must_handle_conflicts_with_existing_fields_if_types_are_compatible() {
    let input = indoc! {r#"
    model Blog {
      id    String @id
      posts Post[]
    }
    
    model Post {
      id     String   @id      
      blogId String?
    }
    "#};

    let expected = indoc! {r#"
    model Blog {
      id    String @id
      posts Post[]
    }
    
    model Post {
      id     String  @id
      blogId String?
      Blog   Blog?   @relation(fields: [blogId], references: [id])
    }
    "#};

    assert_reformat(input, expected);
}

#[test]
fn must_handle_conflicts_with_existing_fields_if_types_are_incompatible() {
    let input = indoc! {r#"
    model Blog {
      id    String @id
      posts Post[]
    }
    
    model Post {
      id     String   @id      
      blogId Int?     // this is not compatible with Blog.id  
    }
    "#};

    let expected = indoc! {r#"
    model Blog {
      id    String @id
      posts Post[]
    }
    
    model Post {
      id                String  @id
      blogId            Int? // this is not compatible with Blog.id  
      Blog              Blog?   @relation(fields: [blogId_BlogToPost], references: [id])
      blogId_BlogToPost String?
    }
    "#};

    assert_reformat(input, expected);
}

#[test]
fn should_add_back_relations_for_more_complex_cases() {
    let input = indoc! {r#"
    model User {
        id Int @id
        posts Post[]
    }

    model Post {
        post_id Int @id
        comments Comment[]
        categories PostToCategory[]
    }

    model Comment {
        comment_id Int @id
    }

    model Category {
        category_id Int @id
        posts PostToCategory[]
    }

    model PostToCategory {
        id          Int @id
        postId      Int
        categoryId  Int
        
        post     Post     @relation(fields: [postId], references: [post_id])
        category Category @relation(fields: [categoryId], references: [category_id])
        @@map("post_to_category")
    }
    "#};

    let expected = indoc! {r#"
    model User {
      id    Int    @id
      posts Post[]
    }
    
    model Post {
      post_id    Int              @id
      comments   Comment[]
      categories PostToCategory[]
      User       User?            @relation(fields: [userId], references: [id])
      userId     Int?
    }
    
    model Comment {
      comment_id  Int   @id
      Post        Post? @relation(fields: [postPost_id], references: [post_id])
      postPost_id Int?
    }
    
    model Category {
      category_id Int              @id
      posts       PostToCategory[]
    }
    
    model PostToCategory {
      id         Int @id
      postId     Int
      categoryId Int
    
      post     Post     @relation(fields: [postId], references: [post_id])
      category Category @relation(fields: [categoryId], references: [category_id])
      @@map("post_to_category")
    }
    "#};

    assert_reformat(input, expected);
}

#[test]
fn should_add_missing_embed_ids_on_self_relations() {
    let input = indoc! {r#"
    model Human {
        id Int @id
        father Human? @relation("paternity")
        son Human? @relation("paternity")
    }
    "#};

    let expected = indoc! {r#"
    model Human {
      id      Int    @id
      father  Human? @relation("paternity", fields: [humanId], references: [id])
      son     Human? @relation("paternity")
      humanId Int?
    }
    "#};

    assert_reformat(input, expected);
}

#[test]
fn should_add_referenced_fields_on_the_correct_side_list() {
    let input = indoc! {r#"
    model User {
        id Int @id
        post Post[]
    }

    model Post {
        post_id Int @id
        user User
    }
    "#};

    let expected = indoc! {r#"
    model User {
      id   Int    @id
      post Post[]
    }
    
    model Post {
      post_id Int  @id
      user    User @relation(fields: [userId], references: [id])
      userId  Int
    }
    "#};

    assert_reformat(input, expected);
}

#[test]
fn no_changes_for_many_to_many_relations() {
    let input = indoc! {r#"
    model User {
      user_id Int    @id
      posts   Post[]
    }

    model Post {
      post_id Int    @id
      users   User[]
    }
    "#};

    assert_reformat(input, input);
}

#[test]
fn should_add_referenced_fields_on_the_correct_side_tie_breaker() {
    let input = indoc! {r#"
    model User {
        user_id Int @id
        post Post?
    }

    model Post {
        post_id Int @id
        user User?
    }
    "#};

    let expected = indoc! {r#"
    model User {
      user_id Int   @id
      post    Post?
    }
    
    model Post {
      post_id     Int   @id
      user        User? @relation(fields: [userUser_id], references: [user_id])
      userUser_id Int?
    }
    "#};

    assert_reformat(input, expected);
}

#[test]
fn should_not_get_confused_with_complicated_self_relations() {
    let input = indoc! {r#"
    model Human {
      id        Int  @id
      husbandId Int?
      fatherId  Int?
      parentId  Int?
    
      wife    Human? @relation("Marrige")
      husband Human? @relation("Marrige", fields: husbandId, references: id)
    
      father Human? @relation("Paternity", fields: fatherId, references: id)
      son    Human? @relation("Paternity")
    
      children Human[] @relation("Offspring")
      parent   Human?  @relation("Offspring", fields: parentId, references: id)
    }
    "#};

    assert_reformat(input, input);
}

#[test]
fn back_relations_must_be_added_even_when_env_vars_are_missing() {
    // missing env vars led to errors in datamodel validation. A successful validation is prerequisite to find missing back relation fields though.
    // I changed the Reformatter to ignore env var errors.
    let input = r#"
datasource db {
  provider = "sqlite"
  url      = env("DATABASE_URL")
}

model Blog {
  id    Int    @id
  posts Post[]
}

model Post {
  id Int   @id
}
"#;

    let expected = r#"datasource db {
  provider = "sqlite"
  url      = env("DATABASE_URL")
}

model Blog {
  id    Int    @id
  posts Post[]
}

model Post {
  id     Int   @id
  Blog   Blog? @relation(fields: [blogId], references: [id])
  blogId Int?
}
"#;

    let result = datamodel::ast::reformat::Reformatter::new(&input).reformat_to_string();

    assert_eq!(result, expected);
}

//todo formatter failure??
#[test]
fn must_handle_conflicts_with_existing_fields_if_types_are_incompatible_and_name_generation_breaks_down() {
    //just dont add anything
    let input = indoc! {r#"
    model Blog {
      id    String @id
      posts Post[]
    }
    
    model Post {
      id                String @id
      blogId            Int? // this is not compatible with Blog.id
      blogId_BlogToPost Int? // clashes with the auto generated name
    }
    "#};

    let result = datamodel::ast::reformat::Reformatter::new(&input).reformat_to_string();

    assert_eq!(input, result);
}

fn assert_reformat(schema: &str, expected_result: &str) {
    let result = datamodel::ast::reformat::Reformatter::new(&schema).reformat_to_string();
    parse_datamodel(&result).unwrap();
    assert_eq!(result, expected_result);
}
