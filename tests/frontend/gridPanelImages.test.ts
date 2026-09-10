/**
 * `GridPanel.vue` — ảnh đúng vị trí trong ô nguyên văn + nhãn vai. Story 6.14, FR42 · FR43.
 *
 * ⚠️ `happy-dom` canh HÀNH VI/HÌNH DẠNG DOM, không hình học thật (`tests/AGENTS.md`) — mệnh
 * đề "ảnh không phá bố cục cột" thuộc bàn đo e2e. Tệp này khẳng định đúng những gì
 * `vue-test-utils` đo được tất định: SỐ CON của mỗi cột, ảnh nằm TRONG đúng cell, nhãn vai
 * hiện đúng chữ, và khung giữ chỗ khi `<img>` trượt.
 *
 * Giả ở BIÊN IPC (`config/segment.ts::readOpenChapterSegments`), khuôn
 * `editorTypingZone.test.ts:34-51` — mount THẬT, không giả GridPanel.vue.
 */
import { beforeEach, describe, expect, it, vi } from 'vitest'
import { mount } from '@vue/test-utils'
import type { ChapterAsset, ChapterSegment } from '../../src/config/segment'

vi.mock('@tauri-apps/api/core', () => ({
  // Passthrough dễ đọc — không phải asset:// thật, chỉ cần MỘT chuỗi tất định chứa đường ghép.
  convertFileSrc: (path: string) => `mock-asset://${path}`,
  invoke: () => Promise.reject(new Error('invoke khong duoc goi truc tiep trong ca nay')),
}))

const ASSETS_DIR = '/tmp/Work.atproj/assets'

const SEGMENTS: ChapterSegment[] = [
  {
    id: 1,
    ord: 1,
    source_text: 'Cau mot.',
    target_text: 'Dich mot.',
    is_paragraph_end: false,
    retired_at: null,
    status: 'draft',
    is_omitted: false,
    is_target_paragraph_end: false,
    role: null,
  },
  {
    id: 2,
    ord: 2,
    source_text: 'mo ta',
    target_text: 'Mo ta da dich',
    is_paragraph_end: false,
    retired_at: null,
    status: 'draft',
    is_omitted: false,
    is_target_paragraph_end: false,
    role: 'alt',
  },
  {
    id: 3,
    ord: 3,
    source_text: 'Chu thich',
    target_text: 'Chu thich da dich',
    is_paragraph_end: false,
    retired_at: null,
    status: 'draft',
    is_omitted: false,
    is_target_paragraph_end: false,
    role: 'caption',
  },
  {
    id: 4,
    ord: 4,
    source_text: 'Cau hai.',
    target_text: 'Dich hai.',
    is_paragraph_end: false,
    retired_at: null,
    status: 'draft',
    is_omitted: false,
    is_target_paragraph_end: false,
    role: null,
  },
]

const ASSETS: ChapterAsset[] = [
  // Neo `0` — đầu ô của câu ĐẦU TIÊN (segment 1), trước chữ. Tệp KHÔNG có trên "đĩa" trong ca
  // test này (kích hoạt bằng `@error` thủ công) — khung giữ chỗ chỉ mang `file_name` (không
  // `source_url`), đúng I/O Matrix "ảnh thiếu và không source_url".
  {
    asset_id: 200,
    file_name: 'lead.jpg',
    source_url: null,
    after_segment_id: null,
    alt_text: null,
    caption_text: null,
  },
  // Neo sau segment 1 — mang cả alt lẫn caption đã dịch.
  {
    asset_id: 100,
    file_name: 'trail.jpg',
    source_url: 'https://example.test/trail.jpg',
    after_segment_id: 1,
    alt_text: 'Mo ta da dich',
    caption_text: 'Chu thich da dich',
  },
]

vi.mock('../../src/config/segment', async (importOriginal) => {
  const actual = await importOriginal<typeof import('../../src/config/segment')>()
  return {
    ...actual,
    readOpenChapterSegments: () =>
      Promise.resolve({
        loaded: { chapter_id: 7, segments: SEGMENTS, caret_segment_id: null, assets: ASSETS, assets_dir: ASSETS_DIR },
        error: null,
      }),
  }
})

const STUBS = { PanelFrame: { template: '<div class="panel-frame"><slot /></div>' } }

async function mountGrid() {
  vi.resetModules()
  const GridPanel = (await import('../../src/panels/GridPanel.vue')).default
  const state = await import('../../src/panels/editorPanelState')
  const wrapper = mount(GridPanel, { props: { params: {} } as never, global: { stubs: STUBS }, attachTo: document.body })
  await state.ensureSegmentsLoaded()
  await wrapper.vm.$nextTick()
  return wrapper
}

/**
 * Gốc DOM của một wrapper, MANG KIỂU. `ReturnType<typeof mount>` phân giải `mount` theo tham
 * số kiểu mặc định của nó, nên `wrapper.element` rơi về một kiểu không mang `Element` —
 * `wrapper.element.querySelector<HTMLElement>(...)` vì thế đỏ ở `vue-tsc` (`npm run build`)
 * trong khi vitest vẫn xanh, vì vitest KHÔNG kiểm kiểu. Ép MỘT lần ở đây thay vì rải `as` ở
 * từng chỗ gọi.
 */
function rootOf(wrapper: ReturnType<typeof mount>): Element {
  return wrapper.element as unknown as Element
}

function cellSrcFor(wrapper: ReturnType<typeof mount>, segmentId: number): HTMLElement {
  const el = rootOf(wrapper).querySelector<HTMLElement>(`[data-col="src"][data-segment-id="${segmentId}"]`)
  if (el === null) throw new Error(`o nguyen van cua cau ${segmentId} khong co trong DOM`)
  return el
}

beforeEach(() => {
  document.body.innerHTML = ''
})

describe('GridPanel.vue — ảnh đúng vị trí + nhãn vai (Story 6.14)', () => {
  it('🔴 mỗi trong năm cột có đúng N phần tử con và `gridTemplateRows` đếm đúng N — ảnh không sinh thêm track nào', async () => {
    const wrapper = await mountGrid()

    const grid = rootOf(wrapper).querySelector<HTMLElement>('.grid')
    expect(grid).not.toBeNull()
    expect(grid?.style.gridTemplateRows).toBe(`repeat(${SEGMENTS.length}, auto)`)

    for (const colSelector of ['.col-rule', '.col-num', '.col-src', '.col-tgt', '.col-state']) {
      const col = rootOf(wrapper).querySelector(colSelector)
      expect(col, `cột ${colSelector} phải có mặt`).not.toBeNull()
      expect(col?.children.length, `cột ${colSelector} phải có đúng ${SEGMENTS.length} phần tử con`).toBe(SEGMENTS.length)
    }
  })

  it('neo `0` đặt ảnh Ở ĐẦU ô của câu ĐẦU TIÊN, trước chữ — ảnh sau câu 1 đứng Ở CUỐI cùng ô đó', async () => {
    const wrapper = await mountGrid()

    const firstCell = cellSrcFor(wrapper, 1)
    // Hai ảnh — neo `0` (lead.jpg) VÀ neo sau câu 1 (trail.jpg) — đều thuộc ô của câu 1.
    const figures = firstCell.querySelectorAll('figure.grid-image')
    expect(figures).toHaveLength(2)

    // Không ô nào khác (câu 2/3/4) mang ảnh nào — cả hai neo đều trỏ vào câu 1.
    for (const id of [2, 3, 4]) {
      expect(cellSrcFor(wrapper, id).querySelectorAll('figure.grid-image')).toHaveLength(0)
    }
  })

  it('nhãn vai phân biệt hàng `alt`/`caption` với văn xuôi — hai hàng còn lại KHÔNG mang nhãn', async () => {
    const wrapper = await mountGrid()

    expect(cellSrcFor(wrapper, 2).querySelector('.role-label')?.textContent).toBe('mô tả ảnh')
    expect(cellSrcFor(wrapper, 3).querySelector('.role-label')?.textContent).toBe('chú thích ảnh')
    expect(cellSrcFor(wrapper, 1).querySelector('.role-label')).toBeNull()
    expect(cellSrcFor(wrapper, 4).querySelector('.role-label')).toBeNull()
  })

  it('ảnh CÓ tệp render `<img>` với `src` ghép đúng `assetsDir`/`fileName`', async () => {
    const wrapper = await mountGrid()

    const firstCell = cellSrcFor(wrapper, 1)
    const imgs = firstCell.querySelectorAll('img.chapter-image')
    expect(imgs).toHaveLength(2)
    const srcs = Array.from(imgs).map((img) => img.getAttribute('src'))
    expect(srcs).toContain(`mock-asset://${ASSETS_DIR}/lead.jpg`)
    expect(srcs).toContain(`mock-asset://${ASSETS_DIR}/trail.jpg`)
  })

  it('🔴 tệp ảnh THIẾU trên đĩa ⇒ khung giữ chỗ mang danh tính, KHÔNG trang trắng, KHÔNG throw', async () => {
    const wrapper = await mountGrid()

    const firstCell = cellSrcFor(wrapper, 1)
    const leadImg = firstCell.querySelector('img.chapter-image')
    expect(leadImg).not.toBeNull()
    await leadImg?.dispatchEvent(new Event('error'))
    await wrapper.vm.$nextTick()

    const placeholder = firstCell.querySelector('.chapter-image-missing')
    expect(placeholder).not.toBeNull()
    expect(placeholder?.textContent).toContain('lead.jpg')
    // Ảnh KHÔNG có `source_url` (nhúng cục bộ) ⇒ không dòng nguồn rỗng nào được vẽ ra.
    expect(placeholder?.textContent).not.toContain('Nguồn')
  })

  it('🔴 `<figure>` trong ô nguyên văn KHÔNG làm lệch `data-src-start`/phép đếm của `sourceCutOffsetOf`', async () => {
    // §Tasks spec 6.14 khai mệnh đề này và tự đòi: "kiểm lại bằng ca test chứ không bằng lập
    // luận". Vòng rà 2026-09-10 bắt được rằng nó chưa có ca nào — đây là ca đó.
    const wrapper = await mountGrid()
    const { sourceCutOffsetOf } = await import('../../src/panels/editorSegments')

    const cell = cellSrcFor(wrapper, 1)
    // Ô của câu 1 mang HAI `<figure>` (neo `0` ở đầu, neo-sau-câu-1 ở cuối) kẹp lấy chữ.
    expect(cell.querySelectorAll('figure.grid-image')).toHaveLength(2)

    const piece = cell.querySelector('[data-src-start]')
    expect(piece, 'ô phải có ít nhất một mảnh nguồn mang neo').not.toBeNull()
    const batDau = Number(piece?.getAttribute('data-src-start'))
    const textNode = piece?.firstChild
    expect(textNode?.nodeType).toBe(3)

    // Bấm sau ký tự thứ ba của mảnh ⇒ offset nguồn phải là `batDau + 3`, không cộng thêm gì
    // cho hai `<figure>` đứng cạnh (chúng không mang text node nào).
    expect(sourceCutOffsetOf(cell, textNode as Node, 3)).toBe(batDau + 3)
  })
})
