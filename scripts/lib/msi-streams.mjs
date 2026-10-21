/**
 * Lists the streams of an `.msi` (a Compound File Binary) with their sizes, reading only the
 * header, FAT and directory sectors. `msiexec /a` never extracts the Binary table, so this is
 * the only way to see what an MSI carries there (the WebView2 offline installer, NFR6).
 */
import { closeSync, openSync, readSync } from 'node:fs'

const MAGIC = Buffer.from('d0cf11e0a1b11ae1', 'hex')
const END_OF_CHAIN = 0xfffffffa
const STREAM = 2

/** @param {number} v */
function msiChar(v) {
  if (v < 10) return String.fromCharCode(48 + v)
  if (v < 36) return String.fromCharCode(65 + v - 10)
  if (v < 62) return String.fromCharCode(97 + v - 36)
  return v === 62 ? '.' : '_'
}

/**
 * MSI packs two name characters into one UTF-16 unit in U+3800..U+47FF, one in U+4800..U+483F,
 * and marks table streams with U+4840 (`!`).
 *
 * @param {string} raw
 */
export function decodeMsiStreamName(raw) {
  let out = ''
  for (const ch of raw) {
    const c = ch.charCodeAt(0)
    if (c >= 0x3800 && c < 0x4800) out += msiChar((c - 0x3800) & 0x3f) + msiChar(((c - 0x3800) >> 6) & 0x3f)
    else if (c >= 0x4800 && c < 0x4840) out += msiChar(c - 0x4800)
    else if (c === 0x4840) out += '!'
    else out += ch
  }
  return out
}

/** @param {Buffer} buf */
const words = (buf) => Array.from({ length: buf.length / 4 }, (_, i) => buf.readUInt32LE(i * 4))

/**
 * @param {string} path
 * @returns {Array<{ name: string, size: number }>}
 */
export function msiStreamSizes(path) {
  const fd = openSync(path, 'r')
  try {
    /** @param {number} offset @param {number} length */
    const read = (offset, length) => {
      const buf = Buffer.alloc(length)
      if (readSync(fd, buf, 0, length, offset) !== length) throw new Error(`${path}: truncated at byte ${offset}`)
      return buf
    }
    const header = read(0, 512)
    if (!header.subarray(0, 8).equals(MAGIC)) throw new Error(`${path}: not a Compound File (MSI) package`)
    const major = header.readUInt16LE(0x1a)
    const sectorSize = 2 ** header.readUInt16LE(0x1e)
    /** @param {number} n */
    const sector = (n) => read((n + 1) * sectorSize, sectorSize)
    const difat = words(header.subarray(0x4c, 512))
    for (let s = header.readUInt32LE(0x44); s < END_OF_CHAIN; ) {
      const next = words(sector(s))
      difat.push(...next.slice(0, -1))
      s = /** @type {number} */ (next.at(-1))
    }
    const fat = difat.slice(0, header.readUInt32LE(0x2c)).flatMap((s) => words(sector(s)))
    /** @type {Array<{ name: string, size: number }>} */
    const streams = []
    let visited = 0
    for (let s = header.readUInt32LE(0x30); s < END_OF_CHAIN; s = fat[s] ?? END_OF_CHAIN) {
      if (++visited > fat.length) throw new Error(`${path}: directory chain loops`)
      const dir = sector(s)
      for (let off = 0; off < sectorSize; off += 128) {
        const nameBytes = dir.readUInt16LE(off + 64)
        if (dir[off + 66] !== STREAM || nameBytes < 2) continue
        const high = major >= 4 ? dir.readUInt32LE(off + 124) : 0
        streams.push({
          name: decodeMsiStreamName(dir.toString('utf16le', off, off + nameBytes - 2)),
          size: high * 2 ** 32 + dir.readUInt32LE(off + 120),
        })
      }
    }
    return streams
  } finally {
    closeSync(fd)
  }
}
