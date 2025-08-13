# DEFECTS

This document records current module and build issues observed in the repository and suggests a plan for remediation.

## 1. Missing CLI module at crate root
- **Problem**: `src/main.rs` declares a `cli` module, but no corresponding `src/cli.rs` or `src/cli/mod.rs` exists, leading to compilation failures.
- **Plan**: Implement a CLI module (`src/cli.rs` or directory with `mod.rs`) defining `Cli` and `Commands`, then ensure `mod cli;` in `main.rs` points to it.

## 2. `bookdb` does not expose `oxidize` submodule
- **Problem**: `src/bookdb/mod.rs` lacks a `pub mod oxidize;` declaration even though an `oxidize` directory exists, hiding its functionality.
- **Plan**: Add `pub mod oxidize;` to `src/bookdb/mod.rs` so `bookdb::oxidize` functions become accessible.

## 3. Mismatch between `flags` module declaration and file name
- **Problem**: `src/bookdb/oxidize/mod.rs` declares `pub mod flags;` but the file is named `flag.rs`, causing a missing module error.
- **Plan**: Rename `flag.rs` to `flags.rs` or update the declaration to `pub mod flag;`, and adjust any `use` statements accordingly.

## 4. API module declarations out of sync with files
- **Problem**: `src/bookdb/service/api/mod.rs` declares a `delv` module and omits existing files like `reset.rs`, `stats.rs`, `validate.rs`, and `migrate.rs`.
- **Plan**: Create `delv.rs` if needed or remove its declaration, then add module declarations for `reset`, `stats`, `validate`, and `migrate` to match existing files.

## 5. Database module references missing submodules
- **Problem**: `src/bookdb/service/db/mod.rs` references `core`, `dbutils`, `project`, `workspace`, `keystore`, `docstore`, and `multibase`, none of which exist.
- **Plan**: Implement these missing modules or remove their `mod` statements and related exports if obsolete.

## 6. Context type module naming inconsistencies
- **Problem**: `src/bookdb/service/ctx/mod.rs` expects `types::segments` while the directory contains `segment.rs`; `types/mod.rs` is empty, causing unresolved imports.
- **Plan**: Rename `segment.rs` to `segments.rs` or update `ctx/mod.rs` to `pub mod segment;`, and populate `types/mod.rs` with the required submodules.

## 7. App module typos and missing submodules
- **Problem**: `src/bookdb/app/mod.rs` exports `hanlders` (typo) and `app/ctrl/dispatch.rs` references submodules (`error`, `sql`, `db`, `ctx`, `api`, `cli`) that do not exist.
- **Plan**: Correct the typo to `handlers` and add or remove the referenced submodules in `app/ctrl` to match the intended design.

## 8. Build script misconfiguration affects `include!` macros
- **Problem**: The build script is disabled (`___build.rs`) and `Cargo.toml` lacks a `build` entry, preventing generation of `sql_consts.rs` required by `include_sql_mod!()`.
- **Plan**: Re-enable the build script by renaming `___build.rs` to `build.rs` or specifying it in `Cargo.toml`, ensuring `include_sql_mod!()` can access the generated files without errors.

---
These issues prevent successful compilation and should be addressed before further development.
