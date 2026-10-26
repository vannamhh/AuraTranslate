---
title: 'Tác phẩm nhớ bộ prompt đã chọn; danh sách Bộ prompt hai cột'
type: 'feature'
ticket: ''
created: '2026-10-26'
status: 'built'
baseline_revision: '7ed0c1aa72566e81dcb3a5f349ccb4f1f03fe6ee'
route: 'full'
route_source: 'auto'
risk: 'medium'
review: 'quick'
review_source: 'pinned'
lenses_ran: ['quick']
review_loop_iteration: 0
context:
  - '{project-root}/src-tauri/AGENTS.md'
  - '{project-root}/src/AGENTS.md'
---

<frozen-after-approval reason="human-owned intent — do not modify unless human renegotiates">

## Intent

**Problem:** Bộ prompt chọn ở panel Đề xuất AI chỉ nằm trong bộ nhớ webview (`promptSetState.ts:56`): khởi động lại app là mất; mở một Tác phẩm có thể loại trùng tên một bộ thì bộ theo thể loại đè lựa chọn tay (`applyGenre`, `promptSetState.ts:194-196`). Ice phải chọn lại liên tục. Danh sách Bộ prompt (29 mẫu trở lên) là một cột dài.

**Approach:** Mỗi Tác phẩm nhớ tên bộ prompt Ice chọn tay, lưu ở `project.db`. Mở Tác phẩm: bộ đã lưu còn giải được thì chọn nó; không có thì mới tự chọn theo thể loại. Danh sách Bộ prompt hiện hai cột bằng `SelectMenu :columns="2"` sẵn có.

**Đã chốt (Ice, 2026-10-26):**
- Lưu theo từng Tác phẩm (không phải một bộ chung cho cả app).
- Hai cột.
- Giữ một plan dù có thể vượt 1600 token (1119 từ).
- Không cần AD mới: đây là một lựa chọn được ghi nhớ của Tác phẩm, cùng loại `work.last_chapter_id` (bậc 26), không đổi chính sách nào như `tm_only_mine` (AD-57). Quyết định #4 của Story 4.4 chỉ nói `prompt_set` mang tên + thân, không nói về lựa chọn.

## Boundaries & Constraints

**Always:**
- Cột mới `work.chosen_prompt_set_name TEXT` (NULL = chưa chọn tay) ở bậc `PROJECT_MIGRATIONS` 41, bằng `ALTER TABLE work ADD COLUMN`. Ghi cột này không đụng `updated_at`, không dựng lại `meta.json`, không reindex (khuôn `tm_only_mine`).
- Chỉ lượt chọn tay ghi xuống đĩa; tự chọn theo thể loại không ghi. Chọn "— Chưa chọn —" bằng tay ghi NULL.
- Thứ tự khi mở Tác phẩm: bộ đã lưu (so khớp tên sau trim, không phân biệt hoa thường như `applyGenre`) ⇒ thể loại ⇒ không chọn. Bộ đã lưu được chọn với `selectionIsAutomatic = false`.
- Ghi lỗi thì báo qua `tError`; lựa chọn trong phiên vẫn giữ.

**Never:**
- Không đổi bảng `prompt_set`, không đổi Global tier, không dùng `localStorage`.
- Không tự sửa tên đã lưu khi bộ bị đổi tên/xoá; tên không còn giải được thì coi như chưa lưu.

## I/O & Edge-Case Matrix

| Scenario | Input / State | Expected Output / Behavior | Error Handling |
|----------|--------------|---------------------------|----------------|
| Nhớ qua khởi động lại | Ice chọn "Thời sự" ở Tác phẩm A, thoát app, mở lại A | "Thời sự" được chọn sẵn | — |
| Đè thể loại | Tác phẩm A thể loại Tiên hiệp, có bộ "Tiên hiệp", Ice đã chọn "Thời sự" | Mở A ⇒ "Thời sự" | — |
| Chưa từng chọn tay | Tác phẩm B thể loại Pháp lý, có bộ "Pháp lý", cột NULL | Mở B ⇒ "Pháp lý" (tự chọn), cột vẫn NULL | — |
| Mỗi Tác phẩm riêng | A nhớ "Thời sự", B nhớ "Pháp lý" | Đổi qua lại A/B ⇒ mỗi bên đúng bộ của mình | — |
| Bộ đã lưu bị xoá | A nhớ "Cũ", bộ "Cũ" đã xoá | Mở A ⇒ theo thể loại; không lỗi | — |
| Bỏ chọn tay | Ice chọn "— Chưa chọn —" ở A | Cột về NULL; lần mở sau theo thể loại | — |
| Lỗi ghi | Ghi cột lỗi IO | Báo `tError`; lựa chọn trong phiên giữ | Không thử lại |

</frozen-after-approval>

## Code Map

- `src-tauri/src/core/store/schema.rs:2256` -- `PROJECT_MIGRATIONS` kết thúc ở bậc 40; thêm bậc 41. Khuôn DDL `WORK_TM_ONLY_MINE_DDL` (`:1064-1068`, bậc 37).
- `src-tauri/src/core/tm/mod.rs:245-265`, `src-tauri/src/commands/tm.rs:331-347,953-983`, `src/config/tm.ts:103-104` -- khuôn đọc/ghi một cột của `work` + lệnh + wire + adapter TS. Lệnh mới đăng ký ở `src-tauri/src/lib.rs` (cạnh `:1372-1387` của `prompt_set_*`).
- `src-tauri/src/core/library/indexer.rs:2146-2153` -- luật indexer khi đường đọc Library đọc cột `work`; cột này không cần vào Library.
- `src/promptSetState.ts:56,169-172,182-204` -- `selectedSetName`, nhánh tên không còn giải được, `setSelectedPromptSetName`, `applyGenre`, `autoSelectPromptSetForGenre`. `:19-25` comment "trạng thái webview thuần" cần sửa cho đúng.
- `src/modes/libraryChapters.ts:362`, `src/modes/libraryImport.ts:459` -- chỗ gọi tự chọn khi mở/tạo Tác phẩm.
- `src/panels/AiTranslationPanel.vue:491-498` -- `SelectMenu variant="field"`; `SelectMenu.vue:24,107-120` đã có `columns?: 2 | 3`.
- Test hiện có: `tests/frontend/promptSetState.test.ts:565-640` (ca thể loại, `:589-590` khẳng định thể loại đè lựa chọn tay — sẽ đổi), `selectMenu.test.ts`, `libraryChaptersResetsAiPromptInspector.test.ts`; Rust `tests/project_contract.rs` / migration tests, `config_invariants.rs` và `ipc_contract.rs` đếm lệnh.

## Tasks & Acceptance

**Execution:**
- [x] `src-tauri/src/core/store/schema.rs` -- bậc 41 thêm `work.chosen_prompt_set_name TEXT`.
- [x] `src-tauri/src/core/promptset/` + `src-tauri/src/commands/promptset.rs` + `src-tauri/src/lib.rs` -- `prompt_set_choice_get() -> Option<String>`, `prompt_set_choice_set(name: Option<String>)` cho Tác phẩm đang mở (không có Tác phẩm ⇒ `work_tier_unavailable`); không đụng `updated_at`/`meta.json`.
- [x] `src/config/promptset.ts` -- adapter hai lệnh, không ném.
- [x] `src/promptSetState.ts` -- lượt chọn tay (`setSelectedPromptSetName`, kể cả `null`) ghi cột khi có Tác phẩm; hàm mới cho lúc mở Tác phẩm: đọc cột ⇒ chọn nếu giải được ⇒ không thì `applyGenre`; sửa comment `:19-25`.
- [x] `src/modes/libraryChapters.ts`, `src/modes/libraryImport.ts` -- gọi hàm mới thay `autoSelectPromptSetForGenre`.
- [x] `src/panels/AiTranslationPanel.vue` -- `:columns="2"` cho ô Bộ prompt.
- [x] Rust test -- migration 41 trên `project.db` cũ (cột NULL), get/set khứ hồi, set không đổi `updated_at`; cập nhật `config_invariants`/`ipc_contract` cho hai lệnh mới.
- [x] `tests/frontend/promptSetState.test.ts` -- mọi hàng ma trận; sửa ca `:589-590` theo luật mới (bộ đã lưu thắng thể loại; chưa lưu thì thể loại vẫn đè như cũ); `tests/frontend/selectMenu.test.ts` hoặc ca panel -- ô Bộ prompt có hai cột.
- [x] Đối chứng đỏ: gỡ thật lượt đọc cột khi mở Tác phẩm ⇒ ca "Nhớ qua khởi động lại" đỏ; gỡ thật lượt ghi khi chọn tay ⇒ ca ghi đỏ.

**Acceptance Criteria:**
- Given Ice chọn một bộ ở Tác phẩm A, when khởi động lại app và mở lại A, then bộ đó được chọn sẵn.
- Given ô Bộ prompt có trên 10 bộ, when mở danh sách, then các bộ xếp thành hai cột và phím mũi tên vẫn đi qua hết.

## Implementation Notes

- Mở một Tác phẩm chưa lưu lựa chọn thì lựa chọn trong phiên của Tác phẩm trước bị xoá trước khi áp thể loại; nếu không, lựa chọn của A sẽ mang sang B.
- `autoSelectPromptSetForGenre` không còn chỗ gọi nào nên đã xoá. `restorePromptSetForWork` đặt `lastGenre`, nên "Thêm từ mẫu" vẫn tự chọn cho Tác phẩm đang mở.
- Ghi cột chỉ khi `workTierAvailable` (cờ của lượt nạp gần nhất). Đóng Tác phẩm rồi chọn thì Rust trả `work_tier_unavailable` và lỗi đó được báo ra, không ghi sai chỗ.
- Schema 41 kéo theo việc sửa con số phiên bản trong 9 tệp test Rust và nâng `REGISTERED_COMMAND_FLOOR` từ 137 lên 147. Lượt `cargo test` đầy đủ bắt thêm ba ca phase 1 sót: `segment_role_contract`, `tm_contract` (kỳ vọng 40) và fixture hạ về bậc 28 trong `tm_contract` quên xoá nhật ký bậc 41, nên mở lại thất bại `store.open_failed`.
- `src/config/aiconfig.ts` (biểu thức ba ngôi làm `ipc_argument_contract` đỏ từ `983ef03`) đã sửa ở commit riêng `3b5eca7d57fb1f16485d426da43d0c9003401ab1`.

## Plan Change Log

## Review Triage Log

Lượt 1 (quick): high 1 · medium 0 · low 4 · false 2 · rejected-low 2.
- high, patch — `loadPromptSets` xoá cờ khôi phục trước khi chờ đọc cột; một lượt nạp chen vào (panel AI nạp ngay khi hiện ra) làm cả hai lượt bỏ khôi phục, nên Tác phẩm mở ra không có bộ nào ⇒ chỉ xoá cờ khi lượt đó còn là lượt mới nhất sau lúc chờ; thêm ca có lượt nạp chen vào.
- low, patch — đọc cột lỗi thì im lặng coi như chưa lưu ⇒ báo `promptSetChoiceError` rồi mới áp thể loại.
- low, rejected — `workTierAvailable` là cờ cũ sau khi đóng Tác phẩm: Rust trả `work_tier_unavailable` và lỗi được báo ra; ca chọn rồi đổi Tác phẩm cực nhanh hiếm gặp.
- low, patch — Thư viện prompt cũng ghi lựa chọn mà không hiện lỗi ghi ⇒ hiện `promptSetChoiceError` ở đó.
- false — không tái lập được đối chứng đỏ: handoff ghi ba phép gỡ thật (đọc: 5 ca đỏ; ghi: 4 ca đỏ; cột: 1 ca đỏ).
- low, rejected — phím mũi tên đi qua hai cột không có test riêng: `selectMenu.test.ts` đã phủ bàn phím cho `columns`; happy-dom không có hình học bố cục.
- false — `aiconfig.ts` nằm ngoài phạm vi: đã commit riêng ở `3b5eca7`, nằm sau baseline.
- low, patch — thông báo assert ở `pinned_contract.rs` còn ghi "ba mươi lăm bước".
- false — sổ plan chưa cập nhật: phần này do người điều phối ghi, đã ghi.

## Verification

**Commands:**
- `cargo test --test project_contract --test config_invariants --test ipc_contract --test prompt_set_contract` (sau `npm run build`) -- expected: xanh.
- `npx vitest run tests/frontend/promptSetState.test.ts tests/frontend/selectMenu.test.ts` -- expected: xanh.
- Chạm migration và `lib.rs` ⇒ chạy đủ: 14 cổng, toàn bộ vitest (mã thoát 0, không dòng `Errors`), toàn bộ `cargo test` -- expected: xanh.

**Manual checks (if no CLI):**
- Trong app thật: chọn bộ ở một Tác phẩm, thoát app, mở lại Tác phẩm đó ⇒ bộ còn được chọn.
