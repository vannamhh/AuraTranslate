---
title: 'Panel ẩn nhớ đúng chỗ và lưới giữ vị trí cuộn'
type: 'bugfix'
ticket: ''
created: '2026-10-31'
status: 'built'
baseline_revision: '995e479ec905c79462ae465db54e23f1ddd9f5d3'
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

**Problem:** Ẩn panel là `removePanel` thật nên panel rời cây và bị tháo: tắt rồi bật Đối chiếu đưa lưới về đầu Chương (600 → 0); tắt hai panel rồi bật không theo thứ tự ngược cho ra `Đối chiếu | Đề xuất AI | Tra cứu`; mở lại app thì panel ẩn mất chỗ. Ba mục `Chủ: Epic 14` của Story 14.8 ở `deferred-work.md`.

**Approach:** Giữ `removePanel`. Lúc ẩn ghi lại cây đủ panel, lúc bật tính chỗ từ cây đó sau khi bỏ các panel còn ẩn, và lưu cây vào một trường mới của JSON bố cục. Lưới nhớ `scrollTop` theo Chương rồi trả lại khi gắn lại.

## Boundaries & Constraints

**Decision (Ice 2026-10-31):** Chọn nhớ cây đầy đủ, không dùng `setVisible` của dockview: `setVisible` để panel ẩn vẫn gắn, buộc tầng hẹp giữ một cơ chế ẩn thứ hai và sửa 11 chỗ đọc `getPanel`.

**Decision (Ice 2026-10-31):** Làm cả ba mục trong một plan.

**Always:** `hiddenLayout` trong JSON là tuỳ chọn và được kiểm kiểu lúc đọc; thiếu hoặc hỏng thì rơi về sentinel hiện có, không ném. Chỗ tính từ cây chỉ dùng khi neo còn trong dock, nếu không thì đi nhánh dự phòng hiện có. Ba chỗ xoá `hidden` xoá luôn cây đã nhớ, trừ khi JSON vừa nạp có `hiddenLayout`. Tầng dùng chung đường `hidePanel`/`showPanel`, và lượt gộp của tầng vẫn lưu dạng chưa gộp.

**Never:** Dùng `setVisible`. Đổi `autoHiddenIds`, cờ chặn ghi hay thứ tự hy sinh của tầng. Đọc `Date.now()` hay thêm lịch ghi mới. Thêm việc nặng vào `onGridScroll`: chỉ ghi một biến module, vì nó chạy trên mọi lượt cuộn. Giữ caret hay tiêu điểm qua lượt gắn lại lưới.

## I/O & Edge-Case Matrix

| Scenario | Input / State | Expected Output / Behavior | Error Handling |
|----------|--------------|---------------------------|----------------|
| Không theo thứ tự ngược | `Đối chiếu \| (Tra cứu / AI)`: tắt Tra cứu, tắt AI, bật Tra cứu, bật AI | Về đúng cây và cỡ cũ | Không lỗi |
| Theo thứ tự ngược | Tắt Tra cứu, tắt AI, bật AI, bật Tra cứu | Về đúng cây và cỡ cũ | Không lỗi |
| Đối chiếu trong chuỗi | Tắt Đối chiếu, tắt Tra cứu, bật Đối chiếu, bật Tra cứu | Về đúng cây và cỡ cũ | Không lỗi |
| Chung tab | Tra cứu và AI chung nhóm, tắt AI rồi bật | Về tab cũ | Không lỗi |
| Mở lại app | Tắt một panel, đóng, mở, bật | Panel về đúng chỗ và cỡ cũ | Không lỗi |
| Mở lại, hai panel ẩn | Tắt hai panel, đóng, mở, bật cả hai theo thứ tự bất kỳ | Về đúng cây | Không lỗi |
| JSON cũ | Bố cục đã lưu không có `hiddenLayout` | Hành vi hiện tại: bên phải panel đầu tiên | Không lỗi |
| `hiddenLayout` hỏng | Trường sai hình dạng | Như JSON cũ, bố cục còn lại vẫn dựng | Bỏ qua trường, ghi `console.error` |
| Đổi preset khi có panel ẩn | Tắt một panel, bấm preset, tắt panel khác, bật | Chỗ theo preset mới, không theo cây cũ | Không lỗi |
| Preset người dùng có panel ẩn | Lưu preset lúc một panel ẩn, áp lại, bật | Panel về chỗ lúc lưu | Không lỗi |
| Kéo panel lúc có panel ẩn | Tắt A, kéo B, bật A | Panel hiện, neo còn sống | Chỗ gần đúng, chấp nhận |
| Tầng | `full → narrow → full` | Cây và cỡ về như trước | Các ca tầng hiện có giữ xanh |
| Cuộn lưới | Cuộn 600, tắt rồi bật Đối chiếu | `scrollTop` về 600 | Không lỗi |
| Cuộn khi đổi Chương | Tắt Đối chiếu, đổi Chương, bật | Lưới ở đầu Chương mới | Không lỗi |
| Cuộn khi đổi preset | Cuộn 600, bấm preset | `scrollTop` về 600 | Không lỗi |

</frozen-after-approval>

## Code Map

- `src/layout/WorkspaceDock.vue` -- `hidden` dòng 174, `hidePanel` 381, `showPanel` 398, `spotPosition` 326, `jsonForPersist`, `applyLayoutJson` 838–861, `restore` 980–1001. Ba chỗ xoá `hidden`: dòng 246, 272, 855; `restore` đọc cây, không xoá. `MAX_VALUE_BYTES` 64 KiB ở `store.rs` đủ chỗ: chỉ thêm `grid`, không thêm `panels`. `currentLayoutJson` đi qua `jsonForPersist`, nên preset người dùng tự có trường mới.
- `src/layout/dockTree.ts` -- thêm hàm thuần: bỏ panel khỏi cây, tính chỗ của một panel từ cây (`within` khi lá còn panel khác, ngược lại `findTreeSpot`), đọc cỡ từ cây, kiểm kiểu `hiddenLayout`. `besideKind` và `unmergeForPersist` dùng lại.
- ⚠️ Cỡ trong JSON: lá có `size` theo trục của nhánh cha, nhánh có `size` theo trục vuông góc. Cỡ đo bằng `groupSizeOf` khi đã có panel khác ẩn là cỡ sai (Đề xuất AI đo cả cột), nên lấy từ cây. Happy-dom cho `size` 0: coi 0 là không có và dùng `groupSizeOf`, để ca D2, D3 giữ nguyên.
- ⚠️ `tierMerged`: `jsonForPersist` phải đưa `hiddenLayout` qua `unmergeForPersist` khi đang gộp.
- `src/panels/GridPanel.vue` -- `onGridScroll` dòng 1902, `.grid-scroll` dòng 2022, `chapterId` dòng 1897. Lưới chỉ có `.grid-scroll` khi `hasSegments`. ⚠️ `onBeforeUnmount` có thể đọc `scrollTop` 0 vì dockview đã tháo phần tử, nên ghi ở `onGridScroll`.
- `tests/frontend/workspaceDockTier.test.ts` -- `shapeOf`, khung mount dock thật; ca `off A, off B, on A` và ca `a panel hidden in the saved layout` đang khoá hành vi cũ, viết lại theo Matrix.
- `tests/frontend/dockTree.test.ts`, `e2e/specs/panel-toggle-mouse.e2e.mjs`, `_bmad-output/implementation-artifacts/14-8-ban-do/chuot-that.e2e.mjs`.

## Tasks & Acceptance

**Execution:**
- [x] `src/layout/dockTree.ts` -- thêm bốn hàm thuần theo Code Map -- phần tính toán kiểm được không cần dock.
- [x] `src/layout/WorkspaceDock.vue` -- ghi cây ở `hidePanel`, tính chỗ ở `showPanel`, đọc ở `restore` và `applyLayoutJson`, ghi `hiddenLayout` ở `jsonForPersist`; sửa comment sentinel ở `restore` -- chỗ của panel ẩn.
- [x] `src/panels/gridScrollMemory.ts` -- tệp mới: ghi và đọc `scrollTop` theo `chapterId`, biến module thường -- sống qua lượt tháo `GridPanel`.
- [x] `src/panels/GridPanel.vue` -- ghi ở `onGridScroll`, trả lại khi `.grid-scroll` gắn sau khi dock đã đặt cỡ -- mục nợ cuộn.
- [x] `tests/frontend/dockTree.test.ts`, `tests/frontend/workspaceDockTier.test.ts` -- phủ mọi hàng Matrix về chỗ, cỡ và JSON -- Matrix.
- [x] `tests/frontend/gridScrollMemory.test.ts` -- mount `GridPanel`, cuộn, tháo, gắn lại, đổi Chương -- Matrix.
- [x] `e2e/specs/panel-toggle-mouse.e2e.mjs` -- thêm chuỗi không theo thứ tự ngược và cuộn lưới, đọc định danh panel và hình học -- WKWebView thật.
- [x] `_bmad-output/implementation-artifacts/14-8-ban-do/` -- đo lại, cập nhật `ket-qua.md` -- số 600 → 0 đổi.
- [x] `_bmad-output/initiative-auratranslate/deferred-work.md` -- đóng ba mục bằng `→ ✅ ĐÃ ĐÓNG 2026-10-31 (Story 14.8)` -- sổ nợ.

**Acceptance Criteria:**
- Given app thật, when tắt hai panel rồi bật theo cả hai thứ tự, then định danh, vị trí và cỡ cả ba nhóm trong sai số 2 px của lúc đầu.
- Given lưới cuộn 600 trên app thật, when tắt rồi bật Đối chiếu, then `scrollTop` về 600 trong sai số 2 px.
- Given JSON bố cục có một panel ẩn, when dựng lại dock từ JSON đó rồi bật panel, then cây bằng cây lúc đầu.
- Given gỡ thật từng mối nối, when chạy ca của nó, then ca đỏ đúng lý do.

## Implementation Notes

- `hiddenTree` dạng sống để đặt panel, `hiddenTreeSaved` dạng ghi đĩa; `onDidMovePanel` xoá cả hai, nên kéo panel rơi về chỗ từng panel.
- `resetEditorPanel()` là ranh giới của trí nhớ cuộn: chạy khi đổi Chương hoặc Tác phẩm, không chạy khi dock gắn lại lưới hay đổi preset.
- Vitest đủ bộ ba lượt thoát 0, 1, 0: lượt giữa `EnvironmentTeardownError` ở `editorConfirmSegment.test.ts`, mọi ca qua, không tái hiện ở hai lượt sau.
- Ca mở lại app dùng `window.location.reload()` trên WKWebView: chờ `bootstrap_config` có `hiddenLayout` và đúng panel, không ngủ. Khởi động lại cả tiến trình thuộc lượt dùng thật của Ice.

## Plan Change Log

- Lượt review 1, không loopback: hai tiền đề Code Map sai. Happy-dom cho `size` 100, không phải 0, nên ca D2 không giữ nguyên được: `sizeFromTree` được ưu tiên, D2 nay so với cỡ đọc từ cây chụp trước lượt đổi cỡ. Trí nhớ cuộn khoá theo `chapterId` không đủ vì mỗi `project.db` đánh số riêng: thay bằng xoá ở `resetEditorPanel()`. Verification thiếu `check:panel-refs`, thêm vào. KEEP: `removeViews` + `spotInTree`, trường `hiddenLayout`, thử lại tối đa 10 khung hình khi trả cuộn.

## Review Triage Log

Lượt 1, `quick`: 1 high · 3 medium · 8 low; 12 patch, 0 reject.

| Verdict | Route | Phát hiện | Bằng chứng |
|---|---|---|---|
| high | patch | `check:panel-refs` đỏ, chặn `pre-push` | Chạy cổng thấy `remembered` không có `reset*`. Thêm `resetGridScrollMemory()`. |
| medium | patch | Vị trí cuộn lan sang Tác phẩm khác | `chapter.id` đánh số riêng theo `project.db`, không gì xoá `remembered`. Xoá ở `resetEditorPanel()`. |
| medium | patch | Trả cuộn đè cuộn theo caret khi mở Chương | `.grid-scroll` dựng lại mỗi lần mở Chương. Cùng bản vá trên. |
| medium | patch | Tắt rồi bật ở tầng `short` tách nhóm đã gộp | `fullTreeNow` chụp dạng chưa gộp. Giữ `hiddenTree` dạng sống, `hiddenTreeSaved` dạng lưu. |
| low | patch | Cây cũ đè chỗ mới sau khi kéo panel | `hiddenTree` chỉ chụp ở lượt ẩn đầu. Xoá cây ở `onDidMovePanel`. |
| low | patch | `hiddenLayout` hỏng khi gộp sau lúc chụp | `unmergeForPersist` chạy trên cây chụp trước lượt gộp. Cùng bản vá hai dạng. |
| low | patch | Ca preset không đỏ được | Áp đúng preset mặc định nên hai cây bằng nhau. Đổi `layout.preset_columns`. |
| low | patch | Ca đổi Chương không đổi Chương | Gọi `rememberGridScroll` bằng tay. Đi qua `resetEditorPanel()`. |
| low | patch | Thiếu ca kéo panel, cuộn khi đổi preset, cỡ ở hai ca | Đối chiếu Matrix. Thêm ca, chặn cỡ qua spy `setSize`. |
| low | patch | D2 khoá số 100 của happy-dom | Xem Plan Change Log. So với `sizeFromTree`. |
| low | patch | Đóng mục cuộn khi chưa đo lại bàn đo | `ket-qua.md` ghi bảng cũ. Đo lại ở lượt kiểm cuối. |
| low | patch | Ba lỗi đặt comment | Import chen giữa, comment ba dòng, doc comment hàm riêng. |

## Verification

**Commands:**
- `npm run check:commands`, `check:i18n`, `check:tokens`, `check:layout`, `check:panel-refs`, `check:debt-owner`, `check:lint`, `npx vue-tsc --noEmit` -- exit 0.
- `npx vitest run tests/frontend/dockTree.test.ts tests/frontend/workspaceDockTier.test.ts tests/frontend/gridScrollMemory.test.ts` -- exit 0.
- `npm run build`, `cargo build --locked --features wdio --manifest-path src-tauri/Cargo.toml`, `npx wdio run e2e/wdio.conf.mjs --spec e2e/specs/panel-toggle-mouse.e2e.mjs` -- xanh, ngoài sandbox. ⚠️ Dịch `wdio` sau `cargo test`, vì `cargo test` ghi đè binary.

**Manual checks (if no CLI):**
- Mở lại app thật không tự động hoá được trong wdio: ca Mở lại app kiểm bằng dock thật trong Vitest, phần còn lại thuộc lượt dùng thật của Ice trước khi Epic 14 `done`.
