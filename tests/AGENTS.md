<!-- bmad:context -->
<!-- Condensed 2026-09-23 (Ice): rules only; history in `_bmad-output/implementation-artifacts/agent-rules-evidence.md`. Managed by bmad-project-context; edits inside this block are replaced on refresh. -->

## tests/frontend/ — vitest

Behaviour of pure modules, DOM-touching code and `.vue`. Four acceptance paths, no overlap: static gates `scripts/check-*.mjs` · `src-tauri/tests/**` · vitest · `e2e/**`. Before writing a check, confirm the claim has no owner on another path.

## Conventions

- `happy-dom` is not WebKit: geometry, layout and real-engine claims belong to a probe or e2e.
- Tests live in `tests/frontend/**`, never co-located in `src/**` (gates count `src/**`).
- `tsconfig.json` includes the test tree.
- Every `happy-dom` patch lives in `tests/frontend/support/setup.ts`, one line each saying what it lacks.
- No `vi.useFakeTimers()` when the function already takes the timestamp as a parameter.
- `fileParallelism: false` is deliberate. A red file you did not touch ⇒ re-run it alone before blaming the diff.

## Known pitfalls

- 🔴 Never add `?.` to production code to clear a red test. Mock gaps are patched in `setup.ts`; product defects in `src/`.

<!-- /bmad:context -->
