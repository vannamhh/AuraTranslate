/**
 * State của dải "Chờ chốt lần đầu gặp" — Story 3.6, FR114.
 *
 * ─────────────────────────────────────────────────────────────────────────────
 * ⚠️ PHẠM VI — mock `@tauri-apps/api/core` Ở ĐÚNG BIÊN IPC, khuôn `bootstrap.test.ts:18-21`
 * ─────────────────────────────────────────────────────────────────────────────
 * KHÔNG mock trọn `config/glossary.ts` (khuôn `glossaryQuickAdd.test.ts`): hàm đang kiểm
 * (`confirmGlossaryConfirmStrip`) gọi HAI adapter khác nhau của cùng module đó
 * (`confirmPendingGlossaryTranslation` + gián tiếp `glossaryMarksForChapter` qua
 * `panels/glossaryMarksState.ts::refreshGlossaryMarks`) — mock ở tầng `invoke()` cho phép
 * bộ phân giải/type-guard THẬT của `config/glossary.ts` chạy qua cả hai đường cùng lúc,
 * đúng khuôn `bootstrap.test.ts`/`glossaryMarksRefresh.test.ts`.
 *
 * `watch(editorCaretSegmentId, …)` THẬT sống trong `GlossaryConfirmStrip.vue` (một LEAF —
 * xem doc-comment đầu `glossaryConfirmStripState.ts`), không trong module đang kiểm ở đây.
 * Bộ test này gọi thẳng `syncGlossaryConfirmStripTarget(...)` để mô phỏng đúng những gì
 * watcher đó sẽ gọi — cùng khuôn `glossaryMarksMap.test.ts` kiểm hàm thuần không cần mount
 * component thật.
 */
import { beforeEach, describe, expect, it, vi } from 'vitest'

const mockInvoke = vi.fn()
vi.mock('@tauri-apps/api/core', () => ({ invoke: (...args: unknown[]) => mockInvoke(...args) }))

async function freshState() {
  vi.resetModules()
  mockInvoke.mockReset()
  const state = await import('../../src/glossaryConfirmStripState')
  const marksState = await import('../../src/panels/glossaryMarksState')
  return { state, marksState }
}

function seg(id: number, sourceText: string) {
  return { id, source_text: sourceText }
}

function markWire(opts: {
  start: number
  end: number
  id: number
  sourceTerm: string
  tier?: 'global' | 'work'
  isConfirmed?: boolean
  translation?: string | null
}) {
  return {
    start: opts.start,
    end: opts.end,
    tier: opts.tier ?? 'global',
    is_confirmed: opts.isConfirmed ?? false,
    translation: opts.translation ?? null,
    id: opts.id,
    source_term: opts.sourceTerm,
  }
}

beforeEach(() => {
  document.body.innerHTML = ''
})

describe('syncGlossaryConfirmStripTarget — chọn span chờ chốt trái nhất', () => {
  it('câu có MỘT thuật ngữ chờ chốt ⇒ dải mọc, mang source_term và tầng; KHÔNG tự focus', async () => {
    const { state } = await freshState()
    const segments = [seg(1, '萧炎登场')]
    const mark = markWire({ start: 0, end: 2, id: 7, sourceTerm: '萧炎', tier: 'work' })

    state.syncGlossaryConfirmStripTarget(1, segments, true, [mark])

    expect(state.confirmStripIsOpen.value).toBe(true)
    expect(state.confirmStripSourceTerm.value).toBe('萧炎')
    expect(state.confirmStripTier.value).toBe('work')
    // "Dải không cướp tiêu điểm khi mọc" — `sync` một mình không được kích một lượt focus.
    expect(state.confirmStripFocusRequest.value).toBe(0)
  })

  it('câu không có thuật ngữ chờ chốt (0 span, hoặc mọi span đã chốt) ⇒ 0 dải', async () => {
    const { state } = await freshState()
    const segments = [seg(1, '你好')]

    state.syncGlossaryConfirmStripTarget(1, segments, true, [])
    expect(state.confirmStripIsOpen.value).toBe(false)

    const confirmed = markWire({ start: 0, end: 2, id: 9, sourceTerm: '你好', isConfirmed: true, translation: 'chao' })
    state.syncGlossaryConfirmStripTarget(1, segments, true, [confirmed])
    expect(state.confirmStripIsOpen.value).toBe(false)
    expect(mockInvoke).not.toHaveBeenCalled()
  })

  it('marks CHƯA nạp xong (marksHaveLoaded=false) ⇒ 0 dải — chưa biết thì không khẳng định "không có"', async () => {
    const { state } = await freshState()
    const segments = [seg(1, '萧炎登场')]
    const mark = markWire({ start: 0, end: 2, id: 7, sourceTerm: '萧炎' })

    state.syncGlossaryConfirmStripTarget(1, segments, false, [mark])

    expect(state.confirmStripIsOpen.value).toBe(false)
  })

  it('câu có HAI thuật ngữ chờ chốt ⇒ hỏi span TRÁI NHẤT trước', async () => {
    const { state } = await freshState()
    const segments = [seg(1, '萧炎和薰儿登场')]
    const markLeft = markWire({ start: 0, end: 2, id: 7, sourceTerm: '萧炎' })
    const markRight = markWire({ start: 3, end: 5, id: 8, sourceTerm: '薰儿' })

    state.syncGlossaryConfirmStripTarget(1, segments, true, [markRight, markLeft])

    expect(state.confirmStripSourceTerm.value).toBe('萧炎')
  })
})

describe('confirmGlossaryConfirmStrip — chốt bản dịch', () => {
  it('chuỗi rỗng/toàn khoảng trắng ⇒ 0 lượt IPC, dải Ở LẠI kèm lỗi qua t()', async () => {
    const { state } = await freshState()
    const segments = [seg(1, '萧炎登场')]
    state.syncGlossaryConfirmStripTarget(1, segments, true, [markWire({ start: 0, end: 2, id: 7, sourceTerm: '萧炎' })])

    state.confirmStripTranslationInput.value = '   '
    const ok = await state.confirmGlossaryConfirmStrip(1, segments, 'zh')

    expect(ok).toBe(false)
    expect(state.confirmStripIsOpen.value).toBe(true)
    expect(state.confirmStripEmptyInputError.value).toBe(true)
    expect(mockInvoke).not.toHaveBeenCalled()
  })

  it('lượt ghi TRƯỢT ⇒ dải Ở LẠI kèm lỗi đọc được (IpcError), KHÔNG đóng', async () => {
    const { state } = await freshState()
    const segments = [seg(1, '萧炎登场')]
    state.syncGlossaryConfirmStripTarget(1, segments, true, [markWire({ start: 0, end: 2, id: 7, sourceTerm: '萧炎' })])

    const err = { code: 'store.write_failed', message_key: 'err.store.write_failed', params: {}, retryable: false }
    mockInvoke.mockImplementation((cmd: string) => {
      if (cmd === 'glossary_confirm_pending_translation') return Promise.reject(err)
      return Promise.reject(new Error(`lenh khong mong doi: ${cmd}`))
    })

    state.confirmStripTranslationInput.value = 'Tieu Viem'
    const ok = await state.confirmGlossaryConfirmStrip(1, segments, 'zh')

    expect(ok).toBe(false)
    expect(state.confirmStripIsOpen.value).toBe(true)
    expect(state.confirmStripSaveError.value).toEqual(err)
  })

  it('Chốt xong: ghi thành công, marks nạp lại, và câu có mục THỨ HAI ⇒ dải kế mọc CÙNG SLOT, không thao tác nào', async () => {
    const { state, marksState } = await freshState()
    const segments = [seg(1, '萧炎和薰儿登场')]
    const markLeft = markWire({ start: 0, end: 2, id: 7, sourceTerm: '萧炎' })
    const markRight = markWire({ start: 3, end: 5, id: 8, sourceTerm: '薰儿' })
    state.syncGlossaryConfirmStripTarget(1, segments, true, [markLeft, markRight])
    expect(state.confirmStripSourceTerm.value).toBe('萧炎')

    mockInvoke.mockImplementation((cmd: string) => {
      if (cmd === 'glossary_confirm_pending_translation') return Promise.resolve(undefined)
      if (cmd === 'glossary_marks_for_chapter') {
        return Promise.resolve([
          { ...markLeft, is_confirmed: true, translation: 'Tieu Viem' },
          markRight,
        ])
      }
      return Promise.reject(new Error(`lenh khong mong doi: ${cmd}`))
    })

    state.confirmStripTranslationInput.value = 'Tieu Viem'
    const ok = await state.confirmGlossaryConfirmStrip(1, segments, 'zh')
    expect(ok).toBe(true)

    // `refreshGlossaryMarks` bên trong hàm là `void` (không chờ) — cùng khuôn
    // `glossaryQuickAddState.ts::refreshGlossaryMarksAfterSave`. Đợi vài vi-tác vụ để lượt
    // làm mới đó chạy hết vòng IPC giả lập rồi mới đọc lại `glossaryMarks`.
    await Promise.resolve()
    await Promise.resolve()
    await Promise.resolve()

    expect(marksState.glossaryMarksHaveLoaded()).toBe(true)

    // Component thật re-sync qua `watch(glossaryMarks, …)` — mô phỏng đúng bước đó.
    state.syncGlossaryConfirmStripTarget(1, segments, marksState.glossaryMarksHaveLoaded(), marksState.glossaryMarks.value)

    expect(state.confirmStripSourceTerm.value).toBe('薰儿')
  })

  it('lượt ghi ĐANG BAY, `watch` đổi identity giữa chừng (caret rời câu), rồi LỖI về MUỘN ⇒ KHÔNG dán `saveError` lên mục MỚI', async () => {
    const { state } = await freshState()
    const segments = [seg(1, '萧炎登场')]
    const mark = markWire({ start: 0, end: 2, id: 7, sourceTerm: '萧炎' })
    state.syncGlossaryConfirmStripTarget(1, segments, true, [mark])

    let rejectConfirm: (err: unknown) => void = () => {}
    const pending = new Promise((_resolve, reject) => {
      rejectConfirm = reject
    })
    mockInvoke.mockImplementation((cmd: string) => {
      if (cmd === 'glossary_confirm_pending_translation') return pending
      return Promise.reject(new Error(`lenh khong mong doi: ${cmd}`))
    })

    state.confirmStripTranslationInput.value = 'Tieu Viem'
    const confirming = state.confirmGlossaryConfirmStrip(1, segments, 'zh') // KHONG await -- dang bay.

    // Trong luc dang bay: caret roi cau nay -- mo phong dung thu `watch` thi lam trong component.
    state.syncGlossaryConfirmStripTarget(null, segments, true, [])
    expect(state.confirmStripIsOpen.value).toBe(false)

    const err = { code: 'store.write_failed', message_key: 'err.store.write_failed', params: {}, retryable: false }
    rejectConfirm(err)
    const ok = await confirming

    expect(ok).toBe(false)
    expect(state.confirmStripSaveError.value).toBe(null) // loi cua muc CU khong duoc dan len muc moi
    expect(state.confirmStripIsOpen.value).toBe(false) // van dung o trang thai da doi
  })

  it('lượt ghi ĐANG BAY, `watch` đổi identity giữa chừng, rồi THÀNH CÔNG về MUỘN ⇒ vẫn `refreshGlossaryMarks` (bản dịch đã xuống đĩa thật, không được nuốt lượt làm mới)', async () => {
    // ⚠️ SỬA 2026-08-22 (rà ba lớp, vòng 2) — bản trước dùng `document.activeElement` để
    // chứng minh `restoreFocusAndSelection()` bị bỏ qua. Sau khi `applyTarget` (đúng chỗ
    // `sequence` bump) TỰ dọn `savedFocusEl`/`savedRange`/`enteredViaChord` khi danh tính đổi
    // — vá cùng lượt này — `restoreFocusAndSelection()` đã LUÔN là no-op ngay tại thời điểm
    // identity lệch, bất kể `stillCurrentTarget` có canh hay không; quan sát qua focus không
    // còn PHÂN BIỆT được có vệ hay không. Phép đối chứng đỏ→xanh THẬT của vệ `sequence` nay
    // sống ở ca "...rồi LỖI về MUỘN ⇒ KHÔNG dán `saveError` lên mục MỚI" ngay trên (quan sát
    // `confirmStripSaveError`, KHÔNG dính tới ba biến focus). Ca dưới đây giữ lại đúng phần
    // còn PHÂN BIỆT được của nhánh thành công: `refreshGlossaryMarks` phải chạy vô điều kiện.
    const { state, marksState } = await freshState()
    const segments = [seg(1, '萧炎登场')]
    const mark = markWire({ start: 0, end: 2, id: 7, sourceTerm: '萧炎' })
    state.syncGlossaryConfirmStripTarget(1, segments, true, [mark])

    let resolveConfirm: (v: unknown) => void = () => {}
    const pendingConfirm = new Promise((resolve) => {
      resolveConfirm = resolve
    })
    mockInvoke.mockImplementation((cmd: string) => {
      if (cmd === 'glossary_confirm_pending_translation') return pendingConfirm
      if (cmd === 'glossary_marks_for_chapter') {
        return Promise.resolve([{ ...mark, is_confirmed: true, translation: 'Tieu Viem' }])
      }
      return Promise.reject(new Error(`lenh khong mong doi: ${cmd}`))
    })

    state.confirmStripTranslationInput.value = 'Tieu Viem'
    const confirming = state.confirmGlossaryConfirmStrip(1, segments, 'zh')

    // Trong luc dang bay: caret roi cau nay -- mo phong dung thu `watch` thi lam trong component.
    state.syncGlossaryConfirmStripTarget(null, segments, true, [])
    expect(state.confirmStripIsOpen.value).toBe(false)

    resolveConfirm(undefined)
    const ok = await confirming
    expect(ok).toBe(true)

    await Promise.resolve()
    await Promise.resolve()
    await Promise.resolve()
    expect(marksState.glossaryMarksHaveLoaded()).toBe(true) // ban dich DA xuong dia that -- van phai nap lai marks
  })

  it('🔴 chord → đổi mục qua `sync` → chord LẦN HAI cho mục MỚI → Lưu ⇒ tiêu điểm trả về ô của mục MỚI, KHÔNG phải ô của mục CŨ (Ice bắt, rà ba lớp vòng 2)', async () => {
    const { state } = await freshState()
    const segmentA = [seg(1, '萧炎登场')]
    const markA = markWire({ start: 0, end: 2, id: 7, sourceTerm: '萧炎' })
    state.syncGlossaryConfirmStripTarget(1, segmentA, true, [markA])
    expect(state.confirmStripSourceTerm.value).toBe('萧炎')

    // Vào dải bằng hợp âm cho mục A -- luu tieu diem CU (`anchor`, o CAU A) lam `savedFocusEl`.
    const anchorForA = document.createElement('button')
    document.body.appendChild(anchorForA)
    anchorForA.focus()
    expect(state.focusGlossaryConfirmStrip('', true)).toBe(true)
    expect(state.confirmStripFocusRequest.value).toBe(1)

    // Caret roi cau A, sang cau B -- watch bat, doi identity SANG MUC KHAC ma KHONG di qua
    // `deferGlossaryConfirmStrip`/`confirmGlossaryConfirmStrip`.
    const segmentB = [seg(2, '薰儿登场')]
    const markB = markWire({ start: 0, end: 2, id: 8, sourceTerm: '薰儿' })
    state.syncGlossaryConfirmStripTarget(2, segmentB, true, [markB])
    expect(state.confirmStripSourceTerm.value).toBe('薰儿')

    // Tieu diem THAT (DOM) da roi khoi `anchorForA` -- mo phong dung nguoi dung dang go o mot
    // o KHAC (o cua cau B) truoc khi bam hop am lan hai.
    const anchorForB = document.createElement('button')
    document.body.appendChild(anchorForB)
    anchorForB.focus()

    // Hop am LAN HAI, cho muc MOI (B). Neu `enteredViaChord`/`savedFocusEl` con sot lai cua A
    // (loi da vá), nhanh nay se doc nham la "chot tai nhap" va KHONG luu `anchorForB`.
    expect(state.focusGlossaryConfirmStrip('', true)).toBe(true)
    expect(state.confirmStripFocusRequest.value).toBe(2) // van tang -- khong bi nhanh tai nhap nuot.

    mockInvoke.mockImplementation((cmd: string) => {
      if (cmd === 'glossary_confirm_pending_translation') return Promise.resolve(undefined)
      if (cmd === 'glossary_marks_for_chapter') return Promise.resolve([{ ...markB, is_confirmed: true, translation: 'Huan Nhi' }])
      return Promise.reject(new Error(`lenh khong mong doi: ${cmd}`))
    })

    state.confirmStripTranslationInput.value = 'Huan Nhi'
    const ok = await state.confirmGlossaryConfirmStrip(2, segmentB, 'zh')
    expect(ok).toBe(true)

    // Mệnh đề trung tâm: tiêu điểm trả về đúng `anchorForB` (ô của mục MỚI, B) -- KHÔNG nhảy
    // ngược về `anchorForA` (ô của mục CŨ, A) như lỗi đã bắt.
    expect(document.activeElement).toBe(anchorForB)
  })
})

describe('focusGlossaryConfirmStrip — điền sẵn ô nhập bằng vùng chọn (cột bản dịch), nếu có', () => {
  it('vào dải với một vùng chọn hiện có ⇒ ô nhập điền sẵn đúng chuỗi đó', async () => {
    const { state } = await freshState()
    const segments = [seg(1, '萧炎登场')]
    state.syncGlossaryConfirmStripTarget(1, segments, true, [markWire({ start: 0, end: 2, id: 7, sourceTerm: '萧炎' })])

    state.focusGlossaryConfirmStrip('Tieu Viem', true)

    expect(state.confirmStripTranslationInput.value).toBe('Tieu Viem')
    expect(state.confirmStripFocusRequest.value).toBe(1)
  })

  it('vào dải KHÔNG có vùng chọn ⇒ ô nhập vẫn rỗng, không phải chuỗi rác', async () => {
    const { state } = await freshState()
    const segments = [seg(1, '萧炎登场')]
    state.syncGlossaryConfirmStripTarget(1, segments, true, [markWire({ start: 0, end: 2, id: 7, sourceTerm: '萧炎' })])

    state.focusGlossaryConfirmStrip('', true)

    expect(state.confirmStripTranslationInput.value).toBe('')
  })

  it('hợp âm bắn LẦN HAI trong khi đã ở trong dải (chốt tái nhập) ⇒ KHÔNG ghi đè chữ người dùng đang gõ', async () => {
    const { state } = await freshState()
    const segments = [seg(1, '萧炎登场')]
    state.syncGlossaryConfirmStripTarget(1, segments, true, [markWire({ start: 0, end: 2, id: 7, sourceTerm: '萧炎' })])

    state.focusGlossaryConfirmStrip('Tieu Viem', true)
    state.confirmStripTranslationInput.value = 'Tieu Viem, da sua tay'
    state.focusGlossaryConfirmStrip('mot vung chon khac hoan toan', true)

    expect(state.confirmStripTranslationInput.value).toBe('Tieu Viem, da sua tay')
    expect(state.confirmStripFocusRequest.value).toBe(2) // vẫn tang -- component vẫn phai focus lai
  })

  it('dải chốt KHÔNG PHẢI dải đang hiện (GlossaryQuickAdd thắng sổ ưu tiên) ⇒ KHÔNG chạm ô nhớ nào, KÊU, trả false', async () => {
    const { state } = await freshState()
    const segments = [seg(1, '萧炎登场')]
    state.syncGlossaryConfirmStripTarget(1, segments, true, [markWire({ start: 0, end: 2, id: 7, sourceTerm: '萧炎' })])
    expect(state.confirmStripIsOpen.value).toBe(true) // co mot muc dang cho hoi that

    const errorSpy = vi.spyOn(console, 'error').mockImplementation(() => {})
    const ok = state.focusGlossaryConfirmStrip('Tieu Viem', false) // isVisible=false: GlossaryQuickAdd dang thang

    expect(ok).toBe(false)
    expect(errorSpy).toHaveBeenCalledTimes(1)
    // KHONG cham mot o nho nao: o nhap van rong, chua co focusRequest nao, chua "vao" dai.
    expect(state.confirmStripTranslationInput.value).toBe('')
    expect(state.confirmStripFocusRequest.value).toBe(0)

    errorSpy.mockRestore()
  })

  it('0 mục đang chờ hỏi ⇒ trả false, không KÊU (đây là trạng thái BÌNH THƯỜNG, không phải lỗi)', async () => {
    const { state } = await freshState()
    const errorSpy = vi.spyOn(console, 'error').mockImplementation(() => {})

    const ok = state.focusGlossaryConfirmStrip('', true)

    expect(ok).toBe(false)
    expect(errorSpy).not.toHaveBeenCalled()

    errorSpy.mockRestore()
  })
})

describe('deferGlossaryConfirmStrip — "Để sau" (Esc)', () => {
  it('source_term đó KHÔNG hỏi lại trong Chương đang mở; tiêu điểm trả về ô gõ cũ', async () => {
    const anchor = document.createElement('button')
    document.body.appendChild(anchor)
    anchor.focus()

    const { state } = await freshState()
    const segments = [seg(1, '萧炎登场')]
    const mark = markWire({ start: 0, end: 2, id: 7, sourceTerm: '萧炎' })
    state.syncGlossaryConfirmStripTarget(1, segments, true, [mark])

    state.focusGlossaryConfirmStrip('', true) // "vào dải bằng hợp âm" — lưu tiêu điểm cũ.
    const insideStrip = document.createElement('input')
    document.body.appendChild(insideStrip)
    insideStrip.focus()

    state.deferGlossaryConfirmStrip()

    expect(state.confirmStripIsOpen.value).toBe(false)
    expect(document.activeElement).toBe(anchor)

    // Cùng câu, cùng mark ⇒ vẫn KHÔNG hỏi lại — sổ "Để sau" chặn.
    state.syncGlossaryConfirmStripTarget(1, segments, true, [mark])
    expect(state.confirmStripIsOpen.value).toBe(false)
  })

  it('câu còn một mục CHỜ CHỐT KHÁC ⇒ dải kế mọc NGAY sau khi Để sau, cùng slot', async () => {
    const { state } = await freshState()
    const segments = [seg(1, '萧炎和薰儿登场')]
    const markLeft = markWire({ start: 0, end: 2, id: 7, sourceTerm: '萧炎' })
    const markRight = markWire({ start: 3, end: 5, id: 8, sourceTerm: '薰儿' })
    state.syncGlossaryConfirmStripTarget(1, segments, true, [markLeft, markRight])
    expect(state.confirmStripSourceTerm.value).toBe('萧炎')

    state.deferGlossaryConfirmStrip()

    expect(state.confirmStripIsOpen.value).toBe(true)
    expect(state.confirmStripSourceTerm.value).toBe('薰儿')
  })

  it('BỊ CHẶN trong lúc một lượt ghi CỦA CHÍNH MỤC ĐÓ đang bay — cùng luật với nút "Để sau" đã `:disabled` trong `<fieldset>`', async () => {
    const { state } = await freshState()
    const segments = [seg(1, '萧炎登场')]
    const mark = markWire({ start: 0, end: 2, id: 7, sourceTerm: '萧炎' })
    state.syncGlossaryConfirmStripTarget(1, segments, true, [mark])

    let resolveConfirm: (v: unknown) => void = () => {}
    const pending = new Promise((resolve) => {
      resolveConfirm = resolve
    })
    mockInvoke.mockImplementation((cmd: string) => {
      if (cmd === 'glossary_confirm_pending_translation') return pending
      return Promise.reject(new Error(`lenh khong mong doi: ${cmd}`))
    })

    state.confirmStripTranslationInput.value = 'Tieu Viem'
    const confirming = state.confirmGlossaryConfirmStrip(1, segments, 'zh') // dang bay.

    state.deferGlossaryConfirmStrip() // "Esc" trong luc dang luu -- phai KHONG co hieu luc.

    expect(state.confirmStripIsOpen.value).toBe(true)
    expect(state.confirmStripSourceTerm.value).toBe('萧炎') // van la muc dang luu, chua bi "De sau"

    resolveConfirm(undefined)
    await confirming
  })
})

describe('resetGlossaryConfirmStrip — đổi Chương/Tác phẩm', () => {
  it('xoá sổ "Để sau" — cùng source_term hỏi lại được sau reset', async () => {
    const { state } = await freshState()
    const segments = [seg(1, '萧炎登场')]
    const mark = markWire({ start: 0, end: 2, id: 7, sourceTerm: '萧炎' })
    state.syncGlossaryConfirmStripTarget(1, segments, true, [mark])
    state.deferGlossaryConfirmStrip()
    expect(state.confirmStripIsOpen.value).toBe(false)

    state.resetGlossaryConfirmStrip()
    state.syncGlossaryConfirmStripTarget(1, segments, true, [mark])

    expect(state.confirmStripIsOpen.value).toBe(true)
    expect(state.confirmStripSourceTerm.value).toBe('萧炎')
  })

  it('dải đang mở ⇒ reset thu dải, KHÔNG để lại state của segment vừa về hưu', async () => {
    const { state } = await freshState()
    const segments = [seg(1, '萧炎登场')]
    const mark = markWire({ start: 0, end: 2, id: 7, sourceTerm: '萧炎' })
    state.syncGlossaryConfirmStripTarget(1, segments, true, [mark])
    expect(state.confirmStripIsOpen.value).toBe(true)

    state.resetGlossaryConfirmStrip()

    expect(state.confirmStripIsOpen.value).toBe(false)
    expect(state.confirmStripTranslationInput.value).toBe('')
    expect(state.confirmStripSaveError.value).toBe(null)
  })
})

describe('sổ ưu tiên — dải "Thêm thuật ngữ" đang mở thắng dải "Chờ chốt"', () => {
  it('cả hai cùng đủ điều kiện ⇒ topmostStrip chọn glossary_quick_add; dải chốt mọc lại khi dải kia đóng', async () => {
    const { state } = await freshState()
    const quickAdd = await import('../../src/glossaryQuickAddState')
    const { topmostStrip } = await import('../../src/panels/inlineStripPriority')

    const segments = [seg(1, '萧炎登场')]
    state.syncGlossaryConfirmStripTarget(1, segments, true, [markWire({ start: 0, end: 2, id: 7, sourceTerm: '萧炎' })])
    expect(state.confirmStripIsOpen.value).toBe(true)

    mockInvoke.mockImplementation((cmd: string) => {
      if (cmd === 'glossary_lookup_term') return Promise.resolve({ work_tier_available: false, entry: null })
      return Promise.reject(new Error(`lenh khong mong doi: ${cmd}`))
    })
    quickAdd.openGlossaryQuickAdd('')
    expect(quickAdd.quickAddIsOpen.value).toBe(true)

    const eligibleWithQuickAdd: ('glossary_quick_add' | 'glossary_confirm')[] = []
    if (quickAdd.quickAddIsOpen.value) eligibleWithQuickAdd.push('glossary_quick_add')
    if (state.confirmStripIsOpen.value) eligibleWithQuickAdd.push('glossary_confirm')
    expect(topmostStrip(eligibleWithQuickAdd)).toBe('glossary_quick_add')

    quickAdd.closeGlossaryQuickAdd()

    const eligibleAfterClose: ('glossary_quick_add' | 'glossary_confirm')[] = []
    if (quickAdd.quickAddIsOpen.value) eligibleAfterClose.push('glossary_quick_add')
    if (state.confirmStripIsOpen.value) eligibleAfterClose.push('glossary_confirm')
    expect(topmostStrip(eligibleAfterClose)).toBe('glossary_confirm')
  })
})
