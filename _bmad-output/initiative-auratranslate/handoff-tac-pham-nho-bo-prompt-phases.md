## Phase 1 (Rust)

Done in `src-tauri/` only (nothing under `src/` or `tests/frontend/` touched).

- Migration 41: `work.chosen_prompt_set_name TEXT` (`WORK_CHOSEN_PROMPT_SET_NAME_DDL`, `core/store/schema.rs`). `PROJECT_MIGRATIONS.len()` is now 40, `schema_version` 41.
- Core: `core::promptset::{read_chosen_name, write_chosen_name}` (store.rs). Write trims; blank or `None` stores NULL. No `updated_at`, no `meta.json`.
- Commands (registered in `lib.rs` after `prompt_set_delete`), no Work id:
  - `invoke('prompt_set_choice_get')` -> `string | null` (JSON `Option<String>`).
  - `invoke('prompt_set_choice_set', { name })` -> `null`/void; `name: string | null` (Tauri arg key is `name`). Passing `null` clears the column.
- Errors (same shape as other `prompt_set_*`): no open Work -> code `prompt_set.work_tier_unavailable` (message_key `err.prompt_set.work_tier_unavailable`, not retryable); store failure -> `store.read_failed` / `store.write_failed`. The adapter must not throw (return a result/null as the plan says) and the caller reports write failures via `tError`.
- The name is stored exactly as the webview sends it (trimmed); matching (trim + case-insensitive, like `applyGenre`) is the webview's job. Rust never validates it against `prompt_set`.
- Rust tests added in `tests/prompt_set_contract.rs` (round trip, reopen, per Work, blank -> NULL, no `updated_at`/`meta.json` change, migration from step 40, no Work -> error); `ipc_contract` registration list; `config_invariants` census promptset 7 -> 9 plain, tree total 107 -> 109. Existing tests bumped for schema 41: `pinned_contract`, `tm_contract` (downgrade SQL now drops the column), `proofread_decision_contract`, `review_harvest_contract`, `segment_contract` (newer-than-app fixture is now step 42).
- Rust half of counter-check not run separately: removing the two `lib.rs` registration lines turns `ipc_contract::the_prompt_set_wires_are_registered` red by construction (it scans `lib.rs`). The frontend read/write counter-checks stay for phase 3.
- Adapter arg shape: the `ipc_argument_contract` scanner only accepts an object literal as the second `invoke` argument after a `CMD_*` const, so write `invoke(CMD_CHOICE_SET, { name })` (never a ternary of two shapes). `src/config/aiconfig.ts:179` already breaks that scanner (`tier === undefined ? { field } : { field, tier }`), so `cargo test --test ipc_argument_contract` is red with a panic at `tests/ipc_argument_contract.rs:538` for a reason independent of this story (src/ is outside phase 1; not fixed, not verified against a clean baseline). `REGISTERED_COMMAND_FLOOR` raised 137 -> 147 here (two new commands tipped the 80% rule).
- Status at end of phase 1: `cargo test --no-fail-fast` full run had only the three failures that were then fixed (chapter_origin, glossary_unconfirm schema-version literals) plus the ipc_argument_contract one above; `check:clippy` 0, `check:fmt` 0.

## Phase 2 (Webview)

Edited only `src/` (no tests touched, nothing committed).

- `src/config/promptset.ts`: `promptSetChoiceGet(): {name, error}` and `promptSetChoiceSet(name|null): IpcError|null`; plain `{ name }` literal, never throws.
- `src/promptSetState.ts`: `setSelectedPromptSetName` (sync, still `void`) now fires `promptSetChoiceSet` when `workTierAvailable` is true (proxy for "a Work is open"; skipped otherwise); write error goes to new exported `promptSetChoiceError` (shown with `tError` in `AiTranslationPanel.vue`, class `ai-inspector-alert`), session selection kept. New `restorePromptSetForWork(genre)` (sets `pendingRestore`, then `loadPromptSets`): after the list loads it reads the column, matches trim + case-insensitive, selects with `selectionIsAutomatic = false`; if unresolved or NULL it CLEARS the session selection, then `applyGenre`. (Clearing is deliberate: per-Work memory means Work A's hand pick must not leak into Work B.) `autoSelectPromptSetForGenre` is kept (not used by callers any more; only `addPromptTemplates` still uses the pending-genre path). Header comment block rewritten. `resetPromptSets` clears the new refs.
- `libraryChapters.ts:362`, `libraryImport.ts:459`: call `restorePromptSetForWork`.
- `AiTranslationPanel.vue`: `:columns="2"` on the prompt-set `SelectMenu`.

Vitest files red after this phase (all targets of phase 3):
- `tests/frontend/promptSetState.test.ts` x2: `vứt toàn bộ state về mặc định` and `Ice đã chọn tay: thêm mẫu trùng thể loại giữ nguyên bộ đã chọn` — the `vi.mock('../../src/config/promptset')` factory lacks `promptSetChoiceSet`/`promptSetChoiceGet` ("No promptSetChoiceSet export defined"), triggered by `setSelectedPromptSetName`. Add both to the mock; then add the matrix rows and fix the `:589-590` case.
- `tests/frontend/libraryChaptersResetsAiPromptInspector.test.ts` x1: `openWorkById() tự chọn bộ prompt theo genre` expects `autoSelectPromptSetForGenre` to be called; it is now `restorePromptSetForWork` (rename in the mock/expectation).
- `selectMenu.test.ts` passes; no panel test asserts `columns=2` yet.
- Counter-check seams for phase 3: the `promptSetChoiceGet()` call inside the `pendingRestore` block of `loadPromptSets`, and the `void promptSetChoiceSet(name)` in `setSelectedPromptSetName`.

## Phase 3 (Tests)

- Suspected `lastGenre` defect: not real. `restorePromptSetForWork` sets `lastGenre` exactly as the old function did; the test "Tác phẩm đang mở, chưa chọn bộ: thêm mẫu cùng thể loại thì tự chọn ngay" now opens the Work through `restorePromptSetForWork` and passes. No production fix for it.
- `autoSelectPromptSetForGenre` had no production caller: removed from `src/promptSetState.ts` (only production edit this phase). All tests that used it now call `restorePromptSetForWork`.
- Mock gaps patched in tests: `promptSetChoiceGet`/`promptSetChoiceSet` added to the `config/promptset` mock in `promptSetState.test.ts` and `promptLibraryOverlayRender.test.ts` (the latter went red only in the full run); the two `library*ResetsAiPromptInspector` tests renamed their mock to `restorePromptSetForWork`.
- New `tests/frontend/aiPromptSetColumns.test.ts` mounts the real panel with 12 sets: `.select-options--cols-2` holds 13 radios.
- Counter-checks (real removals, restored after): deleting the `promptSetChoiceGet()` read in `loadPromptSets` ⇒ 5 red (matrix rows 1, 2, 4 + two); deleting the `void promptSetChoiceSet(...)` write ⇒ 4 red (write, unpick, write error, error cleared); deleting `:columns="2"` ⇒ the columns test red.
- Arrow-key traversal across both columns (AC 2 second half) is not tested in happy-dom (geometry); `selectMenu.test.ts` covers keyboard traversal of `columns` menus generally.

Row map (all in `tests/frontend/promptSetState.test.ts` unless noted; all passed in the final full run, vitest exit 0, 232 files / 3327 tests):

| Row / AC | Test | Passed |
|---|---|---|
| Nhớ qua khởi động lại / AC1 | "Nhớ qua khởi động lại: bộ đã lưu được chọn sẵn" | yes |
| Đè thể loại | "Đè thể loại: ... đã lưu "Thời sự"" | yes |
| Chưa từng chọn tay | "Chưa từng chọn tay: cột NULL ⇒ ... KHÔNG ghi" | yes |
| Mỗi Tác phẩm riêng | "Mỗi Tác phẩm riêng: đổi qua lại A/B" + "lựa chọn tay của A không rò sang B" | yes |
| Bộ đã lưu bị xoá | "Bộ đã lưu bị xoá: theo thể loại, không lỗi, không sửa tên" | yes |
| Bỏ chọn tay | "Bỏ chọn tay ... ghi null; lần mở sau theo thể loại" | yes |
| Lỗi ghi | "Lỗi ghi: báo qua promptSetChoiceError, lựa chọn trong phiên giữ, không thử lại" | yes |
| Match trim/case; saved = hand pick | "tên đã lưu khớp sau trim..." / "bộ đã lưu được chọn như lựa chọn tay" | yes |
| Only hand pick writes | "chọn tay ⇒ ghi đúng tên", "không có Tác phẩm ⇒ không ghi", "tên lạ ... không ghi" | yes |
| AC2 two columns | `tests/frontend/aiPromptSetColumns.test.ts` | yes |
| Open Work calls the new function | `libraryChaptersResetsAiPromptInspector` / `libraryImportResetsAiPromptInspector` | yes |
