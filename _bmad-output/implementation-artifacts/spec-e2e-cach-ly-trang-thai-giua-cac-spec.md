---
title: 'e2e: no spec inherits the previous spec in-session state'
type: 'bugfix'
created: '2026-09-12'
status: 'draft'
route: 'dispatch'
baseline_commit: 'f5feca0657f983b8d2fde418c3be28eea0803e82'
review_loop_iteration: 0
context: ['e2e/AGENTS.md']
---

<frozen-after-approval reason="human-owned intent — do not modify unless human renegotiates">

## Intent

**Problem:** `npm run test:e2e` is red — 8 spec files, 15 cases — and nightly `e2e (macos-26)` has been red every night since 2026-09-03. Measured 2026-09-12: **12 of the 15 cases pass when their spec runs alone** (`story-5-3-rescan`: 7 red in the full run, 7 green in 2.8 s alone). No selector was renamed, no product code regressed. Cause: **one app process serves all 24 sequential specs** (measured: 1 app pid across 2 sessions), so each spec inherits the previous one's in-memory state — active mode, reading run, library works/rescan/search singletons. A suite reporting false reds is worse than no suite.

**Approach:** Extend the reset seam that already exists for this defect class — `panelReset.mjs`, which resets product module state over an `import()` bridge — to cover the leaking state, and invoke it from the per-spec `before` hook in `wdio.conf.mjs`, so every spec starts from a known mode and known-empty module state. Every reset needed is already exported by product code. **Decision (Ice, 2026-09-12):** also add a fourteenth gate failing when a product `reset*` export is missing from the harness list, so this rot cannot return silently.

## Boundaries & Constraints

**Always:**
- Reset over the existing `import()` bridge, calling `reset*` functions the product already exports; keep its "bridge reaches the real module" self-check (it throws on a stale `?t=` copy).
- For each module added, decide and record whether it needs a paired `LOAD_CALLS` entry. Measured trap: the first `panelReset.mjs` reset without re-firing a load and made the suite **worse by 3 spec files** (run #10, 2026-08-18: 5 passed / 6 failed vs an 8/3 baseline) because three modes live in `<KeepAlive>` and never see a second `mounted`.
- Verify two-sided: the ordered pair `story-5-13-reading-marks` → `story-5-6-library-grid` goes green, **and** none of today's 16 passing spec files is lost.
- Register the new gate in all **three** lists — `package.json`, `ci.yml`, `.githooks/pre-push` — or in the documented exemption table with a reason; `check:gates` enforces this.
- Leave `onComplete`'s positive self-check (`global.db` inside the temp dir) and its negative real-Library signature check untouched.

**Never:**
- Do not relaunch the app per spec. Measured and rejected: 18m51s and 120 s timeouts (`panelReset.mjs:31-36`).
- Do not give each spec its own `$APPDATA`/Library root: both env vars are read once at launch, so with one app process this fixes nothing, and it would need the per-spec relaunch just ruled out.
- Do not touch product code under `src/` or `src-tauri/`. If a needed reset is missing there, stop and report rather than adding it.
- Do not fix `story-5-4-lifecycle` (2) or `story-5-5-progress` (1) — red when run alone too, deferred as G2 in `deferred-work.md`.
- Do not add `continue-on-error`, retries, or a raised timeout to reach green.

</frozen-after-approval>

## Code Map

- `e2e/support/panelReset.mjs` -- the seam. `PANEL_MODULES` (:59) lists 5 panel modules; `LOAD_CALLS` (:104); `resetPanelState()` (:119) with the bridge self-check (:228) and the `__auraPanelResetLoadError` surface (:245). `readingState.ts` and every `src/modes/library*.ts` are **absent** — that absence is the defect.
- `e2e/support/workspace.mjs:125` -- today's **only** caller, inside `openWorkspaceWithWork()`; specs entering Library or Reading never reach it. Keep it — it must still run *after* Work creation.
- `e2e/wdio.conf.mjs:550` -- the `before` hook: once per session, i.e. once per spec file. `specs` (:357) resolves alphabetically — why `story-5-13-*` runs immediately before `story-5-3-*`.
- `e2e/wdio.conf.mjs:136-148` -- doc-comment claiming sequential runs buy "không một lớp đỏ giả nào". Its reason ① covers only two apps running *concurrently* over shared dirs, not one app carrying state *forward in time*. Correct in place; do not delete.
- `src/modes/modeState.ts:38` -- `setMode(next: ModeId)`; `currentMode` (:36) is readonly.
- `src/modes/libraryRescan.ts:207` -- `resetLibraryRescan()`; fixes `story-5-3-rescan`'s "Chưa quét lần nào" precondition (:118-124). No Rust-side rescan state exists (`grep 'never_scanned|last_scan|ScanState' src-tauri/src/` → 0 hits), so the frontend reset suffices.
- `src/modes/readingState.ts` -- `resetReading()` (:137), `resetReadingToc()` (:416), `resetReadingMarks()` (:283). `ensureReadingLoaded()` is latched by `requested` (:106-108) and early-returns forever — why `story-5-12` reads Story 5.11's text.
- `src/modes/libraryWorks.ts:355`, `libraryChapters.ts:473`, `librarySearch.ts:370` -- further candidates; add only what a failing case needs, and measure.
- `e2e/specs/story-5-3-rescan.e2e.mjs:113` -- `parkingLot` uses a **fixed** leaf in shared system temp, cleaned only in `after()` (:265-268), so a crashed run poisons the next; produced the observed `ENOENT … rename … /T/e2e-parking-lot/`. Violates `e2e/AGENTS.md`.
- `scripts/check-gates.mjs:2-11` -- the three-list gate and its exemption table (:82). Model the new gate's self-check on `scripts/check-debt-owner.mjs` Kiểm B, which proves the gate goes red on a seeded violation and not unfairly.

## Tasks & Acceptance

**Execution:**
- [ ] `e2e/support/panelReset.mjs` -- add the leaking modules to `PANEL_MODULES` plus a `setMode('library')` normalisation; comment per module whether it needs a `LOAD_CALLS` pair -- reset-without-reload already cost one full run
- [ ] `e2e/wdio.conf.mjs` -- call the reset from the `before` hook (:550) -- the sole current caller is reachable only through the workspace fixture
- [ ] `e2e/wdio.conf.mjs` -- amend the :136-148 doc-comment with the 2026-09-12 measurement -- a falsified claim must not keep standing
- [ ] `e2e/specs/story-5-3-rescan.e2e.mjs` -- derive the parking-lot name uniquely per run -- a crashed run must not poison the next
- [ ] `scripts/check-reset-coverage.mjs` (new) -- fail when a `src/**/*State.ts` exports a `reset*` the harness list does not call; include a self-check proving it goes red on a seeded violation -- a gate nobody proved red is a gate nobody can trust
- [ ] `package.json` · `.github/workflows/ci.yml` · `.githooks/pre-push` -- register the new gate in all three -- `check:gates` fails otherwise
- [ ] `e2e/AGENTS.md` -- add the pitfall: one app process serves the whole run, so module singletons and the active mode carry across specs

**Acceptance Criteria:**
- Given the full suite on a clean tree, when `npm run test:e2e` runs, then all 12 cases in `story-5-3-rescan`, `story-5-6-library-grid`, `story-5-7-open-chapter`, `story-5-8-reorganise-chapters`, `story-5-9-library-search`, `story-5-12-reading-frontier` pass, and the 16 spec files passing today still pass.
- Given only `story-5-13-reading-marks` then `story-5-6-library-grid`, when the run completes, then both pass.
- Given the full suite, when it completes, then the only red is `story-5-4-lifecycle` (2) and `story-5-5-progress` (1) — exactly the deferred G2 set, nothing else.
- Given a product `reset*` export deliberately removed from the harness list, when `npm run check:reset-coverage` runs, then it fails naming the module and function; when restored, it passes.
- Given `git diff --stat -- src src-tauri`, when read, then the output is empty.

## Implementation Notes

**Outcome 2026-09-13: built, measured, REVERTED by Ice's decision. Acceptance criteria NOT met.**

Measured on Ice's machine, still tree, one full-suite run each:

| tree | spec files | failing cases | wall |
|---|---|---|---|
| baseline `f5feca0` | 16 passed / 8 failed | 15 | 131 s |
| patch (reset from `before` hook) | 12 passed / 12 failed | 20 | 389 s |
| patch + `LOAD_CALLS` | 12 passed / 12 failed | 20 | 372 s |
| after revert | 16 passed / 8 failed | 15 | 179 s |

What the patch got right: the mode-normalisation half worked — `story-5-6` lost its "Library block absent after 30 s" error. The fourteenth gate was built and independently verified: four seeded violations (`resetReading`, `resetLibraryRescan`, `resetLibraryWorks`, `resetLibrarySearch`) each turned it red and it named the function. Its first version scanned only `*State.ts`, which left the four `src/modes/library*.ts` modules — the ones that caused the outage — unguarded; that was caught by seeding the violation the gate claimed to forbid, not by reading its self-check.

Why it was abandoned: four green spec files went red, and the property the spec set out to establish did not hold — on the patched tree `story-5-6` and `story-5-11` were STILL green when run alone, so the full-vs-solo gap stayed open. Two mechanism hypotheses were refuted by measurement: the implementer's ("`onActivated` reloads, so no `LOAD_CALLS` pair is needed") and the reviewer's ("reset without reload, the 2026-08-18 trap") — adding the four `LOAD_CALLS` entries changed 17 seconds and zero cases.

Kept from this run (harness-only, no behaviour change beyond the first item):
- `e2e/specs/story-5-3-rescan.e2e.mjs` — parking-lot directory name is now unique per run instead of a fixed leaf in shared system temp.
- `e2e/wdio.conf.mjs` — the ":136-148" claim that sequential execution buys "không một lớp đỏ giả nào" is corrected in place, with the three measurements and an explicit warning not to rebuild the reverted patch unchanged.
- `e2e/AGENTS.md` — the one-app-process pitfall, and the instruction to run a spec alone before reading its failure as a defect.

Reverted: `e2e/support/panelReset.mjs`, `e2e/wdio.conf.mjs` before-hook wiring, `scripts/check-reset-coverage.mjs`, `package.json`, `.githooks/pre-push`, `.github/workflows/ci.yml`.

The defect is recorded in `deferred-work.md` with **Chủ: Ice**, because the three remaining roads (per-spec app relaunch at 18m51s, a product-side test seam behind the `wdio` feature, or accepting the suite as solo-only and changing how CI invokes it) are all forbidden by this spec's Never clauses and need a planning decision rather than another patch. Status returned to `draft` so a future session re-plans instead of auto-resuming implementation.

## Spec Change Log

## Review Triage Log

## Verification

**Commands:**
- `npm run test:e2e -- --spec e2e/specs/story-5-13-reading-marks.e2e.mjs --spec e2e/specs/story-5-6-library-grid.e2e.mjs` -- expected: 2 spec files passed. Red before the fix.
- `npm run test:e2e` -- expected: 22 of 24 spec files passed, failures only `story-5-4-lifecycle` and `story-5-5-progress`. Record wall-clock against today's 2m11s; a large jump means a relaunch crept in.
- `npm run check:reset-coverage` -- expected: passes, and its self-check proves it can go red.
- `npm run check:gates` -- expected: passes with the new gate in all three lists.
- `npm run check:lint` -- expected: clean.
- `git diff --stat -- src src-tauri` -- expected: empty.
