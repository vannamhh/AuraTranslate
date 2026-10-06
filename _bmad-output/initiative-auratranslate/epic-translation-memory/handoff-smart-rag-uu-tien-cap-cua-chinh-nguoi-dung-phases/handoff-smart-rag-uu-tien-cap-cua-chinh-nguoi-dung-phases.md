---
type: handoff
title: "7-11 phase handoffs"
status: done
created: 2026-10-04
skill: bmad-build
---

# 7-11 phase handoffs

## Rust phase

Done, untested in real app. Files touched (all under `src-tauri/`):
- `src/core/tm/mod.rs`: `SimilarSegment` + `side`, `tier`, `percent`; new `fuzzy_pairs_in_candidates` (borrowed rows, uncut, AD-18 order); `rank_fuzzy_candidates` now calls it (strip contract unchanged).
- `src/core/promptset/vars.rs`: `MarkerWarnings.tm_similar_segments_missing` (not on the prompt-set wire).
- `src/core/ai/rag.rs`: `InjectedTmPair`, `TmInjectionStatus::{NotAsked, Searched(Vec<InjectedTmPair>)}` (`NotBuiltYet` gone), `TM_PAIR_CAP = 3`, `load_tm_rows`, `gather_tm_context`, pure `select_tm_pairs`/`render_tm_pairs`. Label is `\u{..}`-escaped (check:i18n rejects accented literals in code).
- `src/commands/aiprompt.rs`: wire types; `assemble_and_record_prompt` unchanged signature, delegates to new `assemble_and_record_prompt_with_tm(.., &mut TmRowsCache)`.
- `src/commands/aitranslate.rs`: `prepare_batch_call` shares one `TmRowsCache` (TM read once per batch).
- Tests: `tests/ai_rag_contract.rs`, `tests/ai_prompt_contract.rs`, `tests/ai_boundary.rs` (allowlist 9 -> 13 names).

### Wire shapes (ledger, from `ai_prompt_assemble` / `ai_prompt_read_record`)
- `ledger.tm = { kind: "not_asked" | "searched", similar_segments: SimilarSegmentWire[] | null }`.
  - `"not_built_yet"` no longer exists: remove it from `TmInjectionStatusWire`, `isTmInjectionStatusWire`, `tm_not_built_yet` in vi.json.
  - `not_asked` => `similar_segments: null` (body has no `{{tm_similar_segments}}`); `searched` with `[]` => "0 cau TM".
- `SimilarSegmentWire = { source_text, target_text, side: "mine"|"others", tier: "work"|"global", percent: number (0..99), reference: boolean }`.
  - `similar_segments` are the INSERTED pairs only (max 3), own first (`reference: false`), then others (`reference: true`). `reference` is true exactly for the pairs under the label.
- `pieces` may now contain `kind: "tm"` (one piece: the whole expanded block, label line included). `PromptPieceKindWire` unchanged.
- Prompt text: own `src -> tgt` lines (arrow is `→`), then, only when others are inserted, one line `Văn phong tham khảo (bản dịch của người khác, không phải văn phong của người dùng):` before all others' lines.

### Behaviour the webview must reflect
- Summary: "Đã chèn N thuật ngữ Glossary và M câu TM tương tự." M = `similar_segments.length` when `searched`; `not_asked` has no M (decide copy: omit the TM half or 0, spec matrix says "0 câu TM" only for searched-empty).
- The record is written by the translate call (single and batch) but the panel does not re-read it: add `ai_prompt_read_record` refresh after a call finishes.
- Inspector TM block: list pairs like the Glossary list, with tier label and a reference tag for `reference: true`; states: not_asked, searched-empty, searched-with-pairs.
- No new IPC command, no new message key from Rust.

### Verified
`cargo test --test`: ai_rag_contract 41, ai_prompt_contract 23, ai_boundary 25 (1 ignored by design), prompt_set_contract 18, tm_contract 131 (1 ignored), ai_translate_contract 67, ai_translate_wire 2 (33 s), all green; `cargo test --lib promptset` 31; `node scripts/check-i18n.mjs` green. Full suite NOT run (phase rule).
Counter-check: replacing own-first selection with one global percent sort turned exactly enough-own, mixed and one-piece cases red; the real-store mixed case stayed green (own pair also happens to score highest there).
Env note: two `cargo test` runs stalled in exec (process state U, 0% CPU) under machine load; re-running the built binary directly passed.

## Webview phase

Done, no `src-tauri/` change. Files: `src/config/aiprompt.ts` (SimilarSegmentWire +side/tier/percent/reference, guard checks all; `not_built_yet` -> `not_asked`), `src/aiPromptInspectorState.ts` (`GlossarySummary` carries `tmCount: number | null`; null = TM not asked, 0 = searched-empty), `src/AiPromptInspectorOverlay.vue` (TM list with tier, percent, reference tag; `data-aip-tm-kind` not_asked|searched), `src/panels/AiTranslationPanel.vue` (summary copy with two counts; `watch` on single/batch translate state leaving `generating` -> `refreshAiPromptRecord()`), `src/i18n/vi.json` (+`summary_asked_tm`, `summary_not_asked_tm`, `tm_not_asked`, `tm_reference_tag`, `tm_percent`; `tm_not_built_yet` removed).
- Summary when Glossary not asked but TM searched: "Bộ prompt không truy vấn Glossary cho câu này; đã chèn M câu TM tương tự." (own decision, not in spec).
- Batch: record holds the last sentence's assembly, so the summary after a batch reflects that sentence.
- Tests: aiPromptInspector (53), aiPromptConfigGuards, two library*ResetsAiPromptInspector files green; vue-tsc, eslint, check:i18n/tokens/commands green. Counter-check: neutering the refresh in the watch turned only the new refresh case red.
- Full vitest suite not run.
