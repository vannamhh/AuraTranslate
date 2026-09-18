/**
 * Guard lúc chạy của `src/config/aiprompt.ts` — Story 4.7 loop 2, finding P2 (task 6-2).
 *
 * ─────────────────────────────────────────────────────────────────────────────
 * ⚠️ PHẠM VI — mock `@tauri-apps/api/core` Ở ĐÚNG BIÊN IPC, khuôn `glossaryConfigGuards.test.ts`
 * ─────────────────────────────────────────────────────────────────────────────
 * `tests/frontend/aiPromptInspector.test.ts` mock TRỌN `../../src/config/aiprompt` (đúng cho
 * mục đích của tệp đó: canh state/màn hình, không canh biên IPC) — nên toàn bộ bảng
 * `{ value, error }` mà loop 1 sửa (findings B4/E2/E3/E10) chưa từng có một ca nào chạy qua
 * MÃ THẬT của `isAssembledPromptWire`/`isInjectionLedgerWire`/… Một ca "mock trọn module là
 * một ca không chạy guard nào" (§Always của spec) — đúng khuyết tật `glossaryConfigGuards.test.ts`'s
 * header đã đặt tên cho `config/glossary.ts`, tái diễn ở đây cho `config/aiprompt.ts`.
 *
 * Tệp này mock ở tầng `invoke()` để bộ guard THẬT của `config/aiprompt.ts` chạy qua từng ca.
 * Mỗi mục mang MỘT ca dữ liệu HỎNG (guard phải từ chối) VÀ MỘT ca dữ liệu ĐÚNG (guard không
 * được nói oan — đối chứng chiều ngược).
 */
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'
import type { IpcError } from '../../src/i18n'

const mockInvoke = vi.fn()
vi.mock('@tauri-apps/api/core', () => ({ invoke: (...args: unknown[]) => mockInvoke(...args) }))

async function freshAdapter() {
  vi.resetModules()
  mockInvoke.mockReset()
  return import('../../src/config/aiprompt')
}

function validPieceWire(over: Record<string, unknown> = {}) {
  return { kind: 'authored', text: 'Terms: ', ...over }
}

function validInjectedTermWire(over: Record<string, unknown> = {}) {
  return { source_term: 'dragon', translation: 'rong', start: 2, end: 8, tier: 'global', ...over }
}

function validSimilarSegmentWire(over: Record<string, unknown> = {}) {
  return { source_text: 'A dog barked.', target_text: 'Mot con cho sua.', ...over }
}

function validLedgerWire(over: Record<string, unknown> = {}) {
  return {
    glossary: { kind: 'asked', injected: [validInjectedTermWire()], suppressed_by_pending_overlap: [] },
    tm: { kind: 'not_built_yet', similar_segments: null },
    unknown_markers: [],
    source_segment_missing: false,
    pieces: [validPieceWire()],
    ...over,
  }
}

/** Một `AssembledPromptWire` hợp lệ trên dây — dùng làm gốc cho các biến thể hỏng. */
function validWire(over: Record<string, unknown> = {}) {
  return {
    prompt: 'Terms: dragon → rong',
    segment_id: 1,
    chapter_id: 10,
    prompt_set_name: 'Happy',
    prompt_set_tier: 'global',
    ledger: validLedgerWire(),
    ...over,
  }
}

beforeEach(() => {
  document.body.innerHTML = ''
})

afterEach(() => {
  Reflect.deleteProperty(window, '__TAURI_INTERNALS__')
})

describe('① isAssembledPromptWire (tầng ngoài cùng) — qua aiPromptAssemble', () => {
  it('segment_id là chuỗi (không phải số) ⇒ value null, error khác null', async () => {
    const state = await freshAdapter()
    mockInvoke.mockResolvedValue(validWire({ segment_id: '1' }))

    const result = await state.aiPromptAssemble('Happy', 1)

    expect(result.value).toBeNull()
    expect(result.error).not.toBeNull()
  })

  it('đối chứng chiều ngược — wire ĐÚNG hình dạng ⇒ error === null, value khớp NGUYÊN VĂN', async () => {
    const state = await freshAdapter()
    const wire = validWire()
    mockInvoke.mockResolvedValue(wire)

    const result = await state.aiPromptAssemble('Happy', 1)

    expect(result.error).toBeNull()
    expect(result.value).toEqual(wire)
  })
})

describe('② isPromptSetTier / isGlossaryTier — chuỗi tag phải là "global"/"work"', () => {
  it('prompt_set_tier: "shared" (tag lạ) ⇒ value null, error khác null', async () => {
    const state = await freshAdapter()
    mockInvoke.mockResolvedValue(validWire({ prompt_set_tier: 'shared' }))

    const result = await state.aiPromptAssemble('Happy', 1)

    expect(result.value).toBeNull()
    expect(result.error).not.toBeNull()
  })

  it('đối chứng chiều ngược — prompt_set_tier: "work" ⇒ error === null', async () => {
    const state = await freshAdapter()
    mockInvoke.mockResolvedValue(validWire({ prompt_set_tier: 'work' }))

    const result = await state.aiPromptAssemble('Happy', 1)

    expect(result.error).toBeNull()
    expect(result.value?.prompt_set_tier).toBe('work')
  })
})

describe('③ isInjectedGlossaryTermWire / isSuppressedGlossaryTermWire — đủ NĂM trường', () => {
  it('một mục injected thiếu `end` ⇒ value null, error khác null (bỏ sót một trường làm hỏng CẢ bản ghi)', async () => {
    const state = await freshAdapter()
    const injected = [{ source_term: 'dragon', translation: 'rong', start: 2, tier: 'global' }]
    mockInvoke.mockResolvedValue(
      validWire({ ledger: validLedgerWire({ glossary: { kind: 'asked', injected, suppressed_by_pending_overlap: [] } }) }),
    )

    const result = await state.aiPromptAssemble('Happy', 1)

    expect(result.value).toBeNull()
    expect(result.error).not.toBeNull()
  })

  it('đối chứng chiều ngược — đủ năm trường trên MỘT mục suppressed ⇒ error === null', async () => {
    const state = await freshAdapter()
    const suppressed = [validInjectedTermWire({ source_term: 'castle', tier: 'work' })]
    mockInvoke.mockResolvedValue(
      validWire({ ledger: validLedgerWire({ glossary: { kind: 'asked', injected: [], suppressed_by_pending_overlap: suppressed } }) }),
    )

    const result = await state.aiPromptAssemble('Happy', 1)

    expect(result.error).toBeNull()
    expect(result.value?.ledger.glossary.kind).toBe('asked')
  })
})

describe('④ isGlossaryInjectionStatusWire — BA GIÁ TRỊ không được collapse', () => {
  it('kind: "not_asked" nhưng injected: [] (KHÔNG phải null, phá bất biến ba-trạng-thái) ⇒ error khác null', async () => {
    const state = await freshAdapter()
    mockInvoke.mockResolvedValue(
      validWire({ ledger: validLedgerWire({ glossary: { kind: 'not_asked', injected: [], suppressed_by_pending_overlap: null } }) }),
    )

    const result = await state.aiPromptAssemble('Happy', 1)

    expect(result.value).toBeNull()
    expect(result.error).not.toBeNull()
  })

  it('đối chứng chiều ngược — kind: "not_asked" với CẢ HAI payload null (đúng bất biến) ⇒ error === null', async () => {
    const state = await freshAdapter()
    mockInvoke.mockResolvedValue(
      validWire({ ledger: validLedgerWire({ glossary: { kind: 'not_asked', injected: null, suppressed_by_pending_overlap: null } }) }),
    )

    const result = await state.aiPromptAssemble('Happy', 1)

    expect(result.error).toBeNull()
    expect(result.value?.ledger.glossary.kind).toBe('not_asked')
  })
})

describe('⑤ isTmInjectionStatusWire / isSimilarSegmentWire', () => {
  it('kind: "searched" nhưng similar_segments: null (không phải mảng) ⇒ error khác null', async () => {
    const state = await freshAdapter()
    mockInvoke.mockResolvedValue(validWire({ ledger: validLedgerWire({ tm: { kind: 'searched', similar_segments: null } }) }))

    const result = await state.aiPromptAssemble('Happy', 1)

    expect(result.value).toBeNull()
    expect(result.error).not.toBeNull()
  })

  it('đối chứng chiều ngược — kind: "searched" với một mảng SimilarSegmentWire hợp lệ ⇒ error === null', async () => {
    const state = await freshAdapter()
    mockInvoke.mockResolvedValue(
      validWire({ ledger: validLedgerWire({ tm: { kind: 'searched', similar_segments: [validSimilarSegmentWire()] } }) }),
    )

    const result = await state.aiPromptAssemble('Happy', 1)

    expect(result.error).toBeNull()
    expect(result.value?.ledger.tm.kind).toBe('searched')
  })
})

describe('⑥ isPromptPieceKindWire / isPromptPieceWire — bốn nhãn, không hơn', () => {
  it('một piece mang kind lạ ("injected", không phải một trong bốn nhãn thật) ⇒ error khác null', async () => {
    const state = await freshAdapter()
    mockInvoke.mockResolvedValue(validWire({ ledger: validLedgerWire({ pieces: [validPieceWire({ kind: 'injected' })] }) }))

    const result = await state.aiPromptAssemble('Happy', 1)

    expect(result.value).toBeNull()
    expect(result.error).not.toBeNull()
  })

  it.each(['authored', 'glossary', 'source_segment', 'tm'] as const)(
    'đối chứng chiều ngược — kind: "%s" (một trong bốn nhãn thật) ⇒ error === null',
    async (kind) => {
      const state = await freshAdapter()
      mockInvoke.mockResolvedValue(validWire({ ledger: validLedgerWire({ pieces: [validPieceWire({ kind })] }) }))

      const result = await state.aiPromptAssemble('Happy', 1)

      expect(result.error).toBeNull()
      expect(result.value?.ledger.pieces[0].kind).toBe(kind)
    },
  )
})

describe('⑦ isInjectionLedgerWire — unknown_markers phải là mảng chuỗi', () => {
  it('unknown_markers chứa một số (không phải chuỗi) ⇒ error khác null', async () => {
    const state = await freshAdapter()
    mockInvoke.mockResolvedValue(validWire({ ledger: validLedgerWire({ unknown_markers: ['{{foo}}', 7] }) }))

    const result = await state.aiPromptAssemble('Happy', 1)

    expect(result.value).toBeNull()
    expect(result.error).not.toBeNull()
  })

  it('đối chứng chiều ngược — unknown_markers toàn chuỗi ⇒ error === null', async () => {
    const state = await freshAdapter()
    mockInvoke.mockResolvedValue(validWire({ ledger: validLedgerWire({ unknown_markers: ['{{chapter_context}}'] }) }))

    const result = await state.aiPromptAssemble('Happy', 1)

    expect(result.error).toBeNull()
    expect(result.value?.ledger.unknown_markers).toEqual(['{{chapter_context}}'])
  })
})

// ═══════════════════════════════════════════════════════════════════════════════════
// aiPromptReadRecord — BỐN hình dạng trả về khác nhau (task 6-2: "tất cả bốn nhánh
// aiPromptReadRecord"). Ba trong bốn đi qua CÙNG guard `isAssembledPromptWire` mà các mục
// ①-⑦ ở trên đã canh qua `aiPromptAssemble` — ở đây canh riêng đường ĐỌC vì `aiPromptReadRecord`
// không mang `Result` phía Rust, nên hình dạng lỗi của nó LỆCH khỏi mọi adapter khác của tệp
// này (xem doc-comment tại chỗ: nhánh "không có cầu Tauri" trả `error: UNKNOWN_IPC_ERROR`,
// KHÁC `aiPromptAssemble`'s `error: null` cho cùng nhánh đó).
// ═══════════════════════════════════════════════════════════════════════════════════
describe('⑧ aiPromptReadRecord — bốn nhánh trả về', () => {
  it('1) Rust trả null ⇒ { value: null, error: null } — "chưa lắp lần nào", KHÔNG một lỗi', async () => {
    const state = await freshAdapter()
    mockInvoke.mockResolvedValue(null)

    const result = await state.aiPromptReadRecord()

    expect(result).toEqual({ value: null, error: null })
  })

  it('2) Rust trả một hình dạng KHÔNG đúng AssembledPromptWire ⇒ { value: null, error: khác null }', async () => {
    const state = await freshAdapter()
    mockInvoke.mockResolvedValue(validWire({ chapter_id: '10' }))

    const result = await state.aiPromptReadRecord()

    expect(result.value).toBeNull()
    expect(result.error).not.toBeNull()
  })

  it('3) đối chứng chiều ngược — Rust trả một wire ĐÚNG hình dạng ⇒ { value: wire, error: null }', async () => {
    const state = await freshAdapter()
    const wire = validWire()
    mockInvoke.mockResolvedValue(wire)

    const result = await state.aiPromptReadRecord()

    expect(result).toEqual({ value: wire, error: null })
  })

  it('4) invoke() ném một IpcError thật (có cầu Tauri) ⇒ { value: null, error: đúng IpcError đó }', async () => {
    const state = await freshAdapter()
    Object.defineProperty(window, '__TAURI_INTERNALS__', { configurable: true, value: {} })
    const err: IpcError = { code: 'store.open_failed', message_key: 'err.store.open_failed', params: {}, retryable: false }
    mockInvoke.mockRejectedValue(err)

    const result = await state.aiPromptReadRecord()

    expect(result).toEqual({ value: null, error: err })
  })

  it('⚠️ nhánh KHÔNG có cầu Tauri LỆCH quy ước — trả error: UNKNOWN_IPC_ERROR, không phải error: null', async () => {
    // Không gắn `window.__TAURI_INTERNALS__` (mặc định của môi trường test này) — cùng khuôn
    // `readingState.test.ts`. Adapter `Result`-mang khác (`aiPromptAssemble`, `config/promptset.ts`)
    // trả `error: null` ở nhánh này; `aiPromptReadRecord` không mang `Result` phía Rust nên một
    // `IpcError` ở đây LÀ lỗi cấu hình, không phải ca sản phẩm bình thường — doc-comment tại
    // chỗ giải thích vì sao nó cố ý trả khác.
    const state = await freshAdapter()
    mockInvoke.mockRejectedValue(new Error('khong co cau IPC trong window nay'))

    const result = await state.aiPromptReadRecord()

    expect(result.value).toBeNull()
    expect(result.error).not.toBeNull()
    expect(result.error?.code).toBe('ipc.unknown')
  })
})
