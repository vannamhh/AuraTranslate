<!-- bmad:context -->
<!-- Condensed 2026-09-23 (Ice): rules only; history in `_bmad-output/implementation-artifacts/agent-rules-evidence.md`. Managed by bmad-project-context; edits inside this block are replaced on refresh. -->

## tools/dict-build/ — the dictionary builder

An independent Rust workspace (not a member of `src-tauri`). `rust-version 1.97.1`, deliberately different from `src-tauri` — don't sync. Raw input in `docs/dics/`.

## Conventions

- 🔴 `dict-tran-van-chanh.db` stays a separate file (still in copyright; FR112 is enforced by deleting exactly one file). Never merge or consolidate layers.
- 🔴 A schema change ⇒ rebuild all four `.db` files with `--layer all` ⇒ four new SHA-256s in `dict-manifest.toml` ⇒ a new release. Source of truth: `dict-manifest.toml` + `src/schema.rs`, not `README.md`.
- Every manifest entry has `url` · `sha256` · `source_version` (version of the raw input). Never a placeholder.
- `is_han` has two deliberate copies (`src/char_idx.rs`, `src-tauri/src/core/dict/mod.rs`); change both together.
- Strings here are exempt from `check:i18n` Check A.
- `// dict-build:allow <token> — <reason>` is read by `check:dict`; never strip it.

## Known pitfalls

- `check:dict-manifest` checks shape only; it never opens a `.db` and does not catch data mixed between files.

<!-- /bmad:context -->
