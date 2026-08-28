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
 * 🔵 **SÁU** mục kể từ Story 2.5b (2026-08-14): ba chế độ + **ba** panel.
 * ⚠️ Mệnh đề cũ — *"BẢY mục kể từ Story 1.14: ba chế độ + bốn panel"* — đã hết đúng:
 * `panel.source` + `panel.editor` gộp thành `panel.grid` *(lưới hai cột đối chiếu)*. Trước
 * Story 1.14 là năm — hai panel của `WorkspaceMode` được dựng thẳng bằng `PanelFrame`.
 *
 * 🔴 Thứ tự ở đây là thứ tự KHAI BÁO, **không** phải thứ tự vòng xoay focus. Vòng xoay
 * đi theo lưới đang hiện (AC9) — xem `deps.panelRing`.
 */
export const FOCUS_OWNERS: readonly FocusOwner[] = [
  'mode.library',
  'mode.workspace',
  'mode.reading',
  'panel.grid',
  'panel.lookup',
  'panel.ai_translation',
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

  // ── Story 5.3 — "Quét lại thư mục" (FR99) ───────────────────────────────────────
  //
  // ⚠️ TIÊM VÀO, cùng cửa và cùng lý do với `submitPastedText`: state sống ở
  // `src/modes/libraryRescan.ts`, một module Vue thật (`ref`) — import thẳng nó ở đây giết
  // Kiểm C/D/E cùng lý do `@tauri-apps/api` bị cấm.

  /** Quét lại thư mục gốc đang cấu hình. Handler của `library.rescan` (AC1, AC2 — có phím
   * mặc định). ⚠️ Cài đặt thật là `async`; `() => void` khớp cùng khuôn `openGlossaryManage`
   * — promise trả về bị bỏ qua có chủ ý, kết quả đi ra qua các `ref` ở tầng module. */
  rescanLibraryFolder?: () => void
  /** Mở hộp thoại chọn thư mục, đổi thư mục gốc rồi quét lại. Handler của
   * `library.choose_root` (AD-48). */
  chooseLibraryRootFolder?: () => void
  /** Gỡ mục mồ côi đang chọn khỏi chỉ mục. Handler của `library.forget_orphan`. */
  forgetCurrentLibraryOrphan?: () => void
  /** Chuyển con trỏ xuống mục mồ côi kế tiếp. Handler của `library.orphan_next`. */
  nextLibraryOrphan?: () => void
  /** Chuyển con trỏ lên mục mồ côi trước. Handler của `library.orphan_prev`. */
  prevLibraryOrphan?: () => void

  // ── Story 5.4 — "Bốn trạng thái vòng đời" (FR5/FR6) ─────────────────────────────
  //
  // ⚠️ TIÊM VÀO, cùng cửa và cùng lý do với `rescanLibraryFolder`: state sống ở
  // `src/modes/libraryWorks.ts`, một module Vue thật (`ref`) — import thẳng nó ở đây giết
  // Kiểm C/D/E cùng lý do `@tauri-apps/api` bị cấm.

  /** Tải (hoặc tải lại) danh sách Tác phẩm theo bộ lọc hiện thời. Handler của
   * `library.list_works` (có phím mặc định). */
  loadLibraryWorks?: () => void
  /** Bật/tắt lọc theo *Chưa bắt đầu*. Handler của `library.filter_not_started`. */
  toggleLibraryFilterNotStarted?: () => void
  /** Bật/tắt lọc theo *Đang dịch*. Handler của `library.filter_in_progress`. */
  toggleLibraryFilterInProgress?: () => void
  /** Bật/tắt lọc theo *Tạm ngưng*. Handler của `library.filter_paused`. */
  toggleLibraryFilterPaused?: () => void
  /** Bật/tắt lọc theo *Đã xong*. Handler của `library.filter_done`. */
  toggleLibraryFilterDone?: () => void
  /** Bỏ mọi bộ lọc trạng thái đang bật. Handler của `library.filter_clear`. */
  clearLibraryFilter?: () => void
  /** Ghi đè trạng thái Tác phẩm đang mở thành *Tạm ngưng*. Handler của
   * `lifecycle.set_work_override_paused`. */
  setOpenWorkOverridePaused?: () => void
  /** Bỏ ghi đè trạng thái Tác phẩm đang mở. Handler của `lifecycle.clear_work_override`. */
  clearOpenWorkOverride?: () => void
  /** Đặt Chương đang mở thành *Đã xong*. Handler của `lifecycle.set_chapter_done`. */
  setOpenChapterDone?: () => void

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
   * phải hợp đồng vùng chọn dùng chung cho ba panel (đó là Story 1.18); nó là một dep
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

  /**
   * Nhảy tới **câu chưa dịch kế tiếp**. Handler của `editor.next_untranslated` (Story 2.5b,
   * AC12).
   *
   * ⚠️ Trả `boolean`, **không** `void`, và đó là một mệnh đề: `false` nghĩa *"không còn câu
   * nào chưa dịch ở phía dưới"* — một câu trả lời **hợp lệ**, không một lỗi. Chỗ gọi **kêu**
   * bằng một dòng chẩn đoán và để con trỏ ở nguyên; nó KHÔNG ném và KHÔNG tự quay vòng về
   * đầu Chương *(một lượt quay vòng im lặng đọc ra thành "phím này nhảy lung tung")*.
   */
  goToNextUntranslated?: () => boolean

  /**
   * Dời con trỏ sang segment **ngay sau** / **ngay trước**. Handler của `editor.next_segment`
   * và `editor.prev_segment` (Story 2.10, AC1 · AC2).
   *
   * ⚠️ Cùng hợp đồng `boolean` với [`goToNextUntranslated`], và cùng lý do — nhưng `false` ở
   * đây nghĩa **khác**: *"đã ở biên Chương"*, không *"đã dịch hết"*. Hai câu khác nhau nên
   * chúng là **hai** ô nhớ khác nhau ở thanh trạng thái.
   *
   * 🔴 Ba lệnh này **không** cùng luật lọc: next/prev là điều hướng **vị trí** nên chúng
   * **dừng** ở câu đã cắt bỏ *(Quyết định #3, Ice ký 2026-08-18)*, còn `next_untranslated` là
   * điều hướng **theo việc** nên nó bỏ qua. Lý do ở `segmentNavigation.ts::nextSegmentId`.
   */
  goToNextSegment?: () => boolean
  /** Xem [`goToNextSegment`]. */
  goToPrevSegment?: () => boolean

  /**
   * **Chuyển sang Chương kề** trong cùng Tác phẩm. Handler của `editor.next_chapter` và
   * `editor.prev_chapter` (Story 2.11, FR26 · AC1 · AC2 · AC3 · AC4).
   *
   * 🔴 **`void`, KHÔNG `boolean`** — và đây là chỗ hợp đồng của hai lệnh này khác ba lệnh
   * điều hướng ở trên, không một lượt viết cẩu thả. Một lượt chuyển Chương đi qua **hai**
   * lượt IPC nối tiếp *(flush theo AD-35, rồi `open_adjacent_chapter`)*, nên kết quả của nó
   * **chưa tồn tại** tại thời điểm `run()` phải trả về. Một `boolean` ở đây chỉ có thể là một
   * lời khai **đoán trước**, và nó sẽ nói *"đã đổi"* cho một lượt bị chặn.
   *
   * ⇒ Mọi kết cục — chặn vì flush trượt, biên Chương, lỗi IPC — đi ra bằng **thanh trạng
   * thái** bên trong `editorPanelState.ts::switchChapter`, không bằng giá trị trả về. Khuôn
   * fire-and-forget này đã có tiền lệ ở [`submitPastedText`]/[`submitFilePath`].
   */
  goToNextChapter?: () => void
  /** Xem [`goToNextChapter`]. */
  goToPrevChapter?: () => void

  /**
   * **Cắt bỏ / bỏ cờ** câu đang có con trỏ. Handler chung của `editor.omit_segment` và
   * `editor.restore_segment` (Story 2.5c, FR133 · AC1 · AC4).
   *
   * ⚠️ **MỘT** cổng cho **HAI** command, và đó không mâu thuẫn Quyết định #3 (Ice ký đường
   * (b) ngày 2026-08-15). Quyết định đó đòi *"hai lệnh, hai nhãn, trạng thái đọc được từ tên
   * lệnh"* — tức hai **id đăng ký**, và chúng có thật ngay dưới. Hai id ấy khác nhau ở đúng
   * một boolean; một cổng thứ hai ở đây chỉ nhân đôi một đường dây để nói cùng một điều.
   *
   * 🔴 Cắm `setSegmentOmitted` của `config/segment.ts` thẳng vào đây là **sai đường**: ảnh
   * chụp hiển thị của `editorSegments` sống ở `editorPanelState.ts::setCurrentSegmentOmitted`,
   * và một lượt nối tắt sẽ đổi đĩa mà **không** đổi lưới. Cùng cái bẫy `confirmSegment` ghi
   * ở trên.
   */
  setSegmentOmitted?: (omitted: boolean) => void

  /**
   * **Đặt / bỏ cờ kết đoạn của BẢN DỊCH** cho câu đang có con trỏ. Handler chung của
   * `editor.end_target_paragraph` và `editor.join_target_paragraph` (Story 2.5d, FR134 ·
   * AD-46 · AC2).
   *
   * ⚠️ **MỘT** cổng cho **HAI** command, cùng khuôn và cùng lý do với `setSegmentOmitted`
   * ngay trên: hai **id đăng ký** khác nhau ở đúng một boolean.
   *
   * 🔴 Cắm `setSegmentParagraphEnd` của `config/segment.ts` thẳng vào đây là **sai đường** —
   * ảnh chụp hiển thị sống ở `editorPanelState.ts::setCurrentSegmentParagraphEnd`.
   */
  setSegmentParagraphEnd?: (endsParagraph: boolean) => void
  /**
   * **Gộp câu đang có caret với câu liền trên nó** — Story 2.8, AC1 · AC8.
   *
   * ⚠️ Cùng cái bẫy mà `confirmSegment` đã ghi: đừng cắm thẳng `mergeSegments` của
   * `config/segment.ts` vào đây. Adapter đó chỉ ghi **đĩa**; lượt gộp còn phải **flush bộ
   * đệm gõ trước** *(bản dịch đi vào hàng mới đọc từ đĩa)* và **cập nhật mảng `segments`**
   * sau *(mốc so sánh của AD-47 ① sống ở webview)*. Cổng nối thật là
   * `editorPanelState.ts`, và `main.ts` nối hai đầu.
   */
  mergeSegments?: () => void
  /**
   * **Tách câu đang có caret tại chỗ cắt trong cột nguyên văn** — Story 2.8, AC2 · AC8.
   *
   * ⚠️ Cùng nghĩa vụ và cùng cái bẫy với `mergeSegments` ngay trên.
   */
  splitSegment?: () => void
  /**
   * **Xoá trọn tập điểm cắt đang chờ ở cột nguyên văn** — Story 2.9, AC8.
   *
   * ⚠️ Đây là một lượt xoá **state của webview**, không một lượt ghi đĩa — nên nó KHÔNG mang
   * nghĩa vụ flush của hai dep ngay trên. Nó vẫn đi qua `editorPanelState.ts` vì ô nhớ sống ở
   * đó, không vì một hợp đồng nào.
   */
  clearSourceCuts?: () => void

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

  // ── Story 2.6 — lịch sử phiên bản segment ──────────────────────────────────────
  //
  // ⚠️ TIÊM VÀO, cùng cửa và cùng lý do với `openShortcuts`: bề mặt lịch sử dùng `ref` của Vue
  // **và** gọi `@tauri-apps/api` xuyên qua `config/segment.ts` — import thẳng nó ở đây giết
  // Kiểm C/D/E cùng lúc, và Kiểm I thì `abort()` chứ không FAIL, tức nó **dừng hẳn** CI.

  /**
   * Mở lịch sử phiên bản của **câu đang nhắm**. Handler của `history.open` (AC1).
   *
   * 🔴 **Không** một tham số `segment_id`, và đó là cùng lý lẽ đã ghi cho `toggleDictSource`,
   * `toggleLookupPin` và `shortcuts.capture` ngay trên: một command cho mỗi segment phá chính
   * cơ chế đếm tĩnh mà `check-commands.mjs` dùng (`COMMAND_FLOOR`), và một id không tồn tại
   * lúc dựng màn hình phím thì Story 1.21 không gán lại được. ⇒ handler đọc **câu đang nhắm**
   * từ trạng thái quanh nó, tại thời điểm chạy.
   */
  openSegmentHistory?: () => void
  /** Đóng lịch sử phiên bản. Handler của `history.close` (AC1). */
  closeSegmentHistory?: () => void
  /**
   * Khôi phục **phiên bản đang nhắm**. Handler của `history.restore` (AC2).
   *
   * 🔴 **Không** một tham số `version_id` — cùng lý lẽ đã ghi cho `toggleDictSource`,
   * `toggleLookupPin` và `shortcuts.capture`. Hàng đang nhắm đọc từ trạng thái quanh nó, và
   * nó được nhắm bằng `@mousedown`/`@focusin` chứ không bằng `@click` *(Kiểm A của
   * `check:commands` nói nguyên văn "chỉ `@click`")* — đúng khuôn bảng phím của Story 1.21.
   */
  restoreAimedVersion?: () => void
  /** Đồng ý ghi đè một bản nháp chưa ký. Handler của `history.confirm_restore` (AC2). */
  confirmPendingRestore?: () => void
  /** Giữ bản đang soạn, bỏ câu hỏi. Handler của `history.cancel_restore` (AC2). */
  cancelPendingRestore?: () => void

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

  // ── Story 3.3 — "Thêm nhanh thuật ngữ" (FR48) ───────────────────────────────────
  //
  // ⚠️ TIÊM VÀO, cùng cửa và cùng lý do với `currentSelection`/`runLookup`:
  // `glossaryQuickAddState.ts` dùng `ref`/`computed` của Vue **và** gọi `@tauri-apps/api`
  // xuyên qua `config/glossary.ts` — import thẳng nó ở đây giết Kiểm C/D/E cùng lúc.

  /**
   * 🔴 Chỗ lấy vùng chọn — **đường RIÊNG của lệnh này**, KHÔNG tái dùng `currentSelection`
   * (đó là `currentSelectionText()`, lọc vai `source` — trả rỗng ở ba trong bốn bề mặt
   * FR48). Cài đặt thật: `selectionContract.ts::currentSelectionTextForGlossaryQuickAdd`.
   */
  currentSelectionForGlossary?: () => string
  /** Mở dải "Thêm thuật ngữ" với văn bản vùng chọn thô (rỗng nếu không có gì được bôi đen).
   * Handler của `glossary.add_term`. */
  openGlossaryQuickAdd?: (sourceText: string) => void
  /**
   * Lưu (Thêm hoặc Sửa tuỳ chế độ hiện tại của dải). Handler của `glossary.save_term`.
   *
   * ⚠️ Cài đặt thật là `async`; kiểu `() => void` ở đây khớp cùng khuôn `confirmSegment` —
   * promise trả về bị bỏ qua có chủ ý, kết quả đi ra qua `ref` ở tầng module
   * (`glossaryQuickAddState.ts::quickAddSaveError`).
   */
  saveGlossaryQuickAdd?: () => void
  /** Đóng dải mà không lưu gì — trả lại focus và vùng chọn cũ. Handler của
   * `glossary.close_quick_add`. */
  closeGlossaryQuickAdd?: () => void

  // ── Story 3.5 — lớp phủ "Cài đặt ngưỡng quét Glossary" (FR47) ──────────────────
  /** Mở lớp phủ ngưỡng quét. Handler của `glossary.settings.open`. */
  openGlossarySettings?: () => void
  /** Đóng lớp phủ ngưỡng quét mà KHÔNG lưu. Handler của `glossary.settings.close`. */
  closeGlossarySettings?: () => void
  /**
   * Lưu ngưỡng đang gõ trong ô nhập. Handler của `glossary.settings.save`.
   *
   * ⚠️ Cài đặt thật là `async`; kiểu `() => void` ở đây khớp cùng khuôn
   * `saveGlossaryQuickAdd` — promise trả về bị bỏ qua có chủ ý, kết quả đi ra qua `ref` ở
   * tầng module (`glossarySettingsState.ts::glossarySettingsSaveError`).
   */
  saveGlossarySettings?: () => void

  // ── Story 3.6 — dải "Chờ chốt lần đầu gặp" (FR114) ──────────────────────────────
  /**
   * Vào dải bằng hợp âm — lưu tiêu điểm/vùng chọn cũ rồi focus ô nhập. Không làm gì nếu 0
   * mục đang chờ hỏi. Handler của `glossary.confirm.focus`.
   *
   * Tham số là vùng chọn hiện có (cột bản dịch, hoặc bất kỳ bề mặt đã đăng ký nào) — đọc
   * qua `currentSelectionForGlossary` NGAY TRONG handler này, cùng khuôn `openGlossaryQuickAdd`.
   */
  focusGlossaryConfirmStrip?: (initialTranslation: string) => void
  /**
   * Chốt bản dịch cho mục đang hỏi. Handler của `glossary.confirm.save`.
   *
   * ⚠️ Cài đặt thật là `async` và nhận `chapterId`/`segments`/`sourceLang` — kiểu `() => void`
   * ở đây khớp cùng khuôn `saveGlossaryQuickAdd`: promise trả về bị bỏ qua có chủ ý, tham số
   * ngữ cảnh (Chương đang mở) do CÀI ĐẶT tự đọc từ `editorPanelState.ts`/`sourcePanelState.ts`
   * tại thời điểm chạy — dải không tự biết Chương nào đang mở.
   */
  saveGlossaryConfirmStrip?: () => void
  /** Để sau — `source_term` hiện tại không hỏi lại trong Chương đang mở, trả lại tiêu điểm.
   * Handler của `glossary.confirm.defer`. */
  deferGlossaryConfirmStrip?: () => void

  // ── Story 3.8 — lớp phủ "Duyệt hàng loạt một phím" (FR53/FR55) ─────────────────
  /**
   * Mở lớp phủ, tải bảng chờ của Tác phẩm đang mở. Handler của `glossary.queue.open`.
   *
   * ⚠️ Cài đặt thật là `async`; kiểu `() => void` ở đây khớp cùng khuôn
   * `saveGlossaryQuickAdd` — promise trả về bị bỏ qua có chủ ý, kết quả đi ra qua các `ref`
   * ở tầng module (`glossaryQueueState.ts`).
   */
  openGlossaryQueue?: () => void
  /** Đóng lớp phủ — KHÔNG dọn state (mở lại luôn tải lại từ đầu). Handler của
   * `glossary.queue.close`. */
  closeGlossaryQueue?: () => void
  /** Nhận ứng viên đang chọn. Handler của `glossary.queue.accept` — cùng khuôn `async`
   * bị bỏ qua kết quả như `openGlossaryQueue`. */
  acceptGlossaryQueueCandidate?: () => void
  /** Bỏ ứng viên đang chọn. Handler của `glossary.queue.reject`. */
  rejectGlossaryQueueCandidate?: () => void
  /** Chuyển con trỏ xuống hàng kế tiếp. Handler của `glossary.queue.next`. */
  nextGlossaryQueueCandidate?: () => void
  /** Chuyển con trỏ lên hàng trước. Handler của `glossary.queue.prev`. */
  prevGlossaryQueueCandidate?: () => void

  // ── Story 3.9 — lớp phủ "Quản lý Glossary" (FR49) ───────────────────────────────
  /**
   * Mở lớp phủ, tải cả hai tầng của Glossary. Handler của `glossary.manage.open`.
   *
   * ⚠️ Cài đặt thật là `async`; kiểu `() => void` ở đây khớp cùng khuôn `openGlossaryQueue`
   * — promise trả về bị bỏ qua có chủ ý, kết quả đi ra qua các `ref` ở tầng module
   * (`glossaryManageState.ts`).
   */
  openGlossaryManage?: () => void
  /** Đóng lớp phủ — KHÔNG dọn state (mở lại luôn tải lại từ đầu). Handler của
   * `glossary.manage.close`. */
  closeGlossaryManage?: () => void
  /** Mở form Sửa cho hàng đang chọn. Handler của `glossary.manage.edit`. */
  beginGlossaryManageEdit?: () => void
  /** Lưu form Sửa đang mở. Handler của `glossary.manage.save` — cùng khuôn `async` bị bỏ
   * qua kết quả như `openGlossaryManage`. */
  saveGlossaryManageEdit?: () => void
  /** Đóng form Sửa mà KHÔNG lưu. Handler của `glossary.manage.cancel`. */
  cancelGlossaryManageEdit?: () => void
  /** Xoá hàng đang chọn — kể cả một mục ĐÃ CHỐT. Handler của `glossary.manage.delete`. */
  deleteGlossaryManageEntry?: () => void
  /** Đẩy hàng đang chọn (tầng Tác phẩm) lên tầng Toàn cục. Handler của
   * `glossary.manage.promote`. */
  promoteGlossaryManageEntry?: () => void
  /** Chuyển con trỏ xuống hàng kế tiếp (danh sách đã lọc). Handler của
   * `glossary.manage.next`. */
  nextGlossaryManageRow?: () => void
  /** Chuyển con trỏ lên hàng trước (danh sách đã lọc). Handler của `glossary.manage.prev`. */
  prevGlossaryManageRow?: () => void

  // ── Story 3.10b — hộp thoại chọn tệp nối vào xuất/nhập Glossary (AD-48) ─────────
  /** Mở hộp thoại LƯU rồi xuất tầng đang chọn. Handler của `glossary.manage.export_csv` —
   * cùng khuôn `async` bị bỏ qua kết quả như `openGlossaryManage`. */
  exportGlossaryManageTier?: () => void
  /** Mở hộp thoại CHỌN rồi đọc/xem-trước một lượt nhập ở tầng đang chọn. Handler của
   * `glossary.manage.import_csv`. */
  openGlossaryImportPreview?: () => void
  /** Xác nhận lượt nhập đang xem trước. Handler của `glossary.import.confirm`. */
  confirmGlossaryImportPreview?: () => void
  /** Huỷ lượt nhập đang xem trước. Handler của `glossary.import.cancel`. */
  cancelGlossaryImportPreview?: () => void
}

/**
 * **Ba** panel của Workspace, theo thứ tự khai báo. ⚠️ Chép từ `src/layout/workspaceLayout.ts`.
 *
 * 🔵 2026-08-14 (Story 2.5b): bốn → **ba** ⇒ **ba** command `layout.toggle_*`, không bốn.
 * Đây là chỗ duy nhất con số đó sống trong `src/commands/`; sàn `COMMAND_FLOOR` của
 * `check-commands.mjs` phải được đếm lại cùng lượt.
 */
const PANEL_SUFFIXES: readonly string[] = ['grid', 'lookup', 'ai_translation']

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
   * ⚠️ Có `prev` chứ không chỉ `next`: một vòng ba panel đi được một chiều thì lùi một
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
   * ═══════════════════════════════════════════════════════════════════════════════
   * 🔴 STORY 5.3 — "QUÉT LẠI THƯ MỤC" (FR99)
   * ═══════════════════════════════════════════════════════════════════════════════
   *
   * `library.rescan` là điểm vào có phím mặc định — `Mod+Alt+K` (họ `Mod+Alt+…` đã dùng cho
   * `glossary.manage.open`/`…add_term`/`…settings.open`/`…confirm.focus`/`…queue.open`).
   * ⚠️ `Mod+Alt+R` đã thuộc `editor.restore_segment` (Story 2.5c) — đo 2026-08-27:
   * `grep -oE "Mod\+Alt\+[A-Za-z0-9]+"` trên tệp này cho `Mod+Alt+K` = 0, còn trống; `R` đã
   * chiếm (`installCommands()` ném khi đăng ký nếu dùng lại nó, thấy bằng chạy thật
   * `npm run check:commands`).
   *
   * Bốn lệnh còn lại giữ 0 hợp âm mặc định, cùng chủ ý `glossary.manage.next`/`…prev`/
   * `…edit`/…: nút bấm và mũi tên xử lý bằng `dispatch('<id>')` từ `.vue`, không gọi thẳng
   * (§Always của spec: "phím tắt và Auto-Lookup phát cùng một dispatch, không gọi thẳng
   * hàm" — cùng luật áp cho mọi lối vào của một thao tác).
   */
  target.register({
    id: 'library.rescan',
    labelKey: 'command.library.rescan',
    keys: ['Mod+Alt+K'],
    run: () => {
      if (deps.rescanLibraryFolder === undefined) return portMissing('library.rescan', 'rescanLibraryFolder')
      deps.rescanLibraryFolder()
    },
  })
  target.register({
    id: 'library.choose_root',
    labelKey: 'command.library.choose_root',
    keys: undefined,
    run: () => {
      if (deps.chooseLibraryRootFolder === undefined) {
        return portMissing('library.choose_root', 'chooseLibraryRootFolder')
      }
      deps.chooseLibraryRootFolder()
    },
  })
  target.register({
    id: 'library.forget_orphan',
    labelKey: 'command.library.forget_orphan',
    keys: undefined,
    run: () => {
      if (deps.forgetCurrentLibraryOrphan === undefined) {
        return portMissing('library.forget_orphan', 'forgetCurrentLibraryOrphan')
      }
      deps.forgetCurrentLibraryOrphan()
    },
  })
  target.register({
    id: 'library.orphan_next',
    labelKey: 'command.library.orphan_next',
    keys: undefined,
    run: () => {
      if (deps.nextLibraryOrphan === undefined) return portMissing('library.orphan_next', 'nextLibraryOrphan')
      deps.nextLibraryOrphan()
    },
  })
  target.register({
    id: 'library.orphan_prev',
    labelKey: 'command.library.orphan_prev',
    keys: undefined,
    run: () => {
      if (deps.prevLibraryOrphan === undefined) return portMissing('library.orphan_prev', 'prevLibraryOrphan')
      deps.prevLibraryOrphan()
    },
  })

  /**
   * ═══════════════════════════════════════════════════════════════════════════════
   * 🔴 STORY 5.4 — "BỐN TRẠNG THÁI VÒNG ĐỜI" (FR5/FR6)
   * ═══════════════════════════════════════════════════════════════════════════════
   *
   * `library.list_works` là điểm vào có phím mặc định — `Mod+Alt+W` (mnemonic "Works").
   * ⚠️ Đo 2026-08-27: `grep -oE "Mod\+Alt\+[A-Za-z0-9]+"` trên tệp này cho `Mod+Alt+W` = 0,
   * còn trống.
   *
   * Tám lệnh còn lại giữ 0 hợp âm mặc định, cùng chủ ý các nút lọc/vòng đời khác của kho:
   * bấm/gõ qua `dispatch('<id>')` từ `.vue`, không gọi thẳng.
   */
  target.register({
    id: 'library.list_works',
    labelKey: 'command.library.list_works',
    keys: ['Mod+Alt+W'],
    run: () => {
      if (deps.loadLibraryWorks === undefined) return portMissing('library.list_works', 'loadLibraryWorks')
      deps.loadLibraryWorks()
    },
  })
  target.register({
    id: 'library.filter_not_started',
    labelKey: 'command.library.filter_not_started',
    keys: undefined,
    run: () => {
      if (deps.toggleLibraryFilterNotStarted === undefined) {
        return portMissing('library.filter_not_started', 'toggleLibraryFilterNotStarted')
      }
      deps.toggleLibraryFilterNotStarted()
    },
  })
  target.register({
    id: 'library.filter_in_progress',
    labelKey: 'command.library.filter_in_progress',
    keys: undefined,
    run: () => {
      if (deps.toggleLibraryFilterInProgress === undefined) {
        return portMissing('library.filter_in_progress', 'toggleLibraryFilterInProgress')
      }
      deps.toggleLibraryFilterInProgress()
    },
  })
  target.register({
    id: 'library.filter_paused',
    labelKey: 'command.library.filter_paused',
    keys: undefined,
    run: () => {
      if (deps.toggleLibraryFilterPaused === undefined) {
        return portMissing('library.filter_paused', 'toggleLibraryFilterPaused')
      }
      deps.toggleLibraryFilterPaused()
    },
  })
  target.register({
    id: 'library.filter_done',
    labelKey: 'command.library.filter_done',
    keys: undefined,
    run: () => {
      if (deps.toggleLibraryFilterDone === undefined) {
        return portMissing('library.filter_done', 'toggleLibraryFilterDone')
      }
      deps.toggleLibraryFilterDone()
    },
  })
  target.register({
    id: 'library.filter_clear',
    labelKey: 'command.library.filter_clear',
    keys: undefined,
    run: () => {
      if (deps.clearLibraryFilter === undefined) return portMissing('library.filter_clear', 'clearLibraryFilter')
      deps.clearLibraryFilter()
    },
  })
  target.register({
    id: 'lifecycle.set_work_override_paused',
    labelKey: 'command.lifecycle.set_work_override_paused',
    keys: undefined,
    run: () => {
      if (deps.setOpenWorkOverridePaused === undefined) {
        return portMissing('lifecycle.set_work_override_paused', 'setOpenWorkOverridePaused')
      }
      deps.setOpenWorkOverridePaused()
    },
  })
  target.register({
    id: 'lifecycle.clear_work_override',
    labelKey: 'command.lifecycle.clear_work_override',
    keys: undefined,
    run: () => {
      if (deps.clearOpenWorkOverride === undefined) {
        return portMissing('lifecycle.clear_work_override', 'clearOpenWorkOverride')
      }
      deps.clearOpenWorkOverride()
    },
  })
  target.register({
    id: 'lifecycle.set_chapter_done',
    labelKey: 'command.lifecycle.set_chapter_done',
    keys: undefined,
    run: () => {
      if (deps.setOpenChapterDone === undefined) return portMissing('lifecycle.set_chapter_done', 'setOpenChapterDone')
      deps.setOpenChapterDone()
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
  /**
   * ═══════════════════════════════════════════════════════════════════════════════
   * 🔴 STORY 2.6 — HAI COMMAND CHO LỊCH SỬ PHIÊN BẢN (FR101, AC1)
   * ═══════════════════════════════════════════════════════════════════════════════
   *
   * 🔴 **HAI lệnh, KHÔNG một lệnh bập bênh** — Quyết định #3 của Story 2.5c đã bác hình dạng
   * bập bênh bằng chữ: *"nhãn của một phím bập bênh không nói được nó sắp làm gì"*. Bảng phím
   * tắt của Story 1.21 hiện **đúng một nhãn cho mỗi hàng**, nên một lệnh `history.toggle` sẽ
   * hiện một nhãn nói dối ở một nửa số lần người dùng nhìn nó. Khuôn đã chạy hai lượt:
   * `attribution.open`/`close` và `shortcuts.open`/`close`.
   *
   * 🔴 **Hợp âm PHẢI mang `Mod`.** `keys.ts` nuốt một hợp âm thiếu phím bổ trợ chính khi tiêu
   * điểm đang ở trong vùng gõ (`lacksPrimaryMod && isTypingZone`), và con trỏ người dùng
   * **đang nằm trong ô bản dịch** đúng lúc họ muốn mở lịch sử. `editor.next_untranslated` và
   * bốn lệnh của 2.5c/2.5d đều đã phải ghi ra giới hạn này.
   *
   * **`Mod+H`, đo lại 2026-08-16 chứ không chép:** `grep KeyH src/commands/index.ts` cho **0**
   * kết quả ⇒ trống. Mockup vẽ `⌘H`, và `conflictFor` chạy trên **toàn registry** *(không
   * theo chế độ)* nên một lượt trùng sẽ lộ ra ngay ở `register()` chứ không âm thầm.
   *
   * ⚠️ `history.close` giữ **0 hợp âm mặc định** — cùng chủ ý với `shortcuts.close` và
   * `attribution.close`: `Esc` đóng lớp phủ bằng một handler **cục bộ**, cố ý **không** đăng
   * ký. Đăng ký `Escape` toàn cục biến nó thành một phím **gán lại được trên toàn ứng dụng**,
   * và một người dùng gán nó đi chỗ khác sẽ không đóng được lớp phủ nào nữa.
   */
  for (const [id, port, chord] of [
    ['history.open', 'openSegmentHistory', 'Mod+H'],
    ['history.close', 'closeSegmentHistory', undefined],
    // ⚠️ Ba command dưới giữ **0 hợp âm mặc định**, cùng chủ ý với bốn command dưới
    // `shortcuts.open`: họ `Mod+Alt+…` đã kín chỗ có nghĩa, cả ba tới được bằng Tab +
    // Enter/Space bên trong lớp phủ, VÀ chúng là nhiên liệu cho `unbound()`.
    ['history.restore', 'restoreAimedVersion', undefined],
    ['history.confirm_restore', 'confirmPendingRestore', undefined],
    ['history.cancel_restore', 'cancelPendingRestore', undefined],
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

  /**
   * ═══════════════════════════════════════════════════════════════════════════════
   * 🔴 STORY 2.5b — `editor.next_untranslated` (AC12 · AC13)
   * ═══════════════════════════════════════════════════════════════════════════════
   *
   * **Command thứ hai mang tiền tố miền `editor.`** — cùng văn phạm khoá chấm với khoá i18n,
   * đúng thứ Quyết định #4 của Story 2.5 đã chốt cho cả 2.8/2.9/2.10.
   *
   * ─────────────────────────────────────────────────────────────────────────────
   * 🔵 HỢP ÂM ĐÃ ĐỔI 2026-08-18 — `⌥↓` ⇒ `⌘⌥↓`. Story 2.10, Quyết định #1 đường (c), Ice ký
   * ─────────────────────────────────────────────────────────────────────────────
   * **Mệnh đề cũ ở đây đã hết đúng, và nó hết đúng vì một phép đo, không vì một lượt đổi gu.**
   *
   * Bản 2.5b viết rằng việc `Alt` không phải phím bổ trợ chính — nên hợp âm **bị nuốt trong
   * vùng gõ** (`keys.ts:415` + `:510`) — là *"đúng thứ ta cần"*, vì `⌥↓` trong một ô văn bản
   * trên macOS là *"xuống cuối đoạn"* của hệ điều hành. Khối đó cũng **giao món nợ đích danh
   * cho Story 2.10**. Đây là lượt trả.
   *
   * 🔴 **Vế "đúng thứ ta cần" là vế hết đúng, và bàn đo cho nó một con số** *(2026-08-18,
   * WKWebView 605.1.15 — `2-10-ban-do/README.md` §Task 1.3 vòng 2)*. Biến duy nhất khác nhau
   * giữa hai ca là `event.target`:
   *
   * | ca | target | `defaultPrevented` | vạch lề |
   * |---|---|---|---|
   * | `⌥↓` | ô đang gõ | **`false`** | 0 → 0, **không dời** |
   * | `⌥↓` *(đối chứng dương)* | `body` | **`true`** | **0 → 1** |
   *
   * ⇒ Ca **thường nhất** của FR25 — *vừa gõ xong một câu, muốn nhảy tới câu chưa dịch kế tiếp*
   * — là ca caret **đang ở trong vùng gõ**, và ở đó phím **không bắn**. AC12 vẫn "đạt" theo
   * chữ, nhưng tính năng **chết trong tay người dùng**. Một giới hạn có chủ khác một tính năng
   * không dùng được, và 2.5b không có cách nào biết vế nào là vế thật cho tới khi đo.
   *
   * 🔴 **`Mod+Alt+ArrowDown` đi qua `keys.ts:510` sạch, và điều đó cũng ĐO ĐƯỢC** — cùng bàn đo,
   * trong **cùng một ô đang gõ**: `Mod+Enter` cho `defaultPrevented: true` *(đi qua)*,
   * `Alt+ArrowDown` cho `false` *(bị nuốt)*, và `F9` chưa đăng ký cho `false` *(đối chứng ÂM —
   * nó chứng minh `true` ở hàng đầu là tín hiệu thật, không phải giá trị mặc định)*.
   *
   * ⚠️ **Vì sao KHÔNG đi đường "cửa thứ hai ở `onEditKeydown`"** *(khuôn `Backspace`/`Escape`
   * của 2.9, và nó rẻ hơn)*: đường ấy **giữ** `⌥↓` nên nó **cướp** *"xuống cuối đoạn"* của
   * macOS, và vế *"`preventDefault()` có chặn nổi một phím THẬT không"* là mệnh đề **không
   * đường nghiệm thu nào của dự án đóng được** — mọi sự kiện driver mang `isTrusted: false`, và
   * một sự kiện không tin cậy **không có default action**, nên phép kiểm sẽ trả *"chặn được"*
   * trên mọi engine kể cả engine không cho chặn. Ký đường đó là ký một mệnh đề chưa ai kiểm.
   *
   * 🔵 **Rủi ro *"mồ côi phím người dùng đã gán"* KHÔNG xảy ra, và đó là một mệnh đề về KIỂU:**
   * `ChordOverrides` là `Record<CommandId, readonly string[]>` (`keys.ts:457`) — khoá theo
   * **id**. Lượt này giữ **nguyên id** `editor.next_untranslated`, chỉ đổi `keys`, và
   * `createKeymap` ưu tiên `overrides` qua `hasOwnProperty` (`:474-477`). ⇒ Ai đã tự gán phím
   * thì **giữ nguyên phím của họ**; chỉ **mặc định** đổi.
   *
   * ⚠️ Khớp bằng `event.code` (`ArrowDown`) nên việc `⌥↓` sinh một ký tự lạ trên macOS không
   * thành vấn đề — xem `keys.ts` §"KHỚP BẰNG `event.code`". Bàn đo xác nhận driver giao đúng
   * `code: "ArrowDown"`, khác ca `⌘/` của 2.8 nơi nó giao `code: "/"` thay vì `"Slash"`.
   */
  target.register({
    id: 'editor.next_untranslated',
    labelKey: 'command.editor.next_untranslated',
    keys: ['Mod+Alt+ArrowDown'],
    run: () => {
      if (deps.goToNextUntranslated === undefined) {
        return portMissing('editor.next_untranslated', 'goToNextUntranslated')
      }
      // 🔵 STORY 2.10 · AC6 — `console.info` ĐÃ GỠ, và đó là chỗ AC6 đang hỏng trước lượt này.
      //
      // Bản cũ ghi `console.info('[grid] khong con cau chua dich nao…')` khi không dời được.
      // Theo định nghĩa của dự án, `console` **là** im lặng: người dùng bấm phím và **không một
      // pixel nào đổi**. AC6 đòi *"báo rõ điều đó thay vì im lặng không làm gì"* ⇒ mệnh đề ấy
      // **không đạt** cho tới hôm nay.
      //
      // 🔴 Câu báo nay đi vào `editorNavNotice` và ra `StatusBar.vue` — cùng hạ tầng, cùng bất
      // biến *"thao tác vừa xảy ra sở hữu thanh trạng thái"* với hai ô nhớ đã có. Vế đó nằm
      // trọn trong `editorPanelState.ts`, nên chỗ này chỉ gọi **một** hàm và không kiểm gì.
      deps.goToNextUntranslated()
    },
  })

  /**
   * ═══════════════════════════════════════════════════════════════════════════════
   * 🔴 STORY 2.10 — `editor.next_segment` · `editor.prev_segment` (FR25 · AC1 · AC2 · AC9)
   * ═══════════════════════════════════════════════════════════════════════════════
   *
   * ─────────────────────────────────────────────────────────────────────────────
   * 🔴 KHÔNG PHÍM MẶC ĐỊNH — Quyết định #2 đường (c), Ice ký 2026-08-18
   * ─────────────────────────────────────────────────────────────────────────────
   * `keys` **vắng mặt**, có chủ ý. AC9 đòi *"đều là command đăng ký, gán phím được"* — nó
   * **không** đòi một phím mặc định cụ thể, và đăng ký ở đây là đủ cả hai vế: lệnh chạy được
   * qua `dispatch`, và nó hiện trong bảng phím của Story 1.21 để người dùng tự gán.
   *
   * ⚠️ **Phân biệt ba trạng thái của `keys`, và nó có nghĩa** (`keys.ts:444-455`): `keys`
   * **vắng mặt** ở đây nghĩa *"spec không đề xuất phím nào"*, khác hẳn một `overrides[id] = []`
   * của người dùng — cái sau nghĩa *"thao tác này CỐ Ý không có phím"*. Hai câu khác nhau, và
   * `hasOwnProperty` là chỗ chúng được phân biệt.
   *
   * ⚠️ Đường bị loại và lý do: **(a)** `⌥↑`/`⌥↓`-đối xứng thừa hưởng nguyên vấn đề của Quyết
   * định #1 *(và `⌥↓` đã bị chiếm — `createKeymap` **ném** khi hai command giành một hợp âm)*;
   * **(b)** một hợp âm mang `Mod` thì chạy được ở mọi chỗ, nhưng nó buộc một **hàng mới** vào
   * bảng Phím của `EXPERIENCE.md` — tài liệu thuộc quyền Ice — cho một hợp âm bốn ngón chưa ai
   * xin.
   *
   * 🔴 **Hai lệnh riêng, không một lệnh nhận tham số hướng.** Cùng lý lẽ mà Quyết định #3 của
   * Story 2.5c đã chốt cho `omit`/`restore`: một id **là** thứ người dùng gán phím vào và thấy
   * trong bảng phím tắt, nên *"segment kế tiếp"* và *"segment trước đó"* phải là hai dòng đọc
   * được ở đó. Hai `run` này khác nhau đúng một lời gọi — nhưng chúng khác nhau **ở tầng người
   * dùng**, và đó là tầng quyết định hình dạng registry.
   */
  target.register({
    id: 'editor.next_segment',
    labelKey: 'command.editor.next_segment',
    run: () => {
      if (deps.goToNextSegment === undefined) {
        return portMissing('editor.next_segment', 'goToNextSegment')
      }
      // AC7 — biên Chương báo bằng `editorNavNotice`, xem nhánh `editor.next_untranslated`.
      deps.goToNextSegment()
    },
  })

  target.register({
    id: 'editor.prev_segment',
    labelKey: 'command.editor.prev_segment',
    run: () => {
      if (deps.goToPrevSegment === undefined) {
        return portMissing('editor.prev_segment', 'goToPrevSegment')
      }
      deps.goToPrevSegment()
    },
  })

  /**
   * ═══════════════════════════════════════════════════════════════════════════════
   * 🔴 STORY 2.11 — `editor.next_chapter` · `editor.prev_chapter` (FR26, AC1 · AC2 · AC6)
   * ═══════════════════════════════════════════════════════════════════════════════
   *
   * **HAI id**, cùng khuôn `editor.next_segment`/`prev_segment` ngay trên: *"một id **là** thứ
   * người dùng gán phím vào và thấy trong bảng phím tắt"* (Quyết định #3 của Story 2.5c).
   *
   * ─────────────────────────────────────────────────────────────────────────────
   * 🔴 VÌ SAO `Mod+Alt+]` / `Mod+Alt+[` — và vì sao KHÔNG `⌥←`/`⌥→` trần
   * ─────────────────────────────────────────────────────────────────────────────
   * Ice ký Quyết định #6, hai nửa, ngày 2026-08-18. Ba đường bị loại, mỗi đường một lý do
   * **đo được**:
   *
   * ① **`⌥←`/`⌥→` trần — CHẾT ở đúng ca thường nhất của FR26.** `keys.ts:510` bỏ mọi hợp âm
   *    `lacksPrimaryMod` khi `isTypingZone(event.target)`, và ca thường nhất của *"sang Chương
   *    sau"* là **người dùng vừa gõ xong câu cuối** ⇒ caret **đang** trong một ô bản dịch
   *    (`isContentEditable`) ⇒ phím không bắn. Đây **không** một suy luận: Story 2.10 đã đo
   *    thật trên WKWebView 605.1.15 và **lật một chữ ký** vì đúng chuyện này (`⌥↓` → `⌘⌥↓`,
   *    số đo ở `:1092-1112`).
   *
   * ② 🔴 **Và "chỗ đã đặt trước" cho `⌥←`/`⌥→` DỰA TRÊN MỘT LƯỢT ĐỌC NHẦM — đo lại 2026-08-18.**
   *    `deferred-work.md:151` (Story 1.14) viết *"không đụng `⌥←` `⌥→` trần (Chương trước/sau
   *    — `EXPERIENCE.md:148`, Story 2.11)"*. Dòng 148 của tệp đó **nay là đoạn Auto-Lookup**;
   *    hàng thật `| ⌥← ⌥→ | Chương trước / sau trong cùng lần nhập |` nằm ở **`:184`**, và nó
   *    thuộc bảng *"Sửa ranh giới bóc"* (`:174-186`) — tức **màn xem trước NHẬP**, UX-DR33
   *    (`epics.md:599`), không phải Workspace. Bảng Phím của **Workspace** (`:261-269`) không
   *    một hàng nào cho chuyển Chương. ⇒ `⌥←`/`⌥→` **chưa bao giờ** được đặt chỗ ở đây.
   *
   * ③ **Không phím mặc định** (khuôn `editor.next_segment`, chữ ký #2(c) của 2.10) — loại, vì
   *    story này **không** có bề mặt bấm được nào, nên FR26 *("mạch làm việc không bị cắt")*
   *    sẽ chỉ với tới được **sau khi** người dùng tự gán phím ở màn hình phím tắt.
   *
   * ⇒ Cặp ngoặc mang `Mod`, nên chúng đi qua `keys.ts:510` **sạch** — cùng lý lẽ đã ghi cho
   * `editor.omit_segment`/`restore_segment` ở `:1208-1214`. Đo trên bộ 49 command trước lượt
   * này: `Mod+Alt` đang dùng `1` `2` `←` `→` `↓` `O` `J` `V` `L` `S` `X` `R` `P` `U` —
   * `BracketLeft`/`BracketRight` **chưa ai chiếm**, và cả hai đã có sẵn hàng trong
   * `NAMED_CODES` (`keys.ts:121-122`) lẫn bảng ký hiệu hiển thị (`:299-300`).
   *
   * ⚠️ **Họ `editor.*`, không một họ `chapter.*` mới** (nửa sau của chữ ký #6): mọi lệnh của
   * lưới sống ở đây, và người dùng tìm chúng cạnh `editor.next_segment` trong bảng phím tắt.
   * 🔴 Đổi id về sau là **mồ côi phím người dùng đã gán, im lặng** — `ScopeKind::Shortcut` lưu
   * **theo id** (`kinds.rs:200-204`). Chốt một lần, và đây là lần đó.
   */
  target.register({
    id: 'editor.next_chapter',
    labelKey: 'command.editor.next_chapter',
    keys: ['Mod+Alt+BracketRight'],
    run: () => {
      if (deps.goToNextChapter === undefined) {
        return portMissing('editor.next_chapter', 'goToNextChapter')
      }
      // AC4 — biên Chương báo bằng `editorNavNotice`; xem hợp đồng `void` ở [`CommandDeps`].
      deps.goToNextChapter()
    },
  })

  target.register({
    id: 'editor.prev_chapter',
    labelKey: 'command.editor.prev_chapter',
    keys: ['Mod+Alt+BracketLeft'],
    run: () => {
      if (deps.goToPrevChapter === undefined) {
        return portMissing('editor.prev_chapter', 'goToPrevChapter')
      }
      deps.goToPrevChapter()
    },
  })

  /**
   * ═══════════════════════════════════════════════════════════════════════════════
   * 🔴 STORY 2.5c — `editor.omit_segment` · `editor.restore_segment` (FR133, AC1 · AC4)
   * ═══════════════════════════════════════════════════════════════════════════════
   *
   * **HAI** command, không một command bập bênh — Quyết định #3 đường (b), Ice ký 2026-08-15.
   *
   * ─────────────────────────────────────────────────────────────────────────────
   * 🔴 VÌ SAO HAI CHỨ KHÔNG MỘT, và vì sao tiền lệ trong kho KHÔNG quyết hộ
   * ─────────────────────────────────────────────────────────────────────────────
   * `editor.confirm_segment` là một chiều — bỏ xác nhận xảy ra **ngầm** khi người dùng sửa
   * văn bản (`unconfirm_edited_segments`), nên nó không cần một lệnh nghịch và không phải
   * khuôn cho một cờ **đảo ngược tường minh** như AC4 đòi.
   *
   * Cái giá của một lệnh bập bênh nằm ở **bảng phím tắt** (Story 1.21): mỗi hàng hiện một
   * nhãn, và một nhãn *"Cắt bỏ / bỏ cắt bỏ"* không nói được phím sắp làm gì — nó phụ thuộc
   * một trạng thái mà bảng phím tắt không nhìn thấy. Hai lệnh cho hai nhãn đọc thẳng ra
   * hành động, và người dùng gán lại được **riêng từng cái**.
   *
   * ─────────────────────────────────────────────────────────────────────────────
   * ⚠️ VÌ SAO `Mod+Alt+…` CHỨ KHÔNG MỘT HỢP ÂM KHÔNG CÓ `Mod`
   * ─────────────────────────────────────────────────────────────────────────────
   * Thao tác này có nghĩa **ở đúng chỗ người dùng đang gõ** — con trỏ nằm trong ô bản dịch.
   * Một hợp âm thiếu phím bổ trợ chính bị nuốt trong vùng gõ (`keys.ts:287`
   * `lacksPrimaryMod && isTypingZone`), tức phím sẽ **không bao giờ bắn** ở đúng chỗ nó cần
   * bắn — cùng giới hạn mà `editor.next_untranslated` đã phải ghi ra ở trên. `Mod+Alt+X` và
   * `Mod+Alt+R` mang `Mod`, nên chúng đi qua.
   *
   * ⚠️ Hai hợp âm này **chưa ai chiếm**: `Mod+Alt` hôm nay dùng `1` `2` `←` `→` `O` `J` `V`
   * `L` `S`. Phép kiểm trùng hợp âm (`conflictFor`) chạy trên **toàn registry**, không theo
   * chế độ, nên một lượt trùng sẽ lộ ra ngay ở `register()`.
   */
  for (const [id, omitted, chord] of [
    ['editor.omit_segment', true, 'Mod+Alt+X'],
    ['editor.restore_segment', false, 'Mod+Alt+R'],
  ] as const) {
    target.register({
      id,
      labelKey: `command.${id}`,
      keys: [chord],
      run: () => {
        if (deps.setSegmentOmitted === undefined) {
          return portMissing(id, 'setSegmentOmitted')
        }
        deps.setSegmentOmitted(omitted)
      },
    })
  }

  // ── Story 2.5d — CỜ KẾT ĐOẠN CỦA BẢN DỊCH (FR134 · AD-46) ────────────────────
  //
  // **HAI** command, không một command bập bênh — Quyết định #3 đường (c), Ice ký 2026-08-15.
  //
  // 🔴 Cùng lý do nguyên văn với `editor.omit_segment`/`editor.restore_segment` ngay trên, và
  // lần này nó là một tiền lệ **đã được kiểm**: Quyết định #3 của Story 2.5c bác hình dạng
  // bập bênh vì *"nhãn của một phím bập bênh không nói được nó sắp làm gì"*, và bảng phím tắt
  // của Story 1.21 hiện đúng một nhãn cho mỗi hàng.
  //
  // ⚠️ Hợp âm mang `Mod`, cùng lý do đã ghi ở trên: một hợp âm thiếu phím bổ trợ chính bị nuốt
  // trong vùng gõ (`keys.ts` — `lacksPrimaryMod && isTypingZone`), tức nó sẽ **không bao giờ
  // bắn** ở đúng chỗ nó cần bắn — con trỏ đang nằm trong ô bản dịch.
  //
  // ⚠️ `Mod+Alt+P` và `Mod+Alt+U` **chưa ai chiếm**: `Mod+Alt` hôm nay dùng `1` `2` `←` `→`
  // `O` `J` `V` `L` `S` `X` `R`. `conflictFor` chạy trên **toàn registry** (không theo chế
  // độ), nên một lượt trùng lộ ra ngay ở `register()`.
  for (const [id, endsParagraph, chord] of [
    ['editor.end_target_paragraph', true, 'Mod+Alt+P'],
    ['editor.join_target_paragraph', false, 'Mod+Alt+U'],
  ] as const) {
    target.register({
      id,
      labelKey: `command.${id}`,
      keys: [chord],
      run: () => {
        if (deps.setSegmentParagraphEnd === undefined) {
          return portMissing(id, 'setSegmentParagraphEnd')
        }
        deps.setSegmentParagraphEnd(endsParagraph)
      },
    })
  }
  /**
   * ═══════════════════════════════════════════════════════════════════════════════
   * 🔴 STORY 2.8 — GỘP và TÁCH segment tường minh (FR78 · AD-5 · AC8)
   * ═══════════════════════════════════════════════════════════════════════════════
   *
   * **AC8 nguyên văn:** *"`⌘M` và `⌘/` là command đã đăng ký, **không phải hệ quả phụ của
   * việc gõ**"*. Hai lời gọi `register()` dưới đây **là** vế nghiệm thu của AC đó —
   * `check:commands` Kiểm A canh mệnh đề ấy trên toàn cây.
   *
   * ─────────────────────────────────────────────────────────────────────────────
   * 🔴 HAI CÁI TÊN, VÀ MỘT TRONG HAI ĐÃ CÓ CHỮ KÝ TỪ 2026-08-14
   * ─────────────────────────────────────────────────────────────────────────────
   * `mockups/settings.html:276-277` viết `editor.segment.merge` / `editor.segment.split`;
   * doc-comment ở đầu khối `editor.confirm_segment` — **Ice ký 2026-08-14, Quyết định #4 của
   * Story 2.5** — khai đích danh `editor.merge_segments` / `editor.split_segment`.
   *
   * **Ice chốt 2026-08-17: theo chữ ký cũ.** Lý do không phải thâm niên: command id nằm
   * trong **bảng keybinding của người dùng** (Story 1.21, `global.db` loại `shortcut`), nên
   * một lượt đổi tên về sau **mồ côi phím tắt người dùng đã gán, IM LẶNG** — đúng bài học
   * Quyết định #5 của Story 2.5b. ⇒ `mockups/settings.html` là tài liệu phải sửa, và nó đã
   * được sửa cùng lượt này.
   *
   * ─────────────────────────────────────────────────────────────────────────────
   * ⚠️ HỢP ÂM: BA TÀI LIỆU NÓI BA ĐIỀU, và bảng Phím là bản CŨ
   * ─────────────────────────────────────────────────────────────────────────────
   * `epics.md:2498, 2502` *(AC — nguồn chính thức)* và `EXPERIENCE.md:169` đều `⌘M`/`⌘/`;
   * chỉ bảng Phím `EXPERIENCE.md:267` viết `⌘T` cho tách. Ice chốt theo **AC**, và
   * `EXPERIENCE.md:267` đã sửa tại chỗ kèm 🔵 + ngày.
   *
   * ⚠️ **Cả hai hợp âm rảnh hôm nay** — đo 2026-08-17: `grep KeyM` trên `src/commands/**` cho
   * **0**; `Slash` chỉ có trong bảng tra `NAMED_CODES`/`KEY_GLYPHS` (`keys.ts:114, 292`),
   * chưa command nào dùng. `conflictFor` chạy trên **toàn registry**, không theo chế độ, nên
   * một lượt trùng lộ ra ngay ở `register()`.
   *
   * 🔴 **MỘT XUNG ĐỘT TƯƠNG LAI, ghi ra hôm nay thay vì để nó nổ im lặng ở Epic 7:**
   * `mockups/tm-manage.html:128` dùng `⌘M` mở màn hình **Quản lý TM**. Va chạm đó chưa xảy
   * ra *(Quản lý TM là Epic 7)*, và tài liệu **chưa từng gọi tên nó** — trong khi xung đột
   * `⌘⇧T` thì `settings.html:274-275` đã đánh dấu bằng `class="conflict"`. Món nợ có chủ
   * **Epic 7** đã vào `deferred-work.md`.
   *
   * ⚠️ Hai hợp âm mang `Mod`, nên chúng đi qua được luật vùng gõ (`keys.ts:415` —
   * `lacksPrimaryMod = !meta && !ctrl`): chúng vẫn bắn khi caret đang nằm trong ô bản dịch,
   * đúng chỗ duy nhất hai thao tác này có nghĩa.
   */
  //
  // 🔵 **STORY 2.9, AC8 — `editor.clear_source_cuts` KHÔNG mang `Mod`, và đó là một ngoại lệ
  // CÓ ĐO, không một lượt quên.** `Escape` là phím lui theo quy ước của mọi trình soạn thảo;
  // gắn nó sau một phím bổ trợ là đặt một quy ước riêng cho một thao tác quen thuộc.
  // 🔴 **Hệ quả phải ghi ra:** vì thiếu `Mod`, `keys.ts:510` **chặn** hợp âm này khi tiêu điểm
  // nằm trong vùng gõ — tức ở đúng chỗ người dùng đang đứng sau khi vừa gõ. Command này vì thế
  // **một mình không đủ**, và `GridPanel.vue::onEditKeydown` bắt `Escape` trực tiếp rồi
  // `dispatch` **chính id này**. Hai cửa, MỘT command — cùng khuôn cử chỉ `Backspace` của AC1.
  // ⇒ Đăng ký ở đây **không thừa**: nó là thứ làm phím gán lại được (FR22) và hiện trong bảng
  //   phím của Story 1.21. Gỡ nó đi là rút một tính năng, không dọn một dòng.
  for (const [id, port, chord] of [
    ['editor.merge_segments', 'mergeSegments', 'Mod+M'],
    ['editor.split_segment', 'splitSegment', 'Mod+Slash'],
    ['editor.clear_source_cuts', 'clearSourceCuts', 'Escape'],
  ] as const) {
    target.register({
      id,
      labelKey: `command.${id}`,
      keys: [chord],
      run: () => {
        const handler = deps[port]
        if (handler === undefined) return portMissing(id, port)
        handler()
      },
    })
  }

  /**
   * ═══════════════════════════════════════════════════════════════════════════════
   * 🔴 STORY 3.3 — "THÊM NHANH THUẬT NGỮ" (FR48)
   * ═══════════════════════════════════════════════════════════════════════════════
   *
   * `glossary.add_term` là điểm vào DUY NHẤT có phím mặc định — `Mod+Alt+G`, họ phím
   * `Mod+Alt+…` đã dùng cho preset bố cục/đi lại panel (Story 1.14) và tab Nguyên văn
   * (Story 1.16). Đo 2026-08-20: `grep` trên hằng số hợp âm của tệp này = 0, hợp âm còn
   * trống.
   *
   * `glossary.save_term`/`glossary.close_quick_add` giữ **0 hợp âm mặc định** — cùng chủ ý
   * với `attribution.close`/`shortcuts.close`/`history.close`: `↵`/`Esc` xử lý bằng một
   * handler CỤC BỘ trong `GlossaryQuickAdd.vue` (Kiểm A của `check:commands` nói nguyên văn
   * "chỉ `@click`" — `@keydown`/`@submit` không thuộc luật đó, nên chúng được xử lý tự do).
   * Hai command này tồn tại để nút Lưu/Huỷ có một `dispatch('<id>')` hợp lệ (AC1) VÀ để
   * màn hình phím tắt (Story 1.21) liệt kê được cả hai thao tác.
   */
  target.register({
    id: 'glossary.add_term',
    labelKey: 'command.glossary.add_term',
    keys: ['Mod+Alt+G'],
    run: () => {
      if (deps.openGlossaryQuickAdd === undefined) {
        return portMissing('glossary.add_term', 'openGlossaryQuickAdd')
      }
      if (deps.currentSelectionForGlossary === undefined) {
        return portMissing('glossary.add_term', 'currentSelectionForGlossary')
      }
      deps.openGlossaryQuickAdd(deps.currentSelectionForGlossary())
    },
  })
  target.register({
    id: 'glossary.save_term',
    labelKey: 'command.glossary.save_term',
    keys: undefined,
    run: () => {
      if (deps.saveGlossaryQuickAdd === undefined) return portMissing('glossary.save_term', 'saveGlossaryQuickAdd')
      deps.saveGlossaryQuickAdd()
    },
  })
  target.register({
    id: 'glossary.close_quick_add',
    labelKey: 'command.glossary.close_quick_add',
    keys: undefined,
    run: () => {
      if (deps.closeGlossaryQuickAdd === undefined) {
        return portMissing('glossary.close_quick_add', 'closeGlossaryQuickAdd')
      }
      deps.closeGlossaryQuickAdd()
    },
  })

  /**
   * ═══════════════════════════════════════════════════════════════════════════════
   * 🔴 STORY 3.5 — "CÀI ĐẶT NGƯỠNG QUÉT GLOSSARY" (FR47), lớp phủ thứ tư
   * ═══════════════════════════════════════════════════════════════════════════════
   *
   * `glossary.settings.open` là điểm vào có phím mặc định — `Mod+Alt+T` (họ `Mod+Alt+…`
   * đã dùng cho `glossary.add_term`/preset bố cục/đi lại panel/tab Nguyên văn). Đo
   * 2026-08-22: `grep` trên hằng số hợp âm của tệp này cho `Mod+Alt+T` = 0, còn trống.
   *
   * `glossary.settings.close`/`glossary.settings.save` giữ **0 hợp âm mặc định** — cùng
   * chủ ý với `glossary.close_quick_add`/`glossary.save_term`: `Esc`/`↵` xử lý bằng một
   * handler CỤC BỘ trong `GlossarySettingsOverlay.vue`. Hai command này tồn tại để nút
   * Lưu/Huỷ có một `dispatch('<id>')` hợp lệ (Kiểm A) VÀ để màn hình phím tắt liệt kê được
   * cả ba thao tác.
   */
  target.register({
    id: 'glossary.settings.open',
    labelKey: 'command.glossary.settings.open',
    keys: ['Mod+Alt+T'],
    run: () => {
      if (deps.openGlossarySettings === undefined) {
        return portMissing('glossary.settings.open', 'openGlossarySettings')
      }
      deps.openGlossarySettings()
    },
  })
  target.register({
    id: 'glossary.settings.close',
    labelKey: 'command.glossary.settings.close',
    keys: undefined,
    run: () => {
      if (deps.closeGlossarySettings === undefined) {
        return portMissing('glossary.settings.close', 'closeGlossarySettings')
      }
      deps.closeGlossarySettings()
    },
  })
  target.register({
    id: 'glossary.settings.save',
    labelKey: 'command.glossary.settings.save',
    keys: undefined,
    run: () => {
      if (deps.saveGlossarySettings === undefined) {
        return portMissing('glossary.settings.save', 'saveGlossarySettings')
      }
      deps.saveGlossarySettings()
    },
  })

  /**
   * ═══════════════════════════════════════════════════════════════════════════════
   * 🔴 STORY 3.6 — "TRẠNG THÁI CHỜ CHỐT VÀ DẢI MỌC CHỐT LẦN ĐẦU GẶP" (FR114)
   * ═══════════════════════════════════════════════════════════════════════════════
   *
   * `glossary.confirm.focus` là đường DUY NHẤT dải nhận tiêu điểm — nó cố ý KHÔNG cướp tiêu
   * điểm lúc mọc (§Boundaries của spec), nên toàn bộ thao tác "vào dải bằng bàn phím" đi qua
   * ĐÚNG một hợp âm mặc định — `Mod+Alt+C` (đo 2026-08-22: `grep` trên hằng số hợp âm của tệp
   * này = 0, còn trống).
   *
   * `glossary.confirm.save`/`glossary.confirm.defer` giữ **0 hợp âm mặc định** — cùng chủ ý
   * với `glossary.save_term`/`glossary.close_quick_add`: `↵`/`Esc` xử lý bằng một handler CỤC
   * BỘ trong `GlossaryConfirmStrip.vue` (Kiểm A của `check:commands` chỉ canh `@click`). Hai
   * command này tồn tại để nút Lưu/Để sau có một `dispatch('<id>')` hợp lệ VÀ để màn hình
   * phím tắt liệt kê được cả ba thao tác.
   */
  target.register({
    id: 'glossary.confirm.focus',
    labelKey: 'command.glossary.confirm.focus',
    keys: ['Mod+Alt+C'],
    run: () => {
      if (deps.focusGlossaryConfirmStrip === undefined) {
        return portMissing('glossary.confirm.focus', 'focusGlossaryConfirmStrip')
      }
      // `currentSelectionForGlossary` là ĐỌC, không bắt buộc — thiếu nó chỉ mất phần điền
      // sẵn (dải vẫn vào được, ô nhập rỗng), nên đây KHÔNG phải một `portMissing`.
      deps.focusGlossaryConfirmStrip(deps.currentSelectionForGlossary?.() ?? '')
    },
  })
  target.register({
    id: 'glossary.confirm.save',
    labelKey: 'command.glossary.confirm.save',
    keys: undefined,
    run: () => {
      if (deps.saveGlossaryConfirmStrip === undefined) {
        return portMissing('glossary.confirm.save', 'saveGlossaryConfirmStrip')
      }
      deps.saveGlossaryConfirmStrip()
    },
  })
  target.register({
    id: 'glossary.confirm.defer',
    labelKey: 'command.glossary.confirm.defer',
    keys: undefined,
    run: () => {
      if (deps.deferGlossaryConfirmStrip === undefined) {
        return portMissing('glossary.confirm.defer', 'deferGlossaryConfirmStrip')
      }
      deps.deferGlossaryConfirmStrip()
    },
  })

  /**
   * ═══════════════════════════════════════════════════════════════════════════════
   * 🔴 STORY 3.8 — "DUYỆT HÀNG LOẠT MỘT PHÍM" (FR53/FR55)
   * ═══════════════════════════════════════════════════════════════════════════════
   *
   * `glossary.queue.open` là điểm vào có phím mặc định — `Mod+Alt+Q` (họ `Mod+Alt+…` đã
   * dùng cho `glossary.add_term`/`glossary.settings.open`/`glossary.confirm.focus`/preset bố
   * cục/đi lại panel/tab Nguyên văn. Đo 2026-08-24: `grep` trên hằng số hợp âm của tệp này
   * cho `Mod+Alt+Q` = 0, còn trống.
   *
   * `glossary.queue.accept`/`glossary.queue.reject`/`glossary.queue.next`/
   * `glossary.queue.prev`/`glossary.queue.close` giữ **0 hợp âm mặc định** — đúng chủ ý
   * `glossary.save_term`/`glossary.close_quick_add`: `N`/`B`/mũi tên/`Esc` xử lý bằng một
   * handler CỤC BỘ trong `GlossaryQueueOverlay.vue`, `dispatch('<id>')` chứ không gọi thẳng
   * (§Always của spec: "một lời gọi thẳng dựng đường thứ hai mà `check:commands` Kiểm A
   * không nhìn thấy"). Năm command này tồn tại để nút bấm có một `dispatch('<id>')` hợp lệ
   * (Kiểm A) VÀ để màn hình phím tắt liệt kê được cả sáu thao tác.
   */
  target.register({
    id: 'glossary.queue.open',
    labelKey: 'command.glossary.queue.open',
    keys: ['Mod+Alt+Q'],
    run: () => {
      if (deps.openGlossaryQueue === undefined) {
        return portMissing('glossary.queue.open', 'openGlossaryQueue')
      }
      deps.openGlossaryQueue()
    },
  })
  target.register({
    id: 'glossary.queue.close',
    labelKey: 'command.glossary.queue.close',
    keys: undefined,
    run: () => {
      if (deps.closeGlossaryQueue === undefined) {
        return portMissing('glossary.queue.close', 'closeGlossaryQueue')
      }
      deps.closeGlossaryQueue()
    },
  })
  target.register({
    id: 'glossary.queue.accept',
    labelKey: 'command.glossary.queue.accept',
    keys: undefined,
    run: () => {
      if (deps.acceptGlossaryQueueCandidate === undefined) {
        return portMissing('glossary.queue.accept', 'acceptGlossaryQueueCandidate')
      }
      deps.acceptGlossaryQueueCandidate()
    },
  })
  target.register({
    id: 'glossary.queue.reject',
    labelKey: 'command.glossary.queue.reject',
    keys: undefined,
    run: () => {
      if (deps.rejectGlossaryQueueCandidate === undefined) {
        return portMissing('glossary.queue.reject', 'rejectGlossaryQueueCandidate')
      }
      deps.rejectGlossaryQueueCandidate()
    },
  })
  target.register({
    id: 'glossary.queue.next',
    labelKey: 'command.glossary.queue.next',
    keys: undefined,
    run: () => {
      if (deps.nextGlossaryQueueCandidate === undefined) {
        return portMissing('glossary.queue.next', 'nextGlossaryQueueCandidate')
      }
      deps.nextGlossaryQueueCandidate()
    },
  })
  target.register({
    id: 'glossary.queue.prev',
    labelKey: 'command.glossary.queue.prev',
    keys: undefined,
    run: () => {
      if (deps.prevGlossaryQueueCandidate === undefined) {
        return portMissing('glossary.queue.prev', 'prevGlossaryQueueCandidate')
      }
      deps.prevGlossaryQueueCandidate()
    },
  })

  /**
   * ═══════════════════════════════════════════════════════════════════════════════
   * 🔴 STORY 3.9 — "QUẢN LÝ GLOSSARY" (FR49)
   * ═══════════════════════════════════════════════════════════════════════════════
   *
   * `glossary.manage.open` là điểm vào có phím mặc định — `Mod+Alt+M` (họ `Mod+Alt+…` đã
   * dùng cho `glossary.add_term`/`glossary.settings.open`/`glossary.confirm.focus`/
   * `glossary.queue.open`/preset bố cục/đi lại panel/tab Nguyên văn. Đo 2026-08-24: `grep`
   * trên hằng số hợp âm của tệp này cho `Mod+Alt+M` = 0, còn trống.
   *
   * Tám lệnh còn lại giữ **0 hợp âm mặc định**, đúng chủ ý `glossary.queue.accept`/`…reject`/
   * `…next`/`…prev`: `Sửa`/`Lưu`/`Huỷ`/`Xoá`/`Đẩy`/mũi tên xử lý bằng một handler CỤC BỘ
   * trong `GlossaryManageOverlay.vue`, `dispatch('<id>')` chứ không gọi thẳng (§Always của
   * spec: "một lời gọi thẳng dựng đường thứ hai mà `check:commands` Kiểm A không nhìn
   * thấy"). Chín command này tồn tại để nút bấm có một `dispatch('<id>')` hợp lệ (Kiểm A) VÀ
   * để màn hình phím tắt liệt kê được cả chín thao tác.
   */
  target.register({
    id: 'glossary.manage.open',
    labelKey: 'command.glossary.manage.open',
    keys: ['Mod+Alt+M'],
    run: () => {
      if (deps.openGlossaryManage === undefined) {
        return portMissing('glossary.manage.open', 'openGlossaryManage')
      }
      deps.openGlossaryManage()
    },
  })
  target.register({
    id: 'glossary.manage.close',
    labelKey: 'command.glossary.manage.close',
    keys: undefined,
    run: () => {
      if (deps.closeGlossaryManage === undefined) {
        return portMissing('glossary.manage.close', 'closeGlossaryManage')
      }
      deps.closeGlossaryManage()
    },
  })
  target.register({
    id: 'glossary.manage.edit',
    labelKey: 'command.glossary.manage.edit',
    keys: undefined,
    run: () => {
      if (deps.beginGlossaryManageEdit === undefined) {
        return portMissing('glossary.manage.edit', 'beginGlossaryManageEdit')
      }
      deps.beginGlossaryManageEdit()
    },
  })
  target.register({
    id: 'glossary.manage.save',
    labelKey: 'command.glossary.manage.save',
    keys: undefined,
    run: () => {
      if (deps.saveGlossaryManageEdit === undefined) {
        return portMissing('glossary.manage.save', 'saveGlossaryManageEdit')
      }
      deps.saveGlossaryManageEdit()
    },
  })
  target.register({
    id: 'glossary.manage.cancel',
    labelKey: 'command.glossary.manage.cancel',
    keys: undefined,
    run: () => {
      if (deps.cancelGlossaryManageEdit === undefined) {
        return portMissing('glossary.manage.cancel', 'cancelGlossaryManageEdit')
      }
      deps.cancelGlossaryManageEdit()
    },
  })
  target.register({
    id: 'glossary.manage.delete',
    labelKey: 'command.glossary.manage.delete',
    keys: undefined,
    run: () => {
      if (deps.deleteGlossaryManageEntry === undefined) {
        return portMissing('glossary.manage.delete', 'deleteGlossaryManageEntry')
      }
      deps.deleteGlossaryManageEntry()
    },
  })
  target.register({
    id: 'glossary.manage.promote',
    labelKey: 'command.glossary.manage.promote',
    keys: undefined,
    run: () => {
      if (deps.promoteGlossaryManageEntry === undefined) {
        return portMissing('glossary.manage.promote', 'promoteGlossaryManageEntry')
      }
      deps.promoteGlossaryManageEntry()
    },
  })
  target.register({
    id: 'glossary.manage.next',
    labelKey: 'command.glossary.manage.next',
    keys: undefined,
    run: () => {
      if (deps.nextGlossaryManageRow === undefined) {
        return portMissing('glossary.manage.next', 'nextGlossaryManageRow')
      }
      deps.nextGlossaryManageRow()
    },
  })
  target.register({
    id: 'glossary.manage.prev',
    labelKey: 'command.glossary.manage.prev',
    keys: undefined,
    run: () => {
      if (deps.prevGlossaryManageRow === undefined) {
        return portMissing('glossary.manage.prev', 'prevGlossaryManageRow')
      }
      deps.prevGlossaryManageRow()
    },
  })

  /**
   * ═══════════════════════════════════════════════════════════════════════════════
   * 🔴 STORY 3.10b — HỘP THOẠI CHỌN TỆP NỐI VÀO XUẤT/NHẬP GLOSSARY (AD-48)
   * ═══════════════════════════════════════════════════════════════════════════════
   *
   * Bốn lệnh, KHÔNG hợp âm mặc định (cùng chủ ý phần lớn lệnh của `GlossaryManageOverlay.vue`
   * — mở hộp thoại là một thao tác bấm nút, không phải một thao tác gõ phím). Cả bốn mở
   * hộp thoại/ghi TRONG RUST — `dispatch()` là chỗ DUY NHẤT frontend chạm tới chúng.
   */
  target.register({
    id: 'glossary.manage.export_csv',
    labelKey: 'command.glossary.manage.export_csv',
    keys: undefined,
    run: () => {
      if (deps.exportGlossaryManageTier === undefined) {
        return portMissing('glossary.manage.export_csv', 'exportGlossaryManageTier')
      }
      deps.exportGlossaryManageTier()
    },
  })
  target.register({
    id: 'glossary.manage.import_csv',
    labelKey: 'command.glossary.manage.import_csv',
    keys: undefined,
    run: () => {
      if (deps.openGlossaryImportPreview === undefined) {
        return portMissing('glossary.manage.import_csv', 'openGlossaryImportPreview')
      }
      deps.openGlossaryImportPreview()
    },
  })
  target.register({
    id: 'glossary.import.confirm',
    labelKey: 'command.glossary.import.confirm',
    keys: undefined,
    run: () => {
      if (deps.confirmGlossaryImportPreview === undefined) {
        return portMissing('glossary.import.confirm', 'confirmGlossaryImportPreview')
      }
      deps.confirmGlossaryImportPreview()
    },
  })
  target.register({
    id: 'glossary.import.cancel',
    labelKey: 'command.glossary.import.cancel',
    keys: undefined,
    run: () => {
      if (deps.cancelGlossaryImportPreview === undefined) {
        return portMissing('glossary.import.cancel', 'cancelGlossaryImportPreview')
      }
      deps.cancelGlossaryImportPreview()
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
