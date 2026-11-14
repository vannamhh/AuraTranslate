---
title: 'Bảng Mod+K: tìm mở Tác phẩm và chạy lệnh'
type: 'feature'
ticket: ''
created: '2026-11-14'
status: 'in-progress'
baseline_revision: '0a76018144e1a0bc827a82b954b83d2bebd40296'
route: 'full'
route_source: 'auto'
risk: 'medium'
review: ''
review_source: ''
lenses_ran: []
review_loop_iteration: 0
context: []
---

<frozen-after-approval reason="human-owned intent — do not modify unless human renegotiates">

## Intent

**Problem:** Mở một Tác phẩm phải quay về Library rồi tìm trên lưới bìa. Một thao tác không có phím thì phải nhớ nó nằm ở menu nào. App chưa có lối vào bằng bàn phím nào gom cả hai.

**Approach:** Thêm một lớp phủ kiểu Spotlight, mở bằng `Mod+K`: một ô gõ và một danh sách kết quả chia hai nhóm. Nhóm Tác phẩm lọc theo tên, không phân biệt dấu. Nhóm Lệnh lọc theo nhãn i18n của command. Enter hoặc bấm chuột mở Tác phẩm hay chạy lệnh đang chọn.

**Decisions (Ice, 2026-11-14):**
- Chọn một Tác phẩm thì vào thẳng Workspace ở vị trí đọc cuối, giống `library.continue_translation` (FR12): `openWorkById`, và chỉ khi nó trả `true` mới `setMode('workspace')`.
- Nhóm Lệnh chỉ liệt kê command có đánh dấu, qua một trường mới tuỳ chọn trên `CommandSpec` (`palette?: true`). Chỉ đánh dấu lối vào: lệnh chạy đúng mà không cần một mục đang chọn hay một lớp phủ đang mở. Các lệnh `open/toggle` là ứng viên, trước đó có cỡ 50.
- Một plan duy nhất cho cả hai nhóm, dù vượt trần 1600 token.

## Boundaries & Constraints

**Always:**
- Mở, đóng, chọn và chạy đều là command trong registry, có nhãn `command.<id>` (AD-34 §1). Lệnh mở có phím mặc định `Mod+K`, gán lại được (FR22).
- Lớp phủ nằm trong cửa sổ `main`, không phải chế độ thứ tư (AD-24). Nó theo khuôn của các `*Overlay.vue` hiện có: scrim, `role="dialog"`, giữ Tab trong lớp phủ, Esc đóng, `focusReturnTargetOnOpen`, đệm thanh tiêu đề trên mac, có tên trong cổng `isBlocked`.
- Mở Tác phẩm đi qua `openWorkById` để lượt flush Editor vẫn chặn trước khi đổi Tác phẩm. Không gọi IPC `openWork` trực tiếp.
- Danh sách chưa nạp hoặc nạp lỗi phải hiện trạng thái riêng, không hiện "không có kết quả" (lỗi im lặng). Hỏi qua `libraryWorksHaveLoaded`.
- Chạy lệnh thì đóng lớp phủ và trả tiêu điểm trước, rồi mới `dispatch` ở `nextTick`, để lệnh nhắm tiêu điểm chạy trên đúng chỗ người dùng đứng.
- Mọi chữ nằm trong `vi.json`. Màu, bóng và z-index theo token, miễn trừ nào cũng có tên.

**Never:**
- Không coi một dòng trong bảng là điểm chuột của lệnh đó (AD-62, Kiểm M). Không thêm loại `pointer` thứ sáu, không thêm `POINTER_DEBT`.
- Không mở bảng khi đang có lớp phủ khác (cổng `isBlocked` đã nuốt `Mod+K`). Không lồng bảng vào lớp phủ khác.
- Không tìm theo nội dung Chương: FR8 đã có ô tìm Library. Không thêm danh sách "mở gần đây" (chưa có FR).
- Không thêm IPC hay cột Rust mới.

## I/O & Edge-Case Matrix

| Scenario | Input / State | Expected Output / Behavior | Error Handling |
|----------|--------------|---------------------------|----------------|
| Mở trong ô dịch | Con trỏ trong ô Editor, bấm `Mod+K` | Bảng mở, tiêu điểm vào ô gõ; Esc trả con trỏ về đúng ô cũ | — |
| Gõ không dấu | Tác phẩm tên "Đấu Phá Thương Khung", gõ `dau pha` | Tác phẩm này có trong nhóm Tác phẩm | — |
| Chưa nạp Library | Mở app thẳng vào Workspace rồi bấm `Mod+K` | Nhóm Tác phẩm hiện "đang tải" rồi tới kết quả | Nạp lỗi thì hiện câu lỗi, không hiện danh sách rỗng |
| Mở Tác phẩm khi ô dịch chưa lưu được | flush trả `still-dirty` | Không đổi Tác phẩm, bảng đóng, `openWorkNotice` hiện như ở Library | Theo nhánh hiện có của `openWorkById` |
| Lệnh không đánh dấu | Gõ nhãn của `settings.close` | Lệnh đó không có trong nhóm Lệnh | — |
| Không khớp | Gõ chuỗi không khớp gì | Một dòng "không có kết quả", Enter không làm gì | — |
| Lớp phủ khác đang mở | Settings mở, bấm `Mod+K` | Không có gì xảy ra | — |

</frozen-after-approval>

## Code Map

- `src/commands/registry.ts:51-124` -- `CommandSpec` (`id`, `labelKey`, `run`, `keys`, `pointer`). `list()` trả mọi command; không có predicate theo ngữ cảnh. `dispatch` không chặn theo chế độ.
- `src/commands/index.ts` -- `registerAll` là chỗ duy nhất đăng ký; mẫu mở lớp phủ `tm.manage.open` ở `:4433` và `settings.open` ở `:4969`; `library.open_work` ở `:2301`; `library.continue_translation` ở `:2323`; `portMissing` ở `:1416`.
- `src/main.ts:1194-1232` -- `attachKeyboard` cùng `isBlocked`: thêm cờ mở của bảng và nối các port `CommandDeps`.
- `src/commands/keys.ts:519-560` -- chord có phím mod chính chạy cả trong vùng gõ (`isTypingZone`), nên `Mod+K` không cần ngoại lệ. `formatChord` ở `:328`.
- `src/config/shortcutsState.ts:138-179,467` -- `shortcutRows`/`labelOf`: dùng lại để lấy nhãn và chord hiển thị của từng lệnh. Không đọc `spec.keys`, không gọi `unbound()` trong `.vue` (Kiểm L).
- `src/modes/libraryWorks.ts:44-204` -- `libraryWorks`, `libraryWorksHaveLoaded`, `libraryWorksBusy`/`Error`, `loadWorks`. `WorkRow` (`src/config/library.ts:221`) có `work_id`, `name`, `chapter_count`, `chapter_done_count`.
- `src/modes/libraryChapters.ts:368` -- `openWorkById`, kèm `openWorkNotice`; `src/modes/modeState.ts:36` -- `setMode`.
- `src/SettingsOverlay.vue:96-155,335-349,935` và `src/TmManageOverlay.vue` -- khuôn lớp phủ cần chép: trap Tab, trả tiêu điểm, scrim `z-index: 10` có `aura-allow-z-index`.
- `src/TitlebarToolsMenu.vue:141-150` -- điểm chuột của `tm.manage.open` và `settings.open`; mục mới cho `palette.open` đặt ở đây.
- `scripts/check-commands.mjs:242` -- `COMMAND_FLOOR`; Kiểm E (nhãn), Kiểm M (điểm chuột).
- Test để chép: `tests/frontend/tmManage.test.ts:550,763`, `settingsFrameDom.test.ts:73-95`, `dialogAccessibleName.test.ts`; `overlayCaretHold.test.ts:384` tự quét mọi `src/*Overlay.vue`.

## Tasks & Acceptance

**Execution:**
- [ ] `src/commands/registry.ts`, `src/commands/index.ts` -- thêm `palette?: true` vào `CommandSpec`, đặt cho các lệnh lối vào theo tiêu chí ở Decisions -- danh sách gọn, không có dòng chết
- [ ] `src/paletteState.ts` -- cờ mở, chuỗi gõ, chỉ số chọn, `paletteResults` tính từ `libraryWorks` và `shortcutRows` (chỉ các command có `palette`), hàm gập dấu tiếng Việt (NFD bỏ dấu kết hợp, `đ`→`d`); nạp Works khi chưa nạp -- trạng thái tách khỏi component để test không cần mount
- [ ] `src/commands/index.ts` -- đăng ký `palette.open` (`Mod+K`), `palette.close`, `palette.select_next`/`select_prev`, `palette.run_selected`; port mới trong `CommandDeps`; lệnh nào không có `@click` literal thì khai `pointer` thuộc năm loại sẵn có (Kiểm M) -- mọi hành động đi qua registry
- [ ] `src/PaletteOverlay.vue` -- lớp phủ theo khuôn ở Code Map; ô gõ `role="combobox"`, danh sách `role="listbox"`, mỗi lệnh hiện chord của nó; nút đóng `@click="dispatch('palette.close')"`
- [ ] `src/App.vue`, `src/TitlebarToolsMenu.vue`, `src/main.ts` -- gắn lớp phủ, mục menu cho `palette.open`, cờ vào `isBlocked`, nối port
- [ ] `src/i18n/vi.json`, `scripts/check-commands.mjs` -- nhãn `command.palette.*`, chữ của bảng; nâng `COMMAND_FLOOR` theo số đăng ký thật
- [ ] `tests/frontend/palette.test.ts` -- một ca cho mỗi hàng của Matrix, chord mặc định, gập dấu, chạy lệnh sau khi trả tiêu điểm

**Acceptance Criteria:**
- Given Ice đang ở bất kỳ chế độ nào và không có lớp phủ, when bấm `Mod+K`, gõ một phần tên Tác phẩm rồi Enter, then Workspace mở Tác phẩm đó ở Chương và vị trí đọc cuối.
- Given bảng đang mở, when gõ một phần nhãn của một lệnh rồi Enter, then lệnh chạy giống như bấm phím của nó, và bảng đã đóng.
- Given chỉ dùng chuột, when mở menu Công cụ trên thanh tiêu đề và chọn mục bảng lệnh, then bảng mở; bấm một dòng sẽ chạy dòng đó.
- Given Ice gán lại `palette.open` sang chord khác trong Settings, when bấm chord mới, then bảng mở, còn `Mod+K` không mở nữa.

## Design Notes

Gập dấu làm ở webview vì danh sách Works nhỏ và đã nằm trong bộ nhớ. Rust `core/matching::fold_char` vẫn là nguồn đúng cho tìm toàn văn. Hàm TS chỉ phục vụ lọc tên và nhãn, không dùng cho dữ liệu lưu.

## Verification

**Commands:**
- `npx vitest run tests/frontend/palette.test.ts tests/frontend/overlayCaretHold.test.ts tests/frontend/dialogAccessibleName.test.ts` -- expected: xanh, không có dòng Errors
- `npm run check:commands && npm run check:i18n && npm run check:tokens` -- expected: xanh
- Chạy đủ bộ một lần vì `main.ts` nối lệnh: theo AGENTS.md §Tests, việc đăng ký command là dây chung

**Manual checks (if no CLI):**
- Trong app thật: `Mod+K` từ ô dịch, gõ không dấu, Enter mở Tác phẩm, Esc trả con trỏ về đúng ô. Đây là mục nợ `Chủ: Ice`, cho lượt dùng thật.

## Implementation Notes

## Plan Change Log

## Review Triage Log
