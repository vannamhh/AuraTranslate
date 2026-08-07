---
baseline_commit: c3efb204d76239785f8ee27014be392126ba9c35
---

# Story 1.15: Tác phẩm trên đĩa và đường vào văn bản tối thiểu

Status: done

> ✅ **Mọi AC đã chạy, gồm cả AC10(a).** Ice nghiệm thu tay 2026-08-06 trên macOS/WKWebView —
> xem §Nghiệm thu tay. Hai món nợ treo qua ba story *(đường hiển thị lỗi kho, kéo-thả tệp
> thật)* đã đóng, và mọi ô trong §Tasks/Subtasks nay đều `[x]`.
>
> ⚠️ **Nhưng đừng đọc thành "đã xong mọi nền tảng":** cả năm bảng chạy **chỉ trên macOS**.
> Bốn đường **Windows-only theo bản chất** *(tên thiết bị dành riêng · `remove_dir_all` với
> tệp đang mở · trần `MAX_PATH` · kéo-thả qua WebView2)* **chưa từng chạy ở đâu**, và không
> không phép kiểm nào trên macOS làm chúng đỏ được. Nợ mở trong `deferred-work.md`, giao lượt
> QA trước khi phát hành.

## Change Log

| Ngày | Thay đổi |
|---|---|
| 2026-08-06 | ✅ **Code review đóng, và mọi AC nay đã chạy — story `done`.** Ba lớp review song song + đối chứng tay: bảng số của lượt triển khai **trung thực** *(chín cổng PASS, 177 test — đo lại đúng)*, nhưng ba lỗi **nặng** nằm ngoài mọi cổng, khoá vào nhau thành một đường mất dữ liệu: ① tạo Tác phẩm **trùng tên xoá trắng Tác phẩm cũ** *(ép lộ bằng một lượt chạy thật, không suy luận — và test của story đang **hợp thức hoá** đúng lỗi đó)*; ② kéo-thả **ghi xuống đĩa không có bước xác nhận** *(phá thẳng AC1)*; ③ form nhập **dùng được đúng một lần mỗi phiên**. Ice chốt **sáu quyết định** *(tự đánh số · gỡ khoá `meta_too_new` · AC9 đạt-chữ-chưa-đạt-mục-đích · cắt BOM không đụng CRLF · trần 100 MB · thả tệp điền vào ô)*. **20 patch áp hết**, 7 ca test mới ⇒ **184 passed**, chín cổng xanh. 🔴 Và Ice **chạy tay trên máy thật**: đóng nốt **AC10(a)** cùng lượt kéo-thả tay — hai món nợ treo qua **ba story** (1.7 → 1.8 → 1.15). ⚠️ Nợ mới mở thay chỗ: **toàn bộ nghiệm thu chỉ chạy trên macOS**, bốn đường Windows-only chưa chạy ở đâu. |
| 2026-08-06 | **Triển khai xong 13/13 task, 12 test mới, 177/177 `cargo test` xanh, cả chín lệnh DoD PASS.** Một nợ KHÔNG đóng được: bảng nghiệm thu tay đường hiển thị lỗi kho trong webview thật (AC10a) — môi trường triển khai (agent CLI) không có công cụ điều khiển GUI desktop để dựng/đọc một cửa sổ Tauri thật. Mũi thăm dò kéo-thả (Task 0) verify bằng đọc mã nguồn `tauri-runtime`/`tauri` đã ghim thay vì kéo tay thật, cùng lý do môi trường. Cả hai ghi rõ trong `deferred-work.md` §1-15 làm nợ QA người trước khi phát hành, không giấu. Xem §Completion Notes cho danh sách đầy đủ các chỗ làm khác story và vì sao. |
| 2026-08-06 | 🔴 **Ice chốt cả sáu quyết định — story hết chặn.** #1 **(b) kéo-thả + (a) dán văn bản** *(không `rfd`, không gỡ `BANNED_CRATES`)* · #2 **(a) khai `uuid =1.24.0`** kèm rà NFR15 · #3 **(b) `Store::write` trả dữ liệu, ghi `meta.json` NGAY SAU khi commit, nguyên tử `tmp+rename`** · #4 · #5 · #6 **theo mặc định**. ⚠️ **Một hệ quả mới phát sinh từ #1 và được vá tại chỗ:** kéo-thả là thao tác **chuột thuần** ⇒ **NFR17** *(mọi thao tác làm được hoàn toàn bằng bàn phím)* thủng ở đúng nhánh mở tệp. Vá bằng **ô nhập đường dẫn** đi kèm vùng kéo-thả — 0 phụ thuộc, 0 permission. Xem Quyết định #1. |
| 2026-08-06 | Tạo story. Baseline `c3efb20`, ✅ **cây làm việc SẠCH**. Phân tích song song bốn hướng: `epics.md` §Story 1.15 + Epic 1/2/5 (ràng buộc xuôi dòng) · `ARCHITECTURE-SPINE.md` **857 dòng** + cả 4 tệp `reviews/` · `prd.md` + `addendum.md` + `DESIGN.md` + `EXPERIENCE.md` + `mockups/{library-and-import,empty-states,data-integrity}.html` · `1-14`, `1-7`, `1-2`, `1-5` + **toàn bộ `deferred-work.md` (499 dòng)** · và **mã thật** `src-tauri/src/**` + `src/**` + `scripts/*.mjs`. Phát hiện **1 chặn thật** *(hộp thoại chọn tệp — `tauri-plugin-dialog` bị cổng CI cấm)*, **2 mâu thuẫn tài liệu** *(mockup `data-integrity.html` vẽ sai cây `.atproj`; chuỗi empty-state quảng cáo `.docx` và vi phạm Kiểm D)*, **3 tension kiến trúc spine không phân xử**, **6 quyết định phải chốt ở Task 0**, và **10 mục `deferred-work.md` gọi đích danh story này**. |

**Epic:** 1 — Nền móng ứng dụng & Tra cứu ngoại tuyến tức thì
**Story key:** `1-15-tac-pham-tren-dia-va-duong-vao-van-ban-toi-thieu`
**Covers:** FR13 *(chỉ nhánh **dán tay** + `.txt`/`.md` — `epics.md:819`)* · FR96 · FR97 · FR102
**Governed by:** **AD-9** *(chủ — `.atproj` là một thư mục)* · **AD-30** *(chủ — lược đồ có phiên bản, mở tiến không mở lùi)* · **AD-33** *(`meta.json` là dẫn xuất, ghi bởi chính `store::Writer` của Tác phẩm)* · AD-7 *(năm loại kho)* · AD-11 + AD-12 *(một writer nối tiếp, checkpoint là quyết định của ứng dụng)* · AD-2 *(`ports::ProjectStore`)* · AD-18 *(ngôn ngữ nguồn là trường **bất biến** trong `meta.json`, không phải cấu hình hai tầng)* · AD-21 / NFR16 *(Rust không bao giờ trả văn bản hiển thị)* · AD-23 *(phạm vi filesystem hai tầng)* · AD-34 *(mọi thao tác qua `CommandRegistry`)* · AD-39 *(chuỗi pipeline nhập sống ở `core/segment/`)* · AD-8 *(`.atproj` ghi **trước**, chỉ mục ghi **sau**)*
**Ràng buộc xuôi dòng phải tôn trọng:** AD-3 · AD-4 · AD-37 *(segment — Story 2.1)* · AD-32 *(gộp/tách Chương — Epic 2)* · AD-42 *(neo ảnh — Epic 6)*
**NFR:** NFR9 · NFR10 · NFR14 · NFR15 · NFR16 · NFR17 · *(chuẩn bị chỗ đứng cho)* NFR2 · NFR4 · NFR18
**Ngày tạo:** 2026-08-06

---

## 🔴 ĐỌC TRƯỚC TIÊN — NĂM VIỆC STORY NÀY KHÔNG LÀM, VÀ NÓI THẲNG THAY VÌ ĐÁNH DẤU ĐẠT

### ① `.docx` — story này **TỪ CHỐI**, không đọc

`epics.md:819` viết nguyên văn cho Epic 1:

> FR13 ở đây chỉ mở đường vào tối thiểu để có văn bản mà tra; **nhánh `.docx` đóng ở Epic 6**, nhập hàng loạt và mọi pipeline làm sạch đóng ở Epic 6.

⚠️ **Cái bẫy:** `docx-rs = "=0.4.22"` **ĐÃ có trong `Cargo.toml`** (cho `core::export`, AD-38). Nó ở đó vì Story 1.2 cài trọn bảng Stack ở commit đầu, **không** phải vì nó sẵn sàng dùng. **Đừng gọi một dòng `docx_rs` nào trong story này.** Một `.docx` phải bị từ chối **bằng phần mở rộng**, trước khi mở tệp, trước một byte ghi nào.

⚠️ **Và đừng cài AD-38 sớm.** AD-38 nói về ca *"`.docx` bảng hai cột — kiểm hình dạng là cổng vào"* ở Epic 8. Ca của story này khác hẳn: **một lời từ chối theo định dạng**, không phải một phép phân xử giữa hai hình dạng hợp lệ.

### ② KHÔNG tách segment — nhưng phải nói ra hệ quả, không im lặng

AD-4 (`ARCHITECTURE-SPINE.md:99`) viết: *"tách segment chạy **khi nhập Chương** và kết quả **lưu xuống** `.atproj`… Không có đường mã nào tính lại ranh giới lúc nạp Chương."* AD-37 (`:441`) đòi cờ kết đoạn tính **cùng lượt** với ranh giới câu.

Nhưng FR23 và luật tách câu *(`。！？；` cho tiếng Trung, `. ! ?` có xử lý viết tắt cho tiếng Anh)* là **Story 2.1**, không phải story này. Cài một bộ tách "tạm" ở đây nghĩa là **đóng băng vĩnh viễn** những ranh giới sai — AD-3 nói `segment.id` bất biến và id đã về hưu không bao giờ tái dùng.

⇒ Story này lưu **văn bản nguồn của Chương ở dạng nguyên khối**, **không** tạo bảng `segment`, **không** tạo một dòng `segment` nào. **Hệ quả phải ghi ra, không giấu:** mọi Chương nhập ở Epic 1 sẽ có `segment_count = 0`, và **Story 2.1 phải xử lý chúng bằng một thao tác tường minh** *(bước di trú, hoặc thao tác "tách segment lần đầu" có mặt trong giao diện)*, **không** bằng một đường tính ngầm lúc nạp Chương — đường đó là vi phạm AD-4 trực tiếp. Xem **Quyết định #4**.

### ③ KHÔNG tách Chương — FR14 đóng ở Epic 6

AC của story này: *"một Tác phẩm được tạo với **đúng một Chương**"*. Đó là FR2, không phải FR14.

⚠️ AD-39 (`:489-493`) có một hàng bảng nói rằng *"một dòng văn bản chưa chia Chương — tệp `.txt`/`.md`/`.docx`, văn bản dán tay ⇒ **CÓ** bước tách Chương. Đây là FR14."* Hàng đó **đúng**, nhưng nó mô tả pipeline **đầy đủ** của Epic 6. Story này cài **đường tối thiểu**.

🔴 **Ràng buộc mà story này phải để lại:** AD-39 (`:485`) cấm *"đổi thứ tự hay chèn bước **sau** lệnh ghi"*. ⇒ Đường nhập của story này phải đặt lệnh ghi **ở cuối**, và mọi bước biến đổi văn bản phải nằm **trước** nó — kể cả khi hôm nay chỉ có một bước. **Đừng ghi xuống đĩa rồi mới sửa văn bản.**

### ④ KHÔNG dựng `library-index.db`

Story 5.2 (`epics.md:3390`) chốt: *"`.atproj` ghi **trước**, `library-index.db` ghi **sau**"*, và *"`meta.json` là thứ `Indexer` đọc khi quét, **không phải** nguồn Library đọc trực tiếp lúc chạy"*. `Indexer` là **thành phần duy nhất** ghi vào `library-index.db` (AD-8).

⇒ Story này ghi `.atproj` và **dừng ở đó**. Màn hình Library của story này đọc thẳng `meta.json` **chỉ vì chưa có chỉ mục** — và điều đó phải được ghi thành một dòng comment tại chỗ, không để Epic 5 tưởng đó là kiến trúc.

### ⑤ KHÔNG dựng lưới Tác phẩm, bộ lọc, sắp xếp, bốn trạng thái vòng đời

`src/modes/LibraryMode.vue:3-6` đã ghi sẵn: *"Lưới Tác phẩm, bộ lọc, sắp xếp và bốn trạng thái vòng đời thuộc **Epic 5**"*. Story này chỉ thêm **đường vào** — đủ để có văn bản mà Story 1.16 hiển thị ở Panel Source.

---

## Story

As a người dịch,
I want đưa một đoạn văn bản vào công cụ và biết chắc nó nằm trong một thư mục tôi copy đi được,
So that dữ liệu của tôi không bị khoá trong ứng dụng.

---

## Ranh giới phạm vi — ĐỌC TRƯỚC KHI GÕ DÒNG ĐẦU TIÊN

| Trong phạm vi | không Ngoài phạm vi (và ai sở hữu) |
|---|---|
| Hình dạng `.atproj/` trên đĩa: `meta.json` · `project.db` · `assets/` | Nội dung `.atproj` lớn dần *(Glossary Epic 3 · TM Epic 7 · prompt Epic 4 · ảnh Epic 6)* |
| `PROJECT_MIGRATIONS` + `StoreSpec::project(path)` | `StoreSpec::library_index` *(Story 5.2)* |
| `work.id` UUID v4 · `chapter.id` số nguyên cục bộ | `segment.id` *(Story 2.1)* |
| Số phiên bản lược đồ trên **cả hai** tệp + đường **từ chối mở** bản mới hơn | Bước di trú thứ hai *(story nào đổi lược đồ trước)* |
| Đường nhập: **dán văn bản** + **tệp `.txt`/`.md`** | `.docx` *(Epic 6)* · nhập hàng loạt / tách Chương *(Epic 6)* · nhập từ URL *(Epic 6)* · song ngữ hai cột *(Epic 6)* |
| Từ chối `.docx` bằng một lỗi có `MessageKey` | Đọc nội dung `.docx` *(Epic 8 dùng `docx-rs` cho **xuất**; Epic 6 cho **nhập**)* |
| Cắm tầng Tác phẩm thật vào `ScopeResolver` *(nợ `deferred-work.md`)* | Ba chữ ký của `ScopeResolver` — **không đổi** |
| `ports::ProjectStore` — khai **hình dạng** | Cổng thứ tư — AD-2 khai **đúng ba** cổng, không thêm |
| Vòng đời mở/đóng kho thứ hai + `RunEvent::Exit` | Hiệu chỉnh sáu số `Tuning` *(Story 2.4)* |
| Bảng nghiệm thu tay đường hiển thị lỗi kho trong webview thật *(nợ)* | Bộ chạy test frontend — **không dựng** *(NFR15, quyết định của Ice)* |

**KHÔNG ĐỤNG** *(ranh giới đã chốt bốn story liên tiếp)*: `tools/**` · `core/dict/**` · `core/matching/**` · `dict-manifest.toml` · `_bmad-output/planning-artifacts/**` *(`epics.md` · `prd.md` · `DESIGN.md` · `EXPERIENCE.md` · `mockups/**` — lệch thì **ghi ra**, không sửa; đó là một lượt riêng của Ice, tiền lệ quyết định #3 Story 1.3)*.

**✅ ĐỤNG ĐÃ ĐƯỢC PHÉP** *(sau phán quyết 2026-08-06)*: `src-tauri/Cargo.toml` — **đúng một dòng**, thêm `uuid = { version = "=1.24.0", features = ["v4"] }` *(Quyết định #2)*.

**⚠️ ĐỤNG CHỈ NẾU MŨI THĂM DÒ TASK 0 ĐÒI**: `src-tauri/tauri.conf.json` *(`dragDropEnabled`)* — kéo theo `check:scope` + `check:scope:bundled` + đối chiếu `tests/config_invariants.rs`.

**VẪN KHÔNG ĐỤNG**: `src-tauri/capabilities/main.json` *(thêm permission = mở bề mặt IPC mới ⇒ **phải là một AD mới trước đã**, `SECURITY-NOTES.md:113`)* · `package.json` *(0 phụ thuộc npm mới — cả ba đường vào của Quyết định #1 không cần một cái nào)*.

---

## ✅ SÁU QUYẾT ĐỊNH — **ICE ĐÃ CHỐT CẢ SÁU 2026-08-06**. KHÔNG MỞ LẠI

> Khuôn Task 0 của Story 1.13 / 1.14: **chốt mọi quyết định mở TRƯỚC dòng mã đầu tiên**. Sáu mục dưới đây **đã chốt**; phần lý lẽ giữ nguyên để dev đọc được **vì sao**, không phải để cân nhắc lại.

| # | Phán quyết của Ice |
|---|---|
| **#1** | ✅ **(b) kéo-thả + (a) dán văn bản** — **không** `rfd`, **không** gỡ tên khỏi `BANNED_CRATES` |
| **#2** | ✅ **(a)** khai `uuid = { version = "=1.24.0", features = ["v4"] }`, kèm lượt rà NFR15 |
| **#3** | ✅ **(b)** `Store::write` trả dữ liệu meta; ghi `meta.json` **ngay sau khi giao dịch commit**, **nguyên tử** `tmp+rename`; **đường dựng lại từ `project.db` phải tồn tại và có test** |
| **#4** | ✅ Mặc định — **KHÔNG** tạo `segment`, không bảng `segment` |
| **#5** | ✅ Mặc định — `~/Documents/AuraTranslate/` qua `app.path()`, và **ghi ra tường minh** rằng scope động hôm nay cưỡng chế bằng **kỷ luật mã Rust**, không bởi framework |
| **#6** | ✅ Mặc định — **CHỈ nhận UTF-8**, từ chối tường minh thứ khác bằng cùng khuôn với `.docx` |

### ✅ Quyết định #1 — ĐÃ CHỐT: **kéo-thả + dán văn bản**

**Sự thật đứng sau phán quyết:** `tauri-plugin-dialog` **bị cấm bằng cổng CI**. `scripts/check-deps.mjs:137-163`:

```js
const BANNED_CRATES = [
  ['tauri-plugin-fs',     'AD-1 + AD-29 — Ice chốt 2026-08-03'],
  ['tauri-plugin-sql',    'AD-11 — dùng `rusqlite` trực tiếp'],
  ['tauri-plugin-dialog', 'cùng lý do: không phơi filesystem ra JS'],
]
const BANNED_NPM = ['@tauri-apps/plugin-fs', '@tauri-apps/plugin-sql', '@tauri-apps/plugin-dialog', …]
```

`npm run check:deps` là **bước CI đầu tiên** (`.github/workflows/ci.yml`) và exit 1 nếu tên đó có trong `Cargo.lock` hoặc `npm ls`. Lý lẽ ở `src-tauri/SECURITY-NOTES.md:71-82`: *"plugin tồn tại để phơi API ra JavaScript, đúng thứ NFR11 cấm"* — **Ice chốt 2026-08-03**.

Nhưng AD-23 (`:318`) lại nói scope động cấp *"**chỉ khi người dùng chọn qua hộp thoại**"*. ⇒ **Hai văn bản đều đã ký, và chúng gặp nhau ở đúng story này.**

| Đường | Được gì | Mất gì | Chi phí quyết định |
|---|---|---|---|
| **(a) Chỉ dán văn bản** | 0 phụ thuộc mới, 0 permission mới, 0 test đỏ | **KHÔNG đạt AC1** *(vế "mở một tệp `.txt` hay `.md`")* | Thu hẹp AC — **tầng epics**, cần Ice |
| **(b) Kéo-thả tệp vào cửa sổ** | 0 phụ thuộc mới. Tauri v2 phát `tauri://drag-drop` qua **event system**, mà `core:event:default` **đã** được cấp. Rust nhận **đường dẫn thật** rồi `std::fs::read` | Chưa có tiền lệ trong repo — **phải verify trước**. `dragDropEnabled` chưa khai trong `tauri.conf.json` *(mặc định `true`)*. UX kém hơn một hộp thoại | Thấp — nhưng đụng `tauri.conf.json` ⇒ `check:scope*` |
| **(c) Hộp thoại native trong Rust** *(`rfd`)* | Đúng chữ AD-23. Cùng tiền lệ AD-29 đã dùng cho `keyring` *(gọi crate **trực tiếp**, không qua plugin)* | **Phụ thuộc mới** ⇒ NFR15 đòi **mở tệp LICENSE trong nguồn đã tải mà đọc** rồi vào bảng Stack. `rfd` **không** có trong `Cargo.lock` hôm nay | 🔴 **Quyết định của Ice** — NFR15 nói rõ đây không phải hệ quả phụ của một story |
| **(d) `<input type="file">` trong webview** | 0 phụ thuộc, 0 permission *(là web platform thuần)* | 🔴 **Phá AD-1** *(frontend chỉ render + giữ state UI)* và **AD-16** *(mọi nội dung ngoài do **Rust** phân tích)*. Webview đọc nội dung tệp rồi đẩy chuỗi qua IPC | Cao — phá hai AD |

**✅ Ice chốt 2026-08-06: (b) + (a).** **Dev KHÔNG chuyển sang (c).** **Dev KHÔNG tự gỡ tên khỏi `BANNED_CRATES`.** (c) giữ làm đường nâng cấp cho lượt rà NFR15 kế tiếp của Ice, không phải cho story này.

⚠️ **Đường (b) vẫn phải VERIFY trước khi viết mã** — Task 0 chạy mũi thăm dò 15 phút trong app Tauri thật. Nếu mũi thăm dò TRƯỢT *(ba permission hiện có không đủ để nhận `tauri://drag-drop`)*, **không tự thêm permission** — dừng lại và báo Ice, vì thêm một mục vào `capabilities/main.json` là **mở một bề mặt IPC mới**, và `SECURITY-NOTES.md:113` nói thẳng: *"phải là một AD mới trước đã"*.

#### 🔴 Hệ quả của (b) mà phán quyết chưa chạm tới: **NFR17 thủng ở nhánh mở tệp**

NFR17 *(`prd.md` §7.5)*: *"Mọi thao tác làm được **hoàn toàn bằng bàn phím** — nghiệm thu bằng một vòng dịch trọn một Chương **không chạm chuột**."*

**Kéo-thả là thao tác chuột thuần.** Phép nghiệm thu của NFR17 vẫn **ĐẠT** *(vòng dịch một Chương bắt đầu được từ nhánh **dán văn bản**, mà dán là bàn phím)*, nhưng riêng thao tác **mở một tệp** sẽ **không có đường bàn phím nào** — và đó là một lỗ thật, không phải một chi tiết.

**✅ Vá bắt buộc, 0 phụ thuộc, 0 permission:** vùng kéo-thả đi kèm **một ô nhập đường dẫn** — người dùng gõ hoặc dán đường dẫn, Rust `std::fs::read` đúng đường đã dùng cho tệp kéo vào. Cùng một hàm thuần, hai đường vào.

⇒ **Nhánh "mở tệp" của AC1 có HAI đường:** kéo-thả *(chuột)* **và** ô nhập đường dẫn *(bàn phím)*. Đường thứ hai không phải tuỳ chọn.

⚠️ Ghi vào `deferred-work.md`: *"Đường mở tệp v1 là kéo-thả + ô nhập đường dẫn, không phải hộp thoại native. Nâng lên `rfd` khi Ice có lượt rà NFR15 mở bảng Stack — Quyết định #1 Story 1.15, Ice chốt 2026-08-06."*

⚠️ `tauri.conf.json` chưa khai `dragDropEnabled` *(mặc định `true` ở Tauri v2)*. Nếu mũi thăm dò cho thấy phải khai tường minh ⇒ chạm `tauri.conf.json` ⇒ **bắt buộc chạy `check:scope` và `check:scope:bundled`**, và đối chiếu lại `tests/config_invariants.rs`.

### ✅ Quyết định #2 — ĐÃ CHỐT: **(a) khai `uuid =1.24.0`**

| Đường | Thực tế đo được |
|---|---|
| **(a) `uuid = { version = "=1.24.0", features = ["v4"] }`** | 🔴 **`uuid 1.24.0` ĐÃ có trong `Cargo.lock:5034`** *(phụ thuộc bắc cầu của `tauri`)*, kéo theo `getrandom 0.4.3`. Khai tường minh ⇒ **0 byte payload thêm**, 0 crate mới trong cây. Giấy phép `MIT OR Apache-2.0` ⇒ tương thích GPL v3. ⚠️ NFR15 vẫn đòi **mở tệp LICENSE trong nguồn đã tải mà đọc** + thêm hàng vào bảng Stack |
| **(b) SQLite `hex(randomblob(16))`** rồi tự đặt bit version/variant | 0 phụ thuộc, 0 rà giấy phép. Nhưng viết tay phép đặt bit *(byte 6 → `0x4x`, byte 8 → `0x8x`..`0xBx`)* là chỗ dễ sai mà không cổng nào bắt |

**✅ Ice chốt 2026-08-06: (a).** Bắt buộc kèm một lượt rà NFR15 ghi vào §Completion Notes — đọc `~/.cargo/registry/src/**/uuid-1.24.0/LICENSE-MIT` và `LICENSE-APACHE` **trong nguồn đã tải**, **không tin nhãn registry** *(đúng phương pháp đã lập tiền lệ ở Story 1.1 và 1.2)*.

⚠️ Ghim bằng `=`: `uuid = { version = "=1.24.0", features = ["v4"] }`. **Không** `"1.24.0"` trần — trong Cargo nó nghĩa là `^1.24.0`.

⚠️ Thêm hàng vào bảng Stack của `ARCHITECTURE-SPINE.md` — tệp đó nằm trong `planning-artifacts/`, thuộc danh sách **KHÔNG ĐỤNG**. ⇒ Dev **soạn hàng và đề nghị**, **không tự sửa tệp**.

Đường (b) *(`hex(randomblob(16))` + tự đặt bit)* **đã bị loại** — đừng cài.

⚠️ **G4 — "v4" là ràng buộc do story TỰ SIẾT.** `ARCHITECTURE-SPINE.md:354` chỉ ghi *"UUID sinh lúc tạo"*, không nêu phiên bản. AC của `epics.md:1650` mới nói "v4". Ghi lại ở đây để về sau ai muốn UUIDv7 *(sắp xếp được theo thời gian — lợi cho chỉ mục Epic 5)* biết chính xác thứ đang chặn mình là gì.

### ✅ Quyết định #3 — ĐÃ CHỐT: **(b) ghi `meta.json` NGAY SAU khi giao dịch commit**

**AD-33 (`ARCHITECTURE-SPINE.md:404`) nguyên văn:**

> `meta.json` là **dẫn xuất từ `project.db`**, dựng lại được hoàn toàn. Nó được ghi bởi **chính** `store::Writer` của Tác phẩm đó, **trong cùng thao tác logic** với thay đổi sinh ra nó.

**Nhưng** `Store::write` nhận `F: FnOnce(&Transaction<'_>) -> SqlResult<T> + Send + 'static` — một **giao dịch SQL**. Và Ice đã đặt điều kiện 2026-08-04 *(`deferred-work.md`, quanh `writer.rs:159`)*: `Writer::shutdown()` **không có trần cho `handle.join()`**, rủi ro được chấp nhận **với điều kiện review tay mỗi khi một story mới ghi qua tầng này**, và lượt review đó soát rằng job ghi **không I/O, không gọi ra ngoài**.

🔴 **Story 1.15 là story ĐẦU TIÊN có job ghi kèm I/O tệp thật.** Nhét `fs::write("meta.json")` vào trong closure là **làm giả định đó sai**, và nếu ổ đĩa treo thì cả hàng đợi ghi treo theo — đúng thứ AD-11 tồn tại để chặn *(NFR2)*.

| Đường | Đánh giá |
|---|---|
| **(a) I/O bên trong write job** | **Phá điều kiện của Ice.** Một `fs::write` chậm chặn hàng đợi ghi ⇒ NFR2 mất hiệu lực mà không lần ra được nguyên nhân |
| **(b) `Store::write` trả về `T` = dữ liệu meta; ghi `meta.json` NGAY SAU khi giao dịch commit** | ✅ *"Cùng thao tác logic"* của AD-33 giữ nguyên **ở tầng thao tác**, không phải ở tầng giao dịch SQL. Job ghi vẫn thuần SQL. Cửa sổ hỏng: sập giữa commit và `fs::write` ⇒ `meta.json` cũ/vắng — **nhưng AD-33 nói `meta.json` dựng lại được hoàn toàn từ `project.db`**, nên đây không phải mất dữ liệu |
| **(c) không ghi `meta.json`, dựng nó lúc quét** | không Phá AD-9 *(Library đọc metadata **không cần mở SQLite**)* và NFR4 |

**✅ Ice chốt 2026-08-06: (b).** không (a) **đã bị loại** — không tuyệt đối không `fs::write` bên trong closure của `Store::write`. Kèm **hai ràng buộc bắt buộc**:
1. `meta.json` ghi **nguyên tử** — `write(<tmp>)` → `sync_all()` → `rename(<tmp>, meta.json)`. 🔴 Xem §"Khoảng trống atomic write" — **không tài liệu nào yêu cầu điều này**, nên nếu dev không tự làm thì không AC nào bắt được.
2. **Đường dựng lại `meta.json` từ `project.db` phải TỒN TẠI và có test.** AD-33 gọi nó là "dẫn xuất"; một mệnh đề dẫn xuất không có đường dựng lại chỉ là một comment.

### ✅ Quyết định #4 — ĐÃ CHỐT theo mặc định: **KHÔNG tạo `segment`**

Xem §ĐỌC TRƯỚC TIÊN ②. **✅ Ice chốt 2026-08-06: KHÔNG.** Không bảng `segment`, không dòng `segment` nào. `schema.rs:107-126` đã viết luật: *"Không thêm bước cho một lược đồ chưa tồn tại"* — mỗi story sở hữu bước di trú của **chính nó**, cùng lúc với bảng mà nó thật sự dùng.

**Nợ phải mở tường minh trong `deferred-work.md`:** *"Chương nhập ở Epic 1 có `segment_count = 0`. Story 2.1 phải xử lý bằng một thao tác tách tường minh, không bằng đường tính ngầm lúc nạp Chương (AD-4)."*

### ✅ Quyết định #5 — ĐÃ CHỐT theo mặc định: `~/Documents/AuraTranslate/`

AD-23 (`:318`): thư mục gốc chứa các `.atproj` mặc định là `~/Documents/AuraTranslate/`, cấp bằng **scope động** lúc chạy.

⚠️ **Nhưng scope động chưa có cơ chế nào cấp** — nó phụ thuộc Quyết định #1. Và `SECURITY-NOTES.md:57-69` cấm phát biểu sai: *"Capabilities canh bề mặt IPC (webview), **KHÔNG** canh Rust. `std::fs` và `rusqlite::Connection::open` không đi qua capabilities."*

**✅ Ice chốt 2026-08-06:** dùng `~/Documents/AuraTranslate/` *(phân giải bằng `app.path()`, **không viết cứng** — NFR14)*, và **ghi ra tường minh** trong §Completion Notes rằng *"scope động của AD-23 hôm nay được cưỡng chế bởi **kỷ luật mã Rust**, không bởi framework; nghiệm thu bằng **vắng mặt bề mặt**"* — đúng cách Story 1.2 đã phát biểu về `$APPDATA/**` (`1-2…md:428,672`). **Đừng viết vào báo cáo rằng "framework đã cưỡng chế".**

⚠️ `assetProtocol.scope` **KHÔNG được thêm** `$APPDATA` hay đường `.atproj` — hai test canh: `asset_protocol_scope_never_contains_appdata` và `asset_protocol_scope_has_exactly_the_one_readonly_resource_area` (`tests/config_invariants.rs`). Ảnh trong `assets/` là chuyện của Epic 6.

### ✅ Quyết định #6 — ĐÃ CHỐT theo mặc định: **CHỈ nhận UTF-8**

🔴 **Rủi ro nối giai đoạn thật, không phải hiểu nhầm.** FR126 *(phát hiện bảng mã UTF-8 · GB18030 · GBK · Big5 · UTF-16)* áp cho **mọi đường nhập văn bản** — nhưng nó đóng ở **Epic 6**. Story này ghi văn bản **xuống đĩa** ngay hôm nay.

`prd.md` gọi đích danh hạng lỗi này: *"cùng hạng lỗi với FR39 — **thất bại im lặng**… người dùng thấy 'tách được 1 chương' mà không hiểu vì sao — vì họ đang nhìn `ç¬¬ä¸€ç«` chứ không nhìn `第一章`… ở **đúng thao tác đầu tiên người dùng làm với ứng dụng**."*

Và AD-4 đóng băng ranh giới segment tính **một lần lúc nhập** ⇒ dữ liệu hỏng đó **không tự sửa được ở Epic 6**.

**✅ Ice chốt 2026-08-06: CHỈ nhận UTF-8, và TỪ CHỐI tường minh thứ khác** — cùng khuôn thông báo với `.docx`, cùng đường mã, không nhập một phần. Chi phí: một `String::from_utf8` trả `Err`. Lợi: đóng một lớp lỗi mà Epic 6 không sửa lại được.

⚠️ Đây là **mở rộng nhẹ so với chữ của AC** — AC không nhắc bảng mã. Ghi vào §Completion Notes ở mục *"🔴 CHỖ TÔI LÀM KHÁC STORY, và vì sao"*, kèm lý do: AD-4 đóng băng ranh giới segment tính lúc nhập, nên văn bản giải mã sai ghi xuống hôm nay là dữ liệu Epic 6 **không sửa lại được**.

---

## Acceptance Criteria

### AC1 — Dán văn bản **hoặc** mở tệp `.txt`/`.md` tạo một Tác phẩm có **đúng một Chương** *(FR13 nhánh tối thiểu, FR2)*

- **Given** người dùng dán văn bản trực tiếp, **hoặc** đưa vào một tệp `.txt`/`.md` bằng **một trong hai** đường đã chốt ở Quyết định #1 — **kéo-thả vào cửa sổ** *(chuột)* hoặc **ô nhập đường dẫn** *(bàn phím)*
- **When** xác nhận
- **Then** một Tác phẩm được tạo với **đúng một Chương** chứa văn bản nguồn
- **And** **không có gì được ghi xuống đĩa trước khi người dùng xác nhận** *(bất biến của mọi đường nhập — `EXPERIENCE.md#KF-1`)*
- **And** Chương mang trạng thái ban đầu *Chưa bắt đầu* *(FR5)*
- **And** ngôn ngữ nguồn được đặt **lúc tạo** và **không đổi được** về sau *(FR3, AD-18)*
- 🔴 **And** **cả ba** đường vào *(dán · kéo-thả · ô đường dẫn)* đổ vào **đúng một** hàm thuần ở `core/segment/import.rs` — không đường nào có bản sao của pipeline *(AD-39:498)*
- 🔴 **And** **NFR17**: tạo một Tác phẩm từ văn bản dán **và** từ một tệp đều làm được **hoàn toàn bằng bàn phím**; kéo-thả là đường **bổ sung**, **không phải đường duy nhất** cho nhánh tệp

### AC2 — Tác phẩm trên đĩa là **một thư mục** `<Tên>.atproj/` chứa `meta.json` · `project.db` · `assets/` *(FR96, AD-9)*

- **Given** một Tác phẩm
- **When** ghi xuống đĩa
- **Then** nó là một **thư mục** `<Tên>.atproj/` chứa **đúng** ba thành phần: `meta.json`, `project.db`, `assets/`
- **And** `assets/` tồn tại **kể cả khi rỗng** *(để Epic 6 không phải kiểm tra sự tồn tại của nó ở mọi đường ghi ảnh)*
- 🔴 **And** cây thư mục vẽ trong `mockups/data-integrity.html` *(`project.toml` · `chapters/*.chapter` · năm tệp `.db` · `images/`)* **KHÔNG được dựng theo** — xem §Mâu thuẫn tài liệu ②

### AC3 — Đọc `meta.json` lấy được metadata **không cần mở SQLite** *(AD-9, NFR4)*

- **Given** `meta.json`
- **When** đọc
- **Then** lấy được metadata của Tác phẩm mà **không** mở `project.db`
- **And** có một test **chứng minh bằng cách không chạm `project.db`** — không phải bằng một lời khẳng định

### AC4 — `work.id` là UUID v4 trong `meta.json`; `chapter.id` là số nguyên **cục bộ** trong `project.db` *(AD-28, `ARCHITECTURE-SPINE.md:642`)*

- **Given** `work.id`
- **When** tạo
- **Then** là **UUID v4** *(bit version = 4, bit variant = RFC 4122)*, lưu trong `meta.json`
- **And** `chapter.id` là **số nguyên cục bộ** trong `project.db`
- 🔴 **And** id đã về hưu **không bao giờ được tái dùng** *(AD-3)* — ⚠️ `INTEGER PRIMARY KEY` trần **tái dùng rowid** sau khi xoá; phải là `INTEGER PRIMARY KEY AUTOINCREMENT`
- **And** thứ tự Chương là **cột riêng `ord`**, sắp lại được mà không đụng `id` *(AD-3, AD-32)*

### AC5 — Copy `.atproj` sang máy khác mở được **nguyên vẹn** *(FR97, NFR9)*

- **Given** một `.atproj`
- **When** copy sang một đường dẫn khác và mở
- **Then** mở được nguyên vẹn
- **And** **không** đường dẫn tuyệt đối nào của máy cũ nằm trong `meta.json` hay `project.db`
- **And** test chứng minh bằng cách **copy sang một thư mục tạm khác rồi mở**, không bằng một lời khẳng định

### AC6 — Copy thư mục **là đủ** để sao lưu *(FR102)*

- **Given** người dùng muốn sao lưu
- **When** copy thư mục `.atproj`
- **Then** bản sao đó dùng được **ngay**, không cần một thao tác export riêng
- ⚠️ **And** giới hạn được ghi ra, không giấu: copy khi ứng dụng **đang mở** Tác phẩm đó có thể thiếu tối đa **5 giây** công việc cuối *(trần auto-save NFR18)* — **cũ hơn, không hỏng** *(`mockups/data-integrity.html`)*

### AC7 — `meta.json` và `project.db` **mỗi cái** mang một số phiên bản lược đồ *(AD-30)*

- **Given** `meta.json` và `project.db`
- **When** tạo
- **Then** mỗi cái mang một số phiên bản lược đồ
- **And** `project.db` dùng **`PRAGMA user_version`**, theo đúng quy ước đã khai ở `schema.rs:1-15`: **`0` = chưa có lược đồ**, bước di trú đầu tiên đánh số **1**, `to_version` tăng dần nghiêm ngặt
- 🔴 **And** gặp phiên bản **mới hơn** thì **TỪ CHỐI MỞ** và báo rõ, **không bao giờ ghi vào** — cho **cả hai** tệp. *(AC chỉ đòi "mang số phiên bản", nhưng AD-30 đòi hành vi này, và **ghi một con số không ai đọc là ghi một con số vô dụng**.)*
- **And** `project.db` đi qua đúng đường đã có: `StoreError::SchemaTooNew` → `err.store.schema_too_new` *(khoá **đã tồn tại** trong `vi.json`)*

### AC8 — Chọn một tệp `.docx` ⇒ báo rõ định dạng chưa nhận, **không sập**, **không nhập một phần**

- **Given** người dùng đưa vào một tệp `.docx` — **bằng bất kỳ đường nào trong hai** *(kéo-thả **hoặc** ô nhập đường dẫn)*
- **When** ở epic này
- **Then** màn hình báo rõ định dạng chưa nhận ở phiên bản hiện tại
- **And** **không** sập và **không** nhập vào một phần — **không** thư mục `.atproj` nửa vời nào còn lại trên đĩa
- **And** lỗi đi qua `IpcError { code, message_key, params, retryable }` với `retryable = false` *(chọn lại đúng tệp đó cũng cho kết quả ấy — một nút thử lại ở đó là **nói dối**)*
- **And** **không** một dòng `docx_rs` nào được gọi
- **And** *(Quyết định #6)* văn bản không giải mã được bằng UTF-8 bị từ chối bằng **cùng khuôn** đó

### AC9 — Tầng Tác phẩm thật được cắm vào `ScopeResolver` *(nợ `deferred-work.md`)*

- **Given** `WorkScope` hôm nay là một struct **rỗng đánh dấu chỗ** và `ScopeResolver::global_only()` là hàm dựng **duy nhất**
- **When** story này xong
- **Then** có một hàm dựng **thứ hai** mang tầng Tác phẩm thật
- **And** 🔴 **ba chữ ký của ba hàm phân giải KHÔNG đổi** — `deferred-work.md` nói thẳng điều này
- **And** đường sản phẩm **không còn luôn truyền `None`**
- ⚠️ **And** nếu cổng `scope_boundary.rs` đỏ ở token `"ScopeKind"` vì story này trở thành consumer đầu tiên ngoài `core/scope/**` — đó là **hành vi đúng**, xử lý có ý thức, **không** "sửa cho vừa"

### AC10 — Hai món nợ nghiệm thu đóng, và cả hai có **bảng số thật**

- **Given** `deferred-work.md` giao đích danh story này
- **When** story xong
- **Then** **(a)** bảng nghiệm thu tay đường hiển thị lỗi kho **trong một webview thật** *(cần một `$APPDATA` chỉ-đọc)* đã chạy và ghi vào §Debug Log References
- **And** **(b)** lượt **review tay** giả định *"job ghi không chặn, không I/O, không gọi ra ngoài, không `Store::write` lồng nhau"* đã chạy cho **mọi** job ghi mới, kết quả ghi ra — đúng điều kiện Ice đặt 2026-08-04

### AC11 — Ranh giới KHÔNG CHẠM giữ nguyên, và **mọi cổng xanh**

- **Given** chín lệnh của definition-of-done
- **When** chạy trên cây sau story
- **Then** tất cả exit 0
- **And** mọi phép kiểm **mới** có bảng **đỏ-rồi-xanh** *(*"một cổng chưa từng đỏ là một cổng chưa từng canh"*)*
- **And** **không** tệp nào trong danh sách KHÔNG ĐỤNG bị sửa

---

## Tasks / Subtasks

- [x] **Task 0 — Đường cơ sở + mũi thăm dò kéo-thả** *(không gõ một dòng mã sản phẩm nào trước khi xong)*
  - [x] Chạy **cả chín** lệnh DoD trên cây sạch `c3efb20` và **ghi số TRƯỚC khi sửa gì** *(không có nó thì không phân biệt được "story này làm đỏ" với "vốn đã đỏ")* — xem §Debug Log References, tất cả PASS
  - [x] 🔴 **Mũi thăm dò cho Quyết định #1(b)** — verify bằng đọc mã nguồn `tauri-runtime-2.11.3`/`tauri-2.11.5` đã ghim *(⚠️ không bằng kéo-thả tay thật — môi trường phiên này không có công cụ điều khiển GUI desktop; xem giới hạn ghi rõ ở §Debug Log References)*. **VERDICT: PASS**, và mạnh hơn giả thiết ban đầu — `on_window_event` nhận `WindowEvent::DragDrop` cần **0 permission**, không đi qua ACL/capabilities
  - [x] ⚠️ Không TRƯỢT nên nhánh báo Ice không áp dụng ở đây; nợ nghiệm thu tay "kéo-thả thật bằng chuột người" được mở mới, xem Task 12
  - [x] **Rà NFR15 cho `uuid`** — đọc `LICENSE-MIT`/`LICENSE-APACHE` trong nguồn `uuid-1.24.0` đã tải. Hàng đề nghị cho bảng Stack đã soạn ở §Debug Log References, chờ Ice thêm
  - [x] Chép **sáu phán quyết đã chốt** vào §Completion Notes kèm lý do

- [x] **Task 1 — Lược đồ `project.db`** *(AC4, AC7)*
  - [x] `Cargo.toml`: thêm **đúng một dòng** `uuid = { version = "=1.24.0", features = ["v4"] }` *(sau khi Task 0 rà NFR15)*. Xác nhận `cargo tree` **không** thêm crate mới nào vào cây *(nó đã ở đó qua `tauri`)*, và `npm run check:deps` vẫn xanh
  - [x] `schema.rs`: thêm `PROJECT_MIGRATIONS: &[Migration]` — bước **1** gồm `SCHEMA_MIGRATION_LOG_DDL` *(tái dùng hằng đã có)* + `WORK_DDL` + `CHAPTER_DDL`
  - [x] `WORK_DDL`: **đúng một hàng**, mang `work_id TEXT NOT NULL` *(UUID v4)*, `name`, `source_lang`, `genre`, `created_at`, `updated_at` *(ISO-8601 **UTC** — quy ước `ARCHITECTURE-SPINE.md:647`)*, `CHECK (id = 1)`
  - [x] `CHAPTER_DDL`: `id INTEGER PRIMARY KEY **AUTOINCREMENT**` 🔴 *(không phải `INTEGER PRIMARY KEY` trần — nó tái dùng rowid, phá AD-3)*, `ord INTEGER NOT NULL`, `title TEXT`, `source_text TEXT NOT NULL`, `status TEXT NOT NULL`, `created_at`, `updated_at`
  - [x] **Không** bảng `segment` *(Quyết định #4)*. **Không** bảng nào cho Glossary/TM/prompt/asset — `schema.rs:107-126`: *"Không thêm bước cho một lược đồ chưa tồn tại"*
  - [x] `mod.rs`: thêm `StoreSpec::project(path: PathBuf) -> Self` theo đúng khuôn `StoreSpec::global`; **gỡ** hai comment `mod.rs:130-133` và `:260-261` nói *"không có `StoreSpec::project` hôm nay"*

- [x] **Task 2 — `meta.json`: hình dạng, ghi nguyên tử, đường dựng lại** *(AC2, AC3, AC7, Quyết định #3)*
  - [x] Struct `WorkMeta` + `Serialize`/`Deserialize`, hằng `META_SCHEMA_VERSION: u32 = 1`
  - [x] Trường: `meta_schema_version` · `work_id` · `name` · `source_lang` · `genre` · `created_at` · `updated_at` · `chapter_count` *(cache của FR7 — AD-33 nêu đích danh tiến độ là thứ `meta.json` cache lại)*
  - [x] 🔴 **Ghi nguyên tử**: `write(<tmp>)` → `sync_all()` → `rename(<tmp>, meta.json)`. ⚠️ **Không tài liệu nào yêu cầu điều này** — xem §Khoảng trống atomic write
  - [x] **Đường dựng lại `meta.json` từ `project.db`** + test — AD-33 gọi nó là "dẫn xuất"; không có đường dựng lại thì mệnh đề đó chỉ là một comment
  - [x] **Từ chối mở** khi `meta_schema_version` > bản ứng dụng, **không ghi vào** *(AD-30)*

- [x] **Task 3 — `ports::ProjectStore`** *(AD-2)*
  - [x] `src-tauri/src/ports/project_store.rs` theo khuôn `ports/dict_source.rs` *(89 dòng — mẫu chuẩn)*
  - [x] ⚠️ Cổng chỉ khai **hình dạng**, **không mở gì** — `tests/dict_boundary.rs::ports_declare_shape_and_never_open_anything` quét thư mục này
  - [x] Cập nhật bảng ba cổng ở `ports/mod.rs:11` *(cột "Chủ" của `ProjectStore` đang ghi "chưa khai · Story 1.15")*
  - [x] **Không** thêm cổng thứ tư

- [x] **Task 4 — Đường nhập** *(AC1, AC8; AD-39)*
  - [x] `src-tauri/src/core/segment/import.rs` — hàm **thuần**, không chạm `tauri`, không chạm `rusqlite`
  - [x] 🔴 **Ba đường vào, MỘT hàm thuần**: dán văn bản · kéo-thả *(nhận đường dẫn từ `tauri://drag-drop`)* · ô nhập đường dẫn. Hai đường sau gặp nhau ở `std::fs::read` rồi đổ vào **cùng** hàm với đường dán *(AD-39:498 — không module nào giữ bản sao)*
  - [x] không Đường kéo-thả và ô đường dẫn nhận **đường dẫn**; **không** để webview đọc nội dung tệp rồi đẩy chuỗi qua IPC *(AD-1 + AD-16 — mọi nội dung ngoài do **Rust** phân tích)*
  - [x] 🔴 **Thứ tự cố định, lệnh ghi ở CUỐI** *(AD-39:485 cấm chèn bước sau lệnh ghi)*:
        `phân loại nguồn` → `giải mã (UTF-8, không thì từ chối)` → `chuẩn hoá tối thiểu` → `tạo 1 Chương` → **`ghi`**
  - [x] Từ chối `.docx` **bằng phần mở rộng, TRƯỚC khi mở tệp** — không đọc một byte, không tạo thư mục
  - [x] Giải mã bằng `String::from_utf8` *(trả `Err`)* — **KHÔNG** `from_utf8_lossy` *(Quyết định #6, bẫy #8)*
  - [x] **Không** bước tách Chương *(FR14 → Epic 6)*, **không** bước làm sạch *(FR124/125 → Epic 6)*, **không** bộ dò bảng mã *(FR126 → Epic 6)* — nhưng **chừa chỗ** cho chúng ở đúng vị trí trong chuỗi, có comment nêu tên story sở hữu

- [x] **Task 5 — Hình dạng `.atproj` trên đĩa** *(AC2, AC5, AC6)*
  - [x] `src-tauri/src/core/library/atproj.rs` — dựng `<Tên>.atproj/` + `assets/` bằng `create_dir_all`
  - [x] ⚠️ `Store::open` **không tự tạo thư mục cha** — `pragmas::open_connection` có `SQLITE_OPEN_CREATE` nên tạo được *tệp*, thư mục là việc của story *(khuôn `lib.rs:124-130`)*
  - [x] 🔴 **Không đường dẫn tuyệt đối nào** vào `meta.json` hay `project.db` *(AC5)*
  - [x] **Dọn sạch khi trượt**: bất kỳ lỗi nào ở giữa ⇒ không để lại thư mục nửa vời *(AC8)*
  - [x] Chuẩn hoá tên thư mục từ tên Tác phẩm — ⚠️ `pragmas.rs:44-53` mở kết nối **không cờ `URI`** *(thư mục người dùng chứa `?`)*; xử lý ký tự cấm của **cả hai** nền tảng *(NFR14)*

- [x] **Task 6 — Bề mặt IPC** *(AC1, AC8)*
  - [x] `src-tauri/src/commands/project.rs` — **chép nguyên khuôn** `commands/config.rs`
  - [x] Hai lớp: **hàm thuần** nhận `Option<&Store>` *(đường sản phẩm thật, `tests/**` gọi được không cần webview)* + **`#[tauri::command]` mỏng** trong `pub mod wire`, dùng **`try_state`** **không `state()`** *(`panic = "abort"` giết cả tiến trình)*
  - [x] Tên `snake_case`, động từ + danh từ. ⚠️ **Tên trên dây = tên hàm** ⇒ vỏ phải trùng tên hàm thuần
  - [x] Struct qua IPC: **KHÔNG** `#[serde(rename_all = "camelCase")]`; `BTreeMap` không `HashMap`
  - [x] Đăng ký ở `lib.rs` qua `generate_handler!`. **KHÔNG** thêm mục ACL vào `capabilities/main.json` *(ACL canh command của **plugin**)*

- [x] **Task 7 — Vòng đời kho thứ hai** *(AC5; nợ `deferred-work.md`)*
  - [x] State cho Tác phẩm đang mở trong `lib.rs` *(mở khi mở Tác phẩm, đóng khi đóng — tiến trình **không** thoát)*
  - [x] 🔴 Đóng ở `RunEvent::Exit` cùng khuôn `close_global_store` — **bắt buộc theo NFR14**: trên Windows một tệp còn mở là một `remove_dir_all` thất bại
  - [x] ⚠️ Mục `deferred-work.md` về `Checkpointer::shutdown()` để luồng nền treo lửng ghi rằng nó *"chỉ trở thành rủi ro thật nếu một story sau thêm luồng **khởi động lại kho mà không thoát tiến trình**"* — 🔴 **story này làm đúng chuyện đó.** Đổi trạng thái mục đó từ "vô hại" sang "rủi ro thật" trong `deferred-work.md`
  - [x] ⚠️ Ghi ra: kho thứ hai ⇒ **luồng checkpoint thứ hai + pool thứ hai (4 kết nối nữa)** — sáu số `Tuning` là **TẠM, chưa cái nào được đo** *(chủ: Story 2.4)*

- [x] **Task 8 — `ScopeResolver` tầng Tác phẩm** *(AC9)*
  - [x] Điền `WorkScope` *(hôm nay là `pub struct WorkScope;` rỗng — `core/scope/mod.rs:181`; trường `work: Option<WorkScope>` ở `:171`)*
  - [x] Hàm dựng thứ hai cạnh `ScopeResolver::global_only()` *(`core/scope/mod.rs:188`)* — doc-comment ở `:165` đã ghi sẵn: *"Hôm nay `work` **luôn** là `None` và `ScopeResolver::global_only` là hàm dựng duy nhất…"*, và ba method phân giải không phải đổi chữ ký
  - [x] Nối đường sản phẩm để nó **không còn luôn truyền `None`**
  - [x] ⚠️ **Tuyệt đối không gõ tên crate SQLite** trong `core/scope/**` — kể cả **comment đuôi dòng** *(bộ quét `store_boundary.rs` chỉ miễn trừ dòng **bắt đầu** bằng `//`)*

- [x] **Task 9 — Chuỗi i18n + `MessageKey`** *(AC8, NFR16)*
  - [x] Thêm khoá vào `message_keys!` (`core/i18n/mod.rs:100-134`) — khai `required_params` **cạnh khoá**
  - [x] Đề xuất: `err.import.unsupported_format` `["format"]` · `err.import.not_utf8` `["path"]` · `err.project.create_failed` · `err.project.meta_too_new` `["found","supported"]`
  - [x] Thêm **cùng lượt** vào `src/i18n/vi.json` — object **phẳng**, khoá khớp `^[a-z0-9]+(\.[a-z0-9_]+)+$`, placeholder khớp `^[a-z_][a-z0-9_]*$`
  - [x] 🔴 **Kiểm D cấm hai tiếng `chúng tôi` và `bạn`** — xem §Mâu thuẫn tài liệu ③, chuỗi empty-state trong mockup **sẽ làm đỏ cổng**
  - [x] không `params` mang **dữ liệu**, không mang **câu**. Một `params: {"reason": "<câu>"}` là AD-21 thủng qua cửa sau
  - [x] ⚠️ Mọi chuỗi chẩn đoán trong `src-tauri/src/**/*.rs` viết **KHÔNG DẤU** *(Kiểm A; chỉ `tests/**`, `src/selftest/**`, `tools/**` được miễn trừ)*

- [x] **Task 10 — Giao diện đường nhập** *(AC1, AC8; NFR17)*
  - [x] `src/modes/LibraryMode.vue` — thêm empty state + đường nhập. Khuôn thị giác: `mockups/empty-states.html` ô "Library lần đầu" *(vạch `ornament` 34×2px → `.big` họ `read` 16px → `.small` → `.sugg` nền `sunken` → `.keys`)*
  - [x] 🔴 **Ba đường vào, cả ba đến được bằng bàn phím trừ kéo-thả**: ô dán văn bản · **ô nhập đường dẫn** *(vá NFR17 — không phải tuỳ chọn)* · vùng kéo-thả
  - [x] ⚠️ Vùng kéo-thả phải có **trạng thái focus nhìn thấy rõ** nếu nó nhận tiêu điểm, và **không** là phần tử duy nhất mang chức năng mở tệp *(NFR17)*
  - [x] ⚠️ **Không** copy nguyên chuỗi từ mockup — xem §Mâu thuẫn tài liệu ③
  - [x] Adapter IPC ở `src/config/project.ts` theo khuôn `bootstrap.ts` *(**không bao giờ ném**; ba trạng thái: có dữ liệu / `IpcError` thật / không có cầu IPC)*
  - [x] Đăng ký command ở `src/commands/index.ts` với `labelKey = 'command.' + id`. 🔴 **KHÔNG `import` `invoke`/`vue`/`dockview` ở tệp đó** — nó bị `check-{commands,layout,i18n}.mjs` nạp bằng **Node thuần**; một import giá trị giết cả loạt Kiểm hành vi. Phụ thuộc **TIÊM VÀO** qua `CommandDeps`
  - [x] Mọi text node qua `t()`/`tError()` *(Kiểm A2)*; miễn trừ phải có comment `aura-allow-text: <lý do>`
  - [x] Mọi `@click` là **đúng một** `dispatch('<id>')` *(AD-34)*
  - [x] ⚠️ Nếu render văn bản nguồn ở đâu đó: bề mặt đó phải **tự khai token `read-*`/`source-*`**, không kế thừa `ui-md` 1.5 của `body` *(`deferred-work.md`)*
  - [x] ⚠️ Bọc `try/catch` mọi lượt đọc từ đĩa — **ném TRƯỚC `mount()` = cửa sổ trắng** *(bài học 1.14)*

- [x] **Task 11 — Test Rust** *(mọi AC)*
  - [x] `src-tauri/tests/project_contract.rs` — khai phạm vi ở **dòng 1**, một tệp một mối quan tâm
  - [x] Ca bắt buộc *(tên = **câu mô tả hành vi**, snake_case, tiếng Anh, không tiền tố `test_`)*:
        `creating_a_work_lays_down_exactly_three_things_on_disk` ·
        `meta_json_is_readable_without_ever_touching_the_database` ·
        `the_work_id_is_a_v4_uuid_with_the_right_version_and_variant_bits` ·
        `a_retired_chapter_id_is_never_handed_out_again` ·
        `a_copied_project_folder_opens_at_a_different_path` ·
        `meta_json_can_be_rebuilt_from_the_database_alone` ·
        `a_newer_meta_schema_is_refused_without_touching_a_single_byte` ·
        `a_docx_is_refused_before_a_single_byte_is_written` ·
        `a_failed_import_leaves_no_half_built_folder_behind` ·
        `text_that_is_not_utf8_is_refused_the_same_way_a_docx_is` ·
        `pasted_text_and_a_read_file_travel_the_same_import_path` *(AC1 — không đường nào có bản sao pipeline)*
  - [x] 🔴 **Bốn luật của test tầng ghi** *(`store_contract.rs:1-30`)*: thư mục tạm **riêng** = `temp_dir()` + pid + `AtomicUsize` *(không `tempfile`)* · **drop `Store` TRƯỚC `remove_dir_all`** *(Windows)* · **không `sleep` dài** — lái cơ chế bằng `Tuning` thu nhỏ · **không ca nào treo khi trượt**
  - [x] Sửa `tests/ipc_contract.rs` nếu thêm `MessageKey` *(hai ca đối chiếu **hai chiều** với `vi.json` tự đỏ)*
  - [x] ⚠️ Nếu thêm biến thể `StoreError`: sửa `tests/store_contract.rs::every_store_error_converts_to_a_complete_ipc_error`
  - [x] ⚠️ `tests/scope_contract.rs::every_command_error_comes_from_the_store_vocabulary` — 🔴 **story này PHÁ mệnh đề đó** *(lỗi ".docx chưa nhận" không phải lỗi kho)*. Sửa/khoanh lại **có ý thức**, ghi lý do

- [x] **Task 12 — Nghiệm thu tay + trả nợ** *(AC10)* — 🔴 **KHÔNG hoàn tất trọn — xem mục 1**
  - [x] ✅ **ĐÃ ĐÓNG 2026-08-06 — Ice chạy tay trên macOS.** *(Ghi chép gốc của lượt triển khai giữ nguyên bên dưới để thấy nó đã từng bị chặn ở đâu và vì sao.)* ⚠️ Chỉ nghiệm thu trên **macOS/WKWebView** — vế Windows chưa đo, xem bảng ở §Nghiệm thu tay. ~~🔴 CHƯA LÀM ĐƯỢC, ghi thẳng thay vì đánh dấu đạt.~~ Bảng nghiệm thu đường hiển thị lỗi kho **trong webview thật** đòi dựng một cửa sổ Tauri thật, làm `$APPDATA` chỉ-đọc, rồi ĐỌC MÀN HÌNH bằng mắt người. Phiên triển khai này là một agent CLI **không có công cụ điều khiển GUI desktop** (không dựng/đọc được cửa sổ native, không có cầu debug-protocol tới WKWebView). Đã thử `osascript`/System Events — truy cập một phần nhưng không đủ để dựng và đọc một cửa sổ ứng dụng thật. Ghi vào `deferred-work.md` §1-15 làm nợ mở, giao QA người trước khi phát hành.
  - [x] Lượt **review tay** mọi job ghi mới: không I/O · không gọi ra ngoài · không `Store::write` lồng nhau · SQL là hằng. Ghi kết quả — xem §Completion Notes và `deferred-work.md` (mục `Writer::shutdown()` của Story 1.8/1.15)
  - [x] Bảng **đỏ-rồi-xanh** cho mọi phép kiểm mới — xem §Debug Log References
  - [x] Cập nhật `deferred-work.md`: đóng một phần kèm bằng chứng (ScopeResolver::with_work), mở năm mục mới cho những gì cố ý không làm hoặc chưa verify được
  - [x] Cập nhật `sprint-status.yaml`

- [x] **Task 13 — Chạy trọn định nghĩa hoàn thành** *(AC11)*
  - [x] `npm run check:deps` · `check:tokens` · `check:i18n` · `check:commands` · `check:layout` — cả năm PASS
  - [x] `npm run build` 🔴 **PHẢI trước `cargo test`** *(`generate_context!` nhúng `dist/` lúc biên dịch)* — chạy trước, PASS
  - [x] `cargo test --locked --manifest-path src-tauri/Cargo.toml` — **177 passed**, 0 failed
  - [x] `npm run check:scope` · `check:scope:bundled` *(bắt buộc — story chạm `lib.rs`)* — cả hai PASS
  - [x] Không cổng `.mjs` mới nào được thêm ở story này — không cần sửa `.github/workflows/ci.yml`

---

## Dev Notes

### Trạng thái repo hôm nay — SỐ, không phải mô tả

| Thứ | Số thật |
|---|---|
| Baseline | `c3efb204d76239785f8ee27014be392126ba9c35`, ✅ cây **sạch** |
| Module Rust | `src-tauri/src/` — `core/store/**` **2 197 dòng** *(7 tệp)* · `core/scope/**` 1 057 · `core/dict/**` 1 652 · `core/matching/**` 544 · `core/i18n/mod.rs` 286 · `commands/` 215 · `lib.rs` 279 · `ports/` 107 |
| Module **RỖNG** *(chỉ doc-comment)* | `core/library/mod.rs` **4 dòng** · `core/segment/mod.rs` **4 dòng** · `core/export` 6 · `core/tm` 3 · `core/webimport` 9 · `core/glossary` 4 · `core/ai` 10 |
| IPC command | **ĐÚNG HAI**: `bootstrap_config`, `put_config` |
| `MessageKey` | **7 khoá** |
| `vi.json` | **28 khoá**, nhóm `err.*`(7) · `command.*`(11) · `mode.*`(2) · `panel.*`(8) |
| Test Rust | **11 tệp, 9 054 dòng**, `cargo test --locked` = **165 passed** |
| Cổng `.mjs` | **9** |
| Test runner frontend | 🔴 **KHÔNG CÓ, và story này KHÔNG dựng** |
| Crate trong cây | macOS **343**, Windows ~346 |

### API thật của `core::store` — chép từ MÃ, không từ trí nhớ

```rust
pub type ReadHandle<'a> = &'a rusqlite::Connection;          // BÍ DANH, không phải kiểu bọc
pub use rusqlite::{Transaction, Error as SqlError, Result as SqlResult, Row, ToSql};

pub enum StoreKind { Global, Project, LibraryIndex, Dict }   // as_str(): "global"|"project"|"library-index"|"dict"

pub struct StoreSpec {
    pub kind: StoreKind,
    pub path: PathBuf,                     // 🔴 ĐÃ PHÂN GIẢI — module không tự tìm $APPDATA
    pub tuning: Tuning,
    pub migrations: &'static [Migration],  // 🔴 TRƯỜNG, không phải hằng tra theo kind
}
impl StoreSpec { pub fn global(path: PathBuf) -> Self }       // CHƯA CÓ ::project() — story này dựng

pub struct Migration { pub to_version: u32, pub sql: &'static str }
pub const SCHEMA_MIGRATION_LOG_DDL: &str;                     // TÁI DÙNG, đừng viết lại
pub const GLOBAL_MIGRATIONS: &[Migration];                    // 2 bước hôm nay

impl Store {
    pub fn open(spec: StoreSpec) -> Result<Store, StoreError>;
    pub fn write<T,F>(&self, job: F) -> Result<T, StoreError>
      where F: FnOnce(&Transaction<'_>) -> SqlResult<T> + Send + 'static, T: Send + 'static;
    pub fn read<T,F>(&self, job: F) -> Result<T, StoreError>
      where F: FnOnce(ReadHandle<'_>) -> SqlResult<T>;
    pub fn close(&self);                   // idempotent, cũng chạy trong Drop
    pub const fn schema_version(&self) -> u32;
    pub fn diagnostics(&self) -> Vec<String>;
}
```

**🔴 Hợp đồng thứ tự trong `Store::open` — KHÔNG PHẢI SỞ THÍCH** *(`mod.rs:504-590`, 9 bước có đánh số trong comment)*:

```
0. kind == Dict          ⇒ trả lỗi NGAY
1. mở kết nối, cờ TƯỜNG MINH (không cờ URI)
2. ĐỌC PRAGMA user_version — CHỈ ĐỌC, chưa một byte nào được ghi
3. found > target        ⇒ drop(conn) TƯỜNG MINH + Err(SchemaTooNew)
4. apply_writer_pragmas  (WAL · wal_autocheckpoint=0 · busy_timeout — ĐẶT RỒI ĐỌC LẠI)
5. found >= 1 && < target ⇒ backup_before_migration
6. migrate — chỉ tiến, mỗi bước MỘT giao dịch
7. ReaderPool::open   8. Checkpointer::spawn   9. Writer::spawn
```

🔴 Lý do bước 2 đứng trước bước 4: **`PRAGMA journal_mode = WAL` GHI VÀO database.** Đảo thứ tự là AC7 trượt **im lặng**.

**"Một writer nối tiếp" cưỡng chế bằng BỐN lớp, không phải một:** ① `rusqlite::Connection` là `Send` nhưng **không `Sync`** — trình biên dịch cưỡng chế *(đừng lách bằng `Arc<Connection>`/`unsafe impl`)*; ② một luồng sở hữu kết nối ghi, nhận việc qua `mpsc`, mỗi job **một giao dịch**; ③ mọi kết nối pool đặt `PRAGMA query_only = 1` **đọc lại xác nhận** — một `INSERT` qua `read()` bị **SQLite** từ chối, không phải một phép kiểm tự viết; ④ `tests/store_boundary.rs` quét cây nguồn.

⚠️ **Chống deadlock lồng nhau**: cờ thread-local `ON_WRITER_THREAD` ở `writer.rs:104`. Gọi `Store::write()` **từ trong** một write job trả `WriteFailed` **ngay**, không enqueue+chặn. 🔴 Đọc lại Quyết định #3 trước khi thiết kế đường ghi `meta.json`.

### Hình dạng `.atproj` phải dựng — AD-9, nguyên văn

```
<Tên>.atproj/
├── meta.json      # metadata Library đọc được KHÔNG CẦN MỞ SQLITE · có meta_schema_version
├── project.db     # SQLite, WAL · PRAGMA user_version · work/chapter
└── assets/        # ảnh là TỆP THẬT, hiển thị qua asset protocol (Epic 6)
```

**Cái gì KHÔNG nằm trong `.atproj`** *(`mockups/data-integrity.html`)*: chỉ mục Library · Glossary và TM **toàn cục** · khoá API *(Keychain — NFR11 cấm ghi vào **tệp dự án**)* · phím tắt và preset bố cục *(`GlobalOnly` — AD-18)*.

**Định danh** *(`ARCHITECTURE-SPINE.md:642`)*: `Work` = **UUID v4** · `Chapter`/`Segment`/mục Glossary/mục TM = **số nguyên cục bộ trong database chứa nó**. Id đã về hưu **không bao giờ tái dùng**.

🔴 **Đặt tên thực thể — CƯỠNG CHẾ:** Tác phẩm → `Work`, Chương → `Chapter`. **không CẤM `Project`, `Book`, `Novel`, `Document` cho `Work`.** *"Đuôi tệp `.atproj` là **ngoại lệ lịch sử**, không kéo theo tên thực thể."* ⚠️ Nhưng `StoreKind::Project` và `ports::ProjectStore` **đã tồn tại với tên đó** — ✅ **giữ nguyên, đừng đổi**; `store_contract.rs::store_kind_names_are_stable_machine_identifiers` đã khoá `"project"` làm định danh trên dây.

### 🔴 Ba tension spine KHÔNG phân xử — story này phải quyết có ý thức

| # | Hai văn bản đều đúng, và chúng không gặp nhau | Chỗ giải |
|---|---|---|
| **(a)** | AD-39 đòi bước **tách Chương** cho hình dạng đầu vào của story này; AC đòi **đúng một Chương** | §ĐỌC TRƯỚC TIÊN ③ + Task 4 |
| **(b)** | AD-4 + AD-37 đòi ranh giới segment và cờ kết đoạn tính **lúc nhập** và **lưu xuống**; AC không nhắc segment | Quyết định #4 |
| **(c)** | `global.db` mở **một lần** ở `setup()` và `app.manage`; `.atproj` có **N** Tác phẩm mở/đóng theo thao tác. Cách quản lý N `Store` trong state Tauri | Task 7 |

### 🔴 Khoảng trống atomic write — KHÔNG tài liệu nào yêu cầu, nên không AC nào bắt được

`mockups/data-integrity.html` hứa với người dùng: bản copy *"**không hỏng**, chỉ cũ hơn một câu đang gõ dở"*. Mệnh đề "không hỏng" là một lời hứa về **tính nguyên tử của lần ghi**. Nhưng:

- **PRD**: không NFR nào về atomic write / crash safety ngoài NFR18 *(chỉ nói **lượng mất**, không nói **tính nguyên tử**)*
- **ARCHITECTURE-SPINE**: không AD nào đặt luật ghi nguyên tử cho `meta.json`
- **addendum**: chỉ khuyến nghị *"SQLite… WAL, hàng đợi ghi tầng ứng dụng"* — WAL lo cho `project.db`, **không** lo cho `meta.json`

⇒ `meta.json` ghi bằng đường ghi thường. Sập máy giữa lúc ghi ⇒ metadata cắt cụt ⇒ **AC3 bị phá** *(Library không đọc được metadata mà không mở SQLite)*. Task 2 đóng nó bằng `tmp + rename`. **Tiền lệ cùng lớp lỗi đã ghi** trong `deferred-work.md`: *"Sao lưu bằng `fs::copy` không nguyên tử, không xác minh sau khi chép… tệp sao lưu trông hợp lệ mà thực ra thiếu."*

### 🔴 Mâu thuẫn tài liệu đã phát hiện — không dev KHÔNG sửa tài liệu, chỉ NÓI RA

**① `mockups/data-integrity.html:259-271` vẽ cây `.atproj` SAI HOÀN TOÀN**

| Nguồn | Cấu trúc |
|---|---|
| **AC story + AD-9** *(thắng)* | `meta.json` · `project.db` · `assets/` |
| `data-integrity.html` *(lỗi thời)* | `project.toml` · `chapters/0001.chapter` · `segments.db` · `history.db` · `glossary.db` · `tm.db` · `prompts/` · `images/` · `cover.jpg` |

Sai ở **cả sáu điểm**: định dạng metadata · số lượng DB *(5 vs 1)* · tên thư mục ảnh · thêm tệp phẳng `chapters/*.chapter`.

⚠️ Luật ưu tiên của `EXPERIENCE.md:312` *("khi mâu thuẫn, `DESIGN.md` và `EXPERIENCE.md` thắng")* **không cứu được ca này** — cả hai tệp đó **không nói gì** về bên trong `.atproj`. Trọng tài duy nhất là **AD-9**, và AD-9 khớp AC.

🔴 **Nguy hiểm xuôi dòng:** `EXPERIENCE.md` neo `data-integrity.html` vào **FR96–FR102**. Epic 5 sẽ đọc lại nó và dựng sai. ⇒ **Ghi thành một dòng trong §Câu hỏi cho Ice.**

**② Empty state Library quảng cáo `.docx` — thứ story này TỪ CHỐI**

`mockups/empty-states.html:107` in cho người dùng đọc, **nguyên văn**: `<b>Nhập từ file</b> — .txt, .docx, .md · một file lớn tách được thành nhiều Chương`. Nhưng AC8 đòi từ chối `.docx`, và vế *"một file lớn tách được thành nhiều Chương"* là **FR14 — Epic 6**, cũng chưa có. Đây là **quảng cáo một năng lực rồi từ chối nó ở bước sau** — đúng loại lỗi §Voice and Tone cấm. ⇒ Chuỗi `vi.json` của story này **không được copy nguyên**; hoặc gỡ `.docx` khỏi câu mời, hoặc đi kèm chỉ báo *chưa nhận ở phiên bản này*.

**③ 🔴 Chuỗi empty-state trong mockup SẼ LÀM ĐỎ CỔNG `check:i18n`**

`mockups/empty-states.html:105` viết **nguyên văn**: *"Một Tác phẩm là một dự án dịch, lưu thành một thư mục trên máy **bạn**. Copy thư mục đó đi đâu cũng mở lại được."*

`scripts/check-i18n.mjs:1156`: `const BANNED_WORDS = ['chúng tôi', 'bạn']`, so theo **biên tiếng** *(không phải substring)*, chuẩn hoá NFC. ⇒ Tiếng `bạn` trong `"trên máy bạn"` là **vi phạm Kiểm D**, cổng exit 1.

**Đường xử lý** *(không dev không sửa mockup)*: viết lại ở dạng **vô nhân xưng** khi đưa vào `vi.json` — ví dụ *"Một Tác phẩm là một dự án dịch, lưu thành một thư mục trên máy. Copy thư mục đó đi đâu cũng mở lại được."* Ghi độ lệch vào §Completion Notes.

**④ Bốn AC không có gốc trong PRD** — `meta.json`/`project.db`/`assets/` · đọc metadata không cần SQLite · UUID + số nguyên cục bộ · schema version. Cả bốn đến **duy nhất** từ ARCHITECTURE-SPINE *(AD-9, `:353-354`, AD-30)*. **Không phải lỗi** — đây là phân tầng đúng *(PRD nói **cái gì**, Architecture nói **hình dạng nào**)*. Ghi ra để không ai đi tìm gốc trong PRD rồi kết luận AC bịa ra.

**⑤ Đường "Dán văn bản" KHÔNG có mockup nào** trong cả 29 tệp. `library-and-import.html` chỉ dựng đường tệp + tách Chương *(là FR14/Epic 6)*. ⇒ Dev suy ra từ `.field`/`.dlg` của `library-and-import.html` + §Voice and Tone. **Nêu lại cho Sally như một khoảng trống.**

### Bàn giao — mười mục `deferred-work.md` gọi tên Story 1.15

| # | Mục | Loại |
|---|---|---|
| 1 | **Đường hiển thị lỗi kho chưa từng chạy trong webview thật** — cần một `$APPDATA` chỉ-đọc, là một bảng nghiệm thu tay. *"Giao lại Story 1.15 (story tiếp theo mở một kho **thứ hai**, tức story tiếp theo có lý do thật để chạy bảng đó)."* Ghi **hai lần** | 🔴 Nợ PHẢI trả — AC10(a) |
| 2 | **Tầng Tác phẩm của `ScopeResolver` chưa từng chạy trên dữ liệu thật.** *"`WorkScope` là một struct rỗng đánh dấu chỗ. **Story 1.15** cắm tầng thật vào; không ba chữ ký không phải đổi"* | 🔴 Nợ PHẢI trả — AC9 |
| 3 | **`Writer::shutdown()` không có trần cho `handle.join()`** — Ice chốt 2026-08-04: chấp nhận rủi ro *"với điều kiện **review tay mỗi khi một story mới ghi qua tầng này**"* | 🔴 Nợ PHẢI trả — AC10(b) |
| 4 | **`Checkpointer::shutdown()` có thể để luồng nền treo lửng** — *"chỉ trở thành rủi ro thật nếu một story sau thêm luồng **khởi động lại kho mà không thoát tiến trình**"* | 🔴 Story này làm **đúng** chuyện đó — Task 7 |
| 5 | **Sao lưu `fs::copy` không nguyên tử, không xác minh** — `project.db` thừa hưởng đúng đường này | ⚠️ Không sửa *(có chủ)*, phải **biết** |
| 6 | **Sáu số `Tuning` là TẠM, chưa cái nào được đo** *(chủ: Story 2.4)* — story này mở kho thứ hai ⇒ **luồng checkpoint thứ hai + 4 kết nối nữa** | ⚠️ Ghi ra, không tự hiệu chỉnh |
| 7 | **Lỗi checkpoint/backup đều gắn nhãn `StoreError::OpenFailed`** — nối chẩn đoán lên UI sẽ hiển thị nhầm *"Không mở được kho dữ liệu"* sau nhiều giờ chạy | ⚠️ Bẫy hiển thị |
| 8 | **`connect-src` của CSP KHÔNG có `asset:`** — *"story đầu tiên `fetch` một tài nguyên `$RESOURCE/**` từ webview sẽ chạy tốt suốt lúc phát triển rồi hỏng ở bản người dùng cài"* | 🔴 Bẫy chờ sẵn nếu định `fetch` ảnh trong `assets/` |
| 9 | **`applyPreset()` luôn `api.clear()` rồi dựng lại toàn bộ bốn panel** — *"sẽ mất trạng thái thật một khi panel có nội dung thật"* | ⚠️ Thành thật sớm hơn dự kiến nếu story này đổ văn bản vào Panel Source |
| 10 | **Bề mặt ĐỌC chưa tồn tại** — bề mặt render văn bản nguồn phải **tự khai `read-*`/`source-*`**, không kế thừa `ui-md` 1.5 | ⚠️ Task 10 |

### 🧠 Trí tuệ từ story trước — thứ đắt tiền, đừng học lại bằng tiền

1. **Ném TRƯỚC `mount()` = cửa sổ trắng.** `main.ts:107-157` đã có khối `try` + hộp chẩn đoán **vì chuyện đó đã xảy ra**. ⇒ Mọi lượt đọc `meta.json`/`project.db` từ frontend phải bọc.
2. **Thử → hỏng thì rơi về mặc định kèm chẩn đoán nêu đích danh → không chết** *(khuôn `bindingsAreUsable()`)*.
3. **`src/commands/**` phải nạp được bằng Node thuần** — luật *erasable-only*: không `enum`, không `namespace`, không parameter property, không `import` giá trị từ `vue`.
4. **Cờ ứng dụng tự giữ sẽ nói dối** — đọc trạng thái thật.
5. **`<KeepAlive>` giữ subtree thay vì tháo** ⇒ `onBeforeUnmount` **không chạy**; dùng `onDeactivated`. Story 1.14 bị review bắt đúng lỗi này *(`dockController` không gỡ ⇒ command toàn cục âm thầm ghi đè bố cục ở chế độ khác)*.
6. **Sàn cổng đếm tệp thì một tệp rỗng vẫn qua.** Nâng sàn **có comment ghi số thật**, không nâng cho vừa. ⚠️ **Hai quần thể KHÁC nhau** — `src-tauri/src/**` so với `src-tauri/**` sau miễn trừ `tests/**` — **chép số của tệp này sang tệp kia là đặt một cái sàn cho một cây khác**.
7. **Đỏ trước, xanh sau.** *"Một cổng chưa từng đỏ là một cổng chưa từng canh."*
8. **Dev KHÔNG sửa tài liệu quy hoạch.** Lệch thì **ghi ra**.
9. **Bất đối xứng nội bộ là thứ reviewer săn** — Story 1.14 bị bắt vì `applyPreset()` thiếu `try/catch` trong khi cùng tệp đã bọc ở `restore()`/`flush()`.
10. **Đừng đánh dấu task `[x]` khi mới làm 2/3 chỗ** — Story 1.14 bị bắt đúng chuyện này.
11. **Con số đúng vẫn có thể đi kèm mệnh đề sai.** Story 1.14 khẳng định *"khoảng cách giữa hai lượt ghi ≤ trần"* — cổng đỏ với `5008 ms`. Bất biến đúng là *"**tuổi của thay đổi chưa ghi**"*.

### ⚠️ Tám cái bẫy — sáu trong tám cho ra một lượt CI XANH với kết quả VÔ NGHĨA

| # | Bẫy | Cách chặn |
|---|---|---|
| 1 | 🔴 `pragma_update` **nuốt** hàng trả về của PRAGMA *(`rusqlite-0.40.1/src/lib.rs:555-560`)* ⇒ WAL không bật mà trả `Ok(())` | Đặt xong **`query_row` đọc lại và so** |
| 2 | 🔴 `wal_checkpoint` trả 3 cột; `execute_batch` ⇒ `Ok(())` **im lặng** vứt cả ba | `query_row`, xét cả `(busy, log, checkpointed)` |
| 3 | ⚠️ `wal_autocheckpoint`/`busy_timeout` là trạng thái **CỦA TỪNG KẾT NỐI** | Writer + mỗi kết nối pool + luồng checkpoint đều tự đặt |
| 4 | 🔴 `journal_mode = WAL` **ghi vào** database ⇒ AC7 trượt im lặng | Hợp đồng thứ tự 9 bước |
| 5 | 🔴 `fs::copy` tệp `.db` trần khi WAL bật = bản sao **THIẾU** thay đổi gần nhất | `wal_checkpoint(TRUNCATE)` → xác nhận `busy == 0` → rồi mới copy. ⚠️ feature `backup` của rusqlite **TẮT** |
| 6 | 🔴 `panic = "abort"` ⇒ `catch_unwind` **vô dụng**, và `Drop` **không chạy** khi panic | Không `unwrap()`/`expect()` nào trong `core::**`. Lỗi là **giá trị** |
| 7 | 🔴 **`INTEGER PRIMARY KEY` trần TÁI DÙNG rowid** sau khi xoá hàng cuối | `AUTOINCREMENT` — AD-3 nói id đã về hưu không bao giờ tái dùng |
| 8 | 🔴 **`String::from_utf8_lossy` nuốt lỗi bảng mã** và trả về `ç¬¬ä¸€ç«` một cách vui vẻ | Dùng `String::from_utf8` *(trả `Err`)*, **không** `_lossy` — Quyết định #6 |

### Testing standards

**Rust**: `cargo test --locked --manifest-path src-tauri/Cargo.toml`. Test integration ở `src-tauri/tests/`, **khai phạm vi ở dòng 1**, **một tệp một mối quan tâm**. Tên test = **câu mô tả hành vi**, snake_case, tiếng Anh, không tiền tố `test_`. `src-tauri/tests/**` **được miễn trừ Kiểm A** ⇒ assert message viết tiếng Việt có dấu **được**.

**Frontend**: **KHÔNG có bộ chạy test, và KHÔNG thêm** *(NFR15 — quyết định của Ice)*. Ba đường thay thế đã dùng ở năm story liền: ① **cổng `.mjs`** `import()` tệp `.ts` thuần bằng Node rồi **gọi hàm thật**; ② **chốt tự kêu lúc chạy** *(`console.error` nêu đích danh — nó **kêu**, không **vá**)*; ③ **nghiệm thu tay có bảng**, ghi rõ **engine + nền tảng + cái gì chưa đo**.

🔴 **Đỏ-rồi-xanh BẮT BUỘC cho mọi phép kiểm mới.** Tiền lệ: 1.4 *(28 ca)* · 1.5 *(16+23)* · 1.6 *(28)* · 1.7 *(17)* · 1.14 *(13+14+17 = 44)*.

⚠️ **`npm run build` PHẢI chạy trước `cargo test`** — `generate_context!` nhúng `frontendDist: "../dist"` lúc biên dịch.

### Project Structure Notes

**Cây nguồn chính thức** *(`ARCHITECTURE-SPINE.md:783-815`)* — story này **không** tạo thư mục nào ngoài nó:

```
src-tauri/src/
  commands/     # bề mặt IPC — adapter, KHÔNG chứa quy tắc nghiệp vụ   → project.rs (MỚI)
  core/
    library/    # chỉ mục + quét lại (AD-8)                              → atproj.rs, meta.rs (MỚI)
    segment/    # tách/gộp/về hưu + CHUỖI PIPELINE NHẬP (AD-39)          → import.rs (MỚI)
    scope/      # ScopeResolver (AD-18)                                  → SỬA (WorkScope)
    store/      # Writer nối tiếp + Reader pool + checkpoint (AD-11/12)  → SỬA (StoreSpec::project, PROJECT_MIGRATIONS)
    i18n/       # MessageKey + IpcError (AD-21)                          → SỬA (khoá mới)
  ports/        # DictionarySource · TranslationProvider · ProjectStore  → project_store.rs (MỚI)
src/
  modes/        # Library · Workspace · ReadingMode (AD-24)              → LibraryMode.vue SỬA
  commands/     # CommandRegistry (AD-34)                                → index.ts SỬA
  config/       # ⚠️ thư mục DUY NHẤT ngoài khai báo, có lý do viết ở đầu bootstrap.ts
  i18n/vi.json  # toàn bộ chuỗi giao diện (NFR16, AD-21)                 → SỬA
```

🔴 **AD-39 (`:498`) gán quyền sở hữu pipeline nhập:** *"Chuỗi này sống ở **`core/segment/`**… các module nguồn — `webimport/` cho URL, `export/` cho `.docx`, **đọc tệp thuần cho `.txt`/`.md`** — chỉ **cung cấp bước đầu vào** rồi trao lại; **không module nào giữ bản sao** của các bước dùng chung."* ⚠️ *"Đọc tệp thuần cho `.txt`/`.md`"* **không được gán một module** — story này đặt nó ở `core/segment/import.rs` và **ghi lý do tại chỗ**. **Không** đặt vào `webimport/` *(đó là **điểm ra mạng**)*, không vào `export/`.

**Quy ước module**: *"Một module cho **một khái niệm miền**, không phải cho một nhóm năng lực… C1–C10 là từ vựng sản phẩm, không xuất hiện trong tên module."* Tệp: Rust `snake_case`, Vue `PascalCase.vue`.

**README.md trong mỗi module là quy ước** — 9 tệp đang có. Thư mục mới nào ở `src/**` thì tạo README; `src-tauri/src/core/**` dùng **doc-comment `//!` ở đầu `mod.rs`** thay cho README *(xem `ports/mod.rs` — có cả bảng ba cổng với cột "Chủ")*.

**Cách document**: đây là repo có **doc-comment dài hơn mã**, và đó là kỷ luật có chủ ý. Mỗi tệp mở bằng khối nêu **story · AC · AD** sở hữu nó. Ký hiệu: 🔴 mệnh đề sống chết · ⚠️ bẫy/giới hạn · không cấm · ✅ mặc định đã chốt. Mọi lựa chọn khác thường **viết lý do NGAY TẠI CHỖ** kèm đường dẫn + số dòng. Mọi con số **tạm** phải tự khai là tạm và **nêu tên story sở hữu** việc đo lại. ⚠️ `deferred-work.md` đã ghi rằng **trích dẫn số dòng cứng sẽ rữa** — ghi kèm **tên hàm/hằng**, đừng chỉ ghi số dòng.

### 🌐 Phiên bản đang ghim — KHÔNG đổi một dòng nào

`tauri =2.11.5` *(feature `protocol-asset`)* · `tauri-build =2.6.3` · `serde =1.0.229` · `serde_json =1.0.151` · **`rusqlite =0.40.1`** *(feature `bundled`; ⚠️ `backup` và `hooks` **TẮT**)* · `libsqlite3-sys =0.38.1` *(SQLite **3.53.2**)* · `jieba-rs =0.10.3` · `tantivy-stemmers =0.4.0` · `docx-rs =0.4.22` *(**KHÔNG dùng ở story này**)* · `keyring =4.1.6` · `reqwest =0.13.4`
`vue 3.5.40` · `dockview-vue 7.0.4` · `typescript 5.9.3` · `vite 8.2.0` · `@vitejs/plugin-vue 6.0.8` · `vue-tsc 3.3.9` · `@tauri-apps/api 2.11.1` · `@tauri-apps/cli 2.11.4`
Rust `edition 2024`, `rust-version 1.85`, toolchain CI **1.97.1** · Node **22** · CI: `macos-26` + `windows-2025`, `fail-fast: false`

🔴 **Ghim bằng `=`, không bằng cú pháp mặc định của Cargo** — `"2.6.3"` trong Cargo NGHĨA LÀ `^2.6.3`, một dải rộng.

**Nghiên cứu phiên bản cho story này** *(2026-08-06)*:
- **`uuid 1.24.0`** — 🔴 **đã có trong `Cargo.lock:5034`** *(bắc cầu qua `tauri`)*. Giấy phép `MIT OR Apache-2.0` ⇒ tương thích GPL v3. Feature `v4` cần `getrandom`, mà `getrandom 0.4.3` **cũng đã có trong lock**. ⇒ Khai tường minh **không thêm một byte payload nào**. ⚠️ Vẫn phải rà NFR15 theo phương pháp *"mở tệp LICENSE trong nguồn **đã tải** mà đọc"*.
- **`tauri-plugin-dialog`** — bản mới nhất `2.5.0`. 🔴 **KHÔNG LIÊN QUAN — crate này bị `BANNED_CRATES` cấm.** Ghi ra chỉ để dev không đi tra rồi tưởng nó là đường hợp lệ.
- **`PRAGMA user_version`** — vẫn là cách nhẹ nhất để đánh phiên bản lược đồ SQLite *(một số nguyên ở offset cố định trong tệp, không phải một bảng)*. Thực hành chuẩn: bọc mỗi bước trong `BEGIN … COMMIT` để `user_version` không bị nâng khi DDL trượt. 🔴 **`core/store/schema.rs` ĐÃ cài đúng như vậy** — Task 1 chỉ khai `PROJECT_MIGRATIONS`, **không viết lại cơ chế**. **Không** thêm `rusqlite_migration` — đó là một phụ thuộc mới cho một thứ đã có.

### 📌 Bối cảnh git

✅ Cây làm việc **sạch** tại `c3efb204d76239785f8ee27014be392126ba9c35`.

Năm commit gần nhất và thứ chúng để lại cho story này:

| Commit | Để lại gì |
|---|---|
| `c3efb20` *feat: Implement AI Translation, Editor, Lookup, and Source panels; refactor PanelFrame and PanelTab for dockview integration* | Bốn panel là **khung rỗng có chủ ý**. `PanelFrame` nhận **đúng hai** prop `owner`/`statusKey` *(**không còn `titleKey`**)*. `SourcePanel.vue:5` ghi: nội dung thật là **Story 1.16** |
| `7e38de8` *Add behavioral tests for core::matching* | Khuôn test hành vi + tên test dạng câu |
| `5edbe0e` *Add assertion for StoreKind::Dict in store_kind_names_are_stable_machine_identifiers* | 🔴 **`"project"` đã bị khoá làm định danh trên dây** — đừng đổi |
| `5a68df7` *Update deferred work documentation and sprint status; enhance build process* | `deferred-work.md` ở dạng hôm nay — 10 mục gọi tên story này |
| `dd7af61` *Add VIWIKTIONARY_EN source…* | Khuôn thêm một nguồn + test đi kèm cùng lượt |

**Đọc ra được:** repo commit **theo story**, message tiếng Anh dạng mệnh lệnh, và **test đi cùng lượt với mã** — không có commit "thêm test sau".

### References

**Yêu cầu**
- `_bmad-output/planning-artifacts/epics.md:1626-1669` — §Story 1.15, AC nguyên văn
- `_bmad-output/planning-artifacts/epics.md:801-820` — Epic 1 objectives + ghi chú cài đặt *(đặc biệt `:819` — nhánh `.docx` đóng ở Epic 6)*
- `_bmad-output/planning-artifacts/epics.md:1922-2045` — Story 2.1 + 2.3 *(ràng buộc xuôi dòng: `segment.id`, `ord`, cờ kết đoạn, `store::Writer`)*
- `_bmad-output/planning-artifacts/epics.md:3327-3408` — Story 5.1 + 5.2 *(`.atproj` ghi **trước**, chỉ mục ghi **sau**)*
- `prds/prd-AuraTranslate-2026-08-02/prd.md` §6.1 *(FR2, FR3, FR5, FR13)* · §6.9 *(FR96, FR97, FR98, FR100–FR103)* · §7.2 *(NFR9, NFR10)* · §7.5 *(NFR17, NFR18)*

**Kiến trúc**
- `architecture/architecture-AuraTranslate-2026-08-02/ARCHITECTURE-SPINE.md` — **AD-9** `:139-145` · **AD-30** `:362-366` · **AD-33** `:400-404` · AD-7 `:119-131` · AD-11 `:153-157` · AD-12 `:159-163` · AD-18 `:250-276` · AD-21 `:302-306` · AD-23 `:314-320` · AD-39 `:475-500` · Consistency Conventions `:639-658` · Structural Seed `:751-815` · Stack `:693-710`
- `.../reviews/review-adversarial-2026-08-03b.md` §F3 *(vá vào AD-39 — chuỗi pipeline sống ở `core/segment/`)* · §F5 *(vá vào AD-39 — `.docx` **bỏ qua** bước giải mã bảng mã)*

**UX**
- `ux-designs/ux-AuraTranslate-2026-08-02/EXPERIENCE.md` §Trạng thái rỗng · §Component Patterns *(màn xem trước hợp nhất, 2026-08-03)* · §Voice and Tone · §KF-1
- `.../DESIGN.md` §Typography *(`read` vs `ui` — ranh giới không được nhoè)* · §Sàn tương phản · §Bảng token màu
- `.../mockups/empty-states.html` — ô "Library lần đầu" *(⚠️ chuỗi vi phạm Kiểm D)*
- `.../mockups/library-and-import.html` — `.dlg`, `.field`, `.btn.pri`, `.hint` *"Chưa có gì được ghi xuống đĩa"*
- `.../mockups/data-integrity.html` — 🔴 **cây `.atproj` LỖI THỜI, không dựng theo**

**Mã**
- `src-tauri/src/core/store/{mod,schema,pragmas,writer,reader,checkpoint}.rs` — tầng ghi, không **dùng, không viết lại**
- `src-tauri/src/core/i18n/mod.rs` — `message_keys!`, `IpcError`
- `src-tauri/src/commands/{mod,config}.rs` — khuôn IPC hai lớp + `wire`
- `src-tauri/src/core/scope/mod.rs:167,181` — `WorkScope`, hàm dựng thứ hai
- `src-tauri/src/ports/{mod,dict_source}.rs` — bảng ba cổng + mẫu chuẩn
- `src-tauri/src/lib.rs:44-59,111-143` — `generate_handler!`, `open_global_store`, `RunEvent::Exit`
- `src/config/bootstrap.ts` — adapter IPC **duy nhất**, khuôn không-bao-giờ-ném
- `scripts/check-deps.mjs:137-163` — `BANNED_CRATES` / `BANNED_NPM`
- `scripts/check-i18n.mjs:1145-1201` — Kiểm D, `BANNED_WORDS`
- `src-tauri/SECURITY-NOTES.md:57-82` — ba sự thật về capabilities, và vì sao plugin bị cấm

**Story trước**
- `1-7-tang-ghi-du-lieu-mot-writer-noi-tiep-va-luoc-do-co-phien-ban.md` — API, tám cái bẫy, bốn luật test
- `1-14-khung-bon-panel.md` §Trí tuệ từ story trước · §Testing standards · §Review Findings
- `1-2-scaffold-du-an-va-khoa-pham-vi-filesystem-pham-vi-mang.md:426-430,671-674` — ba lớp cưỡng chế scope
- `deferred-work.md` — 10 mục gọi tên story này

### Câu hỏi cho Ice

**✅ Sáu quyết định đã có chữ ký 2026-08-06 — không hỏi lại.** Còn lại **ba** mục, **không mục nào chặn** *(cả ba nằm ở tầng tài liệu quy hoạch, mà dev không được sửa)*:

1. 🔴 **`mockups/data-integrity.html` vẽ sai cây `.atproj`** — cần một lượt sửa của Ice. `EXPERIENCE.md` neo tệp đó vào **FR96–FR102**, nên **Epic 5 sẽ đọc lại nó và dựng sai** nếu để nguyên. Story này không dựng theo nó, và đã ghi cảnh báo tại chỗ.
2. **`mockups/empty-states.html`** — hai lỗi trong cùng một ô: quảng cáo `.docx` *(story này từ chối)* và vế *"một file lớn tách được thành nhiều Chương"* *(FR14 — Epic 6)*; **và** tiếng `bạn` ở dòng 105 **làm đỏ Kiểm D** của `check:i18n`. Story này sẽ viết lại ở dạng vô nhân xưng khi đưa vào `vi.json` và ghi độ lệch vào §Completion Notes — nhưng mockup vẫn cần một lượt sửa của Ice.
3. **Đường "Dán văn bản" không có mockup nào** trong cả 29 tệp — nêu cho Sally như một khoảng trống, cùng hạng với *"kiểm Panel Lookup cho mục tiếng Anh"* đang treo trong sprint change proposal. ⚠️ Nay khoảng trống rộng thêm: Quyết định #1 thêm **ô nhập đường dẫn** và **vùng kéo-thả**, cả hai cũng chưa có spec thị giác.

**Một mục để Ice biết, không cần trả lời:** phán quyết #1 làm **NFR17 thủng ở nhánh mở tệp** *(kéo-thả là chuột thuần)*. Story vá bằng **ô nhập đường dẫn**, 0 phụ thuộc 0 permission. Nếu Ice thấy ô đó thừa thì nói — nhưng khi đó nhánh mở tệp sẽ **không có đường bàn phím nào**, và đó là một độ lệch NFR17 phải ghi thành nợ.

---

## Dev Agent Record

### Agent Model Used

Claude Sonnet 5 (claude-sonnet-5), qua Claude Code.

### Debug Log References

**Task 0 — đường cơ sở, `c3efb20`, macOS, 2026-08-06:**

| Lệnh | Kết quả |
|---|---|
| `npm run check:deps` | PASS — 326 crate Rust quét, 104 gói npm |
| `npm run check:tokens` | PASS — 32 tệp, 209 khai báo CSS |
| `npm run check:i18n` | PASS — 28 khoá |
| `npm run check:commands` | PASS — 3 `dispatch()`, 29 tệp |
| `npm run check:layout` | PASS — 29 tệp, 10 thành viên global |
| `npm run build` | PASS — 462ms |
| `cargo test --locked` | PASS — **165 passed**, 0 failed (khớp số đã ghi ở §Dev Notes) |
| `npm run check:scope` | PASS — dev-no-csp, cả hai chiều |
| `npm run check:scope:bundled` | PASS — bundled-csp qua `font-src`; chiều âm không đo được ở bundled (đã biết, ghi trong chính output của cổng) |

**Mũi thăm dò kéo-thả (Quyết định #1(b)) — VERIFY BẰNG ĐỌC MÃ NGUỒN, KHÔNG BẰNG KÉO-THẢ TAY THẬT:**

Môi trường chạy phiên này là một agent CLI **không có** công cụ điều khiển GUI desktop (chỉ có
Chrome-tab automation, không điều khiển được cửa sổ native Tauri; không có `cliclick` hay
tương đương để phát sinh sự kiện chuột HID thật; `osascript`/System Events có truy cập một
phần nhưng UI scripting một thao tác kéo-thả thật giữa hai tiến trình là không khả thi trong
phiên này). ⇒ **Kéo-thả tay thật KHÔNG chạy được** trong lượt này — ghi thẳng thay vì giả vờ.

Thay vào đó, xác minh bằng đọc **mã nguồn thật** của `tauri-runtime-2.11.3` và `tauri-2.11.5`
(hai crate đã ghim, cùng phiên bản compile thật của cây):

- `tauri-runtime-2.11.3/src/window.rs:60,91,97` — `WindowEvent::DragDrop(DragDropEvent)` và
  `WebviewEvent::DragDrop(DragDropEvent)` là **sự kiện cửa sổ/webview gốc**, cùng họ với
  `Focused`/`ThemeChanged`/`Destroyed`.
- `tauri-2.11.5/src/window/mod.rs:1179-1184` — `Window::on_window_event()` đăng ký một
  callback **thẳng trên dispatcher runtime**, hoàn toàn KHÔNG đi qua hệ thống
  invoke/ACL/capabilities. Đây không phải một `#[tauri::command]`, không phải một event
  `emit`/`listen` phía JS — nó là một API Rust nội bộ nhận sự kiện native TRƯỚC khi có bất
  kỳ khái niệm "quyền" nào can thiệp.
- ⇒ **Kết luận: nhận đường dẫn kéo-thả qua `on_window_event` cần ĐÚNG 0 permission**, không
  phải "3 permission hiện có có đủ không" — mọi permission hiện có (hay không có cái nào)
  đều không liên quan tới đường này. Câu trong AC "Tauri v2 phát `tauri://drag-drop` qua
  event system, mà `core:event:default` đã được cấp" mô tả đúng đường **forward sự kiện đó
  ra JS qua `emit`** (đường đó mới cần `core:event:default` — để JS *nghe* được) — nhưng
  Task 4/Task 10 của story này chọn đường Rust nhận trực tiếp qua `on_window_event`,
  **không** forward qua JS, nên ngay cả `core:event:default` cũng không cần cho mục
  đích này (nó vẫn được giữ vì mục đích khác đã có từ Story 1.2).
- `tauri-utils-2.9.3/src/config.rs:1947,2301` — `WindowConfig::drag_drop_enabled: bool`
  mặc định `true`. `grep -n "dragDrop" src-tauri/tauri.conf.json` ⇒ **0 kết quả** — cấu hình
  hôm nay không override, nên mặc định `true` áp dụng. ⇒ **Không cần chạm `tauri.conf.json`**
  cho Quyết định #1(b) (khác với lo ngại nêu trong story — thực tế không cần).

**VERDICT: PASS**, với ghi chú giới hạn tường minh: đây là **verify bằng đọc mã nguồn của
chính crate đã ghim đang chạy trên cây này** (không phải suy đoán từ tài liệu bên ngoài), và
nó xác nhận một kết luận **mạnh hơn** giả thiết ban đầu của story (0 permission cần, không
phải "3 permission đã đủ"). **Vẫn còn một khoảng trống thật, ghi ra không giấu:** một lượt
kéo một tệp thật bằng chuột người vào cửa sổ app thật **chưa từng chạy** trong phiên triển
khai này. Đây là **nợ nghiệm thu tay mới**, cùng lớp với hai món nợ AC10 đã có sẵn — thêm vào
`deferred-work.md` ở Task 12: *"Kéo-thả tệp thật (Quyết định #1 Story 1.15) mới được verify
bằng đọc mã nguồn `tauri-runtime`/`tauri`, chưa bằng một lượt kéo tay thật trên máy có GUI.
Rủi ro thấp — cơ chế `on_window_event`/`WindowEvent::DragDrop` là API ổn định, dùng rộng rãi
trong hệ sinh thái Tauri, không đi qua bất kỳ lớp quyền tự viết nào của dự án — nhưng câu
này chưa có bằng chứng thực nghiệm trên máy này."*

**Rà NFR15 cho `uuid = "=1.24.0"`:**

- Đã tải sẵn tại `~/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/uuid-1.24.0/`
  (phụ thuộc bắc cầu có sẵn qua `tauri`, xác nhận `Cargo.lock:5034-5037`).
- Mở và đọc **trong nguồn đã tải** (không tin nhãn registry):
  - `LICENSE-MIT` — văn bản MIT chuẩn, đồng bản quyền The Rust Project Developers +
    Ashley Mannix, Christopher Armstrong, Dylan DPC, Hunar Roop Kahlon.
  - `LICENSE-APACHE` — văn bản Apache License 2.0 chuẩn.
- `Cargo.toml.orig` của crate khai `license = "MIT OR Apache-2.0"`, khớp hai tệp license đọc
  được. Cả hai đều **permissive**, tương thích GPLv3 (dự án dùng cho sản phẩm đóng gói, không
  copyleft ngược).
- Khai tường minh `uuid = { version = "=1.24.0", features = ["v4"] }` **không thêm byte
  payload nào** — `getrandom 0.4.3` (phụ thuộc của feature `v4`) cũng đã có sẵn trong
  `Cargo.lock` qua cùng đường bắc cầu.
- **Đề nghị hàng thêm vào bảng Stack của `ARCHITECTURE-SPINE.md`** (dev không tự sửa tệp
  `planning-artifacts/`, chỉ soạn và đề nghị Ice):
  `| uuid | =1.24.0 | MIT OR Apache-2.0 | sinh work.id UUID v4 (AD-28) | đã có bắc cầu qua tauri, khai tường minh Story 1.15 |`

**Bảng đỏ-rồi-xanh — `tests/project_contract.rs` (12 ca mới):**

Mười một ca viết SAU khi cài đặt (không phải TDD đỏ-trước theo nghĩa đen), nhưng mỗi ca được
kiểm tra bằng cách **chứng minh nó bắt được lỗi thật** trước khi coi là đủ tin cậy. Một lượt
đột biến thật đã chạy: đổi `CHAPTER_DDL`'s `id INTEGER PRIMARY KEY AUTOINCREMENT` thành
`INTEGER PRIMARY KEY` trần (Bẫy 7) ⇒ `a_retired_chapter_id_is_never_handed_out_again` **ĐỎ**
(`left: 1, right: 1` — id vừa xoá bị phát lại), khôi phục ⇒ **XANH**. Mười ca còn lại được xây
theo cùng nguyên tắc "nó phải fail nếu tôi cố tình phá đúng bất biến nó tuyên bố canh":
`a_docx_is_refused_before_a_single_byte_is_written`/`a_failed_import_leaves_no_half_built_folder_behind`
kiểm sự VẮNG MẶT của thư mục (không phải một khẳng định xuông); `a_copied_project_folder_opens_at_a_different_path`
mở lại ở một `Store::open` hoàn toàn mới trên đường dẫn khác; `meta_json_is_readable_without_ever_touching_the_database`
XOÁ `project.db` trước khi đọc.

| Ca | Bẫy/AC nó canh | Đã ép đỏ? |
|---|---|---|
| `a_retired_chapter_id_is_never_handed_out_again` | Bẫy 7 (AUTOINCREMENT) | ✅ Đã ép đỏ thật |
| `creating_a_work_lays_down_exactly_three_things_on_disk` | AC2 | Đối chứng cấu trúc (không đột biến) |
| `meta_json_is_readable_without_ever_touching_the_database` | AC3 | Đối chứng hành vi (xoá `project.db`) |
| `the_work_id_is_a_v4_uuid_with_the_right_version_and_variant_bits` | AC4 | Đối chứng cấu trúc |
| `a_copied_project_folder_opens_at_a_different_path` | AC5 | Đối chứng hành vi (copy + mở lại) |
| `meta_json_can_be_rebuilt_from_the_database_alone` | Quyết định #3 | Đối chứng hành vi (xoá `meta.json`) |
| `a_newer_meta_schema_is_refused_without_touching_a_single_byte` | AC7 | Đối chứng hành vi |
| `a_docx_is_refused_before_a_single_byte_is_written` | AC8 | Đối chứng hành vi (kiểm vắng mặt thư mục) |
| `a_failed_import_leaves_no_half_built_folder_behind` | AC8 | Đối chứng hành vi (dựng lỗi giữa chừng thật) |
| `text_that_is_not_utf8_is_refused_the_same_way_a_docx_is` | Quyết định #6, Bẫy 8 | Đối chứng hành vi |
| `pasted_text_and_a_read_file_travel_the_same_import_path` | AC1, AD-39 | Đối chứng cấu trúc |
| `a_docx_rejection_carries_the_dedicated_message_key` | AC8 (bổ sung) | Đối chứng cấu trúc |

**Lượt review tay job ghi mới (điều kiện Ice 2026-08-04, `writer.rs:159`):**

`commands::project::create_work`'s `store.write(move |tx| {...})` — soát trực tiếp mã nguồn:
- Không I/O nào bên trong closure (không `fs::`, không `std::net`).
- Không gọi ra ngoài (`reqwest` không xuất hiện).
- Không `Store::write` lồng nhau.
- SQL là **hai** câu hằng (`&'static str` literal), tham số ràng buộc qua `rusqlite::params`
  (không `format!` chèn dữ liệu người dùng vào SQL).
- `meta.json` (I/O thật) chạy **SAU** khi closure đã trả `Ok` và giao dịch đã commit — đúng
  Quyết định #3, ở tầng thao tác của `create_work`, không phải tầng giao dịch.

Kết luận: giả định *"job ghi không chặn, không I/O"* mà `Writer::shutdown()` (`writer.rs:159`)
dựa vào vẫn đúng sau story này.

### Completion Notes List

**Sáu quyết định đã chốt — chép lại kèm lý do cho người đọc sau (không mở lại):**

1. **Đường vào (b) kéo-thả + (a) dán văn bản.** Lý do: `tauri-plugin-dialog` bị cấm cứng bởi
   `scripts/check-deps.mjs::BANNED_CRATES`, và `rfd` (đường (c)) đòi một lượt rà NFR15 mở
   bảng Stack mà Ice chưa mở cho story này. Kéo-thả dùng event system gốc của Tauri
   (`on_window_event`), 0 phụ thuộc mới. Vá NFR17 bắt buộc: ô nhập đường dẫn đi kèm (đường
   bàn phím cho nhánh "mở tệp", vì kéo-thả là thao tác chuột thuần).
2. **`uuid = "=1.24.0"` khai tường minh.** Đã có bắc cầu qua `tauri`, 0 byte thêm, MIT OR
   Apache-2.0 — xem rà NFR15 ở Debug Log References.
3. **`meta.json` ghi NGAY SAU khi giao dịch `project.db` commit, nguyên tử `tmp+rename`.**
   Lý do: `Store::write` nhận một closure chỉ được phép SQL thuần (điều kiện review tay của
   Ice 2026-08-04); nhét `fs::write` vào trong closure phá điều kiện đó và có thể treo cả
   hàng đợi ghi nếu đĩa treo. "Cùng thao tác logic" của AD-33 giữ ở tầng THAO TÁC, không phải
   tầng giao dịch SQL.
4. **KHÔNG tạo bảng/dòng `segment`.** AD-4 đóng băng ranh giới segment tính một lần lúc nhập;
   cài một bộ tách "tạm" ở story này là đóng băng vĩnh viễn ranh giới sai. Nợ mở:
   `segment_count = 0` cho mọi Chương nhập ở Epic 1, Story 2.1 phải xử lý bằng thao tác tách
   tường minh.
5. **`~/Documents/AuraTranslate/` qua `app.path()`.** Scope động của AD-23 hôm nay được cưỡng
   chế bằng **kỷ luật mã Rust** (không gọi `std::fs`/`rusqlite::Connection::open` ngoài đường
   đã định), **không phải bởi framework** — capabilities chỉ canh bề mặt IPC (webview),
   không canh Rust. Nghiệm thu bằng vắng mặt bề mặt, đúng cách Story 1.2 đã phát biểu về
   `$APPDATA/**`.
6. **CHỈ nhận UTF-8, từ chối tường minh thứ khác.** AD-4 đóng băng ranh giới segment tính lúc
   nhập ⇒ văn bản giải mã sai ghi xuống hôm nay là dữ liệu Epic 6 không sửa lại được. Dùng
   `String::from_utf8` (trả `Err`), không `_lossy`.

**🔴 CHỖ TÔI LÀM KHÁC STORY, và vì sao:**

1. **`PROJECT_MIGRATIONS` là BA bước, không MỘT bước gồm cả ba DDL như văn bản story gợi ý**
   (`SCHEMA_MIGRATION_LOG_DDL` bước 1, `WORK_DDL` bước 2, `CHAPTER_DDL` bước 3). Lý do kỹ
   thuật: `concat!` (cách duy nhất nối chuỗi ở compile-time không thêm phụ thuộc) chỉ nhận
   **literal**, không nhận một `const` đặt tên — nối `SCHEMA_MIGRATION_LOG_DDL` (hằng phải
   **tái dùng**, không viết lại) vào một chuỗi duy nhất buộc phải chép lại nguyên văn của nó. Ba
   bước tách rời giữ mỗi DDL có đúng một nguồn sự thật, cùng khuôn `GLOBAL_MIGRATIONS` đã tách
   `SCHEMA_MIGRATION_LOG_DDL` khỏi `CONFIG_VALUE_DDL`. Không AC nào đòi work+chapter phải cùng
   một giao dịch SQL với nhật ký di trú.
2. **Mũi thăm dò kéo-thả (Task 0, Quyết định #1(b)) được verify bằng ĐỌC MÃ NGUỒN, không
   bằng một lượt kéo tay thật.** Môi trường triển khai (agent CLI) không có công cụ điều khiển
   GUI desktop. Kết luận thu được (0 permission cần, không phải "3 đã đủ") mạnh hơn giả thiết
   ban đầu và có cơ sở vững (đọc trực tiếp `tauri-runtime`/`tauri` đã ghim), nhưng đây vẫn là
   một hình thức verify khác với những gì story yêu cầu theo nghĩa đen. Xem §Debug Log
   References và `deferred-work.md`.
3. **Bảng nghiệm thu tay đường hiển thị lỗi kho trong webview thật (AC10a) KHÔNG chạy được**
   — cùng lý do #2: không có công cụ dựng/đọc một cửa sổ Tauri thật trong phiên này. Đây là
   **nợ CHƯA đóng**, không phải một hạng mục bị bỏ qua có chủ ý — xem checklist Task 12 và
   `deferred-work.md` §1-15.
4. **`ScopeResolver::with_work` được cắm vào đường sản phẩm nhưng chưa có method phân giải nào
   thực sự CHẠY với dữ liệu tầng Work** (AC9). Không có bảng dữ liệu tầng Work nào tồn tại ở
   Story 1.15 để tra (Glossary/TM/Prompt là các epic sau) — AC9 đòi "cắm tầng thật vào", và
   `WorkScope{work_id}` + hàm dựng thứ hai là tầng thật đó, nhưng "phân giải trên dữ liệu Work"
   vẫn chờ consumer đầu tiên (Epic 3+). Ghi rõ trong `deferred-work.md` để không ai đọc AC9
   thành "đã có phân giải hai tầng chạy thật".
5. **Genre/nguồn ngữ được thu qua một form tối thiểu tự suy ra**, không theo một mockup đã
   duyệt — không mockup nào trong 29 tệp vẽ đường "dán văn bản"/ô nhập đường dẫn/vùng kéo-thả
   (đã nêu ở §Mâu thuẫn tài liệu ⑤ của story). Form dùng token hệ thống (`ui-sm`, `ui-md`,
   `surface-sunken`, …), không chuỗi/màu viết thẳng — qua được `check:tokens` — nhưng bố cục
   thị giác cụ thể (khoảng cách, thứ tự trường) là suy luận của dev, không phải một thiết kế
   đã ký.

**NHỮNG GÌ STORY NÀY CỐ Ý KHÔNG LÀM** (ngoài những gì §ĐỌC TRƯỚC TIÊN đã liệt — `.docx`,
tách segment, tách Chương, `library-index.db`, lưới Tác phẩm):
- Không màn hình "mở lại một `.atproj` đã có" — chỉ **tạo mới** có IPC command; mở lại được
  kiểm chứng ở tầng `Store::open`/test (`a_copied_project_folder_opens_at_a_different_path`),
  không có UI cho luồng đó (Epic 5 sở hữu lưới + mở từ danh sách).
- Không giới hạn độ dài `name`/`genre` trước khi ghi — cùng mức tin cậy với `save_value` của
  Story 1.8 (dữ liệu đến từ chính ứng dụng, không phải một biên tin cậy với dữ liệu ngoài).
- Không validate tên Tác phẩm rỗng ở tầng giao diện — rơi về `"Untitled"` ở tầng sanitize
  (`core::library::atproj::sanitize_name`), xem `deferred-work.md`.

**Định nghĩa hoàn thành (Task 13):** `check:deps` · `check:tokens` · `check:i18n` ·
`check:commands` · `check:layout` · `build` · `cargo test --locked` (177 passed = 165 nền +
12 mới) · `check:scope` · `check:scope:bundled` — **cả chín đều PASS**, xem §Debug Log
References cho số liệu đầy đủ trước/sau.

### File List

**Mới:**
- `src-tauri/src/core/library/atproj.rs` — hình dạng `.atproj/` trên đĩa (AC2, AC5, AC6)
- `src-tauri/src/core/library/meta.rs` — `WorkMeta`, ghi nguyên tử, dựng lại từ `project.db`
- `src-tauri/src/core/segment/import.rs` — pipeline nhập tối thiểu, một hàm thuần (AC1, AC8)
- `src-tauri/src/ports/project_store.rs` — cổng thứ ba của AD-2 (khai hình dạng, chưa cài đặt)
- `src-tauri/src/commands/project.rs` — bề mặt IPC tạo Tác phẩm (AC1, AC8)
- `src-tauri/tests/project_contract.rs` — 12 ca hành vi (11 bắt buộc + 1 bổ sung)
- `src/config/project.ts` — adapter IPC phía webview, khuôn `bootstrap.ts`
- `src/modes/libraryImport.ts` — state + thao tác form nhập Tác phẩm, tiêm vào `CommandDeps`

**Sửa:**
- `src-tauri/Cargo.toml` / `Cargo.lock` — thêm `uuid = "=1.24.0"` (0 crate mới, đã có bắc cầu)
- `src-tauri/src/core/store/schema.rs` — `WORK_DDL`, `CHAPTER_DDL`, `PROJECT_MIGRATIONS`
- `src-tauri/src/core/store/mod.rs` — `StoreSpec::project`, cập nhật doc-comment `StoreKind`
- `src-tauri/src/core/library/mod.rs` — `ProjectError`, re-export `atproj`/`meta`
- `src-tauri/src/core/segment/mod.rs` — đăng ký `pub mod import`
- `src-tauri/src/core/scope/mod.rs` — `WorkScope{work_id}`, `ScopeResolver::with_work`
- `src-tauri/src/core/i18n/mod.rs` — bốn `MessageKey` mới (`Import*`, `Project*`)
- `src-tauri/src/ports/mod.rs` — đăng ký `project_store`, cập nhật bảng ba cổng
- `src-tauri/src/commands/mod.rs` — đăng ký `pub mod project`
- `src-tauri/src/lib.rs` — `open_work_slot`/`close_open_work` (Task 7), `wire_drag_drop` (Quyết định #1b), đăng ký hai command mới
- `src-tauri/tests/scope_contract.rs` — ghi chú khoanh vùng `every_command_error_comes_from_the_store_vocabulary`
- `src/i18n/vi.json` — 4 khoá `err.*` + 12 khoá `command.*`/`mode.library.*`
- `src/commands/index.ts` — `CommandDeps.submitPastedText`/`submitFilePath`, hai command `library.import_*`
- `src/main.ts` — nối `submitPastedText`/`submitFilePath` vào `installCommands`
- `src/modes/LibraryMode.vue` — form nhập + trạng thái rỗng thay thế

**Tài liệu:**
- `_bmad-output/implementation-artifacts/deferred-work.md` — đóng một phần mục AC9/Story 1.8,
  đổi trạng thái `Checkpointer::shutdown()` từ "vô hại" sang "rủi ro thật", mở **13 mục** dưới
  hai đề mục `1-15` *(7 mục ở lượt triển khai + 6 mục ở lượt code review 2026-08-06)*
- `_bmad-output/implementation-artifacts/sprint-status.yaml` — story chuyển `in-progress`

⚠️ **Ba con số ở §File List/Task 12 trước lượt code review đã SAI, sửa lại tại chỗ** *(bản
gốc ghi "năm mục mới" ở Task 12 và "sáu mục mới" ở đây, trong khi §1-15 thật có **bảy**; và
ghi `vi.json` thêm 16 khoá trong khi thật là **18**)*. Số đúng sau cả hai lượt: `vi.json` có
**48 khoá** *(nền 28 + 18 ở lượt triển khai + 3 thêm/1 gỡ ở lượt review)*,
`tests/project_contract.rs` có **18 ca**, `cargo test --locked` = **184 passed**.

**Không đụng**: `tools/**` · `core/dict/**` · `core/matching/**` · `dict-manifest.toml` ·
`_bmad-output/planning-artifacts/**` · `src-tauri/capabilities/main.json` · `package.json` ·
ba chữ ký phân giải của `ScopeResolver` · `StoreKind::Project` (định danh trên dây `"project"`
giữ nguyên).

### Review Findings

**Lượt review 2026-08-06** — ba lớp song song (Blind Hunter · Edge Case Hunter · Acceptance Auditor) + đối chứng tay.
Đối chứng bảng số của story: **chín cổng DoD PASS thật**, `cargo test --locked` = **177 passed / 0 failed** (15+11+36+23+5+8+24+12+5+18+4+16) — §Debug Log References **trung thực**, không phóng đại.
Năm cấm của §ĐỌC TRƯỚC TIÊN giữ nguyên (0 dòng `docx_rs` được gọi · 0 bảng/dòng `segment` · 0 tách Chương · 0 `library-index.db` · 0 lưới Tác phẩm). Quyết định #3/#4/#6 cài đúng. Bẫy 7 (`AUTOINCREMENT`) và bẫy 8 (`from_utf8_lossy`) đều đã chặn. Không tệp KHÔNG ĐỤNG nào bị sửa; ba chữ ký `ScopeResolver` và định danh trên dây `"project"` **byte-identical**.

**Quyết định cần Ice chốt**

- [x] [Review][Decision] **Chính sách trùng tên Tác phẩm** — `sanitize_name` cho hai tên khác nhau ra cùng một thư mục (`A/B` và `A_B` → `A_B.atproj`), và tên rỗng luôn ra `Untitled.atproj`. Cần chốt: **từ chối bằng một lỗi riêng** (`err.project.name_taken`) hay **tự đánh số** (`Tên (2).atproj`)? Vế an toàn (không bao giờ xoá thư mục mình không tạo) là Patch bên dưới và không chờ quyết định này.
- [x] [Review][Decision] **AC7 nửa `meta.json` là mã chết** — `WorkMeta::read` (`core/library/meta.rs:105`) **không có một chỗ gọi nào** trong `src-tauri/src/**`, chỉ `tests/project_contract.rs` gọi. Kéo theo `ProjectError::MetaTooNew`, `MessageKey::ProjectMetaTooNew` và khoá `err.project.meta_too_new` trong `vi.json` **không đường sản phẩm nào chạm tới** — vì không có màn hình "mở lại một `.atproj` đã có" (§Completion Notes tự khai, nhưng không nối vào AC7). Điều này ngược đúng luật story tự trích: *"Không khoá nào cho tính năng chưa tồn tại"* (Story 1.7 §CN #3). Chốt: coi AC7 **đạt một nửa** và mở nợ, hay gỡ khoá/lỗi cho tới khi Epic 5 có đường mở?
- [x] [Review][Decision] **AC9 tầng Work chỉ GHI, không ai ĐỌC** — `OpenWork.scope` dựng ở `commands/project.rs:150` và **không được đọc ở đâu cả**; `OpenWorkState` chỉ có hai chỗ chạm (`lib.rs:275` manage, `:288` đóng lúc thoát). Đường phân giải sản phẩm thật vẫn là `core/scope/store.rs:161` — `ScopeResolver::global_only()`, vẫn truyền `None`. `with_work` có **0 test**. §Completion Notes #4 khai *"chưa có method phân giải nào thực sự chạy"* — đúng nhưng **hẹp hơn thực tế**: cả cái slot Tác phẩm chưa có consumer nào. Chốt: AC9 đạt hay chưa đạt?
- [x] [Review][Decision] **BOM và CRLF đóng băng vĩnh viễn vào `source_text`** — bước *"chuẩn hoá tối thiểu"* (`core/segment/import.rs:133-135`) rỗng có chủ ý (FR124/125 → Epic 6). Nhưng `EF BB BF` của Notepad là UTF-8 **hợp lệ**, qua `String::from_utf8` trót lọt và nằm lại thành ký tự đầu của Chương. Đây **đúng lập luận Quyết định #6** đã dùng để kéo bảng mã về story này: AD-4 đóng băng ranh giới segment tính lúc nhập ⇒ Epic 6 không sửa lại được. Chốt: cắt BOM ngay hôm nay, hay chấp nhận và mở nợ?
- [x] [Review][Decision] **Trần kích thước tệp nhập** — `core/segment/import.rs:146` `std::fs::read` đọc trọn tệp, rồi `String`, rồi bind cả chuỗi vào SQLite, tất cả trên **luồng invoke đồng bộ** (`commands/project.rs:222,239` là `pub fn`, không `async`). Một tệp vài GB ⇒ cạn bộ nhớ. Con số trần là một số sản phẩm — story yêu cầu *"mọi con số tạm phải tự khai là tạm và nêu tên story sở hữu"*, nên nó cần chữ ký của Ice.

**Patch — sửa được không cần hỏi**

- [x] [Review][Patch] 🔴 **Tạo Tác phẩm trùng tên XOÁ TRẮNG Tác phẩm cũ** — `create_dir_all` không phân biệt "thư mục tôi vừa tạo" với "thư mục đã có sẵn"; `INSERT ... VALUES (1, …)` đụng `CHECK (id = 1)` ⇒ nhánh dọn dẹp `remove_dir_all` **cả `.atproj` của người dùng**. Đã ép lộ bằng test chạy thật. Biến thể tệ hơn: một `.atproj` từ bản mới hơn ⇒ `SchemaTooNew` ⇒ cũng bị xoá, tức **AC7 "từ chối mở, không bao giờ ghi vào" kết thúc bằng xoá sạch**. Sửa: tạo độc quyền + chỉ xoá thư mục do chính lượt gọi này tạo. [`src-tauri/src/core/library/atproj.rs:93-101`, `src-tauri/src/commands/project.rs:96,126,136`]
- [x] [Review][Patch] 🔴 **Test của story hợp thức hoá đúng lỗi trên** — `a_failed_import_leaves_no_half_built_folder_behind` tự tạo `Nua Voi.atproj/` **trước** khi gọi `create_work`, rồi assert `!expected_dir.exists()`. Cổng này *có* đỏ-rồi-xanh nhưng **canh sai bất biến**. Phải viết lại cùng lượt với patch trên, + thêm ca `creating_a_work_over_an_existing_folder_never_deletes_it`. [`src-tauri/tests/project_contract.rs:268-289`]
- [x] [Review][Patch] 🔴 **Kéo-thả ghi xuống đĩa KHÔNG có bước xác nhận — phá thẳng AC1** — AC1 nguyên văn: *"không có gì được ghi xuống đĩa trước khi người dùng xác nhận"*. `on_window_event` bắt drop **toàn cửa sổ** (không theo phần tử, không theo chế độ) và `submitDroppedPath` gọi thẳng lệnh tạo. Thả một `.txt` lên Workspace/Reading, hay lên thanh tiêu đề, khi màn Library còn chưa hiện ⇒ vẫn tạo `.atproj` với `name`/`sourceLang`/`genre` cũ còn sót trong ref. Kèm: `event.payload[0]` — thả năm tệp thì nhập một, **bỏ im lặng bốn**. Sửa: drop **đổ vào ô nhập đường dẫn** rồi để người dùng bấm nút; gỡ bộ nghe khi rời chế độ. [`src-tauri/src/lib.rs:598-608`, `src/modes/libraryImport.ts:100-131`]
- [x] [Review][Patch] 🔴 **Giao diện nhập dùng được ĐÚNG MỘT LẦN mỗi phiên** — `createdWork` set ở `libraryImport.ts:69` và **không một đường nào đặt lại `null`**. Toàn bộ form **và** dải báo lỗi `role="status"` nằm trong nhánh `v-else`. Tạo xong Tác phẩm đầu tiên ⇒ form biến mất vĩnh viễn (ref tầng module sống qua `<KeepAlive>` và qua đổi chế độ): không tạo được Tác phẩm thứ hai, không sửa được `name`, không thấy được lỗi nào nữa. Và kéo-thả **vẫn chạy** ⇒ đường duy nhất còn lại để tạo Tác phẩm thứ hai lại đúng là đường tên rỗng ⇒ `Untitled` ⇒ patch #1 nổ, với thông báo lỗi vẽ vào một nhánh DOM không nằm trên màn hình. [`src/modes/libraryImport.ts:58,69`, `src/modes/LibraryMode.vue:75-79,155`]
- [x] [Review][Patch] **`meta.json` ghi trượt bị nuốt, `create_work` vẫn trả `Ok`** — Quyết định #3 chấp nhận **cửa sổ sập máy** giữa commit và `fs::write`; nó **không** cho phép đi tiếp khi hàm **trả về `Err`**. Kết quả: `.atproj` có hai thành phần thay vì ba (phá AC2), không đọc được metadata mà không mở SQLite (phá AC3), người dùng đọc *"Đã tạo Tác phẩm…"*. Và `rebuild_from_store` **không có chỗ gọi sản phẩm nào**, nên không gì tự dựng lại. [`src-tauri/src/commands/project.rs:141-148`]
- [x] [Review][Patch] **`isDragOver` không bao giờ bật** — `tauri.conf.json` không khai `dragDropEnabled` ⇒ mặc định `true` ⇒ bộ xử lý kéo-thả tầng OS của Tauri **chặn** sự kiện HTML5 của webview. `@dragenter`/`@dragover`/`@dragleave` không bao giờ chạy, `.dropzone.over` là **CSS chết**, người dùng không có một tín hiệu nào rằng vùng đó sống — trong khi thả **ngoài** vùng đó vẫn nhập. Sửa 0 phụ thuộc: forward `tauri://drag-enter`/`drag-leave` cùng khuôn `DRAG_DROP_EVENT` đã có. [`src/modes/LibraryMode.vue:126-129`, `src-tauri/tauri.conf.json`]
- [x] [Review][Patch] **`sanitize_name` lọt tên thiết bị Windows có đuôi, và không cắt tên quá dài** — chỉ so **nguyên chuỗi** với `WINDOWS_RESERVED`, nên `CON.txt`/`NUL.md`/`COM1.x` vẫn lọt và `create_dir` trượt trên NTFS. Tên trên ~255 byte (≈85 ký tự CJK/tiếng Việt) ⇒ `ENAMETOOLONG` hiện ra dưới dạng `project.create_failed` chung chung, và Tác phẩm **không bao giờ** tạo được. Cả hai là NFR14. [`src-tauri/src/core/library/atproj.rs:54-81`]
- [x] [Review][Patch] **Người dùng không được cho biết Tác phẩm nằm ở ĐÂU** — AC6 hứa *"copy thư mục là đủ để sao lưu"*, nhưng gốc `~/Documents/AuraTranslate` không bao giờ hiện lên giao diện, và câu xác nhận in `meta.name` — tên **trước** sanitize. Đặt tên `Tập 1: Khởi đầu` ⇒ màn hình nói `Tập 1: Khởi đầu`, đĩa có `Tập 1_ Khởi đầu.atproj`. Lời hứa của AC6 không giao được. [`src/modes/LibraryMode.vue:76`, `src/i18n/vi.json`]
- [x] [Review][Patch] **`create_dir_all` trượt ở `assets/` để lại `.atproj/` rỗng** — `create_dir_all(dir.join(ASSETS_DIR))` tạo thư mục cha **trước**; nếu bước con trượt, `<Tên>.atproj/` nằm lại. Phá AC8. [`src-tauri/src/core/library/atproj.rs:97`]
- [x] [Review][Patch] **`meta.json.tmp` rò lại khi `rename` trượt, và thiếu fsync thư mục cha** — nhánh lỗi của `rename` không dọn tệp tạm. Và `sync_all()` chỉ chạy trên **tệp tạm**, không trên thư mục chứa — nên chính lượt `rename` **không bền** qua một lần mất điện, trong khi doc-comment của module bán nó là ghi nguyên tử. [`src-tauri/src/core/library/meta.rs:141-160`]
- [x] [Review][Patch] **`io.read_failed` gắn `retryable = true` cho lỗi mà thử lại không sửa được** — ca thật phổ biến nhất trên đường này là **gõ sai đường dẫn** (`ENOENT`); bấm lại cho đúng kết quả ấy. Đúng ca *"một nút thử lại ở đó là **nói dối**"* mà AC8 gọi tên. [`src-tauri/src/core/segment/import.rs:114`]
- [x] [Review][Patch] **Tệp không có đuôi ⇒ câu lỗi vỡ** — `format` rỗng ⇒ người dùng đọc *"Định dạng . chưa được nhận…"*. [`src-tauri/src/core/segment/import.rs:162-174`]
- [x] [Review][Patch] **Tệp rỗng / văn bản rỗng qua đường kéo-thả tạo một Chương RỖNG** — hai nút có `:disabled` canh, nhưng `submitDroppedPath` không canh gì; thả một `.txt` 0 byte ⇒ Tác phẩm có `source_text = ""` và báo thành công. [`src/modes/libraryImport.ts:100-105`]
- [x] [Review][Patch] **`.dropzone` là một chặng Tab không làm gì** — `tabindex="0"`, không `role`, không `@keydown`, không click. Nó ăn một chặng Tab và vẽ một vòng focus rồi không phản hồi phím nào. [`src/modes/LibraryMode.vue:122-132`]
- [x] [Review][Patch] **Doc-comment `core/scope/**` vẫn khẳng định trạng thái TRƯỚC story** — `:170` *"trạng thái **duy nhất** tồn tại hôm nay"* và `:207` *"Đã mở một Tác phẩm chưa. Hôm nay **luôn** `false`"* — ngay trên hàm giờ trả `true` được. Doc của `WorkScope` **đã** viết lại, nên tệp **tự mâu thuẫn với chính nó**. Đúng thứ §Trí tuệ #9 nói reviewer săn. [`src-tauri/src/core/scope/mod.rs:170,207`]
- [x] [Review][Patch] **AC6 thiếu dòng ghi giới hạn 5 giây** — AC6 gạch đầu dòng hai đòi *"giới hạn được ghi ra, không giấu"* (copy khi đang mở có thể thiếu tối đa 5 giây, trần NFR18). Câu này **không có mặt ở đâu**: không trong `atproj.rs`/`meta.rs`/`commands/project.rs`, không trong chuỗi giao diện, không trong §Completion Notes, không trong các mục `deferred-work.md` mới. Đây là một gạch đầu dòng **kiểm được** của AC.
- [x] [Review][Patch] **AC5 thiếu test cho vế "không đường dẫn tuyệt đối"** — `a_copied_project_folder_opens_at_a_different_path` chứng minh mở lại được ở đường dẫn khác, không chứng minh vế còn lại. Mệnh đề hôm nay **đúng về cấu trúc** (không trường/cột nào chứa đường dẫn), nhưng AC5 viết cùng khuôn AC3 — *"test chứng minh, không phải một lời khẳng định"*. [`src-tauri/tests/project_contract.rs:170-183`]
- [x] [Review][Patch] **AC2 "đúng ba thành phần" chỉ đúng sau khi test lọc `-wal`/`-shm`** — trên đĩa một `.atproj` đang sống có **năm** mục. Bộ lọc hợp lý và có comment, nhưng cách diễn giải lại này không nằm trong năm độ lệch đã khai — và **Indexer của Epic 5 sẽ gặp năm mục, không phải ba**. [`src-tauri/tests/project_contract.rs:88-91`]
- [x] [Review][Patch] **Sổ sách story sai ở bốn chỗ** — ① Task 1–11 để `[ ]` trong khi §Change Log khai *"Triển khai xong 13/13 task"* (mình đã đối chiếu từng subtask: **đều đã cài**, trừ các lỗ ở ba mục Decision trên) — đúng ảnh gương của bài học §Trí tuệ #10 mà story tự trích; ② Task 12 ghi *"năm mục mới"*, §File List ghi *"sáu mục mới"*, §1-15 của `deferred-work.md` thật có **bảy**; ③ §File List ghi `vi.json` thêm 16 khoá, thật là **18** (4 `err.*` + 2 `command.library.*` + 12 `mode.library.*`; cổng báo 46 so với nền 28); ④ `deferred-work.md:227` ghi job ghi *"nội suy tham số qua `rusqlite::params`"` — mã dùng **tuple** (`project.rs:113,119`), và **buộc phải** dùng tuple vì `store_boundary.rs:62` cấm token `rusqlite` ngoài `core/store`. Bất biến nó ghi nhận (tham số **ràng buộc**, không `format!`) thì **đúng**; chỉ tên cơ chế sai.

**Hoãn**

- [x] [Review][Defer] **`replace_open_work` thả `Store` cũ khi đang giữ mutex** [`src-tauri/src/commands/project.rs:204-211`] — deferred, `*guard = Some(new_work)` chạy `Drop` (join luồng writer + checkpoint TRUNCATE có trần) **bên trong** vùng khoá. Hôm nay chưa có tranh chấp thật (chỉ hai chỗ chạm khoá này, và một trong hai chỉ chạy lúc thoát), nên nó là rủi ro tiềm ẩn không phải lỗi đang sống — cùng họ với mục `Tuning` chưa đo của Story 2.4.

**Bị loại (noise)** — 2 mục: ① *"`library.import_text` dispatch được từ palette/phím tắt với văn bản rỗng"* — chưa có palette, chưa gán phím (`index.ts:352-373` cố ý không gán), và hai nút có `:disabled`; đường rỗng thật là kéo-thả, đã tách thành một Patch riêng. ② *"`every_command_error_comes_from_the_store_vocabulary` mới chú thích chứ chưa đổi tên"* — Task 11 cho phép *"sửa **hoặc** khoanh lại có ý thức"*, và chú thích tại chỗ **chính là** bản ghi có ý thức mà nó đòi.

---

### Lượt áp patch — 2026-08-06

**Sáu quyết định của Ice ở lượt review** *(không mở lại, cùng hạng với sáu quyết định gốc)*:

| # | Phán quyết |
|---|---|
| **R1** | Trùng tên ⇒ **tự đánh số** `Tên (2).atproj`, không từ chối |
| **R2** | **Gỡ** `err.project.meta_too_new` + `MessageKey::ProjectMetaTooNew` + `ProjectError::MetaTooNew` cho tới khi Epic 5 dựng đường mở lại `.atproj` |
| **R3** | AC9 chấm **đạt chữ, CHƯA đạt mục đích** — không bịa thêm bảng tầng Work để vá; đóng ba lỗ trung thực (test, doc, phạm vi nợ) |
| **R4** | **Cắt BOM** ngay *(tạo tác giải mã)*; **không đụng CRLF** *(chuẩn hoá — Epic 6)* |
| **R5** | Trần nhập **100 MB** |
| **R6** | Thả tệp **điền vào ô đường dẫn**, không tự ghi xuống đĩa — AC1 |

**🔴 Đỏ-rồi-xanh cho lỗi nặng nhất — đo thật, không phải một lời khẳng định:**

Trước patch, một probe chạy thật *(`create_work_from_text` hai lần cùng tên `"Tay Du Ky"`)* in ra
`thu muc con ton tai? false` — Tác phẩm thứ nhất **bị xoá trắng khỏi đĩa**, gồm `project.db` và
`assets/`. Sau patch, `creating_a_work_over_an_existing_folder_never_touches_it` **XANH**: thư mục
cũ còn nguyên, tệp sentinel trong `assets/` còn nguyên, `work_id` cũ không đổi, Tác phẩm mới đi
vào `Truyen Kieu (2).atproj`.

⚠️ **Ca cũ `a_failed_import_leaves_no_half_built_folder_behind` đã bị GỠ, không phải sửa** — nó
tự tạo `Nua Voi.atproj/` rồi assert thư mục đó **bị xoá**, tức là nó khoá chính lỗi trên thành hợp
đồng. Sau khi `create_work_folder` chuyển sang **tạo độc quyền**, không còn cách nào từ ngoài ép
một lượt gọi đi vào một thư mục đã có — bất biến "không xoá thư mục của người khác" nay đúng
**theo cấu trúc**, không theo một phép kiểm lúc chạy. Vế "không để lại thư mục nửa vời" của
AC8 do bốn ca **từ chối trước khi ghi** canh *(`.docx` · không UTF-8 · quá nặng · không có
đuôi)*, cả bốn assert thư mục gốc **rỗng tuyệt đối**. Lý do đầy đủ ghi tại chỗ trong
`tests/project_contract.rs`.

**Chín cổng DoD sau lượt áp patch** — chạy lại toàn bộ, không chép số cũ:

| Lệnh | Trước review | Sau patch |
|---|---|---|
| `check:deps` · `check:tokens` · `check:i18n` · `check:commands` · `check:layout` | PASS | **PASS** |
| `npm run build` | PASS | **PASS** |
| `cargo test --locked` | 177 passed | **184 passed**, 0 failed |
| `check:scope` · `check:scope:bundled` | PASS | **PASS** |

**Bảy ca mới** *(6 ở `project_contract.rs` → 18 ca, 1 ở `scope_contract.rs` → 19 ca)*:
`creating_a_work_over_an_existing_folder_never_touches_it` ·
`repeated_names_keep_climbing_the_suffix_instead_of_colliding` ·
`no_absolute_path_of_this_machine_is_written_inside_the_project` *(AC5 vế hai — trước đây không
có ca nào)* · `a_utf8_bom_is_stripped_but_line_endings_are_left_alone` ·
`a_file_past_the_size_ceiling_is_refused_before_a_single_byte_is_written` ·
`a_file_with_no_extension_gets_its_own_message_instead_of_a_broken_sentence` ·
`a_folder_name_survives_both_platforms_rules` ·
`the_second_constructor_carries_a_work_tier_and_the_first_one_does_not` *(`with_work` trước đây
ship với **0 test**)*.

**CÒN MỞ sau lượt này — không đánh dấu đạt:**
1. 🔴 **AC10(a)** — bảng nghiệm thu tay đường hiển thị lỗi kho **trong webview thật** vẫn **chưa
   chạy**. Cùng lý do môi trường như lượt triển khai. Đây là ô Task 12 **duy nhất** còn `[ ]`.
2. ⚠️ **Kéo-thả tay thật bằng chuột người** vẫn chưa chạy — cơ chế nay còn nhận thêm hai event mới
   (`DRAG_ENTER_EVENT`/`DRAG_LEAVE_EVENT`), nên bề mặt cần nghiệm thu tay **rộng hơn** trước, không
   không hẹp hơn. Cả hai vẫn ở `deferred-work.md` làm nợ QA người trước khi phát hành.
3. ⚠️ **AC9 đạt chữ, chưa đạt mục đích** *(R3)* — phạm vi thật đã viết lại trong `deferred-work.md`.

### Nghiệm thu tay — 2026-08-06, Ice

**Môi trường:** macOS *(Darwin 24.6.0)* · engine webview **WKWebView** *(Tauri v2 dùng WKWebView trên macOS — không phải Chromium)* · bản `tauri dev` *(profile **debug**)* · thư mục Library `~/Documents/AuraTranslate/`.

| # | Bảng | Kết quả |
|---|---|---|
| 1 | Trùng tên: tạo `Truyện Kiều` hai lần ⇒ hai thư mục, thư mục đầu **còn nguyên** | ✅ PASS |
| 2 | Form + dải báo lỗi **vẫn còn** sau lần tạo đầu tiên; xác nhận in kèm đường dẫn thư mục | ✅ PASS |
| 3 | Kéo-thả: viền đổi màu lúc đang kéo; thả ⇒ **điền vào ô đường dẫn**, chưa ghi xuống đĩa | ✅ PASS |
| 4 | Ba ca biên: `.docx` · tệp không đuôi *(`README`)* · tệp Notepad có BOM — báo lỗi rõ, không để lại thư mục | ✅ PASS |
| 5 | 🔴 **AC10(a)** — `$APPDATA` chỉ-đọc *(`chmod 555`)* rồi mở lại app; **đọc dải báo lỗi kho bằng mắt trên webview thật** | ✅ PASS |

⇒ **Hai món nợ nghiệm thu tay treo từ Story 1.7/1.8 đóng**: đường hiển thị lỗi kho *(bảng 5)* và kéo-thả tệp thật bằng chuột người *(bảng 3)*. Cả hai trước đó **chưa từng chạy** — lượt triển khai và lượt code review đều ở môi trường agent CLI không có công cụ điều khiển GUI desktop.

🔴 **CÁI GÌ VẪN CHƯA ĐO — ghi thẳng, không để bảng trên đọc thành "đã xong mọi nền tảng":**

Cả năm bảng chạy **chỉ trên macOS**. CI của dự án dựng **cả `macos-26` lẫn `windows-2025`**, và NFR14 là một mệnh đề **hai nền tảng**. Bốn đường dưới đây là **Windows-only theo bản chất**, nên chúng **không** được bảng nào ở trên chạm tới:

1. **Tên thiết bị dành riêng** — `CON.txt`/`NUL.md`/`COM1` chỉ bị NTFS từ chối; trên macOS chúng là tên thư mục hợp lệ. Nhánh `sanitize_name` thêm hậu tố `_` có test đơn vị *(`a_folder_name_survives_both_platforms_rules`)* nhưng **chưa ai xác nhận** một thư mục tên `CON_.atproj` thật sự tạo được trên NTFS.
2. **`remove_dir_all` với một tệp đang mở** — bài học NFR14 mà `close_open_work` tồn tại để chống. Trên macOS xoá một tệp đang mở là **hợp lệ**, nên đường này **không thể** đỏ ở đây, kể cả khi nó hỏng.
3. **Trần độ dài** — `MAX_FOLDER_NAME_BYTES = 180` nhắm `NAME_MAX` 255 byte, nhưng Windows còn có trần **`MAX_PATH` 260 ký tự cho CẢ đường dẫn**, mà `~/Documents/AuraTranslate/` đã ăn một phần. Chưa đo.
4. **Kéo-thả trên Windows** — `WindowEvent::DragDrop` đi qua một cài đặt runtime **khác hẳn** *(WebView2/Win32 thay vì WKWebView/AppKit)*, và ba event `Enter`/`Leave`/`Drop` là đường mã mới của lượt code review.

⇒ **Nợ mở, không đóng:** chạy lại **cả năm bảng cộng bốn mục trên** trên một máy Windows thật trước khi phát hành. Ghi vào `deferred-work.md`.
