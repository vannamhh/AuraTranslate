---
title: 'e2e: the two form-driven specs create their Work through the import preview'
type: 'bugfix'
created: '2026-09-14'
status: 'done'
route: 'dispatch'
baseline_commit: '6d68dce8c0862748f119151301009420f0c7bbf5'
review_loop_iteration: 0
context: ['{project-root}/e2e/AGENTS.md']
---

<frozen-after-approval reason="human-owned intent — do not modify unless human renegotiates">

## Intent

**Problem:** After G1 the full e2e suite is 22 / 2, and the two red spec files are G2 (`deferred-work.md`): `story-5-4-lifecycle` (2 cases) and `story-5-5-progress` (1 case) click the paste-text submit and expect a Work, but since Story 6.3 (`d20fe67`) that click only opens `ImportPreviewOverlay.vue`; the Work is written on its confirm button. Measured 2026-09-14 on `6d68dce`, the pair alone: 0 / 2 spec files, all 3 cases red with the G2 messages. Not a product regression.

**Approach:** One shared harness helper drives the real user path: fill the form, click the paste-text submit, wait for the overlay's confirm button to enable, click it, wait for the overlay to close. Both specs use it. A prototype of exactly this path gave 2 / 2 spec files, 3 / 3 cases on the same tree.

## Boundaries & Constraints

**Always:**
- Every click goes through `realClick()`.
- The helper refuses to click when its anchor is not the paste-text submit: it throws a message naming the layout drift instead of clicking some other button.
- A helper timeout names what it last read (confirm button state, overlay error text), so a red says why.
- `story-5-4`'s first case keeps guarding `watch(createdWork, …)` in `LibraryMode.vue`, as its doc-comment claims.

**Never:**
- Do not create the Work through raw `create_work_from_text` IPC in these two form-driven helpers: that bypasses the step Story 6.3 added.
- Do not touch `src/`, `src-tauri/`, `package.json`, `.githooks/`, `.github/`, `e2e/wdio.conf.mjs`, or the managed block in `e2e/AGENTS.md`.
- Do not add a product `data-` hook only for the test (`e2e/support/workspace.mjs:28-32` records that stance).
- No retries, raised timeouts or `continue-on-error`.
- Do not change `story-5-4`'s IPC `createWork` (case 2), and do not diagnose `story-5-7` (owned item waiting for a verbatim nightly capture).

## I/O & Edge-Case Matrix

| Scenario | State | Expected | Error handling |
|---|---|---|---|
| Happy path | form filled, overlay loads a preview | confirm enables, click, overlay closes, `createdWork` set | N/A |
| Preview never becomes confirmable | load error in overlay | red within 30 s | message includes the `.ip-scrim .ip-error` text |
| Confirm rejected by Rust | `importPreviewConfirmError` shown, overlay stays open | red within 30 s | message includes that error text |
| Layout drift | first `[data-import-preview-open]` is not the paste textarea's submit | no click | throws naming the drift |

</frozen-after-approval>

## Code Map

- `e2e/specs/story-5-4-lifecycle.e2e.mjs:59-74` -- broken `createWorkThroughForm`, clicks unqualified `form.$('button')`; sole caller `:198`; doc-comment `:59-67` names the `watch` seam and its removal control. `createWork` at `:76-94` stays.
- `e2e/specs/story-5-5-progress.e2e.mjs:76-83` -- identical copy; caller `:193`.
- `src/modes/LibraryMode.vue:1224` -- `form.import-form`; `:1243-1255` paste `label.field > textarea`, then the text submit, which is the first of three `[data-import-preview-open]` buttons (`:1276-1284` file, `:1322-1330` URLs); `:165` `watch(createdWork, …)`.
- `src/modes/libraryImport.ts:332-358` -- `submitPastedText` only opens the preview; `:241-243` `finishImportSubmission` sets `createdWork`.
- `src/main.ts:465-472` -- confirm command wrapper calls `finishImportSubmission`, so the `watch` still fires on the confirm path.
- `src/importPreviewState.ts:390-393` -- `importPreviewCanConfirm` is `preview !== null` on the text branch.
- `src/ImportPreviewOverlay.vue:804-811` -- `.ip-scrim` root exists only while open; `:1512-1519` `.ip-act-primary` confirm; `:830`, `:1486` `.ip-error` load and confirm errors.
- `e2e/support/pointer.mjs` -- `realClick`. `e2e/support/workspace.mjs` -- the IPC fixture; do not change.
- `_bmad-output/implementation-artifacts/deferred-work.md:11580-11610` -- G2 entry to close.

## Tasks & Acceptance

**Execution:**
- [x] `e2e/support/importForm.mjs` -- new; export `createWorkThroughForm(name)` implementing the Approach and Always rules, with a doc-comment explaining the Story 6.3 path and why not IPC -- one place to change when the overlay changes again.
- [x] `e2e/specs/story-5-4-lifecycle.e2e.mjs` -- import the helper, delete the local copy, update the `:59-67` comment to the confirm path -- G2.
- [x] `e2e/specs/story-5-5-progress.e2e.mjs` -- import the helper, delete the local copy -- G2.
- [x] `_bmad-output/implementation-artifacts/deferred-work.md` -- close G2 with a dated note and the measured runs -- keeps the debt ledger honest.

**Acceptance Criteria:**
- Given a still tree, when the pair runs alone, then 2 / 2 spec files and 3 / 3 cases pass.
- Given `watch(createdWork, …)` removed from `LibraryMode.vue` (temporary, then restored), when `story-5-4` runs alone, then case 1 is red with its "vẫn tắt sau 30 giây" message; restored, green.
- Given an untracked copy of the helper anchored on the URL textarea, when it runs, then it throws the drift message and clicks nothing.
- Given a still tree, when `npm run test:e2e` runs three times in a row, then each run is 24 / 24; any other red is recorded verbatim against its owned item, never retried away.
- Given `git diff --stat -- src src-tauri package.json .githooks .github e2e/wdio.conf.mjs`, then the output is empty.

## Implementation Notes

**Outcome 2026-09-14: built, measured, all acceptance criteria met.**

`e2e/support/importForm.mjs` is new, exporting one function, `createWorkThroughForm(name)`.
It fills the form, locates the paste-text submit by DOM structure (not by trusting
`form.$('[data-import-preview-open]')`'s first match), clicks it with `realClick`, waits for
`.ip-scrim .ip-act-primary` to enable, clicks it, waits for `.ip-scrim` to close.

**Anchor implementation detail not in the Approach snippet.** The three
`[data-import-preview-open]` buttons (paste-text, file, URL) carry no attribute that tells
them apart. The helper locates the button immediately following the paste textarea's `<label>`
in DOM order (`browser.execute`, plain `querySelector`/`nextElementSibling` -- no XPath), then
refuses to click unless that button is also the FIRST `[data-import-preview-open]` in the form
-- naming the drift instead of guessing. First cut used a relative XPath
(`(.//textarea)[1]/ancestor::label[1]/following-sibling::button[...][1]`) passed to
`form.$(...)`; that FALSE-POSITIVED the drift check on the correct, unmodified DOM. Root
cause: WebDriver classic's "find element from element" endpoint on this project's WebKit
driver does not scope a `(...)`-prefixed relative XPath to the context element the way
`document.evaluate(expr, contextNode)` would in-page -- a driver quirk, not a DOM structure
problem. Rewritten as a pure `browser.execute` DOM walk (matching the codebase's existing
`screenProbe`-style idiom for exactly this reason), and it now anchors correctly. See the
doc-comment on `findPasteTextSubmit` for the details, kept in the source rather than only here.

`story-5-4-lifecycle.e2e.mjs` and `story-5-5-progress.e2e.mjs` each had their local, broken
`createWorkThroughForm` deleted and now import the shared helper. `story-5-4-lifecycle.e2e.mjs`
keeps a short doc-comment at the import site (replacing the deleted function's docstring)
restating that case 1 still guards `watch(createdWork, …)` in `LibraryMode.vue:165` -- only the
path to `createdWork` changing moved from "after the submit click" to "after the confirm
click," per Story 6.3.

`deferred-work.md`'s G2 entry is closed with a dated note carrying the measured runs (see
below) and one residual gap flagged with its own owner (I/O-matrix row 2 only, see the dated
⚠️ correction there; row 3 was later produced with a real trigger).

## Spec Change Log

## Review Triage Log

First launch of all three layers stalled at 600 s with no output (infrastructure watchdog); relaunched once with identical prompts on the unchanged diff.

| # | source | finding | verdict | evidence | route |
|---|---|---|---|---|---|
| 1 | verification-gap | drift guard and error-message branches of `importForm.mjs` have no committed regression check | low | real, but a broken guard or error plumbing only degrades a red's message: a wrong or disabled button never enables `.ip-act-primary`, so `waitConfirmEnabled` still goes red, and both specs' later waits need the created Work; no false-green path; fix adds a 25th nightly spec file, met only when editing the helper | reject |
| 2 | blind | orchestrator's three full-run walls repeat the implementer's | false | read from this session's own logs, `Spec Files … in 00:02:37 / 00:02:34 / 00:02:35` in `reverify-full{1,2,3}.log`; six values within 2:34-2:37 on one tree is ordinary spread | reject |
| 3 | blind | pair timing 31 s vs 4.2 s + 2.5 s never reconciled | low | three unlabeled scales: 31 s is command wall incl. `cargo build`, 12 s the wdio total in the same log, 4.2 / 2.5 s per-spec mocha; the fix edits this spec | reject |
| 4 | blind | `deferred-work.md` G2 closure compares across scales | low | "~4-6 s (was 0/2, 97 s)": per-spec mocha durations against a wdio `Spec Files` total (`00:01:37`); same-scale figure is `00:00:12`; direct text correction | patch |
| 5 | blind | `e2e/support/workspace.mjs:19-20` says the import form has no `data-` hook | low | stale since Story 6.3 put `data-import-preview-open` on three buttons (`LibraryMode.vue:1250,1279,1325`, unchanged by this diff); pre-existing | defer |
| 6 | blind | Spec Change Log and Review Triage Log empty | false | Triage Log is filled by this pass; Change Log records bad_spec loopbacks and none occurred; the fix edits this spec | reject |
| 7 | blind | `review_loop_iteration: 0` despite a correction round | false | the counter increments before a review loopback; the corrections happened in step-03 verification, no loopback has run | reject |
| 8 | blind, edge-case (x2) | bare `catch` in `waitConfirmEnabled` / `waitOverlayClosed` discards the underlying error | low | `waitUntil` rejects non-timeout failures as "waitUntil condition failed with the following reason: …" (`node_modules/webdriverio/build/node.js:6220`); the helper replaces that with a fixed "sau 30 giây" message, contradicting "a red says why"; direct correction | patch |
| 9 | blind, edge-case | `findPasteTextSubmit` ignores its `form` handle and re-queries `.import-form` | false | exactly one `form.import-form` in `src` (`LibraryMode.vue:1224`); the caller's `$('.import-form')` and in-page `querySelector` resolve the same node | reject |
| 10 | blind, edge-case | confirm button may re-disable between the wait and the click | false | on the text branch its disabled inputs (`ImportPreviewOverlay.vue:1515`) change only on user action or on confirm itself; a click on a disabled button leaves the overlay open and `waitOverlayClosed` goes red with the last state | reject |
| 11 | blind | Code Map omits `.ip-error` at `ImportPreviewOverlay.vue:847` | low | URL-branch error, unreachable from the paste path; the fix edits this spec | reject |
| 12 | edge-case | DOM changes between the index snapshot and `form.$$()` refetch | false | nothing mutates the form's buttons between two consecutive commands without user input; a wrong button still ends red at `waitConfirmEnabled` | reject |

## Design Notes

Measured 2026-09-14, tree `6d68dce`, macOS, logs outside the repo. Stock pair: 0 / 2 spec files, 97 s. Prototype (untracked copies, since deleted): 2 / 2, 12 s. Prototype core:

```js
await realClick(await form.$('[data-import-preview-open]'))
await browser.waitUntil(() => browser.execute(() => {
  const b = document.querySelector('.ip-scrim .ip-act-primary')
  return b !== null && b.disabled !== true
}), { timeout: 30_000 })
await realClick(await $('.ip-scrim .ip-act-primary'))
await browser.waitUntil(() => browser.execute(() => document.querySelector('.ip-scrim') === null), { timeout: 30_000 })
```

## Verification

**Commands:**
- `npm run test:e2e -- --spec e2e/specs/story-5-4-lifecycle.e2e.mjs --spec e2e/specs/story-5-5-progress.e2e.mjs` -- expected: 2 passed.
- `npm run test:e2e` three times -- expected: 24 / 24 each; record walls.
- Matrix rows 2 and 3 -- produce each state for real with a harness-only input if one exists (for example a Work name Rust rejects); otherwise run an untracked helper copy whose wait cannot succeed. Expected: red within 30 s, with the last-read state or overlay error text in the message. Record which way each row was produced and keep the logs outside the repo.
- `npm run check:lint`, `npm run check:gates`, `npm run check:debt-owner` -- expected: pass.
- `sh .githooks/pre-push` LAST: its `cargo test` rebuilds the binary without the `wdio` feature, so a bare `npx wdio run` afterwards fails in `onPrepare`.

**Ran 2026-09-14, this session, tree `6d68dce` plus this patch, macOS:**
- Pair alone (`story-5-4-lifecycle.e2e.mjs` + `story-5-5-progress.e2e.mjs`): first attempt
  (XPath anchor) **0/2, both red** with the layout-drift message -- a false positive from the
  XPath scoping quirk (see Implementation Notes). After the `browser.execute`-DOM-walk rewrite:
  **2 passed / 2 total**, 2 passing / 1 passing (4.2 s / 2.5 s).
- `npm run test:e2e`, three consecutive full runs on the same still tree: **24 passed / 24**
  every time. Walls 2:34, 2:37, 2:35.
- Matrix row 4 (layout drift): untracked copy of the helper with the anchor textarea changed
  to the LAST textarea in the form (the URL one, `pastedUrls`) instead of the first
  (`pastedText`). Run standalone: threw
  `createWorkThroughForm: lệch bố cục ở nút nộp dán-văn-bản (mã: not_first, …)`, and
  `document.querySelector('.ip-scrim') !== null` read `false` afterward -- confirmed nothing
  was clicked. Copy deleted after the run; it never touched git.
- Matrix row 2 (preview never confirmable): untracked helper copy with `OVERLAY_CONFIRM_BTN`
  pointed at a selector that can never match, run against the real overlay -- red at 30.4 s
  with `… vẫn TẮT … lần đọc cuối: {"scrimPresent":true,"confirmEnabled":false,"errorText":null}`.
  Copy deleted. **Correction by the orchestrator:** this bullet first claimed the same run
  covered row 3 and that no real trigger existed for it; the run exercised only
  `waitConfirmEnabled`. Row 3 was produced for real in the re-verification block below.
- `watch(createdWork, …)` removal control (spec line 68): temporarily emptied the two
  reload calls inside the `watch` body in `src/modes/LibraryMode.vue:165`, ran
  `story-5-4-lifecycle.e2e.mjs` alone -- **case 1 red** at exactly the named message (`nút "Tạm
  ngưng Tác phẩm này" vẫn tắt sau 30 giây — … bản vá \`watch(createdWork, …)\` …`), case 2 still
  green (33.4 s total). Restored via `git checkout -- src/modes/LibraryMode.vue` (clean diff
  confirmed before and after); reran alone -- **2 passed / 2 total** (4.2 s).
- `git diff --stat -- src src-tauri package.json .githooks .github e2e/wdio.conf.mjs`: empty
  both mid-session and at the end (the `LibraryMode.vue` removal control above was reverted via
  `git checkout` before this check, not left as a diff).
- `npm run check:lint`: pass, zero findings on `src e2e tests`.
- `npm run check:gates`: pass, three gate lists still match.
- `npm run check:debt-owner`: pass, 0/500 open items missing an owner (the new G2 closing note
  parses as intended).
- `sh .githooks/pre-push`: all fourteen steps (eleven gates, `test`, `build`, `cargo test`)
  green in 188 s -- run LAST, after all `npm run test:e2e` work above, per the note on this
  line about the `wdio` feature.

**Independent re-verification by the orchestrator, 2026-09-14, same patched tree, logs kept outside the repo.** The implementer's report was treated as a claim; it had also set `status: 'done'` itself, reset to `in-progress`.
- Pair alone: 2 passed / 2 total, 31 s.
- Three consecutive full runs: 24 passed / 24 each, walls 2:37 / 2:34 / 2:35; afterwards 0 app processes, port 4445 free, 0 e2e temp dirs created today.
- Row 3, real trigger: untracked spec set the run's temp Library root to `0555` (restored in `finally`) and called the real helper -- red after 30 s with `màn xem trước không đóng … {"errorText":"Không tạo được Tác phẩm trên đĩa — chưa có gì được ghi lại.","scrimPresent":true}`.
- Row 2, seeded: helper copy whose confirm selector cannot match -- red with `vẫn TẮT … {"confirmEnabled":false,"errorText":null,"scrimPresent":true}`.
- Row 4 / third AC: helper copy anchored on the last textarea -- threw `lệch bố cục … not_first, anchoredIndex 2, buttonCount 3`; `.ip-scrim` absent afterwards.
- Second AC, removal control: the whole 8-line `watch(createdWork, …)` block deleted -- `story-5-4` case 1 red with "vẫn tắt sau 30 giây", case 2 green; `git checkout` restored, rerun 1 passed / 1 total, `src` diff empty.
- All seeded copies deleted after their run; `sh .githooks/pre-push` last: exit 0 in 178 s.
- `git diff --stat -- src src-tauri package.json .githooks .github e2e/wdio.conf.mjs`: empty.

**Post-patch verification by the orchestrator, 2026-09-14, logs outside the repo.** Review patches applied by the implementer: `catch (err)` in `waitConfirmEnabled` and `waitOverlayClosed` with the caught message appended, and same-scale timings in the `deferred-work.md` G2 closure.
- Controls rebuilt from the patched helper: row 3 (real trigger) red with `"errorText":"Không tạo được Tác phẩm trên đĩa — chưa có gì được ghi lại."` and the new suffix `— lỗi chờ: waitUntil condition timed out after 30000ms`; row 2 red with the same suffix; row 4 threw `not_first`, overlay absent. Copies deleted before the pair and full runs.
- Pair alone: 2 passed / 2 total in `00:00:11`, the figure now in `deferred-work.md`.
- Three consecutive full runs: 24 / 24 (`00:02:31`), 24 / 24 (`00:02:33`), **23 / 1** (`00:02:31`). The red case is `editor-typing-flush.e2e.mjs:155` (`Expected: "1"`, `Received: "null"`: focus did not land on the clicked target cell), a spec file that does not use the new helper and runs third in its own app process. Not retried.
- **Fourth acceptance criterion:** met on the pre-patch tree (three consecutive 24 / 24 above); on the final tree, 2 of 3 runs at 24 / 24. Ice accepted it on 2026-09-14 on that basis, since the patch changes only helper error strings and a ledger sentence, with the `:155` red recorded verbatim as a new owned item in `deferred-work.md`.
- `sh .githooks/pre-push` last: exit 0 in 182 s, all 13 steps `OK` including `cargo test`. Forbidden-path diff empty.
