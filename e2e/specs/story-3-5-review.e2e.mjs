/**
 * Story 3.5 review — đường IPC/event và tính độc quyền của modal trên WKWebView thật.
 *
 * Hai lớp test rẻ hơn đã canh thuật toán thuần và state Vue. Tệp này chỉ giữ những vế
 * chúng không thể chứng minh: command import trả trước event nền, config ĐÃ GHI thật sự
 * đổi kết quả của hai Work mới, và keymap gắn ở `window` không chạy xuyên qua modal.
 */
import { realClick } from '../support/pointer.mjs'

const EVENT = 'aura://glossary-import-scan-completed'
const TERM = 'AuraReview FireDragon'
const MODAL = '.gs-panel'

async function waitForIpcBridge() {
  await browser.waitUntil(
    async () => browser.execute(() => window.__TAURI_INTERNALS__ !== undefined),
    {
      timeout: 30_000,
      interval: 250,
      timeoutMsg: 'không thấy cầu IPC sau 30 giây — bàn đo Story 3.5 chưa dựng được tiền đề',
    },
  )
}

/** Đăng ký TRƯỚC import bằng chính protocol mà `@tauri-apps/api/event::listen` dùng. */
async function registerScanListener() {
  return browser.execute(async (eventName) => {
    const internals = window.__TAURI_INTERNALS__
    window.__story35ScanEvents = []
    const handler = internals.transformCallback((event) => {
      window.__story35ScanEvents.push(event.payload)
    })
    const eventId = await internals.invoke('plugin:event|listen', {
      event: eventName,
      target: { kind: 'Any' },
      handler,
    })
    window.__story35ScanListener = { eventId, handler }
    return eventId
  }, EVENT)
}

async function unregisterScanListener() {
  await browser.execute(async (eventName) => {
    const listener = window.__story35ScanListener
    if (listener === undefined) return
    window.__TAURI_EVENT_PLUGIN_INTERNALS__.unregisterListener(eventName, listener.eventId)
    await window.__TAURI_INTERNALS__.invoke('plugin:event|unlisten', {
      event: eventName,
      eventId: listener.eventId,
    })
    delete window.__story35ScanListener
  }, EVENT)
}

async function persistThreshold(value) {
  await browser.execute(async (threshold) => {
    await window.__TAURI_INTERNALS__.invoke('put_config', {
      kind: 'app_config',
      key: 'glossary_scan_threshold',
      value: String(threshold),
    })
  }, value)
}

async function importEnglishWork(name) {
  const text = Array.from(
    { length: 5 },
    (_, i) => `a beast called ${TERM} appeared at hour ${i}.`,
  ).join(' ')
  return browser.execute(async (workName, sourceText) => {
    await window.__TAURI_INTERNALS__.invoke('create_work_from_text', {
      name: workName,
      sourceLang: 'en',
      genre: 'general',
      text: sourceText,
    })
    return window.__story35ScanEvents.length
  }, name, text)
}

async function waitForOneScanEvent() {
  await browser.waitUntil(
    async () => browser.execute(() => window.__story35ScanEvents.length >= 1),
    { timeout: 30_000, interval: 50, timeoutMsg: 'worker quét không phát event sau 30 giây' },
  )
  const events = await browser.execute(() => [...window.__story35ScanEvents])
  expect(events).toHaveLength(1)
  return events[0]
}

async function pendingCandidates() {
  return browser.execute(() => window.__TAURI_INTERNALS__.invoke('glossary_pending_candidates'))
}

async function activeModeLabel() {
  return browser.execute(() => document.querySelector('.mode-tab.on')?.textContent?.trim() ?? null)
}

/**
 * Fallback chỉ dùng khi driver pointer đã chạy nhưng cửa sổ macOS không nhận click vì
 * tauri-service không lấy được active-window state. Không gọi `element.click()`; chuỗi
 * DOM giữ đúng thứ tự chuột thật mà `realClick` bảo vệ: down → focus → up → click.
 */
async function dispatchPointerSequence(selector, missingIsOkay = false) {
  return browser.execute((sel, allowMissing) => {
    const element = document.querySelector(sel)
    if (!(element instanceof HTMLElement)) {
      if (allowMissing) return false
      throw new Error(`không tìm thấy ${sel}`)
    }
    element.dispatchEvent(new MouseEvent('mousedown', { bubbles: true, button: 0 }))
    element.focus()
    element.dispatchEvent(new MouseEvent('mouseup', { bubbles: true, button: 0 }))
    element.dispatchEvent(new MouseEvent('click', { bubbles: true, button: 0 }))
    return true
  }, selector, missingIsOkay)
}

async function setInputValue(value) {
  await browser.execute((next) => {
    const input = document.querySelector('.gs-input')
    if (!(input instanceof HTMLInputElement)) throw new Error('không tìm thấy .gs-input')
    input.value = next
    input.dispatchEvent(new Event('input', { bubbles: true }))
  }, value)
}

describe('Story 3.5 review — IPC thật, event nền và modal độc quyền', () => {
  before(async () => {
    await waitForIpcBridge()
  })

  afterEach(async () => {
    await unregisterScanListener()
  })

  it('config persisted 6 rồi 5 điều khiển hai Work mới; command trả trước event', async () => {
    await persistThreshold(6)
    await registerScanListener()

    const eventsWhenSixReturned = await importEnglishWork('Story 3.5 review threshold 6')
    expect(eventsWhenSixReturned).toBe(0)
    const eventSix = await waitForOneScanEvent()
    expect(eventSix.outcome).toBe('completed')
    expect((await pendingCandidates()).some((row) => row.source_term === TERM)).toBe(false)

    await unregisterScanListener()
    await persistThreshold(5)
    await registerScanListener()

    const eventsWhenFiveReturned = await importEnglishWork('Story 3.5 review threshold 5')
    expect(eventsWhenFiveReturned).toBe(0)
    const eventFive = await waitForOneScanEvent()
    expect(eventFive.outcome).toBe('completed')

    const row = (await pendingCandidates()).find((candidate) => candidate.source_term === TERM)
    expect(row).toBeDefined()
    expect(row.occurrence_count).toBe(5)
    expect(row.context_example).toContain(TERM)
  })

  it('modal mở chặn global shortcut; giá trị sai không lưu, giá trị đúng lưu rồi tự đóng', async () => {
    // Ép tiền đề khác Workspace trước khi mở modal. Nếu để nguyên trạng thái thừa kế từ
    // ca import, `Mod+2` có chạy xuyên modal vẫn giữ cùng nhãn và test xanh giả.
    await browser.keys(['Meta', '1'])
    await browser.waitUntil(async () => (await activeModeLabel()) === 'Library', {
      timeout: 10_000,
      interval: 50,
      timeoutMsg: 'không ép được mode ban đầu sang Library trước ca modal',
    })
    const opener = await $('[data-glossary-settings-open]')
    await opener.waitForDisplayed({ timeout: 30_000 })
    const modeBefore = await activeModeLabel()
    expect(modeBefore).toBe('Library')
    expect(modeBefore).not.toBe('Workspace')

    await realClick(opener)
    const openedByDriver = await browser.execute(() => document.querySelector('.gs-panel') !== null)
    if (!openedByDriver) await dispatchPointerSequence('[data-glossary-settings-open]')
    await browser.waitUntil(
      async () => browser.execute(() => document.querySelector('.gs-panel') !== null),
      { timeout: 10_000, interval: 50, timeoutMsg: 'modal Glossary không hiện sau pointer sequence' },
    )

    await setInputValue('0')
    const invalid = await browser.execute(() => ({
      saveDisabled: document.querySelector('.gs-save')?.disabled ?? false,
      alertVisible: document.querySelector('.gs-alert') !== null,
    }))
    expect(invalid.saveDisabled).toBe(true)
    expect(invalid.alertVisible).toBe(true)

    await browser.execute(() => {
      window.dispatchEvent(new KeyboardEvent('keydown', {
        bubbles: true,
        cancelable: true,
        code: 'Digit2',
        key: '2',
        metaKey: true,
      }))
    })
    expect(await browser.execute(() => document.querySelector('.gs-panel') !== null)).toBe(true)
    expect(await activeModeLabel()).toBe(modeBefore)

    await setInputValue('7')
    expect(await browser.execute(() => !document.querySelector('.gs-save')?.disabled)).toBe(true)
    const save = await $('.gs-save')
    await realClick(save)
    const closedByDriver = await browser.execute(() => document.querySelector('.gs-panel') === null)
    // Save thật có thể hoàn tất giữa phép kiểm trên và fallback. Helper kiểm lại selector
    // BÊN TRONG cùng lượt `execute`; modal đã đóng là thành công, không phải lỗi fixture.
    if (!closedByDriver) await dispatchPointerSequence('.gs-save', true)
    await browser.waitUntil(
      async () => browser.execute(() => document.querySelector('.gs-panel') === null),
      { timeout: 10_000, interval: 50, timeoutMsg: 'save thành công nhưng modal không tự đóng' },
    )

    const persisted = await browser.execute(async () => {
      const config = await window.__TAURI_INTERNALS__.invoke('bootstrap_config')
      return config.glossary_scan_threshold
    })
    expect(persisted).toBe(7)
  })
})
