/**
 * Chỗ **DUY NHẤT** đăng ký thao tác — đúng khuôn "một chỗ chạm" của `src/i18n/index.ts`.
 * Story 1.6 · AC1 · AC2 · AC3 · AC6 · AD-34 · FR22.
 *
 * ─────────────────────────────────────────────────────────────────────────────
 * ⚠️ TỆP NÀY VẪN NẠP ĐƯỢC BẰNG NODE THUẦN — và đó là một ràng buộc, không phải may mắn
 * ─────────────────────────────────────────────────────────────────────────────
 * Nó chỉ `import` ba module thuần cùng thư mục. Không `vue`, không `../modes/**`,
 * không `@tauri-apps/api`. Nhờ vậy Kiểm E của `scripts/check-commands.mjs` nạp được
 * **chính bộ command của sản phẩm** rồi đối chiếu `labelKey` với `vi.json` thật — thay
 * vì đối chiếu với một bản chép trong script, thứ sẽ trôi khỏi sự thật trong hai story.
 *
 * Cách giữ ràng buộc đó mà vẫn có thao tác thật: **handler phụ thuộc trạng thái được
 * TIÊM VÀO** qua `installCommands({ setMode })`. `App.vue` là chỗ nối hai đầu.
 *
 * Đừng thêm command nghiệp vụ (tra cứu, xác nhận, dịch…) vào đây — mỗi story tự thêm
 * command của mình, và mỗi lần thêm là một lần đi qua `register()` với ba phép cưỡng chế.
 */
import { createRegistry } from './registry.ts'
import { createFocusRegistry } from './focus.ts'
import { attachKeymap, createKeymap } from './keys.ts'
import type { CommandId, Registry } from './registry.ts'
import type { FocusEntry, FocusOwner, FocusRegistry } from './focus.ts'
import type { Keymap } from './keys.ts'

export type { CommandId, CommandSpec, Registry } from './registry.ts'
export type { FocusEntry, FocusOwner, FocusRegistry } from './focus.ts'
export type { Binding, ChordEvent, Keymap } from './keys.ts'

/** Ba chế độ NGANG HÀNG trong MỘT cửa sổ hệ điều hành (AD-24, AC3). */
export type ModeId = 'library' | 'workspace' | 'reading'

/**
 * Thứ tự này là thứ tự `⌘1` `⌘2` `⌘3` và thứ tự tab ở thanh tiêu đề.
 * `library` đứng đầu vì PRD §5.2 gọi nó là **điểm vào ứng dụng**.
 */
export const MODE_IDS: readonly ModeId[] = ['library', 'workspace', 'reading']

/**
 * Tiền tố của vòng xoay panel.
 *
 * ⚠️ Story 1.14 KHÔNG còn dùng nó cho `focus.next_panel`: vòng xoay nay đi theo **thứ tự
 * bố cục** do `deps.panelRing()` cấp, không theo thứ tự `declare()` (AC9). Hằng số ở
 * lại vì `focusRegistry.next(prefix)` vẫn là đường lui khai được và Story 1.21 sẽ đọc nó.
 */
const PANEL_PREFIX = 'panel.'

/**
 * 🔴 SỔ ĐIỂM VÀO FOCUS — danh sách này là HỢP ĐỒNG, không phải một chú thích.
 *
 * Kiểm E của cổng đọc chính danh sách này rồi đối chiếu HAI CHIỀU với mã nguồn: mọi
 * owner xuất hiện trong `.vue` phải có ở đây, và mọi mục ở đây phải xuất hiện trong
 * `.vue`. Chiều thứ hai là thứ bắt được một chế độ quên khai điểm vào — đúng ca mà AC4
 * nói *"focus không bao giờ rơi về `body`"* và cũng đúng ca mà không con mắt nào bắt
 * được sau khi có mười panel.
 *
 * ⚠️ **BẢY** mục kể từ Story 1.14: ba chế độ + **bốn** panel. Trước đó là năm — hai panel
 * của `WorkspaceMode` được dựng thẳng bằng `PanelFrame`. Nay cả bốn sống trong dockview.
 *
 * 🔴 Thứ tự ở đây là thứ tự KHAI BÁO, **không** phải thứ tự vòng xoay focus. Vòng xoay
 * đi theo lưới đang hiện (AC9) — xem `deps.panelRing`.
 */
export const FOCUS_OWNERS: readonly FocusOwner[] = [
  'mode.library',
  'mode.workspace',
  'mode.reading',
  'panel.source',
  'panel.lookup',
  'panel.ai_translation',
  'panel.editor',
]

const registry: Registry = createRegistry()
const focus: FocusRegistry = createFocusRegistry()

/** Kho đọc được cho Story 1.21 (màn hình gán phím) và cho Kiểm C/E của cổng. */
export const commandRegistry: Registry = registry
export const focusRegistry: FocusRegistry = focus

/**
 * ĐÂY LÀ CỬA DUY NHẤT MÀ MỘT HANDLER CHUỘT ĐƯỢC ĐI QUA (AC1).
 *
 * `@click` trong `.vue` phải là **đúng một** lời gọi `dispatch('<id>')` — Kiểm A của
 * `scripts/check-commands.mjs` cưỡng chế bằng cú pháp, và `registry.dispatch` ném với
 * một id lạ nên mọi đường còn lại (`.ts`, handler bàn phím, Story 1.21) cũng bị canh.
 */
export function dispatch(id: CommandId): void {
  registry.dispatch(id)
}

/** Mỗi chế độ và mỗi panel gọi hàm này lúc mount (AD-34 §2). */
export function declareFocus(owner: FocusOwner, resolve: FocusEntry): void {
  focus.declare(owner, resolve)
}

/** Gỡ khai báo lúc tháo — không có nó thì một lượt mount lại là một lần ném. */
export function releaseFocus(owner: FocusOwner): void {
  focus.release(owner)
}

/** Dời focus DOM tường minh tới điểm vào đã khai. `false` khi trượt, kèm `console.error`. */
export function enterFocus(owner: FocusOwner): boolean {
  return focus.enter(owner)
}

/**
 * Nhận biết nền tảng — **chỗ DUY NHẤT** trong dự án đọc tới nó.
 *
 * ⚠️ Cả hai API đều đang trôi: `navigator.userAgentData` là Chromium-only và chưa vào
 * chuẩn; `navigator.platform` bị đánh dấu deprecated trong HTML spec nhưng vẫn là thứ
 * WKWebView trả lời đúng. Đọc cái mới trước, rơi về cái cũ, và giữ cả hai ở đúng một
 * dòng để ngày một trong hai chết thì chỉ có một chỗ phải sửa.
 *
 * Đừng gọi hàm này từ `./keys.ts` — §Trap 1: nhận biết nền tảng phải TIÊM ĐƯỢC, nếu
 * không Kiểm D không lái được hai ca và NFR14 mất lưới duy nhất của nó.
 */
export function detectIsMac(): boolean {
  if (typeof navigator === 'undefined') return false
  const nav = navigator as Navigator & { userAgentData?: { platform?: string } }
  const platform = nav.userAgentData?.platform ?? nav.platform ?? ''
  return /mac/i.test(platform)
}

let keymap: Keymap | null = null

export type CommandDeps = {
  /** Đổi chế độ đang hiện. `App.vue` nối vào `src/modes/modeState.ts`. */
  setMode: (mode: ModeId) => void
  /** Tiêm được để Kiểm D lái hai nền tảng. Mặc định đọc từ `detectIsMac()`. */
  isMac?: boolean
  /**
   * Hợp âm đọc **từ đĩa** (`global.db`, loại `shortcut`) — Story 1.8, AC5.
   *
   * ⚠️ **TIÊM VÀO, không `invoke` ở tệp này.** `scripts/check-commands.mjs` (Kiểm
   * C/D/E) và `scripts/check-i18n.mjs` (Kiểm E) nạp thẳng tệp này bằng **Node thuần**,
   * nên một `import` giá trị của `@tauri-apps/api` giết ba phép kiểm hành vi cùng lúc.
   * Đường nạp sống ở `src/config/bootstrap.ts`; `src/main.ts` nối hai đầu — đúng cùng
   * cửa mà `setMode` đã đi qua từ Story 1.6.
   *
   * Khoá là id thao tác, giá trị là danh sách hợp âm. Thiếu một id ⇒ dùng mặc định của
   * chính hàm này; không `undefined` (chưa nạp gì) khác hẳn `{}` (đã nạp, chưa ai đặt gì)
   * chỉ ở chỗ đọc, không ở kết quả.
   */
  bindings?: Readonly<Record<string, readonly string[]>>

  // ── Story 1.14 — ba cổng của tầng bố cục ────────────────────────────────────────
  //
  // ⚠️ TIÊM VÀO, cùng cửa và cùng lý do với `setMode`: `DockviewApi` chỉ tồn tại sau
  // `@ready`, tức SAU `mount()`, trong khi `installCommands()` chạy TRƯỚC. Ba hàm này
  // hỏi *"cái dock đang sống là cái nào"* tại thời điểm CHẠY — cài đặt ở
  // `src/layout/dockController.ts`, nối ở `src/main.ts`.
  //
  // 🔴 **Tuỳ chọn, và đó là chủ ý.** Kiểm C/D/E của cổng gọi `installCommands()` trong
  // Node thuần, nơi không có dockview và cũng không cần có: chúng kiểm văn phạm id,
  // nhãn, hợp âm và sổ focus. Handler vắng cổng thì **KÊU** (`console.error` nêu đích
  // danh) chứ không ném và không im — cùng kỷ luật với `focus.ts`.

  /** Áp một preset bố cục đã khai. Handler của `layout.preset_*` (AC5). */
  applyPreset?: (presetId: string) => boolean
  /** Ẩn/hiện một panel. Handler của bốn `layout.toggle_*` (AC3). */
  togglePanel?: (panelId: string) => boolean

  // ── Story 1.15 — nộp form nhập Tác phẩm ở Library ───────────────────────────────
  //
  // ⚠️ TIÊM VÀO, cùng cửa và cùng lý do với `applyPreset`/`togglePanel`: state của form
  // (`name`, `pastedText`, `filePath`, …) sống trong `src/modes/libraryImport.ts`, một
  // module Vue thật — import thẳng nó ở đây giết Kiểm C/D/E cùng lý do `@tauri-apps/api`
  // bị cấm. `src/main.ts` nối `submitPastedText`/`submitFilePath` vào `installCommands`.

  /** Nộp `pastedText` hiện tại. Handler của `library.import_text` (AC1 nhánh dán). */
  submitPastedText?: () => void
  /** Nộp `filePath` hiện tại. Handler của `library.import_file` (AC1 nhánh tệp/NFR17). */
  submitFilePath?: () => void

  // ── Story 1.16 — dải tab và kiểu xem của Panel Source ───────────────────────────
  //
  // ⚠️ TIÊM VÀO, cùng cửa và cùng lý do với `applyPreset`/`submitPastedText`: state sống ở
  // `src/panels/sourcePanelState.ts`, một module Vue thật (`ref`) — import thẳng nó ở đây
  // giết Kiểm C/D/E cùng lý do `@tauri-apps/api` bị cấm.

  /** Chọn tab của Panel Source. Handler của `source.select_tab_*` (AC6). */
  selectSourceTab?: (tab: 'original' | 'han_viet') => void
  /** Đổi kiểu xem (chuyển đổi ↔ song song) của tab Hán Việt. Handler của
   * `source.toggle_han_viet_view` (AC6). */
  toggleHanVietView?: () => void

  // ── Story 1.17 — một lượt tra Panel Lookup ──────────────────────────────────────
  //
  // ⚠️ TIÊM VÀO, cùng cửa và cùng lý do với `selectSourceTab`: `lookupPanelState.ts` dùng
  // `ref`/`computed` của Vue — import thẳng nó ở đây giết Kiểm C/D/E cùng lý do
  // `@tauri-apps/api` bị cấm.

  /**
   * Chạy một lượt tra Panel Lookup. Handler của `lookup.lookup_selection` (Quyết định #1a).
   *
   * ⚠️ `runLookup` thật là `async`; kiểu `() => void` ở đây khớp cùng khuôn
   * `submitPastedText`/`submitFilePath` — promise trả về bị bỏ qua có chủ ý (fire-and-forget,
   * state cập nhật qua `ref` module-level).
   */
  runLookup?: (query: string) => void
  /**
   * 🔴 Chỗ lấy vùng chọn — **Quyết định #1a, ranh giới với Story 1.18**. Đây **không**
   * phải hợp đồng vùng chọn dùng chung cho bốn panel (đó là Story 1.18); nó là một dep
   * TỐI THIỂU chỉ để story này nghiệm thu được mà không lấn phạm vi. Story 1.18 thay ĐÚNG dep
   * này bằng hợp đồng thật, không phải chạm `runLookup`/component nào khác.
   */
  currentSelection?: () => string

  // ── Story 1.18 — bôi đen bằng BÀN PHÍM (`deferred-work.md:608`) ─────────────────
  //
  // ⚠️ TIÊM VÀO, cùng cửa và cùng lý do với `currentSelection`: cài đặt sống ở
  // `src/panels/selectionContract.ts` và chạm DOM (`window.getSelection`,
  // `document.createRange`) — import thẳng nó ở đây giết Kiểm C/D/E.
  //
  // 🔴 **VÌ SAO CHÚNG LÀ COMMAND, KHÔNG MỘT `@keydown` TRÊN BỀ MẶT CHỮ:** AD-34 §1 nói sàn
  // khả năng tiếp cận là **CẤU TRÚC**, không kỷ luật. Một handler gắn thẳng vào phần tử không
  // gán lại phím được, không liệt kê được ở màn hình gán phím của **Story 1.21**, và không đi qua
  // ba phép cưỡng chế của `register()`.

  /** Đặt caret vào bề mặt chữ đầu tiên đã đăng ký. Handler của `selection.focus_source`. */
  focusSelectionSource?: () => boolean
  /** Mở rộng vùng chọn một KÝ TỰ sang trái. Handler của `selection.extend_left`. */
  extendSelectionLeft?: () => void
  /** Mở rộng vùng chọn một KÝ TỰ sang phải. Handler của `selection.extend_right`. */
  extendSelectionRight?: () => void
  /** Mở rộng vùng chọn một TỪ sang trái. Handler của `selection.extend_word_left`. */
  extendSelectionWordLeft?: () => void
  /** Mở rộng vùng chọn một TỪ sang phải. Handler của `selection.extend_word_right`. */
  extendSelectionWordRight?: () => void

  /**
   * Các panel đang HIỆN, theo **thứ tự bố cục** (AC9).
   *
   * 🔴 Không phải thứ tự `declare()`, và khác biệt đó là cả nội dung của AC9: hai thứ
   * tự tách nhau ngay lần đầu người dùng kéo một panel sang chỗ khác, và một vòng xoay
   * đi theo thứ tự khai báo sẽ nhảy lung tung trên màn hình mà không cổng nào đỏ.
   */
  panelRing?: () => readonly string[]
}

/** Bốn panel của Workspace, theo thứ tự khai báo. ⚠️ Chép từ `src/layout/workspaceLayout.ts`. */
const PANEL_SUFFIXES: readonly string[] = ['source', 'lookup', 'ai_translation', 'editor']

/**
 * Cổng vắng mặt ⇒ **kêu**, không ném và không im.
 *
 * ⚠️ Ném ở đây là một hợp âm bấm nhầm lúc đang ở Library giết luôn handler bàn phím
 * (`keys.ts::handle` gọi `registry.dispatch` không bọc `try`). Im lặng thì tệ hơn: một
 * phím tắt không làm gì và không ai lần được về dòng nào.
 */
function portMissing(commandId: string, port: string): void {
  console.error(
    `[commands] \`${commandId}\` chạy nhưng cổng \`${port}\` chưa được tiêm — thao tác ` +
      'KHÔNG có hiệu lực. `src/main.ts` là chỗ nối `installCommands()` với ' +
      '`src/layout/dockController.ts`.',
  )
}

/**
 * Hợp âm cho một thao tác: đĩa thắng, không nhưng chỉ khi đĩa **có nói gì**.
 *
 * ⚠️ `?? fallback` chứ không `|| fallback`: một mảng rỗng trên đĩa là một phát biểu hợp lệ
 * — *"thao tác này cố ý không có phím"* — và `||` sẽ lặng lẽ dựng lại hợp âm mặc định cho
 * nó. Story 1.21 (màn gán phím) dựa vào việc gỡ hết phím là một trạng thái lưu được.
 */
function chordsFor(
  id: CommandId,
  bindings: CommandDeps['bindings'],
  fallback: readonly string[] | undefined,
): readonly string[] | undefined {
  return bindings?.[id] ?? fallback
}

/**
 * Đăng ký bộ command khởi động vào **một** registry. Tách ra để dùng được hai lần: một lần
 * trên registry thật, một lần trên một registry nháp — xem [`bindingsAreUsable`].
 */
function registerAll(target: Registry, deps: CommandDeps, bindings: CommandDeps['bindings']): void {
  const setMode = deps.setMode
  for (const mode of MODE_IDS) {
    target.register({
      id: `mode.${mode}`,
      // §Quyết định thiết kế #4 — tiền tố `command.` là bắt buộc, không dùng thẳng id
      // làm khoá: `command.mode.library` cho nhãn hôm nay, `command.mode.library.hint`
      // cho mô tả ở màn hình gán phím của Story 1.21, và một lượt grep `"command."`
      // trong `vi.json` liệt kê đúng bộ nhãn thao tác.
      labelKey: `command.mode.${mode}`,
      // `Mod` — KHÔNG phải `Meta`. Xem §Trap 1 ở đầu `./keys.ts`.
      keys: chordsFor(`mode.${mode}`, bindings, [`Mod+${MODE_IDS.indexOf(mode) + 1}`]),
      run: () => {
        setMode(mode)
        // KHÔNG gọi `enterFocus` ở đây. Phần tử của chế độ vừa chọn chưa có trong
        // DOM tại thời điểm này — Vue mới chỉ nhận được thay đổi state. Mỗi chế độ tự
        // gọi `enterFocus` trong `onActivated`, tức đúng lúc nó đã dựng xong.
      },
    })
  }

  /**
   * ═══════════════════════════════════════════════════════════════════════════════
   * 🔴 STORY 1.14 · §QUYẾT ĐỊNH #1 và #2 — HỌ PHÍM `Mod+Alt+…`
   * ═══════════════════════════════════════════════════════════════════════════════
   *
   * `mockups/key-screen-workspace.html:89` vẽ `⌘1` `⌘2` cho **preset bố cục**. Xung đột
   * đó **đã bị phân xử ở Story 1.6: chế độ thắng** (UX-DR34 · `EXPERIENCE.md:49` · AC3
   * Story 1.6). Mockup chưa sửa và dev không sửa nó — sửa tài liệu quy hoạch là một
   * lượt riêng của Ice. Việc còn lại là chọn phím KHÁC, và đây là chỗ chọn.
   *
   * Chốt: **`Mod+Alt+<số>` cho preset · `Mod+Alt+<mũi tên>` cho đi lại giữa panel.**
   *   - giữ nguyên "số thứ tự preset" mà mockup dạy, chỉ thêm một phím bổ trợ;
   *   - `Mod+Alt+3` để TRỐNG cho **Review Mode** ở Story 8.11 — đúng thứ tự mockup;
   *   - một họ phím cho cả hai nhóm, nên người dùng học một lần;
   *   - không đụng `Tab` (thứ tự tiêu điểm của trình duyệt), không đụng `⌥←` `⌥→`
   *     trần (*Chương trước/sau*, `EXPERIENCE.md:148`, Story 2.11), không đụng `⌘⇧…`
   *     (không gian của UX-DR35).
   *
   * ⚠️ Khớp bằng `event.code` (`Digit1`, `ArrowRight`) nên việc `⌥1` sinh ký tự `¡` trên
   * macOS không thành vấn đề — xem `keys.ts` §"KHỚP BẰNG `event.code`".
   * ⚠️ `Alt` đã có trong `parseChord`; `Digit1`/`Digit2`/`ArrowLeft`/`ArrowRight` đều phân
   * giải được. Không thêm tên phím mới vào `NAMED_CODES`.
   */
  for (const preset of ['grid', 'columns']) {
    const id = `layout.preset_${preset}`
    target.register({
      id,
      labelKey: `command.${id}`,
      keys: chordsFor(id, bindings, [`Mod+Alt+${preset === 'grid' ? 1 : 2}`]),
      run: () => {
        if (deps.applyPreset === undefined) return portMissing(id, 'applyPreset')
        deps.applyPreset(id)
      },
    })
  }

  /**
   * ═══════════════════════════════════════════════════════════════════════════════
   * 🔴 BỐN COMMAND ẨN/HIỆN PANEL — HANDLER THẬT, CỐ Ý KHÔNG GÁN PHÍM (AC3, §QĐ #3)
   * ═══════════════════════════════════════════════════════════════════════════════
   *
   * AC6 của Story 1.6 nghiệm thu bằng việc `unbound()` trả về **ít nhất một phần tử
   * thật**. Trước story này phần tử duy nhất đó là `focus.next_panel` — mà story này vừa
   * gán phím cho nó (§QĐ #2). Nếu bốn command dưới đây cũng có phím thì `unbound()` trả
   * mảng rỗng, **AC6 của Story 1.6 mất bằng chứng**, và không cổng nào đỏ. Đó là §Bẫy 5
   * của story, ghi ra bằng chữ.
   *
   * ⚠️ Nên đây là một **lỗ NFR17 mở ra CÓ Ý THỨC**: hôm nay ẩn/hiện panel chỉ tới được
   * bằng chuột. Lỗ này **có tên và có chủ** — màn hình gán phím là **Story 1.21**, và
   * `deferred-work.md` mang một mục cho nó. Một lỗ có tên tốt hơn một bằng chứng bị xoá.
   *
   * Và handler thì CHẠY THẬT. Một command rỗng đăng ký cho đủ số là đúng thứ
   * `CommandRegistry` tồn tại để chặn (`registry.ts` ném khi thiếu `run`).
   *
   * ⚠️ `keys: chordsFor(id, bindings, undefined)` — mặc định không phím, NHƯNG nếu
   * `global.db` có một hợp âm cho id này thì nó ĐƯỢC dùng: gán phím là quyền của người
   * dùng, và Story 1.21 là màn hình để làm việc đó, không phải một cái khoá lên chính
   * dữ liệu đó.
   */
  for (const suffix of PANEL_SUFFIXES) {
    const id = `layout.toggle_${suffix}`
    target.register({
      id,
      labelKey: `command.${id}`,
      keys: chordsFor(id, bindings, undefined),
      run: () => {
        if (deps.togglePanel === undefined) return portMissing(id, 'togglePanel')
        deps.togglePanel(`panel.${suffix}`)
      },
    })
  }

  /**
   * `focus.next_panel` / `focus.prev_panel` — AC9, đóng `deferred-work.md:134` và `:161`.
   *
   * 🔴 Trước story này **không có đường bàn phím nào vào panel**: §Quyết định #5 của
   * Story 1.6 cố ý để trống vì *"bốn panel chưa tồn tại, nên vòng xoay chưa biết gồm những
   * gì"*. Nay chúng tồn tại, nên `deferred-work.md:161` — *"Không đánh dấu AC4 đạt trọn
   * cho tới lúc đó"* — đóng ở đây.
   *
   * 🔴 Vòng xoay đi theo **thứ tự bố cục hiện tại**, không theo thứ tự `declare()`.
   * `deps.panelRing()` là chỗ biết lưới; `focus.cycle` chỉ biết đi trên một vòng đã cho.
   * Panel đã ẩn (AC3) không có trong vòng — vì `visiblePanelsInLayoutOrder()` chỉ đọc
   * những panel THẬT SỰ đang trong dockview.
   *
   * ⚠️ Có `prev` chứ không chỉ `next`: một vòng bốn panel đi được một chiều thì lùi một
   * bước tốn ba lần bấm.
   */
  for (const [id, step] of [
    ['focus.next_panel', 1],
    ['focus.prev_panel', -1],
  ] as const) {
    target.register({
      id,
      labelKey: `command.${id}`,
      keys: chordsFor(id, bindings, [`Mod+Alt+Arrow${step > 0 ? 'Right' : 'Left'}`]),
      run: () => {
        if (deps.panelRing === undefined) {
          // ⚠️ Đường lui là thứ tự KHAI BÁO — đúng hành vi của Story 1.6, và đúng thứ
          // Kiểm C/E của cổng chạy trên. Không im lặng: nó không phải hành vi sản phẩm.
          portMissing(id, 'panelRing')
          focus.next(PANEL_PREFIX)
          return
        }
        focus.cycle(deps.panelRing(), step)
      },
    })
  }

  /**
   * `library.import_text` / `library.import_file` — Story 1.15, AC1.
   *
   * ⚠️ Cố ý KHÔNG gán phím: nộp một form là một thao tác cần đọc lại giá trị đã gõ
   * trước khi kích hoạt, không phải một lối tắt đáng nhớ — cùng lý lẽ mà các
   * `layout.toggle_*` cố ý không gán phím ở Story 1.6 (§QĐ #3 phía trên). Hai nút bấm
   * tương ứng vẫn tới được bằng bàn phím qua Tab + Enter/Space, chuẩn HTML gốc.
   */
  target.register({
    id: 'library.import_text',
    labelKey: 'command.library.import_text',
    run: () => {
      if (deps.submitPastedText === undefined) return portMissing('library.import_text', 'submitPastedText')
      deps.submitPastedText()
    },
  })
  target.register({
    id: 'library.import_file',
    labelKey: 'command.library.import_file',
    run: () => {
      if (deps.submitFilePath === undefined) return portMissing('library.import_file', 'submitFilePath')
      deps.submitFilePath()
    },
  })

  /**
   * `source.select_tab_original` / `source.select_tab_han_viet` / `source.toggle_han_viet_view`
   * — Story 1.16, AC6. **CÓ phím** (không thao tác nào chỉ tới được bằng chuột — khác hẳn
   * §Quyết định #3 của `layout.toggle_*` ở trên, cố ý KHÔNG có phím vì lý do khác).
   *
   * ⚠️ HAI command CHỌN tab (không MỘT command đổi/toggle) — cùng khuôn
   * `layout.preset_grid`/`layout.preset_columns`: bấm đúng tab đang chọn là một thao tác
   * VÔ HẠI (idempotent), không lật sang tab kia. Một toggle duy nhất cho hai nút bấm sẽ
   * lật nhầm khi người dùng bấm đúng tab đang mở.
   */
  target.register({
    id: 'source.select_tab_original',
    labelKey: 'command.source.select_tab_original',
    keys: chordsFor('source.select_tab_original', bindings, ['Mod+Alt+O']),
    run: () => {
      if (deps.selectSourceTab === undefined) {
        return portMissing('source.select_tab_original', 'selectSourceTab')
      }
      deps.selectSourceTab('original')
    },
  })
  target.register({
    id: 'source.select_tab_han_viet',
    labelKey: 'command.source.select_tab_han_viet',
    // 🔴 `Mod+Alt+J`, KHÔNG `Mod+Alt+H`: `⌘⌥H` là "Hide Others" của macOS và hệ điều
    // hành nuốt nó trước khi webview thấy. `check:commands` chỉ kiểm trùng **nội bộ** bộ
    // command — nó không biết gì về phím của OS, nên lưới ở đây là con người. Ice chốt ở
    // lượt code review 2026-08-06. (`Mod+Alt+O`/`Mod+Alt+V` không mang nghĩa hệ thống.)
    keys: chordsFor('source.select_tab_han_viet', bindings, ['Mod+Alt+J']),
    run: () => {
      if (deps.selectSourceTab === undefined) {
        return portMissing('source.select_tab_han_viet', 'selectSourceTab')
      }
      deps.selectSourceTab('han_viet')
    },
  })
  target.register({
    id: 'source.toggle_han_viet_view',
    labelKey: 'command.source.toggle_han_viet_view',
    keys: chordsFor('source.toggle_han_viet_view', bindings, ['Mod+Alt+V']),
    run: () => {
      if (deps.toggleHanVietView === undefined) {
        return portMissing('source.toggle_han_viet_view', 'toggleHanVietView')
      }
      deps.toggleHanVietView()
    },
  })

  /**
   * `lookup.lookup_selection` — Story 1.17, Quyết định #1a.
   *
   * 🔴 **CÓ phím** (không thao tác chỉ tới được bằng chuột — cùng lý lẽ với
   * `source.select_tab_*`, khác `layout.toggle_*` cố ý không gán phím). ⚠️ Vùng chọn RỖNG
   * là thao tác VÔ HẠI (không lỗi) — cùng luật `selectSourceTab`/`toggleHanVietView` khi
   * thao tác không áp dụng.
   */
  target.register({
    id: 'lookup.lookup_selection',
    labelKey: 'command.lookup.lookup_selection',
    keys: chordsFor('lookup.lookup_selection', bindings, ['Mod+Alt+L']),
    run: () => {
      if (deps.runLookup === undefined) return portMissing('lookup.lookup_selection', 'runLookup')
      if (deps.currentSelection === undefined) {
        return portMissing('lookup.lookup_selection', 'currentSelection')
      }
      const text = deps.currentSelection()
      if (text.trim() === '') return
      deps.runLookup(text)
    },
  })

  /**
   * ═══════════════════════════════════════════════════════════════════════════════
   * 🔴 STORY 1.18 — NĂM COMMAND BÔI ĐEN BẰNG BÀN PHÍM (AC11, `deferred-work.md:608`)
   * ═══════════════════════════════════════════════════════════════════════════════
   *
   * `deferred-work.md:608` — *"Ice chốt 2026-08-06 ở lượt code review: **ghi nợ cho
   * 1.18**"*. AC1 của epic nói *"thả chuột **hoặc kết thúc vùng chọn bằng bàn phím**"*, và
   * vế thứ hai trước story này không **thực hiện được**.
   *
   * ─────────────────────────────────────────────────────────────────────────────
   * 🔴 CHỌN HỢP ÂM — HAI RÀNG BUỘC LOẠI HẾT CÁC ĐƯỜNG HIỂN NHIÊN
   * ─────────────────────────────────────────────────────────────────────────────
   * ① **không `Mod+Shift+Mũi tên`**, dù story đề xuất nó: `⌘⇧…` là không gian mà **UX-DR35**
   *    giữ, và chính §Task 4 của story liệt kê nó trong danh sách *"không đụng"*. Hai mệnh đề
   *    của story mâu thuẫn nhau; ràng buộc UX-DR35 thắng vì nó là tài liệu quy hoạch.
   * ② **không `Mod+Alt+Mũi tên`** — đã thuộc `focus.next_panel`/`focus.prev_panel` (1.14).
   * ③ **không `⌥←`/`⌥→` TRẦN** — *Chương trước/sau*, `EXPERIENCE.md:148`, Story 2.11.
   *
   * ⇒ `Shift+Mũi tên` (ký tự) và `Alt+Shift+Mũi tên` (từ) — **đúng hợp âm bản địa** mà cả
   * macOS lẫn Windows đã dạy người dùng cho thao tác này, nên không phải học gì mới.
   *
   * 🔴 **VÌ SAO CHÚNG KHÔNG GIẾT BÔI ĐEN TRONG Ô NHẬP CỦA LIBRARY:** cả hai hợp âm **KHÔNG mang
   * `Meta`/`Ctrl`**, nên luật vùng gõ của `keys.ts:287` (`lacksPrimaryMod && isTypingZone`)
   * bỏ qua chúng khi tiêu điểm ở `<input>`/`<textarea>`/`contenteditable` — hành vi bôi đen
   * gốc của trình duyệt đi tiếp nguyên vẹn. Luật đó có từ Story 1.6 và story này là
   * **người tiêu thụ đầu tiên** của nó với một hợp âm thật.
   *
   * ⚠️ **GIỚI HẠN ĐÃ BIẾT, không giấu:** `keys.ts:295` trả sớm khi `event.repeat === true`, nên
   * **giữ** `Shift+→` không mở rộng liên tục — phải bấm lặp. Luật đó đúng cho 17 command hiện
   * có (*lặp lại "đổi chế độ" là vô nghĩa*) và sai cho đúng bốn command này. Nới nó cần một
   * cờ `repeatable` trên `CommandSpec` — một thay đổi chạm `registry.ts` + `keys.ts` và
   * **mọi** command đang có, tức không thuộc story này. Hai command **theo TỪ** bù phần lớn
   * chi phí. Ghi vào `deferred-work.md`, chủ: Story 1.21.
   */
  target.register({
    id: 'selection.focus_source',
    labelKey: 'command.selection.focus_source',
    // `Mod+Alt+S` — họ phím `Mod+Alt+…` của Story 1.14. `S` còn trống (`O`/`J`/`V`/`L`
    // đã dùng), và `⌘⌥S` không mang nghĩa hệ điều hành trên cả hai nền tảng.
    keys: chordsFor('selection.focus_source', bindings, ['Mod+Alt+S']),
    run: () => {
      if (deps.focusSelectionSource === undefined) {
        return portMissing('selection.focus_source', 'focusSelectionSource')
      }
      deps.focusSelectionSource()
    },
  })

  for (const [id, port, chord] of [
    ['selection.extend_left', 'extendSelectionLeft', 'Shift+ArrowLeft'],
    ['selection.extend_right', 'extendSelectionRight', 'Shift+ArrowRight'],
    ['selection.extend_word_left', 'extendSelectionWordLeft', 'Alt+Shift+ArrowLeft'],
    ['selection.extend_word_right', 'extendSelectionWordRight', 'Alt+Shift+ArrowRight'],
  ] as const) {
    target.register({
      id,
      labelKey: `command.${id}`,
      keys: chordsFor(id, bindings, [chord]),
      run: () => {
        const handler = deps[port]
        if (handler === undefined) return portMissing(id, port)
        handler()
      },
    })
  }
}

/**
 * 🔴 §Bẫy 5 — **hợp âm đọc từ đĩa gây xung đột ⇒ ứng dụng không mở được**.
 *
 * `keys.ts:270` phát hiện hai command trùng hợp âm và `createKeymap` **ném**;
 * `installCommands` chạy **trước `mount()`**. Một `global.db` sửa tay *(hoặc một lượt nhập
 * cấu hình từ máy khác)* là đủ để một cú ném ở đó cho ra **cửa sổ trắng** — và người dùng
 * mất luôn đường vào để sửa chính cái làm hỏng.
 *
 * Chốt: thử dựng keymap trên một registry **nháp** trước. Xung đột ⇒ ghi chẩn đoán rõ rồi
 * **rơi về hợp âm mặc định**, và ứng dụng lên bình thường.
 *
 * ⚠️ Registry nháp chứ không thử trên registry thật rồi dọn: `register()` ném với id
 * trùng (AC2 của Story 1.6, và đó là hành vi đúng), nên không có lượt đăng ký thứ hai nào
 * để dọn về. Nháp là đường duy nhất không phải nới một phép cưỡng chế đang đúng.
 *
 * Đây **không** phải màn giải quyết xung đột — đó là **Story 1.21**. Ở đây chỉ có
 * *"đừng chết"*.
 */
function bindingsAreUsable(
  bindings: NonNullable<CommandDeps['bindings']>,
  isMac: boolean,
): boolean {
  try {
    const scratch = createRegistry()
    // Deps rỗng: registry nháp không bao giờ được `dispatch`, nó chỉ tồn tại để
    // `createKeymap` có cái để phân giải hợp âm trên.
    registerAll(scratch, { setMode: () => {} }, bindings)
    createKeymap(scratch, { isMac })
    return true
  } catch (err) {
    console.error(
      '[commands] hợp âm đọc từ `global.db` không dựng được keymap — rơi về hợp âm mặc ' +
        'định. Lựa chọn phím tắt của bạn KHÔNG bị xoá, chỉ tạm không áp; màn hình gán ' +
        `phím (Story 1.21) là chỗ sửa. Nguyên nhân: ${String(err)}`,
    )
    return false
  }
}

/**
 * Đăng ký bộ command khởi động và dựng keymap. Gọi **một lần**, từ `src/main.ts`.
 *
 * ⚠️ Gọi lần thứ hai ⇒ `register()` ném vì id trùng. Đó là hành vi ĐÚNG (AC2) chứ không
 * phải một chỗ cần nới: hai lượt cài đặt trong một tiến trình nghĩa là hai keymap cùng
 * nghe một cửa sổ, và cái sau sẽ dispatch mọi hợp âm hai lần.
 */
export function installCommands(deps: CommandDeps): Keymap {
  const isMac = deps.isMac ?? detectIsMac()

  // Hợp âm từ đĩa chỉ được dùng khi chúng dựng được một keymap — xem `bindingsAreUsable`.
  const bindings =
    deps.bindings !== undefined && bindingsAreUsable(deps.bindings, isMac)
      ? deps.bindings
      : undefined

  registerAll(registry, deps, bindings)

  keymap = createKeymap(registry, { isMac })
  return keymap
}

/**
 * Gắn keymap vào cửa sổ. Trả về hàm gỡ.
 *
 * ⚠️ `noUnusedLocals` đang bật — dùng hàm gỡ hoặc `void` nó tường minh ở chỗ gọi.
 */
export function attachKeyboard(target: EventTarget): () => void {
  if (keymap === null) {
    throw new Error('[commands] attachKeyboard() gọi trước installCommands() — không có keymap nào để gắn.')
  }
  return attachKeymap(keymap, target)
}
