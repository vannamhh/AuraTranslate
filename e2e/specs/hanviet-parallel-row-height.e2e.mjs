/**
 * Bugfix — Hán Việt, kiểu xem song song: hàng lưới không được cắt/đè lên hàng kế.
 *
 * `.hv-surface` từng tự khai `overflow: auto` (`SourceHanViet.vue`), việc này khiến track
 * hàng của `subgrid` (`GridPanel.vue`) đo chiều cao ô nguồn văn sai khi cột hẹp buộc văn bản
 * xuống thêm một dòng sau khi track đã được tính — dòng cuối cùng (`rt`, `ruby-position:
 * under`) tràn ra ngoài ô và đè lên hàng kế. Spec đầy đủ:
 * `_bmad-output/implementation-artifacts/spec-fix-hanviet-parallel-row-overflow.md`.
 *
 * ⚠️ Bề rộng cửa sổ thật không cố định giữa các lượt chạy máy Ice, nên phép ép cột hẹp
 * (`NARROW_COLUMNS`) là CSS chỉ dành cho bàn đo — không có nó, ca này có thể xanh giả vì
 * cột tình cờ đủ rộng để không cần xuống dòng thêm.
 */
import { waitForGridRows } from '../support/gridWait.mjs'
import { openWorkspaceWithWork } from '../support/workspace.mjs'
import { realClick } from '../support/pointer.mjs'

const TEXT =
  '【大纪元2022年05月03日讯】布拉姆斯的小提琴协奏曲是小提琴最受欢迎的演奏曲目之一。\n' +
  '这首协奏曲丰富的旋律与壮丽的气势，展现了作曲家深厚的功力，也考验着演奏者的技巧与音乐性。'

const NARROW_COLUMNS =
  '.grid { grid-template-columns: 3px 30px minmax(0,110px) minmax(0,110px) 96px !important; }'

// Sai số làm tròn subpixel giữa hai `getBoundingClientRect()` độc lập — không phải biên độ
// cho phép tràn thật.
const TOLERANCE_PX = 1

async function injectProbeCss(css) {
  await browser.execute((c) => {
    let style = document.getElementById('__hv-row-height-probe')
    if (!style) {
      style = document.createElement('style')
      style.id = '__hv-row-height-probe'
      document.head.appendChild(style)
    }
    style.textContent = c
  }, css)
  await browser.pause(300)
}

async function toggleHanVietView() {
  await realClick(await $('.view-toggle'))
  await browser.pause(400)
}

function measureSourceCells() {
  const cells = [...document.querySelectorAll('[data-col="src"]')]
  return cells.map((cell) => {
    const rect = cell.getBoundingClientRect()
    let contentBottom = rect.top
    for (const el of cell.querySelectorAll('*')) {
      for (const r of el.getClientRects()) {
        if (r.bottom > contentBottom) contentBottom = r.bottom
      }
    }
    return { top: rect.top, bottom: rect.bottom, contentBottom }
  })
}

function overflowYOfCellDescendants() {
  const cells = [...document.querySelectorAll('[data-col="src"], [data-col="tgt"]')]
  const out = []
  for (const cell of cells) {
    for (const el of cell.querySelectorAll('*')) {
      out.push(getComputedStyle(el).overflowY)
    }
  }
  return out
}

async function assertRowsFitAndDoNotOverlap() {
  const rows = await browser.execute(measureSourceCells)
  await expect(rows.length).toBeGreaterThan(0)
  for (const row of rows) {
    await expect(row.contentBottom).toBeLessThanOrEqual(row.bottom + TOLERANCE_PX)
  }
  for (let i = 0; i < rows.length - 1; i += 1) {
    await expect(rows[i].bottom).toBeLessThanOrEqual(rows[i + 1].top + TOLERANCE_PX)
  }
}

async function openParallelHanViet(name) {
  await openWorkspaceWithWork(name, TEXT)
  await waitForGridRows(2)
  await realClick(await $('#grid-tab-han-viet'))
  await browser.pause(500)
  await injectProbeCss(NARROW_COLUMNS)
  await toggleHanVietView() // switch → parallel
}

describe('Bugfix — Hán Việt song song không tràn/đè hàng lưới', () => {
  it('vào song song lần đầu: mỗi ô nguồn cao đủ chứa nội dung, không hàng nào đè hàng kế', async () => {
    await openParallelHanViet('HV row-height — lần đầu')
    await assertRowsFitAndDoNotOverlap()
  })

  it('bật/tắt song song ba lần: mệnh đề trên giữ đúng ở mỗi lượt', async () => {
    await openParallelHanViet('HV row-height — bật tắt ba lần')
    for (let cycle = 0; cycle < 3; cycle += 1) {
      await assertRowsFitAndDoNotOverlap()
      await toggleHanVietView() // song song → chuyển đổi
      await toggleHanVietView() // chuyển đổi → song song
    }
    await assertRowsFitAndDoNotOverlap()
  })

  it('thanh cuộn luôn hiện (giả lập máy có overlay scrollbar): mệnh đề trên vẫn giữ', async () => {
    await openWorkspaceWithWork('HV row-height — scrollbar luôn hiện', TEXT)
    await waitForGridRows(2)
    await realClick(await $('#grid-tab-han-viet'))
    await browser.pause(500)
    // 🔴 Khai cả hai luật TRƯỚC lượt chuyển vào song song — chính lượt CHUYỂN mới là nơi
    // track hàng của subgrid được đo (xem đầu tệp); khai luật scrollbar SAU lượt chuyển
    // chỉ kích một lượt tính lại không đại diện cho ca thật.
    await injectProbeCss(`${NARROW_COLUMNS} .grid-scroll { overflow-y: scroll !important; }`)
    await toggleHanVietView() // chuyển đổi → song song
    await assertRowsFitAndDoNotOverlap()
  })

  it('không phần tử nào trong một ô [data-col] tự cuộn, ở cả hai kiểu xem', async () => {
    await openWorkspaceWithWork('HV row-height — không hộp cuộn riêng', TEXT)
    await waitForGridRows(2)
    await realClick(await $('#grid-tab-han-viet'))
    await browser.pause(500)

    const switchModeOverflow = await browser.execute(overflowYOfCellDescendants)
    await expect(switchModeOverflow.length).toBeGreaterThan(0)
    await expect(switchModeOverflow.every((v) => v === 'visible')).toBe(true)

    await toggleHanVietView() // chuyển đổi → song song
    const parallelModeOverflow = await browser.execute(overflowYOfCellDescendants)
    await expect(parallelModeOverflow.length).toBeGreaterThan(0)
    await expect(parallelModeOverflow.every((v) => v === 'visible')).toBe(true)
  })
})
