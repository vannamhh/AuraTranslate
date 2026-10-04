/**
 * Runtime guards of `src/config/tm.ts`, mocked at the `invoke()` boundary so the real `isX`
 * functions run. Every payload kind has one malformed case (the guard must refuse) and one
 * well-formed case (the guard must not accuse).
 */
import { beforeEach, describe, expect, it, vi } from 'vitest'

const mockInvoke = vi.fn()
vi.mock('@tauri-apps/api/core', () => ({
  invoke: (...args: unknown[]) => mockInvoke(...args),
}))

async function freshAdapter() {
  vi.resetModules()
  mockInvoke.mockReset()
  vi.spyOn(console, 'error').mockImplementation(() => undefined)
  return import('../../src/config/tm')
}

function rowWire(over: Record<string, unknown> = {}) {
  return {
    tier: 'work',
    unit_id: 1,
    copies: [{ tier: 'work', unit_id: 1 }],
    target_text: 'Xin chào',
    translation_origin: 'self',
    side: 'mine',
    created_at: '2026-10-01T08:00:00.000Z',
    ...over,
  }
}

function listingWire(over: Record<string, unknown> = {}) {
  return {
    work_open: true,
    tm_empty: false,
    health: [
      { translation_origin: 'self', count: 1 },
      { translation_origin: 'other', count: 0 },
      { translation_origin: 'bilingual_import', count: 0 },
    ],
    total_pairs: 1,
    total_groups: 1,
    groups: [{ source_text: '你好', distinct_targets: 1, rows: [rowWire()] }],
    ...over,
  }
}

function pairWire(over: Record<string, unknown> = {}) {
  const { copies: _copies, ...rest } = rowWire()
  return { ...rest, source_text: '你好', ...over }
}

const COPIES = [{ tier: 'work' as const, unit_id: 1 }]

beforeEach(() => {
  document.body.innerHTML = ''
})

describe('tmListPairs', () => {
  it('sends origin, tier and search, and accepts a well-formed listing', async () => {
    const tm = await freshAdapter()
    mockInvoke.mockResolvedValue(listingWire())
    const result = await tm.tmListPairs('all', 'both', 'x')
    expect(mockInvoke).toHaveBeenCalledWith('tm_list_pairs', {
      origin: 'all',
      tier: 'both',
      search: 'x',
    })
    expect(result.error).toBeNull()
    expect(result.listing?.groups[0].rows[0].copies).toEqual([{ tier: 'work', unit_id: 1 }])
  })

  it.each([
    [
      'row without copies',
      {
        groups: [
          {
            source_text: '你好',
            distinct_targets: 1,
            rows: [rowWire({ copies: undefined })],
          },
        ],
      },
    ],
    [
      'row with empty copies',
      {
        groups: [
          {
            source_text: '你好',
            distinct_targets: 1,
            rows: [rowWire({ copies: [] })],
          },
        ],
      },
    ],
    [
      'copy with a bad tier',
      {
        groups: [
          {
            source_text: '你好',
            distinct_targets: 1,
            rows: [rowWire({ copies: [{ tier: 'x', unit_id: 1 }] })],
          },
        ],
      },
    ],
    [
      'unknown origin',
      {
        groups: [
          {
            source_text: '你好',
            distinct_targets: 1,
            rows: [rowWire({ translation_origin: 'mine' })],
          },
        ],
      },
    ],
    ['health entry without count', { health: [{ translation_origin: 'self' }] }],
    ['work_open as a string', { work_open: 'yes' }],
  ])('refuses %s with a readable error and no throw', async (_name, over) => {
    const tm = await freshAdapter()
    mockInvoke.mockResolvedValue(listingWire(over))
    const result = await tm.tmListPairs('all', 'both', '')
    expect(result.listing).toBeNull()
    expect(result.error?.code).toBe('ipc.unknown')
  })

  it('an IpcError from Rust comes back as that error', async () => {
    const tm = await freshAdapter()
    const err = {
      code: 'work.none_open',
      message_key: 'err.unknown',
      params: {},
      retryable: false,
    }
    mockInvoke.mockRejectedValue(err)
    expect((await tm.tmListPairs('all', 'work', '')).error).toEqual(err)
  })
})

describe('write wrappers', () => {
  it('update sends copies, sourceText, expectedTarget and targetText and accepts a stored pair', async () => {
    const tm = await freshAdapter()
    mockInvoke.mockResolvedValue(pairWire())
    const result = await tm.tmUpdatePairTarget(COPIES, '你好', 'Xin chào', 'Chào')
    expect(mockInvoke).toHaveBeenCalledWith('tm_update_pair_target', {
      copies: COPIES,
      sourceText: '你好',
      expectedTarget: 'Xin chào',
      targetText: 'Chào',
    })
    expect(result.pair?.source_text).toBe('你好')
  })

  it('update refuses a pair without source_text', async () => {
    const tm = await freshAdapter()
    mockInvoke.mockResolvedValue(pairWire({ source_text: undefined }))
    const result = await tm.tmUpdatePairTarget(COPIES, '你好', 'Xin chào', 'Chào')
    expect(result.pair).toBeNull()
    expect(result.error?.code).toBe('ipc.unknown')
  })

  it('delete and push send copies, sourceText and expectedTarget', async () => {
    const tm = await freshAdapter()
    mockInvoke.mockResolvedValue(null)
    expect((await tm.tmDeletePair(COPIES, '你好', 'Xin chào')).ok).toBe(true)
    expect(mockInvoke).toHaveBeenLastCalledWith('tm_delete_pair', {
      copies: COPIES,
      sourceText: '你好',
      expectedTarget: 'Xin chào',
    })

    mockInvoke.mockResolvedValue(pairWire({ tier: 'global', unit_id: 9 }))
    expect((await tm.tmPushPairToGlobal(COPIES, '你好', 'Xin chào')).pair?.unit_id).toBe(9)
    expect(mockInvoke).toHaveBeenLastCalledWith('tm_push_pair_to_global', {
      copies: COPIES,
      sourceText: '你好',
      expectedTarget: 'Xin chào',
    })
  })

  it('push refuses a payload with a bad tier', async () => {
    const tm = await freshAdapter()
    mockInvoke.mockResolvedValue(pairWire({ tier: 'nowhere' }))
    expect((await tm.tmPushPairToGlobal(COPIES, '你好', 'Xin chào')).pair).toBeNull()
  })

  it('delete-others refuses counts that are not numbers and accepts the two counts', async () => {
    const tm = await freshAdapter()
    mockInvoke.mockResolvedValue({ deleted_work: '2', deleted_global: 0 })
    expect((await tm.tmDeleteOthers('both')).outcome).toBeNull()

    mockInvoke.mockResolvedValue({ deleted_work: 2, deleted_global: 0 })
    const ok = await tm.tmDeleteOthers('work')
    expect(mockInvoke).toHaveBeenLastCalledWith('tm_delete_others', {
      tier: 'work',
    })
    expect(ok.outcome).toEqual({ deleted_work: 2, deleted_global: 0 })
  })
})

describe('TMX exchange wrappers', () => {
  it('export sends the tier; null is a cancelled dialog; an empty path is refused', async () => {
    const a = await freshAdapter()
    mockInvoke.mockResolvedValue('/tmp/tm_work.tmx')
    expect(await a.tmExportTier('work')).toEqual({ outcome: 'done', path: '/tmp/tm_work.tmx' })
    expect(mockInvoke).toHaveBeenCalledWith('tm_export_tier', { tier: 'work' })

    mockInvoke.mockResolvedValue(null)
    expect(await a.tmExportTier('global')).toEqual({ outcome: 'cancelled' })

    mockInvoke.mockResolvedValue('')
    expect((await a.tmExportTier('global')).outcome).toBe('error')
  })

  it('open-preview sends the tier and accepts only a well-formed preview', async () => {
    const a = await freshAdapter()
    const wire = { file_name: 'a.tmx', tier: 'global', unit_count: 3, new_count: 1, already_count: 1, skipped_count: 1 }
    mockInvoke.mockResolvedValue(wire)
    expect(await a.tmOpenImportPreview('global')).toEqual({ outcome: 'loaded', preview: wire })
    expect(mockInvoke).toHaveBeenCalledWith('tm_open_import_preview', { tier: 'global' })

    mockInvoke.mockResolvedValue(null)
    expect(await a.tmOpenImportPreview('global')).toEqual({ outcome: 'cancelled' })

    mockInvoke.mockResolvedValue({ ...wire, new_count: 'x' })
    expect((await a.tmOpenImportPreview('global')).outcome).toBe('error')
    mockInvoke.mockResolvedValue({ ...wire, tier: 'both' })
    expect((await a.tmOpenImportPreview('global')).outcome).toBe('error')
  })

  it('confirm and cancel take no arguments; confirm refuses a summary without counts', async () => {
    const a = await freshAdapter()
    mockInvoke.mockResolvedValue({ inserted: 2, already_count: 1 })
    expect(await a.tmConfirmImport()).toEqual({ summary: { inserted: 2, already_count: 1 }, error: null })
    expect(mockInvoke).toHaveBeenCalledWith('tm_confirm_import')

    mockInvoke.mockResolvedValue({ inserted: 2 })
    expect((await a.tmConfirmImport()).error).not.toBeNull()

    mockInvoke.mockResolvedValue(null)
    expect(await a.tmCancelImport()).toEqual({ ok: true, error: null })
    expect(mockInvoke).toHaveBeenLastCalledWith('tm_cancel_import')
  })

  it('an IpcError from Rust comes back as that error on every wrapper', async () => {
    const a = await freshAdapter()
    const err = { code: 'tm.tmx_malformed', message_key: 'err.unknown', params: { line: '4' }, retryable: false }
    mockInvoke.mockRejectedValue(err)
    expect(await a.tmExportTier('work')).toEqual({ outcome: 'error', error: err })
    expect(await a.tmOpenImportPreview('work')).toEqual({ outcome: 'error', error: err })
    expect((await a.tmConfirmImport()).error).toEqual(err)
    expect((await a.tmCancelImport()).error).toEqual(err)
  })
})
