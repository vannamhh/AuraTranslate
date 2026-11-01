---
title: 'Áp dụng kết quả Winston rà AD-63'
type: 'chore'
ticket: ''
created: '2026-11-01'
status: 'built'
baseline_revision: '63e5fbda4b16e35918e11e818ecf5f801c487929'
route: 'oneshot'
route_source: 'auto'
risk: 'low'
review: 'quick'
review_source: 'pinned'
lenses_ran: ['quick']
review_loop_iteration: 0
context: ['{project-root}/_bmad-output/initiative-auratranslate/architecture-auratranslate/reviews/review-ad-63-winston-2026-11-01.md']
---

<frozen-after-approval reason="human-owned intent — do not modify unless human renegotiates">

## Intent

**Problem:** Winston chưa ký AD-63: văn bản lệch mã ở 13 chỗ, PRD lệch ở 3 chỗ, ước lượng FR137 lệch lượt quét thật, và ba bất biến chưa có canh.

**Approach:** Ice chốt 2026-11-01: D1 giữ ranh giới jieba trên văn bản gốc và khai giới hạn kèm số đo; D2 giữ nguyên văn trước tầng. Áp dụng toàn bộ A, B, C của bản rà, rồi xin Winston ký lại.

</frozen-after-approval>

## Implementation Notes

Oneshot vì mã sản phẩm dưới 40 dòng, phần còn lại là văn bản và test.

- C1 khác đề nghị của bản rà: chỉ gấp phần Glossary, tách `EstimateExclusions`, vì `ON CONFLICT` của lượt quét thật so hàng chờ nguyên văn; gấp cả hàng chờ làm ước lượng thấp hơn số thật. Winston đã nhận.
- Đối chứng gỡ thật: đảo nguyên văn và tầng đỏ `a_verbatim_global_entry_beats_a_fold_only_work_entry`; so Glossary nguyên văn đỏ ca ước lượng; gấp hàng chờ đỏ ca ước lượng; bỏ hàng chờ ở ước lượng đỏ ca ước lượng; gọi `fold_zh` từ `core/scope/kinds.rs` đỏ `only_the_listed_comparison_paths_call_the_fold_functions`.
- `cargo fmt --check`, `clippy --all-targets -D warnings`, `glossary_estimate_contract`, `glossary_marks_contract`, `matching_contract`, `matching_boundary`, `config_invariants`, `check:doc-refs`, `check:debt-owner` xanh.
- D1 đo trên Tác phẩm phồn thể thật của Ice, 200 câu, 9.031 ký tự, bản `test`; bản giản chuyển gần đúng bằng cặp một chữ của `cc-cedict`.
- Cây lúc commit còn `cover-name` (`tokens.json`, `LibraryMode.vue`, `check-tokens.mjs`, `DESIGN.md`, một spec e2e) và ba plan đổi `done`, không phải việc này nên không gộp.

## Verification

**Commands:**
- `cargo test --test glossary_estimate_contract --test matching_contract --test matching_boundary --test glossary_marks_contract` -- expected: xanh.
- `npm run check:doc-refs` -- expected: xanh.

## Review Triage Log

Lens `quick`: medium 3 · low 3 · false 3.

- low, patch: B1 thiếu 🔵 ở hàng addendum; đã thêm.
- false: A5 `không nhận MatchLang`; đã sửa theo lượt rà thứ hai của Winston trước khi xét.
- medium, patch: Rule 6 sai khuôn AD-61, AD-62; viết lại thành ca (a)–(g) kèm phép gỡ.
- low, patch: C4 chỉ chặn nơi gọi mới; Rule 6 (g) nói rõ giới hạn đó.
- medium, patch: phần hàng chờ của ước lượng không có canh; thêm khẳng định `pending_verbatim`. Lượt ráp ở `wire.rs` có từ trước, ghi nợ `Chủ: Ice`.
- medium, patch: quần thể D1 sai; sửa thành 28.803 trong 28.868 cặp có dạng giản nằm trong từ điển jieba, nêu Tác phẩm và bản dựng.
- low, patch: tiêu đề hẹp hơn đường tìm ở bản đích; bỏ chữ `nguồn`, Rule 5 nêu cả đích.
- false: chú thích ở `scripts/check-tokens.mjs`; không thuộc commit này.
- false: `cover-name` lọt vào diff; thay đổi đó không do việc này tạo, commit theo đúng danh sách tệp.
