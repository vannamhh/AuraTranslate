<!-- bmad:context -->
<!-- Condensed 2026-09-23 (Ice): rules only; history in `_bmad-output/implementation-artifacts/agent-rules-evidence.md`. Managed by bmad-project-context; edits inside this block are replaced on refresh. -->

## scripts/ — the rules of a GATE

`check:*` gates enforce claims across the whole tree. Adding a gate means editing three lists (`package.json` · `.github/workflows/ci.yml` · `.githooks/pre-push`); `check:gates` guards them.

`test-story.mjs` is not a gate: it only selects tests for the dev loop, stays out of the three lists, and its exit code is not a quality signal.

## Conventions

- The exit code is the verdict; no gate logs and carries on.
- 🔴 An infrastructure failure is not a red check: unreadable file ⇒ `abort()` with *"this is an infrastructure failure, not a pass"*.
- 🔴 A verdict never reads its parameters from what it checks: floors, role lists and exclusion lists are frozen in the script.
- Population floor at ~80–85% of the real count; an empty tree is not a clean tree.
- Plain Node, no bash (Windows runs `npm run` through `cmd.exe`). No npm dependency; parsers are strict hand-written subsets and unknown syntax ⇒ FAIL.
- A new gate carries a self-check proving it can go red and does not go red wrongly — and a case proving the real assert goes through the same filter as the self-check.
- A source-scanning gate anchors on something every spelling shares (`run_import(` misses `run_import_with_order(`). Seed the real violation; don't derive positive cases from the gate's own behaviour.

## Known pitfalls

- ⚠️ Only 4 gates have a self-check (`check-gates` C · `check-layout` D · `check-panel-refs` C · `check-debt-owner` B); a green run from the others is not a guarantee.
- ⚠️ `check-panel-refs.mjs:78` aborts with the wrong shape, and `check-scope`, `check-scope-bundled`, `check-dict-manifest` exit 1 bare — an infrastructure failure there reads like a red check.
- ⚠️ `test:e2e` is guarded in only two of the three lists; a green `check:gates` does not mean the three lists agree for it.

<!-- /bmad:context -->
