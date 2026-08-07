---
baseline_commit: 5a68df7
baseline_note: 'Cây làm việc SẠCH tại 5a68df7 (đã `git status` 2026-08-05). Mọi con số ở §Trạng thái repo hiện tại đo trên đúng commit này.'
---

# Story 1.11: Ba nhánh truy vấn tiếng Trung

Status: done

<!-- Note: Validation is optional. Run validate-create-story for quality check before dev-story. -->

**Covers:** FR39 · NFR1 · AD-26 · AD-19 *(hình dạng kết quả)* · AD-11 *(đường mở tệp)*
**Epic:** 1 — Nền móng ứng dụng & Tra cứu ngoại tuyến tức thì
**Story trước:** [1.10b — Dựng dữ liệu từ điển tiếng Anh](1-10b-dung-du-lieu-tu-dien-tieng-anh.md) *(done)*

---

> 🔴 **Đây là dòng mã ĐỌC từ điển ĐẦU TIÊN của dự án.** Chín story trước đã dựng **dữ liệu** (`tools/dict-build` → ba tệp `.db`); `src-tauri/src/core/dict/mod.rs` hôm nay là **7 dòng doc-comment, 0 dòng mã**. Story này viết tầng truy vấn.
>
> 🔴 **Story này KHÔNG dựng cổng `DictionarySource` (1.13), KHÔNG gom kết quả nhiều tệp (1.13), KHÔNG dựng IPC command (1.13/1.17), KHÔNG chạm frontend.** Nó giao **ba nhánh truy vấn trên MỘT tệp `.db`** — thứ mà 1.13 sẽ bọc lại thành adapter cho từng tệp.
>
> 🔴 **Hai vật cản cấu trúc phải giải TRƯỚC khi gõ dòng SQL đầu tiên** — cả hai đã có lời giải chốt sẵn ở §Quyết định #1 và #2, đừng tự phát minh lại:
> 1. `tests/store_boundary.rs` **cấm** chuỗi `rusqlite` và `Connection::open` ở mọi tệp ngoài `src/core/store/**`. `core/dict/` nằm ngoài.
> 2. Ba tệp `.db` thật **không nằm trong git** (`.gitignore: *.db`) và tệp nền nặng **194.998.272 byte**. CI không có tệp nào để tra.

---

## Story

As a người dịch,
I want tra một chữ Hán đơn hay một từ hai chữ đều ra kết quả,
So that công cụ không im lặng trả về rỗng ở đúng những từ tôi tra nhiều nhất.

---

## Acceptance Criteria

### AC1 — Ba nhánh, chọn nhánh bằng **số KÝ TỰ**, không bằng số byte

**Given** một truy vấn
**When** tầng tra cứu chọn đường đi
**Then** đúng ba nhánh tồn tại, và bảng này là hợp đồng:

| Chế độ | Độ dài *(ký tự)* | Nhánh | Chỉ mục dùng |
|---|---|---|---|
| **Tra chính xác đầu mục** | bất kỳ | 1 | B-tree `idx_entry_headword` + `idx_entry_headword_simp` |
| **Chuỗi con** | **1–2** | 2 | bảng đảo ngược `char_idx` |
| **Chuỗi con** | **≥ 3** | 3 | FTS5 `entry_fts` (`trigram`) |

**And** phép đo độ dài là `query.chars().count()`, 🔴 **không** `query.len()`
**And** nhánh đã chọn **quan sát được từ ngoài** *(giá trị trả về, không phải một dòng log)* — nếu không, AC này không nghiệm thu được và AC2 cũng không

> 🔴 **Đây là bẫy đắt nhất của cả story, và nó cho một lượt CI XANH.** `"山".len()` là **3** (UTF-8), `"中國".len()` là **6**. Chọn nhánh theo `len()` ⇒ mọi truy vấn tiếng Trung 1–2 ký tự rơi vào nhánh 3 ⇒ `entry_fts` trả **0** hàng trong 0,01 ms, không lỗi nào được ném — **đúng nguyên văn lớp lỗi mà FR39 và AD-26 tồn tại để chặn**, và đúng thứ mũi thăm dò Giai đoạn 0 đã đo. Xem §Bẫy 1.

### AC2 — Ba truy vấn mốc trả kết quả KHÁC RỖNG, đối chiếu số ĐO THẬT

**Given** `dict-core.db` thật *(194.998.272 byte, sha `2145c7ae…`)*
**When** tra ba truy vấn của epic
**Then** cả ba khác rỗng, và số hàng khớp bảng dưới đây:

| Truy vấn | Chế độ | Nhánh | Số ĐO THẬT *(2026-08-05, `sqlite3` trên `out/dict-core.db`)* |
|---|---|---|---:|
| `山` | chính xác | 1 | **6** |
| `山` | chuỗi con | 2 | **3.177** |
| `中國` | chính xác | 1 | **4** |
| `中國` | chuỗi con | 2 | **350** *(ứng viên 390, xem AC4)* |
| `中國人` | chuỗi con | 3 | **33** |

**And** 🔴 **đối chứng âm bắt buộc**, ghi nguyên văn vào §Debug Log References: `entry_fts MATCH '"山"'` ⇒ **0** hàng và `entry_fts MATCH '"中國"'` ⇒ **0** hàng
**And** đối chứng âm đó là **bằng chứng dương** rằng nhánh 2 phải tồn tại — không phải một sự cố

> ⚠️ **Số của Giai đoạn 0 KHÔNG phải số nghiệm thu.** `phase-0-spike-results-2026-08-02.md` đo `山` ra **2.576** bằng `LIKE` trên một database dựng từ **ba** nguồn; `dict-core.db` hôm nay có **sáu** nguồn. Epic viết *"đối chiếu được với số đo Giai đoạn 0"* — *đối chiếu được* nghĩa là **cùng bậc độ lớn và cùng dấu** *(khác rỗng, hàng nghìn)*, không phải bằng nhau. Bảng trên là số nghiệm thu; số Giai đoạn 0 là bối cảnh.

### AC3 — Mọi nhánh lọc `dict_entry.lang = 'zh'`

**Given** `dict-core.db` nay có **119.039** hàng `lang = 'en'` *(20,1% của 592.538 đầu mục — Story 1.10b)*
**When** bất kỳ nhánh nào chạy
**Then** mệnh đề `lang = 'zh'` có mặt, không giả định mọi hàng là `zh`

**And** 🔴 **đối chứng âm bắt buộc — và nó CHỈ đo được bằng truy vấn LATIN:**

| Truy vấn | Nhánh | Không lọc `lang` | Có lọc `lang='zh'` |
|---|---|---:|---:|
| `lock` *(chính xác)* | 1 | **1** | **0** |
| `dic` *(chuỗi con)* | 3 | **572** *(100% `lang='en'`)* | **0** |

> ⚠️ **Đính chính một mệnh đề trong `deferred-work.md:279`, ĐO ĐƯỢC chứ không suy luận.** Mục đó viết *"tra một chữ Hán sẽ nhận về `dictionary`, `lock`, `API`, `Wikipedia`"*. **Sai với truy vấn thuần Hán:** đã đo — `entry_fts MATCH '"中國人"'` giao với `lang='en'` cho **0** hàng, và `char_idx` chỉ chứa **9** cặp thuộc lớp tiếng Anh trên tổng **1.341.179**. Trigram Latin không khớp trigram Hán.
> **Nhưng mệnh lệnh thì KHÔNG đổi**, vì hai lý do thật: (a) một truy vấn **Latin** rơi vào đường này *(người dùng bôi đen chữ Latin trong văn bản tiếng Trung — chuyện thường)* trả về mục tiếng Anh **dán nhãn kết quả tiếng Trung**; (b) 1.11b và 1.13 dựng trên đúng hợp đồng này. Ghi đính chính vào §Completion Notes cho Ice.

### AC4 — Chuỗi con phải được **XÁC MINH LẠI**, và xác minh chạy ở **Rust**

**Given** `char_idx` là bảng `(ký tự, entry_id)` — nó trả lời *"đầu mục có chứa cả hai ký tự"*, **không** trả lời *"có chứa hai ký tự ĐÓ LIỀN NHAU"*
**When** nhánh 2 chạy với truy vấn ≥ 2 ký tự
**Then** tập ứng viên từ `INTERSECT` được lọc lại bằng phép kiểm chuỗi con **trong Rust** *(`str::contains` trên `headword` và `headword_simp`)*
**And** phép lọc áp cho **cả** `headword` **lẫn** `headword_simp` — không bỏ vế `headword_simp` làm `国` trả rỗng, đúng Bẫy 8 của Story 1.9

*Đạt nghĩa là* số đo này tái lập được:

| | `中國` |
|---|---:|
| Ứng viên `char_idx` INTERSECT *(có lọc `lang='zh'`)* | **390** |
| Sau khi xác minh chuỗi con | **350** |
| ⇒ dương tính giả bị loại | **40** |

**And** nhánh 3 chạy **cùng** phép xác minh, và số đo ghi lại: `中國人` ⇒ **33 ứng viên → 33 sau xác minh, 0 dương tính giả**. 🔴 Kết quả 0 chênh lệch là một **phép đo**, không phải cái cớ để bỏ bước xác minh — bỏ nó là để một hành vi không được kiểm chứng của FTS5 quyết định đúng/sai của FR39.
**And** truy vấn chuỗi con **1 ký tự** không cần xác minh *(một ký tự có mặt trong `char_idx` ⇔ nó là chuỗi con)* — ghi mệnh đề này ra thay vì để nó ngầm

### AC5 — Không một câu `LIKE` nào trên đường tra cứu nóng

**Given** toàn bộ mã của `src-tauri/src/core/dict/**`
**When** rà bằng máy
**Then** không token `LIKE` nào, không `GLOB`, không `instr(`
**And** phép rà là một **test**, không phải một lượt đọc bằng mắt

> Số của Giai đoạn 0: `LIKE` 1 ký tự **20,09 ms** · 2 ký tự **50,14 ms**, so với `char_idx` **0,15 / 4,49 ms** — nhanh hơn **134×** và **11×**. `LIKE` bị **bảng Stack liệt kê đích danh** vào danh sách *"Không dùng, đã loại có lý do"*.
> ⚠️ `instr(` cũng bị cấm ở mã sản phẩm vì cùng lý do *(quét toàn bảng)*, dù nó **được phép** trong SQL nghiệm thu chạy tay ở `sqlite3` — bảng AC4 ở trên chính là dùng nó.

### AC6 — Kết quả mang `source` theo **`code`** (chuỗi), không theo `id` (số)

**Given** mỗi tệp `.db` có bảng `dict_source` **RIÊNG**, nên `id = 1` tồn tại ở **cả ba** tệp và trỏ ba nguồn khác nhau *(`viwiktionary` · `thieu-chuu` · `vietphrase`)*
**When** một hàng kết quả được dựng
**Then** nó mang `source_code: String` *(vd. `"cvdict"`)*, 🔴 **không** `source_id: i64`
**And** **không** tồn tại bước hợp nhất nghĩa giữa các nguồn ở bất kỳ đâu trong story này *(AD-19)*

> 🔴 Đây là mục `deferred-work.md:260` giao **đích danh cho Story 1.11/1.13**. Khoá theo `id` sẽ dán nhãn *"Thiều Chửu"* cho một nghĩa thật ra từ CVDICT ngay khi 1.13 gom nhiều tệp — **FR31 vỡ theo cách thầm lặng nhất có thể**, và nó vỡ ở story SAU chứ không ở story này, tức đắt gấp đôi để lần ra.

### AC7 — Tệp từ điển mở **CHỈ ĐỌC**, và không một byte nào bị ghi

**Given** AD-7 — dữ liệu từ điển là **chỉ đọc, luôn luôn**
**When** một tệp `.db` từ điển được mở
**Then** cờ mở là `SQLITE_OPEN_READ_ONLY | SQLITE_OPEN_NO_MUTEX` — **không** `CREATE`, **không** `READ_WRITE`, **không** `URI`
**And** `PRAGMA query_only = 1` được **đặt rồi ĐỌC LẠI để xác nhận** *(khuôn `pragmas::set_and_verify`)*
**And** 🔴 **KHÔNG** đặt `journal_mode = WAL` trên tệp từ điển

*Đạt nghĩa là* ba mệnh đề nghiệm thu bằng test, không bằng lý lẽ:

1. Tệp mở xong ⇒ **SHA-256 của tệp không đổi** *(băm trước, mở, tra, đóng, băm lại)*
2. **Không** tệp `-wal` / `-shm` nào xuất hiện cạnh nó
3. Mở một đường dẫn **không tồn tại** ⇒ trả `Err`, **không** tạo ra một tệp rỗng

> 🔴 **Vì sao vế `journal_mode = WAL` là mệnh lệnh chứ không phải sở thích.** `tools/dict-build/src/finalize.rs` cố ý đặt `journal_mode = DELETE` và gọi đó là *"Bẫy 1, bẫy đắt nhất của story"*. `pragmas::apply_reader_pragmas` **hiện có** thì `verify_wal` — chạy nó trên tệp từ điển là **đỏ ngay** *(`StoreError::WalUnavailable { mode: "delete" }`)*. Và nếu ai đó "sửa" bằng cách đặt WAL: `PRAGMA journal_mode = WAL` **GHI VÀO** database ⇒ **SHA-256 đổi** ⇒ `dict-manifest.toml` thành sai ⇒ AD-25 vỡ. Trên một `$RESOURCE` chỉ-đọc thật thì nó chỉ trượt.
> Đây là lý do **không** tái dùng thẳng `pragmas::apply_reader_pragmas` — xem §Quyết định #1.

### AC8 — Ranh giới `rusqlite` giữ nguyên: `core/dict/` không gõ tên crate SQLite

**Given** `tests/store_boundary.rs::only_core_store_may_name_rusqlite`
**When** chạy `cargo test` sau story
**Then** nó **XANH mà không nới `STORE_DIR` và không nới `FORBIDDEN`**
**And** `core/dict/**` viết truy vấn qua các kiểu **tái xuất** của `core::store` — `ReadHandle`, `SqlResult`, `Row`, `SqlError` — không `use rusqlite::…`, không `Connection::open`

> **Nới cổng là đường SAI, và story chốt sẵn đường đúng** *(§Quyết định #1)*. Cổng này canh AD-11 (*"không module nào được tự mở kết nối ghi"*); tệp từ điển **chỉ đọc** nên AD-11 không áp — nhưng cổng không phân biệt được điều đó, và một cổng có hai miễn trừ là một cổng sẽ có ba. Lời giải: **đường mở tệp sống trong `core/store/`**, `core/dict/` chỉ nhận một `ReadHandle`.

### AC9 — NFR1: p95 backend đo THẬT trên `dict-core.db`, không suy ra

**Given** ba nhánh
**When** đo p95 phía backend trên **tệp thật** *(không phải fixture)*, ≥ 200 lượt mỗi nhánh, bỏ 10 lượt làm nóng
**Then** một bảng trong §Debug Log References, mỗi nhánh một hàng, và phán quyết **ĐẠT/VƯỢT** theo trần dưới đây:

| Nhánh | AD-26 công bố | Trần story *(= 2× AD-26, làm tròn lên)* | Đo được |
|---|---|---:|---|
| 1 — B-tree chính xác | 0,02 ms | **1 ms** | *(điền)* |
| 2 — `char_idx` 1–2 ký tự | 0,15–4,5 ms | **10 ms** | *(điền)* |
| 3 — FTS5 trigram 3+ | 0,13–0,19 ms | **1 ms** | *(điền)* |
| **Nhánh chậm nhất** | — | 🔴 **≤ 10 ms** | *(điền)* |

**And** trần 10 ms là **dẫn xuất, không phải phát minh**: NFR1 cho **100 ms đầu-cuối**, và PRD ghi *"toàn bộ phần còn lại (~99,95 ms) dành cho vòng IPC Tauri và render frontend"* — 10 ms giữ lại **≥ 90 ms** cho hai thứ chưa ai đo *(giả định `[A1]`)*
**And** 🔴 **phép đo này chạy TAY và ghi vào story, KHÔNG thành một test trong CI** — CI không có tệp `.db` nào, và một ngưỡng thời gian trong CI là một test flaky sẽ bị gỡ trong tháng *(cùng tiền lệ `unmeasured` của Story 1.3 và §Testing standards)*
**And** VƯỢT trần ⇒ ghi số, nêu nhánh, rồi **DỪNG và báo Ice**. Không tự thêm chỉ mục, không tự đổi lược đồ *(`schema.rs` của `tools/dict-build` — §Bẫy 8 của Story 1.10b vẫn nguyên hiệu lực)*

---

## Tasks / Subtasks

- [x] **Task 1 — Đường cơ sở: chạy, ghi số, không sửa gì** (không AC)
  - [x] `git status` — xác nhận cây sạch tại `5a68df7`
  - [x] `npm run build` *(bắt buộc TRƯỚC `cargo test` của `src-tauri` — `generate_context!` nhúng `dist/` lúc biên dịch)*
  - [x] `cargo test --locked --manifest-path src-tauri/Cargo.toml` ⇒ ghi tổng số test *(bản ghi: **62**)*
  - [x] `npm run check:deps` · `check:i18n` · `check:commands` · `check:tokens` · `check:dict` · `check:dict-manifest`
  - [x] Ghi số tệp `.rs` dưới `src-tauri/src/**` *(bản ghi: **26**)*
  - [x] `ls -la tools/dict-build/out/` ⇒ xác nhận **ba** tệp `.db` có mặt và kích thước khớp §Trạng thái repo. Thiếu tệp ⇒ **DỪNG và báo** *(không tự dựng lại — một lượt `cargo run` sai cờ làm ba checksum trong `dict-manifest.toml` thành sai, §Bẫy 5 của Story 1.10b)*
  - [x] không Một lệnh đỏ sẵn ⇒ **DỪNG và báo**

- [x] **Task 2 — Đường mở tệp CHỈ ĐỌC, sống trong `core/store/`** (AC7, AC8) 🔴 **hạt nhân cấu trúc**
  - [x] `core/store/mod.rs`: thêm biến thể `StoreKind::Dict` với `as_str() = "dict"`. Doc-comment nêu đích danh AD-7 *(dữ liệu từ điển — **chỉ đọc, luôn luôn**)* và nói rõ nó không có `StoreSpec`, không có writer, không có checkpoint, không có di trú.
  - [x] `core/store/pragmas.rs`:
    - [x] `open_readonly_connection(path, kind)` — cờ `SQLITE_OPEN_READ_ONLY | SQLITE_OPEN_NO_MUTEX`. Doc-comment chép **nguyên lý lẽ** của `open_connection` về `SQLITE_OPEN_URI` *(một thư mục chứa `?` trong tên)*, cộng lý do mới: không `CREATE` ⇒ đường dẫn sai trả `Err` thay vì sinh một tệp rỗng mà mọi truy vấn sau đó trả rỗng **không lỗi**.
    - [x] `apply_dict_reader_pragmas(conn, kind, tuning)` — **chỉ** `query_only = 1` *(đặt rồi đọc lại)* + `busy_timeout`. 🔴 **KHÔNG** `verify_wal`, **KHÔNG** `wal_autocheckpoint`. Doc-comment nêu đích danh AC7 và `finalize.rs::set_journal_mode_delete`.
  - [x] `core/store/reader.rs`: bóc phần thân dùng chung của `ReaderPool::open` và thêm `ReaderPool::open_readonly(path, kind, tuning)` — cùng pool `Mutex` + `Condvar` + `Lease`, khác **đúng hai hàm** *(opener và bộ pragma)*. Không sao chép cả tệp; không thêm crate pool nào *(doc-comment `reader.rs` đã cấm `r2d2`/`deadpool`/`bb8`)*.
  - [x] `core/store/readonly.rs` **(tệp mới)** — `pub struct ReadOnlyDb`:
    - [x] `open(path: PathBuf, kind: StoreKind) -> Result<ReadOnlyDb, StoreError>` · `read<T,F>(&self, job: F)` *(chữ ký y hệt `Store::read`)* · `close(&self)` · `path(&self)`
    - [x] `Drop` gọi `close()` — trên Windows một tệp còn mở là một `remove_dir_all` thất bại (NFR14), đúng bài học của `Store`
    - [x] Dùng `Tuning::default()` nhưng **chỉ đọc** `pool_size` và `busy_timeout`; doc-comment nói thẳng bốn trường kia không áp cho tệp chỉ đọc
    - [x] **Không** `use tauri::…` *(`core_store_does_not_depend_on_tauri` quét cả `core/store/**`)*
  - [x] `core/store/mod.rs`: `pub mod readonly;` + `pub use readonly::ReadOnlyDb;`
  - [x] 🔴 Cập nhật `tests/store_contract.rs:1132-1134` — thêm `assert_eq!(StoreKind::Dict.as_str(), "dict")`. Không xoá ba dòng đang có.
  - [x] **Mọi chuỗi trong `core/store/**` viết KHÔNG DẤU** — thư mục này không nằm trong `EXEMPT` của `check-i18n.mjs`

- [x] **Task 3 — `core/dict/` — hình dạng công khai** (AC1, AC6, AC8)
  - [x] `core/dict/mod.rs` — thay 7 dòng doc-comment hiện có bằng module thật. Giữ nguyên tinh thần doc-comment cũ *(AD-19, AD-10, AD-25)*, cộng bảng ba nhánh của AC1.
  - [x] Ba kiểu công khai, không hơn:
    - [x] `pub enum LookupMode { Exact, Substring }` — chế độ do **chỗ gọi** quyết, không đoán từ nội dung truy vấn
    - [x] `pub enum QueryBranch { ExactBtree, CharIdx, FtsTrigram }` — 🔴 **trả về cùng kết quả**, vì AC1 đòi nhánh đã chọn *quan sát được từ ngoài*. Không dùng `eprintln!` để "quan sát".
    - [x] `pub struct EntryHit { pub entry_id: i64, pub source_code: String, pub lang: String, pub headword: String, pub headword_simp: Option<String> }` — 🔴 `source_code`, **không** `source_id` (AC6)
    - [x] `pub struct LookupResult { pub branch: QueryBranch, pub hits: Vec<EntryHit> }`
  - [x] `pub fn lookup(db: ReadHandle<'_>, query: &str, mode: LookupMode) -> SqlResult<LookupResult>` — nhận **`ReadHandle`**, không nhận `ReadOnlyDb` *(chỗ gọi mở kho; hàm này thuần theo kết nối, và đó là điều kiện để 1.13 gọi nó một lần cho mỗi tệp)*
  - [x] `pub fn pick_branch(query: &str, mode: LookupMode) -> QueryBranch` — 🔴 **hàm riêng, `pub`, thuần**. Nó là chỗ AC1 nghiệm thu được không cần một tệp `.db` nào.
  - [x] **Không** `use rusqlite`. **Không** `Connection::open`. Chỉ `crate::core::store::{ReadHandle, SqlResult, Row}` (AC8).
  - [x] **Không** `#[tauri::command]`, không đụng `commands/`, không đụng `lib.rs::invoke_handler`, không đụng `ports/mod.rs`

- [x] **Task 4 — Nhánh 1: tra chính xác qua B-tree** (AC1, AC2, AC3, AC6)
  - [x] SQL hằng, tham số ràng buộc *(không `format!` chuỗi truy vấn vào SQL)*:
        `WHERE (e.headword = ?1 OR e.headword_simp = ?1) AND e.lang = 'zh'`
  - [x] `JOIN dict_source s ON s.id = e.source_id` để lấy `s.code` → `EntryHit.source_code` (AC6)
  - [x] ⚠️ `idx_entry_headword_simp` **tồn tại** *(`schema.rs:111`)* nên vế `OR` vẫn đi qua chỉ mục — nhưng SQLite chỉ dùng được hai chỉ mục cho một `OR` khi kế hoạch là `MULTI-INDEX OR`. 🔴 Chạy `EXPLAIN QUERY PLAN` và **dán nguyên văn** vào §Debug Log References. Thấy `SCAN dict_entry` ⇒ tách thành hai truy vấn `UNION`, không để nguyên rồi ghi *"chắc là ổn"*.
  - [x] Không `LIKE`, không `instr(`

- [x] **Task 5 — Nhánh 2: `char_idx` + xác minh ở Rust** (AC1, AC2, AC3, AC4)
  - [x] 1 ký tự ⇒ `SELECT … WHERE e.id IN (SELECT entry_id FROM char_idx WHERE ch = ?1) AND e.lang='zh'`. Không xác minh *(AC4 mệnh đề cuối)*.
  - [x] 2 ký tự ⇒ `INTERSECT` **hai** tập `char_idx`. 🔴 **Không** `ch IN ('中','國')` — đó là **hợp** chứ không phải **giao**, và nó cho ra một tập rộng hơn hẳn mà mọi test "khác rỗng" vẫn xanh.
  - [x] 🔴 Xác minh chuỗi con **trong Rust**: giữ hàng khi `hit.headword.contains(query)` **hoặc** `hit.headword_simp.as_deref().is_some_and(|s| s.contains(query))`. Không `instr()` trong SQL (AC5).
  - [x] Ghi vào §Debug Log References **cả hai** số: ứng viên và sau xác minh *(kỳ vọng `中國`: 390 → 350)*.
  - [x] ⚠️ Ghi ra một giới hạn đã biết thay vì để nó ngầm: truy vấn 2 ký tự mà **một ký tự không phải chữ Hán** *(vd. `A山`)* cho tập ứng viên **rỗng** — `char_idx` chỉ chứa ký tự khớp `char_idx::is_han`. Đó là hành vi **đúng** cho một đường tra cứu **tiếng Trung**; ghi vào doc-comment, đừng "sửa" bằng một nhánh thứ tư.

- [x] **Task 6 — Nhánh 3: FTS5 trigram + xác minh** (AC1, AC2, AC3, AC4)
  - [x] `entry_fts` là FTS5 **external-content** trên `dict_entry` với `content_rowid='id'` ⇒ nối qua `f.rowid = e.id`.
  - [x] Truy vấn dạng **cụm có ngoặc kép**: `entry_fts MATCH ?1` với tham số là `format!("\"{}\"", escaped)`. 🔴 Ngoặc kép là bắt buộc: không ngoặc, chuỗi đi vào **cú pháp truy vấn FTS5** và một ký tự như `*` `-` `^` `(` `:` làm SQLite trả `SQLITE_ERROR` — tức một truy vấn của người dùng làm tra cứu **báo lỗi** thay vì trả rỗng.
  - [x] 🔴 Thoát dấu `"` trong truy vấn bằng cách nhân đôi (`"` → `""`) **trước** khi bọc — không bỏ qua, không xoá ký tự.
  - [x] Xác minh chuỗi con ở Rust, **cùng hàm** với nhánh 2 *(không viết hai bản)*. Ghi số: kỳ vọng `中國人` **33 → 33**.
  - [x] `EXPLAIN QUERY PLAN` dán vào §Debug Log References — phải thấy `VIRTUAL TABLE INDEX`, không thấy `SCAN`.

- [x] **Task 7 — Fixture `.db` cho CI, có cổng chống trôi lược đồ** (AC1–AC6)
  - [x] 🔴 **Quyết định đã chốt** *(§Quyết định #3)*: fixture dựng **trong test**, DDL **chép** từ `tools/dict-build/src/schema.rs`, cộng **một test parity** đọc tệp đó dưới dạng văn bản.
  - [x] `src-tauri/tests/dict_lookup.rs` **(tệp mới)** — helper `build_fixture(dir) -> PathBuf`:
    - [x] Chép nguyên văn bảy hằng DDL cần dùng: `DICT_META_DDL` · `DICT_SOURCE_DDL` · `DICT_ENTRY_DDL` · `DICT_SENSE_DDL` · `CHAR_IDX_DDL` · `ENTRY_INDEXES_DDL` · `ENTRY_FTS_DDL`
    - [x] Nạp dữ liệu **nhỏ mà đủ ba nhánh và đủ đối chứng âm**: `山` · `中國` · `中國人` · `國中` *(dương tính giả của nhánh 2 — đảo thứ tự hai ký tự)* · `國`/`国` *(cặp phồn/giản, khoá vế `headword_simp`)* · **≥ 2 hàng `lang='en'`** với headword Latin *(AC3, vd. `lock` · `dictionary`)* · **≥ 2 nguồn** `dict_source` khác `code` *(AC6)*
    - [x] Sinh `char_idx` cho từng đầu mục `zh` — chép luật của `char_idx::insert_for_entry`: phủ **cả** `headword` **lẫn** `headword_simp`
    - [x] `INSERT INTO entry_fts(entry_fts) VALUES('rebuild')` sau khi nạp *(external-content ⇒ không tự đầy)*
    - [x] Đóng kết nối dựng fixture **trước** khi `ReadOnlyDb::open` chạm vào nó
    - [x] ⚠️ Fixture **không** đặt `journal_mode`; mặc định `delete` — **giống tệp thật** *(đã đo: `PRAGMA journal_mode` của cả ba tệp = `delete`)*. Đặt WAL ở đây làm ca AC7 mất ý nghĩa.
  - [x] 🔴 `fixture_ddl_is_verbatim_from_dict_build_schema` — đọc `<CARGO_MANIFEST_DIR>/../tools/dict-build/src/schema.rs` thành `String` và khẳng định **từng** khối DDL đã chép có mặt **nguyên văn**. Thất bại ⇒ lược đồ hai cây đã trôi khỏi nhau, và mọi ca dưới đây đang kiểm một database không tồn tại trong sản phẩm.
  - [x] ⚠️ `src-tauri/tests/**` được **miễn trừ** khỏi `store_boundary.rs` *(doc-comment `:27-31`)*, nên tệp test này **được phép** gõ `rusqlite` để dựng fixture. Mã sản phẩm thì không.

- [x] **Task 8 — Test hành vi** (AC1–AC8)
  - [x] `dict_lookup.rs`:
    - [x] `branch_is_picked_by_char_count_not_byte_length` — 🔴 **ca đắt nhất**: `pick_branch("山", Substring) == CharIdx` *(không `FtsTrigram`)*; `pick_branch("中國", Substring) == CharIdx`; `pick_branch("中國人", Substring) == FtsTrigram`; `pick_branch("ab", Substring) == CharIdx`. Doc-comment ghi `"山".len() == 3`.
    - [x] `exact_mode_always_takes_the_btree_branch` — kể cả truy vấn 1 ký tự và ≥ 3 ký tự
    - [x] `a_one_character_query_returns_rows` / `a_two_character_query_returns_rows` / `a_three_character_query_returns_rows` — **AC2** trên fixture
    - [x] 🔴 `fts_returns_nothing_for_one_and_two_character_queries` — **đối chứng âm của AC2**, chạy `entry_fts MATCH` thẳng trong test. Đây là ca chứng minh nhánh 2 **phải** tồn tại; không xoá nó vì "nó khẳng định một thứ hỏng".
    - [x] 🔴 `every_branch_filters_out_english_entries` — **AC3**, tra `lock` *(chính xác)* và `dic` *(chuỗi con 3 ký tự)* ⇒ **0** hàng, dù fixture có đúng những hàng đó với `lang='en'`
    - [x] 🔴 `char_idx_candidates_are_verified_as_real_substrings` — **AC4**: tra chuỗi con `中國` ⇒ `國中` **không** có trong kết quả, `中國人` **có**
    - [x] `simplified_headwords_are_reachable` — tra `国` ⇒ khác rỗng *(Bẫy 8 của Story 1.9)*
    - [x] `results_carry_the_source_code_not_the_id` — **AC6**: `source_code` khác rỗng, và hai nguồn của fixture phân biệt được bằng chuỗi
    - [x] `an_fts_query_with_syntax_characters_does_not_error` — truy vấn chứa `*` `-` `"` ⇒ `Ok`, không `Err` *(Task 6, vế thoát ngoặc kép)*
    - [x] 🔴 `opening_a_dictionary_leaves_the_file_byte_identical` — **AC7**: băm SHA-256 trước/sau *(không thêm crate — dùng so **nội dung tệp** bằng `std::fs::read`, tương đương và rẻ hơn)*, cộng khẳng định không tệp `-wal`/`-shm` nào cạnh nó
    - [x] `opening_a_missing_dictionary_fails_and_creates_nothing` — **AC7**: `Err`, và đường dẫn đó vẫn không tồn tại sau đó
    - [x] `a_write_through_the_dictionary_handle_is_refused` — `INSERT` qua `ReadOnlyDb::read` ⇒ `Err` *(bằng chứng dương của `query_only = 1`, đúng khuôn `Store::read`)*
  - [x] `src-tauri/tests/dict_boundary.rs` **(tệp mới)** — **AC5**: quét `src-tauri/src/core/dict/**`, cấm token `LIKE` · `GLOB` · `instr(`, bỏ qua dòng bắt đầu `//`. 🔴 Kèm **sàn quần thể** *(≥ 1 tệp)* và **đối chứng dương** *(`core/dict/**` có thật sự chứa `char_idx` và `entry_fts`)* — đúng khuôn `store_boundary.rs`, không phát minh khuôn mới.
  - [x] **Không sửa** `scope_contract.rs` · `scope_boundary.rs` · `ipc_contract.rs` · `config_invariants.rs`. `store_contract.rs` **chỉ** thêm một dòng của Task 2.

- [x] **Task 9 — Đo THẬT trên `dict-core.db` và ghi bảng** (AC2, AC3, AC4, AC9)
  - [x] Viết một ca `#[test] #[ignore]` *(vd. `bench_three_branches_on_the_real_dictionary`)* đọc đường dẫn tệp từ **biến môi trường** `AURA_DICT_BENCH_DB`. 🔴 **Không** viết cứng đường dẫn; không để nó chạy trong CI *(`#[ignore]` + biến vắng mặt ⇒ bỏ qua)*.
  - [x] Chạy: `AURA_DICT_BENCH_DB=tools/dict-build/out/dict-core.db cargo test --manifest-path src-tauri/Cargo.toml -- --ignored --nocapture`
  - [x] ≥ 200 lượt mỗi nhánh, bỏ 10 lượt làm nóng, ghi **p50 · p95 · p99**
  - [x] Đối chiếu **AC2** *(6 · 3.177 · 4 · 350 · 33)* và **AC4** *(390 → 350 · 33 → 33)*. Lệch ⇒ **DỪNG**, không viết *"dữ liệu đổi rồi"* — tệp `.db` mang `built_at = 2026-08-04T23:53:16Z` và không lượt build nào chạy giữa hai lần đo.
  - [x] Đối chiếu **AC9** với trần. VƯỢT ⇒ ghi số, nêu nhánh, **DỪNG và báo Ice**.
  - [x] Ghi **`EXPLAIN QUERY PLAN` nguyên văn** cho cả ba nhánh *(Task 4/6)*.
  - [x] Chạy lại **đối chứng âm AC3** trên tệp thật: `lock` ⇒ 1 → 0, `dic` ⇒ 572 → 0.

- [x] **Task 10 — Cổng và tài liệu** (AC5, AC8)
  - [x] Chạy lại **toàn bộ** danh sách lệnh của Task 1 ⇒ tất cả XANH
  - [x] `store_boundary.rs::the_scanned_tree_is_large_enough_to_be_real` — sàn **20**, số thật đi từ 26 lên ~30. **Không** nâng sàn *(sàn bắt cây bị cắt, không bắt việc thêm tệp — doc-comment `:39-43`)*. Cùng luật cho `scope_boundary.rs` và `check-i18n.mjs` `RS_FLOOR = 21`.
  - [x] **Không** đụng `scripts/check-dict-build.mjs` — nó canh `tools/dict-build/**`, và story này không sửa một dòng nào ở đó
  - [x] **Không** đụng `dict-manifest.toml` — không tệp `.db` nào được dựng lại ở story này
  - [x] `deferred-work.md`: thêm mục `## Deferred from: 1-11-…` ghi (a) **đính chính** mệnh đề `:279` kèm số đo *(§AC3)*; (b) mục `:266` *(VietPhrase 18 đầu mục trùng)* **vẫn mở** — story này không gộp trùng, và nó là quyết định của **1.13** vì nó chỉ quan sát được khi gom nhiều nguồn; (c) mục `:260` *(khoá theo `code`)* **đã đóng một nửa** — 1.11 giữ đúng hợp đồng, 1.13 phải giữ nốt khi gom
  - [x] **Không sửa** `prd.md` / `epics.md` / `ARCHITECTURE-SPINE.md` / mockup — lệch giữa tài liệu và mã ⇒ ghi vào §Completion Notes cho Ice *(tiền lệ quyết định #3 của Ice ở Story 1.3)*
  - [x] **Không sửa** tệp story đã `done` nào

### Review Findings

- [x] [Review][Defer] NFR1 nhánh 2 (1 ký tự) còn 27% dư địa tới trần, và biện pháp giảm chưa được chọn [src-tauri/src/core/dict/query.rs:121] — deferred, quyết định của người dùng lúc review (2026-08-05): chấp nhận nguyên trạng, không sửa gì bây giờ — revisit khi Story 1.13/1.17 dựng Panel Lookup và có hình dạng phân trang thật để giới hạn số hàng trả về. Đã đo p95 **7,324 ms** bản release / **15,045 ms** bản debug so với trần **10 ms**.
- [x] [Review][Patch] `char_idx()` thiếu `debug_assert!` cho tiền đề "≤ 2 ký tự" [src-tauri/src/core/dict/query.rs:121] — Hàm chỉ đọc hai ký tự đầu của `query` và không có gì canh nó không bị gọi với chuỗi dài hơn (hôm nay chỉ `pick_branch` đảm bảo qua `lookup()`); một lượt gọi trực tiếp sau này với chuỗi ≥ 3 ký tự sẽ âm thầm cắt cụt truy vấn mà không lỗi nào được ném. **Đã sửa:** thêm `debug_assert!` ở đầu hàm.
- [x] [Review][Patch] `dict_boundary.rs::FORBIDDEN` khớp `LIKE`/`GLOB` theo chuỗi con thô, không có ranh giới từ như `instr(` [src-tauri/tests/dict_boundary.rs:44] — `instr(` được cố ý thêm dấu ngoặc để tránh khớp nhầm từ tiếng Anh (`instruction`); `LIKE`/`GLOB` không có cùng biện pháp, nên một hằng số kiểu `GLOBAL_X` (đúng quy ước SCREAMING_SNAKE_CASE mà chính tệp này dùng) sẽ làm cổng đỏ nhầm trong tương lai. **Đã sửa:** thêm `contains_forbidden_token()` xét ranh giới từ ở cả hai đầu khi ký tự đầu/cuối của needle là ký tự "từ".
- [x] [Review][Patch] `ReadOnlyDb::open` nhận mọi `StoreKind`, không riêng `Dict`, không có khẳng định [src-tauri/src/core/store/readonly.rs:61] — không gì canh việc gọi hàm này với ví dụ `StoreKind::Global`; hậu quả hôm nay chỉ là chẩn đoán lỗi bị dán nhãn sai, nhưng đó là dấu hiệu thiếu một ràng buộc mà tài liệu đã khai rõ ý định. **Đã sửa:** thêm `debug_assert_eq!(kind, StoreKind::Dict, …)`.
- [x] [Review][Patch] `Store::open` không có ràng buộc runtime từ chối `StoreKind::Dict` [src-tauri/src/core/store/mod.rs:504] — `StoreSpec` mọi trường đều `pub`, nên không gì trong hệ kiểu ngăn một lượt gọi tương lai dựng `StoreSpec { kind: StoreKind::Dict, .. }` rồi mở qua `Store::open`, tức mở tệp từ điển `READ_WRITE` và có thể đặt `journal_mode = WAL` — chính lớp lỗi thầm lặng mà AD-25/AC7 của story này tồn tại để chặn. Chưa có đường gọi nào khai thác điều này hôm nay, nhưng hệ kiểu không ngăn nó. **Đã sửa:** `Store::open` trả `StoreError::OpenFailed` ngay bước 0, trước khi mở bất kỳ kết nối nào, nếu `kind == StoreKind::Dict`.
- [x] [Review][Patch] Không ca nào tra một truy vấn rỗng (`""`) ở chế độ `Substring` [src-tauri/tests/dict_lookup.rs] — `char_idx()` có nhánh xử lý tường minh cho 0 ký tự (trả `Ok(vec![])` không chạm database) nhưng không có test nào gọi `lookup(db, "", LookupMode::Substring)` để xác nhận điều đó. **Đã sửa:** thêm test `an_empty_substring_query_returns_no_rows`.
- [x] [Review][Patch] `EntryHit.lang` chưa từng được khẳng định trực tiếp trong test [src-tauri/tests/dict_lookup.rs] — mọi ca chỉ đọc `hit.headword`; đúng của trường `lang` chỉ được suy ra gián tiếp qua việc lọc tiếng Anh đúng (`every_branch_filters_out_english_entries`), nên một hồi quy làm sai giá trị `lang` mà vẫn lọc đúng sẽ không bị bắt. **Đã sửa:** thêm `assert_eq!(hit.lang, "zh", …)` vào `results_carry_the_source_code_not_the_id`.
- [x] [Review][Defer] Kế hoạch truy vấn của nhánh 1/2 (B-tree, `char_idx` primary key) chỉ được xác nhận bằng `EXPLAIN QUERY PLAN` chạy tay, không có cổng CI tự động [src-tauri/src/core/dict/query.rs] — deferred, pre-existing (cùng ràng buộc mà AC9 đã chấp nhận: không có tệp `.db` thật trong CI nên không thể tự động hoá theo cùng lý do NFR1 không thành test ngưỡng thời gian trong CI).
- [x] [Review][Defer] Nhánh `char_idx` 1 ký tự bỏ qua xác minh chuỗi con ở Rust, dựa hoàn toàn vào bất biến của `tools/dict-build` (không sinh cặp `char_idx` sai) mà không có cổng chéo hai workspace nào kiểm chứng [src-tauri/src/core/dict/query.rs:121] — deferred, pre-existing (ranh giới hai workspace tách rời đã chốt từ Story 1.9 AC4; story này kế thừa chứ không tạo ra quyết định đó).
- [x] [Review][Defer] Không có giới hạn trên cho độ dài truy vấn trước khi đưa vào `chars()`/cấp phát chuỗi/FTS [src-tauri/src/core/dict/query.rs] — deferred, pre-existing (story này tường minh cấm dựng IPC command hay chạm frontend; validate độ dài đầu vào thuộc về tầng IPC/UI của Story 1.13/1.17, nơi bên gọi không tin cậy thật sự xuất hiện).

---

## Dev Notes

### Ranh giới phạm vi — đọc trước khi gõ dòng đầu tiên

| Thứ | Trong story này? |
|---|---|
| `pick_branch` theo `chars().count()` | ✅ **Có** — hạt nhân |
| Ba nhánh SQL trên **MỘT** tệp `.db` | ✅ **Có** |
| Xác minh chuỗi con ở Rust | ✅ **Có** |
| Lọc `lang = 'zh'` mọi nhánh | ✅ **Có** |
| `ReadOnlyDb` + `open_readonly_connection` trong `core/store/` | ✅ **Có** — §Quyết định #1 |
| `StoreKind::Dict` | ✅ **Có** |
| Fixture `.db` dựng trong test + cổng parity DDL | ✅ **Có** |
| Bench tay trên tệp thật (`#[ignore]` + env) | ✅ **Có** |
| **Cổng `DictionarySource`** | ❌ **Không** — **1.13**. không `ports/mod.rs` giữ nguyên **0 trait** |
| **Gom kết quả từ NHIỀU tệp `.db`** | ❌ **Không** — **1.13** |
| **Nhóm kết quả theo nguồn, `dict_sense`/`example`/`citation`** | ❌ **Không** — **1.13** (FR29–FR32) |
| **Gộp đầu mục trùng của VietPhrase** | ❌ **Không** — **1.13**, `deferred-work.md:266` |
| **Đường tra cứu TIẾNG ANH** | ❌ **Không** — **1.11b**, và nó đang **🔴 BỊ CHẶN** chờ AD mới của Winston |
| **`Matcher` / stemming / `jieba-rs`** | ❌ **Không** — **1.12** (AD-17). không `core/matching/` giữ nguyên |
| **`#[tauri::command]` / IPC / `invoke_handler`** | ❌ **Không** — **1.13/1.17** |
| **Panel Lookup, bất kỳ tệp `.vue`/`.ts` nào** | ❌ **Không** — **1.17**. **0 dòng frontend** |
| **`MessageKey` mới / khoá `vi.json` mới** | ❌ **Không** — §Quyết định #4 |
| Sửa `tools/dict-build/**` | ❌ **Không** — **0 dòng** |
| Dựng lại tệp `.db` nào / sửa `dict-manifest.toml` | ❌ **Không** |
| Đưa `.db` vào `bundle.resources` / `tauri.conf.json` | ❌ **Không** — **10.1**, `deferred-work.md:238` |
| Crate mới cho bất kỳ cây nào | ❌ **Không** — **0 crate** |
| Nới `STORE_DIR` / `FORBIDDEN` của `store_boundary.rs` | ❌ **KHÔNG BAO GIỜ** — AC8 |
| Đổi `[profile.release]` / `Cargo.toml` | ❌ **Không** — Ice chốt lần thứ năm |

### Trạng thái repo hiện tại — số, không phải mô tả

| Thứ | Số / trạng thái |
|---|---|
| Commit nền | `5a68df7`, cây **sạch** |
| `.rs` dưới `src-tauri/src/**` | **26** |
| `.rs` dưới `src-tauri/tests/**` | **6** *(62 test)* |
| `core/dict/mod.rs` | **7 dòng doc-comment, 0 dòng mã** |
| `ports/mod.rs` | **0 trait** — đúng như 1.13 sẽ nhận |
| `core/matching/mod.rs` | **7 dòng doc-comment, 0 dòng mã** — **1.12** |
| `RS_FLOOR` *(`store_boundary.rs:52`)* | **20** — không nâng |
| `RS_FLOOR` *(`scope_boundary.rs:50`)* | **20** — không nâng |
| `RS_FLOOR` / `VUE_FLOOR` *(`check-i18n.mjs`)* | **21 / 1** — không nâng |
| Khoá trong `src/i18n/vi.json` | **16** — không phải không đổi |
| `rusqlite` của `src-tauri` | **0.40.1** *(`tools/dict-build` dùng **0.37** — hai workspace tách rời, không so API giữa hai cây)* |
| `tools/dict-build/out/dict-core.db` | **194.998.272** byte · sha `2145c7ae…` · `built_at = 2026-08-04T23:53:16Z` |
| `tools/dict-build/out/dict-thieu-chuu.db` | **5.787.648** byte · **9.897** đầu mục |
| `tools/dict-build/out/dict-vietphrase.db` | **160.083.968** byte · **679.302** đầu mục · **2.576.667** cặp `char_idx` |
| `PRAGMA journal_mode` của cả ba tệp | **`delete`** |
| `PRAGMA user_version` / `dict_meta('schema_version')` | **1 / 1** |
| Tệp `.db` trong git | **KHÔNG** — `.gitignore: *.db` (AD-25) |

### 🔴 Số ĐO THẬT trên `dict-core.db` — đo 2026-08-05, tái lập được

Mọi số dưới đây đo bằng `sqlite3 tools/dict-build/out/dict-core.db`. SQL nguyên văn ở §References để lượt rà sau tái lập.

**Phân bố đầu mục** *(592.538 tổng)*:

| `code` | `lang` | Đầu mục |
|---|---|---:|
| `cc-cedict` | zh | 124.758 |
| `cvdict` | zh | 122.596 |
| `en-wiktionary` | zh | 174.677 |
| `unihan` | zh | 49.870 |
| `viwiktionary` | zh | 1.598 |
| **`viwiktionary-en`** | **en** | **119.039** |

**Ba nhánh** *(có lọc `lang='zh'`)*:

| Truy vấn | Nhánh 1 *(chính xác)* | Nhánh 2 *(`char_idx`)* | Nhánh 3 *(trigram)* |
|---|---:|---:|---:|
| `山` | **6** | **3.177** | 🔴 **0** ← bẫy Giai đoạn 0, tái lập được |
| `中國` | **4** | **390 → 350** *(sau xác minh)* | 🔴 **0** ← bẫy Giai đoạn 0 |
| `中國人` | — | — | **33 → 33** |
| `国` *(giản thể, qua `headword_simp`)* | **5** | — | — |

**Rò rỉ tiếng Anh** — đo cả hai chiều:

| | Không lọc `lang` | Lọc `lang='zh'` |
|---|---:|---:|
| `entry_fts MATCH '"中國人"'` giao `lang='en'` | **0** | 0 |
| `char_idx` thuộc lớp tiếng Anh / tổng | **9 / 1.341.179** *(0,00067%)* | — |
| `headword = 'lock'` | **1** | **0** |
| `entry_fts MATCH '"dic"'` | **572** *(100% `lang='en'`)* | **0** |

> ⇒ Với truy vấn **thuần Hán**, rò rỉ đo được là **0**. Với truy vấn **Latin**, rò rỉ là **thật và lớn**. Đó là toàn bộ lý do AC3 đòi đối chứng âm bằng truy vấn Latin, không bằng truy vấn Hán.

### Quyết định đã chốt trong story — không phải lựa chọn của dev

#### Quyết định #1 — Đường mở tệp sống ở `core/store/`, không ở `core/dict/`

**Vấn đề:** `tests/store_boundary.rs:62` cấm hai chuỗi `"rusqlite"` và `"Connection::open"` ở mọi tệp ngoài `src/core/store/**`. `core/dict/` cần mở tệp `.db`.

**Ba đường, chốt đường thứ ba:**

| Đường | Vì sao loại |
|---|---|
| Nới `STORE_DIR` thành hai thư mục | Cổng canh AD-11. Một cổng có hai miễn trừ là một cổng sẽ có ba — và miễn trừ thứ ba sẽ là một module **có** ghi |
| Tái dùng thẳng `Store::open` | Nó mở `READ_WRITE \| CREATE`, đặt `journal_mode = WAL` *(**GHI VÀO** tệp ⇒ SHA-256 đổi ⇒ AD-25 vỡ)*, chạy di trú, dựng luồng writer + luồng checkpoint. Bốn thứ đó không có nghĩa gì với một tệp chỉ đọc |
| ✅ **`ReadOnlyDb` mới trong `core/store/`** | Cổng giữ **đúng một** miễn trừ; `core/dict/` viết SQL qua `ReadHandle`/`SqlResult`/`Row` đã tái xuất, không gõ tên crate; pool `Mutex`+`Condvar`+`Lease` của `reader.rs` **dùng lại nguyên**, không viết pool thứ hai |

⚠️ Doc-comment `store_boundary.rs:146-151` **đã dự liệu chính tình huống này** — nó nêu đích danh `core/dict/mod.rs:6` và giải thích vì sao dòng comment ở đó hợp lệ. Đường đã được nghĩ tới; đừng đi đường khác.

#### Quyết định #2 — `lookup` nhận `ReadHandle`, không nhận `ReadOnlyDb`

`lookup(db: ReadHandle<'_>, …)` là **hàm thuần theo kết nối**. Chỗ gọi mở kho. Ba hệ quả, cả ba đều là điều kiện của một story sau:

1. **1.13** gọi `lookup` **một lần cho mỗi tệp** rồi gom — với một chữ ký nhận `ReadOnlyDb`, nó phải mở/đóng hoặc mượn lồng nhau.
2. Test dựng fixture rồi gọi thẳng, không cần dựng cả `ReadOnlyDb` cho ca thuần logic.
3. Cùng khuôn `bootstrap_config(store: Option<&Store>)` của Story 1.8 §Quyết định #6: **hàm thuần là đường sản phẩm**, vỏ là thứ bỏ đi được trong test.

#### Quyết định #3 — Fixture chép DDL + **cổng parity đọc tệp nguồn**

CI không có tệp `.db` nào *(`.gitignore: *.db`)*, và tệp thật nặng 195 MB. Ba đường:

| Đường | Vì sao loại |
|---|---|
| Commit một `.db` nhỏ vào git | Phá `.gitignore: *.db` — dòng đó là AD-25, và doc-comment của nó viết *"Đừng gỡ dòng này"* |
| Chạy `dict-build` trong test | Hai workspace tách rời **có chủ ý** *(AC4 của Story 1.9)*; gọi chéo là hút build tool vào cây phụ thuộc sản phẩm |
| ✅ **Chép DDL + test parity** | Rẻ, chạy trong `cargo test`, và **cổng bắt được việc trôi**: nếu `tools/dict-build/src/schema.rs` đổi mà fixture không đổi, `fixture_ddl_is_verbatim_from_dict_build_schema` **đỏ** |

⚠️ Cổng parity so **văn bản**, không so `sqlite_master`. Nó chạy được mà không cần một tệp `.db` nào — đó chính là điều kiện để nó ở trong CI.

#### Quyết định #4 — Không `MessageKey` mới, không khoá `vi.json` mới

Mọi lỗi story này phát ra là lỗi **kho**: mở tệp trượt ⇒ `StoreError::OpenFailed`, truy vấn trượt ⇒ `StoreError::ReadFailed`. Cả hai đã có khoá từ Story 1.7, và `From<StoreError> for IpcError` điền `params = {"store": "dict"}` **tự động** nhờ `StoreKind::Dict::as_str()`. Đúng tiền lệ §Quyết định #7 của Story 1.8.

Và story này **không** dựng IPC command, nên không lỗi nào của nó vượt ranh giới hôm nay. Câu chữ cho người dùng là việc của **1.13/1.17**.

#### Quyết định #5 — Chế độ do **chỗ gọi** quyết, không đoán từ nội dung truy vấn

`LookupMode` là tham số. Một hàm tự đoán *"chắc người dùng muốn tra chính xác"* là một quy tắc nghiệp vụ **ẩn** mà 1.17 (Auto-Lookup) và 1.18 sẽ phải đoán ngược lại. AD-26 khai **ba nhánh**, không khai một cơ chế đoán.

**Và không fallback dây chuyền** *(thử nhánh 1, rỗng thì thử nhánh 2…)*. AD-26 nói *"tra chính xác → B-tree"*, không nói *"thử B-tree trước"*. Một fallback ngầm làm số đo AC9 vô nghĩa *(mỗi lượt tra chạy hai đến ba truy vấn)* và làm `QueryBranch` trả về nói dối.

### Bảy cái bẫy — sáu trong bảy cho ra một lượt CI XANH với hành vi SAI

#### Bẫy 1 — Chọn nhánh bằng `len()` thay vì `chars().count()` 🔴 **đắt nhất**

```rust
// 🔴 SAI — "山".len() == 3
let branch = if query.len() <= 2 { CharIdx } else { FtsTrigram };
```

Mọi truy vấn tiếng Trung 1–2 ký tự *(tức **phần lớn từ được tra nhiều nhất** — 山, 打, 中國, 學生)* rơi vào `entry_fts`, trả **0** hàng trong 0,01 ms, không lỗi nào được ném. Build xanh, mọi test "khác rỗng" trên một fixture chứa từ 3 ký tự cũng xanh.

**Đó chính xác là phát hiện nghiêm trọng nhất của Giai đoạn 0**, là lý do FR39 tồn tại, và là lý do AD-26 có ba nhánh chứ không hai. **Luật:** `chars().count()`, và ca `branch_is_picked_by_char_count_not_byte_length` là ca không được xoá.

#### Bẫy 2 — `ch IN ('中','國')` thay vì `INTERSECT` 🔴

`IN` là **hợp** — nó trả mọi đầu mục chứa `中` **hoặc** `國` *(hàng chục nghìn)*. `INTERSECT` là **giao**. Cả hai "khác rỗng", nên mọi AC phát biểu bằng *"khác rỗng"* đều xanh. Số duy nhất bắt được: **390** ứng viên của AC4.

#### Bẫy 3 — Bỏ bước xác minh chuỗi con 🟡

`char_idx` INTERSECT trả về `國中` khi tra `中國`. Đã đo: **40 dương tính giả trên 390 ứng viên** cho đúng một truy vấn. Người dùng tra *"Trung Quốc"* nhận về *"trong trường"* — kết quả **khác rỗng**, **sai**, và không phép kiểm nào phát biểu bằng `> 0` bắt được.

#### Bẫy 4 — `entry_fts MATCH` không bọc ngoặc kép 🔴

Không ngoặc, chuỗi truy vấn đi vào **cú pháp truy vấn FTS5**. Một truy vấn chứa `*` `-` `^` `(` `:` `NEAR` ⇒ `SQLITE_ERROR`. Nghĩa là **tra cứu báo lỗi vì nội dung người dùng bôi đen** — tệ hơn hẳn trả rỗng, vì nó lộ ra ở tay người dùng thật chứ không ở CI *(fixture chỉ có chữ Hán sạch)*.

#### Bẫy 5 — Tái dùng `pragmas::apply_reader_pragmas` 🔴

Nó gọi `verify_wal`. Cả ba tệp từ điển ở `journal_mode = delete` *(đã đo)* ⇒ `StoreError::WalUnavailable { mode: "delete" }` ngay lượt mở đầu tiên. Cám dỗ tiếp theo — *"thì đặt WAL cho nó"* — là đường hỏng thật: `PRAGMA journal_mode = WAL` **ghi vào** tệp, SHA-256 đổi, `dict-manifest.toml` thành sai, và AD-25 *(artifact có checksum)* vỡ mà không cổng nào bắt *(`check-dict-manifest.mjs` cố ý không đọc `.db`)*.

#### Bẫy 6 — Mở tệp với cờ mặc định 🔴

`OpenFlags::default()` là `READ_WRITE | CREATE | NO_MUTEX | URI`. Hai thứ hỏng cùng lúc: `CREATE` biến một đường dẫn gõ sai thành **một tệp rỗng mới toanh** — mọi truy vấn sau đó trả rỗng, không lỗi nào được ném, và người dùng thấy *"tra từ không ra kết quả"*; `URI` làm một thư mục chứa `?` trong tên mở ra ở một chỗ khác *(bẫy đã ghi ở `pragmas.rs:34-39`)*.

#### Bẫy 7 — Chuỗi tiếng Việt CÓ DẤU ở vị trí mã 🟡

`scripts/check-i18n.mjs` Kiểm A quét `src-tauri/**/*.rs`. `src/core/store/**` và `src/core/dict/**` **không** nằm trong `EXEMPT`; chỉ `src-tauri/tests/**` được miễn trừ. Doc-comment và comment có dấu là **hợp lệ**; `panic!`, `debug_assert!`, `format!`, `Display` thì không. *(Cổng đã bắt đúng ca này một lần ở `core/i18n/mod.rs`, lượt review 2026-08-04.)*

### Trí tuệ từ story trước — thứ áp thẳng vào story này

- **Đối chứng âm là bắt buộc, không phải trang trí.** Story 1.10b: `wiktextract_common` viết cứng `lang: "zh"`, và **chỉ** đối chứng âm của AC3 bắt được — mọi phép khẳng định dương đều xanh. Story này có **ba** đối chứng âm bắt buộc: `entry_fts` trên 1–2 ký tự *(AC2)*, truy vấn Latin *(AC3)*, `國中` bị loại *(AC4)*.
- **Số nghiệm thu phải là số của một lượt chạy thật, không phải `grep`.** Story 1.10b §Bẫy 3: `grep -c` lệch 16% so với số thật và một dev tin nó sẽ sửa parser cho tới khi nó hỏng thật.
- **`--layer all` là mặc định và nó dựng lại CẢ BA tệp.** Story này không chạy `dict-build` một lần nào — nhưng nếu bạn nghĩ mình cần, đọc §Bẫy 5 của Story 1.10b trước.
- **Miễn trừ cổng phải có TÊN và có LÝ DO.** Khuôn có sẵn ở `store_boundary.rs:27-31` và `check-i18n.mjs::EXEMPT`. `dict_boundary.rs` mới phải theo đúng khuôn đó *(sàn quần thể + đối chứng dương)*, không phát minh khuôn thứ hai.
- **Sàn quần thể đếm TỆP, không đếm nội dung.** Đã ghi bốn lần trong `deferred-work.md`. Story này thêm tệp nên mọi sàn đều dư — đừng nâng chúng, chúng tồn tại để bắt một cây **bị cắt**.

### Testing standards

- **Không `sleep` dài, không ngưỡng thời gian trong CI.** Bench của AC9 là `#[test] #[ignore]` lái bằng biến môi trường; vắng biến ⇒ bỏ qua. *(Tiền lệ: `Tuning` thu nhỏ của Story 1.7 để ca chạy dưới một giây trên cả hai nền tảng.)*
- **`cargo test` chạy được mà không cần webview** — mọi ca của story này chạy trên một fixture trong thư mục tạm. 🔴 **Không thêm `tempfile`**: nó là dev-dependency của `tools/dict-build`, **không** của `src-tauri`, và `store_contract.rs:12` cấm tường minh. **Khuôn có sẵn — đọc và dùng lại, đừng viết bản thứ hai:** `store_contract.rs:54` `fn temp_dir(tag: &str) -> PathBuf` *(`std::env::temp_dir()` + pid + bộ đếm nguyên tử, để hai ca chạy song song không đụng nhau và test không thành flaky)*.
- **Đường dẫn tương đối trong test lấy qua `env!("CARGO_MANIFEST_DIR")`**, chuẩn hoá `\` → `/` khi so chuỗi *(bài học NFR14 ở `store_boundary.rs:68-73`: `starts_with` trên Windows không bao giờ khớp, và test chỉ đỏ trên một nhánh ma trận)*.
- **`npm run build` chạy TRƯỚC `cargo test` của `src-tauri`** — `generate_context!` nhúng `dist/` lúc biên dịch.

### Project Structure Notes

Cây nguồn sau story này — mọi đường dẫn khớp `ARCHITECTURE-SPINE.md#Cây nguồn`, không thư mục mới nào ngoài khai báo:

```text
src-tauri/src/core/
  dict/
    mod.rs        # SỬA — hôm nay 7 dòng doc-comment. API công khai + pick_branch
    query.rs      # MỚI — ba nhánh SQL + xác minh chuỗi con
  store/
    mod.rs        # SỬA — StoreKind::Dict · pub mod readonly · pub use ReadOnlyDb
    pragmas.rs    # SỬA — open_readonly_connection · apply_dict_reader_pragmas
    reader.rs     # SỬA — ReaderPool::open_readonly (dùng chung thân với open)
    readonly.rs   # MỚI — ReadOnlyDb: open · read · close · Drop
src-tauri/tests/
  dict_lookup.rs    # MỚI — fixture + hành vi ba nhánh + parity DDL
  dict_boundary.rs  # MỚI — AC5: cấm LIKE/GLOB/instr( dưới core/dict/**
  store_contract.rs # SỬA — MỘT dòng: StoreKind::Dict.as_str() == "dict"
```

⚠️ `query.rs` là tệp thứ hai của `core/dict/` chứ không phải một module con mới trong `ARCHITECTURE-SPINE.md` — Consistency Conventions nói *"một module cho một khái niệm miền"*, và `dict/` vẫn là **một** khái niệm. Tách tệp là bố cục trong module, không phải một module mới.

### References

**Kiến trúc**
- AD-26 *Ba nhánh truy vấn tiếng Trung* `[ADOPTED]` — [ARCHITECTURE-SPINE.md#AD-26](../planning-artifacts/architecture/architecture-AuraTranslate-2026-08-02/ARCHITECTURE-SPINE.md)
- AD-19 *Không tồn tại bước hợp nhất nguồn từ điển* · AD-10 *Mỗi lớp gỡ rời một tệp `.db`* · AD-7 *dữ liệu từ điển chỉ đọc, luôn luôn* · AD-11 *một writer duy nhất* · AD-25 *artifact có phiên bản và checksum* — cùng tệp
- Bảng Stack — `LIKE` trên đường nóng tra cứu nằm trong *"Không dùng, đã loại có lý do"*
- Capability map: **C3** Dictionary & Lookup → `core/dict/`, `ports/DictionarySource`, `resources/dict/`

**PRD / Epic**
- FR39 — `prd.md:496` *(*"trả về kết quả cho truy vấn 1 ký tự, 2 ký tự và 3 ký tự trở lên"*)*
- NFR1 + giả định `[A1]` — `prd.md:814`, `:1072`
- Rủi ro R11 — `prd.md:1109`
- Story 1.11 — `epics.md:1434-1475`; story kề: 1.11b `:1478` · 1.12 `:1497` · 1.13 `:1530`

**Nghiên cứu — nguồn của mọi con số Giai đoạn 0**
- `research/phase-0-spike-results-2026-08-02.md` §Phép đo 1 *(độ trễ)* · §Phép đo 3a *(trigram trả rỗng cho 1–2 ký tự)* · §3b *(`char_idx`)* · §3c *(ba nhánh)*

**Mã đã có — đọc trước khi viết**
- `tools/dict-build/src/schema.rs` — **nguồn sự thật của lược đồ**; DDL của fixture chép từ đây
- `tools/dict-build/src/char_idx.rs` — `is_han()` + luật phủ **cả** `headword` lẫn `headword_simp`
- `tools/dict-build/src/finalize.rs` — vì sao ba tệp ở `journal_mode = DELETE`
- `src-tauri/src/core/store/{mod,pragmas,reader}.rs` — khuôn mở kết nối, "đặt rồi đọc lại", pool
- `src-tauri/tests/store_boundary.rs` — cổng AC8 + khuôn cho `dict_boundary.rs`
- `src-tauri/src/commands/config.rs` — khuôn "hàm thuần là đường sản phẩm"

**Bàn giao đang mở**
- `deferred-work.md:260` *(khoá theo `code`, không theo `id`)* — **1.11/1.13**
- `deferred-work.md:266` *(VietPhrase 18 đầu mục trùng)* — **1.11/1.13**, story này không giải
- `deferred-work.md:277-280` *(lọc `lang`)* — **1.11b/1.13**, story này giữ hợp đồng và **đính chính** một mệnh đề
- `deferred-work.md:275` *(AD mới cho tiếng Anh — Winston)* — **vẫn CHẶN 1.11b**, không phải story này

**SQL nghiệm thu — chép-dán, tái lập mọi số ở §Số ĐO THẬT**

```sh
sqlite3 tools/dict-build/out/dict-core.db <<'SQL'
SELECT s.code, e.lang, COUNT(*) FROM dict_entry e
  JOIN dict_source s ON s.id=e.source_id GROUP BY s.code,e.lang ORDER BY s.code;
SELECT COUNT(*) FROM dict_entry WHERE headword='山' AND lang='zh';           -- 6
SELECT COUNT(*) FROM dict_entry WHERE headword='中國' AND lang='zh';         -- 4
SELECT COUNT(*) FROM dict_entry WHERE headword_simp='国';                    -- 5
SELECT COUNT(*) FROM char_idx WHERE ch='山';                                 -- 3177
SELECT COUNT(*) FROM entry_fts WHERE entry_fts MATCH '"山"';                 -- 0  (đối chứng âm)
SELECT COUNT(*) FROM entry_fts WHERE entry_fts MATCH '"中國"';               -- 0  (đối chứng âm)
SELECT COUNT(*) FROM entry_fts WHERE entry_fts MATCH '"中國人"';             -- 33
SELECT COUNT(*) FROM dict_entry e WHERE e.lang='zh' AND e.id IN (
  SELECT entry_id FROM char_idx WHERE ch='中'
  INTERSECT SELECT entry_id FROM char_idx WHERE ch='國');                    -- 390 (ứng viên)
SELECT COUNT(*) FROM dict_entry e WHERE e.lang='zh' AND e.id IN (
  SELECT entry_id FROM char_idx WHERE ch='中'
  INTERSECT SELECT entry_id FROM char_idx WHERE ch='國')
  AND (instr(e.headword,'中國')>0 OR instr(COALESCE(e.headword_simp,''),'中國')>0); -- 350
SELECT lang, COUNT(*) FROM dict_entry WHERE headword='lock' GROUP BY lang;   -- en|1
SELECT e.lang, COUNT(*) FROM entry_fts f JOIN dict_entry e ON e.id=f.rowid
  WHERE entry_fts MATCH '"dic"' GROUP BY e.lang;                             -- en|572
SELECT (SELECT COUNT(*) FROM char_idx),
  (SELECT COUNT(*) FROM char_idx c JOIN dict_entry e ON e.id=c.entry_id
   WHERE e.lang='en');                                                       -- 1341179 | 9
PRAGMA journal_mode;  PRAGMA user_version;  SELECT key,value FROM dict_meta;
SQL
```

---

## Dev Agent Record

### Agent Model Used

`claude-opus-5` (Claude Code, `bmad-dev-story`) — 2026-08-05.

### Debug Log References

#### ① Task 1 — đường cơ sở tại `5a68df7` (cây sạch, không sửa gì)

| Phép đo | Số |
|---|---|
| `git rev-parse HEAD` | `5a68df78706fc1bc150240f5dadc5a6a57cf4ac4` |
| `cargo test --locked` | **62** passed · 0 failed · 1 ignored *(doctest)* |
| `.rs` dưới `src-tauri/src/**` | **26** |
| `.rs` dưới `src-tauri/tests/**` | **6** |
| `check:deps` · `check:i18n` · `check:commands` · `check:tokens` · `check:dict` · `check:dict-manifest` | **6/6 OK** |

`ls -la tools/dict-build/out/` — ba tệp có mặt, kích thước **khớp §Trạng thái repo**:

```text
-rw-r--r--  194998272  5 Aug 11:19 dict-core.db
-rw-r--r--    5787648  5 Aug 09:32 dict-thieu-chuu.db
-rw-r--r--  160083968  5 Aug 09:33 dict-vietphrase.db
```

`PRAGMA journal_mode` của `dict-core.db` ⇒ **`delete`** *(xác nhận Bẫy 5 là thật: `verify_wal` sẽ đỏ ngay lượt mở đầu tiên)*.

#### ② Task 4/6 — `EXPLAIN QUERY PLAN` NGUYÊN VĂN, cả ba nhánh

Chạy trên `dict-core.db` thật, SQL chép nguyên từ `src-tauri/src/core/dict/query.rs`.

**Nhánh 1 — B-tree chính xác** ⇒ 🔴 `MULTI-INDEX OR`, **không** `SCAN dict_entry`. Vế `OR` đi qua **cả hai** chỉ mục, nên không cần tách thành `UNION`:

```text
QUERY PLAN
|--MULTI-INDEX OR
|  |--INDEX 1
|  |  `--SEARCH e USING INDEX idx_entry_headword (headword=?)
|  `--INDEX 2
|     `--SEARCH e USING INDEX idx_entry_headword_simp (headword_simp=?)
|--SEARCH s USING INTEGER PRIMARY KEY (rowid=?)
`--USE TEMP B-TREE FOR ORDER BY
```

**Nhánh 2 — `char_idx`, 1 ký tự:**

```text
QUERY PLAN
|--SEARCH e USING INTEGER PRIMARY KEY (rowid=?)
|--LIST SUBQUERY 1
|  `--SEARCH char_idx USING PRIMARY KEY (ch=?)
`--SEARCH s USING INTEGER PRIMARY KEY (rowid=?)
```

**Nhánh 2 — `char_idx`, 2 ký tự** ⇒ 🔴 `INTERSECT`, **không** phải phép hợp:

```text
QUERY PLAN
|--SEARCH e USING INTEGER PRIMARY KEY (rowid=?)
|--LIST SUBQUERY 2
|  `--COMPOUND QUERY
|     |--LEFT-MOST SUBQUERY
|     |  `--SEARCH char_idx USING PRIMARY KEY (ch=?)
|     `--INTERSECT USING TEMP B-TREE
|        `--SEARCH char_idx USING PRIMARY KEY (ch=?)
`--SEARCH s USING INTEGER PRIMARY KEY (rowid=?)
```

**Nhánh 3 — FTS5 trigram:**

```text
QUERY PLAN
|--SCAN f VIRTUAL TABLE INDEX 0:M1
|--SEARCH e USING INTEGER PRIMARY KEY (rowid=?)
|--SEARCH s USING INTEGER PRIMARY KEY (rowid=?)
`--USE TEMP B-TREE FOR ORDER BY
```

> ⚠️ **Chữ `SCAN` ở nhánh 3 là hành vi ĐÚNG, không phải một vi phạm.** Story viết *"phải thấy `VIRTUAL TABLE INDEX`, không thấy `SCAN`"*; SQLite **luôn** dùng từ `SCAN` cho một bảng ảo. Phần mang nghĩa là hậu tố **`:M1`** — nó nói ràng buộc `MATCH` **đã** được đẩy xuống mô-đun FTS5. Kế hoạch hỏng sẽ là `VIRTUAL TABLE INDEX 0:` **không** có `M`. Ghi vào `deferred-work.md` để lượt rà sau không đọc nhầm.

#### ③ Task 9 — bảng AC2, đo lại trên `dict-core.db` THẬT

Đo hai lượt độc lập: `sqlite3` CLI, và đường sản phẩm qua `ReadOnlyDb` + `lookup`. **Hai lượt khớp nhau tuyệt đối.**

| Truy vấn | Chế độ | Nhánh đã đi | Số nghiệm thu | ĐO ĐƯỢC | |
|---|---|---|---:|---:|---|
| `山` | chính xác | `ExactBtree` | 6 | **6** | ✓ |
| `山` | chuỗi con | `CharIdx` | 3.177 | **3.177** | ✓ |
| `中國` | chính xác | `ExactBtree` | 4 | **4** | ✓ |
| `中國` | chuỗi con | `CharIdx` | 350 | **350** | ✓ |
| `中國人` | chuỗi con | `FtsTrigram` | 33 | **33** | ✓ |

🔴 **Đối chứng âm bắt buộc của AC2 — tái lập nguyên văn:**

```text
SELECT COUNT(*) FROM entry_fts WHERE entry_fts MATCH '"山"';    ⇒ 0
SELECT COUNT(*) FROM entry_fts WHERE entry_fts MATCH '"中國"';  ⇒ 0
SELECT COUNT(*) FROM entry_fts WHERE entry_fts MATCH '"中國人"'; ⇒ 33
```

⇒ Hai số **0** đó là **bằng chứng dương** rằng nhánh 2 phải tồn tại: FTS5 `trigram` không lập chỉ mục token ngắn hơn ba ký tự. Khoá bằng test `fts_returns_nothing_for_one_and_two_character_queries`, kèm đối chứng dương *(`中國人` ⇒ > 0)* để ca đó không xanh trên một `entry_fts` rỗng vì quên `rebuild`.

#### ④ Task 9 — bảng AC4, xác minh chuỗi con

| | `中國` | `中國人` |
|---|---:|---:|
| Ứng viên *(`char_idx` INTERSECT, lọc `lang='zh'`)* | **390** | **33** |
| Sau khi xác minh chuỗi con ở Rust | **350** | **33** |
| ⇒ dương tính giả bị loại | **40** | **0** |

Cả bốn số **khớp tuyệt đối** số nghiệm thu. Kết quả **0** của `中國人` là một **phép đo**, không phải cái cớ bỏ bước — nhánh 3 chạy **cùng một hàm** `verify_substring` với nhánh 2, không có bản thứ hai.

🔴 **Kiểm chứng bằng ĐỘT BIẾN — test có bắt được bẫy thật không:**

| Đột biến | Test đỏ? |
|---|---|
| `query.chars().count()` → `query.len()` *(Bẫy 1)* | ✅ **5 ca đỏ**, gồm `branch_is_picked_by_char_count_not_byte_length` |
| Bỏ `verify_substring` khỏi nhánh 2 và 3 *(Bẫy 3)* | ✅ **2 ca đỏ**, gồm `char_idx_candidates_are_verified_as_real_substrings` |

⇒ Hai ca đắt nhất của story không phải test-trang-trí; chúng đỏ thật trên bản sai. *(Đột biến đã được hoàn nguyên; cây hiện ở bản đúng.)*

#### ⑤ Task 9 — bảng AC9, p50 · p95 · p99

200 lượt mỗi nhánh, bỏ 10 lượt làm nóng, `dict-core.db` thật *(194.998.272 byte)*.

**Bản RELEASE — 🔴 đây là số NGHIỆM THU** *(bản người dùng chạy)*:

| Nhánh | AD-26 công bố | Trần story | p50 | **p95** | p99 | Phán quyết |
|---|---|---:|---:|---:|---:|---|
| 1 — B-tree chính xác (`山`) | 0,02 ms | 1 ms | 0,069 | **0,083** | 0,108 | ✅ **ĐẠT** |
| 2 — `char_idx` **1 ký tự** (`山`) | 0,15–4,5 ms | 10 ms | 5,409 | 🔴 **7,324** | 8,017 | ⚠️ **ĐẠT** *(sát trần)* |
| 2 — `char_idx` **2 ký tự** (`中國`) | 0,15–4,5 ms | 10 ms | 0,798 | **1,039** | 1,326 | ✅ **ĐẠT** |
| 3 — FTS5 trigram (`中國人`) | 0,13–0,19 ms | 1 ms | 0,241 | **0,448** | 0,651 | ✅ **ĐẠT** |
| **Nhánh chậm nhất** | — | 🔴 **≤ 10 ms** | — | **7,324** | — | ✅ **ĐẠT** |

**Bản DEBUG — không phải số nghiệm thu, nhưng là số mọi dev sẽ thấy:**

| Nhánh | p50 | **p95** | p99 | Phán quyết |
|---|---:|---:|---:|---|
| 1 — B-tree (`山`) | 0,125 | 0,133 | 0,154 | ✅ ĐẠT |
| 2 — `char_idx` 1 ký tự (`山`) | 10,594 | 🔴 **15,045** | 16,564 | 🔴 **VƯỢT** |
| 2 — `char_idx` 2 ký tự (`中國`) | 1,920 | 2,566 | 2,633 | ✅ ĐẠT |
| 3 — trigram (`中國人`) | 0,445 | 0,768 | 0,897 | ✅ ĐẠT |

> 🔴 **BÁO CHO ICE — đây là mục cần một quyết định, không phải một dòng ghi chú.** Xem §Completion Notes ④.

Lệnh tái lập:

```sh
AURA_DICT_BENCH_DB="$PWD/tools/dict-build/out/dict-core.db" \
  cargo test --release --manifest-path src-tauri/Cargo.toml --test dict_lookup -- --ignored --nocapture
```

⚠️ Đường dẫn phải **tuyệt đối** — CWD của một test là `src-tauri/`, không phải gốc repo.

#### ⑥ Task 9 — đối chứng âm AC3 trên tệp THẬT *(chỉ đo được bằng truy vấn LATIN)*

| Truy vấn | Nhánh | Không lọc `lang` | Có lọc `lang='zh'` | Số nghiệm thu |
|---|---|---:|---:|---|
| `lock` *(chính xác)* | 1 | **1** | **0** | 1 → 0 ✓ |
| `dic` *(chuỗi con)* | 3 | **572** *(100% `lang='en'`)* | **0** | 572 → 0 ✓ |

Đường sản phẩm trả **0 hàng** cho cả hai — khoá bằng `every_branch_filters_out_english_entries`, kèm đối chứng dương *(fixture **có thật** hai hàng `lang='en'`)* để ca đó không xanh trên một fixture không có hàng tiếng Anh nào.

#### ⑦ Task 10 — lượt chạy cổng cuối cùng, TẤT CẢ XANH

| Lệnh | Kết quả |
|---|---|
| `npm run build` | ✅ OK |
| `cargo test --locked` | ✅ **81** passed · **0** failed · 2 ignored *(bench + doctest)* |
| `check:deps` | ✅ OK |
| `check:i18n` | ✅ OK |
| `check:commands` | ✅ OK |
| `check:tokens` | ✅ OK |
| `check:dict` | ✅ OK |
| `check:dict-manifest` | ✅ OK |

Đường cơ sở **62** → **81** test *(**+19**)*: 16 ca `dict_lookup.rs` *(cộng 1 bench `#[ignore]`)* + 3 ca `dict_boundary.rs`.

**Sàn quần thể — KHÔNG nâng cái nào** *(sàn bắt cây **bị cắt**, không bắt việc thêm tệp)*:

| Sàn | Giá trị | Số thật sau story |
|---|---:|---:|
| `store_boundary.rs::RS_FLOOR` | 20 | 28 |
| `scope_boundary.rs::RS_FLOOR` | 20 | 28 |
| `check-i18n.mjs::RS_FLOOR` | 21 | 29 |
| `dict_boundary.rs::DICT_FLOOR` *(mới)* | 1 | 2 |

Khoá trong `src/i18n/vi.json`: **16**, **không đổi** *(Quyết định #4 — story này không phát ra một `MessageKey` mới nào)*.

### Completion Notes List

#### ① 🔵 ĐÍNH CHÍNH `deferred-work.md:279` — kèm SỐ ĐO, không phải suy luận

Mục đó viết *"tra một chữ Hán sẽ nhận về `dictionary`, `lock`, `API`, `Wikipedia`"*. **Mệnh đề đó SAI với truy vấn thuần Hán**, và đây là số đo được trên `dict-core.db` thật:

- `entry_fts MATCH '"中國人"'` ⇒ 33 hàng, giao với `lang='en'` ⇒ **0**. Trigram Latin không khớp trigram Hán.
- `char_idx` thuộc lớp tiếng Anh: **9** cặp trên tổng **1.341.179** *(0,00067%)*.

⇒ Rò rỉ với truy vấn **thuần Hán** đo được là **0**.

**NHƯNG mệnh lệnh lọc `lang` KHÔNG đổi**, vì rò rỉ là **thật và lớn** với truy vấn **Latin** — chuyện thường khi người dùng bôi đen một chữ Latin trong văn bản tiếng Trung: `lock` ⇒ **1 → 0**, `dic` ⇒ **572 → 0** *(100% `lang='en'`)*. Không lọc thì chúng lên giao diện **dán nhãn kết quả tiếng Trung**.

⇒ Cả ba nhánh lọc `lang = 'zh'`. Đính chính đã ghi vào `deferred-work.md`.

#### ② ⚠️ Lệch giữa tài liệu quy hoạch và mã — dev KHÔNG tự sửa, ghi ra cho Ice

1. 🔴 **`ARCHITECTURE-SPINE.md` AD-26 — dải hiệu năng công bố của nhánh 2 đã LỖI THỜI.** AD-26 ghi *0,15–4,5 ms*; đo thật hôm nay trên bản release là **7,324 ms** cho truy vấn 1 ký tự. Số của AD-26 đo ở Giai đoạn 0 trên một database **ba** nguồn; `dict-core.db` nay có **sáu**, và `char_idx` của `山` đi từ ~2.576 lên **3.177** hàng. ⇒ **Dải công bố của AD-26 nên được đo lại**, không nên được trích tiếp như số hiện hành. Xem ④.
2. 🟡 **Story §AC9 mong `EXPLAIN QUERY PLAN` của nhánh 3 *"không thấy `SCAN`"*.** Kế hoạch đúng của SQLite cho một bảng ảo **luôn** chứa từ `SCAN`; phần mang nghĩa là hậu tố `:M1`. Câu chữ của AC nên sửa thành *"phải thấy `VIRTUAL TABLE INDEX … :M<n>`"*. Kế hoạch thật đã dán nguyên văn ở §Debug Log ②.
3. 🟡 **`core/store/mod.rs:119` — doc-comment ghi *"Năm loại kho của AD-7"* trong khi enum có 3 biến thể *(nay là 4)*.** Lệch có **từ trước** story này *(commit `d9bc252` trở về trước)*; story này thêm `Dict` nên khoảng cách còn 1. Không sửa vì nó chạm câu chữ của một quyết định AD-7 mà dev không sở hữu.
4. 🟡 **`cargo fmt --check` ĐỎ trên cây, và nó đỏ TỪ BASELINE.** Ba tệp story này không chạm — `core/i18n/mod.rs:239`, `core/scope/resolve.rs:267`, `core/store/pragmas.rs:96` *(hàm `read_pragma`, mã có sẵn)* — cộng thứ tự `pub use` trong `core/store/mod.rs` vốn đã không theo thứ tự chữ cái. **CI không chạy `cargo fmt` hay `clippy`** *(đã kiểm `.github/workflows/ci.yml`)*, nên đây không phải một cổng đỏ. Mọi tệp **mới** của story đều `fmt` sạch. → Ice quyết: hoặc thêm `cargo fmt --check` thành cổng CI kèm một lượt format cả cây, hoặc ghi rõ rằng repo không dùng rustfmt như một cổng.

#### ③ Trạng thái từng mục bàn giao đang mở ở §References

| Mục | Trạng thái sau 1.11 |
|---|---|
| `deferred-work.md:260` — khoá theo `code`, không theo `id` | 🔵 **ĐÓNG MỘT NỬA.** `EntryHit` mang `source_code: String` và không có trường `source_id` nào; khoá bằng `results_carry_the_source_code_not_the_id` trên fixture hai nguồn. **Nửa còn lại là của 1.13** — lúc gom nhiều tệp, khoá gom **phải** là `code`. |
| `deferred-work.md:266` — VietPhrase 18 đầu mục trùng | 🟡 **VẪN MỞ, và đúng là phải mở.** 1.11 chạy trên **MỘT** tệp mỗi lượt, AD-19 cấm hợp nhất nguồn ⇒ nó không gộp trùng và không được phép gộp. Hậu quả chỉ **quan sát được** khi gom nhiều nguồn ⇒ quyết định của **1.13**. |
| `deferred-work.md:277-280` — lọc `lang` | 🔵 **ĐÓNG cho 1.11** *(ba nhánh đều lọc, có đối chứng âm Latin)*; **vẫn mở cho 1.11b và 1.13**. Kèm một **đính chính** — xem ①. |
| `deferred-work.md:275` — AD mới cho tiếng Anh *(Winston)* | 🔴 **VẪN CHẶN 1.11b.** Story này không chạm, đúng phạm vi. |

#### ④ 🔴 CẦN QUYẾT ĐỊNH CỦA ICE — NFR1 nhánh 2 với truy vấn MỘT ký tự

**Phán quyết AC9: ĐẠT.** p95 của nhánh chậm nhất là **7,324 ms** trên bản **release** — bản người dùng chạy — so với trần **10 ms**. Không có HALT nào theo đúng chữ của AC9 *(trần áp cho bản sản phẩm)*.

**Nhưng ba dữ kiện phải đi cùng con số đó, và cả ba đều là việc của Ice chứ không của story này:**

1. **7,324 ms vượt hẳn dải 0,15–4,5 ms mà AD-26 công bố** — gấp **1,6×** cận trên. Dư địa còn lại tới trần chỉ **27%**, và nó sẽ mỏng đi mỗi lần thêm một nguồn từ điển *(mỗi nguồn mới làm `char_idx` của các ký tự thông dụng dày thêm)*.
2. **Chi phí nằm ở số HÀNG, không ở chỉ mục.** Kế hoạch truy vấn đúng — `SEARCH char_idx USING PRIMARY KEY`, không `SCAN`. Toàn bộ chi phí là **3.177 hàng × 4 chuỗi cấp phát mỗi hàng**. ⇒ nó **không** sửa được bằng một chỉ mục mới, và story này bị cấm thêm chỉ mục — đúng lý do.
3. **Đường ra là một quyết định SẢN PHẨM, không phải một lượt tối ưu.** Giới hạn số hàng trả về *(phân trang, hoặc `LIMIT` + một con số đếm)* là hình dạng tự nhiên nhất — nhưng nó chạm hợp đồng của **Panel Lookup (1.17)** *(hiển thị bao nhiêu kết quả, và nói gì về phần bị cắt)* và của **tầng gom (1.13)** *(cắt trước hay sau khi gom nhiều nguồn)*. Cả hai đều chưa tồn tại. Không tự chọn ở 1.11.

⚠️ **Bản debug VƯỢT trần: 15,045 ms.** Không phải số nghiệm thu, nhưng nó là số mà mọi dev chạy `cargo test` sẽ thấy — ghi ra để lượt sau không đọc nó thành một hồi quy mới.

#### ⑤ Ba quyết định cấu trúc — đã đi đúng đường story chốt, không phát minh lại

1. **Đường mở tệp sống ở `core/store/`** *(Quyết định #1)*. `ReadOnlyDb` + `open_readonly_connection` + `apply_dict_reader_pragmas`. `store_boundary.rs` XANH mà **không nới `STORE_DIR`, không nới `FORBIDDEN`** — cổng giữ đúng **một** miễn trừ. `core/dict/**` không gõ tên crate SQLite ở một vị trí mã nào; nó viết truy vấn qua `ReadHandle` / `SqlResult` / `Row` / **`ToSql`** *(tái xuất MỚI — cần cho `&[&dyn ToSql]`, cùng lý do với bốn kiểu kia)*.
2. **`ReaderPool` dùng lại NGUYÊN, không có pool thứ hai** *(Quyết định #1)*. Thân `open` được bóc thành `open_with(…, open_one, apply_pragmas)`; `open` và `open_readonly` khác **đúng hai hàm**. Không sao chép tệp, không thêm crate pool.
3. **`lookup` nhận `ReadHandle`** *(Quyết định #2)*, nên 1.13 gọi được một lần cho mỗi tệp, và test gọi thẳng trên fixture không cần dựng `ReadOnlyDb`.

#### ⑥ Ranh giới phạm vi — mọi mệnh đề "Không" đều giữ

0 trait trong `ports/mod.rs` · 0 `#[tauri::command]` · 0 dòng frontend · 0 khoá `vi.json` mới *(vẫn **16**)* · 0 dòng sửa trong `tools/dict-build/**` · 0 tệp `.db` được dựng lại · 0 dòng sửa trong `dict-manifest.toml` · 0 crate mới ở bất kỳ cây nào · 0 thay đổi ở `[profile.release]` / `Cargo.toml` · `core/matching/mod.rs` giữ nguyên 7 dòng doc-comment · không sửa `scope_contract.rs` / `scope_boundary.rs` / `ipc_contract.rs` / `config_invariants.rs` — `store_contract.rs` **chỉ** thêm đúng **một** dòng *(`StoreKind::Dict.as_str() == "dict"`)*.

### File List

**Mới**

| Đường dẫn | Vai trò |
|---|---|
| `src-tauri/src/core/store/readonly.rs` | `ReadOnlyDb` — mở/đọc/đóng một tệp `.db` chỉ đọc *(AC7, AC8)* |
| `src-tauri/src/core/dict/query.rs` | Ba nhánh SQL + `verify_substring` *(AC1–AC6)* |
| `src-tauri/tests/dict_lookup.rs` | Fixture + 16 ca hành vi + cổng parity DDL + bench `#[ignore]` *(AC1–AC7, AC9)* |
| `src-tauri/tests/dict_boundary.rs` | 3 ca — cấm `LIKE`/`GLOB`/`instr(` dưới `core/dict/**` *(AC5)* |

**Sửa**

| Đường dẫn | Thay đổi |
|---|---|
| `src-tauri/src/core/store/mod.rs` | `StoreKind::Dict` + `as_str()` · `pub mod readonly` · `pub use ReadOnlyDb` · tái xuất `ToSql` |
| `src-tauri/src/core/store/pragmas.rs` | `open_readonly_connection` · `apply_dict_reader_pragmas` |
| `src-tauri/src/core/store/reader.rs` | Bóc thân dùng chung `open_with` · thêm `ReaderPool::open_readonly` |
| `src-tauri/src/core/dict/mod.rs` | Module thật: `LookupMode` · `QueryBranch` · `EntryHit` · `LookupResult` · `pick_branch` · `lookup` |
| `src-tauri/tests/store_contract.rs` | **MỘT** dòng: `assert_eq!(StoreKind::Dict.as_str(), "dict")` |
| `_bmad-output/implementation-artifacts/deferred-work.md` | Mục `## Deferred from: 1-11-…` — đính chính, trạng thái ba bàn giao, phát hiện NFR1 |
| `_bmad-output/implementation-artifacts/sprint-status.yaml` | `1-11-…: ready-for-dev → in-progress → review` |
| `_bmad-output/implementation-artifacts/1-11-ba-nhanh-truy-van-tieng-trung.md` | Checkbox · Dev Agent Record · File List · Change Log · Status |

### Change Log

| Ngày | Thay đổi |
|---|---|
| 2026-08-05 | Story 1.11 triển khai xong trên nền `5a68df7`. Ba nhánh truy vấn tiếng Trung *(B-tree chính xác · `char_idx` 1–2 ký tự · FTS5 trigram 3+)* với nhánh chọn bằng `chars().count()` và **quan sát được từ giá trị trả về**; xác minh chuỗi con chạy ở Rust cho cả nhánh 2 lẫn nhánh 3; mọi nhánh lọc `lang='zh'`; kết quả mang `source_code` chuỗi. Đường mở tệp chỉ-đọc *(`ReadOnlyDb`)* sống trong `core/store/` nên `store_boundary.rs` XANH mà không nới cổng. **+19 test** *(62 → 81)*, 6/6 cổng XANH. Đo thật trên `dict-core.db`: AC2/AC3/AC4 **khớp tuyệt đối** số nghiệm thu; AC9 p95 nhánh chậm nhất **7,324 ms ≤ 10 ms — ĐẠT** *(release)*. 🔴 Một mục cần Ice quyết: nhánh 2 một-ký-tự vượt dải công bố của AD-26 — xem §Completion Notes ④. |
