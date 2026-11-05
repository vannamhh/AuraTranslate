# 9-15 Webview phase handoff (for the Tests agent)

## Changed files
- `src/config/promptset.ts`: wrappers `promptSetProofreadChoiceGet|SetGlobal|SetWork|DeleteConsequence` (+ types `ProofreadChoiceWire`, `ProofreadDeleteConsequence`). `cargo test --test ipc_argument_contract` is green with the 3 exemptions gone. Scanner trap: never pass a `CMD_*` constant as a non-object second argument to a helper (`failure(CMD_X, err)` broke it); helper gets a string literal.
- `src/promptSetState.ts`: separate proofread state (`proofreadInstructionSets`, `proofreadInstructionChoice`, `loadProofreadInstructions`, `chooseProofreadGlobal`, `chooseProofreadWork(mode, name)`, `proofreadDeleteConsequence`). `createPromptSet`/`renamePromptSet`/`updatePromptSetBody`/`deletePromptSet`/`exportPromptSet(s)` take a trailing `kind` (default `translate`). The translate list and `selectedSetName` are never touched by the proofread path.
- `src/promptSetImportState.ts`: `promptImportConfirmedKind` (set on confirm from `preview.set_kind`).
- `src/PromptLibraryOverlay.vue`: kind tabs (`[data-prompt-kind-option=translate|proofread]`), choice section `[data-proofread-choice]` with selects `[data-proofread-choice-global]` / `[data-proofread-choice-work]` (work select value: `follow` | `off` | `named:<name>`), missing notes `[data-proofread-choice-missing=global|work]`, write error `[data-proofread-choice-error]`, delete hint `[data-delete-hint]` with `data-delete-consequence`. Proofread tab hides: variable chips, "Dùng bộ này", "Xem prompt cuối", "Thêm từ mẫu"; mirror renders one plain segment. A confirmed import switches the tab to the file's kind and reloads.
- `src/i18n/vi.json`: 14 `prompt.library.kind_*|choice_*|delete_consequence_*` keys. `check:i18n` and `check:tokens` green.

## Tests added
`tests/frontend/promptLibraryProofreadCard.test.ts` (9 cases): translate choice untouched across tab switch/delete; marker body no chips/highlight/warn; missing flag on both selects; both writes then re-read; Work select disabled without Work; delete consequence x3 (counter-check: replacing the consequence call with `'none'` made the 3 red); import switches tab.
Run with the existing `promptLibraryOverlayRender` / `promptSetState` / `promptImportOverlayRender` / `promptSetExportMany` / `promptTemplatePicker`: 148 pass. `vue-tsc` and eslint on src clean.

## Left for the Tests agent
- Rust "missing" counter-check (AC 2), e2e spec (template `e2e/specs/story-12-14-prompt-library.e2e.mjs`), `deferred-work.md` closure (3 `NO_FRONTEND_CALLER` items `Chủ: Story 9.15`; add `Chủ: Epic 9` item for Ice's real-use pass), the single full run (`lib.rs` changed), `Implementation Notes`.
- Not done: no mockup for the card; the card was not looked at in a real window (no screenshot). The Work select lists all resolved names (both tiers); Global select lists Global-tier names only (incl. shadowed).
- `tests/frontend/promptLibraryOverlayRender.test.ts` mock factory does not define the proofread wrappers (not needed there).
