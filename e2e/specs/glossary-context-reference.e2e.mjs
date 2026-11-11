/**
 * The Glossary queue shows the Work's own translation of a candidate's Context sentence and
 * offers `Dịch thử` only when none exists; without AI configured the button is off and
 * Settings is one click away. No network call is made in either branch.
 *
 *     npm run test:e2e -- --spec e2e/specs/glossary-context-reference.e2e.mjs
 */
import { realClick } from '../support/pointer.mjs'

const TRANSLATED_TERM = 'AuraRef FireDragon'
const BARE_TERM = 'AuraRef IceWolf'
const TRANSLATION = 'Con rong lua xuat hien.'

async function invoke(command, args) {
  return browser.execute(
    async (c, a) => window.__TAURI_INTERNALS__.invoke(c, a),
    command,
    args,
  )
}

async function waitForCandidates() {
  await browser.waitUntil(
    async () => {
      const rows = await invoke('glossary_pending_candidates')
      return (
        rows.some((r) => r.source_term === TRANSLATED_TERM) &&
        rows.some((r) => r.source_term === BARE_TERM)
      )
    },
    { timeout: 30_000, interval: 250, timeoutMsg: 'the import scan never produced both candidates' },
  )
  return invoke('glossary_pending_candidates')
}

async function moveCursorBy(delta) {
  for (let i = 0; i < Math.abs(delta); i += 1) {
    await browser.keys([delta > 0 ? 'ArrowDown' : 'ArrowUp'])
    await browser.pause(50)
  }
}

describe('Glossary queue -- context sentence reference', () => {
  it('shows the Chapter translation, then the off button and the way to Settings', async () => {
    await browser.waitUntil(
      async () =>
        browser.execute(
          () => window.__TAURI_INTERNALS__ !== undefined && (document.querySelector('#app')?.children.length ?? 0) > 0,
        ),
      { timeout: 30_000, interval: 250 },
    )
    await invoke('put_config', { kind: 'app_config', key: 'glossary_scan_threshold', value: '5' })

    const sentences = (term) =>
      Array.from({ length: 5 }, (_, i) => `a beast called ${term} appeared at hour ${i}.`)
    await invoke('create_work_from_text', {
      name: 'Glossary context reference',
      sourceLang: 'en',
      genre: 'general',
      text: [...sentences(TRANSLATED_TERM), ...sentences(BARE_TERM)].join('\n\n'),
    })

    const candidates = await waitForCandidates()
    const translated = candidates.find((c) => c.source_term === TRANSLATED_TERM)
    const bare = candidates.find((c) => c.source_term === BARE_TERM)

    const loaded = await invoke('read_open_chapter_segments', {})
    const segment = loaded.segments.find((s) => s.source_text.startsWith(translated.context_example))
    await expect(segment).toBeDefined()
    await invoke('save_segment_targets', {
      chapterId: loaded.chapter_id,
      edits: [{ id: segment.id, target_text: TRANSLATION }],
    })

    await browser.keys(['Meta', 'Alt', 'q'])
    await $('.gq-scrim').waitForDisplayed({ timeout: 10_000 })

    await moveCursorBy(candidates.indexOf(translated))
    const text = await $('[data-gq-context-ref-text]')
    await text.waitForDisplayed({ timeout: 10_000 })
    await expect(await text.getText()).toBe(TRANSLATION)
    await expect(await $('[data-gq-context-translate]').isExisting()).toBe(false)

    await moveCursorBy(candidates.indexOf(bare) - candidates.indexOf(translated))
    const button = await $('[data-gq-context-translate]')
    await button.waitForExist({ timeout: 10_000 })
    await $('[data-gq-context-unconfigured]').waitForExist({ timeout: 10_000 })
    await expect(await button.getAttribute('disabled')).not.toBe(null)

    await realClick(await $('[data-gq-context-open-settings]'))
    await $('.set-panel').waitForDisplayed({ timeout: 10_000 })
  })
})
