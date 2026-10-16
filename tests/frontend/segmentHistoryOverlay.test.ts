import { afterEach, describe, expect, it, vi } from 'vitest'
import { mount } from '@vue/test-utils'
import { readFileSync } from 'node:fs'

import { readFixture, recordSave, resetRecorder } from './support/segmentFixture'

type Origin = 'self' | 'other' | 'bilingual_import' | null
type Span = { kind: 'equal' | 'delete' | 'insert'; text: string }

const served: {
  current: number | null
  versions: { id: number; text: string; origin: Origin; spans: Span[] }[]
} = { current: null, versions: [] }

vi.mock('../../src/config/segment', async (importOriginal) => {
  const actual = await importOriginal<typeof import('../../src/config/segment')>()
  return {
    ...actual,
    readOpenChapterSegments: readFixture,
    saveSegmentTargets: recordSave,
    readSegmentHistory: async () => ({
      history: {
        current_version_id: served.current,
        source_text: 'Nguyen van cua cau.',
        versions: served.versions.map((v, index) => ({
          id: v.id,
          segment_id: 11,
          target_text: v.text,
          created_at: `2026-10-16T0${9 - index}:00:00.000Z`,
          translation_origin: v.origin,
          spans: v.spans,
        })),
      },
      error: null,
    }),
  }
})

const eq = (text: string): Span[] => [{ kind: 'equal', text }]

async function setup() {
  resetRecorder()
  const editor = await import('../../src/panels/editorPanelState')
  await editor.ensureSegmentsLoaded()
  const history = await import('../../src/panels/segmentHistoryState')
  const commands = await import('../../src/commands')
  commands.installCommands({
    closeSegmentHistory: history.closeSegmentHistory,
    toggleHistoryCompare: history.toggleHistoryCompare,
    restoreAimedVersion: history.restoreAimedVersion,
  } as unknown as Parameters<typeof commands.installCommands>[0])
  const { default: Overlay } = await import('../../src/SegmentHistoryOverlay.vue')
  editor.setEditorCaret(11)
  history.openSegmentHistory()
  await vi.waitFor(() => expect(history.historyPending.value).toBe(false))
  const w = mount(Overlay, { attachTo: document.body })
  await w.vm.$nextTick()
  return { w, history }
}

afterEach(() => {
  document.body.innerHTML = ''
  vi.resetModules()
})

describe('SegmentHistoryOverlay hai ngăn', () => {
  it('danh sách bên trái, chi tiết bên phải với nguyên văn và toàn văn phiên bản đang nhắm', async () => {
    served.current = 203
    served.versions = [
      { id: 203, text: 'Ban A.', origin: 'other', spans: eq('Ban A.') },
      {
        id: 202,
        text: 'Ban B.',
        origin: 'self',
        spans: [
          { kind: 'delete', text: 'B.' },
          { kind: 'insert', text: 'A.' },
        ],
      },
      { id: 201, text: 'Ban A.', origin: null, spans: eq('Ban A.') },
    ]
    const { w } = await setup()

    expect(w.findAll('[data-history-row]')).toHaveLength(3)
    expect(w.find('[data-history-detail]').exists()).toBe(true)
    expect(w.find('[data-history-source]').text()).toBe('Nguyen van cua cau.')
    expect(w.find('[data-history-full]').text()).toBe('Ban A.')
    expect(w.find('[data-history-diff]').exists()).toBe(false)
    expect(w.findAll('[data-history-row]').map((r) => r.attributes('data-history-origin'))).toEqual(
      ['other', 'self', 'unknown'],
    )
  })

  it('hai hàng trùng chữ: chỉ hàng có `current_version_id` mang nhãn đang dùng, kể cả khi đó là hàng cũ hơn', async () => {
    served.current = 201
    served.versions = [
      { id: 203, text: 'Ban A.', origin: 'self', spans: eq('Ban A.') },
      { id: 202, text: 'Ban B.', origin: 'self', spans: eq('Ban B.') },
      { id: 201, text: 'Ban A.', origin: 'self', spans: eq('Ban A.') },
    ]
    const { w } = await setup()

    expect(
      w.findAll('[data-history-row]').map((r) => r.attributes('data-history-current')),
    ).toEqual(['false', 'false', 'true'])
  })

  it('không phiên bản nào đang dùng ⇒ không hàng nào có nhãn và có dòng "chưa ký"', async () => {
    served.current = null
    served.versions = [{ id: 201, text: 'Ban A.', origin: 'self', spans: eq('Ban A.') }]
    const { w } = await setup()

    expect(w.findAll('[data-history-current="true"]')).toHaveLength(0)
    expect(w.find('[data-history-unsigned]').exists()).toBe(true)
  })

  it('bấm "So với bản đang dùng" bật diff, bấm lần nữa trở lại toàn văn', async () => {
    served.current = 203
    served.versions = [
      { id: 203, text: 'Ban A.', origin: 'self', spans: eq('Ban A.') },
      {
        id: 202,
        text: 'Ban B.',
        origin: 'self',
        spans: [
          { kind: 'delete', text: 'B.' },
          { kind: 'insert', text: 'A.' },
        ],
      },
    ]
    const { w, history } = await setup()
    history.aimHistoryVersion(202)
    await w.vm.$nextTick()
    const compare = w
      .findAll('button.hist-act')
      .find(
        (b) => b.text() !== '' && b.attributes('disabled') === undefined && b.text().includes('So'),
      )
    expect(compare).toBeDefined()

    await compare!.trigger('click')
    expect(w.find('[data-history-diff]').exists()).toBe(true)
    expect(w.find('[data-history-diff] del').text()).toBe('B.')
    expect(w.find('[data-history-diff] ins').text()).toBe('A.')
    expect(w.find('[data-history-full]').exists()).toBe(false)

    await compare!.trigger('click')
    expect(w.find('[data-history-full]').text()).toBe('Ban B.')
    expect(w.find('[data-history-diff]').exists()).toBe(false)
  })

  it('↑↓ trên lớp phủ đổi hàng nhắm và chi tiết theo', async () => {
    served.current = 202
    served.versions = [
      { id: 202, text: 'Ban B.', origin: 'self', spans: eq('Ban B.') },
      { id: 201, text: 'Ban A.', origin: 'self', spans: eq('Ban A.') },
    ]
    const { w } = await setup()

    await w.find('.hist-scrim').trigger('keydown', { key: 'ArrowDown' })
    expect(w.find('[data-history-full]').text()).toBe('Ban A.')
    await w.find('.hist-scrim').trigger('keydown', { key: 'ArrowUp' })
    expect(w.find('[data-history-full]').text()).toBe('Ban B.')
  })

  it('không dùng `v-html` ở lớp phủ và ở `ReviewSpans`', () => {
    for (const file of ['src/SegmentHistoryOverlay.vue', 'src/panels/ReviewSpans.vue']) {
      expect(readFileSync(file, 'utf8')).not.toMatch(/v-html/)
    }
  })
})
