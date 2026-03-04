mod common;

#[test]
fn placeholder_test() {
    let (_dir, path) = common::test_data_dir();
    assert!(path.exists());
}
