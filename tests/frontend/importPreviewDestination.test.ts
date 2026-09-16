/**
 * `modes/libraryImport.ts` + `modes/LibraryMode.vue` — đích của một lượt nhập đơn ngữ
 * (Story 6.7b, FR122 nửa hai, Phase 4).
 *
 * ─────────────────────────────────────────────────────────────────────────────
 * 🔴 PHẠM VI — theo ĐÚNG nơi Phase 3 đã dựng, không theo vị trí spec ban đầu đoán
 * ─────────────────────────────────────────────────────────────────────────────
 * Radio + picker sống ở `LibraryMode.vue`'s `import-form` (PRE-SUBMIT), không bên trong
 * `ImportPreviewOverlay.vue` — xem Implementation Notes Phase 3, mục "Placement decision".
 * Đích KHÔNG đổi được sau khi màn xem trước đã mở (không màn hình nào cho re-pick giữa
 * phiên) — nên "changing the destination triggers exactly one preview rebuild" ở đây đo
 * đúng cái CÓ THẬT: chọn đích rồi nộp form gọi ĐÚNG MỘT vòng IPC xem trước, mang đích đã
 * chọn — không phải một điều khiển sống lại giữa lúc màn xem trước đang mở.
 *
 * ⚠️ Đo IPC bằng HIỆU SỐ lần gọi trước/sau (không phải `not.toHaveBeenCalled()`), theo đúng
 * yêu cầu của Task list — một hiệu số phân biệt được "0 lần gọi thừa" với "chưa từng gọi gì
 * cả", điều `not.toHaveBeenCalled()` không phân biệt được khi test khác đã gọi trước đó.
 */
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'
import { flushPromises, mount } from '@vue/test-utils'
import type { WorkRow } from '../../src/config/library'

const previewTextMock = vi.fn()
// 🔵 THÊM (vòng rà, mục B10/B11) — trước bản sửa này chỉ `previewImportEncodingFromText` có
// mock riêng; `submitFilePath`/`submitPastedUrls` gọi thẳng hàm THẬT (`previewImportEncodingFromFile`/
// `startUrlImport`), vốn không phát nổi trong môi trường test (không cầu IPC) và rơi về
// `{ batch: null, error: null }` best-effort — im lặng, không nói CÁI GÌ đã được gửi. Một hồi
// quy đổi `effectiveSourceLang` (đích) về `sourceLang` (ô gõ tay) trong hai nhánh này ship
// XANH vì không có gì đọc lại tham số đã gửi.
const previewFileMock = vi.fn()
const startUrlMock = vi.fn()
const mockInvoke = vi.fn()

vi.mock('../../src/config/project', async (importOriginal) => {
  const actual = await importOriginal<typeof import('../../src/config/project')>()
  return {
    ...actual,
    previewImportEncodingFromText: (...args: unknown[]) => previewTextMock(...args),
    previewImportEncodingFromFile: (...args: unknown[]) => previewFileMock(...args),
    startUrlImport: (...args: unknown[]) => startUrlMock(...args),
  }
})

vi.mock('../../src/config/segment', async (importOriginal) => {
  const actual = await importOriginal<typeof import('../../src/config/segment')>()
  return { ...actual, readOpenChapterSegments: async () => ({ segments: [], caret_segment_id: null }) }
})

vi.mock('@tauri-apps/api/event', () => ({ listen: async () => () => {} }))
vi.mock('@tauri-apps/api/core', () => ({ invoke: (...args: unknown[]) => mockInvoke(...args) }))

function workRow(overrides: Partial<WorkRow> = {}): WorkRow {
  return {
    work_id: 'w-1',
    atproj_path: '/tmp/Truyen Da Co.atproj',
    name: 'Truyen Da Co',
    source_lang: 'zh',
    genre: 'action',
    created_at: '2026-09-01T00:00:00.000Z',
    updated_at: '2026-09-01T00:00:00.000Z',
    chapter_count: 3,
    status: 'in_progress',
    status_is_override: false,
    chapter_done_count: 1,
    ...overrides,
  }
}

/** Cùng khuôn `libraryWorks.test.ts::mockInvokeForMount` — `library_list_works` phục vụ CẢ
 * lưới Tác phẩm LẪN picker đích (cùng một `listLibraryWorks()`, hai chỗ gọi độc lập). */
function mockInvokeWithWorks(works: WorkRow[]): void {
  mockInvoke.mockImplementation((cmd: string) => {
    if (cmd === 'library_list_works') {
      return Promise.resolve({ total: works.length, matched: works.length, works, genres: [], source_langs: [] })
    }
    if (cmd === 'read_work_lifecycle' || cmd === 'list_chapters') {
      return Promise.reject({
        code: 'work.none_open',
        message_key: 'err.work.none_open',
        params: {},
        retryable: false,
      })
    }
    return Promise.reject(new Error(`invoke gia khong biet lenh: ${cmd}`))
  })
}

beforeEach(() => {
  vi.resetModules()
  mockInvoke.mockReset()
  previewTextMock.mockReset()
  previewFileMock.mockReset()
  startUrlMock.mockReset()
})

// ═════════════════════════════════════════════════════════════════════════════════
// Mặc định là Tác phẩm mới
// ═════════════════════════════════════════════════════════════════════════════════

describe('libraryImport.ts — đích mặc định là Tác phẩm mới', () => {
  it('destinationMode mặc định "new", destinationWorkId là null, effectiveName/effectiveSourceLang/effectiveGenre đọc ba ô gõ tay', async () => {
    const nhap = await import('../../src/modes/libraryImport')

    expect(nhap.destinationMode.value).toBe('new')
    expect(nhap.destinationWorkId.value).toBeNull()
    expect(nhap.pickedDestinationWork.value).toBeNull()

    nhap.name.value = 'Ten Go Tay'
    nhap.sourceLang.value = 'en'
    nhap.genre.value = 'romance'
    expect(nhap.effectiveName.value).toBe('Ten Go Tay')
    expect(nhap.effectiveSourceLang.value).toBe('en')
    expect(nhap.effectiveGenre.value).toBe('romance')
  })
})

// ═════════════════════════════════════════════════════════════════════════════════
// Chọn một Tác phẩm đã có ⇒ tên/nguồn/thể loại READ-ONLY, hiện giá trị của ĐÍCH
// ═════════════════════════════════════════════════════════════════════════════════

describe('LibraryMode.vue — chọn Tác phẩm đã có khoá ba ô tên/nguồn/thể loại (mount thật)', () => {
  let wrapper: ReturnType<typeof mount> | null = null

  afterEach(() => {
    wrapper?.unmount()
    wrapper = null
  })

  it('chọn "Tác phẩm đã có" rồi chọn một Tác phẩm trong picker: ba ô đổi sang disabled, hiện đúng giá trị của đích', async () => {
    mockInvokeWithWorks([workRow()])

    const { default: LibraryMode } = await import('../../src/modes/LibraryMode.vue')
    wrapper = mount(LibraryMode)
    await flushPromises() // onMounted -> loadDestinationWorks() (best-effort, không đồng bộ)
    await wrapper.vm.$nextTick()

    // Trước khi chọn — ba ô còn là ô gõ tay, KHÔNG disabled.
    const before = wrapper.findAll('.import-form .field input[disabled], .import-form .field select[disabled]')
    expect(before).toHaveLength(0)

    const existingRadio = wrapper.get('[data-import-destination] input[value="existing"]')
    expect((existingRadio.element as HTMLInputElement).disabled).toBe(false)
    await existingRadio.setValue(true)
    await wrapper.vm.$nextTick()

    const picker = wrapper.get('[data-import-destination-picker] select')
    await picker.setValue('w-1')
    await wrapper.vm.$nextTick()

    // Sau khi chọn — tên, nguồn, thể loại đều là input disabled mang ĐÚNG giá trị của đích.
    const disabledFields = wrapper.findAll('.import-form .field input[disabled]')
    const values = disabledFields.map((f) => (f.element as HTMLInputElement).value)
    expect(values).toContain('Truyen Da Co')
    expect(values).toContain('zh')
    expect(values).toContain('action')
  })

  it('Library RỖNG: radio "Tác phẩm đã có" bị khoá và một dòng nói vì sao, không chỉ khoá im lặng', async () => {
    mockInvokeWithWorks([])

    const { default: LibraryMode } = await import('../../src/modes/LibraryMode.vue')
    wrapper = mount(LibraryMode)
    await flushPromises()
    await wrapper.vm.$nextTick()

    const existingRadio = wrapper.get('[data-import-destination] input[value="existing"]')
    expect((existingRadio.element as HTMLInputElement).disabled).toBe(true)
    // Picker của Tác phẩm đã có không hiện — radio đó không chọn được để mà mở nó ra.
    expect(wrapper.find('[data-import-destination-picker]').exists()).toBe(false)
    expect(wrapper.find('[data-import-destination] .hint').exists()).toBe(true)
  })
})

// ═════════════════════════════════════════════════════════════════════════════════
// Chọn đích rồi nộp ⇒ ĐÚNG MỘT vòng IPC xem trước, mang đích đã chọn
// ═════════════════════════════════════════════════════════════════════════════════

describe('libraryImport.ts::submitPastedText — đích đã chọn đi trọn tới lệnh xem trước, đúng MỘT lần', () => {
  it('đích là một Tác phẩm đã có: previewImportEncodingFromText nhận đúng nguồn ngữ + work_id của đích, hiệu số lần gọi là 1', async () => {
    const nhap = await import('../../src/modes/libraryImport')
    previewTextMock.mockResolvedValue({ preview: null, error: null })

    const before = previewTextMock.mock.calls.length

    // Mô phỏng "đã chọn đích" -- cùng những gì `setDestinationMode('existing')` +
    // `setDestinationWorkId('w-1')` làm, không qua DOM (đúng khuôn `libraryImportBlocksResubmitWhilePreviewOpen.test.ts`).
    nhap.destinationWorksHaveLoaded.value = true
    nhap.destinationWorks.value = [workRow({ source_lang: 'en', name: 'Dich Truoc' })]
    nhap.destinationMode.value = 'existing'
    nhap.destinationWorkId.value = 'w-1'
    nhap.pastedText.value = 'mot doan van ban moi'

    await nhap.submitPastedText()

    const after = previewTextMock.mock.calls.length
    expect(after - before).toBe(1) // ĐÚNG một vòng IPC mới -- đo bằng HIỆU SỐ, không phải not.toHaveBeenCalled()

    const [text, sourceLangArg, chapterPatternArg, destinationArg] = previewTextMock.mock.calls.at(-1) ?? []
    expect(text).toBe('mot doan van ban moi')
    expect(sourceLangArg).toBe('en') // effectiveSourceLang KẾ THỪA từ đích, không phải ô gõ tay
    expect(chapterPatternArg).toBeNull()
    expect(destinationArg).toBe('w-1') // đích đi tới ĐÚNG tham số cuối của previewImportEncodingFromText
  })

  it('radio "Tác phẩm đã có" đã chọn nhưng CHƯA chọn Tác phẩm nào trong picker: nộp form là no-op, 0 vòng IPC mới', async () => {
    const nhap = await import('../../src/modes/libraryImport')
    previewTextMock.mockResolvedValue({ preview: null, error: null })

    const before = previewTextMock.mock.calls.length

    nhap.destinationWorksHaveLoaded.value = true
    nhap.destinationWorks.value = [workRow()]
    nhap.destinationMode.value = 'existing'
    nhap.destinationWorkId.value = null // radio đã chọn "đã có", picker CHƯA chọn Tác phẩm nào
    nhap.pastedText.value = 'mot doan van ban khac'

    expect(nhap.destinationPending.value).toBe(true)
    await nhap.submitPastedText()

    const after = previewTextMock.mock.calls.length
    expect(after - before).toBe(0) // KHÔNG được âm thầm đọc destinationWorkId: null thành "Tác phẩm mới"
  })

  it('đích là Tác phẩm mới (mặc định): previewImportEncodingFromText nhận destination = null, đúng MỘT lần', async () => {
    const nhap = await import('../../src/modes/libraryImport')
    previewTextMock.mockResolvedValue({ preview: null, error: null })

    const before = previewTextMock.mock.calls.length
    nhap.pastedText.value = 'van ban cho tac pham moi'
    nhap.name.value = 'Ten Moi'
    nhap.sourceLang.value = 'zh'

    await nhap.submitPastedText()

    const after = previewTextMock.mock.calls.length
    expect(after - before).toBe(1)
    const [, sourceLangArg, , destinationArg] = previewTextMock.mock.calls.at(-1) ?? []
    expect(sourceLangArg).toBe('zh')
    expect(destinationArg).toBeNull()
  })
})

// ═════════════════════════════════════════════════════════════════════════════════
// destinationPending — một id LẠ hoặc chuỗi RỖNG cũng phải chặn (vòng rà, mục E4/E5)
// ═════════════════════════════════════════════════════════════════════════════════

describe('libraryImport.ts::destinationPending — id lạ hoặc chuỗi rỗng cũng chặn nộp form (mục E4/E5)', () => {
  it('destinationWorkId là một id KHÔNG CÒN trong destinationWorks (Tác phẩm vừa bị xoá/đổi tên khỏi Library): pending = true, nộp là no-op', async () => {
    const nhap = await import('../../src/modes/libraryImport')
    previewTextMock.mockResolvedValue({ preview: null, error: null })

    nhap.destinationWorksHaveLoaded.value = true
    nhap.destinationWorks.value = [workRow({ work_id: 'w-1' })]
    nhap.destinationMode.value = 'existing'
    nhap.destinationWorkId.value = 'w-99-khong-con-trong-danh-sach'
    nhap.pastedText.value = 'mot doan van ban'

    expect(nhap.pickedDestinationWork.value).toBeNull()
    expect(nhap.destinationPending.value).toBe(true) // TRƯỚC bản sửa: false — id khác null nên guard cũ bỏ qua

    const before = previewTextMock.mock.calls.length
    await nhap.submitPastedText()
    expect(previewTextMock.mock.calls.length - before).toBe(0)
  })

  it('destinationWorkId là chuỗi RỖNG (mục giữ chỗ bị vô hiệu hoá của picker, LibraryMode.vue): pending = true, nộp là no-op', async () => {
    const nhap = await import('../../src/modes/libraryImport')
    previewTextMock.mockResolvedValue({ preview: null, error: null })

    nhap.destinationWorksHaveLoaded.value = true
    nhap.destinationWorks.value = [workRow({ work_id: 'w-1' })]
    nhap.destinationMode.value = 'existing'
    nhap.destinationWorkId.value = ''
    nhap.pastedText.value = 'mot doan van ban'

    expect(nhap.destinationPending.value).toBe(true) // TRƯỚC bản sửa: false — '' !== null

    const before = previewTextMock.mock.calls.length
    await nhap.submitPastedText()
    expect(previewTextMock.mock.calls.length - before).toBe(0)
  })
})

// ═════════════════════════════════════════════════════════════════════════════════
// Hint "Library trống" và dải lỗi tải KHÔNG được hiện cùng lúc (mục E6); thử lại sau một
// lượt tải LỖI (mục E7)
// ═════════════════════════════════════════════════════════════════════════════════

describe('libraryImport.ts — hint rỗng và dải lỗi nói HAI câu khác nhau (mục E6), thử lại sau lượt lỗi (mục E7)', () => {
  it('lượt tải picker LỖI: destinationLibraryGenuinelyEmpty = false, destinationWorksError khác null — hai dòng KHÔNG cùng hiện', async () => {
    mockInvoke.mockImplementation((cmd: string) => {
      if (cmd === 'library_list_works') {
        return Promise.reject({
          code: 'library.root_invalid',
          message_key: 'err.library.root_invalid',
          params: {},
          retryable: true,
        })
      }
      return Promise.reject(new Error(`invoke gia khong biet lenh: ${cmd}`))
    })

    const nhap = await import('../../src/modes/libraryImport')
    await nhap.loadDestinationWorks()

    expect(nhap.destinationWorksError.value).not.toBeNull()
    // TRƯỚC bản sửa: `destinationLibraryGenuinelyEmpty` không tồn tại và hint "Library trống"
    // canh thẳng `!destinationExistingWorkAvailable` — ĐÚNG cả khi lỗi lẫn khi rỗng thật, nên
    // cả hai câu cùng hiện. Sau bản sửa, hai mệnh đề loại trừ nhau.
    expect(nhap.destinationLibraryGenuinelyEmpty.value).toBe(false)
  })

  it('Library rỗng THẬT (0 Tác phẩm, KHÔNG lỗi): destinationLibraryGenuinelyEmpty = true, destinationWorksError là null', async () => {
    mockInvokeWithWorks([])
    const nhap = await import('../../src/modes/libraryImport')
    await nhap.loadDestinationWorks()

    expect(nhap.destinationWorksError.value).toBeNull()
    expect(nhap.destinationLibraryGenuinelyEmpty.value).toBe(true)
  })

  it('LibraryMode.vue mount thật: lượt tải LỖI ngay từ onMounted chỉ hiện dải lỗi, KHÔNG kèm câu "Library trống"', async () => {
    mockInvoke.mockImplementation((cmd: string) => {
      if (cmd === 'library_list_works') {
        return Promise.reject({
          code: 'library.root_invalid',
          message_key: 'err.library.root_invalid',
          params: {},
          retryable: true,
        })
      }
      if (cmd === 'read_work_lifecycle' || cmd === 'list_chapters') {
        return Promise.reject({ code: 'work.none_open', message_key: 'err.work.none_open', params: {}, retryable: false })
      }
      return Promise.reject(new Error(`invoke gia khong biet lenh: ${cmd}`))
    })

    const { default: LibraryMode } = await import('../../src/modes/LibraryMode.vue')
    const wrapper = mount(LibraryMode)
    await flushPromises()
    await wrapper.vm.$nextTick()

    expect(wrapper.find('[data-import-destination] .hint-error').exists()).toBe(true)
    // TRƯỚC bản sửa, cùng lượt trượt này cũng bật hint "Library trống" bên cạnh — cả hai
    // `<p class="hint">` cùng hiện, nói dối về lý do thật.
    expect(wrapper.find('[data-import-destination] .hint:not(.hint-error)').exists()).toBe(false)

    wrapper.unmount()
  })

  it('setDestinationMode("existing") sau một lượt tải LỖI gọi lại listLibraryWorks() — không còn khoá im lặng cho hết phiên', async () => {
    let calls = 0
    mockInvoke.mockImplementation((cmd: string) => {
      if (cmd === 'library_list_works') {
        calls += 1
        if (calls === 1) {
          return Promise.reject({
            code: 'library.root_invalid',
            message_key: 'err.library.root_invalid',
            params: {},
            retryable: true,
          })
        }
        return Promise.resolve({ total: 1, matched: 1, works: [workRow()], genres: [], source_langs: [] })
      }
      return Promise.reject(new Error(`invoke gia khong biet lenh: ${cmd}`))
    })

    const nhap = await import('../../src/modes/libraryImport')

    await nhap.loadDestinationWorks() // lượt ĐẦU, trượt — mô phỏng `onMounted`
    expect(calls).toBe(1)
    expect(nhap.destinationWorksError.value).not.toBeNull()
    expect(nhap.destinationWorksHaveLoaded.value).toBe(true)

    nhap.setDestinationMode('existing') // `void loadDestinationWorks()` bên trong — không await được trực tiếp
    await flushPromises()

    // TRƯỚC bản sửa: guard là `!destinationWorksHaveLoaded.value`, đã là `false` từ lượt lỗi
    // trên ⇒ `calls` sẽ dừng ở 1 mãi mãi. Sau bản sửa, một `destinationWorksError !== null`
    // cũng kích một lượt tải lại.
    expect(calls).toBe(2)
    expect(nhap.destinationWorksError.value).toBeNull()
    expect(nhap.destinationWorks.value).toHaveLength(1)
  })
})

// ═════════════════════════════════════════════════════════════════════════════════
// Quay radio về "Tác phẩm mới" xoá đích đã chọn — đường phòng thủ đã có (§reset), chưa ai
// canh trực tiếp (vòng rà, mục VG2 "radio-reset path")
// ═════════════════════════════════════════════════════════════════════════════════

describe('libraryImport.ts::setDestinationMode — quay về "new" xoá Tác phẩm đã chọn (mục VG2)', () => {
  it('đã chọn một Tác phẩm rồi quay radio về "new": destinationWorkId về null, không sống sót dưới một radio đã đổi ý', async () => {
    const nhap = await import('../../src/modes/libraryImport')

    nhap.destinationWorksHaveLoaded.value = true
    nhap.destinationWorks.value = [workRow()]
    nhap.setDestinationMode('existing')
    nhap.setDestinationWorkId('w-1')
    expect(nhap.destinationWorkId.value).toBe('w-1')

    nhap.setDestinationMode('new')

    expect(nhap.destinationMode.value).toBe('new')
    expect(nhap.destinationWorkId.value).toBeNull()
  })
})

// ═════════════════════════════════════════════════════════════════════════════════
// destinationWorks tải lại sau một lượt nhập THÀNH CÔNG (vòng rà, mục B5)
// ═════════════════════════════════════════════════════════════════════════════════

describe('libraryImport.ts::finishImportSubmission — tải lại destinationWorks sau một lượt nhập thành công (mục B5)', () => {
  it('một Tác phẩm vừa tạo TRONG PHIÊN NÀY xuất hiện làm đích chọn được ngay sau đó, không cần remount', async () => {
    let calls = 0
    mockInvoke.mockImplementation((cmd: string) => {
      if (cmd === 'library_list_works') {
        calls += 1
        const works = calls === 1 ? [] : [workRow()]
        return Promise.resolve({ total: works.length, matched: works.length, works, genres: [], source_langs: [] })
      }
      if (cmd === 'read_work_lifecycle' || cmd === 'list_chapters') {
        return Promise.reject({ code: 'work.none_open', message_key: 'err.work.none_open', params: {}, retryable: false })
      }
      return Promise.reject(new Error(`invoke gia khong biet lenh: ${cmd}`))
    })

    const nhap = await import('../../src/modes/libraryImport')

    await nhap.loadDestinationWorks() // mô phỏng lượt tải BAN ĐẦU -- Library còn rỗng
    expect(calls).toBe(1)
    expect(nhap.destinationWorks.value).toHaveLength(0)

    nhap.finishImportSubmission(
      {
        meta: {
          meta_schema_version: 2,
          work_id: 'w-1',
          name: 'Truyen Vua Tao',
          source_lang: 'zh',
          genre: '',
          created_at: '2026-09-16T00:00:00.000Z',
          updated_at: '2026-09-16T00:00:00.000Z',
          chapter_count: 1,
        },
        folder: '/tmp/Truyen Vua Tao.atproj',
        images_saved: 0,
        images_failed: 0,
      },
      null,
    )
    await flushPromises()

    // TRƯỚC bản sửa: `destinationWorks` chỉ tải MỘT LẦN ở `onMounted` -- không lượt gọi thứ
    // hai nào ở đây, `calls` dừng ở 1 mãi mãi.
    expect(calls).toBe(2)
    expect(nhap.destinationWorks.value).toHaveLength(1)
  })
})

// ═════════════════════════════════════════════════════════════════════════════════
// Chữ trên ba nút nộp phản ánh ĐÍCH đã chọn (vòng rà, mục B4)
// ═════════════════════════════════════════════════════════════════════════════════

describe('LibraryMode.vue — chữ trên nút nộp đổi theo đích (mục B4)', () => {
  let wrapper: ReturnType<typeof mount> | null = null

  afterEach(() => {
    wrapper?.unmount()
    wrapper = null
  })

  it('đích Tác phẩm mới (mặc định): nút dán/tệp đọc "Tạo Tác phẩm từ …", nút URL giữ nguyên "Tải danh sách URL"', async () => {
    mockInvokeWithWorks([workRow()])
    const { default: LibraryMode } = await import('../../src/modes/LibraryMode.vue')
    wrapper = mount(LibraryMode)
    await flushPromises()
    await wrapper.vm.$nextTick()

    // Thứ tự tài liệu: dán văn bản, tệp, URL — cả ba đều mang `data-import-preview-open`.
    const buttons = wrapper.findAll('[data-import-preview-open]')
    expect(buttons).toHaveLength(3)
    const [textBtn, fileBtn, urlBtn] = buttons
    expect(textBtn.text()).toContain('Tạo Tác phẩm')
    expect(fileBtn.text()).toContain('Tạo Tác phẩm')
    expect(urlBtn.text()).toBe('Tải danh sách URL')
  })

  it('đích là một Tác phẩm đã có: ba nút đổi sang "Thêm Chương từ …", không còn nói "Tạo Tác phẩm"/"Tải danh sách URL"', async () => {
    mockInvokeWithWorks([workRow()])
    const { default: LibraryMode } = await import('../../src/modes/LibraryMode.vue')
    wrapper = mount(LibraryMode)
    await flushPromises()
    await wrapper.vm.$nextTick()

    const existingRadio = wrapper.get('[data-import-destination] input[value="existing"]')
    await existingRadio.setValue(true)
    await wrapper.vm.$nextTick()

    // Radio "đã có" chọn được nhưng picker CHƯA chọn Tác phẩm nào -- chữ trên nút đổi theo
    // RADIO (destinationMode), không đợi một Tác phẩm cụ thể được chọn.
    const buttons = wrapper.findAll('[data-import-preview-open]')
    expect(buttons).toHaveLength(3)
    const [textBtn, fileBtn, urlBtn] = buttons
    expect(textBtn.text()).toBe('Thêm Chương từ văn bản')
    expect(fileBtn.text()).toBe('Thêm Chương từ tệp')
    expect(urlBtn.text()).toBe('Thêm Chương từ danh sách URL')
  })

  it('nhóm hai radio đích nằm trong một `<fieldset>`/`<legend>` (mục B7)', async () => {
    mockInvokeWithWorks([workRow()])
    const { default: LibraryMode } = await import('../../src/modes/LibraryMode.vue')
    wrapper = mount(LibraryMode)
    await flushPromises()
    await wrapper.vm.$nextTick()

    const group = wrapper.get('[data-import-destination]')
    expect(group.element.tagName).toBe('FIELDSET')
    expect(group.find('legend').exists()).toBe(true)
  })
})

// ═════════════════════════════════════════════════════════════════════════════════
// submitFilePath/submitPastedUrls — cùng đích, cùng effectiveSourceLang, đúng MỘT lần
// (vòng rà, mục B10/B11 — trước bản sửa chỉ `submitPastedText` có ca)
// ═════════════════════════════════════════════════════════════════════════════════

describe('libraryImport.ts::submitFilePath — đích đã chọn đi trọn tới previewImportEncodingFromFile (mục B10)', () => {
  it('đích là một Tác phẩm đã có: nhận effectiveSourceLang CỦA ĐÍCH (không phải ô gõ tay), work_id đúng, đúng MỘT lần', async () => {
    const nhap = await import('../../src/modes/libraryImport')
    previewFileMock.mockResolvedValue({ batch: null, error: null })

    const before = previewFileMock.mock.calls.length
    nhap.destinationWorksHaveLoaded.value = true
    nhap.destinationWorks.value = [workRow({ source_lang: 'en', name: 'Dich Truoc' })]
    nhap.destinationMode.value = 'existing'
    nhap.destinationWorkId.value = 'w-1'
    nhap.sourceLang.value = 'zh' // ô gõ tay khác hẳn đích -- một hồi quy đọc nhầm ô này sẽ lộ ở đây
    nhap.filePath.value = '/tmp/mot-tep.txt'

    await nhap.submitFilePath()

    const after = previewFileMock.mock.calls.length
    expect(after - before).toBe(1)
    const [paths, sourceLangArg, , destinationArg] = previewFileMock.mock.calls.at(-1) ?? []
    expect(paths).toEqual(['/tmp/mot-tep.txt'])
    expect(sourceLangArg).toBe('en')
    expect(destinationArg).toBe('w-1')
  })
})

describe('libraryImport.ts::submitPastedUrls — đích đã chọn đi trọn tới startUrlImport (mục B11)', () => {
  it('đích là một Tác phẩm đã có: nhận effectiveSourceLang CỦA ĐÍCH (không phải ô gõ tay), work_id đúng, đúng MỘT lần', async () => {
    const nhap = await import('../../src/modes/libraryImport')
    startUrlMock.mockResolvedValue({ batch: null, error: null })

    const before = startUrlMock.mock.calls.length
    nhap.destinationWorksHaveLoaded.value = true
    nhap.destinationWorks.value = [workRow({ source_lang: 'en', name: 'Dich Truoc' })]
    nhap.destinationMode.value = 'existing'
    nhap.destinationWorkId.value = 'w-1'
    nhap.sourceLang.value = 'zh' // ô gõ tay khác hẳn đích -- một hồi quy đọc nhầm ô này sẽ lộ ở đây
    nhap.pastedUrls.value = 'https://example.com/1'

    await nhap.submitPastedUrls()

    const after = startUrlMock.mock.calls.length
    expect(after - before).toBe(1)
    const [urls, sourceLangArg, destinationArg] = startUrlMock.mock.calls.at(-1) ?? []
    expect(urls).toEqual(['https://example.com/1'])
    expect(sourceLangArg).toBe('en')
    expect(destinationArg).toBe('w-1')
  })
})

// ═════════════════════════════════════════════════════════════════════════════════
// ImportPreviewOverlay.vue — dải `.ip-destination` (vòng rà, mục B10/VG1 — trước bản sửa
// KHÔNG một ca nào mount overlay để đọc dải này)
// ═════════════════════════════════════════════════════════════════════════════════

describe('ImportPreviewOverlay.vue — dải .ip-destination đọc đúng đích (mục B10/VG1)', () => {
  it('đích Tác phẩm mới: dải đọc "Tác phẩm mới"', async () => {
    const state = await import('../../src/importPreviewState')
    const { default: ImportPreviewOverlay } = await import('../../src/ImportPreviewOverlay.vue')
    previewTextMock.mockResolvedValue({ preview: null, error: null })

    await state.openImportPreviewFromText('Ten Moi', 'zh', '', 'noi dung', null)
    const wrapper = mount(ImportPreviewOverlay)

    expect(wrapper.get('.ip-destination').text()).toContain('Tác phẩm mới')

    wrapper.unmount()
    state.resetImportPreview()
  })

  it('đích là một Tác phẩm đã có: dải nêu TÊN đích, không còn nói "Tác phẩm mới"', async () => {
    const state = await import('../../src/importPreviewState')
    const { default: ImportPreviewOverlay } = await import('../../src/ImportPreviewOverlay.vue')
    previewTextMock.mockResolvedValue({ preview: null, error: null })

    await state.openImportPreviewFromText('Truyen Da Co', 'zh', '', 'noi dung', 'w-1')
    const wrapper = mount(ImportPreviewOverlay)

    const banner = wrapper.get('.ip-destination').text()
    expect(banner).toContain('Truyen Da Co')
    expect(banner).not.toContain('Tác phẩm mới')

    wrapper.unmount()
    state.resetImportPreview()
  })
})
