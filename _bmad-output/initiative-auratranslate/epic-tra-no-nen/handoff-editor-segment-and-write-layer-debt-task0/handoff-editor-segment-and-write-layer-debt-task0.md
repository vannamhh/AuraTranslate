---
type: handoff
title: "Story 11.5 — Task 0 re-read (HEAD 968529a1b957baf4d6dea45de8591184772a8cb3, 2026-09-27)"
status: done
created: 2026-09-27
skill: bmad-build
---

# Story 11.5 — Task 0 re-read (HEAD 968529a1b957baf4d6dea45de8591184772a8cb3, 2026-09-27)

43 open/🟡 items whose last `Chủ:` is Story 11.5, listed with the `check:debt-owner` parser. Four read-only agents, one per group; the planning session's corrections are in the spec.

## Group A — store/write layer (HEAD 968529a)

### L251 — backup copy not atomic, not verified
Claim: `backup_before_migration`'s `fs::copy` has no post-copy size check and no atomic rename, so a torn copy (disk full, FS error not surfaced through `Result`) can look like a valid `.bak-v{n}`.
Status: still true. `schema.rs:2491-2522` — TRUNCATE → `busy==0` check → `std::fs::copy(path, &target)` straight to the final name (line 2516), no size check, no rename.
Ice decision: none specific; assigned to 11.5 by the batch reassignment doc (not a numbered phiếu quyết).
Disposition: fix in Rust. Copy to `<target>.tmp` in the same dir, compare `fs::metadata` sizes of source and tmp, then `std::fs::rename(tmp, target)` (atomic, same filesystem); on any failure remove the tmp file and return `StoreError::OpenFailed`. Reuse `StoreError::OpenFailed`; don't touch the TRUNCATE/`busy==0` gate at lines 2497-2510 — correct and unrelated. Extend `one_step_runs_and_a_backup_is_written_first` (`store_contract.rs:1240`, already asserts `backup.exists()`/`file_len>0`) to also assert no `.tmp` remains. Caveat: a real "torn write" isn't economically fault-injectable in a unit test — the rename gives atomicity by construction, but the regression guard here is weaker than a true red/green fault injection; say so, don't overclaim it.

### L254 — `target_version` stayed `pub(crate)`
Claim: no external caller (e.g. future IPC diagnostics) could read the target schema version without opening a `Store`.
Status: moot. `schema.rs:2435` is `pub(crate)`, but that's crate-wide, and same-crate callers outside `core::store` already exist (`core/library/indexer.rs:1436`, `core/store/mod.rs:651,948`). `src-tauri` has no external crate consumer — a future diagnostics command lives in the same crate and can call it today.
Disposition: KHÔNG LÀM — the worry doesn't hold under this crate's actual boundary. Reopen if `src-tauri` is ever split so a diagnostics surface needs a separate crate.

### L260 — `Writer::shutdown()` `handle.join()` has no time cap
Claim: an unbounded `join()` could hang `RunEvent::Exit` forever if a future write job blocks.
Status: unchanged, `writer.rs:317-333`. Its own doc-comment (320-325) already states the accepted rationale: finite queue, each job one local transaction on a connection nobody else holds, no calling out — matches Ice's 2026-08-04 verdict verbatim.
Ice decision: **2026-08-04 code review** — accept the risk for v1, enforce by manual review each time a new story writes through `core/store`, not by mechanism. Restated by every `(rà sổ nợ)` sweep since, not superseded.
Disposition: KHÔNG LÀM — a live, still-honored decision, not stale debt (Stories 1.8's `save_value`, 1.15's `create_work` were both reviewed against this exact invariant and passed). Reopen when a future job closure does I/O, calls out, or nests another `Store::write`.

### L264 — `ReaderPool::acquire()` `Condvar::wait` has no time cap
Claim: same shape as L260 for the read path — could wait forever if the pool is drained (leaked `Lease` or a stuck reader job).
Status: unchanged, `reader.rs:146-166`; doc-comment (159-160) already states the rationale (only `ReaderPool::close` wakes waiters).
Ice decision: **2026-08-04 code review** — accept as-is; read path has no blocking side effect like a write job, `Lease` leak judged lower-risk.
Disposition: KHÔNG LÀM, same reasoning/reopen class as L260 (a measured `Lease` leak or a blocking reader job).

### L284 — duplicate observation on the same first-real-migration backup
Claim: "first real migration on a populated `global.db`" observation, pointing at the same defect as L251, not a new one.
Status: confirmed duplicate — its own 2026-09-23 sweep line says so ("chỉ trỏ tới cùng khiếm khuyết đã xác nhận … fs::copy không xác minh").
Disposition: no separate action — resolved by the L251 fix; close this pointer alongside L251 in the commit message.

### L2211 — hard exit (`panic=abort`/`SIGKILL`/power loss) skips the flush path
Claim: `wire_exit_flush` only covers normal `WindowEvent::CloseRequested`; a hard kill never reaches it.
Status: unchanged, already documented as intentional at `lib.rs:1173,1340,1471-1472`. `deferred-work.md:38` (origin item) already records this as safe by design: WAL guarantees no data loss (SQLite replays on next open); only cosmetic cleanup (a `.db-wal` left at size) is skipped.
Ice decision: none on this pointer directly; it inherits from `panic = "abort"`, frozen per `src-tauri/AGENTS.md` ("`[profile.release]` is frozen").
Disposition: KHÔNG LÀM. No in-process code can run after `SIGKILL`/power loss; un-freezing `panic=abort` to add a signal handler is a change to a frozen profile — an AD, not a story fix, out of 11.5's authority. Reopen if a future NFR caps stale `.db-wal` disk usage after a crash, or a SIGTERM-specific path is explicitly requested.

### L4023 — `ORDER BY ord` missing a secondary key, sweep incomplete
Claim: `ord` is deliberately non-`UNIQUE` (`schema.rs:279-282`); some `ORDER BY ord` queries in `commands/**` lack a tie-breaker.
Status: **one instance left**, line-drifted from the 2026-09-23 note (`project/mod.rs:1879` → `:1922-1923`): `read_chapter_segment_texts` — `ORDER BY ord` with no `, id`. Verified by grepping every `ORDER BY ord` in `src-tauri/src/commands/`: 15 other call sites (`chapter.rs:270,275,505,685,690,794,1050`; `segment.rs:908,1425,3129,3292`; `project/mod.rs:5137`) already carry `, id`/`DESC, id DESC`; remaining hits are doc-comments.
Disposition: fix in Rust. Change `project/mod.rs:1923` to `ORDER BY ord, id`, matching every sibling site (rationale already written at `segment.rs:1005-1007`). Caller: `read_chapter_segment_texts`, called from `project/mod.rs:2085`, tested at `commands/project/tests.rs:279,357`. Guard: extend one of those tests to insert ≥2 live segments sharing one `ord` value in an order forced to diverge from `id`, then assert output order == ascending `id`. Flag: SQLite's tie-break for equal `ord` already tends to follow rowid(==`id`) order in practice, so this guard risks a symmetric fixture (passes with and without the fix) — run the counter-check (temporarily revert `, id`) before trusting it.

### L4849 — CI WAL-test regression window, reassigned to 11.5 (Ice question)
Claim (Story 11.1 lô D Quyết định 8): the 08-16→08-19 regression window (runner variance ⒜ vs the 208-line migration diff ⒝) is measured-closed for ⒜ — 14 sampled CI runs (09-14→09-25) show `the_wal_stops_growing_once_it_crosses_the_threshold` red **7/14**, both OSes, same signature `wal_checkpoint(PASSIVE) blocked: busy=1` — but "không đủ để tự sửa mã đường checkpoint", handed to the next `core/store` story.
Status at HEAD: unchanged. `checkpoint.rs:270-295` already treats `busy!=0` PASSIVE as expected/non-fatal, retries next `checkpoint_tick` by design (`AD-12` forbids escalating PASSIVE to TRUNCATE on the background path). `.githooks/pre-push` already runs `scripts/ci-previous-verdict.mjs` (phiếu quyết #37), which warns without blocking on a red previous push.
Two-option call for Ice:
- **A — KHÔNG LÀM.** ⒜/⒝ is closed (runner variance, not a regression); PASSIVE-busy retry is the designed behavior; `ci-previous-verdict.mjs` already surfaces red CI without blocking. Reopen if the flake rate rises or actually blocks a release.
- **B — retune the test's CI timing budget** (`settled_wal_len`'s deadline / checkpoint retry cadence for this test). Cost: needs new CI runs to validate (Ice's Mac isn't representative — local `rustc` deliberately differs from CI's pin, per `AGENTS.md` §"This machine"); risks moving the flake elsewhere, the exact trap `settled_wal_len`'s own redesign history (`store_contract.rs:554-577`) already hit once on 2026-09-13.
No new gate, no new dependency either way.

### L10545 — `settled_wal_len` gives no signal on deadline give-up (Ice question, "xếp nợ đứng tên Ice")
Claim: on deadline expiry the poll loop falls through to `file_len(wal)` with nothing distinguishing "3 stable polls reached" from "gave up while still moving" — risking a silent repeat of the "captured too early" defect it was redesigned to fix.
Status: still true. `store_contract.rs:563-578` — `while stable_polls < 3 && Instant::now() < stop {…}` then unconditional `file_len(wal)`. Used at 500 ms deadline, 3 call sites (`:771,779,782`).
Cross-measurement from L4849: CI genuinely hits WAL-checkpoint slowness under load (7/14 red, both OSes) — a deadline give-up here is plausible, not theoretical, in CI.
- **A — panic loudly when the deadline is hit without 3 stable polls.** Makes the give-up visible at the point of ambiguity. Risk: given L4849, could turn today's tolerated flake into a harder failure more often.
- **B — keep the blind read but tag it** (return a settled/gave-up value, or log to `store.diagnostics()`), let call sites decide if an unsettled read invalidates their comparison. No new flakiness risk, but only surfaces the ambiguity after a failure elsewhere — doesn't close the underlying risk, just makes it legible.

### L10561 — `file_len` maps any `fs::metadata` error to `0` (Ice question, "xếp nợ đứng tên Ice")
Claim: `fs::metadata(path).map(|m| m.len()).unwrap_or(0)` turns any stat error — not just "doesn't exist" — into a silent `0`.
Status: still true, `store_contract.rs:82-84`. All 4 call sites checked: `settled_wal_len` (line 577, covered by L10545); `close_truncates_the_wal_to_nothing` line 991 asserts `file_len(...) > 0` — error fails **loud**; line 998 `let len = file_len(&wal)` feeding `assert!(!wal.exists() || len == 0, …)` — here an error IS masked as a legitimate `0`, the one real silent-pass instance; `one_step_runs_and_a_backup_is_written_first` line 1262 asserts `>0` — loud.
- **A — panic on any `fs::metadata` error**, per the ledger's own suggestion. Simplest; correct only if no call site legitimately expects a missing sidecar as `0`.
- **B — distinguish `io::ErrorKind::NotFound` (→ `0`, legitimate "not created yet") from every other error (→ panic with path+error)**. Matches line 998, where the WAL sidecar may legitimately not exist post-`close()`. Safer given that site.
Caveat for Ice either way: at line 1000 the assertion uses `!wal.exists()`, and `Path::exists()` itself swallows **all** stat errors (not just `NotFound`), returning `false` on any of them. Tightening `file_len` alone doesn't fully close the silent-pass risk there unless the assertion also switches to `path.try_exists()` (surfaces IO errors as `Err`) instead of `.exists()`.

---

### Code map (A)
- `src-tauri/src/core/store/schema.rs:2491-2522` `backup_before_migration` (L251/L284 fix); `:2435` `target_version` (L254, no change)
- `src-tauri/src/core/store/writer.rs:317-338` `Writer::shutdown`/`Drop` (L260, no change)
- `src-tauri/src/core/store/reader.rs:146-166` `ReaderPool::acquire` (L264, no change)
- `src-tauri/src/lib.rs:1094,1341,1474,1002-1005` exit-flush wiring (L2211, no change)
- `src-tauri/src/commands/project/mod.rs:1916-1932` `read_chapter_segment_texts` (called `:2085`) — L4023 fix target
- `src-tauri/src/commands/project/tests.rs:279,357` — tests to extend for L4023's guard
- `src-tauri/src/core/store/checkpoint.rs:270-295,333-388` — PASSIVE/TRUNCATE busy handling (L4849)
- `.githooks/pre-push` + `scripts/ci-previous-verdict.mjs` — existing CI-red warning (L4849)
- `src-tauri/tests/store_contract.rs:82-84` `file_len`, `:563-578` `settled_wal_len`, call sites `:771,779,782,991,998,1262` — L10545/L10561

## Group B — translation origin and write paths (HEAD 968529a)

### L3568 — no on-disk answer to "which segment_version is in use"
Claim: closing Story 2.6's origin-highlight debt safely needs answering "in use" by `id`, i.e. a `segment.current_version_id` pointer.
Status: unchanged — `segment_version` DDL still 4 cols (`schema.rs:1495-1500`); 0 hits for `current_version_id` repo-wide.
Ice decision **#95** (`sprint-change-proposal-2026-09-24b-phieu-quyet.md:313-314`): "không mở năng lực 'phiên bản nào đang dùng'; không thêm con trỏ `segment.current_version_id`." KHÔNG LÀM.
Disposition: **KHÔNG LÀM** (closed by #95). Reopen only via a new AD.

### L3650 — FR101 restore doesn't restore origin (AD-47 ⑤, named exception)
Claim: `restore_segment_version` can't return origin because `segment_version` carries none.
Status: unchanged, and the spine already names this as intentional — AD-47 ⑤ (`ARCHITECTURE-SPINE.md:714,720-722`): "Khôi phục (FR101) làm (a) mà KHÔNG làm (b)... hệ quả bắt buộc." Not undetected debt; a documented invariant, reconfirmed true.
Disposition: **KHÔNG LÀM** — decision #95 forecloses the only path (a version-scoped origin column) that would close it. Cite AD-47 ⑤ + #95 when closing the ledger line.

### L3660 — Rust trusts webview `textAtLoad`; no Rust gate catches a valid-but-wrong value
Claim: only e2e exercises the real wire shape; a future caller sending a syntactically valid wrong mốc is undetected.
Status: unchanged — `segment.rs:2299-2301` doc-comment still says exactly this. Single UI caller `editor.confirm_segment` (`src/commands/index.ts:2611`); second real caller still `e2e/specs/segment-history-restore.e2e.mjs:77,175` (direct `invoke`).
Ice decision cited in-code: Quyết định #2(b) (2026-08-16) already chose "mốc lives in webview, Rust trusts it" — this is that decision's accepted, documented cost, not new debt.
Disposition: **KHÔNG LÀM**. Reopens only if a future AD makes the mốc Rust-derivable (e.g. persisted server-side).

### L3754 — session-pinned mốc vs. live-read origin loses origin-at-load on a second sign-back
Claim: sign → edit → sign (`self`) → edit back to load text → sign again ⇒ origin stays `self` instead of returning to the origin the segment had at panel load (e.g. `bilingual_import`).
Status: confirmed live. `confirm_segment` origin branch (`segment.rs:2438-2443`): text differs ⇒ `self`; origin empty ⇒ `self`; else **keep the live on-disk `translation_origin`** — no "origin at panel load" concept exists in the Rust signature or in `editorPanelState.ts`.
Ice decision **#96** (`...-phieu-quyet.md:316-317`): mốc theo phiên panel — text back to load-time text ⇒ origin-at-load returns (FR118).
Disposition: **FIX (Rust + webview)**, Story 11.5.
- `commands/segment.rs::confirm_segment` — add param `origin_at_load: &str` (same trust model/precedent as `text_at_load`). Else-branch returns `origin_at_load` instead of the live-read value; still one `UPDATE` writing both columns (AD-47①(b) unchanged). `mod wire` pass-through; `tests/ipc_contract.rs` field-name lock needs `originAtLoad`.
- `src/config/segment.ts` adapter + `editorPanelState.ts` — capture origin at the same moment the snapshot/`text_at_load` is captured, thread through `confirmCurrentSegmentUnguarded`. Update `tests/frontend/editorConfirmSegment.test.ts` and the two direct `e2e/specs/segment-history-restore.e2e.mjs:77,175` invokes (new required key — this IS the wire-shape guard, per L3660).
- Reuse: `text_at_load` plumbing/trust model. Do NOT touch: AD-31 tables, AD-47③/④ tables, `segment_version`'s 4-column shape (L3650/L3568 stay KHÔNG LÀM).
- Guard test (real caller): `segment_contract.rs` case calling `confirm_segment` twice in one session — sign with `origin_at_load="bilingual_import"`, edit, sign (⇒ `self`), edit back, sign again ⇒ assert `bilingual_import`. Red today without the fix.

### L3787 — no read-side validation of `translation_origin` against the closed catalogue
Claim: `TRANSLATION_ORIGINS` is only checked by one test against source-code values, never against column values; a 5th value from a future `.atproj` passes through.
Status: unchanged — `segment.rs:2042-2048` still states the gap; `is_translation_origin` still doesn't exist (0 hits). Confirmed **no `CHECK` constraint** on the column (`schema.rs:1786-1795`).
Task 0 measurement Ice asked for ("sàn phiên bản đã chặn sẵn ca này chưa"): **No.** `Store::open`'s `SchemaTooNew` (`core/store/mod.rs:616-659`) only compares migration-step counts; adding a 5th origin value needs no DDL/migration step (no CHECK exists), so version count is unaffected and `SchemaTooNew` would not catch it. Separate, real gap.
Ice decision **#97** (`...-phieu-quyet.md:319-320`): từ chối mở `.atproj` có giá trị lạ, cùng khuôn lược đồ mới hơn.
Disposition: **FIX (Rust)**, Story 11.5. No migration required (no DDL shape change).
- New scan at open time: `commands/project/mod.rs::open_work` (`:5050`, alongside `MetaError::SchemaTooNew` at `:5059`) or inside `Store::open` — `SELECT DISTINCT translation_origin FROM segment WHERE translation_origin NOT IN (...)` against `TRANSLATION_ORIGINS` (`segment.rs:2065-2069`, reuse the const).
- New closed `message_keys!` entry (e.g. `store.unknown_translation_origin`), mirroring `SchemaTooNew`'s shape (`core/store/mod.rs:403,461,475,496,531,576,659`) — don't reuse `store.schema_too_new`.
- Guard test: seed a `project.db` via raw SQL with a 5th origin string, call the real `open_work`/`Store::open`, assert refusal; remove the check to confirm it wrongly opens.

### L3802 — Unicode NFC/NFD not covered in FR117 mốc comparison
Claim: `trim()`-only comparison in `confirm_segment` misses precomposed-vs-combining-mark equivalents.
Status: unchanged (`segment.rs:2426-2430`). `unicode-normalization` is only **transitive** today (`Cargo.lock:5496-5499`, v0.1.25) — absent from `Cargo.toml [dependencies]`.
NFR15 licence check (done now): `~/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/unicode-normalization-0.1.25/Cargo.toml:50` declares `MIT OR Apache-2.0`; both `LICENSE-MIT`/`LICENSE-APACHE` present in source and match (✓-grade evidence). GPLv3-compatible.
Ice decision **#105** (`...-phieu-quyet.md:349-350`): nhận phụ thuộc qua cửa NFR15.
Disposition: **FIX (Rust)**, Story 11.5.
- Add `unicode-normalization = "=0.1.25"` to `src-tauri/Cargo.toml [dependencies]` (0 new crates — already resolved in lock).
- Record the Stack-table row in `ARCHITECTURE-SPINE.md` **before** the `Cargo.toml` edit (NFR15 order).
- Apply `.nfc()` to both sides of the `segment.rs:2438` comparison, composed with (not replacing) `trim()`.
- Guard test: confirm where `text_at_load` is NFC and current `target_text` is the same string NFD — assert no false `self` flip. Red today without `.nfc()`.

### L4115 — `restore_segment_version` + empty text at merge/split — no contract test
Claim: 2026-08-17 review correctly found Story 2.9 doesn't touch this (merge already filters empty fragments), but no contract test exists for the general claim.
Status: still true — `regroup.rs:131,188` still filter empty fragments; none of `segment_contract.rs`'s 8 `restore_segment_version` cases (`:5043,5135,5163,5211,5248,5310,5336,5376`) combine merge/split with empty `target_text`.
Disposition: **FIX (Rust, test-only)**, Story 11.5, low-risk. Add one case: merge two unconfirmed empty segments, leave `target_text` empty, call `restore_segment_version` — assert today's real behaviour (no `segment_version` row exists for a never-confirmed merged segment). No production code change; do NOT touch `regroup::merge`'s filter or the `!current_text.is_empty()` exemption at `:823` (both already correct).

### L12143 — `editorPromoteAiTranslationError` exported, read by nobody (Ice options)
Claim: ref exists (`editorPanelState.ts:284-286`, set `:316`/`:836`) but nothing else reads it — a failed `⌘⇧↵` promote is silent.
Status: confirmed — grep for the export across `src/`,`tests/` finds only its own declaration. A Phase-2 attempt was reverted because `aiTranslate.test.ts:146` and `aiPromptInspector.test.ts:150,199` `vi.doMock` narrow object-literal factories omitting it, turning 24 green tests red.
Assignment: xếp nợ **đứng tên Ice** (`...no-dung-ten-ice.md:43`, weak match with L12359 via shared `promote_ai_translation` path) — options, not a pick.
- **A — build it** (matches spec 4.10 Decision 4): wire into a `tError()` surface (same shape as `editorConfirmError`/`splitChapterError`, `:833-836`); same phase, fix both mock factories. Cost: one UI surface + 2 factory edits + re-verify the ~24 fragile tests (not a full run).
- **B — drop the dead export**: remove it and its `:836` reset; defer to whichever story closes the UX-DR30 AI-error cluster. Cost: near zero, reopens spec 4.10 Decision 4 as unmet.
No measurement favors either — a product-surface timing call.

### L12359 — `promote_ai_translation` overwrites unconfirmed drafts, no confirmation (AD-49 iii)
Claim: `segment.rs:2107` UPDATEs `target_text`+`translation_origin` unconditionally, no check the about-to-be-lost draft has a `segment_version` copy. Sub-issue: `replaceEditorSegment` replaces the whole cell, killing native `⌘Z` history + unflushed keystrokes under the caret.
Status: confirmed — no `needs_confirmation`/`force` in `promote_ai_translation` (`:2107-2132`). `restore_segment_version` (`:807-838`) already implements the AD-49 pattern: `EXISTS(...segment_version...)` check, `needs_confirmation`+`unsigned_draft` held out if absent, `force` to override, zero bytes written otherwise.
Assignment: xếp nợ **đứng tên Ice**, and AD-49 itself declines to choose ("AD không chọn cách sửa") — options required.
- **A — mirror `restore_segment_version`**: add `force: bool`; if current text is non-empty and has no `segment_version` copy, return `needs_confirmation` holding the draft instead of writing; caller re-invokes with `force=true` (same UX FR101 trained users on). Cost: symmetric with an already-tested pattern (8+ contract cases); one IPC-shape change + `ipc_contract.rs` update; two steps only when it would destroy text.
- **B — auto-preserve, no dialog**: `INSERT` the current `target_text` into `segment_version` before the `UPDATE`; promote stays one keystroke, recovery via the history panel. Cost: changes when `segment_version` rows get created (today only `confirm_segment`'s transition, `:2448-2452`) — needs its own AC on whether an implicit promote-version row counts as a "version" for FR101's UI, and whether `save_segment_targets`/`flush_segment_targets` (`:2093`, deliberately unversioned) should follow. Larger blast radius.
- `replaceEditorSegment`'s native-undo/unflushed-text loss is orthogonal to A/B (a DOM whole-cell-replace-vs-patch problem) — flag as unscoped follow-on regardless of choice.
No measurement decides A vs B; A is lower blast radius/precedent-matching, B is lower UI friction but reopens versioning semantics.

### L12364 — AD-49 Rule ① has no enforcement (Ice options)
Claim: `keys.ts` only yields the typing zone for chords lacking a primary modifier; a future `Mod+Z` would still `preventDefault` inside the Editor, compiling/testing green. 0 `KeyZ` bindings exist today.
Status: confirmed — `keys.ts:415` `lacksPrimaryMod = (m) => !m.meta && !m.ctrl`; `:510` `if (lacksPrimaryMod(entry.mods) && isTypingZone(...)) return false` — any `Mod+…` chord skips the yield. 0 `KeyZ` registrations in `src/commands/index.ts`.
Assignment: xếp nợ **đứng tên Ice**; both structural options from the ledger item confirmed still unbuilt.
- **A — hard-code the yield**: special-case `Mod+Z`/`Mod+Shift+Z` in `handle()` (`:510`) to always yield when `isTypingZone`. Cheapest, but a rule living only in the interpreter, not checked against future registrations.
- **B — extend `scripts/check-commands.mjs` Kiểm K** (`:2233` onward, self-checking pattern already at `:393-399`) with a scan asserting any `KeyZ`+primary-modifier command is spec-annotated for typing-zone yield — a gate the AD-49 Rule text can cite by name. Per AGENTS.md, adding a genuinely **new** gate needs the three-list update and is explicitly out of Task 0's disposition menu ("no new gates") — B would have to land as an extension of the *existing*, already-gated `check-commands.mjs` invocation, not a new script; otherwise it stays a proposal for Ice to weigh against that rule.
No runtime measurement possible (0 live bindings) — pure design-cost tradeoff.

### Code map (B)
`src-tauri/src/commands/segment.rs` (confirm_segment, restore_segment_version, promote_ai_translation, TRANSLATION_ORIGIN* consts, `mod wire`) · `src-tauri/src/core/store/schema.rs` (segment_version DDL, no-CHECK precedent) · `src-tauri/src/core/store/mod.rs` (Store::open/SchemaTooNew) · `src-tauri/src/commands/project/mod.rs::open_work` · `src-tauri/src/core/segment/regroup.rs` (empty-fragment filters) · `src-tauri/tests/segment_contract.rs` + `ipc_contract.rs` · `src/panels/editorPanelState.ts` (replaceEditorSegment, editorPromoteAiTranslationError, promoteAiTranslationToEditor, confirmCurrentSegment) · `src/commands/index.ts` (editor.confirm_segment) · `src/commands/keys.ts` (lacksPrimaryMod, `:510`) · `src/main.ts` (promoteAiTranslationToEditor callers) · `e2e/specs/segment-history-restore.e2e.mjs` · `sprint-change-proposal-2026-09-24b-phieu-quyet.md` (#95/#96/#97/#105) · `sprint-change-proposal-2026-09-24b-no-dung-ten-ice.md:43` · `ARCHITECTURE-SPINE.md` (AD-47, AD-49, §Stack) · `~/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/unicode-normalization-0.1.25/` (NFR15 licence evidence). Exact line numbers are cited per-item above.

## Group C — segment structure (HEAD 968529a)

### L733 — CRLF/whitespace migration of `chapter.source_text` written before Story 6.4

Claim: Chapters imported before Story 6.4 keep raw `\r\n`/untrimmed line breaks on disk; a
migration was left open for Ice. This is a **pointer** item — the real debt lives at
`deferred-work.md:8548-8564` (`## Deferred from: 6-4-chuan-hoa-xuong-dong-va-khoang-trang`).

Status: that item is **already closed** — `deferred-work.md:8564`: `→ KHÔNG LÀM 2026-09-24
(phiếu quyết #8) — chưa phát hành nên Chương nhập trước Story 6.4 chỉ có trên máy dev; bản
dịch neo theo segment nên không hỏng.` Matches `sprint-change-proposal-2026-09-24b-phieu-quyet.md:32-33` (Q8).

Disposition: **already self-closed**, cite `:8564` + phiếu quyết #8. Nothing to build.

Ice question: no.

---

### L1840 — Chapter-level "re-split with retirement warning" (AC8, half two)

Claim: no command re-splits a whole Chapter with a warning about retiring data; blocked on a
`SegmentVersion` concept for AD-5 history.

Status: still unbuilt. `src-tauri/tests/segment_boundary.rs:266` — guard test **renamed**
(now `the_splitter_has_exactly_one_product_call_site_outside_core_segment`, was cited as
`..._exactly_two_product_call_sites`) and now counts source lines matching
`split_source_text` (2, both `commands/segment.rs:45,334`), not "call sites" — same
conclusion holds: no second product call site, no auto re-split feature.
`commands/segment.rs:31-32` doc-comment still states the gap.

New finding: the stated **blocker is resolved**. `SegmentVersionRow`/`read_segment_history`
(`commands/segment.rs:486-560`, Story 2.6/2.7, FR101) already give the AD-5 shape —
`segment_version` rows stay on the retired old id (tombstone, queryable); a fresh segment
starts with zero version rows. Existed since 2026-08-16, predating the 2026-09-23 sweep, which
only re-checked the call-site count, not this blocker.

Disposition: **KHÔNG LÀM within Story 11.5** — building the re-split UI/command is new
user-visible capability, forbidden by Epic 11's own charter. Reassign `Chủ: Ice` to schedule a
future story; record that `SegmentVersionRow` already clears the AD-5 precondition.

Ice question: no (scope call).

---

### L3150 — AC1 "một dải câu" mechanism gap + AC5 "ẩn hoàn toàn ở đầu ra"

Claim: no project doc (`prd.md`/`epics.md`/`EXPERIENCE.md`) specifies the *mechanism*
(Shift+click/drag/Shift+arrow) for selecting a sentence range to excise (FR133). AC5 sub-claim
(`:3176-3179`) says two output-consumption surfaces for hidden-excised-sentences don't exist,
but the 2026-09-23 note under that heading actually re-describes the merge-selection gap
(same root cause as L3814), not AC5 itself — AC5 was **not re-verified** by this pass.

Status: unchanged, twin of L3814. `segmentSelectionState.ts` (Story 4.9 anchor+focus) exists
but its doc-comment scopes it to AI batch translate only, explicitly excluding caret and
`editor.merge_segments`/excise. No range type feeds the excise command or `mergeCurrentSegment`.

Disposition: **Ice question**, same options as L3814 (present together, one answer covers
both):
- **A** — reuse Story 4.9's `segmentSelectionState.ts` anchor/focus surface for both
  excise-range and merge-group; add a multi-id Rust entry point + adapter. Cheap — range math
  and chord handling already built/tested.
- **B** — defer both AC1-range and AC6-group to a UX story that specifies the mechanism from
  scratch. Unbounded cost until that spec exists.

Ice question: yes (shared with L3814).

---

### L3552 — adapter payload-trust split, now 11-vs-1 (was "six")

Claim: after Story 2.6, most `src/config/segment.ts` adapters trust the IPC success payload
(`isIpcError` only); `readSegmentHistory` also validates shape (`isSegmentVersionArray`). The
trust pattern once let a missing Rust `status` column ship silently past 74/74 green frontend
tests (hand-written fixtures).

Status, measured: 12 exported adapters; **11** trust-only (all but `readSegmentHistory`);
**1** validates (`readSegmentHistory:833-858` via `isSegmentVersionArray:394`). Ratio grew
from the ledger's 6-vs-1 as new adapters were added without revisiting the convention.
`src-tauri/tests/ipc_contract.rs` has 0 `SegmentRow`/segment-shape assertions — the wire gate
doesn't currently backstop this drift class.

Disposition: **Ice question** (K item, per `no-dung-ten-ice` classification):
- **A** — raise all 11 to validate shape like `readSegmentHistory` (~11 new type guards).
  Defense-in-depth against the exact bug class that already shipped once.
- **B** — drop validation from `readSegmentHistory` too, trust Rust + `ipc_contract.rs` + e2e.
  Cheaper, but `ipc_contract.rs` doesn't cover segment fields today, so this reopens the same
  risk unless that gate is extended first.

Ice question: yes.

---

### L3814 — AC6 "gộp một NHÓM" (merge a group) — same root cause as L3150

Claim: Ice's #1(a) (2026-08-17) fixed merge to exactly two sentences; `core::segment::
regroup::merge` already accepts an arbitrary slice, but no multi-row selection surface feeds
it and no doc specifies one. Story 2.8 declined to build it; owner moved to "a later story."

Status: unchanged. `src/config/segment.ts:1027` `mergeSegments(segmentId: number)` still one
id. `src/panels/editorPanelState.ts:2649-2665` `mergeCurrentSegment()` reads only
`caretSegmentId.value`. No array/range input anywhere in the merge path.
`segmentSelectionState.ts` (Story 4.9) confirmed scoped to AI batch translate only.

Disposition: **Ice question**, identical to L3150's (Option A/B) — answer once for both.

Ice question: yes (shared with L3150).

---

### L3954 — `segment.ord` goes stale in the webview snapshot after merge/split

Claim: Rust renumbers `ord` 1..N for the whole Chapter on every regroup; `applyRegroup` only
patches touched rows. Harmless only because no webview code reads `segment.ord`.

Status, re-measured: `grep -rn '\.ord\b' src/` outside chapter scope hits only
`ImportPreviewOverlay.vue:1565`, `BilingualImportPreviewOverlay.vue:549`,
`config/project.ts:444`, `ReadingMode.vue:748` — all `chapter`/`entry.ord`, not
`segment.ord`. Zero reads of `segment.ord`. `segmentNavigation.ts:138-141` still documents
"DUYỆT BẰNG CHỈ SỐ MẢNG"; `NavigationSegment` still has no `ord` field. Fully confirmed.

Disposition: **KHÔNG LÀM** — reopen condition ("first webview read of `segment.ord`") hasn't
happened; no pre-emptive fix (Epic 11 AC forbids defensive code for unrealized conditions).
Keep `Chủ: Ice`.

Ice question: no.

---

### L4015 — `split_at` cuts by code point, not grapheme cluster

Claim: can orphan a combining mark at a code-point cut; unreachable today because the only
`cuts` source is the WebKit caret, which never sits mid-cluster. Flagged because `split_at`
is `pub` and a second call site would lose that guarantee.

Status, re-measured: `core/segment/regroup.rs:246` `pub fn split_at(...)` still code-point
based. Exactly one call site, `commands/segment.rs:3389`, unchanged.

Disposition: **KHÔNG LÀM / keep pending** — reopen condition ("a second, non-caret call
site") hasn't happened; no speculative grapheme handling. Keep `Chủ: Ice`.

Ice question: no.

---

### L4436 — `if (s.retiredAt !== null) continue` in `segmentNavigation.ts::buocTu`

Claim: two upstream guards (Rust `WHERE retired_at IS NULL`; `applyRegroup` stripping retired
rows) already make this branch unreachable by real data; no gate cross-checks the two
workspaces.

Status, re-verified: `segmentNavigation.ts:179-181` still carries the §GIỚI HẠN THẬT
doc-comment; `continue` at `:210` in `buocTu` (second copy `:107`, different function).
`commands/segment.rs:908` `WHERE ... retired_at IS NULL ORDER BY ord, id` — guard #1 stands.
`editorPanelState.ts:2451` `applyRegroup` still strips retired rows — guard #2 stands (line
numbers drifted from the ledger's `:1508-1517` but function intact).

Disposition: **KHÔNG LÀM / keep pending** — reopen condition ("a merge-history screen
exposing retired rows to webview") hasn't occurred; Epic 5 (owns navigation) is `done` without
building one. No cross-workspace gate buildable yet. Keep `Chủ: Ice`.

Ice question: no.

---

### L4748 — AC6 regroup-column test covers only the MERGE branch

Claim: `a_row_born_from_regroup_has_every_column_set_on_purpose_not_by_default` exercises only
`merge_segments`; the same `INSERT` also serves `split_segment` — a column dropped for
SPLIT-born rows only wouldn't be caught.

Status: test at `segment_contract.rs:6920` (ledger cited `:6324` — **line drift**, ~600
lines). Confirmed it calls only `merge_segments`, builds one 14-column `SegmentRow` tuple,
asserts against a live re-read. Every `split_segment` use in the same file checks narrower
properties (refusal codes, flags) — none does the same full-column comparison.

Disposition: **fix in Rust (test only)**. Add
`a_row_born_from_split_has_every_column_set_on_purpose_not_by_default` mirroring
`:6920-7010`, driving `split_segment`, asserting the same 14-column tuple via live re-read.
- **Reuse**: `SegmentRow` tuple, the read-all-rows helper, `temp_dir`/`cleanup`,
  `create_work_from_text` setup.
- **Do NOT touch** `write_regroup` (`:3042`) unless the new test goes red — if so, fix the
  split-path `INSERT` (~`:3389-3410`) to match the merge path.
- **Guard reachability**: call `split_segment` (public pure-layer fn), not `write_regroup`
  directly — reaches the seam through its real product caller, same as the existing test.

Ice question: no.

---

### L10053 — image-anchor snap direction on regroup — Ice signed (phiếu quyết #67)

Claim: anchored image inside a merged/split group snaps to land right AFTER the new group;
in-code comment flags this as unsigned by Ice.

Status: comment unchanged — `commands/segment.rs:3062-3067`: "...CHỌN CÓ CHỦ: snap ảnh về
NGAY SAU nhóm MỚI... Đây là một QUYẾT ĐỊNH CHƯA CÓ ICE KÝ RIÊNG..."

Ice decision found: `sprint-change-proposal-2026-09-24b-phieu-quyet.md:217-218`, Q67
(`b8f22f7:9743`) — snap SAU nhóm (a) vs TRƯỚC nhóm (b)? → **giao Story 11.5 — Ice ký hướng
SAU nhóm như mã hiện tại; 11.5 thay dòng "chưa có Ice ký."** Cross-ref
`deferred-work.md:10065`.

Disposition: **fix in Rust**, trivial — edit `commands/segment.rs:3065-3067`, replace the
"chưa ký" sentence with a plain settled-fact statement (no behavior change). ⚠️ Per AGENTS.md
§Code comments: do **not** write "phiếu quyết #67," a date, or decision provenance into the
comment — that belongs in the commit message/spec.

Ice question: no (already answered; implementation-only).

---

### L10067 — `merge_chapter_into_previous` runs `normalize_chapter_ord` before the existence check

Claim: `normalize_chapter_ord(tx)` runs first, before the `chapter_id` existence check; a
nonexistent id returns `Ok(0)`, and `Store::write` commits any `Ok`, so a renumbering side
effect could commit despite the doc-comment's "0 hàng bị chạm." Pre-dates 6.11 (owned by 5.8).

Status, confirmed unchanged: `commands/chapter.rs:763` `normalize_chapter_ord(tx)?;` before
the existence check at `:776-783` (`Ok(0)` on `ton_tai == 0`).

**Cross-item surprise**: same bug in a **second** call site, `move_chapter`
(`chapter.rs:655` normalize before its existence check). A **third** site, chapter-split, gets
it right: `:988` normalize runs *after* its checks (`:965-986`). 2 of 3 callers buggy, 1
correct — a working precedent for the fix shape.

Disposition: **Ice question** (K item, Task 0 presents options):
- **A** — move the existence check before `normalize_chapter_ord(tx)?;` in both
  `merge_chapter_into_previous` and `move_chapter`, matching the already-correct split-path
  order. Small, mechanical, consistent across all three callers; owned by Story 5.8, but the
  change is pure statement reordering, no new behavior on the valid-id path.
- **B** — `KHÔNG LÀM`, reopen when "a caller passes an unvalidated `chapter_id` while
  `chapter.ord` actually has a gap" — today every caller uses a validated id, so the
  coincidence is unobserved. Zero cost, but the doc-comment guarantee doesn't actually hold.

No migration needed (statement ordering, not schema).

Ice question: yes.

---

### Code map (C)

- `src-tauri/src/commands/segment.rs` — `write_regroup` (`:3042`), snap comment
  (`:3062-3067`), `merge_segments` (`:3253`), `split_segment` (`:3389`), `SegmentVersionRow`
  (`:486`), `read_segment_history` (`:558`)
- `src-tauri/src/commands/chapter.rs` — `normalize_chapter_ord` (`:503`),
  `merge_chapter_into_previous` (`:749`), `move_chapter` (`:642`), chapter-split fn (checks
  `:965-986`, normalize call `:988`)
- `src-tauri/src/core/segment/regroup.rs` — `merge` (`:169`), `split_at` (`:246`)
- `src-tauri/tests/segment_contract.rs` — merge-column test (`:6920`)
- `src-tauri/tests/segment_boundary.rs` — splitter call-site count (`:266`)
- `src/config/segment.ts` — 12 adapters (`:512-1075`), `isIpcError` (`:43`),
  `isSegmentVersionArray` (`:394`)
- `src/panels/editorPanelState.ts` — `mergeCurrentSegment` (`:2649`), `applyRegroup` (`:2451`)
- `src/panels/segmentSelectionState.ts` — Story 4.9 anchor/focus (AI batch only)
- `src/panels/segmentNavigation.ts` — `buocTu` (`:200`), `NavigationSegment` (`:26`), array-index
  doc (`:138-141`)
- `deferred-work.md:8548-8564` — real CRLF-migration item behind L733's pointer
- `sprint-change-proposal-2026-09-24b-phieu-quyet.md:32-33` (Q8), `:217-218` (Q67)

## Group D — editor webview (HEAD 968529a)

### L140 — AC4 focus-to-body on reset/reload

Claim: grid `v-for` remounts by `segment.id` on reload; nothing recovers DOM focus ⇒ can fall
to `body` (AD-34 §2).

Status: **partially self-closed.** Fix (`await nextTick(); enterFocus('panel.grid')`) exists in
`src/panels/editorPanelState.ts::switchChapter` (`~:1820`) and `::openChapterById` (`~:1962`).
Missing from three other `resetEditorPanel()` + reload call sites: `src/modes/libraryImport.ts
::finishImportSubmission` (`:410`, load `:492`), `src/modes/libraryChapters.ts::openWorkById`
(`:231`, load `:327`), `::mergeCurrentChapterUp` (`:435`, load `:471`). Both existing sites
still carry "🔴 VẾ NÀY CHƯA CÓ ĐƯỜNG NGHIỆM THU" — no e2e yet.

Ice: #62 (`…phieu-quyet.md:202-203`) — give focus to GridPanel entry-focus on Chapter change,
kèm e2e (happy-dom can't reproduce).

Disposition: **fix in webview.** Copy the `await nextTick(); enterFocus('panel.grid')` tail
into the three missing call sites, right after their load-await. Don't touch
`resetEditorPanel()` itself (state-clear only). Guard: e2e per #62, asserting
`document.activeElement !== body` after Work switch / merge reload; counter-check by removing
the added line.

### L1384 — GridPanel tab strip arrow-key focus

Claim: `@keydown.right/left` dispatches tab-select but never `.focus()`s the new tab button;
with two tabs, second arrow press is a no-op.

Status: **still open**, unchanged. `src/panels/GridPanel.vue:1617-1632` — both tabs dispatch on
click/arrow, no `.focus()` anywhere. `SourcePanel.vue` confirmed gone (merged into GridPanel at
2.5b); defect moved intact.

Ice: none (#62/64/65 silent); plain xếp-nợ reassignment.

Disposition: **fix in webview.** Roving-tabindex: after dispatch in the arrow handlers (or a
`watch(activeTab, …)`), `.focus()` the button matching the new `activeTab`
(`grid-tab-original`/`grid-tab-han-viet`). `check:commands` Kiểm A only inspects `@click`, so
adding focus in `@keydown.*` doesn't trip it — verify with one `check:commands` run. Guard:
vitest mount of `GridPanel.vue`, dispatch `keydown.right` twice from `original`, assert
`activeElement.id` cycles back to `grid-tab-original` — pure DOM `.focus()` behaviour, not a
WebKit-engine claim, so vitest not e2e.

### L2177 — WKWebView click doesn't land the caret

Claim: click-to-place-caret loses selection because `WorkspaceDock`'s `enterFocus` (on grid's
first activating click) races the click handler.

Status: **already self-closed.** `GridPanel.vue::onCellMouseUp` + `ensureCaretNextFrame`
(`~:841-1030`): `cell.focus()`, `placeCaretAtPoint`/`setCaret` fallback, then a two-shot
(rAF + macrotask) re-assert that runs *after* dockview's `enterFocus`. Doc comments (`:902-936`)
correct Story 2.3's wrong diagnosis ("AD-34 giành tiêu điểm") with the measured cause, and
record a real removal counter-check (2026-08-15: remove ⇒ e2e red; restore ⇒ green). Fixed at
`ca33072` ("… chỗ hỏng thật không nằm ở lưới, nó nằm ở ai giành tiêu điểm") — predates this
ledger line's own 2026-09-24 reassignment. `e2e/specs/grid-empty-cell.e2e.mjs:150-165` asserts
`selectionType === 'Caret'`, `rangeCount === 1`, anchor inside cell.

Ice: ledger cites #62, but #62 is actually the *chapter-change* focus decision (L140) — looks
like a mechanical bulk re-tag, not a re-verified citation.

Disposition: **close, already done** (cite `ca33072` + `grid-empty-cell.e2e.mjs:150-165`). Not
covered: caret *visual* rendering before typing — see L2307.

### L2307 — caret visibility (not just selection state) on empty cell

Claim: no test asserts the caret visually renders (`getClientRects`/outline) on a 0px-wide
empty `<span>` before typing.

Status: **still open**, confirmed — `grid-empty-cell.e2e.mjs:150-165` asserts only `id`, `col`,
`isContentEditable`, `selectionType`, `rangeCount`, anchor-inside-cell. No rect/outline
assertion anywhere in the file.

Ice: undecided — not in #62/64/65 nor in `…no-dung-ten-ice.md`. Two options, present both:

- **A — add the e2e visual check.** Extend step ② with `getClientRects()` on the caret range,
  asserting non-zero size after `realClick`, before typing. Small addition to an existing spec;
  may need the same `browser.pause` already used there.
- **B — accept `selectionType === 'Caret'` + `rangeCount === 1` as sufficient**, close with a
  documented reason (valid single-range Selection on a focused contenteditable renders visibly
  in WebKit). Zero code; the literal "nhìn thấy" claim stays inferred, not measured.

No prior attempt at the rect check exists to prefer one — surfaced for Ice, not decided here.

### L2610 / L3510 — err.segment.* and flush-error surfacing (UX-DR30)

Claim: confirm-reject errors and flush errors don't reach the screen with their real
`message_key`; row label shows a fixed string; restore-rejection (3 more branches) inherits the
same half-done surface.

Status: **still half-closed, as ledger's last entry says.** `GridPanel.vue:1563-1564` —
`confirmErrorKey`/`confirmErrorParams` computed and wired, but `:1867-1868` template still
renders the **fixed** string `t('panel.grid.state_refused')`, gated only by
`errorSegmentId === s.id && confirmErrorKey !== null` — the real key's value is never passed to
`t()`. `:1559` comment ("không component nào đọc") is stale — it IS read, just not displayed.
No flush-error computed wired to the same label.

Ice: #64 (`…phieu-quyet.md:208-209`) — extend UX-DR30: confirm error + flush error + 3 restore
branches all show real `message_key` in the row-label column, kèm e2e.

Disposition: **fix in webview** (+confirm Rust already returns `message_key` for 3 restore
branches). Change `:1867-1868` to `t(confirmErrorKey, confirmErrorParams ?? undefined)` instead
of the hardcoded key. Add an analogous computed for the flush error (re-grep near old
`editorPanelState.ts:269-271` — drifted) and merge into the SAME label condition (Ice: "hai món
này phải đóng cùng một bề mặt", not two surfaces). Check the 96px label-column width before
wiring real sentences in (may need truncation). Guard: e2e per #64 for the four cases
(confirm/3×restore/flush) asserting real `vi.json` text; the `computed()` wiring is also a fine
vitest target (mount `GridPanel`, set `editorConfirmError`, assert rendered text).

### L3859 — `resolveSegmentRule`'s `'ornament'` branch is dead by construction

Claim: unreachable, consequence of Story 2.8's "remove retired rows from grid, don't delete
from disk"; not to be silently deleted (still names a live UX-DR19 value) without a decision.

Status: **still open**, confirmed dead-in-practice. `editorSegments.ts:162` — `if
(input.retiredAt !== null) return 'ornament'` is the first (winning) branch, but `:90` documents
`retiredAt` as "null cho mọi segment hôm nay", and `editorPanelState.ts:2443` confirms retired
rows are filtered out of `segments.value` before `segmentRuleInputOf` (`GridPanel.vue:270`)
runs. No caller can hit this branch today.

Ice: #65 (`…phieu-quyet.md:211-212`) — rescinded the retired-dimmed value from UX-DR19
(epics/DESIGN/EXPERIENCE already updated 🔵); 11.5 removes the branch.

Disposition: **fix in webview, TS-guided.** Drop `'ornament'` from `SEGMENT_RULE_VALUES`
(`:69-76`, → 5 values) and the branch (`:162`); re-check whether `retiredAt` stays on
`SegmentRuleInput` (grep other readers first). Shrinking `SegmentRuleValue` makes
`Record<SegmentRuleValue,string>` at `GridPanel.vue:321-328` (`STATE_LABEL_KEYS`) and
`.rule-ornament` CSS (`:2086-2114`) over-specified — let `vue-tsc`'s excess-property check find
every touch point. Do NOT touch the `--color-ornament` design token or its unrelated uses
(`LookupPanel.vue`, `ImportPreviewOverlay.vue`, `ReadingMode.vue`, `dockview-theme.css`) —
same name, different thing. Drop `vi.json`'s now-unused `panel.grid.state_retired`; let
`check:i18n` confirm. Guard: `vue-tsc` is the primary guard (type-level removal).

### L3877 — `sourceCutOffsetOf` × Hán Việt, integration gap

Claim: structural correctness proven (`editorSourceCut.test.ts`) and Hán Việt DOM shape proven
separately (`hanVietCutAnchors.test.ts`), but nothing mounts the real component AND calls
`sourceCutOffsetOf` on its DOM at the caret path with Hán Việt on.

Status: **still open**, confirmed. `hanVietCutAnchors.test.ts` mounts `SourceHanViet` and reads
DOM shape but never calls `sourceCutOffsetOf`. `editorSourceCut.test.ts` calls it only on
hand-built DOM. `editorOriginalOffsets.test.ts` looks related by its header comment but closes a
*different* gap (Story 2.9 AC9 `\r\n` normalization mapping) — don't conflate.

Ice: none; plain xếp-nợ reassignment.

Disposition: **fix in webview, test-only.** Add one `it()` to `hanVietCutAnchors.test.ts`:
mount `SourceHanViet` with Hán Việt on, grab the real rendered cell DOM, call
`sourceCutOffsetOf(cell, node, idx)` directly on it, assert `<rt>` content is skipped from the
count (`SourceHanViet.vue:980`, `user-select: none`). DOM-walk claim on `happy-dom`'s tree, not
a WebKit-rendering claim ⇒ vitest, not e2e.

### L4166 — `GridPanel.vue`'s `PLATFORM` read once, not injectable

Claim: `hasPrimaryModifier` takes platform as a param and is tested both ways, but component
wiring calls `detectIsMac()` once at setup; no test proves the component passes the right value.

Status: **still open**, unchanged. `GridPanel.vue:621` — `const PLATFORM = { isMac:
detectIsMac() }`, used at `:788`. No prop/injection. `editorSourceCutGesture.test.ts` drives
`hasPrimaryModifier` directly, never the component wiring.

Ice: none; item names its own reopen trigger ("first story adding a second Mod-gated mouse
gesture") which has not fired.

Disposition: **KHÔNG LÀM — reopen condition unmet, leave as debt.** Story 11.5 is debt paydown,
not that trigger story; one call site with a demonstrably-correct value doesn't need an
injected, untested seam. Re-affirm owner/condition in the ledger.

### L4201 — stale "NO DOM" doc-comment

Claim: header says "KHÔNG import GIÁ TRỊ NÀO, KHÔNG VUE, KHÔNG DOM" but the file has used
`createTreeWalker`/`Element`/`Selection` since 2.8/2.9; real gate condition is "loadable by
plain Node" (no value `import`s), not "body never touches DOM".

Status: **still open**, unchanged. `editorSegments.ts:26` — comment unchanged.
`createTreeWalker` (`:322`), `caretAtCellStart` (`:434`), `hasPrimaryModifier` (`:488`) all
touch DOM/Selection in their bodies; only `import type` is used, so `check:commands`'s plain-Node
load still passes — comment is stricter than the enforced rule.

Ice: none; xếp-nợ reassignment ("sửa chú thích hoặc tách tệp").

Disposition: **fix in webview, comment-only, in place.** Reword `:26` to state the real
invariant: module is Node-loadable because it imports no *values* (only `import type`); function
*bodies* may touch DOM/Selection. No split needed (smaller fix suffices). No test change — gate
already tests the true condition via `check:commands`' actual `import()` behaviour.

### L4364 — sync navigation during in-flight confirm overwrites caret

Claim: while `⌘Enter` awaits flush+confirm, a sync navigation command isn't blocked by
`confirmInFlight` (which only serializes confirms with each other); on resolution the confirm
still applies `setEditorCaret(following.id)` from its pre-navigation snapshot, overwriting
wherever the user navigated to. No data loss (AD-35 flush still lands) — only caret jump.

Status: **still open**, unchanged. `editorPanelState.ts` — `confirmInFlight` (`:1134`) read/
written only inside the confirm branch (`:817,1081,1085,1110`); no nav command checks it.
`:2555` — sibling `regroupInFlight` was deliberately given reject-and-warn instead of merging
into in-flight — open question is whether nav commands should match.

Ice: none; item itself frames this as an open design question Story 11.5 inherits unresolved.
Two options:

- **A — block navigation while `confirmInFlight` set**, mirroring `regroupInFlight`'s
  reject-and-warn (guard clause per nav command). Small, consistent with precedent. Trade-off:
  nav visibly stalls up to the AD-35 hard cap on every confirm — today it doesn't (race window
  narrow, unnoticed in practice).
- **B — re-resolve `following` at apply time**: skip `setEditorCaret` if caret moved since the
  snapshot. No stall, more surface (needs a "moved since" signal, new race: nav twice before
  confirm resolves).

No prototype either way exists — genuine "two options" item, not decided here.

### L6981 — `Escape` overload leaks into `SegmentHistoryOverlay`/`ShortcutsOverlay` (just-open)

Claim: `editor.clear_source_cuts` is the only bare-`Escape` command; `isTypingZone` doesn't
swallow it for a focused `<button>`. `Escape` on a button inside either overlay both closes it
AND clears source cuts, silently. `historyIsOpen` was never added to `isBlocked()`;
`ShortcutsOverlay` guarded only by `captureIsArmed` (armed-for-chord, not merely "open").

Status: **still open, both bugs, unchanged.** `main.ts:1105-1124`'s `isBlocked()` has ten refs
(none is `historyIsOpen`, exported at `src/panels/segmentHistoryState.ts:39`).
`ShortcutsOverlay` still only `captureIsArmed`-guarded.

Ice: none; item's own text already prescribes the shape (not an open question): narrow
`clearSourceCuts`'s own gate in `main.ts`, do NOT widen `isBlocked` (an open overlay shouldn't
swallow all keyboard input).

Disposition: **fix in webview**, per the item's own shape. Find `main.ts`'s
`clear_source_cuts`/`clearSourceCuts` condition (re-grep — line drifted) and add two guards
directly there: `historyIsOpen.value`, and a ShortcutsOverlay "is open at all" ref (not
`captureIsArmed`; add one following `historyIsOpen`'s `readonly(isOpen)` pattern if the overlay
lacks one). Do NOT add either to the global `isBlocked()` list. Guard: vitest dispatching
`editor.clear_source_cuts` with the refs `true`, asserting source-cut state is NOT cleared;
counter-check by removing the guard (should go red). No e2e needed.

### Code map (D)

- `src/panels/editorPanelState.ts`: `resetEditorPanel:681`, `switchChapter:1657`
  (`enterFocus:1820`), `openChapterById:1887` (`enterFocus:1962`), `confirmInFlight:1134` (cf.
  `regroupInFlight:2555`), `confirmErrorKey:1563-1568`.
- Missing `enterFocus`: `src/modes/libraryImport.ts::finishImportSubmission:410` (load `:492`);
  `src/modes/libraryChapters.ts::openWorkById:231` (load `:327`), `::mergeCurrentChapterUp:435`
  (load `:471`).
- `src/panels/GridPanel.vue`: tab strip `:1617-1632`; `onCellMouseUp`/`ensureCaretNextFrame
  :841-1030` (already fixed); confirm-error wiring `:1559-1568`, row label `:1855-1868`;
  `PLATFORM:621`; `STATE_LABEL_KEYS:321-328`; `.rule-ornament` CSS `:2086-2114`.
- `src/panels/editorSegments.ts`: `SEGMENT_RULE_VALUES:69-76`; comment `:26`;
  `resolveSegmentRule:161-174` (branch `:162`); `sourceCutOffsetOf ~:322`;
  `hasPrimaryModifier/caretAtCellStart:434,488`.
- `src/commands/focus.ts::enter()`/`FOCUS_OWNERS` (no-op-if-already-focused).
- `src/main.ts::isBlocked:1105-1124` (ten refs, no `historyIsOpen`);
  `src/panels/segmentHistoryState.ts::historyIsOpen:39`.
- Tests: `hanVietCutAnchors.test.ts` (no `sourceCutOffsetOf` call), `editorSourceCut.test.ts`,
  `editorOriginalOffsets.test.ts` (different gap), `editorSourceCutGesture.test.ts` (fn-only).
- `e2e/specs/grid-empty-cell.e2e.mjs:150-165` (selection asserted; no visual rect — L2307).
- Ice: `sprint-change-proposal-2026-09-24b-phieu-quyet.md:200-225` (#62 L202-3, #63 L205-6,
  #64 L208-9, #65 L211-2).

