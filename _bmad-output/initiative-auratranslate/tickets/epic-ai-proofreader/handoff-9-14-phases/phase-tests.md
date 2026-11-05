# Phase 4 (Tests) handoff, story 9-14

Nothing committed. Production code untouched in this phase (counter-checks restored byte-for-byte, verified with `cmp`).

## Changed (src-tauri/tests/)
- `proofread_instruction_contract.rs`: golden (a, Rust phase) plus order (b), marker set (g, all six markers, plus "longer line / other case untouched"), marker placeholder verbatim in assembly (h, pure side), resolution (c: Work wins, `NULL` follows Global, `off` silences, Global-only name from Work, Work shadows Global, blank body), missing name error params (d, store side), kind boundary (e: same name `translate`/`proofread`, translate list never shows `proofread`), rename (j: Work-tier rename, Global-tier rename, shadowed Global rename keeps Work choice, failed rename leaves choices), delete consequence AC (3 outcomes, both tiers), choice writes (one UPDATE, `off` wins and NULLs the name, blank rejected, Global `None` deletes the row).
- `proofread_instruction_wire.rs` (new): `MockRuntime` app + loopback SSE server through `wire::ai_proofread_segments` and `wire::ai_proofread_scan_record`. Covers I/O rows: no choice (prompt == `assemble_proofread_batch_prompt(.., None)`, trace `NULL`/`NULL`), Global B (instruction before `Reply with only a JSON array`, no `system` message, trace `B/global`), Work A beats Global B, Work naming a Global-only set (trace tier `global`), Work `off`, missing name (0 connections accepted, params `name`/`tier`, no `proof_scan` row), translate/proofread same name, marker verbatim end to end (h), prose reply (i: one batch unreadable -> `not_scanned = 1`, batch event has no segments; all batches prose -> existing `reply_malformed` error with `not_scanned = 3`), FR76 AC (3 batches each carry the long instruction, usage per batch sums to 18 x batches), rename then scan (j end to end), scan code rejected when unknown, `0`, older than the latest scan, or reused.
- `proofread_memory_contract.rs`, `ai_proofread_contract.rs`: compile fixes only (`scan_id`, `instruction`, `ScanTrace` arg, `Done { scan_id }`, serialised `"scan_id":0`).
- `ipc_contract.rs`: `ai_proofread_scan_record` params gain `scan_id: u64`; the 3 new promptset wires are registered and keep their parameter lists.
- `config_invariants.rs`: census `promptset.rs` 9 -> 12 plain, tree total 111 -> 114 (owner text: chua do, chu: Dev).

## Counter-checks (real removal, one target each, then restored)
| Case | Removal | Target | Red |
|---|---|---|---|
| a | blank line inserted when custom is empty | contract | 3 (2 golden + order) |
| b | custom placed before the intro | contract | order case only |
| c | Work `Named` arm folded into Global branch | contract | 5 |
| c | Work `Off` arm folded into Global branch | contract | `the_work_choice_wins_*` only |
| d | missing set returns `Ok(None)` instead of error | wire | `a_chosen_name_that_is_gone_*` only |
| e | `kind = ?1` in the read query dropped | contract | translate-list and same-name cases |
| g | SOURCE/TEXT markers dropped from the set | contract | zero-width-space case only |
| g | SEGMENT markers dropped from the set | contract | zero-width-space case only |
| h | body passed through `expand_prompt_body` | contract | verbatim case only |
| i | `not_scanned += ..` removed from `Malformed` branch | wire | both prose cases |
| j | Work choice UPDATE rewrites to old name | contract | `renaming_the_set_the_open_work_resolves_*` only |

First attempt at the g/SEGMENT removal inverted the match by mistake (every plain line flagged) and was discarded; the number above is the corrected run.

## Verification run (full)
One full run, tree as left: `npm run build`, `cargo fmt --check`, `cargo clippy --locked --all-targets -D warnings`, full `cargo test --no-fail-fast` (106 binaries `ok`, 0 failed), full `npx vitest run` (254 files, 4066 tests), `eslint src e2e tests`, and `check:deps|tokens|i18n|commands|layout|panel-refs|doc-refs|dict|dict-manifest|gates|debt-owner`: all green.
- The first full `cargo test` stopped at `ipc_argument_contract` (plain `cargo test` aborts at the first red binary, so later binaries only ran in the `--no-fail-fast` rerun): the three new `prompt_set_proofread_*` commands have no TS caller. Fix: added them to `NO_FRONTEND_CALLER` in `tests/ipc_argument_contract.rs` (a named exemption that dies when the 9-15 wrappers land).
- Not run: `check:scope`, `check:scope:bundled` (outside pre-push), e2e.

## Open / risky
- No e2e spec for proofread exists in `e2e/`; the plan's "one e2e proofread spec before push" has nothing to run. Not invented here.
- Frontend mocks returning `done` without `scan_id` (phase 3 note) left as is.
- `ai_proofread_scan_record` consumes the scan code on first use: a webview retry after a failed record call is rejected by design (AD-68 decision of Ice 2026-11-05).
