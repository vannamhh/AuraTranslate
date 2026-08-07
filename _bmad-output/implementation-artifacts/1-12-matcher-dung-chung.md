---
baseline_commit: 5edbe0ee868e2cf3cc3b54d42bf58f694d56d231
---

# Story 1.12: Matcher dùng chung

Status: done

## Change Log

| Ngày | Thay đổi |
|---|---|
| 2026-08-05 | `ready-for-dev` → `in-progress`. Baseline `5edbe0e`, cây làm việc sạch. |
| 2026-08-05 | Task 0 chốt **phương án B** *(ba nguyên hàm **+** `find_terms`)* — không có chỉ đạo khác của Ice. |
| 2026-08-05 | Giao `core/matching/mod.rs` *(514 dòng)* + `tests/matching_contract.rs` *(22 ca)* + `tests/matching_boundary.rs` *(8 cổng)*. |
| 2026-08-05 | Đo thật ba phép của Task 7: bảng stemmer AD-44 ③ *(**đúng 3/3**)* · `Jieba` init **179–329 ms** *(🔴 vượt NFR2 = 50 ms)* · delta `.dmg` **0 trong sai số** *(LTO cắt sạch vì chưa có người tiêu thụ)*. |
| 2026-08-05 | Ba giả định bị **số đo lật** và được sửa theo số: cờ `HMM` · luật ranh giới token đường `Zh` · ví dụ `happiest` của AC7. |
| 2026-08-05 | 11 đột biến M1–M11, tất cả **đỏ đúng ý** rồi hoàn nguyên. |
| 2026-08-05 | Thêm 6 mục vào `deferred-work.md`. `in-progress` → `review`. |
| 2026-08-05 | Code review (bmad-code-review): 2 quyết định + 4 patch (đã vá, tests xanh), 5 defer (ghi `deferred-work.md`), 10 dismiss. `review` → `done`. |

**Epic:** 1 — Nền móng ứng dụng & Tra cứu ngoại tuyến tức thì
**Story key:** `1-12-matcher-dung-chung`
**Covers:** FR40 *(nửa **giới hạn đã tuyên bố** — nửa tra cứu từ điển đã đóng ở 1.11b bằng tập khoá, không bằng stemming)* · **nền** cho FR51 *(Story 3.4)* và FR61 *(Story 7.6)*
**Governed by:** **AD-17** *(chủ)* · AD-44 ③⑤ *(ranh giới không gọi)* · AD-13 *(chiều phụ thuộc)* · AD-21 · NFR15 · NFR6 · NFR1 · NFR2
**Ngày tạo:** 2026-08-05

---

## 🔴 ĐỌC TRƯỚC TIÊN — MỘT AC CỦA `epics.md` ĐÃ BỊ KIẾN TRÚC LẬT, VÀ STORY NÀY THEO KIẾN TRÚC

`epics.md:1510` viết cho story này:

> ~~**And** `dict/` **dùng nó**; `glossary/` và `tm/` sẽ dùng chính nó ở các epic sau, không cài lại~~

**Vế `dict/` dùng nó` KHÔNG CÒN ĐÚNG.** AD-17 đã được sửa Rule ngày 2026-08-05 *(`ARCHITECTURE-SPINE.md:236`)*, và nó nói thẳng:

> ⚠️ **AD này nói *mọi nơi cần khớp ngôn ngữ dùng chung MỘT cài đặt* — nó KHÔNG nói mọi đường đều phải gọi Matcher.** Đường tra cứu **từ điển** tiếng Anh không gọi […] Glossary (FR51) và TM (FR61) thì **có** […] Phân biệt này không nới lỏng AD-17: vẫn **đúng một** cài đặt, chỉ là không phải đường nào cũng là người tiêu thụ nó.

**Story 1.11b đã giao xong đường tra cứu tiếng Anh mà không gọi Matcher một lần nào** *(AC10 của 1.11b, đã `done`)*. Nếu dev story này đi "nối `core/dict/` vào `core/matching/`" để làm cho AC trên xanh, kết quả là:

1. **Hồi quy đường tra cứu** — 1.11b đo thật p95 `0,052–0,961 ms`; một lượt stemming trên đường nóng đổi lấy **~0 recall** *(AD-44 ③, dữ kiện mạnh: corpus đã có sẵn mọi dạng biến thể làm đầu mục riêng, **16/16** mẫu thử gồm cả bất quy tắc)*.
2. **Đỏ cổng** — `dict_boundary.rs` sẽ đỏ, và AC4 dưới đây dựng thêm một cổng nữa để nó đỏ **có tên**.

⇒ **Story này giao `core/matching/` cho Glossary và TM, và cưỡng chế bằng cổng rằng `core/dict/**` KHÔNG gọi nó.**

> 🟡 **HAI chỗ tài liệu ĐANG LỆCH — không DEV KHÔNG SỬA. Ghi ra để không ai tưởng mình đọc nhầm.**
> *(a)* `epics.md:1510` *(vế `dict/` dùng nó)* và `epics.md:1491` *(mục 1.11b)* — chủ sở hữu **John (PM)**, đã ghi ở `deferred-work.md:282`.
> *(b)* 🔵 **PHÁT HIỆN MỚI của lượt tạo story này:** sơ đồ mermaid của **AD-13** *(`ARCHITECTURE-SPINE.md:189`)* còn cạnh `dict --> matching`. Nó vẽ trước lượt sửa Rule của AD-17 và nay **mâu thuẫn với chính thân Rule ở `:236`**. Chủ sở hữu **Winston (architect)**. **Dev theo THÂN RULE của AD-17 + AD-44 ③, không theo mũi tên trong mermaid.** Thêm mục này vào `deferred-work.md` là **Task 7.5** của story này.

---

## Story

As a **người dựng**,
I want **một component khớp ngôn ngữ duy nhất phục vụ Glossary và Translation Memory**,
So that **hai nơi không bao giờ bắt được những biến thể khác nhau mà không ai hiểu vì sao**.

---

## Ranh giới phạm vi — ĐỌC TRƯỚC KHI GÕ DÒNG ĐẦU TIÊN

Story này là **một module thư viện thuần trong `core/matching/`**. Nó **không** phải một lượt dựng UI, **không** phải một lượt dựng cổng, **không** phải một lượt chạm database, và **không** phải một lượt sửa đường tra cứu.

| Thứ | Trong phạm vi? | Chủ sở hữu thật |
|---|---|---|
| `core/matching/**` — tokenize · normalize · n-gram · tìm thuật ngữ trong văn bản | ✅ **CÓ** | story này |
| Cổng cưỡng chế *"đúng một cài đặt"* + *"`dict/` không gọi"* | ✅ **CÓ** | story này |
| Đo chi phí khởi tạo `Jieba` *(NFR2)* và delta NFR6 | ✅ **CÓ** | story này |
| Chạy **stemmer thật** và thay bảng phỏng đoán Porter của AD-44 ③ bằng số đo | ✅ **CÓ** *(rẻ, xem AC8)* | story này |
| Bất kỳ dòng nào dưới `core/dict/**` | **KHÔNG** — 🔴 và có **cổng** chặn | đã xong ở 1.11/1.11b |
| Bảng Glossary, vòng đời ba trạng thái, đánh dấu thuật ngữ trong Panel Source | **KHÔNG** | **Epic 3** *(3.1, 3.4)* |
| Khớp mờ TM, ngưỡng % tương đồng, xếp hạng ứng viên | **KHÔNG** | **Epic 7** *(7.5, 7.6)* |
| Bất kỳ dòng `.vue` / `.ts` / `.css` nào | **KHÔNG** | Epic 3 / Epic 5 |
| `MessageKey` mới, khoá `vi.json` mới, lệnh IPC mới, `#[tauri::command]` | **KHÔNG** | story sở hữu bề mặt tương ứng |
| `ports/`, `commands/`, `core/store/**`, `core/scope/**`, `core/i18n/**` | **KHÔNG** | — |
| `tools/dict-build/**`, `schema.rs`, dựng lại bất kỳ tệp `.db` nào | **KHÔNG** | đã xong ở 1.9/1.10/1.10b |
| Thêm **bất kỳ** crate nào vào `Cargo.toml` | **KHÔNG** — cả hai crate cần dùng **đã ghim** | — |

🔴 **Story này KHÔNG có người tiêu thụ trong cây mã hôm nay.** `core/glossary/mod.rs` và `core/tm/mod.rs` mỗi tệp **4 dòng doc-comment, 0 dòng mã**. Đó là **chủ ý của AD-17**: dựng **một** cài đặt trước khi ba nơi mọc ba bản. Hệ quả phải chấp nhận có ý thức: **hình dạng API là một phỏng đoán có căn cứ, không phải một hợp đồng đã nghiệm thu bằng người dùng thật.** Cách story này giảm rủi ro đó — đừng bỏ qua:

- Mọi hàm công khai đều **suy ra từ một AC có thật của một story có thật** *(3.4 và 7.6, trích nguyên văn ở §Dev Notes)*, không từ trí tưởng tượng.
- Hình dạng trả về mang **span byte** chứ không chỉ mang `bool` — vì Story 3.4 phải **tô màu** thuật ngữ tại đúng vị trí trong câu *(`epics.md:2528`)*, và một API trả `bool` sẽ buộc 3.4 tự dò lại vị trí, tức **bản cài thứ hai** ra đời ngay ở nơi AD-17 tồn tại để chặn.
- **Không** hàm nào được viết mà không có ít nhất một ca test khẳng định hành vi của nó.

---

## 🔴 QUYẾT ĐỊNH PHẢI CHỐT TRONG STORY

**Story này giao *bốn nguyên hàm* thôi, hay giao thêm cả điểm vào `find_terms`?**

| Phương án | Nội dung | Hệ quả |
|---|---|---|
| **A — chỉ nguyên hàm** | `tokenize` · `normalize` · `ngrams` | 3.4 và 7.6 mỗi bên tự lắp một vòng khớp trên các nguyên hàm ⇒ **hai vòng khớp**, đúng thứ AD-17 tồn tại để chặn *(*"Glossary bắt được biến thể mà TM không bắt được"*)*. |
| **B — nguyên hàm + `find_terms`** ✅ **khuyến nghị** | thêm điểm vào tìm thuật ngữ trong văn bản, trả span | AD-17 nói *"một **component**"*, không nói *"một túi hàm tiện ích"*. `epics.md:1509` đòi *"tồn tại **đúng một** cài đặt khớp ngôn ngữ"* — một vòng khớp nằm ở 3.4 **là** một cài đặt khớp thứ hai. |

**Khuyến nghị B, và đừng nới rộng hơn thế:** không xếp hạng, không ngưỡng %, không chỉ mục ngược, không cache — ba thứ đó thuộc 7.5/7.6 và phụ thuộc dữ liệu thật. **Nếu Ice chọn A, đó là một quyết định phạm vi — ghi vào Completion Notes, đừng âm thầm giao A rồi đánh dấu AC xanh.**

---

## Acceptance Criteria

### AC1 — Tồn tại **đúng MỘT** cài đặt khớp ngôn ngữ, và nó nằm ở `core/matching/`

**Given** `src-tauri/src/core/matching/`
**When** rà mã
**Then** tồn tại **đúng một** cài đặt khớp ngôn ngữ, không phải hai
**And** **chỉ** `core/matching/**` được gõ `jieba_rs` và `tantivy_stemmers` ở **vị trí mã** — `core/{dict,glossary,tm,segment,library,export,webimport,ai,store,scope,i18n}/**`, `commands/**`, `ports/**` đều **0 lần**
**And** **đối chứng dương bắt buộc:** `core/matching/**` **có thật sự** gõ **cả hai** — không có nó thì cổng trên xanh y hệt trên một `core/matching/` **rỗng**, *"không ai vi phạm"* và *"không có gì để vi phạm"* đọc giống hệt nhau *(khuôn `core_store_actually_uses_rusqlite` của `store_boundary.rs:209`)*
**And** cổng có **sàn quần thể** cho `core/matching/**` — một đường dẫn gõ sai làm `walk` khớp 0 tệp và cổng xanh mà không kiểm gì cả

### AC2 — `glossary/` và `tm/` là người tiêu thụ; `dict/` **KHÔNG** *(AD-17 thân Rule + AD-44 ③)*

**Given** `core/matching/**`
**When** rà chữ ký công khai
**Then** mọi hàm công khai **dùng được** từ `core::glossary` và `core::tm` mà không cần thêm một lớp bọc nào — nghĩa là chúng nhận `&str` + `MatchLang` và trả dữ liệu thuần, không nhận kiểu riêng của một module miền nào

**Given** `src-tauri/src/core/dict/**`
**When** rà
**Then** **0** lời gọi tới `core::matching`, **0** lần gõ `matching`, `jieba`, `stemmer`, `stem(` ở **vị trí mã**
**And** cưỡng chế bằng test tĩnh trong `matching_boundary.rs`, kèm **thông báo assert nêu đích danh AD-17 `:236` và AD-44 ③** — một cổng đỏ mà không nói vì sao sẽ bị gỡ trong tuần
**And** **không** một dòng nào dưới `core/dict/**` bị sửa ở story này *(đối chiếu bằng `git status`)*

> 🔴 **Vì sao đây là AC chứ không phải một dòng ghi chú:** vế `dict/ dùng nó` **vẫn còn nguyên trong `epics.md`**, và nó là thứ một dev đọc epics *(hoặc một lượt review sau)* sẽ đọc thấy trước khi đọc AD-17. Cổng là chỗ duy nhất mệnh đề này sống sót qua một lượt đọc ẩu.

### AC3 — `core/matching/` là **lá** trong đồ thị phụ thuộc

**Given** `core/matching/**`
**When** rà `use`
**Then** **không** `use crate::core::{dict, glossary, tm, segment, library, export, webimport, ai, store, scope, i18n}` nào
**And** **không** `use crate::ports`, không `use crate::commands`
**And** đặc biệt **không** phụ thuộc `ai/` — AD-13 nói *"không module nào ngoài `ai/` được phụ thuộc `ai/`"*
**And** module **không** chạm filesystem, không chạm database, không ra mạng *(AD-15: đúng ba điểm ra mạng, không có điểm thứ tư)*

### AC4 — Ngôn ngữ là **THAM SỐ từ chỗ gọi**, **KHÔNG** đoán từ nội dung

**Given** mọi điểm vào công khai của `core/matching/`
**When** rà chữ ký
**Then** mỗi hàm nhận `lang: MatchLang` **tường minh**
**And** **không tồn tại** một vị từ dò script nào trong `core/matching/**` — không `is_han`, không `is_cjk`, không `detect_lang`, không một dải Unicode nào viết cứng

> 🔴 **Ba lý do, cả ba cưỡng chế được:**
> 1. **Đã có một cổng đang canh** — `exactly_one_definition_of_is_han_exists_under_src_tauri` *(`dict_boundary.rs`, Story 1.11b)* quét **toàn** `src-tauri/**` và **sẽ đỏ** nếu story này thêm một định nghĩa thứ hai. **Đừng nới cổng đó**; hãy đừng tạo ra thứ làm nó đỏ.
> 2. **Ngữ nghĩa khác hẳn `pick_route` của AD-44.** `pick_route` trả lời *"tra vào bảng nào của tệp `.db` nào"* — một câu hỏi về **hình dạng truy vấn**. Matcher trả lời *"khớp thuật ngữ trong **văn bản của một Tác phẩm**"*, và ngôn ngữ nguồn của Tác phẩm là một trường **bất biến trong `meta.json`, đặt lúc tạo** *(`ARCHITECTURE-SPINE.md:272-276`, `prd.md:765-774`)*. Đoán lại từ nội dung là bỏ đi một dữ kiện đã có và **thay bằng một phỏng đoán**.
> 3. Cùng luật đã đặt ở `LookupMode`: *"chế độ do **chỗ gọi** quyết, không đoán từ nội dung"* *(`core/dict/mod.rs`)*.

### AC5 — Tiếng Trung: khớp chính xác + n-gram ký tự, tách từ qua `jieba-rs` khi cần

**Given** `MatchLang::Zh`
**When** khớp
**Then** khớp thuật ngữ là **khớp chính xác** *(`epics.md:2532` — Story 3.4: *"văn bản tiếng Trung → dùng khớp chính xác"*)*
**And** n-gram là **n-gram KÝ TỰ** *(`epics.md:4946` — Story 7.6: *"n-gram ký tự — không có ranh giới từ"*)*
**And** tách từ đi qua `jieba-rs`, và **cờ `hmm` là một hằng đã chốt trong module** kèm lý do — **không** phơi ra thành tham số cho mỗi chỗ gọi tự chọn *(hai chỗ gọi chọn hai giá trị = hai bộ ranh giới từ = đúng lớp lỗi AD-17 tồn tại để chặn)*
**And** phép đếm độ dài n-gram là **`chars().count()`**, **không bao giờ** `len()`

> 🔴 **`len()` là bẫy đắt nhất đã cắn story 1.11 một lần** *(kiểm chứng bằng 5 ca đột biến đỏ)*: `"山".len()` là **3**, `"中國".len()` là **6**. Một n-gram ký tự cắt theo byte trên chữ Hán không chỉ sai — nó **panic** ở biên không phải ranh giới UTF-8.

**Given** một chuỗi tiếng Trung không có dấu cách
**When** sinh n-gram ký tự với `n = 2`
**Then** `"中國人"` ⇒ `["中國", "國人"]` — **cửa sổ trượt theo ký tự**, không theo token
**And** `n` lớn hơn số ký tự của chuỗi ⇒ trả **rỗng**, không panic, không trả một n-gram cụt

### AC6 — Tiếng Anh: **stemming rồi token n-gram**

**Given** `MatchLang::En`
**When** chuẩn hoá một token
**Then** **hạ chữ thường TRƯỚC, rồi mới stem** — 🔴 tài liệu của chính crate nói: *"❗️❗️ Tokens are expected to be lowercased beforehand"* *(`tantivy-stemmers-0.4.0/src/lib.rs:12`)*
**And** phép hạ chữ thường dùng `str::to_lowercase()` của Rust, **không phụ thuộc locale** — cùng luật AD-44 ③ đã chốt cho đường tra cứu *(một phép fold theo locale làm **cùng một đầu vào cho hai kết quả trên hai máy** cài ngôn ngữ hệ điều hành khác nhau — một hồi quy không tái lập được trên máy người sửa)*
**And** thuật toán là **`algorithms::english_porter_2`** *(Porter2/English — feature `english_porter_2`, **mặc định đã bật**, không cần đổi `Cargo.toml`)*
**And** n-gram là **token n-gram SAU stemming** *(`epics.md:4950`)*, không phải n-gram ký tự

**Given** `n = 2` trên `"the running dogs"`
**When** sinh token n-gram sau stemming
**Then** cửa sổ trượt trên **danh sách token đã stem**, không trên chuỗi gốc

### AC7 — Một biến thể hình thái tiếng Anh **nhận diện được về dạng gốc**

**Given** một từ tiếng Anh ở dạng biến thể hình thái *(`running` · `dogs` · `studies` · `happiest`)*
**When** khớp qua Matcher
**Then** nó khớp với thuật ngữ ở dạng gốc tương ứng — hai vế cùng đi qua **cùng một** phép chuẩn hoá, và đó chính là cơ chế
**And** ca test khẳng định **chuỗi stem THẬT** do `english_porter_2` sinh ra, **không** khẳng định một chuỗi chép từ mô tả kinh điển của Porter

### AC8 — Giới hạn **stemming ≠ lemmatization** được ghi lại **tường minh và ĐO ĐƯỢC**

**Given** một biến thể **bất quy tắc** — `went` · `gone` · `children` · `better` · `mice`
**When** tra cứu qua Matcher
**Then** giới hạn được ghi lại tường minh: đây là ***stemming***, **không phải** *lemmatization* *(FR40 — `epics.md:156`)*
**And** giới hạn là một **ca test có tên**, không phải một câu trong doc-comment: ca khẳng định `stem("went") != stem("go")` *(và các cặp bất quy tắc khác)*, tức **nó đỏ nếu một ngày ai đó đổi sang lemmatizer** — lúc đó người sửa **phải** đọc lý do trước khi đổi con số

**Given** AD-44 ③ ghi ⚠️ *"ba chuỗi stem đó lấy từ hành vi kinh điển của Porter chứ **chưa chạy qua stemmer mà sản phẩm sẽ dùng** […] ai muốn mở lại câu hỏi stemming thì việc đầu tiên là **chạy stemmer thật và thay bảng này bằng số đo**"* *(`ARCHITECTURE-SPINE.md:616`)*
**When** story này chạy — **lượt đầu tiên trong toàn dự án có stemmer thật trong cây mã**
**Then** chạy `english_porter_2` trên **đúng bốn chuỗi** AD-44 nêu: `dictionary` · `study` · `happy` · `run`
**And** **đầu ra THẬT** ghi vào Completion Notes thành bảng, kèm câu trả lời: chuỗi thật **có** trùng `dictionari` · `studi` · `happi` không hay không
**And** **KHÔNG sửa `ARCHITECTURE-SPINE.md`** — dev **báo cáo số**, **Winston** cầm bút *(AD là tài sản của architect; xem `deferred-work.md:282` về cùng ranh giới đó với `epics.md`)*

> 🔴 **Vì sao AC này rẻ mà đáng:** món nợ đo đạc này đang mở trong AD, chi phí đóng nó ở đây là **bốn lời gọi hàm**, và nếu không đóng bây giờ thì nó chỉ được đóng vào ngày ai đó muốn **lật** AD-44 ③ — tức đúng lúc đắt nhất và thiên lệch nhất.

### AC9 — `Jieba` khởi tạo **ĐÚNG MỘT LẦN**, và chi phí đó **đo được**

**Given** `jieba-rs` với feature `default-dict` *(mặc định)*
**When** rà mã
**Then** tồn tại **đúng một** điểm khởi tạo `Jieba`, qua `std::sync::LazyLock` *(hoặc `OnceLock`)* — **không** một lời gọi `Jieba::new()` nào nằm trong thân một hàm bị gọi lặp
**And** cổng tĩnh: chuỗi `Jieba::new()` xuất hiện ở **đúng một** vị trí mã dưới `src-tauri/src/**`

**Given** chi phí khởi tạo đó
**When** đo
**Then** thời gian `LazyLock` khởi tạo lần đầu **ghi thành số thật** vào Completion Notes, **bản release**
**And** đối chiếu với **NFR2** *(không frame nào vượt 50 ms)* — nếu số vượt 50 ms thì đó là một **bàn giao có tên** cho story đầu tiên gọi Matcher trên đường gõ *(Story 3.4)*: ai đó phải **hâm nóng** nó ngoài đường gõ. **Ghi số và bàn giao, đừng tự dựng một cơ chế hâm nóng ở story này** — chưa có đường gõ nào tồn tại để hâm nóng vào.

> 🔴 **Đo thật, không suy đoán:** `include_flate::flate!(static DEFAULT_DICT: str from "src/data/dict.txt")` *(`jieba-rs-0.10.3/src/lib.rs:104`)*, và `dict.txt` **thô là 5.071.843 byte**. `Jieba::new()` = giải nén **cộng** nạp từng dòng vào một cây `cedar`. Đó **không** phải một hằng số biên dịch — nó là công việc chạy lúc chạy, và nó xảy ra **ở lần gọi đầu tiên**, tức có thể rơi đúng vào phím đầu tiên người dùng gõ.

### AC10 — Delta **NFR6** đo lại, vì lớp từ điển jieba **hôm nay chưa sống**

**Given** `jieba-rs` và `tantivy-stemmers` đã ghim trong `Cargo.toml` từ Story 1.2 nhưng **chưa có một dòng mã nào gọi tới** *(`1-11b…md:458` — *"`core/matching/mod.rs`: 8 dòng, toàn doc-comment, 0 dòng mã"*)*
**When** story này thành lời gọi thật đầu tiên
**Then** dung lượng `.dmg` **có** và **không** phần mã story này được đo, và **chênh lệch ghi thành số cụ thể** vào Completion Notes
**And** đối chiếu trần **NFR6 = 400.000.000 byte**; payload đo thật với bảy nguồn hiện là **343.991.430 byte**, dư **56.008.570** *(`ARCHITECTURE-SPINE.md` §Deferred, hàng font — bản cập nhật 2026-08-05)*
**And** dùng **đúng phương pháp và đúng `[profile.release]`** mà Story 1.1 đã đo trên đó — **không** đổi một dòng nào của `[profile.release]` *(đổi là làm mọi số đo NFR6 trước đó hết so sánh được; `Cargo.toml:60-67` nói thẳng điều này)*

> ⚠️ **Vì sao con số này không đoán được:** LTO + `strip` + `opt-level = "s"` **có thể** đã loại bỏ `DEFAULT_DICT` khỏi bản build hôm nay vì không ai tham chiếu nó. Lời gọi `Jieba::new()` đầu tiên làm nó **sống**. Dải kỳ vọng ~2–5 MB *(dict nén)*, nhưng **đừng ghi con số kỳ vọng — hãy ghi con số đo được.**

### AC11 — **KHÔNG crate mới**, không đổi `Cargo.toml` / `Cargo.lock`

**Given** toàn bộ mã story này giao
**When** rà
**Then** `src-tauri/Cargo.toml` và `Cargo.lock` **không đổi một dòng nào**
**And** `npm run check:deps` **xanh**, `RUST_TREE_FLOOR = 200` giữ nguyên
**And** đặc biệt: **không** thêm `unicode-segmentation`, không `regex`, không `once_cell`, không `tantivy` — **cả bốn đều KHÔNG cần**:
  - tách token tiếng Anh dùng `char::is_alphanumeric` của `std`;
  - `LazyLock` nằm trong `std::sync` *(ổn định từ Rust 1.80; dự án ở `rust-version = "1.85"`, edition 2024)*;
  - `tantivy-stemmers` phơi `pub type Algorithm = fn(&str) -> Cow<str>` và `pub fn english_porter_2(&str) -> Cow<str>` — **gọi thẳng hàm**, **KHÔNG** đi qua `StemmerTokenizer`/`TokenFilter` *(hai thứ đó đòi hạ tầng `Tokenizer` của `tantivy`, mà `tantivy` chỉ là **dev-dependency** của crate kia — không có trong cây phụ thuộc của ta)*.

> 🔴 **NFR15 nói mọi phụ thuộc mới phải rà tương thích GPLv3 TRƯỚC khi thêm và ghi vào bảng Stack** *(`ARCHITECTURE-SPINE.md:658`)*. Thêm một crate ở story này không phải một lượt tiện tay — nó là một lượt phải làm hết quy trình. **Đường rẻ là đừng thêm.**

### AC12 — Ranh giới KHÔNG CHẠM

**Given** danh sách tệp story này giao
**When** đối chiếu bằng `git status`
**Then** **không** một dòng `.vue` / `.ts` / `.css` nào
**And** **không** `MessageKey` mới, **không** khoá `vi.json` mới — `npm run check:i18n` vẫn báo đúng **16 khoá · 9 placeholder**
**And** **không** chạm `core/dict/**` *(🔴 AC2)*, không `core/store/**`, không `core/scope/**`, không `core/i18n/**`, không `ports/`, không `commands/`
**And** **không** chạm `tools/dict-build/**`, **không** một lượt `cargo run` nào của `dict-build` *(🔴 mặc định là `--layer all` và nó **dựng lại cả ba** tệp `.db` ⇒ hai `sha256` trong `dict-manifest.toml` thành sai, và **không cổng nào bắt được** — `check-dict-manifest.mjs` cố ý không đọc `.db`)*
**And** **không** chạm `[profile.release]` *(AC10)*

### AC13 — Mọi cổng xanh, không hạ một sàn nào

**Given** cây mã sau story
**When** chạy toàn bộ
**Then** `npm run build` → `cargo test --locked --manifest-path src-tauri/Cargo.toml` **xanh**, và **102 ca cũ giữ nguyên xanh**
**And** sáu cổng `.mjs` **xanh**: `check:deps` · `check:i18n` · `check:commands` · `check:tokens` · `check:dict` · `check:dict-manifest`
**And** **không** hạ / nới một sàn quần thể nào: `store_boundary::RS_FLOOR = 20` · `scope_boundary::RS_FLOOR = 20` · `check-i18n::RS_FLOOR = 21` / `VUE_FLOOR = 1` · `dict_boundary::DICT_FLOOR = 1` · `dict_boundary::SRC_TAURI_RS_FLOOR = 20` · `check-dict-build::RS_FILE_FLOOR = 21`
**And** **không** nới `STORE_DIR` / `FORBIDDEN` của `store_boundary.rs`, không nới `FORBIDDEN` của `dict_boundary.rs`
**And** sàn **mới** của `matching_boundary.rs` là một sàn **thêm vào**, không phải một sàn bị hạ

---

## Tasks / Subtasks

- [x] **Task 0 — Chốt phạm vi API** *(§Quyết định phải chốt)*
  - [x] 0.1 Đọc §Quyết định; nếu không có chỉ đạo khác của Ice ⇒ đi **phương án B**
  - [x] 0.2 Ghi lựa chọn + lý do vào Completion Notes **trước** khi viết dòng mã đầu tiên

- [x] **Task 1 — Khung module và hằng cấu hình** *(AC1, AC3, AC4, AC9)*
  - [x] 1.1 Viết doc-comment module `core/matching/mod.rs` theo phong cách sẵn có của `core/dict/mod.rs`: nêu AD-17, nêu **ranh giới không `dict/` không gọi** kèm lý do đo được, nêu hai người tiêu thụ tương lai *(3.4 · 7.6)*
  - [x] 1.2 `pub enum MatchLang { Zh, En }` — doc-comment nói rõ: **tham số từ chỗ gọi**, **không** đoán từ nội dung, và nêu vì sao nó **không phải** `QueryRoute` của AD-44 *(hai câu hỏi khác nhau — xem AC4)*
  - [x] 1.3 `static JIEBA: LazyLock<Jieba>` — **đúng một** điểm khởi tạo, doc-comment ghi số `5.071.843 byte` và lý do `LazyLock`
  - [x] 1.4 Chốt hằng `HMM: bool` trong module kèm lý do; **không** phơi thành tham số công khai
  - [x] 1.5 **KHÔNG** viết một vị từ dò script nào — nếu tay tự gõ `is_han` thì dừng lại và đọc AC4

- [x] **Task 2 — Tokenize hai đường** *(AC5, AC6)*
  - [x] 2.1 `Zh`: `JIEBA.cut(text, HMM)` → `Vec<Token>` *(jieba 0.10.3 trả **`Token` có sẵn `byte_start`/`byte_end`** — đừng tự tính offset, đừng dùng `tokenize(_, TokenizeMode::Default, _)` vì nó **chính là** `cut`)*
  - [x] 2.2 `En`: tách theo `char::is_alphanumeric` của `std`, giữ **span byte** của từng token — **không** crate mới
  - [x] 2.3 Cả hai đường trả **cùng một** kiểu token mang span byte vào chuỗi gốc
  - [x] 2.4 Ca test: chuỗi rỗng · chuỗi toàn dấu câu · chuỗi lẫn Hán + Latin ⇒ không panic, span luôn là **ranh giới UTF-8 hợp lệ** *(khẳng định bằng `text.get(span)` trả `Some`)*

- [x] **Task 3 — Chuẩn hoá token** *(AC6, AC7, AC8)*
  - [x] 3.1 `Zh` ⇒ đồng nhất *(trả nguyên token)*; ghi lý do trong doc-comment: chữ Hán không có hình thái từ để chuẩn hoá
  - [x] 3.2 `En` ⇒ `to_lowercase()` **rồi** `algorithms::english_porter_2(...)` — 🔴 **đúng thứ tự đó**, crate nói *"Tokens are expected to be lowercased beforehand"*
  - [x] 3.3 Ca test khẳng định **chuỗi stem thật**: `running` · `dogs` · `studies` · `happiest` — chạy hàm rồi chép số ra, **đừng đoán trước rồi bắt hàm khớp phỏng đoán**
  - [x] 3.4 Ca test **giới hạn** *(AC8)*: `stem("went") != stem("go")` · `stem("children") != stem("child")` · `stem("mice") != stem("mouse")` — tên ca nói thẳng đây là **stemming không phải lemmatization**
  - [x] 3.5 Ca test **không phụ thuộc locale**: `normalize("I", En)` cho cùng kết quả bất kể môi trường *(khẳng định trên `to_lowercase()` của Rust — `"I"` ⇒ `"i"`)*

- [x] **Task 4 — n-gram hai đường** *(AC5, AC6)*
  - [x] 4.1 `Zh` ⇒ **n-gram ký tự**, cửa sổ trượt bằng `chars()`; 🔴 `chars().count()`, **không** `len()`
  - [x] 4.2 `En` ⇒ **token n-gram sau stemming**
  - [x] 4.3 Ca biên: `n = 0` · `n` lớn hơn quần thể · chuỗi rỗng ⇒ trả **rỗng**, không panic, không n-gram cụt
  - [x] 4.4 Ca test bằng chữ Hán **ngoài BMP** *(`𠧜`)* ⇒ n-gram vẫn đúng ký tự, không vỡ *(ca đối chứng sống cho 4.1)*

- [x] **Task 5 — `find_terms`** *(AC2, AC5, AC6, AC7 — **chỉ khi Task 0 chọn B**)*
  - [x] 5.1 Chữ ký nhận `(&str, &[&str], MatchLang)`, trả `Vec<TermMatch>` mang **chỉ số thuật ngữ** + **span byte**
  - [x] 5.2 `Zh`: khớp chính xác, và dùng ranh giới token của jieba để quyết định *"khi cần"* — ghi rõ trong doc-comment cách xử lý ca `中國` nằm trong `中國人`
  - [x] 5.3 `En`: so khớp trên **dạng đã chuẩn hoá của cả hai vế**, ⇒ `running` khớp thuật ngữ `run` *(AC7)*
  - [x] 5.4 Span trả về trỏ vào **chuỗi gốc**, không vào chuỗi đã chuẩn hoá — 🔴 Story 3.4 tô màu trên văn bản gốc; một span vào chuỗi đã hạ chữ thường **vẫn đúng độ dài với ASCII và sai với mọi thứ khác**, tức một lỗi không lộ ra trong test tiếng Anh thuần
  - [x] 5.5 Ca test: span dùng được thật — `&text[m.span]` trả đúng cụm người dùng sẽ thấy tô màu
  - [x] 5.6 **KHÔNG** xếp hạng, không ngưỡng %, không chỉ mục ngược, không cache

- [x] **Task 6 — Cổng ranh giới `matching_boundary.rs`** *(AC1, AC2, AC3, AC4, AC9)*
  - [x] 6.1 Sàn quần thể `MATCHING_FLOOR` cho `core/matching/**` *(khuôn `DICT_FLOOR` của `dict_boundary.rs:36`)*
  - [x] 6.2 Cổng: **chỉ** `core/matching/**` gõ `jieba_rs` / `tantivy_stemmers`; + **đối chứng dương** *(AC1)*
  - [x] 6.3 Cổng: `core/dict/**` có **0** lần `matching` / `jieba` / `stemmer` / `stem(`; thông báo assert nêu **AD-17 `:236` + AD-44 ③**
  - [x] 6.4 Cổng: `core/matching/**` không `use crate::core::*` / `crate::ports` / `crate::commands` *(AC3)*
  - [x] 6.5 Cổng: `Jieba::new()` xuất hiện ở **đúng một** vị trí mã dưới `src-tauri/src/**` *(AC9)*
  - [x] 6.6 Dùng lại khuôn đã có, **đừng chế khuôn mới**: `walk` · `rel_posix` *(🔴 chuẩn hoá `\` → `/` — bài học NFR14, `dict_boundary.rs:94-96`)* · `contains_forbidden_token` *(so khớp **không phân biệt hoa/thường**, `dict_boundary.rs:57`)*
  - [x] 6.7 **Không** nới `FORBIDDEN`/`STORE_DIR` của `store_boundary.rs`; không nới `FORBIDDEN` của `dict_boundary.rs`

- [x] **Task 7 — Đo và bàn giao** *(AC8, AC9, AC10)*
  - [x] 7.1 Chạy `english_porter_2` trên `dictionary` · `study` · `happy` · `run` ⇒ bảng vào Completion Notes; trả lời thẳng: **trùng hay không trùng** phỏng đoán của AD-44 ③
  - [x] 7.2 Đo thời gian khởi tạo `LazyLock<Jieba>` **bản release**; đối chiếu **NFR2 = 50 ms**
  - [x] 7.3 Đo `.dmg` **có** / **không** mã story này; ghi delta; đối chiếu trần **400.000.000 byte** *(AC10)*
  - [x] 7.4 **KHÔNG** sửa `ARCHITECTURE-SPINE.md` — báo cáo số, Winston cầm bút
  - [x] 7.5 Thêm vào `deferred-work.md`: *(a)* mermaid AD-13 `:189` còn cạnh `dict --> matching`, lệch thân Rule AD-17 `:236` — **Winston**; *(b)* `epics.md:1510` còn vế *"`dict/` dùng nó"* — **John**; *(c)* bảng phỏng đoán Porter của AD-44 ③ nay **có số đo thật** — **Winston**

- [x] **Task 8 — Cổng cuối** *(AC13)*
  - [x] 8.1 `npm run build` *(**BẮT BUỘC TRƯỚC** `cargo test` — `generate_context!` nhúng `dist/` lúc biên dịch)*
  - [x] 8.2 `cargo test --locked --manifest-path src-tauri/Cargo.toml`
  - [x] 8.3 Sáu cổng `.mjs`
  - [x] 8.4 `git status` đối chiếu §AC12; điền **File List** và **Completion Notes**

### Review Findings

- [x] [Review][Patch] *(đã vá)* `find_terms` (En) có thể nối thuật ngữ nhiều từ XUYÊN ranh giới câu — vd. thuật ngữ `"fast dog"` khớp `"fast. Dog"` bắc qua dấu chấm câu, vì `tokenize` (En) coi dấu chấm câu và khoảng trắng là DẤU TÁCH giống hệt nhau, và `find_terms` nối các token liền kề mà không xét dấu tách nào nằm giữa. Đã kiểm chứng bằng thực thi thật (`find_terms("The wolf ran fast. Dog barked loudly.", &["fast dog"], MatchLang::En)` trả về span cắt ra đúng `"fast. Dog"`). **Ice quyết định:** chặn nối token liền kề trong nhánh `En` của `find_terms` khi dấu tách nằm giữa hai token chứa `.`/`!`/`?`/xuống dòng — rẻ, không cần crate mới, không vi phạm AC nào của 1.12. [`src-tauri/src/core/matching/mod.rs:544-576`]
- [x] [Review][Patch] *(đã vá)* Đường `En` của `tokenize` dùng `char::is_alphanumeric()` — hàm này nhận CẢ chữ Hán/script khác là "chữ cái" theo Unicode, nên một đoạn lẫn chữ Latin và Hán (vd. `"hello世界world"`) bị dính thành MỘT token duy nhất vô nghĩa thay vì tách theo script. **Ice quyết định:** đổi thành `char::is_ascii_alphanumeric()` — chặn được lỗi dính script, đánh đổi lấy việc chữ Latin có dấu (vd. `café`) bị cắt sai thành `caf`. Ghi lý do đánh đổi vào doc-comment của `tokenize` khi vá. [`src-tauri/src/core/matching/mod.rs:307-330`]
- [x] [Review][Patch] *(đã vá)* Cổng `the_matching_module_is_a_leaf_in_the_dependency_graph` (AC3) chỉ khớp chuỗi có tiền tố `"use "` (`MATCHING_FORBIDDEN_USES`) qua `.contains()` trần, khác với MỌI cổng anh em khác trong cùng tệp (AC1/AC2/AC4 đều dùng `contains_forbidden_token` — không phân biệt hoa/thường, khớp token trần không cần tiền tố `use`) và khác với khuôn tiền lệ `store_boundary.rs::core_store_does_not_depend_on_tauri` (khớp cả dạng trần `"tauri::"`). Một lời gọi đủ điều kiện KHÔNG qua `use` (vd. `crate::core::dict::foo()` viết thẳng trong thân hàm) sẽ lọt qua cổng này mà không bị bắt — đúng lớp vi phạm mà AC3/AD-13 tồn tại để chặn. Sửa: đổi `MATCHING_FORBIDDEN_USES` thành dạng trần (`"crate::core::"`, `"crate::ports"`, `"crate::commands"`, `"super::"`) và dùng qua `contains_forbidden_token`, đúng khuôn các cổng anh em. [`src-tauri/tests/matching_boundary.rs:64-73,342-372`]
- [x] [Review][Patch] *(đã vá)* Nhánh `Zh` của `find_terms` chỉ chặn `term.is_empty()`, không chặn thuật ngữ chỉ gồm dấu tách (khoảng trắng) — khác với nhánh `En` (đã có test khẳng định `"   "` không bao giờ khớp) và khác với chính lời hứa trong doc-comment của `find_terms` rằng "thuật ngữ rỗng hoặc chỉ gồm dấu tách không bao giờ khớp" (phát biểu như một hợp đồng ngôn ngữ-trung lập). Nếu jieba tách một chuỗi khoảng trắng thành một token riêng, một thuật ngữ Glossary toàn khoảng trắng có thể khớp đúng token đó. Sửa: thêm cùng điều kiện chặn đã dùng ở nhánh `En` vào nhánh `Zh` trước khi quét. [`src-tauri/src/core/matching/mod.rs:508-543`; thiếu ca test ở `tests/matching_contract.rs:470-481`]
- [x] [Review][Defer] Các cổng ranh giới trong `matching_boundary.rs` (và `dict_boundary.rs`/`store_boundary.rs` trước đó) là phép quét CHỮ, không phải phân tích đồ thị gọi hàm ngữ nghĩa — một lớp bọc re-export dưới tên khác (vd. `pub use matching::find_terms as glossary_probe;` trong `core/mod.rs`) có thể để `core/dict/**` gọi vào Matcher mà không chạm token cấm nào. Đây là giới hạn CÓ SẴN của cả khuôn "cổng quét chữ" dùng xuyên dự án từ Story 1.9, không phải do story 1.12 gây ra — deferred, pre-existing.
- [x] [Review][Defer] Không có test nào cưỡng chế lời hứa "không chạm filesystem/database/mạng" (AD-15) mà doc-comment module tuyên bố — đúng hôm nay qua rà tay (không có lời gọi I/O nào trong diff), nhưng không gì bắt được một lượt thêm I/O trong tương lai — deferred, pre-existing (khuôn cổng ranh giới hiện tại chưa có tiền lệ kiểm loại này).
- [x] [Review][Defer] Vài con số "đo được" gắn cứng trong doc-comment/`deferred-work.md` (`dict.txt` = 5.071.843 byte; khởi tạo `Jieba` 179–329 ms) không được một test nào khẳng định, sẽ lặng lẽ lạc hậu khi dữ liệu/phụ thuộc đổi — deferred, low priority (rủi ro tài liệu, không phải rủi ro đúng/sai).
- [x] [Review][Defer] `ngrams` và `find_terms` mỗi hàm tự tokenize/normalize lại toàn bộ văn bản đầu vào — không có bề mặt API nào để tái dùng token đã tính, nên một người tiêu thụ tương lai (Story 7.6) cần cả hai sẽ trả giá tokenize hai lần mỗi segment — deferred to Story 3.4/7.6 consumer.
- [x] [Review][Defer] Văn bản chuẩn hoá NFD (dấu tổ hợp, vd. clipboard macOS) tokenize/stem khác với văn bản NFC cùng nội dung — cùng lớp vấn đề chuẩn hoá Unicode đã ghi nhận cho một module khác ở `deferred-work.md` (từ Story 1.11, dòng về `core/dict/mod.rs:242`) — deferred, pre-existing class of issue, low priority.

---

## Dev Notes

### 🎯 Hình dạng MỤC TIÊU — suy ra từ AC thật của story thật, đừng phát minh thêm

```rust
// src-tauri/src/core/matching/mod.rs

use std::borrow::Cow;
use std::ops::Range;
use std::sync::LazyLock;

use jieba_rs::Jieba;
use tantivy_stemmers::algorithms::english_porter_2;

/// Ngôn ngữ khớp — **THAM SỐ từ chỗ gọi**, không đoán từ nội dung (AC4).
///
/// không Đây **KHÔNG** phải `core::dict::QueryRoute`. `QueryRoute` trả lời *"tra vào bảng nào
/// của tệp `.db` nào"* — một thuộc tính của **hình dạng chuỗi truy vấn** (AD-44 ①). Kiểu
/// này trả lời *"khớp thuật ngữ trong văn bản của MỘT Tác phẩm"*, và ngôn ngữ nguồn của
/// Tác phẩm là một trường **bất biến trong `meta.json`, đặt lúc tạo**.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MatchLang { Zh, En }

/// Một token cùng **span byte vào chuỗi GỐC**.
///
/// 🔴 Span là byte, không phải chỉ số ký tự: Story 3.4 tô màu thuật ngữ trong Panel
/// Source (`epics.md:2528`) và nó cắt chuỗi bằng byte.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MatchToken<'a> {
    pub text: &'a str,
    pub span: Range<usize>,
}

/// Một lượt khớp thuật ngữ. `term_index` trỏ vào lát `terms` mà chỗ gọi truyền vào —
/// không phải một id Glossary (module này không biết Glossary tồn tại, AC3).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TermMatch {
    pub term_index: usize,
    pub span: Range<usize>,
}

pub fn tokenize(text: &str, lang: MatchLang) -> Vec<MatchToken<'_>>;

/// Dạng chuẩn hoá của MỘT token. `Zh` ⇒ đồng nhất. `En` ⇒ hạ chữ thường **rồi** stem.
pub fn normalize(token: &str, lang: MatchLang) -> Cow<'_, str>;

/// `Zh` ⇒ n-gram **ký tự**. `En` ⇒ token n-gram **sau** stemming. `n` vượt quần thể ⇒ rỗng.
pub fn ngrams(text: &str, lang: MatchLang, n: usize) -> Vec<String>;

/// Điểm vào của Glossary (FR51) — chỉ có ở **phương án B**.
pub fn find_terms(text: &str, terms: &[&str], lang: MatchLang) -> Vec<TermMatch>;
```

**⚠️ Đây là hình dạng ĐỀ XUẤT, không phải một AC.** Thứ **là** AC: ngôn ngữ là tham số *(AC4)*, span là byte vào chuỗi gốc *(Task 5.4)*, hai đường có hai cơ chế đúng như AC5/AC6, và không hàm nào sống mà không có test. Tên hàm đổi được nếu có lý do tốt hơn — **ghi lý do vào Completion Notes**.

### 📚 API hai crate — **đọc từ nguồn ĐÃ TẢI**, không từ trí nhớ

*(Cả hai đã ghim `=` trong `Cargo.toml:56-57` từ Story 1.2. Rà giấy phép NFR15 lượt hai đã xong: `jieba-rs` MIT ⚠️ · `tantivy-stemmers` BSD-3-Clause ✓ — `ARCHITECTURE-SPINE.md:701`, `:704`.)*

| Dữ kiện | Giá trị — **đọc từ `~/.cargo/registry/src/…`** |
|---|---|
| `jieba_rs::Jieba::new()` | Có, **cần feature `default-dict`** — mặc định đã bật |
| `Jieba::empty()` | Có — instance **không** dict, hữu ích cho test không cần dict |
| `Jieba::cut(&self, s: &'a str, hmm: bool)` | ⇒ **`Vec<Token<'a>>`** *(lib.rs:933)* — 🔴 **đã mang offset**, đừng tự tính |
| `Token` | `{ word: &'a str, start, end (**ký tự**), byte_start, byte_end (**byte**) }` *(lib.rs:242)* |
| `Jieba::tokenize(s, TokenizeMode::Default, hmm)` | **chính là** `cut(s, hmm)` *(lib.rs:1019)* — đừng thêm một lớp gọi vòng |
| `Jieba::cut_for_search` | `cut` **cộng** các gram con có trong dict — **không** dùng cho khớp thuật ngữ *(sinh span chồng nhau)* |
| Dict nhúng | `include_flate::flate!(static DEFAULT_DICT: str from "src/data/dict.txt")` *(lib.rs:104)*; `dict.txt` **thô 5.071.843 byte** |
| `idf.txt` / `posseg.txt` | **không** vào bản build — chúng thuộc feature `tfidf`/`textrank`, **không bật** |
| `tantivy_stemmers::algorithms::Algorithm` | `pub type Algorithm = fn(&str) -> Cow<str>` *(algorithms.rs:5)* |
| `algorithms::english_porter_2` | `pub fn english_porter_2<'a>(input: &'a str) -> Cow<'a, str>` *(algorithms.rs:81)* — **gọi thẳng** |
| Feature mặc định của `tantivy-stemmers` | `default = ["english_porter_2"]` ⇒ **không** cần đổi `Cargo.toml` |
| 🔴 Điều kiện tiên quyết của stemmer | *"❗️❗️ **Tokens are expected to be lowercased beforehand**"* *(lib.rs:12)* |
| `StemmerTokenizer` / `StemmerFilter` | **KHÔNG dùng** — đòi trait `Tokenizer`/`TokenFilter` của `tantivy-tokenizer-api`; `tantivy` chỉ là **dev-dependency** của crate kia, không có trong cây của ta |

### 🪤 Mười một bẫy — **tám cái đã cắn một lần rồi ở story trước**

1. 🔴 **`chars().count()`, KHÔNG `len()`.** Bẫy đắt nhất của Story 1.11 *(kiểm chứng bằng 5 ca đột biến đỏ)*. `"山".len()` = **3**. N-gram ký tự cắt theo byte không chỉ sai — nó **panic** ở biên không phải ranh giới UTF-8.
2. 🔴 **`Jieba::new()` trong thân một hàm được gọi lặp = một hồi quy NFR2 không ai thấy trong test.** Test chạy một lần; người dùng gõ một nghìn lần. `LazyLock`, và có **cổng** *(AC9)*.
3. 🔴 **Hạ chữ thường TRƯỚC stem, không phải sau.** Crate nói thẳng. Sai thứ tự ⇒ `Running` và `running` cho hai stem khác nhau ⇒ **đúng lỗ chữ HOA mà AD-44 ③ vừa bịt ở đường tra cứu, tái sinh ở đường khớp.**
4. 🔴 **`to_lowercase()` của Rust, không phải một phép fold theo locale.** AD-44 ③ đã trả giá cho bài học này: *"cùng một truy vấn cho hai kết quả trên hai máy"*.
5. 🔴 **Span phải trỏ vào chuỗi GỐC.** Một span đo trên chuỗi đã chuẩn hoá **vẫn đúng với ASCII thuần và sai với mọi thứ khác** — tức một lỗi đi qua trọn bộ test tiếng Anh mà không đỏ một ca nào. Khẳng định bằng `&text[span]`.
6. 🟡 **không Chuỗi tiếng Việt CÓ DẤU ở vị trí mã bị `check-i18n.mjs` Kiểm A bắt.** `src-tauri/src/core/matching/**` **không** nằm trong `EXEMPT` *(chỉ `src-tauri/tests/**`, `src/selftest/**`, `tools/**` được miễn)*. **Doc-comment và comment có dấu thì hợp lệ**; `panic!` / `debug_assert!` / `format!` / `Display` thì **không**. **Tên test = tiếng Anh `snake_case`** — cổng đã bắt đúng ca này một lần ở một tên test của Story 1.10b.
7. 🟡 **Đừng dùng `tempfile`** trong `src-tauri` — nó là dev-dep của `tools/dict-build`, không của `src-tauri` *(`src-tauri/Cargo.toml` **không có** `[dev-dependencies]`)*. Story này không cần tệp tạm nào cả.
8. 🟡 **`rel_posix` phải chuẩn hoá `\` → `/`.** Bài học NFR14 ở `store_boundary.rs:68-73`: `starts_with("core/matching")` trên Windows so với `core\matching` và **không bao giờ khớp** ⇒ cổng quét 0 tệp và chỉ đỏ trên **một** nhánh của ma trận CI.
9. 🟡 **Cổng so khớp token phải KHÔNG phân biệt hoa/thường** khi thứ nó canh là một danh định có thể viết nhiều kiểu — `dict_boundary.rs:57` đã phải vá đúng chỗ này ở lượt review 1.11b.
10. 🟡 **Sàn quần thể là để bắt cây BỊ CẮT, không phải để bắt việc thêm tệp.** Đặt **dưới** số thật *(`dict_boundary.rs:31-36`)*. Một `MATCHING_FLOOR` đặt bằng đúng số tệp hôm nay sẽ đỏ vào ngày Epic 3 thêm một tệp — và nó sẽ bị hạ chứ không được đọc.
11. 🟡 **`cut_for_search` sinh span CHỒNG NHAU.** Nó thêm các gram con có trong dict *(lib.rs:953-1008)*. Với khớp thuật ngữ, hai span chồng nhau = hai lượt tô màu chồng nhau ở 3.4. Dùng **`cut`**.

### 🧾 Trạng thái baseline — biết trước để không hoảng

- 🟡 **`cargo fmt --check` ĐỎ từ baseline: 29 chỗ trên 12 tệp** *(đo thật ở 1.11b)* — `core/i18n/mod.rs` · `core/scope/resolve.rs` · `core/store/{mod,pragmas,schema}.rs` · `lib.rs` · sáu tệp `tests/**`. **CI không chạy `fmt` cũng không chạy `clippy`.** ⇒ **Đừng** chạy `cargo fmt` toàn cây; đừng nhét 29 chỗ đó vào diff. **Tệp mới thì viết cho fmt sạch** *(chạy `rustfmt` riêng cho tệp của story này)*.
- 🟡 `core/store/mod.rs:119` doc-comment ghi *"Năm loại kho"* trong khi `StoreKind` có **4** biến thể — lệch có **từ trước**, **không** sửa ở story này.
- 🟡 **Một lượt biên dịch `--release` có thể hỏng RỒI TỰ HẾT:** `the crate 'wry' requires panic strategy 'abort'…` × 144, do artifact release cũ còn trong `target/` gặp `panic = "abort"` của `[profile.release]`. **Chạy lại đúng cùng lệnh** ⇒ xanh. Nếu gặp lại: `cargo clean --release` rồi chạy lại. *(Không phải lỗi của story này — 1.11b đã gặp và ghi lại.)*
- 🟡 Cây làm việc **sạch** ở baseline `5edbe0e` *(`git status` rỗng)* — khác với lúc 1.11b bắt đầu.

### 📂 Trạng thái mã HÔM NAY — đọc trước khi sửa

| Tệp | Hôm nay | Story này đổi gì |
|---|---|---|
| `src-tauri/src/core/matching/mod.rs` | **8 dòng, toàn doc-comment, 0 dòng mã.** Doc-comment **đã ghi sẵn** *"Crate dành cho module này: `jieba-rs` (nhánh tiếng Trung) · `tantivy-stemmers` (nhánh tiếng Anh). Cả hai đã ghim ở `Cargo.toml`, **chưa có mã nào gọi tới**."* | **viết lại + mở rộng** — đây là toàn bộ story |
| `src-tauri/src/core/mod.rs` | `pub mod matching;` **đã khai** | **không đổi** |
| `src-tauri/src/core/dict/{mod,query}.rs` | Đường tra cứu hoàn chỉnh, 5 nhánh, **0** lời gọi tới `matching` | 🔴 **KHÔNG CHẠM** *(AC2 có cổng)* |
| `src-tauri/src/core/{glossary,tm}/mod.rs` | mỗi tệp **4 dòng doc-comment, 0 dòng mã** | **không đổi** — người tiêu thụ là Epic 3 / Epic 7 |
| `src-tauri/Cargo.toml:56-57` | `jieba-rs = "=0.10.3"` · `tantivy-stemmers = "=0.4.0"`, **kèm comment `core::matching — … (AD-17)`** | **không đổi một dòng** *(AC11)* |
| `src-tauri/tests/` | 8 tệp, **102 ca** *(`dict_lookup` 37 · `scope_contract` 17 · `store_contract` 16 · `config_invariants` 15 · `dict_boundary` 5 · `scope_boundary` 5 · `ipc_contract` 5 · `store_boundary` 4)* | **+** `matching_contract.rs` · `matching_boundary.rs` |

Quần thể `.rs` hiện tại: **28** dưới `src-tauri/src/**`, **36** tính cả `tests/**`.

### 🧪 Quy ước test — khuôn đã có, đừng chế khuôn mới

- **Vị trí:** `src-tauri/tests/matching_contract.rs` *(hành vi)* + `matching_boundary.rs` *(cổng tĩnh)*. Đúng cặp tên đã dùng cho `store_*` và `scope_*`.
- **Import:** `use auratranslate_lib::core::matching::{MatchLang, normalize, ngrams, tokenize};`
- **Tên test = câu khẳng định đầy đủ, `snake_case`, tiếng Anh** — không dấu tiếng Việt *(`check-i18n` đã bắt đúng ca này một lần)*.
- **Ca giới hạn phải có tên nói ra giới hạn** *(AC8)*: ví dụ `stemming_is_not_lemmatization_irregular_forms_do_not_reach_their_lemma`.
- **Không tệp dữ liệu ngoài** — mọi ca dựng chuỗi ngay trong test. Story này không cần một tệp `.db` nào, tức **100% ca chạy được trong CI** *(khác 1.11/1.11b, nơi phần đắt nhất phải `#[ignore]`)*. **Đó là một lợi thế — hãy tiêu nó, đừng đánh rơi.**
- **Ba phép đo của Task 7** *(stemmer thật · thời gian init · delta `.dmg`)* **không phải là ca test có ngưỡng** — chúng là **số ghi vào Completion Notes**. Đừng đặt một `assert!` thời gian vào CI: máy runner không phải máy dev, và một cổng thời gian nhiễu sẽ bị `#[ignore]` trong tuần.
- **Đột biến:** trước khi đóng story, **phá từng cổng mới rồi chạy lại** để chứng minh nó đỏ được, rồi hoàn nguyên. Khuôn bảng M1–M7 của 1.11b — chép cách trình bày đó vào Debug Log References.

### 🔧 Lệnh — chép nguyên

```sh
# BẮT BUỘC TRƯỚC cargo test (generate_context! nhúng dist/ lúc biên dịch)
npm run build

cargo test --locked --manifest-path src-tauri/Cargo.toml
cargo test --manifest-path src-tauri/Cargo.toml --test matching_contract
cargo test --manifest-path src-tauri/Cargo.toml --test matching_boundary

npm run check:deps && npm run check:i18n && npm run check:commands \
  && npm run check:tokens && npm run check:dict && npm run check:dict-manifest

# fmt CHỈ cho tệp của story này — KHÔNG chạy toàn cây (29 chỗ đỏ từ baseline)
rustfmt --edition 2024 src-tauri/src/core/matching/mod.rs \
  src-tauri/tests/matching_contract.rs src-tauri/tests/matching_boundary.rs

# Đo delta NFR6 (AC10) — cùng phương pháp Story 1.1, KHÔNG đổi [profile.release]
npm run tauri build -- --bundles dmg
```

**KHÔNG chạy** `cargo run --manifest-path tools/dict-build/Cargo.toml` — mặc định `--layer all` dựng lại **cả ba** tệp `.db` và làm hai `sha256` trong `dict-manifest.toml` thành sai **mà không cổng nào bắt được**.

### 🧭 Trí nhớ Git — năm commit gần nhất

| Commit | Nói lên điều gì cho story này |
|---|---|
| `5edbe0e` *Add assertion for `StoreKind::Dict`…* | **baseline**; cây sạch |
| `5a68df7` *Update deferred work…* | `deferred-work.md:278-283` — **phát hiện AD-44 về stemming**, và câu *"**Story 1.12 vẫn dựng Matcher đầy đủ** cho Glossary (FR51) và TM (FR61)"*. Đọc nguyên mục này trước khi gõ dòng đầu. |
| `dd7af61` *Add `VIWIKTIONARY_EN` source…* | nguồn tiếng Anh vào `dict-core.db` — **không** việc gì cho story này ở tầng dữ liệu |
| `ed8ce52` / `a3ed5cd` | Khuôn **fixture dựng trong test** + integration test — cùng triết lý `matching_contract.rs` dùng, không trừ phần fixture *(story này không cần `.db`)* |

**Nhịp đã đặt qua 1.9 → 1.11b:** một story = một module + hai tệp test *(hành vi + cổng)* + doc-comment mang **lý do** chứ không chỉ mang **mô tả** + Completion Notes mang **số đo thật**. Giữ nguyên nhịp đó.

### 🌐 Thông tin kỹ thuật mới nhất

- **`std::sync::LazyLock`** ổn định từ **Rust 1.80**; dự án ở `rust-version = "1.85"`, `edition = "2024"` ⇒ **dùng được, không cần `once_cell`**. `LazyLock` hơn `OnceLock` ở chỗ hàm khởi tạo nằm **cạnh** khai báo, không rải ra chỗ gọi.
- **`jieba-rs` 0.10.3** dùng `edition 2024`, ⇒ không xung đột toolchain. Feature `default-dict` kéo `include-flate`; `textrank`/`tfidf` **không bật** nên `idf.txt` *(6.200.957 byte)* và `posseg.txt` *(2.551.696 byte)* **không vào bản build** — chỉ `dict.txt` vào.
- **`tantivy-stemmers` 0.4.0** dùng `edition 2021`; phụ thuộc `aho-corasick` · `precis-*` · `unicode-normalization` · `tantivy-tokenizer-api` · `serde`. Tất cả **đã** nằm trong `Cargo.lock` từ Story 1.2 ⇒ AC11 *(không đổi lock)* là khả thi.
- ⚠️ **Tệp `LICENSE` của `tantivy-stemmers` còn sót placeholder `{{ project }}` chưa thay** — lỗi hình thức của thượng nguồn, **không** đổi bản chất giấy phép *(BSD-3-Clause, đã phân xử bằng mắt ở `ARCHITECTURE-SPINE.md:704`)*. Đừng mở lại câu hỏi này ở story này.

### Project Structure Notes

Cây nguồn *(`ARCHITECTURE-SPINE.md:790`)* **đã dành sẵn chỗ** — dòng chú thích viết trước khi mã tồn tại:

```text
core/
  matching/      # jieba + stemmer — DÙNG CHUNG (AD-17)        ← ✅ story này
  dict/          # ba nhánh zh (AD-26) + hai nhánh en (AD-44)  ← 🔴 KHÔNG CHẠM
  glossary/      # + bảng chờ đề xuất (AD-20)                  ← người tiêu thụ, Epic 3
  tm/            # khoá theo cặp văn bản (AD-6)                ← người tiêu thụ, Epic 7
ports/           # DictionarySource · … (AD-2)                 ← không Story 1.13
```

**Không có biến variance nào.** Mọi tệp story này chạm nằm trong `src-tauri/src/core/matching/**` và `src-tauri/tests/**`. Quy ước: module Rust `snake_case`; **một module cho một khái niệm miền** ⇒ nếu `mod.rs` phình quá, tách theo **khái niệm** *(`zh.rs` / `en.rs`)*, **không** theo tầng kỹ thuật *(không `helpers.rs`, không `utils.rs`)*. Nhóm năng lực C1–C10 **không** xuất hiện trong tên tệp.

### References

- [Source: `_bmad-output/planning-artifacts/epics.md#Story 1.12: Matcher dùng chung`] — `:1497-1527` *(🔴 vế `dict/` dùng nó ở `:1510` **đã bị AD-17 lật** — xem đầu story)*
- [Source: `…/epics.md#Story 3.4`] — `:2518-2554` *(người tiêu thụ thứ nhất: khớp chính xác cho zh, stemming cho en, **đánh dấu bằng màu** ⇒ cần span)*
- [Source: `…/epics.md#Story 7.6`] — `:4932-4959` *(người tiêu thụ thứ hai: n-gram ký tự cho zh, token n-gram sau stemming cho en; *"một biến thể Glossary bắt được thì TM cũng bắt được"*)*
- [Source: `…/epics.md#Requirements Inventory`] — `:156` *(FR40)* · `:172` *(FR51)* · `:200` *(FR61)* · `:420` *(tóm tắt AD-17)*
- [Source: `…/ARCHITECTURE-SPINE.md#AD-17`] — `:230-236` 🔴 **chủ; thân Rule ở `:236` là mệnh đề quyết định**
- [Source: `…/ARCHITECTURE-SPINE.md#AD-44`] — `:604-618` *(③ stemming không trên đường nóng, kèm ⚠️ ở `:616` về bảng phỏng đoán Porter — **AC8 đóng nó**)* · `:624-631` *(⑤ ranh giới mã-riêng-theo-ngôn-ngữ)*
- [Source: `…/ARCHITECTURE-SPINE.md#AD-13`] — `:165-198` 🔵 **mermaid `:189` còn cạnh `dict --> matching`, LỆCH thân Rule AD-17 — Winston**
- [Source: `…/ARCHITECTURE-SPINE.md#Consistency Conventions`] — `:640` *(module Rust)* · `:641` *(tên tệp)* · `:652` *(chuỗi giao diện)* · `:658` *(giấy phép)*
- [Source: `…/ARCHITECTURE-SPINE.md#Stack`] — `:681-682` *(`jieba-rs` 0.10.3 MIT ⚠️ · `tantivy-stemmers` 0.4.0 BSD-3-Clause ✓)* · `:695`, `:701`, `:704` *(rà giấy phép lượt hai)*
- [Source: `…/ARCHITECTURE-SPINE.md#Deferred`] — hàng *Stemming trên đường tra cứu TỪ ĐIỂN tiếng Anh* `:852` · hàng font *(trần NFR6 = **400.000.000 byte**, payload **343.991.430**)*
- [Source: `…/ARCHITECTURE-SPINE.md#Cây nguồn`] — `:788-795`
- [Source: `_bmad-output/implementation-artifacts/deferred-work.md`] — `:278-283` *(phát hiện AD-44 về stemming; `:281` nói **Story 1.12 vẫn dựng Matcher đầy đủ**; `:282` chủ sở hữu các chỗ lệch)*
- [Source: `_bmad-output/implementation-artifacts/1-11b-duong-tra-cuu-tieng-anh.md`] — AC10 *(không Matcher)* · §Bẫy kế thừa · §Quy ước test · §Trạng thái baseline · bảng đột biến M1–M7
- [Source: `_bmad-output/implementation-artifacts/1-11-ba-nhanh-truy-van-tieng-trung.md`] — bẫy `len()` vs `chars().count()`, khuôn cổng ranh giới
- [Source: `src-tauri/src/core/matching/mod.rs`] *(8 dòng, 0 mã)* · [`core/mod.rs`] · [`core/dict/mod.rs`] *(phong cách doc-comment mẫu)* · [`core/{glossary,tm}/mod.rs`]
- [Source: `src-tauri/Cargo.toml`] — `:56-57` *(hai crate đã ghim)* · `:60-67` *(`[profile.release]` — **không đổi**)*
- [Source: `src-tauri/tests/dict_boundary.rs`] — `:27-64` *(khuôn `DICT_DIR` · `DICT_FLOOR` · `FORBIDDEN` · `contains_forbidden_token` không phân biệt hoa/thường)* · `:88-104` *(`src_root` · `rel_posix` · `walk`)* · `:197-203` *(khuôn đối chứng dương)*
- [Source: `src-tauri/tests/store_boundary.rs`] — `:37-62` *(khuôn `STORE_DIR` · `RS_FLOOR` · `FORBIDDEN`)* · `:68-73` *(bài học NFR14 dấu `\`)*
- [Source: `scripts/check-i18n.mjs`] — `:126-148` *(`EXEMPT` — 🔴 `core/matching/**` **KHÔNG** được miễn)*
- [Source: `scripts/check-deps.mjs`] — `:50-51` *(`RUST_TREE_FLOOR = 200` · `NPM_TREE_FLOOR = 30`)*
- [Source: `~/.cargo/registry/src/…/jieba-rs-0.10.3/src/lib.rs`] — `:104` *(dict nhúng)* · `:233-256` *(`TokenizeMode`, `Token`)* · `:322-380` *(`empty`/`new`/`load_default_dict`)* · `:933-1032` *(`cut` · `cut_all` · `cut_for_search` · `tokenize`)*
- [Source: `~/.cargo/registry/src/…/tantivy-stemmers-0.4.0/src/lib.rs`] — `:12` *(🔴 *"Tokens are expected to be lowercased beforehand"*)* · `:33-46` *(`TokenFilter`/`StemmerFilter` — **không** dùng)* · [`src/algorithms.rs`] — `:5` *(`type Algorithm`)* · `:81` *(`english_porter_2`)* · [`Cargo.toml`] — `default = ["english_porter_2"]`
- [Source: `_bmad-output/planning-artifacts/prds/prd-AuraTranslate-2026-08-02/prd.md`] — `:765-774` *(ngôn ngữ nguồn là trường **bất biến**, đặt lúc tạo — nền của AC4)*

---

## Câu hỏi để lại cho người duyệt

1. 🔴 **Phạm vi API — A hay B?** *(§Quyết định phải chốt)*. Khuyến nghị **B**. **Ice** chốt. Nếu chọn A, Task 5 bỏ và Completion Notes phải nói rõ *ai* sẽ lắp vòng khớp, và *bằng cách nào* AD-17 vẫn đứng.
2. 🔵 **Mermaid AD-13 `:189` còn cạnh `dict --> matching`** — lệch thân Rule AD-17 `:236`. Chủ sở hữu **Winston**. Không chặn story này *(dev theo thân Rule)*, nhưng nó sẽ làm lệch mọi lượt đọc kiến trúc sau.
3. 🟡 **`epics.md:1510` còn vế *"`dict/` dùng nó"*** — chủ sở hữu **John (PM)**. Cùng lượt sửa với `:1491` mà `deferred-work.md:282` đã ghi.
4. **Cờ `hmm` của jieba** *(AC5)* — story chốt **một** giá trị trong module. Nếu Winston muốn nó thành một mệnh đề của AD-17 *(để Epic 3 và Epic 7 không mở lại)*, đây là chỗ để nói.
5. **Nếu thời gian init `Jieba` vượt NFR2 = 50 ms** *(AC9)* — story **ghi số và bàn giao cho 3.4**, không tự dựng cơ chế hâm nóng. Xác nhận cách xử lý này đúng ý.

---

## Dev Agent Record

### Agent Model Used

`claude-opus-5` (Claude Code, BMad `dev-story`) — 2026-08-05.

### Debug Log References

#### 🧬 Bảng đột biến M1–M11 — mỗi cổng mới đều **đã được chứng minh đỏ được**, rồi hoàn nguyên

⚠️ Khuôn chép từ bảng M1–M7 của Story 1.11b. Mỗi hàng: phá **đúng một** mệnh đề, chạy lại, xác nhận cổng đỏ, `cp` bản gốc về, `diff -q` xác nhận cây sạch.

| # | Đột biến | Cổng phải đỏ | Kết quả |
|---|---|---|---|
| **M1** | Thay `use tantivy_stemmers::…` bằng một hàm giả cùng tên trong module | `the_matching_module_actually_uses_both_language_crates` *(đối chứng dương)* | 🔴 **ĐỎ** |
| **M2a** | Thêm `use jieba_rs::Jieba;` vào `core/dict/mod.rs` | `only_the_matching_module_ever_names_the_two_language_crates` | 🔴 **ĐỎ** |
| **M2b** | *(cùng đột biến)* | `the_dictionary_lookup_path_never_calls_the_matcher` | 🔴 **ĐỎ** |
| **M3** | Thêm `use crate::core::dict as _unused_dict;` vào `core/matching/mod.rs` | `the_matching_module_is_a_leaf_in_the_dependency_graph` | 🔴 **ĐỎ** |
| **M4a** | Thêm `fn is_han(c: char) -> bool { c > '\u{4DFF}' }` vào `core/matching/mod.rs` | `the_matching_module_never_guesses_the_language_from_the_content` | 🔴 **ĐỎ** — *"1 chỗ dưới `core/matching/**` tự đoán ngôn ngữ: `core/matching/mod.rs:516`"* |
| **M4b** | *(cùng đột biến)* | `dict_boundary::exactly_one_definition_of_is_han_exists_under_src_tauri` **(cổng CŨ của 1.11b)** | 🔴 **ĐỎ** — *"2 định nghĩa `is_han` dưới `src-tauri/**`"*. ✅ Xác nhận cổng cũ **vẫn sống** và story này không nới nó |
| **M5** | Thêm `pub fn second_instance() -> Jieba { Jieba::new() }` | `the_jieba_dictionary_is_constructed_at_exactly_one_place` | 🔴 **ĐỎ** |
| **M6** | Đổi `static JIEBA: LazyLock<Jieba>` thành `fn jieba_now() -> Jieba { Jieba::new() }` | `the_single_jieba_instance_is_actually_lazily_initialised_once` *(đối chứng dương)* | 🔴 **ĐỎ** — ✅ đúng lỗ mà M5 một mình không bịt được |
| **M7** | Đổi `MATCHING_DIR` thành `"core/matchingXX"` *(đường dẫn gõ sai)* | `the_scanned_tree_is_large_enough_to_be_real` | 🔴 **ĐỎ** — ✅ *"cây rỗng đọc thành sạch"* bị chặn |
| **M8** | n-gram `Zh` đếm theo **byte** thay vì ký tự *(`text.as_bytes()…as char`)* | `chinese_ngram_population_is_counted_in_characters_never_in_bytes` + 2 ca khác | 🔴 **ĐỎ 3/3** — `"中國"` ra `["ä¸", "¸\u{ad}", …]` |
| **M9** | Đảo thứ tự: stem **trước** rồi mới hạ chữ thường | `english_normalization_lowercases_before_stemming_so_case_never_splits_a_term` | 🔴 **ĐỎ** |
| **M10** | Bỏ phép chặn theo ranh giới token ở đường `Zh` *(`if true`)* | `chinese_term_matching_is_exact_and_arbitrated_by_jieba_token_boundaries` | 🔴 **ĐỎ** — ca `文` trong `文化` |
| **M11** | Span `En` trả về bắt đầu từ `0` thay vì đầu token | `english_match_spans_point_into_the_original_text_even_after_non_ascii_bytes` | 🔴 **ĐỎ** |

**Sau mỗi đợt:** `diff -q` giữa bản lưu và cây làm việc ⇒ `mod.rs KHOP ban story` · `dict/mod.rs NGUYEN VEN` · `matching_boundary.rs NGUYEN VEN`.

⚠️ **Ghi lại một sai sót của chính lượt chạy đột biến, không giấu:** hai lượt chạy hàng loạt đầu tiên báo M4 và M8 *"xanh"*. Nguyên nhân là **escape trong script `perl -0pi`/`printf` của lượt chạy đột biến**, **không** phải cổng yếu — cả hai tái lập thủ công đều đỏ ngay, kèm nguyên văn thông báo assert ghi ở bảng trên. Bài học cho lượt sau: đột biến phải **xác minh bản vá đã áp** *(`grep` chuỗi mới)* trước khi đọc kết quả test, đừng đọc *"xanh"* thành *"cổng yếu"*.

#### 🧾 Trạng thái baseline gặp lại đúng như story cảnh báo

- **`bundle_dmg.sh` chết ở bước AppleScript** — `failed to run bundle_dmg.sh`, đúng bẫy Story 1.1 đã ghi *(`1-1-…md:320`: *"không có phiên Finder tương tác"*)*. Sửa bằng `CI=true` ⇒ Tauri truyền `--skip-jenkins`. **Lệnh đo `.dmg` của story này phải là `CI=true npm run tauri build -- --bundles dmg`**, không phải lệnh trần ghi ở §Lệnh.
- **`cargo fmt --check` đỏ 29 chỗ từ baseline** — **không** chạy `cargo fmt` toàn cây. Chỉ chạy `rustfmt --edition 2024` cho **ba tệp của story**; cả ba `--check` sạch.
- **Không** gặp lỗi `wry requires panic strategy 'abort'`.

### Completion Notes List

#### ① 🔴 Task 0 — chốt **phương án B** *(nguyên hàm **+** `find_terms`)*

Không có chỉ đạo khác của Ice, nên đi khuyến nghị của story. Lý do giữ nguyên như §Quyết định phát biểu: AD-17 nói *"một **component**"* và `epics.md:1509` đòi *"đúng **một** cài đặt khớp ngôn ngữ"* — một vòng khớp lắp ở Story 3.4 **là** một cài đặt thứ hai. Bốn hàm công khai: `tokenize` · `normalize` · `ngrams` · `find_terms`. **Không** nới rộng hơn: không xếp hạng, không ngưỡng %, không chỉ mục ngược, không cache.

Tên hàm và hình dạng kiểu giữ **đúng** §Hình dạng MỤC TIÊU của Dev Notes, không đổi một tên nào.

#### ② 🔴 AC8 — chạy stemmer THẬT: **phỏng đoán của AD-44 ③ ĐÚNG 3/3**

`tantivy_stemmers::algorithms::english_porter_2`, đo 2026-08-05:

| Đầu vào | AD-44 ③ phỏng đoán | **Đo thật** | Trùng? |
|---|---|---|---|
| `dictionary` | `dictionari` | **`dictionari`** | ✅ |
| `study` | `studi` | **`studi`** | ✅ |
| `happy` | `happi` | **`happi`** | ✅ |
| `run` | *(không nêu chuỗi, chỉ nêu "1 hàng")* | **`run`** | ✅ |

⇒ **Trả lời thẳng câu AC8 hỏi: CÓ trùng.** Ba chuỗi mà AD-44 ③ lấy từ hành vi kinh điển của Porter **đúng** là ba chuỗi mà stemmer sản phẩm sinh ra. Kết luận của ③ *(stemming không nằm trên đường nóng tra từ điển)* **không** bị lật — nó **được củng cố**: ba chuỗi đó đúng là thứ sẽ được tra, và chúng đúng là cho **0** hàng trên `dict-core.db`.

⚠️ **KHÔNG sửa `ARCHITECTURE-SPINE.md`** — dev báo cáo số, **Winston cầm bút**. Món nợ ở `:616` đã ghi vào `deferred-work.md` kèm bảng trên.

#### ③ 🔴 AC9 — `Jieba` khởi tạo **179–329 ms bản release**: VƯỢT NFR2 từ 3,6× đến 6,6×

Đo thật, `[profile.release]` không đổi một dòng, 6 lượt, mỗi lượt một tiến trình mới *(macOS darwin 24.6.0)*:

| Lượt | 1 | 2 | 3 | 4 | 5 | 6 |
|---|---:|---:|---:|---:|---:|---:|
| Khởi tạo **lạnh** (ms) | 328,588 | 244,444 | 224,407 | 179,161 | 242,224 | 255,437 |

Trung vị **~243 ms** · thấp nhất **179 ms** · cao nhất **329 ms**. Lượt gọi **ấm** kế tiếp: **1 µs** *(dưới ngưỡng đo)*.

🔴 **NFR2 = 50 ms mỗi frame ⇒ VƯỢT.** Đúng như AC9 dự liệu, story **ghi số và bàn giao**, **không** tự dựng cơ chế hâm nóng — chưa có đường gõ nào tồn tại để hâm nóng vào, và một cơ chế dựng trước người tiêu thụ là một phỏng đoán về chỗ gọi. **Bàn giao có tên: Story 3.4.** Đã ghi vào `deferred-work.md`.

Cái gì tốn: giải nén `dict.txt` *(**5.071.843 byte** thô, nhúng qua `include_flate::flate!`)* + nạp từng dòng vào cây `cedar` — công việc **chạy lúc chạy**, rơi vào **lần gọi đầu tiên**. Hai cổng *(`…constructed_at_exactly_one_place` + `…actually_lazily_initialised_once`)* giữ cho nó không nhân lên.

#### ④ 🔴 AC10 — delta NFR6 **bằng 0 trong sai số phép đo**, và lý do đáng đọc hơn con số

`CI=true npm run tauri build -- --bundles dmg`, `[profile.release]` không đổi một dòng:

| Bản dựng | `.dmg` (byte) | Nhị phân `auratranslate` (byte) |
|---|---:|---:|
| **CÓ** mã story *(lượt 1)* | **23.641.727** | — |
| **KHÔNG** có mã story *(`mod.rs` về lại 7 dòng doc-comment)* | **23.641.710** | **4.538.776** |
| **CÓ** mã story *(lượt 2, cùng nguồn với lượt 1)* | **23.641.719** | **4.538.776** |

🔴 **Nhị phân CÙNG kích thước tuyệt đối ở cả hai cấu hình: 4.538.776 byte.** Và hai lượt dựng có nguồn **giống hệt nhau** cho `.dmg` lệch **8 byte** ⇒ **nền nhiễu của phép đo là ±17 byte**, tức delta 17 byte quan sát được **nằm trọn trong nhiễu**. ⇒ **Delta NFR6 = 0.**

⚠️ **Vì sao — đây mới là phần quan trọng, và nó lật tiền đề của AC10.** AC10 giả định story này là *"lời gọi thật đầu tiên"* nên `DEFAULT_DICT` sẽ **sống dậy** trong bản build. Số đo nói **không**: `lto = true` + `strip = true` vẫn loại sạch cả module, vì **không có một lời gọi nào từ điểm vào của ứng dụng** tới `core::matching` — `core::glossary` và `core::tm` vẫn 0 dòng mã, và không có lệnh IPC nào chạm tới. Một hàm `pub` của thư viện mà nhị phân không gọi thì LTO vẫn cắt.

⇒ **Chi phí NFR6 của jieba là một khoản HOÃN, không phải một khoản đã trả.** Nó rơi vào **story đầu tiên nối Matcher vào một đường thật** *(3.4)*. Cận đo được để story đó không phải đoán:

| Đại lượng | Byte |
|---|---:|
| `dict.txt` thô | **5.071.843** |
| `dict.txt` nén deflate *(`gzip -9`, trừ ~18 B header)* | **≈ 1.904.845** |

Vì `include_flate` nhúng **bản đã nén** và `.dmg` của Tauri là **UDZO/zlib** *(đã nén không nén thêm được)*, cận trên thực tế cho delta `.dmg` tương lai là **≈ 1,9 MB**. **Đây là một CẬN suy từ số đo, không phải delta đã đo** — đừng trích nó như số đo.

**Đối chiếu trần NFR6:** trần **400.000.000** · payload bảy nguồn **343.991.430** · dư địa **56.008.570** — **không đổi ở story này**, và cận 1,9 MB ở trên tiêu **3,4%** dư địa khi nó đến.

#### ⑤ 🔴 AC7 — một phát hiện đo được LẬT một ví dụ của chính AC: `happiest` không về được `happy`

AC7 liệt kê bốn biến thể *(`running` · `dogs` · `studies` · `happiest`)*. Đo thật:

| Biến thể | Stem | Dạng gốc | Stem | Gặp nhau? |
|---|---|---|---|---|
| `running` | `run` | `run` | `run` | ✅ |
| `dogs` | `dog` | `dog` | `dog` | ✅ |
| `studies` | `studi` | `study` | `studi` | ✅ |
| 🔴 `happiest` | **`happiest`** | `happy` | `happi` | **KHÔNG** |

**Porter2 không có luật cho hậu tố so sánh/cực cấp (`-er` · `-est`).** Nên một biến thể **có quy tắc** cũng rơi vào đúng giới hạn mà FR40 tuyên bố cho dạng **bất quy tắc**. Đây **không** phải lỗi cài đặt và không sửa được mà không đổi thuật toán *(mà NFR15 đòi rà giấy phép trước)*.

**Cách story xử lý:** `happiest` chuyển từ ca AC7 sang **ca giới hạn có tên** của AC8 — `stemming_is_not_lemmatization_irregular_and_comparative_forms_never_reach_their_lemma` — đứng chung hàng `went`/`gone`/`children`/`mice`/`better`. ⇒ AC7 **đạt trên 3 biến thể**, và biến thể thứ tư được **ghi lại tường minh là giới hạn** thay vì bị lặng lẽ bỏ.

⚠️ **Hệ quả cần biết cho Epic 3:** một người dịch thêm `happy` vào Glossary sẽ **không** thấy `happiest` tô màu. Nếu mức phủ đó không chấp nhận được thì đường ra là lemmatizer — **quyết định của Ice/John**, đã ghi `deferred-work.md`.

#### ⑥ 🔴 Cờ `HMM = false` — chốt bằng SỐ ĐO, không bằng trực giác

Bản nháp đầu của module chốt `false` với lý do *"HMM có thể cắt ngang tên riêng"*. **Số đo lật một nửa lập luận đó và giữ nguyên kết luận** *(`jieba-rs` 0.10.3, dict mặc định)*:

| Đầu vào | `hmm = false` | `hmm = true` |
|---|---|---|
| `中国人` *(giản thể)* | `中国` · `人` | `中国` · `人` *(không đổi)* |
| `中國人` *(phồn thể)* | `中` · `國` · `人` | **`中國人`** *(một token)* |
| `我喜歡中國人的文化` | 8 token đơn ký tự + `文化` | `我`·`喜歡`·**`中國人`**·`的`·`文化` |
| `萧炎和林动` | `萧`·`炎`·`和`·`林`·`动` | `萧炎`·`和`·`林动` |

**Hàng phồn thể là hàng quyết định**, và nó không phải hàng tôi đoán trước: từ điển mặc định của `jieba-rs` là **giản thể**, nên với một Tác phẩm nguồn viết **phồn thể** *(nguồn Đài/Hồng Kông — và là lý do lược đồ từ điển của dự án mang cả `headword_simp`)*, `hmm = true` gộp gần như mọi thứ thành khối **do HMM bịa**. Thuật ngữ `中國` khi đó rơi vào **giữa** khối `中國人` ⇒ **im lặng không khớp**.

Tính chất tổng quát đứng sau: HMM chỉ **gộp** ký tự mà phép cực đại xác suất đã bỏ rời ⇒ ranh giới của `hmm = true` là **tập con** của `hmm = false` ⇒ gộp thêm nghĩa là **từ chối thêm**. Cộng thêm: đầu ra HMM phụ thuộc **ngữ cảnh xung quanh**, nên cùng một thuật ngữ khớp ở câu này và không khớp ở câu kia — đúng lớp lỗi *"không ai hiểu vì sao"* của AD-17.

⚠️ **Cái giá có tên:** `hmm = false` không phát hiện từ mới nên nhận **rộng hơn** ở vùng ngoài từ điển. Với khớp thuật ngữ đó là **đúng hướng an toàn** — thừa một chỗ tô màu thì người dịch bỏ qua trong một giây; một thuật ngữ **im lặng vắng mặt** thì không ai phát hiện. *(Câu hỏi #4 của §Câu hỏi để lại: giá trị đã chốt là `false`, kèm bảng số trên trong doc-comment của hằng `HMM`.)*

#### ⑦ Luật khớp `Zh` phát biểu lại cho đúng thứ mã LÀM — §Task 5.2

Bản nháp đầu viết *"`中國` nằm trong `中國人` ⇒ **bị từ chối**"*. **Số đo nói không**, và luật thật hẹp hơn: module **không** tự phân xử *"cụm này có phải một từ không"* — nó **nhường** câu hỏi đó cho jieba. Một lượt khớp được nhận ⟺ jieba **đã cắt** đúng ở **cả hai** đầu của nó.

| Văn bản | jieba cắt | Thuật ngữ | Phán quyết |
|---|---|---|---|
| `中国人` *(giản thể)* | `中国` · `人` | `中国` | ✅ **nhận** — jieba tự nói `中国` là một từ ở đây |
| `中國人` *(phồn thể)* | `中` · `國` · `人` | `中國` | ✅ **nhận** — cả hai đầu là ranh giới |
| `…的文化` | `的` · `文化` | `文` | không **từ chối** — đầu cuối rơi **giữa** token |

⇒ Luật này **không** phải *"cấm khớp chuỗi con"*. Nó chặn đúng một thứ: một thuật ngữ **cắt ngang** một từ mà jieba đã nhận diện. Cả ba hàng là ca test sống *(`chinese_term_matching_is_exact_and_arbitrated_by_jieba_token_boundaries`)*, và M10 chứng minh cổng đỏ được.

#### ⑧ Nghiệm thu từng AC

| AC | Kết quả | Bằng chứng |
|---|---|---|
| **AC1** — đúng MỘT cài đặt ở `core/matching/` | ✅ **Đạt** | `only_the_matching_module_ever_names_the_two_language_crates` *(M2a đỏ)* + **đối chứng dương** `the_matching_module_actually_uses_both_language_crates` *(M1 đỏ)* + sàn `MATCHING_FLOOR = 1` & `SRC_RS_FLOOR = 20` *(M7 đỏ)* |
| **AC2** — `glossary`/`tm` dùng được; `dict/` KHÔNG | ✅ **Đạt** | Mọi hàm nhận `&str` + `MatchLang`, trả dữ liệu thuần *(`term_matches_carry_the_callers_own_index_not_a_domain_identifier`)* · `the_dictionary_lookup_path_never_calls_the_matcher` *(M2b đỏ)*, thông báo assert nêu đích danh **AD-17 `:236` + AD-44 ③** · `git status`: **0** dòng dưới `core/dict/**` bị sửa |
| **AC3** — `core/matching/` là **lá** | ✅ **Đạt** | `the_matching_module_is_a_leaf_in_the_dependency_graph` *(M3 đỏ)*; không `use crate::core::*`/`ports`/`commands`/`super::`; không filesystem, không database, không mạng — toàn bộ bề mặt là hàm thuần trên `&str` |
| **AC4** — ngôn ngữ là **tham số** | ✅ **Đạt** | `the_matching_module_never_guesses_the_language_from_the_content` *(M4a đỏ)* + cổng CŨ `exactly_one_definition_of_is_han…` **vẫn sống, không bị nới** *(M4b đỏ)* |
| **AC5** — `Zh`: khớp chính xác + n-gram ký tự + jieba | ✅ **Đạt** | 5 ca `chinese_*`; `HMM` là **hằng module** kèm bảng số *(⑥)*; `chars().count()` *(M8 đỏ 3/3)*; `"中國人"` `n=2` ⇒ `["中國","國人"]`; `n` vượt quần thể ⇒ **rỗng**, không panic |
| **AC6** — `En`: stemming rồi token n-gram | ✅ **Đạt** | Hạ chữ thường **TRƯỚC** *(M9 đỏ)*; `str::to_lowercase` không phụ thuộc locale *(`english_lowercasing_is_locale_independent`)*; `english_porter_2`; token n-gram **sau** stemming *(`the running dogs` ⇒ `["the run","run dog"]`)* |
| **AC7** — biến thể về dạng gốc | ⚠️ **Đạt 3/4 — biến thể thứ tư là GIỚI HẠN ĐO ĐƯỢC, không phải chỗ chưa làm** | `running`/`dogs`/`studies` ✅ · **`happiest` KHÔNG** — xem ⑤. Mọi chuỗi trong test là **đầu ra thật** của `english_porter_2`, không chép từ mô tả kinh điển |
| **AC8** — giới hạn stemming ≠ lemmatization, **đo được** | ✅ **Đạt** | Ca có tên `stemming_is_not_lemmatization_irregular_and_comparative_forms_never_reach_their_lemma`, **6 cặp**; bảng 4 chuỗi AD-44 ③ ở ②; **không** chạm `ARCHITECTURE-SPINE.md` |
| **AC9** — `Jieba` đúng MỘT LẦN, chi phí đo được | ⚠️ **Đạt vế cổng · 🔴 VƯỢT vế NFR2** | `LazyLock` + hai cổng *(M5, M6 đỏ)*; **179–329 ms** so với trần 50 ms ⇒ **bàn giao có tên cho Story 3.4**, không tự dựng hâm nóng — xem ③ |
| **AC10** — delta NFR6 | ✅ **Đạt — delta = 0 trong sai số** | Xem ④. Nhị phân **cùng kích thước tuyệt đối** ở cả hai cấu hình ⇒ LTO cắt sạch vì chưa có người tiêu thụ. Cận cho tương lai: **≈1.904.845 B**. Dư địa **56.008.570** không đổi |
| **AC11** — không crate mới | ✅ **Đạt** | `git status`: `Cargo.toml`/`Cargo.lock` **không xuất hiện** · `check:deps` **XANH**, `RUST_TREE_FLOOR = 200` không đổi · không `unicode-segmentation`/`regex`/`once_cell`/`tantivy` · gọi **thẳng** `english_porter_2`, không qua `StemmerTokenizer` |
| **AC12** — ranh giới KHÔNG CHẠM | ✅ **Đạt** | `git status` 6 mục: 3 tệp mã/test của story + 3 tệp tài liệu. **0** `.vue`/`.ts`/`.css` · **0** `MessageKey`/khoá `vi.json` mới — `check:i18n` vẫn **16 khoá · 9 placeholder** · không `core/{dict,store,scope,i18n}`, không `ports/`, không `commands/` · không `tools/dict-build/**`, **không một lượt `cargo run` nào** của `dict-build` · không `[profile.release]` |
| **AC13** — mọi cổng xanh, không hạ sàn nào | ✅ **Đạt** | `npm run build` → `cargo test --locked` **XANH** · **sáu** cổng `.mjs` **XANH** · không sàn nào bị hạ; `MATCHING_FLOOR` là sàn **THÊM VÀO** |

#### ⑨ Quần thể test — số đo, để lượt sau đối chiếu

| Tệp | Trước | Sau |
|---|---:|---:|
| `dict_lookup.rs` | 37 *(1 `#[ignore]`)* | 37 *(không đổi)* |
| `scope_contract.rs` | 17 | 17 |
| `store_contract.rs` | 16 | 16 |
| `config_invariants.rs` | 15 | 15 |
| `dict_boundary.rs` · `scope_boundary.rs` · `ipc_contract.rs` | 5 · 5 · 5 | không đổi |
| `store_boundary.rs` | 4 | 4 |
| 🆕 `matching_contract.rs` | — | **22** |
| 🆕 `matching_boundary.rs` | — | **8** |
| **Tổng** | **104** *(103 chạy + 1 ignore)* | **134** *(133 chạy + 1 ignore)* |

⚠️ §Trạng thái mã của story ghi *"102 ca"*; số đếm thật trên tám tệp cũ là **104**. Không tệp cũ nào bị đổi — chênh lệch là ở con số ghi trong story, không ở cây test.

🔴 **100% ca của story này chạy được trong CI** — không một `#[ignore]` nào, không một tệp dữ liệu ngoài nào. Đó là lợi thế mà §Quy ước test dặn *"hãy tiêu nó, đừng đánh rơi"*.

#### ⑩ Ba việc bàn giao ra ngoài story — đã ghi `deferred-work.md`, không dev không tự sửa

1. 🔵 **Mermaid AD-13 `:189` còn cạnh `dict --> matching`** — lệch thân Rule AD-17 `:236`. **Winston**.
2. 🟡 **`epics.md:1510` còn vế *"`dict/` dùng nó"*** — **John (PM)**.
3. 🔵 **Bảng phỏng đoán Porter của AD-44 ③ nay có số đo thật** *(đúng 3/3)* — **Winston**.

Cộng ba mục **mới phát sinh từ số đo**: 🔴 `Jieba` init vượt NFR2 *(→ 3.4)* · 🔵 `happiest` không về `happy` *(→ Ice/John)* · 🟡 `find_terms` là O(thuật ngữ × văn bản), chưa đo vì chưa có người tiêu thụ *(→ 3.4/7.5)*.

### File List

**Sửa**

- `src-tauri/src/core/matching/mod.rs` — từ 7 dòng doc-comment / 0 dòng mã ⇒ **514 dòng**: `MatchLang` · `MatchToken` · `TermMatch` · `HMM` · `static JIEBA: LazyLock<Jieba>` · `tokenize` · `normalize` · `ngrams` · `find_terms`
- `_bmad-output/implementation-artifacts/deferred-work.md` — thêm §*Deferred from: 1-12-matcher-dung-chung (2026-08-05)*, 6 mục
- `_bmad-output/implementation-artifacts/sprint-status.yaml` — `1-12-matcher-dung-chung`: `ready-for-dev` → `in-progress` → `review`

**Thêm**

- `src-tauri/tests/matching_contract.rs` — **22** ca hành vi
- `src-tauri/tests/matching_boundary.rs` — **8** cổng tĩnh

**KHÔNG chạm** *(đối chiếu bằng `git status`)*: `src-tauri/Cargo.toml` · `src-tauri/Cargo.lock` · `src-tauri/src/core/dict/**` · `core/mod.rs` · `core/{glossary,tm}/**` · `core/{store,scope,i18n}/**` · `ports/` · `commands/` · `tools/dict-build/**` · mọi tệp `.db` · mọi tệp `.vue`/`.ts`/`.css` · `[profile.release]` · `ARCHITECTURE-SPINE.md` · `epics.md` · `prd.md`
