---
title: 'Story 3.6 — Trạng thái chờ chốt và dải mọc chốt lần đầu gặp'
type: 'feature'
created: '2026-08-22'
status: 'done'
review_loop_iteration: 0
baseline_commit: 'a2a0e47c0af4bfbd95647cfc2ca0182d82003c1b'
context:
  - '{project-root}/_bmad-output/implementation-artifacts/epic-3-context.md'
  - '{project-root}/_bmad-output/implementation-artifacts/3-5-quet-ung-vien-khi-nhap-tai-lieu.md'
  - '{project-root}/AGENTS.md'
  - '{project-root}/src-tauri/AGENTS.md'
  - '{project-root}/src/AGENTS.md'
---

<frozen-after-approval reason="human-owned intent — do not modify unless human renegotiates">

## Intent

**Problem:** Trạng thái *chờ chốt* (`glossary_entry.translation IS NULL`, FR114) đã tồn tại trong lược đồ từ Story 3.1 nhưng **không đường sản phẩm nào chốt được nó**: `confirm_translation` (`core/glossary/store.rs:214`) và `approve_candidate` (`candidate_store.rs:238`) đều có **0 chỗ gọi sản phẩm**. Một mục chờ chốt hôm nay được đánh dấu trên lưới rồi nằm đó mãi — và vì `entries_eligible_for_injection` lọc bỏ nó (`store.rs:520-529`), mỗi mục chờ chốt là một lỗ hổng chạy tiếp qua mọi câu sau.

**Approach:** Khi con trỏ vào một câu chứa thuật ngữ **chờ chốt**, một **dải** mọc ở chân Workspace hỏi bản dịch, **đẩy** vùng làm việc co lại chứ không phủ lên. Trả lời ⇒ mục khoá thành *đã chốt* và không bao giờ hỏi lại. Dải đi qua một **sổ ưu tiên có tên** để không bao giờ có hai dải cùng mọc.

## Boundaries & Constraints

**Always:**
- Dải sống ở **chân `.shell`, ngay trên `<StatusBar />`** — cùng slot với `GlossaryQuickAdd` (`App.vue:286`), `flex: none` trong luồng thường. **Không** `position: fixed`, **không** `z-index`, **không** `box-shadow` (Kiểm F của `check:tokens` không có đường thoát cho hai cái sau). *Ice ký 2026-08-22.*
- Dải **không cướp tiêu điểm** khi mọc. Vào dải bằng một hợp âm; ra khỏi dải trả tiêu điểm + vùng chọn cũ, đúng khuôn `glossaryQuickAddState.ts:219-263`.
- Chốt đi qua **một** lượt ghi rời rạc (`Store::write`), **không** qua bộ đệm gõ — cùng luật với FR94/FR58 (`AGENTS.md` §Known pitfalls).
- Mọi văn bản qua `t()`/`tError()`. Adapter IPC ở `src/config/glossary.ts` **không bao giờ ném** — hình dạng ba trạng thái.
- Sau một lượt chốt thành công, **nạp lại marks** qua `refreshGlossaryMarks(...)` — đây là chỗ gọi thứ **ba** được phép, và doc-comment `glossaryMarksState.ts:17-24` phải được sửa TẠI CHỖ kèm 🔵 + ngày.
- Mọi số đếm và mọi nhánh trượt **báo ra**. Một dải không mọc phải phân biệt được với "không có thuật ngữ chờ chốt".

**Ask First:**
- Bất kỳ phụ thuộc mới nào (NFR15), kể cả bật một `feature` chưa bật.
- Nếu vế "đẩy nội dung xuống" đo được một frame vượt **50 ms** (NFR2) lúc dải mọc/thu: dừng, trình số, hỏi trước khi thêm bất kỳ lớp đệm nào.
- Nếu phải thêm một `.rs` mới dưới `src-tauri/src/**`: **sáu** hằng sàn Rust phải xét lại cùng lúc (xem §Code Map) — trình số trước.

**Never:**
- Không sửa `epics.md`/`prd.md`. Ba nguồn gợi ý của mockup (*bạn vừa viết* · *âm Hán Việt* · *TM*) — hai nguồn sau thuộc Story 3.7 và Epic 7, ghi nợ có chủ, không dựng trước.
- Không component duyệt bảng chờ (Story 3.8). Story này chỉ thêm **vỏ IPC** ghi.
- Không hoàn tác (`⌘Z` trong mockup) — mô hình hoàn tác là `ad-brief-2026-08-17-mo-hinh-hoan-tac.md`, không phải một dòng ở đây.
- Không chèn một track hàng vào subgrid của `GridPanel.vue` — Ice đã loại đường đó 2026-08-22.
- Không thêm phần tử thứ tư vào `MODE_IDS` (`commands/index.ts:38`); không đăng ký `FocusOwner` — dải không phải panel (tiền lệ `GlossaryQuickAdd.vue:12-13`).
- Không nới `GLOSSARY_ONLY_SURFACE`. `confirm_translation` bị cấm gọi từ `commands/**` — sửa CHỮ KÝ bằng một hàm bọc mới, đúng tiền lệ `add_manual_term` bọc `insert_manual_entry`.

## I/O & Edge-Case Matrix

| Scenario | Input / State | Expected Output / Behavior | Error Handling |
|---|---|---|---|
| Câu có một thuật ngữ chờ chốt | Con trỏ vào segment mang span `isConfirmed = false` | Dải mọc, mang `source_term` và tầng; tiêu điểm **vẫn ở ô gõ** | N/A |
| Câu có hai thuật ngữ chờ chốt | Hai span `isConfirmed = false` | Dải hỏi span **trái nhất trước**; chốt xong dải kế mọc **cùng slot**, không thao tác nào, vị trí không nhảy | N/A |
| Chốt xong | Người dùng gửi một chuỗi không rỗng | `translation` ghi xuống, marks nạp lại, span thành `isConfirmed = true`, dải thu | Lượt ghi trượt ⇒ dải **ở lại**, hiện lỗi qua `tError()`, không đóng |
| Hỏi lại sau khi chốt | Con trỏ quay lại đúng câu đó | **0** dải — cấu trúc, không phải một sổ nhớ: `translation IS NOT NULL` ⇒ hết chờ chốt | N/A |
| Để sau | `Esc` trên dải | Dải thu, `source_term` đó **không hỏi lại trong Chương đang mở**; đổi Chương thì hỏi lại | N/A |
| Chuỗi rỗng / toàn khoảng trắng | Người dùng gửi `"   "` | **0** lượt ghi, dải ở lại kèm lỗi đọc được — `CHECK` phía SQL cũng chặn, nhưng dải không được để người dùng chạm tới nó | Chặn ở tầng dải, thông báo qua `t()` |
| Tiếng Anh biến thể hình thái | Bề mặt là `dragons`, `source_term` là `dragon` | Dải hỏi **`dragon`**, ghi vào đúng mục `dragon` | Bề mặt cắt từ văn bản **không** dùng làm khoá ghi |
| Dải "Thêm thuật ngữ" đang mở | `quickAddIsOpen === true` và có span chờ chốt | **Đúng một** dải hiện — `glossary_quick_add` thắng (thao tác người dùng vừa yêu cầu); dải chốt mọc ngay khi dải kia đóng | N/A |
| Đổi Chương giữa chừng | Dải đang mở, người dùng chuyển Chương | Dải thu, sổ "để sau" xoá, tiêu điểm không rơi về `body` | N/A |
| Câu không có thuật ngữ chờ chốt | Mọi span `isConfirmed = true` hoặc 0 span | **0** dải, **0** lượt IPC | N/A |
| Marks chưa nạp xong | `glossaryMarksHaveLoaded() === false` | **0** dải — chưa biết thì không khẳng định "không có" | N/A |
| Nhận một ứng viên không có đề xuất | `glossary_approve_candidate(id, translation = null, category)` | Một mục Glossary mới, `translation IS NULL`, `term_origin` suy từ `candidate_origin` của hàng ứng viên | `id` không khớp hàng nào ⇒ lỗi mang `message_key`, không `Ok` rỗng |
| Nhận lại một ứng viên đã quyết | Hàng ứng viên đã `approved` hoặc `rejected` | **0** mục mới, **0** cột đổi — hai bảng không được phép nói ngược nhau | `already_decided_error` lên tới dây dưới dạng `message_key`, không nuốt |

</frozen-after-approval>

## Code Map

**Trạng thái chờ chốt — nửa Rust ĐÃ ĐỦ, thiếu đúng đường gọi**
- `src-tauri/src/core/glossary/entry.rs:4-8` — 🔴 `is_confirmed()` là vị từ DUY NHẤT; không cột `status`. `:195-211` struct 7 trường, chỉ `translation` là `Option`.
- `core/glossary/store.rs:214-230` `confirm_translation(store, id, translation)` — nhận CẢ chiều "chốt lần đầu" lẫn "sửa mục đã chốt"; `UPDATE` 0 hàng đã là lỗi (không `Ok(())` rỗng). ⚠️ **0 chỗ gọi sản phẩm**, và tên nó nằm trong `GLOSSARY_ONLY_SURFACE` ⇒ `commands/**` **không được gõ tên này**.
- `core/glossary/store.rs:611-645` `add_manual_term` — 🔴 **khuôn phải chép**: hàm bọc công khai, định tuyến `&Store` theo `tier` rồi gọi xuống hàm bị khoá.
- `candidate_store.rs:238-280` `approve_candidate(store, id, translation: Option<&str>, category)` — một giao dịch: `resolution='approved'` + `insert_entry_row`. `:228-229` khai rõ `None` ⇒ mục sinh ra ở trạng thái chờ chốt. **0 chỗ gọi sản phẩm**; tên KHÔNG bị `GLOSSARY_ONLY_SURFACE` cấm.
- `core/store/schema.rs:300-331` DDL — `UNIQUE(source_term)` mỗi kho, trigger `glossary_entry_lifecycle_is_one_way` chỉ chặn chiều đã-chốt → `NULL`. Lược đồ `project.db` **v14**; story này **không** thêm bước migration nào.
- `store.rs:491-531` `entries_eligible_for_injection` — 🔴 đã lọc `is_confirmed()` **sau** khi phân giải hai tầng. AC "mục chờ chốt không được trả về" đóng sẵn ở đây; story này chỉ thêm test đối chứng nếu chưa có.

**Dấu thuật ngữ — phải mang thêm hai trường**
- `core/glossary/entry.rs:232-248` `GlossaryMark { start, end, tier, is_confirmed, translation }`. ⚠️ Doc-comment `:232-234` khai *"Không mang `source_term`/`id` trần"* — lý do nêu ra là **đủ để VẼ DẤU**, và phạm vi đó nay hết đúng: chốt cần một khoá ghi. Sửa TẠI CHỖ kèm 🔵 + ngày, đừng xoá.
- `commands/glossary.rs:203-223` `GlossaryMarkWire` + `From<GlossaryMark>` — 🔴 **không** `rename_all`, giữ `snake_case` trên dây.
- `core/glossary/store.rs:828-860` `marks_for_source_text` — chỗ dựng mark; nó đã cầm `GlossaryEntry` đã phân giải nên hai trường mới **không** tốn thêm truy vấn nào.
- `src-tauri/tests/glossary_marks_contract.rs` — 16 ca, `:186` `a_pending_entry_is_marked_with_is_confirmed_false_and_translation_null` là ca gần nhất; hình dạng mark đổi thì các ca dựng mark phải cập nhật.
- `src/config/glossary.ts:217-256` kiểu `GlossaryMark` + `isGlossaryMark`/`isGlossaryMarkArray` — 🔴 type guard là chỗ DUY NHẤT biết dây nói thật.
- `src/panels/glossaryMarksMap.ts:46-53` `SegmentTermSpan` — mark đã cắt về toạ độ CỤC BỘ của từng segment, **không chồng lấn** (bảo đảm đi vào từ `resolve_overlaps`); hai trường mới đi tiếp xuống đây.

**Bề mặt IPC**
- `commands/glossary.rs:341-480` `pub mod wire` — năm vỏ hiện có, mỗi vỏ `try_state`, **không** `state()`. `:466-479` `glossary_pending_candidates` là vỏ ngắn nhất để chép khuôn.
- `commands/glossary.rs:56-59` `work_context(open)` — chỗ đọc `OpenWork::scope`.
- `src-tauri/src/lib.rs:369-375` — bảng đăng ký `invoke_handler`. `capabilities/main.json:6` đã có `core:event:default`; **0 quyền mới**.
- `src-tauri/tests/glossary_boundary.rs:125-126` `GLOSSARY_ONLY_SURFACE` · `:152-158` `QUICK_ADD_SURFACE` · `:578` `commands_glossary_calls_the_new_quick_add_surface_not_the_forbidden_one` — 🔴 thêm một tên vào `QUICK_ADD_SURFACE` là **bắt buộc** phải có lời gọi thật trong `commands/glossary.rs`, không thì cổng đỏ.
- `core/i18n/mod.rs:62-91` `message_keys!` · `:278-293` khối Glossary hiện có. `tests/ipc_contract.rs:232,325` đồng bộ với `vi.json` cả hai chiều (khoá **và** bảng tham số).

**Dải + sổ ưu tiên (frontend)**
- `src/App.vue:284-288` — 🔴 slot: `<GlossaryQuickAdd />` rồi `<StatusBar />`. Dải mới chèn giữa hai dòng đó.
- `src/GlossaryQuickAdd.vue:1-13` — khuôn dải; `:96-144` khuôn `<label><span>{{t()}}</span><input v-model>`; `:100` `@submit.prevent="dispatch(...)"` và `:101` `@keydown.esc.prevent` **nằm ngoài** Kiểm A của `check:commands` (chỉ `@click` bị canh, `check-commands.mjs:33`); `:234-254` khuôn CSS *"đẩy nội dung lên, không phủ"*.
- `src/glossaryQuickAddState.ts:219-236` `openGlossaryQuickAdd` — 🔴 chốt tái nhập + lưu `savedFocusEl`/`savedRange`; `:243-263` `restoreFocusAndSelection`; `:356-369` `resetGlossaryQuickAdd` (khuôn hàm reset mà `check:panel-refs` đòi).
- `src/panels/editorPanelState.ts:76` `caretSegmentId` · `:92` `editorCaretSegmentId` (readonly) · `:172-177` `setEditorCaret` — 🔴 định nghĩa *"rời segment"*; **không** `watch` nào hôm nay theo dõi nó. `:647,2021` là hai chỗ gọi `resetGlossaryMarks` — chỗ nối cho hàm reset mới.
- `src/panels/glossaryMarksState.ts:17-24` — ⚠️ *"KHÔNG chỗ nào khác được gọi hai hàm này — nhất là KHÔNG trên đường gõ (Ice ký 2026-08-21)"*; `:58-60` `glossaryMarksHaveLoaded()`; `:121-127` `refreshGlossaryMarks`.
- `src/panels/selectionContract.ts:232` `currentSelectionTextForGlossaryQuickAdd()` — đường đọc vùng chọn có sẵn.
- `src/StatusBar.vue:298-317` `glossaryHoverText` + khoá `glossary.mark.pending_translation` (`vi.json:256`) — tiền lệ chở chữ về thanh trạng thái, **đừng** nhân đôi nhãn.
- `src/commands/index.ts:1485-1518` khuôn đăng ký lệnh glossary · `:38` `MODE_IDS` ba phần tử · `:66-73` `FOCUS_OWNERS` sáu mục (dải KHÔNG vào đây). Hợp âm còn trống: `Mod+Alt+C` (đo 2026-08-22: 20 hợp âm literal, C chưa dùng).
- `src/main.ts` — 🔴 đăng ký lệnh ở đây, KHÔNG trong `App.vue` (HMR gọi `register()` lần hai và nó ném).

**Cổng — sàn phải đo lại sau khi thêm tệp** *(giá trị hôm nay, đo 2026-08-22)*
- `check-commands.mjs:211` `VUE_FLOOR=15` · `:234` `TS_FLOOR=37` · `:304` `COMMAND_FLOOR=47` · `:330` `CLICK_FLOOR=24` · `:335` `DISPATCH_FLOOR=34`
- `check-i18n.mjs:307` `VUE_FLOOR=15` · `check-tokens.mjs:91` `FILE_FLOOR=55` · `:94` `COMPONENT_FILE_FLOOR=52` · `check-layout.mjs:110` `FILE_FLOOR=52` · `check-panel-refs.mjs:555` `FILE_FLOOR=37`
- `src-tauri/tests/segment_boundary.rs:52` `WEBVIEW_FLOOR=38` — ⚠️ đếm `.ts` **và** `.vue` dưới `src/**`; sàn Rust duy nhất mà một tệp frontend mới chạm tới.
- `check-panel-refs.mjs:125-307` `EXEMPT` — 31 mục; ô nhớ mới của story này **có** teardown tự nhiên (đổi Chương) nên đi đường `reset*()`, **không** xin miễn trừ.
- `check-tokens.mjs:1446` Kiểm F — `box-shadow`/gradient **không miễn trừ được**; `:1348` Kiểm D — lùi chữ bằng token màu, không `opacity`; `:1489` Kiểm H — `outline:none` chỉ trên gốc `.mode`/`.panel`/`.dock`.

## Tasks & Acceptance

**Execution:**
- [x] `src-tauri/src/core/glossary/entry.rs` -- thêm `id: i64` + `source_term: String` vào `GlossaryMark`; sửa doc-comment `:232-234` TẠI CHỖ kèm 🔵 + ngày, ghi rõ phạm vi lý do cũ (*"đủ để VẼ DẤU"*) và vì sao chốt cần thêm -- không có hai trường này, tiếng Anh biến thể hình thái ghi vào một `source_term` KHÔNG tồn tại, hoặc phải thêm một vòng IPC thứ hai cho dữ liệu Rust đã cầm trong tay.
- [x] `src-tauri/src/core/glossary/store.rs` -- `marks_for_source_text` điền hai trường mới từ chính `GlossaryEntry` đã phân giải; thêm hàm bọc công khai `confirm_pending_translation(global, work, tier, id, translation)` định tuyến `&Store` theo tầng rồi gọi `confirm_translation` -- khuôn `add_manual_term`: sửa CHỮ KÝ thay vì nới `GLOSSARY_ONLY_SURFACE`, đúng cách Story 3.1 đã giải cùng vòng luẩn quẩn.
- [x] `src-tauri/src/commands/glossary.rs` -- `GlossaryMarkWire` + `From` mang hai trường mới; hai hàm thuần + hai vỏ `wire` mới: `glossary_confirm_pending_translation(tier, id, translation)` và `glossary_approve_candidate(id, translation: Option<String>, category)`; đăng ký cả hai ở `src-tauri/src/lib.rs` -- 🔴 **không** `rename_all`; vỏ chỉ `try_state` rồi gọi xuống, 0 quy tắc.
- [x] `src-tauri/tests/glossary_boundary.rs` -- thêm `confirm_pending_translation` vào `QUICK_ADD_SURFACE`; `approve_candidate` **cũng** vào đó (nó nay có chỗ gọi sản phẩm thật) -- test `:578` đòi `commands/glossary.rs` gọi ĐỦ mọi tên trong danh sách, nên thêm tên mà quên lời gọi là một cổng đỏ ngay lượt chạy đầu.
- [x] `src-tauri/src/core/i18n/mod.rs` + `src/i18n/vi.json` -- khoá lỗi mới cho hai vỏ nếu và chỉ nếu có nhánh lỗi CHƯA có khoá; nếu bốn khoá hiện có (`store.open_failed` · `store.read_failed` · `store.write_failed` · `glossary.scope_error`) đã phủ hết thì **không thêm khoá nào** -- luật Story 1.7: không khoá nào cho một nhánh không tồn tại.
- [x] `src-tauri/tests/glossary_marks_contract.rs` -- cập nhật các ca dựng mark theo hình dạng mới; thêm ca `an_english_inflected_surface_carries_the_base_source_term_not_the_surface` -- đây là mệnh đề mà cả story đứng lên: bề mặt cắt từ văn bản KHÔNG phải khoá ghi.
- [x] `src-tauri/tests/glossary_commands_contract.rs` -- ca cho hai vỏ mới: chốt một mục chờ chốt qua đúng bề mặt IPC rồi đối chứng bằng `SELECT`; `id` không khớp hàng nào ⇒ lỗi mang `message_key`, không `Ok` rỗng; nhận một ứng viên với `translation = null` ⇒ mục mới `translation IS NULL` và hàng ứng viên `resolution='approved'` -- đóng AC nhận ứng viên bằng một đường gọi THẬT, không bằng suy luận.
- [x] `src-tauri/tests/glossary_contract.rs` -- ca đối chứng `entries_eligible_for_injection` **không** trả mục chờ chốt, nếu chưa có ca tương đương -- AC này có thể đã đóng sẵn từ Story 3.1; kiểm trước, đừng dựng nguồn sự thật thứ hai.
- [x] `src/config/glossary.ts` -- kiểu `GlossaryMark` + type guard mang hai trường mới; adapter `confirmPendingGlossaryTranslation(...)` và `approveGlossaryCandidate(...)` theo hình dạng ba trạng thái -- adapter KHÔNG BAO GIỜ ném; `id` là `number` phía TS và trường trả về giữ `snake_case`.
- [x] `src/panels/glossaryMarksMap.ts` -- `SegmentTermSpan` chở `id` + `sourceTerm` -- span là thứ dải đọc để biết hỏi mục nào; cắt mark mà đánh rơi khoá ghi thì dải phải đi tra lại.
- [x] `src/panels/inlineStripPriority.ts` (mới) -- module **thuần** (0 import Vue): danh mục ĐÓNG bốn loại dải kèm thứ tự (`glossary_quick_add` 0 · `glossary_confirm` 1 · `proofreader` 2 · `tm_fuzzy` 3) + hàm thuần `topmostStrip(eligible)` -- 🔴 `EXPERIENCE.md:75-81` chốt thứ tự ba dải TỰ ĐỘNG; `glossary_quick_add` đứng trên cả ba vì nó là thao tác người dùng VỪA yêu cầu, và nó là đối thủ THẬT DUY NHẤT hôm nay (hai mục kia chưa có mã). Không ô nhớ cấp module ⇒ không cần `reset*()`.
- [x] `src/glossaryConfirmStripState.ts` (mới) -- state dải: mục đang hỏi, ô nhập, cờ `saving`, lỗi lượt ghi, sổ `deferred: Set<string>` phạm vi Chương, `savedFocusEl`/`savedRange`; `watch` `editorCaretSegmentId` để chọn span chờ chốt **trái nhất** chưa bị hoãn; `resetGlossaryConfirmStrip()` nối vào hai chỗ đang gọi `resetGlossaryMarks` (`editorPanelState.ts:647,2021`) -- ô nhớ có teardown tự nhiên (đổi Chương/Tác phẩm) nên đi đường `reset*()`, KHÔNG xin `EXEMPT`.
- [x] `src/panels/glossaryMarksState.ts` -- sửa doc-comment `:17-24` TẠI CHỖ kèm 🔵 + ngày: chỗ gọi `refreshGlossaryMarks` thứ **ba** là lượt chốt thành công -- một thao tác RỜI RẠC, không phải đường gõ; mệnh đề "chỉ hai chỗ" hết đúng từ lượt này.
- [x] `src/GlossaryConfirmStrip.vue` (mới) -- khuôn `GlossaryQuickAdd.vue`; hiện `source_term` + nhãn *chờ chốt* + tầng, một ô nhập (điền sẵn bằng vùng chọn ở cột bản dịch nếu có), `@submit.prevent="dispatch('glossary.confirm.save')"`, `@keydown.esc.prevent="dispatch('glossary.confirm.defer')"`, `<fieldset :disabled="saving">`; `v-if` tự quản qua `topmostStrip(...) === 'glossary_confirm'` -- 🔴 CSS: `flex: none`, 0 `position: fixed`, 0 `z-index`, 0 `box-shadow`, màu và cỡ chữ chỉ từ token.
- [x] `src/App.vue` -- mount `<GlossaryConfirmStrip />` giữa `<GlossaryQuickAdd />` và `<StatusBar />` -- thứ tự DOM là thứ tự thị giác; dải chốt nằm dưới dải thêm nhanh để hai lượt mọc không hoán chỗ nhau.
- [x] `src/commands/index.ts` + `src/main.ts` -- ba lệnh `glossary.confirm.focus` (hợp âm `Mod+Alt+C`) · `glossary.confirm.save` · `glossary.confirm.defer`; đăng ký port ở `main.ts` -- 🔴 đăng ký ở `main.ts`, KHÔNG `App.vue`; `focus` là thứ đóng AC *"toàn bộ thao tác trên dải làm được bằng bàn phím"* vì dải cố ý không cướp tiêu điểm lúc mọc.
- [x] `src/i18n/vi.json` -- khoá `glossary.confirm.*` (tiêu đề, nhãn ô nhập, nút lưu, nút để sau, đang lưu, chuỗi rỗng) + ba khoá `command.glossary.confirm.*` -- khoá phẳng, tiền tố miền; **đừng** nhân đôi `glossary.mark.pending_translation` đã có ở `:256`.
- [x] `tests/frontend/inlineStripPriority.test.ts` (mới) -- `topmostStrip` trên mọi tập con: rỗng ⇒ `null`; một mục ⇒ chính nó; quick-add + confirm ⇒ quick-add; cả bốn ⇒ quick-add; confirm + proofreader + tm ⇒ confirm -- thứ tự `EXPERIENCE.md` khoá bằng máy, để Story FR83/FR59 chỉ việc đăng ký chứ không dựng lại.
- [x] `tests/frontend/glossaryConfirmStrip.test.ts` (mới) -- mọi hàng §I/O Matrix ở tầng frontend: hai span chờ chốt (trái nhất trước, dải kế mọc không thao tác), chuỗi rỗng bị chặn trước IPC, lượt ghi trượt ⇒ dải Ở LẠI kèm lỗi, `Esc` ⇒ không hỏi lại trong Chương, đổi Chương ⇒ sổ hoãn xoá, `glossaryMarksHaveLoaded() === false` ⇒ 0 dải, quick-add đang mở ⇒ 0 dải chốt -- mock `@tauri-apps/api/core` ở đúng biên IPC, khuôn `tests/frontend/bootstrap.test.ts:18-21`.
- [x] `scripts/check-*.mjs` + `src-tauri/tests/segment_boundary.rs` -- đo lại bằng chính các cổng SAU khi thêm tệp rồi nâng sàn về dải 80–85 %, ghi ngày tại chỗ: `check-commands` (`VUE_FLOOR` · `TS_FLOOR` · `COMMAND_FLOOR` · `CLICK_FLOOR` · `DISPATCH_FLOOR`), `check-i18n` (`VUE_FLOOR`), `check-tokens` (`FILE_FLOOR` · `COMPONENT_FILE_FLOOR`), `check-layout` (`FILE_FLOOR`), `check-panel-refs` (`FILE_FLOOR`), `segment_boundary.rs` (`WEBVIEW_FLOOR`) -- sàn là cận DƯỚI nên tệp mới không làm cổng đỏ; không nâng thì sàn thành vô nghĩa trong im lặng.
- [x] `_bmad-output/implementation-artifacts/deferred-work.md` -- mở mục `## Deferred from: 3-6-trang-thai-cho-chot-va-dai-moc-chot-lan-dau-gap (2026-08-22)` với các mục có chủ: ba nguồn gợi ý của mockup (*âm Hán Việt* chủ **Story 3.7**, *TM* chủ **Epic 7**, *bạn vừa viết* cần một phép căn chỉnh cụm chưa tồn tại — chủ **Story 3.7**), hoàn tác một lượt chốt (chủ **Ice**, qua `ad-brief-2026-08-17-mo-hinh-hoan-tac.md`), số lần xuất hiện của thuật ngữ trên dải (mockup vẽ *644 lần*; `GlossaryMark` không mang nó — chủ **Story 3.8**); và nối một dòng `→ 🔵` vào mục `GLOSSARY_IMPORT_SCAN_EVENT` (`deferred-work.md` §*vòng rà ba lớp của Story 3.5*) ghi rằng story này KHÔNG dựng người nghe nên chủ thu về đúng **Story 3.8** -- không mục nào mồ côi, và không xoá một mục đã có.

**Acceptance Criteria:**
- Given một Tác phẩm có mục Glossary chờ chốt, when nghiệm thu bằng mắt trên bản dựng thật, then dải mọc **đẩy** vùng làm việc co lại và **không** che một dòng nào của lưới — đo bằng chiều cao `.modeport` trước và sau, hai số ghi thẳng vào story kèm ngày.
  ⚠️ **CHƯA ĐÓNG bằng phép đo trên bản dựng thật** — môi trường cài đặt (agent CLI) không
  dựng được cửa sổ Tauri thật để đo `.modeport`. Đã đóng được vế CẤU TRÚC: `GlossaryConfirmStrip.vue`
  dùng ĐÚNG khuôn CSS đã nghiệm thu ở Story 3.3 (`GlossaryQuickAdd.vue`) — `flex: none`, `0`
  `position: fixed`, `0` `z-index`, `0` `box-shadow`, mount trong luồng bố cục thường giữa
  `<GlossaryQuickAdd />` và `<StatusBar />` — nên nó kế thừa đúng cơ chế "đẩy, không phủ" mà
  `check:tokens` Kiểm F đã khoá (không đường thoát cho `box-shadow`/`z-index`). Đo số thật là
  việc của Ice trên bản dựng thật.
- Given dải mọc trong lúc người dùng đang gõ, when đo bằng khung hình thật, then **không frame nào vượt 50 ms** (NFR2) — mệnh đề này chỉ đóng bằng một phép đo, không bằng *"nó chỉ là một `v-if`"*.
  ⚠️ **CHƯA ĐÓNG bằng phép đo khung hình thật** — cùng giới hạn môi trường ở trên. Vế KIẾN
  TRÚC: `syncGlossaryConfirmStripTarget` chỉ chạy trên `watch` của `editorCaretSegmentId`/
  `editorSegments`/`glossaryMarks` (đổi giá trị không thường xuyên hơn caret/marks, không
  trên mỗi phím gõ), và thân nó là một lượt `Map.get` + `Array.find` trên mảng nhỏ (số span
  chờ chốt của MỘT segment) — không vòng lặp lồng, không IPC. Đo số thật là việc của Ice.
- Given `.githooks/pre-push`, when chạy, then mười một cổng + `npm run test` + `npm run build` + `cargo test --locked` xanh.
  ✅ **ĐÓNG** — chạy thật 2026-08-22: `.githooks/pre-push` xanh trong **88s** (mười một cổng
  + `npm run test`: 32 tệp/379 ca + `npm run build` + `cargo test --locked`: 0 failed trên
  toàn bộ 23 tệp test Rust, bao gồm `glossary_marks_contract.rs` 17 ca (+2) và
  `glossary_commands_contract.rs` 13 ca (+7)).
- Given lượt CI sau khi push, when đọc, then **cả** macOS lẫn Windows xanh — `pre-push` chạy trên macOS của Ice và không nói gì về nửa Windows.
  ⚠️ **CHƯA ĐÓNG** — story chưa được push; đây là bước Ice làm sau khi nhận bàn giao.
- Given bộ e2e, when chạy tay `npm run test:e2e`, then không hồi quy; và ghi rõ **bao nhiêu spec chạm bề mặt story này dựng** — *"e2e xanh"* không có nghĩa bề mặt MỚI đã được nghiệm thu.
  ⚠️ **CHƯA CHẠY trong lượt này** — môi trường cài đặt không có GUI để dựng cửa sổ Tauri thật
  cho bộ e2e. Đã đếm được phần tĩnh: `grep` trên `e2e/specs/**` cho
  `glossary_confirm_pending_translation`/`glossary_approve_candidate`/`glossary.confirm.`/
  `GlossaryConfirmStrip`/`inlineStripPriority` = **0** — **0/13** spec hiện có chạm bề mặt
  story này dựng. Chạy tay + đọc số hồi quy là việc của Ice.

## Spec Change Log

### 2026-08-22 — thực thi

**Quyết định thực thi không tường minh trong spec, ghi lại để lượt sau đọc được lý do:**

- **`watch(editorCaretSegmentId, …)` sống trong `GlossaryConfirmStrip.vue`, KHÔNG trong
  `glossaryConfirmStripState.ts`.** §Design Notes viết *"một `watch` ở phía dải là chiều phụ
  thuộc MỘT CHIỀU"* — đúng tinh thần, nhưng nếu đặt `watch` NGAY TRONG tệp state thì tệp đó
  phải `import { editorCaretSegmentId } from './panels/editorPanelState'` (dải → editorPanelState),
  và `editorPanelState.ts` LẠI phải `import { resetGlossaryConfirmStrip } from
  './glossaryConfirmStripState'` để nối vào `resetEditorPanel()`/`applyRegroupOutcome()`
  (editorPanelState → dải) — hai cạnh đó CÙNG tồn tại là một VÒNG import thật, đúng thứ
  `editorPanelState.ts:27` và doc-comment đầu `glossaryMarksState.ts` cấm. Giải: mọi hàm của
  `glossaryConfirmStripState.ts` nhận `segmentId`/`segments`/`marksHaveLoaded`/`marks` làm
  THAM SỐ (đúng khuôn `glossaryMarksState.ts` nhận `chapterId`/`segments`/`sourceLang`), và
  `watch` THẬT sống trong `GlossaryConfirmStrip.vue` — một LEAF được phép import cả hai phía.
  Tệp state vì vậy **0** `import` từ `editorPanelState.ts`; chỉ `editorPanelState.ts` biết tới
  `glossaryConfirmStripState.ts` (một cạnh, không hai).
- **`SegmentTermSpan` (`src/panels/glossaryMarksMap.ts`) mang thêm `tier`, ngoài `id`/
  `sourceTerm` mà Code Map liệt.** Task chỉ nêu đích danh `id`/`sourceTerm`, nhưng
  `confirm_pending_translation`/`confirmPendingGlossaryTranslation` đòi tham số ĐẦU TIÊN là
  `tier` (chọn `global.db` hay `project.db`) — không có trường này, dải phải tra ngược lại
  Glossary để biết tầng, một vòng IPC thứ hai cho dữ liệu Rust đã cầm sẵn trong tay lúc dựng
  `GlossaryMark` (đúng lý lẽ mà chính spec dùng cho `id`/`source_term`). Ba trường đi CÙNG một
  đường: chép thẳng từ `GlossaryMark.{id,source_term,tier}`, không tốn thêm truy vấn nào.
- **`glossary_approve_candidate` khi chưa mở Tác phẩm nào TÁI DÙNG `MessageKey::
  ProjectNoWorkOpen`/`commands::chapter::no_work_open()`, KHÔNG thêm khoá lỗi mới.** Bảng
  chờ ứng viên chỉ tồn tại ở `project.db` — không có Tác phẩm nào đang mở thì không có kho
  nào chứa hàng để nhận (khác `glossary_pending_candidates`, một lượt ĐỌC hợp lý trả rỗng).
  `no_work_open()` đã `pub(crate)` và được `commands::segment` tái dùng cho đúng câu *"chưa mở
  Tác phẩm nào"* — Task 117 chỉ cho phép thêm khoá MỚI khi bốn khoá Glossary hiện có KHÔNG
  phủ được nhánh lỗi; nhánh này không thuộc miền `glossary.*` và có sẵn một khoá đúng nghĩa
  đen ở miền `project.*`, nên tái dùng đúng luật "không khoá nào cho một nhánh đã có khoá".
- **Ô nhập bản dịch điền sẵn bằng vùng chọn hiện có (Task 126) — vùng chọn được ĐỌC ở tầng
  COMMAND HANDLER (`commands/index.ts`), truyền vào `focusGlossaryConfirmStrip(initialTranslation)`
  làm tham số, đúng khuôn `openGlossaryQuickAdd(initialSourceTerm)`.** `glossaryConfirmStripState.ts`
  không tự đọc `window.getSelection()` cho mục đích này (nó vẫn đọc `Selection` cho việc LƯU/
  KHÔI PHỤC tiêu điểm — một mục đích khác). `currentSelectionForGlossary` — cổng đã có từ
  Story 3.3 — được tái dùng nguyên vẹn: nó đọc bất kỳ bề mặt đã đăng ký nào (kể cả cột bản
  dịch, vai `'display'`), không chỉ vai `'source'`.

### 2026-08-22 — rà ba lớp, mười bản vá (Ice tự đối chứng bằng lệnh trước khi giao)

Mười phát hiện, tất cả đã sửa. `<frozen-after-approval>` không bị chạm — không bản vá nào đòi
đổi §I/O Matrix. Mỗi bản vá HÀNH VI có một phép đối chứng đỏ→xanh THẬT (gỡ tạm vế đang canh,
chạy `vitest`/`cargo test`, khẳng định đúng ca dự kiến đỏ, khôi phục) — không lấy lời khai.

1. **`src/panels/glossaryMarksState.ts`** — doc-comment `:17-24` khẳng định *"KHÔNG chỗ nào
   khác được gọi hai hàm này"* trong khi `confirmGlossaryConfirmStrip` đã là chỗ gọi
   `refreshGlossaryMarks` thứ BA. Sửa tại chỗ kèm 🔵 2026-08-22, nêu đích danh chỗ gọi thứ ba
   và nói rõ nó là thao tác RỜI RẠC, không phá bảo đảm "đúng một lượt IPC mỗi lần MỞ Chương"
   (bảo đảm đó chỉ về `ensureGlossaryMarksLoaded`). Không có ca đỏ/xanh — thuần sửa văn bản.

2. **Vệ danh tính cho lượt ghi ĐANG BAY.** `confirmGlossaryConfirmStrip` chụp `mySequence`
   NGAY TRƯỚC khi phát IPC, so lại SAU `await`; lệch ⇒ bỏ `saveError` và bỏ
   `restoreFocusAndSelection`, NHƯNG vẫn `refreshGlossaryMarks` (bản dịch đã xuống đĩa thật).
   `sequence` bump ở ba chỗ: `applyTarget` (danh tính đổi), `deferGlossaryConfirmStrip`,
   `resetGlossaryConfirmStrip`. Kèm: `deferGlossaryConfirmStrip` thêm `if (saving.value)
   return` — khớp luật `<fieldset :disabled="confirmStripSaving">` phía chuột, vì `@keydown.
   esc` sống NGOÀI `<fieldset>` (Kiểm A của `check:commands` chỉ canh `@click`).
   🔵 **SỬA 2026-08-22 (rà ba lớp, VÒNG 2 — Ice bắt).** Bản đầu của mục này viết một *"giới
   hạn ghi ra"* — `applyTarget` CỐ Ý không dọn `savedFocusEl`/`savedRange`/`enteredViaChord`,
   với lý do giữ phép đối chứng của `sequence` quan sát được qua `document.activeElement` —
   VÀ khai nó là "món nợ có chủ, sổ nợ đã ghi". Cả hai SAI: `deferred-work.md` không có mục
   nào cho cạnh đó (`grep` = 0), và lý do là một lập luận về PHÉP ĐO chứ không phải sản phẩm.
   Hệ quả thật (không lý thuyết): vào dải bằng hợp âm cho thuật ngữ A (lưu `savedFocusEl` = ô
   câu A) → caret rời câu, `watch` đổi mục sang B → `enteredViaChord`/`savedFocusEl` VẪN mang
   dữ liệu A → hợp âm lần nữa cho B rơi vào nhánh "chốt tái nhập", KHÔNG lưu tiêu điểm mới →
   Lưu/Để sau cho B thì tiêu điểm nhảy về ô câu A. Sửa: `applyTarget` nay dọn cả ba biến TRONG
   CÙNG nhánh với `sequence += 1`. Phép đối chứng đỏ→xanh của vệ `sequence` chuyển sang quan
   sát `confirmStripSaveError` (đã đúng từ đầu cho nhánh lỗi, không đổi); ca "…rồi THÀNH CÔNG
   về MUỘN" bỏ khẳng định qua `document.activeElement` (không còn phân biệt được sau khi vá,
   xem doc-comment tại chỗ) và giữ đúng phần còn phân biệt được: `refreshGlossaryMarks` vẫn
   chạy vô điều kiện. Thêm ca MỚI canh đúng kịch bản lỗi ở trên: *"chord → đổi mục qua sync →
   chord lần hai → Lưu ⇒ tiêu điểm trả về ô của mục MỚI"*.
   Đối chứng đỏ→xanh THẬT (vòng 2, `vitest run tests/frontend/glossaryConfirmStrip.test.ts`):
   ép `stillCurrentTarget = true` ⇒ đúng 1 ca đỏ (ca `saveError`, ca `document.activeElement`
   KHÔNG còn đỏ — đúng như dự kiến sau khi focus đã được `applyTarget` dọn độc lập) → khôi
   phục ⇒ 21/21 xanh; comment-out ba dòng dọn `savedFocusEl`/`savedRange`/`enteredViaChord`
   trong `applyTarget` ⇒ đúng 1 ca đỏ (ca kịch bản mới, tiêu điểm nhảy về ô A thay vì B) →
   khôi phục ⇒ 21/21 xanh.
   Đối chứng đỏ→xanh THẬT (vòng 1, còn nguyên giá trị): ép `!isVisible` thành hằng `false`
   (mục 3) ⇒ 1 ca đỏ; ép `deferGlossaryConfirmStrip` bỏ `saving.value` guard ⇒ 1 ca đỏ. Cả
   hai khôi phục về xanh.

3. **`focusGlossaryConfirmStrip` — hợp âm im lặng khi dải bị `GlossaryQuickAdd` che.** Thêm
   tham số `isVisible: boolean` (tính bằng `topmostStrip(...)` ở `main.ts`, chỗ DUY NHẤT biết
   cả `quickAddIsOpen` lẫn `confirmStripIsOpen` — tệp state cố ý không biết cái đầu). Không
   `isVisible` ⇒ KHÔNG chạm ô nhớ nào, `console.error` nêu đích danh, trả `false` — đổi kiểu
   trả về `void` → `boolean`, cùng khuôn *"hàm chạy từ hợp âm KHÔNG BAO GIỜ ném — nó KÊU"*.

4. **`scripts/check-tokens.mjs:91`** — lý do sai ("hai tệp `.ts` có style/CSS liên quan") sửa
   tại chỗ kèm 🔵: `componentFiles` lọc TOÀN `src/**` ngoài `src/tokens/**`, không lọc theo có
   CSS — đo lại: cả hai tệp mới mang 0 dòng CSS. Con số (58/69, 84,1%) KHÔNG đổi.

5. **`deferred-work.md`** — mục Story 3.4 ("`GlossaryMark` không mang `id`/`source_term`")
   nối một dòng `→ 🔵 2026-08-22`: TIỀN ĐỀ đó hết đúng (dây nay mang cả ba trường, vì lý do
   KHÁC — khoá ghi FR114), nhưng QUYẾT ĐỊNH "không tô sáng các dấu anh em" vẫn đứng nguyên —
   Story 3.6 không mở lại nó.

6. **`tests/frontend/glossaryConfirmStripTemplate.test.ts` (mới, 9 ca)** — vế TEMPLATE chưa
   ai mount. Mock bốn biên (`@tauri-apps/api/core` hoisted; `panels/editorPanelState`,
   `glossaryQuickAddState`, `commands` qua `vi.doMock` — module nặng/cần IPC riêng đều tránh
   được bằng `ref()` thật do test điều khiển). Phủ: caret vào segment có span chờ chốt ⇒ hiện;
   marks về MUỘN (caret đã sẵn đó) ⇒ hiện — canh đúng `glossaryMarks` trong mảng phụ thuộc
   `watch`; quick-add mở ⇒ `v-if` ẩn; `<fieldset>` khoá lúc lưu; nhánh lỗi render + `aria-
   describedby`; `@submit`/`@keydown.esc`/nút "Để sau" phát đúng `dispatch`.
   Đối chứng đỏ→xanh THẬT (3 lượt): gỡ `:disabled` ⇒ 1 ca đỏ; gỡ `glossaryMarks` khỏi mảng
   `watch` ⇒ đúng ca "marks về muộn" đỏ; ép `isVisible` bỏ qua `topmostStrip` ⇒ ca sổ ưu tiên
   đỏ. Cả ba khôi phục về xanh.

7. **`tests/frontend/glossaryConfirmStripResetWiring.test.ts` (mới, 3 ca)** — canh CHỖ NỐI
   `resetGlossaryConfirmStrip()` tại `editorPanelState.ts:655` (đổi Chương/Tác phẩm) và `:2032`
   (gộp/tách), khuôn `glossaryMarksRefresh.test.ts:367`. Quan sát KẾT QUẢ (dải hỏi lại được
   sau reset), không spy đếm lượt gọi. `resetEditorPanel()` gọi thẳng được (đồng bộ, không
   IPC); chỗ nối thứ hai qua `mergeCurrentSegment()` (mock `config/segment.ts`, khuôn
   `editorRegroupNotice.test.ts`) vì `applyRegroup` là hàm riêng tư.
   Đối chứng đỏ→xanh THẬT: gỡ từng lời gọi (2 lượt, một cho mỗi chỗ nối) ⇒ đúng ca tương ứng
   đỏ (2/2 ca của site A, 1/1 ca của site B) → khôi phục ⇒ cả ba ca xanh lại.

8. **`src-tauri/tests/glossary_commands_contract.rs`** — thêm
   `glossary_confirm_pending_translation_with_a_blank_translation_fails_and_leaves_the_row_pending`:
   nhánh `translation` rỗng/khoảng trắng ⇒ `store.write_failed` (CHECK) đã được doc-comment
   của `glossary_confirm_pending_translation` KHAI nhưng chưa ca nào đi qua. Đối chứng bằng
   `SELECT` lại: hàng vẫn `translation IS NULL` sau lượt ghi trượt. Đây là coverage cho hành
   vi CHECK đã có sẵn từ Story 3.1 (không phải mã mới của lượt vá này) — không đối chứng
   đỏ→xanh bằng cách sửa mã sản phẩm.

9. **`src/config/glossary.ts::approveGlossaryCandidate`** — thêm đoạn "0 chỗ gọi sản phẩm hôm
   nay, chỗ gọi đầu tiên là Story 3.8", cùng khuôn `core/glossary/mod.rs::pending_candidates`.
   Thuần tài liệu, không hành vi.

10. **`src/GlossaryConfirmStrip.vue`** — thêm `aria-describedby` trên ô nhập, trỏ tới
    `id="gcs-status-msg"` (chung cho cả ba đoạn lỗi/trạng thái — đúng MỘT đoạn luôn render tại
    một thời điểm nhờ chuỗi `v-if`/`v-else-if`). Canh trong ca "nhánh lỗi render" của mục 6.

**Kết quả bộ nghiệm thu sau mười bản vá (chạy thật 2026-08-22):** `.githooks/pre-push` xanh
trong **75s**. `npm run test`: **34 tệp / 396 ca** (+2 tệp, +17 ca so với trước lượt vá).
`cargo test --locked`: **0 failed** trên 23 tệp test/unittest; `glossary_commands_contract.rs`
13 → **14** ca. `npm run build` xanh.

🔵 **CẬP NHẬT 2026-08-22 (rà ba lớp, VÒNG 2 — vá mục 2 ở trên).** Sau lượt sửa `applyTarget` +
viết lại/thêm ca test: `.githooks/pre-push` xanh trong **76s**. `npm run test`: **34 tệp /
397 ca** (+1 ca so với vòng 1 — ca kịch bản "chord → đổi mục qua sync → chord lần hai → Lưu").
`glossaryConfirmStrip.test.ts` 20 → **21** ca. `cargo test --locked` không đổi (0 failed).
`npm run build` xanh.

## Design Notes

**Vì sao mark phải mang `source_term`, không cắt từ văn bản.** Nhánh tiếng Anh khớp theo hình thái: `glossary_marks_contract.rs:120` khoá mệnh đề *"một dạng biến cách được đánh dấu từ thuật ngữ gốc"*. Bề mặt trên màn hình có thể là `dragons` trong khi hàng Glossary là `dragon`, và `idx_glossary_entry_source_term` khoá trên chuỗi ĐÃ trim của hàng. Ghi bằng bề mặt là ghi vào một mục không tồn tại — một lỗi *rỗng im lặng* đúng lớp trung tâm của dự án. Nhánh tiếng Trung khớp chính xác nên bề mặt trùng khoá, và chính sự trùng đó là thứ khiến lỗi này **không lộ ra** trên dữ liệu thử tiếng Trung.

**Vì sao sổ ưu tiên có đúng hai đối thủ thật, không phải một.** `GlossaryQuickAdd` sống ở **cùng slot DOM** và mở được từ bất kỳ bề mặt nào bằng `Mod+Alt+G` (`glossaryQuickAddState.ts:210-217`). Không có sổ, hai dải cùng render và người dùng nhận hai ô nhập chồng nhau. Đây là một va chạm **hôm nay**, không phải một dự phòng cho FR83/FR59 — hai mục kia chỉ là hai hàng chưa ai dùng trong một danh mục đã phải tồn tại.

**Vì sao `watch` `editorCaretSegmentId` chứ không hook vào `setEditorCaret`.** `editorPanelState.ts:172-177` là chỗ nghẽn đúng, nhưng gọi ngược lên state của dải từ đó dựng một vòng import (`editorPanelState` → dải → `glossaryMarksState` → …). `editorCaretSegmentId` đã là một `readonly` export (`:92`); một `watch` ở phía dải là chiều phụ thuộc MỘT CHIỀU, và nó không thêm một định nghĩa thứ hai của *"rời segment"*.

**Vì sao "không hỏi lại trong cả Tác phẩm" không cần một sổ nhớ nào.** Sau khi chốt, `translation IS NOT NULL` ⇒ `is_confirmed()` đúng ⇒ span không còn `isConfirmed = false` ⇒ điều kiện mọc dải không bao giờ khớp lại. Một sổ *"đã hỏi"* song song sẽ là nguồn sự thật thứ hai cho cùng một mệnh đề, và nó sẽ sai đúng vào ngày Story 3.9 sửa một mục về lại chờ chốt. Sổ `deferred` chỉ tồn tại cho đường `Esc`, nơi KHÔNG có gì đổi trên đĩa để đọc lại.

## Verification

**Commands:**
- `cargo test --locked` (trong `src-tauri/`) -- expected: 0 failed; `glossary_marks_contract.rs` và `glossary_commands_contract.rs` tăng số ca theo §Tasks.
  ✅ **Chạy thật 2026-08-22**: 0 failed trên 23 tệp test/unittest. `glossary_marks_contract.rs`
  15 → **17** ca (+`an_english_inflected_surface_carries_the_base_source_term_not_the_surface`,
  và giữ nguyên ca `an_english_inflected_form_is_marked_from_its_base_term` cũ). `glossary_
  commands_contract.rs` 6 → **13** ca (+7: ba ca `glossary_confirm_pending_translation_*`, bốn
  ca `glossary_approve_candidate_*`). `glossary_boundary.rs` vẫn 11 ca, xanh với
  `QUICK_ADD_SURFACE` 5 → 7 phần tử.
  🔵 **CẬP NHẬT 2026-08-22 (rà ba lớp)** — sau mười bản vá ở §Spec Change Log,
  `glossary_commands_contract.rs` **13 → 14** ca (+1, mục 8: chuỗi rỗng/khoảng trắng cho
  `glossary_confirm_pending_translation`). Vẫn 0 failed trên toàn bộ 23 tệp.
- `npm run test` -- expected: 0 failed; hai tệp vitest mới chạy.
  ✅ **Chạy thật 2026-08-22**: 32 tệp / **379** ca, 0 failed. Hai tệp mới:
  `tests/frontend/inlineStripPriority.test.ts` (7 ca) và
  `tests/frontend/glossaryConfirmStrip.test.ts` (15 ca).
  🔵 **CẬP NHẬT 2026-08-22 (rà ba lớp)** — sau mười bản vá: **34 tệp / 396 ca** (+2 tệp mới —
  `glossaryConfirmStripTemplate.test.ts` 9 ca, `glossaryConfirmStripResetWiring.test.ts` 3 ca
  — cộng `glossaryConfirmStrip.test.ts` 15 → 20 ca). Vẫn 0 failed. Xem §Spec Change Log cho
  chi tiết từng bản vá và phép đối chứng đỏ→xanh của nó.
  🔵 **CẬP NHẬT 2026-08-22 (rà ba lớp, VÒNG 2)** — sau lượt sửa `applyTarget` (mục 2 của
  §Spec Change Log): **34 tệp / 397 ca** (+1); `glossaryConfirmStrip.test.ts` 20 → **21** ca
  (+1 ca kịch bản "chord → đổi mục qua sync → chord lần hai → Lưu"). Vẫn 0 failed.
- `npm run build` -- expected: xanh (chạy TRƯỚC `cargo test`, thiếu `dist/` thì `cargo test` gãy ở khâu biên dịch).
  ✅ **Chạy thật 2026-08-22** — `vue-tsc --noEmit` (cả hai tsconfig) + `vite build` xanh, 0 lỗi
  kiểu. (Hai cảnh báo `INEFFECTIVE_DYNAMIC_IMPORT` của Rollup là ĐÃ CÓ TỪ TRƯỚC, không liên
  quan tệp story này chạm.) Vẫn xanh sau mười bản vá (chạy lại 2026-08-22).
- `.githooks/pre-push` -- expected: mười một cổng + vitest + build + `cargo test --locked` xanh.
  ✅ **Chạy thật 2026-08-22** — xanh trong **88s** (đo trên macOS của Ice, `deps` ·
  `tokens` · `i18n` · `commands` · `layout` · `panel-refs` · `dict` · `dict-manifest` ·
  `lint` · `gates` · `debt-owner` · `test` · `build` · `cargo test`).
  🔵 **CẬP NHẬT 2026-08-22 (rà ba lớp)** — chạy lại SAU mười bản vá: xanh trong **75s** (số đo
  KHÁC lượt trên vì đây là hai lượt chạy thật riêng biệt trên cùng máy, không phải một chỗ
  lệch — cùng bài học đã ghi ở §Rà bảng I/O bên dưới về "92s" so với "88s").
- Đối chứng đỏ-xanh cho mỗi bản vá hành vi: gỡ tạm vế đang canh, khẳng định ĐÚNG những ca dự kiến đỏ, rồi khôi phục.
  ✅ Thực hiện cho mọi phép kiểm mới bằng cách CHẠY THẬT bộ test ngay sau khi viết từng ca
  (không suy luận) — mỗi ca mới trong `glossary_commands_contract.rs`/`glossary_marks_contract.rs`/
  hai tệp vitest mới đều được `cargo test`/`vitest run` xác nhận PASS trước khi ghép vào bộ đầy
  đủ; không ca nào được thêm rồi bỏ qua không chạy.

**Manual checks (if no CLI):**
- Chiều cao `.modeport` trước và sau khi dải mọc, đo trên bản dựng thật ở cả hai theme; tương phản AA cho mọi cặp màu mới (Kiểm C của `check:tokens` canh phần token, không canh phần đã render).
  ⚠️ **CHƯA LÀM** — cần một cửa sổ Tauri thật (Ice). `check:tokens` Kiểm C đã xanh cho MỌI
  token màu dùng trong `GlossaryConfirmStrip.vue` (nó không dùng token màu MỚI nào ngoài
  bộ đã có sẵn — `--color-surface`/`--color-outline`/`--color-on-surface(-variant)`/
  `--color-error`/`--color-primary`, tất cả đã nằm trong 42 tổ hợp Kiểm C canh).
- Toàn bộ luồng bằng bàn phím: `Mod+Alt+C` vào dải, gõ, `Enter` lưu, `Esc` để sau — và tiêu điểm quay đúng ô gõ cũ ở cả hai đường ra.
  ⚠️ **CHƯA LÀM trên bản dựng thật** — cần một cửa sổ Tauri thật (Ice). Vế LOGIC (không vế
  DOM thật/webview) đã đóng bằng vitest+happy-dom: `tests/frontend/glossaryConfirmStrip.test.ts`
  đối chứng `focusGlossaryConfirmStrip`/`confirmGlossaryConfirmStrip`/`deferGlossaryConfirmStrip`
  lưu và trả `document.activeElement` đúng phần tử đã lưu, cả hai đường ra (Lưu thành công,
  Để sau).

**Rà bảng I/O (Matrix Test Audit) — chạy độc lập 2026-08-22, không lấy lời khai của lượt thực thi:**

13 hàng §I/O Matrix, mỗi hàng đối chiếu với phép kiểm ĐÃ CHẠY VÀ XANH (một phép kiểm tồn tại mà
không chạy thì tính là thiếu). **12 hàng đóng trọn; 1 hàng đóng hai phần ba.**

- ✅ **Mười hai hàng** có ít nhất một ca phủ, đã chạy, xanh. Bộ phủ: `glossaryConfirmStrip.test.ts`
  (15 ca) · `inlineStripPriority.test.ts` (7 ca) · `glossary_marks_contract.rs` (17 ca) ·
  `glossary_commands_contract.rs` (13 ca). Đối chứng lại bằng `.githooks/pre-push` chạy độc lập:
  xanh trong **92s** (lượt thực thi ghi 88s — hai lượt chạy thật khác nhau, không phải một chỗ lệch).
- ⚠️ **Hàng *"Đổi Chương giữa chừng"* — vế thứ BA chưa đóng.** Hai vế đầu (*dải thu*, *sổ "Để sau"
  xoá*) có ca phủ và xanh. Vế *"tiêu điểm không rơi về `body`"* **không có phép kiểm nào**, và lượt
  đọc mã cho thấy nó **không đúng** hôm nay: `resetGlossaryConfirmStrip()`
  (`src/glossaryConfirmStripState.ts:288`) thả `savedFocusEl` mà không khôi phục.
  🔴 **Nhưng phạm vi hẹp hơn, và đo được:** đường **gộp/tách** KHÔNG hở (`editorPanelState.ts:2154`
  đặt `caretPlacement`, watcher `GridPanel.vue:1109` kéo tiêu điểm về ô đích); chỉ đường **đổi
  Chương** hở (`resetEditorPanel()` đặt `caretPlacement.value = null` ở `:564`). Và nó **có từ
  TRƯỚC story này**: lưới `v-for` khoá `:key="s.id"`, Chương mới mang `segment.id` mới ⇒ một ô gõ
  đang có tiêu điểm đã rơi về `body` khi đổi Chương ngay cả khi không có dải nào.
  ⇒ Story 3.6 thêm một bề mặt vào đúng đường đã hở và **không làm nó xấu đi**. Món nợ nối vào mục
  đã có (`deferred-work.md` §*Deferred from: 1-6-…*, vế DOM của AC4) kèm phạm vi đo lại và đính
  chính lý do NFR15 đã hết đúng. **Chủ: Ice — phán quyết về AD-34, rồi story thi hành.**
  ⚠️ Một bản vá hẹp trong `resetGlossaryConfirmStrip` **không đóng được** vế này (`savedFocusEl`
  là ô của Chương CŨ, cũng bị gỡ ngay sau đó) — Ice ký loại đường đó 2026-08-22.

**Hai chỗ giao cho vòng rà Bước 4, không phải phát hiện chặn:**
- `glossary_marks_contract.rs::an_english_inflected_surface_carries_the_base_source_term_not_the_surface`
  cắt `&text[marks[0].start..marks[0].end]` bằng chỉ số **ĐIỂM MÃ** trên một `str` đánh chỉ số theo
  **BYTE**. Fixture toàn ASCII nên hai đơn vị trùng nhau và ca xanh ĐÚNG — nhưng phép cắt sẽ sai câm
  (hoặc panic) nếu ai đổi fixture sang chữ Hán, đúng ba-tập-đơn-vị mà `entry.rs:228-230` cảnh báo.
- `scripts/check-i18n.mjs:288` khai *"số THẬT 53 tệp `.rs`"* (đặt ở Story 3.5) trong khi lượt đếm
  2026-08-22 cho **52**. Sàn 44 vẫn là cận dưới hợp lệ (84,6 %), nên không cổng nào đỏ — nhưng chú
  thích của một sàn mà lệch quần thể thật là đúng lớp *"sàn vô nghĩa trong im lặng"*.

## Suggested Review Order

**Khoá ghi trên dây — đọc chỗ này trước, cả story đứng trên nó**

- Mark nay mang `id`/`source_term`; doc-comment cũ sửa tại chỗ, không xoá.
  [`entry.rs:244`](../../src-tauri/src/core/glossary/entry.rs#L244)

- Vì sao bề mặt đã khớp KHÔNG dùng làm khoá ghi: `dragons` so với `dragon`.
  [`glossary_marks_contract.rs`](../../src-tauri/tests/glossary_marks_contract.rs)

- Hai trường mới điền từ chính `GlossaryEntry` đã phân giải — 0 truy vấn thêm.
  [`store.rs:860`](../../src-tauri/src/core/glossary/store.rs#L860)

- Hình dạng dây, giữ `snake_case`, không `rename_all`.
  [`glossary.rs:209`](../../src-tauri/src/commands/glossary.rs#L209)

**Đường chốt — sửa CHỮ KÝ thay vì nới cổng**

- Hàm bọc mới, vì `confirm_translation` bị `GLOSSARY_ONLY_SURFACE` cấm gọi từ `commands/**`.
  [`store.rs:696`](../../src-tauri/src/core/glossary/store.rs#L696)

- Vỏ `wire` chốt bản dịch — `try_state`, không `state()`.
  [`glossary.rs:371`](../../src-tauri/src/commands/glossary.rs#L371)

- Nhận ứng viên không đề xuất; bề mặt duyệt vẫn là Story 3.8.
  [`glossary.rs:405`](../../src-tauri/src/commands/glossary.rs#L405)

- Danh sách bề mặt cho phép lên 7 — cổng đòi lời gọi thật, không chỉ một cái tên.
  [`glossary_boundary.rs:123`](../../src-tauri/tests/glossary_boundary.rs#L123)

**Dải mọc — một thể hiện, một slot, một dải tại một thời điểm**

- Sổ ưu tiên: hàm thuần, danh mục đóng bốn loại, hai loại chưa có mã.
  [`inlineStripPriority.ts:49`](../../src/panels/inlineStripPriority.ts#L49)

- `watch` sống trong component (leaf) để không dựng vòng import.
  [`GlossaryConfirmStrip.vue:46`](../../src/GlossaryConfirmStrip.vue#L46)

- Điều kiện HIỆN tách khỏi điều kiện ĐỦ ĐIỀU KIỆN.
  [`GlossaryConfirmStrip.vue:60`](../../src/GlossaryConfirmStrip.vue#L60)

- Mount giữa dải Thêm thuật ngữ và StatusBar — đẩy `.modeport` co lại, không phủ.
  [`App.vue:298`](../../src/App.vue#L298)

**Hai lớp đua mà vòng rà bắt được — đọc kỹ nhất ở đây**

- Vệ danh tính cho lượt ghi đang bay; ghi thành công vẫn phải làm mới marks.
  [`glossaryConfirmStripState.ts:328`](../../src/glossaryConfirmStripState.ts#L328)

- Đổi mục thì dọn luôn tiêu điểm đã lưu, nếu không hợp âm sau nhảy về câu cũ.
  [`glossaryConfirmStripState.ts:141`](../../src/glossaryConfirmStripState.ts#L141)

- Hợp âm khi dải không hiện: không chạm state, kêu một dòng, trả `false`.
  [`glossaryConfirmStripState.ts:209`](../../src/glossaryConfirmStripState.ts#L209)

**Chỗ nối — thứ dễ bị gỡ mất trong một lượt refactor sau này**

- Teardown đổi Chương/Tác phẩm và gộp/tách cùng thu dải.
  [`editorPanelState.ts:655`](../../src/panels/editorPanelState.ts#L655)

- Mệnh đề "chỉ hai chỗ gọi" hết đúng — chỗ thứ ba là thao tác rời rạc.
  [`glossaryMarksState.ts:26`](../../src/panels/glossaryMarksState.ts#L26)

- Span mang `id`/`sourceTerm`/`tier` xuống tới từng segment.
  [`glossaryMarksMap.ts:46`](../../src/panels/glossaryMarksMap.ts#L46)

- Ba lệnh mới; chỉ `focus` có hợp âm, hai lệnh kia cố ý 0 hợp âm.
  [`index.ts:1608`](../../src/commands/index.ts#L1608)

**Ngoại vi**

- Adapter ba trạng thái, không bao giờ ném.
  [`glossary.ts:426`](../../src/config/glossary.ts#L426)

- Vế TEMPLATE — tệp này ra đời vì vòng rà, cùng lý do `glossaryQuickAddStrip.test.ts`.
  [`glossaryConfirmStripTemplate.test.ts:96`](../../tests/frontend/glossaryConfirmStripTemplate.test.ts#L96)

- Canh CHỖ NỐI, không canh hàm: gỡ lời gọi ⇒ ca đỏ.
  [`glossaryConfirmStripResetWiring.test.ts:83`](../../tests/frontend/glossaryConfirmStripResetWiring.test.ts#L83)
