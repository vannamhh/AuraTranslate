import { beforeEach, describe, expect, it, vi } from 'vitest'
import { exportChooseFile, exportDocxOneBlock, exportDocxTwoColumn, exportText } from '../../src/config/export'

const mockInvoke = vi.hoisted(() => vi.fn())
vi.mock('@tauri-apps/api/core', () => ({ invoke: mockInvoke }))

const scope = { kind: 'chapters' as const, chapter_ids: [7] }
const options = { number_segments: false, glossary_appendix: false, mark_unconfirmed: false, curly_quotes: false }
const file = { path: '/tmp/custom', chapter_count: 1, segment_count: 1, image_count: 0, images_skipped_missing_link: 0, images_dir: null }

beforeEach(() => {
  mockInvoke.mockReset()
})

describe('Save As IPC adapter', () => {
  it.each(['docx_two_column', 'docx_one_block', 'markdown', 'plain'] as const)('passes scope and %s to Rust without deriving the destination', async (format) => {
    mockInvoke.mockResolvedValue('/tmp/custom')
    expect(await exportChooseFile(scope, format)).toEqual({ outcome: 'picked', path: '/tmp/custom' })
    expect(mockInvoke).toHaveBeenCalledWith('export_choose_file', { scope, format })
  })

  it('treats cancellation as a quiet result', async () => {
    mockInvoke.mockResolvedValue(null)
    expect(await exportChooseFile(scope, 'plain')).toEqual({ outcome: 'cancelled' })
  })

  it.each([42, {}, [], ''])('rejects malformed destination %j', async (value) => {
    mockInvoke.mockResolvedValue(value)
    expect(await exportChooseFile(scope, 'plain')).toMatchObject({ outcome: 'error', error: { code: 'ipc.unknown' } })
  })

  it('retains structured IPC failure for the screen', async () => {
    const error = { code: 'export.write_failed', message_key: 'err.export.write_failed', params: {}, retryable: true }
    mockInvoke.mockRejectedValue(error)
    expect(await exportChooseFile(scope, 'plain')).toEqual({ outcome: 'error', error })
  })

  it('sends the exact destination through every writer adapter', async () => {
    mockInvoke.mockResolvedValue(file)
    await exportDocxTwoColumn(scope, 'file', file.path, false, options)
    await exportDocxOneBlock(scope, 'file', file.path, false, options)
    await exportText(scope, 'file', 'markdown', file.path, false, options)
    await exportText(scope, 'file', 'plain', file.path, false, options)
    expect(mockInvoke.mock.calls).toEqual([
      ['export_docx_two_column', { scope, imageMode: 'file', destination: file.path, attribution: false, options }],
      ['export_docx_one_block', { scope, imageMode: 'file', destination: file.path, attribution: false, options }],
      ['export_text', { scope, imageMode: 'file', format: 'markdown', destination: file.path, attribution: false, options }],
      ['export_text', { scope, imageMode: 'file', format: 'plain', destination: file.path, attribution: false, options }],
    ])
  })
})
