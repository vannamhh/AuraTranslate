/**
 * Bàn đo Story 2.14 — chi phí dời con trỏ và một phím gõ ở Chương 9.850 câu, WKWebView thật.
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

describe('Bàn đo 2.14 — dời con trỏ ở 9.850 câu', () => {
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

  it('bấm chuột thật sang ô khác', async function () {
    this.timeout(600_000)
    const down = []
    const up = []
    for (let i = 0; i < SO_LUOT + 1; i += 1) {
      const o = await $$('[data-col="tgt"]')
      await armMouse()
      await realClick(o[3 + (i % 2) * 5])
      await browser.pause(1500)
      const r = await disarmMouse()
      if (i > 0) {
        down.push(r.down)
        up.push(r.up)
      }
    }
    tomTat('bấm: mousedown → 2 frame (ms)', down)
    tomTat('bấm: mouseup → 2 frame (ms)', up)
  })

  it('Mod+Alt+ArrowDown', async function () {
    this.timeout(600_000)
    const lan = []
    for (let i = 0; i < SO_LUOT + 1; i += 1) {
      const ms = await browser.execute(async () => {
        const target = document.activeElement ?? document.body
        const t0 = performance.now()
        target.dispatchEvent(
          new KeyboardEvent('keydown', {
            code: 'ArrowDown',
            key: 'ArrowDown',
            metaKey: true,
            altKey: true,
            bubbles: true,
            cancelable: true,
          }),
        )
        await new Promise((r) => requestAnimationFrame(() => requestAnimationFrame(r)))
        return +(performance.now() - t0).toFixed(2)
      })
      if (i > 0) lan.push(ms)
      await browser.pause(300)
    }
    tomTat('Mod+Alt+ArrowDown → 2 frame (ms)', lan)
  })

  it('bấm qua lại giữa câu nháp và câu đã xác nhận (đổi kiểu nét, không đổi độ dày)', async function () {
    this.timeout(600_000)
    const o = await $$('[data-col="tgt"]')
    const ids = [await o[20].getAttribute('data-segment-id'), await o[25].getAttribute('data-segment-id')].map(Number)
    await browser.execute(async (ids) => {
      const state = await import(/* @vite-ignore */ '/src/panels/editorPanelState.ts')
      state.setEditorCaret(ids[0])
      state.noteEditorEdit(ids[0], 'Bản dịch nháp.')
      await state.flushEditorNow()
      state.setEditorCaret(ids[1])
      state.noteEditorEdit(ids[1], 'Bản dịch đã ký.')
      await state.flushEditorNow()
      await state.confirmCurrentSegment()
    }, ids)
    await browser.pause(1000)
    await langLai()
    const down = []
    const net = []
    for (let i = 0; i < SO_LUOT + 1; i += 1) {
      const cells = await $$('[data-col="tgt"]')
      await armMouse()
      await realClick(cells[i % 2 === 0 ? 20 : 25])
      await browser.pause(1500)
      const r = await disarmMouse()
      if (i > 0) down.push(r.down)
      net.push(
        await browser.execute(() =>
          [...document.querySelectorAll('.col-rule .rule')]
            .map((el) => {
              const s = getComputedStyle(el)
              return `${[...el.classList].find((c) => c.startsWith('rule-'))}:${s.borderLeftStyle} ${s.borderLeftWidth}`
            })
            .join(','),
        ),
      )
    }
    tomTat('bấm nháp ↔ đã xác nhận: mousedown → 2 frame (ms)', down)
    console.log('[2.14] nét sau mỗi lượt:', JSON.stringify(net))
  })

  it('một lượt gõ', async function () {
    this.timeout(600_000)
    const lan = []
    const o = await $$('[data-col="tgt"]')
    await realClick(o[2])
    for (let i = 0; i < SO_LUOT + 1; i += 1) {
      const ms = await browser.execute(async (n) => {
        const cell = document.activeElement
        if (cell === null || cell.getAttribute('data-col') !== 'tgt') throw new Error('khong o trong o ban dich')
        const t0 = performance.now()
        cell.textContent = 'x'.repeat(n + 1)
        cell.dispatchEvent(new InputEvent('input', { inputType: 'insertText', data: 'x', bubbles: true }))
        await new Promise((r) => requestAnimationFrame(() => requestAnimationFrame(r)))
        return +(performance.now() - t0).toFixed(2)
      }, i)
      if (i > 0) lan.push(ms)
      await browser.pause(300)
    }
    tomTat('một phím gõ → 2 frame (ms)', lan)
  })
})
