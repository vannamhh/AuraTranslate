# Handoff phase 1 (Rust) -> phase 2 (Webview + tests)

Plan: `plan-thu-tu-nguon-va-danh-so-nghia.md`. Phase 1 is done and uncommitted; no `src/` or `tests/frontend/` file was touched.

## What landed
- Config key `dict_sources_order`: comma-joined `dict_source.code`, no migration. Written through the existing `put_config` (same as `dict_sources_disabled`).
- `BootstrapConfig` has a new field `dict_sources_order: String` (snake_case on the wire, raw string, `''` when never set). The webview must add it to `src/config/bootstrap.ts` (~:78, :195), `main.ts` (~:1222) and every BootstrapConfig fixture in `tests/frontend/{settingsUpdateSection,bootstrap,aiUsageVisibleState,panelHeaders}.test.ts`.
- Parsing is Rust-only (`core::scope::parse_source_order`: trim, drop empties, drop repeats, keep order). The webview should split on `,` the way it does for `dict_sources_disabled`.
- `lookup_dictionary` (IPC): groups are already ordered by the user order (applied after Exact/Substring choice). Response shape is unchanged.
- `list_dict_sources` (IPC): `sources` is already ordered by the user order; it now also reads the Store. Disabled sources keep their position. Shape unchanged. Codes in the order with no file are ignored; unlisted sources follow in the old default order.
- No new Tauri command. Rust signatures changed: `commands::dict::lookup(layers, query, disabled, order: &[String])` and `commands::dict::list_sources(layers, order)`. Pure helper `commands::dict::order_by_user`.

## Webview must know
- After a write of `dict_sources_order`, the webview must call `listDictSources` and `runLookup` again; Rust does the sort, the webview only swaps two rows optimistically.
- The written value should be the FULL current menu order of codes (all rows shown), so the stored order stays complete.

## Verified
- `cargo test --test dict_sources --test ipc_contract` green (95 + 48); `cargo clippy --all-targets` clean; `cargo fmt` run.
- New tests in `src-tauri/tests/dict_sources.rs`: default/chosen/orphan/duplicate/disabled for lookup, list order incl. orphan, parse_source_order.
- Not run: full suite, vitest, check:* gates.

## Unfinished
- All phase 2 items (dictSourcesState, commands/shortcuts, LookupSourcesMenu, LookupRecord numbering, vitest, COMMAND_FLOOR +2, vi.json).

## Phase 2 (Webview + tests) notes
- `dictSourcesState.ts`: `moveDictSource(code, ±1)` swaps rows in `sources` (Rust order is the only order; no separate order ref), writes the full code list via `writeQueue`, rolls back on error (latest press only), then `listDictSources` + `runLookup`. `moveFocusedDictSourceUp/Down` aim like `toggleFocusedDictSource`; `refocusSourceRow` restores focus after Vue moves the node. `main.ts` `dict_sources_order` is NOT read at boot (not needed); only the type, `KEY_DICT_ORDER` and fixtures were added.
- Commands `lookup.move_source_up/down` (`Alt+Shift+ArrowUp/Down`), ports `moveDictSourceUp/Down`, vi.json, `COMMAND_FLOOR` 267 -> 269.
- Menu: each `li.sources-row` has the item button plus two `ui-icon-btn` sibling buttons (carry `data-source-code`); `onKeydown` ignores modified arrows.
- `LookupRecord.vue`: `<ol class="lookup-senses">` + `<li class="lookup-sense">`, numbered via `lookup-senses--numbered` (>=2 senses), `--sense-digits` sets the hanging indent; left bar removed.
- Tests: new cases in `lookupSourcesPopup.test.ts` and `lookupPanelBlocks.test.ts`; counter-check (removing the refocus) turned the focus test red.
