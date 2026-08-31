/**
 * Bàn đo Story 5.13 — marker đọc trong WKWebView thật.
 *
 * Ca này cố ý đi qua bề mặt WKWebView thật để đo Vue/CSS sau `mouseenter` + focus, và để
 * neo `segment.id` thật sự cuộn vào vùng nhìn sau Reading → Workspace → Reading. WebDriver
 * nhúng không dựng `:hover` từ pointer Actions (đo và ghi tại chỗ bên dưới), nên không khai
 * nghiệm thu pseudo-class ấy. Regroup đi qua IPC sản phẩm vì Workspace chưa có bề mặt chọn
 * nhóm ổn định cho bàn đo; nó không giả dữ liệu marker hay sửa DB.
 */

import { realClick } from '../support/pointer.mjs'
import { openWorkspaceWithWork } from '../support/workspace.mjs'

const RUN_TAG = `${Date.now() % 1_000_000}`
const WORK_NAME = `e2e-reading-marks-${RUN_TAG}`
const SOURCE_TEXT = 'Cau mot。Cau hai。Cau ba。Cau bon。Cau nam。'

async function invoke(command, payload = {}) {
  return browser.execute(
    async (name, args) => {
      const internals = window.__TAURI_INTERNALS__
      if (internals === undefined) throw new Error('không có cầu IPC trong webview')
      return internals.invoke(name, args)
    },
    command,
    payload,
  )
}

async function setChapterStatus(chapterId, status) {
  return invoke('set_chapter_status', { chapterId, status })
}

async function waitForReading() {
  await browser.waitUntil(
    async () => browser.execute(() => document.querySelector('[data-reading-segment]') !== null),
    { timeout: 15_000, timeoutMsg: 'Chế độ đọc không dựng segment sau 15 giây' },
  )
}

async function focusReadingSegment(segmentId) {
  const focused = await browser.execute((id) => {
    const node = document.querySelector(`[data-reading-segment="${id}"]`)
    if (!(node instanceof HTMLElement)) return false
    node.focus()
    return document.activeElement === node
  }, segmentId)
  await expect(focused).toBe(true)
}

describe('Story 5.13 — đánh dấu chỗ cần sửa khi đang đọc', () => {
  it('ẩn/hiện affordance, M giữ nhịp, Enter mở exact, marker retired sống qua hai regroup và neo ID cuộn trở lại', async () => {
    await openWorkspaceWithWork(WORK_NAME, SOURCE_TEXT)

    const initial = await invoke('read_open_chapter_segments')
    await expect(initial.segments.length).toBe(5)
    const chapterOneId = initial.chapter_id
    const originalMarkedId = initial.segments[1].id
    const chapterTwoMarkedId = initial.segments[3].id

    // `create_work_from_text` khởi tạo bản dịch rỗng. Một wrapper câu rỗng có bề rộng 0,
    // nên `moveTo()` không thể chứng minh `:hover` dù CSS đúng. Đổ chữ qua ĐÚNG đường flush
    // sản phẩm; lặp đủ dài còn khiến neo cuộn ở cuối ca thật sự nằm ngoài viewport đầu.
    await invoke('save_segment_targets', {
      chapterId: chapterOneId,
      edits: initial.segments.map((segment, index) => ({
        id: segment.id,
        target_text: `Bản dịch bàn đo số ${String(index + 1)} — một câu đủ dài để tạo trang đọc có hình học thật. `.repeat(12),
      })),
    })

    await invoke('split_chapter_at_segment', { segmentId: chapterTwoMarkedId })
    const chapters = await invoke('list_chapters')
    await expect(chapters.length).toBe(2)
    const chapterTwo = chapters.find((chapter) => chapter.chapter_id !== chapterOneId)
    await expect(chapterTwo).not.toBe(undefined)
    await setChapterStatus(chapterOneId, 'done')
    await setChapterStatus(chapterTwo.chapter_id, 'done')

    // Marker ở Chương 2 dựng qua API thật để danh sách có nhiều Chương; marker Chương 1
    // bên dưới vẫn đi qua phím `M`, là bề mặt đang được nghiệm thu.
    await invoke('mark_reading_segment', { segmentId: chapterTwoMarkedId })

    await browser.keys(['Meta', '3'])
    await waitForReading()

    const wrapper = await $(`[data-reading-segment="${originalMarkedId}"]`)
    const affordance = await wrapper.$('.mark-affordance')
    await expect(await affordance.isExisting()).toBe(true)
    await expect(await affordance.getCSSProperty('display')).toHaveProperty('value', 'none')

    // Embedded WebDriver của WKWebView không sinh `:hover` từ Actions `move`: bàn đo
    // 2026-08-31 đã nhắm đúng `ClientRect` (elementFromPoint = `.segment-text`) nhưng
    // `document.querySelectorAll(':hover')` vẫn rỗng, cùng lớp giới hạn đã đo cho text
    // selection ở `glossary-quick-add.e2e.mjs`. Phát `mouseenter` trên engine thật để
    // nghiệm thu đường SẢN PHẨM (`@mouseenter` → aim → Vue render); CSS `:hover` vẫn là
    // đường bổ sung cho con trỏ người dùng thật.
    const entered = await browser.execute((id) => {
      const node = document.querySelector(`[data-reading-segment="${id}"]`)
      if (!(node instanceof HTMLElement)) return false
      node.dispatchEvent(new MouseEvent('mouseenter', { bubbles: true, composed: true }))
      return true
    }, originalMarkedId)
    await expect(entered).toBe(true)
    await browser.waitUntil(
      async () => browser.execute((id) => {
        const button = document.querySelector(`[data-reading-segment="${id}"] .mark-affordance`)
        return button instanceof HTMLElement && getComputedStyle(button).display !== 'none'
      }, originalMarkedId),
      { timeout: 5_000, timeoutMsg: 'affordance marker không hiện khi hover' },
    )
    await browser.execute((id) => {
      const node = document.querySelector(`[data-reading-segment="${id}"]`)
      if (node instanceof HTMLElement) node.dispatchEvent(new MouseEvent('mouseleave'))
    }, originalMarkedId)
    await browser.waitUntil(
      async () => browser.execute((id) => {
        const button = document.querySelector(`[data-reading-segment="${id}"] .mark-affordance`)
        return button instanceof HTMLElement && getComputedStyle(button).display === 'none'
      }, originalMarkedId),
      { timeout: 5_000, timeoutMsg: 'affordance marker không ẩn sau khi chuột rời câu' },
    )
    await focusReadingSegment(originalMarkedId)
    await browser.waitUntil(
      async () => browser.execute((id) => {
        const button = document.querySelector(`[data-reading-segment="${id}"] .mark-affordance`)
        return button instanceof HTMLElement && getComputedStyle(button).display !== 'none'
      }, originalMarkedId),
      { timeout: 5_000, timeoutMsg: 'affordance marker không hiện khi wrapper nhận focus' },
    )
    await expect(await affordance.getAttribute('tabindex')).toBe('-1')

    const beforeMark = await browser.execute((id) => {
      const node = document.querySelector(`[data-reading-segment="${id}"]`)
      return {
        top: node?.getBoundingClientRect().top ?? null,
        focused: document.activeElement?.getAttribute('data-reading-segment') ?? null,
      }
    }, originalMarkedId)
    await browser.keys(['m'])
    await browser.waitUntil(
      async () => browser.execute((id) => document.querySelector(`[data-reading-segment="${id}"].marked`) !== null, originalMarkedId),
      { timeout: 5_000, timeoutMsg: 'M không vẽ trạng thái marked sau khi Rust trả lời' },
    )
    const afterMark = await browser.execute((id) => {
      const node = document.querySelector(`[data-reading-segment="${id}"]`)
      return {
        top: node?.getBoundingClientRect().top ?? null,
        focused: document.activeElement?.getAttribute('data-reading-segment') ?? null,
        stillReading: document.querySelector('.column') !== null,
      }
    }, originalMarkedId)
    await expect(afterMark.stillReading).toBe(true)
    await expect(afterMark.focused).toBe(String(originalMarkedId))
    // WebKit 605 điều chỉnh scroll anchoring đúng 2 px khi nhãn marker nhập line box.
    // Neo sản phẩm là ID, không pixel; ngưỡng 3 px vẫn bắt một cú nhảy dòng/viewport.
    await expect(Math.abs(afterMark.top - beforeMark.top)).toBeLessThan(3)

    // `M` lặp là INSERT idempotent, không toggle. Hai marker tổng cộng: một ở mỗi Chương.
    await browser.keys(['m'])
    const twoMarks = await invoke('list_reading_marks')
    await expect(twoMarks.length).toBe(2)

    // Enter chỉ sống trên wrapper đang focus và mở đúng segment, không một command global.
    await browser.keys(['Enter'])
    await browser.waitUntil(
      async () => browser.execute(() => document.querySelector('[data-col="tgt"]') !== null),
      { timeout: 15_000, timeoutMsg: 'Enter trên câu đọc không mở Workspace' },
    )
    const exactAfterEnter = await browser.execute(() => ({
      id: document.activeElement?.getAttribute('data-segment-id') ?? null,
      col: document.activeElement?.getAttribute('data-col') ?? null,
    }))
    await expect(exactAfterEnter).toEqual({ id: String(originalMarkedId), col: 'tgt' })

    // Lượt một retire segment gốc bằng merge; lượt hai retire chính neo mới bằng split.
    const merged = await invoke('merge_segments', { segmentId: originalMarkedId })
    const firstAnchor = merged.new_segments[0]
    const cut = Math.max(1, Math.floor([...firstAnchor.source_text].length / 2))
    const split = await invoke('split_segment', { segmentId: firstAnchor.id, cuts: [cut] })
    const liveAnchorId = split.new_segments[0].id

    const rebased = await invoke('list_reading_marks')
    await expect(rebased.length).toBe(2)
    await expect(rebased[0].segment_id).toBe(originalMarkedId)
    await expect(rebased[0].navigation_segment_id).toBe(liveAnchorId)
    await expect(rebased[0].is_retired).toBe(true)
    await expect(rebased[0].chapter_ord).toBeLessThan(rebased[1].chapter_ord)

    await browser.keys(['Meta', '3'])
    await waitForReading()
    const tunerButton = await $('[data-reading-toggle-tuner]')
    await realClick(tunerButton)
    await browser.waitUntil(
      async () => browser.execute(() => document.querySelector('[data-reading-tuner-font-size]') !== null),
      { timeout: 5_000, timeoutMsg: 'thanh tinh chỉnh không dựng sau khi bấm nút' },
    )
    const changedTypography = await browser.execute(() => {
      const input = document.querySelector('[data-reading-tuner-font-size]')
      if (!(input instanceof HTMLInputElement)) return false
      input.value = '28'
      input.dispatchEvent(new Event('input', { bubbles: true }))
      return true
    })
    await expect(changedTypography).toBe(true)
    const listButton = await $('[data-reading-marks]')
    await realClick(listButton)
    await browser.waitUntil(
      async () => browser.execute(() => document.querySelectorAll('.toc-overlay .toc-list li').length === 2),
      { timeout: 5_000, timeoutMsg: 'danh sách marker không hiện đủ hai Chương' },
    )
    const listRows = await browser.execute(() =>
      [...document.querySelectorAll('.toc-overlay .toc-list li')].map((node) => node.textContent ?? ''),
    )
    await expect(listRows[0]).toContain('Câu này đã đổi')
    await expect(listRows[0]).toContain('Chương 1')
    await expect(listRows[1]).toContain('Chương 2')

    // Hai nút điều hướng phải đổi đúng `aria-current` và dừng ở biên; sau đó quay về mục
    // đầu để ca mở marker retired ngay dưới vẫn nghiệm thu neo của Chương 1.
    const nextMarked = await $('.toc-overlay .toc-actions .btn:nth-child(2)')
    const prevMarked = await $('.toc-overlay .toc-actions .btn:nth-child(1)')
    await realClick(nextMarked)
    await realClick(nextMarked)
    await expect(await browser.execute(() =>
      document.querySelector('.toc-overlay .toc-list li[aria-current="true"]')?.textContent ?? '',
    )).toContain('Chương 2')
    await realClick(prevMarked)
    await expect(await browser.execute(() =>
      document.querySelector('.toc-overlay .toc-list li[aria-current="true"]')?.textContent ?? '',
    )).toContain('Chương 1')

    // Mục đầu là marker retired của Chương 1. Mở nó phải dùng neo SỐNG sau lượt split.
    const openMarked = await $('.toc-overlay .toc-actions .btn:nth-child(3)')
    await realClick(openMarked)
    await browser.waitUntil(
      async () => browser.execute(() => document.querySelector('[data-col="tgt"]') !== null),
      { timeout: 15_000, timeoutMsg: 'không mở được marker retired trong Workspace' },
    )
    const exactRetiredAnchor = await browser.execute(() =>
      document.activeElement?.getAttribute('data-segment-id') ?? null,
    )
    await expect(exactRetiredAnchor).toBe(String(liveAnchorId))

    // Neo là ID, không pixel: đổi cỡ chữ trước khi trở lại rồi chỉ đòi đúng node sống nằm
    // trong viewport. Một scrollTop cũ sẽ không đủ bảo đảm mệnh đề này khi hình học đổi.
    await browser.keys(['Meta', '3'])
    await waitForReading()
    await browser.waitUntil(
      async () => browser.execute((id) => {
        const node = document.querySelector(`[data-reading-segment="${id}"]`)
        if (!(node instanceof HTMLElement)) return false
        const rect = node.getBoundingClientRect()
        return rect.bottom > 0 && rect.top < window.innerHeight
      }, liveAnchorId),
      { timeout: 5_000, timeoutMsg: 'neo segment ID không được cuộn trở lại vùng nhìn' },
    )
    const anchorViewport = await browser.execute((id) => {
      const node = document.querySelector(`[data-reading-segment="${id}"]`)
      if (!(node instanceof HTMLElement)) return { found: false, visible: false }
      const rect = node.getBoundingClientRect()
      return { found: true, visible: rect.bottom > 0 && rect.top < window.innerHeight }
    }, liveAnchorId)
    await expect(anchorViewport).toEqual({ found: true, visible: true })

    // Đường thường không chọn marker: cuộn tới một câu khác, rời Reading, rồi quay lại.
    // `onDeactivated` phải chụp ID gần mép viewport; ca trước chỉ nghiệm thu neo được đặt
    // tường minh bởi thao tác mở marker nên không bắt được nếu phép chụp viewport bị gỡ.
    const scrolled = await browser.execute((id) => {
      const node = document.querySelector(`[data-reading-segment="${id}"]`)
      if (!(node instanceof HTMLElement)) return false
      node.scrollIntoView({ block: 'center' })
      return true
    }, chapterTwoMarkedId)
    await expect(scrolled).toBe(true)
    await browser.keys(['Meta', '2'])
    await browser.waitUntil(
      async () => browser.execute(() => document.querySelector('[data-col="tgt"]') !== null),
      { timeout: 15_000, timeoutMsg: 'Workspace không hiện sau khi rời Reading' },
    )
    await browser.keys(['Meta', '3'])
    await waitForReading()
    await browser.waitUntil(
      async () => browser.execute((id) => {
        const node = document.querySelector(`[data-reading-segment="${id}"]`)
        if (!(node instanceof HTMLElement)) return false
        const rect = node.getBoundingClientRect()
        return rect.bottom > 0 && rect.top < window.innerHeight
      }, chapterTwoMarkedId),
      { timeout: 5_000, timeoutMsg: 'neo chụp từ viewport không được khôi phục' },
    )
  })
})
