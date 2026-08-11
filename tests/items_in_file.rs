use accio::{find_item_in_file, find_item_in_file_by_name_or_path, find_item_path_in_file};
use indoc::indoc;
use std::path::PathBuf;

fn sample_lib() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("test_samples/my-lib/src/lib.rs")
}

#[test]
fn find_enum_in_file() {
    let root_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let res = find_item_in_file(root_dir.join("test_samples/my-lib/src/lib.rs"), "SomeEnum".to_string()).unwrap().unwrap();
    println!("res:\n{res}");
    assert_eq!(
        res,
        indoc! {"
            enum SomeEnum {
                Variant1,
                Variant2(bool),
                Variant3 { x: u64, y: u64 },
            }"
        }
    );
}

#[test]
fn find_fn_in_file() {
    let root_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let res = find_item_in_file(root_dir.join("test_samples/my-lib/src/lib.rs"), "add".to_string()).unwrap().unwrap();
    println!("res:\n{res}");
    assert_eq!(
        res,
        indoc! {"
            pub fn add(left: u64, right: u64) -> u64 {
                left + right
            }"
        }
    );
}

#[test]
fn find_struct_in_file() {
    let root_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let res = find_item_in_file(root_dir.join("test_samples/my-lib/src/lib.rs"), "UtilStruct".to_string()).unwrap().unwrap();
    println!("res:\n{res}");
    assert_eq!(
        res,
        "    #[derive(Debug)]
    struct UtilStruct {
        u: u8,
    }"
    );
}

#[test]
fn find_struct_path_in_mod_in_file() {
    let root_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let res = find_item_path_in_file(root_dir.join("test_samples/my-lib/src/lib.rs"), "utils::UtilStruct").unwrap().unwrap();
    println!("res:\n{res}");
    assert_eq!(
        res,
        "    #[derive(Debug)]
    struct UtilStruct {
        u: u8,
    }"
    );
}

#[test]
fn find_struct_method_in_mod_in_file() {
    let root_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let res = find_item_path_in_file(root_dir.join("test_samples/my-lib/src/lib.rs"), "utils::UtilStruct::new").unwrap().unwrap();
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

#[test]
fn find_struct_method_in_second_impl_block_in_mod_in_file() {
    let root_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let res = find_item_path_in_file(root_dir.join("test_samples/my-lib/src/lib.rs"), "utils::UtilStruct::old").unwrap().unwrap();
    println!("res:\n{res}");
    assert_eq!(
        res,
        "        pub fn old(&self) -> u8 {
            self.u
        }"
    );
}

/// A path whose last segment doesn't exist must not fall back to reporting the ancestor that *did*
/// match (here, the whole `mod utils`).
#[test]
fn missing_item_in_existing_mod_is_not_found() {
    assert_eq!(find_item_path_in_file(sample_lib(), "utils::nope").unwrap(), None);
}

/// Same, one level deeper: the matched ancestor is an impl block rather than a module.
#[test]
fn missing_method_in_existing_struct_is_not_found() {
    assert_eq!(
        find_item_path_in_file(sample_lib(), "utils::UtilStruct::does_not_exist").unwrap(),
        None
    );
}

#[test]
fn missing_top_level_item_is_not_found() {
    assert_eq!(find_item_path_in_file(sample_lib(), "nope").unwrap(), None);
}

/// `old` is nested inside an impl block, so it doesn't resolve as the single-segment path "old".
/// The path search has to report "not found" for the name search to get its turn and find it.
#[test]
fn falls_back_to_name_search_when_path_does_not_resolve() {
    let res = find_item_in_file_by_name_or_path(sample_lib(), "old".to_string()).unwrap().unwrap();
    println!("res:\n{res}");
    assert_eq!(
        res,
        "        pub fn old(&self) -> u8 {
            self.u
        }"
    );
}
