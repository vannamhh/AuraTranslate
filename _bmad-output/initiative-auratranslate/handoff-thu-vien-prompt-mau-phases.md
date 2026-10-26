
## Phase 1 (Webview)

Done: all `src/` tasks. No test files written or changed.

Exports
- `src/modes/libraryWorkCard.ts`: `GENRE_IDS`, `GenreId` now exported.
- `src/promptTemplates.ts`: `PROMPT_TEMPLATES: Record<GenreId, string>` (29 bodies, frame = guidance + glossary + TM + source; no "chỉ trả về bản dịch").
- `src/promptSetState.ts`: `promptTemplateExists(tier, id)`; `addPromptTemplates(tier, ids): Promise<number>` (returns created count; skips existing, skips `prompt_set.name_taken`, stops at other error with `actionError` set, no rollback; reloads once only if created>0 or error; re-applies `lastGenre` via `pendingGenre`; returns 0 if `busy`). New module var `lastGenre` (set by `autoSelectPromptSetForGenre`, cleared by `resetPromptSets`).
- `src/PromptTemplatePicker.vue`: no props, emits `close`; reads state directly.

DOM hooks (picker): root `[data-prompt-template-picker]`; rows `li[data-template-id=<genreId>]`; checkbox `[data-template-check]`; badge `[data-template-exists]`; preview button `[data-template-preview]`; preview body `[data-template-preview-body]` (`pre`); tier radios `[data-template-tier=global|work]` (work radio only when `promptSetWorkTierAvailable`); `[data-template-pick-missing]`, `[data-template-add]` (label "Thêm N bộ"), `[data-template-close]` (handler `onCloseTemplates`, emits `close`); `[data-template-all-present]`, `[data-template-added]`. Existing rows are checkbox-disabled. Overlay opener: `[data-prompt-template-open]` (form submit, `onOpenTemplates`); picker replaces the editor pane (`templateOpen` ref); Escape closes it before the overlay.

i18n keys added (vi.json, after `prompt.library.new_set`): `prompt.library.template_open|title|hint|exists_badge|preview_button|preview_heading({name})|pick_missing|add_button({count})|close|all_present|added({count})`.

Green: `vue-tsc --noEmit`, `check:i18n`, `check:tokens`, `check:commands`, `check:lint`; vitest promptSetState + promptLibraryOverlayRender + libraryWorkCard files (81 tests).
Red: nothing caused by this phase. `check:panel-refs` fails on `src/aiConfigState.ts:162 defaultSystemPrompt` (pre-existing from the system-prompt plan, not touched here; my new `lastGenre` is reset in `resetPromptSets`).

Phase 2: write `promptTemplates.test.ts`, extend `promptSetState.test.ts` (I/O matrix cases), `promptTemplatePicker.test.ts`. The picker uses global state, so mock `./config/promptset` as other state tests do and `resetPromptSets()` between cases.

Update: `check:commands` HANDLER_TABLE in `scripts/check-commands.mjs` now lists `onOpenTemplates` and the picker's `onPreview`, `onPickAllMissing`, `onSubmitAdd`, `onCloseTemplates` as `R_SUBMIT_DIRECT`; the inline `emit('close')` became the named `onCloseTemplates`. `check:commands` green.

## Phase 2 (Tests)

Files: new `tests/frontend/promptTemplates.test.ts`, `tests/frontend/promptTemplatePicker.test.ts`; extended `tests/frontend/promptSetState.test.ts` (describe `addPromptTemplates`).

Production defect found by a test and fixed (`src/promptSetState.ts`, `addPromptTemplates`): `applyGenre` lets a matching set override a hand pick, so re-applying `lastGenre` after adding a template switched away from a hand-picked "X" (matrix row "Ice đã chọn tay"). Now the re-apply runs only when `selectedSetName === null || selectionIsAutomatic`; otherwise `pendingGenre = null`.

Row map (all passed; 4 files, 89 tests incl. promptLibraryOverlayRender):
- Row Thêm mẫu -> promptSetState "Thêm mẫu: tạo bộ mang nhãn thể loại và thân mẫu ở tầng đích" -> passed
- Row Đã có trùng tên -> "Đã có bộ trùng tên khác hoa thường/khoảng trắng…", "bộ Tác phẩm che bộ Toàn cục…", "name_taken ngoài dự kiến…" ; picker "mẫu đã có hiện Đã có và bị khoá" -> passed
- Row Tác phẩm đang mở -> "Tác phẩm đang mở, chưa chọn bộ: thêm mẫu cùng thể loại thì tự chọn ngay" -> passed
- Row Ice đã chọn tay -> "Ice đã chọn tay: thêm mẫu trùng thể loại giữ nguyên bộ đã chọn" -> passed
- Row Lỗi ghi -> "Lỗi ghi: báo lỗi, dừng, giữ các bộ đã tạo và vẫn nạp lại một lần" (+ picker shows tError via actionError, not asserted in DOM) -> passed
- AC1 (29 mẫu, xem thân) -> picker "liệt kê 29 mẫu…"; "Thư viện prompt có hàng mở bộ chọn mẫu" -> passed
- AC2 (Pháp lý Đang dùng) -> state "Đã thêm mẫu Pháp lý ở Toàn cục…" (selected set; the overlay "Đang dùng" line itself is rendered from selectedPromptSet and covered by existing overlay tests) -> passed
- AC3 (29 Đã có) -> picker "đã có cả 29 mẫu…" -> passed
- Template content (3 vars, no unknown markers, 29 keys, no "chỉ trả về bản dịch") -> promptTemplates.test.ts -> passed
- Picker: count N, pick-missing 28, tier work hidden/shown -> passed

Counter-checks (real removal in src/promptSetState.ts, restored after; target = the 3 new/extended test files):
- case-insensitive compare removed from `promptTemplateExists` -> 2 red (state "Đã có bộ trùng tên…", picker "Đã có")
- re-apply of lastGenre removed -> 1 red ("Tác phẩm đang mở…")
- hand-pick guard removed -> 1 red ("Ice đã chọn tay…")

Also green: vue-tsc, check:lint, check:i18n, check:tokens. Not run: full suite, e2e, manual in-app check (Verification "Manual checks"). Picker test note: 28 sequential creates need `vi.waitFor`, `flushPromises` alone stops at 16.
