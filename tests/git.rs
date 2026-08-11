#![cfg(feature = "git")]

use hol::find_item_path_in_file_in_git_ref;
use std::path::PathBuf;

#[test]
fn find_struct_method_in_mod_in_file_in_branch() {
    let root_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let res = find_item_path_in_file_in_git_ref(root_dir.join("test_samples/my-lib/src/lib.rs"), "utils::UtilStruct::new", "test-changing-func").unwrap().unwrap();
    println!("res:\n{res}");
    assert_eq!(
        res,
        "        pub fn new(u: u8) -> Self {
            Self {
                u
            }
        }"
    );
}
