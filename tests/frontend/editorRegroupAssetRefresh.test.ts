/**
 * `editorPanelState.ts::refreshChapterAssetsAfterRegroup` — a merge/split re-reads
 * `chapterAssets`/`assetsDir` through the same `readOpenChapterSegments()` IPC that
 * `ensureSegmentsLoaded` uses, right after `applyRegroup` patches the segment array.
 *
 * No existing test file called this function or read `editorChapterAssets` before this one —
 * confirmed by `grep -rl editorChapterAssets tests/frontend`. Same mocking scaffold as
 * `editorRegroupGuards.test.ts` (IPC boundary, not a test-only export).
 */
import { beforeEach, describe, expect, it, vi } from 'vitest'
import { FIXTURE_SEGMENTS, readFixture, recordSave, resetRecorder } from './support/segmentFixture'
import type { ChapterAsset, ChapterSegment } from '../../src/config/segment'
import type { IpcError } from '../../src/i18n'

const CHUONG_CUA_SEGMENT = 900

const HANG_MOI: ChapterSegment = {
  id: 14,
  ord: 1,
  source_text: '一。二。',
  target_text: 'Câu gộp.',
  is_paragraph_end: true,
  retired_at: null,
  status: 'draft',
  is_omitted: false,
  is_target_paragraph_end: true,
  role: null,
  translation_origin: '',
}

const ANH_SAU_GOP: ChapterAsset = {
  asset_id: 1,
  file_name: 'sau-gop.png',
  source_url: null,
  after_segment_id: HANG_MOI.id,
  alt_text: null,
  caption_text: null,
}

const ANH_TRUOC_GOP: ChapterAsset = {
  asset_id: 2,
  file_name: 'truoc-gop.png',
  source_url: null,
  after_segment_id: null,
  alt_text: null,
  caption_text: null,
}

const LOI_GIA: IpcError = { code: 'unknown', message_key: 'err.unknown', params: {}, retryable: false }

const ketQuaGop: {
  value: { outcome: { retired: ChapterSegment[]; new_segments: ChapterSegment[] } | null; error: unknown }
} = { value: { outcome: null, error: null } }

async function mergeGia() {
  return ketQuaGop.value
}

/** Lượt gọi ĐẦU (nạp Chương) khai ảnh của [`anhLanDau`]; lượt gọi SAU (do regroup) khai MỘT ảnh
 * MỚI, trừ khi [`ketQuaLanHai`] bị đặt `'loi'` -- chỉ có cách phân biệt "gọi lại IPC" với "tái
 * dùng ảnh chụp cũ" nếu lượt hai thật sự đổi giá trị. */
let soLuotGoiDocSegment = 0
let anhLanDau: ChapterAsset[] = []
let ketQuaLanHai: 'anh' | 'loi' = 'anh'
async function docSegmentGia() {
  soLuotGoiDocSegment += 1
  if (soLuotGoiDocSegment === 1) {
    return {
      loaded: {
        chapter_id: CHUONG_CUA_SEGMENT,
        segments: FIXTURE_SEGMENTS.map((s) => ({ ...s })),
        assets: anhLanDau,
        assets_dir: anhLanDau.length > 0 ? '/tac-pham/assets-cu' : '',
      },
      error: null,
    }
  }
  if (ketQuaLanHai === 'loi') return { loaded: null, error: LOI_GIA }
  return {
    loaded: {
      chapter_id: CHUONG_CUA_SEGMENT,
      segments: FIXTURE_SEGMENTS.map((s) => ({ ...s })),
      assets: [ANH_SAU_GOP],
      assets_dir: '/tac-pham/assets',
    },
    error: null,
  }
}

async function docNguyenVanGia() {
  return {
    chapter: { chapter_id: CHUONG_CUA_SEGMENT, source_text: 'khong dung o day', source_lang: 'zh' },
    error: null,
  }
}

async function traDauGia() {
  return { marks: [], error: null }
}

// ⚠️ `vi.mock` HOIST lên đầu tệp; đường dẫn phân giải tương đối với TỆP NÀY.
vi.mock('../../src/config/segment', async (importOriginal) => {
  const actual = await importOriginal<typeof import('../../src/config/segment')>()
  return {
    ...actual,
    readOpenChapterSegments: docSegmentGia,
    saveSegmentTargets: recordSave,
    mergeSegments: mergeGia,
  }
})
vi.mock('../../src/config/chapter', async (importOriginal) => {
  const actual = await importOriginal<typeof import('../../src/config/chapter')>()
  return { ...actual, readOpenChapter: docNguyenVanGia }
})
vi.mock('../../src/config/glossary', async (importOriginal) => {
  const actual = await importOriginal<typeof import('../../src/config/glossary')>()
  return { ...actual, glossaryMarksForChapter: traDauGia }
})

async function tuoi() {
  vi.resetModules()
  soLuotGoiDocSegment = 0
  const editorState = await import('../../src/panels/editorPanelState')
  const sourceState = await import('../../src/panels/sourcePanelState')
  await editorState.ensureSegmentsLoaded()
  await sourceState.ensureChapterLoaded()
  return { editorState, sourceState }
}

beforeEach(() => {
  resetRecorder()
  readFixture
  ketQuaGop.value = { outcome: null, error: null }
  soLuotGoiDocSegment = 0
  anhLanDau = []
  ketQuaLanHai = 'anh'
})

describe('applyRegroup — ảnh chụp Chương nạp lại sau một lượt gộp/tách', () => {
  it('sau khi gộp hai segment, editorChapterAssets đọc ảnh MỚI (từ lượt IPC thứ hai), không kẹt ở [] của lượt nạp đầu', async () => {
    const { editorState } = await tuoi()
    expect(editorState.editorChapterAssets.value).toEqual([])
    expect(soLuotGoiDocSegment).toBe(1)

    editorState.setEditorCaret(12)
    ketQuaGop.value = {
      outcome: { retired: FIXTURE_SEGMENTS.slice(0, 2).map((s) => ({ ...s })), new_segments: [HANG_MOI] },
      error: null,
    }
    expect(await editorState.mergeCurrentSegment()).toBe('done')
    // `refreshChapterAssetsAfterRegroup` chạy `void` (không chặn `mergeCurrentSegment`'s own
    // promise) -- đợi một tick cho nó tới nơi, cùng khuôn `editorRegroupGuards.test.ts`.
    await new Promise((resolve) => setTimeout(resolve, 0))

    expect(soLuotGoiDocSegment).toBe(2)
    expect(editorState.editorChapterAssets.value).toEqual([ANH_SAU_GOP])
    expect(editorState.editorAssetsDir.value).toBe('/tac-pham/assets')
    // `applyRegroup` chính nó không đụng `chapterAssets` -- mệnh đề trên phải đến từ lượt IPC
    // thứ hai, không phải từ `applyRegroup` tự vá.
    expect(editorState.editorSegments.value.map((s) => s.id)).toContain(HANG_MOI.id)
  })

  it('lượt IPC thứ hai (sau gộp) trả `loaded: null` ⇒ giữ ảnh CŨ và báo lỗi, không xoá về []', async () => {
    anhLanDau = [ANH_TRUOC_GOP]
    const { editorState } = await tuoi()
    expect(editorState.editorChapterAssets.value).toEqual([ANH_TRUOC_GOP])
    expect(editorState.editorAssetsDir.value).toBe('/tac-pham/assets-cu')

    const spyLoi = vi.spyOn(console, 'error').mockImplementation(() => {})
    editorState.setEditorCaret(12)
    ketQuaLanHai = 'loi'
    ketQuaGop.value = {
      outcome: { retired: FIXTURE_SEGMENTS.slice(0, 2).map((s) => ({ ...s })), new_segments: [HANG_MOI] },
      error: null,
    }
    expect(await editorState.mergeCurrentSegment()).toBe('done')
    await new Promise((resolve) => setTimeout(resolve, 0))

    expect(soLuotGoiDocSegment).toBe(2)
    expect(editorState.editorChapterAssets.value).toEqual([ANH_TRUOC_GOP])
    expect(editorState.editorAssetsDir.value).toBe('/tac-pham/assets-cu')
    expect(spyLoi).toHaveBeenCalled()
    spyLoi.mockRestore()
  })
})
