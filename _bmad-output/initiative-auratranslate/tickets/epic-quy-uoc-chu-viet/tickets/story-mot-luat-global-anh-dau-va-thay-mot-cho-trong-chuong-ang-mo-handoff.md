# Handoff: Rust phase done, Webview phase next

Plan: `story-mot-luat-global-anh-dau-va-thay-mot-cho-trong-chuong-ang-mo-plan.md`. Nothing committed.

## Wire shapes (Rust field names are snake_case; invoke args are camelCase)

All commands are registered in `lib.rs`. Result fields keep snake_case.

- `convention_list_rules()` -> `[{id, rule_key, avoid_phrase, replacement_phrase: string|null, enabled, tier: "global"}]`, ordered by `rule_key`.
- `convention_add_rule({avoidPhrase, replacementPhrase?: string|null})` -> `number` (id). Blank replacement is stored as NULL ("khong dung"). Trimmed. Errors: `convention.empty_avoid_phrase`, `convention.duplicate_avoid_phrase` (same NFC+lowercase key). Drops the cached matcher; the webview must bump its generation and call the scan again.
- `convention_scan_chapter({generation})` (async, open Chapter only) -> `{generation, chapter_id, segments: [{segment_id, scanned_text, spans: [{start, end, rule_key, avoid, replacement_cased: string|null, tier}]}], ignored, glossary_suppressed, overlapped, not_scanned}`. `start/end` are UTF-16 on `scanned_text` (the stored string, any normal form). Only sentences with at least one span are listed. `generation` is echoed, Rust keeps no counter; the webview drops results older than its own. `ignored` is always 0. `not_scanned` counts omitted sentences only (empty targets are skipped, not counted). No open Work -> `work.none_open`.
- `convention_accept({segmentId, expectedText, start, end, expectedPhrase, ruleKey})` -> `{segment_id, target_text, translation_origin, status, chapter_started, start, end, phrase, version_created}`. `expectedPhrase` is `scanned_text[start..end)`; the replacement is computed in Rust (never sent). `start/end` in the result are the span of the inserted `phrase` in the NEW `target_text`; use them for the undo tray and for shifting other spans (AD-53 rule 6). `version_created` is true when the sentence was confirmed. If `chapter_started`, call `refresh_after_chapter_started` like other writes. Apply text via `replaceEditorSegment`.
- `convention_revert({segmentId, expectedText, start, end, expectedPhrase, originalPhrase})` -> same outcome. `expectedPhrase` is the phrase now in the text (the tray's `phrase`), `originalPhrase` the text it replaced (the span's `avoid`-cased original, i.e. what `scanned_text[start..end)` was). On a confirmed sentence revert adds another version and TM pair.

## Error codes (all `err.convention.*`, already in `src/i18n/vi.json`)

`convention.empty_avoid_phrase`, `convention.duplicate_avoid_phrase`, `convention.text_changed` {segment_id}, `convention.range_invalid` {segment_id}, `convention.rule_unavailable` (rule gone, disabled, no replacement, or `ruleKey` not the key of `expectedPhrase`), `convention.text_would_be_empty` {segment_id}. `segment.retired`, `segment.not_in_open_chapter`, `work.none_open` pass through unchanged. Also `convention.state_missing` (config error, key `err.unknown`).

## Surprises

- The rule lives in `global.db`, the sentence in `project.db`: no shared transaction. `convention_accept` reads the rule just before the write transaction, so "rule still active" is checked at call time, not inside the tx.
- Case transfer: all-lower -> replacement lowercased; Capitalised (or one upper letter) -> lowercased then first letter upper; ALL-UPPER (2+ letters) -> uppercased; any other mix -> rule spelling.
- Matching runs per cluster (starter + combining marks) NFC+lowercase, so spans on NFD text cover whole clusters.
- `ipc_argument_contract` (NO_FRONTEND_CALLER) is red until the five commands have `invoke` callers in `src/config/convention.ts` with the exact camelCase keys above. `check:i18n`/vi.json: only the six `err.convention.*` keys were added; every other key is for the Webview phase.
- Matcher cache is a managed state `ConventionMatcherState` (Global rules only). The Work tier in 16-3 must key it by Work.
- Revert has no rule check by design (it must work after the rule is deleted).
- Rust shared fns touched: `confirm.rs::insert_segment_version` (extracted), `proofread_accept.rs::arbitrate_before_write` (extracted; Proofreader behaviour unchanged).

# Handoff: Webview phase done, Tests-that-move phase next

Nothing committed. Gates green at the end of this phase: `check:commands`, `check:i18n`, `check:tokens`, `npm run build`, full vitest (257 files), eslint on `src`, `ipc_argument_contract`.

## What exists

- `src/config/convention.ts`: five adapters (never throw). Helpers take a `Label` string, not the `CMD_*` const, because `ipc_argument_contract` panics on `fn(CMD_X, non-object)`.
- `src/conventionState.ts`: scan (generation = module counter, `scanConventionNow()`), results map, `opened` spot, undo tray LIFO per segment, `stepMarks(1|-1)` (F8 over both families, text order, `no_next`/`no_prev`/`no_marks` as a 5 s status notice), accept/revert/skip/hide, `clearConventionFor` (typing). `conventionStripIsEligible` feeds `inlineStripEligibility.ts`.
- `src/conventionManageState.ts` + `ConventionManageOverlay.vue`: Global list + add row (avoid, replacement); a successful add reloads the list, flushes the editor and rescans if a Chapter is open.
- Marks: `panels/proofreadHighlight.ts` (`.convention-mark`, `data-convention-start`, selector shared with Proofreader spans so one unwrap kills both; `repaintProofreadHost` takes a 4th arg). `GridPanel.vue`: scan on `[editorChapterId, openWorkId]` watch (reset + scan), `document` click listener opens the strip when the selection is collapsed (no `@click`, Kiểm A), repaint watches `conventionResults`. CSS in `GridBlock.vue`.
- Strip: second `<section>` inside `ProofreaderStrip.vue` (same shell classes, `--convention` left rule). Kind `convention` after `proofreader` in `InlineStripKind`/`PRIORITY_ORDER`; floats like the Proofreader strip, place `track`.
- Commands (`commands/index.ts`): `marks.next` F8, `marks.prev` Shift+F8 (removed from `ai.proofread.next/prev`), `convention.accept|skip|revert|hide`, `convention.manage.open` (`Mod+Alt+A`, also in `TitlebarToolsMenu.vue`), `convention.manage.close|add`. Ports in `conventionHandlers.ts`, spread in `main.ts`; `conventionManageIsOpen` blocks layout chords.
- Status bar: `StatusBar.vue` shows `conventionNotice` right after the confirm notice (`data-status-convention`).

## Deviations to know

- Label keys follow the gate (`command.<id>` with dots): `command.convention.manage.open|close|add`, not the `manage_open` spelling in EXPERIENCE.
- `convention.strip.hint` uses `{chord_next}`/`{chord_prev}` (two distinct params; one `{chord}` cannot hold both).
- New keys not in EXPERIENCE: `convention.status.scan_failed` (the "Lỗi quét" text), `convention.manage.add`/`add_avoid`/`add_replacement`/`loading`/`empty_rules` (empty text shortened: no import link yet).
- `DISPATCH_FLOOR` 385 -> 412 (`check-commands.mjs`) and `RS_FLOOR` 123 -> 134 (`check-i18n.mjs`): both floors had drifted under 80% of the live count (phase 1 added `.rs` files); raised as the gate's own message says.
- Existing tests updated: `titlebarToolsMenu` (ten items, eighth hook), `proofreadWiring` (4th repaint arg), `proofreaderStripContent` (chord ids `marks.*`), `inlineStripPriority` (six kinds). New: `conventionState.test.ts`, `conventionManage.test.ts`, convention block in `proofreadHighlight.test.ts`.

## For phase 3

- e2e hooks: overlay inputs `[data-convention-add-avoid]`, `[data-convention-add-replacement]`, button `[data-convention-add]`, close `[data-convention-manage-close]`, rows `[data-convention-rule-row]`; marks `.convention-mark[data-convention-start]` inside `[data-col="tgt"]`; strip `[data-convention-strip]` with `[data-convention-accept]`, `-skip`, `-prev`, `-next`, `-revert`, `-hide`, `-manage`; status `[data-status-convention]`. Strip buttons use `@mousedown.prevent` on the action row.
- Mark click is a `document` `click` listener, so a real mouse click is the only faithful test (Playwright `.click()` works but note it needs a collapsed selection).
- A Proofreader scan-level strip (scanning/error/not_configured) outranks `convention` per the frozen order; after a Proofreader error the convention strip will not show until that state is reset. Not guarded by a test; decide whether to record as debt.
- Not done here (belongs to 16.2+/phase 3): e2e spec, measurement, counter-checks by removal, debt item, `NO_FRONTEND_CALLER` stays empty for the five commands (all have callers).
- Counter-check candidates on the Webview side: remove `clearConventionFor(id)` from `reportEdit` (GridPanel) and see `proofreadWiring`/state tests; remove the generation check in `scanConventionNow` (test "drops a result older than the newest request").

# Handoff: Tests-that-move phase

Nothing committed. Measurement not rerun (phase 1 numbers stand).

## Counter-checks by real removal (each re-ran only `--test convention_contract`; all restored, 21/21 green afterwards)

- NFC: `normalize.rs` `.nfc()` replaced by `.chars()` -> 3 red (`an_nfd_rule_matches_nfc_text`, `nfd_text_gives_utf16_spans_on_the_stored_string`, `accepting_on_nfd_text_keeps_every_byte_outside_the_span`).
- Arbitration: body of `arbitrate_before_write` returns the baseline origin without `arbitrate` (signature kept) -> 1 red (`accepting_on_a_sentence_the_user_typed_over_keeps_self`).
- TM insert on the confirmed branch: `insert_pair` removed from `write_convention_target` -> 1 red (`accepting_on_a_confirmed_sentence_keeps_it_confirmed_and_adds_one_version_and_one_pair`).
- Glossary check: the pinned-span suppression in `match_sentences` made unreachable -> 1 red (`a_match_inside_a_pinned_glossary_translation_gets_no_mark`).
- Boundary: done in phase 1.

## e2e: spec written, NOT green

`e2e/specs/story-16-1-convention-mark-accept.e2e.mjs` (lint clean): rule through the overlay (tools menu, real clicks), real click on the mark, Nhận thay, Hoàn lại. It has never passed.
Every run (also the untouched `story-12-19`) stalls at the first `invoke` after the grid opens: `sample` shows the main thread in `read_open_chapter_segments` waiting on the `OpenWorkState` mutex while a `tokio` task is inside `ai_config_get` -> `SecurityAgent` keychain probe (`keychain::configured`). The Mac was at the login window, so the keychain prompt cannot be answered. Environment, not convention code (reproduced with the convention scan disabled in `GridPanel.vue`, since restored). Rerun the spec on an unlocked, attended Mac; then `cargo test` has overwritten the wdio binary, so `npm run test:e2e` rebuilds it first.
Side finding (pre-existing, not fixed): `ai_config_get` holds the `OpenWorkState` lock across the keychain probe, so a pending keychain prompt freezes every sync command on the main thread.
The spec's assumption to verify on first green run: `promote_ai_translation` then `open_work` refreshes the grid target cell, and `library_list_works` finds the Work by name.
