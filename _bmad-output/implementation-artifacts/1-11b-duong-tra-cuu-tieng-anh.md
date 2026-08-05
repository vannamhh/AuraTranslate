---
baseline_commit: 5a68df78706fc1bc150240f5dadc5a6a57cf4ac4
---

# Story 1.11b: Đường tra cứu tiếng Anh

Status: done

**Epic:** 1 — Nền móng ứng dụng & Tra cứu ngoại tuyến tức thì
**Story key:** `1-11b-duong-tra-cuu-tieng-anh`
**Covers:** FR34 *(nửa tra cứu)* · FR40 *(qua tập khoá, ⛔ không qua stemming — AD-44 ③)* · NFR1 *(đo trên đường tiếng Anh)*
**Governed by:** **AD-44** *(mới, ADOPTED 2026-08-05)* · AD-26 · AD-19 · AD-10 · AD-2 · AD-25 · AD-27 · AD-17 *(ranh giới)* · AD-21
**Ngày tạo:** 2026-08-05

> ✅ **Mệnh đề CHẶN đã gỡ.** `epics.md:1480` còn ghi 🔴 *"CHẶN: cần một AD mới cho đường tra cứu tiếng Anh — chủ sở hữu Winston"*. **AD-44 đã vào `ARCHITECTURE-SPINE.md:571` ngày 2026-08-05** và đã qua Reviewer Gate *(`reviews/review-ad-44-2026-08-05.md` — 5 phát hiện, 2 nghiêm trọng, tất cả đã vá; `lint_spine.py` 0 findings)*. `deferred-work.md:276` ghi *"Story 1.11b ⛔ KHÔNG còn bị chặn bởi mục này."* **Dev bắt đầu được ngay.**

> 🟡 **`epics.md` ĐANG LỆCH khỏi AD-44 ở hai chỗ — ⛔ DEV KHÔNG SỬA `epics.md`. Chủ sở hữu: John (PM).**
> *(a)* `epics.md:1491` ghi *"biến thể hình thái (FR40, stemming) dùng `Matcher` của **AD-17**"* — **AD-44 ③ nay nói đường TỪ ĐIỂN ⛔ không gọi Matcher**, và ghi số đo làm lý do. **Story này theo AD-44, ⛔ không theo dòng đó của epics.**
> *(b)* Dòng 🔴 CHẶN nói trên nên gỡ.
> Chi tiết: `deferred-work.md:282`.

---

## Story

As a **người dịch**,
I want **bôi đen một từ tiếng Anh và thấy ngay nghĩa tiếng Việt kèm nhãn từ loại**,
So that **cặp Anh → Việt dùng được thật, ⛔ không chỉ có trong tài liệu**.

---

## Ranh giới phạm vi — ĐỌC TRƯỚC KHI GÕ DÒNG ĐẦU TIÊN

Story này là **một tầng chiến lược truy vấn trong `core/dict/`**. Nó ⛔ **không** phải một lượt dựng UI, ⛔ **không** phải một lượt dựng cổng, ⛔ **không** phải một lượt dựng dữ liệu.

| Thứ | Trong phạm vi? | Chủ sở hữu thật |
|---|---|---|
| Vị từ điều phối zh/en + hai nhánh truy vấn tiếng Anh trong `core/dict/` | ✅ **CÓ** | story này |
| Đọc `dict_sense` / `dict_example` / `dict_citation` *(nghĩa, ví dụ, trích dẫn)* | ⛔ **KHÔNG** | **Story 1.13** |
| Gom nhiều tệp `.db`, nhóm theo nguồn, cổng `DictionarySource` | ⛔ **KHÔNG** | **Story 1.13** |
| Bất kỳ dòng `.vue` / `.ts` nào, Panel Lookup, chuỗi *"truy vấn quá ngắn"* hiển thị | ⛔ **KHÔNG** | **Story 1.17** |
| `MessageKey` mới, khoá `vi.json` mới, lệnh IPC mới | ⛔ **KHÔNG** | **Story 1.13 / 1.17** |
| `Matcher`, `jieba-rs`, `tantivy-stemmers`, `core/matching/**` | ⛔ **KHÔNG** | **Story 1.12** *(và AD-44 ③ nói đường từ điển ⛔ không gọi nó)* |
| Chọn **cái gì** để tra *(một từ? một cụm?)*, ngưỡng bôi đen | ⛔ **KHÔNG** | **Story 1.18** *(Auto-Lookup)* |
| `tools/dict-build/**`, `schema.rs`, dựng lại bất kỳ tệp `.db` nào | ⛔ **KHÔNG** | đã xong ở 1.10b — ⛔ **không chạm** |

🔴 **FR19 nằm trong dòng `Covers:` của `epics.md:1483` nhưng FR19 = *"Panel Source + tab Hán Việt"* và chủ sở hữu thật là Story 1.16** *(`epics.md:639`, `epics.md:1672`)*. Story này ⛔ **không** dựng Panel Source. Tiền lệ đã đặt: **Story 1.11 giao 0 dòng frontend.** Story này cũng vậy.

### 🔴 Một AC của `epics.md` bị **THU HẸP CÓ CHỦ Ý** — đọc lý do, ⛔ đừng bỏ qua

`epics.md:1492` liệt cho story này: *"**And** mục từ tiếng Anh **hiển thị** nhãn từ loại + nghĩa tiếng Việt *(FR34)*, ghi rõ nguồn, ⛔ không hợp nhất *(AD-19)*."*

**Story này ⛔ KHÔNG giao vế đó, và đây là một lượt thu hẹp TƯỜNG MINH — ⛔ không phải một chỗ bỏ sót.** Ba lý do, cả ba truy được về tài liệu:

1. **Động từ là *"hiển thị"*.** Hiển thị là Panel Lookup — **Story 1.17** *(`epics.md:1706`)*. Story này giao **0 dòng frontend**, đúng tiền lệ Story 1.11.
2. **Đọc `dict_sense` đòi chốt hình dạng nhóm-theo-nguồn**, và `core/dict/mod.rs:78-80` *(Story 1.11 viết)* nói thẳng: *"đọc nghĩa là **Story 1.13** (FR29–FR32), và hình dạng của nó phụ thuộc vào quyết định nhóm-theo-nguồn mà story này ⛔ không được phép đoán trước."* AD-44 ⑤ siết thêm: ⛔ **không** bản ghi kết quả thứ hai cho tiếng Anh.
3. **`epics.md:1559-1561` giao đúng vế đó cho Story 1.13:** *"Given một mục từ tiếng Anh · When trả kết quả · Then có nhãn từ loại và nghĩa tiếng Việt."* ⇒ Hai story cùng mang một câu; **1.13 là chủ**, story này là **nền dữ liệu đường đi** cho nó.

**Story này giao thứ khiến vế đó khả thi:** một `EntryHit` tiếng Anh **đúng**, mang `entry_id` + `source_code`, tra ra được bằng cả `Running` lẫn `API` — 1.13 nối `dict_sense` vào đó. **Nếu Ice muốn kéo vế hiển thị vào đây, đó là một quyết định phạm vi, ⛔ không phải một lượt vá.**

🟡 **Panel Lookup CHƯA có hình dạng hiển thị cho mục từ tiếng Anh** — `EXPERIENCE.md`/`DESIGN.md` dựng quanh mục tiếng Trung *(tab Hán Việt, âm đọc, bộ thủ)*. Chủ sở hữu: **Sally (`bmad-ux`)**, ghi ở `deferred-work.md:289`. ⇒ **Đây là lý do THỨ HAI để story này ⛔ không chạm UI:** tự chế một hình dạng hiển thị ở tầng story chính là cách một bất nhất giao diện ra đời.

---

## Acceptance Criteria

### AC1 — Vị từ điều phối là **hình dạng chuỗi truy vấn**, và nó là một **hàm thuần, công khai, nhị phân**

**Given** một chuỗi truy vấn bất kỳ
**When** gọi `pick_route(query)`
**Then** trả `QueryRoute::Zh` nếu chuỗi chứa **bất kỳ** ký tự Hán nào; ⛔ ngược lại trả `QueryRoute::En`
**And** hàm ⛔ **không** chạm database — nghiệm thu được **không cần một tệp `.db` nào** *(điều kiện để ca này chạy trong CI, nơi ⛔ không có tệp từ điển — `.gitignore: *.db`)*
**And** vị từ **NHỊ PHÂN, ⛔ không có nhánh thứ ba**: chuỗi rỗng · toàn chữ số `"2026"` · toàn dấu câu `"..."` · một hệ chữ viết thứ ba `"Ελλάδα"` / `"日本語のひらがな"` *(kana thuần)* — **tất cả đi đường `En`**
**And** ⛔ **không** điều phối theo ngôn ngữ của Tác phẩm — bôi đen `API` trong một truyện tiếng Trung phải ra kết quả, ⛔ không ra rỗng *(AD-44 Prevents #2)*

> ⚠️ Ca `"日本語"` chứa kanji ⇒ `Zh`. Đó là **hành vi đúng theo AD-44** *(vị từ nói về **script**, ⛔ không nói về **ngôn ngữ**)*, ⛔ không phải một lỗi cần vá ở story này.

### AC2 — Định nghĩa *"ký tự Hán"* là **MỘT**, và món nợ hai-workspace được **đóng bằng một cổng kiểm chéo**

**Given** `src-tauri/src/core/dict/`
**When** rà mã
**Then** tồn tại **đúng một** `pub fn is_han(c: char) -> bool`, chép **nguyên văn bảy dải** của `tools/dict-build/src/char_idx.rs::is_han`:
`0x3400..=0x4DBF` · `0x4E00..=0x9FFF` · `0xF900..=0xFAFF` · `0x20000..=0x2A6DF` · `0x2A700..=0x2EBEF` · `0x2F800..=0x2FA1F` · `0x30000..=0x3134F`

**Given** hai workspace tách rời có chủ ý *(Story 1.9 AC4)* và ⛔ **không** có cổng kiểm chéo — món nợ ghi ở `review-ad-44-2026-08-05.md:49`, `ARCHITECTURE-SPINE.md:588`
**When** chạy bộ test
**Then** tồn tại một **cổng parity văn bản** khẳng định bảy dải trên có mặt **nguyên văn** trong `tools/dict-build/src/char_idx.rs`
**And** cổng dùng đúng khuôn đã có: `fixture_ddl_is_verbatim_from_dict_build_schema` *(`dict_lookup.rs`)* — đọc tệp workspace kia **dưới dạng văn bản** qua `env!("CARGO_MANIFEST_DIR")/../tools/dict-build/src/char_idx.rs`, ⛔ không import chéo crate
**And** cổng có **sàn quần thể** *(≥ 7 dải)* để một tệp bị cắt cụt ⛔ không đọc thành "đạt"

**Given** `src-tauri/tests/dict_lookup.rs:211` đang mang một **bản sao `is_han` CHỈ BMP (3 dải)**
**When** story này xong
**Then** bản sao đó **đã bị xoá**; test dùng `auratranslate_lib::core::dict::is_han`
**And** trong toàn bộ `src-tauri/**` chỉ còn **một** định nghĩa

> 🔴 **Vì sao đây là AC chứ ⛔ không phải "nice to have":** hai định nghĩa lệch nhau sẽ định tuyến một truy vấn sang đường tiếng Trung rồi tra vào một `char_idx` **chưa bao giờ lập chỉ mục ký tự đó** ⇒ **rỗng, ⛔ không lỗi** — đúng lớp lỗi AD-26 ra đời để chặn. Bản sao 3-dải trong test hôm nay **đã lệch thật** so với bản 7-dải của build tool. AD-44 kế thừa món nợ này chứ ⛔ không tạo ra nó; story này **đóng được nó với chi phí gần bằng 0** vì khuôn cổng parity đã có sẵn.

### AC3 — Vị từ chạy **ĐÚNG MỘT LẦN mỗi lượt tra**, và chạy **TRÊN** adapter *(AD-44 ①, vá A1)*

**Given** chữ ký `lookup`
**When** rà
**Then** `lookup` **nhận** `route: QueryRoute` **đã quyết** từ chỗ gọi
**And** `lookup` và `query.rs` ⛔ **không bao giờ** gọi `pick_route` — adapter ⛔ không tự phân xử lại một câu hỏi thuộc về **cả lượt tra**
**And** cưỡng chế bằng test tĩnh trong `dict_boundary.rs`: chuỗi `pick_route` xuất hiện ở **đúng một** tệp dưới `core/dict/**` *(là `mod.rs`, nơi khai nó)*, và ⛔ **không** xuất hiện ở vị trí mã của `query.rs`

> Để vị từ chạy **trong** adapter là để mỗi tệp `.db` tự trả lời một câu hỏi của cả lượt tra — và hai tệp sẽ trả lời **khác nhau** ngay khi định nghĩa `is_han` của chúng lệch. Tầng gom thật *(Story 1.13)* chưa tồn tại; `pick_route` được khai **công khai** chính là để 1.13 gọi nó **một lần** rồi truyền cùng một `route` xuống mọi tệp.

### AC4 — ⛔ **Không tồn tại sổ đăng ký *"tệp `.db` nào chứa ngôn ngữ nào"*** *(AD-44 ①, vá A2)*

**Given** toàn bộ mã story này giao
**When** rà
**Then** ⛔ **không** có bảng / map / hằng / enum nào ánh xạ đường dẫn tệp `.db` → ngôn ngữ
**And** **mọi** tệp đang gắn đều được tra; `lang` lọc **trong SQL**
**And** đối chứng dương trong `dict_boundary.rs`: văn bản gộp của `core/dict/**` chứa **cả** `lang = 'zh'` **lẫn** `lang = 'en'`

> Một sổ đăng ký là **nguồn sự thật thứ hai cho một dữ kiện đã nằm trong dữ liệu** — cùng lớp lỗi AD-8 và AD-33 tồn tại để chặn — và nó sai **im lặng** vào đúng ngày một lớp gỡ rời được thêm hay gỡ đi *(FR112)*.

### AC5 — Đường tiếng Anh có **HAI** nhánh, ⛔ không phải ba *(AD-44 ②)*

**Given** `pick_branch(query, mode, route)`
**When** `route == En`

| Chế độ | Độ dài *(ký tự)* | Nhánh trả về | Chỉ mục |
|---|---|---|---|
| `Exact` | bất kỳ | `QueryBranch::ExactBtree` | `idx_entry_headword` |
| `Substring` | **≥ 3** | `QueryBranch::FtsTrigram` | `entry_fts` *(`trigram`)* |
| `Substring` | **< 3** | 🔴 `QueryBranch::NoBranchQueryTooShort` | — *(⛔ không nhánh nào chạy)* |

**Then** ⛔ **không** nhánh `CharIdx` nào cho `route == En`
**And** phép đo độ dài là **`chars().count()`**, ⛔ **không bao giờ** `len()`

> 🔴 **Đo thật, ⛔ không suy đoán:** lớp `viwiktionary-en` sinh **đúng 9** cặp `char_idx` trên **119.039** đầu mục *(0,0076%; 0,00067% của tổng 1.341.179 cặp)*. Nhánh `char_idx` ⛔ **không áp được** cho tiếng Anh — đó là **dữ kiện**. SQL tái lập: `SELECT COUNT(*) FROM char_idx c JOIN dict_entry e ON e.id=c.entry_id JOIN dict_source s ON s.id=e.source_id WHERE s.code='viwiktionary-en';`

### AC6 — Nhánh tra chính xác tiếng Anh: tập khoá `{nguyên văn, hạ chữ thường}` trong **MỘT** truy vấn *(AD-44 ③)*

**Given** `LookupMode::Exact` trên `route == En`
**When** chạy
**Then** SQL dùng `e.headword IN (?1, ?2)` — **một** lượt qua B-tree, **một** truy vấn
**And** ⛔ **KHÔNG fallback dây chuyền** *(tra nguyên văn, rỗng thì tra lại dạng hạ chữ thường)* — nó làm mỗi lượt tra chạy hai truy vấn ⇒ số đo NFR1 mất nghĩa, và làm `LookupResult::branch` **nói dối** về đường đã đi
**And** phép hạ chữ thường tính **ở Rust** bằng `str::to_lowercase()`, ⛔ **không** bằng `lower()` của SQLite *(hàm dựng sẵn của SQLite chỉ hạ **ASCII** — nó ⛔ không chạm `É`, `Ü`, `Ø`, và một đầu mục tiếng Anh mượn từ nước ngoài sẽ rơi im lặng)*
**And** phép hạ chữ thường ⛔ **không phụ thuộc locale** — `"I".to_lowercase()` của Rust luôn ra `"i"` *(một phép fold theo locale làm **cùng một truy vấn cho hai kết quả trên hai máy** cài ngôn ngữ hệ điều hành khác nhau — một hồi quy ⛔ không tái lập được trên máy người sửa)*

**Given** đầu mục `running` *(chữ thường)* có trong dữ liệu
**When** tra `Running` ở chế độ `Exact`
**Then** trả kết quả **khác rỗng** — 🔴 **đây là lỗ mà AD-44 tồn tại để bịt**: đo thật `headword = 'running'` ⇒ **1** hàng; `headword = 'Running'` ⇒ **0** hàng. Bôi đen một từ ở **đầu câu** là thao tác thường ngày.

**Given** đầu mục `API` *(chữ hoa có nghĩa)* có trong dữ liệu
**When** tra `API` ở chế độ `Exact`
**Then** trả kết quả **khác rỗng** — hạ chữ thường là **THÊM** một khoá, ⛔ **không phải THAY** khoá gốc *(**1.635** đầu mục tiếng Anh mang chữ hoa có nghĩa: `API` · `Wikipedia` · `English`)*

**Given** cùng đầu mục `API`
**When** tra `api` *(chữ thường)* ở chế độ `Exact`
**Then** ⛔ **không** trả `API` — ⛔ **không** hạ chữ thường phía **đầu mục**
**And** ca này phải có **test riêng ghi lại tính bất đối xứng đó**, để một giai đoạn sau ⛔ không "sửa" nó bằng một chỉ mục hàm `lower(headword)` — thứ đó đòi đổi `schema.rs`, dựng lại `dict-core.db`, điền lại `[base].sha256`, đo lại NFR6, và làm **184** nhóm đầu mục *(chỉ phân biệt nhau bằng chữ hoa)* sập vào nhau

> ⚠️ **Hệ quả đã khai và đã chấp nhận có ý thức** *(`.memlog.md:169`)*: hạ chữ thường phía truy vấn làm tập kết quả **rộng hơn** ở đúng **184** chỗ. Chấp nhận được vì AD-19 vốn đã nói kết quả **giữ nguyên bất đồng** và **mang nhãn nguồn** — người dịch tự phán xét.

### AC7 — Chuỗi con **< 3 ký tự** tiếng Anh: **KHÔNG HỖ TRỢ**, và trạng thái đó **phân biệt được** với *"không có kết quả"* *(AD-44 ④)*

**Given** `LookupMode::Substring`, `route == En`, truy vấn **< 3 ký tự** *(gồm cả chuỗi rỗng)*
**When** gọi `lookup`
**Then** trả `LookupResult { branch: QueryBranch::NoBranchQueryTooShort, hits: vec![] }`
**And** ⛔ **không** chạm database *(⛔ không một câu SQL nào được chuẩn bị)*
**And** trạng thái đó **quan sát được từ ngoài** và **phân biệt được** với một lượt tra đã chạy mà ⛔ không tìm thấy gì
**And** ⛔ **không làm tràn** qua nhánh tra chính xác *(nhánh trả về sẽ nói dối)*
**And** ⛔ **không hạ ngưỡng trigram xuống 1** — đo thật: FTS5 `trigram` ⛔ **không** lập chỉ mục token ngắn hơn ba ký tự, `entry_fts MATCH '"山"'` ⇒ **0** hàng

> **Rỗng im lặng bị cấm; rỗng có lý do thì không.** Đây là chỗ tinh thần AD-26 được phát biểu **tổng quát**.

> 🔴 **Ca 0 ký tự — AD-44 ② để trống, story này CHỐT và ghi lý do.** Bảng của AD-44 phủ `≥ 3` và `1–2`; nó ⛔ không nói gì về `0`. **Chốt: `0` đi cùng đường với `1–2` ⇒ `NoBranchQueryTooShort`.** Lý do: vị từ độ dài là **một** mệnh đề `chars().count() < 3`, ⛔ không phải hai mệnh đề với một ca đặc biệt ở giữa; và một chuỗi rỗng **đúng là quá ngắn**. Điều này ⛔ **không** mâu thuẫn phần vá A3 *(*"một kết quả rỗng ở đường en là 'không có kết quả' thật"*)* — A3 nói về **vị từ ĐIỀU PHỐI** *(chuỗi rỗng vẫn đi đường `En`, ⛔ không sinh nhánh thứ ba)* và về những lượt tra **đã chạy một nhánh** mà ⛔ không tìm thấy gì.
> ⚠️ **Bất đối xứng có chủ ý với đường zh:** ở đường tiếng Trung, truy vấn rỗng trả `branch = CharIdx` với `hits` rỗng *(hành vi 1.11 hiện có, `query.rs::char_idx` trả sớm ⛔ không chạm DB)*. **⛔ Đừng "đồng bộ" hai bên** — hai bảng nhánh khác nhau vì hai chỉ mục khác nhau.
> 📌 **Nếu Winston ⛔ không đồng ý với cách chốt này, đây là chỗ để lật** — ghi vào Completion Notes, ⛔ đừng sửa AD.

### AC8 — **Mọi** nhánh tiếng Anh lọc `dict_entry.lang = 'en'` **tường minh trong SQL**

**Given** cả hai nhánh của đường tiếng Anh
**When** rà SQL
**Then** mỗi câu chứa `AND e.lang = 'en'` *(hoặc mệnh đề tương đương tường minh)*, ⛔ **không** giả định *"tệp này chỉ có tiếng Anh"*

**Given** một truy vấn chứa ký tự Hán, **ép** `route = En` *(chỗ gọi truyền vào, ⛔ không qua `pick_route`)*
**When** tra
**Then** trả **0 hàng** — đối chứng âm cưỡng chế bộ lọc `lang` của đường en
**And** đối chứng dương: fixture có ≥ 2 hàng `lang='zh'` mà truy vấn đó khớp khi `route = Zh`

> 🔴 **Đây là đúng lý do `route` phải là THAM SỐ chứ ⛔ không phải một phép đoán bên trong `lookup`:** với một tham số, test **ép được** một tổ hợp mà vị từ ⛔ không bao giờ sinh ra, và bộ lọc `lang` trở thành thứ nghiệm thu được thay vì thứ *"chắc là đúng vì đầu vào không bao giờ tới đó"*.

### AC9 — Đường tiếng Trung **⛔ KHÔNG đổi hành vi một chút nào**

**Given** toàn bộ **18** ca của `src-tauri/tests/dict_lookup.rs` và **3** ca của `dict_boundary.rs` mà Story 1.11 đã giao
**When** chạy lại
**Then** **tất cả xanh**
**And** thay đổi duy nhất được phép ở các ca cũ là **chỗ gọi** — thêm đối số `route` *(cơ học)*
**And** ⛔ **không** một câu SQL nào của đường zh bị sửa; ba nhánh zh giữ nguyên, bộ lọc `lang = 'zh'` giữ nguyên
**And** ca `every_branch_filters_out_english_entries` giữ nguyên ý nghĩa: truyền `QueryRoute::Zh` **tường minh** để chứng minh adapter zh vẫn loại hàng `lang='en'`

> `review-ad-44-2026-08-05.md:52`: *"AD-44 ⛔ không đòi sửa một dòng nào của Story 1.11 đã giao. Đường zh giữ nguyên ba nhánh và giữ nguyên bộ lọc `lang='zh'`; vị từ điều phối là một tầng **mới ở trên**, ⛔ không phải một lần viết lại."*

### AC10 — ⛔ **Không stemming, ⛔ không `Matcher`, ⛔ không crate mới** *(AD-44 ③ + AD-17 ranh giới)*

**Given** toàn bộ mã story này giao
**When** rà
**Then** ⛔ **không** một lời gọi nào tới `core::matching`, `tantivy_stemmers`, `jieba_rs`
**And** `src-tauri/Cargo.toml` ⛔ **không** thêm một dòng nào; `npm run check:deps` xanh *(326 crate · 104 gói npm giữ nguyên)*
**And** `core/matching/mod.rs` vẫn **0 dòng mã** — Story 1.12 mới là chủ của nó

> 🔴 **Dữ kiện MẠNH đứng sau quyết định này — đo trên `dict-core.db` thật:** corpus **đã có sẵn** mọi dạng biến thể làm **đầu mục riêng**. Mẫu thử **16/16** có mặt, **gồm cả bất quy tắc** `went` · `gone` · `children` · `happiest` — thứ stemming về nguyên tắc ⛔ **không bao giờ** làm được. Quy mô: **7.656** đầu mục `-ing` · **8.855** `-ed` · **19.616** `-s` · **228** `-est` trên **119.039**. ⇒ **Nhánh tra chính xác một mình đã phủ FR40 RỘNG HƠN thứ stemming phủ được.**
> *(Dữ kiện phụ, **yếu hơn** — ⛔ đừng trích như cái mạnh: ba dạng stem Porter kinh điển `dictionari` · `studi` · `happi` tra vào `dict-core.db` cho **0** hàng, `run` cho **1**. ⚠️ Số hàng là đo thật, nhưng **ba chuỗi stem đó chưa chạy qua stemmer mà sản phẩm sẽ dùng**.)*

### AC11 — Hình dạng bản ghi kết quả: **`lang` là một TRƯỜNG, ⛔ không phải một KIỂU** *(AD-44 ⑤)*

**Given** kiểu trả về của đường tra cứu
**When** rà
**Then** ⛔ **không tồn tại** một bản ghi kết quả thứ hai dành riêng cho tiếng Anh — `EntryHit` và `LookupResult` giữ nguyên hình dạng, dùng chung cho cả hai đường
**And** ⛔ **không** chạm `ports/` *(cổng `DictionarySource` là **Story 1.13**; adapter là một cho mỗi **tệp `.db`** theo AD-10, ⛔ **không bao giờ** một cho mỗi **ngôn ngữ**)*
**And** ⛔ **không** tồn tại bước hợp nhất `zh` với `en` ở bất kỳ đâu *(AD-19, mở rộng bởi AD-44 ⑤)*
**And** mọi hit vẫn mang `source_code: String` *(`dict_source.code`)*, ⛔ **không** `source_id: i64`

### AC12 — **NFR1 đo TRÊN đường tiếng Anh**, ⛔ không mượn số tiếng Trung *(AD-44 ⑥)*

**Given** ca bench `#[ignore]` đã có trong `dict_lookup.rs`
**When** mở rộng cho đường tiếng Anh
**Then** đo **p50 / p95 / p99** riêng cho: nhánh exact en *(truy vấn chữ thường)* · nhánh exact en *(truy vấn chữ HOA — tập khoá 2 phần tử)* · nhánh trigram en
**And** giữ nguyên khuôn đã có: `WARMUP = 10`, `RUNS = 200`, percentile **nearest-rank** *(⛔ không nội suy)*, `assert!(worst_p95 <= CEILING_MS)` với `CEILING_MS = 10.0`
**And** giữ nguyên **hai lớp chặn**: `#[ignore]` **và** vắng `AURA_DICT_BENCH_DB` ⇒ `return` sớm — ⛔ **không** ngưỡng thời gian nào chạy trong CI *(CI ⛔ không có tệp `.db` thật)*
**And** số đo **thật** ghi vào **Completion Notes**, kèm **bản đã đo** *(release hay debug)*

> 🔴 **Trần 10 ms là DẪN XUẤT, ⛔ không phải phát minh, và ⛔ không phải mệnh đề spine.** `ARCHITECTURE-SPINE.md` ⛔ **không** phát biểu số ngân sách NFR1 ở bất kỳ đâu. Xuất xứ: `prd.md:814` cho **p95 < 100 ms đầu-cuối** và ghi *"toàn bộ phần còn lại (~99,95 ms) dành cho vòng IPC Tauri và render frontend"*; Story 1.11 *(`:166`)* dẫn xuất trần **10 ms** để giữ lại **≥ 90 ms** cho hai thứ chưa ai đo *(giả định `[A1]`)*. **Trích xuất xứ này, ⛔ đừng trích spine.**
> ⚠️ **Bản debug chậm ~2× bản release** *(nhánh 2 zh: 15,045 ms debug vs 7,324 ms release — bản debug **VƯỢT** trần)*. Đó là số mọi dev sẽ thấy khi chạy không có `--release`. **Nói rõ bản nào khi ghi số.**
> ⚠️ **⛔ Đừng trích dải hiệu năng cũ của AD-26** *(0,02 ms · 0,15–4,5 ms · 0,13–0,19 ms)* — spine **đã đánh dấu LỖI THỜI** trong cùng lượt vá AD-44.

### AC13 — Đường tra cứu nóng **⛔ không quét bảng**

**Given** mọi mã mới dưới `src-tauri/src/core/dict/**`
**When** `dict_boundary.rs` chạy
**Then** ⛔ **không** một vị trí mã nào chứa `LIKE`, `GLOB`, hay `instr(` — cả ba nhánh cũ lẫn hai nhánh mới
**And** đối chứng dương giữ nguyên: văn bản gộp của `core/dict/**` vẫn chứa `char_idx`, `entry_fts`, `INTERSECT`
**And** mọi truy vấn dùng **tham số ràng buộc**; `format!` chỉ được nội suy **hằng** *(`COLUMNS`, `JOIN_SOURCE`)*, ⛔ **không bao giờ** dữ liệu người dùng

### AC14 — Ranh giới ⛔ KHÔNG CHẠM

**Given** danh sách tệp story này giao
**When** đối chiếu
**Then** ⛔ **không** một dòng `.vue` / `.ts` / `.css` nào
**And** ⛔ **không** `MessageKey` mới, ⛔ **không** khoá `vi.json` mới — `npm run check:i18n` vẫn báo đúng **16 khoá · 9 placeholder**
**And** ⛔ **không** chạm `tools/dict-build/**`, ⛔ **không** đổi một dòng DDL nào, ⛔ **không** dựng lại một tệp `.db` nào *(🔴 `cargo run --manifest-path tools/dict-build/Cargo.toml` mặc định là `--layer all` và nó dựng lại **CẢ BA** tệp ⇒ hai `sha256` trong `dict-manifest.toml` thành sai, và ⛔ **không cổng nào bắt được** — `check-dict-manifest.mjs` cố ý ⛔ không đọc `.db`)*
**And** ⛔ **không** chạm `src-tauri/src/core/store/**` *(đường mở tệp chỉ-đọc đã đủ dùng: `ReadOnlyDb` + `apply_dict_reader_pragmas`)*
**And** ⛔ **không** chạm `ports/`, `commands/`, `core/matching/`, `core/scope/`, `core/i18n/`

### AC15 — Mọi cổng xanh, ⛔ không hạ một sàn nào

**Given** cây mã sau story
**When** chạy toàn bộ
**Then** `npm run build` → `cargo test --locked --manifest-path src-tauri/Cargo.toml` **xanh**
**And** sáu cổng `.mjs` **xanh**: `check:deps` · `check:i18n` · `check:commands` · `check:tokens` · `check:dict` · `check:dict-manifest`
**And** ⛔ **không** hạ / nới một sàn quần thể nào: `store_boundary::RS_FLOOR = 20` · `scope_boundary::RS_FLOOR = 20` · `check-i18n::RS_FLOOR = 21` / `VUE_FLOOR = 1` · `dict_boundary::DICT_FLOOR = 1` · `check-dict-build::RS_FILE_FLOOR = 21`
**And** ⛔ **không** nới `STORE_DIR` / `FORBIDDEN` của `store_boundary.rs` — `core/dict/**` ⛔ **không** được gõ `rusqlite` hay `Connection::open`

---

## Tasks / Subtasks

- [x] **Task 1 — Vị từ điều phối và `is_han` một-nguồn-sự-thật** *(AC1, AC2)*
  - [x] 1.1 Thêm `pub fn is_han(c: char) -> bool` vào `src-tauri/src/core/dict/` với **bảy dải chép nguyên văn** từ `tools/dict-build/src/char_idx.rs:14-25`
  - [x] 1.2 Thêm `pub enum QueryRoute { Zh, En }` và `pub fn pick_route(query: &str) -> QueryRoute` *(hàm thuần, ⛔ không chạm DB)*
  - [x] 1.3 Xoá bản sao `fn is_han` chỉ-BMP ở `src-tauri/tests/dict_lookup.rs:211-214`; mọi chỗ dùng chuyển sang `auratranslate_lib::core::dict::is_han`
  - [x] 1.4 Viết cổng parity văn bản `han_ranges_are_verbatim_from_dict_build_char_idx` theo khuôn `fixture_ddl_is_verbatim_from_dict_build_schema` *(đọc tệp workspace kia dạng văn bản + sàn ≥ 7 dải)*
  - [x] 1.5 Test `pick_route`: Hán ⇒ `Zh`; rỗng · `"2026"` · `"..."` · `"Ελλάδα"` · `"ひらがな"` · `"API"` ⇒ `En`; `"中國API"` ⇒ `Zh` *(**bất kỳ** ký tự Hán nào)*; ngoài-BMP `"𠧜"` ⇒ `Zh` *(ca này chỉ đạt nếu dùng bản 7 dải — nó là **đối chứng sống** cho AC2)*

- [x] **Task 2 — Đưa `route` thành tham số, thêm nhánh thứ tư của `QueryBranch`** *(AC3, AC5, AC7, AC9)*
  - [x] 2.1 Thêm biến thể `QueryBranch::NoBranchQueryTooShort` kèm doc-comment nêu AD-44 ④
  - [x] 2.2 Đổi `pick_branch(query, mode)` → `pick_branch(query, mode, route)`; bảng nhánh theo AC5
  - [x] 2.3 Đổi `lookup(db, query, mode)` → `lookup(db, query, mode, route)`; nhánh `NoBranchQueryTooShort` trả sớm, ⛔ **không** chạm DB
  - [x] 2.4 Cập nhật **chỗ gọi** trong 18 ca của `dict_lookup.rs` — thêm đối số, ⛔ **không** đổi ý nghĩa ca nào
  - [x] 2.5 `every_branch_filters_out_english_entries`: truyền `QueryRoute::Zh` **tường minh**
  - [x] 2.6 Chạy lại toàn bộ → **21 ca cũ xanh** trước khi viết một dòng SQL tiếng Anh nào

- [x] **Task 3 — Hai nhánh SQL tiếng Anh trong `query.rs`** *(AC5, AC6, AC8, AC13)*
  - [x] 3.1 `pub(super) fn exact_en(db, query)` — `WHERE e.headword IN (?1, ?2) AND e.lang = 'en' ORDER BY e.id`, hai tham số `{query, query.to_lowercase()}`, **một** truy vấn
  - [x] 3.2 `pub(super) fn fts_trigram_en(db, query)` — cùng khuôn `fts_trigram` zh, đổi bộ lọc thành `e.lang = 'en'`, **dùng lại** hàm bọc cụm `format!("\"{}\"", query.replace('"', "\"\""))`
  - [x] 3.3 **Dùng lại** `run()`, `row_to_hit()`, `verify_substring()`, `COLUMNS`, `JOIN_SOURCE` — ⛔ **không** viết bản thứ hai của bất kỳ hàm nào
  - [x] 3.4 Nhánh trigram en **vẫn** đi qua `verify_substring` *(vế `headword_simp` sẽ luôn `None` với tiếng Anh — ⛔ **đừng** bỏ hàm vì thế; nó là hàng rào chống dương tính giả của trigram)*
  - [x] 3.5 Cả hai hàm ở **trong `query.rs`**, ⛔ **không** tạo `query_en.rs` *(Consistency Conventions: một module cho một khái niệm miền; helper dùng chung ở cùng tệp thì ⛔ không cần `pub(super)` thêm)*

- [x] **Task 4 — Cổng ranh giới** *(AC3, AC4, AC13)*
  - [x] 4.1 `dict_boundary.rs`: ca mới — `pick_route` xuất hiện ở **đúng một** tệp dưới `core/dict/**`, ⛔ **không** ở vị trí mã của `query.rs`
  - [x] 4.2 `dict_boundary.rs`: mở rộng đối chứng dương — văn bản gộp chứa **cả** `lang = 'zh'` **lẫn** `lang = 'en'`
  - [x] 4.3 Kiểm lại `FORBIDDEN = ["LIKE","GLOB","instr("]` vẫn xanh trên mã mới; ⛔ **không** nới danh sách
  - [x] 4.4 ⛔ **Không** nâng `DICT_FLOOR` *(sàn bắt cây **bị cắt**, ⛔ không bắt việc thêm tệp)*

- [x] **Task 5 — Fixture và ca hành vi tiếng Anh** *(AC6, AC7, AC8)*
  - [x] 5.1 Bổ sung `SEEDS` *(hiện 8 hàng)*: `running` `lang='en'` · `API` `lang='en'` · giữ nguyên `lock` (id 7) và `dictionary` (id 8)
  - [x] 5.2 Sau khi nạp `dict_entry` **bắt buộc** chạy `INSERT INTO entry_fts(entry_fts) VALUES('rebuild');` *(FTS5 external-content ⛔ không tự đầy)*
  - [x] 5.3 Ca: `Running` / `Exact` / `En` ⇒ khớp `running` — **lỗ chữ HOA đã bịt**
  - [x] 5.4 Ca: `API` / `Exact` / `En` ⇒ khớp `API` — khoá gốc **được giữ**
  - [x] 5.5 Ca: `api` / `Exact` / `En` ⇒ ⛔ **không** khớp `API` — bất đối xứng có chủ ý, ghi lý do trong doc-comment của ca
  - [x] 5.6 Ca: `dic` / `Substring` / `En` ⇒ khớp `dictionary` *(nhánh trigram)*
  - [x] 5.7 Ca: `lo` và `l` và `""` / `Substring` / `En` ⇒ `branch == NoBranchQueryTooShort`, `hits` rỗng
  - [x] 5.8 Ca đối chứng âm: truy vấn Hán, **ép** `route = En` ⇒ **0 hàng**; cùng truy vấn với `route = Zh` ⇒ **> 0 hàng**
  - [x] 5.9 Ca cú pháp FTS5 cho tiếng Anh: `don't` · `state-of-the-art` · `a*b` · `NEAR foo` · `x(y):z` ⇒ đều `Ok`, ⛔ không `SQLITE_ERROR`

- [x] **Task 6 — Đo NFR1 trên đường tiếng Anh** *(AC12)*
  - [x] 6.1 Mở rộng ca `#[ignore]` với ba tổ hợp tiếng Anh *(exact chữ thường · exact chữ HOA · trigram)*
  - [x] 6.2 Chạy **bản release** trên `dict-core.db` thật *(đường dẫn **TUYỆT ĐỐI** — CWD của test là `src-tauri/`)*
  - [x] 6.3 Ghi p50/p95/p99 **thật** + số hàng thật vào Completion Notes; đối chiếu trần **10 ms**
  - [x] 6.4 Nếu **vượt trần**: ⛔ **không** tự sửa bằng `LIMIT` — ghi số, nêu nguyên nhân *(số **HÀNG**, ⛔ không phải chỉ mục)*, và bàn giao cho **1.13/1.17** *(cùng cách Ice đã chốt cho nhánh 2 zh: "chấp nhận nguyên trạng, không sửa bây giờ")*

- [x] **Task 7 — Cổng cuối** *(AC15)*
  - [x] 7.1 `npm run build` *(**BẮT BUỘC TRƯỚC** `cargo test` — `generate_context!` nhúng `dist/` lúc biên dịch)*
  - [x] 7.2 `cargo test --locked --manifest-path src-tauri/Cargo.toml`
  - [x] 7.3 Sáu cổng `.mjs`
  - [x] 7.4 Điền **File List** và **Completion Notes**

### Review Findings

- [x] [Review][Patch] Tra cứu chuỗi con tiếng Anh đánh rơi im lặng kết quả khác chữ hoa/thường — `verify_substring()` so khớp phân biệt hoa/thường (`str::contains`) trong khi tokenizer `trigram` của FTS5 không phân biệt (đo thật bằng `sqlite3`: `entry_fts MATCH '"api"'` khớp hàng `headword='API'`, nhưng `"API".contains("api")` là `false` ⇒ bị lọc mất). Đúng lớp lỗi AD-26 cấm — rỗng im lặng — tái xuất hiện ở đường tiếng Anh. Chưa ca test nào phủ một truy vấn chuỗi con lệch hoa/thường. **Đã vá:** hạ chữ thường cả hai vế trong `verify_substring` (`src-tauri/src/core/dict/query.rs:89`), cùng tinh thần Rust-side lowercase của AD-44 ③; vô hại với đường zh vì chữ Hán không phân biệt hoa/thường. Thêm ca hồi quy `an_english_substring_query_matches_a_headword_of_different_case` (`dict_lookup.rs`). `cargo test` + sáu cổng `.mjs` xanh sau vá.
- [x] [Review][Patch] Cổng cấm `LIKE`/`GLOB`/`instr(` của `dict_boundary.rs` so khớp phân biệt hoa/thường, trong khi từ khoá SQLite thì không — một `like`/`Like`/`INSTR(` viết khác hoa/thường sẽ chạy quét toàn bảng thật (đúng hồi quy 134×/11× mà cổng này sinh ra để chặn) mà cổng vẫn xanh. **Đã vá:** `contains_forbidden_token` so khớp trên bản `to_ascii_uppercase()` của cả `code` lẫn `needle` (`src-tauri/tests/dict_boundary.rs:57`). `cargo test` xanh sau vá (không đổi kết quả 5/5 ca — không token cấm nào tồn tại thật trong mã hôm nay, dù hoa hay thường).
- [x] [Review][Patch] Doc-comment của `run()` ghi "ba nhánh dùng đúng bốn câu SQL hằng" — sau lượt này có năm nhánh và sáu hình dạng SQL khác nhau, câu đã lỗi thời. **Đã vá:** cập nhật doc-comment (`src-tauri/src/core/dict/query.rs:67`).
- [x] [Review][Defer] Điều kiện tiên quyết `≤ 2 ký tự` của `char_idx()` chỉ cưỡng chế bằng `debug_assert!` — vô tác dụng ở bản release; một lượt gọi trực tiếp trong tương lai với truy vấn dài hơn sẽ âm thầm cắt còn hai ký tự đầu. [src-tauri/src/core/dict/query.rs:138] — deferred, pre-existing (kế thừa từ Story 1.11, story này không đổi hàm)
- [x] [Review][Defer] Ngưỡng độ dài đếm bằng `chars().count()` — đếm code point Unicode, không đếm cụm ký tự hiển thị (grapheme cluster); văn bản chuẩn hoá NFD (vd. clipboard macOS với dấu tổ hợp) có thể đẩy một truy vấn qua lằn ranh nhánh sai lệch so với số ký tự người dùng cảm nhận đã gõ. [src-tauri/src/core/dict/mod.rs:242] — deferred, pre-existing (phép đo kế thừa nguyên xi từ ngưỡng zh của Story 1.11)

---

## Dev Notes

### 🎯 Chữ ký MỤC TIÊU — chép được, ⛔ đừng phát minh lại

Đây là hình dạng đã được suy ra từ AD-44 *(gồm cả bốn chỗ vá của Reviewer Gate)* và từ mã đang có. Doc-comment viết thêm theo phong cách sẵn có của `mod.rs`.

```rust
// src-tauri/src/core/dict/mod.rs

/// Đường tra cứu — **đã quyết ở tầng trên**, adapter ⛔ không tự quyết lại (AD-44 ①).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum QueryRoute {
    /// Truy vấn chứa ít nhất một ký tự Hán ⇒ ba nhánh của AD-26, lọc `lang = 'zh'`.
    Zh,
    /// Mọi thứ còn lại ⇒ hai nhánh của AD-44 ②, lọc `lang = 'en'`.
    En,
}

/// Vị từ điều phối — **hình dạng CHUỖI TRUY VẤN**, ⛔ không phải ngôn ngữ của Tác phẩm.
/// Hàm thuần, ⛔ không chạm database. **NHỊ PHÂN, ⛔ không có nhánh thứ ba.**
///
/// 🔴 Gọi **ĐÚNG MỘT LẦN cho mỗi lượt tra**, ở tầng gom (Story 1.13) — ⛔ không bên trong
/// [`lookup`], ⛔ không bên trong `query.rs`.
pub fn pick_route(query: &str) -> QueryRoute {
    if query.chars().any(is_han) { QueryRoute::Zh } else { QueryRoute::En }
}

/// Bảy dải CJK — **chép nguyên văn** `tools/dict-build/src/char_idx.rs::is_han`.
/// Cổng parity văn bản trong `tests/dict_lookup.rs` giữ hai bản ⛔ không trôi khỏi nhau.
pub fn is_han(c: char) -> bool { /* 7 dải, xem AC2 */ }

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum QueryBranch {
    ExactBtree,
    CharIdx,
    FtsTrigram,
    /// 🔴 ⛔ **Không nhánh nào chạy** — chuỗi con tiếng Anh < 3 ký tự (AD-44 ④).
    /// ⛔ Không phải "không có kết quả": nó là một trạng thái **không hỗ trợ**, và
    /// Panel Lookup (FR41, Story 1.17) nói *"truy vấn quá ngắn"*.
    NoBranchQueryTooShort,
}

pub fn pick_branch(query: &str, mode: LookupMode, route: QueryRoute) -> QueryBranch;

pub fn lookup(
    db: ReadHandle<'_>,
    query: &str,
    mode: LookupMode,
    route: QueryRoute,
) -> SqlResult<LookupResult>;
```

**Vì sao thêm biến thể vào `QueryBranch` thay vì bọc `LookupResult` trong một enum mới:** AD-44 ⑤ cấm *"một bản ghi kết quả thứ hai dành riêng cho tiếng Anh"*. Một `enum LookupOutcome { Ran(..), TooShort }` là đúng thứ đó về hình dạng, và nó buộc **mọi** chỗ gọi zh phải bóc thêm một lớp — tức sửa đường zh, thứ `review-ad-44:52` nói AD-44 ⛔ không đòi. Một biến thể thứ tư giữ **một** hình dạng bản ghi, giữ trạng thái **quan sát được từ ngoài**, và ⛔ không nói dối *(nó khai thẳng: ⛔ không nhánh nào chạy)*.

**Vì sao `route` là tham số của `lookup`:** ba lý do, cả ba đều cưỡng chế được — *(1)* AD-44 vá A1: vị từ chạy **trên** adapter; *(2)* Story 1.13 gọi `lookup` **một lần cho mỗi tệp** và phải truyền **cùng một** `route` xuống mọi tệp, ⛔ không để mỗi tệp tự tính; *(3)* test **ép được** tổ hợp `(truy vấn Hán, route = En)` mà vị từ ⛔ không bao giờ sinh — đó là cách bộ lọc `lang` của đường en trở thành thứ **nghiệm thu được** *(AC8)*.

### 📐 SQL mục tiêu — hai câu, dùng lại toàn bộ hạ tầng đã có

```sql
-- exact_en: MỘT truy vấn, tập khoá {nguyên văn, hạ chữ thường}
SELECT e.id, s.code, e.lang, e.headword, e.headword_simp
FROM dict_entry e JOIN dict_source s ON s.id = e.source_id
WHERE e.headword IN (?1, ?2) AND e.lang = 'en'
ORDER BY e.id
-- params: &[&query, &query.to_lowercase()]
-- ⛔ KHÔNG headword_simp (tiếng Anh luôn NULL) · ⛔ KHÔNG lower() của SQLite (chỉ ASCII)
-- ⛔ KHÔNG UNION ALL (sinh trùng khi hai khoá cùng khớp — `IN` trả mỗi hàng đúng một lần)

-- fts_trigram_en: khuôn y hệt nhánh 3 của zh, đổi bộ lọc lang
SELECT e.id, s.code, e.lang, e.headword, e.headword_simp
FROM entry_fts f
JOIN dict_entry e ON e.id = f.rowid
JOIN dict_source s ON s.id = e.source_id
WHERE entry_fts MATCH ?1 AND e.lang = 'en'
ORDER BY e.id
-- param: format!("\"{}\"", query.replace('"', "\"\"")) — rồi verify_substring() ở Rust
```

⚠️ **`e.headword = ?1 OR e.headword = ?2` và `e.headword IN (?1, ?2)` ⛔ không tương đương về kế hoạch** trên mọi phiên bản SQLite. AD-44 ③ khai đích danh `IN (?1, ?2)` — **dùng đúng hình dạng đó**, và kiểm bằng `EXPLAIN QUERY PLAN` khi chạy bench *(⚠️ với bảng ảo FTS5, kế hoạch **luôn** chứa chữ `SCAN`; phần mang nghĩa là hậu tố **`:M1`** — ⛔ đừng đọc `SCAN` thành vi phạm)*.

### 🗄️ Dữ liệu tiếng Anh — nó nằm ở đâu và nó có hình dạng gì

*(Story 1.10b đã giao; ⛔ không phải dựng lại gì.)*

| Dữ kiện | Giá trị **đo thật** |
|---|---|
| Tệp | **`dict-core.db`** — nguồn **NỀN thứ SÁU**, ⛔ **không** phải một lớp gỡ rời |
| `dict_source.code` | **`viwiktionary-en`** *(`id = 6`)* — ⛔ **không** phải `viwiktionary` *(đó là vai B, `lang='zh'`, **1.598** hàng)* |
| `dict_entry.lang` | **`'en'`** trên **100%** hàng của nguồn này; **0** hàng `zh` |
| Đầu mục / nghĩa / ví dụ | **119.039** / **190.543** / **27.396** |
| `dict_entry.reading` | **`NULL` trên toàn bộ 119.039 mục** *(IPA có trong nguồn nhưng ⛔ không bóc — Quyết định #5 của 1.10b)* |
| `dict_entry.han_viet` · `headword_simp` | **⛔ không có** |
| `dict_sense.pos` / `pos_lang` | `"Danh từ"` · `"Động từ"` … / **`'vi'`** ⇒ FR34 *(nhãn từ loại + nghĩa tiếng Việt)* đã có nền dữ liệu |
| Đầu mục chữ HOA có nghĩa | **1.635** |
| Nhóm chỉ khác nhau bằng chữ hoa | **184** |
| Đầu mục chứa **dấu cách** *(cụm từ)* | **8.283** — đi nhánh exact như mọi đầu mục khác, ⛔ không cần nhánh riêng *(Deferred → Story 1.18)* |
| `char_idx` do lớp EN sinh | **9** / 1.341.179 |
| Kích thước `dict-core.db` | **194.998.272 byte** |

🔴 **`entry_fts` lập chỉ mục trigram trên `headword` của MỌI hàng — cả zh lẫn en.** Đo thật: `entry_fts MATCH '"dic"'` ⇒ **572** hàng, **100% `lang='en'`**. Rò rỉ theo chiều ngược *(trigram Latin khớp headword Hán)* đo được là **0**, nhưng ⛔ **đừng** dựa vào đó — `deferred-work.md:285` ghi mục này là **🔴 vẫn mở cho 1.11b**: *"Đường tra cứu **PHẢI** lọc theo `dict_entry.lang` — ⛔ KHÔNG được giả định mọi hàng là `zh`."* Với đường en, mệnh đề đảo chiều nhưng cùng luật.

### 🪤 Mười bẫy kế thừa — **mỗi cái đã cắn một lần rồi**

1. 🔴 **`chars().count()`, ⛔ KHÔNG `len()`.** Bẫy đắt nhất của 1.11, đã kiểm chứng bằng đột biến *(5 ca đỏ)*. `"山".len()` là **3**, `"中國".len()` là **6**. Ngưỡng `< 3` của đường en dùng **cùng** phép đo.
2. 🔴 **FTS5 `trigram` ⛔ không lập chỉ mục token < 3 ký tự** — đây chính là lý do AD-44 ④ khai 1–2 ký tự là *"không hỗ trợ"* thay vì để nó rỗng im lặng.
3. 🔴 **`entry_fts MATCH` phải bọc ngoặc kép + nhân đôi `"`.** ⚠️ **Rủi ro CAO HƠN NHIỀU với tiếng Anh** — truy vấn Latin dễ chứa `'`, `-`, `*`, `:` *(`don't`, `state-of-the-art`)*. Không bọc ⇒ SQLite trả `SQLITE_ERROR`. Khuôn ca test đã có: `an_fts_query_with_syntax_characters_does_not_error`.
4. 🔴 **Xác minh chuỗi con phải chạy ở Rust** *(`verify_substring`)* cho **cả** nhánh trigram — đo được: `中國` 390 → 350 *(40 dương tính giả)*. **Dùng lại cùng hàm**, ⛔ không viết bản thứ hai.
5. 🔴 **Mở tệp CHỈ ĐỌC.** `OpenFlags::default()` = `READ_WRITE|CREATE|NO_MUTEX|URI` — cờ `CREATE` biến một đường dẫn sai thành **tệp rỗng**, và mọi truy vấn sau đó trả rỗng **⛔ không lỗi**. Đường đúng đã có: `ReadOnlyDb::open` → `open_readonly_connection` *(`SQLITE_OPEN_READ_ONLY | SQLITE_OPEN_NO_MUTEX`)*.
6. 🔴 **⛔ KHÔNG tái dùng `apply_reader_pragmas`** — nó gọi `verify_wal`, mà ba tệp từ điển ở `journal_mode = delete` ⇒ đỏ ngay. Dùng `apply_dict_reader_pragmas` *(chỉ `busy_timeout` + `query_only=1`)*. ⛔ **KHÔNG** đặt WAL: ghi vào tệp ⇒ SHA-256 đổi ⇒ `dict-manifest.toml` sai ⇒ **AD-25 vỡ**.
7. 🔴 **⛔ KHÔNG nới `STORE_DIR`/`FORBIDDEN` của `store_boundary.rs`** — `core/dict/**` ⛔ không được gõ `rusqlite` / `Connection::open`. Viết truy vấn qua kiểu **tái xuất** của `core::store`: `ReadHandle`, `SqlResult`, `Row`, `ToSql`.
8. 🔴 **Khoá theo `dict_source.code`, ⛔ không theo `id`** — `id = 1` tồn tại ở **cả ba** tệp và trỏ ba nguồn khác nhau.
9. 🟡 **⛔ Chuỗi tiếng Việt CÓ DẤU ở vị trí mã bị `check-i18n.mjs` Kiểm A bắt.** `src/core/dict/**` ⛔ **không** nằm trong `EXEMPT`; chỉ `src-tauri/tests/**` được miễn. **Doc-comment và comment có dấu thì hợp lệ**; `panic!` / `debug_assert!` / `format!` / `Display` thì **không**. *(Đã bắt đúng ca này ở `core/i18n/mod.rs` và ở một **tên test** của 1.10b.)*
10. 🟡 **⛔ Đừng dùng `tempfile`** trong `src-tauri` — nó là dev-dep của `tools/dict-build`, ⛔ không của `src-tauri` *(`src-tauri/Cargo.toml` ⛔ **không có** `[dev-dependencies]`)*. Khuôn thư mục tạm đã có: `temp_dir(tag)` = `std::env::temp_dir()` + pid + `AtomicU64`.

### 🧾 Trạng thái baseline — biết trước để ⛔ không hoảng

- 🟡 **`cargo fmt --check` ĐỎ từ baseline** ở ba chỗ: `core/i18n/mod.rs:239`, `core/scope/resolve.rs:267`, `core/store/pragmas.rs:96`. **CI ⛔ không chạy `fmt` cũng ⛔ không chạy `clippy`** *(kiểm trên `.github/workflows/ci.yml`: 0 hit)*. ⇒ ⛔ **Đừng** chạy `cargo fmt` toàn cây và ⛔ đừng nhét ba chỗ đó vào diff của story này. Tệp/hàm **mới** thì viết cho fmt sạch.
- 🟡 `core/store/mod.rs:119` doc-comment ghi *"Năm loại kho"* trong khi `StoreKind` có **4** biến thể — lệch có **từ trước**, ⛔ **không** sửa ở story này.
- 🟡 **NFR1 nhánh 2 zh còn 27% dư địa** *(7,324 ms / trần 10 ms)*. Chi phí nằm ở **số HÀNG** *(3.177 hàng × 4 chuỗi cấp phát)*, ⛔ không sửa được bằng chỉ mục. **Ice đã chốt: chấp nhận nguyên trạng, ⛔ không sửa bây giờ** — đường ra là phân trang/`LIMIT`, quyết định sản phẩm của **1.13/1.17**. ⇒ Đường en đối mặt **cùng lớp vấn đề** *(`entry_fts MATCH '"dic"'` ⇒ 572 hàng)*. Nếu bench en vượt trần: **ghi số và bàn giao**, ⛔ **đừng** tự thêm `LIMIT`.
- 🟡 **⛔ Không có giới hạn trên cho độ dài truy vấn** — validate thuộc tầng IPC/UI *(1.13/1.17)*, ⛔ không phải story này.
- 🟡 **⛔ Không có cổng CI cho `EXPLAIN QUERY PLAN`** — CI ⛔ không có tệp `.db` thật. Cùng ràng buộc đã chấp nhận ở AC9 của 1.11.

### 📂 Trạng thái mã HÔM NAY — đọc trước khi sửa

`src-tauri/src/core/dict/` có **đúng hai tệp**:

| Tệp | Hôm nay | Story này đổi gì |
|---|---|---|
| `mod.rs` *(165 dòng)* | `LookupMode` · `QueryBranch` *(3 biến thể)* · `EntryHit` · `LookupResult` · `pick_branch(query, mode)` · `lookup(db, query, mode)`. Doc-comment `EntryHit.lang` **đã báo trước**: *"Trên đường này luôn là `zh` … một hằng ngầm ở chỗ gọi là thứ **1.11b** sẽ phải gỡ."* | **+** `QueryRoute` · `pick_route` · `is_han` · biến thể `NoBranchQueryTooShort`; **sửa** chữ ký `pick_branch` / `lookup`; **cập nhật** doc-comment module *(bảng nhánh nay có hai đường)* và doc-comment `EntryHit.lang` |
| `query.rs` *(186 dòng)* | `COLUMNS` · `JOIN_SOURCE` · `row_to_hit` · `run` *(dùng `prepare_cached`)* · `verify_substring` · `exact` · `char_idx` · `fts_trigram` — **cả bốn câu SQL viết cứng `e.lang = 'zh'`** | **+** `exact_en` · `fts_trigram_en`; **⛔ không đổi** một câu SQL zh nào |

`query.rs:116-120` **đã ghi sẵn** giới hạn mà story này gỡ: *"đường tra cứu tiếng Anh là **Story 1.11b**, và ⛔ không phải một nhánh thứ tư ở đây."*

Tầng store *(⛔ không chạm, chỉ dùng)*: `ReadOnlyDb::open/read/close` · `ReaderPool::open_readonly` · `open_readonly_connection` · `apply_dict_reader_pragmas`. **⛔ Không tồn tại `ATTACH`** ở bất kỳ đâu trong `src-tauri/src` — mô hình là **một `ReadOnlyDb` = một tệp = một pool**, và gom nhiều tệp là **Story 1.13**.

`core/matching/mod.rs`: **8 dòng, toàn doc-comment, ⛔ 0 dòng mã**. `jieba-rs =0.10.3` và `tantivy-stemmers =0.4.0` **đã ghim** trong `Cargo.toml` nhưng ⛔ **chưa có mã nào gọi** — và AC10 nói story này **giữ nguyên con số 0** đó.

### 🧪 Quy ước test — khuôn đã có, ⛔ đừng chế khuôn mới

- **Vị trí:** `src-tauri/tests/dict_lookup.rs` *(18 ca)* · `dict_boundary.rs` *(3 ca)*. `src-tauri/tests/**` được **miễn trừ CÓ TÊN** khỏi `store_boundary.rs` ⇒ test **được phép** `use rusqlite` để dựng fixture; mã sản phẩm thì **không**.
- **Import:** `use auratranslate_lib::core::dict::{LookupMode, QueryBranch, QueryRoute, lookup, pick_branch, pick_route};`
- **Fixture dựng TRONG test** — ⛔ **không** tệp `.db` nào trong git *(`.gitignore: *.db`, AD-25)*. 9 hằng DDL chép **nguyên văn** từ `tools/dict-build/src/schema.rs`, gom vào `COPIED_DDL`, và có **cổng parity** `fixture_ddl_is_verbatim_from_dict_build_schema` giữ chúng ⛔ không trôi. **Cổng `is_han` của AC2 dùng đúng khuôn này.**
- **Fixture ⛔ không đặt `journal_mode`** — mặc định `delete`, giống ba tệp thật. Đóng kết nối dựng fixture **trước** khi `ReadOnlyDb` chạm vào.
- **Tên test = câu khẳng định đầy đủ, `snake_case`, tiếng Anh** *(⛔ không dấu tiếng Việt — `check-i18n` đã bắt đúng ca này một lần ở 1.10b)*.
- **Ca nào chạy được ⛔ KHÔNG CẦN `.db` thì tách ra** — `pick_route` và `pick_branch` là hàm thuần, và đó là điều kiện để phần đắt nhất của story nghiệm thu được **trong CI**.

### 🔧 Lệnh — chép nguyên

```sh
# BẮT BUỘC TRƯỚC cargo test (generate_context! nhúng dist/ lúc biên dịch)
npm run build

cargo test --locked --manifest-path src-tauri/Cargo.toml
cargo test --manifest-path src-tauri/Cargo.toml --test dict_lookup      # chỉ một tệp
cargo test --manifest-path src-tauri/Cargo.toml --test dict_boundary

npm run check:deps && npm run check:i18n && npm run check:commands \
  && npm run check:tokens && npm run check:dict && npm run check:dict-manifest

# Bench NFR1 — chạy TAY, đường dẫn phải TUYỆT ĐỐI (CWD của test là src-tauri/)
AURA_DICT_BENCH_DB="$PWD/tools/dict-build/out/dict-core.db" \
  cargo test --release --manifest-path src-tauri/Cargo.toml \
  --test dict_lookup -- --ignored --nocapture

# Đối chiếu số bằng sqlite3 (tuỳ chọn, để kiểm chứng số hàng bench)
sqlite3 tools/dict-build/out/dict-core.db \
  "SELECT COUNT(*) FROM dict_entry WHERE headword IN ('Running','running') AND lang='en';"
```

⛔ **KHÔNG chạy** `cargo run --manifest-path tools/dict-build/Cargo.toml` — mặc định `--layer all` dựng lại **cả ba** tệp `.db` và làm hai `sha256` trong `dict-manifest.toml` thành sai **mà ⛔ không cổng nào bắt được**.

### 🧭 Trí nhớ Git — bốn commit gần nhất

| Commit | Nói lên điều gì cho story này |
|---|---|
| `5a68df7` *Update deferred work…* | Story 1.10b đóng lại; `deferred-work.md` **+8 dòng** — trong đó có mục 🔴 *"đường tra cứu PHẢI lọc `lang`"* **chủ sở hữu là story này** |
| `dd7af61` *Add VIWIKTIONARY_EN source…* | Nguồn thứ sáu vào `dict-core.db`; `sources_meta.rs`, `dict-manifest.toml`, `check-dict-build.mjs` đều đã cập nhật ⇒ ⛔ **không** còn việc gì ở tầng dữ liệu |
| `ed8ce52` / `a3ed5cd` | Khuôn **fixture dựng trong test** + **integration test** đã đặt xong ở `tools/dict-build/tests/parse.rs` — cùng triết lý mà `dict_lookup.rs` dùng |

⚠️ **Cây làm việc đang có tệp CHƯA COMMIT** thuộc Story 1.11: `src-tauri/src/core/dict/query.rs`, `src-tauri/src/core/store/readonly.rs`, `src-tauri/tests/dict_lookup.rs`, `src-tauri/tests/dict_boundary.rs`, cộng sửa ở `core/dict/mod.rs`, `core/store/{mod,pragmas,reader}.rs`, `tests/store_contract.rs`. **Đó là nền của story này, ⛔ không phải rác** — ⛔ **đừng** `git checkout` / `git clean` chúng.

### Project Structure Notes

Cây nguồn *(`ARCHITECTURE-SPINE.md:790-794`)* **đã dành sẵn chỗ** cho story này — dòng chú thích viết trước khi mã tồn tại:

```text
core/
  matching/      # jieba + stemmer — DÙNG CHUNG (AD-17)          ← ⛔ story này KHÔNG chạm
  dict/          # ba nhánh zh (AD-26) + hai nhánh en và vị từ điều phối (AD-44)
                 # không hợp nhất nguồn, không hợp nhất zh với en (AD-19)
  store/         # Writer nối tiếp + Reader pool (AD-11, AD-12)   ← chỉ DÙNG, ⛔ không sửa
ports/           # DictionarySource · … (AD-2)                    ← ⛔ Story 1.13
```

**⛔ Không có biến variance nào.** Mọi tệp story này chạm đều nằm trong `src-tauri/src/core/dict/**` và `src-tauri/tests/**`. Quy ước: module Rust `snake_case`; một module cho **một khái niệm miền** *(⇒ giữ hai hàm SQL tiếng Anh **trong `query.rs`**, ⛔ không tách `query_en.rs`)*.

### References

- [Source: `_bmad-output/planning-artifacts/epics.md#Story 1.11b: Đường tra cứu tiếng Anh`] — `:1478-1495`
- [Source: `_bmad-output/planning-artifacts/architecture/architecture-AuraTranslate-2026-08-02/ARCHITECTURE-SPINE.md#AD-44`] — `:571-633` *(sáu mệnh đề ①–⑥ + bốn chỗ vá Reviewer Gate)*
- [Source: `…/ARCHITECTURE-SPINE.md#AD-26`] — `:334-342` *(phạm vi tiếng Trung nay nằm trong **thân** Rule; dải hiệu năng cũ **LỖI THỜI**)*
- [Source: `…/ARCHITECTURE-SPINE.md#AD-17`] — `:230-236` *(ranh giới: đường **từ điển** tiếng Anh ⛔ không gọi Matcher)*
- [Source: `…/ARCHITECTURE-SPINE.md#AD-19`] · `#AD-10` · `#AD-2` · `#AD-25` · `#AD-27` · `#AD-21`
- [Source: `…/ARCHITECTURE-SPINE.md#Consistency Conventions`] — hàng *Tra cứu* · *Hình dạng lỗi* · *Chuỗi giao diện* · *Module Rust*
- [Source: `…/ARCHITECTURE-SPINE.md#Deferred`] — `:852-853` *(stemming · cụm từ nhiều chữ)*
- [Source: `…/reviews/review-ad-44-2026-08-05.md`] — A1 *(vị từ chạy ở đâu)* · A2 *(cấm sổ đăng ký)* · A3 *(vị từ nhị phân)* · A4 *(lowercase không phụ thuộc locale)* · V1 *(bảng stem là assertion)*
- [Source: `…/architecture-AuraTranslate-2026-08-02/.memlog.md`] — `:162-172` *(mọi số đo `sqlite3` 2026-08-05; mục 🔴 CHẶN **đã đóng**)*
- [Source: `_bmad-output/implementation-artifacts/1-11-ba-nhanh-truy-van-tieng-trung.md`] — chữ ký, SQL, khuôn test, bench, 15 bẫy
- [Source: `_bmad-output/implementation-artifacts/1-10b-dung-du-lieu-tu-dien-tieng-anh.md`] — `viwiktionary-en`, hình dạng mục tiếng Anh, số đo
- [Source: `_bmad-output/implementation-artifacts/deferred-work.md`] — `:275-290` *(AD-44 đóng chặn)* · `:285-309` *(lọc `lang` — **vẫn mở cho story này**)* · `:289` *(UX mục tiếng Anh — Sally)*
- [Source: `_bmad-output/planning-artifacts/prds/prd-AuraTranslate-2026-08-02/prd.md`] — `:814` *(NFR1 p95 < 100 ms đầu-cuối; ~99,95 ms cho IPC + render)*
- [Source: `src-tauri/src/core/dict/mod.rs`] · [`query.rs`] · [`src-tauri/src/core/store/{readonly,pragmas,reader,mod}.rs`]
- [Source: `src-tauri/tests/{dict_lookup,dict_boundary,store_contract,store_boundary}.rs`]
- [Source: `tools/dict-build/src/char_idx.rs`] — `:14-25` *(bảy dải `is_han` — **nguồn sự thật**)*
- [Source: `tools/dict-build/src/schema.rs`] — DDL *(⛔ chỉ đọc, ⛔ không sửa)*

---

## Câu hỏi để lại cho người duyệt

1. **Ca 0 ký tự** *(AC7)* — story chốt `NoBranchQueryTooShort` và ghi lý do. AD-44 ② để trống ô này. **Winston** xác nhận hoặc lật.
2. **Bất đối xứng `api` ⇏ `API`** *(AC6)* — đúng theo chữ của AD-44 ③ *(hạ chữ thường là khoá **thêm** phía **truy vấn**)*. Nếu sản phẩm muốn khớp hai chiều thì đó là **chỉ mục hàm lúc build** ⇒ đổi `schema.rs` + dựng lại `dict-core.db` + `[base].sha256` + đo lại NFR6 ⇒ **tầng PRD/kiến trúc**, ⛔ không phải story này.
3. **`epics.md:1491` còn nói "dùng `Matcher` của AD-17"** — lệch AD-44 ③. Chủ sở hữu **John (PM)**. Story này theo **AD-44**.
4. **Panel Lookup chưa có hình dạng cho mục tiếng Anh** — chủ sở hữu **Sally**. ⛔ Không chặn story này *(0 dòng frontend)*, nhưng **chặn 1.17**.

---

## Dev Agent Record

### Agent Model Used

`claude-opus-5` (Claude Code, dev-story workflow) — 2026-08-05

### Debug Log References

**Đột biến — bảy lượt, cả bảy đều bị bắt.** Mỗi cổng mới được kiểm bằng cách *phá* thứ nó
canh, chạy `dict_lookup` + `dict_boundary`, rồi hoàn nguyên:

| # | Đột biến | Ca đỏ |
|---|---|---|
| M1 | `exact_en`: `IN (?1, ?2)` → `IN (?1, ?1)` *(bỏ khoá hạ chữ thường)* | `an_uppercase_english_query_still_finds_a_lowercase_headword` + 5 ca khác |
| M2 | `exact_en`: tham số `{query, lowered}` → `{lowered, lowered}` *(THAY khoá gốc)* | `an_uppercase_headword_is_still_reachable_by_its_own_spelling`, `lowercasing_happens_on_the_query_never_on_the_headword` |
| M3 | `fts_trigram_en`: bỏ vế `e.lang = 'en'` | `both_english_branches_filter_out_chinese_entries` |
| M4 | Ngưỡng En: `chars().count() < 3` → `len() < 3` | `the_english_length_threshold_counts_characters_not_bytes` |
| M5 | `NoBranchQueryTooShort` → `FtsTrigram` *(tràn qua nhánh)* | `a_too_short_english_query_prepares_no_sql_at_all` + 3 ca khác |
| M6 | `is_han`: dải Extension B tụt về BMP | `a_han_character_outside_the_bmp_still_routes_to_the_chinese_path`, `han_ranges_are_verbatim_from_dict_build_char_idx` |
| M7 | `query.rs` gọi `super::pick_route` | `the_routing_predicate_lives_in_exactly_one_file_and_the_adapter_never_calls_it` |

**`EXPLAIN QUERY PLAN` trên `dict-core.db` thật** *(chạy tay qua `sqlite3 -readonly`)*:

```text
-- exact_en: e.headword IN ('Running','running') AND e.lang = 'en'
|--SEARCH e USING INDEX idx_entry_headword (headword=?)   ← ⛔ KHÔNG có SCAN dict_entry
|--SEARCH s USING INTEGER PRIMARY KEY (rowid=?)
`--USE TEMP B-TREE FOR ORDER BY

-- fts_trigram_en: entry_fts MATCH '"dic"' AND e.lang = 'en'
|--SCAN f VIRTUAL TABLE INDEX 0:M1   ← hậu tố `:M1` = MATCH đã dùng; `SCAN` ở bảng ảo
|--SEARCH e USING INTEGER PRIMARY KEY (rowid=?)   FTS5 ⛔ KHÔNG phải một vi phạm
|--SEARCH s USING INTEGER PRIMARY KEY (rowid=?)
`--USE TEMP B-TREE FOR ORDER BY
```

⚠️ **Một lượt biên dịch `--release` hỏng RỒI TỰ HẾT** *(⛔ không phải lỗi của story này)*:
lần chạy bench thứ hai đỏ với `the crate 'wry' requires panic strategy 'abort' which is
incompatible with this crate's strategy of 'unwind'` × 144. Nguyên nhân: `[profile.release]`
đặt `panic = "abort"` *(`Cargo.toml:65`)*, và một tập artifact release cũ còn nằm lại trong
`target/`. Chạy lại **đúng cùng lệnh** ⇒ xanh, ⛔ không sửa một dòng nào. Nếu ai gặp lại:
`cargo clean --release` rồi chạy lại.

### Completion Notes List

- [x] **Số đo NFR1 thật trên đường tiếng Anh — BẢN RELEASE, ⛔ không phải debug.**

  Lệnh: `AURA_DICT_BENCH_DB="$PWD/tools/dict-build/out/dict-core.db" cargo test --release
  --manifest-path src-tauri/Cargo.toml --test dict_lookup -- --ignored --nocapture`
  · `WARMUP = 10` · `RUNS = 200` · percentile nearest-rank · trần **10 ms**.

  | Nhánh | Truy vấn | p50 | p95 | p99 | Số hàng |
  |---|---|---|---|---|---|
  | `en-1-btree-lower` | `running` | 0,047 ms | **0,063 ms** | 0,088 ms | 1 |
  | `en-1-btree-upper` | `Running` *(tập khoá 2 phần tử)* | 0,046 ms | **0,052 ms** | 0,186 ms | 1 |
  | `en-2-trigram` | `dic` | 0,725 ms | **0,961 ms** | 1,392 ms | 571 |

  ⇒ **Cả ba tổ hợp tiếng Anh đều dưới trần 10 ms với biên rất rộng** — nhánh en chậm nhất
  dùng **9,6%** ngân sách. Lượt đo thứ nhất *(cùng bản release, cùng máy)* cho
  0,099 / 0,065 / 1,166 ms ở cột p95 — cùng bậc độ lớn, chênh là nhiễu máy.

  ⚠️ **Tập khoá hai phần tử ⛔ KHÔNG đắt hơn một phần tử.** `Running` *(hai khoá khác
  nhau ⇒ hai lượt dò B-tree)* đo được **nhanh hơn** `running` *(hai khoá trùng nhau)* ở cả
  p50 lẫn p95. Nỗi lo *"`IN (?1, ?2)` làm đường nóng chậm gấp đôi"* là **⛔ không có cơ
  sở** — chi phí một lượt tra chính xác nằm ở dựng chuỗi kết quả, ⛔ không ở lượt dò chỉ mục.

  🟡 **Nhánh chậm nhất của CẢ HAI đường vẫn là `zh-2-charidx-1` *(`山`)*: p95 **7,107 ms**
  / trần 10 ms — dưới trần, còn 29% dư địa.** Đó là món nợ Ice **đã chốt "chấp nhận nguyên
  trạng"** ở Story 1.11; story này ⛔ **không** chạm vào nó và ⛔ không thêm `LIMIT` nào.
  *(Lượt đo thứ nhất cho 6,075 ms; story ghi 7,324 ms từ lượt đo của 1.11 — cùng dải.)*

  **Số hàng thật, đo lại trên `dict-core.db` (`built_at = 2026-08-04T23:53:16Z`, ⛔ không
  dựng lại):** `running`/`Running`/`API` ⇒ **1** hàng mỗi truy vấn · `api` ⇒ **0** *(bất
  đối xứng có chủ ý)* · `dic` ⇒ **571** · truy vấn Hán ép `route = En` ⇒ **0** ở cả hai
  nhánh. Bảy con số của đường zh giữ **nguyên vẹn**.

  🔴 **`dic`: 572 ứng viên → 571 sau `verify_substring`.** Trigram FTS5 ⛔ không phân biệt
  chữ hoa; `str::contains` thì có ⇒ **1** dương tính giả bị loại. Con số đó là bằng chứng
  rằng bước xác minh **có việc thật để làm trên đường tiếng Anh nữa**, ⛔ không chỉ trên
  đường tiếng Trung *(nơi `中國` cho 390 → 350)*.

- [x] **Ca 0 ký tự — GIỮ NGUYÊN cách story đã chốt, ⛔ không lật.**

  `""` + `Substring` + `En` ⇒ `NoBranchQueryTooShort`, `hits` rỗng, **⛔ không một câu SQL
  nào được chuẩn bị**. Lý do giữ: vị từ độ dài là **một** mệnh đề `chars().count() < 3`, và
  một ca đặc biệt cho `0` là một nhánh thứ ba ⛔ không ai đòi.

  **Vế *"⛔ không chạm database"* nay là thứ NGHIỆM THU ĐƯỢC, ⛔ không phải một lời hứa
  trong doc-comment** — `a_too_short_english_query_prepares_no_sql_at_all` mở một tệp `.db`
  hợp lệ **⛔ không có bảng từ điển nào**: truy vấn quá ngắn trả `Ok`, truy vấn 3 ký tự
  *(cùng chế độ, cùng đường)* trả `Err`. Vế `Err` là đối chứng dương và nó ⛔ không bỏ được.

  📌 **Winston vẫn là người chốt cuối** *(Câu hỏi #1)*. Nếu lật, chỗ sửa là **một** mệnh đề
  trong `pick_branch` cộng ba ca test — ⛔ không phải một lượt viết lại.

- [x] **Cổng parity `is_han` — món nợ hai-workspace ĐÃ ĐÓNG.**

  `han_ranges_are_verbatim_from_dict_build_char_idx` có **hai vế**, và bỏ vế nào cũng làm
  nó thành trang trí: *(1)* bảy chuỗi dải có mặt **nguyên văn** trong
  `tools/dict-build/src/char_idx.rs` *(đọc dạng văn bản qua `env!("CARGO_MANIFEST_DIR")`,
  ⛔ không import chéo crate)* + sàn ≥ 7; *(2)* chính hàm `core::dict::is_han` nhận đúng
  bảy dải đó, kiểm ở **cả hai biên** cộng hai điểm ngay ngoài biên.

  Bản sao chỉ-BMP ở `tests/dict_lookup.rs:211` **đã bị xoá**, và
  `exactly_one_definition_of_is_han_exists_under_src_tauri` *(`dict_boundary.rs`)* giữ cho
  toàn `src-tauri/**` chỉ còn **một** định nghĩa — quét cả `src/**` lẫn `tests/**`, sàn
  quần thể **20** trên **36** tệp `.rs` thật. Đột biến M6 xác nhận cả hai vế đều đỏ được.

- [x] **⛔ Không mục nào cần thêm vào `deferred-work.md`.**

  Món nợ 🔴 *"đường tra cứu PHẢI lọc `dict_entry.lang`"* *(`deferred-work.md:285`, chủ sở
  hữu là story này)* nay **đã đóng bằng cổng**: `the_lookup_path_actually_uses_the_two_indexes`
  đòi văn bản gộp của `core/dict/**` chứa **cả** `lang = 'zh'` **lẫn** `lang = 'en'`, và
  `both_english_branches_filter_out_chinese_entries` ép tổ hợp `(truy vấn Hán, route = En)`
  trên **cả hai** nhánh tiếng Anh. Món nợ hai-workspace `is_han` *(`review-ad-44:49`,
  `ARCHITECTURE-SPINE.md:588`)* cũng đã đóng — xem mục trên. Ba mục còn mở
  *(`epics.md:1491` lệch AD-44 — **John**; dòng 🔴 CHẶN cần gỡ — **John**; hình dạng hiển
  thị mục tiếng Anh — **Sally**)* đều **⛔ không thuộc quyền story này** và đã có chủ.

- [x] **Một lượt THU HẸP so với AC9, khai ra thay vì giấu.**

  AC9 nói *"thay đổi duy nhất được phép ở các ca cũ là **chỗ gọi**"*. Có **đúng một** ngoại
  lệ: `every_branch_filters_out_english_entries` khẳng định `english == 2`, và Task 5.1
  *(bắt buộc)* thêm `running` + `API` vào `SEEDS` ⇒ con số thành **4**. Đó là **quần thể
  fixture**, ⛔ không phải một mệnh đề của Story 1.11 — ý nghĩa của phép kiểm *(fixture CÓ
  hàng `lang='en'` thật, nên hai phép khẳng định `is_empty()` có việc để làm)* giữ nguyên.
  ⛔ **Không** một câu SQL nào của đường zh bị sửa; ⛔ không một số đo zh nào đổi.

- [x] **`cargo fmt` — baseline đỏ RỘNG HƠN story ghi; tệp story này chạm thì SẠCH.**

  Story ghi *"đỏ từ baseline ở ba chỗ"*. Đo thật hôm nay: **29 chỗ trên 12 tệp**
  *(`core/i18n/mod.rs` · `core/scope/resolve.rs` · `core/store/{mod,pragmas,schema}.rs` ·
  `lib.rs` · `tests/{config_invariants,ipc_contract,scope_boundary,scope_contract,store_boundary,store_contract}.rs`)*.
  ⛔ **Không** đụng một chỗ nào trong số đó. Bốn tệp story này chạm —
  `core/dict/{mod,query}.rs` + `tests/{dict_lookup,dict_boundary}.rs` — **⛔ không còn một
  chỗ nào** *(chạy `rustfmt` riêng cho hai tệp test; hai tệp `src` vốn đã sạch)*. CI ⛔
  không chạy `fmt` cũng ⛔ không chạy `clippy`, nên đây là kỷ luật chứ ⛔ không phải cổng.

  ⚠️ Bảng bench được viết bằng **tên rút gọn cục bộ** *(`use LookupMode::{Exact, Substring}`
  … **chỉ trong hàm bench**)* — nếu không, `rustfmt` bung mỗi hàng thành **5 dòng** và bảng
  ⛔ không còn đọc được như một bảng.

- [x] **Cổng cuối — tất cả xanh, ⛔ không hạ một sàn nào.**

  `npm run build` ✅ · `cargo test --locked` ✅ *(**102** ca xanh, 2 `#[ignore]`; `dict_lookup`
  18 → **35**, `dict_boundary` 3 → **5**)* · sáu cổng `.mjs` ✅ *(exit 0 cả sáu;
  `check:i18n` vẫn báo đúng **16 khoá · 9 placeholder**)*.

  ⛔ Không hạ/nới sàn nào: `DICT_FLOOR = 1` · `store_boundary::RS_FLOOR = 20` ·
  `scope_boundary::RS_FLOOR = 20` · `check-i18n::RS_FLOOR = 21`/`VUE_FLOOR = 1` ·
  `check-dict-build::RS_FILE_FLOOR = 21` — **giữ nguyên**. `FORBIDDEN` của
  `dict_boundary.rs` ⛔ không nới. `STORE_DIR`/`FORBIDDEN` của `store_boundary.rs` ⛔ không
  nới — `core/dict/**` ⛔ vẫn không gõ `rusqlite`. Sàn **mới** `SRC_TAURI_RS_FLOOR = 20` là
  một sàn **thêm vào**, ⛔ không phải một sàn bị hạ.

  **Ranh giới ⛔ KHÔNG CHẠM, đã đối chiếu bằng `git status`:** ⛔ 0 dòng `.vue`/`.ts`/`.css`
  · ⛔ `src-tauri/Cargo.toml` và `Cargo.lock` ⛔ không đổi một dòng · ⛔ `tools/dict-build/**`
  ⛔ không chạm · ⛔ ⛔ **không** lượt `cargo run` nào của `dict-build` · ⛔ `ports/`,
  `commands/`, `core/matching/`, `core/scope/`, `core/i18n/`, `core/store/**` ⛔ không chạm
  *(bốn tệp `core/store/**` đang `M` trong `git status` là **nền chưa commit của Story
  1.11**, có từ trước lượt này)* · `core/matching/mod.rs` vẫn **0 dòng mã** · ⛔ 0 lời gọi
  tới `core::matching` / `tantivy_stemmers` / `jieba_rs`.

  🔴 **`sha256` của `dict-core.db` đo lại sau toàn bộ lượt bench:
  `2145c7ae…b4f305a` — KHỚP nguyên `dict-manifest.toml:62`.** ⛔ Không tệp `-wal`/`-shm`/
  `-journal` nào xuất hiện cạnh ba tệp `.db`. AD-25 nguyên vẹn.

### File List

| Tệp | Trạng thái |
|---|---|
| `src-tauri/src/core/dict/mod.rs` | **sửa** — `+ QueryRoute` · `+ pick_route` · `+ is_han` · `+ QueryBranch::NoBranchQueryTooShort`; chữ ký `pick_branch` / `lookup` nhận `route`; doc-comment module *(bảng hai đường)* và `EntryHit::lang` viết lại |
| `src-tauri/src/core/dict/query.rs` | **sửa** — `+ exact_en` · `+ fts_trigram_en`; doc-comment module *(luật 2 đảo chiều theo đường)*. ⛔ **Không** một câu SQL zh nào bị sửa |
| `src-tauri/tests/dict_lookup.rs` | **sửa** — xoá bản sao `is_han` chỉ-BMP; `+ HAN_RANGES`; `SEEDS` 8 → 10 hàng; `+ 17` ca *(1 cổng parity · 4 vị từ điều phối · 2 bảng nhánh En · 10 hành vi En/AC11)*; bench mở rộng 3 tổ hợp tiếng Anh; 18 ca cũ chỉ đổi **chỗ gọi** |
| `src-tauri/tests/dict_boundary.rs` | **sửa** — `+ 2` ca *(vị từ ở đúng một tệp · một định nghĩa `is_han` trong `src-tauri/**`)*; `+ SRC_TAURI_RS_FLOOR`; đối chứng dương nới quần thể sang `lang = 'en'` |
| `_bmad-output/implementation-artifacts/1-11b-duong-tra-cuu-tieng-anh.md` | **sửa** — `baseline_commit`, Status, Tasks, Dev Agent Record, File List |
| `_bmad-output/implementation-artifacts/sprint-status.yaml` | **sửa** — `1-11b` → `review`, `last_updated` |
