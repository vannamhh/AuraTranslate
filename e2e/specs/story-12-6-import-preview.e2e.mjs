/**
 * Import preview screens in the real webview: two-column layout, no scrim, title-bar source
 * cell, global shortcuts blocked, focus returned to the opener, the "Nhập từ website" screen
 * and the bilingual "import now, link later" path. One spec file = one app process.
 */
import { mkdtempSync, writeFileSync } from 'node:fs'
import { tmpdir } from 'node:os'
import { join } from 'node:path'
import { realClick } from '../support/pointer.mjs'

const RUN_TAG = `${Date.now() % 1_000_000}`
const PASTE_WORK = `e2e-126-paste-${RUN_TAG}`
const BILINGUAL_WORK = `e2e-126-bilingual-${RUN_TAG}`

const OPEN_BTN = '[data-library-import-open]'
const FORM = '.li-panel .import-form'
const SCREEN = '.ip-screen'
const BIP_SCREEN = '.bip-screen'
const TITLEBAR_SOURCE = '[data-titlebar-import-source]'

async function ready() {
  await browser.waitUntil(async () => browser.execute(() => window.__TAURI_INTERNALS__ !== undefined), {
    timeout: 30_000,
    interval: 250,
  })
  await $(OPEN_BTN).waitForDisplayed({ timeout: 30_000 })
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

async function activeDescription() {
  return browser.execute(() => {
    const el = document.activeElement
    if (el === null) return 'null'
    return `${el.tagName.toLowerCase()}.${el.className} ${Array.from(el.attributes).map((a) => a.name).join(',')}`
  })
}

async function expectFocusOnAttribute(attribute) {
  try {
    await browser.waitUntil(
      async () => browser.execute((name) => document.activeElement?.hasAttribute(name) ?? false, attribute),
      { timeout: 10_000 },
    )
  } catch {
    throw new Error(`focus did not land on [${attribute}]; active element is ${await activeDescription()}`)
  }
}

async function fieldByLabel(form, index) {
  return (await form.$$('.field input[type="text"]'))[index]
}

describe('Story 12-6: import preview screens', () => {
  it('pasted text: two-column screen without a scrim, title-bar source cell, estimate settles, shortcuts blocked, Cancel returns focus', async () => {
    await ready()
    await realClick(await $(OPEN_BTN))
    const form = await $(FORM)
    await form.waitForDisplayed({ timeout: 10_000 })
    await (await fieldByLabel(form, 0)).setValue(PASTE_WORK)
    await (await form.$('textarea')).setValue('Cau mot。Cau hai。')
    const submit = (await form.$$('[data-import-preview-open]'))[0]
    await realClick(submit)

    await $(SCREEN).waitForDisplayed({ timeout: 30_000 })
    const shape = await browser.execute((screenSel) => {
      const screen = document.querySelector(screenSel)
      const two = screen.querySelector('.ip-two')
      const columns = getComputedStyle(two).gridTemplateColumns.split(' ').filter((c) => c !== '').length
      return {
        scrim: document.querySelector('.ip-scrim, .li-scrim') !== null && getComputedStyle(document.querySelector('.ip-scrim, .li-scrim')).display !== 'none',
        modal: screen.querySelector('[aria-modal], [role="dialog"]') !== null,
        columns,
        tabs: document.querySelectorAll('.mode-tab').length,
        source: (document.querySelector('[data-titlebar-import-source]')?.textContent || '').trim(),
      }
    }, SCREEN)
    expect(shape.scrim).toBe(false)
    expect(shape.modal).toBe(false)
    expect(shape.columns).toBeGreaterThanOrEqual(2)
    expect(shape.tabs).toBe(3)
    expect(shape.source).toContain('Văn bản đã dán')

    await browser.waitUntil(
      async () =>
        browser.execute(() => {
          const text = (document.querySelector('[data-ip-glossary-estimate]')?.textContent || '').trim()
          return text !== '' && !text.includes('đang ước lượng')
        }),
      { timeout: 60_000, timeoutMsg: 'Glossary estimate never settled' },
    )

    const before = await browser.execute(() => document.querySelector('.titlebar') !== null && document.querySelector('.ip-screen') !== null)
    expect(before).toBe(true)
    await browser.keys(['Control', 'Shift', 'p'])
    await browser.keys(['Alt', '2'])
    await browser.pause(300)
    expect(await $(SCREEN).isDisplayed()).toBe(true)

    const cancel = await $(`${SCREEN} .ip-actions .ip-act:not(.ip-act-primary)`)
    await realClick(cancel)
    await $(SCREEN).waitForExist({ reverse: true, timeout: 10_000 })
    await expectFocusOnAttribute('data-import-preview-open')
  })

  it('"Nhập từ website": count line, promise, submit label and Cancel returning focus to its opener', async () => {
    await browser.execute(() => document.querySelector('.li-panel textarea')?.focus())
    await realClick(await $('[data-library-url-screen-open]'))
    const screen = await $('[data-library-url-import-screen]')
    await screen.waitForDisplayed({ timeout: 10_000 })
    await (await screen.$('textarea')).setValue('https://a.example/1\nhttps://a.example/2\nhttps://a.example/3')

    const text = await browser.execute(() => ({
      count: (document.querySelector('.us-count')?.textContent || '').trim(),
      promise: (document.querySelector('.us-promise')?.textContent || '').trim(),
      submit: (document.querySelector('[data-library-url-import-screen] [data-import-preview-open]')?.textContent || '').trim(),
      source: (document.querySelector('[data-titlebar-import-source]')?.textContent || '').trim(),
    }))
    expect(text.count).toBe('3 link · sẽ tạo 3 Chương · 1 domain')
    expect(text.promise).toContain('Chỉ tải đúng 3 link')
    expect(text.submit).toBe('Tải 3 link')
    expect(text.source).toContain('3 link')

    await realClick(await screen.$('.us-actions .us-btn:not(.us-btn-primary)'))
    await screen.waitForExist({ reverse: true, timeout: 10_000 })
    await expectFocusOnAttribute('data-library-url-screen-open')
  })

  it('bilingual file with a mismatched row: "Nhập luôn, nối sau" creates the Work with the row marked unlinked', async () => {
    const dir = mkdtempSync(join(tmpdir(), 'e2e-126-'))
    const csv = join(dir, 'lech.csv')
    writeFileSync(csv, '甲。乙。,Mot hai\n丙。,Ba\n', 'utf8')

    await realClick(await $('[data-library-import-open]'))
    const form = await $(FORM)
    await form.waitForDisplayed({ timeout: 10_000 })
    await (await fieldByLabel(form, 0)).setValue(BILINGUAL_WORK)
    await browser.execute(() => {
      for (const select of document.querySelectorAll('.li-panel .import-form select')) {
        if (Array.from(select.options).some((o) => o.value === 'zh')) {
          select.value = 'zh'
          select.dispatchEvent(new Event('change', { bubbles: true }))
        }
      }
    })
    const inputs = await form.$$('.field input[type="text"]')
    await inputs[inputs.length - 1].setValue(csv)
    await realClick(await form.$('[data-bilingual-import-preview-open]'))

    await $(BIP_SCREEN).waitForDisplayed({ timeout: 30_000 })
    const probe = await browser.execute(() => {
      const unlinked = document.querySelector('[data-bilingual-import-unlinked]')
      return {
        scrim: document.querySelector('.bip-scrim') !== null,
        unlinkedPresent: unlinked !== null,
        unlinkedDisabled: unlinked === null ? null : unlinked.disabled,
        primaryDisabled: document.querySelector('.bip-act-primary')?.disabled ?? null,
        estimate: document.querySelector('[data-ip-glossary-estimate]') !== null,
      }
    })
    expect(probe.scrim).toBe(false)
    expect(probe.estimate).toBe(false)
    expect(probe.unlinkedPresent).toBe(true)
    expect(probe.unlinkedDisabled).toBe(false)
    expect(probe.primaryDisabled).toBe(true)

    await realClick(await $('[data-bilingual-import-unlinked]'))
    await $(BIP_SCREEN).waitForExist({ reverse: true, timeout: 30_000 })

    const chapters = await ipc('list_chapters')
    expect(chapters.reduce((sum, c) => sum + c.unlinked_count, 0)).toBe(1)
    await ipc('open_chapter', { chapterId: chapters[0].chapter_id })
    const loaded = await ipc('read_open_chapter_segments')
    expect(loaded.segments.filter((s) => s.unlinked)).toHaveLength(1)
    expect(loaded.segments.filter((s) => s.unlinked)[0].status).toBe('draft')
  })

  it('the Glossary estimate runs over the real wire and leaves the pending queue untouched across a pattern change and cancel', async () => {
    const created = await ipc('create_work_from_text', {
      name: `e2e-126-estimate-${RUN_TAG}`,
      sourceLang: 'zh',
      genre: 'general',
      text: '很短的一句话。',
    })
    const workId = created.meta.work_id

    const pendingCount = async () => (await ipc('glossary_pending_candidates')).length
    let previous = -1
    let settled = 0
    await browser.waitUntil(
      async () => {
        const now = await pendingCount()
        settled = now === previous ? settled + 1 : 0
        previous = now
        return settled >= 2
      },
      { timeout: 30_000, interval: 1_000, timeoutMsg: 'pending queue never settled after creating the Work' },
    )
    const before = previous

    const line = '蓝澜渡看着蓝澜渡的剑，蓝澜渡笑了。'
    const body = (heading) => `${heading}\n${line.repeat(8)}`
    const text = `${body('第1章')}\n${body('第2章')}`
    await ipc('preview_import_encoding_from_text', { text, sourceLang: 'zh', chapterPattern: null, destination: workId })

    const first = await ipc('estimate_import_glossary', { sourceLang: 'zh', encoding: 'UTF-8', chapterPattern: null })
    expect(first.outcome).toBe('estimated')
    expect(first.count).toBeGreaterThan(0)

    await ipc('preview_import_encoding_from_text', {
      text,
      sourceLang: 'zh',
      chapterPattern: { pattern: '第', kind: 'literal' },
      destination: workId,
    })
    const second = await ipc('estimate_import_glossary', {
      sourceLang: 'zh',
      encoding: 'UTF-8',
      chapterPattern: { pattern: '第', kind: 'literal' },
    })
    expect(second.outcome).toBe('estimated')
    expect(second.count).toBeGreaterThan(0)

    await ipc('cancel_import_glossary_estimate')
    expect(await pendingCount()).toBe(before)
  })

  it('the estimate subtracts the destination Glossary (open Work) and the pending import-scan terms (Work not open) exactly', async () => {
    const line = '赤霄岭望着赤霄岭的云，赤霄岭静了。'
    const text = `第1章\n${line.repeat(8)}`
    const scanned = await ipc('create_work_from_text', {
      name: `e2e-126-excl-a-${RUN_TAG}`,
      sourceLang: 'zh',
      genre: 'general',
      text,
    })
    const scannedId = scanned.meta.work_id
    let previous = -1
    let settled = 0
    await browser.waitUntil(
      async () => {
        const now = (await ipc('glossary_pending_candidates')).length
        settled = now === previous ? settled + 1 : 0
        previous = now
        return settled >= 2
      },
      { timeout: 30_000, interval: 1_000, timeoutMsg: 'pending queue never settled' },
    )
    const pending = await ipc('glossary_pending_candidates')
    expect(pending.length).toBeGreaterThan(0)

    const other = await ipc('create_work_from_text', {
      name: `e2e-126-excl-b-${RUN_TAG}`,
      sourceLang: 'zh',
      genre: 'general',
      text: '另一个很短的故事。',
    })
    const otherId = other.meta.work_id

    const estimateInto = async (destination) => {
      await ipc('preview_import_encoding_from_text', { text, sourceLang: 'zh', chapterPattern: null, destination })
      const out = await ipc('estimate_import_glossary', { sourceLang: 'zh', encoding: 'UTF-8', chapterPattern: null })
      expect(out.outcome).toBe('estimated')
      return out.count
    }

    const intoNew = await estimateInto(null)
    expect(intoNew).toBeGreaterThan(0)
    expect(intoNew).toBe(pending.length)

    await ipc('glossary_add_term', {
      tier: 'work',
      sourceTerm: pending[0].source_term,
      translation: 'x',
      note: '',
      category: 'other',
    })
    expect(await estimateInto(otherId)).toBe(intoNew - 1)
    expect(await estimateInto(scannedId)).toBe(0)
  })
})
