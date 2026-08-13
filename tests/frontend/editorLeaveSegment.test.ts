/**
 * *"Rời segment"* — **nhóm ③** của Quyết định #6. Story 2.3 · AC3 · AC13 · AC18 · Task 4.1.
 *
 * AC18 nói định nghĩa **đo được** của *"rời segment"* trên một trang liền mạch là
 * **`editorCaretSegmentId` đổi giá trị** — không có widget nào để "rời", vì AC1 của Story 2.2
 * cấm ô, bảng và khối. Và nó phải kích hoạt **ĐÚNG MỘT LẦN** cho mỗi lượt đổi.
 *
 * ⚠️ Đếm **số lượt gọi IPC**, không chỉ hiệu ứng cuối: hai lượt flush cho cùng một lượt rời
 * segment là hai giao dịch trên writer **duy nhất, nối tiếp** của AD-11, và không một hiệu ứng
 * quan sát được nào phân biệt chúng với một lượt.
 */
import { beforeEach, describe, expect, it, vi } from 'vitest'
import {
  failNextSave,
  FIXTURE_CHAPTER_ID,
  readFixture,
  recordSave,
  resetRecorder,
  saveCalls,
} from './support/segmentFixture'

vi.mock('../../src/config/segment', async (importOriginal) => {
  const actual = await importOriginal<typeof import('../../src/config/segment')>()
  return { ...actual, readOpenChapterSegments: readFixture, saveSegmentTargets: recordSave }
})

async function freshState() {
  vi.resetModules()
  const state = await import('../../src/panels/editorPanelState')
  await state.ensureSegmentsLoaded()
  return state
}

/** Nhường một vòng microtask cho lượt flush mà `setEditorCaret` phát bằng `void`. */
const settle = (): Promise<void> => new Promise((resolve) => setTimeout(resolve, 0))

beforeEach(() => {
  resetRecorder()
})

describe('rời segment = caret đổi câu (AC18)', () => {
  it('caret đi từ câu A sang câu B ⇒ ĐÚNG MỘT lượt flush, mang đúng câu A', async () => {
    const state = await freshState()

    state.setEditorCaret(11)
    state.noteEditorEdit(11, 'bản dịch câu A')
    expect(saveCalls.length).toBe(0)

    state.setEditorCaret(12)
    await settle()

    expect(saveCalls.length).toBe(1)
    expect(saveCalls[0].chapterId).toBe(FIXTURE_CHAPTER_ID)
    expect(saveCalls[0].edits).toEqual([{ id: 11, target_text: 'bản dịch câu A' }])
  })

  it('caret về `null` KHÔNG flush — bấm sang panel khác không phải rời segment', async () => {
    // Người dùng bấm sang Panel Lookup mấy chục lần trong một phiên. Mỗi lượt là một lượt ghi
    // trên writer nối tiếp nếu ca này sai, và tập chờ vẫn còn nguyên cho lượt flush theo nhịp.
    const state = await freshState()

    state.setEditorCaret(11)
    state.noteEditorEdit(11, 'đang gõ dở')
    state.setEditorCaret(null)
    await settle()

    expect(saveCalls.length).toBe(0)
    expect(state.editorEditedText.value.get(11)).toBe('đang gõ dở')
  })

  it('đặt lại caret vào CÙNG một câu KHÔNG flush', async () => {
    // `selectionchange` bắn liên tục khi người dùng kéo chọn trong một câu — mỗi lượt gọi
    // `setEditorCaret` với cùng một id. Một lượt flush ở đó là hàng chục giao dịch cho một cú kéo.
    const state = await freshState()

    state.setEditorCaret(12)
    state.noteEditorEdit(12, 'gõ trong câu B')
    for (let i = 0; i < 20; i += 1) state.setEditorCaret(12)
    await settle()

    expect(saveCalls.length).toBe(0)
  })

  it('không có gì đã đổi thì rời segment KHÔNG mở một lô rỗng', async () => {
    // Đọc qua trang rồi bấm từ câu này sang câu khác là chuyện thường xuyên nhất. Một lô rỗng
    // vẫn là một lượt IPC cộng một giao dịch trên writer nối tiếp của AD-11.
    const state = await freshState()

    state.setEditorCaret(11)
    state.setEditorCaret(12)
    state.setEditorCaret(13)
    await settle()

    expect(saveCalls.length).toBe(0)
  })

  it('một lô mang NHIỀU câu khi người dùng gõ xuyên qua chúng (AC13)', async () => {
    // AD-35 nói flush chạy mỗi 2 giây, và gõ xuyên qua ba câu trong 5 giây là chuyện thường.
    // Mệnh đề: MỘT lượt IPC cho cả lô, không một lượt mỗi câu.
    const state = await freshState()

    state.noteEditorEdit(11, 'câu một')
    state.noteEditorEdit(12, 'câu hai')
    state.noteEditorEdit(13, 'câu ba')
    await state.flushEditorNow()

    expect(saveCalls.length).toBe(1)
    expect(saveCalls[0].edits.map((e) => e.id).sort((a, b) => a - b)).toEqual([11, 12, 13])
  })

  it('một lượt ghi TRƯỢT giữ nguyên tập chờ và KHÔNG dựng mốc *"Đã lưu"*', async () => {
    // 🔴 Một mốc đi lên sau một lượt ghi trượt là màn hình nói dối theo hướng an tâm — đúng thứ
    // UX-DR30 tồn tại để tránh, và đúng lớp khuyết tật NFR18 chống.
    const state = await freshState()

    state.noteEditorEdit(11, 'chữ chưa lưu được')
    failNextSave.value = true
    await state.flushEditorNow()

    expect(saveCalls.length).toBe(1)
    expect(state.editorLastSavedAt.value).toBeNull()
    expect(state.editorEditedText.value.get(11)).toBe('chữ chưa lưu được')

    // Lượt sau thành công ⇒ tập chờ mới được dọn, và mốc mới được dựng.
    await state.flushEditorNow()
    expect(saveCalls.length).toBe(2)
    expect(saveCalls[1].edits).toEqual([{ id: 11, target_text: 'chữ chưa lưu được' }])
    expect(state.editorLastSavedAt.value).not.toBeNull()
  })

  it('`resetEditorPanel` vứt tập chờ — nên đường *"đóng Tác phẩm"* phải flush TRƯỚC nó', async () => {
    // Mệnh đề này là lưới của thứ tự đã ghi ở `libraryImport.ts::beginSubmit`: `resetEditorPanel`
    // **vứt** tập chờ không ghi, nên gọi nó trước lượt flush là ăn mất bản dịch của Tác phẩm cũ,
    // im lặng. Ca này khẳng định phần *"nó thật sự vứt"* — nửa còn lại là thứ tự ở chỗ gọi.
    const state = await freshState()

    state.noteEditorEdit(11, 'bản dịch của Tác phẩm CŨ')
    state.resetEditorPanel()
    await state.flushEditorNow()

    expect(saveCalls.length).toBe(0)
    expect(state.editorEditedText.value.size).toBe(0)
    expect(state.editorLastSavedAt.value).toBeNull()
  })
})
