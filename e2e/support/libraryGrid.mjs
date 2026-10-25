import { realClick } from './pointer.mjs'

export const WORK_CELL = '.works-grid .work-cell'
const STATUS_CHIP = '[data-library-chip="status"]'
const FILTER_ANY = '[data-lifecycle-filter="not_started"]'

export async function workCellNames() {
  return browser.execute((sel) => Array.from(document.querySelectorAll(`${sel} .work-name`)).map((n) => (n.textContent || '').trim()), WORK_CELL)
}

export async function openChip(chip) {
  const menu = await $(`[data-library-chip="${chip}"]`)
  await menu.waitForExist({ timeout: 20_000 })
  if (!(await menu.getProperty('open'))) await realClick(await menu.$('summary'))
  await browser.waitUntil(async () => menu.getProperty('open'), { timeout: 5_000, timeoutMsg: `chip ${chip} did not open` })
  return menu
}

async function waitUntilFilterEnabled() {
  await browser.waitUntil(async () => (await $(FILTER_ANY)).isEnabled(), {
    timeout: 20_000,
    timeoutMsg: 'the status filter stayed disabled (a list load never finished)',
  })
}

/** Forces a fresh `library_list_works` by switching one status filter on and off (the chip is the only reload path left). */
export async function reloadWorks() {
  await openChip('status')
  await waitUntilFilterEnabled()
  await realClick(await $(FILTER_ANY))
  await waitUntilFilterEnabled()
  await realClick(await $(FILTER_ANY))
  await waitUntilFilterEnabled()
  await realClick(await $(STATUS_CHIP).$('summary'))
}

export async function currentWorkName() {
  return browser.execute((sel) => {
    const name = document.querySelector(`${sel}[aria-current="true"] .work-name`)
    return name === null ? '' : (name.textContent || '').trim()
  }, WORK_CELL)
}

/** Presses the named card (mousedown aims it) and waits until it is the selected one. */
export async function aimWork(name) {
  await browser.waitUntil(async () => (await workCellNames()).includes(name), {
    timeout: 30_000,
    timeoutMsg: `the card "${name}" never appeared in the grid`,
  })
  const names = await $$(`${WORK_CELL} .work-name`)
  for (const node of names) {
    if ((await node.getText()).trim() === name) {
      await realClick(node)
      break
    }
  }
  await browser.waitUntil(async () => (await currentWorkName()) === name, {
    timeout: 10_000,
    timeoutMsg: `the grid cursor never reached "${name}"`,
  })
  const focused = await browser.execute((sel) => {
    const cell = document.querySelector(`${sel}[aria-current="true"]`)
    if (cell === null) return 'no-cell'
    return document.activeElement === cell ? 'ok' : `active=${document.activeElement?.tagName}`
  }, WORK_CELL)
  expect(focused).toBe('ok')
}

/** Enter on the focused selected card runs library.open_work. */
export async function openAimedWork() {
  const focusedOnCard = await browser.execute(() => document.activeElement?.matches('.work-cell') ?? false)
  if (!focusedOnCard) {
    await browser.execute((sel) => document.querySelector(`${sel}[aria-current="true"]`)?.focus(), WORK_CELL)
  }
  await browser.keys(['Enter'])
}
