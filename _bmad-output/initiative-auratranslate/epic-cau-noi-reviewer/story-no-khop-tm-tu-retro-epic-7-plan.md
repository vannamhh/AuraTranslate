---
title: 'Nợ khớp TM từ retro Epic 7'
type: 'chore'
ticket: '17'
created: '2026-10-07'
status: 'built'
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

1. F2.4. Decision (2026-10-07, Ice): phương án A — `accept_tm_fuzzy`/`accept_tm_exact` nhận `expectedTarget` (văn bản dải đã hiển thị), so với `pair.target_text`, lệch thì `tm.pair_not_found` và không ghi; cùng khuôn `tm_update_pair_target`; không migration. Phương án B (tem phiên bản, cột mới, migration một chiều) bị loại.
2. F2.7, ký tự điều khiển. Đo: `("一\u{1}二。", "a\u{fffe}b")` xuất rồi nhập lại thành `("一二。", "ab")`. Decision (2026-10-07, Ice): phương án A — cặp XML 1.0 không biểu diễn được bị bỏ khỏi tệp xuất, kết quả xuất báo số cặp bị bỏ, màn xuất hiện số đó (khoá `tm.exchange.export_left_out`). Phương án D (chặn ký tự từ lúc cặp vào TM) thành một mục nợ riêng, `Chủ: Ice`.
3. F3.4. Đo `SimilarityScorer::percent`, ngưỡng mặc định 65, mẫu tự dựng: câu dài khác ngôn ngữ tối đa 52; tiêu đề ngắn hỗn hợp chữ vượt ("Chapter 1 Hello" ~ "第1章 Hello" = 75, "Chapter 3" ~ "第3章 Chapter 3" = 75, "A." ~ "A。" = 100). Decision (2026-10-07, Ice): phương án A — giữ nguyên; sổ nợ `KHÔNG LÀM`, mở lại khi thấy một ca thật trong TM của người dùng.
4. F3.5. Đo: Zh "第三章" ~ " 第三章" = 83 trong khi `diff_spans` toàn `Equal`; NFC khác NFD = 60; chữ hoa/thường En = 100. Decision (2026-10-07, Ice): phương án B — nguồn mà `diff_spans` coi toàn `Equal` nhưng điểm thô dưới 100 hiện 99; không cặp nào khác đổi điểm; không đụng ca khoá NFC/NFD của `tm_contract.rs`.
   - Mâu thuẫn đo được giữa hai vế của quyết định: áp B tới NFC (chuẩn hoá `trim` + NFC như `diff_spans`) làm đỏ ca khoá `glossary_and_tm_catch_exactly_the_same_variants_in_both_directions` (TM bắt NFD~NFC ở ngưỡng 99, Glossary không). Đã làm phần không mâu thuẫn: chỉ khác khoảng trắng đầu/cuối (`str::trim`, cùng bộ ký tự với `diff_spans`). Phần NFC còn hở, 🟡 trong sổ nợ, `Chủ: Ice`.

## Code Map

- `src-tauri/src/commands/segment/tm_match.rs` -- `accept_tm_fuzzy`, `accept_tm_exact` (F2.4).
- `src-tauri/src/core/tm/tmx_io.rs`, `tmx.rs` -- `write_tmx_file` (trần xuất), `escape_into` (F2.7).
- `src-tauri/src/core/tm/mod.rs` -- `TierRows`/`read_tier_rows` (F2.9), `fuzzy_pairs_in_candidates` (F3.4), `concordance_key` (F3.5).
- `src-tauri/src/core/matching/mod.rs` -- `SimilarityScorer`, `diff_spans` (F3.5).

## Tasks & Acceptance

**Execution:**
- [x] `core/tm/mod.rs`, `commands/tm.rs`, `commands/segment/tm_match.rs`, `core/ai/rag.rs` -- F2.9: ba cấu trúc giống hệt và ba hàm nạp thành `TierRows` + `read_tier_rows`.
- [x] `core/tm/tmx_io.rs` -- F2.7: `write_tmx_file` từ chối nội dung quá `MAX_TMX_BYTES` trước khi tạo tệp.
- [x] `commands/segment/{tm_match,wire}.rs`, `src/config/segment.ts`, `tmFuzzyStripState.ts`, `panels/editorPanelState.ts`, `tmFuzzyCommandDeps.ts` -- F2.4: `expectedTarget`.
- [x] `core/tm/tmx.rs`, `commands/tm.rs`, `src/config/tm.ts`, `tmManageState.ts`, `TmManageOverlay.vue`, `i18n/vi.json` -- F2.7: bỏ và đếm cặp không biểu diễn được.
- [x] `core/tm/mod.rs` -- F3.5: nguồn chỉ khác khoảng trắng đầu/cuối hiện 99.
- [x] `deferred-work.md` -- F3.4 KHÔNG LÀM, F2.4/F2.7/F2.9 đóng, F3.5 🟡, một mục nợ mới cho phương án D.

**Acceptance Criteria:**
- Given năm mục, when story xong, then mỗi mục kết bằng đúng một dòng đóng/🟡/KHÔNG LÀM trong sổ nợ.

## Implementation Notes

- F2.9: `core/ai/rag.rs::TmRows` thêm ngưỡng và `tmx::distinct_tier_pairs` đọc một kho — không phải bản sao của ba cấu trúc kia, giữ nguyên. Không thêm tệp `.rs`, nên không có sàn quần thể nào đổi.
- F2.7 trần: kiểm ở `write_tmx_file`, một chỗ cho cả `tm_export_tier` lẫn đường `export_tier_to`; thông báo `tm.exchange.err_too_large` ("chưa ghi gì") đã khớp. Phản chứng: gỡ thật khối kiểm, `an_export_over_the_import_cap_is_refused_before_any_file_is_created` đỏ ở `expect_err`.

- F3.5: so `row.source_text.trim()` với câu, không gọi `diff_spans` cho từng hàng; trim-bằng-nhau suy ra `diff_spans` toàn `Equal`, ca `a_source_differing_only_by_surrounding_whitespace…` kiểm cả hai vế.
- Phản chứng thật (gỡ rồi khôi phục): phép trim ở `fuzzy_pairs_in_candidates`, phép so `expectedTarget` ở hai hàm `accept_*`, phép bỏ cặp ở `render_tmx` — mỗi ca mới đỏ đúng lý do.

## Verification

**Commands:**
- `cargo test --test tmx_contract --test tm_contract --test ai_rag_contract --test tm_wire` -- 42 + 141 + 42 + 7 passed.
