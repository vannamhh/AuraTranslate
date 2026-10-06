---
ticket: 10
title: 'Story 7.10 — TMX export and import for one TM tier'
type: 'feature'
created: '2026-10-04'
status: done
route: 'dispatch'
baseline_revision: '8475bcec92f48031f6c43f7c8e90f6a01c5c8057'
review_loop_iteration: 0
context:
  - '{project-root}/AGENTS.md'
  - '{project-root}/src/AGENTS.md'
  - '{project-root}/src-tauri/AGENTS.md'
  - '{project-root}/_bmad-output/implementation-artifacts/epic-7-context.md'
---

<frozen-after-approval reason="human-owned intent — do not modify unless human renegotiates">

## Intent

**Problem:** TM lives only inside `project.db` and `global.db` (FR64, NFR9): the translator cannot take their pairs to another CAT tool, and the Global tier can be filled from outside only by pushing pairs up one by one (AD-18 names TMX import as its other writer).

**Approach:** From the TM management overlay, export one tier to a TMX 1.4b file and import a TMX file into a chosen tier through a preview-then-confirm overlay, on the Glossary CSV exchange pattern (AD-48 dialogs in Rust, one transaction per import).

**Decisions (Ice, 2026-10-04):**
- Q1 ⇒ (A) an imported pair takes `x-aura-origin` when it is one of the three stored values, else `other` (mockup line 294). Ice drops the epic AC "imported pairs carry an origin distinguishable from confirm-generated ones": the origin set stays closed (AD-47 ⑥), round-trip keeps origin, no migration.
- Q2 ⇒ (B) import skips a `<tu>` whose (source, target) already exists in the target tier or earlier in the same file, counted as "đã có" in the preview; export writes one `<tu>` per distinct (source, target) in the tier, carrying its first copy in AD-18 order (origin and `created_at`). Re-importing a file changes nothing.
- Q3 ⇒ (A) one tier per export, every origin; no "chỉ cặp tôi dịch" option.
- Spec length 2,991 tokens kept as one spec (round-trip needs both halves).
- Q4 (review loop 0) ⇒ the import cap is 256 MiB, not 64 MiB: a typical `<tu>` is 466 bytes, so 64 MiB held ~144k pairs, below the 100k-per-tier design point of 7.9 plus a Global tier fed by every Work; an exported tier must import back. Peak memory of a 256 MiB import is measured on a release build before sign-off.

## Boundaries & Constraints

**Always:**
- Dialogs open in Rust only (AD-48); the webview never receives file contents; wires that block on a dialog are `#[tauri::command(async)]`.
- Export writes atomically (`.tmp` then rename). Each `<tu>` carries `creationdate` (`YYYYMMDDThhmmssZ`), `<prop type="x-aura-origin">` and `<prop type="x-aura-created-at">` (the stored ISO value), one source `<tuv>` and one `vi` `<tuv>`. Header: `srclang`, `creationtool="AuraTranslate"`, `segtype="sentence"`, `datatype="plaintext"`, `o-tmf`, `adminlang`.
- Source language: the Work tier uses `work.source_lang`; the Global tier (no language stored) labels each pair `zh` when its source holds a Han character, else `en`, with header `srclang="*all*"`.
- Import is two-phase: preview parses and keeps the plan in Rust state; confirm writes every pair in ONE transaction through the tier's single writer, or nothing. Cancel and a new preview drop the plan.
- A file that is not well-formed XML, has no `<tmx>`/`<body>`, is over 256 MiB, or yields no usable pair ⇒ an error naming the problem (code `tm.tmx_*`), nothing written. UTF-8 (BOM allowed) and UTF-16 with BOM are read.
- Per `<tu>`: source = the `<tuv>` whose `xml:lang` primary subtag matches the Work's `source_lang` (Work tier) or the header `srclang` (Global; `*all*` ⇒ the first non-`vi` `<tuv>`); target = the first `vi` `<tuv>`. A `<tu>` missing either, or with a blank trimmed side, is skipped and counted in the preview. Inline elements (`bpt`, `ept`, `ph`, `it`, `ut`, `sub`) contribute no text; `hi` keeps its text.
- `created_at` written on import is always `YYYY-MM-DDTHH:MM:SS.mmmZ`: `x-aura-created-at` when valid, else `creationdate` with `.000`, else now (closes the 7.8 ledger item on date format).
- `<seg>` text is stored as read (no trim, no NFC); "identical" is exact string equality on both sides, the 7.8 read-collapse rule. Confirm re-runs the identical check inside its transaction, so pairs written between preview and confirm are skipped, not doubled.
- Import into the Work tier with no Work open ⇒ `work.none_open`. Global is shared by every Work; the import overlay says so when Global is chosen.
- Import is AD-49 class (ii): it only adds rows, each removable in TM management; the preview's confirm is the explicit act.
- Keyboard: tier choice, export, import, confirm, cancel all reachable; the import overlay traps Tab, Esc cancels. Copy obeys `check:i18n`.

**Never:**
- A new origin value without a new AD (AD-47 ⑥); `tauri-plugin-fs` or any `dialog:*`/`fs:*` capability (AD-48); a new XML or date crate (`quick-xml =0.41.0` is already a direct dependency).
- Overwriting or merging an existing pair (7.8, AD-6).
- Smart RAG (7.11).

## I/O & Edge-Case Matrix

| Scenario | Input / State | Expected |
|---|---|---|
| Export Work | Work `zh`, pairs `(S,A,self)`, `(S,B,other)`, `(S,A,other)` | TMX `srclang="zh"`, two `<tu>`: `(S,A)` origin `self`, `(S,B)` origin `other` |
| Round-trip | export, then import into an empty tier | same distinct (source, target) set, each with the exported origin and `created_at` |
| Re-import | import the same file twice | second preview: every pair "đã có", confirm writes nothing |
| Duplicate in file | two `<tu>` with the same (source, target) | one row written, one counted "đã có" |
| Foreign TMX | `<tu>` with `zh-CN` and `vi-VN` tuvs, no props | imported, origin `other`, date from `creationdate` with `.000` |
| Missing side | one `<tu>` has no `vi` tuv | skipped, preview says 1 skipped |
| Broken XML | unclosed `<seg>` | `tm.tmx_malformed` with line, nothing written |
| Cancel | preview shown, user cancels | nothing written |
| Global, no Work | target tier Global | imports; Work option disabled |

</frozen-after-approval>

## Code Map

- `src-tauri/src/core/tm/mod.rs` -- `PairOrigin` :37 (`as_str`/`from_stored`), `insert_pair` :70 (stamps now; keep for confirm), `TmTier` :85, `load_manage_snapshot` :475, `merge_tiers` (AD-18 order), `push_copies_to_global` :739 (precedent for inserting an explicit `created_at`). New `core/tm/tmx.rs`: pure render and parse (AD-15, no I/O), an insert with explicit date.
- `src-tauri/src/core/glossary/exchange_io.rs` -- `read_import_file` :62 (cap, BOM), `write_export_file` :117 (atomic); generalise or mirror for TM, do not import Glossary error types into `tm/`.
- `src-tauri/src/core/glossary/exchange.rs` ~:523 `looks_like_iso8601_utc` (private shape check); `src-tauri/src/commands/project/work_creation.rs:1653` `is_common_cjk_char` (private).
- `src-tauri/src/commands/glossary.rs` -- analog: `glossary_export_tier` :1368, `glossary_open_import_preview` :1450, `glossary_confirm_import` :1506, `glossary_cancel_import` :1530, `PendingImportState` :686. TM commands go in `commands/tm.rs` (tier mapping, `tm.pair_not_found`-style codes :105-117); register in `src-tauri/src/lib.rs:974-977` and manage a new pending state.
- `src-tauri/src/core/glossary/store.rs:1265` `match_lang_for_source_lang`; `core/segment/split.rs:48` `LANG_CHINESE`; `work.source_lang` `core/store/schema.rs:842`. No `target_lang` exists: target is `vi`.
- `src/TmManageOverlay.vue` -- add an exchange block (tier radiogroup, Xuất TMX, Nhập TMX) near `.tm-actions` :561; `src/tmManageState.ts` write pattern (`deleteTmManageOthers` :370); `src/tmManageCommandDeps.ts`; `src/config/tm.ts` wrappers (`malformed(value, command)` order).
- `src/GlossaryManageOverlay.vue:621-695` `.gm-exchange`, `src/glossaryExchangeGate.ts` (one dialog at a time; reuse), `src/GlossaryImportOverlay.vue` + `src/glossaryImportState.ts` -- copy the shape into `src/TmImportOverlay.vue` + `src/tmImportState.ts`, mount in `src/App.vue` beside :383, wire in `src/main.ts`.
- `src/i18n/vi.json` -- `tm.manage.*` :929-985; mirror `glossary.manage.exchange_*` :1029 and `glossary.import.*` :1036.
- Mockup `_bmad-output/planning-artifacts/ux-designs/ux-AuraTranslate-2026-08-02/mockups/tm-manage.html:261-308` (rewrite copy; no "bạn").

## Tasks & Acceptance

**Execution:**
- [x] `src-tauri/src/core/tm/tmx.rs`, `core/tm/mod.rs` -- render, parse (quick-xml), import plan, insert with explicit date.
- [x] `src-tauri/src/commands/tm.rs`, `src-tauri/src/lib.rs` -- `tm_export_tier`, `tm_open_import_preview`, `tm_confirm_import`, `tm_cancel_import`, pending state.
- [x] `src-tauri/tests/tm_contract.rs` (or `tmx_contract.rs`) -- the matrix through the wired pure fns; round-trip; malformed files from fixtures written in the test.
- [x] `src/config/tm.ts`, `src/tmManageState.ts`, `src/TmManageOverlay.vue`, `src/tmImportState.ts`, `src/TmImportOverlay.vue`, `src/App.vue`, `src/commands/index.ts`, `src/tmManageCommandDeps.ts`, `src/main.ts`, `src/i18n/vi.json`.
- [x] `tests/frontend/tmManage.test.ts`, `tests/frontend/tmConfigGuards.test.ts`, new `tests/frontend/tmImport.test.ts`, `tests/frontend/dialogAccessibleName.test.ts`.
- [x] Guards to live counts: `REGISTERED_COMMAND_FLOOR` (`tests/ipc_argument_contract.rs:530`), `COMMAND_FILE_CENSUS` tm.rs row and `blocking_wire_cases` (`tests/config_invariants.rs`), `tests/ipc_contract.rs` wire fields, `scripts/check-commands.mjs` floors and `HANDLER_TABLE`, `scripts/check-i18n.mjs` `VUE_FLOOR`, `scripts/check-tokens.mjs` `COMPONENT_FILE_FLOOR`, `scripts/check-layout.mjs` `FILE_FLOOR`, `tests/naming_boundary.rs` `FRONTEND_FLOOR`, `tests/segment_boundary.rs` `WEBVIEW_FLOOR`.

**Acceptance Criteria:**
- Given the round-trip test, when import drops the `x-aura-created-at` read, then only the date assertion goes red.
- Given a malformed file in the middle of an otherwise valid batch, when confirm runs, then the tier's row count is unchanged (one transaction).
- Given an exported file, when parsed by an independent reader in the test (quick-xml event walk, not `tmx.rs`), then every `<tu>` has both `<tuv>` and the header carries the TMX 1.4 required attributes.
- Given only the keyboard, when the user chooses a tier, exports, imports, confirms and cancels, then every step works and focus returns to the opener.

## Implementation Notes

- TMX lives in `core/tm/tmx.rs` (pure render, decode, parse, plan, one-transaction write) and `core/tm/tmx_io.rs` (capped read, atomic write with a unique `.tmp`); commands `tm_export_tier`, `tm_open_import_preview`, `tm_confirm_import`, `tm_cancel_import` in `commands/tm.rs`, all `(async)`.
- The preview keeps only the pairs the tier lacks plus the preview's already count; confirm re-checks identity inside its transaction and reports already = preview count + pairs that raced in.
- A new preview drops any earlier plan before the dialog opens, so a cancel or a failed read, parse or plan leaves none. Closing or swapping the Work drops a Work plan; a Global plan survives.
- The wires read and parse the import file before taking `OpenWorkState` and write the export after dropping it; plan and render run under the guard.
- Peak memory at the 256 MiB cap (Q4), release test binary under `/usr/bin/time -l`, Ice's Mac, one synthetic 262,144,115-byte file of 883,388 units into an empty zh Work tier: max RSS ~1.53 GiB, footprint ~1.09 GiB; preview 4.9 s, preview plus confirm 20.6 s. Harness: the ignored `a_cap_sized_file_previews_and_confirms`.
- `tm.tmx_no_usable_pair` carries `source_lang` for the Work tier so the copy names the expected language; Global has none.
- Export drops XML-1.0-illegal control characters, so such text does not round-trip (triage B7, rejected as rare).
- TMX error codes carry no `message_key`; `src/tmExchangeError.ts` maps them. Export and import share `glossaryExchangeGate` with Glossary and prompt-set exchange.
- Guards moved to live counts: `REGISTERED_COMMAND_FLOOR` 110 to 114, tm.rs census 3/6, tree 66/49, `check-commands` VUE 30 / COMMAND 171 / CLICK 127 / DISPATCH 183, `check-i18n` VUE 30, `check-tokens` 108, `check-layout` 109, `naming_boundary`/`segment_boundary` 109.
- Ledger: 7.8 date-format item closed; new items for the real-app pass (dialogs, layout, focus, OmegaT) and the unguarded `isBlocked` line, both Chủ: Epic 7.

## Spec Change Log

## Review Triage Log

Loop 0 (blind B, edge E, verification-gap VG):
- E6 export over 64 MiB refused on re-import -- medium: a typical tu is 466 bytes, so 64 MiB holds ~144k pairs while 7.9 sizes tiers at 100k; root is the frozen 64 MiB cap -> intent_gap, resolved by Ice as Q4 (256 MiB, keep the code, no revert).
- B1 import-done "bỏ qua N cặp đã có" counts only the confirm-time race -- medium: `already_count` = `outcome.already_there`, preview's already pairs were filtered out before -> patch.
- B3/E1/E12/E claim 1-2 failed or cancelled preview keeps the old plan -- low: spec says a new preview drops the plan; doc comment "leaves no plan" is false -> patch.
- B4/E7 `OpenWorkState` held across file read/parse and export write -- medium: guard is live through `read_tmx_file`/`parse_tmx` and `write_tmx_file` -> patch.
- B2/VG1 Work-plan clearing in `close_open_work`/`replace_open_work` unguarded -- medium: test calls the helper directly; removing either call stays green -> patch.
- VG2 Global import with concrete `srclang` untested -- medium: arm returning `None` stays green -> patch.
- VG3 export AD-18 date/id tiebreak untested -- low: duplicates in tests differ by side only -> patch.
- VG4 "refresh falls back to Global" test never refreshes -- low: deleting the line stays green -> patch.
- VG6 date range check and prop->creationdate fallback untested -- medium: `date_parts_valid` -> true stays green -> patch.
- B8 language mismatch reported as generic "no usable pair" -- low: copy blames missing sides; add the Work's source language -> patch.
- B11 ledger lines (close 7.8 date item, person-check debt) missing -- medium: no `deferred-work.md` change in diff -> patch (orchestrator).
- VG5 `main.ts` `isBlocked` line for the import overlay untested -- medium: same class as the open `main.ts` wiring debt -> defer.
- B5 write failure reported as lookup -- false: `tm_lookup_failed` maps `TmStoreError::Store` through `IpcError::from`, so a store write error keeps its store code.
- B6 day-of-month not checked against month -- low: a foreign tool writing Feb 31 is unlikely; fix adds branches.
- B7/E2/E claim 3 C0 controls dropped on export, accepted on import -- low: XML 1.0 cannot carry them; TM text with them is rare.
- B9 `lang` matched by local name -- false: accepting TMX 1.1 `lang` is harmless; `type`/`creationdate` carry no namespace.
- B10 Tasks lists `tmManage.test.ts` untouched -- rejected: fix edits this spec.
- B12/E9 Manage closed while the pick dialog is up -- low: the OS dialog is modal; `.ti-close` click while confirming is a guarded no-op.
- B13 missing tests for confirming/sequence guards -- low: guards on states not shown reachable.
- B14 duplicate radiogroup name, "cầu IPC" copy, fixed file name -- low: Glossary exchange uses the same copy and pattern.
- VG other 1 `resetTmImport` has no caller -- low: dead export, harmless.
- VG other 2 `resetTmManage` clears the shared gate -- false: `glossaryManageState.ts:666` does the same, overlays are modal.
- VG other 3 Work swap during dialog -- false: the plan targets the Work open at plan time, consistently.
- E3 lang-less tuv or empty `source_lang` -- low: `work.source_lang` is NOT NULL and set at creation.
- E4 multilingual Global TMX takes the first non-vi tuv -- false: the spec rule for `*all*`.
- E5 nested `<prop>` -- low: not valid TMX.
- E8 save path without `.tmx` -- low: the dialog filter adds it.
- E10 confirm sequence bump hides the notice -- false: nothing bumps `sequence` during a confirm.
- E11 cancel racing a new preview -- low: the new dialog takes seconds, cancel microseconds.
- E13 tier changed while a dialog is up -- low: the OS dialog is modal.
- E14 whole tier loaded for export/preview -- low: the 7.9 measured design (load-all).

## Design Notes

TMX is TM-specific (import and export), so it lives in `core/tm/tmx.rs`, not in the empty `core/export/` placeholder the spine tree comment names. CAT tools (OmegaT, Trados, memoQ) ignore unknown `<prop>`, so origin and the millisecond date ride along without breaking other readers.

## Verification

**Commands:**
- `cargo test --test tm_contract` -- green.
- `npm run test:story 7-10 -- --list`, then the listed vitest files -- green.
- Full suite once (`lib.rs` registration changes).

**Manual checks:**
- Open an exported file in OmegaT or another CAT tool (AC "đọc được"): a person check, debt item `Chủ: Epic 7`.

## Acceptance criteria from epics.md

Source: `epics.md` §Story 7.10 (v6, nay ở archive-v6).

**Covers:** FR64

As a người dịch,
I want mang kho dịch của mình sang một công cụ khác,
So that dữ liệu của tôi sống lâu hơn phần mềm và tôi không bị khoá vào `.atproj`.

**Acceptance Criteria:**

**Given** một Translation Memory ở tầng bất kỳ
**When** người dùng xuất
**Then** sinh ra file **TMX** hợp lệ

**Given** file TMX vừa xuất
**When** mở bằng một CAT tool khác
**Then** đọc được các cặp

**Given** file TMX vừa xuất
**When** nhập lại vào một TM rỗng
**Then** **round-trip đầy đủ** các cặp

**Given** người dùng chọn tầng đích khi nhập
**When** thực hiện
**Then** cặp vào đúng tầng đó

**Given** cặp nhập từ TMX
**When** ghi
**Then** mang xuất xứ phân biệt được với cặp sinh ra từ việc xác nhận segment

**Given** một file TMX sai định dạng
**When** nhập
**Then** báo lỗi nêu rõ vấn đề
**And** không ghi một phần

**Given** một cặp TMX trùng với cặp đã có
**When** nhập
**Then** áp đúng quy tắc của Story 7.8 — giữ cả hai, không ghi đè
