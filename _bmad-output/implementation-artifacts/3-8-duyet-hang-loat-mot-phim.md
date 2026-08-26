---
title: 'Story 3.8 — Duyệt hàng loạt một phím'
type: 'feature'
created: '2026-08-24'
status: 'done'
review_loop_iteration: 0
baseline_commit: '3170ce4db5d547106fd933bebde97aa2f3c8c500'
context:
  - '{project-root}/_bmad-output/implementation-artifacts/epic-3-context.md'
  - '{project-root}/_bmad-output/implementation-artifacts/3-7-de-xuat-ban-dich-bang-am-han-viet.md'
  - '{project-root}/AGENTS.md'
  - '{project-root}/src-tauri/AGENTS.md'
  - '{project-root}/src/AGENTS.md'
---

<frozen-after-approval reason="human-owned intent — do not modify unless human renegotiates">

## Intent

**Problem:** Một lượt nhập sinh hàng trăm ứng viên và **0 component nào duyệt chúng** — `pendingGlossaryCandidates()` lẫn `approveGlossaryCandidate()` đứng sẵn ở `src/config/glossary.ts` với **0 chỗ gọi sản phẩm**, `reject_candidate` chưa có vỏ IPC, và `pending_candidates` sắp bằng `ORDER BY source_term` (đối chiếu BYTE, vô nghĩa cho chữ Hán). Bảng chờ hôm nay là dữ liệu không ai nhìn thấy.

**Approach:** Một lớp phủ modal mới duyệt bảng chờ bằng bàn phím — sắp theo tần suất giảm dần, mỗi hàng chở sẵn số lần xuất hiện · ví dụ ngữ cảnh · đề xuất Hán Việt mà Story 3.7 đã gắn vào chính hàng đó. Nhận/Bỏ là lệnh đã đăng ký; con trỏ tự tiến. Nửa Rust chỉ thêm **một** vỏ IPC (bỏ ứng viên) và **một** mệnh đề `ORDER BY`.

## Boundaries & Constraints

**Always:**
- 🔴 **0 bước di trú.** `project.db` giữ **v14**. Không cột `phân loại`, không cột `con trỏ đang duyệt` — cả hai *có thể* thuộc story này (`schema.rs:412`) và story này **cố ý không lấy**: xem §Design Notes.
- Máy chỉ ĐỀ XUẤT (AD-20/FR55): **0** lượt ghi tự động. Mỗi mục vào Glossary phải do một phím người dùng bấm.
- Hàng ứng viên **ở lại** sau khi quyết (`resolution` được đặt, hàng không bị xoá) — đó là thứ làm `UNIQUE(source_term)` chặn lượt quét sau chèn lại. Không `DELETE`.
- Vòng đời một chiều (AD-36): nhận **có** đề xuất ⇒ mục **đã chốt**; nhận **không** đề xuất ⇒ mục **chờ chốt**. Không trạng thái thứ tư. Trigger `glossary_candidate_resolution_is_one_way` là lớp cuối, không phải lớp duy nhất — đọc `resolution` trước khi ghi.
- Mọi thao tác đi qua `CommandRegistry` (AD-34). Nhận/Bỏ/Chuyển/Đóng là lệnh đã `register()`; phím trong lớp phủ `dispatch('<id>')` chứ không gọi thẳng handler — một lời gọi thẳng dựng đường thứ hai mà `check:commands` Kiểm A không nhìn thấy.
- Phím **1–4** đổi phân loại bằng `@keydown` **cục bộ trong lớp phủ**, KHÔNG qua registry — đúng tiền lệ `GlossaryQuickAdd.vue:55-60`. Phím số trần đã hứa cho Chế độ đọc (UX-DR46); một hợp âm toàn cục ở đây là tranh chấp.
- 🔴 **Rỗng phải nói vì sao nó rỗng.** `glossary_pending_candidates` trả `Ok(vec![])` khi **chưa mở Tác phẩm nào** — cùng hình dạng với "đã mở, bảng chờ sạch". Lớp phủ phải phân biệt được hai ca đó trên màn hình, không để một danh sách rỗng tự kể chuyện.
- Vỏ `wire` dùng `try_state`; adapter `src/config/glossary.ts` **không bao giờ ném**; trường trả về giữ `snake_case`.
- Lùi hàng đã xử lý bằng **màu chữ** `--color-on-surface-variant` + dấu `✓`/`✕`. **Cấm `opacity`** — không miễn trừ, kể cả có tên. Chính mockup của bảng chờ này là chỗ luật đó ra đời (`EXPERIENCE.md:490`).

**Ask First:**
- Bất kỳ phụ thuộc mới nào (NFR15).
- Nếu hoá ra **buộc** phải thêm một cột (một bước di trú **v15**): dừng và trình lý do.
- Nếu một cặp màu mới trượt sàn AA ở một trong hai theme: dừng, trình số đo — đừng đổi sàn.

**Never:**
- Không phím "để lại, tính sau" (`Space` trong mockup). Ice chốt 2026-08-24: không dựng — nó là thứ kéo theo bước di trú.
- Không bộ lọc theo phân loại, không "bỏ hàng loạt", không số Chương xuất hiện, không nhiều ví dụ ngữ cảnh, không đoán phân loại — mockup vẽ cả năm, **không cột nào đỡ chúng**. Ghi nợ, đừng bịa.
- Không sửa/xoá/lọc mục **đã ở trong** Glossary (Story 3.9), không CSV/TSV (Story 3.10).
- Không suy "đã duyệt" từ sự tồn tại của một hàng `glossary_entry` — Story 3.9 cho xoá mục, và một suy diễn chéo bảng làm ứng viên đã duyệt sống lại trong im lặng.

## I/O & Edge-Case Matrix

| Scenario | Input / State | Expected Output / Behavior | Error Handling |
|----------|--------------|---------------------------|----------------|
| Mở bảng chờ | Tác phẩm mở, N ứng viên `resolution IS NULL` | Danh sách sắp `occurrence_count DESC, id ASC`; con trỏ ở hàng đầu | N/A |
| Nhận, CÓ đề xuất | Hàng có `han_viet_status === 'ok'` | `approve(id, han_viet_suggestion, category)` ⇒ mục **đã chốt**; hàng lùi màu + `✓`; con trỏ tiến | Lỗi ⇒ hiện `tError`, hàng KHÔNG đổi, con trỏ KHÔNG tiến |
| Nhận, KHÔNG đề xuất | `han_viet_status` là một trong bốn nhánh còn lại | `approve(id, null, category)` ⇒ mục **chờ chốt** (FR114); hàng lùi + `✓`; con trỏ tiến | như trên |
| Bỏ | Hàng đang chọn | `reject(id)`; hàng lùi màu + `✕`; con trỏ tiến | như trên |
| Đổi phân loại | Bấm `1`–`4` | Phân loại của **hàng đang chọn** đổi; 0 lượt ghi cho tới khi Nhận | N/A |
| Đóng rồi mở lại | Đã quyết k hàng | Nạp lại; con trỏ ở ứng viên **chưa quyết** tần suất cao nhất | N/A |
| Chưa mở Tác phẩm | `open` là `None` | Câu nói rõ *"chưa mở Tác phẩm"* — **không** phải "bảng chờ trống" | N/A |
| Ứng viên đã quyết nơi khác | `resolution` đã có giá trị | `store.write_failed` mang `message_key`; hàng giữ nguyên | Hiện lỗi, nạp lại danh sách |
| Cầu IPC vắng | Chạy ngoài Tauri | `error: null`, danh sách rỗng kèm câu giải thích | Không ném |

</frozen-after-approval>

## Code Map

**Rust — sửa ít, thêm một vỏ**
- `src-tauri/src/core/glossary/candidate_store.rs:79-113` -- `pending_candidates`, `ORDER BY source_term` **cần đổi**; doc-comment ngay trên tự nhận là nợ của story này.
- `…/candidate_store.rs:238-280` `approve_candidate(store, id, translation: Option<&str>, category)` -- dùng nguyên, đã một giao dịch. `:290-309` `reject_candidate(store, id)` -- **0 chỗ gọi sản phẩm**, story này là chỗ đầu tiên.
- `src-tauri/src/commands/glossary.rs:355-416` -- `glossary_pending_candidates` + `GlossaryCandidateWire` (đã chở `occurrence_count`/`context_example`/`han_viet_*`, ghép theo KHOÁ không theo vị trí). `:468-477` `glossary_approve_candidate` -- **khuôn chép** cho vỏ bỏ. `wire` module `:481-705`, `try_state`.
- `src-tauri/src/lib.rs:369-384` -- danh sách `invoke_handler`; thêm vỏ thứ tám.
- `src-tauri/src/core/glossary/entry.rs:27-40` -- `enum Category` bốn giá trị, thứ tự cho phím `1`–`4`.
- `src-tauri/tests/glossary_boundary.rs:181-190` -- `QUICK_ADD_SURFACE` **8 → 9** (`reject_candidate`); `:132-139` ghi tên chủ là story này.
- `src-tauri/src/core/store/schema.rs:405-436` -- DDL bảng chờ; câu ở `:412` giao ba cột cho story này (không lấy — §Design Notes).

**Frontend — bề mặt mới**
- `src/config/glossary.ts:376-419` `GlossaryCandidate` + guard lúc chạy (đủ mọi trường). `:436` `pendingGlossaryCandidates()`, `:521` `approveGlossaryCandidate()` -- cả hai **0 chỗ gọi sản phẩm**; thêm `rejectGlossaryCandidate()` cùng khuôn ba trạng thái.
- `src/GlossarySettingsOverlay.vue` -- **khuôn modal**: scrim, `role="dialog"`, bẫy Tab tự viết `:73-89`, focus-return `:39-61`.
- `src/glossarySettingsState.ts:45,137-148` -- khuôn `ref` module-level **KHÔNG reset lúc đóng**. Phản mẫu phải tránh: `src/config/shortcutsState.ts:291-294` xoá `aimedRow` lúc đóng ⇒ mất vị trí.
- `src/GlossaryQuickAdd.vue:55-60` `onCategoryKeydown` -- khuôn phím số cục bộ.
- `src/commands/index.ts:1547-1590` -- khuôn đăng ký lệnh glossary (`labelKey`, `portMissing`, phím mặc định họ `Mod+Alt+…`); `:66-73` `FOCUS_OWNERS`.
- `src/main.ts:151-158` -- khuôn tiêm handler qua `CommandDeps` (state Vue **không** được import vào `commands/index.ts`).
- `src/App.vue` -- chỗ dựng ba lớp phủ hiện có. `src/i18n/vi.json:241-272` -- nhóm `glossary.*`. `src/tokens/tokens.json:24,43,536` -- `on-surface-variant` + bảng `contrast.pairs`.
- Khuôn test: `tests/frontend/glossarySettings.test.ts` (lớp phủ), `glossaryQuickAddStrip.test.ts` (`mount()`), `glossaryConfirmStrip.test.ts` (mock `invoke` ở biên IPC).

**Cổng sẽ phán**: `check:commands` Kiểm A/B/E · `check:tokens` Kiểm C (tương phản) / D (opacity) · `check:i18n` Kiểm A/A2 (chuỗi cứng).

## Tasks & Acceptance

**Execution:**
- [x] `src-tauri/src/core/glossary/candidate_store.rs` -- `ORDER BY occurrence_count DESC, id ASC`; sửa doc-comment nợ tại chỗ kèm 🔵 và ngày -- `id` là mốc phụ TẤT ĐỊNH và duy nhất; thiếu nó, hai ứng viên cùng tần suất đổi chỗ giữa hai lượt mở và "mở lại đúng vị trí" thành ngẫu nhiên.
- [x] `src-tauri/src/commands/glossary.rs` -- hàm thuần `glossary_reject_candidate(open, id)` + vỏ `wire`, chép khuôn `glossary_approve_candidate` (`no_work_open` khi chưa mở Tác phẩm) -- vỏ IPC đầu tiên của `reject_candidate`.
- [x] `src-tauri/src/lib.rs` -- đăng ký vỏ mới kèm chú thích nói story và FR.
- [x] `src-tauri/tests/glossary_boundary.rs` -- `QUICK_ADD_SURFACE` 8 → 9; cập nhật doc-comment đang ghi `reject_candidate` là nợ mở -- cổng thật, sẽ đỏ nếu bỏ qua.
- [x] `src-tauri/tests/glossary_commands_contract.rs` -- hợp đồng vỏ bỏ (thành công · `id` lạ · ứng viên đã quyết · chưa mở Tác phẩm) và **thứ tự** của `pending_candidates` gồm ca đồng hạng tần suất.
- [x] `src/config/glossary.ts` -- `rejectGlossaryCandidate(id)`, ba trạng thái lỗi như các hàm anh em.
- [x] `src/glossaryQueueState.ts` (mới) -- danh sách đã nạp, con trỏ, phân loại đang chọn theo hàng, kết quả `✓`/`✕` theo hàng, cờ `hasLoaded` phân biệt ba ca rỗng; chống đua bằng `sequence` -- **không** reset trong hàm đóng.
- [x] `src/GlossaryQueueOverlay.vue` (mới) -- modal theo khuôn `GlossarySettingsOverlay`; mỗi hàng: thuật ngữ · số lần · ví dụ ngữ cảnh · đề xuất khi có; hàng đã xử lý đổi màu + ký hiệu.
- [x] `src/commands/index.ts` -- `glossary.queue.open` (phím mặc định `Mod+Alt+Q`) · `accept` · `reject` · `next` · `prev` · `close` (**0** hợp âm mặc định, đúng chủ ý `glossary.save_term`) -- `createKeymap` ném khi hai lệnh giành cùng hợp âm, nên xung đột là lỗi lúc dựng chứ không phải một phím chết im lặng.
- [x] `src/main.ts` + `src/App.vue` -- tiêm sáu handler qua `CommandDeps`, dựng lớp phủ + một nút đường vào ở titlebar (`data-glossary-queue-open`, cùng khuôn `glossary.settings.open`).
- [x] `src/i18n/vi.json` -- nhãn `command.glossary.queue.*` + chuỗi `glossary.queue.*` (gồm **ba** câu rỗng khác nhau).
- [x] `src/tokens/tokens.json` -- **0 cặp mới cần thêm**, xem §Spec Change Log ("Kiểm C đã đầy đủ trước khi story này chạm tới").
- [x] `tests/frontend/glossaryQueue.test.ts` -- phủ §I/O Matrix: thứ tự, hai nhánh Nhận, Bỏ, con trỏ tiến, con trỏ ĐỨNG YÊN khi lỗi, ba ca rỗng, phím số đổi phân loại mà không ghi (21 ca).

**Acceptance Criteria:**
- Given bảng chờ có ứng viên cùng tần suất, when mở hai lần liên tiếp, then thứ tự hai lượt **giống hệt nhau**.
- Given người dùng duyệt k mục rồi đóng và mở lại, when lớp phủ nạp xong, then k mục đó **không còn** trong danh sách và con trỏ đứng ở ứng viên chưa quyết có tần suất cao nhất.
- Given toàn bộ luồng duyệt, when thực hiện từ mở tới đóng, then **không lượt gõ chữ nào** cần thiết.
- Given một hàng đã xử lý, when hiển thị, then nó phân biệt bằng màu chữ và ký hiệu, và `check:tokens` Kiểm D xanh **không cần** một miễn trừ `aura-allow-opacity` nào.
- Given mọi lệnh của bảng chờ, when `check:commands` chạy, then mỗi lệnh có nhãn trong `vi.json` và mọi `@click` là đúng một `dispatch`.

## Spec Change Log

### 2026-08-24 — thực thi

**Quyết định thực thi không tường minh trong spec, ghi lại để lượt sau đọc được lý do:**

- **`tokens.json::contrast.pairs` KHÔNG nhận cặp mới nào.** Task 101 dự liệu một cặp fg/bg
  mới cho lớp phủ. Đo trước khi tin (`src/tokens/tokens.json::roles`): `text` × `surface` là
  MỘT cross-product ĐÃ ĐÓNG (7 vai chữ × 6 vai nền = 42 tổ hợp, toàn bộ đã nằm ở `pairs` hoặc
  `excluded`, `check:tokens` Kiểm C cưỡng chế tính đầy đủ đó). `GlossaryQueueOverlay.vue` chỉ
  dùng bốn vai chữ đã khai (`on-surface` · `on-surface-variant` · `primary` · `error`) trên
  ba vai nền đã khai (`background` · `surface` · `surface-accent`) — cả chín tổ hợp đã có sẵn
  trong `pairs`. Thêm một cặp TRÙNG là vi phạm "Đừng thêm token để cho khớp một con số cũ"
  (`AGENTS.md`); không thêm gì là đúng, không phải một chỗ sót. Đối chứng: `check:tokens`
  Kiểm C chạy sau khi `GlossaryQueueOverlay.vue` đã tồn tại vẫn báo **31 cặp** (số cũ, không
  đổi) và cả 31 đều đạt AA ở hai theme.
- **"Chưa mở Tác phẩm" vs "đã mở, bảng chờ sạch" phân biệt bằng MỘT lượt `lookupGlossaryTerm('')`
  bổ sung, KHÔNG đổi hình dạng trả về của `glossary_pending_candidates`.** §Code Map giới hạn
  Rust ở đúng "một vỏ IPC (bỏ ứng viên) và một `ORDER BY`" — `glossary_pending_candidates`
  trả `Ok(vec![])` cho CẢ HAI ca (đã ghi trong doc-comment của chính nó từ Story 3.5) và
  KHÔNG được đổi chữ ký để tự phân biệt. `glossaryQueueState.ts::openGlossaryQueue` gọi
  THÊM `lookupGlossaryTerm('')` (adapter đã có từ Story 3.3, không một vỏ IPC mới) — CHỈ khi
  danh sách rỗng — để đọc `workTierAvailable`, đúng cờ mà `QuickAddLookup`/`quickAddWorkTierAvailable`
  đã dùng cho cùng câu hỏi. Khi danh sách CÓ hàng, câu hỏi tự trả lời (bảng chờ chỉ tồn tại ở
  `project.db`), nên lượt gọi thêm không bao giờ chạy trên đường có dữ liệu.
- **Hàng "Ứng viên đã quyết nơi khác" (§I/O Matrix) — lỗi hiện ra, KHÔNG tự động nạp lại
  danh sách giữa phiên.** Cột "Error Handling" của hàng đó viết "Hiện lỗi, nạp lại danh
  sách", trong khi hàng "Nhận, CÓ đề xuất" ngay trên viết "hàng KHÔNG đổi, con trỏ KHÔNG
  tiến" cho MỌI lỗi khác. Hai câu này áp cho hai kịch bản gốc rễ khác nhau nhưng
  KHÔNG PHÂN BIỆT ĐƯỢC ở tầng lỗi: `already_decided_error` (Rust) và mọi
  `StoreError::WriteFailed` khác đều đi ra `message_key = "store.write_failed"` — frontend
  không có cách nào tách hai ca đó từ chính lỗi nhận được (đo: `grep "message_key"
  candidate_store.rs` — cả hai nhánh dùng chung một khoá). Chọn: MỌI lỗi Nhận/Bỏ hiện qua
  `queueActionError`, hàng/con trỏ KHÔNG đổi — thống nhất với hàng lỗi chung. "Nạp lại danh
  sách" xảy ra ở đường ĐÃ CÓ SẴN của I/O Matrix: "Đóng rồi mở lại" (mỗi lượt mở luôn tải lại
  từ đầu). Một thiết kế thay thế (tự động `reload` khi lỗi, gắn nhãn hàng "đã bị quyết nơi
  khác" bằng một dấu THỨ BA) bị bác: §Always chỉ cho phép ĐÚNG hai dấu (`✓`/`✕`) cho hàng
  đã xử lý — một dấu thứ ba là mở rộng UI ngoài mệnh đề đã đóng băng.

### 2026-08-24 — vòng rà ba lớp, chín bản vá

Vòng rà chạy trên diff đầy đủ. `<frozen-after-approval>` **không bị chạm** — không phát
hiện nào đòi đổi §I/O Matrix, nên **0 `intent_gap`, 0 `bad_spec`**.

**Chín bản vá:**

1. **P1 — `GlossaryQueueOverlay.vue::onKeydown` không lọc phím bổ trợ.** `⌘N` khớp `case
   'n'` y hệt một phím `N` trần ⇒ Nhận một ứng viên ngoài ý định (trái AD-20/FR55); `⌘1`
   đổi phân loại ngoài ý định. Vá: thoát sớm khi `ctrlKey`/`metaKey`/`altKey`; `Shift` cố ý
   không lọc (không đổi `event.key` của digit ở bố cục US, không xung đột `trapTab`).
2. **P2 — 0 test `mount()` `GlossaryQueueOverlay.vue`.** Thêm bốn `describe` mount thật vào
   `tests/frontend/glossaryQueue.test.ts` (khuôn `glossarySettings.test.ts:245-274`): vùng
   chọn thật vai `display`, bốn phím `n`/`b`/mũi tên dispatch đúng lệnh qua
   `installCommands()` THẬT (không một bản giả registry), phím số đổi phân loại mà không
   dispatch, và đối chứng P1 (`⌘N`/`⌘1` không kích hoạt gì + đối chứng dương cùng phím
   không bổ trợ vẫn chạy).
3. **P3 — `glossary_contract.rs` khai sai thứ tự.** Hai `assert_eq!` còn viết `"ORDER BY
   source_term"` sau khi story đổi sang `occurrence_count DESC, id ASC`; ca vẫn xanh vì
   fixture dùng `insert_candidate` (không đặt `occurrence_count`, mặc định 0 cho mọi hàng)
   nên suy biến về nhánh đồng hạng (`id ASC`, trùng thứ tự chèn). Sửa thông điệp tại chỗ kèm
   🔵 + ngày, giải thích vì sao fixture NÀY vẫn ra đúng thứ tự đó — không đổi fixture vì mục
   đích GỐC của ca (vòng lặp NHIỀU hàng) không cần tần suất khác nhau.
4. **P4 — template tự dựng lại `queueRows[queueCursor]` thay vì dùng `queueCurrentRow`.**
   `glossaryQueueState.ts` đã export `queueCurrentRow` (`.at()`, có doc-comment giải thích
   vì sao) đúng để tránh phép truy cập mảng trần — template lại viết lại đúng phép đó. Đổi
   cả hai chỗ sang `queueCurrentRow`.
5. **P5 — đua con trỏ giữa `next`/`prev` và lượt TỰ TIẾN sau Nhận/Bỏ.** `cursor.value =
   next` vô điều kiện sau `await` đè mất một lượt điều hướng người dùng vừa bấm trong lúc
   ghi đang bay. Vá: chỉ tự tiến khi `cursor.value === index` (con trỏ vẫn đứng ở hàng vừa
   quyết) — cùng lớp vệ với `sequence`, khác chỗ đua (đồng bộ chồng lên bất đồng bộ, không
   phải hai round-trip IPC).
6. **P6 — không có trạng thái "đang tải".** `queueStatus === 'unknown'` không khớp nhánh
   `v-if`/`v-else-if` nào ⇒ thân modal trắng trong khoảng round-trip đầu. Thêm khoá
   `glossary.queue.loading` + một nhánh.
7. **P7 — hai tín hiệu chỉ tồn tại bằng MÀU.** Dấu `✓`/`✕` mang `aria-hidden="true"` không
   kèm văn bản thay thế; chip phân loại đang chọn không có `aria-pressed`. Vá: thêm
   `aria-pressed` cho chip (giữ nguyên phần nhìn), và một `span.gq-sr-only` (khuôn
   `App.vue::.sr-announcer`, ẩn bằng `clip`/`position: absolute`, KHÔNG `display: none`)
   mang văn bản `glossary.queue.row_status_accepted`/`_rejected` cho hàng đã xử lý — dấu
   `✓`/`✕` VẪN `aria-hidden` (decorative, phần nhìn không đổi).
8. **P8 — `.gq-context` cắt cụt không lối đọc lại.** Thêm `:title="row.candidate.
   context_example ?? ''"` — tooltip gốc trình duyệt, 0 CSS/JS mới.
9. **P9 — doc-comment của P1 khai SAI CƠ CHẾ chặn `⌘1`/`⌘2`/`⌘3`.** Bản đầu của P1 viết
   rằng ba lệnh chuyển chế độ bị nuốt vì "`preventDefault()` chạy trước `attachKeyboard`" —
   sai cả hai vế: `preventDefault()` không chặn nổi bọt, và listener của keymap chạy ở pha
   `capture` trên `window` (`src/commands/keys.ts:585-593`), tức TRƯỚC handler DOM cục bộ,
   không sau. Thủ phạm thật của việc ba lệnh chuyển chế độ không bắn là
   `KeymapGate.isBlocked` (`src/main.ts:527-532`, `queueOverlayIsOpen.value` nằm trong danh
   sách) — nó chặn bằng cách THOÁT SỚM, không `preventDefault()`, nên nuốt mọi hợp âm khỏi
   `CommandRegistry` trong khi lớp phủ mở nhưng KHÔNG chặn đường DOM cục bộ song song
   (`onKeydown` của chính lớp phủ, không đi qua `isBlocked`) — đó mới là lỗ P1 bịt. Sửa lại
   doc-comment cho đúng cơ chế, trỏ `tệp:dòng` thật.

**Đối chứng đỏ→xanh THẬT (P1/P9):** tạm bỏ dòng lọc `ctrlKey || metaKey || altKey` ⇒ ca
mount *"⌘N/⌘1 KHÔNG kích hoạt gì"* đỏ đúng dự kiến (`acceptMock` bị gọi, category đổi thành
`'other'`≠ kỳ vọng) → khôi phục → xanh lại, 26/26 ca của `glossaryQueue.test.ts`.

**Hai lượt chạy `.githooks/pre-push` THẬT trong phiên này, cả hai exit 0** (số giây khác
nhau vì là hai lượt chạy riêng biệt trên cùng máy):

| Lượt | Cây | Kết quả |
|---|---|---|
| 1 | như agent thi hành giao (trước vòng rà) | xanh **111s** |
| 2 | sau chín bản vá của vòng rà ba lớp | xanh **92s** |

## Design Notes

**Vì sao 0 bước di trú, dù `schema.rs:412` giao ba cột cho story này.** Cả ba đều biến mất khi bỏ phím *"để lại, tính sau"* (Ice chốt 2026-08-24):

- *Con trỏ đang duyệt* — không cần lưu. Truy vấn là `resolution IS NULL`, nên một hàng đã quyết **tự rời** bảng chờ. Mở lại ⇒ ứng viên chưa quyết có tần suất cao nhất **chính là** chỗ đang dở. Điều này đúng qua cả lần khởi động lại ứng dụng, thứ mà một `ref` trong bộ nhớ không làm được — miễn là thứ tự tất định, và đó là lý do mốc phụ `id ASC` không phải một chi tiết làm đẹp. Có phím bỏ qua thì mệnh đề này sập: một ứng viên bị bỏ qua vẫn `resolution IS NULL` nên nó **không** rời hàng đợi, và con trỏ phải được nhớ riêng.
- *Phân loại* — `approve_candidate` đã nhận `category` **từ chỗ gọi**; nó chỉ cần sống tới lúc bấm Nhận, không cần sống trên đĩa.

⇒ Một cột chỉ nên tồn tại khi có câu hỏi mà truy vấn hiện tại không trả lời được. Ở đây chưa có.

**Phân loại mặc định là `other`.** Mockup vẽ *"máy đoán sẵn"* và ba chip lọc theo phỏng đoán, nhưng `surnames.rs:17-19` từ chối đúng vai đó bằng chữ: nó chỉ nới ngưỡng quét, **không** nhận diện tên người. Vẽ một phỏng đoán mà máy chưa hề tính là dựng một sự thật không có. Giá phải trả (tên người tốn hai phím thay vì một) là một món nợ có chủ, không phải một chỗ để bịa dữ liệu.

## Verification

**Commands — chạy LẦN CUỐI sau chín bản vá của vòng rà ba lớp, số đo dưới đây là số SAU:**
- `npm run build` -- kiểu sạch; chạy **trước** `cargo test`, thiếu `dist/` thì `cargo test` gãy ở khâu biên dịch.
  ✅ **Chạy thật 2026-08-24 (sau vòng rà)** — `vue-tsc --noEmit` (cả hai tsconfig) + `vite
  build` xanh, 0 lỗi kiểu. Hai cảnh báo `INEFFECTIVE_DYNAMIC_IMPORT` của Rollup ĐÃ CÓ TỪ
  TRƯỚC (ghi nhận từ Story 3.6/3.7), không liên quan tệp story này chạm.
- `cargo test --locked --manifest-path src-tauri/Cargo.toml` -- **584 ca hiện có xanh** + ca mới; `glossary_boundary` xanh **sau** khi `QUICK_ADD_SURFACE` lên 9.
  ✅ **Chạy thật 2026-08-24 (sau vòng rà)** — 0 failed trên **590 ca**, KHÔNG đổi so lượt
  thực thi (584 → 590, +6 ở `glossary_commands_contract.rs`: 15 → 21) — P3 chỉ sửa thông
  điệp `assert_eq!` của một ca đã có (`glossary_contract.rs`), không thêm/bớt ca nào.
  `glossary_boundary.rs` vẫn 11 ca, `QUICK_ADD_SURFACE` 8 → 9 phần tử.
- `npm test` -- vitest, gồm tệp mới.
  ✅ **Chạy thật 2026-08-24 (sau vòng rà)** — 36 tệp / **428 ca**, 0 failed.
  `glossaryQueue.test.ts` riêng: **26 ca** (21 ca state cũ + **5 ca mount THẬT mới**: vùng
  chọn `display`, bốn phím `n`/`b`/mũi tên dispatch qua `installCommands()` THẬT, phím số
  đổi phân loại mà không dispatch, đối chứng P1/P9 `⌘N`/`⌘1` không kích hoạt gì, dấu sr-only
  của hàng đã xử lý — đo riêng: `npx vitest run tests/frontend/glossaryQueue.test.ts` =
  26/26). 35 tệp còn lại (không tệp nào bị chạm ở vòng rà này): **402 ca** — đo riêng:
  `npx vitest run --exclude "tests/frontend/glossaryQueue.test.ts"` = 402/402. ⚠️ Lượt thực
  thi trước vòng rà báo tổng **422** — số 402 của 35 tệp còn lại KHÔNG khớp phép cộng ngược
  (422 − 21 = 401, không 402); không tìm được nguyên nhân chênh 1 ca trong khi git xác nhận
  0 tệp nào ngoài `glossaryQueue.test.ts` bị chạm ở vòng rà này. Ghi ra thay vì im lặng: số
  ĐÁNG TIN là số VỪA ĐO (428 tổng, 402 + 26), không phải một phép trừ từ một con số cũ.
- `npm run check:tokens` · `check:commands` · `check:i18n` · `check:lint` -- bốn cổng story này chạm.
  ✅ **Chạy thật 2026-08-24 (sau vòng rà)** — cả bốn xanh. `check:tokens` Kiểm C: **31 cặp,
  KHÔNG đổi** dù P7 thêm `aria-pressed` + một span sr-only (không phải màu/CSS mới cần cặp
  tương phản); cả hai theme đạt AA. Kiểm D: 0 `opacity` mới. `check:commands` Kiểm A/B/E:
  **66 command, không đổi** (P1/P4/P5/P6/P7/P8/P9 không thêm/bớt command nào). `check:i18n`
  Kiểm A/A2: **290 khoá `vi.json`** (287 → 290, +3: `glossary.queue.loading` ·
  `row_status_accepted` · `row_status_rejected`), 0 chuỗi hiển thị ở vị trí mã.
- `npm run check:panel-refs` · `check:layout` -- hai cổng khác story này chạm (state module
  mới, 0 thành viên `window`/`document` mới).
  ✅ **Chạy thật 2026-08-24 (sau vòng rà)** — cả hai xanh.
- `.githooks/pre-push` -- 11 cổng; xong rồi **đọc lượt CI** trước khi kết luận xanh (pre-push chỉ chạy nửa macOS).
  ✅ **Chạy thật 2026-08-24 (sau vòng rà)** — xanh trong **92s** (`deps` · `tokens` · `i18n` ·
  `commands` · `layout` · `panel-refs` · `dict` · `dict-manifest` · `lint` · `gates` ·
  `debt-owner` · `test` · `build` · `cargo test`). Lượt TRƯỚC vòng rà: xanh 111s — xem bảng
  §Spec Change Log.
  ⚠️ **CHƯA ĐỌC ĐƯỢC LƯỢT CI** — môi trường thực thi (agent CLI) không push được; đây là bước
  Ice làm sau khi nhận bàn giao (cùng giới hạn đã ghi ở Story 3.6/3.7).

**Bộ e2e:**
⚠️ **CHƯA CHẠY** — môi trường cài đặt không có GUI để dựng cửa sổ Tauri thật cho
`npm run test:e2e`. Đếm phần tĩnh: `grep` trên `e2e/specs/**` cho
`glossary_reject_candidate`/`glossary.queue` = **0** — 0 spec hiện có chạm bề mặt story này
dựng. Chạy tay + đọc số hồi quy là việc của Ice.

**Manual checks (chưa nghiệm thu bằng mắt trên bản dựng thật — môi trường agent không dựng
được cửa sổ Tauri):**
- Bảng chờ rỗng vì **chưa mở Tác phẩm** và rỗng vì **đã duyệt hết** phải đọc lên thành hai câu khác nhau trên màn hình.
  🟡 Đóng được vế CẤU TRÚC bằng `tests/frontend/glossaryQueue.test.ts` (mock IPC, không
  webview thật): ba trạng thái `no_work`/`loaded`-rỗng/`ipc_unavailable` được đo riêng biệt
  và ánh xạ đúng ba khoá `glossary.queue.empty_*` trong template.
- Duyệt vài mục, đóng ứng dụng hẳn, mở lại: danh sách ngắn đi đúng số đã quyết.
  🟡 Đóng được vế CẤU TRÚC — test "AC — đóng rồi mở lại" trong `glossaryQueue.test.ts` mô
  phỏng backend trả về danh sách đã bớt hàng đã quyết, và con trỏ khởi động lại ở hàng 0.
  Vế THỊ GIÁC/bàn phím thật (`Mod+Alt+Q`, `N`/`B`/mũi tên chạy trên webview thật) là việc
  của Ice — ghi nợ đo bằng mắt, cùng khuôn Story 3.6/3.7.

## Suggested Review Order

**Điểm vào — quyết định trung tâm của story**

- Thứ tự mới; mốc phụ `id ASC` là thứ làm "mở lại đúng vị trí" tất định.
  [`candidate_store.rs:74`](../../src-tauri/src/core/glossary/candidate_store.rs#L74)

- Vì sao 0 bước di trú: hàng đã quyết tự rời `resolution IS NULL`.
  [`3-8…-mot-phim.md:150`](3-8-duyet-hang-loat-mot-phim.md#L150)

**Đường ghi mới (Rust) — một vỏ IPC duy nhất**

- Vỏ IPC đầu tiên của `reject_candidate`; chép khuôn `approve`.
  [`glossary.rs:494`](../../src-tauri/src/commands/glossary.rs#L494)

- Đăng ký vỏ thứ tám vào `invoke_handler`.
  [`lib.rs:388`](../../src-tauri/src/lib.rs#L388)

- Cổng thật: danh sách được phép lên 9, đóng món nợ mang tên story này.
  [`glossary_boundary.rs:187`](../../src-tauri/tests/glossary_boundary.rs#L187)

**Trạng thái bảng chờ — chỗ "rỗng im lặng" bị chặn**

- Năm trạng thái, để một danh sách rỗng không tự kể chuyện.
  [`glossaryQueueState.ts:60`](../../src/glossaryQueueState.ts#L60)

- Phân biệt "chưa mở Tác phẩm" khỏi "bảng chờ sạch", chỉ khi rỗng.
  [`glossaryQueueState.ts:160`](../../src/glossaryQueueState.ts#L160)

- Nhận: đề xuất có ⇒ đã chốt, không ⇒ chờ chốt; con trỏ tiến có chốt đua.
  [`glossaryQueueState.ts:223`](../../src/glossaryQueueState.ts#L223)

- Adapter ba trạng thái cho lượt Bỏ.
  [`glossary.ts:561`](../../src/config/glossary.ts#L561)

**Lớp phủ — nơi vòng rà tìm ra lỗ nặng nhất**

- Lọc hợp âm bổ trợ: `⌘N` từng ghi thẳng một mục vào Glossary.
  [`GlossaryQueueOverlay.vue:140`](../../src/GlossaryQueueOverlay.vue#L140)

- Dấu `✓`/`✕` là trang trí; tín hiệu đọc được đi đường riêng.
  [`GlossaryQueueOverlay.vue:252`](../../src/GlossaryQueueOverlay.vue#L252)

**Lệnh, chuỗi, chỗ dựng**

- Sáu lệnh; chỉ lệnh mở mang hợp âm mặc định.
  [`index.ts:1670`](../../src/commands/index.ts#L1670)

- Tiêm handler qua `CommandDeps` — state Vue không vào `commands/`.
  [`main.ts:161`](../../src/main.ts#L161)

- Chỗ dựng lớp phủ thứ năm.
  [`App.vue:59`](../../src/App.vue#L59)

- Ba câu rỗng khác nhau, cộng câu đang tải.
  [`vi.json:286`](../../src/i18n/vi.json#L286)

**Nghiệm thu**

- Đối chứng `⌘N`/`⌘1` không kích hoạt gì, kèm đối chứng dương phím trần.
  [`glossaryQueue.test.ts:460`](../../tests/frontend/glossaryQueue.test.ts#L460)

- Hợp đồng thứ tự, gồm ca đồng hạng tần suất.
  [`glossary_commands_contract.rs:247`](../../src-tauri/tests/glossary_commands_contract.rs#L247)

- Mệnh đề cũ đã hết đúng, sửa tại chỗ thay vì để nó lặng lẽ sai.
  [`glossary_contract.rs:1169`](../../src-tauri/tests/glossary_contract.rs#L1169)

---

## Nhật ký sprint-status

Gỡ nguyên văn từ `sprint-status.yaml` ngày 2026-08-26: tệp đó giữ TRẠNG THÁI, nội dung story
thuộc về tệp này. Không sửa một ký tự.

```
  # Spec: `3-8-duyet-hang-loat-mot-phim.md`, baseline `3170ce4db5d547106fd933bebde97aa2f3c8c500`.
  # 0 bước di trú — `project.db` giữ v14. Ice chốt 2026-08-24 bỏ phím "để lại, tính sau", và
  # chính quyết định đó làm hai cột mà `schema.rs:412` giao cho story này trở nên thừa: hàng
  # đã quyết tự rời `WHERE resolution IS NULL`, nên "mở lại đúng vị trí" đến từ mô hình dữ
  # liệu chứ không từ một con trỏ lưu xuống đĩa.
```

