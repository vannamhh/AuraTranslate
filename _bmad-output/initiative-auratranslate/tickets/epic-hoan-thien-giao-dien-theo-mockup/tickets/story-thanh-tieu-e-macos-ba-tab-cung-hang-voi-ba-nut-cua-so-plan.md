---
title: 'Thanh tiêu đề macOS: ba tab cùng hàng với ba nút cửa sổ'
type: 'feature'
ticket: '25'
created: '2026-10-25'
status: 'built'
baseline_revision: '019c7658b75592a6278b71f6da8d339396972281'
route: 'full'
route_source: 'auto'
risk: 'medium'
review: 'quick'
review_source: 'pinned'
lenses_ran: ['quick']
review_loop_iteration: 0
context:
  - '{project-root}/_bmad-output/initiative-auratranslate/architecture-auratranslate/architecture-auratranslate.md §AD-58'
---

<frozen-after-approval reason="human-owned intent — do not modify unless human renegotiates">

## Intent

**Problem:** Trên macOS, thanh tiêu đề của hệ điều hành chiếm một dải 28pt riêng phía trên ba tab, khác thanh `.tb` của `mockups/web-import.html` nơi ba nút cửa sổ và ba tab cùng một hàng.

**Approach:** Thi hành AD-58 (Rule 1–8): thanh tiêu đề macOS phủ lên webview, năm quyền, một vùng kéo chỉ trên macOS do `drag.js` của Tauri xử lý, ba menu đóng ở pha capture, header và lớp phủ chừa chỗ cho ba nút. Windows không đổi.

## Boundaries & Constraints

**Always:** đúng năm chuỗi quyền của AD-58 Rule 2; vùng kéo chỉ là `header.titlebar` và chỉ khi `detectIsMac()`; lề trái header và đệm lớp phủ đều là token khoảng cách, chỉ áp trên macOS qua một dấu nền tảng trên `<html>`; mọi chỗ ghi *ba quyền* đổi cùng commit; test mới phải đỏ khi gỡ đúng phần nó canh (gỡ thật, không comment).

**Never:** `core:window:default` hay quyền `core:window:*` thứ sáu; `decorations: false`; `acceptFirstMouse`; import `@tauri-apps/api/window`/`webviewWindow` hay chuỗi `plugin:window|` trong `src/`; handler kéo tự viết; thêm hàng vào `EXPECTED_SPACING` (bảng đóng băng là bản sao DESIGN.md); đổi hành vi trên Windows.

## I/O & Edge-Case Matrix

| Scenario | Input / State | Expected Output / Behavior | Error Handling |
|----------|--------------|---------------------------|----------------|
| Kéo | macOS, nhấn giữ chỗ trống của header rồi rê | cửa sổ di theo | — |
| Bấm tab | macOS, bấm một tab | đổi chế độ, cửa sổ không di | — |
| Bấm đúp | macOS, bấm đúp chỗ trống header | phóng to / thu lại | chuột di giữa hai lần ⇒ không gì |
| Menu đang mở | `Công cụ ▾` (hoặc Dropdown, nguồn tra cứu) mở, bấm chỗ trống header | menu đóng | — |
| Lớp phủ mở | mở một lớp phủ toàn cửa sổ | nội dung không nằm dưới ba nút; không kéo được | — |
| Windows | mọi thao tác trên header | như trước story: không vùng kéo, không lề | — |

**Decisions (Ice, 2026-10-25):**
- Lớp phủ tránh ba nút bằng cách đẩy xuống: trên macOS, scrim của 17 lớp phủ ở Code Map thêm `padding-top` = `var(--space-titlebar-height)`; không quy tắc theo bề rộng. Panel mất 24px chiều cao.
- Giữ trọn plan dù vượt 1600 token; rủi ro trôi ngữ cảnh do ba pha agent mới (Rust · Webview · Tests) gánh.

</frozen-after-approval>

## Code Map

- `src-tauri/tauri.conf.json:14-23` -- cửa sổ `main`: thêm `titleBarStyle`, `hiddenTitle`, `trafficLightPosition` (bắt đầu `{x:16,y:13}`, số đo của Ice chốt sau).
- `src-tauri/capabilities/main.json` -- 3 → 5 quyền; câu `description` *"Ba tập quyền"*.
- `src-tauri/tests/config_invariants.rs:19-23` (`read_json`), `:370-404` -- sửa `assert_eq!` thành năm chuỗi + thông điệp `:394`; thêm test khối `app.windows` (chưa test nào đọc nó).
- `src-tauri/SECURITY-NOTES.md:134,157-174` -- *đúng BA*, tiêu đề, bảng thêm hai hàng.
- `scripts/check-deps.mjs:180`, `src-tauri/src/lib.rs:1931,1993,2130` -- chú thích *ba quyền* đổi thành năm (`:1993` còn trỏ sai `config_invariants.rs:333`).
- `src/commands/index.ts:128` `detectIsMac()` -- đã export; dùng lại, không viết bộ dò mới.
- `src/main.ts:1255` -- trước `mount`, đặt `document.documentElement.dataset.platform = 'mac'` khi `detectIsMac()`. Hiện không có dấu nền tảng nào trên DOM; `tokens/index.ts:135` là khuôn (`dataset.theme`).
- `src/App.vue:206` header, `:424-436` CSS -- `:data-tauri-drag-region` gắn theo nền tảng; `user-select: none`; lề trái macOS bằng token.
- `src/tokens/tokens.json:596-608` spacing, `:642` `deviations` -- token mới (`traffic-light-inset`, đề xuất 88px, bội của 4 cho Kiểm K) kèm một mục `deviations` có `question`/`reason`; `scripts/check-tokens.mjs:665-694` cấm thêm vào `EXPECTED_SPACING`.
- `src/TitlebarToolsMenu.vue:69`, `src/DropdownMenu.vue:69-70`, `src/panels/LookupSourcesMenu.vue:79` -- listener bấm-ra-ngoài thêm `true` (capture) ở cả add lẫn remove.
- Lớp phủ có nội dung chạm góc trên trái (17): `AiPromptInspectorOverlay.vue:571` (tràn viền) và 16 lớp neo trên có `max-width`: TmImport `:167`, GlossaryManage `:890`, PromptLibrary `:916`, ReviewerImport `:241`, SegmentHistory `:374`, GlossaryHarvest `:247`, Alignment `:308`, GlossaryImport `:268`, ChapterSplit `:162`, Settings `:749`, Export `:627`, LibraryImport `:332`, GlossaryQueue `:473`, TmManage `:990`, ChapterMerge `:167`, PromptImport `:270`. Không đụng: ProofreadMemory, ProofreadScope (neo phải), `LookupDrawer` (ngăn phải), ReadingMode toc/marks (giữa màn), `AttributionOverlay` (không phải lớp phủ cửa sổ).
- Test sẵn có: `tests/frontend/titlebarToolsMenu.test.ts:104,116` (đã giả lập bấm ra ngoài), `:182-202` đọc `App.vue` bằng regex; `dropdownMenu.test.ts`, `lookupSourcesPopup.test.ts` chưa có ca bấm ra ngoài; Vitest chạy `happy-dom`; `e2e/specs/titlebar-tools.e2e.mjs` bấm `.tools-trigger` rồi các mục.

## Tasks & Acceptance

**Execution:**
- [ ] `src-tauri/tauri.conf.json`, `capabilities/main.json`, `tests/config_invariants.rs`, `SECURITY-NOTES.md`, `scripts/check-deps.mjs`, `src/lib.rs` (chỉ chú thích) -- Rule 1, 2 -- pha Rust.
- [ ] `src/main.ts`, `src/App.vue`, `src/tokens/tokens.json` -- dấu nền tảng, vùng kéo, lề token -- Rule 3, 5.
- [ ] ba tệp menu -- capture -- Rule 4.
- [ ] 17 lớp phủ ở Code Map -- trên macOS, scrim thêm `padding-top: var(--space-titlebar-height)` (khoá theo `:root[data-platform="mac"]`) -- Rule 5.
- [ ] `tests/frontend/titlebarDragRegion.test.ts` (mới) -- đọc `App.vue`: đúng một thuộc tính, `deep`, trên `header.titlebar`, gắn theo nền tảng; quét `src/` không có ba mẫu cấm; một ca chạy dấu nền tảng cả hai chiều -- Rule 3, 5.
- [ ] `dropdownMenu.test.ts`, `lookupSourcesPopup.test.ts`, `titlebarToolsMenu.test.ts` -- mỗi menu: mousedown lên một header mang thuộc tính kéo, có listener bubble chặn sự kiện (giả `drag.js`), menu vẫn đóng -- Rule 4.
- [ ] `deferred-work.md` -- hai mục nợ: `Chủ: Epic 12` (Ice kiểm app thật macOS: ba nút giữa thanh, kéo, bấm đúp, menu, góc lớp phủ; chốt `trafficLightPosition` và token) và `Chủ: B7` (Windows không đổi).

**Acceptance Criteria:**
- Given cấu hình mới, when chạy `config_invariants`, then đúng năm quyền và khối cửa sổ khớp Rule 1; đối chứng: xoá `titleBarStyle` ⇒ đỏ, thêm `core:window:default` ⇒ đỏ.
- Given `App.vue`, when chạy `titlebarDragRegion.test.ts`, then xanh; đối chứng: dời thuộc tính lên phần tử bao header ⇒ đỏ.
- Given một menu đang mở, when bấm header có giả `drag.js`, then menu đóng; đối chứng: bỏ `capture` ở menu đó ⇒ đỏ.
- Given dấu nền tảng không phải mac, then header không có lề macOS và không có vùng kéo.

## Design Notes

`drag.js` đăng ký `mousedown` bubble trên `document` lúc khởi động và gọi `stopImmediatePropagation` trên vùng kéo, nên listener bubble của menu không bao giờ chạy; capture chạy trước. Giả `drag.js` trong test bằng một listener bubble trên `document` đăng ký trước menu, gọi `stopImmediatePropagation` khi target nằm trong `[data-tauri-drag-region]`.

AD-58 Rule 5 ghi token *"khoá trong `check-tokens.mjs` như `titlebar-height`"*; cơ chế thật của cổng cho token không có trong DESIGN.md là mục `deviations` (bảng `EXPECTED_SPACING` cấm thêm hàng). Mục `deviations` vẫn bị `compare()` đối chiếu, nên ý của Rule giữ nguyên.

## Verification

**Commands:**
- `npm run build` rồi `cargo test --test config_invariants` -- expected: xanh; đối chứng đỏ như AC.
- `npx vitest run tests/frontend/titlebarDragRegion.test.ts tests/frontend/titlebarToolsMenu.test.ts tests/frontend/dropdownMenu.test.ts tests/frontend/lookupSourcesPopup.test.ts` -- expected: xanh.
- `npm run check:tokens`, `npm run check:debt-owner` -- expected: xanh.
- Bộ đầy đủ một lần (story chạm `tauri.conf.json` và capabilities, là wiring dùng chung theo AGENTS.md).
- e2e ngoài sandbox: `titlebar-tools` và một spec mở lớp phủ (`story-12-2-library-top-grid-overlay`) -- expected: xanh.

**Manual checks:**
- Ice: ảnh app thật macOS cạnh `web-import.html`; kéo, bấm đúp, mở Cài đặt và Xem prompt xem góc trên trái (mục nợ `Chủ: Epic 12`).

## Implementation Notes

- Ba pha agent mới (Rust · Webview · Tests), ghi chú ở `../handoff-thanh-tieu-de-macos-phases/`.
- Thuộc tính kéo là biểu thức gắn (`:data-tauri-drag-region="titlebarDragRegion"`, `detectIsMac() ? 'deep' : undefined`), nên test đọc `App.vue` khớp biểu thức chứ không khớp chuỗi `"deep"`.
- Token `traffic-light-inset` 88px vào `tokens.json` kèm mục `deviations`; `EXPECTED_SPACING` không đổi.
- Thêm một ca e2e vào `titlebar-tools` (ngoài Execution của plan) cho hàng "Lớp phủ mở" của ma trận và để kiểm dấu nền tảng bằng hành vi, vì ca Vitest chỉ kiểm thứ tự chuỗi trong `main.ts`. Đối chứng: gỡ dòng dấu nền tảng ⇒ ca đỏ (`platform ''`, lề 16px, đệm 16px).
- Ma trận: kéo và bấm đúp là hành vi cửa sổ gốc, WebDriver không lái được; chúng nằm trong mục nợ `Chủ: Epic 12`. Bấm nút trong header vẫn chạy (`realClick` lên `.tools-trigger` trong e2e). Windows: ca `detectIsMac` false và quy tắc lề chỉ dưới dấu `mac`; máy thật ở mục `B7`.
- Bộ đầy đủ: 13/14 cổng xanh, Vitest 3179/3179, build, cargo test 60 binary 0 đỏ; e2e hai spec 11/11. `check:gates` Kiểm G đỏ từ trước story: `RULE_LIGHTNESS_FLOOR` của `41e956f` trong `check-commands.mjs` không đi qua `judgeFloor`; story không chạm tệp đó.

## Plan Change Log

## Review Triage Log

Lượt 1 (quick): high 0 · medium 0 · low 2 · false 1 · rejected low 1 · rejected (sửa plan) 1.
- false · reject — "Không canh phía không-mac": gỡ `if (detectIsMac())` khỏi `main.ts` làm ca thứ tự chuỗi đỏ (chuỗi đó chứa điều kiện), gỡ selector `:root[data-platform='mac']` làm ca lề đỏ; e2e đã đối chứng bằng gỡ dòng dấu nền tảng.
- low · patch — dọn listener giả `drag.js` và `unmount` chạy sau `expect` nên một ca đỏ rò sang ca sau (cùng gốc với ca e2e để lớp phủ mở, đã thấy ở lượt đối chứng): `installFakeDragJs` dọn bằng `onTestFinished`, `titlebarToolsMenu` unmount bằng `onTestFinished`, ca e2e đóng lớp phủ trong `finally`. Vitest 4 tệp 30/30, e2e `titlebar-tools` 7/7.
- low · reject — header giả trong ba ca menu không phải header của `App.vue`: đúng chữ của plan; vị trí thuộc tính đã có ca đọc `App.vue` và e2e canh.
- reject — task của plan còn `- [ ]`: sửa plan.
- low · patch — `lib.rs:1931` viết lại câu hỏi lịch sử "ba permission hiện có có đủ không" của Task 0 Story 1.15: trả về "ba".
