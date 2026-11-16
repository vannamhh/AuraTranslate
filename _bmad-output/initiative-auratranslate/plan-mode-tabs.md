---
title: 'Ba tab chuyển chế độ thành một cụm điều khiển'
type: 'feature'
ticket: ''
created: '2026-11-16'
status: 'built'
baseline_revision: '9dad5acd0f4f9945ec20401d92f1e8a7101fc3e1'
route: 'oneshot'
route_source: 'auto'
risk: 'low'
review: 'quick'
review_source: 'pinned'
lenses_ran: [quick]
review_loop_iteration: 0
context: []
---

<frozen-after-approval reason="human-owned intent — do not modify unless human renegotiates">

## Intent

**Problem:** Ba tab chuyển chế độ hiện chỉ là chữ và gạch chân, chưa có một vùng điều khiển rõ ràng.

**Approach:** Ice chọn A: cụm nền chìm, bo 4px, các nút cao 28px; nút đang chọn nền xanh đậm và chữ sáng. Giữ nhãn Thư viện · Dịch · Đọc, token hiện có, hành vi chuyển chế độ và bàn phím. Focus và hover phải phân biệt được ở hai theme. Căn baseline với tên Tác phẩm và công cụ bên phải; không tăng chiều cao thanh tiêu đề 42px.

</frozen-after-approval>

## Implementation Notes

- Phạm vi dưới 100 dòng, triển khai trực tiếp theo oneshot. Ice chọn phương án A sau khi được báo cây đang có thay đổi website; giữ nguyên phần website, không đưa nó vào commit của tab.
- `src/App.vue` chứa cả nút chuyển chế độ và CSS; dùng lại `currentMode`, `dispatch`, `t` và token có sẵn.
- Dùng `aria-pressed` cho ba nút; mọi nút dùng cùng trọng lượng chữ để không đổi bề rộng khi chuyển chế độ. Theme tối dùng cặp primary/on-primary hiện có.
- Bỏ test soi CSS gạch chân trong `titlebarWorkName.test.ts` vì thiết kế không còn gạch chân. Baseline vẫn thuộc phép đo thực trong `e2e/specs/story-13-4-canh-baseline.e2e.mjs`; không đổi test e2e đó.

## Plan Change Log

## Review Triage Log

- low → patch: tab được chọn thiếu hover do cascade; thêm viền on-primary khi hover.
- false: theme tối đảo sáng/tối so với mô tả cho theme sáng; cặp token có sẵn thích nghi tương phản, đã xem ảnh cả hai theme.

## Verification

- `npm run check:tokens`, `npm run check:commands`, `npm run check:i18n`.
- Kiểm tra TypeScript, lint riêng App.vue và test frontend tác động thanh tiêu đề.
- Xem cụm tab trong trình duyệt ở theme sáng/tối; đo chiều cao và baseline. Kiểm tra chọn từng chế độ, hover và focus bàn phím.
- Đã đạt: token, i18n, command gates; eslint App.vue; vue-tsc; 52 ca frontend liên quan sau khi bỏ phép soi CSS lỗi thời (46 ca trong hai tệp không sửa, 6 ca ở titlebarWorkName).
- Chromium trên dev server: nút 28px, thanh 42px; tab và Công cụ cùng baseline 25.5px. Đã xem ảnh sáng/tối, click Dịch, Enter mở Đọc, focus-visible có outline 2px. Chưa chạy lại phép đo WKWebView có tên Tác phẩm cho diff này.
- CI gần nhất: https://github.com/vannamhh/AuraTranslate/actions/runs/43769215036 (commit 7e6b6b1e5f25a196288123ba889f1075b3514c23, trước diff này): macOS check và nightly e2e đạt; Windows cargo test đỏ ở `purging_all_rows_shrinks_db_plus_wal_on_disk` vì reader chặn wal_checkpoint(TRUNCATE). Chưa có CI cho diff này; không push.
