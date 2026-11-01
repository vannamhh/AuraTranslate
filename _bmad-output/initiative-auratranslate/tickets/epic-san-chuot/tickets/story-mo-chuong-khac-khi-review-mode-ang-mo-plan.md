---
title: 'Mở Chương khác khi Review Mode đang mở'
type: 'bugfix'
ticket: '12'
created: '2026-11-01'
status: 'built'
baseline_revision: 'f48b442016cc979e58dc32075b22bea3202e6b95'
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

**Problem:** Từ Thư viện hoặc Chế độ đọc, `openChapterById` đổi Chương của Editor trong khi Review Mode vẫn giữ Chương cũ, nên lượt chấp nhận ghi vào Chương cũ và đóng Review Mode thì người dùng đứng ở Chương khác lúc vào.

**Approach:** `openChapterById` báo một lần khi Chương đã mở xong, `reviewModeState.ts` nghe và đóng phiên nếu đó là Chương khác, nên cả 11 đường gọi có cùng kết quả; lượt mở bị chặn thì phiên còn nguyên và lý do hiện ở cả Library.

## Boundaries & Constraints

**Decision (Ice chọn 2026-11-01, epic Notes):** đóng phiên khi Chương khác đã mở xong, không chặn, không chuyển phiên theo Chương. Mở đúng Chương đang rà soát thì giữ nguyên. Đổi Tác phẩm đã qua `resetReviewMode`, không thuộc story này.

**Decision (repo chứng minh, không cần ký):** `onChapterOpened` là export của `editorPanelState.ts`, cùng khuôn `onAlignmentClosed` nằm ngay ở module phát. `reviewModeState.ts` đã import `editorPanelState.ts`, nên không có vòng import và không thêm tệp. Listener đăng ký ở cấp module cần export này tồn tại dưới `vi.mock`: 1 trong 12 tệp mock trần (`titlebarProofreadStatus.test.ts`) tới được `reviewModeState.ts`, thêm export vào mock đó nếu nó đỏ.

**Decision (repo chứng minh, không cần ký):** Library hiện đúng ba câu `chapter-flush-failed`, `chapter-still-dirty`, `chapter-switch-failed`, dùng lại khoá đã có. `StatusBar.vue` nhánh `library` hiện không đọc `editorNavNotice`, nên lượt mở bị chặn ở Library im lặng, trái với comment trên `openCurrentChapter`. Các câu điều hướng khác không hiện ở Library vì chúng trả lời một phím của Workspace.

**Always:** Phiên gồm mọi trạng thái khác `idle`: `loading` hay lời báo của Chương cũ (`not_imported`, `stale`, `no_chapter`, `error`, `diff_failed`) cũng đóng khi Chương khác mở xong, không thì dải của Chương cũ hiện ở Chương mới. Báo bằng `targetChapterId`, cả khi `ensureSegmentsLoaded` lỗi, vì Rust đã đổi Chương. Báo trước `await nextTick()` để Workspace hết `inert` kịp cho `enterFocus('panel.grid')`.

**Never:** Đổi `closeReviewMode`, `refuseChapterMoveInReview`, phím hay thứ tự flush của `openChapterById`. `editorPanelState.ts` import `reviewModeState.ts`. Khoá tab chế độ. Sửa từng nơi gọi trong 11 đường. Thêm chuỗi i18n. Ghi `tickets.toml`.

## I/O & Edge-Case Matrix

| Scenario | Input / State | Expected Output / Behavior | Error Handling |
|----------|--------------|---------------------------|----------------|
| Mở Chương khác từ Library | Review ở A, mở B | Workspace hiện B, dải Rà soát đóng, tiêu điểm ở gốc Workspace như mọi lượt mở Chương từ Library, không rơi về `body` | Không |
| Mở từ Chế độ đọc | Review ở A, đi tới dấu ở B | Dải Rà soát đóng, Editor ở B | Không |
| Mở trong Workspace | Review ở A, lượt đi dấu đọc sang B | Dải đóng, tiêu điểm ở lưới của B | Không |
| Mở lại đúng Chương | Review ở A, mở A | Dải và vị trí diff còn nguyên | Không |
| Flush trượt | Review ở A, flush `failed` hoặc `still-dirty`, mở B từ Library | Phiên còn, người dùng ở Library, câu `nav_chapter_flush_failed` hoặc `nav_chapter_still_dirty` hiện | Câu trên thanh trạng thái |
| Lỗi mở | Review ở A, `open_chapter` lỗi | Phiên còn, câu `nav_chapter_switch_failed` hiện | Câu trên thanh trạng thái |
| Đang nạp | `loading` ở A, mở B xong trước | Không bao giờ thành `open` ở A | Không |
| Lời báo cũ | `not_imported` ở A, mở B | Lời báo biến mất; mở A thì lời báo còn | Không |
| Không có phiên | `idle`, mở B | Không gọi `closeReviewMode` | Không |

</frozen-after-approval>

## Code Map

- `src/panels/editorPanelState.ts`:2210 `openChapterById`; chèn lời báo sau khối `targetSegmentId`:2271, trước `await nextTick()`:2284. `switchChapter`:2025 cùng phát sau `moved`.
- `src/alignmentState.ts`:35–43 -- khuôn `Set` listener của `onAlignmentClosed`. `src/reviewModeState.ts`:456 -- khuôn nghe ở cấp module.
- `src/reviewModeState.ts`:71 `openChapterId`, gán ở :230, xoá ở `clearData`:197; dời phép gán lên ngay sau `status.value = 'loading'`:212. Mọi đường ghi đã canh `status !== 'open'` nên gán sớm an toàn.
- `src/reviewModeState.ts`:248 `closeReviewMode` -- đồng bộ tới `setDockSuspended(false)`, ngoài Workspace thì dừng trước `nextTick`. `sequence` đổi nên lượt accept đang bay không reload.
- `src/StatusBar.vue`:549 nhánh `library`, :555 `.status-live`; `NAV_NOTICE_KEYS`:447, `navNoticeKey`:476. Chế độ đọc đã đi nhánh `v-else` nên đã hiện câu.
- 11 chỗ gọi: `chapterSplitState.ts`:55, `libraryChapters.ts`:492 502, `readingState.ts`:255 267 389 430 447 496 576, `librarySearch.ts`:552.
- `tests/frontend/chapterPosition.test.ts`:192 -- khuôn gọi `openChapterById` thật. `tests/frontend/libraryChapters.test.ts`:24 -- mock flush bằng `importOriginal`. `tests/frontend/statusBarLibrary.test.ts` -- mount nhánh Library.
- `e2e/specs/review-mouse.e2e.mjs`:38 `answerReviewReads`, :130 ca mở bằng ô -- khuôn cho ca e2e.

## Tasks & Acceptance

**Execution:**
- [ ] `src/panels/editorPanelState.ts` -- export `onChapterOpened(listener)` theo khuôn `Set`, phát `targetChapterId` đúng một lần trong `openChapterById` tại vị trí ở Code Map.
- [ ] `src/reviewModeState.ts` -- dời phép gán `openChapterId`; nghe ở cấp module: `idle` hoặc cùng Chương thì thôi, còn lại `void closeReviewMode()`.
- [ ] `src/StatusBar.vue` -- computed chỉ nhận ba giá trị ở Decision, một `v-else-if` sau `libraryErrorText` trong `.status-live` của Library.
- [ ] `tests/frontend/reviewModeChapterOpen.test.ts` (mới) -- mọi hàng I/O Matrix, đi từ `openCurrentChapter` và một đường `readingState.ts` qua `openChapterById` thật tới `reviewModeState.ts` thật, chỉ mock IPC và flush.
- [ ] `tests/frontend/statusBarLibrary.test.ts` -- ba câu hiện ở Library; `at-last` không hiện.
- [ ] `e2e/specs/review-mouse.e2e.mjs` -- ca hai Chương: Review ở A, sang Thư viện, mở B bằng chuột, dải đóng và lưới hiện B; mở lại A khi Review ở A thì dải còn.
- [ ] `deferred-work.md` -- đóng mục `Chủ: Story 14.12` của §Deferred from: Story 14.4; thêm lượt dùng thật `Chủ: Epic 14`.
- [ ] `tickets/epic-san-chuot/ux-san-chuot/EXPERIENCE.md` -- 🔵 ở §State Patterns nếu câu về Review Mode và đổi Chương còn trái với Decision.

**Acceptance Criteria:**
- Given Review Mode ở A và phiên đóng theo I/O Matrix, when bấm `Xác nhận` bằng lệnh `review.accept_change`, then không IPC nào ghi vào A.
- Given ca đóng phiên, when gỡ thật lượt phát hoặc listener, then ca đó đỏ đúng vì dải còn mở.
- Given ca `Lời báo cũ` và `Đang nạp`, when trả phép gán `openChapterId` về chỗ cũ, then ca mở lại đúng Chương ở lời báo cũ đỏ.
- Given ca Library, when gỡ `v-else-if` mới, then ca lý do đỏ.
- Given cây sau story, when chạy `npm run check:commands`, then Kiểm M không đổi số điểm chuột, khai `pointer` và nợ.

## Implementation Notes

- `switchChapter` cũng phát `onChapterOpened`, trái dòng Code Map "không đổi": `refuseChapterMoveInReview` chỉ chặn ở `open`, nên hợp âm và ‹ › vẫn chạy ở `loading` và các lời báo.
- Ice sửa hàng đầu I/O Matrix 2026-11-01: đo trên WKWebView, tiêu điểm sau lượt mở từ Library ở `section.mode`, cả khi không có Review Mode, vì `WorkspaceMode.vue` cố ý vào gốc chế độ. Mở trong Workspace thì tiêu điểm về lưới.
- `vi.resetModules` giữ kết quả factory của `vi.mock`, nên `editorPanelState` sống qua các ca của `statusBarLibrary.test.ts`; `setup()` gọi `resetEditorPanel()`, xáo 8 seed xanh.
- `chapter-still-dirty` không tới được bằng đường thật vì phím bị từ chối khi đã khoá (`chapterPosition.test.ts`:220); nhánh giữ phiên dùng chung với `failed`, câu ở Library kiểm bằng ref ép.
- e2e chạy trên Vite của `tauri dev` có tem HMR nên từng ca chạy riêng bằng `--mochaOpts.grep`; ca Workspace đọc URL module từ `main.ts` để không ra instance khác.

## Plan Change Log

## Review Triage Log

Lượt 1 (lens Quick): high 0 · medium 4 · low 3 · false 0 · maybe-false 0.

- medium, patch: `switchChapter` không phát sự kiện nên `loading` và lời báo của A sống qua hợp âm sang B; đã phát qua `notifyChapterOpened`, thêm hai ca `goToNextChapter`.
- medium, patch: ca accept xanh ở cả hai nhánh vì `diffCursor` là -1; đã dời cursor, gỡ listener thì đỏ.
- medium, patch: e2e Library có thể qua trước `setMode`; đã kiểm dải sau khi lưới và tiêu điểm tới.
- medium, patch: e2e Workspace chấp nhận mọi panel và `activeElement` rỗng; đã đòi đúng lưới.
- low, patch: spy `closeReviewMode` không thấy binding nội bộ; đã kiểm `setDockSuspended`, gỡ cửa `idle` thì đỏ.
- low, patch: ca flush và lỗi mở không qua `openCurrentChapter`, ca mở lại không kiểm vị trí diff; đã sửa cả ba. Vế `still-dirty` giữ phiên chưa đo thật, xem Implementation Notes.
- low, reject: câu `chapter-*` của hợp âm Workspace còn hiện ở Library và câu cũ ở lại tới lượt editor kế; cần flush trượt mới gặp, sửa phải thêm trạng thái nguồn phát.

## Design Notes

Phép so dùng `openChapterId`, không dùng `editorChapterId`: phiên đọc Chương của chính nó, còn `editorChapterId` có thể `null` giữa hai lượt nạp. Lượt `no_chapter` có `openChapterId` là `null` nên mọi Chương mở xong đều đóng lời báo, đúng vì lời báo đó đã hết đúng.

Ở Workspace, `closeReviewMode` và `openChapterById` cùng chờ `nextTick`: phiên đăng ký trước nên trả tiêu điểm về `previousOwner` trước, rồi `enterFocus('panel.grid')` thắng. Kết quả cuối là lưới, khớp mọi lượt mở Chương.

## Verification

**Commands:**
- `npx vitest run tests/frontend/reviewModeChapterOpen.test.ts tests/frontend/reviewMode.test.ts tests/frontend/statusBarLibrary.test.ts` -- exit 0.
- `npm run test:story 14-12` -- exit 0.
- `npx vue-tsc --noEmit`, `npm run check:lint` -- exit 0.
- `npm run check:commands`, `npm run check:debt-owner` -- exit 0; đỏ có từ trước phải chỉ ra commit gây đỏ.
- `cargo build --locked --features wdio --manifest-path src-tauri/Cargo.toml`, rồi `npx wdio run e2e/wdio.conf.mjs --spec e2e/specs/review-mouse.e2e.mjs` -- xanh, chạy ngoài sandbox.

**Manual checks (if no CLI):**
- Epic 14: Ice mở Review ở A, sang Thư viện, mở B, thấy dải đóng; mở lại A khi Review ở A, thấy dải còn.
