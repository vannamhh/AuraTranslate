/**
 * Pure text scanner for `check-commands.mjs`: no fs, no `process.exit`, safe to `import()`
 * from a test — unlike `check-commands.mjs` itself.
 *
 * @module scripts/lib/commands-scan
 */

const IDENT_CHAR = /[A-Za-z0-9_]/

const REGEX_PRECEDERS = new Set([...'([{},;:=!&|?+-*%^~<>'])
const REGEX_KEYWORDS = new Set([
  'return', 'typeof', 'instanceof', 'in', 'of', 'case', 'do', 'else',
  'yield', 'await', 'new', 'delete', 'void', 'throw',
])

/**
 * Whether a `/` opens a regex rather than being division, given the last significant char.
 * `}` must stay in the preceder set: a regex opening right after a block (`function f()
 * {}` then `/^https?:\/\//`) would otherwise read as division and swallow a real `dispatch(`.
 * @param {string} lastSig
 * @param {string} text
 * @param {number} at
 * @returns {boolean}
 */
function regexAllowed(lastSig, text, at) {
  if (lastSig === '') return true
  if (REGEX_PRECEDERS.has(lastSig)) return true
  if (!IDENT_CHAR.test(lastSig)) return false
  const m = /[A-Za-z_$][A-Za-z0-9_$]*$/.exec(text.slice(Math.max(0, at - 24), at).trimEnd())
  return m ? REGEX_KEYWORDS.has(m[0]) : false
}

/**
 * @param {string[]} chars
 * @param {number} s
 * @param {number} e
 */
export const blank = (chars, s, e) => {
  for (let i = s; i < e && i < chars.length; i += 1) if (chars[i] !== '\n') chars[i] = ' '
}

/**
 * @typedef {object} MaskScriptOptions
 * @property {boolean} [blankLiterals] Also blanks string/template-literal content (keeps
 *   `${...}`), producing an additive `code` view alongside `masked` — never a replacement.
 */

/**
 * JS/TS region masking: always blanks `//` and `/* *\/`, and tracks string/template/regex
 * literals so a `//`/`/*` inside one doesn't open a fake comment.
 * @param {string} text
 * @param {number} from
 * @param {number} to
 * @param {string[]} chars
 * @param {MaskScriptOptions} [opts]
 */
export function maskScript(text, from, to, chars, opts = {}) {
  const blankLiterals = opts.blankLiterals === true
  let i = from
  let state = 'code'
  let quote = ''
  let lastSig = ''
  /** @type {number[]} */
  const interp = []
  let literalStart = -1

  while (i < to) {
    const ch = text[i]
    if (state === 'code') {
      if (text.startsWith('/*', i)) {
        let end = text.indexOf('*/', i + 2)
        end = end === -1 || end + 2 > to ? to : end + 2
        blank(chars, i, end)
        i = end
        continue
      }
      if (text.startsWith('//', i)) {
        let end = text.indexOf('\n', i)
        if (end === -1 || end > to) end = to
        blank(chars, i, end)
        i = end
        continue
      }
      if (ch === '"' || ch === "'") {
        state = 'string'
        quote = ch
        literalStart = i
        i += 1
        continue
      }
      if (ch === '`') {
        state = 'template'
        literalStart = i + 1
        i += 1
        continue
      }
      if (ch === '/' && regexAllowed(lastSig, text, i)) {
        let j = i + 1
        let inClass = false
        let closed = false
        while (j < to) {
          const c = text[j]
          if (c === '\\') {
            j += 2
            continue
          }
          if (c === '\n') break
          if (c === '[') inClass = true
          else if (c === ']') inClass = false
          else if (c === '/' && !inClass) {
            closed = true
            break
          }
          j += 1
        }
        if (closed) {
          j += 1
          while (j < to && /[a-z]/.test(text[j])) j += 1
          lastSig = '/'
          i = j
          continue
        }
        // Unclosed: the guess was wrong, this `/` is division. Advance one step.
      }
      if (interp.length) {
        if (ch === '{') interp[interp.length - 1] += 1
        else if (ch === '}') {
          if (interp[interp.length - 1] === 0) {
            interp.pop()
            state = 'template'
            literalStart = i + 1
            i += 1
            continue
          }
          interp[interp.length - 1] -= 1
        }
      }
      if (!/\s/.test(ch)) lastSig = ch
      i += 1
      continue
    }
    if (state === 'string') {
      if (ch === '\\') {
        i += 2
        continue
      }
      // A single-quote string must close on the same line, or `don't` in prose would
      // swallow the rest of the file.
      if (ch === '\n' || ch === quote) {
        if (blankLiterals) blank(chars, literalStart, ch === quote ? i + 1 : i)
        state = 'code'
        lastSig = quote
        i += 1
        continue
      }
      i += 1
      continue
    }
    // state === 'template'
    if (ch === '\\') {
      i += 2
      continue
    }
    if (text.startsWith('${', i)) {
      if (blankLiterals) blank(chars, literalStart, i)
      interp.push(0)
      state = 'code'
      i += 2
      continue
    }
    if (ch === '`') {
      if (blankLiterals) blank(chars, literalStart, i)
      state = 'code'
      lastSig = '`'
      i += 1
      continue
    }
    i += 1
  }
}

/**
 * Template region masking: blanks `<!-- -->`, only in text state — `<!--` inside an
 * attribute value (`title="a <!-- b"`) is valid HTML, not a comment.
 * @param {string} text
 * @param {number} from
 * @param {number} to
 * @param {string[]} chars
 */
export function maskTemplate(text, from, to, chars) {
  let i = from
  let state = 'text'
  let quote = ''
  while (i < to) {
    const ch = text[i]
    if (state === 'text') {
      if (text.startsWith('<!--', i)) {
        const end = text.indexOf('-->', i + 4)
        const stop = end === -1 || end + 3 > to ? to : end + 3
        blank(chars, i, stop)
        i = stop
        continue
      }
      if (ch === '<' && /[A-Za-z/]/.test(text[i + 1] ?? '')) {
        state = 'tag'
        i += 1
        continue
      }
      i += 1
      continue
    }
    if (state === 'tag') {
      if (ch === '"' || ch === "'") {
        state = 'attr'
        quote = ch
        i += 1
        continue
      }
      if (ch === '>') {
        state = 'text'
        i += 1
        continue
      }
      i += 1
      continue
    }
    if (ch === quote) {
      state = 'tag'
      i += 1
      continue
    }
    i += 1
  }
}

// An attribute value may contain `>` (`@click="a > b ? f() : g()"` is valid Vue), so a
// value closes on its quote character, never on `>`.
const ATTR_NAME_START = /[@:A-Za-z_]/
const ATTR_NAME_CHAR = /[@:A-Za-z0-9_.\-[\]]/

/**
 * @typedef {object} TemplateAttr
 * @property {string} name
 * @property {string} value
 * @property {number} index
 */

/**
 * Extracts attributes in a template region — stateful.
 * @param {string} masked
 * @param {number} from
 * @param {number} to
 * @returns {TemplateAttr[]}
 */
export function attributesIn(masked, from, to) {
  /** @type {TemplateAttr[]} */
  const out = []
  let i = from
  let state = 'text'
  while (i < to) {
    const ch = masked[i]
    if (state === 'text') {
      if (ch === '<' && /[A-Za-z/]/.test(masked[i + 1] ?? '')) {
        state = 'tag'
        i += 1
        continue
      }
      i += 1
      continue
    }
    // state === 'tag'
    if (ch === '>') {
      state = 'text'
      i += 1
      continue
    }
    if (!ATTR_NAME_START.test(ch)) {
      i += 1
      continue
    }
    let j = i
    while (j < to && ATTR_NAME_CHAR.test(masked[j])) j += 1
    const name = masked.slice(i, j)
    let k = j
    while (k < to && /\s/.test(masked[k])) k += 1
    if (masked[k] !== '=') {
      i = j
      continue
    }
    k += 1
    while (k < to && /\s/.test(masked[k])) k += 1
    const q = masked[k]
    if (q === '"' || q === "'") {
      let e = k + 1
      while (e < to && masked[e] !== q) e += 1
      out.push({ name, value: masked.slice(k + 1, e), index: k + 1 })
      i = Math.min(e + 1, to)
      continue
    }
    let e = k
    while (e < to && !/[\s>]/.test(masked[e])) e += 1
    out.push({ name, value: masked.slice(k, e), index: k })
    i = e
  }
  return out
}

/**
 * @typedef {object} TemplateRegion
 * @property {number} start
 * @property {number} end
 */

/**
 * @typedef {object} ParsedFile
 * @property {string} file
 * @property {string} text
 * @property {string} masked
 * @property {string} [code]
 * @property {TemplateRegion[]} templates
 * @property {boolean} isVue
 */

/**
 * @typedef {object} ScannedAttr
 * @property {ParsedFile} p
 * @property {TemplateAttr} a
 */

/**
 * The body range of `function <name>(...) { ... }` on a `code` view (strings already
 * blanked, so a `{` inside one can't desync the depth count). Returns `null` when there is
 * no local `function` declaration (e.g. the name is imported, or declared as an arrow) —
 * callers decide what that means, this never guesses a body that isn't there.
 * @param {string} code
 * @param {string} name
 * @returns {{start: number, end: number}|null}
 */
export function functionBodyRange(code, name) {
  const head = new RegExp(`\\bfunction\\s+${name}\\s*\\(`)
  const m = head.exec(code)
  if (!m) return null
  const open = code.indexOf('{', m.index)
  if (open === -1) return null
  let depth = 0
  for (let i = open; i < code.length; i += 1) {
    if (code[i] === '{') depth += 1
    else if (code[i] === '}') {
      depth -= 1
      if (depth === 0) return { start: open + 1, end: i }
    }
  }
  return null
}

/**
 * Every attribute in every template region of the parsed `.vue` files — callers filter by
 * name themselves, this does not.
 * @param {ParsedFile[]} parsedFiles
 * @returns {ScannedAttr[]}
 */
export function scanVueAttrs(parsedFiles) {
  /** @type {ScannedAttr[]} */
  const found = []
  for (const p of parsedFiles) {
    if (!p.isVue) continue
    for (const region of p.templates) {
      for (const a of attributesIn(p.masked, region.start, region.end)) {
        found.push({ p, a })
      }
    }
  }
  return found
}
