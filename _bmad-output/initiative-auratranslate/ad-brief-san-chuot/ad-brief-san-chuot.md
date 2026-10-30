---
type: ad-brief
title: "Hồ sơ bàn giao cho Winston — một `AD` mới: **sàn chuột, mọi command gọi được bằng phím phải có một đường bấm chuột**"
status: draft
created: 2026-10-30
skill: bmad-architecture
---
# Hồ sơ bàn giao cho Winston — sàn chuột cho `CommandRegistry`

**Ngày:** 2026-10-30 · **Người bàn giao:** Amelia · **Người nhận:** Winston (architect)
**Quyết định của Ice (2026-10-30):** yêu cầu *"ngoài dùng phím tắt thì phải có thể sử dụng với chuột, nghĩa là phải có đường click"*. Ice chọn **hướng 2 trước**: viết AD và dựng cổng, rồi các story vá giao diện đi theo cổng đỏ. Phương án bị hoãn: vá giao diện từng bề mặt mà không có cổng (lệnh thêm sau lại có thể thiếu nút mà không ai biết).
**Baseline:** `44573b6e` · spine dừng ở AD-61, mọi `ad-brief-*` khác đã `done` ⇒ số kế tiếp **AD-62**.

## 1. Vì sao cần một AD

AD-34 §1 giữ **một chiều**: handler chuột chỉ được `dispatch` một command đã đăng ký, nên *"thao tác nào chưa gán được phím"* liệt kê được bằng máy (`unbound()`), và NFR17 là sàn **bàn phím**. Chiều ngược lại, *"command nào chưa bấm được bằng chuột"*, không AD nào giữ, không cổng nào canh, PRD không ghi (grep `prd-auratranslate/` không thấy). Thêm chiều đó là đổi bất biến của AD-34, nên phải qua một AD.

## 2. Đo (baseline trên, rà tĩnh, chưa thử trong app thật)

Nạp bộ command thật bằng `installCommands({ setMode: () => {}, isMac: true })` dưới Node, cùng đường `scripts/check-commands.mjs` Kiểm E dùng. "Điểm chuột" = một thuộc tính `@click` / `@dblclick` / `@submit` / `@contextmenu` (kèm modifier) có giá trị đúng là `dispatch('<id>')` literal trong `src/**/*.vue`.

| Phạm vi: 329 command đã đăng ký | Số |
|---|---|
| Có ít nhất một điểm chuột | 259 |
| **Không có điểm chuột** | **70** |
| ↳ có hợp âm mặc định | 32 |
| ↳ không hợp âm mặc định, gọi từ `@keydown`/hàm trong thành phần | 29 |
| ↳ không hợp âm mặc định, **không chỗ nào gọi** (chỉ tới được nếu người dùng tự gán phím) | 9 |

**32 command có hợp âm**, phân loại bằng tay:
- **Không đường chuột nào (15):** `editor.confirm_segment` · `review.open` · `review.diff_next` · `review.diff_prev` · `history.open` · `editor.merge_segments` · `editor.split_segment` · `editor.omit_segment` · `editor.restore_segment` · `editor.end_target_paragraph` · `editor.join_target_paragraph` · `editor.next_untranslated` · `glossary.add_term` · `segment.selection.extend_down` · `segment.selection.extend_up`.
- **Đường chuột tự nhiên, không cần nút (12):** dời focus (`focus.next_panel` · `focus.prev_panel` · `library.focus_search` · `selection.focus_source` · `glossary.confirm.focus` · `tm.fuzzy.focus`): bấm vào đích; bôi đen (`selection.extend_left/right/word_left/word_right`): kéo chuột; tự chạy (`lookup.lookup_selection` khi thả chuột, `src/main.ts:1302`; `library.list_works` khi vào Thư viện, `src/modes/LibraryMode.vue:154`).
- **Đường vòng (5):** `editor.next_chapter` · `editor.prev_chapter` (Thư viện → `library.open_chapter`) · `editor.split_chapter` (Thư viện → `library.chapter_split.open`, ngữ nghĩa khác: theo lớp phủ, không theo caret) · `glossary.settings.open` (Công cụ → Cài đặt → Glossary) · `editor.clear_source_cuts` (bấm lại từng điểm; điểm cắt đặt bằng `⌘`+bấm, `src/panels/GridPanel.vue:947`).

**9 command không ai gọi:** `editor.next_segment` · `editor.prev_segment` · `layout.toggle_ai_translation` · `layout.toggle_grid` · `layout.toggle_lookup` · `library.chapter_next` · `library.chapter_prev` · `library.list_chapters` · `shortcuts.open`.

**29 command gọi từ `@keydown`**, nặng nhất: tầng khối của nhập URL (`import.preview.block_*`, `jump_to_cleanup_rules`, `chapter_next/prev`, `src/ImportPreviewOverlay.vue:625-711`; các `<li class="ip-block">` không handler chuột) và cắt cặp ở nhập song ngữ (`import.preview.bilingual_caret_left/right` · `bilingual_toggle_cut` · `bilingual_next/previous_mismatch`, `src/BilingualImportPreviewOverlay.vue:235`). Phần còn lại là next/prev của danh sách đã nhắm được bằng `@mousedown` (Harvest, TM fuzzy, Alignment, Thư viện, dấu đọc), `Esc` của bề mặt có nút Đóng, và hai ca đi qua hàm (`tm.manage.edit` qua `@submit.prevent="onRowEdit(i)"`, `src/TmManageOverlay.vue:723`; `prompt.import.open` qua `@submit.prevent="onOpenImport"`, `src/PromptLibraryOverlay.vue:763`): có đường chuột thật mà bộ đọc literal không thấy.

Danh sách thô của lượt đo: `scratchpad/bound.tsv`, `hits.txt`, `nopointer.txt` (thư mục tạm của phiên 2026-10-30, không giữ). Đo lại: chạy lại phép nạp `installCommands` ở trên rồi đối chiếu với điểm chuột.

## 3. Câu hỏi cho Winston (mỗi câu có hai nhánh, Ice chọn)

**Q1. Phạm vi.** (a) chỉ command có hợp âm mặc định: 32 đỏ, bỏ lọt tầng khối nhập URL và cắt song ngữ vì chúng không có hợp âm toàn cục. (b) mọi command đã đăng ký: 70 đỏ, bắt cả 29 ca `@keydown` và 9 ca không ai gọi.

**Q2. Thế nào là một điểm chuột.** (a) chỉ `dispatch('<id>')` literal trong bốn thuộc tính trên: hai ca qua hàm phải viết lại cho literal hoặc vào miễn trừ. (b) thêm `@mousedown`/`@pointerdown` và lần theo một bậc hàm: bắt được hai ca kia, nhưng `@mousedown` ở kho này đang là cửa **nhắm/đặt caret**, không phải thao tác (`src/panels/GridPanel.vue:962-975`, đầu tệp `src/modes/LibraryWorkPage.vue:2`), nên nhánh này làm mờ ranh giới mà Kiểm A dựng.

**Q3. Miễn trừ đặt ở đâu.** (a) một trường trên `CommandSpec` (`src/commands/registry.ts:45`), khai cạnh lượt `register()` trong `src/commands/index.ts`, mang lý do, chết cùng command. `index.ts` phải nạp được bằng Node thuần (erasable-only), một trường kiểu thường thì vẫn được. (b) một danh sách id + lý do trong `scripts/check-commands.mjs`. Ràng buộc chung: mỗi miễn trừ có tên, có lý do, chết được (`AGENTS.md` §Known pitfalls). Cổng **không** kiểm được lý do *"đường chuột tự nhiên"* là thật; một lý do sai sẽ đứng mãi.

**Q4. Đặt cổng ở đâu.** (a) một Kiểm mới trong `check:commands` (đã có A–L): 0 thay đổi ở ba danh sách cổng, `check:gates` không đổi, dùng lại bộ đọc thuộc tính của Kiểm A (`scripts/check-commands.mjs:449-533`). (b) một cổng riêng: phải sửa `package.json` · `.github/workflows/ci.yml` · `.githooks/pre-push`, đổi lại tách được trách nhiệm.

**Q5. Đỏ lúc hạ cổng.** (a) hạ cổng cùng một danh sách **nợ** tách khỏi danh sách miễn trừ, mỗi mục có `Chủ:` trỏ một story trong cây vé, rút dần theo từng story vá. (b) chỉ hạ cổng khi các story vá đã xanh hết. `AGENTS.md` cấm *"mở rộng danh sách loại trừ để gỡ đỏ"*, nên nếu chọn (a) thì danh sách nợ phải là thứ khác hẳn miễn trừ, và Kiểm C của `check:debt-owner` phải đọc được chủ của nó.

## 4. Phần không cần AD

- Đặt nút/menu ở đâu cho 15 + N command là việc của Sally (UX), rồi các story vá theo bề mặt: lưới Editor · mở Rà soát/Lịch sử · thêm thuật ngữ · lớp phủ nhập.
- Lớp phủ và popup đã đạt: 25 lớp phủ đều có nút Đóng/Huỷ; `DropdownMenu`, `TitlebarToolsMenu`, `LookupSourcesMenu`, `ReadingLookupPopup` đều đóng khi bấm ra ngoài.

## 5. Đã chốt và còn lại cho Winston

Ice chọn (2026-10-30):
- **Q1:** nhánh (b) **có giới hạn**: mọi command đã đăng ký, nhưng chỉ những command là **một chức năng độc lập mà người dùng cần dùng**. Winston định nghĩa tiêu chí đó sao cho máy đọc được, và xếp các nhóm ở §2 theo nó (dời focus, bôi đen, bước next/prev trong danh sách đã nhắm được bằng chuột, 9 command không ai gọi).
- **Q2:** nhánh (a): chỉ `dispatch('<id>')` literal trong `@click` / `@dblclick` / `@submit` / `@contextmenu`.
- **Q3, Q4:** Winston tự phân tích và đề xuất, kèm số đo cho từng nhánh.
- **Q5:** nhánh (a): hạ cổng cùng một sổ nợ có chủ, rút dần theo từng story vá.

Winston viết AD-62 với các câu trên; một dòng tham chiếu chéo ở AD-34; câu cập nhật cho `src/commands/README.md` §*Thêm một command* và một dòng luật trong `src/AGENTS.md` §Conventions.
