---
title: 'Nhãn "Đánh dấu · ↵ sửa" ở Chế độ đọc không đè lên câu đang chọn'
type: 'bugfix'
ticket: ''
created: '2026-10-28'
status: 'built'
baseline_revision: '7189c7049f1483aa856028affe548b3b16317e4e'
route: 'oneshot'
route_source: 'auto'
risk: 'low'
review: 'quick'
review_source: 'pinned'
lenses_ran: ['quick']
review_loop_iteration: 0
context: []
---

<frozen-after-approval reason="human-owned intent — do not modify unless human renegotiates">

## Intent

**Problem:** Ở Chế độ đọc, khi câu đang chọn xuống từ hai dòng trở lên, nhãn `.mark-actions` ("Đánh dấu · ↵ sửa") hiện ở dòng ĐẦU của câu nhưng lấy hoành độ của điểm cuối dòng CHÓT. Vì vậy nó che chữ của chính câu đó. Đo trên WebKit 27.2 với cột 600px và câu 3 dòng: nhãn rộng 127px nằm ở y=8, che 2.739px² chữ của câu. Câu một dòng thì không bị.

**Approach:** Ice đã chọn phương án A (2026-10-28): nhãn đứng ngay sau chữ cuối của câu, trên dòng cuối của câu, nghĩa là dùng vị trí tĩnh của phần tử định vị tuyệt đối thay cho `inset-inline-start: 100%` / `inset-block-start: 0`. Hệ quả đã biết và đã chấp nhận: nhãn che chữ đầu câu kế tiếp trên cùng dòng (khoảng 127×16px). Câu kết thúc sát lề phải thì nhãn tràn ra lề, giống hành vi hiện tại. Phương án B (lề phải của cột, đo bằng JS) bị loại. 🔵 2026-10-29 (Ice đổi): sau khi dùng thử, Ice chọn đặt nhãn ở lề phải ngoài cột chữ, ngang dòng cuối của câu, vì nhìn đẹp hơn. Khi lề hẹp hơn nhãn cộng hai khe, nhãn quay về ngay sau chữ cuối như phương án A.

</frozen-after-approval>

## Implementation Notes

- Đi đường oneshot vì chỉ bỏ hai inset trong `.mark-actions` (`src/modes/ReadingMode.vue`) và thêm một phép đo hình học vào `e2e/specs/story-5-13-reading-marks.e2e.mjs`.
- `.mark-actions` vẫn là con của `<span class="reading-segment">`, nên `onSegmentMouseLeave` và `:focus-within` không đổi; chuột đi thẳng từ chữ cuối sang nhãn mà không rời câu. Vị trí không phụ thuộc vào việc nhãn hiện ra qua `:hover`, `.aimed` hay `:focus-within`, vì ba đường chỉ đổi `display`.
- Fixture của spec (`.repeat(12)`) cho câu 12 dòng; ca mới đòi số dòng > 1, nhãn nằm trên dòng cuối và mép trái nhãn cách chữ cuối trong khoảng [0, 32) px.
- Ca e2e còn đòi mọi điểm trên đường đi ngang từ chữ cuối sang nhãn (ba độ cao) thuộc câu đang chọn và điểm giữa nhãn trúng nút; xoá `::before` ⇒ đỏ, 24 điểm rơi ra ngoài câu.
- Lề phải: `.column` là khối chứa của nhãn (`position: relative`), class `mark-side` đặt `inset-inline-start: 100%`, còn trục dọc giữ vị trí tĩnh nên nhãn ngang dòng cuối. `measureMarkChipSide` đo một lần mỗi khi câu đang chọn đổi (`watch` với `flush: 'post'`): đủ lề thì trả độ dài cầu `--mark-bridge` từ chữ cuối tới nhãn, thiếu lề thì `null`.
- Giới hạn đã biết: đổi cỡ cửa sổ hoặc thanh Tinh chỉnh trong lúc một câu đang được chọn thì phải tới lần chọn kế mới đo lại. Trong lúc câu được chọn, cầu phủ phần câu kế trên cùng dòng cuối, nên rê chuột vào đúng phần đó không đổi câu đang chọn.
- Bất ngờ: trong fixture này mọi dòng, kể cả dòng cuối, chạy hết bề rộng cột (mép phải 592), nên nhãn cũ nằm ở dòng 1 nhưng ngoài lề (x=608), diện tích giao = 0. Vì vậy assert diện tích giao không thể đỏ ở đây và đã bị bỏ; `onLastLine` là assert bắt được mã cũ.
- Lượt e2e đầu tiên sau build đỏ ở bước `M` lặp (3 dấu thay vì 2), sau phần đã sửa; chạy lại đúng binary hai lần đều xanh, lượt cuối cũng xanh. Không tìm nguyên nhân thêm.

## Plan Change Log

- 2026-10-28, Ice dùng thử: nhãn ở cuối câu rất khó bấm, vừa rê chuột là mất. Khe `margin-inline-start` giữa chữ cuối và nhãn, cùng phần dòng chữ cao hơn nhãn 24px, nằm trên câu kế tiếp, nên câu kế lấy mất câu đang chọn. Đo WebKit: chỉ 4/10 điểm (nửa trên, giữa) và 1/10 (đáy dòng) trên đường từ chữ cuối sang nhãn thuộc câu đang chọn. Bổ sung: `.mark-actions::before` trong suốt phủ khe và trọn `1lh` của dòng đọc, `z-index: -1` để không chặn cú bấm vào nút ⇒ 10/10 ở cả ba độ cao. GIỮ: vị trí nhãn theo phương án A.
- 2026-10-29, Ice: "đổi nhãn sang bên cạnh đi nhìn đẹp hơn", chọn lề phải ngang dòng cuối. Bổ sung nhánh lề phải đo bằng JS, kèm nhánh dự phòng khi lề hẹp; cầu `::before` nay dài theo `--mark-bridge`. Ca e2e đo cả hai nhánh (nhánh hẹp bằng cách tạm thu `.page`). Đối chứng: xoá `watch` ⇒ đỏ, nhãn nằm trước mép cột 17,8px; xoá `::before` ⇒ đỏ, 75 điểm rơi ra ngoài câu. GIỮ: cầu bắt chuột và vị trí dọc theo dòng cuối.

## Review Triage Log

- low · patch — ghi chú cũ nói ca e2e thu hẹp `.column`; mã thì không. Đã sửa ghi chú.
- low · patch — phần kiểm tay không có mục nợ có chủ. Vị trí không phụ thuộc đường hiện (chỉ đổi `display`) và e2e đo nó; phần kiểm tay đổi thành "nhìn trong `tauri dev` đang chạy" chứ không phải một việc chỉ người làm được.
- low · patch — ca mới không kiểm vế "ngay sau chữ cuối". Đã thêm `gapAfterLastGlyph` trong [0, 32).
- medium · patch — chưa chạy đối chứng đỏ. Đã chạy: đặt lại hai inset ⇒ đỏ ở `onLastLine` (dòng 153), vì nhãn nằm ở dòng 1; xem bất ngờ về diện tích giao ở trên.

## Verification

**Commands:**
- `npx wdio run e2e/wdio.conf.mjs --spec e2e/specs/story-5-13-reading-marks.e2e.mjs` (sau `cargo build --locked --features wdio`) -- expected: xanh. Đối chứng: đặt lại `inset-inline-start: 100%` và `inset-block-start: 0` ⇒ đỏ ở `onLastLine`.
- `npx vitest run tests/frontend/readingMarks.test.ts tests/frontend/readingModeImages.test.ts` -- expected: xanh, thoát 0.

**Manual checks (if no CLI):**
- Trong `tauri dev`, rê chuột lên một câu dài từ hai dòng trở lên: nhãn đứng sau dấu chấm câu ở dòng cuối.
