# Handoff: Rust phase -> Webview phase (workspace-luoi-hai-cot-va-thanh-trang-thai)

Plan: `../story-workspace-luoi-hai-cot-va-thanh-trang-thai-plan.md`. Rust phase done (first Execution task), nothing committed.

## Wire (both plain `#[tauri::command]`, one `SELECT COUNT(*)` per tier summed; registered in `lib.rs` next to their list command)

| Command | Args (JS) | Returns |
|---|---|---|
| `tm_count_pairs` | none | `number` (u64) = global `tm_unit` rows + open Work's rows = `tm_list_pairs("all","both","").total_pairs` |
| `glossary_count_entries` | none | `number` (u64) = global `glossary_entry` rows + Work rows = `glossary_list_entries(...).len()` |

TS wrappers already exist (needed by `ipc_argument_contract`): `tmCountPairs()` in `src/config/tm.ts`, `countGlossaryEntries()` in `src/config/glossary.ts`. Both return `number | null`, never throw; `null` = unknown (command failed, bad shape, no bridge) so the status bar hides that item, never shows 0.

## Behaviour the webview can rely on

- Same set as the list commands for the same Work. No Work open: Global tier only, no error. A missing `global.db` is an error, so `null`.
- TM count is pairs, not source groups; a pair stored in both tiers counts twice (the listing does not dedupe). Glossary count includes shadowed Global rows and pending rows (`source_term` is UNIQUE per tier, so list length = global + work rows).
- Cheap (about 0.15 ms of SQL vs about 124 ms for the old full read on 12,409 pairs, measured by the coordinator): safe to call after every segment confirm and every Glossary/TM change.
- Unlike the list commands, the count does not decode rows, so an unknown stored origin is not an error here.

## Surprises

- Census in `config_invariants`: `commands/tm.rs` 3,6,6 -> 4,6,6; `commands/glossary.rs` 7,8,8 -> 8,8,8; tree total 89/60 -> 91/60. Plain commands, so no blocking-wire rows.
- Do not run bare `cargo fmt` (reformats four unrelated files); use `rustfmt --edition 2024 <file>`.

## Verified (how)

- `cargo test --test` green: `tm_contract` (146; cases `the_count_wire_equals_the_unfiltered_listing_across_both_tiers` and `the_same_pair_in_both_tiers_counts_twice_like_the_listing`), `glossary_commands_contract` (32), `config_invariants` (46), `ipc_argument_contract`, `ipc_contract`. `check:commands` green.
- Counter-check: dropping the Work term from `count_tier_pairs` (signature kept) turned both count cases red (`left: 2, right: 4` and `left: 1, right: 2`); restored, green.
- Full suite not run (`lib.rs` changed: once before push).

# Webview phase -> Tests phase

Webview phase done, nothing committed. Green: `vue-tsc --noEmit`, `check:commands`, `check:tokens`, `check:i18n`, `check:lint`. Rust untouched.

## GridPanel (`src/panels/GridPanel.vue`)

- Grid is four columns (`3px 30px 1fr 1fr`). `.col-state` / `.cell-state` and the DOM status text are gone; `STATE_LABEL_KEYS` stays and feeds only the accessible name.
- Translation cell `.cell-tgt` now carries `role="textbox"`, `aria-multiline="true"`, `aria-label` = `t('panel.grid.cell_label', {n, state})` -> "Câu 3, chưa dịch"; with a refused row `panel.grid.cell_label_error` -> "Câu 3, chưa dịch. <tError text>". `n` = 1-based row index (same number as the number column).
- Row error: `data-row-error="<tError text>"` attribute on `.cell-tgt` (absent when no error) + class `refused` on `.cell-tgt` (moved from `.cell-state`). It is drawn by `.cell-tgt[data-row-error]::after { content: attr(data-row-error) }`, NOT a DOM child: the write path reads `cell.textContent`, so a child element would corrupt the typed text. Consequence: the visible label is not in `textContent`/`innerText`; tests read the attribute or the aria-label, e2e `grid-row-error-label` must read `getAttribute('data-row-error')` (or computed `::after` content).
- Rules: `.rule.rule-<x>` blocks now use `border-left: <w> <style> var(--color-<x>)`: confirmed 2px solid, primary 3px solid, draft 2px dashed, tm-rule 2px dotted; `none` renders no element. Rule column still `aria-hidden`.
- Gate change (outside `src/`): `scripts/check-commands.mjs` Kiểm I accepted only `background-color: var(--color-x)` in each `.rule-x` block; it now accepts `background-color`, `border-color` or `border-left` carrying `var(--color-x)`. Without it dashed/dotted cannot exist (a background fills the dashes). Counter-check still owed: remove `border-left` style from one block and see Kiểm I / the new test go red.

## SourceHanViet / i18n

- `.hv-sources` line, `sourcesLine`, its CSS, the `dictSources`/`sourcesUsed` imports and key `panel.source.han_viet_sources_prefix` removed. `StatusBar.vue` comment that cited the key edited.
- New keys: `panel.grid.cell_label`, `panel.grid.cell_label_error`, `status.position`, `status.position_sentence_only`, `status.confirmed_counter`, `status.regroup_hint`, `status.proofread_counts`, `status.lookup_offline`, `status.glossary_count`, `status.tm_count`.

## StatusBar (`src/StatusBar.vue`)

`<footer class="status">` (no role now) holds three children:
1. `.status-info[data-status-info]`: `[data-status-position]` "Chương 47 · câu 6/184" (or `status.position_sentence_only` when the Chapter row is not in `libraryChapters`; hidden when no caret/last caret, or `!editorHasLoaded()`), `[data-status-confirmed]` "12/34 đã xác nhận" (confirmed = `isSegmentConfirmed` over `editorSegments`, total = `editorSegments.length`; hidden when 0 segments), `[data-status-hint]` "⌘M gộp · ⌘/ tách" (chords from `shortcutRows` display of `editor.merge_segments` / `editor.split_segment`; hidden if either is unbound), `[data-status-proofread]` "N chính tả · M nghi nghĩa" (only when `proofreadStateValue === 'done'`; spelling+grammar vs `isMeaningKind`).
2. `.status-live[role="status"][data-status-live]`: ONLY the notice chain (`.notice` spans, same priority and classes as before).
3. `.status-end[data-status-end]`: not retreated -> `[data-status-lookup-offline]`, `[data-status-glossary]` "Glossary N mục", `[data-status-tm]` "TM N cặp" (each hidden when the count is `null`), `[data-status-saved]` / `.saved` only when `workspaceLayoutTier === 'full'` and a save happened. Retreated (`lookupHasRetreated`) -> `[data-status-lookup-line]` (gloss of the first sense of the first source group, from `groupedLookup`/`sensesByLayer`; absent when none), `[data-status-lookup-chord]` (chord of `layout.lookup_drawer_open`), then the unchanged `.lookup-entry` button. No Glossary/TM/auto-save then.
- New module `src/workspaceCountsState.ts`: `workspaceTmPairCount`, `workspaceGlossaryEntryCount` (`number | null`), `refreshWorkspaceCounts()` (latest-call-wins; calls `tmCountPairs()` + `countGlossaryEntries()`).
- Refresh is called: StatusBar `onMounted`; `watch(editorChapterId)`; `confirmCurrentSegment` after a `'confirmed'` result (`editorPanelState.ts`); Glossary: `refreshGlossaryMarksAfterSave` (quick add), `refreshGlossaryMarksAfterMutation` (manage), confirm-strip save, glossary import confirm success; TM: `tmImportState` confirm success, `tmManageState` delete / delete-others / push-to-global success.
- `layout/lookupDrawerState.ts` gained `workspaceLayoutTier` (readonly ref set by `syncLayoutTier`) so the bar knows the wide tier; `null` until the dock measures, so the auto-save line is hidden then (and in unit tests that mount `StatusBar` alone).

## DESIGN.md

One 🔵 line under "Vạch lề segment" records the chosen shapes.

## Red tests (12 listed files: 6 failed / 16 tests; the other 6 files green)

- `statusBar.test.ts` (6): `.saved` is absent without `workspaceLayoutTier === 'full'` (set tier via `syncLayoutTier('full')`), and the "saved REPLACES the notice" tests are now wrong by design (saved sits outside the notice region; both can show). The empty-bar test sees "Tra cứu ngoại tuyến" (always shown when not retreated). Behavioural -> rewrite.
- `editorRegroupNotice.test.ts` `⑥` and `glossaryMarksRefresh.test.ts` P7: same reason (assert `.saved` after the notice ends; needs tier `full`).
- `gridPanelRowErrorPriority.test.ts` (6) and `gridPanelTmFilled.test.ts` (1): read `.col-state` / `.cell-state` text. Move to `.cell-tgt` `data-row-error` / `aria-label`; the TM test also checks the state label text (now in the aria-label) and `rule-tm-rule` / `rule-draft` classes (unchanged).
- `gridPanelImages.test.ts` (1): "five columns each with N children" -> now four.
- Green without change: `editorTmFilled`, `editorNavNotice`, `editorChapterSwitch`, `workspaceDockTier`, `lookupSensesFailed`, `editorSourceCut`. E2E specs not run.
- Still to write: cell accessible name per state, `getComputedStyle` of the three `.rule` differing in `border-left-style`/width, status bar `role=status` text unchanged on caret move, hint follows a rebound chord, counts `null` hides item, narrow lookup line, no "Nguồn:" line in Hán Việt, counter-checks (remove `border-left` style from one `.rule-*`; remove `refreshWorkspaceCounts()` from the confirm path).

## Surprises

- Error label cannot be a DOM child of the contenteditable cell (see above); pseudo-element via `attr()` was the only shape that leaves `textContent` clean.
- The existing Kiểm I regex tied `.rule-x` to `background-color`; widened (see above).
- StatusBar comments in the notice chain still say the notice "replaces" the saved line; no longer true (left, no mass cleanup).

## Webview phase correction (supersedes the StatusBar and red-test lines above)

- `.saved` / `data-status-saved` is back as the LAST `v-else-if` of the notice chain inside `.status-live[role=status]`, every tier, same priority as before.
- The Decision 6 hint is `[data-status-autosave-hint]` in `.status-end`, outside the live region, only when `workspaceLayoutTier === 'full'` (non-retreated branch). Key `status.autosave_hint` with `idle_seconds` / `cap_seconds` interpolated from `EDITOR_IDLE_MS` / `EDITOR_HARD_CAP_MS` (`panels/editorFlush.ts`).
- Red now: only `statusBar.test.ts` "KHÔNG hiện câu nào trước lượt flush đầu tiên" (bar text is "Tra cứu ngoại tuyến", not ''). `editorRegroupNotice` and `glossaryMarksRefresh` are green again with original expectations. Ignore the earlier claims that those need tier `full`.

# Tests phase result

## Changed
- vitest fixed: `gridPanelRowErrorPriority` (reads `.cell-tgt[data-row-error]`, plus one aria-label/`refused` assertion), `gridPanelTmFilled` (aria-label carries the TM state), `gridPanelImages` (four columns), `statusBar` ("nothing before first flush" now asserts the live region `[data-status-live]` is empty, still no `.saved`).
- vitest new: `gridPanelCellLabel` (3 states: no visible status text, aria-label, rule class per state), `statusBarWorkspace` (12 cases: position, untitled Chapter, caret move leaves the live region unchanged and only `[data-status-live]` has a role, rebound/unbound chord, null count hides item, 0 shows, confirm refreshes TM + counter, autosave hint outside live region and tier full only, narrow lookup line, narrow with no lookup), `sourceHanVietNoSourcesLine`.
- Gate: `scripts/check-commands.mjs` Kiểm I also requires every `.rule-*` block to declare width + line style in `border-left`, pairwise distinct. happy-dom does not apply scoped SFC CSS to `getComputedStyle` (measured), so the stroke AC is guarded by this gate + e2e computed style.
- e2e: `grid-row-error-label` (reads `data-row-error`, plus `::after` content and aria-label), `grid-image-row-alignment` (four columns), `hanviet-parallel-row-height` (four-column probe CSS), new `story-12-7-workspace-grid-status-bar` (computed border style of confirmed/draft/primary differ, four columns, no status text, status bar outside live region, counts). `segment-navigation`, `editor-typing-flush` unchanged and green.
- Fixes found by the gates: `check:panel-refs` was red (`workspaceCountsState.ts` had no reset; `lookupDrawerState.ts::currentTier` unexempted). Added `resetWorkspaceCounts()` (called from `resetEditorPanel`) and a named EXEMPT.

## Counter-checks (all restored afterwards)
- (a) `.rule-draft` `2px dashed` -> `2px`: `check:commands` FAIL "`.rule-draft` không khai độ dày và kiểu nét trong `border-left`"; `2px solid`: FAIL "`.rule-confirmed` và `.rule-draft` cùng nét `2px solid`".
- (b) deleted `void refreshWorkspaceCounts()` in `confirmCurrentSegment`: `statusBarWorkspace` red "expected 1 to be greater than 1".
- (c) `role="status"` back on `<footer>`: red "expected [footer, div] to have a length of 1 but got 2"; position span moved inside `.status-live`: red "expected 'Chương 47 · câu 3/3' to be 'Chương 47 · câu 1/3'".

## Environment trap
- A vite started 2026-10-13 (pid 38434) still listened on 1420; `test:e2e` reuses it, its HMR timestamps made `import()` return module copies, and 4 of 6 specs (3 untouched) went red. Killed it, re-ran: 6/6 green.

## Full run (once, at the end)
- First full vitest was red for reasons of this story: five tests mocked `config/tm` / `config/glossary` without the new count exports (28 unhandled rejections: `glossaryImportPreview`, `glossaryManage`, `glossaryQuickAdd`, `tmImport`, `tmManage`) and `glossaryIpcBridge` ②c did not know `countGlossaryEntries` (now a `COUNT_ADAPTERS` exemption plus a ②d group). After the fix: vitest 172 files / 2401 tests green; `npm run build` green; `cargo test --locked` green; gates deps tokens i18n commands layout panel-refs doc-refs dict dict-manifest lint clippy gates debt-owner green.
- `check:fmt` RED, not from this story: `cargo fmt --check` diffs in `core/library/indexer.rs`, `tests/ipc_contract.rs`, `tests/library_commands_contract.rs`, `tests/library_index_contract.rs` (none modified in this working tree; from 12-3/12-4 commits). Left alone.
- Two timeouts in the first full vitest (`settingsFrame`, `libraryChapterBulkWrites`) did not recur; green alone and in the second full run (machine load while cargo built).
- `npm run test:story 12-7 -- --list` selects only the 3 Rust targets and warns the plan declares no tests; the plan's Code Map "Tests that move" does not name the new test files.
