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
import type {
  BilingualImportEncodingPreview,
  BilingualEncodingCandidateWire,
  BilingualMismatchWire,
  BilingualRegroupingInput,
  ChapterSplitPreviewEntryWire,
  ChapterSplitPreviewWire,
} from '../../src/config/project'
import { pickSelectOption } from './support/selectMenu'

const previewMock = vi.fn()
const rebuildMock = vi.fn()
const confirmMock = vi.fn()

vi.mock('../../src/config/project', async (importOriginal) => {
  const actual = await importOriginal<typeof import('../../src/config/project')>()
  return {
    ...actual,
    // 🔴 KHÔNG có tham số `regroupings` — hàm THẬT hardcode `[]` BÊN TRONG (lượt MỞ luôn bắt
    // đầu 0 quy nhóm, không đọc gì từ chỗ gọi). Thêm một tham số ở đây sẽ giả một chữ ký hàm
    // thật không có, làm ca đối chứng đỏ SAI CHỖ.
    previewBilingualImportFromFile: (
      path: string,
      sourceLang: string,
      chapterPattern: unknown,
      sourceColumn: number,
      targetColumn: number,
      hasHeader: boolean,
      tableIndex: number | null,
    ) => previewMock(path, sourceLang, chapterPattern, sourceColumn, targetColumn, hasHeader, tableIndex),
    rebuildBilingualImportPreview: (
      sourceLang: string,
      chapterPattern: unknown,
      sourceColumn: number,
      targetColumn: number,
      hasHeader: boolean,
      regroupings: BilingualRegroupingInput[],
      tableIndex: number | null,
    ) => rebuildMock(sourceLang, chapterPattern, sourceColumn, targetColumn, hasHeader, regroupings, tableIndex),
    confirmBilingualImport: (
      name: string,
      sourceLang: string,
      genre: string,
      encoding: string,
      chapterPattern: unknown,
      sourceColumn: number,
      targetColumn: number,
      hasHeader: boolean,
      regroupings: BilingualRegroupingInput[],
      tableIndex: number | null,
      importMismatchedRows: boolean,
    ) =>
      confirmMock(
        name,
        sourceLang,
        genre,
        encoding,
        chapterPattern,
        sourceColumn,
        targetColumn,
        hasHeader,
        regroupings,
        tableIndex,
        importMismatchedRows,
      ),
  }
})

/** Một hàng lệch cặp de facto tối giản — khớp `BilingualMismatchWire` mới (Story 6.17). */
function mismatch(over: Partial<BilingualMismatchWire> = {}): BilingualMismatchWire {
  return {
    chapter_index: 0,
    row_number: 2,
    source_sentences: ['One.', 'Two.'],
    target_line: 'Mot hai',
    target_sentence_count: 1,
    candidate_positions: [3],
    initial_cuts: [],
    proposed_cuts: [3],
    ...over,
  }
}

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
    skipped_target_sentence_count: 0,
    mismatches: [],
    // **THÊM Story 6.16b** — `null` mặc định (đường KHÔNG có mẫu tách/chưa dựng khối tách
    // Chương); ca nào cần chip/danh sách Chương thật tự truyền `chapters: chaptersWire(...)`.
    chapters: null,
    ...over,
  }
}

/** **THÊM Story 6.16b (FR132)** — một Chương tối giản của khối tách Chương, hình dạng THẬT
 * `ChapterSplitPreviewEntryWire`. */
function chapterEntry(over: Partial<ChapterSplitPreviewEntryWire> = {}): ChapterSplitPreviewEntryWire {
  return {
    ord: 1,
    title: 'CHUONG MOT',
    length: 100,
    cleanup_match_count: 0,
    joined_line_count_in_chapter: 0,
    needs_review: false,
    review_causes: [],
    chapter_origin: {
      author: null,
      site_name: null,
      url: null,
      published_at: null,
      author_confirmed: false,
      site_name_confirmed: false,
      url_confirmed: false,
      published_at_confirmed: false,
    },
    source_file: null,
    ...over,
  }
}

/** Khối tách Chương tối giản — hình dạng THẬT `ChapterSplitPreviewWire`, TÁI DÙNG nguyên trên
 * đường song ngữ (Story 6.16b) qua `BilingualEncodingCandidateWire.chapters`. */
function chaptersWire(over: Partial<ChapterSplitPreviewWire> = {}): ChapterSplitPreviewWire {
  return {
    chapter_count: 1,
    chapters: [chapterEntry()],
    broken_item_count: 0,
    needs_review_count: 0,
    clean_count: 1,
    any_signal_participated: true,
    already_chaptered: false,
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
    source_tables: [],
    table_index: null,
    table_choice_required: false,
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

    expect(previewMock).toHaveBeenCalledWith('/tmp/hai-cot.csv', 'en', null, 0, 1, false, null)
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

    expect(rebuildMock).toHaveBeenCalledWith('en', null, 1, 0, false, [], null)
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

    expect(rebuildMock).toHaveBeenCalledWith('en', null, 0, 1, true, [], null)
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

    expect(rebuildMock).toHaveBeenCalledWith('en', { pattern: 'CHUONG', kind: 'literal' }, 0, 1, false, [], null)
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
    expect(confirmMock).toHaveBeenCalledWith('Ten', 'zh', '', 'GBK', null, 1, 0, true, [], null, false)
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
    expect(confirmMock).toHaveBeenCalledWith('Ten', 'zh', '', 'GBK', { pattern: 'CHUONG', kind: 'literal' }, 1, 0, true, [], null, false)
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
    expect(rebuildMock).toHaveBeenCalledWith('en', null, 1, 0, false, [], null)
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
    expect(rebuildMock).toHaveBeenCalledWith('en', null, 1, 0, false, [], null)
  })
})

describe('bilingualImportPreviewState — xác nhận bị KHOÁ khi còn hàng lệch cặp (Rust-side đã canh — đây là vị từ HIỂN THỊ)', () => {
  it('còn mismatch ⇒ `canConfirm === false`, xác nhận là no-op (0 lời gọi Rust)', async () => {
    const state = await freshState()
    previewMock.mockResolvedValue({
      preview: preview({ candidates: [candidate({ mismatches: [mismatch({ row_number: 2 })] })] }),
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

    expect(confirmMock).toHaveBeenCalledWith('Ten Tac Pham', 'en', 'Tieu thuyet', 'UTF-8', null, 0, 1, false, [], null, false)
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
            mismatches: [mismatch({ row_number: 3 })],
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
    expect(wrapper.findAll('.bip-mismatch-focus')).toHaveLength(1)

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
    expect(wrapper.findAll('.bip-mismatch-focus')).toHaveLength(0)

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

// ═══════════════════════════════════════════════════════════════════════════════
// Story 6.16c (FR115) — nguồn bảng .docx
// ═══════════════════════════════════════════════════════════════════════════════

const TWO_TABLES = [
  { index: 0, row_count: 3, column_count: 2, first_row: ['a', 'b'] },
  { index: 1, row_count: 5, column_count: 3, first_row: ['x', 'y', 'z'] },
]

function unchosenPreview(): BilingualImportEncodingPreview {
  return preview({
    candidates: [],
    row_count: 0,
    column_count: 0,
    sample_rows: [],
    source_tables: TWO_TABLES,
    table_index: null,
    table_choice_required: true,
  })
}

function chosenPreview(index: number): BilingualImportEncodingPreview {
  return preview({
    candidates: [candidate({ label: 'docx' })],
    source_tables: TWO_TABLES,
    table_index: index,
    table_choice_required: false,
  })
}

describe('bilingualImportPreviewState — bảng .docx', () => {
  it('nhiều bảng, chưa chọn: mở gửi tableIndex null, xác nhận bị khoá và không gọi Rust', async () => {
    const state = await freshState()
    previewMock.mockResolvedValue({ preview: unchosenPreview(), error: null })
    await state.openBilingualImportPreview('Ten', 'en', '', '/tmp/hai-bang.docx')

    expect(previewMock).toHaveBeenCalledWith('/tmp/hai-bang.docx', 'en', null, 0, 1, false, null)
    expect(state.bilingualImportPreviewTableIndex.value).toBeNull()
    expect(state.bilingualImportPreviewCanConfirm.value).toBe(false)
    const result = await state.confirmBilingualImportPreview()
    expect(result).toEqual({ created: null, error: null })
    expect(confirmMock).not.toHaveBeenCalled()
  })

  it('khoá vẫn giữ khi Rust trả một ứng viên nhưng table_index còn null', async () => {
    const state = await freshState()
    previewMock.mockResolvedValue({
      preview: preview({ source_tables: TWO_TABLES, table_index: null, table_choice_required: false }),
      error: null,
    })
    await state.openBilingualImportPreview('Ten', 'en', '', '/tmp/hai-bang.docx')

    expect(state.bilingualImportPreviewCanConfirm.value).toBe(false)
  })

  it('chọn bảng: dựng lại với tableIndex đó, đưa hai cột về 0/1, mở khoá, xác nhận mang tableIndex', async () => {
    const state = await freshState()
    previewMock.mockResolvedValue({ preview: unchosenPreview(), error: null })
    await state.openBilingualImportPreview('Ten', 'en', '', '/tmp/hai-bang.docx')
    rebuildMock.mockResolvedValue({ preview: chosenPreview(1), error: null })

    await state.setBilingualTableIndex(1)

    expect(rebuildMock).toHaveBeenCalledWith('en', null, 0, 1, false, [], 1)
    expect(state.bilingualImportPreviewTableIndex.value).toBe(1)
    expect(state.bilingualImportPreviewCanConfirm.value).toBe(true)

    confirmMock.mockResolvedValue({ created: { work_id: 'w' }, error: null })
    await state.confirmBilingualImportPreview()
    expect(confirmMock).toHaveBeenCalledWith('Ten', 'en', '', 'UTF-8', null, 0, 1, false, [], 1, false)
  })

  it('một bảng: Rust tự chọn bảng 0, xác nhận mở ngay và gửi tableIndex 0', async () => {
    const state = await freshState()
    previewMock.mockResolvedValue({
      preview: preview({ source_tables: [TWO_TABLES[0]], table_index: 0 }),
      error: null,
    })
    await state.openBilingualImportPreview('Ten', 'en', '', '/tmp/mot-bang.docx')

    expect(state.bilingualImportPreviewTableIndex.value).toBe(0)
    expect(state.bilingualImportPreviewCanConfirm.value).toBe(true)
    confirmMock.mockResolvedValue({ created: { work_id: 'w' }, error: null })
    await state.confirmBilingualImportPreview()
    expect(confirmMock).toHaveBeenCalledWith('Ten', 'en', '', 'UTF-8', null, 0, 1, false, [], 0, false)
  })

  it('chỉ số bảng ngoài khoảng: lỗi có kiểu lên trạng thái error, không lùi về bảng khác', async () => {
    const state = await freshState()
    previewMock.mockResolvedValue({ preview: unchosenPreview(), error: null })
    await state.openBilingualImportPreview('Ten', 'en', '', '/tmp/hai-bang.docx')
    const error = { message_key: 'err.import.bilingual_table_out_of_range', params: { index: '7', count: '2' } }
    rebuildMock.mockResolvedValue({ preview: null, error })

    await state.setBilingualTableIndex(7)

    expect(state.bilingualImportPreviewStatus.value).toBe('error')
    expect(state.bilingualImportPreviewLoadError.value).toEqual(error)
    expect(state.bilingualImportPreviewCanConfirm.value).toBe(false)
  })

  it('đóng rồi mở tệp khác: tableIndex về null', async () => {
    const state = await freshState()
    previewMock.mockResolvedValue({ preview: chosenPreview(1), error: null })
    await state.openBilingualImportPreview('Ten', 'en', '', '/tmp/hai-bang.docx')
    expect(state.bilingualImportPreviewTableIndex.value).toBe(1)

    state.resetBilingualImportPreview()
    expect(state.bilingualImportPreviewTableIndex.value).toBeNull()
  })
})

describe('BilingualImportPreviewOverlay.vue — bảng .docx', () => {
  it('chưa chọn: có bộ chọn bảng, không có dải bảng mã, không có bộ chọn cột, nút xác nhận khoá', async () => {
    const { state, BilingualImportPreviewOverlay } = await freshOverlay()
    previewMock.mockResolvedValue({ preview: unchosenPreview(), error: null })
    await state.openBilingualImportPreview('Ten', 'en', '', '/tmp/hai-bang.docx')

    const wrapper = mount(BilingualImportPreviewOverlay, { attachTo: document.body })
    expect(wrapper.find('#bip-table-select').exists()).toBe(true)
    expect(wrapper.findAll('#bip-table-select input[type="radio"]')).toHaveLength(2)
    expect(wrapper.find('.bip-strip').exists()).toBe(false)
    expect(wrapper.find('.bip-columns').exists()).toBe(false)
    expect(wrapper.find('.bip-act-primary').attributes('disabled')).toBeDefined()

    wrapper.unmount()
    state.resetBilingualImportPreview()
  })

  it('đổi bộ chọn bảng gọi dựng lại với chỉ số đó', async () => {
    const { state, BilingualImportPreviewOverlay } = await freshOverlay()
    previewMock.mockResolvedValue({ preview: unchosenPreview(), error: null })
    await state.openBilingualImportPreview('Ten', 'en', '', '/tmp/hai-bang.docx')
    rebuildMock.mockResolvedValue({ preview: chosenPreview(0), error: null })

    const wrapper = mount(BilingualImportPreviewOverlay, { attachTo: document.body })
    await pickSelectOption(wrapper.get('#bip-table-select'), '0')

    expect(rebuildMock).toHaveBeenCalledWith('en', null, 0, 1, false, [], 0)

    wrapper.unmount()
    state.resetBilingualImportPreview()
  })

  it('nguồn .csv: không có bộ chọn bảng, dải bảng mã còn nguyên', async () => {
    const { state, BilingualImportPreviewOverlay } = await freshOverlay()
    previewMock.mockResolvedValue({ preview: preview(), error: null })
    await state.openBilingualImportPreview('Ten', 'en', '', '/tmp/hai-cot.csv')

    const wrapper = mount(BilingualImportPreviewOverlay, { attachTo: document.body })
    expect(wrapper.find('#bip-table-select').exists()).toBe(false)
    expect(wrapper.find('.bip-strip').exists()).toBe(true)

    wrapper.unmount()
    state.resetBilingualImportPreview()
  })
})

// ═══════════════════════════════════════════════════════════════════════════════
// Story 6.17 (FR116) — khớp câu trong từng cặp hàng lệch, trước khi ghi
// ═══════════════════════════════════════════════════════════════════════════════

describe('BilingualImportPreviewOverlay.vue — bảy phím dispatch ĐÚNG bảy lệnh (đối chứng đỏ: gỡ một dòng keydown)', () => {
  it('↑ ↓ ← → Enter A S dispatch đúng bảy lệnh, 0 lệnh nào khác', async () => {
    const nextMock = vi.fn()
    const prevMock = vi.fn()
    const caretLeftMock = vi.fn()
    const caretRightMock = vi.fn()
    const toggleMock = vi.fn()
    const acceptAllMock = vi.fn()
    const skipMock = vi.fn()
    const { state, BilingualImportPreviewOverlay } = await freshOverlay({
      moveToNextBilingualMismatch: nextMock,
      moveToPreviousBilingualMismatch: prevMock,
      moveBilingualCaretLeft: caretLeftMock,
      moveBilingualCaretRight: caretRightMock,
      toggleBilingualCutAtCaret: toggleMock,
      acceptAllBilingualProposals: acceptAllMock,
      skipActiveBilingualRow: skipMock,
    })
    previewMock.mockResolvedValue({ preview: preview({ candidates: [candidate({ mismatches: [mismatch()] })] }), error: null })
    await state.openBilingualImportPreview('Ten', 'en', '', '/tmp/lech.csv')

    const wrapper = mount(BilingualImportPreviewOverlay, { attachTo: document.body })
    const scrim = wrapper.get('.bip-screen')

    await scrim.trigger('keydown', { key: 'ArrowDown' })
    expect(nextMock).toHaveBeenCalledTimes(1)
    await scrim.trigger('keydown', { key: 'ArrowUp' })
    expect(prevMock).toHaveBeenCalledTimes(1)
    await scrim.trigger('keydown', { key: 'ArrowLeft' })
    expect(caretLeftMock).toHaveBeenCalledTimes(1)
    await scrim.trigger('keydown', { key: 'ArrowRight' })
    expect(caretRightMock).toHaveBeenCalledTimes(1)
    await scrim.trigger('keydown', { key: 'Enter' })
    expect(toggleMock).toHaveBeenCalledTimes(1)
    await scrim.trigger('keydown', { key: 'a' })
    expect(acceptAllMock).toHaveBeenCalledTimes(1)
    await scrim.trigger('keydown', { key: 's' })
    expect(skipMock).toHaveBeenCalledTimes(1)

    wrapper.unmount()
    state.resetBilingualImportPreview()
  })

  it('một hợp âm có bổ trợ (Mod+ArrowDown) KHÔNG bắn lệnh — lọc trước mọi nhánh', async () => {
    const nextMock = vi.fn()
    const { state, BilingualImportPreviewOverlay } = await freshOverlay({ moveToNextBilingualMismatch: nextMock })
    previewMock.mockResolvedValue({ preview: preview({ candidates: [candidate({ mismatches: [mismatch()] })] }), error: null })
    await state.openBilingualImportPreview('Ten', 'en', '', '/tmp/lech.csv')

    const wrapper = mount(BilingualImportPreviewOverlay, { attachTo: document.body })
    await wrapper.get('.bip-screen').trigger('keydown', { key: 'ArrowDown', ctrlKey: true })

    expect(nextMock).not.toHaveBeenCalled()

    wrapper.unmount()
    state.resetBilingualImportPreview()
  })
})

describe('bilingualImportPreviewState — Story 6.17, Matrix bàn phím thật (không mock deps)', () => {
  it('di chuyển giữa các hàng, di caret, bật một chỗ cắt — rebuild mang đúng quy nhóm', async () => {
    const state = await freshState()
    const rowA = mismatch({ row_number: 2, source_sentences: ['One.', 'Two.'], target_line: 'Mot hai', candidate_positions: [3], proposed_cuts: [3] })
    const rowB = mismatch({
      row_number: 5,
      source_sentences: ['Ba.'],
      target_line: '',
      target_sentence_count: 0,
      candidate_positions: [],
      proposed_cuts: [],
    })
    previewMock.mockResolvedValue({ preview: preview({ candidates: [candidate({ mismatches: [rowA, rowB] })] }), error: null })
    await state.openBilingualImportPreview('Ten', 'en', '', '/tmp/lech.csv')

    expect(state.bilingualImportPreviewActiveMismatch.value?.row_number).toBe(2)

    state.moveToNextBilingualMismatch()
    expect(state.bilingualImportPreviewActiveMismatch.value?.row_number).toBe(5)
    state.moveToPreviousBilingualMismatch()
    expect(state.bilingualImportPreviewActiveMismatch.value?.row_number).toBe(2)

    state.moveBilingualCaretRight()
    expect(state.bilingualImportPreviewCaretPosition.value).toBe(3) // duy nhat mot diem ung vien

    rebuildMock.mockResolvedValue({ preview: preview({ candidates: [candidate({ mismatches: [] })] }), error: null })
    await state.toggleBilingualCutAtCaret()

    expect(rebuildMock).toHaveBeenCalledWith('en', null, 0, 1, false, [
      { row_number: 2, source_sentences: ['One.', 'Two.'], target_line: 'Mot hai', kind: 'cuts', cuts: [3] },
    ], null)
    // Hang vua cat da cap duoc (Rust tra 0 mismatch) -- danh sach rong, khoa GHI mo.
    expect(state.bilingualImportPreviewMismatches.value).toHaveLength(0)
    expect(state.bilingualImportPreviewCanConfirm.value).toBe(true)
  })

  it('bulk accept: áp đề xuất cho MỌI hàng cắt-được trong MỘT lượt, hàng chỉ-skip không nhận đề xuất', async () => {
    const state = await freshState()
    const rowA = mismatch({ row_number: 1, source_sentences: ['A1.', 'A2.'], target_line: 'a1 a2', candidate_positions: [2], proposed_cuts: [2] })
    const rowB = mismatch({ row_number: 2, source_sentences: ['B1.', 'B2.'], target_line: 'b1 b2', candidate_positions: [2], proposed_cuts: [2] })
    const rowC = mismatch({ row_number: 3, source_sentences: [], target_line: 'c mo cot day', candidate_positions: [], proposed_cuts: [] })
    previewMock.mockResolvedValue({ preview: preview({ candidates: [candidate({ mismatches: [rowA, rowB, rowC] })] }), error: null })
    await state.openBilingualImportPreview('Ten', 'en', '', '/tmp/ba-hang.csv')
    rebuildMock.mockResolvedValue({ preview: preview({ candidates: [candidate({ mismatches: [rowC] })] }), error: null })

    await state.acceptAllBilingualProposals()

    expect(rebuildMock).toHaveBeenCalledWith('en', null, 0, 1, false, [
      { row_number: 1, source_sentences: ['A1.', 'A2.'], target_line: 'a1 a2', kind: 'cuts', cuts: [2] },
      { row_number: 2, source_sentences: ['B1.', 'B2.'], target_line: 'b1 b2', kind: 'cuts', cuts: [2] },
    ], null)
    // Hang C (0 cau nguon) van con lai -- chi Skip giai quyet duoc no, khong nhan mot de xuat.
    expect(state.bilingualImportPreviewMismatches.value).toHaveLength(1)
    expect(state.bilingualImportPreviewMismatches.value[0].row_number).toBe(3)
  })

  it('bỏ qua hàng 0-vs-n gửi Skip; hàng cả hai phía có câu thì nút bỏ qua không hiện và 0 lệnh gọi Rust', async () => {
    const state = await freshState()
    const skippable = mismatch({ row_number: 3, source_sentences: [], target_line: 'con mo cot', candidate_positions: [], proposed_cuts: [] })
    const notSkippable = mismatch({ row_number: 4, source_sentences: ['X.', 'Y.'], target_line: 'x y' })
    previewMock.mockResolvedValue({ preview: preview({ candidates: [candidate({ mismatches: [skippable, notSkippable] })] }), error: null })
    await state.openBilingualImportPreview('Ten', 'en', '', '/tmp/skip.csv')

    expect(state.bilingualImportPreviewCanSkipActiveRow.value).toBe(true)
    rebuildMock.mockResolvedValue({ preview: preview({ candidates: [candidate({ mismatches: [notSkippable] })] }), error: null })
    await state.skipActiveBilingualRow()

    expect(rebuildMock).toHaveBeenCalledWith('en', null, 0, 1, false, [
      { row_number: 3, source_sentences: [], target_line: 'con mo cot', kind: 'skip', cuts: [] },
    ], null)

    // Tiêu điểm rơi về hàng còn lại — cả hai phía có câu ⇒ nút "Bỏ qua" không đủ điều kiện.
    rebuildMock.mockClear()
    expect(state.bilingualImportPreviewCanSkipActiveRow.value).toBe(false)
    await state.skipActiveBilingualRow()
    expect(rebuildMock).not.toHaveBeenCalled()
  })

  // §I/O Matrix "Skip blank source" — "count shown in preview". Con so phai DEN TU Rust
  // (`BilingualEncodingCandidateWire.skipped_target_sentence_count`, tinh lai MOI luot chay
  // tron chuoi) va SONG SOT qua chinh luot rebuild giai quyet hang do -- khac ban truoc, cong
  // don `target_sentence_count` cua cac hang CON trong `mismatches`, thu roi ve 0 dung luc
  // hang duoc giai quyet va bien mat khoi danh sach do.
  it('bỏ qua một hàng 0-vs-n — tổng câu bị bỏ đến từ Rust, còn nguyên SAU KHI rebuild giải quyết hàng đó', async () => {
    const state = await freshState()
    const skippable = mismatch({
      row_number: 3,
      source_sentences: [],
      target_line: 'con mo cot. hai cau day.',
      target_sentence_count: 2,
      candidate_positions: [],
      proposed_cuts: [],
    })
    const notSkippable = mismatch({ row_number: 4, source_sentences: ['X.', 'Y.'], target_line: 'x y' })
    previewMock.mockResolvedValue({
      preview: preview({ candidates: [candidate({ mismatches: [skippable, notSkippable] })] }),
      error: null,
    })
    await state.openBilingualImportPreview('Ten', 'en', '', '/tmp/skip.csv')
    expect(state.bilingualImportPreviewSkippedTargetSentenceCount.value).toBe(0)

    // Rust giai quyet hang 3 bang Skip NGAY TRONG luot rebuild nay -- no bien MAT khoi
    // `mismatches`, nhung `skipped_target_sentence_count` tren chinh ung vien mang dung tong
    // (2), tinh lai tu ban ghi cua CHINH luot chay nay, khong suy tu danh sach mismatch.
    rebuildMock.mockResolvedValue({
      preview: preview({
        candidates: [candidate({ mismatches: [notSkippable], skipped_target_sentence_count: 2 })],
      }),
      error: null,
    })
    await state.skipActiveBilingualRow()

    expect(state.bilingualImportPreviewMismatches.value).toEqual([notSkippable])
    expect(state.bilingualImportPreviewSkippedTargetSentenceCount.value).toBe(2)
  })
})

describe('bilingualImportPreviewState — Story 6.17, huỷ dọn SẠCH quy nhóm', () => {
  it('huỷ sau khi đã bật một chỗ cắt ⇒ lượt mở KẾ TIẾP không mang quy nhóm cũ nào', async () => {
    const state = await freshState()
    previewMock.mockResolvedValue({ preview: preview({ candidates: [candidate({ mismatches: [mismatch()] })] }), error: null })
    await state.openBilingualImportPreview('Ten', 'en', '', '/tmp/lech.csv')
    state.moveBilingualCaretRight() // caret 0 -> 3, diem ung vien duy nhat cua mismatch() mac dinh
    rebuildMock.mockResolvedValue({ preview: preview({ candidates: [candidate({ mismatches: [] })] }), error: null })
    await state.toggleBilingualCutAtCaret()
    expect(rebuildMock).toHaveBeenCalled()

    state.cancelBilingualImportPreview()
    expect(state.bilingualImportPreviewIsOpen.value).toBe(false)

    previewMock.mockClear()
    previewMock.mockResolvedValue({ preview: preview(), error: null })
    await state.openBilingualImportPreview('Ten', 'en', '', '/tmp/hai-cot.csv')
    rebuildMock.mockClear()
    rebuildMock.mockResolvedValue({ preview: preview(), error: null })

    await state.setBilingualHasHeader(true)
    expect(rebuildMock).toHaveBeenCalledWith('en', null, 0, 1, true, [], null)
  })
})

describe('bilingualImportPreviewState — Story 6.17, quy nhóm cũ (stale) không được áp lên hàng đã đổi', () => {
  it('Rust vẫn liệt kê hàng sau một chỗ cắt (ảnh chụp cũ) ⇒ hàng ở lại danh sách, xác nhận vẫn khoá', async () => {
    const state = await freshState()
    const row = mismatch({ row_number: 7 })
    previewMock.mockResolvedValue({ preview: preview({ candidates: [candidate({ mismatches: [row] })] }), error: null })
    await state.openBilingualImportPreview('Ten', 'en', '', '/tmp/doi-giua-chung.csv')
    state.moveBilingualCaretRight() // caret 0 -> 3, diem ung vien duy nhat cua mismatch() mac dinh

    // Mo phong Rust tu choi anh chup (cot doi giua chung, hoac cat khong hop le) -- hang VAN
    // con nguyen trong danh sach, khong panic, khong ném ngoại lệ nào ở tầng frontend.
    rebuildMock.mockResolvedValue({ preview: preview({ candidates: [candidate({ mismatches: [row] })] }), error: null })
    await state.toggleBilingualCutAtCaret()

    expect(state.bilingualImportPreviewMismatches.value).toHaveLength(1)
    expect(state.bilingualImportPreviewCanConfirm.value).toBe(false)

    const result = await state.confirmBilingualImportPreview()
    expect(confirmMock).not.toHaveBeenCalled()
    expect(result).toEqual({ created: null, error: null })
  })
})

// ═══════════════════════════════════════════════════════════════════════════════
// Story 6.16b (FR132) — bộ lọc "cần xem" cho bản xem trước song ngữ (tầng 4 TÁI DÙNG)
// ═══════════════════════════════════════════════════════════════════════════════

describe('BilingualImportPreviewOverlay.vue — chip "cần xem"/"sạch" + danh sách Chương', () => {
  it('chip hiện đúng needs_review_count/clean_count, Chương cần xem mang badge + nguyên nhân', async () => {
    const { state, BilingualImportPreviewOverlay } = await freshOverlay()
    const wire = chaptersWire({
      chapter_count: 2,
      needs_review_count: 1,
      clean_count: 1,
      any_signal_participated: true,
      already_chaptered: false,
      chapters: [
        chapterEntry({ ord: 1, title: 'CHUONG MOT', needs_review: false }),
        chapterEntry({ ord: 2, title: 'CHUONG HAI', needs_review: true, review_causes: ['short_length'] }),
      ],
    })
    previewMock.mockResolvedValue({ preview: preview({ candidates: [candidate({ chapters: wire })] }), error: null })
    await state.openBilingualImportPreview('Ten', 'en', '', '/tmp/hai-cot.csv')

    const wrapper = mount(BilingualImportPreviewOverlay, { attachTo: document.body })

    expect(wrapper.find('.bip-chapter-filter-chip-needs-review').text()).toContain('1')
    expect(wrapper.find('.bip-chapter-filter-chip-clean').text()).toContain('1')
    expect(wrapper.get('.bip-chapter-filter-chip-needs-review').attributes('aria-pressed')).toBe('false')
    const entries = wrapper.findAll('.bip-chapters-entry')
    expect(entries).toHaveLength(2)
    expect(entries[1].find('.bip-chapters-needs-review-badge').exists()).toBe(true)
    expect(entries[1].find('.bip-chapters-review-cause').text()).toBe('Ngắn bất thường')
    expect(entries[0].find('.bip-chapters-needs-review-badge').exists()).toBe(false)

    wrapper.unmount()
    state.resetBilingualImportPreview()
  })

  // **THÊM (đáp phản biện)** — bản trước chỉ gieo `short_length`, nên hoán đổi nội dung hai
  // nhánh còn lại (hoặc trỏ một nhánh vào sai khoá `vi.json`) trong bản sao
  // `reviewCauseMessageKey` của tệp này vẫn xanh. Bốn Chương, mỗi Chương ĐÚNG một nguyên nhân,
  // cùng khuôn `importPreviewChapters.test.ts:535` ("bốn nguyên nhân review_causes đọc ĐÚNG
  // chữ vi.json, không lẫn nhánh").
  it('bốn nguyên nhân review_causes đọc ĐÚNG chữ vi.json, không lẫn nhánh', async () => {
    const { state, BilingualImportPreviewOverlay } = await freshOverlay()
    const wire = chaptersWire({
      chapter_count: 4,
      needs_review_count: 4,
      clean_count: 0,
      any_signal_participated: true,
      already_chaptered: false,
      chapters: [
        chapterEntry({ ord: 1, title: 'C1', needs_review: true, review_causes: ['short_length'] }),
        chapterEntry({ ord: 2, title: 'C2', needs_review: true, review_causes: ['high_cleanup_matches'] }),
        chapterEntry({ ord: 3, title: 'C3', needs_review: true, review_causes: ['high_joined_lines'] }),
        chapterEntry({ ord: 4, title: 'C4', needs_review: true, review_causes: ['not_measured'] }),
      ],
    })
    previewMock.mockResolvedValue({ preview: preview({ candidates: [candidate({ chapters: wire })] }), error: null })
    await state.openBilingualImportPreview('Ten', 'en', '', '/tmp/bon-nguyen-nhan.csv')

    const wrapper = mount(BilingualImportPreviewOverlay, { attachTo: document.body })
    const rows = wrapper.findAll('.bip-chapters-entry')
    expect(rows).toHaveLength(4)
    expect(rows[0]?.find('.bip-chapters-review-cause').text()).toBe('Ngắn bất thường')
    expect(rows[1]?.find('.bip-chapters-review-cause').text()).toBe('Xoá quá nhiều')
    expect(rows[2]?.find('.bip-chapters-review-cause').text()).toBe('Nối dòng cao')
    expect(rows[3]?.find('.bip-chapters-review-cause').text()).toBe('Không đo được')

    wrapper.unmount()
    state.resetBilingualImportPreview()
  })

  it('dưới bốn Chương (any_signal_participated === false) hiện dòng "chưa đủ Chương để so", không chip', async () => {
    const { state, BilingualImportPreviewOverlay } = await freshOverlay()
    const wire = chaptersWire({
      chapter_count: 3,
      needs_review_count: 0,
      clean_count: 0,
      any_signal_participated: false,
      already_chaptered: false,
      chapters: [chapterEntry({ ord: 1 }), chapterEntry({ ord: 2 }), chapterEntry({ ord: 3 })],
    })
    previewMock.mockResolvedValue({ preview: preview({ candidates: [candidate({ chapters: wire })] }), error: null })
    await state.openBilingualImportPreview('Ten', 'en', '', '/tmp/ba-chuong.csv')

    const wrapper = mount(BilingualImportPreviewOverlay, { attachTo: document.body })

    expect(wrapper.find('.bip-chapter-filter-chip-needs-review').exists()).toBe(false)
    expect(wrapper.find('.bip-chapter-filter-note').exists()).toBe(true)

    wrapper.unmount()
    state.resetBilingualImportPreview()
  })

  it('confidence "low" hiện cờ tin cậy thấp NGOÀI chip cần xem/sạch', async () => {
    const { state, BilingualImportPreviewOverlay } = await freshOverlay()
    const wire = chaptersWire({ needs_review_count: 0, clean_count: 1 })
    previewMock.mockResolvedValue({
      preview: preview({ confidence: 'low', candidates: [candidate({ chapters: wire })] }),
      error: null,
    })
    await state.openBilingualImportPreview('Ten', 'en', '', '/tmp/thap.csv')

    const wrapper = mount(BilingualImportPreviewOverlay, { attachTo: document.body })

    expect(wrapper.find('.bip-chapter-filter-low-confidence').exists()).toBe(true)

    wrapper.unmount()
    state.resetBilingualImportPreview()
  })

  // I/O Matrix "Encoding candidate switched" — mỗi ứng viên mang khối `chapters` CỦA CHÍNH NÓ
  // (Rust dựng một khối cho mỗi ứng viên), nên đổi ứng viên phải đổi cả hai con số lẫn danh
  // sách. `selectBilingualEncoding` KHÔNG gọi IPC, nên nếu màn hình đọc khối của ứng viên ĐẦU
  // TIÊN thay vì ứng viên ĐANG CHỌN thì không lượt chạy lại nào che được chỗ đó.
  it('đổi ứng viên bảng mã làm chip và danh sách Chương đọc lại khối của CHÍNH ứng viên đó', async () => {
    const { state, BilingualImportPreviewOverlay } = await freshOverlay()
    const utf8Wire = chaptersWire({
      chapter_count: 2,
      needs_review_count: 1,
      clean_count: 1,
      any_signal_participated: true,
      already_chaptered: false,
      chapters: [
        chapterEntry({ ord: 1, title: 'UTF8 MOT', needs_review: false }),
        chapterEntry({ ord: 2, title: 'UTF8 HAI', needs_review: true, review_causes: ['short_length'] }),
      ],
    })
    const gbkWire = chaptersWire({
      chapter_count: 3,
      needs_review_count: 2,
      clean_count: 1,
      any_signal_participated: true,
      already_chaptered: false,
      chapters: [
        chapterEntry({ ord: 1, title: 'GBK MOT', needs_review: true, review_causes: ['high_joined_lines'] }),
        chapterEntry({ ord: 2, title: 'GBK HAI', needs_review: true, review_causes: ['not_measured'] }),
        chapterEntry({ ord: 3, title: 'GBK BA', needs_review: false }),
      ],
    })
    previewMock.mockResolvedValue({
      preview: preview({
        candidates: [
          candidate({ chapters: utf8Wire }),
          candidate({ label: 'GBK', encoding: 'GBK', chapters: gbkWire }),
        ],
      }),
      error: null,
    })
    await state.openBilingualImportPreview('Ten', 'zh', '', '/tmp/hai-ung-vien.csv')

    const wrapper = mount(BilingualImportPreviewOverlay, { attachTo: document.body })

    expect(wrapper.find('.bip-chapter-filter-chip-needs-review').text()).toContain('1')
    expect(wrapper.find('.bip-chapter-filter-chip-clean').text()).toContain('1')
    expect(wrapper.findAll('.bip-chapters-entry')).toHaveLength(2)

    state.selectBilingualEncoding('GBK')
    await wrapper.vm.$nextTick()

    expect(wrapper.find('.bip-chapter-filter-chip-needs-review').text()).toContain('2')
    expect(wrapper.find('.bip-chapter-filter-chip-clean').text()).toContain('1')
    const entries = wrapper.findAll('.bip-chapters-entry')
    expect(entries).toHaveLength(3)
    expect(entries[0].find('.bip-chapters-needs-review-badge').exists()).toBe(true)
    expect(entries[2].find('.bip-chapters-needs-review-badge').exists()).toBe(false)

    wrapper.unmount()
    state.resetBilingualImportPreview()
  })
})

// **THÊM (đáp phản biện)** — bộ ⌥W ở describe ngay dưới TIÊM `toggleBilingualImportPreviewChapterFilter`
// qua `CommandDeps`, nên nó chỉ canh được rằng `dispatch()` GỌI đúng hàm — không canh được
// hàm THẬT (`bilingualImportPreviewState.ts::toggleBilingualImportPreviewChapterFilter`) hay
// nhánh lọc `needs_review` của `bilingualChapterEntriesRendered` (`BilingualImportPreviewOverlay.vue`)
// có tự chạy đúng không. Xoá một trong hai (hàm thật, hoặc nhánh lọc trong `.vue`) vẫn để cả bộ
// ⌥W xanh — bốn ca dưới đây gọi THẲNG `state.toggleBilingualImportPreviewChapterFilter()`, cùng
// khuôn `importPreviewChapters.test.ts:1148/1163/1169` (bật/tắt, hai điều kiện chặn) và
// `importPreviewOverlayRender.test.ts:549` (mounted, đếm hàng DOM đổi theo bộ lọc).
describe('bilingualImportPreviewState — toggleBilingualImportPreviewChapterFilter THẬT (không tiêm mock)', () => {
  it('bật rồi tắt lại — bấm hai lần đảo trạng thái', async () => {
    const state = await freshState()
    const wire = chaptersWire({
      chapter_count: 2,
      needs_review_count: 1,
      clean_count: 1,
      any_signal_participated: true,
      already_chaptered: false,
      chapters: [
        chapterEntry({ ord: 1, needs_review: false }),
        chapterEntry({ ord: 2, needs_review: true, review_causes: ['short_length'] }),
      ],
    })
    previewMock.mockResolvedValue({ preview: preview({ candidates: [candidate({ chapters: wire })] }), error: null })
    await state.openBilingualImportPreview('Ten', 'en', '', '/tmp/hai-cot.csv')

    expect(state.bilingualImportPreviewChapterFilterActive.value).toBe(false)

    state.toggleBilingualImportPreviewChapterFilter()
    expect(state.bilingualImportPreviewChapterFilterActive.value).toBe(true)

    state.toggleBilingualImportPreviewChapterFilter()
    expect(state.bilingualImportPreviewChapterFilterActive.value).toBe(false)

    state.resetBilingualImportPreview()
  })

  it('0 Chương cần xem (needs_review_count === 0) — bộ lọc KHÔNG bật, không ném', async () => {
    const state = await freshState()
    const wire = chaptersWire({ needs_review_count: 0, clean_count: 2, any_signal_participated: true })
    previewMock.mockResolvedValue({ preview: preview({ candidates: [candidate({ chapters: wire })] }), error: null })
    await state.openBilingualImportPreview('Ten', 'en', '', '/tmp/sach-het.csv')

    expect(() => state.toggleBilingualImportPreviewChapterFilter()).not.toThrow()
    expect(state.bilingualImportPreviewChapterFilterActive.value).toBe(false)

    state.resetBilingualImportPreview()
  })

  it('dưới bốn Chương (any_signal_participated === false) — bộ lọc KHÔNG bật dù needs_review_count > 0', async () => {
    const state = await freshState()
    // Cùng ca thật "4 link 1 hỏng" của importPreviewChapters.test.ts: needs_review_count CỘNG
    // tự broken_item_count dù không tín hiệu nào tham gia (đường song ngữ luôn broken_item_count
    // === 0 — xem broken_item_count_is_always_zero_on_the_bilingual_path phía Rust — nên dựng
    // trực tiếp trạng thái "chưa đủ Chương để so" mà needs_review_count vẫn > 0 để canh ĐÚNG
    // vị từ, không suy từ đường link hỏng không tồn tại ở đây).
    const wire = chaptersWire({
      chapter_count: 3,
      needs_review_count: 1,
      clean_count: 2,
      any_signal_participated: false,
      already_chaptered: false,
      chapters: [chapterEntry({ ord: 1 }), chapterEntry({ ord: 2 }), chapterEntry({ ord: 3, needs_review: true })],
    })
    previewMock.mockResolvedValue({ preview: preview({ candidates: [candidate({ chapters: wire })] }), error: null })
    await state.openBilingualImportPreview('Ten', 'en', '', '/tmp/ba-chuong.csv')

    state.toggleBilingualImportPreviewChapterFilter()

    expect(state.bilingualImportPreviewChapterFilterActive.value).toBe(false)

    state.resetBilingualImportPreview()
  })

  // Cùng bất biến với `importPreviewChapters.test.ts`: đổi ứng
  // viên bảng mã trong lúc bộ lọc đang bật phải TỰ TẮT nếu ứng viên MỚI không đủ Chương để so.
  it('bật lọc rồi đổi sang ứng viên KHÔNG đủ Chương để so — bộ lọc TỰ TẮT', async () => {
    const state = await freshState()
    const participating = candidate({
      label: 'UTF-8',
      encoding: 'UTF-8',
      chapters: chaptersWire({
        chapter_count: 2,
        needs_review_count: 1,
        clean_count: 1,
        any_signal_participated: true,
        already_chaptered: false,
        chapters: [chapterEntry({ ord: 1, needs_review: false }), chapterEntry({ ord: 2, needs_review: true })],
      }),
    })
    const notParticipating = candidate({
      label: 'GBK',
      encoding: 'GBK',
      chapters: chaptersWire({ chapter_count: 2, needs_review_count: 0, clean_count: 2, any_signal_participated: false }),
    })
    previewMock.mockResolvedValue({ preview: preview({ candidates: [participating, notParticipating] }), error: null })
    await state.openBilingualImportPreview('Ten', 'en', '', '/tmp/hai-ung-vien.csv')

    state.toggleBilingualImportPreviewChapterFilter()
    expect(state.bilingualImportPreviewChapterFilterActive.value).toBe(true)

    state.selectBilingualEncoding('GBK')

    expect(state.bilingualImportPreviewChapterFilterActive.value).toBe(false)

    state.resetBilingualImportPreview()
  })

  it('mounted: bật lọc THẬT co danh sách về CHỈ Chương cần xem', async () => {
    const { state, BilingualImportPreviewOverlay } = await freshOverlay()
    const wire = chaptersWire({
      chapter_count: 4,
      needs_review_count: 2,
      clean_count: 2,
      any_signal_participated: true,
      already_chaptered: false,
      chapters: [
        chapterEntry({ ord: 1, needs_review: false }),
        chapterEntry({ ord: 2, needs_review: true, review_causes: ['short_length'] }),
        chapterEntry({ ord: 3, needs_review: false }),
        chapterEntry({ ord: 4, needs_review: true, review_causes: ['not_measured'] }),
      ],
    })
    previewMock.mockResolvedValue({ preview: preview({ candidates: [candidate({ chapters: wire })] }), error: null })
    await state.openBilingualImportPreview('Ten', 'en', '', '/tmp/bon-chuong.csv')

    const wrapper = mount(BilingualImportPreviewOverlay, { attachTo: document.body })

    expect(wrapper.findAll('.bip-chapters-entry')).toHaveLength(4)

    state.toggleBilingualImportPreviewChapterFilter()
    await wrapper.vm.$nextTick()

    const filtered = wrapper.findAll('.bip-chapters-entry')
    expect(filtered).toHaveLength(2)
    for (const row of filtered) {
      expect(row.find('.bip-chapters-needs-review-badge').exists()).toBe(true)
    }

    wrapper.unmount()
    state.resetBilingualImportPreview()
  })
})

// 🔴 Đối chứng đỏ — `⌥W` PHẢI so `event.code === 'KeyW'`, KHÔNG `event.key`: trên macOS `⌥W`
// gõ ra `∑`, nên mọi ca dưới đây dựng sự kiện với CẢ HAI trường (`code: 'KeyW'`, `key: '∑'`) —
// cùng khuôn `importPreviewOverlayRender.test.ts::"bộ lọc 'cần xem' DOM THẬT"`.
describe('BilingualImportPreviewOverlay.vue — bộ lọc "cần xem" DOM THẬT (`⌥W`)', () => {
  it('⌥W (event.code === KeyW, event.key === macOS ∑) bắn import.preview.bilingual_chapter_filter_toggle', async () => {
    const toggleMock = vi.fn()
    const { state, BilingualImportPreviewOverlay } = await freshOverlay({
      toggleBilingualImportPreviewChapterFilter: toggleMock,
    })
    previewMock.mockResolvedValue({ preview: preview({ candidates: [candidate({ chapters: chaptersWire() })] }), error: null })
    await state.openBilingualImportPreview('Ten', 'en', '', '/tmp/hai-cot.csv')

    const wrapper = mount(BilingualImportPreviewOverlay, { attachTo: document.body })
    const scrim = wrapper.get('.bip-screen')

    await scrim.trigger('keydown', { code: 'KeyW', key: '∑', altKey: true })
    expect(toggleMock).toHaveBeenCalledTimes(1)

    wrapper.unmount()
    state.resetBilingualImportPreview()
  })

  it('giữ ⌥W cho auto-repeat KHÔNG bắn một tràng lệnh — chỉ lần đầu', async () => {
    const toggleMock = vi.fn()
    const { state, BilingualImportPreviewOverlay } = await freshOverlay({
      toggleBilingualImportPreviewChapterFilter: toggleMock,
    })
    previewMock.mockResolvedValue({ preview: preview({ candidates: [candidate({ chapters: chaptersWire() })] }), error: null })
    await state.openBilingualImportPreview('Ten', 'en', '', '/tmp/hai-cot.csv')

    const wrapper = mount(BilingualImportPreviewOverlay, { attachTo: document.body })
    const scrim = wrapper.get('.bip-screen')

    await scrim.trigger('keydown', { code: 'KeyW', key: '∑', altKey: true })
    await scrim.trigger('keydown', { code: 'KeyW', key: '∑', altKey: true, repeat: true })
    await scrim.trigger('keydown', { code: 'KeyW', key: '∑', altKey: true, repeat: true })
    expect(toggleMock).toHaveBeenCalledTimes(1)

    wrapper.unmount()
    state.resetBilingualImportPreview()
  })

  it('`w` TRẦN (không `⌥`) KHÔNG bắn lệnh lọc', async () => {
    const toggleMock = vi.fn()
    const { state, BilingualImportPreviewOverlay } = await freshOverlay({
      toggleBilingualImportPreviewChapterFilter: toggleMock,
    })
    previewMock.mockResolvedValue({ preview: preview({ candidates: [candidate({ chapters: chaptersWire() })] }), error: null })
    await state.openBilingualImportPreview('Ten', 'en', '', '/tmp/hai-cot.csv')

    const wrapper = mount(BilingualImportPreviewOverlay, { attachTo: document.body })
    const scrim = wrapper.get('.bip-screen')

    await scrim.trigger('keydown', { code: 'KeyW', key: 'w' })
    expect(toggleMock).not.toHaveBeenCalled()

    wrapper.unmount()
    state.resetBilingualImportPreview()
  })

  // §Never spec 6.16b — `onMismatchKeydown` không được NỚI để chấp nhận Alt; ⌥ + một trong
  // bảy phím Story 6.17 vẫn phải KHÔNG bắn lệnh quy nhóm nào (nó từ chối MỌI hợp âm Alt ở
  // chính dòng đầu, `onScrimKeydown` chỉ TỔNG HỢP, không nới vị từ của handler kia).
  it('⌥↓ (Alt cộng một phím Story 6.17) KHÔNG bắn lệnh quy nhóm nào', async () => {
    const nextMock = vi.fn()
    const toggleMock = vi.fn()
    const { state, BilingualImportPreviewOverlay } = await freshOverlay({
      moveToNextBilingualMismatch: nextMock,
      toggleBilingualImportPreviewChapterFilter: toggleMock,
    })
    previewMock.mockResolvedValue({ preview: preview({ candidates: [candidate({ mismatches: [mismatch()] })] }), error: null })
    await state.openBilingualImportPreview('Ten', 'en', '', '/tmp/lech.csv')

    const wrapper = mount(BilingualImportPreviewOverlay, { attachTo: document.body })
    await wrapper.get('.bip-screen').trigger('keydown', { key: 'ArrowDown', code: 'ArrowDown', altKey: true })

    expect(nextMock).not.toHaveBeenCalled()

    wrapper.unmount()
    state.resetBilingualImportPreview()
  })

  it('lớp phủ ĐÃ ĐÓNG — `⌥W` không đổi gì (0 command nào bắn)', async () => {
    const toggleMock = vi.fn()
    const { state, BilingualImportPreviewOverlay } = await freshOverlay({
      toggleBilingualImportPreviewChapterFilter: toggleMock,
    })
    previewMock.mockResolvedValue({ preview: preview({ candidates: [candidate({ chapters: chaptersWire() })] }), error: null })
    await state.openBilingualImportPreview('Ten', 'en', '', '/tmp/hai-cot.csv')

    const wrapper = mount(BilingualImportPreviewOverlay, { attachTo: document.body })

    state.cancelBilingualImportPreview()
    await wrapper.vm.$nextTick()
    expect(wrapper.find('.bip-screen').exists()).toBe(false)

    expect(toggleMock).not.toHaveBeenCalled()

    wrapper.unmount()
  })
})

describe('BilingualImportPreviewOverlay.vue — "Nhập luôn, nối sau" và "Xem chỗ lệch"', () => {
  async function openWith(mismatches: BilingualMismatchWire[]) {
    const ctx = await freshOverlay()
    previewMock.mockResolvedValue({
      preview: preview({ candidates: [candidate({ mismatches })] }),
      error: null,
    })
    await ctx.state.openBilingualImportPreview('Ten', 'en', '', '/tmp/lech.csv')
    return ctx
  }

  it('còn hàng lệch có nguồn: "Nhập luôn" mở được, xác nhận thường khoá, "Xem N chỗ lệch" hiện đúng số', async () => {
    const { state, BilingualImportPreviewOverlay } = await openWith([mismatch(), mismatch({ row_number: 5 })])
    const wrapper = mount(BilingualImportPreviewOverlay, { attachTo: document.body })

    expect(wrapper.get('[data-bilingual-import-unlinked]').attributes('disabled')).toBeUndefined()
    expect(wrapper.get('.bip-act-primary').attributes('disabled')).toBeDefined()
    expect(wrapper.get('.bip-mismatch-focus, .bip-actions .bip-act').text()).toContain('2')

    wrapper.unmount()
    state.resetBilingualImportPreview()
  })

  it('còn hàng lệch NGUỒN RỖNG: "Nhập luôn" khoá kèm lý do có số hàng rỗng, và gọi hàm nhập luôn là no-op', async () => {
    const { state, BilingualImportPreviewOverlay } = await openWith([
      mismatch(),
      mismatch({ row_number: 7, source_sentences: [], proposed_cuts: [], candidate_positions: [] }),
    ])
    const wrapper = mount(BilingualImportPreviewOverlay, { attachTo: document.body })

    expect(wrapper.get('[data-bilingual-import-unlinked]').attributes('disabled')).toBeDefined()
    expect(wrapper.text()).toContain('Còn 1 hàng lệch không có câu nguồn')

    const result = await state.confirmBilingualImportPreview(true)
    expect(result).toEqual({ created: null, error: null })
    expect(confirmMock).not.toHaveBeenCalled()

    wrapper.unmount()
    state.resetBilingualImportPreview()
  })

  it('0 hàng lệch: nút "Nhập luôn" vắng mặt (không có gì để nối sau)', async () => {
    const { state, BilingualImportPreviewOverlay } = await openWith([])
    const wrapper = mount(BilingualImportPreviewOverlay, { attachTo: document.body })

    expect(wrapper.find('[data-bilingual-import-unlinked]').exists()).toBe(false)

    wrapper.unmount()
    state.resetBilingualImportPreview()
  })

  it('xác nhận thường gửi importMismatchedRows=false, nhập luôn gửi true (tham số cuối của lệnh)', async () => {
    const clean = await freshOverlay()
    previewMock.mockResolvedValue({ preview: preview(), error: null })
    confirmMock.mockResolvedValue({ created: null, error: null })
    await clean.state.openBilingualImportPreview('Ten', 'en', '', '/tmp/ok.csv')
    await clean.state.confirmBilingualImportPreview()
    expect(confirmMock.mock.calls.at(-1)?.at(-1)).toBe(false)
    clean.state.resetBilingualImportPreview()

    const lech = await openWith([mismatch()])
    confirmMock.mockResolvedValue({ created: null, error: null })
    await lech.state.confirmBilingualImportPreview(true)
    expect(confirmMock.mock.calls.at(-1)?.at(-1)).toBe(true)
    lech.state.resetBilingualImportPreview()
  })

  it('xác nhận thường khi còn hàng lệch KHÔNG gọi Rust (cờ vắng ⇒ từ chối ở tầng giao diện)', async () => {
    const { state } = await openWith([mismatch()])
    await state.confirmBilingualImportPreview(false)
    expect(confirmMock).not.toHaveBeenCalled()
    state.resetBilingualImportPreview()
  })

  it('"Xem N chỗ lệch rồi nhập" chạy lệnh THẬT import.preview.bilingual_focus_mismatch: hàng lệch đầu thành hàng đang chọn, KHÔNG nhập', async () => {
    const { state, BilingualImportPreviewOverlay } = await freshOverlay({
      focusFirstBilingualMismatch: () => {
        state.focusFirstBilingualMismatch()
      },
    })
    previewMock.mockResolvedValue({
      preview: preview({ candidates: [candidate({ mismatches: [mismatch({ row_number: 3 }), mismatch({ row_number: 9 })] })] }),
      error: null,
    })
    await state.openBilingualImportPreview('Ten', 'en', '', '/tmp/lech.csv')
    const wrapper = mount(BilingualImportPreviewOverlay, { attachTo: document.body })
    state.moveToNextBilingualMismatch()
    expect(state.bilingualImportPreviewActiveMismatch.value?.row_number).toBe(9)

    const viewBtn = wrapper.findAll('.bip-actions .bip-act').find((b) => b.text().startsWith('Xem 2'))
    expect(viewBtn).toBeDefined()
    await viewBtn!.trigger('click')

    expect(state.bilingualImportPreviewActiveMismatch.value?.row_number).toBe(3)
    expect(confirmMock).not.toHaveBeenCalled()

    wrapper.unmount()
    state.resetBilingualImportPreview()
  })

  it('màn song ngữ không có dòng ước lượng Glossary và không có scrim', async () => {
    const { state, BilingualImportPreviewOverlay } = await openWith([])
    const wrapper = mount(BilingualImportPreviewOverlay, { attachTo: document.body })

    expect(wrapper.find('[data-ip-glossary-estimate]').exists()).toBe(false)
    expect(wrapper.text()).not.toContain('Ứng viên Glossary')
    expect(wrapper.find('.bip-scrim').exists()).toBe(false)
    expect(wrapper.find('[aria-modal]').exists()).toBe(false)

    wrapper.unmount()
    state.resetBilingualImportPreview()
  })
})

function spaced(over: Partial<BilingualMismatchWire> = {}): BilingualMismatchWire {
  return mismatch({
    row_number: 5,
    source_sentences: ['One.', 'Two.'],
    target_line: 'Mot hai',
    candidate_positions: [3, 4],
    proposed_cuts: [3],
    ...over,
  })
}

function rebuildReturns(rows: BilingualMismatchWire[]): void {
  rebuildMock.mockResolvedValue({
    preview: preview({ candidates: [candidate({ mismatches: rows })] }),
    error: null,
  })
}

async function mountCutOverlay(rows: BilingualMismatchWire[], deps: Partial<CommandDeps> = {}, real = false) {
  const { commands, state, BilingualImportPreviewOverlay } = await freshOverlay(deps)
  if (real) {
    deps.toggleBilingualCutAtCaret = state.toggleBilingualCutAtCaret
    deps.moveToNextBilingualMismatch = state.moveToNextBilingualMismatch
    deps.moveToPreviousBilingualMismatch = state.moveToPreviousBilingualMismatch
    deps.moveBilingualCaretLeft = state.moveBilingualCaretLeft
    deps.moveBilingualCaretRight = state.moveBilingualCaretRight
  }
  previewMock.mockResolvedValue({ preview: preview({ candidates: [candidate({ mismatches: rows })] }), error: null })
  await state.openBilingualImportPreview('Ten', 'en', '', '/tmp/lech.csv')
  const wrapper = mount(BilingualImportPreviewOverlay, { attachTo: document.body })
  return { commands, state, wrapper }
}

async function settle(wrapper: { vm: { $nextTick: () => Promise<void> } }): Promise<void> {
  await new Promise((r) => setTimeout(r))
  await wrapper.vm.$nextTick()
}

describe('BilingualImportPreviewOverlay.vue — đích bấm trên dòng đích (14-7)', () => {
  it('mỗi candidate_positions một đích, hai đầu dòng không có đích, và chỗ dừng không đổi khi caret đổi', async () => {
    const { state, wrapper } = await mountCutOverlay([spaced()])
    const stopCountBefore = wrapper.findAll('.bip-cut-stop').length
    expect(wrapper.findAll('.bip-cut-target')).toHaveLength(2)
    expect(wrapper.findAll('.bip-cut-end')).toHaveLength(2)
    state.moveBilingualCaretRight()
    await wrapper.vm.$nextTick()
    expect(wrapper.findAll('.bip-cut-stop')).toHaveLength(stopCountBefore)
    expect(wrapper.findAll('.bip-cut-aimed')).toHaveLength(1)
    wrapper.unmount()
    state.resetBilingualImportPreview()
  })

  it('mousedown trái trên đích nhắm caret, ← → đi tiếp từ đó, 0 IPC', async () => {
    const { state, wrapper } = await mountCutOverlay([spaced()])
    await wrapper.findAll('.bip-cut-target')[1]!.trigger('mousedown', { button: 0 })
    expect(state.bilingualImportPreviewCaretPosition.value).toBe(4)
    state.moveBilingualCaretLeft()
    expect(state.bilingualImportPreviewCaretPosition.value).toBe(3)
    expect(rebuildMock).not.toHaveBeenCalled()
    wrapper.unmount()
    state.resetBilingualImportPreview()
  })

  it('nút khác nút trái không đổi caret', async () => {
    const { state, wrapper } = await mountCutOverlay([spaced()])
    await wrapper.findAll('.bip-cut-target')[1]!.trigger('mousedown', { button: 2 })
    expect(state.bilingualImportPreviewCaretPosition.value).toBe(0)
    expect(rebuildMock).not.toHaveBeenCalled()
    wrapper.unmount()
    state.resetBilingualImportPreview()
  })

  it('tiêu điểm đang ở ô nhập mẫu Chương thì cú nhấn trên đích đưa nó về panel', async () => {
    const { wrapper } = await mountCutOverlay([spaced()])
    const input = wrapper.get<HTMLInputElement>('#bip-chapter-pattern-text')
    input.element.focus()
    expect(document.activeElement).toBe(input.element)
    await wrapper.findAll('.bip-cut-target')[0]!.trigger('mousedown', { button: 0 })
    expect(document.activeElement).toBe(wrapper.get('.bip-panel').element)
    wrapper.unmount()
  })

  it('(i) bấm đích bắn bilingual_toggle_cut', async () => {
    const toggle = vi.fn()
    const { state, wrapper } = await mountCutOverlay([spaced()], { toggleBilingualCutAtCaret: toggle })
    await wrapper.findAll('.bip-cut-target')[0]!.trigger('click')
    expect(toggle).toHaveBeenCalledTimes(1)
    wrapper.unmount()
    state.resetBilingualImportPreview()
  })

  it('(i) Cặp lệch trước/sau bắn hai lệnh và chỉ hiện khi có hàng khác để tới', async () => {
    const prev = vi.fn()
    const next = vi.fn()
    const { state, wrapper } = await mountCutOverlay([spaced({ row_number: 5 }), spaced({ row_number: 8 })], {
      moveToPreviousBilingualMismatch: prev,
      moveToNextBilingualMismatch: next,
    })
    const buttons = wrapper.findAll('.bip-mismatch-heading-row button')
    expect(buttons.map((b) => b.text())).toEqual(['Cặp lệch trước', 'Cặp lệch sau'])
    await buttons[0]!.trigger('click')
    await buttons[1]!.trigger('click')
    expect(prev).toHaveBeenCalledTimes(1)
    expect(next).toHaveBeenCalledTimes(1)
    wrapper.unmount()
    state.resetBilingualImportPreview()
  })

  it('một cặp lệch duy nhất không có hai nút', async () => {
    const { state, wrapper } = await mountCutOverlay([spaced()])
    expect(wrapper.findAll('.bip-mismatch-heading-row button')).toHaveLength(0)
    wrapper.unmount()
    state.resetBilingualImportPreview()
  })

  it('hàng không cắt được (một phía trống) không có dòng đích, không có đích', async () => {
    const { state, wrapper } = await mountCutOverlay([
      spaced({ source_sentences: [], target_line: 'Mot hai', candidate_positions: [3, 4] }),
    ])
    expect(wrapper.find('.bip-mismatch-caret-line').exists()).toBe(false)
    expect(wrapper.findAll('.bip-cut-target')).toHaveLength(0)
    wrapper.unmount()
    state.resetBilingualImportPreview()
  })
})

describe('bilingualImportPreviewState — hàng vừa khớp bằng chuột ở lại (14-7)', () => {
  it('cắt làm hàng vừa khớp: hàng ở lại, đã khớp, nhập được; bấm lại đưa hàng về lệch với caret giữ nguyên', async () => {
    const { state, wrapper } = await mountCutOverlay([spaced()], {}, true)
    rebuildReturns([])
    await wrapper.findAll('.bip-cut-target')[0]!.trigger('mousedown', { button: 0 })
    await wrapper.findAll('.bip-cut-target')[0]!.trigger('click')
    await settle(wrapper)

    expect(state.bilingualImportPreviewMismatches.value).toHaveLength(0)
    expect(state.bilingualImportPreviewActiveMismatchHeld.value).toBe(true)
    expect(wrapper.find('.bip-mismatch-focus').exists()).toBe(true)
    expect(wrapper.get('.bip-mismatch-heading').text()).toContain('Đã khớp')
    const target = wrapper.findAll('.bip-cut-target')[0]!
    expect(target.text()).toBe('‖')
    expect(target.attributes('aria-pressed')).toBe('true')
    expect(wrapper.findAll('.bip-sentence-list')[1]!.findAll('li').map((li) => li.text())).toEqual(['Mot', 'hai'])
    expect(state.bilingualImportPreviewCanConfirm.value).toBe(true)
    expect(wrapper.findAll('.bip-mismatch-heading-row button')).toHaveLength(0)

    rebuildReturns([spaced()])
    await wrapper.findAll('.bip-cut-target')[0]!.trigger('mousedown', { button: 0 })
    await wrapper.findAll('.bip-cut-target')[0]!.trigger('click')
    await settle(wrapper)
    expect(state.bilingualImportPreviewActiveMismatchHeld.value).toBe(false)
    expect(state.bilingualImportPreviewMismatches.value).toHaveLength(1)
    expect(state.bilingualImportPreviewActiveMismatch.value?.row_number).toBe(5)
    expect(state.bilingualImportPreviewCaretPosition.value).toBe(3)
    expect(wrapper.get('.bip-mismatch-heading').text()).toContain('Chỗ lệch 1 / 1')
    wrapper.unmount()
    state.resetBilingualImportPreview()
  })

  it('cắt vẫn còn lệch thì không giữ, và hàng đổi mảnh ngay', async () => {
    const { state, wrapper } = await mountCutOverlay([spaced({ source_sentences: ['A.', 'B.', 'C.'] })], {}, true)
    rebuildReturns([spaced({ source_sentences: ['A.', 'B.', 'C.'], initial_cuts: [3] })])
    await wrapper.findAll('.bip-cut-target')[0]!.trigger('mousedown', { button: 0 })
    await wrapper.findAll('.bip-cut-target')[0]!.trigger('click')
    await settle(wrapper)
    expect(state.bilingualImportPreviewActiveMismatchHeld.value).toBe(false)
    expect(state.bilingualImportPreviewActiveCuts.value).toEqual([3])
    expect(wrapper.findAll('.bip-cut-target')[0]!.text()).toBe('‖')
    wrapper.unmount()
    state.resetBilingualImportPreview()
  })

  it('cắt còn lệch rồi bấm lại: cột mảnh đổi theo và chỗ cắt mất', async () => {
    const row = spaced({ source_sentences: ['A.', 'B.', 'C.'] })
    const { state, wrapper } = await mountCutOverlay([row], {}, true)
    const pieces = () => wrapper.findAll('.bip-sentence-list')[1]!.findAll('li').map((li) => li.text())
    expect(pieces()).toEqual(['Mot hai'])
    rebuildReturns([row])
    await wrapper.findAll('.bip-cut-target')[0]!.trigger('mousedown', { button: 0 })
    await wrapper.findAll('.bip-cut-target')[0]!.trigger('click')
    await settle(wrapper)
    expect(pieces()).toEqual(['Mot', 'hai'])

    await wrapper.findAll('.bip-cut-target')[0]!.trigger('mousedown', { button: 0 })
    await wrapper.findAll('.bip-cut-target')[0]!.trigger('click')
    await settle(wrapper)
    expect(pieces()).toEqual(['Mot hai'])
    expect(wrapper.findAll('.bip-cut-target')[0]!.text()).toBe('¦')
    wrapper.unmount()
    state.resetBilingualImportPreview()
  })

  it('hàng giữ khi hết lệch: ← → đi trên hàng đó, Esc huỷ, Áp mọi đề xuất tắt', async () => {
    const cancel = vi.fn()
    const { state, wrapper } = await mountCutOverlay([spaced()], { cancelBilingualImportPreview: cancel }, true)
    rebuildReturns([])
    await wrapper.findAll('.bip-cut-target')[1]!.trigger('mousedown', { button: 0 })
    await state.toggleBilingualCutAtCaret()
    await wrapper.vm.$nextTick()
    expect(state.bilingualImportPreviewActiveMismatchHeld.value).toBe(true)

    const panel = wrapper.get('.bip-panel')
    expect(document.activeElement).toBe(panel.element)
    await panel.trigger('keydown', { key: 'ArrowLeft' })
    expect(state.bilingualImportPreviewCaretPosition.value).toBe(3)
    await panel.trigger('keydown', { key: 'ArrowRight' })
    expect(state.bilingualImportPreviewCaretPosition.value).toBe(4)
    expect(state.bilingualImportPreviewActiveMismatchHeld.value).toBe(true)
    expect(wrapper.get('.bip-mismatch-action').attributes('disabled')).toBeDefined()

    await panel.trigger('keydown', { key: 'Escape' })
    expect(cancel).toHaveBeenCalledTimes(1)
    wrapper.unmount()
    state.resetBilingualImportPreview()
  })

  it('bấm đích thứ hai khi lượt rebuild đầu chưa về: lượt mới thắng, hàng còn đó', async () => {
    const row = spaced()
    const { state, wrapper } = await mountCutOverlay([row])
    let releaseFirst: (v: unknown) => void = () => {}
    rebuildMock.mockReturnValueOnce(new Promise((r) => (releaseFirst = r)))
    state.aimBilingualCaret(3)
    const first = state.toggleBilingualCutAtCaret()

    rebuildReturns([row])
    state.aimBilingualCaret(4)
    await state.toggleBilingualCutAtCaret()
    releaseFirst({ preview: preview({ candidates: [candidate({ mismatches: [] })] }), error: null })
    await first

    expect(state.bilingualImportPreviewMismatches.value).toHaveLength(1)
    expect(state.bilingualImportPreviewActiveMismatchHeld.value).toBe(false)
    expect(state.bilingualImportPreviewActiveCuts.value).toEqual([3, 4])
    wrapper.unmount()
    state.resetBilingualImportPreview()
  })

  it('rời sang hàng khác khi rebuild đang chạy: hàng vừa khớp không kéo người dùng quay lại', async () => {
    const rows = [spaced({ row_number: 5 }), spaced({ row_number: 8 })]
    const { state, wrapper } = await mountCutOverlay(rows)
    let release: (v: unknown) => void = () => {}
    rebuildMock.mockReturnValueOnce(new Promise((r) => (release = r)))
    state.aimBilingualCaret(3)
    const pending = state.toggleBilingualCutAtCaret()
    state.moveToNextBilingualMismatch()
    release({ preview: preview({ candidates: [candidate({ mismatches: [rows[1]!] })] }), error: null })
    await pending

    expect(state.bilingualImportPreviewActiveMismatch.value?.row_number).toBe(8)
    expect(state.bilingualImportPreviewActiveMismatchHeld.value).toBe(false)
    wrapper.unmount()
    state.resetBilingualImportPreview()
  })

  it('đường phím (Enter) làm hàng khớp thì hàng biến mất, không giữ', async () => {
    const { state, wrapper } = await mountCutOverlay([spaced({ row_number: 5 }), spaced({ row_number: 8 })])
    rebuildReturns([spaced({ row_number: 8 })])
    state.moveBilingualCaretRight()
    await state.toggleBilingualCutAtCaret()
    expect(state.bilingualImportPreviewActiveMismatchHeld.value).toBe(false)
    expect(state.bilingualImportPreviewActiveMismatch.value?.row_number).toBe(8)
    wrapper.unmount()
    state.resetBilingualImportPreview()
  })

  it('rời hàng giữ bằng Cặp lệch sau/trước đi theo row_number và hàng giữ không quay lại', async () => {
    const rows = [spaced({ row_number: 2 }), spaced({ row_number: 5 }), spaced({ row_number: 8 })]
    const { state, wrapper } = await mountCutOverlay(rows)
    state.moveToNextBilingualMismatch()
    rebuildReturns([rows[0], rows[2]])
    state.aimBilingualCaret(3)
    await state.toggleBilingualCutAtCaret()
    expect(state.bilingualImportPreviewActiveMismatchHeld.value).toBe(true)
    expect(wrapper.findAll('.bip-mismatch-heading-row button')).toHaveLength(2)

    state.moveToNextBilingualMismatch()
    expect(state.bilingualImportPreviewActiveMismatch.value?.row_number).toBe(8)
    expect(state.bilingualImportPreviewActiveMismatchHeld.value).toBe(false)

    state.moveToPreviousBilingualMismatch()
    expect(state.bilingualImportPreviewActiveMismatch.value?.row_number).toBe(2)
    wrapper.unmount()
    state.resetBilingualImportPreview()
  })

  it('rời hàng giữ về phía trước bắt đầu từ hàng đầu, không bỏ sót nó', async () => {
    const rows = [spaced({ row_number: 2 }), spaced({ row_number: 5 })]
    const { state, wrapper } = await mountCutOverlay(rows)
    state.moveToNextBilingualMismatch()
    rebuildReturns([rows[0]])
    state.aimBilingualCaret(3)
    await state.toggleBilingualCutAtCaret()
    state.moveToPreviousBilingualMismatch()
    expect(state.bilingualImportPreviewActiveMismatch.value?.row_number).toBe(2)
    wrapper.unmount()
    state.resetBilingualImportPreview()
  })

  it('đổi cột nguồn khi đang giữ thì hết giữ', async () => {
    const { state, wrapper } = await mountCutOverlay([spaced()])
    rebuildReturns([])
    state.aimBilingualCaret(3)
    await state.toggleBilingualCutAtCaret()
    expect(state.bilingualImportPreviewActiveMismatchHeld.value).toBe(true)
    await state.setBilingualSourceColumn(1)
    expect(state.bilingualImportPreviewActiveMismatchHeld.value).toBe(false)
    await wrapper.vm.$nextTick()
    expect(wrapper.find('.bip-mismatch-focus').exists()).toBe(false)
    wrapper.unmount()
    state.resetBilingualImportPreview()
  })

  it('xác nhận khi đang giữ gửi đúng quy nhóm đã cắt', async () => {
    const { state, wrapper } = await mountCutOverlay([spaced()])
    rebuildReturns([])
    state.aimBilingualCaret(3)
    await state.toggleBilingualCutAtCaret()
    confirmMock.mockResolvedValue({ created: { meta: { work_id: 'w1' } }, error: null })
    await state.confirmBilingualImportPreview()
    const regroupings = confirmMock.mock.calls[0]![8] as BilingualRegroupingInput[]
    expect(regroupings).toHaveLength(1)
    expect(regroupings[0]).toMatchObject({ row_number: 5, kind: 'cuts', cuts: [3] })
    wrapper.unmount()
  })

  it('đang xác nhận: đích và hai nút tắt, mousedown không nhắm', async () => {
    const { state, wrapper } = await mountCutOverlay([spaced({ row_number: 5 }), spaced({ row_number: 8 })])
    let release: (v: unknown) => void = () => {}
    confirmMock.mockReturnValue(new Promise((r) => (release = r)))
    rebuildReturns([])
    const pending = state.confirmBilingualImportPreview(true)
    await wrapper.vm.$nextTick()
    expect(state.bilingualImportPreviewConfirming.value).toBe(true)
    expect(wrapper.findAll('.bip-cut-target').every((b) => b.attributes('disabled') !== undefined)).toBe(true)
    expect(wrapper.findAll('.bip-mismatch-heading-row button').every((b) => b.attributes('disabled') !== undefined)).toBe(true)
    state.aimBilingualCaret(4)
    expect(state.bilingualImportPreviewCaretPosition.value).toBe(0)
    const target = wrapper.findAll('.bip-cut-target')[0]!
    await target.trigger('mousedown', { button: 0 })
    await target.trigger('click')
    expect(rebuildMock).not.toHaveBeenCalled()
    release({ created: null, error: null })
    await pending
    wrapper.unmount()
    state.resetBilingualImportPreview()
  })

  it('nhấn không thành cú bấm rồi Enter thật không giữ hàng', async () => {
    const { state, wrapper } = await mountCutOverlay([spaced()], {}, true)
    rebuildReturns([])
    await wrapper.findAll('.bip-cut-target')[0]!.trigger('mousedown', { button: 0 })
    await wrapper.get('.bip-screen').trigger('keydown', { key: 'Enter' })
    await settle(wrapper)
    expect(rebuildMock).toHaveBeenCalledTimes(1)
    expect(state.bilingualImportPreviewActiveMismatchHeld.value).toBe(false)
    expect(wrapper.find('.bip-mismatch-focus').exists()).toBe(false)
    wrapper.unmount()
    state.resetBilingualImportPreview()
  })

  it('mousedown, mouseleave rồi click vẫn giữ hàng', async () => {
    const { state, wrapper } = await mountCutOverlay([spaced()], {}, true)
    rebuildReturns([])
    const target = wrapper.findAll('.bip-cut-target')[0]!
    await target.trigger('mousedown', { button: 0 })
    await target.trigger('mouseleave')
    await target.trigger('click')
    await settle(wrapper)
    expect(state.bilingualImportPreviewActiveMismatchHeld.value).toBe(true)
    wrapper.unmount()
    state.resetBilingualImportPreview()
  })

  it('rời hàng giữ bằng đổi bảng mã đưa caret về 0', async () => {
    const { state, wrapper } = await mountCutOverlay([spaced()], {}, true)
    const two = preview({
      candidates: [candidate({ mismatches: [] }), candidate({ label: 'GBK', encoding: 'GBK', mismatches: [spaced()] })],
    })
    rebuildMock.mockResolvedValue({ preview: two, error: null })
    state.aimBilingualCaret(3)
    await state.toggleBilingualCutAtCaret()
    expect(state.bilingualImportPreviewActiveMismatchHeld.value).toBe(true)
    expect(state.bilingualImportPreviewCaretPosition.value).toBe(3)
    state.selectBilingualEncoding('GBK')
    expect(state.bilingualImportPreviewActiveMismatchHeld.value).toBe(false)
    expect(state.bilingualImportPreviewCaretPosition.value).toBe(0)
    wrapper.unmount()
    state.resetBilingualImportPreview()
  })

  it('Cặp lệch sau ở hàng cuối vòng về hàng đầu, caret 0, → chạy tiếp', async () => {
    const { state, wrapper } = await mountCutOverlay([spaced({ row_number: 5 }), spaced({ row_number: 8 })], {}, true)
    state.moveToNextBilingualMismatch()
    state.aimBilingualCaret(4)
    expect(state.bilingualImportPreviewCaretPosition.value).toBe(4)
    const next = wrapper.findAll('.bip-mismatch-heading-row button')[1]!
    await next.trigger('click')
    expect(state.bilingualImportPreviewActiveMismatch.value?.row_number).toBe(5)
    expect(state.bilingualImportPreviewCaretPosition.value).toBe(0)
    await wrapper.get('.bip-screen').trigger('keydown', { key: 'ArrowRight' })
    expect(state.bilingualImportPreviewCaretPosition.value).toBe(3)
    wrapper.unmount()
    state.resetBilingualImportPreview()
  })

  it('đích giữ nguyên phần tử DOM qua lần vẽ lại khi nhắm', async () => {
    const { state, wrapper } = await mountCutOverlay([spaced()])
    const before = wrapper.findAll('.bip-cut-target')[1]!.element
    await wrapper.findAll('.bip-cut-target')[0]!.trigger('mousedown', { button: 0 })
    await wrapper.vm.$nextTick()
    expect(wrapper.findAll('.bip-cut-target')[1]!.element).toBe(before)
    expect(wrapper.findAll('.bip-cut-aimed')).toHaveLength(1)
    wrapper.unmount()
    state.resetBilingualImportPreview()
  })
})
