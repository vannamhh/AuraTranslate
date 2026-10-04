---
title: 'Story 7.11 — Smart RAG inserts similar TM pairs, the user''s own first'
type: 'feature'
created: '2026-10-04'
status: 'done'
route: 'dispatch'
baseline_commit: '1eaebeeac39627a2b0fcb7437dcf70b9eca890be'
review_loop_iteration: 0
context:
  - '{project-root}/AGENTS.md'
  - '{project-root}/src/AGENTS.md'
  - '{project-root}/src-tauri/AGENTS.md'
---

<frozen-after-approval reason="human-owned intent — do not modify unless human renegotiates">

## Intent

**Problem:** `{{tm_similar_segments}}` always expands to nothing: the only call site passes `tm: None` (FR70 TM half, FR118). The AI never sees how the user translated similar sentences, so its style drifts toward nobody's.

**Approach:** Gather TM pairs whose source is similar to the sentence (both tiers, `tm_fuzzy_threshold`), let the pure assembler pick the user's own pairs first and fill the rest with others' pairs under a "reference style" label, record each inserted pair in the ledger, and show them in Xem prompt and in the summary line.

**Decisions (Ice, 2026-10-04):**
- Q1 a: at most 3 TM pairs per call.
- Q2 b: a pair whose source equals the sentence exactly is not inserted (same rule as the strip; exact belongs to pre-fill).
- Q3 A: own pairs as bare `source → target` lines; when any others' pair is inserted, one line `Văn phong tham khảo (bản dịch của người khác, không phải văn phong của người dùng):` precedes all others' lines.
- Q4 A: rows are read once per prepare (single, assemble and batch) and every sentence is scored while `OpenWorkState` is held. Strip figures (7.5, release, 100,000 pairs per tier): ≈ 0.5 s per sentence, so a 100-sentence batch holds the lock ≈ 50 s at that size.

## Boundaries & Constraints

**Always:** `gather_glossary_context`/`assemble_prompt` signatures unchanged (`SimilarSegment` may grow fields). Selection and rendering live in the pure assembler; same input ⇒ byte-identical prompt. Own (`PairSide::Mine`) pairs always precede others' pairs, whatever their percent; inside one side, higher percent first, ties in AD-18 order. A body without the marker searches nothing (ledger "not asked", never "0 found"). The "not built yet" TM state is removed.

**Never:** touch `rank_fuzzy_candidates`'s strip contract (`FUZZY_MATCH_LIMIT`, exact exclusion); a second similarity scorer (AD-17); `tm → ai` imports (AD-13); a new prompt variable; the excluded-pairs list of the mockup.

## I/O & Edge-Case Matrix

| Scenario | Input / State | Expected Output / Behavior |
|---|---|---|
| Enough own | ≥ cap own pairs above threshold, others at higher % | Only own pairs inserted, no reference label |
| Mixed | 1 own, 4 others above threshold, cap 3 | own first, then 2 others under the label |
| Only others | 0 own, 2 others | 2 others under the label |
| None similar | no pair ≥ threshold | marker line removed; ledger searched-empty; summary "0 câu TM" |
| No marker | body lacks `{{tm_similar_segments}}` | no TM read; ledger not-asked |
| Summary | AI call finished (single or batch) | line refreshes: "Đã chèn N thuật ngữ Glossary và M câu TM tương tự." |

</frozen-after-approval>

## Code Map

- `src-tauri/src/core/ai/rag.rs` -- `gather_glossary_context` :200 (marker-gated gatherer model), `assemble_prompt` :497 (`tm: Option<&[SimilarSegment]>`; `None` ⇒ `NotBuiltYet` :507, becomes not-asked), `TmInjectionStatus` :115, `{{tm_similar_segments}}` ⇒ `""` :280, Glossary line render :261-269, multi-line block spacing :475, `PromptPieceKind::Tm` :138-150/:317, `InjectionLedger` :166.
- `src-tauri/src/core/tm/mod.rs` -- `SimilarSegment` :20 (add origin side, tier, percent), `PairOrigin::side()` :33-70, `merge_tiers` :214 (AD-18), `load_fuzzy_candidates` :269, `rank_fuzzy_candidates` :294 (truncates to 3 after a percent sort: reuse its scoring, not its cut).
- `src-tauri/src/commands/aiprompt.rs` -- sole call site :416 (`None`), `assemble_and_record_prompt` :381 (stores and `source_lang` in scope), wires `InjectionLedgerWire` :265, TM wire :189-218.
- `src-tauri/src/commands/aitranslate.rs` -- `prepare_translate_call` :243, `prepare_batch_call` :363 (loop :425); `commands/segment.rs` `prepare_tm_fuzzy` :2420 / `score_tm_fuzzy` :2488 (threshold, `match_lang_for_source_lang`, lock split precedent).
- `src-tauri/tests/ai_boundary.rs` -- `aiprompt.rs` rag-name allowlist :228-238; Glossary door count :1178.
- Tests that flip: `tests/ai_rag_contract.rs` :468, :502; `tests/ai_prompt_contract.rs` :182; wire shape :797, :906; determinism :904 stays; fixtures in `tests/frontend/libraryChaptersResetsAiPromptInspector.test.ts`, `tests/frontend/libraryImportResetsAiPromptInspector.test.ts`.
- `src/config/aiprompt.ts` -- `SimilarSegmentWire` :60, `TmInjectionStatusWire` :67, guard `isTmInjectionStatusWire` :187.
- `src/AiPromptInspectorOverlay.vue` -- TM block :288-296 (count only; list pairs like the Glossary list :252-260, tier label, reference tag); `src/aiPromptInspectorState.ts` `glossaryInjectionSummary` :211; `src/panels/AiTranslationPanel.vue` summary :130-137/:390 (record not re-read after translate: add the refresh).
- `src/i18n/vi.json` -- `ai.prompt.summary_*` :101-103, `ai.prompt_inspector.tm_*` :120-122 (`tm_not_built_yet` dies).
- Mockups: `ux-designs/ux-AuraTranslate-2026-08-02/mockups/prompt-inspector.html:130-133` (TM block), `EXPERIENCE.md:390` (summary copy).

## Tasks & Acceptance

**Execution:**
- [x] `core/tm/mod.rs`, `core/ai/rag.rs` -- grow `SimilarSegment`; TM gatherer (marker-gated, all candidates ≥ threshold); pure selection + render + ledger entries with `reference` flag.
- [x] `commands/aiprompt.rs`, `commands/aitranslate.rs` -- pass the gathered TM per Q4; wire fields.
- [x] `tests/ai_rag_contract.rs`, `tests/ai_prompt_contract.rs`, `tests/ai_boundary.rs` -- the matrix through `assemble_prompt` and through `assemble_and_record_prompt` on real stores; flip the 4.6 cases.
- [x] `src/config/aiprompt.ts`, `src/aiPromptInspectorState.ts`, `src/AiPromptInspectorOverlay.vue`, `src/panels/AiTranslationPanel.vue`, `src/i18n/vi.json` -- wire types/guards, TM list with tag, two-count summary, refresh after a call.
- [x] `tests/frontend/aiPromptInspector.test.ts`, `tests/frontend/aiPromptConfigGuards.test.ts`.

**Acceptance Criteria:**
- Given the selection, when the own-first ordering is removed, then only the mixed and enough-own cases go red.
- Given a seeded Work with own and others' pairs, when `ai_prompt_assemble`'s pure fn runs, then the prompt contains the reference label exactly before others' lines and the ledger flags the same pairs.
- Given a finished translate call, when the panel shows the summary, then both counts match the record without reopening the panel.

## Implementation Notes

- TM half of AD-14 lives beside the Glossary half in `core/ai/rag.rs`: `load_tm_rows` (marker-gated, reads both tiers and `tm_fuzzy_threshold` once), `gather_tm_context` (every pair at or above the threshold, AD-18 order, uncut), and the pure `select_tm_pairs`/`render_tm_pairs` inside `assemble_prompt`. Signatures of `assemble_prompt`/`gather_glossary_context` unchanged; `SimilarSegment` grew `side`, `tier`, `percent`.
- `core/tm::fuzzy_pairs_in_candidates` is the uncut scorer over borrowed rows; `rank_fuzzy_candidates` (strip) now calls it and keeps its own sort and cut to 3.
- Selection drops identical (source, target) duplicates (TM keeps them on disk since 7.8) before the cap, own copy first.
- `assemble_and_record_prompt_with_tm` takes a caller-owned `TmRowsCache`; `prepare_batch_call` shares one per batch, single translate and Xem prompt read once per call.
- `TmInjectionStatus::NotBuiltYet` became `NotAsked` (wire `not_asked`); `ai_boundary` allowlist 9 to 13 names.
- Webview: the summary watcher re-reads the record when a single or batch run leaves `generating`; Glossary-not-asked plus TM searched reads "Bộ prompt không truy vấn Glossary cho câu này; đã chèn M câu TM tương tự." (Ice kept it).
- The reference label is a plain literal under a named `// aura-allow-text:` exemption, which `check:i18n` Kiểm A now honours for `.rs` (Ice chose B over the `\u{..}` spelling).
- Ledger: real-app pass and the unmeasured batch lock hold, both Chủ: Epic 7.

## Spec Change Log

## Review Triage Log

- B1 batch cache stale / single re-reads / call site unguarded — false for staleness (prepare finishes under the held lock before any send, nothing writes TM meanwhile) and for single re-reads (Q4); call-site gap folded into V1.
- B2 cache not keyed by body or Work — false: one `prompt_set_name` and one open Work per prepare call.
- B3 no-marker case cannot fail — false: removing the early return makes `load_tm_rows` return rows, the ledger turns `searched`, the case goes red.
- B4 exact-source filter in two layers — low, rejected: the pure assembler guards its own contract for any input; both use raw equality.
- B5 own pair beats a higher-percent others' pair — false: frozen Always rule and AC.
- B6 TM text pasted raw (newlines, markers, length) — low, rejected: same path Glossary text has used since 4.6, no new class.
- B7 Vietnamese label hard-coded — false (Q3 A); its `\u{..}` spelling is O1.
- B8 stale names/docs, percent range asserted against a literal 65 — low, rejected; threshold half folded into V2.
- B9 threshold saturates to 255 — false: `parse_tm_fuzzy_threshold` falls back to 65 outside 50..=99 (`scope_contract.rs:1178`).
- B10 UI: `{percent}%` key, index `:key`, counts-only summary, "0 câu TM", watcher transitions, mocked wiring — low/false, rejected (summary copy is the matrix row; pairs live in Xem prompt); batch watcher branch folded into V3.
- B11 housekeeping: allowlist count, `tm_lookup_failed` reuse, stale comments, missing debt item — false for the count (5 + 8 = 13), low rejected for the rest; the real-app debt item is written at close.
- B12 missing cases (no Work, blank sentence, load failure, cross-tier ties, marker twice) — false for no Work (assembly errors `work.none_open` first), low rejected for the rest; zh folded into V2.
- V1 `prepare_batch_call` never runs with the TM marker — medium, patch: batch case asserting each prepared prompt carries its own pair (read count itself is not observable without a new seam).
- V2 configured threshold and zh path never exercised — medium, patch: non-default threshold case and zh Work case.
- V3 batch branch of the summary watcher untested — low, patch: batch case.
- E1 threshold above 255 — false, see B9.
- E2 newlines inside TM text — low, rejected, see B6.
- E3 inline marker with a multi-line TM block not isolated — false: `needs_isolation = replacement.contains('\n')` applies to every variable (`rag.rs` expand loop).
- E4 cache keyed / marker-less body rescanned per sentence — false (B2); rescan is a string scan with no store read, low rejected.
- E5 batch never sees pairs written earlier in the batch — false, see B1.
- E6 refresh races a record cleared by a Work change — maybe-false, low if true, rejected: would need whether closing a Work clears the Rust record mid-call.
- E7 single and batch generating together — low, rejected: needs both runs at once.
- E8 NFC/whitespace-only difference counts as exact — false: Q2 takes the strip's rule, raw equality.
- E9 `summary_not_asked_tm` only type-level — low, rejected.
- E10 AC "only mixed and enough-own go red" not exact (one-piece case also red) — low, rejected: the fix edits this spec.
- E11 assemble re-reads TM each time — false: Q4 reads once per prepare, assemble included.
- O1 (orchestrator) reference label spelled with `\u{..}` so `check:i18n` cannot see the accented literal — medium, an unnamed exemption; Ice chose a named `// aura-allow-text:` exemption, now honoured by Kiểm A for `.rs` (removing the comment turns that line red).
- O2 (orchestrator) identical (source, target) pairs kept on disk (7.8) can be inserted twice and waste the cap — medium, patch: dedupe in selection, own copy first.

## Verification

**Commands:**
- `cargo test --test ai_rag_contract`, `--test ai_prompt_contract`, `--test ai_boundary`, `--test tm_contract` -- green.
- `npm run test:story 7-11 -- --list`, then the listed vitest files -- green.

**Manual checks:**
- Real-app pass: Xem prompt with mixed pairs, summary after a batch (debt `Chủ: Epic 7`).
