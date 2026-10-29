---
title: 'Chế độ đọc: cột chữ nằm giữa trang, nguyên văn song ngữ ngang đoạn bản dịch'
type: 'bugfix'
ticket: ''
created: '2026-10-29'
status: 'built'
baseline_revision: '3b77890675c4ca666bac4e6ab4e2563c517321ec'
route: 'oneshot'
route_source: 'auto'
risk: 'low'
review: 'quick'
review_source: 'pinned'
lenses_ran: ['quick']
review_loop_iteration: 0
context: ['{project-root}/_bmad-output/initiative-auratranslate/ux-auratranslate/mockups/reading-mode.html']
---

<frozen-after-approval reason="human-owned intent — do not modify unless human renegotiates">

## Intent

**Problem:** Ở Chế độ đọc, `.page` là flex không căn giữa, nên cột chữ dính lề trái thay vì nằm giữa như `mockups/reading-mode.html` (`.page{justify-content:center}`). Khi bật song ngữ, nguyên văn là một cột `.margin` riêng chạy từ đỉnh trang, nên đoạn nguyên văn thứ n không ngang với đoạn bản dịch thứ n. Trong ảnh Ice chụp, đoạn 2 lệch khoảng 32px, và độ lệch cộng dồn qua từng đoạn.

**Approach:** Cột chữ nằm giữa trang. Mỗi đoạn nguyên văn nằm ở lề trái, ngang đỉnh đoạn bản dịch của nó, và không bao giờ đè lên đoạn nguyên văn kế tiếp. Khi nguyên văn cao hơn bản dịch, đoạn bản dịch kế tiếp lùi xuống cho đủ chỗ. Khi cửa sổ hẹp, cột chữ dịch sang phải nhường chỗ cho nguyên văn trước, rồi mới co lại. Ice chọn bề rộng lề nguyên văn 30ch, thay cho 22ch (2026-10-29). Đo trên văn bản trong ảnh, khoảng lùi giữa hai đoạn bản dịch là: Thoáng 19px (không lùi), Cân 28–33px, Đặc 53–80px. Với 22ch các số tương ứng là 54–80, 108–113 và 133–159px. 🔵 2026-10-29: các số đó đo khi ghi chú chưa có lề dưới; bản đã dựng (lề dưới 1em) cho Thoáng 19px, Cân 44–49px, Đặc 69–96px. Sau đó Ice thấy chữ nguyên văn nhỏ khó đọc và chọn tăng hai bậc thang chữ đọc, từ 16px lên 19px (token `read-lg`). Khoảng lùi lúc đó: Thoáng 56–93px, Cân 115–121px, Đặc 135–172px; lề rộng 285px.

</frozen-after-approval>

## Implementation Notes

- Đi đường oneshot: một tệp Vue (template và CSS của `src/modes/ReadingMode.vue`), cộng việc đổi selector `.margin` trong `e2e/specs/story-5-11-reading-mode.e2e.mjs` và `story-5-12-reading-frontier.e2e.mjs`, cộng một ca e2e đo vị trí.
- Đo trước khi chọn (Playwright WebKit 27.2, `.page` 1460px, font thật Source Serif 4 và Noto Serif CJK TC, văn bản trong ảnh của Ice, 3 đoạn):
  - Định vị tuyệt đối như mockup (`left:-19ch`) làm ghi chú đè lên nhau 35–143px ở cả ba mức chữ, vì đoạn chữ Hán cao hơn đoạn bản dịch ⇒ loại.
  - Đẩy ghi chú xuống bằng JS thì sau hai đoạn đã lệch 120–284px, và độ lệch cộng dồn ⇒ loại.
  - Ghi chú `float` có `margin-inline-start` âm, đặt trong khung đoạn `display: flow-root`: đỉnh luôn ngang (lệch 0), không đè.
- Bố cục `.page`: grid ba rãnh `minmax(<lề ghi chú>, 1fr) minmax(0, auto) minmax(0, 1fr)`. Đủ chỗ thì hai rãnh bên bằng nhau nên cột nằm giữa; thiếu chỗ thì rãnh trái giữ mức tối thiểu và cột dịch sang phải. Khi tắt song ngữ, lề tối thiểu bằng 0.
- `.page` mang font `source-latin` để `ch` trong rãnh grid đo cùng bề rộng với ghi chú (`--source-note-inline: 30ch`). Các đoạn nay nằm trong `.reading-paragraph`, nên `readingModeImages.test.ts` lấy figure và đoạn theo thứ tự tài liệu thay vì theo con trực tiếp của `.column`.
- Bất ngờ: spec 5-11 kéo cỡ chữ lên 22px bằng Tinh chỉnh trước bước song ngữ, nên ở cửa sổ e2e 1280px cột rộng 745,75px; mỗi bên còn 251px, ít hơn rãnh tối thiểu 265px, nên cột dịch phải 14px như thiết kế. Assert viết theo luật `lề trái = max(rãnh ghi chú, nửa khoảng trống)`, không đòi căn giữa tuyệt đối.
- Đối chứng trên e2e 5-11: mã HEAD ⇒ đỏ (cột lệch 502px khi tắt song ngữ); bỏ `flow-root` ⇒ đỏ (ghi chú dài thò khỏi khung đoạn 663px); bỏ rãnh `.page.bilingual` ⇒ đỏ (ghi chú tràn khỏi mép trang).
- Bấm `B` nay làm các đoạn cao thêm, nên trước khi vẽ lại `ReadingMode.vue` ghi câu đầu vùng nhìn và vị trí của nó, sau khi vẽ lại thì cuộn bù phần chênh (hai `watch(readingBilingual)`, `flush: 'pre'` và `'post'`). Đổi mức chữ 1/2/3 cũng làm trang chảy lại mà không giữ chỗ, nhưng chuyện đó có từ trước và không nằm trong plan này.
- Theo mockup: nguyên văn căn phải, sát cột chữ (`text-align: end`); khối "Điểm dừng của lượt đọc" căn giữa.

## Plan Change Log

- 2026-10-29, Ice: "tăng size của chữ tiếng trung lên 2 size nữa, chữ nhỏ khó đọc". Đo hai cách trên văn bản trong ảnh (WebKit 27.2, 1460px): 19px/1,95 (`read-lg`, hai bậc 16 → 17,5 → 19, không cần token mới) và 18px/1,66 (cần token mới, ký trong `deviations` theo AD-34). Ice chọn 19px. `.page` và `.source-note` đổi từ `source-latin` sang `read-lg`, nên rãnh 30ch nở theo (mỗi dòng vẫn 15 chữ Hán). Các ca e2e đo vị trí tính rãnh từ kích thước thật nên không phải sửa. GIỮ: lề 30ch, cách thả nổi trong `flow-root`.

## Review Triage Log

- medium · patch — `.frontier` vẫn dính trái dưới cột chữ đã căn giữa. Đã thêm `text-align: center`.
- medium · patch — bấm `B` làm chỗ đang đọc trôi đi, vì ghi chú làm đoạn cao thêm. Đã thêm phần giữ câu đầu vùng nhìn; có vitest, gỡ watcher cuộn bù ⇒ đỏ (`scrollTop` 0 thay vì 60).
- low · patch — `margin-block`/`margin-inline-start` nằm ngoài tầm đọc của Kiểm K, nên `aura-allow-spacing` là miễn trừ chết. Đã gộp thành `margin`; gỡ miễn trừ ⇒ `check:tokens` đỏ ở dòng đó. Lỗ hổng chung của cổng đã ghi vào `deferred-work.md` (Chủ: Ice).
- low · patch — chú thích `.column` gán phép đo thời flex cho rãnh grid. Đã viết lại không kèm số đo; e2e 5-11 (68 ký tự mỗi dòng ±0,5) xanh dưới grid.
- low · patch — `readingState.ts` còn nói `.column` mang `flex: 0 0 auto`. Đã sửa.
- low · patch — chú thích `.source-note` dài 4 dòng, hai chủ đề. Đã rút còn 2 dòng.
- medium · patch — e2e chỉ có một đoạn, nên không chạm tới đoạn thứ hai trở đi. Đã nhân bản tạm một khung đoạn: đoạn kế bắt đầu dưới ghi chú dài, và ghi chú của nó ngang đoạn (lệch < 1px).
- low · patch — nhánh "rồi mới co lại" không có ca nào. Đã tạm thu hẹp `.page`: cột co đúng 100px, lề trái bằng rãnh ghi chú, ghi chú vẫn nằm trong trang.
- low · patch — ghi chú căn trái, khác mockup (`text-align:right`). Đã đổi sang `text-align: end`.
