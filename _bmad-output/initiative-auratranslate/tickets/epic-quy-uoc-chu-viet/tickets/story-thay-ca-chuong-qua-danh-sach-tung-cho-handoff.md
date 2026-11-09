# Handoff: Rust phase done, for the Webview phase

Working notes only. Spec is the plan beside this file.

## Commands (registered in `lib.rs`)

### `convention_replace_chapter` (async)
- Args: `{ groups: ConventionReplaceGroup[] }` (camelCase for the top-level arg only).
- Nested items are **snake_case** (no serde rename, like `SegmentTargetEdit`):
  `group = { segment_id, expected_text, spots: [{ start, end, expected_phrase, rule_key }] }`
  `start`/`end` UTF-16 `[start,end)` of the scan's span; `rule_key` from the scan span; no replacement is sent.
- Spots of a group must be ascending by `start`, non-overlapping; a group with no spots is refused (`convention.range_invalid`). Send only selected spots; send no group for a sentence with none selected.
- Success (`ConventionReplaceOutcome`, snake_case): `{ outcomes: [{ segment_id, target_text, translation_origin, status, version_created, spans: [{ start, end, phrase }] }], replaced, sentences, confirmed, drafts, chapter_started }`.
  `spans` = inserted phrases at their new UTF-16 positions in `target_text`. `replaced` counts spots, `sentences` sentences, `confirmed`/`drafts` sentences by status.
  Empty `groups` returns zeros without a transaction.

### `convention_scan_chapter_progress` (async)
- Args: `{ generation, progress: Channel<{ done, total }> }`. Result is the same `ConventionScanWire` as `convention_scan_chapter` (unchanged, still there).
- Progress is sent every 250 sentences and once at the end (`done == total`; also `0/0` for an empty Chapter). `total` counts sentences with non-blank target text (omitted ones are in `not_scanned`, not in `total`). A closed channel is ignored.
- Rust requires params `generation` and `progress` (ipc_argument_contract checks key sets).

## Errors (IpcError `code` / `message_key` / params)
- `convention.chapter_changed` / `err.convention.chapter_changed` / `segment_ids`: a **comma+space joined string of segment ids** (`"12, 15"`), every sentence whose stored text differs from `expected_text` or that was omitted since the scan. Map ids to `Câu {ord}` via `editorSegments`. 0 rows written.
- `convention.rule_changed` / `err.convention.rule_changed` / no params: a rule is gone, disabled, has no replacement, or its key does not match the spot's phrase. Raised before the Work lock for gone/disabled.
- `convention.empty_result` / `err.convention.empty_result` / `segment_id`. Not reachable from real data today: a DB CHECK forbids a blank `replacement_phrase`, so no contract test reaches it; the guard stays.
- Other refusals, all 0 rows: `convention.range_invalid` (overlap, unsorted, empty group, bad range; param `segment_id`), `segment.retired`, `segment.not_in_open_chapter`, `segment.not_found`. When several sentences fail for different reasons, `chapter_changed` wins; otherwise the first other error found is returned.
- `vi.json` already has the three `err.convention.*` keys (added in the Rust phase so the key/translation parity holds). Texts are mine; edit freely.

## Surprises
- An omitted sentence (FR133) named by a spot is reported as `chapter_changed`, not a separate key.
- `ipc_argument_contract` is RED until the webview adapters exist: it lists `convention_scan_chapter_progress` and `convention_replace_chapter` as registered without an `invoke()` caller. It turns green when `config/convention.ts` calls them with exactly the keys above (`groups`; `generation` + `progress`).
- `config_invariants` census updated: `convention.rs` 2/2/2, `segment/wire.rs` 22/5/5, whole tree 117/74.
- The scan wire and `convention_accept`/`convention_revert` are untouched; the single-spot path now shares `write_convention_sentence` with the batch (behaviour unchanged, same error codes).
- The scan has no `ord`/`status`; join on `editorSegments` as 16.1 does.

## Not done here
- Webview, e2e. No commit. Full `cargo test` and vitest not run (per AGENTS.md they run once at the end; `lib.rs` registration changed, so the full run is due).
- Counter-check "deselection = drop the apply_spans filter" has no Rust seam (deselection is the webview omitting a spot); measured instead: removing `insert_pair` turns 3 batch/single cases red, removing the text check in `check_proofread_range` turns the batch-changed case and the single-spot changed case red.

# Webview phase done, for the e2e phase

Working notes only. Webview not committed; Vitest for it green (see Verified).

## Selectors (all `data-` hooks)
- Strip button: `[data-convention-replace-chapter]` (label `Thay cả Chương ({n} chỗ)`; absent when the viewed rule has no replacement).
- Tools menu item: `[data-tools-item="convention.replace_chapter_open"]` (opens on *Tất cả luật*).
- Overlay root `[data-convention-replace]`; summary `[data-convention-replace-summary]`; filter `[data-convention-replace-filter]` (native radios inside `label.ui-seg__item`, arrow keys switch); list `[data-convention-replace-list]` (`role=listbox`, ONE tab stop, `aria-activedescendant`).
- Rows `[data-convention-replace-spot]` with `data-segment-id`, `data-spot-start`, `data-selected="true|false"`, `aria-selected`. Rows are `li role=option`; mouse toggles on `mousedown` (not click). `data-segment-id` is the segment id, the visible heading is `Câu {ord}`.
- Footer counts `[data-convention-replace-hidden]`; armed line `[data-convention-replace-armed]`; refusal frame `[data-convention-replace-refused]` (role=alert); scan failure `[data-convention-replace-error]`; kept note `[data-convention-replace-kept]`; empty `[data-convention-replace-empty]`; scanning line (only after 250 ms) `[data-convention-replace-scanning]`.
- Buttons: `-select-all`, `-select-none`, `-confirm` (label flips to `Xác nhận thay K chỗ` when armed), `-rescan` (only in the stuck state), `-close` (Huỷ / Đóng).
- Keys: `↑↓` move cursor, `Space` toggle, `Esc` disarms then closes. No Enter default. After open, focus lands on the list once the scan ends.

## Commands
- Plan's five plus two I added (reason: a Huỷ/Đóng button needs a dispatchable id, Check A; the strip button must open pre-filtered while the menu/command opens on all rules and `dispatch` carries no argument): `convention.replace_chapter_close`, `convention.replace_chapter_open_rule`. The strip button dispatches `_open_rule`, which reads `viewedConventionRuleKey()` at run time. No default keys. All declared by mouse points; none needed `POINTER_DEBT`.
- Status text after success is `conventionNotice` (key `convention.status.replaced`), shown wherever the 16.1 notice shows. The overlay closes itself on success, marks of the written sentences are cleared and `scanConventionNow()` re-marks the rest (so a deselected spot keeps its dotted mark).

## Surprises / deviations
- Overlay title uses `convention.replace.this_chapter` ("Chương này") for `{chapter_title}`: no chapter title exists in the editor state. Swap it if a title source appears.
- Added `VUE_FLOOR` 57 -> 62 in `scripts/check-i18n.mjs` and `scripts/check-commands.mjs` and `POINTER_FLOOR` 253 -> 273 (both gates told me to: the new `.vue` file and 7 commands drifted the floors below 80%). `HANDLER_TABLE` got four `ConventionReplaceOverlay.vue::*` rows.
- `titlebarToolsMenu.test.ts` pinned the menu length and the opener hooks: updated (+1 item, `data-convention-replace-open` on the trigger).
- `vi.json`: `err.convention.chapter_changed` text rewritten to the UX wording with `{changed_count}`/`{segment_list}`; the overlay builds those params itself from `segment_ids` and `editorSegments` ords (`câu 22, câu 30`). `rule_changed` / `empty_result` still show the Rust-phase wording.
- The overlay's list comes from `convention_scan_chapter_progress` with its own generation counter (starts at 1_000_000) and its own state; the marks scan (`scanConventionNow`) is separate.
- Rows show `before + del + ins + after` when selected, plain text with a dotted underline on the phrase when deselected (a real `<del>`/`<ins>` pair, so e2e can read `del`/`ins` text).
- Spots of a rule with no replacement are never listed; a scan segment not found in `editorSegments` is counted into `not_scanned`.

## Verified
- Vitest: new `conventionReplaceState.test.ts` (19), `conventionReplaceOverlay.test.ts` (5), +2 in `conventionState.test.ts`, `titlebarToolsMenu.test.ts` updated; plus the 44 convention/shortcut/command-related files (771 tests) green. `vue-tsc`, `eslint src tests`, `check:commands|i18n|tokens|layout|gates` green; `npm run build` then `ipc_argument_contract` green (now has both callers).
- Not run: full vitest, full `cargo test`, e2e. Full run is still due once (lib.rs registration changed in the Rust phase).

## For the e2e spec
- Open from the strip: click a dotted mark, then `[data-convention-replace-chapter]`; filter should stand on that rule.
- Deselect with `↓` + `Space` on the list, or real mouse down on a row; confirm with a real mouse click on `-confirm` (twice when a draft is selected).
- Expect the overlay to be gone after success; assert editor cells and `data-selected` before that.
