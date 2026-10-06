---
title: 'Story 11.6, lot B — URL import, downloaded images, .docx, Chapter origin and the bilingual preview keep what they promise'
type: evidence
created: '2026-09-28'
status: done
route: 'dispatch'
baseline_revision: '121c17834ce757ddcf407dd914b2e14f915b3d6c'
review_loop_iteration: 0
context:
  - '{project-root}/src-tauri/AGENTS.md'
  - '{project-root}/src/AGENTS.md'
  - '{project-root}/_bmad-output/implementation-artifacts/epic-11-context.md'
relates_to: 6
---

<frozen-after-approval reason="human-owned intent — do not modify unless human renegotiates">

## Intent

**Problem:** On HEAD `121c178`, 38 open `deferred-work.md` items still end in `Chủ: Story 11.6` (Task 0: `11-6-task0-2026-09-28.md`, groups C and D; `L…` numbers below are Task 0's). Sixteen signed Ice decisions (#14, #15, #16, #18, #19, #20, #29, #30, #31, #43, #44, #45, #50, #52, #71, #73) are not in the code. A pasted link list has no cap, no dedupe and no progress. `data:` images count as failed downloads. `.docx` images after a split boundary land in Chapter 1. Tier-2 keys at Chapter k>0 silently write Chapter 1's overrides.

**Approach:** Lot B, the last lot; the story goes `done` after it. Each item gets one Epic 11 disposition below. Every fix gets a guard, and each guard is counter-checked by really removing its seam.

## Boundaries & Constraints

**Always:** Ledger items keep their text; only `→` lines are appended, a stale claim gets a 🔵 fix in place. A wire-shape change updates `ipc_contract.rs` and the direct `invoke()` calls in `e2e/`. A new event follows the `spawn_import_scan` emit + generation-counter shape. A check only a person can make in the real app goes to `Chủ: Epic 11`; a Windows-only check to `Chủ: B7`. `base64` gets its licence read in the downloaded source and its Stack row before `Cargo.toml` changes; that change triggers the full suite once. The orphan sweep deletes only files under `assets/` that no `asset` row names, and runs inside `replace_open_work` before the Work accepts any write.

**Never:** Add a gate, a migration, or any dependency other than `base64`. Build a user-visible capability nobody signed. Split Tier-2 overrides per Chapter (#16). Add a time budget to the image phase (#45). Delete files on the APPEND error path (spec 6.7b §Never). Touch the webview column swap in `bilingualImportPreviewState.ts`. Write decision numbers, dates or provenance into code comments.

## Dispositions

Agent, 2026-09-28 (from Task 0 and a re-read at HEAD; Ice may override any line at approval):
1. ✅ fix, Rust: L9880 (#44: decode `data:` images before the host check, raster only through `is_raster_image_mime`, SVG still refused — this gives L9861's predicate its product caller, closing L9861), L10143 (#20: footnotes and endnotes appended at the end of the Chapter; header, footer and comments stay unread), L10232 (#50: each `.docx` Chapter carries its own slice of `blocks`, at both create and append), L10429 (#52: `forbid_directory` on the outgoing Work's `assets/`, re-`allow` when the same Work reopens), L11256 (`preview_bilingual_import` refuses equal source and target columns with its own `message_key`).
2. ✅ fix, both layers: L9265 (#14: over the cap the whole list is refused with a reason), L9276 + L9889 (#45: one per-item progress event for the page phase and the image phase; the image phase gets a cancel that keeps the Chapters and drops the remaining images), L9285 (#15: dedupe by URL without `#fragment`, host lowercased; the preview shows how many were dropped), L9299 (#16: the two Tier-2 wires refuse at Chapter k>0 with a `message_key`; the StatusBar says only Chapter 1's blocks can be edited), L9717 (#18: "N cần xem" shows from broken links alone; "M sạch" stays gated with its reason; mono screen only, bilingual has no links), L9988 (#19: apply all four origin fields of the selected Chapter to a contiguous `ord` range), L11245 (#31: any `Err` on the selected bilingual candidate reaches the UI with its reason).
3. ✅ fix, webview: L9823 (#43: after import, "N ảnh không tải được" with a way to the domain log, only when `images_failed > 0`), L11197 (#29: the needs-review filter clears when the new candidate has `any_signal_participated === false`, both screens; the old "filter survives candidate change" assertion is replaced), L11233 (#30: `aria-pressed` on both filter chips; `bip-chapters-entry-current` binding and CSS removed).
4. ✅ test-only: L9531 (a `perf_probe_*` with `extract_main_content = true` at N = 20 and 100, `bench-release`), L9571 (#71: the `#[ignore]` seven-sample case compares chosen blocks/tags, not joined text; spec 6.9 §Verification fixed with 🔵), L10518 (#73: a real redirect import asserts `chapter.origin_url` is the requested URL), L10501 (the Rust phase first measures whether `origin.site_name` differs across two encoding candidates for a multi-byte `og:site_name`; equal ⇒ contract cleanup and close; different ⇒ back to Ice before any fix).
5. Self-closed with a pointer: L11126 (#27 = #2, built in `121c17834ce757ddcf407dd914b2e14f915b3d6c`: `source_lang_mismatch` over all new Chapters, shown after an APPEND; neither decision asks for it before confirm).
6. `KHÔNG LÀM` with a reopen condition: L10027 (reopen when a product caller builds a mixed `PipelineShape::Chapters`; the homogeneity test is the tripwire).
7. Reassign: L9834 → `Chủ: B7`.

Ice, 2026-09-28 (spec kept whole at ~3 100 tokens):
8. L9265: the cap is 200 links per paste.
9. L9332 (option A): a cleanup-rule or split-pattern change while the URL screen is open re-renders the Chapter on screen.
10. L9502, L9739 (option A each): `KHÔNG LÀM`. Reopen: L9502 when a real mis-decode is seen to change the block structure; L9739 when a real library passes ~5 000 Chapters or a person reports cursor lag.
11. L9909 + L11158 (option A): one sweep on Work open, in `replace_open_work`, deletes every `assets/` file no `asset` row names; it covers both the `create_work` and the APPEND orphans.
12. L10156 (option a): a nested table becomes its own sibling `TableShape`; its text is no longer dropped.
13. L10341 (option a): after a merge or split, the open Chapter is re-read through `read_open_chapter_segments`, so its assets come back.
14. L10462 (option a): both commands return a distinguishable error for a non-UTF-8 `assets_dir`; the real-path check goes to `Chủ: B7`.
15. L10484 (option 1): a real italic token in `src/tokens/`, used by the empty origin cell.
16. L11211 (option a): a `debug_assert_eq!` on the three vector lengths.
17. L9215, L9466, L9800, L10281: `KHÔNG LÀM`; no real-page corpus will be gathered.
18. L9880: `base64` 0.22.1 becomes a direct dependency.

Ice, 2026-09-29, after the Rust phase measured `origin.site_name` differing across encoding candidates:
19. L10501: the origin draft tracks which fields the user touched; `ChapterOrigin.vue`'s `@commit` sends only those, and an untouched field is sent as `null` so the machine value keeps winning. Both surfaces using the component change with it.

Ice, 2026-09-29, after the Tests phase found the Decision 11 sweep racing an APPEND into a Work that is not open:
20. The APPEND branch that opens its destination itself records that `work_id` in a managed in-progress set for its whole run; `replace_open_work` skips the orphan sweep for a Work in that set, and the next open sweeps it.

</frozen-after-approval>

## Code Map

- `src-tauri/src/commands/project/` -- split in lot A: `mod.rs` (`spawn_import_scan` :623, `cleanup_and_chapters_preview_for` :1681 with the view-side Tier-2 rule :1724, `chapter_detail_for_index` :2310, `Tier2BlockOverridesState` :2686, `ChapterOriginOverridesState` :2759, `replace_open_work` :3089 with the only `allow_directory` :3122); `wire.rs` (command shells: `CreatedWork` fields :335-353, `start_url_import` :1214, `tier2_block_set_kept` :1393, `tier2_block_confirm_range` :1455); `url_import.rs` (`fetch_url_import_items` :227, sequential, no emit; shape builders :261/:294); `work_creation.rs` (docx `first_mut()` wiring :247 create and :694 append, docx image `if i == 0` :1070; `prepare_chapter_images` :1020, fetch loop :1160, `data:` counted failed :1114; assets written :322/:725 before `store.write` :422/:798); `bilingual.rs` (`preview_bilingual_import` :168, `selected_refusal` :208-298, `is_bilingual_table_refusal` :314).
- `src-tauri/src/lib.rs` -- `close_open_work` :1343 (also gets `forbid_directory`).
- `src-tauri/src/core/webimport/` -- `assets.rs` `resolve_absolute_url` :65 (rejects `data:`), `is_raster_image_mime` :144; `extractor.rs` :222.
- `src-tauri/src/core/docx/mod.rs` -- `read_docx` :134 reads `document.xml` and its rels only; `parse_cell` :441, nested `tbl` skipped :451. `core/segment/import.rs` `DocxSidecar` :700. `core/segment/pipeline.rs` `split_bilingual_chapters` :1660.
- `src-tauri/src/commands/chapter.rs` -- `update_chapter_origin` :595, wire :1305. `commands/segment.rs` `to_string_lossy()` :1126, :1547.
- Tests: `docx_contract.rs:571` (saved=1/failed=1 at :598, rewrite to 2 saved), `webimport_contract.rs` (redirects :96…:1253, defense ② :1383, per-link probe :1806, homogeneity :1698), `cleanup_contract.rs` probes :1121-:1981 (none with extraction), `chapter_origin_contract.rs:572`, `asset_contract.rs` `#[cfg(unix)]` :661/:780.
- Webview: `src/importPreviewState.ts` (`openImportPreviewFromUrls` :888 awaits one blocking call, no progress; filter ref :289, resets :703/:826/:937/:1353/:2052; `runImportPreviewReload` :1435, early return :1448); `src/bilingualImportPreviewState.ts` (filter :92, resets :319/:410/:584); `src/ImportPreviewOverlay.vue` chips :1490-1518; `src/BilingualImportPreviewOverlay.vue` chip :505-528, cursor mark :552/:971; `src/config/project.ts` `CreatedWork` :46 (`images_failed` read nowhere); `src/ChapterOrigin.vue` placeholder :67-100/:154; `src/panels/editorPanelState.ts` `applyRegroup` :2564, `chapterAssets` :104; `src/modes/LibraryMode.vue` post-import lines :1461.

## Tasks & Acceptance

**Execution:**
- [x] `src-tauri/**` -- Dispositions 1-2 Rust halves, Decisions 8-18 Rust halves, the L10501 measurement, `base64` licence and Stack row first -- Rust phase.
- [x] `src/**`, `vi.json`, `tests/frontend/*`, `e2e/specs/*` -- Dispositions 2-3 and Decisions 9, 13, 15, 19 webview halves -- Webview phase.
- [x] Disposition 4 and a real-removal counter-check for every new guard; the full suite once (`Cargo.toml`, and `lib.rs` if registration changes) -- Tests phase.
- [x] `deferred-work.md` -- one `→` disposition for each of the 38 items -- Ledger phase.

**Acceptance Criteria:**
- Given lot B is done, when the 38 item lines are read, then each ends in one `→` disposition and `npm run check:debt-owner` is green.
- Given a pasted list with the same URL twice (differing only in `#fragment` or host case), when it loads, then one Chapter is created and the preview reports one duplicate dropped; given 201 links, then nothing is fetched and the reason is shown; given 200, then all load.
- Given the URL screen open on Chapter 2, when a cleanup rule is added, then Chapter 2 re-renders with the rule applied.
- Given a 3-link import with images, when it runs, then a progress event fires per page and per image; given cancel during the image phase, then the Chapters are kept and no further image is fetched.
- Given a page with a `data:image/png;base64` image and a `data:image/svg+xml` one, when imported, then the PNG is saved and the SVG is refused.
- Given the cursor on Chapter 2 of a URL preview, when `Space` is pressed, then the Tier-2 wire refuses with its `message_key` and Chapter 1's overrides are unchanged.
- Given a `.docx` split into two Chapters with one image in each, when imported, then `images_saved == 2` and each image anchors in its own Chapter; given a footnote, then its text ends the Chapter; given a header, then it is absent.
- Given an `assets/` file no `asset` row names, when its Work opens, then the file is gone and every named file remains.
- Given a `.docx` cell holding a nested table, when imported, then the nested table's text is present.
- Given an image-bearing Chapter open, when two segments merge, then the grid still shows the image.
- Given Work A then Work B opened, when an `asset://` URL of A is requested, then it fails; given A reopened, then it succeeds.
- Given a range of five Chapters, when origin range-apply runs, then all four fields match the source Chapter and the Chapter outside the range is unchanged.
- Given a bilingual preview with equal columns at the wire, or a non-table error on the selected candidate, when it returns, then a reason reaches the UI.
- Given each new guard, when its seam is really removed, then it goes red for that reason.

## Implementation Notes

- Progress/cancel (L9276+L9889): every layer in the `create_work`/`append_chapters_to_work`/`fetch_url_import_items` chain got a parallel `_with_progress` function that does the real work, with the original name becoming a thin no-op wrapper — this kept ~80 existing test call sites unchanged. A future entry point needing progress/cancel should follow the same pattern, not a third parallel set of names.
- `withImportProgressListener()` (`importPreviewState.ts`) must call the real IPC (`run()`) synchronously first, before the `await import('@tauri-apps/api/event')` dynamic import — the reverse order delayed the real call by a macrotask and silently hung a mocked test with no assertion failure.
- L10501's own suggested measurement (does `origin.site_name` differ across encoding candidates for a multi-byte `og:site_name`?) came back yes, real mojibake — this flipped the item from "back to Ice" to a real fix (Decision 19): the origin draft now tracks touched-vs-untouched per field, sent as `string | null`, not a frozen four-string snapshot.
- Decision 20 (APPEND/`replace_open_work` race) was not in the spec's original Dispositions — the Tests phase found that an in-flight APPEND into a Work that isn't the open editor Work can race a concurrent `replace_open_work`'s orphan sweep. Fixed with `AppendInProgressState`, unit-tested; the real two-concurrent-command scenario is unverified (no `tauri::AppHandle` in Rust tests) and is now one line item in the combined Epic-11 real-use-pass debt entry together with L10429/L9909/L11158.
- L10232's counter-check gave a false negative on the first attempt: patching the consumer lookup stayed green because the fixture's two chapters share the same *local* block index for their image. The real seam is the producer (`ChapterDocxImage.chapter_index`); a future fixture change could reopen this blind spot (new debt item, `Chủ: Murat`).
- `base64 = "=0.22.1"` added as a direct dependency for L9880's `data:` URI decoding: 0 new packages (already transitive via `docx-rs`/`reqwest`/`tauri`), licence read and Stack row recorded before the `Cargo.toml` change.
- Four items (L9215, L9466, L9800, L10281) share one `KHÔNG LÀM` reason (Decision 17): no real story-reading-site/blog/forum corpus exists in this offline repo beyond the 7 `epochtimes.com` samples from bàn đo 6.1, and none was fabricated.
- L9502 and L9739 both closed `KHÔNG LÀM` under Decision 10 with explicit reopen conditions (a real mis-decode changing block structure; a library exceeding ~5 000 Chapters or a reported lag) rather than being fixed speculatively.
- Nightly e2e (`36494988256`, 2026-09-28) had one red case, `editor-typing-flush.e2e.mjs:218` — a StatusBar/tooltip timing race unrelated to any file this lot touches; recorded per AGENTS.md, not investigated further.
- Ledger: the 38 items end as 26 `✅ ĐÃ ĐÓNG`, 1 self-closed with a pointer to `121c178`, 7 `KHÔNG LÀM`, 3 `🟡` reassigned to `Chủ: Epic 11` (AppHandle wiring of the `asset://` revocation, the orphan sweep and the Decision 20 skip), 1 reassigned to `Chủ: B7`. New items: one `Chủ: Epic 11` real-use check, one `Chủ: B7` Windows path check, one `Chủ: Murat` fixture blind spot, closed in review.

## Spec Change Log

## Review Triage Log

- blind: nested `w:tbl` blocks are absorbed after the whole body loop (`core/docx/mod.rs` :197), so their text moves to the end of the document, into the last Chapter after a split — medium, patch: absorb each nested table right after its parent (Decision 12 says sibling).
- blind: `should_skip_orphan_sweep` releases the lock before the sweep runs, so an APPEND registering in between still loses its files — medium, patch: hold the `AppendInProgressState` lock across check and sweep.
- edge: an unmanaged `AppendInProgressState` makes the skip fail open — low, patch: a missing state skips the sweep (same site as the previous row).
- edge + blind: `refreshChapterAssetsAfterRegroup` sets `chapterAssets = []` and `assetsDir = ''` when the re-read fails, so images vanish with no error — medium, patch: keep the previous values and log the error.
- verification-gap: the APPEND path's `distribute_docx_blocks_across_chapters` call (`work_creation.rs` :744) runs in no test; reverting it stays green — medium, patch: an APPEND counterpart of the `.docx` split-boundary image test.
- verification-gap (other) + blind: both `.docx` split fixtures put the two images at the same local block index, so a consumer-side regression stays green — low, patch: distinct local indices in the unit and contract fixtures.
- edge: `decode_data_uri_image` rejects `image/png; base64` (space after `;`) — low, patch: trim each header parameter.
- blind: `setChapterOriginApplyThroughOrd` accepts any integer, so a typo silently widens the overwrite range — low, patch: accept only an `ord` between the current Chapter's and the last Chapter's.
- blind: the spine's new `base64` Stack row carries story and decision provenance, unlike its sibling rows — low, patch: drop it.
- verification-gap: the orphan sweep and Decision 20 wiring in `replace_open_work` run in no test — medium, defer: already the owned `Chủ: Epic 11` real-use item of this lot; no `AppHandle` test harness exists.
- verification-gap: `forbid_directory` wiring in `replace_open_work`/`close_open_work` runs in no test — medium, defer: same `Chủ: Epic 11` item.
- edge: `data:` payload has no size cap — false: it sits inside a page already capped by `fetcher.rs::MAX_RESPONSE_BYTES` (20 MiB), and decoding shrinks it.
- edge: nested-table recursion has no depth limit — low, rejected: needs a crafted `.docx` thousands of tables deep; a depth guard is new complexity.
- edge: `distribute_docx_blocks_across_chapters` has no runtime length check — false: a drifted boundary fails `compute_anchor`'s own self-check and the image counts as failed, not misfiled.
- edge + blind: the bilingual length guard is `debug_assert_eq!`, silent in release — low, rejected: Decision 16 chose exactly this.
- blind: `ImageDownloadGeneration` is one counter for every import — low, rejected: one pending-import slot and the webview's `confirming` gate keep two image passes from overlapping.
- blind: the same facts appear in the handoff file, Implementation Notes and ledger lines — low, rejected: the handoff file is working notes; ledger closures carry their own evidence by convention.
- blind: `sweep_orphaned_asset_files` skips symlinks — false: the app never writes symlinks into `assets/`, and not deleting is the safe direction.
- blind: the range-apply `UPDATE` has no Work filter — false: `chapter` lives in the Work's own `project.db`; the connection is that Work.
- blind: the bilingual `debug_assert` closure note omits the release gap — low, rejected: the ledger line names `debug_assert_eq!`, which states it.

## Verification

**Commands:**
- `npm run build && cargo test --test <each changed contract test>` -- expected: green.
- `npx vitest run <changed test files>` -- expected: green.
- `npm run check:debt-owner && npm run check:i18n && npm run check:commands && npm run check:tokens` -- expected: green.

## Acceptance criteria from epics.md

Source: `epics.md` §Story 11.6 (v6, nay ở archive-v6).

As a chủ dự án,
I want nợ nhóm đường nhập và Library (Epic 5, Epic 6) được đóng hoặc quyết dứt điểm,
So that dữ liệu vào TM là dữ liệu đã nhập đúng.

Thừa kế AC chung của epic (xem Notes của `epic-tra-no-nen.md`).

**Acceptance Criteria:**

**Given** mọi mục mang `Chủ: Story 11.6` trong `deferred-work.md`
**When** story hoàn tất
**Then** thoả AC chung của Epic 11; AC riêng rút từ các mục lúc `create-story`
