/**
 * The Proofreader's decision ratio (accepted · not an error · skipped) follows accept, skip and
 * revert, survives reopening the Work, and reads "no decision yet" rather than 0% on a fresh Work;
 * clearing every Reading Mode mark of a Work leaves the marks list empty.
 *
 *     npm run test:e2e -- --spec e2e/specs/story-12-15-proofread-ratio-reading-marks.e2e.mjs
 *
 * The findings are written through the real accept / skip / revert commands, not by a model scan
 * (no provider in the runner). Not yet run (the webview stands at `tauri://localhost`); the nightly
 * is its first measurement.
 */
const SCANNED = 'Mọi nguời đã đi'
const PHRASE = 'nguời'
const SUGGESTION = 'người'
const START = 4
const END = 9

async function invoke(command, args) {
  return browser.execute(
    async (c, a) => window.__TAURI_INTERNALS__.invoke(c, a),
    command,
    args,
  )
}

async function waitForApp() {
  await browser.waitUntil(
    async () =>
      browser.execute(
        () => window.__TAURI_INTERNALS__ !== undefined && (document.querySelector('#app')?.children.length ?? 0) > 0,
      ),
    { timeout: 30_000, interval: 250 },
  )
}

async function createWork(name) {
  const created = await invoke('create_work_from_text', {
    name,
    sourceLang: 'en',
    genre: 'general',
    text: SCANNED,
  })
  const loaded = await invoke('read_open_chapter_segments', { prefill: false })
  const segmentId = loaded.segments[0].id
  await invoke('promote_ai_translation', { segmentId, targetText: SCANNED, force: false })
  return { workId: created.meta.work_id, segmentId }
}

async function counts() {
  const c = await invoke('ai_proofread_decision_counts', {})
  return [c.accepted, c.not_error, c.skipped]
}

describe('Story 12-15 -- decision ratio and Reading Mode marks', () => {
  let workId
  let segmentId
  let decisionId

  it('a Work with no decision reads zero in every cell, which the strip shows as "chưa có quyết định"', async () => {
    await waitForApp()
    ;({ workId, segmentId } = await createWork(`Story 12-15 -- ratio ${Date.now()}`))
    await expect(await counts()).toEqual([0, 0, 0])
  })

  it('accept one, skip one: 1 - 0 - 1, and the same after reopening the Work', async () => {
    const accepted = await invoke('ai_proofread_accept', {
      segmentId,
      expectedText: SCANNED,
      start: START,
      end: END,
      expectedPhrase: PHRASE,
      replacement: SUGGESTION,
      kind: 'spelling',
    })
    decisionId = accepted.decision_id
    await invoke('ai_proofread_decision_skip', { kind: 'grammar' })
    await expect(await counts()).toEqual([1, 0, 1])

    await invoke('open_work', { workId })
    await expect(await counts()).toEqual([1, 0, 1])
  })

  it('reverting the accept: 0 - 0 - 1', async () => {
    await invoke('ai_proofread_revert', {
      segmentId,
      expectedText: SCANNED.replace(PHRASE, SUGGESTION),
      start: START,
      end: START + SUGGESTION.length,
      expectedPhrase: SUGGESTION,
      replacement: PHRASE,
      kind: 'spelling',
      decisionId,
    })
    await expect(await counts()).toEqual([0, 0, 1])
  })

  it('clearing every mark of the Work leaves the marks list empty', async () => {
    await invoke('mark_reading_segment', { segmentId })
    await expect((await invoke('list_reading_marks', {})).length).toBe(1)
    await invoke('clear_reading_marks', {})
    await expect((await invoke('list_reading_marks', {})).length).toBe(0)
  })
})
