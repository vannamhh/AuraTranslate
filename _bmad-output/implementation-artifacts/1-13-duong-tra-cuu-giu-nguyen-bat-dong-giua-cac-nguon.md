---
baseline_commit: 7e38de8625c76dfb218fc6b613314123c69e455e
---

# Story 1.13: Đường tra cứu giữ nguyên bất đồng giữa các nguồn

Status: done

## Change Log

| Ngày | Thay đổi |
|---|---|
| 2026-08-05 | Tạo story. Baseline `7e38de8`, cây làm việc **sạch** (Story 1.12 đã commit). |
| 2026-08-05 | Triển khai xong. Ba quyết định đi **B · A · A**. Cổng `DictionarySource` + adapter một-tệp + tập lớp quét thư mục + đường đọc nghĩa theo lô + tầng gom nhóm-theo-`code`. **FR36 nghiệm thu HÀNH VI — món nợ mở từ Story 1.10 ĐÓNG.** 22 ca hành vi mới, 7 cổng ranh giới mới; 163 ca xanh, 0 đỏ. Sáu cổng `.mjs` xanh. 🔴 **NFR1 đường gom: nhánh 2 một ký tự p95 12,569 ms — VƯỢT trần 10 ms; ghi số và bàn giao 1.17, KHÔNG thêm `LIMIT`** (AC13). |
| 2026-08-05 | Code review lượt 1 (phạm vi code sản xuất, chưa gồm test files). 1 decision-needed + 5 patch — Ice chọn sửa cả 6: `branch` nay truyền xuống qua trait thay vì tính lại (`lookup_with_branch`); lỗi quét thư mục hết bị nuốt im lặng; `senses()` khử trùng `entry_ids`; `open_dict_layers` luôn `app.manage()` kể cả khi lỗi; hit mồ côi nay có `eprintln!`; mã trùng `dict_source.code` trong cùng một tệp nay bị từ chối (`SkipReason::DuplicateSourceCodeInFile`). 1 defer. Build + `cargo test --locked` + sáu cổng `.mjs` xanh sau sửa. |
| 2026-08-06 | Code review lượt 2 (`tests/dict_boundary.rs` + `tests/dict_sources.rs`). 3 patch: cổng `no_function_merges_meanings_across_sources` (AC6/AD-19) bị vượt qua bởi biến cách bất quy tắc tiếng Anh (`entries`/`unified`/`coalescing`) — đã vá; cổng `ports_declare_shape_and_never_open_anything` (AC1) bị vượt qua bởi import gộp `use std::{fs, …}` — đã vá; thêm ca hồi quy cho phép khử trùng `entry_ids` xuyên lô. 4 defer (khoảng trống độ phủ test, không chặn). `cargo test --locked` xanh sau sửa. **Status → `done`.** |

**Epic:** 1 — Nền móng ứng dụng & Tra cứu ngoại tuyến tức thì
**Story key:** `1-13-duong-tra-cuu-giu-nguyen-bat-dong-giua-cac-nguon`
**Covers:** FR29 · FR30 · FR31 · FR32 · FR34 *(nửa hiển thị — 1.11b giao nửa đường đi)* · FR35 · **FR36** *(nghiệm thu HÀNH VI — món nợ mở từ Story 1.10)*
**Governed by:** **AD-19** *(chủ)* · **AD-2** *(đúng ba cổng)* · **AD-10** *(một tệp = một lớp)* · **AD-44 ①⑤** · AD-26 · AD-25 · AD-13 · AD-11 · AD-1 · NFR1 · NFR14 · NFR15 · NFR16
**Ngày tạo:** 2026-08-05

---

## 🔴 ĐỌC TRƯỚC TIÊN — MỘT AC CỦA `epics.md` KHÔNG CÓ DỮ LIỆU ĐỂ NGHIỆM THU, VÀ STORY NÀY NÓI THẲNG THAY VÌ ĐÁNH DẤU ĐẠT

`epics.md:1568-1570` viết cho story này:

> **Given** lớp HVTĐTD được bật · **When** tra một mục Hán Việt · **Then** hiển thị từ loại, ví dụ và trích dẫn **bằng tiếng Việt**

**`dict-hvtdtd.db` KHÔNG TỒN TẠI, và nó không tồn tại vì một lý do ở tầng dữ liệu chứ không phải vì ai đó quên:** chưa có **nguồn thô**. `src-tauri/resources/dict/README.md:13` — *"Hai lớp gỡ rời còn lại (HVTĐTD · Cổ hán văn) **chưa tồn tại** — chưa có nguồn thô"*; `prd.md:856` xếp cùng việc đó vào **[A2]** còn mở. Ba tệp có thật hôm nay: `dict-core.db` · `dict-thieu-chuu.db` · `dict-vietphrase.db`.

⇒ **AC11 nghiệm thu mệnh đề đó bằng một FIXTURE mang ĐÚNG hình dạng HVTĐTD** *(`pos_lang = 'vi'`, ví dụ + trích dẫn tiếng Việt)*, không bằng dữ liệu thật. Đó là **thứ nghiệm thu được hôm nay** và nó nghiệm thu đúng thứ đang được hỏi — *đường mã có phân biệt được nhãn tiếng Việt với nhãn ngoại ngữ không*, chứ không phải *nguồn HVTĐTD có tồn tại không*. Vế còn lại — **chạy trên tệp HVTĐTD thật** — là một **bàn giao có tên** cho story dựng lớp đó. **Đừng đánh dấu FR35/FR36 là "đã nghiệm thu trên dữ liệu thật"**; ghi đúng thứ đã đo.

> 🟡 **HAI chỗ tài liệu ĐANG LỆCH — không DEV KHÔNG SỬA. Ghi ra để không ai tưởng mình đọc nhầm.**
> *(a)* `epics.md:1510` *(vế *"`dict/` dùng nó"* của Story 1.12)* và `epics.md:1491` — chủ sở hữu **John (PM)**, đã ghi ở `deferred-work.md`. Story này **kế thừa** phán quyết đó: `core/dict/**` **không** gọi `core/matching/**`, và **đã có cổng** canh *(`matching_boundary.rs::the_dictionary_lookup_path_never_calls_the_matcher`)*. Đừng "nối dict vào matcher" ở đây.
> *(b)* Sơ đồ mermaid AD-13 *(`ARCHITECTURE-SPINE.md:189`)* còn cạnh `dict --> matching` — chủ sở hữu **Winston**, đã ghi ở `deferred-work.md`. **Dev theo THÂN RULE AD-17 `:236` + AD-44 ③.**

---

## Story

As a **người dịch**,
I want **thấy mỗi định nghĩa đến từ đâu và thấy các nguồn nói khác nhau**,
So that **tôi tự phán xét thay vì tin một câu trả lời đã bị gộp lại**.

---

## Ranh giới phạm vi — ĐỌC TRƯỚC KHI GÕ DÒNG ĐẦU TIÊN

Story này là **tầng GOM + cổng `DictionarySource` + đường đọc `dict_sense`/`dict_example`/`dict_citation`**. Nó **không** phải một lượt dựng UI, **không** phải một lượt dựng IPC, **không** phải một lượt sửa chiến lược truy vấn *(1.11/1.11b đã đóng)*, và **không** phải một lượt chạm `tools/dict-build`.

| Thứ | Trong phạm vi? | Chủ sở hữu thật |
|---|---|---|
| `ports/` — trait `DictionarySource` *(cổng thứ nhất trong ba của AD-2)* | ✅ **CÓ** | story này |
| Adapter **một cho mỗi tệp `.db`** + tập lớp phát hiện bằng **quét thư mục** | ✅ **CÓ** | story này |
| Đọc `dict_sense` · `dict_example` · `dict_citation` · `dict_source` · `dict_meta` | ✅ **CÓ** | story này |
| Gom nhiều tệp, **nhóm theo nguồn**, không hợp nhất *(AD-19)* | ✅ **CÓ** | story này |
| Cổng cưỡng chế *"không mã riêng cho từng nguồn"* + *"không hàm hợp nhất"* | ✅ **CÓ** | story này |
| **FR36 hành vi**: xoá một tệp `.db` ⇒ toàn bộ bộ test tra cứu vẫn xanh | ✅ **CÓ** — 🔴 món nợ mở từ 1.10 | story này |
| Đo lại **NFR1 trên đường GOM** *(nhiều tệp + đọc nghĩa)* | ✅ **CÓ** | story này |
| `pick_route` / `pick_branch` / SQL ba nhánh zh + hai nhánh en | **KHÔNG** — dùng lại nguyên | đã xong ở 1.11/1.11b |
| `#[tauri::command]`, `MessageKey` mới, khoá `vi.json` mới | **KHÔNG** | **Story 1.17** *(Panel Lookup)* |
| Bất kỳ dòng `.vue` / `.ts` / `.css` nào | **KHÔNG** | Story 1.14 / 1.17 |
| Bật/tắt từng nguồn, thứ tự hiển thị do người dùng chọn, màn hình ghi công | **KHÔNG** | **Story 1.19** *(FR37, FR38)* · **10.4** |
| Phân trang / `LIMIT` / ngưỡng số kết quả hiện lên | **KHÔNG** — 🔴 xem §Quyết định #1 | **Story 1.17** |
| Gọi `core::matching` từ `core/dict/**` | **KHÔNG** — 🔴 và có **cổng** chặn | phán quyết 1.12, AD-17 `:236` |
| `tools/dict-build/**`, `schema.rs` của nó, một lượt `cargo run` dict-build nào | **KHÔNG** | đã xong ở 1.9/1.10/1.10b |
| `core/store/**` *(gồm `readonly.rs`, `pragmas.rs`)*, `core/scope/**`, `core/i18n/**` | **KHÔNG** — chỉ **dùng** | Story 1.7/1.8 |
| Thêm **bất kỳ** crate nào vào `Cargo.toml` / `Cargo.lock` | **KHÔNG** | — |
| `[profile.release]` | **KHÔNG** *(mọi số đo NFR6 mất so sánh — `deferred-work.md` [D4], đã chốt **bốn** lần)* | — |

🔴 **Story này KHÔNG có người tiêu thụ trên giao diện hôm nay** — đúng như 1.12. Người tiêu thụ là **Story 1.17** *(Panel Lookup)* và **1.18** *(Auto-Lookup)*. Hệ quả phải chấp nhận có ý thức: **hình dạng bản ghi là một hợp đồng suy ra từ FR28–FR32, không phải một hợp đồng đã nghiệm thu bằng mắt người dùng.** Cách story này giảm rủi ro đó:

- Mọi trường của bản ghi **truy ngược được về một FR có số** *(FR28 liệt kê đúng sáu phần: nguồn · từ loại · nghĩa · ví dụ[] · trích dẫn[] · ghi chú)*, không từ trí tưởng tượng.
- **Không** trường nào được thêm mà không có ít nhất một ca test khẳng định nó đi hết đường từ SQLite ra tới bản ghi.

---

## 🔴 BA QUYẾT ĐỊNH PHẢI CHỐT TRONG STORY

### Quyết định #1 — **Đọc nghĩa MỘT PHA hay HAI PHA?** *(🔴 đây là quyết định đắt nhất của story)*

**Dữ kiện, không phải phỏng đoán** *(`deferred-work.md`, đo thật trên `dict-core.db` 194.998.272 byte, 200 lượt, bản release)*:

| Nhánh | Số hàng trả về | p95 **release** | Trần story 1.11 |
|---|---:|---:|---:|
| 2 — `char_idx` **1 ký tự** (`山`) | **3.177** | 🔴 **7,324 ms** | 10 ms |
| 2 — `char_idx` **2 ký tự** (`中國`) | 350 | 1,039 ms | 10 ms |
| 3 — FTS5 trigram (`中國人`) | 33 | 0,448 ms | 1 ms |

Con số 7,324 ms đó là chi phí của **một tệp**, **chưa đọc một hàng `dict_sense` nào**. Story này thêm **hai** thứ lên trên nó: **ba tệp** thay vì một, và **đọc nghĩa + ví dụ + trích dẫn** cho từng đầu mục.

| Phương án | Nội dung | Hệ quả |
|---|---|---|
| **A — một pha** | `lookup_grouped()` trả kết quả **đã đầy đủ nghĩa** | Với `山`: đọc nghĩa cho **3.177** đầu mục × 3 tệp. ⇒ **Chắc chắn vượt** trần 10 ms, và đường ra duy nhất là một `LIMIT` — tức story này **tự quyết một chính sách sản phẩm** mà 1.11 đã tường minh giao cho **1.17** |
| **B — hai pha** ✅ **khuyến nghị** | `lookup_grouped()` trả **nhóm theo nguồn + đầu mục** *(rẻ, đúng số đo đã có)*; `hydrate(&[entry_id])` đọc nghĩa cho **một tập do chỗ gọi chọn** | Hình dạng bản ghi vẫn **đầy đủ** *(AC7–AC11 nghiệm thu trên `hydrate`)*; chính sách *"hiện bao nhiêu"* ở lại **1.17**, đúng chỗ nó thuộc về; NFR1 đo được **cả hai pha riêng** thay vì một con số gộp không quy trách nhiệm được |

**Khuyến nghị B.** **Đừng nới rộng hơn thế:** không cache, không chỉ mục ngược trong bộ nhớ, không xếp hạng, không `LIMIT` — bốn thứ đó thuộc 1.17/1.18 và phụ thuộc hành vi người dùng thật. **Nếu Ice chọn A, đó là một quyết định phạm vi *cộng* một quyết định sản phẩm — ghi cả hai vào Completion Notes, đừng âm thầm giao A rồi đánh dấu AC13 xanh bằng một con số đo trên `中國人`.**

### Quyết định #2 — **18 đầu mục TRÙNG của VietPhrase: gộp lúc đọc, hay để nguyên?**

`deferred-work.md` *(Story 1.10, không vẫn mở, chủ sở hữu ghi đích danh là **1.13**)*: `dict-vietphrase.db` chứa **18** đầu mục trùng *(46 trong nguồn thô)*. Tra `不是他的对手` trả **HAI** `dict_entry` **từ CÙNG một nguồn** ⇒ Panel Lookup của 1.17 hiện **hai khối "VietPhrase" giống hệt nhau**.

🔴 **AD-19 KHÔNG phân xử ca này.** AD-19 cấm hợp nhất **GIỮA các nguồn**; đây là trùng **TRONG một nguồn** — một câu hỏi khác hẳn.

| Phương án | Hệ quả |
|---|---|
| **A — để nguyên** ✅ **khuyến nghị** | Đúng dữ liệu như nguồn ghi; không một dòng mã nào phải phân biệt *"gộp trong nguồn"* với *"gộp giữa nguồn"* — mà chính chỗ phân biệt đó là nơi AD-19 sẽ bị xói mòn. UI hiện hai khối là **thật**, và 1.17 có thể trình bày gọn mà không cần dữ liệu đổi |
| **B — gộp lúc đọc** | Đưa một hàm gộp vào đúng module mà AC6 cấm có hàm gộp; cổng của AC6 sẽ phải mang một ngoại lệ, và một cổng có ngoại lệ thứ nhất sẽ có ngoại lệ thứ hai |
| **C — quyết lại mô hình lúc dựng** | **Ngoài phạm vi** — chạm `tools/dict-build`, dựng lại `dict-vietphrase.db`, điền lại `sha256`, đo lại NFR6 |

**Khuyến nghị A, và ghi con số 18 vào Completion Notes kèm bàn giao tường minh cho 1.17.**

### Quyết định #3 — **Nối tập lớp vào `lib.rs` (`app.manage`) ở story này, hay để 1.17?**

`core/store/readonly.rs:37` viết sẵn: *"`Send + Sync` cùng lý do với `Store` — điều kiện để nó vào `app.manage(…)` ở **Story 1.13**"*, và `:57-60` giao thẳng chính sách **từ chối một tệp mới hơn ứng dụng** cho story này.

| Phương án | Hệ quả |
|---|---|
| **A — nối vào `lib.rs`** ✅ **khuyến nghị** | Mở **một lần lúc khởi động**, đúng khuôn `open_global_store` *(ghi chẩn đoán rồi **đi tiếp**, không chặn khởi động)*; đóng ở `RunEvent::Exit` — 🔴 **bắt buộc theo NFR14**: một tệp còn mở trên Windows là một bản cập nhật không thay được tệp đó, và FR112 *(chính sách gỡ bỏ)* đứng trên đúng khả năng đó. Mở **N pool SQLite** ở phím đầu tiên người dùng gõ là hình dạng chắc chắn vỡ NFR1 |
| **B — để 1.17** | 1.17 vốn đã phải dựng IPC + panel + trạng thái rỗng; nhét thêm vòng đời tài nguyên vào đó là dồn ba việc vào một story |

**Khuyến nghị A.** ⚠️ Kèm theo: `src-tauri/resources/dict/` hôm nay **rỗng** *(không tệp `.db` nào trong git — AD-25)*, và `bundle.resources` chưa mang thư mục đó *(**Story 10.1**)*. Nên đường khởi động phải coi **"không có lớp nào"** là một trạng thái **bình thường có tên**, không phải một lỗi — và đó **chính là** hình dạng FR36 đòi hỏi.

---

## Acceptance Criteria

### AC1 — Cổng `DictionarySource` tồn tại ở `ports/`, và một adapter là **MỘT TỆP `.db`**

**Given** `src-tauri/src/ports/`
**When** rà mã
**Then** tồn tại trait `DictionarySource`, khai ở `ports/` *(AD-2 — cổng thứ nhất trong **đúng ba**)*
**And** **không** trait thứ tư nào được thêm vào `ports/` ở story này
**And** đơn vị mà một adapter bọc là **một tệp `.db`** — **không bao giờ** một **ngôn ngữ** *(AD-44 ⑤: *"Cổng `DictionarySource`: **Cấm** một adapter cho mỗi ngôn ngữ"* — nó phá mệnh đề *"gỡ một lớp = xoá một file"* của AD-10 và làm FR36 không nghiệm thu được bằng test thật nữa)*
**And** `ports/**` **không** gõ `rusqlite`, không `Connection::open`, không chạm filesystem — nó khai **hình dạng**, không mang **cài đặt** *(cổng `store_boundary.rs::only_core_store_may_name_rusqlite` **đã** quét `src/**` và sẽ đỏ; **đừng nới nó**)*

> ⚠️ **Kiểu bản ghi sống ở `core::dict`, không ở `ports/`.** `EntryHit` · `LookupMode` · `QueryRoute` · `LookupResult` **đã** ở đó từ 1.11/1.11b, và một bản sao thứ hai trong `ports/` là hai từ vựng cho một khái niệm. Trait ở `ports/` **tham chiếu** chúng.

### AC2 — Runtime **KHÔNG** có mã riêng cho từng nguồn — cưỡng chế bằng **cổng**

**Given** toàn bộ `src-tauri/src/**`
**When** rà **vị trí mã** *(không tính comment/doc-comment)*
**Then** **không** literal nào bằng một mã nguồn: `cvdict` · `cc-cedict` · `unihan` · `viwiktionary` · `en-wiktionary` · `viwiktionary-en` · `thieu-chuu` · `vietphrase` · `hvtdtd`
**And** **không** literal tên tệp `dict-core.db` / `dict-*.db` ở vị trí mã — tập lớp đến từ **quét thư mục** *(AC3)*, không từ một danh sách viết cứng
**And** cổng có **sàn quần thể** — một đường dẫn gõ sai làm `walk` khớp 0 tệp và cổng xanh mà không kiểm gì cả
**And** **đối chứng dương bắt buộc:** cổng phải chứng minh nó **thật sự bắt được** các chuỗi đó — nghiệm thu bằng một lượt đột biến *(chèn tạm `let _ = "thieu-chuu";` vào `core/dict/`, chạy cổng, thấy **ĐỎ**, hoàn nguyên)*, ghi vào Debug Log References

> 🔴 **Vì sao đây là AC chứ không phải một dòng ghi chú:** AD-10 nói *"Runtime **không có mã riêng cho từng nguồn**"* và `epics.md:1543` lặp lại. Cả hai là văn xuôi. Hình dạng vi phạm rẻ nhất là một `if code == "vietphrase"` để *"sửa cho gọn"* đúng 18 đầu mục trùng ở §Quyết định #2 — và nó sẽ được viết bởi người thật lòng nghĩ mình đang vá một lỗi. `tools/dict-build/src/build.rs:365` đã đặt đúng luật này cho phía dựng *(*"KHÔNG viết thành `if code == "..."`"*)*; story này đặt nó cho phía **đọc**.

### AC3 — Tập lớp phát hiện bằng **QUÉT THƯ MỤC**, không bằng một sổ đăng ký

**Given** một thư mục chứa các tệp `.db`
**When** mở tập lớp
**Then** mọi tệp `*.db` trong thư mục đó được thử mở, **không** một danh sách tên tệp mong đợi nào tồn tại trong mã
**And** danh tính lớp đọc từ **`dict_meta('layer')` của chính tệp** — `"base"` hoặc mã lớp gỡ rời *(`tools/dict-build/src/insert.rs:110-112` viết sẵn cho story này: *"Story 1.13 đọc hàng này để biết mình vừa mở tệp nào TRƯỚC khi đọc `dict_source`"*)*
**And** thứ tự lớp **tất định và là một GIÁ TRỊ quan sát được**: `base` trước, rồi các lớp gỡ rời theo thứ tự **mã lớp** tăng dần — không phụ thuộc thứ tự `read_dir` trả về *(nó khác nhau giữa macOS và Windows — NFR14)*
**And** thư mục không tồn tại, hoặc rỗng ⇒ **tập lớp RỖNG**, không lỗi, không panic

> 🔴 **Vì sao không được có sổ đăng ký:** AD-44 ① — *"**Không tồn tại sổ đăng ký "tệp `.db` nào chứa ngôn ngữ nào"**. Một sổ như thế là nguồn sự thật thứ hai cho một dữ kiện đã nằm trong dữ liệu […] và nó sai **im lặng** vào đúng ngày một lớp gỡ rời được thêm hay gỡ đi (FR112).*" Luật đó áp cho **ngôn ngữ**; story này mở rộng nó sang **danh tính lớp** vì cùng một lý do, và vì FR36 nói *"gỡ một lớp = xoá một file"* — một danh sách tên tệp làm mệnh đề đó thành sai.

### AC4 — Một lớp hỏng, thiếu, hay **quá mới** ⇒ bị bỏ qua **CÓ TÊN**; các lớp còn lại vẫn tra được

**Given** một tệp trong thư mục không mở được, không đúng lược đồ, hoặc mang `PRAGMA user_version` **lớn hơn** phiên bản ứng dụng biết
**When** mở tập lớp
**Then** lớp đó **không được nạp**, và lý do đi vào **giá trị trả về** *(một danh sách lớp bị bỏ qua, mỗi mục mang **đường dẫn + lý do**)* — **không** chỉ là một dòng `eprintln!`
**And** **mọi lớp còn lại vẫn tra được bình thường**
**And** phiên bản đọc từ `PRAGMA user_version`; nếu `dict_meta('schema_version')` **không khớp** nó ⇒ lớp bị **từ chối** với lý do riêng *(hai chỗ ghi phiên bản là chủ ý của Story 1.9 §Quyết định #2; hai chỗ **không nói khác nhau** nghĩa là tệp không do `tools/dict-build` viết ra, và tin nửa nào cũng là đoán)*
**And** chuỗi chẩn đoán viết **KHÔNG DẤU** — `scripts/check-i18n.mjs` Kiểm A quét `src-tauri/**/*.rs` và các tệp story này giao **không** nằm trong danh sách miễn trừ *(cùng bài học `lib.rs:99-100`)*

> 🔴 `core/store/readonly.rs:57-60` giao đúng chính sách này cho story này: *"**Không đọc `PRAGMA user_version`, không di trú, không kiểm phiên bản lược đồ ở đây.** Việc từ chối một tệp mới hơn ứng dụng là quyết định của **tầng gọi (Story 1.13**, nơi biết mình đang mở *lớp* nào và làm gì khi một lớp bị từ chối)"*. **Đừng đẩy phép kiểm này ngược vào `ReadOnlyDb`** — đó là chôn một chính sách vào một cơ chế, và `core/store/**` nằm ngoài phạm vi story.

### AC5 — `pick_route` gọi **ĐÚNG MỘT LẦN** cho mỗi lượt tra, ở tầng gom

**Given** một lượt tra trên N tệp
**When** chạy
**Then** [`pick_route`] được gọi **đúng một lần**, ở tầng gom, và **cùng một** `QueryRoute` đi xuống **mọi** tệp
**And** `branch` là một thuộc tính của **cả lượt tra**, không phải của từng tệp — nó xuất hiện **một lần** trong kết quả gom
**And** **không** một lời gọi `pick_route` nào nằm dưới `core/dict/query.rs` hay trong thân adapter *(cổng `dict_boundary.rs` **đã** canh `query.rs`; đừng nới)*
**And** **không** gọi thẳng `query::char_idx` / `query::exact` / `query::fts_trigram` — 🔴 tầng gom đi qua **`lookup()`**, vì điều kiện `≤ 2 ký tự` của `char_idx()` chỉ là một `debug_assert!` *(vô tác dụng ở bản release; `deferred-work.md` nêu đích danh **"tầng gom Story 1.13"** là ca sẽ cắn)*
**And** 🔴 trạng thái **`QueryBranch::NoBranchQueryTooShort`** *(chuỗi con tiếng Anh < 3 ký tự)* **sống sót qua tầng gom** và xuất hiện trong kết quả — **không** bị dịch thành *"không có kết quả"*. Hai câu đó dẫn người dùng đi hai đường khác nhau *(AD-44 ④; `QueryBranch` doc-comment `core/dict/mod.rs:160-170`)*, và **1.17 đọc đúng trường này** để nói *"truy vấn quá ngắn"*

### AC6 — Kết quả **nhóm theo nguồn**, khoá gom là `code`, và **KHÔNG TỒN TẠI** hàm hợp nhất

**Given** một truy vấn khớp ở nhiều nguồn
**When** trả kết quả
**Then** kết quả **nhóm theo từng nguồn**, mỗi nhóm mang `dict_source.code` **và** `display_name` của chính tệp chứa nó
**And** khoá gom là **`code` (chuỗi)**, **không** `source_id` (số) — 🔴 `id = 1` tồn tại ở **cả ba** tệp và trỏ **ba** nguồn khác nhau
**And** **không** dựng lại một bảng tra `id → nguồn` ở tầng gom *(`deferred-work.md`, mục *"Khoá theo `code` chứ không theo `id`"*, nửa còn lại giao đích danh cho story này)*
**And** hai nguồn **bất đồng** về cùng một đầu mục ⇒ **cả hai nhóm có mặt**, nghĩa giữ nguyên, không có nhóm nào bị chọn làm *"câu trả lời"* **(FR32)**
**And** trong toàn `src-tauri/src/**` **không tồn tại** một hàm hợp nhất nghĩa giữa các nguồn — cưỡng chế bằng cổng tĩnh, thông báo assert nêu đích danh **AD-19** và **FR31/FR32**
**And** **hai lớp khai cùng một `code`** ⇒ đó là **lỗi dữ liệu có tên** *(một mục trong danh sách lớp bị bỏ qua của AC4)*, **không** im lặng gộp hai tệp vào một nhóm
**And** một nguồn **đã tra mà không khớp gì** ⇒ **không sinh nhóm rỗng**; nó phân biệt được với *"lớp không nạp được"* qua danh sách `skipped` của AC4 — 🔴 khai mệnh đề này ra thay vì để 1.17 đoán, vì hai trạng thái đó **không** được phép trông giống nhau

> ⚠️ **Khoá gom đến MIỄN PHÍ, đừng dựng lại nó.** `EntryHit` của 1.11 **đã** mang `source_code: String` và **không có** trường `source_id` nào — `query.rs:50` chọn `s.code` qua `JOIN dict_source s ON s.id = e.source_id` ở **cả năm** nhánh, chính xác vì story này. `tests/dict_lookup.rs::results_carry_the_source_code_not_the_id` đang khoá mệnh đề đó.

### AC7 — Một từ nhiều **từ loại** ⇒ **nhiều mục riêng biệt**, mỗi mục ví dụ riêng *(FR29)*

**Given** một đầu mục có nhiều hàng `dict_sense`
**When** trả kết quả
**Then** mỗi hàng là **một mục riêng**, **không** nối `gloss` thành một chuỗi
**And** thứ tự là `ord` tăng dần, **và có khoá phụ `id`** — 🔴 **không** `ORDER BY ord` trần: `tools/dict-build/src/sources/vietphrase.rs` tách `/` vô điều kiện và sinh nhiều `dict_sense` **cùng `ord`** *(`deferred-work.md`, Story 1.10)*, nên thiếu khoá phụ là hai lượt chạy cho hai thứ tự và một ca test **flaky sẽ bị gỡ trong tháng**
**And** `note` đi cùng mục *(FR28 liệt kê **sáu** phần: nguồn · từ loại · nghĩa · ví dụ[] · trích dẫn[] · **ghi chú**)*

### AC8 — Ví dụ gắn theo **TỪ LOẠI**; trích dẫn là **trường RIÊNG** có xuất xứ *(FR30)*

**Given** một mục nghĩa
**When** trả kết quả
**Then** ví dụ treo vào **`sense_id`**, **không** vào `entry_id` — 🔴 lược đồ đã cưỡng chế điều này *(`dict_example.sense_id REFERENCES dict_sense(id)`)*, và đọc bằng một `JOIN` qua `entry_id` là tự đánh mất nó
**And** **trích dẫn** là một danh sách **riêng biệt** với ví dụ, mang `work` và `author` *(xuất xứ văn bản)* — **không** trộn hai bảng vào một danh sách
**And** ví dụ mang `translation` + **`translation_lang`** — trường thứ hai không bỏ được: nó là thứ AC10 dùng để nói *"bản dịch ví dụ này là tiếng Anh"*
**And** cả hai danh sách sắp theo `ord` **cộng khoá phụ `id`** *(cùng lý do AC7)*

### AC9 — Mục từ **tiếng Anh**: nhãn từ loại + nghĩa tiếng Việt *(FR34)*

**Given** một đầu mục `lang = 'en'`
**When** trả kết quả
**Then** mỗi mục nghĩa mang **`pos`** *(nhãn từ loại)* và **`gloss`** *(nghĩa tiếng Việt)*
**And** đi qua **cùng** đường gom và **cùng** hình dạng bản ghi với mục tiếng Trung — 🔴 AD-44 ⑤: *"`lang` là một **trường**, không phải một **kiểu** — không tồn tại bản ghi kết quả thứ hai dành riêng cho tiếng Anh"*
**And** **không** bước hợp nhất kết quả `zh` với `en` ở bất kỳ đâu *(AD-44 ⑤, cùng luật AD-19)*

### AC10 — Mục từ **tiếng Trung** trên lớp nền: từ loại + ít nhất một ví dụ khi nguồn có; nhãn ngoại ngữ **đánh dấu bằng TRƯỜNG** *(FR35)*

**Given** một đầu mục `lang = 'zh'` khi **chỉ có các lớp nền**
**When** trả kết quả
**Then** mục mang nhãn từ loại, và **ít nhất một ví dụ khi nguồn có dữ liệu**
**And** 🔴 nhãn ngoại ngữ được nhận ra qua **`dict_sense.pos_lang`** — một **TRƯỜNG**, **không** đoán từ nội dung `pos`, không một bảng tra `"noun" ⇒ tiếng Anh` nào
**And** **không** dịch, không viết lại, không ẩn nhãn ngoại ngữ ở tầng này — 1.17 **hiển thị** dấu hiệu đó; story này chỉ phải làm cho nó **không mất trên đường đi**

> 🔴 `tools/dict-build/src/schema.rs:57-58` viết sẵn lý do trường này tồn tại: *"`pos_lang` tồn tại vì **FR35** — nhãn từ loại ngoại ngữ phải được **ĐÁNH DẤU RÕ**, không đoán được từ nội dung `pos`"*. Bỏ trường này khỏi bản ghi là làm FR35 không nghiệm thu được ở 1.17, và lỗi lộ ra ở **story sau**.

### AC11 — Lớp kiểu **HVTĐTD**: từ loại · ví dụ · trích dẫn **tiếng Việt**, và **rơi về nhãn tiếng Anh** khi gỡ

**Given** một lớp gỡ rời mang `pos_lang = 'vi'`, ví dụ và trích dẫn tiếng Việt — 🔴 **fixture**, xem §ĐỌC TRƯỚC TIÊN
**When** tra một mục Hán Việt có mặt ở **cả** lớp đó **và** lớp nền
**Then** nhóm của lớp đó mang từ loại, ví dụ **và** trích dẫn tiếng Việt
**And** nhóm của lớp nền vẫn có mặt **cạnh nó** với nhãn ngoại ngữ *(AC6 — không nhóm nào bị chọn làm câu trả lời)*
**And** **xoá tệp `.db` của lớp đó** ⇒ cùng truy vấn trả về **đúng** nhóm lớp nền, mang nhãn tiếng Anh — *"rơi về nhãn tiếng Anh của lớp nền, không có đường tra cứu nào hỏng"* *(`epics.md:1575`)*
**And** Completion Notes ghi thẳng: mệnh đề này nghiệm thu **trên fixture**, và vế **dữ liệu HVTĐTD thật** là một bàn giao có tên

### AC12 — 🔴 **FR36 HÀNH VI**: xoá tệp `.db` của một lớp gỡ rời **bất kỳ** ⇒ **toàn bộ** bộ test tra cứu vẫn **XANH**

**Given** một tập lớp gồm base + ít nhất **hai** lớp gỡ rời *(fixture)*
**When** xoá tệp `.db` của **một lớp gỡ rời bất kỳ** rồi chạy lại **toàn bộ** bộ test tra cứu
**Then** **tất cả vẫn xanh** — không một ca nào cần sửa, không một nhánh `#[cfg]` nào
**And** phép thử này chạy **cho từng lớp gỡ rời**, không chỉ cho một lớp được chọn sẵn — *"một lớp gỡ rời **bất kỳ**"* là mệnh đề của `epics.md:1572`, và nó không nghiệm thu được bằng một ca
**And** **không** một ca test nào ngoài nhóm này được viết dựa trên giả định *"lớp X có mặt"*
**And** Completion Notes ghi rằng đây là món nợ FR36 mở từ Story 1.10 và nó **đóng ở đây** *(`deferred-work.md`: *"Không đánh dấu FR36 là 'đã nghiệm thu' cho tới khi 1.13 viết phép thử này"*)*

> 🔴 **Ca dễ trượt nhất, và nó trượt XANH:** một bộ test dựng fixture rồi **luôn** mở đủ ba tệp sẽ *"đạt"* AC này mà chưa bao giờ chạy đường thiếu tệp. Phép thử phải **xoá tệp thật** rồi **mở lại tập lớp** — và trên **Windows**, xoá một tệp còn mở là một lỗi *(NFR14)*, nên `ReadOnlyDb` phải được **drop trước** khi xoá. Đó là luật 2 của `dict_lookup.rs`, và nó là lý do luật đó tồn tại.

### AC13 — **NFR1 đo lại TRÊN đường gom**, không suy ra từ số một tệp

**Given** ba tệp `.db` thật *(`tools/dict-build/out/`: `dict-core.db` 194.998.272 · `dict-thieu-chuu.db` 5.787.648 · `dict-vietphrase.db` 160.083.968 byte)*
**When** đo p95 **bản release**, đúng khuôn `bench_three_branches_on_the_real_dictionary` *(`#[ignore]` + biến môi trường; **không** ngưỡng thời gian trong CI)*
**Then** ghi số thật cho **cả hai pha** *(nếu §Quyết định #1 chọn B)* trên **cả ba nhánh zh** và **hai nhánh en**, gồm ca xấu nhất đã biết `山` *(3.177 hàng × 3 tệp)*
**And** đối chiếu trần **10 ms** của backend và ngân sách NFR1 **< 100 ms** đầu-cuối
**And** 🔴 **vượt trần ⇒ GHI SỐ VÀ BÀN GIAO, KHÔNG tự thêm `LIMIT`** — `deferred-work.md` chốt: *"đường ra là một **quyết định sản phẩm** […] nó chạm hợp đồng của Panel Lookup **(1.17)**"*, và Ice **đã** chọn *"chấp nhận nguyên trạng"* một lần cho cùng câu hỏi ở 1.11
**And** **không** đọc nghĩa bằng **một truy vấn cho mỗi đầu mục** *(N+1)*: một tập `entry_id` đi vào **một** câu SQL theo lô, kích thước lô là **hằng có tên kèm lý do** — 🔴 lô cỡ cố định giữ **đúng một** hình dạng SQL nên `prepare_cached` còn tác dụng; lô co giãn sinh một câu mới cho mỗi kích thước và **vô hiệu hoá cache câu lệnh** trên chính đường nóng
**And** **không** `format!` id vào câu SQL — tham số ràng buộc, đúng luật `query.rs`

### AC14 — Ranh giới KHÔNG CHẠM, và mọi cổng xanh

**Given** danh sách tệp story này giao
**When** đối chiếu bằng `git status`
**Then** **không** một dòng `.vue` / `.ts` / `.css` nào
**And** **không** `#[tauri::command]` mới, không `MessageKey` mới, không khoá `vi.json` mới — `npm run check:i18n` vẫn báo đúng **16 khoá · 9 placeholder**
**And** **không** chạm `tools/dict-build/**`, và **không** một lượt `cargo run` nào của `dict-build` *(🔴 mặc định là `--layer all`, nó **dựng lại cả ba** tệp `.db` ⇒ ba `sha256` trong `dict-manifest.toml` thành sai, và **không cổng nào bắt được** — `check-dict-manifest.mjs` cố ý không đọc `.db`)*
**And** **không** chạm `core/store/**`, `core/scope/**`, `core/i18n/**`, `core/matching/**`, `[profile.release]`, `Cargo.toml`, `Cargo.lock`
**And** `npm run build` → `cargo test --locked --manifest-path src-tauri/Cargo.toml` **xanh**, và **mọi ca cũ giữ nguyên xanh**
**And** sáu cổng `.mjs` **xanh**: `check:deps` · `check:i18n` · `check:commands` · `check:tokens` · `check:dict` · `check:dict-manifest`
**And** **không** hạ / nới một sàn quần thể nào: `dict_boundary::DICT_FLOOR = 1` · `dict_boundary::SRC_TAURI_RS_FLOOR = 20` · `matching_boundary::MATCHING_FLOOR = 1` / `SRC_RS_FLOOR = 20` · `store_boundary::RS_FLOOR = 20` · `scope_boundary::RS_FLOOR = 20` · `check-i18n::RS_FLOOR = 21` / `VUE_FLOOR = 1` · `check-deps::RUST_TREE_FLOOR = 200`
**And** **không** nới `FORBIDDEN` của `dict_boundary.rs` *(`LIKE` · `GLOB` · `instr(`)* — 🔴 luật đó áp cho **mọi** SQL mới của story này, gồm cả câu đọc nghĩa
**And** **không** nới `FORBIDDEN`/`STORE_DIR` của `store_boundary.rs`, không nới cổng của `matching_boundary.rs`

---

## Tasks / Subtasks

- [x] **Task 0 — Chốt ba quyết định** *(§Quyết định #1, #2, #3)*
  - [x] 0.1 Đọc §Quyết định; không có chỉ đạo khác của Ice ⇒ đi **B · A · A**
  - [x] 0.2 Ghi ba lựa chọn + lý do vào Completion Notes **trước** khi viết dòng mã đầu tiên

- [x] **Task 1 — Cổng `DictionarySource`** *(AC1)*
  - [x] 1.1 `ports/dict_source.rs`: trait `DictionarySource`, doc-comment nêu AD-2 *(đúng ba cổng)*, AD-10 *(một adapter = một **tệp**)*, AD-44 ⑤ *(không bao giờ một adapter cho mỗi **ngôn ngữ**)*
  - [x] 1.2 Trait **tham chiếu** kiểu của `core::dict`, không dựng bản sao thứ hai
  - [x] 1.3 Cập nhật doc-comment `ports/mod.rs` — nó viết *"mỗi cổng do story sở hữu năng lực tương ứng dựng"*; nay cổng thứ nhất có chủ
  - [x] 1.4 **KHÔNG** gõ `rusqlite` / `Connection::open` ở `ports/**`

- [x] **Task 2 — Adapter một tệp và tập lớp** *(AC3, AC4, AC5)*
  - [x] 2.1 `core/dict/layer.rs` *(hoặc tên tương đương)*: adapter bọc **một** `ReadOnlyDb`, đọc `dict_meta('layer')` + `PRAGMA user_version` + `dict_meta('schema_version')` **lúc mở** *(⚠️ `ReadOnlyDb::open` mang `debug_assert_eq!(kind, StoreKind::Dict)` — truyền `StoreKind::Dict`, không loại khác)*
  - [x] 2.2 Tập lớp: quét `*.db` trong một thư mục **nhận từ chỗ gọi** *(không tự phân giải `$RESOURCE` — đường đó sống ở `lib.rs`, đúng khuôn `$APPDATA` của `Store`)*
  - [x] 2.3 Thứ tự tất định: `base` trước, rồi mã lớp tăng dần; **không** tin thứ tự `read_dir`
  - [x] 2.4 Lớp bị bỏ qua ⇒ **giá trị** mang đường dẫn + lý do; chuỗi chẩn đoán **KHÔNG DẤU**
  - [x] 2.5 Đọc `dict_source` của tệp *(`code` + `display_name`)*; hai lớp cùng `code` ⇒ lớp thứ hai vào danh sách bị bỏ qua *(AC6)*
  - [x] 2.6 Ca test: thư mục không tồn tại · rỗng · chứa một tệp không phải SQLite · chứa một tệp `user_version` cao hơn ⇒ **không panic**, các lớp còn lại vẫn tra được

- [x] **Task 3 — Đường đọc nghĩa · ví dụ · trích dẫn** *(AC7, AC8, AC13)*
  - [x] 3.1 `core/dict/senses.rs` *(hoặc trong `query.rs` — cùng luật không `LIKE`/`GLOB`/`instr(`)*: **một** truy vấn theo **lô** `entry_id`, lô cỡ **hằng có tên**
  - [x] 3.2 `ORDER BY ord, id` ở **cả ba** bảng — 🔴 không `ORDER BY ord` trần *(AC7)*
  - [x] 3.3 `dict_example` và `dict_citation` treo vào **`sense_id`**, không `entry_id`
  - [x] 3.4 Giữ đủ trường FR28: `pos` · `pos_lang` · `gloss` · `note` · `ord`; ví dụ giữ `translation` + `translation_lang`; trích dẫn giữ `work` + `author`
  - [x] 3.5 **Không** `format!` id vào SQL; không một truy vấn cho mỗi đầu mục
  - [x] 3.6 Ca test: đầu mục không có nghĩa nào ⇒ danh sách rỗng, không lỗi; nghĩa không có ví dụ ⇒ danh sách rỗng

- [x] **Task 4 — Tầng gom** *(AC5, AC6, AC9, AC10)*
  - [x] 4.1 `pick_route` **một lần**; `branch` **một lần**; truyền cùng giá trị xuống mọi lớp
  - [x] 4.2 Đi qua `lookup()`, **không** gọi thẳng `query::*` *(AC5 — bẫy `debug_assert!` của `char_idx`)*
  - [x] 4.3 Nhóm theo `code`; hai nguồn bất đồng ⇒ hai nhóm; **không** hàm gộp
  - [x] 4.4 Ca test **bất đồng**: hai nguồn, cùng đầu mục, `gloss` **mâu thuẫn** ⇒ **cả hai** có mặt, không cái nào biến mất
  - [x] 4.5 Ca test `lang='en'` đi **cùng** hình dạng bản ghi với `lang='zh'` *(AD-44 ⑤)*
  - [x] 4.6 Ca test `pos_lang` sống sót nguyên vẹn từ SQLite ra bản ghi *(AC10)*

- [x] **Task 5 — Fixture nhiều tệp** *(AC11, AC12)*
  - [x] 5.1 Dùng lại khuôn `build_fixture` của `dict_lookup.rs` — 🔴 **và cổng parity DDL đi theo**: DDL chép nguyên văn từ `tools/dict-build/src/schema.rs`, gồm **`DICT_CITATION_DDL`** *(tệp test hiện có đã chép nó nhưng chưa ca nào dùng)*
  - [x] 5.2 Ba tệp: `base` *(nhãn ngoại ngữ, `pos_lang='en'`)* + hai lớp gỡ rời, một trong đó mang hình dạng **HVTĐTD** *(`pos_lang='vi'`, ví dụ + trích dẫn tiếng Việt)*
  - [x] 5.3 🔴 `entry_fts` là external-content ⇒ `INSERT INTO entry_fts(entry_fts) VALUES('rebuild');` **cho từng tệp** — thiếu dòng này nhánh 3 trả rỗng và mọi ca của nó *"xanh"* theo đúng cách sai nhất
  - [x] 5.4 Mỗi ca một thư mục tạm riêng *(pid + bộ đếm nguyên tử)*; không `tempfile`
  - [x] 5.5 **Drop `ReadOnlyDb` TRƯỚC khi xoá tệp** — NFR14, và là điều kiện để AC12 chạy được trên Windows

- [x] **Task 6 — Nghiệm thu FR36** *(AC12)*
  - [x] 6.1 Vòng lặp **trên từng lớp gỡ rời**: xoá tệp → mở lại tập lớp → chạy lại các mệnh đề tra cứu
  - [x] 6.2 Ca *"rơi về nhãn tiếng Anh"* của AC11 nằm trong cùng vòng
  - [x] 6.3 Đối chứng dương: **trước** khi xoá, lớp đó **thật sự** đóng góp một nhóm — không có nó thì *"xoá xong vẫn xanh"* và *"lớp đó chưa bao giờ được nạp"* đọc giống hệt nhau

- [x] **Task 7 — Cổng ranh giới** *(AC1, AC2, AC6)*
  - [x] 7.1 Thêm vào `tests/dict_boundary.rs` *(đừng chế tệp cổng mới — cùng vùng mã, cùng chủ đề)*
  - [x] 7.2 Cổng: không literal mã nguồn / tên tệp `.db` ở vị trí mã dưới `src-tauri/src/**` *(AC2)*, kèm **sàn quần thể** và **đối chứng dương bằng đột biến**
  - [x] 7.3 Cổng: không hàm hợp nhất nghĩa giữa các nguồn; thông báo assert nêu **AD-19 + FR31/FR32** *(AC6)*
  - [x] 7.4 Cổng: `ports/**` không `rusqlite` / `Connection::open` *(AC1)*
  - [x] 7.5 Dùng lại khuôn đã có, **đừng chế khuôn mới**: `walk` · `rel_posix` *(🔴 chuẩn hoá `\` → `/` — bài học NFR14)* · `contains_forbidden_token` *(so khớp không phân biệt hoa/thường)*

- [x] **Task 8 — Nối vào `lib.rs`** *(AC3, AC4 — **chỉ khi Task 0 chọn A ở §Quyết định #3**)*
  - [x] 8.1 Mở tập lớp ở `setup()`, đúng khuôn `open_global_store`: **ghi chẩn đoán rồi ĐI TIẾP**, không chặn khởi động *(một `setup()` trả `Err` làm **hai cổng** `check:scope` / `check:scope:bundled` đỏ vì tầng dữ liệu, không vì phạm vi chúng canh)*
  - [x] 8.2 Thư mục lấy qua `app.path().resource_dir()` — **không** ghép chuỗi bằng tay *(NFR14 hỏng đầu tiên ở đúng chỗ đó)*
  - [x] 8.3 `app.manage(…)` tập lớp; đóng ở `RunEvent::Exit` cạnh `close_global_store` — 🔴 NFR14 + FR112
  - [x] 8.4 **Không** thêm mục ACL vào `capabilities/main.json` *(`config_invariants.rs` khoá tệp đó ở đúng ba quyền)*
  - [x] 8.5 Mọi chuỗi chẩn đoán **KHÔNG DẤU** *(`check:i18n` Kiểm A)*

- [x] **Task 9 — Đo và bàn giao** *(AC13, và các mục còn mở)*
  - [x] 9.1 Bench `#[ignore]` + biến môi trường *(khuôn `AURA_DICT_BENCH_DB`; đường gom cần một **thư mục** — đặt tên biến mới, ghi vào doc-comment cách chạy)*
  - [x] 9.2 Đo **bản release** trên ba tệp thật; ghi bảng vào Completion Notes; đối chiếu trần 10 ms
  - [x] 9.3 Vượt trần ⇒ **ghi số + nêu nguyên nhân + bàn giao 1.17**, **không** thêm `LIMIT`
  - [x] 9.4 Thêm vào `deferred-work.md`: *(a)* trạng thái FR36 sau story này; *(b)* phán quyết §Quyết định #2 *(18 đầu mục trùng)* + bàn giao 1.17; *(c)* vế **HVTĐTD dữ liệu thật** còn mở; *(d)* số NFR1 mới của đường gom
  - [x] 9.5 **KHÔNG** sửa `ARCHITECTURE-SPINE.md`, `prd.md`, `epics.md` — dev **báo cáo số**, **Winston/John** cầm bút

- [x] **Task 10 — Cổng cuối** *(AC14)*
  - [x] 10.1 `npm run build` *(**BẮT BUỘC TRƯỚC** `cargo test` — `generate_context!` nhúng `dist/` lúc biên dịch)*
  - [x] 10.2 `cargo test --locked --manifest-path src-tauri/Cargo.toml`
  - [x] 10.3 Sáu cổng `.mjs`
  - [x] 10.4 `git status` đối chiếu §AC14; điền **File List** và **Completion Notes**

### Review Findings

*(Code review 2026-08-05, phạm vi: `ports/dict_source.rs` · `core/dict/layer.rs` · `core/dict/senses.rs` · `core/dict/mod.rs` · `lib.rs` · `ports/mod.rs` — chưa gồm `tests/dict_boundary.rs`/`tests/dict_sources.rs`, để dành lượt review sau)*

- [x] [Review][Decision→Patch] `branch` bị tính **hai lần** thay vì truyền xuống, dựa vào một `debug_assert_eq!` vô tác dụng ở release — Task 4.1 nói *"branch một lần; truyền cùng giá trị xuống mọi lớp"*. **Ice chọn (b): sửa.** `DictionarySource::lookup` đổi chữ ký nhận `branch: QueryBranch` thay cho `mode: LookupMode`; `super::lookup` (4 tham số, dùng bởi `tests/dict_lookup.rs` thuộc 1.11/1.11b) giữ nguyên, chỉ ủy quyền sang hàm nội bộ mới `lookup_with_branch(db, query, route, branch)`; `DictLayer::lookup` gọi hàm mới đó; `lookup_grouped` tính `branch` đúng một lần rồi truyền xuống `layer.lookup(query, route, branch)`, bỏ hẳn `debug_assert_eq!`. Không đụng `tests/dict_lookup.rs`. `cargo test` toàn bộ xanh sau khi sửa (163+ ca, gồm `dict_sources.rs::the_route_and_the_branch_are_one_value_of_the_whole_lookup`).
- [x] [Review][Patch] Lỗi quét thư mục bị nuốt im lặng, giống hệt trạng thái "chưa có từ điển nào" [src-tauri/src/core/dict/layer.rs:349] — đã sửa: `NotFound` vẫn im lặng (bình thường, AC3), mọi lỗi `read_dir`/`DirEntry` khác nay ghi `eprintln!` chẩn đoán, vẫn trả tập lớp rỗng, không panic.
- [x] [Review][Patch] `senses()`/`read_senses` không phòng `entry_ids` trùng giữa hai lô — sinh bản ghi lặp với ví dụ/trích dẫn rỗng [src-tauri/src/core/dict/senses.rs:110] — đã sửa: khử trùng `entry_ids` bằng `HashSet` trước khi chia lô.
- [x] [Review][Patch] `open_dict_layers` không `app.manage(...)` khi `resource_dir()` lỗi — để `DictLayers` hoàn toàn chưa quản lý thay vì một tập rỗng có tên [src-tauri/src/lib.rs:188] — đã sửa: thêm `DictLayers::empty()`, `app.manage(...)` nó trên cả nhánh lỗi.
- [x] [Review][Patch] Hit "mồ côi" (source_code không có trong `dict_source` của chính tệp) bị `debug_assert!(false, ...)` rồi drop — vô tác dụng và im lặng tuyệt đối ở bản release [src-tauri/src/core/dict/mod.rs:548] — đã sửa: giữ `debug_assert!` cho lượt phát triển, thêm `eprintln!` luôn chạy ở mọi profile.
- [x] [Review][Patch] Hai hàng cùng `dict_source.code` **trong cùng một tệp** không bị phát hiện — `source()` trả hàng đầu, hàng thứ hai bị giấu im lặng; đối xứng còn thiếu với `SkipReason::DuplicateSourceCode` (vốn đã bắt ca trùng **giữa** các lớp) [src-tauri/src/core/dict/layer.rs:279] — đã sửa: thêm `SkipReason::DuplicateSourceCodeInFile`, `DictLayer::open` từ chối tệp nếu `dict_source` (đã `ORDER BY code`) mang hai hàng liền kề cùng mã.

*(Sau khi áp patch: `npm run build` ✅ · `cargo test --locked --manifest-path src-tauri/Cargo.toml` ✅ toàn bộ xanh, không hồi quy · sáu cổng `.mjs` ✅.)*
- [x] [Review][Defer] `SchemaTooNew` chỉ từ chối tệp **mới hơn** `SUPPORTED_SCHEMA_VERSION`; một tệp **cũ hơn** (khi ứng dụng nâng version sau này) sẽ lọt qua cổng phiên bản hôm nay — deferred, pre-existing shape of AC4 (spec chỉ đòi hỏi bắt ca "quá mới"; chưa có tệp version 0 nào tồn tại) [src-tauri/src/core/dict/layer.rs:229]

#### Lượt 2 — `tests/dict_boundary.rs` + `tests/dict_sources.rs` (2026-08-06)

- [x] [Review][Patch] Cổng `no_function_merges_meanings_across_sources` (AC6, canh AD-19 — bất biến kiến trúc trung tâm của story) bị vượt qua bởi biến cách tiếng Anh bất quy tắc: `"entries"` không chứa `"entry"`, `"unified"`/`"unifies"` không chứa `"unify"`, `"coalescing"` không chứa `"coalesce"` — `fn merge_entries(...)`/`fn unified_glosses(...)` sẽ lọt qua cổng mà không ai biết [src-tauri/tests/dict_boundary.rs:548] — đã sửa: thêm các dạng biến cách vào `MERGE_VERBS`/`MERGE_NOUNS`, thêm ba đối chứng dương khoá lại lỗ hổng.
- [x] [Review][Patch] Cổng `ports_declare_shape_and_never_open_anything` (AC1) — needle `"std::fs"` bị vượt qua bởi import gộp hoàn toàn bình thường `use std::{fs, path::Path};`; đồng thời không nhất quán rigor với các cổng láng giềng (dùng `code.contains` trần thay vì `contains_forbidden_token` có biên từ) [src-tauri/tests/dict_boundary.rs:676] — đã sửa: đổi needle thành `"fs::"` (bắt được mọi cách `use`, kể cả đường đủ `std::fs::…`), đổi vòng quét dùng `contains_forbidden_token` cho cả sáu needle.
- [x] [Review][Patch] Hành vi khử trùng `entry_ids` xuyên lô của `senses()` (đã sửa ở Lượt 1) chưa có ca test riêng chứng minh — `reading_senses_across_many_batches_never_duplicates_or_drops_a_row` chỉ dùng id duy nhất, không thật sự chạm đường dedup [src-tauri/tests/dict_sources.rs:1227] — đã thêm `a_duplicate_entry_id_spanning_two_batches_is_not_double_counted`.

*(Sau khi áp patch: `cargo test --locked --manifest-path src-tauri/Cargo.toml` ✅ toàn bộ xanh, không hồi quy.)*

- [x] [Review][Defer] `ordering_lacks_a_tiebreaker` (AC7 gate) quét theo **từng dòng vật lý**; một câu `ORDER BY` bị tách dòng sẽ lọt qua cổng hoàn toàn (không đỏ, không xanh — chỉ im lặng không xét) — deferred: rủi ro thật thấp (toàn bộ SQL hiện tại trong `core/dict/**` viết một dòng, đúng quy ước đã có), và một bản sửa an toàn cần phân tích ranh giới chuỗi ký tự Rust thật sự thay vì ghép cặp dòng kề nhau (thử nghiệm cho thấy ghép cặp dòng kề nhau có rủi ro làm mất phát hiện ở ca một-dòng hiện tại nếu dòng kế cận tình cờ chứa token `id`) [src-tauri/tests/dict_boundary.rs:807].
- [x] [Review][Defer] `mentions_a_dict_db_file` (AC2 gate thứ hai) đòi literal `"dict-"` (có gạch nối); một tên tệp viết cứng không mang tiền tố đó (vd. `"hvtdtd.db"`) sẽ lọt qua — deferred: mọi tên tệp thật trong `dict-manifest.toml` hôm nay đều theo quy ước `dict-*.db`, nên khoảng trống này chưa có rủi ro thật; xem lại nếu quy ước đặt tên đổi [src-tauri/tests/dict_boundary.rs:540].
- [x] [Review][Defer] Sáu trong mười biến thể `SkipReason` (`OpenFailed`, hai khoá của `MetaRowMissing`, `SourcesUnreadable`, `DuplicateLayer`, `LookupFailed`) chưa có ca hành vi nào gọi tên chúng trong `dict_sources.rs` — chỉ `MetaUnreadable`/`SchemaTooNew`/`SchemaVersionDisagrees`/`DuplicateSourceCode` được test trực tiếp — deferred: mỗi ca cần dựng fixture hỏng riêng (tệp không mở được, `dict_meta` thiếu hàng, `dict_source` không đọc nổi, hai tệp cùng khai một `layer`, một lượt tra hỏng giữa chừng sau khi mở) — khối lượng việc thuộc một lượt hardening riêng, không phải một sửa nhanh trong review này.
- [x] [Review][Defer] Một nhóm khoảng trống độ phủ nhỏ hơn, gộp lại: (a) không ca nào chứng minh `.DB` viết hoa được nạp như `.db` dù doc-comment khẳng định (NFR14); (b) không ca nào chứng minh lỗi `read_dir` khác `NotFound` (vd. từ chối quyền) vẫn trả tập lớp rỗng mà không panic — đúng đường vừa sửa ở Lượt 1; (c) không ca nào xoá tệp của một lớp bị `conflict_with` từ chối để chứng minh nó không còn bị khoá; (d) không ca nào gọi `senses()` (chỉ `lookup()`) trên hai lớp khác nhau cùng `entry_id` để chứng minh chúng KHÔNG trộn dữ liệu; (e) không ca nào ép `layer.lookup()` hỏng giữa chừng để chứng minh `SkipReason::LookupFailed` hoạt động đúng; (f) không ca nào dựng một **thư mục** tên `*.db` để chứng minh nó bị từ chối an toàn; (g) nhánh `FtsTrigram`/nhánh `CharIdx` 2-ký-tự chỉ xuất hiện trong bench `#[ignore]`, chưa có ca khẳng định nào ở tầng gom — deferred, hardening cho lượt sau, không chặn story này.

*(Dismiss — noise/đã được thiết kế bù trừ: `SkipReason::DuplicateSourceCodeInFile` không test được vì `dict_source.code TEXT NOT NULL UNIQUE` đã cấm ca này ở tầng lược đồ cho mọi tệp hợp lệ — code phòng thủ cho một tệp bị chỉnh sửa tay, không phải lỗ hổng · `senses_sharing_one_ord_are_still_ordered_deterministically` "đúng một phần nhờ may" trên fixture nhỏ — chính doc-comment của test đã ghi rõ đây là lý do cổng tĩnh song hành tồn tại, hai lớp phòng thủ có chủ ý · `MERGE_NOUNS` chứa `"source"` có rủi ro dương tính giả — đánh đổi đã cân nhắc trong chính doc-comment của cổng · cổng `.rs` nói chung bị vượt qua được bằng dựng chuỗi lúc chạy (`format!`, `.chars().collect()`) — giới hạn cố hữu của quét văn bản tĩnh, không riêng story này · một `DirEntry` hỏng giữa vòng lặp không dựng được bằng test portable — chính agent review cũng xác nhận không khả thi · `in_layer` sắp theo `code` chưa từng thật sự phải sắp lại gì trên fixture hiện tại — đúng nhưng rủi ro thấp, thao tác hai dòng.)*

---

## Dev Notes

### 🎯 Hình dạng MỤC TIÊU — suy ra từ FR28–FR32 và từ lược đồ đã có, đừng phát minh thêm

```rust
// src-tauri/src/ports/dict_source.rs
//
// AD-2: ĐÚNG BA cổng. Đây là cổng thứ nhất. Cổng thứ tư phải là một AD mới.

use crate::core::dict::{LookupMode, LookupResult, QueryRoute, SenseRecord, SourceInfo};
use crate::core::store::StoreError;

/// Một tệp `.db` từ điển, nhìn qua cổng.
///
/// 🔴 Đơn vị là **MỘT TỆP**, không bao giờ một **NGÔN NGỮ** (AD-44 ⑤). Một adapter
/// theo ngôn ngữ phá mệnh đề *"gỡ một lớp = xoá một file"* của AD-10 và làm FR36
/// không nghiệm thu được bằng test thật nữa.
pub trait DictionarySource {
    /// `dict_meta('layer')` — `"base"` hoặc mã lớp gỡ rời. Đọc từ CHÍNH tệp.
    fn layer(&self) -> &str;

    /// Các nguồn tệp này mang. `dict-core.db` mang **sáu**; mỗi lớp gỡ rời mang một.
    fn sources(&self) -> &[SourceInfo];

    /// 🔴 `route` **nhận từ chỗ gọi** (AD-44 ①) — adapter không tự phân xử lại một
    /// câu hỏi thuộc về CẢ LƯỢT TRA.
    fn lookup(
        &self,
        query: &str,
        mode: LookupMode,
        route: QueryRoute,
    ) -> Result<LookupResult, StoreError>;

    /// Pha hai (§Quyết định #1B): đọc nghĩa cho một tập đầu mục **do chỗ gọi chọn**.
    /// Đọc theo LÔ — không một truy vấn cho mỗi đầu mục.
    fn senses(&self, entry_ids: &[i64]) -> Result<Vec<SenseRecord>, StoreError>;
}
```

```rust
// src-tauri/src/core/dict/  — kiểu bản ghi sống ở đây, không nhân bản sang ports/

/// Một nguồn, khoá theo **`code` (chuỗi)** — 🔴 KHÔNG `source_id` (số):
/// `id = 1` tồn tại ở cả ba tệp và trỏ ba nguồn khác nhau.
pub struct SourceInfo { pub code: String, pub display_name: String }

/// Một **mục nghĩa** = một hàng `dict_sense`. FR29: nhiều từ loại ⇒ nhiều mục,
/// không nối `gloss` thành một chuỗi.
pub struct SenseRecord {
    pub entry_id: i64,
    pub sense_id: i64,
    pub pos: Option<String>,
    /// 🔴 FR35 — nhãn ngoại ngữ là một **TRƯỜNG**, không đoán từ nội dung `pos`.
    pub pos_lang: Option<String>,
    pub gloss: String,
    pub note: Option<String>,
    pub ord: i64,
    /// FR30 — treo theo **TỪ LOẠI** (`sense_id`), không theo đầu mục.
    pub examples: Vec<ExampleRecord>,
    /// FR30 — bảng RIÊNG với ví dụ: trích dẫn mang **xuất xứ**.
    pub citations: Vec<CitationRecord>,
}

pub struct ExampleRecord {
    pub text: String,
    pub translation: Option<String>,
    /// Thứ AC10 dùng để nói *"bản dịch ví dụ này là tiếng Anh"*.
    pub translation_lang: Option<String>,
    pub ord: i64,
}

pub struct CitationRecord {
    pub text: String,
    pub work: Option<String>,
    pub author: Option<String>,
    pub ord: i64,
}

/// 🔴 Một nhóm cho một **nguồn**. AD-19: không có bước hợp nhất nào, ở bất kỳ đâu.
pub struct SourceGroup {
    pub layer: String,
    pub source: SourceInfo,
    pub entries: Vec<EntryHit>,
}

/// Một lớp không nạp được — **giá trị**, không phải một dòng log.
/// *"Rỗng im lặng bị cấm; rỗng có lý do thì không"* (AD-44 ④).
pub struct SkippedLayer { pub path: PathBuf, pub reason: SkipReason }

pub struct GroupedLookup {
    /// Một GIÁ TRỊ của cả lượt tra — không phải của từng tệp (AC5).
    pub route: QueryRoute,
    /// 🔴 Gồm cả `NoBranchQueryTooShort` — *"rỗng có lý do"*, không phải *"không có
    /// kết quả"* (AD-44 ④). 1.17 đọc đúng trường này.
    pub branch: QueryBranch,
    pub groups: Vec<SourceGroup>,
    pub skipped: Vec<SkippedLayer>,
}

/// Điểm vào PHA MỘT. `pick_route` chạy **ở đây**, đúng một lần (AD-44 ①).
///
/// ⚠️ `mode` là **tham số từ chỗ gọi**, không đoán từ nội dung — cùng luật
/// `LookupMode` đã chốt ở 1.11.
pub fn lookup_grouped(layers: &DictLayers, query: &str, mode: LookupMode) -> GroupedLookup;
```

### 🔴 Lược đồ tệp `.db` — chép từ `tools/dict-build/src/schema.rs`, đừng đoán tên cột

| Bảng | Cột story này đọc |
|---|---|
| `dict_meta` | `key`, `value` — 🔴 `'layer'` · `'schema_version'` |
| `dict_source` | `id`, `code`, `display_name` *(+ bốn trường giấy phép — **không đọc ở story này**, chúng thuộc 1.19/10.4)* |
| `dict_entry` | `id`, `source_id`, `lang`, `headword`, `headword_simp` *(1.11 đã đọc)* · `reading`, `han_viet` — ⚠️ **`han_viet` là ÂM ĐỌC, không phải NGHĨA** *(`schema.rs:41-42`)*; **tab Hán Việt là Story 1.16**, không phải story này |
| `dict_sense` | `id`, `entry_id`, `source_id`, `pos`, `pos_lang`, `gloss`, `note`, `ord` |
| `dict_example` | `id`, `sense_id`, `text`, `translation`, `translation_lang`, `ord` |
| `dict_citation` | `id`, `sense_id`, `text`, `work`, `author`, `ord` |

Chỉ mục đã có sẵn *(`ENTRY_INDEXES_DDL`)*: `idx_sense_entry` · `idx_example_sense` · `idx_citation_sense`. 🔴 **Chúng tồn tại chính xác vì đường đọc này** — `schema.rs:106-108`: *"cả ba đều là mục tiêu JOIN trên khoá ngoại mà đường đọc […] sẽ dùng — thiếu chỉ mục thì JOIN quét toàn bảng"*. **Đừng thêm chỉ mục** *(chạm `schema.rs` ⇒ dựng lại `.db` ⇒ sai `sha256`)*.

### ⚠️ Sáu cái bẫy đã cắn dự án này rồi — đọc trước khi gõ

1. **`ORDER BY ord` trần.** VietPhrase tách `/` vô điều kiện ⇒ nhiều `dict_sense` **cùng `ord`** *(`deferred-work.md`, Story 1.10)*. Thiếu khoá phụ `id` ⇒ hai lượt chạy hai thứ tự ⇒ một ca flaky, và một ca flaky bị gỡ chứ không được sửa.
2. **N+1 truy vấn.** 3.177 đầu mục × 3 tệp × 3 bảng. Một truy vấn cho mỗi đầu mục là ba bậc độ lớn, và nó *"chạy đúng"* trên fixture 20 hàng.
3. **Lô co giãn giết `prepare_cached`.** `run()` của `query.rs` dùng `prepare_cached` có lý do — *"một lượt tra cứu là đường nóng của NFR1"*. Một câu SQL sinh theo số phần tử thật của lô là **một hình dạng SQL mới mỗi lần** ⇒ cache trống ⇒ biên dịch lại câu ở mỗi lượt gõ.
4. **`source_id` là cái bẫy im lặng nhất trong story.** Ba tệp, ba bảng `dict_source` riêng, `id = 1` ở cả ba. Gom theo `id` dán nhãn *"Thiều Chửu"* cho một nghĩa của CVDICT — **FR31 vỡ, không lỗi, không test hành vi nào đỏ** trừ khi ca test dùng **ít nhất hai tệp**.
5. **`entry_fts` external-content không tự đầy.** Fixture thiếu `INSERT INTO entry_fts(entry_fts) VALUES('rebuild');` ⇒ nhánh 3 trả rỗng và mọi ca *"xanh"*.
6. **Chuỗi tiếng Việt có dấu trong `.rs` là một cổng ĐỎ.** `check-i18n.mjs` Kiểm A quét `src-tauri/**/*.rs`; **comment thì được**, **chuỗi thì không**. Miễn trừ chỉ có `src-tauri/tests/**` và `src/selftest/**`.

### 🧠 Trí tuệ từ story trước — thứ đắt tiền, đừng học lại bằng tiền

- **1.11/1.11b:** `lookup()` nhận `ReadHandle`, không `ReadOnlyDb` — *"Story 1.13 gọi hàm này **một lần cho mỗi tệp** rồi gom; với một chữ ký nhận `ReadOnlyDb`, nó phải mở/đóng hoặc mượn lồng nhau"* `core/dict/mod.rs:266-276`. **Chữ ký đó được thiết kế cho story này. Dùng nó.**
- **1.11b đo thật đường en:** p95 **0,052–0,961 ms**. Đường en rẻ; ca đắt là **nhánh 2 zh một ký tự**.
- **1.12:** cổng ranh giới là **phép quét CHỮ**, không phải phân tích đồ thị gọi hàm — một re-export đổi tên lách được. Giới hạn **có sẵn** của khuôn, không phải thứ story này phải giải; không nhưng cũng đừng dựa vào cổng như thể nó ngữ nghĩa.
- **1.12:** **đừng** đoán trước rồi bắt hàm khớp phỏng đoán — chạy hàm, chép số thật ra. Áp cho **mọi** số của AC13.
- **1.10:** `dict_meta('layer')` là **một HÀNG trong bảng khoá/giá trị ĐÃ CÓ**, không phải cột mới ⇒ `sqlite_master` không đổi. Đọc nó là đường rẻ nhất để biết mình vừa mở tệp nào.

### 📌 Bối cảnh git — năm commit gần nhất

`5edbe0e` *(assert `StoreKind::Dict`)* · `5a68df7` *(deferred-work + sprint status + build)* · `dd7af61` *(nguồn `VIWIKTIONARY_EN`)* · `ed8ce52` *(fixture Thiều Chửu + VietPhrase, `tools/dict-build/tests/layers.rs`)* · `a3ed5cd` *(test dict-build + schema)*. Khuôn đang chạy: **mỗi lượt giao gồm mã + cổng + số đo**, và tài liệu quy hoạch **không** do dev sửa. HEAD hôm nay `7e38de8`, cây làm việc **sạch**.

### 🌐 Phiên bản đang ghim — không đổi một dòng nào

`rusqlite` *(feature `bundled`, `backup` **TẮT**)* — đường mở tệp **chỉ** ở `core/store/**`; story này không gõ tên crate đó. Rust `edition 2024`, `rust-version = "1.85"`. **Không** thêm crate: NFR15 đòi rà tương thích GPLv3 **trước** khi thêm và ghi vào bảng Stack — **đường rẻ là đừng thêm**, và story này không cần gì mới *(`std::fs::read_dir` + `std::path` là đủ cho quét thư mục)*.

### Project Structure Notes

```
src-tauri/src/
  core/dict/
    mod.rs        # ⚠️ SỬA — thêm kiểu bản ghi + tầng gom; không đụng pick_route/pick_branch/lookup
    query.rs      # ⚠️ có thể SỬA — thêm SQL đọc nghĩa; không giữ nguyên năm hàm nhánh đã có
    layer.rs      # ➕ MỚI — adapter một tệp + tập lớp
    senses.rs     # ➕ MỚI (tuỳ chọn) — SQL đọc nghĩa/ví dụ/trích dẫn theo lô
  ports/
    mod.rs        # ⚠️ SỬA — khai module cổng thứ nhất
    dict_source.rs# ➕ MỚI — trait DictionarySource (AD-2)
  lib.rs          # ⚠️ SỬA (Quyết định #3A) — mở tập lớp ở setup(), đóng ở RunEvent::Exit
src-tauri/tests/
  dict_sources.rs # ➕ MỚI — hành vi: gom, nhóm theo nguồn, bất đồng, FR36, bench NFR1
  dict_boundary.rs# ⚠️ SỬA — thêm cổng AC2/AC6/AC1; không hạ DICT_FLOOR/SRC_TAURI_RS_FLOOR
```

**Không** tệp nào ngoài danh sách này. Đặc biệt không `tools/**`, không `src/**` *(frontend)*, không `scripts/**`, không `_bmad-output/planning-artifacts/**`.

### References

- `_bmad-output/planning-artifacts/epics.md:1530-1576` — Story 1.13, tám AC gốc
- `ARCHITECTURE-SPINE.md:290-294` — **AD-19** *(không hợp nhất nguồn)*
- `ARCHITECTURE-SPINE.md:84-88` — **AD-2** *(đúng ba cổng)*
- `ARCHITECTURE-SPINE.md:151` — **AD-10** *(một tệp = một lớp; runtime không mã riêng cho từng nguồn)*
- `ARCHITECTURE-SPINE.md:571-640` — **AD-44** ①⑤ *(vị từ điều phối chạy TRÊN tầng gom; không adapter theo ngôn ngữ; không sổ đăng ký)*
- `prd.md:459-495` — **FR28–FR36**
- `src-tauri/src/core/dict/mod.rs:52-58, 264-292` — phạm vi *"một tệp, một lượt"* và ba lý do `route` là tham số
- `src-tauri/src/core/store/readonly.rs:35-60` — `Send + Sync` cho `app.manage`; chính sách phiên bản giao cho story này
- `tools/dict-build/src/schema.rs:21-114` — lược đồ sáu bảng + ba chỉ mục JOIN
- `tools/dict-build/src/insert.rs:105-140` — `dict_meta('layer')`, `schema_version`, `user_version`
- `src-tauri/tests/dict_lookup.rs:1-45, 240-316, 1313-1345` — bốn luật test, khuôn fixture, khuôn bench
- `deferred-work.md` — FR36 hành vi · khoá theo `code` · 18 đầu mục trùng VietPhrase · bảng NFR1 nhánh 2

---

## Dev Agent Record

### Agent Model Used

`claude-opus-5` (Claude Code, `bmad-dev-story`) — 2026-08-05.

### Debug Log References

#### ① 🔴 Đối chứng dương của AC2 — lượt ĐỘT BIẾN chạy thật, không phải một lời hứa

Chèn tạm **hai** dòng vào `src-tauri/src/core/dict/layer.rs::DictLayer::close()`:

```rust
let _ = "thieu-chuu";
let _ = "dict-core.db";
```

Chạy `cargo test --test dict_boundary`:

```
test the_layer_set_never_hardcodes_a_db_filename ... FAILED
test the_runtime_never_names_a_single_dictionary_source ... FAILED
1 chỗ dưới `src-tauri/src/**` viết mã riêng cho một nguồn cụ thể:
test result: FAILED. 9 passed; 2 failed
```

**Hoàn nguyên**, chạy lại: `11 passed; 0 failed`. ⇒ Cả hai cổng **thật sự bắt được** hình dạng vi phạm, không xanh vì không có gì để bắt.

⚠️ Ngoài lượt đột biến một lần này, **mỗi cổng còn mang một đối chứng dương THƯỜNG TRỰC** chạy ở mọi lượt CI: nó gieo một chuỗi vi phạm tổng hợp vào chính bộ so khớp và khẳng định bộ so khớp đỏ trên nó *(và **không** đỏ trên ca hợp lệ gần nhất — `global.db`, `merge_by_key`, `ORDER BY e.id`)*. Một lượt đột biến chạy tay là bằng chứng cho **hôm nay**; đối chứng thường trực là bằng chứng cho **mọi ngày sau**.

#### ② Bench đỏ lần đầu vì đường dẫn tương đối — ghi lại vì nó sẽ cắn lượt sau

```
AURA_DICT_BENCH_DIR trỏ tới tools/dict-build/out — không phải một thư mục
```

`cargo test` chạy nhị phân với **cwd = `src-tauri/`**, không phải gốc kho. Doc-comment của bench nay ghi thẳng vế đó cộng vế `--release`. *(Doc-comment của `dict_lookup.rs::bench_three_branches_on_the_real_dictionary` mang **cùng** cái bẫy ở dòng `AURA_DICT_BENCH_DB=tools/dict-build/out/dict-core.db` — **không sửa**, tệp đó ngoài phạm vi story; ghi vào §Completion Notes ⑥.)*

#### ③ `cargo fmt` — cây vốn KHÔNG sạch trước story này

`cargo fmt --check` báo diff ở **12** tệp, trong đó có `core/i18n/mod.rs` · `core/scope/resolve.rs` · `core/store/{mod,pragmas,schema}.rs` · `tests/{config_invariants,ipc_contract,scope_*,store_*}.rs` — **không tệp nào của story này chạm tới**, và CI *(`.github/workflows`)* **không** chạy `fmt`. ⇒ Chỉ định dạng **năm tệp story này sở hữu**; chạy `cargo fmt` toàn cây sẽ sửa tệp ngoài File List, tức phá AC14.

### Completion Notes List

#### ① Ba quyết định của Task 0 — chốt **TRƯỚC** dòng mã đầu tiên (2026-08-05)

Không có chỉ đạo khác của Ice ⇒ đi đúng khuyến nghị của story: **B · A · A**.

| # | Câu hỏi | Chọn | Lý do ghi lại |
|---|---|---|---|
| 1 | Đọc nghĩa một pha hay hai pha | **B — hai pha** | `lookup_grouped()` trả nhóm + đầu mục *(rẻ, khớp số đo 1.11)*; `senses()` đọc nghĩa cho một tập **do chỗ gọi chọn**. Phương án A buộc đọc nghĩa cho 3.177 đầu mục × 3 tệp với truy vấn `山` ⇒ chắc chắn vượt trần 10 ms, và đường ra duy nhất là một `LIMIT` — tức story này tự quyết một **chính sách sản phẩm** mà 1.11 đã giao tường minh cho **1.17**. Với B, chính sách *"hiện bao nhiêu"* ở lại 1.17, và NFR1 đo được **hai pha riêng** thay vì một con số gộp không quy trách nhiệm được. |
| 2 | 18 đầu mục trùng của VietPhrase | **A — để nguyên** | Trùng **TRONG một nguồn**; AD-19 không phân xử ca này. Gộp lúc đọc đưa một hàm gộp vào đúng module mà AC6 cấm có hàm gộp, và cổng của AC6 sẽ phải mang ngoại lệ đầu tiên. Dữ liệu giữ đúng như nguồn ghi; 1.17 trình bày gọn mà không cần dữ liệu đổi. |
| 3 | Nối tập lớp vào `lib.rs` | **A — nối ở story này** | Mở **một lần lúc khởi động** theo khuôn `open_global_store` *(ghi chẩn đoán rồi đi tiếp)*, đóng ở `RunEvent::Exit` — NFR14 + FR112. Mở N pool SQLite ở phím đầu tiên người dùng gõ là hình dạng chắc chắn vỡ NFR1. |

#### ② 🔴 NFR1 ĐO THẬT trên đường gom — và §Quyết định #1B được số đo XÁC NHẬN

Đo 2026-08-05 trên **ba tệp `.db` thật** *(194.998.272 + 5.787.648 + 160.083.968 byte)*, bản **release**, 200 lượt, bỏ 10 lượt làm nóng. `[profile.release]` **không đổi một dòng**.

**Pha một — `lookup_grouped` trên 3 lớp:**

| Nhánh | Truy vấn | Nhóm | Hàng | p50 | **p95** | p99 | Trần | |
|---|---|---:|---:|---:|---:|---:|---:|---|
| 1 — B-tree chính xác | `山` | 6 | 8 | 0,196 | **0,223** | 0,268 | 1 ms | ĐẠT |
| 2 — `char_idx` 1 ký tự | `山` | 7 | **6.563** | 11,059 | 🔴 **12,569** | 13,368 | 10 ms | **VƯỢT** |
| 2 — `char_idx` 2 ký tự | `中國` | 5 | 354 | 2,423 | **3,608** | 4,318 | 10 ms | ĐẠT |
| 3 — FTS5 trigram | `中國人` | 4 | 35 | 0,454 | **0,563** | 0,644 | 1 ms | ĐẠT |
| en-1 B-tree *(thường)* | `running` | 1 | 1 | 0,133 | **0,144** | 0,320 | 1 ms | ĐẠT |
| en-1 B-tree *(HOA)* | `Running` | 1 | 1 | 0,144 | **0,239** | 0,323 | 1 ms | ĐẠT |
| en-2 trigram | `dic` | 1 | 572 | 1,057 | **1,575** | 1,842 | 10 ms | ĐẠT |

**Pha hai — `senses()` theo lô (`SENSE_BATCH = 64`):**

| Ca | Lớp | Đầu mục | p50 | **p95** | p99 |
|---|---|---:|---:|---:|---:|
| `山` substring — **một trang** | `vietphrase` | 20 | 0,151 | **0,294** | 0,369 |
| `山` substring — **tất cả** | `vietphrase` | 3.385 | 12,244 | 🔴 **13,015** | 13,380 |
| `中國` substring — một trang | `base` | 20 | 0,223 | **0,299** | 0,352 |
| `中國` substring — tất cả | `base` | 147 | 0,748 | **0,972** | 1,309 |
| `中國人` trigram — tất cả | `base` | 13 | 0,179 | **0,305** | 0,408 |
| `dic` substring — một trang | `base` | 20 | 0,229 | **0,315** | 0,422 |
| `dic` substring — tất cả | `base` | 572 | 3,732 | **5,110** | 5,785 |

**Bốn kết luận, cả bốn đều từ số chứ không từ lý luận:**

1. 🔴 **§Quyết định #1B được XÁC NHẬN.** Hydrate **một trang 20 đầu mục** hết **0,29–0,32 ms** ở **mọi** ca. Hydrate **cả** 3.385 đầu mục hết **13,015 ms**. Con số thứ hai **chính là** thứ phương án A *(một pha)* sẽ phải trả **bên trong** `lookup_grouped`, **cộng dồn cho cả ba tệp**. Hai pha đẩy chi phí đó sang chỗ **quyết định được**, và chỗ đó là 1.17.
2. **Chi phí theo SỐ HÀNG, không theo số tệp.** 1.11 đo một tệp: 3.177 hàng ⇒ 7,324 ms. Nay 6.563 hàng qua ba tệp ⇒ 12,569 ms — **2,07× hàng ⇒ 1,72× thời gian**. Gom nhiều tệp không thêm chi phí cố định đáng kể. ⇒ Đường ra **không** phải *"mở ít tệp hơn"*.
3. 🔴 **VƯỢT trần ⇒ GHI SỐ VÀ BÀN GIAO. Story này KHÔNG thêm `LIMIT`** — AC13 nói thẳng, và `deferred-work.md` §1-11 đã chốt đường ra là một **quyết định sản phẩm** chạm hợp đồng Panel Lookup. Ice **đã** chọn *"chấp nhận nguyên trạng"* một lần cho cùng câu hỏi ở lượt review 1.11.
4. ⚠️ **Trần 10 ms là số DẪN XUẤT, không phải NFR1.** NFR1 cho **100 ms** đầu-cuối; 10 ms là phần backend theo giả định `[A1]` *(~99,95 ms cho IPC + render — thứ **chưa ai đo**)*. Vượt trần backend nghĩa là dư địa cho hai thứ chưa đo còn **87,4 ms** thay vì 90 ms, **không** nghĩa là NFR1 vỡ. Số nghiệm thu thật của NFR1 chỉ có sau khi Panel Lookup tồn tại.

#### ③ AC11 nghiệm thu trên **FIXTURE** — và đây là vế phải đọc nguyên văn

Mệnh đề *"lớp HVTĐTD hiện từ loại · ví dụ · trích dẫn **bằng tiếng Việt**"* nghiệm thu bằng một fixture mang **đúng hình dạng** HVTĐTD *(`pos_lang = 'vi'`, ví dụ + trích dẫn tiếng Việt)*, **không** bằng dữ liệu thật — `dict-hvtdtd.db` **không tồn tại** vì chưa có nguồn thô *(`src-tauri/resources/dict/README.md:13`; `prd.md:856` [A2])*.

- ✅ **Đã nghiệm thu:** đường mã **phân biệt được** nhãn tiếng Việt với nhãn ngoại ngữ, và **không đánh mất** trường nào trên đường từ SQLite ra bản ghi.
- **Chưa nghiệm thu:** hình dạng đó **trên dữ liệu HVTĐTD thật**. Đã ghi thành một bàn giao **có tên** ở `deferred-work.md`.
- **Đừng đánh dấu FR35/FR36 là *"đã nghiệm thu trên dữ liệu thật"***.

#### ④ FR36 — món nợ mở từ Story 1.10 **ĐÓNG Ở ĐÂY**

`tests/dict_sources.rs::deleting_any_detachable_layer_keeps_the_whole_lookup_suite_green` **xoá tệp `.db` thật** rồi **mở lại tập lớp**, và chạy **cùng một** hàm mệnh đề trước/sau — không một nhánh `#[cfg]`, không một ca nào phải sửa. Ba vế làm nó không xanh giả được:

1. Danh sách lớp gỡ rời **dẫn xuất từ chính tập lớp** *(lọc `layer != "base"`)*, không viết cứng — *"một lớp gỡ rời **bất kỳ**"* (`epics.md:1572`) nghiệm thu đúng nghĩa, và ca sẽ tự phủ một lớp thứ ba vào ngày nó tồn tại.
2. **Đối chứng dương trước khi xoá**: lớp đó **thật sự** đóng góp một nhóm. Không có vế này, *"xoá xong vẫn xanh"* và *"lớp đó chưa bao giờ được nạp"* đọc **giống hệt nhau**.
3. Tệp đã xoá phải **không** xuất hiện trong `skipped` — *gỡ một lớp* là thao tác **BÌNH THƯỜNG** (FR112), không phải một lỗi dữ liệu.

⚠️ Vế còn lại: phép thử chạy trên **fixture**, không trên ba tệp thật *(195 MB, `.gitignore: *.db` — AD-25)*. Cùng ràng buộc mọi story từ 1.9 tới nay đã chấp nhận **có tên**.

#### ⑤ §Quyết định #2 — 18 đầu mục trùng của VietPhrase: **ĐỂ NGUYÊN**, kèm bàn giao

Dữ liệu giữ đúng như nguồn ghi. Tra `不是他的对手` vẫn trả **HAI** `dict_entry` từ **cùng** nguồn, và vì khoá gom là `code`, cả hai nằm trong **MỘT** nhóm `vietphrase` — không thành hai khối.

🔴 **Bàn giao cho Story 1.17:** một nhóm nguồn có thể chứa **nhiều đầu mục cùng `headword`**; Panel Lookup phải trình bày được ca đó mà **không đổi dữ liệu**. Con số hôm nay: **18** trong `dict-vietphrase.db` *(46 trong nguồn thô)*. Đã ghi ở `deferred-work.md`.

#### ⑥ Ba thứ story này **CỐ Ý KHÔNG LÀM**, và mỗi thứ có một lý do

1. **Không sửa hai chỗ tài liệu đang lệch** — `epics.md:1510` *(vế "`dict/` dùng nó")* và sơ đồ mermaid AD-13 `:189` *(cạnh `dict --> matching`)*. Chủ sở hữu là **John (PM)** và **Winston**; cả hai đã ở `deferred-work.md` từ Story 1.12. Dev theo **thân Rule** AD-17 `:236` + AD-44 ③, và cổng `matching_boundary.rs::the_dictionary_lookup_path_never_calls_the_matcher` vẫn xanh — `core/dict/**` không gọi Matcher một lần nào.
2. **Không sửa doc-comment của `dict_lookup.rs::bench_three_branches_on_the_real_dictionary`** dù nó mang **cùng** cái bẫy đường-dẫn-tương-đối vừa cắn *(§Debug Log ②)*. Tệp đó ngoài File List của story này. Ghi ra để lượt sau không mất thời gian truy lại.
3. **Không thêm crate nào, không chạm `Cargo.toml`/`Cargo.lock`, không chạm `[profile.release]`, không chạm `tools/dict-build/**`.** `std::fs::read_dir` + `std::path` đủ cho quét thư mục; `std::sync::LazyLock` *(ổn định từ Rust 1.80, dự án ghim 1.85)* đủ cho ba câu SQL hằng.

#### ⑦ Nghiệm thu — mọi cổng đã chạy, và đây là số

- `npm run build` ✅ *(chạy **TRƯỚC** `cargo test` — `generate_context!` nhúng `dist/` lúc biên dịch)*
- `cargo test --locked --manifest-path src-tauri/Cargo.toml` ✅ — **163 ca xanh, 0 đỏ, 2 `#[ignore]`** *(hai bench cần tệp `.db` thật)*. Trong đó **22 ca mới** ở `dict_sources.rs` và **7 cổng mới** ở `dict_boundary.rs`.
- Sáu cổng `.mjs` ✅ — `check:deps` · `check:i18n` *(**16 khoá · 9 placeholder**, đúng như AC14 đòi)* · `check:commands` · `check:tokens` · `check:dict` · `check:dict-manifest`.
- `git status` đối chiếu AC14 ✅ — không một dòng `.vue`/`.ts`/`.css`, không `tools/**`, không `scripts/**`, không `Cargo.toml`/`Cargo.lock`, không tài liệu quy hoạch *(`prd.md` · `epics.md` · `ARCHITECTURE-SPINE.md` không đổi một byte)*.
- **Không hạ một sàn quần thể nào.** `DICT_FLOOR = 1` · `SRC_TAURI_RS_FLOOR = 20` · `MATCHING_FLOOR`/`SRC_RS_FLOOR` · `store_boundary::RS_FLOOR = 20` · `scope_boundary::RS_FLOOR = 20` · `check-i18n::RS_FLOOR = 21`/`VUE_FLOOR = 1` · `check-deps::RUST_TREE_FLOOR = 200` — **giữ nguyên**. Story này **thêm** một sàn mới *(`SRC_ONLY_RS_FLOOR = 20`, quần thể thật 31)*, không nới sàn cũ.
- **Không nới `FORBIDDEN` của `dict_boundary.rs`** *(`LIKE` · `GLOB` · `instr(`)* — luật đó áp cho **mọi** SQL mới của story, gồm cả ba câu đọc nghĩa.

### File List

**Mới:**

| Tệp | Vai trò |
|---|---|
| `src-tauri/src/ports/dict_source.rs` | Cổng **thứ nhất** của AD-2 — trait `DictionarySource` *(AC1)* |
| `src-tauri/src/core/dict/layer.rs` | Adapter **một tệp** + tập lớp quét thư mục + `SkipReason` *(AC3, AC4)* |
| `src-tauri/src/core/dict/senses.rs` | Đường đọc nghĩa · ví dụ · trích dẫn **theo lô** *(AC7, AC8, AC13)* |
| `src-tauri/tests/dict_sources.rs` | 22 ca hành vi + bench NFR1 đường gom *(AC3–AC13)* |

**Sửa:**

| Tệp | Thay đổi |
|---|---|
| `src-tauri/src/core/dict/mod.rs` | Kiểu bản ghi *(`SourceInfo` · `SenseRecord` · `ExampleRecord` · `CitationRecord` · `SourceGroup` · `GroupedLookup`)* + `lookup_grouped` + khai hai module. Không đụng `pick_route`/`pick_branch`/`lookup`. |
| `src-tauri/src/ports/mod.rs` | Khai `mod dict_source` + tái xuất; bảng trạng thái ba cổng của AD-2 |
| `src-tauri/src/lib.rs` | `open_dict_layers` ở `setup()` · `close_dict_layers` ở `RunEvent::Exit` · hằng `DICT_RESOURCE_DIR` |
| `src-tauri/tests/dict_boundary.rs` | **7 cổng mới**: sàn quần thể `src/**` · không literal mã nguồn · không tên tệp `.db` · không hàm hợp nhất · `ports/**` không mang cài đặt · `ORDER BY ord` phải có khoá phụ |
| `_bmad-output/implementation-artifacts/deferred-work.md` | Bàn giao: FR36 **đóng** · phán quyết 18 đầu mục trùng · số NFR1 đường gom · HVTĐTD dữ liệu thật còn mở · `app.manage` chưa có người tiêu thụ |
| `_bmad-output/implementation-artifacts/sprint-status.yaml` | `ready-for-dev` → `in-progress` → `review` |
