<!-- bmad:context -->
<!-- Verified 2026-09-10 against 39ae75d. Managed by bmad-project-context; edits inside this block are replaced on refresh. -->

## src/ — Vue 3 + TypeScript

The frontend only renders and holds UI state. No business rule lives in TypeScript (AD-1); the single explicit exception is the text currently being typed in the Editor.

## Conventions that differ from defaults

- `invoke()` sends parameters in **camelCase** even though the Rust function takes `snake_case` ⇒ write `sourceLang`. BUT the fields of the RETURNED struct stay `snake_case` (`meta_schema_version`, `work_id`). The two directions differ — this is the easiest thing on the wire to get wrong.
- IPC adapters in `src/config/*.ts` NEVER throw: one `invoke`, one `try/catch`, returning the three-state shape `{ <value> | null, error: IpcError | null }`. The UI layer displays errors through `tError()`, not through `try/catch`. (`shortcutsState.ts` is not an adapter — it is Vue state calling down into `bootstrap.ts`.)
- Always type-check data crossing the wire AT RUNTIME. `IpcError` on the TS side is a claim about data that already crossed IPC, not a compiler guarantee.
- `verbatimModuleSyntax` is on ⇒ `import type` must be explicit. vitest sets `globals: false` ⇒ every test file imports its own `{ describe, it, expect } from 'vitest'`.
- `@click` in a `.vue` must be EXACTLY ONE `dispatch('<id>')` call — no other function, no inline code (`check:commands` Check A). Shortcuts and Auto-Lookup emit the same `dispatch(...)`: a direct call builds a second path that Check A cannot see.
- Command ids use the same dotted grammar as i18n keys (`review.accept_change`) — a bare id will be registered twice by two phases months apart and silently overwrite.
- A function run from a keyboard chord NEVER throws — it logs a diagnostic naming the cause and returns `false`. Don't "fix" it by switching mode: that is guessing the user's intent.
- An `⌥` chord must compare `event.code`, never `event.key` — `⌥W` produces `∑`, so `event.key === 'w'` is never true (`keys.ts` already names `Alt+M` → `µ`). If a counter-check for this comes back GREEN, suspect the mock before the test: raise it rather than changing the production predicate to match `happy-dom`.
- ⚠️ `check:commands` has three MEASURED blind spots — don't read its green as coverage: Check A only watches `@click`, so an in-place edit committed through `@change` needs no registration; and seeding `keys: ['Space']` or `keys: ['Alt+w']` both pass, because the gate catches chord COLLISIONS, not whether a chord is safe. A bare `Space` as a global chord is unsafe: `isTypingZone` covers only `INPUT`/`TEXTAREA`/`SELECT`/`contenteditable`, so it would `preventDefault()` every `<button>` in the app.
- Colour AND font size come only from tokens; no drop shadows, no gradients, no floating layers. An intermediate `opacity` needs a NAMED exemption.
- A directory carrying a concept has a `README.md` — missing today in `src/config/` and `src/selftest/`.

## Known pitfalls

- 🔴 A `Ref` does NOT auto-unwrap inside a `<script>` block, only in `template`. `if (someRef)` runs on the **object** and is therefore always true, and because it is valid TypeScript, `vue-tsc` stays silent. This bug passed NINE gates out of nine and is the reason the tenth (`check:lint`, type-aware) exists.
- 🔴 Five files must load under **plain Node**, because gates `import()` them to run BEHAVIOURAL checks against production code itself: `src/i18n/resolve.ts` (this file imports nothing at all), `src/commands/{index,registry,focus}.ts`, `src/layout/writeSchedule.ts`. No VALUE imports from `vue`/`dockview`; no `enum`, `namespace`, or parameter properties (`constructor(private x)`) — all three emit code, so Node rejects them. One offending line kills three checks at once. `src/layout/dockController.ts` exists for exactly this reason: `main.ts` injects functions into it rather than importing back.
- 🔴 The startup order in `src/main.ts` is mandatory, all three clauses: `applyTheme()` before `mount()` (otherwise every `var(--color-…)` is empty on the first render ⇒ a white flash — and on a packaged build that flash is SHORTER than on a dev machine, so the bug only shows up on someone else's computer); `installCommands()` before `mount()` (`dispatch` throws on an unregistered id); `loadFonts()` starts before `await loadBootstrapConfig()`.
- Register commands in `main.ts`, NOT in `App.vue` — an HMR round rebuilding the component calls `installCommands()` a second time and `register()` throws on the duplicate id.
- dockview's `onDidLayoutChange` fires CONTINUOUSLY while a sash is dragged: writing one `putConfig` per fire turns a 3-second drag into hundreds of serialised jobs through `store::Writer`. No gate goes red for that — it surfaced in Epic 2 as *"typing stutters"*. Every write cadence goes through `src/layout/writeSchedule.ts`.
- Three write-cadence constant pairs (🔵 FIXED 2026-08-29, Story 5.7 — previously "two pairs", now a third), only ONE of which carries the AD-35 guarantee: layout uses `IDLE_MS 500`/`HARD_CAP_MS 5000` in `layout/writeSchedule.ts` (no guarantee); the Editor uses `EDITOR_IDLE_MS 2000`/`EDITOR_HARD_CAP_MS 5000` in `panels/editorFlush.ts` (guaranteed); Chapter working position uses `POSITION_IDLE_MS 500`/`POSITION_HARD_CAP_MS 5000` in `panels/positionFlush.ts` (no — losing a position write loses ONE REMINDER, not work). Same shape, different guarantees — do not merge the three pairs.
- Write-cadence functions never read `Date.now()` themselves: every timestamp arrives as a parameter, so checks are deterministic and instant instead of having to `sleep` for real.
- No second OS window (AD-24): `addPopoutGroup` is dockview's only path that calls `window.open` — banned. `check:layout` Check C is an ALLOW-LIST of every `window`/`document` member `src/**` touches; adding a name is a decision that has to be written down.
- External content is NEVER rendered as HTML: no `v-html`, no equivalent (AD-16). Rust parses it into a structured data model; Vue renders from that model.
- `src/selftest/**` deliberately stays out of the release build (`#[cfg(debug_assertions)]` on the Rust side + dynamic `import()` on the frontend) — no production code may import statically from it.

<!-- /bmad:context -->
