---
title: 'Lượt ghi không qua gõ phím hiện ra trong ô đã gõ ở phiên này'
type: 'bugfix'
ticket: ''
created: '2026-10-27'
status: 'built'
baseline_revision: 'b0a8e00b3c09c30fd74b6918e4e17cd0a89694dd'
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

**Problem:** Ô bản dịch đã được gõ trong phiên này (kể cả gõ rồi xoá trắng) không hiện văn bản do một lượt ghi không qua gõ phím đặt vào: "Đưa sang bản dịch", khôi phục lịch sử, nhận soát lỗi, nhận bản duyệt, nhận TM. `replaceEditorSegment` chỉ cập nhật `segments`, còn `editedText` vẫn giữ chữ cũ và `GridPanel.restoreEditedText` chép chữ cũ đó đè lại vào ô. Ice thấy ô trống; bấm "Đưa sang bản dịch" lần nữa thì Rust hỏi vì đĩa đã có bản AI chưa ký, và "Ghi đè" hay "Giữ bản đang soạn" đều đóng câu hỏi mà ô vẫn trống, nên trông như hai nút không có tác dụng. Console của Ice: `edited: [[8, ""]]`, `dom: ""`, `state: "Trong năm 1878…"`, `pending: null`.

**Approach:** Khi `replaceEditorSegment` nhận `target_text` cho một câu đã có trong `editedText`, nó thay chữ đang gõ của câu đó bằng chữ vừa ghi (kèm hình dạng `editedShape`). Lưới chép chữ đó vào ô, kể cả ô đang giữ tiêu điểm, miễn là không đang soạn IME; caret đặt cuối ô.

</frozen-after-approval>

## Implementation Notes

Oneshot: một tệp nguồn, nguyên nhân đọc từ Console app thật của Ice (`edited: [[8, ""]]`).
- Chỉ thay chữ đang gõ khi câu không còn trong tập chờ flush: chữ chưa xuống đĩa vẫn thắng, đúng ca cũ của `gridRestoreEditedText.test.ts`.
- Bỏ thay đổi ở `GridPanel.restoreEditedText` (cho ghi cả ô có tiêu điểm): Vue tự gán `textContent` khi `target_text` đổi, đối chứng gỡ nó vẫn xanh nên nó là phần thừa. Vế "ô có tiêu điểm" của Approach do Vue lo, không có ca riêng: một ca như thế xanh ở cả hai nhánh.
- Ca test xoá trắng ô bằng `replaceChildren()`, giả định WebKit gỡ text node khi xoá hết chữ; giả định này chưa được đo riêng.
- Vế "caret đặt cuối ô / không đè lúc soạn IME" của Approach không được làm: nó thuộc thay đổi `GridPanel` đã bỏ, đường cũ không chạm ô có tiêu điểm.
- Đối chứng webview thật (spec dò tạm, gõ bằng `execCommand`, rồi xoá): gỡ nhánh ⇒ ô trống sau lượt Đưa sang thành công; có nhánh ⇒ ô hiện chữ AI.

## Verification

**Commands:**
- `npx vitest run tests/frontend/gridRestoreEditedText.test.ts` -- expected: xanh; gỡ nhánh mới ở `replaceEditorSegment` thì ca "ô đã xoá trắng" đỏ
- `npm run test:story` không áp dụng (không có ticket); chạy thêm các tệp vitest đang canh `restoreEditedText`/`replaceEditorSegment`

**Manual checks:**
- Spec dò e2e tạm (gõ rồi xoá trắng ô, "Đưa sang bản dịch") cho ô hiện chữ AI ngay lần đầu

## Review Triage Log

Lượt quick: 0 high · 0 medium · 3 low · 1 false · 2 maybe-false ⇒ low.
- low · patch — chú thích mới ba dòng và nói lưới chép đè mọi ô; nay hai dòng, ghi "unfocused cells" (`GridPanel.restoreEditedText` bỏ qua ô có tiêu điểm).
- low · patch — ca "ô đang giữ tiêu điểm" xanh cả khi gỡ nhánh mới (đo lại trên mã cuối: chỉ ca "ô đã xoá trắng" đỏ); đã xoá ca đó cùng docstring sai của `clearByTyping`.
- low · patch — dòng Verification ghi `<tệp test mới>`; nay trỏ đúng `gridRestoreEditedText.test.ts`.
- low · không làm, báo Ice — Approach (khối đóng băng) còn câu "caret cuối ô, không đè lúc soạn IME" mà mã không làm; hành vi người dùng thấy (ô có tiêu điểm hiện chữ mới) do Vue lo, đường cũ chưa từng ghi vào ô có tiêu điểm nên không có hồi quy IME. Chỉ Ice sửa được khối đó.
- maybe-false — ô có tiêu điểm phụ thuộc Vue gán `textContent`; ca jsdom xanh ở cả hai nhánh cho thấy Vue có gán, phần WebKit chưa đo. Cần một lượt Ice dùng thật trên ô đang có caret để chốt.
- false — chữ chưa flush lệch với `segments`: hành vi cũ, mọi chỗ gọi `replaceEditorSegment` có `target_text` đều flush trước (`promoteAiTranslationToEditor`, `restoreVersion`, TM, soát lỗi, duyệt).

