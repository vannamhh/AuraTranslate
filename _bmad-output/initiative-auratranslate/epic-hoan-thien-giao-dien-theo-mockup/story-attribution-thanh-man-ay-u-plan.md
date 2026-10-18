---
title: 'Attribution thành màn đầy đủ'
type: 'feature'
ticket: '21'
created: '2026-10-18'
status: 'built'
baseline_revision: '0dc7b46b00963b13d11e1612e5a9d9ba2803bcd7'
route: 'full'
route_source: 'auto'
risk: 'medium'
review: 'quick'
review_source: 'pinned'
lenses_ran: ['quick']
review_loop_iteration: 2
context:
  - '{project-root}/_bmad-output/initiative-auratranslate/ux-auratranslate/mockups/sources-attribution.html'
---

<frozen-after-approval reason="human-owned intent — do not modify unless human renegotiates">

## Intent

**Problem:** Attribution (FR109) còn là hộp thoại phủ: khung viền `.attr-panel` rộng tối đa 1100px cuộn trong `.attr-scrim` che luôn thanh tiêu đề (b:99); mỗi hàng nguồn mang hai nút riêng thay vì một cặp nút ở chân bảng (b:103); thiếu chữ tĩnh của mockup (B11).

**Approach:** Dựng màn đầy đủ theo khuôn các màn nhập (`ip-screen`): thanh tiêu đề ba tab vẫn hiện, ô phải của nó ghi "Giới thiệu · Nguồn dữ liệu", không scrim, không khung viền. Hai nút "Mở văn bản giấy phép" và "Sao chép bản ghi công" dời xuống chân bảng, dùng chung cả bảng, chạy lại đúng hai lệnh `attribution.open_license` / `attribution.copy_credit`. Bảng vẫn dựng từ `list_dict_sources` (FR36, FR109), không phép lọc.

## Decisions (Ice, 2026-10-18)

1. Hai nút chân bảng nhắm **hàng đang chọn**: mở màn thì hàng đầu được chọn sẵn, hàng chọn có viền/nền nhìn thấy, bấm hoặc Tab vào hàng để đổi (`aimAttributionSource`); giữ hợp đồng một nguồn của 10.4 (`creditTextFor`, `readDictLicenseText(code)`).
2. In ngay chữ tĩnh "Toàn bộ giấy phép kèm trong bản phát hành" như mockup (đúng sau 10.5), cùng "Tab để duyệt bảng".
3. Bấm tab chế độ khi màn đang mở thì **đóng Attribution** (xoá nhắm, nhả khung giấy phép); tiêu điểm theo chế độ mới.
4. Giữ plan đầy đủ dù vượt 1.600 token (2.934 trước khi trả lời câu hỏi).
5. Khi màn đang mở, mọi điều khiển thanh tiêu đề ngoài ba tab chế độ (công tắc bố cục, `Công cụ ▾`) bị **vô hiệu**, như công tắc bố cục đã vô hiệu khi Review Mode mở; chỉ tab chế độ còn chạy (Quyết định 3).
6. Khung văn bản giấy phép **đi theo hàng đang chọn**: chọn hàng khác khi khung đang mở thì khung hiện giấy phép của hàng đó; hàng đó không có văn bản giấy phép thì khung đóng và tiêu điểm về hàng.
7. Màn **phủ toàn bộ vùng dưới thanh tiêu đề**, gồm cả dải thêm nhanh Glossary, dải Proofreader ở chân và thanh trạng thái; chỉ thanh tiêu đề còn nhìn thấy (điều khiển của nó theo Quyết định 5).

## Boundaries & Constraints

**Always:** Hàng nguồn dẫn xuất từ các tệp có mặt; gỡ một tệp thì hàng biến mất, không code theo `code` nào (`dict_boundary.rs`). Nguồn đang tắt vẫn liệt kê (AC10 của 1.19). Mọi `@click` là đúng một `dispatch`; chữ qua `vi.json`, màu qua token; giữ `trapTab`, Escape đóng khung xem giấy phép trước, bề mặt chọn vai `'display'`, trả tiêu điểm về `[data-attribution-open]` (UX-DR17). Nút chân bảng không hoạt động im lặng: không có hàng được nhắm thì nút bị vô hiệu hoá.

**Never:** Không đổi Rust, schema `.db`, `dict-manifest.toml`; không thêm chế độ thứ tư (AD-24); không đổi bốn cột của bảng (Quyết định 1 của 10.4); không thêm lối mở Attribution khác (vẫn `attribution.open` từ panel Tra cứu); không gán phím mặc định.

## I/O & Edge-Case Matrix

| Scenario | Input / State | Expected Output / Behavior | Error Handling |
|----------|--------------|---------------------------|----------------|
| Mở màn | Có nguồn nạp | Thanh tiêu đề còn ba tab, ô phải "Giới thiệu · Nguồn dữ liệu", không `.attr-scrim` | — |
| Chân bảng | Có nguồn | Đúng hai nút ở chân bảng; không `.attr-actions` trong hàng nào | — |
| Gỡ một tệp nguồn | Thiếu một `.db` | Hàng của nguồn đó biến mất, màn không lỗi | — |
| Không nguồn nào | `dictSources = []` | Màn vẫn mở, hai thẻ pháp lý hiện, hai nút vô hiệu | `attribution.empty` |
| Nguồn không có văn bản giấy phép | `license_text_len = 0` | "Mở văn bản giấy phép" vô hiệu khi hàng đó được nhắm | — |
| Clipboard từ chối | `writeText` reject | Câu lỗi hiện cạnh nút ở chân bảng | `attribution.copy_failed` |
| Mở màn có nguồn | Hàng đầu chưa ai chọn | Hàng đầu được chọn, nhìn thấy; hai nút nhắm hàng đó | — |
| Đổi tab chế độ | Màn đang mở | Màn đóng, khung giấy phép đóng, nhắm bị xoá | — |

</frozen-after-approval>

## Code Map

Đường dẫn tính từ gốc repo. Cây làm việc đang ở `baseline_revision`. **Bước 0 của người dựng:** `git apply /tmp/claude-501/-Users-hoangnam-LocalSites-addon-AuraTranslate/8c94afef-a267-4682-a41c-6c8bfbb0b325/scratchpad/12-21-loop2-code.patch` (11 tệp: `App.vue` · `AttributionOverlay.vue` · `TitlebarToolsMenu.vue` · `dictSourcesState.ts` · `vi.json` · `commands/keys.ts` · `layout/LookupDrawer.vue` · ba vitest · `attribution-focus.e2e.mjs`), rồi làm đúng các delta dưới đây. Mọi thứ bản vá đã có mà Tasks không nêu thì giữ nguyên: `.attr-screen` bỏ khung viền, ô phải `data-titlebar-attribution`, `:disabled` ở `.layout-switch` và `[data-tools-trigger]`, không `aria-modal`, aim watch (immediate, re-aim khi mã biến mất), Quyết định 6 watch, tiêu điểm lui về `tr.attr-aimed`, chân bảng sau `.attr-legal`, `aria-current`, harness App-mount của vitest, khối hình e2e.

- `src/App.vue` -- `.shell` là flex dọc `100vh`: `p.config-error` · `header.titlebar` · `div.modeport` (:514, `flex:1; min-height:0; isolation:isolate`) · `GlossaryQuickAdd` · `ProofreaderStrip place="foot"` · `StatusBar` · `pre.selftest` · các lớp phủ gốc, gồm `<LookupDrawer />` (:361). Bản vá bọc riêng `.modeport` + `<AttributionOverlay />` trong `div.stage` và đổi `.modeport` sang `height:100%`: SAI với Quyết định 7 (ba dải đứng ngoài `.stage`). `GlossaryQuickAdd`, `ProofreaderStrip`, `StatusBar` không có `position`/`z-index` (đã grep), nên đưa vào `.stage` không đổi hình học.
- `src/AttributionOverlay.vue` -- bản vá thêm `watch(currentMode)` + cờ `skipFocusReturn` + import `currentMode`: bỏ cả ba (xem `src/commands/index.ts`). Watch `attributionIsOpen` nhánh đóng gọi `back.focus()` đồng bộ trong flush `pre`, TRƯỚC render. Chân bảng hiện hiện lỗi sao chép theo `attributionCopyFailedCode !== null`.
- `src/commands/index.ts` -- `registerAll` :1346-1362: vòng `for (const mode of MODE_IDS)` đăng ký `mode.*`, `run` chỉ gọi `setMode(mode)`. `deps.closeAttribution?: () => void` đã có (:724) và `src/main.ts:924` đã tiêm. `src/commands/**` nạp được bằng Node thuần: không import state Vue vào đây, chỉ dùng dep tiêm.
- `src/modes/modeState.ts` -- `setMode` thoát sớm sau `enterFocus` khi cùng chế độ nên `currentMode` không đổi. KHÔNG sửa tệp này.
- `src/layout/LookupDrawer.vue` -- gốc `div.ld-scrim` `v-if="lookupDrawerIsOpen"`, `position: fixed; inset:0; z-index:10`, chứa `LookupPanel` thật; nút `[data-attribution-open]` trong panel đó nghĩa là Attribution MỞ ĐƯỢC từ ngăn kéo khi ngăn kéo đang mở. `src/main.ts:1118` `isBlocked` có `attributionIsOpen`: chặn mọi hợp âm, nên `layout.lookup_drawer_open` không chạy khi màn mở; nút của `StatusBar` bị màn che.
- `src/panels/dictSourcesState.ts` -- `aimAttributionSource`/`copyAimedCredit`: `copyFailedCodeRef` được ghi SAU `await writeText`, tức có thể ghi sau khi nhắm đã đổi. Mọi `ref` module-level mới cần `reset*` (`check:panel-refs` quét toàn `src/**/*.ts`), nên không thêm bộ đếm vào `modeState.ts`.
- `src/TitlebarToolsMenu.vue` -- `[data-tools-trigger]` :82.
- `src/GlossaryQuickAdd.vue`:253-257 và `src/StatusBar.vue`:658-665 -- bình luận nói `.modeport` là `flex:1` của `.shell`; thành sai sau khi có `.stage`.
- `scripts/check-layout.mjs` F.3 -- chỉ canh `.modeport {…isolation: isolate}`; giữ. `scripts/check-tokens.mjs` Kiểm F -- `box-shadow` thật bị cấm KHÔNG miễn trừ (:1440), nên không dùng bóng đổ để đánh dấu hàng nhắm. `check-commands.mjs` `CLICK_FLOOR = 237`: không đổi.
- `e2e/specs/glossary-quick-add.e2e.mjs`:196-232 -- đo `.modeport` bị đẩy lên bởi dải; vẫn đúng khi `.modeport` là con flex của `.stage` và dải là em của nó. `story-12-7-workspace-grid-status-bar.e2e.mjs` đọc `footer.status`, không phụ thuộc cha.
- `grep -rn "attr-scrim" src tests e2e scripts` sau khi xong chỉ còn hai khẳng định PHỦ ĐỊNH (`attributionOverlayActions.test.ts` và `shape.scrim` ở `attribution-focus.e2e.mjs`).
- Không đổi: `src-tauri/**`, `LookupPanel.vue:434`, bảng lệnh `attribution.*`, `src/modes/modeState.ts`, `src/main.ts`.

## Tasks & Acceptance

**Execution:**
- [ ] `src/App.vue` -- (a) `div.stage` bao `.modeport`, `<GlossaryQuickAdd />`, `<ProofreaderStrip place="foot" />`, `<StatusBar />` rồi `<AttributionOverlay />` (cuối, trước `</div>`). `.stage { position: relative; flex: 1; min-height: 0; display: flex; flex-direction: column }`; `.modeport` GIỮ nguyên `flex:1; min-height:0; isolation:isolate` (bỏ `height:100%` của bản vá). `<LookupDrawer />` và `pre.selftest` ở lại ngoài `.stage`. Ba dải vẫn mounted (không `v-if`/KeepAlive mới); màn mờ đục phủ chúng, `trapTab` giữ tiêu điểm trong màn, hợp âm bị `isBlocked`. (b) Sửa chữ ở `.modeport`/`.selftest`: thay dòng bản vá bằng 1-2 dòng tiếng Anh nêu sự thật hiện tại, không lịch sử (`.attr-screen` là em sau `.modeport` trong `.stage`, không z-index; `.selftest` vẫn con của `.shell` còn `.stage` là `flex:1`). -- Quyết định 7, b:99
- [ ] `src/commands/index.ts` -- trong `run` của `mode.${mode}`, gọi `deps.closeAttribution?.()` TRƯỚC `setMode(mode)` (đóng cả khung giấy phép, xoá nhắm; chạy cả khi bấm lại tab đang bật). Không sửa `modeState.ts`, `main.ts`. Đã loại: bộ đếm `modeActivation` trong `modeState.ts` (thêm `ref` module-level ⇒ `reset*` của `check:panel-refs`, và buộc Attribution theo dõi một tín hiệu của `modes/`). -- Quyết định 3
- [ ] `src/AttributionOverlay.vue` -- (a) bỏ `watch(currentMode)`, `skipFocusReturn`, import `currentMode` của bản vá. (b) Chân bảng: lỗi sao chép chỉ hiện khi `attributionCopyFailedCode === attributionAimedCode` (không `!== null`). (c) Nhánh đóng của `watch(attributionIsOpen)`: sau `back.focus()` đồng bộ thêm `void nextTick(() => { if (back.isConnected && document.activeElement !== back) back.focus() })`, để ngăn kéo Tra cứu bị ẩn lúc màn mở (xem LookupDrawer) kịp hiện lại; ca đổi chế độ thì `back` đã rời DOM nên bỏ qua êm, không `console.warn` mới. (d) CSS hàng nhắm không thêm padding: `.attr-table td:first-child { border-left: 2px solid transparent }` cố định cho MỌI hàng, `.attr-table tr.attr-aimed td:first-child { border-left-color: var(--color-primary) }`, nền `--color-surface`; bỏ `padding-left: 8px` của bản vá. (e) Bình luận: `.attr-scrim` ở :148,150,166 và đầu tệp; dòng `focusableWithin` bản vá viết lại bằng tiếng Việt: xoá hoặc thay bằng 1-2 dòng tiếng Anh nêu bất biến (ví dụ `:not([tabindex="-1"])` loại panel khỏi vòng Tab). Không giữ lịch sử. -- Quyết định 1, 3, 6; b:103
- [ ] `src/layout/LookupDrawer.vue` -- thêm `v-show="!attributionIsOpen"` trên `div.ld-scrim` (import `attributionIsOpen` từ `../panels/dictSourcesState`): giữ nguyên DOM và `returnFocusTo`, chỉ ẩn khi màn mở. Bình luận :140,147: thay bằng tiếng Anh 1-2 dòng (`Anchored to the right edge; colours via tokens only.`; `aura-allow-z-index: dockview builds a stacking context per group; .modeport isolates it.`). -- Quyết định 7
- [ ] `src/commands/keys.ts`:566 -- đổi `.attr-scrim` thành `.attr-screen` giữ nghĩa; câu đó tiếng Việt nên thay cả khối `⚠️` chạm bởi dòng đổi bằng 1-2 dòng tiếng Anh (`Attribution's Escape is a plain @keydown.esc on .attr-screen, not a command; swallow chords by returning early, no preventDefault.`).
- [ ] `src/GlossaryQuickAdd.vue`:253-257, `src/StatusBar.vue`:658-665 -- sửa câu `.modeport` là `flex: 1` cho khớp `.stage` (1-2 dòng tiếng Anh: `.stage` is the flex:1 column holding .modeport and these strips).
- [ ] `src/TitlebarToolsMenu.vue`, `src/panels/dictSourcesState.ts`, `src/i18n/vi.json` -- như bản vá, không delta.
- [ ] `tests/frontend/attributionOverlayActions.test.ts` -- delta trên bản vá: (1) mock `read_dict_license_text` trả văn bản RIÊNG theo `code` trong tham số lệnh (ví dụ `LICENSE BODY <code>`); ca Quyết định 6 khẳng định `attributionLicenseView.value?.code` là mã mới và văn bản chứa đúng mã đó (cả ở DOM); ca "hàng không giấy phép" khẳng định khung đóng và `document.activeElement` là `tr.attr-aimed`. (2) Lỗi sao chép muộn: clipboard reject có điều khiển (promise mở tay), bấm sao chép, nhắm hàng khác, rồi mới reject: không còn `.attr-error` trong chân bảng. (3) Tách ca vị trí thành các `it` MỘT khẳng định mỗi cái, đọc nguồn đã bỏ dòng chú thích: `.attr-screen` là con của `.stage` sau `.modeport`; CSS `.attr-screen` có `position: absolute`; không `position: fixed`; không `--space-titlebar-height`; không `z-index`; luật `tr.attr-aimed` không chứa `padding`/`box-shadow`, còn `td:first-child` có `border-left` cùng độ rộng ở luật nền và luật nhắm. (4) Harness App-mount: stub ba dải bằng `<div data-stub="quick-add|proofreader|status">`; khẳng định cả ba nằm TRONG `.stage` và đứng trước `.attr-screen`; `LookupDrawer` ở ngoài `.stage`. (5) Tab chế độ: bấm tab KHÁC rồi bấm tab ĐANG BẬT (Workspace), mỗi lần qua `dispatch('mode.*')`, mỗi lần màn, khung giấy phép và nhắm đóng/xoá; `installCommands` trong harness phải tiêm `closeAttribution` thật. (6) `.layout-switch-btn` và `[data-tools-trigger]` kiểm `disabled` thành các `expect` RIÊNG (không gộp mảng) để gỡ một cái chỉ đỏ đúng nó.
- [ ] `tests/frontend/titlebarToolsMenu.test.ts`, `attributionOverlayUnreadable.test.ts` -- như bản vá. Thêm ca LookupDrawer (ở `attributionOverlayActions.test.ts` hoặc tệp riêng, stub `LookupPanel`): ngăn kéo mở + `attributionIsOpen` ⇒ `.ld-scrim` có `display: none`, đóng màn ⇒ hiện lại và tiêu điểm về nút mở màn.
- [ ] `e2e/specs/attribution-focus.e2e.mjs` -- như bản vá; xoá bình luận `Focus must be inside the screen, which no longer declares aria-modal.` (nói lịch sử) và giữ bình luận Escape ở dạng 1 dòng tiếng Anh. Không thêm bước mới.

**Acceptance Criteria:**
- Given Attribution mở, when đọc DOM, then không `.attr-scrim`, ba `.mode-tab` bấm được (`elementFromPoint`), ô phải ghi "Giới thiệu · Nguồn dữ liệu", `.attr-screen` không `position: fixed`, không lệ thuộc chiều cao thanh tiêu đề (`p.config-error` không làm nó che thanh tiêu đề).
- Given màn mở, then `GlossaryQuickAdd`, `ProofreaderStrip place="foot"`, `StatusBar` nằm trong `.stage` dưới màn (bị màn phủ), và `LookupDrawer` (mở sẵn từ trước) bị ẩn; đóng màn thì ngăn kéo hiện lại, tiêu điểm về nút mở màn.
- Given có ít nhất hai nguồn, then chân bảng đúng hai nút (nút đầu là nút chính), không hàng nào còn `button`.
- Given một nguồn bị gỡ, when mở màn, then hàng đó không còn, bảng không lỗi.
- Given khung giấy phép mở, when Escape, then khung đóng trước, tiêu điểm về nút chân bảng; Escape lần nữa đóng màn, tiêu điểm về nút mở.
- Given không có đích, then nút tương ứng vô hiệu; không bao giờ `focus()` lên nút vô hiệu.
- Given màn vừa mở (kể cả nguồn nạp sau), then hàng đầu mang `attr-aimed` + `aria-current`, "Sao chép bản ghi công" chép đúng nguồn đó; hàng nhắm không dịch ngang nội dung ô đầu.
- Given màn mở, when bấm một tab chế độ khác HOẶC tab đang bật, then màn và khung giấy phép đóng, nhắm bị xoá.
- Given màn mở, then `.layout-switch` hai nút và `Công cụ ▾` `disabled`; đóng màn thì hết.
- Given khung giấy phép mở, when nhắm hàng khác: có văn bản ⇒ khung hiện văn bản CỦA mã đó; không có ⇒ khung đóng, tiêu điểm ở hàng.
- Given sao chép bị từ chối rồi nhắm hàng khác (kể cả khi lời từ chối đến sau lúc đổi nhắm), then không hiện lỗi cạnh nút.
- Given màn mở, then có "Toàn bộ giấy phép kèm trong bản phát hành" và keycap `Tab` "để duyệt bảng" ngay sau `.attr-legal`.

## Implementation Notes

- Ba vòng review quick; mỗi vòng có intent_gap nên trả code về baseline rồi lập lại plan. Quyết định 5–7 sinh ra từ đó: màn để thanh tiêu đề sống nên lộ dần các lối ra không ai chặn (công tắc bố cục, `Công cụ ▾`, dải chân, thanh trạng thái).
- Màn là `div.attr-screen` `absolute; inset: 0` không `z-index`, con cuối của `div.stage` bọc `.modeport` + ba dải chân; đứng sau `.modeport` (vẫn `isolation: isolate`) nên vẽ đè lên dockview. `fixed` + offset chiều cao thanh tiêu đề bị loại vì `p.config-error` đẩy thanh tiêu đề xuống.
- `mode.*` gọi `deps.closeAttribution?.()` trước `setMode`: `setMode` cùng chế độ chỉ `enterFocus` rồi thoát, nên `watch(currentMode)` bỏ sót cú bấm tab đang bật.
- Bỏ `aria-modal` (thanh tiêu đề còn bấm được), giữ `role="dialog"` + `aria-labelledby` + bẫy Tab; hàng nhắm dùng `aria-current` (`aria-selected` vô nghĩa trên `<tr>` của bảng thường).
- `LookupDrawer` ẩn bằng `v-show` khi màn mở; lượt đóng thử `focus()` lần hai sau `nextTick` vì nút mở trong ngăn kéo còn `display: none` ở lượt đồng bộ. happy-dom bỏ qua `display: none` nên chỉ lượt dùng thật thấy được.
- `check:tokens` Kiểm F cấm `box-shadow`, nên hàng nhắm giữ `border-left: 2px solid transparent` trên mọi hàng và chỉ đổi màu viền.
- Còn hai chỗ `.attr-scrim` trong test, đều là assert phủ định.
- Nợ mở `Chủ: Epic 12`: lượt dùng thật màn đầy đủ; ngăn kéo hiện lại sau khi đổi chế độ (triage 24); bấm vùng trống thanh tiêu đề làm rơi tiêu điểm (triage 29).

## Plan Change Log

- **Loop 1 (2026-10-18), review quick.** Trigger: triage rows 1 and 4 (intent_gap, Ice answered as Decisions 5 and 6) plus bad_plan rows 2, 3, 5–9, 11, 12. Code reverted to `baseline_revision`; the reverted diff is kept at `/tmp/claude-501/-Users-hoangnam-LocalSites-addon-AuraTranslate/8c94afef-a267-4682-a41c-6c8bfbb0b325/scratchpad/12-21-loop1-code.patch` for reuse.
  - Known-bad states to avoid: `position: fixed` with `top: var(--space-titlebar-height)` (covers the titlebar under `p.config-error`); `aria-modal="true"` while the titlebar stays live; aim set only inside `openAttribution` (lost when sources load later); shared copy error shown regardless of aim; focus return to a disabled foot button; `aria-selected` on a plain `<tr>`; foot placed under the table instead of after `.attr-legal`.
  - KEEP: i18n keys `titlebar.attribution_label`, `attribution.hint_licenses_bundled`, `attribution.hint_browse`; titlebar cell `span[data-titlebar-attribution]` while open; readonly export `attributionAimedCode`; `watch(currentMode)` that closes the screen and skips the opener focus return; foot with exactly two `.attr-action` buttons (`data-license-open` without value), disabled when there is no target, rendered in the empty state too (not on load error); rows `tabindex="0"` with a visible aimed class; the reworked vitest cases and the e2e shape asserts (no scrim, label, 2 foot buttons, 0 row buttons); `CLICK_FLOOR` unchanged.

- **Loop 2 (2026-10-18), review quick.** Trigger: triage row 14 (intent_gap, Ice answered as Decision 7), bad_plan row 13, patch rows 15–17, 19, 22, 23. Code reverted to `baseline_revision` again; the reverted diff is kept at `/tmp/claude-501/-Users-hoangnam-LocalSites-addon-AuraTranslate/8c94afef-a267-4682-a41c-6c8bfbb0b325/scratchpad/12-21-loop2-code.patch` and is the base to rebuild from.
  - Known-bad states to avoid: closing only on `watch(currentMode)` (a click on the already-active tab never changes `currentMode`); anything interactive below the titlebar left uncovered (`GlossaryQuickAdd`, `ProofreaderStrip place="foot"`, `StatusBar`, and the root-level `LookupDrawer` painting over the screen); a Decision 6 test whose licence mock returns the same text for every code; copy error gated on `!== null` instead of the aimed code; extra `padding-left` on the aimed row's first cell; a counter-check whose red comes from a different assert than the one measured; rewritten comments on changed lines kept in Vietnamese or as history (`App.vue` `.modeport` / `.selftest`, `AttributionOverlay.vue` header and `focusableWithin` doc, `commands/keys.ts:566`, `layout/LookupDrawer.vue:140,147`), and `GlossaryQuickAdd.vue:255` / `StatusBar.vue:664` still saying `.modeport` is `flex: 1`.
  - KEEP: everything from loop 2's patch not named above — `div.stage` with `.attr-screen` absolute and no `z-index`, `.modeport` keeping `isolation: isolate`; Decision 5 `:disabled` on `.layout-switch` and `[data-tools-trigger]`; no `aria-modal`, `role="dialog"` + `aria-labelledby`; the immediate aim watch, re-aim on vanish, copy error cleared on aim change; Decision 6 watch and the focus fallback to `tr.attr-aimed`; foot after `.attr-legal` with primary first button and `<kbd>Tab</kbd>`; the vitest App-mount harness (`KeepAlive: false`, plain stubs, reset `sourcesResult`); the e2e shape block with `elementFromPoint` on all three `.mode-tab`.

## Review Triage Log

| # | Finding | Verdict | Route | Evidence |
|---|---------|---------|-------|----------|
| 1 | Titlebar controls other than the mode tabs stay live under `aria-modal` | medium | intent_gap | `.layout-switch` (`App.vue`, only in Workspace, where Attribution opens from) runs `layout.preset_*` → `api.clear()` behind the screen; `TitlebarToolsMenu` opens a second overlay; focus leaves `.attr-screen` and Esc/Tab stop. Decision 3 covers only the mode tabs, so the frozen block does not settle it. |
| 2 | Fixed `top: --space-titlebar-height` covers the titlebar when `p.config-error` shows | medium | bad_plan | `.shell` is a flex column with `.config-error` above `.titlebar` (`App.vue`), so the titlebar sits lower than one titlebar height; Design Notes chose `fixed` + offset, while `ip-screen` sits in flow inside `LibraryMode.vue`. |
| 3 | Opening before `loadDictSources` resolves leaves no row aimed | low | bad_plan | `openAttribution` aims `sources.value[0]` only at open; nothing re-aims when `sources` fills; the plan fixed the aim at open time. |
| 4 | Aiming another row while the licence viewer is open makes Escape focus a disabled button | medium | intent_gap | Rows are Tab stops; aiming a row with `license_text_len = 0` disables `button[data-license-open]`, and `focus()` on it is a no-op (focus falls to `body` if it was on the viewer's close button). Whether the viewer follows the aim, closes, or locks the aim is not settled. |
| 5 | Copy-failure message persists after the aim moves | low | bad_plan | Foot shows the error when `attributionCopyFailedCode !== null`; `aimAttributionSource` does not clear it; before, it was tied to `=== src.code`. |
| 6 | `aimedSource` computeds split the `'display'` comment from `useSelectionSurface` | low | bad_plan | `AttributionOverlay.vue` script order; folded into re-derivation. |
| 7 | Stale `.attr-scrim` / "row button" comments | low | bad_plan | `AttributionOverlay.vue` (focus-return and `trapTab` comments), `App.vue`, `commands/keys.ts`, `layout/LookupDrawer.vue` still name `.attr-scrim`. |
| 8 | New/edited e2e comments are Vietnamese and restate the asserts | low | bad_plan | `e2e/specs/attribution-focus.e2e.mjs` added comment + edited ⚠️ line, against AGENTS.md comment rules. |
| 9 | `aria-selected` on `<tr>` of a plain table is ignored by AT | low | bad_plan | Implicit role `row` outside `grid`/`treegrid`; `aria-current` is valid. |
| 10 | "Source removed" test cannot fail | false | — | It asserts `rows.toHaveLength(1)` with one source given; rows not driven by the list would render a different count, so the assert can go red. |
| 11 | ACs untested: tabs visible/clickable, tab click, Escape-twice after the viewer | low | bad_plan | e2e checks only no-scrim + label (finding 2 shows the tabs can be covered); vitest drives `setMode`, not `dispatch('mode.*')`; no case runs viewer-Escape then screen-Escape. |
| 12 | Layout differs from `sources-attribution.html` | low | bad_plan | Mockup `.btnrow` comes after `.legal`, first button is `btn pri`, `Tab` is a `.k` keycap; plan did not record these as deviations. |
| 13 | Clicking the active Workspace tab leaves the screen open, focus drops into the hidden Workspace | medium | bad_plan | `setMode` (`modes/modeState.ts`) returns after `enterFocus` when `mode.value === next`, so `watch(currentMode)` never fires; Attribution only opens from Workspace; Esc/Tab are bound on `.attr-screen` and `isBlocked` blocks chords. Decision 3 covers every tab click. |
| 14 | Status bar, Glossary quick-add and Proofreader foot strips stay live under the screen | medium | intent_gap | They render after `div.stage` in `App.vue`; `StatusBar` dispatches `glossary.queue.open` / `layout.lookup_drawer_open`, `ProofreaderStrip` runs `ai.proofread.*`; the Lookup drawer is `fixed; z-index: 10` and paints over `.attr-screen`. Decision 5 settles only the titlebar. |
| 15 | Decision 6 "swap" test cannot fail | medium | patch | The licence mock returns `'LICENSE BODY'` for every code and the case never reads `attributionLicenseView.value.code`; removing the `open_license` branch keeps it green. |
| 16 | A clipboard rejection that resolves after the aim moves shows the error beside the new aim | low | patch | `copyAimedCredit` writes `copyFailedCodeRef` after `await writeText`; the foot tests `!== null`, not `=== attributionAimedCode`. |
| 17 | Aimed row's first cell shifts 10px sideways | low | patch | `.attr-table td` has 0 left padding under `border-collapse`; `.attr-aimed td:first-child` adds `border-left: 2px` + `padding-left: 8px`. |
| 18 | "Focus follows the new mode" untested | low | rejected | Behaviour comes from `setMode`'s existing `enterFocus`; covering it needs real mode mounts, more than a direct correction, and users never meet a test gap. |
| 19 | Placement counter-check went red for the wrong reason; tools-trigger seam never removed | low | patch | Phase-3 handoff: the `fixed` edit turned red only on the `absolute` match; no removal of `:disabled="attributionIsOpen"` on `[data-tools-trigger]`. |
| 20 | `grep attr-scrim` not empty | low | rejected | Only two negative asserts remain; the fix is editing this plan (record in Implementation Notes at present). |
| 21 | `test:story 21` not used | false | — | `npm run test:story 12-21 -- --list` resolves this plan and selects the Attribution vitest files; the plan's `21` form is the wrong id shape, a plan edit. |
| 22 | Comments made false by the change | low | patch | `GlossaryQuickAdd.vue:255`, `StatusBar.vue:664` say `.modeport` is `flex: 1`. |
| 23 | Rewritten comments on changed lines still Vietnamese/history | low | patch | `App.vue` `.modeport` block ("lớp phủ Attribution cũ"), `.selftest`; `AttributionOverlay.vue` header and `focusableWithin` doc; `commands/keys.ts:566`; `layout/LookupDrawer.vue:140,147`; e2e "no longer declares aria-modal". |
| 24 | Opened from the Lookup drawer, a mode-tab click brings the drawer back over the new mode and the `nextTick` re-focus pulls focus into it | low | rejected → debt | Real: `lookupDrawerIsOpen` survives `mode.*`, the root-level `.ld-scrim` reappears when `v-show` flips, `back` is still connected. Needs a narrow window + drawer + Attribution + tab click; Esc closes the drawer. Calling `deps.closeLookupDrawer` from `mode.*` is not a direct correction: the drawer's own focus return then competes with the new mode. Ledger item `Chủ: Epic 12`. |
| 25 | Decision 6 "no licence ⇒ focus on row" test cannot fail | medium | patch | The case focuses `rows[1]` before `focusin`, so `activeElement` is already the aimed row; removing the `tr.attr-aimed` fallback stays green. |
| 26 | LookupDrawer case does not reach the `nextTick` re-focus | low | rejected → debt | happy-dom `focus()` ignores `display: none`, so the sync focus succeeds; only WebKit can show it (`tests/AGENTS.md`). Users never meet a missing guard and a narrow-window e2e step is more than a direct correction; added to the Epic 12 real-use check. |
| 27 | Placement source test also matches `<AttributionOverlay />` inside `.modeport` | low | patch | The lazy `[\s\S]*?` then `</div>` can consume `.modeport`'s own closing tag. |
| 28 | Comments on changed lines still Vietnamese or false | low | patch | `GlossaryQuickAdd.vue:255` ("nó" now means `.stage`), `StatusBar.vue:664`, `App.vue` `.selftest` and the new quick-add comment (`ProofreaderStrip` sits between), `AttributionOverlay.vue` header. |
| 29 | A click on a non-focusable titlebar area blurs to `body`; Escape and Tab stop reaching `.attr-screen` | low | defer | Same class existed before: a click on the old scrim margin outside `.attr-panel` also blurred to `body`. The live titlebar widens the area; fixing it means an app-level Escape route past `isBlocked`. |
| 30 | Manual real-app check has no ledger entry | low | patch | Plan Verification names debt `Chủ: Epic 12`; `deferred-work.md` has none. |
| 31 | Source-scanning placement cases live in vitest | low | rejected | Precedent in the same suite (`titlebarToolsMenu.test.ts` asserts `:disabled` source strings before this change); no other path owns `.attr-screen` placement (`check:layout` F.3 owns only `.modeport` isolation); moving them means a new `check-layout` rule, which is shared gate wiring. |

## Design Notes

Chỗ ngồi: `div.stage` (cột flex `flex:1` còn lại sau thanh tiêu đề, `position: relative`) chứa `.modeport`, ba dải dưới, rồi `.attr-screen` `absolute; inset: 0`. Màn bắt đầu ngay dưới `header.titlebar` kể cả khi `p.config-error` đẩy nó xuống, và phủ tới đáy vỏ (Quyết định 7). Đã loại: `fixed` + `top: var(--space-titlebar-height)` (sai khi có `p.config-error`); đặt trong `LibraryMode` như `ip-screen` (Attribution mở từ Workspace); đặt trong `.modeport` (sash 99 của dockview thắng `z-index` 10); `.stage` chỉ bao `.modeport` (để ba dải lộ và bấm được dưới màn, đúng lỗi row 14). Không `z-index`, nên không cần `aura-allow-z-index`; `isolation: isolate` của `.modeport` giữ sash nằm dưới (F.3). `.modeport` vẫn `flex:1` trong cột `.stage`, nên phép đo "dải đẩy `.modeport` lên" của `glossary-quick-add.e2e.mjs` giữ nguyên. Ba dải không unmount: không có state cần dựng lại, màn mờ đục che chúng, `trapTab` giữ tiêu điểm, `isBlocked` chặn hợp âm. Chưa dùng `inert`.

`LookupDrawer` (fixed, z-index 10, ở gốc) là thứ duy nhất vẽ đè được màn, và mở được màn từ chính nó, nên ẩn bằng `v-show` thay vì `v-if` (giữ `LookupPanel` đang mount và nút mở để trả tiêu điểm). Watch nhánh đóng chạy trước render nên nút nằm trong ngăn kéo còn `display:none` lúc `focus()` đồng bộ: vì vậy thêm lượt `nextTick` thứ hai, chỉ khi `back` còn trong DOM và chưa có tiêu điểm.

Quyết định 3 qua `mode.*`: seam là lệnh, vì mọi tab chế độ đã đi qua `dispatch('mode.*')` (một `dispatch` mỗi `@click`, `check:commands`) và `closeAttribution` đã là dep tiêm. Ca bấm tab đang bật không đổi `currentMode`, nên mọi cách theo dõi `currentMode` sai. Ca đổi sang chế độ khác: watch đóng chạy TRƯỚC render nên nút mở (Workspace) còn trong DOM, `focus()` thành công, rồi `onActivated` của chế độ mới gọi `enterFocus` (tiêu điểm theo chế độ mới). Ca cùng chế độ: tiêu điểm về nút mở trong Workspace. Các đường `setMode` không qua lệnh (khởi động, `openLibraryRescanReport`) không thể xảy ra khi màn mở: hợp âm bị chặn, Cài đặt không mở được dưới màn.

Hàng nhắm: `box-shadow` bị `check:tokens` Kiểm F cấm không miễn trừ, nên dùng `border-left: 2px solid transparent` cố định mọi hàng và chỉ đổi màu ở hàng nhắm (tiền lệ: `GlossaryHarvestOverlay.vue:318`); không padding thêm nên nội dung ô đầu không dịch.

Lệch mockup có chủ ý: bảng có hàng chọn nhìn thấy (Quyết định 1); khung giấy phép nằm ngay dưới bảng, trước thẻ pháp lý. Hai hit `.attr-scrim` còn lại trong `tests/` và `e2e/` là khẳng định phủ định (không có scrim), không phải sót.

`aria-modal` bỏ vì thanh tiêu đề còn tab sống; tiền lệ `ip-screen` cũng không khai. `isBlocked` giữ nguyên: phím `mode.*` bị chặn, chỉ chuột đổi tab.

Tách pha: **Rust: trống.** **Webview:** `App.vue`, `commands/index.ts`, `AttributionOverlay.vue`, `LookupDrawer.vue`, `keys.ts`, `GlossaryQuickAdd.vue`, `StatusBar.vue`, `TitlebarToolsMenu.vue`, `dictSourcesState.ts`, `vi.json`. **Tests chuyển chỗ:** ba vitest, `attribution-focus.e2e.mjs`. Chạy `npm run test:story 12-21 -- --list` rồi `npm run test:story 12-21`; không chạy toàn bộ.

## Verification

**Commands:**
- `npm run test:story 12-21` -- expected: xanh
- `npx vitest run tests/frontend/attributionOverlayActions.test.ts tests/frontend/attributionOverlayUnreadable.test.ts tests/frontend/titlebarToolsMenu.test.ts` -- expected: xanh
- `npm run check:commands && npm run check:i18n && npm run check:tokens && npm run check:panel-refs && npm run check:layout` -- expected: xanh
- `npm run test:e2e -- --spec e2e/specs/attribution-focus.e2e.mjs` và `glossary-quick-add.e2e.mjs` -- expected: xanh ngoài sandbox
- Đối chứng (mỗi phép GỠ thật, chạy đúng tệp test đó, và đỏ phải đến từ đúng `it` đang đo; ghi tên `it` đỏ):
  - Thêm `position: fixed;` vào `.attr-screen` GIỮ `position: absolute` -- chỉ ca "không `position: fixed`" đỏ. Thêm `top: var(--space-titlebar-height);` tương tự -- chỉ ca "không titlebar-offset" đỏ.
  - Chuyển `<StatusBar />` ra ngoài `.stage` -- ca ba dải trong `.stage` đỏ.
  - Bỏ `deps.closeAttribution?.()` ở `mode.*` -- cả hai ca bấm tab (khác và đang bật) đỏ.
  - Bỏ `:disabled="attributionIsOpen"` ở `[data-tools-trigger]` -- chỉ `expect` của nút công cụ đỏ; bỏ `|| attributionIsOpen` ở `.layout-switch` -- chỉ `expect` của hai nút bố cục đỏ.
  - Bỏ nhánh `dispatch('attribution.open_license')` của watch Quyết định 6 -- ca đổi nội dung đỏ ở mệnh đề `code`/văn bản theo mã.
  - Đổi điều kiện lỗi sao chép về `!== null` -- ca từ chối muộn đỏ.
  - Bỏ `v-show` ở `LookupDrawer` -- ca ngăn kéo ẩn đỏ.
  - Thêm `padding-left` vào `tr.attr-aimed td:first-child` -- ca CSS hàng nhắm đỏ.

**Manual checks (if no CLI):**
- Bố cục nhìn thấy được (màn phủ tới đáy kể cả dải Glossary/Proofreader/thanh trạng thái, `p.config-error` hiện, mở từ ngăn kéo Tra cứu ở cửa sổ hẹp) thuộc nợ `Chủ: Epic 12` chạy một lượt dùng thật.
