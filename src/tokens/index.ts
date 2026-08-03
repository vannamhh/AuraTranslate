/**
 * Áp bộ token lên `document.documentElement` dưới dạng CSS custom properties.
 * Story 1.4 · AD-34 · NFR17.
 *
 * ⚠️ Gọi `applyTheme()` TRƯỚC `mount()` trong `main.ts`. Ngược lại thì lượt render đầu
 * chạy với biến chưa tồn tại — mọi `var(--color-…)` rơi về giá trị rỗng, và người dùng
 * thấy một nháy trắng trước khi giao diện lên màu. Trên bản đóng gói nháy đó ngắn hơn
 * hẳn so với máy dev, nên nó là loại lỗi chỉ lộ ra ở máy người khác.
 *
 * ─────────────────────────────────────────────────────────────────────────────────
 * QUY ƯỚC TÊN BIẾN — chốt MỘT LẦN ở story này, 127 story sau dùng lại.
 * ─────────────────────────────────────────────────────────────────────────────────
 *
 *   --color-<token>       16 màu của theme đang áp        var(--color-on-surface)
 *   --family-<họ>         4 họ chữ                        var(--family-read)
 *   --space-<token>       khoảng cách + thước đọc         var(--space-panel-inline)
 *   --radius-<token>      bo góc (`DEFAULT` → `default`)  var(--radius-default)
 *
 * Mỗi token typography phát BẢY biến, mỗi biến đúng một nghĩa — không có biến rút gọn
 * kiểu `font` shorthand, vì shorthand không chở được `letter-spacing` với
 * `font-synthesis` và một token chở nửa nghĩa sẽ bị dùng như thể chở đủ:
 *
 *   --font-<token>        font-size          --leading-<token>    line-height
 *   --weight-<token>      font-weight        --style-<token>      font-style
 *   --tracking-<token>    letter-spacing     --synthesis-<token>  font-synthesis
 *   --face-<token>        font-family (trỏ về `--family-<họ>` của chính token)
 *
 * Bảy biến LUÔN được phát, kể cả khi token không khai (mặc định `400` / `normal` /
 * `normal` / `auto`). Nhờ vậy một component áp trọn bộ mà không cần biết token nào khai gì.
 */
import tokens from './tokens.json'

export type Theme = 'light' | 'dark'

/** Tên 16 token màu — dùng được ở kiểu, không chỉ ở lúc chạy. */
export type ColorToken = keyof typeof tokens.colors.light
export type TypographyToken = Exclude<keyof typeof tokens.typography, '$doc'>
export type FamilyToken = Exclude<keyof typeof tokens.families, '$doc'>

/** `$doc` là chú giải cho người đọc tệp JSON, không phải token. Lọc ở đúng một chỗ. */
const isToken = (key: string): boolean => key !== '$doc'

type PlainRecord = Record<string, unknown>

/** `DEFAULT` là tên trong DESIGN.md; biến CSS thì viết thường cho khớp phần còn lại. */
const cssName = (key: string): string => (key === 'DEFAULT' ? 'default' : key)

export const THEMES: readonly Theme[] = ['light', 'dark']

/** Theme mặc định — nền giấy, đúng hướng "Bàn viết". */
export const DEFAULT_THEME: Theme = 'light'

export const isTheme = (value: unknown): value is Theme =>
  typeof value === 'string' && (THEMES as readonly string[]).includes(value)

/**
 * Ghi toàn bộ token lên `:root`.
 *
 * Phần không đổi theo theme (họ chữ, typography, spacing, rounded) vẫn được ghi lại ở
 * mỗi lượt gọi. Ghi thừa vài chục dòng là rẻ; một nhánh "chỉ ghi lần đầu" thì phải
 * đúng ở mọi thứ tự gọi, và nó sẽ sai ở lần ai đó gọi `applyTheme` từ một chỗ mới.
 *
 * ⚠️ THAM SỐ ĐƯỢC KIỂM LÚC CHẠY, không chỉ lúc biên dịch. Kiểu `Theme` chỉ canh được
 * những nơi TypeScript nhìn thấy; Story 1.8 sẽ nạp lựa chọn theme **từ đĩa**, nơi giá
 * trị có thể là `'system'`, `''` hay `undefined` sau một lần sửa tay tệp cấu hình.
 * Không có chốt này thì `Object.entries(undefined)` ném `TypeError` ở **tầng module** —
 * `main.ts` gọi hàm này trước `mount()`, nên ứng dụng không mount và người dùng thấy
 * một cửa sổ trắng, đúng thứ khối chú thích đầu tệp nói mình tồn tại để chặn.
 * Rơi về `light` và kêu to hơn là chết im lặng.
 */
export function applyTheme(theme: Theme, root: HTMLElement = document.documentElement): void {
  if (!isTheme(theme)) {
    console.warn(
      `[tokens] theme không hợp lệ: ${JSON.stringify(theme)} — rơi về '${DEFAULT_THEME}'. ` +
        `Giá trị hợp lệ: ${THEMES.join(' · ')}.`,
    )
    theme = DEFAULT_THEME
  }
  const style = root.style

  // ── Màu ────────────────────────────────────────────────────────────────────────
  const palette = tokens.colors[theme] as unknown as PlainRecord
  for (const [name, value] of Object.entries(palette)) {
    if (isToken(name)) style.setProperty(`--color-${name}`, String(value))
  }

  // ── Họ chữ ─────────────────────────────────────────────────────────────────────
  for (const [name, value] of Object.entries(tokens.families as unknown as PlainRecord)) {
    if (isToken(name)) style.setProperty(`--family-${name}`, String(value))
  }

  // ── Typography — bảy biến mỗi token ────────────────────────────────────────────
  for (const [name, raw] of Object.entries(tokens.typography as unknown as PlainRecord)) {
    if (!isToken(name)) continue
    const t = raw as PlainRecord
    style.setProperty(`--font-${name}`, String(t.fontSize))
    style.setProperty(`--leading-${name}`, String(t.lineHeight))
    style.setProperty(`--weight-${name}`, String(t.fontWeight ?? '400'))
    style.setProperty(`--style-${name}`, String(t.fontStyle ?? 'normal'))
    style.setProperty(`--tracking-${name}`, String(t.letterSpacing ?? 'normal'))
    style.setProperty(`--synthesis-${name}`, String(t.fontSynthesis ?? 'auto'))
    style.setProperty(`--face-${name}`, `var(--family-${String(t.family)})`)
  }

  // ── Khoảng cách và hình dạng ───────────────────────────────────────────────────
  for (const [name, value] of Object.entries(tokens.spacing as unknown as PlainRecord)) {
    if (isToken(name)) style.setProperty(`--space-${name}`, String(value))
  }
  for (const [name, value] of Object.entries(tokens.rounded as unknown as PlainRecord)) {
    if (isToken(name)) style.setProperty(`--radius-${cssName(name)}`, String(value))
  }

  // ── Phân tách panel (AC6) ──────────────────────────────────────────────────────
  //
  // ⚠️ Hai theme khai HAI cơ chế khác nhau, và đó là điều kiện nghiệm thu chứ không
  // phải một chi tiết cài đặt: mặt sáng phân tách bằng NÉT, mặt tối bằng KHE. Bê cách
  // của theme sáng sang theme tối làm bốn panel chìm thành một khối nâu — `outline`
  // trên `surface` ở theme tối chỉ đạt 1,32:1.
  //
  // Story này KHÔNG dựng panel (Story 1.14). Nó chỉ khai cơ chế; `check-tokens.mjs`
  // canh cho hai theme không bị thống nhất về một cách làm.
  const sep = tokens.panelSeparator[theme] as unknown as PlainRecord
  style.setProperty('--panel-separator-mechanism', String(sep.mechanism))
  style.setProperty('--panel-gap', String(sep.gap))
  style.setProperty('--panel-border-width', String(sep.borderWidth))
  style.setProperty(
    '--panel-border-color',
    sep.borderColor === null ? 'transparent' : `var(--color-${String(sep.borderColor)})`,
  )
  style.setProperty('--panel-radius', `var(--radius-${cssName(String(sep.radius))})`)

  // Thanh cuộn, ô nhập và mọi widget do hệ điều hành vẽ đi theo theme. Thiếu dòng này
  // thì ở theme tối vẫn còn một dải sáng chạy dọc mép phải — thứ không token nào chạm tới.
  root.style.colorScheme = theme
  root.dataset.theme = theme
}

export { tokens }
