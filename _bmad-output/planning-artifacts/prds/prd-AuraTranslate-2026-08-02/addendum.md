---
title: "Addendum — PRD AuraTranslate"
status: final
created: 2026-08-02
updated: 2026-08-02
---

# Addendum — PRD AuraTranslate

Tài liệu này giữ **chiều sâu kỹ thuật đứng sau các FR/NFR của PRD** — thứ cần cho `bmad-architecture` nhưng làm loãng PRD nếu đưa vào. Mỗi mục ghi rõ nó chống lưng cho yêu cầu nào.

> Bối cảnh cạnh tranh, khảo sát nguồn từ điển và nguyên văn PRD v8.0 nằm ở `briefs/brief-AuraTranslate-2026-08-02/addendum.md`, không lặp lại ở đây.

---

## A. Cơ chế truy vấn từ điển tiếng Trung — ba nhánh

> Chống lưng cho **FR39** *(tra cứu phải trả kết quả cho truy vấn 1, 2 và 3+ ký tự)*.

Báo cáo nghiên cứu ban đầu đề xuất chiến lược **hai nhánh** (`unicode61` + `trigram`, fallback `LIKE` cho token ngắn). Số đo Giai đoạn 0 bác bỏ phương án này: `LIKE` tốn 20–50 ms, **quá chậm để nằm trên đường nóng**.

Chiến lược đã kiểm chứng:

| Loại truy vấn | Cơ chế | Độ trễ đo được |
|---|---|---|
| Tra chính xác đầu mục | Chỉ mục B-tree | **0,02 ms** — đường nóng Auto-Lookup |
| Chuỗi con 1–2 ký tự | **`char_idx`** — bảng đảo ngược `(ký tự, entry_id)` | 0,15 ms (1 ký tự) · 4,49 ms (2 ký tự, giao hai tập) |
| Chuỗi con 3+ ký tự | FTS5 `trigram` | 0,13–0,19 ms |

**`char_idx`:** 1.297.115 cặp, dựng trong 3,7 giây, tốn 33,4 MB — thành phần tốn dung lượng nhất của database, và **chưa hề có trong kế hoạch ban đầu**.

### Vì sao đây là cái bẫy nguy hiểm

FTS5 trigram trả về **rỗng** cho truy vấn dưới ba ký tự, **và không báo lỗi**:

| Truy vấn | Số ký tự | FTS5 trigram | LIKE (đối chứng) |
|---|---|---|---|
| 山 | 1 | **0** | 2.576 |
| 中國 | 2 | **0** | 318 |
| 中國人 | 3 | 26 | 26 |

Phần lớn từ tiếng Trung được tra nhiều nhất dài 1–2 ký tự. Truy vấn chạy trong 0,01 ms rồi trả về rỗng — biểu hiện thành *"tra từ không ra kết quả"*, rất khó lần ra nguyên nhân nếu không đối chứng.

> **Bài học tổng quát cho các quyết định kỹ thuật còn lại:** rủi ro không nằm ở chỗ trông có vẻ khó, mà ở chỗ **mặc định của công cụ không khớp với ca sử dụng**. FTS5 *có* chạy với tiếng Trung — chỉ là ra kết quả vô nghĩa.

---

## B. Chỉ mục FTS5 và dấu tiếng Việt

> Chống lưng cho **NFR8** và **FR9**.

`unicode61` mặc định đặt `remove_diacritics 1`, gộp sáu từ khác nghĩa thành một kết quả:

| Truy vấn | Mặc định | `remove_diacritics 0` |
|---|---|---|
| `má` | `má, ma, mà, mả, mã, mạ` 🔴 | `má` ✅ |
| `núi` | `núi, nui` 🔴 | `núi` ✅ |

**Khuyến nghị đã chốt:** lập chỉ mục hai lần — `remove_diacritics 0` làm chỉ mục **chính**, cộng một chỉ mục **phụ** có xoá dấu cho chế độ tìm kiếm khoan dung. Chi phí đã biết: **~17 MB mỗi chỉ mục FTS**.

---

## C. Hình dạng dữ liệu từ điển

> Chống lưng cho **FR28–FR32**.

Schema **không thể** chỉ là `(từ, nghĩa, nguồn)`. Tối thiểu cần:

```
từ khoá  →  [ nguồn, từ loại, nghĩa, ví dụ[], trích dẫn[], ghi chú ]
```

- **Một từ có nhiều từ loại** → nhiều bản ghi, không phải một chuỗi nghĩa gộp
- **Ví dụ gắn với từng từ loại**, không gắn với cả từ
- **Trích dẫn** là trường riêng, khác ví dụ: trích dẫn có xuất xứ văn bản
- **`source` là cột bắt buộc trên mọi bản ghi** — đã kiểm chứng qua truy vấn gộp nguồn, chi phí 0,02 ms

### Quyết định có bán kính ảnh hưởng rộng nhất

**Chuyển mọi từ điển sang SQLite ở bước build**, không đọc định dạng gốc lúc chạy. Một quyết định gỡ được năm vấn đề:

| Giải quyết | Cách |
|---|---|
| Truy vấn thống nhất | Cùng tầng SQL với TM, Glossary, Library |
| Full-text search | FTS5 dùng được ngay |
| Gộp nhiều nguồn | Một bảng, nhiều nguồn |
| Ghi nguồn định nghĩa | Cột `source` — yêu cầu bắt buộc của FR31 |
| Rủi ro giấy phép parser | Parser chỉ nằm trong build tool, **không vào bản phát hành** |

### Ngân sách kích thước theo tầng

> Chống lưng cho **NFR6**. Đo trên 604.357 bản ghi nghĩa + 27.956 ví dụ, từ CVDICT + CC-CEDICT + viwiktionary.

| Tầng | Kích thước | Tăng thêm |
|---|---|---|
| Dữ liệu thô (`entry` + `example`) | 48,7 MB | — |
| + chỉ mục B-tree | 65,7 MB | +17,0 MB |
| + FTS5 `unicode61` trên nghĩa | 82,7 MB | +17,0 MB |
| + FTS5 `trigram` trên đầu mục | 96,6 MB | +13,9 MB |
| + `char_idx` | **130,0 MB** | +33,4 MB |

Phỏng đoán *"chỉ mục trigram lớn hơn đáng kể"* **bị bác** — trigram chỉ thêm 14% tổng.

---

## D. Language-aware matching — một cơ chế dùng chung

> Chống lưng cho **FR40** (từ điển), **FR51** (Glossary), **FR61** (TM).

Ba FR ở ba nhóm năng lực khác nhau dựa trên **cùng một cơ chế**. Đây phải là **một thành phần dùng chung**, không phải ba lần cài đặt riêng.

| Ngôn ngữ | Cơ chế | Dùng ở |
|---|---|---|
| **Tiếng Trung** | Khớp chính xác; n-gram ký tự cho fuzzy (không có ranh giới từ). Tách từ qua `jieba-rs` khi cần | FR51 Glossary, FR61 TM |
| **Tiếng Anh** | Stemming rồi token n-gram | FR40 từ điển, FR51 Glossary, FR61 TM |

**Giới hạn đã tuyên bố:** hệ sinh thái Rust **không có lemmatizer trưởng thành**. Stemming đủ cho khớp Glossary nhưng không xử lý được biến thể bất quy tắc. FR40 ghi rõ giới hạn này thay vì giấu đi.

---

## E. Stack đề xuất & trạng thái giấy phép

> Chống lưng cho **NFR15**. Không phải quyết định kiến trúc cuối cùng — đó là việc của `bmad-architecture`.

| Vùng | Khuyến nghị | Giấy phép |
|---|---|---|
| Lõi backend | Rust | — |
| Khung desktop | Tauri v2 *(ổn định từ 10/2024)* | MIT/Apache ✅ |
| Frontend | Web (React/Vue/TypeScript) | — |
| Database | SQLite + FTS5 lai, WAL, hàng đợi ghi tầng ứng dụng | Phạm vi công cộng ✅ |
| Tách từ tiếng Trung | `jieba-rs` | MIT ✅ |
| Stemming tiếng Anh | `tantivy-stemmers` *(bảo trì tốt hơn `rust-stemmers`)* | BSD ✅ |
| Diff | `dissimilar` *(semantic cleanup)* hoặc `similar` *(grapheme-level)* — thử cả hai | Apache+MIT ✅ / cần xác nhận |
| `.docx` | `docx-rs` | MIT ✅ |
| Client LLM | `reqwest` + SSE — **không dùng client tự reconnect** | cần xác nhận |
| Lưu khoá API | `tauri-plugin-keyring` — **không dùng Stronghold** | cần xác nhận |

**Chưa xác nhận giấy phép, phải kiểm tra trước khi đưa vào dự án:** `similar`, `tauri-plugin-keyring`, `reqwest-sse`, `sseer`, `rdocx`, `ollama-rs`.

---

## F. Phương án đã loại và lý do

Giữ lại để người đọc sau không đề xuất lại những gì đã cân nhắc và bỏ.

| Phương án | Vì sao loại |
|---|---|
| **`tauri-wire`** (IPC nhị phân) | Không cần. Payload đo được 679 byte, rất xa ngưỡng 10 KB; chi phí tuần tự hoá 0,002 ms |
| **`LIKE` làm fallback cho token ngắn** | 20–50 ms — quá chậm cho đường nóng. Thay bằng `char_idx` |
| **Stronghold** lưu API key | Đã bị khai tử. Phần lớn hướng dẫn Tauri vẫn còn chỉ dùng nó |
| **kaikki.org làm lớp từ loại cho tiếng Trung** | Chỉ phủ 2,76% đầu mục CVDICT; 0,067% có ví dụ |
| **zh.wiktionary** | Độ phủ khổng lồ (2,5 triệu đầu mục) nhưng định nghĩa và nhãn từ loại **bằng tiếng Trung** — không phục vụ trực tiếp người dùng Việt |
| **FVDP / OVDP** (Anh-Việt) | kaikki.org phủ luôn cặp Anh–Việt **và** kèm từ loại. Bỏ FVDP còn gỡ luôn ràng buộc GPL v2+ lan truyền |
| **Trần Văn Chánh (1999)** | Còn bản quyền |
| **GPL v2** | Không tương thích crate Apache-2.0 — phủ gần trọn hệ sinh thái Rust |
| **Cơ chế tải từ điển sau khi cài** | **Vẫn loại**, nhưng lý do đã đổi *(2026-08-05)*. Lý do cũ *"tổng ước tính 150–200 MB, chấp nhận được"* **sai** — số thật là **343.991.430 byte**. Lý do mới, mạnh hơn: lời hứa *"không tải thêm sau khi cài"* chống đỡ cho **NFR7** *(tra cứu ngoại tuyến 100%)* và **NFR12**; tải sau khi cài làm hỏng chính điều kiện tồn tại của sản phẩm. Ice chọn **nâng trần lên 400.000.000 byte** và chấp nhận bộ cài ~344 MB thay vì đổi cơ chế |
| **Định dạng gói chia sẻ cộng đồng** | Ngoài phạm vi v1. Trao đổi file (CSV/TSV, TMX, file prompt) là đủ |
| **Ký số qua Azure Key Vault + `relic`** | Không có kinh phí. Giữ lại làm tham chiếu cho thời điểm dự án có tài trợ |

---

## G. Chi phí phát hành desktop app — tham chiếu cho tương lai

> Chống lưng cho **§9.1** của PRD. Đây là **chi phí tiền mặt định kỳ duy nhất** của dự án, hiện đã quyết định không chi.

| Nền tảng | Yêu cầu | Chi phí |
|---|---|---|
| macOS | Chứng chỉ ký từ Apple Developer Program | Tài khoản miễn phí **không notarize được** |
| macOS notarization | Nộp app đã ký lên máy chủ Apple quét mã độc | Bắt buộc nếu muốn mở không cảnh báo |
| Windows | Chứng chỉ EV code signing | **Trên 400 USD** + bắt buộc token phần cứng |
| Windows *(từ 06/2023)* | CA không còn cấp chứng chỉ OV dạng file xuất được; phải nằm trên HSM | — |

Phương án rẻ nhất cho lập trình viên độc lập khi có kinh phí: **Azure Key Vault** làm HSM đám mây + **`relic`** để ký file thực thi Windows.

---

## H. Việc chưa đo được ở Giai đoạn 0

| Hạng mục | Vì sao chưa đo | Rủi ro còn lại |
|---|---|---|
| **Vòng IPC Tauri thật** + thời gian render frontend | Cần app có cửa sổ, không đo được ở môi trường dòng lệnh | Đã giảm mạnh nhờ payload 679 byte. Nếu Auto-Lookup chậm, nguyên nhân sẽ ở frontend |
| **Unihan, Thiều Chửu, Cổ hán văn, VietPhrase** trong database | 🟢 **Ba trong bốn đã nạp và đo thật 2026-08-05** *(Unihan · Thiều Chửu · VietPhrase — Story 1.9/1.10)*. **Cổ hán văn** vẫn chưa — chưa có nguồn thô | Ước cũ *"tổng 150–200 MB dựa trên ba nguồn đã có"* **sai gấp đôi**: thật là **343.991.430 byte**. Riêng VietPhrase nở **6,7×** từ nguồn thô *(23.844.586 → 160.083.968 byte)*, không phải hệ số 2,67× dùng để ước. **Bài học: hệ số nở phụ thuộc HÌNH DẠNG dữ liệu** — nhiều đầu mục ngắn tốn chỉ mục hơn ít đầu mục dài |
| **`jieba-rs` và `tantivy-stemmers`** chạy thật | Mới xác minh giấy phép và độ trưởng thành | Chưa có số đo chất lượng tách từ / stemming |
| **Tìm kiếm Library trên thư viện thật** | Chưa có Library | Ngưỡng NFR3, NFR4 còn là `[ASSUMPTION]` |

**Mã nguồn các mũi thăm dò** (`build_db.py`, `bench.py`, `rustbench/`) nằm trong scratchpad của phiên nghiên cứu, không đưa vào repo — mã dùng một lần.
