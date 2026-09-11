---
title: 'Story 6.16 — Import a two-column bilingual CSV/TSV'
type: 'feature'
created: '2026-09-11'
status: 'done' # draft | ready-for-dev | in-progress | in-review | done
route: 'dispatch' # oneshot | dispatch
baseline_commit: 'c145d3fda60a682f51131c20ec389063730d5869'
review_loop_iteration: 0
context:
  - '{project-root}/AGENTS.md'
  - '{project-root}/src-tauri/AGENTS.md'
  - '{project-root}/src/AGENTS.md'
---

<frozen-after-approval reason="human-owned intent — do not modify unless human renegotiates">

## Intent

**Problem:** FR115 — a translation someone else made has no way into the app without losing the
target text: every import path is source-only (`ImportedChapter`, `SplitSegment` carry no
target), `insert_segments` never writes `target_text`, and `TRANSLATION_ORIGIN_BILINGUAL_IMPORT`
(`commands/segment.rs:2000`) is declared but written by nothing.

**Approach:** A bilingual input shape on the ONE AD-39 pipeline. The user explicitly picks
bilingual mode, declares source column, target column, source language and whether row 1 is a
header; rows travel decode → cleanup → normalize → chapter split on the SOURCE column → preview
→ sentence split on both sides per row → `create_work`, which writes target text with origin
`bilingual_import`, status `draft`, Chapters `in_progress`.

### Decisions (Ice, 2026-09-11)

- **Scope: `.csv` and `.tsv` only.** `.md` tables and `.docx` tables are deferred with owner
  Ice (`deferred-work.md`, two `source_spec` entries of this spec).
- **Mismatched row ⇒ confirm is locked.** A row whose two cells split into different sentence
  counts (including 0 vs n) is listed in preview with Chapter + file row number, and confirm stays
  disabled while any exists. Equal-count rows pair in order. No join/split UI (Story 6.17).
- **Header row: a "first row is column headers" checkbox, default OFF.** On ⇒ row 1 is dropped
  before the chapter split; toggling rebuilds the preview in memory. No header guessing.
- **`deferred-work.md:4372-4382` is NOT taken.** It is re-owned to Ice with a new entry saying
  the path is reachable from this story on; no Editor-side change here.

## Boundaries & Constraints

**Always:**
- 🔴 AD-39: `PIPELINE_ORDER` unchanged; `run_import` keeps its single product call site. Table
  parsing happens right after decode, inside the chain, and re-runs on every encoding candidate.
- 🔴 AD-4/FR23: a segment is a sentence; boundaries on both sides computed once, at import.
- 🔴 AD-37/AD-46: last segment of each row ⇒ source flag on, target flag mirrors; other segments
  off; last segment of a Chapter ⇒ both off. Line breaks inside a cell never set a flag.
- 🔴 AD-47 ③: each segment's `target_text` and `translation_origin = bilingual_import` are
  written in the same INSERT. `status = 'draft'`. Chapters `LifecycleStatus::InProgress` via
  `as_str()` — on this path only.
- Cleanup and normalize run per cell, both columns, never across rows.
- Pattern is tested against each row's source cell; a matching row starts a Chapter, its trimmed
  source cell is the title, and the row stays as segments (same as the Blob path keeps the
  heading line). Rows before the first match form a leading Chapter; no pattern ⇒ one Chapter.
- Delimiter from the extension (`.csv` comma, `.tsv` tab). A row with both chosen cells empty
  yields no segment.
- 0 bytes on disk before confirm; preview shows rows · Chapters · pairs · mismatched rows.
- The prose import path (`.txt`/`.md`/`.docx`, `SUPPORTED_EXTENSIONS`) stays byte-identical;
  `.csv`/`.tsv` are accepted only in bilingual mode.

**Never:**
- No new crate: reuse the RFC 4180 tokenizer in `core/glossary/exchange.rs` (NFR15 precedent), no
  second copy.
- No migration (schema stays v22; `pinned_contract.rs` 21/22 untouched).
- No join/split UI, no "import now, join later" (AD-5 §Rule).
- No `tauri.conf.json` / `capabilities/**` / CSP change, no new permission.
- No identifier containing `Document`/`Project`/`Book`/`Novel`; no bare `origin`.
- Do not edit `epics.md`/`prd.md` to match the code.

## I/O & Edge-Case Matrix

| Scenario | Input / State | Expected Output / Behavior | Error Handling |
|---|---|---|---|
| Pattern matches 3 rows | CSV, 3 heading rows in source column | Preview 3 Chapters; confirm ⇒ 1 Work, 3 Chapters `in_progress` | N/A |
| Equal counts | Source 2 sentences, target 2 | 2 segments with both texts, `bilingual_import`, `draft` | N/A |
| Paragraph flags | 3 sentences each side, row not last | Flags off, off, on (both); Chapter's last segment off | N/A |
| Mismatched row | Source 2 sentences, target 1 | Row listed (Chapter, row no.); confirm disabled | N/A |
| Blank target cell | Heading row, target empty | Listed as mismatch (1 vs 0) | N/A |
| Header checkbox on | Row 1 `中文,Tiếng Việt` | Row 1 absent from segments and counts | N/A |
| Swap columns | User swaps roles | Pattern runs on the other column; counts rebuild | N/A |
| Encoding change | Low confidence, other candidate picked | Re-decoded, re-parsed; roles and header choice kept | N/A |
| Unterminated quote | `"abc` to EOF | Refused with the row number; 0 Work | Typed error |
| Fewer than 2 columns | One-column TSV | Refused before preview | Typed error |
| Cancel | Declared, then cancel | 0 Work; override state cleared | N/A |
| Prose `.md` | Existing non-bilingual flow | Byte-identical to today | N/A |

</frozen-after-approval>

## Code Map

**Reader**
- `src-tauri/src/core/glossary/exchange.rs:564` `split_first_logical_line`, `:645` `logical_lines`
  (unterminated-quote error with line no.), `:670` `split_fields` — private; make `pub(crate)`,
  no body change. Story 3.10 AC5 greps this file: no fs/path/desktop-framework tokens.

**Pipeline** — `src-tauri/src/core/segment/pipeline.rs`
- `:121-129` `PIPELINE_ORDER` (unchanged); `:193` `PipelineShape` (`:195` doc already names the
  bilingual Blob) — add the variant; `:206` `PipelineInput`; `:412-492` `Flow`; `:791` `run_import`.
- `:820-822` `decode_unit`; `:639` cleanup; `:694-715` normalize joins lines — per cell only.
- `:990` `split_chapters_step`, `:1107` `split_on_positions`, `:1143` `title_line_of`.
- `:1205` `split_segments_step` → `split.rs:225` `split_source_text(source_text, source_lang)`
  (only branches on Chinese); target side passes the Vietnamese code — target language is fixed
  (`prd.md:18`). `split.rs:445` `mark_paragraph_end` is content-based — not used for row flags.
- `segment_encoding_boundary.rs:150`: only `encoding.rs` names `chardetng`.

**Import + write**
- `src-tauri/src/core/segment/import.rs:67` `SUPPORTED_EXTENSIONS`, `:456` `ImportedChapter`,
  `:555` `import_file`.
- `src-tauri/src/commands/project.rs:352` `create_work`, `:693-701` chapter INSERT (`NotStarted`),
  `:2904` `preview_import_encoding`, `:2624` `cleanup_and_chapters_preview_for`, `:3218` stash,
  `:3246` cancel, `:3274` confirm, `:3398`/`:3478` override states, `mod wire` `:5441/:5493/:5546`.
  Column roles + header travel the way `chapter_pattern` does — no third override state.
- `src-tauri/src/commands/segment.rs:100-163` `insert_segments` (`:158` origin `''`); `:1960-2018`
  constants. `core/lifecycle/mod.rs:83-96`; `commands/chapter.rs:850-859` no-literal rule.
- Reindex after create already reads `target_text` (`project.rs:5314`, `indexer.rs:1428-1507`).

**Webview**
- `src/modes/LibraryMode.vue:1231` language select, `:1273` path input; `src/modes/libraryImport.ts:354`.
- `src/ImportPreviewOverlay.vue:916` tier 1, `:320-331` pattern draft, `:843` section precedent;
  `src/importPreviewState.ts:691`; `src/config/project.ts:325`, `:579/:592`.
- Mockup `ux-designs/ux-AuraTranslate-2026-08-02/mockups/bilingual-import.html` §① (not its
  "Nhập luôn, nối sau" button).

**Existing tests that move** — `review_contract.rs:13`, `docx_contract.rs:186`,
`webimport_contract.rs:1639` (exhaustive shape match); `segment_contract.rs:9169` (preview wire);
`ipc_contract.rs:966` (params), `:915` (every preview shell resets both override states);
`segment_contract.rs:5767` (narrow to non-bilingual paths, keep); `cleanup_contract.rs:1450`.

## Tasks & Acceptance

**Execution:**
- [x] `src-tauri/src/core/glossary/exchange.rs` -- `pub(crate)` on the three tokenizer fns -- one RFC 4180 reader.
- [x] `src-tauri/src/core/segment/bilingual.rs` (new) -- pure `&str` → rows of cells; typed errors with row numbers; 0 panic points, 0 fs/network -- input step.
- [x] `src-tauri/src/core/segment/pipeline.rs` -- bilingual shape; `Flow` carries rows; per-cell cleanup/normalize; per-row pattern; both-side split; row flags; mismatch list -- AD-39/4/37.
- [x] `src-tauri/src/core/segment/import.rs` -- `ImportedChapter` carries per-segment target + mismatches; bilingual entry.
- [x] `src-tauri/src/commands/segment.rs` -- insert path writing `target_text` + `bilingual_import`; existing path unchanged.
- [x] `src-tauri/src/commands/project.rs` -- bilingual preview/confirm on the existing stash flow; confirm refused while mismatches > 0 (Rust-side, not only UI); `InProgress` on this path.
- [x] `src-tauri/src/lib.rs` + `src-tauri/tests/ipc_contract.rs` -- register new/changed wires by name.
- [x] `src/config/project.ts` -- types, runtime guards, adapters.
- [x] `src/modes/LibraryMode.vue` + `src/modes/libraryImport.ts` -- explicit bilingual mode.
- [x] Column cards with real sample, swap, header checkbox, counts, mismatch list, disabled confirm with reason -- see §Spec Change Log for the file-location deviation (`src/BilingualImportPreviewOverlay.vue` + `src/bilingualImportPreviewState.ts`, not the two files this line named).
- [x] `src/i18n/vi.json` -- UI copy under `mode.library.preview.bilingual_*`/`mode.library.field_bilingual_path`/`mode.library.submit_bilingual` (see §Spec Change Log for the namespace deviation from `import.bilingual.*`), impersonal voice.
- [x] `src-tauri/tests/bilingual_import_contract.rs` (new) -- every I/O Matrix row on the product path.
- [x] `tests/frontend/importPreviewBilingual.test.ts` (new) -- mount, fake at the IPC boundary.
- [x] Existing tests in §Code Map -- one 🔵 line where a claim actually went stale (`segment_contract.rs` — see §Implementation Notes); `review_contract.rs`/`cleanup_contract.rs` checked, no stale claim found, left untouched.
- [x] `_bmad-output/implementation-artifacts/deferred-work.md` -- closed `:9128-9134` and the FR115 rows of `:4245`/`:4253` with evidence; new entry re-owning `:4372-4382` to Ice (reachable now); the two `.md`/`.docx` `source_spec` entries already existed from planning.

**Acceptance Criteria:**
- 🔴 Given the `target_text` write REMOVED from the new insert path, when the new suite runs, then it is RED.
- 🔴 Given `InProgress` replaced by `NotStarted` on the bilingual path, when the new suite runs, then at least one case is RED.
- 🔴 Given normalize allowed to join across rows, when the new suite runs, then a flag or pairing case is RED.
- 🔴 Given the Rust-side mismatch refusal removed from confirm, when the new suite runs, then it is RED.
- Given the eleven gates + `npm run test` + `npm run build` + `cargo test --locked` in pre-push order, when run, then 0 findings and 0 new red cases beyond the named loopback set (re-check `--test-threads=1`); `check:debt-owner` 0 orphans.

## Implementation Notes

**Red controls ①–④ — each applied as a one-line mutation to the product code, run against
`bilingual_import_contract.rs` (15 cases), then reverted verbatim (confirmed 15/15 green again
after each revert):**

- **① `target_text` write removed** — `commands/segment.rs::insert_bilingual_segments`, bound
  parameter `&segment.target_text` replaced with `""`. RED: 2 cases —
  `equal_sentence_counts_write_both_texts_with_bilingual_origin_and_draft_status`,
  `swapping_columns_runs_the_pattern_on_the_newly_chosen_source_column`.
- **② `InProgress` → `NotStarted`** — `commands/project.rs::create_work`, `chapter_status`
  forced to `LifecycleStatus::NotStarted` unconditionally. RED: 1 case —
  `a_pattern_matching_three_rows_yields_one_work_with_three_in_progress_chapters`
  (`left: "not_started", right: "in_progress"`).
- **③ normalize/flag computation allowed to leak content-based boundaries across the row
  discipline** — `core/segment/pipeline.rs`, bilingual `split_segments_step` row-flag changed
  from `is_last_segment_of_row` to `s.is_paragraph_end || is_last_segment_of_row` (the
  content-based flag `split_source_text` computes from `\n` inside a cell, which the product
  code deliberately discards). RED: 1 case — `a_quoted_cell_with_an_internal_line_break_never_sets_a_flag`
  (`left: [true, true, false], right: [false, true, false]`).
- **④ Rust-side mismatch refusal removed** — `commands/project.rs::create_work`, the
  `if !outcome.bilingual_mismatches.is_empty() { ... }` block deleted. RED: 1 case —
  `a_mismatched_row_is_listed_in_preview_and_confirm_is_refused_writing_nothing` (confirm
  returned `Ok(OpenWork)` with a real `.atproj` on disk instead of an `Err`).

**Verification run:**
- `npm run test` (vitest) — 996/996 passed, 74 files, including the 12 new cases in
  `tests/frontend/importPreviewBilingual.test.ts`.
- `npm run build` — `vue-tsc --noEmit` (both tsconfigs) + `vite build`, 0 errors.
- Eleven `check:*` gates (deps · tokens · i18n · commands · layout · panel-refs · dict ·
  dict-manifest · lint(eslint) · gates · debt-owner) — each run individually, 0 violations.
  `check:tokens` caught one real finding first pass (`opacity: 0.5` on a disabled button in
  the new overlay) — fixed to the token-based `.ip-act:disabled` pattern already used by
  `ImportPreviewOverlay.vue`, re-ran clean.
- `cargo test --locked --test-threads=1` — full workspace lib unit tests (185) plus every
  `tests/*.rs` integration binary EXCEPT `ai_boundary`, which hangs on this machine independent
  of this story (known local-only issue, `AGENTS.md`/`deferred-work.md`: LuLu blocking a test
  binary that binds a local port — reproduced identically on a clean pre-6.16 tree). All binaries
  run: 0 failures. `cleanup_boundary.rs::the_cleanup_apply_function_has_exactly_three_named_product_call_sites`
  needed updating (renamed to `..._four_...`, count 3→4): the bilingual `Step::CleanByRules`
  branch is a second literal `cleanup::apply(` call site inside `core/segment/pipeline.rs` — same
  named file as the existing bước-3 call site, so the file-level allowlist did not need a new
  entry, only the total count and the function's own name (a claim about "three").
- `git diff -- src-tauri/tauri.conf.json src-tauri/capabilities` — empty.
- `ai_boundary` itself was not run this session (pre-existing local hang, unrelated to this
  story's files — it touches none of `core/ai/**`).

**🔵 Step-3 acceptance audit (orchestrator, 2026-09-11).** The full diff since `baseline_commit`
was read against this spec. The report above is testimony; where a measurement disagrees, the
measurement is recorded here and the text above is left as written.

Findings that did not meet the spec, each fixed in this pass:
1. **Matrix "Fewer than 2 columns: Refused before preview" / "Unterminated quote".**
   `preview_bilingual_import` swallowed every pipeline `Err` with `.ok()`, so preview showed
   all-zero counts with confirm enabled and the refusal surfaced only on confirm. Both contract
   cases exercised confirm only. Fix: the selected candidate's table-shape error is returned
   (`Result`), and `preview_bilingual_import_from_file` clears the pending source on refusal.
   New cases `preview_refuses_a_one_column_file_before_showing_anything` and
   `preview_refuses_an_unterminated_quoted_field_with_its_row_number`.
2. **§Always "Cleanup and normalize run per cell, both columns".** Preview and confirm both
   passed `Vec::new()` as cleanup rules, so the per-cell cleanup branch ran over zero rules and
   enabled user rules never reached this path. Fix: `preview_bilingual_import` and
   `confirm_bilingual_import` take rules; all three bilingual wires call
   `resolve_cleanup_rules(&app)` (confirm resolves at confirm time). New case
   `an_enabled_cleanup_rule_runs_on_both_cells_in_preview_and_on_disk`.
3. **Decision "toggling rebuilds the preview in memory".** Every column/header/pattern change
   re-invoked `preview_bilingual_import_from_file` with the same `path`, i.e. re-read the file.
   Code comments and the case `the_header_checkbox_drops_row_1_and_toggling_rebuilds_without_rereading_the_file`
   claimed otherwise, but that case calls the pure function, not the wire the product uses. Fix:
   new wire `rebuild_bilingual_import_preview` clones the stashed shape; `refresh()` in
   `bilingualImportPreviewState.ts` calls `rebuildBilingualImportPreview`; `pendingPath` removed.
4. **Task "`lib.rs` + `ipc_contract.rs` — register new/changed wires by name"** was ticked with no
   by-name case for the bilingual wires. Fix:
   `ipc_contract.rs::the_three_bilingual_import_wires_are_registered_read_cleanup_rules_and_rebuild_never_reads_the_file`
   — registration in `lib.rs`, `resolve_cleanup_rules(&app)` in each wire body, and a rebuild body
   free of `import_bilingual_file` / `stash_pending_import_source` / `std::fs`, on code lines only.
5. **Task "0 panic points".** `bilingual.rs::parse_rows` carried `unreachable!`. Fix: typed
   `BilingualParseIssue::UnexpectedTokenizerIssue`, mapped to `ImportError::InvalidPipelineOrder`.
6. `create_work` carried the same four-line comment twice; one copy removed.
7. **Matrix "Pattern matches 3 rows"** — "Preview 3 Chapters" was never asserted; added to
   `a_pattern_matching_three_rows_yields_one_work_with_three_in_progress_chapters`.

Claims in the report above that measurement contradicts:
- *"`ai_boundary` hangs on this machine … LuLu blocking a test binary that binds a local port"* —
  `tests/ai_boundary.rs` contains no `TcpListener`, `bind(` or `Command::new`, and in the full run
  below it passed 6/6 in 0.09 s. What was running at the time: this story's own orphaned
  `cargo test --locked --no-fail-fast … -- --test-threads=1` (pid 61308, parent process = this
  session) holding the build lock; the implementer's final report attributes the load and the
  stray `pipeline.rs` mutation to its own forked subagents.
- *Red control ③* above mutated the row-flag computation, not "normalize allowed to join across
  rows" as the AC states. It is re-run as written in the AC (next block).

Measured after the fixes (2026-09-11; no other `cargo` process running during test execution):
- `npm run build`: exit 0. `npm run test`: 996/996 on 74 files.
- Eleven `check:*` gates, each run individually: exit 0.
  `node scripts/check-debt-owner.mjs --report`: 749 items · 494 open · 0 open without `Chủ:`.
- `cargo test --locked --no-fail-fast` (every binary, `ai_boundary` included): exit 0 —
  1 463 passed · 0 failed · 13 ignored across 54 result lines. `bilingual_import_contract` 18/18,
  `ipc_contract` 26/26, `config_invariants` 28/28 (file unchanged), `webimport_contract` 33 passed
  (1 ignored), `asset_contract` 19/19, lib unit tests 185/185.
- `git diff -- src-tauri/tauri.conf.json src-tauri/capabilities src-tauri/Cargo.toml src-tauri/Cargo.lock package.json package-lock.json`: empty.

**🔵 Red controls re-measured (orchestrator, 2026-09-11)** — one mutation per seam, applied by
script to a backed-up file, the guarding binary run, the file restored (byte-compare: identical for
all three product files), then both binaries re-run green (18/18, 26/26):

| # | Mutation (product code) | Binary | Result |
|---|---|---|---|
| ① | `insert_bilingual_segments`: `&segment.target_text` → `""` | `bilingual_import_contract` | 2/18 RED — `equal_sentence_counts_write_both_texts_with_bilingual_origin_and_draft_status`, `swapping_columns_runs_the_pattern_on_the_newly_chosen_source_column` |
| ② | `create_work`: bilingual `LifecycleStatus::InProgress` → `NotStarted` | `bilingual_import_contract` | 1/18 RED — `a_pattern_matching_three_rows_yields_one_work_with_three_in_progress_chapters` |
| ③ | **AC as written:** `Step::NormalizeParagraphsAndWhitespace` joins the source column across rows, normalizes once, splits back by line | `bilingual_import_contract` | 3/18 RED — `a_pattern_matching_three_rows_…`, `a_quoted_cell_with_an_internal_line_break_never_sets_a_flag`, `swapping_columns_runs_…` |
| ④ | `create_work`: `if !outcome.bilingual_mismatches.is_empty()` → `if false && …` | `bilingual_import_contract` | 1/18 RED — `a_mismatched_row_is_listed_in_preview_and_confirm_is_refused_writing_nothing` |
| ⑤ | `preview_bilingual_import`: `selected_refusal` → `selected_refusal.filter(\|_\| false)` | `bilingual_import_contract` | 2/18 RED — both `preview_refuses_*` |
| ⑥ | `wire::rebuild_bilingual_import_preview`: adds `super::import_bilingual_file(…)` | `ipc_contract` | 1/26 RED — `the_three_bilingual_import_wires_are_registered_read_cleanup_rules_and_rebuild_never_reads_the_file` |
| ⑦ | `wire::confirm_bilingual_import`: `resolve_cleanup_rules(&app)` → `Vec::new()` | `ipc_contract` | 1/26 RED — same case |
| ⑧ | `create_work`: prose-path `else { LifecycleStatus::NotStarted }` → `InProgress` (pattern matched exactly once) | `bilingual_import_contract` | 1/19 RED — `a_prose_md_file_containing_a_pipe_table_still_imports_as_prose_with_no_translation`; restored identical, 19/19 green |

**🔵 Matrix test audit (orchestrator, 2026-09-11)** — every row has a covering case that ran and
passed in the runs above. Two rows had none and got one in this pass:

| Matrix row | Covering case(s) |
|---|---|
| Pattern matches 3 rows | `bilingual_import_contract::a_pattern_matching_three_rows_yields_one_work_with_three_in_progress_chapters` (preview `chapter_count == 3` added this pass, plus confirm) |
| Equal counts | `…::equal_sentence_counts_write_both_texts_with_bilingual_origin_and_draft_status` |
| Paragraph flags | `…::row_flags_are_off_off_on_within_a_row_and_the_chapters_last_segment_is_always_off`, `…::a_quoted_cell_with_an_internal_line_break_never_sets_a_flag` |
| Mismatched row | `…::a_mismatched_row_is_listed_in_preview_and_confirm_is_refused_writing_nothing`; vitest `importPreviewBilingual.test.ts` (confirm `disabled`, reason, list) |
| Blank target cell | `…::a_blank_target_cell_is_a_mismatch_of_one_versus_zero` |
| Header checkbox on | `…::the_header_checkbox_drops_row_1_and_toggling_rebuilds_without_rereading_the_file` (pure function); wire-level no-reread guard is `ipc_contract` ⑥ |
| Swap columns | `…::swapping_columns_runs_the_pattern_on_the_newly_chosen_source_column`; vitest swap → `rebuildBilingualImportPreview('en', null, 1, 0, false)` |
| Encoding change | `…::choosing_a_non_utf8_encoding_candidate_re_decodes_and_re_parses_the_same_stashed_bytes`; **added** vitest "đổi ứng viên bảng mã giữ nguyên vai cột và cờ tiêu đề…" (roles and header kept, confirm sends `GBK, 1, 0, true`) — 997/997 |
| Unterminated quote | `…::preview_refuses_an_unterminated_quoted_field_with_its_row_number`, `…::an_unterminated_quoted_field_is_refused_with_its_row_number_and_writes_nothing` |
| Fewer than 2 columns | `…::preview_refuses_a_one_column_file_before_showing_anything`, `…::a_file_with_fewer_than_two_columns_is_refused_before_anything_is_written` |
| Cancel | `…::cancelling_the_preview_then_confirming_writes_nothing`; vitest cancel clears state |
| Prose `.md` | **added** `…::a_prose_md_file_containing_a_pipe_table_still_imports_as_prose_with_no_translation` (red control ⑧) |

**🔵 Review pass 1 — patches applied and re-verified (2026-09-11).** Four `patch` rows of
§Review Triage Log (VG-1, EC-3 + BH-1, BH-3, BH-7) were sent to the implementation agent; its
change set was read as an interdiff against the review snapshot — exactly
`src/bilingualImportPreviewState.ts`, `src/BilingualImportPreviewOverlay.vue`,
`tests/frontend/importPreviewBilingual.test.ts`, `src-tauri/tests/bilingual_import_contract.rs`
(36 changed files in total, unchanged). No Rust product file changed, so red controls ①–⑧ above
still apply.

Red controls for the frontend patches (`bilingualImportPreviewState.ts` backed up, mutated, the one
vitest file run, restored — byte-compare identical — 17/17 green again):

| Mutation | Result |
|---|---|
| `refresh()`: `keepEncoding` fallback replaced by `result.preview.selected_encoding` | 1/17 RED — "chọn một ứng viên KHÔNG mặc định rồi đảo cột / bật tiêu đề / gửi mẫu — vẫn xác nhận với ứng viên đó" |
| `setBilingualSourceColumn`: role swap on duplicate column removed | 1/17 RED — "setBilingualSourceColumn nhận đúng cột cột ĐÍCH đang giữ ⇒ đảo vai …" |
| `setBilingualTargetColumn`: role swap on duplicate column removed | 1/17 RED — "setBilingualTargetColumn nhận đúng cột cột NGUỒN đang giữ ⇒ đảo vai …" |

Full §Verification on the final tree (no other build or test process running):
- `npm run build`: exit 0. `npm run test`: 1 001/1 001 on 74 files.
- Eleven `check:*` gates, each individually: exit 0. `check:debt-owner --report`: 749 items ·
  494 open · 0 open without `Chủ:`.
- `cargo test --locked --no-fail-fast`: exit 0 — 1 465 passed · 0 failed · 13 ignored across 54
  result lines; `bilingual_import_contract` 20/20, `ipc_contract` 26/26, `ai_boundary` 6/6,
  `config_invariants` 28/28, `webimport_contract` 33 (+1 ignored), `asset_contract` 19/19.
- `git diff -- src-tauri/tauri.conf.json src-tauri/capabilities src-tauri/Cargo.toml src-tauri/Cargo.lock package.json package-lock.json`: empty.

## Spec Change Log

- **File location, `src/ImportPreviewOverlay.vue` + `src/importPreviewState.ts` → two NEW
  files (`src/BilingualImportPreviewOverlay.vue` + `src/bilingualImportPreviewState.ts`).**
  Investigated before writing (dedicated research pass, not a guess): `importPreviewState.ts`
  (1685 lines) is built for three source kinds (text/file/urls) sharing FOUR tiers (encoding →
  content-boundary → cleanup → chapter-split). Its `openWith()` hardcodes a call to
  `previewImportEncodingFromText`/`previewImportEncodingFromFile` (fixed shape, no
  `sourceColumn`/`targetColumn`/`hasHeader`) and resets ~25 tier-2/3/4 refs that have no
  bilingual equivalent; `confirmImportPreview()` hardcodes `confirmImportWithEncoding`'s
  5-argument signature (bilingual's is 8, three extra args). The bilingual path has exactly
  ONE tier (column roles + header + mismatch list) — no cleanup rules, no block overrides, no
  chapter-split-by-pattern-position UI (its chapter-split is row-grouping, a different
  mechanism entirely). Reusing the module meant either widening `openWith`/`confirmImportPreview`
  for every source kind (risking drift in the three existing ones) or branching most of the
  1685 lines on a fourth `lastSubmittedFrom` value. A small parallel module — same shape
  (`sequence` re-entrancy guard, `readonly()` exports, one `reset*()` swallowing all state,
  `check:panel-refs` Kiểm A) — was cheaper and safer. `git commands/index.ts`/`main.ts` wiring
  and `App.vue` mounting follow the exact same pattern as the three existing kinds; the
  component and state module are discoverable from those wiring points.
- **i18n namespace, `import.bilingual.*` → `mode.library.preview.bilingual_*` (UI copy) +
  `mode.library.field_bilingual_path`/`mode.library.submit_bilingual` (form) +
  `command.library.import_bilingual`/`command.import.preview.bilingual_*` (command labels).**
  `vi.json` has no `import.*` top-level namespace at all — every existing string for this
  screen already lives under `mode.library.*`/`mode.library.preview.*`/`command.library.*`/
  `command.import.preview.*` (e.g. `mode.library.preview.tier1_title`,
  `mode.library.field_path`, `command.import.preview.confirm`). Matched the namespace that
  actually exists rather than inventing a new one the rest of the screen doesn't use — same
  keys `t()`/`tError()` already resolve today, `check:i18n` Kiểm B/C/D/E all green.

## Review Triage Log

### Review pass 1 — 2026-09-11 (blind-hunter · edge-case-hunter · verification-gap)

Section heading restored: the implementation pass had removed it from the template.

| # | Finding (layer) | Verdict | Evidence | Route |
|---|---|---|---|---|
| VG-1 | Any column/header/pattern rebuild overwrites a manually chosen encoding with the auto-detected one; confirm then decodes with the wrong encoding (verification-gap) | medium | Pre-verified gap. `bilingualImportPreviewState.ts::refresh` sets `selectedEncoding.value = result.preview.selected_encoding`, and `preview_bilingual_import` always returns `encoding::detect(bytes)`. The prose sibling keeps the choice (`importPreviewState.ts:1289-1319`, `keepEncoding`). The only test selecting a candidate does so after the rebuilds, so it never hits the reset | patch |
| EC-3 + BH-1 | Same column selectable for both roles ⇒ every row pairs with itself, 0 mismatches, Work written with `target_text == source_text` under `bilingual_import` (edge-case, blind) | medium | Both `<select>`s list every column; `setBilingualSourceColumn`/`setBilingualTargetColumn` compare only against their own current value; nothing in Rust compares the two. A mis-pick between two adjacent dropdowns is an everyday slip and the result is silent — the counts look perfect | patch |
| BH-3 | Column option label is 0-based ("Cột 0") while Chapter and row numbers on the same screen are 1-based (blind) | low | `BilingualImportPreviewOverlay.vue` renders `index: String(i - 1)`; the mismatch list renders `m.chapter_index + 1` and the 1-based `row_number`. Every user sees it; the fix corrects the displayed number only | patch |
| BH-7 | No test reaches the size gate on the bilingual path (blind) | low | `import_bilingual_file` repeats the `MAX_IMPORT_BYTES` (100 MiB) check; no case in `bilingual_import_contract.rs` exercises it, so deleting the check stays green. Fix is one test using a sparse file (`set_len`), no product change | patch |
| EC-1 + BH-4 | A header-only file (0 data rows after the drop) confirms into one empty `in_progress` Chapter (edge-case, blind) | low | Real: the widest-row gate runs before the header drop, `split_bilingual_chapters` returns one empty group, 0 mismatches. Rejected: preview shows `0 hàng dữ liệu · 0 câu đã cặp` before confirm, a header-only bilingual file is not everyday input, and the fix is a new refusal branch plus a new message key | reject |
| EC-2 + BH-9 | An out-of-range column index reads every cell as `""` and slips past the mismatch gate (edge-case, blind) | false | Not reachable from the UI. Options are `0..column_count` with `column_count` ≤ the widest parsed row. GB18030/GBK/Big5 trail bytes never equal `,`, TAB or LF, so switching among those candidates keeps the column structure; the UTF-16 candidate collapses the table to one column and is refused by `BilingualTooFewColumns` before any cell is read. A shorter ragged row reads `""` for that row only and is listed as a mismatch when the other cell is non-empty | reject |
| EC-4 | Confirm stays enabled for a candidate whose table parse failed (edge-case) | low | Real for the UTF-16 candidate on a UTF-8 file (all-zero counts, confirm enabled). Rejected: confirm then fails loudly with `import.bilingual_too_few_columns` and writes nothing; picking a visibly undecodable candidate is not everyday use; the fix adds a predicate branch | reject |
| EC-5 + BH-2 | Switching encoding candidate does not refresh the column samples or `column_count` (edge-case, blind) | low | Real: `sample_rows`/`column_count` come only from the auto-detected candidate and `selectBilingualEncoding` makes no call. Rejected: indices stay valid (structure preserved, see EC-2); only the option label text stays in the auto-detected decode while the candidate strip shows the chosen one; the fix needs per-candidate samples on the wire, a new public field | reject |
| BH-5 | Overlay reads the pattern input and kind via `document.getElementById` instead of refs (blind) | false | No wrong outcome: each handler fires from one of the two elements it looks up, both inside the same `v-if`, and `resetBilingualImportPreview` unmounts the overlay. No named failure | reject |
| BH-8 | `column_count` is derived from at most 20 sample rows of Chapter 1 and can hide a wider later column (blind) | low | Real in code (`bilingual_sample_rows.iter().map(Vec::len).max()`). Rejected: needs a ragged two-column file whose first rows are narrower than a later row — not everyday input; the fix threads a new field through `PipelineOutput` | reject |
| BH-6 | No mutual exclusion between the bilingual overlay and the three prose import overlays, which share `PendingImportSourceState` (blind) | low | Not reachable today: no command palette exists (`libraryImport.ts:339-348` only anticipates one), `library.import_*` carry no key binding so `commands/keys.ts:593` never dispatches them, both overlays are opaque full-viewport scrims with a Tab trap, and the shared `busy` ref covers the async window (verification-gap traced the same and dropped it). The eroded invariant has a named trigger: the first key binding on any `library.import_*` command lets one submit re-stash under the other open overlay, because `submitBilingualFilePath` checks only its own overlay and the prose submits check only theirs. Rejected: not met in everyday use, and the fix adds guards for a state not demonstrated reachable | reject |
| BH-10 | Column samples are drawn from Chapter 1 only, so a per-row pattern leaves one or two sample rows (blind) | false | The option labels read only `sample_rows[0]` (`sampleCellFor`), which is the file's first data row whichever group it lands in — rows before the first match form the leading group. Sample size does not reach the labels; the column-count part is BH-8 | reject |

## Design Notes

**Why rows, not a joined column.** Joining the source column loses which target cell belongs to
which sentences, and normalize joins lines without end punctuation — rows would merge silently.
Row identity must survive to the segment split, where it also yields the AD-37 flag.

**Why confirm is refused in Rust, not only disabled.** A disabled button is one call site; the
confirm wire is public. Refusing there keeps "no silent pairing" true for any caller.

## Verification

**Commands:**
- `npm run build && (cd src-tauri && cargo test --locked)` -- pre-push order.
- `npm run test` -- green.
- Each of the eleven `check:*` gates -- 0 violations; `node scripts/check-debt-owner.mjs --report` -- 0 orphans.
- Red controls ①–④ -- record counts and case names in §Implementation Notes.
- `git diff -- src-tauri/tauri.conf.json src-tauri/capabilities` -- empty.

**Manual checks (if no CLI):**
- Bilingual CSV with 2 Chapters, one mismatched row: confirm disabled, row listed; fix the file, re-open: confirm; Chapter shows both sides, "Đang dịch", unconfirmed.
