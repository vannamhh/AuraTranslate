---
title: 'Thể loại chọn từ danh sách, tự chọn bộ prompt cùng tên'
type: 'feature'
ticket: ''
created: '2026-10-25'
status: 'built'
baseline_revision: '5aff0ff4fe17e01e7e9033428908c7cddc33d77a'
route: 'full'
route_source: 'auto'
risk: 'medium'
review: 'quick'
review_source: 'pinned'
lenses_ran: ['quick']
review_loop_iteration: 0
context: []
---

<frozen-after-approval reason="human-owned intent — do not modify unless human renegotiates">

## Intent

**Problem:** Thể loại là ô gõ tự do ở màn nhập, và giá trị đó không tới AI ở đâu cả: bộ prompt dịch luôn phải chọn tay.

**Approach:** Ô Thể loại thành `SelectMenu variant="field"` trên 20 thể loại cố định + "Chưa chọn"; lưu định danh, nhãn ở `vi.json` (khuôn `sourceLangChoices`/`sourceLangLabel`, AD-21 · NFR16). Khi một Tác phẩm được tạo hoặc mở, webview tự chọn bộ prompt (FR69) có tên trùng nhãn thể loại. Không có bộ trùng: bỏ bộ do lần tự chọn trước đặt, giữ bộ Ice chọn tay. Đổi thể loại Tác phẩm có sẵn là plan kế tiếp (`deferred-work.md`, Chủ: Ice); hàm tự chọn phải gọi lại được từ đó.

Danh sách (định danh → nhãn): `xianxia` Tiên hiệp · `xuanhuan` Huyền huyễn · `wuxia` Kiếm hiệp · `urban` Đô thị · `romance` Ngôn tình · `danmei` Đam mỹ · `historical` Lịch sử – Cổ đại · `scifi` Khoa huyễn · `game` Võng du – Game · `horror` Linh dị – Kinh dị · `mystery` Trinh thám · `fanfic` Đồng nhân · `literary` Văn học · `children` Thiếu nhi · `nonfiction` Phi hư cấu · `science` Khoa học – Kỹ thuật · `legal` Pháp lý · `news` Báo chí · `business` Kinh tế – Kinh doanh · `other` Khác.

## Boundaries & Constraints

**Always:** Một `genre` ngoài danh sách (fixture e2e `'Huyền huyễn'`, `'general'`) hiện nguyên văn ở thẻ, chip lọc và trang Tác phẩm. "Chưa chọn" lưu chuỗi rỗng. Tự chọn bộ prompt là trạng thái webview thuần (spec 4.4 Quyết định #4).

**Never:** Không đổi tệp nào dưới `src-tauri/`. Không thêm biến `{{genre}}`. Không tự tạo bộ prompt.

## I/O & Edge-Case Matrix

| Scenario | Input / State | Expected Output / Behavior | Error Handling |
|----------|--------------|---------------------------|----------------|
| Tạo có thể loại | Chọn "Tiên hiệp" ở màn nhập | `genre = 'xianxia'`; thẻ "Tiên hiệp · …" | — |
| Giá trị lạ | `genre = 'Huyền huyễn'` | Thẻ/chip/trang hiện "Huyền huyễn" | — |
| Khớp bộ | Mở Tác phẩm `xianxia`, có bộ tên " tiên hiệp" | Bộ đó thành bộ đang dùng (trim, không phân biệt hoa thường) | Tải danh sách bộ lỗi ⇒ giữ nguyên lựa chọn |
| Không khớp, bộ tự chọn | Bộ đang dùng do tự chọn đặt | Bỏ chọn | — |
| Không khớp, bộ chọn tay | Ice đã chọn tay | Giữ nguyên | — |
| Chọn tay sau tự chọn | Ice chọn tay một bộ | Bộ đó không còn bị coi là do tự chọn | — |

</frozen-after-approval>

## Code Map

- `src/modes/libraryWorkCard.ts:13-25,41-47` -- chép khuôn `sourceLangChoices`/`sourceLangLabel`: `genreChoices()`, `genreLabel(raw)` (rỗng ⇒ `null`, lạ ⇒ nguyên văn); `workCardMeta` dùng `genreLabel` (trang Tác phẩm đọc `subtitle` từ đây, `LibraryWorkPage.vue:94`).
- `src/LibraryImportOverlay.vue:200-205` -- `<input v-model="genre">` thành `SelectMenu` như ô ngôn ngữ nguồn ngay trên (dòng 183-199); nhánh đích có sẵn hiện `genreLabel(...) ?? ''`.
- `src/modes/LibraryMode.vue:256-259,508` -- chip "Lĩnh vực": `label` và `trigger-text` qua `genreLabel`, `value` giữ thô (Rust lọc khớp thô).
- `src/modes/libraryImport.ts:111` -- `genre` ref giữ nguyên.
- `src/promptSetState.ts:140-172` -- thêm hàm tự chọn theo nhãn thể loại và cờ "bộ đang dùng do tự chọn đặt"; `setSelectedPromptSetName` (chỗ chọn tay: `AiTranslationPanel.vue:147`, `PromptLibraryOverlay.vue:581`) xoá cờ. `loadPromptSets` đã tự bỏ tên không còn giải được.
- `src/modes/libraryChapters.ts:360`, `src/modes/libraryImport.ts:443` -- hai chỗ duy nhất gọi `setOpenWorkId`, đều có `meta.genre`: gọi hàm tự chọn ngay sau, không chờ (`void`).
- `src/i18n/vi.json:829` -- 21 khoá `mode.library.genre_*`.
- Không đụng: `src-tauri/`, `PromptLibraryOverlay.vue`, `AiTranslationPanel.vue`.
- Dữ liệu thật: 47/47 Tác phẩm `genre` rỗng, `global.db` có 0 bộ prompt.

## Tasks & Acceptance

**Execution:**
- [ ] `src/i18n/vi.json`, `src/modes/libraryWorkCard.ts` -- danh sách, nhãn, `workCardMeta`.
- [ ] `src/LibraryImportOverlay.vue`, `src/modes/LibraryMode.vue` -- ô chọn, nhãn chip lọc.
- [ ] `src/promptSetState.ts`, `src/modes/libraryChapters.ts`, `src/modes/libraryImport.ts` -- tự chọn bộ prompt.
- [ ] `tests/frontend/libraryWorkCard.test.ts`, `tests/frontend/libraryImportOverlay.test.ts`, `tests/frontend/promptSetState.test.ts` -- phủ mọi hàng Matrix.

**Acceptance Criteria:**
- Given màn nhập, when mở ô Thể loại, then thấy "Chưa chọn" và 20 thể loại theo thứ tự trên, điều khiển bằng bàn phím được như ô ngôn ngữ nguồn.
- Given có bộ prompt "Tiên hiệp", when mở một Tác phẩm `xianxia`, then panel AI Translation hiện bộ "Tiên hiệp" là bộ đang dùng.

## Implementation Notes

- `genre` không đi vào prompt AI ở đâu (`commands/aitranslate.rs`, `core/ai/rag.rs`); Ice chọn nối bằng tự chọn bộ prompt cùng tên.
- Tự chọn đi qua `pendingGenre` mà lượt `loadPromptSets` hoàn tất SAU CÙNG áp dụng: panel AI Translation nạp danh sách khi mount, và lượt nạp của tự chọn bị nó vượt mặt thì trả về sớm với danh sách cũ.
- Ca "lượt nạp mới vượt mặt" ban đầu xanh cả trên mã cũ (panel xong trước khi tự chọn quay lại); đã đổi thứ tự cho lượt tự chọn trả về trước, nay đỏ trên mã cũ và xanh trên mã mới. Hai chỗ gọi đối chứng bằng cách gỡ dòng gọi thật.
- Thêm `genre: ''` vào fixture `WorkMeta` của tám tệp test: thiếu nó thì `void autoSelectPromptSetForGenre` văng lỗi không ai bắt.
- Trọn `tests/frontend` ba lượt khi tải máy 130–275: lần lượt 1 ca đỏ cộng 2 unhandled rejection, rồi 3 ca đỏ, rồi xanh trọn 3148/3148. Các ca đỏ (`settingsFrame`, `librarySearchScope`) chạy riêng hai lần đều xanh; không còn fixture nào thiếu `genre` trên đường mở/tạo Tác phẩm.

## Verification

**Commands:**
- `npx vitest run tests/frontend/libraryWorkCard.test.ts tests/frontend/libraryImportOverlay.test.ts tests/frontend/promptSetState.test.ts` -- xanh.
- `npm run check:i18n` và `npx vue-tsc --noEmit` (hoặc lệnh typecheck trong `package.json`) -- xanh.

**Manual checks (if no CLI):**
- Nợ `Chủ: Ice`: app thật — tạo bộ prompt "Tiên hiệp", nhập Tác phẩm chọn Tiên hiệp, xem panel AI Translation.

## Review Triage Log

Quick, vòng 1: medium 2 · low 2 · false 1.

- medium · patch — `autoSelectPromptSetForGenre` đọc `resolvedSets`/`loadError` cũ khi một lượt `loadPromptSets` mới hơn vượt mặt (`if (mine !== sequence) return`); xác nhận ở `promptSetState.ts:146`, `AiTranslationPanel.vue:117` nạp khi mount.
- medium · patch — không ca nào chạm hai chỗ gọi ở `openWorkById`/`finishImportSubmission`; gỡ dòng gọi thì mọi test vẫn xanh.
- low · patch — nhãn chip "Lĩnh vực" qua `genreLabel` chưa có ca; hàng Matrix "Giá trị lạ" thiếu phần chip.
- low · patch — JSDoc mới tiếng Việt, ba dòng, kể lại thân hàm (AGENTS.md §Code comments).
- false — "giá trị tự do cũ trùng tên bộ cũng tự chọn": `genreLabel` trả nguyên văn giá trị lạ, nên tự chọn theo đúng "tên trùng nhãn thể loại" của Intent.
