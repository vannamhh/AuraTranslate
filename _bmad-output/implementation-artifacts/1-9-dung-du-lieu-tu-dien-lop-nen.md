---
baseline_commit: 0ff36a0f8d77cddeee09e32306f0e427438d2e35
baseline_note: 'Cây làm việc tại 0ff36a0 CỘNG toàn bộ thay đổi CHƯA COMMIT của Story 1.8 — xem §Trạng thái repo hiện tại'
---

# Story 1.9: Dựng dữ liệu từ điển lớp nền

Status: ready-for-dev

<!-- Note: Validation is optional. Run validate-create-story for quality check before dev-story. -->

**Covers:** FR27 · FR28 · FR29 · FR30 · FR31 (nửa dữ liệu) · NFR6 · NFR8 · AD-19 · AD-25 · AD-26 · AD-27
**Epic:** 1 — Nền móng ứng dụng & Tra cứu ngoại tuyến tức thì

> 🔴 **Đây là story đầu tiên của cả dự án sinh ra mã KHÔNG nằm trong bản phát hành.** Tám story trước đều viết mã sẽ chạy trên máy người dùng. Story này viết một **build tool** chạy trên máy Ice, và AD-25 nói thẳng lý do nó phải tách ra: *"Parser định dạng từ điển chỉ nằm trong build tool, không vào bản phát hành — nên giấy phép parser không ràng buộc sản phẩm."* Ranh giới đó phải là **hệ quả cấu trúc**, không phải kỷ luật — xem §Quyết định #1.
>
> 🔴 **Và đây là chỗ NFR6 lần đầu có dữ liệu thật để đo.** Tám tháng tài liệu đã chép đi chép lại con số *"130 MB"* của Giai đoạn 0 — một phép đo trên **ba** nguồn, **một** chỉ mục FTS trên nghĩa, và **không** dòng mã sản phẩm nào. Story này đo lại bằng năm nguồn, hai chỉ mục FTS trên nghĩa, và một ứng dụng đã có tầng dữ liệu. Nếu con số vượt trần, ⛔ **không tự sửa** — đó là quyết định tầng PRD, và §Quyết định của Ice đã dựng sẵn chỗ ghi.
>
> ⚠️ **Story này KHÔNG viết một dòng mã tra cứu nào.** Đường tra cứu ba nhánh là **Story 1.11**; cổng `DictionarySource` và việc giữ nguyên bất đồng giữa nguồn là **Story 1.13**; bốn lớp gỡ rời là **Story 1.10**. Story này giao **dữ liệu** và **hình dạng của dữ liệu**. Viết mã đọc hôm nay là viết mã không ai gọi — cùng lỗi mà `core/store/mod.rs:122-134` đã ghi thành luật.

---

## Story

As a chủ dự án,
I want năm nguồn từ điển có giấy phép sạch gộp thành một artifact SQLite có phiên bản và checksum,
So that bản phát hành kiểm chứng được và không parser nào lọt vào sản phẩm.

---

## Acceptance Criteria

### AC1 — `tools/dict-build` sinh `dict-core.db` theo lược đồ đã khai

**Given** nguồn thô CVDICT, Unihan, CC-CEDICT, viwiktionary, en.wiktionary
**When** `tools/dict-build` chạy
**Then** sinh ra `dict-core.db` theo lược đồ `từ khoá → [nguồn, từ loại, nghĩa, ví dụ[], trích dẫn[], ghi chú]`

*Đạt nghĩa là* **năm** mệnh đề, không phải một:

1. **Đúng năm nguồn, không hơn không kém.** Danh sách này là nguyên văn AC của epic và khớp đúng năm hàng **Nền** của `prd.md:888-892`. ⛔ Thiều Chửu · Cổ hán văn · VietPhrase · HVTĐTD **không** thuộc story này — chúng là lớp **gỡ rời** (AD-10) và thuộc **Story 1.10**. Xem §Bẫy 10 về hai danh sách đang mâu thuẫn trong tài liệu.
2. **Lược đồ khai bằng SQL hằng, đọc được cạnh mã**, đúng khuôn `store::schema::SCHEMA_MIGRATION_LOG_DDL` — ⛔ không dựng DDL bằng `format!` từ trạng thái lúc chạy.
3. **Hình dạng bản ghi phủ đủ sáu trường của FR28** — nguồn · từ loại · nghĩa · ví dụ[] · trích dẫn[] · ghi chú. Lược đồ cụ thể ở §Quyết định #2; ⛔ dev không tự thiết kế lại, và ⛔ cũng không được bỏ bớt bảng nào vì *"nguồn hôm nay chưa có dữ liệu cho nó"*.
4. **FR29 là ràng buộc lược đồ, không phải ràng buộc hiển thị:** một từ nhiều từ loại ⇒ **nhiều hàng `dict_sense`**, mỗi hàng có `pos` riêng và tập ví dụ riêng. ⛔ Không gộp nhiều từ loại vào một chuỗi `gloss`.
5. **FR30:** `dict_example` và `dict_citation` treo vào **`sense_id`**, ⛔ không treo vào `entry_id`. Trích dẫn là bảng **riêng** với ví dụ vì nó mang xuất xứ văn bản (`work`, `author`).

### AC2 — Cột `source` bắt buộc, và không tồn tại bước hợp nhất ở bất kỳ đâu

**Given** mọi bản ghi nghĩa
**When** ghi vào database
**Then** cột `source` bắt buộc có giá trị
**And** không tồn tại bước hợp nhất nghĩa giữa các nguồn ở bất kỳ đâu trong build tool

*Đạt nghĩa là* **ba** cơ chế, không phải một — đúng khuôn AC1 của Story 1.8 (*"kiểu + test"*), thêm một vế thứ ba vì đây là bất biến **nội dung** chứ không chỉ bất biến **cấu trúc**:

1. **Lược đồ** — `dict_sense.source_id INTEGER NOT NULL REFERENCES dict_source(id)`, `PRAGMA foreign_keys = ON` lúc dựng. Một hàng nghĩa không nguồn **không chèn được**; cưỡng chế là của SQLite, không của người viết.
2. **Cổng cây nguồn** — `scripts/check-dict-build.mjs` quét `tools/dict-build/src/**` tìm từ vựng hợp nhất (§Task 7). Miễn trừ phải **có tên, có lý do, in ra mỗi lượt chạy** — đúng khuôn `EXEMPT` của `check-i18n.mjs`.
3. **Nghiệm thu bằng dữ liệu thật** — một đầu mục có mặt ở **cả CVDICT lẫn CC-CEDICT** phải cho ra **≥ 2 hàng `dict_sense`** mang **`source_id` khác nhau**, và ⛔ không hàng nào bị nuốt. Test này chạy trên `dict-core.db` **đã dựng thật**, không trên fixture — fixture chứng minh mã đúng, chỉ dữ liệu thật chứng minh **dữ liệu** đúng.

⚠️ *"Không hợp nhất"* ở đây **không** cấm khử trùng lặp **trong lòng một nguồn** (cùng nguồn phát ra hai hàng y hệt do lỗi parse). Ranh giới: khử trùng lặp **trong một `source_id`** là hợp lệ và phải ghi lý do tại chỗ; khử trùng lặp **xuyên `source_id`** là vi phạm AD-19.

### AC3 — Artifact có phiên bản và checksum, và `dict-manifest.toml` là hợp đồng có cổng canh

**Given** `dict-core.db` đã sinh
**When** đẩy lên một GitHub Release có phiên bản
**Then** `dict-manifest.toml` trong repo ghi URL, SHA-256 và phiên bản nguồn thô của file đó

*Đạt nghĩa là* AC này **tách làm hai vế**, và chỉ vế thứ nhất là mã:

1. **Vế cơ chế (dev làm trọn):**
   - Build tool in ra **SHA-256 của chính tệp `.db` vừa sinh** cộng **phiên bản nguồn thô của từng nguồn** (ngày dump / tag / phiên bản Unicode — §Thông tin kỹ thuật), ở một dạng chép thẳng vào manifest được.
   - `scripts/check-dict-manifest.mjs` **đọc và phán quyết** `dict-manifest.toml`: mỗi mục đủ ba trường, `sha256` đúng **64 ký tự hex thường**, `url` là `https://` và trỏ vào một tag dạng `dict-v<N>`, `source_version` không rỗng. Mã thoát là phán quyết. Gắn vào job `check` của `ci.yml`.
   - 🔴 Cổng phải chạy **không cần tệp `.db` có mặt** — `ci.yml` ⛔ không tải dữ liệu từ điển (AC cuối của Story 1.3). Nó kiểm **hình dạng manifest**, không kiểm nội dung tệp.
2. **Vế phát hành (Ice làm tay):** tạo tag `dict-v1`, tải các tệp `.db` lên GitHub Release của `vannamhh/AuraTranslate`, rồi điền URL thật vào manifest. Dev **chuẩn bị đủ mọi con số** để việc đó là chép-dán, ⛔ không tự tạo release và ⛔ không điền URL đoán trước.

⛔ **Không điền giá trị giả để "cho có".** `dict-manifest.toml:16-17` đã viết sẵn lý do: *"Một checksum sai trong manifest hỏng im lặng đúng kiểu tệ nhất: file tải về vẫn dùng được, chỉ là không ai biết nó là bản nào."*

### AC4 — Không parser nào trong cây phụ thuộc của bản phát hành

**Given** parser định dạng từ điển
**When** kiểm cây phụ thuộc của bản phát hành
**Then** không parser nào có mặt — chúng chỉ sống trong `tools/dict-build`

*Đạt nghĩa là* **cưỡng chế bằng cấu trúc, không bằng danh sách cấm**:

- `tools/dict-build/Cargo.toml` khai `[workspace]` rỗng ⇒ nó là **workspace root của chính nó**, có `Cargo.lock` **riêng**. Cây phụ thuộc của `src-tauri` và cây phụ thuộc của build tool **không giao nhau về mặt vật lý**, nên AC này đúng theo cách không ai phá được bằng một dòng `use`.
- `npm run check:deps` chạy trên `src-tauri/Cargo.toml` **giữ nguyên xanh** và **giữ nguyên sàn 200** (`check-deps.mjs:52`) — nếu sàn nhảy vọt sau story này thì hai cây đã dính vào nhau, và đó là dấu hiệu AC4 vừa thủng.
- `scripts/check-dict-build.mjs` khẳng định `tools/dict-build/Cargo.toml` **có** dòng `[workspace]` và **không** có `workspace = true` / `path = "../../src-tauri"` ở bất kỳ phụ thuộc nào.

⚠️ Danh sách cấm theo tên crate (kiểu `BANNED_CRATES`) **không** đủ ở đây: parser dịch từ điển không có một tập tên hữu hạn biết trước, và `config_invariants.rs:92-94` đã ghi thành luật *"Danh sách CHO PHÉP, không phải danh sách CẤM"*.

### AC5 — Chỉ mục FTS trên nghĩa: chính phân biệt dấu, phụ xoá dấu

**Given** chỉ mục FTS trên nghĩa
**When** tạo
**Then** chỉ mục **chính** dùng `remove_diacritics 0`
**And** chỉ mục xoá dấu tồn tại như chỉ mục **phụ**, không bao giờ là mặc định

*Đạt nghĩa là* **bốn** mệnh đề:

1. Tồn tại **đúng hai** bảng FTS5 trên `dict_sense.gloss`: `sense_fts` (`unicode61 remove_diacritics 0`) và `sense_fts_nd` (`unicode61 remove_diacritics 2`).
2. **Tên nói rõ cái nào là chính** — bảng chính mang tên trần, bảng phụ mang hậu tố. Một giai đoạn sau đọc tên bảng chứ không đọc AD-27.
3. **Nghiệm thu bằng chính phép đo của Giai đoạn 0:** nạp `má ma mà mả mã mạ núi nui`, truy vấn `má` trên `sense_fts` trả về **đúng `má`**; cùng truy vấn trên `sense_fts_nd` trả về **cả sáu**. Hai chiều, ⛔ không chỉ một.
4. **Chỉ mục trigram nằm trên ĐẦU MỤC, không nằm trên nghĩa** — `entry_fts` dùng `tokenize="trigram"` trên `dict_entry.headword`. AD-26 nhánh 3 là *"chuỗi con 3+ ký tự"* của **đầu mục**; Giai đoạn 0 cũng đo trigram trên đầu mục (`phase-0-spike-results-2026-08-02.md:64`). Đặt trigram lên `gloss` là dựng nhầm chỉ mục cho nhầm nhánh, và Story 1.11 sẽ không có gì để đi.

### AC6 — Đối chiếu NFR6 trên payload sản phẩm thật, hai dòng, quy về byte

**Given** **mọi** artifact dữ liệu sẽ đóng gói — `dict-core.db` **và** bốn lớp gỡ rời của Story 1.10, không chỉ `dict-core.db`
**When** đo tổng
**Then** cộng thêm **21,29 MB bộ font** (đo thật ở Story 1.1) và **baseline ứng dụng thật** rồi đối chiếu lại với trần **NFR6**
**And** phép đối chiếu tính trên **payload sản phẩm**; **bản WebView2 Runtime nhúng của `.msi` KHÔNG được cộng vào** — ghi thành dòng riêng *(NFR6 sửa 2026-08-03)*
**And** phép đối chiếu quy về **byte** trước rồi mới đổi sang MB thập phân — 200 MB là **trần**, 150 MB là mốc kỳ vọng chứ không phải điều kiện đạt
**And** nếu vượt trần, đó là quyết định **tầng PRD** — **không** tự subset font, **không** tự bỏ một nguồn từ điển, **không** tự đổi sang font hệ điều hành

*Đạt nghĩa là* một **bảng kế toán** trong §Debug Log References với **mỗi dòng là một số byte đo được**, ⛔ không một dòng nào là ước lượng không ghi nguồn:

| Dòng | Nguồn số |
|---|---|
| `dict-core.db` sau `VACUUM` | đo thật ở story này |
| Bốn lớp gỡ rời | 🔴 **CHƯA TỒN TẠI — Story 1.10.** Ghi `[----] chưa đo` kèm lý do, ⛔ không ước, ⛔ không bỏ dòng |
| Bộ font | 21.285.713 byte — `font-spike-results-2026-08-03.md:82` |
| Baseline ứng dụng **thật** | dựng `.dmg` của cây nguồn hôm nay, trừ đi phần font. ⛔ Không dùng lại 1,40 MB của app thăm dò rỗng |
| WebView2 Runtime nhúng | **dòng riêng**, ⛔ không cộng vào tổng |
| **Tổng payload sản phẩm** | cộng bằng byte |
| Đối chiếu trần 200.000.000 byte | ĐẠT / VƯỢT / **CHƯA KẾT LUẬN ĐƯỢC** |

🔴 **AC này KHÔNG đóng trọn được ở story này, và điều đó phải ghi thẳng thay vì đánh dấu đạt.** AC đòi cộng bốn lớp gỡ rời của Story 1.10 — mà Story 1.10 dùng chính build tool của story này nên **không thể chạy trước**. Phán quyết đúng hôm nay là **CHƯA KẾT LUẬN ĐƯỢC**, kèm dư địa còn lại tính bằng byte để Story 1.10 chỉ việc trừ tiếp. Xem §Quyết định của Ice #1.

---

## Tasks / Subtasks

- [ ] **Task 1 — Đường cơ sở: chạy bảy lệnh trên cây sạch, ghi số vào §Debug Log References** (không AC)
  - [ ] `npm run build` *(bắt buộc trước `cargo test` — `generate_context!` nhúng `dist/` lúc biên dịch)*
  - [ ] `cargo test --manifest-path src-tauri/Cargo.toml` · `check:deps` · `check:tokens` · `check:i18n` · `check:commands` · `check:scope`
  - [ ] Ghi lại: số tệp `.rs` dưới `src-tauri/src/**` · quần thể `check-i18n.mjs` · tổng số test Rust · số khoá `vi.json` · **số phụ thuộc trong cây Rust mà `check-deps.mjs` đếm được** *(số này là chứng cứ AC4 sau story — ghi cả trước lẫn sau)*
  - [ ] ⛔ Không sửa gì ở task này. Một lệnh đỏ sẵn thì **dừng và báo**, không sửa lấn sang.

- [ ] **Task 2 — Khung crate `tools/dict-build`, tách khỏi cây phụ thuộc sản phẩm** (AC4)
  - [ ] `tools/dict-build/Cargo.toml` — 🔴 **có khối `[workspace]` rỗng** ở đầu tệp. Đây là toàn bộ AC4 ở tầng cấu trúc; thiếu nó thì một `Cargo.toml` workspace ở gốc repo về sau sẽ hút build tool vào cây sản phẩm mà không lỗi nào được ném.
  - [ ] `edition = "2024"`, `rust-version` khớp toolchain CI (`1.97.1` — ⛔ không chép `1.85` của `src-tauri/Cargo.toml`, xem `deferred-work.md:83`).
  - [ ] Phụ thuộc: `rusqlite` (feature `bundled`) · `serde` + `serde_json` (đọc JSONL của kaikki) · `sha2` (băm artifact) · `zip` **hoặc** giải nén tay ngoài tool (§Quyết định #6). ⛔ **Không** ghim `=` như `src-tauri` — đây không phải bảng Stack, và AD-25 nói giấy phép parser không ràng buộc sản phẩm. Vẫn **ghi giấy phép từng crate** vào `tools/dict-build/README.md` để lượt rà NFR15 sau không phải đoán.
  - [ ] `tools/dict-build/Cargo.lock` **được commit** (đây là một binary, không phải library).
  - [ ] `.gitignore`: thêm `tools/dict-build/raw/`, `tools/dict-build/out/`, `tools/dict-build/work/`. ⚠️ `*.db` đã có sẵn và ⛔ **đừng gỡ** (`.gitignore:52-54`).
  - [ ] Cập nhật `tools/dict-build/README.md` — gỡ dòng *"Chưa là crate Rust… Hình dạng của nó là quyết định của Story 1.9"* và ghi hình dạng đã chốt.

- [ ] **Task 3 — Lược đồ `dict-core.db`** (AC1, AC2, AC5)
  - [ ] `tools/dict-build/src/schema.rs` — DDL là **hằng `&'static str`**, một hằng cho một khối logic, đọc được cạnh mã.
  - [ ] Chín bảng theo §Quyết định #2: `dict_meta` · `dict_source` · `dict_entry` · `dict_sense` · `dict_example` · `dict_citation` · `char_idx` + ba bảng ảo FTS5.
  - [ ] 🔴 `dict_sense.source_id INTEGER NOT NULL REFERENCES dict_source(id)` và `PRAGMA foreign_keys = ON` **trước** mọi lệnh chèn. Không có ràng buộc này thì AC2 chỉ là một lời hứa.
  - [ ] `PRAGMA user_version = 1` trên tệp sinh ra, và một hàng `dict_meta('schema_version','1')` — hai chỗ vì `user_version` là thứ đường đọc kiểm rẻ nhất, còn `dict_meta` là thứ người đọc tệp bằng tay thấy được.
  - [ ] Ba chỉ mục FTS5 dựng theo AC5 và §Quyết định #3, **dùng external-content** (`content='dict_sense'` / `content='dict_entry'`) rồi `rebuild` một lượt ở cuối — ⚠️ **quên `rebuild` là chỉ mục rỗng, truy vấn trả 0 kết quả, không lỗi nào được ném** (§Bẫy 3).

- [ ] **Task 4 — Năm parser, mỗi nguồn một module, chạy trên fixture trước** (AC1, AC2)
  - [ ] `tools/dict-build/src/sources/{cvdict,unihan,cc_cedict,viwiktionary,en_wiktionary}.rs`
  - [ ] Mỗi module phơi **một** hàm cùng chữ ký: `fn parse(reader) -> impl Iterator<Item = Result<RawEntry>>`. Cùng một hình dạng cho năm nguồn là điều kiện để AC2 vế *"không hợp nhất"* kiểm được — mã hợp nhất bao giờ cũng xuất hiện ở chỗ năm hình dạng khác nhau phải quy về một.
  - [ ] 🔴 Mỗi module ghi **`source_version`** của chính nó vào `dict_source` — đọc từ nội dung tệp nguồn khi nguồn có khai (CC-CEDICT có dòng `#! date=`), ⛔ không viết cứng.
  - [ ] **Fixture 20–50 dòng cho mỗi nguồn** dưới `tools/dict-build/tests/fixtures/` — commit được vì nhỏ, và là thứ duy nhất chạy trong `cargo test` của build tool. ⚠️ Fixture phải chứa **ít nhất một đầu mục có mặt ở hai nguồn** (ứng viên: `山`, `中國`) để ca nghiệm thu AC2 có gì để bắt.
  - [ ] Ca lỗi bắt buộc mỗi parser: dòng hỏng ⇒ **đếm và báo cáo**, ⛔ không `panic!`, ⛔ không nuốt im lặng. Cuối lượt in bảng `nguồn → dòng đọc / dòng bỏ / lý do`.

- [ ] **Task 5 — `char_idx` phủ cả phồn thể lẫn giản thể** (AC1)
  - [ ] Sinh cặp `(ký tự, entry_id)` cho **mọi** ký tự Hán trong `headword` **và** trong `headword_simp`. §Bẫy 8: phủ mỗi phồn thể thì `国` trả rỗng trong 0,01 ms mà không lỗi nào được ném — đúng lớp lỗi mà FR39 tồn tại để chặn.
  - [ ] `WITHOUT ROWID` + khoá chính `(ch, entry_id)`.
  - [ ] Ghi lại **số cặp** sinh ra; Giai đoạn 0 đo được **1.297.115** cặp trên ba nguồn (`phase-0-spike-results-2026-08-02.md:93`) — lệch một bậc độ lớn so với số đó là dấu hiệu đọc sót nguồn.

- [ ] **Task 6 — Hoàn tất tệp: `ANALYZE`, `VACUUM`, và 🔴 `journal_mode = DELETE`** (AC1, AC6)
  - [ ] `INSERT INTO <fts>(<fts>) VALUES('rebuild')` cho cả ba bảng FTS **trước** `VACUUM`.
  - [ ] `ANALYZE` rồi `VACUUM` — `VACUUM` là điều kiện để số đo AC6 là số thật chứ không phải số có lỗ.
  - [ ] 🔴 `PRAGMA journal_mode = DELETE` trên tệp sinh ra. **Đây là bẫy đắt nhất của story** — xem §Bẫy 1. Một tệp còn ở chế độ WAL cần quyền ghi vào thư mục chứa nó để dựng `-shm`, mà `$RESOURCE/dict/` trên máy người dùng là **chỉ đọc** (AD-7, AD-23). Lỗi này chạy hoàn hảo suốt lúc phát triển.
  - [ ] Kiểm ngay sau khi đóng: `PRAGMA journal_mode` trả `delete`, và ⛔ **không tệp `-wal`/`-shm` nào còn sót cạnh `.db`**.
  - [ ] In SHA-256 của tệp cuối + kích thước **bằng byte**.

- [ ] **Task 7 — Cổng `scripts/check-dict-build.mjs`** (AC2, AC4)
  - [ ] **Kiểm A — từ vựng hợp nhất.** Quét `tools/dict-build/src/**/*.rs` tìm danh sách token (§Quyết định #4). Miễn trừ khai bằng comment `// dict-build:allow <token> — <lý do>` **ngay dòng trên**, và cổng **in ra số miễn trừ mỗi lượt chạy**.
  - [ ] **Kiểm B — cách ly workspace.** `tools/dict-build/Cargo.toml` có `[workspace]`; ⛔ không phụ thuộc nào trỏ `path` sang `src-tauri`.
  - [ ] **Kiểm C — sàn số tệp.** Cây rỗng ⛔ không được đọc thành sạch — bài học `check-deps.mjs:15-17`, `store_boundary.rs:44`.
  - [ ] Nghiệm thu **đỏ-rồi-xanh**: tối thiểu 8 đối chứng âm (mỗi kiểm ≥ 2), ghi vào §Debug Log References.

- [ ] **Task 8 — Cổng `scripts/check-dict-manifest.mjs`** (AC3)
  - [ ] Parser TOML **tập con nghiêm ngặt, tự viết** — ⛔ không thêm phụ thuộc npm (NFR15 đòi rà GPLv3 + vào bảng Stack trước; một tệp 40 dòng không đáng một lượt rà). Cú pháp ngoài tập con ⇒ **FAIL**, ⛔ không bỏ qua. Tiền lệ: `parseCssBlocks` của `check-tokens.mjs`.
  - [ ] Luật: `[base]` **phải có mặt**; mỗi mục đủ `url` · `sha256` · `source_version`; `sha256` khớp `/^[0-9a-f]{64}$/`; `url` bắt đầu `https://` và chứa `/releases/download/dict-v`; `source_version` không rỗng.
  - [ ] ⛔ Cổng **không** đọc tệp `.db` và **không** tải gì — nó phải xanh trên một runner CI không có byte dữ liệu nào.
  - [ ] Nghiệm thu đỏ-rồi-xanh ≥ 10 ca (thiếu trường · sha256 41 ký tự · sha256 hoa · `http://` · thiếu `[base]` · TOML ngoài tập con · mục `[[detachable]]` thiếu `name` …).

- [ ] **Task 9 — Gắn hai cổng vào CI, và xử món nợ gốc quét của `check-i18n.mjs`** (AC2, AC3)
  - [ ] `package.json`: `check:dict` và `check:dict-manifest`.
  - [ ] `ci.yml` job `check`: hai bước mới, đặt **kề `check:deps`** và **trước `npm run build`** — ⛔ không dựng pipeline thứ hai (AC4 của Story 1.3).
  - [ ] 🔴 **Món nợ `deferred-work.md:44` kích hoạt ở chính story này.** Nó ghi: *"Gốc quét cứng ở `src/` và `src-tauri/` … Mở lại khi cây mọc nhánh thứ ba."* `tools/dict-build/` **là nhánh thứ ba**. Xử theo đúng doctrine của chính cổng đó — **thêm `tools/` vào gốc quét, rồi miễn trừ nó trong `EXEMPT` kèm tên và lý do** (*build tool không vào bản phát hành, không có bề mặt giao diện, chuỗi của nó là chẩn đoán cho người dựng*). ⛔ Bỏ qua im lặng là để một lần thu hẹp phạm vi trốn khỏi sổ.
  - [ ] Cập nhật `deferred-work.md:44` thành đã xử, ghi cách xử.
  - [ ] ⚠️ **Thêm gốc quét làm ĐỔI quần thể mà `check-i18n.mjs` in ra.** Sàn `RS_FLOOR`/`VUE_FLOOR` đo trên quần thể **sau** miễn trừ, nên nếu `tools/` được miễn trừ trọn thì con số phải **không đổi**. Số nhảy lên nghĩa là miễn trừ chưa ăn — sửa miễn trừ, ⛔ đừng chỉnh sàn cho vừa.
  - [ ] ⚠️ **Hai cổng mới phải chạy được trên Windows.** `check-deps.mjs:38-46` đã ghi lại đúng lần bị cắn: `npm`/`npx` trên Windows cần `shell: true` vì `libuv` không dò `.cmd`. Hai cổng của story này **không spawn `npm`** nên không dính bẫy đó — nhưng chúng đọc tệp, nên ⛔ dùng `path.join`, ⛔ không nối chuỗi bằng `/`, và so sánh nội dung theo dòng phải chịu được `\r\n`.

- [ ] **Task 10 — Gỡ `$RESOURCE/dict/**` khỏi `assetProtocol.scope`** (không AC — **Ice phê chuẩn 2026-08-04**, đóng `deferred-work.md:21` + `:57`)
  - [ ] `src-tauri/tauri.conf.json:28` — `scope` còn **đúng một** mục: `["$RESOURCE/fonts/**"]`.
  - [ ] `src-tauri/tests/config_invariants.rs:300-318` — sửa assertion **và đổi tên hàm** thành `asset_protocol_scope_has_exactly_the_one_readonly_resource_area`. ⛔ Đổi mỗi giá trị mà giữ tên `..._the_two_...` là để lại một cái tên nói dối, và tên test là thứ lượt rà soát sau đọc trước tiên.
  - [ ] Viết **lý do vào chính test**, không chỉ vào story: webview ⛔ **không bao giờ** đọc tệp từ điển — AD-1 và AD-11 đặt mọi truy cập dữ liệu ở Rust, và `rusqlite` mở tệp bằng đường dẫn hệ thống, **không** đi qua asset protocol. Mục scope đó là một **quyền thừa**; mâu thuẫn với `connect-src` chỉ là hệ quả của việc nó thừa.
  - [ ] Chạy lại `npm run check:scope` và `npm run check:scope:bundled` — cả hai ⛔ **không** tham chiếu `dict` *(đã kiểm 2026-08-04)*, nên chúng phải xanh **không đổi một dòng**. Một cái đỏ nghĩa là có đường phụ thuộc chưa ai biết → dừng và báo.
  - [ ] 🔴 **Dựng lưới thay thế trước khi gỡ.** Sau lượt này, ⛔ **không còn dòng nào** trong `tauri.conf.json` nhắc tới `dict` cho tới Story 10.1 — tức lưới bắt *"ship một bản không có byte từ điển nào"* (`deferred-work.md:21`) mất luôn chỗ bấu. Ghi một mục `deferred-work.md` mới, đích danh **Story 10.1**: *thêm `dict/*.db` vào `bundle.resources` **và** một test khẳng định nó có mặt*.
  - [ ] ⛔ **Không sửa `ARCHITECTURE-SPINE.md`.** AD-23 (`:316`) còn liệt kê `$RESOURCE/dict/**` bằng chữ; sửa nó là lượt riêng của Ice — tiền lệ quyết định #3 ở Story 1.3. Ghi vào §Completion Notes là AD-23 **đang lệch khỏi cấu hình** và ai đọc AD-23 trước cấu hình sẽ hiểu sai.

- [ ] **Task 11 — Chạy thật trên năm nguồn, ghi bảng số** (AC1, AC2, AC5)
  - [ ] Tải năm nguồn theo §Thông tin kỹ thuật. ⚠️ Đây là bước **nặng nhất và duy nhất không tự động hoá được** — vài GB. Tải vào `tools/dict-build/raw/` (đã ignore).
  - [ ] Chạy build tool. Ghi bảng: nguồn → dòng đọc · dòng bỏ · `dict_entry` · `dict_sense` · `dict_example` · `dict_citation`.
  - [ ] Đối chiếu với Giai đoạn 0 (**604.357** nghĩa, **27.956** ví dụ trên **ba** nguồn) — số hôm nay phải **lớn hơn**; nhỏ hơn nghĩa là có nguồn đọc hỏng im lặng.
  - [ ] Ba phép nghiệm thu chạy trên tệp thật:
    - **AC2** — `山` (hoặc đầu mục chồng lấn khác) cho ra ≥ 2 hàng nghĩa, `source_id` khác nhau.
    - **AC5** — hai chiều của phép thử `má/ma/mà/mả/mã/mạ`.
    - **AD-26** — `SELECT` qua `entry_fts` với `中國人` trả khác rỗng; qua `char_idx` với `山` và `中國` trả khác rỗng. *(Đây là **đối chứng dữ liệu**, ⛔ không phải cài đặt đường tra cứu — đường đó là Story 1.11.)*

- [ ] **Task 12 — Kế toán NFR6** (AC6)
  - [ ] Dựng `.dmg` của cây nguồn **hôm nay** để lấy **baseline ứng dụng thật**; trừ phần font ra bằng lớp phủ `bundle.resources: null` đã có sẵn (`config_invariants.rs:470` mô tả chính lớp phủ đó, và ⚠️ chiều của phép trừ đã **đảo** so với mũi thăm dò — `font-spike-results-2026-08-03.md:437`).
  - [ ] Dựng bảng kế toán của AC6, **mọi dòng bằng byte**, rồi mới đổi sang MB thập phân.
  - [ ] Dòng *"bốn lớp gỡ rời"* ghi `[----] chưa đo — Story 1.10`. Phán quyết cuối là **CHƯA KẾT LUẬN ĐƯỢC**, kèm **dư địa còn lại tính bằng byte**.
  - [ ] Nếu ngay `dict-core.db` một mình đã làm tổng vượt trần: ⛔ **dừng, ghi số, báo Ice.** Không subset font, không bỏ nguồn, không đổi font hệ điều hành, không bỏ chỉ mục phụ của AC5.

- [ ] **Task 13 — Điền `dict-manifest.toml` tới ranh giới dev làm được** (AC3)
  - [ ] Bỏ comment khối `[base]`, điền `sha256` và `source_version` **thật**.
  - [ ] `url`: dạng `https://github.com/vannamhh/AuraTranslate/releases/download/dict-v1/dict-core.db`. ⚠️ Nếu Ice chưa tạo release, URL vẫn phải **đúng dạng và đúng tag dự kiến** — cổng của Task 8 kiểm hình dạng, và §Completion Notes phải ghi rõ *"release chưa tồn tại, URL sẽ 404 tới khi Ice tải lên"*. ⛔ Không viết một URL của một tag khác chỉ để nó phản hồi 200.
  - [ ] Ghi vào §Completion Notes **lệnh chép-dán** để Ice tạo release (`gh release create dict-v1 …`).

- [ ] **Task 14 — Chốt sổ** (không AC)
  - [ ] Chạy lại trọn bộ lệnh của Task 1 cộng hai cổng mới; ghi số sau story.
  - [ ] Cập nhật `src-tauri/resources/dict/README.md`: tệp nào tồn tại, tải từ đâu, ai sở hữu bước tiếp, **và** rằng vùng này ⛔ không còn nằm trong `assetProtocol.scope` (Task 10) — README hiện đang khẳng định ngược lại.
  - [ ] Cập nhật `deferred-work.md`: đóng `:44` · `:79` · `:21` · `:57`; ghi trạng thái `:31` (§Quyết định của Ice #3); thêm mục mới cho **Story 10.1** (lưới `bundle.resources`, Task 10).
  - [ ] §Quyết định của Ice: điền phán quyết thật của AC6.

---

## Dev Notes

### Ranh giới phạm vi — đọc trước khi gõ dòng đầu tiên

| Thứ | Trong story này? |
|---|---|
| Crate `tools/dict-build`, lược đồ, năm parser | ✅ **Có** — hạt nhân |
| `char_idx` + ba chỉ mục FTS5 | ✅ **Có** |
| Hai cổng mới + gắn vào CI | ✅ **Có** |
| Điền `dict-manifest.toml` phần dev làm được | ✅ **Có** |
| Bảng kế toán NFR6 | ✅ **Có** — kết luận là *chưa kết luận được*, và đó là kết quả hợp lệ |
| Đóng `deferred-work.md:44` và `:79` | ✅ **Có** |
| **Đường tra cứu ba nhánh** (`core/dict/`) | ❌ **Không** — **Story 1.11** |
| **Cổng `DictionarySource`**, adapter, giữ nguyên bất đồng lúc chạy | ❌ **Không** — **Story 1.13**. ⛔ `ports/mod.rs` giữ nguyên 5 dòng |
| **Bốn lớp gỡ rời** + metadata giấy phép của chúng | ❌ **Không** — **Story 1.10** |
| **Matcher dùng chung** (`jieba-rs`, stemming) | ❌ **Không** — **Story 1.12** |
| Bật/tắt nguồn, màn hình Attribution | ❌ **Không** — Story 1.19 / 10.4 |
| Đóng gói `.db` vào bản phát hành (`bundle.resources`) | ❌ **Không** — **Story 10.1** |
| **Gỡ `$RESOURCE/dict/**` khỏi `assetProtocol.scope`** | ✅ **Có** — Ice phê chuẩn 2026-08-04, xem **Task 10** |
| Nới CSP (`connect-src`, thêm `asset:`) | ❌ **Không** — ⛔ và không bao giờ. Gỡ scope làm mâu thuẫn biến mất **theo chiều siết**, đó là toàn bộ lý do chọn đường này |
| Sửa `[profile.release]` của `src-tauri/Cargo.toml` | ❌ **Không** — §Quyết định của Ice #3 |
| Tạo GitHub Release, tải tệp `.db` lên | ❌ **Không** — Ice làm tay (AC3 vế 2) |
| Sửa `prd.md` / `epics.md` / `ARCHITECTURE-SPINE.md` | ❌ **Không** — tiền lệ quyết định #3 của Ice ở Story 1.3: dev không sửa tài liệu quy hoạch |
| Phụ thuộc mới cho `src-tauri` | ❌ **Không** — 0 dòng đổi trong `src-tauri/Cargo.toml` |

### Trạng thái repo hiện tại — số, không phải mô tả

> ⚠️ **Baseline là CÂY LÀM VIỆC, không phải `0ff36a0`.** Toàn bộ thay đổi của Story 1.8 *(status `done`)* **chưa được commit**: `commands/config.rs`, `core/scope/{kinds,resolve,store}.rs`, `tests/scope_{boundary,contract}.rs`, `src/config/`, cộng sửa ở `lib.rs`, `commands/mod.rs`, `core/store/*`, `App.vue`, `main.ts`, `check-i18n.mjs`. Chạy `git status` trước khi tin bất kỳ con số nào dưới đây.

| Thứ | Số / trạng thái |
|---|---|
| `.rs` dưới `src-tauri/src/**` | **26** |
| Test Rust | **62** — `config_invariants` 15 · `ipc_contract` 5 · `scope_boundary` 5 · `scope_contract` 17 · `store_boundary` 4 · `store_contract` 16 |
| Khoá `vi.json` | **16** |
| `tools/dict-build/` | **một tệp README**, chưa là crate, chưa có `Cargo.toml` |
| `dict-manifest.toml` | tồn tại, **comment toàn bộ**, không cổng nào đọc |
| `src-tauri/resources/dict/` | `.gitkeep` + `README.md` — **không byte dữ liệu nào** |
| `bundle.resources` (`tauri.conf.json:36`) | chỉ `fonts/*` và `license/*.txt` — ⛔ **không** có `dict/` |
| `assetProtocol.scope` | `["$RESOURCE/dict/**", "$RESOURCE/fonts/**"]` — dict **có** trong scope nhưng **không** được đóng gói |
| Cổng npm hiện có | `check:deps` · `check:tokens` · `check:i18n` · `check:commands` · `check:scope` · `check:scope:bundled` |
| `git remote origin` | `https://github.com/vannamhh/AuraTranslate.git`, nhánh `master`, có `origin/master` |
| Release `dict-v*` | **chưa có** |
| Phụ thuộc mới cần thêm cho `src-tauri` | **0** |

Doc-comment sẵn có ở `src-tauri/src/core/dict/mod.rs` — **giữ nguyên, ⛔ không viết mã vào đó ở story này**:
```rust
//! Tra cứu từ điển — ba nhánh truy vấn tiếng Trung (AD-26).
//!
//! KHÔNG tồn tại bước hợp nhất nguồn (AD-19): mỗi kết quả luôn mang `source` của nó.
//! Mỗi lớp gỡ rời là một file `.db` độc lập, chỉ đọc (AD-10, AD-25).
```

### 🔴 "Nguồn" mang bốn nghĩa khác nhau trong story này — đừng trộn

| Cách dùng | Nghĩa |
|---|---|
| `dict_sense.source_id` · AD-19 · FR31 | **Nguồn từ điển** của một nghĩa — cái AC2 nói tới |
| *"nguồn thô"* · `source_version` | **Tệp dữ liệu gốc** tải về (`CVDICT.u8`, `Unihan.zip`…) |
| *"cây nguồn"* · `src/` | **Mã nguồn** |
| `SOURCE_ORIGIN` trong ER diagram | **Xuất xứ tài liệu** của một Chương (FR128) — ❌ Epic 6, không liên quan |

### Mười một cái bẫy — bảy trong mười một cho ra một lượt CI XANH với hành vi sai

#### Bẫy 1 — Tệp `.db` giao ở chế độ WAL 🔴 đắt nhất trong story

Build tool mở `rusqlite`, và cám dỗ tự nhiên là bật `journal_mode = WAL` cho nhanh — đúng thứ `core::store` làm. Tệp sinh ra chạy **hoàn hảo** trên máy dev, chạy hoàn hảo trong mọi test, và **hỏng trên máy người dùng**: `$RESOURCE/dict/` là chỉ đọc (AD-7 *"chỉ đọc, luôn luôn"*; AD-23), mà SQLite ở chế độ WAL cần **quyền ghi vào thư mục chứa tệp** để dựng `-shm`. Lỗi lộ ra ở **lần tra cứu đầu tiên của người dùng thật**, tức Epic 1 giao xong rồi mới biết.

Đúng hình dạng lỗi mà `deferred-work.md:57` đã mô tả cho một đường khác: *"chạy tốt suốt lúc phát triển rồi hỏng ở bản người dùng cài — đúng lớp lỗi mà dự án này liên tục đi săn."*

**Luật:** tệp giao ra ở `journal_mode = DELETE`, kiểm ngay sau khi đóng, và ⛔ không tệp `-wal`/`-shm` nào cạnh nó.

#### Bẫy 2 — Đưa `tools/dict-build` vào cùng workspace với `src-tauri` 🔴

Cargo hợp nhất phiên bản và **feature** trong một workspace. `src-tauri` ghim `rusqlite = "=0.40.1"` với đúng feature `bundled`; build tool cũng dùng `rusqlite`. Chung workspace ⇒ một `Cargo.lock`, và **feature unification** có thể bật thêm feature của `rusqlite` cho **cả bản phát hành** — trong khi `schema.rs:29-32` ghi rõ feature `backup` đang **TẮT có chủ ý**. Kết quả: bảng Stack nói một đằng, nhị phân giao đi một nẻo, và `check:deps` vẫn xanh vì nó đếm **tên crate**, không đếm feature.

**Luật:** `[workspace]` rỗng trong `tools/dict-build/Cargo.toml`. Hai `Cargo.lock`, hai cây, ⛔ không giao nhau.

#### Bẫy 3 — Bảng FTS5 external-content không `rebuild` ⇒ chỉ mục RỖNG, không lỗi 🔴

`CREATE VIRTUAL TABLE … USING fts5(gloss, content='dict_sense', content_rowid='id', …)` **không** tự nạp gì. Không có `INSERT INTO sense_fts(sense_fts) VALUES('rebuild')` thì mọi `MATCH` trả **0 hàng trong 0,01 ms, không lỗi**. Đây là **cùng một hình dạng** với phát hiện nghiêm trọng nhất của Giai đoạn 0 (`phase-0-spike-results-2026-08-02.md:89`): *"Nguy hiểm ở chỗ nó không báo lỗi."*

**Luật:** `rebuild` cả ba bảng, và nghiệm thu bằng một truy vấn `MATCH` trả **khác rỗng** trên tệp thật — ⛔ một test chỉ khẳng định bảng *tồn tại* không bắt được ca này.

#### Bẫy 4 — Quên mệnh đề `tokenize` ⇒ rơi về `remove_diacritics 1` im lặng 🔴

`fts5(gloss)` không có `tokenize` mặc định về `unicode61` với `remove_diacritics 1` — chính xác cái làm `má/ma/mà/mả/mã/mạ` gộp thành một. Không lỗi, không cảnh báo. Kiểu hỏng này chỉ lộ ra bằng **phép thử hai chiều** của AC5.

⚠️ Kèm theo: `remove_diacritics=0` (có dấu `=`) là **lỗi cú pháp**; cú pháp đúng là dấu cách. Lỗi cú pháp thì đỏ ngay — đó là ca dễ. Ca khó là **quên hẳn**.

#### Bẫy 5 — Đặt trigram lên `gloss` thay vì lên `headword`

AD-26 nhánh 3 là **chuỗi con của đầu mục**. Giai đoạn 0 đo trigram **trên đầu mục** (+13,9 MB). Đặt nó lên nghĩa cho ra một chỉ mục lớn hơn nhiều, cho một nhánh truy vấn **không tồn tại trong kiến trúc**, và Story 1.11 sẽ không tìm thấy thứ nó cần. Mọi test của story này vẫn xanh.

#### Bẫy 6 — Hợp nhất nghĩa "cho gọn" 🔴

Khi năm nguồn cùng có `山`, cám dỗ là gộp thành một hàng có `sources = 'cvdict,cc-cedict'`. AD-19 gọi đích danh: *"ai đó thêm một bước khử trùng lặp cho gọn UI và làm mất FR31/FR32."* PRD nói mạnh hơn (`prd.md:474`): *"Một công cụ hợp nhất mọi từ điển thành một câu trả lời duy nhất là một công cụ giấu đi sai sót."*

Hình dạng sai điển hình — trông rất hợp lý:
```rust
let mut by_headword: HashMap<String, Sense> = HashMap::new();
by_headword.entry(hw).or_insert(sense);   // ⛔ nguồn thứ hai bị NUỐT, không lỗi
```

#### Bẫy 7 — `source` có cột nhưng không có ràng buộc

Khai `source_id INTEGER` rồi chèn `NULL` khi parser không xác định được nguồn. AC2 nói *"cột `source` bắt buộc có giá trị"* — cưỡng chế là `NOT NULL` **cộng** `PRAGMA foreign_keys = ON`. ⚠️ `foreign_keys` **mặc định TẮT** trong SQLite và phải bật **mỗi kết nối**; quên bật thì `REFERENCES` chỉ là chú thích.

#### Bẫy 8 — `char_idx` chỉ phủ phồn thể

CVDICT và CC-CEDICT đều mang **cả hai** dạng. Phủ mỗi `headword` ⇒ tra `国` trả rỗng còn `國` trả kết quả, và người dùng đọc thành *"tra từ không ra kết quả"*. Cùng lớp lỗi với FR39.

#### Bẫy 9 — Đo NFR6 bằng MiB, hoặc cộng WebView2 vào tổng

`font-spike-results-2026-08-03.md:78` đã khai một lần cho tất cả: **quy về byte trước**, MB **thập phân**, 200 MB là **trần** chứ không phải dải. Và `prd.md:832`: WebView2 Runtime nhúng **nằm ngoài** ngân sách, ghi **dòng riêng**. Ở mốc 200, MB và MiB lệch **7 %** — đủ lật một phán quyết.

#### Bẫy 10 — Lấy nhầm danh sách nguồn 🔴

Ba tài liệu đang nói ba danh sách khác nhau (`font-spike-results-2026-08-03.md:377-388`):

| Nơi | Danh sách |
|---|---|
| `prd.md:1022` — giả định `[A2]` | Unihan · Thiều Chửu · Cổ hán văn · VietPhrase |
| `epics.md` Story 1.9 (**AC của story này**) | CVDICT · Unihan · CC-CEDICT · viwiktionary · en.wiktionary |
| `ARCHITECTURE-SPINE.md:651` | thêm Thiều Chửu · Cổ hán văn · VietPhrase vào cùng một ô sơ đồ |

**Phân xử — đã làm sẵn, dev ⛔ không phải quyết lại:** danh sách của `epics.md` **thắng**, vì (a) nó là AC nghiệm thu của chính story này; (b) nó khớp **đúng** năm hàng `Nền` của `prd.md:888-892`; (c) AD-10 nói năm nguồn nền gộp trong `dict-core.db` còn bốn nguồn kia mỗi cái một tệp riêng, tức **Story 1.10**. Danh sách của `[A2]` là **phần chưa đo**, không phải phạm vi story này. Sơ đồ ở `ARCHITECTURE-SPINE.md:651` gộp cả chín vào một ô vì nó vẽ **đường dữ liệu**, không vẽ phạm vi story.

#### Bẫy 11 — Tải nhầm tệp kaikki, hoặc tải bản 23 GB

Hai nhầm lẫn khác nhau, cùng đắt:

1. **`zh.wiktionary` ≠ *"tiếng Trung bóc từ en.wiktionary"*.** Giai đoạn 0 đã **bác** zh.wiktionary (1,13 GB): *"định nghĩa và nhãn từ loại đều bằng tiếng Trung"* (`phase-0-spike-results-2026-08-02.md:169`). Thứ PRD chọn là **`en.wiktionary` qua Wiktextract** — tức trang *Chinese* trên kaikki.org của **ấn bản tiếng Anh**. ⚠️ `ARCHITECTURE-SPINE.md:651` viết `en.wiktionary … (1,13 GB)` và con số 1,13 GB đó là của **zh.wiktionary**; đừng dùng nó để nhận dạng tệp.
2. **`raw-wiktextract-data.jsonl` là 23,1 GB.** ⛔ Không tải bản đó. Lấy **bản trích theo ngôn ngữ** trên kaikki.org — nhỏ hơn hàng bậc và đúng thứ cần.

### Quyết định thiết kế — đã chốt, không phải lựa chọn của dev

#### #1 — Build tool là một crate Rust ĐỘC LẬP, không phải script Python

Giai đoạn 0 dựng bằng Python, nhưng đó là mã scratchpad dùng một lần. Ba lý do chọn Rust và tách workspace:

- **AC4 thành hệ quả cấu trúc.** Hai cây phụ thuộc tách vật lý; ⛔ không cần tin ai.
- **Một toolchain.** Thêm Python là thêm một sàn cài đặt cho mọi máy dựng và mọi runner CI, trong khi NFR14 đòi hành vi tương đương hai nền tảng.
- **Dùng lại đúng SQLite của sản phẩm.** `rusqlite` feature `bundled` ghim chính bản SQLite mà ứng dụng đọc — nên *"trigram khả dụng"* và *"`remove_diacritics 0` khả dụng"* (`ARCHITECTURE-SPINE.md:638`) là cùng một câu trả lời ở cả hai đầu. Dựng bằng một SQLite khác là mở đúng khe *dựng được, đọc không được*.

#### #2 — Lược đồ `dict-core.db`, chốt cứng

```sql
-- Siêu dữ liệu của chính tệp
CREATE TABLE dict_meta (key TEXT PRIMARY KEY, value TEXT NOT NULL);
--   'schema_version' | 'built_at' (ISO-8601 UTC) | 'builder_version'

-- Một nguồn, tự mang giấy phép và ghi công của chính nó.
-- Cùng khuôn với lớp gỡ rời (AD-10) — Story 1.10 dùng LẠI bảng này, không dựng bảng khác.
CREATE TABLE dict_source (
  id             INTEGER PRIMARY KEY,
  code           TEXT NOT NULL UNIQUE,   -- 'cvdict' | 'unihan' | 'cc-cedict' | 'viwiktionary' | 'en-wiktionary'
  display_name   TEXT NOT NULL,
  license_kind   TEXT NOT NULL,          -- 'open' | 'public-domain' | 'author-grant'
  license_id     TEXT,                   -- 'CC-BY-SA-4.0' | 'Unicode-3.0' | NULL
  license_text   TEXT NOT NULL,
  attribution    TEXT NOT NULL,
  source_version TEXT NOT NULL,
  source_url     TEXT NOT NULL
);

CREATE TABLE dict_entry (
  id            INTEGER PRIMARY KEY,
  source_id     INTEGER NOT NULL REFERENCES dict_source(id),
  lang          TEXT NOT NULL,           -- 'zh' | 'en'
  headword      TEXT NOT NULL,
  headword_simp TEXT,                    -- giản thể, khi nguồn có
  reading       TEXT,                    -- pinyin
  han_viet      TEXT                     -- âm Hán Việt (nền tab Hán Việt, FR33)
);

CREATE TABLE dict_sense (
  id        INTEGER PRIMARY KEY,
  entry_id  INTEGER NOT NULL REFERENCES dict_entry(id),
  source_id INTEGER NOT NULL REFERENCES dict_source(id),  -- 🔴 AC2 — NOT NULL
  pos       TEXT,                        -- nhãn từ loại
  pos_lang  TEXT,                        -- 'vi' | 'en' — FR35 đòi ĐÁNH DẤU RÕ nhãn ngoại ngữ
  gloss     TEXT NOT NULL,
  note      TEXT,                        -- 'ghi chú' của FR28
  ord       INTEGER NOT NULL
);

CREATE TABLE dict_example (
  id               INTEGER PRIMARY KEY,
  sense_id         INTEGER NOT NULL REFERENCES dict_sense(id),   -- FR30: theo TỪ LOẠI
  text             TEXT NOT NULL,
  translation      TEXT,
  translation_lang TEXT,
  ord              INTEGER NOT NULL
);

CREATE TABLE dict_citation (
  id       INTEGER PRIMARY KEY,
  sense_id INTEGER NOT NULL REFERENCES dict_sense(id),
  text     TEXT NOT NULL,
  work     TEXT,                         -- xuất xứ văn bản — điểm phân biệt với ví dụ (FR30)
  author   TEXT,
  ord      INTEGER NOT NULL
);

CREATE TABLE char_idx (
  ch       TEXT    NOT NULL,
  entry_id INTEGER NOT NULL REFERENCES dict_entry(id),
  PRIMARY KEY (ch, entry_id)
) WITHOUT ROWID;

CREATE INDEX idx_entry_headword      ON dict_entry(headword);
CREATE INDEX idx_entry_headword_simp ON dict_entry(headword_simp);
CREATE INDEX idx_sense_entry         ON dict_sense(entry_id);
```

Ba điểm ⛔ không thương lượng:

- **`pos_lang` tồn tại vì FR35**: *"nhãn từ loại và bản dịch ví dụ bằng tiếng Anh được chấp nhận và phải được **đánh dấu rõ là nhãn ngoại ngữ**."* Không có cột này thì Story 1.17 phải đoán, hoặc viết mã riêng cho từng nguồn — đúng thứ AD-10 cấm.
- **`license_kind` là chuỗi mở, ⛔ KHÔNG phải enum các giấy phép mở.** AD-10 nói thẳng: *"mô hình hoá trường này thành enum các giấy phép mở sẽ khiến nó bị gán nhãn sai ngay trên màn hình Attribution."* Năm nguồn hôm nay đều `open`/`public-domain`, nhưng HVTĐTD ở Story 1.10 là `author-grant` — và bảng này phải nhận được nó **mà không đổi lược đồ**.
- **`dict_source` có mặt trong `dict-core.db` dù nó là "một tệp năm nguồn".** Nó là điều kiện của AC2 (`source_id` phải trỏ đi đâu đó) **và** là khuôn mà Story 1.10 chép sang từng tệp lớp gỡ rời — cùng một hình dạng ⇒ *"runtime không có mã riêng cho từng nguồn"* (AD-10) là hệ quả chứ không phải nỗ lực.

#### #3 — Ba bảng FTS5, external-content, dựng ở cuối

```sql
CREATE VIRTUAL TABLE entry_fts USING fts5(
  headword, content='dict_entry', content_rowid='id', tokenize="trigram");

CREATE VIRTUAL TABLE sense_fts USING fts5(          -- CHÍNH (AD-27)
  gloss, content='dict_sense', content_rowid='id',
  tokenize="unicode61 remove_diacritics 0");

CREATE VIRTUAL TABLE sense_fts_nd USING fts5(       -- PHỤ — ⛔ không bao giờ mặc định
  gloss, content='dict_sense', content_rowid='id',
  tokenize="unicode61 remove_diacritics 2");
```

- **External-content** vì tệp là **chỉ đọc trọn đời** — không có `UPDATE` nào để trigger phải theo, và nó tiết kiệm đúng phần lưu bản sao văn bản.
- **`rebuild` một lượt ở cuối**, sau khi mọi hàng đã chèn, trước `VACUUM` (§Bẫy 3).
- **`remove_diacritics 2`** cho bảng phụ (bản đầy đủ hơn `1`; cần SQLite ≥ 3.27, mà `bundled` vượt xa — `ARCHITECTURE-SPINE.md:638`).
- 🔴 **Chi phí phải ghi ra:** Giai đoạn 0 đo **~17 MB mỗi chỉ mục FTS**, và mức 130 MB của nó **chỉ gồm một** chỉ mục trên nghĩa. `sense_fts_nd` là **chỉ mục thứ hai** ⇒ +~17 MB **chưa từng nằm trong bất kỳ con số nào đã lưu hành**. Ghi thành **dòng riêng** trong bảng kế toán AC6.

#### #4 — Danh sách token của cổng hợp nhất, hẹp có chủ ý

Cấm dưới `tools/dict-build/src/**`: `merge` · `unify` · `dedup` · `dedupe` · `coalesce` · `consolidate` · `combine_senses` · `or_insert` · `or_insert_with` · `entry(`.

- Bốn tên cuối là **hình dạng thật** của lỗi ở §Bẫy 6, không phải từ khoá ngữ nghĩa — đó là lý do chúng có mặt.
- ⛔ **Không** cấm `distinct` / `DISTINCT`: `SELECT DISTINCT` khi sinh `char_idx` là hợp lệ và cần thiết.
- Miễn trừ khai **ngay dòng trên** bằng `// dict-build:allow <token> — <lý do>`, và cổng **in ra tổng số miễn trừ mỗi lượt**. Miễn trừ im lặng là cách một cổng chết mà vẫn xanh (`check-i18n.mjs` §EXEMPT là tiền lệ).

#### #5 — Build tool KHÔNG dùng `core::store`

Cám dỗ: *"đã có `store::Writer` nối tiếp rồi, dùng lại."* Sai ba đường:

1. AD-11 nói writer nối tiếp là cho **kho ghi được lúc chạy** (`project.db`, `global.db`, `library-index.db`). `dict-core.db` là **chỉ đọc, luôn luôn** (AD-7) — nó không phải một kho theo nghĩa đó.
2. `core::store` sống trong `src-tauri`; dùng nó buộc build tool vào cây phụ thuộc sản phẩm, phá AC4 và §Bẫy 2.
3. `store_boundary.rs` cấm token `rusqlite` ngoài `src/core/store/**`. ⚠️ Phép quét đó **chỉ chạy trên `src-tauri/src/**`** (`store_boundary.rs:44`), nên `tools/` nằm ngoài — điều này là **đúng và cố ý**, ⛔ nhưng đừng vì thế mà đặt build tool dưới `src-tauri/`.

#### #6 — Giải nén nằm ngoài build tool nếu nó tốn một phụ thuộc

`Unihan.zip` cần giải nén. Nếu crate `zip` làm build tool nặng thêm đáng kể, chấp nhận **giải nén tay** và cho build tool đọc thư mục đã giải. Đây **không** phải quyết định kiến trúc; ghi lựa chọn vào `tools/dict-build/README.md` kèm lệnh chép-dán. ⛔ Điều **không** được làm: để build tool tự tải nguồn từ mạng — AD-15 khoá số điểm ra mạng ở **ba**, và dù build tool không nằm trong bản phát hành, một tệp nguồn tải ngầm là một artifact không ai biết phiên bản.

#### #7 — `dict-core.db` mang `PRAGMA user_version`, dù nó không di trú

AD-30 phủ `meta.json` · `project.db` · `global.db`; tệp từ điển **không** di trú (nó được thay nguyên tệp qua release mới). Nhưng nó vẫn cần **một số phiên bản lược đồ** để đường đọc của Story 1.11 từ chối một tệp mới hơn thay vì đọc sai im lặng — cùng lý lẽ đúng chữ của AD-30 (*"gặp phiên bản mới hơn thì từ chối mở"*). Một dòng khi dựng, một điều kiện khi đọc; ⛔ không dựng bộ máy di trú cho một tệp chỉ đọc.

#### #8 — Số dòng bỏ là DỮ LIỆU, không phải log

Mỗi parser đếm dòng đọc / dòng bỏ / lý do bỏ, và tổng in ra cuối lượt. Không có bảng này thì *"nguồn thứ tư đọc hỏng 90 %"* trông giống hệt *"nguồn thứ tư vốn nhỏ"*. Bảng vào §Debug Log References.

### Bàn giao từ các story trước — thứ ảnh hưởng trực tiếp

1. **Story 1.1 → NFR6.** Chênh lệch font đo thật: **21.285.713 byte = 21,29 MB**. Baseline app thăm dò 1,40 MB **⛔ không dùng lại** — nó là app một cửa sổ rỗng. Dư địa còn lại sau font và ba nguồn đầu: **47,31 MB** (`font-spike-results-2026-08-03.md:356-365`).
2. **Story 1.2 → cây nguồn và manifest.** `tools/dict-build/` và `dict-manifest.toml` đã tồn tại làm khung; `src-tauri/resources/dict/README.md` đã ghi *"Story sở hữu: 1.9"*. `.gitignore` đã có `*.db` **cố ý** — ⛔ đừng gỡ.
3. **Story 1.3 → CI.** Gắn cổng mới vào **job `check` đã có**, ⛔ không dựng pipeline thứ hai. Và giữ nguyên mệnh đề *"CI ⛔ không tải dữ liệu từ điển"* — cổng manifest phải xanh trên runner không có byte dữ liệu nào.
4. **Story 1.5 → NFR16 và hình dạng lỗi.** `check-i18n.mjs` Kiểm A quét ký tự có dấu ở **vị trí mã** trong `.rs`. Gốc quét hôm nay là `src/` + `src-tauri/`; story này mọc nhánh thứ ba (Task 9).
5. **Story 1.7 → `core::store`.** Khuôn viết DDL hằng (`schema.rs`), quy ước `user_version`, và bài học *"cây rỗng đọc thành sạch"* ⇒ **sàn số tệp** cho mọi cổng mới.
6. **Story 1.8 → cổng và macro.** Khuôn `scope_kinds!`/`message_keys!` (*một khai báo, nhiều thứ sinh ra*) áp được cho bảng năm nguồn: khai `code` · `display_name` · `license_*` · `attribution` ở **một** chỗ, ⛔ không rải năm chỗ.

### Nợ nhận lại — bốn mục `deferred-work.md` chạm story này

| Mục | Trạng thái story này giao |
|---|---|
| `:79` — *"`dict-manifest.toml` đặt ra luật ba trường rồi không cưỡng chế bằng gì cả"* | ✅ **Đóng** — Task 8 |
| `:44` — *"Gốc quét cứng ở `src/` và `src-tauri/` … mở lại khi cây mọc nhánh thứ ba"* | ✅ **Đóng** — Task 9, bằng *thêm gốc + `EXEMPT` có tên* |
| `:21` + `:57` — `$RESOURCE/dict/**` trong `assetProtocol.scope` nhưng không trong `bundle.resources`; và `connect-src` thiếu `asset:` | ✅ **Đóng** — Task 10, **Ice phê chuẩn 2026-08-04**: gỡ mục scope thừa. ⚠️ Kèm một mục **mới** cho Story 10.1 thay chỗ lưới vừa mất |
| `:23` + `:31` — `[profile.release]` giao lại cho *"Story 1.9 / 10.9"* | 🟡 **Đo và ghi số, ⛔ không sửa `Cargo.toml`** — §Quyết định của Ice #3 |

### Testing standards

- **Test của build tool nằm trong build tool.** `tools/dict-build/tests/` chạy bằng `cargo test --manifest-path tools/dict-build/Cargo.toml`, trên **fixture nhỏ đã commit**. ⛔ Không test nào của build tool được phụ thuộc vào nguồn thô đã tải.
- **`src-tauri/tests/` ⛔ KHÔNG thêm tệp nào ở story này.** Không có mã sản phẩm mới nào để nghiệm thu; thêm test ở đó là nghiệm thu một thứ không tồn tại.
- **Hai cổng `.mjs` nghiệm thu đỏ-rồi-xanh**, ghi từng ca vào §Debug Log References — tiền lệ 28 ca của `check-tokens.mjs`, 23 ca của `check-i18n.mjs`. ⚠️ Bản thân script `.mjs` **không được type-check và không có test** (`deferred-work.md:78`, `:101`) — đó là lý do đỏ-rồi-xanh bằng tay là lưới duy nhất, và nó phải được chạy thật chứ không mô tả.
- **Ba phép nghiệm thu trên dữ liệu thật** (Task 11) chạy **tay**, ghi kết quả kèm truy vấn SQL nguyên văn để lượt rà soát sau tái lập được.
- **Đối chứng âm bắt buộc cho AC5**, cả hai chiều — một test chỉ khẳng định `sense_fts` trả đúng `má` sẽ **vẫn xanh** nếu ai đó xoá hẳn `sense_fts_nd`.

### Thông tin kỹ thuật — nguồn thô, kiểm chứng 2026-08-04

| Nguồn | Tệp / điểm tải | Định dạng | Giấy phép | `source_version` lấy ở đâu |
|---|---|---|---|---|
| **CVDICT** | `github.com/ph0ngp/CVDICT` → `CVDICT.u8` | văn bản dòng, khuôn CC-CEDICT: `phồn giản [pinyin] /nghĩa/nghĩa/` | **CC BY-SA 4.0** | tag/commit của repo |
| **CC-CEDICT** | `mdbg.net/chinese/export/cedict/cedict_1_0_ts_utf-8_mdbg.txt.gz` | như trên | **CC BY-SA 4.0** | dòng header `#! date=` trong chính tệp |
| **Unihan** | `unicode.org` → `Unihan.zip` (kèm UCD của mỗi bản Unicode) | 8 tệp `.txt`, UTF-8/NFC, `U+XXXX<TAB>kProperty<TAB>value` | **Unicode License** | số phiên bản Unicode của bản UCD |
| **viwiktionary** | kaikki.org — bản trích **theo ngôn ngữ** của ấn bản `vi` | JSONL, một object mỗi dòng | **CC-BY-SA + GFDL** | ngày dump ghi trên trang tải |
| **en.wiktionary** *(cho tiếng Trung)* | kaikki.org — trang **Chinese** của ấn bản **tiếng Anh** | JSONL | **CC-BY-SA + GFDL** | ngày dump ghi trên trang tải |

⚠️ **Ba điều phải đọc kỹ:**

1. **Trường cần lấy ở JSONL của kaikki** (đo ở Giai đoạn 0, `phase-0-spike-results-2026-08-02.md:133-138`): `pos` · **`pos_title`** *(đã sẵn tiếng Việt ở viwiktionary — dùng thẳng, `pos_lang = 'vi'`)* · `senses[].glosses` · `senses[].examples[]` *(có `text` và bản dịch)*. Với ấn bản tiếng Anh thì `pos_title` là tiếng Anh ⇒ `pos_lang = 'en'` (FR35).
2. **Độ phủ đã biết trước, ⛔ đừng đọc thành lỗi parse:** viwiktionary phủ **2,76 %** đầu mục tiếng Trung của CVDICT và chỉ **0,067 %** có ví dụ. Con số thấp ở nhánh ZH của viwiktionary là **đúng**; con số thấp ở nhánh **EN** (kỳ vọng ~133.319 mục, 100 % có từ loại) mới là lỗi.
3. **Unihan cho âm Hán Việt** — nền của tab Hán Việt (FR33). Trường liên quan: `kVietnamese` trong `Unihan_Readings.txt`. Nó nạp vào `dict_entry.han_viet`, ⛔ **không** thành một hàng `dict_sense` (nó là **âm đọc**, không phải **nghĩa** — trộn hai thứ làm Panel Lookup hiện âm đọc như một định nghĩa).

**Phụ thuộc của build tool:** ⛔ **không** vào bảng Stack của `ARCHITECTURE-SPINE.md` — AD-25 nói giấy phép parser không ràng buộc sản phẩm. Nhưng **ghi vào `tools/dict-build/README.md`** kèm giấy phép từng crate, để lượt rà NFR15 sau không phải đoán và để §Quyết định của Ice có chỗ bám nếu Ice muốn siết.

### Project Structure Notes

Cây sau story này (chỉ phần đổi):

```text
AuraTranslate/
  tools/dict-build/
    Cargo.toml               # 🔴 [workspace] rỗng — AC4
    Cargo.lock               # commit — đây là binary
    README.md                # cập nhật: hình dạng đã chốt + giấy phép crate
    src/
      main.rs                # CLI: --raw <dir> --out <file>
      schema.rs              # DDL hằng (AC1, AC5)
      build.rs               # điều phối: parse → chèn → index → rebuild → vacuum
      sources/{cvdict,unihan,cc_cedict,viwiktionary,en_wiktionary}.rs
    tests/
      fixtures/              # 20–50 dòng mỗi nguồn, commit được
      parse.rs · schema.rs
    raw/ out/ work/          # .gitignore
  scripts/
    check-dict-build.mjs     # AC2, AC4
    check-dict-manifest.mjs  # AC3
  dict-manifest.toml         # [base] điền thật
  .github/workflows/ci.yml   # +2 bước trong job `check`
  package.json               # +2 script
  .gitignore                 # +3 dòng
  src-tauri/
    tauri.conf.json          # Task 10 — assetProtocol.scope còn ĐÚNG MỘT mục
    tests/config_invariants.rs   # Task 10 — sửa assertion VÀ đổi tên hàm
    resources/dict/README.md     # cập nhật
```

⛔ **Không tệp nào dưới `src-tauri/src/**` bị sửa ở story này.** Nếu diff chạm vào đó, dừng lại và đọc lại §Ranh giới phạm vi — gần như chắc chắn là đang cài trước một phần của Story 1.11 hoặc 1.13. *(`tauri.conf.json` và `tests/` nằm **ngoài** `src/`, và cả hai chỉ đổi vì Task 10.)*

**Đặt tên:** `snake_case` cho Rust; bảng SQL `snake_case` số ít (`dict_entry`, không `dict_entries`) — khớp `schema_migration_log` và `config_value` đã có. Thực thể tiếng Anh theo Consistency Conventions; `BaseLayer`/`DetachableLayer` là tên đã ánh xạ cho *lớp nền / lớp gỡ rời*.

### References

- **AC gốc của story:** [`epics.md`](../planning-artifacts/epics.md) §Story 1.9, dòng 1331–1371 *(gồm khối bổ sung NFR6 ngày 2026-08-03)*
- **FR27–FR41:** [`prd.md`](../planning-artifacts/prds/prd-AuraTranslate-2026-08-02/prd.md) §6.3, dòng 454–496
- **Bộ nguồn + giấy phép:** `prd.md` §8.2 dòng 886–896 · §8.3 dòng 898–921 · §8.6 dòng 933–952
- **NFR6 + giả định `[A2]`:** `prd.md:826`, `:832`, `:1022`
- **AD-7** (năm loại kho, `dict-core.db` chỉ đọc) · **AD-10** (lớp gỡ rời, trường giấy phép) · **AD-19** (không hợp nhất) · **AD-25** (artifact có checksum) · **AD-26** (ba nhánh) · **AD-27** (`remove_diacritics 0`): [`ARCHITECTURE-SPINE.md`](../planning-artifacts/architecture/architecture-AuraTranslate-2026-08-02/ARCHITECTURE-SPINE.md) dòng 119–151, 288–292, 326–342
- **Cây nguồn + sơ đồ đường dữ liệu:** `ARCHITECTURE-SPINE.md` dòng 644–744
- **Sàn SQLite** (trigram ≥ 3.34, `remove_diacritics 0` ≥ 3.27): `ARCHITECTURE-SPINE.md:638`
- **Số đo Giai đoạn 0:** [`phase-0-spike-results-2026-08-02.md`](../planning-artifacts/research/phase-0-spike-results-2026-08-02.md) — kích thước theo tầng `:55-69` · trigram rỗng `:75-89` · `char_idx` `:91-98` · dấu tiếng Việt `:110-125` · độ phủ kaikki `:129-169`
- **Số đo font + kế toán NFR6:** [`font-spike-results-2026-08-03.md`](../planning-artifacts/research/font-spike-results-2026-08-03.md) — `:78` quy ước đơn vị · `:82` số byte font · `:352-388` bảng dư địa và ba danh sách · `:437` chiều phép trừ đã đảo
- **Nợ đang mở:** [`deferred-work.md`](deferred-work.md) `:21` `:23` `:31` `:44` `:57` `:79`
- **Khuôn mã trong repo:** `src-tauri/src/core/store/schema.rs` (DDL hằng, `user_version`) · `src-tauri/tests/store_boundary.rs:44` (sàn số tệp) · `scripts/check-deps.mjs:15-27` (doctrine cổng) · `scripts/check-i18n.mjs` (§EXEMPT có tên)
- **Nguồn ngoài, kiểm 2026-08-04:** [CVDICT](https://github.com/ph0ngp/CVDICT) · [CC-CEDICT (MDBG)](https://www.mdbg.net/chinese/dictionary?page=cc-cedict) · [UAX #38 Unihan](https://www.unicode.org/reports/tr38/) · [kaikki.org raw data](https://kaikki.org/dictionary/rawdata.html)

### Quyết định của Ice — chốt 2026-08-04, ⛔ không phải lựa chọn của dev

#### #1 — AC6 đóng một phần; phần còn lại giao cho Story 1.10 ✅ **CHỐT**

AC6 nói *"**mọi** artifact dữ liệu sẽ đóng gói — `dict-core.db` **và** bốn lớp gỡ rời của Story 1.10"*. Nhưng Story 1.10 dùng chính build tool của story này, nên nó **không thể chạy trước**. Phép đối chiếu trần NFR6 vì vậy **không đóng trọn được hôm nay**.

**Ice chốt:** đo trọn phần đo được, ghi dòng *"bốn lớp gỡ rời — `[----] chưa đo, Story 1.10"*, phán quyết **CHƯA KẾT LUẬN ĐƯỢC**, và giao **dư địa còn lại bằng byte** cho Story 1.10 trừ tiếp. ⛔ Không gộp 1.10 vào 1.9 — AC của 1.10 còn mang metadata giấy phép và phép thử *xoá tệp* của FR36, tức một story thật chứ không phải phần đuôi.

⚠️ **Hệ quả phải nói ra:** *"CHƯA KẾT LUẬN ĐƯỢC"* là một phán quyết **hợp lệ và bắt buộc phải ghi đúng chữ đó**. ⛔ Không viết *"đạt"*, ⛔ không viết *"nằm trong trần"* — cùng mệnh đề ⛔ mà Story 1.3 đã áp cho `unmeasured` của AC8.

#### #2 — Gỡ `$RESOURCE/dict/**` khỏi `assetProtocol.scope` ✅ **CHỐT — ĐỒNG Ý GỠ**

`deferred-work.md:21` và `:57` giao mục này cho *"Story 1.9 / 10.1, story nào chốt trước thì giải luôn cả hai"*. Sự thật đã rõ hơn lúc mục đó được viết: **webview không bao giờ đọc tệp từ điển** — AD-1 và AD-11 đặt mọi truy cập dữ liệu ở Rust, và `rusqlite` mở tệp bằng đường dẫn hệ thống, ⛔ **không** đi qua asset protocol. Mục scope đó là **một quyền thừa**; mâu thuẫn với `connect-src` chỉ là hệ quả của việc nó thừa.

**Ice chốt 2026-08-04: gỡ.** Thi hành ở **Task 10**. Hai điều kiện kèm theo, ⛔ không được rơi:

1. **Lưới thay thế trước khi gỡ.** Sau lượt này không còn dòng nào trong `tauri.conf.json` nhắc tới `dict` cho tới Story 10.1 — ghi một mục `deferred-work.md` mới đích danh Story 10.1.
2. **⛔ Không sửa `ARCHITECTURE-SPINE.md`.** AD-23 (`:316`) còn liệt kê `$RESOURCE/dict/**`; sửa nó là lượt riêng của Ice. Dev ghi vào §Completion Notes rằng AD-23 **đang lệch khỏi cấu hình**.

⚠️ **Vì sao gỡ scope là đường đúng chứ không phải nới `connect-src`:** hai đường đều làm mâu thuẫn biến mất, nhưng một đường siết và một đường nới. `deferred-work.md:56` đã ghi tiền lệ của chính Ice ngày 2026-08-03: *"giữ nguyên CSP, không nới `connect-src` chỉ để một phép kiểm đo được."*

#### #3 — Hai khoản chi trong `Cargo.toml`: ⛔ **KHÔNG ĐỤNG** ✅ **CHỐT**

`deferred-work.md:31` giao lại cho *"Story 1.9 / 10.9"*, và `:71` [D4] ghi rõ **sửa hai chỗ này làm số `.dmg`/`.msi` khác đi, nên nếu làm thì phải làm trước khi chốt baseline NFR6**:

- `reqwest = "=0.13.4"` để nguyên default features ⇒ kéo `aws-lc-sys` (biên dịch từ nguồn C) vào mọi lượt cache lạnh, trong khi chính manifest tự khai *"chưa có một dòng mã nào gọi tới"*.
- `crate-type = ["staticlib", "cdylib", "rlib"]` phục vụ iOS/Android của template Tauri, trong khi `bundle.targets` chỉ có `["dmg","msi"]`.

**Ice chốt:** ⛔ **không đụng `Cargo.toml`** (lần thứ ba), **đo baseline như hiện trạng**, và ghi con số đó kèm ghi chú rằng nó **chưa phản ánh** hai khoản chi này. Nếu bảng kế toán AC6 cho ra một tổng **sát trần**, đây là hai đòn bẩy đầu tiên nên thử — và lúc đó nó thành một quyết định có dữ liệu thay vì một tối ưu mù. Ghi lại mục `deferred-work.md:31` với đúng trạng thái đó, ⛔ không đánh dấu đã đóng.

#### #4 — Dựng cả hai chỉ mục FTS; ~17 MB ghi thành dòng riêng ✅ **CHỐT**

Mức 130 MB của Giai đoạn 0 gồm **một** chỉ mục FTS trên nghĩa. AC5 đòi **hai**. Với dư địa còn **47,31 MB** trước khi cộng Unihan, en.wiktionary và toàn bộ mã sản phẩm, ~17 MB là một khoản đáng kể.

**Ice chốt:** **dựng cả hai** — AC5 là AC nghiệm thu, và FR9 (chế độ khoan dung) dựa vào bảng phụ. Ghi nó thành **dòng riêng** trong bảng kế toán để nếu phải cắt thì Ice cắt trên số thật. ⛔ Dev không tự bỏ chỉ mục phụ để cho vừa trần — đó đúng là *"tự bỏ một nguồn từ điển"* nói bằng cách khác, và AC6 cấm.

---

## Dev Agent Record

### Agent Model Used

_(điền lúc thực thi)_

### Debug Log References

#### Đường cơ sở (Task 1) — bảy lệnh trên cây làm việc

_(điền)_

#### Bảng dựng dữ liệu (Task 11) — nguồn → dòng đọc · dòng bỏ · bản ghi

_(điền)_

#### Bảng kế toán NFR6 (Task 12) — mọi dòng bằng byte

_(điền)_

#### Nghiệm thu đỏ-rồi-xanh — hai cổng mới

_(điền)_

#### Ba phép nghiệm thu trên dữ liệu thật (AC2 · AC5 · AD-26)

_(điền — kèm truy vấn SQL nguyên văn)_

#### Sau story (Task 14) — bảy lệnh cũ + hai cổng mới

_(điền)_

### Completion Notes List

_(điền)_

### File List

_(điền)_

## Change Log

| Ngày | Thay đổi |
|---|---|
| 2026-08-04 | Story tạo — phân tích context đầy đủ, trạng thái `ready-for-dev` |
| 2026-08-04 | Ice phán quyết bốn câu hỏi. #1 #3 #4 theo mặc định; **#2 đổi phạm vi** — đồng ý gỡ `$RESOURCE/dict/**` khỏi `assetProtocol.scope` ⇒ thêm **Task 10**, dồn số các task sau, §Câu hỏi cho Ice đổi thành §Quyết định của Ice |
