<!-- bmad:context -->
<!-- Condensed 2026-09-23 (Ice): rules only; history in `_bmad-output/implementation-artifacts/agent-rules-evidence.md`. Managed by bmad-project-context; edits inside this block are replaced on refresh. -->

## src/ — Vue 3 + TypeScript

The frontend only renders and holds UI state. No business rule in TypeScript (AD-1); the single exception is the text being typed in the Editor.

## Conventions

- `invoke()` sends parameters in camelCase (`sourceLang`), but returned struct fields stay snake_case (`work_id`).
- IPC adapters in `src/config/*.ts` never throw: one `invoke`, one `try/catch`, returning `{ <value> | null, error: IpcError | null }`. The UI shows errors through `tError()`.
- Type-check data crossing the wire at runtime.
- `verbatimModuleSyntax` ⇒ explicit `import type`. vitest `globals: false` ⇒ every test imports `{ describe, it, expect } from 'vitest'`.
- `@click` in a `.vue` is exactly one `dispatch('<id>')` call (`check:commands` Check A). Shortcuts and Auto-Lookup dispatch the same way.
- Command ids use the dotted i18n-key grammar (`review.accept_change`).
- A function run from a keyboard chord never throws: it logs the cause and returns `false`.
- An `⌥` chord compares `event.code`, never `event.key`.
- ⚠️ `check:commands` does not see `@change`, and does not judge whether a chord is safe. A bare `Space` global chord is unsafe (`isTypingZone` skips `<button>`).
- Colour and font size come only from tokens; no shadows, gradients or floating layers. An intermediate `opacity` needs a named exemption.
- A directory carrying a concept has a `README.md` (missing in `src/config/` and `src/selftest/`).

## Known pitfalls

- 🔴 A `Ref` does not auto-unwrap inside `<script>`: `if (someRef)` is always true and `vue-tsc` is silent (`check:lint` exists for this).
- 🔴 Five files must load under plain Node because gates `import()` them: `src/i18n/resolve.ts`, `src/commands/{index,registry,focus}.ts`, `src/layout/writeSchedule.ts`. No value imports from `vue`/`dockview`; no `enum`, `namespace` or parameter properties. `src/layout/dockController.ts` exists for this.
- 🔴 Startup order in `src/main.ts`: `applyTheme()` and `installCommands()` before `mount()`; `loadFonts()` starts before `await loadBootstrapConfig()`.
- Register commands in `main.ts`, not `App.vue` (HMR re-registers and throws).
- dockview's `onDidLayoutChange` fires continuously during a drag: every write cadence goes through `src/layout/writeSchedule.ts`.
- Three write-cadence pairs, only the Editor's carries AD-35: layout `IDLE_MS 500`/`HARD_CAP_MS 5000` (`layout/writeSchedule.ts`); Editor `EDITOR_IDLE_MS 2000`/`EDITOR_HARD_CAP_MS 5000` (`panels/editorFlush.ts`); position `POSITION_IDLE_MS 500`/`POSITION_HARD_CAP_MS 5000` (`panels/positionFlush.ts`). Don't merge them.
- Write-cadence functions never read `Date.now()`; timestamps arrive as parameters.
- No second OS window (AD-24): `addPopoutGroup` is banned. `check:layout` Check C is an allow-list of `window`/`document` members; adding one is a written decision.
- External content is never rendered as HTML: no `v-html` (AD-16).
- `src/selftest/**` stays out of the release build; no static import from it.

<!-- /bmad:context -->
