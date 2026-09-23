/**
 * `WorkspaceDock.vue` **mounted thật** — Story 4.12, Phase 4b. Sáu mệnh đề của phase file
 * (A–F), tất cả đòi một dock THẬT dựng bằng `dockview`, thứ mà `layoutTierFor.test.ts`
 * (Phase 4a, hàm thuần) không chạm tới.
 *
 * ─────────────────────────────────────────────────────────────────────────────
 * ⚠️ SỐ VÀO / TẦNG RA, KHÔNG PHẢI HÌNH HỌC — `happy-dom` không tính layout thật
 * ─────────────────────────────────────────────────────────────────────────────
 * `setWindowSize`/`setChromeTokens` bên dưới đặt thẳng `window.innerWidth`/`innerHeight`
 * và hai token chrome (qua `document.documentElement.style`, đúng nguồn mà
 * `WorkspaceDock.vue::readChromeToken` đọc lúc chạy) rồi phát `resize` — đi qua đúng
 * listener sản phẩm (`onWindowResize` → `measureAndApplyTier`), không gọi tắt một hàm nội
 * bộ nào. Đây là khẳng định "số vào ⇒ tầng ra" qua đường THẬT; khẳng định về PIXEL trên
 * màn hình thuộc bàn đo/e2e (spec §Never: "Do not assert geometry in vitest").
 *
 * ─────────────────────────────────────────────────────────────────────────────
 * BƯỚC 0 — KHẢ THI: đã đo, xem `4-12-phases-2026-09-22.md` §Phase 4b
 * ─────────────────────────────────────────────────────────────────────────────
 * `dockview` mount được dưới `happy-dom` không cần vá `setup.ts` nào mới (`ResizeObserver`
 * và `document.fonts` đã có từ trước). Điều kiện DUY NHẤT: `await` một `nextTick()` rồi một
 * `setTimeout(…, 0)` sau `mount()`/mỗi `resize` — `@ready` của `<DockviewVue>` và
 * `requestAnimationFrame` của `restoreFocusIfLost` không giải quyết ĐỒNG BỘ với `mount()`.
 *
 * Các panel con (`GridPanel`/`LookupPanel`/`AiTranslationPanel`) mount THẬT và tự gọi
 * `invoke()` qua các adapter `src/config/*.ts` — những adapter đó KHÔNG BAO GIỜ ném (quy
 * ước `src/AGENTS.md`), nên chạy ngoài Tauri chỉ in một dòng chẩn đoán
 * (`"khong goi duoc ... chay ngoai Tauri?"`), không làm hỏng phép mount. Bỏ qua các dòng đó
 * trong log — chúng không phải lỗi của cây test này.
 */
import { afterEach, describe, expect, it, vi } from 'vitest'
import { mount } from '@vue/test-utils'
import { defineComponent, nextTick } from 'vue'
import WorkspaceDock from '../../src/layout/WorkspaceDock.vue'
import WorkspaceMode from '../../src/modes/WorkspaceMode.vue'
import { applyPreset, panelRing, togglePanel } from '../../src/layout/dockController'
import { t } from '../../src/i18n'
import type { GridNode, SerializedGrid } from '../../src/layout/dockTree'

function setWindowSize(width: number, height: number): void {
  Object.defineProperty(window, 'innerWidth', { configurable: true, value: width })
  Object.defineProperty(window, 'innerHeight', { configurable: true, value: height })
}

/** Hai token chrome mà `computeWorkArea` đọc LÚC CHẠY — 40/34 là số thật của bundle hôm nay. */
function setChromeTokens(titlebarPx = 40, statusPx = 34): void {
  document.documentElement.style.setProperty('--space-titlebar-height', `${titlebarPx}px`)
  document.documentElement.style.setProperty('--space-status-height', `${statusPx}px`)
}

/** Đợi hết `nextTick` (Vue vẽ lại) cộng một macrotask (`requestAnimationFrame` của focus). */
async function settle(): Promise<void> {
  await nextTick()
  await new Promise((resolve) => setTimeout(resolve, 0))
  await nextTick()
}

async function resize(width: number, height: number): Promise<void> {
  setWindowSize(width, height)
  window.dispatchEvent(new Event('resize'))
  await settle()
}

const SHORT_PANEL_ID: Record<string, string> = {
  'panel.grid': 'grid',
  'panel.lookup': 'lookup',
  'panel.ai_translation': 'ai_translation',
}

/**
 * HÌNH DẠNG cây lưới (nhánh + thứ tự lá) đọc từ một JSON đã `persist` — Phase 4d, đối
 * chứng cho lỗi hướng đọc từ `boundingBox`. Đọc HÌNH DẠNG, không phải TỈ LỆ sash: case D
 * (trên) đã ghi rõ tỉ lệ có thể đổi dù đúng cùng vị trí tương đối, nên so JSON đầy đủ ở
 * đây sẽ đỏ oan. `|` = nhánh NGANG (con cạnh nhau trái→phải), `/` = nhánh DỌC (con xếp
 * trên→dưới) — cùng ký hiệu bảng ASCII của hai preset ở `workspaceLayout.ts`.
 */
function shapeOf(json: string): string {
  const grid = (JSON.parse(json) as { grid: SerializedGrid }).grid
  const walk = (node: GridNode, orientation: SerializedGrid['orientation']): string => {
    if (node.type === 'leaf') {
      const views = (node.data as { views?: string[] }).views ?? []
      return views.map((v) => SHORT_PANEL_ID[v] ?? v).join(',')
    }
    const sep = orientation === 'HORIZONTAL' ? '|' : '/'
    const childOrientation: SerializedGrid['orientation'] =
      orientation === 'HORIZONTAL' ? 'VERTICAL' : 'HORIZONTAL'
    const kids = node.data as GridNode[]
    return `(${kids.map((child) => walk(child, childOrientation)).join(sep)})`
  }
  return walk(grid.root, grid.orientation)
}

/** Bản ghi `workspace_layout` thật trên máy Ice, 2026-09-23 09:23 — chép nguyên văn từ `global.db`. */
const ICE_DISK_LAYOUT_2026_09_23 =
  '{"grid":{"root":{"type":"branch","data":[{"type":"leaf","data":{"views":["panel.grid"],"activeView":"panel.grid","id":"1"},"size":578},{"type":"leaf","data":{"views":["panel.ai_translation"],"activeView":"panel.ai_translation","id":"3"},"size":488},{"type":"leaf","data":{"views":["panel.lookup"],"activeView":"panel.lookup","id":"2"},"size":419}],"size":833},"width":1485,"height":833,"orientation":"HORIZONTAL"},"panels":{"panel.grid":{"id":"panel.grid","contentComponent":"grid","tabComponent":"aura","params":{"titleKey":"panel.grid.title"},"title":"panel.grid"},"panel.lookup":{"id":"panel.lookup","contentComponent":"lookup","tabComponent":"aura","params":{"titleKey":"panel.lookup.title"},"title":"panel.lookup"},"panel.ai_translation":{"id":"panel.ai_translation","contentComponent":"aiTranslation","tabComponent":"aura","params":{"titleKey":"panel.ai_translation.title"},"title":"panel.ai_translation"}},"activeGroup":"1"}'

/** A layout where the USER dragged AI Translation into Lookup's tab group at a wide size. */
const USER_TABBED_LAYOUT =
  '{"grid":{"root":{"type":"branch","data":[{"type":"leaf","data":{"views":["panel.grid"],"activeView":"panel.grid","id":"1"},"size":578},{"type":"leaf","data":{"views":["panel.lookup","panel.ai_translation"],"activeView":"panel.lookup","id":"2"},"size":907}],"size":833},"width":1485,"height":833,"orientation":"HORIZONTAL"},"panels":{"panel.grid":{"id":"panel.grid","contentComponent":"grid","tabComponent":"aura","params":{"titleKey":"panel.grid.title"},"title":"panel.grid"},"panel.lookup":{"id":"panel.lookup","contentComponent":"lookup","tabComponent":"aura","params":{"titleKey":"panel.lookup.title"},"title":"panel.lookup"},"panel.ai_translation":{"id":"panel.ai_translation","contentComponent":"aiTranslation","tabComponent":"aura","params":{"titleKey":"panel.ai_translation.title"},"title":"panel.ai_translation"}},"activeGroup":"1"}'

// Bốn cỡ cửa sổ, mỗi cỡ rơi đúng một tầng theo `LAYOUT_THRESHOLDS['layout.preset_grid']`
// (`minFullWidth 1100` · `minFullHeight 820` · `minShortHeight 700` · `minSupportedWidth 860`,
// trừ 74px token chrome khỏi chiều cao trước khi so).
const SIZE_FULL = { width: 1200, height: 1000 } // work area 1200×926 ⇒ full
const SIZE_SHORT = { width: 1200, height: 800 } // work area 1200×726 ⇒ short (700≤h<820)
const SIZE_NARROW = { width: 1000, height: 1000 } // work area 1000×926 ⇒ narrow (w<1100)
const SIZE_UNSUPPORTED = { width: 700, height: 1000 } // work area 700×926 ⇒ unsupported (w<860)

afterEach(() => {
  vi.useRealTimers()
})

describe('WorkspaceDock — mounted thật (Story 4.12, Phase 4b)', () => {
  // ───────────────────────────────────────────────────────────────────────────
  // A — Cửa ghi TRỰC TIẾP: một lượt đổi tầng tự động, MỘT MÌNH, không bao giờ bắn `persist`.
  // ───────────────────────────────────────────────────────────────────────────
  it('A — full → narrow (autoHiddenIds đi từ rỗng lên 2) không tự bắn persist, kể cả sau khi đợi hết idle 500ms + trần cứng 5000ms', async () => {
    setChromeTokens()
    setWindowSize(SIZE_FULL.width, SIZE_FULL.height)
    const wrapper = mount(WorkspaceDock, { props: { savedLayout: '' } })
    try {
      await settle()
      expect(wrapper.find('.dock-host').attributes('data-layout-tier')).toBe('full')

      // ⚠️ ĐO ĐƯỢC: `restore()` dựng preset mặc định TRƯỚC KHI `onDidLayoutChange` được đăng
      // ký (`onReady`, xem doc-comment đầu tệp) — nên lượt dựng ban đầu KHÔNG bẩn lịch, và
      // `flush()` đầu `applyTier` no-op. Baseline thật vì vậy là 0, không phải một số dương
      // "đương nhiên". Đối chứng DƯƠNG trước: một vòng ẩn/hiện tay + `beforeunload` PHẢI bắn
      // persist — nếu không, cả cây kiểm này không chứng minh được gì (đường ghi có thể chỉ
      // đang im lặng vì không component nào chạm nó).
      expect(togglePanel('panel.ai_translation')).toBe(true)
      await settle()
      expect(togglePanel('panel.ai_translation')).toBe(true)
      await settle()
      window.dispatchEvent(new Event('beforeunload'))
      await settle()
      const baseline = wrapper.emitted('persist')?.length ?? 0
      expect(baseline).toBeGreaterThan(0)

      vi.useFakeTimers()
      setWindowSize(SIZE_NARROW.width, SIZE_NARROW.height)
      window.dispatchEvent(new Event('resize'))
      await vi.advanceTimersByTimeAsync(0)
      expect(wrapper.find('.dock-host').attributes('data-layout-tier')).toBe('narrow')
      expect(panelRing()).toEqual(['panel.grid'])

      // Không một hành động nào khác — "tự nó" đúng nghĩa đen. Đợi qua HẲN idle 500ms lẫn
      // trần cứng 5000ms của `writeSchedule.ts`: nếu cờ chặn ghi thật sự chặn thì lịch
      // KHÔNG BAO GIỜ bẩn, nên không có timer nào chờ để bắn — số `persist` phải giữ nguyên.
      // 🔵 SỬA 2026-09-23 (Phase 4c) — câu cũ ở đây khẳng định lượt ẩn ĐẦU TIÊN bắn
      // `onDidLayoutChange` trong khi `autoHiddenIds` CÒN RỖNG, nên nó phụ thuộc ĐÚNG một mình
      // `suppressPersist`. SAI: `onDidLayoutChange` là một `AsapEvent`, bắn qua `queueMicrotask`
      // (xem doc-comment `endSuppressPersist` ở `WorkspaceDock.vue`) — nó KHÔNG BAO GIỜ chạy
      // trong cùng lượt đồng bộ với `applyGridOnlySacrifice`'s vòng `for`. Cả hai lượt
      // `autoHide` (ai_translation rồi lookup) chạy xong TRƯỚC khi microtask đầu tiên có cơ
      // hội trổ ra, nên khi nó bắn, `autoHiddenIds` đã CÓ CẢ HAI id — case A vì vậy được canh
      // bởi `autoHiddenIds.size > 0`, không phải một mình `suppressPersist` (đo bằng counter-
      // check c ở dưới: gỡ vế `autoHiddenIds.size > 0` làm case B đỏ, không phải case A).
      await vi.advanceTimersByTimeAsync(6000)
      expect(wrapper.emitted('persist')?.length ?? 0).toBe(baseline)
    } finally {
      wrapper.unmount()
    }
  })

  // 🔵 SỬA 2026-09-23 (Phase 4c) — trước đây `it.fails` pin một rò rỉ ĐÃ ĐO: `full → short`
  // (nhánh `applyMerge`) bắn một `persist` TRỄ ~500ms sau khi `applyTier` đã trả về, mang
  // payload `"activeView":"panel.ai_translation"` trong group vừa gộp. Nguyên nhân: dockview
  // tự cập nhật "panel đang active của group" SAU một nhịp — KHÔNG đồng bộ với
  // `removePanel`/`addPanel` của `applyMerge` — và lượt đó bắn `onDidLayoutChange` một lần
  // nữa SAU khi cờ chặn ghi cũ (tắt ĐỒNG BỘ) đã về `false`. Phase 4c đổi `endSuppressPersist`
  // sang tắt cờ qua `queueMicrotask` (FIFO, chạy sau microtask của `onDidLayoutChange`) — cờ
  // giờ còn `true` đúng lúc lượt "active view" trễ đó bắn, nên nó bị chặn. Case này giờ
  // khẳng định KHÔNG rò, không còn là một FINDING đang chờ sửa.
  it('A2 — full → short không tự bắn persist, kể cả sau khi đợi hết idle 500ms + trần cứng 5000ms', async () => {
    setChromeTokens()
    setWindowSize(SIZE_FULL.width, SIZE_FULL.height)
    const wrapper = mount(WorkspaceDock, { props: { savedLayout: '' } })
    try {
      await settle()
      expect(togglePanel('panel.ai_translation')).toBe(true)
      await settle()
      expect(togglePanel('panel.ai_translation')).toBe(true)
      await settle()
      window.dispatchEvent(new Event('beforeunload'))
      await settle()
      const baseline = wrapper.emitted('persist')?.length ?? 0

      vi.useFakeTimers()
      setWindowSize(SIZE_SHORT.width, SIZE_SHORT.height)
      window.dispatchEvent(new Event('resize'))
      await vi.advanceTimersByTimeAsync(0)
      expect(wrapper.find('.dock-host').attributes('data-layout-merged')).toBe('true')
      await vi.advanceTimersByTimeAsync(6000)
      expect(wrapper.emitted('persist')?.length ?? 0).toBe(baseline)
    } finally {
      wrapper.unmount()
    }
  })

  // ───────────────────────────────────────────────────────────────────────────
  // B — Cửa ghi GIÁN TIẾP: tầng ẩn một panel, một hành động THẬT của người dùng làm bẩn
  // lịch, rồi một lượt flush thật xảy ra — vẫn không có `persist` nào trong khi panel còn ẩn.
  // ───────────────────────────────────────────────────────────────────────────
  it('B — narrow ẩn ai_translation + lookup; người dùng tự hiện lại lookup rồi `beforeunload` bắn flush — vẫn KHÔNG persist trong khi ai_translation còn bị tầng ẩn', async () => {
    setChromeTokens()
    setWindowSize(SIZE_FULL.width, SIZE_FULL.height)
    const wrapper = mount(WorkspaceDock, { props: { savedLayout: '' } })
    try {
      await settle()

      // Đối chứng DƯƠNG trước (cùng lý do ở case A — dựng ban đầu không bẩn lịch): một vòng
      // ẩn/hiện tay ở tầng `full` rồi `beforeunload` PHẢI bắn persist.
      expect(togglePanel('panel.ai_translation')).toBe(true)
      await settle()
      expect(togglePanel('panel.ai_translation')).toBe(true)
      await settle()
      window.dispatchEvent(new Event('beforeunload'))
      await settle()
      const baseline = wrapper.emitted('persist')?.length ?? 0
      expect(baseline).toBeGreaterThan(0)

      // Co xuống narrow: TẦNG tự ẩn cả ai_translation lẫn lookup — không persist mới (claim A).
      await resize(SIZE_NARROW.width, SIZE_NARROW.height)
      expect(wrapper.find('.dock-host').attributes('data-layout-tier')).toBe('narrow')
      expect(panelRing()).toEqual(['panel.grid'])
      expect(wrapper.emitted('persist')?.length ?? 0).toBe(baseline)

      // Hành động THẬT của người dùng, đi qua đúng cổng `layout.toggle_lookup` dùng —
      // `dockController.togglePanel`, không một lối tắt nội bộ nào của component.
      expect(togglePanel('panel.lookup')).toBe(true)
      await settle()
      expect(panelRing()).toContain('panel.lookup')
      // ai_translation vẫn ẩn VÌ TẦNG — đây chính là điều kiện claim B đòi.
      expect(panelRing()).not.toContain('panel.ai_translation')

      // Lượt flush THẬT — người dùng đóng cửa sổ ngay sau lượt hiện tay ở trên.
      window.dispatchEvent(new Event('beforeunload'))
      await settle()

      expect(wrapper.emitted('persist')?.length ?? 0).toBe(baseline)
    } finally {
      wrapper.unmount()
    }
  })

  // ───────────────────────────────────────────────────────────────────────────
  // C — Ẩn tay ≠ tầng tự động: ẩn tay đứng vững qua lượt tầng cải thiện; panel TẦNG ẩn thì
  // quay lại; và một panel người dùng tự hiện lại (trái ý tầng) bị tầng ẩn lại ở lượt ÁP KẾ
  // TIẾP — "Show direction was ruled the other way", không phải chiều ngược.
  // ───────────────────────────────────────────────────────────────────────────
  it('C — ẩn tay đứng vững qua narrow→full; panel tầng ẩn (ai_translation) thì quay lại; lookup tự hiện lại giữa lúc full vẫn bị tầng ẩn lại ở lượt narrow KẾ TIẾP', async () => {
    setChromeTokens()
    setWindowSize(SIZE_FULL.width, SIZE_FULL.height)
    const wrapper = mount(WorkspaceDock, { props: { savedLayout: '' } })
    try {
      await settle()

      // Ẩn TAY panel.lookup — `layout.toggle_lookup`, không phải tầng.
      expect(togglePanel('panel.lookup')).toBe(true)
      await settle()
      expect(panelRing()).not.toContain('panel.lookup')

      // Co cửa sổ xuống narrow: ai_translation (đang hiện) bị TẦNG ẩn; lookup đã ẩn tay từ
      // trước nên không có gì để tầng làm thêm với nó.
      await resize(SIZE_NARROW.width, SIZE_NARROW.height)
      expect(wrapper.find('.dock-host').attributes('data-layout-tier')).toBe('narrow')
      expect(panelRing()).toEqual(['panel.grid'])

      // Nới lại full: CHỈ panel TẦNG ẩn (ai_translation) trở lại. lookup vẫn ẩn — người dùng
      // chọn ẩn nó, không phải tầng, nên tầng không được đem nó về.
      await resize(SIZE_FULL.width, SIZE_FULL.height)
      expect(wrapper.find('.dock-host').attributes('data-layout-tier')).toBe('full')
      const visibleAfterGrow = panelRing()
      expect(visibleAfterGrow).toContain('panel.ai_translation')
      expect(visibleAfterGrow).not.toContain('panel.lookup')

      // Người dùng tự hiện lại lookup TRONG LÚC đang ở full — vẫn là một lượt ẩn/hiện tay.
      expect(togglePanel('panel.lookup')).toBe(true)
      await settle()
      expect(panelRing()).toContain('panel.lookup')

      // Co lại xuống narrow LẦN NỮA: tầng cấm CẢ ai_translation lẫn lookup ở tầng này — cả
      // hai bị ẩn lại, bất kể lookup vừa được người dùng tự hiện tay một khắc trước đó. Đây
      // là "tầng CẤM một panel thì ẩn lại nó ở lượt áp KẾ TIẾP" — không phải chiều ngược.
      await resize(SIZE_NARROW.width, SIZE_NARROW.height)
      expect(panelRing()).toEqual(['panel.grid'])
    } finally {
      wrapper.unmount()
    }
  })

  // ───────────────────────────────────────────────────────────────────────────
  // D — Cửa sổ nới lại: co qua ngưỡng rồi nới lại ⇒ mọi panel quay về; JSON của lượt flush
  // HỢP LỆ kế tiếp so với JSON trước lượt co, nếu đo được.
  // ───────────────────────────────────────────────────────────────────────────
  it('D — co qua ngưỡng rồi nới lại: cả ba panel quay về đúng chỗ', async () => {
    setChromeTokens()
    setWindowSize(SIZE_FULL.width, SIZE_FULL.height)
    const wrapper = mount(WorkspaceDock, { props: { savedLayout: '' } })
    try {
      await settle()

      // Mốc TRƯỚC — cùng lý do case A/B: lượt dựng ban đầu không tự bẩn lịch, nên cần một
      // vòng ẩn/hiện tay + `beforeunload` để có một lượt flush THẬT phản ánh bố cục lúc này.
      //
      // ⚠️ BẪY ĐO ĐÃ BẮT ĐƯỢC: `wrapper.emitted('persist')` trả về THAM CHIẾU SỐNG tới mảng
      // ghi nhận của `@vue/test-utils` — giữ nguyên biến đó rồi đọc `.length` SAU một lượt
      // hành động khác đọc ra độ dài MỚI, không phải độ dài lúc chụp. Chốt mốc bằng một SỐ
      // nguyên thuỷ (`beforeCount`) ngay tại chỗ, không giữ mảng.
      expect(togglePanel('panel.ai_translation')).toBe(true)
      await settle()
      expect(togglePanel('panel.ai_translation')).toBe(true)
      await settle()
      window.dispatchEvent(new Event('beforeunload'))
      await settle()
      const beforeCount = wrapper.emitted('persist')?.length ?? 0
      expect(beforeCount).toBeGreaterThan(0)
      const beforeJson = wrapper.emitted('persist')?.at(-1)?.[0] as string
      expect(typeof beforeJson).toBe('string')
      const beforeVisible = panelRing()
      expect(new Set(beforeVisible)).toEqual(
        new Set(['panel.grid', 'panel.lookup', 'panel.ai_translation']),
      )

      await resize(SIZE_NARROW.width, SIZE_NARROW.height)
      expect(panelRing()).toEqual(['panel.grid'])

      await resize(SIZE_FULL.width, SIZE_FULL.height)
      const afterVisible = panelRing()
      expect(new Set(afterVisible)).toEqual(
        new Set(['panel.grid', 'panel.lookup', 'panel.ai_translation']),
      )
      // Claim A/B (đã đóng ở trên): bản thân lượt co/nới không tự bẩn lịch, nên số `persist`
      // vẫn giữ NGUYÊN mốc trước — kiểm lại đúng claim đó, không giả định.
      expect(wrapper.emitted('persist')?.length ?? 0).toBe(beforeCount)

      // Mốc SAU — cùng lý do: bản thân lượt co/nới không tự bẩn lịch, nên cần một cớ THẬT
      // khác để có một lượt flush MỚI phản ánh đúng trạng thái đã nới lại.
      expect(togglePanel('panel.ai_translation')).toBe(true)
      await settle()
      expect(togglePanel('panel.ai_translation')).toBe(true)
      await settle()
      window.dispatchEvent(new Event('beforeunload'))
      await settle()
      const afterCount = wrapper.emitted('persist')?.length ?? 0
      expect(afterCount).toBeGreaterThan(beforeCount)
      const afterJson = wrapper.emitted('persist')?.at(-1)?.[0] as string
      expect(typeof afterJson).toBe('string')

      // ⚠️ ĐO ĐƯỢC, GHI RA THAY VÌ GIẢ ĐỊNH: `showPanel`/`rememberSpot` chỉ nhớ ANCHOR +
      // DIRECTION, không nhớ TỈ LỆ sash cũ — `addPanel` lại có thể chia lại theo tỉ lệ MỚI
      // (vd. 50/50) thay vì đúng tỉ lệ trước lúc bị tầng ẩn. Do đó JSON đầy đủ (kèm kích
      // thước sash) có thể KHÔNG bằng nhau dù đúng cùng panel, đúng cùng vị trí tương đối —
      // và đây chính là điều mệnh đề D nói "nếu đo được": đo bằng chính hai JSON thật, không
      // giả định trước rồi diễn giải cho khớp.
      const parsedBefore = JSON.parse(beforeJson) as Record<string, unknown>
      const parsedAfter = JSON.parse(afterJson) as Record<string, unknown>
      const panelIdsOf = (doc: Record<string, unknown>): string[] =>
        Object.keys((doc.panels as Record<string, unknown> | undefined) ?? {}).sort()
      expect(panelIdsOf(parsedAfter)).toEqual(panelIdsOf(parsedBefore))
      console.log('[Phase 4b][D] JSON đầy đủ (kèm sash) bằng hệt nhau?', beforeJson === afterJson)
    } finally {
      wrapper.unmount()
    }
  })

  // ───────────────────────────────────────────────────────────────────────────
  // E — Thông báo `unsupported`: `WorkspaceMode.vue` hiện thông báo không chặn ở tầng
  // `unsupported`, lưới vẫn mounted và dùng được; tầng khác thì không.
  // ───────────────────────────────────────────────────────────────────────────
  it('E — WorkspaceMode ở tầng unsupported: [data-workspace-narrow-notice] hiện đúng chuỗi đã dịch, lưới vẫn mounted; tầng full thì không', async () => {
    setChromeTokens()
    setWindowSize(SIZE_UNSUPPORTED.width, SIZE_UNSUPPORTED.height)
    const wrapper = mount(WorkspaceMode)
    try {
      await settle()
      const notice = wrapper.find('[data-workspace-narrow-notice]')
      expect(notice.exists()).toBe(true)
      expect(notice.text()).toBe(t('mode.workspace.narrow_notice'))
      expect(wrapper.find('.dock-host').exists()).toBe(true)

      await resize(SIZE_FULL.width, SIZE_FULL.height)
      expect(wrapper.find('[data-workspace-narrow-notice]').exists()).toBe(false)
      expect(wrapper.find('.dock-host').exists()).toBe(true)
    } finally {
      wrapper.unmount()
    }
  })

  // ───────────────────────────────────────────────────────────────────────────
  // F — Tra cứu vào từ thanh trạng thái: ở tầng hẹp, điểm vào `[data-lookup-drawer-open]`
  // của `StatusBar.vue` mở `LookupDrawer.vue`, và ngăn kéo render Tra cứu THẬT.
  // ───────────────────────────────────────────────────────────────────────────
  it('F — StatusBar mở LookupDrawer ở tầng narrow, ngăn kéo render Tra cứu thật', async () => {
    vi.resetModules()
    const commandsMod = await import('../../src/commands')
    const dockControllerMod = await import('../../src/layout/dockController')
    const lookupDrawerStateMod = await import('../../src/layout/lookupDrawerState')
    const { default: FreshWorkspaceDock } = await import('../../src/layout/WorkspaceDock.vue')
    const { default: FreshStatusBar } = await import('../../src/StatusBar.vue')
    const { default: FreshLookupDrawer } = await import('../../src/layout/LookupDrawer.vue')

    // Cùng khuôn `App.vue`: ba component sống như ANH EM, nối qua state module-level
    // (`lookupDrawerState.ts`), không qua prop — chỗ nối THẬT mà claim F đòi kiểm.
    const Host = defineComponent({
      components: { FreshWorkspaceDock, FreshStatusBar, FreshLookupDrawer },
      template:
        '<div><FreshWorkspaceDock saved-layout="" /><FreshStatusBar /><FreshLookupDrawer /></div>',
    })

    // `setMode` là trường BẮT BUỘC DUY NHẤT của `CommandDeps`; mọi trường khác optional.
    // Ba hàm còn lại là ĐÚNG những gì `main.ts` tiêm cho `layout.toggle_*`/`layout.preset_*`/
    // `layout.lookup_drawer_*` — không một lối tắt bỏ qua registry.
    commandsMod.installCommands({
      setMode: () => {},
      applyPreset: dockControllerMod.applyPreset,
      togglePanel: dockControllerMod.togglePanel,
      panelRing: dockControllerMod.panelRing,
      openLookupDrawer: lookupDrawerStateMod.openLookupDrawer,
      closeLookupDrawer: lookupDrawerStateMod.closeLookupDrawer,
    })

    setChromeTokens()
    setWindowSize(SIZE_NARROW.width, SIZE_NARROW.height)
    const wrapper = mount(Host)
    try {
      await settle()

      const opener = wrapper.find('[data-lookup-drawer-open]')
      expect(opener.exists()).toBe(true)
      expect(wrapper.find('[role="dialog"]').exists()).toBe(false)

      await opener.trigger('click')
      await settle()

      const dialog = wrapper.find('[role="dialog"]')
      expect(dialog.exists()).toBe(true)
      expect(dialog.text()).toContain(t('panel.lookup.title'))

      // Đóng lại qua đúng cổng — dọn state module-level trước khi tệp chạy ca kế tiếp.
      await wrapper.find('.ld-close').trigger('click')
      await settle()
      expect(wrapper.find('[role="dialog"]').exists()).toBe(false)
    } finally {
      wrapper.unmount()
    }
  })

  // Matrix row "Lookup reached from the status bar": closing the drawer returns focus where it
  // came from. Focus starts in the grid (not on the status-bar opener) and the drawer is opened
  // through `dispatch`, the keyboard path, so "back to the grid" and "back to the opener" differ.
  it('F2 — closing the Lookup drawer returns focus to where it was before opening', async () => {
    vi.resetModules()
    const commandsMod = await import('../../src/commands')
    const dockControllerMod = await import('../../src/layout/dockController')
    const lookupDrawerStateMod = await import('../../src/layout/lookupDrawerState')
    const { default: FreshWorkspaceDock } = await import('../../src/layout/WorkspaceDock.vue')
    const { default: FreshStatusBar } = await import('../../src/StatusBar.vue')
    const { default: FreshLookupDrawer } = await import('../../src/layout/LookupDrawer.vue')

    const Host = defineComponent({
      components: { FreshWorkspaceDock, FreshStatusBar, FreshLookupDrawer },
      template:
        '<div><FreshWorkspaceDock saved-layout="" /><FreshStatusBar /><FreshLookupDrawer /></div>',
    })
    commandsMod.installCommands({
      setMode: () => {},
      applyPreset: dockControllerMod.applyPreset,
      togglePanel: dockControllerMod.togglePanel,
      panelRing: dockControllerMod.panelRing,
      openLookupDrawer: lookupDrawerStateMod.openLookupDrawer,
      closeLookupDrawer: lookupDrawerStateMod.closeLookupDrawer,
    })

    setChromeTokens()
    setWindowSize(SIZE_NARROW.width, SIZE_NARROW.height)
    const wrapper = mount(Host, { attachTo: document.body })
    try {
      await settle()

      expect(commandsMod.enterFocus('panel.grid')).toBe(true)
      const origin = document.activeElement
      expect(origin).not.toBe(document.body)
      expect(origin?.matches('[data-lookup-drawer-open]')).toBe(false)

      commandsMod.dispatch('layout.lookup_drawer_open')
      await settle()
      const dialog = document.querySelector('[role="dialog"]')
      expect(dialog).not.toBeNull()
      expect(dialog?.contains(document.activeElement)).toBe(true)

      commandsMod.dispatch('layout.lookup_drawer_close')
      await settle()
      expect(document.querySelector('[role="dialog"]')).toBeNull()
      expect(document.activeElement).toBe(origin)
    } finally {
      wrapper.unmount()
    }
  })

  // AC2: a tier that removes the panel holding focus must leave focus on a real element, never
  // on `body`. The first assert after the resize proves the focused panel really left the DOM,
  // so the focus assert cannot pass merely because nothing was removed.
  it('M — focus inside Lookup survives the tier removing Lookup (full → narrow)', async () => {
    vi.resetModules()
    const commandsMod = await import('../../src/commands')
    const dockControllerMod = await import('../../src/layout/dockController')
    const { default: FreshWorkspaceDock } = await import('../../src/layout/WorkspaceDock.vue')
    commandsMod.installCommands({
      setMode: () => {},
      applyPreset: dockControllerMod.applyPreset,
      togglePanel: dockControllerMod.togglePanel,
      panelRing: dockControllerMod.panelRing,
    })

    setChromeTokens()
    setWindowSize(SIZE_FULL.width, SIZE_FULL.height)
    const wrapper = mount(FreshWorkspaceDock, { props: { savedLayout: '' }, attachTo: document.body })
    try {
      await settle()

      expect(commandsMod.enterFocus('panel.lookup')).toBe(true)
      const lookupRoot = document.activeElement as HTMLElement
      expect(lookupRoot).not.toBe(document.body)

      await resize(SIZE_NARROW.width, SIZE_NARROW.height)
      await settle()

      expect(wrapper.find('.dock-host').attributes('data-layout-tier')).toBe('narrow')
      expect(lookupRoot.isConnected).toBe(false)
      const active = document.activeElement as HTMLElement | null
      expect(active).not.toBeNull()
      expect(active).not.toBe(document.body)
      expect(active?.isConnected).toBe(true)
    } finally {
      wrapper.unmount()
    }
  })

  // The drawer is open at `narrow` and one resize lands on `full`. `applyTier` must close the
  // drawer and release `panel.lookup` before `restoreAllAutoHidden` mounts the grid copy, or the
  // grid copy's `declareFocus('panel.lookup')` throws "đã khai rồi".
  it('N — drawer open at narrow, one resize to full: drawer closes, Lookup returns to the grid once', async () => {
    vi.resetModules()
    const commandsMod = await import('../../src/commands')
    const dockControllerMod = await import('../../src/layout/dockController')
    const lookupDrawerStateMod = await import('../../src/layout/lookupDrawerState')
    const { default: FreshWorkspaceDock } = await import('../../src/layout/WorkspaceDock.vue')
    const { default: FreshStatusBar } = await import('../../src/StatusBar.vue')
    const { default: FreshLookupDrawer } = await import('../../src/layout/LookupDrawer.vue')

    const Host = defineComponent({
      components: { FreshWorkspaceDock, FreshStatusBar, FreshLookupDrawer },
      template:
        '<div><FreshWorkspaceDock saved-layout="" /><FreshStatusBar /><FreshLookupDrawer /></div>',
    })
    commandsMod.installCommands({
      setMode: () => {},
      applyPreset: dockControllerMod.applyPreset,
      togglePanel: dockControllerMod.togglePanel,
      panelRing: dockControllerMod.panelRing,
      openLookupDrawer: lookupDrawerStateMod.openLookupDrawer,
      closeLookupDrawer: lookupDrawerStateMod.closeLookupDrawer,
    })

    const errors: string[] = []
    const spy = vi.spyOn(console, 'error').mockImplementation((...args: unknown[]) => {
      errors.push(args.map(String).join(' '))
    })
    setChromeTokens()
    setWindowSize(SIZE_NARROW.width, SIZE_NARROW.height)
    const wrapper = mount(Host, { attachTo: document.body })
    try {
      await settle()
      commandsMod.dispatch('layout.lookup_drawer_open')
      await settle()
      expect(document.querySelector('[role="dialog"]')).not.toBeNull()

      await resize(SIZE_FULL.width, SIZE_FULL.height)
      await settle()

      expect(errors.filter((e) => e.includes('đã khai rồi'))).toEqual([])
      expect(lookupDrawerStateMod.lookupDrawerIsOpen.value).toBe(false)
      expect(document.querySelector('[role="dialog"]')).toBeNull()
      expect(dockControllerMod.panelRing().filter((id) => id === 'panel.lookup')).toEqual(['panel.lookup'])
      expect(commandsMod.enterFocus('panel.lookup')).toBe(true)
    } finally {
      wrapper.unmount()
      spy.mockRestore()
    }
  })

  // A panel the tier hid and the user re-showed by hand must leave `autoHiddenIds`
  // (`reconcileAutoHidden`), or the next grow re-adds it and dockview throws
  // "panel with id … already exists".
  it('P — Lookup re-shown by hand at narrow, then grow to full: every panel present exactly once', async () => {
    setChromeTokens()
    setWindowSize(SIZE_FULL.width, SIZE_FULL.height)
    const wrapper = mount(WorkspaceDock, { props: { savedLayout: '' } })
    try {
      await settle()
      await resize(SIZE_NARROW.width, SIZE_NARROW.height)
      expect(panelRing()).toEqual(['panel.grid'])
      expect(togglePanel('panel.lookup')).toBe(true)
      await settle()

      await resize(SIZE_FULL.width, SIZE_FULL.height)

      expect(wrapper.find('.dock-host').attributes('data-layout-tier')).toBe('full')
      expect([...panelRing()].sort()).toEqual(['panel.ai_translation', 'panel.grid', 'panel.lookup'])
    } finally {
      wrapper.unmount()
    }
  })

  // A tab group the USER made is not the tier's merge. Entering `short` must not claim it, so
  // growing back to `full` must leave it tabbed.
  it(
    'O — a user-made Lookup + AI tab group survives full → short → full',
    async () => {
      setChromeTokens()
      setWindowSize(SIZE_FULL.width, SIZE_FULL.height)
      const wrapper = mount(WorkspaceDock, { props: { savedLayout: USER_TABBED_LAYOUT } })
      try {
        await settle()
        await resize(SIZE_SHORT.width, SIZE_SHORT.height)
        await resize(SIZE_FULL.width, SIZE_FULL.height)
        expect(wrapper.find('.dock-host').attributes('data-layout-tier')).toBe('full')
        await new Promise((resolve) => setTimeout(resolve, 700))
        const baseline = wrapper.emitted('persist')?.length ?? 0

        expect(togglePanel('panel.lookup')).toBe(true)
        await settle()
        expect(togglePanel('panel.lookup')).toBe(true)
        await settle()
        window.dispatchEvent(new Event('beforeunload'))
        await settle()

        expect(wrapper.emitted('persist')?.length ?? 0).toBeGreaterThan(baseline)
        const shapeJson = wrapper.emitted('persist')?.at(-1)?.[0] as string
        expect(shapeOf(shapeJson)).toMatch(/^\(grid\|(lookup,ai_translation|ai_translation,lookup)\)$/)
      } finally {
        wrapper.unmount()
      }
    },
    30000,
  )

  // ───────────────────────────────────────────────────────────────────────────
  // G/H — Nới cửa sổ TRỞ LẠI (`narrow → full`, `short → full`) không tự bắn persist — chiều
  // NGƯỢC của case A/A2, đo được cùng lượt rò ở doc-comment `suppressPersist` (Phase 4b): mọi
  // chiều nới cửa sổ trở lại ghi bố cục tầng vừa dựng lại, vì cờ đồng bộ cũ đã tắt trước khi
  // `onDidLayoutChange` (một `AsapEvent`, chạy qua `queueMicrotask`) kịp tới. `endSuppressPersist`
  // sửa ở Phase 4c (tắt cờ qua `queueMicrotask`, sau microtask của dockview) phải canh cả hai
  // case này, không chỉ chiều co như A/A2.
  // ───────────────────────────────────────────────────────────────────────────
  it(
    'G — grow back from narrow persists nothing (full → narrow → full)',
    async () => {
      setChromeTokens()
      setWindowSize(SIZE_FULL.width, SIZE_FULL.height)
      const wrapper = mount(WorkspaceDock, { props: { savedLayout: '' } })
      try {
        await settle()
        expect(togglePanel('panel.ai_translation')).toBe(true)
        await settle()
        expect(togglePanel('panel.ai_translation')).toBe(true)
        await settle()
        window.dispatchEvent(new Event('beforeunload'))
        await settle()
        const baseline = wrapper.emitted('persist')?.length ?? 0
        expect(baseline).toBeGreaterThan(0)

        vi.useFakeTimers()
        setWindowSize(SIZE_NARROW.width, SIZE_NARROW.height)
        window.dispatchEvent(new Event('resize'))
        await vi.advanceTimersByTimeAsync(0)
        expect(wrapper.find('.dock-host').attributes('data-layout-tier')).toBe('narrow')
        await vi.advanceTimersByTimeAsync(6000)
        expect(wrapper.emitted('persist')?.length ?? 0).toBe(baseline)

        setWindowSize(SIZE_FULL.width, SIZE_FULL.height)
        window.dispatchEvent(new Event('resize'))
        await vi.advanceTimersByTimeAsync(0)
        expect(wrapper.find('.dock-host').attributes('data-layout-tier')).toBe('full')
        await vi.advanceTimersByTimeAsync(6000)
        expect(wrapper.emitted('persist')?.length ?? 0).toBe(baseline)

        // ⚠️ Phase 4d — G tới đây chỉ canh SỐ ĐẾM `persist`, thứ giữ nguyên dù `rememberSpot`
        // trả sai HƯỚNG (lỗi `boundingBox` cũ luôn đọc `'right'` không đổi số đếm). Cần một
        // JSON THẬT sau vòng full→narrow→full để đọc HÌNH DẠNG cây — cùng cớ "mốc SAU" của
        // case D: một vòng ẩn/hiện tay + `beforeunload`.
        vi.useRealTimers()
        expect(togglePanel('panel.ai_translation')).toBe(true)
        await settle()
        expect(togglePanel('panel.ai_translation')).toBe(true)
        await settle()
        window.dispatchEvent(new Event('beforeunload'))
        await settle()
        const shapeJson = wrapper.emitted('persist')?.at(-1)?.[0] as string
        expect(shapeOf(shapeJson)).toBe('(grid|(lookup/ai_translation))')
      } finally {
        wrapper.unmount()
      }
    },
    30000,
  )

  it(
    'H — grow back from short persists nothing (full → short → full)',
    async () => {
      setChromeTokens()
      setWindowSize(SIZE_FULL.width, SIZE_FULL.height)
      const wrapper = mount(WorkspaceDock, { props: { savedLayout: '' } })
      try {
        await settle()
        expect(togglePanel('panel.ai_translation')).toBe(true)
        await settle()
        expect(togglePanel('panel.ai_translation')).toBe(true)
        await settle()
        window.dispatchEvent(new Event('beforeunload'))
        await settle()
        const baseline = wrapper.emitted('persist')?.length ?? 0
        expect(baseline).toBeGreaterThan(0)

        vi.useFakeTimers()
        setWindowSize(SIZE_SHORT.width, SIZE_SHORT.height)
        window.dispatchEvent(new Event('resize'))
        await vi.advanceTimersByTimeAsync(0)
        expect(wrapper.find('.dock-host').attributes('data-layout-merged')).toBe('true')
        await vi.advanceTimersByTimeAsync(6000)
        expect(wrapper.emitted('persist')?.length ?? 0).toBe(baseline)

        setWindowSize(SIZE_FULL.width, SIZE_FULL.height)
        window.dispatchEvent(new Event('resize'))
        await vi.advanceTimersByTimeAsync(0)
        expect(wrapper.find('.dock-host').attributes('data-layout-tier')).toBe('full')
        expect(wrapper.find('.dock-host').attributes('data-layout-merged')).toBeUndefined()
        await vi.advanceTimersByTimeAsync(6000)
        expect(wrapper.emitted('persist')?.length ?? 0).toBe(baseline)

        // Cùng cớ đã ghi ở case G: số đếm không đổi khi lỗi hướng CŨ vẫn còn — đọc HÌNH DẠNG
        // thật của một JSON `persist` sau vòng full→short→full.
        vi.useRealTimers()
        expect(togglePanel('panel.ai_translation')).toBe(true)
        await settle()
        expect(togglePanel('panel.ai_translation')).toBe(true)
        await settle()
        window.dispatchEvent(new Event('beforeunload'))
        await settle()
        const shapeJson = wrapper.emitted('persist')?.at(-1)?.[0] as string
        expect(shapeOf(shapeJson)).toBe('(grid|(lookup/ai_translation))')
      } finally {
        wrapper.unmount()
      }
    },
    30000,
  )

  // ───────────────────────────────────────────────────────────────────────────
  // I — Một hành động THẬT của người dùng TRONG LÚC đang gộp (tầng `short`) VẪN được ghi —
  // phán quyết Ice 2026-09-23 "giữ gộp, lưu dạng chưa gộp", lật lại luật cũ (thêm sau Phase
  // 4b) từng chặn TUYỆT ĐỐI mọi lượt ghi trong lúc gộp qua vế `tierMerged.value` ở
  // `onLayoutChange`. Vế đó đã bị GỠ (không còn trong guard); cửa un-merge-để-lưu giờ nằm ở
  // `jsonForPersist` (ngay trên `flush()` trong `WorkspaceDock.vue`), gọi
  // `dockTree.ts::unmergeForPersist` để đĩa luôn chở hình dạng CHƯA GỘP dù màn hình đang hiển
  // thị đã gộp — JSON ghi xuống phải KHÔNG có lá nào chở cả hai panel.
  // ───────────────────────────────────────────────────────────────────────────
  it(
    'I — a user action while merged persists, in UN-MERGED shape',
    async () => {
      setChromeTokens()
      setWindowSize(SIZE_FULL.width, SIZE_FULL.height)
      const wrapper = mount(WorkspaceDock, { props: { savedLayout: '' } })
      try {
        await settle()

        await resize(SIZE_SHORT.width, SIZE_SHORT.height)
        expect(wrapper.find('.dock-host').attributes('data-layout-merged')).toBe('true')

        // Đợi qua hẳn idle THẬT (500ms) của `writeSchedule.ts` bằng timer thật trước khi chốt
        // mốc — cùng lý do case A/A2 phải đợi hết idle: một lịch còn "đang chờ flush" từ trước
        // sẽ làm mốc đọc sai.
        await new Promise((resolve) => setTimeout(resolve, 700))
        const baseline = wrapper.emitted('persist')?.length ?? 0

        // Hành động THẬT của người dùng TRONG LÚC còn đang gộp — đi qua đúng cổng
        // `layout.toggle_lookup`, không một lối tắt nội bộ nào.
        expect(togglePanel('panel.lookup')).toBe(true)
        await settle()
        expect(togglePanel('panel.lookup')).toBe(true)
        await settle()
        window.dispatchEvent(new Event('beforeunload'))
        await settle()

        expect(wrapper.emitted('persist')?.length ?? 0).toBeGreaterThan(baseline)
        const shapeJson = wrapper.emitted('persist')?.at(-1)?.[0] as string
        expect(shapeOf(shapeJson)).toBe('(grid|(lookup/ai_translation))')
        // 🔵 2026-09-23 (orchestrator) — hai assert trước đây gọi `not.toContain('lookup,…')` trên
        // JSON THÔ, nơi view được viết `"panel.lookup","panel.ai_translation"`, nên chúng đúng ở
        // cả hai nhánh. `shapeOf` mới là chỗ nối view cùng lá bằng dấu phẩy.
        expect(shapeOf(shapeJson)).not.toContain(',')
      } finally {
        wrapper.unmount()
      }
    },
    30000,
  )

  // ───────────────────────────────────────────────────────────────────────────
  // K — điểm neo trước gộp KHÔNG phải panel bị gộp vào. `savedLayout` là NGUYÊN VĂN bản ghi
  // `workspace_layout` trên đĩa của Ice (2026-09-23 09:23): `grid | ai_translation | lookup`.
  // Anh em trong cây của `ai_translation` là `grid`, nên `mergedSpot` là `{grid, right}`. Bản
  // Phase 4e đầu tiên chỉ tách khi nhóm gộp chứa luôn điểm neo, nên ca này ghi nguyên nhóm đã
  // gộp xuống đĩa.
  // ───────────────────────────────────────────────────────────────────────────
  it(
    'K — merged from a user layout whose anchor is not the merge partner persists the user layout',
    async () => {
      setChromeTokens()
      setWindowSize(SIZE_FULL.width, SIZE_FULL.height)
      const wrapper = mount(WorkspaceDock, { props: { savedLayout: ICE_DISK_LAYOUT_2026_09_23 } })
      try {
        await settle()
        await resize(SIZE_SHORT.width, SIZE_SHORT.height)
        expect(wrapper.find('.dock-host').attributes('data-layout-merged')).toBe('true')
        await new Promise((resolve) => setTimeout(resolve, 700))
        const baseline = wrapper.emitted('persist')?.length ?? 0

        expect(togglePanel('panel.lookup')).toBe(true)
        await settle()
        expect(togglePanel('panel.lookup')).toBe(true)
        await settle()
        window.dispatchEvent(new Event('beforeunload'))
        await settle()

        expect(wrapper.emitted('persist')?.length ?? 0).toBeGreaterThan(baseline)
        const shapeJson = wrapper.emitted('persist')?.at(-1)?.[0] as string
        expect(shapeOf(shapeJson)).toBe('(grid|ai_translation|lookup)')
      } finally {
        wrapper.unmount()
      }
    },
    30000,
  )

  // ───────────────────────────────────────────────────────────────────────────
  // L — áp preset khi tầng đang gộp/ẩn vẫn GHI preset. Đường Ice dùng để sửa bố cục ba cột
  // trên đĩa là `⌘⌥1` ở cửa sổ mặc định (tầng `short`). Đo được trước bản sửa: 0 lượt ghi ở
  // `short` lẫn `narrow`, vì thay đổi của preset và lượt gộp/ẩn của tầng về cùng MỘT microtask
  // của dockview, lúc cờ chặn ghi đang bật.
  // ───────────────────────────────────────────────────────────────────────────
  for (const [label, size] of [
    ['short', SIZE_SHORT],
    ['narrow', SIZE_NARROW],
  ] as const) {
    it(
      `L — applying Ⓑ-2 at the ${label} tier persists Ⓑ-2`,
      async () => {
        setChromeTokens()
        setWindowSize(size.width, size.height)
        const wrapper = mount(WorkspaceDock, { props: { savedLayout: ICE_DISK_LAYOUT_2026_09_23 } })
        try {
          await settle()
          expect(wrapper.find('.dock-host').attributes('data-layout-tier')).toBe(label)
          await new Promise((resolve) => setTimeout(resolve, 700))
          const baseline = wrapper.emitted('persist')?.length ?? 0

          expect(applyPreset('layout.preset_grid')).toBe(true)
          await settle()
          await new Promise((resolve) => setTimeout(resolve, 700))
          window.dispatchEvent(new Event('beforeunload'))
          await settle()

          expect(wrapper.emitted('persist')?.length ?? 0).toBeGreaterThan(baseline)
          const shapeJson = wrapper.emitted('persist')?.at(-1)?.[0] as string
          expect(shapeOf(shapeJson)).toBe('(grid|(lookup/ai_translation))')
        } finally {
          wrapper.unmount()
        }
      },
      30000,
    )
  }

  // ───────────────────────────────────────────────────────────────────────────
  // J — Ⓑ-1 (`layout.preset_columns`): cùng đối chứng G, nhưng đổi preset trước. Nhánh gốc
  // đảo trục so với Ⓑ-2 (DỌC thay vì NGANG) — nếu `findTreeSpot` đọc sai luật xen kẽ tầng
  // thì đúng ca này lộ ra, không phải Ⓑ-2 (nơi lỗi `boundingBox` cũ, luôn đọc `'right'`,
  // TÌNH CỜ khớp một nửa vì trục gốc của Ⓑ-2 vốn dĩ là NGANG).
  // ───────────────────────────────────────────────────────────────────────────
  it(
    'J — Ⓑ-1: full → narrow → full ends grid / (lookup | ai_translation)',
    async () => {
      setChromeTokens()
      setWindowSize(SIZE_FULL.width, SIZE_FULL.height)
      const wrapper = mount(WorkspaceDock, { props: { savedLayout: '' } })
      try {
        await settle()
        expect(applyPreset('layout.preset_columns')).toBe(true)
        await settle()

        vi.useFakeTimers()
        setWindowSize(SIZE_NARROW.width, SIZE_NARROW.height)
        window.dispatchEvent(new Event('resize'))
        await vi.advanceTimersByTimeAsync(0)
        expect(wrapper.find('.dock-host').attributes('data-layout-tier')).toBe('narrow')

        setWindowSize(SIZE_FULL.width, SIZE_FULL.height)
        window.dispatchEvent(new Event('resize'))
        await vi.advanceTimersByTimeAsync(0)
        expect(wrapper.find('.dock-host').attributes('data-layout-tier')).toBe('full')

        vi.useRealTimers()
        expect(togglePanel('panel.ai_translation')).toBe(true)
        await settle()
        expect(togglePanel('panel.ai_translation')).toBe(true)
        await settle()
        window.dispatchEvent(new Event('beforeunload'))
        await settle()
        const shapeJson = wrapper.emitted('persist')?.at(-1)?.[0] as string
        expect(shapeOf(shapeJson)).toBe('(grid/(lookup|ai_translation))')
      } finally {
        wrapper.unmount()
      }
    },
    30000,
  )
})
