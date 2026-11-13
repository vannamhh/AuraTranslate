---
title: 'Dải đề xuất TM bỏ đề xuất trùng bản dịch câu đang dùng'
type: 'feature'
ticket: ''
created: '2026-11-13'
status: 'built'
baseline_revision: '89518b15044eca2c3acfa35791abd3cfdd4aa939'
route: 'oneshot'
route_source: 'auto'
risk: 'medium'
review: 'quick'
review_source: 'pinned'
lenses_ran: ['quick']
review_loop_iteration: 0
context: []
---

<frozen-after-approval reason="human-owned intent — do not modify unless human renegotiates">

## Intent

**Problem:** Dải đề xuất TM (`tm_fuzzy_matches`) vẫn liệt kê cặp có `target_text` trùng bản dịch câu đang có con trỏ, tức đề xuất người dùng đã đang dùng; khi đó là đề xuất duy nhất, dải hiện ra mà không mang gì mới.

**Approach:** Ice quyết 2026-11-13: bỏ khỏi dải mọi đề xuất (danh sách khớp chính xác lẫn khớp mờ) có bản dịch trùng bản dịch hiện có của câu; nếu sau khi bỏ không còn đề xuất nào thì dải không hiện. Luật nằm ở Rust (AD-1); webview giữ nguyên.

</frozen-after-approval>

## Tasks & Acceptance

**Execution:**
- [x] `src-tauri/src/core/tm/mod.rs` -- `rank_fuzzy_candidates` nhận thêm `in_use_target: Option<&str>`; loại cặp có `target_text.trim()` bằng nó TRƯỚC khi cắt còn `FUZZY_MATCH_LIMIT`, để ba chỗ còn lại là ba đề xuất khác -- cặp đang dùng không chiếm chỗ.
- [x] `src-tauri/src/commands/segment/tm_match.rs` -- `prepare_tm_fuzzy` đọc thêm `segment.target_text`; bản dịch rỗng sau `trim` thì không loại gì. Nhánh khớp chính xác: vẫn rỗng khi < 2 bản dịch khác nhau (luật cũ), còn lại bỏ bản dịch đang dùng, rỗng sau khi bỏ thì trả danh sách rỗng. Mang bản dịch vào `TmFuzzyScan` cho `score_tm_fuzzy`.
- [x] `src-tauri/tests/tm_policy_contract.rs` -- truyền `None` ở chỗ gọi `rank_fuzzy_candidates`.
- [x] `src/TmFuzzyStrip.vue`, `src/i18n/vi.json`, `tests/frontend/tmFuzzyStrip.test.ts` -- gỡ nhãn "Đang dùng" (nhánh `inUse`, CSS `.tmf-in-use`, khoá `tm.exact.in_use`, ca vitest); `tm.exact.title` thành "Các bản dịch khác đã có cho câu nguồn này" -- Ice chọn B.
- [x] `src-tauri/tests/tm_contract.rs` -- ca: (1) một đề xuất mờ trùng bản dịch câu ⇒ `matches` rỗng; (2) bốn đề xuất mờ, một đang dùng ⇒ ba đề xuất còn lại, không có cái đang dùng; (3) hai bản dịch khớp chính xác, một đang dùng ⇒ `exact` chỉ còn cái kia; (4) bản dịch câu chỉ khác đề xuất ở khoảng trắng đầu/cuối vẫn bị loại; (5) câu chưa dịch ⇒ danh sách như cũ.

**Acceptance Criteria:**
- Given câu đang có bản dịch trùng đề xuất duy nhất của nó, when con trỏ dừng ở câu, then dải đề xuất TM không hiện.
- Given câu có nhiều đề xuất và một cái trùng bản dịch hiện có, when dải hiện, then cái đó không có trong dải và các đề xuất khác vẫn đủ.
- Given câu chưa có bản dịch, when dải hiện, then danh sách như trước thay đổi này.

## Implementation Notes

- Oneshot: ~90 dòng Rust, thêm phần gỡ ở webview.
- Ice chọn B 2026-11-13 sau review: lọc cả danh sách khớp 100%, lật 7-8 Q1(A) (nhãn "Đang dùng"). Câu "webview giữ nguyên" ở Approach vì thế không còn đúng: nhãn đã gỡ. Đã ghi 🔵 ở plan 7-8 và 🟡 ở mục nợ 7-8.
- `distinct_exact_targets` khử trùng theo bản dịch đã `trim`: hai bản chỉ khác khoảng trắng (vd. nhập TMX) là một bản dịch.
- Câu chưa dịch so với chuỗi rỗng: chỉ cặp có bản dịch toàn khoảng trắng bị bỏ, vốn không phải đề xuất dùng được.
- So bằng `trim()` hai vế (`core::tm::is_in_use`).
- Rust đọc bản dịch trên đĩa; chữ đang gõ chưa flush (≤ 5 s, AD-35) chưa được tính. Dải quét lại khi con trỏ dừng ở câu, nên lần quét sau sẽ đúng.
- Dòng "N cặp bị ẩn" của công tắc "chỉ của tôi" (AD-57) không phải đề xuất; luật hiện dải của nó giữ nguyên.

## Verification

**Commands:**
- `npm run build` (nếu chưa có `dist/`), rồi `cargo test --test tm_contract --test tm_policy_contract` trong `src-tauri/` -- expected: xanh.
- `cargo clippy --locked --all-targets -- -D warnings` và `cargo fmt --check` trong `src-tauri/` -- expected: mã thoát 0.

## Review Triage Log

Lượt quick, 6 phát hiện: 3 medium, 3 low; cả 6 đã sửa.
- medium, HALT rồi sửa: lọc danh sách khớp 100% lật 7-8 Q1(A), nhãn "Đang dùng" thành mã chết. Ice chọn B: gỡ nhãn ở webview.
- medium, sửa: quyết định 7-8 bị lật mà hồ sơ không đổi. Ghi 🔵 ở plan 7-8, 🟡 ở mục nợ 7-8.
- low, sửa: ca câu chưa dịch không chạm vế "bản dịch rỗng". Đã bỏ vế đó (so với chuỗi rỗng là đủ), thêm ca đường khớp mờ.
- low, sửa: luật `trim` chưa có ca ở danh sách khớp 100%. Thêm `an_exact_target_in_use_with_surrounding_whitespace_still_drops_out`, gỡ `trim` thì đỏ.
- medium, sửa: khử trùng so chuỗi gốc còn lọc so `trim`. Đổi khử trùng sang `trim`, thêm `exact_targets_differing_only_by_surrounding_whitespace_are_one_translation`, trả về chuỗi gốc thì đỏ.
- low, sửa: plan chưa đánh dấu việc xong và chưa ghi đối chứng. Đã cập nhật; đối chứng gỡ thật: `retain` ở rank (3 ca đỏ), lọc ở danh sách khớp 100% (1), `trim` (2), khử trùng (1).
