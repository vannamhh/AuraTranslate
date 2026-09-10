<!-- bmad:context -->
<!-- Verified 2026-09-10 against 39ae75d. Managed by bmad-project-context; edits inside this block are replaced on refresh. -->

## tests/frontend/ — vitest

Role: behaviour of pure modules, DOM-touching code, and `.vue`. Four acceptance paths that do not overlap — static gates `scripts/check-*.mjs` (declarative claims across the whole tree) · `src-tauri/tests/**` (contracts, boundaries, config invariants) · vitest · `e2e/**` (real WKWebView/WebView2). Before writing a new check, ask: does this claim already have an owner on another path? Two paths guarding one claim is two sources of truth.

## Conventions that differ from defaults

- `happy-dom` is NOT WebKit. Every claim about geometry, layout, or a real engine belongs to a probe or to e2e — not to vitest.
- Tests live in `tests/frontend/**`, NOT co-located in `src/**`: four gates count the `src/**` population and a test file dropped in there inflates the denominator, plus two collisions (`check-i18n` Check A goes red on Vietnamese text, `check-tokens` Check B goes red on a hard-coded colour).
- `tsconfig.json` must `include` the test tree — an unchecked test tree is a test tree that will rot: it keeps running green while the types of what it checks change underneath it.
- Every `happy-dom` patch lives in `tests/frontend/support/setup.ts`, each entry carrying one line saying what it lacks, AND SOMEONE READS IT. That list is a measurable debt — 3 entries today, two of which no longer have a reader.
- No `vi.useFakeTimers()` when the function already takes the timestamp as a parameter: wrapping a fake clock trades a guarantee for a habit.
- `vitest.config.ts` sets `fileParallelism: false` deliberately, at a cost of 27 s → 99 s. It was measured: worker contention gave 5 red cases that were not about the code — baseline 822/822 green, 835 green + 5 red with the story, `--no-file-parallelism` 840/840 green, and the four red files run alone 42/42 green while none of them even loaded what the story touched. Before blaming a diff for a red here, re-run the file alone.

## Known pitfalls

- 🔴 The wrong path is very cheap and has to be blocked by hand: adding a `?.` to PRODUCTION CODE to clear a red. That is a branch the types say never runs — permanently dead code in the product, serving a mock. Gaps in the mock get patched in `setup.ts`; product defects get patched in `src/`.

<!-- /bmad:context -->
