# Phase 3 (Webview) handoff, story 9-14

State: `vue-tsc --noEmit` clean; eslint clean on touched src files; vitest green for proofread*/proofreader*/threeStripsWire/aiBatchPanelRows/aiChapterRun. Nothing committed.

## Done
- `src/config/proofread.ts`: `ProofreadOutcomeWire` done variant has `scan_id: number`; `isProofreadOutcomeWire` requires `isCount(scan_id)` for `done`; `recordProofreadScan(scanId, workId, scope, reported, filtered, blocked)` sends `{ scanId, workId, ... }`.
- `src/proofreadState.ts`: `recordFinishedScan(scanId, ...)` fed from `result.value.scan_id`.
- `src/ProofreaderStrip.vue`: `scanErrorText(err)` replaces the `tier` param of `ai_proofread.instruction_missing` with existing keys `ai.prompt_inspector.tier_work` / `tier_global`; other errors unchanged. `vi.json` string was already added by Rust phase.
- Tests updated: `proofreadConfig`, `threeStripsWire`, `proofreadState`, `proofreaderStripContent` (new case: missing-instruction message shows tier label for work and global). No TS wrappers for the 3 new commands (9.15).

## For Tests (phase 4)
- Rust items from phase-rust.md still open. Not run: full vitest, `check:i18n`/other gates beyond what is above, e2e proofread spec (needed before push).
- Frontend mocks in several specs (proofreaderStrip, proofreaderStripSlots, proofreadWiring, ...) return `done` without `scan_id`/`blocked`; they bypass wire validation so pass, left as is.
