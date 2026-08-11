/**
 * Tầng bàn phím — TRUNG LẬP NỀN TẢNG. Story 1.6 · AC1 · AC3 · NFR14 · FR22.
 *
 * ─────────────────────────────────────────────────────────────────────────────
 * 🔴 VÌ SAO `Mod` CHỨ KHÔNG PHẢI `event.metaKey` — §Trap 1 của story
 * ─────────────────────────────────────────────────────────────────────────────
 * `⌘1` trong AC3 là ký hiệu macOS của một phím **trừu tượng**. Trên Windows phím đó
 * là `Ctrl`. Dự án không có test nào chạm tầng bàn phím, và CI hai nền tảng của Story
 * 1.3 chỉ `cargo test` + build — nên một `if (e.metaKey && e.key === '1')` **đi qua cả
 * hai nền tảng của CI** rồi hỏng ở tay người dùng Windows. Đó là vi phạm NFR14 nặng
 * nhất còn lọt được hôm nay.
 *
 * Lời giải ba phần, và cả ba đều bắt buộc:
 *   1. hợp âm viết ở dạng dữ liệu trung lập — `'Mod+1'`, `'Mod+Shift+Enter'`;
 *   2. nhận biết nền tảng đi qua **một tham số tiêm được** (`{ isMac }`), KHÔNG đọc
 *      thẳng `navigator` ở tầng module — không tiêm được thì cổng không lái được hai ca;
 *   3. Kiểm D của `scripts/check-commands.mjs` lái CẢ HAI ca và khẳng định cùng một hợp
 *      âm khớp `metaKey` ở ca một và `ctrlKey` ở ca hai.
 *
 * **KHÔNG dùng `tauri-plugin-global-shortcut`.** Ba lý do, mỗi lý do đủ để loại:
 * (1) một phụ thuộc mới phải rà GPLv3 và vào bảng Stack TRƯỚC khi thêm (NFR15) — chưa
 * ai rà; (2) nó đăng ký phím ở tầng HỆ ĐIỀU HÀNH, tức `⌘1` bị cướp khỏi mọi ứng dụng
 * khác trong khi AuraTranslate chạy nền; (3) *"Global Hotkeys"* của FR22 nghĩa là TOÀN
 * ỨNG DỤNG, không phải toàn hệ điều hành.
 *
 * Cùng luật "erasable-only" như `./registry.ts` — Kiểm D `import()` tệp này bằng Node
 * thuần. Lần `import` duy nhất ở đây là một `import type`, và nó bị xoá hoàn toàn lúc
 * bóc kiểu nên tệp này không kéo theo thứ gì lúc chạy.
 */
import type { CommandId, Registry } from './registry.ts'

/**
 * Hình dạng tối thiểu mà `handle()` cần đọc từ một sự kiện bàn phím.
 *
 * `KeyboardEvent` thật khớp cấu trúc này, và Kiểm D đẩy vào một object giả — đó là toàn
 * bộ lý do kiểu này tồn tại thay vì `KeyboardEvent`: một phép kiểm hai nền tảng không
 * dựng được `KeyboardEvent` thật trong Node.
 */
export type ChordEvent = {
  /** ⚠️ `code`, KHÔNG phải `key`. Xem `KEY_TO_CODE`. */
  code: string
  metaKey?: boolean
  ctrlKey?: boolean
  shiftKey?: boolean
  altKey?: boolean
  repeat?: boolean
  /**
   * 🔴 IME. Đây là ứng dụng dịch tiếng Việt — đường này được đi hằng ngày từ Epic 2.
   *
   * Một bộ gõ tiếng Việt lúc commit composition phát `keydown` mang `code` VẬT LÝ. Nếu
   * `handle()` không hỏi cờ này thì phím commit bị ăn như một hợp âm và `preventDefault()`
   * giết luôn lượt commit — người dùng gõ "được" và mất chữ. ⚠️ Luật vùng gõ KHÔNG cứu
   * được ca này: nó chỉ áp cho hợp âm không có phím bổ trợ chính, còn `⌘…` thì không.
   */
  isComposing?: boolean
  target?: unknown
  preventDefault?: () => void
}

export type Binding = {
  /** Hợp âm nguyên văn như đã khai: `'Mod+1'`. */
  chord: string
  /** Hợp âm đã phân giải cho nền tảng đang chạy: `'Meta+Digit1'`. */
  resolved: string
  id: CommandId
}

export type Keymap = {
  /** `true` nếu một hợp âm khớp và đã `dispatch`. `false` ⇒ KHÔNG đụng vào event. */
  handle(event: ChordEvent): boolean
  bindings(): readonly Binding[]
}

export type Platform = {
  /** Tiêm được có chủ ý — xem §Trap 1 ở đầu tệp. */
  isMac: boolean
}

/** Bốn cờ bổ trợ của một sự kiện bàn phím, ở dạng so sánh được. */
type Mods = { meta: boolean; ctrl: boolean; shift: boolean; alt: boolean }

const NO_MODS: Mods = { meta: false, ctrl: false, shift: false, alt: false }

/**
 * 🔴 KHỚP BẰNG `event.code`, KHÔNG BẰNG `event.key`.
 *
 * `event.key` là ký tự bố cục bàn phím SINH RA. Trên AZERTY, phím vật lý `1` cho
 * `event.key === '&'`; trên bố cục tiếng Việt và trên mọi bố cục không phải US, chữ và
 * số đều trôi. `event.code` là vị trí VẬT LÝ và nó ổn định — cùng lý lẽ mà mọi trình
 * soạn thảo dùng để `⌘1` nằm đúng một chỗ trên mọi máy.
 *
 * Bảng dưới chỉ chở những phím mà lộ trình đã gọi tên (`⌘1..3` chế độ · `⌘⇧↵` đưa bản
 * dịch AI sang, UX-DR35 · `⌘M` `⌘/` gộp/tách, UX-DR32 · `⌘,` tinh chỉnh và `M` `B`
 * `1 2 3` của Chế độ đọc, UX-DR46). ⚠️ Một tên phím ngoài bảng là một lần NÉM lúc dựng
 * keymap, không phải một hợp âm lặng lẽ không bao giờ khớp.
 */
const NAMED_CODES: Readonly<Record<string, string>> = {
  Enter: 'Enter',
  Escape: 'Escape',
  Tab: 'Tab',
  Space: 'Space',
  Backspace: 'Backspace',
  Delete: 'Delete',
  Home: 'Home',
  End: 'End',
  PageUp: 'PageUp',
  PageDown: 'PageDown',
  ArrowUp: 'ArrowUp',
  ArrowDown: 'ArrowDown',
  ArrowLeft: 'ArrowLeft',
  ArrowRight: 'ArrowRight',
  Comma: 'Comma',
  Period: 'Period',
  Slash: 'Slash',
  Backslash: 'Backslash',
  Minus: 'Minus',
  Equal: 'Equal',
  Semicolon: 'Semicolon',
  Quote: 'Quote',
  Backquote: 'Backquote',
  BracketLeft: 'BracketLeft',
  BracketRight: 'BracketRight',
}

/**
 * Bốn phím bổ trợ phát `keydown` của CHÍNH CHÚNG — và đó không phải một hợp âm.
 *
 * Nhấn `⌘` phát một `keydown` mang `code === 'MetaLeft'` **trước** khi người dùng kịp gõ
 * phím thứ hai. [`chordFromEvent`] phải trả `null` ở đó và **tiếp tục chờ**; chốt sớm là
 * gán cho người dùng một hợp âm họ chưa gõ xong.
 */
const MODIFIER_CODES: Readonly<Record<string, true>> = {
  MetaLeft: true,
  MetaRight: true,
  ControlLeft: true,
  ControlRight: true,
  ShiftLeft: true,
  ShiftRight: true,
  AltLeft: true,
  AltRight: true,
}

function keyToCode(name: string): string {
  if (/^[0-9]$/.test(name)) return `Digit${name}`
  if (/^[A-Za-z]$/.test(name)) return `Key${name.toUpperCase()}`
  const named = Object.prototype.hasOwnProperty.call(NAMED_CODES, name) ? NAMED_CODES[name] : undefined
  if (named === undefined) {
    throw new Error(
      `[keys] tên phím \`${name}\` không có trong bảng — thêm nó vào \`NAMED_CODES\` kèm ` +
        'một dòng nói phím đó phục vụ thao tác nào. Một hợp âm không phân giải được là một ' +
        'phím tắt chết im lặng.',
    )
  }
  return named
}

/**
 * `'Mod+Shift+Enter'` ⇒ `{ mods, code }`, đã phân giải theo nền tảng.
 *
 * `Mod` là phím bổ trợ chính của nền tảng: `⌘` trên macOS, `Ctrl` ở nơi khác. `Ctrl` và
 * `Meta` viết tường minh vẫn dùng được cho những hợp âm thật sự khác nhau giữa hai nền
 * tảng — hôm nay không có cái nào, và đó là điều tốt.
 */
function parseChord(chord: string, platform: Platform): { mods: Mods; code: string } {
  const parts = chord.split('+').map((p) => p.trim())
  if (parts.length === 0 || parts.some((p) => p === '')) {
    throw new Error(`[keys] hợp âm \`${chord}\` sai hình dạng — viết dạng \`Mod+Shift+Enter\`.`)
  }
  const mods: Mods = { ...NO_MODS }
  /**
   * ⚠️ `'Mod+Mod+1'` đặt `meta` hai lần và biên dịch ra CÙNG `resolved` với `'Mod+1'`,
   * nên lỗi gõ chỉ lộ ra nếu hợp âm đúng tình cờ cũng được đăng ký. Cùng lý lẽ với id
   * trùng ở `register()`: một hợp âm viết sai mà vẫn bind được là một phím tắt không ai
   * truy được nguồn gốc. Ném ở cửa vào.
   */
  const seen = new Set<string>()
  for (const part of parts.slice(0, -1)) {
    if (seen.has(part)) {
      throw new Error(
        `[keys] phím bổ trợ \`${part}\` lặp lại trong \`${chord}\` — mỗi phím bổ trợ viết ` +
          'đúng một lần.',
      )
    }
    seen.add(part)
    switch (part) {
      case 'Mod':
        if (platform.isMac) mods.meta = true
        else mods.ctrl = true
        break
      case 'Meta':
        mods.meta = true
        break
      case 'Ctrl':
        mods.ctrl = true
        break
      case 'Shift':
        mods.shift = true
        break
      case 'Alt':
        mods.alt = true
        break
      default:
        throw new Error(
          `[keys] phím bổ trợ \`${part}\` trong \`${chord}\` không hợp lệ — ` +
            'chỉ có `Mod`, `Meta`, `Ctrl`, `Shift`, `Alt`.',
        )
    }
  }
  return { mods, code: keyToCode(parts[parts.length - 1] as string) }
}

/**
 * Nghịch đảo của [`keyToCode`]: `'KeyD'` ⇒ `'D'`, `'Digit7'` ⇒ `'7'`, `'Comma'` ⇒ `'Comma'`.
 *
 * ⚠️ Trả `null` — **không ném** — cho một `code` ngoài bảng. Khác biệt với `keyToCode` là
 * khác biệt về **nguồn dữ liệu**, không phải một chỗ thiếu nhất quán: `keyToCode` đọc một
 * hợp âm ĐÃ LƯU *(hằng số trong mã, hoặc một hàng `global.db`)*, nên một tên phím lạ ở đó
 * là lỗi lập trình và phải ném. Hàm này đọc một **cử chỉ người dùng** — ai cũng bấm được
 * `F1` — và ném ở đó là biến một lượt bấm phím thành một sự cố.
 */
function codeToKey(code: string): string | null {
  const digit = /^Digit([0-9])$/.exec(code)
  if (digit !== null) return digit[1] as string
  const letter = /^Key([A-Z])$/.exec(code)
  if (letter !== null) return letter[1] as string
  // Bảng khai `tên ⇒ code` và cả 25 hàng đều là ánh xạ đồng nhất, nhưng phép tra ngược
  // viết theo GIÁ TRỊ chứ không giả định điều đó: một hàng tương lai kiểu
  // `Plus: 'NumpadAdd'` sẽ vẫn tra ngược đúng thay vì lặng lẽ rơi.
  for (const name of Object.keys(NAMED_CODES)) {
    if (NAMED_CODES[name] === code) return name
  }
  return null
}

/**
 * Một sự kiện bàn phím ⇒ hợp âm TRUNG LẬP nền tảng, hoặc `null`. Story 1.21 · AC2 · AC11.
 *
 * Nghịch đảo của [`parseChord`], và vòng khứ hồi `parseChord(chordFromEvent(e, p), p)` được
 * Kiểm D của `scripts/check-commands.mjs` khẳng định trên **cả hai** nền tảng (NFR14).
 *
 * 🔴 **Thứ tự phần tử là CỐ ĐỊNH: `Mod` → `Meta` → `Ctrl` → `Alt` → `Shift` → phím.**
 * `parseChord` chấp nhận mọi thứ tự, nhưng chuỗi này đi xuống `global.db` và lên màn hình,
 * và mọi phép so xung đột trên hợp âm là một phép so CHUỖI. Hai lượt gán cùng một phím cho
 * ra hai chuỗi khác nhau là hai hàng `config_value` cho một sự thật.
 *
 * ⚠️ `Mod` **nuốt** phím bổ trợ chính của nền tảng đang chạy: `⌘` trên macOS, `Ctrl` ở nơi
 * khác. Phím bổ trợ CÒN LẠI thì viết tường minh (`Ctrl` trên macOS, `Meta` ở nơi khác) —
 * cả hai đều biểu diễn được (`parseChord`), và cả hai đều có nghĩa thật.
 *
 * Trả `null` ở ba ca, và **không ném** ở ca nào:
 *  1. lượt commit của bộ gõ (`isComposing`) — đây là ứng dụng dịch tiếng Việt;
 *  2. keydown của chính một phím bổ trợ ([`MODIFIER_CODES`]);
 *  3. một `code` không biểu diễn được (`F1`, `NumpadEnter`, `IntlBackslash`, `CapsLock`…).
 */
export function chordFromEvent(event: ChordEvent, platform: Platform): string | null {
  if (event.isComposing === true) return null
  if (Object.prototype.hasOwnProperty.call(MODIFIER_CODES, event.code)) return null
  const key = codeToKey(event.code)
  if (key === null) return null

  const mods = modsOf(event)
  const parts: string[] = []
  if (platform.isMac ? mods.meta : mods.ctrl) parts.push('Mod')
  // Trên mỗi nền tảng đúng MỘT trong hai nhánh này sống được — nhánh kia đã bị `Mod`
  // nuốt ở dòng trên — nên thứ tự `Meta` trước `Ctrl` của chuẩn không bao giờ bị thử.
  if (!platform.isMac && mods.meta) parts.push('Meta')
  if (platform.isMac && mods.ctrl) parts.push('Ctrl')
  if (mods.alt) parts.push('Alt')
  if (mods.shift) parts.push('Shift')
  parts.push(key)
  return parts.join('+')
}

/**
 * Tên phím ⇒ ký hiệu HIỂN THỊ. Chỉ những phím mà một ký hiệu đọc nhanh hơn một chữ.
 *
 * ⚠️ Bảng này KHÔNG dùng để phân giải gì cả — nó chỉ chở chữ ra màn hình. Một tên vắng
 * mặt ở đây hiện nguyên văn, và đó là hành vi đúng: `Home`, `PageUp` đọc rõ hơn mọi ký
 * hiệu người ta có thể nghĩ ra cho chúng.
 */
const KEY_GLYPHS: Readonly<Record<string, string>> = {
  ArrowLeft: '←',
  ArrowRight: '→',
  ArrowUp: '↑',
  ArrowDown: '↓',
  Comma: ',',
  Period: '.',
  Slash: '/',
  Backslash: '\\',
  Minus: '-',
  Equal: '=',
  Semicolon: ';',
  Quote: "'",
  Backquote: '`',
  BracketLeft: '[',
  BracketRight: ']',
}

/** Ký hiệu chỉ đúng trên macOS — ở nơi khác `⌫` và `⇥` không phải quy ước của ai cả. */
const MAC_KEY_GLYPHS: Readonly<Record<string, string>> = {
  Enter: '↩',
  Escape: '⎋',
  Tab: '⇥',
  Backspace: '⌫',
  Delete: '⌦',
}

/**
 * Hợp âm ⇒ chuỗi cho người ĐỌC: `'Mod+Alt+ArrowRight'` ⇒ `'⌥⌘→'` hoặc `'Ctrl+Alt+→'`.
 *
 * 🔴 Hàm này ở `keys.ts` chứ không ở lớp giao diện, và lý do là một ràng buộc chứ không
 * một sở thích: nó cần [`NAMED_CODES`] và [`KEY_GLYPHS`], mà export chúng ra là mở một
 * cửa thứ hai vào bảng phím — đúng thứ `parseChord` tồn tại để làm cửa duy nhất.
 *
 * ⚠️ Thứ tự ký hiệu trên macOS là `⌃⌥⇧⌘` — quy ước của chính Apple, và nó **khác** thứ tự
 * chuẩn của chuỗi hợp âm ([`chordFromEvent`]). Hai thứ tự cho hai mục đích: một để so
 * bằng máy, một để mắt người đọc quen. Đừng gộp chúng.
 *
 * Không ném: một hợp âm sai hình dạng hiện **nguyên văn**. Hàm này chạy ở đường vẽ, và
 * một màn hình chết vì một hàng dữ liệu xấu là tệ hơn một hàng hiện xấu.
 */
export function formatChord(chord: string, platform: Platform): string {
  const parts = chord.split('+').map((p) => p.trim())
  if (parts.length === 0 || parts.some((p) => p === '')) return chord
  const key = parts[parts.length - 1] as string
  const mods = new Set(parts.slice(0, -1))

  const glyph =
    (platform.isMac ? MAC_KEY_GLYPHS[key] : undefined) ??
    KEY_GLYPHS[key] ??
    (key === 'Space' ? 'Space' : key)

  if (platform.isMac) {
    const prefix =
      (mods.has('Ctrl') ? '⌃' : '') +
      (mods.has('Alt') ? '⌥' : '') +
      (mods.has('Shift') ? '⇧' : '') +
      (mods.has('Mod') || mods.has('Meta') ? '⌘' : '')
    return prefix + glyph
  }
  const prefix = [
    mods.has('Meta') ? 'Meta' : null,
    mods.has('Mod') || mods.has('Ctrl') ? 'Ctrl' : null,
    mods.has('Alt') ? 'Alt' : null,
    mods.has('Shift') ? 'Shift' : null,
  ].filter((p): p is string => p !== null)
  return [...prefix, glyph].join('+')
}

/**
 * Hợp âm ⇒ **chuỗi đã phân giải** cho nền tảng đang chạy: `'Mod+D'` ⇒ `'Meta+KeyD'`.
 *
 * 🔴 Đây là khoá của mọi phép so *"hai thao tác có giành cùng một phím không"* — và nó
 * phải là **CHÍNH** chuỗi mà `createKeymap` dựng, không một bản chép. Story 1.21 · AC9:
 * trên macOS `'Mod+D'` và `'Meta+D'` là **hai chuỗi hợp âm khác nhau** nhưng **một phím**,
 * và một màn hình xung đột so trên chuỗi hợp âm sẽ nói *"không xung đột"* cho đúng ca đó.
 * Ngoài macOS chúng thật sự khác nhau. Cùng một hàm, hai câu trả lời đúng.
 *
 * ⚠️ **NÉM** với hợp âm sai hình dạng, cùng `parseChord` — chỗ gọi phải tự quyết định làm
 * gì với một chuỗi không phân giải được, và nuốt nó ở đây là dựng một xung đột-ma.
 */
export function resolveChord(chord: string, platform: Platform): string {
  const { mods, code } = parseChord(chord, platform)
  return resolvedOf(mods, code)
}

/**
 * Khoá phân giải từ `{ mods, code }`. **Một** công thức, hai chỗ đọc — xem [`resolveChord`].
 */
function resolvedOf(mods: Mods, code: string): string {
  return (
    [mods.meta && 'Meta', mods.ctrl && 'Ctrl', mods.alt && 'Alt', mods.shift && 'Shift']
      .filter((p): p is string => typeof p === 'string')
      .join('+') + (hasNoMods(mods) ? code : `+${code}`)
  )
}

const modsOf = (event: ChordEvent): Mods => ({
  meta: event.metaKey === true,
  ctrl: event.ctrlKey === true,
  shift: event.shiftKey === true,
  alt: event.altKey === true,
})

/**
 * So KHỚP TUYỆT ĐỐI cả bốn cờ, không phải "có đủ những cờ đã khai".
 *
 * ⚠️ Một phép so lỏng (`e.metaKey && code khớp`) làm `⌘⇧1` cũng kích hoạt `Mod+1`. Hôm
 * nay vô hại; ngày UX-DR35 đưa `⌘⇧↵` vào thì hai hợp âm chồng lên nhau và cái đăng ký
 * trước thắng — một lỗi không ai nối lại được với dòng mã này.
 */
const sameMods = (a: Mods, b: Mods): boolean =>
  a.meta === b.meta && a.ctrl === b.ctrl && a.shift === b.shift && a.alt === b.alt

const hasNoMods = (m: Mods): boolean => !m.meta && !m.ctrl && !m.shift && !m.alt

/**
 * 🔴 Hợp âm KHÔNG mang phím bổ trợ CHÍNH (`⌘` trên macOS, `Ctrl` ở nơi khác).
 *
 * ⚠️ Đây KHÔNG phải `hasNoMods`, và khác biệt đó là một lỗi đã sửa: bản đầu canh vùng
 * gõ bằng `hasNoMods`, nên `Shift+B` — khớp đúng keydown mà người dùng tạo ra khi gõ
 * chữ "B" hoa — và `Alt+M` *(Option+M gõ `µ` trên macOS)* vẫn bắn giữa câu, nuốt luôn
 * ký tự vì `preventDefault()` chạy trước. Hôm nay không hợp âm nào như vậy được đăng ký,
 * tức luật cũ đúng do TÌNH CỜ; UX-DR46 (`M`, `B`, `1 2 3` trần) và Editor của Epic 2 làm
 * nó sống.
 *
 * `⌘1` vẫn phải chuyển chế độ được khi con trỏ đang ở trong Editor — đó là điều NFR17
 * hứa, và đó là lý do phép kiểm hỏi phím bổ trợ CHÍNH chứ không hỏi "có phím bổ trợ nào
 * không".
 */
const lacksPrimaryMod = (m: Mods): boolean => !m.meta && !m.ctrl

/**
 * 🔴 LUẬT VÙNG GÕ — chốt từ hôm nay dù chưa có một ô nhập nào.
 *
 * Chế độ đọc dùng `M`, `B`, `1 2 3` TRẦN (UX-DR46) và Editor của Epic 2 là một vùng gõ
 * tự do. Không có luật này thì gõ chữ "b" trong bản dịch sẽ bật chế độ song ngữ. Rẻ
 * hôm nay, đắt ở Epic 2 — và ở Epic 2 nó sẽ được phát hiện bằng một người dùng thật
 * đang gõ dở một câu.
 *
 * ⚠️ Chỉ áp cho hợp âm KHÔNG có phím bổ trợ. `⌘1` vẫn phải chuyển chế độ được khi con
 * trỏ đang ở trong Editor — đó chính là điều NFR17 hứa.
 *
 * 🔴 ĐỌC HÌNH DẠNG, KHÔNG DÙNG `instanceof HTMLElement` — và lý do không phải là để test
 * dễ hơn. `instanceof` so sánh theo *realm*: một phần tử đến từ một `<iframe>` hay từ một
 * document khác là `HTMLElement` của realm ĐÓ, nên phép so trả `false` và luật vùng gõ
 * lặng lẽ tắt ở đúng chỗ nó cần bật nhất. (Lợi ích phụ, và nó có thật: Kiểm D lái được
 * cả hai nhánh bằng một object giả, nên luật này có lưới thay vì chỉ có lời hứa.)
 */
function isTypingZone(target: unknown): boolean {
  if (typeof target !== 'object' || target === null) return false
  const el = target as { tagName?: unknown; isContentEditable?: unknown }
  if (el.isContentEditable === true) return true
  return el.tagName === 'INPUT' || el.tagName === 'TEXTAREA' || el.tagName === 'SELECT'
}

/**
 * Lớp hợp âm ĐÈ LÊN `CommandSpec.keys`, khoá theo id thao tác. Story 1.21.
 *
 * 🔴 Phép phân biệt sống ở **sự có mặt của khoá**, không ở giá trị:
 *
 * | Khoá | Nghĩa |
 * |---|---|
 * | vắng mặt | *"không ai đè lên"* ⇒ dùng `spec.keys` |
 * | có mặt, mảng rỗng | *"thao tác này CỐ Ý không có phím"* ⇒ không hợp âm nào |
 * | có mặt, có phần tử | hợp âm của người dùng |
 *
 * ⚠️ Nên phép tra phải là `hasOwnProperty`, **không** `overrides[id] ?? spec.keys`: một
 * mảng rỗng là falsy-ish theo nghĩa nào đó với mọi bản đọc lướt, và `??` thì đúng ở đây
 * nhưng chỉ đúng do `[]` không phải `undefined` — một sự thật quá mỏng để chống đỡ ba
 * trạng thái. Cùng lý lẽ đã ghi cho `chordsFor` ở `./index.ts`.
 */
export type ChordOverrides = Readonly<Record<CommandId, readonly string[]>>

export function createKeymap(
  registry: Registry,
  platform: Platform,
  /**
   * ⚠️ **Tuỳ chọn, và vắng mặt phải giữ nguyên hành vi cũ từng dòng một.** Bảy lời gọi
   * trong `scripts/check-commands.mjs` đều hai tham số, và Kiểm D là lưới hai nền tảng
   * duy nhất của tầng này — một thay đổi ngữ nghĩa lúc vắng mặt sẽ vượt qua nó im lặng.
   */
  overrides?: ChordOverrides,
): Keymap {
  const compiled: { mods: Mods; code: string; repeatable: boolean; binding: Binding }[] = []
  /** Hợp âm ⇒ id, để bắt hai command cùng giành một phím. */
  const claimed = new Map<string, CommandId>()

  for (const spec of registry.list()) {
    const chords =
      overrides !== undefined && Object.prototype.hasOwnProperty.call(overrides, spec.id)
        ? overrides[spec.id]
        : spec.keys
    for (const chord of chords ?? []) {
      const { mods, code } = parseChord(chord, platform)
      const resolved = resolvedOf(mods, code)
      // Cùng lý lẽ với id trùng ở `register()`: hai chỗ giành một phím thì cái sau lặng
      // lẽ không bao giờ chạy, và biểu hiện là "phím tắt X không làm gì".
      const owner = claimed.get(resolved)
      if (owner !== undefined) {
        throw new Error(
          `[keys] hợp âm \`${chord}\` (${resolved}) đã thuộc về \`${owner}\`, không gán thêm cho ` +
            `\`${spec.id}\` được. Hai command giành một phím thì cái sau chết im lặng.`,
        )
      }
      claimed.set(resolved, spec.id)
      compiled.push({
        mods,
        code,
        // Đọc MỘT LẦN lúc biên dịch, không mỗi keydown: `registry.list()` dựng một mảng
        // mới ở mỗi lời gọi, và đường lặp phím chạy ~30 lần/giây khi người ta giữ `Shift+→`.
        repeatable: spec.repeatable === true,
        binding: { chord, resolved, id: spec.id },
      })
    }
  }

  const handle = (event: ChordEvent): boolean => {
    // 🔴 IME ĐỨNG TRƯỚC MỌI THỨ. Một lượt commit composition của bộ gõ tiếng Việt phát
    // `keydown` mang `code` vật lý; ăn nó như một hợp âm là ăn mất chữ người dùng vừa gõ.
    // Không đụng vào event — đây không phải hợp âm của ứng dụng.
    if (event.isComposing === true) return false
    const mods = modsOf(event)
    for (const entry of compiled) {
      if (entry.code !== event.code || !sameMods(entry.mods, mods)) continue
      if (lacksPrimaryMod(entry.mods) && isTypingZone(event.target)) return false
      // Khớp ⇒ chặn hành vi mặc định RỒI mới dispatch. Ngược thứ tự thì một handler ném
      // sẽ để `⌘1` chạy tiếp xuống webview.
      event.preventDefault?.()
      // ⚠️ Giữ phím không lặp lại THAO TÁC — nhưng hợp âm đã khớp vẫn không được rơi
      // xuống webview, nên phép kiểm này đứng SAU `preventDefault()`. Bản đầu đặt nó ở
      // đầu hàm, và hệ quả là từ keydown thứ hai trở đi một hợp âm giữ phím lại đi tiếp
      // xuống tầng dưới với hành vi mặc định nguyên vẹn.
      //
      // 🔴 Story 1.21 · `deferred-work.md:656` — **trừ khi spec khai `repeatable`.** Bốn
      // command `selection.extend_*` PHẢI lặp: giữ `Shift+→` là cách người ta bôi đen một
      // cụm từ. Xem doc-comment của [`CommandSpec.repeatable`] về vì sao mặc định là KHÔNG.
      if (event.repeat === true && !entry.repeatable) return true
      registry.dispatch(entry.binding.id)
      return true
    }
    // Không khớp ⇒ KHÔNG `preventDefault`, không `stopPropagation`, không gì cả.
    return false
  }

  /**
   * ⚠️ Đóng băng, cùng lý lẽ với `frozen()` ở `./registry.ts`: Story 1.21 dựng màn hình
   * gán phím trên bề mặt này, và một lượt "sửa tại chỗ" ở đó sẽ đổi keymap đang sống mà
   * không đi qua `createKeymap()`. Mảng mới mỗi lời gọi thôi thì chưa đủ — nó không ngăn
   * được `bindings()[0].id = 'x'`.
   */
  const bindings = (): readonly Binding[] => compiled.map((c) => Object.freeze({ ...c.binding }))

  return { handle, bindings }
}

/**
 * Điều kiện **NUỐT** một hợp âm trước khi nó tới được `Keymap.handle`.
 *
 * 🔴 Vì sao nó là một hàm **TIÊM VÀO** chứ không một lượt `import` ở đây (Story 1.19, bắt ở
 * code review 2026-08-10): tệp này được `scripts/check-commands.mjs` nạp bằng **Node thuần**
 * ở Kiểm C/D/E, nên một `import` tới `panels/dictSourcesState.ts` *(dùng `ref` của Vue và
 * `@tauri-apps/api`)* làm cổng gãy ngay. Cùng cửa mà `currentSelection`/`runLookup` đã đi
 * qua: chính sách quyết ở `main.ts`, tầng này chỉ nhận một vị từ.
 */
export type KeymapGate = {
  /**
   * Trả `true` ⇒ hợp âm bị **nuốt trọn**, không một command nào chạy.
   *
   * ⚠️ Nuốt bằng cách **thoát sớm**, KHÔNG `preventDefault()`: `Escape` của lớp phủ
   * Attribution là một `@keydown.esc` **DOM thường** trên `.attr-scrim`, không một command
   * (§AC11 — `Escape` là một lượt huỷ **trong ngữ cảnh**). Listener này chạy ở pha
   * `capture` trên `window`, tức **trước** nó; chặn dòng sự kiện ở đây là để lớp phủ không
   * đóng được bằng phím, tức nhốt người dùng bàn phím vào trong đúng thứ nó vừa mở.
   */
  isBlocked?: () => boolean
}

/**
 * Gắn keymap vào một `EventTarget` và trả về hàm gỡ.
 *
 * `{ capture: true }` để hợp âm của ứng dụng đứng TRƯỚC handler của component — không
 * thì một `@keydown` ở Editor của Epic 2 nuốt mất `⌘1`.
 *
 * ⚠️ `noUnusedLocals` đang bật: dùng hàm gỡ, hoặc `void` nó tường minh.
 */
const attached = new WeakSet<EventTarget>()

export function attachKeymap(keymap: Keymap, target: EventTarget, gate?: KeymapGate): () => void {
  /**
   * ⚠️ CANH GÁC GỌI LẠI. `installCommands()` ném ở lần gọi thứ hai, nhưng hàm này thì
   * không — nên hai lời gọi cài hai listener capture trên cùng một target và **mọi hợp âm
   * dispatch hai lần**. `setMode` tình cờ idempotent nên ca đó ẩn đi; `focus.next_panel`
   * thì nhảy cách một panel. Cùng lý lẽ với id trùng ở `register()`: ném thì đỏ ngay.
   */
  if (attached.has(target)) {
    throw new Error(
      '[keys] keymap đã gắn vào target này rồi — gắn hai lần là mỗi hợp âm dispatch hai ' +
        'lần. Gọi hàm gỡ do lần gắn trước trả về, hoặc gộp hai chỗ gắn thành một.',
    )
  }
  attached.add(target)
  const listener = (event: Event): void => {
    // Xem [`KeymapGate`]: thoát sớm, không `preventDefault()` — sự kiện phải đi tiếp tới
    // handler DOM của lớp phủ đang mở.
    if (gate?.isBlocked?.() === true) return
    keymap.handle(event as unknown as ChordEvent)
  }
  target.addEventListener('keydown', listener, { capture: true })
  return () => {
    target.removeEventListener('keydown', listener, { capture: true })
    attached.delete(target)
  }
}
