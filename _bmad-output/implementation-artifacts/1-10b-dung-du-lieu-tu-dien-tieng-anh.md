---
baseline_commit: ed8ce52
baseline_note: 'Cây làm việc tại ed8ce52. ⚠️ Story 1.9 + 1.10 (cả hai `done`) CHƯA commit hết — chạy `git status` trước khi tin bất kỳ con số nào ở §Trạng thái repo hiện tại'
---

# Story 1.10b: Dựng dữ liệu từ điển tiếng Anh

Status: done

<!-- Note: Validation is optional. Run validate-create-story for quality check before dev-story. -->

**Covers:** FR34 · NFR6 *(đo lại)* · NFR8 · AD-19 · AD-25
**Epic:** 1 — Nền móng ứng dụng & Tra cứu ngoại tuyến tức thì
**Story trước:** [1.10 — Đóng gói bốn lớp gỡ rời](1-10-dong-goi-bon-lop-go-roi-thanh-file-doc-lap.md) *(done)*
**Nguồn gốc:** [`sprint-change-proposal-2026-08-05.md`](../planning-artifacts/sprint-change-proposal-2026-08-05.md) — ✅ Ice duyệt 2026-08-05

---

> 🔴 **Story này sửa một lỗ hổng đã ĐO ĐƯỢC, không phải một tính năng mới.**
>
> `dict-core.db` đã dựng có **473.499 đầu mục, 100% `lang='zh'`, 0 mục tiếng Anh**. **FR34** — *"Mục từ **tiếng Anh** phải có nhãn từ loại và nghĩa tiếng Việt"* — hiện **không có một byte dữ liệu nào** để đứng lên. Mục tiêu Epic 1 viết thẳng *"văn bản tiếng Trung **hoặc tiếng Anh**"*; một nửa mệnh đề đó chưa nghiệm thu được.
>
> **Nguyên nhân gốc là mơ hồ của PRD, không phải lỗi cài đặt của Story 1.9.** `raw/viwiktionary/vi-extract.jsonl` là bản trích **toàn ấn bản** `vi.wiktionary.org` và mang **HAI VAI** tuỳ theo bộ lọc `lang_code`. PRD §8.2 giao vai tiếng Anh, §8.3 *(tiêu đề "Lớp từ loại tiếng **Trung**")* bàn cùng tệp đó ở vai tiếng Trung, và **không chỗ nào nói đó là hai vai song song**. Story 1.9 đọc §8.3 và cài đúng một vai. ✅ **PRD đã sửa hết mơ hồ 2026-08-05** — phần còn lại thuần là **thi công**.
>
> 🟢 **Nguồn thô ĐÃ CÓ SẴN** ở `tools/dict-build/raw/viwiktionary/vi-extract.jsonl` *(273.075.442 byte, 415.254 dòng)* — **không phải tải gì thêm, không phải xin phép ai**. Cùng tệp, cùng giấy phép, cùng parser đã tham số hoá sẵn.
>
> ⚠️ **Story này KHÔNG viết một dòng mã tra cứu nào và KHÔNG chạm `src-tauri/`.** Đường tra cứu tiếng Anh là **Story 1.11b** — và nó đang **🔴 BỊ CHẶN** chờ một AD mới của Winston *(`AD-26` chỉ áp cho tiếng Trung)*. Story này giao **dữ liệu**, không giao đường đọc.

---

## Story

As a chủ dự án,
I want đầu mục tiếng Anh có mặt trong dữ liệu từ điển đóng gói,
So that cặp Anh → Việt có nền dữ liệu như cặp Trung → Việt đã có.

---

## Acceptance Criteria

### AC1 — `viwiktionary` vai A dựng thành nguồn thứ SÁU, dùng LẠI parser đã có

**Given** `raw/viwiktionary/vi-extract.jsonl` đã có
**When** dựng dữ liệu từ điển
**Then** vai A dựng qua `wiktextract_common::parse(reader, "vi", Some("en"))`
**And** **không parser mới**, **không crate mới**, **không đổi một dòng DDL nào của `schema.rs`**

*Đạt nghĩa là* sau một lượt chạy, `dict-core.db` có **đúng SÁU** hàng `dict_source`:

| # | `code` | Vai |
|---|---|---|
| 1 | `cvdict` | ZH |
| 2 | `cc-cedict` | ZH |
| 3 | `unihan` | ZH |
| 4 | `viwiktionary` | ZH — **vai B, không đổi một chữ** |
| 5 | `en-wiktionary` | ZH |
| 6 | **`viwiktionary-en`** | **EN — vai A, MỚI ở story này** |

**Không** đổi tên `viwiktionary` thành `viwiktionary-zh` "cho đối xứng" — xem §Bẫy 6.

### AC2 — Đối chiếu số đo THẬT; lệch quá 1% ⇒ parser sai

**Given** lượt build hoàn tất
**When** đọc bảng `SourceStats` của nguồn `viwiktionary-en`
**Then** ba con số khớp mũi thăm dò 2026-08-05 trong **1%**:

| Chỉ số | Giá trị mũi thăm dò | Ngưỡng 1% |
|---|---:|---|
| `entries` | **119.039** | 117.849 – 120.229 |
| `senses` | **190.543** | 188.638 – 192.448 |
| `examples` | **27.396** | 27.122 – 27.670 |

🔴 **Lệch quá 1% ⇒ PARSER SAI, không phải *"nguồn vốn thế"*.** Mũi thăm dò chạy trên **chính tệp này**, cùng hàm `parse`, cùng ngày. Không có nguồn ngẫu nhiên nào giữa hai lượt chạy.

⚠️ **`stats.lines_read` KHÔNG phải số dòng của tệp** — xem §Bẫy 3. Không đối chiếu nó với `wc -l`.

### AC3 — Mọi mục của nguồn này mang `lang = 'en'` — kèm ĐỐI CHỨNG ÂM bắt buộc

**Given** nguồn `viwiktionary-en`
**When** đọc `dict_entry`
**Then** **100%** hàng có `lang = 'en'`
**And** 🔴 **đối chứng âm bắt buộc:** nguồn này sinh **0** hàng `lang = 'zh'`
**And** 🔴 **đối chứng âm thứ hai:** nguồn `viwiktionary` *(vai B)* vẫn sinh **0** hàng `lang = 'en'` và số đầu mục của nó **không đổi** *(1.598)*

*SQL nghiệm thu — ghi nguyên văn vào §Debug Log References:*

```sql
SELECT s.code, e.lang, COUNT(*)
FROM dict_entry e JOIN dict_source s ON s.id = e.source_id
GROUP BY s.code, e.lang ORDER BY s.code, e.lang;
```

Kết quả phải cho `viwiktionary-en | en | ~119039` và **không có hàng nào** `viwiktionary-en | zh`.

🔴 **Đây là AC dễ trượt IM LẶNG nhất của story** — `wiktextract_common::parse_line` hôm nay **viết cứng `lang: "zh".to_string()`**. Không sửa chỗ đó ⇒ 119.039 đầu mục tiếng Anh vào database **mang nhãn tiếng Trung**, build XANH, mọi test khác XANH, và đường tra cứu tiếng Trung của Story 1.11 nhận về `"Wiktionary"`/`"API"` khi tra chữ Hán. Xem §Bẫy 1.

### AC4 — `dict_source` mang đủ bốn trường giấy phép, ghi công nêu đúng vai

**Given** hàng `dict_source` của `viwiktionary-en`
**When** đọc
**Then** cả bốn trường khác rỗng: `license_kind` · `license_text` · `attribution` · `source_url`
**And** `license_text` là **văn bản thật** *(CC-BY-SA 4.0 + GFDL 1.3, dùng lại `LicenseRef::CcBySaAndGfdl` đã có)*, không phải chuỗi giữ chỗ
**And** `attribution` nêu **Wiktionary tiếng Việt** + **kaikki.org** + **CC-BY-SA 4.0 và GFDL**
**And** `attribution` nêu rõ đây là **mục tiếng Anh**, phân biệt được với vai B trên màn hình Attribution *(Story 10.4)*

*Đạt nghĩa là* hai hàng đọc ra khác nhau bằng mắt, không phải hai bản sao cùng chuỗi:

| `code` | `display_name` |
|---|---|
| `viwiktionary` | Wiktionary tiếng Việt |
| `viwiktionary-en` | **Wiktionary tiếng Việt (mục tiếng Anh)** |

### AC5 — 🔴 Quyết định đóng gói: lớp này vào `dict-core.db` — ĐÃ CHỐT, không phải lựa chọn của dev

**Given** câu hỏi *"vào `dict-core.db` hay thành tệp `.db` riêng?"* mà `epics.md` giao cho story này
**When** chốt
**Then** lớp tiếng Anh là **nguồn NỀN thứ sáu trong `dict-core.db`**, **không** phải một tệp `.db` gỡ rời

**Bốn lý do, không phải sở thích** — xem §Quyết định #1 để đọc đầy đủ:

1. **PRD §8.2** xếp cả hai vai của viwiktionary vào hàng **Nền**.
2. **Giấy phép sạch y hệt vai B** *(CC-BY-SA + GFDL)* — không có rủi ro pháp lý nào để FR112 phải gỡ.
3. **AD-10 liệt kê ĐÍCH DANH bốn lớp gỡ rời** *(Thiều Chửu · Cổ hán văn · VietPhrase · HVTĐTD)*. Lớp tiếng Anh không nằm trong danh sách đó.
4. **FR34 là phạm vi LÕI.** Làm nó gỡ rời nghĩa là xoá một tệp làm **biến mất cả cặp ngôn ngữ Anh → Việt** — trái thẳng mục tiêu Epic 1.

⚠️ **Hệ quả bắt buộc, không được bỏ:** phải **dựng lại `dict-core.db`** và điền lại `[base].sha256` + `[base].source_version` trong `dict-manifest.toml`. 🟢 Nay rẻ và **tái lập được** sau bản vá `built_at` của Story 1.10 — cùng cây `raw/` cho ra cùng tệp byte-for-byte.

🔴 **Dựng bằng `--layer base`, KHÔNG `--layer all`** — xem §Bẫy 5.

### AC6 — Bảng kế toán NFR6 cập nhật với số THẬT, đối chiếu trần 400.000.000 byte

**Given** `dict-core.db` mới đã dựng thật
**When** cộng vào bảng kế toán của Story 1.10
**Then** phán quyết **ĐẠT** hoặc **VƯỢT** trần **400.000.000 byte**, quy về **byte** trước rồi mới đổi MB thập phân

*Đạt nghĩa là* một bảng trong §Debug Log References, mỗi dòng một số byte **đo được**:

| Dòng | Nguồn số |
|---|---:|
| Baseline `.dmg` không font/license | **2.334.696** — Story 1.9/1.10, tái dùng *(điều kiện: §Quyết định #8)* |
| License trong bundle | **35.149** — tái dùng |
| Bộ font | **21.285.713** — `font-spike-results-2026-08-03.md:82` |
| `dict-core.db` **MỚI** | 🔴 **đo thật ở story này** *(cũ: 154.464.256)* |
| `dict-thieu-chuu.db` | **5.787.648** — Story 1.10, không dựng lại |
| `dict-vietphrase.db` | **160.083.968** — Story 1.10, không dựng lại |
| Hai lớp chưa dựng *(HVTĐTD, Cổ hán văn)* | `[----] chưa đo — story nối tiếp` — không ước, không bỏ dòng |
| WebView2 Runtime nhúng | **dòng riêng**, không cộng vào tổng *(NFR6 sửa 2026-08-03)* |
| **Tổng payload sản phẩm hôm nay** | cộng bằng byte |
| Đối chiếu trần **400.000.000** byte | **ĐẠT** / **VƯỢT** |
| **Dư địa còn lại** | trần − tổng, bằng byte |

**Dự phóng để đối chiếu** *(PRD §8.2, `deferred-work.md:272`)*: **384.324.742 byte** — ĐẠT, dư **15.675.258**.

⚠️ **Trần là 400.000.000, không còn là 200.000.000.** Ice nâng trần 2026-08-05 trên số đo thật của Story 1.10 *(`prd.md:834` "NFR6 sửa lần hai")*. Một lượt rà đọc số 200.000.000 ở story 1.10 rồi kết luận "VƯỢT" là đọc bản ghi lỗi thời.

🔴 **Nếu VƯỢT** ⇒ **DỪNG và báo**. Không tự bỏ nguồn, không tự bỏ `sense_fts_nd`, không đụng `[profile.release]` hay `Cargo.toml` *(Ice chốt lần thứ năm)*, không subset font. Quyết định **tầng PRD**.

### AC7 — Hai cổng `.mjs` đi theo nguồn mới

**Given** `check-dict-build.mjs` và `check-dict-manifest.mjs`
**When** chạy `npm run check:dict` và `npm run check:dict-manifest`
**Then** cả hai **XANH** sau story
**And** `RS_FILE_FLOOR` cập nhật khớp số tệp `.rs` thật *(20 → 21 nếu thêm module)*
**And** `[base].source_version` trong manifest liệt kê **đủ sáu** nguồn nền
**And** `[[detachable]]` vẫn **ĐÚNG HAI** mục, không đổi *(`EXPECTED_DETACHABLE_NAMES`)*

⚠️ Kiểm D/E/F chỉ soi **lớp gỡ rời** — nguồn nền thứ sáu không rơi vào phạm vi của chúng. **Kiểm A** *(từ vựng hợp nhất)* thì CÓ: module mới nằm dưới `src/` nên tự động vào phạm vi quét.

### AC8 — NFR8 giữ nguyên hiệu lực trên dữ liệu tiếng Anh

**Given** nghĩa tiếng Việt của một mục từ tiếng Anh *(có dấu)*
**When** tìm qua `sense_fts` *(chỉ mục CHÍNH, `remove_diacritics 0`)*
**Then** **phân biệt dấu** — `má` không khớp `ma`
**And** `sense_fts_nd` *(chỉ mục PHỤ, `remove_diacritics 2`)* vẫn xoá dấu như thiết kế
**And** **không** bỏ `sense_fts_nd` để tiết kiệm dung lượng — phá AC4 của Story 1.10 *(lược đồ đồng nhất giữa các tệp)*

*Khuôn có sẵn:* `tests/parse.rs::ac5_primary_fts_is_diacritic_sensitive_secondary_is_not` — **đọc nó trước khi viết mới**, chỉ đổi sang một mục từ tiếng Anh.

---

## Tasks / Subtasks

- [x] **Task 1 — Đường cơ sở: chạy sáu lệnh, ghi số vào §Debug Log References** (không AC)
  - [x] `git status` — xác nhận cây làm việc trước khi tin số nào ở §Trạng thái repo hiện tại
  - [x] `npm run build` *(bắt buộc trước `cargo test` của `src-tauri` — `generate_context!` nhúng `dist/` lúc biên dịch)*
  - [x] `cargo test --locked --manifest-path src-tauri/Cargo.toml` · `cargo test --manifest-path tools/dict-build/Cargo.toml`
  - [x] `npm run check:deps` · `check:dict` · `check:dict-manifest` · `check:i18n`
  - [x] Ghi lại: số tệp `.rs` dưới `tools/dict-build/src/**` *(hôm nay **20**)* · tổng test hai cây *(`tools/dict-build` **88** · `src-tauri` **62**)* · số crate `check-deps.mjs` đếm được *(phải **không đổi** sau story)*
  - [x] Không sửa gì ở task này. Một lệnh đỏ sẵn thì **DỪNG và báo**.

- [x] **Task 2 — Tham số hoá `lang` trong `wiktextract_common`** (AC1, AC3) 🔴 **hạt nhân của story**
  - [x] `parse_line(...)`: thêm tham số `entry_lang: &str`, thay `lang: "zh".to_string()` *(`wiktextract_common.rs:143`)* bằng `lang: entry_lang.to_string()`.
  - [x] `parse(...)`: thêm cùng tham số, truyền xuống `parse_line`.
  - [x] Cập nhật **cả hai** caller đã có — `viwiktionary::parse` và `en_wiktionary::parse` — truyền `"zh"`. **Hành vi của chúng phải KHÔNG đổi một chút nào.**
  - [x] 🔴 Đặt `entry_lang` là **tham số bắt buộc**, không mặc định `"zh"`: mặc định biến đúng lỗi này thành thứ lặp lại được ở nguồn thứ bảy.
  - [x] Cập nhật doc-comment module: nói rõ **`filter_lang_code` và `entry_lang` là HAI thứ khác nhau** — cái đầu là *"đọc dòng nào"*, cái sau là *"ghi nhãn gì"*, và `en_wiktionary` là bằng chứng sống rằng chúng không suy ra được từ nhau *(`filter="zh"`, `pos_lang="en"`)*.
  - [x] Cập nhật mọi test `#[cfg(test)]` trong `wiktextract_common.rs` theo chữ ký mới. **Thêm** một test khẳng định `entry_lang` đi thẳng vào `RawEntry.lang`, cho **cả hai** giá trị `"zh"` và `"en"`.

- [x] **Task 3 — Module nguồn vai A** (AC1)
  - [x] `tools/dict-build/src/sources/viwiktionary_en.rs` — **wrapper 3 dòng** đúng khuôn `en_wiktionary.rs`:
        `pub const SOURCE_CODE: &str = "viwiktionary-en";` + `parse(reader) → wiktextract_common::parse(reader, "vi", Some("en"), "en")`.
  - [x] 🔴 `pos_lang = "vi"` *(tham số thứ 2)* — ấn bản `vi` có `pos_title` **đã sẵn tiếng Việt** *(đã kiểm thật trên fixture: `"Danh từ"`, `"Danh từ riêng"`)*. Không phải `"en"`.
  - [x] `mod.rs`: thêm `pub mod viwiktionary_en;`.
  - [x] Doc-comment nêu **đích danh** hai vai trên **cùng một tệp thô**, và trỏ sang `viwiktionary.rs` *(vai B)* — đây là chỗ duy nhất một người đọc mã tương lai hiểu được vì sao có hai module đọc một tệp.
  - [x] Test module-level: một dòng `lang_code:"en"` **cho ra** entry với `lang == "en"`; một dòng `lang_code:"zh"` **bị lọc** thành `ParseIssue`.
  - [x] **Không** thêm crate. **Không** đổi `Cargo.toml` của `tools/dict-build` *(ngoài `version`, xem Task 6)*.

- [x] **Task 4 — `SourceMeta` thứ sáu** (AC1, AC4)
  - [x] `sources_meta.rs`: `pub const VIWIKTIONARY_EN: SourceMeta` — `code: "viwiktionary-en"`, `display_name: "Wiktionary tiếng Việt (mục tiếng Anh)"`, `license_kind: "open"`, `license_id: Some("CC-BY-SA-4.0")`, `license_ref: LicenseRef::CcBySaAndGfdl` *(**không** thêm biến thể `enum` mới — giấy phép **y hệt** vai B)*.
  - [x] `attribution` nêu: Wiktionary tiếng Việt (`vi.wiktionary.org`) · **mục tiếng Anh** · qua Wiktextract/kaikki.org · CC BY-SA 4.0 và GFDL.
  - [x] `BASE_ALL: [&SourceMeta; 5]` → `[&SourceMeta; 6]`, thêm `&VIWIKTIONARY_EN` vào **cuối** *(thứ tự chèn = thứ tự `dict_source.id`; thêm vào cuối giữ id của năm nguồn cũ không đổi)*.
  - [x] 🔴 **Đổi `exactly_five_sources_with_the_epics_md_codes` → `exactly_six_sources_with_the_epics_md_codes`**: `assert_eq!(BASE_ALL.len(), 6)` + danh sách sáu mã. **Giữ nguyên doc-comment nói lớp gỡ rời KHÔNG thuộc `BASE_ALL`** — đó mới là thứ test này khoá *(§Bẫy 7)*.
  - [x] `base_and_detachable_code_sets_are_disjoint` và `every_source_declares_a_non_empty_license_text` chạy tự động qua danh sách mới — không sửa.
  - [x] Thêm `viwiktionary_and_viwiktionary_en_are_two_distinct_sources` — khẳng định hai `code` khác nhau **và** hai `display_name` khác nhau *(AC4: phân biệt được trên màn Attribution)*.

- [x] **Task 5 — Khối chèn thứ sáu trong `run_base`** (AC1, AC2, AC3)
  - [x] `build.rs::run_base`: thêm một khối `// ── viwiktionary (vai A — mục tiếng Anh) ──` **sau** khối `en_wiktionary`, đúng khuôn năm khối đã có.
  - [x] Đọc **cùng tệp** `raw_dir.join("viwiktionary").join("vi-extract.jsonl")` — mở **`File::open` lần thứ hai**, **không** tái dùng reader của vai B, **không** gộp hai vai vào một lượt đọc *(§Bẫy 2)*.
  - [x] `source_version` = `version_or_warn(sources::viwiktionary_en::SOURCE_CODE, file_mtime_date(&path))` — cùng hàm, cùng tệp ⇒ **cùng giá trị** với vai B. Đó là **đúng**: cùng một dump.
  - [x] `require_nonempty(&stats)?` như năm nguồn kia.
  - [x] **Không** đụng `run_detachable_layer` / `DETACHABLE_LAYERS` / `run_all` / `output_file_name` — nguồn nền không đi qua bảng phân phối lớp gỡ rời.
  - [x] **Không** đụng `char_idx.rs`. Đầu mục tiếng Anh sinh ~**9** cặp `char_idx` trên 119.039 mục — **đó là ĐÚNG**, không phải bug *(§Quyết định #6)*.

- [x] **Task 6 — Bump `builder_version`** (AC5)
  - [x] `tools/dict-build/Cargo.toml`: `version = "0.2.0"` → **`"0.3.0"`**. Nó vào `dict_meta('builder_version')` và là cách **duy nhất** phân biệt tệp dựng bởi CLI có/không có nguồn thứ sáu.
  - [x] Không đổi gì khác trong `Cargo.toml` — không thêm dependency, không đụng `[workspace]` *(Kiểm B)*.

- [x] **Task 7 — Fixture: bổ sung mục tiếng Anh THẬT** (AC1, AC2, AC8)
  - [x] 🟢 **Đã kiểm:** `tests/fixtures/raw/viwiktionary/vi-extract.jsonl` hôm nay đã có **3 mục top-level `lang_code:"en"`** dùng được *(`Wiktionary` · `Wikipedia` · `API`)*, cộng 4 mục `zh`. ⇒ `require_nonempty` **sẽ không hỏng**. Không xoá dòng nào đang có.
  - [x] 🔴 **Bổ sung, trích NGUYÊN VĂN từ `raw/viwiktionary/vi-extract.jsonl` thật:**
    - [x] ≥ 1 mục tiếng Anh **CÓ `examples`** — 27.396 ví dụ là một con số AC2 đối chiếu; fixture hôm nay có **0** ví dụ tiếng Anh, tức đường ví dụ **không được test nào chạm tới**.
    - [x] ≥ 1 **headword tiếng Anh xuất hiện trên HAI dòng JSONL** *(hai từ loại)* — chứng minh phép gộp theo headword chạy đúng cho **cả vai A**, đúng cách `馬` chứng minh cho `en_wiktionary`.
    - [x] ≥ 1 mục có nghĩa tiếng Việt **CÓ DẤU** phục vụ AC8.
  - [x] 🔴 **KHÔNG BỊA MỘT GIÁ TRỊ NÀO.** Lượt review Story 1.9 đối chiếu fixture byte-for-byte với nguồn thật và **đã bắt được** một fixture bịa 20/20 dòng. Cắt bớt trường không dùng cho gọn là **được**; đổi một giá trị là **không**.
  - [x] Ghi vào §Debug Log References **lệnh trích** đã dùng, để lượt rà sau tái lập được.

- [x] **Task 8 — Test** (AC1, AC2, AC3, AC4, AC8)
  - [x] `tests/parse.rs`:
    - [x] 🔴 Đổi `all_five_sources_produce_at_least_one_entry` → **`all_six_sources_produce_at_least_one_entry`**, thêm `viwiktionary-en`.
    - [x] 🔴 **`viwiktionary_en_entries_are_all_tagged_lang_en`** — **AC3**, và đối chứng âm `COUNT(*) WHERE lang='zh' AND source=viwiktionary-en` phải bằng **0**.
    - [x] 🔴 **`viwiktionary_role_b_still_produces_zero_english_rows`** — **AC3** đối chứng âm chiều ngược, chống hồi quy cho vai B.
    - [x] **`viwiktionary_and_viwiktionary_en_read_the_same_file_into_two_separate_sources`** — **AD-19**: hai `source_id` khác nhau, không hàng nào mang cả hai.
    - [x] **`an_english_entry_carries_pos_label_and_vietnamese_gloss`** — **FR34** nghiệm thu bằng **test thật**, không bằng suy luận *(tiêu chí thành công #2 của sprint change proposal)*: `dict_sense.pos` khác NULL, `pos_lang = 'vi'`, `gloss` khác rỗng.
    - [x] **`english_headword_on_two_lines_becomes_one_entry`** — phép gộp headword cho vai A.
    - [x] **`primary_fts_is_diacritic_sensitive_on_an_english_entry_gloss`** — **AC8**, khuôn từ `ac5_primary_fts_is_diacritic_sensitive_secondary_is_not`.
    - [x] `all_sources_have_a_real_non_unknown_source_version` — 🔴 **story dự đoán SAI**: nó có `assert_eq!(rows.len(), 5)` **viết cứng**, không "chạy tự động". Đã sửa **đúng con số** 5 → 6; mệnh đề nó khoá *(không nguồn nào rơi về `unknown`)* không đổi.
  - [x] `tests/layers.rs`: `sqlite_master_is_byte_identical_across_all_outputs` **phải vẫn xanh** — nguồn thứ sáu không đổi một dòng DDL nào. Không sửa test này; nếu nó đỏ thì §Bẫy 8 đã xảy ra.
  - [x] `tests/schema.rs`: **không đổi một dòng nào**. `SCHEMA_VERSION` giữ **1**.
  - [x] không **`src-tauri/tests/` không thêm và không sửa một dòng nào. 62 test phải ra đúng 62.**

- [x] **Task 9 — Siết hai cổng `.mjs`** (AC7)
  - [x] `check-dict-build.mjs`: `RS_FILE_FLOOR = 20` → **21** *(số thật sau Task 3)*. 🔴 Sàn phải **SÁT** số thật — sàn hở đúng bằng số tệp story vừa thêm là lỗi lượt review Story 1.10 đã bắt.
  - [x] Cập nhật comment `:57` với ngày và số thật mới.
  - [x] **Không** đụng `EXPECTED_DETACHABLE_NAMES` trong `check-dict-manifest.mjs` — vẫn **đúng hai** *(`thieu-chuu` · `vietphrase`)*. Story này không thêm lớp gỡ rời nào.
  - [x] Không nới `URL_RE`. Cổng vẫn không đọc `.db`, không tải mạng.
  - [x] Chạy `npm run check:dict` + `check:dict-manifest` — **cả hai XANH** trước khi sang Task 10.

- [x] **Task 10 — Chạy THẬT và ghi số** (AC1, AC2, AC3, AC5, AC6)
  - [x] 🔴 **Lệnh chính xác — `--layer base`, KHÔNG `--layer all`** *(§Bẫy 5)*:
        ```sh
        cargo run --release --manifest-path tools/dict-build/Cargo.toml -- \
          --raw tools/dict-build/raw --out-dir tools/dict-build/out --layer base
        ```
  - [x] ⚠️ Lượt chạy này mất **hàng chục phút** *(415.254 dòng JSONL đọc HAI lần + 4 nguồn kia + VACUUM trên ~195 MB)*. Không huỷ giữa chừng rồi tin số dở.
  - [x] Ghi bảng `SourceStats` **đầy đủ** cho nguồn mới: đọc / bỏ / **lý do bỏ** / entry / sense / example / citation.
  - [x] 🔴 Đối chiếu **AC2** với ngưỡng 1%. Lệch ⇒ **DỪNG**, không viết *"nguồn vốn thế"*. ⇒ **lệch 0,00% cả ba chỉ số**.
  - [x] Ghi **SHA-256 + kích thước byte** của `dict-core.db` mới.
  - [x] **Xác nhận `dict-thieu-chuu.db` và `dict-vietphrase.db` KHÔNG bị đụng** — `ls -la` trước/sau, so mtime **và** SHA-256 với `dict-manifest.toml`. Lệch ⇒ đã chạy nhầm `--layer all`. ⇒ **cả mtime lẫn SHA-256 giữ nguyên**.
  - [x] Ba phép nghiệm thu tay, ghi **SQL nguyên văn** để lượt rà sau tái lập được:
    - [x] SQL của AC3 *(`GROUP BY code, lang`)* ⇒ đúng sáu nguồn, đúng phân bố `lang`.
    - [x] Tra một từ tiếng Anh thật *(vd. `SELECT ... WHERE headword = 'dictionary'`)* ⇒ ra **từ loại + nghĩa tiếng Việt** *(FR34)*.
    - [x] `SELECT COUNT(*) FROM char_idx` ⇒ ghi số; chênh so với lượt trước ≈ **9** là **đúng dự báo**, không phải lỗi. ⇒ **đúng 9**.
    - [x] `ls -la` thư mục đầu ra ⇒ **0** tệp `-wal`/`-shm`.

- [x] **Task 11 — Điền lại `dict-manifest.toml`** (AC5, AC7)
  - [x] `[base].sha256` ← hash **từ chính lượt build Task 10**.
  - [x] `[base].source_version` ← thêm nguồn thứ sáu: `… · viwiktionary-en@<ngày>`. Không bỏ năm nguồn cũ.
  - [x] Cập nhật khối comment trên `[base]` — nêu **lý do** dựng lại lần này *(nguồn thứ sáu, Story 1.10b)*, không xoá lịch sử comment cũ.
  - [x] **Không đụng** hai khối `[[detachable]]` — sha256 của chúng vẫn đúng vì tệp không bị dựng lại.
  - [x] ⚠️ Task 10 + 11 + 9 phải **cùng MỘT commit** — manifest công bố một checksum cho một tệp chưa dựng là đúng lớp lỗi §Bẫy 8 của Story 1.10.

- [x] **Task 12 — Bảng kế toán NFR6 và phán quyết** (AC6)
  - [x] Dựng bảng đúng khuôn AC6, **mỗi dòng một số byte đo được**.
  - [x] Tổng bằng **byte**, rồi mới đổi MB thập phân. Đối chiếu **400.000.000**.
  - [x] Ghi **dư địa còn lại** bằng byte, và so với dự phóng **15.675.258** của `deferred-work.md:272`.
  - [x] Phán quyết **ĐẠT** *(ghi rõ "với bảy nguồn, thiếu HVTĐTD + Cổ hán văn")* hoặc **VƯỢT**, viết đúng chữ. ⇒ **ĐẠT**, dư **15.474.554** byte.
  - [x] Nếu **VƯỢT**: ghi số byte vượt, liệt kê đòn bẩy **kèm số**, rồi **DỪNG**. Quyết định tầng PRD. — *(không áp dụng)*
  - [x] Không sửa `Cargo.toml`, không `[profile.release]`, không bỏ chỉ mục, không bỏ nguồn, không subset font.

- [x] **Task 13 — Tài liệu và bàn giao** (không AC)
  - [x] `tools/dict-build/README.md`:
    - [x] Bảng nguồn: **5 nền → 6 nền** *(+ 2 gỡ rời = 8)*. Thêm hàng `viwiktionary-en`.
    - [x] Cây thư mục `src/sources/{…}` thêm `viwiktionary_en.rs`.
    - [x] 🔴 Khối giải thích **HAI VAI trên MỘT tệp thô** — `vi-extract.jsonl` được đọc **hai lần** với hai bộ lọc `lang_code`. Đây là điều một người dựng lại từ đầu không đoán ra được từ bảng thư mục.
    - [x] `version = 0.3.0`.
  - [x] `src-tauri/resources/dict/README.md`: `dict-core.db` mô tả **sáu** nguồn, không phải năm.
  - [x] `deferred-work.md`:
    - [x] 🔄 Cập nhật mục `:272` với **dư địa NFR6 THẬT** đo ở story này *(thay số dự phóng bằng số đo)*.
    - [x] 🔄 Mục `:273` *(AD mới cho đường tra cứu tiếng Anh — Winston)*: xác nhận **vẫn CHẶN 1.11b**, cộng số `char_idx` thật đo ở Task 10.
    - [x] ➕ Mục MỚI đích danh **1.11b/1.13**: dữ liệu tiếng Anh nay **CÓ THẬT** trong `dict-core.db` — đường tra cứu phải lọc theo `dict_entry.lang`, không giả định mọi hàng là `zh`.
  - [x] §Completion Notes: **lệnh chép-dán cho Ice** tải `dict-core.db` mới lên release `dict-v1` *(và cảnh báo checksum cũ)*.
  - [x] **Không sửa** `prd.md` / `epics.md` / `ARCHITECTURE-SPINE.md` / `sprint-change-proposal-2026-08-05.md` — tiền lệ quyết định #3 của Ice ở Story 1.3. Lệch giữa tài liệu và mã ⇒ ghi vào §Completion Notes để Ice sửa. *(Đã biết một lệch: `prd.md` §8.2 ghi "141.407 mục" — xem §Bẫy 13.)* ⇒ **ba** lệch, ghi ở §Completion Notes ②.
  - [x] **Không sửa** tệp story 1.9 / 1.10 *(cả hai `done`)* — chúng là bản ghi.
  - [x] **Không sửa và không xoá** `docs/dics/**`.

### Review Findings

*(code review, 2026-08-05, đối chiếu commit `dd7af61` với baseline `ed8ce52`)*

- [x] [Review][Defer] Commit `dd7af61` gộp mã story vào cùng một commit với các sửa đổi tài liệu quy hoạch không thuộc story này [git: `dd7af61`] — Task 13 và §Ranh giới phạm vi cấm dev sửa `prd.md`/`epics.md`/`ARCHITECTURE-SPINE.md`/`sprint-change-proposal-2026-08-05.md`, và File List xác nhận các tệp này "đã bị sửa TRƯỚC khi story này bắt đầu". Nội dung hợp lệ (thuộc gói `correct-course` Ice đã duyệt 2026-08-05). **Ice chốt 2026-08-05: giữ nguyên, không tách commit** — chưa push lên remote công khai, không ai khác đụng vào giữa chừng, tách lúc này chỉ thêm rủi ro thao tác git mà không đổi nội dung. Quy ước cho lần sau: commit riêng tài liệu quy hoạch trước khi bắt đầu code của story — deferred, đã quyết định
- [x] [Review][Patch] Doc-comment `run_base` ghi sai số thư mục thô [tools/dict-build/src/build.rs:100] — sửa "BỐN thư mục con" thành "NĂM", vì mệnh đề liệt kê ngay sau đó vẫn có đủ năm thư mục (`cvdict/`, `cc_cedict/`, `unihan/`, `viwiktionary/`, `en_wiktionary/`) và số thư mục thô không đổi ở story này (chỉ số NGUỒN đổi 5→6). — đã sửa
- [x] [Review][Patch] `entry_lang`/`filter_lang_code`/`pos_lang` trong `parse_line` không có kiểm tra runtime với tập giá trị đã biết `{"zh","en"}` [tools/dict-build/src/sources/wiktextract_common.rs:50-56] — đã được giảm nhẹ bởi thiết kế tham số bắt buộc + test đối chứng âm AC3, nhưng thêm `debug_assert!` sẽ bắt lỗi gõ nhầm ngay tại nguồn thay vì trông cậy hoàn toàn vào test hạ nguồn. — đã thêm `debug_assert!` tại `wiktextract_common.rs:56-59`
- [x] [Review][Defer] `vi-extract.jsonl` (273 MB) bị đọc và parse lại từ đầu hai lần (một lần mỗi vai) thay vì thiết kế một lượt đọc với hai bộ tích luỹ song song [tools/dict-build/src/build.rs:253-283] — đánh đổi có chủ ý đã ghi rõ lý do (gộp một lượt `parse()` sẽ hợp nhất headword xuyên nguồn, bị AD-19 cấm), nhưng chưa đo chi phí thời gian build, và không có bước đối chiếu nội dung/hash giữa hai lần `File::open` nếu tệp đổi giữa chừng — deferred, pre-existing design trade-off
- [x] [Review][Defer] Ngưỡng định lượng AC2 (119.039/190.543/27.396 đầu mục·nghĩa·ví dụ, lệch ≤1%) chỉ được đối chiếu một lần thủ công ở một mũi thăm dò đã bị revert khỏi cây mã; test CI đóng gói chỉ khẳng định `count > 0` cộng hai headword mẫu cố định [tools/dict-build/tests/parse.rs] — một hồi quy tương lai làm rơi một phần lớn đầu mục tiếng Anh thật sẽ không bị bắt tự động — deferred, pre-existing test-coverage gap
- [x] [Review][Defer] `epics.md` mục Story 1.10b vẫn ghi quyết định base-vs-detachable là "🔴 Quyết định phải chốt TRONG story" dù Completion Notes của chính story này đã đánh dấu AC5 **ĐẠT** — đây là lệch tài liệu/mã thứ TƯ chưa nằm trong danh sách "ba lệch" ở §Completion Notes ②; theo đúng tiền lệ (dev không sửa epics.md), nên bổ sung vào danh sách đó cho Ice — deferred, pre-existing doc/code mismatch pattern
- [x] [Review][Defer] Nguồn dual-license (CC-BY-SA-4.0 + GFDL-1.3) chỉ biểu diễn được bằng một trường `license_id` duy nhất (`LicenseRef::CcBySaAndGfdl` gộp cả hai vào một enum) [tools/dict-build/src/sources_meta.rs] — kế thừa nguyên trạng từ `viwiktionary`/`en_wiktionary` ở Story 1.9, story này chỉ tái dùng cùng khuôn mẫu và bị cấm sửa `schema.rs` — deferred, pre-existing schema limitation

---

## Dev Notes

### Ranh giới phạm vi — đọc trước khi gõ dòng đầu tiên

| Thứ | Trong story này? |
|---|---|
| Tham số hoá `lang` trong `wiktextract_common` | ✅ **Có** — hạt nhân |
| `sources/viwiktionary_en.rs` *(wrapper 3 dòng)* | ✅ **Có** |
| `SourceMeta` thứ sáu + `BASE_ALL: [;6]` | ✅ **Có** |
| Khối chèn thứ sáu trong `run_base` | ✅ **Có** |
| Dựng lại `dict-core.db` + điền lại `[base]` | ✅ **Có** — hệ quả bắt buộc của AC5 |
| `RS_FILE_FLOOR` 20 → 21 | ✅ **Có** |
| Bảng kế toán NFR6 với số thật | ✅ **Có** — kể cả khi kết luận là VƯỢT |
| **Đường tra cứu tiếng Anh** *(`core/dict/`)* | ❌ **Không** — **1.11b**, và nó đang **BỊ CHẶN** chờ AD mới |
| **Stemming / `Matcher`** *(FR40)* | ❌ **Không** — **1.12** *(AD-17)* |
| **Cổng `DictionarySource`** | ❌ **Không** — **1.13**. không `ports/mod.rs` giữ nguyên |
| **Hiển thị mục tiếng Anh ở Panel Lookup** | ❌ **Không** — **1.17**, và Sally chưa kiểm hình dạng UX *(`deferred-work.md:274`)* |
| Dựng lại `dict-thieu-chuu.db` / `dict-vietphrase.db` | ❌ **Không** — §Bẫy 5 |
| Thêm lớp gỡ rời nào *(HVTĐTD · Cổ hán văn)* | ❌ **Không** — story nối tiếp của 1.10 |
| Đổi **bất kỳ** DDL nào của `schema.rs` | ❌ **KHÔNG BAO GIỜ** — §Bẫy 8 |
| Đổi tên `viwiktionary` → `viwiktionary-zh` | ❌ **Không** — §Bẫy 6 |
| Bóc IPA của mục tiếng Anh vào `reading` | ❌ **Không** — §Quyết định #5 |
| Sửa `src-tauri/**` *(src, tests, Cargo.toml, tauri.conf.json)* | ❌ **Không** — **0 dòng** |
| Crate mới cho bất kỳ cây nào | ❌ **Không** — **0 crate** |
| Tạo GitHub Release, tải tệp lên | ❌ **Không** — Ice làm tay |
| Sửa `prd.md` / `epics.md` / `ARCHITECTURE-SPINE.md` | ❌ **Không** — ghi vào §Completion Notes |

### Trạng thái repo hiện tại — số, không phải mô tả

> ⚠️ **Baseline là CÂY LÀM VIỆC.** Story 1.9 và 1.10 *(cả hai `done`)* có phần **chưa commit**. **Chạy `git status` trước khi tin bất kỳ con số nào dưới đây.**

| Thứ | Số / trạng thái |
|---|---|
| `.rs` dưới `tools/dict-build/src/**` | **20** — thành **21** sau Task 3 |
| `.rs` dưới `src-tauri/src/**` | **26** — không phải không đổi |
| Test `tools/dict-build` | **88** *(bản ghi Story 1.10 — xác nhận lại ở Task 1)* |
| Test `src-tauri` | **62** — không phải không đổi |
| `RS_FILE_FLOOR` *(`check-dict-build.mjs:57`)* | **20** — nâng lên **21** |
| `EXPECTED_DETACHABLE_NAMES` *(`check-dict-manifest.mjs`)* | `['thieu-chuu','vietphrase']` — **không đổi** |
| `dict_source` trong `dict-core.db` | **5** hàng → **6** |
| `dict_entry` trong `dict-core.db` | **473.499**, **100% `lang='zh'`**, **0** mục `en` |
| `dict-core.db` hiện có | **154.464.256 byte**, sha `741e1666…`, `2026-08-05 09:32` |
| `dict-thieu-chuu.db` | **5.787.648 byte**, sha `e9417c12…` — không đụng |
| `dict-vietphrase.db` | **160.083.968 byte**, sha `9d304210…` — không đụng |
| `raw/viwiktionary/vi-extract.jsonl` | **273.075.442 byte**, **415.254 dòng**, mtime `2026-08-04 20:27` |
| `dict_meta('built_at')` của base | `2026-08-04T23:53:16Z` — dẫn xuất từ mtime `raw/`, **không đổi** ở story này |
| `SCHEMA_VERSION` / `PRAGMA user_version` | **1** — không giữ nguyên |
| `builder_version` | `0.2.0` → **`0.3.0`** ở story này |
| Miễn trừ `dict-build:allow` đang dùng | **Kiểm A: 3** · **Kiểm E: 1** *(đã chạy `npm run check:dict` 2026-08-05 — cả sáu phép kiểm XANH)*. Khai báo ở `model.rs:93-94` · `sources/unihan.rs:85` · `sources/wiktextract_common.rs:162` · `build.rs:364` |
| Release `dict-v1` trên GitHub | **CHƯA TỒN TẠI** — Ice chưa chạy lệnh |
| Trần NFR6 | **400.000.000 byte** *(nâng 2026-08-05; **không còn** 200.000.000)* |
| Payload hôm nay *(7 nguồn)* | **343.991.430 byte** |

### 🔴 Nguồn thô: MỘT tệp, HAI vai — đây là toàn bộ nội dung kỹ thuật của story

`tools/dict-build/raw/viwiktionary/vi-extract.jsonl` là bản trích **toàn ấn bản** `vi.wiktionary.org` — nó chứa mục từ của **mọi** ngôn ngữ mà ấn bản đó có. Đã đo thật trên tệp trong repo:

| Chỉ số | Giá trị |
|---|---:|
| Byte | **273.075.442** |
| Dòng | **415.254** |
| Dòng có chuỗi `"lang_code": "en"` *(kể cả lồng trong `translations`)* | 138.513 |
| Dòng có chuỗi `"lang_code": "zh"` *(kể cả lồng)* | 4.254 |

⚠️ **Hai số cuối là số dòng CHỨA chuỗi đó ở BẤT KỲ đâu, không phải số đầu mục.** Bản ghi Wiktextract mang `lang_code` **lồng bên trong** `translations[]`/`sounds[]`, nên một mục từ tiếng Việt có bản dịch tiếng Anh vẫn khớp. **Không dùng `grep` để nghiệm thu AC2** — chỉ `stats.entries` của lượt build mới là con số thật.

**Hai vai, quyết bởi bộ lọc `lang_code` ở top-level:**

| Vai | `filter_lang_code` | `RawEntry.lang` | `pos_lang` | Phục vụ | Trạng thái |
|---|---|---|---|---|---|
| **B** | `"zh"` | `'zh'` | `'vi'` | Lớp từ loại ZH *(PRD §8.3)* | ✅ đã dựng — **1.598** đầu mục |
| **A** | `"en"` | 🔴 **`'en'`** | `'vi'` | **FR34**, cặp **Anh → Việt** | 🔴 **story này** — **119.039** đầu mục |

🟢 **`pos_lang = 'vi'` cho CẢ HAI vai** — ấn bản `vi` có `pos_title` đã sẵn tiếng Việt. Đã kiểm thật trên fixture: mục `API` mang `pos_title = "Danh từ"`, mục `Wikipedia` mang `"Danh từ riêng"`. **Không** đặt `pos_lang = 'en'` — FR35 chỉ đòi đánh dấu nhãn **NGOẠI NGỮ**, và nhãn ở đây là tiếng Việt.

### Số học của `SourceStats` — đọc trước khi hoảng vì một con số không khớp

`stats.lines_read` được `build::ingest` đếm là **số phần tử iterator**, **không phải số dòng tệp**:

```
lines_read = số ParseIssue  +  số RawEntry ĐÃ GỘP theo headword
```

Kiểm chứng trên vai B đã dựng: `đọc 415.115 = bỏ 413.517 + giữ 1.598`. Tệp có **415.254** dòng. Chênh **139** là các dòng đã bị **gộp** vào một headword đã thấy trước đó — chúng không xuất hiện thành phần tử iterator riêng.

Suy ra cho vai A, từ số mũi thăm dò:

| | Giá trị |
|---|---:|
| Dòng bỏ vì `lang_code != en` | **281.935** |
| Dòng bỏ vì không có gloss dùng được | **127** |
| `lines_read` *(iterator)* | **401.101** |
| **`entries` sau khi gộp headword** | **119.039** |
| ⇒ dòng `lang_code=en` thật *(dẫn xuất)* | ≈ **133.319** |

⇒ **Đối chiếu AC2 với `entries`/`senses`/`examples`, không với `lines_read`, không với `wc -l`, không với `grep -c`.**

### Chín cái bẫy — bảy trong chín cho ra một lượt CI XANH với hành vi SAI

#### Bẫy 1 — `wiktextract_common` viết cứng `lang: "zh"` 🔴 **đắt nhất trong story**

`tools/dict-build/src/sources/wiktextract_common.rs:143`:

```rust
Ok(Some(RawEntry {
    lang: "zh".to_string(),   // 🔴 HẰNG SỐ, không phải tham số
    ...
```

Không sửa dòng này ⇒ **119.039 đầu mục tiếng Anh vào `dict_entry` mang `lang = 'zh'`**. Và mọi thứ khác **XANH**: build thành công, `require_nonempty` đạt, số entry/sense/example khớp AC2 tuyệt đối, `sqlite_master` không đổi, hai cổng `.mjs` xanh. **Chỉ AC3 bắt được** — đó chính là lý do AC3 có **đối chứng âm bắt buộc** thay vì chỉ một phép khẳng định dương.

**Hậu quả nếu lọt:** đường tra cứu tiếng Trung của Story 1.11 lọc theo `lang='zh'` sẽ nhận về `"Wiktionary"`, `"API"`, `"dictionary"` khi người dùng tra chữ Hán — và FR34 vẫn không có dữ liệu vì không hàng nào mang `lang='en'`. Story này sẽ được đánh dấu `done` mà không giao được điều nó tồn tại để giao.

**Luật:** `entry_lang` là **tham số bắt buộc** của cả `parse_line` và `parse`. Không giá trị mặc định.

#### Bẫy 2 — "Tối ưu": đọc tệp MỘT lần, phát cả hai vai 🔴

Tệp 273 MB đọc hai lần trông lãng phí, và cám dỗ là viết một hàm đọc một lượt rồi rẽ nhánh theo `lang_code`. Ba thứ vỡ:

1. **AD-19.** `wiktextract_common::parse` gộp theo headword **trong một lượt gọi**. Một lượt gọi phát cả hai vai ⇒ hai nguồn dùng chung một `HashMap` ⇒ một headword xuất hiện ở cả hai vai bị gộp thành **một** `RawEntry` mang **một** `source_id`. Đó **là** hợp nhất xuyên nguồn, đúng thứ miễn trừ `dict-build:allow .entry(` ở `:162` tuyên bố **không bao giờ** xảy ra.
2. **Hình dạng chung của năm parser vỡ.** Mọi module `sources/*` phơi đúng `parse(reader) -> impl Iterator`. Một hàm hai đầu ra không vừa khuôn đó, và khuôn đó **là** điều kiện để Kiểm A kiểm được *(doc-comment `model.rs`)*.
3. **Không tiết kiệm được gì đáng kể.** Lượt build này chạy **tay, một lần**, trên máy người dựng. `VACUUM` trên ~195 MB tốn nhiều hơn một lượt đọc tuần tự 273 MB.

**Luật:** hai lần `File::open`, hai lượt `parse` độc lập, hai `source_id`. Đúng như `run_base` đã làm với năm nguồn.

⚠️ **Đi kèm một sự thật về bộ nhớ:** `wiktextract_common::parse` **không lazy** — nó tích luỹ **toàn bộ** `RawEntry` vào `HashMap` trước khi phát phần tử đầu tiên *(phải đọc hết mới biết một headword còn xuất hiện ở dòng sau không)*. Với 119.039 entry + 190.543 sense + 27.396 example, đỉnh bộ nhớ vài trăm MB là **bình thường**, không phải rò rỉ. Không "sửa" bằng cách bỏ phép gộp.

#### Bẫy 3 — Nghiệm thu AC2 bằng `grep`/`wc -l` 🟡

Đã bàn ở §Số học của `SourceStats`. `grep -c '"lang_code": "en"'` cho **138.513** — cách **119.039** đúng 16%, tức "lệch quá 1%" theo AC2, và một dev tin con số đó sẽ đi sửa parser cho tới khi nó hỏng thật.

**Luật:** con số nghiệm thu **duy nhất** là `SourceStats` in ra ở cuối lượt build.

#### Bẫy 4 — Điền `[base].sha256` từ một lượt build khác lượt sẽ upload 🔴

Bài học nguyên văn của lượt code review Story 1.10 *(`built_at` mili-giây ⇒ mọi lượt build ra hash khác)*. 🟢 **Nay đã vá** — `built_at` dẫn xuất từ mtime của `raw/`, nên cùng cây nguồn ⇒ cùng hash. **Nhưng bản vá đó chỉ cứu bạn nếu `raw/` không bị đụng.** Chép một tệp vào `raw/` *(kể cả một tệp không liên quan)* làm `newest_mtime_secs` đổi ⇒ `built_at` đổi ⇒ **cả ba** hash đổi.

**Luật:** **Không đụng `raw/` sau Task 10.** Điền manifest từ chính lượt chạy đó, cùng một commit với Task 9 và Task 12.

#### Bẫy 5 — Chạy `--layer all` 🔴 lỗi ĐÃ XẢY RA MỘT LẦN ở Story 1.10

`--layer all` là **mặc định** và nó dựng lại **cả ba** tệp. Ở Story 1.10 điều này đã xảy ra thật: `dict-core.db` bị dựng lại ngoài ý muốn, đổi từ `154.836.992` → `154.464.256` byte, và `[base].sha256` phải sửa theo — lượt code review ghi thẳng *"đây là lỗi ở LỆNH đã chạy, không phải ở kiến trúc"*.

Ở story này hậu quả nặng hơn: dựng lại hai lớp gỡ rời làm **hai `sha256` đang đúng trong manifest thành sai**, mà **không cổng nào bắt được** *(`check-dict-manifest.mjs` cố ý không đọc `.db`)*.

**Luật:** `--layer base`. Và Task 10 **bắt buộc** kiểm lại mtime + SHA-256 của hai tệp gỡ rời sau lượt chạy.

#### Bẫy 6 — Đổi tên `viwiktionary` → `viwiktionary-zh` "cho đối xứng" 🟡

Cám dỗ thẩm mỹ mạnh: hai vai, hai hậu tố. Chi phí thật:

- `dict-manifest.toml` `[base].source_version` ghi `viwiktionary@…` — chuỗi chép tay, không cổng nào đối chiếu cho nguồn **nền** *(Kiểm F chỉ soi lớp gỡ rời)*.
- `prd.md` §8.2/§8.3, `epics.md`, `deferred-work.md`, `README.md` đều gọi nguồn đó là `viwiktionary`. Dev **không được sửa** ba tệp đầu.
- `dict_source.code` là khoá đối chiếu **xuyên tệp** mà Story 1.13 sẽ dùng *(§Bẫy 9 của Story 1.10: khoá theo `code`, không theo `id`)`. Đổi nó là đổi một khoá công khai để lấy đối xứng.

**Luật:** vai B giữ `viwiktionary`. Vai A là `viwiktionary-en`. Sự bất đối xứng được **giải thích bằng doc-comment**, không bằng một lượt đổi tên.

#### Bẫy 7 — Xoá lưới `exactly_five_sources_with_the_epics_md_codes` thay vì cập nhật nó 🟡

Test này tồn tại với đúng một mục đích, ghi trong doc-comment của nó: khoá *"Thiều Chửu · Cổ hán văn · VietPhrase · HVTĐTD **KHÔNG** thuộc `BASE_ALL`"*. Story 1.10 §Bẫy 4 đã cấm gộp hai danh sách để `assert_eq!(7)`.

Story này **được phép** đổi `5` → `6` — vì nó thêm một nguồn **NỀN** thật, không phải kéo một lớp gỡ rời vào. Nhưng:

**Luật:** đổi **số** và **danh sách mã**, không **giữ nguyên doc-comment nói về lớp gỡ rời**. Không xoá test. Không đổi `base_and_detachable_code_sets_are_disjoint`.

#### Bẫy 8 — Đổi lược đồ để chiều dữ liệu tiếng Anh 🔴

Mục từ tiếng Anh mang thứ mục từ tiếng Trung không có: **IPA** *(`sounds[].ipa`)*, và không mang thứ mục tiếng Trung có: `han_viet`, `headword_simp`. Cám dỗ là thêm một cột `ipa`. Ba thứ vỡ cùng lúc — nguyên văn §Bẫy 1 của Story 1.10:

1. **AC4 của Story 1.10 chết ngay.** `sqlite_master` của `dict-core.db` khác hai tệp gỡ rời ⇒ runtime buộc phải hỏi *"tệp này có cột đó không?"* trước mỗi truy vấn ⇒ đúng thứ AD-10 cấm.
2. **`SCHEMA_VERSION` desync.** Bump lên `2` thì hai tệp gỡ rời *(đang là `1`, **không** dựng lại)* thành tệp cũ hơn — mà luật *"gặp phiên bản mới hơn thì từ chối mở"* làm đường đọc 1.11 từ chối chính tệp mới.
3. **Không bump** thì ba tệp cùng khai `schema_version = 1` mà lược đồ khác nhau — hỏng im lặng.

**Luật:** **0 dòng đổi trong `schema.rs`.** IPA ⇒ **không bóc** *(§Quyết định #5)*. Thật sự không vừa ⇒ **DỪNG và hỏi Ice**.

#### Bẫy 9 — Tưởng ~9 cặp `char_idx` là một lỗi 🟡

`char_idx::is_han` chỉ chèn ký tự thuộc khối CJK. Đầu mục tiếng Anh là chữ Latin ⇒ **0 cặp**, trừ vài mục lẫn chữ Hán. Mũi thăm dò đo được đúng **9** cặp trên 119.039 đầu mục.

**Đó là ĐÚNG, không phải bug** — và nó chính là bằng chứng kỹ thuật khiến `AD-26` nhánh 2 *(chuỗi con 1–2 ký tự qua `char_idx`)* **không áp được** cho tiếng Anh, tức lý do **Story 1.11b bị chặn** chờ AD mới của Winston *(`deferred-work.md:273`)*.

**Luật:** Không đụng `char_idx.rs`. Không "sửa" `is_han` để nhận chữ Latin — làm thế là dựng một chỉ mục vô nghĩa **và** phá `char_idx` của tiếng Trung. Ghi con số thật vào §Debug Log References, không im lặng.

---

### Quyết định thiết kế — đã chốt, không phải lựa chọn của dev

#### #1 — 🔴 Lớp tiếng Anh vào `dict-core.db`, không phải tệp `.db` gỡ rời

`epics.md` giao câu hỏi này cho story quyết. **Chốt: nguồn NỀN thứ sáu.** Đọc đầy đủ bốn lý do:

| Lý do | Chứng cứ |
|---|---|
| PRD xếp nó vào hàng **Nền** | `prd.md` §8.2 — cả hai hàng `viwiktionary` *(vai A và vai B)* đều mang nhãn **Nền** |
| Giấy phép **sạch y hệt** vai B | CC-BY-SA 4.0 + GFDL 1.3, **cùng tệp thô, cùng kho** ⇒ không có rủi ro pháp lý nào để FR112 phải gỡ |
| **AD-10 liệt kê đích danh** lớp gỡ rời | *"Thiều Chửu, Cổ hán văn, VietPhrase, HVTĐTD mỗi nguồn một file riêng"* — lớp tiếng Anh không có tên trong đó, và AD không nói *"mọi nguồn tương lai đều gỡ rời"* |
| **FR34 là phạm vi LÕI** | Làm nó gỡ rời nghĩa là xoá một tệp làm **biến mất cả cặp Anh → Việt**. FR36 nói sản phẩm phải **chạy đầy đủ** khi thiếu một lớp gỡ rời — mệnh đề đó không thể đúng nếu lớp đó mang trọn một cặp ngôn ngữ |

**Cái giá đã biết và chấp nhận:** phải dựng lại `dict-core.db` *(hàng chục phút)* và điền lại `[base].sha256`. 🟢 Rẻ và tái lập được sau bản vá `built_at`.

**Cái được:** không thêm tệp `.db` thứ tư vào `bundle.resources` *(Story 10.1)*, không thêm mục `[[detachable]]` thứ ba *(cổng manifest giữ nguyên)*, không thêm một `SOURCE_VERSION` const phải đồng bộ tay *(Kiểm F)*, và Story 1.13 vẫn mở đúng **ba** tệp.

#### #2 — `code = "viwiktionary-en"`; vai B giữ nguyên `viwiktionary`

Xem §Bẫy 6. `display_name` phân biệt được bằng mắt vì màn Attribution *(Story 10.4)* sẽ liệt kê cả hai và *"Wiktionary tiếng Việt"* xuất hiện hai lần giống hệt là một lỗi hiển thị.

#### #3 — Tham số hoá `wiktextract_common`, không viết parser thứ sáu

Sao chép `wiktextract_common.rs` thành `wiktextract_en.rs` là cách nhanh nhất để tránh đụng chữ ký hàm — và là cách chắc nhất để hai bản sao trôi khỏi nhau ở lượt sửa sau. Tiền lệ đã có ngay trong crate này: `char_idx::is_han` được `pub(crate)` **chính vì** lượt review 1.10 chặn một bản sao thứ hai của bảng dải CJK.

`sprint-change-proposal` §1 nói thẳng: *"`wiktextract_common` đã tham số hoá sẵn `filter_lang_code`"* — chi phí xây dựng **gần bằng không**.

#### #4 — Một module `.rs` riêng cho vai A, không nhét vào `viwiktionary.rs`

Mỗi nguồn một module là khuôn của cả bảy nguồn hiện có, và `sources/mod.rs` doc-comment ghi nó thành luật. Wrapper 3 dòng, đúng khuôn `en_wiktionary.rs`. Hệ quả: **21 tệp `.rs`** ⇒ `RS_FILE_FLOOR = 21` *(Task 9)*.

#### #5 — IPA của mục tiếng Anh KHÔNG bóc vào `reading`

`wiktextract_common` bóc `reading` chỉ khi `sounds[].tags` chứa **cả** `Mandarin` **và** `Pinyin` ⇒ mục tiếng Anh cho `reading = NULL`. Đã kiểm thật trên fixture: ba mục `en` có `sounds[].ipa` nhưng không tag Pinyin.

**Giữ nguyên.** Ba lý do: FR34 không đòi phiên âm; `dict_entry.reading` doc-comment nói nó là *"Pinyin hoặc cách đọc khác"* và trộn IPA vào là trộn hai hệ ký hiệu vào một cột không có cột `reading_kind`; và thêm nó là **thay đổi hành vi của một hàm dùng chung** cho ba nguồn ở một story đang có bảy bẫy khác.

Ghi vào §Completion Notes như một mục có thể xét ở 1.11b/1.17 nếu UX cần.

#### #6 — `char_idx` và `entry_fts` không đụng

Xem §Bẫy 9. `entry_fts` *(trigram trên headword)* **sẽ** tự động chỉ mục hoá đầu mục tiếng Anh — đó là hành vi sẵn có, không phải quyết định của story này, và nó **chạy được** nhưng **không phải hình dạng đúng** cho tiếng Anh *(`sprint-change-proposal` §2.4)*. Chiến lược truy vấn là việc của AD mới + Story 1.11b.

#### #7 — Thứ tự chèn: nguồn thứ sáu vào CUỐI `BASE_ALL`

`dict_source.id` sinh theo thứ tự chèn. Thêm vào cuối ⇒ năm nguồn cũ giữ nguyên `id 1..5`. Chèn vào giữa *(cạnh `viwiktionary` cho "gọn")* làm mọi `id` sau nó dịch đi — vô hại **hôm nay** *(mọi FK nằm trong cùng tệp và cùng lượt dựng)*, nhưng nó làm hai lượt dựng khác nhau ra hai bảng `id` khác nhau mà không có lý do gì.

#### #8 — Tái dùng baseline `.dmg` **2.334.696** và license **35.149** của Story 1.9/1.10

Điều kiện: story này **không chạm** `src-tauri/src/**`, `bundle.resources`, hay bất kỳ tệp nào **thật sự** nằm trong bundle. `src-tauri/resources/dict/README.md` *(Task 13)* **không** nằm trong `bundle.resources` — chỉ `resources/fonts/*` + `resources/license/*` — nên nó không đi vào `.dmg`.

🔴 **Kiểm điều kiện này bằng `git diff --stat -- src-tauri/ src/ package.json` trước khi tái dùng số**, đúng cách Story 1.10 đã phải sửa một lần vì khai sai.

---

### Trí tuệ nhận từ Story 1.10 *(và lượt code review của nó)*

| Bài học | Áp vào story này ở đâu |
|---|---|
| **`--layer all` dựng lại thứ bạn không định dựng lại** — đã xảy ra thật, `[base].sha256` phải sửa theo | §Bẫy 5 · Task 10 dùng `--layer base` + kiểm lại hai tệp gỡ rời |
| **`built_at` phải tất định** — `strftime('now')` từng làm mọi `sha256` trong manifest chỉ đúng cho đúng một lượt chạy | §Bẫy 4 · không đụng `raw/` sau Task 10 |
| **Sàn cổng phải SÁT số thật** — sàn 18 với 20 tệp thật cho phép xoá cả hai parser mà Kiểm C vẫn xanh | Task 9: `RS_FILE_FLOOR = 21`, không để 20 |
| **Fixture phải trích THẬT** — lượt review 1.9 bắt được fixture bịa 20/20 dòng | Task 7 |
| **Một danh sách rỗng không phải "đạt"** — Kiểm D/E được bổ sung sàn sau review | Task 8: mọi test đếm đều có đối chứng âm |
| **Đối chứng âm mạnh hơn khẳng định dương** — `dict_core_holds_zero_rows_for_any_detachable_code` | AC3 có **hai** đối chứng âm |
| **`dict_source.id` không toàn cục giữa các tệp** — khoá theo `code` | §Quyết định #7 · bàn giao 1.13 |
| **Dev không sửa tài liệu quy hoạch** — tiền lệ Ice ở Story 1.3, lặp lại ở 1.9 và 1.10 | Task 13 · §Bẫy 13 |
| **Test lock không bị xoá để cho qua** — §Bẫy 4 của 1.10 | §Bẫy 7 |

### Trí tuệ từ git — năm commit gần nhất

```
ed8ce52 Add fixtures for Thieu Chuu and VietPhrase dictionaries; implement integration tests for detachable layers
a3ed5cd Add integration tests for dict-build and schema validation
d9bc252 feat: implement error handling for configuration loading and enhance accessibility
0ff36a0 Add Vietnamese translations for new error messages related to data store operations
85f4529 feat: Implement Library, Workspace, and Reading modes with corresponding UI components
```

**Đọc được:** hai commit gần nhất **đều** là *fixture + integration test* cho `tools/dict-build`. Khuôn đã ổn định: fixture trích thật dưới `tests/fixtures/raw/<nguồn>/`, test tích hợp **dựng thật từ fixture vào thư mục tạm** rồi truy vấn SQL *(`tests/parse.rs::build_fixture_db`, `tests/layers.rs::build_all_fixture_dbs`)*, không mock, không test đơn vị trên một chuỗi JSON rời.

**Story này đi theo đúng khuôn đó** — không phát minh cách test mới. `tests/parse.rs::build_fixture_db()` đã sẵn dựng base từ fixture; mọi test mới của Task 8 bám vào nó.

---

### Project Structure Notes

**Tệp MỚI (1):**

```
tools/dict-build/src/sources/viwiktionary_en.rs
```

**Tệp SỬA (≤ 9):**

```
tools/dict-build/src/sources/wiktextract_common.rs   # tham số entry_lang  (Task 2)
tools/dict-build/src/sources/viwiktionary.rs         # truyền "zh"          (Task 2)
tools/dict-build/src/sources/en_wiktionary.rs        # truyền "zh"          (Task 2)
tools/dict-build/src/sources/mod.rs                  # + pub mod            (Task 3)
tools/dict-build/src/sources_meta.rs                 # SourceMeta #6, BASE_ALL[;6] (Task 4)
tools/dict-build/src/build.rs                        # khối chèn thứ sáu    (Task 5)
tools/dict-build/Cargo.toml                          # version 0.3.0        (Task 6)
tools/dict-build/tests/{parse.rs,fixtures/raw/viwiktionary/vi-extract.jsonl}  (Task 7,8)
scripts/check-dict-build.mjs                         # RS_FILE_FLOOR 21     (Task 9)
dict-manifest.toml                                   # [base] sha256+version (Task 11)
```

**Tài liệu (3):** `tools/dict-build/README.md` · `src-tauri/resources/dict/README.md` · `_bmad-output/implementation-artifacts/deferred-work.md`

**KHÔNG đụng:** `tools/dict-build/src/{schema,insert,model,char_idx,finalize,licenses,main}.rs` · `tools/dict-build/src/sources/{cvdict,cc_cedict,unihan,thieu_chuu,vietphrase,cedict_common}.rs` · `tools/dict-build/tests/{schema,layers}.rs` · `scripts/check-dict-manifest.mjs` · **toàn bộ `src-tauri/**`** · **toàn bộ `_bmad-output/planning-artifacts/**`** · `docs/dics/**`

---

### Thông tin kỹ thuật ngoài — không cần tra thêm ở lượt thi công

**Không có phụ thuộc mới nào.** Cây `tools/dict-build` giữ nguyên bốn crate *(`Cargo.lock` 2026-08-04)*:

| Crate | Phiên bản ghim | Giấy phép | Dùng cho |
|---|---|---|---|
| `rusqlite` *(feature `bundled`)* | 0.37.0 | MIT | SQLite nhúng, FTS5, trigram |
| `serde` / `serde_json` | 1.0.229 / 1.0.151 | MIT OR Apache-2.0 | đọc JSONL Wiktextract |
| `sha2` | 0.10.9 | MIT OR Apache-2.0 | SHA-256 của `.db` |
| `tempfile` *(dev-only)* | 3.x | MIT OR Apache-2.0 | thư mục tạm trong test |

**Hình dạng JSONL Wiktextract** — trường THẬT, đã kiểm trên tệp trong repo *(không suy từ tài liệu)*: `word` · `lang_code` · `pos` · `pos_title` · `senses[].glosses[]` · `senses[].examples[].text` + `.english`/`.translation` · `sounds[]`. Ấn bản `vi` **CÓ** `pos_title` bằng tiếng Việt. `lang_code` xuất hiện **cả ở top-level lẫn lồng trong `translations[]`** — chỉ top-level mới quyết định vai.

**Giấy phép nguồn:** CC-BY-SA 4.0 + GFDL 1.3, **y hệt** vai B ⇒ dùng lại `LicenseRef::CcBySaAndGfdl` và hai tệp `assets/licenses/{CC-BY-SA-4.0,GFDL-1.3}.txt` đã có. Không tải thêm văn bản giấy phép nào.

**`SQLite`/FTS5:** không đổi. `remove_diacritics 2` cần SQLite ≥ 3.27; `bundled` vượt xa.

---

### References

- **Story gốc + AC:** [`epics.md`](../planning-artifacts/epics.md) §Story 1.10b *(`:1407-1432`)*
- **Nguồn gốc + số đo mũi thăm dò:** [`sprint-change-proposal-2026-08-05.md`](../planning-artifacts/sprint-change-proposal-2026-08-05.md) §1 *(bảng mũi thăm dò)* · §2.4 *(char_idx 9 cặp)* · §2.5 *(NFR6)* · §5 *(tiêu chí thành công)*
- **Hai vai của viwiktionary:** [`prd.md`](../planning-artifacts/prds/prd-AuraTranslate-2026-08-02/prd.md) §8.2 *(bảng nguồn + cảnh báo)* · §8.3 *(khoanh phạm vi "chỉ vai B")*
- **FR34:** `epics.md:144` · **FR35** *(nhãn ngoại ngữ)*: `epics.md:146` · **FR29/FR30** *(một nghĩa một hàng, ví dụ theo từ loại)*: `epics.md`
- **NFR6 trần 400.000.000:** `prd.md:826` · `prd.md:834` *("sửa lần hai")* · `epics.md:336` · giả định **[A2]** `prd.md:1073`
- **NFR8:** `prd.md:858` *(chi phí chỉ mục theo tỷ lệ nguồn, không phải hằng số)*
- **AD-10** *(lớp gỡ rời — liệt kê đích danh bốn lớp)*: [`ARCHITECTURE-SPINE.md`](../planning-artifacts/architecture/architecture-AuraTranslate-2026-08-02/ARCHITECTURE-SPINE.md) `:147`
- **AD-19** *(không hợp nhất nguồn)*: `ARCHITECTURE-SPINE.md:288`
- **AD-25** *(artifact có phiên bản + checksum)*: `ARCHITECTURE-SPINE.md:326`
- **AD-26** *(ba nhánh truy vấn **tiếng Trung** — không áp cho EN)*: `ARCHITECTURE-SPINE.md:332`
- **AD-27** *(FTS chính phân biệt dấu)*: `ARCHITECTURE-SPINE.md:338`
- **Story trước, bài học + bảng kế toán NFR6:** [`1-10-dong-goi-bon-lop-go-roi-thanh-file-doc-lap.md`](1-10-dong-goi-bon-lop-go-roi-thanh-file-doc-lap.md) §Debug Log Task 11 · §Bẫy 1–9 · §Quyết định #1–#8 · §Review Follow-ups
- **Bàn giao đang mở:** [`deferred-work.md`](deferred-work.md) `:272` *(dư địa NFR6)* · `:273` *(AD mới — CHẶN 1.11b)* · `:274` *(UX Panel Lookup)* · `:275` *(SPEC FR34)*
- **Mã sẽ sửa:** `tools/dict-build/src/sources/wiktextract_common.rs:143` *(hằng `lang: "zh"`)* · `:170-211` *(`parse`, phép gộp headword)* · `sources_meta.rs:136` *(`BASE_ALL`)* · `build.rs:210-246` *(khuôn khối chèn)* · `check-dict-build.mjs:57` *(`RS_FILE_FLOOR`)*

---

## Dev Agent Record

### Agent Model Used

`claude-opus-5` (Claude Code — bmad-dev-story)

### Debug Log References

#### Task 1 — Đường cơ sở, đo 2026-08-05

`git rev-parse HEAD` ⇒ `ed8ce52c50d8cccf8f6bb799532f67896dd018da`

`git status --porcelain` ⇒ **không một tệp mã nào bị sửa.** 10 tệp `M` + 2 tệp `??`, **tất cả** dưới `_bmad-output/**`. ⇒ điều kiện §Quyết định #8 *(tái dùng baseline `.dmg` 2.334.696 + license 35.149)* **đạt tại thời điểm bắt đầu** — kiểm lại lần hai ở Task 12.

| Chỉ số | Bản ghi story | **Đo thật Task 1** | Khớp? |
|---|---:|---:|---|
| `.rs` dưới `tools/dict-build/src/**` | 20 | **20** | ✅ |
| `.rs` dưới `src-tauri/src/**` | 26 | **26** | ✅ |
| Test `src-tauri` | 62 | **62** | ✅ |
| Test `tools/dict-build` | 88 | 🔴 **110** | ❌ **bản ghi story sai** |
| Crate Rust `check-deps.mjs` đếm | — | **326** | *(chốt: phải không đổi sau story)* |
| Gói npm `check-deps.mjs` đếm | — | **104** | *(chốt: phải không đổi sau story)* |
| Miễn trừ `dict-build:allow` — Kiểm A | 3 | **3** | ✅ |
| Miễn trừ `dict-build:allow` — Kiểm E | 1 | **1** | ✅ |

🔴 **Lệch duy nhất: 110 test `tools/dict-build`, không phải 88.** Phân rã thật: `67` *(unit trong `src/`)* + `6` + `13` + `15` + `9` + `0` = **110**. Con số **88** trong §Trạng thái repo hiện tại là bản ghi Story 1.10 **lỗi thời** — hai commit `a3ed5cd` và `ed8ce52` *(cả hai là "add integration tests")* đổ thêm test vào sau khi con số đó được ghi. ⇒ **Ngưỡng hồi quy của story này là 110, không phải 88.** Không sửa tệp story 1.10 *(§Task 13 cấm)*.

**Sáu lệnh, tất cả XANH:**

| Lệnh | Kết quả |
|---|---|
| `npm run build` | ✅ `vue-tsc` hai `tsconfig` sạch, `vite build` 44 module → `dist/` |
| `cargo test --locked --manifest-path src-tauri/Cargo.toml` | ✅ **62** passed, 0 failed |
| `cargo test --manifest-path tools/dict-build/Cargo.toml` | ✅ **110** passed, 0 failed |
| `npm run check:deps` | ✅ Kiểm 1 + Kiểm 2 đạt (326 crate · 104 gói) |
| `npm run check:dict` | ✅ **cả sáu** phép kiểm A–F đạt |
| `npm run check:dict-manifest` | ✅ cú pháp + luật đạt |
| `npm run check:i18n` | ✅ A–E đạt (16 khoá · 9 placeholder) |

**Trạng thái tệp trước khi đụng gì — đối chiếu ở Task 10:**

| Tệp | Byte | SHA-256 | mtime |
|---|---:|---|---|
| `raw/viwiktionary/vi-extract.jsonl` | 273.075.442 | — | `2026-08-04 20:27` |
| `out/dict-core.db` | 154.464.256 | `741e1666…3450a34` | `2026-08-05 09:32` |
| `out/dict-thieu-chuu.db` | 5.787.648 | `e9417c12…e43c9d5` | `2026-08-05 09:32` |
| `out/dict-vietphrase.db` | 160.083.968 | `9d304210…a78f735` | `2026-08-05 09:33` |

Cả bốn dòng khớp §Trạng thái repo hiện tại **byte-for-byte và hash-for-hash**. `[base].source_version` trong manifest hôm nay liệt kê **năm** nguồn — xác nhận nguồn thứ sáu chưa tồn tại.

#### Task 7 — Lệnh trích fixture, tái lập được

Fixture `tests/fixtures/raw/viwiktionary/vi-extract.jsonl` đi từ **19 → 23** dòng. **0 dòng bị xoá** *(đối chiếu bằng `diff` với bản sao trước khi ghép — `grep -c '^<'` ⇒ **0**)*.

Bốn dòng thêm vào, trích **NGUYÊN VĂN** từ `raw/viwiktionary/vi-extract.jsonl` thật:

| Dòng nguồn | Headword | `pos_title` | senses | examples | Phục vụ |
|---:|---|---|---:|---:|---|
| **151** | `dictionary` | Danh từ | 2 | **3** | ví dụ tiếng Anh · gloss có dấu *(AC8)* · phép tra tay của Task 10 |
| **194** | `lock` | Danh từ | 9 | 4 | headword ba dòng — phép gộp vai A |
| **195** | `lock` | Ngoại động từ | 6 | 1 | ↑ |
| **196** | `lock` | Nội động từ | 4 | 1 | ↑ |

**Lệnh trích** *(chạy từ gốc repo — cắt khoá top-level parser không đọc, không đổi một giá trị nào)*:

```python
import json
src = "tools/dict-build/raw/viwiktionary/vi-extract.jsonl"
want = [151, 194, 195, 196]
KEEP = ["word", "lang", "lang_code", "pos", "pos_title", "sounds", "senses"]
for i, l in enumerate(open(src, encoding="utf-8"), 1):
    if i in want:
        d = json.loads(l)
        print(json.dumps({k: d[k] for k in KEEP if k in d},
                         ensure_ascii=False, separators=(",", ":")))
    if i > 200: break
```

**Khoá bị CẮT** *(không khoá nào được `parse_line` đọc)*: `anagrams` · `categories` · `derived` · `etymology_texts` · `forms` · `hyphenations` · `hyponyms` · `related` · `tags`. Đã chạy phép đối chiếu ngược từng khoá còn lại với dòng gốc ⇒ **✅ MỌI GIÁ TRỊ KHỚP NGUỒN THẬT**, 4/4 dòng. Kích thước sau khi cắt: 4.792 → 1.628 · 1.704 → 1.199 · 983 → 654 · 1.268 → 570 byte.

**Giá trị thật đã kiểm bằng mắt** *(không bịa)*:

- `dictionary` glosses = `['Từ điển.', 'Có tính chất từ điển, có tính chất sách vở.']`, examples = `['A walking (living) dictionary.', 'A dictionary style.', …]`
- `lock` glosses = `['Khóa, ổ khóa.', …]` / `['Khóa (cửa tủ...)', …]` / `['Khóa được.', …]`

🟢 **Chứng cứ sống cho §Quyết định #5** *(không bóc IPA vào `reading`)*: cả bốn dòng **CÓ** `sounds[].ipa` — `dictionary` ⇒ `/ˈdɪk.ʃə.nə.ɹi/`, `lock` ⇒ `/lɒk/`, `/lɑk/` — nhưng **không dòng nào** mang tag `Pinyin`+`Mandarin`. `parse_line` vì vậy cho `reading = NULL` trên toàn bộ nguồn vai A, **không cần một dòng mã nào**.

#### Task 8 — Phát hiện khi viết test AC8: cặp đối lập dấu KHÔNG phải `điển`/`dien`

Test AC8 viết theo khuôn story lần đầu **ĐỎ**. Nguyên nhân không phải lỗi cài đặt:

`đ` *(U+0111 LATIN SMALL LETTER D WITH STROKE)* là một **CHỮ CÁI**, **không phải dấu phụ tổ hợp**. `remove_diacritics=2` bóc `ể → e` nhưng **để nguyên `đ`**. Đo thật trên `dict-core.db` dựng từ fixture:

| Truy vấn | `sense_fts` *(chính)* | `sense_fts_nd` *(phụ)* |
|---|---:|---:|
| `điển` | **2** | 2 |
| `dien` | 0 | 🔴 **0** |
| `đien` | **0** | ✅ **2** |
| `khóa` | 6 | 8 |
| `khoa` | 2 | 8 |

⇒ Cặp đối lập đúng là **`điển` / `đien`**. Đã đối chiếu nguồn của từng hit: cả bốn hit đều thuộc `viwiktionary-en | dictionary | en`, không phải nguồn tiếng Trung nào. Test vì vậy ràng buộc `s.code = 'viwiktionary-en' AND e.lang = 'en'` thay vì đếm toàn bảng — một hit từ nguồn ZH không nghiệm thu được mệnh đề *"NFR8 giữ hiệu lực trên dữ liệu TIẾNG ANH"*.

⚠️ `khóa`/`khoa` bị **loại làm ứng viên**: `khoa` cho 2 hit ở chỉ mục chính *(có nghĩa khác chứa "khoa" không dấu thật)* ⇒ không phải một cặp đối lập sạch.

#### Task 9 — Kiểm A bắt chính test vừa thêm

`npm run check:dict` **ĐỎ ở Kiểm A** sau Task 8: tên test `merged_headword_keeps_the_entry_lang_of_the_call` chứa token từ vựng hợp nhất, và Kiểm A quét toàn cây `src/**` ⇒ không phân biệt mã thật với tên test.

🔴 **Xử lý: đổi tên test → `two_lines_of_one_headword_keep_the_entry_lang_of_the_call`.** **Không** nới miễn trừ — thêm một miễn trừ cho một *tên test* là làm hỏng chính phép kiểm mà AD-19 dựa vào. Miễn trừ giữ nguyên **Kiểm A: 3 · Kiểm E: 1**, đúng bằng Task 1.

Sau khi đổi tên: **cả sáu** phép kiểm A–F **XANH**, `21 tệp .rs đã quét (sàn 21)`.

---

#### Task 10 — Lượt dựng THẬT, `--layer base`

**Lệnh đã chạy** *(nguyên văn)*:

```sh
cargo run --release --manifest-path tools/dict-build/Cargo.toml -- \
  --raw tools/dict-build/raw --out-dir tools/dict-build/out --layer base
```

**Bảng `SourceStats` đầy đủ — sáu nguồn:**

| nguồn | đọc | bỏ | entry | sense | example | citation |
|---|---:|---:|---:|---:|---:|---:|
| cvdict | 122.597 | 1 | 122.596 | 200.195 | 0 | 0 |
| cc-cedict | 124.758 | 0 | 124.758 | 199.615 | 0 | 0 |
| unihan | 49.870 | 0 | 49.870 | 23.285 | 0 | 0 |
| viwiktionary *(vai B)* | 415.115 | 413.517 | **1.598** | 2.242 | 536 | 0 |
| en-wiktionary | 306.358 | 131.681 | 174.677 | 255.372 | 89.939 | 0 |
| **viwiktionary-en** *(vai A)* | **401.101** | **282.062** | **119.039** | **190.543** | **27.396** | 0 |

**Lý do bỏ của nguồn mới** — cả hai đều là nhóm **có tên**, không phải lỗi đọc:

| Lý do | Số dòng |
|---|---:|
| `lang_code != en (filtered, expected — không phải lỗi đọc)` | **281.935** |
| `no usable glosses on any sense` | **127** |
| **Tổng bỏ** | **282.062** |

🟢 **AC2 — ba con số khớp mũi thăm dò CHÍNH XÁC, lệch 0,00%** *(ngưỡng cho phép 1%)*:

| Chỉ số | Mũi thăm dò | Ngưỡng 1% | **Đo thật** | Lệch |
|---|---:|---|---:|---|
| `entries` | 119.039 | 117.849 – 120.229 | **119.039** | **0,00%** ✅ |
| `senses` | 190.543 | 188.638 – 192.448 | **190.543** | **0,00%** ✅ |
| `examples` | 27.396 | 27.122 – 27.670 | **27.396** | **0,00%** ✅ |

🟢 Cả `lines_read` **401.101** lẫn hai số dòng-bỏ **281.935 / 127** cũng khớp §Số học của `SourceStats` từng con số — nghĩa là mô hình *"`lines_read` = ParseIssue + RawEntry đã gộp"* của story **đúng**, không phải trùng hợp.

**Tệp đầu ra:**

| | Giá trị |
|---|---|
| `dict-core.db` byte | **194.998.272** *(cũ 154.464.256, **+40.534.016**)* |
| SHA-256 | `2145c7aebb914fb9bf4def6a356bd5ad2fd73328c787de26e29e8abecb4f305a` |
| `journal_mode` sau khi đóng | `delete` |
| Tệp `-wal`/`-shm` trong thư mục ra | **0** ✅ |

**`dict_meta`:**

| key | value |
|---|---|
| `schema_version` | **1** — không đổi ✅ |
| `built_at` | `2026-08-04T23:53:16Z` — 🟢 **KHÔNG ĐỔI** ⇒ `raw/` không bị đụng *(§Bẫy 4 tránh được)* |
| `builder_version` | **`0.3.0`** ✅ |
| `layer` | `base` |

🟢 **§Bẫy 5 tránh được — hai tệp gỡ rời KHÔNG bị đụng.** `ls -lT` + SHA-256 trước/sau:

| Tệp | mtime trước | mtime **sau** | SHA-256 trước/sau |
|---|---|---|---|
| `dict-thieu-chuu.db` | `09:32:48` | **`09:32:48`** ✅ | `e9417c12…e43c9d5` → **không đổi** ✅ |
| `dict-vietphrase.db` | `09:33:21` | **`09:33:21`** ✅ | `9d304210…a78f735` → **không đổi** ✅ |
| `dict-core.db` | `09:32:46` | `11:19:11` *(dựng lại — đúng ý định)* | `741e1666…` → `2145c7ae…` |

⇒ Hai `sha256` trong `[[detachable]]` của manifest **vẫn đúng**, không được đụng.

**Ba phép nghiệm thu tay — SQL nguyên văn:**

**① AC3 — phân bố `(code, lang)`:**

```sql
SELECT s.code, e.lang, COUNT(*)
FROM dict_entry e JOIN dict_source s ON s.id = e.source_id
GROUP BY s.code, e.lang ORDER BY s.code, e.lang;
```

| code | lang | COUNT(\*) |
|---|---|---:|
| cc-cedict | zh | 124.758 |
| cvdict | zh | 122.596 |
| en-wiktionary | zh | 174.677 |
| unihan | zh | 49.870 |
| viwiktionary | zh | **1.598** |
| **viwiktionary-en** | **en** | **119.039** |

🟢 **AC3 ĐẠT — cả ba mệnh đề:**
- ✅ `viwiktionary-en | en | 119039` — 100% hàng mang `lang='en'`
- ✅ **Đối chứng âm 1:** **KHÔNG có hàng** `viwiktionary-en | zh` — §Bẫy 1 tránh được
- ✅ **Đối chứng âm 2:** vai B vẫn đúng **1.598** *(không đổi một mục)* và **không có hàng** `viwiktionary | en`
- ✅ Đúng **SÁU** hàng nguồn, không hơn không kém

**② FR34 — tra một từ tiếng Anh thật:**

```sql
SELECT s.code, e.headword, e.lang, e.reading, sn.pos, sn.pos_lang, sn.gloss
FROM dict_entry e JOIN dict_source s ON s.id=e.source_id JOIN dict_sense sn ON sn.entry_id=e.id
WHERE e.headword='dictionary';
```

| code | headword | lang | reading | pos | pos_lang | gloss |
|---|---|---|---|---|---|---|
| viwiktionary-en | dictionary | **en** | *(NULL)* | **Danh từ** | **vi** | Từ điển. |
| viwiktionary-en | dictionary | en | *(NULL)* | Danh từ | vi | Có tính chất từ điển, có tính chất sách vở. |
| viwiktionary-en | dictionary | en | *(NULL)* | **Động từ** | vi | (ngoại động từ) Tra cứu từ điển. |
| viwiktionary-en | dictionary | en | *(NULL)* | Động từ | vi | (ngoại động từ) Thêm vào từ điển. |

🟢 **FR34 nghiệm thu bằng dữ liệu THẬT**: nhãn từ loại **có**, nghĩa **tiếng Việt** có. Bốn nghĩa / hai từ loại gộp về **MỘT** `entry_id` *(FR30)*. `reading = NULL` ⇒ **§Quyết định #5 giữ đúng** — IPA không bị bóc.

**③ `char_idx`:**

```sql
SELECT COUNT(*) FROM char_idx;
SELECT COUNT(*) FROM char_idx c JOIN dict_entry e ON e.id=c.entry_id
JOIN dict_source s ON s.id=e.source_id WHERE s.code='viwiktionary-en';
```

| | Giá trị |
|---|---:|
| `char_idx` tổng | **1.341.179** |
| Do `viwiktionary-en` đóng góp | 🟢 **9** |

🟢 **Đúng CHÍNH XÁC con số dự báo của §Bẫy 9 / §Quyết định #6.** Đây là **bằng chứng đo được** rằng `AD-26` nhánh 2 *(chuỗi con 1–2 ký tự qua `char_idx`)* **không áp được** cho tiếng Anh — 9 cặp trên 119.039 đầu mục là **0,0076%**. Lý do Story 1.11b bị chặn chờ AD mới của Winston.

**④ AC4 — bốn trường giấy phép, hai vai phân biệt được:**

```sql
SELECT id, code, display_name, license_kind, license_id, source_version, source_url,
       length(license_text), attribution
FROM dict_source WHERE code LIKE 'viwiktionary%' ORDER BY id;
```

| Trường | `viwiktionary` *(vai B)* | `viwiktionary-en` *(vai A)* |
|---|---|---|
| `id` | 4 | **6** — chèn CUỐI, id 1–5 không dịch ✅ |
| `display_name` | Wiktionary tiếng Việt | **Wiktionary tiếng Việt (mục tiếng Anh)** ✅ khác |
| `license_kind` | open | **open** ✅ |
| `license_id` | CC-BY-SA-4.0 | **CC-BY-SA-4.0** ✅ |
| `length(license_text)` | 43.304 | **43.304** ✅ văn bản THẬT, y hệt vai B |
| `source_url` | kaikki.org/…/vi-extract.jsonl | **khác rỗng** ✅ |
| `source_version` | 2026-08-04 | **2026-08-04** — cùng dump ⇒ cùng giá trị, ĐÚNG |
| `attribution` | …qua Wiktextract/kaikki.org… | …**mục tiếng Anh**, qua Wiktextract/kaikki.org… ✅ |

🟢 **AC4 ĐẠT** — bốn trường đầy đủ, `license_text` là văn bản thật *(không giữ chỗ)*, hai hàng **phân biệt được bằng mắt** trên màn Attribution *(Story 10.4)*.

---

#### Task 12 — Bảng kế toán NFR6, mọi dòng là byte ĐO ĐƯỢC

| Dòng | Byte | Nguồn số |
|---|---:|---|
| Baseline `.dmg` không font/license | 2.334.696 | Story 1.9/1.10 — tái dùng, điều kiện §Quyết định #8 **đã kiểm** *(xem dưới)* |
| License trong bundle | 35.149 | tái dùng |
| Bộ font | 21.285.713 | `font-spike-results-2026-08-03.md:82` |
| **`dict-core.db` MỚI** | 🔴 **194.998.272** | **đo thật ở story này** *(cũ 154.464.256, **+40.534.016**)* |
| `dict-thieu-chuu.db` | 5.787.648 | Story 1.10 — không dựng lại, SHA-256 đã đối chiếu |
| `dict-vietphrase.db` | 160.083.968 | Story 1.10 — không dựng lại, SHA-256 đã đối chiếu |
| Hai lớp chưa dựng *(HVTĐTD, Cổ hán văn)* | `[----]` | **chưa đo — story nối tiếp**, không ước |
| WebView2 Runtime nhúng | *(dòng riêng)* | **không cộng vào tổng** — NFR6 sửa 2026-08-03 |
| **TỔNG payload sản phẩm hôm nay** | **384.525.446** | cộng bằng byte |
| **Trần NFR6** | **400.000.000** | `prd.md:834` *("sửa lần hai", nâng 2026-08-05)* |
| **Dư địa còn lại** | **15.474.554** | trần − tổng |

### 🟢 PHÁN QUYẾT NFR6: **ĐẠT** — với **BẢY** nguồn *(6 nền + 2 gỡ rời − không thiếu HVTĐTD và Cổ hán văn)*

- Tổng **384.525.446 byte** = **384,53 MB thập phân**, dưới trần **400.000.000**.
- **Dư địa 15.474.554 byte**, so với dự phóng **15.675.258** của `deferred-work.md:272` ⇒ đo thật **hẹp hơn dự phóng 200.704 byte** *(0,05% của trần — dự phóng chính xác tới 99,95%)*.
- ⚠️ **Dư địa này phải gánh CẢ HAI lớp chưa dựng.** 15,47 MB cho HVTĐTD + Cổ hán văn là **chật** — ghi thành cảnh báo ở `deferred-work.md`, không phải kết luận của story này.

**Không** sửa `Cargo.toml` · không `[profile.release]` · không bỏ chỉ mục · không bỏ nguồn · không subset font — và không cần, vì phán quyết là ĐẠT.

**Điều kiện §Quyết định #8 — kiểm bằng lệnh, không bằng trí nhớ:**

```sh
git diff --stat -- src-tauri/ src/ package.json   # ⇒ RỖNG
```

⇒ **0 dòng** đổi trong `src-tauri/**`, `src/**`, `package.json` ⇒ baseline `.dmg` **2.334.696** và license **35.149** tái dùng được. *(`src-tauri/resources/dict/README.md` sửa ở Task 13 **không** nằm trong `bundle.resources` — chỉ `resources/fonts/*` + `resources/license/*` — nên không vào `.dmg`.)*

### Completion Notes List

#### Đã giao

**Nguồn NỀN thứ SÁU `viwiktionary-en` đã có trong `dict-core.db` — 119.039 đầu mục tiếng Anh, 100% `lang='en'`.** FR34 lần đầu có dữ liệu để đứng lên. Không parser mới · không crate mới · không một dòng DDL nào đổi · không một dòng nào trong `src-tauri/**`.

| AC | Phán quyết | Chứng cứ |
|---|---|---|
| **AC1** — nguồn thứ sáu qua parser đã có | ✅ **ĐẠT** | `dict_source` đúng **6** hàng, đúng thứ tự; `viwiktionary` vai B không đổi một chữ |
| **AC2** — lệch ≤ 1% | ✅ **ĐẠT** | **0,00%** cả ba: entries/senses/examples = 119.039 / 190.543 / 27.396 **khớp tuyệt đối** |
| **AC3** — 100% `lang='en'` + hai đối chứng âm | ✅ **ĐẠT** | **0** hàng `viwiktionary-en\|zh`; vai B giữ **1.598** và **0** hàng `en` |
| **AC4** — bốn trường giấy phép | ✅ **ĐẠT** | `license_text` 43.304 ký tự thật; `display_name`+`attribution` phân biệt được |
| **AC5** — vào `dict-core.db` | ✅ **ĐẠT** | nguồn nền thứ sáu; dựng lại + điền lại `[base]` |
| **AC6** — NFR6 | ✅ **ĐẠT** | **384.525.446 / 400.000.000**, dư **15.474.554** |
| **AC7** — hai cổng `.mjs` | ✅ **ĐẠT** | cả hai XANH; `RS_FILE_FLOOR = 21`; `[[detachable]]` vẫn đúng hai |
| **AC8** — NFR8 trên dữ liệu EN | ✅ **ĐẠT** | chỉ mục chính phân biệt dấu trên gloss của `dictionary` |

**Test:** `tools/dict-build` **110 → 125** *(+15)*. `src-tauri` **62 → 62** *(không đụng một dòng)*. Cổng: `check:deps` · `check:dict` *(A–F)* · `check:dict-manifest` · `check:i18n` — **tất cả XANH**.

#### 🔴 Ba việc cho Ice

**① Lệnh chép-dán — tải `dict-core.db` MỚI lên release `dict-v1`**

⚠️ **`dict-core.db` đã đổi LẦN HAI.** Nếu Ice **đã** tải bản cũ lên, dùng `--clobber`; ba tệp phải thuộc **một thế hệ dữ liệu**.

```sh
cd /Users/hoangnam/LocalSites/addon/AuraTranslate

# Nếu release CHƯA tồn tại — tạo và tải cả ba tệp một lượt:
gh release create dict-v1 \
  tools/dict-build/out/dict-core.db \
  tools/dict-build/out/dict-thieu-chuu.db \
  tools/dict-build/out/dict-vietphrase.db \
  --title "Dữ liệu từ điển v1" \
  --notes "6 nguồn nền + 2 lớp gỡ rời. Checksum đối chiếu ở dict-manifest.toml."

# Nếu release ĐÃ tồn tại và đã có bản cũ — ghi đè CẢ BA:
gh release upload dict-v1 \
  tools/dict-build/out/dict-core.db \
  tools/dict-build/out/dict-thieu-chuu.db \
  tools/dict-build/out/dict-vietphrase.db \
  --clobber

# Đối chiếu ba checksum sau khi tải lên:
shasum -a 256 tools/dict-build/out/*.db
```

Ba giá trị phải khớp `dict-manifest.toml`:

| Tệp | SHA-256 | Byte |
|---|---|---:|
| `dict-core.db` | `2145c7aebb914fb9bf4def6a356bd5ad2fd73328c787de26e29e8abecb4f305a` | 194.998.272 |
| `dict-thieu-chuu.db` | `e9417c12f5adc256e8cc7d49c42d09c3378fb9082fc6fd678beadf7ebe43c9d5` | 5.787.648 |
| `dict-vietphrase.db` | `9d304210c16cd65abe9f5ed529d1b00542c3aa19cfe14d3eb6bfcc8a1a78f735` | 160.083.968 |

🟢 **Checksum tái lập được**: cùng cây `raw/` ⇒ cùng tệp byte-for-byte *(bản vá `built_at` của Story 1.10)*. `built_at` giữ nguyên `2026-08-04T23:53:16Z` qua lượt dựng này — bằng chứng `raw/` không bị đụng.

**② ⚠️ Ba lệch giữa TÀI LIỆU và MÃ — dev không sửa *(tiền lệ quyết định #3 của Ice ở Story 1.3)***

| # | Lệch | Ở đâu | Số ĐÚNG |
|---|---|---|---|
| 1 | *"141.407 mục đang bị vứt bỏ"* cho vai A | `prd.md` §8.2 | **119.039** đầu mục *(141.407 đếm DÒNG THÔ, 119.039 là đầu mục sau khi gộp headword — hai thứ khác nhau)*. Đã biết trước, ghi ở §Câu hỏi cho Ice #2. |
| 2 | *"Test `tools/dict-build`: **88**"* | §Trạng thái repo hiện tại của **chính story này** | **110** trước story, **125** sau. Bản ghi Story 1.10 lỗi thời — hai commit `a3ed5cd`/`ed8ce52` đổ thêm test vào sau. |
| 3 | *"`all_sources_have_a_real_non_unknown_source_version` chạy tự động qua nguồn mới — không sửa"* | §Task 8 của **chính story này** | Test đó có `assert_eq!(rows.len(), 5)` **viết cứng** ⇒ **buộc** phải sửa 5 → 6. Mệnh đề nó khoá không đổi. |

**③ Ba mục `correct-course` còn mở — không thuộc story này nhưng chặn phần sau của Epic 1**

- 🔴 **AD mới cho đường tra cứu tiếng Anh** *(Winston)* — **VẪN CHẶN 1.11b**. Story này vừa cấp **bằng chứng đo được**: `char_idx` của vai A = **9** cặp / 119.039 đầu mục = **0,0076%** ⇒ `AD-26` nhánh 2 không áp được cho tiếng Anh, đây là dữ kiện chứ không còn là suy đoán.
- 🟡 **Panel Lookup có hình dạng cho mục tiếng Anh chưa** *(Sally)* — mục tiếng Anh nay **CÓ THẬT**: từ loại + nghĩa tiếng Việt + ví dụ, không Hán Việt, không âm đọc *(`reading` là NULL trên toàn bộ 119.039 mục)*.
- 🟡 **SPEC có bản sao FR34 cần đồng bộ không** *(`bmad-spec`)*.

#### Ghi chú kỹ thuật đáng giữ

**🔴 `đ` (U+0111) KHÔNG bị `remove_diacritics` bóc.** Nó là một **chữ cái**, không phải dấu phụ tổ hợp — `remove_diacritics=2` cho `ể → e` nhưng để nguyên `đ`. Đo thật: `'dien'` cho **0** hit ở **cả hai** chỉ mục; cặp đối lập đúng là `điển` *(chính: 2 · phụ: 2)* / `đien` *(chính: **0** · phụ: **2**)*. Bất kỳ lượt viết test FTS tiếng Việt nào sau này cần biết điều này, nếu không sẽ kết luận nhầm *"chỉ mục phụ hỏng"*.

**Kiểm A quét cả tên test, không chỉ mã.** Một test đặt tên có từ vựng hợp nhất tiếng Anh làm `check:dict` ĐỎ. Đã xử lý bằng **đổi tên test**, **không** nới miễn trừ — miễn trừ giữ nguyên **3 / 1**. Nới miễn trừ cho một *tên test* là làm hỏng chính phép kiểm AD-19 dựa vào.

**§Quyết định #5 giữ nguyên và có chứng cứ:** cả 119.039 mục tiếng Anh cho `reading = NULL`. `sounds[].ipa` **có** trong dữ liệu *(`dictionary` ⇒ `/ˈdɪk.ʃə.nə.ɹi/`)* nhưng không tag `Pinyin`+`Mandarin`, nên `parse_line` bỏ qua — **không cần một dòng mã nào**. Nếu 1.11b/1.17 cần phiên âm cho tiếng Anh thì đó là một quyết định mới, cần một cột `reading_kind` hoặc tương đương.

**⚠️ §Bẫy 8 của Story 1.10 còn treo, nay áp cho story này:** Task 9 *(cổng)* + Task 10 *(tệp `.db`)* + Task 11 *(manifest)* phải vào **CÙNG MỘT commit**. `dict-manifest.toml` hiện công bố `2145c7ae…` cho một tệp nằm ngoài git — tách thành hai commit làm `check:dict-manifest` không đỏ *(cổng cố ý không đọc `.db`)* nhưng để lại một checksum không ai đối chiếu được.

### File List

**MỚI (1):**

| Tệp |
|---|
| `tools/dict-build/src/sources/viwiktionary_en.rs` |

**SỬA (11):**

| Tệp | Thay đổi |
|---|---|
| `tools/dict-build/src/sources/wiktextract_common.rs` | tham số `entry_lang` *(bắt buộc)* · doc-comment ba tham số · +3 test |
| `tools/dict-build/src/sources/viwiktionary.rs` | truyền `"zh"` · doc-comment vai B, trỏ sang vai A |
| `tools/dict-build/src/sources/en_wiktionary.rs` | truyền `"zh"` |
| `tools/dict-build/src/sources/mod.rs` | `+ pub mod viwiktionary_en;` · doc-comment 6 nền + 2 gỡ rời |
| `tools/dict-build/src/sources_meta.rs` | `VIWIKTIONARY_EN` · `BASE_ALL[;6]` · +3 test, đổi tên 1 test |
| `tools/dict-build/src/build.rs` | khối chèn thứ sáu trong `run_base` · doc-comment "6 nguồn / 5 thư mục" |
| `tools/dict-build/Cargo.toml` | `version 0.2.0 → 0.3.0` *(không đổi gì khác)* |
| `tools/dict-build/Cargo.lock` | 🔄 **hệ quả tự động** của dòng trên — diff là **ĐÚNG MỘT dòng** `version = "0.2.0"` → `"0.3.0"`; **0** crate thêm/bớt/đổi phiên bản |
| `tools/dict-build/tests/parse.rs` | +6 test mới · đổi tên `all_five…`→`all_six…` · `rows.len()` 5→6 |
| `tools/dict-build/tests/fixtures/raw/viwiktionary/vi-extract.jsonl` | **+4 dòng** trích thật *(19 → 23)*, 0 dòng bị xoá |
| `tools/dict-build/README.md` | §Một tệp thô hai vai · bảng nguồn 5→6 · cây thư mục · version · bảng giấy phép 7→8 |
| `scripts/check-dict-build.mjs` | `RS_FILE_FLOOR 20 → 21` |
| `dict-manifest.toml` | `[base].sha256` + `[base].source_version` *(không đụng `[[detachable]]`)* |
| `src-tauri/resources/dict/README.md` | `dict-core.db` mô tả **sáu** nguồn + cảnh báo checksum đổi lần hai |
| `_bmad-output/implementation-artifacts/deferred-work.md` | `:272` số NFR6 thật · `:273` char_idx thật · **+1 mục mới** cho 1.11b/1.13 |

**BẢN GHI quy trình (2):**

| Tệp | Thay đổi |
|---|---|
| `_bmad-output/implementation-artifacts/1-10b-dung-du-lieu-tu-dien-tieng-anh.md` | tệp story này — checkbox · Dev Agent Record · File List · Change Log · Status |
| `_bmad-output/implementation-artifacts/sprint-status.yaml` | `1-10b…: ready-for-dev → in-progress → review` · `last_updated` |

**ARTIFACT dựng lại (ngoài git, `*.gitignore`):** `tools/dict-build/out/dict-core.db`

⚠️ `_bmad-output/implementation-artifacts/1-10-dong-goi-bon-lop-go-roi-thanh-file-doc-lap.md` và `_bmad-output/planning-artifacts/**` xuất hiện `M` trong `git status` — chúng **đã bị sửa TRƯỚC khi story này bắt đầu** *(có trong ảnh chụp `git status` ở Task 1)*. **Story này không đụng một dòng nào của chúng.**

**KHÔNG đụng:** toàn bộ `src-tauri/src|tests|Cargo.toml|tauri.conf.json` *(`git diff --stat` **rỗng**)* · `tools/dict-build/src/{schema,insert,model,char_idx,finalize,licenses,main,lib}.rs` · `tools/dict-build/src/sources/{cvdict,cc_cedict,unihan,thieu_chuu,vietphrase,cedict_common}.rs` · `tools/dict-build/tests/{schema,layers}.rs` · `scripts/check-dict-manifest.mjs` · toàn bộ `_bmad-output/planning-artifacts/**` · `docs/dics/**` · tệp story 1.9 và 1.10

---

## Câu hỏi cho Ice *(không chặn thi công — trả lời sau khi có số thật)*

1. **Nếu NFR6 VƯỢT lần hai** *(tổng > 400.000.000 byte)* — `prd.md` §11 [A2] đã nói trước đường ra **không còn là nâng trần**: ở mức đó phải cân nhắc lại chính lời hứa *"không tải thêm sau khi cài"*, tức chạm **NFR7** và **NFR12**. Dev sẽ **DỪNG** và báo số, không tự quyết.
2. **`prd.md` §8.2 ghi *"141.407 mục đang bị vứt bỏ"*** cho vai A, trong khi mũi thăm dò cùng tài liệu đo **119.039** đầu mục. Hai con số đo hai thứ khác nhau *(dòng thô vs đầu mục sau khi gộp)*. Story này nghiệm thu theo **119.039**. Cần Ice sửa một dòng trong PRD để lượt rà sau không đọc nhầm — **dev không sửa PRD**.
3. **Ba mục còn mở của `correct-course`** không thuộc story này nhưng chặn phần sau của Epic 1: 🔴 **AD mới cho đường tra cứu tiếng Anh** *(Winston — **CHẶN 1.11b**)* · 🟡 **Panel Lookup có hình dạng cho mục tiếng Anh chưa** *(Sally)* · 🟡 **SPEC có bản sao FR34 cần đồng bộ không**.
4. **Release `dict-v1` vẫn CHƯA TỒN TẠI.** Sau story này `dict-core.db` đổi lần thứ hai — nếu Ice đã tải bản cũ lên, cần `--clobber` **cả ba** tệp để ba checksum thuộc **một thế hệ dữ liệu**.

---

## Change Log

| Ngày | Thay đổi |
|---|---|
| 2026-08-05 | **THI CÔNG XONG — 13/13 task, 8/8 AC ĐẠT.** Nguồn nền thứ sáu `viwiktionary-en` vào `dict-core.db`: **119.039** đầu mục tiếng Anh, **100% `lang='en'`**, **0** hàng `lang='zh'` *(đối chứng âm)*; vai B giữ nguyên **1.598** và **0** hàng `en` *(đối chứng âm chiều ngược)*. **AC2 lệch 0,00%** cả ba chỉ số. Hạt nhân: `wiktextract_common::parse_line` bỏ hằng `lang: "zh"`, nhận `entry_lang` **bắt buộc** *(§Bẫy 1)*. Dựng bằng **`--layer base`** ⇒ hai tệp gỡ rời không bị đụng *(mtime + SHA-256 đối chiếu trước/sau — §Bẫy 5)*. `dict-core.db` `154.464.256` → **`194.998.272`** byte, sha `2145c7ae…`; `built_at` không đổi ⇒ `raw/` không bị đụng *(§Bẫy 4)*. **NFR6: ĐẠT** — **384.525.446 / 400.000.000**, dư **15.474.554**. Test `tools/dict-build` **110 → 125**; `src-tauri` **62 → 62** *(0 dòng đụng)*; bốn cổng XANH, `RS_FILE_FLOOR` 20 → 21, miễn trừ giữ nguyên **3/1**. 🔴 Ba lệch tài liệu↔mã ghi ở §Completion Notes ② *(dev không sửa PRD)*. Phát hiện đáng giữ: **`đ` (U+0111) không bị `remove_diacritics` bóc** — cặp đối lập AC8 là `điển`/`đien`, không phải `điển`/`dien`. |
| 2026-08-05 | Story tạo từ `epics.md` §Story 1.10b + `sprint-change-proposal-2026-08-05.md` *(Ice duyệt cùng ngày)*. **AC5 CHỐT: lớp tiếng Anh vào `dict-core.db`** *(nguồn nền thứ sáu)* — bốn lý do ở §Quyết định #1. Phát hiện chặn khi rà mã: `wiktextract_common::parse_line:143` **viết cứng `lang: "zh"`** ⇒ §Bẫy 1 + Task 2 + đối chứng âm bắt buộc ở AC3. Đo thật nguồn thô: **415.254 dòng / 273.075.442 byte**; làm rõ `SourceStats.lines_read` không phải số dòng tệp. Xác nhận fixture đã có **3 mục `lang_code:"en"`** dùng được. Trần NFR6 xác nhận **400.000.000** *(không còn 200.000.000 như bản ghi Story 1.10)*. |
