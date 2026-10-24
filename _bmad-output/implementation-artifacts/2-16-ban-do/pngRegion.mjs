import { inflateSync } from 'node:zlib'

/** Decodes a non-interlaced 8-bit RGB/RGBA PNG (what WebKit's window screenshot returns). */
export function decodePng(base64) {
  const buf = Buffer.from(base64, 'base64')
  let pos = 8
  let width = 0
  let height = 0
  let channels = 4
  const idat = []
  while (pos < buf.length) {
    const len = buf.readUInt32BE(pos)
    const type = buf.toString('ascii', pos + 4, pos + 8)
    const data = buf.subarray(pos + 8, pos + 8 + len)
    if (type === 'IHDR') {
      width = data.readUInt32BE(0)
      height = data.readUInt32BE(4)
      const depth = data[8]
      const colour = data[9]
      if (depth !== 8 || data[12] !== 0 || (colour !== 2 && colour !== 6)) throw new Error('unsupported PNG')
      channels = colour === 6 ? 4 : 3
    } else if (type === 'IDAT') idat.push(data)
    pos += 12 + len
  }
  const raw = inflateSync(Buffer.concat(idat))
  const stride = width * channels
  const out = Buffer.alloc(stride * height)
  for (let y = 0; y < height; y++) {
    const filter = raw[y * (stride + 1)]
    for (let x = 0; x < stride; x++) {
      const cur = raw[y * (stride + 1) + 1 + x]
      const left = x >= channels ? out[y * stride + x - channels] : 0
      const up = y > 0 ? out[(y - 1) * stride + x] : 0
      const upLeft = y > 0 && x >= channels ? out[(y - 1) * stride + x - channels] : 0
      let v
      if (filter === 0) v = cur
      else if (filter === 1) v = cur + left
      else if (filter === 2) v = cur + up
      else if (filter === 3) v = cur + ((left + up) >> 1)
      else {
        const p = left + up - upLeft
        const pa = Math.abs(p - left)
        const pb = Math.abs(p - up)
        const pc = Math.abs(p - upLeft)
        v = cur + (pa <= pb && pa <= pc ? left : pb <= pc ? up : upLeft)
      }
      out[y * stride + x] = v & 255
    }
  }
  return { width, height, channels, data: out }
}

/** Number of pixels that differ between two decoded PNGs inside `rect` (CSS px, scaled by `scale`). */
export function differingPixels(a, b, rect, scale) {
  const x0 = Math.max(0, Math.floor(rect.x * scale))
  const y0 = Math.max(0, Math.floor(rect.y * scale))
  const x1 = Math.min(a.width, Math.ceil((rect.x + rect.width) * scale))
  const y1 = Math.min(a.height, Math.ceil((rect.y + rect.height) * scale))
  let count = 0
  for (let y = y0; y < y1; y++) {
    for (let x = x0; x < x1; x++) {
      const i = (y * a.width + x) * a.channels
      if (a.data[i] !== b.data[i] || a.data[i + 1] !== b.data[i + 1] || a.data[i + 2] !== b.data[i + 2]) count++
    }
  }
  return count
}
