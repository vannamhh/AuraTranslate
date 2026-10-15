# Handoff: Glossary bảng chờ, Quản lý, màn duyệt thu hoạch (ticket 13)

Plan: `story-glossary-bang-cho-quan-ly-man-duyet-thu-hoach-plan.md`. Working notes only; not copied into the spec.

## Phase 1 (Webview: bảng chờ + duyệt thu hoạch + menu + lối mở Workspace) — done, not committed

### Changed
- `src/glossaryQueueState.ts`: row gets `left`; `queueSessionCounts` (accepted / rejected / left / remaining / total), `queuePaceSeconds` (null until 2 decisions; mean gap between decision stamps), `leaveGlossaryQueueCandidate()` (no write, advances to next undecided row), cursor memory `rememberedCandidateId` (kept across close/open, cleared only by `resetGlossaryQueue`; set on close only when the cursor row is undecided, otherwise null so the next open starts at row 0). Accept already called `refreshWorkspaceCounts()`; unchanged.
- `src/GlossaryQueueOverlay.vue`: subtitle with threshold, progress bar + legend + pace line, rows (term · category of the session · "N lần" · peek), expanded card for the cursor row (big term, "xuất hiện N lần · trong K Chương" in `.gq-chapter-span`, "i / n", category chips with digits, one context, FR114 block when no Hán Việt and no harvest proposal), key buttons N / B / Space / ↑ / ↓, Esc sentence, two footer sentences. `Space` handled in the existing root `onKeydown` (skips when the target is a `<button>`), dispatches `glossary.queue.leave`. Margin bars: `gq-row-current` 3px solid primary, accepted 2px solid confirmed, rejected 2px solid ornament, left 2px dashed draft; status always has text (sr-only).
- New `src/glossaryHarvestState.ts`, `src/GlossaryHarvestOverlay.vue` (mounted in `App.vue`): list filtered to `candidate_origin === 'review_harvest'`, checkbox pre-selected by `!isInconsistentHarvest`, per-row Nhận N / Bỏ B act on the cursor row (row `@mousedown`/`@focusin` aims the cursor), "Nhận N mục đã chọn" is sequential `approveGlossaryCandidate` (category `'other'`, translation = `proposed_translation`), a failed row keeps its tick and shows its own error, the batch continues. "Để lại trong bảng chờ" and Esc = `glossary.harvest.close`, no write.
- Commands (no chord): `glossary.queue.leave`, `glossary.harvest.{open,close,accept,reject,next,prev,accept_selected}`. Wired in `src/commands/index.ts` (CommandDeps + register), `src/main.ts` (deps + `isBlocked` with `harvestOverlayIsOpen`), `src/TitlebarToolsMenu.vue` (item after Nối tay, `data-glossary-harvest-open` on the trigger), `scripts/check-commands.mjs` `HANDLER_TABLE` (queue `onKeydown` + `leave`; harvest `trapTab` / `aimRow` / `onKeydown`; `StatusBar.vue::focusOnPointerDown`).
- `src/glossarySettingsState.ts`: new `glossaryScanThresholdInUse` (null when `configError` is set; else the saved-this-session value or the bootstrap one). The queue subtitle uses it and drops the number when null.
- Workspace opener (Sally role): `src/StatusBar.vue` button `data-glossary-queue-open`, `@click="dispatch('glossary.queue.open')"`, label = `command.glossary.queue.open`. Placed at the right end of the status bar next to the Glossary count it belongs to, outside the `!lookupHasRetreated` branch so it stays reachable on narrow tiers; always shown (queue explains its own emptiness). No pending-count on it: no cheap count exists on the wire and `0` would be a lie, so the number is absent, not hidden.
- `src/i18n/vi.json`: new `glossary.queue.*`, `glossary.harvest.*`, `command.glossary.queue.leave`, `command.glossary.harvest.*`; removed the now unused `glossary.queue.chapter_span_count`. `check:i18n` forbids "bạn" in values.
- `scripts/check-tokens.mjs`: `COMPONENT_FILE_FLOOR` 152 -> 164 (the gate was already red on HEAD: 192 live components, floor under 80%; its own message asked for 164).

### Tests touched
- `tests/frontend/titlebarToolsMenu.test.ts`: seven -> eight items / hooks.
- `tests/frontend/glossaryQueue.test.ts`: chapter-span expectations use `card_meta`; new describe for Space/leave, cursor memory, pace.
- New `tests/frontend/glossaryHarvest.test.ts` (state + overlay, 8 cases). The Tests phase should still add: counter-check (remove the `!isInconsistentHarvest` pre-select / the sequential loop and see red), StatusBar opener + focus return, queue progress legend, `review_harvest_contract.rs` case, e2e `story-12-13-glossary-harvest.e2e.mjs`.

### Verified
`npm run check:commands`, `check:tokens`, `check:i18n`, `check:panel-refs`, `vue-tsc --noEmit -p tsconfig.json`, eslint on `src` + touched tests: green. vitest: glossaryQueue, glossaryHarvest, titlebarToolsMenu, reviewerImportFocus, statusBar, statusBarWorkspace, workspaceDockTier, panelTabChord, settingsFrame green (statusBar* timed out at 5 s under load average ~35 and passed with `--testTimeout=60000`). Not run: cargo, e2e, full suite.

### Open risks / notes for next phases
- Cursor memory is keyed by candidate id only; ids are per-project, and `resetGlossaryQueue` has no caller on a Work change, so a remembered id from another Work could land the cursor on an unrelated row. Low impact (cursor only).
- Pace denominator: `queueUnprocessedCount` (includes rows marked left).
- Hán Việt `ok` rows show no FR114 block; harvest rows with a proposal also show none.
- The queue still shows harvest rows (deliberate, plan); `GlossaryHarvestOverlay` does not refresh the queue (it reloads on open).
- Space on the hidden row radios: root handler prevents default, so the radio does not toggle; verify in a real webview (manual debt item `Chủ: Epic 12`).
- Phase 2 (Quản lý Glossary, `GlossaryManageOverlay.vue` / `glossaryManageState.ts`, c:34–c:50, C5, C6) is untouched. Reuse `GlossaryConfirmStrip.vue:105` origin mapping; margin-bar shapes per plan Design Notes (pending = `draft` 2px dashed, Work-confirmed = 2px solid, Global-confirmed = none).
- Implementation Notes of the plan are still empty; the Sally placement reasoning above belongs there in the last phase.

## Phase 2 (Webview: Quản lý Glossary) — done, not committed

### Changed
- `src/glossaryManageState.ts`: `tierFilter` (`global` | `work` | `both`, default `both`, reset on open/reset), `manageTierCounts` (per tier, all OTHER filters applied, so the number equals the table rows at that nấc), `managePendingCount` (tier-scoped only), `setGlossaryManageTierFilter`, `setGlossaryManageCursor` (moves + discards open edit, like next/prev), shadowed Global rows re-placed directly under the Work row that shadows them (`placeShadowedBelowWinner`, after filter and sort; no-op when the winner is filtered out).
- `src/GlossaryManageOverlay.vue`: tier segmented control with counts (Work disabled and count hidden while no Work is open or not loaded), category pills, pending pill "Chờ chốt · N" (checkbox -> `confirmed` filter `pending`/`all`), origin kept as a `<select>`, sort label "Sắp theo: <current>", `<table role=grid>` with seven columns (`.gm-list`, rows `<tr class=gm-row>`; `role=option` gone), margin bar classes `gm-row-bar-pending` (2px dashed draft) / `gm-row-bar-work` (2px solid confirmed) / none for confirmed Global, sr-only status text per row, explainer boxes (pending, shadowed), "Thêm thuật ngữ" + hint in `.gm-foot`. In-place edit: translation / category / note inputs render in the cursor row with `form="gm-edit-form"`; the form below keeps only Lưu/Huỷ. Row `@mousedown` aims the cursor, `@dblclick` dispatches `glossary.manage.edit`, row `@keydown.esc` (`onRowEscape`) cancels the edit only (else bubbles to close). Edit start focuses `[data-gm-edit-translation]`; edit end refocuses the table if focus left the panel. Filters/tier/pills use `@change`, not `@click`.
- New command `glossary.manage.add_term` (no chord): dep `addGlossaryTermFromManage` in `src/commands/index.ts` + `src/main.ts` (closes Manage, then `nextTick` -> `openGlossaryQuickAdd('')`, because the quick-add strip sits in the Workspace under the full-screen overlay and would be invisible otherwise).
- `src/GlossaryImportOverlay.vue`: C6 — heading `glossary.import.read_title` ("Đọc được gì trong file") above the existing file / rows / ignored-columns summary; no data change.
- `src/i18n/vi.json`: `glossary.manage.{tier_*,category_all,pending_pill*,sort_prefix,sort_current_*,col_*,status_*,add_term,foot_hint,pending_box_*,shadow_box_*}`, `command.glossary.manage.add_term`, `glossary.import.read_title`; `glossary.manage.pending_badge` text now "chờ chốt bản dịch".
- `scripts/check-commands.mjs`: `HANDLER_TABLE` rows `GlossaryManageOverlay.vue::aimRow` (cursor-local) and `::onRowEscape` (`glossary.manage.cancel`).
- `tests/frontend/glossaryManage.test.ts`: `li` selectors -> `tbody tr`; new describe with 6 cases (tier counts == rows per nấc, shadowed under winner, margin-bar classes + status text, origin + date, dblclick edit in row, add_term dep).

### Decisions
- Default tier is `both` (keeps backend order and existing cases), not Work as in the mockup.
- Category `<select>` and status `<select>` replaced by pills; the `confirmed` filter value of the state type is now unreachable from UI (state kept, tests use it).
- "Thêm ngày" = `created_at` sliced to `DD-MM-YYYY` (no timezone conversion); unparseable -> "—". "Dùng" column stays a bare number (existing `.gm-count` test), not `N×`.
- Mockup's reading line under the term (Hán Việt) and "reviewer đổi ở n/m lần" are not on the wire; omitted.

### Verified
`check:commands`, `check:tokens`, `check:i18n`, `check:panel-refs`, `vue-tsc`, eslint on touched src: green. vitest: glossaryManage (50), glossaryImportPreview, commandsRegistry, commandsRegistryPortMissing green. Not run: cargo, e2e, full suite, `npm run test:story 12-13` as a whole.

### Open risks / left for the Tests phase
- Counter-checks not done: remove `placeShadowedBelowWinner` call; drop the tier clause from `matchesFilters` (counts must go red); make `marginBarClass` return one class for all rows. Border-left STYLE is only asserted via classes in jsdom (scoped CSS not computed); the dashed vs solid difference needs the real webview (debt `Chủ: Epic 12`, with light/dark distinguishability).
- `form="gm-edit-form"` association and Enter-to-save from the in-row inputs are not exercised in jsdom; check in webview. Esc inside a row input relies on `onRowEscape`; no test covers it.
- `addGlossaryTermFromManage` returns early-ish: if `closeGlossaryManage` is blocked by `saving`, quick-add still opens beneath the overlay. Edge only.
- Table keyboard: ArrowUp/Down still move the cursor from the focused `<table>`; screen-reader behaviour of `role=grid` + `aria-activedescendant` is untested.
- Pending-count scope is tier only (ignores search/category/origin); the pill title says so.
- No spec under `e2e/` selects `.gm-list`, `.gm-row` or the old filter selects (grep empty).

## Phase 3 (Tests that move) — done, not committed

- Added: Rust case `approving_two_of_three_harvest_candidates_...` in `review_harvest_contract.rs` (18/18 green); `tests/frontend/glossaryQueueOpener.test.ts`, `glossaryQueueResetWiring.test.ts` (new); cases in `glossaryQueue.test.ts` (legend), `glossaryHarvest.test.ts` (Esc focus return, sequential), `glossaryManage.test.ts` (tier counts with another filter); e2e `story-12-13-glossary-harvest.e2e.mjs` (not run) and `titlebar-tools.e2e.mjs` updated to eight items.
- Code: `resetGlossaryQueue()` now called from `libraryImport.ts::finishSubmit`; `GlossaryManageConfirmedFilter` is `'all' | 'pending'`.
- Counter-checks (real removal, that file's target only, restored): harvest pre-select -> 4 red; stop-on-failure in the batch -> 1 red; batch in parallel (`Promise.all`) -> first GREEN, so the "next id waits" case was added, then red; shadow placement call removed -> 3 red; tier counts ignoring other filters -> first GREEN, so the "with another filter" case was added, then red; tier clause dropped from `matchesFilters` -> 2 red; one margin-bar class -> 1 red; `resetGlossaryQueue()` removed from `finishSubmit` -> 1 red; opener `dispatch` removed -> 1 red.
- Not counter-checked: Space handler in the queue root (covered only through the dep spy), `onRowEscape` (no test).
