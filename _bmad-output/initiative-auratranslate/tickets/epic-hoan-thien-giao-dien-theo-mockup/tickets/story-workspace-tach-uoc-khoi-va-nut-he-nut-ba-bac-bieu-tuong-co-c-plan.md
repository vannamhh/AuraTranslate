---
title: 'Workspace tách được khối và nút: hệ nút ba bậc, biểu tượng, cỡ chữ'
type: 'feature'
ticket: '26'
created: '2026-10-26'
status: 'done'
baseline_revision: 'cf30c50430e15069b0423db4927895d3312a7c3a'
route: 'full'
route_source: 'auto'
risk: 'medium'
review: 'quick'
review_source: 'pinned'
lenses_ran: ['quick']
review_loop_iteration: 0
context:
  - '{project-root}/_bmad-output/initiative-auratranslate/ux-auratranslate/DESIGN.md'
  - '{project-root}/src/AGENTS.md'
---

<frozen-after-approval reason="human-owned intent — do not modify unless human renegotiates">

## Intent

**Problem:** Trên Workspace mọi nút là chữ 12px gạch chân bằng `outline` (1,31:1), nút khoá trông như nút bấm được, ba cấp điều khiển cùng một dáng, và khối nguồn từ điển, khối kết quả AI không có ranh giới đọc được. Ice duyệt cách sửa ở DESIGN.md 🔵 2026-10-26 và `mockups/workspace-tinh-chinh.html` (c481518).

**Approach:** Thêm token `control-rule`, nâng `ui-label`/`ui-mono` lên 12px, dựng một bộ lớp nút ba bậc dùng chung và một component biểu tượng SVG, rồi áp lên panel Đối chiếu, Tra cứu, Đề xuất AI, thanh trạng thái và thanh tiêu đề.

## Decisions (Ice, 2026-10-26)

1. Một story, không tách nền và phân khối.
2. Nút chính nền `on-surface`, chữ `surface`; không dùng `primary`.
3. Bỏ hẳn câu auto-save khỏi thanh trạng thái và xoá key `status.autosave_hint`; điều này lật quyết định 6 của plan 12.7.
4. `Ⓑ-2`/`Ⓑ-1` thành biểu tượng sơ đồ bố cục.
5. Không đổi cổng cho 3:1 của `control-rule`: không thêm ngưỡng hay cặp vào `contrast.pairs`; số đo trong DESIGN.md là bằng chứng.
6. Nền câu đang sửa giữ như hiện có, chỉ trên ô bản dịch (`.cell-tgt.editing`); không đụng lưới.

## Boundaries & Constraints

**Always:**
- Màu và cỡ chữ chỉ qua token (AD-34); không bóng, không gradient; SVG dùng `currentColor`.
- Nút chỉ có biểu tượng mang `:aria-label`/`:title` từ `t()`; trạng thái bật/tắt nói bằng hình (rỗng/đặc) và `aria-pressed`.
- `@click` vẫn đúng một `dispatch('<id>')`.
- Nút khoá khác dáng nút bấm được (viền đứt), không chỉ khác con trỏ.
- "Huỷ dịch"/"Huỷ lô" thế chỗ "Dịch câu này" khi đang dịch: một nút ở một chỗ, hàng nút không đổi chiều rộng do nút biến mất.
- Mọi lối vào hiện có vẫn còn, cùng lệnh, cùng phím.

**Never:**
- Không sửa lớp phủ, Library, Chế độ đọc, nút `.btn` cục bộ của Library.
- Không đổi độ dày hay kiểu vạch lề segment (Story 2.14), không sửa `GridBlock.vue`.
- Không thêm dependency, không thư viện biểu tượng.
- Không xoá key i18n mà còn lời gọi `t()` tới nó (Kiểm E không bắt lời gọi có tham số).

## I/O & Edge-Case Matrix

| Scenario | Input / State | Expected Output / Behavior | Error Handling |
|----------|--------------|---------------------------|----------------|
| Ghim | Mục chưa ghim → bấm | Biểu tượng rỗng → đặc, `aria-pressed` false → true, nhãn truy cập "Ghim" → "Bỏ ghim" | Lỗi ghi ghim: banner hiện có |
| Dịch câu | Bấm "Dịch câu này" | Nút đó thành "Huỷ dịch" cùng chỗ tới khi xong/huỷ | Lỗi: dòng alert hiện có |
| Lô | Lô đang chạy | Cùng chỗ hiện "Huỷ lô" | — |
| Bố cục | Review Mode hoặc Attribution mở | Hai nút bố cục khoá, dáng viền đứt | — |
| Theme tối | Đổi theme | `control-rule` `#807868`, khe 2px giữa panel giữ nguyên | — |

</frozen-after-approval>

## Code Map

- `src/tokens/tokens.json` -- `colors.light`/`colors.dark` thêm `control-rule`; `roles.stroke`; typography `ui-label` (:572) `ui-mono` (:581).
- `scripts/check-tokens.mjs` -- `EXPECTED_COLORS_LIGHT/DARK` (:141, :182), `EXPECTED_COUNTS.colorsPerTheme` 38→39 (:313), `EXPECTED_ROLES.stroke` (:330), `EXPECTED_TYPOGRAPHY` ui-label/ui-mono (:251-258).
- `src/tokens/index.ts:71-136` -- `applyTheme()` tự sinh `--color-control-rule`; không cần sửa.
- `src/tokens/reset.css` -- CSS toàn cục duy nhất; nơi đặt lớp nút dùng chung (hoặc tệp mới cạnh nó, nạp ở `main.ts`).
- `src/panels/LookupPanel.vue` -- tab con :386-412 (CSS 1062-1080); `lookup-sources-attr` :432 → nút biểu tượng ⓘ; ghim đầu :464 (CSS 983, 1278-1295); scope :719/726; clear :734; pointer :591.
- `src/panels/LookupRecord.vue` -- dải đầu nguồn `.lookup-source-head` :102 (CSS 216-225) → 32px nền lùi; thu gọn :110; ghim mục :150 (CSS 324-343).
- `src/panels/LookupSourcesMenu.vue:91` -- `.sources-trigger` → nút phụ nhỏ.
- `src/panels/AiTranslationPanel.vue` -- hàng nút :474-510, rule chung 8 lớp 879-917; `.ai-translate-cancel` bị dùng lại cho 3 lối không phải huỷ (:411, :422, :650) → đổi tên lớp; `canCancelAiTranslate` :173; kết quả `.ai-translate-text` :605 → khung; lô :625-741.
- `src/panels/GridPanel.vue:1868-1899` -- tab Trung/Hán Việt + `.view-toggle` → nút phân đoạn (CSS 1955-2005).
- `src/panels/PanelTab.vue:92` -- `.tab-chord` → chip phím; `src/layout/dockview-theme.css:185` nét đáy thanh tab.
- `src/StatusBar.vue` -- autosave :243 và :652 (xoá), TM :225/:650 → rút gọn + `title`; nút :670/:680/:689 (CSS 821-843).
- `src/App.vue:242-270` -- `.layout-switch-btn` → nút phân đoạn biểu tượng; `src/TitlebarToolsMenu.vue:83` → nút chìm.
- `src/i18n/vi.json` -- xoá `status.autosave_hint`, `titlebar.layout.b1/b2`; thêm khoá TM rút gọn.
- Tests: `tests/frontend/statusBarWorkspace.test.ts:273-307` (autosave), `lookupPanelBlocks.test.ts:195-215`, `attributionOverlayActions.test.ts:577`, `titlebarToolsMenu.test.ts:208` (regex trên App.vue), `aiTranslate.test.ts:306,505,718`, `aiTranslateBatch.test.ts:510,690`, `aiBatchPanelRows.test.ts:279,288`; `e2e/specs/titlebar-tools.e2e.mjs`.

## Tasks & Acceptance

**Execution (phase 1 · nền):**
- [x] `src/tokens/tokens.json`, `scripts/check-tokens.mjs` -- `control-rule` (thêm cả vào `roles.stroke`), 39 màu, cỡ 12px.
- [x] `src/tokens/` -- lớp dùng chung: nút chính/phụ/chìm/khoá, cỡ nhỏ 24px, nút biểu tượng 26px, nút phân đoạn, chip phím.
- [x] `src/AppIcon.vue` -- component SVG `currentColor`: pin, pin-on, chevron, info, layout-left, layout-top (đường vẽ lấy từ mockup).

**Execution (phase 2 · áp):**
- [x] Từng tệp trong Code Map -- đổi sang lớp dùng chung, xoá rule gạch chân cục bộ trong dòng đang sửa; Huỷ thế chỗ Chạy; ghim, ⓘ, thu gọn, bố cục thành `AppIcon`; dải đầu nguồn; khung kết quả AI; thanh trạng thái.

**Execution (phase 3 · test và đo):**
- [x] Sửa các test trong Code Map theo hành vi mới; thêm ca: nút chỉ-biểu-tượng thiếu `aria-label` thì đỏ (đối chứng: gỡ thuộc tính khỏi một nút thật); ca Huỷ thế chỗ Chạy.

**Acceptance Criteria:**
- Given Workspace mở, when nhìn bất kỳ nút nào của năm bề mặt, then nó thuộc đúng một bậc của DESIGN.md §Nút và không chữ giao diện nào dưới 12px.
- Given trình đọc màn hình, when tới nút ghim hoặc nút bố cục, then nghe tên và trạng thái nhấn.
- Given Ice, when đặt ảnh app thật 1490×935 dpr=2 sáng và tối cạnh mockup, then mỗi mục 1–6 ghi khớp hoặc có lý do.

## Implementation Notes

- Lớp dùng chung ở `src/tokens/controls.css` (nạp sau `reset.css`): `ui-btn` (+ `--primary`, `--quiet`, `--sm`), `ui-icon-btn`, `ui-seg`/`ui-seg__item`, `ui-kbd`. Trạng thái chọn của nút phân đoạn đọc `aria-pressed`/`aria-selected`, không lớp `.on`.
- Lớp móc mà test và e2e đọc (`tools-trigger`, `layout-switch(-btn)`, `lookup-scope`, `lookup-pin`, `lookup-collapse`, `view-toggle`, `tab-chord`) giữ lại, không mang CSS.
- Tab dock mở xuống thân panel: nét đáy dời từ `.dv-tabs-and-actions-container` xuống từng con và từng `.dv-tab`, `.dv-active-tab` bỏ nét đáy (`dockview-theme.css`). Chưa nhìn trong app thật.
- ⚠️ Lượt xoá CSS nút cũ của agent dựng để lại selector `.ai-panel-head` treo trước một comment, biến `.ai-surface` thành selector con không áp vào đâu, và xoá oan kiểu của `.ai-unconfigured*` và `.lookup-section-count`. Không cổng, không test nào bắt; sửa tay ở bước dựng.
- Thêm `AppIcon.vue` đẩy số `.vue` lên 66 nên `VUE_FLOOR` của `check-i18n`/`check-commands` lên 57 (`ceil(0.85 × 66)`).
- Bỏ `▾` khỏi `titlebar.tools.label` và `panel.lookup.sources_button` (thay bằng biểu tượng chevron); xoá `panel.lookup.collapse_open/closed`, `titlebar.layout.b1/b2`, `status.autosave_hint`; thêm `status.tm_short`, `panel.ai_translation.chapter_group`.
- Chưa chạy e2e (Ice đang mở `tauri dev` trên cổng 1420); chưa có ảnh app thật. Cả hai ở `deferred-work.md` §Story 12-26, Chủ: Epic 12.

## Plan Change Log

## Review Triage Log

Lượt 1 (quick): high 0 · medium 2 · low 2 · false 3 · maybe-false 0.

| Verdict | Route | Finding | Evidence |
|---|---|---|---|
| medium | patch | `.ui-seg { overflow: hidden }` cắt vòng tiêu điểm `outline-offset: 1px` của ô phân đoạn | Đúng; vòng của `.ui-seg__item` dời vào trong (`outline-offset: -2px`) |
| medium | patch | `.ui-seg__item:disabled` đứng sau rule chọn, cùng độ đặc hiệu, xoá nền ô bố cục đang chọn khi Review Mode/Attribution | Đúng; rule khoá dời lên trước rule chọn |
| low | patch | Dịch ↔ Huỷ khác chiều rộng nên nút sau xê dịch | Đúng; hai nút chung lớp `ai-run-slot` `min-width: 8.5em` |
| low | patch | `.ui-kbd` không ai dùng, `.tab-chord` khai lại cùng hình | Đúng; `.tab-chord` dùng `ui-kbd`, chỉ giữ lề và nét chữ |
| false | — | `.ai-group-label` ghép `ui-sm` với `--weight-ui-md-strong` | Cùng khuôn đã có ở `GlossaryManageOverlay.vue:1034`, `ImportPreviewOverlay.vue:1912`, `BilingualImportPreviewOverlay.vue:1181`; DESIGN.md §Khối trong panel Tra cứu ghi nhãn nhóm `ui-sm` 600 |
| false | — | `ui-label` `letterSpacing` 0.1em → 0.08em không có lý do | DESIGN.md §Bảng token typography hàng 13 (context của plan) ghi 0.08em, Ice duyệt 2026-10-26 |
| false | — | Implementation Notes trống, thiếu ảnh app thật cho AC3 | Notes ghi ở bước review; AC3 là việc của Ice, mục nợ Chủ: Epic 12 |

## Verification

**Commands:**
- `npm run check:tokens && npm run check:i18n && npm run check:commands` -- expected: xanh.
- `npx vitest run` các tệp test đã sửa -- expected: xanh, mã thoát 0, không dòng Errors.
- Đủ bộ test một lần vì chạm `scripts/check-tokens.mjs` (AGENTS.md §Tests).
- `npm run test:e2e -- --spec e2e/specs/titlebar-tools.e2e.mjs` (ngoài sandbox) -- expected: xanh.

**Manual checks:**
- Ảnh app thật cạnh `workspace-tinh-chinh.html`; Ice chạy app một lượt.
