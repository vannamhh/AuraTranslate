/**
 * Bố cục Workspace — **tầng THUẦN**. Story 1.14 · AC3 · AC5 · AC6 · AC7 · FR17 · FR18.
 *
 * ─────────────────────────────────────────────────────────────────────────────
 * ⚠️ TỆP NÀY PHẢI NẠP ĐƯỢC BẰNG NODE THUẦN — và đó là một ràng buộc, không phải may mắn
 * ─────────────────────────────────────────────────────────────────────────────
 * `scripts/check-layout.mjs` `import()` thẳng tệp này (type-stripping, Node ≥ 22.18) rồi
 * gọi [`nextToSacrifice`] thật. Nhờ vậy ba mệnh đề của AC7 được cưỡng chế trên **chính
 * hàm của sản phẩm**, không phải trên một bản chép trong script.
 *
 * ⇒ Luật "erasable-only", y hệt `src/commands/**`:
 *   không `import` giá trị của `vue` · `dockview` · `@tauri-apps/api`;
 *   không `enum`, không `namespace`, không parameter property.
 *
 * ⚠️ Kiểu vị trí ở đây được KHAI LẠI thay vì `import type` từ `dockview`: `Direction` của
 * dockview có năm giá trị (`left` · `right` · `above` · `below` · `within`) và story này
 * chỉ dùng hai trong đó ở hai preset *(🔵 2026-08-14: bốn → hai, sau khi bốn panel thành
 * ba — `left`/`above` không còn chỗ dùng)*. Khai lại giữ tệp này **không có một dòng import
 * nào** — tức không có gì để hỏng khi Node bóc kiểu, và không có gì để một lượt nâng phiên
 * bản dockview làm trôi. Chỗ nối kiểu thật là `WorkspaceDock.vue`, nơi `vue-tsc` kiểm.
 *
 * ─────────────────────────────────────────────────────────────────────────────
 * CÁI GÌ KHÔNG Ở ĐÂY
 * ─────────────────────────────────────────────────────────────────────────────
 * Không một `matchMedia`, không một ngưỡng kích thước màn hình, không ngăn kéo,
 * không "rút Tra cứu về thanh trạng thái". Cả bốn là **Story 4.12** và `epics.md:1617`
 * cấm tường minh việc đóng chúng ở đây. Story này giao đúng **CƠ CHẾ**: một thứ tự hy sinh
 * khai được, kiểm được bằng máy, và một hàm thuần không đọc kích thước cửa sổ — điều
 * kiện để 4.12 **chỉ phải nối ngưỡng vào**, không phải mổ lại bố cục.
 */

/**
 * **BA** panel của Workspace. ⚠️ CŨNG là ba điểm vào focus (`FOCUS_OWNERS`).
 *
 * 🔵 **CẬP NHẬT 2026-08-14 (Story 2.5b) — mệnh đề "bốn panel" đã HẾT ĐÚNG.**
 * `panel.source` + `panel.editor` gộp thành **một** bề mặt `panel.grid` *(lưới hai cột đối
 * chiếu)*. Lý do không phải gọn gàng: UX-DR13 đòi **nguyên văn và bản dịch của cùng một câu
 * trên cùng một HÀNG**, và hai panel rời nhau không diễn đạt được mệnh đề đó — mắt người
 * dùng phải tự làm việc ghép hàng.
 *
 * 🔴 **Panel id KHÔNG nằm trên đĩa** *(Quyết định #5(a), Ice ký 2026-08-14)* — nên đổi tên ở
 * đây **không** làm mồ côi thứ gì. Thứ **có** nằm trên đĩa là **`PresetId`** và **command
 * id**; xem [`LAYOUT_PRESETS`].
 */
export type PanelId = 'panel.grid' | 'panel.lookup' | 'panel.ai_translation'

/**
 * Thứ tự KHAI BÁO, không phải thứ tự hiển thị.
 *
 * ⚠️ Vòng xoay `focus.next_panel` đi theo **thứ tự bố cục thật** (trái→phải, trên→dưới của
 * lưới đang hiện), không theo mảng này — xem `layoutOrder()` ở `WorkspaceDock.vue`.
 * Mảng này chỉ để đếm và để đối chiếu tính đầy đủ.
 */
export const PANEL_IDS: readonly PanelId[] = ['panel.grid', 'panel.lookup', 'panel.ai_translation']

/**
 * Khoá `vi.json` của tiêu đề từng panel.
 *
 * Không chuỗi đã dịch — NFR16 nói mọi văn bản hiển thị sống ở `vi.json` và chỉ ở đó.
 * `PanelTab.vue` và `PanelFrame.vue` là chỗ `t()` chạy.
 */
export const PANEL_TITLE_KEYS: Readonly<Record<PanelId, string>> = {
  'panel.grid': 'panel.grid.title',
  'panel.lookup': 'panel.lookup.title',
  'panel.ai_translation': 'panel.ai_translation.title',
}

/** Tên component nội dung đã đăng ký với dockview, theo panel. */
export const PANEL_COMPONENTS: Readonly<Record<PanelId, string>> = {
  'panel.grid': 'grid',
  'panel.lookup': 'lookup',
  'panel.ai_translation': 'aiTranslation',
}

/** Hướng đặt một panel so với panel tham chiếu. Tập con của `Direction` bên dockview. */
export type PlacementDirection = 'right' | 'below' | 'left' | 'above'

/**
 * Một bước dựng bố cục. `reference === null` nghĩa là panel đầu tiên — nó chiếm cả lưới,
 * và những bước sau cắt lưới đó ra.
 */
export type PanelPlacement = {
  readonly id: PanelId
  readonly reference: PanelId | null
  readonly direction: PlacementDirection | null
}

export type PresetId = 'layout.preset_grid' | 'layout.preset_columns'

export type LayoutPreset = {
  readonly id: PresetId
  readonly labelKey: string
  readonly placements: readonly PanelPlacement[]
}

/**
 * 🔴 PRESET MẶC ĐỊNH — **bố cục Ⓑ-2**: lưới bên trái, chiếm TOÀN chiều cao (AC6, UX-DR13).
 *
 *   ┌───────────────┬───────────────┐
 *   │               │   Tra cứu     │
 *   │     Lưới      ├───────────────┤
 *   │               │  Đề xuất AI   │
 *   └───────────────┴───────────────┘
 *
 * ⚠️ Đối chiếu ngang **đã đi vào trong lưới** — nó là hai cột của cùng một hàng, không còn
 * là hai panel cạnh nhau. Đó là toàn bộ lý do lượt correct-course 2026-08-14 lật hình dạng:
 * *"đối chiếu ngang là thao tác lặp hàng trăm lần mỗi Chương"* (UX-DR13), và hai panel rời
 * nhau bắt mắt người dùng tự ghép hàng.
 *
 * ⚠️ Lưới **toàn chiều cao** là điều kiện để một hàng dài không bị cắt — cái giá là cột hẹp
 * hơn, và cái giá đó có một phép đo chưa chạy *(chiều cao hàng khi bật Hán Việt song song —
 * Task 7 của Story 2.5b)*.
 */
const B2_GRID_LEFT: readonly PanelPlacement[] = [
  { id: 'panel.grid', reference: null, direction: null },
  { id: 'panel.lookup', reference: 'panel.grid', direction: 'right' },
  { id: 'panel.ai_translation', reference: 'panel.lookup', direction: 'below' },
]

/**
 * Preset thứ hai — **bố cục Ⓑ-1**: lưới chiếm cả bề ngang ở trên, hai panel tra cứu ở dưới.
 *
 * ⚠️ Ⓑ-1 là bố cục cho lượt **đọc hàng dài** *(cột rộng gấp đôi Ⓑ-2)*; Ⓑ-2 là bố cục cho
 * lượt **tra cứu dày**. Đó là điều hai preset tồn tại để làm.
 */
const B1_GRID_TOP: readonly PanelPlacement[] = [
  { id: 'panel.grid', reference: null, direction: null },
  { id: 'panel.lookup', reference: 'panel.grid', direction: 'below' },
  { id: 'panel.ai_translation', reference: 'panel.lookup', direction: 'right' },
]

/**
 * ═══════════════════════════════════════════════════════════════════════════════
 * 🔴 HAI CÁI TÊN DƯỚI ĐÂY LÀ **LỊCH SỬ**, KHÔNG PHẢI MÔ TẢ — NGHĨA Ở BẢNG NGAY DƯỚI
 * ═══════════════════════════════════════════════════════════════════════════════
 * 🔵 **CẬP NHẬT 2026-08-14 (Story 2.5b, Quyết định #5(a) do Ice ký).**
 *
 * | `PresetId` | Nghĩa TRƯỚC 2.5b | Nghĩa TỪ 2.5b |
 * |---|---|---|
 * | `layout.preset_grid` | lưới 2×2 bốn panel | **Ⓑ-2** — lưới trái toàn chiều cao *(mặc định)* |
 * | `layout.preset_columns` | bốn cột | **Ⓑ-1** — lưới cả bề ngang ở trên |
 *
 * 🔴 **Vì sao KHÔNG đổi tên id, dù `preset_columns` không còn tả đúng thứ nó dựng:**
 *   1. `PresetId` **nằm trên đĩa** — `ScopeKind::LayoutPreset` (`kinds.rs:213`) và bố cục
 *      đang hiển thị trong `ScopeKind::AppConfig` (`WorkspaceMode.vue:56-73`).
 *      `presetById()` trả `undefined` cho một id lạ.
 *   2. **Command id cũng nằm trên đĩa** — Story 1.21 cho gán lại phím, và bảng `keybinding`
 *      khoá theo **command id**. Đổi `layout.preset_grid` thành một tên mới làm **mồ côi**
 *      phím tắt người dùng đã gán, **im lặng**. Không cổng nào đỏ.
 *
 * ⇒ Cái giá đã chọn là **một cái tên không tả đúng**; cái giá bị loại là **dữ liệu người
 * dùng mất im lặng**. Đường (b) *(đổi id + một bước di trú)* đắt hơn và không mua thêm gì
 * ngoài một cái tên đẹp.
 *
 * ⚠️ Preset **bốn cột** đã RÚT (`epics.md:539`) — nó tách `Nguyên văn` khỏi `Bản dịch`, thứ
 * không còn tồn tại. Đừng dựng lại nó khi đọc thấy chữ `columns`.
 */
export const LAYOUT_PRESETS: readonly LayoutPreset[] = [
  { id: 'layout.preset_grid', labelKey: 'command.layout.preset_grid', placements: B2_GRID_LEFT },
  { id: 'layout.preset_columns', labelKey: 'command.layout.preset_columns', placements: B1_GRID_TOP },
]

/**
 * Preset áp cho một kho rỗng — cùng luật với `DEFAULT_THEME` / `DEFAULT_MODE`.
 *
 * ⚠️ Giá trị **không đổi** qua lượt lật của 2.5b, nhưng **nghĩa thì đổi**: nó nay trỏ vào
 * **Ⓑ-2**, không phải lưới 2×2. AC6 đòi Ⓑ-2 là mặc định — xem bảng ở [`LAYOUT_PRESETS`].
 */
export const DEFAULT_PRESET_ID: PresetId = 'layout.preset_grid'

export function presetById(id: string): LayoutPreset | undefined {
  return LAYOUT_PRESETS.find((p) => p.id === id)
}

/**
 * ═══════════════════════════════════════════════════════════════════════════════
 * 🔴 THỨ TỰ HY SINH — AC7. CHÉP NGUYÊN VĂN `epics.md:1616`, KHÔNG DIỄN GIẢI LẠI
 * ═══════════════════════════════════════════════════════════════════════════════
 *
 * > **Đề xuất AI nhường trước · Tra cứu nhường sau nhưng rút về thanh trạng thái, không
 * > bao giờ mất hẳn · cặp `Nguyên văn | Bản dịch` không bao giờ nhường.**
 *
 * ⚠️ UX-DR15 gọi đây là một **QUYẾT ĐỊNH**, không phải một số hiệu chỉnh được. Bốn
 * ngưỡng kích thước (`1100×820` · `<820 cao` · `<1100 rộng hoặc <700 cao` · `<860 rộng`)
 * là số, và chúng thuộc **Story 4.12**. Thứ tự thì thuộc về đây.
 *
 * ⚠️ *"rút về thanh trạng thái"* là vế mà story này KHÔNG cài — `panel.lookup` ở đây
 * chỉ **nhường**, và cái gì hiện ra thay nó là việc của 4.12. Ghi ra để 4.12 không đọc
 * mảng này thành *"Tra cứu được phép biến mất"*.
 */
export const SACRIFICE_ORDER: readonly PanelId[] = ['panel.ai_translation', 'panel.lookup']

/**
 * Panel KHÔNG BAO GIỜ nhường. Hai tập này rời nhau và hợp lại đúng **ba** panel (AC7).
 *
 * 🔵 **CẬP NHẬT 2026-08-14 (Story 2.5b) — một phần tử, không hai.** Mệnh đề của UX-DR15
 * *(«cặp `Nguyên văn | Bản dịch` không bao giờ nhường»)* **không đổi một chữ**; cái đổi là
 * cặp đó nay **là một panel**. Đây là lượt thu gọn theo cấu trúc, không một lượt nới luật.
 */
export const NEVER_SACRIFICED: readonly PanelId[] = ['panel.grid']

/**
 * Panel kế tiếp phải nhường chỗ, cho một tập panel đang hiện.
 *
 * 🔴 **HÀM THUẦN.** Nó **không** đọc `window.innerWidth`, không `matchMedia`, không
 * biết một cái ngưỡng nào tồn tại. Đó chính là điều kiện để Story 4.12 chỉ phải viết
 * *"khi ngưỡng X chạm thì gọi hàm này"* thay vì mổ lại bố cục — và là điều `epics.md:1617`
 * đòi bằng chữ.
 *
 * @returns panel kế tiếp trong thứ tự hy sinh mà **đang hiện**, hoặc `null` khi không
 *   còn gì được phép nhường. `null` **không** phải "hy sinh `panel.grid`" — nó là
 *   *"đã hết chỗ để nhường, phải giải bằng cách khác"*.
 */
export function nextToSacrifice(visible: readonly string[]): PanelId | null {
  for (const id of SACRIFICE_ORDER) {
    if (visible.includes(id)) return id
  }
  return null
}

/**
 * Panel kế tiếp phải được trả lại khi có thêm chỗ — **nghịch đảo** của [`nextToSacrifice`].
 *
 * ⚠️ Trả theo thứ tự NGƯỢC: cái nhường sau cùng được lấy lại trước. Không có nó thì một
 * lượt nới cửa sổ trả `panel.ai_translation` về trước `panel.lookup`, tức đảo đúng ưu tiên
 * mà thứ tự hy sinh vừa phát biểu.
 */
export function nextToRestore(visible: readonly string[]): PanelId | null {
  for (let i = SACRIFICE_ORDER.length - 1; i >= 0; i -= 1) {
    const id = SACRIFICE_ORDER[i] as PanelId
    if (!visible.includes(id)) return id
  }
  return null
}
