---
baseline_commit: d9bc252e3631c2c6a39f6d631214b559ac6ba713
baseline_note: 'Cây làm việc tại d9bc252 CỘNG toàn bộ thay đổi CHƯA COMMIT của Story 1.9 (tools/dict-build/, hai cổng .mjs, dict-manifest.toml, tauri.conf.json…) — xem §Trạng thái repo hiện tại'
---

# Story 1.10: Đóng gói bốn lớp gỡ rời thành file độc lập

Status: done

<!-- Note: Validation is optional. Run validate-create-story for quality check before dev-story. -->

**Covers:** FR27 · FR36 · FR38 (nửa dữ liệu) · FR112 · NFR6 (đóng phần Story 1.9 để mở) · AD-7 · AD-10 · AD-19 · AD-25
**Epic:** 1 — Nền móng ứng dụng & Tra cứu ngoại tuyến tức thì

> 🔴 **PHẠM VI THU HẸP — Ice chốt 2026-08-05: story này giao HAI lớp, không phải bốn.**
>
> | Lớp | Trạng thái | Story |
> |---|---|---|
> | **Thiều Chửu** | ✅ nguồn thô đã có, đã kiểm chứng | **1.10 — story này** |
> | **VietPhrase** | ✅ nguồn thô đã có, đã kiểm chứng | **1.10 — story này** |
> | **HVTĐTD** | 🔴 ⛔ không tồn tại bản tải hàng loạt công khai; phải xin trực tiếp tác giả | story nối tiếp |
> | **Cổ hán văn** | 🔴 ⛔ chưa có nguồn, và cần quyết lại *"nó là lớp gì"* trước khi đi tìm tệp | story nối tiếp |
>
> Tên story giữ nguyên chữ của `epics.md` *(tài liệu quy hoạch, dev ⛔ không sửa)*. **Hai lớp còn lại ⛔ không bị bỏ** — chúng được ghi thành mục bàn giao đích danh trong `deferred-work.md` (Task 12). Lý do thu hẹp thay vì chờ: mọi thứ tốn công của story này — CLI đa lớp, `finalize` dùng chung, parity lược đồ, hai cổng, kế toán NFR6 — **dùng chung cho cả bốn lớp**, nên dựng nó trên hai nguồn có thật rẻ hơn hẳn việc chờ, và lớp thứ ba chỉ còn là *"thêm một dòng vào bảng phân phối"* *(§Quyết định #2)*.

> 🔴 **Story này biến một rủi ro pháp lý thành một quyết định đóng gói.** VietPhrase **không xác định được tác giả** (R6); Thiều Chửu là **phạm vi công cộng** nhưng PRD §8.6 đã cảnh báo *"bản số hoá có thể kèm tuyên bố quyền riêng"* (R7, giả định `[A3]`). AD-10 nói cách sống chung: mỗi lớp **một file `.db` riêng**, gỡ một lớp = **xoá một file**, ⛔ không đổi một dòng mã.
>
> 🟢 **Tin tốt đã kiểm chứng 2026-08-05:** bản Thiều Chửu được chọn đến từ kho **CC0 1.0 Universal**, đối chiếu **byte-for-byte** bằng SHA-256 ⇒ **R7 và `[A3]` không còn áp cho bản này**. Chi tiết: §Thông tin kỹ thuật.
>
> 🔴 **Và đây là chỗ NFR6 phải kết luận.** Story 1.9 phán quyết **CHƯA KẾT LUẬN ĐƯỢC** đúng chữ và giao lại **dư địa 21.507.450 byte**. ⚠️ **Ước tính có cơ sở nói con số này sẽ VƯỢT** — xem §Quyết định #7. Đó là một kết quả hợp lệ; ⛔ dev không tự cắt gì.
>
> ⚠️ **Story này KHÔNG viết một dòng mã tra cứu nào, và KHÔNG chạm `src-tauri/`.** Đường tra cứu ba nhánh là **1.11**; cổng `DictionarySource` + phép thử *"xoá file rồi chạy lại bộ test tra cứu"* của AD-10 là **1.13**; bật/tắt nguồn là **1.19**; màn Attribution là **10.4**; đóng gói vào bản phát hành là **10.1**.

---

## Story

As a chủ dự án,
I want gỡ một nguồn dữ liệu khỏi bản phát hành chỉ bằng cách xoá một file,
So that chính sách gỡ bỏ thực thi được mà không đổi một dòng mã.

---

## Acceptance Criteria

### AC1 — Mỗi lớp gỡ rời là MỘT file `.db` độc lập, ⛔ không gộp vào `dict-core.db`

**Given** Thiều Chửu và VietPhrase *(hai lớp trong phạm vi đã chốt)*
**When** `tools/dict-build` chạy
**Then** mỗi nguồn sinh ra **một file `.db` độc lập**, không gộp vào `dict-core.db`

*Đạt nghĩa là* sau một lượt chạy, thư mục đầu ra có **đúng ba** tệp `.db`:

| Tệp | Nội dung `dict_source` |
|---|---|
| `dict-core.db` | đúng **5** hàng — `cvdict` · `cc-cedict` · `unihan` · `viwiktionary` · `en-wiktionary` |
| `dict-thieu-chuu.db` | đúng **1** hàng — `thieu-chuu` |
| `dict-vietphrase.db` | đúng **1** hàng — `vietphrase` |

⛔ **Hai điều kiện âm bắt buộc**, vì một trong hai vi phạm vẫn cho lượt build XANH:

1. `dict-core.db` chứa **0** hàng `dict_source` có `code` thuộc mã lớp gỡ rời.
2. Mỗi tệp lớp gỡ rời chứa **0** hàng của năm mã lớp nền, và **0** hàng của lớp gỡ rời còn lại.

### AC2 — Mỗi file tự mang metadata giấy phép và ghi công của chính nó

**Given** mỗi file `.db` lớp gỡ rời
**When** mở
**Then** nó tự mang metadata giấy phép và ghi công của chính nó

*Đạt nghĩa là* trong **mỗi** tệp lớp gỡ rời, hàng `dict_source` duy nhất có **cả bốn trường khác rỗng**: `license_kind` · `license_text` · `attribution` · `source_url`; và `license_text` là **văn bản thật**, ⛔ không phải chuỗi giữ chỗ, ⛔ không phải bản tóm tắt tự viết.

🔴 Đây là điều kiện của AD-10: *"gỡ một lớp cũng gỡ luôn ghi công của nó, không để lại ghi công mồ côi khi thực thi FR112."* Ghi công sống **trong tệp**, ⛔ không ở một bảng tra cứu nào trong `src-tauri`.

🔴 **`attribution` của Thiều Chửu BẮT BUỘC nêu tên Thiều Chửu (Nguyễn Hữu Kha, 1902–1954).** Quyền nhân thân được bảo hộ **vô thời hạn** kể cả khi tác phẩm đã vào phạm vi công cộng — đây là **nghĩa vụ pháp lý**, không phải phép lịch sự *(`technical-…-research-2026-08-02.md:446`)*.

### AC3 — Trường giấy phép biểu diễn được cả giấy phép mở lẫn phép riêng của tác giả

**Given** trường giấy phép trong metadata
**When** mô hình hoá
**Then** biểu diễn được **cả** giấy phép mở **lẫn** phép sử dụng riêng do tác giả cấp
**And** không nguồn nào bị ép vào một enum các giấy phép mở

*Đạt nghĩa là* hai hàng mang đúng hai hình dạng dưới đây, ⛔ không hàng nào bị gán nhãn `open` cho tiện:

| `code` | `license_kind` | `license_id` | Vì sao |
|---|---|---|---|
| `thieu-chuu` | `public-domain` | `CC0-1.0` | Tác phẩm gốc hết hạn bảo hộ; **bản số hoá phát hành CC0** *(đã kiểm chứng — §Thông tin kỹ thuật)* |
| `vietphrase` | `unknown` | `NULL` | PRD §8.2: *"❓ Không xác định được tác giả"* |

🔴 ⛔ `vietphrase` **không** phải `public-domain`. *Không biết* và *không có bản quyền* là hai điều khác nhau, và gán nhầm là đúng thứ AD-10 cảnh báo: *"mô hình hoá trường này thành enum các giấy phép mở sẽ khiến nó bị gán nhãn sai ngay trên màn hình Attribution."*

🔴 **Cột `license_kind` PHẢI giữ nguyên kiểu `TEXT` (chuỗi mở)** — nó đã đúng từ Story 1.9. AC này ⛔ **không** đòi đổi lược đồ; nó đòi **dùng đúng** cột đã có, **và** giữ khả năng nhận `author-grant` của HVTĐTD ở story nối tiếp **mà không đổi lược đồ** *(§Bẫy 1)*.

### AC4 — Runtime đọc một nguồn bất kỳ qua CÙNG một đường, ⛔ không mã riêng cho từng nguồn

**Given** runtime đọc một nguồn bất kỳ
**When** thực hiện
**Then** không có mã riêng cho từng nguồn — mọi nguồn đi qua cùng một đường

*Đạt nghĩa là* — vì đường tra cứu chưa tồn tại (1.11/1.13) — **AC này nghiệm thu ở tầng CẤU TRÚC, và phép nghiệm thu đó mạnh hơn một test hành vi**:

```sql
SELECT type, name, tbl_name, sql FROM sqlite_master ORDER BY type, name;
```

Kết quả của **cả ba** tệp phải **giống nhau từng ký tự**. Cộng thêm: `PRAGMA user_version` bằng nhau ở cả ba tệp.

> **Vì sao đây là AC4 chứ không phải một phép kiểm phụ:** nếu một tệp thiếu `sense_fts_nd`, đường đọc chế độ khoan dung (FR9) phải hỏi *"tệp này có bảng đó không?"* trước mỗi truy vấn — và câu hỏi đó **chính là** mã riêng cho từng nguồn. Lược đồ đồng nhất làm AD-10 thành hệ quả chứ không phải nỗ lực.

⛔ Không so `dict_meta` giữa các tệp — `built_at` khác nhau theo thiết kế. So **`sqlite_master`**, không so nội dung.

### AC5 — `dict-manifest.toml` có một mục cho từng file, kèm URL và SHA-256

**Given** `dict-manifest.toml`
**When** kiểm
**Then** có một mục cho từng file `.db` kèm URL và SHA-256

*Đạt nghĩa là* **hai** vế, cả hai bắt buộc:

1. **Dữ liệu:** hai khối `[[detachable]]`, mỗi khối đủ **bốn** trường `name` · `url` · `sha256` · `source_version`, giá trị **thật** từ chính lượt build. ⛔ Không giá trị giả *"cho có"* — `dict-manifest.toml:16` đã cấm đúng chữ.
2. **Cổng:** `check-dict-manifest.mjs` **đòi đủ hai mục với đúng hai `name`**, ⛔ không dư không thiếu. Hôm nay nó chấp nhận `0 mục` *(dòng 230)* — để nguyên là để hợp đồng tự vô hiệu hoá đúng lúc nó bắt đầu có việc.

⚠️ Cổng đòi **đúng hai** *(⛔ không phải "≥ 1")*. Khi story nối tiếp thêm lớp thứ ba, con số đó đổi cùng lúc với dữ liệu — đó là điều kiện để cổng bắt được một lớp **bị rơi mất** chứ không chỉ một lớp bị điền sai.

### AC6 — Đóng phép đối chiếu NFR6 mà Story 1.9 để mở *(nhận bàn giao, không phải AC gốc của epics)*

> **Nguồn:** §Quyết định của Ice #1 của Story 1.9 — *"giao **dư địa còn lại bằng byte** cho Story 1.10 trừ tiếp"*; Completion Note #6 ghi **21.507.450 byte**.

**Given** hai tệp lớp gỡ rời đã dựng thật
**When** cộng vào bảng kế toán của Story 1.9
**Then** phán quyết **ĐẠT** hoặc **VƯỢT** trần **200.000.000 byte**, quy về **byte** trước rồi mới đổi sang MB thập phân
**And** nếu **VƯỢT**, đó là quyết định **tầng PRD** — ⛔ **không** tự bỏ một nguồn, ⛔ **không** tự bỏ chỉ mục `sense_fts_nd` của một lớp, ⛔ **không** tự sửa `[profile.release]` hay hai khoản `deferred-work.md:75`

*Đạt nghĩa là* một bảng trong §Debug Log References, mỗi dòng một số byte đo được:

| Dòng | Nguồn số |
|---|---|
| Baseline `.dmg` không font/license | **2.334.696** — Story 1.9, tái dùng *(điều kiện: §Quyết định #8)* |
| License trong bundle | **35.149** — Story 1.9, tái dùng |
| Bộ font | **21.285.713** — `font-spike-results-2026-08-03.md:82` |
| `dict-core.db` | **154.836.992** — Story 1.9, ⛔ không dựng lại *(§Quyết định #8)* |
| `dict-thieu-chuu.db` | đo thật ở story này |
| `dict-vietphrase.db` | đo thật ở story này |
| Hai lớp chưa dựng *(HVTĐTD, Cổ hán văn)* | `[----] chưa đo — story nối tiếp` — ⛔ không ước, ⛔ không bỏ dòng |
| WebView2 Runtime nhúng | **dòng riêng**, ⛔ không cộng vào tổng *(NFR6 sửa 2026-08-03)* |
| **Tổng payload sản phẩm hôm nay** | cộng bằng byte |
| Đối chiếu trần 200.000.000 byte | **ĐẠT** / **VƯỢT** |

⚠️ Vì hai lớp còn để trống, một kết quả **ĐẠT** hôm nay chỉ là *"đạt với hai lớp"* — ghi đúng chữ đó, ⛔ không viết *"NFR6 đã đóng"*. Còn **VƯỢT** thì là kết luận **cuối cùng và đủ**: thêm lớp chỉ làm nó vượt xa hơn.

⚠️ **150.000.000 byte không phải điều kiện đạt** — nó là mốc kỳ vọng. Trần là **200.000.000**. Tổng hôm nay đã là 178.492.550, tức **đã vượt mốc kỳ vọng từ Story 1.9** và điều đó hợp lệ.

---

## Tasks / Subtasks

- [x] **Task 0 — Đưa hai nguồn thô vào đúng chỗ và xác nhận bằng checksum** (chặn Task 3–6)
  - [x] `mkdir -p tools/dict-build/raw/{thieu_chuu,vietphrase}`
  - [x] Chép **đúng một** tệp Thiều Chửu: `docs/dics/Thieu chuu/TudienThienChuu.txt` → `tools/dict-build/raw/thieu_chuu/TudienThienChuu.txt`. ⛔ **Không** chép `.tab` *(thiếu cột âm Hán Việt)*, ⛔ không chép `.dict.dz`/`.dsl.dz`/`.mobi`/`.html`/`.idx`/`.opf`/`-Inflections.txt` *(bản dẫn xuất từ chính `.txt` này)*.
  - [x] Chép VietPhrase: `docs/dics/VietPhrase.txt` → `tools/dict-build/raw/vietphrase/VietPhrase.txt`. 🟢 **Tệp này ĐÃ là UTF-8 và đã bỏ BOM** *(Ice chuyển 2026-08-05)* — ⛔ **không chạy `iconv` lại**, chuyển mã hai lần làm hỏng tệp.
  - [x] Ghi lệnh chép-dán vào `tools/dict-build/README.md`, **kèm bước `iconv` cho trường hợp tải lại từ kho gốc** *(kho gốc phát hành UTF-16LE — §Bẫy 5)*. Đúng tiền lệ `Unihan.zip` *(§Quyết định #6 của Story 1.9)*.
  - [x] 🔴 **Đối chiếu SHA-256 nguồn** trước khi tin bất cứ số nào — bảng ở §Thông tin kỹ thuật. Lệch ⇒ **DỪNG**, tệp không phải bản đã khảo sát. *(Cả hai khớp byte-for-byte)*
  - [x] Ghi vào §Debug Log References: số dòng · số byte · **20 dòng đầu nguyên văn** của mỗi tệp, sau khi đã đặt đúng chỗ.
  - [x] ⛔ **Không đụng `docs/dics/_khong-dung/`.** Đọc `docs/dics/_khong-dung/README.md` trước nếu định lấy gì ở đó ra — nó chứa **Trần Văn Chánh** *(PRD đã loại vì còn bản quyền)* và một nguồn thứ năm chưa được PRD nhận. *(Không lấy gì từ đó, không cần đọc)*

- [x] **Task 1 — Đường cơ sở: chạy sáu lệnh, ghi số vào §Debug Log References** (không AC)
  - [x] `npm run build` *(bắt buộc trước `cargo test` — `generate_context!` nhúng `dist/` lúc biên dịch)*
  - [x] `cargo test --locked --manifest-path src-tauri/Cargo.toml` · `cargo test --manifest-path tools/dict-build/Cargo.toml`
  - [x] `npm run check:deps` · `check:dict` · `check:dict-manifest` · `check:i18n`
  - [x] Ghi lại: số `.rs` dưới `tools/dict-build/src/**` *(chứng cứ cho sàn Kiểm C mới)* · tổng test Rust hai cây · số crate `check-deps.mjs` đếm được *(phải **không đổi** sau story)*
  - [x] ⛔ Không sửa gì ở task này. Một lệnh đỏ sẵn thì **dừng và báo**.

- [x] **Task 2 — Tách `sources_meta` thành hai danh sách có khoá riêng** (AC1, AC2, AC3)
  - [x] `BASE_ALL: [&SourceMeta; 5]` — đổi tên từ `ALL`, ⛔ giữ nguyên nội dung.
  - [x] `DETACHABLE_ALL: [&SourceMeta; 2]` — `THIEU_CHUU` · `VIETPHRASE`.
  - [x] 🔴 **Giữ NGUYÊN test `exactly_five_sources_with_the_epics_md_codes`**, chỉ đổi `ALL` → `BASE_ALL`. ⛔ **Không** gộp hai danh sách rồi `assert_eq!(7)` — mục đích của test là khoá *"lớp gỡ rời KHÔNG nằm trong `dict-core.db`"* *(§Bẫy 4)*.
  - [x] Thêm `exactly_two_detachable_sources_in_scope_today` — khoá `["thieu-chuu","vietphrase"]`, và **doc-comment nêu đích danh HVTĐTD + Cổ hán văn thuộc story nối tiếp**, đúng khuôn doc-comment mà Story 1.9 đã dùng để chỉ sang story này.
  - [x] Thêm `base_and_detachable_code_sets_are_disjoint`.
  - [x] `LicenseRef` nhận thêm hai biến thể. ⛔ Giữ `enum` **đóng** và `match` **toàn vẹn** — ⛔ không thêm nhánh `_ =>`, đó chính là lỗi `unreachable!()` mà lượt review Story 1.9 đã gỡ.
  - [x] `license_kind`/`license_id` theo đúng bảng AC3. `attribution` nêu **tên tác giả** (Thiều Chửu) và **trạng thái xuất xứ** (VietPhrase: không xác định được tác giả). ⛔ Không thêm cột mới *(§Bẫy 1)*.

- [x] **Task 3 — Văn bản giấy phép cho hai lớp** (AC2, AC3)
  - [x] `tools/dict-build/assets/licenses/CC0-1.0.txt` — **tải nguyên văn** từ `creativecommons.org/publicdomain/zero/1.0/legalcode.txt`, ⛔ không tự tóm tắt. Đúng khuôn ba tệp giấy phép Story 1.9 đã làm.
  - [x] Tệp tuyên bố cho **Thiều Chửu**: phạm vi công cộng của tác phẩm gốc (1942, Nguyễn Hữu Kha †1954) + bản số hoá theo **CC0 1.0** + 🔴 **nghĩa vụ ghi công vô thời hạn theo quyền nhân thân**.
  - [x] Tệp tuyên bố cho **VietPhrase**: *"dữ liệu cộng đồng, không xác định được tác giả; đóng gói theo FR36 + chính sách gỡ bỏ FR112"*.
  - [x] `include_str!` như `licenses.rs` đang làm — ⛔ không hằng chuỗi dài nhúng giữa mã *(§Quyết định #4)*.

- [x] **Task 4 — CLI đa lớp: `--raw <dir> --out-dir <dir> [--layer <code>]`** (AC1)
  - [x] `main.rs`: thay `--out <file>` bằng `--out-dir <dir>`; thêm `--layer <base|thieu-chuu|vietphrase|all>`, mặc định `all`.
  - [x] ⛔ Gặp `--out` ⇒ **lỗi tường minh** nêu tên tham số thay thế. ⛔ Không nhận âm thầm để "tương thích ngược" — một lượt build ghi nhầm chỗ là một tệp cũ bị đè.
  - [x] `--layer all` ⇒ dựng **đủ ba** tệp. ⛔ **Không** có chế độ *"bỏ qua lớp thiếu nguồn"* *(§Bẫy 7)*.
  - [x] Tên tệp đầu ra cố định trong mã: `dict-core.db` · `dict-<code>.db`. ⛔ Không cho người gọi tự đặt tên *(§Quyết định #3)*.
  - [x] Bump `version` của `tools/dict-build/Cargo.toml` lên `0.2.0` — nó vào `dict_meta('builder_version')` của **cả ba** tệp và là cách duy nhất phân biệt tệp dựng bởi CLI cũ/mới.

- [x] **Task 5 — `build.rs`: một `finalize` dùng chung cho mọi lớp** (AC1, AC4)
  - [x] Tách phần đuôi hiện có *(rebuild FTS → ANALYZE/VACUUM → `journal_mode=DELETE` → kiểm no-wal → băm → `rename` từ `.tmp`)* thành **một** hàm dùng chung, gọi bởi **cả ba** đường dựng.
  - [x] 🔴 ⛔ **Không copy-paste phần đuôi đó cho lớp gỡ rời** *(§Bẫy 2)*.
  - [x] Giữ nguyên `insert::create_schema` cho **mọi** lớp — đây là điều kiện của AC4.
  - [x] Giữ nguyên `require_nonempty` cho mọi lớp.
  - [x] Thêm một hàng `dict_meta('layer', 'base'|'<code>')`. Đây là **hàng trong bảng khoá/giá trị đã có**, ⛔ **không** phải cột mới ⇒ `sqlite_master` không đổi ⇒ AC4 vẫn đạt *(§Quyết định #5)*.
  - [x] 🔴 Đường dựng lớp gỡ rời ⛔ **không bao giờ mở `dict-core.db`** *(§Bẫy 3)*.

- [x] **Task 6 — Hai parser, mỗi lớp một module** (AC1, AC2)
  - [x] `tools/dict-build/src/sources/{thieu_chuu,vietphrase}.rs`, cùng chữ ký `fn parse(reader) -> impl Iterator<Item = Result<RawEntry, ParseIssue>>` như năm module đã có.
  - [x] **Thiều Chửu** — TSV 3 cột, ánh xạ ở §Thông tin kỹ thuật:
    - [x] Cột 2 tách bằng `|` ⇒ **nhiều âm Hán Việt**. Giữ nguyên chuỗi có `|` vào `dict_entry.han_viet` *(1.639/9.897 mục có nhiều âm)* — ⛔ không nhân bản `dict_entry`, âm đọc không phải nghĩa.
    - [x] Cột 3 tách bằng `<br>` **và** số thứ tự `1.` `2.` `3.` ⇒ **nhiều hàng `dict_sense`**, `ord` theo số. ⛔ Không nối thành một `gloss` — FR29 đòi mỗi nghĩa một hàng.
    - [x] `lang = 'zh'`, `pos = NULL`, `pos_lang = NULL` *(nguồn không phân định từ loại)*.
    - [x] 🟢 Trích dẫn có tác giả — mẫu `(Nguyễn Du 阮攸)` — ⇒ `dict_citation.author`. **Đây là nguồn đầu tiên của dự án làm `dict_citation` có dữ liệu** *(bảng này đang **0** hàng)*. Nếu bóc tách quá giòn, để nguyên trong `gloss` là **chấp nhận được**; ⛔ bịa `work`/`author` thì **không**.
    - [x] 🔴 **Dòng 108 hỏng thật** *(`亯` — chỉ 2 cột, có thẻ HTML rơi rớt `</h4>`)* ⇒ **`ParseIssue`, ⛔ không `panic!`**. Đây là ca thật, không phải giả thuyết — nó phải có mặt trong fixture.
  - [x] **VietPhrase** — `<hán>=<nghĩa1>/<nghĩa2>/…`:
    - [x] Bỏ **BOM ở ký tự đầu tệp** nếu có — dòng phòng vệ rẻ. Tệp hôm nay ⛔ không có, nhưng một lượt `iconv` khác cấu hình sẽ để lại, và BOM lọt vào thành **một đầu mục rác** thay vì một lỗi *(§Bẫy 5)*.
    - [x] Tách `=` bằng `splitn(2, '=')` — **679.311/679.311** dòng có **đúng một** dấu `=` *(100 %, đã kiểm)*, nhưng `splitn` đúng nghĩa hơn `split` và miễn nhiễm với dòng lạ.
    - [x] Tách `/` ⇒ **nhiều hàng `dict_sense`**, `ord` giữ **thứ tự ưu tiên** của tệp gốc *(mục đầu là bản dịch được ưu tiên)*. Khuôn tách đã có ở `sources/cedict_common.rs` — **đọc nó trước khi viết mới**.
    - [x] 🔴 **Luật lọc rác** — bỏ dòng có nghĩa rỗng hoặc bằng `()`. **9 dòng** trong tệp thật, đều là spam quảng cáo *(`txt8 小说下载网`, `zuilu 书院`…)*. Mỗi dòng bỏ ⇒ `ParseIssue` có lý do, vào bảng `SourceStats`.
    - [x] `lang = 'zh'`, `reading = NULL`, `han_viet = NULL`, `pos = NULL`.
    - [x] ⚠️ Đầu mục **không chỉ là từ** — có cả cụm và **cả câu** *(`去那里要干什么?`)*. Đó là **đúng dữ liệu**, ⛔ không lọc bỏ theo độ dài.
  - [x] 🔴 **Fixture trích THẬT**, 20–50 dòng mỗi nguồn, commit được. ⛔ **Không bịa một giá trị nào** — lượt review Story 1.9 đối chiếu byte-for-byte và bắt được fixture bịa 20/20 dòng. Cắt bớt số nghĩa mỗi dòng cho gọn là **được**; đổi một giá trị là **không**.
  - [x] Fixture bắt buộc chứa: dòng 108 của Thiều Chửu · ≥1 dòng rác `()` của VietPhrase · ≥1 mục nhiều âm `|` · ≥1 mục nhiều nghĩa ở cả hai nguồn.
  - [x] `char_idx` chạy qua `insert::insert_entry` như cũ ⇒ ⛔ không viết đường chèn riêng.

- [x] **Task 7 — Điền `dict-manifest.toml` và siết cổng của nó** (AC5)
  - [x] Hai khối `[[detachable]]`, `name` = `thieu-chuu` · `vietphrase`, `url` cùng tag `dict-v1` với `[base]`, `sha256` + `source_version` **từ chính lượt build**.
  - [x] `check-dict-manifest.mjs`: thay nhánh *"0 mục hôm nay, hợp lệ"* bằng **đòi đúng hai mục, đúng hai `name`, không trùng, không dư**.
  - [x] ⚠️ **Hai việc trên phải cùng MỘT commit** *(§Bẫy 8)*. *(Chưa commit trong phiên này — sẽ commit cùng nhau khi Ice yêu cầu)*
  - [x] ⛔ Không nới `URL_RE` — nó ghim đúng `github.com/vannamhh/AuraTranslate/releases/download/dict-v`, và lượt review Story 1.9 đã đóng đúng lỗ hổng đó. *(URL_RE không đổi)*
  - [x] ⛔ Cổng vẫn ⛔ không đọc `.db`, ⛔ không tải mạng — phải xanh trên runner CI không có byte dữ liệu nào. *(Không đổi hành vi đó)*

- [x] **Task 8 — Siết `check-dict-build.mjs`** (AC1, AC4)
  - [x] **Kiểm C** — nâng `RS_FILE_FLOOR` khớp cây mới *(số thật ghi ở Task 1; đặt sàn thấp hơn số thật vài đơn vị như tiền lệ 10/18)*. *(20 tệp thật, sàn đặt 18)*
  - [x] **Kiểm D mới — chống trôi giữa Rust và manifest.** Quét `sources_meta.rs` lấy `code:` của lớp gỡ rời, đối chiếu với `name` mà `check-dict-manifest.mjs` đòi. Lệch ⇒ FAIL. *(Khuôn quét chéo tệp: `check-i18n.mjs`.)* — 🔄 **XANH** sau Task 7/10 *(§Bẫy 8: nó FAIL có chủ ý trong lúc manifest còn trống, ⛔ không phải trạng thái cuối)*. 🔄 **Lượt code review bổ sung một SÀN**: `DETACHABLE_ALL` rỗng giờ là **FAIL**, ⛔ không còn đọc thành "đạt" — cùng doctrine với sàn số tệp của Kiểm C.
  - [x] **Kiểm E mới — cách ly lớp.** Trong `src/sources/{thieu_chuu,vietphrase}.rs` và đường dựng lớp gỡ rời: ⛔ không token `dict-core` / `dict_core` *(§Bẫy 3)*.
  - [x] Kiểm A giữ nguyên danh sách token cấm — hai module mới nằm dưới `src/` nên **tự động** vào phạm vi quét. Mọi miễn trừ mới khai `// dict-build:allow <token> — <lý do>` và **in ra tổng số miễn trừ mỗi lượt**.

- [x] **Task 9 — Test** (AC1, AC2, AC3, AC4)
  - [x] `tests/layers.rs` mới. Dựng **cả ba** tệp từ **fixture** vào thư mục tạm, rồi:
  - [x] 🔴 `sqlite_master_is_byte_identical_across_all_outputs` — **AC4**. So chuỗi `type|name|tbl_name|sql` sắp xếp, cộng `PRAGMA user_version`.
  - [x] `each_detachable_file_holds_exactly_one_dict_source_row_with_its_own_code` — **AC1**.
  - [x] `dict_core_holds_zero_rows_for_any_detachable_code` — **AC1**, đối chứng âm.
  - [x] `each_detachable_source_declares_non_empty_license_text_and_attribution` — **AC2**.
  - [x] `thieu_chuu_attribution_names_the_author` — **AC2**, khẳng định chuỗi chứa *"Thiều Chửu"* *(nghĩa vụ quyền nhân thân)*.
  - [x] `vietphrase_is_unknown_not_public_domain` — **AC3**, đối chứng âm cho lỗi gán nhãn dễ mắc nhất.
  - [x] `license_kind_column_accepts_a_value_outside_the_open_license_set` — **AC3**. Chèn thẳng một hàng `license_kind = 'author-grant'` và khẳng định **thành công** — đây là cách duy nhất chứng minh *"biểu diễn được phép riêng của tác giả"* khi HVTĐTD chưa có mặt.
  - [x] `every_layer_uses_delete_journal_mode_with_no_wal_artifacts` — chạy cho **cả hai** lớp *(§Bẫy 2)*.
  - [x] `tests/parse.rs`: hai nhóm ca mới. Bắt buộc có ca **dòng 108 hỏng** (Thiều Chửu) và ca **nghĩa rỗng `()`** (VietPhrase) ⇒ đếm được qua `ParseIssue`, ⛔ không `panic!`.
  - [x] ⛔ **`src-tauri/tests/` không thêm và không sửa một dòng nào.** **62 test phải ra đúng 62.** *(Xác nhận: `git status`/`git diff --stat` không chạm `src-tauri/` — 62 test vẫn 62)*

- [x] **Task 10 — Chạy thật trên hai nguồn thô và ghi số** (AC1, AC5, AC6)
  - [x] `cargo run --release --manifest-path tools/dict-build/Cargo.toml -- --raw tools/dict-build/raw --out-dir tools/dict-build/out`
  - [x] Ghi bảng `SourceStats` đầy đủ *(đọc / bỏ / **lý do bỏ** / entry / sense / example / citation)* cho hai nguồn mới.
  - [x] 🔴 **Đối chiếu số đọc được với số đã khảo sát** ở §Thông tin kỹ thuật: Thiều Chửu **9.897** mục *(1 dòng bỏ)*; VietPhrase **679.311** mục *(≈9 dòng bỏ)*. Lệch quá 1% ⇒ parser sai, ⛔ không phải *"nguồn vốn thế"*. *(Khớp tuyệt đối, 0% lệch cho cả hai)*
  - [x] Ghi SHA-256 + kích thước byte của hai tệp `.db`.
  - [x] Ba phép nghiệm thu tay, ghi **SQL nguyên văn** để lượt rà sau tái lập được:
    - [x] Một chữ Hán có mặt ở **cả ba** tệp ⇒ mỗi tệp có bản ghi **riêng**, ⛔ không tệp nào chứa bản ghi của tệp kia.
    - [x] `SELECT code, license_kind, license_id FROM dict_source` trên từng tệp lớp gỡ rời ⇒ đúng bảng AC3.
    - [x] `ls -la` thư mục đầu ra ⇒ **0** tệp `-wal`/`-shm`.

- [x] **Task 11 — Bảng kế toán NFR6 và phán quyết** (AC6)
  - [x] Dựng bảng đúng khuôn AC6, **mỗi dòng một số byte đo được**; hai lớp chưa dựng ghi `[----] chưa đo`.
  - [x] Tổng bằng **byte**, rồi mới đổi MB thập phân. Đối chiếu **200.000.000**.
  - [x] Phán quyết **ĐẠT** *(ghi rõ "với hai lớp")* hoặc **VƯỢT**, viết đúng chữ. *(VƯỢT)*
  - [x] Nếu **VƯỢT**: ghi **số byte vượt**, liệt kê đòn bẩy **kèm số** *(hai khoản `deferred-work.md:75` · `sense_fts_nd` từng lớp · bỏ một lớp · nâng trần)*, rồi ⛔ **DỪNG**. Quyết định tầng PRD — §Câu hỏi cho Ice #1.
  - [x] ⛔ Không sửa `src-tauri/Cargo.toml`, ⛔ không `[profile.release]`, ⛔ không bỏ chỉ mục, ⛔ không bỏ nguồn.

- [x] **Task 12 — Tài liệu và bàn giao** (không AC)
  - [x] `tools/dict-build/README.md` — CLI mới, bảng bảy nguồn *(5 nền + 2 gỡ rời)*, quy ước `raw/`, giấy phép từng nguồn, và 🔴 **lệnh `iconv` chép-dán kèm lý do** cho ca *tải lại VietPhrase từ kho gốc* *(kho gốc là UTF-16LE — §Bẫy 5)*.
  - [x] `src-tauri/resources/dict/README.md` — *"tệp nào tồn tại"* đổi từ một sang ba; nêu hai lớp còn thiếu.
  - [x] `deferred-work.md`:
    - [x] ➕ Mục MỚI **đích danh story nối tiếp**: HVTĐTD + Cổ hán văn, kèm **lý do thu hẹp** và trạng thái nguồn *(HVTĐTD phải xin tác giả; Cổ hán văn chưa quyết được là lớp gì)*.
    - [x] ➕ Mục MỚI **đích danh 1.13**: nghiệm thu **hành vi** FR36 *(xoá file → chạy lại bộ test tra cứu)* *(§Bẫy 6)*.
    - [x] ➕ Mục MỚI **đích danh 1.11/1.13**: `dict_source.id` ⛔ không toàn cục giữa các tệp *(§Bẫy 9)*.
    - [x] 🔄 Cập nhật `:75` [D4] với phán quyết NFR6 thật.
    - [x] 🔄 Cập nhật `:236` — phạm vi Story 10.1 giờ là **ba** tệp, ⛔ không đánh dấu đóng.
    - [x] ➕ Mục MỚI **đích danh 10.4**: nghĩa vụ thông báo tác giả HVTĐTD (PRD §8.5).
  - [x] §Completion Notes: **lệnh chép-dán cho Ice** tải hai tệp lên release `dict-v1`.
  - [x] ⛔ **Không sửa** `prd.md` / `epics.md` / `ARCHITECTURE-SPINE.md` — tiền lệ quyết định #3 của Ice ở Story 1.3. Lệch giữa tài liệu và mã ⇒ ghi vào §Completion Notes để Ice sửa. *(Đã biết một lệch: `epics.md` §Story 1.10 nói **bốn** lớp, story này giao **hai** — Ice chốt 2026-08-05.)*
  - [x] ⛔ **Không sửa** tệp story 1.9 (`done`) — nó là bản ghi. Lệnh `gh release create` cũ ở đó thành lỗi thời; câu thay thế nằm ở story này.
  - [x] ⛔ **Không sửa và không xoá** `docs/dics/**` — kho nguồn thô của Ice, gồm cả `_khong-dung/` và `tudien-2.2.zip` *(⚠️ zip đó **là bằng chứng giấy phép CC0**, không phải rác)*.

---

## Dev Notes

### Ranh giới phạm vi — đọc trước khi gõ dòng đầu tiên

| Thứ | Trong story này? |
|---|---|
| Hai parser lớp gỡ rời trong `tools/dict-build/src/sources/` | ✅ **Có** — hạt nhân |
| `sources_meta` tách `BASE_ALL` / `DETACHABLE_ALL` + văn bản giấy phép | ✅ **Có** |
| CLI `--out-dir` + `--layer` | ✅ **Có** |
| Hai `[[detachable]]` trong `dict-manifest.toml` + siết cổng | ✅ **Có** |
| Ba phép kiểm mới trong hai cổng `.mjs` | ✅ **Có** |
| Đóng bảng kế toán NFR6 | ✅ **Có** — kể cả khi kết luận là **VƯỢT** |
| **HVTĐTD · Cổ hán văn** | ❌ **Không** — story nối tiếp *(Ice chốt 2026-08-05)*. ⛔ Không dựng tệp `.db` rỗng cho chúng |
| **Trung Việt** *(nguồn thứ năm)* | ❌ **KHÔNG** — ⛔ không nằm trong PRD §8.2. Xem `docs/dics/_khong-dung/README.md` |
| **Trần Văn Chánh** | ❌ **KHÔNG BAO GIỜ** — PRD §8.2 đã loại vì còn bản quyền |
| **Đường tra cứu ba nhánh** (`core/dict/`) | ❌ **Không** — **1.11** |
| **Cổng `DictionarySource`** + phép thử *"xoá file"* của AD-10 | ❌ **Không** — **1.13**. ⛔ `ports/mod.rs` giữ nguyên 5 dòng |
| **Matcher dùng chung** | ❌ **Không** — **1.12** |
| Bật/tắt nguồn, ghi công trên UI | ❌ **Không** — 1.19 / 10.4 |
| `bundle.resources`, đóng gói `.db` vào bản phát hành | ❌ **Không** — **10.1** |
| Tạo GitHub Release, tải tệp lên | ❌ **Không** — Ice làm tay |
| Đổi **bất kỳ** DDL nào của `schema.rs` | ❌ **KHÔNG BAO GIỜ** — §Bẫy 1 |
| Dựng lại `dict-core.db` | ❌ **Không** — §Quyết định #8 |
| Sửa `src-tauri/**` *(src, tests, Cargo.toml, tauri.conf.json)* | ❌ **Không** — **0 dòng** |
| Phụ thuộc mới cho `src-tauri` **hoặc** cho `tools/dict-build` | ❌ **Không** — 0 crate mới, ⛔ kể cả crate dò mã hoá |
| Chuyển mã lại `VietPhrase.txt` | ❌ **Không** — đã là UTF-8 *(Ice, 2026-08-05)*. ⛔ `iconv` hai lần làm hỏng tệp |

### Trạng thái repo hiện tại — số, không phải mô tả

> ⚠️ **Baseline là CÂY LÀM VIỆC, không phải `d9bc252`.** Toàn bộ Story 1.9 *(status `done`)* **chưa được commit**: `tools/dict-build/**`, `scripts/check-dict-build.mjs`, `scripts/check-dict-manifest.mjs`, cộng sửa ở `dict-manifest.toml`, `package.json`, `.github/workflows/ci.yml`, `scripts/check-i18n.mjs`, `src-tauri/tauri.conf.json`, `src-tauri/tests/config_invariants.rs`, `src-tauri/resources/dict/README.md`, `.gitignore`. **Chạy `git status` trước khi tin bất kỳ con số nào dưới đây.**

| Thứ | Số / trạng thái |
|---|---|
| `.rs` dưới `tools/dict-build/src/**` | **18** |
| `.rs` dưới `src-tauri/src/**` | **26** — ⛔ phải không đổi |
| Test `tools/dict-build` | **49** |
| Test `src-tauri` | **62** — ⛔ phải không đổi |
| `RS_FILE_FLOOR` (`check-dict-build.mjs:46`) | **10** — số thật hôm nay là 18 |
| Miễn trừ `dict-build:allow` đang dùng | **3** — cả ba ở `model.rs:93-95` |
| `dict_source` trong `dict-core.db` | **5** hàng |
| `dict_citation` trong `dict-core.db` | **0** hàng — Thiều Chửu là nguồn đầu tiên cấp |
| `dict-manifest.toml` | `[base]` **thật**; `[[detachable]]` **0 mục** |
| Release `dict-v1` trên GitHub | **CHƯA TỒN TẠI** — Ice chưa chạy lệnh của Story 1.9 |
| `tools/dict-build/raw/` | **5** thư mục nguồn nền · 🔴 **0** thư mục lớp gỡ rời *(Task 0 tạo)* |
| `assetProtocol.scope` | đúng **1** mục `$RESOURCE/fonts/**` — dict đã bị gỡ (1.9 Task 10) |
| Cổng npm hiện có | `check:deps` · `check:tokens` · `check:i18n` · `check:commands` · `check:scope` · `check:scope:bundled` · `check:dict` · `check:dict-manifest` |
| `SCHEMA_VERSION` | **1** — ⛔ giữ nguyên |
| `builder_version` | `0.1.0` → **0.2.0** ở story này |

### 🔴 Hai nguồn — trạng thái pháp lý KHÁC NHAU, ⛔ đừng gộp thành một nhãn

| Nguồn | `license_kind` | Vì sao | Rủi ro PRD |
|---|---|---|---|
| **Thiều Chửu** (1942) | `public-domain` + `license_id = CC0-1.0` | Nguyễn Hữu Kha **mất 1954** ⇒ tác phẩm gốc hết hạn bảo hộ. 🟢 **Bản số hoá phát hành CC0 1.0** — đã đối chiếu SHA-256 với kho gốc | **R7** / `[A3]` 🟢 **hạ xuống** — xem §Thông tin kỹ thuật |
| **VietPhrase** | `unknown` | Kho `truyencuatui/VietPhrase` ⛔ **không có LICENSE**, đóng băng 2020, ⛔ không truy được tác giả. *Không biết* ≠ *không có bản quyền* | **R6** 🟡 · FR112 |

⚠️ **Hai nghĩa vụ đi kèm, không mang số FR nên rất dễ rơi:**

1. **Ghi công Thiều Chửu là nghĩa vụ pháp lý** *(quyền nhân thân vô thời hạn)*, ⛔ không phải phép lịch sự. Cưỡng chế bằng test `thieu_chuu_attribution_names_the_author`.
2. **Tác giả HVTĐTD đề nghị được thông báo khi công cụ hoàn thành** (PRD §8.5). Không thuộc story này, nhưng phải vào `deferred-work.md` — nếu không, nó không còn xuất hiện ở đâu trong dòng chảy story.

### Chín cái bẫy — sáu trong chín cho ra một lượt CI XANH với hành vi sai

#### Bẫy 1 — Đổi lược đồ để chiều một nguồn 🔴 đắt nhất trong story

Một nguồn không vừa khuôn *(Thiều Chửu cần trường "bộ thủ"; VietPhrase cần trường "độ ưu tiên")* và cám dỗ tự nhiên là thêm một cột. Ba thứ vỡ cùng lúc:

1. **AC4 chết ngay.** `sqlite_master` của tệp đó khác hai tệp kia ⇒ runtime buộc phải có nhánh riêng ⇒ đúng thứ AD-10 cấm.
2. **`SCHEMA_VERSION` desync.** Bump lên `2` thì `dict-core.db` *(đang là `1`, và ⛔ **không** dựng lại)* thành tệp cũ hơn — mà §Quyết định #7 của Story 1.9 đã đặt luật *"gặp phiên bản mới hơn thì từ chối mở"*. Đường đọc của 1.11 sẽ từ chối chính tệp mới.
3. **Không bump** thì hai tệp cùng khai `schema_version = 1` mà lược đồ khác nhau — hỏng im lặng, đúng kiểu tệ nhất.

**Luật:** ⛔ **0 dòng đổi trong `schema.rs`.** Dữ liệu bẻ cho vừa lược đồ, ⛔ không ngược lại. Trường không có chỗ ⇒ `dict_sense.note` *(là *"ghi chú"* của FR28)* hoặc `dict_source.attribution`. Thật sự không vừa ⇒ **DỪNG và hỏi Ice**.

#### Bẫy 2 — Một trong hai đường dựng bỏ `journal_mode = DELETE` 🔴

Bẫy 1 của Story 1.9 nguyên văn, nay **nhân đôi**. Tệp còn ở WAL cần **quyền ghi vào thư mục chứa nó** để dựng `-shm`, mà `$RESOURCE/dict/` trên máy người dùng là **chỉ đọc** (AD-7 *"chỉ đọc, luôn luôn"*). Lỗi chạy hoàn hảo suốt lúc phát triển và lộ ra ở **lần tra cứu đầu tiên của người dùng thật**.

Nguy hiểm hơn Story 1.9 ở một điểm: nếu chỉ **một** lớp sai, lớp kia vẫn chạy, sản phẩm vẫn tra cứu được, và triệu chứng là *"một nguồn im lặng không ra kết quả"* — đúng lớp lỗi mà AD-26 tồn tại để chặn.

**Luật:** **một** hàm `finalize` dùng chung (Task 5), và test lặp qua **cả hai** lớp.

#### Bẫy 3 — Đọc `dict-core.db` để lọc trùng 🔴

Thiều Chửu và VietPhrase trùng đầu mục với CVDICT **rất nặng**. Cám dỗ: *"bỏ những đầu mục đã có ở lớp nền cho gọn file"* — đặc biệt mạnh ở story này vì NFR6 đang căng. Đây là **hợp nhất nguồn**, cấm bởi AD-19:

> *"Một công cụ hợp nhất mọi từ điển thành một câu trả lời duy nhất là một công cụ giấu đi sai sót."* — PRD:474

Nó cũng phá FR36 theo cách không thấy được: gỡ `dict-core.db` *(giả thuyết)* sẽ làm lớp gỡ rời thủng lỗ chỗ, vì chúng đã bị cắt theo nội dung của một tệp khác.

**Luật:** đường dựng lớp gỡ rời ⛔ **không bao giờ mở `dict-core.db`**. Cưỡng chế bằng **Kiểm E** (Task 8).

#### Bẫy 4 — Sửa `exactly_five_sources_with_the_epics_md_codes` thành `assert_eq!(7)` 🔴

Test đó tồn tại với đúng một mục đích, ghi thẳng trong doc-comment của nó:

> *"Bẫy 10: Thiều Chửu · Cổ hán văn · VietPhrase · HVTĐTD KHÔNG thuộc story này — chúng là lớp gỡ rời (Story 1.10)."*

Gộp hai danh sách **xoá đúng lưới đó**. Hai danh sách tách biệt + hai test khoá + một test disjoint là hình dạng đúng (Task 2). Và doc-comment mới phải chỉ sang **story nối tiếp** đúng cách doc-comment cũ đã chỉ sang story này.

#### Bẫy 5 — Mã hoá VietPhrase: tệp trong repo đã sạch, **kho gốc thì không** 🟡

🟢 **Tệp ở `docs/dics/VietPhrase.txt` hôm nay là UTF-8, không BOM** — Ice chuyển 2026-08-05, đã kiểm chứng: **679.311/679.311** dòng có đúng một dấu `=` *(100 %)*. Dùng thẳng, ⛔ **không `iconv` lại** *(chuyển mã hai lần làm hỏng tệp)*.

🔴 **Nhưng kho gốc `truyencuatui/VietPhrase` phát hành UTF-16LE.** Bất kỳ ai tải lại — dựng lại dữ liệu sau này, một máy khác, một story sau — sẽ nhận UTF-16 và vấp đúng bẫy đã tránh được một lần.

Vì sao nó đắt: đọc UTF-16 như UTF-8 ⛔ **không hỏng ngay**. Tuỳ đường đọc, hoặc hỏng cả lượt, hoặc — tệ hơn — **đếm ra hàng trăm nghìn `ParseIssue`** rồi build vẫn chạy tiếp với 0 entry cho tới khi `require_nonempty` chặn lại. Hình dạng thứ hai làm mất **nửa giờ** đi tìm sai chỗ.

**Luật:** build tool chỉ đọc **UTF-8**, như năm nguồn kia. Chuyển mã là **bước tay** *(`iconv -f UTF-16LE -t UTF-8`)*, ghi thành lệnh chép-dán trong `tools/dict-build/README.md` **kèm lý do**. ⛔ Không thêm crate dò mã hoá — đúng §Quyết định #6 của Story 1.9.

⚠️ **Và giữ một dòng phòng vệ rẻ trong parser:** bỏ BOM nếu nó là ký tự đầu tệp. Tệp hôm nay ⛔ không có BOM, nhưng một lượt `iconv` khác cấu hình sẽ để lại — và một BOM lọt vào sẽ thành **một đầu mục rác** thay vì một lỗi.

#### Bẫy 6 — Tự dựng một đường tra cứu giả để "nghiệm thu FR36" 🟡

AD-10 nói nghiệm thu FR36 bằng *"xoá file, chạy lại bộ test tra cứu"*. **Bộ test tra cứu chưa tồn tại** — nó là 1.11/1.13. Viết một đường đọc tối thiểu ở đây là: viết mã không ai gọi *(đúng lỗi mà `core/store/mod.rs:122-134` đã ghi thành luật)*; gần như chắc chắn **lệch** với cổng `DictionarySource` mà 1.13 sẽ dựng thật; và làm cả hai story phải gỡ nó ra.

**Luật:** story này giao **điều kiện cấu trúc** của FR36 *(tệp độc lập + lược đồ đồng nhất — AC1 + AC4)*, và ghi **bàn giao đích danh 1.13**. ⛔ Không đánh dấu FR36 là *"đã nghiệm thu"* trong §Completion Notes.

#### Bẫy 7 — Chế độ "bỏ qua lớp thiếu nguồn" 🟡

Rất tiện lúc phát triển, nhất là khi hai trong bốn lớp còn thiếu nguồn thật. Nhưng nó sống sót vào lúc phát hành và cho ra một bản cài **thiếu một lớp** với lượt build **XANH** — cùng hình dạng lỗi mà `require_nonempty` đã được thêm để chặn ở lượt review Story 1.9.

**Luật:** `--layer <code>` dựng **đúng** lớp đó và hỏng nếu thiếu raw. `--layer all` *(mặc định)* dựng **đúng ba** tệp và hỏng nếu **bất kỳ** lớp nào thiếu raw. ⛔ Không cờ `--skip-missing`. **Hai lớp chưa có nguồn ⛔ không được khai trong bảng phân phối** — chúng chưa tồn tại, không phải *"tồn tại nhưng thiếu dữ liệu"*.

#### Bẫy 8 — Siết cổng manifest trước khi có SHA-256 thật 🟡

`check:dict-manifest` chạy trong job `check` ở **mọi** push, trên **cả hai** nền tảng. Đổi nó thành *"đòi đủ hai mục"* khi manifest còn 0 mục ⇒ CI đỏ ở mọi lượt push cho tới khi build thật xong — và một CI đỏ dài ngày là cách nhanh nhất để người ta bắt đầu bỏ qua nó. Thứ tự đúng: **build thật trước** (Task 10) → điền manifest → siết cổng, **cùng một commit**.

#### Bẫy 9 — Tưởng `id` của `dict_source` là toàn cục 🟡

Mỗi tệp có bảng `dict_source` **của riêng nó**, nên `id = 1` xuất hiện ở **cả ba** tệp trỏ tới ba nguồn khác nhau. Trong phạm vi một tệp thì FK vẫn đúng tuyệt đối. Nhưng đường đọc của 1.11/1.13 gom kết quả từ nhiều tệp **phải khoá theo `code`, ⛔ không theo `id`** — gộp theo `id` sẽ dán nhãn *"Thiều Chửu"* lên một nghĩa của CVDICT, tức FR31 vỡ theo cách thầm lặng nhất có thể.

Không phải việc của story này, **nhưng phải ghi vào `deferred-work.md`** — vì story này chính là nơi tình huống đó ra đời.

### Quyết định thiết kế — đã chốt, không phải lựa chọn của dev

#### #1 — Lớp gỡ rời dùng LẠI nguyên lược đồ của `dict-core.db`, ⛔ không lược đồ rút gọn

*"Lớp gỡ rời nhỏ hơn, cho nó lược đồ nhẹ hơn"* nghe hợp lý và sai ở ba tầng: AC4 chết *(§Bẫy 1)*; `schema.rs:23-26` đã ghi sẵn *"Story 1.10 dùng LẠI bảng này khi dựng từng tệp lớp gỡ rời, không dựng bảng khác"*; và một lược đồ thứ hai là một khuôn thứ hai phải nuôi mãi mãi.

Hệ quả phải nói ra: **mỗi lớp gỡ rời mang đủ ba chỉ mục FTS5 + `char_idx`**, chi phí theo hệ số Giai đoạn 0 *(dữ liệu thô 48,7 MB → 130,0 MB sau đủ chỉ mục, **2,67×**)*. Đây là khoản chi lớn nhất của AC6 và nó **được biết trước** *(§Quyết định #7)*, ⛔ không phải bất ngờ để dev tự xử lý.

#### #2 — `--layer` là một bảng phân phối, ⛔ không phải ba nhánh `if`

Mỗi lớp = một bộ ba *(mã lớp, `SourceMeta`, hàm `parse`)*. Viết thành bảng tra cứu ⇒ **thêm HVTĐTD ở story nối tiếp là thêm một dòng**. Viết thành `if code == "thieu-chuu" { … } else if …` ⇒ **đúng hình dạng của "mã riêng cho từng nguồn"** mà AC4 cấm, chỉ là ở phía build thay vì phía runtime — và nó sẽ được chép sang phía runtime bởi story tiếp theo đọc mã này.

🔴 Đây là **lý do chính** phạm vi thu hẹp xuống hai lớp mà vẫn đáng làm: hạ tầng đắt tiền dựng một lần, lớp thứ ba gần như miễn phí.

#### #3 — Tên tệp và mã nguồn cố định trong Rust, ⛔ không tham số hoá

`dict-<code>.db` sinh từ `code` của `SourceMeta`. `name` trong manifest, tên tệp, và `dict_source.code` là **cùng một chuỗi**. Cho người gọi tự đặt `--out` từng tệp là mở đúng khe *"manifest ghi một tên, tệp mang tên khác, cả hai đều hợp lệ"* — và `bundle.resources` của Story 10.1 sẽ khớp glob với một tập tệp mà không ai kiểm được là đúng tập.

#### #4 — Giấy phép là **tệp** trong `assets/licenses/`, ⛔ không phải chuỗi trong mã

`licenses.rs` đã dùng `include_str!`. Giữ nguyên khuôn: văn bản pháp lý nằm trong tệp đọc được bằng mắt, diff được, đối chiếu được với bản gốc. Một `const &str` nhiều nghìn ký tự nhúng giữa mã Rust là chỗ một dấu ngoặc kép sai làm hỏng văn bản mà không ai thấy.

#### #5 — `dict_meta('layer', …)` là hàng, ⛔ không phải cột

Story 1.13 cần biết mình vừa mở tệp nào **trước** khi đọc `dict_source`. Một hàng trong bảng khoá/giá trị đã có cho câu trả lời đó với **0 dòng đổi trong DDL** ⇒ `sqlite_master` không đổi ⇒ AC4 vẫn đạt. Một cột mới cho cùng thông tin sẽ phá AC4.

#### #6 — Hai tệp lên **cùng release `dict-v1`**, ⛔ không tag mới

`[base]` đã ghi `dict-v1` và release đó **chưa được tạo** ⇒ Ice tạo **một lần** với cả ba tệp. Ba tệp thuộc **một** thế hệ dữ liệu; tách tag là mở khe *"người dùng có `dict-core` v1 và `vietphrase` v2"* mà không cơ chế nào phát hiện. `URL_RE` đã ghim tiền tố `dict-v` nên `dict-v1` vừa khớp mà ⛔ không phải nới gì.

#### #7 — 🔴 **VƯỢT** trần NFR6 là kết cục ĐƯỢC DỰ BÁO, hợp lệ, và là điểm DỪNG

Ước tính có cơ sở, ghi ra đây để phán quyết ở Task 11 **không phải một bất ngờ**:

| Nguồn | Thô (UTF-8) | × 2,67 *(hệ số Giai đoạn 0)* |
|---|---|---|
| Thiều Chửu | 2.061.779 byte | ≈ 5,5 MB |
| VietPhrase | 23.844.586 byte | ≈ 63,7 MB |
| **Cộng** | | **≈ 69 MB** |
| **Dư địa còn lại** | | **21,5 MB** |

⚠️ **Đây là ƯỚC, ⛔ không phải số đo** — hệ số 2,67× đo trên dữ liệu có pinyin và ví dụ, còn VietPhrase thì `gloss` ngắn và ⛔ không có ví dụ, nên số thật có thể thấp hơn đáng kể. Nhưng khoảng cách quá lớn để trông chờ vào sai số.

**Luật khi VƯỢT:** ghi **VƯỢT**, ghi **số byte vượt**, liệt kê đòn bẩy **kèm số**, rồi dừng. ⛔ Không viết *"gần đạt"*, ⛔ không *"chấp nhận được"*, ⛔ không tự kéo số xuống dưới trần bằng bất cứ cách nào. AC6 gốc của Story 1.9 nói thẳng: *"nếu vượt trần, đó là quyết định tầng PRD."*

🔴 **Và nói thẳng một hệ quả mà chỉ story này nhìn thấy:** VietPhrase một mình ăn hết ngân sách, trong khi nó là lớp **gỡ rời** — thứ mà theo FR36 sản phẩm phải chạy đầy đủ khi **không có**. Nếu Ice muốn một đường ra không phải bỏ nguồn nào, câu hỏi tự nhiên là *"NFR6 có tính lớp gỡ rời tuỳ chọn không"*. ⛔ Dev **không** tự trả lời câu đó — nó sửa NFR6. Chỉ nêu vào §Completion Notes.

#### #8 — ⛔ Không dựng lại `dict-core.db`, và điều kiện tái dùng baseline `.dmg`

`dict-core.db` mất hàng chục phút để dựng và **không có gì đổi** ở story này *(0 dòng trong `schema.rs`, 0 dòng trong năm parser nền)*. Tái dùng **154.836.992 byte** và SHA-256 đã ghi ở `[base]`.

Baseline `.dmg` **2.334.696** và license **35.149** tái dùng được **với đúng một điều kiện**: `git diff --stat` cho thấy **0 dòng** đổi dưới `src-tauri/`, `src/`, `package.json` *(mục `dependencies`)*, `Cargo.lock` của `src-tauri`. Nếu **bất kỳ** dòng nào đổi ⇒ **dựng lại `.dmg` và đo lại**, ⛔ không chép số cũ. Ghi kết quả `git diff --stat` vào §Debug Log References làm chứng cứ.

### Bàn giao từ các story trước — thứ ảnh hưởng trực tiếp

1. **Story 1.9 → toàn bộ hạ tầng.** Lược đồ, `finalize`, `SourceStats`, hai cổng `.mjs`, khuôn `SourceMeta`. **Đọc `tools/dict-build/src/{schema,insert,build,finalize,sources_meta,model}.rs` trước khi viết dòng đầu tiên** — mọi thứ story này cần đã có khuôn, và viết mới là phá cùng lúc AC4 lẫn §Quyết định #1.
2. **Story 1.9 → dư địa NFR6.** **21.507.450 byte**, từ tổng **178.492.550** và trần **200.000.000**.
3. **Story 1.9 → bài học fixture.** Lượt review đối chiếu **byte-for-byte** với tệp thật và bắt fixture bịa. Hai fixture mới sẽ bị soi cùng cách.
4. **Story 1.9 → hình dạng cổng.** Miễn trừ **có tên, có lý do, in ra mỗi lượt**; lỗi hạ tầng ⛔ không được báo thành *"đạt"*; sàn số tệp chống *"cây rỗng đọc thành sạch"*.
5. **Story 1.9 → `require_nonempty` và bảng `skip_reasons`.** Cả hai là lưới chính bắt một parser đọc sai ở story này *(§Bẫy 5)*.
6. **Story 1.3 → CI.** Gắn mọi thứ vào **job `check` đã có**, ⛔ không tệp workflow thứ hai. Giữ nguyên *"CI ⛔ không tải dữ liệu từ điển"*.
7. **Story 1.8 → khuôn macro.** `scope_kinds!`/`message_keys!`: *một khai báo, nhiều thứ sinh ra*. ⛔ Không rải khai báo ra nhiều chỗ.
8. **Story 1.1 → NFR6.** Font **21.285.713 byte** đo thật. ⛔ Không subset font để lấy chỗ *(AC6 cấm tường minh)*.

### Nợ nhận lại — mục `deferred-work.md` chạm story này

| Mục | Trạng thái story này giao |
|---|---|
| `:75` — [D4] `reqwest` default features + `crate-type` | 🟡 **Cập nhật, ⛔ không đóng.** Ghi phán quyết NFR6 thật. Nếu **VƯỢT**, đây là hai đòn bẩy đầu — nhưng ⛔ **dev không đụng `Cargo.toml`** *(Ice chốt lần thứ ba ở Story 1.9)* |
| `:236` — lưới thay thế `bundle.resources`/`dict/*.db` | ⏭️ **Không đụng nội dung** — chủ sở hữu là **10.1**. Nhưng phạm vi lớn thêm: **ba** tệp, không phải một. **Cập nhật câu chữ**, ⛔ không đánh dấu đóng |
| **Mục MỚI** — HVTĐTD + Cổ hán văn | ➕ **Thêm, đích danh story nối tiếp** — kèm lý do thu hẹp và trạng thái nguồn |
| **Mục MỚI** — FR36 nghiệm thu **hành vi** | ➕ **Thêm, đích danh 1.13** *(§Bẫy 6)* |
| **Mục MỚI** — `dict_source.id` ⛔ không toàn cục | ➕ **Thêm, đích danh 1.11 / 1.13** *(§Bẫy 9)* |
| **Mục MỚI** — nghĩa vụ thông báo tác giả HVTĐTD (PRD §8.5) | ➕ **Thêm, đích danh 10.4** — không mang số FR nên không story nào tự nhận |

### Testing standards

- **Test của build tool nằm trong build tool.** `cargo test --manifest-path tools/dict-build/Cargo.toml`, trên **fixture nhỏ đã commit**. ⛔ Không test nào phụ thuộc nguồn thô đã tải.
- **`src-tauri/tests/` ⛔ KHÔNG thêm và KHÔNG sửa một dòng nào.** **62 test phải ra đúng 62.**
- **Đối chứng âm bắt buộc** cho AC1 và AC3. Một test chỉ khẳng định *"`dict-thieu-chuu.db` có nguồn `thieu-chuu`"* sẽ **vẫn xanh** nếu nó cũng chứa cả nguồn kia. Phải khẳng định **cả cái không có mặt**.
- **AC3 cần một test nhìn về TƯƠNG LAI.** `license_kind_column_accepts_a_value_outside_the_open_license_set` chèn thẳng `'author-grant'` — đây là cách **duy nhất** chứng minh *"biểu diễn được phép riêng của tác giả"* khi HVTĐTD chưa có mặt. ⛔ Bỏ test này là để AC3 đạt bằng lời hứa.
- **Ca lỗi phải là ca THẬT.** Dòng 108 của Thiều Chửu và 9 dòng rác của VietPhrase **tồn tại trong dữ liệu thật** — chúng vào fixture, ⛔ không bịa ca lỗi tổng hợp thay thế.
- **Test parity `sqlite_master` chạy trên FIXTURE**, ⛔ không trên tệp thật — phải xanh trên runner CI không có byte dữ liệu nào.
- **Ba cổng `.mjs` nghiệm thu ĐỎ-RỒI-XANH bằng tay**, ghi từng ca vào §Debug Log References. Script `.mjs` ⛔ không được type-check và ⛔ không có test *(`deferred-work.md:78`, `:101`)* — đỏ-rồi-xanh là lưới **duy nhất**, và phải **chạy thật**, ⛔ không mô tả.
- **Phép nghiệm thu trên dữ liệu thật** (Task 10) chạy tay, ghi **SQL nguyên văn**.

### Thông tin kỹ thuật — nguồn thô, 🟢 ĐÃ KIỂM CHỨNG 2026-08-05

> Khảo sát thật trên tệp Ice đã tải, ⛔ không phải suy đoán. Kho nguồn: `docs/dics/` — đọc `docs/dics/README.md` trước.

#### Thiều Chửu — `docs/dics/Thieu chuu/TudienThienChuu.txt`

| Thuộc tính | Giá trị |
|---|---|
| SHA-256 | `20ec62ba8dd9ec79d408bb824d7aba8b5eca4afca3c91426e45d693e77275f5f` |
| Kích thước | **2.061.779** byte · **9.898** dòng |
| Mã hoá | UTF-8 |
| Cấu trúc | **TSV 3 cột** |
| Mục hợp lệ | **9.897** *(khớp `wordcount=9897` trong `TudienThienChuu.ifo`)* |
| Dòng hỏng | **1** — dòng **108** *(`亯`)*: chỉ 2 cột, có thẻ HTML rơi rớt `</h4>` |
| Mục nhiều âm Hán Việt *(dấu `\|`)* | **1.639** |
| `source_version` | `catusf/tudien@2.2 (2022-10-10)` — lấy từ `tudien-2.2.zip` |

```
一	nhất	1. Một, là số đứng đầu các số đếm...<br>2. Cùng. Như sách Trung Dung 中庸 nói...
丁	đinh|chênh	1. Can Ðinh, can thứ tư trong mười can. <br>2. Ðang...
```

| Cột | → Lược đồ | Ghi chú |
|---|---|---|
| 1 — chữ Hán | `dict_entry.headword` | `lang = 'zh'`; `headword_simp = NULL` |
| 2 — âm Hán Việt | `dict_entry.han_viet` | Nhiều âm tách bằng `\|` — **giữ nguyên chuỗi**, ⛔ không nhân bản entry |
| 3 — nghĩa | **nhiều** `dict_sense` | Tách bằng `<br>` **và** số `1.` `2.` `3.`; `ord` theo số |
| — trong cột 3: `Như X 漢 …` | `dict_example` | Bóc được thì bóc; ⛔ giòn quá thì để nguyên trong `gloss` |
| — trong cột 3: `(Nguyễn Du 阮攸)` | `dict_citation.author` | 🟢 **Nguồn đầu tiên làm `dict_citation` có dữ liệu.** ⛔ Bịa `work`/`author` thì không |

Phân bố số nghĩa *(đếm `<br>`)*: 4.596 mục ×1 · 2.287 ×2 · 1.261 ×3 · 678 ×4 · 439 ×5 · 249 ×6 · 147 ×7 · 95 ×8 · còn lại nhiều hơn.

🟢 **Xuất xứ và giấy phép — đã đối chiếu, và nó hạ một rủi ro của PRD:**

Tệp này **khớp byte-for-byte** *(cùng SHA-256)* với `tudien-2.2/dict/TudienThienChuu.txt` bên trong `docs/dics/tudien-2.2.zip`, và kho đó *(`catusf/tudien`)* phát hành theo **CC0 1.0 Universal** — người số hoá **từ bỏ mọi quyền**. Mọi tệp `TudienThienChuu.*` khác nằm dưới `output/` trong cùng kho, tức **bản dẫn xuất**.

⇒ Giả định `[A3]` và rủi ro **R7** *("bản số hoá có thể kèm tuyên bố quyền riêng")* **không còn áp cho bản này**. ⚠️ Đây là dữ kiện **mới so với `prd.md`** — dev ⛔ không sửa PRD, chỉ ghi vào §Completion Notes để Ice cập nhật.

⚠️ ⛔ **Giữ `tudien-2.2.zip` lại** — nó **là bằng chứng giấy phép**, không phải rác.

#### VietPhrase — `docs/dics/VietPhrase.txt`

| Thuộc tính | Giá trị |
|---|---|
| SHA-256 | `5cb6a00d9697642c4e3cf735c24e88369ec199c69fcde45cb193036a0e26d617` |
| Kích thước | **23.844.586** byte · **679.311** dòng |
| Mã hoá | 🟢 **UTF-8, không BOM** — Ice chuyển 2026-08-05, dùng thẳng. ⚠️ **Kho gốc phát hành UTF-16LE** *(§Bẫy 5)* |
| Mục hợp lệ | **679.311** — mọi dòng đều là mục, ⛔ không dòng thừa |
| Dòng có đúng một `=` | **679.311 / 679.311** *(100 %)* |
| Dòng rác *(nghĩa `()` hoặc rỗng)* | **9** — spam quảng cáo: `txt8 小说下载网`, `zuilu 书院`, `(未完待续` |
| Dòng chứa URL/domain | **8** |
| `source_version` | `truyencuatui/VietPhrase@master` + ngày tải |

```
一个又一个=lần lượt
出没=qua lại/thường lui tới/ẩn hiện/xuất ẩn
去那里要干什么?=đi vào đó để làm gì?
```

| Phần | → Lược đồ |
|---|---|
| trước `=` | `dict_entry.headword`, `lang = 'zh'` |
| sau `=`, tách `/` | **nhiều** `dict_sense`, `ord` = **thứ tự ưu tiên** của tệp gốc |
| — | `reading` · `han_viet` · `pos` · `pos_lang` đều `NULL` |

Phân bố số nghĩa: **613.478 mục ×1** *(90,3 %)* · 37.487 ×2 · 13.610 ×3 · 6.945 ×4 · 3.648 ×5 · 1.894 ×6 · 947 ×7 · 563 ×8 · còn lại nhiều hơn.

⚠️ **Ba điều phải đọc kỹ:**

0. 🟢 **Nội dung đã kiểm chứng nguyên vẹn sau khi chuyển mã** *(2026-08-05)*: 679.311/679.311 dòng đúng một `=`, 9 dòng rác, phân bố nghĩa **không đổi** so với bản UTF-16. ⇒ Số ở bảng trên là số **đối chiếu được** ở Task 10, ⛔ không phải ước.
1. **Đầu mục không chỉ là từ.** Có cả cụm và **cả câu** *(`去那里要干什么?`)*. Đó là **đúng dữ liệu** — lớp này gần **Translation Memory cộng đồng** hơn là từ điển. ⛔ Không lọc bỏ theo độ dài.
2. **Có dòng dùng `,` thay `/` cho đa đầu mục** *(`太一,正一,纯一…=thái nhất,chính nhất,thuần nhất…`)*. Số lượng nhỏ. ⛔ Không tự bóc tách kiểu đó — nếu tách sai thì một đầu mục ghép sẽ mang nghĩa của đầu mục khác, tức **sai nguồn ở mức nghĩa**. Nạp nguyên, hoặc bỏ có ghi `ParseIssue`.
3. **Chỉ 9/679.311 dòng là rác** — thấp hơn nhiều so với lo ngại ban đầu về dữ liệu cộng đồng. ⛔ Đừng viết bộ lọc phức tạp cho một tỷ lệ 0,001 %; luật *"nghĩa rỗng hoặc `()`"* là đủ, và mọi dòng bỏ đi vào `skip_reasons`.

#### ⛔ KHÔNG dùng — đã khảo sát và loại

| Thứ | Vì sao |
|---|---|
| `_khong-dung/Tu-dien-ThienChuu+TranVanChanh-*.{txt,xlsx}` | 🔴 Chứa **Trần Văn Chánh (1999)** — PRD §8.2: *"Còn bản quyền · ⛔ Đã loại"*. Hai từ điển **trộn chung một cột nghĩa**, ⛔ không tách được nguồn từng mục ⇒ vi phạm **AD-19** |
| `_khong-dung/Trung Viet/` | 🟡 Nguồn **thứ năm**, ⛔ không có trong PRD §8.2. Nhận thêm nguồn là quyết định **tầng PRD** |
| `Thieu chuu/TudienThienChuu.tab` | Chỉ **2 cột** — ⛔ thiếu cột âm Hán Việt (nền tab Hán Việt, FR33) |
| `Thieu chuu/TudienThienChuu.{dict.dz,dsl.dz,mobi,html,idx,ifo,opf}` · `-Inflections.txt` | Bản **dẫn xuất** từ chính `.txt`, nằm dưới `output/` trong kho gốc |
| `Thieu chuu/hanodict_tbl_dictionary.pbix` | Power BI (VertiPaq nén) — trích rất đắt, ⛔ không thêm gì mà `.txt` chưa có |
| `HanViet.jar` | ⛔ **0 byte dữ liệu từ điển** *(bóc 53 tệp: chỉ `.class` + `images/`)*. Applet **tải dữ liệu từ máy chủ** lúc chạy, và nó là applet **Thiều Chửu**, ⛔ không phải HVTĐTD |

**Phụ thuộc mới của build tool: 0.** `serde_json` · `rusqlite` · `sha2` đã đủ. Chuyển mã UTF-16 làm **bằng tay** *(`iconv`)*, ⛔ không thêm crate — đúng §Quyết định #6 của Story 1.9.

### Project Structure Notes

Cây sau story này (chỉ phần đổi):

```text
AuraTranslate/
  tools/dict-build/
    Cargo.toml               # version 0.2.0 — ⛔ 0 phụ thuộc mới
    README.md                # CLI mới + bảng bảy nguồn + lệnh iconv
    assets/licenses/         # + CC0-1.0.txt · thieu-chuu.txt · vietphrase.txt
    src/
      main.rs                # --raw <dir> --out-dir <dir> [--layer <code>]
      build.rs               # run_base + run_layer + finalize DÙNG CHUNG
      sources_meta.rs        # BASE_ALL[5] + DETACHABLE_ALL[2]
      licenses.rs            # + hằng include_str!
      sources/{thieu_chuu,vietphrase}.rs
      schema.rs              # 🔴 0 DÒNG ĐỔI
      insert.rs · model.rs · char_idx.rs · finalize.rs   # 🔴 0 dòng đổi (trừ tách finalize)
    tests/
      layers.rs              # MỚI — parity sqlite_master, cách ly nguồn, WAL từng lớp
      parse.rs               # + 2 nhóm ca (gồm dòng 108 và rác '()')
      fixtures/raw/{thieu_chuu,vietphrase}/
    raw/ out/                # .gitignore — + 2 thư mục nguồn thô, + 2 tệp .db
  scripts/
    check-dict-build.mjs     # Kiểm D + Kiểm E mới, sàn Kiểm C nâng
    check-dict-manifest.mjs  # đòi ĐÚNG 2 [[detachable]]
  dict-manifest.toml         # + 2 khối [[detachable]] điền thật
  src-tauri/resources/dict/README.md   # "tệp nào tồn tại": 1 → 3
  _bmad-output/implementation-artifacts/deferred-work.md
  docs/dics/                 # ⛔ CHỈ ĐỌC — kho nguồn thô của Ice
```

⛔ **Không tệp nào dưới `src-tauri/` bị sửa** *(kể cả `tests/`, `tauri.conf.json`, `Cargo.toml`)*. Nếu diff chạm vào đó, **dừng lại và đọc lại §Ranh giới phạm vi** — gần như chắc chắn là đang cài trước một phần của 1.11, 1.13 hoặc 10.1.

⛔ **`package.json` và `.github/workflows/ci.yml` cũng không đổi** — hai cổng đã gắn vào job `check` ở Story 1.9; story này chỉ đổi **nội dung** hai script.

⛔ **`docs/dics/**` chỉ đọc** — kho nguồn thô của Ice. Task 0 **chép ra**, ⛔ không di chuyển, ⛔ không xoá, ⛔ không sửa.

**Đặt tên:** `snake_case` cho module Rust (`thieu_chuu.rs`); `kebab-case` cho `code`, `name` trong manifest và tên tệp (`thieu-chuu`, `dict-thieu-chuu.db`). Đây ⛔ không phải bất nhất — nó khớp đúng khuôn năm nguồn đã có (`cc_cedict.rs` ↔ `code: "cc-cedict"`).

### References

- **AC gốc của story:** [`epics.md`](../planning-artifacts/epics.md) §Story 1.10, dòng 1374–1405 *(⚠️ nói **bốn** lớp; Ice thu hẹp xuống **hai** ngày 2026-08-05)*
- **AC6 nhận bàn giao:** [`1-9-dung-du-lieu-tu-dien-lop-nen.md`](1-9-dung-du-lieu-tu-dien-lop-nen.md) §Quyết định của Ice #1 · §Completion Notes #6 *(dư địa 21.507.450 byte)* · §Debug Log References *(bảng kế toán NFR6)*
- **FR27 · FR36 · FR112:** [`prd.md`](../planning-artifacts/prds/prd-AuraTranslate-2026-08-02/prd.md) `:458` · `:488` · `:802`
- **Bộ nguồn + trạng thái pháp lý:** `prd.md` §8.2 `:886-897` · §8.5 `:925-931` · §8.6 `:933-952` · rủi ro **R6/R7** `:1053-1054` · giả định **[A2]/[A3]** `:1022-1023`
- **AD-7** *(dict `.db` chỉ đọc, luôn luôn)* · **AD-10** *(lớp gỡ rời, trường giấy phép, nghiệm thu FR36)* · **AD-19** *(không hợp nhất)* · **AD-25** *(artifact có checksum)*: [`ARCHITECTURE-SPINE.md`](../planning-artifacts/architecture/architecture-AuraTranslate-2026-08-02/ARCHITECTURE-SPINE.md) dòng 119–151, 288–292, 324–336
- **Hệ số dung lượng 2,67× và chi phí từng tầng chỉ mục:** [`phase-0-spike-results-2026-08-02.md`](../planning-artifacts/research/phase-0-spike-results-2026-08-02.md) `:55-69`
- **Số byte font + quy ước đơn vị:** [`font-spike-results-2026-08-03.md`](../planning-artifacts/research/font-spike-results-2026-08-03.md) `:78` `:82`
- **Quyền nhân thân vô thời hạn của Thiều Chửu:** [`technical-…-research-2026-08-02.md`](../planning-artifacts/research/technical-auratranslate-tauri-rust-local-first-research-2026-08-02.md) `:433-450`
- **Kho nguồn thô + phán quyết từng tệp:** [`docs/dics/README.md`](../../docs/dics/README.md) · [`docs/dics/_khong-dung/README.md`](../../docs/dics/_khong-dung/README.md)
- **Nợ đang mở:** [`deferred-work.md`](deferred-work.md) `:75` `:236`
- **Khuôn mã phải đọc trước khi viết:** `tools/dict-build/src/schema.rs` *(DDL hằng)* · `src/sources_meta.rs` *(khuôn `SourceMeta` + `LicenseRef` enum đóng)* · `src/build.rs` *(điều phối + `require_nonempty` + transaction)* · `src/finalize.rs` *(đuôi dùng chung)* · `src/sources/cedict_common.rs` *(tách `/` thành nhiều nghĩa)* · `scripts/check-dict-build.mjs` *(doctrine cổng + miễn trừ có tên)*

---

## Quyết định của Ice — chốt 2026-08-05, ⛔ không phải lựa chọn của dev

#### #1 — Phạm vi thu hẹp xuống hai lớp ✅ **CHỐT**

Thiều Chửu + VietPhrase giao ở story này. HVTĐTD + Cổ hán văn chuyển sang story nối tiếp, vì cả hai **chưa có nguồn thô** *(§Thông tin kỹ thuật)* và không nguồn nào có thể tự tìm được thay thế: HVTĐTD phải xin trực tiếp tác giả; Cổ hán văn cần quyết lại *"nó là lớp gì"* trước.

⛔ **Không dựng tệp `.db` rỗng** cho hai lớp thiếu dữ liệu — một tệp rỗng đi qua **mọi** phép kiểm cấu trúc của AC1/AC4 và chỉ hỏng ở đúng người dùng.

#### #3 — 🔴 CHẤP NHẬN VƯỢT TRẦN NFR6 ✅ **CHỐT 2026-08-05** *(sau khi có số đo thật)*

Payload **343.991.430 byte** / trần **200.000.000** — vượt **143.991.430 byte**, **chấp
nhận**. ⛔ Không bỏ nguồn, ⛔ không bỏ chỉ mục, ⛔ không đụng `Cargo.toml` *(lần thứ năm)*,
⛔ không subset font. Chi tiết đầy đủ và việc còn lại ở tầng PRD: §Câu hỏi cho Ice #1.

🔴 **Đây là quyết định của Ice, ⛔ không phải một phép đo bị bỏ qua.** Lượt rà nào sau này
đọc `prd.md` và thấy trần 200.000.000 sẽ kết luận "VƯỢT ⇒ chưa đạt" — đọc mục này trước.

#### #2 — Hai nguồn không dùng đã chuyển sang `docs/dics/_khong-dung/` ✅ **CHỐT**

**Trần Văn Chánh** *(PRD đã loại vì còn bản quyền)* và **Trung Việt** *(nguồn thứ năm chưa được PRD nhận)*. Giữ lại thay vì xoá, kèm README nêu lý do, để lượt rà sau ⛔ không phải quyết lại từ đầu. Dev ⛔ không lấy gì ra từ đó.

---

## Câu hỏi cho Ice

### #1 — ✅ ĐÃ CHỐT: NFR6 **VƯỢT** trần — Ice **CHẤP NHẬN VƯỢT**

> ## ✅ **QUYẾT ĐỊNH CỦA ICE — 2026-08-05: CHẤP NHẬN VƯỢT TRẦN.**
>
> Payload sản phẩm **343.991.430 byte** trên trần **200.000.000** — vượt **143.991.430
> byte** — được **chấp nhận có ý thức, trên số ĐO THẬT**. Không phương án (a)/(b)/(c)/(d)
> nào ở bảng dưới được kích hoạt; bảng giữ lại làm bản ghi cho lượt rà sau.
>
> **Hệ quả — điều gì KHÔNG xảy ra:**
> - ⛔ **Không bỏ nguồn nào.** Cả Thiều Chửu lẫn VietPhrase đều đi vào bản phát hành.
> - ⛔ **Không bỏ `sense_fts_nd`** của bất kỳ lớp nào — phá AC4, ⛔ không đánh đổi ở bất kỳ giá nào.
> - ⛔ **Không đụng `Cargo.toml`** *(hai khoản `deferred-work.md:75`)* — chốt lần thứ **năm**.
>   Mục [D4] vì vậy trở lại đúng bản chất ban đầu: một khoản tối ưu **AC7 (thời gian
>   build)**, ⛔ không còn là đòn bẩy AC6 (dung lượng) đang chờ.
> - ⛔ **Không subset font.**
>
> 🔴 **Việc CÒN LẠI, tầng PRD, chủ sở hữu là Ice — dev ⛔ không sửa `prd.md`:**
> trần 200.000.000 byte của NFR6 giờ **mâu thuẫn với sản phẩm thật**. Cần một trong hai:
> 1. **Nâng trần** lên một con số phản ánh sản phẩm có đủ hai lớp gỡ rời; **hoặc**
> 2. **Ghi rằng NFR6 ⛔ không tính lớp gỡ rời** — cách diễn giải tự nhiên nhất, vì
>    VietPhrase *(160.083.968 byte, tức 46,5 % toàn bộ payload)* là lớp **gỡ rời**, mà
>    **FR36** nói sản phẩm phải chạy **đầy đủ** khi **không có** nó. Theo cách đọc này,
>    payload lõi bắt buộc là **183.907.462 byte** — vẫn **DƯỚI** trần, dư 16.092.538 byte.
>
> ⚠️ Tới khi PRD được cập nhật, mọi lượt rà NFR6 sau sẽ đọc ra *"VƯỢT"* và ⛔ không có
> cách nào biết nó đã được chấp nhận nếu chỉ đọc `prd.md`. Đó là lý do quyết định này
> được ghi ở **cả ba** chỗ: đây, `deferred-work.md` mục [D4], và §Change Log.

🔴 **Cập nhật 2026-08-05 (dev):** ước tính ở §Quyết định #7 (hai lớp ≈ 69 MB) đã được đo
THẬT ở Task 10/11 — hai lớp gỡ rời chiếm **165.871.616 byte** *(5.787.648 + 160.083.968)*,
gần **2,4 lần** ước tính. Riêng **VietPhrase một mình 160.083.968 byte** — gấp **6,7×**
dữ liệu thô của nó *(23.844.586 byte)*, không phải hệ số 2,67× dùng để ước tính; xem
§Debug Log References Task 11 để biết lý do nhiều khả năng *(đầu mục dài + ba chỉ mục
FTS5)*. **Tổng payload hôm nay: 343.991.430 byte — VƯỢT trần 200.000.000 byte đúng
143.991.430 byte.** Đây không còn là dự báo — đó là kết luận cuối cùng của story này.

| Phương án | Hệ quả |
|---|---|
| **(a) Đo, ghi VƯỢT, DỪNG, Ice quyết sau** *(khuyến nghị)* | Khớp tiền lệ Story 1.9 và đúng chữ AC6 gốc. Ice quyết trên **số thật** thay vì trên ước lượng |
| (b) Cho phép trước: bỏ `sense_fts_nd` ở lớp gỡ rời | ⛔ **Phá AC4** — lược đồ không còn đồng nhất ⇒ runtime phải có nhánh riêng ⇒ đúng thứ AD-10 cấm. Không khuyến nghị ở bất kỳ giá nào |
| (c) Cho phép trước: gỡ hai khoản `deferred-work.md:75` | Đảo quyết định *"không đụng `Cargo.toml`"* đã chốt **ba lần**. Tiết kiệm chưa đo được |
| (d) Xem lại **NFR6 có tính lớp gỡ rời không** | Câu hỏi tự nhiên nhất: VietPhrase là lớp **gỡ rời**, mà FR36 nói sản phẩm phải chạy đầy đủ khi **không có** nó. Đây là sửa NFR6 ⇒ **quyết định tầng PRD**, ⛔ ngoài tầm dev |

~~**Mặc định nếu Ice không trả lời: (a).**~~ → ✅ Ice đã trả lời: **chấp nhận vượt trần**, xem khối trên.

### #2 — 🟢 Xác nhận: hai tệp lên cùng release `dict-v1`?

`[base]` đã trỏ `dict-v1` và release đó **chưa được tạo** — nên Ice tạo **một lần** với cả **ba** tệp thay vì tạo rồi bổ sung. Nếu Ice đã kịp chạy lệnh của Story 1.9, `gh release upload dict-v1 <hai tệp>` bổ sung vào đúng tag đó, ⛔ không cần tag mới.

**Mặc định nếu Ice không trả lời:** cùng `dict-v1` *(§Quyết định #6)*.

### #3 — 🟢 Có cần một mục `1-10b` trong `sprint-status.yaml` cho hai lớp còn lại không?

Story này ghi bàn giao vào `deferred-work.md`, nhưng ⛔ **không tự thêm khoá mới** vào `sprint-status.yaml` — đó là việc của `sprint-planning`, không phải của `create-story`. Nếu Ice muốn hai lớp đó hiện trong sprint, chạy `sprint-planning` hoặc thêm tay một khoá sau `1-10-…`.

### #4 — 🔴 MỚI *(dev phát hiện ở Task 10)*: `dict-core.db` bị dựng LẠI ngoài ý muốn — SHA-256 trong manifest đã đổi

> 🔄 **ĐÃ GIẢI QUYẾT — Ice chốt 2026-08-05 sau lượt code review: phương án (a).**
> Giữ tệp mới + checksum mới trong manifest *(checksum phải khớp tệp THẬT sẽ lên release)*.
> Bảng kế toán AC6 đã dùng số mới **154.464.256**; chênh 372.736 byte, ⛔ không đổi phán
> quyết **VƯỢT**.
>
> 🔴 **Nhưng câu chữ dưới đây SAI và phải sửa lại cho đúng bản ghi:** lượt review chỉ ra
> rằng *"không tránh được"* là **không đúng** — CLI mới **có** `--layer thieu-chuu` và
> `--layer vietphrase` chạy đơn lẻ *(`build::run_detachable_by_code`)*, dựng được hai lớp
> gỡ rời mà ⛔ **không chạm** `dict-core.db`. Đây là hệ quả của **lệnh đã chạy** ở Task 10,
> ⛔ không phải một ràng buộc kiến trúc. *(Ghi nhận một lệch của chính story: Task 10 kê
> lệnh không có `--layer` ⇒ mặc định `all`, mâu thuẫn với §Quyết định #8 — Ice sửa spec.)*

`--layer all` (đường dựng mặc định của CLI mới, Task 4) dựng **cả ba** tệp trong MỘT lượt
chạy, kể cả `dict-core.db` — Task 10 kê đúng lệnh mặc định đó, nên lượt chạy thật đã dựng
lại base. ⚠️ *(Câu gốc ở đây viết "không tránh được" — sai, xem khối trên.)*
Việc dựng lại kéo theo `insert_meta` (Task 5, dùng CHUNG cho cả ba đường dựng) thêm một
hàng `dict_meta('layer', 'base')` vào **CHÍNH `dict-core.db`** — đây là **1 dòng dữ liệu
mới**, ⛔ không phải đổi `schema.rs` hay năm parser nền, nên **vẫn đúng** tinh thần
§Quyết định #8 *("0 dòng trong schema.rs, 0 dòng trong năm parser nền")* — nhưng
§Quyết định #8 còn giả định thêm **"không có gì đổi"** ở mức TỆP NHỊ PHÂN, và giả định đó
hoá ra sai: tệp mới **154.464.256 byte**, **nhỏ hơn** số Story 1.9 đã ghi
*(154.836.992 byte)* đúng **372.736 byte** — khả năng cao do thứ tự chèn khác đi làm
`ANALYZE`/`VACUUM` nén trang khác đi (đã đối chiếu `Cargo.lock`: chỉ đổi đúng 1 dòng
`version`, không phải trôi phụ thuộc).

**Dev đã làm gì:** cập nhật `[base].sha256` trong `dict-manifest.toml` sang số MỚI —
giữ số CŨ sẽ là một checksum SAI cho tệp thật sẽ lên release (vi phạm chính lời văn
`dict-manifest.toml:14`: *"Không điền giá trị giả để 'cho có'... một checksum sai... hỏng
im lặng đúng kiểu tệ nhất"*). `source_version` và `url` của `[base]` **giữ nguyên**.

**Câu hỏi:** Ice có đồng ý với lựa chọn "cập nhật checksum theo tệp thật" thay vì "giữ số
cũ, chấp nhận tệp thật lên release có checksum không khớp manifest"? Nếu Ice muốn tránh
việc `dict-core.db` bị dựng lại ở các lần chạy `--layer all` sau này *(vd. khi 1.11 build
lại từ đầu)*, cách duy nhất là tách `insert_meta` thành hai đường (base không nhận
`layer`) — nhưng điều đó phá đúng yêu cầu AC4/Task 5 *"một hàm dùng chung"*. Dev **không**
tự chọn phá AC4 để giữ `dict-core.db` bất biến.

**Mặc định nếu Ice không trả lời:** giữ nguyên lựa chọn đã áp dụng — checksum MỚI trong
manifest khớp tệp THẬT.

---

## Dev Agent Record

### Agent Model Used

### Debug Log References

#### Task 0 — Nguồn thô đã đặt chỗ và đối chiếu SHA-256

```
$ shasum -a 256 tools/dict-build/raw/thieu_chuu/TudienThienChuu.txt
20ec62ba8dd9ec79d408bb824d7aba8b5eca4afca3c91426e45d693e77275f5f  tools/dict-build/raw/thieu_chuu/TudienThienChuu.txt
$ shasum -a 256 tools/dict-build/raw/vietphrase/VietPhrase.txt
5cb6a00d9697642c4e3cf735c24e88369ec199c69fcde45cb193036a0e26d617  tools/dict-build/raw/vietphrase/VietPhrase.txt
```

Cả hai khớp **byte-for-byte** với giá trị đã ghi ở §Thông tin kỹ thuật của story. ✅

| Tệp | Dòng (`wc -l`) | Byte (`wc -c`) |
|---|---|---|
| `TudienThienChuu.txt` | 9.898 | 2.061.779 |
| `VietPhrase.txt` | 679.311 | 23.844.586 |

Khớp bảng khảo sát của story (Thiều Chửu: 9.898 dòng vật lý, 9.897 mục hợp lệ vì
dòng cuối không có newline thừa hay tương tự; VietPhrase: 679.311 dòng = 679.311 mục).

**20 dòng đầu nguyên văn — Thiều Chửu:**

```
一	nhất	1. Một, là số đứng đầu các số đếm. Phàm vật gì chỉ có một đều gọi là Nhất cả.<br>2. Cùng. Như sách Trung Dung 中庸 nói. Cập kì thành công nhất dã 及其成工一也 nên công cùng như nhau vậy. <br>3. Dùng về lời nói hoặc giả thế chăng. Như vạn nhất 萬一 muôn một, nhất đán 一旦 một mai, v.v. <br>4. Bao quát hết thẩy. Như nhất thiết 一切 hết thẩy, nhất khái 一概 một mực như thế cả, v.v. <br>5. Chuyên môn về một mặt. Như nhất vị 一味 một mặt, nhất ý 一意 một ý, v.v.<br>
丁	đinh|chênh	1. Can Ðinh, can thứ tư trong mười can. <br>2. Ðang. Như đang để tang cha mẹ gọi là đinh ưu 丁憂 nghĩa là đang ở lúc đau xót vậy. <br>3. Người. Như thành đinh 成丁 nghĩa là người đến tuổi thành nhân. <br>4. Ðã lớn, là đã phải đóng thuế. Như ta 18 tuổi phải đóng sưu vào sổ đinh gọi là đinh tịch 丁藉. <br>5. Kẻ làm lụng. Như bào đinh 庖丁 là người nấu bếp, viên đinh 園丁 là người làm vườn, v.v. <br>6. Răn bảo kỹ càng. Như đinh ninh 丁寧. <br>7. Chữ. Như mục bất thức đinh 目不識丁. <br>8. Một âm là chênh. Như phạt mộc chênh chênh 伐木丁丁 chặt cây chan chát.<br>
七	thất	1. Bảy, tên số đếm. <br>2. Có nghĩa chỉ về thể văn. Như lối văn thất vấn thất đáp 七問七答 của Mai Thừa, lối văn song thất của ta.<br>
万	vạn|mặc	1. Muôn, cũng như chữ vạn 萬. <br>2. Một âm là Mặc. Như là Mặc Kỳ 万俟, họ Mặc Kỳ.<br>
丈	trượng	1. Trượng, mười thước ta là một trượng. <br>2. Ðo. Như thanh trượng 清丈 nghĩa là đo xong số ruộng đất nào rồi. <br>3. Già cả. Như lão trượng 老丈, trượng nhân 丈人 (người già cả). Bố vợ gọi là nhạc trượng 岳丈.<br>
三	tam|tám	1. Ba, tên số đếm. <br>2. Một âm là tám. Hai ba lần, đọc đi đọc lại. Như. Nam Dong tám phúc bạch khuê 南容三復白圭 ông Nam Dong đọc đi đọc lại thơ bạch khuê.<br>
上	thượng|thướng	1. Trên. phàm ở trên đều gọi là thượng. Như thượng bộ 上部 bộ trên, thượng quyển 上卷 quyển trên, thượng đẳng 上等 bực trên, v.v. <br>2. Ngày vua gọi vua là Chủ thượng 主上 gọi ông vua đang đời mình là Kim thượng 今上. <br>3. Một âm là thướng. Lên. Như thướng đường 上堂 lên thềm. <br>4. Dâng lên. Như thướng thư 上書 dâng tờ thư, thướng biểu 上表 dâng biểu, v.v.<br>
下	hạ|há	1. Dưới, đối lại với chữ thượng. Phàm cái gì ở dưới đều gọi là hạ. <br>2. Bề dưới, nhời nói nhún mình với người trên. Như hạ tình 下情 tình kẻ dưới. hạ hoài 下懷 tấm lòng kẻ dưới. <br>3. Một âm là há. Xuống, từ trên xuống dưới. Như há sơn 下山 xuống núi, há lâu 下樓 xuống lầu. 4. Cuốn. Như há kì 下旗 cuốn cờ, há duy 下帷 cuốn màn, v.v.<br>
不	bất|phầu|phủ|phi	1. Chẳng. Như bất khả 不可 không thể, bất nhiên 不然 chẳng thế, v.v. <br>2. Một âm là phầu. Là nhời nói lưỡng lự chưa quyết hẳn. Như đương phục như thử phầu 當復如此不 sẽ lại như thế chăng ? Cũng đọc là chữ phủ. <br>3. Một âm là phi. Lớn. Như phi hiển tai văn vương mô 不顯哉文王謀 cả rõ rệt thay mưu vua Văn Vương.<br>
与	dữ	1. Tục dùng như chữ 與.<br>
丐	cái	1. Xin. Như khất cái 乞丐 người ăn mày, ăn xin. 2. Cho. Như thiêm cái hậu nhân 沾丐後人 để ơn lại cho người sau.<br>
丑	sửu	1. Một chi trong 12 chi. Từ 1 giờ đêm đến 3 giờ sáng là giờ sửu. <br>2. Vai hề trong tuồng tầu cũng xưng là sửu.<br>
且	thả|thư	1. Vả, nhời nói giáo đầu. Như thả phù 且夫 vả chưng. <br>2. Nhời nói chuyển sang câu khác. Như huống thả 況且 phương chi lại. <br>3. Hãy thế. Như tạm thả 暫且 hãy tạm thế. Làm việc gì luộm thuộm, chỉ cầu cho tắc trách gọi là cẩu thả 苟且. <br>4. Sắp. Như thả tận 且盡 sắp hết. <br>5. Lại. Như kinh Thi nói. quân tử hữu tửu đa thả chỉ 君子有酒多且旨 quân tử có rượu nhiều lại ngon. <br>6. Vừa, lời nói lúc vội vàng. Như thả chiến thả tẩu 且戰且走 vừa đánh vừa chạy. <br>7. Một âm là thư. Lời nói lòng, tiếng nói còn rớt giọng ra. Như kinh Thi nói. kì lạc chỉ thư 其樂只且 thửa vui vui lắm thay !<br>
丕	phi	1. Lớn lao. Như phi cơ 丕基 nghiệp lớn.<br>
世	thế	1. Ðời, ba mươi năm là một đời, hết đời cha đến đời con cũng gọi là một đời. Như nhất thế 一世 một đời, thế hệ 世系 nối đời. <br>2. Họ nhà vua thay đổi cũng gọi là nhất thế 一世 cho nên sách thường gọi tóm lại cuộc đời là thế. Như thịnh thế 盛世 đời thịnh, quí thế 季世 đời suy. <br>3. Lại có nghĩa nói về sự giao tiếp của xã hội. Như thế cố 世故 thói đời. <br>4. Nối đời. Như bác ruột gọi là thế phụ 世父 con trưởng của chư hầu gọi là thế tử 世子. <br>5. Chỗ quen cũ. Như thế giao 世交 đời chơi với nhau, thế nghị 世誼 nghĩa cũ với nhau, hết thẩy ai có tình chơi với hàng trên mình trước đều gọi là thế cả. Như con thầy học mình gọi là thế huynh 世兄.<br>
丘	khâu|khiêu	1. Cái gò, tức là đống đất nhỏ. <br>2. phép tỉnh điền ngày xưa chia bốn tỉnh là ấp, bốn ấp là khâu. <br>3. Hợp, ngày xưa gọi sách địa dư là cửu khâu 九丘 nghĩa là các thứ trong chín châu đều hợp cả ở đấy. <br>4. Nhớn, ngày xưa gọi chị dâu trưởng là khâu tẩu 丘嫂. <br>5. Tên đức Khổng tử, vì thế sách nhà Hán đổi chữ 丘 làm 邱. <br>6. Một âm là khiêu. Như tỉ khiêu 比丘 dịch âm tiếng Phạn, người tu hành đạo Phật đã chịu đủ 250 giới luật, lần lượt đến các nhà xin ăn, trên cầu tu cho thành Phật, dưới hóa độ cho chúng sinh.<br>
丙	bính	1. Một can trong mười can. Nhà tu luyện xưa cho can bính thuộc hành hỏa, nên có nghĩa là lửa. Như phó bính 付丙 cho lửa vào đốt.<br>
业	nghiệp	1. Nghiệp. Ngày xưa cắt miếng gỗ ra từng khớp để ghi các việc hàng ngày, xong một việc bỏ một khớp, xong cả thì bỏ cả đi, gọi là tu nghiệp 修業, nay đi học ở tràng gọi là tu nghiệp, học hết lớp gọi là tất nghiệp 畢業 đều là nói nghĩa ấy cả, nói rộng ra thì phàm việc gì cũng đều gọi là nghiệp cả. Như học nghiệp 學業, chức nghiệp 職業, v.v.. Của cải ruộng nương cũng gọi là nghiệp. Như gia nghiệp 家業 nghiệp nhà, biệt nghiệp 別業 cơ nghiệp riêng, v.v. <br>2. Làm việc, nghề nghiệp. Như nghiệp nho 業儒 làm nghề học, nghiệp nông 業農 làm ruộng, v.v. <br>3. Sự đã rồi. Như nghiệp dĩ như thử 業已如此 nghiệp đã như thế rồi. <br>4. Sợ hãi. Như căng căng nghiệp nghiệp 兢兢業業 đau đáu sợ hãi. <br>5. Cái nhân. Như nghiệp chướng 業障 nhân ác làm chướng ngại. Có ba nghiệp khẩu nghiệp 口業 nhân ác bởi miệng làm ra, thân nghiệp 身業 nhân ác bởi thân làm ra, ý nghiệp 意業 nhân ác bởi ý làm ra, ba món miệng, thân, ý gọi là tam nghiệp 三業, túc nghiệp 宿業 ác nghiệp kiếp trước đã làm kiếp này phải chịu khổ gọi là túc nghiệp, v.v. Làm thiện cũng gọi là thiện nghiệp 善業. <br>6. Công nghiệp. Như đế nghiệp 帝業 công nghiệp vua.<br>
丛	tùng	1. Hợp. Sưu tập số nhiều để vào một chỗ gọi là tùng. Như tùng thư 叢書, tùng báo 叢報 tích góp nhiều sách báo làm một bộ, một loại. <br>2. Bụi rậm. Như tùng lâm 叢林 rừng rậm, cây mọc từng bụi gọi là tùng. Bây giờ gọi chùa là tùng lâm 叢林 vì xưa Phật tổ thuyết pháp thường ở các nơi rừng rậm vắng vẻ sạch sẽ, cho tăng chúng tiện chỗ tu hành vậy.<br>
东	đông	1. Phương đông, tục gọi người chủ là đông. Nước Trịnh nói với người nước Sở tự xưng nước mình là đông đạo chủ 東道主 nghĩa là người chủ ở phương đông. Tục gọi các chủ cổ phần công ty là cổ đông 股東 là do nghĩa đó. <br>2. Nước Nhật Bản ở phía đông nước Tàu nên gọi là đông dương 東洋, văn tự Nhật Bản gọi là đông văn 東文. <br>3. Ðông sàng 東牀 chàng rể (theo tích truyện Vương Hy Chi, đời Tấn).<br>
```

**20 dòng đầu nguyên văn — VietPhrase:**

```
一个又一个=lần lượt
黄沙=cát vàng
出没=qua lại/thường lui tới/ẩn hiện/xuất ẩn
之后=về sau/sau đó/lúc/khi/sau/sau khi
倒在了路途之中=chết trên đường
去那里要干什么?=đi vào đó để làm gì?
去那里要干什么=đi vào đó để làm
在那里干什么?=đang làm gì ở đó vậy?
太一,正一,纯一,神一,唯一,抱一,守一,虚一,无一=thái nhất,chính nhất,thuần nhất,thần nhất,duy nhất,bão nhất,thủ nhất,hư nhất,vô nhất
(txt8 小说下载网 -Www.txt8.cn)=()
txt8 小说下载网 -Www.txt8.cn=()
仿佛两道幽魂慢悠悠朝着白误,米娅的方向潜去=như hai đạo u hồn chậm rãi lẽn về phía Bạch Ngộ,Mễ Á đi tới.
来的时候是这么来的走的时候也就这么走了=khi tới như thế nào khi đi cũng như vậy
在他听到白误,米娅手中持有妖花的那一霎=tại sát na hắn nghe được trong tay Bạch Ngộ,Mễ Á nắm giữ yêu hoa
充滿希望的跋涉比到達目的地更能給人樂趣=niềm hi vọng còn đem lại cho người ta nhiều lạc thú hơn cả khi đạt tới mục đích
充满希望的跋涉比到达目的地更能给人乐趣=niềm hi vọng còn đem lại cho người ta nhiều lạc thú hơn cả khi đạt tới mục đích
無一事而不學無一時而不學無一處而不得=vô nhất sự nhi bất học/vô nhất thì nhi bất học/vô nhất xử nhi bất đắc
无一事而不学无一时而不学无一处而不得=vô nhất thì nhi bất học/vô nhất xử nhi bất đắc
通过商影月来获知许多不明的星空消息=thông qua Thương Ảnh Nguyệt biết được rất nhiều tin tức không rõ về Tinh Không
说起来容易,可做起来却是十分的难=nói thì dễ lúc làm thì mới khó
```

#### Task 1 — Đường cơ sở, sáu lệnh

Tất cả sáu lệnh chạy XANH trước khi đụng bất kỳ dòng mã nào của story:

| Lệnh | Kết quả |
|---|---|
| `npm run build` | ✅ built in 300ms |
| `cargo test --locked --manifest-path src-tauri/Cargo.toml` | ✅ **62** test — 15+5+5+17+4+16, 1 doctest ignored |
| `cargo test --manifest-path tools/dict-build/Cargo.toml` | ✅ **62** test — 40 (unit lib) + 13 (`tests/parse.rs`) + 9 (`tests/schema.rs`) |
| `npm run check:deps` | ✅ 326 crate Rust · 104 gói npm quét sạch |
| `npm run check:dict` | ✅ 18 tệp `.rs`, Kiểm A/B/C đạt, 3 miễn trừ |
| `npm run check:dict-manifest` | ✅ `[base]` đúng hình dạng, `[[detachable]]` 0 mục (hợp lệ hôm nay) |
| `npm run check:i18n` | ✅ 27 `.rs` + 5 `.vue`, 16 khoá `vi.json` |

⚠️ **Số thật KHÁC số ghi ở §Trạng thái repo hiện tại của story** (49 test
`tools/dict-build`) — commit `a3ed5cd` (đã có trên `master` từ trước khi story này bắt
đầu, ngoài `baseline_commit: d9bc252` ghi ở frontmatter) đã bổ sung `tests/parse.rs` +
`tests/schema.rs` tích hợp, nâng tổng lên **62**. Số **18** tệp `.rs` và **62** test
`src-tauri` khớp đúng story. Ghi số THẬT ở đây làm sàn cho Task 8 (Kiểm C mới của
`check-dict-build.mjs`) và cho đối chiếu "test không đổi" ở `src-tauri` cuối story.

#### Task 10 — Chạy thật trên hai nguồn thô, cả ba tệp

```
$ cargo run --release --manifest-path tools/dict-build/Cargo.toml -- \
    --raw tools/dict-build/raw --out-dir tools/dict-build/out
```

**Bảng `SourceStats` đầy đủ** (in ra bởi chính chương trình, không chép tay):

| nguồn | đọc | bỏ | entry | sense | example | citation |
|---|---:|---:|---:|---:|---:|---:|
| cvdict | 122.597 | 1 | 122.596 | 200.195 | 0 | 0 |
| cc-cedict | 124.758 | 0 | 124.758 | 199.615 | 0 | 0 |
| unihan | 49.870 | 0 | 49.870 | 23.285 | 0 | 0 |
| viwiktionary | 415.115 | 413.517 | 1.598 | 2.242 | 536 | 0 |
| en-wiktionary | 306.358 | 131.681 | 174.677 | 255.372 | 89.939 | 0 |
| **thieu-chuu** | **9.898** | **1** | **9.897** | **22.681** 🔄 | 0 | **263** |
| **vietphrase** | **679.311** | **9** | **679.302** | **805.558** | 0 | 0 |

🔄 **Số `sense` của `thieu-chuu` cập nhật 2026-08-05 sau lượt code review: 22.658 → 22.681 (+23).** Bản vá `split_senses` tách theo cả `<br>` VÀ số thứ tự (Task 6), cứu về 23 nghĩa từng bị gộp vào nghĩa liền trước — ca `丐` và `下` có sẵn trong fixture. Ba dòng SHA-256 dưới đây cũng là số của lượt dựng lại đó.

Lý do bỏ: `thieu-chuu` — 1× *"expected 3 tab-separated columns, got 2"* (dòng 108 thật).
`vietphrase` — 9× *"empty or placeholder '()' gloss field"* (spam quảng cáo thật).

🔴 **Đối chiếu với số đã khảo sát ở §Thông tin kỹ thuật:**

| Nguồn | Khảo sát | Đo thật | Lệch |
|---|---|---|---|
| Thiều Chửu | 9.897 mục (1 dòng bỏ) | **9.897 mục (1 dòng bỏ)** | **0%** — khớp tuyệt đối |
| VietPhrase | 679.311 mục (≈9 dòng bỏ) | **679.302 mục (9 dòng bỏ)** | **0%** — khớp tuyệt đối |

**SHA-256 + kích thước byte** (từ `report.sha256`/`report.size_bytes`, đối chiếu độc lập bằng `shasum -a 256`):

| Tệp | SHA-256 | Byte |
|---|---|---|
| `dict-core.db` | `741e166673534ab4e9666b3f9638aef40f27850762f94df42537d0d4b3450a34` 🔄 | 154.464.256 |
| `dict-thieu-chuu.db` | `e9417c12f5adc256e8cc7d49c42d09c3378fb9082fc6fd678beadf7ebe43c9d5` 🔄 | 5.787.648 |
| `dict-vietphrase.db` | `9d304210c16cd65abe9f5ed529d1b00542c3aa19cfe14d3eb6bfcc8a1a78f735` 🔄 | 160.083.968 |

**Ba phép nghiệm thu tay — SQL nguyên văn:**

1. Một chữ Hán ('山') có mặt ở CẢ BA tệp ⇒ mỗi tệp có bản ghi RIÊNG:
```sql
SELECT de.id, ds.code FROM dict_entry de JOIN dict_source ds ON ds.id = de.source_id WHERE de.headword='山';
```
```
-- dict-core.db
35059|cvdict
35060|cvdict
158051|cc-cedict
158052|cc-cedict
257164|unihan
301497|en-wiktionary
-- dict-thieu-chuu.db
1789|thieu-chuu
-- dict-vietphrase.db
676822|vietphrase
```
✅ Ba `id` khác nhau, ba `code` khác nhau — không tệp nào chứa bản ghi của tệp kia.

2. `SELECT code, license_kind, license_id FROM dict_source` trên từng tệp lớp gỡ rời:
```sql
SELECT code, license_kind, license_id FROM dict_source;
```
```
-- dict-thieu-chuu.db
thieu-chuu|public-domain|CC0-1.0
-- dict-vietphrase.db
vietphrase|unknown|
```
✅ Khớp đúng bảng AC3.

3. `ls -la` thư mục đầu ra ⇒ **0** tệp `-wal`/`-shm`:
```
total 625656
-rw-r--r--  dict-core.db        154464256
-rw-r--r--  dict-thieu-chuu.db    5787648
-rw-r--r--  dict-vietphrase.db  160083968
```
✅ Chỉ ba tệp `.db`, không `-wal`/`-shm` nào.

🔴 **Phát hiện lệch với §Quyết định #8 — cần Ice xem *(mục mới, xem §Câu hỏi cho Ice #4)*:**
`--layer all` dựng LẠI cả `dict-core.db` — không tránh được, vì Task 5 (AC1/AC4) đòi hàng
`dict_meta('layer', 'base')` được thêm vào **CẢ BA** tệp qua **một** `insert_meta` dùng
chung, kể cả base. Đây là **1 dòng dữ liệu mới**, ⛔ không phải đổi `schema.rs` hay năm
parser nền (đúng như §Quyết định #8 mô tả) — nhưng nó vẫn làm SHA-256 lệch so với số Story
1.9 đã ghi ở `dict-manifest.toml` (`358cf0f8afcc52c210caa205cd1b0b175eb9562de1b0917e48850a629cd8bdb5`,
154.836.992 byte). Số MỚI **nhỏ hơn** số cũ **372.736 byte** — không rõ nguyên nhân chính
xác (khả năng cao: thứ tự chèn khác đi làm `ANALYZE`/`VACUUM` nén trang khác đi; **không**
phải do `Cargo.lock` trôi — đã đối chiếu `git diff`, chỉ đổi đúng một dòng `version`).
Đã cập nhật `[base].sha256` trong `dict-manifest.toml` sang số MỚI (khớp tệp THẬT sẽ
lên release) thay vì giữ số cũ đã lỗi thời — giữ số cũ sẽ là một checksum SAI trong
manifest, đúng thứ AD-25/dict-manifest.toml:14 cấm.

#### Task 7/8 — Đỏ-rồi-xanh bằng tay, ba cổng `.mjs`

**`check-dict-manifest.mjs`** — RED (xoá tạm khối `[[detachable]]` của `vietphrase`):
```
[31mFAIL[0m [[detachable]] có 1 mục, cần ĐÚNG 2 mục (["thieu-chuu","vietphrase"]) — không dư không thiếu
1 phép kiểm thất bại.
```
GREEN (khôi phục `dict-manifest.toml` thật): `Tất cả phép kiểm dict-manifest.toml đạt.`

**`check-dict-build.mjs` — Kiểm D** — RED tự nhiên trước khi điền manifest (ghi lại nguyên
văn ở Task 8, cùng phiên chạy này): `FAIL DETACHABLE_ALL có mã lớp KHÔNG có mục
[[detachable]] tương ứng trong manifest: ["thieu-chuu","vietphrase"]`. GREEN sau khi điền
manifest (Task 7): `OK dict-manifest.toml [[detachable]] khớp CHÍNH XÁC DETACHABLE_ALL`.

**`check-dict-build.mjs` — Kiểm E** — RED (thêm tạm `const _TEMP_TEST: &str = "dict-core";`
vào `sources/vietphrase.rs`):
```
[31mFAIL[0m 1 lần dùng token 'dict-core'/'dict_core' không có miễn trừ hợp lệ trong đường dựng lớp gỡ rời:
       tools/dict-build/src/sources/vietphrase.rs:180 — token 'dict-core' — const _TEMP_TEST: &str = "dict-core";
```
GREEN sau khi gỡ dòng test (không phải một phần của mã thật): `Tất cả phép kiểm
tools/dict-build đạt.` — `cargo build` xác nhận cây biên dịch sạch sau khi khôi phục.

#### Task 11 — Bảng kế toán NFR6 và phán quyết

**Điều kiện tái dùng baseline `.dmg`/license** (§Quyết định #8) — 🔄 **SỬA sau lượt code review 2026-08-05, số cũ khai SAI:**

```
$ git diff --stat -- src-tauri/ src/ package.json src-tauri/Cargo.lock
 src-tauri/resources/dict/README.md | 14 +++++++++++---
 1 file changed, 11 insertions(+), 3 deletions(-)
```

⇒ **KHÔNG phải 0 dòng** — `+11/−3`. Bản ghi cũ ghi *"0 dòng đổi ✅"* là sai sự thật.

✅ **Tái dùng vẫn HỢP LỆ, nhưng vì một lý do khác** phải nói ra: `src-tauri/resources/dict/**` ⛔ **không nằm trong `bundle.resources`** của `tauri.conf.json` *(chỉ `resources/fonts/*` + `resources/license/*` — chính là hệ quả của Task 10 Story 1.9, đã ghi ở `deferred-work.md`)*, nên tệp README đó ⛔ không đi vào `.dmg` và số **2.334.696** + **35.149** không đổi. ⚠️ Nếu một lượt sau chạm bất kỳ tệp nào **thật sự** nằm trong `bundle.resources` hoặc trong `src-tauri/src`, điều kiện này ⛔ không còn — phải dựng lại `.dmg` và đo lại.

| Dòng | Nguồn số |
|---|---:|
| Baseline `.dmg` không font/license | **2.334.696** — Story 1.9, tái dùng *(điều kiện đã kiểm — xem khối `git diff --stat` ngay trên, ⚠️ hợp lệ vì `resources/dict/**` ⛔ không nằm trong `bundle.resources`, ⛔ không phải vì "0 dòng đổi")* |
| License trong bundle | **35.149** — Story 1.9, tái dùng |
| Bộ font | **21.285.713** — `font-spike-results-2026-08-03.md:82` |
| `dict-core.db` | **154.464.256** — 🔴 **đo LẠI thật** ở story này, ⛔ không phải số Story 1.9 tái dùng — xem giải thích ở Task 10 và §Câu hỏi cho Ice #4 |
| `dict-thieu-chuu.db` | **5.787.648** — đo thật, story này |
| `dict-vietphrase.db` | **160.083.968** — đo thật, story này |
| Hai lớp chưa dựng *(HVTĐTD, Cổ hán văn)* | `[----] chưa đo — story nối tiếp` |
| WebView2 Runtime nhúng | **`[----]` không áp dụng** — build macOS, ⛔ không có `.msi`; Windows chưa đo *(khớp đúng chữ Story 1.9 `:812`)*. Dòng riêng, ⛔ không cộng vào tổng *(NFR6 sửa 2026-08-03)* |
| **Tổng payload sản phẩm hôm nay** | **343.991.430 byte** *(343,99 MB thập phân)* |
| Đối chiếu trần 200.000.000 byte | 🔴 **VƯỢT — 143.991.430 byte** *(≈ 143,99 MB)* |

⚠️ Kết luận VƯỢT hôm nay chỉ tính **hai** lớp gỡ rời — hai lớp còn để trống. ⛔ Không viết
*"NFR6 đã đóng"*.

🔴 **VietPhrase một mình chiếm 160.083.968 byte — VƯỢT XA ước tính §Quyết định #7 của
story** *(ước ≈ 63,7 MB dựa trên hệ số 2,67× của Story 1.9; số thật gấp **6,7×** dữ liệu
thô 23.844.586 byte, không phải 2,67×)*. Lý do nhiều khả năng: đầu mục VietPhrase là
**cụm/câu dài** (không phải từ đơn như CVDICT/CEDICT), và MỖI đầu mục đi qua **cả ba**
chỉ mục FTS5 *(entry_fts trigram + sense_fts + sense_fts_nd, đều external-content nhân
đôi dữ liệu gloss)* — chi phí theo tỷ lệ ký tự cao hơn nhiều so với hệ số đo trên dữ liệu
Trung-Việt ngắn gọn có pinyin. Đây là **số đo thật**, ⛔ không phải suy diễn.

**Đòn bẩy — kèm số** *(⛔ dev không tự áp dụng cái nào, liệt kê để Ice quyết)*:

| Đòn bẩy | Byte tiết kiệm được |
|---|---:|
| Bỏ lớp **VietPhrase** khỏi bản phát hành | **160.083.968** byte ⇒ tổng còn **183.907.462** byte — **ĐẠT** trần *(còn dư 16.092.538 byte cho hai lớp tương lai)* |
| Bỏ lớp **Thiều Chửu** khỏi bản phát hành | **5.787.648** byte ⇒ tổng còn **338.203.782** byte — **vẫn VƯỢT** |
| Bỏ `sense_fts_nd` ở lớp gỡ rời | ⛔ **KHÔNG khuyến nghị dù tiết kiệm bao nhiêu** — phá AC4 *(§Câu hỏi #1 phương án (b), đã bị story tự loại)* |
| Hai khoản `deferred-work.md:75` *(`reqwest` default features tắt + `crate-type`)* | **[----] chưa đo** — tác động lên **binary thực thi**, ⛔ không lên `.db`; cần đo riêng, ⛔ không suy đoán |
| Nâng trần NFR6 lên ví dụ **360.000.000** byte | Đủ chỗ cho tổng hôm nay **+ dư ~16 MB** cho hai lớp còn lại *(ước rất thô, chưa đo)* |

**Phán quyết cuối: 🔴 VƯỢT — 143.991.430 byte trên trần 200.000.000 byte.** Đây là kết
luận **CUỐI CÙNG và ĐỦ** *(§Quyết định #7 của story: "VƯỢT thì là kết luận cuối cùng và
đủ — thêm lớp chỉ làm nó vượt xa hơn")*. ⛔ Dev **không** tự bỏ nguồn nào, không tự bỏ chỉ
mục, không sửa `[profile.release]` hay `deferred-work.md:75`. **DỪNG** — quyết định tầng
PRD, chuyển cho Ice ở §Câu hỏi cho Ice #1 (mặc định nếu Ice không trả lời: phương án (a),
đã áp dụng ở story này — đo, ghi VƯỢT, dừng).

### Completion Notes List

- ✅ Hai lớp gỡ rời **Thiều Chửu** (public-domain/CC0-1.0) và **VietPhrase** (unknown) giao đầy đủ AC1–AC5. AC6 (NFR6) đóng với phán quyết **VƯỢT** — hợp lệ theo đúng chữ story, không phải một thất bại.
- 🔴 **Lệnh chép-dán cho Ice — tải ba tệp lên release `dict-v1`** *(release chưa tồn tại — tạo MỘT lần với cả ba, §Quyết định #6)*:
  ```sh
  gh release create dict-v1 \
    tools/dict-build/out/dict-core.db \
    tools/dict-build/out/dict-thieu-chuu.db \
    tools/dict-build/out/dict-vietphrase.db \
    --title "dict-v1" \
    --notes "Dữ liệu từ điển AuraTranslate — lớp nền (5 nguồn) + 2 lớp gỡ rời (Thiều Chửu, VietPhrase). Xem dict-manifest.toml để biết checksum."
  ```
  Nếu release `dict-v1` **đã được tạo** (Ice đã chạy lệnh của Story 1.9 trước khi đọc note này), dùng thay:
  ```sh
  gh release upload dict-v1 \
    tools/dict-build/out/dict-thieu-chuu.db \
    tools/dict-build/out/dict-vietphrase.db \
    --clobber   # nếu dict-core.db trên release đã cũ, xem cảnh báo dưới
  ```
  ⚠️ **`dict-core.db` trên máy (154.464.256 byte, SHA-256 `741e166673...` (sau lượt dựng lại 2026-08-05T09:33)) KHÔNG khớp
  tệp đã ghi trong `dict-manifest.toml` TRƯỚC story này** (`358cf0f8afcc5...`,
  154.836.992 byte) — xem §Câu hỏi cho Ice #4. Nếu release cũ đã có `dict-core.db` với
  checksum CŨ, `gh release upload ... --clobber` cần **cả ba** tệp để tránh lệch.
- 🔴 **Dữ kiện MỚI so với `prd.md` — Ice cần cập nhật tài liệu quy hoạch** *(dev ⛔ không tự sửa `prd.md`/`epics.md`, tiền lệ quyết định #3 của Ice ở Story 1.3)*:
  1. `epics.md` §Story 1.10 ghi **bốn** lớp; story này giao **hai** *(Ice đã chốt 2026-08-05 trong chính story này, nhưng `epics.md` chưa phản ánh)*.
  2. Thiều Chửu: bản số hoá **CC0 1.0 Universal** (đã đối chiếu SHA-256 byte-for-byte với `catusf/tudien@2.2`) ⇒ hạ rủi ro **R7** và giả định **`[A3]`** của `prd.md` §8.6 xuống mức không còn áp cho bản này.
  3. NFR6 **VƯỢT trần THẬT** (không còn là dự báo) — 343.991.430 byte so với trần 200.000.000. Xem §Câu hỏi cho Ice #1.
- ⚠️ FR36 *(gỡ một nguồn = xoá một file)* **CHƯA được nghiệm thu hành vi** — story này chỉ giao điều kiện cấu trúc (AC1 + AC4). Nghiệm thu hành vi thật *("xoá file, chạy lại bộ test tra cứu")* là **Story 1.13**, đã ghi vào `deferred-work.md`.
- ⚠️ `dict_citation` lần đầu có dữ liệu (263 hàng, từ Thiều Chửu) — trích dẫn tác giả dạng `(Tên 漢字)` được bóc bằng heuristic hẹp, có chủ ý; phần không khớp mẫu vẫn nằm nguyên trong `gloss` (không mất dữ liệu, chỉ không được cấu trúc hoá thành `dict_citation`).
- Không sửa `prd.md` / `epics.md` / `ARCHITECTURE-SPINE.md` / story 1.9 (`done`) / `docs/dics/**` — xác nhận qua `git status`/`git diff --stat`.

### File List

**Mới:**
- `tools/dict-build/src/sources/thieu_chuu.rs`
- `tools/dict-build/src/sources/vietphrase.rs`
- `tools/dict-build/tests/layers.rs`
- `tools/dict-build/tests/fixtures/raw/thieu_chuu/TudienThienChuu.txt`
- `tools/dict-build/tests/fixtures/raw/vietphrase/VietPhrase.txt`
- `tools/dict-build/assets/licenses/CC0-1.0.txt`
- `tools/dict-build/assets/licenses/thieu-chuu.txt`
- `tools/dict-build/assets/licenses/vietphrase.txt`

**Sửa:**
- `tools/dict-build/Cargo.toml` (version 0.2.0) · `tools/dict-build/Cargo.lock`
- `tools/dict-build/src/main.rs` (CLI `--out-dir`/`--layer`)
- `tools/dict-build/src/build.rs` (`run_base` đổi tên từ `run`, `run_detachable_layer`, `run_detachable_by_code`, `run_all`, `DETACHABLE_LAYERS`)
- `tools/dict-build/src/finalize.rs` (`prepare_fresh_output`, `finish` — đuôi dùng chung)
- `tools/dict-build/src/insert.rs` (`insert_meta` nhận thêm `layer: &str`)
- `tools/dict-build/src/licenses.rs` (+ `CC0_1_0`, `THIEU_CHUU_DECLARATION`, `VIETPHRASE_DECLARATION`, `thieu_chuu_license_text()`, `vietphrase_license_text()`)
- `tools/dict-build/src/sources_meta.rs` (`BASE_ALL`/`DETACHABLE_ALL`, `THIEU_CHUU`, `VIETPHRASE`, `LicenseRef` +2 biến thể)
- `tools/dict-build/src/sources/mod.rs` (+2 module)
- `tools/dict-build/tests/parse.rs` (`build::run` → `build::run_base`, +2 test tích hợp)
- `tools/dict-build/tests/schema.rs` (`insert_meta(&conn, "base")`)
- `tools/dict-build/README.md`
- `scripts/check-dict-build.mjs` (`RS_FILE_FLOOR` 10→18, +Kiểm D, +Kiểm E)
- `scripts/check-dict-manifest.mjs` (`[[detachable]]` đòi đúng 2 mục)
- `dict-manifest.toml` (2 khối `[[detachable]]` điền thật, `[base].sha256` cập nhật)
- `src-tauri/resources/dict/README.md`
- `_bmad-output/implementation-artifacts/deferred-work.md`
- `_bmad-output/implementation-artifacts/sprint-status.yaml`
- `_bmad-output/implementation-artifacts/1-10-dong-goi-bon-lop-go-roi-thanh-file-doc-lap.md` (chính story này)

**Không commit (`.gitignore`, artifact tải về):**
- `tools/dict-build/raw/thieu_chuu/TudienThienChuu.txt` · `tools/dict-build/raw/vietphrase/VietPhrase.txt`
- `tools/dict-build/out/dict-core.db` · `dict-thieu-chuu.db` · `dict-vietphrase.db`

### Review Findings

> Lượt `bmad-code-review` 2026-08-05 — ba lớp song song (Blind Hunter · Edge Case Hunter · Acceptance Auditor), tất cả chạy ở cấp Opus. Diff review: cây làm việc chưa commit + tệp mới, đối chiếu `HEAD = a3ed5cd`.
>
> 🟢 **Fixture SẠCH** — 23/23 dòng Thiều Chửu và 20/20 dòng VietPhrase khớp **nguyên văn** (`grep -F -x`) với nguồn thật; ca bắt buộc đủ (dòng 108 `亯`, rác `()`, nhiều âm `|`). Bài học fixture bịa của Story 1.9 đã được học.
> 🟢 **AC1 · AC2 · AC4 · AC5 ĐẠT** trên tệp THẬT: `sqlite_master` băm giống hệt ba tệp (`b64c6f10…a156`), đối chứng âm hai chiều có test, SHA-256 manifest khớp `shasum`. Bẫy 1–9 và §Quyết định #1–#6 đều đạt. `src-tauri` giữ đúng **62** test, **0** crate mới.

**Cần quyết (`decision-needed`)**

- [x] [Review][Decision] **`dict_citation.author` chứa TÊN TÁC PHẨM, không phải tác giả — vi phạm điều cấm tường minh của Task 6** — `extract_citation` (`tools/dict-build/src/sources/thieu_chuu.rs:128-150`) lấy chuỗi trong cặp ngoặc ĐẦU TIÊN, chấp nhận nếu ký tự đầu viết hoa + có ≥1 chữ Hán, rồi gán phần trước chữ Hán vào `author` với `work = None`. Trên `dict-thieu-chuu.db` thật: **263 hàng, 105 giá trị `author` phân biệt**, phổ biến nhất là `Luận ngữ` (27) · `Mạnh Tử` (22) · `Thi Kinh` (17) · `Thư Kinh` (10) · `Tiêu dao du` (8) · `Thuật nhi` (8) · `Hương đảng` (5) — **tên sách/tên thiên, không phải người**; chỉ ~27/263 ≈ 10% là tên người. Ca có sẵn trong fixture đã commit (`tests/fixtures/raw/thieu_chuu/TudienThienChuu.txt:22`, chữ `关`): tác giả THẬT `Nguyễn Du` nằm NGOÀI ngoặc, trong ngoặc là tên bài `Thăng Long` ⇒ code ghi `author = "Thăng Long"`. Task 6 nói thẳng: *"Nếu bóc tách quá giòn, để nguyên trong `gloss` là **chấp nhận được**; ⛔ bịa `work`/`author` thì **không**."* Doc-comment `:126` tự tuyên bố *"⛔ không bịa `work`"* trong khi đang bịa `author` — cột nguy hiểm hơn vì nó hiện lên UI ở 1.11/1.13. Đây là **toàn bộ** dữ liệu `dict_citation` của dự án. **Ba đường ra:** (a) bỏ trích dẫn hẳn, để nguyên trong `gloss` — đúng chữ Task 6, `dict_citation` về 0 hàng; (b) chuyển giá trị sang cột `work`, `author = NULL` — giữ dữ liệu, gán đúng cột, ⛔ không đổi lược đồ; (c) siết heuristic bằng danh sách tên người đã biết. **Khuyến nghị (b).**
- [x] [Review][Decision] **`sha256` trong `dict-manifest.toml` KHÔNG tái lập được — `built_at` mili-giây làm mọi lượt build ra hash khác nhau** — `insert_meta` ghi `strftime('%Y-%m-%dT%H:%M:%fZ','now')` (`tools/dict-build/src/insert.rs:119`). Đã chứng minh thực nghiệm: hai lượt build liên tiếp từ **cùng** một cây fixture cho ra 6 hash khác nhau đôi một. Release `dict-v1` **chưa tồn tại** (`dict-manifest.toml:23-30`), nên ba dòng `sha256` hôm nay mô tả những tệp chưa ai từng tải lên. Nếu Ice `cargo run` lại một lần trước khi upload — hoàn toàn bình thường sau một `git pull` — cả ba hash sai 100%, mọi máy khách fail checksum, và ⛔ **không cổng nào bắt được** (`check-dict-manifest.mjs` cố ý không đọc `.db`). Điều này vô hiệu hoá **AD-25** trên thực tế. Kết hợp với phán quyết NFR6 **VƯỢT** (quyết định về VietPhrase còn treo), khoảng thời gian giữa "điền manifest" và "upload thật" có thể dài. **Hai đường ra:** (a) làm build tất định — bỏ `built_at` khỏi tệp, hoặc nhận `SOURCE_DATE_EPOCH`, hoặc dẫn xuất từ `source_version`; (b) giữ nguyên nhưng ghi thành **luật vận hành** ở `tools/dict-build/README.md` + §Completion Notes: *"điền `sha256` từ chính lượt build sẽ upload, ⛔ không build lại sau khi điền"*. **Khuyến nghị (a)** — (b) là một lời hứa không có lưới.
- [x] [Review][Decision] **`dict-core.db` bị dựng lại và `[base].sha256` bị sửa — lý do dev nêu ở §Câu hỏi #4 KHÔNG đúng** — §Ranh giới phạm vi dòng *"Dựng lại `dict-core.db` \| ❌ **Không**"* và bảng AC6 chốt cứng `154.836.992`. Thực tế: tệp mới **154.464.256 byte**, hash `e0b12718…31ab3`, manifest `[base].sha256` đã bị đổi theo. Dev khai *"`--layer all` … không tránh được"* — nhưng CLI mới **có** `--layer thieu-chuu` và `--layer vietphrase` chạy đơn lẻ (`build.rs::run_detachable_by_code`), dựng được hai lớp mà ⛔ không chạm base. Đây là lỗi ở **lệnh đã chạy**, không phải ở kiến trúc. *(Ghi nhận: Task 10 lại kê đúng lệnh không có `--layer` ⇒ mặc định `all` — spec tự mâu thuẫn với §Quyết định #8.)* Hậu quả số học nhỏ: chênh 372.736 byte, ⛔ không đổi phán quyết VƯỢT. **Hai đường ra:** (a) chấp nhận tệp mới + checksum mới (lựa chọn dev đã áp dụng), sửa bảng AC6 + §Quyết định #8 cho khớp; (b) khôi phục `dict-core.db` của Story 1.9 và trả `[base].sha256` về `358cf0f8…bdb5`. **Khuyến nghị (a)** — checksum phải khớp tệp THẬT sẽ lên release; nhưng câu chữ *"không tránh được"* cần sửa lại cho đúng.
- [x] [Review][Decision] **`docs/` — 313 MB nhị phân chưa track, chưa `.gitignore`, mà `README.md` mới biến nó thành phụ thuộc bắt buộc của quy trình dựng** — `tools/dict-build/README.md` hướng dẫn `cp "docs/dics/Thieu chuu/TudienThienChuu.txt" …`. Trạng thái thật: `git status` cho `?? docs/`, `grep -n docs .gitignore` **không có kết quả**, thư mục chứa `tudien-2.2.zip` 289.049.176 byte + `VietPhrase.txt` 23.844.586 byte + `hanodict_tbl_dictionary.pbix` 31.229.506 byte. Hai hậu quả: (1) một `git add -A` vô ý nhét ~344 MB nhị phân vào lịch sử git **vĩnh viễn** — không có gì chặn; (2) người thứ hai clone repo làm theo README sẽ `cp` thất bại vì `docs/dics/` không tồn tại ở đâu cả, và README ⛔ không nói tệp đó lấy ở đâu ra. **Hai đường ra:** (a) thêm `docs/dics/` vào `.gitignore` + README ghi rõ *"kho cục bộ của Ice, ⛔ không có trong repo"* kèm nguồn tải; (b) commit `docs/dics/` thật (kể cả `tudien-2.2.zip` — nó **là bằng chứng giấy phép CC0**, story nói ⛔ không xoá). **Khuyến nghị (a).**

**Cần vá (`patch`)**

- [x] [Review][Patch] Task 6 đánh `[x]` nhưng CHƯA làm: `split_senses` chỉ tách `<br>`, ⛔ không tách theo số thứ tự — `丐` ra 1 `dict_sense` thay vì 2, `下` ra 3 thay vì 4; 22 nghĩa bị gộp trên dữ liệu thật; `ord` là chỉ số 0-based chứ ⛔ không "theo số" như spec đòi; cả hai ca đã nằm sẵn trong fixture mà ⛔ không test nào bắt [tools/dict-build/src/sources/thieu_chuu.rs:86]
- [x] [Review][Patch] `--layer all` hỏng giữa chừng PHÁ HUỶ tệp `.db` cũ còn tốt và để lại `.tmp` mồ côi — `prepare_fresh_output` xoá `out_path` TRƯỚC khi `File::open(raw_file_path)`; đã chứng minh: thiếu `raw/vietphrase/` ⇒ `dict-vietphrase.db` biến mất, còn lại `.tmp` 118 KB, out-dir trộn thế hệ mà ⛔ không dấu hiệu nào [tools/dict-build/src/build.rs:401, tools/dict-build/src/finalize.rs:26]
- [x] [Review][Patch] Kiểm D XANH khi `DETACHABLE_ALL` rỗng — cả hai nhánh fail đều bị chặn bởi `rustDetachableCodes.length > 0`; đã mô phỏng: hai lớp biến mất khỏi Rust mà cổng chống-trôi vẫn báo "Tất cả phép kiểm đạt". Vi phạm chính doctrine *"cây rỗng ⛔ không được đọc thành sạch"* viết ở đầu script [scripts/check-dict-build.mjs:301]
- [x] [Review][Patch] Kiểm D canh `DETACHABLE_ALL` trong khi đường dựng THẬT dùng `DETACHABLE_LAYERS` — hai danh sách ⛔ không mã/test/cổng nào ràng buộc; quên `DETACHABLE_LAYERS` ở story nối tiếp ⇒ mọi cổng xanh, `--layer all` im lặng thiếu một tệp, manifest công bố một tệp không tồn tại. Đây đúng là "một lớp BỊ RƠI MẤT" mà AC5 nói cổng phải bắt [scripts/check-dict-build.mjs:236, tools/dict-build/src/build.rs:339]
- [x] [Review][Patch] Kiểm E ⛔ không có sàn số tệp — `isolationFiles` là ba đường dẫn viết cứng; đổi tên `sources/thieu_chuu.rs` ⇒ quét 0 tệp ⇒ in **OK**. Cùng lỗ hổng mà Kiểm C của chính script này tồn tại để chặn [scripts/check-dict-build.mjs:319]
- [x] [Review][Patch] AC3 — `vietphrase.license_id` là chuỗi rỗng `''`, ⛔ không phải `NULL` như bảng AC3 chốt; và test bị nới `assert!(license_id.is_none() || license_id == Some(String::new()))` để chấp nhận cả hai. Sửa `license_id` sang `Option<&'static str>` được mà ⛔ không đụng `schema.rs` [tools/dict-build/src/sources_meta.rs, tools/dict-build/tests/layers.rs:165]
- [x] [Review][Patch] `check-dict-manifest.mjs` ⛔ không ràng buộc `url` với `name` của chính mục đó, và cho phép hai mục dùng CHUNG `url`/`sha256` — hoán đổi url giữa hai `[[detachable]]` ⇒ mọi cổng xanh, người dùng tải `dict-thieu-chuu.db` nhận nội dung VietPhrase. Quy tắc `dict-<code>.db` đã cố định trong `build::output_file_name` nên kiểm được [scripts/check-dict-manifest.mjs:183]
- [x] [Review][Patch] `RS_FILE_FLOOR = 18` trong khi số thật là **20** — khoảng hở bằng ĐÚNG hai tệp mà story này thêm vào; xoá cả `thieu_chuu.rs` lẫn `vietphrase.rs` vẫn xanh [scripts/check-dict-build.mjs:54]
- [x] [Review][Patch] `SOURCE_CODE` của hai module mới là hằng CHẾT (⛔ không nơi nào tham chiếu, khác hẳn năm nguồn nền) và `SOURCE_VERSION` bị chép tay hai nơi mà ⛔ không cổng nào so với manifest — sửa một trong hai chỗ, mọi test/cổng vẫn xanh [tools/dict-build/src/sources/thieu_chuu.rs:10, tools/dict-build/src/sources/vietphrase.rs:13]
- [x] [Review][Patch] `is_han` sao chép nguyên bảy dải từ `char_idx.rs`, bỏ hết comment giải thích — hai hàm cùng tên cùng nội dung sẽ trôi khỏi nhau khi bổ sung CJK Ext H/I [tools/dict-build/src/sources/thieu_chuu.rs:152, tools/dict-build/src/char_idx.rs:9]
- [x] [Review][Patch] `thieu_chuu.rs` ⛔ không lột BOM, bất đối xứng với `vietphrase.rs:32-37` — `raw.trim()` KHÔNG bỏ U+FEFF (ký tự `Cf`), nên một lượt `iconv` để lại BOM sẽ tạo headword `"\u{feff}一"`: ⛔ không rỗng, ⛔ không lỗi, một đầu mục vĩnh viễn không tra ra được. README mới cảnh báo đúng ca này [tools/dict-build/src/sources/thieu_chuu.rs:26]
- [x] [Review][Patch] CLI ⛔ không có một test nào dù mở rộng CLI là Task 4 — `--out-dir --layer` (thiếu giá trị) làm `create_dir_all("--layer")` tạo thật một thư mục rồi dựng đủ ba `.db` vào đó với `ExitCode::SUCCESS`; cờ lặp lại nhận âm thầm; `run_detachable_by_code` là `pub` nhưng ⛔ không `create_dir_all` (khác `run_all`); và §Bẫy 7 *"hỏng nếu BẤT KỲ lớp nào thiếu nguồn"* ⛔ không có test — chính lượt xác minh tay đã lộ ra lỗi phá huỷ tệp ở trên [tools/dict-build/src/main.rs:26, tools/dict-build/src/build.rs:372]
- [x] [Review][Patch] Năm chỗ câu chữ sai/lỗi thời: §Debug Log Task 11 khai `git diff --stat` = *"0 dòng đổi ✅"* trong khi thật là `+11/−3` ở `src-tauri/resources/dict/README.md` *(số `.dmg` vẫn tái dùng được vì `resources/dict/**` ⛔ không nằm trong `bundle.resources`, nhưng bản ghi sai sự thật và nó là chứng cứ DUY NHẤT cho hai dòng đầu bảng AC6)* · AC6 bỏ trống số byte dòng WebView2 dù Story 1.9 đã có · Task 8 còn ghi *"Kiểm D hiện ĐANG FAIL có chủ ý"* trong khi đã xanh · `Cargo.toml:12` `description` vẫn nói *"gộp năm nguồn thành dict-core.db"* ngay tại commit nâng lên `0.2.0` · `deferred-work.md` hứa thêm một lớp chỉ là *"3 chỗ"* trong khi thật là ≥10 chỗ (`sources_meta.rs` ×4 kể cả test hardcode `2` · `licenses.rs` · `sources/mod.rs` · `build.rs:339` · manifest · `check-dict-manifest.mjs:234` · `check-dict-build.mjs:54`+`:319` · `layers.rs:55`+`:70` · usage `main.rs:67` · README) [_bmad-output/implementation-artifacts/1-10-dong-goi-bon-lop-go-roi-thanh-file-doc-lap.md, tools/dict-build/Cargo.toml:12]

**Hoãn (`defer`)**

- [x] [Review][Defer] `require_nonempty` chỉ chặn ĐÚNG mốc 0 entry — ⛔ không ngưỡng tỉ lệ bỏ dòng [tools/dict-build/src/build.rs:66] — deferred, hạ tầng Story 1.9
- [x] [Review][Defer] VietPhrase: 46 đầu mục trùng trong nguồn thô (18 trong tệp đã dựng) ⛔ không được gộp, ngược khuôn Group A của Story 1.9 [tools/dict-build/src/sources/vietphrase.rs:19] — deferred, khớp mô hình "mọi dòng là một mục" mà spec chốt
- [x] [Review][Defer] VietPhrase tách `/` vô điều kiện — `và/hoặc`, `24/7`, URL trong nghĩa bị bẻ thành nhiều `dict_sense` giả [tools/dict-build/src/sources/vietphrase.rs:77] — deferred, spec chốt tách `/`
- [x] [Review][Defer] §Bẫy 8 — hai vế *(điền manifest + siết cổng)* phải cùng MỘT commit, chưa xác minh được vì story chưa commit gì [dict-manifest.toml, scripts/check-dict-manifest.mjs] — deferred, điều kiện còn treo tới lượt commit

---

### ✅ DỰNG LẠI SAU LƯỢT VÁ — hoàn tất 2026-08-05T09:33, AC5 đóng lại

Bản vá **đổi dữ liệu**, ⛔ không chỉ đổi mã: `split_senses` cho ra **22.681** `dict_sense`
thay vì 22.658 — **+23 nghĩa** được cứu, đúng bằng phép đo của lượt review. Ba tệp trong
`out/` và ba `sha256` trong manifest vì thế mô tả bản dựng bằng parser CŨ ⇒ AC5 *("giá trị
**thật** từ chính lượt build")* tạm thời không đạt. Đã dựng lại đủ ba tệp và điền lại
manifest.

**Lượt dựng lại — `--layer all`, một lượt chạy, ba tệp:**

| Tệp | SHA-256 | Byte | Đổi so với trước? |
|---|---|---:|---|
| `dict-core.db` | `741e1666…0a34` | **154.464.256** | hash đổi *(chỉ `built_at`)*, byte **y nguyên** |
| `dict-thieu-chuu.db` | `e9417c12…c9d5` | **5.787.648** | hash đổi, **+23 sense**, byte **y nguyên** |
| `dict-vietphrase.db` | `9d304210…f735` | **160.083.968** | hash đổi *(chỉ `built_at`)*, byte **y nguyên** |

⇒ 🟢 **Bảng kế toán AC6 và phán quyết VƯỢT 143.991.430 byte ⛔ KHÔNG ĐỔI** — cả ba kích
thước giữ nguyên từng byte.

**`SourceStats` lượt dựng lại:** cvdict 122.596 entry · cc-cedict 124.758 · unihan 49.870 ·
viwiktionary 1.598 · en-wiktionary 174.677 · **thieu-chuu 9.897 entry / 22.681 sense / 263
citation** · **vietphrase 679.302 entry / 805.558 sense** *(9 dòng rác bỏ, đúng như khảo sát)*.

**Nghiệm thu lại trên ba tệp THẬT vừa dựng:**

```
AC4  sqlite_master băm  b64c6f107a14f2be  — GIỐNG NHAU cả ba, user_version = 1 cả ba
AC1  dict-core chứa mã lớp gỡ rời         → 0
     mỗi tệp gỡ rời chứa mã khác          → 0 · 0
AC3  thieu-chuu | public-domain | 'CC0-1.0' | text
     vietphrase | unknown       | NULL     | null   ← 🟢 NULL THẬT, không còn chuỗi rỗng
     ls -la out/ → 0 tệp -wal / -shm
```

🟢 **Tái lập xác nhận ba lần:** `dict-thieu-chuu.db` cho **đúng** hash `e9417c12…c9d5` qua
ba lượt dựng vào **ba thư mục đích khác nhau**. `built_at` giờ là `2026-08-04T23:53:16Z`
— **cùng một giá trị ở cả ba tệp**, dẫn xuất từ mtime nguồn thô, ⛔ không từ đồng hồ.
Từ đây, `cargo run` lại ⛔ không còn làm hỏng manifest.

---

**Đã bác (`dismiss`, 7)** — luật lọc rác `"()"` *(đã đo: 9/9 dòng rác thật khớp chính xác, 0 biến thể; spec cấm bộ lọc phức tạp cho tỷ lệ 0,001%)* · test `license_kind_column_accepts_a_value_outside_the_open_license_set` "luôn xanh" *(cột TEXT không CHECK CHÍNH LÀ điều AC3 đòi chứng minh)* · `assert_ne!` thừa sau `assert_eq!` *(đối chứng âm mang tính tài liệu, spec đòi)* · dòng 4 cột bị vứt cả dòng *(⛔ không tồn tại trong dữ liệu: 9.897 + 1 hỏng = 9.898 khớp tuyệt đối)* · `strip_leading_ordinal` cắt nhầm `"1942."` *(giả thuyết, mọi mảnh thật đều bắt đầu bằng số thứ tự)* · va chạm tên tệp nếu một lớp gỡ rời mang `code = "core"` *(giả thuyết)* · vòng lặp `ParseIssue` vô hạn khi `BufRead::lines()` trả `Err` lặp *(giả thuyết)*

---

## Change Log

| Ngày | Thay đổi |
|---|---|
| 2026-08-05 | ✅ **Ice chốt: CHẤP NHẬN VƯỢT trần NFR6** — 343.991.430 / 200.000.000 byte, vượt **143.991.430 byte**, chấp nhận trên số đo thật. ⛔ Không bỏ nguồn · ⛔ không bỏ `sense_fts_nd` · ⛔ không đụng `Cargo.toml` *(lần thứ năm)* · ⛔ không subset font. §Câu hỏi #1 và `deferred-work.md` [D4] cập nhật theo. 🔴 **Còn lại ở tầng PRD (chủ sở hữu: Ice):** trần NFR6 mâu thuẫn với sản phẩm thật — nâng trần, hoặc ghi rằng NFR6 ⛔ không tính lớp gỡ rời *(theo cách đọc đó, payload lõi 183.907.462 byte vẫn DƯỚI trần, dư 16.092.538)*. |
| 2026-08-05 | **Lượt code review + áp dụng vá.** Ba lớp song song; fixture xác minh SẠCH (43/43 dòng khớp nguyên văn); AC1/AC2/AC4/AC5 và Bẫy 1–9 ĐẠT trên tệp thật. 4 `decision-needed` *(Ice chốt cả bốn)* · 13 `patch` · 4 `defer` · 7 bác — **17/17 đã vá và xác minh**. Thay đổi hành vi: `split_senses` tách theo cả `<br>` **và** số thứ tự ⇒ **22.681** sense *(+23)*; `dict_citation` chuyển sang cột `work` *(263 hàng, `author` = 0)*; `built_at` dẫn xuất từ nguồn thô ⇒ **build tái lập được** *(AD-25)*; `vietphrase.license_id` là `NULL` thật; `run_all` kiểm nguồn TRƯỚC khi xoá tệp; sàn cho Kiểm D/E; Kiểm F mới *(chống trôi `source_version`)*; `docs/dics/` vào `.gitignore`. **110** test dict-build *(từ 88)*, `src-tauri` giữ đúng **62**, 7/7 cổng xanh. ✅ Đã dựng lại đủ ba tệp và điền lại `sha256` (2026-08-05T09:33) — kích thước cả ba GIỮ NGUYÊN từng byte nên AC6 và phán quyết VƯỢT không đổi; AC1/AC3/AC4 nghiệm thu lại trên tệp thật; tái lập xác nhận qua ba thư mục đích. |
| 2026-08-05 | **Story 1.10 triển khai đầy đủ** — Task 0–12. Hai lớp gỡ rời (Thiều Chửu, VietPhrase) đóng gói thành `dict-thieu-chuu.db`/`dict-vietphrase.db` độc lập, dùng lại nguyên lược đồ base (AC1/AC4, `sqlite_master` byte-identical trên fixture). Metadata giấy phép/ghi công tự mang trong từng tệp (AC2/AC3). `dict-manifest.toml` điền thật, cổng siết đúng hai mục (AC5). NFR6 đóng với phán quyết **VƯỢT** 143.991.430 byte trên trần 200.000.000 (AC6) — số thật, không phải ước tính, chuyển quyết định cho Ice. 88 test Rust `tools/dict-build` (từ 62), `src-tauri` giữ nguyên 62. Phát hiện phụ: `--layer all` dựng lại `dict-core.db` (thêm 1 hàng `dict_meta`), SHA-256 đổi — đã cập nhật `[base]` trong manifest, ghi rõ ở §Câu hỏi cho Ice #4. |
| 2026-08-04 | Story tạo — phân tích context đầy đủ, trạng thái `ready-for-dev`. 🔴 Bốn câu hỏi cho Ice, #1 **chặn** *(nguồn thô cho bốn lớp)* |
| 2026-08-05 | **Ice chuyển `VietPhrase.txt` sang UTF-8 và bỏ BOM.** Kiểm chứng nguyên vẹn: 679.311/679.311 dòng đúng một `=` *(100 %)*, 9 dòng rác và phân bố nghĩa **không đổi**. SHA-256 mới `5cb6a00d…d617`. Task 0 bỏ bước `iconv`; §Bẫy 5 đổi khung từ *"tệp là UTF-16"* sang *"tệp đã sạch, **kho gốc** thì chưa"* — bẫy vẫn còn giá trị cho mọi lượt tải lại về sau |
| 2026-08-05 | **Ice chốt phạm vi: HAI lớp.** Khảo sát thật nguồn Ice tải về ⇒ §Thông tin kỹ thuật đổi từ *"CHƯA KIỂM CHỨNG"* sang **đã kiểm chứng** *(SHA-256, số dòng, ánh xạ cột, ca lỗi thật)*. Thiều Chửu xác nhận **CC0 1.0** ⇒ hạ **R7**/`[A3]`. Câu hỏi chặn #1 **đóng**. Thêm §Bẫy 5 *(UTF-16LE)*; AC/Task/cổng đổi từ bốn lớp sang hai; ước tính NFR6 **VƯỢT** ghi vào §Quyết định #7. Trần Văn Chánh + Trung Việt chuyển sang `docs/dics/_khong-dung/` |
