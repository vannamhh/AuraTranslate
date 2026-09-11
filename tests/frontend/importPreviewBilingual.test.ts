/**
 * State + kết dựng THẬT của lớp phủ **Xem trước lượt nhập song ngữ**
 * (`src/bilingualImportPreviewState.ts` + `src/BilingualImportPreviewOverlay.vue`) — Story
 * 6.16, FR115.
 *
 * ⚠️ Cùng khuôn `importPreviewEncoding.test.ts`/`importPreviewOverlayRender.test.ts`:
 * `config/project.ts` là biên IPC, giả lập bằng `vi.mock`, không gọi `@tauri-apps/api` thật.
 *
 * ⚠️ **Thứ tự bắt buộc trong mỗi ca**: `freshState()`/`freshOverlay()` TRƯỚC, cấu hình
 * `mockResolvedValue` SAU — cả hai tự `mockReset()` mọi mock.
 */
import { beforeEach, describe, expect, it, vi } from 'vitest'
import { mount } from '@vue/test-utils'
import type { CommandDeps } from '../../src/commands'
import type { BilingualImportEncodingPreview, BilingualEncodingCandidateWire } from '../../src/config/project'

const previewMock = vi.fn()
const rebuildMock = vi.fn()
const confirmMock = vi.fn()

vi.mock('../../src/config/project', async (importOriginal) => {
  const actual = await importOriginal<typeof import('../../src/config/project')>()
  return {
    ...actual,
    previewBilingualImportFromFile: (
      path: string,
      sourceLang: string,
      chapterPattern: unknown,
      sourceColumn: number,
      targetColumn: number,
      hasHeader: boolean,
    ) => previewMock(path, sourceLang, chapterPattern, sourceColumn, targetColumn, hasHeader),
    rebuildBilingualImportPreview: (
      sourceLang: string,
      chapterPattern: unknown,
      sourceColumn: number,
      targetColumn: number,
      hasHeader: boolean,
    ) => rebuildMock(sourceLang, chapterPattern, sourceColumn, targetColumn, hasHeader),
    confirmBilingualImport: (
      name: string,
      sourceLang: string,
      genre: string,
      encoding: string,
      chapterPattern: unknown,
      sourceColumn: number,
      targetColumn: number,
      hasHeader: boolean,
    ) => confirmMock(name, sourceLang, genre, encoding, chapterPattern, sourceColumn, targetColumn, hasHeader),
  }
})

/** Nạp lại module mỗi ca — state của lớp phủ là module-level singleton. */
async function freshState() {
  vi.resetModules()
  previewMock.mockReset()
  rebuildMock.mockReset()
  confirmMock.mockReset()
  return import('../../src/bilingualImportPreviewState')
}

/** Cùng khuôn `importPreviewOverlayRender.test.ts::freshOverlay` — nạp `commands` + state +
 * component ĐỘNG trong CÙNG một lượt, để cả hai cùng một thể hiện module. */
async function freshOverlay(deps: Partial<CommandDeps> = {}) {
  vi.resetModules()
  previewMock.mockReset()
  rebuildMock.mockReset()
  confirmMock.mockReset()

  const commands = await import('../../src/commands')
  commands.installCommands(deps as CommandDeps)
  const state = await import('../../src/bilingualImportPreviewState')
  const BilingualImportPreviewOverlay = (await import('../../src/BilingualImportPreviewOverlay.vue')).default
  return { commands, state, BilingualImportPreviewOverlay }
}

function candidate(over: Partial<BilingualEncodingCandidateWire> = {}): BilingualEncodingCandidateWire {
  return {
    label: 'UTF-8',
    encoding: 'UTF-8',
    preview: 'a,b',
    row_count: 1,
    chapter_count: 1,
    pair_count: 1,
    mismatches: [],
    ...over,
  }
}

function preview(over: Partial<BilingualImportEncodingPreview> = {}): BilingualImportEncodingPreview {
  return {
    confidence: 'high',
    selected_encoding: 'UTF-8',
    candidates: [candidate()],
    sample_rows: [['a', 'b']],
    row_count: 1,
    column_count: 2,
    ...over,
  }
}

beforeEach(() => {
  document.body.innerHTML = ''
})

describe('bilingualImportPreviewState — mở, đóng, dọn', () => {
  it('mở lớp phủ gọi previewBilingualImportFromFile với cột/tiêu đề mặc định (0, 1, false)', async () => {
    const state = await freshState()
    previewMock.mockResolvedValue({ preview: preview(), error: null })

    await state.openBilingualImportPreview('Ten', 'en', '', '/tmp/hai-cot.csv')

    expect(previewMock).toHaveBeenCalledWith('/tmp/hai-cot.csv', 'en', null, 0, 1, false)
    expect(state.bilingualImportPreviewIsOpen.value).toBe(true)
    expect(state.bilingualImportPreviewStatus.value).toBe('loaded')
    expect(state.bilingualImportPreviewSelectedEncoding.value).toBe('UTF-8')
  })

  it('huỷ dọn SẠCH state — mở lại đọc thấy `unknown`, không dữ liệu cũ', async () => {
    const state = await freshState()
    previewMock.mockResolvedValue({ preview: preview(), error: null })
    await state.openBilingualImportPreview('Ten', 'en', '', '/tmp/hai-cot.csv')

    state.cancelBilingualImportPreview()

    expect(state.bilingualImportPreviewIsOpen.value).toBe(false)
    expect(state.bilingualImportPreviewStatus.value).toBe('unknown')
    expect(state.bilingualImportPreview.value).toBeNull()
  })

  it('lỗi IPC ⇒ status "error", `preview` giữ `null`', async () => {
    const state = await freshState()
    const err = { code: 'import.bilingual_too_few_columns', message_key: 'err.import.bilingual_too_few_columns', params: { found: '1' }, retryable: false }
    previewMock.mockResolvedValue({ preview: null, error: err })

    await state.openBilingualImportPreview('Ten', 'en', '', '/tmp/mot-cot.tsv')

    expect(state.bilingualImportPreviewStatus.value).toBe('error')
    expect(state.bilingualImportPreviewLoadError.value).toEqual(err)
    expect(state.bilingualImportPreview.value).toBeNull()
  })
})

describe('bilingualImportPreviewState — vai cột, tiêu đề, mẫu phân tách rebuild TRÊN BYTE ĐÃ CẤT', () => {
  it('đảo vai cột dựng lại trên nguồn đã cất (không gửi lại path), vai cột đã đảo', async () => {
    const state = await freshState()
    previewMock.mockResolvedValue({ preview: preview(), error: null })
    await state.openBilingualImportPreview('Ten', 'en', '', '/tmp/hai-cot.csv')
    previewMock.mockClear()
    rebuildMock.mockResolvedValue({ preview: preview(), error: null })

    await state.swapBilingualColumns()

    expect(rebuildMock).toHaveBeenCalledWith('en', null, 1, 0, false)
    expect(previewMock).not.toHaveBeenCalled()
    expect(state.bilingualImportPreviewSourceColumn.value).toBe(1)
    expect(state.bilingualImportPreviewTargetColumn.value).toBe(0)
  })

  it('bật cờ tiêu đề dựng lại trên nguồn đã cất (không gửi lại path), `hasHeader: true`', async () => {
    const state = await freshState()
    previewMock.mockResolvedValue({ preview: preview(), error: null })
    await state.openBilingualImportPreview('Ten', 'en', '', '/tmp/hai-cot.csv')
    previewMock.mockClear()
    rebuildMock.mockResolvedValue({ preview: preview(), error: null })

    await state.setBilingualHasHeader(true)

    expect(rebuildMock).toHaveBeenCalledWith('en', null, 0, 1, true)
    expect(previewMock).not.toHaveBeenCalled()
    expect(state.bilingualImportPreviewHasHeader.value).toBe(true)
  })

  it('gửi mẫu phân tách Chương dựng lại trên nguồn đã cất với mẫu đó', async () => {
    const state = await freshState()
    previewMock.mockResolvedValue({ preview: preview(), error: null })
    await state.openBilingualImportPreview('Ten', 'en', '', '/tmp/hai-cot.csv')
    previewMock.mockClear()
    rebuildMock.mockResolvedValue({ preview: preview(), error: null })

    await state.setBilingualChapterPattern('CHUONG', 'literal')

    expect(rebuildMock).toHaveBeenCalledWith('en', { pattern: 'CHUONG', kind: 'literal' }, 0, 1, false)
    expect(previewMock).not.toHaveBeenCalled()
  })

  // §I/O Matrix "Encoding change" — vế "roles and header choice kept".
  it('đổi ứng viên bảng mã giữ nguyên vai cột và cờ tiêu đề, xác nhận gửi đúng bộ ba đó cùng bảng mã mới', async () => {
    const state = await freshState()
    const twoCandidates = preview({ candidates: [candidate(), candidate({ label: 'GBK', encoding: 'GBK' })] })
    previewMock.mockResolvedValue({ preview: twoCandidates, error: null })
    rebuildMock.mockResolvedValue({ preview: twoCandidates, error: null })
    confirmMock.mockResolvedValue({ created: { meta: { work_id: 'w1' } }, error: null })
    await state.openBilingualImportPreview('Ten', 'zh', '', '/tmp/gbk.csv')
    await state.swapBilingualColumns()
    await state.setBilingualHasHeader(true)
    rebuildMock.mockClear()

    state.selectBilingualEncoding('GBK')

    expect(rebuildMock).not.toHaveBeenCalled()
    expect(state.bilingualImportPreviewSelectedEncoding.value).toBe('GBK')
    expect(state.bilingualImportPreviewSourceColumn.value).toBe(1)
    expect(state.bilingualImportPreviewTargetColumn.value).toBe(0)
    expect(state.bilingualImportPreviewHasHeader.value).toBe(true)

    await state.confirmBilingualImportPreview()
    expect(confirmMock).toHaveBeenCalledWith('Ten', 'zh', '', 'GBK', null, 1, 0, true)
  })

  // 🔴 SỬA (vòng rà đối kháng) — `refresh()` từng GHI ĐÈ ứng viên NGƯỜI DÙNG đang chọn bằng
  // `selected_encoding` (Rust tự dò) sau MỌI lượt dựng lại; ba lượt dựng lại dưới đây (đảo
  // cột, bật tiêu đề, gửi mẫu) đều phải GIỮ 'GBK' dù Rust luôn trả `selected_encoding: 'UTF-8'`.
  it('chọn một ứng viên KHÔNG mặc định rồi đảo cột / bật tiêu đề / gửi mẫu — vẫn xác nhận với ứng viên đó', async () => {
    const state = await freshState()
    const twoCandidates = preview({ candidates: [candidate(), candidate({ label: 'GBK', encoding: 'GBK' })] })
    // Rust luôn trả 'UTF-8' làm `selected_encoding` tự dò — lượt chọn tay 'GBK' không được
    // để lộ ra ngoài qua giá trị này ở bất kỳ lượt dựng lại nào.
    previewMock.mockResolvedValue({ preview: twoCandidates, error: null })
    rebuildMock.mockResolvedValue({ preview: twoCandidates, error: null })
    confirmMock.mockResolvedValue({ created: { meta: { work_id: 'w1' } }, error: null })
    await state.openBilingualImportPreview('Ten', 'zh', '', '/tmp/gbk.csv')

    state.selectBilingualEncoding('GBK')
    expect(state.bilingualImportPreviewSelectedEncoding.value).toBe('GBK')

    await state.swapBilingualColumns()
    expect(state.bilingualImportPreviewSelectedEncoding.value).toBe('GBK')

    await state.setBilingualHasHeader(true)
    expect(state.bilingualImportPreviewSelectedEncoding.value).toBe('GBK')

    await state.setBilingualChapterPattern('CHUONG', 'literal')
    expect(state.bilingualImportPreviewSelectedEncoding.value).toBe('GBK')

    await state.confirmBilingualImportPreview()
    expect(confirmMock).toHaveBeenCalledWith('Ten', 'zh', '', 'GBK', { pattern: 'CHUONG', kind: 'literal' }, 1, 0, true)
  })

  // 🔴 SỬA (vòng rà đối kháng) — nếu ứng viên đang chọn tay BIẾN MẤT khỏi dải mới (không còn
  // khớp `encoding` nào), phải rơi về `selected_encoding` Rust trả, không giữ một mã đã chết.
  it('ứng viên đang chọn biến mất khỏi dải mới ⇒ rơi về `selected_encoding` Rust trả', async () => {
    const state = await freshState()
    const twoCandidates = preview({ candidates: [candidate(), candidate({ label: 'GBK', encoding: 'GBK' })] })
    const onlyUtf8 = preview({ candidates: [candidate()], selected_encoding: 'UTF-8' })
    previewMock.mockResolvedValue({ preview: twoCandidates, error: null })
    rebuildMock.mockResolvedValue({ preview: onlyUtf8, error: null })
    await state.openBilingualImportPreview('Ten', 'zh', '', '/tmp/gbk.csv')
    state.selectBilingualEncoding('GBK')

    await state.setBilingualHasHeader(true)

    expect(state.bilingualImportPreviewSelectedEncoding.value).toBe('UTF-8')
  })
})

describe('bilingualImportPreviewState — trùng cột thì ĐẢO vai thay vì để hai vai chỉ cùng một cột', () => {
  // 🔴 SỬA (vòng rà đối kháng) — trước sửa, chọn cho cột NGUỒN đúng cột cột ĐÍCH đang giữ làm
  // cả hai vai TRÙNG cột: mỗi hàng tự ghép với chính nó (0 hàng lệch cặp giả) và nhập nguyên
  // văn nguồn làm bản dịch của chính nó.
  it('setBilingualSourceColumn nhận đúng cột cột ĐÍCH đang giữ ⇒ đảo vai (đích lấy lại cột nguồn cũ)', async () => {
    const state = await freshState()
    previewMock.mockResolvedValue({ preview: preview(), error: null })
    await state.openBilingualImportPreview('Ten', 'en', '', '/tmp/hai-cot.csv')
    // Mặc định: nguồn=0, đích=1.
    rebuildMock.mockResolvedValue({ preview: preview(), error: null })

    await state.setBilingualSourceColumn(1)

    expect(state.bilingualImportPreviewSourceColumn.value).toBe(1)
    expect(state.bilingualImportPreviewTargetColumn.value).toBe(0)
    expect(rebuildMock).toHaveBeenCalledWith('en', null, 1, 0, false)
  })

  it('setBilingualTargetColumn nhận đúng cột cột NGUỒN đang giữ ⇒ đảo vai (nguồn lấy lại cột đích cũ)', async () => {
    const state = await freshState()
    previewMock.mockResolvedValue({ preview: preview(), error: null })
    await state.openBilingualImportPreview('Ten', 'en', '', '/tmp/hai-cot.csv')
    // Mặc định: nguồn=0, đích=1.
    rebuildMock.mockResolvedValue({ preview: preview(), error: null })

    await state.setBilingualTargetColumn(0)

    expect(state.bilingualImportPreviewTargetColumn.value).toBe(0)
    expect(state.bilingualImportPreviewSourceColumn.value).toBe(1)
    expect(rebuildMock).toHaveBeenCalledWith('en', null, 1, 0, false)
  })
})

describe('bilingualImportPreviewState — xác nhận bị KHOÁ khi còn hàng lệch cặp (Rust-side đã canh — đây là vị từ HIỂN THỊ)', () => {
  it('còn mismatch ⇒ `canConfirm === false`, xác nhận là no-op (0 lời gọi Rust)', async () => {
    const state = await freshState()
    previewMock.mockResolvedValue({
      preview: preview({ candidates: [candidate({ mismatches: [{ chapter_index: 0, row_number: 2, source_sentence_count: 2, target_sentence_count: 1 }] })] }),
      error: null,
    })
    await state.openBilingualImportPreview('Ten', 'en', '', '/tmp/lech.csv')

    expect(state.bilingualImportPreviewCanConfirm.value).toBe(false)

    const result = await state.confirmBilingualImportPreview()
    expect(confirmMock).not.toHaveBeenCalled()
    expect(result).toEqual({ created: null, error: null })
  })

  it('0 mismatch ⇒ `canConfirm === true`, xác nhận gọi `confirmBilingualImport` rồi dọn state', async () => {
    const state = await freshState()
    previewMock.mockResolvedValue({ preview: preview(), error: null })
    confirmMock.mockResolvedValue({ created: { meta: { work_id: 'w1' } }, error: null })
    await state.openBilingualImportPreview('Ten Tac Pham', 'en', 'Tieu thuyet', '/tmp/hai-cot.csv')

    expect(state.bilingualImportPreviewCanConfirm.value).toBe(true)
    const result = await state.confirmBilingualImportPreview()

    expect(confirmMock).toHaveBeenCalledWith('Ten Tac Pham', 'en', 'Tieu thuyet', 'UTF-8', null, 0, 1, false)
    expect(result.created).toEqual({ meta: { work_id: 'w1' } })
    expect(state.bilingualImportPreviewIsOpen.value).toBe(false)
  })

  it('xác nhận trượt (Rust từ chối vì mismatch xuất hiện GIỮA lúc xem trước và lúc xác nhận) ⇒ giữ nguyên overlay, hiện lỗi', async () => {
    const state = await freshState()
    previewMock.mockResolvedValue({ preview: preview(), error: null })
    const err = { code: 'import.bilingual_mismatched_rows', message_key: 'err.import.bilingual_mismatched_rows', params: { count: '1' }, retryable: false }
    confirmMock.mockResolvedValue({ created: null, error: err })
    await state.openBilingualImportPreview('Ten', 'en', '', '/tmp/hai-cot.csv')

    const result = await state.confirmBilingualImportPreview()

    expect(result.error).toEqual(err)
    expect(state.bilingualImportPreviewConfirmError.value).toEqual(err)
    expect(state.bilingualImportPreviewIsOpen.value).toBe(true)
  })
})

describe('BilingualImportPreviewOverlay.vue — nút xác nhận khoá kèm lý do khi còn hàng lệch cặp', () => {
  it('còn mismatch ⇒ nút xác nhận `disabled`, hiện đúng số hàng lệch cặp', async () => {
    const { state, BilingualImportPreviewOverlay } = await freshOverlay()
    previewMock.mockResolvedValue({
      preview: preview({
        candidates: [
          candidate({
            mismatches: [
              { chapter_index: 0, row_number: 3, source_sentence_count: 2, target_sentence_count: 1 },
            ],
          }),
        ],
      }),
      error: null,
    })
    await state.openBilingualImportPreview('Ten', 'en', '', '/tmp/lech.csv')

    const wrapper = mount(BilingualImportPreviewOverlay, { attachTo: document.body })
    const confirmBtn = wrapper.find('.bip-act-primary')
    expect(confirmBtn.attributes('disabled')).toBeDefined()
    expect(wrapper.find('.bip-tier-empty-reason').exists()).toBe(true)
    expect(wrapper.findAll('.bip-mismatch-item')).toHaveLength(1)

    wrapper.unmount()
    state.resetBilingualImportPreview()
  })

  it('0 mismatch ⇒ nút xác nhận KHÔNG `disabled`, 0 dòng lệch cặp hiện ra', async () => {
    const { state, BilingualImportPreviewOverlay } = await freshOverlay()
    previewMock.mockResolvedValue({ preview: preview(), error: null })
    await state.openBilingualImportPreview('Ten', 'en', '', '/tmp/hai-cot.csv')

    const wrapper = mount(BilingualImportPreviewOverlay, { attachTo: document.body })
    const confirmBtn = wrapper.find('.bip-act-primary')
    expect(confirmBtn.attributes('disabled')).toBeUndefined()
    expect(wrapper.find('.bip-tier-empty-reason').exists()).toBe(false)
    expect(wrapper.findAll('.bip-mismatch-item')).toHaveLength(0)

    wrapper.unmount()
    state.resetBilingualImportPreview()
  })

  it('bấm nút đảo vai gọi ĐÚNG một dispatch (`import.preview.bilingual_swap_columns`), 0 hàm khác', async () => {
    const swapMock = vi.fn()
    const { state, BilingualImportPreviewOverlay } = await freshOverlay({ swapBilingualColumns: swapMock })
    previewMock.mockResolvedValue({ preview: preview(), error: null })
    await state.openBilingualImportPreview('Ten', 'en', '', '/tmp/hai-cot.csv')

    const wrapper = mount(BilingualImportPreviewOverlay, { attachTo: document.body })
    await wrapper.find('.bip-swap').trigger('click')

    expect(swapMock).toHaveBeenCalledTimes(1)

    wrapper.unmount()
    state.resetBilingualImportPreview()
  })
})
