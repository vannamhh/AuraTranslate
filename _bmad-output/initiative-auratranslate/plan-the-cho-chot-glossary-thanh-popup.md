---
title: 'Dải chờ chốt Glossary nổi thành popup, bỏ dòng xuất xứ/phân loại'
type: 'feature'
ticket: ''
created: '2026-11-16'
status: 'built'
baseline_revision: '948977cfacde413ee1b1eb5ccb93a102d9c5e902'
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

**Problem:** Dải chờ chốt bản dịch Glossary (FR114, `GlossaryConfirmStrip.vue`) chèn vào lưới dưới câu đang đứng, chỉ trong cột bản dịch, và đẩy các câu bên dưới xuống; dòng "xuất xứ: Quét khi nhập · phân loại: Khác" chiếm chỗ mà không giúp quyết định bản dịch.

**Approach:** Cho dải nổi thành popup đè lên các hàng bên dưới, cùng khuôn TM (7189c704) và Proofreader (3937fb24): thêm `glossary_confirm` vào `stripTrackFloats`, kiểu popup (bo góc, `--shadow-popover`, bỏ viền trên). Bỏ dòng xuất xứ/phân loại cùng hai khoá i18n và hai computed state chỉ nó dùng. Ice chọn "gọn hẳn" (2026-11-16): bỏ đoạn giải thích Smart RAG Injector; bỏ nhãn "Bản dịch" hiện trên ô (giữ tên truy cập cho trình đọc màn hình); dòng meta chỉ còn "{count} lần trong Tác phẩm" nằm trên hàng tiêu đề, không có số thì không in gì; câu gợi ý Esc rút thành "Esc để sau" cạnh nút. Bố cục: `國鎮  Tác phẩm này · 26 lần trong Tác phẩm` / ô nhập / `[Lưu] [Để sau]  Esc để sau`.

</frozen-after-approval>

## Implementation Notes

Oneshot: thay đổi ~80 dòng, có hai tiền lệ chép nguyên khuôn (`stripTrackFloats`, `.proofreader-strip--popup`).

- `GlossaryConfirmStrip.vue` có hai gốc (`.gcs-done` và `<form>`) có thể cùng render; trong `.strip-track-float > *` mỗi con bị `position: absolute; top: 0` nên hai gốc sẽ chồng lên nhau. Bọc cả hai trong một gốc `.gcs-popup` mang nền, bo góc và `--shadow-popover`; khi cả hai cùng hiện, một vạch 1px ngăn giữa.
- `confirmStripTermOrigin`/`confirmStripCategory` và hai trường `termOrigin`/`category` của `CurrentTarget` chỉ dòng đã bỏ dùng nên gỡ luôn; khoá `meta_pending`, `origin_clause`, `category_clause`, `rag_note` gỡ khỏi `vi.json`.
- Khi `occurrence_count` là `null` không in gì thay cho "chờ chốt bản dịch": trạng thái chờ chốt vẫn nằm trong `aria-label` của form.
- Đối chứng: gỡ `top === 'glossary_confirm'` khỏi `stripTrackFloats` ⇒ ca "floats the Glossary confirm strip…" đỏ `expected false to be true`.

## Review Triage Log

Quick: 3 low (patch), 2 false.
- low · patch — `.gcs-origin` nay chỉ tạo kiểu dòng ẩn TM, tên sai nghĩa: đổi thành `.gcs-note`.
- low · patch — thân `.gcs-popup` chưa thụt lề: thụt lại.
- low · patch — khung tiêu điểm vuông lòi khỏi góc bo của popup: form nhận `border-radius` khi `:focus-visible`.
- false — "thông báo đã chốt hiện như popup trong track không nổi khi Quick Add thắng": component chỉ mount trong strip track của `GridBlock.vue`, và `inlineStripAnchorSegmentId` trả `null` khi Quick Add thắng nên không có track.
- false — "đối chứng chỉ là lời khai": đã gỡ seam và chạy lại `stripAnchorSegment.test.ts` trong phiên, đỏ `expected false to be true`.

## Verification

**Commands:**
- `npx vitest run tests/frontend/stripAnchorSegment.test.ts tests/frontend/glossaryConfirmStripTmOptions.test.ts tests/frontend/glossaryConfirmStripTemplate.test.ts tests/frontend/glossaryConfirmStrip.test.ts tests/frontend/glossaryConfirmStripUndo.test.ts tests/frontend/gridPanelStripTrack.test.ts tests/frontend/controls-adoption.test.ts` -- expected: xanh, mã thoát 0
- `npm run check:i18n` và `npm run check:tokens` -- expected: xanh
- Một spec e2e: `e2e/specs/story-12-20-glossary-unconfirm.e2e.mjs` -- expected: xanh (vị trí dải đổi)

**Manual checks:**
- Trong app thật: đứng ở câu có thuật ngữ chờ chốt ⇒ popup đè lên các câu dưới, rộng cột nguồn + cột bản dịch, không còn dòng xuất xứ/phân loại.
