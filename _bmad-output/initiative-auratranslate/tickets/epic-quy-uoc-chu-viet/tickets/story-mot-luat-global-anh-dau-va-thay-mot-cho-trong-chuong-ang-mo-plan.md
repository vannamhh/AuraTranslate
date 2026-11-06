---
title: 'Một luật Global đánh dấu và thay một chỗ trong Chương đang mở'
type: 'feature'
ticket: '1'
created: '2026-11-05'
status: 'done'
baseline_revision: 'bfd6603d0a828f41a8031d1058f757e3e902722b'
route: 'full'
route_source: 'auto'
risk: 'high'
review: 'quick'
review_source: 'pinned'
lenses_ran: ['quick']
review_loop_iteration: 0
context:
  - '{project-root}/_bmad-output/initiative-auratranslate/architecture-auratranslate/architecture-auratranslate.md §AD-67 (lines ~1665–1742), §AD-53 rules 2–6'
  - '{project-root}/_bmad-output/initiative-auratranslate/ux-auratranslate/EXPERIENCE.md §Quy ước chữ Việt — FR159 (Component Patterns, State Patterns, Interaction Primitives, Khoá i18n mới)'
  - '{project-root}/_bmad-output/initiative-auratranslate/ux-auratranslate/mockups/quy-uoc-chu-viet.html (dấu và dải only)'
---

<frozen-after-approval reason="human-owned intent — do not modify unless human renegotiates">

## Intent

**Problem:** Người dịch không có cách nào bắt các cụm chữ Việt mình muốn tránh (*đảm bảo → bảo đảm*) trong bản dịch mà không gọi AI (FR159). Story này là tracer bullet của Epic 16: một luật Global đi trọn từ kho tới dấu trong ô và lượt nhận thay một chỗ.

**Approach:** Kho `convention_rule` ở `global.db`; bộ khớp thuần trong `core/convention/` (aho-corasick trên NFC + chữ thường, khoảng UTF-16 trên chuỗi đã lưu); lệnh quét Chương đang mở (async, ngoài main thread) trả chỗ khớp và bốn số đếm; dấu chấm trong ô, dải `convention` mở bằng bấm dấu hoặc F8; nhận thay một chỗ và hoàn lại qua `write_convention_target` theo AD-67 mục 7–8.

**Decision (2026-11-05, Ice):** luật được thêm qua phần nhỏ nhất của lớp phủ Quản lý quy ước — `Mod+Alt+A` và mục *Quản lý quy ước chữ Việt* trong `Công cụ ▾`, chỉ tầng Toàn cục, danh sách luật + hàng thêm luật — theo khuôn lớp phủ Quản lý Glossary; 16.3 mở rộng chính lớp phủ này (tầng, sửa, xoá, tắt, thẻ Không phải lỗi). Không miễn trừ `NO_FRONTEND_CALLER`. Thêm luật làm Chương đang mở quét lại.

## Boundaries & Constraints

**Always:**
- AD-67 mục 1–8, 11, 12 (a)(b)(c)(f)(h), 12b phần dải và `marks.next/prev`; AD-47 ③ hàng FR159; AD-53 mục 4–6 cho phép kiểm, vòng đời dấu và dời khoảng.
- Webview không gửi cụm thay; Rust tính cụm thay theo kiểu hoa và kiểm khoá luật còn hiệu lực trong giao dịch ghi.
- Kết quả quét luôn mang `ignored`, `glossary_suppressed`, `overlapped`, `not_scanned` (Prevents 6); `ignored` = 0 cho tới 16.5.
- Câu đã ký: nhận thay giữ `confirmed`, chèn đúng một `segment_version` và một cặp TM (`insert_pair`) mang văn bản mới; câu nháp đi như `write_non_user_target`.
- `aho-corasick` thành dep trực tiếp chỉ sau khi đọc licence trong `~/.cargo/registry/src/…/aho-corasick-1.1.4/` và ghi hàng Stack (NFR15).
- Tạo mục nợ `Chủ: Epic 16` trong `deferred-work.md`: lượt dùng thật của Ice (epic Done when 4) và phép đo "không frame nào vượt 50 ms" khi quét Chương (AD-67 mục 11), vì cần app thật.

**Never:**
- Quét chữ đang gõ; IPC mới trên đường gõ; quét lại sau flush (16.4); quét gợi ý AI (16.4).
- Tầng Tác phẩm, kind AD-18 `convention_rule`, sửa/xoá/tắt luật, ô tìm, thẻ Không phải lỗi của màn Quản lý (16.3); Thay cả Chương (16.2); "Không phải lỗi" và `CONVENTION_IGNORE` (16.5); xuất nhập (16.6). Dải không hiện nút của các story đó.
- Import `core::ai` từ `core/convention/` hay ngược lại; đặt phép kiểm Glossary trong `core/convention/`.
- Đổi chữ ký ghi nhớ, lượt ghi, error code hay hành vi dải của Proofreader, trừ việc F8/Shift+F8 mặc định chuyển sang `marks.next/prev` (id `ai.proofread.next/prev` và phím người dùng đã gán giữ nguyên).
- Dải `convention` tự mở theo vị trí con trỏ.

## I/O & Edge-Case Matrix

| Scenario | Input / State | Expected Output / Behavior | Error Handling |
|---|---|---|---|
| Khớp đầu câu | luật `đảm bảo→bảo đảm`, câu `Đảm bảo an toàn.` | một chỗ `[0,7)`, cụm thay hiện `Bảo đảm` | — |
| Kiểu hoa | `ĐẢM BẢO` / `đảm bảo` / `đẢm Bảo` | `BẢO ĐẢM` / `bảo đảm` / cách viết trong luật | — |
| i/y | luật `kỷ`, văn bản `kỉ` | 0 chỗ | — |
| Ranh giới | luật `an`, văn bản `toàn an` | chỉ chỗ thứ hai | — |
| NFD | văn bản lưu dạng NFD | khoảng UTF-16 đúng trên chuỗi NFD; nhận thay giữ phần ngoài khoảng nguyên byte | — |
| Chồng nhau | luật `bảo đảm` và `bảo đảm bảo` trên `bảo đảm bảo vệ` | luật dài thắng, `overlapped` = 1 | — |
| Glossary thắng | chỗ khớp nằm trọn trong bản dịch Glossary đã chốt, nguồn chứa thuật ngữ | không dấu, `glossary_suppressed` = 1 | — |
| Câu cắt / trống | `is_omitted` hoặc `target_text` trống | không quét; câu cắt đếm vào `not_scanned` | — |
| Luật không cụm thay | `replacement` NULL | có dấu; dải không có Nhận thay hoạt động | lệnh nhận ⇒ từ chối, không ghi |
| Văn bản đã đổi | gõ sau lượt quét rồi nhận | — | `text_changed`, không ghi; dấu câu đó đã chết khi gõ |
| Luật bị xoá giữa chừng | khoá luật không còn | — | từ chối có kiểu, không ghi |
| Rỗng sau thay | văn bản mới trim rỗng | — | từ chối, không ghi |
| Thế hệ cũ | kết quả đến sau khi đổi Chương/Tác phẩm/luật | bị bỏ | — |
| F8 hết chỗ | không còn dấu phía sau | thanh trạng thái `convention.status.no_next`, không quay vòng | — |

</frozen-after-approval>

## Code Map

Rust (`src-tauri/src/`):
- `core/store/schema.rs:800` `GLOBAL_MIGRATIONS` (latest 16, header sentence :768 read by `segment_contract.rs`) -- add step 17 `CONVENTION_RULE_DDL` (one const; 16.3 reuses it for `project.db`). Two-tier DDL precedent: `IMPORT_CLEANUP_RULE_DDL` :1286.
- `core/cleanup/store.rs` (`list_tier` :180, `add_rule` :189) + `commands/cleanup.rs` (`store_for_tier` :36, pure fns + `pub mod wire` :176) -- shape to copy for the rule store and commands.
- `core/mod.rs:6-24` -- declare `convention`.
- `commands/segment/proofread_accept.rs:61` `check_proofread_range` -- the AD-53 mục 4 check, reuse as-is; `:182-197` inline arbitration branch -- extract to a shared `pub(crate)` fn used by both accept paths. Don't add a `write_non_user_target(` call to this file (`proofread_write_boundary.rs:590-705`).
- `core/segment/translation_origin.rs:35` `arbitrate`; `core/segment/utf16.rs:5` `utf16_to_byte`.
- `commands/segment/targets.rs:265` `write_non_user_target` (hardcodes draft) -- add `write_convention_target` beside it.
- `commands/segment/confirm.rs:288` the only `segment_version` insert (inline, plus `segment_unlinked` delete :281 and `insert_pair` :292) -- extract the version insert into a helper both callers use; read what `segment_unlinked` means before deciding whether the convention write touches it. `segment_baseline_guard.rs` requires `target_text` writes to also write `baseline_target_text`.
- `core/tm/mod.rs:77` `insert_pair`, `PairOrigin::from_stored` :51.
- `commands/segment/chapter_read.rs:236` `select_chapter_segments` (filters retired; `is_omitted` marks FR133).
- `commands/glossary.rs:1311` `glossary_marks_for_chapter` -- closest precedent for an async per-Chapter scan; `commands/tm.rs:920` lock-scope-then-compute pattern.
- `core/glossary/store.rs:1645` `GlossarySnapshot::load`, `:1666` `confirmed_terms(text, MatchLang)`; `match_lang_for_source_lang` :1501; `WorkMeta.source_lang`.
- `lib.rs:1121` handler list; `core/i18n/mod.rs:100` `message_keys!` for new `err.convention.*`.
- Gates: `tests/config_invariants.rs:1449` `blocking_wire_cases` (list the scan wire); `tests/ipc_argument_contract.rs:630` `NO_FRONTEND_CALLER`; `tests/ai_boundary.rs:922` already bans convention→ai bare deps; template for the new boundary test: `tests/matching_boundary.rs:335` + `tests/support/boundary_scan.rs`.
- Measurement pattern: `tests/story_22_measure_export_tm.rs` (`#[ignore]`, `--profile bench-release`).

Webview (`src/`):
- `panels/proofreadHighlight.ts` -- inline `<span>` marks, draws only when `textContent === scannedText`; `unwrapProofread` removes only its own selector. Mirror for convention spans; Proofreader wins on overlap.
- `panels/GridBlock.vue:757-775` mark styles; `:359-371` strip track. `panels/GridPanel.vue` repaint :1261/:1274/:1291, unwrap on typing :1516/:1695, `reportEdit` :1727.
- `panels/inlineStripPriority.ts:30/43` `InlineStripKind`, `PRIORITY_ORDER`; `inlineStripEligibility.ts:31/77`.
- `ProofreaderStrip.vue` (390 lines; hint reads `ai.proofread.next/prev` :48) -- DESIGN: convention strip is a variant of this shell, not a second visual component.
- `proofreadState.ts` -- results map :56, undo tray :68, `clearProofreadFor` :273, accept :457 (`replaceEditorSegment` :449), revert :516, `findingPositions` :644 / `step` :654.
- `commands/index.ts:5305-5326` `ai.proofread.next/prev` with F8 keys; ports :1203-1216 wired in `proofreadHandlers.ts`, spread in `main.ts:1129`; `commands/keys.ts:485-500` throws on chord conflicts; Check M `pointer` examples :1504/:1803.
- `GlossaryManageOverlay.vue` (1288 lines) + `glossaryManageState.ts` -- overlay shape to follow (width 1099px, dialog, focus return); `ProofreadMemoryOverlay.vue` (330 lines) is the smaller sibling to read first. `TitlebarToolsMenu.vue` -- `Công cụ ▾` entries; `Mod+Alt+A` is free (EXPERIENCE §Accessibility).
- `panels/editorPanelState.ts:999` `flushEditorBeforeDiscreteWrite`; `config/proofread.ts:227/245` adapter shape (never throws); `i18n/vi.json`.

E2E: `e2e/specs/story-12-15-proofread-ratio-reading-marks.e2e.mjs` (seed via `__TAURI_INTERNALS__.invoke`, `promote_ai_translation`); `proofread-underline.e2e.mjs:146`; helpers `e2e/support/{workspace,gridWait,pointer,flushWait}.mjs`.

## Tasks & Acceptance

**Execution:**
- [ ] spine `## Stack` + `src-tauri/Cargo.toml` -- read aho-corasick licence in the downloaded source, add the Stack row (transitive→direct precedent: `regex` row), then `aho-corasick = "=1.1.4"` -- NFR15.
- [ ] `core/store/schema.rs` -- global step 17: `convention_rule(id, avoid_phrase, replacement_phrase NULL, rule_key UNIQUE, enabled, created_at)`; bump header sentence.
- [ ] `core/convention/{mod,rule,matcher,casing}.rs` -- rule key (NFC + Unicode lowercase), matcher built once from resolved rules (longest wins, tie by key, losers → `overlapped`), letter/digit boundaries, UTF-16 spans on the stored string, case transfer (all-lower / Capitalised / ALL-UPPER, else rule spelling); pure, no `core::ai`.
- [ ] `commands/convention.rs` -- Global rule list/add (pure + wire), cached matcher invalidated on add; `convention_scan_chapter(generation)` async: lock only to copy what it needs, one read, release, match, Glossary check only on sentences with matches, return per-segment `{segment_id, scanned_text, spans[{start,end,rule_key,avoid,replacement_cased|null,tier}]}` + four counts + generation.
- [ ] `commands/segment/` -- `convention_accept(segment_id, expected_text, start, end, expected_phrase, rule_key)` and `convention_revert(…, original_phrase)` in a new file; shared check + rule-still-active check + arbitration on text before the write + `write_convention_target` (draft vs confirmed branches); empty after trim ⇒ refuse.
- [ ] `lib.rs`, `core/i18n/mod.rs`, `config_invariants.rs` -- register; `err.convention.*` keys (incl. empty / duplicate avoid phrase on add); list scan wire as blocking.
- [ ] `src/ConventionManageOverlay.vue` + `src/conventionManageState.ts` -- Global-tier list (`Cụm cần tránh → Cụm thay thế`, empty replacement in italic *không dùng*) and the add row; empty state *Chưa có luật nào…*; opened by `convention.manage.open` (`Mod+Alt+A`, `TitlebarToolsMenu.vue` entry); Esc closes and returns focus; a successful add bumps the scan generation and rescans the open Chapter.
- [ ] `src/config/convention.ts`, `src/conventionState.ts`, `src/panels/conventionHighlight.ts` -- adapters; per-segment results with generation; marks die on typing in that sentence and on any discrete write of it; undo tray LIFO per segment; span shifting after accept as AD-53 mục 6; scan on Chapter open.
- [ ] `GridPanel.vue` / `GridBlock.vue` -- dotted 2px `on-surface-variant` mark, `text-underline-offset: 4px`; click on a mark with no selection opens the strip at that spot; Proofreader wins on overlap.
- [ ] strip -- `convention` kind after `proofreader` in `InlineStripKind`/`PRIORITY_ORDER`; strip from the Proofreader shell: Tránh/tier/position line, Đề xuất line, Nhận thay (locked when no replacement), Bỏ qua (webview only, moves to next spot), prev/next, Ẩn, revert note + Hoàn lại, confirmed note; closes on Esc/Ẩn or when the caret leaves the sentence; buttons `@mousedown.prevent`.
- [ ] `src/commands/index.ts` -- `marks.next`/`marks.prev` (F8/Shift+F8) over both families in text order, landing on a Proofreader finding moves the caret there, on a convention spot opens its strip, end ⇒ status `no_next`/`no_prev`; remove default keys from `ai.proofread.next/prev`; `convention.accept/skip/revert/hide`; `pointer` declarations; Proofreader strip hint reads `marks.next/prev`.
- [ ] `src/i18n/vi.json` -- keys from EXPERIENCE §Khoá i18n mới used by this story only.
- [ ] tests -- `src-tauri/tests/convention_contract.rs` (I/O matrix rows; AD-67 12 (a)(b)(c)(f); two scans of the same input equal; scan writes nothing); `src-tauri/tests/convention_boundary.rs` (12 (h), both directions); `src-tauri/tests/story_16_1_measure_convention.rs` (ignored, bench-release: build ≤ 20 ms, per-sentence p95 ≤ 1 ms, Chapter 9,850 sentences p95 ≤ 250 ms at 2,000 rules); vitest for strip priority, F8 order across families, span shift, generation drop; update `inlineStripPriority.test.ts`, `proofreaderStripContent.test.ts:72`.
- [ ] `e2e/specs/story-16-1-convention-mark-accept.e2e.mjs` -- ticket `verify`: add the rule through the overlay, then real mouse click on the mark and on Nhận thay.
- [ ] `deferred-work.md` -- append the `Chủ: Epic 16` item (≤ 5 lines).

**Acceptance Criteria:**
- Given the user adds `đảm bảo → bảo đảm` in Quản lý quy ước (keyboard only, then mouse only), when the overlay closes, then the rule is listed and the open Chapter is rescanned.
- Given a Global rule `đảm bảo → bảo đảm` and an AI-translated sentence `Đảm bảo an toàn.`, when the Chapter opens, then a dotted mark sits under `Đảm bảo`; when the user clicks it and presses Nhận thay, then the cell reads `Bảo đảm an toàn.` and `translation_origin` is unchanged.
- Given a confirmed sentence with a match, when the user accepts, then the sentence stays confirmed, gains exactly one `segment_version` and one TM pair with the new text, and Hoàn lại restores the old text byte for byte.
- Given marks from both Proofreader and convention in one Chapter, when the user presses F8 repeatedly, then the caret visits them in text order and stops with a status message at the end.
- Given any scan, when it returns, then no `segment` row changed.
- Each new guard (NFC, arbitration, TM insert, Glossary check, boundary) turns red when its seam is really removed.

## Implementation Notes

- The rule-active check cannot share the write transaction: the rule lives in `global.db`, the sentence in `project.db`. `convention_active_rule` reads it just before the write and passes the row in (`segment_boundary` forbids a `&Store` below the shell). Harmless while 16.1 has no edit/disable/delete path; debt owned by Story 16.3.
- `write_convention_target` leaves `segment_unlinked` alone: a confirmed sentence already left that table when it was confirmed, and a draft stays draft.
- `insert_segment_version` (from `confirm.rs`) and `arbitrate_before_write` (from `proofread_accept.rs`) were extracted so Proofreader and convention share one version insert and one arbitration branch.
- The matcher cache is Global-only; 16.3 must key it by Work.
- Convention marks are drawn by `panels/proofreadHighlight.ts` and the strip is a second section of `ProofreaderStrip.vue` (DESIGN: a variant, not a second component); no `conventionHighlight.ts`.
- Label keys follow `check:commands`: `command.convention.manage.open|close|add`, not EXPERIENCE's `manage_open`; `convention.strip.hint` takes `{chord_next}`/`{chord_prev}`.
- The F8 notice for a sentence being typed (`sentence_pending`) is left to 16.4, which owns rescan after flush.
- Measurement (bench-release, 2,000 rules, 9,850 sentences of ~150 chars, 1 in 20 matching, machine not idle): build p95 4.6 ms, one sentence p95 0.005 ms, Chapter matcher p95 50.8 ms, `convention_scan_chapter` end to end warm p95 72.5 ms.
- E2E spec written; first runs stalled on a locked Mac at a Keychain prompt (pre-existing lock hold in `ai_config_get`, debt owned by Ice).

## Plan Change Log

## Review Triage Log

Pass 1 (quick): 0 high · 2 medium · 4 low · 4 false · 0 maybe-false.
- Rule-active check outside the write transaction — false for 16.1: no edit/disable/delete path exists yet; debt `Chủ: Story 16.3`.
- Close during add skips the rescan (`conventionManageState.ts`) — low · patch.
- `Matcher::build` swallows an automaton error — low · rejected: only past aho-corasick's state limit, far above 2,000 rules; fix adds an error path.
- Glossary check skipped when the Chapter changes mid-scan — false: a Chapter change requests a newer scan and the older generation is dropped.
- F8 after Esc/Ẩn restarts at the sentence's first mark (`stepMarks`) — medium · patch.
- Proofreader scan-level state outranks an opened convention strip — medium · patch.
- Esc in the cell does not hide the convention strip — low · patch.
- `segment_unlinked` untouched — false: confirmed sentences have no row; drafts keep theirs.
- `err.convention.text_changed` promises a rescan that 16.1 does not run — low · patch.
- Vietnamese `aura-allow-text` reason in `StatusBar.vue` — low · patch; `conventionHighlight.ts` name and strip manage link — false: DESIGN asks for a variant, the overlay exists.

## Design Notes

- Revert uses the same write as accept with inverted parameters (`[start, start + len16(replacement))` back to the original phrase), so on a confirmed sentence it also adds one version and one TM pair; AD-67 mục 8 puts confirmed sentences in AD-49 lớp (i) via the tray while it lives.
- The rule-active check and the case transfer both run inside the write transaction from the stored rule, so a rule edited between scan and accept is refused, never applied with a stale replacement.
- Proofreader's existing error codes come back from `check_proofread_range`; map them to `err.convention.*` at the convention command so the user never reads a Proofreader message.
- Phases (AGENTS.md): Rust · Webview · Tests that move (e2e + measurement), one fresh agent each, handed off through a notes file beside this plan.

## Verification

**Commands:**
- `npm run build` then `cargo test --manifest-path src-tauri/Cargo.toml --test convention_contract --test convention_boundary` -- expected: green.
- full `cargo test` + vitest once (Cargo.toml, migration and `lib.rs` changed) -- expected: green.
- `npm run check:commands` and `npm run check:i18n` -- expected: green, `POINTER_DEBT` empty.
- `cargo test --profile bench-release --locked --manifest-path src-tauri/Cargo.toml --test story_16_1_measure_convention -- --ignored --nocapture --test-threads=1` -- expected: numbers within AD-67 mục 11, stated with build and population.
- `npm run test:e2e -- --spec e2e/specs/story-16-1-convention-mark-accept.e2e.mjs` (outside sandbox) -- expected: green on WKWebView.
