extern crate datamodel;

use pretty_assertions::assert_eq;
use std::fs;

#[test]
fn test_dmmf_rendering() {
    let test_cases = vec![
        "general",
        "functions",
        "source",
        "source_with_comments",
        "source_with_generator",
        "without_relation_name",
    ];

    for test_case in test_cases {
        println!("TESTING: {}", test_case);
        let datamodel_string = load_from_file(format!("{}.prisma", test_case).as_str());
        let dml = datamodel::parse_datamodel(&datamodel_string).unwrap();
        let dmmf_string = datamodel::json::dmmf::render_to_dmmf(&dml);
        assert_eq!(dmmf_string, load_from_file(format!("{}.txt", test_case).as_str()));
    }
}

fn load_from_file(file: &str) -> String {
    let server_root = std::env::var("SERVER_ROOT").expect("Env var SERVER_ROOT required but not found.");
    let samples_folder_path = format!("{}/libs/datamodel/core/tests/render_to_dmmf/files", server_root);
    fs::read_to_string(format!("{}/{}", samples_folder_path, file)).unwrap()
}
