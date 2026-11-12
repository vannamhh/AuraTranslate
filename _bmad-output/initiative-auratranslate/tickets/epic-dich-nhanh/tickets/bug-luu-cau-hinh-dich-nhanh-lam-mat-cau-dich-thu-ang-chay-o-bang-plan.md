---
title: 'Lưu cấu hình dịch nhanh làm mất câu `Dịch thử` đang chạy ở Bảng chờ Glossary'
type: 'bugfix'
ticket: '8'
created: '2026-11-12'
status: 'built'
baseline_revision: 'b0a688db8a47a34e079a30a86d9c324c3af87b28'
route: 'full'
route_source: 'auto'
risk: 'medium'
review: 'quick'
review_source: 'pinned'
lenses_ran: ['quick']
review_loop_iteration: 0
context: []
---

<frozen-after-approval reason="human-owned intent — do not modify unless human renegotiates">

## Intent

**Problem:** In the Glossary queue, saving any quick-config field, saving or deleting the quick key, or toggling `Tự dịch` aborts the running Context translation even when the user clicked `Dịch thử`/`Dịch lại`: the row reverts and the partly paid sentence is lost. The URL import preview already aborts only automatic runs.

**Approach:** The Glossary queue remembers which trigger started the running run and its settings watcher aborts only an automatic one, matching the preview. A shared matrix of trigger × settings change, run on both surfaces, locks the two in step.

**Decision (2026-11-12, Ice, retro Epic 15 A2):** saving quick config or changing `Tự dịch` cancels only automatic runs, never manual ones, on both surfaces.

## Boundaries & Constraints

**Always:** AD-64 §8 run-id cancellation; a pending automatic wait is still cleared on every settings change; `paused`→`none` reset and rescheduling after a settings change stay as they are; Rust untouched (it does not cancel on save).

**Never:** change the other cancel paths (row change, overlay close, `Esc`/`Huỷ`, store purge, chapter change); touch `src/quickAiConfigState.ts` signals; touch Rust, migrations or dependencies; edit the ticket file.

## I/O & Edge-Case Matrix

| Running run | Settings change (save field · save key · delete key · `Tự dịch` on · `Tự dịch` off) | Glossary queue | Import preview blocks |
|---|---|---|---|
| Manual `Dịch thử` | any of the five | completes; text shows with real model; no cancel IPC | same |
| Manual `Dịch lại` | any of the five | completes; new text replaces old | not in matrix (covered by manual batch) |
| Automatic, running | any of the five | aborted: cancel IPC with its `run_id`, row back to `none`, late tokens dropped | aborted, blocks back to empty |
| Automatic, waiting on timer | any of the five | timer cleared; no call until conditions hold again | same |

</frozen-after-approval>

## Code Map

- `src/glossaryQueueState.ts:129-168` -- `activeRunId`, `runningCandidateId`, `runningPrevious`, `abortContextReference()`: add a sibling `runningTrigger` and clear it wherever the other two are cleared (abort, run end at ~301).
- `src/glossaryQueueState.ts:248-254` -- the defect: watcher on `[quickAiAutoEnabled, quickAiConfigRevision]` calls `abortContextReference()` unconditionally.
- `src/glossaryQueueState.ts:268-320` -- `runContextReference(retranslate, trigger)`; set `runningTrigger` beside `activeRunId = mine` (~285).
- `src/importPreviewState.ts:1149-1157` -- reference behaviour: `if (quickBatchRun.value?.trigger === 'auto') cancelImportPreviewQuickBatch()`. Do not change.
- `src/quickAiConfigState.ts:97-157` -- `commitQuickAiConfigDrafts`/`saveQuickAiConfigKey`/`deleteQuickAiConfigKey` bump `savedRevision` only when their adapter (`src/config/quickAiConfig.ts:57,66,75`, `Promise<IpcError | null>`) returns `null`; a field save needs a draft differing from stored. `refreshQuickAiConfigReadiness` drives `quickAiAutoEnabled`.
- `tests/frontend/glossaryContextReference.test.ts` -- `vi.mock('../../src/config/quickAiConfig')` at 30-33 overrides only `quickAiConfigGet`; `automaticRows()` at 697 sets up auto + fake timers; case at 733 is the existing auto-cancel lock (keep green).
- `tests/frontend/importPreviewQuickBatch.test.ts` -- same mock shape at 48-51; `auto()` helper at 683 with `setAuto`; case at 939 covers auto toggle only (keep green).

## Tasks & Acceptance

**Execution:**
- [ ] `src/glossaryQueueState.ts` -- track the running run's trigger; the settings watcher aborts only when it is `'auto'` -- the fix.
- [ ] `tests/frontend/glossaryContextReference.test.ts` -- mock the three save adapters to resolve `null`; add a `it.each` matrix over trigger {manual `Dịch thử`, manual `Dịch lại`, auto running, auto waiting} × change {save field, save key, delete key, auto on, auto off}, each asserting the I/O row -- AC 1, 2, 4.
- [ ] `tests/frontend/importPreviewQuickBatch.test.ts` -- same mocks; matrix over {manual, auto running, auto waiting} × the same five changes, asserting the same cancel/no-cancel outcome -- AC 3.

**Acceptance Criteria:**
- Given a manual run in the Glossary queue, when any of the five settings changes happens and the provider then finishes, then the row shows `kind: 'ai'` with the provider's model and `cancelContextReference` was never called.
- Given each matrix cell, when run on both surfaces, then cancelled/not-cancelled matches across surfaces.
- Given the fix is really removed (watcher back to unconditional `abortContextReference()`), when the Glossary test file runs, then exactly the manual cells go red because the text does not show, and the automatic cells stay green.

## Implementation Notes

- Manual cells capture the post-change state and assert the final translation first, so removing the fix reds them at the translation assert (row stays `none`, or `Dịch lại` falls back to `Cũ.`), not at "cancel called".
- Counter-check, both seams really removed one at a time: Glossary watcher unconditional ⇒ exactly the 10 manual Glossary cells red, 8 automatic green; preview `trigger === 'auto'` guard removed ⇒ the 5 manual preview cells plus the older toggle case red.
- The `aiTranslateActivity` mock ref survives `vi.resetModules()`; the preview matrix `setup` resets `active_count` to 0, as Glossary `fresh` already did.

## Plan Change Log

## Review Triage Log

Pass 1 (quick): high 0 · medium 0 · low 3 · false 0 · maybe-false 0 · rejected 1.
- low · patch — preview `automatic running` cell name differs from the Glossary one; renamed to match.
- low · patch — preview `automatic running` `toBeNull()` could not fail (no event reached block 0; key deleted at `runQuickBatch` start); now emits `started`/`token` before and a late `received` after.
- low · patch — preview `automatic running` asserted cancel count, not `run_id`; now asserts the id from `blocksMock`.
- rejected — unticked Execution boxes in this plan; fix edits the plan.

## Design Notes

"Auto on" while a manual run is running must not start a second run: `scheduleAutomaticReference` already returns while `activeRunId !== null`; the manual × auto-on cell starts with auto off. Automatic × auto-on is unreachable (an automatic run or wait exists only while auto is on), so the automatic rows run the other four changes: 18 cells in the Glossary queue, 13 in the preview. Cells present on both surfaces carry the same name so the two case lists can be compared line by line.

## Verification

**Commands:**
- `npx vitest run tests/frontend/glossaryContextReference.test.ts tests/frontend/importPreviewQuickBatch.test.ts` -- expected: exit 0, no `Errors` line.
- `npm run check:lint` and `npx vue-tsc --noEmit` -- expected: exit 0.
- Counter-check: delete the trigger condition in the Glossary watcher, re-run the Glossary file, record which cells go red, restore.
