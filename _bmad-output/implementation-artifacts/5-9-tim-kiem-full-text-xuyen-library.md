---
title: 'Story 5.9: Tìm kiếm full-text xuyên Library'
type: 'feature'
created: '2026-08-29'
status: 'done'
baseline_revision: 'd48f076cd6ce58d2d6cb6e5bcd48554df4bd7a19'
review_loop_iteration: 0
followup_review_recommended: true
context:
  - '{project-root}/AGENTS.md'
  - '{project-root}/src-tauri/AGENTS.md'
  - '{project-root}/src/AGENTS.md'
  - '{project-root}/tests/AGENTS.md'
  - '{project-root}/e2e/AGENTS.md'
warnings: ['oversized']
deferred:
  - summary: >-
      Truy vấn 1–2 ký tự (đặc biệt từ ghép hai chữ Hán như `天下`/`江湖`) không tra được ở nửa
      nguyên văn — `trigram` không lập chỉ mục token dưới 3 ký tự.
    evidence: |-
      Đo 2026-08-29, SQLite 3.43.2: `library_source_fts MATCH '"天下"'` trả 0 hàng trên chính
      văn bản chứa nó, không lỗi không cảnh báo. `SearchReport::short_query` nói trạng thái đó
      ra màn hình nhưng không đóng vùng câm. Hai phương án đóng (nhánh `char_idx` thứ ba theo
      khuôn AD-26, hoặc không phương án nào khác khả thi với FTS5 chuẩn) ghi đầy đủ ở
      `deferred-work.md`, chủ Ice.
    location: >-
      src-tauri/src/core/library/indexer.rs — search_source_text/MIN_SUBSTRING_QUERY_CHARS
    severity: medium
  - summary: >-
      Chế độ khoan dung dấu tiếng Việt khi tìm kiếm — câu mời đã có trên màn hình
      (`mode.library.search_no_match`), chưa có cửa bấm phía sau nó.
    evidence: |-
      §I/O Matrix + AD-27 cấm dựng chỉ mục `remove_diacritics 2` ở story này; câu báo "không
      khớp" đã nói đúng lý do (phân biệt dấu) và trỏ tới Story 5.10, nơi bảng
      `library_target_fts_nd` + một nút bật/tắt sẽ đóng nó.
    location: 'src/i18n/vi.json — mode.library.search_no_match'
    severity: low
  - summary: >-
      `Indexer::rebuild` thu hoạch TOÀN PHẦN văn bản mỗi lượt quét — chưa có guard tăng dần,
      vì `work.updated_at`/`chapter.updated_at` (món nợ có chủ Story 5.6) chưa đáng tin.
    evidence: |-
      Một guard dựa trên `updated_at` hôm nay sẽ đọc "vừa sửa bản dịch" thành "không đổi",
      làm chỉ mục tìm kiếm giữ chữ CŨ mà không cổng nào đỏ — xem §Design Notes của chính
      story này. Chỉ an toàn thêm SAU khi Story 5.6 đóng món nợ `updated_at`.
    location: 'src-tauri/src/core/library/indexer.rs — Indexer::rebuild'
    severity: low
  - summary: >-
      NFR3 (p95 tìm kiếm) — số đo của story này là SƠ BỘ (fixture 5.000 Chương tổng hợp,
      không qua sản phẩm), không đủ điều kiện đánh dấu đạt.
    evidence: |-
      `epics.md:334` khai ngưỡng NFR3 là tạm `[A6]`; phép đo đủ điều kiện đòi FR14 (Epic 6)
      để có 5.000 Chương thật. Con số sơ bộ ghi ở §Auto Run Result bên dưới, kèm phiên bản
      toolchain và ngày.
    location: '§Auto Run Result — Đo p95 (sơ bộ)'
    severity: low
  - summary: >-
      Không có bước chuẩn hoá Unicode (NFC/NFD) trước khi lập chỉ mục hay trước khi tra — trên
      một chỉ mục PHÂN BIỆT dấu, hai chuỗi trông giống hệt nhau có thể không khớp nhau.
    evidence: |-
      macOS thường sinh tiếng Việt dạng NFD (`a` + U+0301), Windows/web thường NFC (U+00E1);
      FTS5 so trên điểm mã nên hai dạng không khớp và không lỗi nào được ném. Nó còn lệch vị
      từ `short_query`: `chars().count()` đếm ĐIỂM MÃ, nên một từ hai chữ cái dạng NFD đếm ra
      4 và lượt tìm đi nhầm nhánh. Đây là lớp CÓ SẴN của kho — `sense_fts` (Epic 1) mang đúng
      tính chất đó — nên quyết định áp cho cả đường từ điển lẫn đường Library. Chủ Ice.
    location: >-
      src-tauri/src/core/library/indexer.rs — harvest_work_text / Indexer::search
    severity: medium
  - summary: >-
      Trần ứng viên của nhánh `trigram` sắp theo `(work_id, chapter_ord, segment_ord)` chứ
      không theo liên quan — ứng viên dương-tính-giả dồn ở các `work_id` đầu bảng chữ cái có
      thể ăn hết trần trước khi hàng của Tác phẩm sau được đọc.
    evidence: |-
      Biến thể XUYÊN TÁC PHẨM của "Bẫy 11" mà `core/dict/query.rs` đã ghi cho đường từ điển.
      Chưa đo được hôm nay: không đường nào tạo một thư viện đủ lớn để dựng nhiều hơn
      `search_candidate_ceiling(limit)` ứng viên thật. Đo ở Story 6.18 cùng lượt với p95;
      `SearchReport::truncated` (thêm ở lượt rà này) là chỗ báo ra nếu có thật.
    location: 'src-tauri/src/core/library/indexer.rs — search_source_text'
    severity: medium
  - summary: >-
      Mỗi thao tác vòng đời/tổ chức Chương nay mở `project.db` của MỌI Tác phẩm trong thư
      viện, không chỉ Tác phẩm vừa sửa.
    evidence: |-
      `reindex_after_lifecycle_write` và `wire::reindex_library` gọi `Indexer::rebuild` toàn
      bộ, và `rebuild` nay mở-đọc-đóng một `project.db` cho mỗi Tác phẩm. Đo 2026-08-29
      (rustc 1.97.1, macOS): thu hoạch 50.000 segment trong MỘT `.atproj` mất 2.180,9 ms;
      hình dạng thật (hàng trăm `.atproj`) chưa đo. ⚠️ Vế "sau MỖI lượt sửa segment" thì KHÔNG
      đúng — `grep -n "reindex" src-tauri/src/commands/segment.rs` cho 0. Chủ Story 6.18.
    location: >-
      src-tauri/src/commands/lifecycle.rs:225 — reindex_after_lifecycle_write
    severity: medium
  - summary: >-
      Một `INSERT` hỏng khi ghi hàng văn bản của MỘT Tác phẩm làm trượt cả lượt `rebuild` của
      toàn thư viện, trong khi một lượt ĐỌC hỏng thì chỉ bỏ qua đúng Tác phẩm đó.
    evidence: |-
      `harvest_work_text` bắt lỗi theo từng Tác phẩm và dồn vào `RebuildOutcome::text_skipped`,
      nhưng vòng `tx.execute("INSERT INTO library_segment …")?` chạy trong một `store.write`
      chung nên một lỗi ở Tác phẩm thứ ba huỷ luôn phần đã ghi của hai Tác phẩm trước. Bất đối
      xứng đó chưa được viết ra ở đâu và chưa ca nào chạm. Chủ Ice.
    location: 'src-tauri/src/core/library/indexer.rs — Indexer::rebuild'
    severity: low
---

<intent-contract>

## Intent

**Problem:** FR8 hứa tìm một câu từng dịch ở bất kỳ đâu trong cả thư viện — đồng thời trên
nguyên văn và bản dịch — nhưng hôm nay `library-index.db` **không chứa một ký tự văn bản nào**:
`library_work` có đúng 11 cột metadata (`schema.rs:1685-1703`), và `Indexer::rebuild` **chỉ đọc
`meta.json`**, không mở `project.db` lần nào (`indexer.rs:33-34`). Không bảng FTS5 nào tồn tại
trong `src-tauri/src/**` (đo 2026-08-29: `grep -rn "fts5" src-tauri/src` chỉ ra `core/dict/**`,
đường **ĐỌC** trên một kho dựng sẵn bởi `tools/dict-build`). Không `MessageKey`, không khoá
`vi.json`, không lệnh nào cho tìm kiếm.

**Approach:** Một bảng nội dung `library_segment` cộng **hai** chỉ mục FTS5 external-content
trong chính `library-index.db` — một cho nửa bản dịch, một cho nửa nguyên văn — do
`Indexer::rebuild` thu hoạch, giữ nguyên AD-8 (**một đường ghi duy nhất**, không đúc
`index_one`). Một bề mặt tìm kiếm đọc trên `Indexer` chạy **cả hai** chỉ mục mỗi lượt và hợp
kết quả; một khối tìm kiếm ở Library; và một lượt chọn kết quả đi qua đúng đường mở đã có
(`openWorkById` → `openChapterById`) với con trỏ đặt tại segment khớp.

## Boundaries & Constraints

**Always:**
- 🔴 **Hai chỉ mục, hai tokenizer, và cặp đó bị ÉP bởi phép đo — không phải một sở thích.**
  Nửa **bản dịch** dùng `tokenize="unicode61 remove_diacritics 0"` (AD-27); nửa **nguyên văn**
  dùng `tokenize="trigram"`. Mệnh đề `tokenize` viết tường minh ở cả hai: đo 2026-08-29 trên
  SQLite 3.43.2, bỏ `tokenize` rơi về `remove_diacritics 1` **im lặng** — không lỗi, không cảnh
  báo, chỉ sai kết quả (đúng "Bẫy 4" mà `tools/dict-build/src/schema.rs:174-176` đã ghi). Bảng
  số đầy đủ và lý lẽ loại trừ ở §Design Notes.
- 🔴 **Mỗi lượt tìm chạy CẢ HAI chỉ mục rồi hợp kết quả — không một bộ điều phối chọn một
  nhánh.** AC nói *"tìm **đồng thời** trong văn bản nguồn và văn bản dịch"*, và hai chỉ mục có
  hai vùng câm khác nhau: một bộ điều phối chọn sai nhánh sẽ trả 0 hàng trên một kho **có** dữ
  liệu khớp, không lỗi nào ném.
- 🔴 **Nhánh `trigram` PHẢI đi qua một bước xác minh chuỗi con ở Rust.** FTS5 trigram trả lời
  *"chứa các trigram này"*, **không** trả lời *"chứa chuỗi này"* — `core/dict/query.rs:392-394`
  ghi thẳng điều đó và `verify_substring` (`:176-187`) là khuôn: `to_lowercase()` **cả hai vế**
  rồi `contains`. ⚠️ Bỏ phép hạ chữ thường ở vế xác minh làm chính bước xác minh **vứt đi**
  những hàng trigram vừa tìm đúng (trigram không phân biệt hoa/thường — đo 2026-08-29: `"BROWN"`
  khớp `the quick brown fox`), tức một hàng rào chống dương-tính-giả biến thành một cỗ máy sinh
  âm-tính-giả.
- 🔴 **Truy vấn dưới 3 ký tự là một trạng thái CÓ TÊN, không phải "không có kết quả".** Đo
  2026-08-29: FTS5 `trigram` không lập chỉ mục token dưới 3 ký tự ⇒ `"天下"` trả **0** hàng
  trên chính văn bản chứa nó. Ở dải đó chỉ nửa bản dịch còn trả lời, và chỉ khớp **trọn từ**.
  Cùng khuôn *"chuỗi con 1–2 ký tự khai là KHÔNG HỖ TRỢ và trả một trạng thái phân biệt được"*
  mà AD-26 đã dựng cho đường từ điển.
- 🔴 **AD-8 giữ nguyên: `Indexer::rebuild` vẫn là đường ghi DUY NHẤT vào `library-index.db`.**
  Không thêm `index_one`, không đường ghi thứ hai chạy song song — lý lẽ đầy đủ ở
  `indexer.rs:59-70`. Thu hoạch văn bản đi vào **chính giao dịch** của `rebuild`.
- 🔴 **Thu hoạch mở `project.db` của một `.atproj` khác CHỈ ĐỌC, và không bao giờ di trú nó.**
  `Store::open` mở `READ_WRITE | CREATE`, đặt `journal_mode = WAL`, **chạy bộ di trú** và dựng
  luồng writer (`readonly.rs:19-24` liệt kê đúng bốn thứ đó) — dùng nó ở đây là ghi vào một Tác
  phẩm mà lượt quét không sở hữu, và còn **di trú hàng loạt** cả thư viện chỉ vì người dùng mở
  Library. Đường đúng là `ReadOnlyDb`, nới danh sách `StoreKind` cho phép từ `{Dict}` thành
  `{Dict, Project}` bằng một **miễn trừ CÓ TÊN** kèm lý do tại chỗ.
- 🔴 **Một `project.db` ở phiên bản lược đồ MỚI HƠN ứng dụng ⇒ BỎ QUA phần văn bản của Tác phẩm
  đó và BÁO RA, không im lặng** (AD-30 cấm ghi vào nó). Một lượt thu hoạch trượt không được đọc
  lên giống *"Tác phẩm này không có chữ nào"*.
- 🔴 **Một danh sách kết quả rỗng phải nói VÌ SAO nó rỗng.** Năm ca **phân biệt được**, không
  gộp: ① chưa gõ gì · ② đang tìm · ③ chỉ mục chưa có dòng nào (`indexed_segments == 0`) ·
  ④ truy vấn dưới 3 ký tự (nửa nguyên văn câm) · ⑤ chỉ mục có `N` dòng mà không khớp. Ca ⑤ nêu
  đúng lý do (chỉ mục **phân biệt dấu**) và nói chế độ khoan dung là việc của Story 5.10 —
  **không** dựng một nút chưa có đường chạy phía sau.
- **Truy vấn của người dùng đi vào FTS5 dạng CỤM CÓ NGOẶC KÉP, dấu `"` bên trong nhân đôi**, ở
  **cả hai** nhánh — chép nguyên cách của `core/dict/query.rs:310`, không có "biến thể" thứ hai.
  Không bọc thì `*` `-` `^` `(` `:` hay từ `NEAR` làm SQLite trả `SQLITE_ERROR`, tức **tìm kiếm
  báo lỗi vì chữ người dùng gõ**.
- **Khuôn hai lớp cho bề mặt IPC** (`src-tauri/AGENTS.md:11`): hàm thuần nhận
  `Option<&Indexer>` + vỏ `#[tauri::command]` mỏng trong `mod wire` lấy `try_state`. Vỏ mang
  `(async)` vì nó chạm đĩa — `config_invariants.rs::the_blocking_wires_run_off_the_main_thread`.
- **`commands/library.rs` KHÔNG được nhắc `StoreKind::LibraryIndex`** (`library_index_boundary.rs`);
  mọi SQL của chỉ mục sống trong `core/library/indexer.rs`, lệnh chỉ gọi xuống.
- **Kết quả trả về là DỮ LIỆU, render bằng Vue** — không `v-html`, không chuỗi mang thẻ đánh
  dấu (AD-16). `snippet()` dùng cặp dấu **văn bản thuần**.
- **Lượt mở một kết quả đi qua đúng một đường dời con trỏ** (`editorPanelState.ts::doiConTroToi`,
  `:1367`) — không dựng bản thứ hai.

**Block If:**
- Một AC nào của §Story 5.9 chỉ nghiệm thu được bằng cách **đổi một bất biến đã ADOPTED**
  (AD-8 · AD-27 · AD-30) ⇒ HALT `blocked`: đó là một `AD` mới, và AD mới không do dev soạn.

**Never:**
- **Không** dựng chỉ mục xoá dấu (`_nd`, `remove_diacritics 2`) — đó là chế độ khoan dung của
  Story 5.10, và AD-27 cấm nó làm mặc định.
- **Không** đổi ngưỡng NFR3 và **không** khai NFR3 là đạt: `epics.md:334` ghi rõ đây là ngưỡng
  **tạm** `[A6]`, và phép đo đủ điều kiện là Story 6.18 (cần FR14 của Epic 6 để có 5.000 Chương
  thật). Story này **đo và ghi lại** một con số truy nguyên được, không hơn.
- **Không** đường tìm kiếm nào đọc thẳng `.atproj` hay `meta.json` lúc chạy (AD-8/AD-33).
- **Không** thêm phụ thuộc mới (cửa NFR15 không mở cho gói nào ở story này).

## I/O & Edge-Case Matrix

| Scenario | Input / State | Expected Output / Behavior | Error Handling |
|----------|--------------|---------------------------|----------------|
| Khớp ở bản dịch | segment `target_text = 'má của tôi rất hiền'`; truy vấn `má` | 1 hit kèm `work_id`, `work_name`, `chapter_id`, `chapter_ord`, `chapter_title`, `segment_id`, `field = "target"`, `snippet` chứa `má` | No error expected |
| Khớp ở nguyên văn, chữ Hán | `source_text = '天下大势，分久必合'`; truy vấn `分久必合` | 1 hit `field = "source"` | No error expected |
| Khớp ở nguyên văn, chuỗi con Latin | `source_text = 'the quick brown fox'`; truy vấn `uick bro` | 1 hit `field = "source"` | No error expected |
| Phân biệt dấu | sáu segment `má`, `ma`, `mà`, `mả`, `mã`, `mạ` ở `target_text`; truy vấn `má` | **đúng 1** hit — hàng `má`; năm hàng kia **không** ra | No error expected |
| Đồng thời hai nửa | một segment khớp ở `source_text`, một segment khác khớp ở `target_text`, cùng truy vấn | **cả hai** hit, mỗi hit mang `field` của chính nó | No error expected |
| Xuyên Tác phẩm | hai `.atproj` cùng chứa chuỗi khớp | hit của **cả hai**, mỗi hit mang `work_id` của chính nó | No error expected |
| Chương chưa tách segment | Chương có `chapter.source_text` và **0** hàng `segment` sống | một hit cấp Chương: `segment_id = null`, `field = "source"` — Chương vẫn tìm được | No error expected |
| Dương tính giả của trigram | truy vấn `abc` trên `source_text = 'axbxc ... abq'` (đủ trigram, không đủ chuỗi) | **0** hit — bước xác minh chuỗi con loại nó | No error expected |
| Trigram không phân biệt hoa/thường | `source_text = 'the quick BROWN fox'`; truy vấn `brown` | 1 hit — bước xác minh hạ chữ thường **cả hai vế** trước khi so | No error expected |
| Truy vấn dưới 3 ký tự | truy vấn `天下` trên kho chứa `天下大势…` | `short_query = true`; nửa nguyên văn **không trả lời được**, và giao diện nói ra điều đó thay vì *"không có kết quả"* | No error expected |
| Chỉ mục rỗng | `library_segment` có 0 hàng; truy vấn bất kỳ | `hits = []`, `total = 0`, **`indexed_segments = 0`** ⇒ giao diện nói *chỉ mục chưa có gì*, khác hẳn *không khớp* | No error expected |
| Không khớp | `indexed_segments = N > 0`, truy vấn ≥ 3 ký tự | `hits = []`, `total = 0`, `indexed_segments = N` ⇒ giao diện nêu lý do (phân biệt dấu) và trỏ tới Story 5.10 | No error expected |
| Ký tự cú pháp FTS5 | truy vấn `state-of-the-art`, `a"b`, `NEAR`, `*` | chạy bình thường trên **cả hai** nhánh, **0** lỗi SQLite | No error expected |
| Truy vấn rỗng / chỉ khoảng trắng | ô tìm kiếm rỗng sau `trim()` | **0 lượt IPC**; giao diện ở trạng thái *chưa gõ gì* | No error expected |
| Chưa mở chỉ mục | `try_state::<Indexer>()` trả `None` | 0 truy vấn SQL | `indexer_is_missing()` (tái dùng `StoreOpenFailed` `{store: "library_index"}`) |
| Thu hoạch: `project.db` mới hơn | một `.atproj` mang lược đồ v18 khi ứng dụng biết v17 | hàng `library_work` của nó **vẫn được UPSERT** từ `meta.json`; phần văn bản bị bỏ qua và **đếm vào `RebuildOutcome`** | chẩn đoán nêu đích danh `work_id`; `rebuild` **không** trượt cả lượt |
| Thu hoạch: `project.db` vắng mặt | thư mục `.atproj` có `meta.json`, thiếu `project.db` | như trên — metadata vào chỉ mục, văn bản bỏ qua **có đếm** | chẩn đoán nêu đích danh |
| Thu hoạch: chữ vừa ghi còn trong WAL | Tác phẩm vừa flush xong, `wal_autocheckpoint = 0` nên `-wal` chưa gộp | chỉ mục chứa **chữ mới nhất** | No error expected |
| Xoá chỉ mục rồi mở lại | người dùng xoá `library-index.db` | lượt quét kế tiếp dựng lại **cả** metadata **và** toàn bộ văn bản; kết quả tìm kiếm giống hệt trước khi xoá | No error expected |
| Lệch phiên bản lược đồ | `library-index.db` ở `to_version` 1..5 | `Indexer::open` xoá tệp + sidecar rồi dựng lại ở 6 (AD-8, nhánh KHÔNG-DI-TRÚ) | No error expected |
| Mở một kết quả | người dùng chọn hit của Tác phẩm B trong khi đang mở Tác phẩm A | flush Editor → `openWorkById(B)` → `openChapterById(chapter_id, segment_id)` → Workspace, con trỏ ở đúng câu khớp | flush trả `'failed'`/`'still-dirty'` ⇒ **CHẶN** kèm câu báo, không lượt mở nào chạy |
| Mở một kết quả cấp Chương | hit có `segment_id = null` | mở đúng Chương; con trỏ để **Rust quyết** như mọi lượt mở Chương bình thường | No error expected |
| Mở một kết quả đã cũ | `segment_id` của hit không còn trong Chương (đã về hưu, chỉ mục chưa quét lại) | mở đúng Chương, con trỏ để Rust quyết, **và ghi chẩn đoán nêu đích danh** | không lượt mở nào bị huỷ vì chuyện này |

</intent-contract>

## Code Map

**Rust — lược đồ và kho**

- `src-tauri/src/core/store/schema.rs` — `LIBRARY_WORK_DDL` (`:1685-1703`, 11 cột hôm nay) và
  `LIBRARY_INDEX_MIGRATIONS` (`:1734-1737`, **đúng một bước, `to_version: 5`**). 🔴 Kho dẫn xuất
  **viết lại DDL TẠI CHỖ** và bump `to_version` 5 → 6 — **không** thêm bước thứ hai; khối
  doc-comment `:1705-1733` là khuôn ghi lý do cho từng lần bump, chép đúng hình dạng đó.
- `tools/dict-build/src/schema.rs:166-188` — **khuôn FTS5 duy nhất trong kho**: `ENTRY_FTS_DDL`
  (`trigram`), `SENSE_FTS_DDL` (`unicode61 remove_diacritics 0`), `SENSE_FTS_ND_DDL`
  (`remove_diacritics 2`, hậu tố `_nd` *"nói rõ đây KHÔNG phải mặc định"*). Chép **hình dạng và
  cách đặt tên**, không import — hai workspace tách rời.
- `src-tauri/src/core/store/readonly.rs` — `ReadOnlyDb::open` (`:63`) mang
  `debug_assert_eq!(kind, StoreKind::Dict)` (`:64-68`). Doc-comment `:1-29` chở hai lý lẽ phải
  đọc **trước** khi nới: vì sao đường mở tệp ở lại `core/store/**` (cổng
  `store_boundary.rs::only_core_store_may_name_rusqlite`) và vì sao **không** tái dùng `Store`.
- `src-tauri/src/core/store/mod.rs:303-350` — `StoreSpec::global/project/library_index`;
  `:341-349` ghi 🔴 *"chỉ `Indexer::open` được gọi `Store::open(StoreSpec::library_index(..))`"*.

**Rust — Indexer và bề mặt IPC**

- `src-tauri/src/core/library/indexer.rs` (1116 dòng) — `open` (`:155`, nhánh không-di-trú qua
  `delete_if_schema_version_differs` `:156-160`), `rebuild` (`:210`, **đường ghi duy nhất**, đọc
  `WorkMeta::read` ở `:272`, chạy dưới `rebuild_lock` `:143`), `list_works` (`:523`), `find_work`
  (`:595`), `list_orphans` (`:617`), `forget_orphan` (`:641`). 🔵 Doc-comment `:33-34` khai *"đọc
  **chỉ** `meta.json` (AD-9: không mở `project.db` lần nào)"* — mệnh đề đó **hết đúng** khi story
  này chạy: sửa TẠI CHỖ kèm 🔵 + ngày. `:59-70` là lý lẽ *"vì sao một đường ghi duy nhất"* — đọc
  trước khi định tách một đường thu hoạch riêng.
- `src-tauri/src/commands/library.rs` — `list_works` (`:354`, khuôn hàm thuần `Option<&Indexer>`
  + `indexer_is_missing()` ở `:361`), `unknown_sort`, `library_rescan` (`:161`), vỏ
  `library_list_works` (`:505-521`) là **khuôn chép cho vỏ mới**.
- `src-tauri/src/core/i18n/mod.rs` — `message_keys!` (`:100+`); cụm Library `:395-449`
  (`LibraryNotOrphaned`, `LibraryRootInvalid`, `LibraryUnknownSort`, `LibraryWorkNotIndexed`)
  cộng khối chú thích `:395-400` khai *"danh mục ĐÓNG, ca X tái dùng khoá Y"*.
- `src-tauri/src/lib.rs:324+` — `generate_handler![…]`, lệnh Library ở `:331-334`.
- `src-tauri/src/commands/project.rs:491-507` — `read_chapter_segment_texts`: khuôn câu SQL đọc
  segment **sống** (`WHERE chapter_id = ?1 AND retired_at IS NULL ORDER BY ord`). ⚠️ Nó chỉ đọc
  `source_text`, và mọi chỗ gọi (`:659`, `:1275`, `:1353`) truyền `Store` của **Tác phẩm đang
  mở** — không phải đường dùng lại được cho thu hoạch xuyên Tác phẩm.
- `src-tauri/src/core/dict/query.rs` — 🔴 **khuôn bọc cụm FTS5** (`format!("\"{}\"",
  query.replace('"', "\"\""))`, `:310`) kèm lý do `:299-308`; `verify_substring` (`:176-187`,
  `to_lowercase()` hai vế); `fts_trigram` (`:309`) và `fts_trigram_en` (`:395`) — hai chỗ dùng
  cùng một phép bọc, **không** có bản thứ hai; `candidate_ceiling` là **trần AN TOÀN cho bộ
  nhớ**, không phải một phép cắt thời gian.

**Frontend**

- `src/config/library.ts` — adapter Library hiện có; nhận adapter tìm kiếm mới. Khuôn ba trạng
  thái chép từ `src/config/chapter.ts:161-183` (`readOpenChapter`) cùng `isIpcError` (`:33`),
  `hasIpcBridge` (`:93`), `UNKNOWN_IPC_ERROR` (`:144-154`). ⚠️ `invoke()` gửi **camelCase**,
  trường TRẢ VỀ giữ **snake_case**.
- `src/modes/libraryWorks.ts` — state cấp module (`works` `:42`, `worksHaveLoaded` `:62`,
  `worksBusy` `:63`, `worksReloadPending` `:66`, `worksSequence` `:77`), `resetLibraryWorks()`
  `:355`. **Khuôn chống đua** `*ReloadPending` + `*Sequence` ở `:140-176` — chép cho ô tìm kiếm,
  nơi mỗi lượt gõ có thể phát một lượt mới.
- `src/modes/libraryChapters.ts` — `openWorkById` (`:198-283`): 🔴 khuôn **flush → IPC → vứt
  state → nạp lại → nhả cờ bận SAU CÙNG**; `setMode('workspace')` ở `:301`. Khối `:234-252` là
  🔴 *"CỬA SỔ MỞ NHẦM CHƯƠNG"* — `chapter_id` chỉ AUTOINCREMENT **cục bộ theo từng
  `project.db`**, nên một `chapter_id` của Tác phẩm B áp lên Tác phẩm A mở đúng một Chương SAI
  mà không lỗi nào ném. Đây là lỗ mà đường "chọn kết quả tìm kiếm" đi thẳng vào.
- `src/panels/editorPanelState.ts` — `ensureSegmentsLoaded` (`:130-163`, đặt
  `caretPlacement.value = loaded?.caret_segment_id` ở `:157`), `setEditorCaret` (`:190`),
  `doiConTroToi` (`:1367`, 🔴 **đường dời con trỏ có ĐÚNG MỘT bản**; `:1362-1363` nói một đường
  thứ hai làm mất chữ vừa gõ, im lặng), `openChapterById` (`:1753`),
  `flushEditorBeforeDiscreteWrite` (`:562`), `flushChapterPositionNow` (`:281`),
  `resetEditorPanel` (`:592`), `caretPlacement` (`:887`) + `editorCaretPlacement` (`:889`).
- `src/panels/GridPanel.vue:1110-1129` — watcher `editorCaretPlacement`:
  `querySelector('[data-segment-id="…"]')` rồi `target.focus()` (`:1126`); `focus()` **tự cuộn**,
  không `scrollIntoView` thứ hai (đã đo ba lượt, ghi ở `:1119-1125`). Đây là cơ chế "cuộn tới
  đúng câu" mà story này tái dùng nguyên.
- `src/modes/LibraryMode.vue` (1577 dòng) — `<template>` `:281`, khối gốc `:290-446`, khối lưới
  Tác phẩm `:447-771`, khối Chương `:704-978`, form nhập `:979-1068`, `<style scoped>` `:1072`.
  🔴 **Khuôn `role="status"` nhiều nhánh phân biệt được** ở `:572-581` — chép đúng hình dạng đó.
- `src/commands/index.ts` — khuôn đăng ký lệnh Library có phím `library.rescan` (`:967-975`) và
  không phím `library.choose_root` (`:976-985`); `CommandDeps` cụm Library `:199-320`. Đo
  2026-08-29: **0** lệnh và **0** hợp âm tìm kiếm tồn tại trong tệp này.
- `src/main.ts:356` — `installCommands({…})`, cụm Library `:366-394` (một khối một Story).
- `src/i18n/vi.json` — `mode.library.*` (79 khoá), `command.library.*` (24 khoá). Đo 2026-08-29:
  **0** khoá tìm kiếm Library (`glossary.manage.search_label` `:430` là ô tìm của **Glossary**,
  không liên quan). Phẳng, khoá chấm, không giá trị rỗng, placeholder khớp `[a-z_][a-z0-9_]*`.

**Cổng và bàn đo**

- `src-tauri/tests/library_index_contract.rs` — **tệp nhận bộ ca chính**. Khuôn gần nhất:
  `rebuilding_from_disk_indexes_exactly_n_works_matching_meta_json_field_for_field` (`:152`),
  `an_index_file_at_schema_version_4_is_deleted_and_rebuilt_at_version_5_with_the_new_column`
  *(🔵 2026-08-29: ca đó nay đổi tên thành `…_at_version_6_with_the_new_columns` — đích đã nâng 5 → 6, và một cái tên khai con số CŨ là một cái tên khai khác thứ nó đo.)*
  (`:355` — **khuôn trực tiếp cho ca 5 → 6**), `a_root_that_existed_with_rows_then_vanishes_…`
  (`:587`).
- `src-tauri/tests/library_index_boundary.rs` — `only_the_named_two_files_may_name_the_library_index_store`
  (`:117`), `the_forbidden_needles_appear_in_exactly_the_two_exempt_files_and_nowhere_else`
  (`:284`). 🔴 Story này **không được** làm danh sách hai tệp đó dài ra.
- `src-tauri/tests/store_boundary.rs` — `only_core_store_may_name_rusqlite`: SQL thu hoạch phải
  đi qua `ReadHandle`/`Transaction`, không gõ tên crate SQLite ngoài `core/store/**`.
- `src-tauri/tests/ipc_contract.rs:472-535` — đóng băng khoá `snake_case` của `WorkRow`/
  `WorkListReport`; **mọi struct mới trên dây phải có ca ở đây**.
- `src-tauri/tests/config_invariants.rs::the_blocking_wires_run_off_the_main_thread` — đếm số vỏ
  `(async)` **theo từng tệp**; thêm một vỏ là sửa con số của `commands/library.rs`.
- `src-tauri/tests/meta_write_boundary.rs` — story này **không** thêm chỗ đọc/ghi `meta.json`
  ⇒ cổng phải giữ xanh **không đổi con số**; nó đỏ tức lượt cài đã đi sai đường.
- `tests/frontend/libraryWorks.test.ts` · `libraryChapters.test.ts` — khuôn `invoke` giả
  (`vi.mock('@tauri-apps/api/core', …)`, `libraryChapters.test.ts:12-13`, reset `:53`) và mount
  `LibraryMode.vue` (`:203` trở đi).
- `e2e/specs/story-5-8-reorganise-chapters.e2e.mjs` — helper `createWork(name, text)` (`:57`),
  `realClick` từ `../support/pointer.mjs` (`:37`). 🔴 Cấm `.click()` của driver.
  ⚠️ `story-5-6-library-grid.e2e.mjs` **đỏ từ baseline** (chủ Story 5.6) — không đọc thành hồi quy.
- `scripts/check-commands.mjs` — `COMMAND_FLOOR` 52 (`:319`) · `CLICK_FLOOR` 27 (`:352`) ·
  `DISPATCH_FLOOR` 40 (`:363`), đều là **cận DƯỚI**. Kiểm A: mỗi `@click` là đúng một
  `dispatch('<id>')` id literal.
- `scripts/check-panel-refs.mjs` — mọi ô nhớ cấp module phải có một đường `reset*()`, mỗi khai
  báo state nằm **trên một dòng** (`FILE_FLOOR` 39, `:555`). `scripts/check-i18n.mjs` —
  `RS_FLOOR` 44 (`:288`), Kiểm A cấm chữ tiếng Việt **có dấu** ở vị trí mã trong `src-tauri/src/**`.

## Tasks & Acceptance

**Execution:**

1. `src-tauri/src/core/store/schema.rs` — viết lại `LIBRARY_WORK_DDL` **TẠI CHỖ**: thêm bảng nội
   dung `library_segment` (`work_id`, `chapter_id`, `chapter_ord`, `chapter_title`, `segment_id`
   **NULL được** cho hàng cấp Chương, `segment_ord`, `source_text`, `target_text`) cộng **hai**
   chỉ mục external-content — `library_target_fts(target_text, content='library_segment',
   content_rowid='rowid', tokenize="unicode61 remove_diacritics 0")` và
   `library_source_fts(source_text, content='library_segment', content_rowid='rowid',
   tokenize="trigram")` — rồi bump `LIBRARY_INDEX_MIGRATIONS` `to_version` 5 → 6 kèm một khối 🔵
   đúng khuôn bốn lần bump trước. -- Rationale: kho dẫn xuất không di trú (AD-8); `tokenize`
   tường minh vì thiếu nó rơi về `remove_diacritics 1` im lặng.
2. `src-tauri/src/core/store/readonly.rs` — đổi `debug_assert_eq!(kind, StoreKind::Dict)` thành
   một danh sách cho phép **có tên** `{Dict, Project}`, kèm doc-comment nói **vì sao** `Project`
   được thêm (thu hoạch đọc một `.atproj` mà lượt quét không sở hữu) và **vì sao** `Store::open`
   sai đường ở đây (bốn thứ nó ghi vào tệp, `:19-24`). -- Rationale: một miễn trừ phải CÓ TÊN,
   có lý do tại chỗ, và phải chết được.
3. `src-tauri/src/core/library/indexer.rs` — ① thu hoạch trong `rebuild`: với mỗi `.atproj` đọc
   được `meta.json`, mở `project.db` **chỉ đọc**, đọc Chương cùng segment **sống**, ghi
   `library_segment` trong **cùng giao dịch** đã có, rồi `INSERT INTO
   library_target_fts(library_target_fts) VALUES('rebuild')` và tương tự cho
   `library_source_fts`; ② `Indexer::search(query, limit) -> SearchReport` chạy **cả hai** nhánh
   và hợp kết quả, nhánh `trigram` đi qua bước xác minh chuỗi con; ③ sửa TẠI CHỖ doc-comment
   `:33-34` kèm 🔵 + ngày; ④ đếm số Tác phẩm bị bỏ qua phần văn bản vào `RebuildOutcome`.
   -- Rationale: AD-8 giữ một đường ghi; một lượt thu hoạch trượt phải đếm được, không im lặng.
4. `src-tauri/src/core/i18n/mod.rs` — **không đúc khoá mới**; thêm vào khối chú thích cụm Library
   (`:395-400`) một mệnh đề khai danh mục ĐÓNG: ca *"chưa mở chỉ mục"* của tìm kiếm tái dùng
   `indexer_is_missing()`; các ca *rỗng*, *dưới 3 ký tự*, *chỉ mục trống* là **trạng thái trong
   báo cáo**, không phải lỗi. -- Rationale: một khoá trùng nghĩa là hai chuỗi phải giữ khớp bằng
   kỷ luật.
5. `src-tauri/src/commands/library.rs` — hàm thuần `search_library(indexer: Option<&Indexer>,
   query: &str, limit: Option<u32>) -> Result<SearchReport, IpcError>` + vỏ
   `#[tauri::command(async)] library_search` trong `mod wire` dùng `try_state`. -- Rationale:
   khuôn hai lớp; `commands/` không được nhắc `StoreKind::LibraryIndex`.
6. `src-tauri/src/lib.rs` — thêm vỏ mới vào `generate_handler!`. -- Rationale: một vỏ ngoài danh
   sách là một lệnh không tồn tại trên dây, và frontend chỉ thấy nó lúc chạy.
7. `src/config/library.ts` — adapter `searchLibrary(query, limit)` theo khuôn ba trạng thái, kèm
   type guard **LÚC CHẠY** cho hình dạng `SearchReport`. -- Rationale: adapter không bao giờ ném;
   `IpcError` phía TS là một lời khai, không một bảo đảm của trình biên dịch.
8. `src/modes/librarySearch.ts` — **tệp state mới**: `searchQuery`, `searchHits`,
   `searchHasLoaded`, `searchBusy`, `searchError`, `searchTotal`, `searchIndexedSegments`,
   `searchShortQuery`, `searchCursor`, cơ chế `searchReloadPending` + `searchSequence`, và
   `resetLibrarySearch()`. -- Rationale: `check:panel-refs` đòi mọi ô nhớ cấp module có một
   đường `reset*()`, mỗi khai báo trên một dòng.
9. `src/modes/LibraryMode.vue` — khối `.search-block`: ô nhập `v-model`, nút tìm
   (`@click="dispatch('library.search')"`), danh sách kết quả (mỗi hàng nêu Tác phẩm · Chương ·
   `field` · đoạn khớp), và một `role="status"` **năm nhánh**; `<style scoped>` chỉ dùng token.
   -- Rationale: `@click` phải là đúng một `dispatch('<id>')`; một danh sách rỗng phải nói vì sao
   nó rỗng.
10. `src/panels/editorPanelState.ts` — `openChapterById(targetChapterId, targetSegmentId?)`: sau
    `ensureSegmentsLoaded()`, nếu có `targetSegmentId` **và** nó có trong `segments` thì đi qua
    đúng `doiConTroToi` để đặt con trỏ; không tìm thấy ⇒ giữ con trỏ Rust quyết **và ghi chẩn
    đoán nêu đích danh**. -- Rationale: đường dời con trỏ có đúng một bản; một segment đã về hưu
    không được lặng lẽ mở sai chỗ.
11. `src/modes/librarySearch.ts` — `openSearchHit(hit)`: `openWorkById(hit.work_id)` (đã chở
    flush + reset + `setMode('workspace')`), **chờ nó xong**, rồi `openChapterById(hit.chapter_id,
    hit.segment_id ?? undefined)`. 🔴 Không bao giờ phát lượt mở Chương trước khi lượt mở Tác
    phẩm đã xong — khối `libraryChapters.ts:234-252` giải thích cửa sổ đó. -- Rationale:
    `chapter_id` là số cục bộ theo từng `project.db`.
12. `src/commands/index.ts` + `src/main.ts` — lệnh `library.search` và `library.open_search_hit`
    (id theo văn phạm khoá chấm), dep tiêm ở một khối mới của `installCommands`. -- Rationale:
    nút và phím tắt phát **cùng một** `dispatch(...)`; một lời gọi thẳng dựng đường thứ hai mà
    `check:commands` Kiểm A không nhìn thấy.
13. `src/i18n/vi.json` — cụm `mode.library.search_*` (nhãn ô, nút, **năm** câu trạng thái, nhãn
    `field` nguồn/dịch) và `command.library.search` · `command.library.open_search_hit`. Phẳng,
    không giá trị rỗng. -- Rationale: NFR16, và `check:i18n` đồng bộ hai chiều.
14. `src-tauri/tests/library_index_contract.rs` — bộ ca cho **mọi hàng** của §I/O Matrix thuộc
    tầng Rust: thu hoạch, phân biệt dấu (`má` ra 1, năm biến thể ra 0), chuỗi con Hán và Latin,
    đồng thời hai nửa, xuyên Tác phẩm, Chương chưa tách segment, dương tính giả trigram,
    hoa/thường, dưới 3 ký tự, chỉ mục rỗng vs không khớp, ký tự cú pháp FTS5, `project.db` mới
    hơn / vắng mặt, chữ còn trong WAL, xoá-và-dựng-lại, bump 5 → 6. Tên hàm là **câu khẳng
    định**. -- Rationale: hợp đồng và bất biến kho thuộc đường Rust, không thuộc vitest.
15. `src-tauri/tests/ipc_contract.rs` · `config_invariants.rs` — khoá `snake_case` của
    `SearchHit`/`SearchReport`; số vỏ `(async)` của `commands/library.rs`. -- Rationale: bốn tên
    trường là **dây**.
16. `tests/frontend/librarySearch.test.ts` — ca cho: năm trạng thái rỗng phân biệt được, truy vấn
    rỗng **không** phát IPC, lượt gõ nhanh không cho kết quả cũ ghi đè kết quả mới
    (`searchSequence`), và `openSearchHit` gọi đúng **thứ tự** hai lượt mở. -- Rationale: hành vi
    module thuần và `.vue` thuộc vitest.
17. `e2e/specs/story-5-9-library-search.e2e.mjs` — trong WKWebView thật: tạo hai Tác phẩm, gõ
    truy vấn, `realClick` một kết quả, khẳng định Workspace mở đúng Chương của đúng Tác phẩm và
    con trỏ ở đúng câu. -- Rationale: hành vi trong engine thật không có chủ ở ba đường kia.
18. `_bmad-output/implementation-artifacts/deferred-work.md` — mục nợ **có chủ** cho: ① truy vấn
    1–2 ký tự (đặc biệt từ ghép chữ Hán hai chữ) không tra được nửa nguyên văn — hai phương án
    kèm số đo, chủ **Ice**; ② câu mời chế độ khoan dung chưa có cửa bấm, chủ **Story 5.10**;
    ③ thu hoạch toàn phần mỗi lượt `rebuild` vì món nợ `work.updated_at` (chủ **Story 5.6**) làm
    một guard tăng dần **sai im lặng**; ④ p95 đo trên kho tổng hợp, phép đo đủ điều kiện là
    **Story 6.18**. -- Rationale: không mục nào mồ côi; không đánh dấu đạt bằng suy luận.
19. `src-tauri/AGENTS.md` — sửa TẠI CHỖ kèm 🔵 + ngày mệnh đề *"Indexer chỉ đọc `meta.json`"*, và
    ghi ra hệ quả: một đường ghi mới vào bảng `segment` không đi qua lượt quét lại sẽ làm chỉ mục
    tìm kiếm nói dối trong im lặng. -- Rationale: mệnh đề hết đúng thì sửa tại chỗ, đừng để nó
    lặng lẽ sai.

**Acceptance Criteria:**

- Given một `library-index.db` vừa bị xoá, when ứng dụng mở lại và quét, then mọi kết quả tìm
  kiếm trả về **giống hệt** trước lượt xoá — không một byte dữ liệu người dùng nào phải có mặt ở
  đâu khác ngoài các `.atproj`.
- Given toàn bộ cây nguồn, when rà, then **chỉ** `Indexer::rebuild` ghi vào `library_segment` và
  hai bảng FTS, và `library_index_boundary.rs` vẫn khai **đúng hai** tệp được nhắc kho này.
- Given một lượt tìm kiếm, when thực hiện, then **0** truy vấn nào chạm `.atproj` hoặc
  `meta.json` — mọi hàng đến từ `library-index.db`.
- Given năm ca rỗng (chưa gõ · đang tìm · chỉ mục trống · dưới 3 ký tự · không khớp), when hiển
  thị, then **năm câu khác nhau**, và ca *không khớp* nói ra rằng chỉ mục phân biệt dấu.
- Given một kết quả của một Tác phẩm **khác** Tác phẩm đang mở, when chọn, then Workspace hiện
  đúng Chương của đúng Tác phẩm đó và con trỏ ở đúng câu khớp — không lượt nào mở nhầm Chương vì
  `chapter_id` trùng số giữa hai `project.db`.
- Given bộ e2e của story, when chạy trong WKWebView thật, then xanh; và
  `story-5-6-library-grid.e2e.mjs` đỏ từ baseline **không** đọc thành hồi quy của story này.
- Given một phép đo p95, when ghi lại, then nó mang **phiên bản toolchain và ngày**, và nó được
  khai là **sơ bộ** — NFR3 **không** được đánh dấu đạt ở story này.

## Spec Change Log

- **Task 11 — `openWorkById(hit.work_id)` "đã chở … `setMode('workspace')`".** Đo lại
  `src/modes/libraryChapters.ts:198-283`: `setMode('workspace')` sống trong `openCurrentChapter()`
  (hàm KẾ TIẾP `openWorkById` trong cùng tệp), không PHẢI bên trong `openWorkById` — code map của
  chính story này ("`setMode('workspace')` ở :301") trỏ đúng vị trí đó, chỉ câu văn ở Task 11 tóm
  tắt gộp nhầm. `openCurrentLibrarySearchHit` (`src/modes/librarySearch.ts`) vì thế tự gọi
  `setMode('workspace')` SAU `openChapterById` thành công — đúng khuôn `openCurrentChapter`, không
  đổi Ý ĐỊNH của Task 11 (không mở Chương trước khi Tác phẩm xong), chỉ sửa lại đúng nơi
  `setMode` được gọi. Đo trước khi tin — không sửa spec, chỉ ghi lại phát hiện.
- **Hai lệnh THÊM ngoài danh sách Task 12** — `library.search_next`/`library.search_prev` (con
  trỏ danh sách kết quả) không có tên trong Task 12 ("lệnh `library.search` và
  `library.open_search_hit`"), nhưng danh sách kết quả (Task 9) cần một cơ chế "hàng đang chọn"
  để nút "Mở kết quả" biết mở HÀNG NÀO trên một danh sách nhiều hơn một hit — đúng khuôn đã có ba
  tiền lệ trong CHÍNH tệp `LibraryMode.vue` (`library.orphan_next/prev`,
  `library.work_next/prev`, `library.chapter_next/prev`), mỗi cặp đi kèm một cờ con trỏ + một nút
  "hành động trên hàng đang chọn". Không đúc một cơ chế mới; tái dùng nguyên khuôn đã có.

### 2026-08-29 — ba chỗ SAI trong chính spec, và KHÔNG một lượt dựng lại mã

Ba chỗ sai về **sự thật kiểm được**, lộ ra khi chạy §Verification. Cả ba nằm NGOÀI
`<intent-contract>` và **không** chỗ nào làm mã đi sai — nên không lượt `bad_spec` loopback nào
được kích hoạt. Ghi ở đây thay vì sửa lặng lẽ.

1. §Verification gọi `npm run check:dict-build` — **không có script tên đó** (`package.json:22`
   khai `check:dict`). Một tên sai cho `npm error` exit 1 và đọc lên **giống hệt một cổng ĐỎ**;
   lượt chạy đầu báo `dict-build=1` và mất một vòng chẩn đoán trước khi lộ ra đó là lỗi hạ tầng,
   không phải một phép kiểm. Đúng thứ `scripts/AGENTS.md` cấm: *"đừng bao giờ báo một kết quả
   không có thật"* — ở đây là chiều ngược lại, một kết quả ĐỎ không có thật.
2. §Verification đòi `grep -rn 'StoreKind::LibraryIndex' src-tauri/src/commands/` cho **0**. Nó
   cho **2**, cả hai là dòng **chú thích** nói chính luật đó và có từ trước `d48f076`. Phán quyết
   thật là `tests/library_index_boundary.rs`. Sửa thành *"0 ở vị trí mã"* kèm lý do.
3. §Code Map trỏ tới `…_rebuilt_at_version_5_…`; đích lược đồ nay là **6** ⇒ ca đó đổi tên.

**KEEP — ba thứ phải sống sót mọi lượt dựng lại:** (a) bảng đo tokenizer ở §Design Notes là phần
**ép** ra hình dạng hai chỉ mục, không phải minh hoạ; (b) luật *"chạy CẢ HAI chỉ mục mỗi lượt,
không điều phối theo hình dạng truy vấn"*; (c) luật *"con trỏ đặt qua `doiConTroToi`, không qua
`save_chapter_position`"* — đường kia ghi đè dữ liệu người dùng.

## Review Triage Log

### 2026-08-29 — Review pass

- intent_gap: 0
- bad_spec: 0
- patch: 9: (high 2, medium 4, low 3)
- defer: 4: (high 0, medium 3, low 1)
- reject: 9: (high 0, medium 3, low 6)
- addressed_findings:
  - `[high]` `[patch]` **Câu người dùng đã CẮT BỎ (`is_omitted = 1`) hiện lại qua đoạn trích của nửa bản dịch.** `core/segment/omit.rs` khai `is_omitted` là *"chốt lọc cho MỌI đầu ra"* (FR133/AC5) và doc-comment của chính module đó **dự đoán đúng lỗi này**: một bề mặt tiêu thụ MỚI đọc AC của riêng nó, thấy đủ, rồi phát lại nguyên câu người dùng đã bỏ. Thu hoạch nay ghi `''` cho `target_text` của câu đã cắt và **giữ nguyên `source_text`** — FR133 cắt câu khỏi BẢN DỊCH, không xoá nó khỏi nguyên tác, nên lọc cả hàng là đổi một lớp rỗng im lặng lấy một lớp khác. Ca canh khẳng định CẢ HAI vế.
  - `[high]` `[patch]` **Danh sách bị trần cắt trong IM LẶNG trong khi màn hình khai một con số dứt khoát.** `total` bằng `hits.len()` và giao diện đọc nó thành *"{total} kết quả"*, nên một truy vấn khớp hàng nghìn hàng vẫn hiện *"100 kết quả"* — một câu đúng về hình dạng, sai về sự thật, và vi phạm chính luật *"không trần nào cắt trong im lặng"* của story. Thêm `SearchReport::truncated` (lấy `limit + 1` làm BẰNG CHỨNG, **không** một `COUNT(*)` thứ hai trên cùng `MATCH`), một câu chữ riêng, ca hợp đồng dây, và một ca canh chiều ngược (không bị cắt thì KHÔNG được báo là bị cắt).
  - `[medium]` `[patch]` **`librarySearchStatus` bỏ qua `busy` một khi đã nạp lần nào** ⇒ lượt tìm THỨ HAI hiện danh sách của truy vấn TRƯỚC và khai nó là kết quả, suốt thời gian lượt mới bay. Chính doc-comment của module khai ca ② là `searchBusy` không kèm điều kiện. Đưa `busy` lên nhánh đầu; hai ca canh.
  - `[medium]` `[patch]` **`isSearchHitArray` chỉ kiểm `hits[0]`** ⇒ một hàng thứ hai sai hình dạng đi thẳng vào `v-for`. Kiểm mọi phần tử; hai ca canh (hàng thứ hai hỏng; và một báo cáo THIẾU hẳn `truncated`).
  - `[medium]` `[patch]` **`snippet(library_source_fts, …, 10)` cho ZERO chữ ngữ cảnh.** Một "token" của `trigram` là cụm ba ký tự trượt từng ký tự, nên 10 token ≈ 12 ký tự. Đo 2026-08-29 (SQLite 3.43.2): `max_tokens = 10` ⇒ `"… ‹zzqqmarker› …"`; `= 64` ⇒ một câu đọc được. AC đòi *"đoạn văn bản khớp"*, và một đoạn chỉ chứa đúng từ khoá không phải một đoạn văn bản. Hai nhánh nay mang HAI con số, kèm bảng đo tại chỗ.
  - `[medium]` `[patch]` **Lượt dọn văn bản khi thư mục gốc biến mất không có ca nào chạm.** Ca sẵn có (`a_root_that_existed_with_rows_then_vanishes_…`) dựng fixture bằng `write_atproj`, nơi `project.db` cố ý KHÔNG hợp lệ — nên nó xanh **kể cả khi** lượt dọn bị gỡ hẳn: nó khẳng định trên một bảng vốn rỗng. Thêm ca đi qua `write_atproj_with_real_project_db` rồi xoá gốc, khẳng định `hits` rỗng VÀ `indexed_segments == 0`.
  - `[low]` `[patch]` **AC phân biệt dấu chỉ có lưới ở tầng ĐỘNG CƠ, không ở đường người dùng gõ.** Thêm một ca e2e thứ hai với cặp `khoáng…`/`khoang…` (≥ 3 ký tự, đặt ở nửa nguyên văn vì fixture không dựng được `target_text`) — đo 2026-08-29 xác nhận `trigram` mặc định PHÂN BIỆT dấu, nên phép đối chứng có nghĩa. ⚠️ Lượt đầu của ca này ĐỎ vì nó chờ trên SỐ HÀNG trong khi danh sách còn kết quả của ca trước; phép chờ đổi sang bám NỘI DUNG, ghi lại thay vì sửa lặng lẽ.
  - `[low]` `[patch]` `an_index_file_at_schema_version_4_is_deleted_and_rebuilt_at_version_**5**_…` đo một đích nay là **6** — đổi tên.
  - `[low]` `[patch]` **`trigram` không mang mệnh đề `remove_diacritics` nào, nên *"nó có lặng lẽ khoan dung dấu không"* là một câu hỏi THẬT chưa ai trả lời.** Đo rồi ghi vào `schema.rs`: mặc định của `trigram` là PHÂN BIỆT dấu ⇒ AD-27 đứng vững ở CẢ HAI chỉ mục.

**Bốn mục hoãn** (chi tiết + chủ ở `deferred-work.md` §"vòng review"): chuẩn hoá Unicode NFC/NFD
(chủ Ice, rộng hơn story vì đường từ điển cũng dính) · trần ứng viên `trigram` sắp theo thứ tự
KHO chứ không theo liên quan (chủ Story 6.18) · mỗi thao tác vòng đời nay mở `project.db` của
MỌI Tác phẩm (chủ Story 6.18) · một `INSERT` hỏng làm trượt cả lượt `rebuild` trong khi một lượt
ĐỌC hỏng thì chỉ bỏ qua đúng Tác phẩm đó (chủ Ice).

**Chín mục bác**, ba trong số đó bác **sau khi đo** chứ không bằng phán đoán:
- *"Đua giữa hai lượt mở Chương làm con trỏ đặt theo một `targetSegmentId` cũ"* — `dangChuyenChuong`
  (`editorPanelState.ts:1774`–`:1844`) bao trọn thân hàm, kể cả đoạn đặt con trỏ.
- *"`rebuild` toàn thư viện chạy sau MỖI lượt sửa segment"* — `grep -n "reindex" src-tauri/src/commands/segment.rs`
  cho **0**; các chỗ kích hoạt đều là thao tác rời rạc và thưa. (Vế CÒN LẠI của mục này — chi phí
  mở `project.db` của mọi Tác phẩm — thì có thật và đã vào sổ nợ.)
- *"`project.db` cũ hơn đích cho một lỗi SQL chung thay vì một lý do có tên"* — lỗi đó ĐÃ được bắt
  theo từng Tác phẩm và dồn vào `RebuildOutcome::text_skipped`, tức đã là một lượt bỏ qua có đếm.
Sáu mục còn lại: bộ e2e không chạy ở `push`/PR (có thật, nhưng là điều kiện toàn kho đã có chủ —
`e2e/AGENTS.md`, chủ Ice) · thứ tự hit (bản dịch trước nguyên văn) · một segment khớp cả hai nửa
hiện hai hàng (`field` phân biệt được, đúng thiết kế) · `PROJECT_DB_FILE` trùng một literal ·
`limit: 0` kẹp lên 1 (frontend không bao giờ gửi 0) · *"câu rỗng phải MỜI thử chế độ khoan dung"*
(§Never của story cấm dựng một nút chưa có đường chạy; đã có mục nợ, chủ Story 5.10).


## Design Notes

### Vì sao HAI tokenizer, và vì sao không một cái nào đủ

Đo 2026-08-29, SQLite 3.43.2, chạy thật chứ không suy luận. Kho thử: ba hàng tiếng Việt
(`má của tôi rất hiền` · `ma quỷ trong đêm` · `mà anh ấy đi rồi`) và ba hàng nguyên văn
(`天下大势，分久必合，合久必分。` · `the quick brown fox` · `state-of-the-art tooling`).

| Truy vấn | `unicode61 remove_diacritics 0` | `unicode61 remove_diacritics 2` | `trigram` |
|---|---|---|---|
| `má` | **1** — đúng hàng `má` | **3** — gộp cả `ma`/`mà` | **0** (2 ký tự) |
| `ma` | 1 — đúng hàng `ma` | — | — |
| `分久必合` trong `天下大势，分久必合` | **0** | — | **1** |
| `天下` trong `天下大势…` | **0** | — | **0** (2 ký tự) |
| `uick bro` trong `the quick brown fox` | 0 | — | **1** |
| `BROWN` trong `…brown fox` | — | — | **1** (không phân biệt hoa/thường) |

Ba kết luận, mỗi cái loại một phương án:

- **`remove_diacritics 2` phá thẳng AC phân biệt dấu** — nó là chỉ mục PHỤ của Story 5.10, đúng
  như AD-27 viết. Không dựng ở đây.
- **`unicode61` một mình làm nửa nguyên văn CÂM với chữ Hán.** `unicode61` gộp một dải Hán liền
  nhau thành **một token**, nên một từ nằm giữa câu trả 0. Với một công cụ dịch mà nguồn chủ yếu
  là chữ Hán, đó gần như là không giao nửa nguồn của FR8.
- **`trigram` một mình phá AC `má`** — 2 ký tự nằm dưới sàn của trigram.

⇒ Cặp `unicode61 remove_diacritics 0` cho **bản dịch** (phân biệt dấu, khớp trọn từ, dải bất kỳ)
và `trigram` cho **nguyên văn** (chuỗi con thật, Hán và Latin, từ 3 ký tự) là hình dạng hẹp nhất
thoả được cả hai AC. `ARCHITECTURE-SPINE.md` §Câu hỏi để ngỏ ghi *"Cấu trúc chi tiết chỉ mục FTS
cho tìm kiếm Library — AD-27 đã cố định phần bất biến; phần còn lại là **hình dạng bảng mà code
sở hữu**, Giai đoạn 3"*, nên chọn hình dạng ở đây **không** cần một AD mới.

⚠️ **Vùng câm còn lại, ghi ra thay vì để người sau tưởng đã xét:** một truy vấn **1–2 ký tự**
không tra được nửa nguyên văn, và từ ghép hai chữ Hán (`天下`, `江湖`) rơi đúng vào đó. Đóng hẳn
nó cần một nhánh thứ ba theo khuôn `char_idx` của AD-26 — một **cơ chế**, không một dòng thêm.
Nó vào sổ nợ với **chủ là Ice** kèm chính bảng số trên, vì `AGENTS.md:15` nói hai phương án đều
hợp lệ thì trình cả hai kèm số đo, đừng tự chọn rồi đi tiếp. Hôm nay nó là một trạng thái **có
tên trên màn hình**, không phải một con số 0 im lặng.

### Vì sao chạy cả hai chỉ mục mỗi lượt, không điều phối theo hình dạng truy vấn

Một bộ điều phối *"có chữ Hán ⇒ nhánh nguyên văn, ngược lại ⇒ nhánh bản dịch"* nghe rẻ hơn và
**sai**. Bản dịch tiếng Việt của một truyện Trung Quốc chứa đầy tên riêng phiên âm, và nguyên văn
tiếng Anh chứa đầy chữ Latin — hai vùng chữ **không** chia đôi theo cột. AD-44 đã ghi đúng lớp lỗi
này ở đường từ điển: *"bôi đen `API` trong một truyện tiếng Trung mà lọc `lang='zh'` cho 0 hàng dù
mục `API` có thật"*. Hai truy vấn song song, hợp kết quả, và mỗi hit tự khai `field` của nó.

### Vì sao thu hoạch nằm TRONG `rebuild`, không phải một `index_one`

`indexer.rs:59-70` đã bác sẵn đường tách: hai đường ghi cho cùng một bảng là hai chỗ phải giữ đúng
cùng logic, và chúng **sẽ** trôi khỏi nhau. Thu hoạch văn bản không đổi lý lẽ đó — nó chỉ làm mỗi
lượt `rebuild` đắt hơn, và phép đánh đổi giữ nguyên hình dạng cũ.

⚠️ **Một guard tăng dần "chỉ thu hoạch lại khi `meta.json.updated_at` đổi" là SAI HÔM NAY, và sai
im lặng.** `src-tauri/AGENTS.md:29` ghi món nợ đang mở: `work.updated_at`/`chapter.updated_at`
**không** được bơm vào giao dịch flush (chủ Story 5.6), vì làm thế khiến cổng đang xanh
`segment_contract.rs::a_flush_touches_exactly_target_text_and_updated_at_and_nothing_else` đỏ.
⇒ Người dùng sửa bản dịch mà `updated_at` không đổi ⇒ guard bỏ qua ⇒ chỉ mục giữ chữ CŨ: tìm ra
câu đã xoá, không tìm ra câu vừa gõ, **không cổng nào đỏ**. Guard đó chỉ an toàn sau khi món nợ
kia đóng; nó vào sổ nợ, không vào mã.

### Vì sao `ReadOnlyDb` chứ không `Store::open`

`readonly.rs:19-24` liệt kê đúng bốn thứ `Store::open` ghi vào tệp: `READ_WRITE | CREATE`,
`journal_mode = WAL`, **bộ di trú**, luồng writer. Ba trong bốn là hành vi **ghi vào một `.atproj`
mà lượt quét không sở hữu**, và cái thứ ba nguy hiểm riêng: một lượt quét sẽ **di trú hàng loạt**
mọi Tác phẩm trong thư viện chỉ vì người dùng mở Library. `ReadOnlyDb` là phần còn lại sau khi bỏ
hết bốn thứ đó.

⚠️ **Giới hạn thật:** một tệp mở chỉ đọc vẫn phải thấy được những lượt ghi còn nằm trong sidecar
WAL, và `PRAGMA wal_autocheckpoint = 0` (AD-12) nghĩa là WAL của một Tác phẩm vừa đóng có thể còn
dài. Ca đó nằm trong §I/O Matrix và phải **đo** — thu hoạch một Tác phẩm vừa ghi xong rồi khẳng
định chữ mới nhất có trong chỉ mục — chứ không suy luận.

### Vì sao con trỏ đặt qua `doiConTroToi`, không qua `save_chapter_position`

Đường "ghi vị trí làm việc rồi mở Chương để Rust trả về đúng câu đó" nghe gọn hơn, nhưng nó **ghi
đè dữ liệu người dùng**: `chapter_position` là chỗ Chương nhớ *"tôi đang dở ở câu nào"*, và một
lượt xem kết quả tìm kiếm không phải một lượt làm việc. Đặt `caretPlacement` ở frontend là trạng
thái trình bày, không chạm đĩa; lượt gõ tiếp theo mới ghi vị trí qua nhịp `positionFlush` như mọi
khi. Và nó đi qua **đúng một** bản của đường dời con trỏ (`:1367`) — `:1362-1363` ghi thẳng rằng
một đường dời thứ hai làm mất chữ người dùng vừa gõ, im lặng, không cổng nào đỏ.

## Verification

**Commands:**
- `npm run check:deps && npm run check:tokens && npm run check:i18n && npm run check:commands && npm run check:layout && npm run check:panel-refs && npm run check:debt-owner && npm run check:gates && npm run check:dict && npm run check:dict-manifest && npm run check:lint` -- expected: mọi cổng exit 0
  *(🔵 sửa 2026-08-29: bản đầu của spec viết `check:dict-build` — **không có script tên đó**; `package.json:22` khai `check:dict` chạy `scripts/check-dict-build.mjs`. Một tên sai ở đây cho `npm error` exit 1 và đọc lên giống một cổng ĐỎ.)*
- `npm run build` -- expected: `vue-tsc` 0 lỗi, `dist/` có mặt (chạy TRƯỚC `cargo test`)
- `npm run test` -- expected: 0 ca đỏ, và bộ ca của `tests/frontend/librarySearch.test.ts` **có chạy** (số tệp/ca tăng so với baseline)
- `cd src-tauri && cargo test --locked` -- expected: 0 ca đỏ; bộ ca mới của `library_index_contract.rs` có mặt trong đầu ra
- `npm run test:e2e -- --spec e2e/specs/story-5-9-library-search.e2e.mjs` -- expected: 1 passing trong WKWebView thật
- `grep -n 'remove_diacritics' src-tauri/src/core/store/schema.rs` -- expected: ≥ 1 kết quả, và **mọi** kết quả ở vị trí DDL mang giá trị `0` (không `1`, không `2` — `_nd` thuộc Story 5.10)
- `grep -rn 'StoreKind::LibraryIndex' src-tauri/src/commands/` -- expected: **0** kết quả **ở vị trí mã**
  *(🔵 sửa 2026-08-29: `commands/library.rs:13` và `:123` là hai dòng **chú thích** nói chính luật này, và cả hai có từ trước `d48f076` — một phép grep trần đếm chúng. Phán quyết thật là `src-tauri/tests/library_index_boundary.rs`, và nó xanh.)*
- `grep -rn 'StoreSpec::project' src-tauri/src/core/library/indexer.rs` -- expected: **0** kết quả (thu hoạch đi qua `ReadOnlyDb`, không qua đường có di trú)

**Manual checks (if no CLI):**
- **Bốn đối chứng ĐỎ bắt buộc, mỗi cái GỠ một chỗ nối rồi chạy bộ test CŨ** — một bộ xanh không
  chứng minh chỗ nối được canh: ① đổi `remove_diacritics 0` thành `2` ⇒ ca phân biệt dấu phải ĐỎ;
  ② gỡ lượt `INSERT INTO …_fts(…) VALUES('rebuild')` ⇒ ca tìm-thấy phải ĐỎ; ③ gỡ phép bọc cụm
  ngoặc kép ⇒ ca `state-of-the-art` phải ĐỎ; ④ trả `indexed_segments` bằng `hits.len()` thay vì
  quần thể thật ⇒ ca *"chỉ mục trống khác không khớp"* phải ĐỎ. 🔴 Mỗi đối chứng phải là một phép
  **GỠ** hoặc **DỜI** thật, không phải một dòng chèn thêm — một dòng chèn để dòng gốc chạy tiếp và
  ca vẫn xanh, tức "đối chứng" không chứng minh gì. Ghi lại kết quả từng cái; một đối chứng cho
  XANH thì nghi phép đối chứng trước, nghi bộ test sau.
- **Đo p95 tìm kiếm** trên một `library-index.db` tổng hợp cỡ 5.000 Chương (dựng bằng fixture,
  không qua sản phẩm), ghi **con số, phiên bản toolchain và ngày** vào §Auto Run Result. Đây là
  phép đo **sơ bộ**, chạy tay, **không** dựng thành một ca test — một phép kiểm dựa trên thời gian
  đo mã CỘNG máy và sẽ chập chờn.
- **Đọc lượt CI** trước khi kết luận xanh: `pre-push` chạy trên macOS của Ice và không nói gì về
  nửa Windows.

## Auto Run Result

Status: done
Blocking condition: (không có)

### Đã dựng

Tìm kiếm full-text xuyên toàn Library (FR8), đọc từ `library-index.db` (AD-8): một bảng nội dung
`library_segment` cộng **hai** chỉ mục FTS5 external-content — `library_target_fts`
(`unicode61 remove_diacritics 0`, AD-27, phân biệt dấu, khớp trọn từ) và `library_source_fts`
(`trigram`, chuỗi con thật, phủ chữ Hán). `LIBRARY_INDEX_MIGRATIONS` **5 → 6**, viết lại DDL TẠI
CHỖ đúng luật kho dẫn xuất — không một bước di trú thứ hai.

`Indexer::rebuild` vẫn là **đường ghi duy nhất**; nó nay mở `project.db` của mỗi `.atproj`
**chỉ đọc** qua `ReadOnlyDb` (danh sách `StoreKind` cho phép nới từ `{Dict}` thành
`{Dict, Project}`, miễn trừ có tên) — `Store::open` sai đường ở đây vì nó ghi bốn thứ vào tệp,
kể cả **chạy bộ di trú** trên một Tác phẩm mà lượt quét không sở hữu. Một bề mặt IPC mới
(`library_search`, vỏ `(async)`), một tệp state mới (`src/modes/librarySearch.ts`), khối tìm
kiếm ở `LibraryMode.vue`, và `openChapterById(chapterId, segmentId?)` đặt con trỏ vào đúng câu
khớp qua **đúng một** đường dời con trỏ đã có. **Không** `MessageKey` mới — ca *"chưa mở chỉ
mục"* tái dùng `indexer_is_missing()`; các ca rỗng là **trạng thái trong báo cáo**, không phải lỗi.

### Số đo nghiệm thu (sau lượt vá)

- 11 cổng tĩnh + `check:lint` — **xanh**.
- `npm run build` (`vue-tsc`) — **0 lỗi**.
- `npm run test` — **48/48 tệp, 681/681 ca**.
- `cargo test --locked` — **952 ca, 0 đỏ** (35 binary).
- `npm run test:e2e -- --spec e2e/specs/story-5-9-library-search.e2e.mjs` — **2 passing (49,8 s)**
  trong WKWebView thật.

### Đo p95 (SƠ BỘ — không nghiệm thu NFR3)

Chạy tay: `cargo test --locked --test library_index_contract -- --ignored --nocapture`
(`bench_p95_of_a_library_search_over_five_thousand_chapters`, `#[ignore]` có chủ ý — một phép
kiểm dựa trên thời gian đo mã CỘNG máy và sẽ chập chờn nếu dựng thành cổng).

| | |
|---|---|
| Quần thể | 5.000 Chương × 10 segment = **50.000 hàng** |
| Thu hoạch (`rebuild` toàn phần) | **2.180,9 ms** |
| `search` p50 | **67,300 ms** |
| `search` p95 | **138,490 ms** |
| Ngưỡng NFR3 (TẠM, `[A6]`) | 500 ms |
| Toolchain | rustc **1.97.1** (8bab26f4f 2026-07-14) · Node **v26.7.0** · macOS |
| Ngày | **2026-08-29** |

⚠️ **SƠ BỘ, và NFR3 KHÔNG được đánh dấu đạt.** Kho tổng hợp có đúng MỘT `.atproj` mang 5.000
Chương, trong khi một thư viện thật trải trên hàng trăm `.atproj` — hình dạng quét khác hẳn, và
chính con số thu hoạch 2.180,9 ms là chỗ khác biệt đó sẽ lộ ra. Phép đo đủ điều kiện là **Story
6.18**, sau khi FR14 (Epic 6) có đường tạo 5.000 Chương thật.

### Bảy đối chứng ĐỎ — một bộ xanh không chứng minh chỗ nối được canh

| Gỡ chỗ nối | Kết quả bắt buộc | Đo được |
|---|---|---|
| `remove_diacritics 0` → `2` | ĐỎ | **1 ca đỏ** (`diacritics_distinguish_six_…`) |
| gỡ hai câu `INSERT INTO …_fts(…) VALUES('rebuild')` | ĐỎ | **11 ca đỏ** |
| gỡ phép bọc cụm ngoặc kép (`fts_phrase` trả chuỗi trần) | ĐỎ | **1 ca đỏ** (`fts5_syntax_characters_…`) |
| `indexed_segments` = `hits.len()` | ĐỎ | **1 ca đỏ** (`an_empty_index_and_a_populated_index_…`) |
| trung hoà `doiConTroToi(targetSegmentId)` | ĐỎ | **1 ca vitest đỏ** (con trỏ ở đúng câu khớp) |
| gỡ `CASE WHEN is_omitted = 1 THEN ''` | ĐỎ | ca `an_omitted_sentence_is_gone_…` (đối chứng ghi sẵn trong doc-comment của ca) |
| trả `truncated: false` cứng | ĐỎ | ca `a_result_list_cut_by_the_limit_…` (đối chứng ghi sẵn trong doc-comment của ca) |

⚠️ **Một lượt đối chứng KHÔNG hợp lệ, ghi ra thay vì giấu:** phép gỡ thứ ba lần đầu chạy bằng một
`perl -0pi` **không khớp** (chuỗi thoát sai), nên tệp KHÔNG đổi và bộ test xanh. Một "đối chứng"
cho XANH không chứng minh gì — nghi phép đối chứng trước, nghi bộ test sau. Làm lại bằng một phép
sửa chính xác thì ca đỏ đúng chỗ.

### Ba chỗ sửa thêm, tìm ra khi chạy §Verification (ngoài vòng review)

1. **60 trong 61 ca vitest đỏ ở lượt đầu là TẢI MÁY, không phải một dòng mã.** Cả 61 đều là
   `Test timed out in 5000ms`, và máy đang chạy một lượt `cargo test` nền (load average **6,06**).
   Chờ máy rảnh (load **3,11**) rồi đo lại: **1** ca đỏ. Một phép kiểm dựa trên timeout đo mã
   CỘNG máy; hai lượt đo phải cùng tải máy mới so được.
2. **Ca đỏ THẬT còn lại phát hiện rằng chính nó không đo được thứ nó khai.** Ca *"chống đua giữa
   hai lượt gõ"* đặt tên cơ chế là `searchSequence`, nhưng `runLibrarySearch` chặn ở `busy`
   **TRƯỚC** khi bump `sequence`, nên hai lượt không bao giờ cùng bay và `mySequence !== sequence`
   **không với tới được** từ đường đó. Cơ chế THẬT là cặp `busy` + `reloadPending`. Ca được viết
   lại để đo đúng cơ chế đó (khẳng định lượt ghi nhớ chạy lại bằng truy vấn MỚI), và câu chú thích
   trong mã sản phẩm được sửa để nói đúng đường DUY NHẤT còn với tới hàng rào đó
   (`resetLibrarySearch` chạy giữa chừng).
3. **Một lỗi kiểu ở tệp test** (`vue-tsc`: `Type 'never' has no call signatures`) — TypeScript
   không theo dõi lượt gán nằm trong hàm lồng. Sửa **trong tệp test** bằng khuôn "tạo promise
   trước, giữ resolver", **không** thêm một `?.` vào mã sản phẩm (`tests/AGENTS.md`).

### Tệp đã đổi

**Rust**
- `core/store/schema.rs` — `library_segment` + hai chỉ mục FTS5, bump `to_version` 5 → 6, khối đo tokenizer.
- `core/store/readonly.rs` — danh sách `StoreKind` cho phép nới thành `{Dict, Project}`, miễn trừ có tên.
- `core/library/indexer.rs` — thu hoạch trong `rebuild`, `Indexer::search` hai nhánh + xác minh chuỗi con, `SearchReport::truncated`, lọc `is_omitted` ở nửa bản dịch, `snippet` 64 cho nhánh trigram, dọn văn bản khi gốc vắng mặt.
- `core/i18n/mod.rs` — **0 khoá mới**, một khối khai danh mục ĐÓNG.
- `commands/library.rs` — hàm thuần `search_library` + vỏ `(async)`; `lib.rs` — vỏ vào `generate_handler!`.

**Frontend**
- `config/library.ts` — adapter `searchLibrary`, type guard kiểm MỌI phần tử.
- `modes/librarySearch.ts` (mới) · `modes/LibraryMode.vue` — khối tìm kiếm, năm trạng thái phân biệt được + câu "còn nữa".
- `panels/editorPanelState.ts` — `openChapterById(chapterId, segmentId?)`.
- `commands/index.ts` · `main.ts` · `i18n/vi.json`.

**Test và bàn đo**
- `tests/library_index_contract.rs` — 19 ca hành vi + bàn đo p95 `#[ignore]`.
- `tests/ipc_contract.rs` · `config_invariants.rs` · `library_commands_contract.rs`.
- `tests/frontend/librarySearch.test.ts` (mới, 21 ca) · `chapterPosition.test.ts`.
- `e2e/specs/story-5-9-library-search.e2e.mjs` (mới, 2 ca).

**Tài liệu** — `src-tauri/AGENTS.md` (mệnh đề *"Indexer chỉ đọc `meta.json`"* sửa tại chỗ) ·
`deferred-work.md` (8 mục có chủ) · `sprint-status.yaml`.

### Kết quả review — 9 vá · 4 hoãn · 9 bác

Bốn lớp chạy song song trên diff 3.386 dòng. **0 `intent_gap`, 0 `bad_spec`.** Chi tiết ở
§Review Triage Log. Hai mục `high`: câu đã cắt bỏ hiện lại qua tìm kiếm, và một trần cắt danh
sách trong im lặng.

**Follow-up review khuyến nghị: `true`** — đếm trên đúng chín mục `patch` của lượt này:
high **2**, medium **4**, low **3**; có mục `high` ⇒ `true` (điểm `3 × 4 + 1 × 3 = 15`, cũng vượt
ngưỡng 5).

### Rủi ro còn lại

- ⚠️ **CHƯA đọc lượt CI.** Mọi số đo trên đây chạy trên macOS của Ice; nửa Windows chưa nói gì.
- 🔴 **Truy vấn 1–2 ký tự không tra được nửa nguyên văn** — từ ghép hai chữ Hán (`天下`, `江湖`)
  rơi đúng vào đó. Hôm nay là một trạng thái CÓ TÊN trên màn hình, không phải một con số 0 im
  lặng; đóng hẳn cần một cơ chế thứ ba theo khuôn AD-26. Sổ nợ, **chủ Ice**.
- ⚠️ **Không có chuẩn hoá Unicode NFC/NFD** — hai chuỗi trông giống hệt nhau có thể không khớp
  trên một chỉ mục phân biệt dấu. Lớp có sẵn của kho (đường từ điển cũng dính). Sổ nợ, **chủ Ice**.
- ⚠️ **Thu hoạch chạy toàn phần mỗi lượt `rebuild`**, và `rebuild` nay mở `project.db` của MỌI
  Tác phẩm. Đường tối ưu hiển nhiên bị chặn bởi món nợ `updated_at` (chủ Story 5.6). Sổ nợ, **chủ
  Story 6.18**.
- ⚠️ **Bộ e2e nằm ngoài `push`/PR** (nhịp đêm, `e2e/AGENTS.md`) — ca duy nhất chứng minh "mở đúng
  Tác phẩm khi hai `project.db` cùng `chapter_id`" vì thế không canh lượt push nào. Điều kiện toàn
  kho đã có chủ (Ice), không phải chỗ hở của story này.
- ⚠️ `story-5-6-library-grid.e2e.mjs` **đỏ từ baseline** (chủ Story 5.6) — không đọc thành hồi quy.
