---
title: 'Story 3.9 — Quản lý Glossary'
type: 'feature'
created: '2026-08-24'
status: 'done'
review_loop_iteration: 0
baseline_commit: 'da681257e6ccebfc3bc41dcb88c0e896320310a9'
context:
  - '{project-root}/_bmad-output/implementation-artifacts/epic-3-context.md'
  - '{project-root}/_bmad-output/implementation-artifacts/3-8-duyet-hang-loat-mot-phim.md'
  - '{project-root}/AGENTS.md'
  - '{project-root}/src-tauri/AGENTS.md'
  - '{project-root}/src/AGENTS.md'
---

<frozen-after-approval reason="human-owned intent — do not modify unless human renegotiates">

## Intent

**Problem:** Glossary có mục nhưng **0 bề mặt nào rà lại chúng**. Hàm liệt kê duy nhất là `load_tier`, và nó bị `GLOSSARY_ONLY_SURFACE` cấm gọi ngoài `core/glossary/**`; `grep "fn delete\|fn remove\|fn promote"` trên `core/glossary/**` trả **rỗng**. Hệ quả nặng nhất: một mục tầng Toàn cục bị một mục cùng `source_term` ở tầng Tác phẩm che thì **không màn hình nào nói nó tồn tại** — người dùng thấy thuật ngữ toàn cục của mình "không ăn" mà không chỗ nào trả lời vì sao.

**Approach:** Một lớp phủ modal mới liệt kê mục của **cả hai tầng**, mỗi hàng mang tầng và một cờ *đang bị che* **do Rust tính**. Tìm kiếm và ba bộ lọc chạy trong bộ nhớ trên danh sách đã nạp. Nửa Rust thêm **ba** hàm store (liệt kê · xoá · đẩy tầng) và **ba** vỏ IPC; vế "sửa" dùng lại `update_manual_term` đã có sẵn.

## Boundaries & Constraints

**Always:**
- 🔴 **0 bước di trú.** `project.db` giữ **v14**, `global.db` giữ **v4**.
- 🔴 **Xoá một mục ĐÃ CHỐT là hợp lệ** *(Ice chốt 2026-08-24)*. Bất biến một chiều nghĩa là *"không lượt `UPDATE` nào lùi trạng thái trong im lặng"* — trigger `glossary_entry_lifecycle_is_one_way` vẫn đứng — chứ **không** phải *"không chuỗi thao tác nào tái tạo được một mục chờ chốt"*. Xoá rồi thêm lại là **hai** thao tác người dùng nhìn thấy.
- 🔴 **Hai kho KHÔNG có giao dịch chung.** Đẩy tầng bắt buộc theo thứ tự: `INSERT` vào `global.db` **TRƯỚC**, `DELETE` khỏi `project.db` **SAU**. Sập giữa hai bước ⇒ thuật ngữ có ở cả hai tầng, Tác phẩm vẫn thắng, **ngữ nghĩa không đổi**, làm lại được. Thứ tự ngược lại làm mục **biến mất hẳn**.
- 🔴 **`source_term` đã có ở tầng Toàn cục ⇒ đẩy tầng TRẢ LỖI CÓ TÊN, không ghi đè.** Ghi đè là im lặng vứt đi bản dịch riêng của một trong hai tầng.
- **Cờ *đang bị che* do Rust tính**, qua `ScopeResolver::apply_override`. Chép quy tắc "Tác phẩm thắng" sang TypeScript là dựng nguồn sự thật thứ hai (AD-1, AD-18).
- 🔴 **Rỗng phải nói vì sao nó rỗng — BỐN ca, phân biệt được trên màn hình:** chưa nạp · cầu IPC vắng · Glossary trống thật · **bộ lọc không khớp gì**. Ca thứ tư nói *"không khớp bộ lọc"*, **không** nói *"Glossary trống"*.
- Mọi thao tác đi qua `CommandRegistry` (AD-34). Phím cục bộ trong lớp phủ **phải thoát sớm khi `ctrlKey || metaKey || altKey`** — bài học vá P1 của Story 3.8 (`GlossaryQueueOverlay.vue:141`), quên nó thì `⌘Backspace` xoá một mục ngoài ý định.
- Lùi hàng bằng **màu chữ** + văn bản `sr-only`. **Cấm `opacity`**, không miễn trừ.
- Vỏ `wire` dùng `try_state`; adapter `src/config/glossary.ts` **không bao giờ ném**; trường trả về giữ `snake_case`.
- **`check:panel-refs`:** mọi ô nhớ cấp module của tệp state mới phải được `resetGlossaryManage()` nhắc **TÊN**, và mỗi khai báo nằm trên **MỘT dòng** — `const a = ref(0), b = ref(0)` cho **ĐỎ** kèm câu *"cú pháp ngoài tập con"*, không bỏ qua im lặng.
- Xoá **hoặc** đẩy tầng thành công ⇒ gọi `refreshGlossaryMarks(...)`. Hàm đã có, **không dựng hàm invalidate mới**.

**Ask First:**
- Bất kỳ phụ thuộc mới nào (NFR15).
- Nếu hoá ra **buộc** phải thêm một cột hoặc một bước di trú.
- Nếu một cặp màu mới trượt sàn AA ở một trong hai theme: dừng, trình số đo — đừng đổi sàn.
- Nếu `update_manual_term` hoá ra **không đủ** cho vế "sửa".

**Never:**
- **Không sửa `source_term` của một mục.** `update_manual_term` không đụng cột đó và `idx_glossary_entry_source_term` là `UNIQUE`; người dùng đạt cùng kết quả bằng xoá + thêm. Ghi nợ, đừng mở rộng chữ ký.
- Không CSV/TSV (Story 3.10). Không sửa/xoá **ứng viên** ở bảng chờ (Story 3.8 sở hữu bề mặt đó).
- **Không cột "Dùng"/tần suất, không "sắp theo tần suất", không thống kê reviewer** — `mockups/glossary-manage.html` vẽ cả ba, nhưng `occurrence_count` **chỉ** có ở `glossary_candidate` (`schema.rs:448`), **không** ở `glossary_entry`, và `term_origin` chỉ mang ba giá trị. Ghi nợ, đừng bịa dữ liệu.
- Không suy *"đã duyệt"* từ sự tồn tại của một hàng `glossary_entry` — xoá một mục sẽ làm ứng viên đã duyệt sống lại trong im lặng.
- **Không gánh ba mục nợ e2e** `deferred-work.md:5876` · `:5906` · `:5933` *(Ice chốt 2026-08-24: chuyển chủ)*.
- Không nới `GLOSSARY_ONLY_SURFACE`; `commands/glossary.rs` **không được gõ** `load_tier`.

## I/O & Edge-Case Matrix

| Scenario | Input / State | Expected Output / Behavior | Error Handling |
|----------|--------------|---------------------------|----------------|
| Mở, có Tác phẩm | Global M mục, Work N mục | Mọi mục của **cả hai** tầng, mỗi hàng mang tầng | Lỗi ⇒ `tError`, danh sách giữ rỗng kèm câu nói rõ là LỖI |
| Mở, chưa mở Tác phẩm | `open` là `None` | Chỉ mục tầng Toàn cục; màn hình nói rõ tầng Tác phẩm **không có mặt** | N/A |
| Mục Global bị che | Cùng `source_term` ở cả hai tầng | **Hai** hàng: hàng Work (thắng) và hàng Global mang cờ *đang bị che* | N/A |
| Tìm kiếm | Gõ chuỗi con | Lọc theo `source_term` **và** `translation`; 0 lượt IPC | N/A |
| Lọc | Phân loại · xuất xứ · trạng thái chốt | Giao của ba vị từ với ô tìm kiếm | N/A |
| Bộ lọc không khớp | Danh sách đã nạp, 0 hàng qua lọc | Câu *"không khớp bộ lọc"* — **không** phải *"Glossary trống"* | N/A |
| Sửa | Đổi bản dịch/ghi chú/phân loại | `update_manual_term(tier, id, …)`; hàng cập nhật tại chỗ | Lỗi ⇒ hiện lỗi, hàng **giữ nguyên giá trị cũ** |
| Sửa lùi về chưa chốt | Mục đã chốt, xoá trống bản dịch | Trigger `RAISE(ABORT)` ⇒ `store.write_failed` | Hiện lỗi, hàng giữ nguyên |
| Xoá | Hàng đang chọn, kể cả đã chốt | Mục biến khỏi danh sách; `refreshGlossaryMarks` chạy ⇒ dấu ở lưới mất theo | Lỗi ⇒ hàng ở lại, **không** gọi refresh |
| Xoá một `id` đã biến mất | Xoá nơi khác giữa chừng | `GlossaryError::EntryMissing` ⇒ `err.glossary.entry_missing` | Hiện lỗi, nạp lại danh sách |
| Đẩy tầng, đích trống | Mục tầng Work, Global chưa có `source_term` đó | `INSERT` global ⇒ `DELETE` work; hàng đổi tầng; `refreshGlossaryMarks` chạy | Sập giữa chừng ⇒ có ở cả hai tầng, Work thắng, làm lại được |
| Đẩy tầng, đích đã có | Cùng `source_term` đã ở Global | **Lỗi có tên**; 0 lượt ghi; cả hai mục giữ nguyên | Hiện lỗi nói rõ đã có ở tầng Toàn cục |
| Đẩy một mục Global | Hàng đang chọn ở tầng Toàn cục | Lệnh **không áp dụng** — nói ra, không im lặng | N/A |
| Cầu IPC vắng | Chạy ngoài Tauri | `error: null`, danh sách rỗng kèm câu giải thích | Không ném |

</frozen-after-approval>

## Code Map

**Rust — ba hàm store, ba vỏ, một khoá lỗi**
- `src-tauri/src/core/glossary/store.rs:493-533` `entries_eligible_for_injection` -- **khuôn chép** cho hàm liệt kê: `load_tier` hai tầng ⇒ `resolver.apply_override(GLOSSARY_SCOPE_KIND, …)`. Khác ở chỗ **không** `filter(is_confirmed)` và **phải** phát cả `shadowed()` thành hàng.
- `…/store.rs:639-679` `update_manual_term(global, work, tier, id, translation, note, category)` -- dùng **nguyên**, doc-comment `:639` tự nhận là đường "sửa có hiệu lực ngay" của story này. `:613-629` `add_manual_term` -- khuôn định tuyến `&Store` theo `tier`, chép cho `delete_manual_term`.
- `…/store.rs:117-171` `insert_manual_entry` -- nửa `INSERT` của đẩy tầng (nội bộ module, không gọi từ `commands/`). `:155` giải thích vì sao `note` được `trim`.
- `src-tauri/src/core/scope/resolve.rs:45-77` `Resolved<V>` -- `value()` · `tier()` · `shadowed()`. `:24` viết thẳng *"`shadowed` không phải trang trí"*; `:284-289` trỏ tới chính màn hình này.
- `src-tauri/src/core/glossary/entry.rs:27-40` `Category` (4 giá trị) · `:82-89` `TermOrigin` (3 giá trị, cột `term_origin`) · `:144-151` `GlossaryTier` · `:215-217` `is_confirmed()` = `translation.is_some()` — **trạng thái chốt không phải một cột riêng**.
- `src-tauri/src/core/store/schema.rs:300-331` `GLOSSARY_ENTRY_DDL` -- `UNIQUE INDEX` `:327`, trigger một chiều `:328-331`. 🔴 Rào rỗng liệt **25 điểm mã `White_Space`**; `trim()` của SQLite chỉ cắt dấu cách ASCII.
- `src-tauri/src/commands/glossary.rs:77-89` `QuickAddTerm` -- khuôn struct wire. `:190` `glossary_update_term` -- khuôn hàm thuần + vỏ. Module `wire` `:501-739`, `try_state` + `lock().unwrap_or_else(PoisonError::into_inner)`.
- `src-tauri/src/lib.rs:369-388` -- **tám** vỏ glossary hôm nay; thành mười một.
- `src-tauri/src/core/i18n/mod.rs:283-293` -- ba khoá `err.glossary.*`; thêm **một** cho ca đích đã có.
- `src-tauri/tests/glossary_boundary.rs:187-197` `QUICK_ADD_SURFACE` **9 → 12**; `:142-143` `GLOSSARY_ONLY_SURFACE` (bốn tên cấm, giữ nguyên).

**Frontend — bề mặt mới**
- `src/config/glossary.ts:133` `GlossaryWriteResult<T>` -- khuôn ba trạng thái. `:175` `updateGlossaryTerm` -- **dùng lại**. `:383-419` `isGlossaryCandidate`/`…Array` -- khuôn type guard lúc chạy cho hàng mới.
- `src/GlossaryQueueOverlay.vue` -- **khuôn modal chuẩn nhất** (đã rà ba lớp): scrim + `role="dialog"` `:78-94` bẫy Tab, `:41-66` focus-return, `:141` lọc phím bổ trợ, `:197-218` mỗi ca rỗng một nhánh `v-if` riêng, `:397-399` lùi bằng màu chữ, `:447-457` `.gq-sr-only`.
- `src/glossaryQueueState.ts:60` `GlossaryQueueStatus` 5 trạng thái · `:74-77,231` chống đua `sequence` · `:183-186` **không reset lúc đóng** · `:291-300` `reset…()` tồn tại cho `check:panel-refs`.
- `src/panels/glossaryMarksState.ts:129` `refreshGlossaryMarks(chapterId, segments, sourceLang)` -- **đã có**, hai chỗ gọi sản phẩm. Khuôn gọi: `src/glossaryQuickAddState.ts:314,328,343`; khuôn đọc chương/segment lúc chạy: `src/main.ts:472-476`.
- `src/commands/index.ts:1684-1750` -- khuôn đăng ký họ `glossary.queue.*`. Hợp âm `Mod+Alt+…` **đang chiếm**: `1 2 ← → ↓ O J V L S ] [ X R P U G T C Q`; `3` để dành Review Mode. **Còn trống**: `A B D E F H I K M N W Y Z ↑`.
- `src/main.ts:486-499` -- khuôn tiêm `CommandDeps`. `:527-533` `KeymapGate.isBlocked` -- **bốn** cờ hôm nay; overlay mới phải vào danh sách này.
- `src/App.vue:263` `data-glossary-queue-open` · `:340` chỗ dựng overlay -- hai khuôn để chép.
- `src/i18n/vi.json:89` `command.glossary.queue.open` · `:287-289` ba câu rỗng của bảng chờ. `src/tokens/tokens.json` -- `roles.text` 7 vai × `roles.surface` 6 vai, `contrast.pairs` **31 cặp**; `check:tokens` Kiểm C đòi cross-product **đầy đủ**.
- Khuôn test: `tests/frontend/glossaryQueue.test.ts` (mock IPC ở biên + `mount()` thật + `installCommands()` thật, `:366`), `glossaryMarksRefresh.test.ts` (**đối chứng gỡ-lời-gọi-refresh-thì-phải-đỏ**).

**Cổng sẽ phán**: `check:commands` A/B/E · `check:tokens` C/D · `check:i18n` A/A2 · `check:panel-refs`.

## Tasks & Acceptance

**Execution:**
- [x] `src-tauri/src/core/glossary/store.rs` -- `list_all_entries(resolver, global, work)` trả mọi mục **hai tầng**, mỗi hàng `(tier, entry, is_shadowed)`: với mỗi khoá đã phân giải, phát hàng thắng, **và** phát `shadowed()` thành hàng thứ hai `(Global, …, true)` -- đó là chỗ duy nhất biết mục Global nào bị che, và bỏ nó đi là làm một mục có thật biến mất im lặng.
- [x] `src-tauri/src/core/glossary/store.rs` -- `delete_manual_term(global, work, tier, id)` (khuôn `add_manual_term`, `EntryMissing` khi 0 hàng đổi) và `promote_to_global(global, work, id)` -- `INSERT` global **trước**, `DELETE` work **sau**; đích đã có `source_term` ⇒ `GlossaryError::GlobalTermExists`, **0 lượt ghi**.
- [x] `src-tauri/src/core/i18n/mod.rs` -- khoá `err.glossary.global_term_exists`, không tham số; thêm vào `message_keys!` để nó vào `ALL`.
- [x] `src-tauri/src/commands/glossary.rs` -- `GlossaryEntryWire` (`tier` · `id` · `source_term` · `translation` · `note` · `category` · `term_origin` · `created_at` · `is_shadowed`) + ba hàm thuần và ba vỏ `wire` -- `id` chỉ duy nhất **trong một kho**, nên `tier` là thứ chọn đúng kho; bỏ nó là một lượt ghi nhắm nhầm kho, im lặng.
- [x] `src-tauri/src/lib.rs` -- đăng ký ba vỏ mới kèm chú thích nói story và FR.
- [x] `src-tauri/tests/glossary_boundary.rs` -- `QUICK_ADD_SURFACE` 9 → 12.
- [x] `src-tauri/tests/glossary_contract.rs` -- hàng bị che có mặt và mang cờ; xoá mục **đã chốt** thành công; `promote` thứ tự hai kho, ca đích-đã-có không ghi gì.
- [x] `src-tauri/tests/glossary_commands_contract.rs` -- hợp đồng ba vỏ (thành công · `id` lạ · chưa mở Tác phẩm · đẩy một mục Global).
- [x] `src/config/glossary.ts` -- `listGlossaryEntries()` · `deleteGlossaryTerm(tier, id)` · `promoteGlossaryTermToGlobal(id)` + type guard lúc chạy cho hàng mới, ba trạng thái như các hàm anh em.
- [x] `src/glossaryManageState.ts` (mới) -- danh sách, con trỏ, ô tìm, ba bộ lọc, trạng thái sửa, `sequence`; trạng thái nạp phân biệt **bốn** ca rỗng; `resetGlossaryManage()` nhắc **tên** mọi ô, mỗi khai báo một dòng.
- [x] `src/GlossaryManageOverlay.vue` (mới) -- theo khuôn `GlossaryQueueOverlay`; mỗi hàng: thuật ngữ · bản dịch · ghi chú · phân loại · xuất xứ · tầng · cờ bị che.
- [x] `src/commands/index.ts` -- `glossary.manage.open` (`Mod+Alt+M`) · `close` · `edit` · `save` · `cancel` · `delete` · `promote` · `next` · `prev` (**0** hợp âm mặc định cho tám lệnh sau) -- `createKeymap` ném khi hai lệnh giành cùng hợp âm, nên xung đột là lỗi lúc dựng.
- [x] `src/main.ts` + `src/App.vue` -- tiêm handler qua `CommandDeps`, thêm cờ mở vào `KeymapGate.isBlocked`, dựng lớp phủ + nút `data-glossary-manage-open` ở titlebar.
- [x] `src/i18n/vi.json` -- `command.glossary.manage.*` + `glossary.manage.*`, gồm **bốn** câu rỗng khác nhau.
- [x] `tests/frontend/glossaryManage.test.ts` -- phủ §I/O Matrix, và một ca **đối chứng**: gỡ lời gọi `refreshGlossaryMarks` sau xoá ⇒ ca phải ĐỎ.

**Acceptance Criteria:**
- Given một mục Global bị che, when mở màn hình, then nó **có mặt** và phân biệt được với mục đang thắng.
- Given mọi thao tác từ mở tới đóng, when thực hiện, then **không lượt chạm chuột nào** là bắt buộc.
- Given một mục vừa bị xoá, when lưới vẽ lại, then dấu của nó ở cột nguyên văn **mất theo**, và phép đối chứng gỡ lời gọi refresh cho ca test ĐỎ.
- Given `check:panel-refs` chạy, when tệp state mới có mặt, then cổng xanh **không cần** một miễn trừ nào.
- Given `check:tokens` Kiểm C/D, when màn hình mới có mặt, then xanh và **0 miễn trừ `opacity`**.

## Spec Change Log

### 2026-08-24 — vòng rà ba lớp, chín bản vá

Vòng rà chạy trên diff đầy đủ (140 KB) qua ba lớp độc lập. `<frozen-after-approval>` **không
bị chạm**: không phát hiện nào đòi đổi §I/O Matrix hay §Boundaries, nên **0 `intent_gap`,
0 `bad_spec`** — spec đã viết đúng điều cần làm, chỗ hỏng nằm ở mã.

⚠️ **Lượt thi hành đầu CHẾT giữa chừng** (máy ngủ, lỗi API) ngay trước phép đối chứng đỏ→xanh
mà §Verification đòi. Mọi số đo dưới đây là của lượt kiểm chạy lại từ đầu, không phải của báo
cáo lượt đã chết.

**Hai phát hiện nặng nhất, cả hai được NHIỀU lớp rà tìm ra độc lập:**

1. **Ca rỗng thứ tư không tới được — cả ba lớp cùng chỉ vào.** `rows` là ô nhớ riêng tư nên
   template tự bịa tham số `totalCount` bằng `manageFilteredRows.length === 0 ? 0 : 1` — một
   biểu thức lấy chính `filteredCount` làm nguồn. Bộ lọc loại hết hàng trên một Glossary CÓ
   dữ liệu ⇒ `totalCount` giả thành 0 ⇒ màn hình nói *"Glossary đang trống"*, và nhánh
   `empty_filter_no_match` là mã chết. Vi phạm thẳng §Always, và đúng lớp lỗi trung tâm của
   kho — chỉ khác là nó không im, nó nói SAI. Vá: export `manageTotalRows`.
   ⚠️ **Vì sao mọi cổng vẫn xanh:** ca cũ gọi THẲNG hàm thuần `manageEmptyReasonFor` với
   `totalCount = 1` tự chọn tay, nên nó đi vòng qua đúng chỗ gọi hỏng.
2. **`Enter` nuốt mọi nút trong lớp phủ.** `onKeydown` gắn trên `.gm-scrim` (tổ tiên của mọi
   nút) chỉ miễn `input`/`textarea`/`select`. Bấm Enter trên nút "Xoá" hay "Đẩy lên Toàn cục"
   gọi `preventDefault()` rồi `dispatch('glossary.manage.edit')` — **một phím làm VIỆC KHÁC**,
   không phải một phím chết. `Space` thoát vì rơi vào `default`, nên AC7 xanh giả ở nửa số
   cách kích hoạt một nút. Vá: thêm `HTMLButtonElement` vào danh sách miễn.

**Bảy bản vá còn lại:** `promote_to_global` trả `Ok` thay vì `EntryMissing` khi `DELETE` đổi 0
hàng *(trạng thái đích đã đạt; bản trước báo trượt một lượt đã thành công và chú thích gộp hai
kịch bản khác hẳn nhau)* · ba câu trạng thái riêng cho sửa/xoá/đẩy tầng *(trước đó xoá một mục
thì màn hình nói "Đang lưu…")* · `role="listbox"/"option"` + `aria-selected` cho hàng đang chọn
*(trước đó con trỏ chỉ tồn tại bằng MÀU)* · ghi chú "chưa mở Tác phẩm" chờ `status === 'loaded'`
*(trước đó nó nháy lên trong khoảng round-trip đầu với người ĐANG mở Tác phẩm)* · bỏ một dòng
CSS chết *(`.gm-badge-shadowed` màu `error` luôn thua độ đặc hiệu)* · sửa doc-comment
`store.rs` khai một năng lực sắp-xếp chưa từng tồn tại · ba ca test mới.

**Đối chứng đỏ→xanh THẬT, chạy trên cây:**

| Gỡ chỗ nối | Kết quả |
|---|---|
| Bỏ lời gọi `refreshGlossaryMarks` sau Xoá | ĐỎ đúng ca, khôi phục ⇒ xanh |
| Trả chỗ gọi về dạng bịa `totalCount` | ĐỎ đúng ca, khôi phục ⇒ xanh |
| Bỏ `HTMLButtonElement` khỏi danh sách miễn | ĐỎ đúng ca, khôi phục ⇒ xanh |

**Số đo sau vá:** `npm run build` sạch · vitest **449/449** (37 tệp, tăng 3 ca) · `cargo test
--locked` 25 bộ, **0 failed** · mười cổng tĩnh xanh · `contrast.pairs` vẫn **31** cặp · **0**
miễn trừ mới (không `eslint-disable`, không `aura-allow-opacity`).

**Bốn mục vào sổ nợ, không làm tròn lên:** xoá không có bước xác nhận · lọc xoá bản sửa dở
trong im lặng · hàng không bấm chuột chọn được *(và vì thế mục nợ cùng dạng của Story 3.8
CHƯA đóng)* · nhánh `changed == 0` của `promote_to_global` chưa có phép kiểm tất định.

**KEEP — thứ đã đúng và phải sống sót mọi lượt dựng lại:** thứ tự hai kho của `promote_to_global`
(kiểm đích ⇒ `INSERT` global ⇒ `DELETE` work) và nguyên tắc chọn nó *(một lượt sập để lại trạng
thái DƯ, không để lại trạng thái THIẾU)*; `list_all_entries` phát cả `shadowed()` thành hàng
riêng; nạp lại TRỌN danh sách sau Xoá/Đẩy tầng *(vì `is_shadowed` của hàng KHÁC đổi theo)* mà
KHÔNG nạp lại sau Sửa; và lọc theo TARGET trong `onKeydown` để `Backspace` trong ô gõ không xoá
một mục.

## Design Notes

**Vì sao lọc chạy ở frontend** *(Ice chốt 2026-08-24)*. `load_tier` đã nạp trọn một tầng vào một `BTreeMap` (`store.rs:237-265`) — **không có đường phân trang nào bị bỏ lỡ** bằng lựa chọn này. Đổi lại, gõ một ký tự không sinh round-trip nào, thứ mà NFR2 (không frame nào vượt 50 ms) đòi. ⚠️ **Giới hạn thật, ghi ra thay vì để người sau tự phát hiện:** chưa ai đo Glossary bao nhiêu mục thì lục — và `deferred-work.md:5601` đã ghi rằng `entries_eligible_for_injection` quét toàn bảng hai lần mỗi lượt gọi, cũng **chưa đo**.

**Vì sao đẩy tầng là hai giao dịch, không một.** `global.db` và `project.db` là hai `Store` riêng với hai `store::Writer` nối tiếp riêng; không tồn tại giao dịch bắc qua hai tệp SQLite. Thứ chọn được là **thứ tự**, và thứ tự an toàn là thứ tự mà một lượt sập để lại trạng thái **dư**, không phải trạng thái **thiếu**.

**Món nợ story này làm LỚN hơn, và không đóng.** `deferred-work.md:5548` · `:5810` giao cho 3.9 việc gói `(&Store, &ScopeResolver)` vào một chữ ký kiểu để hai vế không lệch nhau ngoài `debug_assert`. Story này thêm **ba** chỗ gọi cùng hình dạng ấy mà **không** gói — vì gói là một lượt sửa chữ ký chạm mọi chỗ gọi đã có, một đích giao được riêng. Ghi lại kèm số đo mới (3 chỗ gọi ⇒ 6), đừng làm tròn lên thành đã đóng.

## Verification

**Commands:**
- `npm run build` -- kiểu sạch; chạy **trước** `cargo test`, thiếu `dist/` thì `cargo test` gãy ở khâu biên dịch.
- `npm run test` -- vitest, gồm ca đối chứng refresh.
- `cargo test --locked` (trong `src-tauri/`) -- hợp đồng ba vỏ + ranh giới.
- `.githooks/pre-push` -- mười một cổng → vitest → build → cargo test; exit 0.

**Manual checks:**
- Đối chứng **GỠ chỗ nối** (`AGENTS.md`: một bộ test xanh không chứng minh chỗ nối mới được canh): bỏ lời gọi `refreshGlossaryMarks` sau xoá ⇒ ca tương ứng phải ĐỎ; khôi phục ⇒ xanh lại. Ghi số ca vào §Spec Change Log.
- `check:tokens` in ra số cặp tương phản: phải vẫn là **31** nếu không vai màu mới nào được dùng.
</content>
</invoke>

## Suggested Review Order

**Hai tầng và cờ "đang bị che" — chỗ mang ý đồ thiết kế**

- Điểm vào: phát cả hàng thắng lẫn hàng bị che, nên không mục nào biến mất im lặng.
  [`store.rs:723`](../../src-tauri/src/core/glossary/store.rs#L723)

- `Resolved` giữ mục Global thua ở `shadowed()` — nguồn duy nhất của cờ, không chép sang TypeScript.
  [`resolve.rs:45`](../../src-tauri/src/core/scope/resolve.rs#L45)

**Ghi bắc qua hai kho — chỗ rủi ro nhất của story**

- Kiểm đích trước, `INSERT` global trước, `DELETE` work sau: sập để lại trạng thái DƯ, không THIẾU.
  [`store.rs:828`](../../src-tauri/src/core/glossary/store.rs#L828)

- Đích đã có `source_term` ⇒ lỗi có tên, 0 lượt ghi, không ghi đè bản dịch nào.
  [`store.rs:414`](../../src-tauri/src/core/glossary/store.rs#L414)

- Xoá định tuyến `&Store` theo `tier` — `id` chỉ duy nhất trong một kho.
  [`store.rs:777`](../../src-tauri/src/core/glossary/store.rs#L777)

**Bề mặt IPC — ba vỏ mới, tám thành mười một**

- `tier` đi trên dây theo từng hàng; thiếu nó là một lượt ghi nhắm nhầm kho.
  [`glossary.rs:513`](../../src-tauri/src/commands/glossary.rs#L513)

- Ba vỏ mỏng, `try_state` chứ không `state()` — mở kho có thể đã thất bại.
  [`glossary.rs:863`](../../src-tauri/src/commands/glossary.rs#L863)

- Đăng ký ba vỏ vào `invoke_handler`.
  [`lib.rs:394`](../../src-tauri/src/lib.rs#L394)

**Bốn ca rỗng — chỗ vòng rà tìm ra lỗi nặng nhất**

- Tổng số hàng THẬT phải là export riêng; suy từ danh sách đã lọc là tự bịa.
  [`glossaryManageState.ts:134`](../../src/glossaryManageState.ts#L134)

- Vị từ bốn ca: "bộ lọc không khớp" khác hẳn "Glossary trống".
  [`glossaryManageState.ts:168`](../../src/glossaryManageState.ts#L168)

- Chỗ gọi sau vá — đọc số thật thay vì `filteredCount` trá hình.
  [`GlossaryManageOverlay.vue:292`](../../src/GlossaryManageOverlay.vue#L292)

**Bàn phím và khả năng tiếp cận**

- `HTMLButtonElement` trong danh sách miễn: thiếu nó, Enter trên nút làm VIỆC KHÁC.
  [`GlossaryManageOverlay.vue:192`](../../src/GlossaryManageOverlay.vue#L192)

- `role="listbox"` + `aria-selected`: con trỏ không còn chỉ tồn tại bằng màu.
  [`GlossaryManageOverlay.vue:311`](../../src/GlossaryManageOverlay.vue#L311)

- Lớp phủ mới phải chặn keymap toàn cục trong lúc mở.
  [`main.ts:567`](../../src/main.ts#L567)

**Đồng bộ dấu ở lưới và trạng thái**

- Xoá/đẩy tầng thành công mới gọi refresh; SỬA thì không, và lý do ghi tại chỗ.
  [`glossaryManageState.ts:409`](../../src/glossaryManageState.ts#L409)

- Thao tác nào đang bay — trước đó xoá một mục thì màn hình nói "Đang lưu…".
  [`glossaryManageState.ts:95`](../../src/glossaryManageState.ts#L95)

- Hàm reset nhắc TÊN mọi ô nhớ, mỗi khai báo một dòng (`check:panel-refs`).
  [`glossaryManageState.ts:445`](../../src/glossaryManageState.ts#L445)

**Phép kiểm**

- Ba ca đối chứng của vòng rà — mỗi ca đã được nghiệm bằng cách GỠ chỗ nối và thấy nó đỏ.
  [`glossaryManage.test.ts:436`](../../tests/frontend/glossaryManage.test.ts#L436)
