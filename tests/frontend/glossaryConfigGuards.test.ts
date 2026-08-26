/**
 * Guard lúc chạy của `src/config/glossary.ts` — cụm D vá (vòng rà Epic 3, 2026-08-26).
 *
 * ─────────────────────────────────────────────────────────────────────────────
 * ⚠️ PHẠM VI — mock `@tauri-apps/api/core` Ở ĐÚNG BIÊN IPC, khuôn `glossaryConfirmStrip.test.ts`
 * ─────────────────────────────────────────────────────────────────────────────
 * KHÔNG mock trọn module `src/config/glossary` (khuôn `glossaryQuickAdd.test.ts`/
 * `glossaryManage.test.ts`): làm vậy thì các hàm `isX` THẬT của tệp không hề chạy, và một ca
 * "mock trọn module là một ca không chạy guard nào" (§Always của spec). Mock ở tầng
 * `invoke()` để bộ guard thật của `config/glossary.ts` chạy qua từng ca.
 *
 * Mỗi mục dưới đây mang MỘT ca dữ liệu HỎNG (guard phải từ chối) VÀ MỘT ca dữ liệu ĐÚNG
 * (guard không được nói oan — §Manual checks "đối chứng chiều ngược").
 */
import { beforeEach, describe, expect, it, vi } from 'vitest'

const mockInvoke = vi.fn()
vi.mock('@tauri-apps/api/core', () => ({ invoke: (...args: unknown[]) => mockInvoke(...args) }))

async function freshAdapter() {
  vi.resetModules()
  mockInvoke.mockReset()
  return import('../../src/config/glossary')
}

/** Một mục `GlossaryQuickAddEntry` hợp lệ trên dây — dùng làm gốc cho các biến thể hỏng. */
function validEntryWire(over: Record<string, unknown> = {}) {
  return {
    tier: 'global',
    id: 7,
    source_term: '慕容',
    translation: 'Mộ Dung',
    note: '',
    category: 'person',
    term_origin: 'manual',
    created_at: '2026-08-24T00:00:00.000Z',
    ...over,
  }
}

beforeEach(() => {
  document.body.innerHTML = ''
})

describe('① lookupGlossaryTerm — QuickAddLookupWire, mục lồng phải qua guard', () => {
  it('Rust trả work_tier_available: null (đúng khuôn hỏng của I/O Matrix) ⇒ error đọc được, 0 lượt truy cập trường trên dữ liệu chưa kiểm', async () => {
    const state = await freshAdapter()
    mockInvoke.mockResolvedValue({ entry: validEntryWire(), work_tier_available: null })

    const result = await state.lookupGlossaryTerm('慕容')

    expect(result.found).toBe('unknown')
    expect(result.found === 'unknown' ? result.error : null).not.toBeNull()
  })

  it('mục lồng (entry) thiếu trường ⇒ error đọc được, KHÔNG ném', async () => {
    const state = await freshAdapter()
    mockInvoke.mockResolvedValue({ entry: { tier: 'global', id: 7 }, work_tier_available: true })

    const result = await state.lookupGlossaryTerm('慕容')

    expect(result.found).toBe('unknown')
  })

  it('đối chứng chiều ngược — dữ liệu ĐÚNG hình dạng ⇒ error === null, hành vi đường thường không đổi', async () => {
    const state = await freshAdapter()
    mockInvoke.mockResolvedValue({ entry: validEntryWire(), work_tier_available: true })

    const result = await state.lookupGlossaryTerm('慕容')

    expect(result.found).toBe('entry')
    expect(result.found === 'entry' ? result.entry.source_term : null).toBe('慕容')
  })

  it('entry: null (chế độ THÊM, không tìm thấy) ⇒ found "none", vẫn hợp lệ', async () => {
    const state = await freshAdapter()
    mockInvoke.mockResolvedValue({ entry: null, work_tier_available: true })

    const result = await state.lookupGlossaryTerm('新')

    expect(result).toEqual({ found: 'none', workTierAvailable: true })
  })
})

describe('② addGlossaryTerm / approveGlossaryCandidate — id trả về phải là số nguyên', () => {
  it('addGlossaryTerm: Rust trả "12" (chuỗi) ⇒ error khác null, id KHÔNG đi tiếp vào state', async () => {
    const state = await freshAdapter()
    mockInvoke.mockResolvedValue('12')

    const result = await state.addGlossaryTerm('global', '新', null, '', 'other')

    expect(result.value).toBeNull()
    expect(result.error).not.toBeNull()
  })

  it('addGlossaryTerm: Rust trả 12.5 (số thực) ⇒ error khác null', async () => {
    const state = await freshAdapter()
    mockInvoke.mockResolvedValue(12.5)

    const result = await state.addGlossaryTerm('global', '新', null, '', 'other')

    expect(result.value).toBeNull()
    expect(result.error).not.toBeNull()
  })

  it('addGlossaryTerm: đối chứng chiều ngược — id số nguyên hợp lệ ⇒ error === null', async () => {
    const state = await freshAdapter()
    mockInvoke.mockResolvedValue(12)

    const result = await state.addGlossaryTerm('global', '新', null, '', 'other')

    expect(result).toEqual({ value: 12, error: null })
  })

  it('approveGlossaryCandidate: Rust trả một số thực ⇒ error khác null', async () => {
    const state = await freshAdapter()
    mockInvoke.mockResolvedValue(12.5)

    const result = await state.approveGlossaryCandidate(1, null, 'other')

    expect(result.value).toBeNull()
    expect(result.error).not.toBeNull()
  })

  it('approveGlossaryCandidate: đối chứng chiều ngược — id số nguyên hợp lệ ⇒ error === null', async () => {
    const state = await freshAdapter()
    mockInvoke.mockResolvedValue(42)

    const result = await state.approveGlossaryCandidate(1, null, 'other')

    expect(result).toEqual({ value: 42, error: null })
  })
})

describe('③ exportGlossaryTier — path phải là chuỗi; null VẪN là "đã huỷ hộp thoại"', () => {
  it('Rust trả 42 (không phải chuỗi) ⇒ outcome "error", KHÔNG phải "done"', async () => {
    const state = await freshAdapter()
    mockInvoke.mockResolvedValue(42)

    const result = await state.exportGlossaryTier('global')

    expect(result.outcome).toBe('error')
  })

  it('⚠️ null VẪN là "đã huỷ hộp thoại" — im lặng có chủ, KHÔNG phải lỗi (phân biệt được huỷ ↔ hỏng)', async () => {
    const state = await freshAdapter()
    mockInvoke.mockResolvedValue(null)

    const result = await state.exportGlossaryTier('global')

    expect(result).toEqual({ outcome: 'cancelled' })
  })

  it('đối chứng chiều ngược — một chuỗi đường dẫn hợp lệ ⇒ outcome "done"', async () => {
    const state = await freshAdapter()
    mockInvoke.mockResolvedValue('/tmp/a.csv')

    const result = await state.exportGlossaryTier('global')

    expect(result).toEqual({ outcome: 'done', path: '/tmp/a.csv' })
  })

  it('🔴 #11 (vòng rà thứ hai) — chuỗi RỖNG không phải một đường dẫn hợp lệ ⇒ outcome "error", KHÔNG "done"', async () => {
    // Đối chứng gỡ-chỗ-nối: đổi điều kiện về lại `typeof path !== 'string'` trần (bỏ vế
    // `|| path === ''`) ⇒ ca này ĐỎ (`outcome` thành `'done'`, `path: ''`).
    const state = await freshAdapter()
    mockInvoke.mockResolvedValue('')

    const result = await state.exportGlossaryTier('global')

    expect(result.outcome).toBe('error')
  })
})

describe('④ confirmGlossaryImport — GlossaryImportSummary, ba trường số phải là số nguyên', () => {
  it('tóm tắt có số thực (inserted: 1.5) ⇒ guard từ chối CẢ bản ghi, error khác null', async () => {
    const state = await freshAdapter()
    mockInvoke.mockResolvedValue({ inserted: 1.5, updated: 2, identical: 0 })

    const result = await state.confirmGlossaryImport({})

    expect(result.summary).toBeNull()
    expect(result.error).not.toBeNull()
  })

  it('đối chứng chiều ngược — ba số nguyên hợp lệ ⇒ error === null', async () => {
    const state = await freshAdapter()
    mockInvoke.mockResolvedValue({ inserted: 1, updated: 2, identical: 0 })

    const result = await state.confirmGlossaryImport({})

    expect(result).toEqual({ summary: { inserted: 1, updated: 2, identical: 0 }, error: null })
  })
})

describe('⑤ isGlossaryImportPreview (qua openGlossaryImportPreview) — bốn trường số phải là số nguyên', () => {
  function previewWire(over: Record<string, unknown> = {}) {
    return {
      file_name: 'a.csv',
      tier: 'global',
      row_count: 3,
      recognized_column_count: 2,
      ignored_columns: [],
      term_origin_column_present: false,
      new_count: 1,
      identical_count: 0,
      conflicts: [],
      ...over,
    }
  }

  it('row_count: 3.7 (số thực) ⇒ isGlossaryImportPreview trả false, outcome "error"', async () => {
    const state = await freshAdapter()
    mockInvoke.mockResolvedValue(previewWire({ row_count: 3.7 }))

    const result = await state.openGlossaryImportPreview('global')

    expect(result.outcome).toBe('error')
  })

  it('đối chứng chiều ngược — bốn trường số nguyên hợp lệ ⇒ outcome "loaded"', async () => {
    const state = await freshAdapter()
    mockInvoke.mockResolvedValue(previewWire())

    const result = await state.openGlossaryImportPreview('global')

    expect(result.outcome).toBe('loaded')
  })
})

describe('⑥ isGlossaryEntry (qua listGlossaryEntries) — bất biến chéo trường is_shadowed ⇒ tier === "global"', () => {
  function entryWire(over: Record<string, unknown> = {}) {
    return {
      tier: 'global',
      id: 1,
      source_term: '青丘',
      translation: 'Thanh Khâu',
      note: '',
      category: 'place',
      term_origin: 'manual',
      created_at: '2026-08-24T00:00:00.000Z',
      is_shadowed: false,
      ...over,
    }
  }

  it('is_shadowed: true nhưng tier: "work" (bất biến khai trong doc-comment bị phá) ⇒ isGlossaryEntry trả false, error khác null', async () => {
    const state = await freshAdapter()
    mockInvoke.mockResolvedValue([entryWire({ tier: 'work', is_shadowed: true })])

    const result = await state.listGlossaryEntries()

    expect(result.entries).toBeNull()
    expect(result.error).not.toBeNull()
  })

  it('đối chứng chiều ngược — is_shadowed: true VÀ tier: "global" (đúng bất biến) ⇒ error === null', async () => {
    const state = await freshAdapter()
    mockInvoke.mockResolvedValue([entryWire({ tier: 'global', is_shadowed: true })])

    const result = await state.listGlossaryEntries()

    expect(result.entries).toEqual([entryWire({ tier: 'global', is_shadowed: true })])
    expect(result.error).toBeNull()
  })

  it('đối chứng chiều ngược — is_shadowed: false ở bất kỳ tầng nào ⇒ error === null', async () => {
    const state = await freshAdapter()
    mockInvoke.mockResolvedValue([entryWire({ tier: 'work', is_shadowed: false })])

    const result = await state.listGlossaryEntries()

    expect(result.error).toBeNull()
  })
})
