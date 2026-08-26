---
title: 'Story 3.7 — Đề xuất bản dịch bằng âm Hán Việt'
type: 'feature'
created: '2026-08-22'
status: 'done'
review_loop_iteration: 0
baseline_commit: 'dae3a3d3ba2665b64038bf339f7b1f5ffcd47fa3'
context:
  - '{project-root}/_bmad-output/implementation-artifacts/epic-3-context.md'
  - '{project-root}/_bmad-output/implementation-artifacts/3-6-trang-thai-cho-chot-va-dai-moc-chot-lan-dau-gap.md'
  - '{project-root}/AGENTS.md'
  - '{project-root}/src-tauri/AGENTS.md'
  - '{project-root}/src/AGENTS.md'
---

<frozen-after-approval reason="human-owned intent — do not modify unless human renegotiates">

## Intent

**Problem:** Với danh từ riêng tiếng Trung, âm Hán Việt **chính là** bản dịch quy ước (`北涼` → *Bắc Lương*) và nó đã nằm sẵn trong dữ liệu nhúng — nhưng hôm nay người dịch phải gõ lại từng cái. `core/glossary/**` có **0** dòng `use crate::core::dict` (`scan.rs` cố ý tiêm closure), nên cạnh mà **AD-36** chỉ định *(“âm Hán Việt cho FR113 đọc qua cổng `DictionarySource`… thêm cạnh `glossary/ → dict/`”)* **chưa tồn tại**, và dải chốt của Story 3.6 mở ra một ô nhập trần.

**Approach:** Một module **thuần** mới trong `glossary/` gọi `dict::lookup_han_viet`, ghép âm `primary` của từng ký tự thành một đề xuất, rồi chở nó ra **hai bề mặt đã có** — hàng ứng viên và dấu thuật ngữ chờ chốt. Dải chốt của Story 3.6 điền sẵn đề xuất đó; người dùng vẫn là người bấm.

## Boundaries & Constraints

**Always:**
- Cạnh `glossary/ → dict/` là cạnh **THẬT** (`use crate::core::dict::{...}`), không closure tiêm như `scan.rs` — AD-36 chỉ định đích danh. Chiều ngược `dict/ → glossary/` hôm nay là **0**; giữ nguyên ⇒ không chu trình.
- **0** bản chép dữ liệu Hán Việt trong `glossary/`; **0** định nghĩa `is_han` thứ hai (`dict_boundary.rs:330` canh) — gọi `crate::core::dict::is_han`.
- Vị từ *“ứng viên tiếng Trung”* là **HÌNH DẠNG CHUỖI** (mọi ký tự là Hán), **không** phải `source_lang` của Tác phẩm (`src-tauri/AGENTS.md:34`) — bảng `glossary_candidate` **không có cột ngôn ngữ** nào.
- 🔴 **Ba lý do RỖNG phải phân biệt được trên dây**: chưa gắn lớp từ điển · không phải chữ Hán · là chữ Hán nhưng thiếu âm. Một `Option<String>` trần là đúng lớp *rỗng im lặng* mà `AGENTS.md:46` gọi là lỗi trung tâm của dự án — trong cây git `resources/dict/` **rỗng** (AD-25), nên ca “chưa cài” là ca THƯỜNG GẶP NHẤT ở máy dev.
- Đề xuất theo **cùng bộ lọc nguồn đã tắt** với tab Hán Việt (Story 1.19 §Quyết định #3a) — tái dùng `commands::dict::disabled_sources`, không đúc phép đọc thứ hai.
- Đọc theo **LÔ**: một lượt `lookup_han_viet` cho **cả tập** thuật ngữ chờ chốt của một Chương, không một lượt cho mỗi thuật ngữ.
- Máy chỉ **ĐỀ XUẤT** (AD-20/FR55): **0** lượt ghi tự động. `approve_candidate`/`confirm_pending_translation` vẫn nhận `translation` **từ chỗ gọi**.
- Vỏ `wire` dùng `try_state`; adapter `src/config/glossary.ts` **không bao giờ ném**; trường trả về giữ `snake_case`.

**Ask First:**
- Bất kỳ phụ thuộc mới nào (NFR15), kể cả bật một `feature` chưa bật.
- Nếu lượt tính đề xuất trên đường `marks_for_source_text` đo được vượt **50 ms** (NFR2) ở một Chương thật: dừng, trình số.
- Nếu hoá ra **buộc** phải lưu đề xuất xuống lược đồ (một bước migration v15): dừng và hỏi — kế hoạch là tính LÚC ĐỌC.

**Never:**
- Không thêm cột `suggested_translation` vào `glossary_candidate`. Một bản chép dữ liệu từ điển trong `project.db` là **nhân bản dữ liệu** (AD-36 cấm) và nó lệch câm đúng ngày người dùng bật/tắt một nguồn.
- Không phơi `HanVietReading.all` ra dây — chỉ `primary`. *(Ice ký 2026-08-22.)*
- Không dựng màn duyệt hàng loạt (Story 3.8), không dựng nguồn gợi ý **“bạn vừa viết”** (chuyển chủ sang Ice) và **“TM”** (Epic 7).
- Không khai một `source_code` cho đề xuất: một cụm nhiều ký tự rút âm từ **nhiều lớp khác nhau**, nên *“nguồn của đề xuất”* không phải một giá trị xác định được.
- Không sửa `epics.md`/`prd.md`.

## I/O & Edge-Case Matrix

| Scenario | Input / State | Expected Output / Behavior | Error Handling |
|---|---|---|---|
| Danh từ riêng tra đủ | `北涼`, hai ký tự đều có âm | `han_viet_suggestion = "Bắc Lương"`, `han_viet_status = "ok"` — hoa đầu mỗi âm tiết, ghép bằng một khoảng trắng | N/A |
| Ký tự đa âm | `西` ⇒ `tây,tê` | Lấy `primary` ⇒ `"Tây"`; `all` **không** ra dây | N/A |
| Ứng viên tiếng Anh | `dragon` | `null` + `"not_chinese"`; mục đi đường **chờ chốt của Story 3.6** | N/A |
| Chuỗi Hán thiếu một âm | Một trong các ký tự `reading == None` | `null` + `"no_reading"` — **không** đề xuất một phần | N/A |
| Chuỗi lẫn Hán và Latin | `A北` | `null` + `"not_chinese"` — vị từ là **MỌI** ký tự là Hán | N/A |
| Chưa gắn lớp từ điển nào | `layers_loaded == false` (AD-25) | `null` + `"dict_unavailable"` — **khác** `"no_reading"`; màn hình không được nói *“thuật ngữ này không có âm”* | N/A |
| Nguồn thắng bị TẮT | Người dùng tắt một lớp gỡ rời | Đề xuất tính lại theo lớp kế tiếp — chữ có thể **ĐỔI**, không biến mất (FR36/FR37) | N/A |
| Dấu ĐÃ CHỐT | `is_confirmed == true` | **0** lượt tra Hán Việt cho dấu đó; `null` + `"not_requested"` — một nhãn RIÊNG, không mượn `"not_chinese"` | N/A |
| Dải chốt mọc | `聽潮閣` chờ chốt, đề xuất `"Thính Triều Các"` | Ô nhập **điền sẵn** chuỗi đó kèm nhãn *âm Hán Việt*; người dùng sửa hoặc xoá được trước khi gửi | N/A |
| Vào dải bằng hợp âm khi đang có vùng chọn | `focusGlossaryConfirmStrip(initialTranslation)` với vùng chọn khác rỗng | **Vùng chọn THẮNG** đề xuất — thao tác người dùng vừa làm đứng trên gợi ý của máy | N/A |
| Dải mọc khi chưa cài từ điển | `"dict_unavailable"` | Ô nhập **rỗng** + một dòng nói *chưa cài dữ liệu từ điển*, không im lặng | N/A |
| Nhận một ứng viên có đề xuất | `glossary_approve_candidate(id, "Bắc Lương", category)` | Mục Glossary mới, `translation` khác `NULL` ⇒ **đã chốt**; hàng ứng viên `resolution='approved'` | `id` không khớp ⇒ lỗi mang `message_key`, không `Ok` rỗng |
| Sửa một mục đã vào từ đề xuất | `glossary_update_term(...)` | Sửa được **như mọi mục khác** — không đường nào khoá riêng mục sinh từ đề xuất | N/A |
| Một lớp `.db` hỏng lúc tra | `lookup_han_viet` bỏ qua lớp đó | Lượt marks/ứng viên **vẫn trả đủ**; đề xuất tính trên các lớp còn lại | 0 lỗi ném lên dây — cùng luật `lookup_grouped` |
| Ngắt kết nối mạng | Rút mạng | Mọi nhánh trên **không đổi** — dữ liệu nhúng, **0** điểm ra mạng mới | N/A |

</frozen-after-approval>

## Code Map

**Đường Hán Việt — đọc, không cài lại**
- `src-tauri/src/ports/dict_source.rs:128` `fn han_viet(&self, chars: &[&str]) -> Result<Vec<HanVietHit>, StoreError>` — 🔴 **cổng** mà AD-36 chỉ định; đọc theo LÔ, trả hàng thô chưa tách âm.
- `src-tauri/src/core/dict/mod.rs:1125-1129` `lookup_han_viet(layers, chars, disabled) -> HanVietLookup` — tầng gom: dedupe `chars` trước khi tra, tách nhiều âm, chọn ưu tiên lớp. **Đây là hàm story này gọi.**
- `core/dict/mod.rs:1005-1013` `CharacterReading { character, reading: Option<HanVietReading> }` · `:1016-1025` `HanVietReading { primary, all, source_code }` · `:1029-1042` `HanVietLookup { characters, sources_used, layers_loaded }`.
  ⚠️ `:1019-1020` khai *“Story 1.17 và 3.7 cần danh sách đầy đủ”* — **hết đúng cho 3.7** (Ice ký 2026-08-22: chỉ `primary`). Sửa TẠI CHỖ kèm 🔵 + ngày, thu hẹp về Story 1.17.
- `core/dict/mod.rs:164` `pub fn is_han(c: char)` — 🔴 định nghĩa **DUY NHẤT**, `dict_boundary.rs:330` canh. Gọi, đừng chép.
- `src-tauri/src/commands/dict.rs:40-51` `fn disabled_sources(store: Option<&Store>) -> BTreeSet<String>` — **riêng tư hôm nay**; nâng lên `pub(crate)` và tái dùng, không đúc bản thứ hai.

**Bảng chờ ứng viên — chưa có trường đề xuất**
- `core/store/schema.rs:415-436` DDL `glossary_candidate` — `id · source_term · candidate_origin · resolution · created_at`, cộng `occurrence_count`/`context_example` (bước 14). **Không cột ngôn ngữ, không cột đề xuất.** Lược đồ `project.db` **v14**; story này thêm **0** bước migration.
- `core/glossary/candidate.rs:113-133` `GlossaryCandidate` · `:136-139` `is_pending()`.
- `core/glossary/candidate_store.rs:79-113` `pending_candidates(store)` · `:238-280` `approve_candidate(store, id, translation: Option<&str>, category)` — 🔴 `translation` **đến từ chỗ gọi**, đường tự sinh không tồn tại và không được dựng.
- `commands/glossary.rs:302-326` `GlossaryCandidateWire` + `From` — **không** `rename_all`; `:343-352` hàm thuần `glossary_pending_candidates(open)` (chưa mở Tác phẩm ⇒ `Ok(vec![])`); `:542-555` vỏ `wire`.

**Dấu thuật ngữ — chỗ dải chốt đọc**
- `core/glossary/entry.rs:243-263` `GlossaryMark { start, end, tier, is_confirmed, translation, id, source_term }` — 3.6 vừa thêm hai trường cuối; doc-comment `:232-242` là khuôn “sửa tại chỗ kèm 🔵” phải chép.
- `core/glossary/store.rs:859-864` `marks_for_source_text(resolver, global, work, text, lang)` — 🔴 **không** lọc `is_confirmed`; nó đã cầm `GlossaryEntry` đã phân giải nên biết ngay mục nào *chờ chốt*.
- `commands/glossary.rs:203-223` `GlossaryMarkWire` + `From` · `:272-291` hàm thuần `glossary_marks_for_chapter` · `:523-539` vỏ `wire`.
- `src/config/glossary.ts:217-256` kiểu `GlossaryMark` + `isGlossaryMark`/`isGlossaryMarkArray` — 🔴 type guard là chỗ **duy nhất** biết dây nói thật; `:333-368` kiểu/type guard `GlossaryCandidate`; `:385-410` `pendingGlossaryCandidates()`; `:470-493` `approveGlossaryCandidate(...)` (**0** chỗ gọi sản phẩm, chủ Story 3.8).
- `src/panels/glossaryMarksMap.ts:46-69` `SegmentTermSpan { start, end, isConfirmed, translation, id, sourceTerm, tier }` — mark đã cắt về toạ độ cục bộ; trường mới đi tiếp xuống đây.

**Dải chốt (Story 3.6) — chỗ đề xuất hiện ra**
- `src/glossaryConfirmStripState.ts:141` `applyTarget(next)` — 🔴 chỗ danh tính mục đổi, bump `sequence` và dọn `savedFocusEl`/`savedRange`/`enteredViaChord`. **Đây là chỗ điền sẵn đề xuất.**
- `:180` `syncGlossaryConfirmStripTarget(...)` gọi `selectPendingSpan(...)` — chọn span chờ chốt **trái nhất**.
- `:209-236` `focusGlossaryConfirmStrip(initialTranslation, isVisible)` — 🔴 `:233` *chỉ* điền khi ô đang **RỖNG**; đây chính là chỗ luật ưu tiên “vùng chọn thắng đề xuất” phải được viết lại cho đúng.
- `src/GlossaryConfirmStrip.vue` — khuôn `<label><span>{{t()}}</span><input v-model>`; 🔴 CSS: `flex: none`, 0 `position: fixed`, 0 `z-index`, 0 `box-shadow`, màu và cỡ chữ chỉ từ token.
- `src/i18n/vi.json:241-270` cụm `glossary.confirm.*` đã có — thêm khoá mới **cùng tiền tố**, phẳng, không giá trị rỗng.

**Cổng — thứ story này chạm**
- `src-tauri/tests/glossary_boundary.rs:129-130` `GLOSSARY_ONLY_SURFACE` (4 tên) · `:164-172` `QUICK_ADD_SURFACE` (**7** tên) · `:592` `commands_glossary_calls_the_new_quick_add_surface_not_the_forbidden_one` — 🔴 thêm một tên **bắt buộc** phải có lời gọi thật trong `commands/glossary.rs`.
- `:63` `RS_FLOOR = 44`, và ⚠️ `:64-67` khai *“53 tệp, cùng quần thể với `check-i18n.mjs`”* — **sai**: `all_rust_sources()` chỉ quét `CARGO_MANIFEST_DIR/src` (đo 2026-08-22: **52**), còn `check-i18n.mjs:288` quét cả `build.rs` (**53**). Hai quần thể, không một.
- Sàn khác đếm `src-tauri/src/**`: `check-i18n.mjs:288` (44) · `dict_boundary.rs:316` (46, gồm `tests/**`) · `:384` (34) · `matching_boundary.rs:57` (34) · `segment_boundary.rs:45` (34) · `store_boundary.rs:57` (34) · `scope_boundary.rs:53` (34). Sàn là **cận dưới** ⇒ thêm tệp không làm cổng đỏ.
- `core/i18n/mod.rs:284,286,293` — **ba** `MessageKey` miền `err.glossary.*`, **0** khoá `dict.*`. Story này **không có nhánh lỗi mới** ⇒ **0 khoá `MessageKey` mới** (luật Story 1.7).

## Tasks & Acceptance

**Execution:**
- [x] `src-tauri/src/core/glossary/han_viet_suggestion.rs` (mới) -- module **thuần**: `enum HanVietSuggestion { Ready(String), NotChinese, NoReading, DictUnavailable, NotRequested }` + `as_status_str()` (danh mục ĐÓNG, khuôn `Resolution`/`CandidateOrigin`), và `pub fn suggest_han_viet_batch(layers: &DictLayers, disabled: &BTreeSet<String>, terms: &[&str]) -> Vec<HanVietSuggestion>` gọi `lookup_han_viet` **ĐÚNG MỘT LẦN** cho toàn bộ ký tự đã dedupe -- một `enum` năm nhánh thay cho `Option<String>` là thứ duy nhất làm **bốn** lý do rỗng phân biệt được (`NotRequested` cho dấu đã chốt, xem §Design Notes); một lượt gọi cho mỗi thuật ngữ là đúng khuôn N+1 mà `ports/dict_source.rs:120-126` đã viết sẵn lý do để cấm.
- [x] `src-tauri/src/core/glossary/mod.rs` -- `pub mod han_viet_suggestion;` + tái xuất -- đây là chỗ cạnh `glossary/ → dict/` của AD-36 thành hình; ghi lý do vào doc-comment kèm số hiệu AD, vì `scan.rs:12,19` ngay bên cạnh viết ngược lại (tiêm closure) và người đọc sau sẽ hỏi vì sao hai module cùng thư mục chọn khác nhau.
- [x] `src-tauri/src/core/dict/mod.rs` -- sửa doc-comment `:1019-1020` TẠI CHỖ kèm 🔵 + ngày: mệnh đề *“Story 1.17 và 3.7 cần danh sách đầy đủ”* thu hẹp còn **Story 1.17** -- Ice ký 2026-08-22 rằng 3.7 chỉ dùng `primary`; để nguyên là để một mệnh đề đã hết đúng ở lại làm bằng chứng giả cho lượt sau.
- [x] `src-tauri/src/core/glossary/entry.rs` -- `GlossaryMark` mang `han_viet_suggestion: Option<String>` + `han_viet_status: &'static str` (hoặc `HanVietSuggestion` nguyên khối, tuỳ hình dạng `From` gọn hơn); nối doc-comment `:232-242` kèm 🔵 + ngày -- dấu đã chốt **không** tra Hán Việt, xem §Design Notes; không nêu vì sao thì lượt sau sẽ tưởng đó là một thiếu sót.
- [x] `src-tauri/src/core/glossary/store.rs` -- `marks_for_source_text` nhận thêm `layers: &DictLayers` + `disabled: &BTreeSet<String>`, gom `source_term` của **các mục chờ chốt** rồi gọi `suggest_han_viet_batch` **một lần**, điền hai trường mới -- mục đã chốt đã có `translation` nên một lượt tra cho chúng là công vô ích trên đúng đường mở Chương.
- [x] `src-tauri/src/commands/dict.rs` -- nâng `disabled_sources` lên `pub(crate)` kèm một dòng nói ai dùng -- tái dùng, không đúc bản thứ hai: hai phép đọc danh sách nguồn đã tắt là hai nguồn sự thật, và chúng lệch nhau trong im lặng.
- [x] `src-tauri/src/commands/glossary.rs` -- `GlossaryCandidateWire`/`GlossaryMarkWire` mang `han_viet_suggestion` + `han_viet_status`; `glossary_pending_candidates` và `glossary_marks_for_chapter` nhận thêm `layers`/`disabled`; hai vỏ `wire` lấy `DictLayers` qua **`try_state`** -- 🔴 `try_state`, **không** `state()`: `panic = "abort"` biến một `State` chưa manage thành cái chết của tiến trình; **không** `rename_all`.
- [x] `src-tauri/src/lib.rs` -- bảng `invoke_handler` không đổi tên lệnh nào; chỉ xác nhận hai vỏ vừa sửa vẫn đăng ký đúng -- **0** quyền mới trong `capabilities/main.json`.
- [x] `src-tauri/tests/glossary_boundary.rs` -- thêm `suggest_han_viet_batch` vào `QUICK_ADD_SURFACE`; sửa chú thích `:64-67` TẠI CHỖ kèm 🔵 + ngày (hai quần thể 52 và 53, không phải một) -- cổng `:592` đòi một **lời gọi thật**, nên thêm tên mà quên gọi là một cổng đỏ ngay lượt đầu.
- [x] `src-tauri/tests/glossary_han_viet_suggestion_contract.rs` (mới) -- mọi hàng §I/O Matrix ở tầng Rust: `北涼` ⇒ `"Bắc Lương"` · đa âm lấy `primary` · tiếng Anh ⇒ `NotChinese` · thiếu một âm ⇒ `NoReading` · lẫn Hán-Latin ⇒ `NotChinese` · `DictLayers::empty()` ⇒ `DictUnavailable` **khác** `NoReading` · tắt một nguồn ⇒ chữ ĐỔI chứ không mất -- 🔴 ca `DictUnavailable ≠ NoReading` là ca mà cả story đứng lên; thiếu nó thì cây git (`resources/dict/` rỗng) làm mọi ca khác xanh giả.
- [x] `src-tauri/tests/glossary_commands_contract.rs` -- ca đường **nhận một ứng viên có đề xuất**: `glossary_approve_candidate(id, Some("Bắc Lương"), category)` ⇒ đối chứng bằng `SELECT` rằng mục mới `translation IS NOT NULL` (**đã chốt**) và hàng ứng viên `resolution='approved'`; ca **sửa lại** mục đó qua `glossary_update_term` -- đóng hai AC bằng một đường gọi THẬT, không bằng suy luận.
- [x] `src-tauri/tests/glossary_marks_contract.rs` + `dict_boundary.rs` -- cập nhật các ca dựng mark theo hình dạng mới; thêm ca **đồ thị phụ thuộc**: `core/glossary/**` có ít nhất một `use crate::core::dict`, và `core/dict/**` có **0** `core::glossary` (không chu trình) -- AC “có cạnh `glossary/ → dict/`, không tạo chu trình” là mệnh đề duy nhất của story chưa có cổng nào canh; nó phải trở thành một phép kiểm, không một lời khai.
- [x] `src/config/glossary.ts` -- kiểu `GlossaryMark` và `GlossaryCandidate` + hai type guard mang hai trường mới -- 🔴 type guard là chỗ duy nhất biết dây nói thật; `han_viet_status` giữ `snake_case` và kiểm được là một trong **năm** chuỗi đã khai.
- [x] `src/panels/glossaryMarksMap.ts` -- `SegmentTermSpan` chở `hanVietSuggestion` + `hanVietStatus` -- span là thứ dải đọc; cắt mark về toạ độ cục bộ mà đánh rơi đề xuất thì dải phải đi tra lại bằng một vòng IPC thứ hai cho dữ liệu Rust đã cầm sẵn.
- [x] `src/glossaryConfirmStripState.ts` -- `applyTarget` điền `translationInput` bằng `span.hanVietSuggestion` khi có, và phơi `confirmStripSuggestionStatus` cho template; `focusGlossaryConfirmStrip` giữ nguyên luật *“chỉ điền khi ô RỖNG”* nên **vùng chọn thắng đề xuất** -- thao tác người dùng vừa làm đứng trên gợi ý của máy, cùng doctrine mà `inlineStripPriority.ts` đã dùng để cho `glossary_quick_add` thắng.
- [x] `src/GlossaryConfirmStrip.vue` + `src/i18n/vi.json` -- nhãn *âm Hán Việt* cạnh ô nhập khi `han_viet_status === 'ok'`, và một dòng *chưa cài dữ liệu từ điển* khi `'dict_unavailable'`; hai khoá `glossary.confirm.suggestion_*` -- 🔴 hai chuỗi RIÊNG theo trạng thái, đúng tiền lệ `panel.source.han_viet_unknown`/`han_viet_unavailable` của Story 1.16; một ô rỗng câm là đúng lỗi rỗng im lặng.
- [x] `tests/frontend/glossaryConfirmStripSuggestion.test.ts` (mới) -- hàng §I/O Matrix ở tầng frontend: dải mọc ⇒ ô điền sẵn đề xuất · hợp âm khi đang có vùng chọn ⇒ vùng chọn thắng · `dict_unavailable` ⇒ ô rỗng **và** dòng thông báo hiện · đổi mục sang một thuật ngữ không đề xuất được ⇒ ô **rỗng lại**, không giữ chữ của mục trước -- mock `@tauri-apps/api/core` ở đúng biên IPC, khuôn `glossaryConfirmStripTemplate.test.ts`.
- [x] `scripts/check-i18n.mjs` + bảy hằng sàn Rust -- đo lại **sau** khi thêm tệp rồi xét sàn về dải 80–85 %, ghi ngày tại chỗ; đính chính chú thích *“53 tệp”* ở `:288` nếu số thật đã đổi -- sàn là cận DƯỚI nên tệp mới không làm cổng đỏ; không xét lại thì sàn thành vô nghĩa trong im lặng.
- [x] `_bmad-output/implementation-artifacts/deferred-work.md` -- nối `→ ✅ ĐÃ ĐÓNG 2026-08-22 (Story 3.7)` vào mục *nguồn gợi ý “âm Hán Việt”* (`:6374`); nối `→ 🔵 2026-08-22 (Story 3.7)` vào mục *“bạn vừa viết”* (`:6394`) ghi rằng story đã chạy và quyết định năng lực căn chỉnh cụm nằm **ngoài** FR113, **chủ mới: Ice qua correct-course**; mở mục mới cho phần story này không đóng -- 🔴 **không xoá** một mục đã có, và một mục đóng bằng QUYẾT ĐỊNH KHÔNG LÀM phải nói **điều gì đã đổi**, không chỉ *“không cần nữa”*.

**Acceptance Criteria:**
- Given `.githooks/pre-push`, when chạy, then mười một cổng + `npm run test` + `npm run build` + `cargo test --locked` xanh.
  ✅ **ĐÓNG** — chạy thật 2026-08-24, xanh trong 153s. Xem §Verification.
- Given lượt CI sau khi push, when đọc, then **cả** macOS lẫn Windows xanh — `pre-push` chạy trên macOS của Ice và không nói gì về nửa Windows.
  ⚠️ **CHƯA ĐÓNG** — story chưa được push; đây là bước Ice làm sau khi nhận bàn giao.
- Given một Tác phẩm có mục Glossary chờ chốt mang thuật ngữ chữ Hán, when nghiệm thu bằng mắt trên bản dựng thật **có dữ liệu từ điển**, then dải chốt hiện đề xuất đúng chữ, sửa được, và tương phản AA ở cả hai theme.
  ⚠️ **CHƯA ĐÓNG bằng phép nghiệm thu trên bản dựng thật** — môi trường cài đặt (agent CLI)
  không dựng được cửa sổ Tauri thật với dữ liệu `.db` thật. Đã đóng được vế CẤU TRÚC + DỮ LIỆU
  THẬT (fixture `.db`, không phải bịa): `glossary_han_viet_suggestion_contract.rs` (8 ca)
  dựng fixture SQLite THẬT qua `rusqlite`, đo đúng chữ đề xuất (`北涼` ⇒ `"Bắc Lương"`), đo tắt
  nguồn đổi chữ (`"Bối"` → `"Bắc"`), đo `DictLayers::empty()` ⇒ `DictUnavailable`.
  `glossaryConfirmStripSuggestion.test.ts` (5 ca) mount `GlossaryConfirmStrip.vue` thật, đo ô
  nhập điền sẵn, nhãn hiện, dòng thông báo hiện. `check:tokens` Kiểm C xanh cho
  `.gcs-suggestion-label` — ghi nợ đo bằng mắt cho Ice ở `deferred-work.md`.
- Given bộ e2e, when chạy tay `npm run test:e2e`, then không hồi quy; và ghi rõ **bao nhiêu spec** chạm bề mặt story này dựng — *“e2e xanh”* không có nghĩa bề mặt MỚI đã được nghiệm thu.
  ⚠️ **CHƯA CHẠY trong lượt này** — môi trường cài đặt không có GUI để dựng cửa sổ Tauri thật
  cho bộ e2e. Đếm phần tĩnh: `grep` trên `e2e/specs/**` cho `suggest_han_viet_batch`/
  `han_viet_suggestion`/`glossary.confirm.suggestion` = **0** — **0** spec hiện có chạm bề mặt
  story này dựng. Chạy tay + đọc số hồi quy là việc của Ice.
- Given mỗi phép kiểm mới, when thêm vào, then có một đối chứng **đỏ→xanh THẬT** (gỡ tạm vế đang canh, khẳng định đúng ca dự kiến đỏ, khôi phục) — không lấy lời khai.
  ✅ Thực hiện cho ba mệnh đề trung tâm (xem §Spec Change Log cho chi tiết đầy đủ): ca
  `DictUnavailable ≠ NoReading`, cổng đồ thị phụ thuộc `glossary/ → dict/`, và luật "vùng chọn
  thắng đề xuất". Mọi ca còn lại được xác nhận PASS bằng cách chạy thật ngay sau khi viết
  (không suy luận) trước khi ghép vào bộ đầy đủ.

## Spec Change Log

### 2026-08-24 — thực thi

**Quyết định thực thi không tường minh trong spec, ghi lại để lượt sau đọc được lý do:**

- **`applyTarget` (`glossaryConfirmStripState.ts`) ĐIỀN đề xuất vào `translationInput` NGAY
  khi mục đổi, và `focusGlossaryConfirmStrip` tổng quát hoá luật "chỉ điền khi ô RỖNG" thành
  "chỉ điền khi ô CHƯA BỊ NGƯỜI DÙNG SỬA" (so `translationInput.value` với `target.
  hanVietSuggestion ?? ''`, không còn so với `''` trần).** Spec (Tasks & Acceptance) viết
  *"`focusGlossaryConfirmStrip` giữ nguyên luật 'chỉ điền khi ô RỖNG'"* — đọc CHỮ ĐEN thì hai
  mệnh đề (áp `applyTarget` điền sẵn NGAY, và giữ nguyên hệt luật cũ) mâu thuẫn: nếu
  `applyTarget` điền "Bắc Lương" vào ô lúc mọc, ô KHÔNG còn rỗng khi người dùng bấm hợp âm với
  một vùng chọn — luật `=== ''` cũ sẽ chặn vùng chọn, trái §I/O Matrix *"Vùng chọn THẮNG đề
  xuất"* (mệnh đề ĐÓNG BĂNG, không sửa được). Đã chọn generalize luật thay vì phá mệnh đề đóng
  băng: "rỗng" mở rộng thành "đang mang đúng giá trị mà `applyTarget` vừa đặt" — với một mục
  KHÔNG có đề xuất, giá trị đó VẪN là `''`, nên luật cũ là một trường hợp riêng của luật mới,
  không bị thay thế. Đối chứng đỏ→xanh THẬT (gỡ generalize, ép so `=== ''`): ca *"vùng chọn
  THẮNG đề xuất"* của `glossaryConfirmStripSuggestion.test.ts` đỏ đúng dự kiến (nhận
  `'Tiêu Viêm'` thay vì `'Bản dịch đã chọn'`) → khôi phục → xanh lại.
- **`commands::glossary::glossary_pending_candidates` gọi `suggest_han_viet_batch` TRỰC TIẾP
  (không qua `marks_for_source_text`), còn `glossary_marks_for_chapter` gọi nó GIÁN TIẾP qua
  `marks_for_source_text`.** Spec không nêu tường minh hai đường gọi khác nhau cho cùng một
  hàm — quyết định này giải đúng ràng buộc kép của Task 109/115/117: (a) Task 113 đòi
  `marks_for_source_text` tự gom `source_term` của các mục CHỜ CHỐT rồi gọi hàm MỘT LẦN (bảng
  ứng viên không đi qua đường đó — nó không phải một dấu khớp văn bản); (b) cổng
  `commands_glossary_calls_the_new_quick_add_surface_not_the_forbidden_one`
  (`glossary_boundary.rs:592`) đòi `commands/glossary.rs` chứa một lời gọi THẬT tới MỌI tên
  trong `QUICK_ADD_SURFACE`, và `suggest_han_viet_batch` chỉ xuất hiện ở đó nếu ít nhất MỘT
  hàm trong `commands/glossary.rs` gọi trực tiếp — `glossary_pending_candidates` là chỗ tự
  nhiên (nó cần đề xuất cho `GlossaryCandidateWire`, một kiểu không có mặt trong
  `marks_for_source_text`).
- **`marks_for_source_text` gom `source_term` từ tập SPAN ĐÃ CHỌN (sau `resolve_overlaps`),
  không từ toàn bộ `payload` đã phân giải hai tầng.** Một mục bị một span dài hơn đè lên
  (§Design Notes của Story 3.4: "dài nhất thắng") không bao giờ ra dấu — tra Hán Việt cho nó
  là công vô ích trên đúng đường mở Chương mà NFR2 canh.
- **`GlossaryMark`/`GlossaryMarkWire` mang HAI trường phẳng (`han_viet_suggestion: Option<
  String>` + `han_viet_status: &'static str`), không ôm nguyên `HanVietSuggestion`.** Task 112
  cho phép cả hai hình dạng ("tuỳ hình dạng `From` gọn hơn") — hai trường phẳng khớp thẳng
  hình dạng JSON trên dây (`GlossaryMarkWire`/`GlossaryCandidateWire` đã phẳng sẵn cho mọi
  trường khác), không cần một `#[serde(tag = …)]` cho enum bốn/năm nhánh chỉ để bóc lại thành
  hai trường ở tầng `From`.

**Đối chứng đỏ→xanh THẬT cho ba mệnh đề CẤU TRÚC (không phải hành vi UI), chạy 2026-08-24:**

1. `zero_layers_loaded_is_dict_unavailable_not_no_reading` (ca mà cả story đứng lên) — gỡ tạm
   nhánh `if !layers_loaded { return DictUnavailable }` khỏi `suggest_for_term` ⇒ ca đỏ đúng dự
   kiến (`NoReading` thay vì `DictUnavailable`) → khôi phục → 8/8 ca của
   `glossary_han_viet_suggestion_contract.rs` xanh lại.
2. `glossary_depends_on_dict_and_the_reverse_edge_still_does_not_exist` (cổng đồ thị phụ
   thuộc, Task 120) — đổi tạm CẢ HAI dòng `use crate::core::dict::…` (trong
   `han_viet_suggestion.rs` VÀ `store.rs`) thành `use super::super::dict::…` (biên dịch y hệt,
   chỉ đổi CHUỖI nguồn) ⇒ ca đỏ đúng dự kiến (0 tệp mang chuỗi `use crate::core::dict`) →
   khôi phục → 16/16 ca của `dict_boundary.rs` xanh lại.
3. Luật "vùng chọn thắng đề xuất" — xem mục Spec Change Log ngay trên (đối chứng đã ghi tại
   chỗ, không lặp lại ở đây).

### 2026-08-24 — vòng rà ba lớp (Bước 4), bốn bản vá

Ba lớp rà chạy song song trên diff đầy đủ (3.217 dòng, gồm cả ba tệp mới).
`<frozen-after-approval>` **không bị chạm** — không phát hiện nào đòi đổi §I/O Matrix, nên
**0 `intent_gap`, 0 `bad_spec`**, và `review_loop_iteration` ở nguyên **0**.

🔴 **Phát hiện nặng nhất, và nó là một lỗ TÔI đã bỏ sót ở vòng rà bảng I/O của Bước 3.**
Lượt rà Bước 3 vá chỗ nối `marks_for_source_text` nhưng **bỏ sót chỗ nối thứ hai** —
`commands::glossary::glossary_pending_candidates`. Đo lúc đó: `grep -n "han_viet"
glossary_commands_contract.rs` = **0**, và cả **5** lời gọi `glossary_pending_candidates`
trong tệp đó truyền `DictLayers::empty()`. Cùng lớp lỗ, khác bề mặt — và tôi đã tự tuyên bố
"bảng I/O đã đóng" trong khi một nửa đường sản phẩm chưa ai canh.
Vá: `each_pending_candidate_row_carries_the_suggestion_of_its_own_source_term`, với **BA**
hàng chờ mang **ba kết cục khác nhau** (`ok` · `not_chinese` · `no_reading`) — mọi fixture cũ
chỉ chèn MỘT ứng viên, nên một lượt ghép LỆCH là vô hình theo cấu trúc, không phải vì may.
**Đối chứng đỏ→xanh THẬT:** thay lượt tính đề xuất bằng một mảng `NotRequested` ⇒ đúng ca mới
đỏ (1/13) — **và `glossary_commands_contract.rs` vẫn xanh trọn 15/15**. Khôi phục ⇒ khớp lại
**từng byte** (`diff` rỗng), 13/13 xanh.

**Bốn bản vá:**

1. **`commands/glossary.rs` — bỏ `zip` theo VỊ TRÍ, ghép theo KHOÁ.** `rows.into_iter().
   zip(suggestions)` đúng cặp CHỈ KHI hai vế cùng độ dài và cùng thứ tự, và
   `debug_assert_eq!` ngay trên **không đỡ được ở bản phát hành** — `debug_assert` biên dịch
   thành hư vô ở release, nên lệch độ dài ⇒ `zip` **cắt cụt** phần đuôi trong im lặng, và một
   lượt sắp lại `rows` ⇒ đề xuất của thuật ngữ này dán lên thuật ngữ khác. Cả hai là đúng lớp
   *sai IM LẶNG* (`AGENTS.md:46`). Nay khoá bằng `source_term` (an toàn nhờ `UNIQUE
   (source_term)`, `schema.rs:432`), đúng khuôn `marks_for_source_text` đã dùng.
   *(Hai lớp rà độc lập cùng nêu — gộp làm một.)*
2. **`GlossaryConfirmStrip.vue` — nhãn *"Âm Hán Việt"* ra khỏi `<label>`.** Nó nằm trong
   `<span class="gcs-field-label">`, mà tên khả truy cập của `<input>` tính từ **toàn bộ** nội
   dung văn bản của `<label>` bọc nó ⇒ trình đọc màn hình đọc ô nhập thành *"Bản dịch Âm Hán
   Việt"*, hai nhãn dính làm một cụm vô nghĩa. Nay là một chip độc lập ở hàng tiêu đề, cạnh
   `.gcs-tier`, đọc được riêng. Epic 3 đòi *"toàn bộ thao tác dùng được bằng bàn phím"* — một
   nhãn hỏng cho trình đọc màn hình là cùng một cửa.
3. **`core/glossary/store.rs` — nhánh phòng thủ dư nay TỰ NÓI RA.** Lời thú nhận *"phòng thủ
   dư"* trước đó chỉ sống ở `deferred-work.md`; người đọc `store.rs` một mình không có cách
   nào biết. Nay ghi tại chỗ, kèm số đo (vô hiệu riêng nhánh ⇒ 0 ca đỏ) và kèm hệ quả thật
   của việc gỡ dòng `.filter(...)`.
4. **Hai mục hoãn có chủ Ice** (xem §Verification) — không vá, vì cả hai đòi một phán quyết.

**Ba nhóm phát hiện bị BÁC, nói rõ lý do thay vì im lặng:**
- *Windows CI · e2e · NFR2 · nghiệm thu bằng mắt · `sources_used` không phơi ra* — đều ĐÃ có
  mục hoãn có chủ trước khi vòng rà chạy. Một phát hiện lặp lại một mục đã ghi không phải một
  phát hiện mới.
- *Phép kiểm "không client mạng" là một lượt `grep` yếu* — đúng, và **giới hạn đó đã được ghi
  thành chữ ngay trong doc-comment của chính ca đó** trước khi lớp rà đọc nó. Đây là cách kho
  yêu cầu xử lý một giới hạn: ghi ra, không giấu.
- *`pending_terms` không dedupe theo `source_term`* — `lookup_han_viet` đã dedupe ở tầng KÝ
  TỰ (`mod.rs:1109`), nên phần dư chỉ là vài lượt chèn `BTreeMap`. Bác vì nhiễu.

## Design Notes

**Vì sao tính LÚC ĐỌC chứ không lưu một cột.** Một cột `suggested_translation` trong `project.db` là một **bản chép** của dữ liệu từ điển — thứ AD-36 cấm bằng đúng chữ *“không cài lại”*. Nó còn sai theo một cách không ai sẽ lần ra: đề xuất phụ thuộc **bộ lọc nguồn đang bật** và **tập lớp `.db` đang gắn**, cả hai đổi được sau lượt quét. Một giá trị đã ghi xuống đĩa sẽ tiếp tục khẳng định một âm đến từ một nguồn người dùng vừa tắt. Cộng thêm: bảng ứng viên khoá `UNIQUE(source_term)` (`schema.rs:432`), nên một cột đề xuất kéo theo câu hỏi khoá mới mà `deferred-work.md:5708` đã ghi là việc của Epic 8.

**Vì sao dấu ĐÃ CHỐT không mang đề xuất.** Mục đã chốt đã có `translation` thật; một đề xuất cho nó không có chỗ tiêu thụ nào và sẽ trả giá bằng một lượt tra trên đúng đường mở Chương. Trường để `null` + **`NotRequested`** — một nhãn RIÊNG cho ca *“chưa hỏi”*, không mượn `NotChinese`: `聽潮閣` đã chốt vẫn là chữ Hán, và gắn cho nó nhãn *“không phải tiếng Trung”* là dựng một lời nói dối nhỏ mà lượt sau sẽ đọc như sự thật.

**Vì sao vị từ là hình dạng chuỗi, không phải ngôn ngữ Tác phẩm.** `glossary_candidate` **không có cột ngôn ngữ** và ngôn ngữ chỉ tồn tại lúc quét (`scan.rs:152-160` nhận `lang` làm tham số) rồi biến mất. Kể cả nếu có, `src-tauri/AGENTS.md:34` đã chốt: bôi đen `API` trong một truyện tiếng Trung mà lọc theo ngôn ngữ Tác phẩm cho **0 hàng** dù mục có thật. Cùng lỗi ấy ở đây sẽ đề xuất âm Hán Việt cho một chuỗi Latin.

**Vì sao không khai một `source_code` cho đề xuất.** `priority_order` chọn lớp thắng **theo từng ký tự** (`mod.rs:1080-1098`), nên `聽潮閣` có thể rút ba âm từ ba lớp khác nhau. *“Nguồn của đề xuất”* không phải một giá trị xác định được, và bịa một cái là đúng thứ FR31 tồn tại để chống. ⚠️ **Ghi ra chỗ yếu:** nếu Ice muốn nhãn nguồn trên dải, hình dạng đúng là một danh sách `sources_used` cho cả cụm — `HanVietLookup` đã trả sẵn trường đó, chỉ chưa nối.

## Verification

**Commands:**
- `cargo test --locked` (trong `src-tauri/`) -- expected: 0 failed; `glossary_han_viet_suggestion_contract.rs` mới chạy, `glossary_marks_contract.rs`/`glossary_commands_contract.rs` tăng số ca theo §Tasks.
  ✅ **Chạy thật 2026-08-24**: 0 failed trên **579 ca** across 22 tệp test/unittest (+1 tệp mới
  `glossary_han_viet_suggestion_contract.rs`, 8 ca). `glossary_marks_contract.rs` không đổi số
  ca (17, chỉ đổi CHỮ KÝ mọi lời gọi hiện có). `glossary_commands_contract.rs` 14 → **15** ca
  (+1: `glossary_approve_candidate_with_a_suggestion_creates_a_confirmed_entry`).
  `glossary_boundary.rs` vẫn 11 ca, xanh với `QUICK_ADD_SURFACE` 7 → 8 phần tử.
  `dict_boundary.rs` 16 ca (+2 mới: cổng đồ thị phụ thuộc + đối chứng dương của nó).
- `npm run test` -- expected: 0 failed; tệp vitest mới chạy.
  ✅ **Chạy thật 2026-08-24**: 35 tệp / **402 ca**, 0 failed (+1 tệp mới
  `glossaryConfirmStripSuggestion.test.ts`, 5 ca; bảy tệp có sẵn cập nhật fixture cho hai
  trường mới, không đổi số ca).
- `npm run build` -- expected: xanh (chạy **trước** `cargo test`, thiếu `dist/` thì `cargo test` gãy ở khâu biên dịch).
  ✅ **Chạy thật 2026-08-24** — `vue-tsc --noEmit` (cả hai tsconfig) + `vite build` xanh, 0 lỗi
  kiểu. Hai cảnh báo `INEFFECTIVE_DYNAMIC_IMPORT` của Rollup ĐÃ CÓ TỪ TRƯỚC (ghi nhận từ Story
  3.6), không liên quan tệp story này chạm.
- `.githooks/pre-push` -- expected: mười một cổng + vitest + build + `cargo test --locked` xanh.
  ✅ **Chạy thật 2026-08-24** — xanh trong **153s** (`deps` · `tokens` · `i18n` · `commands` ·
  `layout` · `panel-refs` · `dict` · `dict-manifest` · `lint` · `gates` · `debt-owner` · `test`
  · `build` · `cargo test`).

**Rà bảng I/O (Matrix Test Audit) — chạy ĐỘC LẬP 2026-08-24, không lấy lời khai của lượt thực thi:**

15 hàng §I/O Matrix đối chiếu với phép kiểm ĐÃ CHẠY VÀ XANH. Lượt rà tìm thấy **bốn hàng
không có phép kiểm nào**; cả bốn đã được vá trong chính lượt rà, không hoãn.

- ✅ **Mười một hàng** đã có ca phủ từ lượt thực thi: bảy ca đầu của
  `glossary_han_viet_suggestion_contract.rs`, ba ca dải ở `glossaryConfirmStripSuggestion.test.ts`,
  và `glossary_approve_candidate_with_a_suggestion_creates_a_confirmed_entry`.
- 🔴 **Hàng *"Dải chốt mọc"* (nửa Rust) — LỖ THẬT, đã vá.** Mọi ca của lượt thực thi gọi
  **thẳng** `suggest_han_viet_batch`; **không ca nào** đi qua `marks_for_source_text`, tức qua
  đúng đoạn quyết định *đề xuất nào rơi vào dấu nào*. Cả 17 ca của `glossary_marks_contract.rs`
  truyền `DictLayers::empty()`, nên trong toàn bộ bộ nghiệm thu Rust **không một dấu nào từng
  mang một đề xuất khác `None`**. Thêm
  `a_pending_chinese_mark_carries_the_suggestion_through_marks_for_source_text`.
  **Đối chứng đỏ→xanh THẬT:** thay cả khối `suggestion` bằng hằng `NotRequested` ⇒ đúng ca mới
  đỏ (1/12) — **và `glossary_marks_contract.rs` vẫn XANH TRỌN 17/17**, tức lỗ là thật, không
  phải một ca thừa. Khôi phục ⇒ 12/12 + 17/17 xanh, tệp khớp lại **từng byte** (`diff` rỗng).
- 🔴 **Hàng *"Dấu ĐÃ CHỐT"* — LỖ THẬT, đã vá.** Thêm
  `a_confirmed_chinese_mark_is_not_requested_never_not_chinese` (cùng thuật ngữ chữ Hán
  `北涼`, cùng fixture đã chứng minh nó tra được — nên `"not_chinese"`/`"no_reading"` ở đây sẽ
  là một lời nói dối đọc được).
  ⚠️ **Và lượt đối chứng lật một mệnh đề tôi tưởng đúng:** vô hiệu **riêng** nhánh
  `if is_confirmed` ⇒ **0 ca đỏ**. Nhánh đó là **phòng thủ DƯ**, không phải chỗ gánh — vệ thật
  là `.filter(|entry| !entry.is_confirmed())` lúc dựng `pending_terms`, vì một thuật ngữ đã
  chốt không bao giờ vào `suggestion_by_term` nên `unwrap_or(&NotRequested)` đã đỡ sẵn. Gỡ
  **CẢ HAI** vệ ⇒ đúng ca mới đỏ (1/12). Khôi phục ⇒ xanh lại.
  ⚠️ **GIỚI HẠN THẬT, ghi ra thay vì để người sau tưởng đã được xét:** hàng này có HAI vế và
  chỉ vế đầu được kiểm bằng máy. Vế *"**0 lượt tra** Hán Việt cho dấu đã chốt"* — một mệnh đề
  về CÔNG VÔ ÍCH, không về giá trị trả về — **không phép kiểm nào quan sát**, vì không ca nào
  đếm số lượt gọi `lookup_han_viet`. Nó đứng bằng đúng một dòng `.filter(...)`.
- 🔴 **Hàng *"Sửa một mục đã vào từ đề xuất"* — không có phép kiểm, đã vá** bằng phần nối tiếp
  trong `glossary_approve_candidate_with_a_suggestion_creates_a_confirmed_entry`: gọi
  `glossary_update_term` ngay trên mục vừa sinh rồi đối chứng bằng một lượt tra lại (`id`
  không đổi, `translation` bị đè, `note` mới). Kiểm tại chỗ thay vì một ca rời — tiền đề cần
  canh là *"mục VỪA sinh từ đề xuất"*, tách ra là đánh mất chính tiền đề đó.
- 🔴 **Hàng *"Một lớp `.db` hỏng lúc tra"* — không có phép kiểm, đã vá** bằng
  `a_corrupt_layer_is_skipped_and_the_remaining_layers_still_answer` (đặt một tệp `.db` KHÔNG
  phải SQLite cạnh lớp nền thật).
- 🟡 **Hàng *"Ngắt kết nối mạng"* — vá được MỘT NỬA.** Thêm
  `the_suggestion_path_names_no_network_client_so_it_cannot_depend_on_the_network`: phép kiểm
  **TĨNH** trên văn bản nguồn, khẳng định module đề xuất không gọi tên một client mạng nào.
  ⚠️ Đây **không** phải một phép đo với cáp mạng bị rút, và tôi ghi giới hạn đó ngay trong
  doc-comment của ca. Vế còn lại (mọi đầu vào là một tệp `.db` cục bộ) đứng bằng việc cả 12 ca
  chạy trên fixture trên đĩa.

**Số ca sau lượt rà:** `glossary_han_viet_suggestion_contract.rs` 8 → **12**;
`glossary_commands_contract.rs` 15 ca (ca approve dài thêm, không thêm ca).

🔵 **CẬP NHẬT 2026-08-24 (sau vòng rà Bước 4).** Lượt rà bảng I/O ở trên **chưa đủ**: nó vá
chỗ nối `marks_for_source_text` rồi tuyên bố đóng, trong khi chỗ nối thứ hai
(`glossary_pending_candidates`) chưa ai canh — xem §Spec Change Log, mục vòng rà. Sau bản vá:
`glossary_han_viet_suggestion_contract.rs` 12 → **13** ca.

**Ba lượt chạy `.githooks/pre-push` THẬT trong phiên này, cả ba exit 0** *(số giây khác nhau
vì là ba lượt chạy riêng biệt trên cùng máy, không phải một chỗ lệch — cùng bài học đã ghi ở
Story 3.6)*:
| Lượt | Cây | Kết quả |
|---|---|---|
| 1 | như agent thi hành giao | xanh **155s** |
| 2 | sau 4 bản vá của lượt rà bảng I/O | xanh **125s** |
| 3 | sau 4 bản vá của vòng rà Bước 4 | xanh **143s** |

**Tổng ca Rust cuối:** `cargo test --locked` đếm được **584** ca xanh, 0 failed
*(579 như agent giao → 583 sau lượt rà bảng I/O → 584 sau vòng rà Bước 4; mỗi bước cộng đúng
số ca đã thêm, không có ca nào biến mất giữa chừng).*

**Manual checks (if no CLI):**
- Trên bản dựng thật **có dữ liệu từ điển** (cây git rỗng `resources/dict/`): dải chốt cho một thuật ngữ chữ Hán điền sẵn đúng chữ; tắt một nguồn ⇒ chữ đổi chứ không biến mất; gỡ hết `.db` ⇒ dòng *chưa cài dữ liệu từ điển* hiện, ô rỗng.
- Toàn bộ luồng bằng bàn phím: `Mod+Alt+C` vào dải khi đã có đề xuất ⇒ chữ đề xuất **vẫn còn**, con trỏ đặt được để sửa.

## Suggested Review Order

**Cạnh AD-36 — thứ cả story đứng lên, đọc trước tiên**

- Điểm vào: cạnh `glossary/ → dict/` thành hình, `use` thật chứ không closure như `scan.rs`.
  [`mod.rs:178`](../../src-tauri/src/core/glossary/mod.rs#L178)

- Hàm thuần duy nhất của story; một lượt `lookup_han_viet` cho CẢ lô, không N lượt.
  [`han_viet_suggestion.rs:151`](../../src-tauri/src/core/glossary/han_viet_suggestion.rs#L151)

**Bốn lý do RỖNG phải phân biệt được — lớp lỗi trung tâm của kho**

- Danh mục đóng năm nhánh thay cho một `Option<String>` trần.
  [`han_viet_suggestion.rs:45`](../../src-tauri/src/core/glossary/han_viet_suggestion.rs#L45)

- Ca mà cây git rỗng làm thành ca thường gặp nhất: chưa cài ≠ không có âm.
  [`glossary_han_viet_suggestion_contract.rs:248`](../../src-tauri/tests/glossary_han_viet_suggestion_contract.rs#L248)

- Hai trường mới trên dây, giữ `snake_case`, kèm lý do vì sao dấu đã chốt bỏ trống.
  [`entry.rs:275`](../../src-tauri/src/core/glossary/entry.rs#L275)

**HAI chỗ nối — đọc kỹ nhất ở đây, cả hai đều từng không có ai canh**

- Marks: gom thuật ngữ chờ chốt SAU `resolve_overlaps`; nhánh `is_confirmed` là phòng thủ dư và nó tự nói ra.
  [`store.rs:915`](../../src-tauri/src/core/glossary/store.rs#L915)

- Ứng viên: ghép theo KHOÁ, không theo vị trí — `debug_assert` chết ở bản phát hành.
  [`glossary.rs:393`](../../src-tauri/src/commands/glossary.rs#L393)

- Gỡ chỗ nối marks ⇒ ca này đỏ, còn 17 ca marks cũ vẫn xanh trọn.
  [`glossary_han_viet_suggestion_contract.rs:357`](../../src-tauri/tests/glossary_han_viet_suggestion_contract.rs#L357)

- Ba hàng chờ, ba kết cục khác nhau — một lượt ghép lệch không thể tình cờ đúng.
  [`glossary_han_viet_suggestion_contract.rs:515`](../../src-tauri/tests/glossary_han_viet_suggestion_contract.rs#L515)

**Đồ thị phụ thuộc — mệnh đề AC duy nhất trước đó chưa có cổng nào canh**

- Cạnh thuận tồn tại, cạnh nghịch vẫn là 0 ⇒ không chu trình.
  [`dict_boundary.rs:1132`](../../src-tauri/tests/dict_boundary.rs#L1132)

- Cổng mới phải chứng minh nó ĐỎ ĐƯỢC — đối chứng dương gài sẵn một vi phạm.
  [`dict_boundary.rs:1188`](../../src-tauri/tests/dict_boundary.rs#L1188)

- Bề mặt cho phép lên 8; cổng đòi một lời gọi THẬT, không chỉ một cái tên.
  [`glossary_boundary.rs:176`](../../src-tauri/tests/glossary_boundary.rs#L176)

**Dải chốt — chỗ đề xuất gặp người dùng**

- Điền sẵn lúc danh tính mục đổi; vùng chọn vẫn thắng đề xuất.
  [`glossaryConfirmStripState.ts:176`](../../src/glossaryConfirmStripState.ts#L176)

- Nhãn nguồn là chip ĐỘC LẬP ở hàng tiêu đề — trong `<label>` thì trình đọc màn hình đọc dính hai nhãn.
  [`GlossaryConfirmStrip.vue:119`](../../src/GlossaryConfirmStrip.vue#L119)

- Thao tác người dùng vừa làm đứng trên gợi ý của máy.
  [`glossaryConfirmStripSuggestion.test.ts:112`](../../tests/frontend/glossaryConfirmStripSuggestion.test.ts#L112)

**Ngoại vi**

- Type guard là chỗ duy nhất biết dây nói thật — năm chuỗi trạng thái.
  [`glossary.ts:252`](../../src/config/glossary.ts#L252)

- Span cắt về toạ độ cục bộ mà không đánh rơi đề xuất.
  [`glossaryMarksMap.ts:74`](../../src/panels/glossaryMarksMap.ts#L74)

---

## Nhật ký sprint-status

Gỡ nguyên văn từ `sprint-status.yaml` ngày 2026-08-26: tệp đó giữ TRẠNG THÁI, nội dung story
thuộc về tệp này. Không sửa một ký tự.

```
  # 🔵 2026-08-24 — backlog → in-progress (bmad-build, spec đã duyệt ở CHECKPOINT 1).
  # Spec: `3-7-de-xuat-ban-dich-bang-am-han-viet.md`, baseline `dae3a3d3ba2665b64038bf339f7b1f5ffcd47fa3`.
  # Ba quyết định Ice ký 2026-08-24: ① story NỐI vào dải chốt của 3.6 (không chỉ Rust+IPC) —
  # đóng luôn mục nợ `deferred-work.md:6374` mà chủ đã ghi đích danh là Story 3.7 · ② ký tự đa
  # âm chỉ lấy `primary`, KHÔNG phơi `HanVietReading.all` ra dây, và doc-comment
  # `core/dict/mod.rs:1019` phải sửa TẠI CHỖ vì mệnh đề "3.7 cần danh sách đầy đủ" hết đúng ·
  # ③ nợ "bạn vừa viết" (`:6394`) KHÔNG dựng ở đây — chuyển chủ sang Ice qua correct-course,
  # vì nó đòi một phép căn chỉnh cụm chưa xuất hiện ở bất kỳ epic nào và 0 AC của 3.7 nhắc nó.
  # 🔴 AD-36 đã viết sẵn cạnh `glossary/ → dict/` cho FR113 — AC không phải điều story tự phát
  # minh. Hôm nay cạnh đó là 0 (`scan.rs` cố ý tiêm closure thay vì `use`); chiều ngược
  # `dict/ → glossary/` cũng 0 ⇒ không chu trình. Đây là chỗ gọi SẢN PHẨM ĐẦU TIÊN của
  # `DictionarySource::han_viet` từ ngoài `commands::dict`.
  # 🔴 Đề xuất tính LÚC ĐỌC, 0 bước migration — lược đồ `project.db` ở nguyên v14. Một cột
  # `suggested_translation` là bản chép dữ liệu từ điển vào `project.db` (AD-36 cấm) và nó lệch
  # câm đúng ngày người dùng tắt một nguồn.
  # 🔴 Rỗng có BỐN lý do, không một: chưa cài từ điển · không phải chữ Hán · có chữ Hán nhưng
  # thiếu âm · dấu đã chốt nên không hỏi. `resources/dict/` RỖNG trong cây git (AD-25) nên ca
  # "chưa cài" là ca thường gặp nhất ở máy dev — thiếu nhánh đó thì mọi ca khác xanh giả.
  # ⚠️ Spec đo 22.087 ký tự tổng / 19.241 tới §Spec Change Log ≈ 6.400–7.700 token so với trần
  # 1.600; Ice ký [K] giữ nguyên phạm vi. NHỎ HƠN cả 3.5 (26.151) và 3.6 (23.090) đo cùng
  # thước. ⚠️ Con số "28.289" ghi cho 3.6 ở khối trên KHÔNG so sánh trực tiếp được — phương
  # pháp đo của nó không được ghi lại, nên đây là hai thước khác nhau, không phải một chỗ lệch.
  # 🔵 2026-08-24 — in-progress → review (bmad-build: dựng xong, CHO ICE KÝ).
  # `.githooks/pre-push` xanh trong 153s (mười một cổng + `npm run test` 402 ca + `npm run
  # build` + `cargo test --locked` 579 ca). Ba đối chứng đỏ→xanh THẬT chạy tay: ca
  # `DictUnavailable ≠ NoReading` · cổng đồ thị phụ thuộc `glossary/ → dict/` · luật "vùng chọn
  # thắng đề xuất". CHƯA ĐÓNG (cần môi trường thật, ghi ở `deferred-work.md`): nghiệm thu bằng
  # mắt trên bản dựng thật có dữ liệu từ điển · NFR2 đo bằng số thật · CI hai nền tảng · bộ e2e
  # (0/? spec chạm bề mặt này).
  # 🔵 2026-08-24 — HAI VÒNG RÀ CHẠY SAU khối trên; khối đó là lời khai của lượt thi hành, và
  # cả hai vòng đều tìm thấy chỗ nó chưa nói tới. Trạng thái `review` không đổi.
  # 🔴 VÒNG 1 (rà bảng I/O, độc lập): 15 hàng đối chiếu — BỐN hàng KHÔNG có phép kiểm nào
  # (dải chốt nửa Rust · dấu ĐÃ CHỐT · sửa mục vào từ đề xuất · lớp `.db` hỏng), một hàng
  # (ngắt mạng) chỉ vá được một nửa. Đối chứng: thay cả khối tính đề xuất trong
  # `marks_for_source_text` bằng một hằng ⇒ `glossary_marks_contract.rs` VẪN XANH TRỌN 17/17.
  # ⚠️ Lượt đối chứng LẬT một mệnh đề: nhánh `if is_confirmed` là PHÒNG THỦ DƯ (vô hiệu riêng
  # nó ⇒ 0 ca đỏ); vệ thật là `.filter(|entry| !entry.is_confirmed())` lúc dựng `pending_terms`.
  # 🔴 VÒNG 2 (ba lớp rà đối kháng, Bước 4): 0 intent_gap, 0 bad_spec ⇒ KHÔNG vòng quay lại,
  # `review_loop_iteration` vẫn 0. Phát hiện nặng nhất là MỘT LỖ CỦA CHÍNH VÒNG 1 — lần thứ hai
  # liên tiếp sau Story 3.6 mà lượt tự kiểm bỏ sót: vòng 1 vá chỗ nối `marks_for_source_text`
  # rồi tuyên bố đóng, trong khi chỗ nối THỨ HAI (`glossary_pending_candidates`) chưa ai canh —
  # `grep han_viet` trong `glossary_commands_contract.rs` = 0, cả 5 lời gọi truyền
  # `DictLayers::empty()`. Đối chứng: gỡ lượt tính đề xuất ⇒ đúng 1 ca mới đỏ, 15/15 ca cũ XANH.
  # 🔴 `debug_assert_eq!` canh độ dài TRƯỚC `zip` chết ở bản phát hành ⇒ lệch độ dài thì `zip`
  # CẮT CỤT im lặng, sắp lại `rows` thì đề xuất dán nhầm thuật ngữ. Nay ghép theo KHOÁ.
  # 🔴 Nhãn "Âm Hán Việt" nằm trong `<label>` ⇒ tên khả truy cập của ô nhập thành "Bản dịch Âm
  # Hán Việt". Nay là chip độc lập ở hàng tiêu đề.
  # ⚠️ HAI MỤC CHỜ ICE PHÁN QUYẾT (`deferred-work.md`) — không phải một dòng vá:
  # ① Tắt/bật một nguồn KHÔNG làm mới dấu Glossary (`dictSourcesState.ts:212` chỉ gọi
  #   `refreshHanViet()`). Story này VỪA TẠO khớp nối đó. Thêm chỗ gọi `refreshGlossaryMarks`
  #   thứ TƯ đụng một câu Ice ký 2026-08-21 (`glossaryMarksState.ts:17-24`).
  # ② `targetsEqual` chỉ so `tier`+`id` — cửa sổ ĐANG ĐÓNG hôm nay, mở ra đúng lúc ① được sửa;
  #   và bản vá hiển nhiên là bản vá SAI (`applyTarget` xoá rồi điền lại ⇒ mất chữ đang gõ).
  # ⚠️ Vế "0 lượt tra Hán Việt cho dấu đã chốt" của hàng I/O *Dấu ĐÃ CHỐT* KHÔNG phép kiểm nào
  # quan sát — đã nối vào mục NFR2, chủ Ice.
  # Ba lượt `.githooks/pre-push` THẬT trong phiên, cả ba exit 0: 155s (cây agent giao) · 125s
  # (sau vòng 1) · 143s (sau vòng 2). `cargo test --locked`: 579 → 583 → **584** ca xanh.
```

