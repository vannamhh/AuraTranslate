/**
 * Story 2.9 · AC8 — **`Esc` xoá mọi điểm cắt đang chờ** *(Ice yêu cầu 2026-08-17)*.
 *
 * ═════════════════════════════════════════════════════════════════════════════════
 * 🔴 VÌ SAO PHẢI CÓ ĐƯỜNG LUI NÀY, VÀ VÌ SAO NÓ CẦN **HAI** CỬA VÀO
 * ═════════════════════════════════════════════════════════════════════════════════
 * Đường lui duy nhất hôm nay là *"bấm trùng đúng điểm đã đặt để gỡ nó"* — doc-comment của
 * `sourceCut` đã ghi thẳng rằng nó phải có **vì `⌘Z` chưa được đặc tả**. Với một tập nhiều
 * điểm, bấm trùng **từng cái** là một đường lui trên giấy: người dùng phải nhớ mình đã bấm ở
 * đâu, và bấm trượt thì **thêm** một điểm nữa.
 *
 * ⚠️ **Và `Esc` một mình KHÔNG tới nơi** — đây là cạm bẫy ② của cử chỉ `Backspace`, lặp lại ở
 * một phím khác. `keys.ts:510`:
 *
 * ```ts
 * if (lacksPrimaryMod(entry.mods) && isTypingZone(event.target)) return false
 * ```
 *
 * `Escape` **không có** phím bổ trợ chính, và tiêu điểm gần như luôn nằm trong một ô bản dịch
 * *(`contenteditable`, tức `isTypingZone`)* — người dùng vừa gõ ở đó xong. ⇒ Một command
 * `Escape` đăng ký **đúng luật AD-34** vẫn **không bao giờ chạy ở chỗ nó có nghĩa nhất**.
 *
 * ⇒ **Hai cửa, MỘT command** *(không hai đường xoá)*: ① registry cho ca tiêu điểm ngoài vùng
 * gõ, và nó là thứ làm phím này **gán lại được** (FR22) và **hiện trong bảng phím**;
 * ② `onEditKeydown` bắt trực tiếp rồi `dispatch` **chính** id đó. Cùng khuôn `Backspace` của
 * AC1, và cùng lý do: một cử chỉ trong vùng gõ **là** hệ quả của việc gõ.
 */
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'
import { mount } from '@vue/test-utils'
import { readFixture, recordSave, resetRecorder } from './support/segmentFixture'

vi.mock('../../src/config/segment', async (importOriginal) => {
  const actual = await importOriginal<typeof import('../../src/config/segment')>()
  return { ...actual, readOpenChapterSegments: readFixture, saveSegmentTargets: recordSave }
})

const STUBS = { PanelFrame: { template: '<div class="panel-frame"><slot /></div>' } }

/**
 * 🔴 **Mọi wrapper phải được GỠ giữa hai ca — một khuyết tật của THƯỚC đã trả giá để biết.**
 *
 * `attachTo: document.body` cộng `vi.resetModules()` ở mỗi ca để lại **nhiều** `GridPanel`
 * chồng nhau trong `body`, và `document.querySelector(...)` bắt được ô của lần mount **CŨ
 * NHẤT**. Handler của ô đó trỏ vào một *module instance* khác, nên cú `Escape` đi tới
 * `editorPanelState` của lần trước, còn phép khẳng định thì đọc lần này.
 *
 * ⚠️ Đo được, không suy: ở ca thứ ba, `document.querySelectorAll('.col-tgt').length` = **3**
 * và `wrapper.element.contains(o)` = **false**. Ca đó **xanh khi chạy riêng** — đúng khuôn
 * *"xanh riêng, đỏ trong bộ"* mà kho này đã gặp ở bộ e2e, lần này ở vitest.
 */
const daMount: { unmount: () => void }[] = []
afterEach(() => {
  for (const w of daMount.splice(0)) w.unmount()
  document.body.innerHTML = ''
})

async function mountEditor() {
  vi.resetModules()
  const state = await import('../../src/panels/editorPanelState')
  const commands = await import('../../src/commands')
  // 🔴 **PHẢI gọi `installCommands` — `dispatch` NÉM với một id chưa đăng ký.** Ca ③ đi qua
  // `onEditKeydown` → `dispatch('editor.clear_source_cuts')`, tức **đúng đường sản phẩm**;
  // một bàn test không cài command sẽ đo một ứng dụng chưa khởi động xong.
  // ⚠️ Nối `clearSourceCuts` vào **chính** hàm của `editorPanelState`, y như `main.ts` — không
  // một hàm giả. Chuỗi này là thứ đang được kiểm: phím → command → state.
  commands.installCommands({
    isMac: true,
    // ⚠️ `setMode` là dep **bắt buộc** duy nhất của `CommandDeps` — mọi dep khác tuỳ chọn và
    // `portMissing` lo ca vắng mặt lúc chạy. Một hàm rỗng ở đây là đúng: ca này không đo
    // chuyển chế độ, và bỏ nó đi thì `vue-tsc` đỏ *(cây test CÓ được kiểm kiểu — một cây test
    // không kiểm kiểu là một cây test sẽ mục)*.
    setMode: () => {},
    clearSourceCuts: () => {
      state.clearEditorSourceCut()
    },
  })
  const GridPanel = (await import('../../src/panels/GridPanel.vue')).default
  const wrapper = mount(GridPanel, {
    props: { params: {} } as never,
    global: { stubs: STUBS },
    attachTo: document.body,
  })
  daMount.push(wrapper)
  await state.ensureSegmentsLoaded()
  await wrapper.vm.$nextTick()
  return { state, commands, wrapper }
}

beforeEach(() => {
  resetRecorder()
})

describe('Story 2.9 · AC8 — `Esc` xoá tập điểm cắt', () => {
  it('① `clearEditorSourceCut()` xoá cả tập, không chỉ điểm cuối', async () => {
    const { state } = await mountEditor()
    state.setEditorSourceCut(11, 2)
    state.setEditorSourceCut(11, 5)
    expect(state.editorSourceCut.value?.offsets).toEqual([2, 5])

    state.clearEditorSourceCut()

    // 🔴 Về `null`, **không** một tập RỖNG — hai giá trị cho cùng một trạng thái là chỗ một
    // phép kiểm `!== null` sẽ nói dối. `setEditorSourceCut` đã giữ đúng bất biến đó khi gỡ
    // điểm cuối cùng; đường này phải giữ y hệt.
    expect(state.editorSourceCut.value).toBe(null)
  })

  it('② gọi khi KHÔNG có điểm nào ⇒ vô hại, vẫn `null`', async () => {
    const { state } = await mountEditor()
    expect(state.editorSourceCut.value).toBe(null)
    state.clearEditorSourceCut()
    expect(state.editorSourceCut.value).toBe(null)
  })

  it('🔴 ③ `Esc` trong Ô BẢN DỊCH xoá tập — cửa vào mà registry KHÔNG với tới', async () => {
    const { state, wrapper } = await mountEditor()
    state.setEditorSourceCut(11, 2)
    state.setEditorSourceCut(11, 4)
    expect(state.editorSourceCut.value?.offsets.length).toBe(2)

    const o = document.querySelector<HTMLElement>('[data-col="tgt"][data-segment-id="11"]')
    expect(o).not.toBeNull()
    // 🔴 Chốt của THƯỚC, không của sản phẩm: ô phải thuộc **wrapper của chính ca này**. Nếu
    // `afterEach` thôi dọn, phép khẳng định dưới sẽ đo một module instance khác — và ca đỏ sẽ
    // trỏ vào sản phẩm thay vì vào bàn test. Xem khối 🔴 ở `daMount`.
    expect(wrapper.element.contains(o)).toBe(true)
    o?.dispatchEvent(
      new KeyboardEvent('keydown', { key: 'Escape', code: 'Escape', bubbles: true, cancelable: true }),
    )
    await wrapper.vm.$nextTick()

    expect(state.editorSourceCut.value).toBe(null)
  })

  it('🔴 ④ `Esc` giữa một lượt chốt của BỘ GÕ ⇒ KHÔNG xoá gì', async () => {
    // Cùng dòng và cùng lý do `keys.ts:506`: một lượt commit composition phát `keydown` mang
    // `code` vật lý. Chốt `isComposing` đứng TRƯỚC mọi nhánh của `onEditKeydown`, và ca này
    // là lưới duy nhất chứng minh nhánh mới đứng SAU nó.
    const { state, wrapper } = await mountEditor()
    state.setEditorSourceCut(11, 2)

    const o = document.querySelector<HTMLElement>('[data-col="tgt"][data-segment-id="11"]')
    o?.dispatchEvent(
      new KeyboardEvent('keydown', {
        key: 'Escape',
        code: 'Escape',
        isComposing: true,
        bubbles: true,
        cancelable: true,
      }),
    )
    await wrapper.vm.$nextTick()

    expect(state.editorSourceCut.value?.offsets).toEqual([2])
  })

  it('⑤ command `editor.clear_source_cuts` có mặt trong registry với hợp âm `Escape`', async () => {
    // Vế này là thứ làm phím gán lại được (FR22) và hiện trong bảng phím của Story 1.21 —
    // nó KHÔNG thừa chỉ vì `onEditKeydown` đã bắt trực tiếp.
    const { commands } = await mountEditor()
    const chords = commands.defaultChordsFor('editor.clear_source_cuts')
    expect(chords).toContain('Escape')
  })
})
