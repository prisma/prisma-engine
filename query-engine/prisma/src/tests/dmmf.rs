use prisma_models::DatamodelConverter;
use query_core::{BuildMode, QuerySchema, QuerySchemaBuilder, SupportedCapabilities};
use serial_test::serial;
use std::sync::Arc;

// Tests in this file run serially because the function `get_query_schema` depends on setting an env var.

#[test]
#[serial]
fn dmmf_create_inputs_without_fields_for_parent_records_are_correct() {
    let dm = r#"
        model Blog {
            blogId String @id
            postsField Post[]
        }

        model Post {
            postId String @id
            blogField Blog?
            tagsField Tag[]
        }

        model Tag {
            tagId String @id
            postsField Post[]
        }
    "#;

    let (query_schema, datamodel) = get_query_schema(dm);

    let dmmf = crate::dmmf::render_dmmf(&datamodel, Arc::new(query_schema));

    let inputs = &dmmf.schema.input_types;

    let create_post_from_blog = inputs
        .iter()
        .find(|input| input.name == "PostCreateWithoutBlogFieldInput")
        .expect("finding PostCreateWithoutBlogFieldInput");

    let create_post_from_blog_fields: Vec<(&str, &str)> = create_post_from_blog
        .fields
        .iter()
        .map(|f| (f.name.as_str(), f.input_type.typ.as_str()))
        .collect();

    assert_eq!(
        create_post_from_blog_fields,
        &[
            ("postId", "String"),
            ("tagsField", "TagCreateManyWithoutPostsFieldInput")
        ]
    );

    let create_post_from_tags = inputs
        .iter()
        .find(|input| input.name == "PostCreateWithoutTagsFieldInput")
        .expect("finding PostCreateWithoutTagsFieldInput");

    let create_post_from_tags_fields: Vec<(&str, &str)> = create_post_from_tags
        .fields
        .iter()
        .map(|f| (f.name.as_str(), f.input_type.typ.as_str()))
        .collect();

    assert_eq!(
        create_post_from_tags_fields,
        &[
            ("postId", "String"),
            ("blogField", "BlogCreateOneWithoutPostsFieldInput")
        ]
    );
}

#[test]
#[serial]
fn must_not_fail_on_missing_env_vars_in_a_datasource() {
    let dm = r#"
        datasource pg {
            provider = "postgresql"
            url = env("MISSING_ENV_VAR")
        }
        
        model Blog {
            blogId String @id
        }
    "#;
    let (query_schema, datamodel) = get_query_schema(dm);

    let dmmf = crate::dmmf::render_dmmf(&datamodel, Arc::new(query_schema));

    let inputs = &dmmf.schema.input_types;

    inputs
        .iter()
        .find(|input| input.name == "BlogCreateInput")
        .expect("finding BlogCreateInput");
}

fn get_query_schema(datamodel_string: &str) -> (QuerySchema, datamodel::dml::Datamodel) {
    let dm = datamodel::parse_datamodel_and_ignore_env_errors(datamodel_string).unwrap();
    let internal_dm_template = DatamodelConverter::convert(&dm);
    let internal_ref = internal_dm_template.build("db".to_owned());
    let supported_capabilities = SupportedCapabilities::empty();
    (
        QuerySchemaBuilder::new(&internal_ref, &supported_capabilities, BuildMode::Modern, false).build(),
        dm,
    )
}
