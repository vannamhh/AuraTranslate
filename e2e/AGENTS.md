<!-- bmad:context -->
<!-- Verified 2026-09-10 against 39ae75d. Managed by bmad-project-context; edits inside this block are replaced on refresh. -->

## e2e/ — WebdriverIO in a real webview

One role only: behaviour in a REAL WKWebView/WebView2. This suite runs NIGHTLY on macOS (`schedule` 18:00 UTC = 01:00 Ice time, plus `workflow_dispatch`) — NOT on `push`, not in `pre-push`, and the Windows/WebView2 half has never run. Manual: `npm run test:e2e`.

## Known pitfalls

- 🔴 The driver's `.click()` is banned, use `realClick()` from `e2e/support/pointer.mjs`. The driver fires `click` BEFORE `focusin` — the reverse of a real mouse — so it both gives RED for the wrong reason and GREEN on a broken product. Enforced by `no-restricted-syntax` in `eslint.config.js`.
- Each spec opens a real window (~1.5 min) and it writes to the runner's REAL `global.db` and Library root if the two redirect env vars fail to reach the child process. `wdio.conf.mjs` carries a POSITIVE self-check — `global.db` must sit inside the temp directory — that runs before anything is deleted. Do not remove that check.
- Never assert an ABSOLUTE number against a SHARED resource — that is a claim about what the other specs did. `story-5-3-rescan` asserted `Đã lập chỉ mục 1` and the first full-suite run gave `Đã lập chỉ mục 22`; run alone it was green, so it read as an ordinary pass from the day it was written. Assert the DIFFERENCE against the moment before the action. For the same reason, a spec's scratch directory goes in its own temp dir, never the shared system temp — one crashed run's leftovers kill the next.
- Activating a `<button>` from the keyboard does not work here: `focus()` in JS plus `browser.keys(['Enter'])` leaves the handler UNRUN, while `realClick` on the same button in the same session runs it. So the "by keyboard" half of an AC has no automated acceptance path — record the debt, don't round it up to passed.
- 🔴 A red nightly does NOT mean the product regressed. The first four nights (20–23 Aug) were red twice, and both reds died at the IPC BRIDGE — `core.invoke not available after 5s` ⇒ the fixture could not create a Work ⇒ 0 rows in the grid after 30 s — while the `check` job was green on both platforms all four nights. Read the error before changing a line of product code; and don't patch it with `continue-on-error` or a retry loop, both of which turn the job into something that can never go red. (Owner: Ice since 2026-08-24, `deferred-work.md`.)
- 🔴 ONE app process serves the WHOLE run — measured 2026-09-12 (one `pid` across two sessions), not one process per spec file. Module-level state (active mode, an in-flight reading run, Library's works/rescan/search singletons) carries FORWARD across spec files; only the five panel modules in `support/panelReset.mjs` are reset, and only for specs that go through `openWorkspaceWithWork()`. So a red spec here is often a claim about what the spec ALPHABETICALLY BEFORE it left behind, not about the product: measured 2026-09-12, 12 of the 15 failing cases were green when their spec ran alone. Run the spec alone before you read its failure as a defect. An attempted fix (reset every module from the `before` hook) made the suite WORSE — 16/8 to 12/12 — and was reverted; see the doc-comment in `wdio.conf.mjs` and the owned entry in `deferred-work.md` before trying that road again.

<!-- /bmad:context -->
