## 1. Missing CLI module at crate root
- **Problem**: `src/main.rs` declares a `cli` module, but no corresponding `src/cli.rs` or `src/cli/mod.rs` exists, leading to compilation failures.
- **Plan**:
  - Create `src/cli.rs` or `src/cli/mod.rs` implementing `Cli` and `Commands`.
  - Ensure `mod cli;` in `main.rs` points to the new file.

## 2. `bookdb` does not expose `oxidize` submodule
- **Problem**: `src/bookdb/mod.rs` lacks a `pub mod oxidize;` declaration even though an `oxidize` directory exists, hiding its functionality.
- **Plan**:
  - Add `pub mod oxidize;` to `src/bookdb/mod.rs`.
  - Confirm functions like `init_from_cli` compile via `bookdb::oxidize`.

## 3. Mismatch between `flags` module declaration and file name
- **Problem**: `src/bookdb/oxidize/mod.rs` declares `pub mod flags;` but the file is named `flag.rs`, causing a missing module error.
- **Plan**:
  - Rename `flag.rs` to `flags.rs` **or** change the declaration to `pub mod flag;`.
  - Update all `use` statements to match the chosen naming.

## 4. API module declarations out of sync with files
- **Problem**: `src/bookdb/service/api/mod.rs` declares a `delv` module and omits existing files like `reset.rs`, `stats.rs`, `validate.rs`, and `migrate.rs`.
- **Plan**:
  - Add `src/bookdb/service/api/delv.rs` if the command exists, otherwise remove `pub mod delv;`.
  - Declare and export modules for `reset`, `stats`, `validate`, and `migrate` to mirror existing files.

## 5. Database module references missing submodules
- **Problem**: `src/bookdb/service/db/mod.rs` references `core`, `dbutils`, `project`, `workspace`, `keystore`, `docstore`, and `multibase`, none of which exist.
- **Plan**:
  - Implement the missing modules under `src/bookdb/service/db/`.
  - Remove obsolete `mod` statements and `pub use` exports if those components are no longer planned.

## 6. Context type module naming inconsistencies
- **Problem**: `src/bookdb/service/ctx/mod.rs` expects `types::segments` while the directory contains `segment.rs`; `types/mod.rs` is empty, causing unresolved imports.
- **Plan**:
  - Rename `segment.rs` to `segments.rs` **or** update `ctx/mod.rs` to `pub mod segment;`.
  - Populate `types/mod.rs` with `segments`, `typesV1`, and `typesV3` to consolidate context type definitions.

## 7. App module typos and missing submodules
- **Problem**: `src/bookdb/app/mod.rs` exports `hanlders` (typo) and `app/ctrl/dispatch.rs` references submodules (`error`, `sql`, `db`, `ctx`, `api`, `cli`) that do not exist.
- **Plan**:
  - Fix the export typo to `handlers` in `app/mod.rs`.
  - Add missing files under `src/bookdb/app/ctrl/` for each referenced submodule or remove their declarations if deprecated.

## 8. Build script misconfiguration affects `include!` macros
- **Problem**: The build script is disabled (`___build.rs`) and `Cargo.toml` lacks a `build` entry, preventing generation of `sql_consts.rs` required by `include_sql_mod!()`.
- **Plan**:
  - Rename `___build.rs` to `build.rs` or reference it via `build = "___build.rs"` in `Cargo.toml`.
  - Verify `include_sql_mod!()` reads the generated `OUT_DIR/sql_consts.rs` during builds.