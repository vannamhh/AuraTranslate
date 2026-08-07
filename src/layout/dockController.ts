/**
 * Cổng nối **một chiều** giữa `CommandRegistry` và cái dockview đang sống.
 * Story 1.14 · AC3 · AC5 · AC9 · Task 4 · Task 5 · Task 6.
 *
 * ─────────────────────────────────────────────────────────────────────────────
 * 🔴 VÌ SAO TỆP NÀY TỒN TẠI — MỘT VẤN ĐỀ VỀ THỜI ĐIỂM, KHÔNG PHẢI MỘT LỚP TRỪU TƯỢNG
 * ─────────────────────────────────────────────────────────────────────────────
 * `installCommands()` chạy **trước `mount()`** (`src/main.ts`, khối *"THỨ TỰ BẮT BUỘC #2"*),
 * còn `DockviewApi` chỉ tồn tại sau khi `<DockviewVue>` phát `@ready` — tức **sau** `mount()`.
 * Nên handler của `layout.preset_grid` không thể ôm một `api` lúc đăng ký: lúc đó chưa
 * có gì để ôm.
 *
 * Lời giải: `main.ts` tiêm vào ba hàm của tệp này, và `WorkspaceDock.vue` gọi
 * [`setDockController`] lúc `@ready`. Handler luôn hỏi *"cái dock đang sống là cái nào"*
 * tại thời điểm CHẠY, không tại thời điểm đăng ký.
 *
 * ⚠️ Và vì sao KHÔNG đặt ba hàm này thẳng vào `src/commands/index.ts`: tệp đó phải nạp
 * được bằng **Node thuần** để Kiểm C/D/E của `npm run check:commands` chạy trên chính bộ
 * command của sản phẩm. Một `import` giá trị của `vue`/`dockview` ở đó giết ba phép kiểm
 * hành vi cùng lúc (§Bẫy 6 của Story 1.6). Tệp này không cũng không import `vue` — nó chỉ chở
 * một con trỏ và ba hàm bọc — nhưng nó không cần phải nạp được bằng Node, nên ranh giới
 * đặt ở đây là chỗ rẻ nhất.
 *
 * ─────────────────────────────────────────────────────────────────────────────
 * KHÔNG BAO GIỜ NÉM — nó KÊU
 * ─────────────────────────────────────────────────────────────────────────────
 * Ba hàm dưới đây chạy từ một hợp âm bàn phím. Ném ở đó nghĩa là một phím tắt bấm nhầm
 * lúc đang ở Library làm sập handler bàn phím. Cùng kỷ luật với `focus.ts::armBodyGuard`:
 * ghi chẩn đoán **nêu đích danh** rồi trả `false`. Không "vá" bằng cách tự chuyển sang
 * Workspace — đó là đoán ý người dùng.
 */

/**
 * Bề mặt mà `WorkspaceDock.vue` cung cấp. ⚠️ Đây là **cổng** (AD-1): tệp này không biết
 * `dockview` tồn tại, nó chỉ biết bốn câu hỏi cần trả lời.
 */
export type DockController = {
  /** Áp một preset bố cục đã khai (`layout.preset_grid` · `layout.preset_columns`). */
  applyPreset(presetId: string): boolean
  /** Ẩn nếu đang hiện, hiện lại đúng chỗ đã nhớ nếu đang ẩn (AC3). */
  togglePanel(panelId: string): boolean
  /**
   * Các panel đang HIỆN, theo **thứ tự bố cục** — trái→phải rồi trên→dưới của lưới hiện
   * tại (AC9). Panel đã ẩn không có mặt.
   */
  visiblePanelsInLayoutOrder(): readonly string[]
}

let live: DockController | null = null

/**
 * `WorkspaceDock.vue` gọi hàm này lúc `@ready` và gọi lại với `null` lúc tháo.
 *
 * ⚠️ Gọi với `null` là **bắt buộc**, không phải dọn dẹp cho gọn: `<KeepAlive>` giữ
 * Workspace sống qua các lượt đổi chế độ, nhưng một lượt HMR hoặc một lượt dựng lại thật
 * sẽ để `live` trỏ vào một `DockviewApi` đã `dispose()`. Mọi lời gọi sau đó trượt ở một
 * chỗ sâu trong thư viện thay vì ở dòng `if (live === null)` ngay dưới.
 */
export function setDockController(controller: DockController | null): void {
  live = controller
}

/** Có đang có một dock sống không? Dùng ở chỗ cần rẽ nhánh, không dùng để nuốt lỗi. */
export function hasDockController(): boolean {
  return live !== null
}

function absent(what: string): false {
  console.error(
    `[layout] \`${what}\` gọi khi chưa có bố cục nào đang sống — thao tác KHÔNG chạy. ` +
      'Nguyên nhân thường gặp: chế độ đang hiện không phải Workspace, hoặc `<DockviewVue>` ' +
      'chưa phát `@ready`. Đây không phải lý do để tự chuyển chế độ giùm người dùng.',
  )
  return false
}

/** Handler thật của `layout.preset_grid` / `layout.preset_columns` (AC5). */
export function applyPreset(presetId: string): boolean {
  return live === null ? absent(`applyPreset(${presetId})`) : live.applyPreset(presetId)
}

/** Handler thật của bốn command `layout.toggle_*` (AC3). */
export function togglePanel(panelId: string): boolean {
  return live === null ? absent(`togglePanel(${panelId})`) : live.togglePanel(panelId)
}

/**
 * Vòng xoay của `focus.next_panel` / `focus.prev_panel` (AC9).
 *
 * ⚠️ Trả mảng RỖNG khi chưa có dock — không trả `PANEL_IDS` làm đường lui. Một vòng
 * đoán mò sẽ gọi `enterFocus('panel.source')` cho một panel chưa dựng, và `focus.ts` ghi
 * đúng một dòng chẩn đoán *"khai điểm vào nhưng phần tử chưa có trong DOM"* — một câu
 * đúng về triệu chứng và sai về nguyên nhân.
 */
export function panelRing(): readonly string[] {
  if (live === null) return []
  return live.visiblePanelsInLayoutOrder()
}
