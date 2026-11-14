---
type: change
title: "Sprint Change Proposal — 2026-11-14 · Tra nhanh một từ ở bảng Mod+K"
status: done
created: 2026-11-14
skill: bmad-correct-course
---
# Sprint Change Proposal — 2026-11-14 · Tra nhanh một từ ở bảng Mod+K

**Người soạn:** `bmad-correct-course`, chế độ Batch · **Người duyệt:** Ice
**Baseline:** `98a73f75` (master; cây có hai tệp chưa theo dõi là plan và handoff của chính thay đổi này)
**Nguồn:** `plan-tra-tu-nhanh-trong-bang-mod-k.md` §Intent, quyết định của Ice 2026-11-14
**Trạng thái:** đã viết AD-69 và FR161. 🔵 2026-11-14: Ice ký AD-69, FR161 và cả năm điểm Winston tự đề xuất ở mục 3.

---

## 1. Vấn đề

Ice muốn mở Mod+K, gõ một từ rồi thấy nghĩa ngay trong khung. Hôm nay chỉ tra được một từ ở panel Lookup, phải gõ chữ Hán, và không có chiều Việt → Trung. Từ cần tra có thể là chữ Hán, pinyin, tiếng Việt hoặc tiếng Anh.

Ba chỗ của quy hoạch chưa có chỗ cho việc này:
- **PRD §3.2** ghi ngoài phạm vi mọi cặp ngôn ngữ khác Anh → Việt và Trung → Việt. Tra ngược Việt → Trung đọc theo chữ là một cặp ngoài phạm vi.
- **Không FR nào** nói về tra một từ ngoài panel Lookup, về pinyin, hay về chuyện xem trước không được ghi lịch sử (FR41).
- **Spine:** AD-44 ① chốt vị từ điều phối nhị phân và Latin đi đường tiếng Anh; không AD nào cho đường Latin chạy thêm pinyin và Việt → Trung. Schema từ điển (AD-25) không có chỗ cho khoá pinyin, và `dict_source` không nói nghĩa của nguồn viết bằng tiếng nào.

Bằng chứng đo 2026-11-14 trên `dict-core.db` dict-v3 (chi tiết ở AD-69 §bằng chứng):
- Quét pinyin mất 0,37 s trên 247.354 mục, nên cần chỉ mục.
- Cột `reading` lẫn nhiều dạng (`ni3 hao3`, `nǐ hǎo`, `nǚ (nü³)`, số thanh chữ mũ); nối tay không ra khoá dùng được.
- `sense_fts` phủ cả nghĩa tiếng Anh và 190.543 nghĩa tiếng Việt gắn vào đầu mục tiếng Anh; lọc một vế thì lẫn.

## 2. Phân tích tác động

| Tài liệu | Tác động |
|---|---|
| PRD | §3.2 một câu cập nhật (Việt → Trung là cách tra, không phải cặp dịch). FR161 mới cuối C3, kèm nghiệm thu. FR41 một dòng cập nhật (xem trước không ghi). §5.1 dải C3 và tổng 159 → 160. §5.2 thêm thuật ngữ **Tra nhanh**. |
| Spine | AD-69 mới. AD-44 ①, AD-25, AD-27 và dòng `Tra cứu` của bảng Consistency mỗi chỗ một dòng cập nhật. Map C3 và C10 thêm AD-69. |
| Epic và ticket | Chưa có ticket (`ticket: ''` trong plan). Không sửa epic file. Sofia cắt story khi Ice ký. |
| `EXPERIENCE.md`, `DESIGN.md` | Chưa sửa. Khung Tra từ (nhóm, dòng, khung xem trước, nút) thuộc Sally. |
| `dict-manifest.toml`, `.db` | Chưa đổi. Dựng lại bốn tệp và bốn sha256 là việc của story thi hành (AD-25); release `dict-v4` do Ice tải lên. |
| NFR | NFR1 áp cho lệnh mới, đo trên chính nó. NFR6: thêm một cột và một chỉ mục cho 438.001 hàng, kích thước chưa đo; trần 500.000.000 byte, macOS 432.363.050 byte (đo 2026-10-21). NFR8 và FR9 được dùng lại, không đổi. |
| FR27, FR36, FR39, FR40 | Không đổi. Tệp v3 vẫn nạp (`MINIMUM_SCHEMA_VERSION` giữ 3), gỡ một lớp rời vẫn không làm hỏng tra cứu. |

**Cái giá:** bộ từ điển đổi lược đồ, nên người dùng đang giữ tệp v3 mất hai nhóm kết quả tới khi cập nhật; bản cài mới phát hành kèm `dict-v4`.

## 3. Quyết định của Ice (2026-11-14)

| # | Quyết định | Ice chọn |
|---|---|---|
| 1 | Phạm vi | Đủ ba phần: tra chữ Hán, gõ pinyin, Việt → Trung |
| 2 | Index pinyin | Cột `reading_key` và chỉ mục trong schema v4; dựng lại bốn `.db`, phát hành `dict-v4` |
| 3 | Vào chế độ | Tiền tố `/` ở đầu ô; thêm lệnh "Tra từ" cho chuột |
| 4 | Hình dạng kết quả | Danh sách kèm khung xem trước bản ghi đủ; xem trước không ghi lịch sử, Enter mở panel Lookup và ghi lịch sử |
| 5 | Câu Latin | Hiện cả ba nhóm: Hán theo pinyin, Việt → Trung, Anh → Việt |
| 6 | Chế độ duyệt | Batch |

Các điểm Winston đề xuất thêm khi viết AD-69, chưa có chữ ký riêng của Ice: lọc Việt → Trung bằng cả `lang` và `gloss_lang`; `gloss_lang` khai trên `SourceMeta`; `pinyin_key` cắt ở `(` và dùng bảng ký tự đóng; gõ có dấu không tự nới sang chỉ mục không dấu; mỗi nhóm lấy 21 trả 20 kèm `has_more`.

## 4. Đề xuất sửa chi tiết

### 4.1 PRD
- §3.2, dòng cặp ngôn ngữ: thêm câu cập nhật 2026-11-14.
- §5.1: dải C3 thành `FR27–FR41, FR161`; tổng thành 160 (FR1–FR161, trừ FR20).
- §5.2: hàng **Tra nhanh**, `Không dùng` chỉ về tên của thao tác AI.
- §6.3: FR41 nhận một dòng cập nhật; mục con **Tra nhanh trong bảng Mod+K** chứa FR161 và chín ca nghiệm thu.

### 4.2 Spine
- AD-69 sau AD-68, theo khuôn Binds / Prevents / Rule / Phương án bị loại / bằng chứng. Số kế tiếp đã quét trong spine và trong mọi `ad-brief-*`: không nơi nào dùng AD-69.
- Dòng cập nhật ở AD-44, AD-25, AD-27, dòng `Tra cứu`; AD-69 vào Map C3 và C10; `updated` thành 2026-11-14.

### 4.3 Không làm ở lượt này
- Không viết code, không dựng `.db`, không sửa `dict-manifest.toml`.
- Không tạo ticket hay sửa epic file.
- Chưa chạy Reviewer Gate cho AD-69 (rubric, reality, adversary như AD-68).

## 5. Bàn giao

**Phạm vi:** Moderate. Thêm một FR và một AD, đổi lược đồ dữ liệu phát hành; không xếp lại epic.

- **Ice:** ký AD-69 và FR161, hoặc sửa các điểm ở mục 3 (đoạn "Winston đề xuất thêm").
- **Winston:** Reviewer Gate cho AD-69 nếu Ice muốn, trước khi story thi hành.
- **Sally:** khung Tra từ trong `EXPERIENCE.md`.
- **Sofia:** cắt story theo bốn phase của plan (AD + PRD · Rust và dict-build · Webview · Test).
- **Ice, sau story:** tải bốn tệp `dict-v4` lên GitHub Release, thử ở app thật; nếu chưa kịp trước `done`, ghi nợ `Chủ: Ice`.

**Tiêu chí thành công:**
- `grep -n 'AD-69' architecture-auratranslate.md` thấy tiêu đề AD-69, ba dòng cập nhật ở AD-25, AD-27, AD-44, dòng `Tra cứu` và hai hàng Map.
- `grep -n 'FR161' prd-auratranslate.md` có mặt ở §3.2, §5.1, §5.2, FR41 và FR161.
- `npm run check:debt-owner` xanh (không thêm hay đổi mục nợ nào ở lượt này).
