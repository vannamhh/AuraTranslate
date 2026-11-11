/**
 * Automatic translation of the visible blocks in the URL import preview: scrolling in steps and
 * stopping sends each text block in exactly one provider call, no two calls overlap in time, and
 * with the option off the provider receives nothing.
 *
 *     npm run test:e2e -- --spec e2e/specs/story-15-6-quick-auto-preview.e2e.mjs
 */
import { createServer } from 'node:http'

import { realClick } from '../support/pointer.mjs'

const SCREEN = '.ip-screen'
const PARAGRAPHS = 60
const AUTO_WAIT_MS = 800

function paragraph(n, word) {
  const id = String(n).padStart(2, '0')
  return `${word} ${id} cua trang thu nghiem mang noi dung that su cua bai viet, du dai de Readability giu lai va de moi khoi chiem it nhat hai dong tren man hinh xem truoc, so ${id}.`
}

const page = (title, word) =>
  `<html><head><title>${title}</title></head><body><article><h1>${title}</h1>${Array.from(
    { length: PARAGRAPHS },
    (_, i) => `<p>${paragraph(i + 1, word)}</p>`,
  ).join('\n')}</article></body></html>`
const PAGES = { '/a': page('trangA', 'Doan'), '/b': page('trangB', 'Khac') }

let server
let port
const calls = []

function blocksOf(body) {
  const content = (JSON.parse(body).messages ?? []).map((m) => String(m.content)).join('\n')
  return [...content.matchAll(/<<<SEGMENT (\d+)\n([\s\S]*?)\nSEGMENT>>>/g)].map((m) => ({ n: m[1], text: m[2] }))
}

async function ipc(command, args = {}) {
  return browser.execute(async (c, a) => window.__TAURI_INTERNALS__.invoke(c, a), command, args)
}

async function enable(enabled) {
  await ipc('quick_ai_config_set_auto', { tier: 'global', enabled })
  await browser.execute(async () => {
    await (await import('/src/quickAiConfigState.ts')).refreshQuickAiConfigReadiness()
  })
}

async function openPreview(path) {
  const screen = await $('[data-library-url-import-screen]')
  if (!(await screen.isDisplayed())) {
    await $('[data-library-import-open]').waitForDisplayed({ timeout: 30_000 })
    await realClick(await $('[data-library-import-open]'))
    await $('[data-library-url-screen-open]').waitForDisplayed({ timeout: 10_000 })
    await realClick(await $('[data-library-url-screen-open]'))
    await screen.waitForDisplayed({ timeout: 10_000 })
  }
  await (await screen.$('textarea')).setValue(`http://127.0.0.1:${port}${path}`)
  await (await screen.$('input[type="text"]')).setValue(`e2e-156-${Date.now() % 1_000_000}`)
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

async function scrollStep(fraction) {
  return browser.execute((f) => {
    let el = document.querySelector('.ip-block')?.parentElement
    while (el && !/auto|scroll/.test(getComputedStyle(el).overflowY)) el = el.parentElement
    el ??= document.scrollingElement
    const before = el.scrollTop
    el.scrollTop = before + (el === document.scrollingElement ? innerHeight : el.clientHeight) * f
    return { atEnd: el.scrollTop === before, tag: el.className || el.tagName, top: el.scrollTop }
  }, fraction)
}

async function visibleUntranslated() {
  return browser.execute(() =>
    [...document.querySelectorAll('.ip-block')]
      .filter((el) => {
        const r = el.getBoundingClientRect()
        return r.bottom > 0 && r.top < innerHeight && !el.querySelector('[data-ip-block-translation-text]')
      })
      .map((el) => el.id),
  )
}

async function pageBlockIds() {
  return browser.execute(() =>
    [...document.querySelectorAll('.ip-block')].map((el) => /so (\d+)\./.exec(el.textContent)?.[1] ?? el.id),
  )
}

async function settleQuiet() {
  let quietSince = Date.now()
  let seen = calls.length
  while (Date.now() - quietSince < AUTO_WAIT_MS + 700) {
    await browser.pause(100)
    const running = await $('[data-ip-quick-running]').isExisting()
    if (running || calls.length !== seen || calls.some((c) => c.end === undefined)) {
      seen = calls.length
      quietSince = Date.now()
    }
  }
}

describe('Story 15-6: automatic translation of the visible preview blocks', () => {
  before(async () => {
    server = createServer((req, res) => {
      const chunks = []
      req.on('data', (c) => chunks.push(c))
      req.on('end', async () => {
        if (req.method === 'GET') {
          res.setHeader('Content-Type', 'text/html; charset=utf-8')
          res.end(PAGES[req.url] ?? PAGES['/a'])
          return
        }
        if (req.method !== 'POST' || !req.url.endsWith('/chat/completions')) {
          res.statusCode = 404
          res.end()
          return
        }
        const blocks = blocksOf(Buffer.concat(chunks).toString('utf8'))
        const call = { start: Date.now(), end: undefined, indices: blocks.map((b) => /so (\d+)\./.exec(b.text)?.[1] ?? b.text) }
        calls.push(call)
        res.on('close', () => {
          call.end ??= Date.now()
        })
        res.setHeader('Content-Type', 'text/event-stream')
        const send = (frame) => res.write(`data: ${JSON.stringify(frame)}\n\n`)
        for (const block of blocks) {
          send({
            choices: [{ delta: { content: `<<<SEGMENT ${block.n}\nVI ${block.text.slice(0, 20)}\nSEGMENT>>>\n` }, finish_reason: null }],
          })
          await new Promise((resolve) => setTimeout(resolve, 40))
        }
        send({ choices: [{ delta: {}, finish_reason: 'stop' }] })
        send({ choices: [], usage: { prompt_tokens: 11, completion_tokens: 7, total_tokens: 18 } })
        res.end('data: [DONE]\n\n')
        call.end = Date.now()
      })
    })
    await new Promise((resolve) => server.listen(0, '127.0.0.1', resolve))
    port = server.address().port
    await browser.waitUntil(
      async () =>
        browser.execute(() => window.__TAURI_INTERNALS__ !== undefined && (document.querySelector('#app')?.children.length ?? 0) > 0),
      { timeout: 30_000, interval: 250 },
    )
    await ipc('quick_ai_config_save_fields', {
      tier: 'global',
      fields: [
        { field: 'quick_provider', value: 'ollama' },
        { field: 'quick_endpoint', value: `http://127.0.0.1:${port}/v1` },
        { field: 'quick_model', value: 'aura-auto-preview-fixture' },
      ],
    })
  })

  after(async () => {
    await new Promise((resolve) => server.close(resolve))
  })

  it('option on: each text block goes in exactly one call and calls never overlap', async () => {
    await enable(true)
    await openPreview('/a')
    const pageIds = await pageBlockIds()
    const blockCount = pageIds.length
    let steps = 0
    const settleVisible = (label) =>
      browser.waitUntil(async () => (await visibleUntranslated()).length === 0, {
        timeout: 20_000,
        interval: 50,
        timeoutMsg: `${label}: visible blocks never got a translation`,
      })
    await settleVisible('first screen')
    for (; steps < 80; steps += 1) {
      const moved = await scrollStep(0.6)
      if (moved.atEnd) break
      await settleVisible(`step ${steps}`)
    }
    await settleQuiet()

    const sent = calls.flatMap((c) => c.indices)
    const duplicates = sent.filter((n, i) => sent.indexOf(n) !== i)
    expect(duplicates).toEqual([])
    expect(pageIds.filter((id) => !sent.includes(id))).toEqual([])
    expect(sent.filter((id) => !pageIds.includes(id))).toEqual([])
    expect(sent.length).toBe(blockCount)
    const ordered = [...calls].sort((a, b) => a.start - b.start)
    const overlaps = ordered.filter((c, i) => i > 0 && c.start < ordered[i - 1].end)
    expect(overlaps.length).toBe(0)
    expect(calls.length).toBeGreaterThan(1)
    expect(await $('[data-ip-quick-translate]').isExisting()).toBe(false)
    const translated = await browser.execute(
      () => [...document.querySelectorAll('.ip-block')].filter((el) => el.querySelector('[data-ip-block-translation-text]')).length,
    )
    const untranslatedVisible = await browser.execute(
      () =>
        [...document.querySelectorAll('.ip-block')].filter((el) => {
          const r = el.getBoundingClientRect()
          return r.bottom > 0 && r.top < innerHeight && !el.querySelector('[data-ip-block-translation-text]')
        }).length,
    )
    expect(untranslatedVisible).toBe(0)
    const gaps = ordered.slice(1).map((c, i) => c.start - ordered[i].end)
    console.log(
      `AURA_AUTO_PREVIEW_MEASUREMENT ${JSON.stringify({
        build: 'debug --features wdio; Vite development; WKWebView; fake SSE provider',
        population: { paragraphs: PARAGRAPHS, blocks_listed: blockCount, scroll_steps: steps, step_wait: 'until every visible block shows a translation, 20 s cap that fails', step_fraction: 0.6 },
        calls: calls.length,
        blocks_sent: sent.length,
        blocks_translated_in_dom: translated,
        duplicates: duplicates.length,
        overlaps: overlaps.length,
        min_gap_between_calls_ms: gaps.length ? Math.min(...gaps) : null,
        max_gap_between_calls_ms: gaps.length ? Math.max(...gaps) : null,
        auto_wait_ms: AUTO_WAIT_MS,
      })}`,
    )
    await closePreview()
  })

  it('option off: scrolling the whole page sends nothing to the provider', async () => {
    await enable(false)
    const before = calls.length
    await openPreview('/b')
    for (let i = 0; i < 80; i += 1) {
      await browser.pause(150)
      if ((await scrollStep(0.6)).atEnd) break
    }
    await browser.pause(AUTO_WAIT_MS + 500)
    expect(calls.length - before).toBe(0)
    const shown = await browser.execute(() => {
      const blocks = [...document.querySelectorAll('.ip-block')]
      return {
        fresh: blocks.filter((el) => el.textContent.includes('Khac ')).length,
        translated: blocks.filter((el) => el.querySelector('[data-ip-block-translation-text]')).length,
      }
    })
    expect(shown.fresh).toBeGreaterThanOrEqual(6)
    expect(shown.translated).toBe(0)
    console.log(`AURA_AUTO_PREVIEW_OFF ${JSON.stringify({ off_calls: calls.length - before })}`)
    await closePreview()
  })
})
