---
title: "Giai đoạn 0 — Kết quả bốn mũi thăm dò"
status: complete
created: 2026-08-02
updated: 2026-08-02
relates_to:
  - '_bmad-output/planning-artifacts/briefs/brief-AuraTranslate-2026-08-02/brief.md'
  - '_bmad-output/planning-artifacts/research/technical-auratranslate-tauri-rust-local-first-research-2026-08-02.md'
---

# Giai đoạn 0 — Kết quả bốn mũi thăm dò

**Mục đích:** biến các phỏng đoán trong báo cáo technical research thành số đo thật, trước khi viết PRD.

**Môi trường đo:** macOS (darwin 24.6.0) · Rust 1.97.1 · SQLite hệ thống 3.43.2 · rusqlite bundled 3.46.0 · Python 3.14.6

**Dữ liệu thật đã tải và xử lý:** CVDICT (10 MB, 122.627 dòng) · CC-CEDICT (9,4 MB, 124.755 dòng) · Vietnamese Wiktionary qua kaikki.org (260 MB, 415.254 bản ghi) · zh.wiktionary qua kaikki.org (1,13 GB, 2.914.256 bản ghi)

---

## Tóm tắt: hai phỏng đoán bị bác, hai cái bẫy mới lộ ra

| # | Phép đo | Kết quả |
|---|---|---|
| 1 | Độ trễ Auto-Lookup | 🟢 **Không phải nút thắt.** Phía Rust 0,022 ms; payload 679 byte |
| 2 | Kích thước database | 🟢 **130 MB** trọn bộ. Lo ngại trigram làm phình đã bị bác |
| 3 | Tokenizer lai | 🔴 **Trigram trả về RỖNG cho truy vấn 1–2 ký tự.** Đã tìm được lời giải |
| 3b | Dấu tiếng Việt | 🔴 **unicode61 mặc định xoá dấu**, gộp `má/ma/mà/mả/mã/mạ` thành một |
| 4 | Độ phủ kaikki.org | 🔴 **Chỉ 2,76% cho tiếng Trung.** Không dùng được làm lớp từ loại cho ZH |

---

## Phép đo 1 — Độ trễ Auto-Lookup

Đo bằng Rust thật (`rusqlite` + `serde_json`), 500 lượt tra trên 10 từ tiếng Trung phổ biến, truy vấn gộp mọi nguồn kèm ví dụ — đúng hình dạng dữ liệu Panel 2 cần.

| Giai đoạn | p50 | p95 | p99 |
|---|---|---|---|
| Truy vấn SQLite + dựng struct | **0,020 ms** | 0,042 ms | 0,055 ms |
| Tuần tự hoá JSON (payload IPC) | 0,002 ms | 0,005 ms | 0,007 ms |
| **Tổng phía Rust** | **0,022 ms** | 0,046 ms | 0,065 ms |

**Payload JSON trung bình: 679 byte.**

> ### Kết luận: backend không phải nút thắt, và `tauri-wire` là không cần thiết
>
> Báo cáo technical research cảnh báo Tauri tuần tự hoá mọi payload IPC thành JSON, và nêu `tauri-wire` (nhị phân, nhanh hơn 28–33 lần) làm phương án dự phòng.
>
> **Phương án đó không cần dùng.** Payload 679 byte nằm rất xa ngưỡng 10 KB mà Tauri chuyển sang `JSON.parse()`, và chi phí tuần tự hoá là 0,002 ms — tức 9% của một phần trăm mili giây. Ngay cả khi vòng IPC tốn thêm 1 ms, tổng vẫn dưới ngưỡng cảm nhận của con người.
>
> **Còn lại chưa đo:** vòng IPC Tauri thật và thời gian render frontend. Cần một app Tauri có cửa sổ, không đo được ở môi trường dòng lệnh. Nhưng với payload nhỏ như vậy, rủi ro đã giảm mạnh: nếu Auto-Lookup chậm, nguyên nhân sẽ nằm ở frontend chứ không phải ở đường dữ liệu.

---

## Phép đo 2 — Kích thước database

Dựng database thật từ CVDICT + CC-CEDICT + viwiktionary: **604.357 bản ghi nghĩa, 27.956 ví dụ**.

| Tầng | Kích thước | Tăng thêm |
|---|---|---|
| 1. Dữ liệu thô (`entry` + `example`) | 48,7 MB | — |
| 2. + chỉ mục B-tree | 65,7 MB | +17,0 MB |
| 3. + FTS5 `unicode61` trên nghĩa | 82,7 MB | +17,0 MB |
| 4. + FTS5 `trigram` trên đầu mục | 96,6 MB | **+13,9 MB** |
| 5. + `char_idx` (xem Phép đo 3) | **130,0 MB** | +33,4 MB |

> **Phỏng đoán bị bác:** báo cáo trước cảnh báo *"chỉ mục trigram được ghi nhận là lớn hơn đáng kể"*. Thực tế trigram chỉ thêm **13,9 MB — 14% tổng**. Thành phần tốn nhất lại là `char_idx` (+33,4 MB), thứ chưa hề có trong kế hoạch ban đầu.
>
> **Còn phải cộng thêm:** Unihan, Thiều Chửu, Cổ hán văn, VietPhrase. Ước tính tổng cuối **150–200 MB**. Vẫn hoàn toàn chấp nhận được cho desktop app, **không cần cơ chế tải về sau khi cài**.

---

## Phép đo 3 — Chiến lược tokenizer lai

### 3a. 🔴 Trigram trả về RỖNG cho truy vấn 1–2 ký tự

Đây là phát hiện nghiêm trọng nhất của Giai đoạn 0.

| Truy vấn | Số ký tự | FTS5 trigram | LIKE (đối chứng) | |
|---|---|---|---|---|
| 山 | 1 | **0** | 2.576 | 🔴 mất trắng |
| 中國 | 2 | **0** | 318 | 🔴 mất trắng |
| 中國人 | 3 | 26 | 26 | ✅ khớp chính xác |
| 一個人 | 3 | 4 | 4 | ✅ |
| 天氣預報 | 4 | 2 | 2 | ✅ |

> **Vì sao đây là vấn đề lớn:** trigram lập chỉ mục theo chuỗi ba ký tự, nên truy vấn ngắn hơn ba ký tự không khớp được gì. Mà **phần lớn từ tiếng Trung dài 1–2 ký tự** — 山, 打, 中國, 學生 chính là những từ được tra nhiều nhất.
>
> **Nguy hiểm ở chỗ nó không báo lỗi.** Truy vấn chạy trong 0,01 ms và trả về rỗng. Nếu không đối chứng bằng `LIKE`, lỗi này sẽ lọt vào sản phẩm và biểu hiện thành "tra từ không ra kết quả" — rất khó lần ra nguyên nhân.

### 3b. Lời giải: chỉ mục đảo ngược theo ký tự (`char_idx`)

Bảng `(ký tự, entry_id)` phủ mọi ký tự Hán trong đầu mục. **1.297.115 cặp**, dựng trong 3,7 giây, tốn 33,4 MB.

| Truy vấn | `char_idx` | `LIKE` | Nhanh hơn |
|---|---|---|---|
| 1 ký tự | **0,15 ms** | 20,09 ms | **134×** |
| 2 ký tự (giao hai tập) | **4,49 ms** | 50,14 ms | **11×** |

### 3c. Chiến lược cuối cùng cho tiếng Trung — ba nhánh, không phải hai

```
Tra chính xác đầu mục   →  chỉ mục B-tree      →  0,02 ms   (đường nóng Auto-Lookup)
Chuỗi con 1–2 ký tự     →  char_idx            →  0,15–4,5 ms
Chuỗi con 3+ ký tự      →  FTS5 trigram        →  0,13–0,19 ms
```

> Báo cáo technical research đề xuất chiến lược **hai** nhánh (`unicode61` + `trigram`, fallback `LIKE` cho token ngắn). Số đo cho thấy `LIKE` **quá chậm để làm fallback** (20–50 ms trên đường nóng). Cần nhánh thứ ba là `char_idx`.

### 3d. 🔴 `unicode61` xoá dấu tiếng Việt — gộp sáu từ thành một

Nạp `má ma mà mả mã mạ núi nui` vào hai bảng FTS5 và truy vấn:

| Truy vấn | Mặc định (`remove_diacritics 1`) | `remove_diacritics 0` |
|---|---|---|
| `má` | `má, ma, mà, mả, mã, mạ` 🔴 | `má` ✅ |
| `ma` | `má, ma, mà, mả, mã, mạ` 🔴 | `ma` ✅ |
| `mà` | `má, ma, mà, mả, mã, mạ` 🔴 | `mà` ✅ |
| `núi` | `núi, nui` 🔴 | `núi` ✅ |

> **Với một công cụ dịch thuật tiếng Việt, đây là lỗi nghiêm trọng.** `má`, `mà`, `mả`, `mã`, `mạ` là năm từ khác nghĩa hoàn toàn. Tìm kiếm gộp chúng lại sẽ phá vỡ độ chính xác của full-text search trong Library.
>
> **Nhưng xoá dấu cũng có giá trị** — người dùng thường gõ không dấu cho nhanh.
>
> **Khuyến nghị: lập chỉ mục hai lần.** `remove_diacritics 0` làm chỉ mục chính (chính xác), cộng một chỉ mục phụ có xoá dấu cho chế độ tìm kiếm khoan dung. Người dùng chọn, hoặc hệ thống thử chính xác trước rồi mới nới lỏng. Chi phí lưu trữ đã biết: ~17 MB mỗi chỉ mục FTS.

---

## Phép đo 4 — Độ phủ thực tế của kaikki.org

### 4a. Chất lượng dữ liệu: rất tốt ở chỗ nó có

Mục từ 你好 trong viwiktionary:

- `pos: intj`, **`pos_title: "Thán từ"`** — từ loại đã sẵn tiếng Việt, dùng thẳng cho UI
- nghĩa: *"Xin chào; chào."*
- ví dụ: 你好，好久不見。 → *"Xin chào, lâu rồi không gặp."* — kèm bản dịch tiếng Việt
- các trường khác: `derived`, `descendants`, `forms`, `related`, `sounds`, `notes`, `etymology_texts`

**Đúng chính xác cấu trúc Panel 2 cần.** Vấn đề hoàn toàn nằm ở độ phủ.

### 4b. Độ phủ theo ngôn ngữ (viwiktionary, 415.254 bản ghi)

| Ngôn ngữ | Mục từ | Nghĩa | Có từ loại | Có ví dụ |
|---|---|---|---|---|
| **Tiếng Anh** | 133.319 | 190.670 | 133.319 (100%) | 11.821 (**8,9%**) |
| Tiếng Pháp | 51.717 | 79.541 | 100% | 17.158 |
| Tiếng Việt | 44.361 | 56.286 | 100% | 22.247 |
| **Tiếng Trung Quốc** | 3.444 | 3.949 | 100% | **149** |
| **Tiếng Quan Thoại** | 3.731 | 11.146 | 100% | **14** |

### 4c. 🔴 Đối chiếu với CVDICT — kết quả quyết định

| | Số lượng |
|---|---|
| Đầu mục CVDICT (phồn + giản) | 194.605 |
| Đầu mục tiếng Trung trong viwiktionary | 10.041 |
| **Chồng lấn** | **5.367 = 2,76%** |
| **Chồng lấn *có kèm ví dụ*** | **130 = 0,067%** |

> ### Kết luận: kaikki.org không dùng được làm lớp từ loại cho tiếng Trung
>
> Nó phủ **2,76%** đầu mục, và chỉ **130 mục từ** trên toàn bộ CVDICT có kèm ví dụ. Với một công cụ mà người dùng tra từ tiếng Trung suốt cả ngày, con số này là không dùng được.
>
> **Với tiếng Anh thì ngược lại:** 133.319 mục từ, 100% có từ loại — hoàn toàn dùng được. Chỉ có ví dụ là thưa (8,9%).

### 4d. Đã kiểm tra phương án thay thế

**zh.wiktionary** (1,13 GB, 2.914.256 bản ghi, 2.517.198 đầu mục duy nhất, 63.944 đầu mục có ví dụ) — độ phủ khổng lồ và giàu ví dụ, **nhưng định nghĩa và nhãn từ loại đều bằng tiếng Trung** (動詞, 名詞, 形容詞). Không phục vụ trực tiếp người dùng Việt.

---

## Việc cần quyết sau Giai đoạn 0

### Đã có kết luận chắc chắn — đưa thẳng vào PRD/Architecture

1. **Ba nhánh truy vấn tiếng Trung**, không phải hai: B-tree (chính xác) / `char_idx` (1–2 ký tự) / trigram (3+ ký tự).
2. **`remove_diacritics 0`** cho chỉ mục FTS chính; cân nhắc chỉ mục phụ có xoá dấu cho tìm kiếm khoan dung.
3. **Ngân sách kích thước: 130 MB** hiện tại, ước tính 150–200 MB khi đủ nguồn. Không cần tải về sau cài đặt.
4. **Bỏ `tauri-wire` khỏi kế hoạch dự phòng** — payload 679 byte, không cần tối ưu nhị phân.
5. **Mỗi bản ghi nghĩa phải mang cột `source`** — đã kiểm chứng qua truy vấn gộp nguồn, chi phí 0,02 ms.

### 🔴 Cần Ice quyết: lớp từ loại và ví dụ cho tiếng Trung

Lựa chọn kaikki.org làm lớp từ loại **đúng cho tiếng Anh, sai cho tiếng Trung**. Ba phương án:

| | Phương án | Được | Mất |
|---|---|---|---|
| **A** | Chấp nhận: tiếng Trung chỉ có nghĩa, không có từ loại/ví dụ | Không phát sinh việc, toàn bộ giấy phép sạch | Panel 2 nghèo hẳn ở đúng ngôn ngữ khó nhất |
| **B** | Xin phép **Hán Việt Từ Điển Trích Dẫn** (Đặng Thế Kiệt) | Nguồn duy nhất có từ loại + ví dụ + trích dẫn tiếng Việt cho Hán Việt | Phụ thuộc vào một lời đồng ý; có thể không được |
| **C** | Dùng **en.wiktionary** bản tiếng Trung làm khung từ loại + câu ví dụ, ghép nghĩa tiếng Việt từ CVDICT | Độ phủ lớn, giấy phép sạch, câu ví dụ tiếng Trung vẫn hữu ích với người dịch | Nhãn từ loại và bản dịch ví dụ bằng tiếng Anh; cần công việc ghép nguồn; +1,1 GB dữ liệu nguồn lúc build |

---

## Việc chưa làm được ở giai đoạn này

- **Vòng IPC Tauri thật** — cần app có cửa sổ, không đo được ở môi trường dòng lệnh. Rủi ro đã giảm mạnh nhờ payload 679 byte.
- **Unihan, Thiều Chửu, Cổ hán văn, VietPhrase** chưa nạp vào database đo thử — ước tính kích thước dựa trên ba nguồn đã có.
- **jieba-rs và rust-stemmers** chưa chạy thử; mới xác minh giấy phép và mức độ trưởng thành ở bước nghiên cứu.

## Mã nguồn của các mũi thăm dò

Nằm trong thư mục scratchpad của phiên làm việc, không đưa vào repo (mã dùng một lần):

- `build_db.py` — dựng SQLite từ CVDICT + CC-CEDICT + viwiktionary, đo kích thước theo tầng
- `bench.py` — đo độ trễ truy vấn, đối chứng trigram/LIKE, kiểm tra dấu tiếng Việt
- `rustbench/` — dự án Cargo đo đường nóng bằng `rusqlite` + `serde_json`
