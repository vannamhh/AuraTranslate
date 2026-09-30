# Sprint Change Proposal — 2026-09-30 · Story 11.8 sửa lỗi ranh giới

**Người soạn:** `bmad-correct-course`, chế độ Batch · **Người duyệt:** Ice
**Baseline:** `abcb6b0` (master, cây sạch)
**Nguồn:** `epic-11-retro-2026-09-30.md` — §Việc cần làm V-1, V-2, V-5, V-9, V-10; Ice chọn "story mới 11.8" (Q-2) và phạm vi trong phiên retro
**Trạng thái:** Ice duyệt 2026-09-30; đã thi hành.

---

## 1. Vấn đề

Retro Epic 11 đọc chéo các story và tìm ra những lỗi nằm ở **chỗ giao giữa hai story**. Mỗi story/lô chạy trong một phiên riêng, nên không phiên nào thấy cả hai phía. Lượt review ba lớp của từng story không bắt được chúng, và `check:debt-owner` vẫn xanh vì các lỗi này chưa từng vào sổ.

Bằng chứng chính:
- **Nút Quét lại kẹt đến khi khởi động lại** (F-W-1, cao): `loadLibraryOrphans` của 11-6 lô A tăng chung bộ đếm với ba lượt ghi của Library, còn ba lượt đó thoát sớm trước khi nhả `rescanBusy` (`src/modes/libraryRescan.ts:143-146`, `:224-227`).
- **Nightly e2e đỏ hai tầng** (F-B-1, cao): 11-3 (`4db199c`) đổi `list_dict_sources` từ mảng sang struct, trong khi `attribution-focus.e2e.mjs` mà 11-2 vừa dựng lại vẫn hỏi `Array.isArray`. Trên CI, lỗi này bị che vì `ensureFullLayoutTier()` đỏ trước. Lượt e2e trên máy Ice 2026-09-30 ra 28/29.
- **Một lượt webview gửi sai khoá cả Tác phẩm** (F-R-1, trung): `confirm_segment` ghi origin chưa kiểm, trong khi `open_work` từ chối mở Tác phẩm có origin lạ (`commands/segment.rs:2522-2530`).
- Cùng loại: F-W-2, F-R-2, F-W-3, F-R-4, F-W-8, F-D-9/F-R-8; guard chỉ canh hàm, không canh dây (F-D-1, F-D-8); cảnh báo pre-push không đọc lượt `schedule` (F-CI-4).

## 2. Phân tích tác động

| Tài liệu | Tác động |
|---|---|
| `epics.md` | Thêm khối Story 11.8 sau Story 11.7; thêm một dòng 🔵 vào §Epic 11 "Ghi chú cài đặt". |
| `sprint-status.yaml` | Thêm khoá `11-8-sửa-lỗi-ranh-giới-giữa-các-story: backlog` sau `11-7`. `epic-11` giữ `in-progress`; `epic-11-retrospective` giữ `done`. |
| `deferred-work.md` | Thêm một tiêu đề với 21 mục mới `Chủ: Story 11.8` (8 mục V-2, 1 V-1, 4 V-5, 1 V-9, 7 V-10). Không sửa mục cũ. |
| PRD · UX · `build-sequence.md` · Epic List | Không đổi. Thứ tự epic `… 4 → 11 → 7 …` giữ nguyên; 11.8 nằm trong Epic 11. |
| Spine | Không đổi. F-R-1 **cưỡng chế** hàng "Xuất xứ bản dịch" (FR117, AD-47) đã có, không đổi bất biến. |

**Thứ tự trong Epic 11:** 11.8 → lượt dùng thật của Ice cho các mục `Chủ: Epic 11` (V-8) → `epic-11` lên `done` → Epic 7. Làm 11.8 trước V-8 để bốn mục kiểm tay dây `AppHandle` có thể thành ca tự động (F-D-9), và để lượt dùng thật chạy trên mã đã vá.

**Cái giá:** Epic 7 mở muộn thêm một story.

## 3. Quyết định của Ice (2026-09-30)

| # | Quyết định | Ice chọn |
|---|---|---|
| 1 | Đường cho V-2 | **Story mới 11.8** trong Epic 11 |
| 2 | Kéo thêm vào 11.8 | **V-1** (nightly), **V-5** (guard canh dây), **V-9** (cảnh báo đọc `schedule`) |
| 3 | Các phát hiện *hoãn* (V-10) | **`Chủ: Story 11.8`**, được chốt KHÔNG LÀM theo AC chung Epic 11 |
| 4 | Chế độ duyệt | Batch |

## 4. Đề xuất sửa chi tiết

### 4.1 `epics.md` — khối Story 11.8 (chèn sau Story 11.7)

Khối đã chèn nguyên văn vào `epics.md` §Story 11.8. Tóm tắt:
- Story khuôn *As a chủ dự án…*, thừa kế AC chung Epic 11 như 11.1–11.7.
- Ba AC riêng, mỗi cái khoá một hành vi người dùng thấy hoặc một cổng:
  - ① Quét lại, Chọn thư mục, Gỡ mồ côi dùng lại được sau khi rời/quay lại Library giữa lượt.
  - ② `confirm_segment` từ chối origin ngoài danh mục FR117 và không ghi gì.
  - ③ Một lượt e2e `workflow_dispatch` trên commit cuối xanh, trong đó `attribution-focus` thật sự đo AC11 Story 1.19; nếu đỏ, có dòng lý do nêu run id.

§Epic 11 "Ghi chú cài đặt", thêm:
`- 🔵 2026-09-30: thêm Story 11.8 — lỗi ranh giới do chính các story Epic 11 sinh ra, tìm thấy ở retro (`epic-11-retro-2026-09-30.md`); chạy trước lượt dùng thật của Ice và trước Epic 7.`

### 4.2 `sprint-status.yaml`

Chèn một dòng sau `11-7-…: done`:
`  11-8-sửa-lỗi-ranh-giới-giữa-các-story: backlog`

### 4.3 `deferred-work.md`

Tiêu đề mới `## Deferred from: retro Epic 11 → correct-course Story 11.8 (2026-09-30)` ở cuối tệp, 21 mục theo khuôn `source_spec / summary / evidence … Chủ: Story 11.8.`, mỗi mục ≤ 5 dòng.

| Nhóm | Mục |
|---|---|
| V-2 (8) | F-W-1 · F-W-2 · F-R-1 · F-R-2 · F-W-3 · F-R-4 · F-W-8 · F-D-9 + F-R-8 |
| V-1 (1) | F-B-1 + F-CI-2 |
| V-5 (4) | boundary_scan ép 18 tệp · vỏ `promote_ai_translation` qua MockRuntime · EXPLAIN QUERY PLAN đọc SQL thật · ca ③ hanviet |
| V-9 (1) | `ci-previous-verdict.mjs` đọc lượt `schedule` |
| V-10 (7) | F-R-3 (đo trước) · F-R-5 + F-A-4 · F-R-6 · F-W-4 · F-W-6 · F-W-7 · F-W-9 + F-A-3 |

Năm lượt `workflow_dispatch` của 11.2 giữ nguyên ở ba mục `Chủ: Epic 11` hiện có; mục V-1 chỉ trỏ tới chúng.

## 5. Bàn giao

**Phạm vi:** Minor–Moderate — thêm một story trong epic đang mở, không xếp lại epic. Thi hành ngay sau khi Ice duyệt: sửa ba tệp ở §4, chạy `check:debt-owner`, commit riêng. Story 11.8 soạn bằng `create-story` (Task 0 đọc lại 21 mục trên HEAD), cài bằng `bmad-build`.

**Tiêu chí thành công:**
- `check:debt-owner` xanh sau khi thêm khoá và 21 mục.
- `grep -c 'Chủ: Story 11.8' deferred-work.md` = 21.
- Sau khi thi hành, các action item retro V-1, V-2, V-5, V-9, V-10 chuyển `in-progress` (Ice xác nhận).
