/**
 * `setCurrentSegmentOmitted` — Story 2.5c, AC1 · AC2 · AC4 · FR133.
 *
 * ─────────────────────────────────────────────────────────────────────────────
 * 🔴 MỆNH ĐỀ NÀO THUỘC TỆP NÀY, VÀ MỆNH ĐỀ NÀO **KHÔNG**
 * ─────────────────────────────────────────────────────────────────────────────
 * *"Cắt bỏ không chạm `status`/`target_text` trên ĐĨA"* là mệnh đề của **Rust**, và nó đã có
 * chủ ở `segment_contract.rs::omitting_a_segment_touches_the_flag_and_nothing_else`. Đừng
 * dựng lại nó ở đây — hai đường cùng canh một mệnh đề là hai nguồn sự thật.
 *
 * Thứ **chỉ** tệp này canh được: ảnh chụp hiển thị phía webview. Cụ thể là ba chỗ mà không
 * cổng nào khác nhìn thấy:
 *   ① lượt gọi đi ra mang **đúng** `segment.id` đang có caret và **đúng** cờ;
 *   ② `editorSegments` được thay bằng một **mảng mới** — `shallowRef` không theo dõi sửa tại
 *     chỗ, nên một `segments.value[i].is_omitted = true` sẽ **không** render lại (đường hỏng
 *     mà `confirmCurrentSegment` đã ghi bằng chữ ở `editorPanelState.ts`);
 *   ③ một lượt bị **từ chối** KHÔNG được đổi ảnh chụp — nếu không, lưới nói dối về đĩa.
 *
 * ⚠️ **KHÔNG** flush trước, khác hẳn `confirmCurrentSegment`. Lệnh này không đọc `target_text`
 * một chữ nào, nên thứ tự với đường flush là **không quan trọng** — và ca ④ đo đúng điều đó
 * thay vì để nó thành một giả định.
 */
import { beforeEach, describe, expect, it, vi } from 'vitest'
import { readFixture, recordSave, resetRecorder, saveCalls } from './support/segmentFixture'

/** Đối số của mỗi lượt `setSegmentOmitted`, theo đúng thứ tự. */
const omitCalls: { segmentId: number; omitted: boolean }[] = []
/** Bật để lượt `setSegmentOmitted` kế tiếp trả về một lỗi từ chối. */
const failNextOmit = { value: false }

async function recordOmit(segmentId: number, omitted: boolean) {
  omitCalls.push({ segmentId, omitted })
  if (failNextOmit.value) {
    failNextOmit.value = false
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
  return { outcome: { segment_id: segmentId, is_omitted: omitted }, error: null }
}

vi.mock('../../src/config/segment', async (importOriginal) => {
  const actual = await importOriginal<typeof import('../../src/config/segment')>()
  return {
    ...actual,
    readOpenChapterSegments: readFixture,
    saveSegmentTargets: recordSave,
    setSegmentOmitted: recordOmit,
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
  omitCalls.length = 0
  failNextOmit.value = false
})

describe('① lượt gọi đi ra mang đúng câu và đúng cờ', () => {
  it('cắt bỏ câu đang có caret', async () => {
    const state = await freshState()
    state.setEditorCaret(12)

    expect(await state.setCurrentSegmentOmitted(true)).toBe('omitted')
    expect(omitCalls).toEqual([{ segmentId: 12, omitted: true }])
  })

  it('bỏ cờ dùng CÙNG một đường, chỉ khác giá trị', async () => {
    const state = await freshState()
    state.setEditorCaret(12)

    expect(await state.setCurrentSegmentOmitted(false)).toBe('restored')
    expect(omitCalls).toEqual([{ segmentId: 12, omitted: false }])
  })

  /**
   * 🔴 *"Hàm chạy từ một hợp âm bàn phím KHÔNG BAO GIỜ ném — nó KÊU."* Không câu nào đang
   * chạm là một trạng thái **bình thường** của panel, không một lỗi.
   */
  it('không câu nào đang có caret ⇒ `no-caret`, và KHÔNG một lượt IPC nào', async () => {
    const state = await freshState()
    state.setEditorCaret(null)

    expect(await state.setCurrentSegmentOmitted(true)).toBe('no-caret')
    expect(omitCalls).toEqual([])
  })
})

describe('② ảnh chụp hiển thị — MẢNG MỚI, không sửa tại chỗ', () => {
  /**
   * 🔴 `editorSegments` là một `shallowRef`. Sửa tại chỗ (`segments.value[i].is_omitted = true`)
   * **không** bắn watcher nào ⇒ hàng đã cắt bỏ không bao giờ gạch ngang trên màn hình, trong
   * khi đĩa thì đã đổi. Ca này so **danh tính tham chiếu** chứ không so nội dung, vì đúng cái
   * hỏng ở đây là danh tính.
   *
   * Chạy đỏ-rồi-xanh: đổi thân hàm sang sửa tại chỗ, ca này phải ĐỎ.
   */
  it('mảng đổi danh tính, và đúng một phần tử đổi nội dung', async () => {
    const state = await freshState()
    const before = state.editorSegments.value
    state.setEditorCaret(12)

    await state.setCurrentSegmentOmitted(true)

    const after = state.editorSegments.value
    expect(after).not.toBe(before)
    expect(after.find((s) => s.id === 12)?.is_omitted).toBe(true)
    expect(after.filter((s) => s.is_omitted).map((s) => s.id)).toEqual([12])
  })

  /**
   * 🔴 **AC2 nói bằng ngôn ngữ của ảnh chụp:** lượt cắt bỏ chỉ được đổi **một** trường. Nếu
   * nó dựng lại phần tử từ `outcome` thay vì trải phần tử cũ, `status` và `target_text` sẽ
   * bốc hơi khỏi hàng đó — và triệu chứng là *"xác nhận biến mất khi cắt bỏ"*, một thứ không
   * cổng Rust nào bắt được vì đĩa vẫn đúng.
   */
  it('`status` và `target_text` của chính câu đó KHÔNG đổi', async () => {
    const state = await freshState()
    const before = state.editorSegments.value.find((s) => s.id === 11)
    state.setEditorCaret(11)

    await state.setCurrentSegmentOmitted(true)

    const after = state.editorSegments.value.find((s) => s.id === 11)
    expect(after?.status).toBe(before?.status)
    expect(after?.target_text).toBe(before?.target_text)
    expect(before?.status).toBe('confirmed')
  })

  /** AC4 ở tầng hiển thị — một vòng cắt bỏ rồi bỏ cờ đưa hàng về đúng chỗ cũ. */
  it('cắt bỏ rồi bỏ cờ ⇒ ảnh chụp khớp lại từng trường', async () => {
    const state = await freshState()
    const before = { ...state.editorSegments.value.find((s) => s.id === 11) }
    state.setEditorCaret(11)

    await state.setCurrentSegmentOmitted(true)
    await state.setCurrentSegmentOmitted(false)

    expect({ ...state.editorSegments.value.find((s) => s.id === 11) }).toEqual(before)
  })
})

describe('③ một lượt bị TỪ CHỐI không được đổi ảnh chụp', () => {
  /**
   * 🔴 Đây là lớp lỗi *"giao diện nói dối về đĩa"*: Rust từ chối *(câu đã về hưu — AD-5)*
   * nhưng lưới vẫn gạch ngang. Người dùng tin câu đã bị cắt bỏ, và nó vẫn nằm trong mọi
   * bản xuất.
   */
  it('Rust từ chối ⇒ `refused`, ảnh chụp Y NGUYÊN, lỗi đọc được', async () => {
    const state = await freshState()
    state.setEditorCaret(12)
    const before = state.editorSegments.value
    failNextOmit.value = true

    expect(await state.setCurrentSegmentOmitted(true)).toBe('refused')

    expect(state.editorSegments.value).toBe(before)
    expect(state.editorOmitError.value?.message_key).toBe('err.segment.retired')
  })

  it('một lượt thành công sau đó dọn lỗi cũ', async () => {
    const state = await freshState()
    state.setEditorCaret(12)
    failNextOmit.value = true
    await state.setCurrentSegmentOmitted(true)
    expect(state.editorOmitError.value).not.toBeNull()

    await state.setCurrentSegmentOmitted(true)
    expect(state.editorOmitError.value).toBeNull()
  })
})

describe('④ KHÔNG flush trước — và đây là một phép ĐO, không một giả định', () => {
  /**
   * 🔴 `confirmCurrentSegment` **phải** flush trước (AD-35 vế (c)) vì nó ký **văn bản trên
   * đĩa**. Lệnh này không đọc `target_text` một chữ nào và câu `UPDATE` của nó chạm **đúng
   * một cột**, nên bắt người dùng chờ một lượt IPC nữa là một cái giá không ai trả cho cái gì.
   *
   * ⚠️ Ca này khẳng định **không có** lượt `saveSegmentTargets` nào được phát — tức nó cũng
   * là lưới cho mệnh đề ngược lại: nếu ai đó thêm một lượt flush "cho chắc" vào đây, ca này
   * đỏ và người đó phải viết ra lý do.
   */
  it('văn bản đang gõ vẫn nằm trong tập chờ, không lượt lưu nào bị ép chạy', async () => {
    const state = await freshState()
    state.setEditorCaret(12)
    state.noteEditorEdit(12, 'Chữ đang gõ, chưa flush.')

    expect(await state.setCurrentSegmentOmitted(true)).toBe('omitted')

    expect(saveCalls).toEqual([])
    expect(omitCalls).toEqual([{ segmentId: 12, omitted: true }])
  })
})
