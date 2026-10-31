import { realClick } from '../support/pointer.mjs'

const TRIGGER = '.tools-trigger'
const ITEM = (id) => `[data-tools-item="${id}"]`

const ITEM_IDS = [
  'glossary.manage.open',
  'glossary.settings.open',
  'tm.manage.open',
  'settings.open',
  'ai.proofread.scope.open',
  'ai.proofread.memory.open',
  'export.reviewer_import.open',
  'export.alignment.open',
  'glossary.harvest.open',
  'layout.toggle_panel_headers',
]

const OVERLAYS = [
  ['glossary.manage.open', '.gm-panel'],
  ['tm.manage.open', '.tm-panel'],
  ['settings.open', '.set-panel'],
]

async function activeIsTrigger() {
  return browser.execute(() => document.activeElement?.classList.contains('tools-trigger') ?? false)
}

async function openList() {
  const trigger = await $(TRIGGER)
  await trigger.waitForDisplayed({ timeout: 30_000 })
  await realClick(trigger)
  await $(ITEM('settings.open')).waitForDisplayed({ timeout: 10_000 })
}

describe('Khung app · thanh tiêu đề ba tab và Công cụ ▾', () => {
  it('thanh tiêu đề chỉ có ba tab và Công cụ ▾ ở Library; danh sách có chín mục theo thứ tự IA, không còn Xuất', async () => {
    await browser.keys(['Meta', '1'])
    const trigger = await $(TRIGGER)
    await trigger.waitForDisplayed({ timeout: 30_000 })

    const bar = await browser.execute(() => ({
      tabs: document.querySelectorAll('.titlebar .mode-tab').length,
      loose: document.querySelectorAll('.titlebar .titlebar-act').length,
      layoutSwitch: document.querySelector('.layout-switch') !== null,
    }))
    expect(bar).toEqual({ tabs: 3, loose: 0, layoutSwitch: false })

    await openList()
    const ids = await browser.execute(() =>
      Array.from(document.querySelectorAll('[role="menuitem"], [role="menuitemcheckbox"]')).map((el) => el.getAttribute('data-tools-item')),
    )
    expect(ids).toEqual(ITEM_IDS)
    await browser.keys(['Escape'])
    expect(await activeIsTrigger()).toBe(true)
  })

  for (const [id, panel] of OVERLAYS) {
    it(`mục ${id} mở bằng chuột, và đóng bằng Escape thì tiêu điểm về Công cụ ▾`, async () => {
      await openList()
      await realClick(await $(ITEM(id)))
      await $(panel).waitForDisplayed({ timeout: 10_000 })
      await browser.keys(['Escape'])
      await $(panel).waitForExist({ reverse: true, timeout: 10_000 })
      expect(await activeIsTrigger()).toBe(true)
    })
  }

  it('mục Ngưỡng quét Glossary mở Cài đặt ở mục Glossary dù lần trước ở mục khác, và Escape về Công cụ ▾', async () => {
    await openList()
    await realClick(await $(ITEM('settings.open')))
    await $('.set-panel').waitForDisplayed({ timeout: 10_000 })
    await browser.keys(['Escape'])
    await $('.set-panel').waitForExist({ reverse: true, timeout: 10_000 })

    await openList()
    await realClick(await $(ITEM('glossary.settings.open')))
    await $('.set-panel').waitForDisplayed({ timeout: 10_000 })
    const section = await browser.execute(
      () => document.querySelector('.set-panel [data-settings-section][aria-current="true"]')?.getAttribute('data-settings-section') ?? '',
    )
    await expect(section).toBe('glossary')
    await browser.keys(['Escape'])
    await $('.set-panel').waitForExist({ reverse: true, timeout: 10_000 })
    expect(await activeIsTrigger()).toBe(true)
  })

  it('trên macOS, thanh tiêu đề là vùng kéo có lề cho ba nút, và lớp phủ chừa dải thanh tiêu đề', async () => {
    await openList()
    await realClick(await $(ITEM('settings.open')))
    await $('.set-panel').waitForDisplayed({ timeout: 10_000 })
    let chrome
    try {
      chrome = await browser.execute(() => ({
        platform: document.documentElement.dataset.platform ?? '',
        drag: document.querySelector('.titlebar')?.getAttribute('data-tauri-drag-region') ?? '',
        headerInset: getComputedStyle(document.querySelector('.titlebar')).paddingLeft,
        scrimTop: getComputedStyle(document.querySelector('.set-scrim')).paddingTop,
        titlebarHeight: getComputedStyle(document.documentElement).getPropertyValue('--space-titlebar-height').trim(),
        inset: getComputedStyle(document.documentElement).getPropertyValue('--space-traffic-light-inset').trim(),
      }))
    } finally {
      await browser.keys(['Escape'])
      await $('.set-panel').waitForExist({ reverse: true, timeout: 10_000 })
    }
    expect(chrome).toEqual({
      platform: 'mac',
      drag: 'deep',
      headerInset: chrome.inset,
      scrimTop: chrome.titlebarHeight,
      titlebarHeight: '42px',
      inset: '91px',
    })
  })

  // Enter/Space on a focused button is not drivable here (manual check, deferred-work.md), so the
  // keyboard case covers only what the driver can send: arrows and Escape.
  it('mũi tên đi giữa các mục; Escape đóng danh sách và đưa tiêu điểm về Công cụ ▾', async () => {
    await openList()
    await browser.keys(['ArrowDown', 'ArrowDown', 'ArrowDown'])
    const at = await browser.execute(() => document.activeElement?.getAttribute('data-tools-item') ?? '')
    expect(at).toBe('tm.manage.open')
    await browser.keys(['ArrowUp', 'ArrowUp', 'ArrowUp'])
    const wrapped = await browser.execute(() => document.activeElement?.getAttribute('data-tools-item') ?? '')
    expect(wrapped).toBe('layout.toggle_panel_headers')
    await browser.keys(['Escape'])
    await $(ITEM('settings.open')).waitForExist({ reverse: true, timeout: 10_000 })
    expect(await activeIsTrigger()).toBe(true)
  })

  it('Mod+Alt+B mở Quản lý TM; Mod+N sang Library và mở lớp phủ nhập; Escape đóng và trả tiêu điểm về nút Nhập tài liệu', async () => {
    await browser.keys(['Meta', 'Alt', 'b'])
    await $('.tm-panel').waitForDisplayed({ timeout: 10_000 })
    await browser.keys(['Escape'])
    await $('.tm-panel').waitForExist({ reverse: true, timeout: 10_000 })

    await browser.keys(['Meta', '2'])
    await $('.layout-switch').waitForDisplayed({ timeout: 10_000 })
    await browser.keys(['Meta', 'n'])
    await $('.li-panel .import-form').waitForDisplayed({ timeout: 10_000 })
    const bar = await browser.execute(() => document.querySelector('.layout-switch') !== null)
    expect(bar).toBe(false)
    await browser.waitUntil(
      async () => browser.execute(() => (document.activeElement?.closest('.import-form') ?? null) !== null),
      { timeout: 10_000, interval: 50, timeoutMsg: 'tiêu điểm không vào form trong lớp phủ sau Mod+N' },
    )
    await browser.keys(['Escape'])
    await $('.li-panel').waitForExist({ reverse: true, timeout: 10_000 })
    expect(
      await browser.execute(() => document.activeElement?.hasAttribute('data-library-import-open') ?? false),
    ).toBe(true)
  })
})
