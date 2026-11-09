---
title: 'Thay cả Chương qua danh sách từng chỗ'
type: 'feature'
ticket: '2'
created: '2026-11-09'
status: 'built'
baseline_revision: 'e82a1bd6bd87780240ae26cd799016bb986db5a2'
route: 'full'
route_source: 'auto'
risk: 'high'
review: 'quick'
review_source: 'pinned'
lenses_ran: ['quick']
review_loop_iteration: 0
context:
  - '{project-root}/_bmad-output/initiative-auratranslate/architecture-auratranslate/architecture-auratranslate.md §AD-67 Rule 7, 8, 12 (c)(d)(e), 12b last bullet'
  - '{project-root}/_bmad-output/initiative-auratranslate/ux-auratranslate/EXPERIENCE.md §Quy ước chữ Việt — FR159 (Màn xác nhận thay cả Chương, State Patterns, Interaction Primitives, KF-5, Khoá i18n mới)'
  - '{project-root}/_bmad-output/initiative-auratranslate/ux-auratranslate/DESIGN.md §Màn xác nhận thay cả Chương'
  - '{project-root}/_bmad-output/initiative-auratranslate/ux-auratranslate/mockups/quy-uoc-chu-viet.html (rows 5 and 5b)'
---

<frozen-after-approval reason="human-owned intent — do not modify unless human renegotiates">

## Intent

**Problem:** After 16.1 the user can fix a convention spot one at a time; a Chapter with dozens of `đảm bảo` needs dozens of clicks, and a blind replace-all would corrupt syllable-boundary matches such as `nhằm bảo đảm bảo vệ`.

**Approach:** A confirmation overlay lists every match of every active rule in the open Chapter, grouped by sentence, all pre-selected, deselectable, filterable by rule; one Rust command writes every selected spot in **one** transaction under AD-67 Rule 7, or writes nothing.

## Boundaries & Constraints

**Always:**
- The overlay opens by flushing the typing buffer, then rescanning the stored text; its list comes from that scan, never from the marks already on screen.
- The webview sends spots (`segment_id`, `expected_text`, UTF-16 `[start,end)`, `expected_phrase`, `rule_key`); Rust computes the cased replacement from the stored rule. Spans apply right to left on the pre-write text, so replaced text is never re-matched.
- Per sentence: the AD-53 Rule 4 check, rule still active, arbitration on the pre-write text, `write_convention_target` (draft ⇒ draft; confirmed ⇒ stays confirmed, one `segment_version`, one TM pair). Origin never becomes *tôi dịch* (AD-67 Prevents 1).
- One sentence failing any check ⇒ the whole batch rolls back, 0 rows changed, and the error names every changed sentence.
- Footer always states the four hidden counts (`ignored`, `glossary_suppressed`, `overlapped`, `not_scanned`), or *Không có chỗ nào bị lọc.* when all are 0.
- A selected draft sentence requires a second press that names the draft sentences (AD-67 Rule 8, lớp iii); any checkbox change disarms it. No draft selected ⇒ one press writes.
- Every action is a registered command (AD-34); keyboard and mouse both reach it (R11).
- Decision (2026-11-09, Ice): the overlay's scanning line `Đang quét {done}/{total} câu…` shows real progress streamed from Rust over a `tauri::ipc::Channel`, not a count-free bar.

**Never:**
- No in-memory undo tray for the batch; no `Enter` default for the overlay; no default key for `replace_chapter_*`.
- Do not change `convention_accept_range`, `convention_revert_range`, `check_proofread_range`, `arbitrate_before_write`, or the scan wire's existing field names.
- No duplicate-source prefill (this is not a confirm), no `Indexer::rebuild` (no other `target_text` write runs it).
- Out of scope: "Không phải lỗi" (16.5), Work-tier rules (16.3), rescan after flush (16.4).

## I/O & Edge-Case Matrix

| Scenario | Input / State | Expected Output / Behavior | Error Handling |
|---|---|---|---|
| Deselected syllable-boundary spot | `nhằm bảo đảm bảo vệ`, one match, deselected; other sentences selected | that sentence unchanged byte for byte, others replaced | — |
| Same spot selected | as above, selected | `nhằm bảo bảo đảm vệ` | — |
| Two spots, one sentence | `Đảm bảo rằng đảm bảo.` both selected | `Bảo đảm rằng bảo đảm.` — one version / one TM pair if confirmed | — |
| Confirmed sentence | status `confirmed`, AI origin | stays confirmed, +1 `segment_version`, +1 `tm_unit` with new text, origin unchanged | — |
| Draft sentence | status draft | stays draft, origin per arbitration, 0 versions | — |
| Sentence changed since scan | stored text ≠ `expected_text` for 2 sentences | 0 rows changed | `err.convention.chapter_changed` with both sentences; overlay alert + Quét lại |
| Rule removed/disabled since scan | rule inactive | 0 rows changed | `err.convention.rule_changed` |
| Result empty after trim | rule whose replacement trims to empty; sentence is only the phrase | 0 rows changed | `err.convention.empty_result` |
| Rule without replacement | spots of a mark-only rule | not listed, no filter item | spot naming it ⇒ `rule_changed` |
| Overlapping or out-of-order spans in one sentence | crafted input | 0 rows changed | range error |
| Omitted (FR133) or retired sentence | in Chapter | not listed, counted in `not_scanned` | spot naming it ⇒ refused |
| 0 spots selected | — | primary button locked | — |

</frozen-after-approval>

## Code Map

Rust (`src-tauri/src/`):
- `core/convention/matcher.rs:8` `Hit{start,end,rule}` UTF-16; `casing.rs:3` `transfer_case` -- reuse. No multi-span splice exists: add a pure `apply_spans(text, &[(byte_start, byte_end, replacement)])` here (sorted, non-overlapping, right to left).
- `commands/convention.rs:83` `convention_active_rule(global, rule_key)` -- call once per distinct key in the shell before the transaction (rule lives in `global.db`); `:106/:116/:125` scan wire -- do not rename fields; it carries no ord/status, so the webview joins on `editorSegments`.
- `commands/segment/convention_write.rs:55` `splice_convention_range` -- opens its own `store.write` and returns `Ok(Err(ipc))`, which commits: **not reusable in a batch**. Lift its per-sentence body (check → splice → empty check → arbitrate → status/source → `write_convention_target` → mark started) into a helper taking `&Transaction`, used by the new batch; `:39` `as_convention_error` -- reuse; `:7` `ConventionWriteOutcome` -- per-sentence outcome shape.
- `commands/convention.rs:269` `convention_scan_chapter` -- give the matching loop a progress callback `(done, total)`; the existing wire shell `:342` passes a no-op and keeps its signature (16.1 callers). Channel precedent: `commands/proofread.rs:683` `tauri::ipc::Channel<AiProofreadEventWire>`.
- `commands/segment/proofread_accept.rs:61` `check_proofread_range` (takes a tx; check text once, then each span) and `:121` `arbitrate_before_write` -- reuse unchanged.
- `commands/segment/targets.rs:290` `write_convention_target`; `:97-197` `save_segment_targets` -- the all-or-nothing template: `Arc<Mutex<Option<BatchReject>>>` set in the closure, closure returns `Err` to roll back, mapped after.
- `commands/lifecycle.rs:197` `mark_segment_chapter_started`.
- `commands/segment/wire.rs:604` `convention_accept` shell shape -- add `convention_replace_chapter` beside it; `lib.rs:1235` register.
- `core/i18n/mod.rs:944-949` -- add `err.convention.chapter_changed`, `rule_changed`, `empty_result` (map from `rule_unavailable`/`text_would_be_empty` where the meaning is the same).
- `tests/config_invariants.rs:1449` `blocking_wire_cases` (add), `:2318` census row for `segment/wire.rs` (bump); `tests/ipc_argument_contract.rs:630` -- the new command has a frontend caller.
- `tests/convention_contract.rs:139` `fixture`, `:165` `seed`, `:231` `versions`, `:238` `pairs_with`, `:204` `dump` -- extend.

Webview (`src/`):
- `conventionState.ts:280` `acceptConventionSpot` -- template (flush :289, generation check :294, adapter, `mirror` :272, marks, `say`); `:99` `scanConventionNow`; `:131` `clearConventionFor`; `reverts` tray -- clear it for every changed sentence.
- `config/convention.ts:100/160` -- add the adapter (never throws, runtime type check).
- `panels/editorPanelState.ts:999` `flushEditorBeforeDiscreteWrite`; `:359` `replaceEditorSegment`; `:199` `editorSegments` (ord ⇒ *Câu {n}*, status ⇒ label).
- `ProofreaderStrip.vue:274` `.pfs-actions` -- add **Thay cả Chương ({n} chỗ)** (count of the viewed rule; absent when the rule has no replacement), `@mousedown.prevent`.
- New `ConventionReplaceOverlay.vue` + `conventionReplaceState.ts`: shell from `SettingsOverlay.vue:939` (917px); focus/trap from `ConventionManageOverlay.vue:21-72`; filter `.ui-seg` `role="radiogroup"` (`tokens/controls.css:614`, `GlossaryManageOverlay.vue:439`); list one tab stop like `modes/LibraryWorkPage.vue:601` (`listbox`, `aria-multiselectable`, ↑↓ :222, Space :227); press-again arming like `glossaryManageState.ts:547`. Mount in `App.vue:394`; add open flag to `main.ts:1190`.
- `commands/index.ts:5372-5443` + `conventionHandlers.ts:10` -- `convention.replace_chapter_open|select_all|select_none|confirm|rescan`; `TitlebarToolsMenu.vue:178` entry *Thay cả Chương theo quy ước*.
- `i18n/vi.json:892` -- keys `convention.replace.*`, `convention.strip.replace_chapter`, `convention.status.replaced`, `command.convention.replace_chapter_*`.

E2E: `e2e/specs/story-16-1-convention-mark-accept.e2e.mjs` (`invoke`, `openWorkspaceWithWork`, `promote_ai_translation`).

## Tasks & Acceptance

**Execution:**
- [ ] `core/convention/` -- `apply_spans` pure fn + unit tests (multi-span, right to left, refuses overlap/unsorted).
- [ ] `commands/segment/convention_write.rs` -- per-sentence tx helper lifted from `splice_convention_range` (single-spot path keeps behaviour); `convention_replace_chapter_batch(tx, open_chapter_id, groups, rules)` with the reject-cell rollback, collecting every failing sentence before refusing; returns per-sentence outcomes + `{replaced, sentences, confirmed, drafts}`.
- [ ] `commands/convention.rs` -- `convention_scan_chapter_progress(generation, channel)`: same scan and same result wire, progress `{done,total}` sent over the channel at most every 250 sentences and once at the end; a closed channel never fails the scan.
- [ ] `commands/segment/wire.rs`, `lib.rs`, `core/i18n/mod.rs`, `tests/config_invariants.rs` -- async shell (rules resolved before the lock), register, error keys, blocking list, census.
- [ ] `config/convention.ts`, `conventionReplaceState.ts` -- open (flush; `failed`/`still-dirty` ⇒ `err.convention.flush_failed`, no scan), rescan with a new generation, selection keyed by `(segment_id, rule_key, start)`, filter display-only, select all/none on visible rows only, two-step arming when a selected sentence is draft, rescan keeps a deselection when the sentence's 16 UTF-16 units before and after the phrase are unchanged; on success `replaceEditorSegment` per sentence, clear tray + marks of those sentences, rescan, `convention.status.replaced`.
- [ ] `ConventionReplaceOverlay.vue`, `App.vue`, `main.ts` -- layout per DESIGN; scanning line after 250 ms fed by the progress channel; alert frame with changed sentence numbers + Quét lại / Đóng; Esc disarms then closes; focus returns to the strip button if present, else the caret's cell.
- [ ] `ProofreaderStrip.vue`, `commands/index.ts`, `conventionHandlers.ts`, `TitlebarToolsMenu.vue`, `i18n/vi.json` -- button (opens with filter on the viewed rule), five commands with `pointer` declarations, menu entry (opens on *Tất cả luật*), keys.
- [ ] tests -- `convention_contract.rs`: every matrix row; AD-67 12 (c) via batch, (d), (e) with counter-checks by real removal (TM insert; per-sentence text check; deselection = drop `apply_spans` filter); scan + batch leave unselected sentences' rows identical (`dump`). Vitest: arming/disarming, select-all on filtered view, filter keeps selection, rescan keeps deselection, K counts hidden rows.
- [ ] `e2e/specs/story-16-2-convention-replace-chapter.e2e.mjs` -- epic Done-when 2: three AI sentences + one confirmed, deselect one spot with `↓`/`Space`, confirm with real mouse; deselected cell unchanged character for character, origins unchanged, confirmed stays confirmed.

**Acceptance Criteria:**
- Given KF-5 (5 spots, one in `nhằm bảo đảm bảo vệ`, one in a draft), when the user deselects that spot and confirms twice, then 4 spots in 4 sentences are replaced, the status bar names confirmed and draft consequences, and sentence 22 keeps its mark.
- Given the overlay opened from the strip, then the filter stands on that rule and K counts selections across all rules; opened from `Công cụ ▾`, it stands on *Tất cả luật*.
- Given the whole flow by keyboard only, and again by mouse only, then both complete (R11).
- Given a batch refused, then the alert names the changed sentences, says 0 written, and Quét lại rebuilds the list keeping still-valid deselections.

## Implementation Notes

- `splice_convention_range` could not be reused in a batch (its own `store.write` commits on refusal); the per-sentence body is now `write_convention_sentence(tx, …)`, shared by the single-spot path and `convention_replace_chapter`.
- The batch plans every sentence before writing any; text changes (and a sentence omitted since the scan) are gathered into one `convention.chapter_changed` naming all of them, any other refusal returns the first found. A sentence named twice in one request is not refused (review: rejected, the webview sends one group per sentence).
- Progress: `convention_scan_chapter_progress` streams `{done,total}` over a `Channel` every 250 sentences and once at the end; the 16.1 scan keeps its signature.
- Two commands beyond the plan's five: `convention.replace_chapter_close` (Huỷ/Đóng need an id) and `convention.replace_chapter_open_rule` (`dispatch` carries no argument, so strip and menu need distinct ids).
- The overlay title reads *Chương này*: the editor state has no Chapter title source.
- `err.convention.chapter_changed` keeps Rust's `{segment_ids}`; the UX wording lives in the webview-only key `convention.replace.changed_alert`.
- The filter row no longer wraps (`controls-adoption` forbids `height: auto`/`min-height` on `.ui-seg`); many rules may overflow it.
- A committed batch is mirrored into the Editor even when the overlay closed during the write (review high).
- Gate floors rose as the gates asked (`VUE_FLOOR` 57→62, `POINTER_FLOOR` 253→273, `CONVENTION_FLOOR` 5→6); `POINTER_DEBT` stays empty.
- Deselection has no Rust seam to remove (the webview omits the spot); the e2e spec and Vitest guard it.

## Plan Change Log

## Review Triage Log

Pass 1 (quick): 1 high · 0 medium · 3 low · 4 false · 0 maybe-false.
- Overlay closed during the write drops a committed batch (`confirmConventionReplace`, `mine !== session`) — high · patch: editor stays stale and a later edit can overwrite the batch.
- `not_scanned` adds spots, labelled as sentences (`buildGroups` `unjoined`) — low · patch.
- Duplicate `segment_id` groups are not refused (`convention_replace_chapter`) — low · rejected: the webview builds one group per sentence from a Map, and the fix adds a guard.
- Comment restating the next lines in the 16.2 e2e spec — low · patch.
- Footer shows only non-zero hidden counts — false: EXPERIENCE says *Cả bốn số luôn có mặt khi khác 0*, the all-zero line is covered.
- New commands carry no `pointer` — false: `check:commands` green, `POINTER_DEBT` empty; floors only rose.
- Cursor index out of range after the list shrinks — false: `moveConventionReplaceCursor` clamps the result (`conventionReplaceState.ts:331`).
- Plan `## Implementation Notes` empty — rejected: fix edits this plan.
- (own) empty-after-trim guard duplicated in `plan_sentence`, so its counter-check cannot fail alone — low · patch.

## Design Notes

- Two locks, as 16.1: rules are read from `global.db` before the `project.db` transaction; a rule changed in between is accepted risk until 16.3 adds edit/delete (existing debt `Chủ: Story 16.3`).
- One version and one TM pair per **sentence**, not per spot: group spots by sentence, splice all, write once.
- Phases (AGENTS.md): Rust · Webview · Tests that move (e2e), fresh agent each, notes in a handoff file beside this plan. The 16.1 e2e has never gone green (Keychain stall on a locked Mac); run both specs on an unlocked, attended Mac outside the sandbox.

## Verification

**Commands:**
- `npm run build` then `cargo test --manifest-path src-tauri/Cargo.toml --test convention_contract --test config_invariants --test ipc_argument_contract` -- expected: green, incl. a case that the progress callback reaches `done == total` and the result equals the plain scan.
- full `cargo test` + vitest once (`lib.rs` command registration changed) -- expected: green.
- `npm run check:commands` and `npm run check:i18n` -- expected: green, `POINTER_DEBT` not increased.
- `npm run test:e2e -- --spec e2e/specs/story-16-2-convention-replace-chapter.e2e.mjs` (outside sandbox, unlocked Mac) -- expected: green on WKWebView.
