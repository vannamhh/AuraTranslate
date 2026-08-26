/**
 * Story 3.3 · FR48 — Đối chứng HAI CHIỀU cho `currentSelectionTextForGlossaryQuickAdd`.
 *
 * ─────────────────────────────────────────────────────────────────────────────
 * 🔴 VÌ SAO ĐỐI CHỨNG HAI CHIỀU, KHÔNG MỘT CA DƯƠNG ĐƠN LẺ
 * ─────────────────────────────────────────────────────────────────────────────
 * `deferred-work.md:2687-2697` gọi mệnh đề *"vai `display` KHÔNG chặn đường của
 * `glossary.add_term`"* là **điều kiện khởi hành** của story 3.3: nó là lý do TOÀN BỘ hàm
 * `currentSelectionTextForGlossaryQuickAdd` tồn tại thay vì tái dùng `currentSelectionText`.
 * Một test chỉ khẳng định *"hàm mới trả về chữ ở một bề mặt `display`"* không chứng minh
 * được gì nếu hàm mới tình cờ lọc y hệt hàm cũ nhưng người viết test bôi đen nhầm một bề
 * mặt `source` — cổng vẫn xanh, và điều kiện khởi hành vẫn có thể đã vỡ trong im lặng.
 *
 * ⇒ Cùng khuôn `glossary_boundary.rs::the_non_manual_origin_token_check_catches_term_origin_but_not_candidate_origin`
 * (một vị từ, kiểm CẢ hai chiều: bắt đúng cái phải bắt, KHÔNG bắt cái không được bắt):
 *   ① bề mặt vai `'display'` ⇒ `currentSelectionText()` RỖNG, `…ForGlossaryQuickAdd()` CÓ CHỮ;
 *   ② bề mặt vai `'source'` ⇒ CẢ HAI đều CÓ CHỮ, và cùng một chuỗi.
 * Thiếu vế ①  thì một lượt "sửa" lỡ tái lọc theo `role` trong hàm mới đi lọt — đúng lỗi mà
 * `deferred-work.md:2691-2696` cảnh báo (*"gỡ đăng ký ⇒ SELECTION_SURFACE_FLOOR đỏ; lật vai
 * ⇒ Kiểm F ③ đỏ — nhưng LỌC lại theo role bên trong hàm mới thì KHÔNG cổng tĩnh nào bắt
 * được, chỉ hành vi lúc chạy mới bắt được"*). Thiếu vế ② thì một hàm luôn trả `''` (bị lỗi ở
 * chỗ khác) vẫn "qua" được vế ① một cách vô nghĩa.
 *
 * Dựng bề mặt DOM theo đúng khuôn `editorAutoLookup.test.ts::mountEditorSurface` — cùng
 * tiền lệ, cùng lý do (một hộp chứa text node thật, đăng ký qua `registerSelectionSurface`).
 */
import { beforeEach, describe, expect, it, vi } from 'vitest'
import {
  currentSelectionText,
  currentSelectionTextForGlossaryQuickAdd,
  registerSelectionSurface,
} from '../../src/panels/selectionContract'

beforeEach(() => {
  document.body.innerHTML = ''
  window.getSelection()?.removeAllRanges()
})

/** Dựng một bề mặt DOM tối thiểu mang một vai đã cho — cùng khuôn `mountEditorSurface`. */
function mountSurface(role: 'source' | 'display'): {
  sentence: HTMLElement
  release: () => void
} {
  const box = document.createElement('div')
  box.tabIndex = 0
  const sentence = document.createElement('span')
  sentence.textContent = 'Mộ Dung Phục là một nhân vật.'
  box.append(sentence)
  document.body.append(box)

  const release = registerSelectionSurface(box, role)
  return { sentence, release }
}

/** Bôi đen một khoảng chữ THẬT, neo vào text node — đúng vị từ `surfaceFor`. */
function selectInside(sentence: HTMLElement, start: number, end: number): void {
  const text = sentence.firstChild
  if (text === null) throw new Error('câu không có text node')
  const range = document.createRange()
  range.setStart(text, start)
  range.setEnd(text, end)
  const selection = window.getSelection()
  selection?.removeAllRanges()
  selection?.addRange(range)
}

describe('currentSelectionTextForGlossaryQuickAdd — đối chứng hai chiều với currentSelectionText', () => {
  it('① bề mặt vai `display` (Panel Lookup / AI Translation / cột bản dịch): currentSelectionText() RỖNG, đường RIÊNG có chữ', () => {
    const { sentence, release } = mountSurface('display')
    selectInside(sentence, 0, 9) // "Mộ Dung P"

    expect(currentSelectionText()).toBe('')
    expect(currentSelectionTextForGlossaryQuickAdd()).toBe('Mộ Dung P')

    release()
  })

  it('② bề mặt vai `source` (cột nguyên văn): CẢ HAI đường đều có chữ, và CÙNG một chuỗi', () => {
    const { sentence, release } = mountSurface('source')
    selectInside(sentence, 0, 9)

    const viaOldPath = currentSelectionText()
    const viaNewPath = currentSelectionTextForGlossaryQuickAdd()

    expect(viaOldPath).toBe('Mộ Dung P')
    expect(viaNewPath).toBe('Mộ Dung P')
    expect(viaNewPath).toBe(viaOldPath)

    release()
  })

  it('không có vùng chọn nào (bề mặt chưa đăng ký, hoặc caret rỗng) ⇒ CẢ HAI đường đều RỖNG', () => {
    // Không `mountSurface`/`registerSelectionSurface` nào ở ca này — đúng ca "chưa có vùng
    // chọn" của AC (dải mở, ô nguồn rỗng và nhận focus).
    expect(currentSelectionText()).toBe('')
    expect(currentSelectionTextForGlossaryQuickAdd()).toBe('')
  })
})

/**
 * §I/O Matrix ⑮ (cụm D vá, vòng rà Epic 3) — `resolve()` là mã của PANEL (`SourceHanViet.vue`),
 * ngoài tầm kiểm soát của tệp này, và cả hai hàm chạy dưới một lượt `dispatch` từ bàn phím.
 * Đối chứng gỡ-chỗ-nối: gỡ `try/catch` quanh `surface.resolve(selection)` ở CẢ HAI hàm ⇒ hai
 * ca dưới đây ĐỎ (ngoại lệ thoát ra ngoài thay vì bị bọc).
 */
describe('⑮ resolve() ném — ngoại lệ KHÔNG được thoát ra ngoài, trả về rỗng và chẩn đoán', () => {
  it('currentSelectionText — resolve() ném ⇒ trả "", console.error nêu đích danh', () => {
    const { sentence, release } = mountSurface('source')
    selectInside(sentence, 0, 2)

    // `registerSelectionSurface` chỉ nhận resolver LÚC ĐĂNG KÝ — đăng ký lại đúng phần tử để
    // gắn một resolver ném (cùng cơ chế "idempotent" mà doc-comment của hàm đó khai).
    const throwing = registerSelectionSurface(sentence.parentElement as HTMLElement, 'source', () => {
      throw new Error('panel resolver hỏng')
    })
    const errorSpy = vi.spyOn(console, 'error').mockImplementation(() => {})

    expect(currentSelectionText()).toBe('')
    expect(errorSpy).toHaveBeenCalledTimes(1)
    expect(String(errorSpy.mock.calls[0]?.[0])).toContain('resolve()')

    errorSpy.mockRestore()
    throwing()
    release()
  })

  it('currentSelectionTextForGlossaryQuickAdd — resolve() ném ⇒ trả "", console.error nêu đích danh', () => {
    const { sentence, release } = mountSurface('display')
    selectInside(sentence, 0, 2)

    const throwing = registerSelectionSurface(sentence.parentElement as HTMLElement, 'display', () => {
      throw new Error('panel resolver hỏng')
    })
    const errorSpy = vi.spyOn(console, 'error').mockImplementation(() => {})

    expect(currentSelectionTextForGlossaryQuickAdd()).toBe('')
    expect(errorSpy).toHaveBeenCalledTimes(1)
    expect(String(errorSpy.mock.calls[0]?.[0])).toContain('resolve()')

    errorSpy.mockRestore()
    throwing()
    release()
  })
})
