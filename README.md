# hol

Fetch the source of a single Rust item by name or path, optionally from a git ref.

Point it at a file and an item, and you get back just that item's code — not the
whole file, not a line range you had to guess at.

```rust
use hol::find_item_path_in_file;

let source = find_item_path_in_file("src/lib.rs", "utils::UtilStruct::new")?;
// Some("        pub fn new(u: u8) -> Self {\n            Self {\n                u\n            }\n        }")
```

Git-ref lookups are behind an off-by-default feature:

```toml
[dependencies]
hol = { version = "0.1", features = ["git"] }
```

This is a library; there's no binary target.

## Looking things up

There are multiple ways to identify the item what you want to fetch.

**By name** — searches the whole file, at any nesting depth, and returns the first
exact match. Case sensitive, imports excluded.

```rust
use hol::find_item_in_file;

let source = find_item_in_file("src/lib.rs", "SomeEnum".to_string())?;
```

**By path** — an item path like `utils::UtilStruct::new`, resolved segment by
segment from the top level of the file.

```rust
use hol::find_item_path_in_file;

let source = find_item_path_in_file("src/lib.rs", "utils::UtilStruct::new")?;
```

**Either** — `find_item_in_file_by_name_or_path` tries the path search first and
falls back to the name search if the path doesn't resolve. Handy when the input
comes from a human or a tool that isn't careful about the distinction.

All three return `Ok(None)` when the item isn't in the file. `Err` is reserved for
things that are actually wrong: the file can't be read, the item path can't be
parsed, or the file isn't valid Rust.

## From a git ref

With the `git` feature, the same lookups work against any commit, branch, or tag.

```rust
use hol::hol;

// `None` reads the file on disk; `Some(git_ref)` reads it as of that ref.
let source = hol("src/lib.rs", Some("v0.1.0"), "utils::UtilStruct::new".to_string())?;
```

The repository is discovered from the file path, and the path is resolved relative
to the repo root. `find_item_path_in_file_in_git_ref` and
`find_item_in_file_in_git_by_name_or_path` are also available if you want to pick a
lookup mode explicitly.

## Details worth knowing

**Structs depend on where they sit in the path.** As the last segment,
`utils::UtilStruct` returns the struct definition. As an intermediate segment,
`utils::UtilStruct::new` skips the definition and looks inside the struct's impl
blocks — including across several separate `impl` blocks for the same type, with
unrelated items in between.

**Results start at the beginning of the line.** Attributes and doc comments
attached to the item come along with it, and the original indentation is preserved,
so a nested item comes back indented the way it was written.

**Name search returns one result.** If two items in the file share a name, you get
the first one rust-analyzer's symbol search reports. Use a path when you need to be
precise.

## Tests

```bash
cargo test --all-features
```

The git tests resolve a ref in this repository itself, so they need the `test-changing-func`
branch to be present — a plain `git clone` gets it, a shallow or single-branch one won't.

## License

MIT or Apache-2.0, at your option.
