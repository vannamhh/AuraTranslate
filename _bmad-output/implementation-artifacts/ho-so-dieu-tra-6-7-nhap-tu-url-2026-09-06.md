# Hồ sơ điều tra Story 6.7 — "Nhập từ URL bằng danh sách link"

**Ngày đo:** 2026-09-06 · **Baseline:** `baa2587614df97d4b5a35b3c6ac5e55e57e44eec` (master, cây sạch)
**Trạng thái:** lượt `bmad-build` DỪNG ở bước định tuyến. Ice chốt chạy `correct-course` cho Story 6.6b trước.
**Vai của tệp này:** giữ lại phép đo để lượt lập spec 6.7 sau này không phải điều tra lại. KHÔNG phải một spec.

🔴 **Tệp này CỐ Ý không mang frontmatter `status:`.** `bmad-build` bước 1 quét `_bmad-output/implementation-artifacts/**` tìm mọi tệp có `status:` mang giá trị workflow (`draft`/`ready-for-dev`/`in-progress`/`in-review`) rồi HALT hỏi Ice muốn tiếp tục cái nào. Thêm một dòng `status:` vào đây làm một hồ sơ điều tra hiện lên như một story đang dở — đúng lớp lỗi "một thứ đúng hình dạng nói sai sự thật". Nếu lượt lập spec 6.7 sau này muốn dùng tệp này, nó ĐỌC tệp này rồi ghi ra một `spec-6-7-*.md` riêng, không đổi tệp này thành spec.

---

## 0. Hai phán quyết của Ice trong lượt này (2026-09-06)

### ① Thứ tự — DỪNG, chạy `correct-course` cho Story 6.6b trước

Sổ nợ `deferred-work.md:9681-9707` khai Story **6.6b** (AC7 của `epics.md` §Story 6.6 — chọn nhiều
tệp cùng lúc) đứng **ngay sau 6.6 và TRƯỚC 6.7**. Đo 2026-09-06: chuỗi `6.6b` xuất hiện **0 lần**
trong `epics.md`, **0 lần** trong `sprint-status.yaml`. Chính mục nợ tự khai rằng thêm nó vào hai
tệp đó phải đi qua `correct-course`, và Ice là người kích hoạt.

⚠️ Ghi ra để lượt sau không đọc nhầm: 6.6b và 6.7 **không phụ thuộc nhau về kỹ thuật** — cả hai chỉ
cần bề mặt xem trước nhiều Chương mà Story 6.6 đã dựng. Đây là thứ tự quy hoạch, không phải hàng rào
kỹ thuật.

### ② Link hỏng — GIỮ CHỖ, ĐÁNH DẤU HỎNG

`ARCHITECTURE-SPINE.md:1085` liệt hành vi này ở bảng **câu hỏi mở**: *"Dán 50 link mà link thứ 30
hỏng thì làm gì: dừng, bỏ qua, hay giữ chỗ trống"*, hẹn quyết ở *"Giai đoạn 3, khi dựng màn xem
trước của FR123"*. **Ice chốt 2026-09-06:** mỗi link hỏng thành một mục trong màn xem trước mang
đúng lý do (404 · timeout · không kết nối được), **giữ nguyên VỊ TRÍ** trong danh sách; hai con số
`N link · N Chương` vì thế vẫn bằng nhau; người dùng quyết ở màn xem trước (bỏ mục hỏng, hay tải lại
riêng nó). Vẫn giữ bất biến AD-39 *"mọi thứ trước bước ghi"*.

Lý lẽ đã dùng để loại hai phương án kia:
- **Dừng cả lượt** — 49/50 link tốt bị vứt, và tải lại là 50 lời gọi mạng nữa (hình phạt thật trên
  site chậm hoặc có giới hạn tần suất).
- **Bỏ qua link hỏng** — phá đúng bất biến story tồn tại để dựng: `N link · N−1 Chương`, và người
  dùng mất dấu chính xác link nào đã rơi. Đó là **rỗng im lặng ở tầng danh sách** — lớp lỗi trung tâm
  của dự án.

🔴 Phán quyết ② vẫn nằm **ngoài** `ARCHITECTURE-SPINE.md`. Dòng `:1085` chưa được sửa. Lượt
`correct-course` hoặc lượt lập spec 6.7 phải mang nó vào, không để nó chết trong một transcript.

---

## 1. Phạm vi 6.7 vs 6.8 — ranh giới phải giữ

| | Story 6.7 (`epics.md:4734-4773`) | Story 6.8 (`epics.md:4776-4837`) |
|---|---|---|
| Covers | FR122 | NFR19 · NFR12 · AD-40 · AD-41 |
| Nội dung | dán danh sách link · đúng thứ tự · hai con số bằng nhau · không quét mục lục / không lần "chương sau" · hành vi nhất quán cho link hỏng · **chưa bấm nút thì 0 lời gọi mạng** | allowlist **hai tầng** · nhật ký domain · **bộ test tự động RIÊNG bắt buộc** |

`ci.yml:617-628` có sẵn khối *"CHỖ MÓC CHO EPIC SAU"* liệt kê *"Epic 6 · bốn test allowlist mạng
(AD-41)"* — đó là chỗ cắm của **6.8**, không phải 6.7. 6.7 chỉ nợ AC *"chưa bấm thì 0 lời gọi"*.

---

## 2. Tầng mạng — cái gì đã đóng, cái gì còn hở

- **HTTP client ĐÃ ĐÓNG.** `ARCHITECTURE-SPINE.md:1083` — đóng 2026-09-03 (Story 6.1) bằng **xác
  nhận `reqwest`**, không bằng crate mới. `src-tauri/Cargo.toml:52-60`: `reqwest = { version =
  "=0.13.4", features = ["blocking"] }`. Đo `cargo tree`: **0 crate mới**, +32 dòng cây.
  Ba năng lực đo thật trên server `127.0.0.1` tự dựng (`6-1-ban-do/reqwest-raw.tsv`):
  ① `redirect::Policy::custom` chặn đúng một chặng sang cổng khác, server bị chặn nhận **0** kết nối;
  ② đọc qua `Read` (không `.bytes()`) dừng ở **1.048.576/20.971.520 byte** — cắt theo dòng chảy;
  ③ `is_connect()`/`is_timeout()` phân biệt được lỗi kết nối.
  ⚠️ Chú thích tại `Cargo.toml` ghi rõ feature `blocking` bật **cho mũi thăm dò**; *"`Fetcher` thật
  (Story 6.7+) tự quyết lại có cần `blocking` hay đi hẳn đường async của Tauri"* — đây là một quyết
  định còn MỞ của story 6.7, không phải chuyện đã chốt.

- **`core/webimport/mod.rs` là stub thuần doc-comment**, ~23 dòng, **0 dòng mã**. Không có
  `struct Fetcher`, không `struct Extractor`, không trait injection nào cho HTTP client.

- **Ba điểm ra mạng (AD-15, spine `:210`):** ① `TranslationProvider` của `core::ai` (khai `reqwest`,
  Story 4.x) · ② kiểm tra phiên bản FR111 (Story 10.7, **chưa dựng — 0 kết quả grep**) ·
  ③ `Fetcher` — chính story này. Spine `:216`: điểm ③ khác hai điểm kia vì đích đến là host người
  dùng dán vào **lúc chạy**, không phải host ứng dụng biết trước — đây là lý do AD-41 tồn tại riêng.

- **Không cần thêm quyền Tauri.** `capabilities/main.json` giữ đúng ba mục
  (`core:path:default` · `core:event:default` · `core:resources:default`). `Fetcher` gọi `reqwest`
  **từ Rust**, không qua plugin JS — capabilities chỉ canh bề mặt IPC lộ ra webview. CSP
  (`tauri.conf.json:25`) cũng không áp cho tiến trình Rust, nên **không nới CSP**.
  Neo: `config_invariants.rs:382::capabilities_directory_holds_exactly_the_one_reviewed_file`.

- **`check-deps.mjs` đã biết trước story này.** Dòng cuối script ghi *"ba điểm ra mạng của AD-15 mở ở
  Story 4.x, 6.7, 10.7"* — một lời gọi mạng thật từ đường sản phẩm không phải vi phạm, miễn không
  thêm crate/plugin bị cấm. ⚠️ `@tauri-apps/plugin-http` **không** nằm trong danh sách cấm (vì chưa
  ai thêm) — nếu 6.7 lỡ thêm nó thì **không cổng nào chặn**.

---

## 3. Pipeline — chỗ nối đã có sẵn cho 6.7

- **`PIPELINE_ORDER`** bảy bước, `pipeline.rs:111-119`. `run_import` (`:528-530`) →
  `run_import_with_order` (`:374-523`). Chỗ gọi `run_import` **duy nhất** của cả crate là
  `commands/project.rs:234-238` (`run_pipeline`, hàm private), canh bởi `segment_pipeline_boundary.rs`.

- 🔴 **`already_chaptered` đã được viết SẴN cho story này.** Trường của struct `Flow` (private,
  `pipeline.rs:346`), đặt tại `:382-385`: `PipelineShape::Blob(_) => false`,
  `PipelineShape::Chapters(_) => true`. Doc-comment `:184-188` và `:342-346` nói thẳng: *"mỗi link
  trong danh sách URL (Story 6.7) là một Chương — KỂ CẢ khi danh sách chỉ có ĐÚNG MỘT link"*.
  ⇒ Story 6.7 **không** thêm bước, **không** đổi thứ tự; nó là bề mặt ĐẦU TIÊN dựng
  `PipelineShape::Chapters(N>1)` trên đường sản phẩm (`project.rs:1074` tự ghi đúng câu đó).

- **`PipelineInput`** (`:193-226`): `shape` · `encoding` · `chapter_pattern` · `source_lang` ·
  `cleanup_rules`. Builder `with_encoding` (`:249-261`) là đường sản phẩm có xem trước bảng mã.

- **Đường sản phẩm hôm nay:** `preview_import_encoding_from_text|_from_file` →
  `confirm_import_with_encoding` → `create_work` (`project.rs:299-496`). Nguồn giữa xem trước và xác
  nhận nằm ở `PendingImportSourceState = Mutex<Option<PendingImportSource>>` (`:984`), chở đúng một
  `PipelineShape`. `create_work_from_text|_from_file` còn sống nhưng **0 chỗ gọi sản phẩm** từ `src/`
  kể từ Story 6.3 — chỉ e2e dùng dựng fixture.

- **`ImportError`** 8 biến thể (`import.rs:89-189`); tiền lệ gần nhất cho biến thể mới là
  `InvalidChapterPattern { detail }` (`:183`, lỗi NGƯỜI DÙNG thật). `message_keys!`
  (`core/i18n/mod.rs:62-91`) — cú pháp `Variant => "khoa.cham" [param1, param2]`; **chưa có** khoá
  nào cho "tải URL thất bại".

- 🔴 **`CHAPTER_DDL` (`schema.rs:895-904`) KHÔNG có cột nào cho URL nguồn / xuất xứ.** Xuất xứ tài
  liệu bốn trường là **Story 6.15**, không phải 6.7 — nhưng nếu 6.7 muốn nhớ *link nào sinh ra Chương
  nào* thì đó là một bước di trú (`PROJECT_MIGRATIONS` hiện **19 bước**, bước cuối là
  `IMPORT_CLEANUP_RULE_DDL` của Story 6.5). Cần quyết tường minh, đừng để rơi.

- ⚠️ **Toàn bộ pipeline chạy ĐỒNG BỘ.** `run_import` không `async`. Grep `async fn` trong
  `src-tauri/src`: **0 hàm do dự án tự viết** (hai kết quả đều là comment). Grep `Channel`: **0 kết
  quả** — chưa có Tauri Channel nào đang dùng. Cơ chế nền hiện có là `std::thread::spawn` thuần
  (khuôn `spawn_import_scan`, `project.rs:791`). ⇒ Tải N link là **quyết định kiến trúc còn mở**:
  (a) một vỏ `#[tauri::command] async fn` mới tải N link rồi gói thành `PipelineShape::Chapters`
  TRƯỚC khi vào đường đồng bộ hiện có, hay (b) `std::thread::spawn` + `app.emit` như khuôn có sẵn.
  Không có seam nào dựng sẵn cho việc này.

---

## 4. Frontend — cái gì tái dùng được, cái gì phải dựng mới

- **`ImportPreviewOverlay.vue`** (1414 dòng): bốn tầng ở `:456-507` (bảng mã) · `:510-546` (chuẩn
  hoá) · `:549-554` (ranh giới nội dung — **rỗng có chủ, Story 6.9**) · `:557-713` (làm sạch) ·
  `:716-819` (tách Chương). Bẫy Tab dùng danh sách selector cố định `focusableWithin` (`:375-382`) —
  mọi `<input>` mới phải nằm trong đó. Khoá i18n phải là **literal** qua `switch` cạn (`:71-79` giải
  thích: `check:i18n` chỉ quét được literal). Thao tác mang tham số đi qua `@change`/`@submit.prevent`,
  **không** `@click` (`check:commands` Kiểm A đòi `@click` là đúng một `dispatch('<id>')`).

- **`importPreviewState.ts`**: `resetImportPreview()` (`:814-842`) quét **22 ô** — mọi ô mới phải vào
  đây. `sequence` (`:189`) là vé chống đua. `reloadImportPreviewAfterRuleChange` là lõi tải lại dùng
  chung. Điểm tiêm cho URL: một `openImportPreviewFromUrls` theo khuôn `openImportPreviewFromText`
  (`:378-412`), mở rộng `lastSubmittedFrom` (`:102`) thêm biến thể thứ ba, và một ô thứ ba song song
  `pendingText`/`pendingPath` (`:113-114`).

- 🔴 **Hai con số phải là computed CỤC BỘ trong `.vue`, không phải một trường trên dây.** Đếm dòng
  non-empty của chuỗi vừa dán là JS thuần, 0 IPC — đó chính là thứ làm cho AC *"chưa bấm nút thì
  không lời gọi mạng nào"* đúng theo cấu tạo, không phải theo lời hứa. Một trường mới trên
  `ImportEncodingPreview` chỉ có SAU khi đã gọi Rust, tức đã ra mạng.

- 🔴 **"Thêm Chương vào Tác phẩm sẵn có" CHƯA TỒN TẠI MỘT DÒNG NÀO.** Grep
  `add_chapter|append_chapter|import_chapter` trên `src/config/*.ts`, `src/commands/index.ts` và
  `src-tauri/src`: **0 kết quả cả ba nơi**. Màn xem trước hôm nay luôn **TẠO** một Tác phẩm chưa tồn
  tại (`create_work` qua `confirm_import_with_encoding`) — cùng giới hạn kiến trúc đã buộc Story 6.5
  thu hẹp tầng "Tác phẩm" của luật làm sạch (`ImportPreviewOverlay.vue:255-264`).
  ⇒ Nửa thứ hai của AC *"tạo Tác phẩm mới, HOẶC thêm Chương vào Tác phẩm sẵn có"* là **hạ tầng mới
  hoàn toàn**: một lệnh Rust mới + adapter TS mới + UI chọn đích. Đây là ứng viên số một cho một lượt
  kiểm đa-mục-tiêu khi lập spec 6.7 — đừng đọc nó như một dòng thêm vào.
  *(Bề mặt ĐỌC thì đã có: `library_list_works` / `CMD_LIST_WORKS` ở `src/config/library.ts:310`.)*

---

## 5. Nghiệm thu — khuôn có sẵn để chép

- 🔴 **Khuôn máy chủ cục bộ ĐÃ TỒN TẠI và đã chạy thật.** `src-tauri/tests/webimport_probe.rs` tự dựng
  server HTTP thô bằng `std::net::TcpListener::bind("127.0.0.1:0")` (`spawn_once`, `:611-625`) —
  không framework, một `thread::spawn` accept đúng một kết nối rồi viết response tay. Ba ca dùng nó:
  `redirect_case` (`:391-437`, chuyển hướng cross-host qua PORT khác) · `size_cap_case` (`:439-490`) ·
  `network_failure_case` (`:554-593`, bind rồi thả cổng ngay để có "không ai lắng nghe" tất định).
  ⚠️ Cả bàn đo đang `#[ignore]` (`:358`). Đây là khuôn thẳng nhất để 6.7/6.8 chép và **bỏ `#[ignore]`**
  — test `Fetcher` thật mà không chạm mạng thật. Không có `mockito`/`wiremock`/`httpmock` trong kho.

- **Khuôn "không gọi mạng" gần nhất:**
  `glossary_han_viet_suggestion_contract.rs:478-497::the_suggestion_path_names_no_network_client_so_it_cannot_depend_on_the_network`
  — quét TĨNH văn bản nguồn cấm token `reqwest|ureq|hyper|TcpStream|tauri_plugin_http|http://|https://`.
  ⚠️ Áp được cho module **lân cận**, KHÔNG áp được cho `Fetcher` (thứ phải gọi mạng). Ca "chưa bấm ⇒
  0 lời gọi" của 6.7 phải là một phép **đếm lời gọi thật** ở tầng vitest, không phải một phép quét token.

- **Ba ca "0 lời gọi IPC" hiện có** — `importPreviewChapters.test.ts:111` và `:192`,
  `importPreviewNormalized.test.ts:103`. ⚠️ Cả ba đo nhịp *"SAU khi đã mở overlay, đổi lựa chọn không
  gọi thêm"*. **Chưa ca nào** canh nhịp *"đã dán N link, số đếm đang hiện, NHƯNG chưa bấm nút"* — đó
  là ca MỚI story 6.7 phải dựng.

- **Khuôn cổng ranh giới mới có tự-kiểm:** `cleanup_boundary.rs`. Hàm
  `text_before_first_cfg_test_line` định nghĩa gốc ở `segment_normalize_boundary.rs:98`, chép nguyên
  văn sang `cleanup_boundary.rs:94`, `segment_pipeline_boundary.rs:101`,
  `segment_chapterpattern_boundary.rs:104`. Hai ca tự-kiểm bắt buộc chép kèm:
  `..._is_not_fooled_by_a_comment_mentioning_the_attribute` (`cleanup_boundary.rs:371`) và
  `..._returns_the_whole_text_when_there_is_no_such_line` (`:383`).

- **`ipc_contract.rs`** khoá tham số một lệnh bằng cách đọc thẳng mã nguồn — khuôn
  `the_four_chapter_organise_wires_are_registered_and_keep_their_parameter_names` (`:769-803`).
  Mọi `MessageKey` phải có trong `vi.json` (`:231-257`) và khai đủ params (`:324+`).

- **Cổng:** **13** script `check:*` trong `package.json:14-27`, trong đó **12** là tệp `.mjs`; cái thứ
  mười ba là `check:lint` (chạy eslint, không có tệp script). Hai con số đo hai thứ khác nhau — không
  phải chỗ lệch. Thêm một cổng = sửa BA danh sách (`package.json` · `ci.yml` · `.githooks/pre-push`),
  `check:gates` canh cả ba.

---

## 6. Mockup

**`_bmad-output/planning-artifacts/ux-designs/ux-AuraTranslate-2026-08-02/mockups/web-import.html`**
(531 dòng) — mockup ĐÚNG cho 6.7:
- Bước 1 (`:174-216`): ô dán link đánh số dòng (`:181-189`); ngay dưới **hai con số**
  *"50 link · sẽ tạo 50 Chương · 1 domain"* (`:193`) + câu cam kết (`:194`); cột phải chọn đích
  (Tác phẩm mới / đã có), tên, ngôn ngữ nguồn (`:198-208`); nút *"Tải 50 link"* (`:214`).
  Ghi chú thiết kế `:219` giải thích vì sao hai con số phải khớp — *"màn hình tự tố cáo"*.
- Nhật ký domain (`:406-450`) — nội dung **Story 6.8**, không phải 6.7.
- 🔴 **Mockup KHÔNG vẽ trạng thái lỗi từng link.** Biến CSS `--error` khai ở `:11,155` nhưng **0 class
  nào dùng nó**. Đây đúng là khoảng trống mà phán quyết ② của Ice (§0) lấp vào — lượt lập spec phải
  tự dựng hình dạng đó, mockup không dạy.

⚠️ **`library-and-import.html` là mockup của Story 6.6**, không có ô URL nào (grep `url` rỗng).
Đừng lẫn hai tệp. `6-1-ban-do/` và `5-14-ban-do/` là thư mục **bàn đo**, không phải mockup.

---

## 7. Sổ nợ — sáu mục MỞ mang chủ Story 6.7

| Dòng | Nội dung một câu |
|---|---|
| `:9067-9093` | AC epic 6 *"ba đường chỉ khác ở bước đầu vào"* chưa đếm được vì đường URL chưa tồn tại; vế **"danh sách URL rỗng cần thông điệp lỗi rõ ở tầng trên"** vẫn mở |
| `:9159-9174` | `PipelineInput.encoding: &'static encoding_rs::Encoding` rò kiểu thư viện thứ ba ra bề mặt public `core/segment/` — chi phí ghép nối chỉ đo được khi 6.7 cũng dựng qua đường này (🟡 một phần) |
| `:9188-9193` | Nhánh `Confidence::SelfDeclared` qua `charset` của HTTP header / `<meta charset>` **chưa có nguồn nào để thử** vì `Fetcher` chưa tồn tại |
| `:9262-9296` | 🔴 `PipelineShape::Chapters` chọn MỘT bảng mã từ `chapters.first()` rồi áp lên MỌI đơn vị — URL A trả GBK, URL B trả UTF-8 thì **B bị mượn bảng mã của A**. Phải chọn: ① dò độc lập từng đơn vị, hay ② viết rõ quyết định "một bảng mã cho cả danh sách" vào doc-comment |
| `:9566-9599` | `count_in_chapter` == `count_in_import` mới là trùng hợp hình dạng; phần còn hở là **đường sản phẩm thật** đưa `Chapters(N>1)` vào xem trước (🟡 một phần) |
| `:9681-9707` | Story 6.6b — xem §0① |

**Không phải chủ 6.7, nhưng liên quan:** `:9176-9186` tỉ lệ dò `chardetng` GBK/Big5 thật (**chủ Ice**,
cần fixture `.txt` thật ở `6-1-ban-do/fixtures/encoding/`) · `:9049-9065` chi phí byte NFR6 của
`dom_smoothie` (**chủ Story 6.9**).

---

## 8. Ba quyết định còn MỞ mà lượt lập spec 6.7 phải mang cho Ice chốt

1. **`blocking` hay async cho `Fetcher`** — `Cargo.toml` chú thích thẳng rằng 6.7 tự quyết lại. Kho có
   **0 `async fn`** tự viết và **0 `Channel`**; khuôn nền hiện có là `std::thread::spawn`.
2. **Một bảng mã cho cả danh sách, hay dò độc lập từng link** — nợ `:9262-9296`, và đây là ca rỗng-im-lặng
   thật (B mượn bảng mã của A).
3. **"Thêm Chương vào Tác phẩm sẵn có"** — nửa AC này là hạ tầng mới hoàn toàn (§4). Ứng viên số một
   cho lượt kiểm đa-mục-tiêu: tách ra thành story riêng, hay giữ trong 6.7.
