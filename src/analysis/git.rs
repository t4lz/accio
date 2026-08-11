use crate::analysis::{find_item_in_text_by_name_or_path, find_item_path_in_text};
use anyhow::Context;
use git2::Repository;
use std::io::Read;
use std::path::Path;

/// Get file contexts of the file at path `file_path` in the git ref (branch name, commit name, tag,
/// etc.) `ref_name`.
fn get_file_text_from_git<P: AsRef<Path>>(
    ref_name: &str,
    file_path: P,
) -> anyhow::Result<String> {
    let repo = Repository::discover(&file_path).context("Can't find git repo that contains this path.")?;

    let obj = repo.revparse_single(ref_name).context("Can't find given git ref.")?;
    let commit = obj.peel_to_commit()?;
    let tree = commit.tree()?;
    let relative_path = file_path.as_ref().strip_prefix(repo.workdir().context("Can't find git repo workdir.")?).unwrap_or(file_path.as_ref());
    let tree_entry = tree.get_path(relative_path)?;
    let blob = repo.find_blob(tree_entry.id())?;

    let mut res = String::with_capacity(blob.size());
    // Return the file contents
    blob.content().read_to_string(&mut res)?;
    Ok(res)
}

/// like [`find_item_path_in_text`], but at the `ref_name` ref instead of necessarily the current
/// file at that location.
pub fn find_item_path_in_file_in_git_ref<P: AsRef<Path>>(file_path: P, item_path: &str, ref_name: &str) -> anyhow::Result<Option<String>> {
    let text = get_file_text_from_git(ref_name, file_path)?;
    find_item_path_in_text(text, item_path)
}

/// like [`find_item_in_text_by_name_or_path`], but at the `ref_name` ref instead of necessarily the
/// current file at that location.
pub fn find_item_in_file_in_git_by_name_or_path<P: AsRef<Path>>(file_path: P, item_name_or_path: String, ref_name: &str) -> anyhow::Result<Option<String>> {
    let text = get_file_text_from_git(ref_name, file_path)?;
    find_item_in_text_by_name_or_path(text, item_name_or_path)
}
