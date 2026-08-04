---
baseline_commit: 0ff36a0f8d77cddeee09e32306f0e427438d2e35
baseline_note: 'Cây làm việc tại 0ff36a0 CỘNG toàn bộ thay đổi CHƯA COMMIT của Story 1.8 — xem §Trạng thái repo hiện tại'
---

# Story 1.9: Dựng dữ liệu từ điển lớp nền

Status: done

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

- [x] **Task 1 — Đường cơ sở: chạy bảy lệnh trên cây sạch, ghi số vào §Debug Log References** (không AC)
  - [x] `npm run build` *(bắt buộc trước `cargo test` — `generate_context!` nhúng `dist/` lúc biên dịch)*
  - [x] `cargo test --manifest-path src-tauri/Cargo.toml` · `check:deps` · `check:tokens` · `check:i18n` · `check:commands` · `check:scope`
  - [x] Ghi lại: số tệp `.rs` dưới `src-tauri/src/**` · quần thể `check-i18n.mjs` · tổng số test Rust · số khoá `vi.json` · **số phụ thuộc trong cây Rust mà `check-deps.mjs` đếm được** *(số này là chứng cứ AC4 sau story — ghi cả trước lẫn sau)*
  - [x] ⛔ Không sửa gì ở task này. Một lệnh đỏ sẵn thì **dừng và báo**, không sửa lấn sang.

- [x] **Task 2 — Khung crate `tools/dict-build`, tách khỏi cây phụ thuộc sản phẩm** (AC4)
  - [x] `tools/dict-build/Cargo.toml` — 🔴 **có khối `[workspace]` rỗng** ở đầu tệp. Đây là toàn bộ AC4 ở tầng cấu trúc; thiếu nó thì một `Cargo.toml` workspace ở gốc repo về sau sẽ hút build tool vào cây sản phẩm mà không lỗi nào được ném.
  - [x] `edition = "2024"`, `rust-version` khớp toolchain CI (`1.97.1` — ⛔ không chép `1.85` của `src-tauri/Cargo.toml`, xem `deferred-work.md:83`).
  - [x] Phụ thuộc: `rusqlite` (feature `bundled`) · `serde` + `serde_json` (đọc JSONL của kaikki) · `sha2` (băm artifact) · giải nén Unihan.zip TAY ngoài tool, ⛔ không thêm crate `zip` (§Quyết định #6). ⛔ **Không** ghim `=` như `src-tauri`. Giấy phép từng crate ghi vào `tools/dict-build/README.md`.
  - [x] `tools/dict-build/Cargo.lock` **được commit**.
  - [x] `.gitignore`: thêm `tools/dict-build/raw/`, `tools/dict-build/out/`, `tools/dict-build/work/`. `*.db` giữ nguyên.
  - [x] Cập nhật `tools/dict-build/README.md` — hình dạng đã chốt.

- [x] **Task 3 — Lược đồ `dict-core.db`** (AC1, AC2, AC5)
  - [x] `tools/dict-build/src/schema.rs` — DDL là **hằng `&'static str`**, một hằng cho một khối logic.
  - [x] Bảy bảng thường + ba bảng ảo FTS5 theo §Quyết định #2: `dict_meta` · `dict_source` · `dict_entry` · `dict_sense` · `dict_example` · `dict_citation` · `char_idx` + `entry_fts`/`sense_fts`/`sense_fts_nd`.
  - [x] 🔴 `dict_sense.source_id INTEGER NOT NULL REFERENCES dict_source(id)` và `PRAGMA foreign_keys = ON` trước mọi lệnh chèn — nghiệm thu bằng test `dict_sense_source_id_rejects_null_by_schema` (`tests/parse.rs`).
  - [x] `PRAGMA user_version = 1` trên tệp sinh ra, và một hàng `dict_meta('schema_version','1')` — nghiệm thu `schema_version_is_recorded_in_both_places` (`tests/schema.rs`).
  - [x] Ba chỉ mục FTS5 external-content, `rebuild` một lượt ở cuối — nghiệm thu Bẫy 3 trực tiếp bằng test `fts_without_rebuild_silently_returns_zero_rows_not_an_error`.

- [x] **Task 4 — Năm parser, mỗi nguồn một module, chạy trên fixture trước** (AC1, AC2)
  - [x] `tools/dict-build/src/sources/{cvdict,unihan,cc_cedict,viwiktionary,en_wiktionary}.rs`
  - [x] Mỗi module phơi cùng chữ ký `fn parse(reader) -> impl Iterator<Item = Result<RawEntry, ParseIssue>>`.
  - [x] 🔴 Mỗi module ghi `source_version` đo lúc chạy: CVDICT từ `SOURCE_VERSION.txt` (commit sha thật `c379d909…`) · CC-CEDICT từ dòng `#! date=` trong chính tệp · Unihan từ `# Unicode Version 17.0.0` trong header · hai Wiktionary từ mtime tệp tải (không có header ngày trong nội dung).
  - [x] Fixture cho cả năm nguồn dưới `tools/dict-build/tests/fixtures/raw/<nguồn>/`, trích THẬT từ dữ liệu đã tải 2026-08-04 (không bịa). `山` có mặt ở CẢ CVDICT lẫn CC-CEDICT fixture (dòng thật 35089-35090 / 35485-35486) — nghiệm thu AC2 tại `ac2_shan_appears_under_two_different_source_ids_not_merged`.
  - [x] Ca lỗi bắt buộc: dòng hỏng ⇒ đếm + báo cáo qua `ParseIssue`, ⛔ không `panic!`. `model::SourceStats` gom `skip_reasons` theo lý do, in bảng cuối lượt (`build::print_report`).

- [x] **Task 5 — `char_idx` phủ cả phồn thể lẫn giản thể** (AC1)
  - [x] `src/char_idx.rs::insert_for_entry` — cặp `(ch, entry_id)` cho mọi ký tự Hán (dải CJK + mở rộng A–G) trong `headword` VÀ `headword_simp`. Nghiệm thu Bẫy 8 bằng fixture thật (`U+570B kSimplifiedVariant U+56FD`): test `char_idx_covers_both_traditional_and_simplified_forms`.
  - [x] `WITHOUT ROWID` + khoá chính `(ch, entry_id)`.
  - [x] Số cặp thật ghi ở Task 11 (chạy trên năm nguồn thật, không phải fixture).

- [x] **Task 6 — Hoàn tất tệp: `ANALYZE`, `VACUUM`, và 🔴 `journal_mode = DELETE`** (AC1, AC6)
  - [x] `finalize::rebuild_fts` — `rebuild` cả ba bảng FTS trước `VACUUM`.
  - [x] `finalize::analyze_and_vacuum` — `ANALYZE` rồi `VACUUM`.
  - [x] 🔴 `finalize::set_journal_mode_delete` — `PRAGMA journal_mode = DELETE`. Nghiệm thu Bẫy 1 bằng test `built_file_uses_delete_journal_mode_with_no_wal_artifacts` (trên fixture) — kiểm PRAGMA VÀ không tệp `-wal`/`-shm` nào sót.
  - [x] `finalize::verify_no_wal_artifacts` chạy sau khi đóng kết nối, ném lỗi rõ ràng nếu vi phạm.
  - [x] `finalize::sha256_and_size` — in SHA-256 + kích thước byte, nghiệm thu bằng vector chuẩn NIST (chuỗi rỗng) ở `finalize::tests::sha256_matches_a_known_vector`.

- [x] **Task 7 — Cổng `scripts/check-dict-build.mjs`** (AC2, AC4)
  - [x] **Kiểm A — từ vựng hợp nhất.** Quét `tools/dict-build/src/**/*.rs` tìm danh sách token (§Quyết định #4). Miễn trừ khai bằng comment `// dict-build:allow <token> — <lý do>`, quét NGƯỢC LÊN qua khối comment liền trước (một dòng mã có thể cần nhiều miễn trừ). Cổng **in ra số miễn trừ mỗi lượt chạy** (3 hôm nay).
  - [x] **Kiểm B — cách ly workspace.** `tools/dict-build/Cargo.toml` có `[workspace]`; ⛔ không phụ thuộc nào trỏ `path` sang `src-tauri`; ⛔ không `workspace = true`.
  - [x] **Kiểm C — sàn số tệp.** Cây rỗng ⛔ không được đọc thành sạch (sàn 10, số thật hôm nay 18).
  - [x] Nghiệm thu **đỏ-rồi-xanh**: 8 đối chứng âm (A×3, B×3, C×2 — mỗi kiểm ≥ 2), ghi vào §Debug Log References.

- [x] **Task 8 — Cổng `scripts/check-dict-manifest.mjs`** (AC3)
  - [x] Parser TOML **tập con nghiêm ngặt, tự viết** — `[section]` / `[[array_of_tables]]` / `key = "chuỗi nháy kép"` / comment `#`. Cú pháp ngoài tập con ⇒ **FAIL**.
  - [x] Luật: `[base]` **phải có mặt**; mỗi mục đủ `url` · `sha256` · `source_version`; `sha256` khớp `/^[0-9a-f]{64}$/`; `url` bắt đầu `https://` và chứa `/releases/download/dict-v`; `source_version` không rỗng; `[[detachable]]` còn đòi thêm `name`.
  - [x] Cổng không đọc `.db`, không gọi network — chỉ `readFileSync` trên `dict-manifest.toml`.
  - [x] Nghiệm thu đỏ-rồi-xanh **11 ca** (1 dương + 10 âm), ghi vào §Debug Log References.

- [x] **Task 9 — Gắn hai cổng vào CI, và xử món nợ gốc quét của `check-i18n.mjs`** (AC2, AC3)
  - [x] `package.json`: `check:dict` và `check:dict-manifest`.
  - [x] `ci.yml` job `check`: hai bước mới, đặt kề `check:deps` (ngay sau) và trước `check:tokens`/`npm run build` — không dựng pipeline thứ hai.
  - [x] 🔴 **Món nợ `deferred-work.md:44` xử ở chính story này.** Thêm `tools/` vào gốc quét của `check-i18n.mjs`, miễn trừ TRỌN trong `EXEMPT` kèm tên + lý do.
  - [x] Cập nhật `deferred-work.md:44` thành đã xử.
  - [x] Quần thể sau miễn trừ **không đổi**: vẫn 27 `.rs` + 5 `.vue` — nghiệm thu bằng cách chạy thật `npm run check:i18n` trước/sau, so số.
  - [x] Cả hai cổng mới dùng `path.join` xuyên suốt, `posix()` để so sánh, và `split(/\r\n|\n/)` khi tách dòng — không spawn `npm`/`npx` nên không dính bẫy Windows `.cmd`.

- [x] **Task 10 — Gỡ `$RESOURCE/dict/**` khỏi `assetProtocol.scope`** (không AC — **Ice phê chuẩn 2026-08-04**, đóng `deferred-work.md:21` + `:57`)
  - [x] `src-tauri/tauri.conf.json:28` — `scope` còn **đúng một** mục: `["$RESOURCE/fonts/**"]`.
  - [x] `src-tauri/tests/config_invariants.rs` — sửa assertion **và đổi tên hàm** thành `asset_protocol_scope_has_exactly_the_one_readonly_resource_area`.
  - [x] Lý do viết vào chính test (doc-comment): webview không bao giờ đọc tệp từ điển — AD-1/AD-11, `rusqlite` mở bằng đường dẫn hệ thống.
  - [x] `npm run check:scope` và `npm run check:scope:bundled` chạy lại — cả hai xanh **không đổi một dòng output** (đối chiếu byte-for-byte với baseline Task 1).
  - [x] Lưới thay thế: mục `deferred-work.md` mới, đích danh **Story 10.1** (thêm `dict/*.db` vào `bundle.resources` **và** một test khẳng định nó có mặt).
  - [x] ⛔ Không sửa `ARCHITECTURE-SPINE.md`. Ghi lệch khỏi cấu hình vào `deferred-work.md` (không chỉ vào story) — xem mục mới thêm.

- [x] **Task 11 — Chạy thật trên năm nguồn, ghi bảng số** (AC1, AC2, AC5)
  - [x] Tải năm nguồn thật (CVDICT.u8 10,8 MB · CC-CEDICT 9,8 MB · Unihan.zip giải nén 41 MB · vi-extract.jsonl 273 MB · Chinese.jsonl 1,18 GB) vào `tools/dict-build/raw/`.
  - [x] Chạy build tool (bản `--release`, 37 phút 49 giây). Bảng đầy đủ ghi ở §Debug Log References.
  - [x] Đối chiếu Giai đoạn 0: **680.709 nghĩa** (+12,7%) · **90.475 ví dụ** (+223,6%) · `char_idx` **1.371.273** cặp (+5,7%) — cả ba **lớn hơn** ba nguồn của Giai đoạn 0.
  - [x] Ba phép nghiệm thu trên tệp thật (`out/dict-core.db`, 154.836.992 byte), SQL nguyên văn ghi ở §Debug Log References:
    - **AC2** — `山` cho ra **4** `source_id` khác nhau (vượt yêu cầu ≥2), không hàng nào mang nhiều nguồn gộp lại.
    - **AC5** — hai chiều xác nhận trên `dict_sense.id=49797` (gloss thật `'má'`): chỉ mục chính từ chối `ma`, chỉ mục phụ chấp nhận.
    - **AD-26** — `entry_fts MATCH '中國人'` → 33 hàng; `char_idx` có cả `山`(3.244) · `中`(3.023) · `國`(3.164) · `国`(2.032) — phồn VÀ giản đều phủ.

- [x] **Task 12 — Kế toán NFR6** (AC6)
  - [x] Dựng `.dmg` của cây nguồn **hôm nay** (`--config tauri.nofonts.conf.json`) → baseline thật **2.334.696 byte**, macOS x86_64 (so được với số Story 1.1).
  - [x] Bảng kế toán AC6 dựng đủ, mọi dòng bằng byte trước, đổi MB thập phân sau.
  - [x] *"Bốn lớp gỡ rời"* ghi `[----] chưa đo — Story 1.10`. Phán quyết: **CHƯA KẾT LUẬN ĐƯỢC**, dư địa còn lại **21.507.450 byte**.
  - [x] Tổng đo được (178.492.550 byte) **KHÔNG** vượt trần một mình — không kích hoạt điều kiện dừng của subtask cuối.

- [x] **Task 13 — Điền `dict-manifest.toml` tới ranh giới dev làm được** (AC3)
  - [x] Bỏ comment `[base]`, điền `sha256` = `358cf0f8afcc52c210caa205cd1b0b175eb9562de1b0917e48850a629cd8bdb5` (thật, đo ở Task 6/11), `source_version` = chuỗi ghép năm nguồn.
  - [x] `url` đúng dạng, đúng tag `dict-v1` — release chưa tồn tại, ghi rõ ở §Completion Notes.
  - [x] Lệnh chép-dán cho Ice ghi ở §Completion Notes.
  - [x] `npm run check:dict-manifest` xanh trên manifest đã điền.

- [x] **Task 14 — Chốt sổ** (không AC)
  - [x] Chạy lại trọn bộ lệnh của Task 1 cộng hai cổng mới; ghi số sau story — không lệnh nào đỏ.
  - [x] Cập nhật `src-tauri/resources/dict/README.md`: tệp tồn tại, nguồn, chủ sở hữu, và xác nhận vùng này không còn trong `assetProtocol.scope`.
  - [x] Cập nhật `deferred-work.md`: đóng `:44` · `:79` · `:21` · `:57`; cập nhật `[D4]` với số thật; thêm mục mới cho **Story 10.1**.
  - [x] §Quyết định của Ice #1: phán quyết thật AC6 = **CHƯA KẾT LUẬN ĐƯỢC**, ghi ở Debug Log References + Completion Notes.

### Review Findings

> Review chạy theo `bmad-code-review` (Blind Hunter + Edge Case Hunter + Acceptance Auditor), chia 3 nhóm vì diff > 3000 dòng. Nhóm dưới đây: **Group A — lõi Rust của `tools/dict-build`** (`Cargo.toml`, `src/*.rs`, `src/sources/*.rs`, 2075 dòng). Group B (tests+fixtures) và Group C (tooling/CI/config/docs) sẽ được thêm vào mục này ở các lượt sau.
>
> **Cả 15 patch đã áp dụng 2026-08-04.** `cargo build` sạch, `cargo test` (crate `dict-build`) 55/55 xanh (40 unit + 8 integration `tests/parse.rs` trên fixture thật + 7 `tests/schema.rs`) — bao gồm test tích hợp trọn `build::run` trên fixture, nên các patch chạm `run()` (transaction, temp-file+rename, journal_mode, zero-entries) đã được nghiệm thu end-to-end, không chỉ unit test.

- [x] [Review][Patch] Nhiều dòng cùng đầu mục, khác từ loại (Wiktionary) sinh nhiều hàng `dict_entry` thay vì một entry nhiều `dict_sense` — `wiktextract_common.rs` sinh một `RawEntry` cho MỖI dòng JSONL; `insert_entry` (`insert.rs:47`) luôn `INSERT` một `dict_entry` mới, không tra/gộp theo `headword` trong cùng nguồn. Xác nhận bằng dữ liệu thật: `tests/fixtures/raw/en_wiktionary/Chinese.jsonl` có đầu mục `馬` xuất hiện ở HAI dòng JSONL riêng, cả hai `pos: "character"` — sinh ra hai `entry_id` khác nhau cho cùng một từ. AC1 mệnh đề 4 nói "một từ nhiều từ loại ⇒ nhiều hàng `dict_sense`" — hàm ý MỘT `entry_id` mỗi đầu mục mỗi nguồn; code cũ chẻ thành nhiều `entry_id`. **Ice chốt 2026-08-04: gộp theo headword TRONG một nguồn.** Đã sửa: `wiktextract_common::parse` mới gom mọi dòng JSONL cùng `headword` (trong cùng lượt gọi/nguồn) thành một `RawEntry` nhiều `RawSense` trước khi trả về; `viwiktionary.rs`/`en_wiktionary.rs` chỉ còn gọi hàm dùng chung này. Test mới: `same_headword_on_two_lines_merges_into_one_entry_with_two_senses`, `different_headwords_stay_as_separate_entries`. [tools/dict-build/src/sources/wiktextract_common.rs, tools/dict-build/src/sources/viwiktionary.rs, tools/dict-build/src/sources/en_wiktionary.rs]
- [x] [Review][Patch] Đường dẫn WAL/SHM tính sai khi `--out` không có đúng đuôi `.db` — vô hiệu hoá lưới chặn Bẫy 1. Đã sửa: `finalize::sibling_path` nối thẳng hậu tố `-wal`/`-shm` vào tên tệp đầy đủ (khớp cách SQLite thật sự đặt tên) thay vì `with_extension`; dùng ở cả lượt dọn đầu build lẫn `verify_no_wal_artifacts`. Test mới: `detects_leftover_wal_file_when_out_path_has_no_dot_db_extension`. [tools/dict-build/src/finalize.rs, tools/dict-build/src/build.rs]
- [x] [Review][Patch] Build không dừng/báo khi một nguồn cho ra 0 entry, trái lời hứa trong doc-comment của `BuildReport`. Đã sửa: `require_nonempty` gọi sau mỗi nguồn, dừng build với thông báo rõ nếu `entries == 0`. [tools/dict-build/src/build.rs]
- [x] [Review][Patch] Không có transaction SQL bao lượt chèn — mỗi `INSERT` tự autocommit; doc-comment `char_idx.rs` nói sai là "cùng giao dịch". Đã sửa: `insert_meta` + cả năm nguồn giờ chạy trong MỘT `conn.transaction()`, `commit()` trước khi `VACUUM`/đổi `journal_mode` (hai lệnh này không chạy được trong transaction) — doc-comment `char_idx.rs` giờ đúng nghĩa đen, không cần sửa. [tools/dict-build/src/build.rs]
- [x] [Review][Patch] `journal_mode` trả về từ PRAGMA không được xác nhận bằng `"delete"` trước khi coi build là thành công. Đã sửa: `run()` trả lỗi nếu giá trị trả về khác `"delete"` (không phân biệt hoa/thường). [tools/dict-build/src/build.rs]
- [x] [Review][Patch] `source_version` của cả năm nguồn âm thầm rơi về chuỗi `"unknown"` khi đo hỏng, không log cảnh báo — `"unknown"` không rỗng nên lọt qua `check-dict-manifest.mjs`. Đã sửa: `version_or_warn` in cảnh báo ra stderr mỗi khi rơi về `"unknown"`, dùng cho cả năm nguồn. [tools/dict-build/src/build.rs]
- [x] [Review][Patch] Đọc header CC-CEDICT và nội dung Unihan bằng `read_lines`/`read_to_string` toàn tệp — đọc trùng lặp và hỏng cả lượt build nếu có 1 byte UTF-8 lỗi, thay vì đếm theo dòng như phần còn lại của cây. Đã sửa: `read_header_lines` chỉ đọc tối đa N dòng đầu (bỏ qua dòng lỗi UTF-8 thay vì hỏng cả đọc) cho CC-CEDICT/Unihan; nội dung Unihan dùng để parse đổi sang `std::fs::read` (byte thô) thay vì `read_to_string`, để lỗi UTF-8 cục bộ rơi đúng vào `ParseIssue` theo dòng như `sources::unihan::parse` đã viết sẵn. [tools/dict-build/src/build.rs]
- [x] [Review][Patch] Unihan `kSimplifiedVariant` với codepoint không phân tích được bị bỏ âm thầm, không `ParseIssue` — ký tự có thể biến mất hoàn toàn khỏi output không dấu vết. Đã sửa: đẩy `ParseIssue` khi `parse_codepoint` thất bại. Test mới: `unparseable_ksimplifiedvariant_codepoint_is_reported_not_silently_dropped`. [tools/dict-build/src/sources/unihan.rs]
- [x] [Review][Patch] `dict_example.sense_id` và `dict_citation.sense_id` không có chỉ mục, khác với `dict_sense.entry_id` — ảnh hưởng hiệu năng đường đọc Story 1.11. Đã sửa: thêm `idx_example_sense`, `idx_citation_sense` vào `ENTRY_INDEXES_DDL`. [tools/dict-build/src/schema.rs]
- [x] [Review][Patch] Lượt build hỏng giữa chừng để lại tệp `.db` dở dang tại `out_path` không có dấu hiệu trên đĩa (chỉ phân biệt được qua exit code/stderr); tệp KHÔNG ở chế độ WAL như nghi ngờ ban đầu — `journal_mode` chỉ đổi sang DELETE ở cuối, không nơi nào bật WAL. Đã sửa: `run()` dựng vào tệp tạm (`sibling_path(out_path, ".tmp")`) cùng thư mục, chỉ `rename` sang `out_path` SAU khi mọi bước (rebuild FTS, VACUUM, journal_mode, verify, băm) đã xong — build hỏng giữa chừng không còn để lại gì tại `out_path`. [tools/dict-build/src/build.rs]
- [x] [Review][Patch] Unihan `kDefinition` rỗng/toàn khoảng trắng sinh hàng `dict_sense` với `gloss` rỗng, khác với guard tương đương của `cedict_common::parse_line`. **Điều tra sâu hơn khi viết test cho thấy kịch bản này KHÔNG tái lập được**: `raw.trim()` ở đầu vòng lặp (dòng 58) đã xén cả TAB phân cách cuối cùng của một dòng `kDefinition\t<rỗng>`, nên dòng đó rơi vào lỗi "expected 3 tab-separated fields" TRƯỚC KHI tới nhánh gán `gloss` — không có đường nào tới được `Some("")`. Không thêm canh gác chết (không khớp quy ước dự án: không validate kịch bản không xảy ra được). Test khoá hành vi thật: `kdefinition_with_nothing_after_the_trailing_tab_is_a_field_count_error`. Phần THẬT sự cần và ĐÃ sửa ở vị trí này: khử trùng lặp `kDefinition` (xem finding "Unihan thuộc tính trùng lặp" ngay dưới). [tools/dict-build/src/sources/unihan.rs]
- [x] [Review][Patch] `SourceMeta::license_text()` suy lại giấy phép qua so khớp chuỗi `code`, `unreachable!()` khi không khớp — trùng lặp dữ liệu đã có sẵn 1-1 trên từng hằng `SourceMeta`. Đã sửa: thêm enum đóng `LicenseRef` (`CcBySa4`/`UnicodeV3`/`CcBySaAndGfdl`), khai trực tiếp trên từng hằng `SourceMeta`; `license_text()` match trên enum, không còn nhánh `unreachable!()`. [tools/dict-build/src/sources_meta.rs]
- [x] [Review][Patch] Dòng thuộc tính Unihan trùng lặp cho cùng ký tự bị ghi đè âm thầm, không `ParseIssue`, không comment lý do như quy tắc khử-trùng-lặp-trong-nguồn của AC2 đòi hỏi. Đã sửa: cả bốn thuộc tính (`kDefinition`/`kMandarin`/`kVietnamese`/`kSimplifiedVariant`) đẩy `ParseIssue` khi gặp giá trị thứ hai cho cùng ký tự, giữ giá trị đầu. Test mới: `duplicate_property_line_keeps_first_value_and_reports_an_issue`. [tools/dict-build/src/sources/unihan.rs]
- [x] [Review][Patch] Doc-comment `wiktextract_common::parse_line` nói dòng bị lọc `lang_code` trả `Ok(None)`; code thật trả `Err` (hành vi vẫn đúng, chỉ sai mô tả). Đã sửa doc-comment cho khớp hành vi thật. [tools/dict-build/src/sources/wiktextract_common.rs]
- [x] [Review][Patch] `finalize::sha256_and_size` nạp trọn tệp output (~155MB, sẽ còn lớn hơn ở Story 1.10) vào bộ nhớ thay vì băm theo luồng. Đã sửa: băm theo khối 64 KiB qua `std::fs::File::read`, không còn `std::fs::read` nạp trọn tệp. [tools/dict-build/src/finalize.rs]

**Group B — tests + fixtures** (`tools/dict-build/tests/**`, 561 dòng). Review chạy sau khi Group A đã vá xong, trên cây nguồn đã sửa. **8 patch đã áp dụng 2026-08-04**, cộng 1 defer + 1 dismiss. `cargo test` sau Group B: **62/62 xanh** (40 unit + 13 `tests/parse.rs` + 9 `tests/schema.rs`, tăng từ 55 vì 7 test mới).

- [x] [Review][Patch] 🔴 **Fixture `en_wiktionary/Chinese.jsonl` là dữ liệu BỊA, vi phạm trực tiếp yêu cầu "không bịa" của Task 4** và lời khẳng định trong chính doc-comment của `tests/parse.rs` ("trích từ... kaikki.org thật, không phải bịa"). Acceptance Auditor đối chiếu byte-for-byte với `tools/dict-build/raw/en_wiktionary/Chinese.jsonl` thật (còn tồn tại cục bộ từ lượt chạy Task 11) và xác nhận **0/20 dòng khớp thật**; mọi từ loại của các đầu mục một-ký-tự (犬 西 水 火 土 木 金 愛 很 我 人) đều bị GÁN SAI (ví dụ toàn bộ đều thật ra chỉ có `pos: "character"`), và ví dụ dẫn chứng của `詞典` bị ghép từ hai đầu mục thật không liên quan. Bốn fixture còn lại (CVDICT, CC-CEDICT, cả hai tệp Unihan, viwiktionary) đều khớp thật 100%. **Đã sửa:** trích lại `en_wiktionary/Chinese.jsonl` (17 dòng) TRỰC TIẾP từ tệp thật bằng `jq`, giữ nguyên mọi giá trị (word/pos/lang_code/sounds/glosses/examples), chỉ cắt bớt số lượng nghĩa/ví dụ mỗi dòng cho gọn — không đổi giá trị nào. Fixture mới **mạnh hơn** bản cũ: `馬` giờ có **ba** dòng thật cùng đầu mục (hai dòng nghĩa dùng được + một dòng `tags:["no-gloss"]` thật, không phải bịa), và `A` có thêm một ca gộp thật thứ hai (`pos: "verb"` + `pos: "adj"` cùng đầu mục) — cả hai đều exercise đúng fix gộp-theo-headword của Group A bằng dữ liệu thật. Dòng cú pháp hỏng (thiếu `word`) vẫn giữ lại nguyên — đây là đầu dò kiểm thử tổng hợp có chủ đích cho `ParseIssue`, không giả làm mục từ điển thật nên không vi phạm "không bịa". [tools/dict-build/tests/fixtures/raw/en_wiktionary/Chinese.jsonl, tools/dict-build/src/sources/wiktextract_common.rs (doc-comment)]
- [x] [Review][Patch] Không test nào assert trên `skip_reasons`/`lines_skipped` của `BuildReport` cho bất kỳ nguồn nào, dù fixture có sẵn các dòng cố ý gây bỏ (dòng hỏng cú pháp của en_wiktionary, dòng lọc `lang_code`/`no-gloss` của viwiktionary). Đã sửa: thêm assertion trong `all_five_sources_produce_at_least_one_entry` kiểm `skip_reasons` của `en-wiktionary` chứa đúng lý do "missing 'word' field". [tools/dict-build/tests/parse.rs]
- [x] [Review][Patch] Không có assertion tích hợp nào khoá lại fix gộp-theo-headword của Group A (query `dict_entry`/`dict_sense` qua `build::run` thật cho `馬`). Đã sửa: test mới `en_wiktionary_same_headword_ma_merges_into_one_entry_with_multiple_senses` kiểm `馬` cho ra **đúng 1** `dict_entry` và **nhiều** `dict_sense` (gộp từ ≥2 dòng JSONL thật), chạy qua pipeline `build::run` thật trên fixture. [tools/dict-build/tests/parse.rs]
- [x] [Review][Patch] `built_file_uses_delete_journal_mode_with_no_wal_artifacts` luôn dựng vào đường dẫn cố định có đuôi `.db`, nên không bao giờ exercise được lời hứa "đúng bất kể đuôi tệp" của patch WAL-path Group A. Đã sửa: thêm test `wal_check_survives_output_path_without_dot_db_extension` dựng vào đường dẫn KHÔNG có đuôi `.db`. [tools/dict-build/tests/parse.rs]
- [x] [Review][Patch] Không test nào dựng vào `out_path` đã có sẵn tệp/`-wal`/`-shm` sót từ trước — đúng kịch bản patch tệp-tạm-rồi-rename của Group A nhắm tới. Đã sửa: test mới `rebuilding_over_a_stale_output_and_wal_artifacts_still_succeeds` ghi rác vào `out_path` + `-wal`/`-shm` cạnh nó trước khi gọi `build::run`, xác nhận build vẫn thành công và dọn sạch. [tools/dict-build/tests/parse.rs]
- [x] [Review][Patch] `ac2_shan_appears_under_two_different_source_ids_not_merged` dùng ngưỡng `entry_count >= 2` lỏng hơn dữ liệu thật hỗ trợ (thật ra `entry_count` cho `山` ≥ 6 tính cả CVDICT/CC-CEDICT hai dòng mỗi nguồn), và comment nói "mỗi nguồn có dict_entry RIÊNG" nhưng không kiểm đúng-một-entry-mỗi-source_id. Đã sửa: siết assertion kiểm mỗi `source_id` xuất hiện đúng bằng số dòng thật của nguồn đó cho `山` trong CVDICT/CC-CEDICT (2 mỗi nguồn, không gộp/không thiếu), sửa lại comment cho khớp. [tools/dict-build/tests/parse.rs]
- [x] [Review][Patch] `source_version` chưa từng được assert cho bất kỳ nguồn nào trong 5 nguồn — đúng lỗi "rơi về unknown âm thầm" mà Group A đã vá sẽ lọt qua bộ test này nếu tái phát. Đã sửa: test mới `all_sources_have_a_real_non_unknown_source_version` kiểm `dict_source.source_version` khác `"unknown"` và không rỗng cho cả năm nguồn. [tools/dict-build/tests/parse.rs]
- [x] [Review][Patch] `build_report_includes_a_real_sha256_and_matching_size` chưa từng kiểm hash có ĐÚNG không, chỉ kiểm hình dạng (64 hex thường) và kích thước khớp. Đã sửa: test tính lại SHA-256 thật của tệp output và so với `report.sha256`. [tools/dict-build/tests/parse.rs]
- [x] [Review][Patch] Một số test `schema.rs` chỉ kiểm DDL có chứa chuỗi con (`examples_and_citations_reference_sense_not_entry`, `char_idx_is_without_rowid_with_composite_primary_key`) mà chưa từng chèn thật để chứng minh ràng buộc CÓ HIỆU LỰC lúc chạy. Đã sửa: cả hai test giờ chèn thật — kiểm FK `sense_id` sai bị SQLite từ chối, và kiểm cặp `(ch, entry_id)` trùng bị `char_idx`'s khoá chính từ chối. [tools/dict-build/tests/schema.rs]
- [ ] [Review][Defer] Fixture âm tính (dòng hỏng cú pháp cố ý) chỉ có ở `en_wiktionary`; bốn nguồn còn lại (CVDICT/CC-CEDICT/Unihan/viwiktionary) không có ca hỏng trong fixture tích hợp — dù đã có unit test tổng hợp đầy đủ ở tầng Group A cho từng ca này. Giá trị tăng thêm nhỏ so với công thêm (đặc biệt với Unihan — nguồn Unicode chính thức, thêm dòng hỏng "thật" đòi cân nhắc lại ranh giới "không bịa" của chính finding này). — deferred, giá trị nhỏ so với chi phí, đã có unit test tổng hợp bù đắp ở Group A
- [x] [Review][Dismiss] `viwiktionary/vi-extract.jsonl` có 19 dòng, thiếu 1 dòng so với sàn "20–50 dòng" ghi ở Dev Notes — nhưng dữ liệu 100% thật (khớp byte-for-byte) và đủ chức năng (phủ lọc `lang_code`, `no-gloss`, dạng `hard-redirect`). Auditor đánh giá đây là lệch tài liệu-thực tế nhỏ, không phải vấn đề chất lượng test thật.

**Group C — tooling/CI/config/docs** (`scripts/check-dict-build.mjs`, `scripts/check-dict-manifest.mjs`, `ci.yml`, `package.json`, `.gitignore`, `dict-manifest.toml`, `tauri.conf.json`, `config_invariants.rs`, READMEs, `deferred-work.md`, `sprint-status.yaml`; 765 dòng). Review chạy sau khi Group A/B đã vá xong. **9 patch đã áp dụng 2026-08-04**, cộng 5 defer + 3 dismiss. Mọi fix trong `.mjs` đã nghiệm thu đỏ-rồi-xanh bằng tay (sao lưu tệp thật, gây lỗi giả, xác nhận cổng bắt được, khôi phục, `diff` xác nhận IDENTICAL) trước khi tính là xong.

- [x] [Review][Patch] 🔴 **`check:dict` (cổng Kiểm A) hiện ĐANG ĐỎ trên cây nguồn thật** — mâu thuẫn trực tiếp với Debug Log của story ("✅ xanh — 18 tệp .rs, 3 miễn trừ") và sẽ làm CI đỏ vì `ci.yml` gọi `npm run check:dict` không điều kiện, không `continue-on-error`. Nguyên nhân: test mới `same_headword_on_two_lines_merges_into_one_entry_with_two_senses` (thêm ở Group A) chứa từ tiếng Anh "merges" khớp token cấm `'merge'` — hệ quả trực tiếp của việc so khớp CHUỖI CON THUẦN TUÝ, không có ranh giới từ. Đã sửa: đổi tên hàm test thành `same_headword_on_two_lines_becomes_one_entry_with_two_senses` (né đúng cách các đoạn code khác trong cây đã né các từ cấm). Xác nhận `node scripts/check-dict-build.mjs` xanh trở lại, `cargo test` vẫn 62/62. [tools/dict-build/src/sources/wiktextract_common.rs]
- [x] [Review][Patch] Kiểm A so khớp CHUỖI CON có phân biệt hoa/thường — một định danh PascalCase (`struct Merger`, `impl Combine for X`) không bao giờ khớp `'merge'`/`'combine_senses'` chữ thường; lời gọi UFCS tường minh (`HashMap::entry(&mut map, k)`) né được `.entry(` có chấm. Đã sửa: so khớp KHÔNG phân biệt hoa/thường, và thêm `'::entry('` vào danh sách token cấm để bắt dạng UFCS. Nghiệm thu đỏ-rồi-xanh: chèn `struct DictMerger {}` thật (không phải comment) → bắt được; khôi phục → xanh lại. [scripts/check-dict-build.mjs]
- [x] [Review][Patch] Kiểm B (cách ly workspace, AC4) chỉ kiểm SỰ CÓ MẶT của dòng `[workspace]`, không kiểm THÂN của nó rỗng — `[workspace]\nmembers = ["../../src-tauri"]` qua được cổng dù chính là thứ AC4 cấm (hai cây phụ thuộc giao nhau). Đã sửa: sau khi khớp `[workspace]`, quét các dòng theo sau tới `[section]`/`[[array]]` kế tiếp hoặc hết tệp, FAIL nếu có bất kỳ dòng gán khoá nào (`members`, `exclude`, …) trong thân đó. [scripts/check-dict-build.mjs]
- [x] [Review][Patch] Kiểm B chỉ khớp `path = "..."` nháy KÉP — TOML/Cargo cũng hợp lệ với nháy ĐƠN (`path = '../../src-tauri'`), lọt qua cổng nguyên vẹn. Đã sửa: regex khớp cả hai kiểu nháy. [scripts/check-dict-build.mjs]
- [x] [Review][Patch] `check-dict-manifest.mjs`'s `URL_RE` không ghim host/tổ chức/repo — bất kỳ domain HTTPS nào chứa chuỗi con `/releases/download/dict-v` đều qua được (vd. `https://evil.example.com/x/releases/download/dict-v1/dict-core.db`), trong khi đây chính là URL sẽ được tải xuống máy người dùng. Đã sửa: ghim đúng `github.com/vannamhh/AuraTranslate/releases/download/dict-v` (khớp `git remote origin` đã ghi trong §Trạng thái repo hiện tại của story). [scripts/check-dict-manifest.mjs]
- [x] [Review][Patch] `unescape()` của parser TOML tự viết ÂM THẦM nuốt mọi escape ngoài `\n` (vd. `\t` → `t`, mất luôn dấu `\`) thay vì FAIL — mâu thuẫn với chính lời hứa "ngoài tập con ⇒ FAIL, không đoán" ở đầu tệp. Đã sửa: chỉ chấp nhận `\\`, `\"`, `\n`; escape khác ⇒ `TomlSyntaxError`. [scripts/check-dict-manifest.mjs]
- [x] [Review][Patch] `[base]` (bảng đơn) và `[[base]]` (mảng bảng) không thể phân biệt khi chỉ xuất hiện đúng một lần — cú pháp SAI dạng ngoặc vẫn qua cổng y hệt cú pháp đúng. Đã sửa: gắn nhãn dạng ngoặc (`single`/`array`) theo từng bảng lúc parse, `[base]` bắt buộc dạng `single`; `[[detachable]]` bắt buộc dạng `array`; sai dạng ⇒ FAIL rõ lý do. [scripts/check-dict-manifest.mjs]
- [x] [Review][Patch] Khoá trùng lặp TRONG một bảng bị ghi đè âm thầm (giá trị sau thắng), không lỗi — vd. hai dòng `sha256 = "..."` dưới `[base]` do copy-paste nhầm không bị bắt. Đã sửa: theo dõi khoá đã thấy mỗi bảng, khoá lặp ⇒ `TomlSyntaxError`. [scripts/check-dict-manifest.mjs]
- [x] [Review][Patch] Comment `RS_FILE_FLOOR = 10` ghi "số thật 2026-08-04: 16 tệp .rs" nhưng số thật hôm nay là **18** (cả Blind Hunter lẫn Acceptance Auditor độc lập xác nhận qua `find`) — không ảnh hưởng sàn (18 > 10) nhưng sai sự thật trong chính tệp có vai trò là nguồn số liệu đáng tin. Đã sửa: cập nhật comment thành 18. [scripts/check-dict-build.mjs]
- [x] [Review][Patch] Tệp `.rs` là SYMLINK bị bỏ qua ÂM THẦM (`continue`, không log) trong cả Kiểm A lẫn Kiểm C — một symlink trỏ tới mã hợp nhất thật sẽ không bao giờ bị quét, không dấu vết. Đã sửa: gặp symlink ⇒ `fail()` rõ ràng thay vì bỏ qua êm. [scripts/check-dict-build.mjs]
- [ ] [Review][Defer] Kiểm A escape được qua `#[path = "../outside/file.rs"]`/`include!()` trỏ ra ngoài `tools/dict-build/src/**` — mã hợp nhất đặt ở đó sẽ không bị quét. Sửa đúng cần phân giải thuộc tính/macro Rust thật (ngoài tầm một script quét dòng), không cân xứng với mối đe doạ thực tế (build tool nội bộ, một dev tin cậy dùng). — deferred, sửa đúng cần parser Rust thật, chi phí không cân xứng với rủi ro của một tool nội bộ
- [ ] [Review][Defer] `.entry(` bị né bằng cách chèn khoảng trắng (`foo.entry (hw)`) hoặc xuống dòng giữa lời gọi — so khớp hiện tại theo TỪNG DÒNG, chuỗi con nguyên văn. Sửa đúng cần chuẩn hoá khoảng trắng/token hoá thay vì so khớp dòng-theo-dòng. — deferred, cùng lớp với finding trên (né có chủ ý bởi chính người viết cổng, không phải mối đe doạ thực tế cho tool một-dev)
- [ ] [Review][Defer] Miễn trừ CHỈ nhận dấu gạch ngang dài `—` (U+2014) đúng nghĩa đen; gõ `-`/`--` (ASCII) làm miễn trừ âm thầm KHÔNG khớp, không gợi ý lý do — dòng vẫn báo vi phạm mà không giải thích tại sao "miễn trừ" không được nhận. Sửa: chấp nhận cả `-`/`--`/`—`. — deferred, papercut UX thật nhưng thấp giá trị so với các finding an ninh ở trên, để lượt sau
- [ ] [Review][Defer] Miễn trừ áp theo cặp `(dòng, token)`, không theo TỪNG LẦN XUẤT HIỆN — hai lần dùng CÙNG token cấm trên một dòng chỉ cần một comment miễn trừ. Ca thực tế cực hiếm (đòi hai vi phạm trên đúng một dòng). — deferred, giá trị thấp so với công sửa (cần đếm theo vị trí, không theo dòng)
- [ ] [Review][Defer] Việc bóc comment trong Kiểm A không đối xứng — chỉ nhận dòng `//` thuần, không nhận comment khối `/* ... */` hay comment cuối dòng (`code(); // or_insert`) — cùng lớp lỗ hổng mà `deferred-work.md` đã ghi nhận cho `check-i18n.mjs`/`scope_boundary.rs`, ở đây chưa ghi nhận tương tự. — deferred, cùng tiền lệ chấp nhận được ở các cổng khác trong cây, không riêng cổng này
- [x] [Review][Dismiss] Cả hai script `.mjs` in mã màu ANSI không điều kiện, có thể làm bẩn log CI non-TTY — khớp phong cách sẵn có của các script `check-*.mjs` khác trong cây, không phải vấn đề riêng của diff này.
- [x] [Review][Dismiss] `deferred-work.md` ghi nhận `ARCHITECTURE-SPINE.md` (AD-23) đang lệch khỏi cấu hình thật sau khi gỡ `$RESOURCE/dict/**` — đây là hệ quả ĐÃ ĐƯỢC CHỐT tường minh bởi Quyết định của Ice #2 ("⛔ Không sửa ARCHITECTURE-SPINE.md... Dev ghi vào Completion Notes"), không phải sơ suất của diff này.
- [x] [Review][Dismiss] Tag `dict-v1` trong `dict-manifest.toml` trỏ vào một release CHƯA tồn tại (404 hôm nay) — đúng chủ ý đã ghi rõ trong story (AC3 vế phát hành do Ice làm tay, dev ⛔ không tự tạo release), không phải lỗi.

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

Claude Sonnet 5 (claude-sonnet-5), qua Claude Code — dev-story workflow, 2026-08-04.

### Debug Log References

#### Đường cơ sở (Task 1) — bảy lệnh trên cây làm việc

Chạy 2026-08-04, trên `HEAD=d9bc252` (cây sạch, `git status` không có gì chưa commit — baseline_commit `0ff36a0` trong frontmatter đã lỗi thời so với `HEAD` hiện tại, nhưng cây vẫn sạch nên không ảnh hưởng số liệu dưới đây).

| Lệnh | Kết quả |
|---|---|
| `npm run build` | ✅ xanh — 3 module, 274ms |
| `cargo test --manifest-path src-tauri/Cargo.toml` | ✅ xanh — **62 test** (config_invariants 15 · ipc_contract 5 · scope_boundary 5 · scope_contract 17 · store_boundary 4 · store_contract 16) |
| `check:deps` | ✅ xanh — cây Rust **326 crate**, cây npm **104 gói** |
| `check:tokens` | ✅ xanh — 21 tệp / 116 khai báo CSS |
| `check:i18n` | ✅ xanh — **16 khoá** `vi.json`; quét **27 tệp `.rs` + 5 tệp `.vue`**, miễn trừ 6 (toàn bộ `src-tauri/tests/**`) |
| `check:commands` | ✅ xanh — 5 tệp `.vue` + 14 tệp `.ts`, 4 command, 5 điểm focus |
| `check:scope` + `check:scope:bundled` | ✅ xanh cả hai chiều (dev-no-csp và bundled-csp) |

Số phụ thuộc trong cây Rust mà `check-deps.mjs` đếm được (chứng cứ AC4 — *trước* story): **326 crate**.
`.rs` dưới `src-tauri/src/**`: **26** (khớp §Dev Notes).

Không lệnh nào đỏ — không cần dừng.

#### Bảng dựng dữ liệu (Task 11) — nguồn → dòng đọc · dòng bỏ · bản ghi

Chạy THẬT 2026-08-04 trên năm nguồn tải thật (`./target/release/dict-build --raw raw
--out out/dict-core.db`, bản release — 37 phút 49 giây, chủ yếu I/O đọc 1,1 GB JSON):

| Nguồn | Đọc | Bỏ | `dict_entry` | `dict_sense` | `dict_example` |
|---|---:|---:|---:|---:|---:|
| cvdict | 122.597 | 1 | 122.596 | 200.195 | 0 |
| cc-cedict | 124.758 | 0 | 124.758 | 199.615 | 0 |
| unihan | 49.870 | 0 | 49.870 | 23.285 | 0 |
| viwiktionary | 415.254 | 413.517 | 1.737 | 2.242 | 536 |
| en-wiktionary | 323.840 | 131.681 | 192.159 | 255.372 | 89.939 |
| **Tổng** | | | **491.120** | **680.709** | **90.475** |

`char_idx`: **1.371.273** cặp `(ch, entry_id)`.

**Lý do bỏ, theo nguồn (không phải log — dữ liệu, §Quyết định #8):**
- `cvdict` — 1 dòng: *"gloss field is not wrapped in '/../'"*. Truy ngược: dòng
  67267 của `CVDICT.u8` thật — `澳洲廣播電臺 澳洲广播电台 [[Ao4 zhou1 Guang3 bo1 Dian4
  tai2] ] /Tổng công ty…/`, ngoặc vuông LỒNG NHAU (`[[...] ]`) trong chính dữ liệu
  nguồn. Parser bắt `]` đầu tiên làm ranh giới pinyin, phần còn lại không mở đầu bằng
  `/` ⇒ báo lỗi đúng như thiết kế — không `panic!`, không nuốt im lặng, không đoán mò
  sửa dữ liệu nguồn.
- `viwiktionary` — 413.517 dòng: **411.810** bị lọc *"lang_code != zh (filtered,
  expected)"* (ấn bản `vi` chứa mọi ngôn ngữ, đúng thiết kế phải lọc) + **1.707** *"no
  usable glosses"* (mục từ liên kết nhưng `senses` toàn tag `no-gloss`, ví dụ thật
  `词典`/`Tiếng Trung Quốc` đã thấy lúc khảo sát nguồn). Độ phủ thấp là ĐÚNG — đã cảnh
  báo trước ở Dev Notes (2,76%), ⛔ không đọc thành lỗi.
- `en-wiktionary` — 131.681 dòng *"no usable glosses"* — cùng lớp lý do, một tỷ lệ đáng
  kể mục từ chỉ có `forms`/`sounds` mà chưa có `senses.glosses` (ví dụ: mục hình thái
  biến cách không mang nghĩa riêng).

**Đối chiếu Giai đoạn 0** (604.357 nghĩa · 27.956 ví dụ, BA nguồn) — số hôm nay (NĂM
nguồn): **680.709 nghĩa** (+12,7%) · **90.475 ví dụ** (+223,6%) · `char_idx`
**1.371.273** cặp so với 1.297.115 (ba nguồn, +5,7%). Cả ba số **lớn hơn** — không có
dấu hiệu đọc sót nguồn.

#### Ba phép nghiệm thu trên dữ liệu thật (AC2 · AC5 · AD-26)

Chạy trên `tools/dict-build/out/dict-core.db` THẬT (154.836.992 byte), 2026-08-04,
qua `sqlite3` CLI — SQL nguyên văn để tái lập:

**AC2 — `山` mang ≥ 2 nguồn khác nhau, không hàng nào bị nuốt:**
```sql
SELECT COUNT(DISTINCT ds.source_id) FROM dict_sense ds
  JOIN dict_entry de ON de.id = ds.entry_id WHERE de.headword = '山';
-- kết quả: 4  (cc-cedict, cvdict, en-wiktionary, và một nguồn thứ tư — vượt yêu cầu ≥2)
```
Chi tiết 10 hàng đầu (`headword|source|gloss`): `山|cc-cedict|surname Shan` ·
`山|cc-cedict|mountain; hill (CL:座[zuo4])` · `山|cvdict|họ [Shan1]` ·
`山|cvdict|núi; đồi (lượng từ: 座[zuo4])` · `山|en-wiktionary|mountain; hill
(Classifier: 座 m c; 粒 mn)` … — mỗi nguồn giữ nguyên gloss của chính nó, không hàng
nào mang `sources = 'a,b'`.

**AC5 — hai chiều, TRÊN GLOSS THẬT (không phải chuỗi nạp riêng cho test):**
```sql
-- dict_sense.id = 49797, gloss = 'má' (có thật trong dữ liệu CVDICT/CC-CEDICT)
SELECT sense_fts.rowid FROM sense_fts
  WHERE sense_fts MATCH 'ma' AND sense_fts.rowid = 49797;      -- RỖNG (đúng)
SELECT sense_fts_nd.rowid FROM sense_fts_nd
  WHERE sense_fts_nd MATCH 'ma' AND sense_fts_nd.rowid = 49797; -- 49797 (đúng)
```
Chỉ mục CHÍNH (`remove_diacritics 0`) **từ chối** truy vấn không dấu `ma` cho hàng
`má`; chỉ mục PHỤ (`remove_diacritics 2`) **chấp nhận**. Hai chiều, đúng cả hai — mạnh
hơn phép thử gợi ý của story (nạp chuỗi test riêng) vì chạy thẳng trên dữ liệu sản
phẩm thật.

**AD-26 — trigram trên đầu mục + `char_idx`, dữ liệu (không phải đường tra cứu):**
```sql
SELECT COUNT(*) FROM entry_fts WHERE entry_fts MATCH '中國人';  -- 33 hàng khác rỗng
SELECT COUNT(*) FROM char_idx WHERE ch = '山';                  -- 3.244
SELECT ch, COUNT(*) FROM char_idx WHERE ch IN ('中','國','国')
  GROUP BY ch;   -- 中:3.023 · 國:3.164 · 国:2.032 (CẢ phồn lẫn giản có mặt)
```

#### Bảng kế toán NFR6 (Task 12) — mọi dòng bằng byte

Đo thật 2026-08-04, macOS Intel x86_64 (`uname -m` = `x86_64` — cùng kiến trúc đo của
Story 1.1, số **so được** với `21.285.713` byte font). Lệnh dựng baseline:

```
CI=true npx tauri build --bundles dmg --config src-tauri/tauri.nofonts.conf.json
```

`tauri.nofonts.conf.json` (`{ "bundle": { "resources": null } }`) loại **TOÀN BỘ**
`bundle.resources` — không chỉ font mà cả `license/*.txt` (35.149 byte). Bảng dưới ghi
đúng khoản đó thành dòng riêng thay vì lặng lẽ bỏ sót.

| Dòng | Byte | Nguồn số |
|---|---:|---|
| Baseline `.dmg` hôm nay, KHÔNG font, KHÔNG license (`tauri.nofonts.conf.json`) | 2.334.696 | đo thật, cây nguồn hôm nay — ⛔ không dùng lại 1,40 MB app thăm dò rỗng của Story 1.1 |
| `license/*.txt` (bị loại bởi CÙNG lớp phủ, cộng bù) | 35.149 | `src-tauri/resources/license/COPYING.txt`, đo thật |
| Bộ font | 21.285.713 | đo thật Story 1.1 — `font-spike-results-2026-08-03.md:82` |
| `dict-core.db` sau `VACUUM` | 154.836.992 | đo thật story này (Task 11) |
| Bốn lớp gỡ rời (Thiều Chửu · Cổ hán văn · VietPhrase · HVTĐTD) | `[----]` chưa đo | 🔴 **CHƯA TỒN TẠI — Story 1.10.** Dùng chính build tool của story này nên không thể chạy trước |
| **Tổng payload sản phẩm (4 dòng đo được)** | **178.492.550** | 2.334.696 + 35.149 + 21.285.713 + 154.836.992 |
| WebView2 Runtime nhúng | *(không áp dụng — build macOS, không có `.msi`)* | dòng riêng, KHÔNG cộng vào tổng theo mọi trường hợp; Windows chưa đo ở story này |
| Đối chiếu trần 200.000.000 byte (200 MB thập phân) | **CHƯA KẾT LUẬN ĐƯỢC** | xem §Quyết định của Ice #1 |
| Dư địa còn lại cho Story 1.10 (nếu muốn ĐẠT dưới trần) | 21.507.450 byte (≈ 21,51 MB) | 200.000.000 − 178.492.550 |

**Phán quyết: CHƯA KẾT LUẬN ĐƯỢC** — đúng chữ, không viết "đạt", không viết "nằm
trong trần" (§Quyết định của Ice #1). Bốn lớp gỡ rời của Story 1.10 là phần còn thiếu
duy nhất; Story 1.10 chỉ việc trừ tiếp vào dư địa 21.507.450 byte.

⚠️ **Đọc đúng phạm vi:** tổng 178,49 MB đã **vượt mốc kỳ vọng 150 MB** (không phải
điều kiện đạt, chỉ là mốc kỳ vọng — vẫn dưới trần 200 MB). Dư địa còn lại cho bốn lớp
gỡ rời hẹp hơn nhiều so với con số 47,31 MB mà Story 1.1 tính trước khi có dữ liệu từ
điển thật — Story 1.10 nên đọc kỹ dòng này trước khi ước tính bốn tệp `.db` của mình.

⚠️ **Hai khoản `Cargo.toml` (§Quyết định của Ice #3) KHÔNG bị đụng** — baseline
2.334.696 byte đo TRÊN cấu hình hiện trạng (`reqwest` default features, `crate-type`
đủ bốn), ⛔ chưa phản ánh khoản tiết kiệm nếu Ice quyết định cắt sau. Nếu Story 1.10
đẩy tổng sát trần, đây là hai đòn bẩy đầu tiên nên thử (đã ghi ở `deferred-work.md:23,31`).

#### Nghiệm thu đỏ-rồi-xanh — hai cổng mới

**`check-dict-build.mjs` (Task 7) — 8 đối chứng âm, chạy thật 2026-08-04:**

| # | Kiểm | Đột biến | Kết quả |
|---|---|---|---|
| A1 | A | token `merge` không miễn trừ | ❌ FAIL đúng — bắt được |
| A2 | A | token `.entry(` không miễn trừ | ❌ FAIL đúng — bắt được |
| A3 | A | comment miễn trừ SAI TÊN token (`or_insert` thay vì `.entry(`) | ❌ FAIL đúng — khớp tên chính xác, không khớp mù |
| B1 | B | xoá khối `[workspace]` | ❌ FAIL đúng — bắt được |
| B2 | B | thêm `workspace = true` vào một phụ thuộc | ❌ FAIL đúng — bắt được |
| B3 | B | thêm phụ thuộc `path = "../../src-tauri/shared"` | ❌ FAIL đúng — bắt được |
| C1 | C | cây giả 3 tệp `.rs` (dưới sàn 10) | ❌ FAIL đúng (logic sàn tương đương, không sửa cây thật) |
| C2 | C | cây giả 15 tệp `.rs` (trên sàn) | ✅ OK đúng |

Sau mỗi ca: khôi phục nguyên trạng bằng bản sao lưu, `diff` xác nhận **IDENTICAL**,
chạy lại `npm run check:dict` → xanh, `cargo test` → 49/49 xanh. Hai miễn trừ hợp lệ
đang dùng trong cây thật: `model.rs:93` (`.entry(` + `or_insert` — đếm lý do bỏ TRONG
một nguồn) · `sources/unihan.rs:85` (`.entry(` — tích luỹ thuộc tính một ký tự Unihan
qua nhiều dòng CÙNG nguồn). Cả hai KHÔNG phải hợp nhất xuyên nguồn (AD-19) — chúng
thao tác trên dữ liệu của đúng MỘT nguồn tại một thời điểm.

**`check-dict-manifest.mjs` (Task 8) — 11 ca, chạy thật 2026-08-04 (sha256 dùng
`sha256("test")` thật, không phải chuỗi bịa):**

| # | Nội dung `[base]` đột biến | Kỳ vọng | Kết quả |
|---|---|---|---|
| 1 | Hợp lệ trọn vẹn (dương) | exit 0 | ✅ đạt |
| 2 | Thiếu trường `url` | FAIL | ❌ đúng |
| 3 | `sha256` 43 ký tự | FAIL | ❌ đúng |
| 4 | `sha256` viết HOA | FAIL | ❌ đúng |
| 5 | `url` là `http://` (không `s`) | FAIL | ❌ đúng |
| 6 | Không có `[base]` (chỉ có `[[detachable]]`) | FAIL | ❌ đúng |
| 7 | TOML ngoài tập con — `tags = ["a","b"]` | FAIL, báo đúng dòng/lý do | ❌ đúng |
| 8 | `[[detachable]]` thiếu `name` | FAIL | ❌ đúng |
| 9 | `url` không chứa `/releases/download/dict-v` | FAIL | ❌ đúng |
| 10 | `source_version = ""` | FAIL | ❌ đúng |
| 11 | `[base]` khai LẶP (không phải mảng bảng) | FAIL, báo "khai lại" | ❌ đúng |

Sau mỗi ca: khôi phục bằng bản sao lưu, `diff` xác nhận **IDENTICAL**. Trạng thái
THẬT của `dict-manifest.toml` hôm nay (`[base]` còn comment) chạy cổng ⇒ đúng như kỳ
vọng: FAIL với lý do *"[base] KHÔNG có mặt"* — sẽ chuyển xanh sau Task 13.

*(§Ba phép nghiệm thu trên dữ liệu thật AC2·AC5·AD-26 — xem mục cùng tên ở trên,
ngay sau §Bảng dựng dữ liệu Task 11.)*

#### Sau story (Task 14) — bảy lệnh cũ + hai cổng mới

Chạy lại toàn bộ 2026-08-04, trên cây làm việc sau story (chưa commit):

| Lệnh | Kết quả |
|---|---|
| `npm run build` | ✅ xanh |
| `cargo test --manifest-path src-tauri/Cargo.toml` | ✅ xanh — **62 test**, không đổi so với trước story (không sửa `src-tauri/src/**`) |
| `check:deps` | ✅ xanh — Rust **326 crate** (không đổi — 0 phụ thuộc mới cho `src-tauri`), npm **104 gói** |
| `check:tokens` | ✅ xanh |
| `check:i18n` | ✅ xanh — **27 tệp `.rs` + 5 tệp `.vue`** sau miễn trừ (không đổi so với trước story — `tools/` thêm vào gốc quét, miễn trừ trọn) |
| `check:commands` | ✅ xanh |
| `check:scope` + `check:scope:bundled` | ✅ xanh cả hai — output không đổi so với baseline Task 1 |
| `check:dict` (mới) | ✅ xanh — 18 tệp `.rs`, 3 miễn trừ |
| `check:dict-manifest` (mới) | ✅ xanh — `[base]` đã điền thật |
| `cargo test` (`tools/dict-build`) | ✅ xanh — **49 test** (34 unit + 8 `parse.rs` + 7 `schema.rs`) |

Không lệnh nào đỏ. `.rs` dưới `src-tauri/src/**`: **26** — không đổi so với baseline
Task 1 (đúng khẳng định Dev Notes: *"Không tệp nào dưới `src-tauri/src/**` bị sửa ở
story này"* — chỉ `tauri.conf.json` và `tests/config_invariants.rs` đổi, do Task 10).

### Completion Notes List

1. **AC1 — ĐẠT.** `tools/dict-build` sinh `dict-core.db` theo đúng lược đồ §Quyết định #2 (7 bảng thường + 3 FTS5 ảo). Chạy thật trên năm nguồn: 491.120 `dict_entry` · 680.709 `dict_sense` · 90.475 `dict_example` · 0 `dict_citation` (không nguồn nào trong năm nguồn hôm nay mang trích dẫn có `work`/`author` riêng biệt với ví dụ — bảng tồn tại đúng lược đồ, chỉ chưa có dữ liệu; không nguồn nào của Story 1.9 cung cấp trường này).
2. **AC2 — ĐẠT.** Ba cơ chế đều có: (a) `dict_sense.source_id NOT NULL REFERENCES` + `PRAGMA foreign_keys = ON`, nghiệm thu bằng test `dict_sense_source_id_rejects_null_by_schema`; (b) cổng `check-dict-build.mjs` Kiểm A, 3 miễn trừ hợp lệ đã dùng, 8 đối chứng âm; (c) dữ liệu thật: `山` có **4** `source_id` khác nhau, không hàng nào gộp.
3. **AC3 — ĐẠT vế cơ chế, vế phát hành giao Ice.** `check-dict-manifest.mjs` đọc + phán quyết `dict-manifest.toml`, gắn CI, không đọc `.db`/không tải mạng. `[base]` đã điền SHA-256 thật (`358cf0f8afcc52c210caa205cd1b0b175eb9562de1b0917e48850a629cd8bdb5`) và `source_version` thật. **Release `dict-v1` chưa tồn tại** — lệnh chép-dán cho Ice ở dưới.
4. **AC4 — ĐẠT.** `[workspace]` rỗng, `Cargo.lock` riêng, `check:deps` không đổi (326 crate — 0 phụ thuộc mới cho `src-tauri`), `check-dict-build.mjs` Kiểm B/C xanh.
5. **AC5 — ĐẠT.** Hai chỉ mục FTS5 dựng đủ (`sense_fts` chính, `sense_fts_nd` phụ), nghiệm thu hai chiều trên GLOSS THẬT (`dict_sense.id=49797`, gloss `'má'`) — mạnh hơn phép thử gợi ý (chuỗi nạp riêng) vì chạy trên dữ liệu sản phẩm.
6. **AC6 — CHƯA KẾT LUẬN ĐƯỢC (đúng chữ, hợp lệ theo §Quyết định của Ice #1).** Tổng payload đo được hôm nay: 178.492.550 byte (baseline `.dmg` không font 2.334.696 + license 35.149 + font 21.285.713 + `dict-core.db` 154.836.992). Dư địa còn lại cho bốn lớp gỡ rời của Story 1.10: **21.507.450 byte**. Không subset font, không bỏ nguồn, không bỏ chỉ mục phụ — đúng như cấm.
7. **AD-23 đang LỆCH khỏi cấu hình** — `ARCHITECTURE-SPINE.md` (~dòng 316) còn liệt kê `$RESOURCE/dict/**` trong `assetProtocol.scope` bằng chữ; cấu hình thật (Task 10) đã gỡ nó. Dev không sửa tài liệu quy hoạch (tiền lệ Ice, Story 1.3); ghi lại để Ice sửa khi thuận tiện.
8. **Giới hạn đã biết, ghi thẳng:** `license_text` trong `dict_source` là văn bản giấy phép NGUYÊN VĂN tải thật từ nguồn chính thức (creativecommons.org / unicode.org / gnu.org, 2026-08-04), không phải bản tóm tắt tự viết — nhưng chưa được rà đối chiếu bởi Ice/pháp lý; đó là việc của màn Attribution (Story 10.4), không phải story này. `headword_simp` của hai nguồn Wiktionary luôn `NULL` (quyết định kỹ thuật có ghi trong `wiktextract_common.rs` — độ phức tạp của việc suy luận từ `forms[]` không đáng cho hai nguồn có độ phủ dưới 3%; CVDICT/CC-CEDICT/Unihan đã cho phủ phồn/giản đầy đủ).
9. **Lệnh chép-dán cho Ice** (tạo release + tải `dict-core.db` lên):
   ```bash
   gh release create dict-v1 \
     tools/dict-build/out/dict-core.db \
     --repo vannamhh/AuraTranslate \
     --title "dict-v1 — lớp nền từ điển (CVDICT · CC-CEDICT · Unihan · viwiktionary · en.wiktionary)" \
     --notes "SHA-256: 358cf0f8afcc52c210caa205cd1b0b175eb9562de1b0917e48850a629cd8bdb5"
   ```
   Sau khi chạy, `dict-manifest.toml` KHÔNG cần sửa gì thêm — URL đã đúng dạng và đúng tag từ Task 13.
10. **Không tệp nào dưới `src-tauri/src/**` bị sửa** — đúng ranh giới phạm vi đã khai. Chỉ `src-tauri/tauri.conf.json` và `src-tauri/tests/config_invariants.rs` đổi (Task 10, Ice phê chuẩn).

### File List

**Mới:**
- `tools/dict-build/Cargo.toml`, `Cargo.lock`
- `tools/dict-build/src/{lib,main,build,schema,model,insert,char_idx,finalize,licenses,sources_meta}.rs`
- `tools/dict-build/src/sources/{mod,cvdict,cc_cedict,cedict_common,unihan,viwiktionary,en_wiktionary,wiktextract_common}.rs`
- `tools/dict-build/assets/licenses/{CC-BY-SA-4.0,Unicode-License-v3,GFDL-1.3}.txt`
- `tools/dict-build/tests/{parse,schema}.rs`
- `tools/dict-build/tests/fixtures/raw/{cvdict,cc_cedict,unihan,viwiktionary,en_wiktionary}/**` (fixture thật, trích từ dữ liệu đã tải)
- `scripts/check-dict-build.mjs`
- `scripts/check-dict-manifest.mjs`

**Sửa:**
- `tools/dict-build/README.md` — hình dạng đã chốt
- `package.json` — `check:dict`, `check:dict-manifest`
- `.github/workflows/ci.yml` — hai bước mới trong job `check`
- `scripts/check-i18n.mjs` — gốc quét `tools/**`, miễn trừ trọn, doc-comment cập nhật
- `src-tauri/tauri.conf.json` — `assetProtocol.scope` còn một mục
- `src-tauri/tests/config_invariants.rs` — đổi tên + sửa assertion scope
- `src-tauri/resources/dict/README.md` — trạng thái tệp + scope
- `dict-manifest.toml` — `[base]` điền thật
- `.gitignore` — `tools/dict-build/{raw,out,work,target}/`
- `_bmad-output/implementation-artifacts/deferred-work.md` — đóng `:44` `:79` `:21` `:57`, cập nhật `[D4]`, thêm mục Story 10.1
- `_bmad-output/implementation-artifacts/sprint-status.yaml` — trạng thái story

**Không vào git (đã ignore, đúng chủ ý):** `tools/dict-build/raw/**` (~1,4 GB dữ liệu thô đã tải) · `tools/dict-build/out/dict-core.db` (154.836.992 byte — artifact phát hành qua GitHub Release, không qua git) · `tools/dict-build/target/`.

## Change Log

| Ngày | Thay đổi |
|---|---|
| 2026-08-04 | Story tạo — phân tích context đầy đủ, trạng thái `ready-for-dev` |
| 2026-08-04 | Ice phán quyết bốn câu hỏi. #1 #3 #4 theo mặc định; **#2 đổi phạm vi** — đồng ý gỡ `$RESOURCE/dict/**` khỏi `assetProtocol.scope` ⇒ thêm **Task 10**, dồn số các task sau, §Câu hỏi cho Ice đổi thành §Quyết định của Ice |
| 2026-08-04 | Dev hoàn tất Task 1–14: dựng `tools/dict-build`, chạy thật trên năm nguồn (154.836.992 byte `dict-core.db`), hai cổng CI mới, gỡ `$RESOURCE/dict/**` khỏi scope, kế toán NFR6 (**CHƯA KẾT LUẬN ĐƯỢC**, dư địa 21.507.450 byte), điền `dict-manifest.toml`. 111 test mới xanh (49 `tools/dict-build` + 15 `config_invariants` không đổi). Trạng thái → `review`. |
| 2026-08-04 | Code review (`bmad-code-review`, Blind Hunter + Edge Case Hunter + Acceptance Auditor), chia 3 nhóm vì diff > 3000 dòng. Tổng **32 patch** áp dụng (bao gồm sửa một fixture bịa dữ liệu ở `en_wiktionary`, một lỗ hổng ghim URL trong `check-dict-manifest.mjs`, nhiều lỗ hổng bypass ở cổng `check-dict-build.mjs`, và một lỗi CI đỏ tự phát sinh giữa lượt review), 1 decision Ice đã chốt (gộp Wiktionary theo headword trong-nguồn), 6 defer, 6 dismiss. `cargo test`: 62/62 xanh. Cả ba cổng CI (`check:dict`, `check:dict-manifest`, `check:i18n`) xanh trên cây nguồn cuối. Trạng thái → `done`. |
