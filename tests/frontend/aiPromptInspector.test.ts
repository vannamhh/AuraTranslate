/**
 * Lớp phủ **Xem prompt cuối cùng đã gửi** — Story 4.7 (FR71, AD-14, Decision 2/3).
 *
 * ⚠️ Cùng khuôn `glossaryQueue.test.ts`/`glossaryConfirmStripTemplate.test.ts`:
 * `config/aiprompt.ts` là biên IPC, giả lập bằng `vi.mock` (KHÔNG gọi `@tauri-apps/api` thật);
 * `panels/editorPanelState.ts` bị `vi.doMock` thành một `ref()` THẬT mà test tự điều khiển
 * (component chỉ cần đúng MỘT export, `editorCaretSegmentId` — mock module đó nặng, kéo theo
 * cả Panel Editor, đúng lý do `glossaryConfirmStripTemplate.test.ts` đã ghi).
 *
 * ─────────────────────────────────────────────────────────────────────────────
 * 🔴 VÌ SAO TỆP NÀY, KHÔNG CHỈ CANH `aiPromptInspectorState.ts` Ở TẦNG STATE
 * ─────────────────────────────────────────────────────────────────────────────
 * `AGENTS.md` (root): "412 lines of ribbon tests never mounted the component" là một khuyết
 * tật ĐÃ XẢY RA trong dự án này. Mọi ca ba-trạng-thái dưới đây `mount(AiPromptInspectorOverlay)`
 * thật — không chỉ gọi hàm thuần rồi đọc giá trị trả về — để `v-if`/`data-*` thật của TEMPLATE
 * bị canh, không chỉ state phía sau nó.
 *
 * ─────────────────────────────────────────────────────────────────────────────
 * 🔴 ĐỐI CHỨNG TRỰC TIẾP CHO DECISION 2 — "mở màn hình KHÔNG BAO GIỜ lắp ráp"
 * ─────────────────────────────────────────────────────────────────────────────
 * `describe('dispatch mo/dong...')` dưới đây dispatch qua ĐÚNG `CommandRegistry` thật
 * (`installCommands` + `dispatch`, không gọi tắt hàm state), với `deps.openAiPromptInspector`/
 * `closeAiPromptInspector` trỏ THẲNG vào hàm SẢN PHẨM của `aiPromptInspectorState.ts` — rồi
 * spy trên adapter `aiPromptAssemble` và khẳng định nó KHÔNG BAO GIỜ được gọi. Đây là phép gỡ
 * trực tiếp: nếu một lượt sửa sau này lỡ nối `openAiPromptInspector` sang
 * `assembleCurrentAiPrompt`, ca này đỏ ngay, nêu đích danh.
 */
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'
import { flushPromises, mount } from '@vue/test-utils'
import { ref } from 'vue'
import type { Ref } from 'vue'
import type {
  AssembledPromptWire,
  GlossaryInjectionStatusWire,
  InjectedGlossaryTermWire,
  PromptPieceWire,
  SuppressedGlossaryTermWire,
  TmInjectionStatusWire,
} from '../../src/config/aiprompt'
import type { CommandDeps } from '../../src/commands'
import type { IpcError } from '../../src/i18n'

const assembleMock = vi.fn()
const readRecordMock = vi.fn()
/** Chỉ dùng ở nhóm `freshPanel()` — `AiTranslationPanel.vue`'s `onMounted` tự gọi
 * `loadPromptSets()` (`promptSetState.ts`), và `setSelectedPromptSetName()` từ chối một tên
 * KHÔNG có trong `resolvedSets` đã nạp (0 lượt IPC theo thiết kế Story 4.4 — xem doc-comment
 * `promptSetState.ts::setSelectedPromptSetName`) — bắt được thật lúc viết ca `freshPanel()`
 * đầu tiên: gọi `setSelectedPromptSetName('Happy')` mà không mock `promptSetList` trước làm
 * lượt chọn bị BỎ QUA ÂM THẦM, và `assembleMock` nhận `null` thay vì `'Happy'`. */
const promptSetListMock = vi.fn()

/** Cách nạp lỗi ĐỌC vào `readRecordMock` cho các ca canh findings B4/E3/E10 — xem `vi.mock`
 * ngay dưới cho lý do hình dạng này. */
function readError(error: IpcError): { __readErrorPayload: IpcError } {
  return { __readErrorPayload: error }
}

vi.mock('../../src/config/aiprompt', () => ({
  aiPromptAssemble: (...args: unknown[]) => assembleMock(...args),
  // 🔴 findings B4/E3/E10 (loop 1) — `aiPromptReadRecord` thật trả `{ value, error }`, không
  // còn bare `T | null` (xem `config/aiprompt.ts`). Mười chín chỗ gọi `readRecordMock.
  // mockResolvedValue(record(...) | null)` đã có trong tệp này TRƯỚC bản sửa vẫn viết NGUYÊN
  // VĂN raw value — vỏ mock này tự bọc lại thành `{ value: raw, error: null }` (đọc THÀNH
  // CÔNG) để không phải sửa cả 19 chỗ; các ca MỚI canh riêng đường lỗi truyền một payload đặc
  // biệt qua [`readError`] mà vỏ này mở khoá thành `{ value: null, error }`.
  aiPromptReadRecord: async () => {
    const raw: unknown = await readRecordMock()
    if (raw !== null && typeof raw === 'object' && '__readErrorPayload' in raw) {
      return { value: null, error: (raw as { __readErrorPayload: IpcError }).__readErrorPayload }
    }
    return { value: raw as AssembledPromptWire | null, error: null }
  },
}))

// 🔴 SỬA finding E8 (loop 1) — `promptSetState.ts` (nạp THẬT, không mock, ở cả `freshOverlay()`
// lẫn `freshPanel()`) `import` ĐÚNG SÁU hàm từ `config/promptset.ts`: `promptSetCreate`/
// `promptSetDelete`/`promptSetExport`/`promptSetList`/`promptSetRename`/`promptSetUpdateBody`.
// Bản trước chỉ stub `promptSetList` — năm hàm còn lại là `undefined` trong module đã mock,
// TRÌ TRỆ hôm nay (không ca nào ở tệp này gọi tới đường Tạo/Xoá/Xuất/Đổi tên/Sửa thân bộ
// prompt) nhưng là một quả mìn: ca ĐẦU TIÊN chạm một trong năm đường đó sẽ ném "X is not a
// function", không phải một lỗi rõ ràng về thiếu mock. Stub cả sáu, dù năm cái sau không
// một lượt gọi nào trong tệp này.
vi.mock('../../src/config/promptset', () => ({
  promptSetList: (...args: unknown[]) => promptSetListMock(...args),
  promptSetCreate: vi.fn(),
  promptSetDelete: vi.fn(),
  promptSetExport: vi.fn(),
  promptSetRename: vi.fn(),
  promptSetUpdateBody: vi.fn(),
}))

function injectedTerm(overrides: Partial<InjectedGlossaryTermWire> = {}): InjectedGlossaryTermWire {
  return { source_term: 'dragon', translation: 'rong', start: 2, end: 8, tier: 'global', ...overrides }
}

function suppressedTerm(overrides: Partial<SuppressedGlossaryTermWire> = {}): SuppressedGlossaryTermWire {
  return { source_term: 'dog', translation: 'cho', start: 4, end: 7, tier: 'global', ...overrides }
}

const NOT_ASKED: GlossaryInjectionStatusWire = { kind: 'not_asked', injected: null, suppressed_by_pending_overlap: null }

function asked(
  injected: InjectedGlossaryTermWire[],
  suppressed: SuppressedGlossaryTermWire[] = [],
): GlossaryInjectionStatusWire {
  return { kind: 'asked', injected, suppressed_by_pending_overlap: suppressed }
}

const TM_NOT_BUILT: TmInjectionStatusWire = { kind: 'not_built_yet', similar_segments: null }

/** Mảnh mặc định khớp NGUYÊN VĂN `record()`'s `prompt` mặc định bên dưới — phép nối `.text`
 * của mảng này PHẢI cho lại đúng `prompt` (đối chứng B1, loop 1). Đè cả hai cùng lúc qua
 * `record({ prompt: ..., ledger: { ...NOT_ASKED_LEDGER, pieces: [...] } })` khi một ca cần một
 * `prompt` khác. */
const DEFAULT_PIECES: PromptPieceWire[] = [
  { kind: 'authored', text: 'Terms: ' },
  { kind: 'glossary', text: 'dragon → rong' },
  { kind: 'authored', text: '\nSentence: A dragon roared.' },
]

/** Khuôn `candidate()` của `glossaryQueue.test.ts` — một bản ghi hợp lệ, đè từng phần. */
function record(overrides: Partial<AssembledPromptWire> = {}): AssembledPromptWire {
  return {
    prompt: 'Terms: dragon → rong\nSentence: A dragon roared.',
    segment_id: 1,
    chapter_id: 10,
    prompt_set_name: 'Happy',
    prompt_set_tier: 'global',
    ledger: {
      glossary: NOT_ASKED,
      tm: TM_NOT_BUILT,
      unknown_markers: [],
      source_segment_missing: false,
      pieces: DEFAULT_PIECES,
    },
    ...overrides,
  }
}

/** Đúng khuôn `glossaryConfirmStripTemplate.test.ts::freshStrip` — `vi.doMock` (KHÔNG
 * `vi.mock`) vì cần một `ref()` MỚI mỗi lượt `freshOverlay()`, không hoist. */
async function freshOverlay() {
  vi.resetModules()
  assembleMock.mockReset()
  readRecordMock.mockReset()

  const caretSegmentId: Ref<number | null> = ref(null)
  vi.doMock('../../src/panels/editorPanelState', () => ({ editorCaretSegmentId: caretSegmentId }))

  const commands = await import('../../src/commands')
  const state = await import('../../src/aiPromptInspectorState')
  const i18n = await import('../../src/i18n')
  const AiPromptInspectorOverlay = (await import('../../src/AiPromptInspectorOverlay.vue')).default

  commands.installCommands({
    openAiPromptInspector: state.openAiPromptInspector,
    closeAiPromptInspector: state.closeAiPromptInspector,
  } as CommandDeps)

  return { commands, state, i18n, AiPromptInspectorOverlay, caretSegmentId }
}

/**
 * SỬA 2026-09-18, bắt được ở lượt rà soát build (verification-gap layer): `assembleCurrentAiPrompt`
 * — nhịp GHI thật của Decision 2, thứ DUY NHẤT trong toàn kho gọi `ai_prompt_assemble` — không
 * có một ca nào canh trước bản sửa. `freshOverlay()` chỉ đăng ký `openAiPromptInspector`/
 * `closeAiPromptInspector`; helper riêng này mount THẬT `AiTranslationPanel.vue` (nút "Lắp
 * prompt cho câu này" sống ở đó, không phải ở lớp phủ) và đăng ký CẢ BA command, đúng khuôn
 * `main.ts::boot()`'s `assembleAiPrompt: () => { void assembleCurrentAiPrompt(selectedPromptSetName.value,
 * editorCaretSegmentId.value) }`. Không mock `config/promptset` — `promptSetList()`/
 * `setSelectedPromptSetName()` không gọi `invoke` khi không có cầu Tauri (0 lượt IPC theo
 * thiết kế Story 4.4), nên component mount an toàn mà không cần giả lập tầng đó.
 */
async function freshPanel() {
  vi.resetModules()
  assembleMock.mockReset()
  readRecordMock.mockReset()
  promptSetListMock.mockReset()
  // Một bộ Global tên `Happy` — đủ để `setSelectedPromptSetName('Happy')` (gọi SAU khi mount +
  // `flushPromises()`, không trước — `resolvedSets` chỉ có nội dung sau khi `onMounted`'s
  // `loadPromptSets()` trả lời) không bị từ chối âm thầm.
  promptSetListMock.mockResolvedValue({
    sets: [{ id: 1, name: 'Happy', body: '{{source_segment}}', tier: 'global', shadowed_body: null, shadowed_id: null }],
    workTierAvailable: false,
    variables: [],
    error: null,
  })
  // `AiTranslationPanel.vue`'s `onMounted` tự gọi `refreshAiPromptRecord()` (đồng bộ dòng
  // tóm tắt với bản ghi Rust — xem doc-comment ở đó). Đặt mặc định `null` ở ĐÂY (khớp I/O
  // Matrix "Nothing recorded yet") — một `vi.fn()` chưa cấu hình trả `undefined`, và
  // `aiPromptRecord.value` mang `undefined` thay vì `null` làm `glossaryInjectionSummary`
  // ném (`current === null` không bắt được `undefined`) — bắt được thật lúc viết ca này,
  // không phải một đoán trước; mỗi ca tự ghi đè khi cần một bản ghi khác.
  readRecordMock.mockResolvedValue(null)

  const caretSegmentId: Ref<number | null> = ref(null)
  vi.doMock('../../src/panels/editorPanelState', () => ({ editorCaretSegmentId: caretSegmentId }))

  const commands = await import('../../src/commands')
  const state = await import('../../src/aiPromptInspectorState')
  const promptSetState = await import('../../src/promptSetState')
  const i18n = await import('../../src/i18n')
  const AiTranslationPanel = (await import('../../src/panels/AiTranslationPanel.vue')).default

  commands.installCommands({
    openAiPromptInspector: state.openAiPromptInspector,
    closeAiPromptInspector: state.closeAiPromptInspector,
    assembleAiPrompt: () => {
      void state.assembleCurrentAiPrompt(promptSetState.selectedPromptSetName.value, caretSegmentId.value)
    },
  } as CommandDeps)

  return { commands, state, i18n, promptSetState, AiTranslationPanel, caretSegmentId }
}

beforeEach(() => {
  document.body.innerHTML = ''
})

afterEach(() => {
  vi.restoreAllMocks()
})

// ═══════════════════════════════════════════════════════════════════════════════════
// `glossaryInjectionSummary` — ba trạng thái, ĐỌC TỪ CHÍNH bản ghi (§Always spec 4.7)
// ═══════════════════════════════════════════════════════════════════════════════════

describe('glossaryInjectionSummary — no_record / not_asked / asked-N, không collapse', () => {
  it('current === null ⇒ "no_record" (chưa lắp lần nào trong phiên)', async () => {
    const { state } = await freshOverlay()
    expect(state.glossaryInjectionSummary(null)).toEqual({ kind: 'no_record' })
  })

  it('ledger.glossary.kind === "not_asked" ⇒ "not_asked", KHÔNG một count', async () => {
    const { state } = await freshOverlay()
    const summary = state.glossaryInjectionSummary(record({ ledger: { ...record().ledger, glossary: NOT_ASKED } }))
    expect(summary).toEqual({ kind: 'not_asked' })
  })

  it('asked với injected rỗng ⇒ "asked" count 0 — KHÁC "not_asked"', async () => {
    const { state } = await freshOverlay()
    const summary = state.glossaryInjectionSummary(record({ ledger: { ...record().ledger, glossary: asked([]) } }))
    expect(summary).toEqual({ kind: 'asked', count: 0 })
  })

  it('asked với hai thuật ngữ ⇒ "asked" count 2, đọc trực tiếp từ injected.length', async () => {
    const { state } = await freshOverlay()
    const summary = state.glossaryInjectionSummary(
      record({ ledger: { ...record().ledger, glossary: asked([injectedTerm(), injectedTerm({ source_term: 'castle' })]) } }),
    )
    expect(summary).toEqual({ kind: 'asked', count: 2 })
  })
})

// ═══════════════════════════════════════════════════════════════════════════════════
// `aiPromptRecordIsStale` — hàm thuần
// ═══════════════════════════════════════════════════════════════════════════════════

describe('aiPromptRecordIsStale — hàm thuần', () => {
  it('current === null ⇒ false (không có gì để so)', async () => {
    const { state } = await freshOverlay()
    expect(state.aiPromptRecordIsStale(null, 5)).toBe(false)
  })

  it('focusedSegmentId === null ⇒ false, không tự xưng "cũ" khi không câu nào đang chọn', async () => {
    const { state } = await freshOverlay()
    expect(state.aiPromptRecordIsStale(record({ segment_id: 1 }), null)).toBe(false)
  })

  it('segment_id khác câu đang focus ⇒ true', async () => {
    const { state } = await freshOverlay()
    expect(state.aiPromptRecordIsStale(record({ segment_id: 1 }), 2)).toBe(true)
  })

  it('segment_id CÙNG câu đang focus ⇒ false', async () => {
    const { state } = await freshOverlay()
    expect(state.aiPromptRecordIsStale(record({ segment_id: 1 }), 1)).toBe(false)
  })
})

// ═══════════════════════════════════════════════════════════════════════════════════
// Mount thật — ba cặp ba-trạng-thái phải sống sót qua ĐÚNG LỚP DÂY (wire), không chỉ nội bộ
// ═══════════════════════════════════════════════════════════════════════════════════

describe('AiPromptInspectorOverlay.vue — "no record" khác "một bản ghi có prompt rỗng"', () => {
  it('chưa lắp lần nào trong phiên ⇒ data-aip-record-state="none", không render phần thân', async () => {
    const { state, AiPromptInspectorOverlay } = await freshOverlay()
    readRecordMock.mockResolvedValue(null)
    state.openAiPromptInspector()
    await flushPromises()

    const wrapper = mount(AiPromptInspectorOverlay)
    await wrapper.vm.$nextTick()

    expect(wrapper.find('[data-aip-record-state="none"]').exists()).toBe(true)
    expect(wrapper.find('[data-aip-record-state="present"]').exists()).toBe(false)

    wrapper.unmount()
  })

  it('một bản ghi có prompt RỖNG ⇒ data-aip-record-state="present" + data-aip-body-state="empty"', async () => {
    const { state, AiPromptInspectorOverlay } = await freshOverlay()
    readRecordMock.mockResolvedValue(record({ prompt: '' }))
    state.openAiPromptInspector()
    await flushPromises()

    const wrapper = mount(AiPromptInspectorOverlay)
    await wrapper.vm.$nextTick()

    expect(wrapper.find('[data-aip-record-state="present"]').exists()).toBe(true)
    expect(wrapper.find('[data-aip-body-state="empty"]').exists()).toBe(true)
    expect(wrapper.find('[data-aip-body-state="present"]').exists()).toBe(false)

    wrapper.unmount()
  })

  it('một bản ghi có prompt KHÔNG rỗng ⇒ data-aip-body-state="present", nội dung y byte-for-byte', async () => {
    const { state, AiPromptInspectorOverlay } = await freshOverlay()
    readRecordMock.mockResolvedValue(
      record({
        prompt: 'Prompt thật.',
        ledger: { ...record().ledger, pieces: [{ kind: 'authored', text: 'Prompt thật.' }] },
      }),
    )
    state.openAiPromptInspector()
    await flushPromises()

    const wrapper = mount(AiPromptInspectorOverlay)
    await wrapper.vm.$nextTick()

    expect(wrapper.find('[data-aip-body-state="present"]').exists()).toBe(true)
    expect(wrapper.get('[data-aip-body-state="present"]').text()).toBe('Prompt thật.')

    wrapper.unmount()
  })

  // 🔴 finding B1 (loop 1) -- đối chứng THẬT của §Always/AC2: `pieces` phải vẽ TỪNG mảnh theo
  // nhãn của nó (khác lớp phủ chỉ tô cả `prompt` một màu), VÀ nối `.text` của toàn bộ mảng lại
  // vẫn phải khớp `prompt` TỪNG BYTE -- ca này đi lệch hẳn `DEFAULT_PIECES` để không xanh nhờ
  // một sự trùng hợp giữa fixture mặc định và code.
  //
  // 🔴 SỬA finding P12 (loop 2) -- "TỪNG BYTE" trong tên ca cũ KHÔNG phải thứ assertion đo:
  // `@vue/test-utils`'s `.text()` TRIM khoảng trắng đầu/cuối, nên một khoảng trắng bị nuốt mất
  // ở biên vẫn cho ca xanh. Cả hai mảnh `authored` ở fixture dưới đây cố ý mang khoảng trắng
  // đầu/cuối, và đối chứng đọc `element.textContent` THÔ (không trim) thay vì `.text()`. Đồng
  // thời gộp findings P7/P9: fixture giờ có đủ BỐN nhãn (kể cả `source_segment`/`tm`, hai nhãn
  // vừa được giới thiệu/sửa ở Phase 6), mỗi nhãn một ca so cả `data-aip-piece-kind` LẪN lớp CSS
  // của chính nó VÀ xác nhận nó KHÔNG mang lớp của ba nhãn còn lại.
  it('vẽ TỪNG mảnh `ledger.pieces` theo nhãn (cả bốn nhãn), và nối lại khớp `prompt` TỪNG BYTE kể cả khoảng trắng đầu/cuối', async () => {
    const { state, AiPromptInspectorOverlay } = await freshOverlay()
    const pieces: PromptPieceWire[] = [
      { kind: 'authored', text: ' Truoc: ' },
      { kind: 'glossary', text: 'dragon → rong\ncastle → lau dai' },
      { kind: 'source_segment', text: 'A dragon roared.' },
      { kind: 'tm', text: '[TM: cau tuong tu]' },
      { kind: 'authored', text: '\nSau. ' },
    ]
    const prompt = pieces.map((p) => p.text).join('')
    readRecordMock.mockResolvedValue(record({ prompt, ledger: { ...record().ledger, pieces } }))
    state.openAiPromptInspector()
    await flushPromises()

    const wrapper = mount(AiPromptInspectorOverlay)
    await wrapper.vm.$nextTick()

    const body = wrapper.get('[data-aip-body-state="present"]')
    // Nối lại đúng `prompt` TỪNG BYTE, kể cả khoảng trắng đầu/cuối -- đối chứng chính của B1,
    // đo bằng `textContent` THÔ, không `.text()` (P12).
    expect(body.element.textContent).toBe(prompt)

    const spans = body.findAll('.aip-piece')
    expect(spans).toHaveLength(5)
    const ALL_KIND_CLASSES = ['aip-piece-glossary', 'aip-piece-source_segment', 'aip-piece-tm']

    expect(spans[0]?.attributes('data-aip-piece-kind')).toBe('authored')
    expect(spans[0]?.element.textContent).toBe(' Truoc: ')
    for (const c of ALL_KIND_CLASSES) expect(spans[0]?.classes()).not.toContain(c)

    expect(spans[1]?.attributes('data-aip-piece-kind')).toBe('glossary')
    expect(spans[1]?.element.textContent).toBe('dragon → rong\ncastle → lau dai')
    expect(spans[1]?.classes()).toContain('aip-piece-glossary')

    expect(spans[2]?.attributes('data-aip-piece-kind')).toBe('source_segment')
    expect(spans[2]?.element.textContent).toBe('A dragon roared.')
    expect(spans[2]?.classes()).toContain('aip-piece-source_segment')
    expect(spans[2]?.classes()).not.toContain('aip-piece-glossary')

    expect(spans[3]?.attributes('data-aip-piece-kind')).toBe('tm')
    expect(spans[3]?.element.textContent).toBe('[TM: cau tuong tu]')
    expect(spans[3]?.classes()).toContain('aip-piece-tm')
    expect(spans[3]?.classes()).not.toContain('aip-piece-glossary')

    expect(spans[4]?.attributes('data-aip-piece-kind')).toBe('authored')
    expect(spans[4]?.element.textContent).toBe('\nSau. ')
    for (const c of ALL_KIND_CLASSES) expect(spans[4]?.classes()).not.toContain(c)

    wrapper.unmount()
  })
})

describe('AiPromptInspectorOverlay.vue — Glossary: not_asked / asked-0 / asked-N qua wire thật', () => {
  it('kind "not_asked" ⇒ data-aip-glossary-kind="not_asked", KHÔNG claim một count nào', async () => {
    const { state, i18n, AiPromptInspectorOverlay } = await freshOverlay()
    readRecordMock.mockResolvedValue(record({ ledger: { ...record().ledger, glossary: NOT_ASKED } }))
    state.openAiPromptInspector()
    await flushPromises()

    const wrapper = mount(AiPromptInspectorOverlay)
    await wrapper.vm.$nextTick()

    const node = wrapper.get('[data-aip-glossary-kind="not_asked"]')
    expect(node.text()).toBe(i18n.t('ai.prompt_inspector.glossary_not_asked'))
    expect(wrapper.find('[data-aip-glossary-kind="asked"]').exists()).toBe(false)

    wrapper.unmount()
  })

  it('kind "asked" với 0 khớp ⇒ data-aip-glossary-kind="asked", summary nói ĐÚNG "0"', async () => {
    const { state, i18n, AiPromptInspectorOverlay } = await freshOverlay()
    readRecordMock.mockResolvedValue(record({ ledger: { ...record().ledger, glossary: asked([]) } }))
    state.openAiPromptInspector()
    await flushPromises()

    const wrapper = mount(AiPromptInspectorOverlay)
    await wrapper.vm.$nextTick()

    const node = wrapper.get('[data-aip-glossary-kind="asked"]')
    expect(node.text()).toBe(i18n.t('ai.prompt.summary_asked', { count: '0' }))
    // 🔴 SỬA finding P12 (loop 2) -- assert cũ `.aip-term-list-injected, .aip-note` là MỘT bộ
    // chọn HỢP (OR): nó xanh bất kể nhánh nào render, kể cả khi phần TM ngay dưới LUÔN có sẵn
    // một `.aip-note` khác ("TM chưa dựng") -- không hề canh rằng dòng "Đã chèn 0 thuật ngữ"
    // THẬT SỰ xuất hiện. Đối chứng lại bằng đúng VĂN BẢN dịch của dòng note đó.
    expect(wrapper.find('.aip-term-list-injected').exists()).toBe(false)
    expect(wrapper.text()).toContain(i18n.t('ai.prompt_inspector.glossary_injected_empty'))

    wrapper.unmount()
  })

  it('kind "asked" với hai thuật ngữ ⇒ hai dòng, ĐÚNG source_term/translation/tier từng dòng', async () => {
    const { state, i18n, AiPromptInspectorOverlay } = await freshOverlay()
    readRecordMock.mockResolvedValue(
      record({
        ledger: {
          ...record().ledger,
          glossary: asked([
            injectedTerm({ source_term: 'dragon', translation: 'rong', tier: 'global' }),
            injectedTerm({ source_term: 'castle', translation: 'lau dai', tier: 'work' }),
          ]),
        },
      }),
    )
    state.openAiPromptInspector()
    await flushPromises()

    const wrapper = mount(AiPromptInspectorOverlay)
    await wrapper.vm.$nextTick()

    const rows = wrapper.findAll('.aip-term-list-injected .aip-term-row')
    expect(rows).toHaveLength(2)

    expect(rows[0]?.get('.aip-term-source').text()).toBe('dragon')
    expect(rows[0]?.get('.aip-term-translation').text()).toBe('rong')
    expect(rows[0]?.get('.aip-term-tier').text()).toBe(i18n.t('ai.prompt_inspector.tier_global'))

    expect(rows[1]?.get('.aip-term-source').text()).toBe('castle')
    expect(rows[1]?.get('.aip-term-translation').text()).toBe('lau dai')
    expect(rows[1]?.get('.aip-term-tier').text()).toBe(i18n.t('ai.prompt_inspector.tier_work'))

    wrapper.unmount()
  })
})

describe('AiPromptInspectorOverlay.vue — danh sách "đã cân nhắc nhưng không chèn" (Decision 3)', () => {
  it('mang tier + lý do CHỜ CHỐT trên MỖI dòng, và KHÔNG một hành động sửa-ngay nào', async () => {
    const { state, i18n, AiPromptInspectorOverlay } = await freshOverlay()
    readRecordMock.mockResolvedValue(
      record({
        ledger: {
          ...record().ledger,
          glossary: asked([], [suppressedTerm({ source_term: 'dog', translation: 'cho', tier: 'work' })]),
        },
      }),
    )
    state.openAiPromptInspector()
    await flushPromises()

    const wrapper = mount(AiPromptInspectorOverlay)
    await wrapper.vm.$nextTick()

    const rows = wrapper.findAll('.aip-term-list-suppressed .aip-term-row')
    expect(rows).toHaveLength(1)
    const row = rows[0]

    expect(row.get('.aip-term-source').text()).toBe('dog')
    expect(row.get('.aip-term-translation').text()).toBe('cho')
    expect(row.get('.aip-term-tier').text()).toBe(i18n.t('ai.prompt_inspector.tier_work'))
    expect(row.get('.aip-term-reason').text()).toBe(
      i18n.t('ai.prompt_inspector.glossary_suppressed_reason_pending_overlap'),
    )

    // Decision 3 — read-only: KHÔNG một <button>/<a> nào trong hàng "đã cân nhắc nhưng không
    // chèn", khác hẳn mockup's fix-it-now action.
    expect(row.findAll('button')).toHaveLength(0)
    expect(row.findAll('a')).toHaveLength(0)
    // Và đối chứng bên NGOÀI toàn danh sách suppressed — không sót một nút nào ở đây nói chung.
    expect(wrapper.find('.aip-term-list-suppressed button').exists()).toBe(false)

    wrapper.unmount()
  })

  it('rỗng ⇒ ghi chú "không có thuật ngữ nào bị loại", không một <ul> nào render', async () => {
    const { state, i18n, AiPromptInspectorOverlay } = await freshOverlay()
    readRecordMock.mockResolvedValue(record({ ledger: { ...record().ledger, glossary: asked([]) } }))
    state.openAiPromptInspector()
    await flushPromises()

    const wrapper = mount(AiPromptInspectorOverlay)
    await wrapper.vm.$nextTick()

    expect(wrapper.find('.aip-term-list-suppressed').exists()).toBe(false)
    expect(wrapper.text()).toContain(i18n.t('ai.prompt_inspector.glossary_suppressed_empty'))

    wrapper.unmount()
  })
})

describe('AiPromptInspectorOverlay.vue — TM: not_built_yet KHÔNG BAO GIỜ đọc thành "0 câu tương tự"', () => {
  it('kind "not_built_yet" ⇒ data-aip-tm-kind="not_built_yet", đúng câu "chưa được dựng"', async () => {
    const { state, i18n, AiPromptInspectorOverlay } = await freshOverlay()
    readRecordMock.mockResolvedValue(record({ ledger: { ...record().ledger, tm: TM_NOT_BUILT } }))
    state.openAiPromptInspector()
    await flushPromises()

    const wrapper = mount(AiPromptInspectorOverlay)
    await wrapper.vm.$nextTick()

    const node = wrapper.get('[data-aip-tm-kind="not_built_yet"]')
    expect(node.text()).toBe(i18n.t('ai.prompt_inspector.tm_not_built_yet'))
    expect(node.text()).not.toBe(i18n.t('ai.prompt_inspector.tm_searched', { count: '0' }))
    expect(wrapper.find('[data-aip-tm-kind="searched"]').exists()).toBe(false)

    wrapper.unmount()
  })

  it('kind "searched" (chưa đường gọi nào của story này tạo được, Epic 7) vẫn render đúng nhánh, không thiếu ca', async () => {
    const { state, i18n, AiPromptInspectorOverlay } = await freshOverlay()
    readRecordMock.mockResolvedValue(
      record({ ledger: { ...record().ledger, tm: { kind: 'searched', similar_segments: [] } } }),
    )
    state.openAiPromptInspector()
    await flushPromises()

    const wrapper = mount(AiPromptInspectorOverlay)
    await wrapper.vm.$nextTick()

    const node = wrapper.get('[data-aip-tm-kind="searched"]')
    expect(node.text()).toBe(i18n.t('ai.prompt_inspector.tm_searched', { count: '0' }))

    wrapper.unmount()
  })
})

describe('AiPromptInspectorOverlay.vue — bản ghi CŨ (Stale record)', () => {
  it('segment_id của bản ghi KHÁC câu đang focus ⇒ [data-aip-stale="true"] hiện, nêu tên cả hai câu', async () => {
    const { state, i18n, AiPromptInspectorOverlay, caretSegmentId } = await freshOverlay()
    readRecordMock.mockResolvedValue(record({ segment_id: 1 }))
    caretSegmentId.value = 2
    state.openAiPromptInspector()
    await flushPromises()

    const wrapper = mount(AiPromptInspectorOverlay)
    await wrapper.vm.$nextTick()

    const stale = wrapper.get('[data-aip-stale="true"]')
    expect(stale.text()).toBe(
      i18n.t('ai.prompt_inspector.stale_notice', { record_segment_id: '1', focused_segment_id: '2' }),
    )

    wrapper.unmount()
  })

  it('segment_id của bản ghi CÙNG câu đang focus ⇒ không render ghi chú cũ (v-if vắng, không chỉ thuộc tính false)', async () => {
    const { state, AiPromptInspectorOverlay, caretSegmentId } = await freshOverlay()
    readRecordMock.mockResolvedValue(record({ segment_id: 1 }))
    caretSegmentId.value = 1
    state.openAiPromptInspector()
    await flushPromises()

    const wrapper = mount(AiPromptInspectorOverlay)
    await wrapper.vm.$nextTick()

    expect(wrapper.find('.aip-stale').exists()).toBe(false)

    wrapper.unmount()
  })
})

// ═══════════════════════════════════════════════════════════════════════════════════
// Decision 2 — mở màn hình KHÔNG BAO GIỜ lắp ráp, qua ĐÚNG CommandRegistry thật
// ═══════════════════════════════════════════════════════════════════════════════════

describe('dispatch("ai.prompt_inspector.open"/"close") — chỉ ĐỌC, không bao giờ lắp ráp', () => {
  it('dispatch open ⇒ gọi ai_prompt_read_record ĐÚNG một lần, aiPromptAssemble KHÔNG BAO GIỜ được gọi', async () => {
    const { commands, state } = await freshOverlay()
    readRecordMock.mockResolvedValue(record())

    expect(state.aiPromptInspectorIsOpen.value).toBe(false)
    commands.dispatch('ai.prompt_inspector.open')
    await flushPromises()
    await flushPromises()

    expect(state.aiPromptInspectorIsOpen.value).toBe(true)
    expect(readRecordMock).toHaveBeenCalledTimes(1)
    expect(assembleMock).not.toHaveBeenCalled()
  })

  it('dispatch close ⇒ đóng lớp phủ, KHÔNG dọn bản ghi, và assemble vẫn không bao giờ được gọi', async () => {
    const { commands, state } = await freshOverlay()
    readRecordMock.mockResolvedValue(record())
    commands.dispatch('ai.prompt_inspector.open')
    await flushPromises()
    await flushPromises()
    expect(state.aiPromptInspectorIsOpen.value).toBe(true)

    commands.dispatch('ai.prompt_inspector.close')

    expect(state.aiPromptInspectorIsOpen.value).toBe(false)
    expect(state.aiPromptRecord.value).not.toBeNull()
    expect(assembleMock).not.toHaveBeenCalled()
  })

  it('mở/đóng lặp lại nhiều lần vẫn không một lượt nào chạm aiPromptAssemble', async () => {
    const { commands } = await freshOverlay()
    readRecordMock.mockResolvedValue(record())

    for (let i = 0; i < 3; i += 1) {
      commands.dispatch('ai.prompt_inspector.open')
      await flushPromises()
      await flushPromises()
      commands.dispatch('ai.prompt_inspector.close')
    }

    expect(readRecordMock).toHaveBeenCalledTimes(3)
    expect(assembleMock).not.toHaveBeenCalled()
  })
})

// ═══════════════════════════════════════════════════════════════════════════════════
// AC3 — "khi nó được rebind qua tầng phím tắt, cả hai đường đều mở màn hình" — đo THẬT,
// không suy từ cấu trúc mã
// ═══════════════════════════════════════════════════════════════════════════════════
//
// 🔴 `ai.prompt_inspector.open`/`.close`/`ai.prompt.assemble` đăng ký `keys: undefined` — cả
// ba KHÔNG có hợp âm mặc định, nên đường bàn phím duy nhất có thể canh ở đây là một lượt
// REBIND của người dùng (`applyBindings`, cùng cửa `ShortcutsOverlay.vue`/`shortcutsState.ts`
// dùng — Story 1.21 AC2/AC12), không phải một `KeyboardEvent` khớp phím mặc định như
// `editorClearSourceCuts.test.ts`'s ca `Escape`. Trước ca này, `applyBindings` không có một
// dòng test nào trong toàn kho — dispatch-bằng-id (nhóm `describe` ngay trên) canh CỔNG
// nhưng bỏ qua quãng đường bàn phím; ca dưới đây gắn `attachKeyboard` THẬT, gọi
// `applyBindings` để rebind `ai.prompt_inspector.open` sang một hợp âm không xung đột với gì
// khác trong registry sản phẩm (`Shift+K`, không `Mod` — tránh nhánh macOS/Windows của
// `detectIsMac()` rẽ khác nhau, cùng lý do file này không cần biết đang chạy trên nền nào),
// rồi bắn đúng `KeyboardEvent` đó — bàn phím → `keys.ts` → registry → `openAiPromptInspector`
// THẬT (không mock).
describe('AC3 — rebind qua tầng phím tắt (Story 1.21) mở ĐÚNG cùng màn hình dispatch-theo-id mở', () => {
  it('🔴 rebind `ai.prompt_inspector.open` sang `Shift+K` rồi bấm ⇒ mở lớp phủ, gọi ai_prompt_read_record đúng một lần', async () => {
    const { commands, state } = await freshOverlay()
    readRecordMock.mockResolvedValue(record())
    expect(state.aiPromptInspectorIsOpen.value).toBe(false)

    const outcome = commands.applyBindings({ 'ai.prompt_inspector.open': ['Shift+K'] })
    expect(outcome.ok).toBe(true) // hợp âm không xung đột với thao tác nào khác của registry thật

    const host = document.createElement('div')
    document.body.appendChild(host)
    const detach = commands.attachKeyboard(host)
    try {
      host.dispatchEvent(
        new KeyboardEvent('keydown', {
          key: 'K',
          code: 'KeyK',
          shiftKey: true,
          bubbles: true,
          cancelable: true,
        }),
      )
      await flushPromises()
      await flushPromises()
    } finally {
      detach()
      host.remove()
    }

    expect(state.aiPromptInspectorIsOpen.value).toBe(true)
    expect(readRecordMock).toHaveBeenCalledTimes(1)
    expect(assembleMock).not.toHaveBeenCalled()
  })

  it('cùng hợp âm rebind, bấm HAI lần liên tiếp ⇒ mở rồi đóng, đúng như dispatch-theo-id — hai đường cùng đích', async () => {
    const { commands, state } = await freshOverlay()
    readRecordMock.mockResolvedValue(record())

    commands.applyBindings({
      'ai.prompt_inspector.open': ['Shift+K'],
      'ai.prompt_inspector.close': ['Shift+L'],
    })

    const host = document.createElement('div')
    document.body.appendChild(host)
    const detach = commands.attachKeyboard(host)
    try {
      host.dispatchEvent(
        new KeyboardEvent('keydown', { key: 'K', code: 'KeyK', shiftKey: true, bubbles: true, cancelable: true }),
      )
      await flushPromises()
      await flushPromises()
      expect(state.aiPromptInspectorIsOpen.value).toBe(true)

      host.dispatchEvent(
        new KeyboardEvent('keydown', { key: 'L', code: 'KeyL', shiftKey: true, bubbles: true, cancelable: true }),
      )
      expect(state.aiPromptInspectorIsOpen.value).toBe(false)
    } finally {
      detach()
      host.remove()
    }

    // Đúng bất biến Decision 2 phải giữ trên CẢ HAI đường kích hoạt (click hay bàn phím) —
    // không một đường lắp ráp nào được mở ra chỉ vì nguồn kích hoạt đổi.
    expect(assembleMock).not.toHaveBeenCalled()
  })

  it('🔴 finding E11 — AC3 vế hai: registry.unbound() KHÔNG phải nguồn UI rebind đọc; effectiveUnbound() mới phản ánh lượt rebind', async () => {
    const { commands } = await freshOverlay()

    // Đăng ký (`registerAll` chạy trong `freshOverlay()`) cho cả ba lệnh Story 4.7 `keys:
    // undefined` — I/O Matrix của `registry.ts` nói `unbound()` (tĩnh, đọc thẳng `spec.keys`
    // LÚC ĐĂNG KÝ) coi chúng là "chưa gán phím" MÃI MÃI, vì một lượt gán không đi qua
    // `register()` lần nữa. `effectiveUnbound()` (runtime, trừ đi `effectiveBindings()`) MỚI
    // là hàm `index.ts`'s doc-comment (⚠️ "KHÔNG đọc `commandRegistry.unbound()`... từ màn
    // hình") chỉ đích danh là nguồn UI rebind phải đọc.
    const beforeIds = commands.effectiveUnbound().map((s) => s.id)
    expect(beforeIds).toContain('ai.prompt_inspector.open')

    commands.applyBindings({ 'ai.prompt_inspector.open': ['Shift+K'] })

    const afterIds = commands.effectiveUnbound().map((s) => s.id)
    expect(afterIds).not.toContain('ai.prompt_inspector.open')
    // Hai lệnh CÒN LẠI chưa rebind vẫn phải đứng nguyên trong danh sách -- đối chứng không
    // phải mọi thứ đột nhiên "có phím" sau MỘT lượt `applyBindings`.
    expect(afterIds).toContain('ai.prompt_inspector.close')
    expect(afterIds).toContain('ai.prompt.assemble')
  })
})

// ═══════════════════════════════════════════════════════════════════════════════════
// SỬA 2026-09-18 — `resetAiPromptInspector()` chưa từng được canh THẬT SỰ zero cả sáu ô nhớ
// (Blind Hunter bắt được: hàm này chỉ tồn tại để thoả `check:panel-refs`, không một ca nào
// đọc lại state SAU khi gọi nó — cùng khuôn `resetPromptSets`/`resetPromptLibrary` vốn cũng
// chỉ được GỌI giữa các ca, không có ca riêng khẳng định state THẬT SỰ về không; ca dưới đây
// đóng khoảng đó cho `resetAiPromptInspector`).
//
// 🔴 SỬA finding B14 (loop 1) — "năm ô nhớ"/"bốn giá trị" ở đây đã CŨ kể từ khi
// [`readError`] (ô nhớ THỨ SÁU, findings B4/E3/E10) được thêm vào `aiPromptInspectorState.ts`:
// `overlayOpen`/`record`/`assembleBusy`/`assembleError`/`sequence`/`readError` — sequence
// không có một `Ref` xuất khẩu để đọc lại (nó chỉ là một số nguyên module-private), nên NĂM
// giá trị QUAN SÁT ĐƯỢC (không phải sáu) là thứ ca dưới đây khẳng định.
// ═══════════════════════════════════════════════════════════════════════════════════
describe('resetAiPromptInspector — cả sáu ô nhớ THẬT SỰ về trạng thái rỗng', () => {
  it('🔴 lớp phủ đang mở, có bản ghi, có lỗi Lắp, có lỗi Đọc ⇒ reset đưa cả năm giá trị quan sát được về rỗng', async () => {
    const { state } = await freshOverlay()
    readRecordMock.mockResolvedValue(record({ segment_id: 9 }))
    state.openAiPromptInspector()
    await flushPromises()
    expect(state.aiPromptInspectorIsOpen.value).toBe(true)
    expect(state.aiPromptRecord.value).not.toBeNull()

    const err: IpcError = { code: 'x', message_key: 'err.unknown', params: {}, retryable: false }
    assembleMock.mockResolvedValue({ value: null, error: err })
    await state.assembleCurrentAiPrompt('Set', 1)
    expect(state.aiPromptAssembleError.value).not.toBeNull()

    readRecordMock.mockResolvedValueOnce(readError(err))
    await state.refreshAiPromptRecord()
    expect(state.aiPromptReadError.value).not.toBeNull()

    state.resetAiPromptInspector()

    expect(state.aiPromptInspectorIsOpen.value).toBe(false)
    expect(state.aiPromptRecord.value).toBeNull()
    expect(state.aiPromptAssembleBusy.value).toBe(false)
    expect(state.aiPromptAssembleError.value).toBeNull()
    expect(state.aiPromptReadError.value).toBeNull()
  })
})

// ═══════════════════════════════════════════════════════════════════════════════════
// SỬA 2026-09-18 — `assembleBusy` không được TREO MÃI khi một lượt Đọc chen vào giữa lúc
// đang Lắp (bắt được ở lượt rà soát build, không phải một suy diễn)
// ═══════════════════════════════════════════════════════════════════════════════════
//
// 🔴 Trước bản sửa: `assembleCurrentAiPrompt` chỉ hạ `assembleBusy` khi `mine === sequence`
// — nhưng `sequence` DÙNG CHUNG với `refreshAiPromptRecord` (không đụng `assembleBusy`). Một
// lượt Đọc (mở lớp phủ, hoặc panel dockview mount lại — cả hai đều gọi `refreshAiPromptRecord`)
// chen vào TRƯỚC khi lượt Lắp đang chạy trả lời sẽ nâng `sequence`, khiến nhánh "vượt mặt"
// bỏ luôn dòng hạ cờ — không còn ai hạ nó nữa, nút "Lắp prompt cho câu này" bị khoá vĩnh viễn
// hết phiên. Ca dưới đây dựng ĐÚNG kịch bản chen ngang đó bằng một `Promise` tự điều khiển
// thời điểm trả lời, không suy từ đọc mã.
describe('SỬA đua tranh sequence — assembleBusy không treo mãi khi một lượt Đọc chen vào', () => {
  it('🔴 refreshAiPromptRecord() chen vào TRƯỚC khi Lắp trả lời ⇒ Lắp trả lời xong vẫn phải hạ assembleBusy về false', async () => {
    const { state } = await freshOverlay()

    type AssembleResult = { value: null; error: IpcError | null }
    // Định nghĩa vỏ mặc định TRƯỚC (không `null`) — tránh đúng cạm bẫy suy kiểu của TS khi một
    // biến kiểu hàm bị GÁN LẠI bên trong một closure: phân tích luồng điều khiển của `tsc` không
    // theo dõi được lượt gán đó, và một khai `T | null` ở đây từng khiến `npm run build` báo
    // "Type 'never' has no call signatures" tại chỗ gọi — bắt được thật lúc `npm run build`,
    // không phải một đoán trước.
    let resolveAssemble: (result: AssembleResult) => void = () => {
      throw new Error('resolveAssemble chưa được gán — assembleMock chưa gọi tới executor')
    }
    assembleMock.mockImplementation(
      () =>
        new Promise<AssembleResult>((resolve) => {
          resolveAssemble = resolve
        }),
    )
    readRecordMock.mockResolvedValue(record())

    const assemblePromise = state.assembleCurrentAiPrompt('Set', 1)
    await flushPromises()
    expect(state.aiPromptAssembleBusy.value).toBe(true)

    // Lượt Đọc chen vào TRƯỚC khi Lắp trả lời — panel dockview mount lại, hoặc mở lớp phủ
    // trong khi lượt Lắp còn đang chạy. Đây là dòng KHÔNG có trong bản trước bản sửa mà nếu
    // xoá đi, ca này không còn canh được khuyết tật đã bắt.
    await state.refreshAiPromptRecord()

    // Lắp trả lời xong SAU lượt Đọc chen ngang.
    resolveAssemble({ value: null, error: null })
    await assemblePromise

    expect(state.aiPromptAssembleBusy.value).toBe(false)
  })
})

// ═══════════════════════════════════════════════════════════════════════════════════
// SỬA 2026-09-18 — nút "Lắp prompt cho câu này" (`AiTranslationPanel.vue`) và nhịp GHI
// `assembleCurrentAiPrompt`/`ai.prompt.assemble` không có ca nào canh trước bản sửa
// (verification-gap layer bắt được: nhịp GHI DUY NHẤT của Decision 2 không một dòng test)
// ═══════════════════════════════════════════════════════════════════════════════════
describe('AiTranslationPanel.vue — nút "Lắp prompt cho câu này" (nhịp GHI của Decision 2)', () => {
  it('🔴 click ⇒ gọi aiPromptAssemble ĐÚNG với bộ hiệu lực + câu đang focus, rồi ghi kết quả vào aiPromptRecord', async () => {
    const { state, promptSetState, AiTranslationPanel, caretSegmentId } = await freshPanel()
    caretSegmentId.value = 7
    assembleMock.mockResolvedValue({ value: record({ segment_id: 7, prompt_set_name: 'Happy' }), error: null })

    const wrapper = mount(AiTranslationPanel, { props: { params: { params: {} } }, attachTo: document.body })
    await flushPromises()
    await wrapper.vm.$nextTick()
    // `setSelectedPromptSetName` chỉ chấp nhận một tên ĐÃ CÓ trong `resolvedSets` — gọi SAU
    // khi `onMounted`'s `loadPromptSets()` (đã mock `promptSetListMock` ở `freshPanel()`) trả
    // lời, không trước (xem doc-comment `promptSetListMock`).
    promptSetState.setSelectedPromptSetName('Happy')
    await wrapper.vm.$nextTick()

    const button = wrapper.get('[data-ai-prompt-assemble]')
    expect(button.attributes('disabled')).toBeUndefined()
    await button.trigger('click')
    await wrapper.vm.$nextTick()

    expect(assembleMock).toHaveBeenCalledTimes(1)
    expect(assembleMock).toHaveBeenCalledWith('Happy', 7)
    expect(state.aiPromptRecord.value?.segment_id).toBe(7)
    expect(state.aiPromptRecord.value?.prompt_set_name).toBe('Happy')

    wrapper.unmount()
  })

  it('không câu nào đang focus ⇒ nút bị `disabled`, gợi ý "Chọn một câu" hiện, và click vẫn KHÔNG gọi aiPromptAssemble (lưới phòng thủ thứ hai)', async () => {
    const { state, AiTranslationPanel, caretSegmentId } = await freshPanel()
    caretSegmentId.value = null

    const wrapper = mount(AiTranslationPanel, { props: { params: { params: {} } }, attachTo: document.body })
    await wrapper.vm.$nextTick()

    const button = wrapper.get('[data-ai-prompt-assemble]')
    expect(button.attributes('disabled')).toBeDefined()
    expect(wrapper.find('.ai-inspector-hint').exists()).toBe(true)

    // Đúng lưới phòng thủ HAI LỚP doc-comment `assembleCurrentAiPrompt` ghi: nút bị khoá là
    // lớp THỨ NHẤT; gọi thẳng hàm THẬT (bỏ qua lớp khoá, cùng thể hiện module `state` mà
    // `freshPanel()` đã nạp cho chính component này) để canh lớp THỨ HAI cũng không gọi
    // xuống adapter.
    await state.assembleCurrentAiPrompt('Happy', null)
    expect(assembleMock).not.toHaveBeenCalled()

    wrapper.unmount()
  })

  it('lượt Lắp trượt (IpcError) ⇒ hiện `.ai-inspector-alert`, KHÔNG ghi đè aiPromptRecord, và cờ bận hạ về false', async () => {
    const { state, promptSetState, AiTranslationPanel, caretSegmentId } = await freshPanel()
    promptSetState.setSelectedPromptSetName(null)
    caretSegmentId.value = 3
    const err: IpcError = {
      code: 'ai_prompt.no_set_selected',
      message_key: 'err.ai_prompt.no_set_selected',
      params: {},
      retryable: false,
    }
    assembleMock.mockResolvedValue({ value: null, error: err })

    const wrapper = mount(AiTranslationPanel, { props: { params: { params: {} } }, attachTo: document.body })
    await wrapper.vm.$nextTick()

    // 🔴 SỬA finding B3 (loop 1) — tên ca khẳng định "KHÔNG ghi đè aiPromptRecord" nhưng bản
    // trước KHÔNG một lần đọc `state.aiPromptRecord` — không thể đỏ dù `assembleCurrentAiPrompt`
    // có ghi `record.value = ...` ở nhánh lỗi hay không. Dựng SẴN một bản ghi TỐT trước khi
    // click, để một lượt ghi đè (bug thật) có thứ để XOÁ mất — nếu không, "vẫn null sau khi
    // lỗi" cũng xanh dù record CHƯA từng được bảo vệ khỏi ghi đè.
    await state.refreshAiPromptRecord()
    readRecordMock.mockResolvedValueOnce(record({ segment_id: 3 }))
    await state.refreshAiPromptRecord()
    expect(state.aiPromptRecord.value?.segment_id).toBe(3)

    await wrapper.get('[data-ai-prompt-assemble]').trigger('click')
    await wrapper.vm.$nextTick()

    expect(wrapper.find('.ai-inspector-alert').exists()).toBe(true)
    expect(wrapper.get('[data-ai-prompt-assemble]').attributes('disabled')).toBeUndefined()
    expect(state.aiPromptRecord.value?.segment_id).toBe(3)

    wrapper.unmount()
  })

  it('🔴 SỬA — lỗi Lắp của câu A vẫn hiện sau khi tiêu điểm dời sang câu B chưa hề Lắp ⇒ phải biến mất, không đọc như lỗi của câu B', async () => {
    const { promptSetState, AiTranslationPanel, caretSegmentId } = await freshPanel()
    promptSetState.setSelectedPromptSetName(null)
    caretSegmentId.value = 3
    const err: IpcError = {
      code: 'ai_prompt.no_set_selected',
      message_key: 'err.ai_prompt.no_set_selected',
      params: {},
      retryable: false,
    }
    assembleMock.mockResolvedValue({ value: null, error: err })

    const wrapper = mount(AiTranslationPanel, { props: { params: { params: {} } }, attachTo: document.body })
    await wrapper.vm.$nextTick()

    await wrapper.get('[data-ai-prompt-assemble]').trigger('click')
    await wrapper.vm.$nextTick()
    expect(wrapper.find('.ai-inspector-alert').exists()).toBe(true)

    // Tiêu điểm dời sang một câu KHÁC — người dùng chưa hề bấm Lắp lại cho câu này.
    caretSegmentId.value = 4
    await wrapper.vm.$nextTick()

    expect(wrapper.find('.ai-inspector-alert').exists()).toBe(false)

    wrapper.unmount()
  })

  it('cờ bận: nút đổi nhãn thành "Đang lắp ráp…" và bị `disabled` NGAY khi click, trước khi lượt gọi trả lời', async () => {
    const { AiTranslationPanel, caretSegmentId, i18n } = await freshPanel()
    caretSegmentId.value = 1

    type AssembleResult = { value: null; error: null }
    let resolveAssemble: (result: AssembleResult) => void = () => {
      throw new Error('resolveAssemble chưa được gán')
    }
    assembleMock.mockImplementation(() => new Promise<AssembleResult>((resolve) => (resolveAssemble = resolve)))

    const wrapper = mount(AiTranslationPanel, { props: { params: { params: {} } }, attachTo: document.body })
    await wrapper.vm.$nextTick()

    await wrapper.get('[data-ai-prompt-assemble]').trigger('click')

    const button = wrapper.get('[data-ai-prompt-assemble]')
    expect(button.attributes('disabled')).toBeDefined()
    expect(button.text()).toBe(i18n.t('panel.ai_translation.assemble_busy'))

    resolveAssemble({ value: null, error: null })
    await wrapper.vm.$nextTick()
    await wrapper.vm.$nextTick()

    expect(wrapper.get('[data-ai-prompt-assemble]').text()).toBe(i18n.t('command.ai.prompt.assemble'))

    wrapper.unmount()
  })
})

// ═══════════════════════════════════════════════════════════════════════════════════
// Loop 1 (2026-09-18) — findings B4/E2/E3/E4/E10: "một lượt Đọc/Lắp TRƯỢT không được phép
// xoá một bản ghi TỐT đã có, và một lượt Lắp TRƯỢT-mà-bị-vượt-mặt vẫn phải báo cáo được"
// ═══════════════════════════════════════════════════════════════════════════════════
describe('findings B4/E3/E10 — refreshAiPromptRecord() giữ NGUYÊN bản ghi cũ khi lượt Đọc trượt', () => {
  it('🔴 đọc thành công trước, đọc trượt sau ⇒ record giữ nguyên, readError mang lỗi', async () => {
    const { state } = await freshOverlay()

    readRecordMock.mockResolvedValueOnce(record({ segment_id: 7 }))
    await state.refreshAiPromptRecord()
    expect(state.aiPromptRecord.value?.segment_id).toBe(7)
    expect(state.aiPromptReadError.value).toBeNull()

    const err: IpcError = { code: 'ipc.unknown', message_key: 'err.unknown', params: {}, retryable: false }
    readRecordMock.mockResolvedValueOnce(readError(err))
    await state.refreshAiPromptRecord()

    // Bản ghi CŨ (segment_id 7) vẫn còn nguyên — KHÔNG bị `null` của lượt trượt xoá mất.
    expect(state.aiPromptRecord.value?.segment_id).toBe(7)
    expect(state.aiPromptReadError.value).toEqual(err)
  })

  it('🔴 đọc trượt NGAY LẦN ĐẦU (chưa từng có bản ghi) ⇒ record vẫn null, nhưng readError mang lỗi (khác "chưa lắp lần nào")', async () => {
    const { state } = await freshOverlay()

    const err: IpcError = { code: 'ipc.unknown', message_key: 'err.unknown', params: {}, retryable: false }
    readRecordMock.mockResolvedValueOnce(readError(err))
    await state.refreshAiPromptRecord()

    expect(state.aiPromptRecord.value).toBeNull()
    expect(state.aiPromptReadError.value).toEqual(err)
  })
})

describe('finding E2 — assembleCurrentAiPrompt() không ghi đè record bằng { value: null, error: null } (không có cầu Tauri)', () => {
  it('🔴 một bản ghi TỐT đã có ⇒ một lượt Lắp "không gọi được IPC" không được xoá nó', async () => {
    const { state } = await freshOverlay()

    readRecordMock.mockResolvedValueOnce(record({ segment_id: 3 }))
    await state.refreshAiPromptRecord()
    expect(state.aiPromptRecord.value?.segment_id).toBe(3)

    // `{ value: null, error: null }` -- đúng hình dạng `config/aiprompt.ts::aiPromptAssemble`
    // trả khi KHÔNG có cầu Tauri (xem doc-comment tại đó), không phải một lỗi.
    assembleMock.mockResolvedValueOnce({ value: null, error: null })
    await state.assembleCurrentAiPrompt('Set', 1)

    expect(state.aiPromptRecord.value?.segment_id).toBe(3)
    expect(state.aiPromptAssembleError.value).toBeNull()
  })
})

describe('finding E4 — một lượt Lắp TRƯỢT vẫn phải báo lỗi dù bị một lượt Đọc vượt mặt TOÀN CỤC', () => {
  it('🔴 Đọc chen vào SAU khi Lắp đã gửi đi nhưng TRƯỚC khi Lắp trả lời lỗi ⇒ assembleError vẫn được ghi', async () => {
    const { state } = await freshOverlay()

    type AssembleResult = { value: null; error: IpcError | null }
    let resolveAssemble: (result: AssembleResult) => void = () => {
      throw new Error('resolveAssemble chưa được gán — assembleMock chưa gọi tới executor')
    }
    assembleMock.mockImplementation(
      () =>
        new Promise<AssembleResult>((resolve) => {
          resolveAssemble = resolve
        }),
    )
    readRecordMock.mockResolvedValue(record())

    const assemblePromise = state.assembleCurrentAiPrompt('Set', 1)
    await flushPromises()
    expect(state.aiPromptAssembleBusy.value).toBe(true)

    // Một lượt ĐỌC (không phải một lượt Lắp MỚI HƠN) chen vào TRƯỚC khi Lắp trả lời — nâng
    // `sequence` (vượt mặt TOÀN CỤC) nhưng KHÔNG đụng `latestAssembleSequence`.
    await state.refreshAiPromptRecord()

    const err: IpcError = { code: 'ipc.unknown', message_key: 'err.unknown', params: {}, retryable: false }
    resolveAssemble({ value: null, error: err })
    await assemblePromise

    // Trước bản sửa: `if (mine !== sequence) return` đứng TRƯỚC nhánh đọc lỗi, nên lượt Đọc
    // chen ngang ở trên làm nhánh này thoát SỚM — lỗi không bao giờ được ghi. Sau bản sửa:
    // `stillLatestAssemble` (không bị lượt Đọc đụng tới) quyết định quyền ghi lỗi.
    expect(state.aiPromptAssembleError.value).toEqual(err)
    expect(state.aiPromptAssembleBusy.value).toBe(false)
  })
})

// ═══════════════════════════════════════════════════════════════════════════════════
// Loop 1 (2026-09-18) — finding B6: dòng tóm tắt của panel phải tự đánh dấu khi bản ghi
// nó đọc là CŨ (I/O Matrix "Stale record"), không chỉ lớp phủ mới biết điều đó.
// ═══════════════════════════════════════════════════════════════════════════════════
describe('finding B6 — AiTranslationPanel.vue đánh dấu dòng tóm tắt khi bản ghi CŨ', () => {
  it('🔴 bản ghi thuộc câu 7, tiêu điểm đang ở câu 9 ⇒ data-ai-prompt-summary-stale="true" + dòng cảnh báo hiện', async () => {
    const { AiTranslationPanel, caretSegmentId } = await freshPanel()
    caretSegmentId.value = 9
    readRecordMock.mockResolvedValue(record({ segment_id: 7 }))

    const wrapper = mount(AiTranslationPanel, { props: { params: { params: {} } }, attachTo: document.body })
    await flushPromises()
    await wrapper.vm.$nextTick()

    const summary = wrapper.get('.ai-inspector-summary')
    expect(summary.attributes('data-ai-prompt-summary-stale')).toBe('true')
    expect(wrapper.find('[data-ai-prompt-summary-stale-notice]').exists()).toBe(true)

    wrapper.unmount()
  })

  it('bản ghi và tiêu điểm CÙNG một câu ⇒ không đánh dấu cũ, không dòng cảnh báo', async () => {
    const { AiTranslationPanel, caretSegmentId } = await freshPanel()
    caretSegmentId.value = 7
    readRecordMock.mockResolvedValue(record({ segment_id: 7 }))

    const wrapper = mount(AiTranslationPanel, { props: { params: { params: {} } }, attachTo: document.body })
    await flushPromises()
    await wrapper.vm.$nextTick()

    const summary = wrapper.get('.ai-inspector-summary')
    expect(summary.attributes('data-ai-prompt-summary-stale')).toBeUndefined()
    expect(wrapper.find('[data-ai-prompt-summary-stale-notice]').exists()).toBe(false)

    wrapper.unmount()
  })
})

// ═══════════════════════════════════════════════════════════════════════════════════
// finding B15 (loop 1) — lớp phủ MỚI không kế thừa khoảng hở a11y của cả họ overlay.
// ═══════════════════════════════════════════════════════════════════════════════════
describe('finding B15 — .aip-panel mang aria-labelledby trỏ đúng .aip-title', () => {
  it('🔴 role="dialog" có tên truy cập được qua aria-labelledby, không phơi trống', async () => {
    const { state, AiPromptInspectorOverlay } = await freshOverlay()
    readRecordMock.mockResolvedValue(record())
    state.openAiPromptInspector()
    await flushPromises()

    const wrapper = mount(AiPromptInspectorOverlay)
    await wrapper.vm.$nextTick()

    const panel = wrapper.get('.aip-panel')
    const labelledBy = panel.attributes('aria-labelledby')
    expect(labelledBy).toBeTruthy()
    const titleId = labelledBy ?? ''
    expect(wrapper.find(`#${titleId}`).exists()).toBe(true)
    expect(wrapper.get('.aip-title').attributes('id')).toBe(titleId)

    wrapper.unmount()
  })
})

// ═══════════════════════════════════════════════════════════════════════════════════
// finding P6 (loop 2) — `.aip-read-error` (dòng lỗi của lượt ĐỌC) CHƯA từng được MOUNT bởi
// bất kỳ ca nào trước bản sửa này. Năm phép so sánh của findings B4/E3/E10 (loop 1) chỉ đọc
// `state.aiPromptReadError.value` — chưa một ca nào chạm DOM thật của lớp phủ. Ca dưới đây
// mount THẬT, buộc một lượt Đọc trượt qua helper [`readError`], rồi buộc một lượt Đọc TỐT
// ngay sau để chứng minh dòng lỗi biến mất -- không chỉ "xuất hiện", còn "biến mất đúng lúc".
// ═══════════════════════════════════════════════════════════════════════════════════
describe('finding P6 — .aip-read-error thật sự được mount, không chỉ đọc ref', () => {
  it('🔴 lượt Đọc trượt ⇒ [data-aip-read-error="true"] hiện; lượt Đọc TỐT kế tiếp ⇒ biến mất', async () => {
    const { state, AiPromptInspectorOverlay } = await freshOverlay()
    const err: IpcError = { code: 'ipc.unknown', message_key: 'err.unknown', params: {}, retryable: false }
    readRecordMock.mockResolvedValueOnce(readError(err))
    state.openAiPromptInspector()
    await flushPromises()

    const wrapper = mount(AiPromptInspectorOverlay)
    await wrapper.vm.$nextTick()

    expect(wrapper.find('[data-aip-read-error="true"]').exists()).toBe(true)

    readRecordMock.mockResolvedValueOnce(record())
    await state.refreshAiPromptRecord()
    await wrapper.vm.$nextTick()

    expect(wrapper.find('[data-aip-read-error="true"]').exists()).toBe(false)

    wrapper.unmount()
  })
})

// ═══════════════════════════════════════════════════════════════════════════════════
// finding B2 (loop 1) — hai hàng ma trận I/O chưa có ca phía MÀN HÌNH: `unknown_markers`
// và `source_segment_missing` chỉ xuất hiện làm giá trị mặc định rỗng/false của fixture,
// chưa từng bị ĐÈ để `.aip-warn`/`.aip-marker-list` thật sự được mount và canh.
// ═══════════════════════════════════════════════════════════════════════════════════
describe('finding B2 — unknown_markers và source_segment_missing thật sự render', () => {
  it('🔴 source_segment_missing: true ⇒ .aip-warn hiện; false ⇒ không hiện', async () => {
    const { state, AiPromptInspectorOverlay } = await freshOverlay()
    readRecordMock.mockResolvedValue(record({ ledger: { ...record().ledger, source_segment_missing: true } }))
    state.openAiPromptInspector()
    await flushPromises()

    const wrapper = mount(AiPromptInspectorOverlay)
    await wrapper.vm.$nextTick()
    expect(wrapper.find('.aip-warn').exists()).toBe(true)
    wrapper.unmount()
  })

  it('🔴 unknown_markers không rỗng ⇒ .aip-marker-list liệt kê ĐÚNG toàn văn từng ký hiệu lạ', async () => {
    const { state, AiPromptInspectorOverlay } = await freshOverlay()
    readRecordMock.mockResolvedValue(
      record({ ledger: { ...record().ledger, unknown_markers: ['{{chapter_context}}', '{{foo}}'] } }),
    )
    state.openAiPromptInspector()
    await flushPromises()

    const wrapper = mount(AiPromptInspectorOverlay)
    await wrapper.vm.$nextTick()

    expect(wrapper.find('.aip-markers').exists()).toBe(true)
    const codes = wrapper.findAll('.aip-marker-list code')
    expect(codes.map((c) => c.text())).toEqual(['{{chapter_context}}', '{{foo}}'])

    wrapper.unmount()
  })

  it('cả hai đều "sạch" (mặc định) ⇒ KHÔNG .aip-warn, KHÔNG .aip-markers', async () => {
    const { state, AiPromptInspectorOverlay } = await freshOverlay()
    readRecordMock.mockResolvedValue(record())
    state.openAiPromptInspector()
    await flushPromises()

    const wrapper = mount(AiPromptInspectorOverlay)
    await wrapper.vm.$nextTick()

    expect(wrapper.find('.aip-warn').exists()).toBe(false)
    expect(wrapper.find('.aip-markers').exists()).toBe(false)

    wrapper.unmount()
  })
})

// ═══════════════════════════════════════════════════════════════════════════════════
// finding V5 (loop 1) — dòng tóm tắt của panel CHƯA từng được canh Ở TẦNG MOUNT qua cả ba
// trạng thái; `glossaryInjectionSummary` (hàm thuần) và `.aip-summary` (overlay) đã có ca,
// nhưng `.ai-inspector-summary` (panel) thì chưa — một phần tử KHÁC, một component KHÁC.
// ═══════════════════════════════════════════════════════════════════════════════════
describe('finding V5 — AiTranslationPanel.vue mount dòng tóm tắt qua cả ba trạng thái', () => {
  it('🔴 no_record ⇒ kind="no_record" + đúng văn bản summary_no_record', async () => {
    const { AiTranslationPanel } = await freshPanel()
    readRecordMock.mockResolvedValue(null)

    const wrapper = mount(AiTranslationPanel, { props: { params: { params: {} } }, attachTo: document.body })
    await flushPromises()
    await wrapper.vm.$nextTick()

    const summary = wrapper.get('.ai-inspector-summary')
    expect(summary.attributes('data-ai-prompt-summary-kind')).toBe('no_record')
    wrapper.unmount()
  })

  it('🔴 not_asked ⇒ kind="not_asked"', async () => {
    const { AiTranslationPanel } = await freshPanel()
    readRecordMock.mockResolvedValue(record({ ledger: { ...record().ledger, glossary: NOT_ASKED } }))

    const wrapper = mount(AiTranslationPanel, { props: { params: { params: {} } }, attachTo: document.body })
    await flushPromises()
    await wrapper.vm.$nextTick()

    const summary = wrapper.get('.ai-inspector-summary')
    expect(summary.attributes('data-ai-prompt-summary-kind')).toBe('not_asked')
    wrapper.unmount()
  })

  it('🔴 asked với hai thuật ngữ ⇒ kind="asked" + văn bản mang đúng số đếm 2', async () => {
    const { AiTranslationPanel, i18n } = await freshPanel()
    readRecordMock.mockResolvedValue(
      record({ ledger: { ...record().ledger, glossary: asked([injectedTerm(), injectedTerm({ source_term: 'castle' })]) } }),
    )

    const wrapper = mount(AiTranslationPanel, { props: { params: { params: {} } }, attachTo: document.body })
    await flushPromises()
    await wrapper.vm.$nextTick()

    const summary = wrapper.get('.ai-inspector-summary')
    expect(summary.attributes('data-ai-prompt-summary-kind')).toBe('asked')
    expect(summary.text()).toBe(i18n.t('ai.prompt.summary_asked', { count: '2' }))
    wrapper.unmount()
  })
})
