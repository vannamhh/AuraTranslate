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
import { readFixture, recordSave, resetRecorder } from './support/segmentFixture'

// ⚠️ `vi.mock` được HOIST lên đầu tệp, và đường dẫn module phân giải tương đối với TỆP NÀY —
// nên nó phải sống ở đây, không trong `support/`. Xem doc-comment của `support/segmentFixture.ts`
// về vì sao phép giả đặt ở **biên IPC** thay vì ở một setter chỉ-test.
vi.mock('../../src/config/segment', async (importOriginal) => {
  const actual = await importOriginal<typeof import('../../src/config/segment')>()
  return { ...actual, readOpenChapterSegments: readFixture, saveSegmentTargets: recordSave }
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
