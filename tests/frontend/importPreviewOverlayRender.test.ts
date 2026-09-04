/**
 * Kết dựng THẬT của `ImportPreviewOverlay.vue` — Story 6.3, vòng rà đối kháng 2, mục 15/22.
 *
 * ─────────────────────────────────────────────────────────────────────────────
 * 🔴 VÌ SAO CA NÀY TỒN TẠI
 * ─────────────────────────────────────────────────────────────────────────────
 * Trước ca này, `ImportPreviewOverlay.vue` có **0** test — mọi khẳng định về nó chỉ đi qua
 * `importPreviewState.ts` (state thuần, không đụng DOM). Mục 22 của vòng rà đối kháng 2 đổi
 * `t(\`mode.library.preview.confidence_${…}\`)` (nội suy chuỗi khoá) thành
 * `t(confidenceMessageKey(…))`/`t(tierEmptyMessageKey(…))` (hàm ánh xạ `switch` CẠN) — nhưng
 * hai hàm đó sống TRONG `<script setup>`, không export được, nên đối chứng DUY NHẤT có thể
 * thấy chúng thật sự chạy đúng là DỰNG component và ĐỌC chữ trên màn hình.
 *
 * Test này CHỈ canh phần dựng ba khoá bị đổi (chip tin cậy + hai tầng rỗng) — KHÔNG một bộ
 * test đầy đủ cho toàn bộ overlay (focus trap, Tab, Esc, dải năm ứng viên, … đã có đối
 * chứng ở `importPreviewEncoding.test.ts` cho phần STATE của các hành vi đó, và vẫn CHƯA có
 * ở tầng dựng DOM — nợ còn lại, không phải phạm vi của ca này).
 *
 * ⚠️ Cùng khuôn `glossaryQueue.test.ts::freshOverlay` — `vi.resetModules()` TRƯỚC, rồi nạp
 * ĐỘNG cả state LẪN component trong CÙNG một lượt, để cả hai cùng một thể hiện module (một
 * `import` tĩnh component + một `import` động state sau `resetModules()` sẽ là HAI thể hiện
 * `importPreviewState.ts` khác nhau — component không thấy state test vừa đổi).
 */
import { beforeEach, describe, expect, it, vi } from 'vitest'
import { mount } from '@vue/test-utils'
import type { EncodingCandidateWire, ImportEncodingPreview } from '../../src/config/project'

const previewTextMock = vi.fn()

vi.mock('../../src/config/project', async (importOriginal) => {
  const actual = await importOriginal<typeof import('../../src/config/project')>()
  return {
    ...actual,
    previewImportEncodingFromText: (text: string) => previewTextMock(text),
  }
})

function candidate(over: Partial<EncodingCandidateWire> = {}): EncodingCandidateWire {
  return { label: 'UTF-8', encoding: 'UTF-8', preview: 'plain ascii', ...over }
}

function preview(over: Partial<ImportEncodingPreview> = {}): ImportEncodingPreview {
  return {
    selected_encoding: 'UTF-8',
    confidence: 'high',
    candidates: [candidate()],
    ...over,
  }
}

async function freshOverlay() {
  vi.resetModules()
  previewTextMock.mockReset()

  const state = await import('../../src/importPreviewState')
  const ImportPreviewOverlay = (await import('../../src/ImportPreviewOverlay.vue')).default
  return { state, ImportPreviewOverlay }
}

beforeEach(() => {
  document.body.innerHTML = ''
})

describe('ImportPreviewOverlay.vue — chip tin cậy + hai tầng rỗng dựng ĐÚNG chữ (mục 22)', () => {
  it.each([
    ['self_declared', 'Nguồn tự khai bảng mã'],
    ['high', 'Tự đoán · độ tin cậy cao'],
    ['low', 'Tự đoán · độ tin cậy thấp'],
  ] as const)('confidence=%s dựng đúng chip "%s"', async (confidence, expectedText) => {
    const { state, ImportPreviewOverlay } = await freshOverlay()
    previewTextMock.mockResolvedValue({ preview: preview({ confidence }), error: null })
    await state.openImportPreviewFromText('Ten', 'en', '', 'text')

    const wrapper = mount(ImportPreviewOverlay, { attachTo: document.body })
    expect(wrapper.find('.ip-confidence-chip').text()).toContain(expectedText)

    wrapper.unmount()
    state.resetImportPreview()
  })

  it('tầng 2/3 rỗng dựng đúng lý do (story_6_9 / story_6_5)', async () => {
    const { state, ImportPreviewOverlay } = await freshOverlay()
    previewTextMock.mockResolvedValue({ preview: preview(), error: null })
    await state.openImportPreviewFromText('Ten', 'en', '', 'text')

    const wrapper = mount(ImportPreviewOverlay, { attachTo: document.body })
    const reasons = wrapper.findAll('.ip-tier-empty-reason')
    expect(reasons).toHaveLength(2)
    expect(reasons[0]?.text()).toContain('Story 6.9')
    expect(reasons[1]?.text()).toContain('Story 6.5')

    wrapper.unmount()
    state.resetImportPreview()
  })
})
