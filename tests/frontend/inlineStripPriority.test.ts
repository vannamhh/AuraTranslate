/**
 * `panels/inlineStripPriority.ts::topmostStrip` — Story 3.6.
 *
 * Sổ ưu tiên khoá bằng máy: `EXPERIENCE.md:75-81` chốt thứ tự ba dải TỰ ĐỘNG,
 * `glossary_quick_add` đứng trên cả ba vì nó là thao tác người dùng VỪA yêu cầu. Module
 * THUẦN — không Vue, không DOM.
 */
import { describe, expect, it } from 'vitest'
import { topmostStrip } from '../../src/panels/inlineStripPriority'
import type { InlineStripKind } from '../../src/panels/inlineStripPriority'

describe('topmostStrip — sổ ưu tiên bốn loại dải nội tuyến', () => {
  it('tập rỗng ⇒ null', () => {
    expect(topmostStrip([])).toBeNull()
  })

  it('một mục ⇒ chính nó, cho cả bốn loại', () => {
    const kinds: InlineStripKind[] = ['glossary_quick_add', 'glossary_confirm', 'proofreader', 'tm_fuzzy']
    for (const kind of kinds) {
      expect(topmostStrip([kind])).toBe(kind)
    }
  })

  it('quick-add + confirm ⇒ quick-add thắng', () => {
    expect(topmostStrip(['glossary_confirm', 'glossary_quick_add'])).toBe('glossary_quick_add')
  })

  it('cả bốn cùng đủ điều kiện ⇒ quick-add thắng', () => {
    expect(
      topmostStrip(['tm_fuzzy', 'proofreader', 'glossary_confirm', 'glossary_quick_add']),
    ).toBe('glossary_quick_add')
  })

  it('confirm + proofreader + tm_fuzzy (không quick-add) ⇒ confirm thắng', () => {
    expect(topmostStrip(['tm_fuzzy', 'proofreader', 'glossary_confirm'])).toBe('glossary_confirm')
  })

  it('proofreader + tm_fuzzy (không glossary nào) ⇒ proofreader thắng', () => {
    expect(topmostStrip(['tm_fuzzy', 'proofreader'])).toBe('proofreader')
  })

  it('thứ tự trong mảng `eligible` không ảnh hưởng kết quả — chỉ ưu tiên mới quyết định', () => {
    expect(topmostStrip(['proofreader', 'glossary_confirm', 'tm_fuzzy'])).toBe('glossary_confirm')
    expect(topmostStrip(['glossary_confirm', 'proofreader', 'tm_fuzzy'])).toBe('glossary_confirm')
  })
})
