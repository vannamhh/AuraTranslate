# 9-15 Rust phase handoff (for the Webview agent)

## New command
`prompt_set_proofread_choice_get` (no args) in `commands/promptset.rs::wire`, registered in `lib.rs`.
Returns (snake_case):
```
{ global: {name, missing} | null,
  work: {mode: "follow_global"|"off"|"named", name: string|null, missing: bool} | null }
```
- `global` null = nothing chosen. `work` null = no Work open (no error).
- `missing` = chosen name is in neither tier's `proofread` sets (same lookup as `resolve_proofread_instruction`, via `resolve_two_tiers`). `off`/`follow_global` always `missing: false`; `name` is null for them.
- Note the Global `missing` also counts a Work-tier set of that name as resolving (two-tier lookup).
- Core: `core/promptset/store.rs::read_proofread_choices`; pure fn `prompt_set_proofread_choice_get(global, open)`.

## Already existing wires (9-14) now needing wrappers
`prompt_set_proofread_choice_set_global {name: string|null}`, `prompt_set_proofread_choice_set_work {name: string|null, off: bool}` (no Work => `no_work_open` error), `prompt_set_proofread_delete_consequence {tier, id}` -> `"none"|"global_takes_over"|"set_missing"`.

## Behaviour changed at the source
- `kind = proofread`: `prompt_set_create` / `prompt_set_update_body` return empty warnings (`unknown_markers: []`, `glossary_terms_missing: false`); `prompt_set_list(kind=proofread).variables` is `[]`; import preview warnings are empty when the file's kind is `proofread`. Webview must not render variable chips or marker highlighting for this kind (rendering from an empty `variables` is not enough for highlighting: `splitPromptBody` in the overlay must be skipped for proofread).
- Translate behaviour unchanged.

## Tests done
`tests/proofread_instruction_contract.rs`: 6 new cases (read states, missing flag, agreement after rename/delete, two-tier, no Work, warnings/variables by kind). `ipc_argument_contract.rs`: `NO_FRONTEND_CALLER` now 3 entries (the three `prompt_set_proofread_*` removed) — it requires TS wrappers in `src/config/promptset.ts` whose invoke arg names match the Rust params (`name`, `off`, `tier`, `id`) or it goes red until the wrappers exist. `ipc_contract.rs`: registration + param list for the new command.

## Not done (later phases)
Webview, vi.json, vitest, e2e, `deferred-work.md` closure, full run (lib.rs changed => one full run at the end).
