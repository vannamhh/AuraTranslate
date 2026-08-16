/**
 * `segmentHistoryState` — Story 2.6 · FR101 · AC1 · AC2 · AC3.
 *
 * ─────────────────────────────────────────────────────────────────────────────
 * 🔴 MỆNH ĐỀ NÀO THUỘC TỆP NÀY, VÀ MỆNH ĐỀ NÀO **KHÔNG**
 * ─────────────────────────────────────────────────────────────────────────────
 * *"Khôi phục đặt lại `target_text` và hạ `status` trên ĐĨA"*, *"lịch sử không dài thêm"*,
 * *"một `version_id` của segment khác bị từ chối"*, *"chốt chống mất bản nháp so bằng văn
 * bản"* — tất cả là mệnh đề của **Rust**, và cả bốn đã có chủ ở `segment_contract.rs`. Dựng
 * lại chúng ở đây là hai nguồn sự thật cho một mệnh đề.
 *
 * Thứ **chỉ** tệp này canh được — trạng thái phía webview:
 *   ① ba trạng thái rỗng **phân biệt được** *(chưa nhắm câu · đọc hỏng · chưa từng ký)*;
 *   ② lượt khôi phục dựng một **mảng mới** cho ảnh chụp — `shallowRef` không theo dõi sửa tại
 *     chỗ, và Story 2.5c mất một vòng chẩn đoán ở đúng chỗ này (commit `4ce5bb4`);
 *   ③ `needs_confirmation` **không** đi vào ô lỗi, và nó **không** đổi ảnh chụp;
 *   ④ **flush chạy TRƯỚC** lượt khôi phục — chốt chống mất bản nháp so trên đĩa, nên một ký
 *     tự chưa xuống WAL làm nó không hỏi ở đúng ca cần hỏi nhất.
 *
 * ⚠️ **Giới hạn thật của cây test này, ghi ra thay vì để người sau tưởng đã được canh:**
 * fixture **chép tay** hình dạng của dây. Đó chính là thứ đã cho 74/74 xanh trên một sản phẩm
 * đang hỏng ở Story 2.5. Lưới cho vế *"Rust có thật sự gửi bốn trường không"* nằm ở
 * `segment_contract.rs`, **không** ở đây.
 */
import { beforeEach, describe, expect, it, vi } from 'vitest'

import { readFixture, recordSave, resetRecorder } from './support/segmentFixture'

/** Thứ tự các lượt gọi IPC — đây là thứ ca ④ đọc. */
const callOrder: string[] = []
/** Đối số của mỗi lượt `restoreSegmentVersion`, theo đúng thứ tự. */
const restoreCalls: { segmentId: number; versionId: number; force: boolean }[] = []
/** Đối số của mỗi lượt `readSegmentHistory`. */
const historyCalls: number[] = []

/** Bật để lượt `readSegmentHistory` kế tiếp trả về một lỗi. */
const failNextRead = { value: false }
/** Bật để lượt `restoreSegmentVersion` kế tiếp trả về `needs_confirmation`. */
const holdNextRestore = { value: false }
/** Bật để lượt `restoreSegmentVersion` kế tiếp trả về một lỗi từ chối. */
const failNextRestore = { value: false }
/** Bật để lượt `restoreSegmentVersion` kế tiếp trả `restored: false` (ca vô hại). */
const unchangedNextRestore = { value: false }

/** Lịch sử giả cho segment 11 — hai phiên bản, mới nhất trước. */
const VERSIONS = [
  {
    id: 202,
    segment_id: 11,
    target_text: 'Ban hai.',
    created_at: '2026-08-16T10:00:00.000Z',
  },
  {
    id: 201,
    segment_id: 11,
    target_text: 'Ban mot.',
    created_at: '2026-08-16T09:00:00.000Z',
  },
]

async function recordRead(segmentId: number) {
  callOrder.push('read')
  historyCalls.push(segmentId)
  if (failNextRead.value) {
    failNextRead.value = false
    return {
      versions: null,
      error: {
        code: 'segment.not_found',
        message_key: 'err.segment.not_found',
        params: { segment_id: String(segmentId) },
        retryable: false,
      },
    }
  }
  // Chi segment 11 co lich su; moi segment khac chua tung duoc ky ⇒ RONG, khong loi.
  return { versions: segmentId === 11 ? VERSIONS : [], error: null }
}

async function recordRestore(segmentId: number, versionId: number, force: boolean) {
  callOrder.push('restore')
  restoreCalls.push({ segmentId, versionId, force })

  if (failNextRestore.value) {
    failNextRestore.value = false
    return {
      outcome: null,
      error: {
        code: 'segment.retired',
        message_key: 'err.segment.retired',
        params: { segment_id: String(segmentId) },
        retryable: false,
      },
    }
  }

  if (holdNextRestore.value) {
    holdNextRestore.value = false
    return {
      outcome: {
        segment_id: segmentId,
        status: 'draft',
        restored: false,
        needs_confirmation: true,
        unsigned_draft: 'Ban nhap chua ai ky.',
      },
      error: null,
    }
  }

  if (unchangedNextRestore.value) {
    unchangedNextRestore.value = false
    return {
      outcome: {
        segment_id: segmentId,
        status: 'confirmed',
        restored: false,
        needs_confirmation: false,
        unsigned_draft: null,
      },
      error: null,
    }
  }

  return {
    outcome: {
      segment_id: segmentId,
      status: 'draft',
      restored: true,
      needs_confirmation: false,
      unsigned_draft: null,
    },
    error: null,
  }
}

async function recordSaveOrdered(
  chapterId: number,
  edits: readonly { id: number; target_text: string }[],
) {
  callOrder.push('flush')
  return recordSave(chapterId, edits)
}

vi.mock('../../src/config/segment', async (importOriginal) => {
  const actual = await importOriginal<typeof import('../../src/config/segment')>()
  return {
    ...actual,
    readOpenChapterSegments: readFixture,
    saveSegmentTargets: recordSaveOrdered,
    readSegmentHistory: recordRead,
    restoreSegmentVersion: recordRestore,
  }
})

async function freshState() {
  vi.resetModules()
  const editor = await import('../../src/panels/editorPanelState')
  await editor.ensureSegmentsLoaded()
  const history = await import('../../src/panels/segmentHistoryState')
  return { editor, history }
}

beforeEach(() => {
  resetRecorder()
  callOrder.length = 0
  restoreCalls.length = 0
  historyCalls.length = 0
  failNextRead.value = false
  holdNextRestore.value = false
  failNextRestore.value = false
  unchangedNextRestore.value = false
})

describe('① ba trạng thái rỗng, và cả ba phải phân biệt được', () => {
  it('mở lịch sử của câu đang có caret ⇒ đọc đúng câu đó, mới nhất trước', async () => {
    const { editor, history } = await freshState()
    editor.setEditorCaret(11)

    history.openSegmentHistory()
    await vi.waitFor(() => expect(history.historyPending.value).toBe(false))

    expect(historyCalls).toEqual([11])
    expect(history.historyIsOpen.value).toBe(true)
    expect(history.historyVersions.value.map((v) => v.id)).toEqual([202, 201])
    expect(history.historyLoadError.value).toBeNull()
  })

  /**
   * 🔴 Lớp phủ vẫn **MỞ** và nói ra điều đó. Mở một lớp phủ rỗng rồi im lặng là đúng thứ
   * *"rỗng im lặng"* bị cấm; không mở gì cả thì một cú bấm phím không phản hồi, và người
   * dùng không biết mình vừa làm gì sai.
   */
  it('chưa nhắm được câu nào ⇒ mở, `segmentId` null, và KHÔNG một lượt IPC nào', async () => {
    const { editor, history } = await freshState()
    editor.setEditorCaret(null)

    history.openSegmentHistory()

    expect(history.historyIsOpen.value).toBe(true)
    expect(history.historySegmentId.value).toBeNull()
    expect(historyCalls).toEqual([])
  })

  it('câu CHƯA TỪNG ký ⇒ danh sách rỗng và KHÔNG lỗi — rỗng CÓ LÝ DO', async () => {
    const { editor, history } = await freshState()
    editor.setEditorCaret(12)

    history.openSegmentHistory()
    await vi.waitFor(() => expect(history.historyPending.value).toBe(false))

    expect(history.historyVersions.value).toEqual([])
    expect(history.historyLoadError.value).toBeNull()
  })

  /**
   * 🔴 *"Đọc hỏng"* và *"chưa có phiên bản nào"* dẫn tới **hai việc khác nhau** cho người
   * dùng, nên chúng phải là hai trạng thái. Gộp lại là để màn hình nói một câu SAI —
   * *"câu này chưa có phiên bản nào"* — trong khi nguyên nhân thật là một lượt đọc trượt.
   */
  it('đọc hỏng ⇒ danh sách rỗng NHƯNG `historyLoadError` khác null', async () => {
    const { editor, history } = await freshState()
    editor.setEditorCaret(11)
    failNextRead.value = true

    history.openSegmentHistory()
    await vi.waitFor(() => expect(history.historyPending.value).toBe(false))

    expect(history.historyVersions.value).toEqual([])
    expect(history.historyLoadError.value).not.toBeNull()
    expect(history.historyLoadError.value?.message_key).toBe('err.segment.not_found')
  })
})

describe('② lượt khôi phục dựng một MẢNG MỚI cho ảnh chụp', () => {
  it('khôi phục ⇒ `editorSegments` thay bằng mảng mới, mang văn bản và trạng thái mới', async () => {
    const { editor, history } = await freshState()
    editor.setEditorCaret(11)
    history.openSegmentHistory()
    await vi.waitFor(() => expect(history.historyPending.value).toBe(false))

    const before = editor.editorSegments.value

    expect(await history.restoreVersion(201, false)).toBe('restored')

    const after = editor.editorSegments.value
    // 🔴 Mot MANG MOI, khong mot luot sua tai cho: `shallowRef` khong theo doi sua tai cho
    //    ⇒ dia doi ma luoi thi khong.
    expect(after).not.toBe(before)
    const seg = after.find((s) => s.id === 11)
    expect(seg?.target_text).toBe('Ban mot.')
    expect(seg?.status).toBe('draft')
    expect(history.historyRestoreNotice.value).toBe('restored')
  })

  it('một lượt bị TỪ CHỐI KHÔNG được đổi ảnh chụp — nếu không lưới nói dối về đĩa', async () => {
    const { editor, history } = await freshState()
    editor.setEditorCaret(11)
    history.openSegmentHistory()
    await vi.waitFor(() => expect(history.historyPending.value).toBe(false))

    const before = editor.editorSegments.value
    failNextRestore.value = true

    expect(await history.restoreVersion(201, false)).toBe('refused')

    expect(editor.editorSegments.value).toBe(before)
    expect(history.historyRestoreError.value?.message_key).toBe('err.segment.retired')
    expect(history.historyRestoreNotice.value).toBeNull()
  })

  it('ca VÔ HẠI (`restored: false`) KHÔNG đổi ảnh chụp và KHÔNG phải một lỗi', async () => {
    const { editor, history } = await freshState()
    editor.setEditorCaret(11)
    history.openSegmentHistory()
    await vi.waitFor(() => expect(history.historyPending.value).toBe(false))

    const before = editor.editorSegments.value
    unchangedNextRestore.value = true

    expect(await history.restoreVersion(202, false)).toBe('unchanged')

    expect(editor.editorSegments.value).toBe(before)
    expect(history.historyRestoreError.value).toBeNull()
    expect(history.historyRestoreNotice.value).toBe('unchanged')
  })
})

describe('③ `needs_confirmation` là một CÂU HỎI, không một lỗi', () => {
  it('lượt đầu bị giữ lại ⇒ mang bản nháp ra, KHÔNG vào ô lỗi, KHÔNG đổi ảnh chụp', async () => {
    const { editor, history } = await freshState()
    editor.setEditorCaret(11)
    history.openSegmentHistory()
    await vi.waitFor(() => expect(history.historyPending.value).toBe(false))

    const before = editor.editorSegments.value
    holdNextRestore.value = true

    expect(await history.restoreVersion(201, false)).toBe('needs-confirmation')

    expect(history.historyPendingRestore.value).toEqual({
      versionId: 201,
      draft: 'Ban nhap chua ai ky.',
    })
    // 🔴 KHONG vao o loi: dinh tuyen mot cau hoi vao o bao hong la noi sai voi nguoi dung.
    expect(history.historyRestoreError.value).toBeNull()
    expect(history.historyRestoreNotice.value).toBeNull()
    // Va KHONG mot byte nao da doi.
    expect(editor.editorSegments.value).toBe(before)
  })

  it('đồng ý ⇒ gọi lại với `force = true`, và ĐÚNG phiên bản đang chờ', async () => {
    const { editor, history } = await freshState()
    editor.setEditorCaret(11)
    history.openSegmentHistory()
    await vi.waitFor(() => expect(history.historyPending.value).toBe(false))

    holdNextRestore.value = true
    await history.restoreVersion(201, false)

    // Nguoi dung nham sang HANG KHAC trong luc doc cau hoi.
    history.aimHistoryVersion(202)

    history.confirmPendingRestore()
    await vi.waitFor(() => expect(history.historyPendingRestore.value).toBeNull())

    // 🔴 Phien ban duoc khoi phuc phai la 201 (cai dang CHO), khong phai 202 (cai dang NHAM):
    //    khoi phuc mot hang khac voi loi dong y danh cho hang nay la bien mot loi xac nhan
    //    thanh mot giay phep ghi bat ky dau.
    expect(restoreCalls.map((c) => [c.versionId, c.force])).toEqual([
      [201, false],
      [201, true],
    ])
  })

  it('giữ bản đang soạn ⇒ bỏ câu hỏi và KHÔNG một lượt IPC nào nữa', async () => {
    const { editor, history } = await freshState()
    editor.setEditorCaret(11)
    history.openSegmentHistory()
    await vi.waitFor(() => expect(history.historyPending.value).toBe(false))

    holdNextRestore.value = true
    await history.restoreVersion(201, false)
    const callsBefore = restoreCalls.length

    history.cancelPendingRestore()

    expect(history.historyPendingRestore.value).toBeNull()
    expect(restoreCalls.length).toBe(callsBefore)
  })

  it('đóng lớp phủ cũng bỏ câu hỏi đang chờ', async () => {
    const { editor, history } = await freshState()
    editor.setEditorCaret(11)
    history.openSegmentHistory()
    await vi.waitFor(() => expect(history.historyPending.value).toBe(false))

    holdNextRestore.value = true
    await history.restoreVersion(201, false)

    history.closeSegmentHistory()

    expect(history.historyIsOpen.value).toBe(false)
    expect(history.historyPendingRestore.value).toBeNull()
  })
})

describe('④ FLUSH chạy TRƯỚC lượt khôi phục', () => {
  /**
   * 🔴 Chốt chống mất bản nháp chạy ở **Rust** và nó so trên **ĐĨA**. `editorEditedText` có
   * thể còn giữ ký tự chưa xuống WAL (AD-35: idle 2 s, trần cứng 5 s) ⇒ không flush trước
   * thì chốt so với một bản **cũ hơn thứ người dùng đang nhìn**, và nó sẽ **không hỏi** ở
   * đúng ca cần hỏi nhất — ca người dùng vừa gõ xong một đoạn và chưa ký.
   *
   * ⚠️ Mệnh đề này **không cưỡng chế được ở tầng Rust**: lệnh khôi phục chỉ đọc thứ đã ở
   * trên đĩa và không biết gì về văn bản đang gõ trong webview. Ca này là lưới **duy nhất**.
   */
  it('gõ chưa flush ⇒ lượt flush đi TRƯỚC lượt khôi phục trên dây', async () => {
    const { editor, history } = await freshState()
    editor.setEditorCaret(11)
    history.openSegmentHistory()
    await vi.waitFor(() => expect(history.historyPending.value).toBe(false))

    callOrder.length = 0
    editor.noteEditorEdit(11, 'Vua go, chua flush.')

    await history.restoreVersion(201, false)

    expect(callOrder).toEqual(['flush', 'restore'])
  })

  it('không câu nào đang xem ⇒ `no-segment`, và KHÔNG một lượt IPC nào', async () => {
    const { editor, history } = await freshState()
    editor.setEditorCaret(null)
    history.openSegmentHistory()

    expect(await history.restoreVersion(201, false)).toBe('no-segment')
    expect(restoreCalls).toEqual([])
  })
})

describe('⑤ hàng đang nhắm — khuôn `aimedShortcutRow` của Story 1.21', () => {
  it('khôi phục hàng đang nhắm đi qua đúng `version_id` đó', async () => {
    const { editor, history } = await freshState()
    editor.setEditorCaret(11)
    history.openSegmentHistory()
    await vi.waitFor(() => expect(history.historyPending.value).toBe(false))

    history.aimHistoryVersion(201)
    history.restoreAimedVersion()
    await vi.waitFor(() => expect(restoreCalls.length).toBe(1))

    expect(restoreCalls[0]).toEqual({ segmentId: 11, versionId: 201, force: false })
  })

  /** *"Hàm chạy từ một hợp âm bàn phím KHÔNG BAO GIỜ ném — nó KÊU."* */
  it('chưa nhắm hàng nào ⇒ không ném, không một lượt IPC nào', async () => {
    const { editor, history } = await freshState()
    editor.setEditorCaret(11)
    history.openSegmentHistory()
    await vi.waitFor(() => expect(history.historyPending.value).toBe(false))

    history.aimHistoryVersion(null)
    expect(() => history.restoreAimedVersion()).not.toThrow()
    expect(restoreCalls).toEqual([])
  })

  it('mở lại lịch sử xoá sạch câu hỏi đang chờ và mọi ô trạng thái cũ', async () => {
    const { editor, history } = await freshState()
    editor.setEditorCaret(11)
    history.openSegmentHistory()
    await vi.waitFor(() => expect(history.historyPending.value).toBe(false))

    holdNextRestore.value = true
    await history.restoreVersion(201, false)
    expect(history.historyPendingRestore.value).not.toBeNull()

    history.openSegmentHistory()

    expect(history.historyPendingRestore.value).toBeNull()
    expect(history.historyRestoreNotice.value).toBeNull()
    expect(history.historyRestoreError.value).toBeNull()
  })
})
