# Sprint Change Proposal — Đường tiếng Anh bị rơi khỏi Epic 1

**Ngày:** 2026-08-05 · **Dự án:** AuraTranslate · **Người quyết:** Ice · ✅ **ĐÃ DUYỆT 2026-08-05**
**Trigger:** rà soát Story 1.10 (`1-10-dong-goi-bon-lop-go-roi-thanh-file-doc-lap`)
**Phân loại phạm vi:** 🟡 **Moderate** — tổ chức lại backlog, ⛔ không phải replan nền tảng

---

## 1. Tóm tắt vấn đề

**FR34 — *"Mục từ **tiếng Anh** phải có nhãn từ loại và nghĩa tiếng Việt"* — thuộc Epic 1, và hiện ⛔ KHÔNG có đường dữ liệu nào.**

Phát hiện khi chủ dự án hỏi *"đây là app dịch Anh–Việt và Trung–Việt, sao chỉ thấy tiếng Trung?"* trong lượt rà Story 1.10.

### Bằng chứng đo thật, ⛔ không phải suy đoán

```
dict-core.db đã dựng — 473.499 đầu mục:
  en-wiktionary  zh  174.677      cc-cedict  zh  124.758
  cvdict         zh  122.596      unihan     zh   49.870
  viwiktionary   zh    1.598
  ─────────────────────────────────────────────────────
  lang = 'en'                          0        ← FR34 không có gì để đứng lên

Log build viwiktionary:  đọc 415.115 · bỏ 413.517 · giữ 1.598
  [411.810] lang_code != zh (filtered, expected)
```

### Nguyên nhân gốc — **lỗi mơ hồ của PRD**, ⛔ không phải lỗi cài đặt

`kaikki.org/.../vi-extract.jsonl` là bản trích **toàn ấn bản** `vi.wiktionary.org`, chứa mục từ của **mọi** ngôn ngữ. Lọc `lang_code` quyết định vai:

| Vai | Lọc | Phục vụ | Trạng thái trước 2026-08-05 |
|---|---|---|---|
| **A** | `lang_code = "en"` | **FR34**, cặp **Anh → Việt** | 🔴 chưa dựng |
| **B** | `lang_code = "zh"` | Lớp từ loại ZH (§8.3) | ✅ đã dựng |

PRD `§8.2` giao viwiktionary vai **tiếng Anh**; `§8.3` — tiêu đề *"Lớp từ loại tiếng **Trung**"* — bàn **cùng nguồn đó** ở vai tiếng Trung. **PRD ⛔ chưa bao giờ nói đó là hai vai song song.** Story 1.9 đọc §8.3 và cài đúng một vai.

> ✅ **Đã xử lý ngày 2026-08-05:** `§8.2` tách thành **hai hàng nguồn** riêng + cảnh báo tường minh; `§8.3` khoanh phạm vi *"chỉ vai B"*. Nguyên nhân gốc đã đóng — phần còn lại là **thi công**.

### Mũi thăm dò đo thật (2026-08-05, đã hoàn tác sạch khỏi cây mã)

Dựng thử với `wiktextract_common::parse(reader, "vi", Some("en"))`:

| Chỉ số | Giá trị |
|---|---:|
| Dòng đọc | 401.101 |
| Bỏ *(không phải `en`)* | 281.935 |
| Bỏ *(không có gloss)* | 127 |
| **Đầu mục** *(đã gộp theo headword)* | **119.039** |
| **Nghĩa** | **190.543** |
| **Ví dụ** | **27.396** |
| **Dung lượng** | **40.333.312 byte** |
| Cặp `char_idx` | **9** ← xem §2.4 |

⇒ Dữ liệu **có thật, chất lượng tốt, giấy phép sạch** (CC-BY-SA + GFDL, cùng nguồn đã dùng). Chi phí xây dựng gần bằng không: `wiktextract_common` đã tham số hoá sẵn `filter_lang_code`.

---

## 2. Phân tích tác động

### 2.1 Tác động Epic

**Epic 1 ⛔ KHÔNG hoàn thành được như kế hoạch hiện tại.** Mục tiêu Epic 1 viết thẳng:

> *"Người dịch mở AuraTranslate, đưa một văn bản tiếng Trung **hoặc tiếng Anh** vào… **thấy ngay định nghĩa có ghi nguồn**"*

Với 0 mục tiếng Anh, một nửa mệnh đề đó ⛔ không nghiệm thu được.

**Các Epic khác:** ⛔ không epic nào bị vô hiệu, ⛔ không cần epic mới. Epic 2–10 giả định *"tra cứu hoạt động"* chứ ⛔ không giả định ngôn ngữ nào.

### 2.2 Tác động Story

| Story | Tác động |
|---|---|
| `1-9`, `1-10` *(done)* | ⛔ **Không rollback.** Cả hai làm đúng phạm vi đã viết; hạ tầng chúng dựng **dùng lại nguyên vẹn** cho tiếng Anh |
| **MỚI — dữ liệu tiếng Anh** | ➕ Chèn sau `1-10` |
| `1-11` *(ba nhánh truy vấn **tiếng Trung**)* | ⛔ Không đổi — nó đúng phạm vi tên nó |
| **MỚI — đường tra cứu tiếng Anh** | ➕ Chèn sau `1-11` |
| `1-12` *(matcher dùng chung)* | Mang **FR40** *(stemming tiếng Anh)* — nay có dữ liệu thật để khớp |
| `1-13`, `1-16`, `1-17`, `1-19` | ⛔ Không đổi cấu trúc; chỉ thêm ca kiểm cho mục tiếng Anh |

### 2.3 Xung đột artifact

| Tài liệu | Trạng thái |
|---|---|
| **PRD** | ✅ **Đã sửa 2026-08-05** — §8.2 tách hai vai, §8.3 khoanh phạm vi, [A2] ghi số đo, cảnh báo NFR6 |
| **epics.md** | 🔴 **Cần sửa** — thêm hai story, cập nhật truy vết FR34 |
| **sprint-status.yaml** | 🔴 **Cần sửa** — thêm hai khoá story |
| **ARCHITECTURE-SPINE** | 🔴 **Cần Winston** — ⛔ không có AD nào cho đường tra cứu tiếng Anh *(xem §2.4)* |
| **UX (EXPERIENCE/DESIGN)** | 🟡 **Cần Sally kiểm** — Panel Lookup có hình dạng cho mục tiếng Anh chưa? |
| **SPEC** | 🟡 Kiểm bản sao FR34 |

### 2.4 🔴 Tác động kỹ thuật — đường tra cứu tiếng Anh ⛔ KHÔNG dùng lại được AD-26

**AD-26** tên đầy đủ là *"Ba nhánh truy vấn **tiếng Trung**"*. Cả ba nhánh đều là cơ chế cho chữ Hán:

| Nhánh AD-26 | Dùng được cho tiếng Anh? |
|---|---|
| Tra chính xác đầu mục → B-tree | ✅ Có |
| Chuỗi con 1–2 ký tự → `char_idx` | ❌ **Không** — mũi thăm dò sinh đúng **9** cặp trên 119.039 đầu mục |
| Chuỗi con 3+ ký tự → FTS5 `trigram` | 🟡 Chạy được nhưng ⛔ không phải hình dạng đúng cho tiếng Anh |

Tiếng Anh cần **exact + stemming** (**FR40**, `Matcher` của **AD-17**), ⛔ không phải chuỗi con ký tự. ⇒ **Cần một AD mới cho đường tra cứu tiếng Anh** — đây là việc của Winston, ⛔ không phải của story.

### 2.5 🔴 Tác động NFR6 — biên còn lại rất mỏng

| | Byte |
|---|---:|
| Payload hôm nay *(7 nguồn)* | 343.991.430 |
| **+ Lớp tiếng Anh** | **40.333.312** |
| **Dự phóng** | **384.324.742** — vẫn **ĐẠT** trần 400.000.000 |
| **Dư địa cho HVTĐTD + Cổ hán văn** | **15.675.258** |

⚠️ Thiều Chửu tốn 5.787.648 byte *(vừa)*; **VietPhrase tốn 160.083.968** *(không vừa)*. Nếu HVTĐTD giàu ví dụ + trích dẫn như §8.3 mô tả, **trần 400 MB vượt lần nữa**. ⇒ **Đo HVTĐTD TRƯỚC khi hứa đóng gói.**

---

## 3. Đường đi khuyến nghị

### Đánh giá ba phương án

| Phương án | Kết luận |
|---|---|
| **1. Direct Adjustment** — thêm story vào Epic 1 | ✅ **KHẢ THI. Chọn cái này.** Công **Thấp**, rủi ro **Thấp** |
| **2. Rollback** — hoàn tác 1.9/1.10 | ❌ **Không khả thi và ⛔ không cần.** Cả hai làm đúng phạm vi; hạ tầng chúng dựng là thứ làm tiếng Anh **rẻ** |
| **3. MVP Review** — cắt tiếng Anh khỏi v1 | ❌ **Không khuyến nghị.** `§3.2` chốt cả hai cặp ngôn ngữ trong phạm vi; mục tiêu Epic 1 nêu đích danh tiếng Anh. Cắt là **đổi định vị sản phẩm**, ⛔ không phải điều chỉnh sprint |

### ✅ Chọn: **Phương án 1 — Direct Adjustment**

**Lý do:** vấn đề là **thiếu một story**, ⛔ không phải sai kiến trúc hay sai giả định. Mọi thứ tốn công đã có sẵn — parser tham số hoá, `finalize` dùng chung, hai cổng `.mjs`, khuôn `SourceMeta`, `wiktextract_common` gộp theo headword. Nguồn thô **đã tải, đã nằm trong `raw/`**, và dữ liệu đã được **đo thật**.

**Ước công:** 🟢 **Thấp** cho story dữ liệu · 🟡 **Trung bình** cho story tra cứu *(cần AD mới)*
**Rủi ro:** 🟢 **Thấp** — ⛔ không đổi lược đồ, ⛔ không crate mới, ⛔ không chạm `src-tauri` ở story dữ liệu
**Tác động tiến độ:** cộng thêm vào Epic 1; ⛔ không chặn `1-11`/`1-12`/`1-13` *(chúng là đường tiếng Trung)*

---

## 4. Đề xuất sửa chi tiết

### 4.1 `epics.md` — thêm hai story vào Epic 1

**Story mới #1 — `1-10b-dung-du-lieu-tu-dien-tieng-anh`**

> **Là** chủ dự án, **tôi muốn** đầu mục tiếng Anh có mặt trong dữ liệu từ điển đóng gói, **để** cặp Anh → Việt có nền dữ liệu như cặp Trung → Việt đã có.
>
> **Covers:** FR34 · NFR6 *(đo lại)* · NFR8
>
> **AC1** — `viwiktionary` vai A dựng thành nguồn thứ **sáu**, dùng lại `wiktextract_common::parse(reader, "vi", Some("en"))`. ⛔ Không parser mới, ⛔ không crate mới.
> **AC2** — Đối chiếu số đo: **119.039** đầu mục · **190.543** nghĩa · **27.396** ví dụ. Lệch quá 1% ⇒ parser sai.
> **AC3** — `dict_entry.lang = 'en'` cho mọi mục của nguồn này; đối chứng âm: nguồn này ⛔ không sinh hàng `lang='zh'` nào.
> **AC4** — `dict_source` mang đủ bốn trường giấy phép; `attribution` nêu Wiktionary + kaikki + CC-BY-SA/GFDL.
> **AC5** — 🔴 **Quyết định cần chốt trong story:** lớp này vào `dict-core.db` *(nguồn nền — khuyến nghị, giấy phép sạch, FR34 là phạm vi lõi)* hay thành tệp `.db` riêng? ⚠️ Nếu vào `dict-core.db` thì **phải dựng lại** và điền lại `[base].sha256`.
> **AC6** — Cập nhật bảng kế toán NFR6 với số **thật** *(⛔ không phải 40.333.312 của mũi thăm dò nếu hình dạng đóng gói khác)*. Đối chiếu trần 400.000.000.
> **AC7** — `check-dict-build.mjs` Kiểm C/D/E/F đi theo nguồn mới; `RS_FILE_FLOOR` cập nhật nếu thêm tệp `.rs`.

**Story mới #2 — `1-11b-duong-tra-cuu-tieng-anh`** *(chèn sau `1-11`)*

> **Là** người dịch, **tôi muốn** bôi đen một từ tiếng Anh và thấy ngay nghĩa tiếng Việt kèm từ loại, **để** cặp Anh → Việt dùng được thật.
>
> **Covers:** FR34 · FR19 · FR40 *(dùng chung với 1.12)*
>
> **AC1** — Tra chính xác đầu mục tiếng Anh qua cùng cổng `DictionarySource` của 1.13 — ⛔ không mã riêng cho từng ngôn ngữ ngoài phần **chiến lược truy vấn**.
> **AC2** — Biến thể hình thái *(FR40, stemming)* — dùng `Matcher` của **AD-17**, ⛔ không cài riêng.
> **AC3** — 🔴 **Chặn bởi AD mới của Winston** *(§2.4)*: `char_idx` ⛔ không áp cho tiếng Anh.
> **AC4** — Mục từ tiếng Anh hiển thị **nhãn từ loại** + **nghĩa tiếng Việt** *(FR34)*; nguồn ghi rõ, ⛔ không hợp nhất *(AD-19)*.
> **AC5** — NFR1 *(< 100 ms)* đo trên đường tiếng Anh, ⛔ không suy từ số tiếng Trung.

### 4.2 `epics.md` — cập nhật truy vết FR

| Dòng | Từ | Thành |
|---|---|---|
| `:654` | `\| FR34 \| Epic 1 \| Mục từ tiếng Anh \|` | thêm `— dữ liệu: 1.10b · tra cứu: 1.11b` |

### 4.3 `sprint-status.yaml` — thêm hai khoá

```yaml
  1-10-dong-goi-bon-lop-go-roi-thanh-file-doc-lap: done
  1-10b-dung-du-lieu-tu-dien-tieng-anh: backlog        # ← MỚI
  1-11-ba-nhanh-truy-van-tieng-trung: backlog
  1-11b-duong-tra-cuu-tieng-anh: backlog               # ← MỚI
```

### 4.4 `deferred-work.md` — thêm mục bàn giao NFR6

> 🔴 **Dư địa NFR6 chỉ còn 15.675.258 byte sau lớp tiếng Anh.** HVTĐTD + Cổ hán văn phải được **ĐO** trước khi hứa đóng gói. Chủ sở hữu: story nối tiếp của 1.10.

---

## 5. Bàn giao thi công

| Việc | Chủ sở hữu | Trạng thái |
|---|---|---|
| Sửa mơ hồ PRD §8.2/§8.3 | **John (PM)** | ✅ **Xong 2026-08-05** |
| Thêm hai story + truy vết vào `epics.md` | **John (PM)** | ✅ **Xong 2026-08-05** |
| Đồng bộ `sprint-status.yaml` | **John (PM)** | ✅ **Xong 2026-08-05** |
| 🔴 **AD mới — đường tra cứu tiếng Anh** *(§2.4)* | **Winston (Architect)** | 🔴 **CHẶN `1-11b`** |
| 🟡 Kiểm Panel Lookup có hình dạng cho mục tiếng Anh | **Sally (UX)** | 🟡 Chưa kiểm |
| Viết story chi tiết `1-10b` | `bmad-create-story` | Sau khi duyệt |
| Thi công `1-10b` | **Amelia (Dev)** | Sau story |

### Tiêu chí thành công

1. `dict_entry` có **≥ 119.000** hàng `lang='en'` trong dữ liệu đóng gói.
2. **FR34** nghiệm thu được bằng test thật, ⛔ không bằng suy luận.
3. Bảng kế toán **NFR6** cập nhật với số thật và đối chiếu trần 400.000.000.
4. Mục tiêu Epic 1 — *"văn bản tiếng Trung **hoặc tiếng Anh**"* — đúng cả hai vế.

---

## Phụ lục — Checklist điều hướng thay đổi

| § | Mục | Trạng thái |
|---|---|---|
| 1.1–1.3 | Trigger, vấn đề, bằng chứng | ✅ Done — bằng chứng đo thật |
| 2.1 | Epic 1 hoàn thành được như kế hoạch? | ❗ **Action-needed** — không, thiếu hai story |
| 2.2 | Đổi cấp epic | ✅ Done — thêm story trong epic hiện có, ⛔ không epic mới |
| 2.3–2.4 | Epic tương lai · epic mới | ✅ N/A — ⛔ không epic nào bị vô hiệu |
| 2.5 | Đổi thứ tự epic | ✅ N/A |
| 3.1 | Xung đột PRD | ✅ Done — đã sửa 2026-08-05 |
| 3.2 | Xung đột Architecture | ❗ **Action-needed** — AD-26 chỉ ZH, cần AD mới |
| 3.3 | Xung đột UI/UX | ❗ **Action-needed** — Sally kiểm |
| 3.4 | Artifact khác | ✅ Done — hai cổng `.mjs` đi theo, CI ⛔ không đổi |
| 4.1 | PA 1 Direct Adjustment | ✅ **Viable — CHỌN** |
| 4.2 | PA 2 Rollback | ❌ Not viable |
| 4.3 | PA 3 MVP Review | ❌ Not viable |
| 4.4 | Chọn đường | ✅ Done — Phương án 1 |
| 5.1–5.5 | Thành phần đề xuất | ✅ Done |
| 6.1–6.3 | Rà soát & duyệt | ✅ **Ice duyệt 2026-08-05** |
| 6.4 | Đồng bộ `sprint-status.yaml` | ✅ Done — thêm `1-10b`, `1-11b` |
| 6.5 | Bàn giao | ✅ Done — 4 mục vào `deferred-work.md` |
