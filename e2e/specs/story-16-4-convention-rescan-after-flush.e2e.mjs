/**
 * A typed sentence loses its convention mark at the first keystroke with zero IPC while typing,
 * and gets it back, for that sentence only, after the flush.
 *
 *     npm run test:e2e -- --spec e2e/specs/story-16-4-convention-rescan-after-flush.e2e.mjs
 */
import { waitForGridRows, waitForGridText } from '../support/gridWait.mjs'
import { markFlushBaseline, waitForFlushAfter } from '../support/flushWait.mjs'
import { resetPanelState } from '../support/panelReset.mjs'
import { realClick } from '../support/pointer.mjs'
import { openWorkspaceWithWork } from '../support/workspace.mjs'

const NAME = `Story 16-4 -- rescan ${Date.now()}`
const FIRST = 'Đảm bảo một.'
const SECOND = 'Đảm bảo hai.'
const EDITED = 'Cần đảm bảo nữa.'
const CONVENTION_STATE = '/src/conventionState.ts'

async function invoke(command, args) {
  return browser.execute(
    async (c, a) => window.__TAURI_INTERNALS__.invoke(c, a),
    command,
    args,
  )
}

function cell(id) {
  return `[data-col="tgt"][data-segment-id="${id}"]`
}

async function markCount(id) {
  return browser.execute((sel) => document.querySelectorAll(sel).length, `${cell(id)} .convention-mark`)
}

async function waitForMarks(id, expected, what) {
  await browser.waitUntil(async () => (await markCount(id)) === expected, {
    timeout: 15_000,
    timeoutMsg: what,
  })
}

// `__TAURI_INTERNALS__.invoke` is non-writable, so the spec counts at the transport instead: every
// command leaves the webview as a `fetch` to `ipc://localhost/<command>` (`http://ipc.localhost/` on
// WebView2). The product carries no counting code.
async function startInvokeLog() {
  await browser.execute(() => {
    window.__rescanLog = []
    if (window.__rescanWrapped === true) return
    const original = window.fetch
    const command = /^(?:ipc:\/\/localhost|https?:\/\/ipc\.localhost)\/([^/?#]+)/
    window.fetch = function (input, init) {
      const url = String(input !== null && typeof input === 'object' && 'url' in input ? input.url : input)
      const hit = command.exec(url)
      if (hit !== null) {
        let body = null
        const raw = init?.body
        try {
          if (typeof raw === 'string') body = JSON.parse(raw)
          else if (raw instanceof ArrayBuffer || ArrayBuffer.isView(raw)) body = JSON.parse(new TextDecoder().decode(raw))
        } catch {
          body = null
        }
        window.__rescanLog.push({ command: hit[1], args: body })
      }
      return original.apply(this, arguments)
    }
    window.__rescanWrapped = true
  })
  // Self-check: the product's own adapter must show up in the log, or every "zero calls" below is vacuous.
  const seen = await browser.execute(async () => {
    window.__rescanLog = []
    const mod = await import('/src/config/convention.ts')
    await mod.scanConventionSegments(0, [])
    return window.__rescanLog.map((entry) => entry.command)
  })
  if (!seen.includes('convention_scan_segments')) {
    throw new Error(
      `The IPC log cannot see the product's own calls (saw: ${JSON.stringify(seen)}); this is a bench failure, not a product one.`,
    )
  }
}

async function readInvokeLog() {
  return browser.execute(() =>
    window.__rescanLog
      .filter((entry) => !entry.command.startsWith('plugin%3Awdio'))
      .map((entry) => ({ command: entry.command, args: entry.args })),
  )
}

async function typeInto(id, text) {
  await realClick(await $(cell(id)))
  await browser.pause(300)
  await browser.execute((t) => {
    window.__rescanLog = []
    document.execCommand('selectAll')
    if (t === '') document.execCommand('delete')
    else document.execCommand('insertText', false, t)
  }, text)
}

describe('Story 16-4 -- rescan after flush', () => {
  let firstId
  let secondId

  it('two AI-translated sentences carry a mark under a Global rule', async () => {
    await openWorkspaceWithWork(NAME, '一。二。')
    await waitForGridRows(2, { col: 'tgt', what: 'rescan grid' })
    const read = await invoke('read_open_chapter_segments', {})
    firstId = read.segments[0].id
    secondId = read.segments[1].id
    await invoke('promote_ai_translation', { segmentId: firstId, targetText: FIRST, force: false })
    await invoke('promote_ai_translation', { segmentId: secondId, targetText: SECOND, force: false })
    await invoke('convention_add_rule', { tier: 'global', avoidPhrase: 'đảm bảo', replacementPhrase: 'bảo đảm' })
    await resetPanelState()
    await waitForGridText(0, FIRST, { col: 'tgt' })
    await waitForGridText(1, SECOND, { col: 'tgt' })
    await browser.execute(async (path) => {
      const mod = await import(/* @vite-ignore */ path)
      await mod.scanConventionNow()
    }, CONVENTION_STATE)
    await waitForMarks(firstId, 1, 'the first sentence never got its mark')
    await waitForMarks(secondId, 1, 'the second sentence never got its mark')
  })

  it('typing kills the mark of that sentence with zero IPC calls, then the flush brings it back for that sentence only', async () => {
    await startInvokeLog()
    const baseline = await markFlushBaseline()
    await typeInto(firstId, EDITED)
    await waitForMarks(firstId, 0, 'the typed sentence kept its mark')
    await browser.pause(300)
    const whileTyping = await readInvokeLog()
    await expect(whileTyping.map((entry) => entry.command)).toEqual([])
    await expect(await markCount(secondId)).toBe(1)

    await waitForFlushAfter(baseline, { what: 'the flush after typing' })
    await waitForMarks(firstId, 1, 'the mark never came back after the flush')
    await expect(await $(`${cell(firstId)} .convention-mark`).getText()).toBe('đảm bảo')

    const log = await readInvokeLog()
    const names = log.map((entry) => entry.command)
    await expect(names.filter((name) => name.startsWith('convention_'))).toEqual(['convention_scan_segments'])
    await expect(names.filter((name) => name === 'save_segment_targets').length).toBe(1)
    await expect(names.indexOf('save_segment_targets')).toBeLessThan(names.indexOf('convention_scan_segments'))
    const rescans = log.filter((entry) => entry.command === 'convention_scan_segments')
    await expect(rescans[0].args.segmentIds).toEqual([firstId])
    await expect(await markCount(secondId)).toBe(1)
  })

  it('a sentence emptied by the user has no mark after the flush and the other one keeps its own', async () => {
    const baseline = await markFlushBaseline()
    await typeInto(firstId, '')
    await waitForFlushAfter(baseline, { what: 'the flush after emptying' })
    await browser.pause(500)
    await expect(await markCount(firstId)).toBe(0)
    await expect(await markCount(secondId)).toBe(1)
    const stored = (await invoke('read_open_chapter_segments', {})).segments
    await expect(stored.find((s) => s.id === firstId).target_text).toBe('')
    await expect(stored.find((s) => s.id === secondId).target_text).toBe(SECOND)
  })
})
