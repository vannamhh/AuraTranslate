/**
 * Chỗ **DUY NHẤT** đăng ký thao tác — đúng khuôn "một chỗ chạm" của `src/i18n/index.ts`.
 * Story 1.6 · AC1 · AC2 · AC3 · AC6 · AD-34 · FR22.
 *
 * ─────────────────────────────────────────────────────────────────────────────
 * ⚠️ TỆP NÀY VẪN NẠP ĐƯỢC BẰNG NODE THUẦN — và đó là một ràng buộc, không phải may mắn
 * ─────────────────────────────────────────────────────────────────────────────
 * Nó chỉ `import` ba module thuần cùng thư mục. ⛔ Không `vue`, không `../modes/**`,
 * không `@tauri-apps/api`. Nhờ vậy Kiểm E của `scripts/check-commands.mjs` nạp được
 * **chính bộ command của sản phẩm** rồi đối chiếu `labelKey` với `vi.json` thật — thay
 * vì đối chiếu với một bản chép trong script, thứ sẽ trôi khỏi sự thật trong hai story.
 *
 * Cách giữ ràng buộc đó mà vẫn có thao tác thật: **handler phụ thuộc trạng thái được
 * TIÊM VÀO** qua `installCommands({ setMode })`. `App.vue` là chỗ nối hai đầu.
 *
 * ⛔ Đừng thêm command nghiệp vụ (tra cứu, xác nhận, dịch…) vào đây — mỗi story tự thêm
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
 * Tiền tố của vòng xoay `focus.next_panel`. Hôm nay vòng có hai panel; Story 1.14 dựng
 * lưới 2×2 với `dockview` và vòng lên bốn.
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
 */
export const FOCUS_OWNERS: readonly FocusOwner[] = [
  'mode.library',
  'mode.workspace',
  'mode.reading',
  'panel.source',
  'panel.editor',
]

const registry: Registry = createRegistry()
const focus: FocusRegistry = createFocusRegistry()

/** Kho đọc được cho Story 1.21 (màn hình gán phím) và cho Kiểm C/E của cổng. */
export const commandRegistry: Registry = registry
export const focusRegistry: FocusRegistry = focus

/**
 * ⛔ ĐÂY LÀ CỬA DUY NHẤT MÀ MỘT HANDLER CHUỘT ĐƯỢC ĐI QUA (AC1).
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
 * ⛔ Đừng gọi hàm này từ `./keys.ts` — §Trap 1: nhận biết nền tảng phải TIÊM ĐƯỢC, nếu
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
}

/**
 * Đăng ký bộ command khởi động và dựng keymap. Gọi **một lần**, từ `src/main.ts`.
 *
 * ⚠️ Gọi lần thứ hai ⇒ `register()` ném vì id trùng. Đó là hành vi ĐÚNG (AC2) chứ không
 * phải một chỗ cần nới: hai lượt cài đặt trong một tiến trình nghĩa là hai keymap cùng
 * nghe một cửa sổ, và cái sau sẽ dispatch mọi hợp âm hai lần.
 */
export function installCommands(deps: CommandDeps): Keymap {
  const { setMode } = deps

  for (const mode of MODE_IDS) {
    registry.register({
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
        // ⛔ KHÔNG gọi `enterFocus` ở đây. Phần tử của chế độ vừa chọn chưa có trong
        // DOM tại thời điểm này — Vue mới chỉ nhận được thay đổi state. Mỗi chế độ tự
        // gọi `enterFocus` trong `onActivated`, tức đúng lúc nó đã dựng xong.
      },
    })
  }

  /**
   * 🔴 CỐ Ý KHÔNG GÁN PHÍM — §Quyết định thiết kế #5, ba lý do độc lập:
   *   1. bốn panel chưa tồn tại, nên vòng xoay chưa biết gồm những gì và theo thứ tự nào
   *      (Story 1.14, `dockview`, UX-DR13);
   *   2. mọi phím ứng cử đều đang hoặc sắp có chủ — `Tab` là thứ tự tiêu điểm của trình
   *      duyệt, `⌘1..3` đã là chế độ, `⌘⇧↵` là UX-DR35, `⌘M` `⌘/` là UX-DR32;
   *   3. AC6 cần một phần tử THẬT để chứng minh: một `unbound()` luôn trả mảng rỗng là
   *      một AC chưa được chứng minh, và Story 1.21 sẽ phát hiện nó hỏng khi đã có 40
   *      command.
   *
   * ⛔ Nhưng handler thì CHẠY THẬT. Một command rỗng đăng ký cho đủ số là đúng thứ story
   * này tồn tại để chặn.
   */
  registry.register({
    id: 'focus.next_panel',
    labelKey: 'command.focus.next_panel',
    run: () => {
      focus.next(PANEL_PREFIX)
    },
  })

  keymap = createKeymap(registry, { isMac: deps.isMac ?? detectIsMac() })
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
