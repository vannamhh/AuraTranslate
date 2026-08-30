/**
 * `modes/readingState.ts` — nhóm ③ typography (Story 5.11).
 *
 * ⚠️ **PHẠM VI** — sàn giãn dòng 1.66 lúc chạy KHÔNG đi qua cổng nào (`check:tokens` Kiểm E
 * chỉ đọc `tokens.json`, xem doc-comment của `READING_LINE_HEIGHT_FLOOR`). Bộ này là lưới
 * DUY NHẤT của mệnh đề đó. Ba số của mỗi mức đọc TỪ `tokens.json`, không viết thẳng vào
 * test — một test viết thẳng `19px` là một bản chép thứ hai của cùng một sự thật (§Design
 * Notes của story).
 */
import { beforeEach, describe, expect, it } from 'vitest'
import { tokens } from '../../src/tokens'

beforeEach(async () => {
  const state = await import('../../src/modes/readingState')
  state.resetReading()
  state.resetReadingToc()
  state.resetReadingPreferences()
})

describe('modes/readingState.ts — ba mức đọc từ tokens.json', () => {
  it.each([
    ['lg', 'read-lg', 'read-measure-lg'],
    ['md', 'read-md', 'read-measure-md'],
    ['sm', 'read-sm', 'read-measure-sm'],
  ] as const)('mức %s khớp đúng token %s / %s', async (level, typographyKey, measureKey) => {
    const state = await import('../../src/modes/readingState')
    state.setReadingLevel(level)

    const expectedFontSize = tokens.typography[typographyKey].fontSize
    const expectedLineHeight = tokens.typography[typographyKey].lineHeight
    const expectedMeasure = tokens.spacing[measureKey]

    expect(state.readingStyle.value.fontSize).toBe(`${Number.parseFloat(expectedFontSize)}px`)
    expect(state.readingStyle.value.lineHeight).toBe(String(Number.parseFloat(expectedLineHeight)))
    expect(state.readingStyle.value.measure).toBe(expectedMeasure)
  })

  it('Cân (`md`) là mức mặc định lúc mở lần đầu', async () => {
    const state = await import('../../src/modes/readingState')
    expect(state.currentReadingLevel.value).toBe('md')
  })
})

describe('modes/readingState.ts::setLineHeight — sàn 1.66 lúc chạy', () => {
  it('🔴 `setLineHeight(1.2)` ⇒ giá trị áp lên trang bị ghìm ở 1.66, không phải 1.2', async () => {
    const state = await import('../../src/modes/readingState')
    state.setLineHeight(1.2)
    expect(state.effectiveLineHeight.value).toBe(1.66)
    expect(state.readingStyle.value.lineHeight).toBe('1.66')
  })

  it('giá trị TRÊN sàn đi qua nguyên vẹn, không bị chạm', async () => {
    const state = await import('../../src/modes/readingState')
    state.setLineHeight(2.1)
    expect(state.effectiveLineHeight.value).toBe(2.1)
  })

  it('gọi hàm TRỰC TIẾP cũng bị ghìm — sàn không phải một ràng buộc riêng của thanh trượt', async () => {
    const state = await import('../../src/modes/readingState')
    state.setLineHeight(0)
    expect(state.effectiveLineHeight.value).toBeGreaterThanOrEqual(state.READING_LINE_HEIGHT_FLOOR)
  })
})

describe('modes/readingState.ts::setFontSize — không đụng bề rộng cột', () => {
  it('`setFontSize` đổi cỡ chữ nhưng `readingStyle.measure` (ch) không đổi', async () => {
    const state = await import('../../src/modes/readingState')
    state.setReadingLevel('md')
    const measureBefore = state.readingStyle.value.measure

    state.setFontSize(22)

    expect(state.readingStyle.value.fontSize).toBe('22px')
    expect(state.readingStyle.value.measure).toBe(measureBefore)
  })

  it('`readingStyle.measure` luôn kết thúc bằng `ch` và không bao giờ chứa `px`', async () => {
    const state = await import('../../src/modes/readingState')
    for (const level of ['lg', 'md', 'sm'] as const) {
      state.setReadingLevel(level)
      state.setFontSize(25)
      state.setLineHeight(2)
      expect(state.readingStyle.value.measure.endsWith('ch')).toBe(true)
      expect(state.readingStyle.value.measure).not.toContain('px')
    }
  })
})

describe('modes/readingState.ts::setReadingLevel — xoá tinh chỉnh đang có', () => {
  it('chọn một preset xoá MỌI override đang có, trả về đúng số đã ký', async () => {
    const state = await import('../../src/modes/readingState')
    state.setReadingLevel('md')
    state.setFontSize(99)
    state.setLineHeight(2.3)

    state.setReadingLevel('lg')

    expect(state.readingStyle.value.fontSize).toBe(`${Number.parseFloat(tokens.typography['read-lg'].fontSize)}px`)
    expect(state.readingStyle.value.lineHeight).toBe(String(Number.parseFloat(tokens.typography['read-lg'].lineHeight)))
  })
})

describe('modes/readingState.ts::toggleBilingual — đổi đúng MỘT ô', () => {
  it('đảo `readingBilingual`, không chạm `readingLevel`/`tunerOpen`', async () => {
    const state = await import('../../src/modes/readingState')
    expect(state.readingBilingual.value).toBe(false)

    state.toggleBilingual()
    expect(state.readingBilingual.value).toBe(true)
    expect(state.currentReadingLevel.value).toBe('md')
    expect(state.readingTunerOpen.value).toBe(false)

    state.toggleBilingual()
    expect(state.readingBilingual.value).toBe(false)
  })
})

describe('modes/readingState.ts::toggleTuner', () => {
  it('đảo `readingTunerOpen`', async () => {
    const state = await import('../../src/modes/readingState')
    expect(state.readingTunerOpen.value).toBe(false)
    state.toggleTuner()
    expect(state.readingTunerOpen.value).toBe(true)
    state.toggleTuner()
    expect(state.readingTunerOpen.value).toBe(false)
  })
})

describe('modes/readingState.ts::resetReading — KHÔNG dọn ba ô tuỳ chọn', () => {
  it('🔴 `resetReading()` dọn mọi ô NỘI DUNG, và để nguyên ba ô TYPOGRAPHY', async () => {
    const state = await import('../../src/modes/readingState')
    state.setReadingLevel('sm')
    state.toggleBilingual()
    state.toggleTuner()
    state.setFontSize(20)
    state.setLineHeight(1.9)

    state.resetReading()

    // Ba ô tuỳ chọn ứng dụng KHÔNG bị chạm — đây là tuỳ chọn ứng dụng, không theo Tác phẩm.
    expect(state.currentReadingLevel.value).toBe('sm')
    expect(state.readingBilingual.value).toBe(true)
    expect(state.readingTunerOpen.value).toBe(true)
    expect(state.effectiveFontSize.value).toBe(20)
    expect(state.effectiveLineHeight.value).toBe(1.9)

    // Nhóm nội dung THÌ bị dọn.
    expect(state.readingRun.value).toBeNull()
    expect(state.readingHasLoaded.value).toBe(false)
  })
})

// ═════════════════════════════════════════════════════════════════════════════════
// Hai mệnh đề tìm ra ở lượt rà 2026-08-30.
// ═════════════════════════════════════════════════════════════════════════════════

describe('modes/readingState — dải cỡ chữ ghìm ở CẢ HAI đầu', () => {
  /**
   * 🔴 Cùng lý lẽ với sàn 1.66 của `setLineHeight`: `min`/`max` của `<input type="range">`
   * chỉ canh ĐƯỜNG CHUỘT. Một lời gọi hàm trực tiếp đi vòng qua nó, và bản đầu của
   * `setFontSize` không ghìm gì cả.
   */
  it('`setFontSize` ghìm dưới sàn và trên trần của chính thanh trượt', async () => {
    const state = await import('../../src/modes/readingState')

    state.setFontSize(4)
    expect(state.effectiveFontSize.value).toBe(state.READING_FONT_SIZE_MIN)

    state.setFontSize(999)
    expect(state.effectiveFontSize.value).toBe(state.READING_FONT_SIZE_MAX)

    state.setFontSize(20)
    expect(state.effectiveFontSize.value).toBe(20)
  })
})

describe('src/modes — mọi chỗ dọn nội dung đọc cũng phải dọn MỤC LỤC', () => {
  /**
   * 🔴 **Một phép kiểm KHAI BÁO trên cây nguồn, cố ý — và đây là hạng kiểm mà một ca hành vi
   * không đảm được.** Lỗi thật (bắt ở lượt rà 2026-08-30) không phải *"`resetReadingToc()`
   * chạy sai"* mà là *"`resetReadingToc()` KHÔNG ĐƯỢC GỌI Ở ĐÂU CẢ"* — một hàm export mà chỗ
   * quên gọi vẫn biên dịch sạch, đúng lớp lỗi mà `editorHasLoaded()` đã ghi ra ở
   * `editorPanelState.ts`. Hậu quả: mục lục còn liệt Chương của Tác phẩm CŨ, và `chapter.id`
   * là số nguyên CỤC BỘ theo `project.db` (AD-3) nên nó gần như chắc chắn TRÙNG một Chương có
   * thật của Tác phẩm mới ⇒ mở nhầm Chương, không lỗi nào được ném.
   *
   * ⚠️ Đây KHÔNG chồng lên `check:panel-refs`: cổng đó hỏi *"ô nhớ có đi qua một hàm reset
   * không"*, còn ca này hỏi *"hàm reset có được GỌI ở đúng những chỗ đổi Tác phẩm không"*.
   */
  it('mỗi tệp gọi `resetReading()` cũng gọi `resetReadingToc()`, cùng số lần', async () => {
    const { readFileSync } = await import('node:fs')
    const { resolve } = await import('node:path')
    const files = ['src/modes/libraryChapters.ts', 'src/modes/libraryImport.ts']

    for (const file of files) {
      // ⚠️ `process.cwd()`, KHÔNG `import.meta.url`: vitest chạy tệp này qua một lượt biến
      // đổi làm `import.meta.url` thành `undefined` ở đây (đo 2026-08-30 — `readFileSync` ném
      // `ENOENT … /tests/frontend/undefined`). `cwd` là gốc kho khi `npm run test` chạy.
      const source = readFileSync(resolve(process.cwd(), file), 'utf8')
      const resets = source.match(/(?<![A-Za-z])resetReading\(\)/g) ?? []
      const tocResets = source.match(/(?<![A-Za-z])resetReadingToc\(\)/g) ?? []
      expect(resets.length, `${file} phải có ít nhất một lượt dọn nội dung đọc`).toBeGreaterThan(0)
      expect(tocResets.length, `${file} dọn nội dung ${resets.length} lần nhưng dọn mục lục ${tocResets.length} lần`).toBe(
        resets.length,
      )
    }
  })
})
