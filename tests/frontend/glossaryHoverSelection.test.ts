/**
 * Story 3.4b — **vế TIÊU ĐIỂM/CARET của I/O Matrix "rê chuột / đưa tiêu điểm lên dấu đã
 * chốt"**, bổ sung theo yêu cầu Ice 2026-08-21 (hàng đó nằm TRONG khối
 * `<frozen-after-approval>` của spec, không phải phạm vi tự thu hẹp).
 *
 * ═════════════════════════════════════════════════════════════════════════════════
 * 🔴 MỆNH ĐỀ TRUNG TÂM: ĐẠT VẾ BÀN PHÍM VỚI **0** TAB-STOP MỚI
 * ═════════════════════════════════════════════════════════════════════════════════
 * `GridPanel.vue::onSourceSelectionChange` nghe `selectionchange` ở `document`, đọc
 * `window.getSelection().anchorNode`, ánh xạ về offset qua `sourceCutOffsetOf` (Story 2.9) —
 * ĐÚNG hàm mà click-để-cắt đã dùng — rồi tra `glossaryMarksPerSegment`. Tệp này lái đúng
 * chuỗi đó bằng `Selection`/`Range` **thật** trên component **mount thật**, không gọi thẳng
 * một hàm nội bộ: `onSourceSelectionChange` không export, và mệnh đề cần canh là "component
 * PHÁT tín hiệu đúng lúc", cùng lý do `hanVietCutAnchors.test.ts` mount thật thay vì kiểm
 * `sourceCutOffsetOf` một mình.
 *
 * ⚠️ Dùng `FIXTURE_SEGMENTS` sẵn có (`support/segmentFixture.ts`, id 11/12/13, nguyên văn
 * `一。`/`二。`/`三。`) — không dựng fixture mới; `glossaryMarksForChapter` bị giả ở biên IPC
 * (`src/config/glossary.ts`) để trả một dấu cố định phủ đúng `一` (segment 11, offset [0,1)).
 */
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'
import { mount } from '@vue/test-utils'
import { FIXTURE_CHAPTER_ID, readFixture, resetRecorder } from './support/segmentFixture'
import type { GlossaryMark } from '../../src/config/glossary'

const MARK: GlossaryMark = {
  start: 0,
  end: 1,
  tier: 'global',
  is_confirmed: true,
  translation: 'Một',
}

async function docNguyenVanGia() {
  return {
    chapter: { chapter_id: FIXTURE_CHAPTER_ID, source_text: 'khong dung o day', source_lang: 'zh' },
    error: null,
  }
}

async function traDauGia() {
  return { marks: [MARK], error: null }
}

vi.mock('../../src/config/segment', async (importOriginal) => {
  const actual = await importOriginal<typeof import('../../src/config/segment')>()
  return { ...actual, readOpenChapterSegments: readFixture }
})
vi.mock('../../src/config/chapter', async (importOriginal) => {
  const actual = await importOriginal<typeof import('../../src/config/chapter')>()
  return { ...actual, readOpenChapter: docNguyenVanGia }
})
vi.mock('../../src/config/glossary', async (importOriginal) => {
  const actual = await importOriginal<typeof import('../../src/config/glossary')>()
  return { ...actual, glossaryMarksForChapter: traDauGia }
})

const STUBS = { PanelFrame: { template: '<div class="panel-frame"><slot /></div>' } }

// 🔴 Cùng chốt mà `editorClearSourceCuts.test.ts` đã trả giá để biết — gỡ MỌI wrapper giữa
// hai ca, nếu không `document.querySelector` bắt trúng DOM của lần mount CŨ.
const daMount: { unmount: () => void }[] = []
afterEach(() => {
  for (const w of daMount.splice(0)) w.unmount()
  document.body.innerHTML = ''
})

async function mountGrid() {
  vi.resetModules()
  const editorState = await import('../../src/panels/editorPanelState')
  const sourceState = await import('../../src/panels/sourcePanelState')
  const hover = await import('../../src/panels/glossaryTermHoverState')
  const GridPanel = (await import('../../src/panels/GridPanel.vue')).default

  const wrapper = mount(GridPanel, {
    props: { params: {} } as never,
    global: { stubs: STUBS },
    attachTo: document.body,
  })
  daMount.push(wrapper)

  await editorState.ensureSegmentsLoaded()
  await sourceState.ensureChapterLoaded()
  // Funnel của Story 3.4b là một `watch` — nhường vài vòng microtask/macrotask cho nó VÀ cho
  // `ensureGlossaryMarksLoaded()` (một `await` IPC giả bên trong) hoàn tất trước khi đo DOM.
  await wrapper.vm.$nextTick()
  await new Promise((resolve) => setTimeout(resolve, 0))
  await wrapper.vm.$nextTick()

  return { editorState, hover, wrapper }
}

/** Đặt caret thu gọn ở `(node, offset)` rồi PHÁT `selectionchange` — `happy-dom` không tự bắn
 * sự kiện đó theo lượt đổi `Selection` như engine thật, nên bài test phải tự phát. */
function datCaretVaPhatTinHieu(node: Node, offset: number): void {
  const selection = window.getSelection()
  expect(selection).not.toBeNull()
  const range = document.createRange()
  range.setStart(node, offset)
  range.collapse(true)
  selection?.removeAllRanges()
  selection?.addRange(range)
  document.dispatchEvent(new Event('selectionchange'))
}

/** Node văn bản trực tiếp bên trong `.src-piece[data-src-start="N"]` của segment `id`. */
function textNodeCuaManh(segmentId: number, srcStart: number): Text {
  const cell = document.querySelector<HTMLElement>(`[data-col="src"][data-segment-id="${segmentId}"]`)
  expect(cell).not.toBeNull()
  const piece = cell?.querySelector<HTMLElement>(`.src-piece[data-src-start="${srcStart}"]`)
  expect(piece).not.toBeNull()
  const textNode = piece?.firstChild
  expect(textNode?.nodeType).toBe(Node.TEXT_NODE)
  return textNode as Text
}

beforeEach(() => {
  resetRecorder()
})

describe('Story 3.4b — caret/vùng chọn bàn phím phát cùng tín hiệu với `@mouseenter` chuột', () => {
  it('mảnh mang dấu đã render đúng lớp `glossary-confirmed` (đối chứng dữ liệu trước khi đo caret)', async () => {
    const { wrapper } = await mountGrid()
    const piece = wrapper.find('[data-col="src"][data-segment-id="11"] .src-piece[data-src-start="0"]')
    expect(piece.exists()).toBe(true)
    expect(piece.classes()).toContain('glossary-confirmed')
  })

  // 🔴 Hàng I/O Matrix *"số mảnh tăng, **0** dấu ngắt đoạn mới"* — nửa thứ hai của nó không có
  // chủ cho tới ca này. Mã sản phẩm đúng (`v-if="i > 0 && piece.isPendingCut"`,
  // `GridPanel.vue:1614`), nhưng gỡ vế `&& piece.isPendingCut` thì MỌI cổng vẫn xanh trong khi
  // mỗi thuật ngữ vẽ ra một dấu ngắt đoạn GIẢ — hỏng thấy được trên màn hình, không gì đỏ.
  // Đây đúng ràng buộc "HAI tập điểm, không một" mà `§Boundaries` của spec ghi 🔴: `pendingCuts`
  // CẮT-và-VẼ, biên thuật ngữ chỉ CẮT. Fixture không mang điểm cắt nào đang chờ, nên mọi
  // `.cut-mark` xuất hiện ở đây đều chỉ có thể đến từ một biên thuật ngữ.
  it('🔴 biên thuật ngữ CẮT mà KHÔNG vẽ dấu ngắt đoạn — hai mảnh, không một `.cut-mark` nào', async () => {
    await mountGrid()
    const cell = document.querySelector<HTMLElement>('[data-col="src"][data-segment-id="11"]')
    expect(cell).not.toBeNull()
    // Đối chứng DƯƠNG: biên thuật ngữ có cắt thật (`一。` ⇒ `一` + `。`). Không có vế này thì
    // một lượt "0 `.cut-mark`" cũng xanh khi phép cắt hỏng hoàn toàn.
    expect(cell?.querySelectorAll('.src-piece')).toHaveLength(2)
    expect(cell?.querySelectorAll('.cut-mark')).toHaveLength(0)
  })

  it('🔴 caret ĐỨNG TRONG mảnh mang dấu ⇒ `hoveredGlossaryTerm` khớp ĐÚNG bản dịch của dấu đó', async () => {
    const { hover } = await mountGrid()
    expect(hover.hoveredGlossaryTerm.value).toBe(null)

    const node = textNodeCuaManh(11, 0) // '一' — segment 11, mảnh [0,1), mang MARK.
    datCaretVaPhatTinHieu(node, 0)

    expect(hover.hoveredGlossaryTerm.value).toEqual({ isConfirmed: true, translation: 'Một' })
  })

  it('🔴 caret RỜI khỏi mảnh mang dấu (sang mảnh KHÔNG mang dấu) ⇒ tín hiệu RỜI, về `null`', async () => {
    const { hover } = await mountGrid()
    datCaretVaPhatTinHieu(textNodeCuaManh(11, 0), 0)
    expect(hover.hoveredGlossaryTerm.value).not.toBe(null)

    // Mảnh `。` của CHÍNH segment 11 (data-src-start="1") không nằm trong MARK [0,1).
    datCaretVaPhatTinHieu(textNodeCuaManh(11, 1), 0)
    expect(hover.hoveredGlossaryTerm.value).toBe(null)
  })

  it('caret rời SANG một segment khác hoàn toàn (không mang dấu nào) ⇒ vẫn về `null`', async () => {
    const { hover } = await mountGrid()
    datCaretVaPhatTinHieu(textNodeCuaManh(11, 0), 0)
    expect(hover.hoveredGlossaryTerm.value).not.toBe(null)

    const doanKhac = document.querySelector<HTMLElement>('[data-col="src"][data-segment-id="12"] .src-piece')
    expect(doanKhac).not.toBeNull()
    const node = doanKhac?.firstChild
    expect(node?.nodeType).toBe(Node.TEXT_NODE)
    datCaretVaPhatTinHieu(node as Text, 0)

    expect(hover.hoveredGlossaryTerm.value).toBe(null)
  })

  it('🔴 KHÔNG `tabindex` mới trên `.src-piece` — vế bàn phím KHÔNG đi qua một tab-stop mới', async () => {
    await mountGrid()
    for (const el of document.querySelectorAll('.src-piece')) {
      expect(el.hasAttribute('tabindex')).toBe(false)
    }
  })

  it('chuột (`@mouseenter`) VÀ bàn phím (`selectionchange`) ghi vào ĐÚNG MỘT state — không hai nguồn sự thật', async () => {
    const { hover } = await mountGrid()
    const piece = document.querySelector<HTMLElement>(
      '[data-col="src"][data-segment-id="11"] .src-piece[data-src-start="0"]',
    )
    expect(piece).not.toBeNull()
    piece?.dispatchEvent(new MouseEvent('mouseenter', { bubbles: false }))
    expect(hover.hoveredGlossaryTerm.value).toEqual({ isConfirmed: true, translation: 'Một' })

    piece?.dispatchEvent(new MouseEvent('mouseleave', { bubbles: false }))
    expect(hover.hoveredGlossaryTerm.value).toBe(null)

    // Đúng con đường bàn phím đã lái ở ca trên, ghi vào ĐÚNG cùng ref.
    datCaretVaPhatTinHieu(textNodeCuaManh(11, 0), 0)
    expect(hover.hoveredGlossaryTerm.value).toEqual({ isConfirmed: true, translation: 'Một' })
  })

  it('🔴 P13 — gõ ở CỘT BẢN DỊCH (selectionchange bắn theo từng ký tự) KHÔNG xoá dấu do CHUỘT đang giữ', async () => {
    const { hover, wrapper } = await mountGrid()

    // Chuột đang đứng yên trên thuật ngữ — KHÔNG di chuyển tiếp trong suốt ca này.
    const piece = document.querySelector<HTMLElement>(
      '[data-col="src"][data-segment-id="11"] .src-piece[data-src-start="0"]',
    )
    piece?.dispatchEvent(new MouseEvent('mouseenter', { bubbles: false }))
    expect(hover.hoveredGlossaryTerm.value).toEqual({ isConfirmed: true, translation: 'Một' })

    // Mô phỏng MỘT PHÍM gõ vào ô bản dịch của một segent KHÁC (12) — `contenteditable` bắn
    // `selectionchange` cho MỌI lượt caret đổi, kể cả không kèm `Shift`. Trước bản vá P13,
    // đây LÀ đúng cú xoá bất kể-liên-quan mà rà ba lớp bắt được.
    const tgtCell = document.querySelector('[data-col="tgt"][data-segment-id="12"]')
    expect(tgtCell).not.toBeNull()
    const tgtText = tgtCell?.firstChild
    expect(tgtText?.nodeType).toBe(Node.TEXT_NODE)
    datCaretVaPhatTinHieu(tgtText as Text, 0)

    // 🔴 Mệnh đề trung tâm: dấu của thuật ngữ VẪN CÒN — một phím gõ ở cột bản dịch không phải
    // tín hiệu "caret rời một thuật ngữ", vì nó chưa từng ở trên một thuật ngữ nào cả.
    expect(hover.hoveredGlossaryTerm.value).toEqual({ isConfirmed: true, translation: 'Một' })

    wrapper.unmount()
  })
})
