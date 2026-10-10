# Reality check: AD-53 against the code, 2026-10-10

Spine section read: `### AD-53` (architecture-auratranslate.md:911-958) and the AD-47 ③ row (:718). Paths are under `src-tauri/` unless noted.

## Verdict

Mostly true and buildable. Every claim about `write_non_user_target`, `arbitrate`, the origin table and the revert round trip holds. Three things are wrong or missing:

- the error code `segment_retired` does not exist (the real code is `segment.retired`);
- the helper behind it is not reachable from `commands/proofread.rs`;
- the ignore and filter plumbing needs a migration, an edit to the ai_boundary allow-list, and a change to the 9.1 Done wire, none of which the AD records.

## Verified true

| Claim | Evidence |
|---|---|
| `write_non_user_target(tx, segment_id, target_text, baseline_translation_origin: &str, translation_origin: Option<&str>)` is `pub(super)`. It writes `target_text`, `baseline_target_text` (same `?1`), both origin columns and `status = draft` in one UPDATE. | `src/commands/segment/targets.rs:234-257`. |
| The new accept command must sit in `commands/segment/` and gets `segment_retired`, `write_non_user_target` and `SEGMENT_STATUS_DRAFT` through `use super::*`. | The callers `review_accept.rs:82`, `tm_match.rs:370` and `history.rs:376` are all siblings. `mod.rs:121-138` shows the module list, so a new sibling file needs a `mod` line and a `pub use`. |
| Passing `Some(x)` equal to the baseline origin is already the pattern. | `chapter_read.rs:559`, `tm_match.rs:370`, `review_accept.rs:82`. |
| `arbitrate(target, baseline_text, baseline_origin) -> Result<Arbitrated, UnknownBaselineOrigin>` compares `trim` + NFC on both sides. | `src/core/segment/translation_origin.rs:37-52`. |
| The outcome table is right. Text differing from the baseline, or an empty baseline origin, gives `self`. Text equal to the baseline gives the stored origin: `other`, `bilingual_import`, or the TM pair origin. | The same function. The sources for the stored origin: bilingual import writes baseline text = target and baseline origin = `bilingual_import` (`import.rs:150-158`); a TM fill writes the pair origin into both columns (`tm_match.rs:370`, `chapter_read.rs:559`); FR94 and FR59 write `other` (`review_accept.rs:82`). |
| The refusal clause matches. `Arbitrated::Unsigned.as_str()` is `''`, and an out-of-set stored origin gives `Err(UnknownBaselineOrigin)`. | `translation_origin.rs:21-32` and `:48-51`. |
| `segment_version` is inserted only at confirm. | The only hit for `INSERT INTO segment_version` is `commands/segment/confirm.rs:272`. |
| Webview wire (see the table under finding 3): `start`/`end` are UTF-16, there is no quote, and `Done` carries `scanned_text`. | `src/config/proofread.ts` and `commands/proofread.rs`. |
| `clearProofreadFor` is called on a typed edit, and `resetProofread` clears the result. | `src/proofreadState.ts:102-123`. Whether chapter-change calls `resetProofread` was not traced; the AD labels the webview check "for drawing only", so the worst case is a stale underline. |

### Accept then revert, concretely

Let B be the baseline text, R the baseline origin, O the old text and N the new text.

1. **Unedited non-user text (O equals B after trim + NFC, R non-empty).** Accept writes baseline text N with baseline origin and origin R. Revert runs `arbitrate(N, N, R)` and gets R again. It writes O, with baseline O and origin R. The state matches the pre-accept state, and signing then gives R, the same as before.
2. **User-typed text over a non-user baseline (O differs from B).** Accept gives `self`, so baseline text N and both origins `self`. Revert runs `arbitrate(N, N, self)` and gets `self`, and writes baseline O with origin `self`. `target_text` is the same bytes. Signing O gives `self`, exactly what arbitrating O against the old baseline would have given. What is lost is the old baseline (B, R), as the AD's own warning says. After revert, retyping B and signing gives `self` where it used to give R.
3. **O empty.** Impossible in practice (no finding on empty text), and the refusal clause covers it.
4. **Byte equality.** The revert replaces `[start, start + len16(suggestion))` with the original quote. The text is the same bytes. The stored text is not normalised: only `arbitrate` normalises, and only for comparison.

### Gate `segment_baseline_guard.rs`

- It scans SQL statements, not function calls (`tests/segment_baseline_guard.rs:13-41`). A new command that calls `write_non_user_target` adds no new statement and passes without edits.
- A new command with its own `UPDATE segment SET target_text` would have to write both baseline columns, or the gate fails.
- `SRC_RS_FLOOR` is 113 and the tree has 136 `.rs` files. `assert_population_floor` also requires `floor * 5 >= live * 4` (`tests/support/boundary_scan.rs:360`), which allows at most 141 files, so there is room for 5 more. If 9.4 and 9.6 together add more than 5 files, this floor needs raising (`ai_boundary.rs:69` and `segment_baseline_guard.rs:10` carry the same constant).

## Findings (false or unbuildable first)

### 1. `segment_retired` is not an error code (false claim), and the helper is not reachable from `commands/proofread.rs`

- AD-53 rule 4 lists `segment_retired` next to `proofread.text_changed`. The real code is the dotted `segment.retired` (`confirm.rs:41-48`, `MessageKey::SegmentRetired` → `err.segment.retired`, `core/i18n/mod.rs:274`). `segment_retired` is the Rust function name.
- That function is `pub(super)`, so only `commands/segment/` siblings can call it. The accept command is fine. The ignore command (rule 6) lives in `commands/proofread.rs` and runs "the same four checks", but it cannot call `segment_retired`.
- Precedent for widening it: `segment_not_found` is `pub(crate)` (`confirm.rs:33`). `segment_not_in_chapter` is already imported by proofread.rs from `commands::aiprompt`.

**Fix.** Write `segment.retired` in rule 4. Put the four checks in one function in `commands/segment/` (for example `check_proofread_target`) returning a typed rejection, and call it from both commands. Widen only the entry points the ignore command needs to `pub(crate)`. This also keeps "same checks, same order" true by construction, the same lesson as the 9-12 shared call path.

### 2. Ignore and filter plumbing is wider than the AD says (unrecorded work, and one gate edit)

- **No table exists.** `grep -i proof src/core/store/*.rs` returns nothing. `PROOF_IGNORE` needs a new `PROJECT_MIGRATIONS` step. Per AGENTS.md that is shared wiring, so a full run is required, and the indexer's column-read law (`core/library/indexer.rs:1560-1567`) applies.
- **The module doc of `commands/proofread.rs` says "Read-only: nothing here writes project.db".** Rule 6 makes it the first writer, so the doc and the 9.1 test that asserts "no project.db write" need a deliberate exception. No gate forbids the write: `store_boundary.rs` and `meta_write_boundary.rs` are about other targets. Confirm before building with `grep -n 'project' tests/store_boundary.rs`, which was not read here.
- **The ai_boundary allow-list is exact.** `ALLOWED_AI_PATHS_IN_PROOFREAD_SEAM` has exactly 5 entries and the test asserts each one is still named (`tests/ai_boundary.rs:513-518`, `:582-604`). The filter function lives in `core/ai/proofread.rs`, so `commands/proofread.rs` must import it. That is a sixth path and an edit to the allow-list plus its seeded-violation test. AD-53 says nothing about it.
- **The filter must be pure.** `core/ai` can hold a store-reading function (`rag.rs:46` imports `Store`), but `client.rs` documents the opposite stance. The ignore list has to be read in `prepare_proofread_call`, which has `Option<&OpenWork>` before the `.await`, and passed as a plain slice into the filter.
- **The AD-53 claim "`commands/segment/` does not import `core::ai`" is satisfiable.** The accept command only needs strings and integers.

**Fix.** Add to rule 6 or the "Binds" line: the migration step, the new allow-listed path, the pure-filter-with-list signature, and a named exception to the read-only line in the module doc.

### 3. "The 9.1 wire is unchanged" is half true

| Part of the wire | Status |
|---|---|
| The finding shape (no quote, UTF-16 `start`/`end`) | Unchanged. Cutting the quote in the webview is feasible: `scannedText.slice(start, end)` is already UTF-16 and `scanned_text` is held in `proofreadScannedText` (`proofreadState.ts:13,24`). |
| `Done` | Changes. Rule 6 says the scan result "mang số phát hiện đã lọc". That is a new field on `AiProofreadOutcomeWire::Done` (`commands/proofread.rs:50-58`), plus `ProofreadOutcomeWire` and `isProofreadOutcomeWire` in `src/config/proofread.ts:21-60`, plus `tests/ai_proofread_contract.rs`. |
| `scanned_text` empty on the `EmptyText` path (`proofread.rs:208`) | Harmless. |

**Fix.** Say "the finding shape is unchanged; `Done` gains `filtered: u32`". Do not claim the wire is untouched.

### 4. New error keys need the full i18n set, and the prefix is a choice

- Pattern: a `MessageKey` variant (`core/i18n/mod.rs`, for example `ReviewChangeTextChanged => "err.review.change_text_changed" ["segment_id"]` at :937), a Vietnamese string in `src/i18n/vi.json` (:219 for the precedent), and an `IpcError::new(code, key, params, retryable)` constructor. `proofread.text_changed` and `proofread.range_invalid` fit that convention.
- The backend scan errors use `ai_proofread.*` (`ai_proofread.reply_malformed`, `err.ai_proofread.reply_malformed`), while the webview-only codes use `proofread.*` (`proofread.flush_failed`). The AD puts backend codes under `proofread.*`. That works but mixes the two; pick one prefix in the AD.
- Use the same `["segment_id"]` param as `ReviewChangeTextChanged`.
- Two different failures share `proofread.text_changed` (whole text changed, and phrase mismatch at the range). This is a legitimate choice, but note it so 9.4 tests assert both paths.
- **Missing rejection.** A suggestion that would leave the text empty after trim makes `arbitrate` return `Unsigned`, and rule 1 says refuse. No error code is named for that case (nothing in the four listed). Name one, or state that it reuses `proofread.range_invalid`.

### 5. Minor and verified-OK points

- `trim` + NFC for the ignore signature is the same normalisation as `arbitrate` (`translation_origin.rs:37-42`). The AD's "same operation AD-50 rule 4 uses" is accurate. One difference worth stating: `arbitrate` normalises only for comparison, while the ignore signature stores the normalised value, so a stored phrase never byte-matches the segment text. Matching must normalise the live phrase too.
- There is no existing helper to convert a UTF-16 range to byte offsets or to detect a split surrogate. Only `utf16_len` exists (`core/ai/proofread.rs:99`). Rule 4's `range_invalid` check needs a new pure helper, which belongs in `core/segment` or `core/ai/proofread.rs` and is unit-testable.
- `locate_findings` returns `match_indices` from byte offsets, so findings always sit on char boundaries. A split surrogate is therefore only reachable from a hostile or buggy caller, which is fine for a defensive check.
- `CommandRegistry` (AD-34) is a webview concept (grep hits in `src/config/segment.ts` and others). It is not a Rust registry, so "register both commands" means webview registration plus `generate_handler!` in `lib.rs:1294-1295`, which is shared wiring for the full-suite rule.
