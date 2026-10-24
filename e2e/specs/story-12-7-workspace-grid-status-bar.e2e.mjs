/**
 * 12.7 Verify, in a real WKWebView: the margin rules differ in stroke (computed style, which
 * happy-dom cannot give for a `<style scoped>` block), the grid has four columns with no visible
 * status text, and the status bar keeps its permanent info outside the `role="status"` region.
 */
import { realClick } from '../support/pointer.mjs'
import { openWorkspaceWithWork } from '../support/workspace.mjs'
import { waitForGridRows } from '../support/gridWait.mjs'

const STATE_MODULE = '/src/panels/editorPanelState.ts'
const I18N_MODULE = '/src/i18n/index.ts'

async function readSegmentsFromDisk() {
  return browser.execute(async () => window.__TAURI_INTERNALS__.invoke('read_open_chapter_segments', {}))
}

async function clickTargetCell(segmentId) {
  await realClick(await $(`[data-col="tgt"][data-segment-id="${segmentId}"]`))
  await browser.pause(200)
}

function readRuleStrokes() {
  const out = {}
  for (const rule of document.querySelectorAll('.col-rule .rule')) {
    const cls = [...rule.classList].find((c) => c.startsWith('rule-'))
    const style = getComputedStyle(rule)
    out[cls] = `${style.borderLeftStyle} ${style.borderLeftWidth}`
  }
  return { strokes: out, ruleCount: document.querySelectorAll('.col-rule .rule').length }
}

describe('12.7 — lưới hai cột và thanh trạng thái', () => {
  let segments

  before(async () => {
    const tag = `${Date.now() % 1_000_000}`
    await openWorkspaceWithWork(`e2e-12-7-${tag}`, 'Một。Hai。Ba。Bốn。')
    await waitForGridRows(4)
    segments = (await readSegmentsFromDisk()).segments

    await browser.execute(
      async (modulePath, ids) => {
        const state = await import(/* @vite-ignore */ modulePath)
        state.setEditorCaret(ids[0])
        state.noteEditorEdit(ids[0], 'Một bản dịch.')
        await state.flushEditorNow()
        await state.confirmCurrentSegment()
        state.setEditorCaret(ids[1])
        state.noteEditorEdit(ids[1], 'Bản dịch nháp.')
        await state.flushEditorNow()
        state.setEditorCaret(ids[2])
      },
      STATE_MODULE,
      segments.map((s) => s.id),
    )
    await browser.pause(500)
  })

  it('vạch lề: đã xác nhận · đang sửa · nháp khác nhau ở kiểu nét hoặc độ dày (computed style); chưa dịch không có vạch', async () => {
    const { strokes, ruleCount } = await browser.execute(readRuleStrokes)

    expect(Object.keys(strokes).sort()).toEqual(['rule-confirmed', 'rule-draft', 'rule-primary'])
    expect(ruleCount).toBe(3)
    expect(new Set(Object.values(strokes)).size).toBe(3)
    expect(strokes['rule-draft']).toMatch(/^dashed 3px$/)
    expect(strokes['rule-primary']).toMatch(/^solid 3px$/)
    expect(strokes['rule-confirmed']).toMatch(/^double 3px$/)
  })

  it('lưới còn bốn cột, không chữ trạng thái nhìn thấy, tên truy cập ô bản dịch mang số câu và trạng thái', async () => {
    const result = await browser.execute(async (i18nPath) => {
      const { t } = await import(/* @vite-ignore */ i18nPath)
      const grid = document.querySelector('.grid')
      const labels = [...document.querySelectorAll('.col-tgt .cell-tgt')].map((c) => c.getAttribute('aria-label'))
      const gridText = grid.innerText
      return {
        columns: getComputedStyle(grid.querySelector('.grid-block')).gridTemplateColumns.split(' ').length,
        labels,
        leaked: ['confirmed', 'draft', 'untranslated', 'editing', 'tm'].filter((k) => gridText.includes(t(`panel.grid.state_${k}`))),
        confirmed: t('panel.grid.state_confirmed'),
        untranslated: t('panel.grid.state_untranslated'),
      }
    }, I18N_MODULE)

    expect(result.columns).toBe(4)
    expect(result.leaked).toEqual([])
    expect(result.labels).toHaveLength(4)
    expect(result.labels[0]).toBe(`Câu 1, ${result.confirmed}`)
    expect(result.labels[3]).toBe(`Câu 4, ${result.untranslated}`)
  })

  it('thanh trạng thái: vị trí và bộ đếm nằm ngoài vùng role="status"; đổi câu không đổi vùng đó', async () => {
    await clickTargetCell(segments[2].id)
    const read = () =>
      browser.execute(() => {
        const text = (sel) => document.querySelector(sel)?.textContent?.trim() ?? null
        const live = document.querySelector('[data-status-live]')
        return {
          position: text('[data-status-position]'),
          confirmed: text('[data-status-confirmed]'),
          hint: text('[data-status-hint]'),
          live: live.textContent.replace(/\d+/g, 'N').trim(),
          roleHolders: document.querySelectorAll('footer.status [role="status"]').length + (document.querySelector('footer.status').hasAttribute('role') ? 1 : 0),
          insideLive: live.querySelector('[data-status-position], [data-status-confirmed], [data-status-hint]') !== null,
        }
      })

    const before = await read()
    expect(before.position).toMatch(/[Cc]âu 3\/4$/)
    expect(before.confirmed).toBe('1/4 đã xác nhận')
    expect(before.hint).toMatch(/gộp · .+ tách$/)
    expect(before.roleHolders).toBe(1)
    expect(before.insideLive).toBe(false)

    await clickTargetCell(segments[3].id)
    const after = await read()

    expect(after.position).toMatch(/[Cc]âu 4\/4$/)
    expect(after.live).toBe(before.live)
  })

  it('Glossary và TM hiện số khi Tra cứu chưa rút; khi đã rút thì thay bằng dòng tra cứu', async () => {
    const retreated = await browser.execute(() => document.querySelector('[data-lookup-drawer-open]') !== null)
    const counts = await browser.execute(() => ({
      glossary: document.querySelector('[data-status-glossary]')?.textContent?.trim() ?? null,
      tm: document.querySelector('[data-status-tm]')?.textContent?.trim() ?? null,
    }))

    if (retreated) {
      expect(counts).toEqual({ glossary: null, tm: null })
    } else {
      expect(counts.glossary).toMatch(/^Glossary \d+ mục$/)
      expect(counts.tm).toMatch(/^TM: \d+ cặp · \d+ của tôi · \d+ của người khác · ngưỡng gợi ý \d+%$/)
    }
  })
})
