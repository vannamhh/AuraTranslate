---
ticket: 7
title: 'Story 7.7 — Concordance: reverse-search the whole TM inside Panel Lookup'
type: 'feature'
created: '2026-10-03'
status: done
route: 'dispatch'
baseline_revision: '7c8c1d681b7afd3d9904bac233ae877108a6bcc7'
review_loop_iteration: 0
context:
  - '{project-root}/AGENTS.md'
  - '{project-root}/src/AGENTS.md'
  - '{project-root}/src-tauri/AGENTS.md'
  - '{project-root}/_bmad-output/implementation-artifacts/epic-7-context.md'
---

<frozen-after-approval reason="human-owned intent — do not modify unless human renegotiates">

## Intent

**Problem:** The translator cannot ask "how did I translate this phrase before?" — the TM is only consulted per whole sentence (7.4 exact, 7.5 fuzzy), and the dictionary "not found" state (Story 1.17) still owes its pointer to Concordance (FR60, epics §Story 7.7).

**Approach:** A registered, rebindable command `tm.concordance` reads the selection through its own reader, searches both TM tiers in Rust, and shows every hit (source, target, pair origin side + tier) in a new `Concordance` tab of Panel Lookup, placed between `Từ điển` and `Lịch sử`, with empty states of its own.

**Decisions (Ice, 2026-10-03):**
- Q1 ⇒ (R) raw substring in both languages: Chinese `contains` on NFC text, English case-insensitive (NFC + lowercase both sides), no stemming, may match inside a word. Measured: the Matcher (`find_terms`) returned 0 for 18% of 500 Chinese queries whose phrase is literally present; Zh R ≈ 12 ms, En R ≈ 84 ms per tier at 100k rows.
- Q3 ⇒ (b) the dictionary not-found state runs Concordance in the background for the same query and shows "Concordance có N kết quả"; clicking it, or the Concordance chord, opens the tab. The dictionary result never waits for it.
- Q2 ⇒ (α) a selection in a Vietnamese panel (AI Translation, Editor) is mapped through the Glossary in reverse: among the confirmed Glossary marks of the active segment's source, the earliest whose `translation` equals the selection (NFC, trimmed, case-insensitive) gives the Chinese span; that span is highlighted in the source column and Concordance runs on it. No such mark ⇒ a state saying no Glossary term in this sentence translates to the selection. No dictionary reverse lookup.
- Spec size ⇒ keep as one story (~2,300 tokens).

## Boundaries & Constraints

**Always:**
- Tab order is `Từ điển · Concordance · Lịch sử` (`lookup-history-pins.html:103`, Story 1.20 AC5); a test reads the order.
- Hits merge both tiers through `merge_tiers` (AD-18 order: mine first, then Work before Global, then id). No Work open ⇒ Global tier only, never an error.
- The response distinguishes four states the UI renders differently: not searched yet · TM empty (both tiers 0 rows) · no hit for this phrase · hits. TM-empty copy explains TM fills itself on confirm and there is no "add to TM" action.
- The dictionary `not_found` state gains the background count of Q3; a stale count (query changed meanwhile) is never shown.
- The IPC command is `async`, drops the `OpenWorkState` lock before scanning (same shape as `tm_fuzzy_matches`).
- Query source: the selection via `currentSelectionTextForConcordance()`; empty selection ⇒ the current Lookup query; both empty ⇒ not-searched state.
- Result cap: at most 50 hits are shipped, with the total count; the tab says when it shows fewer than the total.

**Never:**
- Read the selection through `currentSelectionText()` (that is the dictionary path; `display` panels return `''` there).
- Point at a capability that does not exist yet (no "Nhập TMX" suggestion — Story 7.10).
- Add an FTS table, a migration, or a new dependency.
- Write to the TM or to any segment.

## I/O & Edge-Case Matrix

| Scenario | Input / State | Expected |
|---|---|---|
| Hits in both tiers | phrase in a Work pair (other) and a Global pair (mine) | Global-mine first, then Work-other; each row shows source, target, side, tier |
| TM empty | both `tm_unit` tables empty | TM-empty state, not the no-hit state |
| No hit | TM has rows, none contain the phrase | no-hit state naming the phrase |
| Over cap | 120 matching pairs | 50 rows + "50 / 120" notice |
| No selection | command with empty selection and empty Lookup query | not-searched state, no IPC call |
| No Work open | Global has pairs | Global hits only |
| Vietnamese selection, Glossary hit | Editor selection `sư phụ`; active source has a confirmed mark `师父` → `Sư phụ` | `师父` highlighted in the source column; Concordance on `师父` |
| Vietnamese selection, no Glossary hit | Editor selection with no matching confirmed mark in the active segment | "no Glossary term here translates to …" state, no IPC call |
| Store error | IPC rejects | error state, not the no-hit state |

</frozen-after-approval>

## Code Map

- `src-tauri/src/core/tm/mod.rs` -- `TmTier` :85, `PairOrigin` :37 `.side()`, `merge_tiers` :204, `load_all_pair_rows` :153, `rank_fuzzy_candidates` :272 (pattern for a pure rank step). Add the concordance load (per-tier SQL prefilter where the semantics allow it) + pure filter/merge/cap here; nothing outside `core/tm` touches `tm_unit` SQL.
- `src-tauri/src/core/matching/mod.rs:625` `find_terms`; `core/glossary/store.rs:1265` `match_lang_for_source_lang` (lang from the Work's `meta.source_lang`, as `segment.rs:2433`).
- `src-tauri/src/core/store/schema.rs:891` `TM_UNIT_DDL` -- read only; no migration.
- `src-tauri/src/commands/segment.rs` -- copy the `prepare_tm_fuzzy` :2391 / `score_tm_fuzzy` :2439 / `wire::tm_fuzzy_matches` :4172 split for `tm_concordance`; DTO beside `TmFuzzyMatch` :2353 (side/tier as `&'static str`).
- `src-tauri/src/lib.rs:970` -- register. Guards that will move: `tests/ipc_argument_contract.rs:530` (floor), `tests/config_invariants.rs:1656` (segment.rs shell census) and :1033 (blocking scans must be async), `tests/ipc_contract.rs` (wire field names).
- `src/config/segment.ts:1131-1199` -- hand-written TM types, guards, `invoke` wrapper; add the concordance trio next to them.
- `src/panels/lookupHistoryState.ts:56` `LookupTab`, `selectLookupTab` :287 -- add `'concordance'`; rewrite the comment at :55.
- `src/panels/LookupPanel.vue` -- tab strip :428-455 (arrow keys cycle three tabs via `lookup.select_tab_*`), the not-found line :587, `.lookup-empty*` classes for the new states; the comment at :407-427 is obsolete.
- New `src/panels/concordanceState.ts` -- query/response/pending/error refs with a sequence guard and `…HasLoaded`-style predicates, shaped like `lookupPanelState.ts:112-235`.
- `src/panels/selectionContract.ts:242` -- add `currentSelectionTextForConcordance()` beside the quick-add reader (reuse `surfaceFor`, ignore role, report the role when Q2 needs it).
- `src/commands/index.ts` -- `tm.concordance` (`keys: ['F3', 'Mod+Alt+N']` — Ice 2026-10-03: F3 from mockup `lookup-real-density.html:97` is skipped inside typing zones such as the Editor cell, so `Mod+Alt+N` is a second default) and `lookup.select_tab_concordance`, handlers in `CommandDeps`; deps factory `src/tmConcordanceCommandDeps.ts` like `src/tmFuzzyCommandDeps.ts`, spread in `main.ts:865`.
- `src/panels/glossaryMarksState.ts` `glossaryMarks` (chapter-wide, joined source) + `glossaryMarksMap.ts:128` `glossaryMarksBySegment` -- already loaded marks carry `translation`, `start`/`end`; the Q2 mapping is webview-only, no new IPC. Unconfirmed marks have `translation: null` and never match. The source-column highlight is a separate state from `glossaryTermHoverState.ts` (mouse-owned) and uses an existing token, no new colour.
- `src/i18n/vi.json` -- `tm.concordance.*` and `panel.lookup.tab_concordance`; side/tier wording may reuse `tm.fuzzy.side_*`/`tier_*` (:893-896).
- `deferred-work.md` -- close L1418-1432 (tab order) and the 7.7 half of L2444-2462 (selection reader) with ✅ lines; L899-901 (`SUBSTRING_FALLBACK_CEILING`, dictionary-only, Concordance does not use it) ⇒ `→ … Chủ: Ice` (needs real selection logs).

## Tasks & Acceptance

**Execution:**
- [x] `src-tauri/src/core/tm/mod.rs` -- concordance load + pure rank (semantics per Q1/Q2), TM-empty flag, total count, cap 50.
- [x] `src-tauri/src/commands/segment.rs`, `src/lib.rs` -- `tm_concordance` async command and DTO; move the guards listed above to live counts.
- [x] `src-tauri/tests/tm_contract.rs` -- the I/O matrix rows through the wired command, both tiers seeded.
- [x] `src/config/segment.ts`, `src/panels/concordanceState.ts`, `src/panels/selectionContract.ts` -- IPC wrapper, state, selection reader.
- [x] `src/panels/concordanceSourceMapping.ts` (new, pure) + `GridPanel.vue` -- Q2 reverse mapping and the source-span highlight.
- [x] `src/commands/index.ts`, `src/tmConcordanceCommandDeps.ts`, `src/main.ts` -- commands and wiring.
- [x] `src/panels/LookupPanel.vue`, `lookupHistoryState.ts`, `src/i18n/vi.json` -- third tab, hit rows, four states, not-found pointer.
- [x] `tests/frontend/` -- tab order, each state, the not-found pointer dispatching the command, the reader returning text on a `display` surface.
- [x] `deferred-work.md` -- the three lines above.

**Acceptance Criteria:**
- Given the tab-order test, when Concordance is moved to the end of the strip, then it goes red.
- Given the wired Rust test, when the command stops reading the Global tier (a real code change), then the both-tiers row goes red and the Work-only rows stay green.
- Given the TM-empty and no-hit tests, when the TM-empty flag is forced false, then only the TM-empty case goes red.
- Given a selection inside AI Translation, when `tm.concordance` runs, then it reads text through `currentSelectionTextForConcordance`, never `currentSelectionText`.
- Given a Vietnamese selection with a matching mark, when the mapping ignores `translation` and takes the first mark, then the mapping test with two marks in the segment goes red.
- Given `tm.concordance` in Settings › Shortcuts, then it is listed and rebindable.

## Implementation Notes

- Match is a raw substring on the pair SOURCE only (trim + NFC + lowercase on both sides, one rule for zh and en); no SQL prefilter, no Work-language lookup, no migration. `core/tm::load_concordance_candidates` reads both tiers once, `rank_concordance` merges through `merge_tiers` and caps at 50.
- Wire `tm_concordance` echoes the query as sent; the webview drops a response whose `query` differs. The dictionary not-found count is a separate probe keyed by query, so a stale count is never shown.
- Q2 applies only to surfaces that register as Vietnamese (`registerSelectionSurface` 4th argument: AI Translation, Grid target column); any other surface, Lookup panel included, is searched as is.
- `F3` was not bindable (`keys.ts` `NAMED_CODES` lacked it) and bare chords are skipped in typing zones, so the default is `['F3', 'Mod+Alt+N']` (Ice).
- The source highlight uses `--color-surface-accent`, is cleared on Work change, chapter change and when its segment leaves `editorSegments`; not on Esc, and not drawn in the Hán Việt view (debt).
- Guards moved to live counts: `REGISTERED_COMMAND_FLOOR` 98 to 104, `config_invariants` segment.rs census, and the TS/token/panel-ref/doc-ref file floors in `scripts/check-*.mjs`.
- Counter-checks (real changes): Global tier unread, `tm_empty` forced false, Concordance moved after Lịch sử, mapping without the `translation` comparison, reader falling back to `role === 'display'`, chapter watch removed; each turned only its intended cases red.
- Ledger: tab-order and selection-reader items closed, `SUBSTRING_FALLBACK_CEILING` reassigned to Ice, four new items (F3 in typing zones, Hán Việt highlight and Esc, real-use pass, no end-to-end release measurement).

## Spec Change Log

## Review Triage Log

- VG1 real panels' `vietnamese` flag untested (direct registration only) — medium — patch: mount GridPanel/AiTranslationPanel test.
- VG2 Lookup tab arrow-key rewiring untested — medium — patch: keydown test per tab.
- VG3 `concordanceHasLoaded` has no caller — low — patch: delete.
- VG4 / B10a `concordanceState` imported twice in `tmConcordanceCommandDeps.ts` — low — patch: merge.
- B1 / E6 probe scans the whole TM on every dictionary miss — false as a defect: Ice chose Q3(b) knowing it costs one scan per miss (frozen Decisions).
- B2 / E8 per-row NFC+lowercase allocation, `İ` fold — low — reject: inside the measured ~84 ms/tier, fix needs a stored folded column (migration, forbidden).
- B3 hit rows do not highlight the phrase — false: no FR60/epic/UX line asks for it; UX docs draw no hit row.
- B4 no paging past 50 — false: the 50 cap with total is the approved spec (Always).
- B5 / E2 null outcome or query mismatch falls back to `not_searched` — low — patch: error state instead (silent-emptiness class).
- B6a mapping is exact-match only — false: Ice's Q2(α) says "equals".
- B6b mapping uses the caret segment, not the segment the selection sits in (Grid target column of another row) — medium — patch: resolve the row from the selection.
- B7 story ids in comments (`config/segment.ts`, five test headers) — low — patch: delete.
- B8 floors bumped to wrong values — false: `judgeFloor` accepts them and every gate is green.
- B9 positional boolean `vietnamese` — low — reject: style, no failing path shown.
- B10b module-level `watch` at import — low — reject: imported once by `main.ts`; vitest isolates modules per file.
- B11a result from Work A survives a Work switch — false: `resetLookupPanel` (called at `libraryChapters.ts:303`, `libraryImport.ts:423`) calls `resetConcordance`.
- B11b / E-claim3 result or probe count stale after a new pair is confirmed — low — reject: needs TM-change events, nothing user-visible beyond one count.
- B12 no live region for the Concordance states — low — patch: same live-region treatment as the record branch.
- B13a no camelCase case for `query` — false: `ipc_argument_contract` scans every registered command (it was red until the invoke existed).
- B13b / E7 / E-claim2 the wire holds `OpenWorkState` while reading every pair (as `tm_fuzzy_matches` does) — maybe-false, medium if true — defer: settle by timing a flush/confirm during a 100k-row probe in a release build.
- B13c unknown origin through `tm_concordance` untested — low — reject: path shared with 7.3's tested `merge_tiers`.
- B14 / E9 the not-searched hint hard-codes F3 (Mission Control on macOS, rebindable) — medium — patch: no hard-coded key in the string.
- B15a `deferred-work.md` lacks a trailing newline — low — patch.
- B15b Windows check `Mod+Alt+N` owned by Epic 7 — low — patch: own item, `Chủ: B7` (AGENTS.md).
- B15c highlight colour owned by Epic 7 not Sally — false: existing token, no new colour decision.
- E1 `pending` unreachable while an older response/error is held — low — patch: clear both when a search starts.
- E3 pointer click re-reads the selection rather than the probed query — low — reject: needs a selection diverging from the Lookup query (history re-lookup); fix needs a new command.
- E4 marks not loaded shown as "no Glossary term" — low — reject: marks load with the chapter; no reachable race shown.
- E5 highlight requires pieces fully inside the span — false: the span is a Glossary span and source pieces split at Glossary span boundaries.
- E10 an unresolvable source selection falls back to the Lookup query — low — reject: no reachable case shown, fix adds a state.
- E11 one-letter query matches nearly everything — low — reject: capped at 50 with total.
- E-claim1 tab switches before the empty check — low — reject: the not-searched state is the intended answer.

## Verification

**Commands:**
- `cd src-tauri && cargo test --test tm_contract` -- concordance cases green.
- `npx vitest run tests/frontend/<concordance files>` -- green.
- `npm run check:commands && npm run check:i18n && npm run check:tokens` -- green.
- Full suite once: this story touches command registration in `lib.rs`.

**Manual checks:**
- Real-use pass (Ice, Epic 7): select a phrase in the source grid, press F3, read hits; repeat in the narrow-layout drawer.

## Acceptance criteria from epics.md

Source: `epics.md` §Story 7.7 (v6, nay ở archive-v6).

**Covers:** FR60

As a người dịch,
I want hỏi *"cụm này trước đây tôi dịch thế nào?"* ngay tại chỗ tôi đang tra từ điển,
So that tôi không phải nhớ hai chỗ khác nhau cho hai loại tra cứu.

**Acceptance Criteria:**

**Given** một cụm từ đang chọn
**When** người dùng gọi lệnh Concordance
**Then** hệ thống tra ngược trên **toàn bộ** Translation Memory

**Given** kết quả Concordance
**When** hiển thị
**Then** đưa vào **Panel Lookup**, cùng chỗ với kết quả từ điển

**Given** mỗi kết quả Concordance
**When** hiển thị
**Then** thấy câu nguồn, bản dịch, và **xuất xứ** của cặp đó

**Given** kết quả Concordance và kết quả từ điển cùng ở Panel Lookup
**When** hiển thị
**Then** phân biệt được với nhau

**Given** Concordance không có kết quả
**When** hiển thị
**Then** trạng thái rỗng nêu rõ lý do, khác với trạng thái rỗng của từ điển

**Given** trạng thái rỗng *"không tìm thấy"* của tra cứu từ điển ở Story 1.17
**When** Concordance đã tồn tại
**Then** bổ sung đường trỏ sang Concordance vào trạng thái rỗng đó
**And** đây là chỗ lời hứa bị hoãn ở Story 1.17 được trả

**Given** Translation Memory còn trống
**When** người dùng mở Concordance lần đầu
**Then** trạng thái rỗng giải thích **cơ chế** — TM tự đầy khi xác nhận câu, không có nút *"thêm vào TM"*

**Given** lệnh Concordance
**When** gọi
**Then** là command đăng ký, gán phím được
