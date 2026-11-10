/**
 * Bàn đo cho mục sổ nợ *"lớp phủ Quản lý quy ước vẽ MỌI hàng luật một lúc, không cắt khúc và
 * không cuộn ảo"* — 2.000 luật là chính con số AD-67 mục 11 lấy làm thang.
 *
 * ⚠️ Tệp này nằm NGOÀI `e2e/specs/`, nên khuôn gom spec của `wdio.conf.mjs` không thấy nó và
 * lượt nightly không trả giá một cửa sổ thật (~1,5 phút) cộng lượt gieo mỗi đêm. Chạy tay:
 *
 *     npm run test:e2e -- --spec e2e/measure/convention-manage-2000-rules.e2e.mjs
 *
 * 🔴 Nó IN số đo và chỉ khẳng định phần CẤU TRÚC. `e2e/AGENTS.md` cấm khẳng định một con số
 * tuyệt đối trên một tài nguyên dùng chung, và ngưỡng là quyết định của Ice sau khi có số.
 *
 * Lớp phủ được mở bằng chính `openConventionManage()` của sản phẩm, không bằng một bản chép —
 * cùng luật mà `e2e/AGENTS.md` rút ra cho mọi bàn đo.
 */
import { openWorkspaceWithWork } from '../support/workspace.mjs'

const RULES = 2000
const CHUNK = 200
const ROW = '[data-convention-rule-row]'
const STATE = '/src/conventionManageState.ts'

async function invoke(command, args) {
  return browser.execute(async (c, a) => window.__TAURI_INTERNALS__.invoke(c, a), command, args)
}

/** Gieo một khúc luật trong MỘT lời gọi script, để không lượt nào chạm trần script của driver. */
async function seedChunk(from, count) {
  return browser.execute(
    async (start, n) => {
      const t0 = performance.now()
      for (let i = start; i < start + n; i += 1) {
        await window.__TAURI_INTERNALS__.invoke('convention_add_rule', {
          tier: 'global',
          avoidPhrase: `ddd${i}`,
          replacementPhrase: `eee${i}`,
        })
      }
      return Math.round(performance.now() - t0)
    },
    from,
    count,
  )
}

describe('Quản lý quy ước ở 2.000 luật', () => {
  it('đo thời gian mở lớp phủ và độ trễ của phím lên/xuống', async () => {
    await openWorkspaceWithWork('do-2000-luat', 'Bảo đảm an toàn.')

    let seedMs = 0
    for (let from = 0; from < RULES; from += CHUNK) {
      seedMs += await seedChunk(from, Math.min(CHUNK, RULES - from))
    }

    // `convention_list_rules` trả THẲNG một mảng, không bọc trong `{ rules }`.
    const listed = await invoke('convention_list_rules', { tier: 'global' })
    expect(listed.length).toBeGreaterThanOrEqual(RULES)

    // Tách phần Rust ra khỏi phần vẽ: lượt mở lớp phủ gồm MỘT lời gọi `convention_list_rules`
    // rồi mới tới lượt dựng DOM. Binary ở đây là bản DEBUG (`cargo build --features wdio`), nên
    // nửa Rust của con số không phải nửa mà bản release sẽ cho.
    const listMs = await browser.execute(async () => {
      const t0 = performance.now()
      await window.__TAURI_INTERNALS__.invoke('convention_list_rules', { tier: 'global' })
      return Math.round(performance.now() - t0)
    })

    const open = await browser.execute(
      async (sel, statePath) => {
        const frame = () => new Promise((resolve) => requestAnimationFrame(resolve))
        const mod = await import(statePath)
        if (mod.openConventionManage === undefined) {
          return { fail: `"${statePath}" không xuất \`openConventionManage\`` }
        }
        const t0 = performance.now()
        await mod.openConventionManage()
        await frame()
        await frame()
        return {
          ms: Math.round(performance.now() - t0),
          rows: document.querySelectorAll(sel).length,
        }
      },
      ROW,
      STATE,
    )
    if (open.fail !== undefined) throw new Error(open.fail)

    const key = await browser.execute(async (sel) => {
      const frame = () => new Promise((resolve) => requestAnimationFrame(resolve))
      const selectedIndex = () =>
        [...document.querySelectorAll(sel)].findIndex(
          (row) => row.getAttribute('aria-selected') === 'true',
        )
      // 🔴 Phải phát từ một hàng, không từ `body`: `onKeydown` của lớp phủ bỏ qua sự kiện có
      // target là input, textarea, button hay summary, và handler gắn ở gốc lớp phủ nên sự
      // kiện phải NỔI từ trong ra.
      const row = document.querySelector(sel)
      if (row === null) return { fail: 'không có hàng luật nào để phát phím' }
      const before = selectedIndex()
      const t0 = performance.now()
      row.dispatchEvent(new KeyboardEvent('keydown', { key: 'ArrowDown', bubbles: true }))
      await frame()
      await frame()
      return { ms: Math.round(performance.now() - t0), before, after: selectedIndex() }
    }, ROW)
    if (key.fail !== undefined) throw new Error(key.fail)

    console.log(
      `[đo 2000 luật] gieo ${RULES} luật: ${seedMs} ms · chỉ IPC list: ${listMs} ms · ` +
        `mở lớp phủ: ${open.ms} ms · ` +
        `hàng đã vẽ: ${open.rows} · một phím xuống: ${key.ms} ms (con trỏ ${key.before} → ${key.after})`,
    )

    expect(open.rows).toBeGreaterThanOrEqual(RULES)
    expect(key.after).toBe(key.before + 1)
  })
})
