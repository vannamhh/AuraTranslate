/**
 * Pure text scanner for `check-tokens.mjs`: no fs, no `process.exit`, safe to `import()`
 * from a test — unlike `check-tokens.mjs` itself.
 *
 * @module scripts/lib/tokens-scan
 */

/**
 * @param {string[]} chars
 * @param {number} s
 * @param {number} e
 */
export const blankRange = (chars, s, e) => {
  for (let i = s; i < e && i < chars.length; i++) if (chars[i] !== '\n') chars[i] = ' '
}

/**
 * @typedef {object} MaskedComment
 * @property {number} index
 * @property {string} text
 */

/**
 * @typedef {object} MaskResult
 * @property {string} masked
 * @property {MaskedComment[]} comments
 */

/**
 * Masks comments (`//`, `/* *\/`) and string/template literals, GIVING BACK THE SAME OFFSETS
 * — callers report line numbers from the ORIGINAL text, so this blanks with spaces (keeping
 * `\n`) rather than deleting.
 *
 * An UNCLOSED string or comment does not mask anything: the old implementation scanned to the
 * end of the text looking for a closing quote, so a single stray `'` (e.g. `don't` in prose)
 * blanked the rest of the file and swallowed real CSS declarations along with it. The rule
 * here: `'` and `"` must close on the SAME LINE to count as a string (matches both JS and CSS
 * string semantics); `` ` `` and `/* *\/` may span lines but must still have a closing token.
 * An unclosed opener is just an ordinary character — advance one step and keep scanning.
 *
 * `cssRanges` turns off `//`-as-comment inside a CSS region: `background: url(//host/x.png)` is
 * not a comment, and blanking it would eat the following `;` and fuse two declarations.
 * @param {string} text
 * @param {{cssRanges?: {start: number, end: number}[]}} [opts]
 * @returns {MaskResult}
 */
export function maskCommentsAndStrings(text, { cssRanges = [] } = {}) {
  const chars = text.split('')
  /** @type {MaskedComment[]} */
  const comments = []
  /** @type {(i: number) => boolean} */
  const inCss = (i) => cssRanges.some((r) => i >= r.start && i < r.end)
  let i = 0
  while (i < text.length) {
    const two = text.slice(i, i + 2)
    if (two === '/*') {
      const end = text.indexOf('*/', i + 2)
      if (end === -1) {
        i += 1
        continue
      }
      const stop = end + 2
      comments.push({ index: i, text: text.slice(i, stop) })
      blankRange(chars, i, stop)
      i = stop
      continue
    }
    if (two === '//' && !inCss(i)) {
      let end = text.indexOf('\n', i)
      if (end === -1) end = text.length
      comments.push({ index: i, text: text.slice(i, end) })
      blankRange(chars, i, end)
      i = end
      continue
    }
    const ch = text[i]
    if (ch === '"' || ch === "'" || ch === '`') {
      let j = i + 1
      let closed = false
      while (j < text.length) {
        if (text[j] === '\\') {
          j += 2
          continue
        }
        if (text[j] === ch) {
          closed = true
          break
        }
        if (text[j] === '\n' && ch !== '`') break
        j += 1
      }
      if (!closed) {
        i += 1
        continue
      }
      blankRange(chars, i + 1, j)
      i = j + 1
      continue
    }
    i += 1
  }
  return { masked: chars.join(''), comments }
}

/**
 * Line number (1-based) of `index` in `text`.
 * @param {string} text
 * @param {number} index
 * @returns {number}
 */
export const lineOf = (text, index) => text.slice(0, index).split('\n').length

/**
 * @typedef {object} CssDecl
 * @property {string} prop
 * @property {string} value
 * @property {number} index
 * @property {*} source
 */

/**
 * @typedef {object} CssBlock
 * @property {string} prelude
 * @property {CssDecl[]} decls
 * @property {*} source
 */

/**
 * Enough of a CSS parser to answer two questions: which declaration carries which value, and
 * where it sits. `masked` must already have comments/strings blanked (offsets preserved) so a
 * `{`/`}`/`;` inside a string can't desync the brace stack.
 * @param {string} masked
 * @param {*} source
 * @returns {CssBlock[]}
 */
export function parseCssBlocks(masked, source) {
  /** @type {CssBlock[]} */
  const blocks = []
  /** @type {CssBlock[]} */
  const stack = []
  let buf = ''

  /**
   * @param {CssBlock | undefined} block
   * @param {number} endIndex
   */
  const flush = (block, endIndex) => {
    const raw = buf.trim()
    buf = ''
    if (!block || !raw) return
    const colon = raw.indexOf(':')
    if (colon <= 0) return
    const prop = raw.slice(0, colon).trim().toLowerCase()
    const value = raw.slice(colon + 1).trim()
    if (!prop || prop.startsWith('@')) return
    block.decls.push({ prop, value, index: endIndex, source })
  }

  for (let i = 0; i < masked.length; i++) {
    const ch = masked[i]
    if (ch === '{') {
      stack.push({ prelude: buf.trim(), decls: [], source })
      buf = ''
    } else if (ch === '}') {
      const block = stack.pop()
      flush(block, i)
      if (block) blocks.push(block)
      buf = ''
    } else if (ch === ';') {
      flush(stack[stack.length - 1], i)
    } else {
      buf += ch
    }
  }
  return blocks
}

/**
 * `borderTopColor` (JS/Vue object-literal key) → `border-top-color` (CSS property).
 * @param {string} s
 * @returns {string}
 */
export const kebab = (s) => s.replace(/([a-z0-9])([A-Z])/g, '$1-$2').toLowerCase()

const STYLE_ATTR_RE = /(?:^|[\s])(:?|v-bind:)style\s*=\s*("([^"]*)"|'([^']*)')/gi

/**
 * `style="…"` / `:style="…"` in markup, as a synthetic one-property-per-declaration block —
 * NOT one block for the whole file, or unrelated tags' declarations would cross-contaminate a
 * single reported line number.
 *
 * Two shapes: a static string (`style="color: red"`) parsed like a CSS declaration list, or a
 * bound expression (`:style="{ color: 'red', fontSize: '13px' }"` or `:style="'color: red'"`)
 * parsed as a JS object literal / string.
 * @param {string} text
 * @param {*} file
 * @returns {CssBlock[]}
 */
export function inlineStyleBlocks(text, file) {
  /** @type {CssBlock[]} */
  const blocks = []
  let m
  const re = new RegExp(STYLE_ATTR_RE.source, 'gi')
  while ((m = re.exec(text))) {
    const bound = m[1] !== '' // `:style` / `v-bind:style` → giá trị là biểu thức JS
    const body = m[3] !== undefined ? m[3] : m[4]
    const base = m.index + m[0].indexOf(body)
    /** @type {CssDecl[]} */
    const decls = []
    if (!bound) {
      let off = 0
      for (const piece of body.split(';')) {
        const colon = piece.indexOf(':')
        if (colon > 0) {
          decls.push({
            prop: piece.slice(0, colon).trim().toLowerCase(),
            value: piece.slice(colon + 1).trim(),
            index: base + off,
            source: file,
          })
        }
        off += piece.length + 1
      }
    } else {
      // Hai hình dạng: một chuỗi (`:style="'color: red'"`) hoặc một object literal.
      const objRe = /(['"]?)([A-Za-z-]+)\1\s*:\s*(?:(['"])(.*?)\3|([^,}]+))/g
      let o
      while ((o = objRe.exec(body))) {
        const value = (o[4] !== undefined ? o[4] : o[5] || '').trim()
        if (!value) continue
        decls.push({ prop: kebab(o[2]).toLowerCase(), value, index: base + o.index, source: file })
      }
    }
    if (decls.length) blocks.push({ prelude: 'style=""', decls, source: file })
  }
  return blocks
}
