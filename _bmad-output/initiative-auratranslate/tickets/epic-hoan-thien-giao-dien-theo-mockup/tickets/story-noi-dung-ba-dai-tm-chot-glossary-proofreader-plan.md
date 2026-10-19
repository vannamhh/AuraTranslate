---
title: 'Nội dung ba dải: TM, chốt Glossary, Proofreader'
type: 'feature'
ticket: '10'
created: '2026-10-14'
status: done
baseline_revision: '26a731a312b6cb4e3cc59275ce2165ce1b818696'
route: 'full'
route_source: 'auto'
risk: 'medium'
review: 'quick'
review_source: 'pinned'
lenses_ran: ['quick']
review_loop_iteration: 0
context:
  - '{project-root}/_bmad-output/initiative-auratranslate/ux-auratranslate/DESIGN.md'
---

<frozen-after-approval reason="human-owned intent — do not modify unless human renegotiates">

## Intent

**Problem:** Ba dải dưới câu đang sửa (Story 12.9) đúng chỗ nhưng nội dung lệch mockup (`tm-fuzzy-match.html`, `glossary-confirm-inline.html`, `proofreader.html`): dải chốt Glossary chỉ có một ô điền sẵn âm Hán Việt, không đề xuất bản dịch từ TM (FR146), không xuất xứ/phân loại; dải TM thiếu ngày, "Xem cả câu", tiêu đề và gợi ý theo mockup; dải Proofreader không bộ chọn phạm vi, không lối vào Chỗ đã bỏ qua (D11), Chỗ đã bỏ qua không ngày, không số lần chặn. Dòng: b:117, b:121, c:112, c:114, c:116; B13–B17 (B16 không gồm "bạn vừa viết").

**Approach:** Rust: đề xuất TM cho thuật ngữ qua hàm đọc công khai của `core/tm` (để 12.14 gắn `TmSidePolicy`, AD-57 ⑤), `term_origin`/`category` trên mark Glossary, `created_at` trên `TmFuzzyMatch`, đếm TM theo phía, bộ đếm chặn trên `proof_ignore`. Webview đổi nội dung ba dải, Chỗ đã bỏ qua và dòng TM ở thanh trạng thái; giữ chỗ đặt, ưu tiên và lệnh hiện có.

## Decisions (Ice, 2026-10-14)

1. Đề xuất TM (FR146) = cặp TM có câu nguồn đúng bằng thuật ngữ (`pairs_for_source`, cả hai tầng), gom theo bản dịch, nhãn "TM · N lần". Không tra chuỗi con.
2. Dải TM giữ phím 1–3 khi tiêu điểm ở dải; chỉ đổi tiêu đề và gợi ý (`1 2 3 nhận · Esc ẩn dải`). Không chord mới.
3. "Xem cả câu" mở rộng tại chỗ trong hàng (đủ câu nguồn và đích).
4. Bộ chọn phạm vi chỉ hiện số câu, không chi phí ước tính; phần lệch ghi nợ.
5. Thêm bộ đếm chặn vào `proof_ignore` (bậc migration kế tiếp có thật, 36 lúc lập plan); hàng có trước bậc này là `NULL` (chưa đếm), hàng mới bắt đầu 0.
6. Không "Chương N · câu M" ở dải TM (AD-6); không lý do/đề xuất trong Chỗ đã bỏ qua (AD-53 ⑦). Dòng "TM: N cặp · X của tôi · Y của người khác · ngưỡng" ở thanh trạng thái như mockup.
7. Bộ chọn phạm vi nằm trong dải Proofreader khi dải đang hiện; lượt quét đầu vẫn qua `⌘P`/`Công cụ ▾`. Esc trên dải Proofreader chạy lệnh mới `ai.proofread.hide` (không phím), ẩn dải cho câu hiện tại tới khi con trỏ sang câu khác, như `tm.fuzzy.hide` (Ice không bác, 2026-10-14).
8. Giữ một plan dù 4.714 token (Ice, 2026-10-15).

## Boundaries & Constraints

**Always:**
- Đề xuất TM đọc qua `pairs_for_source`, không SQL riêng, không `browse()`.
- Đề xuất đánh số theo vị trí: Hán Việt (khi có) là 1, TM tiếp theo; tối đa 9. Một bản dịch trùng chữ Hán Việt hiện một lần, ở hàng Hán Việt, kèm "TM · N lần".
- Phím số chốt ngay đề xuất đó (qua `glossary.confirm.save` → `glossary_confirm_pending_translation`) chỉ khi tiêu điểm không ở ô "gõ cách khác…"; trong ô đó chữ số là chữ, Enter chốt chữ đã gõ, Esc là Để sau như hiện nay.
- Gợi ý phím của dải Proofreader dựng từ chord thật (`shortcutRows`), không viết cứng `F8`.
- Phím so bằng `event.code`; mọi `@click` là đúng một `dispatch('id')`; handler phím mới có dòng `HANDLER_TABLE`; lệnh mới có `labelKey` trong `vi.json`; trạng thái module mới có hàm reset (`check:panel-refs`).
- Đóng Chỗ đã bỏ qua mở từ link trong dải trả tiêu điểm về link (UX-DR17).
- Bộ đếm tăng 1 cho mỗi phát hiện `filter_remembered` bỏ vì khớp mục đó, trong cùng giao dịch ghi `proof_scan`; hàng `NULL` giữ `NULL`.
- Chữ mới qua `vi.json`, màu/cỡ qua token.

**Never:**
- Không "bạn vừa viết" (loại 3), không tỉ lệ FR150 (12.15), không lọc theo Tác phẩm (12.14), không diff phía bản dịch ở dải TM.
- Không đổi chỗ đặt, `PRIORITY_ORDER`, `topmostStrip` (12.9). Không dependency mới.
- Không gỡ nút Trước/Kế của dải Proofreader (lối chuột duy nhất).

## I/O & Edge-Case Matrix

| Scenario | Input / State | Expected Output / Behavior | Error Handling |
|----------|--------------|---------------------------|----------------|
| Hai bản dịch TM | Thuật ngữ T chờ chốt; TM có 2 cặp nguồn = T, đích X (2 lần) và Y (1 lần); Hán Việt H | Dải: 1 H · âm Hán Việt, 2 X · TM 2 lần, 3 Y · TM 1 lần; phím 2 chốt X | — |
| Câu chứa thuật ngữ | TM chỉ có câu dài chứa T | Không đề xuất TM; chỉ Hán Việt | — |
| Không Hán Việt | Thiếu dữ liệu từ điển, TM có X | 1 X · TM 1 lần; dòng "chưa cài dữ liệu từ điển" giữ | — |
| Trùng Hán Việt | Đích TM = H | Một hàng H · âm Hán Việt · TM N lần | — |
| Chữ số trong ô gõ | Tiêu điểm ở ô, gõ `2` | Ô nhận chữ `2`, không chốt | — |
| Lệnh TM lỗi | `glossary_tm_suggestions` từ chối | Dải vẫn hiện Hán Việt và ô gõ, một dòng lỗi | i18n `err.*` |
| Mark | Mục chờ chốt có `term_origin`, `category` | Dòng "xuất xứ: … · phân loại: …"; trường rỗng thì bỏ vế đó | — |
| Hàng khớp mờ | Cặp tạo 14-07 | Meta "văn phong của tôi · Tác phẩm này · 14-07"; "Xem cả câu" mở/thu đủ hai câu | — |
| Gợi ý Proofreader | `ai.proofread.next` đổi sang `F9` | Gợi ý "F9 kế tiếp · ⇧F8 trước · Esc ẩn" | — |
| Esc dải Proofreader | Tiêu điểm ở dải, câu k | Dải ẩn; sang câu khác rồi về k thì hiện lại | — |
| Bộ đếm chặn | Mục M (đếm 0) chặn 2 phát hiện trong một lượt quét | M = 2; Chỗ đã bỏ qua "bỏ qua dd-MM · đã chặn 2 lần" | — |
| Hàng cũ | M có trước migration | `NULL`, không tăng; chỉ hiện ngày | — |

</frozen-after-approval>

## Code Map

- `src/GlossaryConfirmStrip.vue` -- `<form>`: tiêu đề, chip tầng, "Xuất hiện {count} lần", ô điền sẵn `hanVietSuggestion`, Lưu (`glossary.confirm.save`) / Để sau (Esc, `glossary.confirm.defer`, không hỏi lại trong Chương — đúng câu mockup); i18n `vi.json:1337-1345`, nhãn phân loại `glossary.manage.category_*` :1497. `src/glossaryConfirmStripState.ts:398` `confirmGlossaryConfirmStrip` → `src/config/glossary.ts:552`. Mockup `glossary-confirm-inline.html:127-145`.
- `src-tauri/src/commands/glossary.rs:~215-246` `GlossaryMarkWire` -- thêm `term_origin`, `category` (`core/glossary/entry.rs:220-222`, mẫu `QuickAddTerm` :82-92); dựng ở `core/glossary/store.rs:1523`.
- `src-tauri/src/core/tm/mod.rs` -- `pairs_for_source` :217, `PairOrigin::side()` :68-74, `count` :339-350. Lệnh mới `glossary_tm_suggestions` đặt cạnh lệnh Glossary, đăng ký trong `lib.rs`.
- `src-tauri/src/commands/segment/tm_match.rs:4-15` `TmFuzzyMatch` -- thêm `created_at` (`TmPair` :98-106 đã có).
- `src-tauri/src/commands/tm.rs:296-300` `tm_count_pairs` (wire :850) -- trả thêm số của tôi/người khác theo `side()`; `src/config/tm.ts:238` → `src/workspaceCountsState.ts:17` → `StatusBar.vue:212` (`vi.json:283`); ngưỡng `src/config/bootstrap.ts:226`. Mockup `tm-fuzzy-match.html:204-207`.
- `src/TmFuzzyStrip.vue` -- phím :63-88, meta :198-205, nút :207-216; i18n `vi.json:1379-1391`. Mockup `tm-fuzzy-match.html:159-193`. Lệnh TM `src/commands/index.ts:3686-3752` (mở rộng tại chỗ cần một lệnh không phím).
- `src/ProofreaderStrip.vue` -- tiêu đề :151-162, thẻ :164-195 (Trước/Kế :188-193), tóm tắt :71-129. `ai.proofread.next/prev` `F8`/`Shift+F8` ở `index.ts:4727-4747`; dựng chord như `TitlebarToolsMenu.vue:11-19`; Esc không là chord toàn cục (`index.ts:2887-2890`). Mockup `proofreader.html:198-220`, `:299-303`.
- `src/ProofreadScopeOverlay.vue:72-110` -- ba ô và số câu; dải dùng chung cách đếm, không chép.
- `src/ProofreadMemoryOverlay.vue:40-50`, `ai.proofread.memory.open` (`index.ts:4684`). `proof_ignore` `schema.rs:988-998` (bậc 34, có `created_at`); `filter_remembered` `core/ai/proofread.rs:250-267` (chỉ trả tổng, `proof_scan` ghi ở :485); `proofread_ignore_list` `commands/proofread.rs:411-447`, wire :351-358. Bậc mới nhất 35 (`SEGMENT_UNLINKED_DDL`); AD-57 ⑨: 12.14/15/19/20 lấy bậc kế tiếp có thật.
- e2e: không giả được lượt quét Proofreader (`e2e/specs/proofread-underline.e2e.mjs:142`); `openWorkspaceWithWork` (`e2e/support/workspace.mjs:59`).

## Tasks & Acceptance

**Execution:**
- [ ] `src-tauri/src/core/tm/mod.rs`, `src-tauri/src/commands/glossary.rs`, `src-tauri/src/lib.rs` -- `glossary_tm_suggestions { source_term }` → `[{ target_text, count }]` qua `pairs_for_source` cả hai tầng, gom và xếp theo `count` giảm dần rồi thứ tự AD-18 -- Quyết định 1.
- [ ] `src-tauri/src/commands/glossary.rs`, `core/glossary/store.rs` -- `term_origin`, `category` trên `GlossaryMarkWire` -- B15.
- [ ] `src-tauri/src/commands/segment/tm_match.rs`, `commands/tm.rs`, `core/tm/mod.rs` -- `created_at` trên `TmFuzzyMatch`; `tm_count_pairs` thêm mine/others -- b:121, B14.
- [ ] `src-tauri/src/core/store/schema.rs` + `PROJECT_MIGRATIONS`, `core/ai/proofread.rs`, `commands/proofread.rs` -- bậc migration thêm cột đếm nullable; chèn mới đặt 0; `filter_remembered` trả số chặn theo mục; tăng trong giao dịch `proof_scan`; `proofread_ignore_list` trả `created_at`, đếm -- Quyết định 5, c:116.
- [ ] `src/GlossaryConfirmStrip.vue`, `src/glossaryConfirmStripState.ts`, `src/config/glossary.ts` -- đầu dải (thuật ngữ, "chờ chốt bản dịch · xuất hiện N lần trong Tác phẩm", xuất xứ · phân loại), danh sách đánh số, ô "gõ cách khác…", dòng Để sau, đoạn Smart RAG Injector (B17) -- FR146, B15–B17.
- [ ] `src/TmFuzzyStrip.vue` (+ lệnh mở rộng trong `src/commands/index.ts`) -- tiêu đề, gợi ý, "khớp", meta có ngày, "Xem cả câu" -- b:117, b:121, B13.
- [ ] `src/ProofreaderStrip.vue`, `src/proofreadState.ts`, `src/commands/index.ts`, `src/main.ts` -- tiêu đề "Proofreader — phát hiện i trên n", gợi ý chord, nhãn nút theo mockup, bộ chọn phạm vi ba nút, link Chỗ đã bỏ qua, `ai.proofread.hide` + Esc -- c:112, c:114, D11, Quyết định 7.
- [ ] `src/ProofreadMemoryOverlay.vue`, `src/StatusBar.vue`, `src/workspaceCountsState.ts`, `src/config/tm.ts` -- ngày và số lần chặn; dòng TM theo phía và ngưỡng -- c:116, B14.
- [ ] `scripts/check-commands.mjs` -- dòng `HANDLER_TABLE` cho handler phím mới.
- [ ] Rust tests -- `tm_contract.rs` (đề xuất: khớp đúng câu, bỏ câu chứa, gom đếm, hai tầng); `glossary_marks_contract.rs`; `tm_wire.rs`/`ipc_contract.rs` (trường wire mới); `proofread_memory_contract.rs` (hàng cũ `NULL`, mới 0, tăng theo mục; đối chứng đỏ: gỡ thật lệnh tăng).
- [ ] Vitest -- bảng I/O phía webview: đánh số, phím số chốt / chữ số trong ô, trùng Hán Việt, lỗi lệnh, meta và mở rộng TM, gợi ý theo chord đổi phím (đối chứng đỏ: viết cứng `F8`), Esc ẩn và hiện lại, nút phạm vi và link dispatch đúng lệnh, Chỗ đã bỏ qua `NULL` không hiện đếm.
- [ ] `e2e/specs/story-12-10-glossary-tm-suggestions.e2e.mjs` -- ca Verify phía Glossary trên app thật.
- [ ] `_bmad-output/initiative-auratranslate/deferred-work.md` -- nợ chi phí trước khi quét (c:114, Chủ: Winston); `KHÔNG LÀM` cho "Chương · câu" (AD-6) và lý do/đề xuất (AD-53 ⑦); lượt dùng thật mở Chỗ đã bỏ qua từ dải sau lượt quét (Chủ: Epic 12); đóng mục B16 (`:6031-6038`).

**Acceptance Criteria:**
- Given thuật ngữ có hai cặp TM nguồn đúng bằng nó, when dải chốt mọc, then dải hiện âm Hán Việt và hai bản dịch TM, phím 2 chốt bản dịch TM thứ nhất và mục rời trạng thái chờ chốt.
- Given dải Proofreader đang hiện, when bấm link Chỗ đã bỏ qua rồi đóng lớp phủ, then lớp phủ mở và tiêu điểm về link.

## Implementation Notes

- Đề xuất TM: `glossary_tm_suggestions { sourceTerm }` → `[{ target_text, count }]` qua `core::tm::target_suggestions_for_term` (để 12.14 gắn `TmSidePolicy`); không Work mở thì chỉ đọc Global. Lệnh lỗi chỉ thêm một dòng lỗi, dải vẫn có Hán Việt và ô gõ.
- Dải chốt: ô gõ không còn điền sẵn âm Hán Việt; Hán Việt là lựa chọn 1, TM nối sau, tối đa 9. Giá trị lưu = lựa chọn đang nhắm (nhớ theo chữ, không theo chỉ số, để Hán Việt đến muộn không dời nó), rồi chữ đã gõ, rồi lựa chọn 1. Phím số theo `event.code`, chỉ ngoài ô gõ; vì thế form có `tabindex=-1` và `glossary.confirm.focus` vào form khi có lựa chọn.
- `GlossaryMarkWire` thêm `term_origin` và `category`, `TmFuzzyMatch` thêm `created_at`, luôn có mặt (không Option); webview bỏ vế xuất xứ/phân loại khi id lạ.
- `tm_count_pairs` trả `{ total, mine, others }` (`mine` = origin `self`, `others` = `other` + `bilingual_import`); dòng thanh trạng thái là khoá `status.tm_summary`, thay `status.tm_count`.
- Migration bước 36 thêm `proof_ignore.blocked_count` nullable: hàng cũ `NULL` (chỉ hiện ngày), hàng mới 0. Bộ đếm chỉ tăng khi webview ghi lượt quét (`ai_proofread_scan_record` nhận thêm `blocked`), cùng giao dịch với `proof_scan`; một phát hiện bị nhiều mục chặn tính cho mục có id nhỏ nhất.
- Esc trên dải Proofreader chạy `ai.proofread.hide` (không phím), ẩn tới khi con trỏ sang câu khác và trả tiêu điểm về ô bản dịch của câu đó; khi đang quét, lỗi hay chưa cấu hình thì Esc không làm gì, để Huỷ còn với tới và dải phát hiện không mất sau lượt quét. Link Chỗ đã bỏ qua dùng thuộc tính riêng `data-proofread-strip-memory` vì nút Công cụ giữ `data-proofread-memory-open`.
- `ProofreaderStrip.vue` import `config/shortcutsState` (gọi `detectIsMac()` lúc nạp module): test mock `../../src/commands` phải mock cả `config/shortcutsState`. `check:commands` `DISPATCH_FLOOR` nâng 258 → 280.
- Ngày hiện ở dải TM và Chỗ đã bỏ qua là `dd-MM` giờ địa phương (`src/panels/shortDate.ts`); test dùng mốc 12:00Z để không lệch múi giờ.
- Đối chứng đỏ đã chạy bằng cách gỡ thật: viết cứng `F8` ở gợi ý Proofreader làm đỏ `follows a rebound chord…` và `drops the hint for an unbound command`; gỡ dòng `event.target === translationInputEl.value` làm đỏ `a digit typed in the input is text`; xoá vòng UPDATE bộ đếm làm đỏ ca bộ đếm theo mục của `proofread_memory_contract`. Bốn bản vá sau review (ngưỡng, Esc lúc quét, trả tiêu điểm, nhắm theo chữ) mỗi bản gỡ ra đều làm đỏ đúng ca mới của nó.
- e2e 12-10 phủ vế Glossary (TM → phím số → mục rời chờ chốt); vế Proofreader (link mở Chỗ đã bỏ qua, trả tiêu điểm) không có e2e vì không giả được lượt quét, ghi nợ `Chủ: Epic 12`. ⚠️ Chưa lượt e2e nào xanh trên máy Ice ở lượt này: webview đứng ở `tauri://localhost` với `#app` rỗng, cả spec cũ `glossary-quick-add` lẫn HEAD sạch, trong khi cùng frontend mount bình thường qua Vite trong Chrome; spec 12-10 và regex mới của spec 12-7 chờ nightly.

## Plan Change Log

## Review Triage Log

Lượt 1 (quick): high 0 · medium 4 · low 3 · false 0 · maybe-false 0; 1 phát hiện của người điều phối thêm vào.
- medium · patch · `e2e/specs/story-12-7-workspace-grid-status-bar.e2e.mjs:133` vẫn khớp `TM \d+ cặp`; regex đổi theo `status.tm_summary`.
- medium · patch · `StatusBar.vue` đọc `bootstrapTmFuzzyThreshold`, không đổi sau khi lưu Cài đặt (`tmSettingsState.ts` chỉ giữ `knownThreshold`); thêm `tmFuzzyThresholdInUse`, ca mới đỏ khi gỡ.
- medium · patch · Esc lúc đang quét đặt `hiddenForSegmentId`, nên dải phát hiện biến mất khi lượt quét xong; `hideProofreadStrip` không làm gì ở trạng thái cấp lượt quét, ca mới đỏ khi gỡ.
- medium · patch · `aimedOption` nhớ chỉ số, nên khi Hán Việt đến muộn chèn vào đầu thì lần lưu ghi một bản dịch người dùng không nhắm; đổi sang nhớ chữ, ca mới đỏ khi gỡ.
- medium · patch · (người điều phối) Esc ẩn dải Proofreader nhưng không trả tiêu điểm như `tm.fuzzy.hide` (`restoreFocus`), tiêu điểm rơi về `body`; trả về ô bản dịch của câu neo, ca mới đỏ khi gỡ.
- low · patch · chú thích của `applyTarget` vẫn tả ô điền sẵn Hán Việt; đã sửa.
- low · patch · đổi định dạng ở `core/glossary/mod.rs`, `tests/project_contract.rs`, `tests/segment_contract.rs` không thuộc 12.10, nhưng `cargo fmt --check` trên worktree HEAD sạch đỏ đúng ba tệp này (các commit chưa push); giữ, tách thành commit riêng.
- low · reject · `proofread_scan_record` không kiểm `ignore_id` tồn tại hay tổng ≤ `filtered`: id lạ là hợp lệ khi người dùng bỏ ghi nhớ giữa lượt quét và lượt ghi, payload đến từ webview của chính app như `filtered`; sửa thì thêm nhánh canh, ít gặp khi dùng thường.

## Verification

**Commands:**
- `npm run build` rồi `cargo test --test tm_contract --test glossary_marks_contract --test tm_wire --test proofread_memory_contract --test ipc_contract` -- xanh.
- vitest các tệp của story -- xanh.
- Một lượt toàn bộ (migration và `lib.rs` là wiring chung, AGENTS.md) -- xanh.
- `npm run test:e2e -- --spec e2e/specs/story-12-10-glossary-tm-suggestions.e2e.mjs` -- xanh.
