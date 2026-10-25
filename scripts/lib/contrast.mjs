export const HEX6_RE = /^#[0-9a-fA-F]{6}$/
/** @param {number} c */
const srgbChannel = (c) => (c <= 0.03928 ? c / 12.92 : ((c + 0.055) / 1.055) ** 2.4)

/** @param {string} hex */
function luminance(hex) {
  const n = parseInt(hex.slice(1), 16)
  const [r, g, b] = [(n >> 16) & 255, (n >> 8) & 255, n & 255].map((v) => srgbChannel(v / 255))
  return 0.2126 * r + 0.7152 * g + 0.0722 * b
}

/**
 * ⚠️ Ném thay vì trả `NaN`. `parseInt('#abc'.slice(1), 16)` cho 2748 — một số HỢP LỆ cho
 * một giá trị SAI; với `rgb(…)` hay tên màu thì ra `NaN`, và `NaN < 4.5` là `false`, tức
 * cặp được tuyên bố ĐẠT. Một phép so sánh im lặng nhận `NaN` là chỗ mà mọi phán quyết
 * của Kiểm C thành rác kèm hai chữ số thập phân trông rất thuyết phục.
 * @param {string} a
 * @param {string} b
 * @returns {number}
 */
export function contrast(a, b) {
  if (!HEX6_RE.test(a) || !HEX6_RE.test(b)) {
    throw new Error(`giá trị màu phải là \`#rrggbb\`; nhận \`${a}\` và \`${b}\``)
  }
  const [hi, lo] = [luminance(a), luminance(b)].sort((x, y) => y - x)
  return (hi + 0.05) / (lo + 0.05)
}
