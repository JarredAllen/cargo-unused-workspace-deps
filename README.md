Check for Unused Workspace Dependencies
---

Rust lets you list dependencies common to multiple crates in a workspace in the workspace-level
`Cargo.toml` file, and then each crate can include them via including `<dep>.workspace = true` in
the crate-level `Cargo.toml` file.

In large projects, it's easy to remove the last use of a dependency from a crate without noticing,
and thus accumulate unused dependencies in that list. Enter `cargo-unused-workspace-dependencies`!
This tool can check for any dependencies which aren't in use and, optionally, remove them from the
workspace.
