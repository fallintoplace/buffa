use super::*;

#[test]
fn indexed_file_selection_preserves_descriptor_order_and_duplicates() {
    let descriptors: Vec<FileDescriptorProto> = (0..72)
        .map(|i| proto3_file(&format!("file_{i}.proto")))
        .collect();
    let mut selected: Vec<String> = (0..72).rev().map(|i| format!("file_{i}.proto")).collect();
    selected.push("file_42.proto".to_string());
    selected.push("absent.proto".to_string());

    let found: Vec<String> = crate::generated_files(&descriptors, &selected)
        .map(|(file, _)| file.name.clone().unwrap())
        .collect();
    let expected: Vec<String> = (0..72).map(|i| format!("file_{i}.proto")).collect();
    assert_eq!(found, expected);

    let small: Vec<String> = selected[..3].to_vec();
    let found_small: Vec<String> = crate::generated_files(&descriptors, &small)
        .map(|(file, _)| file.name.clone().unwrap())
        .collect();
    assert_eq!(
        found_small,
        vec!["file_69.proto", "file_70.proto", "file_71.proto"]
    );
}

#[test]
fn indexed_lookup_keeps_first_duplicate_descriptor_and_rejects_missing_names() {
    let mut descriptors: Vec<FileDescriptorProto> = (0..40)
        .map(|i| proto3_file(&format!("file_{i}.proto")))
        .collect();
    let mut duplicate = proto3_file("file_0.proto");
    duplicate.package = Some("later_duplicate".to_string());
    descriptors.push(duplicate);

    let names: Vec<String> = (0..40).map(|i| format!("file_{i}.proto")).collect();
    assert!(crate::generate(&descriptors, &names, &CodeGenConfig::default()).is_ok());

    let mut missing = names;
    missing.push("not-present.proto".to_string());
    assert!(matches!(
        crate::generate(&descriptors, &missing, &CodeGenConfig::default()),
        Err(CodeGenError::FileNotFound(name)) if name == "not-present.proto"
    ));
}
