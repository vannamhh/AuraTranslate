---
title: 'Trả 15 mục nợ có chủ Story 2.4'
type: 'chore'
ticket: ''
created: '2026-10-20'
status: 'built'
baseline_revision: 'bf3f47f48e11854ae1fdb102d2310e9a1619dbcd'
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

**Problem:** 15 mục mở trong `deferred-work.md` có `Chủ:` cuối là Story 2.4 (`:248 :261 :288 :769 :814 :2050 :2125 :2339 :2363 :2370 :2484 :2631 :2752 :3005 :4675`, số dòng ở `bf3f47f`). Story 2.4 (mũi thăm dò NFR2+NFR18, `in-progress`) đang chờ lượt đo thật của Ice. Một số mục trỏ vào mã đã xoá hoặc đã có số đo ở chỗ khác, nhưng vẫn ghi là mở.

**Approach:** Mỗi mục nhận một dòng `→ … 2026-10-20`. Có ba dạng đóng (`✅ ĐÃ ĐÓNG` · `🟡` · `KHÔNG LÀM`) khi bằng chứng có sẵn cho phép. Các mục khác nhận `🔵` ghi trạng thái hôm nay và giữ `Chủ: Story 2.4`.

**Decisions (Ice 2026-10-20):**
- Phạm vi chỉ là sổ: không đo lại, không vá đường dời con trỏ (`:3005`/`:4675`). Ngoại lệ duy nhất là `:2363`.
- `:288`: ghi 🔵, giữ Story 2.4.
- `:2363`: vá ngay. `restoreEditedText` duyệt tập chờ thay vì quét cột bản dịch, có ca vitest.
- `:814`: vế đọc theo khối giao `Chủ: Epic 6`.

**Never:** Không chạm nợ của chủ khác. Không đánh dấu Story 2.4 là `done`, không sửa plan của nó. Không đổi `Tuning` hay hằng nhịp flush.

</frozen-after-approval>

## Implementation Notes

- Phân loại theo mã ở `bf3f47f`:
  - ✅ `:261` (lưới sáu điểm: đỉnh WAL ~754 KB), `:2339` (sửa câu ở `2-2-ban-do-editor.html`), `:2363` (vá), `:2752` (`6d68dce` cộng nightly `39161292771`, 31/31 tệp).
  - KHÔNG LÀM `:2050`, `:2125`, `:2370`, `:2631`: mã đã xoá ở `ca33072`. Phần chi phí còn sống do `:3005` giữ.
  - 🟡 `:814`: số RSS đã đo ở 6.18, nay thuộc 10.9. Đọc theo khối giao Epic 6 (epic còn `in-progress`).
  - 🔵 `:248`, `:288`, `:769`, `:2484`, `:3005`, `:4675`.
- `check:debt-owner` hạ một dòng `✅` xuống 🟡 khi cùng dòng có cụm "còn mở" (`HALF_ON_CLOSED_LINE_RE`). Dòng của `:261` lúc đầu dính lỗi này nên đã viết lại.
- Đối chứng của `gridRestoreEditedText.test.ts`:
  - Đưa lại vòng `querySelectorAll` cũ: ca quét đỏ (1 lượt quét `[data-segment-id]`), ca hành vi xanh.
  - Gỡ hẳn vòng khôi phục: cả hai ca đỏ (`'bản trên đĩa'` thay chữ chờ).
- Chú thích 🔵 phía trên `restoreEditedText` nêu tên story và trích sổ nợ, sai luật comment, nên đã xoá trong dòng đổi.
- Chưa đo thời gian của bản vá. Fixture chỉ có 3 câu.

## Review Triage Log

Lens quick: 1 medium · 2 low · 3 false.
- medium, patch: dòng ✅ của `:2752` không nêu quần thể. Nightly `39161292771` có 31 tệp ở `fe08b04`, chưa có spec 12-x; bộ 49 tệp chưa có nightly xanh. Dòng sổ nay nêu cả hai.
- low, patch: dòng ✅ của `:2363` không nêu dạng chi phí mới. Đã thêm "k lượt `querySelector`, k = số câu đang chờ ghi".
- low, bỏ qua: ca quét chỉ canh selector `[data-segment-id]` nguyên văn. Muốn rộng hơn phải tách lượt `querySelectorAll` của `repaintProofreadHost`, đắt hơn một lượt sửa đơn giản.
- false: ô trùng id trong cột bản dịch. Template dựng đúng một `.cell-tgt` mỗi `s.id` (`GridPanel.vue`, `v-for` cột ④).
- false: luật comment không áp cho tệp bàn đo HTML; câu bị xoá là câu sai.
- false: frontmatter đang review là đúng trạng thái giữa chừng.

## Verification

- `npx vitest run tests/frontend/gridRestoreEditedText.test.ts tests/frontend/editorTypingZone.test.ts tests/frontend/editorChapterSwitch.test.ts tests/frontend/proofreadWiring.test.ts`: 52/52 xanh.
- `npx vue-tsc --noEmit`, `npx eslint` trên hai tệp đổi, `npm run check:debt-owner`: xanh.
