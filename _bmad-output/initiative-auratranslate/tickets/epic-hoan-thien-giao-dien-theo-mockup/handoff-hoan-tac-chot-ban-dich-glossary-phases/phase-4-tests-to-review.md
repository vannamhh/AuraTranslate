# Pha 4 Tests -> review

Uncommitted. New: `src-tauri/tests/glossary_unconfirm_contract.rs` (14 cases, both tiers via `TIERS` loop), `tests/frontend/glossaryConfirmStripUndo.test.ts` (25), `e2e/specs/story-12-20-glossary-unconfirm.e2e.mjs`.

Production code changed in this phase (a bug found by e2e, not by vitest): `src/inlineStripEligibility.ts` + new `confirmStripNoticeShown` in `src/glossaryConfirmStripState.ts`. Before: the "confirmed" notice had no grid track once the last pending term was confirmed (`getBoundingClientRect` 0x0). Also doc headers in `schema.rs` now say "14 buoc / 40".

Existing tests moved: pinned/store/glossary/chapter_origin/segment_role/review_harvest/proofread_decision/tm/segment contracts for Global 14 / Project 40 (ladder slice `[..38]` = Project v39; `review_harvest` `len() - 9`; `segment_contract` fake future step 41). Vitest mocks in `stripAnchorSegment` and `proofreaderStripSlots` gained `confirmStripNoticeShown`.

Counter-checks (real removals, restored): AFTER trigger + its name in `GLOSSARY_LIFECYCLE_TRIGGERS` -> 4 red; no column set in first-confirm branch -> 11/14 red; no `bumpGlossaryWriteCount` in `rejectGlossaryCandidate` -> 3 vitest red; eligibility revert -> the track test red.

Verify yourself: `cargo test` full result is in the story report; e2e `story-12-20` green (1 passing) outside sandbox. Machine load was 45-90 so some vitest cases (librarySearchScope, settingsFrame, tmImport) timed out in the full run and passed alone.
