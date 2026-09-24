# Sprint Change Proposal — 2026-09-24b · Nợ đứng tên Ice ngoài lượt rà

**Người soạn:** John (PM), `bmad-correct-course` chế độ Batch · **Người duyệt:** Ice
**Baseline:** `b8f22f7` (master, cây sạch)
**Nguồn:** `sprint-change-proposal-2026-09-24-epic-11-tra-no-nen.md` §1 — *"239 mục ngoài lượt rà không thuộc phạm vi đề xuất này"*
**Trạng thái:** Ice duyệt hướng 2026-09-24. Bước 1 đã thi hành; bước 2–3 còn mở.

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
| 3 | E · C · F · D | **Thi hành Batch, commit riêng** |

## 4. Bước 1 — đã thi hành

Nối một dòng `→ … (xếp nợ đứng tên Ice)` vào 34 mục của `deferred-work.md`:

- **E, 5 mục:** `→ ✅ ĐÃ ĐÓNG` kèm chỗ đứng của bản vá trên HEAD.
- **C, 21 mục:** giao Story 10.1 · 10.4 · 10.9 · 11.1–11.7. John chỉnh so với phân loại: hai mục số đo NFR1 sang 10.9 (cùng họ NFR1 đầu-cuối đã ở 10.9); `editorPromoteAiTranslationError` sang 11.5 (cùng đường `promote_ai_translation`).
- **F, 8 mục → Winston:** 6 mục hạng F, cộng hai mục luật `is_omitted` khi gộp chưa vào spine, vốn phân loại C nhưng ghi spine là việc của Winston. Ba mục cùng một ràng buộc (`dispatch()` không tham số, AD-34); mục hoàn tác Glossary cần đối chiếu AD-49 trước.
- **D, 13 mục:** giữ `Chủ: Ice`, không sửa. Đúng AC chung của Epic 11 cho mục treo điều kiện.

Danh sách từng mục: `grep -n 'xếp nợ đứng tên Ice' deferred-work.md`. Sau bước 1: **205** mục mở/🟡 đứng tên Ice; `check:debt-owner` xanh.

## 5. Còn mở

- **Bước 2 — AI-7** (`sprint-status.yaml` `epic-3-retro-item-41-…`, cùng B10 của retro Epic 2): Ice chọn giữa một hàng nghiệm thu tay bắt buộc cho mỗi story chạm bề mặt, hoặc một lượt dùng thật có lịch cuối mỗi epic. Sau đó 30 mục B nhận chủ theo lựa chọn; đuôi "chờ B10" được gỡ khỏi mục không liên quan.
- **Bước 3 — hạng A:** tách 162 câu thành hai loại. Câu sản phẩm/UX/quy trình vào phiên quyết với John. Câu kỹ thuật đi theo story thi hành (`Chủ: Story 11.x` hoặc `10.x`). Nếu tải vượt sức một story thì tách story (định tuyến thô đưa 11.6 từ 28 lên 87 mục).

**Tiêu chí thành công:** `check:debt-owner` xanh; mỗi mục còn `Chủ: Ice` thuộc một trong ba loại: câu hỏi đang chờ phiên quyết, mục B chờ lượt nghiệm thu đã có lịch, hoặc mục D có điều kiện ghi rõ.
