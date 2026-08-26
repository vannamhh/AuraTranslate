---
title: 'Cụm F — mục rải rác bốn tầng, và ba phát hiện bị chính phép đo bác'
type: 'bugfix'
created: '2026-08-26'
status: 'done'
review_loop_iteration: 1
baseline_commit: '85a7ab282ab6175f01a3a61166741bf9e88e31c2'
context:
  - '{project-root}/_bmad-output/implementation-artifacts/epic-3-context.md'
  - '{project-root}/AGENTS.md'
  - '{project-root}/src/AGENTS.md'
  - '{project-root}/src-tauri/AGENTS.md'
  - '{project-root}/tests/AGENTS.md'
---

<frozen-after-approval reason="human-owned intent — do not modify unless human renegotiates">

## Intent

**Problem:** Cụm F là cụm cuối của vòng rà Epic 3 và là cụm **duy nhất** mà một lượt đọc lại đã bác ba trong số các phát hiện của chính nó. Sổ nợ khai *"mười bảy mục rải rác"*, nhưng danh sách vị trí ngay dưới nó chỉ đếm được **14 chỗ**, và ba chỗ trong số đó không đứng vững khi đo: ① *"hai `expect` dựa trên bất biến KHÔNG được cưỡng chế"* — chỉ **một** trong hai đúng, cái ở `scan.rs:319` được **borrow-checker** cưỡng chế nên không có đường nổ; ② *"hai lượt `SELECT` ngược thiết kế hai-khoá-ngắn"* trỏ vào `project.rs:177-209`, nơi **không có** `work_state.lock()` lẫn `SELECT source_term` nào — chỗ thật (`filter_and_enqueue_current_import_scan`) làm **đúng** điều doc-comment của chính nó khai; ③ *"`insert_candidate` không loại thuật ngữ đã có trong `glossary_entry`, và không kiểm lại tầng Global"* — hàm đó có **0 chỗ gọi sản phẩm** và đã bị `GLOSSARY_ONLY_SURFACE` khoá, còn đường sản phẩm thật ĐÃ lọc **cả hai** tầng qua `filter_import_scan_candidates_by_scope`. Còn lại **tám** chỗ đứng vững, trải bốn tầng: một `expect` thật giết được tiến trình giữa lượt nhập, một API `bool` nguy hiểm còn `pub` mà 0 ai gọi, hai chỗ nuốt ca "chưa quản lý `DictLayers`", một bảng họ phồn thể chỉ có 1/110 cặp, bốn khai báo px thô, và một `role="listbox"` mà chú thích cạnh nó khai đã đóng một lỗ hổng nó chưa đóng.

**Approach:** Vá đúng **tám** chỗ đứng vững, mỗi chỗ tái dùng khuôn ĐÃ CÓ ngay cạnh (`guarded_dict_layers` của `project.rs`, `MessageKey::Unknown` của AD-21, khuôn cổng cấu trúc của `the_blocking_wires_run_off_the_main_thread`). **Bác ba phát hiện kia bằng chữ, kèm phép đo**, và ghi phần dư địa thật của mỗi cái thành một mục nợ **có chủ** — không đóng bằng suy luận, cũng không im lặng bỏ qua. Con số *"mười bảy"* được sửa tại chỗ bằng 🔵 chứ không trích tiếp.

## Boundaries & Constraints

**Always:**
- 🔴 **Mỗi bản vá kèm phép đối chứng GỠ-CHỖ-NỐI:** gỡ bản vá ra thì phép kiểm mới phải **ĐỎ**, và §Completion Notes ghi **tên ca** cùng **số ca đỏ thật**. Bộ test cũ xanh không đủ (`AGENTS.md` §Known pitfalls — Epic 3 dính năm lần trong bảy ngày).
- 🔴 **Ba phát hiện bị bác phải được ghi thành chữ**, mỗi cái nêu: tiền đề sai là gì, phép đo nào bác nó, và **dư địa còn thật** là gì kèm chủ sở hữu. Bác im lặng bị cấm.
- 🔴 **Bảng họ nhận đúng năm cặp `陳 張 劉 楊 黃`** (Ice chốt 2026-08-26). Cả năm được phép đo trên `dict-core.db` xác nhận. Phần còn lại (~105 họ có vế phồn thể khác) là một mục nợ có chủ **kèm nguyên bảng đo**, không phải một lượt gõ tay.
- 🔴 **Không nhập trọn 134 cặp đo được** — chính phép đo đó bác nó: bảng chứa `鬍→胡` (râu), `週→周` (tuần), `鬱→郁` (u uất), `餘→余`, và `於→于` trong khi `於` ĐÃ là một họ riêng trong `COMMON_SURNAMES`. Một alias sai nới ngưỡng cho chuỗi không phải tên người.
- 🔴 **`guarded_dict_layers` dùng CHUNG, không chép bản thứ hai.** Nó đã có ở `commands/project.rs` kèm hai ca test; lượt này mở phạm vi nó, và chẩn đoán phải nêu đúng bề mặt đang gọi (không in `[import_scan]` cho một lượt `marks_for_chapter`).
- 🔴 **Ba chỗ `unwrap_or(&empty_layers)` ở `commands/dict.rs` KHÔNG đụng tới** — chúng có doc-comment riêng biện minh (`dict.rs:61-64`, AD-25) và là hành vi đúng cho đường tra cứu.
- Chuỗi literal trong `src-tauri/src/**` viết KHÔNG DẤU. Tên hàm test là một CÂU khẳng định. Ca Rust mới theo bốn luật đầu tệp `*_contract.rs`.
- Chuỗi hiển thị: không khoá `vi.json` mới, không token mới. `MessageKey::Unknown` đã tồn tại.

**Ask First:**
- 🔴 **Một ca mới ĐỎ ngay lượt chạy đầu vì mã sản phẩm ⇒ DỪNG và trình lỗi.** Bảy trong tám mục kỳ vọng mã sản phẩm đang đúng về HÀNH VI; một ca đỏ nghĩa là vòng rà vừa tìm ra một khuyết tật thật, và đó là quyết định phạm vi của Ice.
- Nếu việc gỡ `scan_candidates` (vỏ `bool`) làm đỏ một ca **ngoài** `glossary_scan_contract.rs` và `commands/project.rs` `#[cfg(test)]`: trình danh sách chỗ gọi, đừng tự nới phạm vi.

**Never:**
- Không đụng **ba món e2e/NFR2** của cụm F — chúng đã có chủ riêng (*story đầu tiên mở rộng bộ e2e sang Glossary*) và sổ nợ tự khai chúng *"rộng hơn một story vá"*.
- Không đụng mục nợ **C4** (bốn hàm đọc hai tầng qua hai kết nối), không đụng mục **`MutexGuard` của mười lăm vỏ** — đóng chúng là lật một bất biến đã ký, cần một `AD` mới, không phải một dòng mã.
- Không sửa 16 tệp `.vue` còn lại có px thô (Ice chốt 2026-08-26: chỉ tệp cụm F nêu). Không thêm Kiểm spacing vào `check-tokens.mjs` lượt này.
- 🔴 **Không đổi một pixel nào của khuôn "nét dẫn"** (`padding-left: 11px` + `border-left: 2px`, **6 chỗ / 5 tệp**, tổng lề 13px). Ice chốt 2026-08-26 sau khi phép đo bác tiền đề "hai bản sao": `.gs-alert` đổi sang `calc(var(--space-unit) * 2.75)` — **11px giữ nguyên**, và `ShortcutsOverlay.vue` · `AttributionOverlay.vue` · `panels/LookupPanel.vue` · `panels/LookupRecord.vue` **không được chạm một dòng**.
- Không thêm phụ thuộc npm hay crate nào (NFR15). Không `tempfile`. Không `vi.useFakeTimers()`. Không thêm `?.` vào mã sản phẩm cho một ca hết đỏ, không `eslint-disable`, không nới một cổng.
- Không đổi `[profile.release]`. Không thêm `MessageKey` mới — danh mục đóng.

## I/O & Edge-Case Matrix

| Scenario | Input / State | Expected Output / Behavior | Error Handling |
|----------|--------------|---------------------------|----------------|
| **①a** `parse` trượt, danh sách lỗi KHÔNG rỗng | `Err(vec![issue_a, issue_b])` | `IpcError` của **`issue_a`** (lỗi đầu tiên) — hành vi hôm nay, không đổi | `eprintln!` nêu số lỗi và lỗi đầu |
| **①b** `parse` trượt, danh sách lỗi **RỖNG** | `Err(vec![])` | `IpcError` mang `MessageKey::Unknown` — **không panic, không `abort`** | `eprintln!` nêu đích danh rằng danh sách rỗng là một bất biến đã vỡ |
| **②** vỏ `bool` `scan_candidates` | grep bề mặt công khai `core/glossary/**` | **0** hàm quét nào nhận callback `bool`; `DictionaryProbe` ba trạng thái là đường duy nhất | cổng cấu trúc ĐỎ có tên nếu ai đó dựng lại |
| **③a** `DictLayers` **chưa từng** `manage` | `try_state::<DictLayers>()` ⇒ `None` | chẩn đoán `eprintln!` nêu đích danh bề mặt gọi; lượt đó KHÔNG im lặng | không panic (`try_state`, không `state`) |
| **③b** `DictLayers` đã quản lý nhưng **rỗng** | `Some(DictLayers::empty())` | đi tiếp **lặng lẽ** — AD-25, đây là trạng thái bình thường | không chẩn đoán, không lỗi |
| **④a** họ phồn thể có alias | `Zh`, chuỗi 2 ký tự bắt đầu bằng `陳` | ngưỡng hạ còn `threshold - 1` — giống hệt vế giản thể `陈` | N/A |
| **④b** đối chứng ngược | `Zh`, chuỗi 2 ký tự bắt đầu bằng một chữ phồn thể **không** phải họ (`鬍`) | ngưỡng **giữ nguyên** `threshold` — bảng chỉ NỚI cho họ, không nới bừa | N/A |
| **④c** quần thể bảng alias | toàn bộ `TRADITIONAL_SURNAME_ALIASES` | mọi vế **giản** có trong `COMMON_SURNAMES`; **0** vế **phồn** nào tự nó nằm trong `COMMON_SURNAMES` | thiếu một vế ⇒ ca đỏ nêu đích danh cặp |
| **⑤a** con trỏ ở hàng thứ `n` | `manageCursor === n`, danh sách đã lọc | `<ul>` mang `aria-activedescendant` **bằng đúng** `id` của `<li>` thứ `n` | N/A |
| **⑤b** con trỏ di chuyển | `manageCursor` đổi `n → n+1` | `aria-activedescendant` đổi theo, trỏ `id` hàng mới | N/A |
| **⑤c** danh sách rỗng | `manageFilteredRows` rỗng | `<ul>` **không** mang `aria-activedescendant` trỏ một `id` không tồn tại | thuộc tính vắng mặt, không phải chuỗi rỗng |

</frozen-after-approval>

## Code Map

### ① `expect` giết được tiến trình — `commands/glossary.rs`

- `src-tauri/src/commands/glossary.rs:764-771` **`glossary_open_import_preview`**, khối `map_err`. **HAI** điểm panic, không phải một: `issues[0]` trong `eprintln!` (`:768`) và `.expect("Err chi dung khi issues khong rong")` (`:770`). Sổ nợ chỉ nêu cái thứ hai.
- Bất biến *"`Err` chỉ đi kèm `issues` không rỗng"* giữ bằng **cách dựng**, không bằng kiểu: `core/glossary/exchange.rs:776 parse()` có **sáu** điểm `return Err(...)` (`:795 :807 :834 :839 :856 :1012`), mỗi điểm dùng vec-literal khác rỗng hoặc có `if !issues.is_empty()` canh trước. Một lượt sửa sau trả `Err(vec![])` là đủ.
- `src-tauri/src/core/i18n/mod.rs:100-107` **`MessageKey::Unknown => "err.unknown" []`** — khoá dự phòng cuối cùng của AD-21, **không tham số** có chủ ý. Đây là đích rơi đúng.
- `IpcError::new(code, message_key, params, retryable)` là **chỗ duy nhất** được dựng lỗi IPC (`src-tauri/AGENTS.md`). Không struct literal.
- Đây là `expect` sản phẩm **duy nhất** trong `commands/**`; toàn `src-tauri/src/**` có đúng **4** `.expect(` và **0** `.unwrap(` ngoài `#[cfg(test)]` (`lib.rs:454` · `core/segment/split.rs:300` · `core/glossary/scan.rs:319` · `commands/glossary.rs:770`).

### ② `expect` bị BÁC — `core/glossary/scan.rs:319`

- `src-tauri/src/core/glossary/scan.rs:303-319` `zh_nested_padding`. `terms: Vec<&String>` thu **thẳng** từ `freq.keys()`; `freq: &HashMap` là mượn bất biến trọn hàm và không lượt ghi nào xen vào (chỉ `dropped` local đổi) ⇒ `freq.get(term)` **không có đường** trả `None`. Tiền đề *"bất biến KHÔNG được cưỡng chế"* SAI — borrow-checker cưỡng chế nó.
- Lý do vá còn lại **yếu hơn hẳn** và phải nói ra như vậy: một lượt refactor sau đưa `freq` thành `&mut` sẽ mở đường nổ, và `panic = "abort"` (`Cargo.toml`, **chỉ** `[profile.release]` — `cargo test` chạy `unwind`) biến nó thành cái chết của tiến trình. Bản vá là một lượt viết lại **không đổi hành vi**: duyệt `freq.iter()` thay vì `keys()` rồi `get`.

### ③ Vỏ `bool` còn `pub` mà 0 ai gọi — `core/glossary/scan.rs` + `mod.rs`

- `src-tauri/src/core/glossary/scan.rs:118-147` **`pub fn scan_candidates`** — thân nó CHÍNH LÀ adapter cần chuyển xuống bàn test: dựng `probe` gói `is_known` thành `Known`/`Missing`, `never_cancelled` luôn `false`, rồi `match` `ScanOutcome`.
- `src-tauri/src/core/glossary/mod.rs:285-287` re-export (sổ nợ ghi `:266-268` — **đã rot 19 dòng**). `mod.rs:120` doc-comment cũng nêu tên `scan::scan_candidates`.
- `scan.rs:62-67` `pub enum DictionaryProbe { Known, Missing, Inconclusive }`; `scan.rs:149-158` `scan_candidates_controlled` là đường sản phẩm.
- **Chỗ gọi thật, danh sách ĐÓNG:** `src-tauri/tests/glossary_scan_contract.rs` (20 lượt) và `src-tauri/src/commands/project.rs:1059` (**trong** `#[cfg(test)] mod tests`, khối bắt đầu `:753`). **0** chỗ gọi sản phẩm. Đường sản phẩm dùng `scan_candidates_controlled` ở `project.rs:600` với `dictionary_probe_from_grouped` (`:322-336`).
- Khuôn cổng cấu trúc để canh việc nó không sống lại: `src-tauri/tests/config_invariants.rs:887-975` `the_blocking_wires_run_off_the_main_thread` (đọc tệp nguồn → tìm chữ ký → đếm tuyệt đối), và `:769` `RS_FLOOR_FOR_DIALOG_CHECK = 44` là khuôn sàn quần thể.

### ④ Nuốt ca "chưa quản lý `DictLayers`" — `commands/glossary.rs` ×2

- `src-tauri/src/commands/glossary.rs:1038` (`glossary_marks_for_chapter`) và **`:1085`** (`glossary_pending_candidates`) — sổ nợ ghi `:1030,1069`, đó là dòng **doc-comment**, không phải dòng gọi. Cú pháp thật là `layers.as_deref().unwrap_or(&empty_layers)`; chuỗi `unwrap_or(&DictLayers::empty())` **không tồn tại** trong mã sản phẩm nào.
- **Khuôn ĐÚNG, đã có:** `src-tauri/src/commands/project.rs:472-483` doc-comment (*"🔴 VÁ 2026-08-22 — bản trước NUỐT ca `DictLayers` chưa được quản lý… gộp HAI trạng thái khác hẳn nhau vào một nhánh im lặng"*) + `:484-493` `fn guarded_dict_layers`, dùng ở `:577-582`. Hai ca canh: `guarded_dict_layers_returns_none_and_does_not_silently_fall_back_to_empty_when_not_managed` (`project.rs:1330`) và `..._passes_the_managed_layers_through_unchanged` (`:1341`).
- **Quần thể thật 5, không 2:** ba chỗ còn lại ở `commands/dict.rs:78,91,200` có doc-comment riêng biện minh (`:61-64`) — **để nguyên**.
- `DictLayers` ở `core/dict/layer.rs:554`, `empty()` ở `:646-657`. Cả hai chỗ đã dùng `try_state` (đúng), lỗi nằm ở bước sau nó.
- ⚠️ Thời gian là bằng chứng: `project.rs` vá 2026-08-22, `glossary.rs` ghi *"THÊM 2026-08-24"* — anti-pattern được tái lập **hai ngày SAU** khi nó bị gọi tên.

### ⑤ Bảng họ phồn thể — `core/glossary/surnames.rs`

- `src-tauri/src/core/glossary/surnames.rs:58` `pub(super) const TRADITIONAL_SURNAME_ALIASES: &[(char, char)] = &[('蕭', '萧')];`, doc-comment `:51-57` (*"alias là một bước chuẩn hoá hình dạng chữ trước phép tra bảng, không phải một họ mới"*).
- `surnames.rs:31-48` `COMMON_SURNAMES` — **272** ký tự giản thể, nguyên văn *Bách gia tính*, thứ tự cố ý không sắp lại. `陈 张 杨 黄 刘` đều CÓ trong bảng.
- `scan.rs:215-236` `effective_threshold` — chỗ tra: `surnames.contains(&first) || TRADITIONAL_SURNAME_ALIASES.iter().find_map(...).is_some_and(|simp| surnames.contains(&simp))`.
- ⚠️ **`於` là bẫy:** nó nằm SẴN trong `COMMON_SURNAMES` (hàng 13) **và** xuất hiện trong bảng đo dưới dạng `於→于`. Ca quần thể ④c tồn tại để bắt đúng lớp này.
- **Phép đo đã chạy (2026-08-26, `src-tauri/resources/dict/dict-core.db`, `dict_entry` DISTINCT `headword`/`headword_simp`, cả hai dài 1 ký tự, khác nhau):** 7.362 cặp trần→giản một ký tự; **134** cặp có vế giản nằm trong `COMMON_SURNAMES`, phủ **110/272** họ; **0** cặp mơ hồ (một phồn → nhiều giản). Bảng đo đầy đủ đi vào mục nợ.

### ⑥ px thô — `src/GlossarySettingsOverlay.vue`

- Bốn khai báo, tất cả trong `<style scoped>` (`:163-301`): `:235` `gap: 4px` · `:251` `padding: 4px 6px` · `:266` `padding-left: 11px` · `:280` `padding: 4px 12px`.
- `--space-unit` = **`4px`**, khai ở `src/tokens/tokens.json:479`, áp ra CSS var ở `src/tokens/index.ts:107`. Token khoảng cách sẵn có: `--space-panel-inline` 16px · `--space-panel-block` 12px (tệp này ĐÃ dùng chúng ở `:265`, `:275`).
- Khuôn đúng ngay trong nhóm: `src/GlossaryQuickAdd.vue:259-267` `gap: calc(var(--space-unit) * 2)`. `GlossaryManageOverlay.vue`, `GlossaryQueueOverlay.vue`, `GlossaryImportOverlay.vue`, `GlossaryConfirmStrip.vue` đều **0** px spacing thô.
- 🔴 **`:266` `11px` KHÔNG phải một giá trị lạc — nó là khuôn "nét dẫn" của toàn kho.** Đo 2026-08-26: `padding-left: 11px` + `border-left: 2px solid <token>` xuất hiện **6 chỗ ở 5 tệp**, tổng lề quang học **13px**, chỉ khác token màu viền — `.gs-alert` `:266` (`--color-error`) · `ShortcutsOverlay.vue:446` `.sc-alert` (`--color-error`) · `ShortcutsOverlay.vue:432` `.sc-note` (`--color-tm-rule`) · `AttributionOverlay.vue:333` `.attr-note` (`--color-tm-rule`) · `panels/LookupPanel.vue:1059` `.lookup-disagree` (`--color-tm-rule`) · `panels/LookupRecord.vue:312` `.lookup-citation` (`--color-primary`). Chú thích `:260-263` khai *"cùng khuôn `.sc-alert`"* và lời khai đó ĐÚNG.
  ⇒ **Ice chốt: `calc(var(--space-unit) * 2.75)`** — bỏ px thô mà **giữ nguyên 11px**, nên khuôn 6 chỗ vẫn đồng nhất và không tệp nào ngoài cụm F bị chạm. Phương án 12px bị chính phép đo này bác: nó sẽ để `ShortcutsOverlay.vue` mang hai lề lệch nhau 1px cho hai câu nằm cạnh nhau (`:427` và `:444`).
- **Ngoài phạm vi:** `max-width: 560px`, `border: 1px/2px` — không token kích thước/viền nào tồn tại.
- `scripts/check-tokens.mjs` có **7 Kiểm** (A tokens.json · B màu · B2 **chỉ 12 thuộc tính chữ** · C tương phản · D opacity · E giãn dòng · F bóng/gradient · G phân tách panel). **Không Kiểm nào** đọc `padding`/`margin`/`gap` ⇒ xác nhận cổng không thấy.
- **Quần thể toàn cây đã đo:** 17 tệp `.vue`, **104** khai báo px thô ngoài viền 1px, **~53** lệch lưới 4px. Nặng nhất `panels/LookupPanel.vue` (19) · `modes/LibraryMode.vue` (18) · `ShortcutsOverlay.vue` (14). Tệp này chiếm **4** (spacing). 16 tệp kia ⇒ mục nợ.

### ⑦ `aria-activedescendant` — `src/GlossaryManageOverlay.vue`

- `:348` `<ul class="gm-list" role="listbox" :aria-label="…">` — **không** `aria-activedescendant`. `:349-357` `<li … role="option" tabindex="-1" :aria-selected="i === manageCursor">` — **không** `id`. Sổ nợ ghi `:326-347`, dòng thật là `:340-381`.
- `:340-347` chú thích khai đã đóng lỗ hổng *"người dùng trình đọc màn hình điều hướng bằng mũi tên (AC7) không có cách nào biết con trỏ đang ở đâu"*. Tiêu điểm DOM **không** đi theo `manageCursor`, nên `aria-selected` một mình không cấp mệnh đề đó ⇒ chú thích khai quá, phải 🔵 sửa tại chỗ.
- 🔴 **SỬA 2026-08-26 (vòng rà, `review_loop_iteration` 0→1) — bản đầu của mục này CHÉP MỘT MỆNH ĐỀ SAI.** Nó viết *"`tabindex="-1"` trên **cả** `<ul>` lẫn `<li>`"*. Đo lại: `<ul class="gm-list">` **không có `tabindex` nào cả** — chỉ `<li>` có (`:378`). Tiêu điểm đi tới `<section ref="panel" class="gm-panel" tabindex="-1" role="dialog" aria-modal="true">` (`:275`, qua `panel.value?.focus()` ở `:83`), còn phím mũi tên bắt ở `.gm-scrim` (`:273`) qua bubbling.
  ⇒ **Hệ quả:** `aria-activedescendant` đặt trên `<ul>` mà `<ul>` không bao giờ giữ tiêu điểm là một thuộc tính trình đọc màn hình **KHÔNG đọc** (WAI-ARIA: nó chỉ được tôn trọng trên phần tử ĐANG giữ tiêu điểm). Ba hàng ma trận ⑤ vẫn ĐÚNG chữ, nhưng chúng chỉ chốt **nửa** cơ chế; nửa còn thiếu là tiêu điểm.
  ⇒ **Ice chốt 2026-08-26:** cho `<ul role="listbox">` một `tabindex="-1"` và **focus chính nó** (thay `.gm-panel`) khi danh sách có hàng — giữ thuộc tính đúng chỗ ma trận đã ký. Phương án *"chuyển thuộc tính sang `.gm-panel`"* bị loại có lý do: `.gm-panel` mang `role="dialog"`, và ARIA 1.2 **không** liệt `dialog` trong các vai nhận `aria-activedescendant`.
  ⚠️ Chạm `<ul>`: xét lại `trapTab` (`:110-131`, nó liệt phần tử focus được) và `focusReturnTargetOnOpen`/`:83` — và một ca chỉ kiểm GIÁ TRỊ thuộc tính **không** chứng minh được vế này; phải có một ca canh TIÊU ĐIỂM.
- Định danh sẵn có để dựng `id`: `:351` `:key="`${row.tier}:${row.id}`"`.
- State: `manageCursor` — `src/glossaryManageState.ts:144` (`readonly(cursor)`), ref gốc `:83`.
- **Quần thể đã đo:** `role="listbox"` và `role="option"` xuất hiện **chỉ ở tệp này**; `aria-activedescendant` xuất hiện **0 lần trong toàn kho**. Bốn `aria-selected` khác thuộc `role="tab"` (`panels/LookupPanel.vue:434,447` · `panels/GridPanel.vue:1529,1542`) — mẫu khác, tab nhận tiêu điểm DOM thật nên không cần. **Không có tiền lệ đã-sửa trong cây** để chép.
- Bàn test: `tests/frontend/glossaryManage.test.ts` (37 ca) — grep `aria`/`activedescendant`/`listbox` cho **0** kết quả.

### Baseline đã đo trên `85a7ab2` (2026-08-26)

~~`cargo test --locked` **674 ca / 23 tệp** · `npm run test` **569 ca / 43 tệp**~~ · 11 cổng `pre-push` xanh. Tệp liên quan: `glossary_contract` 72 · `glossary_exchange_contract` 61 · `glossary_commands_contract` 29 · `glossary_scan_contract` 27 · `config_invariants` 22 · `glossary_boundary` 11; ~~`glossaryManage.test.ts` 37~~ · `glossarySettings.test.ts` 21.

🔵 **SỬA 2026-08-26 (sau lượt vá, đo bằng lượt CHẠY).** Mọi con số gạch ngang ở trên đến từ một
phép **đếm bằng chữ** (`grep` số `#[test]` / `it(` trong tệp nguồn), không từ một lượt chạy — và
`grep` đếm cả những khai báo mà bộ chạy không sinh ra một ca, nên nó **cao hơn** số thật. Đừng
đọc chúng như số nghiệm thu, và đừng trích tiếp:

- **vitest** — baseline THẬT `561 ca / 43 tệp`, `glossaryManage.test.ts` **33** ca (đo hai chiều:
  agent thi hành `git stash` về `85a7ab2` và chạy; đối chứng độc lập của tôi sau lượt vá cho
  `564 / 43` và `36` ca, đúng `561 + 3` và `33 + 3` với ba ca ⑤a/⑤b/⑤c mới).
- **cargo** — số `674 / 23` không so được với bất cứ số nào sau lượt vá, vì nó chỉ đếm
  `src-tauri/tests/**` và bỏ hẳn 38 ca đơn vị trong `src/`. Số ĐO ĐƯỢC sau lượt vá:
  **716 ca tổng** = 678 tích hợp (`tests/**`) + 38 đơn vị (`src/`), **0 đỏ**, 28 dòng
  `test result:`. Baseline cargo tương ứng KHÔNG được đo lại — nên nó không có số ở đây, thay vì
  có một số suy ra.
- Phần nghiệm thu thật của lượt này không nằm ở các số tổng này mà ở **năm phép GỠ-CHỖ-NỐI**
  (§Completion Notes), nơi mỗi mệnh đề có một con số đỏ đo được của riêng nó.

## Tasks & Acceptance

**Execution:**
- [x] `src-tauri/src/commands/glossary.rs` -- tách khối `map_err` của `glossary_open_import_preview` thành một hàm **thuần, đặt tên** nhận `Vec<ParseIssue>` trả `IpcError`: có lỗi ⇒ lỗi đầu tiên (hành vi cũ), rỗng ⇒ `MessageKey::Unknown` + chẩn đoán nêu bất biến đã vỡ; bỏ CẢ `issues[0]` lẫn `.expect(...)` -- một hàm thuần là thứ `tests/**` lái được cả hai chiều, còn khối closure tại chỗ thì không.
- [x] `src-tauri/src/commands/glossary.rs` -- hai chỗ `:1038` và `:1085` gọi helper `guarded_dict_layers` DÙNG CHUNG (mở phạm vi bản đã có ở `project.rs`, thêm tham số nêu bề mặt gọi cho chẩn đoán); giữ nguyên ba chỗ ở `commands/dict.rs` -- ca "chưa quản lý" là lỗi `setup()`, ca "rỗng" là AD-25 bình thường; gộp chúng là đúng lỗi `project.rs` đã gọi tên hai ngày trước.
- [x] `src-tauri/src/core/glossary/scan.rs` + `mod.rs` -- xoá `pub fn scan_candidates` (vỏ `bool`) và mục re-export ở `mod.rs:285-287`; sửa doc-comment `mod.rs:120`; viết lại `zh_nested_padding` duyệt `freq.iter()` để bỏ `.expect(...)` -- vỏ `bool` biến layer lỗi thành "không có trong từ điển", đúng câu doc-comment của chính module.
- [x] `src-tauri/tests/glossary_scan_contract.rs` + `src-tauri/src/commands/project.rs` (`#[cfg(test)]`) -- chuyển thân adapter `bool → DictionaryProbe` xuống bàn test dưới dạng một helper cục bộ; 20 + 1 chỗ gọi trỏ sang nó -- adapter là tiện nghi của TEST, không phải một API sản phẩm.
- [x] `src-tauri/src/core/glossary/surnames.rs` -- thêm đúng năm cặp `('陳','陈') ('張','张') ('劉','刘') ('楊','杨') ('黃','黄')`; doc-comment ghi phép đo 134/110/272 và nói rõ vì sao KHÔNG nhập trọn -- năm cặp này được đo xác nhận và không cặp nào mơ hồ.
- [x] `src-tauri/tests/glossary_scan_contract.rs` -- ca ④a/④b/④c: nới ngưỡng qua alias, đối chứng ngược bằng một chữ phồn thể không phải họ, và ca **quần thể** khẳng định mọi vế giản có trong `COMMON_SURNAMES` **và** không vế phồn nào tự nó là một họ trong bảng -- ca quần thể là thứ bắt được `於→于` nếu ai đó dán bảng đo vào.
- [x] `src-tauri/tests/glossary_boundary.rs` -- cổng cấu trúc: `core/glossary/**` phơi **0** hàm quét nhận callback `bool`; chép khuôn đọc-nguồn của `config_invariants.rs::the_blocking_wires_run_off_the_main_thread` -- xoá một API không giữ nó khỏi được dựng lại.
- [x] `src-tauri/tests/glossary_import_dialog_contract.rs` -- ca ①a/①b cho hàm thuần mới -- `Err(vec![])` không lái được qua `parse()` thật, nên hàm thuần là đường nghiệm thu duy nhất.
- [x] `src/GlossarySettingsOverlay.vue` -- bốn khai báo px spacing → `calc(var(--space-unit) * N)`, **cả bốn giữ NGUYÊN pixel**: `4px`=×1 · `6px`=×1.5 · `11px`=×2.75 · `12px`=×3; chú thích tại chỗ cho ×2.75 nói nó là khuôn "nét dẫn" 6 chỗ / 5 tệp và vì sao KHÔNG làm tròn lên 12px -- một bội số lẻ có lý do viết ra đọc được hơn một px thô không ai giải thích.
- [x] `src/GlossaryManageOverlay.vue` -- `id` ổn định trên mỗi `<li>` (từ `:key` sẵn có), `aria-activedescendant` trên `<ul>` trỏ hàng ở `manageCursor`, vắng mặt khi danh sách rỗng; 🔵 sửa chú thích `:341-345` cho nó khai đúng phạm vi -- `aria-selected` một mình không nói được con trỏ đang ở đâu khi tiêu điểm DOM không di chuyển.
- [x] `tests/frontend/glossaryManage.test.ts` -- ca ⑤a/⑤b/⑤c -- 37 ca hiện có chạm `aria` **0** lần.

**Vòng rà 1 — thêm 2026-08-26 (xem §Spec Change Log):**
- [x] `src/GlossaryManageOverlay.vue` -- cho `<ul class="gm-list" role="listbox">` một `tabindex="-1"` và **focus chính nó** thay `.gm-panel` khi danh sách có hàng; xét lại `trapTab` (`:110-131`) và đường `focusReturnTargetOnOpen`/`:83` cho khớp; 🔵 sửa lại chú thích vừa viết -- nó khai `<ul>` là *"container giữ tiêu điểm"* trong khi tiêu điểm ở `.gm-panel`, tức bản vá ⑦ vá một chú thích khai quá bằng một chú thích khai quá thứ hai.
- [x] `tests/frontend/glossaryManage.test.ts` -- ca ⑤d: phần tử mang `aria-activedescendant` phải **CHÍNH LÀ** `document.activeElement` khi danh sách có hàng -- ba ca ⑤a/⑤b/⑤c chỉ kiểm GIÁ TRỊ thuộc tính nên cả ba xanh trọn trên một cơ chế trình đọc màn hình không đọc được.
- [x] `src-tauri/tests/config_invariants.rs` -- cổng mới: (a) lọc thêm dòng mở đầu bằng `*` (khuôn `:625` cùng tệp) để một chú thích khối không làm nó đỏ oan; (b) thay phép so chuỗi literal `unwrap_or(&empty_layers)` bằng một phép so CẤU TRÚC bắt mọi `.unwrap_or(&<bất kỳ>)` trên `DictLayers`; (c) `call_count == 2`, không `>= 2` -- một cổng so literal mù với chính khuôn `unwrap_or(&empty)` mà `commands/dict.rs` đang dùng.
- [x] `src-tauri/src/commands/glossary.rs` -- thay khối `match guarded_dict_layers(...) { Some(l) => l, None => &empty_layers }` ở CẢ HAI vỏ bằng `guarded_dict_layers(...).unwrap_or(&empty_layers)` -- hai nhánh viết tay là hai chỗ để gõ nhầm mà không ca nào chạy qua (vỏ `wire::` không gọi được từ `tests/**`); một combinator không có nhánh để đảo.
- [x] `src-tauri/src/core/glossary/scan.rs` -- `:216` và `:222` còn trỏ `scan_candidates` đã bị xoá (`:216` là một intra-doc link gãy) -- đổi sang `scan_candidates_controlled`.
- [x] `src-tauri/src/core/glossary/surnames.rs` + `src-tauri/tests/glossary_scan_contract.rs` (doc ④c) -- sửa HAI mệnh đề sai: `:72` ghi `於` ở *"hàng 6, cột 1"* (đo thật: **hàng 13, cột 11**; hàng 6 cột 2 là `于` GIẢN thể), và câu *"một alias `於→于` sai sẽ nới ngưỡng"* — sai, vì `effective_threshold` đoản mạch ở `surnames.contains(&first)` và `於` ĐÃ có trong bảng ⇒ thêm cặp đó **không đổi hành vi**; lý do loại nó là **mô hình** (nó khai `於` và `于` là một họ, trong khi bảng coi chúng là hai). Mệnh đề "nới ngưỡng cho chuỗi không phải tên người" vẫn ĐÚNG cho `鬍 週 鬱 餘 衚` -- cả năm đã đo là KHÔNG có trong `COMMON_SURNAMES`.
- [x] `src-tauri/tests/glossary_scan_contract.rs` -- doc-comment ④c nói rõ giới hạn: ca này KHÔNG bắt được lớp *"chữ phồn không phải họ, vế giản là họ thật"* (`衚→胡`, `鬍→胡`) -- và sửa câu *"duyệt từng cặp qua ca ④c"* trong sổ nợ, vì theo đúng câu đó vẫn thêm nhầm `衚→胡` được.
- [x] `scripts/check-debt-owner.mjs` -- nâng `ITEM_FLOOR` 443 → **490** (`0,85 × 577 = 490,45`, làm tròn **XUỐNG** đúng bài học của lượt sửa 2026-08-22 ngay trong doc-comment đó) -- 443/577 = **76,8 %**, dưới dải 80–85 % mà chính doc-comment của biến này đặt; lượt vá này nối thêm mục vào sổ mà chưa xét lại sàn.
- [x] `_bmad-output/implementation-artifacts/deferred-work.md` -- nói rõ ĐƠN VỊ ĐẾM (*"tám **bản vá** phủ mười một **vị trí**; ba vị trí còn lại bị bác"* — 8 ≠ 11 vì ④ gộp hai chỗ gọi và ⑥ gộp bốn khai báo px), và ghi `.lookup-citation` (`panels/LookupRecord.vue:312`) mang **11px thứ hai** ở `margin` chưa được đếm vào khuôn "nét dẫn".
- [x] `_bmad-output/implementation-artifacts/deferred-work.md` -- nối `→` vào mục `Cụm F`: tám mục đã đóng, **ba mục bị bác kèm phép đo và dư địa có chủ**, 🔵 sửa tại chỗ con số *"mười bảy"*; thêm hai mục nợ mới (16 tệp px thô kèm số đo; ~105 họ phồn thể kèm bảng 134 cặp) -- không xoá mục, không làm tròn lên.

**Acceptance Criteria:**
- Given hàm thuần mới nhận `Vec::new()`, when chạy `cargo test --test glossary_import_dialog_contract`, then nó trả `MessageKey::Unknown` và **không** ca nào panic; khôi phục `.expect(...)` ⇒ ca ①b ĐỎ.
- Given `guarded_dict_layers` bị thay lại bằng `unwrap_or(&empty_layers)` ở một trong hai chỗ, when chạy bộ cổng cấu trúc, then ĐỎ nêu đích danh chỗ đó.
- Given `pub fn scan_candidates` được dựng lại ở `core/glossary/scan.rs`, when chạy `cargo test --test glossary_boundary`, then cổng cấu trúc ĐỎ; when không dựng lại, then `cargo test --locked` xanh trọn.
- Given cặp `('陳','陈')` bị gỡ khỏi bảng alias, when chạy `cargo test --test glossary_scan_contract`, then ca ④a ĐỎ; given thêm `('於','于')` vào bảng, then ca ④c ĐỎ.
- Given `:aria-activedescendant` bị gỡ khỏi `<ul>`, when chạy `npx vitest run tests/frontend/glossaryManage`, then ca ⑤a và ⑤b ĐỎ.
- Given lượt vá đã xong, when chạy `git status --short src/`, then đúng **hai** tệp `.vue` đổi và **0** pixel render nào của `GlossarySettingsOverlay` khác trước — khuôn "nét dẫn" 6 chỗ / 5 tệp giữ nguyên 13px ở cả sáu.
- Given không gỡ gì cả, when chạy trọn `pre-push`, then **0 ca đỏ** ở cả 11 cổng, vitest, build và `cargo test --locked`.

## Spec Change Log

### Vòng rà 1 — 2026-08-26 (`review_loop_iteration` 0 → 1)

**Phát hiện kích hoạt** *(lăng kính blind-hunter, tôi đã tự kiểm lại trên mã trước khi nhận)*:
`aria-activedescendant` của bản vá ⑦ nằm trên `<ul class="gm-list">`, một phần tử **không bao giờ
giữ tiêu điểm DOM** — tiêu điểm ở `.gm-panel` (`:275`, `panel.value?.focus()` `:83`), phím bắt ở
`.gm-scrim` (`:273`). WAI-ARIA chỉ cho trình đọc màn hình tôn trọng thuộc tính này trên phần tử
ĐANG giữ tiêu điểm ⇒ bản vá không giao được điều nó khai, và ba ca ⑤a/⑤b/⑤c vẫn xanh trọn vì cả
ba chỉ kiểm **giá trị thuộc tính**.

**Vì sao KHÔNG hoàn nguyên toàn bộ mã.** Phân loại đầu là `intent_gap` (gốc chạm ba hàng ⑤ trong
khối đóng băng). Ice giải bằng cách chọn *"tiêu điểm sang `<ul>`, giữ thuộc tính"* — lời giải đó
làm **ba hàng ⑤ vẫn đúng nguyên văn**, nên khối đóng băng không cần sửa một chữ. Gốc còn lại rơi
xuống các mục NGOÀI băng (§Code Map ⑦ chép một mệnh đề sai; §Tasks và §Verification chưa bao giờ
đòi một phép kiểm TIÊU ĐIỂM) ⇒ xử theo đường `bad_spec`: sửa spec trước, tái dẫn xuất phần ⑦, giữ
bảy bản vá đã nghiệm thu xong.

**Đã sửa:** §Code Map ⑦ (mệnh đề *"`tabindex="-1"` trên cả `<ul>` lẫn `<li>`"* — đo lại: `<ul>`
**không có** `tabindex`; đây là chỗ mệnh đề sai đi vào, và bản thi hành đã chép nó xuống chú
thích) · §Tasks (chín task vòng rà 1) · §Verification (phép kiểm tiêu điểm + hai đối chứng gỡ mới).

**Trạng thái xấu đã tránh:** giao một bản vá a11y mà trình đọc màn hình không đọc, kèm một
chú thích khai rằng nó đã đóng lỗ hổng — tức tái sản xuất **đúng** lớp lỗi
`khoi-phuc-trung-thanh-khong-phai-dung` mà chính mục ⑦ sinh ra để đóng, lần thứ hai trong cùng
một lượt vá.

**KEEP — phải sống sót qua lượt tái dẫn xuất:**
1. `manageOptionId()` là **một** hàm dùng chung cho cả `<li :id>` lẫn `<ul :aria-activedescendant>`;
   không chép công thức ghép chuỗi ở hai chỗ. `:` đổi thành `-` vì `:` mở một pseudo-class trong
   `querySelector`.
2. `manageCurrentRow` (`glossaryManageState.ts:196`, `.at(cursor)`) là nguồn của hàng đang chọn —
   không đọc `manageFilteredRows[manageCursor]` lần thứ hai.
3. Bảy bản vá kia (①–⑥ và cổng cấu trúc của ②) đã nghiệm thu xong bằng năm phép GỠ-CHỖ-NỐI —
   **không** dẫn xuất lại, không đụng.
4. Ca ⑤c chứng minh `<ul>` không render khi danh sách rỗng; giữ nguyên hình dạng đó.

## Design Notes

**Vì sao ba phát hiện bị bác vẫn để lại dư địa, và dư địa đó là gì.** Một tiền đề sai không làm kết luận sai — nên mỗi mục bị bác được đọc lại một lần nữa bằng tiền đề khác:

1. **`project.rs` "hai `SELECT` ngược thiết kế"** — chỗ thật là `filter_and_enqueue_current_import_scan` (`project.rs:435-467`) gọi `core/glossary/store.rs:302-328 resolved_source_terms`, hai lượt `SELECT source_term FROM glossary_entry ORDER BY source_term` không `WHERE`/`LIMIT`. Doc-comment `project.rs:432-434` khai thẳng lượt khoá này *"xác nhận work_id, **lọc hai tầng** và enqueue"*, và `:500-518` nói khoá CPU-nặng đã tách ra pha riêng (`:544-561`). ⇒ Mã làm đúng điều nó khai. **Dư địa thật:** hai lượt quét toàn bảng không trần nằm trong vùng khoá — một câu hỏi **chi phí**, chưa ai đo, không phải một vi phạm kiến trúc. Nợ có chủ.
2. **`insert_candidate` "không loại thuật ngữ đã có"** — 0 chỗ gọi sản phẩm, bị `GLOSSARY_ONLY_SURFACE` (`glossary_boundary.rs:142-143`) khoá, và `glossary_contract.rs:1709-1764` đã ghim hành vi này **có chủ ý** kèm chữ *"GHIM HÀNH VI ĐANG KẸT — KHÔNG PHẢI HÀNH VI MONG MUỐN"*.
3. **"không kiểm lại tầng Global"** — đường sản phẩm ĐÃ lọc cả hai tầng: `resolved_source_terms` nạp `global` **và** `work` rồi `resolver.apply_override(...)`, và `filter_import_scan_candidates_by_scope` `retain` theo tập đã phân giải **trước** khi enqueue. **Dư địa thật:** doc-comment `store.rs:299-301` tự khai `WHERE NOT EXISTS` trong câu `INSERT` chỉ canh tầng **Work**, nên còn một cửa sổ đua hẹp giữa ảnh chụp và lượt ghi cho một thuật ngữ vừa được đẩy lên Global. Đóng nó cần snapshot chéo hai kho ⇒ **C4/một `AD` mới**, không phải lượt này.

**Hình dạng bản vá ①.** Hôm nay khối `map_err` là một closure tại chỗ nên `Err(vec![])` không lái được từ `tests/**` bằng bất cứ đường nào — `parse()` thật không sinh nổi nó. Tách thành hàm thuần **không** phải để cho đẹp: đó là điều kiện để mệnh đề ①b có một phép kiểm chạy được thay vì một lời khai.

```rust
// Bat bien "Err chi di kem issues khong rong" giu bang CACH DUNG trong parse(), khong
// bang kieu Result<_, Vec<_>> -- sau diem return Err o exchange.rs. Mot luot sua sau
// tra Err(vec![]) la du, va panic = "abort" bien no thanh cai chet cua tien trinh.
fn first_issue_or_unknown(issues: Vec<ParseIssue>) -> IpcError { … }
```

**Hình dạng bản vá ⑤.** Bảng alias giữ nguyên vai *"chuẩn hoá hình dạng chữ trước phép tra bảng"* — **không** trộn vào `COMMON_SURNAMES`, vì tham số `surnames` là điểm tiêm có chủ cho test. Ca quần thể ④c canh hai mệnh đề mà bảng đo đã cho thấy là dễ vỡ nhất: vế giản phải là một họ thật, và vế phồn **không** được tự nó là một họ khác trong bảng (`於` là ca duy nhất trong 134 cặp mắc điều này).

## Verification

**Commands:**
- `cd src-tauri && cargo test --locked --test glossary_scan_contract --test glossary_boundary --test glossary_import_dialog_contract --test glossary_commands_contract` -- expected: ≥ 27+11+22+29 ca, 0 đỏ.
- `npx vitest run tests/frontend/glossaryManage tests/frontend/glossarySettings` -- expected: ≥ 58 + 3 ca mới, 0 đỏ.
- `npm run check:lint && npm run check:tokens && npm run check:i18n && npm run check:debt-owner` -- expected: exit 0. `check:i18n` Kiểm A cấm chữ có dấu ở vị trí mã trong `src-tauri/src/**`; `check:debt-owner` đòi literal `Chủ:` trong mục nợ mới.
- `npm run build && cd src-tauri && cargo test --locked` -- expected: 0 đỏ trên toàn bộ (`build` TRƯỚC `cargo test`).

**Manual checks:**
- **Vòng rà 1 — hai phép GỠ CHỖ NỐI MỚI:** (6) ~~bỏ `tabindex="-1"` khỏi `<ul>`~~ trả `focus()` về `.gm-panel` ⇒ ca ⑤d phải ĐỎ, trong khi ⑤a/⑤b/⑤c vẫn XANH — chính sự bất đối xứng đó là bằng chứng ba ca cũ không phủ được vế tiêu điểm; (7) đổi `guarded_dict_layers(...).unwrap_or(&empty_layers)` sang một `unwrap_or(&empty)` đổi tên biến ⇒ cổng cấu trúc đã nới phải ĐỎ (bản so literal cũ sẽ XANH trên đúng thí nghiệm đó).
  🔴 **SỬA 2026-08-26 — vế gạch ngang của (6) đã được ĐO và nó KHÔNG đỏ.** Xoá đúng dòng
  `tabindex="-1"` khỏi thẻ `<ul>`, để nguyên mọi thứ khác ⇒ `npx vitest run
  tests/frontend/glossaryManage` cho **37/37 XANH**, ca ⑤d không đỏ; khôi phục ⇒ vẫn xanh.
  Nguyên nhân: `happy-dom` cho `.focus()` thành công trên MỌI phần tử bất kể `tabindex`, còn
  engine thật thì `.focus()` lên một phần tử không focus được là một **no-op**. ⇒ Câu tôi viết
  trong lượt sửa spec này ("bỏ `tabindex` ⇒ ⑤d phải ĐỎ") là một **lời khai chưa đo**, đúng lớp
  lỗi mà cả cụm F tồn tại để đóng — nay đã đo và ghi nợ **có chủ** (`deferred-work.md`, chủ:
  story đầu tiên mở rộng bộ e2e sang Glossary) thay vì chấm đạt. Vế còn lại của (6) — trả
  `focus()` về `.gm-panel` — VẪN là một phép gỡ hợp lệ và ⑤d đỏ đúng trên nó.
- **Vòng rà 1 — đối chứng sàn sổ nợ:** sau khi nâng `ITEM_FLOOR`, hạ tổng số mục xuống dưới sàn mới (chạy `--file` trên một bản cắt) phải cho cổng ĐỎ; và `npm run check:debt-owner` trên sổ thật phải XANH.
- **Năm phép GỠ CHỖ NỐI** của lượt đầu, chạy TỪNG cái rồi khôi phục ngay: (1) khôi phục `.expect(...)` ở đường lỗi rỗng; (2) trả một chỗ về `unwrap_or(&empty_layers)`; (3) dựng lại `pub fn scan_candidates`; (4) gỡ cặp `('陳','陈')`, rồi riêng một lượt thêm `('於','于')`; (5) gỡ `:aria-activedescendant`. Ghi **tên ca** và **số ca đỏ thật** cho từng lượt vào §Completion Notes — một con số suy ra không phải một số đo.
- **Đếm quần thể sau lượt vá:** `grep -c '\.expect(\|\.unwrap('` trên mã sản phẩm `src-tauri/src/**` phải đi từ **4** xuống **3**; `grep -c 'unwrap_or(&empty_layers)'` trong `commands/glossary.rs` phải là **0** và trong `commands/dict.rs` vẫn là **3**.
  🔵 **SỬA 2026-08-26 (đo lúc thực thi, §Completion Notes).** Con số **3** ở trên đã LỆCH với
  chính Tasks/Code Map của spec này: cả HAI `.expect(` gốc (`scan.rs:319` VÀ `commands/glossary.rs:770`)
  bị xoá trong lượt vá — Task ① xoá cái thứ hai qua `first_issue_or_unknown`, Task ③ xoá cái
  đầu qua `zh_nested_padding` duyệt `freq.iter()` (cả hai đều nằm trong danh sách "đúng bốn
  `.expect(`" mà chính Code Map ① liệt). Số đo THẬT (loại trừ `#[cfg(test)]`, loại comment):
  **2** — `lib.rs:454` và `core/segment/split.rs:300`. ⇒ **4 xuống 2**, không phải 3. Đo thứ
  hai (`unwrap_or(&empty_layers)` trong `commands/dict.rs`) cũng lệch CHỮ, không lệch Ý: `dict.rs`
  dùng biến `empty` (`layers.unwrap_or(&empty)`), không `empty_layers` — literal grep trên trả
  **0**, không **3**; mệnh đề THẬT (§Never: "ba chỗ ở `commands/dict.rs` KHÔNG đụng tới") vẫn
  đúng và đã đo lại — `git status --short` xác nhận `commands/dict.rs` không đổi một dòng nào.
- **Thị giác — mệnh đề "0 pixel đổi" phải ĐO, không nhìn:** `git diff src/GlossarySettingsOverlay.vue` chỉ được chứa bốn dòng spacing, và mỗi dòng phải tính ra ĐÚNG giá trị cũ (×1=4 · ×1.5=6 · ×2.75=11 · ×3=12). `git status --short src/` phải cho **đúng hai** tệp (`GlossarySettingsOverlay.vue`, `GlossaryManageOverlay.vue`) — một tệp `.vue` thứ ba xuất hiện là §Never đã bị phá.

## Completion Notes

**Tám bản vá đã đóng** (①–⑦, cộng cổng cấu trúc mới cho ④ và cho ②) — xem `→` mới nối vào mục `Cụm F` của `deferred-work.md` để biết diễn giải đầy đủ từng mục.

**Năm phép GỠ-CHỖ-NỐI đã chạy, mỗi lượt xong khôi phục ngay** (§Boundaries — mỗi bản vá kèm phép đối chứng):

1. **Khôi phục `.expect("Err chi dung khi issues khong rong")` ở nhánh `None` của `first_issue_or_unknown`** (`commands/glossary.rs`) → `cargo test --test glossary_import_dialog_contract` cho **1 ca đỏ**: `first_issue_or_unknown_returns_message_key_unknown_without_panicking_when_the_list_is_empty` (panic tại đúng dòng khôi phục, thay vì trả `MessageKey::Unknown`). 23 ca còn lại của tệp vẫn xanh.
2. **Trả `glossary_marks_for_chapter` về `layers.as_deref().unwrap_or(&empty_layers)`** (bỏ `guarded_dict_layers`) → `cargo test --test config_invariants` cho **1 ca đỏ**: `commands_glossary_uses_the_shared_guarded_dict_layers_helper_not_a_bare_unwrap_or_empty` (cổng cấu trúc mới, thêm trong lượt này để đóng đúng mệnh đề AC "guarded_dict_layers bị thay lại ⇒ cổng cấu trúc ĐỎ nêu đích danh chỗ đó" — spec gốc chưa có sẵn cổng này, tự dựng theo khuôn `the_blocking_wires_run_off_the_main_thread`). 22 ca còn lại của tệp vẫn xanh.
3. **Dựng lại `pub fn scan_candidates` (vỏ `bool`) trong `core/glossary/scan.rs`** → `cargo test --test glossary_boundary` cho **1 ca đỏ**: `zero_scan_functions_under_core_glossary_accept_a_bool_dictionary_callback` (cổng cấu trúc mới của lượt này), nêu đích danh `core/glossary/scan.rs:128`. 12 ca còn lại của tệp vẫn xanh.
4. **(a) Gỡ cặp `('陳','陈')` khỏi `TRADITIONAL_SURNAME_ALIASES`** → `cargo test --test glossary_scan_contract` cho **1 ca đỏ**: `a_new_traditional_surname_alias_below_threshold_by_one_is_kept`. **(b) Riêng một lượt khác, THÊM `('於','于')`** (giữ nguyên năm cặp cũ) → cùng tệp cho **1 ca đỏ**: `every_traditional_surname_alias_maps_to_a_real_surname_and_the_traditional_side_is_not_itself_a_listed_surname`, nêu đích danh `於`/`于`. Cả hai lượt: 29 ca còn lại của tệp vẫn xanh.
5. **Gỡ `:aria-activedescendant` khỏi `<ul class="gm-list">`** (`src/GlossaryManageOverlay.vue`) → `npx vitest run tests/frontend/glossaryManage` cho **2 ca đỏ**: `⑤a con trỏ ở hàng thứ n …` và `⑤b con trỏ di chuyển n → n+1 …` (đúng như AC5 dự đoán: "then ca ⑤a và ⑤b ĐỎ"; ⑤c không đỏ vì nó canh nhánh `<ul>` hoàn toàn không render khi rỗng, không phụ thuộc thuộc tính này). 34 ca còn lại của tệp vẫn xanh.

Sau mỗi lượt, tệp bị sửa được khôi phục lại nguyên trạng (`cp` từ bản sao lưu trước khi gỡ) và cổng liên quan chạy lại xanh trước khi sang lượt kế tiếp.

**Đối chứng quần thể sau lượt vá (đo lại, không suy):**
- `.expect(`/`.unwrap(` trên mã sản phẩm `src-tauri/src/**`, loại `#[cfg(test)]` và dòng comment: **4 → 2** (không phải 3 như Verification gốc ghi — xem 🔵 sửa tại chỗ ở mục Verification; cả hai `.expect(` gốc trong danh sách bốn của Code Map ① đều bị xoá theo đúng Tasks/Code Map, không phải một).
- `unwrap_or(&empty_layers)` trong `commands/glossary.rs`: **0** (đúng AC). Trong `commands/dict.rs`: tệp **không đổi một dòng** (`git status --short` xác nhận) — dict.rs dùng biến `empty`, không `empty_layers`, nên literal grep trả 0 chứ không 3; mệnh đề thật (ba chỗ giữ nguyên) đã đúng.
- `git status --short src/`: đúng **hai** tệp — `GlossaryManageOverlay.vue`, `GlossarySettingsOverlay.vue`.
- `git diff src/GlossarySettingsOverlay.vue`: bốn dòng spacing đổi (`gap`, `padding` của `.gs-input`, `padding-left` của `.gs-alert`, `padding` của `.gs-save`) + một khối chú thích mới cho `.gs-alert` (giải thích khuôn "nét dẫn" 6 chỗ/5 tệp và vì sao không làm tròn 12px, theo đúng Task đòi "chú thích tại chỗ") — bốn giá trị tính đúng ×1=4px · ×1.5=6px · ×2.75=11px · ×3=12px, không tệp nào khác trong khuôn "nét dẫn" (`ShortcutsOverlay.vue` · `AttributionOverlay.vue` · `panels/LookupPanel.vue` · `panels/LookupRecord.vue`) bị chạm.
- Baseline `569 ca / 43 tệp` (vitest) và `glossaryManage.test.ts 37 ca` mà spec §Baseline ghi cũng đã RỘT: đo lại trên `85a7ab2` bằng `git stash` thật sự cho **561 ca / 43 tệp** và **33 ca** cho `glossaryManage.test.ts`. Sau lượt vá (thêm đúng 3 ca ⑤a/b/c): **564 ca / 43 tệp**, khớp `561 + 3`.
  🔵 **SỬA 2026-08-26 (lượt kiểm sau thi hành).** Câu *"Không sửa số trong §Baseline (nằm trong
  khối `frozen-after-approval`)"* mà mục này viết ban đầu **SAI về dữ kiện**: khối đóng băng kết
  thúc ở `</frozen-after-approval>` **trước** §Code Map, còn §Baseline nằm bên trong §Code Map,
  tức **ngoài** khối đóng băng và sửa được. Số cũ vì thế đã được sửa TẠI CHỖ ở §Baseline (kèm 🔵),
  đúng luật *"mệnh đề hết đúng thì sửa tại chỗ, đừng để nó lặng lẽ sai"* của `AGENTS.md`. Lý do
  BỎ QUA thì sai; con số thì đúng — và đó là hai chuyện khác nhau.

**Bàn giao còn lại (không thuộc phạm vi cụm F, đã ghi nợ có chủ trong `deferred-work.md`):**
- Ba mục e2e/NFR2 — giữ nguyên mở, chủ: story đầu tiên mở rộng bộ e2e sang Glossary.
- 16 tệp `.vue` còn lại có px thô — mục nợ mới, chủ: lượt vá CSS diện rộng kế tiếp hoặc lượt dựng Kiểm spacing thứ tám cho `check-tokens.mjs`.
- ~104 họ phồn thể chưa có alias (🔵 sửa 2026-08-26, vòng rà 2 P6 — 110 họ đo được trừ 6 cặp đã có mã (5 mới + `蕭` sẵn có) = 104, không 105; 128/134 cặp đo được còn lại, bảng đầy đủ đã ghi vào `deferred-work.md`) — mục nợ mới, chủ: lượt vá bảng họ kế tiếp, PHẢI đo lại và duyệt TỪNG cặp qua CẢ ca ④c LẪN đối chiếu hình dạng họ trước khi thêm (④c một mình không bắt được lớp "chữ phồn không phải họ, vế giản là họ thật" -- `衚→胡`, xem doc-comment ④c).
- Nợ (b)/(c) của hai phát hiện bị bác — chi phí quét trong lúc giữ khoá (chưa ai đo) và cửa sổ đua hẹp giữa ảnh chụp/ghi cho tầng Global (cần C4/một `AD` mới) — cả hai ghi có chủ trong `deferred-work.md`, không đụng trong lượt này.

**Verification cuối cùng (toàn bộ xanh, đo lại sau các lượt gỡ-chỗ-nối ở trên):**
- `cargo test --locked` (toàn bộ `src-tauri`): 28 tệp/khối, **0 đỏ**.
- `npx vitest run` (toàn bộ `tests/frontend`): 43 tệp / 564 ca, **0 đỏ**.
- `npm run check:lint && npm run check:tokens && npm run check:i18n && npm run check:debt-owner`: exit 0 cho cả bốn.
- `npm run build`: thành công (gồm `vue-tsc --noEmit` hai lượt).
- `npm run check:gates`: ba danh sách cổng khớp nhau.

## Completion Notes — Vòng rà 1 (2026-08-26, `review_loop_iteration` 1)

**Chín mục đã sửa** (§Tasks "Vòng rà 1"):

1. **Cơ chế tiêu điểm của ⑦ vá đúng** — `src/GlossaryManageOverlay.vue`: `<ul class="gm-list" role="listbox">` nay mang `tabindex="-1"` VÀ template ref `list`. `focusInitialTarget()` (gọi từ `watch(manageOverlayIsOpen, …)` lúc mở) focus `<ul>` khi danh sách ĐÃ có hàng lúc đó, ngược lại `panel`. Watcher THỨ HAI, `watch(manageFilteredRows, …)`, bắt lượt danh sách chuyển sang "có hàng" SAU KHI mở (trường hợp thường gặp nhất — `openGlossaryManage()` đặt `manageOverlayIsOpen=true` NGAY rồi mới `await` tải danh sách, nên gần như luôn `focusInitialTarget()` chạy khi `manageStatus` còn `'unknown'`) và dời tiêu điểm từ `panel` sang `<ul>`, CHỈ khi tiêu điểm còn đang đúng ở `panel` (không cướp tiêu điểm khỏi một ô người dùng đã chủ động chuyển tới).
   ⚠️ **Một khuyết tật thật bắt được VÀ vá được ngay trong lượt viết ca ⑤d** (không phải một phát hiện của vòng rà — của chính tôi, khi viết bài kiểm chứng): bản đầu của `focusInitialTarget()` không kiểm `list.value !== null` trước khi gọi `.focus()` — khi trạng thái JS (`manageStatus`/`manageFilteredRows`) đã phản ánh "đã tải xong" nhưng DOM (`<ul>`, qua template ref) CHƯA kịp patch (một khoảng hở thật giữa hai flush của Vue, đo được bằng `console.log` tại chỗ), `list.value?.focus()` là một no-op im lặng và hàm `return` luôn, bỏ qua nhánh dự phòng `panel.value?.focus()` — tiêu điểm ở lại `document.body`. Sửa: thêm điều kiện `list.value !== null` vào nhánh rẽ, để một `list.value` còn `null` rơi đúng về nhánh `panel`.
   `trapTab` (`:157-180`, số dòng sau lượt vá) được XÉT LẠI, KHÔNG cần sửa: `focusableWithin()` đã loại mọi phần tử `[tabindex="-1"]` khỏi `stops`, nên `<ul>` (nay `tabindex="-1"`) không lọt vào `stops`, đúng như `panel` chưa từng lọt vào — nhánh `index === -1` của `trapTab` đã sẵn xử lý đúng cho tiêu điểm xuất phát từ MỘT trong hai. Đã thêm doc-comment ghi lại kết luận này (không phải suy luận suông — xem ca kiểm chứng dưới).
   Chú thích `:404-426` (đánh số sau lượt vá) viết lại HOÀN TOÀN — bản cũ khai `<ul>` là *"container giữ tiêu điểm"* khi tiêu điểm thật ở `panel`; bản mới kể đúng trình tự: bản 2026-08-24 (chỉ `aria-selected`) → bản ⑦ đầu (2026-08-26, thêm `aria-activedescendant` nhưng KHÔNG focus `<ul>` — SAI, đúng phát hiện của vòng rà) → bản này (tiêu điểm thật chuyển tới `<ul>`).

2. **Ca ⑤d mới** — `tests/frontend/glossaryManage.test.ts`: mount component TRƯỚC khi mở lớp phủ (ngược thứ tự ⑤a/⑤b/⑤c cố ý), để `watch(manageOverlayIsOpen, …)` thấy đúng một lượt chuyển `false → true` và THẬT SỰ chạy — ba ca cũ mở TRƯỚC rồi mount nên watcher không bao giờ chạy (Vue `watch()` không tự chạy lúc mới đăng ký), tức ba ca cũ chỉ đọc GIÁ TRỊ `aria-activedescendant` do template tính sẵn, không đi qua đường `.focus()` sản phẩm dùng thật. Ca mới khẳng định `document.activeElement === <ul> element` (KHÔNG khẳng định `document.activeElement.id === activeId` — đó là hiểu sai ARIA: `<ul>` giữ tiêu điểm thật nhưng KHÔNG mang `id`, `aria-activedescendant` là một THAM CHIẾU tới `<li>` "đang chọn ảo", không phải id của chính nó; sửa lại đúng ngay khi viết ca, trước khi giao).

3. **`config_invariants.rs` — ba khuyết tật vá cả ba:**
   (a) Bộ lọc dòng comment nay loại CẢ `//` lẫn dòng mở đầu bằng `*` (khuôn `:625` cùng tệp) — một khối `/* … */` không còn làm cổng đỏ oan.
   (b) Đổi từ so chuỗi literal `contains("unwrap_or(&empty_layers)")` sang so CẤU TRÚC: mọi câu lệnh (cắt bởi `;` trên văn bản đã gộp một dòng) chứa `layers.as_deref()` — điểm DUY NHẤT mở `Option<&DictLayers>` từ `try_state` — cũng phải chứa `guarded_dict_layers(`. Đối chứng ĐÃ CHẠY: đổi MỘT vỏ sang `let empty = empty_layers; let layers = layers.as_deref().unwrap_or(&empty);` (đúng anti-pattern dưới TÊN BIẾN khác, hình dạng mà bản literal cũ sẽ bỏ lọt) ⇒ cổng ĐỎ (qua nhánh đếm `call_count` trước, vì gỡ `guarded_dict_layers(` cũng hạ số lần gọi xuống 1 — cùng một nguyên nhân gốc bị bắt bằng hai lớp phòng thủ khác nhau).
   (c) `call_count >= 2` → `== 2`, đúng khuôn đếm TUYỆT ĐỐI của `the_blocking_wires_run_off_the_main_thread`.
   Thông báo assert cũng viết lại — không còn khuyên `match`/`if let` (đúng thứ mục 4 dưới đây vừa gỡ).

4. **`commands/glossary.rs` — bỏ hai khối `match` viết tay, về combinator** `guarded_dict_layers(layers.as_deref(), "<bề mặt>").unwrap_or(&empty_layers)` ở cả `glossary_marks_for_chapter` lẫn `glossary_pending_candidates`. Chú thích cạnh đó SỬA để không còn tự nhắc lại chuỗi `unwrap_or(&empty_layers)` NGUYÊN VĂN trong PHẦN COMMENT (bài học đo được: comment cũ tự nhắc chuỗi cấm bằng ĐÚNG hình dạng literal, và cổng cấu trúc mới ở mục 3 lọc theo dòng — nên diễn đạt lại bằng cách tách `unwrap_or` và `&empty_layers` ra hai vế của câu, không viết liền).

5. **`core/glossary/scan.rs` — hai intra-doc link/nhắc tên gãy sửa xong:** dòng doc-comment của `count_zh_candidates` (hai chỗ) đổi `[`scan_candidates`]`/`scan_candidates` (tên hàm đã xoá) thành `[`scan_candidates_controlled`]`/`scan_candidates_controlled`. `cargo build --lib` sạch, không cảnh báo intra-doc-link nào.

6. **`surnames.rs` — sửa HAI mệnh đề sai trong doc-comment `TRADITIONAL_SURNAME_ALIASES`, viết lại thành hai LỚP tách biệt:**
   - **Vị trí**: `於` thật sự ở **hàng 13, cột 11** của `COMMON_SURNAMES` (đếm lại từng hàng/cột trực tiếp trên mã nguồn) — KHÔNG phải "hàng 6, cột 1" như bản trước ghi; hàng 6 cột 2 mới là `于` (vế GIẢN của chính cặp này, một ký tự khác hẳn).
   - **Cơ chế**: mệnh đề *"alias `於→于` sai sẽ NỚI NGƯỠNG"* — SAI: `effective_threshold` tra `surnames.contains(&first) || TRADITIONAL_SURNAME_ALIASES.iter()...` — `於` ĐÃ có sẵn trong `COMMON_SURNAMES` nên vế ĐẦU của `||` đoản mạch TRƯỚC khi bảng alias được tra tới; thêm cặp `('於','于')` **không đổi một chút hành vi thời gian chạy nào** (đo lại: ca ④c vẫn đỏ khi thêm cặp này, nhưng vì lý do MÔ HÌNH — "vế phồn tự nó là một họ khác" — không phải vì ngưỡng đổi). Lý do THẬT để loại `於→于` là **MÔ HÌNH**: alias khai `於`/`于` là CÙNG một họ, trong khi `COMMON_SURNAMES` coi chúng là HAI họ RIÊNG.
   - **Giữ nguyên, đo lại xác nhận**: mệnh đề "alias sai nới ngưỡng cho chuỗi không mang hình dạng tên người" vẫn ĐÚNG cho `鬍 週 鬱 餘 衚` — cả năm đo lại 2026-08-26 đều KHÔNG có trong `COMMON_SURNAMES` (script đối chiếu trực tiếp với mảng hằng).
   `cargo build --lib` sạch sau khi sửa hai đoạn doc-comment lớn này.

7. **`glossary_scan_contract.rs` doc ④c — GIỚI HẠN THẬT ghi ra:** ca quần thể chỉ chặn lớp *"vế phồn tự nó là một họ KHÁC trong bảng"* (khuôn `於`), KHÔNG chặn lớp *"chữ phồn KHÔNG phải họ, vế giản LÀ một họ thật"* — `衚→胡` (ngõ hẹp) lọt qua CẢ HAI `assert!` y hệt `鬍→胡` lọt qua ca ④b. Mục nợ mới về 128 cặp còn lại (`deferred-work.md`) đã sửa câu *"duyệt từng cặp qua ca ④c"* thành hai bước (đo lại + duyệt TAY qua cả ④c LẪN đối chiếu hình dạng họ), vì theo câu cũ vẫn có thể thêm nhầm `衚→胡`.

8. **`check-debt-owner.mjs` — `ITEM_FLOOR` 443 → 490** (`0,85 × 577 = 490,45`, làm tròn XUỐNG — cùng bài học lượt sửa 2026-08-22 mà chính doc-comment ngay trên đã ghi, không lặp lại lỗi làm tròn LÊN của lượt đó). Đối chứng GỠ-CHỖ-NỐI ĐÃ CHẠY theo đúng cách sàn này THẬT SỰ có thể kiểm: sàn chỉ áp cho Kiểm A trên sổ THẬT (`DEBT_PATH === REAL_DEBT_PATH`) — `--file` KHÔNG kích hoạt nó (theo đúng thiết kế đã ghi trong chính doc-comment của script, không phải một lỗ hổng). Vì vậy phép gỡ-chỗ-nối thật là: sao lưu `deferred-work.md`, GHI ĐÈ nó bằng bản CẮT (3000 dòng đầu, 348 mục) lên đúng đường THẬT, chạy `npm run check:debt-owner` KHÔNG cờ ⇒ **đỏ đúng chỗ** (*"chỉ 348 muc, duoi san 490"*), rồi khôi phục nguyên vẹn (`diff` xác nhận giống hệt bản gốc) ⇒ `check:debt-owner` xanh lại.
   🔵 **SỬA — mệnh đề Verification gốc của vòng rà 1** (*"hạ tổng số mục xuống dưới sàn mới, chạy `--file` trên một bản cắt, phải cho cổng ĐỎ"*) **không đúng như viết**: đã đo — `--file` không bao giờ chạm nhánh sàn (`DEBT_PATH === REAL_DEBT_PATH` luôn `false` khi có `--file`), đúng chủ ý ghi trong doc-comment của chính script (*"Sàn chỉ áp cho Kiểm A trên sổ THẬT… một bản lịch sử đúng là có ít mục hơn"*). Đối chứng thật phải ghi đè sổ THẬT tạm thời, không dùng `--file`.

9. **`deferred-work.md` — hai bổ sung:**
   - **Đơn vị đếm**: thêm đoạn giải thích *"tám BẢN VÁ phủ mười một VỊ TRÍ"* ngay đầu mục `→` của Cụm F — ④ gộp hai chỗ gọi, ⑥ gộp bốn khai báo px ⇒ `1+1+1+2+1+4+1 = 11`, cộng ba vị trí bị bác = **14**, khớp số đã sửa ở §Intent.
   - **`11px` thứ hai**: `panels/LookupRecord.vue:311` `.lookup-citation` mang `margin: 4px 0 0 11px;` NGAY TRÊN `padding-left: 11px;` (`:312`, chỗ đã đếm trong khuôn "nét dẫn") — cùng rule, hai khai báo cùng giá trị, một cái có tên trong khuôn "6 chỗ/5 tệp", cái kia thì không. Ghi vào cả `deferred-work.md` LẪN chú thích `.gs-alert` của `GlossarySettingsOverlay.vue` (không sửa `LookupRecord.vue` — nằm ngoài phạm vi §Never).

**Hai phép GỠ-CHỖ-NỐI MỚI của vòng rà 1, số ca đỏ thật:**

6. **Gỡ cơ chế focus (thay `focusInitialTarget` bằng `panel.value?.focus()` không điều kiện, xoá watcher thứ hai)** → `npx vitest run tests/frontend/glossaryManage` cho **1 ca đỏ**: `⑤d phần tử mang aria-activedescendant phải CHÍNH LÀ document.activeElement…`; ⑤a/⑤b/⑤c VẪN XANH (đúng dự đoán — ba ca đó chỉ đọc giá trị thuộc tính, không đọc tiêu điểm). 36 ca còn lại của tệp vẫn xanh.
   ⚠️ **Giới hạn đo được, ghi thẳng:** phép gỡ **"bỏ `tabindex="-1"` khỏi `<ul>`"** (đúng như liệt kê ở §Manual checks gốc) KHÔNG đỏ được qua vitest — đã tự kiểm bằng một script `happy-dom` độc lập: `element.focus()` THÀNH CÔNG bất kể phần tử có `tabindex` hay không (`document.activeElement === element` sau `.focus()`, có tabindex hay không đều `true`), khác hành vi trình duyệt thật (nơi thiếu `tabindex` làm phần tử KHÔNG programmatically-focusable). Đây đúng lớp giới hạn `tests/AGENTS.md` đã cảnh báo: *"happy-dom KHÔNG phải WebKit. Mọi mệnh đề về hình học, bố cục, hay engine thật thuộc bàn đo/e2e — không thuộc vitest."* `tabindex="-1"` vẫn ĐÚNG và CẦN cho hành vi trình duyệt thật/WebKit; phép gỡ tương đương THẬT SỰ đỏ được qua vitest là gỡ CƠ CHẾ GỌI `.focus()` (đã làm, xanh đúng), không phải gỡ thuộc tính `tabindex`.
7. **Đổi MỘT vỏ (`glossary_marks_for_chapter`) từ `guarded_dict_layers(...).unwrap_or(&empty_layers)` sang `let empty = empty_layers; layers.as_deref().unwrap_or(&empty)`** (bỏ hẳn `guarded_dict_layers(`, đổi TÊN BIẾN fallback để chứng minh cổng không neo vào tên biến) → `cargo test --test config_invariants commands_glossary_uses…` **ĐỎ** — `call_count` tụt còn 1 (nhánh đếm tuyệt đối `== 2` bắt được trước; đã xác nhận riêng phép so CẤU TRÚC — mục *"violations"* — cũng bắt được đúng câu lệnh vi phạm khi cô lập, không chỉ đi nhờ nhánh đếm). Khôi phục ⇒ xanh lại (`cargo test --test config_invariants` 23/23).

**Đối chứng sàn sổ nợ đã chạy (mục 8 ở trên):** xem chi tiết ngay trên — GHI ĐÈ sổ THẬT tạm thời (không dùng `--file`, vì `--file` không chạm nhánh sàn theo đúng thiết kế của script) cho đỏ đúng thông báo *"duoi san 490"*; khôi phục cho xanh lại.

**Một khuyết tật TỰ GÂY RA VÀ TỰ BẮT trong chính lượt vá này, ghi ra vì nó suýt lọt:** chú thích mới viết cho template `<ul>` (mục 1) ban đầu chứa literal `` `<script setup>` `` để chỉ người đọc quay lại khối kịch bản — cụm ký tự này khớp NGUYÊN VĂN regex `vueRegions()` của `scripts/check-i18n.mjs` (`/<(script|style)\b[^>]*>/gi`, không hiểu HTML comment), mở một vùng "script" GIẢ kéo dài tới HẾT TỆP và làm **56 phép kiểm** Kiểm A đỏ giả trên toàn bộ phần còn lại của `GlossaryManageOverlay.vue` (mọi comment tiếng Việt SAU điểm đó, kể cả những dòng đã có TỪ TRƯỚC lượt vá này, bị đọc nhầm thành "vị trí mã"). Bắt được ngay khi chạy `npm run check:i18n` sau khi hoàn thành phần code, TRƯỚC khi báo cáo — sửa bằng cách đổi câu thành "xem khối kịch bản phía trên" (bỏ hẳn cụm `<script setup>` dạng thẻ). `npm run check:i18n` xanh lại sau sửa. Bài học: **không viết một chuỗi trông giống thẻ mở `<script`/`<style>` bên trong bất kỳ vùng nào của một tệp `.vue`, kể cả trong một comment** — cổng quét vùng không hiểu ngữ cảnh comment.

**Verification cuối cùng của vòng rà 1 (đo lại sau các lượt gỡ-chỗ-nối, toàn bộ xanh):**
- `cargo test --locked` (toàn bộ `src-tauri`): 28 tệp/khối, **0 đỏ** (23 ca ở `config_invariants`, tăng từ 22 — thêm ca cổng mục 3).
- `npx vitest run` (toàn bộ `tests/frontend`): 43 tệp / **565 ca**, **0 đỏ** (tăng từ 564 — thêm ca ⑤d).
- `npm run check:lint && npm run check:tokens && npm run check:i18n && npm run check:debt-owner`: exit 0 cho cả bốn.
- `npm run build`: thành công.
- `npm run check:gates`: ba danh sách cổng khớp nhau.
- `git status --short src/`: vẫn đúng **hai** tệp — `GlossaryManageOverlay.vue`, `GlossarySettingsOverlay.vue`.

**Còn lại / rủi ro cần Ice biết:**
- Ca ⑤d phụ thuộc thời gian chờ `nextTick()` kép để hai watcher (mở → focus tạm thời trên `panel`; danh sách có hàng → dời sang `<ul>`) đều kịp chạy trong môi trường vitest (mock IPC gần như tức thời). Hành vi SẢN PHẨM thật (IPC thật, có độ trễ) an toàn hơn kịch bản test (khoảng cách thời gian giữa hai lượt reactive rộng hơn nhiều) — không phải một điều kiện đua trong sản phẩm, chỉ là một điều kiện phải xử lý cẩn thận LÚC VIẾT TEST.
- Giới hạn `tabindex` không đỏ-được-qua-vitest (ghi ở phép gỡ #6) là một khoảng hở NGHIỆM THU thật: không phép kiểm tự động nào trong kho hiện xác nhận `<ul>` THỰC SỰ programmatically-focusable trên WebKit/Chromium thật. Thuộc về e2e (chưa mở trong lượt cụm F, cùng lý do ba món e2e/NFR2 khác đứng ngoài phạm vi này).

## Completion Notes — Vòng rà 2 (2026-08-26, `review_loop_iteration` giữ nguyên **1**)

**Tám mục đã sửa** (P1–P8, không loopback — không mục nào lật spec):

1. **P1 (🔴 HỒI QUY do chính vòng rà 1 gây ra) — `src/GlossaryManageOverlay.vue`: vệ đối xứng N→0 cho tiêu điểm.** Watcher gốc (`watch(manageFilteredRows, …)` mục ④ cụm F) chỉ có chiều 0→N. Thêm watcher THỨ HAI: khi danh sách VỀ rỗng **và** `document.activeElement === list.value`, dời tiêu điểm về `panel` (`nextTick(() => panel.value?.focus())`). Không có vệ này, xoá hàng CUỐI CÙNG trong khi tiêu điểm ở `<ul>` làm `<ul>` bị gỡ khỏi DOM cùng lúc giữ tiêu điểm ⇒ trình duyệt đẩy tiêu điểm ra `document.body`, NGOÀI `.gm-scrim` ⇒ `trapTab`/`onEscape`/`onKeydown` (nghe qua bubbling) ngừng phản hồi.
2. **P2 — hai ca test mới** (không ba — xem giải thích đơn vị bên dưới), `tests/frontend/glossaryManage.test.ts`:
   - **⑤e** — chiều N→0 của P1: hai nhịp XOÁ THẬT (`state.deleteGlossaryManageEntry()` gọi trực tiếp, KHÔNG `scrim.trigger('keydown', …)` — `.trigger()` bắn thẳng vào phần tử chỉ định, không định tuyến theo `document.activeElement` thật, nên sẽ xanh giả trên đúng lỗi P1) xoá hàng CUỐI CÙNG, khẳng định `document.activeElement === panel.element` SAU đó — khẳng định DƯƠNG đầu tiên về `panel` trong tệp.
   - **⑤f** — mệnh đề "không cướp tiêu điểm khỏi ô đang thao tác": focus THẬT vào ô tìm (`.focus()` trực tiếp trên phần tử DOM, không qua `.trigger()`), đổi bộ lọc sao cho danh sách VẪN còn hàng, khẳng định `document.activeElement` KHÔNG đổi.
   - Mục (c) trong yêu cầu ("đối chứng cho P1: gỡ vệ đối xứng ⇒ ca (a) đỏ") KHÔNG phải một `it()` thứ ba — nó là phép GỠ-CHỖ-NỐI để nghiệm thu chính ca ⑤e, đã chạy (xem dưới). Ba mục (a)/(b)/(c) của yêu cầu ánh xạ vào **hai** `it()` mới + phép gỡ cho MỖI ca.
3. **P3 — `src-tauri/tests/config_invariants.rs`: ca mới `the_guarded_dict_layers_surface_literal_names_the_wire_it_actually_sits_in`.** Khuôn đọc-nguồn tìm chữ ký MỖI vỏ `wire::` bằng marker, cắt thân hàm tới `#[tauri::command` kế tiếp, khẳng định literal `surface` ĐÚNG (`"marks_for_chapter"`/`"pending_candidates"`) nằm TRONG thân hàm tương ứng — không chỉ "có mặt đâu đó trong tệp" (phép `contains` toàn văn bản sẽ xanh dù hai literal đã hoán chỗ).
4. **P4 — cùng tệp: `code_only` nay cắt chú thích CUỐI DÒNG.** Thêm `.map(|l| l.find("//").map_or(l, |idx| &l[..idx]))` vào chuỗi transform trước `join(" ")`. Đo trước khi sửa: `grep '"[^"]*//[^"]*"' commands/glossary.rs` cho 0 kết quả (không chuỗi mã nào mang `//` trong dấu nháy trong tệp này) ⇒ phép cắt thô an toàn cho tệp này; giới hạn ghi thẳng trong doc-comment.
5. **P5 — con số "6 chỗ / 5 tệp" của khuôn "nét dẫn" SAI, đúng là 7 chỗ / 5 tệp.** `panels/LookupPanel.vue:1154 .lookup-row` (`padding: 2px 0 2px 11px; border-left: 2px solid transparent;`) là chỗ thứ bảy, viết `padding` dạng rút gọn nên lọt qua mọi lượt grep `padding-left: 11px` trước đó (cả cụm F ⑥ lẫn vòng rà 1). Sửa ở CẢ HAI nơi đã chép con số: chú thích `.gs-alert` (`GlossarySettingsOverlay.vue`) và `deferred-work.md` (cả mục `→` của Cụm F lẫn mục nợ "16 tệp px thô"). Quyết định `* 2.75` KHÔNG đổi. Ghi cách đo ĐÚNG (cặp `border-left: 2px` + lề `11px`, kể cả shorthand) vào cả hai chỗ để không lặp lại lỗi grep.
6. **P6 — `deferred-work.md`: "~105 họ" sai một đơn vị, đúng là ~104** (110 họ đo được − 6 cặp đã có mã [5 mới + `蕭` sẵn có, sáu vế giản `萧陈张刘杨黄` là sáu họ phân biệt] = 104). Sửa cả hai chỗ trong `deferred-work.md` (tiêu đề mục nợ + câu trong `evidence`), và tiện thể sửa luôn dòng tương ứng trong chính §Completion Notes — Vòng rà 1 của spec này (không thuộc phạm vi P6 gốc nhưng cùng lỗi, cùng tệp sở hữu).
7. **P7 — `scripts/check-debt-owner.mjs`: căn cứ "577 mục" trên `ITEM_FLOOR` đã lệch.** Đo LẠI ngay trước khi ghi (sau khi chính lượt vá P1–P8 nối thêm một mục nợ nữa — xem mục 9 dưới): số thật **580**. Sàn 490 vẫn đúng dải (490/580 = 84,5%) — chỉ sửa con số căn cứ trong chú thích, KHÔNG đổi `ITEM_FLOOR`.
8. **P8 — `deferred-work.md`, mục nợ "16 tệp `.vue` còn lại mang px thô" nay trỏ chéo sang khuôn "nét dẫn".** Ghi rõ BỐN trong 16 tệp (`ShortcutsOverlay` · `AttributionOverlay` · `panels/LookupPanel` · `panels/LookupRecord`) mang khuôn đó, và ràng buộc "không đổi 11px mà không đổi cả bảy chỗ" sống ở đâu (chú thích `.gs-alert` của `GlossarySettingsOverlay.vue`) — để người thi hành mục nợ đọc trước khi chạm CSS diện rộng.
9. **Ghi nợ, không vá — mục nợ mới trong `deferred-work.md`:** điều kiện `list.value !== null` trong `focusInitialTarget()` (tự bắt ở vòng rà 1 bằng `console.log`) không có phép kiểm hồi quy XÁC ĐỊNH — ca ⑤d nương vào một khoảng hở giữa hai lượt flush của Vue mà mock IPC gần-như-tức-thời của vitest TÌNH CỜ tái hiện được, không phải một cơ chế được tạo ra có chủ đích. Chủ: cùng chủ với món e2e Glossary.

**Bác, không làm:** "một ca đối chứng chạy cả hai adapter `bool → DictionaryProbe` rồi so kết quả" — hai adapter sống ở hai crate test khác nhau (`src/` đơn vị so với `tests/` tích hợp), không crate nào thấy crate kia, và NFR15 cấm thêm một crate hỗ trợ chỉ để hợp nhất. Giới hạn đã tự khai tại chỗ ở cả hai bản chép (`glossary_scan_contract.rs` và `commands/project.rs::tests`) từ vòng rà 1 — không cần thêm gì.

**Ba phép GỠ-CHỖ-NỐI mới, số ca đỏ THẬT:**

1. **Gỡ vệ đối xứng của P1** (xoá nguyên watcher N→0 mới thêm) → `npx vitest run tests/frontend/glossaryManage` cho **1 ca đỏ**: `⑤e (P1) tiêu điểm dời VỀ panel khi danh sách cạn hẳn…`. 38 ca còn lại (gồm ⑤a–d và ⑤f) vẫn xanh. (Đây cũng chính là phép gỡ mà yêu cầu P2(c) mô tả — "đối chứng cho P1" và "ca (a)" là MỘT, không hai phép riêng.)
2. **Gỡ dòng `if (document.activeElement !== panel.value) return`** khỏi watcher 0→N gốc → cùng lệnh cho **1 ca đỏ**: `⑤f (P2b) đổi bộ lọc trong khi danh sách vẫn còn hàng KHÔNG cướp tiêu điểm khỏi ô tìm đang gõ`. 38 ca còn lại (gồm ⑤a–e) vẫn xanh — đúng dự đoán của P2(b): "hôm nay gỡ nó thì cả 4 ca ⑤ [cũ] vẫn xanh".
3. **Hoán hai literal `surface`** (`"marks_for_chapter"` ↔ `"pending_candidates"`, đúng vị trí câu lệnh, không đổi số lần gọi) → `cargo test --locked --test config_invariants` cho **1 ca đỏ**: `the_guarded_dict_layers_surface_literal_names_the_wire_it_actually_sits_in`, nêu đích danh `glossary_pending_candidates` mang sai literal. `commands_glossary_uses_the_shared_guarded_dict_layers_helper_not_a_bare_unwrap_or_empty` (cổng `call_count`/cấu trúc của vòng rà 1) VẪN XANH trên đúng thí nghiệm này — xác nhận đúng mô tả P3: cổng cũ không canh được swap.

Sau mỗi lượt, tệp bị sửa được khôi phục lại nguyên trạng (`cp`/tái tạo từ bản sao lưu trước khi gỡ, xác nhận `diff` giống hệt) và cổng liên quan chạy lại xanh trước khi sang lượt kế tiếp.

**Verification cuối cùng của vòng rà 2 (đo lại sau các lượt gỡ-chỗ-nối, toàn bộ xanh):**
- `cargo test --locked` (toàn bộ `src-tauri`): 28 tệp/khối, **0 đỏ** (`config_invariants` nay 24 ca, tăng từ 23 — thêm ca P3).
- `npx vitest run` (toàn bộ `tests/frontend`): 43 tệp / **567 ca**, **0 đỏ** (tăng từ 565 — thêm ⑤e/⑤f).
  ⚠️ **Một lượt chạy XEN với `cargo test` đang biên dịch ở nền cho 2 tệp/2 ca đỏ ngẫu nhiên** — đúng lớp lỗi ĐÃ GHI NỢ trong `deferred-work.md` (Cụm E: *"Bộ vitest của Glossary ĐỎ NGẪU NHIÊN 5–8 ca khi máy đang tải nặng… tranh CPU của bộ chạy song song theo TỆP, không phải một phụ thuộc thật"*). Chạy LẠI `npx vitest run` một mình (không `cargo test` xen vào) ngay sau đó: **43/43, 567/567, 0 đỏ** — xác nhận đây là chính xác chập chờn đã biết, không phải một hồi quy mới.
- `npm run check:lint && npm run check:tokens && npm run check:i18n && npm run check:debt-owner`: exit 0 cho cả bốn (`check:debt-owner` nay 580 mục tổng, 0 mục mở thiếu `Chủ:`).
- `npm run build`: thành công.
- `npm run check:gates`: ba danh sách cổng khớp nhau.
- `git status --short src/`: vẫn đúng **hai** tệp — `GlossaryManageOverlay.vue`, `GlossarySettingsOverlay.vue`.

**Còn lại / rủi ro cần Ice biết (vòng rà 2):**
- Mục nợ mới (P9 ở trên) tự khai giới hạn nghiệm thu thật của ca ⑤d — không phải một khoảng hở MỚI phát sinh từ vòng rà 2, mà một khoảng hở đã có từ vòng rà 1 nay được đặt tên đúng và ghi có chủ thay vì để ngầm.
- Ba phép GỠ mới không tạo crate/phụ thuộc nào, không đổi hành vi sản phẩm ngoài phạm vi P1 (vệ đối xứng tiêu điểm) — P3/P4 chỉ chạm cổng test, P5–P8 chỉ chạm tài liệu/chú thích.

## Suggested Review Order

**Điểm vào — cụm F bác ba phát hiện của chính nó**

- Đọc đây trước: tám mục vá, ba mục bị đo bác, và vì sao "mười bảy" là mười bốn.
  [`deferred-work.md:7391`](./deferred-work.md#L7391)

**⑦ + P1 — vá một lỗ a11y rồi tự mở một lỗ bàn phím, và đóng lại**

- Vòng 1 dời tiêu điểm sang `<ul>` để `aria-activedescendant` có nghĩa thật.
  [`GlossaryManageOverlay.vue:140`](../../src/GlossaryManageOverlay.vue#L140)

- 🔴 P1: vệ đối xứng N→0 — `<ul>` bị `v-if` gỡ thì tiêu điểm phải về `panel`, không rơi ra `body`.
  [`GlossaryManageOverlay.vue:167`](../../src/GlossaryManageOverlay.vue#L167)

- `<ul>` nay có `tabindex="-1"` và giữ tiêu điểm — nửa cơ chế mà bản đầu thiếu.
  [`GlossaryManageOverlay.vue:456`](../../src/GlossaryManageOverlay.vue#L456)

- Một hàm dựng `id` dùng chung cho `<li>` và `<ul>`, không chép công thức hai chỗ.
  [`GlossaryManageOverlay.vue:220`](../../src/GlossaryManageOverlay.vue#L220)

- Rơi về `panel` khi chưa có hàng; điều kiện `list.value !== null` là chỗ đã hụt một lần.
  [`GlossaryManageOverlay.vue:95`](../../src/GlossaryManageOverlay.vue#L95)

**① `panic = "abort"` — hai điểm nổ, sổ nợ chỉ nêu một**

- Hàm thuần thay khối `map_err`: danh sách rỗng rơi về `MessageKey::Unknown`, không panic.
  [`glossary.rs:749`](../../src-tauri/src/commands/glossary.rs#L749)

**④ — gộp "chưa quản lý" với "rỗng" là anti-pattern `project.rs` gọi tên hai ngày trước**

- Combinator, không nhánh viết tay: vỏ `wire::` không gọi được từ `tests/**`.
  [`glossary.rs:1081`](../../src-tauri/src/commands/glossary.rs#L1081)

- Helper dùng chung, mở phạm vi + thêm tham số nêu đúng bề mặt gọi.
  [`project.rs:492`](../../src-tauri/src/commands/project.rs#L492)

**⑤ — bảng họ phồn thể, và cái bẫy `於`**

- Năm cặp Ice chốt; doc-comment chở phép đo 134/110/272 và vì sao KHÔNG nhập trọn.
  [`surnames.rs:102`](../../src-tauri/src/core/glossary/surnames.rs#L102)

**⑥ — px thô về token mà không đổi một pixel nào**

- Bốn khai báo, bốn bội số giữ nguyên giá trị cũ; `×2.75` có lý do viết ra.
  [`GlossarySettingsOverlay.vue:235`](../../src/GlossarySettingsOverlay.vue#L235)

**Cổng — ba cổng cấu trúc mới, và một sàn đã hết nghĩa**

- Xoá một API không giữ nó khỏi được dựng lại.
  [`glossary_boundary.rs:743`](../../src-tauri/tests/glossary_boundary.rs#L743)

- So CẤU TRÚC, không so chuỗi literal — một tên biến khác đã đủ để lách bản đầu.
  [`config_invariants.rs:1094`](../../src-tauri/tests/config_invariants.rs#L1094)

- P3: hoán hai literal `surface` thì cổng kia vẫn xanh; cổng này so theo CẶP.
  [`config_invariants.rs:1173`](../../src-tauri/tests/config_invariants.rs#L1173)

- Sàn 443/577 = 76,8 % đã tụt khỏi dải chính nó đặt; nâng 490, làm tròn XUỐNG.
  [`check-debt-owner.mjs:514`](../../scripts/check-debt-owner.mjs#L514)

**Ngoại vi — các ca test, xếp theo thứ tự chúng bị vòng rà bắt thiếu**

- ⑤a: giá trị thuộc tính đúng — nhưng CHỈ giá trị, đó là điều vòng rà 1 bắt.
  [`glossaryManage.test.ts:953`](../../tests/frontend/glossaryManage.test.ts#L953)

- ⑤d: phần tử mang thuộc tính phải CHÍNH LÀ `document.activeElement`.
  [`glossaryManage.test.ts:1034`](../../tests/frontend/glossaryManage.test.ts#L1034)

- ⑤e: xoá hàng cuối bằng lời gọi THẬT, không `.trigger()` — `.trigger()` cho xanh giả ở đây.
  [`glossaryManage.test.ts:1087`](../../tests/frontend/glossaryManage.test.ts#L1087)

- ⑤f: đổi bộ lọc không được cướp tiêu điểm khỏi ô tìm đang gõ.
  [`glossaryManage.test.ts:1132`](../../tests/frontend/glossaryManage.test.ts#L1132)

- ④a: `陳` nới ngưỡng đúng như vế giản `陈`.
  [`glossary_scan_contract.rs:383`](../../src-tauri/tests/glossary_scan_contract.rs#L383)

- ④b: `鬍` KHÔNG được nới — bảng chỉ nới cho họ.
  [`glossary_scan_contract.rs:402`](../../src-tauri/tests/glossary_scan_contract.rs#L402)

- ④c: ca quần thể bắt `於` nếu ai đó dán nguyên bảng đo vào.
  [`glossary_scan_contract.rs:433`](../../src-tauri/tests/glossary_scan_contract.rs#L433)

- ①b: `Err(vec![])` không lái được qua `parse()` thật, nên hàm thuần là đường duy nhất.
  [`glossary_import_dialog_contract.rs:732`](../../src-tauri/tests/glossary_import_dialog_contract.rs#L732)
