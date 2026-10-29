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

**Approach:** Ice đã chọn phương án A (2026-10-28): nhãn đứng ngay sau chữ cuối của câu, trên dòng cuối của câu, nghĩa là dùng vị trí tĩnh của phần tử định vị tuyệt đối thay cho `inset-inline-start: 100%` / `inset-block-start: 0`. Hệ quả đã biết và đã chấp nhận: nhãn che chữ đầu câu kế tiếp trên cùng dòng (khoảng 127×16px). Câu kết thúc sát lề phải thì nhãn tràn ra lề, giống hành vi hiện tại. Phương án B (lề phải của cột, đo bằng JS) bị loại. 🔵 2026-10-29 (Ice đổi): sau khi dùng thử, Ice chọn đặt nhãn ở lề phải ngoài cột chữ, ngang dòng cuối của câu, vì nhìn đẹp hơn. Khi lề hẹp hơn nhãn cộng hai khe, nhãn quay về ngay sau chữ cuối như phương án A. 🔵 2026-10-29 (Ice đổi lần hai): nhãn không hiện khi rê chuột, chỉ hiện khi bấm chuột vào câu (câu nhận focus, nên Tab bằng bàn phím cũng hiện). 🔵 2026-10-29 (Ice đổi lần ba): bấm chuột vào câu thì nhãn hiện ngay dưới dòng vừa bấm, mép trái tại con trỏ (lật lên trên dòng khi phía dưới bị khung cuộn cắt), có bóng đổ `--shadow-popover`; focus bằng Tab vẫn dùng vị trí lề phải / sau chữ cuối.

</frozen-after-approval>

## Implementation Notes

- Đi đường oneshot vì chỉ bỏ hai inset trong `.mark-actions` (`src/modes/ReadingMode.vue`) và thêm một phép đo hình học vào `e2e/specs/story-5-13-reading-marks.e2e.mjs`.
- `.mark-actions` vẫn là con của `<span class="reading-segment">`, nên `onSegmentMouseLeave` và `:focus-within` không đổi; chuột đi thẳng từ chữ cuối sang nhãn mà không rời câu. Vị trí không phụ thuộc vào việc nhãn hiện ra qua `:hover`, `.aimed` hay `:focus-within`, vì ba đường chỉ đổi `display`. 🔵 2026-10-29: chỉ còn `:focus-within`; ở nhánh lề phải chuột rời câu khi đi sang nhãn, việc đó vô hại vì nhãn không còn tắt theo chuột.
- Fixture của spec (`.repeat(12)`) cho câu 12 dòng; ca mới đòi số dòng > 1, nhãn nằm trên dòng cuối và mép trái nhãn cách chữ cuối trong khoảng [0, 32) px.
- Ca e2e còn đòi mọi điểm trên đường đi ngang từ chữ cuối sang nhãn (ba độ cao) thuộc câu đang chọn và điểm giữa nhãn trúng nút; xoá `::before` ⇒ đỏ, 24 điểm rơi ra ngoài câu. 🔵 2026-10-29: cầu `::before` và assert đường đi đã bỏ cùng lượt bấm-mới-hiện; còn assert nút trúng.
- Lề phải: `.column` là khối chứa của nhãn (`position: relative`), class `mark-side` đặt `inset-inline-start: 100%`, còn trục dọc giữ vị trí tĩnh nên nhãn ngang dòng cuối. 🔵 2026-10-29: `markChipFitsRightMargin` đo một lần mỗi khi câu có focus đổi (`focusedSegmentId`, `watch` với `flush: 'post'`), không theo aim, vì chuột đi qua câu khác chuyển aim mà nhãn của câu đang bấm vẫn hiện.
- Aim: chưa câu nào có focus thì rê chuột vẫn đặt aim cho `M` (câu được tô nền `:hover`); đã có câu được bấm thì chuột không lấy aim khỏi câu đó, vì nhãn đang hiện của nó là đích của `M` và Enter. Rời Chế độ đọc thì xoá `focusedSegmentId`, vì WebKit không phát `focusout` khi `<KeepAlive>` tháo node đang focus.
- Giới hạn đã biết: đổi cỡ cửa sổ hoặc thanh Tinh chỉnh trong lúc một câu đang có focus thì phải tới lần bấm câu kế mới đo lại.
- Bất ngờ: sự kiện chuột của WebDriver nhúng có `isTrusted: false` (đo bằng listener: mousedown/mouseup/click trúng `.segment-text` mà không có `focusin`), nên e2e không chứng minh được bấm ⇒ focus và vẫn gọi `node.focus()`. Bấm thật được đo trên Playwright WebKit 27.2 với HTML tối giản: chỉ rê chuột thì không focus, nhãn ẩn; bấm câu thì focus và nhãn hiện; bấm nút trong nhãn thì focus vào nút, nhãn vẫn hiện; bấm chỗ trống thì nhãn ẩn. Bấm vào khe giữa hai dòng của câu trúng `<p>`, không trúng câu.
- Bất ngờ: trong fixture này mọi dòng, kể cả dòng cuối, chạy hết bề rộng cột (mép phải 592), nên nhãn cũ nằm ở dòng 1 nhưng ngoài lề (x=608), diện tích giao = 0. Vì vậy assert diện tích giao không thể đỏ ở đây và đã bị bỏ; `onLastLine` là assert bắt được mã cũ.
- Lượt e2e đầu tiên sau build đỏ ở bước `M` lặp (3 dấu thay vì 2), sau phần đã sửa; chạy lại đúng binary hai lần đều xanh, lượt cuối cũng xanh. Không tìm nguyên nhân thêm.

## Plan Change Log

- 2026-10-28, Ice dùng thử: nhãn ở cuối câu rất khó bấm, vừa rê chuột là mất. Khe `margin-inline-start` giữa chữ cuối và nhãn, cùng phần dòng chữ cao hơn nhãn 24px, nằm trên câu kế tiếp, nên câu kế lấy mất câu đang chọn. Đo WebKit: chỉ 4/10 điểm (nửa trên, giữa) và 1/10 (đáy dòng) trên đường từ chữ cuối sang nhãn thuộc câu đang chọn. Bổ sung: `.mark-actions::before` trong suốt phủ khe và trọn `1lh` của dòng đọc, `z-index: -1` để không chặn cú bấm vào nút ⇒ 10/10 ở cả ba độ cao. GIỮ: vị trí nhãn theo phương án A.
- 2026-10-29, Ice: "đổi nhãn sang bên cạnh đi nhìn đẹp hơn", chọn lề phải ngang dòng cuối. Bổ sung nhánh lề phải đo bằng JS, kèm nhánh dự phòng khi lề hẹp; cầu `::before` nay dài theo `--mark-bridge`. Ca e2e đo cả hai nhánh (nhánh hẹp bằng cách tạm thu `.page`). Đối chứng: xoá `watch` ⇒ đỏ, nhãn nằm trước mép cột 17,8px; xoá `::before` ⇒ đỏ, 75 điểm rơi ra ngoài câu. GIỮ: cầu bắt chuột và vị trí dọc theo dòng cuối.

- 2026-10-29, Ice: "cho nhãn ẩn khi hover, chỉ hiển thị khi click chuột". Bỏ `:hover` và `.aimed` khỏi luật hiện nhãn, chỉ giữ `:focus-within`; bỏ cầu `::before` (nhãn không còn tắt khi rê chuột, còn cầu sẽ chặn cú bấm vào phần câu kế trên dòng cuối); phép đo lề theo câu có focus. Đối chứng: mã HEAD ⇒ đỏ dòng 132 (nhãn hiện sau `mouseenter`); xoá gán `focusedSegmentId` ⇒ đỏ dòng 159, nhãn nằm trước mép cột 17,8px (lượt gỡ đầu tiên đỏ ở bước hiện nhãn, chạy lại đúng phép gỡ thì đỏ đúng chỗ; nghi Vite nạp giữa hai lần ghi tệp, chưa chứng minh).

- 2026-10-29, Ice: "nhãn nên xuất hiện gần vị trí click, cho nó đổ bóng". `mousedown` trên `.segment-text` lưu điểm bấm theo toạ độ cột; khi câu có focus trùng điểm bấm, nhãn đặt dưới dòng bấm và kẹp trong cột, bấm lại chỗ khác trong câu đang focus thì nhãn đi theo. Đối chứng: gỡ `@mousedown` ⇒ đỏ dòng 203, nhãn cách dòng bấm 283px (nằm ở dòng cuối). GIỮ: đường Tab và hai ca đo lề phải / lề hẹp.

## Review Triage Log

- medium · patch — (lượt bấm-mới-hiện) bấm câu A rồi rê chuột qua B thì aim sang B: nhãn của A ghi "↵ sửa" mà M/Enter tác động B. Đã sửa: câu có focus giữ aim; vitest mới, đối chứng gỡ điều kiện ⇒ đỏ (13 thành `null`).
- low · patch — (lượt bấm-mới-hiện) `focusedSegmentId` không xoá khi rời mode nên phép đo lề có thể cũ. Đã xoá trong `onDeactivated`.
- low · patch — (lượt bấm-mới-hiện) đường `mouseenter` → aim mất ca canh. Ca vitest mới; gỡ `@mouseenter` ⇒ đỏ (12 thành `null`).
- low · patch — (lượt bấm-mới-hiện) câu chữ cũ trong plan này và plan Story 5-13 nói nhãn hiện khi hover. Đã ghi 🔵.
- low · reject — (lượt bấm-mới-hiện) assert phủ định sau `pause(100)` là đoán thời gian. Cùng quy ước 100 ms của `realClick`, và đã đỏ được trên mã HEAD.
- low · patch — ghi chú cũ nói ca e2e thu hẹp `.column`; mã thì không. Đã sửa ghi chú.
- low · patch — phần kiểm tay không có mục nợ có chủ. Vị trí không phụ thuộc đường hiện (chỉ đổi `display`) và e2e đo nó; phần kiểm tay đổi thành "nhìn trong `tauri dev` đang chạy" chứ không phải một việc chỉ người làm được.
- low · patch — ca mới không kiểm vế "ngay sau chữ cuối". Đã thêm `gapAfterLastGlyph` trong [0, 32).
- medium · patch — chưa chạy đối chứng đỏ. Đã chạy: đặt lại hai inset ⇒ đỏ ở `onLastLine` (dòng 153), vì nhãn nằm ở dòng 1; xem bất ngờ về diện tích giao ở trên.

## Verification

**Commands:**
- `npx wdio run e2e/wdio.conf.mjs --spec e2e/specs/story-5-13-reading-marks.e2e.mjs` (sau `cargo build --locked --features wdio`) -- expected: xanh. Đối chứng: đặt lại `inset-inline-start: 100%` và `inset-block-start: 0` ⇒ đỏ ở `onLastLine`.
- `npx vitest run tests/frontend/readingMarks.test.ts tests/frontend/readingModeImages.test.ts` -- expected: xanh, thoát 0.

**Manual checks (if no CLI):**
- Trong `tauri dev`, rê chuột lên một câu dài từ hai dòng trở lên: nhãn đứng sau dấu chấm câu ở dòng cuối. 🔵 2026-10-29: rê chuột thì nhãn không hiện; bấm vào câu thì nhãn hiện ở lề phải ngang dòng cuối (lề hẹp thì sau dấu chấm câu), bấm chỗ trống thì ẩn.
