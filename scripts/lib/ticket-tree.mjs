import { readdirSync, readFileSync } from 'node:fs'
import { join } from 'node:path'

export const INITIATIVE_DIR = ['_bmad-output', 'initiative-auratranslate']
export const TICKETS_DIR = [...INITIATIVE_DIR, 'tickets']

const TABLE_RE = /^\[\[(epic|entry)\]\]$/
const KEY_RE = /^([A-Za-z_][A-Za-z0-9_]*)\s*=\s*(.*)$/
const ID_RE = /^(?:(\d+[a-z]?)|"(\d+[a-z]?)")$/
const SLUG_RE = /^"([a-z0-9][a-z0-9-]*)"$/

/** @typedef {{ id: string | null, slug: string | null }} TicketRow */

/**
 * Strict subset reader: any shape other than `[[kind]]` tables of one-line pairs throws.
 * @param {string} text
 * @param {string} kind
 * @param {string} where
 * @returns {TicketRow[]}
 */
export function parseTicketsToml(text, kind, where) {
  /** @type {TicketRow[]} */
  const rows = []
  /** @type {TicketRow | null} */
  let cur = null
  const lines = text.split('\n')
  /** @type {string | null} */
  let inBlock = null
  for (let i = 0; i < lines.length; i++) {
    const line = lines[i].replace(/\r$/, '').trim()
    if (inBlock) {
      if (line.includes(inBlock)) inBlock = null
      continue
    }
    if (line === '' || line.startsWith('#')) continue
    const t = TABLE_RE.exec(line)
    if (t) {
      if (t[1] !== kind) throw new Error(`${where}:${i + 1} bang \`[[${t[1]}]]\` la, mong \`[[${kind}]]\``)
      cur = { id: null, slug: null }
      rows.push(cur)
      continue
    }
    const k = KEY_RE.exec(line)
    if (!k || !cur) throw new Error(`${where}:${i + 1} dong khong dung hinh dang: ${line}`)
    const open = ['"""', "'''"].find((q) => k[2].startsWith(q))
    if (open && !k[2].slice(3).includes(open)) {
      if (k[1] === 'id' || k[1] === 'slug') throw new Error(`${where}:${i + 1} \`${k[1]}\` nhieu dong: ${line}`)
      inBlock = open
    } else if (k[1] === 'id') {
      const m = ID_RE.exec(k[2].trim())
      if (!m || cur.id !== null) throw new Error(`${where}:${i + 1} dong \`id\` la: ${line}`)
      cur.id = m[1] ?? m[2]
    } else if (k[1] === 'slug') {
      const m = SLUG_RE.exec(k[2].trim())
      if (!m || cur.slug !== null) throw new Error(`${where}:${i + 1} dong \`slug\` la: ${line}`)
      cur.slug = m[1]
    }
  }
  if (inBlock !== null) throw new Error(`${where}: chuoi nhieu dong khong dong`)
  /** @type {Set<string>} */
  const seen = new Set()
  for (const r of rows) {
    if (r.id === null) throw new Error(`${where}: co \`[[${kind}]]\` khong co \`id\``)
    if (seen.has(r.id)) throw new Error(`${where}: id ${r.id} xuat hien hai lan`)
    seen.add(r.id)
    if (kind === 'epic' && r.slug === null) throw new Error(`${where}: epic ${r.id} khong co \`slug\``)
  }
  return rows
}

/**
 * @param {string} text
 * @param {string} key
 * @returns {string | null}
 */
function frontmatterValue(text, key) {
  const lines = text.split('\n')
  if (lines[0].replace(/\r$/, '') !== '---') return null
  const re = new RegExp(`^${key}:\\s*(.*)$`)
  for (let i = 1; i < lines.length; i++) {
    const line = lines[i].replace(/\r$/, '')
    if (line === '---') return null
    const m = re.exec(line)
    if (!m) continue
    let v = m[1].trim()
    const q = /^(['"])(.*?)\1(?:\s+#.*)?$/.exec(v)
    v = q ? q[2] : v.replace(/\s+#.*$/, '')
    return v === '' ? null : v
  }
  return null
}

/**
 * Throws on a missing, malformed or empty tree; a story's status is null when its entry has no plan or the plan has no status.
 * @param {string} root
 */
export function readTicketTree(root) {
  const base = join(root, ...TICKETS_DIR)
  const epicRows = parseTicketsToml(readFileSync(join(base, 'tickets.toml'), 'utf8'), 'epic', 'tickets/tickets.toml')
  /** @type {Map<string, string | null>} */
  const stories = new Map()
  /** @type {Map<string, string | null>} */
  const epics = new Map()
  /** @type {Map<string, string>} */
  const planPaths = new Map()
  /** @type {string[]} */
  const searched = []
  for (const ep of epicRows) {
    if (ep.slug === null || ep.id === null) throw new Error('tickets.toml: epic thieu id hoac slug')
    const dir = join(base, ep.slug)
    const leafDir = join(dir, 'tickets')
    searched.push(leafDir)
    const entries = parseTicketsToml(
      readFileSync(join(leafDir, 'tickets.toml'), 'utf8'),
      'entry',
      `${ep.slug}/tickets/tickets.toml`,
    )
    const epicFile = join(dir, `${ep.slug}.md`)
    epics.set(`epic-${ep.id}`, frontmatterValue(readFileSync(epicFile, 'utf8'), 'status'))
    for (const e of entries) stories.set(`${ep.id}-${e.id}`, null)
    for (const f of readdirSync(leafDir)) {
      if (!f.endsWith('.md')) continue
      const text = readFileSync(join(leafDir, f), 'utf8')
      const ticket = frontmatterValue(text, 'ticket')
      if (ticket === null) continue
      const key = `${ep.id}-${ticket}`
      if (!stories.has(key)) throw new Error(`${ep.slug}/tickets/${f}: ticket ${ticket} khong co trong tickets.toml cua epic`)
      if (planPaths.has(key)) throw new Error(`${ep.slug}/tickets/${f}: ticket ${ticket} co hai plan`)
      planPaths.set(key, join(leafDir, f))
      stories.set(key, frontmatterValue(text, 'status'))
    }
  }
  if (stories.size === 0) throw new Error('cay ve doc ra 0 ticket')
  return {
    stories,
    epics,
    searched,
    /** @param {string} ref */
    planFor(ref) {
      const m = /^(\d+)[.-](\d+[a-z]?)$/.exec(ref)
      return m ? (planPaths.get(`${m[1]}-${m[2]}`) ?? null) : null
    },
  }
}
