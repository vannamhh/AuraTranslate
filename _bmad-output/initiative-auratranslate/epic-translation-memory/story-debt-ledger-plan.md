---
ticket: 17
title: 'Epic 7 retro R-10 — debt ledger: close, correct and record the retro''s ledger items'
type: 'chore'
created: '2026-10-05'
status: 'done'
route: 'dispatch'
baseline_revision: '31d7b7ebc033523d749c91d68df899298eb5e15f'
review_loop_iteration: 0
context: []
---

<frozen-after-approval reason="human-owned intent — do not modify unless human renegotiates">

## Intent

**Problem:** The Epic 7 retro (`epic-7-retro-2026-10-05.md` §Hạng mục hành động, R-10) found ledger drift: an item that should be closed (`⌘M` vs TM management), a 7.9 item describing a command palette the repo does not have, two missing debt items (7.4 real-use pass, 7.5 yield to Proofreader), and six deferred findings plus the `commands/segment.rs` split that live only in the retro, with no owner.

**Approach:** Edit only `deferred-work.md`, per AGENTS.md §Specs, handoffs, ledger: close and correct in words, append new owned items of at most 5 lines. The first two R-10 parts (wording of the 11.8 ✅ line, a live owner for the 6.7b item) were already done by `461f6e7` before the retro was committed; F2.6 was fixed by R-3 (`354c7ec`, `tm_confirm_import` rejects a plan whose `work_id` is not the open Work). Neither gets a new item.

## Boundaries & Constraints

**Always:** grep the ledger, never read it whole; append new items at the end in the tail's `- source_spec:` / `summary:` / `evidence: … Chủ: X.` shape, English, `source_spec` = the retro file; point at `file::symbol`, not line numbers; never delete or rewrite an existing item's text except via an appended `→` line or an in-place 🔵 dated fix; markers only at sentence start.

**Never:** edit code, `epics.md`, the spine, `sprint-status.yaml` beyond R-10's own status line; touch items owned by other R-items (R-4 measurement items, R-11 real-use items).

**Decision (Ice, 2026-10-05) — owners:** 7.4 real-use pass → `Chủ: Epic 7`; 7.5 yield to Proofreader → `Chủ: Story 9.4` (it carries the AC "gợi ý TM nhường cả hai"); the six code findings (F2.4, F2.7, F2.9, F3.4, F3.5, `segment.rs` split) → `Chủ: Epic 8`, reconsidered when Epic 8 is planned; Kiểm C blocks Epic 8 from closing until each is handled. Spec length 1869 tokens accepted (Ice: keep).

</frozen-after-approval>

## Code Map

- `_bmad-output/initiative-auratranslate/deferred-work.md` -- the only file changed; item ⌘M at the line starting `- ⚠️ **\`⌘M\` sẽ va Quản lý TM ở Epic 7.**`; 7.9 real-app item at `summary: In the real app, unverified: the TM management overlay opened from the command palette`
- `scripts/check-debt-owner.mjs` -- read-only; `HALF_ON_CLOSED_LINE_RE` (`một nửa|một phần|vẫn (còn )?mở|còn mở`) must not appear on a new ✅ line; owner regex accepts `Ice|Winston|Sally|John|Amelia|Murat|Mary`, `Story N.M`, `Epic N`
- `src/commands/index.ts` `tm.manage.open` (`keys: undefined`) and `src/App.vue` `data-tm-manage-open` button -- evidence for closing ⌘M and for the 🔵 on 7.9
- Current pointers for findings (verified at HEAD `31d7b7ebc033523d749c91d68df899298eb5e15f`):
  - F2.4: `commands/segment.rs::accept_tm_fuzzy`, `::accept_tm_exact` re-read via `pair_by_id` and write `pair.target_text`; webview sends only `unitId` (`src/tmFuzzyStripState.ts`)
  - F2.7: `core/tm/tmx.rs::escape_into` drops < 0x20 and U+FFFE/FFFF; `MAX_TMX_BYTES` checked only in `decode_tmx_bytes`, `render_tmx`/`commands/tm.rs::tm_export_tier` uncapped
  - F2.9: `core/tm/mod.rs::load_fuzzy_candidates`, `::load_concordance_candidates`, `::load_manage_snapshot` (identical `{global_rows, work_rows}`), `tmx.rs::distinct_tier_pairs`, `core/ai/rag.rs::TmRows` all wrap `load_all_pair_rows`
  - F3.4: `core/tm/mod.rs::fuzzy_pairs_in_candidates` scores all `global_rows` with one `SimilarityScorer::new(source_text, lang)`, no language filter
  - F3.5: `core/matching/mod.rs::SimilarityScorer` raw text vs `::diff_spans` trim+NFC vs `core/tm/mod.rs::concordance_key` trim+NFC+lowercase
  - F5: `commands/segment.rs` 4557 lines, single file, TM commands split between it and `commands/tm.rs`; template is the Epic 6 AI-6 split of `commands/project.rs`

## Tasks & Acceptance

**Execution:**
- [x] `deferred-work.md` ⌘M item -- append `→ ✅ ĐÃ ĐÓNG 2026-10-05 (Story 7.9) — …` stating TM management opens from the title-bar button with no default chord, so `Mod+M` stays `editor.merge_segments` -- R-10 "đóng :3920"
- [x] `deferred-work.md` 7.9 real-app item -- in-place 🔵 2026-10-05 correction: opened from the title-bar button, not a command palette (the repo has none) -- F4.7
- [x] `deferred-work.md` end -- append 8 items: 7.4 real-use pass (pre-fill on Chapter load, `tm-rule` bar, stays unconfirmed, no `SegmentVersion`, origin kept on confirm-without-edit); 7.5 strip does not yet yield to Proofreader (UX-DR21 order); F2.4; F2.7; F2.9; F3.4; F3.5; split `commands/segment.rs` -- F4.8, F2/F3/F5 deferred findings
- [x] `sprint-status.yaml` -- `epic-7-retro-item-89-…` status `done`, in its own commit after the ledger commit, like `bb3d5e9`/`31d7b7e`

**Acceptance Criteria:**
- Given the edited ledger, when `npm run check:debt-owner` runs, then Kiểm A, B and C are all OK and the total rises by 8, the closed count by 1 and the open count by 7, since the ⌘M item moves from open to closed (baseline: 782 total, 253 open, 338 closed).
- Given each new item, when read, then it has at most 5 lines, one `Chủ:` from the accepted set, and a `file::symbol` pointer.
- Given `git diff`, when inspected, then only appended lines and one 🔵 in-place edit appear in `deferred-work.md`; no other file changes in that commit.

## Implementation Notes

- Result: 790 total, 260 open, 339 closed; Kiểm A, B, C OK. The AC first said open +8; the ⌘M item leaving the open set makes it +7, corrected outside the frozen block.
- Ledger commit `fe9f732947b8e7fdc72cf46aed23884d49d4d0d0`, sprint-status commit `c4563995a64987203a5c7604a2ff149031eba427`. Checking the diff against the retro found four content errors in the first pass: the 7.9 🔵 sentence swallowed half the checklist, F3.5 evidence stated the reverse of the retro (whitespace-only source shows 75% with an all-Equal diff), F2.7 omitted the re-import-as-new-pair half and misread "262 MB cách trần ≈ 2,5%" as above the cap, and the 7.4 pointer named no symbol (now `commands/segment.rs::fill_exact_tm_matches`).

## Spec Change Log

## Review Triage Log

- B1 7.9 🔵 not in place — false: ledger rule keeps the old text and appends a dated 🔵 sentence; 🔵 opens its own sentence.
- B2 ⌘M closure unguarded — false: `conflictFor` runs on the whole registry and fails `register()` if a default chord ever collides with `Mod+M`.
- B3 no proof `check:debt-owner` ran — false: run after the fixes printed A/B/C OK, 790 total, 260 open, 339 closed.
- B4 Story 9.4 AC unverified — false: `epics.md` §Story 9.4 carries "gợi ý TM nhường cả hai".
- B5 F2.7 bundles defects — low, rejected: one retro finding (F2.7) with two halves; a 🟡 line can close one half.
- B6 F3.4 lacks a measurement owner — rejected: owner `Epic 8` is Ice's decision in the frozen block.
- B7 "Reconsidered when Epic 8 is planned" vague — false: the owner is `Chủ: Epic 8`, which the gate accepts; the phrase is context.
- B8 first-pass errors not in triage log — false: found by step-03 verification, not review; recorded in Implementation Notes.
- B9 spec in-review while sprint item done — false: transient; spec goes `done` at the end of this run.
- B10 hashes may be hand-written — false: both copied from `git log` output.
- B11/E2 spec and the four content fixes are in no commit — low, patch: land them in one follow-up commit (`deferred-work.md` + this spec).
- E1 split item has file-level pointer only — low, rejected: a file split is anchored by the file.

## Verification

**Commands:**
- `npm run check:debt-owner` -- expected: all three checks OK, counts as in AC
- `git diff --stat` -- expected: `deferred-work.md` only
