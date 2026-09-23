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
 * 🔵 **CẬP NHẬT 2026-09-22 (Story 4.12, Phase 1) — bốn ngưỡng đó nay ĐÃ Ở DƯỚI**, xem
 * [`LAYOUT_THRESHOLDS`] và [`layoutTierFor`] cuối tệp này. Đoạn văn trên vẫn đúng — thứ tự
 * hy sinh và bốn ngưỡng kích thước là hai mối quan tâm tách rời, chỉ là hai mối quan tâm
 * đó nay cùng sống trong một tệp — nên không sửa, chỉ ghi thêm chỗ tìm.
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

/**
 * ═══════════════════════════════════════════════════════════════════════════════
 * NGƯỠNG BỐN TẦNG — Story 4.12, Task 1. Vẫn tầng THUẦN: không `window`, không
 * `matchMedia`, không DOM. `WorkspaceDock.vue` (Story 4.12, Phase 2) là nơi đọc
 * `window.innerWidth`/`innerHeight` thật rồi gọi [`layoutTierFor`] ở đây.
 * ═══════════════════════════════════════════════════════════════════════════════
 *
 * Bốn tầng, TOÀN PHẦN và đánh giá theo một thứ tự ưu tiên cố định — với mọi
 * `(width, height)` hợp lệ, đúng MỘT tầng khớp:
 *
 *   `full`        — đủ cả hai chiều: ba panel đứng nguyên vị trí.
 *   `short`       — đủ rộng, thiếu cao vừa phải: Tra cứu và Đề xuất AI gộp một
 *                   nhóm có tab (Story 4.12, Phase 2 — nhánh `within` của
 *                   `rememberSpot`), lưới không đổi.
 *   `narrow`      — thiếu rộng, hoặc quá thấp: chỉ còn lưới; Đề xuất AI ẩn hẳn,
 *                   Tra cứu rút khỏi lưới (điểm vào chuyển sang thanh trạng thái
 *                   — Quyết định 2, Story 4.12, Phase 3).
 *   `unsupported` — quá hẹp để dùng được thoải mái: cùng bố cục panel với
 *                   `narrow`, cộng một thông báo không chặn (Story 4.12, Phase 3).
 *                   Lưới vẫn hiện và dùng được — một cửa sổ hẹp là một PHIỀN TOÁI,
 *                   không phải một điểm dừng (user story gốc, đóng băng).
 *
 * 🔴 *"quá thấp"* của `narrow` là một mệnh đề RIÊNG khỏi chiều rộng: một cửa sổ ĐỦ
 * rộng (`width ≥ minFullWidth`) nhưng quá thấp (`height < minShortHeight`) vẫn rơi
 * vào `narrow`, không phải `short` hay `full` — đó là hàng *"Narrow or very short"*
 * của ma trận I/O trong spec. Và một cửa sổ dưới `minSupportedWidth` là
 * `unsupported` BẤT KỂ chiều cao — chiều rộng thắng chiều cao khi cả hai cùng tệ,
 * đó chính là thứ tự ưu tiên mà `check-layout.mjs` Kiểm E ghim bằng số.
 */
export type LayoutTier = 'full' | 'short' | 'narrow' | 'unsupported'

/** Mọi giá trị hợp lệ của [`LayoutTier`] — dùng để duyệt toàn phần trong gate và test. */
export const LAYOUT_TIERS: readonly LayoutTier[] = ['full', 'short', 'narrow', 'unsupported']

/**
 * Diện tích làm việc THẬT của lưới, tính bằng CSS px. **Không phải** kích thước cửa
 * sổ OS — spec đòi `height` đã trừ hai token chrome đang triển khai
 * (`titlebar-height` + `status-height`, đọc từ `getComputedStyle` tại runtime, không
 * viết số cứng), còn `width` thì bằng thẳng chiều rộng cửa sổ. Phép trừ đó là việc
 * của `WorkspaceDock.vue` (Phase 2) — hàm ở đây chỉ nhận số đã trừ xong.
 */
export type WorkArea = {
  readonly width: number
  readonly height: number
}

/**
 * Bốn ngưỡng của MỘT preset. Bốn TRƯỜNG, bốn TÊN — mỗi ngưỡng di chuyển ĐƯỢC một
 * mình khi lượt hiệu chỉnh thật (Task 11) đo ra một số khác, mà không phải viết lại
 * hàm [`layoutTierFor`].
 */
export type LayoutThresholds = {
  /** Chiều rộng tối thiểu cho `full`/`short` — dưới số này luôn là `narrow` (trừ khi
   *  đã `unsupported`), BẤT KỂ chiều cao. Hạt giống UX-DR15: `1100`. */
  readonly minFullWidth: number
  /** Chiều cao tối thiểu cho `full` — dưới số này (nhưng vẫn ≥ `minShortHeight`) là
   *  `short`. Hạt giống UX-DR15: `820`. */
  readonly minFullHeight: number
  /** Chiều cao tối thiểu để KHÔNG rơi vào `narrow` vì quá thấp — dưới số này là
   *  `narrow` dù chiều rộng còn thoải mái. Hạt giống UX-DR15: `700`. */
  readonly minShortHeight: number
  /** Chiều rộng tối thiểu để còn dùng được — dưới số này là `unsupported`, bất kể
   *  chiều cao. Hạt giống UX-DR15: `860`. */
  readonly minSupportedWidth: number
}

/**
 * 🔴 HẠT GIỐNG — cả hai preset cùng bốn số này hôm nay, sao chép nguyên văn từ ma
 * trận I/O của spec (2026-09-22). ĐÂY LÀ CHỖ DUY NHẤT bốn số đó xuất hiện; Task 11
 * (Phase 5, người, không phải agent) đo lại trên phần cứng thật, RIÊNG cho Ⓑ-1 và
 * Ⓑ-2 (spec Design Notes — áp lực dọc bén Ⓑ-1 trước, áp lực ngang bén Ⓑ-2 trước), và
 * sửa đúng ở đây. `check-layout.mjs` Kiểm E ghim lại bốn số này ĐỘC LẬP, nên một lượt
 * hiệu chỉnh phải sửa CẢ HAI chỗ.
 *
 * ⚠️ Hai object RIÊNG BIỆT, không phải một object dùng chung cho cả hai khoá của
 * [`LAYOUT_THRESHOLDS`] — dù giá trị hôm nay giống hệt nhau. Một object dùng chung sẽ
 * làm "hiệu chỉnh RIÊNG cho Ⓑ-1" và "hiệu chỉnh RIÊNG cho Ⓑ-2" đổi lẫn nhau.
 */
const SEEDED_THRESHOLDS_B2: LayoutThresholds = {
  minFullWidth: 1100,
  minFullHeight: 820,
  minShortHeight: 700,
  minSupportedWidth: 860,
}
const SEEDED_THRESHOLDS_B1: LayoutThresholds = {
  minFullWidth: 1100,
  minFullHeight: 820,
  minShortHeight: 700,
  minSupportedWidth: 860,
}

/** Bốn ngưỡng, theo preset. Khoá đúng hai `PresetId` của [`LAYOUT_PRESETS`]. */
export const LAYOUT_THRESHOLDS: Readonly<Record<PresetId, LayoutThresholds>> = {
  'layout.preset_grid': SEEDED_THRESHOLDS_B2,
  'layout.preset_columns': SEEDED_THRESHOLDS_B1,
}

/**
 * Thang bậc TỔNG (workArea, preset) → tầng — **HÀM THUẦN**, cùng kỷ luật với
 * [`nextToSacrifice`]: không `window`, không đọc kích thước cửa sổ thật, không biết
 * `PresetId` nào đang hiện trừ tham số truyền vào.
 *
 * `presetId` lạ (không khớp khoá nào của [`LAYOUT_THRESHOLDS`]) dùng ngưỡng của
 * [`DEFAULT_PRESET_ID`] — cùng luật với [`presetById`] không ném lỗi trên một id lạ,
 * để một bản ghi bố cục cũ/hỏng trên đĩa không làm hàm này ném ngoại lệ.
 *
 * 🔴 **`width`/`height` KHÔNG PHẢI SỐ HỮU HẠN ⇒ `null`, không phải `'full'`.** Mọi phép
 * so sánh với `NaN` trả `false`, nên một thang bậc viết bằng chuỗi `if (x < ngưỡng)` rơi
 * qua hết bốn nhánh và trả về `'full'` — tức chính lượt "không đo được kích thước" lại
 * báo bố cục THOẢI MÁI NHẤT, trên một cửa sổ có thể đang rất nhỏ. `AGENTS.md` gọi im
 * lặng-thành-rỗng là lớp lỗi trung tâm của dự án: *"một giá trị có thể KHÔNG XÁC ĐỊNH
 * được nhận một `Option`/`NULL`, không bao giờ một `0` hay một giá trị mặc định ngầm"*.
 * `Number.isFinite` chặn cả `NaN`, `Infinity`, `-Infinity` VÀ mọi giá trị không phải
 * `number` (`undefined` ép kiểu qua JS runtime) trong cùng MỘT lượt kiểm — Phase 2 đọc
 * `getComputedStyle(...).getPropertyValue(...)` rồi `parseFloat`, và một token trống
 * (`''`) cho `NaN` đúng theo cách này, không phải một trường hợp lý thuyết.
 *
 * @returns tầng hợp lệ khi cả hai chiều là số hữu hạn, hoặc `null` khi KHÔNG ĐO ĐƯỢC —
 *   `null` là *"không biết"*, không phải *"đủ chỗ"*. Gọi ở Phase 2 phải GIỮ tầng đang
 *   áp trước đó và ghi một chẩn đoán nêu nguyên nhân, không được coi `null` là `'full'`.
 */
export function layoutTierFor(workArea: WorkArea, presetId: string): LayoutTier | null {
  const { width, height } = workArea
  if (!Number.isFinite(width) || !Number.isFinite(height)) return null
  // 🔵 Story 4.12, Phase 2 — `presetId` nới từ `PresetId` sang `string`, và tra qua
  // `presetById` thay vì đánh chỉ số thẳng vào `LAYOUT_THRESHOLDS`. Bản Phase 1 khai tham số
  // là `PresetId` (một union ĐÃ ĐÓNG hai giá trị) rồi tự viết một nhánh dự phòng cho "id lạ" —
  // nhưng với tham số kiểu `PresetId`, TypeScript CHỨNG MINH chỉ số đó không bao giờ thiếu, và
  // `@typescript-eslint/no-unnecessary-condition` đọc đúng bằng chứng đó (`check:lint` đỏ,
  // đo được — bảng kiểu nói một chuyện, chú thích ngay trên nó nói chuyện khác). Nới kiểu
  // tham số để nó KHỚP với sự thật nhánh dự phòng đang phòng: dữ liệu preset trên đĩa CÓ THỂ
  // sai hình dạng (cùng lý do `presetById(id: string)` ngay trên đây không khai `PresetId`).
  // `presetById` đã có sẵn phép tra CÓ-KIỂM (`LAYOUT_PRESETS.find`, trả `LayoutPreset |
  // undefined`) — tái dùng nó thay vì đánh chỉ số trần giữ nguyên hành vi, chỉ đổi CÁCH kiểu
  // được chứng minh: `preset.id` là `PresetId` THẬT (không phải một `as` ép kiểu), nên chỉ số
  // vào `LAYOUT_THRESHOLDS` vẫn an toàn và `??`/ternary dưới đây giờ canh một `| undefined`
  // CÓ THẬT, không phải một cái TypeScript đã chứng minh không xảy ra.
  const preset = presetById(presetId)
  const t = preset === undefined ? LAYOUT_THRESHOLDS[DEFAULT_PRESET_ID] : LAYOUT_THRESHOLDS[preset.id]
  if (width < t.minSupportedWidth) return 'unsupported'
  if (width < t.minFullWidth) return 'narrow'
  if (height < t.minShortHeight) return 'narrow'
  if (height < t.minFullHeight) return 'short'
  return 'full'
}
