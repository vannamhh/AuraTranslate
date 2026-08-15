/**
 * Thanh trạng thái — **nhóm ④** của Quyết định #6. Story 2.3 · AC7 · AC10 · Task 5.4.
 *
 * Bốn mệnh đề, và mệnh đề thứ nhất là mệnh đề của UX-DR30:
 *   ① câu **KHÔNG** hiện trước lượt flush đầu tiên — *"Đã lưu 0 giây trước"* lúc chưa lưu gì là
 *      câu nói dối tệ nhất mà UX-DR30 tồn tại để chặn;
 *   ② `N` đếm đúng số giây kể từ lượt flush gần nhất, và **không có trần hiển thị**;
 *   ③ không bao giờ hiện một số âm khi đồng hồ hệ thống lùi lại;
 *   ④ `setInterval` **được nhả** khi unmount.
 *
 * ⚠️ `vi.useFakeTimers()` ở đây là **đúng chỗ**, khác `editorFlush.test.ts`: thứ đang kiểm là
 * một component thật sự **đọc đồng hồ** và thật sự dựng một `setInterval`. Luật *"mọi thời điểm
 * đi vào qua tham số"* áp cho **tầng thuần**; một component vỏ ứng dụng không có tham số nào
 * để nhận.
 */
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'
import { mount } from '@vue/test-utils'
import { failNextSave, readFixture, recordSave, resetRecorder } from './support/segmentFixture'

/**
 * Việc chạy **XEN GIỮA** lượt save — cùng khuôn `midFlightHooks` của
 * `editorConfirmSegment.test.ts`, và cùng lý do: lúc `saveSegmentTargets` được gọi thì
 * `flushEditorNow` đã chụp `snapshot` xong và lô **đang bay**. Gõ thêm ở đúng khoảnh khắc đó là
 * cách duy nhất dựng lại ca `'still-dirty'` mà **không** một đồng hồ giả nào cần tới.
 */
const midFlightHooks: (undefined | (() => void))[] = []
let saveIndex = 0

async function recordSaveWithHook(
  chapterId: number,
  edits: readonly { id: number; target_text: string }[],
) {
  midFlightHooks[saveIndex++]?.()
  return recordSave(chapterId, edits)
}

/** Lượt `confirmSegment` luôn thành công ở tệp này — ca `'refused'` có nhà riêng ở `editorConfirmSegment.test.ts`. */
async function confirmOk(segmentId: number) {
  return {
    outcome: { segment_id: segmentId, status: 'confirmed', version_created: true },
    error: null,
  }
}

// ⚠️ `vi.mock` được HOIST lên đầu tệp, và đường dẫn module phân giải tương đối với TỆP NÀY —
// nên nó phải sống ở đây, không trong `support/`. Xem doc-comment của `support/segmentFixture.ts`
// về vì sao phép giả đặt ở **biên IPC** thay vì ở một setter chỉ-test.
vi.mock('../../src/config/segment', async (importOriginal) => {
  const actual = await importOriginal<typeof import('../../src/config/segment')>()
  return {
    ...actual,
    readOpenChapterSegments: readFixture,
    saveSegmentTargets: recordSaveWithHook,
    confirmSegment: confirmOk,
  }
})

/** Nạp lại module registry mỗi ca — state của Panel Editor là module-level singleton. */
async function freshStatusBar() {
  vi.resetModules()
  const state = await import('../../src/panels/editorPanelState')
  const StatusBar = (await import('../../src/StatusBar.vue')).default
  return { state, StatusBar }
}

/**
 * Một lượt flush THẬT qua đúng đường sản phẩm: nạp segment → ghi nhận một lượt sửa → flush.
 * Không setter chỉ-test nào — xem doc-comment của `support/segmentFixture.ts`.
 */
async function flushOnce(state: typeof import('../../src/panels/editorPanelState')): Promise<void> {
  await state.ensureSegmentsLoaded()
  state.noteEditorEdit(11, 'một câu vừa gõ')
  await state.flushEditorNow()
}

beforeEach(() => {
  resetRecorder()
  midFlightHooks.length = 0
  saveIndex = 0
  vi.useFakeTimers()
  vi.setSystemTime(new Date('2026-08-12T12:00:00.000Z'))
})

afterEach(() => {
  vi.useRealTimers()
})

describe('thanh trạng thái — AC7 · AC10 · UX-DR30', () => {
  it('KHÔNG hiện câu nào trước lượt flush đầu tiên', async () => {
    const { StatusBar } = await freshStatusBar()
    const wrapper = mount(StatusBar)

    // Vỏ CÓ mặt — `EXPERIENCE.md:417` tính chiều cao nó vào bố cục cả ba chế độ. Nội dung thì không.
    expect(wrapper.find('.status').exists()).toBe(true)
    expect(wrapper.find('.saved').exists()).toBe(false)
    expect(wrapper.text()).toBe('')

    // Và nó vẫn im sau ba nhịp đồng hồ — không phải chỉ im ở lượt vẽ đầu.
    await vi.advanceTimersByTimeAsync(3000)
    expect(wrapper.find('.saved').exists()).toBe(false)

    wrapper.unmount()
  })

  it('đếm đúng N giây kể từ lượt flush gần nhất, và KHÔNG có trần hiển thị', async () => {
    const { state, StatusBar } = await freshStatusBar()
    const wrapper = mount(StatusBar)

    await flushOnce(state)
    await wrapper.vm.$nextTick()
    expect(wrapper.find('.saved').text()).toBe('Đã lưu 0 giây trước')

    await vi.advanceTimersByTimeAsync(1000)
    expect(wrapper.find('.saved').text()).toBe('Đã lưu 1 giây trước')

    await vi.advanceTimersByTimeAsync(4000)
    expect(wrapper.find('.saved').text()).toBe('Đã lưu 5 giây trước')

    // 🔴 Trần cứng của AD-35 là 5 giây, nên con số này KHÔNG được đứng lại ở đó: nếu một lượt
    // ghi trượt, người dùng phải thấy nó bò lên. Một trần hiển thị ở đây là màn hình nói dối
    // theo hướng an tâm — đúng thứ UX-DR30 cấm.
    await vi.advanceTimersByTimeAsync(25_000)
    expect(wrapper.find('.saved').text()).toBe('Đã lưu 30 giây trước')

    wrapper.unmount()
  })

  it('KHÔNG bao giờ hiện một số âm khi đồng hồ hệ thống lùi lại', async () => {
    // `Date.now()` không đơn điệu tăng: một lượt đồng bộ NTP hay người dùng đổi giờ hệ thống
    // cho ra một hiệu ÂM, và *"Đã lưu −9 giây trước"* là một câu không ai đọc được.
    const { state, StatusBar } = await freshStatusBar()
    const wrapper = mount(StatusBar)

    await flushOnce(state)
    await vi.advanceTimersByTimeAsync(1000)
    expect(wrapper.find('.saved').text()).toBe('Đã lưu 1 giây trước')

    vi.setSystemTime(new Date('2026-08-12T11:59:51.000Z'))
    await vi.advanceTimersByTimeAsync(1000)

    expect(wrapper.find('.saved').text()).toBe('Đã lưu 0 giây trước')

    wrapper.unmount()
  })

  it('nhả `setInterval` khi unmount', async () => {
    const { state, StatusBar } = await freshStatusBar()
    const wrapper = mount(StatusBar)
    await flushOnce(state)
    await vi.advanceTimersByTimeAsync(1000)

    expect(vi.getTimerCount()).toBeGreaterThan(0)
    wrapper.unmount()

    // 🔴 Mệnh đề thật là **KHÔNG còn timer nào sống**, không phải *"`clearInterval` đã được
    // gọi"*: một lượt gọi với handle sai vẫn qua được phép kiểm thứ hai.
    expect(vi.getTimerCount()).toBe(0)
  })
})

/**
 * ═══════════════════════════════════════════════════════════════════════════════
 * Nhóm ⑤ — ĐƯỜNG RA MÀN HÌNH CHO BA `ConfirmResult` KHÔNG ĐI QUA RUST
 * ═══════════════════════════════════════════════════════════════════════════════
 * 🔵 **Thêm 2026-08-15, lượt code review của Story 2.5b.** Vế *"một dòng ở thanh trạng thái"*
 * của **Quyết định #8** *(Ice ký 2026-08-14)* chưa từng được dựng: `'no-caret'` ·
 * `'flush-failed'` · `'still-dirty'` chỉ rơi vào một `console.warn`, nên một cú `⌘Enter` không
 * đổi **một pixel nào**. Bản vá dựng đường đó; nhóm này là lưới duy nhất đứng dưới nó.
 *
 * 🔴 **Vì sao vitest chứ không e2e:** cả năm đường vào/ra đều là **module thuần cộng một
 * component vỏ** — không hình học, không engine. `project-context.md` §Bốn đường nghiệm thu:
 * *"chọn sai đường là dựng nguồn sự thật thứ hai"*. Mệnh đề *"caret có xuất hiện trong ô rỗng
 * không"* mới thuộc WKWebView; mệnh đề *"câu này có hiện lên không"* thì không.
 *
 * ⚠️ **GIỚI HẠN THẬT, ghi ra thay vì để người sau tưởng đã phủ:** nhóm này canh **ô nhớ và
 * người đọc nó**. Nó **không** canh việc `⌘Enter` thật sự tới được `confirmCurrentSegment` —
 * đường đó đi qua `CommandRegistry` và có chủ ở `check-commands.mjs` Kiểm A/B.
 */
describe('⑤ Quyết định #8 vế thanh trạng thái — ba kết quả KHÔNG được im lặng', () => {
  /**
   * 🔴 Ca thường nhất, và ca im lặng nhất trước bản vá: người dùng bấm xác nhận khi con trỏ
   * chưa ở câu nào. Không IPC nào được phát, nên không lỗi nào tồn tại để hiển thị — trước
   * 2026-08-15 màn hình **không đổi gì cả** và người dùng không biết mình vừa làm gì sai.
   *
   * Chạy đỏ-rồi-xanh: bỏ dòng gán `confirmNotice.value` trong `confirmCurrentSegment`, ca ĐỎ.
   */
  it("'no-caret' — bấm xác nhận khi chưa chọn câu nào ⇒ thanh NÓI RA", async () => {
    const { state, StatusBar } = await freshStatusBar()
    const wrapper = mount(StatusBar)
    await state.ensureSegmentsLoaded()

    expect(await state.confirmCurrentSegment()).toBe('no-caret')
    await wrapper.vm.$nextTick()

    expect(wrapper.find('.notice').text()).toBe(
      'Chưa có câu nào đang được chọn — đặt con trỏ vào một câu rồi xác nhận.',
    )
    wrapper.unmount()
  })

  /**
   * 🔴 Ca nguy hiểm nhất trong ba: lượt lưu trượt ⇒ bản dịch **chưa xuống đĩa**. Im ở đây là
   * để người dùng tin rằng câu đã được ký trong khi nó chưa cả được lưu.
   */
  it("'flush-failed' — lượt lưu trượt ⇒ thanh nói rõ bản dịch VẪN CÒN trên màn hình", async () => {
    const { state, StatusBar } = await freshStatusBar()
    const wrapper = mount(StatusBar)
    await state.ensureSegmentsLoaded()
    state.setEditorCaret(11)
    state.noteEditorEdit(11, 'Chữ này sẽ không xuống được đĩa.')
    failNextSave.value = true

    expect(await state.confirmCurrentSegment()).toBe('flush-failed')
    await wrapper.vm.$nextTick()

    expect(wrapper.find('.notice').text()).toBe(
      'Chưa lưu được bản dịch nên chưa xác nhận. Bản dịch vẫn còn trên màn hình.',
    )
    wrapper.unmount()
  })

  /**
   * 🔴 Ca hiếm nhất và khó dựng lại nhất — người dùng gõ nốt một ký tự **trong lúc** cả hai lô
   * flush đang bay. Quyết định #8 chốt *"thử lại đúng một lượt, còn dơ nữa thì TỪ CHỐI"*, và
   * lượt từ chối đó phải nói ra: nếu không, `⌘Enter` trở thành một phím không làm gì.
   */
  it("'still-dirty' — dơ sau HAI lượt flush ⇒ từ chối, và lượt từ chối được NÓI RA", async () => {
    const { state, StatusBar } = await freshStatusBar()
    const wrapper = mount(StatusBar)
    await state.ensureSegmentsLoaded()
    state.setEditorCaret(11)
    state.noteEditorEdit(11, 'Bản đầu.')
    // Gõ thêm trong CẢ HAI lô ⇒ tập chờ vẫn dơ sau lượt thử lại.
    midFlightHooks[0] = () => state.noteEditorEdit(11, 'Bản đầu, và một chữ nữa.')
    midFlightHooks[1] = () => state.noteEditorEdit(11, 'Bản đầu, và hai chữ nữa.')

    expect(await state.confirmCurrentSegment()).toBe('still-dirty')
    await wrapper.vm.$nextTick()

    expect(wrapper.find('.notice').text()).toBe(
      'Bản dịch vừa đổi trong lúc đang lưu — chưa xác nhận. Thử lại.',
    )
    wrapper.unmount()
  })

  /**
   * 🔴 Câu báo **THAY** mốc *"Đã lưu"*, không đứng cạnh nó. Thanh cao 34px chỉ đủ một mệnh đề,
   * và hai câu cùng lúc là hai mệnh đề tranh nhau một chỗ — *"đã lưu 0 giây trước"* đứng cạnh
   * *"chưa lưu được bản dịch"* là màn hình tự mâu thuẫn.
   *
   * Chạy đỏ-rồi-xanh: đổi `v-else-if` của `.saved` thành `v-if`, ca này ĐỎ.
   */
  it('câu báo THAY mốc "Đã lưu", hai câu không bao giờ cùng lúc', async () => {
    const { state, StatusBar } = await freshStatusBar()
    const wrapper = mount(StatusBar)

    // Một lượt flush THẬT trước ⇒ mốc *"Đã lưu"* đang hiện.
    await flushOnce(state)
    await vi.advanceTimersByTimeAsync(1000)
    expect(wrapper.find('.saved').exists()).toBe(true)

    // Rồi một lượt xác nhận hụt.
    expect(await state.confirmCurrentSegment()).toBe('no-caret')
    await wrapper.vm.$nextTick()

    expect(wrapper.find('.notice').exists()).toBe(true)
    expect(wrapper.find('.saved').exists()).toBe(false)
    wrapper.unmount()
  })

  /**
   * 🔴 Câu báo tắt bằng **SỰ KIỆN**, không bằng một hẹn giờ. Người dùng gõ tiếp *là* câu trả
   * lời cho nó; giữ nó lại trong lúc họ đang gõ là để một câu ĐÚNG-LÚC-ĐÓ nói dối ở hiện tại.
   *
   * ⚠️ Ca này cũng là lưới chống một lượt "sửa" tương lai kiểu `setTimeout(clear, 5000)`: một
   * hẹn giờ phải chọn N, và mọi N đều sai với một người đọc chậm hơn.
   */
  it('gõ tiếp ⇒ câu báo TẮT, và mốc "Đã lưu" quay lại chỗ của nó', async () => {
    const { state, StatusBar } = await freshStatusBar()
    const wrapper = mount(StatusBar)
    await flushOnce(state)

    expect(await state.confirmCurrentSegment()).toBe('no-caret')
    await wrapper.vm.$nextTick()
    expect(wrapper.find('.notice').exists()).toBe(true)

    state.noteEditorEdit(11, 'người dùng gõ tiếp')
    await wrapper.vm.$nextTick()

    expect(wrapper.find('.notice').exists()).toBe(false)
    expect(wrapper.find('.saved').exists()).toBe(true)
    wrapper.unmount()
  })

  /**
   * 🔴 Lượt xác nhận **thành công** không để lại câu nào — và `'refused'` cũng vậy, nhưng vì
   * một lý do khác: nó có đường ra RIÊNG và giàu hơn *(một `IpcError` mang `params.segment_id`,
   * nên lỗi dán được lên **đúng hàng** ở `GridPanel.vue`)*. Đẩy nó lên thanh trạng thái nữa là
   * dựng nguồn sự thật thứ hai cho cùng một sự kiện.
   */
  it('lượt xác nhận THÀNH CÔNG không để lại câu báo nào', async () => {
    const { state, StatusBar } = await freshStatusBar()
    const wrapper = mount(StatusBar)
    await state.ensureSegmentsLoaded()
    state.setEditorCaret(11)
    state.noteEditorEdit(11, 'một câu đã dịch xong')

    expect(await state.confirmCurrentSegment()).toBe('confirmed')
    await wrapper.vm.$nextTick()

    expect(wrapper.find('.notice').exists()).toBe(false)
    expect(wrapper.find('.saved').exists()).toBe(true)
    wrapper.unmount()
  })
})
