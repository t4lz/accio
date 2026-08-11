mod analysis;

#[cfg(feature = "git")]
pub use analysis::git::{find_item_in_file_in_git_by_name_or_path, find_item_path_in_file_in_git_ref};
pub use analysis::{find_item_in_file, find_item_in_file_by_name_or_path, find_item_path_in_file};
#[cfg(feature = "git")]
use std::path::Path;

#[cfg(feature = "git")]
/// Find a code item by name (e.g. `new`) or item path (e.g. `MyStruct::new`), in the rust file at
/// `path`, optionally in `git_ref` (a commit/branch/tag/etc.).
pub fn hol<P: AsRef<Path>>(path: P, git_ref: Option<&str>, item_name_or_path: String) -> anyhow::Result<Option<String>> {
    match git_ref {
        None => find_item_in_file_by_name_or_path(path, item_name_or_path),
        Some(git_ref) => find_item_in_file_in_git_by_name_or_path(path, item_name_or_path, git_ref)
    }
}
