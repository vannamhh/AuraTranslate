# Phase 2 (Rust) -> Phase 3 (Webview) handoff

Rust done, `cargo clippy --lib` clean, `cargo test --lib` new unit test and `--test config_invariants` green. Other test targets do NOT compile yet (Phase 4 owns them: `ipc_contract.rs:1690` needs `translation_origin_counts`, `proofread_write_boundary.rs` imports removed `replace_proofread_range`, plus the version 37 -> 38 / table 22 edits in the plan Code Map). No contract test for the new writers exists yet; the SQL paths are unexercised end to end.

## Wire (snake_case fields, camelCase invoke args)

- `ai_proofread_accept(segmentId, expectedText, start, end, expectedPhrase, replacement, kind)` -> the old `PromoteAiTranslationOutcome` fields flattened + `decision_id: number`. Keep `decision_id` in the `RevertItem`.
- `ai_proofread_revert(..same.., kind, decisionId)` -> `PromoteAiTranslationOutcome`. Error `ai_proofread.decision_missing` (params `decision_id`; vi.json key added) writes nothing, text included.
- `ai_proofread_decision_skip(kind)` -> `()`. Inserts one `skipped` row. Call it from `dismissProofreadFinding`.
- `ai_proofread_ignore_add` now also inserts a `not_error` row (no wire change).
- `ai_proofread_decision_counts()` -> `{ accepted, not_error, skipped, by_kind: [{ kind, accepted, not_error, skipped } x5 in ProofreadKind::ALL order] }`. Denominator = accepted + not_error + skipped; sum 0 => "chua co quyet dinh". `ignore_remove`/`clear` do not touch decisions.
- `unmark_reading_segment(segmentId)` -> `boolean` (false = had no mark, not an error).
- `clear_reading_marks()` -> `number` (rows removed; `store.write_failed` leaves marks).
- `ReadingChapter.translation_origin_counts: { self_count, other_count, bilingual_import_count }`. Omitted sentences and `''` origin are not counted; role (alt/caption) sentences are counted if not omitted. Webview merges into one label (all-zero => "chua co ban dich").

## Decisions made in Rust

- `Accept` inserts the row after the text write; `Revert` deletes (`id`, `outcome='accepted'`, `kind`) after all pure checks and before the text write, so 0 rows returns before any write.
- Migration 38 `PROOF_DECISION_DDL` (AUTOINCREMENT id, CHECKs for kind and outcome). Old Works get no backfill.
- `tests/config_invariants.rs` census bumped: `commands/proofread.rs` 7 -> 9, `segment/wire.rs` 20 -> 22, tree total 96 -> 100.
- The old `replace_proofread_range` is gone; tests import `accept_proofread_range` / `revert_proofread_range` (revert takes `decision_id` last).
