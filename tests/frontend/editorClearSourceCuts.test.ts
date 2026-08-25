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
  // 🔵 THÊM 2026-08-25 — hai dải phải nạp trong CÙNG lượt `resetModules()` với `commands`,
  // nếu không cổng dưới đây đọc một thể hiện module khác thể hiện mà ca test bật lên.
  const quickAdd = await import('../../src/glossaryQuickAddState')
  const confirmStrip = await import('../../src/glossaryConfirmStripState')
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
    // 🔵 **CẬP NHẬT 2026-08-25 — bản sao này ĐÃ HẾT KHỚP `main.ts` và phải theo kịp.**
    // Vòng rà Epic 3 thêm một vệ vào chính cổng này (`main.ts::clearSourceCuts`): `Escape`
    // trần thuộc về DẢI đang mở, không thuộc về tập điểm cắt. Chép nguyên vệ đó xuống đây —
    // một bản sao lệch là một bàn test đo một sản phẩm không tồn tại.
    clearSourceCuts: () => {
      if (quickAdd.quickAddIsOpen.value || confirmStrip.confirmStripIsOpen.value) return
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
  return { state, commands, quickAdd, confirmStrip, wrapper }
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

/**
 * 🔵 **THÊM 2026-08-25 — vòng rà Epic 3, và một vệ chỉ tồn tại ở `main.ts`.**
 *
 * `editor.clear_source_cuts` là command DUY NHẤT của kho mang `Escape` **trần** — ngoại lệ
 * có đo của Story 2.9 AC8, ghi ở `commands/index.ts`. Từ Story 3.3/3.6, hai **dải nội tuyến**
 * cũng dùng `Escape` làm cử chỉ lui của riêng chúng (`@keydown.esc.prevent` trên gốc dải), và
 * hai lời tuyên bố ấy va nhau:
 *
 * - `keys.ts::isTypingZone` nuốt hợp âm không-`Mod` khi tiêu điểm ở
 *   `INPUT`/`TEXTAREA`/`SELECT`/`contenteditable` ⇒ ô nhập và cả radio phân loại của dải đều
 *   đã an toàn.
 * - Một `<button>` thì **KHÔNG** thuộc `isTypingZone`. Tab tới nút Lưu/Đóng/Hoãn của một dải
 *   đang mở rồi bấm `Escape` ⇒ dải đóng **và** tập điểm cắt của segment đang mở bị xoá, im
 *   lặng, không ai yêu cầu.
 *
 * 🔴 **Bản vá KHÔNG được là `isBlocked`** — nó nuốt mọi hợp âm suốt thời gian dải mở, tức lật
 * quyết định Ice ký 2026-08-20 (*"dải không nuốt bàn phím, nó không phải một `KeymapGate`"*).
 * Và KHÔNG được là `.stop` trên dải — `attachKeymap` gắn ở pha `capture` trên `window` nên nó
 * tới quá muộn. Vệ đứng ở cổng `clearSourceCuts` của `main.ts`.
 *
 * ⚠️ **GIỚI HẠN THẬT của nhóm này, ghi ra thay vì làm tròn lên:** `main.ts` không nạp được
 * trong vitest, nên `mountEditor()` **chép** cổng đó xuống bàn test. Nhóm ca dưới đây chứng
 * minh HÌNH DẠNG vệ đúng trên state THẬT và đường command THẬT; nó **không** chứng minh
 * `main.ts` mang đúng hình dạng đó. Bản sao ấy không cổng nào canh — món nợ có chủ ở
 * `deferred-work.md`, cùng lớp với ghi chú *"`installCommands(deps)` ở `main.ts` được giữ bởi
 * `check:commands` cộng e2e"* ở `editorNavNotice.test.ts`.
 */
describe('🔵 2026-08-25 — `Esc` thuộc về DẢI đang mở, không thuộc tập điểm cắt', () => {
  /** Một segment đủ để dải chốt mọc, đúng hình dạng `GlossarySegmentSource`. */
  const SEG_CHO_CHOT = [{ id: 11, source_text: '萧炎登场' }]

  /** Một mark CHƯA chốt — điều kiện để `syncGlossaryConfirmStripTarget` mở dải. */
  const MARK_CHO_CHOT = {
    start: 0,
    end: 2,
    tier: 'global' as const,
    is_confirmed: false,
    translation: null,
    id: 7,
    source_term: '萧炎',
    han_viet_suggestion: null,
    han_viet_status: 'not_requested' as const,
  }

  it('🔴 ⑥ dải "Thêm nhanh" ĐANG MỞ ⇒ `Esc` KHÔNG xoá tập điểm cắt', async () => {
    const { state, commands, quickAdd } = await mountEditor()
    state.setEditorSourceCut(11, 2)
    state.setEditorSourceCut(11, 4)

    quickAdd.openGlossaryQuickAdd('萧炎')
    expect(quickAdd.quickAddIsOpen.value).toBe(true)

    // Đúng đường sản phẩm: hợp âm → command → cổng. Không gọi thẳng `clearEditorSourceCut()`,
    // vì thứ đang được kiểm LÀ cổng nằm giữa hai đầu đó.
    commands.dispatch('editor.clear_source_cuts')

    expect(state.editorSourceCut.value?.offsets).toEqual([2, 4])
  })

  it('🔴 ⑦ dải "Chờ chốt" ĐANG MỞ ⇒ `Esc` KHÔNG xoá tập điểm cắt', async () => {
    // Vế thứ hai của biểu thức `||`. Không có ca này thì gỡ hẳn `confirmStripIsOpen` khỏi vệ
    // vẫn xanh trọn — đúng khuôn "gỡ chỗ nối rồi chạy bộ test cũ" mà `AGENTS.md` đòi.
    const { state, commands, confirmStrip } = await mountEditor()
    state.setEditorSourceCut(11, 2)

    confirmStrip.syncGlossaryConfirmStripTarget(11, SEG_CHO_CHOT, true, [MARK_CHO_CHOT])
    expect(confirmStrip.confirmStripIsOpen.value).toBe(true)

    commands.dispatch('editor.clear_source_cuts')

    expect(state.editorSourceCut.value?.offsets).toEqual([2])
  })

  it('⑧ KHÔNG dải nào mở ⇒ `Esc` vẫn xoá như Story 2.9 AC8 — vệ không được lấy mất tính năng', async () => {
    const { state, commands, quickAdd, confirmStrip } = await mountEditor()
    state.setEditorSourceCut(11, 2)
    state.setEditorSourceCut(11, 4)

    expect(quickAdd.quickAddIsOpen.value).toBe(false)
    expect(confirmStrip.confirmStripIsOpen.value).toBe(false)

    commands.dispatch('editor.clear_source_cuts')

    expect(state.editorSourceCut.value).toBe(null)
  })

  it('⑨ dải đóng lại ⇒ `Esc` xoá được trở lại — vệ đọc TRẠNG THÁI, không phải một cờ dính', async () => {
    const { state, commands, quickAdd } = await mountEditor()
    state.setEditorSourceCut(11, 2)

    quickAdd.openGlossaryQuickAdd('萧炎')
    commands.dispatch('editor.clear_source_cuts')
    expect(state.editorSourceCut.value?.offsets).toEqual([2])

    quickAdd.closeGlossaryQuickAdd()
    expect(quickAdd.quickAddIsOpen.value).toBe(false)
    commands.dispatch('editor.clear_source_cuts')

    expect(state.editorSourceCut.value).toBe(null)
  })
})

/**
 * 🔵 **THÊM 2026-08-25 — đóng nốt hàng §I/O Matrix bằng ĐÚNG đường nó khai.**
 *
 * Nhóm ⑥–⑨ ở trên `dispatch` thẳng id command, tức nó canh **cổng** nhưng bỏ qua quãng đường
 * dẫn tới cổng. Hàng bảng I/O thì nói về một cử chỉ THẬT: *"`Escape` khi tiêu điểm ở một
 * `<button>` trong dải đang mở"*. Cả lập luận thiết kế của bản vá dựa trên một mệnh đề của
 * quãng đường ấy — `keys.ts::isTypingZone` trả `false` cho `BUTTON` nhưng `true` cho `INPUT` —
 * nên mệnh đề đó phải được ĐO, không được suy.
 *
 * ⇒ Hai ca dưới gắn keymap THẬT (`attachKeyboard`) rồi bắn một `KeyboardEvent` `Escape` từ một
 * `<button>` thật: bàn phím → `keys.ts` → registry → cổng `clearSourceCuts`.
 *
 * ⚠️ `attachKeymap` **ném ở lần gắn thứ hai** vào cùng target (canh gác gọi lại, `keys.ts`),
 * nên mỗi ca gắn vào một `<div>` riêng của chính nó và gỡ ngay sau khi đo.
 */
describe('🔵 2026-08-25 — cử chỉ `Escape` THẬT đi qua keymap, không chỉ một lượt `dispatch`', () => {
  /** Một `<button>` rời, gắn vào `body` — `BUTTON` KHÔNG thuộc `isTypingZone`. */
  function banPhimTrenNut(target: HTMLElement): void {
    const nut = document.createElement('button')
    target.appendChild(nut)
    nut.dispatchEvent(
      new KeyboardEvent('keydown', { key: 'Escape', code: 'Escape', bubbles: true, cancelable: true }),
    )
  }

  it('🔴 ⑩ dải ĐANG MỞ + `Escape` từ một `<button>` ⇒ tập điểm cắt CÒN NGUYÊN', async () => {
    const { state, commands, quickAdd } = await mountEditor()
    state.setEditorSourceCut(11, 2)
    quickAdd.openGlossaryQuickAdd('萧炎')

    const host = document.createElement('div')
    document.body.appendChild(host)
    const go = commands.attachKeyboard(host)
    try {
      banPhimTrenNut(host)
    } finally {
      go()
      host.remove()
    }

    expect(state.editorSourceCut.value?.offsets).toEqual([2])
  })

  it('⑪ KHÔNG dải nào mở + `Escape` từ một `<button>` ⇒ XOÁ — và đó là mệnh đề `isTypingZone` mà bản vá dựa vào', async () => {
    // Ca này là đối chứng DƯƠNG của quãng đường: nếu `BUTTON` hoá ra bị `isTypingZone` nuốt
    // như `INPUT`, thì lỗ mà bản vá bịt không tồn tại và ca này sẽ ĐỎ — tức chính lập luận
    // thiết kế bị bác, chứ không phải một ca cần nới.
    const { state, commands, quickAdd, confirmStrip } = await mountEditor()
    state.setEditorSourceCut(11, 2)
    expect(quickAdd.quickAddIsOpen.value).toBe(false)
    expect(confirmStrip.confirmStripIsOpen.value).toBe(false)

    const host = document.createElement('div')
    document.body.appendChild(host)
    const go = commands.attachKeyboard(host)
    try {
      banPhimTrenNut(host)
    } finally {
      go()
      host.remove()
    }

    expect(state.editorSourceCut.value).toBe(null)
  })

  it('⑫ `Escape` từ một `<input>` ⇒ keymap NUỐT (không dải nào mở) — vế kia của `isTypingZone`', async () => {
    // Vế đối xứng, và nó giải thích vì sao radio phân loại của dải đã an toàn TỪ TRƯỚC bản vá:
    // `isTypingZone` nhận `INPUT`, nên hợp âm không-`Mod` không bao giờ tới registry từ đó.
    const { state, commands } = await mountEditor()
    state.setEditorSourceCut(11, 2)

    const host = document.createElement('div')
    document.body.appendChild(host)
    const o = document.createElement('input')
    o.type = 'radio'
    host.appendChild(o)
    const go = commands.attachKeyboard(host)
    try {
      o.dispatchEvent(
        new KeyboardEvent('keydown', { key: 'Escape', code: 'Escape', bubbles: true, cancelable: true }),
      )
    } finally {
      go()
      host.remove()
    }

    expect(state.editorSourceCut.value?.offsets).toEqual([2])
  })
})
