---
ticket: 3
title: 'Story 7.3 — Dual-scope TM and two-key sort order'
type: 'feature'
created: '2026-10-02'
status: done
route: 'dispatch'
baseline_revision: '8722e3221082205b91da1ba888b494e4e6361143'
review_loop_iteration: 0
context:
  - '{project-root}/AGENTS.md'
  - '{project-root}/src-tauri/AGENTS.md'
  - '{project-root}/_bmad-output/initiative-auratranslate/archive-v6/epic-7-context.md'
---

<frozen-after-approval reason="human-owned intent — do not modify unless human renegotiates">

## Intent

**Problem:** TM exists only as `tm_unit` in `project.db` with no read path. `global.db` has no TM tier, and the AD-18 two-key order (origin first, tier second) is declared but stated by no code. So Stories 7.4, 7.8 and 7.11 would each invent their own union and their own order.

**Approach:** Add the Global TM tier as a `global.db` step that reuses the `tm_unit` DDL. Add one `core::tm` read function that loads both tiers and resolves them through `ScopeResolver::apply_merge`. Its primary key is the `PairOrigin` mine/others projection; the tier stays the secondary key (Work before Global). Pin with tests that confirming writes only the Work tier.

## Boundaries & Constraints

**Always:**
- Union semantics only, through `ScopeResolver` (AD-18, FR57). `core/tm` never names `ScopeKind`, `Semantics`, `resolve_merge` or `resolve_override` (`scope_boundary.rs`).
- Primary key = `PairOrigin::side()` (mine before others), stated as an exhaustive `match`. Secondary key = tier, left to `apply_merge`. Within the same side and tier, keep load order (`id` ascending). The date key is Story 7.8's.
- A stored origin outside the closed set, in either tier, is an error, never a silently skipped row.
- Confirm writes only the Work tier (FR56). The confirm path never receives the global `Store`.
- Migrations are forward-only (AD-25).

**Never:**
- No writer to the Global tier: push-up is Story 7.9 and TMX import is Story 7.10.
- No IPC command, UI, matching, normalization, fuzzy scoring or RAG change (Stories 7.4–7.11). `SimilarSegment` stays untouched.
- No function named with a `GLOSSARY_ONLY_SURFACE` string (`load_tier`, …; open ledger item, owner Epic 7).
- No bare `origin` identifier.

## I/O & Edge-Case Matrix

| Scenario | Input / State | Expected Output / Behavior | Error Handling |
|----------|--------------|---------------------------|----------------|
| Both tiers hit | source `S`: Work `self`, Global `self` | Work then Global | N/A |
| Origin beats tier | Global `self`, Work `other` | Global `self` first | N/A |
| Same origin side | Work `bilingual_import`, Global `other` | Work first (both others) | N/A |
| Same side and tier | two Work `self` pairs, ids 3 and 7 | id 3 then id 7 | N/A |
| No Work open | `ScopeResolver::global_only()`, `work: None` | Global pairs only | N/A |
| Empty | no row for `S` in either tier | empty `Vec` | N/A |
| Unknown stored origin | Global row with `translation_origin = 'x'` | nothing returned | error naming the value |
| Confirm with both stores live | wire `confirm_segment`, global `Store` managed | Work `tm_unit` +1, Global `tm_unit` unchanged | N/A |
| Fresh and upgraded global.db | new file / file at step 10 | `tm_unit` exists, version 11 | N/A |

</frozen-after-approval>

## Code Map

- `src-tauri/src/core/tm/mod.rs` -- `PairOrigin`, `PairSide`, `side()` (:30-66), `insert_pair` (:70). Add the read function and its error type here.
- `src-tauri/src/core/cleanup/store.rs:85-130` -- model to copy: a private per-tier loader, `resolve_two_tiers(resolver, global, work)`, a kind string const instead of `ScopeKind`, `decode_kind` → `FromSqlConversionFailure` for out-of-set values, and an error enum wrapping `StoreError` + `ScopeError`.
- `src-tauri/src/core/scope/mod.rs:330` -- `apply_merge(kind, global, work, primary)`. `resolve.rs:194` sorts by `primary.then(tier)` stably. `TranslationMemory => "translation_memory" : Merge` already exists (`kinds.rs:206`).
- `src-tauri/src/core/store/schema.rs` -- `TM_UNIT_DDL` (:884, project step 27). `GLOBAL_MIGRATIONS` (:757-812, 10 steps): add step 11 with the same const, in the same pattern as `IMPORT_CLEANUP_RULE_DDL`. The doc comment (~:714) says "bảy"; fix it in place when touching it.
- `src-tauri/tests/store_contract.rs:1146` -- the global step-count fixture moves from 10 to 11.
- `src-tauri/tests/tm_contract.rs` -- helpers `work`, `bilingual_work`, `type_text`, `tm_rows`. Wire test `:379` (`mock_builder` + `OpenWorkState`). `open_global` model: `tests/aiconfig_contract.rs:179`.
- `src-tauri/src/commands/segment.rs:2413,3732` -- `confirm_segment` and its wire read only `OpenWorkState`; unchanged.

## Tasks & Acceptance

**Execution:**
- [x] `src-tauri/src/core/store/schema.rs` -- global step 11 = `TM_UNIT_DDL`; update the doc comment. `tests/store_contract.rs` -- the fixture moves to 11.
- [x] `src-tauri/src/core/tm/mod.rs` -- a read function such as `pairs_for_source(resolver, global, work, source_text) -> Result<Vec<TmPair>, TmStoreError>`. `TmPair` carries source, target, `PairOrigin` and tier. Exact `source_text =` equality (7.4 owns normalization).
- [x] `src-tauri/tests/tm_contract.rs` -- one case per matrix row. Seed Global rows with direct SQL on a real `global.db`. The confirm row runs the wire with both `Store` and `OpenWorkState` managed.

**Acceptance Criteria:**
- Given the "origin beats tier" case, when `primary` is really replaced by `None`, then that case goes red and the "same origin side" and "both tiers hit" cases stay green.
- Given the confirm row, when `insert_pair` is really pointed at the global `Store` in the wire, then that case goes red.
- Given the unknown-origin case, when the decode maps an unknown value to `Other`, then that case goes red.

## Implementation Notes

- `pairs_for_source` maps the scope tier with an exhaustive `match` on `core::scope::Tier` (the `core::glossary` precedent), not the `core::cleanup` string compare with a `Global` fallback. That fallback would send every pair to Global silently if the wire string ever changed.
- The step-10 → 11 move also touched fixtures in `pinned_contract.rs` (version, `GLOBAL_MIGRATIONS.len()`, upgrade test) and `glossary_contract.rs` (upgrade test). They are not in the Code Map, so a later global step will hit them too.
- Counter-check 1 (`primary` really set to `None`): `origin_beats_tier…` red; "both tiers hit" and "same origin side" green.
- Counter-check 2 (the wire really writes `insert_pair` to the managed global `Store` after confirm, signature unchanged): the confirm case red, Global count 1 against 0.
- Counter-check 3 (an unknown decode mapped to `Other`): the unknown-origin case red.
- Global `tm_unit` has no writer until Stories 7.9/7.10. Every Global case seeds rows with direct SQL.

## Spec Change Log

## Review Triage Log

- B1 `pairs_for_source` has no production caller -- false: the frozen Never excludes IPC/consumers; 7.4 is the first.
- B2/E9 Global `tm_unit` never filled in the app -- false: the frozen Never assigns writers to 7.9/7.10 (`epics.md` §Story 7.9, 7.10).
- B3/E2 no index on `tm_unit.source_text`, unbounded rows per lookup -- medium, defer: `TM_UNIT_DDL` has no index, so every lookup scans both tables; there is no caller yet, so the hot path is 7.4's (ledger, Chủ: Story 7.4).
- B4/E3 one bad origin row fails the whole lookup -- false: the frozen matrix row "Unknown stored origin" specifies an error and nothing returned.
- B4 temp dirs leak when an assert fails -- low, reject: only on a red test run; same pattern as the rest of `tm_contract.rs`.
- B5 `bilingual_import`/`other` rank not distinguished -- false: ranking `bilingual_import` after `other` puts Global first in `within_the_others_side…`, which goes red.
- B5 dedup across tiers unpinned -- low, reject: `resolve_merge` concatenates and never dedups; keeping both is 7.8's contract.
- B6 comment hygiene, missing doc comments on `pub` types -- false: the added history comments were removed per AGENTS.md; the type contracts are evident from their fields.
- B7 epic context/sprint status not reflecting 7.3 -- false: process files; the spec is excluded from the reviewed diff by design.
- B8 no counter-check evidence -- false for `by_side` (counter-check 1 red, Implementation Notes); `ORDER BY id` removal stays green because SQLite scans in rowid order -- low, reject.
- E1 `work: Some` under a global-only resolver still merges -- low, reject: pre-existing `apply_merge` behavior (Cleanup is the same), no caller passes a mismatched pair.
- E4 error precedence when both tiers are bad -- low, reject: either error names a real bad value.
- E5 no single four-bucket ordering case -- low, reject: each key pairing is pinned by its own case.
- E6 upgrade on a non-empty step-10 DB -- false: `pinned_contract::an_older_global_database_migrates_up_and_keeps_its_rows` now migrates through step 11 with rows present.
- E7 count-only confirm assertion -- false: any Global write makes the Global count 1 and the case red (counter-check 2).
- E8 `TmPair` lacks id/created_at -- low, reject: the date key is 7.8's and can extend the type then.
- V no verification gaps reported.

## Verification

**Commands:**
- `npm run build` then `cargo test --test tm_contract --test store_contract --test scope_boundary --test scope_contract` -- expected: green.
- Full `cargo test` once, because a migration is shared wiring (AGENTS.md) -- expected: green.

## Acceptance criteria from epics.md

Source: `epics.md` §Story 7.3 (v6, nay ở archive-v6).

**Covers:** FR57

As a người dịch,
I want kết quả TM ưu tiên đúng thứ giống văn phong tôi nhất,
So that gợi ý đầu tiên tôi thấy là gợi ý đáng dùng nhất.

**Acceptance Criteria:**

**Given** Translation Memory
**When** mô hình hoá phạm vi
**Then** có **TM riêng theo Tác phẩm** và **TM chung toàn cục**

**Given** một truy vấn TM
**When** phân giải hai tầng
**Then** ngữ nghĩa là **hợp nhất** — trả kết quả cả hai tầng
**And** đi qua `ScopeResolver`

**Given** kết quả TM từ hai tầng
**When** sắp xếp
**Then** khoá **chính** là **xuất xứ** — cặp *của tôi* trước

**Given** kết quả TM cùng xuất xứ
**When** sắp xếp
**Then** khoá **phụ** là **tầng** — Tác phẩm trước Global

**Given** một cặp toàn cục do chính người dùng dịch và một cặp Tác phẩm do người khác dịch
**When** sắp xếp
**Then** cặp **của chính người dùng** đứng trước — vì mục đích của Smart RAG là học văn phong

**Given** người dùng xác nhận một segment
**When** ghi TM
**Then** **chỉ** ghi vào TM **Tác phẩm**

**Given** TM toàn cục
**When** nhận dữ liệu
**Then** chỉ qua thao tác chủ động — nhập TMX, hoặc đẩy một cặp lên tầng toàn cục
