/**
 * Tầng 2 — bóc nội dung chính và sửa ranh giới bằng bàn phím (Story 6.9, FR123).
 *
 * ─────────────────────────────────────────────────────────────────────────────
 * HAI LỚP, HAI TỆP MỘT MỐI QUAN TÂM — khuôn `glossaryQueue.test.ts`
 * ─────────────────────────────────────────────────────────────────────────────
 * 1. Lớp STATE (`importPreviewState.ts`) — gọi thẳng sáu hàm, giả lập `config/project.ts`
 *    (biên IPC), khẳng định LOGIC: J/K kẹp biên không vòng qua đầu, `Space` gọi
 *    `tier2BlockSetKept` với đúng `(index, !kept, sourceLang)`, `[`/`]` gọi
 *    `tier2BlockConfirmRange` với đúng `(start, end, total, sourceLang)`, `]` trước `[` KÊU
 *    không NÉM (0 lời gọi IPC).
 * 2. Lớp BÀN PHÍM (`ImportPreviewOverlay.vue`) — cài BỘ COMMAND THẬT (`installCommands`),
 *    mount component thật, bắn `keydown` THẬT trên `.ip-screen`, khẳng định handler DOM cục bộ
 *    gọi ĐÚNG `dispatch('<id>')` → `CommandRegistry` thật → `deps.xxx()` — không một đường tắt
 *    tự dựng trong test. Cộng đối chứng AC "tiêu điểm ở NÚT thì `Space` không bị cướp".
 */
import { afterEach, beforeAll, beforeEach, describe, expect, it, vi } from 'vitest'
import { mount } from '@vue/test-utils'
import { nextTick } from 'vue'
import type { CommandDeps } from '../../src/commands'
import type {
  ChapterOriginWire,
  BlockWire,
  ChapterBlocksPreviewWire,
  ImportEncodingPreview,
  UrlImportBatchWire,
  UrlImportItemWire,
} from '../../src/config/project'

/** Story 6.15 — bốn trường xuất xứ RỖNG, dùng làm giá trị mặc định cho mọi fixture
 * `ChapterSplitPreviewEntryWire` ở tệp này (không ca nào trong tệp cần một giá trị khác
 * rỗng — các ca đọc `origin` sống ở `importPreviewChapterOrigin.test.ts`). */
const ORIGIN_STUB: ChapterOriginWire = {
  author: null,
  site_name: null,
  url: null,
  published_at: null,
  author_confirmed: false,
  site_name_confirmed: false,
  url_confirmed: false,
  published_at_confirmed: false,
}


const previewTextMock = vi.fn()
const previewFileMock = vi.fn()
const confirmMock = vi.fn()
const startUrlImportMock = vi.fn()
const reloadUrlImportItemMock = vi.fn()
const removeUrlImportItemMock = vi.fn()
const tier2BlockSetKeptMock = vi.fn()
const tier2BlockConfirmRangeMock = vi.fn()
const previewChapterDetailMock = vi.fn()

vi.mock('../../src/config/project', async (importOriginal) => {
  const actual = await importOriginal<typeof import('../../src/config/project')>()
  return {
    ...actual,
    previewImportEncodingFromText: (text: string, sourceLang: string) => previewTextMock(text, sourceLang),
    previewImportEncodingFromFile: (path: string, sourceLang: string) => previewFileMock(path, sourceLang),
    confirmImportWithEncoding: (name: string, sourceLang: string, genre: string, encoding: string) =>
      confirmMock(name, sourceLang, genre, encoding),
    startUrlImport: (urls: string[], sourceLang: string) => startUrlImportMock(urls, sourceLang),
    reloadUrlImportItem: (index: number, sourceLang: string) => reloadUrlImportItemMock(index, sourceLang),
    removeUrlImportItem: (index: number, sourceLang: string) => removeUrlImportItemMock(index, sourceLang),
    previewChapterDetail: (...args: unknown[]) => previewChapterDetailMock(...args),
    tier2BlockSetKept: (index: number, kept: boolean, sourceLang: string, detailChapterIndex: number) =>
      tier2BlockSetKeptMock(index, kept, sourceLang, detailChapterIndex),
    tier2BlockConfirmRange: (start: number, end: number, total: number, sourceLang: string, detailChapterIndex: number) =>
      tier2BlockConfirmRangeMock(start, end, total, sourceLang, detailChapterIndex),
  }
})

function resetAllMocks(): void {
  previewTextMock.mockReset()
  previewFileMock.mockReset()
  confirmMock.mockReset()
  startUrlImportMock.mockReset()
  reloadUrlImportItemMock.mockReset()
  removeUrlImportItemMock.mockReset()
  tier2BlockSetKeptMock.mockReset()
  tier2BlockConfirmRangeMock.mockReset()
  previewChapterDetailMock.mockReset()
}

async function freshState() {
  vi.resetModules()
  resetAllMocks()
  return import('../../src/importPreviewState')
}

async function freshOverlay(deps: Partial<CommandDeps> = {}) {
  vi.resetModules()
  resetAllMocks()
  const commands = await import('../../src/commands')
  commands.installCommands(deps as CommandDeps)
  const state = await import('../../src/importPreviewState')
  const ImportPreviewOverlay = (await import('../../src/ImportPreviewOverlay.vue')).default
  return { commands, state, ImportPreviewOverlay }
}

function item(url: string): UrlImportItemWire {
  return { url, ok: true, error: null }
}

function block(over: Partial<BlockWire> = {}): BlockWire {
  return { body: { kind: 'paragraph', text: 'noi dung khoi' }, kept: true, confirmed: false, cleanup_spans: [], ...over }
}

function blocksWire(blocks: BlockWire[]): ChapterBlocksPreviewWire {
  return { blocks }
}

/** Xem trước tối giản, ĐÚNG một ứng viên mang `blocks` — đủ cho các ca xoáy vào tầng 2. */
function previewWithBlocks(blocks: BlockWire[]): ImportEncodingPreview {
  return {
    source_lang: 'en',
    source_lang_auto: false,
    confidence: 'self_declared',
    selected_encoding: 'UTF-8',
    candidates: [
      {
        label: 'UTF-8',
        encoding: 'UTF-8',
        preview: 'noi dung',
        normalized: { text: 'noi dung', joined_lines: 0, blank_lines_removed: 0, window_truncated: false },
        cleanup: { text: 'noi dung', spans: [], rules: [], window_truncated: false, final_text: 'noi dung' },
        chapters: {
          chapter_count: 1,
          chapters: [{ ord: 1, title: null, length: 8, cleanup_match_count: 0, joined_line_count_in_chapter: null, needs_review: false, review_causes: [], chapter_origin: ORIGIN_STUB, source_file: null }],
          broken_item_count: 0,
          needs_review_count: 0,
          clean_count: 1,
          any_signal_participated: false,
          already_chaptered: false,
        },
        blocks: blocksWire(blocks),
      },
    ],
    self_declared_normalized: null,
    self_declared_cleanup: null,
    self_declared_chapters: null,
  }
}

function batchWithBlocks(urls: string[], blocks: BlockWire[]): UrlImportBatchWire {
  return {
    items: urls.map(item),
    encoding_preview: previewWithBlocks(blocks),
    domain_log_domain_count: urls.length,
    duplicate_urls_dropped: 0,
  }
}

const THREE_BLOCKS: BlockWire[] = [
  block({ body: { kind: 'paragraph', text: 'khung dieu huong' }, kept: false, confirmed: false }),
  block({ body: { kind: 'paragraph', text: 'than bai may doan' }, kept: true, confirmed: false }),
  block({ body: { kind: 'caption', text: 'chu thich da xac nhan' }, kept: true, confirmed: true }),
]

/** Fixtures start with a dropped block 0, which the overlay shows as a collapsed band:
 * open it, then step the cursor onto block 0 so `.ip-block` indices equal wire indices. */
async function openLeadingBand(state: {
  toggleImportPreviewBlockKept: () => Promise<void>
  nextImportPreviewBlock: () => void
}): Promise<void> {
  await state.toggleImportPreviewBlockKept()
  state.nextImportPreviewBlock()
}

beforeAll(async () => {
  await freshOverlay()
})

beforeEach(() => {
  document.body.innerHTML = ''
})

afterEach(() => {
  document.body.innerHTML = ''
})

// ═════════════════════════════════════════════════════════════════════════════════
// Lớp STATE — logic của sáu hàm, KHÔNG qua DOM
// ═════════════════════════════════════════════════════════════════════════════════

describe('importPreviewState — điều hướng J/K kẹp biên, không vòng qua đầu', () => {
  it('next dừng ở khối cuối, prev dừng ở khối đầu', async () => {
    const state = await freshState()
    const urls = ['https://a.example/1']
    startUrlImportMock.mockResolvedValue({ batch: batchWithBlocks(urls, THREE_BLOCKS), error: null })
    await state.openImportPreviewFromUrls('Ten', 'en', '', urls, null)

    expect(state.importPreviewSelectedBlocks.value?.blocks).toHaveLength(3)
    expect(state.importPreviewBlockFocusedIndex.value).toBe(0)

    state.nextImportPreviewBlock()
    expect(state.importPreviewBlockFocusedIndex.value).toBe(1)
    state.nextImportPreviewBlock()
    expect(state.importPreviewBlockFocusedIndex.value).toBe(2)
    state.nextImportPreviewBlock() // đã ở cuối — dừng, không vòng về 0
    expect(state.importPreviewBlockFocusedIndex.value).toBe(2)

    state.prevImportPreviewBlock()
    state.prevImportPreviewBlock()
    expect(state.importPreviewBlockFocusedIndex.value).toBe(0)
    state.prevImportPreviewBlock() // đã ở đầu — dừng, không âm
    expect(state.importPreviewBlockFocusedIndex.value).toBe(0)
  })
})

describe('importPreviewState — Space gọi tier2BlockSetKept với ĐÚNG tham số', () => {
  it('đảo khối ĐANG CHỌN (index 0, kept=false) thành kept=true, gửi đúng (0, true, sourceLang)', async () => {
    const state = await freshState()
    const urls = ['https://a.example/1']
    startUrlImportMock.mockResolvedValue({ batch: batchWithBlocks(urls, THREE_BLOCKS), error: null })
    await state.openImportPreviewFromUrls('Ten', 'en', '', urls, null)
    await openLeadingBand(state)

    const flippedBlocks = [
      block({ body: { kind: 'paragraph', text: 'khung dieu huong' }, kept: true, confirmed: true }),
      THREE_BLOCKS[1]!,
      THREE_BLOCKS[2]!,
    ]
    tier2BlockSetKeptMock.mockResolvedValue({ batch: batchWithBlocks(urls, flippedBlocks), error: null })

    await state.toggleImportPreviewBlockKept()

    expect(tier2BlockSetKeptMock).toHaveBeenCalledTimes(1)
    expect(tier2BlockSetKeptMock).toHaveBeenCalledWith(0, true, 'en', 0)
    expect(state.importPreviewSelectedBlocks.value?.blocks[0]?.kept).toBe(true)
    expect(state.importPreviewSelectedBlocks.value?.blocks[0]?.confirmed).toBe(true)
  })

  it('lỗi IPC đi vào importPreviewBlockActionError, KHÔNG áp batch nào', async () => {
    const state = await freshState()
    const urls = ['https://a.example/1']
    startUrlImportMock.mockResolvedValue({ batch: batchWithBlocks(urls, THREE_BLOCKS), error: null })
    await state.openImportPreviewFromUrls('Ten', 'en', '', urls, null)

    await openLeadingBand(state)
    const err = { code: 'ipc.unknown', message_key: 'err.unknown', params: {}, retryable: false }
    tier2BlockSetKeptMock.mockResolvedValue({ batch: null, error: err })

    await state.toggleImportPreviewBlockKept()

    expect(state.importPreviewBlockActionError.value).toEqual(err)
    expect(state.importPreviewSelectedBlocks.value?.blocks[0]?.kept).toBe(false) // khong doi
  })
})

describe('importPreviewState — [ ] đặt dải MỘT LƯỢT, "]" trước "[" kêu không ném', () => {
  it('"]" khi CHƯA đặt "[" bật importPreviewBlockRangeMissingStartNotice, 0 lời gọi IPC', async () => {
    const state = await freshState()
    const urls = ['https://a.example/1']
    startUrlImportMock.mockResolvedValue({ batch: batchWithBlocks(urls, THREE_BLOCKS), error: null })
    await state.openImportPreviewFromUrls('Ten', 'en', '', urls, null)

    expect(state.importPreviewBlockRangeMissingStartNotice.value).toBe(false)
    await state.confirmImportPreviewBlockRange()

    expect(state.importPreviewBlockRangeMissingStartNotice.value).toBe(true)
    expect(tier2BlockConfirmRangeMock).not.toHaveBeenCalled()
  })

  it('"[" tại khối 0 rồi di chuyển tới khối 2 rồi "]" gọi tier2BlockConfirmRange(0, 2, 3, sourceLang)', async () => {
    const state = await freshState()
    const urls = ['https://a.example/1']
    startUrlImportMock.mockResolvedValue({ batch: batchWithBlocks(urls, THREE_BLOCKS), error: null })
    await state.openImportPreviewFromUrls('Ten', 'en', '', urls, null)

    state.markImportPreviewBlockRangeStart()
    expect(state.importPreviewBlockRangeStart.value).toBe(0)
    state.nextImportPreviewBlock()
    state.nextImportPreviewBlock()
    expect(state.importPreviewBlockFocusedIndex.value).toBe(2)

    const allKept = THREE_BLOCKS.map((b) => ({ ...b, kept: true, confirmed: true }))
    tier2BlockConfirmRangeMock.mockResolvedValue({ batch: batchWithBlocks(urls, allKept), error: null })

    await state.confirmImportPreviewBlockRange()

    expect(tier2BlockConfirmRangeMock).toHaveBeenCalledTimes(1)
    expect(tier2BlockConfirmRangeMock).toHaveBeenCalledWith(0, 2, 3, 'en', 0)
    expect(state.importPreviewBlockRangeStart.value).toBeNull() // don sach sau khi thanh cong
    expect(state.importPreviewBlockRangeMissingStartNotice.value).toBe(false)
  })
})

// 🔴 THÊM 2026-09-07 (vòng rà bước 4, mục 19) — `watch(importPreviewSelectedBlocks, ...)` ở
// `importPreviewState.ts` (kẹp `blockFocusedIndex`/dọn `blockRangeStart` khi mảng khối CO
// LẠI dưới chân chúng) chưa từng chạy qua nhánh CO LẠI THẬT: mọi fixture của tệp này (kể cả
// `THREE_BLOCKS`) đều dùng ĐÚNG 3 khối trong SUỐT một ca — không ca nào đổi ĐỘ DÀI mảng khối
// giữa hai lần đọc. Một cài đặt SAI (ví dụ quên nhánh `if (blockFocusedIndex.value >= length)`)
// vẫn làm MỌI ca khác của tệp này xanh.
describe('importPreviewState — mảng khối CO LẠI kéo blockFocusedIndex/blockRangeStart về trong phạm vi mới', () => {
  it('mot lot ghi tra ve mang khoi NGAN HON keo ca hai ve dung pham vi con lai', async () => {
    const state = await freshState()
    const urls = ['https://a.example/1']
    startUrlImportMock.mockResolvedValue({ batch: batchWithBlocks(urls, THREE_BLOCKS), error: null })
    await state.openImportPreviewFromUrls('Ten', 'en', '', urls, null)

    // Đưa tiêu điểm tới khối CUỐI (2) và đặt mốc `[` ở đó.
    state.nextImportPreviewBlock()
    state.nextImportPreviewBlock()
    state.markImportPreviewBlockRangeStart()
    expect(state.importPreviewBlockFocusedIndex.value).toBe(2)
    expect(state.importPreviewBlockRangeStart.value).toBe(2)

    const oneBlock = [
      block({ body: { kind: 'paragraph', text: 'chi con dung mot khoi' }, kept: true, confirmed: false }),
    ]
    tier2BlockSetKeptMock.mockResolvedValue({ batch: batchWithBlocks(urls, oneBlock), error: null })
    await state.toggleImportPreviewBlockKept()
    await nextTick()

    expect(state.importPreviewSelectedBlocks.value?.blocks).toHaveLength(1)
    // `length - 1 == 0` -- chi so 2 cu vuot mang moi (do 1 phan tu), phai duoc kep ve chi so
    // hop le CUOI CUNG, khong duoc de lai 2 (doc `blocks[2]` se ra `undefined`).
    expect(state.importPreviewBlockFocusedIndex.value).toBe(0)
    expect(state.importPreviewBlockRangeStart.value).toBeNull()
  })

  it('mang khoi con lai RONG dua ca hai ve 0/null, khong con mot chi so nao het', async () => {
    const state = await freshState()
    const urls = ['https://a.example/1']
    startUrlImportMock.mockResolvedValue({ batch: batchWithBlocks(urls, THREE_BLOCKS), error: null })
    await state.openImportPreviewFromUrls('Ten', 'en', '', urls, null)

    state.nextImportPreviewBlock()
    state.markImportPreviewBlockRangeStart()
    expect(state.importPreviewBlockFocusedIndex.value).toBe(1)
    expect(state.importPreviewBlockRangeStart.value).toBe(1)

    tier2BlockSetKeptMock.mockResolvedValue({
      batch: {
        items: urls.map(item),
        encoding_preview: null,
        domain_log_domain_count: urls.length,
        duplicate_urls_dropped: 0,
      },
      error: null,
    })
    await state.toggleImportPreviewBlockKept()
    await nextTick()

    expect(state.importPreviewSelectedBlocks.value).toBeNull()
    expect(state.importPreviewBlockFocusedIndex.value).toBe(0)
    expect(state.importPreviewBlockRangeStart.value).toBeNull()
  })
})

describe('importPreviewState — R đếm lượt nhảy sang tầng 3, không gọi IPC', () => {
  it('mỗi lượt gọi tăng importPreviewJumpToCleanupRulesSignal đúng 1, kể cả hai lượt liên tiếp', async () => {
    const state = await freshState()
    const before = state.importPreviewJumpToCleanupRulesSignal.value

    state.jumpImportPreviewToCleanupRules()
    expect(state.importPreviewJumpToCleanupRulesSignal.value).toBe(before + 1)
    state.jumpImportPreviewToCleanupRules()
    expect(state.importPreviewJumpToCleanupRulesSignal.value).toBe(before + 2)

    expect(
      previewTextMock.mock.calls.length +
        previewFileMock.mock.calls.length +
        confirmMock.mock.calls.length +
        startUrlImportMock.mock.calls.length +
        tier2BlockSetKeptMock.mock.calls.length +
        tier2BlockConfirmRangeMock.mock.calls.length,
    ).toBe(0)
  })
})

// ═════════════════════════════════════════════════════════════════════════════════
// Lớp BÀN PHÍM — ImportPreviewOverlay.vue, dispatch qua CommandRegistry THẬT
// ═════════════════════════════════════════════════════════════════════════════════

describe('ImportPreviewOverlay.vue — bàn phím cục bộ dispatch ĐÚNG sáu lệnh qua registry THẬT', () => {
  it('j/k/space/[/]/r bắn đúng sáu lệnh import.preview.block_*/jump_to_cleanup_rules', async () => {
    const nextMock = vi.fn()
    const prevMock = vi.fn()
    const toggleMock = vi.fn()
    const markMock = vi.fn()
    const confirmRangeMock = vi.fn()
    const jumpMock = vi.fn()
    const { state, ImportPreviewOverlay } = await freshOverlay({
      nextImportPreviewBlock: nextMock,
      prevImportPreviewBlock: prevMock,
      toggleImportPreviewBlockKept: toggleMock,
      markImportPreviewBlockRangeStart: markMock,
      confirmImportPreviewBlockRange: confirmRangeMock,
      jumpImportPreviewToCleanupRules: jumpMock,
    })
    previewTextMock.mockResolvedValue({
      preview: previewWithBlocks(THREE_BLOCKS),
      error: null,
    })
    await state.openImportPreviewFromText('Ten', 'en', '', 'noi dung', null)

    const wrapper = mount(ImportPreviewOverlay, { attachTo: document.body })
    await wrapper.vm.$nextTick()
    const scrim = wrapper.get('.ip-screen')

    await scrim.trigger('keydown', { key: 'j' })
    expect(nextMock).toHaveBeenCalledTimes(1)
    await scrim.trigger('keydown', { key: 'k' })
    expect(prevMock).toHaveBeenCalledTimes(1)
    await scrim.trigger('keydown', { key: ' ' })
    expect(toggleMock).toHaveBeenCalledTimes(1)
    await scrim.trigger('keydown', { key: '[' })
    expect(markMock).toHaveBeenCalledTimes(1)
    await scrim.trigger('keydown', { key: ']' })
    expect(confirmRangeMock).toHaveBeenCalledTimes(1)
    await scrim.trigger('keydown', { key: 'r' })
    expect(jumpMock).toHaveBeenCalledTimes(1)

    wrapper.unmount()
  })

  // 🔴 THÊM 2026-09-07 (vòng rà bước 4, mục 18) — ca `j/k/space/[/]/r bắn đúng sáu lệnh` ở
  // trên MOCK `jumpImportPreviewToCleanupRules`, nên nó chỉ chứng minh "R gọi đúng hàm", KHÔNG
  // chứng minh hàm đó (hay `.vue`) thật sự CHUYỂN TIÊU ĐIỂM DOM sang tầng 3 — một cài đặt sai
  // (ví dụ `watch()` quên `.focus()`) vẫn làm ca đó XANH. Ca NÀY cài `CommandDeps` với hàm THẬT
  // từ `importPreviewState.ts` (không mock), rồi đọc `document.activeElement` sau khi bấm `r`
  // — cùng khuôn `glossaryManage.test.ts::'⑤d phần tử mang aria-activedescendant...'`.
  it('R chuyển TIÊU ĐIỂM DOM THẬT sang tầng 3, không chỉ tăng một bộ đếm tín hiệu', async () => {
    vi.resetModules()
    resetAllMocks()
    const commands = await import('../../src/commands')
    const state = await import('../../src/importPreviewState')
    commands.installCommands({
      jumpImportPreviewToCleanupRules: state.jumpImportPreviewToCleanupRules,
    } as CommandDeps)
    const ImportPreviewOverlay = (await import('../../src/ImportPreviewOverlay.vue')).default

    previewTextMock.mockResolvedValue({ preview: previewWithBlocks(THREE_BLOCKS), error: null })
    await state.openImportPreviewFromText('Ten', 'en', '', 'noi dung', null)

    const wrapper = mount(ImportPreviewOverlay, { attachTo: document.body })
    await wrapper.vm.$nextTick()
    const scrim = wrapper.get('.ip-screen')
    const tier3 = wrapper.get('[aria-labelledby="ip-tier-3-title"]')
    expect(document.activeElement).not.toBe(tier3.element) // diem khoi dau -- doi chung am.

    await scrim.trigger('keydown', { key: 'r' })
    // `jumpImportPreviewToCleanupRules()` chi tang `jumpToCleanupRulesSignal`; `watch()` cua
    // `.vue` chay TRONG mot `nextTick()` rieng cua no -- can hai luot de DOM + focus on dinh.
    await wrapper.vm.$nextTick()
    await wrapper.vm.$nextTick()

    expect(document.activeElement).toBe(tier3.element)

    wrapper.unmount()
  })

  // 🔴 THÊM 2026-09-07 (vòng rà bước 4, mục 9/10) — `J`/`K` phải chuyển tiêu điểm DOM sang
  // `<ol class="ip-blocks">` (điều kiện để `aria-activedescendant` trên chính nó có nghĩa với
  // trình đọc màn hình, ARIA 1.2) — cùng lý lẽ đo ở ca `R` ngay trên, áp cho J/K.
  it('J chuyển TIÊU ĐIỂM DOM THẬT sang <ol class="ip-blocks">', async () => {
    vi.resetModules()
    resetAllMocks()
    const commands = await import('../../src/commands')
    const state = await import('../../src/importPreviewState')
    commands.installCommands({
      nextImportPreviewBlock: state.nextImportPreviewBlock,
    } as CommandDeps)
    const ImportPreviewOverlay = (await import('../../src/ImportPreviewOverlay.vue')).default

    const urls = ['https://a.example/1']
    startUrlImportMock.mockResolvedValue({ batch: batchWithBlocks(urls, THREE_BLOCKS), error: null })
    await state.openImportPreviewFromUrls('Ten', 'en', '', urls, null)

    const wrapper = mount(ImportPreviewOverlay, { attachTo: document.body })
    await wrapper.vm.$nextTick()
    const scrim = wrapper.get('.ip-screen')
    const blocksList = wrapper.get('.ip-blocks')
    expect(document.activeElement).not.toBe(blocksList.element)

    await scrim.trigger('keydown', { key: 'j' })
    await wrapper.vm.$nextTick()
    await wrapper.vm.$nextTick()

    expect(document.activeElement).toBe(blocksList.element)

    wrapper.unmount()
  })

  it('Mod+Space/Mod+j KHÔNG bắn lệnh — hợp âm có bổ trợ bị lọc trước mọi nhánh', async () => {
    const toggleMock = vi.fn()
    const nextMock = vi.fn()
    const { state, ImportPreviewOverlay } = await freshOverlay({
      toggleImportPreviewBlockKept: toggleMock,
      nextImportPreviewBlock: nextMock,
    })
    previewTextMock.mockResolvedValue({ preview: previewWithBlocks(THREE_BLOCKS), error: null })
    await state.openImportPreviewFromText('Ten', 'en', '', 'noi dung', null)

    const wrapper = mount(ImportPreviewOverlay, { attachTo: document.body })
    await wrapper.vm.$nextTick()
    const scrim = wrapper.get('.ip-screen')

    await scrim.trigger('keydown', { key: ' ', metaKey: true })
    await scrim.trigger('keydown', { key: 'j', ctrlKey: true })

    expect(toggleMock).not.toHaveBeenCalled()
    expect(nextMock).not.toHaveBeenCalled()

    wrapper.unmount()
  })

  // 🔴 AC spec 6.9: "tiêu điểm đang ở một <button> bất kỳ, khi bấm Space, nút đó hoạt động
  // bình thường — không phím nào của story này chiếm hợp âm toàn cục". `trigger()` của
  // test-utils không đặt được `event.target`; bắn một `KeyboardEvent` THẬT từ chính cái nút
  // (nó nổi bọt lên `.ip-screen` đúng như một lượt gõ thật) — khuôn
  // `glossaryManage.test.ts::'Enter khi tiêu điểm đang ở một NÚT không bị nuốt thành lệnh Sửa'`.
  it('Space khi tiêu điểm đang ở một <button> KHÔNG bắn import.preview.block_toggle_kept', async () => {
    const toggleMock = vi.fn()
    const { state, ImportPreviewOverlay } = await freshOverlay({ toggleImportPreviewBlockKept: toggleMock })
    previewTextMock.mockResolvedValue({ preview: previewWithBlocks(THREE_BLOCKS), error: null })
    await state.openImportPreviewFromText('Ten', 'en', '', 'noi dung', null)

    const wrapper = mount(ImportPreviewOverlay, { attachTo: document.body })
    await wrapper.vm.$nextTick()

    const button = wrapper.element.querySelector('button')
    expect(button).not.toBeNull()
    button?.dispatchEvent(new KeyboardEvent('keydown', { key: ' ', bubbles: true }))
    await wrapper.vm.$nextTick()

    expect(toggleMock).not.toHaveBeenCalled()

    wrapper.unmount()
  })
})

// ═════════════════════════════════════════════════════════════════════════════════
// Lớp RENDER — ba vạch lề hiển thị + hai số đầu tầng
// ═════════════════════════════════════════════════════════════════════════════════

describe('ImportPreviewOverlay.vue — dãy khối render đúng ba vạch lề + hai số đếm', () => {
  it('render "Đã loại"/"Giữ · máy đoán"/"Giữ" đúng khối, và đếm đúng "1 khối giữ · ... loại"', async () => {
    const { state, ImportPreviewOverlay } = await freshOverlay()
    const urls = ['https://a.example/1']
    startUrlImportMock.mockResolvedValue({ batch: batchWithBlocks(urls, THREE_BLOCKS), error: null })
    await state.openImportPreviewFromUrls('Ten', 'en', '', urls, null)

    const wrapper = mount(ImportPreviewOverlay, { attachTo: document.body })
    await wrapper.vm.$nextTick()

    expect(wrapper.findAll('.ip-band')).toHaveLength(1)
    expect(wrapper.findAll('.ip-block')).toHaveLength(2)
    await state.toggleImportPreviewBlockKept()
    await wrapper.vm.$nextTick()
    const rows = wrapper.findAll('.ip-block')
    expect(rows).toHaveLength(3)
    expect(rows[0]?.classes()).toContain('ip-block-dropped')
    expect(rows[1]?.classes()).toContain('ip-block-kept-guessed')
    expect(rows[2]?.classes()).toContain('ip-block-kept-confirmed')
    // 🔴 SỬA 2026-09-07 (vòng rà bước 4, mục 17) — bản trước dùng `.text()).toContain('Giữ')`
    // trên CẢ DÒNG: 'Giữ · máy đoán' CŨNG chứa chuỗi con 'Giữ', nên dòng đó KHÔNG PHÂN BIỆT
    // ĐƯỢC "tag đúng là 'Giữ'" khỏi "tag lại là 'Giữ · máy đoán' do một lỗi hiển thị" — ca
    // này XANH ngay cả khi `blockStateMessageKey` trả nhầm khoá `guessed` cho khối đã
    // `confirmed`. Đọc đúng phần tử `.ip-block-tag` của TỪNG hàng rồi so BẰNG (không
    // `toContain`) để một tag SAI (kể cả "sai nhưng vẫn chứa chữ 'Giữ'") bị bắt.
    expect(rows[0]?.get('.ip-block-tag').text()).toBe('Đã loại')
    expect(rows[1]?.get('.ip-block-tag').text()).toBe('Giữ · máy đoán')
    expect(rows[2]?.get('.ip-block-tag').text()).toBe('Giữ')

    const counts = wrapper.get('.ip-tier2-counts').text()
    expect(counts).toContain('2 khối giữ') // block 1 + block 2 kept
    expect(counts).toContain('1 khối loại') // block 0 excluded

    wrapper.unmount()
  })
})

// ═════════════════════════════════════════════════════════════════════════════════
// Lớp CHUỘT — nhắm khối, cụm nút, ‹ ›, Tới luật làm sạch
// ═════════════════════════════════════════════════════════════════════════════════

const FIVE_BLOCKS: BlockWire[] = [...THREE_BLOCKS, block(), block()]

async function mountUrlOverlay(blocks: BlockWire[], chapterCount = 1) {
  vi.resetModules()
  resetAllMocks()
  const commands = await import('../../src/commands')
  const state = await import('../../src/importPreviewState')
  const calls = {
    toggle: vi.fn(),
    mark: vi.fn(),
    confirmRange: vi.fn(),
    jump: vi.fn(),
    chapterNext: vi.fn(),
    chapterPrev: vi.fn(),
  }
  commands.installCommands({
    nextImportPreviewBlock: state.nextImportPreviewBlock,
    prevImportPreviewBlock: state.prevImportPreviewBlock,
    toggleImportPreviewBlockKept: calls.toggle,
    markImportPreviewBlockRangeStart: calls.mark,
    confirmImportPreviewBlockRange: calls.confirmRange,
    jumpImportPreviewToCleanupRules: calls.jump,
    nextImportPreviewChapter: calls.chapterNext,
    prevImportPreviewChapter: calls.chapterPrev,
  } as unknown as CommandDeps)
  const ImportPreviewOverlay = (await import('../../src/ImportPreviewOverlay.vue')).default
  const urls = ['https://a.example/1']
  const batch = batchWithBlocks(urls, blocks)
  const candidate = batch.encoding_preview?.candidates[0]
  const template = candidate?.chapters?.chapters[0]
  if (candidate?.chapters != null && template !== undefined) {
    candidate.chapters = {
      ...candidate.chapters,
      chapter_count: chapterCount,
      chapters: Array.from({ length: chapterCount }, (_, i) => ({ ...template, ord: i + 1 })),
    }
  }
  startUrlImportMock.mockResolvedValue({ batch, error: null })
  await state.openImportPreviewFromUrls('Ten', 'en', '', urls, null)
  const wrapper = mount(ImportPreviewOverlay, { attachTo: document.body })
  await wrapper.vm.$nextTick()
  await openLeadingBand(state)
  await wrapper.vm.$nextTick()
  return { state, wrapper, calls }
}

describe('ImportPreviewOverlay.vue — chuột nhắm khối và cụm nút', () => {
  it('mousedown thật trên khối 3 nhắm khối 3, đúng một cụm nút, J tới khối 4 và K về khối 2', async () => {
    const { state, wrapper } = await mountUrlOverlay(FIVE_BLOCKS)
    expect(wrapper.findAll('.ip-block-actions')).toHaveLength(1)
    await wrapper.findAll('.ip-block')[3]?.trigger('mousedown')
    expect(state.importPreviewBlockFocusedIndex.value).toBe(3)
    expect(wrapper.findAll('.ip-block-actions')).toHaveLength(1)
    expect(wrapper.findAll('.ip-block')[3]?.find('.ip-block-actions').exists()).toBe(true)
    await wrapper.get('.ip-screen').trigger('keydown', { key: 'j' })
    expect(state.importPreviewBlockFocusedIndex.value).toBe(4)
    await wrapper.get('.ip-screen').trigger('keydown', { key: 'k' })
    await wrapper.get('.ip-screen').trigger('keydown', { key: 'k' })
    expect(state.importPreviewBlockFocusedIndex.value).toBe(2)
    wrapper.unmount()
  })

  it('bấm lại khối đang nhắm giữ nguyên chỉ số và J chạy tiếp từ đó', async () => {
    const { state, wrapper } = await mountUrlOverlay(FIVE_BLOCKS)
    await wrapper.findAll('.ip-block')[2]?.trigger('mousedown')
    await wrapper.findAll('.ip-block')[2]?.trigger('mousedown')
    expect(state.importPreviewBlockFocusedIndex.value).toBe(2)
    await wrapper.get('.ip-screen').trigger('keydown', { key: 'j' })
    expect(state.importPreviewBlockFocusedIndex.value).toBe(3)
    wrapper.unmount()
  })

  it('nhãn nút đổi theo block.kept, một nút duy nhất', async () => {
    const { wrapper } = await mountUrlOverlay(FIVE_BLOCKS)
    const first = wrapper.findAll('.ip-block')[0]!
    expect(first.findAll('.ip-block-actions button')[0]?.text()).toBe('Giữ khối này')
    await wrapper.findAll('.ip-block')[1]?.trigger('mousedown')
    expect(wrapper.findAll('.ip-block')[1]?.findAll('.ip-block-actions button')[0]?.text()).toBe('Loại khối này')
    wrapper.unmount()
  })

  it('(i) Loại/Giữ khối này bắn block_toggle_kept', async () => {
    const { wrapper, calls } = await mountUrlOverlay(FIVE_BLOCKS)
    await wrapper.findAll('.ip-block-actions button')[0]?.trigger('click')
    expect(calls.toggle).toHaveBeenCalledTimes(1)
    wrapper.unmount()
  })

  it('(i) Đầu vùng giữ bắn block_mark_range_start', async () => {
    const { wrapper, calls } = await mountUrlOverlay(FIVE_BLOCKS)
    await wrapper.findAll('.ip-block-actions button')[1]?.trigger('click')
    expect(calls.mark).toHaveBeenCalledTimes(1)
    wrapper.unmount()
  })

  it('(i) Cuối vùng giữ chỉ hiện khi đầu vùng ở khối khác, và bắn block_confirm_range', async () => {
    const { state, wrapper, calls } = await mountUrlOverlay(FIVE_BLOCKS)
    await wrapper.findAll('.ip-block')[4]?.trigger('mousedown')
    state.markImportPreviewBlockRangeStart()
    await wrapper.vm.$nextTick()
    expect(wrapper.findAll('.ip-block-actions button')).toHaveLength(2)
    await wrapper.findAll('.ip-block')[1]?.trigger('mousedown')
    const buttons = wrapper.findAll('.ip-block-actions button')
    expect(buttons).toHaveLength(3)
    expect(buttons[2]?.text()).toBe('Cuối vùng giữ')
    await buttons[2]?.trigger('click')
    expect(calls.confirmRange).toHaveBeenCalledTimes(1)
    wrapper.unmount()
  })

  it('(i) Tới luật làm sạch bắn jump_to_cleanup_rules', async () => {
    const { wrapper, calls } = await mountUrlOverlay(FIVE_BLOCKS)
    await wrapper.get('.ip-tier2-jump').trigger('click')
    expect(calls.jump).toHaveBeenCalledTimes(1)
    wrapper.unmount()
  })

  it('(i) ‹ › bắn chapter_prev / chapter_next, tắt ở hai đầu', async () => {
    const { state, wrapper, calls } = await mountUrlOverlay(FIVE_BLOCKS, 3)
    const [prev, next] = wrapper.findAll('.ip-chapter-cursor-row button')
    expect(prev.attributes('disabled')).toBeDefined()
    expect(next.attributes('disabled')).toBeUndefined()
    await next.trigger('click')
    expect(calls.chapterNext).toHaveBeenCalledTimes(1)
    previewChapterDetailMock.mockResolvedValue({ detail: { cleanup: { text: 'x', spans: [], rules: [], window_truncated: false, final_text: 'x' }, blocks: blocksWire(FIVE_BLOCKS) }, error: null })
    state.nextImportPreviewChapter()
    await new Promise((r) => setTimeout(r))
    state.nextImportPreviewChapter()
    await new Promise((r) => setTimeout(r))
    await wrapper.vm.$nextTick()
    expect(state.importPreviewChapterCursor.value).toBe(2)
    const [prev2, next2] = wrapper.findAll('.ip-chapter-cursor-row button')
    expect(next2.attributes('disabled')).toBeDefined()
    expect(prev2.attributes('disabled')).toBeUndefined()
    await prev2.trigger('click')
    expect(calls.chapterPrev).toHaveBeenCalledTimes(1)
    wrapper.unmount()
  })

  it('một Chương: không có ‹ ›', async () => {
    const { wrapper } = await mountUrlOverlay(FIVE_BLOCKS, 1)
    expect(wrapper.find('.ip-chapter-cursor-row').exists()).toBe(false)
    wrapper.unmount()
  })

  it('Chương k > 0: cụm nút hiện, cả ba tắt, title nói lý do', async () => {
    const { state, wrapper } = await mountUrlOverlay(FIVE_BLOCKS, 3)
    previewChapterDetailMock.mockResolvedValue({ detail: { cleanup: { text: 'x', spans: [], rules: [], window_truncated: false, final_text: 'x' }, blocks: blocksWire(FIVE_BLOCKS) }, error: null })
    state.nextImportPreviewChapter()
    await new Promise((r) => setTimeout(r))
    await wrapper.vm.$nextTick()
    await wrapper.findAll('.ip-block')[0]?.trigger('mousedown')
    const buttons = wrapper.findAll('.ip-block-actions button')
    expect(buttons.length).toBeGreaterThanOrEqual(2)
    for (const b of buttons) {
      expect(b.attributes('disabled')).toBeDefined()
      expect(b.attributes('title')).toContain('Chương đầu tiên')
    }
    wrapper.unmount()
  })

  it('đường dán: không cụm nút, không ‹ ›, không nút tới luật', async () => {
    const { state, ImportPreviewOverlay } = await (async () => {
      const f = await freshOverlay()
      return { state: f.state, ImportPreviewOverlay: f.ImportPreviewOverlay }
    })()
    previewTextMock.mockResolvedValue({ preview: previewWithBlocks(THREE_BLOCKS), error: null })
    await state.openImportPreviewFromText('Ten', 'en', '', 'noi dung', null)
    const wrapper = mount(ImportPreviewOverlay, { attachTo: document.body })
    await wrapper.vm.$nextTick()
    expect(wrapper.find('.ip-block-actions').exists()).toBe(false)
    expect(wrapper.find('.ip-chapter-cursor-row').exists()).toBe(false)
    expect(wrapper.find('.ip-tier2-jump').exists()).toBe(false)
    wrapper.unmount()
  })
})

describe('ImportPreviewOverlay.vue — nút chạy qua hàm ghi thật', () => {
  const URLS = ['https://a.example/1']

  async function mountWithRealWrites() {
    vi.resetModules()
    resetAllMocks()
    const commands = await import('../../src/commands')
    const state = await import('../../src/importPreviewState')
    const ImportPreviewOverlay = (await import('../../src/ImportPreviewOverlay.vue')).default
    commands.installCommands({
      nextImportPreviewBlock: state.nextImportPreviewBlock,
      prevImportPreviewBlock: state.prevImportPreviewBlock,
      toggleImportPreviewBlockKept: state.toggleImportPreviewBlockKept,
      markImportPreviewBlockRangeStart: state.markImportPreviewBlockRangeStart,
      confirmImportPreviewBlockRange: state.confirmImportPreviewBlockRange,
    } as unknown as CommandDeps)
    startUrlImportMock.mockResolvedValue({ batch: batchWithBlocks(URLS, FIVE_BLOCKS), error: null })
    await state.openImportPreviewFromUrls('Ten', 'en', '', URLS, null)
    const wrapper = mount(ImportPreviewOverlay, { attachTo: document.body })
    await nextTick()
    await openLeadingBand(state)
    await nextTick()
    return { state, wrapper }
  }

  async function settle(wrapper: { vm: { $nextTick: () => Promise<void> } }): Promise<void> {
    await new Promise((resolve) => setTimeout(resolve))
    await wrapper.vm.$nextTick()
  }

  it('Loại khối này gửi đúng khối, nhãn và bộ đếm đổi, J chạy tiếp', async () => {
    const { state, wrapper } = await mountWithRealWrites()
    await wrapper.findAll('.ip-block')[1]?.trigger('mousedown')
    const dropped = [...FIVE_BLOCKS]
    dropped[1] = block({ kept: false, confirmed: true })
    tier2BlockSetKeptMock.mockResolvedValue({ batch: batchWithBlocks(URLS, dropped), error: null })
    expect(wrapper.get('.ip-tier2-counts').text()).toContain('4 khối giữ · 1 khối loại')

    await wrapper.findAll('.ip-block-actions button')[0]?.trigger('click')
    await settle(wrapper)

    expect(tier2BlockSetKeptMock).toHaveBeenCalledWith(1, false, 'en', 0)
    expect(wrapper.findAll('.ip-block-actions button')[0]?.text()).toBe('Giữ khối này')
    expect(wrapper.get('.ip-tier2-counts').text()).toContain('3 khối giữ · 2 khối loại')
    await wrapper.get('.ip-screen').trigger('keydown', { key: 'j' })
    expect(state.importPreviewBlockFocusedIndex.value).toBe(2)
    wrapper.unmount()
  })

  it('dải đảo chiều gửi (4, 1, 5), Cuối vùng giữ biến mất, J chạy tiếp từ khối cuối', async () => {
    const { state, wrapper } = await mountWithRealWrites()
    await wrapper.findAll('.ip-block')[4]?.trigger('mousedown')
    await wrapper.findAll('.ip-block-actions button')[1]?.trigger('click')
    expect(state.importPreviewBlockRangeStart.value).toBe(4)
    await wrapper.findAll('.ip-block')[1]?.trigger('mousedown')
    tier2BlockConfirmRangeMock.mockResolvedValue({ batch: batchWithBlocks(URLS, FIVE_BLOCKS), error: null })

    await wrapper.findAll('.ip-block-actions button')[2]?.trigger('click')
    await settle(wrapper)

    expect(tier2BlockConfirmRangeMock).toHaveBeenCalledWith(4, 1, 5, 'en', 0)
    expect(wrapper.findAll('.ip-block-actions button')).toHaveLength(2)
    await wrapper.get('.ip-screen').trigger('keydown', { key: 'j' })
    expect(state.importPreviewBlockFocusedIndex.value).toBe(2)
    wrapper.unmount()
  })

  it('lỗi ghi hiện ở dòng role=alert, nút mở lại để thử lại', async () => {
    const { wrapper } = await mountWithRealWrites()
    tier2BlockSetKeptMock.mockResolvedValue({
      batch: null,
      error: { code: 'import.tier2_edit_locked_to_first_chapter', message_key: 'err.import.tier2_edit_locked_to_first_chapter', params: {} },
    })
    await wrapper.findAll('.ip-block-actions button')[0]?.trigger('click')
    await settle(wrapper)
    expect(wrapper.findAll('.ip-tier2-range-notice').some((el) => el.attributes('role') === 'alert')).toBe(true)
    expect(wrapper.findAll('.ip-block-actions button')[0]?.attributes('disabled')).toBeUndefined()
    wrapper.unmount()
  })

  it('đang ghi: cả cụm nút tắt, không gửi lần thứ hai', async () => {
    const { wrapper } = await mountWithRealWrites()
    let release: (value: unknown) => void = () => undefined
    tier2BlockSetKeptMock.mockReturnValue(new Promise((resolve) => (release = resolve)))
    await wrapper.findAll('.ip-block-actions button')[0]?.trigger('click')
    await wrapper.vm.$nextTick()
    for (const button of wrapper.findAll('.ip-block-actions button')) {
      expect(button.attributes('disabled')).toBeDefined()
    }
    await wrapper.findAll('.ip-block-actions button')[0]?.trigger('click')
    expect(tier2BlockSetKeptMock).toHaveBeenCalledTimes(1)
    release({ batch: batchWithBlocks(URLS, FIVE_BLOCKS), error: null })
    await settle(wrapper)
    wrapper.unmount()
  })

  it('cụm nút và ‹ › chặn mặc định mousedown, khối thì không để còn bôi được chữ', async () => {
    const { wrapper } = await mountUrlOverlay(FIVE_BLOCKS, 3)
    const press = (el: Element): boolean => {
      const event = new MouseEvent('mousedown', { bubbles: true, cancelable: true })
      el.dispatchEvent(event)
      return event.defaultPrevented
    }
    for (const button of wrapper.findAll('.ip-block-actions button')) expect(press(button.element)).toBe(true)
    for (const button of wrapper.findAll('.ip-chapter-cursor-row button')) expect(press(button.element)).toBe(true)
    expect(press(wrapper.findAll('.ip-block-text')[3]!.element)).toBe(false)
    wrapper.unmount()
  })

  it('nhắm khối chuyển tiêu điểm sang danh sách mà không để focus() tự cuộn', async () => {
    const { wrapper } = await mountUrlOverlay(FIVE_BLOCKS)
    const list = wrapper.get('.ip-blocks').element as HTMLElement
    const focus = vi.spyOn(list, 'focus')
    await wrapper.findAll('.ip-block')[3]?.trigger('mousedown')
    await nextTick()
    await nextTick()
    expect(focus).toHaveBeenCalledWith({ preventScroll: true })
    wrapper.unmount()
  })

  it('› bằng chuột giữ nguyên tầm nhìn, ⌥→ vẫn chuyển tiêu điểm sang danh sách Chương', async () => {
    vi.resetModules()
    resetAllMocks()
    const commands = await import('../../src/commands')
    const state = await import('../../src/importPreviewState')
    const ImportPreviewOverlay = (await import('../../src/ImportPreviewOverlay.vue')).default
    commands.installCommands({
      nextImportPreviewChapter: state.nextImportPreviewChapter,
      prevImportPreviewChapter: state.prevImportPreviewChapter,
    } as CommandDeps)
    const batch = batchWithBlocks(URLS, FIVE_BLOCKS)
    const candidate = batch.encoding_preview?.candidates[0]
    const template = candidate?.chapters?.chapters[0]
    if (candidate?.chapters == null || template === undefined) throw new Error('fixture has no chapters')
    candidate.chapters = {
      ...candidate.chapters,
      chapter_count: 3,
      chapters: [1, 2, 3].map((ord) => ({ ...template, ord })),
    }
    startUrlImportMock.mockResolvedValue({ batch, error: null })
    previewChapterDetailMock.mockResolvedValue({
      detail: { cleanup: { text: 'x', spans: [], rules: [], window_truncated: false, final_text: 'x' }, blocks: blocksWire(FIVE_BLOCKS) },
      error: null,
    })
    await state.openImportPreviewFromUrls('Ten', 'en', '', URLS, null)
    const wrapper = mount(ImportPreviewOverlay, { attachTo: document.body })
    await nextTick()
    const focus = vi.spyOn(wrapper.get('.ip-chapters-list').element as HTMLElement, 'focus')

    const next = wrapper.findAll('.ip-chapter-cursor-row button')[1]!
    await next.trigger('mousedown')
    await next.trigger('click')
    await settle(wrapper)
    expect(state.importPreviewChapterCursor.value).toBe(1)
    expect(focus).not.toHaveBeenCalled()
    expect(wrapper.get('.ip-screen').element.contains(document.activeElement)).toBe(true)

    await wrapper.get('.ip-screen').trigger('keydown', { key: 'ArrowLeft', altKey: true })
    await settle(wrapper)
    expect(state.importPreviewChapterCursor.value).toBe(0)
    expect(focus).toHaveBeenCalled()
    wrapper.unmount()
  })
})
