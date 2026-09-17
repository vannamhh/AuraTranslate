/**
 * State của mục **bộ prompt theo thể loại** — Story 4.4, FR69, Phase 4a (lớp dữ liệu, không
 * `.vue`).
 *
 * ⚠️ Cùng khuôn `aiConfigState.test.ts`: `config/promptset.ts` là biên IPC, giả lập bằng
 * `vi.mock`, không gọi `@tauri-apps/api` thật.
 */
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'

const promptSetListMock = vi.fn()
const promptSetCreateMock = vi.fn()
const promptSetRenameMock = vi.fn()
const promptSetUpdateBodyMock = vi.fn()
const promptSetDeleteMock = vi.fn()

vi.mock('../../src/config/promptset', () => ({
  promptSetList: (...args: unknown[]) => promptSetListMock(...args),
  promptSetCreate: (...args: unknown[]) => promptSetCreateMock(...args),
  promptSetRename: (...args: unknown[]) => promptSetRenameMock(...args),
  promptSetUpdateBody: (...args: unknown[]) => promptSetUpdateBodyMock(...args),
  promptSetDelete: (...args: unknown[]) => promptSetDeleteMock(...args),
}))

/** Nạp lại module mỗi ca — state là module-level singleton, cùng lý do `freshState` của
 * `aiConfigState.test.ts`. */
async function freshState() {
  vi.resetModules()
  return import('../../src/promptSetState')
}

function emptySets() {
  return []
}

function makeSet(over: Partial<{ id: number; name: string; body: string; tier: 'global' | 'work'; shadowed_body: string | null }> = {}) {
  return { id: 1, name: 'Xianxia', body: '{{glossary_terms}}', tier: 'global' as const, shadowed_body: null, ...over }
}

beforeEach(() => {
  promptSetListMock.mockReset()
  promptSetCreateMock.mockReset()
  promptSetRenameMock.mockReset()
  promptSetUpdateBodyMock.mockReset()
  promptSetDeleteMock.mockReset()
  promptSetListMock.mockResolvedValue({ sets: emptySets(), workTierAvailable: false, variables: [], error: null })
  promptSetCreateMock.mockResolvedValue({ id: 1, warnings: { unknown_markers: [], glossary_terms_missing: false }, error: null })
  promptSetRenameMock.mockResolvedValue(null)
  promptSetUpdateBodyMock.mockResolvedValue({ warnings: { unknown_markers: [], glossary_terms_missing: false }, error: null })
  promptSetDeleteMock.mockResolvedValue(null)
})

afterEach(() => {
  vi.restoreAllMocks()
})

describe('isPromptSetNameValid — hàm THUẦN, đọc lại đúng ràng buộc phía Rust (validate_name)', () => {
  it('rỗng hoặc chỉ khoảng trắng ⇒ false', async () => {
    const { isPromptSetNameValid } = await freshState()
    expect(isPromptSetNameValid('')).toBe(false)
    expect(isPromptSetNameValid('   ')).toBe(false)
    expect(isPromptSetNameValid('\t')).toBe(false)
    expect(isPromptSetNameValid('\u{3000}')).toBe(false) // I/O Matrix spec 4.4 "Blank-ish name"
  })

  it('chuỗi có nội dung, kể cả mang khoảng trắng biên ⇒ true', async () => {
    const { isPromptSetNameValid } = await freshState()
    expect(isPromptSetNameValid('Xianxia')).toBe(true)
    expect(isPromptSetNameValid('  Journalism  ')).toBe(true)
  })
})

describe('loadPromptSets — nạp danh sách, chỉ lượt MỚI NHẤT được ghi kết quả', () => {
  it('nạp sets và workTierAvailable từ response', async () => {
    promptSetListMock.mockResolvedValue({
      sets: [makeSet({ id: 1, name: 'Xianxia' }), makeSet({ id: 2, name: 'Journalism' })],
      workTierAvailable: true,
      error: null,
    })
    const { loadPromptSets, promptSets, promptSetWorkTierAvailable, promptSetLoadError } = await freshState()

    await loadPromptSets()

    expect(promptSets.value).toHaveLength(2)
    expect(promptSetWorkTierAvailable.value).toBe(true)
    expect(promptSetLoadError.value).toBeNull()
  })

  it('lượt đọc trượt ⇒ lỗi hiện qua promptSetLoadError, danh sách cũ giữ nguyên', async () => {
    promptSetListMock.mockResolvedValueOnce({
      sets: [makeSet({ id: 1, name: 'Xianxia' })],
      workTierAvailable: false,
      error: null,
    })
    const err = { code: 'store.open_failed', message_key: 'err.store.open_failed', params: {}, retryable: false }
    promptSetListMock.mockResolvedValueOnce({ sets: null, workTierAvailable: false, error: err })
    const { loadPromptSets, promptSets, promptSetLoadError } = await freshState()

    await loadPromptSets()
    await loadPromptSets()

    expect(promptSetLoadError.value).toEqual(err)
    expect(promptSets.value).toHaveLength(1) // danh sách của lượt thành công trước đó không bị xoá
  })

  it('một lượt tải CŨ trả về SAU một lượt MỚI hơn không được ghi đè state', async () => {
    let resolveFirst: (value: unknown) => void = () => {}
    promptSetListMock.mockImplementationOnce(
      () =>
        new Promise((resolve) => {
          resolveFirst = resolve
        }),
    )
    promptSetListMock.mockResolvedValueOnce({
      sets: [makeSet({ id: 2, name: 'Journalism' })],
      workTierAvailable: true,
      error: null,
    })
    const { loadPromptSets, promptSets } = await freshState()

    const first = loadPromptSets() // treo
    const second = loadPromptSets() // hoàn tất trước
    await second
    expect(promptSets.value).toHaveLength(1)
    expect(promptSets.value[0]?.name).toBe('Journalism')

    resolveFirst({ sets: [makeSet({ id: 1, name: 'Xianxia' })], workTierAvailable: false, error: null })
    await first

    // Lượt ĐẦU (cũ hơn) hoàn tất SAU nhưng không được thắng lượt thứ hai (mới hơn).
    expect(promptSets.value).toHaveLength(1)
    expect(promptSets.value[0]?.name).toBe('Journalism')
  })
})

describe('shadowed_body — Quyết định #1: bộ Global bị che vẫn hiện', () => {
  it('hàng Work mang shadowed_body của bộ Global cùng tên', async () => {
    promptSetListMock.mockResolvedValue({
      sets: [makeSet({ id: 5, name: 'Xianxia', tier: 'work', body: 'than-work', shadowed_body: 'than-global' })],
      workTierAvailable: true,
      error: null,
    })
    const { loadPromptSets, promptSetByName } = await freshState()

    await loadPromptSets()

    const set = promptSetByName('Xianxia')
    expect(set?.tier).toBe('work')
    expect(set?.shadowed_body).toBe('than-global')
  })
})

describe('promptSetById / promptSetByName — hàm truy cập', () => {
  it('trả về bộ khớp, hoặc null nếu không có', async () => {
    promptSetListMock.mockResolvedValue({
      sets: [makeSet({ id: 7, name: 'Xianxia' })],
      workTierAvailable: false,
      error: null,
    })
    const { loadPromptSets, promptSetById, promptSetByName } = await freshState()
    await loadPromptSets()

    expect(promptSetById(7)?.name).toBe('Xianxia')
    expect(promptSetById(999)).toBeNull()
    expect(promptSetByName('Xianxia')?.id).toBe(7)
    expect(promptSetByName('KhongTonTai')).toBeNull()
  })
})

describe('createPromptSet — re-validate trước khi gọi IPC, thành công ⇒ nạp lại danh sách', () => {
  it('tên rỗng/toàn khoảng trắng ⇒ 0 lượt promptSetCreate', async () => {
    const { createPromptSet } = await freshState()

    await createPromptSet('global', '   ', 'than bai')

    expect(promptSetCreateMock).not.toHaveBeenCalled()
  })

  it('tên hợp lệ ⇒ gọi promptSetCreate với tên ĐÃ TRIM, rồi nạp lại danh sách', async () => {
    const { createPromptSet } = await freshState()
    promptSetListMock.mockClear()

    await createPromptSet('work', '  Xianxia  ', '{{glossary_terms}}')

    expect(promptSetCreateMock).toHaveBeenCalledTimes(1)
    expect(promptSetCreateMock).toHaveBeenCalledWith('work', 'Xianxia', '{{glossary_terms}}')
    expect(promptSetListMock).toHaveBeenCalledTimes(1) // đọc lại sau lượt tạo thành công
  })

  it('lượt tạo TRƯỢT ⇒ IpcError hiện qua promptSetActionError, KHÔNG nạp lại danh sách', async () => {
    const err = { code: 'prompt_set.name_taken', message_key: 'err.prompt_set.name_taken', params: { name: 'Xianxia' }, retryable: false }
    promptSetCreateMock.mockResolvedValue({ id: null, warnings: null, error: err })
    const { createPromptSet, promptSetActionError } = await freshState()
    promptSetListMock.mockClear()

    await createPromptSet('global', 'Xianxia', 'than')

    expect(promptSetActionError.value).toEqual(err)
    expect(promptSetListMock).not.toHaveBeenCalled()
  })

  it('Quyết định #3: thân mang token lạ ⇒ lượt THÀNH CÔNG, cảnh báo hiện qua promptSetActionWarnings, KHÔNG phải một lỗi', async () => {
    promptSetCreateMock.mockResolvedValue({
      id: 3,
      warnings: { unknown_markers: ['{{glosary_terms}}'], glossary_terms_missing: true },
      error: null,
    })
    const { createPromptSet, promptSetActionError, promptSetActionWarnings } = await freshState()

    await createPromptSet('global', 'Xianxia', 'Dung {{glosary_terms}} thay vi ten dung.')

    expect(promptSetActionError.value).toBeNull()
    expect(promptSetActionWarnings.value).toEqual({ unknown_markers: ['{{glosary_terms}}'], glossary_terms_missing: true })
  })

  it('đang bận (busy) ⇒ một lượt gọi thứ hai là no-op (khoá tái nhập)', async () => {
    let resolveCreate: (value: unknown) => void = () => {}
    promptSetCreateMock.mockReturnValue(
      new Promise((resolve) => {
        resolveCreate = resolve
      }),
    )
    const { createPromptSet } = await freshState()

    const first = createPromptSet('global', 'Xianxia', 'than')
    await createPromptSet('global', 'Journalism', 'than khac') // lượt thứ hai trong lúc lượt đầu còn treo

    expect(promptSetCreateMock).toHaveBeenCalledTimes(1)
    resolveCreate({ id: 1, warnings: { unknown_markers: [], glossary_terms_missing: false }, error: null })
    await first
  })
})

describe('renamePromptSet — re-validate tên mới, thành công ⇒ nạp lại, vứt cảnh báo cũ', () => {
  it('tên mới rỗng ⇒ 0 lượt promptSetRename', async () => {
    const { renamePromptSet } = await freshState()

    await renamePromptSet('global', 1, '')

    expect(promptSetRenameMock).not.toHaveBeenCalled()
  })

  it('tên mới hợp lệ ⇒ gọi với tên ĐÃ TRIM rồi nạp lại danh sách', async () => {
    const { renamePromptSet } = await freshState()
    promptSetListMock.mockClear()

    await renamePromptSet('work', 5, '  Ten Moi  ')

    expect(promptSetRenameMock).toHaveBeenCalledWith('work', 5, 'Ten Moi')
    expect(promptSetListMock).toHaveBeenCalledTimes(1)
  })

  it('lượt đổi tên TRƯỢT ⇒ IpcError hiện qua promptSetActionError', async () => {
    const err = { code: 'prompt_set.not_found', message_key: 'err.prompt_set.not_found', params: {}, retryable: false }
    promptSetRenameMock.mockResolvedValue(err)
    const { renamePromptSet, promptSetActionError } = await freshState()

    await renamePromptSet('global', 1, 'Ten Moi')

    expect(promptSetActionError.value).toEqual(err)
  })

  it('vứt cảnh báo của một lượt Tạo/Sửa thân TRƯỚC đó — đổi tên không mang thân', async () => {
    promptSetCreateMock.mockResolvedValue({
      id: 1,
      warnings: { unknown_markers: ['{{foo}}'], glossary_terms_missing: true },
      error: null,
    })
    const { createPromptSet, renamePromptSet, promptSetActionWarnings } = await freshState()
    await createPromptSet('global', 'Xianxia', '{{foo}}')
    expect(promptSetActionWarnings.value).not.toBeNull()

    await renamePromptSet('global', 1, 'Ten Moi')

    expect(promptSetActionWarnings.value).toBeNull()
  })
})

describe('updatePromptSetBody — KHÔNG re-validate (Quyết định #3: thân không bao giờ bị từ chối)', () => {
  it('gọi promptSetUpdateBody dù thân rỗng, rồi nạp lại danh sách', async () => {
    const { updatePromptSetBody } = await freshState()
    promptSetListMock.mockClear()

    await updatePromptSetBody('global', 1, '')

    expect(promptSetUpdateBodyMock).toHaveBeenCalledWith('global', 1, '')
    expect(promptSetListMock).toHaveBeenCalledTimes(1)
  })

  it('cảnh báo thiếu {{glossary_terms}} ⇒ lượt THÀNH CÔNG, hiện qua promptSetActionWarnings', async () => {
    promptSetUpdateBodyMock.mockResolvedValue({
      warnings: { unknown_markers: [], glossary_terms_missing: true },
      error: null,
    })
    const { updatePromptSetBody, promptSetActionError, promptSetActionWarnings } = await freshState()

    await updatePromptSetBody('global', 1, '{{source_segment}} khong co thuat ngu.')

    expect(promptSetActionError.value).toBeNull()
    expect(promptSetActionWarnings.value).toEqual({ unknown_markers: [], glossary_terms_missing: true })
  })

  it('lượt sửa thân TRƯỢT ⇒ IpcError hiện qua promptSetActionError', async () => {
    const err = { code: 'prompt_set.not_found', message_key: 'err.prompt_set.not_found', params: {}, retryable: false }
    promptSetUpdateBodyMock.mockResolvedValue({ warnings: null, error: err })
    const { updatePromptSetBody, promptSetActionError } = await freshState()

    await updatePromptSetBody('global', 1, 'than moi')

    expect(promptSetActionError.value).toEqual(err)
  })
})

describe('deletePromptSet — thành công ⇒ nạp lại (bộ Global bị che hết bị che nếu có)', () => {
  it('gọi promptSetDelete rồi nạp lại danh sách', async () => {
    const { deletePromptSet } = await freshState()
    promptSetListMock.mockClear()

    await deletePromptSet('work', 5)

    expect(promptSetDeleteMock).toHaveBeenCalledWith('work', 5)
    expect(promptSetListMock).toHaveBeenCalledTimes(1)
  })

  it('lượt xoá TRƯỢT ⇒ IpcError hiện qua promptSetActionError, KHÔNG nạp lại', async () => {
    const err = { code: 'prompt_set.not_found', message_key: 'err.prompt_set.not_found', params: {}, retryable: false }
    promptSetDeleteMock.mockResolvedValue(err)
    const { deletePromptSet, promptSetActionError } = await freshState()
    promptSetListMock.mockClear()

    await deletePromptSet('global', 1)

    expect(promptSetActionError.value).toEqual(err)
    expect(promptSetListMock).not.toHaveBeenCalled()
  })
})

describe('busy — MỘT cờ dùng chung cho bốn thao tác ghi, canh CẢ BỐN chiều', () => {
  it('lượt Tạo còn treo ⇒ một lượt Xoá gọi giữa chừng là no-op', async () => {
    let resolveCreate: (value: unknown) => void = () => {}
    promptSetCreateMock.mockReturnValue(
      new Promise((resolve) => {
        resolveCreate = resolve
      }),
    )
    const { createPromptSet, deletePromptSet } = await freshState()

    const pendingCreate = createPromptSet('global', 'Xianxia', 'than')
    await deletePromptSet('global', 1)

    expect(promptSetDeleteMock).not.toHaveBeenCalled()
    resolveCreate({ id: 1, warnings: { unknown_markers: [], glossary_terms_missing: false }, error: null })
    await pendingCreate
  })
})

describe('selectedPromptSetName — trạng thái CỤC BỘ cho bảng AI Translation, 0 lượt IPC', () => {
  it('setSelectedPromptSetName bỏ qua một tên KHÔNG có trong danh sách đã nạp', async () => {
    promptSetListMock.mockResolvedValue({ sets: [makeSet({ name: 'Xianxia' })], workTierAvailable: false, error: null })
    const { loadPromptSets, setSelectedPromptSetName, selectedPromptSetName } = await freshState()
    await loadPromptSets()

    setSelectedPromptSetName('KhongTonTai')

    expect(selectedPromptSetName.value).toBeNull()
  })

  it('setSelectedPromptSetName chấp nhận một tên CÓ trong danh sách, và null để bỏ chọn', async () => {
    promptSetListMock.mockResolvedValue({ sets: [makeSet({ name: 'Xianxia' })], workTierAvailable: false, error: null })
    const { loadPromptSets, setSelectedPromptSetName, selectedPromptSetName, selectedPromptSet } = await freshState()
    await loadPromptSets()

    setSelectedPromptSetName('Xianxia')
    expect(selectedPromptSetName.value).toBe('Xianxia')
    expect(selectedPromptSet()?.name).toBe('Xianxia')

    setSelectedPromptSetName(null)
    expect(selectedPromptSetName.value).toBeNull()
  })

  it('một lượt loadPromptSets SAU đó không còn giải được tên đang chọn ⇒ tự bỏ chọn', async () => {
    promptSetListMock.mockResolvedValueOnce({ sets: [makeSet({ name: 'Xianxia' })], workTierAvailable: false, error: null })
    const { loadPromptSets, setSelectedPromptSetName, selectedPromptSetName } = await freshState()
    await loadPromptSets()
    setSelectedPromptSetName('Xianxia')
    expect(selectedPromptSetName.value).toBe('Xianxia')

    // Bộ "Xianxia" bị xoá ở nơi khác — lượt nạp lại kế tiếp không còn thấy nó.
    promptSetListMock.mockResolvedValueOnce({ sets: [makeSet({ id: 9, name: 'Journalism' })], workTierAvailable: false, error: null })
    await loadPromptSets()

    expect(selectedPromptSetName.value).toBeNull()
  })
})

describe('promptSetVariables — Phase 4c: nạp từ PromptSetListWire.variables, KHÔNG một mảng gõ tay ở tầng state', () => {
  it('loadPromptSets ghi promptSetVariables ĐÚNG những gì response trả — kể cả một danh sách KHÁC ba tên ratify thật, chứng minh state không tự khai lại danh sách của riêng nó', async () => {
    promptSetListMock.mockResolvedValue({
      sets: emptySets(),
      workTierAvailable: false,
      // Cố ý khác Ba biến ratify thật (glossary_terms/source_segment/tm_similar_segments) —
      // nếu `promptSetState.ts` từng giữ một bản chép tay riêng, ca này sẽ đỏ vì state sẽ
      // không phản chiếu ĐÚNG mảng giả này.
      variables: ['mock_alpha', 'mock_beta'],
      error: null,
    })
    const { loadPromptSets, promptSetVariables } = await freshState()

    await loadPromptSets()

    expect(promptSetVariables.value).toEqual(['mock_alpha', 'mock_beta'])
  })

  it('một lượt tải TRƯỢT không xoá danh sách biến CŨ đã nạp thành công trước đó', async () => {
    promptSetListMock.mockResolvedValueOnce({
      sets: emptySets(),
      workTierAvailable: false,
      variables: ['mock_alpha'],
      error: null,
    })
    const { loadPromptSets, promptSetVariables } = await freshState()
    await loadPromptSets()
    expect(promptSetVariables.value).toEqual(['mock_alpha'])

    const err = { code: 'store.open_failed', message_key: 'err.store.open_failed', params: {}, retryable: false }
    promptSetListMock.mockResolvedValueOnce({ sets: null, workTierAvailable: false, variables: null, error: err })
    await loadPromptSets()

    expect(promptSetVariables.value).toEqual(['mock_alpha'])
  })

  it('resetPromptSets vứt danh sách biến về rỗng', async () => {
    promptSetListMock.mockResolvedValue({
      sets: emptySets(),
      workTierAvailable: false,
      variables: ['mock_alpha'],
      error: null,
    })
    const { loadPromptSets, resetPromptSets, promptSetVariables } = await freshState()
    await loadPromptSets()
    expect(promptSetVariables.value).toEqual(['mock_alpha'])

    resetPromptSets()

    expect(promptSetVariables.value).toEqual([])
  })
})

describe('resetPromptSets', () => {
  it('vứt toàn bộ state về mặc định', async () => {
    promptSetListMock.mockResolvedValue({ sets: [makeSet({ name: 'Xianxia' })], workTierAvailable: true, error: null })
    const err = { code: 'prompt_set.not_found', message_key: 'err.prompt_set.not_found', params: {}, retryable: false }
    promptSetDeleteMock.mockResolvedValue(err)
    const {
      loadPromptSets,
      setSelectedPromptSetName,
      deletePromptSet,
      resetPromptSets,
      promptSets,
      promptSetWorkTierAvailable,
      promptSetActionError,
      selectedPromptSetName,
    } = await freshState()

    await loadPromptSets()
    setSelectedPromptSetName('Xianxia')
    await deletePromptSet('global', 1) // để lại một IpcError trước khi reset
    expect(promptSets.value).toHaveLength(1)
    expect(promptSetActionError.value).toEqual(err)

    resetPromptSets()

    expect(promptSets.value).toEqual([])
    expect(promptSetWorkTierAvailable.value).toBe(false)
    expect(promptSetActionError.value).toBeNull()
    expect(selectedPromptSetName.value).toBeNull()
  })
})
