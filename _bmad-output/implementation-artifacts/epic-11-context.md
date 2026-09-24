# Epic 11 Context: Trả nợ nền — đóng nợ đã hoãn của Epic 1–6 trước khi xây tiếp

<!-- Compiled from planning artifacts. Edit freely. Regenerate with compile-epic-context if planning docs change. -->

## Goal

A 2026-09-23 debt-ledger sweep left a batch of items that are still valid against the codebase, all inside the Epic 1–6 foundation (check tooling, e2e infrastructure, dictionary lookup, Glossary, editor/segment/write layer, import pipeline, Library). No remaining feature epic touches this code. Epic 11 adds no user-facing capability; it exists to close or explicitly dispose of that backlog before Epic 7 (Translation Memory) builds on the same foundation. It runs immediately after Epic 4 and before Epic 7, even though its block sits at the end of `sprint-status.yaml` — the file's written ordering note governs, not read order (same precedent as Epic 4).

Several debt-resolution sessions have run since the 2026-09-23 sweep and already closed or reassigned much of this backlog, so live counts in `deferred-work.md` no longer match the sizing in the change proposal. Task 0 of every story must re-read its items against current HEAD rather than trust any historical count.

## Stories

- Story 11.1: Trả nợ cổng và công cụ kiểm (`scripts/check-*`, hooks, lint)
- Story 11.2: Trả nợ hạ tầng e2e và bộ chạy test
- Story 11.3: Trả nợ tra cứu và dữ liệu từ điển (Epic 1)
- Story 11.4: Trả nợ Glossary (Epic 3)
- Story 11.5: Trả nợ editor, segment và tầng ghi (Epic 2, `core/store`)
- Story 11.6: Trả nợ đường nhập và Library (Epic 5, Epic 6)
- Story 11.7: Trả nợ nền giao diện dùng chung (command registry, focus, keybindings, a11y) và module AI (Epic 4)

## Requirements & Constraints

- No FRs are covered by this epic; nothing here belongs in `epics.md`/`prd.md` as a new capability.
- Every story inherits the same acceptance blocks: Task 0 re-reads each of its debt items against HEAD (an item may have self-closed or changed shape); on `done`, `check:debt-owner` requires each item to end `→ ✅ ĐÃ ĐÓNG` (with code/test evidence), `→ KHÔNG LÀM <date> (Story 11.N) — <reason>`, or `→ … Chủ: <new specific owner>` — the gate goes red if an open item still points at a `done` story, so no new gate is needed; a "reopen when condition X" item whose condition hasn't happened gets `KHÔNG LÀM` or reassignment to `Chủ: Ice` with the condition stated, never speculative code; an item that would require changing an architectural invariant stops the story and goes to `Chủ: Winston` for a new AD.
- Per-story item lists come from `grep 'Chủ: Story 11.N' deferred-work.md` — never copy that list into a spec or elsewhere. Standard `deferred-work.md` conventions apply (5-line item cap, `→` closing words, last `Chủ:` wins).
- Execution order is fixed: `1 → 2 → 3 → 5 → 6 → 4 → 11 → 7 → 8 → 9 → 10`; Epic 7's matcher/gate/e2e work assumes this epic is done first.
- Background only (already fixed, not epic-11 work): two `epics.md` inaccuracies tied to earlier debt items were corrected alongside adding this epic — the Story 1.10b dictionary-layer note is now marked settled, and the Story 1.12 note now says `core/dict/**` does **not** call the shared matcher (AD-17, AD-44 ③), with `matching_boundary.rs` enforcing that boundary. A stale message in `src-tauri/tests/matching_boundary.rs:325` referencing an outdated `epics.md` line is a 🟡 item owned by Story 11.3.

## Technical Decisions

- AD-17: one shared Matcher backs Glossary (FR51) and TM (FR61); the English **dictionary** lookup path deliberately does not call it (relevant to 11.3).
- AD-44: English lookup routes by query shape and uppercase key, not stemming; five documented failure modes (cross-language AD-26 leakage, Work-language routing causing silent-empty results, case sensitivity, stemming patched into the hot path, per-language instead of per-file `DictionarySource` adapters) — all live in Story 11.3's territory.
- AD-25: dictionary data is a versioned, checksummed artifact via GitHub Release + `dict-manifest.toml`, never `.db` in git — governs 11.3/11.6 build/fetch items.
- AD-36: Glossary is a one-directional three-state lifecycle (candidate → pending-translation → finalized), only finalized entries are prompt-eligible, and HanViet reads through `DictionarySource` rather than being reimplemented — governs 11.4.
- AD-31 and AD-47: segment state machine, and translation-origin baseline is the most recent non-user write (not the load event); every non-typing write to `target_text` must set both the baseline and the origin column together — governs 11.5.
- AD-43 (chapter citation fields are the data, formatted source block built only at export) is background for 11.6. AD-34 (UI actions register in `CommandRegistry` before binding to input) and AD-11 (all writes go through the owning store's `store::Writer`) are background for 11.7/11.5. The bare identifier `origin` is banned — four disjoint concepts (translation/AD-47, Glossary entry/AD-36, source document/AD-43, dictionary citation/FR30) must stay self-distinguishing.
- This epic adds no new gate; `check:debt-owner` (`scripts/check-debt-owner.mjs`) just needs to stay green as stories close.

## Cross-Story Dependencies

- The seven stories are scoped by ledger ownership group, not by feature dependency, and can proceed independently; each is gated individually by `check:debt-owner` at its own `done` transition.
- The whole epic is a prerequisite for Epic 7; Story 11.3 specifically must land before Story 7.7 (Concordance), which builds on the lookup path 11.3 patches.
- A handful of items from the same 2026-09-23 sweep were assigned directly to existing Epic 7/10 stories instead (e.g. matcher NFD/NFC and an `isTmFilled` stub to 7.4/7.6, packaged-app NFR measurement items to 10.9) — out of scope here, not duplicated.
