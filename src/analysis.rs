#[cfg(feature = "git")]
pub mod git;

use anyhow::Context;
use ra_ap_ide::{Analysis, FileStructureConfig, Query, StructureNodeKind, SymbolKind};
use std::path::Path;

/// `location < text.len()` must hold
/// For the given `text` and a `location` in the text, find the beginning of the line that location
/// is in.
fn find_line_start(text: &str, location: usize) -> usize {
    debug_assert!(location < text.len());
    text[..location].rfind('\n').map(|i| i + 1).unwrap_or(0)
}

/// Find an `item_path` (e.g. "utils::UtilsStruct::new") in the string `text`.
///
/// Returns `None` if the given path does not exist in the file.
/// Returns `Err` when:
/// - failed to parse the item_path.
/// - failed to parse the text as Rust code.
///
/// Returns `Ok(Some(item_string))` with the code of the searched item, if found.
///
/// ## How structs are handled
///
/// If the path is to a struct, the struct definition, e.g.:
/// ```no_run
///     struct MyStruct {
///         whatever: u8
///     }
/// ```
///
/// will be returned. If the path is to an item inside a struct, that item will be searched in the
/// struct's impl block.
pub fn find_item_path_in_text(text: String, item_path: &str) -> anyhow::Result<Option<String>> {
    // Parse into a syn::Path
    let path: syn::Path = syn::parse_str(item_path).with_context(|| format!("Can't parse given item path: {item_path}"))?;

    let (analysis, file_id) = Analysis::from_single_file(text);
    let file_text = analysis.file_text(file_id)?;
    let config = FileStructureConfig {
        exclude_locals: false,
    };
    let file_structure = analysis.file_structure(&config, file_id).context("Couldn't parse file as rust code")?;
    let mut items = file_structure.iter().enumerate();
    let mut found_item = None;
    let num_segments = path.segments.len();
    let mut parent_stack = Vec::with_capacity(num_segments - 1);
    let mut segment_index = 0;
    'segments: while segment_index < num_segments {
        'items: while let Some((item_index, item)) = items.next() {
            // This is because we can have multiple impl blocks for the same struct.
            // Example:
            // Parent item index | Code
            // ------------------+------
            //                   |
            // None              | mod utils {                              // item index 1
            // Some(1)           |     struct UtilStruct {                  // item index 2
            // Some(2)           |         u: u8,                           // item index 3
            //                   |     }
            //                   |
            // Some(1)           |     impl UtilStruct {                    // item index 4
            // Some(4)           |         pub fn new(u: u8) -> Self {      // item index 5
            //                   |             Self {
            //                   |                 u
            //                   |             }
            //                   |         }
            //                   |     }
            //                   |
            // Some(1)           |     fn util_func() {                     // item index 6
            //                   |         println!("util_func");
            //                   |     }
            //                   |
            // Some(1)           |     impl UtilStruct {                    // item index 7
            // Some(7)           |         pub fn old(&self) -> u8 {        // item index 8
            //                   |             self.u
            //                   |         }
            //                   |     }
            //                   | }
            //
            // If we're searching for "utils::UtilStruct::old", we walk into item 4,
            // which is `impl UtilStruct {...`. Then the top of the parent stack is 4. When we reach
            // item 6, its parent is lesser than 4, so we know we stepped out of 4. Then we pop from
            // the stack, and look for `impl UtilStruct` again, with 1 as a parent.
            while let Some(parent_item_index) = parent_stack.last() && item.parent.is_none_or(|items_parent| items_parent < *parent_item_index) {
                segment_index -= 1;
                parent_stack.pop();
            }
            if parent_stack.last() != item.parent.as_ref() {
                // If the parent of the currently observed tree item is different from the last
                // parent, it can't be the next item we want to walk into.
                // Examples:
                // - `parent_stack.last()` is `None`, `item.parent` is `Some(...)`:
                //   that means we're looking for the item referred to by the first segment of the
                //   `item_path`, which has to be at the top level scope in this file. But if
                //   `item.parent` is `Some(...)` it means it has a parent and is therefore not at
                //   the top level scope.
                // - If they're both `Some`, but with different values - it means we're looking for
                //   a segment that is supposed to be inside an item x, but this item is inside an
                //   item y!=x.
                continue 'items;
            }
            let item_name = match item.kind {
                StructureNodeKind::SymbolKind(SymbolKind::Impl) if segment_index < num_segments - 1 => &file_text[item.navigation_range.start().into()..item.navigation_range.end().into()],
                // If it's the last part of the path, and it's a struct, we show the struct
                // definition. Example: if it's utils::UtilStruct, and we show the
                // `struct UtilStruct { ...` code
                // If it's NOT the last part of the path, and it's a struct, we look for the
                // struct's impl, not the struct definition. Example: if it's utils::UtilStruct::new
                // then we don't choose the struct definition as the next item, but the struct's
                // impl, because that is where `new` is going to be.
                StructureNodeKind::SymbolKind(SymbolKind::Struct) if segment_index < num_segments - 1 => continue 'items,
                _ => item.label.as_str()
            };

            if path.segments[segment_index].ident.to_string() == item_name {
                found_item = Some(item);
                parent_stack.push(item_index);
                segment_index += 1;
                continue 'segments;
            }
        }
        // If we got here, we've looked at all the items in the tree without matching the segment at
        // `segment_index`, so the path does not exist in this file. Note that a *complete* match
        // never reaches this point: matching the last segment increments `segment_index` to
        // `num_segments` and jumps to `'segments`, which then exits through the loop condition.
        // We must not report `found_item` here, as it may hold an item matched by an *earlier*
        // segment - e.g. searching for "utils::nope" would return the whole `mod utils`.
        return Ok(None);
    }
    let Some(item) = found_item else {
        return Ok(None);
    };

    let start = item.node_range.start().into();
    let start = find_line_start(file_text.as_ref(), start);
    let text = &file_text[start..item.node_range.end().into()];

    Ok(Some(text.to_string()))
}

/// Find an `item_path` (e.g. "utils::UtilsStruct::new") in a file at `file_path`.
///
/// Returns `None` if the given path does not exist in the file.
/// Returns `Err` when:
/// - failed to parse the item_path.
/// - failed to read the file at `file_path`.
/// - failed to parse the file as Rust code.
///
/// Returns `Ok(Some(item_string))` with the code of the searched item, if found.
///
/// ## How structs are handled
///
/// If the path is to a struct, the struct definition, e.g.:
/// ```no_run
///     struct MyStruct {
///         whatever: u8
///     }
/// ```
///
/// will be returned. If the path is to an item inside a struct, that item will be searched in the
/// struct's impl block.
pub fn find_item_path_in_file<P: AsRef<Path>>(file_path: P, item_path: &str) -> anyhow::Result<Option<String>> {
    let text = std::fs::read_to_string(file_path).context("Can't read file")?;
    find_item_path_in_text(text, item_path)
}

/// Find an `item` (e.g. "util_func") in `text`. Regardless of its path.
///
/// Returns `Ok(None)` if the item wasn't found in the file.
pub fn find_item_in_text(text: String, item: String) -> anyhow::Result<Option<String>> {
    let (analysis, file_id) = Analysis::from_single_file(text);
    let file_text = analysis.file_text(file_id)?;

    let mut query = Query::new(item);
    query.case_sensitive();
    query.exclude_imports();
    query.exact();
    let symbol_search_result = analysis.symbol_search(query, 1)?;
    let Some(nav_target) = symbol_search_result.first() else {
        return Ok(None);
    };
    let start = nav_target.full_range.start().into();
    let start = find_line_start(file_text.as_ref(), start);
    let text = &file_text[start..nav_target.full_range.end().into()];

    Ok(Some(text.to_string()))
}

/// Find an `item` (e.g. "util_func") in a file at `file_path`. Regardless of its path.
///
/// # Errors
/// When the file contents cannot be read from `path`.
///
/// Returns `Ok(None)` if the item wasn't found in the file.
pub fn find_item_in_file<P: AsRef<Path>>(path: P, item: String) -> anyhow::Result<Option<String>> {
    let text = std::fs::read_to_string(path).context("Can't read file")?;
    find_item_in_text(text, item)
}

/// Take a name or a path of a code item, try to search it as a path, and if no result can be
/// found, search it as a name.
///
pub fn find_item_in_text_by_name_or_path(text: String, item_name_or_path: String) -> anyhow::Result<Option<String>> {
    let found_item_from_path = find_item_path_in_text(text.clone(), &item_name_or_path)?;
    if found_item_from_path.is_none() {
        find_item_in_text(text, item_name_or_path)
    } else {
        Ok(found_item_from_path)
    }
}

///
pub fn find_item_in_file_by_name_or_path<P: AsRef<Path>>(path: P, item_name_or_path: String) -> anyhow::Result<Option<String>> {
    let text = std::fs::read_to_string(path).context("Can't read file")?;
    find_item_in_text_by_name_or_path(text, item_name_or_path)
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_find_line_start() {
        let text = "hello\nworld\nhello";
        assert_eq!(find_line_start(text, 6), 6);
        assert_eq!(find_line_start(text, 7), 6);
        assert_eq!(find_line_start(text, 12), 12);
        assert_eq!(find_line_start(text, 16), 12);
        assert_eq!(find_line_start(text, 3), 0);
        assert_eq!(find_line_start(text, 5), 0);
        assert_eq!(find_line_start(text, 0), 0);
        let text = "hello\r\nworld";
        assert_eq!(find_line_start(text, 7), 7);
        assert_eq!(find_line_start(text, 8), 7);
        assert_eq!(find_line_start(text, 3), 0);
        assert_eq!(find_line_start(text, 5), 0);
        assert_eq!(find_line_start(text, 0), 0);
    }
}
