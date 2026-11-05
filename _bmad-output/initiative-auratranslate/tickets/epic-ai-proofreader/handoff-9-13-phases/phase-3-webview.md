# Phase 3 (Webview) handoff, story 9-13

State: `vue-tsc --noEmit` clean, eslint clean on touched files, vitest (promptSet, prompt, aiPrompt, dialogAccessible, aiTranslate; 402 tests) green. e2e spec edited but NOT run. Nothing committed.

## What changed
- `src/config/promptset.ts`: exported `PromptSetKind = 'translate' | 'proofread'`. Wrappers take `kind` required: `promptSetList(kind)`, `promptSetCreate(tier, kind, name, body)`, `promptSetRename(tier, kind, id, newName)`, `promptSetUpdateBody(tier, kind, id, body)`, `promptSetDelete(tier, kind, id)`, `promptSetExport(tier, kind, id)`, `promptSetExportMany([{tier, kind, id}])`. `PromptSetExportFile` has `kind` (checked in `isExportFile`); `PromptSetImportPreview` has `kind` (checked in its guard). `PromptSetWire` unchanged. `choice_get/set`, confirm/cancel/open-preview unchanged.
- `src/promptSetState.ts`: `const LIBRARY_KIND = 'translate'` passed at every call; `exportPromptSets` keeps its `{tier,id}` signature and maps `kind` in.
- Tests updated: `tests/frontend/promptSetState`, `promptLibraryOverlayRender`, `promptSetExportMany`, `promptSetExportManyAdapter`. Other mock-only files needed no change.
- `e2e/specs/story-12-14-prompt-library.e2e.mjs`: `prompt_set_create`/`prompt_set_list` now pass `kind: 'translate'`.

## Notes for phase 4
- Export-file wire from Rust must include `kind` (isExportFile rejects it otherwise); preview wire must include `kind`.
- Not run: e2e spec; full vitest.

## Orchestrator fix after phase 3

- `PromptSetExportFileWire` (commands/promptset.rs) had no `kind` while the webview guard `isPromptSetExportFile` requires it; added `pub kind: PromptSetKind` (and `Serialize` on `PromptSetKind`). Phase 4: a Rust test should assert export-many results carry `kind`.
