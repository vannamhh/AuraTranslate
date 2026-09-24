# Sprint Change Proposal — 2026-09-24b · Nợ đứng tên Ice ngoài lượt rà

**Người soạn:** John (PM), `bmad-correct-course` chế độ Batch · **Người duyệt:** Ice
**Baseline:** `b8f22f7` (master, cây sạch)
**Nguồn:** `sprint-change-proposal-2026-09-24-epic-11-tra-no-nen.md` §1 — *"239 mục ngoài lượt rà không thuộc phạm vi đề xuất này"*
**Trạng thái:** Ice duyệt 2026-09-24; đã thi hành trọn.

---

## 1. Vấn đề

Đo bằng bộ phân tích của `check-debt-owner.mjs` trên `b8f22f7`: **239** mục mở/🟡 có `Chủ:` cụ thể cuối cùng là Ice và không mang dòng `(rà sổ nợ)` (209 mở · 30 🟡). Con số trùng khít trên `e3aea77`.

Khác lô 210 của Epic 11, lô này không phải "chủ mơ hồ": `Chủ: Ice` ở đa số mục là chủ đúng, vì mục chờ một quyết định hay một lượt kiểm tay chỉ Ice làm được. Vấn đề thật là không có thời điểm nào lên lịch cho Ice quyết, nên chúng tích lại. Và đuôi *"Chủ: Ice — quyết định hình dạng nghiệm thu tay B10/F8 … mục này chờ B10"* bị dán hàng loạt, kể cả lên mục đã vá xong.

## 2. Phân loại

Bốn agent chia 239 mục, mỗi mục đọc một lượt, đối chiếu mã HEAD cho hạng C và E. John kiểm lại cả 5 mục E trên mã và chỉnh 5 mục C (§3).

| Hạng | Nghĩa | Số |
|---|---|---|
| A | Quyết định chỉ Ice ra được (12 mục tự khai trùng quyết định với mục khác) | 162 |
| B | Kiểm tay/đo trên máy thật | 30 |
| C | Việc mã/test một story chưa done làm được | 23 |
| D | Treo điều kiện chưa xảy ra | 13 |
| E | Đã tự đóng trên HEAD | 5 |
| F | Đòi đổi bất biến kiến trúc | 6 |

## 3. Quyết định của Ice (2026-09-24)

| # | Câu hỏi | Ice chọn |
|---|---|---|
| 1 | Hạng A | **Lai:** câu sản phẩm/UX/quy trình quyết ngay với John; câu kỹ thuật hai phương án giao story, Task 0 trình số đo |
| 2 | Hạng B | **Chốt AI-7 trước** (hình dạng nghiệm thu tay, retro Epic 3); mục B đi theo lựa chọn đó, mục agent làm được tách ra làm luôn |
| 2′ | AI-7 / B10 | **Lượt dùng thật cuối mỗi epic**, không hàng tay mỗi story |
| 3 | E · C · F · D | **Thi hành Batch, commit riêng** |

## 4. Bước 1 — đã thi hành

Nối một dòng `→ … (xếp nợ đứng tên Ice)` vào 34 mục của `deferred-work.md`:

- **E, 5 mục:** `→ ✅ ĐÃ ĐÓNG` kèm chỗ đứng của bản vá trên HEAD.
- **C, 21 mục:** giao Story 10.1 · 10.4 · 10.9 · 11.1–11.7. John chỉnh so với phân loại: hai mục số đo NFR1 sang 10.9 (cùng họ NFR1 đầu-cuối đã ở 10.9); `editorPromoteAiTranslationError` sang 11.5 (cùng đường `promote_ai_translation`).
- **F, 8 mục → Winston:** 6 mục hạng F, cộng hai mục luật `is_omitted` khi gộp chưa vào spine, vốn phân loại C nhưng ghi spine là việc của Winston. Ba mục cùng một ràng buộc (`dispatch()` không tham số, AD-34); mục hoàn tác Glossary cần đối chiếu AD-49 trước.
- **D, 13 mục:** giữ `Chủ: Ice`, không sửa. Đúng AC chung của Epic 11 cho mục treo điều kiện.

Danh sách từng mục: `grep -n 'xếp nợ đứng tên Ice' deferred-work.md`. Sau bước 1: **205** mục mở/🟡 đứng tên Ice; `check:debt-owner` xanh.

## 5. Bước 2 — đã thi hành

26/30 mục B đến từ story đã `done` hoặc đang `review`, nên hàng tay mỗi story không phủ được lô tồn; Ice chọn lượt dùng thật cuối mỗi epic.

- **25 mục → `Chủ: Epic 4`**: lượt dùng thật cuối Epic 4. Kiểm C đỏ nếu `epic-4` lên `done` khi còn mục mở, nên lịch có cổng giữ.
- **1 mục → `B7`**: đối chứng scope asset protocol đòi cả Windows.
- **2 mục đóng** bằng đọc CI: tám (không phải bảy) lượt đêm Windows in `984712 B`, run ID ghi vào `spec-ca-wal-do-tren-windows.md`; lượt CI đầu tiên chứa Story 4.7 (`35436773193`) xanh cả hai nền.
- **2 mục giữ `Chủ: Ice`**: sửa môi trường máy Ice (`rustup` target Windows, `brew reinstall merve`).
- AI-7 và B10 trong `sprint-status.yaml` sang `done`; luật ghi một dòng ở `AGENTS.md` §Tests.

Sau bước 2: **177** mục mở/🟡 đứng tên Ice.

## 6. Bước 3 — hạng A

Một agent tách 162 câu: **119 P** (hành vi người dùng thấy, UX, phạm vi, chính sách quy trình, tài liệu đã ký) · **43 K** (lựa chọn cài đặt người dùng không thấy khác, phép đo trong story chọn được). Câu chạm tài liệu đã ký, ý định một AD hay chính sách test được xếp P dù nghe kỹ thuật.

- **K, đã thi hành:** 43 mục → Story 10.9 · 11.1 · 11.2 · 11.3 · 11.5 · 11.6 · 11.7; Task 0 của story trình phương án kèm số đo cho Ice. Sau đó: **134** mục đứng tên Ice. Tải Epic 11 (`grep -c 'Chủ: Story 11.N.**'`): 49 · 34 · 35 · 33 · 33 · 50 · 23 — 11.1 và 11.6 nên tách đôi lúc `create-story`.
- **P, đã thi hành:** Ice trả lời đủ 108 câu của `sprint-change-proposal-2026-09-24b-phieu-quyet.md` trong phiên 2026-09-24: 40 KHÔNG LÀM · 53 giao story/người · 14 đóng · 1 giữ Ice kèm điều kiện. Mỗi câu nối một dòng `(phiếu quyết #N)` vào mục của nó; câu trả lời nằm ở cột *Quyết* của phiếu.

**Tiêu chí thành công:** `check:debt-owner` xanh; mỗi mục còn `Chủ: Ice` thuộc một trong ba loại: câu trong phiếu chưa trả lời, mục D có điều kiện ghi rõ, hoặc việc môi trường trên máy Ice.

## 7. Sửa tài liệu quy hoạch trong phiên quyết

Correct-course đi kèm câu trả lời, mỗi chỗ mang một dòng 🔵 tại chỗ:

- FR115 bỏ bảng `.md` (`prd.md`, `epics.md` §FR list và AC1 Story 6.16); thêm **Story 6.16c** (bảng `.docx` song ngữ) vào `epics.md` và `sprint-status.yaml`, chạy cùng đợt Epic 11 sau Epic 4 — #22, #23.
- UX-DR19 rút giá trị về hưu (`epics.md`, `DESIGN.md`, `EXPERIENCE.md`) — #65.
- UX-DR15 và `DESIGN.md` frontmatter: titlebar 40px, status 34px; mockup `key-screen-workspace.html` — #77, #80.
- Mockup `data-integrity.html` theo AD-31; mockup `web-import.html` và AC Story 6.10 sang `⌥⌘↵` — #79, #104.
- Covers của Story 2.2; hàng "Trang 0 khối" spec 6.9; dòng 🔵 ở retro Epic 6 §F6 — #76, #17, #84.
- `AGENTS.md` §This machine: dòng LuLu thành "nghi môi trường trước" (Ice tắt LuLu vĩnh viễn); so số đo chỉ cùng toolchain — #72, #40.

**Kết quả:** 239 → **16** mục mở/🟡 đứng tên Ice: 13 treo điều kiện, 2 việc môi trường máy Ice, 1 giữ kèm điều kiện (#3). `check:debt-owner` xanh.
