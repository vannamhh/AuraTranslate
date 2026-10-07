---
title: 'Tách `commands/segment.rs` trước khi thêm lượt ghi segment'
type: 'refactor'
ticket: '16'
created: '2026-10-07'
status: 'in-review'
baseline_revision: 'd6001d8fdf919b2b74cdbad79826e70cc78d4033'
route: 'full'
route_source: 'auto'
risk: 'medium'
review: 'quick'
review_source: 'pinned'
lenses_ran: [quick]
review_loop_iteration: 0
context: []
---

<frozen-after-approval reason="human-owned intent — do not modify unless human renegotiates">

## Intent

**Problem:** `commands/segment.rs` is 4557 lines in one file; 8.9 and 8.13 are about to add segment-write passes to it. The Epic 7 retro debt asks for the AI-6 shape, and the doc comments above `COMMAND_FILE_CENSUS` still say "53 plain / 26 async".

**Approach:** Turn the file into `commands/segment/` the way AI-6 turned `project.rs` into `project/`: `mod.rs` keeps the Rust path `commands::segment` (plus `pub use child::*`), `wire.rs` takes the `#[tauri::command]` shells, domain siblings take the pure functions. Moved text is byte-identical. Fix the stale census doc comments.

## Boundaries & Constraints

**Always:** No IPC command renamed or re-signatured; `commands::segment::*` and `commands::segment::wire::*` paths unchanged for `lib.rs` and every test; moved code byte-identical to the baseline; tests that read the file by name are re-pointed to the meaning they had, never loosened.

**Never:** Move TM commands into `commands/tm.rs` (changes paths and census rows, a separate decision); flip any shell to `(async)`; add a size gate (declined 2026-09-24); touch the webview.

## I/O & Edge-Case Matrix

| Scenario | Input / State | Expected Output / Behavior | Error Handling |
|----------|--------------|---------------------------|----------------|
| Census row moves | `#[tauri::command]` shells now in `segment/wire.rs` | `COMMAND_FILE_CENSUS` row `src/commands/segment/wire.rs` = 17 plain / 2 async / 2 cases | Row count or async count off ⇒ `config_invariants` red |
| Shell loses `(async)` | `tm_fuzzy_matches` plain in `segment/wire.rs` | `the_blocking_wires_run_off_the_main_thread` red naming `segment/wire.rs` | n/a |

</frozen-after-approval>

## Code Map

- `src-tauri/src/commands/segment.rs` (4557) -- no inline tests; items: import insertion `:53-516`, history/restore `:517-932`, open-chapter read + TM prefill `:933-1310`, reading mode + position `:1311-1962`, save/promote/confirm/omit/flush/unconfirm `:1963-3455` (TM glue `:2389-2779` inside), regroup `:3456-4070`, `pub mod wire` `:4071-end`.
- `src-tauri/src/commands/project/{mod,wire,bilingual}.rs` -- the shape to copy: `pub mod wire;` · `mod x; pub use x::*;` · siblings open with `use super::*;`.
- `src-tauri/src/lib.rs` -- registers `commands::segment::wire::*`; untouched.
- Tests that read the file by name: `config_invariants.rs` (cases rows `:1121,1126`, census row `:1781`, `segment.rs` empty-why exemption `:1852`), `segment_boundary.rs` (`split_source_text` 2 sites, `write(move |tx: &Transaction<'_>|`, `&Store` scan), `segment_baseline_guard.rs` (`FLUSH_EXEMPT_FILE`, `rel ==`), `ipc_contract.rs` (`pub mod wire {` slice of the file, twice).

## Tasks & Acceptance

**Execution:**
- [x] `src-tauri/src/commands/segment.rs` -> `segment/{mod,wire,import,history,chapter_read,reading,targets,tm_match,confirm,regroup}.rs` -- move whole line ranges, widen only the visibility a cross-file call needs -- precedent AI-6
- [x] `src-tauri/tests/{config_invariants,segment_boundary,segment_baseline_guard,ipc_contract}.rs` -- re-point file-name reads and the census row -- keep each guard able to fail
- [x] `src-tauri/tests/config_invariants.rs` -- rewrite the two stale "53 plain / 26 async" doc sentences so they carry no tree total -- the table is the only count
- [x] `deferred-work.md` -- close the two `Chủ: Story 8.16` items in words

**Acceptance Criteria:**
- Given the split tree, when `cargo test` runs, then the passed/failed/ignored tuple equals the baseline tuple measured on `d6001d8`.
- Given the moved ranges, when each is diffed against the baseline file, then they are byte-identical.
- Given `COMMAND_FILE_CENSUS`, when a shell in `segment/wire.rs` loses `(async)`, then `config_invariants` goes red naming that file.
- Given `commands/`, when files are counted, then none exceeds the largest file that already exists (`project/mod.rs`) and `segment/*` stay under 1100 lines (largest is 667).

## Implementation Notes

- Shape: AI-6 stage-1 plus domain siblings. `mod.rs` keeps `commands::segment`, `pub mod wire;`, `mod x; pub use x::*;`; siblings open with `use super::*;`. `wire.rs` keeps its 4-space indent (precedent), so ranges diff byte-for-byte.
- Moved ranges are byte-identical to `d6001d8`; the only edits are `pub(super)` on 12 private helpers that now cross files, the dropped `pub mod wire {` wrapper and the `split_source_text` `use`, which moved from `mod.rs` to `import.rs` so the splitter has one file.
- Splitting adds 9 `.rs` files (105 to 114 live), which pushed every `*_FLOOR` population gate under the 80% rule; 21 test files raised 84/88 to 97 and `SRC_TAURI_RS_FLOOR` 148 to 160 as the gate's own message instructs.
- Census row moved to `src/commands/segment/wire.rs` (17/2/2). Counter-check: dropping `(async)` from `tm_fuzzy_matches` turned `the_blocking_wires_run_off_the_main_thread` and the census case red, naming that file.
- Baseline on `d6001d8`: 2182 passed / 0 failed / 27 ignored over 75 `Running` entries; after the split the same tuple.
- TM commands stay divided between `segment/tm_match.rs` and `commands/tm.rs`; merging them changes paths and census rows and is left to Ice.

## Plan Change Log

## Review Triage Log

## Verification

**Commands:**
- `npm run build` then `cargo test --no-run` -- expected: builds
- `cargo test --test config_invariants --test ipc_contract --test segment_boundary --test segment_baseline_guard` -- expected: green
- one full `cargo test` at the end -- expected: baseline tuple
