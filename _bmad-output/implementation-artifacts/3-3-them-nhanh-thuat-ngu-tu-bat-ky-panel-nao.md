---
title: 'Story 3.3 — Thêm nhanh thuật ngữ từ bất kỳ panel nào'
type: 'feature'
created: '2026-08-20'
status: 'done'
baseline_commit: '07766f01df4152144e742b33c2427d6ac9d9e404'
review_loop_iteration: 0
context:
  - '{project-root}/_bmad-output/implementation-artifacts/epic-3-context.md'
  - '{project-root}/_bmad-output/implementation-artifacts/3-2-bang-cho-ung-vien-tach-han-khoi-glossary.md'
  - '{project-root}/AGENTS.md'
  - '{project-root}/src-tauri/AGENTS.md'
  - '{project-root}/src/AGENTS.md'
---

<frozen-after-approval reason="human-owned intent — do not modify unless human renegotiates">

## Intent

**Problem:** Sau 3.1 và 3.2, Glossary có mô hình dữ liệu và bảng chờ nhưng **không một đường sản phẩm nào**: `lib.rs:302-364` chưa đăng ký một `#[tauri::command]` nào của `core/glossary/**`, và `OpenWork.scope` vẫn không được đọc ở đâu trong `src-tauri/src/**` ngoài test (`deferred-work.md:603`). Người dịch không có cách nào đưa một thuật ngữ vào Glossary — FR48 là năng lực đầu tiên mở khoá cả Epic.

**Approach:** Dựng **dải "Thêm thuật ngữ" ở chân workspace** (Ice chốt 2026-08-20, ngay trên `StatusBar` — `App.vue:260`) cộng bề mặt IPC hai lớp đầu tiên của `glossary/`. Lệnh đọc vùng chọn bằng **đường riêng**, không qua `currentSelectionText()`. Trước khi ghi, một lượt **tra hai tầng qua `ScopeResolver`** quyết định dải ở chế độ THÊM hay SỬA — đó là vế cấu trúc của AC *"không tạo mục trùng"*.

## Boundaries & Constraints

**Always:**
- Vùng chọn đọc bằng đường riêng của lệnh. `currentSelectionText()` lọc `role === 'source'` (`selectionContract.ts:208`) nên trả **rỗng ở ba trong bốn** bề mặt FR48. 🔴 Không lật vai `display` → `source` (Kiểm F ③ đỏ), không gỡ `useSelectionSurface` (`SELECTION_SURFACE_FLOOR = 6` đỏ) — `deferred-work.md:2687-2697` gọi đây là *điều kiện khởi hành* của chính story này.
- 🔴 **Ba tên trong `GLOSSARY_ONLY_SURFACE` ở lại bị cấm ngoài `core/glossary/**`.** Bề mặt IPC gọi **hàm phơi ra MỚI**, không gọi `insert_manual_entry`/`confirm_translation`/`load_tier`. Tiền lệ có chữ ký: Story 3.1 gặp đúng vòng luẩn quẩn này và **Ice ký sửa CHỮ KÝ thay vì nới cổng** (`glossary_boundary.rs:80-88`).
- Khuôn hai lớp: hàm thuần nhận `Option<&Store>`; `mod wire` mỏng dùng **`try_state`**, không `state()`.
- `note` **cắt khoảng trắng biên** như `source_term`/`translation` (Ice ký 2026-08-20) — đóng `deferred-work.md:5380-5385`.
- Mọi lượt ghi không khớp hàng phải **trả lỗi**, không `Ok(())` 0 hàng.
- Tầng Tác phẩm không mở được thì lựa chọn đó **hiện kèm LÝ DO**, không biến mất — rỗng có lý do, không rỗng im lặng.
- Adapter `src/config/glossary.ts` **không bao giờ ném**, hình dạng ba trạng thái + type guard lúc chạy.
- Lệnh đăng ký ở `src/commands/index.ts` qua một field `CommandDeps` mới, nối thật ở `main.ts`. `@click` đúng một `dispatch('glossary.add_term')`.
- Dải **đẩy nội dung lên, không phủ**: không `position: fixed`, không `z-index`, không `box-shadow`. Màu và cỡ chữ chỉ qua `var(--…)`.
- Chuỗi literal trong `src-tauri/src/**` viết **không dấu**; mọi chữ hiển thị là khoá `vi.json`.

**Ask First:**
- Bất kỳ đề xuất nào cho lệnh này ghi vào `glossary_candidate` (đóng một ứng viên trùng khi người dùng thêm tay) — lỗ đó có chủ Story 3.5.
- Đổi **tầng** của một mục đã có (chuyển kho `global.db` ↔ `project.db`) — chủ Story 3.9.
- Nếu phải **hạ** một sàn quần thể thay vì nâng.
- Nếu việc đóng lỗ 0-hàng của `confirm_translation` làm đỏ ngoài `glossary_contract.rs`.

**Never:**
- Đánh dấu thuật ngữ ở cột nguyên văn (3.4) · dải mọc theo câu và cơ chế xếp thứ tự (3.6) · đề xuất Hán Việt (3.7) · duyệt hàng loạt (3.8) · tìm/lọc/xoá/đẩy tầng (3.9) · CSV (3.10).
- Gọi `insert_candidate`/`pending_candidates`/`approve_candidate`/`reject_candidate`. Story này **không chạm** `candidate_store`.
- Thêm quyền vào `capabilities/main.json` — ACL chỉ canh command của *plugin* (`lib.rs:295-298`).
- Sửa `epics.md`/`prd.md` cho khớp mã.

## I/O & Edge-Case Matrix

| Scenario | Input / State | Expected Output / Behavior | Error Handling |
|---|---|---|---|
| Thêm từ cột nguyên văn | bôi đen `慕容`, `glossary.add_term` | Dải mở, nguồn điền sẵn `慕容`, chế độ THÊM | — |
| Thêm từ Panel Lookup / AI Translation / cột bản dịch | cùng lệnh, vai `display` | Dải mở với đúng cụm — vai `display` **không** chặn | — |
| Không có vùng chọn | không bôi đen gì | Dải mở, ô nguồn **rỗng và nhận focus** — không phải một phím chết | — |
| Lưu ở tầng Global | nguồn + bản dịch + phân loại, tầng Global | Mục vào `global.db`, `term_origin='manual'`, **đã chốt** | — |
| Lưu ở tầng Tác phẩm | có Tác phẩm đang mở | Mục vào `project.db` của Tác phẩm đó | — |
| Chọn tầng Tác phẩm khi chưa mở Tác phẩm | `OpenWorkState` là `None` | Lựa chọn hiện **kèm lý do**, lưu bị từ chối | `err.glossary.work_tier_unavailable` |
| Cụm đã có ở tầng Global | `慕容` có trong `global.db` | Dải mở chế độ **SỬA**, tầng **ghim** vào Global, các ô mang giá trị cũ | — |
| Cụm có ở CẢ hai tầng | cùng `source_term` hai kho | SỬA mục **tầng Tác phẩm** (AD-18 ghi đè), nhãn tầng nói rõ | — |
| Cụm đã có nhưng **chờ chốt** | `translation IS NULL` | Vẫn mở SỬA — lượt tra **không lọc** `is_confirmed` | — |
| Sửa một `id` đã biến mất | mục bị xoá giữa chừng | Không ghi gì, báo lỗi đọc được | `err.glossary.entry_missing` |
| Ghi chú toàn khoảng trắng | `note = "   "` | Ghi xuống chuỗi rỗng — một cách biểu diễn duy nhất | — |
| Nguồn hoặc bản dịch toàn khoảng trắng | `"\u{3000}"` | Từ chối | `CHECK` ⇒ `StoreError::WriteFailed` |
| Đóng dải bằng `Esc` hoặc sau khi lưu | dải đang mở | Màn hình không đổi, **focus và vùng chọn cũ được trả lại** | — |

</frozen-after-approval>

## Code Map

- `src-tauri/src/core/glossary/store.rs` — `insert_entry_row` `:78` (helper riêng tư dùng chung, **gọi lại cho đường sửa**) · `insert_manual_entry` `:111` · `confirm_translation` `:186` (🔴 `UPDATE` 0 hàng vẫn `Ok(())`; doc-comment `:177-185` ghi thẳng *"Chủ: Story 3.3"*) · `load_tier` `:204` · `decode_category` `:247` / `decode_term_origin` `:260` (riêng tư, khuôn để chép) · `GlossaryError` `:284` + `Display` `:293` + `From<StoreError>` `:305` + `From<ScopeError>` `:311` · `entries_eligible_for_injection` `:335` (gọi `resolver.apply_override` `:344`) · `GLOSSARY_SCOPE_KIND` `:57` (riêng tư). `note` **không** được trim ở `:111`, khác hai cột kia.
- `src-tauri/src/core/glossary/entry.rs` — `Category` `:20-29`/`as_str` `:34`/`from_wire` `:49` · `TermOrigin` `:71-78` · `GlossaryEntry` `:112-128` · `is_confirmed` `:132`. Khuôn enum để chép cho `GlossaryTier`.
- `src-tauri/src/core/glossary/mod.rs` — `pub mod` `:33-34` · re-export `:36-39` · §GIỚI HẠN THẬT `:47-66`.
- `src-tauri/src/core/scope/mod.rs` — `ScopeError` `:132-160` (ba biến thể, **cả ba là lỗi lập trình**) · `ScopeResolver::with_work` `:253` · `has_work_tier`. ⚠️ `:114` viết `ScopeError` *"phải đi qua `impl From<ScopeError> for IpcError`"* — **impl đó không tồn tại**; sửa mệnh đề tại chỗ kèm 🔵 + ngày. `kinds.rs:162` — `Glossary => "glossary" : Override`.
- `src-tauri/src/commands/pinned.rs` — **khuôn hai lớp để chép**: hàm thuần `:110` (`store.ok_or_else(store_is_missing)?` `:111`), `mod wire` `:219-225` (`try_state` + `.as_deref()`), lý do không dùng `state()` `:216-218`.
- `src-tauri/src/commands/project.rs` — `OpenWork` `:43-52` (trường `store` + `scope`) · `OpenWorkState` `:297` · `replace_open_work` `:319-325`. **Chưa có hàm đọc `&Store` từ `OpenWorkState`** — mỗi command tự dựng; story này dựng một helper dùng chung.
- `src-tauri/src/core/i18n/mod.rs` — `macro_rules! message_keys!` `:62-91`, bảng khoá thật từ `:100`, khuôn khai `:143-151` (`Biến Thể => "khoa.cham" ["tham_so"],`) · `IpcError` `:326` (bốn trường riêng tư) · `IpcError::new` `:369-374`. **Chưa có khoá `glossary.*` nào.**
- `src-tauri/src/core/store/mod.rs` — `impl From<StoreError> for IpcError` `:483-506` (khuôn để chép) · `Store::write`/`read` · `StoreSpec::project` `:288`/`global` `:274`.
- `src-tauri/src/lib.rs` — `tauri::generate_handler!` `:302-364`, ví dụ dòng đăng ký `:319-321`. ⚠️ **Không test nào bắt lỗi quên đăng ký handler.** `:295-298` — command của chính app không cần mục ACL.
- `src/panels/selectionContract.ts` — `registerSelectionSurface` `:88-104` · `useSelectionSurface` `:120-138` · vai `:52` · `surfaceFor()` hỏi `display` **trước** `source` `:180-189` · `currentSelectionText()` `:201-212` (🔴 lọc `role === 'source'` ở `:208` — **không dùng**) · `focusSelectionSource()` `:362-383`.
- Bốn bề mặt đã đăng ký, không đụng: `GridPanel.vue:415` (`colSrc`, `source`) · `:416` (`colTgt`, `display`) · `LookupPanel.vue:171` · `AiTranslationPanel.vue:45` (+ chú thích `:41-44` **nêu đích danh FR48/Story 3.3**).
- `src/commands/index.ts` — chỗ đăng ký DUY NHẤT; `CommandDeps` `:162-503`; khuôn một lệnh `:760-773`; `FOCUS_OWNERS` `:66-73` (**sáu mục — dải KHÔNG thêm vào đây**, nó không phải panel). `registry.ts:125` `COMMAND_ID_RE`. Hợp âm còn trống: **`Mod+Alt+G`** (đo 2026-08-20, `grep` trên `index.ts` = 0).
- `src/config/pinned.ts:41-129` — khuôn adapter ba trạng thái + `isIpcError` `:46-60` + `invoke` camelCase `:7-11`. ⚠️ `project-context.md:184-187` viết *"sáu tệp"* — hết đúng khi thêm tệp thứ bảy.
- `src/App.vue:44,260` — `import StatusBar` và `<StatusBar />`; dải chèn **ngay trên** dòng `:260`. `src/StatusBar.vue` — khuôn một dải chân màn hình đã có.
- `src/i18n/index.ts` — `t` `:42`, `tError` `:72`; `vi.json` phẳng khoá chấm.
- **Sàn quần thể sẽ phải xét lại** (số THẬT đo 2026-08-20: **16** `.vue` · **39** `.ts` · **49** `.rs` dưới `src-tauri/src`): `check-commands.mjs:211` `VUE_FLOOR=13` · `:229` `TS_FLOOR=33` · `check-i18n.mjs:279` `RS_FLOOR=36` (**đã tụt còn 73 %, dưới dải từ trước story này**) · `:289` `VUE_FLOOR=13` · `check-panel-refs.mjs:517` `FILE_FLOOR=33` · `check-layout.mjs:110` `FILE_FLOOR=46` · `check-tokens.mjs:91-92` · `glossary_boundary.rs:55` `RS_FLOOR=38`.
- Cưỡng chế đọc trước khi viết: `check-commands.mjs:725-726` (`DISPATCH_ONLY_RE`), `:742-746` (ba cách viết listener bị từ chối), Kiểm E chạy `installCommands()` thật `:1599` trên cả hai nền tảng `:1620-1630`; `check-tokens.mjs:1454-1457` (Kiểm F) + `exemptAt` `:641-646`; `check-panel-refs.mjs:125` (`EXEMPT`) — ô nhớ cấp module trong `src/**/*.ts` phải có `export function reset*()`.
- `src-tauri/tests/ipc_contract.rs:228` `every_message_key_exists_in_vi_json` · `:321` khớp `params` ↔ placeholder hai chiều.
- `src-tauri/tests/glossary_boundary.rs:98` `GLOSSARY_ONLY_SURFACE` (ba tên) + luật ở `:72-97` · `:118` `NON_MANUAL_ORIGIN_TOKENS` · `:55` `RS_FLOOR`.
- `deferred-work.md` — `:603` (`OpenWork.scope` chưa có chỗ gọi sản phẩm, chủ 3.3) · `:2687-2697` (điều kiện khởi hành) · `:5348-5352` (`has_work_tier()` vs `work.is_some()` không ai bắt khớp, chủ 3.3) · `:5380-5385` (`note` trim, chủ 3.3) · `:5446-5458` (bốn hàm `candidate_store`) · `:5506-5520` (`already_decided_error`, chủ 3.3 — **tiền đề sai**).

## Tasks & Acceptance

**Execution:**
- [x] `src-tauri/src/core/glossary/entry.rs` -- thêm `GlossaryTier { Global, Work }` + `as_str`/`from_wire` theo khuôn `Category` -- `id` chỉ duy nhất TRONG một `Store` (`deferred-work.md:5352`), nên một mục trả về mà không mang nhãn tầng là một mục **không sửa lại được**.
- [x] `src-tauri/src/core/glossary/store.rs` -- `.trim()` cho `note` ở `insert_entry_row`; `confirm_translation` trả lỗi khi `UPDATE` khớp 0 hàng; `debug_assert_eq!(resolver.has_work_tier(), work.is_some(), …)` trong `entries_eligible_for_injection` -- ba lỗ này đều có chủ là story hiện tại; đóng cùng lượt với đường gọi sản phẩm đầu tiên chứ không để chúng lộ ra ngoài test.
- [x] `src-tauri/src/core/glossary/store.rs` -- thêm **ba hàm phơi ra mới**: `resolve_term_for_quick_add` (tra hai tầng qua `apply_override`, **không lọc** `is_confirmed`, trả `Option<(GlossaryTier, GlossaryEntry)>`), `add_manual_term` (chọn `&Store` theo tầng người dùng chọn, gọi xuống `insert_manual_entry`), `update_manual_term` (sửa `translation`/`note`/`category` theo `(tier, id)`, lỗi khi 0 hàng) -- ba tên trong `GLOSSARY_ONLY_SURFACE` ở lại bị cấm ngoài module; đây là đường Ice đã ký ở `glossary_boundary.rs:80-88` khi gặp đúng vòng luẩn quẩn này.
- [x] `src-tauri/src/core/glossary/store.rs` -- `impl From<GlossaryError> for IpcError` theo khuôn `store/mod.rs:483-506`; nhánh `Scope` mang `code` ổn định và **không tham số** -- `Display` của lỗi là câu chẩn đoán, mà tham số i18n phải mang DỮ LIỆU chứ không mang CÂU.
- [x] `src-tauri/src/core/scope/mod.rs` -- sửa mệnh đề `:114` kèm 🔵 + ngày: cầu nối nay đi qua `From<GlossaryError>`, `From<ScopeError>` đứng riêng vẫn **chưa tồn tại** -- một mệnh đề mô tả một impl không có là đúng lớp nợ mà luật "sửa tại chỗ" sinh ra để chống.
- [x] `src-tauri/src/core/i18n/mod.rs` -- thêm `GlossaryEntryMissing => "err.glossary.entry_missing" []` · `GlossaryWorkTierUnavailable => "err.glossary.work_tier_unavailable" []` · một khoá cho nhánh `Scope` -- danh mục `message_key` là ĐÓNG và `ALL` sinh từ chính macro, nên đừng viết một danh sách song song.
- [x] `src-tauri/src/commands/glossary.rs` -- mới, khuôn hai lớp: ba hàm thuần (`glossary_lookup_term`, `glossary_add_term`, `glossary_update_term`) nhận `Option<&Store>` cho `global` cộng một helper đọc `(&Store, &ScopeResolver)` từ `OpenWorkState`; `mod wire` mỏng dùng `try_state` -- helper đọc `OpenWorkState` là chỗ **đầu tiên** `OpenWork.scope` được đọc trong mã sản phẩm, đóng `deferred-work.md:603`.
- [x] `src-tauri/src/lib.rs` -- đăng ký ba command vào `generate_handler!` -- không test nào bắt lỗi quên bước này, nên nó là bước dễ mất nhất trong cả story.
- [x] `src/config/glossary.ts` -- adapter thứ **bảy**, khuôn ba trạng thái + `isIpcError`, tham số `invoke` camelCase, trường trả về giữ `snake_case` -- hai chiều đặt tên khác nhau là chỗ dễ sai nhất trên dây.
- [x] `src/glossaryQuickAddState.ts` -- state của dải: cụm nguồn, chế độ (THÊM/SỬA), tầng, phân loại, phần tử focus cũ + `Range` cũ, vị từ `…HasLoaded` cho lượt tra; kèm `export function resetGlossaryQuickAdd()` -- `check:panel-refs` đỏ với ô nhớ cấp module không có đường `reset*`, và vị từ `…HasLoaded` là thứ chặn dải khẳng định *"chưa có mục nào"* trong lúc còn đang chờ IPC.
- [x] `src/GlossaryQuickAdd.vue` -- dải ở chân workspace, chèn ngay trên `<StatusBar />` (`App.vue:260`); bốn trường + chọn tầng + bốn phân loại theo phím số; `↵` lưu, `Esc` huỷ, đóng thì trả lại focus và vùng chọn cũ -- không `position: fixed`, không `z-index`, không `box-shadow`; màu và cỡ chữ chỉ qua `var(--…)`.
- [x] `src/commands/index.ts` + `src/main.ts` -- lệnh `glossary.add_term`, `labelKey: 'command.glossary.add_term'`, `keys: ['Mod+Alt+G']`; field `CommandDeps` mới nhận hàm mở dải và một hàm đọc vùng chọn thô; nối thật ở `main.ts` -- `index.ts` phải nạp được bằng Node thuần nên mọi state Vue đi vào qua `CommandDeps`, không qua `import`.
- [x] `src/i18n/vi.json` -- khoá `command.glossary.add_term` + nhãn dải, bốn phân loại, hai tầng, lý do tầng Tác phẩm chưa dùng được, hai khoá lỗi Rust -- giọng vô nhân xưng, khoá phẳng có tiền tố miền, placeholder đúng dải `{ten_tham_so}`.
- [x] `scripts/check-commands.mjs` · `check-i18n.mjs` · `check-panel-refs.mjs` · `check-layout.mjs` · `check-tokens.mjs` · `src-tauri/tests/glossary_boundary.rs` -- đo lại bằng chính cổng rồi nâng tám hằng sàn về dải 80–85 %, mỗi hằng kèm số THẬT + ngày -- sàn là cận dưới nên tệp thừa không làm cổng đỏ, nó chỉ làm sàn vô nghĩa; `RS_FLOOR = 36` đã tụt còn 73 % **trước** story này.
- [x] `_bmad-output/project-context.md` -- sửa `:184-187` kèm 🔵 + ngày: **bảy** adapter, không sáu -- mệnh đề hết đúng thì sửa tại chỗ.
- [x] `src-tauri/tests/glossary_boundary.rs` -- giữ `GLOSSARY_ONLY_SURFACE` ở ba tên; thêm phép kiểm ba hàm mới **được phép** gọi từ `commands/glossary.rs` và một đối chứng dương rằng cổng vẫn đỏ nếu ai gọi `load_tier` từ đó -- một cổng chưa bao giờ đỏ là một cổng chưa ai biết nó có chạy không.
- [x] `src-tauri/tests/glossary_contract.rs` -- mọi hàng của I/O Matrix chạm Rust, tên hàm là một CÂU khẳng định; tái dùng `every_blank_form()` `:296` -- ca *"cụm có ở CẢ hai tầng"* và ca *"cụm đang chờ chốt vẫn mở SỬA"* là hai ca dễ cài ngược nhất.
- [x] `tests/frontend/glossaryQuickAdd.test.ts` -- chế độ dải là **hàm thuần** của `(source_term, kết quả tra)`; ô nguồn rỗng khi không có vùng chọn; `Esc` trả lại phần tử focus đã lưu; ba trạng thái của vị từ `…HasLoaded` -- vế **thị giác** và vế **vùng chọn trên engine thật** thuộc bàn đo tay, không thuộc `happy-dom`.
- [x] `_bmad-output/implementation-artifacts/deferred-work.md` -- nối 🔵 vào `:5506-5520` chuyển chủ sang **Story 3.8** kèm lý do *(story 3.3 không gọi `approve_candidate`/`reject_candidate` một lần nào — tiền đề "bề mặt IPC đầu tiên chạm `candidate_store`" đo lại là sai)*; đóng `:5380-5385` và `:603`; mở bốn mục mới MỖI mục một chủ: `From<ScopeError> for IpcError` đứng riêng (**Chủ: Epic 7**) · `debug_assert_eq!` không bắn ở release (**Chủ: Story 3.9**) · đổi tầng một mục đã có (**Chủ: Story 3.9**) · ứng viên trùng một mục vừa thêm tay nằm lại bảng chờ (**Chủ: Story 3.5**) -- `check:debt-owner` đỏ với mục mồ côi, và không bao giờ xoá một mục đã đóng.

**Acceptance Criteria:**
- Given một `grep` trên `src-tauri/src/**`, when tìm `insert_manual_entry`/`confirm_translation`/`load_tier`, then không tệp nào ngoài `core/glossary/**` gõ ba tên đó — kể cả `commands/glossary.rs` vừa dựng.
- Given `commands/glossary.rs`, when đọc, then `OpenWork.scope` được đọc trong mã **sản phẩm** lần đầu tiên, và mọi lời gọi `try_state`, không `state()`.
- Given người dùng đổi ô nguồn sau khi dải đã mở, when giá trị mới trùng một mục đã có, then dải **tự chuyển sang SỬA** và tầng bị ghim theo mục tìm được — chế độ là hàm của giá trị hiện tại, không phải của lượt mở.
- Given dải đang mở, when người dùng chỉ dùng bàn phím, then mọi trường, cả hai tầng, cả bốn phân loại, lưu và huỷ đều tới được — không thao tác nào của FR48 đòi chuột.
- Given `.githooks/pre-push`, when chạy, then mười một cổng + vitest + build + `cargo test --locked` đều xanh; `check:gates` không đổi vì story này **không thêm cổng**.
- Given lượt CI sau khi push, when đọc, then cả nửa macOS lẫn nửa Windows xanh — `pre-push` chỉ chạy trên macOS của Ice.

## Spec Change Log

### 2026-08-20 — thực thi

**Quyết định thực thi không tường minh trong spec, ghi lại để lượt sau đọc được lý do:**

- **`glossary_lookup_term` trả một PHONG BÌ (`QuickAddLookup { work_tier_available,
  entry }`), không một `Option<QuickAddTerm>` trần.** I/O Matrix đòi *"Chọn tầng Tác phẩm
  khi chưa mở Tác phẩm ⇒ Lựa chọn hiện KÈM LÝ DO, không biến mất"* — hiện lý do đó TRƯỚC
  khi người dùng bấm Lưu (không chỉ sau một lượt lưu trượt) đòi webview biết "có Tác phẩm
  đang mở không" ngay trong đúng lượt đã đọc `OpenWorkState`, để tránh một vòng IPC thứ hai
  chỉ để hỏi câu đó.
- **`GlossaryTier` là kiểu RIÊNG của `core::glossary`, không tái dùng `core::scope::Tier`**
  dù hai kiểu có hai biến thể y hệt — nó là dữ liệu TRÊN DÂY của module Glossary, và dùng
  chung với `core::scope::Tier` sẽ khiến dây IPC của Glossary phụ thuộc vào một quyết định
  biểu diễn nội bộ của `core::scope`.
- **`GlossaryError` có thêm hai biến thể không mang dữ liệu — `EntryMissing`,
  `WorkTierUnavailable`** — để ba hàm mới (`resolve_term_for_quick_add`/`add_manual_term`/
  `update_manual_term`) có chỗ trả hai lỗi mà I/O Matrix đòi, cộng `impl From<GlossaryError>
  for IpcError` (khuôn chép từ `store/mod.rs::impl From<StoreError>`).
- **Ba command đăng ký, không một.** Spec chỉ nêu đích danh `glossary.add_term`, nhưng Kiểm
  A của `check:commands` đòi mọi `@click` là đúng một `dispatch('<id>')` — nút Lưu/Huỷ của
  dải cần hai command nữa (`glossary.save_term`, `glossary.close_quick_add`), cả hai giữ 0
  hợp âm mặc định và xử lý bằng handler CỤC BỘ (`@submit`/`@keydown.esc`), đúng khuôn
  `attribution.close`/`shortcuts.close`/`history.close` đã có sẵn trong kho.
- **Lượt tra LUÔN gọi IPC, kể cả với `source_term` rỗng** — một đường tắt "chuỗi rỗng thì
  khỏi tra" sẽ làm `work_tier_available` không bao giờ được biết cho tới ký tự đầu tiên,
  tức dải mở với ô nguồn rỗng hiện SAI lý do "chưa mở Tác phẩm" ngay cả khi một Tác phẩm
  đang mở thật. Rust xử lý chuỗi rỗng vô hại (`trim()` rồi tra một khoá rỗng, không khớp).
- **Tám hằng sàn quần thể đo lại và nâng** (`check-commands.mjs` × 4, `check-i18n.mjs` × 2,
  `check-panel-refs.mjs`, `check-layout.mjs`, `check-tokens.mjs` × 2 — nhiều hơn tám vì một
  số cổng có nhiều hơn một hằng sàn `src/**`) về dải 80–85%, mỗi hằng kèm số THẬT đo bằng
  cách chạy chính cổng, không ước lượng.

## Design Notes

**Chế độ của dải là một hàm thuần**, không phải một cờ đặt lúc mở:

```
mode(source_term, lookup) =
  lookup == null                  -> chưa biết  (đang chờ IPC — KHÔNG nói "chưa có")
  lookup == Found(tier, entry)    -> SỬA,  tầng GHIM = tier
  lookup == NotFound              -> THÊM, tầng tự chọn
```

⚠️ **`Vec<GlossaryEntry>` không đủ để sửa lại một mục.** `id` chỉ duy nhất **trong một `Store`**, nên hàm tra phải trả **cặp `(tier, id)`**; một `id` trần đi qua dây là một lượt `UPDATE` vào nhầm kho, im lặng và không cổng nào đỏ.

🔴 **Lượt tra KHÔNG được lọc `is_confirmed`.** `entries_eligible_for_injection` lọc — đó là lý do nó không dùng lại được ở đây: một mục *chờ chốt* bị lọc mất sẽ làm dải mở ở chế độ THÊM và `UNIQUE` chặn lượt lưu, tức người dùng thấy *"không thêm được"* mà không ai nói vì sao.

**Vì sao dải ở chân workspace chứ không phải "dải mọc" của Story 3.6:** cơ chế *chỉ một dải mọc tại một thời điểm* cùng thứ tự nhường nhau là chủ của 3.6; dựng sớm ở đây thì 3.6 phải dựng lại. Một dải **một thể hiện** ở chân cửa sổ né hẳn va chạm đó và dùng được từ cả bốn bề mặt, kể cả Panel Lookup nơi không có "câu đang sửa" nào để mọc dưới.

## Verification

**Commands:**
- `cd src-tauri && cargo test --locked` -- expected: xanh, gồm `glossary_contract`, `glossary_boundary`, `ipc_contract`.
- `npm run build && npm run test` -- expected: xanh; thiếu `dist/` thì `cargo test` gãy ở khâu biên dịch, không ở một assert.
- `npm run check:commands` -- expected: xanh; Kiểm E thấy lệnh mới trên **cả hai** nền tảng, `Mod+Alt+G` không xung đột.
- `npm run check:i18n` · `check:tokens` · `check:panel-refs` · `check:layout` · `check:debt-owner` -- expected: xanh sau khi nâng sàn.

**Manual checks (if no CLI):**
- Bàn đo tay trên `npm run tauri dev`: bôi đen ở **cả bốn** bề mặt, mỗi bề mặt một lượt; xác nhận focus và vùng chọn quay đúng chỗ sau `Esc` — `happy-dom` **không phải** WebKit và không nói gì về vùng chọn thật.
- Xác nhận dải **đẩy** nội dung workspace lên, không phủ lên nó, ở cả hai theme.
- Đọc lượt CI trên GitHub: `pre-push` xanh trên macOS không nói gì về nửa Windows.

## Suggested Review Order

**Bất biến trung tâm — bề mặt IPC đầu tiên của `glossary/`, và ba tên bị cấm vẫn bị cấm**

- Chỗ ĐẦU TIÊN `OpenWork.scope` được đọc trong mã sản phẩm — đóng `deferred-work.md:603`.
  [`glossary.rs:54`](../../src-tauri/src/commands/glossary.rs#L54)

- Ba hàm phơi ra MỚI, không nới cổng: đường Ice đã ký ở Story 3.1.
  [`store.rs:495`](../../src-tauri/src/core/glossary/store.rs#L495)

- Ghi tay đi qua helper dùng chung — một hình dạng hàng, không hai.
  [`store.rs:79`](../../src-tauri/src/core/glossary/store.rs#L79)

- Cầu lỗi sang dây; nhánh `Scope` mang `code` ổn định, KHÔNG tham số mang câu.
  [`store.rs:378`](../../src-tauri/src/core/glossary/store.rs#L378)

**Tra hai tầng — chỗ dễ cài ngược nhất**

- Không lọc `is_confirmed`: mục chờ chốt bị lọc mất thì dải mở nhầm chế độ THÊM.
  [`store.rs:495`](../../src-tauri/src/core/glossary/store.rs#L495)

- Sửa theo cặp `(tầng, id)` — `id` trần là một lượt `UPDATE` vào nhầm kho.
  [`store.rs:577`](../../src-tauri/src/core/glossary/store.rs#L577)

**Vùng chọn — điều kiện khởi hành của story**

- Đường RIÊNG, không lọc `role`: `currentSelectionText()` rỗng ở ba trong bốn bề mặt.
  [`selectionContract.ts:232`](../../src/panels/selectionContract.ts#L232)

**Dải — chế độ là hàm thuần, và lượt ghi có chốt**

- Chế độ suy từ kết quả tra, không phải cờ đặt lúc mở.
  [`glossaryQuickAddState.ts:109`](../../src/glossaryQuickAddState.ts#L109)

- `Esc` giữa lúc ghi KHÔNG đóng dải — màn hình không được nói khác đĩa.
  [`glossaryQuickAddState.ts:248`](../../src/glossaryQuickAddState.ts#L248)

- Chốt tái nhập: lượt Lưu thứ hai bị từ chối, không phát IPC thứ hai.
  [`glossaryQuickAddState.ts:267`](../../src/glossaryQuickAddState.ts#L267)

- Nút Lưu là `submit`, nên Kiểm A của `check:commands` không canh nó — đọc kỹ chỗ này.
  [`GlossaryQuickAdd.vue:99`](../../src/GlossaryQuickAdd.vue#L99)

**Cưỡng chế — cổng biên giữ nguyên ba tên**

- Danh sách cấm KHÔNG nới thêm dù có chỗ gọi sản phẩm mới.
  [`glossary_boundary.rs:99`](../../src-tauri/tests/glossary_boundary.rs#L99)

- Cổng khẳng định bề mặt IPC gọi ba hàm mới, không gọi ba tên cấm.
  [`glossary_boundary.rs:536`](../../src-tauri/tests/glossary_boundary.rs#L536)

**Test — bốn ca đáng đọc nhất**

- Ca giết story nếu `work_context` thoái hoá: AD-18 qua chính bề mặt thật.
  [`glossary_commands_contract.rs:140`](../../src-tauri/tests/glossary_commands_contract.rs#L140)

- Hai bản chép của cùng một hợp đồng dây nay có cổng kiểm chéo.
  [`glossary_contract.rs:2305`](../../src-tauri/tests/glossary_contract.rs#L2305)

- Mục chờ chốt vẫn mở SỬA — vế mà `entries_eligible_for_injection` lọc mất.
  [`glossary_contract.rs:1955`](../../src-tauri/tests/glossary_contract.rs#L1955)

- Đối chứng hai chiều cho vai `display`; một chiều thôi không chứng minh gì.
  [`glossarySelectionContract.test.ts:68`](../../tests/frontend/glossarySelectionContract.test.ts#L68)
