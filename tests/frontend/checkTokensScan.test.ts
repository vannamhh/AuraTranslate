/**
 * Unit tests for `scripts/lib/tokens-scan.mjs`, the pure text scanner `check-tokens.mjs`
 * uses to enforce AD-34 (it lives in its own module because the gate itself is not
 * import-safe: top-level `await`, `process.exit`).
 */
import { describe, it, expect } from 'vitest'
import {
  blankRange,
  maskCommentsAndStrings,
  lineOf,
  parseCssBlocks,
  kebab,
  inlineStyleBlocks,
} from '../../scripts/lib/tokens-scan.mjs'

describe('blankRange — che bằng khoảng trắng, giữ nguyên `\\n`', () => {
  it('thay mọi ký tự trong khoảng bằng dấu cách, trừ `\\n`', () => {
    const chars = 'ab\ncd'.split('')
    blankRange(chars, 0, chars.length)
    expect(chars.join('')).toBe('  \n  ')
  })

  it('không vượt quá độ dài mảng — `e` lớn hơn `chars.length` không ném', () => {
    const chars = 'ab'.split('')
    expect(() => blankRange(chars, 0, 100)).not.toThrow()
    expect(chars.join('')).toBe('  ')
  })
})

describe('maskCommentsAndStrings — che comment/chuỗi, GIỮ NGUYÊN offset', () => {
  it('che `/* … */` nhiều dòng, giữ số dòng bằng cách giữ lại `\\n`', () => {
    const src = 'a { color: red; } /* var(--dead-token)\nvan con o day */ b { color: blue; }'
    const { masked } = maskCommentsAndStrings(src)
    expect(masked).not.toContain('--dead-token')
    expect(masked.split('\n').length).toBe(src.split('\n').length)
  })

  it('che `//` tới cuối dòng khi KHÔNG ở trong vùng CSS', () => {
    const src = 'const x = 1 // var(--dead-token)\nconst y = 2'
    const { masked } = maskCommentsAndStrings(src)
    expect(masked).not.toContain('--dead-token')
  })

  it('`cssRanges` tắt `//`-là-comment bên trong vùng CSS — `url(//host/x.png)` không bị nuốt', () => {
    const src = 'background: url(//host/x.png); color: red;'
    const { masked } = maskCommentsAndStrings(src, { cssRanges: [{ start: 0, end: src.length }] })
    expect(masked).toContain('url(//host/x.png)')
    expect(masked).toContain('color: red')
  })

  it('một dấu nháy đơn KHÔNG ĐÓNG (kiểu `don\'t` trong văn xuôi) không xoá trắng phần còn lại của tệp', () => {
    const src = "<p>don't</p>\n<style>.x { opacity: 0.4; }</style>"
    const { masked } = maskCommentsAndStrings(src)
    expect(masked).toContain('opacity: 0.4')
  })

  it('trả về danh sách `comments` kèm vị trí bắt đầu', () => {
    const src = 'a { color: red; } /* ghi chu */'
    const { comments } = maskCommentsAndStrings(src)
    expect(comments).toHaveLength(1)
    expect(comments[0].text).toBe('/* ghi chu */')
    expect(comments[0].index).toBe(src.indexOf('/*'))
  })
})

describe('lineOf — số dòng 1-based của một offset', () => {
  it('offset ở dòng đầu ⇒ 1', () => {
    expect(lineOf('abc\ndef', 1)).toBe(1)
  })

  it('offset sau ký tự xuống dòng đầu tiên ⇒ 2', () => {
    const text = 'abc\ndef'
    expect(lineOf(text, text.indexOf('def'))).toBe(2)
  })
})

describe('parseCssBlocks — khai báo nào mang giá trị gì, ở đâu', () => {
  it('đọc đúng `prop`/`value` của một khối, prop hạ về chữ thường', () => {
    const { masked } = maskCommentsAndStrings('.x { Gap: var(--space-unit); }')
    const blocks = parseCssBlocks(masked, 'fixture.css')
    expect(blocks).toHaveLength(1)
    expect(blocks[0].decls).toEqual([
      expect.objectContaining({ prop: 'gap', value: 'var(--space-unit)' }),
    ])
  })

  it('nhiều khai báo trong một khối, phân tách bởi `;`', () => {
    const { masked } = maskCommentsAndStrings('.x { color: red; gap: 4px; }')
    const blocks = parseCssBlocks(masked, 'fixture.css')
    expect(blocks[0].decls.map((d) => d.prop)).toEqual(['color', 'gap'])
  })

  it('một chuỗi/comment đã bị che không tạo ra khai báo giả — vào qua `masked`, không phải văn bản gốc', () => {
    const raw = '.x { /* color: red; */ gap: 4px; }'
    const { masked } = maskCommentsAndStrings(raw)
    const blocks = parseCssBlocks(masked, 'fixture.css')
    expect(blocks[0].decls.map((d) => d.prop)).toEqual(['gap'])
  })

  it('bỏ qua at-rule (`@media { … }`) — không đọc `@…` làm một prop', () => {
    const { masked } = maskCommentsAndStrings('@media (min-width: 1px) { .x { color: red; } }')
    const blocks = parseCssBlocks(masked, 'fixture.css')
    expect(blocks.some((b) => b.decls.some((d) => d.prop.startsWith('@')))).toBe(false)
  })
})

describe('kebab — object-literal key (JS/Vue) → CSS property', () => {
  it('chuyển `borderTopColor` thành `border-top-color`', () => {
    expect(kebab('borderTopColor')).toBe('border-top-color')
  })

  it('giữ nguyên một tên đã là kebab-case', () => {
    expect(kebab('gap')).toBe('gap')
  })
})

describe('inlineStyleBlocks — `style="…"` / `:style="…"` trong markup', () => {
  it('đọc một `style=""` tĩnh thành các khai báo riêng biệt', () => {
    const src = '<div style="color: red; gap: 4px"></div>'
    const blocks = inlineStyleBlocks(src, 'fixture.vue')
    expect(blocks).toHaveLength(1)
    expect(blocks[0].decls.map((d) => d.prop)).toEqual(['color', 'gap'])
  })

  it('đọc một `:style` object literal, kebab-hoá tên thuộc tính', () => {
    const src = `<div :style="{ color: 'red', fontSize: '13px' }"></div>`
    const blocks = inlineStyleBlocks(src, 'fixture.vue')
    const props = blocks[0].decls.map((d) => d.prop)
    expect(props).toEqual(['color', 'font-size'])
  })

  it('đọc một `:style` là một chuỗi (`:style="\'color: red\'"`) — bóc đúng lớp nháy ngoài', () => {
    const src = `<div :style="'color: red'"></div>`
    const blocks = inlineStyleBlocks(src, 'fixture.vue')
    expect(blocks[0].decls[0]).toEqual(expect.objectContaining({ prop: 'color', value: 'red' }))
  })

  it('nhiều khai báo trong một `:style` chuỗi, phân tách bởi `;`', () => {
    const src = `<div :style="'color: red; gap: 4px'"></div>`
    const blocks = inlineStyleBlocks(src, 'fixture.vue')
    expect(blocks[0].decls.map((d) => d.prop)).toEqual(['color', 'gap'])
    expect(blocks[0].decls.map((d) => d.value)).toEqual(['red', '4px'])
  })

  it('mỗi thẻ mang MỘT khối riêng — không gộp hai `style=""` không liên quan vào một khối', () => {
    const src = '<div style="color: red"></div><span style="gap: 4px"></span>'
    const blocks = inlineStyleBlocks(src, 'fixture.vue')
    expect(blocks).toHaveLength(2)
  })
})
