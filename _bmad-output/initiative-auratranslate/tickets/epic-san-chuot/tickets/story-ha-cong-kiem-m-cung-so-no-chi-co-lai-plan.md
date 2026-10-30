---
title: 'Hạ cổng Kiểm M cùng sổ nợ chỉ co lại'
type: 'feature'
ticket: '1'
created: '2026-10-30'
status: 'built'
baseline_revision: '9d7f9a0ddd6c8bce73cb88e131f705698371cbd7'
route: 'full'
route_source: 'auto'
risk: 'high'
review: 'quick'
review_source: 'pinned'
lenses_ran: ['quick']
review_loop_iteration: 0
context: []
---

<frozen-after-approval reason="human-owned intent — do not modify unless human renegotiates">

## Intent

**Problem:** 70 trong 329 command không có điểm chuột và không gì chặn command mới thiếu nút; AD-62 biến sàn chuột thành bất biến nhưng cổng chưa tồn tại.

**Approach:** Thêm `pointer` vào `CommandSpec`, khai 27 id đạt điều kiện máy đọc, dựng Kiểm M trong `check:commands` với `POINTER_DEBT` 44 id chỉ co lại, rồi ghi chủ nợ theo bề mặt. Không thêm nút nào.

## Boundaries & Constraints

**Decision (Ice 2026-10-30):** `library.list_works` khai `via: mode.library`, `library.list_chapters` khai `via: library.open_work`; không đổi lời gọi thẳng ở `LibraryMode.vue`. Ice nhận khoảng hở: bấm tab Thư viện khi đang ở Thư viện không nạp lại.

**Decision (Ice 2026-10-30):** nhận 258 điểm chuột, 27 khai `pointer`, 44 nợ; `prompt.library.close` giao Story 14.9. Story này sửa `tickets.toml` entry 9 và thêm dòng 🔵 ở AD-62 mục 7 và file epic, ngoại lệ với luật build không ghi vào tệp vé.

**Always:** Đúng AD-62 mục 1–5. Kiểm M đọc `commandRegistry.list()` thật và `scanVueAttrs` trên `parsed` sẵn có, cùng hàm phán quyết cho cây thật và self-check. `registry.ts` vẫn không `import` gì và không `enum`. `kind` nằm trong tập đóng năm loại. `POINTER_DEBT_BASELINE` là đúng 44 id đỏ lúc hạ cổng. Sáu id ở lại nợ dù đích cuối là `pointer` theo mô tả vé: `segment.selection.extend_down/up`, `import.preview.block_next/prev`, `import.preview.bilingual_caret_left/right`.

**Never:** Thêm nút hoặc đổi hành vi app. Thêm cổng thứ mười lăm, sửa ba danh sách cổng hay `check:gates`. Đổi Kiểm A, AD-34 §1, `unbound()`, màn hình phím tắt. Sửa `src/commands/index.ts:1440` (Story 14.8). Đọc `_bmad-output` từ cổng. Đổi nội dung AD-62 mục 7 ngoài dòng 🔵 Ice đã duyệt: con số ở đó là đề xuất, story này xếp lại.

## I/O & Edge-Case Matrix

| Scenario | Input / State | Expected Output / Behavior | Error Handling |
|----------|--------------|---------------------------|----------------|
| Có điểm chuột | `@contextmenu.prevent="dispatch('x.y')"` | Tính là điểm chuột | Không lỗi |
| Không phải điểm chuột | `@mousedown="dispatch('x.y')"`, `@submit="fn()"` | Không tính | Id đó đỏ nếu không thuộc nhóm khác |
| Hai nhóm | Có điểm chuột và khai `pointer`, hoặc còn trong nợ | Đỏ đúng id | `fail()` |
| Không nhóm nào | Command mới thiếu cả ba | Đỏ đúng id | `fail()` |
| Khai sai | `kind` lạ, `reason` rỗng, `at`/`via` thiếu hoặc thừa | Đỏ đúng id | `fail()` |
| `aim` | Tệp `at` không có `@mousedown`/`@pointerdown` | Đỏ đúng id | `fail()` |
| `auto` | `at` thuộc `src/commands/**`, hoặc literal chỉ ở template/comment | Đỏ đúng id | `fail()` |
| `via` | Đích chưa đăng ký, không có điểm chuột, hoặc tự khai `pointer` | Đỏ đúng id | `fail()` |
| Sổ nợ | Id trùng, ngoài baseline, hoặc không còn đăng ký | Đỏ đúng id | `fail()` |
| Cây rỗng | Số command có điểm chuột dưới sàn | Không xanh | `abort()` qua `judgeFloor` |

</frozen-after-approval>

## Code Map

- `src/commands/registry.ts` -- `CommandSpec`, `frozen()`.
- `src/commands/index.ts` -- `register()`. Các id `focus.*`, `reading.marks_*`, `reading.lookup_close`, `selection.extend_*`, `shortcuts.open` nằm trong vòng tuple; `reading.marks_next/prev` ở chung vòng 10 hàng nên tách vòng riêng, các vòng khác thêm cột hoặc khai trong thân.
- `scripts/check-commands.mjs` -- Kiểm A (`DISPATCH_ONLY_RE`), Kiểm C (`expectEq`), Kiểm E (`registered`), Kiểm K (`selfCheckHandlerInventory` là khuôn self-check), `COMMAND_FLOOR`, `abort()`.
- `scripts/lib/commands-scan.mjs` -- `scanVueAttrs`; mô-đun thuần, test ở `tests/frontend/checkCommandsScan.test.ts`.
- `scripts/lib/floor-judge.mjs` -- `judgeFloor`, không sửa.
- ⚠️ `ParsedFile.code` đã xoá chuỗi literal nên không dò được `dispatch('<id>')`. Dùng `masked` rồi che `templates` (comment đã che).
- `src/commands/README.md` §*Thêm một command*, §*Chạy cổng* (dòng "5 phép kiểm"); `src/AGENTS.md` §Conventions.
- `_bmad-output/initiative-auratranslate/deferred-work.md` -- quá 1 MB, chỉ nối cuối, mỗi mục tối đa 5 dòng.
- Spine §AD-62 mục 3–6: định nghĩa năm loại và phép gỡ (a)–(h).

## Tasks & Acceptance

**Execution:**
- [x] `src/commands/registry.ts` -- thêm `CommandPointer` (union theo `kind`) và `pointer?`; `frozen()` chép và đóng băng sâu -- trường mới không rơi ở cửa vào.
- [x] `src/commands/index.ts` -- khai 27 id theo bảng Design Notes, sửa comment "hai nút thật" của `library.chapter_next` -- comment đó sai và nằm ngay trên khai báo mới.
- [x] `scripts/lib/commands-scan.mjs` -- thêm `mousePointIds()` và `judgePointerFloor()` thuần, chuyển `DISPATCH_ONLY_RE` về đây -- một biểu thức cho Kiểm A và Kiểm M.
- [x] `scripts/check-commands.mjs` -- Kiểm C khẳng định `pointer` còn nguyên; dựng Kiểm M, `POINTER_DEBT`, `POINTER_DEBT_BASELINE`, `POINTER_FLOOR = 220`, self-check (h) -- R1, R2, R4, R5, R6.
- [x] `tests/frontend/checkCommandsScan.test.ts` -- ca cho `mousePointIds`: modifier, `v-on:`, `@mousedown`, `@submit="fn()"`.
- [x] `src/commands/README.md`, `src/AGENTS.md` -- mỗi chỗ một dòng luật, sửa dòng "5 phép kiểm" -- R10.
- [x] `deferred-work.md` -- tám mục `Chủ: Story 14.2`…`14.9`, một mục `Chủ: Epic 14` cho lượt dùng thật, một mục `Chủ: B7` cho focus của nút trên WebView2.
- [x] Đối chứng (a)–(g) -- gỡ thật từng phép, chạy `node scripts/check-commands.mjs`, đọc đúng id đỏ, trả lại tệp từ bản sao ở scratchpad (không `git checkout`, không `git stash`).

**Acceptance Criteria:**
- Given cây thật sau story, when chạy `npm run check:commands`, then Kiểm M xanh với 258 điểm chuột, 27 khai `pointer`, 44 nợ.
- Given mỗi phép gỡ (a)–(g), when chạy cổng, then Kiểm M đỏ đúng id AD-62 mục 6 nêu và vì đúng lý do đó.
- Given các nhánh (h), when cổng chạy self-check, then mọi ca đúng phán quyết, và một ca sai làm cổng đỏ.
- Given `deferred-work.md` đã thêm mục, when chạy `npm run check:debt-owner`, then xanh.

## Implementation Notes

- Số thật là 258/27/44, không phải 259/27/43: số cũ đếm nhầm comment ở `src/PromptLibraryOverlay.vue:18`, phép đo regex thô của plan cũng mắc lỗi đó. Ice nhận 44 và giao `prompt.library.close` cho Story 14.9.
- `prompt.library.close` không viết literal thẳng được: `onClose` ghi nội dung prompt rồi mới dispatch và từ chối đóng khi ghi lỗi, đã ghi vào vé 14.9.
- `judgePointerFloor` và `mousePointIds` ở `scripts/lib/commands-scan.mjs`, Kiểm M đứng sau Kiểm K; `DISPATCH_ONLY_RE` dùng chung với Kiểm A.
- Tách vòng `reading.marks_*` từng đổi thứ tự trên màn hình phím tắt; đã trả lại, `list()` khớp baseline 329/329 id đúng thứ tự.
- Đối chứng (a)–(g) chạy lại trên bản đã vá, đỏ đúng: (a) `mode.reading`; (b) `library.open_work` và `library.list_chapters`; (c) `history.open`; (d) `settings.open` và `shortcuts.open`; (e) `lookup.lookup_selection`; (f) `glossary.harvest.next/prev`; (g) `ai.proofread.hide`.
- Thêm đối chứng ngoài danh sách: `POINTER_FLOOR` đặt 300 thì `abort()`; đọc `p.text` thay `p.masked` thì ca comment của self-check đỏ.
- `check:gates` Kiểm G đỏ từ `9d7f9a0d` (`LINE_HEIGHT_FLOOR_EXEMPT`) và `check:doc-refs` đỏ vì `FILE_FLOOR` 660 trên 828 tệp ở HEAD, không do diff này; cả hai đã ghi nợ `Chủ: Ice`.
- `npm run build` còn soát `scripts/` qua `tsconfig.node.json`; `vue-tsc --noEmit` mặc định không thấy, nên phần Verification của plan từng thiếu một đường.

## Plan Change Log

## Review Triage Log

Lượt 1 (lens Quick cộng soát của người điều phối): high 0 · medium 8 · low 7 · false 0 · maybe-false 0.

- medium, Ice quyết: số khoá 259/27/43 sai so với cổng, sửa thành 258/27/44.
- medium, Ice quyết: vé 14.9 thiếu id thứ tám, đã sửa `tickets.toml` entry 9.
- medium, patch: mục nợ 14.9 không nêu bẫy flush của `onClose`, đã sửa mục nợ và vé.
- medium, patch: tách vòng `reading.marks_*` đổi thứ tự màn hình phím tắt, đã trả lại.
- medium, patch: ca ⑧ `src/commands/**` đỏ vì `fileAt` null, đã thêm fixture thật.
- medium, patch: ca ⑥ `at` ngoài `src/` cùng lỗi, đã thêm fixture thật.
- medium, patch: phép kiểm `fn` không thể đỏ, đã đổi sang `fake.sub` và `fake.more`.
- medium, defer: `shortcuts.open` cùng hình dạng `glossary.settings.open` nhưng `via`; AD-62 mục 3 nêu đích danh nên giữ, ghi nợ `Chủ: Story 14.5`.
- low, patch: dòng luật README sai mục, đã chuyển về §*Thêm một command*.
- low, patch: danh sách phép kiểm ở README thiếu F, I, L.
- low, patch: comment mới bằng tiếng Việt, đã đổi sang tiếng Anh hoặc xoá.
- low, patch: Kiểm M nằm giữa Kiểm E, đã chuyển ra sau Kiểm K.
- low, patch: `dispatchesInCode` tách theo code point lệch offset UTF-16, đã dùng `split('')`.
- low, patch: `CommandPointerKind` không ai dùng, đã xoá.
- low, patch: plan chưa ghi kết quả đối chứng, đã ghi ở Implementation Notes sau khi chạy lại.

## Design Notes

| Loại | Id | `at` / `via` |
|---|---|---|
| `focus` (8) | `focus.next_panel`, `focus.prev_panel`, `library.focus_search`, `selection.focus_source`, `editor.next_segment`, `editor.prev_segment`, `glossary.confirm.focus`, `tm.fuzzy.focus` | |
| `aim` (10) | `library.work_next/prev` | `src/modes/LibraryMode.vue` |
| | `library.chapter_next/prev` | `src/modes/LibraryWorkPage.vue` |
| | `reading.marks_next/prev` | `src/modes/ReadingMode.vue` |
| | `tm.fuzzy.next/prev` | `src/TmFuzzyStrip.vue` |
| | `glossary.harvest.next/prev` | `src/GlossaryHarvestOverlay.vue` |
| `gesture` (4) | `selection.extend_left/right/word_left/word_right` | |
| `auto` (2) | `lookup.lookup_selection` | `src/main.ts` |
| | `reading.lookup_close` | `src/modes/ReadingLookupPopup.vue` |
| `via` (3) | `shortcuts.open`, `library.list_works`, `library.list_chapters` | `settings.open`, `mode.library`, `library.open_work` |

Mọi `via` đã kiểm có điểm chuột; `reason` của `auto` nêu sự kiện chuột.

⚠️ Story 13.3 có thay đổi trạng thái đã staged, không thuộc story này: mọi commit ghi đúng đường dẫn của story, không `git add -A`.

44 nợ theo chủ: 14.2 (10) · 14.3 (4) · 14.4 (4) · 14.5 (2) · 14.6 (8) · 14.7 (5) · 14.8 (3) · 14.9 (8), lấy từ `tickets.toml`.

## Verification

**Commands:**
- `npm run check:commands` -- exit 0, in ba số 258/27/44.
- `npm run check:debt-owner` -- exit 0.
- `npx vitest run tests/frontend/checkCommandsScan.test.ts tests/frontend/commandsRegistry.test.ts` -- exit 0.
- `npx vue-tsc --noEmit`, `npx vue-tsc --noEmit -p tsconfig.node.json` và `npm run check:lint` -- exit 0.
- Chạy đủ bộ một lần: story chạm `scripts/check-*` (AGENTS.md §Tests).
