<!-- bmad:context -->
<!-- Condensed 2026-09-23 (Ice): rules only; history in `_bmad-output/implementation-artifacts/agent-rules-evidence.md`. Managed by bmad-project-context; edits inside this block are replaced on refresh. -->

## e2e/ — WebdriverIO in a real webview

Behaviour in a real WKWebView/WebView2. Runs nightly on macOS (`schedule` 18:00 UTC + `workflow_dispatch`), not on push or in `pre-push`; the Windows half has never run. Manual: `npm run test:e2e` (build with `--features wdio` first; `pre-push` rebuilds without it).

## Known pitfalls

- 🔴 The driver's `.click()` is banned (it fires `click` before `focusin`); use `realClick()` from `e2e/support/pointer.mjs`. Enforced by ESLint.
- Each spec opens a real window (~1.5 min). `wdio.conf.mjs` checks that `global.db` sits inside the temp dir before deleting anything — never remove that check.
- Never assert an absolute number against a shared resource; assert the difference from the moment before the action. Scratch dirs go in the spec's own temp dir.
- Keyboard activation of a `<button>` does not work here; the "by keyboard" half of an AC has no automated path — record the debt.
- 🔴 A red nightly is not proof of a product regression (the IPC bridge has failed before). Read the error first; never add `continue-on-error` or a retry loop.
- 🔴 Each spec FILE gets a fresh app process and fresh `$APPDATA`/Library root (`wdio.conf.mjs::onWorkerEnd`). State still carries between cases inside one file; `openWorkspaceWithWork()` resets only the five panels named in `support/panelReset.mjs`. A red case in an untouched file ⇒ read that file's case order first.

<!-- /bmad:context -->
