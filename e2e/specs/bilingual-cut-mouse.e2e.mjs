/**
 * Bilingual import preview, mismatched pairs split by pointer, in a real WKWebView.
 *
 * A two-row CSV where both rows have more source sentences than target sentences. The spec moves
 * between rows with the pair buttons, clicks cut marks, confirms, and reads the segments back.
 */
import { mkdtempSync, writeFileSync } from 'node:fs'
import { tmpdir } from 'node:os'
import { join } from 'node:path'
import { realClick } from '../support/pointer.mjs'

const RUN_TAG = `${Date.now() % 1_000_000}`
const WORK = `e2e-147-cut-${RUN_TAG}`
const SCREEN = '.bip-screen'
const HEADING = `${SCREEN} .bip-mismatch-heading`

async function ipc(command, args = {}) {
  const json = await browser.execute(
    async (name, payload) => {
      try {
        return JSON.stringify({ ok: true, value: await window.__TAURI_INTERNALS__.invoke(name, payload) })
      } catch (err) {
        return JSON.stringify({ ok: false, error: String(err && err.code ? err.code : err) })
      }
    },
    command,
    args,
  )
  const out = JSON.parse(json)
  if (!out.ok) throw new Error(`${command} failed: ${out.error}`)
  return out.value
}

async function pairButton(label) {
  for (const button of await $$(`${SCREEN} .bip-mismatch-heading-row button`)) {
    if ((await button.getText()).trim() === label) return button
  }
  throw new Error(`no pair button "${label}"`)
}

async function headingText() {
  return (await $(HEADING).getText()).trim()
}

async function targetStates() {
  return browser.execute(() =>
    [...document.querySelectorAll('.bip-cut-target')].map((el) => ({
      glyph: el.textContent,
      pressed: el.getAttribute('aria-pressed'),
    })),
  )
}

describe('Story 14-7: bilingual import cut by pointer', () => {
  it('moves between mismatched pairs, cuts and uncuts by click, keeps a matched row, imports the pieces', async () => {
    await browser.waitUntil(async () => browser.execute(() => window.__TAURI_INTERNALS__ !== undefined), {
      timeout: 30_000,
      interval: 250,
    })
    const dir = mkdtempSync(join(tmpdir(), 'e2e-147-'))
    const csv = join(dir, 'lech.csv')
    writeFileSync(csv, '甲。乙。,Mot hai\n丙。丁。戊。,Ba bon nam\n', 'utf8')

    await $('[data-library-import-open]').waitForDisplayed({ timeout: 30_000 })
    await realClick(await $('[data-library-import-open]'))
    const form = await $('.li-panel .import-form')
    await form.waitForDisplayed({ timeout: 10_000 })
    await (await form.$('.field input[type="text"]')).setValue(WORK)
    await realClick(await (await form.$('[data-import-source-lang] input[type="radio"][value="zh"]')).$('..'))
    const inputs = await form.$$('.field input[type="text"]')
    await inputs[inputs.length - 1].setValue(csv)
    await realClick(await form.$('[data-bilingual-import-preview-open]'))
    await $(SCREEN).waitForDisplayed({ timeout: 30_000 })
    await $(HEADING).waitForDisplayed({ timeout: 15_000 })

    expect(await headingText()).toContain('1 / 2')
    await realClick(await pairButton('Cặp lệch sau'))
    expect(await headingText()).toContain('2 / 2')
    await realClick(await pairButton('Cặp lệch sau'))
    expect(await headingText()).toContain('1 / 2')
    await realClick(await pairButton('Cặp lệch sau'))

    const targets = await $$(`${SCREEN} .bip-cut-target`)
    expect(targets.length).toBeGreaterThanOrEqual(3)
    await realClick(targets[0])
    await browser.waitUntil(async () => (await targetStates())[0]?.glyph === '‖', { timeout: 15_000 })
    expect((await headingText()).includes('Đã khớp')).toBe(false)

    await realClick((await $$(`${SCREEN} .bip-cut-target`))[0])
    await browser.waitUntil(async () => (await targetStates())[0]?.glyph === '¦', { timeout: 15_000 })

    await realClick((await $$(`${SCREEN} .bip-cut-target`))[0])
    await browser.waitUntil(async () => (await targetStates())[0]?.glyph === '‖', { timeout: 15_000 })
    await realClick((await $$(`${SCREEN} .bip-cut-target`))[2])
    await browser.waitUntil(async () => (await headingText()).includes('Đã khớp'), { timeout: 15_000 })
    expect((await targetStates())[2].pressed).toBe('true')

    await realClick(await pairButton('Cặp lệch trước'))
    expect(await headingText()).toContain('1 / 1')
    await realClick((await $$(`${SCREEN} .bip-cut-target`))[0])
    await browser.waitUntil(async () => (await headingText()).includes('Đã khớp'), { timeout: 15_000 })
    expect(await $$(`${SCREEN} .bip-mismatch-heading-row button`)).toHaveLength(0)

    const primary = await $(`${SCREEN} .bip-act-primary`)
    await browser.waitUntil(async () => primary.isEnabled(), { timeout: 15_000 })
    await realClick(primary)
    await $(SCREEN).waitForExist({ reverse: true, timeout: 30_000 })

    const chapters = await ipc('list_chapters')
    await ipc('open_chapter', { chapterId: chapters[0].chapter_id })
    const loaded = await ipc('read_open_chapter_segments')
    expect(loaded.segments.map((seg) => [seg.source_text, seg.target_text])).toEqual([
      ['甲。', 'Mot'],
      ['乙。', 'hai'],
      ['丙。', 'Ba'],
      ['丁。', 'bon'],
      ['戊。', 'nam'],
    ])
  })
})
