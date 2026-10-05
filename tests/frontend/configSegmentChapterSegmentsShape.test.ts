/**
 * `config/segment.ts::readOpenChapterSegments` — mọi adapter trust-only của tệp này kiểm hình
 * dạng **lúc chạy**, cùng mức chặt `isSegmentVersionArray` (trường có mặt, đúng kiểu, cho MỖI
 * hàng — không chỉ `Array.isArray` trần). `isChapterSegments` kiểm `segments` là một mảng
 * nhưng không đọc field nào của TỪNG hàng, nên một hàng thiếu trường (ví dụ Rust quên gửi
 * `translation_origin`) đi qua sạch thành một `ChapterSegment` giả — đúng lớp lỗi mà
 * `isSegmentVersionArray`'s doc-comment ghi lại đã xảy ra thật một lần.
 */
import { describe, expect, it, vi } from 'vitest'

const invokeMock = vi.fn()

vi.mock('@tauri-apps/api/core', () => ({ invoke: (...args: unknown[]) => invokeMock(...args) }))

describe('readOpenChapterSegments — một hàng segment thiếu trường phải trả lỗi, không lọt qua', () => {
  it('hàng thiếu `translation_origin` ⇒ `loaded: null`, `error` khác null', async () => {
    const { readOpenChapterSegments } = await import('../../src/config/segment')

    invokeMock.mockResolvedValueOnce({
      chapter_id: 7,
      caret_segment_id: null,
      assets: [],
      assets_dir: '',
      tm_filled_segment_ids: [],
      tm_prefill: { kind: 'ran' },
      segments: [
        {
          id: 1,
          ord: 1,
          source_text: '一。',
          target_text: 'Một.',
          is_paragraph_end: false,
          retired_at: null,
          status: 'draft',
          is_omitted: false,
          is_target_paragraph_end: false,
          role: null,
          // `translation_origin` THIẾU — đúng lớp lỗi Rust quên gửi một trường.
        },
      ],
    })

    const { loaded, error } = await readOpenChapterSegments()

    expect(loaded).toBeNull()
    expect(error).not.toBeNull()
  })

  it('hình dạng ĐÚNG ⇒ đi qua bình thường, không bị chặn nhầm', async () => {
    const { readOpenChapterSegments } = await import('../../src/config/segment')

    invokeMock.mockResolvedValueOnce({
      chapter_id: 7,
      caret_segment_id: null,
      assets: [],
      assets_dir: '',
      tm_filled_segment_ids: [],
      tm_prefill: { kind: 'ran' },
      segments: [
        {
          id: 1,
          ord: 1,
          source_text: '一。',
          target_text: 'Một.',
          is_paragraph_end: false,
          retired_at: null,
          status: 'draft',
          is_omitted: false,
          is_target_paragraph_end: false,
          role: null,
          translation_origin: '',
        },
      ],
    })

    const { loaded, error } = await readOpenChapterSegments()

    expect(error).toBeNull()
    expect(loaded?.segments).toHaveLength(1)
  })
})

const VALID_ROW = {
  id: 1,
  ord: 1,
  source_text: '一。',
  target_text: 'Một.',
  is_paragraph_end: false,
  retired_at: null,
  status: 'draft',
  is_omitted: false,
  is_target_paragraph_end: false,
  role: null,
  translation_origin: '',
}

describe('mergeSegments — một hàng của `retired`/`new_segments` thiếu trường phải trả lỗi', () => {
  it('`retired` mang một hàng thiếu `status` ⇒ `outcome: null`, `error` khác null', async () => {
    const { mergeSegments } = await import('../../src/config/segment')

    const { status: _status, ...rowMissingStatus } = VALID_ROW
    invokeMock.mockResolvedValueOnce({
      retired: [rowMissingStatus],
      new_segments: [VALID_ROW],
    })

    const { outcome, error } = await mergeSegments(2)

    expect(outcome).toBeNull()
    expect(error).not.toBeNull()
  })

  it('hình dạng ĐÚNG ở cả hai mảng ⇒ đi qua bình thường', async () => {
    const { mergeSegments } = await import('../../src/config/segment')

    invokeMock.mockResolvedValueOnce({ retired: [VALID_ROW], new_segments: [VALID_ROW] })

    const { outcome, error } = await mergeSegments(2)

    expect(error).toBeNull()
    expect(outcome?.retired).toHaveLength(1)
  })
})

describe('readOpenChapterSegments — `tm_prefill` is a closed wire type', () => {
  const base = { chapter_id: 7, caret_segment_id: null, assets: [], assets_dir: '', tm_filled_segment_ids: [], segments: [] }

  it.each([
    ['absent', undefined],
    ['an unknown kind', { kind: 'maybe' }],
    ['skipped without a code', { kind: 'skipped' }],
    ['a bare string', 'ran'],
  ])('%s => loaded null', async (_name, tmPrefill) => {
    const { readOpenChapterSegments } = await import('../../src/config/segment')
    invokeMock.mockResolvedValueOnce({ ...base, tm_prefill: tmPrefill })
    vi.spyOn(console, 'error').mockImplementationOnce(() => {})

    const { loaded, error } = await readOpenChapterSegments()

    expect(loaded).toBeNull()
    expect(error).not.toBeNull()
  })

  it.each([{ kind: 'ran' }, { kind: 'not_asked' }, { kind: 'skipped', code: 'store.open_failed' }])(
    '%j passes',
    async (tmPrefill) => {
      const { readOpenChapterSegments } = await import('../../src/config/segment')
      invokeMock.mockResolvedValueOnce({ ...base, tm_prefill: tmPrefill })

      const { loaded } = await readOpenChapterSegments()

      expect(loaded?.tm_prefill).toEqual(tmPrefill)
    },
  )

  it('sends prefill true by default and false when asked', async () => {
    const { readOpenChapterSegments } = await import('../../src/config/segment')
    invokeMock.mockResolvedValue({ ...base, tm_prefill: { kind: 'ran' } })

    await readOpenChapterSegments()
    await readOpenChapterSegments({ prefill: false })

    const calls = invokeMock.mock.calls.slice(-2)
    expect(calls[0][1]).toEqual({ prefill: true })
    expect(calls[1][1]).toEqual({ prefill: false })
  })
})

