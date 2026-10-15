# Handoff phase 1 (Rust) -> phase 2 (Webview)

Rust is done and uncommitted. Wire field names are snake_case (no rename_all); command ARGUMENT names are camelCase on invoke.

## Wire changes the webview must follow

- NEW command `glossary_tm_suggestions` { sourceTerm: string } -> `{ target_text: string, count: number }[]`
  Exact-source pairs, both tiers, grouped by target, most pairs first, ties in AD-18 order. Empty array = none. With no Work open it reads Global only. Registered in `lib.rs`.
  Core fn: `core::tm::target_suggestions_for_term`.
- `GlossaryMarkWire` gains `term_origin: string` (`manual` | `import_scan` | `review_harvest` | `file_import`) and `category: string` (`Category::as_str`, same ids as `glossary.manage.category_*`). Both always present (not optional); empty-string never occurs, so "drop the clause when empty" in the I/O matrix only applies if the webview maps an unknown id to nothing.
- `TmFuzzyMatch` gains `created_at: string` (ISO UTC with ms, same as `TmExactTarget.created_at`).
- `tm_count_pairs` now returns `{ total, mine, others }` (was a bare number). `mine` = origin `self`; `others` = `other` + `bilingual_import`. `src/config/tm.ts:238` (`CMD_TM_COUNT_PAIRS`) still expects a number: it must read `.total` or the new object; update `workspaceCountsState.ts` and `StatusBar.vue`.
- `proof_ignore` has new nullable `blocked_count` (migration step 36). `ai_proofread_ignore_list` entries gain `blocked_count: number | null` (null = row predates the counter: show only the date, no count).
- `AiProofreadOutcomeWire` `done` gains `blocked: { ignore_id: number, count: number }[]` (only entries with count > 0).
- `ai_proofread_scan_record` gains a 6th argument `blocked: { ignore_id, count }[]` (camelCase arg name `blocked`). The webview must pass the `blocked` array from the `done` outcome through to this call; pass `[]` when there is none. The counter is raised in the same transaction as the `proof_scan` insert; NULL rows stay NULL. Update `src/config/proofread.ts` and the callers that record a finished scan (grep `ai_proofread_scan_record`).

## Things worth knowing

- The block counter is raised only when the webview records the scan (`ai_proofread_scan_record`), not when the scan streams; a scan never recorded leaves counts unchanged.
- A finding silenced by several entries is credited to the first (lowest id).
- Rust-side `filter_remembered` now returns `(kept, per_entry_counts)`; `PreparedProofreadRun` gained `remembered_ids`.
- Schema is now version 36; hardcoded 35s in Rust tests were bumped. Nothing in `src/` pins the schema version as far as I touched (not grepped in `src/`).
- Rust changes verified: tm_contract, glossary_marks_contract, tm_wire, proofread_memory_contract, ipc_contract, pinned_contract, segment_contract, ai_proofread_contract green; counter-check done (deleting the UPDATE loop turns `a_scan_reports_blocked_findings_per_entry_and_recording_it_raises_only_that_entry` red). Full `cargo test --no-fail-fast`: every target green except `ipc_argument_contract`, which is RED BY DESIGN until phase 2: it needs a TS `invoke('glossary_tm_suggestions', { sourceTerm })` in `src/**/*.ts`, and will also compare the keys of `invoke('ai_proofread_scan_record', ...)` against the Rust params (now includes `blocked`). Re-run `cargo test --test ipc_argument_contract` at the end of phase 2.
- Not done by phase 1 (belongs to later phases): all of `src/`, `e2e/`, `scripts/check-commands.mjs`, vitest, deferred-work.md entries.

# Handoff phase 2 (Webview) -> phase 3 (tests that move, e2e, ledger, full run)

Webview done, uncommitted. `cargo test --test ipc_argument_contract` is green. Green: vue-tsc on `src/`, `check:lint` (src+e2e+tests), `check:commands` `i18n` `tokens` `panel-refs` `layout` `doc-refs` `gates`. `src-tauri/` untouched (no wire mismatch).

## Behaviour changes that old tests/e2e still assume (phase 3 must move them)

- Glossary strip: the input is NOT prefilled with the Hán Việt reading any more. Hán Việt is numbered option 1 (`.gcs-option`, `.gcs-key`, `.gcs-option-text`, `.gcs-option-source`); the input (`.gcs-input`) is the "gõ cách khác…" box. Save value = aimed option, else typed text, else option 1, else empty error. `.gcs-suggestion-label` and key `glossary.confirm.suggestion_label`/`occurrence_count_label` are gone (meta line `.gcs-occurrence` now "chờ chốt bản dịch · xuất hiện N lần trong Tác phẩm", `meta_pending` when count null). Title span `.gcs-title` is the bare term. Form has `tabindex=-1`, an extra `@keydown` (`onKeydown`), `glossary.confirm.focus` focuses the form (not the input) when there is at least one option, else the input. Digits are by `event.code` (`Digit1..9`/`Numpad1..9`), only outside the input.
- `GlossaryMark` wire now requires `term_origin` + `category`; `SegmentTermSpan` gained `termOrigin`, `category`. Fixtures that build marks/spans without them fail the guard or vue-tsc: glossaryConfirmStrip*.test.ts (Strip, Suggestion, Template, ResetWiring), glossaryMarksMap/Refresh, glossaryHoverSelection, editorClearSourceCuts, hanVietCutAnchors, tmConcordance{Command,Highlight,Mapping}, reviewerImportFocus (check).
- `TmFuzzyMatch.created_at` required (tmFuzzyStrip.test.ts, tmFuzzyAdapters.test.ts fixtures). Fuzzy rows now have two `.tmf-btn` per row (Nhận n, then Xem cả câu): `tmFuzzyStrip.test.ts` "mousedown on a row button" picks the wrong button; `.tmf-pct` text is "82% khớp". Fuzzy title/hint changed ("Translation Memory — N câu gần giống", "1 2 3 nhận · Esc ẩn dải"). New command `tm.fuzzy.toggle_full` (no key) + dep `toggleTmFuzzyFull`.
- `tmCountPairs()` now returns `{total,mine,others}|null`; store export renamed `workspaceTmPairCount` -> `workspaceTmPairCounts`; status text key `status.tm_count` replaced by `status.tm_summary` ("TM: N cặp · X của tôi · Y của người khác · ngưỡng gợi ý T%"). Fix statusBarWorkspace.test.ts, tmManage.test.ts, tmImport.test.ts mocks.
- Proofread: `ProofreadOutcomeWire` done requires `blocked`; ignore rows require `blocked_count` (number|null); `recordProofreadScan` takes a 5th arg. Fix proofreadConfig.test.ts, proofreadState.test.ts, proofreadMemory.test.ts fixtures (`blocked`, `blocked_count`). Strip title key `proofread.strip.title` is now "Proofreader"; found-title `proofread.strip.title_found`; card buttons use `proofread.strip.accept/dismiss/ignore` ("Nhận đề xuất", "Bỏ qua", "Không phải lỗi"). New `ai.proofread.hide` command, `proofreadStripHidden`/`hideProofreadStrip`/`syncProofreadStripHidden` in proofreadState, filter in `inlineStripEligibility`.
- ProofreaderStrip now imports `config/shortcutsState` (calls `detectIsMac()` at module load): tests that `vi.mock('../../src/commands', ...)` without `detectIsMac` crash at import (proofreaderStripSlots.test.ts). Mock `config/shortcutsState` or add `detectIsMac` to the commands mock.
- Strip has scope buttons `[data-proofread-strip-scope=segment|selection|chapter]` and the link `[data-proofread-strip-memory]` (own attribute on purpose: the Tools button keeps `data-proofread-memory-open`). `check:commands` `DISPATCH_FLOOR` raised 258 -> 280 (gate demanded it).

## New tests written this phase (all green)

glossaryConfirmStripTmOptions, tmFuzzyStripContent, proofreaderStripContent, proofreadMemoryContent, threeStripsWire (all under tests/frontend). Not covered at vitest level: StatusBar rendering of `status.tm_summary` (do it while fixing statusBarWorkspace.test.ts), counter-check of the hard-coded-F8 case was by construction (test rebinds to F9), not by removal run.

## Surprises

- `src/AGENTS.md`/gates: `-webkit-line-clamp` in TmFuzzyStrip.vue passed `check:tokens`.
- Date helper `src/panels/shortDate.ts` (`dd-MM`, local time); tests use 12:00Z timestamps to stay stable across timezones.
- Esc on the Proofreader strip hides only when no scan-level state is present (scanning/error/not_configured keep the strip so Cancel stays reachable).
