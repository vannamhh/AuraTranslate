/**
 * `hasIpcBridge()` của `src/config/glossary.ts` — cụm E vá (vòng rà Epic 3, 2026-08-26).
 *
 * ─────────────────────────────────────────────────────────────────────────────
 * 🔴 VỆ NÀY GÁC 15 NHÁNH `catch`, VÀ TRƯỚC TỆP NÀY KHÔNG CA NÀO CHẠM NÓ
 * ─────────────────────────────────────────────────────────────────────────────
 * `hasIpcBridge()` là thứ quyết định một lượt `invoke` trượt sẽ đi lối nào: **có** cầu Tauri
 * ⇒ đây là một lỗi THẬT, trả `UNKNOWN_IPC_ERROR` để giao diện nói ra; **không** có cầu ⇒
 * đang chạy `npm run dev` ngoài Tauri, im lặng có chủ (`error: null`). Lật vệ này thì MỌI
 * lỗi IPC thật bị nuốt thành `error: null` — đúng lớp *"rỗng im lặng"* mà `AGENTS.md` gọi là
 * lớp lỗi trung tâm của dự án.
 *
 * ⚠️ **PHẠM VI CHÍNH XÁC — tệp này canh NỬA SAU của mỗi khối `catch`, không cả khối.** Mỗi
 * `catch` mở bằng `if (isIpcError(err)) return { … , error: err }`; `hasIpcBridge()` chỉ được
 * hỏi khi nhánh đó KHÔNG khớp. Mọi ca dưới đây reject bằng một `Error` trần *(cố ý: đó là
 * đầu vào duy nhất đưa luồng tới vệ đang canh)*, nên nhánh `isIpcError` **không** chạy ở đây
 * — nó có chủ ở `glossaryConfigGuards.test.ts` và ở các tệp state theo từng bề mặt. Đừng đọc
 * tệp này thành *"15 khối `catch` đã được phủ trọn"*.
 *
 * ⚠️ **Đo 2026-08-26 trên `3be0f5f`** (lăng kính verification-gap): ép `hasIpcBridge()` trả
 * `false` rồi chạy `npx vitest run tests/frontend/glossary` — **16 tệp / 238 ca xanh
 * nguyên**, và `grep -c __TAURI_INTERNALS__` trên cả 16 tệp `glossary*.test.ts` = **0**.
 * *(Sổ nợ ghi 14/186; con số đó có trước cụm D — 🔵 sửa tại chỗ ở `deferred-work.md`.)*
 *
 * ─────────────────────────────────────────────────────────────────────────────
 * ⚠️ PHẠM VI — mock ở ĐÚNG BIÊN `@tauri-apps/api/core`, khuôn `glossaryConfigGuards.test.ts`
 * ─────────────────────────────────────────────────────────────────────────────
 * KHÔNG mock trọn module `src/config/glossary` (khuôn `glossaryQuickAdd`/`glossaryManage`):
 * làm vậy thì `hasIpcBridge()` thật không hề chạy, và ca này thành một ca không canh gì cả —
 * đúng lỗi đã sinh ra khoảng hở này. Khuôn lái `window.__TAURI_INTERNALS__` chép từ
 * `bootstrap.test.ts`, tệp DUY NHẤT trong `tests/frontend/**` đang lái biến đó (nó canh
 * `src/config/bootstrap.ts`, không canh `glossary.ts`).
 *
 * 🔴 **Bảng dưới là một danh sách ĐÓNG: đủ 15 adapter, không 14.** Vá một chỗ nối rồi tuyên
 * bố bảng đã đóng là đúng lỗi mà `AGENTS.md` §Known pitfalls gọi tên. Ca cuối tệp so BẰNG
 * bảng này với tập hàm export THẬT của module, nên một hàm thứ 16 ra đời mà quên thêm vào
 * đây là một lượt ĐỎ, không một lượt im lặng — xem doc-comment của ca đó cho giới hạn.
 */
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'

const mockInvoke = vi.fn()
vi.mock('@tauri-apps/api/core', () => ({ invoke: (...args: unknown[]) => mockInvoke(...args) }))

/** Bản sao của `UNKNOWN_IPC_ERROR` (`src/config/glossary.ts` không export nó). */
const UNKNOWN_IPC_ERROR = {
  code: 'ipc.unknown',
  message_key: 'err.unknown',
  params: {},
  retryable: false,
}

type Adapter = typeof import('../../src/config/glossary')

/**
 * BA hình dạng trả về, gộp thành HAI họ theo trường mang kết luận:
 * - `'error'` — hình dạng ba trạng thái `{ <giá trị>, error }`. Gồm cả `lookupGlossaryTerm`
 *   (`{ found: 'unknown', error }`): tên trường giá trị khác nhau, nhưng trường `error` thì
 *   chung, và trường `error` mới là thứ vệ này quyết định.
 * - `'outcome'` — hai lối ra CÓ TÊN (`'error'` ↔ `'ipc_unavailable'`), dựng cho hai đường đi
 *   qua hộp thoại hệ điều hành, nơi *"đã huỷ"* phải phân biệt được với *"hỏng"*.
 */
type ResultFamily = 'error' | 'outcome'

type BridgeCase = {
  /** Tên hàm adapter — cũng là nhãn ca. */
  readonly name: string
  /** Tên command trên dây; chẩn đoán phải nêu đích danh nó. */
  readonly wire: string
  readonly family: ResultFamily
  /** Tham số chỉ cần hợp kiểu: `invoke` luôn reject nên giá trị không đi tới đâu. */
  readonly run: (glossary: Adapter) => Promise<unknown>
}

/**
 * 15 adapter, thứ tự theo vị trí trong `src/config/glossary.ts`.
 *
 * ⚠️ Thêm một adapter vào tệp nguồn thì thêm một hàng vào đây CÙNG LƯỢT — ca `②c` ở cuối tệp
 * so BẰNG bảng này với tập hàm export thật và sẽ đỏ nếu quên.
 */
const CASES: readonly BridgeCase[] = [
  {
    name: 'lookupGlossaryTerm',
    wire: 'glossary_lookup_term',
    family: 'error',
    run: (g) => g.lookupGlossaryTerm('慕容'),
  },
  {
    name: 'addGlossaryTerm',
    wire: 'glossary_add_term',
    family: 'error',
    run: (g) => g.addGlossaryTerm('global', '慕容', 'Mộ Dung', '', 'person'),
  },
  {
    name: 'updateGlossaryTerm',
    wire: 'glossary_update_term',
    family: 'error',
    run: (g) => g.updateGlossaryTerm('global', 7, 'Mộ Dung', '', 'person'),
  },
  {
    name: 'glossaryMarksForChapter',
    wire: 'glossary_marks_for_chapter',
    family: 'error',
    run: (g) => g.glossaryMarksForChapter('慕容复出现了。', 'zh'),
  },
  {
    name: 'pendingGlossaryCandidates',
    wire: 'glossary_pending_candidates',
    family: 'error',
    run: (g) => g.pendingGlossaryCandidates(),
  },
  {
    name: 'confirmPendingGlossaryTranslation',
    wire: 'glossary_confirm_pending_translation',
    family: 'error',
    run: (g) => g.confirmPendingGlossaryTranslation('global', 7, 'Mộ Dung'),
  },
  {
    name: 'approveGlossaryCandidate',
    wire: 'glossary_approve_candidate',
    family: 'error',
    run: (g) => g.approveGlossaryCandidate(7, 'Mộ Dung', 'person'),
  },
  {
    name: 'rejectGlossaryCandidate',
    wire: 'glossary_reject_candidate',
    family: 'error',
    run: (g) => g.rejectGlossaryCandidate(7),
  },
  {
    name: 'listGlossaryEntries',
    wire: 'glossary_list_entries',
    family: 'error',
    run: (g) => g.listGlossaryEntries(),
  },
  {
    name: 'deleteGlossaryTerm',
    wire: 'glossary_delete_term',
    family: 'error',
    run: (g) => g.deleteGlossaryTerm('global', 7),
  },
  {
    name: 'exportGlossaryTier',
    wire: 'glossary_export_tier',
    family: 'outcome',
    run: (g) => g.exportGlossaryTier('global'),
  },
  {
    name: 'openGlossaryImportPreview',
    wire: 'glossary_open_import_preview',
    family: 'outcome',
    run: (g) => g.openGlossaryImportPreview('global'),
  },
  {
    name: 'confirmGlossaryImport',
    wire: 'glossary_confirm_import',
    family: 'error',
    run: (g) => g.confirmGlossaryImport({}),
  },
  {
    name: 'cancelGlossaryImport',
    wire: 'glossary_cancel_import',
    family: 'error',
    run: (g) => g.cancelGlossaryImport(),
  },
  {
    name: 'promoteGlossaryTermToGlobal',
    wire: 'glossary_promote_term_to_global',
    family: 'error',
    run: (g) => g.promoteGlossaryTermToGlobal(7),
  },
]

/**
 * Đọc một trường trên một giá trị `unknown`.
 *
 * ⚠️ Trả về `unknown`, KHÔNG ép kiểu về hình dạng đã khai: bảng ca đi qua mười lăm kiểu trả
 * về khác nhau, và một `as` ở đây sẽ dạy `@typescript-eslint/no-unnecessary-condition` rằng
 * nhánh nó đang đọc luôn đúng — tức làm đúng cái điều mà cổng thứ mười ra đời để chặn.
 */
function fieldOf(result: unknown, key: string): unknown {
  if (typeof result !== 'object' || result === null) return undefined
  return (result as Record<string, unknown>)[key]
}

function attachBridge(): void {
  Object.defineProperty(window, '__TAURI_INTERNALS__', { configurable: true, value: {} })
}

async function freshAdapter(): Promise<Adapter> {
  vi.resetModules()
  mockInvoke.mockReset()
  // Một lỗi KHÔNG phải `IpcError` — `isIpcError` từ chối nó (không `code`, không
  // `message_key`), nên lượt gọi rơi đúng vào nhánh mà `hasIpcBridge()` gác.
  mockInvoke.mockRejectedValue(new Error('cau IPC tra loi mot Error tran'))
  return import('../../src/config/glossary')
}

let errorSpy: ReturnType<typeof vi.spyOn>
let infoSpy: ReturnType<typeof vi.spyOn>

beforeEach(() => {
  Reflect.deleteProperty(window, '__TAURI_INTERNALS__')
  errorSpy = vi.spyOn(console, 'error').mockImplementation(() => undefined)
  infoSpy = vi.spyOn(console, 'info').mockImplementation(() => undefined)
})

afterEach(() => {
  errorSpy.mockRestore()
  infoSpy.mockRestore()
})

/** Mọi chẩn đoán đã ghi qua một spy, nối thành một chuỗi để tìm tên command. */
function loggedText(spy: ReturnType<typeof vi.spyOn>): string {
  return spy.mock.calls.map((args: unknown[]) => args.map(String).join(' ')).join('\n')
}

describe('②a CÓ cầu Tauri — một lỗi thật KHÔNG được nuốt thành `error: null`', () => {
  it.each(CASES)('$name', async ({ wire, family, run }) => {
    const glossary = await freshAdapter()
    attachBridge()

    const result = await run(glossary)

    if (family === 'outcome') {
      expect(fieldOf(result, 'outcome')).toBe('error')
    }
    expect(fieldOf(result, 'error')).toEqual(UNKNOWN_IPC_ERROR)
    // Chẩn đoán phải nêu ĐÍCH DANH command — mười lăm khối `catch` gần như giống hệt nhau,
    // và một tên chép nhầm sang khối bên cạnh không làm hỏng gì ngoài khả năng chẩn đoán.
    expect(loggedText(errorSpy)).toContain(wire)
  })
})

describe('②b KHÔNG có cầu Tauri — `npm run dev` ngoài Tauri vẫn chạy được, im lặng có chủ', () => {
  it.each(CASES)('$name', async ({ wire, family, run }) => {
    const glossary = await freshAdapter()
    // Không gắn cầu: `beforeEach` đã gỡ `__TAURI_INTERNALS__`.

    const result = await run(glossary)

    if (family === 'outcome') {
      expect(fieldOf(result, 'outcome')).toBe('ipc_unavailable')
      // Hai lối ra CÓ TÊN thì mang tên, không mang thêm một `error` đi kèm: một hình dạng
      // `{ outcome: 'ipc_unavailable', error: <gì đó> }` cho chỗ gọi hai nguồn sự thật để
      // rẽ nhánh, và nó sẽ rẽ nhầm ở đúng ca im lặng-có-chủ này.
      expect(fieldOf(result, 'error')).toBeUndefined()
    } else {
      expect(fieldOf(result, 'error')).toBeNull()
    }
    // Im lặng với NGƯỜI DÙNG, không im lặng với người chẩn đoán.
    expect(loggedText(infoSpy)).toContain(wire)
    expect(errorSpy).not.toHaveBeenCalled()
  })
})

describe('②c quần thể — bảng ca phải phủ ĐỦ số nhánh `catch` có trong tệp nguồn', () => {
  /**
   * 🔴 **Phép so BẰNG hai chiều với chính module, không một con số chốt cứng.**
   *
   * ⚠️ Bản đầu của ca này viết `expect(CASES).toHaveLength(15)` và chỉ kiểm một chiều (mỗi
   * tên trong bảng là một hàm thật). Vòng rà bước 4 (lăng kính edge-case + blind-hunter,
   * 2026-08-26) bắt đúng chỗ đó: một adapter **thứ 16** thêm vào `src/config/glossary.ts`
   * mà quên thêm hàng vào bảng thì ca vẫn XANH, trong khi doc-comment ngay trên lại hứa nó
   * sẽ đỏ. Đó là *"đối chứng KẾT QUẢ với ĐIỀU NÓ KHAI"* — một chú thích khai một bảo đảm mà
   * mã không cấp, đúng lớp lỗi mà cả cụm E này tồn tại để đóng. ⇒ Con số suy RA từ module.
   *
   * ⚠️ **GIỚI HẠN THẬT:** phép so lấy MỌI hàm export của module, nên một hàm phụ trợ được
   * export ở đây sẽ làm ca này đỏ dù nó không phải adapter. Đó là hành vi ĐÚNG và có chủ ý —
   * cùng hình dạng danh-sách-cho-phép của `check:layout` Kiểm C: thêm một cái tên vào bề mặt
   * export của tệp này là một quyết định phải viết ra, không một dòng tiện tay. Đo 2026-08-26:
   * `src/config/glossary.ts` export đúng **15** hàm, không hàm nào ngoài adapter.
   */
  it('bảng ca khớp CHÍNH XÁC tập hàm export của `src/config/glossary.ts`, không thừa không thiếu', async () => {
    const glossary = await freshAdapter()

    const exported = new Set(
      Object.keys(glossary).filter(
        (key) => typeof (glossary as unknown as Record<string, unknown>)[key] === 'function',
      ),
    )
    const tabled = new Set(CASES.map((c) => c.name))

    expect(tabled).toEqual(exported)
    // Tên trên dây cũng phải đôi một khác nhau: hai hàng chép nhầm cùng một `wire` thì phép
    // khẳng định "chẩn đoán nêu đích danh command" của ②a xanh oan cho một trong hai.
    expect(new Set(CASES.map((c) => c.wire)).size).toBe(CASES.length)
  })
})
