/**
 * Import-preview block tier by pointer, in a real WKWebView, on the real URL path.
 *
 * A local `node:http` server on 127.0.0.1 serves two article pages; the spec drops one block,
 * sets a keep range and confirms, then reads the imported segments back from the Work.
 */
import { createServer } from 'node:http'
import { realClick } from '../support/pointer.mjs'

const RUN_TAG = `${Date.now() % 1_000_000}`
const WORK = `e2e-146-url-${RUN_TAG}`
const SCREEN = '.ip-screen'
const BLOCK = '.ip-block'

const AD = 'QUANGCAO mua ngay hang giam gia khong the bo lo hom nay'
const FOOTER = 'CHANTRANG ban quyen thuoc ve trang web va cac ben lien quan khac'

function page(tag) {
  const body = [
    AD,
    `Doan mot cua ${tag} co du chu de duoc Readability chon lam noi dung chinh cua trang, nhieu chu hon de vuot nguong do dai toi thieu.`,
    `Doan hai cua ${tag} tiep tuc noi dung that su cua bai viet, khong phai menu hay quang cao, du dai de cham diem cao cho khoi nay.`,
    `Doan ba cua ${tag} dong y nghia, giu cho tong do dai van ban vuot qua nguong toi thieu can thiet de Readability tin day la bai viet that.`,
    FOOTER,
  ]
  return `<html><head><title>${tag}</title></head><body><article><h1>${tag}</h1>${body
    .map((p) => `<p>${p}</p>`)
    .join('\n')}</article></body></html>`
}

let server
let port

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

async function blockStates() {
  return browser.execute(() =>
    [...document.querySelectorAll('.ip-block')]
      .map((el) => ({
        index: Number(el.id.replace('ip-block-', '')),
        dropped: el.classList.contains('ip-block-dropped'),
        focused: el.classList.contains('ip-block-focused'),
        text: el.querySelector('.ip-block-text')?.textContent?.trim() ?? '',
      }))
      .sort((a, b) => a.index - b.index),
  )
}

async function openAllBands() {
  for (;;) {
    const closed = await $$(`${SCREEN} .ip-band:not(.ip-band-open)`)
    if (closed.length === 0) return
    await realClick(closed[0])
    await browser.pause(50)
  }
}

async function settledBlockStates() {
  await openAllBands()
  return blockStates()
}

async function clickBlock(index) {
  await realClick(await $(`${SCREEN} #ip-block-${index} .ip-block-text`))
}

async function clickAction(label) {
  const buttons = await $$(`${SCREEN} ${BLOCK}.ip-block-focused .ip-block-actions button`)
  for (const button of buttons) {
    if ((await button.getText()).trim() === label) {
      await realClick(button)
      return
    }
  }
  throw new Error(`no action button "${label}" on the aimed block`)
}

describe('Story 14-6: URL import preview block tier by pointer', () => {
  before(async () => {
    server = createServer((req, res) => {
      res.setHeader('Content-Type', 'text/html; charset=utf-8')
      res.end(page(req.url === '/b' ? 'trangB' : 'trangA'))
    })
    await new Promise((resolve) => server.listen(0, '127.0.0.1', resolve))
    port = server.address().port
  })

  after(async () => {
    await new Promise((resolve) => server.close(resolve))
  })

  it('aim a block, drop it, set a keep range, confirm, and the imported Work matches', async () => {
    await browser.waitUntil(async () => browser.execute(() => window.__TAURI_INTERNALS__ !== undefined), {
      timeout: 30_000,
      interval: 250,
    })
    await $('[data-library-import-open]').waitForDisplayed({ timeout: 30_000 })
    await realClick(await $('[data-library-import-open]'))
    await $('[data-library-url-screen-open]').waitForDisplayed({ timeout: 10_000 })
    await realClick(await $('[data-library-url-screen-open]'))
    const screen = await $('[data-library-url-import-screen]')
    await screen.waitForDisplayed({ timeout: 10_000 })
    await (await screen.$('textarea')).setValue(`http://127.0.0.1:${port}/a\nhttp://127.0.0.1:${port}/b`)
    await (await screen.$('input[type="text"]')).setValue(WORK)
    await realClick(await screen.$('[data-import-preview-open]'))

    await $(SCREEN).waitForDisplayed({ timeout: 60_000 })
    await browser.waitUntil(async () => (await $$(`${SCREEN} ${BLOCK}, ${SCREEN} .ip-band`)).length >= 1, {
      timeout: 30_000,
      timeoutMsg: 'tier 2 never listed any block',
    })
    await browser.waitUntil(async () => (await settledBlockStates()).length >= 5, {
      timeout: 30_000,
      timeoutMsg: 'tier 2 never listed the five blocks',
    })

    const startList = await blockStates()
    const adIndex = startList.findIndex((b) => b.text.includes('QUANGCAO'))
    expect(adIndex).toBeGreaterThanOrEqual(0)

    await clickBlock(adIndex)
    const aimed = await blockStates()
    expect(aimed.filter((b) => b.focused)).toHaveLength(1)
    expect(aimed[adIndex].focused).toBe(true)
    await clickAction('Loại khối này')
    await browser.waitUntil(async () => (await settledBlockStates())[adIndex].dropped, { timeout: 15_000 })

    const footerIndex = startList.findIndex((b) => b.text.includes('CHANTRANG'))
    await clickBlock(adIndex + 1)
    await clickAction('Đầu vùng giữ')
    await clickBlock(footerIndex - 1)
    await clickAction('Cuối vùng giữ')
    await browser.waitUntil(async () => (await settledBlockStates())[footerIndex].dropped, { timeout: 15_000 })
    const after = await settledBlockStates()
    expect(after[adIndex].dropped).toBe(true)
    expect(after.slice(adIndex + 1, footerIndex).every((b) => !b.dropped)).toBe(true)

    const [prev, next] = await $$(`${SCREEN} .ip-chapter-cursor-row button`)
    await realClick(next)
    await browser.waitUntil(async () => (await $(`${SCREEN} .ip-chapter-cursor-note`).getText()).includes('2/2'), {
      timeout: 15_000,
    })
    await browser.waitUntil(async () => (await settledBlockStates()).length >= 5, { timeout: 15_000 })
    for (const button of await $$(`${SCREEN} ${BLOCK}.ip-block-focused .ip-block-actions button`)) {
      expect(await button.isEnabled()).toBe(false)
    }
    await realClick(prev)
    await browser.waitUntil(async () => (await $(`${SCREEN} .ip-chapter-cursor-note`).getText()).includes('1/2'), {
      timeout: 15_000,
    })

    await realClick(await $(`${SCREEN} .ip-footer .ip-act-primary`))
    await $(SCREEN).waitForExist({ reverse: true, timeout: 60_000 })

    const chapters = await ipc('list_chapters')
    expect(chapters.length).toBe(2)
    await ipc('open_chapter', { chapterId: chapters[0].chapter_id })
    const loaded = await ipc('read_open_chapter_segments')
    const text = loaded.segments.map((s) => s.source_text).join('\n')
    expect(text).toContain('Doan mot cua trangA')
    expect(text).not.toContain('QUANGCAO')
    expect(text).not.toContain('CHANTRANG')
  })
})
