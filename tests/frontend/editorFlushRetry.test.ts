/**
 * Đường **ghi TRƯỢT** và đường **hai lượt flush chồng nhau** — code review 2026-08-13.
 *
 * Bốn nhóm test của Quyết định #6 phủ đường flush khi nó **chạy đúng**. Không nhóm nào phủ hai
 * đường dưới đây, và cả hai đều là khuyết tật thật đã đo:
 *
 * ① **Vòng lặp thử lại 0 ms.** `onFlushed()` chỉ gọi `schedule.onWrite()` khi tập chờ đã sạch,
 *    nên một lượt ghi trượt để `deadline()` đứng nguyên ở một mốc **đã quá hạn** ⇒ độ trễ tính
 *    ra âm ⇒ kẹp về 0 ⇒ `setTimeout(…, 0)` ⇒ gọi lại ngay ⇒ trượt lại. Không backoff, không
 *    log, và nó dồn thẳng vào writer **duy nhất, nối tiếp** của AD-11.
 *
 * ② **`await flushEditorNow()` là một lời hứa rỗng.** Bản cũ giữ `inFlight` là một `boolean` và
 *    mở đầu bằng `if (inFlight) return`, nên lượt gọi thứ hai trả về **ngay**, không ghi gì,
 *    không chờ gì. Ba chỗ gọi cầm lời hứa đó là `beginSubmit` và listener thoát ứng dụng — tức
 *    đúng hai chỗ mà một lượt trả về sớm nghĩa là **mất chữ**.
 *
 * ⚠️ Tệp này mock `saveSegmentTargets` bằng bộ ghi **của riêng nó**, không dùng `failNextSave`
 * của `segmentFixture`: cái đó trượt **đúng một lượt** rồi tự tắt, còn mệnh đề ① chỉ hiện ra
 * dưới một lượt trượt **dai dẳng**.
 */
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'
import { FIXTURE_CHAPTER_ID, readFixture } from './support/segmentFixture'
import { EDITOR_RETRY_FLOOR_MS } from '../../src/panels/editorFlush'

type Edit = { id: number; target_text: string }
type SaveResult = { outcome: { chapter_id: number; saved: number } | null; error: unknown }

/** Mọi lượt gọi `saveSegmentTargets` đã đi qua, theo thứ tự. */
const calls: { chapterId: number; edits: Edit[] }[] = []
/** `true` ⇒ mọi lượt ghi trượt, không chỉ lượt kế tiếp. */
let failEvery = false
/** Khi đặt, lượt ghi treo tới khi test tự giải phóng nó — để dựng một lô ĐANG BAY. */
let gate: { promise: Promise<void>; open: () => void } | null = null

const WRITE_FAILED = {
  code: 'store.write_failed',
  message_key: 'err.store.write_failed',
  params: {},
  retryable: true,
}

async function save(chapterId: number, edits: readonly Edit[]): Promise<SaveResult> {
  calls.push({ chapterId, edits: edits.map((e) => ({ ...e })) })
  if (gate !== null) await gate.promise
  if (failEvery) return { outcome: null, error: WRITE_FAILED }
  return { outcome: { chapter_id: chapterId, saved: edits.length }, error: null }
}

vi.mock('../../src/config/segment', async (importOriginal) => {
  const actual = await importOriginal<typeof import('../../src/config/segment')>()
  return { ...actual, readOpenChapterSegments: readFixture, saveSegmentTargets: save }
})

async function freshState() {
  vi.resetModules()
  const state = await import('../../src/panels/editorPanelState')
  await state.ensureSegmentsLoaded()
  return state
}

function openGate(): { promise: Promise<void>; open: () => void } {
  let open!: () => void
  const promise = new Promise<void>((resolve) => {
    open = resolve
  })
  return { promise, open }
}

beforeEach(() => {
  calls.length = 0
  failEvery = false
  gate = null
})

afterEach(() => {
  vi.useRealTimers()
  vi.restoreAllMocks()
})

describe('① lượt ghi TRƯỢT không được quay vòng', () => {
  it('trượt dai dẳng suốt 10 giây ⇒ vài lượt thử lại, KHÔNG phải hàng nghìn', async () => {
    vi.useFakeTimers()
    const state = await freshState()
    failEvery = true

    state.noteEditorEdit(11, 'một câu đang gõ dở')

    // 10 giây đồng hồ giả. Với sàn 2 000 ms, số lượt thử lại phải đếm được trên một bàn tay.
    await vi.advanceTimersByTimeAsync(10_000)

    // 🔴 Đây là mệnh đề thật của ca này. Trước bản vá, mỗi lượt trượt đặt lại một
    // `setTimeout(…, 0)` nên con số này là **hàng nghìn** — giới hạn duy nhất là tốc độ CPU.
    const ceiling = Math.ceil(10_000 / EDITOR_RETRY_FLOOR_MS) + 1
    expect(calls.length).toBeGreaterThanOrEqual(2)
    expect(calls.length).toBeLessThanOrEqual(ceiling)

    // Và tập chờ KHÔNG được xoá: một lượt ghi trượt giữ nguyên chữ để lượt sau thử lại.
    expect(calls.at(-1)?.edits).toEqual([{ id: 11, target_text: 'một câu đang gõ dở' }])
  })

  it('trượt để lại một dòng chẩn đoán — im lặng là khuyết tật, không phải phong cách', async () => {
    vi.useFakeTimers()
    const spy = vi.spyOn(console, 'error').mockImplementation(() => {})
    const state = await freshState()
    failEvery = true

    state.noteEditorEdit(11, 'chữ chưa lưu được')
    await vi.advanceTimersByTimeAsync(3_000)

    expect(spy).toHaveBeenCalled()
    // Dòng log phải mang mã lỗi thật, không một câu chung chung: người đọc nó đang đi tìm
    // nguyên nhân, không đi tìm lời an ủi.
    expect(spy.mock.calls.flat().join(' ')).toContain('store.write_failed')
  })

  it('mốc *"Đã lưu"* KHÔNG đi lên sau một lượt trượt', async () => {
    vi.useFakeTimers()
    const state = await freshState()
    failEvery = true

    state.noteEditorEdit(11, 'chữ chưa lưu được')
    await vi.advanceTimersByTimeAsync(3_000)

    // Một mốc đi lên sau một lượt ghi trượt là màn hình nói dối theo hướng an tâm — đúng
    // thứ UX-DR30 tồn tại để cấm.
    expect(state.editorLastSavedAt.value).toBeNull()
  })
})

describe('② `await flushEditorNow()` phải là một lời hứa THẬT', () => {
  it('lượt gọi thứ hai CHỜ lô đang bay, không trả về rỗng', async () => {
    const state = await freshState()
    gate = openGate()

    state.noteEditorEdit(11, 'câu A')
    const first = state.flushEditorNow()
    // Nhường một microtask để lô thứ nhất kịp vào `saveSegmentTargets` và treo ở cổng.
    await Promise.resolve()
    expect(calls.length).toBe(1)

    const second = state.flushEditorNow()
    gate.open()
    gate = null

    expect(await first).toBe('saved')
    // 🔴 Trước bản vá, lượt thứ hai trả về **ngay** vì `inFlight === true` — và ba chỗ gọi
    // `await` nó cầm một lời hứa rỗng. Nay nó chờ lô kia xong rồi mới trả lời.
    expect(await second).toBe('saved')
  })

  it('chữ gõ TRONG LÚC lô đang bay vẫn lên dây ở lượt kế tiếp', async () => {
    const state = await freshState()
    gate = openGate()

    state.noteEditorEdit(11, 'câu A')
    const first = state.flushEditorNow()
    await Promise.resolve()

    // Người dùng gõ tiếp trong lúc lô thứ nhất còn đang bay — chuyện thường nhất trên đời.
    state.noteEditorEdit(12, 'câu B gõ trong lúc lô bay')

    const second = state.flushEditorNow()
    gate.open()
    gate = null
    await first
    await second

    // 🔴 Mệnh đề: câu B **phải** đã đi qua dây. Trước bản vá, lượt gọi thứ hai trả về ngay,
    // và nếu lượt đó là listener thoát ứng dụng thì câu B chết cùng tiến trình.
    const sent = calls.flatMap((c) => c.edits)
    expect(sent).toContainEqual({ id: 12, target_text: 'câu B gõ trong lúc lô bay' })
    expect(calls.every((c) => c.chapterId === FIXTURE_CHAPTER_ID)).toBe(true)
  })

  it('không có gì để ghi ⇒ `clean`, và không một lượt IPC nào', async () => {
    const state = await freshState()

    // Một lượt flush trên tập chờ rỗng là chuyện thường ở đường thoát ứng dụng: người dùng
    // không sửa gì rồi đóng cửa sổ. Nó không được mở một giao dịch rỗng.
    expect(await state.flushEditorNow()).toBe('clean')
    expect(calls.length).toBe(0)
  })
})
