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
import { attachKeymap, createKeymap, resolveChord } from './keys.ts'
import type { CommandId, CommandSpec, Registry } from './registry.ts'
import type { FocusEntry, FocusOwner, FocusRegistry } from './focus.ts'
import type { Binding, ChordOverrides, Keymap, KeymapGate } from './keys.ts'

export type { CommandId, CommandSpec, Registry } from './registry.ts'
export type { FocusEntry, FocusOwner, FocusRegistry } from './focus.ts'
export type { Binding, ChordEvent, ChordOverrides, Keymap } from './keys.ts'
export { chordFromEvent, formatChord } from './keys.ts'

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
  // ⚠️ Đây là DÒ NĂNG LỰC. `navigator.platform` khai `string` trong lib DOM nhưng nó đã bị khai tử, và
  //    một WebView không có nó là ca mà cả hàm này tồn tại để chịu.
  // eslint-disable-next-line @typescript-eslint/no-unnecessary-condition -- xem chú thích ngay trên
  const platform = nav.userAgentData?.platform ?? nav.platform ?? ''
  return /mac/i.test(platform)
}

/**
 * Keymap ĐANG SỐNG. Story 1.21 thay biến này ở mỗi lượt gán phím thành công.
 *
 * ⚠️ Listener trên `window` **không** đọc biến này trực tiếp — nó đọc một proxy ổn định
 * dựng trong [`attachKeyboard`]. Xem lý do ở đó; nó là một cái bẫy đã đo được.
 */
let keymap: Keymap | null = null

/**
 * Nền tảng đã dùng lúc cài đặt, giữ lại cho mọi lượt dựng lại. Story 1.21.
 *
 * ⚠️ Không gọi lại `detectIsMac()` ở lượt dựng lại: Kiểm D tiêm `isMac` để lái hai ca, và
 * một lượt đọc `navigator` giữa chừng làm keymap sau lượt gán phím khác keymap lúc khởi
 * động trên chính máy đó.
 */
let installedIsMac = false

/**
 * Lớp hợp âm của NGƯỜI DÙNG đang có hiệu lực — đĩa, cộng mọi lượt gán trong phiên này.
 *
 * `{}` nghĩa là *"không ai đè lên gì"*, tức đang chạy nguyên bộ mặc định của sản phẩm.
 */
let liveOverrides: ChordOverrides = {}

/**
 * Vì sao hợp âm đọc từ đĩa **không được áp**, hoặc `null` nếu chúng đã được áp.
 *
 * 🔴 Story 1.21 · AC13 · đóng `deferred-work.md:243`. Cho tới story này chẩn đoán đó chỉ đi
 * ra `console.error` — im lặng theo nghĩa thực dụng, vì người dùng chỉ biết nếu họ mở
 * console. Nay màn hình phím tắt đọc được nó và nói ra một câu.
 */
let diskRejection: string | null = null

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

  // ── Story 2.5 — xác nhận segment (FR24 · AD-31) ─────────────────────────────────
  //
  // ⚠️ TIÊM VÀO, cùng cửa và cùng lý do với `runLookup`: cài đặt sống ở
  // `src/panels/editorPanelState.ts`, dùng `ref`/`shallowRef` của Vue **và** `invoke` của
  // `@tauri-apps/api` — import thẳng nó ở đây giết Kiểm C/D/E cùng một lượt.

  /**
   * Xác nhận câu đang có con trỏ trong Panel Editor. Handler của `editor.confirm_segment`.
   *
   * ⚠️ Cài đặt thật là `async`; kiểu `() => void` ở đây khớp cùng khuôn `runLookup` —
   * promise trả về bị bỏ qua có chủ ý, trạng thái đi ra qua `ref` ở tầng module.
   *
   * 🔴 Cổng này **phải** là đường đã flush xong rồi mới `invoke` (AD-35 vế (c)). Một chỗ nối
   * tương lai cắm thẳng `confirmSegment` của `config/segment.ts` vào đây sẽ ký một văn bản
   * cũ hơn thứ người dùng đang nhìn, và **không cổng nào bắt được** — xem doc-comment của
   * `editorPanelState.ts::confirmCurrentSegment`.
   */
  confirmSegment?: () => void

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

  // ── Story 1.19 — bật/tắt nguồn từ điển và bề mặt ghi công ──────────────────────
  //
  // ⚠️ TIÊM VÀO, cùng cửa và cùng lý do với `runLookup`: `panels/dictSourcesState.ts` dùng
  // `ref`/`computed` của Vue **và** gọi `@tauri-apps/api` xuyên qua `config/dict.ts` —
  // import thẳng nó ở đây giết Kiểm C/D/E cùng lúc.

  /**
   * Bật/tắt nguồn từ điển **đang được nhắm**. Handler của `lookup.toggle_source` (AC2).
   *
   * 🔴 **Không** một tham số `code`, và đó là §KHÔNG-LÀM ⑤ viết thành chữ ký: danh sách
   * nguồn **dẫn xuất lúc chạy** (0 tới 10 nguồn tuỳ tệp `.db` có mặt), còn `CommandRegistry`
   * là một danh sách TĨNH mà `check-commands.mjs` đếm bằng máy (`COMMAND_FLOOR`). Một
   * command cho mỗi nguồn phá chính cơ chế cưỡng chế của AD-34, và một id không tồn tại lúc
   * dựng màn hình phím thì Story 1.21 không gán lại được. ⇒ handler đọc mục tiêu từ trạng
   * thái quanh nó, đúng khuôn `deps.currentSelection` của `lookup.lookup_selection`.
   */
  toggleDictSource?: () => void
  /** Mở bề mặt ghi công. Handler của `attribution.open` (AC7, AC11). */
  openAttribution?: () => void
  /** Đóng bề mặt ghi công. Handler của `attribution.close` (AC11). */
  closeAttribution?: () => void

  // ── Story 1.20 — lịch sử tra cứu và mục đã ghim ────────────────────────────────
  //
  // ⚠️ TIÊM VÀO, cùng cửa và cùng lý do với `toggleDictSource`: `panels/lookupHistoryState.ts`
  // dùng `ref`/`computed` của Vue **và** gọi `@tauri-apps/api` xuyên qua `config/pinned.ts`
  // — import thẳng nó ở đây giết Kiểm C/D/E cùng lúc.

  /** Chọn tab của Panel Lookup. Handler của `lookup.select_tab_*` (AC5, AC6). */
  selectLookupTab?: (tab: 'record' | 'history') => void
  /**
   * Ghim hoặc bỏ ghim mục từ **đang xem**. Handler của `lookup.toggle_pin` (AC2).
   *
   * 🔴 **Không** một tham số `entry_id`, và đó là §KHÔNG-LÀM ⑤ viết thành chữ ký — cùng
   * lý lẽ đã ghi cho `toggleDictSource` ngay trên: một command cho mỗi mục ghim phá chính
   * cơ chế đếm tĩnh mà `check-commands.mjs` dùng (`COMMAND_FLOOR`), và một id không tồn
   * tại lúc dựng màn hình phím thì Story 1.21 không gán lại được. ⇒ handler đọc mục tiêu
   * từ trạng thái quanh nó.
   */
  toggleLookupPin?: () => void
  /** Xoá lịch sử tra cứu của phiên. Handler của `lookup.clear_history` (AC6). */
  clearLookupHistory?: () => void

  // ── Story 1.21 — màn hình phím tắt ─────────────────────────────────────────────
  //
  // ⚠️ TIÊM VÀO, cùng cửa và cùng lý do với `toggleLookupPin`: `config/shortcutsState.ts`
  // dùng `ref`/`computed` của Vue **và** gọi `@tauri-apps/api` xuyên qua `config/bootstrap.ts`
  // — import thẳng nó ở đây giết Kiểm C/D/E cùng lúc.

  /** Mở lớp phủ phím tắt. Handler của `shortcuts.open` (AC1). */
  openShortcuts?: () => void
  /** Đóng lớp phủ phím tắt. Handler của `shortcuts.close` (AC1). */
  closeShortcuts?: () => void
  /**
   * Vào trạng thái **chờ một hợp âm** cho thao tác đang nhắm. Handler của
   * `shortcuts.capture` (AC2, AC10).
   *
   * 🔴 **Không** một tham số `command_id` — cùng lý lẽ đã ghi cho `toggleDictSource` và
   * `toggleLookupPin` ngay trên. Bảng có một hàng cho mỗi command đang đăng ký, và Kiểm A
   * đòi mỗi `@click` là **đúng một** `dispatch('<id>')` với id **literal** ⇒ hàng đang
   * nhắm đi bằng `@mousedown`, không bằng tham số.
   */
  captureShortcut?: () => void
  /**
   * Bỏ gán phím của thao tác đang nhắm — ghi một chuỗi **rỗng** xuống đĩa (AC8).
   *
   * ⚠️ **Khác** `resetShortcut`: rỗng nghĩa là *"thao tác này cố ý không có phím"*, một
   * phát biểu lưu được. Xem `ChordOverrides` ở `./keys.ts` về ba trạng thái.
   */
  unassignShortcut?: () => void
  /**
   * Trả phím của thao tác đang nhắm về mặc định của sản phẩm — **xoá khoá** khỏi đĩa (AC8).
   *
   * ⚠️ Xoá chứ không ghi đè bằng chính hợp âm mặc định: ghi đè biến hàng đó thành một giá
   * trị ĐÓNG BĂNG, nên một story sau đổi hợp âm mặc định thì người đã bấm nút này một lần
   * mắc kẹt ở giá trị cũ mãi mãi, không dấu hiệu nào. Ice chốt 2026-08-11.
   */
  resetShortcut?: () => void

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
 * Đăng ký **BỘ MẶC ĐỊNH CỦA SẢN PHẨM** vào một registry.
 *
 * ─────────────────────────────────────────────────────────────────────────────
 * 🔴 STORY 1.21 ĐỔI Ý NGHĨA CỦA HÀM NÀY, VÀ ĐÓ LÀ THAY ĐỔI KIẾN TRÚC LỚN NHẤT CỦA NÓ
 * ─────────────────────────────────────────────────────────────────────────────
 * Cho tới Story 1.20, hàm này nhận một tham số `bindings` *(hợp âm đọc từ `global.db`)* và
 * **nướng** nó vào `spec.keys` ngay lúc đăng ký. Nay nó không nhận nữa: hợp âm ở đây là
 * mặc định của sản phẩm, **luôn luôn**, và lớp của người dùng sống ở một tầng riêng
 * (`ChordOverrides` của `./keys.ts`).
 *
 * Vì sao phải đổi — một phép đo, không một sở thích: AC8 đòi *"trả về mặc định"* phân biệt
 * được với *"bỏ gán"*. Với mô hình cũ, `spec.keys` là *"đĩa-hoặc-mặc-định lúc khởi động"*,
 * nên **không có chỗ nào trong tiến trình còn giữ mặc định thật** để mà trả về. Một người
 * dùng đã gán `Mod+9` cho `mode.library` từ phiên trước sẽ bấm "trả về mặc định" và nhận
 * lại `Mod+9`. Sai, và sai mà 0 cổng đỏ.
 *
 * ⇒ Sau story này `spec.keys` có đúng **một** nghĩa ở mọi chỗ đọc nó — kể cả trong
 * `check-commands.mjs`, nơi nó vốn đã chỉ có nghĩa đó vì Node thuần không có đĩa.
 */
function registerAll(target: Registry, deps: CommandDeps): void {
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
      keys: [`Mod+${MODE_IDS.indexOf(mode) + 1}`],
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
      keys: [`Mod+Alt+${preset === 'grid' ? 1 : 2}`],
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
   * ⚠️ `keys: undefined` là **mặc định của sản phẩm**, không một cái khoá: nếu `global.db`
   * có một hợp âm cho id này thì nó ĐƯỢC dùng, qua lớp `overrides` mà Story 1.21 dựng
   * (xem [`applyBindings`]). Gán phím là quyền của người dùng, và màn hình phím tắt là chỗ
   * để làm việc đó.
   */
  for (const suffix of PANEL_SUFFIXES) {
    const id = `layout.toggle_${suffix}`
    target.register({
      id,
      labelKey: `command.${id}`,
      keys: undefined,
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
      keys: [`Mod+Alt+Arrow${step > 0 ? 'Right' : 'Left'}`],
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
    keys: ['Mod+Alt+O'],
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
    keys: ['Mod+Alt+J'],
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
    keys: ['Mod+Alt+V'],
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
    keys: ['Mod+Alt+L'],
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
    keys: ['Mod+Alt+S'],
    run: () => {
      if (deps.focusSelectionSource === undefined) {
        return portMissing('selection.focus_source', 'focusSelectionSource')
      }
      deps.focusSelectionSource()
    },
  })

  /**
   * ═══════════════════════════════════════════════════════════════════════════════
   * 🔴 STORY 1.19 — BA COMMAND TĨNH CHO BẬT/TẮT NGUỒN VÀ GHI CÔNG (AC11)
   * ═══════════════════════════════════════════════════════════════════════════════
   *
   * ─────────────────────────────────────────────────────────────────────────────
   * 🔴 VÌ SAO **KHÔNG** MỘT COMMAND CHO MỖI NGUỒN (§KHÔNG-LÀM ⑤)
   * ─────────────────────────────────────────────────────────────────────────────
   * `mockups/sources-attribution.html:140` vẽ `⌥1…6` cạnh dải chip. **Bác**, ba lý do đo
   * được:
   * ① danh sách nguồn **dẫn xuất lúc chạy** — 0 tới 10 nguồn tuỳ tệp `.db` có mặt — còn
   *    registry này là một danh sách **TĨNH** mà `check-commands.mjs` đếm bằng máy
   *    (`COMMAND_FLOOR`). Một command sinh động phá chính cơ chế cưỡng chế của AD-34;
   * ② `Mod+Alt+1`/`Mod+Alt+2` **đã thuộc** preset bố cục (Story 1.14, §QĐ #1);
   * ③ FR22/Story 1.21 đòi **mọi** command gán lại được, mà một id không tồn tại lúc dựng
   *    màn hình phím thì không gán được.
   * Ghi lệch so với mockup vào §Change Log của story, kèm ba lý do trên.
   *
   * ─────────────────────────────────────────────────────────────────────────────
   * ⚠️ **BA, KHÔNG PHẢI HAI** — và lệch với §KHÔNG-LÀM ⑤ đúng một command
   * ─────────────────────────────────────────────────────────────────────────────
   * Story kê *"đúng hai command tĩnh"*. `attribution.close` là command thứ ba, và lý do là
   * một **ràng buộc của cổng**, không một tính năng thêm: Kiểm A của `check:commands` đòi
   * **mọi** `@click` là đúng một `dispatch('<id>')`, nên một nút đóng — thứ AC11 cần cho
   * người dùng chuột — **không tồn tại được** nếu không có command của nó. Ba lý do mà
   * §KHÔNG-LÀM ⑤ đưa ra vẫn đứng nguyên: id **tĩnh**, đếm được, gán lại được ở 1.21.
   * (`Escape` thì KHÔNG đi qua registry — nó là một lượt huỷ **trong ngữ cảnh**, và chiếm
   * `Escape` cho cả ứng dụng là một quyết định khác hẳn.)
   *
   * 🔴 **Cố ý KHÔNG gán phím mặc định cho cả ba**, cùng lý lẽ `layout.toggle_*` (§QĐ #3 của
   * Story 1.6): họ `Mod+Alt+…` đã kín chỗ có nghĩa (`1` `2` `O` `J` `V` `L` `S` `←` `→`), và
   * ba thao tác này đều tới được bằng bàn phím qua Tab + Enter/Space — chuẩn HTML gốc. Đây
   * là một **lỗ NFR17 có tên và có chủ**: màn hình gán phím là Story 1.21, và kể từ story
   * đó một hợp âm người dùng tự đặt trong `global.db` **ĐƯỢC** dùng — qua lớp `overrides`,
   * không qua `spec.keys`.
   */
  for (const [id, port] of [
    ['lookup.toggle_source', 'toggleDictSource'],
    ['attribution.open', 'openAttribution'],
    ['attribution.close', 'closeAttribution'],
  ] as const) {
    target.register({
      id,
      labelKey: `command.${id}`,
      keys: undefined,
      run: () => {
        const handler = deps[port]
        if (handler === undefined) return portMissing(id, port)
        handler()
      },
    })
  }

  /**
   * ═══════════════════════════════════════════════════════════════════════════════
   * 🔴 STORY 1.20 — BỐN COMMAND CHO LỊCH SỬ TRA CỨU VÀ MỤC ĐÃ GHIM (AC6)
   * ═══════════════════════════════════════════════════════════════════════════════
   *
   * ─────────────────────────────────────────────────────────────────────────────
   * 🔴 ID ĐẶT THEO MÃ THẬT, KHÔNG THEO `settings.html` — Quyết định #7
   * ─────────────────────────────────────────────────────────────────────────────
   * `mockups/settings.html:282` ghi mã lệnh `lookup.entry.pin`. Cùng bảng đó ghi
   * `lookup.query.selection` cho Auto-Lookup — mã thật là **`lookup.lookup_selection`** —
   * và ghi `editor.segment.nextUntranslated`, thứ **vi phạm văn phạm id** của chính
   * registry: `COMMAND_ID_RE` (`./registry.ts`) không cho chữ hoa, nên `register()` sẽ
   * **ném**. ⇒ cả bảng đó là phác thảo trước khi văn phạm được chốt ở Story 1.6. Id ở đây
   * theo khuôn hai đoạn của `source.select_tab_original`/`lookup.toggle_source`.
   *
   * ─────────────────────────────────────────────────────────────────────────────
   * 🔴 `Mod+D` — ĐÚNG MỘT NGOẠI LỆ SO VỚI TIỀN LỆ "0 PHÍM MẶC ĐỊNH" CỦA STORY 1.19
   * ─────────────────────────────────────────────────────────────────────────────
   * Ice ký 2026-08-10, và ba phép đo đứng sau nó:
   * ① **trống hoàn toàn** — `grep -rn "Mod+D\|KeyD" src/ scripts/` trả **0** lần (đo lại
   *    2026-08-10 trước khi gõ dòng này); hợp âm đang chiếm là `Mod+1/2/3` · `Mod+Alt+1/2`
   *    · `Mod+Alt+O/J/V/L/S` · `Mod+Alt+←/→` · `Shift+Arrow…` · `Alt+Shift+Arrow…`. Không
   *    menu Tauri nào tranh (`tauri.conf.json` không khai menu);
   * ② **luật vùng gõ không chặn nó** — `./keys.ts` áp luật vùng gõ **chỉ khi**
   *    `lacksPrimaryMod`, và `Mod+D` mang phím bổ trợ chính ⇒ ghim được kể cả khi caret
   *    đang trong một ô nhập, đúng lời hứa NFR17;
   * ③ `createKeymap` không ném — không command nào đang giành hợp âm này.
   *
   * 🔴 **`⌘⌫` của mockup bị BÁC** cho `lookup.clear_history`, và bằng chính phép đo ②:
   * một hợp âm mang `Mod` **không** bị luật vùng gõ chặn, nên `Mod+Backspace` sẽ cướp
   * *"xoá tới đầu dòng"* của macOS ngay giữa lúc người dùng đang gõ — để chạy một thao tác
   * **phá hoại**. Xoá lịch sử tới được bằng Tab + Enter, và gán lại được ở Story 1.21.
   *
   * ⚠️ **Rủi ro đã biết, hôm nay bằng 0:** một `global.db` đã gán `Mod+D` cho command khác
   * làm `createKeymap` ném, và `installCommands()` chạy trước `mount()` (§Bẫy 5). Hôm nay
   * chưa có đường nào để người dùng gán phím — màn hình gán phím là Story 1.21, và chính
   * nó phải xử xung đột chứ không im lặng ghi đè.
   *
   * ⚠️ Viết **`Mod`**, không `Cmd`/`Ctrl`: `./keys.ts` là tầng trung lập nền tảng, và một
   * hợp âm viết cứng theo một hệ điều hành là đúng thứ Kiểm D (NFR14) tồn tại để chặn.
   *
   * ⚠️ HAI command CHỌN tab (không MỘT command toggle) — cùng khuôn `source.select_tab_*`:
   * bấm đúng tab đang chọn là một thao tác VÔ HẠI, không lật sang tab kia.
   */
  for (const [id, tab] of [
    ['lookup.select_tab_record', 'record'],
    ['lookup.select_tab_history', 'history'],
  ] as const) {
    target.register({
      id,
      labelKey: `command.${id}`,
      keys: undefined,
      run: () => {
        if (deps.selectLookupTab === undefined) return portMissing(id, 'selectLookupTab')
        deps.selectLookupTab(tab)
      },
    })
  }

  target.register({
    id: 'lookup.toggle_pin',
    labelKey: 'command.lookup.toggle_pin',
    keys: ['Mod+D'],
    run: () => {
      if (deps.toggleLookupPin === undefined) return portMissing('lookup.toggle_pin', 'toggleLookupPin')
      deps.toggleLookupPin()
    },
  })

  target.register({
    id: 'lookup.clear_history',
    labelKey: 'command.lookup.clear_history',
    keys: undefined,
    run: () => {
      if (deps.clearLookupHistory === undefined) {
        return portMissing('lookup.clear_history', 'clearLookupHistory')
      }
      deps.clearLookupHistory()
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
      keys: [chord],
      // 🔴 BỐN CHỖ DUY NHẤT khai `repeatable` — Story 1.21, đóng `deferred-work.md:656`.
      //
      // Giữ `Shift+→` là cách người ta bôi đen một cụm từ; mở rộng đúng một ký tự rồi
      // đứng im là *"bấm mà không có gì xảy ra"* (AD-44 ④). Bốn thao tác này luỹ tiến và
      // rẻ — chúng chạm `Selection` của DOM và không ghi đĩa, không dựng lại bố cục,
      // không một vòng IPC. Xem doc-comment của `CommandSpec.repeatable` về vì sao mọi
      // command khác giữ mặc định KHÔNG.
      repeatable: true,
      run: () => {
        const handler = deps[port]
        if (handler === undefined) return portMissing(id, port)
        handler()
      },
    })
  }

  /**
   * ═══════════════════════════════════════════════════════════════════════════════
   * 🔴 STORY 2.5 — `editor.confirm_segment` (FR24 · AD-31 · AC5)
   * ═══════════════════════════════════════════════════════════════════════════════
   *
   * **Command ĐẦU TIÊN mang tiền tố miền `editor.`**, và việc chốt tiền tố đó là một quyết
   * định có tuổi thọ dài hơn story này: Story 2.8/2.9/2.10 sẽ dùng tiếp
   * (`editor.merge_segments`, `editor.split_segment`, `editor.next_segment`…). Ice ký
   * 2026-08-14, Quyết định #4.
   *
   * ─────────────────────────────────────────────────────────────────────────────
   * 🔴 CHỌN HỢP ÂM `Mod+Enter` — và một xung đột ĐÃ BIẾT, ghi ra thay vì để người sau gặp
   * ─────────────────────────────────────────────────────────────────────────────
   * `EXPERIENCE.md:169` đã dùng `⌘↵` cho *"xác nhận nhập"* ở màn xem trước — **cùng ngữ
   * nghĩa "ký duyệt"**, khác bề mặt. Hôm nay `⌘↵` chưa ai đăng ký, nên nó rảnh.
   *
   * ⚠️ **GIỚI HẠN THẬT:** `check:commands` kiểm trùng hợp âm **trên toàn bộ registry**,
   * không theo chế độ. Ngày Epic 6 đăng ký lệnh *"xác nhận nhập"* cũng bằng `⌘↵`, cổng sẽ
   * **đỏ** và một trong hai phải nhường. Món này ghi trong `deferred-work.md` với chủ là
   * **Story 6.2** — nó không được lộ ra dưới dạng một cổng đỏ không ai hiểu.
   *
   * ⚠️ Cổng chỉ kiểm trùng **nội bộ** bộ command; nó **không biết gì** về phím của hệ điều
   * hành. Lưới ở đây là con người — tiền lệ `⌘⌥H` bị macOS nuốt (Ice chốt đổi 2026-08-06).
   *
   * ─────────────────────────────────────────────────────────────────────────────
   * 🔴 VÌ SAO MỘT COMMAND, KHÔNG MỘT `@keydown` TRÊN BỀ MẶT GÕ
   * ─────────────────────────────────────────────────────────────────────────────
   * AD-34 §1 — sàn khả năng tiếp cận là **cấu trúc**, không kỷ luật. Và có một cái bẫy cụ
   * thể ở đây: **Kiểm A của `check:commands` chỉ canh `@click`** (`deferred-work.md:166`).
   * Một `@keydown` gọi thẳng hàm dựng một đường thứ hai mà **không cổng nào nhìn thấy** —
   * nên mọi bề mặt phải phát **cùng một** `dispatch('editor.confirm_segment')`.
   *
   * ⚠️ `Mod+Enter` **mang `Meta`/`Ctrl`**, nên nó đi qua được luật vùng gõ của `keys.ts:287`
   * (`lacksPrimaryMod && isTypingZone`) — tức nó vẫn bắn khi con trỏ đang ở trong câu đang
   * gõ, đúng chỗ duy nhất mà thao tác này có nghĩa.
   */
  target.register({
    id: 'editor.confirm_segment',
    labelKey: 'command.editor.confirm_segment',
    keys: ['Mod+Enter'],
    run: () => {
      if (deps.confirmSegment === undefined) {
        return portMissing('editor.confirm_segment', 'confirmSegment')
      }
      deps.confirmSegment()
    },
  })
  // ── Story 1.21 — màn hình phím tắt ────────────────────────────────────────────
  //
  // 🔴 NĂM ID TĨNH, và đó là §KHÔNG-LÀM ⑤ viết thành chữ ký — cùng khuôn `toggleDictSource`
  // và `toggleLookupPin`. Bảng phím tắt có một hàng cho MỖI command đang đăng ký (29+ hàng,
  // và con số đó lớn lên mỗi story), nên "một command cho mỗi hàng" phá chính cơ chế đếm
  // tĩnh mà `check-commands.mjs` dùng (`COMMAND_FLOOR`) — và một id không tồn tại lúc dựng
  // màn hình thì chính màn hình này không gán lại được cho nó.
  // ⇒ handler đọc **hàng đang nhắm** từ trạng thái quanh nó, tại thời điểm chạy.
  //
  // ⚠️ Bốn command dưới `shortcuts.open` giữ **0 hợp âm mặc định**, và đó là chủ ý kép:
  // họ `Mod+Alt+…` đã kín chỗ có nghĩa, cả bốn tới được bằng Tab + Enter/Space bên trong
  // lớp phủ, VÀ chúng là nhiên liệu cho `unbound()` — xem `check-commands.mjs:1398`.
  for (const [id, port, chord] of [
    // `Mod+Comma` — `⌘,` là quy ước Preferences của macOS, `Comma` có sẵn trong
    // `NAMED_CODES` (`keys.ts:112`), và hợp âm đó chưa ai chiếm.
    ['shortcuts.open', 'openShortcuts', 'Mod+Comma'],
    ['shortcuts.close', 'closeShortcuts', undefined],
    ['shortcuts.capture', 'captureShortcut', undefined],
    ['shortcuts.unassign', 'unassignShortcut', undefined],
    ['shortcuts.reset', 'resetShortcut', undefined],
  ] as const) {
    target.register({
      id,
      labelKey: `command.${id}`,
      keys: chord === undefined ? undefined : [chord],
      run: () => {
        const handler = deps[port]
        if (handler === undefined) return portMissing(id, port)
        handler()
      },
    })
  }
}

/**
 * Đăng ký bộ command khởi động và dựng keymap. Gọi **một lần**, từ `src/main.ts`.
 *
 * ⚠️ Gọi lần thứ hai ⇒ `register()` ném vì id trùng. Đó là hành vi ĐÚNG (AC2) chứ không
 * phải một chỗ cần nới: hai lượt cài đặt trong một tiến trình nghĩa là hai keymap cùng
 * nghe một cửa sổ, và cái sau sẽ dispatch mọi hợp âm hai lần.
 *
 * ─────────────────────────────────────────────────────────────────────────────
 * 🔴 §Bẫy 5 — **hợp âm đọc từ đĩa gây xung đột ⇒ ứng dụng không mở được**
 * ─────────────────────────────────────────────────────────────────────────────
 * `createKeymap` **ném** khi hai command trùng hợp âm, và hàm này chạy **trước `mount()`**.
 * Một `global.db` sửa tay *(hoặc một lượt nhập cấu hình từ máy khác)* là đủ để một cú ném ở
 * đó cho ra **cửa sổ trắng** — và người dùng mất luôn đường vào để sửa chính cái làm hỏng.
 *
 * Chốt: dựng thử **trước**, rơi về bộ mặc định nếu trượt, và ứng dụng lên bình thường.
 *
 * ⚠️ Story 1.20 làm việc đó bằng một registry **nháp**, vì lớp hợp âm của đĩa khi ấy nằm
 * trong chính `spec.keys` nên thử nó đòi một lượt `register()` thứ hai. Story 1.21 gỡ nhu
 * cầu đó: lớp của đĩa nay là tham số `overrides` của `createKeymap`, và `createKeymap`
 * **chỉ đọc** registry. ⇒ thử ngay trên registry thật, 0 registry nháp, và một biến thể
 * ít hơn để hai đường trôi khỏi nhau.
 *
 * 🔴 Và chẩn đoán không dừng ở `console.error` nữa — nó vào [`shortcutsDiskRejection`] để
 * màn hình phím tắt nói ra một câu (AC13, đóng `deferred-work.md:243`).
 */
export function installCommands(deps: CommandDeps): Keymap {
  const isMac = deps.isMac ?? detectIsMac()
  installedIsMac = isMac

  // Bộ MẶC ĐỊNH của sản phẩm, luôn luôn — xem doc-comment của `registerAll`. Sau dòng này
  // `spec.keys` là câu trả lời cho *"mặc định của thao tác này là gì"*, và AC8 đứng lên nó.
  registerAll(registry, deps)

  const fromDisk = deps.bindings
  if (fromDisk !== undefined) {
    try {
      keymap = createKeymap(registry, { isMac }, fromDisk)
      liveOverrides = fromDisk
      diskRejection = null
      return keymap
    } catch (err) {
      diskRejection = String(err)
      console.error(
        '[commands] hợp âm đọc từ `global.db` không dựng được keymap — rơi về hợp âm mặc ' +
          'định. Lựa chọn phím tắt KHÔNG bị xoá, chỉ tạm không áp; màn hình phím tắt là ' +
          `chỗ sửa. Nguyên nhân: ${diskRejection}`,
      )
    }
  }

  // Đường mặc định. ⚠️ Một lần ném Ở ĐÂY là lỗi lập trình *(hai hằng số hợp âm trong chính
  // tệp này giành nhau)*, không phải lỗi dữ liệu của người dùng — nên nó ĐƯỢC ném lên, và
  // `src/main.ts` vẽ nó ra thay vì để lại một cửa sổ trắng.
  keymap = createKeymap(registry, { isMac })
  liveOverrides = {}
  return keymap
}

/**
 * ═══════════════════════════════════════════════════════════════════════════════
 * 🔴 STORY 1.21 — BỀ MẶT LÚC CHẠY CỦA MÀN HÌNH PHÍM TẮT
 * ═══════════════════════════════════════════════════════════════════════════════
 *
 * Sáu hàm dưới đây là **cửa duy nhất** mà `src/config/shortcutsState.ts` và
 * `src/ShortcutsOverlay.vue` được đi qua. Vì sao chúng ở đây chứ không ở lớp giao diện:
 * `registry` và `keymap` là biến module của tệp này, và một bản sao của chúng ở tầng trên
 * là đúng thứ §Dev Notes ⑤ cảnh báo — hai nguồn cho một sự thật.
 *
 * ⚠️ **KHÔNG** đọc `commandRegistry.unbound()` hay `spec.keys` từ màn hình. Cả hai trả lời
 * *thời điểm cài đặt* kể từ story này; doc-comment ở `./registry.ts` ghi mệnh đề đầy đủ.
 */

/** Mọi hợp âm ĐANG CÓ HIỆU LỰC, kèm chuỗi đã phân giải. Nguồn lúc chạy của AC12. */
export function effectiveBindings(): readonly Binding[] {
  return keymap === null ? [] : keymap.bindings()
}

/**
 * Các thao tác **chưa gán phím nào** lúc chạy — `list()` trừ đi các id có trong
 * [`effectiveBindings`]. AC5 · AC12.
 *
 * ⚠️ Khác `commandRegistry.unbound()`, và khác biệt đó là cả nội dung của AC12: một lượt
 * gán phím **không** đi qua `register()`, nên `unbound()` không bao giờ biết nó đã xảy ra.
 */
export function effectiveUnbound(): readonly CommandSpec[] {
  const bound = new Set(effectiveBindings().map((b) => b.id))
  return registry.list().filter((spec) => !bound.has(spec.id))
}

/**
 * Hợp âm **mặc định của sản phẩm** cho một thao tác — thứ mà nút *"trả về mặc định"* khôi
 * phục. AC8.
 *
 * Đọc thẳng `spec.keys`, và điều đó chỉ đúng vì `registerAll` không còn nướng đĩa vào đó
 * (xem doc-comment của nó). Mảng rỗng ⇒ sản phẩm cố ý không gán phím cho thao tác này.
 */
export function defaultChordsFor(id: CommandId): readonly string[] {
  return registry.list().find((spec) => spec.id === id)?.keys ?? []
}

/**
 * Lớp của NGƯỜI DÙNG cho một thao tác, hoặc `null` nếu họ chưa đè lên gì.
 *
 * ⚠️ `null` *(chưa ai đè)* và `[]` *(cố ý không có phím)* là **hai** câu trả lời khác nhau
 * — đó là cả AC8. Xem `ChordOverrides` ở `./keys.ts` về ba trạng thái.
 */
export function overrideFor(id: CommandId): readonly string[] | null {
  return Object.prototype.hasOwnProperty.call(liveOverrides, id) ? liveOverrides[id] : null
}

/** Toàn bộ lớp người dùng đang có hiệu lực. Màn hình dựng lượt gán tiếp theo từ đây. */
export function currentOverrides(): ChordOverrides {
  return liveOverrides
}

/** Xem [`diskRejection`]. `null` ⇒ hợp âm trên đĩa đã được áp bình thường. AC13. */
export function shortcutsDiskRejection(): string | null {
  return diskRejection
}

/** Một thao tác khác đang giữ đúng phím này. Xem [`conflictFor`]. */
export type ChordConflict = {
  /** Hợp âm nguyên văn của **đối thủ**, để hiện lên màn hình. */
  chord: string
  /** Khoá phân giải chung của cả hai. */
  resolved: string
  heldBy: CommandId
}

/**
 * Hợp âm này có đụng một thao tác KHÁC không? AC3 · AC9.
 *
 * 🔴 So trên **`resolved`**, không trên chuỗi hợp âm. Trên macOS `Mod+D` và `Meta+D` là hai
 * chuỗi khác nhau nhưng cùng phân giải thành `Meta+KeyD` — một phép so chuỗi để lọt đúng ca
 * đó và cho ra hai command giành một phím mà màn hình nói *"không xung đột"*.
 *
 * `null` cũng là câu trả lời cho một hợp âm **không phân giải được**: chỗ gọi đã tự bắt ca
 * đó bằng `chordFromEvent` trả `null` (AC11), và dựng một xung đột-ma ở đây thì tệ hơn.
 */
export function conflictFor(chord: string, id: CommandId): ChordConflict | null {
  let resolved: string
  try {
    resolved = resolveChord(chord, { isMac: installedIsMac })
  } catch {
    return null
  }
  for (const binding of effectiveBindings()) {
    if (binding.resolved === resolved && binding.id !== id) {
      return { chord: binding.chord, resolved, heldBy: binding.id }
    }
  }
  return null
}

/** Kết quả một lượt dựng lại keymap. `detail` chỉ để chẩn đoán — nó không đi lên giao diện. */
export type ApplyOutcome = { ok: true } | { ok: false; detail: string }

/**
 * Áp một lớp hợp âm mới **NGAY**, không khởi động lại. AC2 · AC12.
 *
 * ─────────────────────────────────────────────────────────────────────────────
 * 🔴 §Bẫy 9 — DỰNG XONG MỚI THAY, và thứ tự đó là cả tính an toàn của hàm
 * ─────────────────────────────────────────────────────────────────────────────
 * `createKeymap` ném khi lớp mới gây xung đột. Thay biến `keymap` **trước** khi biết lượt
 * dựng có thành công không — hoặc bắt lỗi rồi để `keymap` thành `null` — là **toàn bộ bàn
 * phím ứng dụng chết** sau một lượt gán sai, và người dùng mất luôn đường bàn phím để sửa
 * nó. Dựng vào một biến tạm, gán khi và chỉ khi thành công.
 *
 * Không ném: một lượt gán trượt là một câu trả lời cho màn hình, không một sự cố.
 *
 * ⚠️ Hàm này **không** ghi đĩa. Ghi là việc của `src/config/shortcutsState.ts`, và tách
 * đôi có chủ: một lượt áp trong phiên phải chạy được kể cả khi lượt ghi trượt (AD-21).
 */
export function applyBindings(next: ChordOverrides): ApplyOutcome {
  if (keymap === null) {
    return { ok: false, detail: 'applyBindings() gọi trước installCommands()' }
  }
  let rebuilt: Keymap
  try {
    rebuilt = createKeymap(registry, { isMac: installedIsMac }, next)
  } catch (err) {
    return { ok: false, detail: String(err) }
  }
  keymap = rebuilt
  liveOverrides = next
  // Lượt gán vừa rồi ĐÃ áp được, nên câu *"bộ phím trên đĩa chưa được áp"* hết đúng — để
  // nó ở lại là một câu báo lỗi sống lâu hơn cái lỗi nó mô tả.
  diskRejection = null
  return { ok: true }
}

/**
 * Gắn keymap vào cửa sổ. Trả về hàm gỡ.
 *
 * `gate` là cửa **nuốt hợp âm** — xem [`KeymapGate`]. Story 1.19 dùng nó để một lớp phủ khai
 * `aria-modal` hành xử **đúng như** nó khai: không một command toàn cục nào chạy phía sau nó.
 *
 * ⚠️ `noUnusedLocals` đang bật — dùng hàm gỡ hoặc `void` nó tường minh ở chỗ gọi.
 */
export function attachKeyboard(target: EventTarget, gate?: KeymapGate): () => void {
  if (keymap === null) {
    throw new Error('[commands] attachKeyboard() gọi trước installCommands() — không có keymap nào để gắn.')
  }
  /**
   * 🔴 STORY 1.21 — PROXY ỔN ĐỊNH, và nó vá một cái bẫy đo được.
   *
   * `attachKeymap(keymap, …)` đóng gói **đối tượng** được truyền vào: listener ở
   * `keys.ts` gọi `keymap.handle(event)` trên đúng tham số đó, không trên biến module của
   * tệp này. ⇒ [`applyBindings`] gán một keymap mới vào biến module và listener **không
   * bao giờ nhìn thấy nó** — triệu chứng là *"gán phím xong, phím mới không chạy, phím cũ
   * vẫn chạy"*, và không lỗi nào ném.
   *
   * ⚠️ Không gỡ-rồi-gắn-lại thay cho proxy: `attachKeymap` ném ở lần gắn thứ hai vào cùng
   * target (`keys.ts`), nên đường đó đòi giữ đúng thứ tự `dispose()` → `attach()` ở mỗi
   * lượt gán, và một lượt ném giữa chừng để cửa sổ **không còn listener nào**. Một lớp uỷ
   * quyền hai dòng thì không có trạng thái nào để làm hỏng.
   */
  const proxy: Keymap = {
    handle: (event) => (keymap === null ? false : keymap.handle(event)),
    bindings: () => (keymap === null ? [] : keymap.bindings()),
  }
  return attachKeymap(proxy, target, gate)
}
