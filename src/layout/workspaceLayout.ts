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
 * chỉ dùng bốn trong đó ở hai preset. Khai lại giữ tệp này **không có một dòng import
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

/** Bốn panel của Workspace. ⚠️ CŨNG là bốn điểm vào focus (`FOCUS_OWNERS`). */
export type PanelId = 'panel.source' | 'panel.lookup' | 'panel.ai_translation' | 'panel.editor'

/**
 * Thứ tự KHAI BÁO, không phải thứ tự hiển thị.
 *
 * ⚠️ Vòng xoay `focus.next_panel` đi theo **thứ tự bố cục thật** (trái→phải, trên→dưới của
 * lưới đang hiện), không theo mảng này — xem `layoutOrder()` ở `WorkspaceDock.vue`.
 * Mảng này chỉ để đếm và để đối chiếu tính đầy đủ.
 */
export const PANEL_IDS: readonly PanelId[] = [
  'panel.source',
  'panel.lookup',
  'panel.ai_translation',
  'panel.editor',
]

/**
 * Khoá `vi.json` của tiêu đề từng panel.
 *
 * Không chuỗi đã dịch — NFR16 nói mọi văn bản hiển thị sống ở `vi.json` và chỉ ở đó.
 * `PanelTab.vue` và `PanelFrame.vue` là chỗ `t()` chạy.
 */
export const PANEL_TITLE_KEYS: Readonly<Record<PanelId, string>> = {
  'panel.source': 'panel.source.title',
  'panel.lookup': 'panel.lookup.title',
  'panel.ai_translation': 'panel.ai_translation.title',
  'panel.editor': 'panel.editor.title',
}

/** Tên component nội dung đã đăng ký với dockview, theo panel. */
export const PANEL_COMPONENTS: Readonly<Record<PanelId, string>> = {
  'panel.source': 'source',
  'panel.lookup': 'lookup',
  'panel.ai_translation': 'aiTranslation',
  'panel.editor': 'editor',
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
 * 🔴 PRESET MẶC ĐỊNH — lưới 2×2 (AC6, UX-DR13).
 *
 *   ┌───────────────┬───────────────┐
 *   │  Nguyên văn   │   Bản dịch    │   ← hàng trên
 *   ├───────────────┼───────────────┤
 *   │   Tra cứu     │  Đề xuất AI   │   ← hàng dưới
 *   └───────────────┴───────────────┘
 *
 * ⚠️ `Nguyên văn` và `Bản dịch` CẠNH NHAU THEO CHIỀU NGANG, và đó không phải khẩu vị.
 * UX-DR13 nêu lý do: *"đối chiếu ngang là thao tác lặp hàng trăm lần mỗi Chương"* — cũng
 * chính là lý do Sync Scrolling (FR20) tồn tại. Đừng xếp dọc "cho cân".
 */
const GRID_2X2: readonly PanelPlacement[] = [
  { id: 'panel.source', reference: null, direction: null },
  { id: 'panel.editor', reference: 'panel.source', direction: 'right' },
  { id: 'panel.lookup', reference: 'panel.source', direction: 'below' },
  { id: 'panel.ai_translation', reference: 'panel.editor', direction: 'below' },
]

/**
 * Preset thứ hai — bốn cột `Nguyên văn | Tra cứu | Đề xuất AI | Bản dịch` (AC5, UX-DR13).
 *
 * ⚠️ Ở đây `Nguyên văn` và `Bản dịch` **ở hai đầu**, và đó là chủ ý của UX-DR13: bốn cột
 * là bố cục cho lượt *tra cứu dày*, không phải cho lượt *đối chiếu*. Ai cần đối chiếu
 * thì bấm về 2×2 — đó là điều hai preset tồn tại để làm.
 */
const FOUR_COLUMNS: readonly PanelPlacement[] = [
  { id: 'panel.source', reference: null, direction: null },
  { id: 'panel.lookup', reference: 'panel.source', direction: 'right' },
  { id: 'panel.ai_translation', reference: 'panel.lookup', direction: 'right' },
  { id: 'panel.editor', reference: 'panel.ai_translation', direction: 'right' },
]

export const LAYOUT_PRESETS: readonly LayoutPreset[] = [
  { id: 'layout.preset_grid', labelKey: 'command.layout.preset_grid', placements: GRID_2X2 },
  { id: 'layout.preset_columns', labelKey: 'command.layout.preset_columns', placements: FOUR_COLUMNS },
]

/** Preset áp cho một kho rỗng — cùng luật với `DEFAULT_THEME` / `DEFAULT_MODE`. */
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

/** Cặp KHÔNG BAO GIỜ nhường. Hai tập này rời nhau và hợp lại đúng bốn panel (AC7). */
export const NEVER_SACRIFICED: readonly PanelId[] = ['panel.source', 'panel.editor']

/**
 * Panel kế tiếp phải nhường chỗ, cho một tập panel đang hiện.
 *
 * 🔴 **HÀM THUẦN.** Nó **không** đọc `window.innerWidth`, không `matchMedia`, không
 * biết một cái ngưỡng nào tồn tại. Đó chính là điều kiện để Story 4.12 chỉ phải viết
 * *"khi ngưỡng X chạm thì gọi hàm này"* thay vì mổ lại bố cục — và là điều `epics.md:1617`
 * đòi bằng chữ.
 *
 * @returns panel kế tiếp trong thứ tự hy sinh mà **đang hiện**, hoặc `null` khi không
 *   còn gì được phép nhường. `null` **không** phải "hy sinh `panel.source`" — nó là
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
