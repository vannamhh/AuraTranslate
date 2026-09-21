/**
 * `panels/segmentSelectionState.ts` — vùng chọn nhiều-segment trong lưới. Story 4.9, Phase 1
 * (spec `4-9-dich-theo-lo-va-huy-giua-chung.md`), Decision 1 (Ice ký 2026-09-21).
 *
 * ─────────────────────────────────────────────────────────────────────────────
 * PHẠM VI — năm mệnh đề, và chỉ năm (§Tasks & Acceptance Phase 1 §Tests that move)
 * ─────────────────────────────────────────────────────────────────────────────
 * mở rộng · thu hẹp · xoá · vượt biên CHƯƠNG (tự lành, không cần một lượt reset tường minh
 * — xem doc-comment đầu `segmentSelectionState.ts`) · và HAI cụm reset đổi TÁC PHẨM
 * (`modes/libraryChapters.ts::openWorkById`, `modes/libraryImport.ts::finishImportSubmission`).
 * KHÔNG có AI ở Phase 1 — không một mệnh đề nào dưới đây chạm `aiTranslateState.ts` hay một
 * lệnh `#[tauri::command]` mới.
 *
 * 🔵 **THÊM Story 4.9, Phase 4b — mệnh đề THỨ SÁU.** Phạm vi "năm mệnh đề, và chỉ năm" ở trên
 * là của Phase 1; nó KHÔNG bao gồm dây nối `isSelected` (Phase 1 Task 2: "a selection the user
 * cannot see is not a selection" — `editorSegments.ts::selectedRowClassOf` +
 * `GridPanel.vue::selectedRowClassById`). Đo được trước bản thêm này: `grep -rn
 * "selectedRowClassOf" tests/frontend/*.ts` cho 0 kết quả — dây nối đó không một ca nào canh.
 * `describe` cuối tệp này đóng khoảng trống đó và là mục tiêu của counter-check by removal
 * Phase 4 đòi cho dây nối `isSelected`.
 */
import { beforeEach, describe, expect, it, vi } from 'vitest'
import { mount } from '@vue/test-utils'
import { FIXTURE_CHAPTER_ID, FIXTURE_SEGMENTS } from './support/segmentFixture'
import type { ChapterSegment } from '../../src/config/segment'

/** Chương THỨ HAI — id KHÔNG chồng với `FIXTURE_SEGMENTS`, đúng thứ `AUTOINCREMENT` của cùng
 * một `project.db` đảm bảo (segment.id đếm CHUNG cho mọi Chương của một Tác phẩm). Dùng để
 * dựng lượt "đổi Chương trong CÙNG một Tác phẩm". */
const CHAPTER_B_ID = 8
const CHAPTER_B_SEGMENTS: readonly ChapterSegment[] = [
  {
    id: 21,
    ord: 1,
    source_text: '甲。',
    target_text: '',
    is_paragraph_end: false,
    retired_at: null,
    status: 'draft',
    is_omitted: false,
    is_target_paragraph_end: false,
    role: null,
  },
  {
    id: 22,
    ord: 2,
    source_text: '乙。',
    target_text: '',
    is_paragraph_end: false,
    retired_at: null,
    status: 'draft',
    is_omitted: false,
    is_target_paragraph_end: false,
    role: null,
  },
]

/** Chương ĐANG "mở" trong bộ giả `readOpenChapterSegments` — đổi giá trị này để mô phỏng một
 * lượt đổi Chương, cùng khuôn `chuongDangMo` của `editorChapterSwitch.test.ts`. */
const activeFixture: { value: { chapterId: number; segments: readonly ChapterSegment[] } } = {
  value: { chapterId: FIXTURE_CHAPTER_ID, segments: FIXTURE_SEGMENTS },
}

async function readActiveFixture(): Promise<{
  loaded: { chapter_id: number; segments: ChapterSegment[] }
  error: null
}> {
  return {
    loaded: {
      chapter_id: activeFixture.value.chapterId,
      segments: activeFixture.value.segments.map((s) => ({ ...s })),
    },
    error: null,
  }
}

vi.mock('../../src/config/segment', async (importOriginal) => {
  const actual = await importOriginal<typeof import('../../src/config/segment')>()
  return { ...actual, readOpenChapterSegments: readActiveFixture }
})

// ── Chỉ cần cho HAI ca cụm reset dưới cùng — mock chung một lượt cho cả tệp, cùng khuôn
// `libraryChaptersResetsAiPromptInspector.test.ts`/`libraryImportResetsAiPromptInspector.test.ts`.
const TAC_PHAM_MOI_CHUONG = { meta: { work_id: 'w-moi' }, folder: '/tmp/w-moi.atproj', chapter_id: 1 }
const TAC_PHAM_MOI_NHAP = { work_id: 'w-moi-nhap', name: 'Tac pham nhap moi', chapter_count: 1 }

vi.mock('../../src/config/library', async (importOriginal) => {
  const actual = await importOriginal<typeof import('../../src/config/library')>()
  return { ...actual, openWork: async () => ({ opened: TAC_PHAM_MOI_CHUONG, error: null }) }
})

vi.mock('../../src/config/chapter', async (importOriginal) => {
  const actual = await importOriginal<typeof import('../../src/config/chapter')>()
  return { ...actual, listChapters: async () => ({ chapters: [], error: null }) }
})

vi.mock('../../src/config/project', async (importOriginal) => {
  const actual = await importOriginal<typeof import('../../src/config/project')>()
  return {
    ...actual,
    previewImportEncodingFromText: async () => ({
      preview: { confidence: 'self_declared', selected_encoding: 'UTF-8', candidates: [] },
      error: null,
    }),
    confirmImportWithEncoding: async () => ({ created: TAC_PHAM_MOI_NHAP, error: null }),
  }
})

// `libraryImport.ts` gọi `listen` ở cấp module; ngoài Tauri nó không phân giải được.
vi.mock('@tauri-apps/api/event', () => ({ listen: async () => () => {} }))

// Tiền đề của `openWorkById`/`finishImportSubmission`, không phải thứ tệp này nghiệm thu —
// cùng khuôn hai tệp chị em đã dẫn ở trên. `editorSegments`/`editorCaretSegmentId`/
// `resetEditorPanel`/`ensureSegmentsLoaded` vẫn là bản THẬT (đi qua `...actual`).
vi.mock('../../src/panels/editorPanelState', async (importOriginal) => {
  const actual = await importOriginal<typeof import('../../src/panels/editorPanelState')>()
  return {
    ...actual,
    flushEditorBeforeDiscreteWrite: async () => 'clean' as const,
    flushChapterPositionNow: async () => {},
  }
})

beforeEach(() => {
  vi.resetModules()
  activeFixture.value = { chapterId: FIXTURE_CHAPTER_ID, segments: FIXTURE_SEGMENTS }
})

/**
 * Nạp lại CẢ HAI module trong cùng một lượt — cùng khuôn `editorChapterSwitch.test.ts::tuoi`.
 *
 * ⚠️ **`resetEditorPanel()` + `resetSegmentSelection()` TRƯỚC lượt nạp — có chủ, không thừa.**
 * Đo được: `vi.mock(..., importOriginal)` giữ nguyên MỘT bản `editorPanelState` thật xuyên
 * suốt cả tệp — `vi.resetModules()` cấp một wrapper mock MỚI mỗi lượt nhưng `caretSegmentId`
 * bên trong `actual` vẫn là ô nhớ CŨ, nên caret của một ca trước rò sang ca sau dù `tuoi()` gọi
 * `import()` "mới". Hai lượt reset tường minh dưới đây khoá đúng tiền đề mỗi ca cần, độc lập
 * với việc `resetModules()` có dọn sạch hay không.
 */
async function tuoi() {
  const editorState = await import('../../src/panels/editorPanelState')
  const selectionState = await import('../../src/panels/segmentSelectionState')
  editorState.resetEditorPanel()
  selectionState.resetSegmentSelection()
  await editorState.ensureSegmentsLoaded()
  return { editorState, selectionState }
}

describe('mở rộng — neo tại CARET, gieo VÀ dời trong CÙNG một lượt bấm (AC1)', () => {
  it('bấm MỘT lần ⇒ đúng HAI hàng liên tiếp được chọn', async () => {
    const { editorState, selectionState } = await tuoi()
    editorState.setEditorCaret(11)

    selectionState.extendSegmentSelectionDown()

    expect(selectionState.segmentSelectionIds.value).toEqual([11, 12])
    expect(selectionState.segmentSelectionCount.value).toBe(2)
  })

  it('bấm HAI lần ⇒ đúng BA hàng — n lần thì n+1 hàng (AC1 nguyên văn)', async () => {
    const { editorState, selectionState } = await tuoi()
    editorState.setEditorCaret(11)

    selectionState.extendSegmentSelectionDown()
    selectionState.extendSegmentSelectionDown()

    expect(selectionState.segmentSelectionIds.value).toEqual([11, 12, 13])
    expect(selectionState.segmentSelectionCount.value).toBe(3)
  })

  it('caret ở BIÊN Chương ⇒ không quay vòng, vùng chọn dừng lại ở ĐÚNG MỘT hàng', async () => {
    // Ca "chọn đúng một" của §I/O Matrix, đạt được qua bàn phím ở một biên — không một lệnh
    // "chọn đúng dòng hiện tại" riêng nào cần tồn tại.
    const { editorState, selectionState } = await tuoi()
    editorState.setEditorCaret(13) // câu CUỐI Chương

    selectionState.extendSegmentSelectionDown()

    expect(selectionState.segmentSelectionIds.value).toEqual([13])
    expect(selectionState.segmentSelectionCount.value).toBe(1)
  })

  it('caret là `null` ⇒ không gieo được gì, vùng chọn ở lại rỗng', async () => {
    const { selectionState } = await tuoi()

    selectionState.extendSegmentSelectionDown()

    expect(selectionState.segmentSelectionCount.value).toBe(0)
    expect(selectionState.segmentSelectionAnchorId.value).toBeNull()
  })
})

describe('thu hẹp — hợp âm ngược hướng dời `focus` VỀ PHÍA `anchor`, không mở một vùng thứ hai', () => {
  it('mở rộng hai lần rồi thu hẹp một lần ⇒ về lại đúng HAI hàng', async () => {
    const { editorState, selectionState } = await tuoi()
    editorState.setEditorCaret(11)
    selectionState.extendSegmentSelectionDown()
    selectionState.extendSegmentSelectionDown()
    expect(selectionState.segmentSelectionIds.value).toEqual([11, 12, 13])

    selectionState.extendSegmentSelectionUp()

    expect(selectionState.segmentSelectionIds.value).toEqual([11, 12])
  })

  it('`focus` có thể dời qua phía BÊN KIA của `anchor` — cùng khuôn `Shift+Home/End` văn bản', async () => {
    const { editorState, selectionState } = await tuoi()
    editorState.setEditorCaret(13)
    selectionState.extendSegmentSelectionUp() // anchor=13, focus=13 → dời lên → [12, 13]
    expect(selectionState.segmentSelectionIds.value).toEqual([12, 13])

    selectionState.extendSegmentSelectionUp() // focus dời tiếp qua 11, vẫn phía TRÊN anchor
    expect(selectionState.segmentSelectionIds.value).toEqual([11, 12, 13])

    // Biên Chương — không quay vòng, vùng chọn giữ nguyên.
    selectionState.extendSegmentSelectionUp()
    expect(selectionState.segmentSelectionIds.value).toEqual([11, 12, 13])
  })
})

describe('xoá — `segment.selection.clear`', () => {
  it('xoá một vùng chọn ĐANG CÓ ⇒ rỗng, cả `anchor` lẫn `focus`', async () => {
    const { editorState, selectionState } = await tuoi()
    editorState.setEditorCaret(11)
    selectionState.extendSegmentSelectionDown()
    expect(selectionState.segmentSelectionCount.value).toBe(2)

    selectionState.clearSegmentSelection()

    expect(selectionState.segmentSelectionCount.value).toBe(0)
    expect(selectionState.segmentSelectionAnchorId.value).toBeNull()
    expect(selectionState.segmentSelectionFocusId.value).toBeNull()
  })

  it('sau khi xoá, mở rộng lại gieo neo MỚI tại caret hiện tại — không hồi sinh vùng cũ', async () => {
    const { editorState, selectionState } = await tuoi()
    editorState.setEditorCaret(11)
    selectionState.extendSegmentSelectionDown()
    selectionState.clearSegmentSelection()
    editorState.setEditorCaret(13)

    selectionState.extendSegmentSelectionUp()

    expect(selectionState.segmentSelectionIds.value).toEqual([12, 13])
  })
})

describe('vượt biên CHƯƠNG (cùng Tác phẩm) — TỰ LÀNH, không cần một lượt reset tường minh', () => {
  it('🔴 đổi Chương dưới một vùng chọn cũ ⇒ vùng chọn tự rỗng, không phải trỏ sai chỗ', async () => {
    // Đường hỏng NẾU đây không tự lành: `anchor`/`focus` vẫn mang id 11/12 của Chương A, và
    // nếu Chương B (đo hiếm nhưng có thể) từng tái dùng đúng hai id đó, vùng chọn sẽ ÂM THẦM
    // trỏ vào hai câu người dùng chưa từng chọn — đúng lớp lỗi `resetEditorPanel()` đã ghi cho
    // `confirmError`/`caretPlacement`/`sourceCut`. `segment.id` KHÔNG tái dùng trong CÙNG một
    // `project.db` (mọi Chương của một Tác phẩm đếm CHUNG một `AUTOINCREMENT`), nên phép
    // `indexOf` dưới đây luôn trả `-1` một cách CÓ CHỦ, không phải một trùng hợp may mắn.
    const { editorState, selectionState } = await tuoi()
    editorState.setEditorCaret(11)
    selectionState.extendSegmentSelectionDown()
    expect(selectionState.segmentSelectionCount.value).toBe(2)

    // Mô phỏng đúng NÉT của lượt đổi Chương thật (`switchChapter`): vứt state Editor rồi nạp
    // lại — không đi qua toàn bộ `openAdjacentChapter`/IPC, vì mệnh đề tệp này canh là hành vi
    // CỦA `segmentSelectionState.ts` khi `editorSegments` đổi, không phải chỗ nối IPC đó.
    activeFixture.value = { chapterId: CHAPTER_B_ID, segments: CHAPTER_B_SEGMENTS }
    editorState.resetEditorPanel()
    await editorState.ensureSegmentsLoaded()
    expect(editorState.editorChapterId.value).toBe(CHAPTER_B_ID)

    // 🔴 KHÔNG một lời gọi `resetSegmentSelection()` nào ở trên — mệnh đề trung tâm của ca này
    // là vùng chọn tự rỗng CHỈ nhờ phép tính lại trên `editorSegments` mới.
    expect(selectionState.segmentSelectionCount.value).toBe(0)
    expect(selectionState.segmentSelectionIds.value).toEqual([])
  })

  it('mở rộng lại sau khi vượt biên ⇒ gieo neo tại caret của Chương MỚI, chọn được bình thường', async () => {
    const { editorState, selectionState } = await tuoi()
    editorState.setEditorCaret(11)
    selectionState.extendSegmentSelectionDown()

    activeFixture.value = { chapterId: CHAPTER_B_ID, segments: CHAPTER_B_SEGMENTS }
    editorState.resetEditorPanel()
    await editorState.ensureSegmentsLoaded()
    editorState.setEditorCaret(21)

    selectionState.extendSegmentSelectionDown()

    expect(selectionState.segmentSelectionIds.value).toEqual([21, 22])
  })
})

describe('`anchor` biến mất mà `focus` còn sống (gộp/tách segment, Story 2.8/2.9) — vẫn phải gieo lại được', () => {
  it('🔴 gỡ phép kiểm `anchorIndex === -1` khỏi lượt gieo lại sẽ làm ca này ĐỎ: vùng chọn đọc RỖNG mãi mãi thay vì gieo lại', async () => {
    // Đường hỏng nếu vế `anchorIndex` bị gỡ: một lượt gộp/tách về hưu ĐÚNG hàng `anchor`
    // (11) trong khi `focus` (12) còn sống — `moveSegmentSelectionFocus` chỉ kiểm
    // `focusIndex`, thấy nó HỢP LỆ, nên KHÔNG gieo lại: nó dời `focus` từ 12 sang 13 một
    // cách ÂM THẦM trong khi `anchor` vẫn trỏ vào id 11 đã chết. `segmentSelectionIds` đòi
    // CẢ HAI chỉ số hợp lệ (`:80`) nên vùng chọn đọc RỖNG — và ở lại rỗng MÃI MÃI, vì
    // `focusIndex` sẽ luôn hợp lệ cho tới lượt gộp/tách kế tiếp.
    const { editorState, selectionState } = await tuoi()
    editorState.setEditorCaret(11)
    selectionState.extendSegmentSelectionDown()
    expect(selectionState.segmentSelectionIds.value).toEqual([11, 12])

    // Mô phỏng kết cục của một lượt gộp/tách: hàng 11 (`anchor`) về hưu và biến mất khỏi
    // `editorSegments`; hàng 12 (`focus`) sống sót; caret dời sang hàng sống sót — đúng nét
    // `editorPanelState.ts::mergeCurrentSegment`/`splitCurrentSegment` để lại sau khi ghi.
    activeFixture.value = {
      chapterId: FIXTURE_CHAPTER_ID,
      segments: [FIXTURE_SEGMENTS[1]!, FIXTURE_SEGMENTS[2]!], // chỉ còn id 12, 13
    }
    editorState.resetEditorPanel()
    await editorState.ensureSegmentsLoaded()
    editorState.setEditorCaret(12)

    selectionState.extendSegmentSelectionDown()

    // 🔴 Mệnh đề trung tâm: gieo lại từ caret (12), KHÔNG dời `focus` cũ trong im lặng.
    expect(selectionState.segmentSelectionIds.value).toEqual([12, 13])
    expect(selectionState.segmentSelectionAnchorId.value).toBe(12)
  })
})

describe('cụm reset #1 — `modes/libraryChapters.ts::openWorkById` (đổi Tác phẩm)', () => {
  it('🔴 gỡ `resetSegmentSelection()` khỏi `openWorkById` sẽ làm ca này ĐỎ: vùng chọn của Tác phẩm CŨ không sống sót', async () => {
    const { editorState, selectionState } = await tuoi()
    editorState.setEditorCaret(11)
    selectionState.extendSegmentSelectionDown()
    expect(selectionState.segmentSelectionCount.value).toBe(2)

    const chuong = await import('../../src/modes/libraryChapters')
    await chuong.openWorkById('w-moi')

    // Tiền đề của ca: lượt mở phải THÀNH CÔNG, không thì hàm thoát sớm và ca sẽ xanh vì một lý
    // do sai — cùng bẫy hai tệp chị em (`libraryChaptersResetsAiPromptInspector.test.ts`) đã ghi.
    expect(chuong.libraryOpenWorkError.value).toBeNull()

    expect(selectionState.segmentSelectionCount.value).toBe(0)
    expect(selectionState.segmentSelectionAnchorId.value).toBeNull()
    expect(selectionState.segmentSelectionFocusId.value).toBeNull()
  })
})

describe('cụm reset #2 — `modes/libraryImport.ts::finishImportSubmission` (đổi Tác phẩm)', () => {
  it('🔴 gỡ `resetSegmentSelection()` khỏi `finishImportSubmission` sẽ làm ca này ĐỎ: vùng chọn của Tác phẩm CŨ không sống sót', async () => {
    const { editorState, selectionState } = await tuoi()
    editorState.setEditorCaret(11)
    selectionState.extendSegmentSelectionDown()
    expect(selectionState.segmentSelectionCount.value).toBe(2)

    const nhap = await import('../../src/modes/libraryImport')
    const preview = await import('../../src/importPreviewState')

    nhap.pastedText.value = 'Cau nguyen van cua Tac pham moi.'
    await nhap.submitPastedText()
    expect(preview.importPreviewIsOpen.value).toBe(true)

    const result = await preview.confirmImportPreview()
    nhap.finishImportSubmission(result.created, result.error)

    // Tiền đề của ca: lượt nhập phải THÀNH CÔNG — cùng bẫy tệp chị em đã ghi.
    expect(nhap.createdWork.value).not.toBeNull()

    expect(selectionState.segmentSelectionCount.value).toBe(0)
    expect(selectionState.segmentSelectionAnchorId.value).toBeNull()
    expect(selectionState.segmentSelectionFocusId.value).toBeNull()
  })
})

describe('GridPanel.vue vẽ vùng chọn — dây nối `isSelected` (Phase 1 Task 2, mục tiêu counter-check Phase 4)', () => {
  it('hàng nằm trong vùng chọn mang class `row-selected` ở ô nguyên văn, hàng ngoài vùng chọn thì không', async () => {
    const { editorState, selectionState } = await tuoi()
    editorState.setEditorCaret(11)
    selectionState.extendSegmentSelectionDown()
    expect(selectionState.segmentSelectionIds.value).toEqual([11, 12])

    const GridPanel = (await import('../../src/panels/GridPanel.vue')).default
    const wrapper = mount(GridPanel, {
      props: { params: {} } as never,
      global: { stubs: { PanelFrame: { template: '<div class="panel-frame"><slot /></div>' } } },
      attachTo: document.body,
    })
    await wrapper.vm.$nextTick()

    const root = wrapper.element as unknown as Element
    const srcCellFor = (id: number): Element | null => root.querySelector(`[data-col="src"][data-segment-id="${id}"]`)

    expect(srcCellFor(11)?.classList.contains('row-selected')).toBe(true)
    expect(srcCellFor(12)?.classList.contains('row-selected')).toBe(true)
    expect(srcCellFor(13)?.classList.contains('row-selected')).toBe(false)

    wrapper.unmount()
  })
})
