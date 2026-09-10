<!-- bmad:context -->
<!-- Verified 2026-09-10 against 39ae75d. Managed by bmad-project-context; edits inside this block are replaced on refresh. -->

## tools/dict-build/ — the dictionary builder

An INDEPENDENT Rust workspace, not a member of `src-tauri` and with no parent workspace. `rust-version 1.97.1` diverges from `src-tauri` (1.85) deliberately — do not sync the two numbers. Raw input in `docs/dics/`.

## Conventions that differ from defaults

- 🔴 `dict-tran-van-chanh.db` must stay a SEPARATE `.db` file. Trần Văn Chánh (1999) is still in copyright — the author is alive; the digitiser's CC0 grant cannot erase copyright in the underlying work. This layer ships detached precisely because of that risk: FR112 is enforceable by deleting exactly one file. Do not merge layers, do not "consolidate for tidiness", do not fold its data into `dict-core.db`. (`src-tauri/tests/dict_sources.rs::deleting_any_detachable_layer_keeps_the_whole_lookup_suite_green` guards that removing a layer stays green — it does NOT guard against someone merging layers.)
- 🔴 A schema change ⇒ rebuild ALL FOUR `.db` files with `--layer all` ⇒ four new SHA-256s in `dict-manifest.toml` ⇒ a new release. True even when a layer's raw input has not changed by a byte. The source of truth is `dict-manifest.toml` + `src/schema.rs`, NOT `README.md` — the README still says `--layer all` builds *"exactly three"* files and `src/build.rs::run_all` still says *"two detachable layers"*, both predating the addition of `tran-van-chanh`.
- Three mandatory fields per manifest entry: `url` · `sha256` · `source_version` — `source_version` is the version of the RAW INPUT, not of the `.db` file. Never fill a placeholder value "just to have one".
- `is_han` has two DELIBERATE copies (`src/char_idx.rs` and `src-tauri/src/core/dict/mod.rs`) because the two workspaces cannot import across each other. Two gates guard it: `dict_lookup.rs::han_ranges_are_verbatim_from_dict_build_char_idx` reads this file as text and compares the CJK ranges, and `dict_boundary.rs::exactly_one_definition_of_is_han_exists_under_src_tauri`. Fix one side and forget the other and a lookup hits a `char_idx` that never indexed that character ⇒ empty, no error.
- Strings here carry a NAMED exemption from `check:i18n` Check A, so accented Vietnamese is fine.
- `// dict-build:allow <token> — <reason>` on the line above a violation is read by `check:dict`; the em dash and a non-empty reason are both required by `ALLOW_RE`. These are code, not prose — never strip them.

## Known pitfalls

- `npm run check:dict-manifest` checks SHAPE only — it has to stay green on a runner holding no dictionary bytes, so it never opens a `.db` file. It catches a dropped layer (it demands exactly 3 `[[detachable]]` entries, by name); it does NOT catch data mixed between files.

<!-- /bmad:context -->
