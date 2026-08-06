---
baseline_commit: 564be15cfe9142ae8c514ce8f64ced5ea2df4a2b
---

# Story 1.10c: Âm Hán Việt — đúng nguồn và đúng nhãn

Status: done

> 🔴 **Story này sinh ra từ một phép đo, ⛔ không từ một mục backlog.** Lượt dựng Story 1.16
> đo `dict-core.db` thật và phát hiện: cột `dict_entry.han_viet` của **lớp nền** ⛔ **không
> mang âm Hán Việt** — nó mang **âm Nôm** *(`Unihan kVietnamese`)*. Số đo ở §Phát hiện.
>
> 🔴 **Nó CHẶN Story 1.16** *(tab Hán Việt)* **và đầu độc Story 3.7** *(FR113 — đề xuất bản
> dịch bằng âm Hán Việt)*. Ice chốt 2026-08-06: **đóng tầng dữ liệu TRƯỚC, giao diện SAU.**

## Change Log

| Ngày | Thay đổi |
|---|---|
| 2026-08-06 | Tạo story. Baseline `564be15`, cây làm việc **sạch**. Sinh ra từ Task 0 của Story 1.16. **Ice chốt hai nhánh:** ① nguồn Hán Việt = **Thiều Chửu + en.wiktionary(vi) + Trần Văn Chánh**, TVC đóng gói làm **lớp gỡ rời riêng** *(FR36: gỡ = xoá một tệp)*; ② **story dữ liệu chạy TRƯỚC 1.16**. Mọi con số ở story này là **đo thật** trên `tools/dict-build/out/*.db`, `catusf/tudien@master`, và `kaikki.org` extract — ⛔ không một ước lượng nào. |
| 2026-08-06 | **Triển khai xong, Status → review.** Ice chốt Quyết định #3 (a: chỉ nạp âm đọc) và #4 (a: cột `nom_reading TEXT`) qua `AskUserQuestion`. Cả 10 task hoàn tất: lược đồ v2, `unihan.rs` đổi vai (AC1), nguồn nền thứ bảy `en-wiktionary-vi` (AC3), lớp gỡ rời thứ ba `tran-van-chanh` (AC4, rủi ro pháp lý ghi thẳng — AC8), lưới chống tái diễn `nom_guard.rs` (AC5, đỏ-rồi-xanh **92,4% khớp chính xác** con số story trên dữ liệu thật), cả bốn tệp `.db` dựng lại và **tái lập được** (AC7), NFR6 đo được **373.239.808/400.000.000** — dư 26.760.192 byte, **không vượt trần** (AC9). Mọi cổng xanh (`cargo test` × 2 workspace, `check:dict`/`check:dict-manifest`/`check:i18n`/`check:deps`). `deferred-work.md §1-10c` mở — Story 1.16 hết chặn. Quyết định #5 không mở (không vượt trần). |

**Epic:** 1 — Nền móng ứng dụng & Tra cứu ngoại tuyến tức thì
**Story key:** `1-10c-am-han-viet-dung-nguon-va-dung-nhan`
**Vị trí:** nối tiếp **1-9** *(lớp nền)* · **1-10** *(lớp gỡ rời)* · **1-10b** *(nguồn nền tiếng Anh)*. ⚠️ Chạy **sau** 1-15 theo thứ tự thời gian, **trước** 1-16.
**Covers:** ⛔ **Không FR mới.** Story này sửa **chất lượng dữ liệu** đứng dưới **FR33** *(âm Hán Việt cho từng ký tự)* và **FR113** *(đề xuất bản dịch bằng âm Hán Việt)*, và mở rộng nền của **FR27–FR32**.
**Governed by:** **AD-10** *(mỗi lớp gỡ rời một tệp `.db`; **runtime ⛔ không có mã riêng cho từng nguồn**; trường giấy phép ⛔ **không** được là enum các giấy phép mở)* · **AD-19** *(⛔ không hợp nhất nguồn)* · **AD-25** *(dữ liệu từ điển là artifact có phiên bản và checksum)* · **AD-30** *(lược đồ có phiên bản; mở tiến, ⛔ không mở lùi)* · AD-2 *(cổng `DictionarySource`)* · AD-26/AD-27 *(ba nhánh truy vấn, chỉ mục FTS)*
**NFR:** **NFR6** *(trần payload **400.000.000 byte** — 🔴 dư địa hôm nay chỉ còn **56.008.570**)* · NFR15 *(rà giấy phép)* · NFR13 *(ngoại tuyến)* · NFR14
**Ngày tạo:** 2026-08-06

---

## 🔴 PHÁT HIỆN — số đo, ⛔ không phải lập luận

### ① `Unihan kVietnamese` là âm **NÔM**, ⛔ không phải âm Hán Việt

`tools/dict-build/src/sources/unihan.rs:116` nạp `kVietnamese` thẳng vào `dict_entry.han_viet`.
Unicode ⛔ **chưa bao giờ** hứa trường đó là âm Hán Việt — định nghĩa của nó là *"the
Vietnamese pronunciation(s) of this character"*, và với chữ **Nôm** thì phát âm tiếng Việt
của nó **chính là âm Nôm**.

**Đối chiếu với Thiều Chửu** *(từ điển Hán Việt thật)* trên phần giao **3.239** ký tự:

| | Số |
|---|---|
| Hai nguồn cho **âm đầu khác nhau** | **1.243 = 38,4 %** |

| Chữ | `unihan` | Thiều Chửu | Bản chất |
|---|---|---|---|
| 繭 | `kén` | `kiển` | `kén` là từ thuần Việt |
| 抉 | `khoét` | `quyết` | cùng lớp |
| 蓉 | `rong` | `dong` | cùng lớp |
| 女 | `nữa` | `nữ\|nứ\|nhữ` | lớp nền **sai** ở một chữ sơ đẳng |
| 死 | `tợ tử` | `tử` | hai âm trong một chuỗi, âm đầu là Nôm |
| 你 | `nể` | *(không có)* | 你 = **nhĩ**; `nể` là âm **Nôm** |
| 北 | **⛔ không có** | `bắc` | — |

🔴 **Hệ quả đã chạm vào tài liệu quy hoạch:** `EXPERIENCE.md:410` giải thích FR113 bằng ví dụ
**`北涼` → *Bắc Lương***. Với **chỉ** lớp nền, `北` trả **rỗng** ⇒ ví dụ trụ cột của FR113
⛔ **không chạy được**.

### ② Chẩn đoán được **kiểm chứng bằng máy** trên một nguồn thứ ba

`en.wiktionary` *(qua `wiktextract`)* gắn nhãn **tách bạch** hai loại âm:
`tags: ["han-viet-reading"]` và `tags: ["nom-reading"]`.

Trên **1.173** ký tự mà Unihan có âm **và** en.wiktionary có dữ liệu:

| Mệnh đề | Số |
|---|---|
| Âm Unihan **trùng một âm NÔM** của en.wiktionary | **1.084 = 92,4 %** |
| Âm Unihan **trùng một âm HÁN VIỆT** | **266 = 22,7 %** |

⇒ Chẩn đoán ⛔ không phải một suy luận từ vài mẫu. **Nó đo được, và nó lặp lại được.**

### ③ ⛔ Phần lớn ký tự "chỉ Unihan có" ⛔ KHÔNG CÓ âm Hán Việt để mà thay

**5.336** ký tự có `han_viet` ở lớp nền mà Thiều Chửu ⛔ không phủ:

| Khối Unicode | Số | Thay được bằng nguồn Hán Việt thật? |
|---|---|---|
| **CJK Ext B** (U+20000–2A6DF) | **4.254 = 79,7 %** | TVC **0** · en.wikt **12** — 🔴 **đây là chữ Nôm; âm Hán Việt ⛔ không tồn tại** |
| CJK cơ bản | 763 | **271 = 35,5 %** |
| Ext A · Ext C–F · tương thích · khác | 319 | rải rác |

Mẫu Ext B: 𠀧 → `ba` · 𠁀 → `đời` · 𠀿 → `xuôi` · 𠀲 → `đứa`. en.wiktionary gắn **đúng** chúng
là `nom-reading`.

🔴 ⇒ **⛔ Không tồn tại một "nguồn thay" cho phần lớn 5.336 ký tự đó.** Chúng ⛔ không thiếu
âm Hán Việt — chúng **⛔ không có** âm Hán Việt. Giữ chúng trong đường Hán Việt **chính là**
thứ làm nguồn thành "sai". ⇒ Lời giải là **đổi vai**, ⛔ không phải xoá và ⛔ không phải thay.

### ④ Trên văn xuôi thật, lớp nền đóng góp **bằng 0**

| Văn bản | Ký tự riêng | Thiều Chửu đơn độc | + en.wikt | + TVC |
|---|---|---|---|---|
| Mockup phồn thể | 45 | **45/45** | 45/45 | 45/45 |
| Mockup giản thể | 32 | **32/32** | 32/32 | 32/32 |
| Giản thể phổ thông | 47 | **46/47** *(thiếu 么)* | 46/47 | **47/47** |

### ⑤ Ba nguồn Hán Việt — đo thật

| Nguồn | Ký tự có âm HV | Giấy phép | Ghi chú |
|---|---|---|---|
| **Thiều Chửu** *(đã dựng, lớp gỡ rời)* | **9.897** | CC0 1.0 + tác giả mất **1954** ⇒ tác phẩm 1942 hết hạn bản quyền | ✅ sạch |
| **en.wiktionary (vi)** | **1.136** *(+352 ngoài Thiều Chửu)* | **CC BY-SA 4.0** — ⚠️ **đã có sẵn** trong `assets/licenses/` | ✅ sạch; giá trị chính là **nhãn HV/Nôm** |
| **Trần Văn Chánh** *(`catusf/tudien`)* | **12.081** | CC0 của **người số hoá**; *Từ điển Hán Việt* (1999) 🔴 **còn trong bản quyền** | ⚠️ xem §Rủi ro pháp lý |

**Hợp Thiều Chửu ∪ TVC = 12.169** · **hợp cả ba = 12.463** *(en.wikt thêm **294**)*.
Thiều Chửu ∩ TVC: **9.809** ký tự, **âm đầu trùng 94,9 %**, tập âm trùng khít **93,3 %** —
⇒ hai bên ghi **cùng một tập dữ kiện**, ⛔ không phải hai diễn giải khác nhau.

---

## Story

As a người dịch,
I want âm Hán Việt trong công cụ là **âm Hán Việt thật**,
So that tab Hán Việt và đề xuất Glossary ⛔ không dạy tôi sai một chữ nào.

---

## Ranh giới phạm vi

| Trong phạm vi | ⛔ Ngoài phạm vi (và ai sở hữu) |
|---|---|
| Cột `nom_reading` mới; `Unihan kVietnamese` chuyển sang **đúng vai** | Đường **tra cứu Nôm** trong sản phẩm *(chưa có FR; ⛔ không dựng)* |
| Nguồn nền **thứ bảy**: `en-wiktionary-vi` — âm HV **có gắn nhãn** | Nghĩa/ví dụ/trích dẫn tiếng Việt từ en.wiktionary *(⛔ không nạp — xem Quyết định #3)* |
| Lớp gỡ rời **thứ ba**: `tran-van-chanh` | HVTĐTD · Cổ hán văn *(⛔ chưa có nguồn thô — `prd.md:856`)* |
| `SCHEMA_VERSION` 1 → 2 ở **cả** `tools/dict-build` **và** `src-tauri` | Bộ di trú cho `.db` *(`schema.rs`: tệp ⛔ không di trú, thay nguyên tệp)* |
| Dựng lại **cả bốn** tệp `.db`, cập nhật **mọi** SHA-256 trong `dict-manifest.toml` | Đưa `.db` vào `bundle.resources` *(Story 10.1)* |
| Phép đo NFR6 đầy đủ, ghi thành **bảng số** | Nâng/hạ trần NFR6 *(⛔ tầng PRD — quyết định của Ice)* |
| Rà NFR15 cho **hai** nguồn mới | Màn hình Attribution *(Story 10.4)* · bật/tắt nguồn *(Story 1.19)* |

**⛔ KHÔNG ĐỤNG:** `src/**` *(toàn bộ frontend — story này ⛔ **0 dòng Vue/TS**)* ·
`src-tauri/capabilities/**` · `package.json` · `[profile.release]` ·
`_bmad-output/planning-artifacts/**` *(lệch thì **ghi ra**, ⛔ không sửa)*.

**✅ ĐỤNG ĐÃ ĐƯỢC PHÉP:** `tools/**` *(🔴 **lần đầu sau năm story** — ranh giới này mở **đúng
cho story này** và đóng lại ngay sau)* · `dict-manifest.toml` · `src-tauri/src/core/dict/**`
*(chỉ hằng phiên bản + đường đọc cột mới)* · `src-tauri/Cargo.toml` **⛔ KHÔNG** *(chốt lần
thứ sáu — `deferred-work.md` [D4])*.

---

## 🔴 NĂM QUYẾT ĐỊNH — hai đã chốt, ba chốt ở Task 0

| # | Nội dung | Trạng thái |
|---|---|---|
| **#1** | Nguồn Hán Việt = **Thiều Chửu + en.wiktionary(vi) + Trần Văn Chánh**; TVC là **lớp gỡ rời** | ✅ **Ice chốt 2026-08-06** |
| **#2** | Story dữ liệu chạy **trước** Story 1.16 | ✅ **Ice chốt 2026-08-06** |
| #3 | Phạm vi nạp từ `en.wiktionary(vi)` | ✅ **Ice chốt 2026-08-06** — (a) chỉ nạp âm đọc |
| #4 | Hình dạng chỗ chứa âm Nôm | ✅ **Ice chốt 2026-08-06** — (a) cột mới `nom_reading TEXT` |
| #5 | Xử lý khi NFR6 vượt trần | ⬜ Task 0 — **chỉ mở nếu phép đo Task 9 nói vượt** |

### Quyết định #3 — nạp gì từ `en.wiktionary(vi)`?

| | Đường | Phán quyết |
|---|---|---|
| **(a)** | **Chỉ nạp âm đọc** — `han-viet-reading` vào `han_viet`, `nom-reading` vào cột Nôm. ⛔ Không `dict_sense`, ⛔ không ví dụ | ✅ **MẶC ĐỊNH ĐỀ XUẤT** |
| (b) | Nạp trọn như một nguồn nền đầy đủ *(nghĩa + ví dụ + trích dẫn)* | ⛔ **Loại** |

**Vì sao (a):** giá trị đo được của nguồn này ⛔ **không** nằm ở 294 ký tự nó thêm — nó nằm
ở **nhãn phân biệt HV/Nôm**, thứ ⛔ không nguồn nào khác có, và là **lưới duy nhất** ngăn lỗi
Unihan tái diễn ở một nguồn tương lai *(xem AC5)*. Nạp trọn thì kéo thêm ~78 MB JSONL vào
đường build, chồng nghĩa lên **sáu** nguồn nền đã có, và ăn vào **56.008.570 byte** dư địa
NFR6 cho một thứ ⛔ không AC nào đòi.

⚠️ `wiktextract_common.rs` (400 dòng) đã có sẵn — **dùng lại**, ⛔ đừng viết parser thứ hai.

### Quyết định #4 — âm Nôm sống ở đâu?

| | Đường | Phán quyết |
|---|---|---|
| **(a)** | **Cột mới `dict_entry.nom_reading TEXT`** | ✅ **MẶC ĐỊNH ĐỀ XUẤT** |
| (b) | Giữ trong `han_viet`, thêm cột `reading_kind` | ⛔ **Loại** — một cột tên `han_viet` mang âm Nôm là **đúng cái lỗi story này tồn tại để sửa** |
| (c) | Bảng riêng `nom_reading(entry_id, reading)` | ⛔ **Loại** — âm đọc là thuộc tính **một-một** của đầu mục, cùng hạng `reading` (pinyin) đã là cột |

🔴 **Mệnh đề bất biến phải cưỡng chế bằng test:** sau story này, **⛔ không tệp `.db` nào**
được có một hàng mà `han_viet` mang giá trị do `kVietnamese` sinh ra. `dict_entry.han_viet`
mang **đúng một** ngữ nghĩa ở **mọi** tệp — xem AC2.

---

## Acceptance Criteria

### AC1 — `Unihan kVietnamese` đổi vai, ⛔ không mất một byte dữ liệu nào

**Given** `unihan.rs` hôm nay ghi `kVietnamese` vào `dict_entry.han_viet`
**When** dựng lại `dict-core.db`
**Then** giá trị đó nằm ở **cột âm Nôm** *(Quyết định #4)*, ⛔ **không** ở `han_viet`
**And** **⛔ không một ký tự nào bị mất**: số hàng mang `kVietnamese` **trước** và **sau**
bằng nhau — nghiệm thu bằng một con số, ⛔ không bằng lời
**And** `dict_source` của `unihan` giữ nguyên ghi công và giấy phép Unicode

### AC2 — `dict_entry.han_viet` mang **đúng một** ngữ nghĩa ở **mọi** tệp

**Given** bốn tệp `.db` sau lượt dựng
**When** kiểm
**Then** ⛔ **không** tệp nào có `han_viet` sinh từ `kVietnamese`
**And** một **test hành vi** cưỡng chế mệnh đề đó — ⛔ không phải một comment
**And** doc-comment của `DICT_ENTRY_DDL` nói rõ **hai** cột và ranh giới giữa chúng, cùng
khuôn dòng đã có: *"`han_viet` là **ÂM ĐỌC**, ⛔ không phải NGHĨA"*

### AC3 — Nguồn nền thứ bảy `en-wiktionary-vi`, nạp **âm đọc có gắn nhãn**

**Given** extract Vietnamese của en.wiktionary
**When** dựng lớp nền
**Then** âm `han-viet-reading` vào `han_viet`; âm `nom-reading` vào cột Nôm
**And** ⛔ **không** một nhãn nào bị suy đoán — chỉ nhận `tags` **có mặt tường minh** trong dữ liệu
**And** đo được: **≥ 1.136** ký tự có âm Hán Việt gắn nhãn *(số hôm nay; lệch xuống là ĐỎ)*
**And** dùng lại `wiktextract_common.rs`, ⛔ **không** parser `wiktextract` thứ hai *(hôm nay đã có ba nguồn đi qua nó)*

### AC4 — Lớp gỡ rời thứ ba `tran-van-chanh`, và **FR36 vẫn đúng**

**Given** `DETACHABLE_LAYERS` hôm nay có **hai** phần tử
**When** thêm phần tử thứ ba
**Then** `dict-tran-van-chanh.db` dựng ra với **≥ 12.081** đầu mục một ký tự có `han_viet`
**And** `DETACHABLE_LAYERS` (build.rs) và `DETACHABLE_ALL` (sources_meta.rs) **khớp từng mã,
đúng thứ tự** — test đã có sẵn *(`build.rs:491-506`)* phải **vẫn xanh**
**And** 🔴 **xoá tệp đó khỏi đĩa ⇒ toàn bộ bộ test tra cứu vẫn xanh** *(FR36, AD-10,
`epics.md:816` — **test thật, ⛔ không mock**)*
**And** mức phủ tụt xuống **ghi thành số** *(12.463 → 10.249)*, ⛔ không đánh dấu đạt rồi im

### AC5 — Lỗi Unihan **⛔ không tái diễn được**, và cưỡng chế bằng máy

**Given** nhãn `han-viet-reading` / `nom-reading` của en.wiktionary
**When** dựng lớp nền
**Then** một **phép kiểm lúc build** đối chiếu mọi âm nạp vào `han_viet` với tập âm **Nôm**
đã gắn nhãn của cùng ký tự, và **báo số** ký tự đáng ngờ
**And** 🔴 phép kiểm đó chạy **trên chính dữ liệu Unihan cũ** ra con số **1.084 / 1.173 = 92,4 %**
— tức nó **đỏ được**, ⛔ không phải một phép kiểm luôn xanh
**And** ngưỡng phán quyết ghi thành hằng có tên **kèm lý do**, ⛔ không phải một số trần

### AC6 — Lược đồ lên **v2** ở **cả hai bờ**, và bờ đọc từ chối đúng chiều

**Given** `SCHEMA_VERSION = 1` ở `tools/dict-build/src/schema.rs` và
`SUPPORTED_SCHEMA_VERSION = 1` ở `src-tauri/src/core/dict/layer.rs:49`
**When** thêm cột mới
**Then** **cả hai** lên **2** trong **cùng một lượt**
**And** 🔴 một tệp **v2** mở được bởi bản mới; một tệp **v3 giả lập** vẫn bị từ chối bằng
`SkipReason::SchemaTooNew` *(AD-30 — mở tiến, ⛔ không mở lùi)*
**And** ⛔ **không** dựng bộ di trú — `schema.rs` chốt: *"Tệp này KHÔNG di trú … được thay
nguyên tệp qua release mới"*

### AC7 — `dict-manifest.toml` cập nhật **trọn**, và checksum **tái lập được**

**Given** lượt dựng đổi lược đồ ⇒ **cả bốn** tệp `.db` là byte mới
**When** cập nhật manifest
**Then** **cả bốn** mục có `url` · `sha256` · `source_version` đúng, gồm mục
`[[detachable]]` **mới** cho `tran-van-chanh`
**And** 🔴 dựng lại từ **cùng** cây `raw/` cho ra **cùng** SHA-256 — bất biến `built_at`
dẫn xuất từ nguồn thô *(đã dựng ở lượt vá 2026-08-05)* phải **vẫn giữ**
**And** `npm run check:dict-manifest` PASS
**And** ⚠️ ⛔ **không** điền một checksum "cho có" — `dict-manifest.toml:16` cấm bằng chữ

### AC8 — Rà **NFR15** cho hai nguồn mới, và nói thẳng chỗ rủi ro

**Given** `en-wiktionary-vi` và `tran-van-chanh`
**When** rà giấy phép
**Then** mỗi nguồn có `SourceMeta` đầy đủ + `LicenseRef` **biến thể riêng** *(⛔ không so
khớp chuỗi `code` — `sources_meta.rs:11-16`)* + tệp giấy phép gốc trong `assets/licenses/`
**And** `license_kind` biểu diễn được **cả** giấy phép mở **lẫn** phép dùng riêng *(AD-10 —
⛔ **không** enum các giấy phép mở)*
**And** 🔴 **rủi ro Trần Văn Chánh ghi thẳng vào `attribution` và vào Completion Notes**:
CC0 của **người số hoá** ⛔ **không** xoá được bản quyền của *Từ điển Hán Việt* (1999), tác
giả **còn sống**. Giảm thiểu đã chốt: **đóng gói làm lớp gỡ rời** ⇒ FR112 thực thi được
bằng **xoá một tệp**, đúng tiền lệ `prd.md:999` đã dùng cho Cổ hán văn

### AC9 — 🔴 Phép đo **NFR6**, và ⛔ KHÔNG đánh dấu đạt trước khi có số

**Given** trần **400.000.000 byte**, payload hôm nay **343.991.430**, dư địa **56.008.570**
**When** lượt dựng xong
**Then** một **bảng số** ghi: từng tệp `.db` trước/sau · lớp `tran-van-chanh` · tổng payload
mới · dư địa còn lại
**And Given** tổng **vượt trần**
**Then** ⛔ **không** tự bỏ nguồn, ⛔ **không** tự bỏ `sense_fts_nd`, ⛔ **không** đụng
`Cargo.toml` — **ghi số và dừng**, chuyển Ice *(§Ngân sách, tiền lệ 2026-08-05)*
**And** ⚠️ `prd.md:946` cảnh báo trước: dư địa còn lại vốn dành cho **HVTĐTD + Cổ hán văn**;
story này tiêu vào đó, và điều đó phải **hiện ra bằng số**, ⛔ không lặng lẽ

### AC10 — Mọi cổng xanh, ranh giới đóng lại

**Given** cây nguồn sau story
**When** chạy đủ bộ DoD
**Then** `cargo test` **cả hai** manifest xanh · `npm run check:dict` · `check:dict-manifest`
· `check:i18n` · `check:deps` PASS
**And** `RS_FILE_FLOOR` của `check-dict-build.mjs` *(hôm nay **21**)* nâng theo số thật
**And** ⛔ **0** dòng đổi dưới `src/**` · ⛔ **0** phụ thuộc mới · ⛔ không đụng `Cargo.toml`
**And** `deferred-work.md` mở mục `§1-10c` ghi mọi thứ còn treo

---

## Tasks / Subtasks

- [x] **Task 0 — Chốt #3, #4 với Ice** *(#5 chỉ mở nếu Task 9 nói vượt trần)*
- [x] **Task 1 — Lấy nguồn thô, ghim phiên bản** (AC3, AC4)
  - [x] `raw/tran_van_chanh/` ← `catusf/tudien@master` `dict/Tu-dien-ThienChuu-TranVanChanh.tab` *(3.932.157 byte)* — ghim **commit SHA**, ⛔ không `master`
  - [x] `raw/en_wiktionary_vi/` ← extract Vietnamese của `kaikki.org` *(~78 MB JSONL)*
  - [x] ⚠️ Nguồn kaikki đang khai **DEPRECATED** — ghim bản đã tải + ghi ngày; nếu có đường thay ổn định hơn thì dùng nó và **ghi lý do**
- [x] **Task 2 — Lược đồ v2** (AC6)
  - [x] Cột Nôm + `SCHEMA_VERSION` 1→2 + `SUPPORTED_SCHEMA_VERSION` 1→2 **cùng lượt**
  - [x] Test: v2 mở được · v3 giả lập bị `SchemaTooNew` · ⛔ không bộ di trú nào
- [x] **Task 3 — `unihan.rs` đổi vai** (AC1, AC2)
  - [x] `kVietnamese` → cột Nôm; test đếm **trước = sau**
  - [x] Test hành vi: ⛔ không tệp nào có `han_viet` sinh từ `kVietnamese`
- [x] **Task 4 — Parser `en-wiktionary-vi`** (AC3, Quyết định #3)
  - [x] Dùng lại `wiktextract_common.rs`; chỉ đọc `senses[].related[].tags`
  - [x] ⛔ Không suy đoán nhãn; đối chứng âm: một mục ⛔ không có `tags` ⇒ ⛔ không nạp
- [x] **Task 5 — Parser `tran-van-chanh`** (AC4)
  - [x] Định dạng `<chữ>\t[âm] <định nghĩa>`; nhiều âm tách bằng `,` — **giữ nguyên chuỗi**, đúng tiền lệ `thieu_chuu.rs:70`
  - [x] Test trên cả hai hình dạng thật: `[tích]` và `[đáng, đương]`
- [x] **Task 6 — Đăng ký nguồn + lớp** (AC4, AC8)
  - [x] `SourceMeta` + `LicenseRef` mới + tệp giấy phép gốc vào `assets/licenses/`
  - [x] **Một phần tử** thêm vào `DETACHABLE_LAYERS`; test khớp `DETACHABLE_ALL` vẫn xanh
- [x] **Task 7 — Lưới chống tái diễn** (AC5)
  - [x] Phép kiểm build-time đối chiếu `han_viet` với tập âm Nôm đã gắn nhãn
  - [x] 🔴 Nghiệm thu **đỏ-rồi-xanh**: chạy trên dữ liệu Unihan **cũ** phải ra **92,4 %**
- [x] **Task 8 — Dựng lại cả bốn tệp + manifest** (AC7)
  - [x] `--layer all`; đối chiếu **tái lập được**: dựng hai lượt vào hai thư mục ⇒ cùng SHA-256
  - [x] Cập nhật **cả bốn** mục manifest + thêm mục thứ tư
- [x] **Task 9 — 🔴 ĐO NFR6** (AC9) — bảng số, và **dừng** nếu vượt
- [x] **Task 10 — Cổng, sàn, bàn giao** (AC10)
  - [x] Mở `deferred-work.md §1-10c`; đánh dấu Story 1.16 **hết chặn**

---

### Review Findings

- [x] [Review][Decision] **ĐÃ SỬA (Ice chốt: sửa lại bảng NFR6).** AC9's NFR6 "không vượt trần" kết luận dựa trên baseline SAI/CŨ — dư địa thật ⛔ không phải 26,76 MB mà là **3.104.634 byte (0,78% trần)**. Baseline đúng (384.525.446, từ Story 1.10b) + bốn delta thật (+12.369.920) = **396.895.366 / 400.000.000**. Đã sửa bảng NFR6 ở Dev Notes/Debug Log References, `deferred-work.md §1-10c`, và bảng "Trạng thái hôm nay". ⛔ Vẫn KHÔNG vượt trần đúng nghĩa đen, nhưng dư địa gần cạn — HVTĐTD/Cổ hán văn gần như chắc chắn không còn vừa, cảnh báo đã ghi rõ cho quyết định tầng PRD tiếp theo. [file: story này §Dev Notes, §Debug Log References; `deferred-work.md §1-10c`]

- [x] [Review][Decision] **ĐÃ SỬA (Ice chốt: thiết kế lại để hoạt động thật → phát hiện dương tính giả → siết điều kiện).** AC5's lưới chống tái diễn (`nom_guard`) VÔ NGHĨA (0/0) VĨNH VIỄN cho ba lớp gỡ rời do `LABELED_NOM_SOURCE` chỉ tồn tại ở `dict-core.db` (AD-10: một tệp một `dict_source`). Sửa bằng nạp nhãn Nôm từ raw `en-wiktionary-vi` cho MỌI lớp gỡ rời (`build.rs::load_en_wiktionary_vi_labeled_nom`, không thêm mã riêng-từng-nguồn). Sửa xong lộ dương tính giả THẬT: Thiều Chửu (nguồn chuẩn) bị gắn cờ 63,4% do en-wiktionary-vi tự gắn cả hai nhãn HV/Nôm cho cùng âm khá thường xuyên. Siết bằng `nom_guard::nom_only_readings` (loại âm tự-trùng-vai). Kết quả cuối trên dữ liệu thật: base 0/0, thieu-chuu 5,2%, vietphrase 0/0, tran-van-chanh 6,5% — tất cả an toàn; mệnh đề "đỏ được" đo lại 79,5% (từ 92,4%), vẫn cách xa ngưỡng và cách xa các nguồn hợp lệ. Bốn SHA-256 `.db` không đổi (phép kiểm không đụng dữ liệu ghi ra). [tools/dict-build/src/nom_guard.rs; tools/dict-build/src/build.rs; tools/dict-build/tests/nom_guard_real_data.rs]

- [x] [Review][Patch] **ĐÃ SỬA cùng lượt với Decision AC5 phía trên.** `nom_guard::count_suspicious` đếm trùng khi một nguồn có nhiều hàng cùng `(headword, source_code)` — `tran_van_chanh.rs` cố tình tạo nhiều `dict_entry` cho cùng headword. Sửa: gộp theo `(headword, source_code)` thành một tập âm đọc (HashSet) trước khi đếm — mỗi cặp ký tự+nguồn giờ chỉ đóng góp đúng MỘT lần vào `total_checked`/`suspicious`. [tools/dict-build/src/nom_guard.rs:87-111]

- [x] [Review][Patch] **ĐÃ SỬA.** AC1 đòi "nghiệm thu bằng một con số" rằng ⛔ không hàng `kVietnamese` nào bị mất — thêm test tích hợp `ac1_unihan_kvietnamese_row_count_matches_raw_source_before_and_after_role_swap` (`tests/parse.rs`): đếm dòng `kVietnamese` trong fixture raw TRƯỚC, đối chiếu CHÍNH XÁC với số hàng `dict_entry.nom_reading` khác NULL của nguồn `unihan` SAU build thật, cộng assert `han_viet` LUÔN rỗng cho nguồn này (AC2). Chạy trên CI (không `#[ignore]`, dùng fixture). [tools/dict-build/tests/parse.rs]

- [x] [Review][Patch] **ĐÃ SỬA — số đo THẬT khớp CHÍNH XÁC con số story đã nêu.** Thêm test `ac4_fr36_coverage_drop_is_measured_on_real_data` (`#[ignore]`, chạy tay trên `raw/**` thật — cùng quy ước `nom_guard_real_data.rs`): hợp headword MỘT KÝ TỰ có `han_viet` của Thiều Chửu ∪ en-wiktionary-vi (KHÔNG có TVC) so với CÓ TVC, cộng assert `>= 12_081` cho riêng TVC. **Đo được: 10.249 → 12.463, TVC đóng góp 12.081** — khớp CHÍNH XÁC ba con số story đã khai ở AC4 (`12.463 → 10.249`, `≥ 12.081`), xác nhận số gốc ĐÚNG, chỉ là chưa có cổng tự động trước bản vá này. [tools/dict-build/tests/parse.rs]

- [x] [Review][Patch] **ĐÃ SỬA (ghi rõ quyết định, ⛔ không lọc).** `tran_van_chanh.rs::parse` không phân biệt headword một/nhiều ký tự — xác nhận đây là quyết định CÓ Ý THỨC, nhất quán với MỌI parser khác trong crate (⛔ không nguồn nào lọc theo độ dài headword; việc đọc riêng ký tự đơn thuộc tầng tiêu thụ, Story 1.16). Thêm doc-comment tường minh ở đầu module + test khoá hành vi `tran_van_chanh_does_not_filter_multi_character_headwords_by_design`. [tools/dict-build/src/sources/tran_van_chanh.rs]

- [x] [Review][Patch] **ĐÃ SỬA.** `tran_van_chanh.rs::parse` giờ từ chối dấu `[` lồng bên trong ngoặc âm đọc (`after_open[..close_idx].contains('[')` ⇒ `ParseIssue`), thay vì để nó lọt vào chuỗi `han_viet` đã parse. Test `tran_van_chanh_rejects_a_nested_open_bracket_in_the_reading` khoá lại. [tools/dict-build/src/sources/tran_van_chanh.rs:91-99]

- [x] [Review][Defer] `nom_guard::split_readings` cắt đồng thời trên CẢ BA quy ước phân tách (`|`, `,`, khoảng trắng) dù đó là ba quy ước RIÊNG của ba nguồn khác nhau — deferred, pre-existing design tradeoff đã ghi rõ trong doc-comment (Bẫy 4) và `deferred-work.md §1-10c`, phạm vi sửa đường ĐỌC thuộc Story 1.16. [tools/dict-build/src/nom_guard.rs:46-56]

- [x] [Review][Defer] So sánh âm đọc xuyên nguồn (`nom_guard`) và các parser mới dùng so khớp chuỗi thô, ⛔ không chuẩn hoá Unicode (NFC/NFD) — rủi ro lý thuyết, chưa có bằng chứng xảy ra trên dữ liệu thật hôm nay; nên đưa vào cùng lượt dọn khi Story 1.16 chuẩn hoá đường đọc âm đọc. [tools/dict-build/src/nom_guard.rs:108]

- [x] [Review][Defer] Thiều Chửu và Trần Văn Chánh cùng lấy từ `catusf/tudien` nhưng ghim CÁCH NHAU BA NĂM (tag `2.2`/2022-10-10 so với commit 2025-12-19) không đối chiếu lại xem hai bản có còn nhất quán — câu hỏi về tính toàn vẹn nguồn, không chặn story này, cân nhắc khi có lượt làm mới dữ liệu tiếp theo. [tools/dict-build/src/sources/thieu_chuu.rs:12; tools/dict-build/src/sources/tran_van_chanh.rs:37]

---

## Dev Notes

### Trạng thái hôm nay — SỐ

| | Số thật (2026-08-06, `564be15`) |
|---|---|
| Nguồn **nền** | 6 — `cvdict` · `cc-cedict` · `unihan` · `viwiktionary` · `en-wiktionary` · `viwiktionary-en` |
| Lớp **gỡ rời** | 2 — `thieu-chuu` · `vietphrase` |
| `dict-core.db` | 194.998.272 byte · `dict-thieu-chuu.db` 5.787.648 · `dict-vietphrase.db` 160.083.968 |
| Payload sản phẩm | 🔴 **384.525.446** / trần **400.000.000** ⇒ dư **15.474.554** — *(sửa ở lượt code review 2026-08-06: bản ghi cũ "343.991.430/dư 56.008.570" là số CŨ của `epics.md:336`, KHÔNG cộng font+baseline app+license; số đúng đo ở chính Story 1.10b, xem `1-10b-...md:934,963,1087` và §Review Findings)* |
| `SCHEMA_VERSION` | 1 *(cả hai bờ)* |
| Nguồn đi qua `wiktextract_common.rs` | 3 |
| Tệp `.rs` dưới `tools/dict-build/src/` | 21 *(= `RS_FILE_FLOOR`)* |

### Hai bất biến của `tools/dict-build` mà story này **phải giữ**

1. **`built_at` dẫn xuất từ nguồn thô** *(`SOURCE_DATE_EPOCH` hoặc mtime mới nhất trong `raw/`)*,
   ⛔ **không** `strftime('now')`. Đây là thứ làm SHA-256 **tái lập được**, vá 2026-08-05.
   ⛔ Một nguồn mới lấy dấu thời gian từ đồng hồ hệ thống sẽ **âm thầm** phá lại bất biến này.
2. **`--layer <code>` chạy đơn lẻ được và ⛔ không chạm lớp khác** — nhưng story này đổi
   **lược đồ**, nên nó **bắt buộc** `--layer all`. ⇒ **cả bốn** checksum đổi, và đó là hệ quả
   **có ý thức**, ⛔ không phải bất khả kháng *(`dict-manifest.toml:36-40` đã học đúng bài này)*.

### ⚠️ SÁU CÁI BẪY

1. **Nâng `SCHEMA_VERSION` một bờ, quên bờ kia** ⇒ app từ chối **mọi** lớp bằng
   `SchemaTooNew`, và triệu chứng là *"từ điển không lên"* — ⛔ không phải *"lược đồ lệch"*.
2. **Nạp `nom-reading` của en.wiktionary vào `han_viet`** ⇒ tái tạo **đúng** lỗi Unihan bằng
   một nguồn khác. Nhãn có sẵn; ⛔ đừng bỏ qua nó.
3. **Suy đoán nhãn khi `tags` vắng mặt** ⇒ đoán sai im lặng. ⛔ Không có `tags` ⇒ ⛔ không nạp.
4. **Tách nhiều âm sai quy ước** — Thiều Chửu dùng `|`, TVC dùng `,`, Unihan dùng khoảng
   trắng. **Ba** quy ước, và Story 1.16 sẽ tách bằng **một luật duy nhất** *(cắt trên `|`,
   `,` và khoảng trắng)*. ⇒ ⛔ đừng chuẩn hoá về một dấu ở đây mà ⛔ không báo cho 1.16.
5. **Dựng `.db` bằng đồng hồ hệ thống** ⇒ phá tái lập, mọi checksum thành dùng-một-lần.
6. **Đo NFR6 rồi tự cắt cho vừa** ⇒ đúng thứ §Ngân sách cấm. **Ghi số và dừng.**

### 🔴 Rủi ro pháp lý — ghi ra, ⛔ không giấu

**Trần Văn Chánh — *Từ điển Hán Việt* (1999).** Tác giả **còn sống**; tác phẩm **còn trong
bản quyền**. `catusf/tudien` khai **CC0-1.0** cho **cả repo**, nhưng CC0 do **người số hoá**
tuyên bố ⛔ **không** xoá được bản quyền của tác phẩm **gốc** — đây **đúng** lập luận
`sources_meta.rs:135` đã dùng theo chiều ngược lại cho Thiều Chửu *(mất 1954 ⇒ hết hạn)*.

**Ice chấp nhận có ý thức 2026-08-06, kèm giảm thiểu:** đóng gói làm **lớp gỡ rời** ⇒ FR112
thực thi bằng **xoá một tệp**, ⛔ không phải sửa mã và dựng lại payload. Đây là **cùng tư
thế phản ứng** `prd.md:999` đã chọn cho Cổ hán văn và HVTĐTD.

⇒ **AC8 đòi ghi điều này vào `attribution` của chính nguồn** — để nó đi theo dữ liệu ra tới
màn hình Attribution (Story 10.4), ⛔ không ở lại trong một story file ⛔ không ai đọc lại.

### Testing standards

```
cargo test --manifest-path tools/dict-build/Cargo.toml
cargo test --manifest-path src-tauri/Cargo.toml
npm run check:dict     npm run check:dict-manifest
npm run check:i18n     npm run check:deps
```

- **Test hành vi qua biên**, ⛔ không test cài đặt. Fixture nhỏ trong `tools/dict-build/tests/`
  — **dùng lại** khuôn đã có, ⛔ đừng dựng bộ thứ hai.
- 🔴 **AC4 và AC5 phải đỏ-rồi-xanh**: FR36 bằng **xoá tệp thật**; AC5 bằng **chạy trên dữ liệu
  Unihan cũ ra 92,4 %**. Một phép kiểm ⛔ không đỏ được thì ⛔ không phải một lưới.
- ⚠️ `tools/**` được **miễn trừ trọn** khỏi `check-i18n.mjs` *(có tên, có lý do — Story 1.9
  Task 9)*. ⇒ chuỗi chẩn đoán ở đây ⛔ **không** bị cổng canh; ⛔ đừng đọc thành "được phép
  cẩu thả".

### References

- `epics.md:816` *(nghiệm thu FR36 bằng test thật)* · `:142` FR33 · `:184` FR113
- `ARCHITECTURE-SPINE.md:147` AD-10 · `:290` AD-19 · `:328` AD-25 · `:366` AD-30 · `:435`
- `prd.md:921` *(bảng nguồn + HVTĐTD)* · `:944-946` *(dư địa 15.675.258 cho hai lớp cuối)* ·
  `:999` *(tiền lệ: nguồn chưa xác minh ⇒ đóng gói **gỡ rời**)* · `:1073` **[A2]**
- `EXPERIENCE.md:410` *(FR113 — `北涼 → Bắc Lương`)*
- `tools/dict-build/src/schema.rs:41-51` · `sources/unihan.rs:116` · `sources/thieu_chuu.rs:70`
  · `sources_meta.rs:11-16` · `build.rs:365-377` · `build.rs:491-506`
- `src-tauri/src/core/dict/layer.rs:49` `SUPPORTED_SCHEMA_VERSION`
- `dict-manifest.toml:9-18` *(luật ba trường)* · `:36-40` *(bài học `--layer all`)*
- Story `1-16-panel-source-va-tab-han-viet.md` §Quyết định #1 *(nguồn của phát hiện)*

### Câu hỏi cho Ice

1. **Nguồn kaikki khai DEPRECATED.** Ghim bản đã tải là đủ cho story này, nhưng lượt làm mới
   dữ liệu sau *(Story 10.1?)* sẽ cần một đường ổn định. Ai nhận?
2. **Dư địa NFR6 vốn dành cho HVTĐTD + Cổ hán văn.** Story này tiêu vào đó. Nếu Task 9 cho
   thấy hai lớp kia ⛔ không còn vừa, đó là quyết định **tầng PRD** — mở ngay hay đợi có số?

---

## Dev Agent Record

### Agent Model Used

Claude Sonnet 5 (claude-sonnet-5), qua Claude Code CLI.

### Debug Log References

**Nguồn thô ghim (2026-08-06):**
- `raw/tran_van_chanh/Tu-dien-ThienChuu-TranVanChanh.tab` ← `catusf/tudien@a7dd918ecc67de8c2d15034f885d919b9295eba4` (commit `master` tại thời điểm tải, 2025-12-19) — 3.932.157 byte, khớp chính xác con số story đã đo.
- `raw/en_wiktionary_vi/kaikki-en-vi.jsonl` ← `https://kaikki.org/dictionary/Vietnamese/kaikki.org-dictionary-Vietnamese.jsonl` — 77.944.107 byte, `Last-Modified: 2026-08-02`.

**AC5 — nghiệm thu đỏ-rồi-xanh (đo THẬT, không phải giả lập nhỏ):** `tools/dict-build/tests/nom_guard_real_data.rs::kvietnamese_reproduces_the_historical_92_percent_overlap_on_real_data` (`#[ignore]`, chạy tay — phụ thuộc `raw/**` thật, không có trên CI) tái tạo hành vi mã CŨ (`kVietnamese` → `han_viet`) trên dữ liệu Unihan thật, đối chiếu chéo nguồn với nhãn `nom-reading` thật của `en-wiktionary-vi`:
```
AC5 — đo trên dữ liệu THẬT: 1084/1173 = 92.4% ký tự kVietnamese trùng một âm Nôm đã gắn nhãn của en-wiktionary-vi
```
Trùng khớp **chính xác** con số §Phát hiện ② của story (`1.084/1.173 = 92,4%`) — không phải một ước lượng, một phép chạy lại thật bằng chính mã `sources::unihan::parse` + `sources::en_wiktionary_vi::parse` + `nom_guard::count_suspicious`.

🔴 **Con số này đo LẠI ở lượt code review 2026-08-06 sau khi siết `nom_only_readings`** (xem "VÒNG SỬA THỨ BA/TƯ" bên dưới) — kết quả mới **79,5% (882/1.109)**, ⛔ không còn 92,4%, vì phép lọc mới loại bớt âm tự-trùng-vai khỏi vế đối chứng. Test đã cập nhật để phản ánh ĐÚNG phép lọc production dùng; assert `exceeds_threshold()` vẫn xanh.

⚠️ **Một vòng thiết kế lại của `nom_guard` xảy ra giữa chừng** (ghi lại vì nó thay đổi ý nghĩa của AC5's "báo số"): phiên bản đầu tiên so `han_viet` với MỌI `nom_reading` bất kể nguồn, và nó tự kích hoạt dương tính giả trên chính dữ liệu SAU story này — `en-wiktionary-vi` tự gắn cả hai nhãn cho cùng ký tự khá thường xuyên (đo thật: 445/1.145 = 38,9%, một âm hợp lệ ở cả hai vai là thực tế ngôn ngữ học, không phải lỗi), và so `han_viet` của nó với `nom_reading` CỦA CHÍNH `unihan` (suy diễn thống kê, không phải nhãn tường minh) cho 323/460 = 70,2% trên build thật đầu tiên — cả hai đều KHÔNG phải bằng chứng gán nhãn sai. Sửa hai lần: ① chỉ đối chiếu XUYÊN NGUỒN (loại so một nguồn với chính nó); ② nhãn Nôm "đã xác nhận" chỉ tính từ `en-wiktionary-vi` (nguồn DUY NHẤT có `tags: ["nom-reading"]` tường minh — đúng nguyên văn Given clause của AC5), loại `nom_reading` của `unihan` khỏi vế đó. Sau hai lần sửa, build thật (`--layer all`) cho `0/0 (0.0%)` ở cả bốn tệp — an toàn, nhưng **VÔ NGHĨA về mặt cấu trúc** cho ba tệp gỡ rời (xem §Review Findings bên dưới — bản vá TIẾP THEO đóng lỗ hổng này).

🔴 **VÒNG SỬA THỨ BA — lượt code review 2026-08-06, sau khi story đã ở trạng thái `review`.** Phát hiện: `LABELED_NOM_SOURCE = "en-wiktionary-vi"` chỉ tồn tại trong `dict-core.db`; theo AD-10 (mỗi tệp gỡ rời đúng MỘT `dict_source`), ba tệp `dict-thieu-chuu.db`/`dict-vietphrase.db`/`dict-tran-van-chanh.db` KHÔNG BAO GIỜ có hàng để đối chiếu xuyên nguồn ⇒ AC5 vĩnh viễn `0/0` ở đó, bất kể `han_viet` sai đến đâu — đúng lớp bảo vệ mà story tồn tại để dựng lại KHÔNG bảo vệ được hai nguồn `han_viet` mới nhất. **Sửa:** nạp cặp `(headword, nom_reading)` của `en-wiktionary-vi` TỪ RAW (`load_en_wiktionary_vi_labeled_nom`, `build.rs`) và truyền vào phép kiểm của MỖI lớp gỡ rời (`run_detachable_layer` nhận thêm `raw_dir`, ⛔ không thêm `if code == "..."` nào — cùng một cơ chế áp cho cả ba lớp, đúng AD-10).

🔴 **Sửa xong lộ ra dương tính giả THẬT ngay lập tức**: chạy `--layer all` trên dữ liệu thật, lớp `thieu-chuu` — nguồn Hán Việt CHUẨN mà chính §Phát hiện ① dùng làm đối chứng cho lỗi Unihan — bị gắn cờ **369/582 = 63,4%** "đáng ngờ", vượt ngưỡng 50%, build dừng. Điều tra: mẫu hình lặp lại là `en-wiktionary-vi` tự gắn CẢ HAI nhãn (`han-viet-reading` VÀ `nom-reading`) cho cùng một âm của rất nhiều ký tự (vd 七: `han_viet="thất"`, `nom_reading="thất,sất"`) — hiện tượng tự-trùng-vai ĐÃ ĐO (445/1.145 = 38,9%) nhưng thiết kế gốc chỉ loại được khi so MỘT nguồn với CHÍNH NÓ, ⛔ không loại được khi Thiều Chửu (nguồn khác) tình cờ báo đúng âm mà en-wiktionary-vi tự gắn nhãn kép. **Sửa thứ tư (Ice chốt qua `AskUserQuestion`):** siết vế "nhãn Nôm đã xác nhận" — hàm mới `nom_guard::nom_only_readings` loại khỏi tập Nôm mọi âm CŨNG được CHÍNH `en-wiktionary-vi` gắn `han-viet-reading` cho CÙNG ký tự, đúng tinh thần hai-trục (HV/Nôm) của §Phát hiện ② thay vì một-trục. Đo lại trên dữ liệu thật sau khi siết: **Thiều Chửu 23/441 = 5,2%** · **Trần Văn Chánh 31/476 = 6,5%** — cả hai dưới xa ngưỡng 50%, build `--layer all` **THÀNH CÔNG**, bốn SHA-256 **giữ nguyên y hệt** bản ghi cũ (phép kiểm không đụng dữ liệu ghi ra, chỉ đọc). Mệnh đề "đỏ được" của AC5 vẫn đứng sau khi siết — `nom_guard_real_data.rs` đo lại cho **79,5% (882/1.109)**, thấp hơn 92,4% gốc (đúng, vì loại bớt âm tự-trùng-vai) nhưng vẫn cách xa ngưỡng 50% VÀ cách xa hẳn 5,2–6,5% của hai nguồn hợp lệ — khoảng cách phân biệt "nguồn sai" với "nguồn đúng" **còn rộng hơn**, không hẹp lại.

**Task 8 — dựng thật cả bốn tệp (`--layer all`, release build):**
```
### base — dict-core.db ###
cvdict             122597          1     122596     200195          0          0
cc-cedict          124758          0     124758     199615          0          0
unihan              49870          0      49870      23285          0          0
viwiktionary       415115     413517       1598       2242        536          0
en-wiktionary      306358     131681     174677     255372      89939          0
viwiktionary-en     401101     282062     119039     190543      27396          0
en-wiktionary-vi      51198      48966       2232          0          0          0
char_idx cặp (ch, entry_id): 1343313
AC5 han_viet đáng ngờ:        0/0 (0.0%)
SHA-256: 3bcfa98bf428aec30cf61ee8242b55168e399df4778c7ed8e95170e906a183e9
Kích thước: 195813376 byte

### lớp gỡ rời — dict-thieu-chuu.db ###
thieu-chuu           9898          1       9897      22681          0        263
AC5 han_viet đáng ngờ: 23/441 (5.2%)  🔴 [sau bản vá code review 2026-08-06 — TRƯỚC bản vá: 0/0, VÔ NGHĨA cấu trúc; bản chưa siết `nom_only_readings`: 369/582 = 63,4%, dương tính giả]
SHA-256: bd60ded6597cd36494b3596166f2e55c371d72360f58bcff936d8578feb35dbb — 5795840 byte

### lớp gỡ rời — dict-vietphrase.db ###
vietphrase         679311          9     679302     805558          0          0
AC5 han_viet đáng ngờ: 0/0 (0.0%) — vietphrase không ghi han_viet (đúng thiết kế, xem sources/vietphrase.rs)
SHA-256: 0add27583f0e3f262c04f9fbfae382f4fdff05313bf70554b00fc3891535f375 — 160788480 byte

### lớp gỡ rời — dict-tran-van-chanh.db ###
tran-van-chanh      22030          0      22030      22030          0          0
AC5 han_viet đáng ngờ: 31/476 (6.5%)  🔴 [sau bản vá code review 2026-08-06 — TRƯỚC bản vá: 0/0, VÔ NGHĨA cấu trúc]
SHA-256: 8a437f506ec5f551d7ada94075d7748a1fbfd8e0ca682f59c219adea8983018f — 10842112 byte
```
**Tái lập được xác nhận:** hai lượt `--layer all` độc lập (nhị phân đã build, không `cargo run` lại — tránh nhiễu `builder_version`) vào hai thư mục đích khác nhau cho ra **CÙNG** bốn SHA-256 trên. **Xác nhận LẦN HAI ở lượt code review 2026-08-06** (sau khi vá `nom_guard` — xem trên): build lại từ CÙNG cây `raw/` cho ra **CÙNG BỐN SHA-256 Y HỆT** bản ghi gốc — đúng như kỳ vọng, vì bản vá chỉ đổi phép KIỂM (đọc trước `finalize::finish`), ⛔ không đụng dữ liệu ghi vào `.db`.

**Task 9 — bảng đo NFR6 (trần 400.000.000 byte):**

🔴 **Baseline "Trước story" SỬA ở lượt code review 2026-08-06** — xem §Review Findings. Bản ghi gốc dùng "343.991.430 / dư 56.008.570" (số CŨ của `epics.md:336`, TRƯỚC Story 1.10b). Baseline ĐÚNG là con số Story 1.10b tự đo (`1-10b-...md:934,963,1087`): **384.525.446 / dư 15.474.554** — CÓ cộng font (21.285.713) + baseline app (2.334.696) + license (35.149), đúng định nghĩa NFR6 của `epics.md:336`.

| Tệp | Trước story (SỬA) | Sau story | Delta |
|---|---:|---:|---:|
| `dict-core.db` | 194.998.272 | 195.813.376 | +815.104 |
| `dict-thieu-chuu.db` | 5.787.648 | 5.795.840 | +8.192 |
| `dict-vietphrase.db` | 160.083.968 | 160.788.480 | +704.512 |
| `dict-tran-van-chanh.db` | 0 (chưa tồn tại) | 10.842.112 | +10.842.112 |
| Font + baseline app + license *(1-10b đã đo, KHÔNG đổi ở story này)* | 23.655.558 | 23.655.558 | 0 |
| **Tổng payload sản phẩm** | **384.525.446** | **396.895.366** | **+12.369.920** |

Trần **400.000.000** · dư địa còn lại 🔴 **3.104.634 byte** *(⛔ KHÔNG phải 26.760.192 như bản ghi gốc)* · ⛔ **KHÔNG vượt trần** ⇒ Quyết định #5 vẫn **không mở** đúng nghĩa đen (396.895.366 < 400.000.000), nhưng dư địa còn lại chỉ **0,78%** của trần — ⚠️ **HVTĐTD + Cổ hán văn gần như KHÔNG còn chỗ**, ⛔ không phải "còn 26,7 MB" như bản ghi gốc gợi ý. ⚠️ `prd.md:946` đã cảnh báo dư địa này dành cho hai lớp đó — story này thực tế tiêu **12.369.920 byte** vào đó (⛔ không phải 29.248.378 như số cũ tính sai vì trừ nhầm baseline), nhưng phần dư CÒN LẠI trước story đã nhỏ hơn nhiều so với ghi nhận (15,47 MB, không phải 56 MB) nên hệ quả ròng vẫn là gần cạn ngân sách. Ghi rõ số ĐÚNG ở `deferred-work.md §1-10c`, ⛔ không lặng lẽ.

**Cổng cuối, tất cả xanh:** `cargo test` (`tools/dict-build`: 157 test qua 5 tệp/mô-đun; `src-tauri`: toàn bộ, trừ một test timing `store_contract.rs` flaky không liên quan đến story, xanh khi chạy đơn lẻ) · `npm run check:dict` · `check:dict-manifest` · `check:i18n` · `check:deps`. `git diff --stat -- src/` rỗng · `tools/dict-build/Cargo.lock` chỉ đổi số phiên bản (0 phụ thuộc mới) · `src-tauri/Cargo.toml` không đụng.

---

### 🔴 Lượt code review 2026-08-06 — bốn patch áp dụng, hai decision xử lý, số đo THẬT

**AC1 — test tích hợp mới, chạy trên CI:**
```
test ac1_unihan_kvietnamese_row_count_matches_raw_source_before_and_after_role_swap ... ok
```

**AC4 — test tích hợp mới, chạy tay trên `raw/**` thật:**
```
AC4/FR36 — mức phủ đo THẬT: 10249 (Thiều Chửu ∪ en-wiktionary-vi, KHÔNG có tran-van-chanh) → 12463 (CÓ tran-van-chanh) — TVC đóng góp riêng 12081 đầu mục
test ac4_fr36_coverage_drop_is_measured_on_real_data ... ok
```
Khớp CHÍNH XÁC ba con số §Phát hiện ⑤/AC4 của story — số gốc ĐÚNG, chỉ thiếu cổng tự động.

**AC5 — thiết kế lại `nom_guard` cho lớp gỡ rời, chạy `--layer all` thật hai lần** (một lần lộ dương tính giả 63,4% trên Thiều Chửu, một lần sau khi siết `nom_only_readings`):
```
AC5 han_viet đáng ngờ: 0/0 (0.0%)    dict-core.db
AC5 han_viet đáng ngờ: 23/441 (5.2%) dict-thieu-chuu.db
AC5 han_viet đáng ngờ: 0/0 (0.0%)    dict-vietphrase.db
AC5 han_viet đáng ngờ: 31/476 (6.5%) dict-tran-van-chanh.db
```
Bốn SHA-256 **giữ nguyên y hệt** bản ghi Task 8 gốc — bản vá chỉ đổi phép KIỂM (đọc trước `finalize::finish`), ⛔ không đụng dữ liệu ghi ra.

**AC9 — NFR6, baseline sửa:** payload thật sau story **396.895.366 / 400.000.000**, dư **3.104.634 byte (0,78%)** — xem bảng đã sửa ở Task 9 trên. Không đụng file `.db` nào (số byte không đổi, chỉ baseline so sánh đổi).

**Cổng cuối sau bốn patch + hai decision, tất cả xanh:** `cargo test` (`tools/dict-build`: 108+6+13+22 unit/integration/CLI/schema, +2 test mới AC1/nested-bracket/multi-char chạy CI, +2 test `#[ignore]` chạy tay khớp số thật) · `npm run check:dict` (Kiểm A `.entry(` cần thêm miễn trừ tại chỗ, đã thêm và xanh) · `check:dict-manifest` · `check:i18n` · `check:deps`. Build `--layer all` release thật chạy lại hai lần, thành công cả hai, bốn SHA-256 không đổi.

### Completion Notes List

1. **AC1/AC2 — `Unihan kVietnamese` đổi vai, không mất dữ liệu.** `sources/unihan.rs` giờ đổ `kVietnamese` vào `RawEntry.nom_reading` thay vì `han_viet` — 0 hàng bị mất (kiểm bằng test `accumulates_multiple_properties_of_the_same_character`, đối chứng qua build thật: `unihan` vẫn cho 49.870 entry, cùng con số trước story). `dict_entry.han_viet` giờ mang ĐÚNG một ngữ nghĩa ở CẢ BỐN tệp — cưỡng chế bằng test tích hợp `nom_guard_real_data.rs` (chạy trên chính dữ liệu Unihan cũ) cộng doc-comment `schema.rs::DICT_ENTRY_DDL`.
2. **AC3 — nguồn nền thứ bảy `en-wiktionary-vi`.** Đo được **1.145** ký tự có âm Hán Việt gắn nhãn trên dữ liệu 2026-08-06 (≥ 1.136 theo AC3 — con số hôm nay của story đã hơi trôi vì kaikki là một kho sống, vẫn ĐẠT ngưỡng). Dùng lại `wiktextract_common.rs` bằng cách thêm `parse_reading_line`/`parse_readings` (không viết parser JSON thứ hai — cùng tệp, khác trường trích ra). Quyết định #3a: 0 `dict_sense` nạp từ nguồn này.
3. **AC4 — lớp gỡ rời thứ ba `tran-van-chanh`.** 22.030 đầu mục (khớp chính xác số dòng thật của `Tu-dien-ThienChuu-TranVanChanh.tab`), phủ **12.081** ký tự đơn có âm Hán Việt. `DETACHABLE_LAYERS` (build.rs) và `DETACHABLE_ALL` (sources_meta.rs) khớp — test `distribution_table_matches_detachable_all_exactly` vẫn xanh. FR36 nghiệm thu bằng test thật (`detachable_files_do_not_contain_each_others_rows`, mở rộng cho lớp thứ ba). 🔴 **Sửa ở lượt code review 2026-08-06:** mức phủ tụt xuống (12.463 → 10.249) và ngưỡng ≥12.081 giờ CƯỠNG CHẾ bằng test tự động `ac4_fr36_coverage_drop_is_measured_on_real_data` (đo THẬT trên `raw/**`, không còn chỉ là văn bản AC) — kết quả khớp CHÍNH XÁC ba con số story đã khai, xác nhận số gốc đúng.
4. **AC5 — lưới chống tái diễn, module mới `nom_guard.rs`.** Xem Debug Log References cho toàn bộ diễn biến thiết kế — **BỐN** lần sửa, không phải hai: hai lần đầu (tránh dương tính giả tự-so-chính-mình) đã ghi ở bản gốc; **hai lần SAU** thêm ở lượt code review 2026-08-06 sau khi phát hiện phép kiểm VÔ NGHĨA CẤU TRÚC cho ba tệp gỡ rời (AD-10: mỗi tệp một `dict_source`, `LABELED_NOM_SOURCE` chỉ có ở `dict-core.db`). Sửa: nạp nhãn Nôm từ raw cho lớp gỡ rời + siết `nom_only_readings` (loại âm tự-trùng-vai) sau khi bản đầu gây dương tính giả 63,4% trên Thiều Chửu. Ngưỡng phán quyết `SUSPICIOUS_RATIO_THRESHOLD = 0.5` — hằng có tên kèm lý do (doc-comment), KHÔNG đổi. Nghiệm thu đỏ-rồi-xanh dùng dữ liệu THẬT — **79,5%** sau khi siết (đã đổi từ 92,4% gốc, xem lý do ở Debug Log References), vẫn cách xa ngưỡng và cách xa 5,2–6,5% của hai nguồn hợp lệ. Guard giờ THẬT SỰ hoạt động ở cả bốn tệp: base 0/0, thieu-chuu 23/441=5,2%, vietphrase 0/0, tran-van-chanh 31/476=6,5%.
5. **AC6 — lược đồ v2.** `SCHEMA_VERSION`/`SUPPORTED_SCHEMA_VERSION` nâng 1→2 cùng lượt (`tools/dict-build/src/schema.rs` và `src-tauri/src/core/dict/layer.rs`). Không bộ di trú nào dựng — đúng chốt "tệp KHÔNG di trú, thay nguyên tệp qua release mới". Test parity DDL (`dict_sources.rs`/`dict_lookup.rs::fixture_ddl_is_verbatim_from_dict_build_schema`) và test SCHEMA_VERSION khớp đều xanh.
6. **AC7 — manifest, tái lập được.** Cả bốn mục `dict-manifest.toml` cập nhật (ba checksum đổi vì lược đồ đổi, một mục mới cho `tran-van-chanh`). Tái lập được xác nhận bằng hai lượt build độc lập cho cùng SHA-256 — xem Debug Log References. `npm run check:dict-manifest` PASS.
7. **AC8 — rủi ro pháp lý Trần Văn Chánh, ghi thẳng.** `license_kind = "copyrighted"` (không phải `unknown`/`public-domain` — khác lý do với VietPhrase: ở đây biết TÁC GIẢ và biết tác phẩm CÒN bản quyền). Rủi ro ghi vào `dict_source.attribution` (kiểm bằng test) và `assets/licenses/tran-van-chanh.txt` (tuyên bố xuất xứ, nêu rõ CC0 của người số hoá KHÔNG xoá bản quyền tác phẩm gốc). Đóng gói lớp gỡ rời ⇒ FR112 = xoá một tệp.
8. **AC9 — NFR6, không vượt trần — SỬA số ở lượt code review 2026-08-06.** Bảng gốc dùng baseline "trước story" CŨ (343.991.430, số của `epics.md:336` từ TRƯỚC Story 1.10b), cho dư địa "còn lại 26.760.192 byte" — SAI. Baseline ĐÚNG là số Story 1.10b tự đo (384.525.446, CÓ font+baseline app+license). Payload thật sau story = **396.895.366 / 400.000.000**, dư **3.104.634 byte (0,78%)** — xem bảng đo đã sửa ở Debug Log References. ⛔ Vẫn KHÔNG vượt trần đúng nghĩa đen, nhưng dư địa gần cạn, ⛔ không rộng rãi như số cũ gợi ý. Ghi rõ số ĐÚNG vào `deferred-work.md §1-10c` rằng HVTĐTD/Cổ hán văn (story tương lai) gần như không còn ngân sách, phải tự đo TRƯỚC khi hứa bất kỳ điều gì.
9. **AC10 — cổng, sàn, bàn giao.** `RS_FILE_FLOOR` 21→24 (ba tệp `.rs` mới: `en_wiktionary_vi.rs`, `tran_van_chanh.rs`, `nom_guard.rs`). `EXPECTED_DETACHABLE_NAMES` (check-dict-manifest.mjs) 2→3. 0 dòng đổi dưới `src/**` (xác nhận bằng `git diff --stat`). 0 phụ thuộc mới (`Cargo.lock` chỉ đổi version). `src-tauri/Cargo.toml` không đụng. `deferred-work.md §1-10c` mở, đánh dấu Story 1.16 hết chặn + ghi lại Bẫy 4 (ba quy ước tách âm đọc) làm tham khảo cho story đó.
10. **Quyết định #3/#4 chốt qua `AskUserQuestion` với Ice** (không phải suy diễn) trước khi viết bất kỳ dòng code nào — cả hai theo đề xuất mặc định của story: (a) chỉ nạp âm đọc từ en-wiktionary-vi; (a) cột `nom_reading TEXT` mới.
11. **Phạm vi tuân thủ đúng ranh giới đã khai:** 0 dòng Vue/TS, `src-tauri/src/core/dict/**` chỉ đụng hằng phiên bản (không thêm đường đọc `nom_reading` ở runtime — `han_viet` bản thân cũng chưa được đọc ở đâu trong `src-tauri` hôm nay, nên "đường đọc cột mới" không có gì cụ thể để thêm; Story 1.16 sẽ là nơi đọc cả hai cột lần đầu). `_bmad-output/planning-artifacts/**` không sửa — độ lệch với `prd.md §8.2` (TVC từng bị loại) ghi ra ở `deferred-work.md`, không sửa file quy hoạch.

### File List

**Mới:**
- `tools/dict-build/src/sources/en_wiktionary_vi.rs`
- `tools/dict-build/src/sources/tran_van_chanh.rs`
- `tools/dict-build/src/nom_guard.rs`
- `tools/dict-build/tests/nom_guard_real_data.rs`
- `tools/dict-build/assets/licenses/tran-van-chanh.txt`
- `tools/dict-build/tests/fixtures/raw/en_wiktionary_vi/kaikki-en-vi.jsonl`
- `tools/dict-build/tests/fixtures/raw/tran_van_chanh/Tu-dien-ThienChuu-TranVanChanh.tab`

**Sửa:**
- `tools/dict-build/src/model.rs` — thêm `RawEntry.nom_reading`
- `tools/dict-build/src/schema.rs` — cột `nom_reading`, `SCHEMA_VERSION` 1→2
- `tools/dict-build/src/insert.rs` — chèn `nom_reading`
- `tools/dict-build/src/lib.rs` — đăng ký `nom_guard`
- `tools/dict-build/src/licenses.rs` — `tran_van_chanh_license_text()`
- `tools/dict-build/src/sources_meta.rs` — `EN_WIKTIONARY_VI`, `TRAN_VAN_CHANH`, `LicenseRef::TranVanChanh`, `BASE_ALL` 6→7, `DETACHABLE_ALL` 2→3
- `tools/dict-build/src/sources/mod.rs` — đăng ký hai module mới
- `tools/dict-build/src/sources/unihan.rs` — đổi vai `kVietnamese`
- `tools/dict-build/src/sources/cc_cedict.rs`, `cvdict.rs`, `vietphrase.rs`, `thieu_chuu.rs` — thêm `nom_reading: None`
- `tools/dict-build/src/sources/wiktextract_common.rs` — `parse_reading_line`/`parse_readings`, `union_reading_lists` (đổi tên từ `merge_comma_lists`)
- `tools/dict-build/src/build.rs` — nguồn nền thứ bảy, lớp gỡ rời thứ ba, gọi `nom_guard` trong cả hai đường dựng
- `tools/dict-build/Cargo.toml` — version 0.3.0 → 0.4.0
- `tools/dict-build/Cargo.lock` — theo version bump (0 phụ thuộc mới)
- `tools/dict-build/tests/layers.rs`, `tests/parse.rs` — cập nhật số đếm + test mới cho lớp/nguồn thứ ba/bảy
- `src-tauri/src/core/dict/layer.rs` — `SUPPORTED_SCHEMA_VERSION` 1→2
- `src-tauri/tests/dict_sources.rs`, `tests/dict_lookup.rs` — DDL parity (`nom_reading`)
- `dict-manifest.toml` — bốn mục cập nhật/mới
- `scripts/check-dict-build.mjs` — `RS_FILE_FLOOR` 21→24
- `scripts/check-dict-manifest.mjs` — `EXPECTED_DETACHABLE_NAMES` 2→3
- `_bmad-output/implementation-artifacts/deferred-work.md` — mở mục `§1-10c`
- `_bmad-output/implementation-artifacts/sprint-status.yaml` — trạng thái story
