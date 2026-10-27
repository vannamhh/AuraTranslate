---
title: 'Tên Tác phẩm đang mở trên thanh tiêu đề'
type: 'feature'
ticket: ''
created: '2026-10-27'
status: 'built'
baseline_revision: '303404c379553eb6415924755988476ed671b3e7'
route: 'oneshot'
route_source: 'auto'
risk: 'low'
review: 'quick'
review_source: 'pinned'
lenses_ran: ['quick']
review_loop_iteration: 0
context:
  - '{project-root}/src/AGENTS.md'
---

<frozen-after-approval reason="human-owned intent — do not modify unless human renegotiates">

## Intent

**Problem:** Ở Workspace không chỗ nào cho biết đang làm Tác phẩm nào; người dùng phải về Library mới biết (Ice, 2026-10-27).

**Approach:** Thanh tiêu đề hiện `meta.name` của Work đang mở. Tên lấy từ chính lượt mở/tạo Work đã thành công (`openWorkById`, `finishImportSubmission`), giữ cạnh `openWorkId` trong `src/openWorkIdentity.ts`; chưa mở Work nào trong phiên ⇒ không hiện gì. Tên dài cắt bằng dấu ba chấm, đủ tên trong `title`.

## Decisions (Ice, 2026-10-27)

1. Hiện ở Workspace và Chế độ đọc; Library không hiện (đã có trang Tác phẩm).
2. Đặt ngay sau ba tab chế độ, chiếm phần trống trước cụm bên phải; co lại và cắt ba chấm, không đè lên dòng trạng thái hay nút Bố cục/Công cụ.

</frozen-after-approval>

## Implementation Notes

Route oneshot: khoảng 80 dòng ở 6 tệp. App chưa có lệnh đổi tên hay đóng Work, nên tên chỉ đổi ở hai lượt mở/tạo; `resetOpenWorkId` chỉ test gọi.

- `setOpenWorkId(id)` đổi thành `setOpenWork(id, name)` để id và tên không thể lệch nhau; `openWorkName` cạnh `openWorkId`. `titlebarWorkName` (`src/titlebarWorkName.ts`) trả `null` ở Library và khi tên trống.
- `App.vue`: `<span class="titlebar-work">` giữa `</nav>` và `.titlebar-right`, vạch ngăn trái, `min-width: 0` + ellipsis; `.titlebar-right` thêm `padding-left` để tên dài không dính cụm phải.
- `isOpenedWork` không kiểm `meta.name` trên dây; Rust `WorkMeta.name: String` luôn có. Thêm kiểm sẽ làm đỏ các fixture mở Work không có `name`, nên không làm ở đây. Không phía nào chặn tên rỗng (`sanitize_name` chỉ thay tên thư mục), nên tên trống coi như không có.
- Đối chứng: gỡ dòng `setOpenWork` ở `libraryChapters.ts` ⇒ 2 ca đỏ (mở từ Library, nhập lỗi giữ tên); gỡ ở `libraryImport.ts` ⇒ ca nhập đỏ; gỡ vế `name.trim() === ''` ⇒ ca tên trống đỏ.
- 🔵 2026-10-27 (Ice xem app thật): tên thấp hơn chữ tab 1px vì `.mode-tab` có gạch chân trong suốt 2px; `.titlebar-work` nay mang cùng `border-bottom: 2px solid transparent` (`margin-bottom: 2px` bị Kiểm K lưới 4px chặn). Bỏ khung ngoài của cụm nút Bố cục (`.layout-switch { border-width: 0 }`), vạch giữa hai nút giữ.
- Chưa thử trong app thật: tên hiện, cắt ba chấm khi cửa sổ hẹp, kéo cửa sổ qua vùng tên trên macOS. Mục nợ Chủ: Epic 12.

## Review Triage Log

Lượt 1 (quick): high 0 · medium 0 · low 2 · false 1 · maybe-false 0.

| Verdict | Route | Finding | Evidence |
|---|---|---|---|
| low | patch | Tên rỗng/chỉ khoảng trắng hiện vạch ngăn trơ trọi | Đúng: `sanitize_name` chỉ thay tên thư mục, `meta.name` giữ nguyên; `titlebarWorkName` nay trả `null`, thêm ca test |
| low | reject | Ca `App.vue` chỉ quét thứ tự văn bản, không canh ellipsis/`title`/không đè | CSS không đo được trong vitest; đưa vào mục nợ kiểm app thật Chủ: Epic 12 |
| false | — | Kiểm tay chưa ghi là đã làm | Đúng ý định: plan `built`, không `done`; mục nợ Chủ: Epic 12 giữ kiểm tay |

## Verification

**Commands:**
- `npx vitest run` các tệp test đã sửa/thêm -- expected: xanh, mã thoát 0, không dòng Errors.
- `npm run check:i18n && npm run check:tokens && npm run check:commands && npm run check:lint` -- expected: xanh.

**Manual checks:**
- App thật: mở một Work ở Library, sang Workspace, thấy tên trên thanh tiêu đề; tên dài cắt ba chấm, rê chuột thấy đủ tên.
