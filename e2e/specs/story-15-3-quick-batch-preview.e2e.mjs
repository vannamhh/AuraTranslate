/**
 * `Dịch các khối đang thấy` in the URL import preview: a half-finished batch keeps the blocks it
 * already closed, reopening the preview shows them with no new provider call, no Work is created,
 * and a translation growing above the aimed block does not move it on screen.
 *
 *     npm run test:e2e -- --spec e2e/specs/story-15-3-quick-batch-preview.e2e.mjs
 */
import { createServer } from 'node:http'

import { realClick } from '../support/pointer.mjs'

const MODEL = 'aura-fake-model'
const SCREEN = '.ip-screen'
const PARAGRAPHS = 30
const CLOSED_BEFORE_HOLD = 2
const AIMED = 'Doan 08 '

function paragraph(n) {
  const id = String(n).padStart(2, '0')
  return `Doan ${id} cua trang thu nghiem mang noi dung that su cua bai viet, du dai de Readability giu lai va de moi khoi chiem it nhat hai dong tren man hinh xem truoc, so ${id}.`
}

const PAGE = `<html><head><title>trangQ</title></head><body><article><h1>trangQ</h1>${Array.from(
  { length: PARAGRAPHS },
  (_, i) => `<p>${paragraph(i + 1)}</p>`,
).join('\n')}</article></body></html>`

let server
let port
let calls = 0
let sentBlocks = 0
let holdOpen = true
let startHeld = true
let startRelease = () => {}
let release = () => {}

function blocksOf(body) {
  const content = (JSON.parse(body).messages ?? []).map((m) => String(m.content)).join('\n')
  return [...content.matchAll(/<<<SEGMENT (\d+)\n([\s\S]*?)\nSEGMENT>>>/g)].map((m) => ({ n: m[1], text: m[2] }))
}

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

async function openPreview() {
  const screen = await $('[data-library-url-import-screen]')
  if (!(await screen.isDisplayed())) {
    await $('[data-library-import-open]').waitForDisplayed({ timeout: 30_000 })
    await realClick(await $('[data-library-import-open]'))
    await $('[data-library-url-screen-open]').waitForDisplayed({ timeout: 10_000 })
    await realClick(await $('[data-library-url-screen-open]'))
    await screen.waitForDisplayed({ timeout: 10_000 })
  }
  await (await screen.$('textarea')).setValue(`http://127.0.0.1:${port}/a`)
  await (await screen.$('input[type="text"]')).setValue(`e2e-153-${Date.now() % 1_000_000}`)
  await realClick(await screen.$('[data-import-preview-open]'))
  await $(SCREEN).waitForDisplayed({ timeout: 60_000 })
  await browser.waitUntil(async () => (await $$(`${SCREEN} .ip-block`)).length >= 6, {
    timeout: 30_000,
    timeoutMsg: 'tier 2 never listed the blocks',
  })
}

async function closePreview() {
  await realClick(await $(`${SCREEN} .ip-actions .ip-act:not(.ip-act-primary)`))
  await $(SCREEN).waitForExist({ reverse: true, timeout: 30_000 })
}

async function translationIndices() {
  return browser.execute(() =>
    [...document.querySelectorAll('.ip-block')]
      .filter((el) => el.querySelector('[data-ip-block-translation-text]'))
      .map((el) => Number(el.id.replace('ip-block-', ''))),
  )
}

async function blockTop(index) {
  return browser.execute((i) => document.querySelector(`#ip-block-${i}`).getBoundingClientRect().top, index)
}

describe('Story 15-3: Dịch các khối đang thấy in the URL import preview', () => {
  before(async () => {
    server = createServer((req, res) => {
      const chunks = []
      req.on('data', (c) => chunks.push(c))
      req.on('end', async () => {
        if (req.method === 'GET') {
          res.setHeader('Content-Type', 'text/html; charset=utf-8')
          res.end(PAGE)
          return
        }
        if (req.method !== 'POST' || !req.url.endsWith('/chat/completions')) {
          res.statusCode = 404
          res.end()
          return
        }
        calls += 1
        const blocks = blocksOf(Buffer.concat(chunks).toString('utf8'))
        sentBlocks += blocks.length
        let aborted = false
        res.on('close', () => {
          aborted = true
          startRelease()
          release()
        })
        res.setHeader('Content-Type', 'text/event-stream')
        if (startHeld) await new Promise((resolve) => (startRelease = resolve))
        const send = (frame) => res.write(`data: ${JSON.stringify(frame)}\n\n`)
        for (const [i, block] of blocks.entries()) {
          if (aborted) return
          if (i === CLOSED_BEFORE_HOLD && holdOpen) await new Promise((resolve) => (release = resolve))
          if (aborted) return
          const reply = `<<<SEGMENT ${block.n}\nVI ${block.text.slice(0, 20)}\nSEGMENT>>>\n`
          send({ choices: [{ delta: { content: reply }, finish_reason: null }] })
          await new Promise((resolve) => setTimeout(resolve, 30))
        }
        send({ choices: [{ delta: {}, finish_reason: 'stop' }] })
        send({ choices: [], usage: { prompt_tokens: 11, completion_tokens: 7, total_tokens: 18 } })
        res.end('data: [DONE]\n\n')
      })
    })
    await new Promise((resolve) => server.listen(0, '127.0.0.1', resolve))
    port = server.address().port
  })

  after(async () => {
    release()
    await new Promise((resolve) => server.close(resolve))
  })

  it('keeps the closed blocks across cancel and reopen with no new call and no Work', async () => {
    await browser.waitUntil(async () => browser.execute(() => window.__TAURI_INTERNALS__ !== undefined), {
      timeout: 30_000,
      interval: 250,
    })
    await ipc('ai_config_save_fields', {
      tier: 'global',
      fields: [
        { field: 'provider', value: 'ollama' },
        { field: 'endpoint', value: `http://127.0.0.1:${port}/v1` },
        { field: 'model', value: MODEL },
      ],
    })
    const worksBefore = JSON.stringify(await ipc('library_list_works'))

    await openPreview()
    const aimedIndex = await browser.execute((needle) => {
      const hit = [...document.querySelectorAll('.ip-block')].find((el) => el.textContent.includes(needle))
      return hit ? Number(hit.id.replace('ip-block-', '')) : -1
    }, AIMED)
    expect(aimedIndex).toBeGreaterThanOrEqual(0)
    await browser.execute((i) => document.querySelector(`#ip-block-${i}`).scrollIntoView({ block: 'center' }), aimedIndex)
    await realClick(await $(`${SCREEN} #ip-block-${aimedIndex} .ip-block-text`))

    await browser.waitUntil(async () => (await $('[data-ip-quick-translate]').isEnabled()), { timeout: 10_000 })
    await browser.pause(300)
    const topBefore = await blockTop(aimedIndex)
    await browser.execute(() =>
      document.querySelector('[data-ip-quick-translate]').dispatchEvent(new MouseEvent('click', { bubbles: true })),
    )
    await $('[data-ip-quick-running]').waitForDisplayed({ timeout: 10_000 })
    await browser.waitUntil(async () => calls === 1, { timeout: 10_000, timeoutMsg: 'the provider never got the batch' })
    startHeld = false
    startRelease()
    await browser.waitUntil(async () => (await translationIndices()).length >= CLOSED_BEFORE_HOLD, {
      timeout: 30_000,
      timeoutMsg: 'the first closed blocks never appeared',
    })
    const closed = await translationIndices()
    expect(closed.every((i) => i < aimedIndex)).toBe(true)
    expect(Math.abs((await blockTop(aimedIndex)) - topBefore)).toBeLessThanOrEqual(1)

    await realClick(await $('[data-ip-quick-cancel]'))
    await $('[data-ip-quick-translate]').waitForDisplayed({ timeout: 15_000 })
    expect(await translationIndices()).toEqual(closed)
    expect(calls).toBe(1)

    await closePreview()
    holdOpen = false
    await openPreview()
    await browser.waitUntil(async () => (await translationIndices()).length >= CLOSED_BEFORE_HOLD, {
      timeout: 30_000,
      timeoutMsg: 'the stored blocks did not come back from the store',
    })
    expect(await translationIndices()).toEqual(closed)
    expect(calls).toBe(1)
    expect(JSON.stringify(await ipc('library_list_works'))).toBe(worksBefore)
    expect(sentBlocks).toBeGreaterThanOrEqual(CLOSED_BEFORE_HOLD)
  })
})
