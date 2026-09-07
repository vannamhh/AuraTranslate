---
title: 'Story 6.8: Allowlist mạng hai tầng và nhật ký domain'
type: 'feature'
created: '2026-09-07'
status: 'done'
baseline_commit: '7cf327761422f75634862d7b9c57c758d2116587'
review_loop_iteration: 0
context:
  - '{project-root}/AGENTS.md'
  - '{project-root}/src-tauri/AGENTS.md'
  - '{project-root}/src/AGENTS.md'
  - '{project-root}/tests/AGENTS.md'
  - '{project-root}/scripts/AGENTS.md'
---

<frozen-after-approval reason="human-owned intent — do not modify unless human renegotiates">

## Intent

**Problem:** `Fetcher` hôm nay cưỡng chế đúng **một** luật — chuyển hướng phải **cùng host** (`fetcher.rs:101-116`) — và nhận một chuỗi URL **trần**, không ngữ cảnh lần nhập nào. `grep allowlist|domain_log` trên `src-tauri/src` + `src` cho **0 dòng mã**. NFR19 hứa người dùng **nhìn thấy** danh sách domain ứng dụng đã gọi; hôm nay **0** bề mặt hiện nó, và màn **Cài đặt** — nơi UX-DR42 đặt bảng đầy đủ — có **0 dòng mã, 0 khoá i18n**, dù `vi.json:471` đã hứa nó với người dùng.

**Approach:** Allowlist **không trạng thái**, dựng tại chỗ từ chính danh sách URL của lượt nhập rồi truyền **vào** `fetch` (Ice chốt 2026-09-07): *"sống đúng một lần nhập"* đúng theo **cấu tạo** — không có vòng đời nào để rò rỉ, và không phải đúc một định nghĩa "hết lần nhập" mà phía Rust hôm nay chưa có. Mọi lời gọi ghi một bản ghi **thô** vào một ô trong bộ nhớ sống theo **phiên chạy ứng dụng**. Hai bề mặt đọc: dòng tóm tắt ở chân màn xem trước, và mục **Quyền riêng tư** trong một khung **Cài đặt** mới — **lớp phủ thứ chín**, không một chế độ thứ tư.

## Boundaries & Constraints

**Always:**
- 🔴 `Fetcher` là chỗ **duy nhất** cưỡng chế allowlist (AD-41 spine `:526`: *"chỗ duy nhất AD-41 phải canh"*). Kiểm ở chỗ gọi thì `fetch` vẫn là một cửa mở ⇒ chữ ký đổi thành `fetch(url, &Allowlist, ResourceKind)`.
- 🔴 Cưỡng chế **cả ở chặng chuyển hướng**, bên trong `redirect::Policy::custom` — không phải kiểm sau `send()`.
- 🔴 Tầng 2 **chỉ ảnh, không bao giờ tài liệu**. Luật này sống trong **kiểu** (`ResourceKind` × tầng), không trong một `if` ở chỗ gọi.
- Nhật ký ghi **THÔ**: một bản ghi cho **mọi** lời gọi, cả cho phép **lẫn từ chối**. Bảng **hiện gộp** theo domain (Ice chốt 2026-09-07).
- Ô nhật ký `.manage` trong `open_work_slot` (`lib.rs:1036`, gọi đúng một lần ở `:817` trong `setup`) ⇒ phiên chạy ứng dụng. **0 bước di trú**, `schema_version` ở nguyên **19**.
- 🔴 Mọi mảnh logic nghiệm thu được phải có một **hàm thuần `pub`** nhận state trần — crate không khai `tauri` feature `test-utils`, **không có `MockRuntime` trong toàn kho**; logic chỉ sống trong `mod wire` là logic **không test được ở tầng Rust** (khuôn: `chapters_shape_if_all_ok` `:2042`, `clear_url_import_items_after_successful_confirm` `:1938`).
- Nhãn hai tầng qua **hàm trả khoá LITERAL** (khuôn `cleanupTierLabelKey` `ImportPreviewOverlay.vue:184-186`), **0 màu phân loại** — `.ip-cleanup-tier` `:1238` là khuôn CSS.
- Cài đặt là **lớp phủ thứ chín** theo khuôn `App.vue:350-372`, vào bằng **nút titlebar** theo khuôn `App.vue:239-247` (ba dòng bắt buộc: `data-*-open` · `@mousedown="focusOnPointerDown"` · `@click="dispatch"`). AD-24 cấm một chế độ thứ tư bằng chữ (`App.vue:39-41`).
- Mười mục nav chưa có thân **luôn hiện** và **luôn nói vì sao rỗng kèm tên chủ** — khuôn `tier_empty_story_6_9` của Story 6.3.
- 🔴 Cổng mới cho tệp mới: `webimport_boundary.rs` neo **cứng** `fetcher.rs`/`extractor.rs` (`:168`, `:207`, `:304`) ⇒ `core/webimport/allowlist.rs` sẽ **không bị cổng nào canh** nếu story này không tự thêm mệnh đề cho nó.

**Ask First:**
- 🔴 **Trần số bản ghi nhật ký** — phải kèm một **phép đo** (cỡ một bản ghi × số lời gọi thật của một phiên), không đúc một con số. Không đo được ⇒ **DỪNG và hỏi Ice**. Cùng kỷ luật `MAX_RESPONSE_BYTES` (`fetcher.rs:26-32`).
- Thứ tự và tên **11 mục** nav Cài đặt, và `Ngưỡng quét Glossary` (`vi.json:179`, hôm nay là lớp phủ riêng) có dọn vào Cài đặt không.
- Bất kỳ bước di trú lược đồ nào. Ice đã chốt nhật ký sống trong bộ nhớ ⇒ muốn bền là một **quyết định mới**.
- Thêm bất kỳ crate hay plugin nào. Kỳ vọng: **0**.

**Never:**
- Không tải ảnh, không `ASSET`, không cột `source_url` — Story 6.11. ⚠️ AC *"không tải lại ảnh đã có"* **không dựng được** ở story này (0 bảng ảnh, 0 cột `source_url` trong lược đồ 19) ⇒ ghi **một món nợ có chủ Story 6.11**, không sửa `epics.md`.
- Không hàng tầng **`AI`** trong bảng — `core/ai/mod.rs` có **10 dòng, 0 dòng mã**; điểm ra mạng thứ nhất chưa tồn tại ⇒ nợ có chủ **Epic 4**. Mockup vẽ hàng đó (`web-import.html:439-443`) là vẽ trước.
- Không phân trang, không "xem thêm" che bớt hàng — mockup `:421` là một cam kết: *"ghi mọi lần gọi, không rút gọn"*.
- Không màu phân loại tầng; không `opacity` trung gian (`check:tokens` Kiểm D `:1361` cấm, không miễn trừ).
- Không đọc `charset` của header `Content-Type` — nợ `:9248` giữ nguyên *"Chủ MỚI: chưa có"*, story này **không** nhận.
- Không chạm ô dán URL của `LibraryMode.vue` ⇒ nợ *"thông điệp cho danh sách rỗng"* (`:9135`) **không** kích hoạt điều khoản "story nào chạm lại form kế tiếp".
- Không khử trùng URL trùng lặp — quyết định sản phẩm, chưa ai chọn.
- Không chế độ thứ tư. Không đường sửa ranh giới bàn phím (6.9). Không bộ lọc *"cần xem"* (6.10).

## I/O & Edge-Case Matrix

| Scenario | Input / State | Expected Output / Behavior | Error Handling |
|---|---|---|---|
| Dán N link, **chưa bấm** | N dòng trong ô | **0** lời gọi mạng ⇒ **0** bản ghi nhật ký | N/A — allowlist chưa tồn tại vì chưa có lượt nhập |
| Tải N link tốt | N host, tất cả tầng 1 | N bản ghi `(thời điểm, domain, Tài liệu, cho phép)`; chân màn hiện *"Đã gọi N domain · xem"* với N = số domain **phân biệt** | N/A |
| Host **ngoài** cả hai tầng | `fetch` gọi tới một host không trong danh sách | **Từ chối trước khi mở kết nối**; một bản ghi `(…, từ chối)` | 🔴 Máy chủ đích nhận **đúng 0 kết nối** |
| Chuyển hướng ra host **ngoài** allowlist | 302 tới host lạ | **Từ chối tại chặng**, `RedirectBlocked…` | Đích nhận **0** kết nối (khuôn ca `webimport_contract.rs:78`) |
| Chuyển hướng tới host **khác nhưng có trong danh sách dán** | 302 giữa hai host tầng 1 | **Cho phép** — AD-41 nói *"host ngoài allowlist"*, không nói *"khác host"* | ⚠️ Nới so với 6.7; xem §Design Notes |
| **Tài liệu** từ host tầng 2 | `ResourceKind::Document` + host chỉ ở tầng 2 | **Từ chối**; bản ghi mang lý do | 🔴 Luật ở tầng KIỂU, không ở chỗ gọi |
| **Ảnh** từ host tầng 2 | `ResourceKind::Image` + host tầng 2 | **Cho phép** | ⚠️ **0 chỗ gọi sản phẩm** hôm nay — chỉ test phủ (xem §Never) |
| Mở Cài đặt › Quyền riêng tư, **chưa gọi mạng lần nào** | Nhật ký rỗng | Bảng hiện một câu nói **vì sao** rỗng, không phải một bảng trắng | 🔴 Rỗng im lặng là lớp lỗi trung tâm |
| Mở Cài đặt sau khi khởi động lại app | Nhật ký trong bộ nhớ | Rỗng, kèm **câu nói ra** rằng nhật ký sống theo phiên chạy | Không giả vờ có dữ liệu |
| Mở một trong **10** mục nav chưa có thân | — | Câu nói rõ mục này thuộc story nào | Không ẩn mục, không `v-if` giấu |

</frozen-after-approval>

## Code Map

**Rust — điểm tiêm**
- `src-tauri/src/core/webimport/fetcher.rs:97` `fetch(url)` — chữ ký đổi. `:56` `MAX_SAME_HOST_REDIRECTS = 10` (chép trần mặc định `reqwest`) giữ nguyên vai trò; `:101-116` chính sách chuyển hướng là chỗ allowlist phải vào. `:179` `classify_send_error`, `:198` `looks_like_html` không đổi.
- `src-tauri/src/core/webimport/allowlist.rs` — **TỆP MỚI**. `Allowlist` + `ResourceKind` + phép quyết định, **hàm thuần**, 0 `tauri::`.
- `src-tauri/src/core/webimport/mod.rs:45-49` — khai `pub mod allowlist;` và re-export. ⚠️ Story 6.7 đã dính đúng lỗi này một lần: thiếu một dòng `mod` thì trình liên kết loại trọn module.
- `src-tauri/src/commands/project.rs:1972` `fetch_url_import_item` · `:2025` `fetch_url_import_items` — chỗ gọi **sản phẩm duy nhất** của `fetch` (`:1973`). Allowlist dựng ở đây từ chính `urls`.
- `project.rs:1915-1926` `UrlImportItem`/`UrlImportItemsState` · `:1963-1967` `UrlImportBatchWire` · `:2080` `url_import_batch_wire` — hình dạng dây, nơi thêm rows nhật ký của lượt gọi.
- `project.rs:3567`/`:3594`/`:3628` ba vỏ `wire` — thêm state nhật ký qua `try_state`, **0 quy tắc trong vỏ**.
- `src-tauri/src/lib.rs:1036` `open_work_slot` (gọi ở `:817`) — `.manage` ô nhật ký cạnh `:1052`. `:649-651` `generate_handler!` — đăng ký lệnh đọc nhật ký.
- `src-tauri/src/core/i18n/mod.rs:63` macro `message_keys!`, khối Story 6.7 ở `:559-584` — khuôn thêm khoá. `:636` `IpcError` là **struct 4 trường**, không enum; `:679` là chỗ dựng duy nhất.

**Rust — cổng**
- `src-tauri/tests/webimport_boundary.rs:168`/`:207`/`:304` — ba mệnh đề neo **cứng** hai tệp; `:242` `path_is_allowed_for_reqwest`; `:26` `SRC_RS_FLOOR = 50`. Thêm mệnh đề cho `allowlist.rs`.
- `src-tauri/tests/webimport_contract.rs:32-46` `spawn_once` (TcpListener `127.0.0.1:0`, port động, **một** kết nối) · `:49` `html_page_with_paragraphs` · `:61` `ok_html_response` · `:71-79` kỹ thuật giả lập host khác bằng **`127.0.0.1` vs `localhost`** kèm `AtomicUsize` đếm kết nối đích. ⚠️ **Không `join()`** handle của server bị chặn — `accept()` treo vĩnh viễn.
- `src-tauri/tests/ipc_contract.rs:232`/`:266`/`:325` — ba ca canh `MessageKey` ↔ `vi.json`, chạy trên `MessageKey::ALL`.
- `src-tauri/tests/segment_contract.rs` — ca ghim `schema_version() == 19`; **không** được đổi.

**Frontend**
- `src/ImportPreviewOverlay.vue:914` đóng `<template v-if="importPreview !== null">` · `:932-934` — 🔴 **điểm chèn dòng tóm tắt là giữa hai dòng này, NGOÀI khối bốn tầng**: nhánh URL với mọi mục hỏng cho `importPreview === null` (`:915-921`) — đúng ca mạng đã bị gọi nhiều nhất. `:184-186` `cleanupTierLabelKey` (khuôn khoá literal) · `:746` chỗ render nhãn tầng · `:1238` `.ip-cleanup-tier` · `:1569` `.ip-hint` · `:1577` `.ip-actions`.
- `src/importPreviewState.ts:1008` `resetImportPreview` — ô tích luỹ rows của lượt nhập phải được **gọi tên** ở đây (`check:panel-refs`). `:542` `applyUrlImportBatch` là chỗ rows chảy vào.
- `src/config/project.ts:534-543` `UrlImportItemWire`/`UrlImportBatchWire` · `:558-578` `isUrlImportItemWire`/`isUrlImportBatchWire`/`callUrlImportBatch` — 🔴 mọi kiểu dây mới **phải** có vị từ kiểm kiểu lúc chạy theo khuôn này; adapter **không bao giờ ném**. `:554-556` hằng `CMD_*`.
- `src/SettingsOverlay.vue` + `src/settingsState.ts` — **HAI TỆP MỚI**. Khuôn bảng: `src/AttributionOverlay.vue:185-260` (thead từ `t()`, `v-for` từ IPC, **nhánh lỗi ĐỨNG TRƯỚC nhánh rỗng** `:185-201`, trạng thái nói bằng chữ `:222-225`, hàm ánh xạ trả khoá literal `:234`/`:246`, `aura-allow-text` cho dữ liệu ngoài `:220`).
- `src/App.vue:239-247` nút titlebar (ba dòng bắt buộc) · `:371-372` mount lớp phủ · `:39-41` luật cấm chế độ thứ tư.
- `src/commands/index.ts:162` `CommandDeps` · `:2661-2671` khuôn đăng ký `glossary.manage.open` · `:38` `MODE_IDS` (**không** đụng) · `:66-73` `FOCUS_OWNERS` (**không** thêm).
- `src/main.ts:413` `installCommands({…})` — tiêm hàm mở lớp phủ, khuôn `openGlossaryManage` `:751`.
- `src/i18n/vi.json` — **một** tệp ngôn ngữ, object **phẳng**, khoá `^[a-z0-9]+(\.[a-z0-9_]+)+$`. `:249-250` là cặp nhãn tầng đã có (`Toàn cục`/`Tác phẩm`) — `Tài liệu`/`Ảnh` là lượt thứ ba của cùng khuôn.

**Chuỗi UX đã chốt — dùng nguyên văn, đừng đúc lại**
- `mockups/web-import.html:393` `Đã gọi <b>N</b> domain · xem` (`·` là middle dot; `xem` **gạch chân, không tô màu**)
- `:420` tiêu đề khối `Đã gọi ra mạng` · `:421` câu dẫn · `:423` năm cột `Thời điểm · Domain · Tầng · Vì sao được phép · Kết quả` · `:428`/`:435` hai câu lý do · `:447` chú giải tầng 2
- `settings.html:145-156` nav **10** mục hiện có, theo thứ tự
- ⚠️ `EXPERIENCE.md:429`: bản dựng là **minh hoạ**, `DESIGN.md`/`EXPERIENCE.md` thắng khi mâu thuẫn ⇒ CSS `.tg-doc` bôi `--primary` (`web-import.html:145`) **thua** `DESIGN.md:165` (primary chỉ cho ba việc cũ).

## Tasks & Acceptance

**Execution:**
- [x] `src-tauri/src/core/webimport/allowlist.rs` -- dựng `Allowlist` (hai tầng) + `ResourceKind` + phép quyết định, hàm thuần -- luật *"tầng 2 chỉ ảnh"* phải sống trong kiểu, không trong một `if` ở chỗ gọi
- [x] `src-tauri/src/core/webimport/mod.rs` -- khai `pub mod allowlist;` + re-export -- thiếu dòng `mod` thì trình liên kết loại trọn module (đã dính một lần ở 6.7)
- [x] `src-tauri/src/core/webimport/fetcher.rs` -- `fetch(url, &Allowlist, ResourceKind)`; allowlist vào **bên trong** `redirect::Policy::custom` -- `Fetcher` là chỗ duy nhất cưỡng chế (AD-41)
- [x] `src-tauri/src/core/webimport/domain_log.rs` -- bản ghi thô + ô chứa + hàm thuần ghi/đọc; **trần số bản ghi kèm phép đo** -- không có `MockRuntime` ⇒ phải test được không cần `AppHandle`
- [x] `src-tauri/src/commands/project.rs` -- dựng allowlist tại chỗ từ `urls`; truyền vào `fetch`; ghi nhật ký mọi lời gọi; rows vào `UrlImportBatchWire` -- allowlist không trạng thái ⇒ "một lần nhập" đúng theo cấu tạo
- [x] `src-tauri/src/lib.rs` -- `.manage` ô nhật ký trong `open_work_slot`; đăng ký lệnh đọc nhật ký -- `&tauri::App` chỉ có ở `setup` ⇒ đúng phạm vi phiên chạy
- [x] `src-tauri/src/core/i18n/mod.rs` -- 🔵 **KHÔNG khoá `MessageKey` mới** -- xem §Design Notes "vì sao domain log không mượn kênh `MessageKey`". Nhãn tầng/lý do đi qua chuỗi định danh máy (`ResourceKind` `serde(rename_all snake_case)`, `DomainLogEntryWire.tier`) + hàm ánh xạ THUẦN phía TS, cùng khuôn `cleanupTierLabelKey` -- không phải kênh lỗi IPC nên không cần `message_keys!`
- [x] `src-tauri/tests/webimport_boundary.rs` -- mệnh đề 5 mới canh `allowlist.rs` -- cổng hôm nay **mù** với tệp mới
- [x] `src-tauri/tests/webimport_contract.rs` -- **bốn ca AD-41 bắt buộc** + ca tầng 2 -- framework không cưỡng chế thay (spine `:542`)
- [x] `src/config/project.ts` -- kiểu dây nhật ký + vị từ kiểm kiểu lúc chạy + hằng `CMD_*` + adapter ba trạng thái -- adapter không bao giờ ném
- [x] `src/ImportPreviewOverlay.vue` -- dòng tóm tắt chèn giữa `:932` và `:934` -- đặt trong khối bốn tầng thì nó biến mất đúng lúc cần nhất
- [x] `src/importPreviewState.ts` -- ô tích luỹ rows theo lượt nhập, gọi tên trong `resetImportPreview` -- `check:panel-refs`
- [x] `src/settingsState.ts` -- state lớp phủ Cài đặt + rows nhật ký + `resetSettings()` -- mọi ô cấp module phải được gọi tên trong `reset*()` của **chính tệp đó**
- [x] `src/SettingsOverlay.vue` -- nav 11 mục; thân **chỉ** cho Quyền riêng tư; 10 mục còn lại nói vì sao rỗng kèm tên chủ -- khuôn `AttributionOverlay.vue:185-260`
- [x] `src/App.vue` -- nút titlebar + mount lớp phủ thứ chín -- ba dòng bắt buộc, không chế độ thứ tư
- [x] `src/commands/index.ts` + `src/main.ts` -- đăng ký command mở Cài đặt + tiêm hàm -- `@click` là **đúng một** `dispatch('<id>')` (`check:commands` Kiểm A)
- [x] `src/i18n/vi.json` -- mọi chuỗi mới, giọng **vô nhân xưng** -- Kiểm D cấm "bạn"/"chúng tôi", `VOICE_EXCEPTIONS` rỗng ⇒ ba câu mockup viết ngôi hai **phải viết lại**
- [x] `tests/frontend/` -- ca cho dòng tóm tắt (kể cả khi `importPreview === null`) và bảng gộp -- khuôn `importPreviewOverlayRender.test.ts:1-60` (nạp **động** cả state lẫn component trong cùng một lượt)
- [x] `_bmad-output/implementation-artifacts/deferred-work.md` -- ba mục: AC ảnh (**Chủ: 6.11**), hàng tầng AI (**Chủ: Epic 4**), khung Cài đặt 10 mục còn rỗng (**Chủ: Ice**); và 🔵 sửa tại chỗ mục `:9942` -- đóng bằng chữ, không xoá

**Acceptance Criteria:**
- Given một host không nằm trong hai tầng, when `fetch` được gọi, then máy chủ đích nhận **đúng 0 kết nối** — đo bằng `AtomicUsize`, không bằng mã trả về
- Given hai tệp `allowlist.rs` và `fetcher.rs`, when gỡ lời gọi allowlist ra khỏi `fetch`, then bộ test **CŨ** phải **đỏ** — đối chứng là một phép **GỠ**, không phải một phép chèn
- Given `cargo test`, when chạy, then `schema_version() == 19` **không đổi** và **0** bước di trú mới
- Given màn Cài đặt, when mở bằng nút titlebar và bằng phím tắt, then tiêu điểm vào lớp phủ và `Tab` không thoát ra ngoài
- Given ứng dụng vừa khởi động lại, when mở Cài đặt › Quyền riêng tư, then bảng **nói ra** rằng nhật ký sống theo phiên chạy — không phải một bảng trắng
- Given tám cổng `check:*` và `cargo test` và `npm run test`, when chạy trọn, then **0** finding và **0** ca đỏ

## Spec Change Log

## Design Notes

**Vì sao allowlist KHÔNG TRẠNG THÁI — và điều đó đóng đúng lỗ hổng nào.** Ice chốt 2026-09-07. Phía Rust hôm nay **không có định nghĩa nào** cho *"hết một lần nhập"*: `cancel_import_preview` (`project.rs:1818`) có đúng một chỗ gọi sản phẩm — `sync_pending_from_url_items` `:2065`, nhánh `None` — và nó nghĩa là *"còn mục hỏng"*, **không** phải *"người dùng đã huỷ"*; huỷ thật là quyết định của `importPreviewState.ts::cancelImportPreview` và **không đi xuống Rust**. Một `AllowlistState` có trạng thái vì thế sẽ **sống sót qua một lượt huỷ** — đúng thứ AD-41 cấm, và không cổng nào bắt được. Dựng tại chỗ từ danh sách URL mà lệnh đang cầm làm mệnh đề *"sống đúng một lần nhập"* thành **hệ quả của cấu tạo**: không có vật thể nào tồn tại giữa hai lệnh, nên không có gì để rò rỉ. Cùng lối lý luận đã dùng cho hai con số *N link* của 6.7 — biến một lời hứa thành một thứ kiểm được bằng hình dạng mã.

**Vì sao chuyển hướng giữa hai host CÙNG trong danh sách dán được phép — và vì sao đó không phải nới lỏng tuỳ tiện.** `fetcher.rs:101-116` hôm nay chặn **mọi** chuyển hướng khác host, chặt hơn AD-41. AD-41 (`spine:526`) viết *"từ chối mọi host **ngoài allowlist**, kể cả khi gặp chuyển hướng"* — luật là **thành viên allowlist**, không phải **cùng host**. Hai host đều do người dùng dán thì cả hai đã sắp được tải trong chính lượt này; chặn chặng chuyển hướng giữa chúng không bảo vệ thêm gì mà làm hỏng một ca thật (site đổi domain giữa danh sách). ⚠️ Ghi thẳng cái mất: ca `webimport_contract.rs:78` hôm nay xanh nhờ *"khác host"*; sau story này nó xanh nhờ *"không trong allowlist"* — **mệnh đề đổi, không chỉ mã đổi**, nên ca đó phải được viết lại cho nói đúng thứ nó đang canh, và một ca MỚI phải phủ chiều ngược lại (hai host cùng trong danh sách ⇒ theo được).

**Vì sao tầng 2 vẫn dựng dù chưa ai đi qua.** Tầng 2 chỉ cho phép **ảnh**, mà ảnh là Story 6.11 — hôm nay **0 chỗ gọi sản phẩm**. Ba lý do dựng ngay thay vì hoãn: AD-41 đòi **bộ test riêng** cho đúng bốn mệnh đề, một trong đó là *"từ chối tài liệu ở tầng 2"* — không có tầng 2 thì mệnh đề đó không phát biểu được; luật *"chỉ ảnh"* nằm trong **kiểu**, và một kiểu thiếu một nhánh sẽ được 6.11 thêm vào dưới áp lực tiến độ, đúng chỗ dễ sai nhất; và `Extractor` sẽ phải trả host tham chiếu ở 6.11 — hình dạng tiếp nhận có mặt trước thì 6.11 không phải sửa ngược `fetcher.rs`. ⇒ Dựng máy móc + test phủ, và ghi một **món nợ có tên** rằng tầng 2 chưa có khách hàng sản phẩm cho tới 6.11. Đây là *"năng lực chưa dựng ≠ lệch spec"*: **không** sửa `epics.md`.

**Vì sao ba câu tiếng Việt của mockup phải viết lại.** `check:i18n` Kiểm D (`check-i18n.mjs:1187-1240`) cấm *"bạn"* và *"chúng tôi"*, `VOICE_EXCEPTIONS` mặc định **rỗng** (`:1200`). Ba chuỗi đã chốt của UX đều ngôi hai: *"Ứng dụng chỉ ra mạng khi **bạn** bấm"* (`:421`), *"Có trong danh sách link **bạn** dán"* (`:428`), *"…chỉ mở ra từ những trang **bạn** đã dán link"* (`:447`). ⇒ Giữ **nghĩa** và giữ **giọng giải thích**, đổi sang vô nhân xưng. Đây không phải một chỗ được xin miễn trừ — một miễn trừ ở đây mở đúng cánh cửa mà cổng dựng ra để đóng.

**🔵 THÊM (lượt thi công 2026-09-07) — bốn quyết định phát sinh KHI VIẾT, không có trong bản spec ban đầu, ghi lại để không ai đọc nhầm là đã được Ice cân nhắc trước:**

1. **`ResourceKind::Document` đổi tên thành `ResourceKind::Page` NGAY LÚC VIẾT — không phải một chọn lựa văn phong.** `cargo test --test naming_boundary` đỏ thật: `tests/naming_boundary.rs::the_real_source_tree_has_zero_naming_violations` bắt đúng biến thể `Document,` tại `allowlist.rs:55` (AGENTS.md:41 cấm `Project`/`Book`/`Novel`/`Document` cho khái niệm `Work`). Ý nghĩa ở đây — "trang/bài viết vừa tải, đối lập với ảnh" — không liên quan `Work`, nên đây là một xung đột TỪ VỰNG tình cờ; đổi tên (không xin miễn trừ, không nới `STORE_EXEMPT`) đóng đúng cổng mà không đánh đổi gì. Ảnh hưởng dây: `DomainLogEntryWire.kind`/TS union đổi `'document'` → `'page'`.
2. **Trần số bản ghi nhật ký domain (§Ask First) — đo được, kết luận là KHÔNG một trần cứng nào**, không phải một điểm còn treo cần hỏi Ice. Phép đo đầy đủ (cỡ một bản ghi, ~56 byte thân cố định + độ dài domain thật; một phiên không thực tế 100.000 bản ghi ~10-15 MiB, nhỏ hơn một `MAX_RESPONSE_BYTES` đơn lẻ) nằm ở doc-comment đầu `domain_log.rs`, giữ sống bởi test `a_domain_log_entry_is_small_enough_that_an_unbounded_vec_is_safe`. Cùng lý lẽ Story 6.7 đã dùng để từ chối một trần SỐ LINK: một trần bản ghi cắt bớt vi phạm thẳng cam kết "ghi mọi lần gọi, không rút gọn" mà không có gì ở thượng nguồn để nó bảo vệ.
3. **`core/i18n/mod.rs` KHÔNG nhận khoá `MessageKey` mới** — lệch với Code Map (dòng đó dự trù thêm khoá). `message_keys!`/`IpcError` là kênh LỖI (AD-21: *"hình dạng lỗi qua IPC là `{code, message_key, params, retryable}`"*); nhãn tầng/lý do của một bản ghi nhật ký KHÔNG phải một lỗi — chúng là dữ liệu hiển thị bình thường, cùng lớp với `CleanupRuleTierWire`/`license_kind`. Rust phát ra chuỗi định danh máy (`ResourceKind` `#[serde(rename_all = "snake_case")]`, `DomainLogEntryWire.tier: "tier1"|"tier2"|"denied"`), TS ánh xạ sang câu bằng `domainLogKindLabelKey`/`domainLogReasonKey` — hai hàm THUẦN, cùng khuôn `cleanupTierLabelKey`. Giữ nguyên kỷ luật AD-21 (Rust không phát văn bản hiển thị) mà không phải mượn một kênh được thiết kế cho lỗi.
4. **Hai câu hỏi của §Ask First KHÔNG có câu trả lời — quyết định TẠM của lượt thi công, không phải một lời chốt:** thứ tự 11 mục nav (mười mục đầu giữ nguyên thứ tự mockup, `privacy` thêm vào CUỐI) và `Ngưỡng quét Glossary` (KHÔNG dọn vào mục `glossary` — vẫn là lớp phủ riêng, `GlossarySettingsOverlay.vue`). Cả hai ghi ở `deferred-work.md` §"Deferred from: 6-8…", chủ **Ice**.

## Verification

**Commands:**
- `npm run build && cargo test --locked` -- expected: 0 đỏ; `dist/` phải có **TRƯỚC** `cargo test`. ⚠️ Số nền **1171** ghi ở spec 6.7 `:161` là số **trước** 6.7 — đo lại số thật trước khi khai một con số.
- `npm run test` -- expected: 0 đỏ. ⚠️ `fileParallelism: false` ⇒ một lượt ~98 s, không phải 27 s
- `npm run check:deps && npm run check:i18n && npm run check:tokens && npm run check:commands && npm run check:layout && npm run check:panel-refs && npm run check:gates && npm run check:debt-owner` -- expected: 0 finding mỗi cổng
- Đối chứng đỏ ①: **GỠ** lời gọi allowlist khỏi `fetch` rồi chạy `webimport_contract.rs` -- expected: **đỏ** ở cả bốn ca AD-41
- Đối chứng đỏ ②: **GỠ** mệnh đề mới khỏi `webimport_boundary.rs`, gieo một dòng `reqwest` vào `allowlist.rs` -- expected: cổng **xanh** (chứng minh cổng cũ mù); trả mệnh đề lại -- expected: **đỏ**
- Đối chứng đỏ ③: gieo chuỗi `"bạn"` vào một khoá mới trong `vi.json` rồi `npm run check:i18n` -- expected: **đỏ**; gỡ ra -- expected: **xanh**
- `cargo test --locked --test segment_contract` -- expected: `schema_version() == 19`, mệnh đề **không đổi**

**Manual checks (if no CLI):**
- Dán 3 link, chưa bấm: DevTools Network trống, chân màn **không** có dòng domain nào.
- Bấm tải với một link cố tình hỏng: bốn tầng biến mất (`importPreview === null`) nhưng dòng *"Đã gọi N domain · xem"* **vẫn hiện**.
- Bấm `xem`: bảng mở, nhãn `Tài liệu`/`Ảnh` là **chữ**, không màu phân loại; chụp màn ở cả hai theme.
- `Tab` xoay vòng trong lớp phủ Cài đặt, không thoát ra; `Esc` đóng và trả tiêu điểm về nút titlebar.
- Khởi động lại app rồi mở Cài đặt › Quyền riêng tư: bảng rỗng **kèm câu giải thích**, không phải một khung trắng.
