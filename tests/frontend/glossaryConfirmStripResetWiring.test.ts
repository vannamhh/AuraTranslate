/**
 * Canh CHỖ NỐI của `resetGlossaryConfirmStrip()` — Story 3.6, rà ba lớp 2026-08-22.
 *
 * ─────────────────────────────────────────────────────────────────────────────
 * 🔴 VÌ SAO TỆP NÀY TỒN TẠI TÁCH KHỎI `glossaryConfirmStrip.test.ts`
 * ─────────────────────────────────────────────────────────────────────────────
 * `glossaryConfirmStrip.test.ts` kiểm `resetGlossaryConfirmStrip()` bằng cách GỌI THẲNG hàm
 * đó — nó chứng minh hàm làm đúng việc, không chứng minh hàm được GỌI ở hai chỗ đúng
 * (`editorPanelState.ts:655` — đổi Chương/Tác phẩm — và `:2032` — gộp/tách segment). Gỡ một
 * trong hai lời gọi thì sổ "Để sau" và mục `current` cũ RÒ qua Chương/lượt gộp khác mà bộ
 * test đó vẫn xanh nguyên. Khuôn chép từ `glossaryMarksRefresh.test.ts:367` (đối chứng "gỡ
 * lời gọi thì ca này ĐỎ") — mỗi ca dưới đây quan sát KẾT QUẢ (dải hỏi lại được sau reset),
 * không một spy đếm số lượt gọi.
 *
 * `resetEditorPanel()` là hàm ĐỒNG BỘ, xuất công khai, không cần mount Vue hay dàn dựng vòng
 * IPC nào — gọi thẳng nó là đủ. `applyRegroup()` (chỗ nối THỨ HAI) là hàm RIÊNG TƯ, chỉ chạm
 * được qua `mergeCurrentSegment()` — khuôn chép từ `editorRegroupNotice.test.ts` (mock
 * `config/segment.ts`, `ensureSegmentsLoaded()` + `setEditorCaret()` + `mergeCurrentSegment()`).
 */
import { beforeEach, describe, expect, it, vi } from 'vitest'
import { FIXTURE_SEGMENTS, readFixture, recordSave, resetRecorder } from './support/segmentFixture'
import type { ChapterSegment } from '../../src/config/segment'
import type { GlossarySegmentSource } from '../../src/panels/glossaryMarksMap'
import type { GlossaryMark } from '../../src/config/glossary'

/** Hàng mới mà một lượt gộp trả về — hình dạng dây, cùng khuôn `editorRegroupNotice.test.ts`. */
const HANG_GOP: ChapterSegment = {
  id: 14,
  ord: 1,
  source_text: '一。二。',
  target_text: 'Câu gộp.',
  is_paragraph_end: true,
  retired_at: null,
  status: 'draft',
  is_omitted: false,
  is_target_paragraph_end: true,
}

const ketQuaGop: {
  value: { outcome: { retired: ChapterSegment[]; new_segments: ChapterSegment[] } | null; error: unknown }
} = { value: { outcome: null, error: null } }

async function mergeGia() {
  return ketQuaGop.value
}

// ⚠️ `vi.mock` HOIST lên đầu tệp — cùng khuôn `editorRegroupNotice.test.ts`.
vi.mock('../../src/config/segment', async (importOriginal) => {
  const actual = await importOriginal<typeof import('../../src/config/segment')>()
  return {
    ...actual,
    readOpenChapterSegments: readFixture,
    saveSegmentTargets: recordSave,
    mergeSegments: mergeGia,
  }
})

/** Nạp lại CẢ HAI module trong cùng một lượt — cùng lý do `glossaryConfirmStrip.test.ts`. */
async function tuoi() {
  vi.resetModules()
  const editorState = await import('../../src/panels/editorPanelState')
  const confirmStripState = await import('../../src/glossaryConfirmStripState')
  return { editorState, confirmStripState }
}

/** Segment 11 của `FIXTURE_SEGMENTS` mang một span chờ chốt giả — dùng chung cho mọi ca. */
const SEGMENTS: GlossarySegmentSource[] = FIXTURE_SEGMENTS.map((s) => ({ id: s.id, source_text: s.source_text }))
const PENDING_MARK: GlossaryMark = {
  start: 0,
  end: 1,
  tier: 'global',
  is_confirmed: false,
  translation: null,
  id: 7,
  source_term: '一',
  han_viet_suggestion: null,
  han_viet_status: 'not_requested',
}

beforeEach(() => {
  resetRecorder()
  ketQuaGop.value = { outcome: null, error: null }
})

describe('resetEditorPanel() (đổi Chương/Tác phẩm) xoá sổ "Để sau" của dải chốt', () => {
  it('🔴 gỡ `resetGlossaryConfirmStrip()` khỏi `resetEditorPanel()` sẽ làm ca này ĐỎ: source_term đã "Để sau" KHÔNG hỏi lại được sau reset', async () => {
    const { editorState, confirmStripState } = await tuoi()

    confirmStripState.syncGlossaryConfirmStripTarget(11, SEGMENTS, true, [PENDING_MARK])
    expect(confirmStripState.confirmStripIsOpen.value).toBe(true)
    confirmStripState.deferGlossaryConfirmStrip()
    expect(confirmStripState.confirmStripIsOpen.value).toBe(false)

    editorState.resetEditorPanel()

    // 🔴 Mệnh đề trung tâm: cùng câu, cùng mark ⇒ dải PHẢI hỏi lại — sổ "Để sau" có phạm vi
    // ĐÚNG MỘT Chương, và `resetEditorPanel()` là chỗ đổi Chương/Tác phẩm dọn nó. Nếu lời gọi
    // `resetGlossaryConfirmStrip()` bị gỡ khỏi `resetEditorPanel()`, `source_term` "一" vẫn
    // còn trong sổ `deferred` và dòng `expect` dưới đây ĐỎ.
    confirmStripState.syncGlossaryConfirmStripTarget(11, SEGMENTS, true, [PENDING_MARK])
    expect(confirmStripState.confirmStripIsOpen.value).toBe(true)
  })

  it('🔴 gỡ `resetGlossaryConfirmStrip()` khỏi `resetEditorPanel()` sẽ làm ca này ĐỎ: mục ĐANG HỎI của Chương cũ không sống sót qua reset', async () => {
    const { editorState, confirmStripState } = await tuoi()

    confirmStripState.syncGlossaryConfirmStripTarget(11, SEGMENTS, true, [PENDING_MARK])
    expect(confirmStripState.confirmStripIsOpen.value).toBe(true)

    editorState.resetEditorPanel()

    expect(confirmStripState.confirmStripIsOpen.value).toBe(false)
    expect(confirmStripState.confirmStripSourceTerm.value).toBe(null)
  })
})

describe('applyRegroup() (gộp/tách qua mergeCurrentSegment) xoá sổ "Để sau" của dải chốt', () => {
  it('🔴 gỡ `resetGlossaryConfirmStrip()` khỏi lượt gộp/tách sẽ làm ca này ĐỎ: source_term đã "Để sau" KHÔNG hỏi lại được sau khi gộp', async () => {
    const { editorState, confirmStripState } = await tuoi()
    await editorState.ensureSegmentsLoaded()
    editorState.setEditorCaret(12)

    confirmStripState.syncGlossaryConfirmStripTarget(11, SEGMENTS, true, [PENDING_MARK])
    expect(confirmStripState.confirmStripIsOpen.value).toBe(true)
    confirmStripState.deferGlossaryConfirmStrip()
    expect(confirmStripState.confirmStripIsOpen.value).toBe(false)

    ketQuaGop.value = {
      outcome: { retired: FIXTURE_SEGMENTS.slice(0, 2).map((s) => ({ ...s })), new_segments: [HANG_GOP] },
      error: null,
    }
    expect(await editorState.mergeCurrentSegment()).toBe('done')

    // 🔴 Mệnh đề trung tâm: một segment đã về hưu (gộp) không được để lại một mục "Để sau"
    // sống sót — segment 11 (source_term "一" của nó) phải hỏi lại được nếu ai đó (giả định)
    // còn nhắc tới cùng cặp `(segmentId, sourceTerm)` đó.
    confirmStripState.syncGlossaryConfirmStripTarget(11, SEGMENTS, true, [PENDING_MARK])
    expect(confirmStripState.confirmStripIsOpen.value).toBe(true)
  })
})
