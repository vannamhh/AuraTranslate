# Handoff: phase 1 (Rust + pinyin) to phase 2 (webview)

Plan: `../story-panel-tra-cuu-popup-nguon-au-khoi-gon-am-oc-chip-lich-su-plan.md`. Nothing committed; no Vue file touched.

## Wire shape
`EntryHit` (Rust `core/dict/mod.rs`, TS `src/config/dict.ts`) gained `reading: string | null` (`dict_entry.reading` verbatim, `None`/`null` when the source has none). Set in `query.rs` `COLUMNS` + `row_to_hit`, so all branches (exact, char_idx, fts, en) carry it. Raw forms seen in data: numeric (`da3`, `zou3 lang2`), toned (`dǎ`), `dǎ (da³)`, NULL.

## `src/panels/pinyin.ts`
- `toneMarkedReading(raw: string): string` numeric pinyin to tone marks per whitespace token (`lu:4` -> `lǜ`, `r5` -> `r`); anything else verbatim (keeps `shān (shan¹)` as-is, trailing paren included).
- `readingKey(raw: string): string` compare key (tone-marked, trailing `(...)` dropped, whitespace dropped, lowercase, NFC). `zou3 lang2` and `zǒuláng` -> `zǒuláng`.
- `sharedReadingKey(readings: readonly (string | null)[]): string | null` Decision 2 helper: key shared by all non-null readings, else `null` (also `null` when none). Use it for the panel-head reading; display the key itself (`zǒuláng`).
- Block-level group reading: show `toneMarkedReading(hit.reading)`.

## Surprises
- Spec head example says `zǒu láng` (Intent matrix) but Decision 2 says display the key (`zǒuláng`); I followed Decision 2 in the helper. Webview agent: decide display, the key has no space.
- Only two TS fixtures built `EntryHit` literals (`lookupSensesFailed.test.ts`, `statusBarWorkspace.test.ts`); both updated with `reading: null`. Other fixtures in "tests that move" not checked for this.

## Verified
- `cargo test --test dict_lookup` (40 pass, incl. new `hits_carry_the_raw_reading_and_none_when_the_source_has_none`: 打 `da2`,`da3` in one source, `None` in another, exact and char_idx branches); `--test ipc_contract --test dict_sources` pass; `cargo fmt --check` clean.
- Test uses a fixture DB, not the real CVDICT (plan phrase "CVDICT" is represented by `fixture-alpha`).
- `npx vitest run tests/frontend/pinyin.test.ts` (new) + the two edited fixtures tests pass; `vue-tsc --noEmit` and eslint on new files clean.
- Not run: full suite, `npm run build`, counter-check (removal) of the Rust seam.

# Webview phase -> Tests phase

Uncommitted. All Execution tasks done except "Tests".

## DOM / selectors
- Sources popup is a new component `src/panels/LookupSourcesMenu.vue` (modelled on `TitlebarToolsMenu.vue`). Trigger `button[data-sources-trigger]` text `Nguồn N/M ▾` (`aria-haspopup=menu`, `aria-expanded`). List `ul[role=menu]`; rows `button[role=menuitemcheckbox][data-source-code]` with `aria-checked`, `.off` when disabled; count span `.sources-count` (number, or `tắt` for off; absent when unknown). Esc closes + focuses trigger; ArrowUp/Down move focus; Space = native button click -> `lookup.toggle_source` (list stays open). Only `display_name` exists (no short name), so a row shows name + count.
- Tab row: `.lookup-tabrow` holds the tablist and `.lookup-tools` (menu + `button[data-attribution-open].lookup-sources-attr`). `.lookup-sources`, `.source-chip`, `.lookup-spine-chip`, `.lookup-disagree` are gone.
- Head: `.lookup-head-row` > `.lookup-headword`, `.lookup-head-reading` (shared key, e.g. `zǒuláng`, only when `sharedReadingKey` non-null), `button.lookup-pin.lookup-head-pin` (outside any `[data-entry-key]`; `@mousedown=aimLookupEntryFrom` on `.lookup-head` clears the aim). Spine now only `.lookup-spine-count` "N nguồn · M nghĩa".
- Block (`LookupRecord.vue`): `.lookup-source-head` = `.lookup-source-name`, `.lookup-source-count` (`N nghĩa` / `ít nhất N nghĩa` when `total_entries !== null`; absent when senses failed), `button.lookup-collapse[data-collapse-code][aria-expanded]` (`⌄` open, `⌃` collapsed). Collapsed hides all `.lookup-entry`. Cluster head: `.lookup-entry-headword` (2+ clusters), `.lookup-entry-reading` (tone-marked) + `.lookup-entry-count` only when `reading !== null`; pin button unchanged.
- Not found: `.lookup-empty-title` "Không nguồn nào có {q}", `.lookup-empty` "Đã tra N nguồn đang bật.", `.lookup-chars` > `button.lookup-char[data-lookup-char]` (multi-char queries only, distinct chars), then the Concordance pointer. Never looked up: `.lookup-empty` with the real chord (`panel.lookup.status_chord`); PanelFrame `.status` is now off (`:show-status="false"`).
- History: chips `button.lookup-scope[aria-pressed]` (`Phiên này`, `Chương này`; the second is `disabled` when `lookupChapterId === null`); rows from `visibleLookupHistory`; `panel.lookup.history_chapter_empty` when scope filters everything.
- Tab title: `PanelTab.vue` adds `span.tab-chord` (formatted chord of `lookup.lookup_selection`) next to "Tra cứu".

## New command ids (no default keys, all in `src/commands/index.ts`, deps injected in `main.ts`)
`lookup.toggle_sources_menu` · `lookup.toggle_source_collapse` (aim: `@mousedown` on `[data-collapse-code]`, else focused element) · `lookup.lookup_char` (aim: `[data-lookup-char]`) · `lookup.history_scope_session` · `lookup.history_scope_chapter`. Labels `command.lookup.*` in `vi.json`.

## State exports / resets
- `dictSourcesState.ts`: `sourcesMenuIsOpen`, `toggleSourcesMenu`, `closeSourcesMenu`; reset in `resetDictSources`.
- `lookupPanelState.ts`: `sourceEntryCount(grouped, source, off)` (null = unknown: off / skipped layer / in `hidden_sources` / `total_entries !== null` / nothing looked up; 0 only when on, not cut, no group), `singleCharsOf`, `sourceIsCollapsed`, `aimSourceCollapseFrom`, `toggleAimedSourceCollapse`, `aimLookupCharFrom`, `lookupAimedChar`; `collapsed` set and aims cleared in `resetLookupPanel`. `sourcesDisagree` deleted.
- `lookupHistoryState.ts`: `LookupHistoryEntry.chapters: readonly number[]`; `lookupChapterId`, `lookupHistoryScope` (effective: falls back to `session` with no Chapter), `visibleLookupHistory`, `selectLookupHistoryScope`, `setLookupChapter`; `scope` + `openChapter` reset in `resetLookupHistory`. Chapter id is pushed in by `editorPanelState.ts` (`watch(chapterId, setLookupChapter, {flush:'sync'})`) because importing `editorChapterId` into the lookup state would create a cycle. `recordLookup` merges the open Chapter into the row's `chapters` (that is the seam for the red counter-check; keep the `mergeChapter(...)` call's signature).

## Gate exemptions added
`scripts/check-commands.mjs` HANDLER_TABLE: `LookupSourcesMenu.vue::{aimDictSourceFrom,focusOnPointerDown,onKeydown}`, `LookupPanel.vue::aimLookupCharFrom`, `LookupRecord.vue::aimSourceCollapseFrom`; removed `LookupPanel.vue::aimDictSourceFrom`. No `check-panel-refs` exemption needed.

## i18n
Removed `panel.lookup.{sources_disagree,sources_label,source_toggle_hint}`. Changed `not_found` (now `{query}`), `all_sources_off`. Added `status_chord`, `not_found_checked`, `lookup_each`, `sources_button`, `sources_menu`, `source_off`, `block_senses(_at_least)`, `collapse_*`, `expand_label`, `scope_*`, `history_chapter_empty`, `head_pin_hint`.

## Surprises
- Head reading shows the key itself (`zǒuláng`), per Decision 2.
- `computeSpine` still returns `sources`/`hidden` (unused by the panel now); I did not trim it, to leave its tests alone.
- Popup row has no "tên ngắn" (no such field on `SourceAttribution`).
- Not verified visually in a real window (no Tauri run); popup uses `position:absolute; z-index:20` like the Tools menu and could be clipped by an ancestor `overflow` if PanelFrame has one: e2e should look.

## Verified
`vue-tsc --noEmit`, `eslint src e2e tests`, `npm run build`, `check:{tokens,i18n,commands,panel-refs,layout,doc-refs,gates,debt-owner}` all green. Ran only the 11 vitest files touching these modules: 117 pass, 2 expected red (below). No full suite, no e2e, no counter-checks.

## Expected red / to rewrite
- `lookupSourceChipToggle.test.ts` (chip `.off` class no longer exists; rewrite against the popup rows and the `Nguồn N/M` label).
- `tmConcordance.test.ts` "shows nothing extra when the count is zero" (asserts the old `Không tìm thấy trong từ điển` text; now `Không nguồn nào có {q}`).
- Likely also: `attributionOverlayActions` (it passed here, but check it still focuses `[data-attribution-open]`), `editorAutoLookup`, `titlebarToolsMenu` (passed), `e2e/specs/attribution-focus.e2e.mjs` (chip strip gone; trigger now in the tab row). Rust tests untouched by this phase.
- To add: tests for `sourceEntryCount`, `singleCharsOf`, collapse toggling, history scope + chapter filter (+ the removal counter-check on `mergeChapter`), `Esc` -> trigger focus (counter-check: remove `trigger.value?.focus()`), PanelTab chord.

# Tests phase -> review

Uncommitted (the rename of `lookupSourceChipToggle` is staged by `git mv`).

## Changed / added
- `computeSpine` now returns only `{sourceCount, senseCount, senseCountAtLeast}`; `SpineSource`/`SpineHiddenSource` and the `sources`/`hidden` outputs are gone (no test read them).
- Renamed + rewritten: `tests/frontend/lookupSourcesPopup.test.ts` (was `lookupSourceChipToggle`). Fixed: `tmConcordance.test.ts` (new not-found text). Extended: `editorChapterSwitch.test.ts` (editor Chapter reaches `lookupChapterId`).
- New: `lookupPanelBlocks.test.ts`, `lookupHistoryChapter.test.ts`, `panelTabChord.test.ts`, `support/lookupFixtures.ts`, `e2e/specs/story-12-8-lookup-sources-popup.e2e.mjs`.
- `e2e/specs/attribution-focus.e2e.mjs`: dropped the 0-source skip (the opener now always renders in the tab row). `attributionOverlayActions`, `editorAutoLookup`, `titlebarToolsMenu` needed no change.
- Plan Code Map "Tests that move" line updated so `test:story 12-8` stops warning.

## Matrix row -> test
- Tra 打 (no lead-in, `name · N nghĩa · ⌄`, `dá · 1` / `dǎ · 2`): `lookupPanelBlocks` "stacks one block per source…", "every block opens with…", "a source with readings groups…".
- Popup `↓↓ Space`, Esc, rollback: `lookupSourcesPopup` "ArrowDown twice then Space…", "Escape closes…", "a failed config write rolls…" (Space = native click, simulated with `trigger('click')`).
- Count unknown/"tắt": `lookupSourcesPopup` "an off source shows…" + `sourceEntryCount` describe.
- Source without reading: `lookupPanelBlocks` "a source without readings…".
- Chip `Chương này` (incl. A again in Chapter 2, no Chapter open): `lookupHistoryChapter` (all of "Chương này chip" + "chips in the history tab").
- Chưa tra (real chord): `lookupPanelBlocks` "never looked up…".
- Không kết quả (title, N, "Tra riêng" click -> lookup 鐵, one char, Concordance line, all sources off): `lookupPanelBlocks` "empty states" describe + `singleCharsOf`.
- Ghim (1 entry / 7 entries / group button): `lookupPanelBlocks` "Pin" describe.
- Âm đầu panel (`zǒuláng` / none for 打): `lookupPanelBlocks` "reading beside the headword".
- AC 4 (rebind chord): `panelTabChord` "rebinding…". AC 3: `lookupHistoryChapter`. AC 1/2: above + e2e.

## Counter-checks (real removal, restored byte-identical)
- Esc: deleted `trigger.value?.focus()` in `LookupSourcesMenu.vue::onKeydown`; `lookupSourcesPopup` "Escape closes the list and puts focus on…" red: `expected <body>… to be <button …>` (focus fell to body). Other 7 cases green.
- Chapter record: made `mergeChapter` in `lookupHistoryState.ts` return `known` (call signature kept); red: `expected [] to deeply equal ['B']` in "A in Chapter 1, B in Chapter 2…" and "clicking Chương này filters…"; the null-Chapter cases stayed green (5/7 pass).

## Real defect found by the e2e (production fix)
The source list was taller than the Lookup panel with the 9 real sources (list 113..488 px, panel body ends 444): its last rows sat under the "Đề xuất AI" tab (`elementFromPoint` -> `span.tab`). Fix in `LookupSourcesMenu.vue`: on open, `max-height` = room down to `.lookup-body` bottom, plus `overflow-y: auto`. The e2e now asserts the list bottom is within the panel and visible rows are not covered. vitest/happy-dom has no layout, so only the e2e guards this.

## Results
- `npm run test:story 12-8` green (Rust `dict_lookup` 40 pass; 9 vitest files, 121 tests). Also green: attributionOverlayActions, attributionOverlayUnreadable, editorAutoLookup, titlebarToolsMenu. `vue-tsc`, `eslint src e2e tests`, `check:{commands,panel-refs,i18n,tokens,layout}` green. No full suite run.
- e2e (macOS WKWebView, debug build, 9 dict sources): `story-12-8-lookup-sources-popup` and `attribution-focus` pass.

## Not covered
- Head reading is `zǒuláng` (Decision 2), not `zǒu láng` as the Intent matrix row reads; the test asserts the key.
- Dictionary-backed block rendering (打 against real CVDICT) is fixture-only; e2e does not run a lookup (no selection helper).
- Space-key activation of a popup row and VoiceOver reading (debt `Chủ: Epic 12`); popup look in the packaged app.
- The tab-title chord is checked in e2e only as non-empty; its rebind is vitest only.
- Full suite / CI Windows half not run.
