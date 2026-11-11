import { createServer } from 'node:http'
import { realClick } from '../support/pointer.mjs'

let server
let endpoint
let fail = false
const requests = []

async function invoke(command, args = {}) {
  return browser.execute(async (c, a) => window.__TAURI_INTERNALS__.invoke(c, a), command, args)
}

async function queue(action, argument) {
  return browser.execute(async (name, arg) => {
    const state = await import('/src/glossaryQueueState.ts')
    if (name === 'inspect') return { count: state.queueRows.value.length, wait: state.QUICK_AUTO_WAIT_MS }
    return state[name](arg)
  }, action, argument)
}

async function enable(enabled) {
  await invoke('quick_ai_config_set_auto', { tier: 'global', enabled })
  await browser.execute(async () => {
    await (await import('/src/quickAiConfigState.ts')).refreshQuickAiConfigReadiness()
  })
}

async function settle(index) {
  await queue('setGlossaryQueueCursor', index)
  await $('[data-gq-context-ref][data-kind="none"]').waitForExist({ timeout: 10_000 })
}

describe('Story 15-5: automatic translation of the settled queue row', () => {
  before(async () => {
    server = createServer((req, res) => {
      let body = ''
      req.on('data', chunk => { body += chunk })
      req.on('end', () => {
        if (req.method !== 'POST' || !req.url.endsWith('/chat/completions')) {
          res.writeHead(404).end()
          return
        }
        requests.push({ at: Date.now(), body })
        if (fail) { res.writeHead(503).end('fixture failure'); return }
        res.setHeader('Content-Type', 'text/event-stream')
        res.write(`data: ${JSON.stringify({ choices: [{ delta: { content: 'Ban dich tham khao.' }, finish_reason: null }] })}\n\n`)
        res.write(`data: ${JSON.stringify({ choices: [{ delta: {}, finish_reason: 'stop' }] })}\n\n`)
        res.end('data: [DONE]\n\n')
      })
    })
    await new Promise(resolve => server.listen(0, '127.0.0.1', resolve))
    endpoint = `http://127.0.0.1:${server.address().port}/v1`
    await browser.waitUntil(async () => browser.execute(() =>
      window.__TAURI_INTERNALS__ !== undefined && (document.querySelector('#app')?.children.length ?? 0) > 0),
    { timeout: 30_000, interval: 100 })
    await invoke('put_config', { kind: 'app_config', key: 'glossary_scan_threshold', value: '5' })
    await invoke('quick_ai_config_save_fields', {
      tier: 'global',
      fields: [
        { field: 'quick_provider', value: 'ollama' },
        { field: 'quick_endpoint', value: endpoint },
        { field: 'quick_model', value: 'aura-auto-fixture' },
      ],
    })
    const names = ['Alpha', 'Bravo', 'Charlie', 'Delta', 'Echo', 'Foxtrot', 'Gamma', 'Hotel', 'India', 'Juliet']
    const text = names.flatMap(name => Array.from({ length: 5 }, (_, i) =>
      `a beast called AuraAuto Beast${name} appeared at hour ${i}.`)).join('\n\n')
    await invoke('create_work_from_text', { name: 'Quick auto queue fixture', sourceLang: 'en', genre: 'general', text })
    await browser.waitUntil(async () => (await invoke('glossary_pending_candidates')).length >= 10,
      { timeout: 30_000, interval: 100 })
  })

  after(async () => {
    await queue('closeGlossaryQueue')
    await new Promise(resolve => server.close(resolve))
  })

  it('measures ten-row traversal and stopped-row latency without prefetch', async () => {
    await enable(false)
    await queue('openGlossaryQueue')
    await $('.gq-scrim').waitForDisplayed({ timeout: 10_000 })
    const population = await queue('inspect')
    const offBefore = requests.length
    for (let i = 0; i < 10; i += 1) { await queue('setGlossaryQueueCursor', i); await browser.pause(60) }
    await browser.pause(population.wait + 300)
    const offCalls = requests.length - offBefore
    expect(offCalls).toBe(0)
    await queue('setGlossaryQueueCursor', 0)
    await enable(true)
    const before = requests.length
    let stoppedAt
    for (let i = 0; i < 10; i += 1) {
      await queue('setGlossaryQueueCursor', i)
      stoppedAt = Date.now()
      await browser.pause(60)
    }
    await browser.waitUntil(() => requests.length > before, { timeout: 10_000, interval: 20 })
    await $('[data-gq-context-ref][data-kind="ai"]').waitForExist({ timeout: 10_000 })
    await browser.pause(population.wait + 150)
    expect(requests.length - before).toBe(1)
    console.log(`AURA_AUTO_MEASUREMENT ${JSON.stringify({ build: 'debug --features wdio; Vite development; WKWebView', population: population.count, traversed: 10, row_pause_ms: 60, candidate_wait_ms: population.wait, off_calls: offCalls, auto_calls: requests.length - before, stop_to_provider_ms: requests[before].at - stoppedAt })}`)
  })

  it('measures a 600 ms intermediate stop before settling on another row', async () => {
    const { wait, count } = await queue('inspect')
    const before = requests.length
    await queue('setGlossaryQueueCursor', 4)
    await browser.pause(600)
    await queue('setGlossaryQueueCursor', 5)
    const stoppedAt = Date.now()
    await $('[data-gq-context-ref][data-kind="ai"]').waitForExist({ timeout: 10_000 })
    await browser.pause(wait + 150)
    const delta = requests.length - before
    expect(delta).toBe(wait < 600 ? 2 : 1)
    console.log(`AURA_AUTO_INTERMEDIATE ${JSON.stringify({ population: count, candidate_wait_ms: wait, intermediate_stop_ms: 600, auto_calls: delta, final_stop_to_provider_ms: requests.at(-1).at - stoppedAt })}`)
  })

  it('disabling or closing during the wait makes no provider call', async () => {
    await enable(false)
    await settle(0)
    const { wait } = await queue('inspect')
    const before = requests.length
    await enable(true)
    await enable(false)
    await browser.pause(wait + 250)
    expect(requests.length - before).toBe(0)
    await enable(true)
    await queue('closeGlossaryQueue')
    await browser.pause(wait + 250)
    expect(requests.length - before).toBe(0)
  })

  it('does not retry a failed row on revisit, allows manual retry, and resets on reopen', async () => {
    await enable(false)
    await queue('openGlossaryQueue')
    await settle(1)
    fail = true
    const before = requests.length
    await enable(true)
    await $('[data-gq-context-ref][data-kind="failed"]').waitForExist({ timeout: 15_000 })
    expect(requests.length - before).toBe(1)
    await queue('setGlossaryQueueCursor', 2)
    await queue('setGlossaryQueueCursor', 1)
    const { wait } = await queue('inspect')
    await browser.pause(wait + 250)
    expect(requests.length - before).toBe(1)
    fail = false
    await realClick(await $('[data-gq-context-translate]'))
    await $('[data-gq-context-ref][data-kind="ai"]').waitForExist({ timeout: 10_000 })
    expect(requests.length - before).toBe(2)
    await queue('closeGlossaryQueue')
    await enable(false)
    await queue('openGlossaryQueue')
    await settle(3)
    fail = true
    await enable(true)
    await $('[data-gq-context-ref][data-kind="failed"]').waitForExist({ timeout: 15_000 })
    await queue('closeGlossaryQueue')
    fail = false
    const reopenBefore = requests.length
    await queue('openGlossaryQueue')
    await $('[data-gq-context-ref][data-kind="ai"]').waitForExist({ timeout: 10_000 })
    expect(requests.length - reopenBefore).toBe(1)
  })
})
