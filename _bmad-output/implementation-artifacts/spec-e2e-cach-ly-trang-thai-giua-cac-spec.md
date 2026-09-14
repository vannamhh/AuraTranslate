---
title: 'e2e: no spec inherits the previous spec in-session state'
type: 'bugfix'
created: '2026-09-12'
status: 'done'
route: 'dispatch'
baseline_commit: '574c8699fa3330379b4e7514f503209d204d9adf'
review_loop_iteration: 1
context: ['e2e/AGENTS.md']
---

<frozen-after-approval reason="human-owned intent — do not modify unless human renegotiates">

## Intent

**Problem:** `npm run test:e2e` gives false reds: the stock full suite is 16 passed / 8 failed, identical to nightly `e2e (macos-26)` (red every night since 2026-09-03), while the failing specs pass when run alone. Measured 2026-09-14: what carries from one spec file to the next has three layers: frontend module and `<KeepAlive>` state; Rust in-process state (`OpenWorkState`, the Library index); and on-disk state in the one shared `$APPDATA` and Library root (the persisted mode restored at `src/main.ts:946`, `glossary_scan_threshold`). Relaunch alone gave 17 / 7; relaunch plus fresh dirs gave 22 / 2. A suite reporting false reds is worse than no suite.

**Approach:** **Decision 1a (Ice, 2026-09-14):** after every spec file the harness stops the app and points the next launch at a fresh `$APPDATA` and Library root, so every spec file starts in a new process on empty dirs, as when run alone. Harness-only, in `e2e/wdio.conf.mjs`; relies on `@wdio/tauri-service@1.3.0` respawning a dead embedded app with `process.env` read at spawn.

**Decisions (Ice, 2026-09-14):**
- 2a: `onComplete` runs its positive `global.db` check and its negative real-Library check over every dir pair an app ran against, not only the first.
- 3a: the fourteenth gate (decision of 2026-09-12) is dropped. No reset list is the mechanism any more; a relaunch that stops happening fails loudly, and 2a catches a relaunch that ignores the fresh dirs.
- 4a: done means three consecutive full runs at exactly 22 / 2 on a still tree. Any other red is diagnosed or recorded as owned debt, never retried away.
- Superseded by this re-plan, with the evidence in Implementation Notes and Design Notes: the 2026-09-12 Approach (extend `panelReset.mjs`), its `LOAD_CALLS` and three-list gate clauses, the "leave `onComplete` untouched" clause, and the two Never clauses forbidding per-spec relaunch and per-spec dirs (the 18m51s they cited was a full-suite run, not a relaunch cost).

## Boundaries & Constraints

**Always:**
- Stop only the process listening on the embedded port whose command is `APP_BIN`; never a process this run did not launch.
- Keep the data-dir guards at least as strong as today for every launch, per 2a.
- Verify two-sided: the ordered pair `story-5-13-reading-marks` → `story-5-6-library-grid` goes green, **and** none of today's 16 passing spec files is lost.
- Keep `resetPanelState()` and its caller in `openWorkspaceWithWork()`: it still resets between Works inside one spec file.

**Never:**
- Do not touch `src/`, `src-tauri/`, `package.json`, `.githooks/` or `.github/`.
- Do not patch or fork `@wdio/tauri-service`.
- Do not fix `story-5-4-lifecycle` (2) or `story-5-5-progress` (1) — red when run alone too, deferred as G2 in `deferred-work.md`.
- Do not add `continue-on-error`, retries, or a raised timeout to reach green.

</frozen-after-approval>

## Code Map

- `node_modules/@wdio/tauri-service/dist/esm/index.js` (1.3.0, read-only) -- `onWorkerStart` calls `ensureEmbeddedServersHealthy()` (:2487); a dead port 4445 triggers `restartEmbeddedServer` (:2515), which respawns via `startEmbeddedDriver`, whose child env is `{ ...process.env, ...options.env }` built at spawn. So killing the app in the launcher's `onWorkerEnd` and rewriting `process.env` yields a new process on new dirs. Measured: 23 restarts for 24 spec files.
- `node_modules/@wdio/cli/build/index.js` (read-only) -- `runLauncherHook` (:354-371) logs and swallows every hook error except `SevereServiceError`; `onWorkerEnd` is called through it with `(rid, exitCode, specs, retries)` (:1207); a `SevereServiceError` becomes a `HookError`, and `_workerHookError` then resolves the whole run with exit 1. `webdriverio` 9.30.1 exports `SevereServiceError` (tauri-service imports it at `index.js:8`).
- `src-tauri/src/lib.rs:841`, `:899` -- `open_global_store` runs in app setup, so every pair an app launched against has `global.db` whether or not its spec file passed.
- `e2e/wdio.conf.mjs` -- `export const config` (:385); env names (:214 `AURATRANSLATE_E2E_DATA_DIR`, :225 `AURATRANSLATE_E2E_LIBRARY_ROOT`); `APP_BIN` (:373); `onPrepare` allocates ONE dir pair into `process.env` (:400-414); `onComplete` guards read the single `dataDir` / `libraryDir` / `realLibraryBefore` (:459-575); no `onWorkerEnd` exists yet.
- `e2e/wdio.conf.mjs:150-178` -- the 2026-09-13 correction block; its mechanism ("state cấp module") is refuted: relaunch alone leaves `story-5-6` red. `:180` says "~3 phút". Correct in place, do not delete.
- `src/main.ts:946` restores `config.mode` from `global.db` at startup; `:1049` persists every change -- why `story-5-13` (ends in reading) still reddens `story-5-6` across a relaunch. Guarded product behaviour (`scope_contract.rs::the_last_mode_survives_a_write_and_a_reopen`); do not touch.
- `src-tauri/src/commands/project.rs:4452` `OpenWorkState`; `src-tauri/src/lib.rs:1143-1172` Library index rebuilt only at startup -- in-process Rust state that fresh dirs alone would not clear; why 1a also relaunches.
- `e2e/specs/story-3-5-review.e2e.mjs:58-61` -- persists `glossary_scan_threshold` = 7 and never restores it: an on-disk leak that fresh dirs remove.
- `e2e/support/panelReset.mjs:23-24` -- rejects per-spec sessions citing "18m51s", which was the ninth full-suite run of 2026-08-18 (`wdio.conf.mjs:82-84`), never a relaunch measurement. Keep `resetPanelState()` and its caller `e2e/support/workspace.mjs:125`: it still resets between Works inside one spec file.
- `e2e/AGENTS.md:15` -- the one-app-process pitfall; false under 1a.
- `_bmad-output/implementation-artifacts/deferred-work.md:11605-11638` -- G1 entry; its "live in-process state, NOT `$APPDATA`" and "18m51s" claims are refuted. `:11580-11603` G2 becomes verifiable by a full run once this lands.

## Tasks & Acceptance

**Execution:**
- [x] `e2e/wdio.conf.mjs` -- add `onWorkerEnd(cid, exitCode)`: SIGTERM only a pid listening on the embedded port (`Number(process.env.TAURI_WEBDRIVER_PORT) || 4445`, as tauri-service resolves it) whose command is exactly `APP_BIN` or `APP_BIN` followed by a space; wait for the port to close; only then capture the boundary real-Library signature; record the pair currently in `process.env` as used, together with this worker's `exitCode`; allocate a fresh data dir and Library root into `process.env`. Any failure in this hook (port still held after 15 s, an unexpected `lsof`/`ps` error) must throw `SevereServiceError` from `webdriverio` -- `@wdio/cli` swallows every other hook error, so a plain throw lets the next spec file run silently on the old app and dirs
- [x] `e2e/wdio.conf.mjs` -- make `onComplete` run the negative guards and the positive `global.db` guard over every used pair, gating the positive guard on that pair's worker `exitCode === 0` rather than the whole-run exit code; collect every failure, delete every pair used or not, then throw one error naming every failing dir -- the suite is structurally red while G2 is open, so a whole-run gate never fires, and a first-failure throw skips later pairs and leaks their dirs
- [x] `e2e/wdio.conf.mjs` -- correct `:150-178` and `:180` in place with the 2026-09-14 table, stating only what was measured and the guarantees the code actually gives (fatal hook failures, per-worker gating); leave no "not yet measured" line behind -- a refuted mechanism must not keep standing, and neither may a new unverified one
- [x] `e2e/support/panelReset.mjs` -- correct the rejection at `:23-24` with what 18m51s measured and the measured relaunch cost, without attributing the 2026-08-18 timeouts to any cause (never measured) -- the rejection rested on a number that measured something else
- [x] `e2e/AGENTS.md` -- replace the one-app-process pitfall: each spec file gets a new app process and fresh dirs; state still carries between cases inside one file; do not point at text that no longer exists
- [x] `_bmad-output/implementation-artifacts/deferred-work.md` -- close G1 with the table, correct its two refuted claims with a dated note, and mark G2 verifiable; no unmeasured causal claims

**Acceptance Criteria:**
- Given a clean, still tree, when `npm run test:e2e` runs three times in a row, then every run has 22 of 24 spec files passing and the only failures are `story-5-4-lifecycle` (2) and `story-5-5-progress` (1).
- Given `story-5-13-reading-marks` then `story-5-6-library-grid` in one invocation, when it completes, then both pass.
- Given the `onWorkerEnd` body removed (removal control), when that pair runs, then `story-5-6` is red again with the "khối Tác phẩm" 30-second message; when restored, green.
- Given the fresh-dir env write skipped for exactly one relaunch in the middle of a run of at least four spec files that also contains a failing spec file (seeded), when the run completes, then `onComplete` fails naming that dir, the pairs after it are still checked, and no `auratranslate-e2e-*` dir created by the run remains.
- Given the kill step skipped while the env write is kept (seeded), when the run reaches the first spec-file boundary, then the run ends there with the `SevereServiceError` message instead of starting the next spec file.
- Given `git diff --stat -- src src-tauri package.json .githooks .github`, when read, then the output is empty.

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

**Re-plan 2026-09-14.** Ice answered the four planning questions (1a, 2a, 3a, 4a) and approved. `baseline_commit` was reset from `f5feca0` to `574c869` by Ice's decision, overriding the workflow's keep-existing-baseline rule: `f5feca0..574c869` contains `bd15661` (this spec's 2026-09-13 kept work, already committed) and `574c869` (unrelated `store_contract.rs` and `spec-ca-wal-do-tren-windows.md`), which would put a `src-tauri/` change this spec forbids into its review diff.

**Review loop 1 re-derivation, 2026-09-14.** A fresh implementer rebuilt Tasks 1-6 from the amended spec. After independent verification the orchestrator corrected doc-only text the implementer had written: `deferred-work.md` claimed three consecutive full runs when one existed, and it and `panelReset.mjs` again attributed the 2026-08-18 18m51s to specific causes (never measured) and compared walls across days; `wdio.conf.mjs` cited `@wdio/cli` `index.js:317-371` and tauri-service `:1808-1819` (actual `:354-371`, `:1811-1823`); `e2e/AGENTS.md` said "one `pid` across 24 spec files" where the measurement was one pid across two sessions.

## Spec Change Log

**Review loop 1, 2026-09-14 (bad_spec).** Trigger: the edge-case finding that the positive `global.db` guard is gated on the whole-run exit code, plus the verified fact that `@wdio/cli` swallows every hook error except `SevereServiceError` (`node_modules/@wdio/cli/build/index.js:354-371`, `:1207`). Together they make both premises of Decision 3a false on every canonical run: a relaunch that stops is not loud (the 15 s throw is only logged and the next spec file reuses the old app and dirs), and 2a never fires (G2 keeps every full run red). Amended: Tasks 1-6, the fourth acceptance criterion (split into a mid-run seeded check on a red run and a held-port seeded check), Code Map (hook error handling, `global.db` opened at startup), Verification. Known-bad state avoided: a harness whose data-dir guard and relaunch-failure alarm are both inert on every real run. Patch-level findings from the same review are folded into the amended tasks: first-failure loop abort and leaked dirs, port read from env, boundary capture after the kill, exact `APP_BIN` match, doc accuracy. Code reverted to `574c869`; the reverted implementation is saved outside the repo as a patch.
KEEP: the kill, port-wait and env-rewrite mechanism; its pid filter via `lsof -ti tcp:PORT -sTCP:LISTEN` plus `ps -p PID -ww -o command=`, with `lsof` exit status 1 meaning no listener; `usedPairs` recorded in `onWorkerEnd`, trailing pair deleted unguarded; the rolling per-pair boundary Library signature; the 18m51s correction in `panelReset.mjs`; the G1 closure and G2-verifiable structure in `deferred-work.md`; the removal-control and seeded-copy verification method with logs kept. That implementation gave 22 / 2 on three consecutive full runs, a red removal control, a green pair and a red seeded guard.

## Review Triage Log

| # | source | finding | verdict | evidence | route |
|---|---|---|---|---|---|
| 1 | edge-case | positive `global.db` guard gated on whole-run `exitCode`, inert while G2 keeps the run red | high | `onComplete` asserts only if `exitCode === 0`; all five full runs today exited 1; this guard alone protects the runner's real `$APPDATA` | bad_spec |
| 2 | edge-case | a throw in `onWorkerEnd` after the pair is recorded leaves env unrewritten | medium | `runLauncherHook` swallows non-Severe errors (`@wdio/cli` `index.js:354-371`, `:1207`); the old app keeps port 4445, tauri-service sees it healthy and reuses it | bad_spec |
| 3 | edge-case | ESRCH if the pid exits between `ps` and `process.kill` | low | real but a millisecond window at a spec boundary; fix adds a guard | reject |
| 4 | edge-case | `lsof`/`ps` called without a timeout | low | no hang observed in 30 relaunches; fix adds a parameter | reject |
| 5 | edge-case, blind | `EMBEDDED_PORT` hardcoded, ignores `TAURI_WEBDRIVER_PORT` | low | tauri-service resolves option, env, then 4445 (`index.js:1812-1823`); if set, nothing is killed and specs share the old app; one-line fix | patch (folded into Task 1) |
| 6 | edge-case | boundary signature captured before the kill | low | a write during shutdown is attributed to the next pair, and after the last spec file it is never compared; reorder | patch (folded into Task 1) |
| 7 | verification-gap, blind | `onComplete` loop stops at the first failing pair | medium | `try/finally` without `catch` inside `for`: later pairs unchecked and their dirs leaked; contradicts Task 2 "delete every pair" | patch (folded into Task 2) |
| 8 | blind | seeded fourth criterion used two spec files, so a mid-array failure was never exercised | low | the injected pair was the last used pair | patch (folded into acceptance criteria) |
| 9 | blind | `command.startsWith(APP_BIN)` has no boundary | low | a process named `APP_BIN` plus a suffix on the embedded port would be killed; direct correction | patch (folded into Task 1) |
| 10 | blind | `lsof`/`ps` are POSIX-only | low | the suite has never run on Windows (owned debt); fix adds a platform branch | reject |
| 11 | blind | no SIGKILL fallback, suggested as the `story-5-7` flake cause | false | every prototype relaunch logged `port closed=true` in about 1 s, including run E2 where `story-5-7` failed | reject |
| 12 | blind | no stale-process check on port 4445 before the first spec file | maybe-false | first launch is tauri-service `onPrepare`, unchanged by this diff; settle by starting a run with 4445 held; if true, low | reject |
| 13 | blind | `libraryDir` lost its doc-comment | false | the merged doc-comment above `dataDir` names both dirs of the pair | reject |
| 14 | blind | up to 48 temp dirs live for the whole run | low | all removed at `onComplete` (0 left after five runs); incremental deletion conflicts with deferred guards | reject |
| 15 | orchestrator | new `wdio.conf.mjs` block says the three runs were not yet measured, a 3a sentence is left half-written, and it asserts the false "fails loudly" | medium | diff lines 340-351 of the review patch; contradicts Verification and finding 2 | patch (folded into Task 3) |
| 16 | orchestrator | `panelReset.mjs` and `deferred-work.md` attribute the 2026-08-18 timeouts to four noise sources | low | never measured; `wdio.conf.mjs:66-68` refuses exactly such attributions | patch (folded into Tasks 4, 6) |
| 17 | orchestrator | `e2e/AGENTS.md` bullet points at "the claim below", which it replaced | low | the old bullet is deleted in the same hunk | defer (agent-context file; moot, file re-derived under Task 5) |
| 18 | orchestrator | positive guard message says three causes and lists four | low | diff line 740-745 of the review patch | patch (folded into Task 2) |
| 19 | verification-gap (loop 1) | `onComplete` loop continuation and the `SevereServiceError` abort have no automated, repeatable check | medium | no test imports `wdio.conf.mjs`; `vitest` covers `tests/frontend/**` only; the guards in this file were always verified by hand-seeded runs (the 2026-08-11 `global.db` guard too), so the gap class predates this change | defer |
| 20 | blind (loop 1) | `SevereServiceError` imported from `webdriverio`, which `package.json` does not declare | medium | two copies installed: root `webdriverio` 9.30.1 (resolved by `@wdio/cli` and this file) and `@wdio/tauri-service/node_modules/webdriverio` 9.30.0; `runLauncherHook` uses `instanceof`, so a future nested copy under `@wdio/cli` silently restores error swallowing; declaring it needs `package.json`, which the frozen Never excludes | defer |
| 21 | blind (loop 1) | `embeddedPort()` comment claims tauri-service's full order but omits the service-option tier | low | this config passes no service options (`services: ['@wdio/tauri-service']`), so the tier is latent; the comment overstates; direct correction | patch |
| 22 | blind (loop 1) | positive `global.db` message omits a silent store-open failure | low | `open_global_store` logs to stderr and returns on `create_dir_all` or `Store::open` error (`src-tauri/src/lib.rs:899-940`) | patch |
| 23 | blind (loop 1) | pairs do not record which spec file used them (`cid` unused) | low | real; a developer meets it only when a guard fires, and correlating by the `[e2e]` log lines works; fix adds a field | reject |
| 24 | blind (loop 1) | cleanup after a `SevereServiceError` abort is not guaranteed | false | held-port control: run aborted in `onWorkerEnd`, afterwards 0 e2e temp dirs from the run and no app process; only `onComplete` deletes pair dirs, so it ran | reject |
| 25 | blind (loop 1) | Design Notes pair rows read "1 / 1" without labels | low | cosmetic; fix edits this spec | reject |
| 26 | blind (loop 1) | G1 closure in `deferred-work.md` does not say which verification block it cites | low | this spec's §Verification holds two reverted round-0 blocks before the loop 1 block; direct correction | patch |
| 27 | blind (loop 1) | fourth-bar acceptance criterion reads as a standing rule | low | fix edits this spec | reject |
| 28 | blind (loop 1) | triage rows 15 and 18 cite lines of a patch kept outside the repo | low | fix edits this spec | reject |
| 29 | blind (loop 1) | run E2's `story-5-7` red has no owned follow-up | low | Decision 4a asks unexplained reds to be diagnosed or owned; the only `story-5-7` entry in `deferred-work.md` (line 8633, 2026-08-29) is about something else; green in 5 solo and 7 later verified full runs | defer |
| 30 | edge-case (loop 1) | `realLibrarySignature()` in `onWorkerEnd` can throw a plain error after the kill | low | `existsSync`/`statSync`/`readdirSync` on `~/Documents/AuraTranslate` (`wdio.conf.mjs:336-341`) can raise EPERM; `@wdio/cli` swallows it, env never rotates; contradicts Task 1 "any failure" | patch (with 31) |
| 31 | edge-case (loop 1) | `allocateFreshPair()`'s `mkdtempSync` can throw a plain error | low | same swallow path, e.g. ENOSPC; same root cause as 30: only the kill and port-wait paths convert to `SevereServiceError` | patch (with 30) |
| 32 | edge-case (loop 1) | `embeddedPort()` omits the option tier and may parse the env var differently from `getEmbeddedPort` | low | option tier unreachable in this config; see row 21, same fix | patch (with 21) |
| 33 | edge-case (loop 1) | unguarded `rmSync` inside the `onComplete` loop can abort it | low | `rmSync` with `force: true` ignores ENOENT and throws only on EACCES/EBUSY in a temp dir this run created; fix adds a guard | reject |

## Design Notes

Measured 2026-09-14 on Ice's machine, still tree at `574c869`, prototypes as wrapper configs outside the repo (no repo change):

| run | spec files | wall |
|---|---|---|
| stock config, full suite (D) | 16 passed / 8 failed | 155 s |
| relaunch per spec file, shared dirs (C) | 17 passed / 7 failed | 204 s |
| relaunch + fresh dirs (E) | 22 passed / 2 failed (G2 only) | 254 s |
| relaunch + fresh dirs, repeat (E2) | 21 passed / 3 failed (G2 + `story-5-7`) | 280 s |
| pair `5-13` then `5-6`, stock | 1 / 1 | 48 s |
| same pair, relaunch only | 1 / 1, same 30 s message | 51 s |
| `story-5-7` alone, stock, 5 runs | 5 / 5 passed | 10-11 s each |

Killing the app releases port 4445 in about 1 s. Nightly CI on `574c869`: stock 16 / 8 in 2m03s. The measured leak has three layers: frontend module and `<KeepAlive>` state, Rust in-process state, and on-disk state in the shared dirs. Relaunch alone clears the first two; only fresh dirs clear the third.

Prototype core of the launcher hook (the rest is the pid filter and port wait):

```js
onWorkerEnd: async () => {
  killAppOnPort(4445)            // only a pid whose command starts with APP_BIN
  await portClosed(4445, 15_000) // throw on timeout
  process.env.AURATRANSLATE_E2E_DATA_DIR = mkdtempSync(join(tmpdir(), 'auratranslate-e2e-'))
  process.env.AURATRANSLATE_E2E_LIBRARY_ROOT = mkdtempSync(join(tmpdir(), 'auratranslate-e2e-library-'))
  // record both for onComplete's guards
},
```

## Verification

**Commands:**
- `npm run test:e2e -- --spec e2e/specs/story-5-13-reading-marks.e2e.mjs --spec e2e/specs/story-5-6-library-grid.e2e.mjs` -- expected: 2 spec files passed. Red before the fix.
- `npm run test:e2e` -- expected: 22 of 24 spec files passed, failures only `story-5-4-lifecycle` and `story-5-5-progress`. Run the stock config in the same session and record both walls; compare only same-session numbers.
- Removal control: empty the `onWorkerEnd` body, rerun the pair -- expected: `story-5-6` red; restore -- expected: green.
- Seeded controls for the mid-run guard and the held-port abort: untracked copies beside `wdio.conf.mjs` (so relative imports resolve), logs kept outside the repo, copies deleted afterwards -- expected: as the acceptance criteria state.
- `npm run check:gates` and `npm run check:lint` -- expected: pass, no gate added or removed.
- `git diff --stat -- src src-tauri` -- expected: empty.

**Ran 2026-09-14, this session, tree `574c869` plus this patch, macOS:**
- Pair `story-5-13-reading-marks` → `story-5-6-library-grid` alone: **2 passed / 2 total**, 10 s.
- Removal control: `onWorkerEnd` body emptied, same pair -- `story-5-6` red at 30 s with
  *"khởi động xong mà khối 'Tác phẩm' (Library) không có mặt sau 30 giây"* (the "khối Tác phẩm"
  message the AC names); `story-5-13` still green. Restored `onWorkerEnd` -- same pair back to
  2/2 green.
- Stock config (`git show HEAD:e2e/wdio.conf.mjs`, i.e. this patch's baseline `574c869`), full
  suite, same session: **16 passed / 8 failed, 149 s** (2m29s).
- This patch, full suite, three consecutive runs on the same still tree: **22 passed / 2
  failed** every time -- `story-5-4-lifecycle` (2 cases) and `story-5-5-progress` (1 case), the
  exact G2 set, nothing else. Walls 234 s / 239 s / 238 s (3m54s / 3m59s / 3m58s). Decision 4a's
  three-consecutive-22/2 bar is met.
- `npm run check:gates`: pass (three gate lists still match; no gate added or removed).
- `npm run check:lint`: pass, zero findings on `src e2e tests`.
- `npm run check:debt-owner`: pass, 0/498 open items missing an owner (the G1 closure and G2
  note parse as intended -- G1 counts as closed, G2 stays open with its existing owner).
- `sh .githooks/pre-push`: all fourteen steps (eleven gates, `test`, `build`, `cargo test`)
  green in 177 s -- confirms the `src`/`src-tauri` Never clause held all the way through
  (nothing in those trees needed touching to make the above pass).
- `git diff --stat -- src src-tauri package.json .githooks .github`: empty.

**Not run:** a tenth/eleventh consecutive full run, or a multi-day nightly-CI streak (Decision
4a's bar is three consecutive local runs, which is met; the nightly `e2e (macos-26)` schedule
has not yet had a chance to run against this patch as of this entry -- read it with
`gh run list --workflow=CI` rather than assuming it is already green).

**Independent re-verification by the orchestrator, 2026-09-14, same patched tree, logs kept.** The implementer's logs above were deleted after transcription and it did not run the fourth acceptance criterion, so every criterion was re-run:
- Seeded fourth criterion (a copy of `wdio.conf.mjs` with only the two `process.env` writes inside `onWorkerEnd` removed; `editor-typing-flush` + `grid-empty-cell`): 2 of 2 spec files passed, exit 1, `Error in onCompleteHook: … KHÔNG thấy global.db trong …/auratranslate-e2e-hyxeeQ` (the pair allocated after the first spec file); the trailing pair was deleted unguarded.
- Removal control (copy with `onWorkerEnd` unregistered), pair `5-13` then `5-6`: 1 passed / 1 failed, `story-5-6` with the "khối Tác phẩm" 30-second message.
- Three consecutive full runs on the unmodified patch: 22 passed / 2 failed each, failing cases exactly `story-5-4-lifecycle` (2) and `story-5-5-progress` (1); walls 238 s / 242 s / 253 s.
- Pair `5-13` then `5-6` on the patch: 2 of 2 passed, exit 0 (so the per-pair positive guard also held on real pairs).
- `git diff --stat 574c869 -- src src-tauri package.json .githooks .github`: empty. No e2e temp dir or app process left behind after the runs.

The two blocks above verified the round-0 implementation, reverted in review loop 1. They no longer describe the code.

**Review loop 1, 2026-09-14: the re-derived implementation, verified by the orchestrator on a still tree, logs kept outside the repo.**
- Seeded mid-run control (copy with the env write skipped on the 2nd and 3rd `onWorkerEnd` call; `editor-typing-flush`, `grid-empty-cell`, `shortcuts-capture-mouse`, `shortcuts-focus`, `story-5-4-lifecycle`): 4 passed / 1 failed, `Error in onCompleteHook: … 2 cặp thư mục thất bại`, naming exactly the two unwritten dirs; no e2e temp dir or app process left.
- Seeded held-port control (copy with the kill skipped, the port wait kept; `editor-typing-flush`, `grid-empty-cell`): `SevereServiceError: Cổng 4445 vẫn còn bị giữ sau 15 giây` raised in `onWorkerEnd`; the run ended at 1 of 2 spec files (50% completed); nothing left behind.
- Removal control (hook unregistered), pair `5-13` then `5-6`: 1 passed / 1 failed, `story-5-6` with the "khối Tác phẩm" 30-second message.
- Pair `5-13` then `5-6` on the implementation: 2 passed, exit 0.
- Three consecutive full runs: 22 passed / 2 failed each, 3 failing cases each (`story-5-4-lifecycle` 2, `story-5-5-progress` 1), no `onComplete` error; walls 243 s / 239 s / 240 s. The implementer's own full run just before gave the same 22 / 2 in 3m55s (log copied).
- `sh .githooks/pre-push`: green in 181 s on that tree. Seeded copies deleted afterwards; no untracked file remains.

**Review loop 1 patches (triage rows 21, 22, 26, 30-32), re-verified 2026-09-14 by the orchestrator, logs kept outside the repo.**
- A first re-run is void: every run, stock config included, exited after about 66 s with no `Spec Files` line and `SevereServiceError: Failed to start embedded WebDriver … did not become ready on port 4445` in `onPrepare`. The debug binary had been rebuilt without the `wdio` feature by the `cargo test` inside the preceding `pre-push` (binary mtime inside that window; 0 `wdio` strings in it). `npm run test:e2e` builds with the feature first; direct `npx wdio run` does not. Any measurement chain must build with `--features wdio` first and run `pre-push` last.
- After `cargo build --locked --features wdio` (37 `wdio` strings in the binary), on the patched tree:
- Seeded mid-run control: 4 passed / 1 failed (`story-5-4-lifecycle`), `2 cặp thư mục thất bại`, naming exactly the two unwritten dirs; nothing left behind.
- Seeded held-port control: `SevereServiceError: Cổng 4445 vẫn còn bị giữ sau 15 giây`, run ended at 1 of 2 spec files; nothing left behind.
- Removal control: `story-5-6` red with the "khối Tác phẩm" 30-second message.
- Pair `5-13` then `5-6`: 2 passed, exit 0.
- Three consecutive full runs: 22 passed / 2 failed each, the same 3 G2 cases each, no `onComplete` error; walls 246 s / 250 s / 249 s.
- `sh .githooks/pre-push` last: green in 194 s. No e2e temp dir, app process or untracked file left.
