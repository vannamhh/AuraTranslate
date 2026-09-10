/**
 * `ReadingMode.vue` — ảnh đúng vị trí, `<figcaption>` là caption đã dịch, alt-text KHÔNG BAO
 * GIỜ hiện trên trang. Story 6.14, FR42 · FR43.
 *
 * ⚠️ `happy-dom` canh HÀNH VI/HÌNH DẠNG DOM, không hình học thật (`tests/AGENTS.md`).
 *
 * Giả Ở BIÊN IPC (`invoke` của `@tauri-apps/api/core`) — cùng khuôn `readingFrontierDom.test.ts`,
 * nên `readReadingRun()` THẬT chạy qua `isReadingRun`/`isReadingChapter` thật (một lớp nghiệm
 * thu thêm: một fixture sai hình dạng sẽ tự lộ ra ở đây, không chỉ ở test của `config/reading.ts`).
 */
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'
import { mount } from '@vue/test-utils'

const mockInvoke = vi.fn()
vi.mock('@tauri-apps/api/core', () => ({
  invoke: (...args: unknown[]) => mockInvoke(...args),
  // Passthrough tất định — không phải asset:// thật, chỉ cần đọc lại được đường đã ghép.
  convertFileSrc: (path: string) => `mock-asset://${path}`,
}))

const ASSETS_DIR = '/tmp/Work.atproj/assets'

const RUN = {
  chapters: [
    {
      chapter_id: 1,
      chapter_ord: 1,
      chapter_title: 'Chuong Mot',
      paragraphs: [
        {
          segments: [
            { id: 1, source_text: 'Mot.', target_text: 'Cau mot.', is_confirmed: true, is_marked: false },
            { id: 2, source_text: 'Hai.', target_text: 'Cau hai.', is_confirmed: true, is_marked: false },
          ],
        },
      ],
      segment_count: 2,
      images: [
        // Neo `0` — trước đoạn ĐẦU TIÊN của Chương.
        { asset_id: 100, file_name: 'lead.jpg', source_url: null, after_segment_id: null, alt_text: 'Mo ta dau chuong', caption_text: null },
        // Neo sau câu 1 — GIỮA hai câu của CÙNG một đoạn (§I/O Matrix "ảnh giữa hai câu").
        {
          asset_id: 200,
          file_name: 'mid.jpg',
          source_url: 'https://example.test/mid.jpg',
          after_segment_id: 1,
          alt_text: 'Mo ta giua doan',
          caption_text: 'Chu thich giua doan',
        },
      ],
    },
  ],
  frontier: { kind: 'end-of-work', chapter: null },
  assets_dir: ASSETS_DIR,
}

beforeEach(async () => {
  mockInvoke.mockReset()
  mockInvoke.mockImplementation((cmd: string) => {
    if (cmd === 'read_reading_run') return Promise.resolve(RUN)
    return Promise.reject(new Error(`lenh khong mong doi: ${cmd}`))
  })
  vi.resetModules()
  const state = await import('../../src/modes/readingState')
  state.resetReading()
  state.resetReadingToc()
})

let wrapper: ReturnType<typeof mount> | null = null

afterEach(() => {
  wrapper?.unmount()
  wrapper = null
})

/**
 * Gốc DOM của một wrapper, MANG KIỂU. `ReturnType<typeof mount>` phân giải `mount` theo tham
 * số kiểu mặc định của nó, nên `wrapper.element` rơi về một kiểu không mang `Element` —
 * `.querySelector(...)` vì thế đỏ ở `vue-tsc` (`npm run build`) trong khi vitest vẫn xanh, vì
 * vitest KHÔNG kiểm kiểu. Ép MỘT lần ở đây thay vì rải `as` ở từng chỗ gọi.
 */
function rootOf(wrapper: ReturnType<typeof mount>): Element {
  return wrapper.element as unknown as Element
}

async function mountReading() {
  const { default: ReadingMode } = await import('../../src/modes/ReadingMode.vue')
  const state = await import('../../src/modes/readingState')
  wrapper = mount(ReadingMode)
  await state.ensureReadingLoaded()
  await wrapper.vm.$nextTick()
  return wrapper
}

describe('ReadingMode.vue — ảnh đúng vị trí (Story 6.14)', () => {
  it('🔴 alt-text KHÔNG BAO GIỜ hiện thành văn bản trên trang — chỉ vào thuộc tính `alt`', async () => {
    const w = await mountReading()

    expect(w.text()).not.toContain('Mo ta dau chuong')
    expect(w.text()).not.toContain('Mo ta giua doan')

    const midImg = rootOf(w).querySelector('img[src*="mid.jpg"]')
    expect(midImg?.getAttribute('alt')).toBe('Mo ta giua doan')
    const leadImg = rootOf(w).querySelector('img[src*="lead.jpg"]')
    expect(leadImg?.getAttribute('alt')).toBe('Mo ta dau chuong')
  })

  it('ảnh KHÔNG có segment `alt` ⇒ `alt=""` (ảnh trang trí, không bịa chữ)', async () => {
    mockInvoke.mockImplementation((cmd: string) => {
      if (cmd === 'read_reading_run') {
        return Promise.resolve({
          ...RUN,
          chapters: [{ ...RUN.chapters[0], images: [{ ...RUN.chapters[0]!.images[0], alt_text: null }] }],
        })
      }
      return Promise.reject(new Error(`lenh khong mong doi: ${cmd}`))
    })
    const w = await mountReading()
    const leadImg = rootOf(w).querySelector('img[src*="lead.jpg"]')
    expect(leadImg?.getAttribute('alt')).toBe('')
  })

  it('`<figcaption>` mang đúng `caption` đã dịch; ảnh không caption ⇒ không dựng `<figcaption>` nào', async () => {
    const w = await mountReading()

    const figures = rootOf(w).querySelectorAll('figure.reading-figure')
    expect(figures).toHaveLength(2)

    const midFigure = Array.from(figures).find((f) => f.querySelector('img')?.getAttribute('src')?.includes('mid.jpg'))
    expect(midFigure?.querySelector('figcaption')?.textContent).toBe('Chu thich giua doan')

    const leadFigure = Array.from(figures).find((f) => f.querySelector('img')?.getAttribute('src')?.includes('lead.jpg'))
    expect(leadFigure?.querySelector('figcaption')).toBeNull()
  })

  it('🔴 caption CHƯA DỊCH (chuỗi RỖNG) ⇒ không dựng `<figcaption>` nào — cùng luật "không chỗ trống" với `null`', async () => {
    // §I/O Matrix hàng "Caption chưa dịch": Rust trả `target_text` RỖNG nguyên vẹn (khoá bởi
    // `segment_image_contract.rs::an_untranslated_caption_segment_still_returns_its_raw_empty_target_text`);
    // quyết định "không chỗ trống" là của TẦNG HIỂN THỊ, và đây là ca canh đúng nửa đó — gỡ
    // `!== ''` khỏi `v-if` của `ReadingMode.vue` thì ca này phải ĐỎ.
    mockInvoke.mockImplementation((cmd: string) => {
      if (cmd === 'read_reading_run') {
        return Promise.resolve({
          ...RUN,
          chapters: [
            {
              ...RUN.chapters[0],
              images: [{ ...RUN.chapters[0]!.images[1], caption_text: '' }],
            },
          ],
        })
      }
      return Promise.reject(new Error(`lenh khong mong doi: ${cmd}`))
    })
    const w = await mountReading()

    const figures = rootOf(w).querySelectorAll('figure.reading-figure')
    expect(figures).toHaveLength(1)
    expect(figures[0].querySelector('figcaption')).toBeNull()
  })

  it('ảnh neo `0` đứng TRƯỚC đoạn đầu tiên; ảnh giữa câu 1 và câu 2 CHẺ đoạn thành hai `<p>` quanh nó', async () => {
    const w = await mountReading()

    const column = rootOf(w).querySelector('.column')
    expect(column).not.toBeNull()
    const children = Array.from(column?.children ?? [])
    const tagOf = (el: Element) => (el.tagName === 'FIGURE' ? 'figure' : el.tagName === 'P' ? 'p' : el.tagName)

    // Thứ tự: tiêu đề Chương, figure(lead), p(câu 1), figure(mid), p(câu 2).
    const relevant = children.filter((el) => el.tagName === 'FIGURE' || el.classList.contains('paragraph'))
    expect(relevant.map(tagOf)).toEqual(['figure', 'p', 'figure', 'p'])

    const [leadFigure, firstP, midFigure, secondP] = relevant
    expect(leadFigure.querySelector('img')?.getAttribute('src')).toContain('lead.jpg')
    // `.segment-text` — không `.textContent` của cả `<p>`, thứ còn gồm cả nhãn nút
    // "Đánh dấu" (`.mark-affordance`, không phải nội dung của câu).
    expect(firstP.querySelector('.segment-text')?.textContent.trim()).toBe('Cau mot.')
    expect(midFigure.querySelector('img')?.getAttribute('src')).toContain('mid.jpg')
    expect(secondP.querySelector('.segment-text')?.textContent.trim()).toBe('Cau hai.')
  })

  it('🔴 hai ảnh CÙNG một neo giữa đoạn ⇒ không sinh `<p>` RỖNG nào giữa hai `<figure>`', async () => {
    // §I/O Matrix hàng "Hai ảnh cùng một neo". `paragraphRuns` đóng mảnh chữ rồi mở mảnh mới
    // sau MỖI ảnh; ảnh thứ hai vì thế đóng một mảnh chữ đã RỖNG.
    mockInvoke.mockImplementation((cmd: string) => {
      if (cmd === 'read_reading_run') {
        return Promise.resolve({
          ...RUN,
          chapters: [
            {
              ...RUN.chapters[0],
              images: [
                { ...RUN.chapters[0]!.images[1], asset_id: 200, file_name: 'mid.jpg' },
                { ...RUN.chapters[0]!.images[1], asset_id: 201, file_name: 'mid2.jpg', caption_text: null },
              ],
            },
          ],
        })
      }
      return Promise.reject(new Error(`lenh khong mong doi: ${cmd}`))
    })
    const w = await mountReading()

    const empties = Array.from(rootOf(w).querySelectorAll('p.paragraph')).filter(
      (p) => p.querySelectorAll('.reading-segment').length === 0,
    )
    expect(empties, 'khong doan nao duoc render rong').toHaveLength(0)
  })

  it('`<img>` src ghép đúng `assetsDir`/`fileName`', async () => {
    const w = await mountReading()

    const midImg = rootOf(w).querySelector('img[alt="Mo ta giua doan"]')
    expect(midImg?.getAttribute('src')).toBe(`mock-asset://${ASSETS_DIR}/mid.jpg`)
  })

  it('🔴 tệp ảnh THIẾU trên đĩa ⇒ khung giữ chỗ mang `file_name` + `source_url`, chọn được để copy', async () => {
    const w = await mountReading()

    const midImg = rootOf(w).querySelector('img[alt="Mo ta giua doan"]')
    expect(midImg).not.toBeNull()
    await midImg?.dispatchEvent(new Event('error'))
    await w.vm.$nextTick()

    const figures = rootOf(w).querySelectorAll('figure.reading-figure')
    const midFigure = Array.from(figures).find((f) => f.textContent.includes('mid.jpg'))
    expect(midFigure).toBeDefined()
    const placeholder = midFigure?.querySelector('.chapter-image-missing')
    expect(placeholder?.textContent).toContain('mid.jpg')
    expect(placeholder?.textContent).toContain('https://example.test/mid.jpg')
    // Chú thích vẫn hiện — khung giữ chỗ thay CHỖ CỦA ẢNH, không nuốt luôn caption.
    expect(midFigure?.querySelector('figcaption')?.textContent).toBe('Chu thich giua doan')
  })
})
