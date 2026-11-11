/**
 * `Dịch thử` in the Glossary queue pays for a sentence once: a second Work holding the same
 * sentence shows the stored translation and the provider receives no further call.
 *
 *     npm run test:e2e -- --spec e2e/specs/story-15-1-quick-translation-cache.e2e.mjs
 */
import { createServer } from 'node:http'

import { realClick } from '../support/pointer.mjs'

const TERM = 'AuraQuick FireDragon'
const MODEL = 'aura-fake-model'
const REPLY = 'Con rong lua xuat hien.'

let server
let port
let calls = 0

async function invoke(command, args) {
  return browser.execute(
    async (c, a) => window.__TAURI_INTERNALS__.invoke(c, a),
    command,
    args,
  )
}

async function workRows() {
  const segments = await invoke('read_open_chapter_segments', {})
  const tm = await invoke('tm_count_pairs', {})
  const glossary = await invoke('glossary_list_entries', {})
  return JSON.stringify({ segments, tm, glossary })
}

async function createWorkAndQueue(name) {
  const sentences = Array.from({ length: 5 }, (_, i) => `a beast called ${TERM} appeared at hour ${i}.`)
  await invoke('create_work_from_text', {
    name,
    sourceLang: 'en',
    genre: 'general',
    text: sentences.join('\n\n'),
  })
  await browser.waitUntil(
    async () => (await invoke('glossary_pending_candidates')).some((r) => r.source_term === TERM),
    { timeout: 30_000, interval: 250, timeoutMsg: 'the import scan never produced the candidate' },
  )
  await browser.keys(['Meta', 'Alt', 'q'])
  await $('.gq-scrim').waitForDisplayed({ timeout: 10_000 })
}

describe('Story 15-1: Dịch thử remembers its result across Works', () => {
  before(async () => {
    server = createServer((req, res) => {
      req.resume()
      req.on('end', () => {
        if (req.method !== 'POST' || !req.url.endsWith('/chat/completions')) {
          res.statusCode = 404
          res.end()
          return
        }
        calls += 1
        res.setHeader('Content-Type', 'text/event-stream')
        const frames = [
          { choices: [{ delta: { content: REPLY }, finish_reason: null }] },
          { choices: [{ delta: {}, finish_reason: 'stop' }] },
          { choices: [], usage: { prompt_tokens: 11, completion_tokens: 7, total_tokens: 18 } },
        ]
        for (const frame of frames) res.write(`data: ${JSON.stringify(frame)}\n\n`)
        res.end('data: [DONE]\n\n')
      })
    })
    await new Promise((resolve) => server.listen(0, '127.0.0.1', resolve))
    port = server.address().port
  })

  after(async () => {
    await new Promise((resolve) => server.close(resolve))
  })

  it('pays once in Work A and shows the stored sentence in Work B with no provider call', async () => {
    await browser.waitUntil(
      async () =>
        browser.execute(
          () => window.__TAURI_INTERNALS__ !== undefined && (document.querySelector('#app')?.children.length ?? 0) > 0,
        ),
      { timeout: 30_000, interval: 250 },
    )
    await invoke('put_config', { kind: 'app_config', key: 'glossary_scan_threshold', value: '5' })
    await invoke('ai_config_save_fields', {
      tier: 'global',
      fields: [
        { field: 'provider', value: 'ollama' },
        { field: 'endpoint', value: `http://127.0.0.1:${port}/v1` },
        { field: 'model', value: MODEL },
      ],
    })

    await createWorkAndQueue('Quick translation A')
    await $('[data-gq-context-ref][data-kind="none"]').waitForDisplayed({ timeout: 15_000 })
    await $('[data-gq-context-translate]').waitForClickable({ timeout: 10_000 })
    const before = await workRows()
    await realClick(await $('[data-gq-context-translate]'))
    const text = await $('[data-gq-context-ref-text]')
    await browser.waitUntil(async () => (await text.getText()).includes(REPLY), {
      timeout: 30_000,
      timeoutMsg: 'the AI sentence never appeared',
    })
    await $('[data-gq-context-retranslate]').waitForDisplayed({ timeout: 10_000 })
    await expect(await $('[data-gq-context-ref-model]').getText()).toContain(MODEL)
    const afterFirst = calls
    await expect(afterFirst).toBe(1)
    await expect(await workRows()).toBe(before)

    await browser.keys(['Escape'])
    await $('.gq-scrim').waitForDisplayed({ timeout: 10_000, reverse: true })

    await createWorkAndQueue('Quick translation B')
    const reopened = await $('[data-gq-context-ref][data-kind="cached"]')
    await reopened.waitForDisplayed({ timeout: 15_000 })
    await expect(await $('[data-gq-context-ref-text]').getText()).toBe(REPLY)
    await expect(await $('[data-gq-context-ref-model]').getText()).toContain(MODEL)
    await expect(await $('[data-gq-context-translate]').isExisting()).toBe(false)
    await expect(calls - afterFirst).toBe(0)
  })
})
