/**
 * `confirmCurrentSegment` — Story 2.5, AC5 · AC7 · AC11 · Quyết định #1(a).
 *
 * ─────────────────────────────────────────────────────────────────────────────
 * 🔴 VÌ SAO TỆP NÀY LÀ LƯỚI **DUY NHẤT** CHO MỘT MỆNH ĐỀ CỦA AD-35
 * ─────────────────────────────────────────────────────────────────────────────
 * Vế (c) của AD-35 — *"xác nhận ⇒ flush trước, và flush chỉ xong sau khi đã vào WAL"* —
 * **không cưỡng chế được ở tầng Rust**: lệnh `confirm_segment` chỉ đọc thứ đã ở trên đĩa và
 * không biết gì về văn bản đang gõ trong webview. ⇒ Thứ tự *flush → confirm* sống ở
 * `editorPanelState.ts::confirmCurrentSegment`, và ca **① dưới đây là cổng duy nhất đứng đó**.
 *
 * ⚠️ **GIỚI HẠN THẬT, ghi ra thay vì để người sau tự phát hiện:** nó canh **chỗ gọi đó**, không
 * canh mọi chỗ gọi tương lai. Một bề mặt mới `invoke('confirm_segment')` thẳng sẽ đi vòng qua
 * cả hàm lẫn ca này mà **không cổng nào đỏ**.
 */
import { beforeEach, describe, expect, it, vi } from 'vitest'
import {
  FIXTURE_CHAPTER_ID,
  failNextSave,
  readFixture,
  recordSave,
  resetRecorder,
  saveCalls,
} from './support/segmentFixture'

/** Thứ tự các lượt gọi IPC, theo đúng thứ tự chúng xảy ra — đây là thứ ca ① đọc. */
const callOrder: string[] = []
/** Đối số của mỗi lượt `confirmSegment`. */
const confirmCalls: number[] = []
/** Bật để lượt `confirmSegment` kế tiếp trả về một lỗi từ chối. */
const failNextConfirm = { value: false }

async function recordConfirm(segmentId: number) {
  callOrder.push('confirm')
  confirmCalls.push(segmentId)
  if (failNextConfirm.value) {
    failNextConfirm.value = false
    return {
      outcome: null,
      error: {
        code: 'segment.nothing_to_confirm',
        message_key: 'err.segment.nothing_to_confirm',
        params: { segment_id: String(segmentId) },
        retryable: false,
      },
    }
  }
  return {
    outcome: { segment_id: segmentId, status: 'confirmed', version_created: true },
    error: null,
  }
}

/**
 * Việc chạy **XEN GIỮA** lượt save thứ `i`, theo thứ tự. `undefined` ⇒ không chen gì.
 *
 * 🔴 Đây là thứ dựng lại được ca đua tranh mà code review 2026-08-14 bắt được, và nó phải là
 * một **hook trong mock** chứ không một `setTimeout`: lúc `saveSegmentTargets` được gọi thì
 * `flushEditorNow` đã chụp `snapshot` xong và lô **đang bay**. Gõ thêm ở đúng khoảnh khắc đó
 * là gõ vào một ký tự nằm **ngoài** snapshot — không một đồng hồ giả nào cần tới.
 *
 * ⚠️ Đúng luật *"không `vi.useFakeTimers()` khi hàm đã nhận thời điểm qua tham số"*: ca này
 * không đo thời gian, nó đo **thứ tự**.
 */
const midFlightHooks: (undefined | (() => void))[] = []
let saveIndex = 0

async function recordSaveOrdered(
  chapterId: number,
  edits: readonly { id: number; target_text: string }[],
) {
  callOrder.push('save')
  midFlightHooks[saveIndex++]?.()
  return recordSave(chapterId, edits)
}

vi.mock('../../src/config/segment', async (importOriginal) => {
  const actual = await importOriginal<typeof import('../../src/config/segment')>()
  return {
    ...actual,
    readOpenChapterSegments: readFixture,
    saveSegmentTargets: recordSaveOrdered,
    confirmSegment: recordConfirm,
  }
})

async function freshState() {
  vi.resetModules()
  const state = await import('../../src/panels/editorPanelState')
  await state.ensureSegmentsLoaded()
  return state
}

beforeEach(() => {
  resetRecorder()
  callOrder.length = 0
  confirmCalls.length = 0
  failNextConfirm.value = false
  midFlightHooks.length = 0
  saveIndex = 0
})

describe('① AD-35 vế (c) — flush TRƯỚC, xác nhận SAU', () => {
  /**
   * 🔴 Gõ rồi bấm xác nhận ngay. Nếu `confirm` chạy trước `save`, chữ ký ghi cho một văn bản
   * **cũ hơn** thứ người dùng đang nhìn, và `SegmentVersion` sinh ra mang đúng văn bản cũ đó
   * ⇒ FR101 khôi phục về một thứ **chưa bao giờ ở trên màn hình**.
   *
   * Chạy đỏ-rồi-xanh: đảo hai dòng đầu của `confirmCurrentSegment`, ca này phải ĐỎ.
   */
  it('văn bản đang gõ được ghi xuống TRƯỚC lượt xác nhận', async () => {
    const state = await freshState()
    state.setEditorCaret(11)
    state.noteEditorEdit(11, 'Chữ vừa gõ, chưa kịp flush.')

    expect(await state.confirmCurrentSegment()).toBe('confirmed')

    expect(callOrder).toEqual(['save', 'confirm'])
    expect(saveCalls.at(0)?.edits).toEqual([{ id: 11, target_text: 'Chữ vừa gõ, chưa kịp flush.' }])
    expect(saveCalls.at(0)?.chapterId).toBe(FIXTURE_CHAPTER_ID)
    expect(confirmCalls).toEqual([11])
  })

  /**
   * 🔴 Lượt lưu TRƯỢT ⇒ **KHÔNG xác nhận**. Ký một câu mà lượt lưu vừa trượt là ghi một chữ ký
   * cho một văn bản **không tồn tại trên đĩa**, và ở Epic 7 nó thành một cặp TM cho một bản
   * dịch chưa bao giờ được lưu.
   */
  it('flush trượt ⇒ DỪNG, và không một lượt xác nhận nào được phát', async () => {
    const state = await freshState()
    state.setEditorCaret(11)
    state.noteEditorEdit(11, 'Chữ này sẽ không xuống được đĩa.')
    failNextSave.value = true

    expect(await state.confirmCurrentSegment()).toBe('flush-failed')

    expect(callOrder).toEqual(['save'])
    expect(confirmCalls).toEqual([])
  })
})

describe('①-bis Quyết định #8 — mã kết quả của flush KHÔNG đồng nghĩa với "tập chờ sạch"', () => {
  /**
   * 🔴 Khe hở code review 2026-08-14 tìm ra. Nhánh *originator* của `flushEditorNow` chụp
   * `snapshot` **trước** lượt IPC rồi trả `'saved'` mà **không** đệ quy — đúng cho auto-save,
   * sai cho lượt ký. Một ký tự gõ trong lúc lô bay nằm ngoài snapshot, nên lượt ký cũ sẽ ký
   * **văn bản trên đĩa**, tức bản thiếu ký tự đó.
   *
   * Chạy đỏ-rồi-xanh: bỏ khối `if (flush.isDirty())` trong `confirmCurrentSegment`, ca này ĐỎ
   * ở `callOrder` *(chỉ còn một `'save'`)* và ở văn bản của lượt save thứ hai.
   */
  it('gõ thêm trong lúc lô đang bay ⇒ flush lượt HAI, và lượt ký thấy văn bản mới', async () => {
    const state = await freshState()
    state.setEditorCaret(11)
    state.noteEditorEdit(11, 'Bản đầu.')
    // Người dùng gõ nốt một ký tự SAU khi đã bấm, trong lúc lô đầu đang bay.
    midFlightHooks[0] = () => state.noteEditorEdit(11, 'Bản đầu, và một chữ nữa.')

    expect(await state.confirmCurrentSegment()).toBe('confirmed')

    expect(callOrder).toEqual(['save', 'save', 'confirm'])
    expect(saveCalls.at(0)?.edits).toEqual([{ id: 11, target_text: 'Bản đầu.' }])
    expect(saveCalls.at(1)?.edits).toEqual([
      { id: 11, target_text: 'Bản đầu, và một chữ nữa.' },
    ])
    expect(confirmCalls).toEqual([11])
  })

  /**
   * 🔴 Vế thứ hai của Quyết định #8: **một** lượt thử lại, không một vòng lặp. Còn dơ sau lượt
   * hai ⇒ **từ chối**, không ký. Thà một phím tắt không làm gì còn hơn một `SegmentVersion`
   * mang văn bản chưa bao giờ ở trên màn hình.
   */
  it('còn dơ sau HAI lượt flush ⇒ `still-dirty`, và KHÔNG một lượt ký nào', async () => {
    const state = await freshState()
    // ⚠️ Segment **12**, không 11: fixture cho 11 khởi đầu ĐÃ KÝ, nên `status` của nó không
    //    phân biệt được *"lượt ký bị chặn"* với *"lượt ký chạy"*. Câu 12 là *đã dịch, chưa ký*.
    state.setEditorCaret(12)
    state.noteEditorEdit(12, 'Bản đầu.')
    midFlightHooks[0] = () => state.noteEditorEdit(12, 'Bản hai.')
    midFlightHooks[1] = () => state.noteEditorEdit(12, 'Bản ba.')

    expect(await state.confirmCurrentSegment()).toBe('still-dirty')

    expect(callOrder).toEqual(['save', 'save'])
    expect(confirmCalls).toEqual([])
    // Và ảnh chụp KHÔNG được nói dối theo hướng an tâm.
    expect(state.editorSegments.value.find((s) => s.id === 12)?.status).toBe('draft')
  })
})

describe('①-ter khoá chống-gọi-lại — hai lượt bấm, một thao tác', () => {
  /**
   * 🔴 Bấm `Mod+Enter` hai lần liên tiếp nhanh. Cả hai lượt đọc `caretSegmentId` **trước** khi
   * lượt đầu xong nên cùng thấy id `11`. AC13 lo phần Rust *(lượt hai không tạo
   * `SegmentVersion`)*, nhưng phần giao diện thì không: không có khoá, cả hai lượt chạy tới
   * nhánh dời con trỏ và cùng gán `caretPlacement`. Watcher bên `EditorPanel.vue` gọi
   * `clearEditorCaretPlacement()` ngay dòng đầu — nó là watcher **một phát** — nên lượt gán
   * thứ hai **vẫn** bắn và kéo caret về **offset 0** của câu kế tiếp, kể cả khi người dùng đã
   * kịp gõ vào đó.
   *
   * Chạy đỏ-rồi-xanh: bỏ khoá `confirmInFlight`, ca này ĐỎ với `confirmCalls` là `[11, 11]`.
   */
  it('lượt thứ hai NHẬP vào lượt đang bay — một lượt ký, một lượt dời con trỏ', async () => {
    const state = await freshState()
    state.setEditorCaret(11)

    const [first, second] = await Promise.all([
      state.confirmCurrentSegment(),
      state.confirmCurrentSegment(),
    ])

    expect(first).toBe('confirmed')
    expect(second).toBe('confirmed')
    expect(confirmCalls).toEqual([11])
    expect(callOrder.filter((c) => c === 'confirm')).toHaveLength(1)
    expect(state.editorCaretSegmentId.value).toBe(12)
  })

  /** Khoá phải **nhả** sau lượt bay — nếu không, lượt bấm kế tiếp chết im lặng. */
  it('bấm lượt sau, sau khi lượt trước đã xong ⇒ chạy bình thường', async () => {
    const state = await freshState()
    state.setEditorCaret(11)
    await state.confirmCurrentSegment()

    expect(await state.confirmCurrentSegment()).toBe('confirmed')

    expect(confirmCalls).toEqual([11, 12])
  })
})

describe('② Quyết định #1(a) — dời con trỏ sang câu kế tiếp', () => {
  /**
   * `primary` thắng `confirmed`, nên câu vừa ký chỉ hiện `confirmed` khi caret rời nó. Đây là
   * vế làm AC1 (*"vạch chuyển `confirmed`"*) quan sát được — nếu con trỏ ở lại, AC1 mô tả một
   * đổi màu **không xảy ra**.
   */
  it('xác nhận xong ⇒ con trỏ sang câu kế, và có một yêu cầu đặt caret cho giao diện', async () => {
    const state = await freshState()
    state.setEditorCaret(11)

    await state.confirmCurrentSegment()

    expect(state.editorCaretSegmentId.value).toBe(12)
    expect(state.editorCaretPlacement.value).toBe(12)
  })

  /** ⚠️ Câu **cuối Chương** không có câu kế ⇒ con trỏ **ở lại**. Ca này có thật. */
  it('câu cuối Chương ⇒ con trỏ ở lại, và KHÔNG yêu cầu đặt caret nào', async () => {
    const state = await freshState()
    state.setEditorCaret(13)
    state.noteEditorEdit(13, 'Câu cuối, nay đã có chữ.')

    expect(await state.confirmCurrentSegment()).toBe('confirmed')

    expect(state.editorCaretSegmentId.value).toBe(13)
    expect(state.editorCaretPlacement.value).toBeNull()
  })

  it('giao diện dọn tín hiệu xong thì lượt sau còn phân biệt được', async () => {
    const state = await freshState()
    state.setEditorCaret(11)
    await state.confirmCurrentSegment()

    state.clearEditorCaretPlacement()

    expect(state.editorCaretPlacement.value).toBeNull()
  })
})

describe('③ ảnh chụp hiển thị — và mốc so sánh của FR117 KHÔNG bị huỷ', () => {
  it('trạng thái mới vào `editorSegments` ngay, không chờ một lượt nạp lại', async () => {
    const state = await freshState()
    state.setEditorCaret(11)

    await state.confirmCurrentSegment()

    expect(state.editorSegments.value.find((s) => s.id === 11)?.status).toBe('confirmed')
  })

  /**
   * 🔴 **AC11(a).** `editorSegments` giữ bản **lúc nạp segment**, tức mốc mà FR117 (*xuất xứ*,
   * Story 2.7) so *"văn bản đích **hiện tại** với bản **lúc nạp**"* — **không dùng cờ dirty**
   * (hợp đồng phụ AD-31). Lượt cập nhật trạng thái được phép chạm `status` và **chỉ** `status`;
   * ghi văn bản đang gõ đè lên đó là huỷ đúng cái mốc, và nó hỏng ở một Epic sau mà **không gì
   * nối được về đây**.
   */
  it('`target_text` trong ảnh chụp GIỮ NGUYÊN bản lúc nạp, dù người dùng vừa gõ đè', async () => {
    const state = await freshState()
    const before = state.editorSegments.value.find((s) => s.id === 11)?.target_text
    state.setEditorCaret(11)
    state.noteEditorEdit(11, 'Một bản dịch hoàn toàn khác.')

    await state.confirmCurrentSegment()

    const after = state.editorSegments.value.find((s) => s.id === 11)
    expect(after?.target_text).toBe(before)
    expect(after?.status).toBe('confirmed')
    // Văn bản đang gõ vẫn sống ở kênh RIÊNG của nó — hai mốc, hai chỗ.
    expect(state.editorEditedText.value.get(11)).toBe('Một bản dịch hoàn toàn khác.')
  })
})

describe('④ AC14 — mọi lối từ chối phân biệt được, không rỗng im lặng', () => {
  it('không có con trỏ ⇒ `no-caret`, và không một lượt IPC nào được phát', async () => {
    const state = await freshState()
    state.setEditorCaret(null)

    expect(await state.confirmCurrentSegment()).toBe('no-caret')
    expect(callOrder).toEqual([])
  })

  it('Rust từ chối ⇒ `refused`, và lỗi đi ra NGUYÊN VẸN để `tError()` đọc', async () => {
    const state = await freshState()
    state.setEditorCaret(13)
    failNextConfirm.value = true

    expect(await state.confirmCurrentSegment()).toBe('refused')

    // 🔴 `message_key` đi ra y nguyên — chỗ gọi KHÔNG đoán lại lý do từ chuỗi lỗi.
    expect(state.editorConfirmError.value?.message_key).toBe('err.segment.nothing_to_confirm')
    // Và trạng thái trong ảnh chụp KHÔNG đổi: màn hình không được nói dối theo hướng an tâm.
    expect(state.editorSegments.value.find((s) => s.id === 13)?.status).toBe('draft')
  })

  it('một lượt thành công sau đó dọn lỗi cũ đi', async () => {
    const state = await freshState()
    state.setEditorCaret(13)
    failNextConfirm.value = true
    await state.confirmCurrentSegment()
    expect(state.editorConfirmError.value).not.toBeNull()

    state.setEditorCaret(11)
    await state.confirmCurrentSegment()

    expect(state.editorConfirmError.value).toBeNull()
  })
})
