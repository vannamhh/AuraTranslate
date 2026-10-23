/**
 * Bàn đo Story 2.14 — chi phí dời con trỏ khi vạch lề đổi bằng CSS khác nhau (tiêm `<style>`), 9.850 câu.
 * Mỗi phép đo từ lúc sự kiện vào tới hết hai frame (`rAF` lồng nhau).
 *
 * Giới hạn: bản dựng debug nạp `devUrl` từ Vite dev (không phải bản đóng gói); phím `Mod+Alt+ArrowDown`
 * và `InputEvent` là sự kiện tổng hợp vì `browser.keys` không đi đường nhập gốc.
 */
import { openWorkspaceWithWork } from '../../../e2e/support/workspace.mjs'
import { realClick } from '../../../e2e/support/pointer.mjs'

const SO_CAU = 9850
const SO_LUOT = 4

function vanBan(n) {
  const parts = []
  for (let i = 0; i < n; i += 1) parts.push(`第${i + 1}句话，用来量渲染成本。`)
  return parts.join('')
}

async function langLai() {
  for (let i = 0; i < 40; i += 1) {
    const nhip = await browser.execute(
      async () =>
        await new Promise((r) => {
          const a = performance.now()
          requestAnimationFrame(() => r(performance.now() - a))
        }),
    )
    if (nhip < 50) return
  }
}

async function armMouse() {
  await browser.execute(() => {
    window.__bench = { down: null, up: null }
    const hai = (key, t0) =>
      requestAnimationFrame(() =>
        requestAnimationFrame(() => {
          window.__bench[key] = +(performance.now() - t0).toFixed(2)
        }),
      )
    window.__benchDown = () => hai('down', performance.now())
    window.__benchUp = () => hai('up', performance.now())
    document.addEventListener('mousedown', window.__benchDown, true)
    document.addEventListener('mouseup', window.__benchUp, true)
  })
}

async function disarmMouse() {
  return browser.execute(() => {
    document.removeEventListener('mousedown', window.__benchDown, true)
    document.removeEventListener('mouseup', window.__benchUp, true)
    return window.__bench
  })
}

function tomTat(ten, lan) {
  const sorted = [...lan].sort((a, b) => a - b)
  console.log(`[2.14] ${ten}:`, JSON.stringify({ lan, trung_vi: sorted[Math.floor(sorted.length / 2)], max: sorted.at(-1) }))
}

describe('Bàn đo 2.14 — biến thể CSS của vạch lề', () => {
  before(async function () {
    this.timeout(600_000)
    await openWorkspaceWithWork('2.14 — do hieu nang')
    await browser.setTimeout({ script: 300_000 })
    await browser.execute(async (text) => {
      await window.__TAURI_INTERNALS__.invoke('create_work_from_text', {
        name: '2.14-hieu-nang-' + Date.now(),
        sourceLang: 'zh',
        genre: 'general',
        text,
      })
    }, vanBan(SO_CAU))
    await browser.execute(() => {
      window.location.reload()
    })
    await $('[data-col="tgt"]').waitForExist({ timeout: 120_000 })
    await langLai()
    const info = await browser.execute(() => ({
      href: location.href,
      so_cau: document.querySelectorAll('[data-col="tgt"]').length,
      ua: navigator.userAgent,
    }))
    console.log('[2.14] quần thể:', JSON.stringify(info))
    if (!info.href.includes('localhost:1420') || info.so_cau < SO_CAU) throw new Error('DANH TINH PHIEN KHONG KHOP')
  })

const C_TUYET_DOI =
  '.margin-slot{position:relative}' +
  '.margin-slot::before{content:"";position:absolute;top:0;bottom:0;left:0;border-left:0 solid transparent;border-radius:var(--radius-sm)}' +
  '.rule.rule-primary,.rule.rule-confirmed,.rule.rule-tm-rule,.rule.rule-draft{border-left:0!important}' +
  '.rule.rule-confirmed::before{border-left:2px solid var(--color-confirmed)}' +
  '.rule.rule-primary::before{border-left:3px solid var(--color-primary)}' +
  '.rule.rule-tm-rule::before{border-left:2px dotted var(--color-tm-rule)}' +
  '.rule.rule-draft::before{border-left:2px dashed var(--color-draft)}'

const BIEN_THE = [
  ['doi-chung', ''],
  ['c-tuyet-doi', C_TUYET_DOI],
  ['vien-deu-3px', '.margin-slot{border-left:3px solid transparent!important}.rule.rule-primary,.rule.rule-confirmed,.rule.rule-tm-rule,.rule.rule-draft{border-left-width:3px!important;border-left-style:solid!important}'],
  ['c-tuyet-doi-lai', C_TUYET_DOI],
  ['doi-chung-lai', ''],
]

  for (const [ten, css] of BIEN_THE) {
    it(`bấm chuột thật — ${ten}`, async function () {
      this.timeout(600_000)
      await browser.execute((c) => {
        document.getElementById('__bt214')?.remove()
        const s = document.createElement('style')
        s.id = '__bt214'
        s.textContent = c
        document.head.appendChild(s)
      }, css)
      await langLai()
      const down = []
      for (let i = 0; i < SO_LUOT + 2; i += 1) {
        const o = await $$('[data-col="tgt"]')
        await armMouse()
        await realClick(o[3 + (i % 2) * 5])
        await browser.pause(1500)
        const r = await disarmMouse()
        if (i > 0) down.push(r.down)
      }
      tomTat(`${ten}: mousedown → 2 frame (ms)`, down)
      const ve = await browser.execute(() => {
        const el = document.querySelector('.rule.rule-primary')
        if (el === null) return null
        const truoc = getComputedStyle(el, '::before')
        const slot = getComputedStyle(el)
        return {
          so_primary: document.querySelectorAll('.rule.rule-primary').length,
          slot: `${slot.borderLeftWidth} ${slot.borderLeftStyle}`,
          before: `${truoc.borderLeftWidth} ${truoc.borderLeftStyle} ${truoc.position} h=${truoc.height}`,
          slot_h: el.getBoundingClientRect().height.toFixed(1),
        }
      })
      console.log(`[2.14] ${ten}: vạch vẽ ra:`, JSON.stringify(ve))
    })
  }
})
