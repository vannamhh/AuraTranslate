---
title: 'Nợ khớp TM từ retro Epic 7'
type: 'chore'
ticket: '17'
created: '2026-10-07'
status: 'blocked'
blocked_reason: 'F2.4, phần ký tự điều khiển của F2.7, F3.4 và F3.5 mỗi mục có từ hai hình dạng hợp lệ trở lên (đổi đối số IPC, cột mới, hoặc đổi phần trăm người dùng thấy); chưa chọn thay Ice. F2.9 và trần xuất của F2.7 đã làm.'
baseline_revision: 'a524023bae24e506c2f5cffba84282dd179c66da'
route: 'full'
route_source: 'auto'
risk: 'medium'
review: ''
review_source: ''
lenses_ran: []
review_loop_iteration: 0
context: []
---

<frozen-after-approval reason="human-owned intent — do not modify unless human renegotiates">

## Intent

**Problem:** Năm mục nợ TM của retro Epic 7 (F2.4, F2.7, F2.9, F3.4, F3.5) hoãn tới Epic 8; mỗi mục phải kết bằng một dòng sổ nợ.

**Approach:** Sửa tại chỗ từng mục (`tm_match.rs` giữ riêng khỏi `tm.rs`, Notes của epic). Mục nào cần Ice chọn thì ghi phương án kèm số đo vào Open Questions, không tự chọn.

## Boundaries & Constraints

**Always:** Ghi `target_text` theo AD-47/AD-50/AD-51; không đổi đối số IPC, không thêm cột, không đổi phần trăm người dùng thấy khi chưa có quyết định của Ice; tệp `.rs` mới (nếu có) đi cùng nâng sàn quần thể.

**Never:** Gộp `tm_match.rs` với `tm.rs`; sửa `diff_spans` (AD-51 mục 4 cố định chuẩn hoá của nó).

</frozen-after-approval>

## Open Questions

Đo trên bản `test` (debug) của worktree này, máy tải rất cao nhưng bộ chấm điểm xác định nên số không phụ thuộc tải.

1. **F2.4 — `accept_tm_fuzzy`/`accept_tm_exact` đọc lại cặp rồi ghi văn bản người dùng chưa thấy.** Người dùng đồng ý với văn bản trên dải (`TmFuzzyStrip`), IPC chỉ gửi `segmentId, tier, unitId, force`. `tm_unit` không có cột phiên bản; `UPDATE` ở `core/tm/mod.rs::update_copies_target` đổi `target_text` tại chỗ, giữ `id` và `created_at`.
   - A. Thêm đối số `expectedTarget` (văn bản đã hiển thị); Rust so với `pair.target_text`, lệch thì `tm.pair_not_found`, không ghi. Không cần migration. Cùng khuôn `tm_update_pair_target`/`tm_delete_pair` đã gửi `expectedTarget`. Đổi chữ ký hai lệnh IPC, `src/config/segment.ts`, `tmFuzzyStripState.ts`, `ipc_argument_contract`.
   - B. Tem phiên bản: cột mới trên `tm_unit` (migration cho cả `global.db` một chiều, AD-30, và `project.db`), tăng khi sửa, gửi kèm `unitId`. Chắc hơn với sửa-rồi-sửa-lại-đúng-chữ cũ, nhưng chữ đó thì người dùng vẫn đã đồng ý.
   - Không chọn.
2. **F2.7, phần ký tự điều khiển.** `escape_into` bỏ im lặng ký tự dưới 0x20 (trừ tab/LF/CR) và U+FFFE/FFFF; XML 1.0 không biểu diễn được. Đo: cặp `("一\u{1}二。", "a\u{fffe}b")` xuất rồi nhập lại thành `("一二。", "ab")`, tức một cặp "mới". Trần `MAX_TMX_BYTES` phía xuất đã đóng (xem Implementation Notes).
   - A. Bỏ cặp không biểu diễn được khỏi tệp xuất và trả số cặp bị bỏ (đổi hình dạng kết quả xuất, thêm khoá i18n).
   - B. Từ chối cả lần xuất bằng lỗi mới nêu số cặp (một cặp hỏng chặn cả kho).
   - C. Giữ nguyên, ghi vào tài liệu người dùng rằng ký tự điều khiển bị bỏ khi xuất.
   - D. Chặn ký tự đó từ lúc cặp vào TM (đổi đường ghi vào `tm_unit`, ngoài tầm story này).
3. **F3.4 — tầng Global không lọc ngôn ngữ.** Đo `SimilarityScorer::percent`, ngưỡng mặc định `DEFAULT_TM_FUZZY_THRESHOLD = 65`:
   - Câu nguồn dài khác ngôn ngữ không chạm ngưỡng; điểm cao nhất của mẫu lệch ngôn ngữ ở loại này là 52.
   - Vượt ngưỡng ở chuỗi ngắn hỗn hợp chữ: Work `en` "Chapter 1 Hello" ~ Global "第1章 Hello" = 75; "Chapter 3" ~ "第3章 Chapter 3" = 75; "A." ~ "A。" = 100.
   - Không có số đo trên TM thật của người dùng. Hậu quả đo được là dải/RAG có thể chèn bản dịch của cặp nguồn tiếng Trung vào Work `en`, chỉ với tiêu đề ngắn.
   - A. Giữ nguyên (ghi là đã đo, hẹp).
   - B. Lọc theo hình dạng chuỗi nguồn (`has_han`) lúc chấm điểm tầng Global — không migration, cùng heuristic mà `render_tmx` đã dùng để gán `xml:lang` cho Global; sai với tiêu đề hỗn hợp.
   - C. Cột ngôn ngữ trên `tm_unit` của `global.db` (migration một chiều AD-30, đường nhập TMX/đẩy lên Global phải ghi cột).
4. **F3.5 — ba chuẩn hoá bất đồng.** Đo: Zh "第三章" ~ " 第三章" = 83 trong khi `diff_spans` trả toàn `Equal`; En "a b c" ~ " a b c" = 100; NFC khác NFD ("café au lait") = 60 (dưới ngưỡng 65, không hiện); chữ hoa/thường En = 100. Có ca khoá trong `tm_contract.rs::glossary_and_tm_catch_exactly_the_same_variants_in_both_directions`: NFC khác NFD thì TM và Glossary cùng KHÔNG khớp.
   - A. Chấm điểm trim + NFC trước khi so (thống nhất với `diff_spans`); đổi phần trăm người dùng thấy, và lật ca khoá NFC/NFD nên Glossary cũng phải đổi theo hoặc ca đó bỏ.
   - B. Giữ chấm điểm; chỉ chặn trần 99 cho nguồn mà `diff_spans` trả toàn `Equal` (không đổi điểm các cặp khác).
   - C. Giữ nguyên, ghi tài liệu rằng điểm tính trên văn bản thô còn dải diff chuẩn hoá.

## Code Map

- `src-tauri/src/commands/segment/tm_match.rs` -- `accept_tm_fuzzy`, `accept_tm_exact` (F2.4).
- `src-tauri/src/core/tm/tmx_io.rs`, `tmx.rs` -- `write_tmx_file` (trần xuất), `escape_into` (F2.7).
- `src-tauri/src/core/tm/mod.rs` -- `TierRows`/`load_tier_rows` (F2.9), `fuzzy_pairs_in_candidates` (F3.4), `concordance_key` (F3.5).
- `src-tauri/src/core/matching/mod.rs` -- `SimilarityScorer`, `diff_spans` (F3.5).

## Tasks & Acceptance

**Execution:**
- [x] `core/tm/mod.rs`, `commands/tm.rs`, `commands/segment/tm_match.rs`, `core/ai/rag.rs` -- F2.9: ba cấu trúc giống hệt (`FuzzyCandidates`, `ConcordanceCandidates`, `ManageSnapshot`) và ba hàm nạp thành một `TierRows` + `load_tier_rows`.
- [x] `core/tm/tmx_io.rs`, `tests/tmx_contract.rs` -- F2.7: `write_tmx_file` từ chối nội dung quá `MAX_TMX_BYTES` trước khi tạo tệp.
- [ ] F2.4, phần điều khiển của F2.7, F3.4, F3.5 -- chờ Ice (Open Questions 1-4).

**Acceptance Criteria:**
- Given một mục cần Ice chọn, when story dừng, then sổ nợ có dòng `🟡` trỏ tới câu hỏi trong plan này, `Chủ: Ice`.

## Implementation Notes

- F2.9: `core/ai/rag.rs::TmRows` thêm ngưỡng và `tmx::distinct_tier_pairs` đọc một kho — không phải bản sao của ba cấu trúc kia, giữ nguyên. Không thêm tệp `.rs`, nên không có sàn quần thể nào đổi.
- F2.7 trần: kiểm ở `write_tmx_file`, một chỗ cho cả `tm_export_tier` lẫn đường `export_tier_to`; thông báo `tm.exchange.err_too_large` ("chưa ghi gì") đã khớp. Phản chứng: gỡ thật khối kiểm, `an_export_over_the_import_cap_is_refused_before_any_file_is_created` đỏ ở `expect_err`.

## Verification

**Commands:**
- `cargo test --test tmx_contract --test tm_contract --test ai_rag_contract --test tm_wire` -- 42 + 141 + 42 + 7 passed.
