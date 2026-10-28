---
title: 'Lưu xong không làm mục đang hiện sụp về "Đang tải…"'
type: 'bugfix'
ticket: ''
created: '2026-10-28'
status: 'built'
baseline_revision: '1708a7befd8ca4e1c4e2e44b3f17c273cd2ec275'
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

**Problem:** Bấm "Lưu khoá" (và mọi lượt lưu khác trong mục AI của Cài đặt) làm màn hình giật. Nguyên nhân: lượt đọc lại sau khi ghi đi qua `loadAiConfigSection()`, hàm này bật `loading = true`, và `SettingsOverlay.vue:427` thay cả mục bằng dòng "Đang tải…". Nội dung bị gỡ ra rồi dựng lại, nên vị trí cuộn nhảy và tiêu điểm mất. Thư viện prompt bị cùng lỗi: `loadPromptSets()` chạy lại sau Tạo/Đổi tên/Sửa thân/Xoá/Thêm mẫu/Khôi phục, và `PromptLibraryOverlay.vue:641/674` ẩn thanh phạm vi, danh sách cùng ô soạn.

**Approach:** Mỗi state có hai đường đọc. Đường **nạp** (vào mục, mở lớp phủ, panel mount) giữ dòng "Đang tải…" như cũ. Đường **đọc lại** sau một lượt ghi thành công không bật cờ loading, nên dữ liệu đang hiện ở nguyên chỗ tới khi kết quả mới đến. Cả hai dùng chung bộ đếm `sequence`, và lượt nào thắng thì lượt đó tắt `loading`.

</frozen-after-approval>

## Implementation Notes

- Oneshot: hai tệp state, khoảng 30 dòng. Không đổi template và không đổi IPC.
- Rà 13 module state còn lại có bước chuyển sang loading/unknown (export, alignment, reviewMode, proofreadDecision, glossaryManage, tmManage, importPreview, bilingualImportPreview, tmImport, glossaryImport, promptSetImport, reviewerImport, glossaryHarvest). Không còn chỗ nào hội đủ ba điều kiện: cờ loading, đọc lại sau ghi, template thay nội dung. `ExportOverlay.vue:552` đã có chặn `exportPreviewData === null`.
- Phương án bị bác: chỉ hiện "Đang tải…" khi chưa nạp lần nào. Phương án này vẽ dữ liệu của Tác phẩm cũ khi vào lại mục sau khi đổi Tác phẩm, tức là một màn hình nói dối trong im lặng.
- `restorePromptSetForWork` (Khôi phục khi mở một Tác phẩm) cố ý ở lại đường nạp: dữ liệu đổi sang Tác phẩm khác. Intent liệt kê nhầm nó trong số đường bị lỗi.
- Đối chứng đỏ: đưa cả 7 lời gọi đọc lại của `aiConfigState.ts` và 5 của `promptSetState.ts` về hàm nạp ⇒ 12/12 hàng bảng đỏ vì `loading === true`. Hai ca render đỏ khi gỡ riêng `saveAiConfigKey`/`renamePromptSet`.
- Vitest dưới tải 115–159 (Blender ~820% CPU): ca đầu mỗi tệp hết giờ 5 s ở bước import nguội. Với `--testTimeout=60000` thì 249/249 xanh.
- Ice chọn cách sửa phát hiện review đầu: lượt đọc lại chỉ làm mới draft và ô Tự nhập của các trường lượt ghi đã chạm. Các trường đó cộng dồn trong `writtenSinceRead` qua mọi lượt bị `sequence` vượt mặt. Đối chứng: trả lại kiểu ghi đè toàn bộ ⇒ ca "gõ ô Model" đỏ; bỏ cộng dồn ⇒ ca "bị vượt mặt" đỏ.

## Review Triage Log

- medium · HALT → Ice chọn "chỉ làm mới ô vừa ghi", đã vá: `reloadAiConfigSection` vẫn ghi đè `drafts` và `closeCustom()`. Trước đây mục bị ẩn nên không gõ được. Giờ chữ gõ vào ô khác trong lúc đọc lại bị thay im lặng. Thư viện prompt không dính: `nameDraft`/`bodyDraft` là ref cục bộ, lượt đọc lại không gán.
- medium · patch: mới có 2/12 chỗ gọi được canh ⇒ thêm bảng `it.each` vào `aiConfigState.test.ts` (7 hàng) và `promptSetState.test.ts` (5 hàng), cộng một ca giữ cờ khi nạp.
- low · ghi chú: Intent ghi "Khôi phục" là đường bị lỗi. Đó là đường nạp khi mở Tác phẩm, giữ nguyên (xem ghi chú trên).
- low · patch: doc comment của `deletePromptSet` còn trỏ `loadPromptSets` ⇒ đổi thành `reloadPromptSets`.
- low · patch: doc comment viết lại của `loadPromptSets` phạm luật comment (tiếng Việt, 5 dòng) ⇒ thay bằng một dòng tiếng Anh.

## Verification

**Commands:**
- `npx vitest run tests/frontend/settingsOverlayAiConfigKeyRender.test.ts tests/frontend/promptLibraryOverlayRender.test.ts tests/frontend/aiConfigState.test.ts tests/frontend/promptSetState.test.ts`. Kỳ vọng: xanh. Ca mới giữ lượt `aiConfigGet`/`promptSetList` sau ghi ở trạng thái treo, rồi khẳng định không có dòng "Đang tải…" và nút DOM của mục vẫn là cùng một node.
- Đối chứng đỏ: tạm đưa lời gọi đọc lại về hàm nạp. Hai ca mới phải đỏ đúng vì lý do đó.
