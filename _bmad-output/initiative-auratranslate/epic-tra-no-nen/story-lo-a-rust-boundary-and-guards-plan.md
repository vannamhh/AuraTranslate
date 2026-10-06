---
ticket: 8
title: 'Story 11.8, lot A — Rust boundary faults between Epic 11 stories, and guards that reach the wiring'
type: 'bugfix'
created: '2026-09-30'
status: done
route: 'dispatch'
baseline_revision: '0736243c8e8946760e1d8c0f36e2f63b9311542d'
review_loop_iteration: 0
context:
  - '{project-root}/src-tauri/AGENTS.md'
  - '{project-root}/_bmad-output/implementation-artifacts/epic-11-context.md'
---

<frozen-after-approval reason="human-owned intent — do not modify unless human renegotiates">

## Intent

**Problem:** On HEAD `8c801ef`, 11 of the 21 items ending `Chủ: Story 11.8` are Rust faults or guards left where two Epic 11 stories meet (Task 0: `11-8-task0-2026-09-30.md` §Group A). One bad `confirm_segment` call locks a whole Work. APPEND into the open Work races the orphan-image sweep. A merge forgets `last_chapter_id`. Flushes can touch retired segments. Several guards call the patched function, not the `#[tauri::command]` shell, or EXPLAIN hand-copied SQL.

**Approach:** Lot A goes first; lot B (webview, e2e, CI) follows and closes the story, because AC ③ needs the e2e run on the story's last commit. Every item below gets one disposition. Every new guard is counter-checked by really removing its seam.

## Boundaries & Constraints

**Always:** Ledger items keep their text; only `→` lines are appended, and a stale claim gets 🔵 in place. AD-47: the `confirm_segment` keep branch still sets status and origin in one `UPDATE`. A shell goes generic over `R: tauri::Runtime` only when a test must drive it. Its text pins (`config_invariants.rs`, `ipc_contract.rs`, `ipc_argument_contract.rs`) change in the same commit. A real-app check goes to `Chủ: Epic 11`, a Windows-only one to `Chủ: B7`.

**Never:** A migration, a new dependency, or a new gate. Changing `glossary_candidate` DDL. Changing `split_chapter_into_segments`'s locking. Writing `self` for an unknown origin. Rewriting stale comments beyond the sites named here.

## Dispositions

Agent, 2026-09-30 (Ice may override any line at approval):
1. F-R-1 fix: in the keep branch, an `origin_at_load` outside `TRANSLATION_ORIGINS` returns a new named error, writes nothing, and the Work still opens afterwards. This is fixed by the story's own AC ② ("từ chối và không ghi gì"), so the "keep disk origin" option is out. The key and its `vi.json` string are new.
2. F-R-2 fix: one `AppendInProgressGuard` is built before the `already_open` split and lives across both branches. Its presence is observed from inside the call through the image-progress event, IF an offline pending source emits one; measure that first. Otherwise the guard is the weaker "set empty after every path" case and the item stays 🟡.
3. F-R-4 fix: the merge calls `set_open_chapter`, guarded by a 3-chapter case (the 2-chapter case cannot see the bug). The premise comment of the `project_contract.rs:2485` case is corrected.
4. F-D-9 + F-R-8: the 9 comment sites are rewritten to what is true (the wire takes a concrete `AppHandle`), and the `open_work` doc at `mod.rs:3016` says `Store::open` may already have set WAL, backed up and migrated. No pre-open peek: whether an old DB has the column is unverified.
5. Wire harness: new `tests/project_wire.rs` on `mock_builder()`. `wire::open_work` (with `replace_open_work`) and `wire::confirm_import_with_encoding` go generic. Cases: orphan sweep at open (L9974), asset scope revoked and re-granted across a Work switch (L10502), reindex after APPEND into the open Work (L7443 seam), and item 2. Ledger: L9974 and L10502 close; L12593 shrinks to the true-concurrency hand check (🟡 `Chủ: Epic 11`); L7443 gets a 🔵 premise fix.
6. F-D-8: `wire::promote_ai_translation` goes generic, with four shell cases: write, retired, `force` forwarded, and no `OpenWorkState` managed.
7. F-D-1 (a): `boundary_scan_contract.rs` enumerates `tests/*_boundary.rs` (19, with a floor) on `code_lines` and asserts each declares the shared module, calls a population helper and `assert_population_floor`, and has no bare `starts_with(<CONST>)`. A seeded-regression case proves the predicate can fail.
8. F-D-1 (b): the three branch SQL strings of `core/dict/query.rs` become `pub` builders, and the EXPLAIN case runs them. Check first that the plan with bound parameters matches the literal plan.
9. F-R-3, measure first (Task 0 §F-R-3 recipe, release, signed binary, 20 runs). If p95 lock-hold and wait are < 100 ms at K=500, C=300 ⇒ `KHÔNG LÀM` with the numbers and a reopen condition. If ≥ 100 ms ⇒ stop and bring the numbers to Ice; no fix in this lot. The APPEND network hold is structural and is recorded only.
10. F-R-5 + F-A-4: `approve_candidate` maps a legacy 0 to `None` (twin case: 37 stays 37). `chapter_span_count` and `bilingual.rs` `column_count` ⇒ `KHÔNG LÀM` with Task 0's reopen conditions.
11. F-R-6: `AND retired_at IS NULL` in both the `unconfirm_edited_segments` membership count and the `save_segment_targets` `UPDATE`, so the batch is rejected whole as `segment.unknown_ids`.
12. F-A-3: `KHÔNG LÀM` for the `matching_close_brace` half: same name, one scans Rust and one scans TypeScript. L12692 gets a 🟡 line; lot B closes its F-W-9 half.

Ice, 2026-09-30:
13. Push run `36702088358` on HEAD `8c801ef` is red on `check (windows-2025)` only (`bindingsEpochWiring.test.ts`, two cases, from `7dbcf23`). It becomes a new ledger item `Chủ: Story 11.8`, fixed in lot B. Until then, lot A's CI reads cite that run id as the reason for the red Windows half.
14. The spec stays whole at ~2.5k tokens; per-item detail lives in Task 0.
15. Found by the Tests phase: Tauri's `forbid_directory` has no inverse and outranks every allow, so A → B → A leaves A's `assets/` forbidden until restart. Ice chose: `replace_open_work` stops forbidding the outgoing Work's `assets/`. Every Work opened in a session stays readable over `asset://` until exit. The ignored guard `reopening_a_work_after_leaving_it_grants_its_assets_again` loses its `#[ignore]` and goes green. The revoke case is rewritten to what is now true. L10502 is closed with this.
15b. Decision 15 was first asked without mentioning that decision sheet #52 (2026-09-24) already signed (a): revoke the outgoing Work on a switch, one Work at a time per AD-23. Told of #52, Ice chose to keep decision 15 as an interim that does not yet carry #52 out. A new ledger item, `Chủ: Winston`, restores #52 by serving Work images through a URI scheme bound to `OpenWorkState`.
16. F-R-3 measured p95 lock-hold ≈ 1.3 s at K=500, C=300 (phases notes, part 2). Ice chose to fix it in lot A: `glossary_pending_candidates` holds `OpenWorkState` only while it reads the DB, and releases it before the Han-Viet lookup and the chapter-span count. `split_chapter_into_segments` is left alone. The proof is the same measurement re-run plus a MockRuntime case where `save_segment_targets` is not kept waiting while the scan runs.

</frozen-after-approval>

## Code Map

- Task 0 per item (file:line at HEAD, guard design, counter-check, risks): `11-8-task0-2026-09-30.md` §Group A. The phase agents read their item there, not here.
- Pattern to copy: `src-tauri/tests/ai_translate_wire.rs` (`Harness`, `mock_builder` :120); generic shells `commands/aitranslate.rs:796-797`.
- Shells to make generic: `commands/project/wire.rs` `confirm_import_with_encoding` :785 (APPEND split :892-943), `open_work` :1208; `commands/project/mod.rs` `replace_open_work` :3314 (sweep :3358-3379, scope :3346-3352 / :3409-3415); `commands/segment.rs` shell `promote_ai_translation` :3865.
- Signature pins: `tests/config_invariants.rs` :1079, :1115, :1195, plain-shell census :1522-1650; `tests/ipc_contract.rs` :2165-2185; `tests/ipc_argument_contract.rs` :151-153.
- Fix sites: `commands/segment.rs` :2519-2530 (keep branch), :1972-1976 and the membership count in `unconfirm_edited_segments` (~:2881); `commands/chapter.rs` :951-955; `core/glossary/candidate_store.rs` :345; `core/dict/query.rs` :200-207, :265-273, :288-299; `core/i18n/mod.rs` `message_keys!` (neighbour `StoreUnknownTranslationOrigin` :144); `src/i18n/vi.json`.
- Tests: `segment_contract.rs` (:5902 misses the keep branch; saves from :2720), `project_contract.rs` :2485, `glossary_commands_contract.rs`, `dict_sources.rs` :5298-5366 (read with offset/limit), `boundary_scan_contract.rs`.

## Tasks & Acceptance

**Execution:**
- [x] `src-tauri/src/**`, `src/i18n/vi.json` -- Dispositions 1-6, 8, 10, 11: the generic shells first (5, 6), then the fixes -- Rust phase.
- [x] `src-tauri/tests/**` -- the guards of Dispositions 1-8, 10, 11; one real removal per guard; the F-R-3 measurement (throwaway probe, not committed); the full suite once, because shell signatures and the pins change -- Tests phase.
- [x] `deferred-work.md` -- one `→` line for each of the 11 items and for L7443, L9974, L10502, L12593 -- Ledger phase.

**Acceptance Criteria:**
- Given a draft segment whose on-disk origin is set and whose text is unchanged, when `confirm_segment` receives an origin outside the FR117 catalogue, then it errors, the row is unchanged, and `open_work` still opens the Work.
- Given a Work with 3 Chapters and Chapter 3 open, when Chapter 3 is merged into Chapter 2 and the Work reopened, then Chapter 2 opens.
- Given a batch holding one live and one retired segment, when it is flushed, then it is rejected whole, and the live segment keeps its text and `confirmed` status.
- Given each new guard, when its seam is really removed in production code, then only that guard's target goes red, for that reason.
- Given lot A is done, when the 11 item lines are read, then each ends in one `→` disposition (L12692 🟡 for lot B) and `npm run check:debt-owner` is green.

## Implementation Notes

Phase working notes: [11-8-lo-a-phases-2026-09-30.md](11-8-lo-a-phases-2026-09-30.md).
- Only the `aitranslate` wires were generic; `open_work`, `confirm_import_with_encoding`, `promote_ai_translation`, `glossary_pending_candidates` and their private helpers now take `AppHandle<R>`, and the text pins in `config_invariants.rs`, `ipc_contract.rs`, `project_contract.rs` follow.
- Tauri 2.11.5 `forbid_directory` has no inverse (`scope/fs.rs` has no remove), so the old revoke broke A → B → A; decisions 15/15b: the scope grows per session until the URI-scheme item (`Chủ: Winston`) restores #52. `close_open_work` still forbids, on `RunEvent::Exit` only.
- F-R-3 before/after in one probe (release, K=500, C=300×4,000 chars, no dict layers): old lock-hold p95 966 ms, new waiter p95 6 ms. The APPEND hold over image download is structural and stays `Chủ: Ice`.
- An offline `.docx` with an embedded image does emit the image-progress event on the APPEND path, so the F-R-2 guard observes the guard from inside the call.
- `rusqlite` refuses `EXPLAIN` with unbound `?N`; the plan case binds values, and the bound plan text was measured equal to the literal plan for all three branches.
- `boundary_scan_contract` names one exemption, `naming_boundary.rs` (`POPULATION_HELPER_EXEMPT`, its own comment-blanking walkers).
- Known gaps, in the ledger: the second `char_idx` arm of the 2-char plan and the second WHERE copy at `query.rs:488` are not guarded.
- `ITEM_FLOOR` in `check-debt-owner.mjs` raised 604 → 643 as the gate demanded when the ledger reached 756 items.

## Spec Change Log

## Review Triage Log

Loop 0 (Blind B1-B13, Edge E1-E12, Verification-gap: no gaps + V1-V2):
- B1/E3 asset scope grows per session, AD-23 reversed in code — false: AD-23 allows the whole Library root as dynamic scope; per-session growth is Ice decisions 15/15b with `Chủ: Winston` for #52.
- B2 keep branch trusts the webview's origin over the disk value — false for this change: the keep branch returns the load-time origin by design (a second confirm must not return `self`); only the unknown-value check is new.
- E5 empty `origin_at_load` refused — false: `TRANSLATION_ORIGIN_NONE` is `""`, inside `TRANSLATION_ORIGINS`.
- B3 `approve_candidate` keeps 0 as a sentinel in the candidate type — false: the candidate column is `NOT NULL DEFAULT 0` and its DDL is out of scope (Never); scans only write counts ≥ threshold.
- B4/E1/E2 merge returns `Err` if `set_open_chapter` fails after commit — low, rejected: needs a write failure right after a committed write; the fix adds a branch.
- B5 APPEND guard untested on the not-open branch — false: that branch held the guard across its `replace_open_work` before this change too; behaviour unchanged.
- E4 guard alive until unwind on panic/`?` — false: RAII guard, dropped on every exit path.
- B6 `project_wire.rs` `Harness::drop` removes dirs while stores are open — low, rejected: errors are ignored, so it leaks temp dirs on Windows at most; the fix is more than a direct correction.
- B7/E6/V2 `glossary_wire.rs` timing-based — low, rejected: the real removal went red with a 2.36 s wait vs a 0.6 s bar; a barrier needs a production seam.
- B8 plan test binds hand-picked limits; builders widened to `pub` — low, rejected: the plan does not depend on the limit value; `pub` builders are Disposition 8.
- B9/E7 boundary predicate evadable by nested parens or split lines — low, rejected: heuristic scan; a parser adds complexity. `BOUNDARY_FILE_FLOOR = 17` is ceil(0.85 × 19), not a guess.
- B10 stale test comments after the generic change — low, patched: `chapter_origin_contract.rs` now names `preview_import_encoding_from_text` as the concrete shell; `ipc_contract.rs` mid-sentence break joined. The `ipc_contract.rs` sites describe the preview wires, still concrete: true.
- B11/E10 new key only in `vi.json`; no webview mapping — false: `vi.json` is the only locale; confirm refusals reach the screen through `tError()`.
- B12/E8 signature lookup may match a same-name fn in another module — false: the helper's contract makes callers narrow `src` first; the targets pass on the generic shells.
- B13 `close_open_work` forbid at exit is a no-op — low, rejected: harmless, its comment now says so.
- E9 retired id reported as `segment.unknown_ids` — false: Disposition 11 by design.
- E11 deleted `assets_dir_to_forbid` tests; close+reopen in-process — false: `close_open_work` runs only on `RunEvent::Exit`.
- E12 unconfirm ordering could demote the live segment — false: the membership check runs before the lowering `UPDATE` in the same transaction; the flush case asserts the live segment stays `confirmed`.
- V1 `save_chapter_position` pair check lacks `retired_at IS NULL` — low, pre-existing: deferred (ledger, `Chủ: Amelia`).

## Verification

**Commands:**
- `npm run build`, then one `cargo test --test <target>` per changed target (`segment_contract`, `project_contract`, `project_wire`, `ai_translate_wire`, `glossary_commands_contract`, `dict_sources`, `boundary_scan_contract`, `config_invariants`, `ipc_contract`, `ipc_argument_contract`) -- green.
- The full suite once in the Tests phase; `npm run check:i18n`, `npm run check:debt-owner` -- green.

## Acceptance criteria from epics.md

Source: `epics.md` §Story 11.8 (v6, nay ở archive-v6).

As a chủ dự án,
I want các lỗi mà hai story Epic 11 cùng tạo ra ở chỗ giao nhau — nơi không phiên nào thấy cả hai phía — được sửa hoặc quyết dứt điểm,
So that Epic 7 dựng trên một nền có nightly e2e xanh và guard đỏ được khi gỡ seam.

Thừa kế AC chung của epic (xem Notes của `epic-tra-no-nen.md`).

**Acceptance Criteria:**

**Given** mọi mục mang `Chủ: Story 11.8` trong `deferred-work.md`
**When** story hoàn tất
**Then** thoả AC chung của Epic 11; AC riêng rút từ các mục lúc `create-story`

**Given** một lượt Quét lại, Chọn thư mục hay Gỡ mồ côi đang chạy
**When** người dùng rời rồi quay lại Library trước khi lượt đó xong
**Then** nút tương ứng dùng lại được ngay, không phải khởi động lại app

**Given** `confirm_segment` nhận một xuất xứ lúc nạp nằm ngoài danh mục FR117
**When** lệnh chạy
**Then** lệnh từ chối và không ghi gì vào `segment`, nên Tác phẩm vẫn mở được ở lần sau

**Given** commit cuối của story
**When** story lên `review`
**Then** một lượt e2e `workflow_dispatch` trên commit đó xanh, trong đó `attribution-focus` thật sự đo AC11 của Story 1.19; nếu đỏ, một dòng lý do nêu run id nằm trong spec
