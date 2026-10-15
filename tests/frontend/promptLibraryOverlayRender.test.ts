/**
 * Dựng THẬT của `PromptLibraryOverlay.vue` VÀ `AiTranslationPanel.vue` — Story 4.4 (FR69),
 * Phase 4b (lớp màn hình).
 *
 * ─────────────────────────────────────────────────────────────────────────────
 * 🔴 VÌ SAO TỆP NÀY MOUNT CẢ HAI COMPONENT, KHÔNG CHỈ LỚP PHỦ
 * ─────────────────────────────────────────────────────────────────────────────
 * I/O Matrix spec 4.4, hàng "Switch effective set from AI panel — effective set changes
 * without opening Settings". Phase 3 (`prompt_set_contract.rs::switching_between_two_
 * resolvable_sets_needs_no_settings_reopen`) chỉ chứng minh nửa Rust — RẰNG kết quả phân
 * giải đổi khi tầng đổi. Nó KHÔNG chứng minh nửa BỀ MẶT — rằng người dùng đổi được bộ hiệu
 * lực từ CHÍNH Panel AI Translation mà không cần mở lớp phủ Cài đặt. `describe` cuối tệp này
 * đóng đúng nửa đó: nó mount `AiTranslationPanel.vue`, đổi lựa chọn qua `<select>` của chính
 * panel, và không bao giờ import/mount `SettingsOverlay.vue` — "không cần mở Cài đặt" được
 * chứng minh bằng việc lớp phủ đó vắng mặt hoàn toàn khỏi lượt kiểm, không chỉ bằng lời khai.
 *
 * Cùng khuôn `settingsOverlayAiConfigKeyRender.test.ts::freshOverlay` — `vi.resetModules()`
 * TRƯỚC, rồi nạp ĐỘNG cả state lẫn `.vue` trong CÙNG một lượt, để component và state module
 * dùng chung một thể hiện.
 */
import { beforeEach, describe, expect, it, vi } from 'vitest'
import { mount } from '@vue/test-utils'
import { flushPromises } from './support/flushMicrotasks'

const promptSetListMock = vi.fn()
const promptSetCreateMock = vi.fn()
const promptSetRenameMock = vi.fn()
const promptSetUpdateBodyMock = vi.fn()
const promptSetDeleteMock = vi.fn()
const promptSetExportMock = vi.fn()

vi.mock('../../src/config/promptset', () => ({
  promptSetList: (...args: unknown[]) => promptSetListMock(...args),
  promptSetCreate: (...args: unknown[]) => promptSetCreateMock(...args),
  promptSetRename: (...args: unknown[]) => promptSetRenameMock(...args),
  promptSetUpdateBody: (...args: unknown[]) => promptSetUpdateBodyMock(...args),
  promptSetDelete: (...args: unknown[]) => promptSetDeleteMock(...args),
  promptSetExport: (...args: unknown[]) => promptSetExportMock(...args),
}))

type WireRow = {
  id: number
  name: string
  body: string
  tier: 'global' | 'work'
  shadowed_body: string | null
  shadowed_id: number | null
}

function row(over: Partial<WireRow> = {}): WireRow {
  return {
    id: 1,
    name: 'Xianxia',
    body: '{{glossary_terms}}',
    tier: 'global',
    shadowed_body: null,
    shadowed_id: null,
    ...over,
  }
}

function resetMocks(): void {
  promptSetListMock.mockReset()
  promptSetCreateMock.mockReset()
  promptSetRenameMock.mockReset()
  promptSetUpdateBodyMock.mockReset()
  promptSetDeleteMock.mockReset()
  promptSetExportMock.mockReset()
  promptSetExportMock.mockResolvedValue({ outcome: 'done', path: '/tmp/x.prompt.md' })
}

/** Nạp lại `promptSetState`/`promptLibraryState`/`PromptLibraryOverlay.vue` CÙNG một lượt —
 * state là module-level singleton, cùng lý do `freshOverlay` của
 * `settingsOverlayAiConfigKeyRender.test.ts`. */
async function freshOverlay() {
  vi.resetModules()
  resetMocks()

  const promptSetState = await import('../../src/promptSetState')
  const promptLibraryState = await import('../../src/promptLibraryState')
  const PromptLibraryOverlay = (await import('../../src/PromptLibraryOverlay.vue')).default
  return { promptSetState, promptLibraryState, PromptLibraryOverlay }
}

beforeEach(() => {
  document.body.innerHTML = ''
})

describe('PromptLibraryOverlay.vue — dựng thật (Story 4.4)', () => {
  it('liệt kê hai tầng: hàng Tác phẩm che một hàng Toàn cục hiện ra HAI LẦN — bản thắng và bản HIỂN THỊ bị che (Quyết định #1)', async () => {
    const { promptSetState, promptLibraryState, PromptLibraryOverlay } = await freshOverlay()
    promptSetListMock.mockResolvedValue({
      sets: [
        row({ id: 5, name: 'Tiên hiệp', tier: 'work', body: 'than tac pham', shadowed_body: 'than toan cuc', shadowed_id: 9 }),
        row({ id: 7, name: 'Bao chi', tier: 'global', body: 'cau ngan' }),
      ],
      workTierAvailable: true,
      error: null,
    })
    promptLibraryState.openPromptLibrary()
    await flushPromises()

    const wrapper = mount(PromptLibraryOverlay, { attachTo: document.body })
    await wrapper.vm.$nextTick()

    const names = wrapper.findAll('.pl-row-name').map((n) => n.text())
    // "Tiên hiệp" xuất hiện HAI LẦN: hàng Tác phẩm đang thắng, và bản hiển thị bị che dưới
    // nhóm Toàn cục — I/O Matrix "the shadowed global stays visible in the list".
    expect(names.filter((n) => n === 'Tiên hiệp')).toHaveLength(2)
    expect(names).toContain('Bao chi')
    expect(wrapper.find('.pl-row-shadowed').exists()).toBe(true)
    expect(wrapper.find('.pl-row-shadowed').text()).toContain('Tiên hiệp')

    wrapper.unmount()
    promptLibraryState.resetPromptLibrary()
    promptSetState.resetPromptSets()
  })

  it('Story 4.5 Quyết định #3 — hàng Global bị che CHỌN ĐƯỢC qua shadowed_id, nạp đúng thân của chính nó (không phải thân Work đang thắng)', async () => {
    const { promptSetState, promptLibraryState, PromptLibraryOverlay } = await freshOverlay()
    promptSetListMock.mockResolvedValue({
      sets: [
        row({ id: 5, name: 'Tiên hiệp', tier: 'work', body: 'than tac pham', shadowed_body: 'than toan cuc', shadowed_id: 9 }),
      ],
      workTierAvailable: true,
      error: null,
    })
    promptLibraryState.openPromptLibrary()
    await flushPromises()

    const wrapper = mount(PromptLibraryOverlay, { attachTo: document.body })
    await wrapper.vm.$nextTick()

    const shadowedButton = wrapper.find('.pl-row-shadowed')
    expect(shadowedButton.exists()).toBe(true)
    await shadowedButton.trigger('submit')
    await wrapper.vm.$nextTick()

    // Ô soạn phải hiện DUNG than cua hang Global bi che (khong phai than Work dang thang) —
    // bang chung selectedRow tra ve tu chinh hang do, khong tu `promptSets` (hang thang).
    expect((wrapper.find('.pl-textarea').element as HTMLTextAreaElement).value).toBe('than toan cuc')

    await wrapper.find('.pl-export-form').trigger('submit')
    await flushPromises()
    expect(promptSetExportMock).toHaveBeenCalledWith('global', 9)

    wrapper.unmount()
    promptLibraryState.resetPromptLibrary()
    promptSetState.resetPromptSets()
  })

  it('Story 4.5 — nút "Nhập từ file" mang data-prompt-import-open, dispatch parameterless', async () => {
    const { promptSetState, promptLibraryState, PromptLibraryOverlay } = await freshOverlay()
    promptSetListMock.mockResolvedValue({ sets: [row()], workTierAvailable: false, error: null })
    promptLibraryState.openPromptLibrary()
    await flushPromises()

    const wrapper = mount(PromptLibraryOverlay, { attachTo: document.body })
    await wrapper.vm.$nextTick()

    const importButton = wrapper.find('[data-prompt-import-open]')
    expect(importButton.exists()).toBe(true)
    expect(importButton.text()).not.toBe('')

    wrapper.unmount()
    promptLibraryState.resetPromptLibrary()
    promptSetState.resetPromptSets()
  })

  it('chọn một hàng qua @submit (đường Enter bàn phím, Kiểm A cấm @click có tham số) nạp tên/thân vào ô soạn', async () => {
    const { promptSetState, promptLibraryState, PromptLibraryOverlay } = await freshOverlay()
    promptSetListMock.mockResolvedValue({
      sets: [row({ id: 7, name: 'Bao chi', body: 'cau ngan chu dong' })],
      workTierAvailable: false,
      error: null,
    })
    promptLibraryState.openPromptLibrary()
    await flushPromises()

    const wrapper = mount(PromptLibraryOverlay, { attachTo: document.body })
    await wrapper.vm.$nextTick()

    // `.pl-row-form` gồm hàng Toàn cục THẬT (duy nhất) rồi form "Bộ mới" — không hàng Tác
    // phẩm nào ở ca này. `.trigger('submit')` là đúng sự kiện DOM mà Enter trên nút focus
    // phát ra — không một `.trigger('click')` nào trong toàn tệp này.
    await wrapper.findAll('.pl-row-form')[0].trigger('submit')
    await wrapper.vm.$nextTick()

    expect((wrapper.find('.pl-rename-form input').element as HTMLInputElement).value).toBe('Bao chi')
    expect((wrapper.find('.pl-body-form textarea').element as HTMLTextAreaElement).value).toBe('cau ngan chu dong')

    wrapper.unmount()
    promptLibraryState.resetPromptLibrary()
    promptSetState.resetPromptSets()
  })

  it('tạo một bộ mới ⇒ gọi `promptSetCreate` rồi CHỌN NGAY bộ vừa tạo', async () => {
    const { promptSetState, promptLibraryState, PromptLibraryOverlay } = await freshOverlay()
    promptSetListMock.mockResolvedValueOnce({ sets: [], workTierAvailable: false, error: null })
    promptLibraryState.openPromptLibrary()
    await flushPromises()

    const wrapper = mount(PromptLibraryOverlay, { attachTo: document.body })
    await wrapper.vm.$nextTick()

    // Form "Bộ mới" là CUỐI trong `.pl-row-form` — ở ca này là DUY NHẤT (0 hàng thật).
    await wrapper.find('.pl-row-form').trigger('submit')
    await wrapper.vm.$nextTick()
    expect(wrapper.find('.pl-create-form').exists()).toBe(true)

    await wrapper.find('.pl-create-form input.pl-input').setValue('Hoc thuat')
    await wrapper.find('.pl-create-form textarea').setValue('giu thuat ngu chuyen nganh')

    promptSetCreateMock.mockResolvedValue({
      id: 42,
      warnings: { unknown_markers: [], glossary_terms_missing: true },
      error: null,
    })
    promptSetListMock.mockResolvedValueOnce({
      sets: [row({ id: 42, name: 'Hoc thuat', body: 'giu thuat ngu chuyen nganh' })],
      workTierAvailable: false,
      error: null,
    })

    await wrapper.find('.pl-create-form').trigger('submit')
    await flushPromises()
    await wrapper.vm.$nextTick()

    // Không tầng — mặc định rơi về `global` vì `workTierAvailable` là `false` ở ca này.
    expect(promptSetCreateMock).toHaveBeenCalledWith('global', 'Hoc thuat', 'giu thuat ngu chuyen nganh')
    expect(wrapper.find('.pl-create-form').exists()).toBe(false)
    expect((wrapper.find('.pl-rename-form input').element as HTMLInputElement).value).toBe('Hoc thuat')

    wrapper.unmount()
    promptLibraryState.resetPromptLibrary()
    promptSetState.resetPromptSets()
  })

  it('review fix — tạo một bộ Global bị một bộ Work cùng tên che NGAY LẬP TỨC ⇒ chọn thẳng hàng Global vừa tạo qua shadowed_id, không rơi về createShadowedNote', async () => {
    const { promptSetState, promptLibraryState, PromptLibraryOverlay } = await freshOverlay()
    promptSetListMock.mockResolvedValueOnce({ sets: [], workTierAvailable: false, error: null })
    promptLibraryState.openPromptLibrary()
    await flushPromises()

    const wrapper = mount(PromptLibraryOverlay, { attachTo: document.body })
    await wrapper.vm.$nextTick()

    await wrapper.find('.pl-row-form').trigger('submit')
    await wrapper.vm.$nextTick()
    expect(wrapper.find('.pl-create-form').exists()).toBe(true)

    await wrapper.find('.pl-create-form input.pl-input').setValue('Hoc thuat')
    await wrapper.find('.pl-create-form textarea').setValue('giu thuat ngu chuyen nganh')

    promptSetCreateMock.mockResolvedValue({
      id: 99,
      warnings: { unknown_markers: [], glossary_terms_missing: false },
      error: null,
    })
    // Lượt nạp lại SAU khi tạo chỉ trả đúng MỘT hàng: hàng Work cùng tên, che ngay hàng Global
    // vừa tạo (Quyết định #1) — không hàng `tier: 'global'` nào khớp `name`, nên `created` ở
    // `onSubmitCreate` là `undefined` và nhánh rơi (Quyết định #3) phải chạy.
    promptSetListMock.mockResolvedValueOnce({
      sets: [
        row({
          id: 5,
          name: 'Hoc thuat',
          tier: 'work',
          body: 'than tac pham co san',
          shadowed_body: 'giu thuat ngu chuyen nganh',
          shadowed_id: 99,
        }),
      ],
      workTierAvailable: true,
      error: null,
    })

    await wrapper.find('.pl-create-form').trigger('submit')
    await flushPromises()
    await wrapper.vm.$nextTick()

    // Form Tạo đóng và KHÔNG rơi về `createShadowedNote` — bằng chứng: không hint "đã tạo
    // nhưng không chọn được" nào hiện, và ô soạn đã nạp đúng hàng Global vừa tạo.
    expect(wrapper.find('.pl-create-form').exists()).toBe(false)
    expect((wrapper.find('.pl-rename-form input').element as HTMLInputElement).value).toBe('Hoc thuat')
    expect((wrapper.find('.pl-textarea').element as HTMLTextAreaElement).value).toBe('giu thuat ngu chuyen nganh')

    // Hàng Global bị che (huy hiệu `pl-row-shadowed`) phải là hàng ĐANG CHỌN — đúng
    // `(tier: 'global', id: shadowed_id)`, không phải hàng Work đang thắng.
    const shadowedButton = wrapper.find('.pl-row-shadowed')
    expect(shadowedButton.exists()).toBe(true)
    expect(shadowedButton.classes()).toContain('pl-row-on')

    wrapper.unmount()
    promptLibraryState.resetPromptLibrary()
    promptSetState.resetPromptSets()
  })

  it('đổi tên ⇒ gọi `promptSetRename` với tên đã TRIM, và Đổi tên là thao tác RIÊNG khỏi Sửa thân', async () => {
    const { promptSetState, promptLibraryState, PromptLibraryOverlay } = await freshOverlay()
    promptSetListMock.mockResolvedValueOnce({
      sets: [row({ id: 3, name: 'Cu', body: 'than cu' })],
      workTierAvailable: false,
      error: null,
    })
    promptLibraryState.openPromptLibrary()
    await flushPromises()

    const wrapper = mount(PromptLibraryOverlay, { attachTo: document.body })
    await wrapper.vm.$nextTick()
    await wrapper.findAll('.pl-row-form')[0].trigger('submit')
    await wrapper.vm.$nextTick()

    await wrapper.find('.pl-rename-form input').setValue('  Moi  ')

    promptSetRenameMock.mockResolvedValue(null)
    promptSetListMock.mockResolvedValueOnce({
      sets: [row({ id: 3, name: 'Moi', body: 'than cu' })],
      workTierAvailable: false,
      error: null,
    })

    await wrapper.find('.pl-rename-form').trigger('submit')
    await flushPromises()
    await wrapper.vm.$nextTick()

    expect(promptSetRenameMock).toHaveBeenCalledWith('global', 3, 'Moi')
    expect(promptSetUpdateBodyMock).not.toHaveBeenCalled()

    wrapper.unmount()
    promptLibraryState.resetPromptLibrary()
    promptSetState.resetPromptSets()
  })

  it('sửa thân ⇒ cảnh báo dấu ngoặc CỦA RUST hiện đúng câu, GỌI TÊN token lạ (Quyết định #3)', async () => {
    const { promptSetState, promptLibraryState, PromptLibraryOverlay } = await freshOverlay()
    promptSetListMock.mockResolvedValueOnce({
      sets: [row({ id: 3, name: 'Cu', body: 'than cu' })],
      workTierAvailable: false,
      error: null,
    })
    promptLibraryState.openPromptLibrary()
    await flushPromises()

    const wrapper = mount(PromptLibraryOverlay, { attachTo: document.body })
    await wrapper.vm.$nextTick()
    await wrapper.findAll('.pl-row-form')[0].trigger('submit')
    await wrapper.vm.$nextTick()

    await wrapper.find('.pl-body-form textarea').setValue('{{glosary_terms}} sai chinh ta')

    promptSetUpdateBodyMock.mockResolvedValue({
      warnings: { unknown_markers: ['{{glosary_terms}}'], glossary_terms_missing: true },
      error: null,
    })
    promptSetListMock.mockResolvedValueOnce({
      sets: [row({ id: 3, name: 'Cu', body: '{{glosary_terms}} sai chinh ta' })],
      workTierAvailable: false,
      error: null,
    })

    await wrapper.find('.pl-body-form textarea').trigger('blur')
    await flushPromises()
    await wrapper.vm.$nextTick()

    expect(promptSetUpdateBodyMock).toHaveBeenCalledWith('global', 3, '{{glosary_terms}} sai chinh ta')
    expect(wrapper.text()).toContain('{{glosary_terms}}')
    expect(wrapper.text()).toContain('glossary_terms')

    wrapper.unmount()
    promptLibraryState.resetPromptLibrary()
    promptSetState.resetPromptSets()
  })

  it('blur lưu thân đang bay rồi Nhân bản ngay ⇒ đúng MỘT bản sao, tạo sau khi lưu xong', async () => {
    const { promptSetState, promptLibraryState, PromptLibraryOverlay } = await freshOverlay()
    promptSetListMock.mockResolvedValueOnce({
      sets: [row({ id: 3, name: 'Cu', body: 'than cu' })],
      workTierAvailable: false,
      error: null,
    })
    promptLibraryState.openPromptLibrary()
    await flushPromises()

    const wrapper = mount(PromptLibraryOverlay, { attachTo: document.body })
    await wrapper.vm.$nextTick()
    await wrapper.findAll('.pl-row-form')[0].trigger('submit')
    await wrapper.vm.$nextTick()
    await wrapper.find('.pl-body-form textarea').setValue('than moi')

    let finishSave: (v: unknown) => void = () => undefined
    promptSetUpdateBodyMock.mockReturnValue(
      new Promise((resolve) => {
        finishSave = resolve
      }),
    )
    promptSetCreateMock.mockResolvedValue({ warnings: { unknown_markers: [], glossary_terms_missing: false }, error: null })
    const saved = [row({ id: 3, name: 'Cu', body: 'than moi' })]
    promptSetListMock.mockResolvedValue({ sets: saved, workTierAvailable: false, error: null })

    await wrapper.find('.pl-body-form textarea').trigger('blur')
    await flushPromises()
    const dup = wrapper.find('.pl-dup-form')
    expect(dup.find('button').attributes('disabled')).toBeUndefined()
    await dup.trigger('submit')
    await flushPromises()
    expect(promptSetCreateMock).not.toHaveBeenCalled()

    finishSave({ warnings: { unknown_markers: [], glossary_terms_missing: false }, error: null })
    await flushPromises()

    expect(promptSetUpdateBodyMock).toHaveBeenCalledTimes(1)
    expect(promptSetCreateMock).toHaveBeenCalledTimes(1)
    expect(promptSetCreateMock.mock.calls[0][2]).toBe('than moi')

    wrapper.unmount()
    promptLibraryState.resetPromptLibrary()
    promptSetState.resetPromptSets()
  })

  it('Phase 4c AC6: màn soạn render ĐÚNG danh sách biến `promptSetList` trả về, KHÁC ba tên ratify thật — chứng minh `PROMPT_VARIABLES` gõ tay đã bị xoá, không còn sống sót dưới một tên khác', async () => {
    const { promptSetState, promptLibraryState, PromptLibraryOverlay } = await freshOverlay()
    promptSetListMock.mockResolvedValue({
      sets: [row({ id: 3, name: 'Cu', body: 'than cu' })],
      workTierAvailable: false,
      // Cố ý KHÁC glossary_terms/source_segment/tm_similar_segments — nếu component còn
      // một mảng gõ tay của riêng nó (dù đổi tên biến), màn hình vẫn sẽ hiện BA tên ratify
      // thật thay vì hai tên giả này, và ca này đỏ.
      variables: ['mock_alpha', 'mock_beta'],
      error: null,
    })
    promptLibraryState.openPromptLibrary()
    await flushPromises()

    const wrapper = mount(PromptLibraryOverlay, { attachTo: document.body })
    await wrapper.vm.$nextTick()
    await wrapper.findAll('.pl-row-form')[0].trigger('submit') // chọn hàng 'Cu' để mở khối biến
    await wrapper.vm.$nextTick()

    const rows = wrapper.findAll('.pl-vrow')
    expect(rows).toHaveLength(2)
    const markers = rows.map((r) => r.find('code').text())
    expect(markers).toEqual(['{{mock_alpha}}', '{{mock_beta}}'])
    expect(wrapper.text()).not.toContain('glossary_terms')
    expect(wrapper.text()).not.toContain('source_segment')
    expect(wrapper.text()).not.toContain('tm_similar_segments')

    wrapper.unmount()
    promptLibraryState.resetPromptLibrary()
    promptSetState.resetPromptSets()
  })

  it('Phase 4c review fix: mọi tên trong `promptSetList().variables` phải giải thành CHỮ THẬT qua `t()`, không phải khoá `prompt.library.var_<name>_desc` thô — `varDescKey` ghép chuỗi nên không cổng tĩnh nào lần được nó, chỉ một lượt dựng thật mới bắt được một tên thiếu khoá vi.json (ca sẽ tái diễn nếu Story 4.6 thêm một biến mà quên thêm khoá)', async () => {
    const { promptSetState, promptLibraryState, PromptLibraryOverlay } = await freshOverlay()
    const names = ['glossary_terms', 'source_segment', 'tm_similar_segments']
    promptSetListMock.mockResolvedValue({
      sets: [row({ id: 3, name: 'Cu', body: 'than cu' })],
      workTierAvailable: false,
      variables: names,
      error: null,
    })
    promptLibraryState.openPromptLibrary()
    await flushPromises()

    const wrapper = mount(PromptLibraryOverlay, { attachTo: document.body })
    await wrapper.vm.$nextTick()
    await wrapper.findAll('.pl-row-form')[0].trigger('submit')
    await wrapper.vm.$nextTick()

    const rows = wrapper.findAll('.pl-vrow')
    expect(rows).toHaveLength(names.length)
    for (const [i, name] of names.entries()) {
      const desc = rows[i]?.find('.pl-vd').text() ?? ''
      // `resolve.ts` trả NGUYÊN VĂN khoá khi khoá thiếu (AC4) — đây chính là dấu hiệu một biến
      // mới thiếu chuỗi vi.json tương ứng: mô tả không được PHÉP bằng đúng cái khoá thô.
      expect(desc).not.toBe(`prompt.library.var_${name}_desc`)
      expect(desc.length).toBeGreaterThan(0)
    }

    wrapper.unmount()
    promptLibraryState.resetPromptLibrary()
    promptSetState.resetPromptSets()
  })

  it('xoá hai nhịp — nhịp MỘT chỉ đặt cờ chờ (0 lượt IPC), nhịp HAI mới gọi `promptSetDelete` thật', async () => {
    const { promptSetState, promptLibraryState, PromptLibraryOverlay } = await freshOverlay()
    promptSetListMock.mockResolvedValueOnce({
      sets: [row({ id: 9, name: 'Xoa toi', body: 'x' })],
      workTierAvailable: false,
      error: null,
    })
    promptLibraryState.openPromptLibrary()
    await flushPromises()

    const wrapper = mount(PromptLibraryOverlay, { attachTo: document.body })
    await wrapper.vm.$nextTick()
    await wrapper.findAll('.pl-row-form')[0].trigger('submit')
    await wrapper.vm.$nextTick()

    const deleteForm = wrapper.find('.pl-delete-form')
    await deleteForm.trigger('submit') // nhịp MỘT
    await wrapper.vm.$nextTick()

    expect(promptSetDeleteMock).not.toHaveBeenCalled()
    expect(wrapper.text()).toContain('Xác nhận xoá vĩnh viễn')

    promptSetDeleteMock.mockResolvedValue(null)
    promptSetListMock.mockResolvedValueOnce({ sets: [], workTierAvailable: false, error: null })

    await deleteForm.trigger('submit') // nhịp HAI
    await flushPromises()
    await wrapper.vm.$nextTick()

    expect(promptSetDeleteMock).toHaveBeenCalledWith('global', 9)
    // Lựa chọn tự xoá sau lượt xoá thành công — form soạn không còn.
    expect(wrapper.find('.pl-rename-form').exists()).toBe(false)

    wrapper.unmount()
    promptLibraryState.resetPromptLibrary()
    promptSetState.resetPromptSets()
  })
})

describe('PromptLibraryOverlay.vue — phạm vi, Đang dùng, nhãn biến, tô biến, Nhân bản, tự lưu (ticket 14)', () => {
  async function openLibrary(sets: WireRow[], extra: Record<string, unknown> = {}) {
    const ctx = await freshOverlay()
    promptSetListMock.mockResolvedValue({ sets, workTierAvailable: true, variables: ['glossary_terms', 'source_segment', 'tm_similar_segments'], error: null, ...extra })
    const commands = await import('../../src/commands')
    const openInspector = vi.fn()
    const closeLibrary = vi.fn()
    commands.installCommands({ isMac: true, setMode: () => {}, openAiPromptInspector: openInspector, closePromptLibrary: closeLibrary } as never)
    const i18n = await import('../../src/i18n')
    ctx.promptLibraryState.openPromptLibrary()
    await flushPromises()
    const wrapper = mount(ctx.PromptLibraryOverlay, { attachTo: document.body })
    await wrapper.vm.$nextTick()
    return { ...ctx, wrapper, i18n, openInspector, closeLibrary }
  }

  async function pick(wrapper: Awaited<ReturnType<typeof openLibrary>>['wrapper'], index = 0) {
    await wrapper.findAll('.pl-row-form')[index].trigger('submit')
    await flushPromises()
  }

  it('phạm vi tầng mang số hàng mỗi tầng; chọn Tác phẩm chỉ còn hàng Tác phẩm', async () => {
    const { wrapper, i18n } = await openLibrary([
      row({ id: 5, name: 'W1', tier: 'work', body: 'a' }),
      row({ id: 6, name: 'W2', tier: 'work', body: 'b' }),
      row({ id: 7, name: 'G1', tier: 'global', body: 'c' }),
    ])
    const labels = wrapper.findAll('input[name="pl-scope"]').map((el) => el.element.parentElement?.textContent.trim())
    expect(labels).toEqual([
      i18n.t('prompt.library.count_label', { label: i18n.t('prompt.library.tier_global'), count: '1' }),
      i18n.t('prompt.library.count_label', { label: i18n.t('prompt.library.tier_work'), count: '2' }),
      i18n.t('prompt.library.scope_both'),
    ])
    await wrapper.get('input[name="pl-scope"][value="work"]').setValue(true)
    await flushPromises()
    const names = wrapper.findAll('.pl-row-name').map((n) => n.text())
    expect(names).toEqual(['W1', 'W2'])
    wrapper.unmount()
  })

  it('dòng "Đang dùng" nói chưa chọn bộ nào, rồi nêu tên bộ sau khi chọn dùng', async () => {
    const { wrapper, promptSetState, i18n } = await openLibrary([row({ id: 3, name: 'Cu', body: 'x' })])
    expect(wrapper.get('[data-prompt-in-use]').text()).toBe(i18n.t('prompt.library.in_use_none'))
    promptSetState.setSelectedPromptSetName('Cu')
    await flushPromises()
    expect(wrapper.get('[data-prompt-in-use]').text()).toBe(i18n.t('prompt.library.in_use_line', { name: 'Cu' }))
    wrapper.unmount()
  })

  it('ba biến mang nhãn bắt buộc có / nên có, không hàng thứ tư', async () => {
    const { wrapper, i18n } = await openLibrary([row({ id: 3, name: 'Cu', body: 'x' })])
    await pick(wrapper)
    const rows = wrapper.findAll('.pl-vrow')
    expect(rows.map((r) => r.get('.pl-vs').text())).toEqual([
      i18n.t('prompt.library.var_glossary_terms_need'),
      i18n.t('prompt.library.var_source_segment_need'),
      i18n.t('prompt.library.var_tm_similar_segments_need'),
    ])
    wrapper.unmount()
  })

  it('lớp phản chiếu tô biến đã biết và biến lạ, và chữ của nó khớp từng ký tự với ô gõ', async () => {
    const body = 'Dich {{glossary_terms}} va {{la_hoac}} xong'
    const { wrapper } = await openLibrary([row({ id: 3, name: 'Cu', body })])
    await pick(wrapper)
    const mirror = wrapper.get('.pl-ta-mirror')
    expect(mirror.text()).toBe(body)
    expect(mirror.findAll('.pl-var').map((e) => e.text())).toEqual(['{{glossary_terms}}', '{{la_hoac}}'])
    expect(mirror.findAll('.pl-var-unknown').map((e) => e.text())).toEqual(['{{la_hoac}}'])
    wrapper.unmount()
  })

  it('thân không đổi thì rời ô không ghi gì', async () => {
    const { wrapper } = await openLibrary([row({ id: 3, name: 'Cu', body: 'than cu' })])
    await pick(wrapper)
    await wrapper.get('.pl-body-form textarea').trigger('blur')
    await flushPromises()
    expect(promptSetUpdateBodyMock).not.toHaveBeenCalled()
    wrapper.unmount()
  })

  it('chọn bộ khác khi thân đang sửa dở ⇒ lưu thân cũ trước, đúng bộ cũ', async () => {
    const { wrapper } = await openLibrary([
      row({ id: 3, name: 'Cu', body: 'than cu' }),
      row({ id: 4, name: 'Moi', body: 'than moi' }),
    ])
    await pick(wrapper, 0)
    await wrapper.get('.pl-body-form textarea').setValue('sua do')
    promptSetUpdateBodyMock.mockResolvedValue({ warnings: { unknown_markers: [], glossary_terms_missing: false }, error: null })
    await pick(wrapper, 1)
    expect(promptSetUpdateBodyMock).toHaveBeenCalledExactlyOnceWith('global', 3, 'sua do')
    wrapper.unmount()
  })

  it('lưu lỗi thì bản nháp còn nguyên trong ô và không rời sang bộ khác', async () => {
    const { wrapper } = await openLibrary([
      row({ id: 3, name: 'Cu', body: 'than cu' }),
      row({ id: 4, name: 'Moi', body: 'than moi' }),
    ])
    await pick(wrapper, 0)
    await wrapper.get('.pl-body-form textarea').setValue('sua do')
    promptSetUpdateBodyMock.mockResolvedValue({ warnings: null, error: { code: 'x.fail', message_key: 'err.unknown', params: {}, retryable: false } })
    await pick(wrapper, 1)
    const area = wrapper.get('.pl-body-form textarea').element as HTMLTextAreaElement
    expect(area.value).toBe('sua do')
    expect(wrapper.get('.pl-rename-form input').element).toHaveProperty('value', 'Cu')
    wrapper.unmount()
  })

  it('Xem prompt lưu thân đang sửa trước rồi mới mở', async () => {
    const { wrapper, openInspector } = await openLibrary([row({ id: 3, name: 'Cu', body: 'than cu' })])
    await pick(wrapper)
    await wrapper.get('.pl-body-form textarea').setValue('sua do')
    promptSetUpdateBodyMock.mockImplementation(async () => {
      expect(openInspector).not.toHaveBeenCalled()
      return { warnings: { unknown_markers: [], glossary_terms_missing: false }, error: null }
    })
    await wrapper.get('.pl-inspect-form').trigger('submit')
    await flushPromises()
    expect(promptSetUpdateBodyMock).toHaveBeenCalledExactlyOnceWith('global', 3, 'sua do')
    expect(openInspector).toHaveBeenCalledTimes(1)
    wrapper.unmount()
  })

  it.each([
    ['Esc', async (w: Awaited<ReturnType<typeof openLibrary>>['wrapper']) => w.get('.pl-scrim').trigger('keydown', { key: 'Escape' })],
    ['nút Đóng', async (w: Awaited<ReturnType<typeof openLibrary>>['wrapper']) => w.get('.pl-head form').trigger('submit')],
  ])('đóng lớp phủ bằng %s lưu thân đang sửa trước rồi mới đóng', async (_name, closeBy) => {
    const { wrapper, closeLibrary } = await openLibrary([row({ id: 3, name: 'Cu', body: 'than cu' })])
    await pick(wrapper)
    await wrapper.get('.pl-body-form textarea').setValue('sua do')
    promptSetUpdateBodyMock.mockImplementation(async () => {
      expect(closeLibrary).not.toHaveBeenCalled()
      return { warnings: { unknown_markers: [], glossary_terms_missing: false }, error: null }
    })
    await closeBy(wrapper)
    await flushPromises()
    expect(promptSetUpdateBodyMock).toHaveBeenCalledExactlyOnceWith('global', 3, 'sua do')
    expect(closeLibrary).toHaveBeenCalledTimes(1)
    wrapper.unmount()
  })

  it('Nhân bản lưu thân đang sửa trước, tạo "<tên> (bản sao)" cùng tầng với thân đó, bộ gốc không bị đổi', async () => {
    const { wrapper } = await openLibrary([row({ id: 3, name: 'A', body: 'than cu' })])
    await pick(wrapper)
    await wrapper.get('.pl-body-form textarea').setValue('sua do')
    const order: string[] = []
    promptSetUpdateBodyMock.mockImplementation(async () => {
      order.push('update')
      return { warnings: { unknown_markers: [], glossary_terms_missing: false }, error: null }
    })
    promptSetCreateMock.mockImplementation(async () => {
      order.push('create')
      return { warnings: { unknown_markers: [], glossary_terms_missing: false }, error: null }
    })
    await wrapper.get('.pl-dup-form').trigger('submit')
    await flushPromises()
    expect(order).toEqual(['update', 'create'])
    expect(promptSetCreateMock).toHaveBeenCalledExactlyOnceWith('global', 'A (bản sao)', 'sua do')
    wrapper.unmount()
  })

  it('name_taken ngoài dự kiến ⇒ thử số kế "(bản sao 2)"; lỗi khác thì dừng và không tạo gì thêm', async () => {
    const { wrapper } = await openLibrary([row({ id: 3, name: 'A', body: 'x' })])
    await pick(wrapper)
    const taken = { warnings: null, error: { code: 'prompt_set.name_taken', message_key: 'err.unknown', params: {}, retryable: false } }
    promptSetCreateMock.mockResolvedValueOnce(taken)
    promptSetCreateMock.mockResolvedValueOnce({ warnings: { unknown_markers: [], glossary_terms_missing: false }, error: null })
    await wrapper.get('.pl-dup-form').trigger('submit')
    await flushPromises()
    expect(promptSetCreateMock.mock.calls.map((c) => c[1])).toEqual(['A (bản sao)', 'A (bản sao 2)'])

    promptSetCreateMock.mockReset()
    promptSetCreateMock.mockResolvedValue({ warnings: null, error: { code: 'x.fail', message_key: 'err.unknown', params: {}, retryable: false } })
    await wrapper.get('.pl-dup-form').trigger('submit')
    await flushPromises()
    expect(promptSetCreateMock).toHaveBeenCalledTimes(1)
    wrapper.unmount()
  })
})

describe('promptMarkers.splitPromptBody và duplicatePromptSetName', () => {
  it('khớp fixture dùng chung với scan_markers của Rust, từng ca', async () => {
    const { readFileSync } = await import('node:fs')
    const { splitPromptBody } = await import('../../src/promptMarkers')
    const fixture = JSON.parse(readFileSync('tests/fixtures/prompt-variable-markers.json', 'utf8')) as {
      known: string[]
      cases: { name: string; body: string; unknown_markers: string[]; known_markers: string[] }[]
    }
    expect(fixture.cases.length).toBeGreaterThanOrEqual(8)
    for (const c of fixture.cases) {
      const parts = splitPromptBody(c.body, fixture.known)
      expect(parts.map((p) => p.text).join(''), c.name).toBe(c.body)
      const unknown = [...new Set(parts.filter((p) => p.kind === 'unknown').map((p) => p.text))]
      expect(unknown, c.name).toEqual(c.unknown_markers)
      expect(parts.filter((p) => p.kind === 'known').map((p) => p.text), c.name).toEqual(c.known_markers)
    }
  })

  it('tách đúng: plain / known / unknown, ghép lại bằng đúng chuỗi gốc', async () => {
    const { splitPromptBody } = await import('../../src/promptMarkers')
    const body = 'a {{x}} b {{y}}{{x}} {{ }} {{z'
    const parts = splitPromptBody(body, ['x'])
    expect(parts.map((p) => p.text).join('')).toBe(body)
    expect(parts.filter((p) => p.kind === 'known').map((p) => p.text)).toEqual(['{{x}}', '{{x}}'])
    expect(parts.filter((p) => p.kind === 'unknown').map((p) => p.text)).toContain('{{y}}')
  })

  it('sinh tên bản sao: bỏ qua tên đã có, rồi bỏ qua `skip` ứng viên đầu', async () => {
    const { duplicatePromptSetName } = await import('../../src/promptSetState')
    expect(duplicatePromptSetName('A', 'bản sao', new Set())).toBe('A (bản sao)')
    expect(duplicatePromptSetName('A', 'bản sao', new Set(['A (bản sao)']))).toBe('A (bản sao 2)')
    expect(duplicatePromptSetName('A', 'bản sao', new Set(['A (bản sao)', 'A (bản sao 3)']))).toBe('A (bản sao 2)')
    expect(duplicatePromptSetName('A', 'bản sao', new Set(), 1)).toBe('A (bản sao 2)')
  })
})

// ═══════════════════════════════════════════════════════════════════════════════════════
// AiTranslationPanel.vue — đóng nửa CÒN LẠI của I/O Matrix "Switch effective set from AI
// panel — effective set changes without opening Settings". Xem doc-comment đầu tệp.
// ═══════════════════════════════════════════════════════════════════════════════════════
describe('AiTranslationPanel.vue — đổi bộ hiệu lực từ CHÍNH panel, không mở Cài đặt (I/O Matrix spec 4.4)', () => {
  it('chọn một bộ trong `<select>` của panel ⇒ `selectedPromptSetName` đổi ngay, 0 lượt `invoke` — và `SettingsOverlay.vue` không hề được nạp trong ca này', async () => {
    vi.resetModules()
    resetMocks()
    promptSetListMock.mockResolvedValue({
      sets: [
        row({ id: 1, name: 'Tien hiep', body: 'a' }),
        row({ id: 2, name: 'Bao chi', body: 'b' }),
      ],
      workTierAvailable: false,
      error: null,
    })

    const promptSetState = await import('../../src/promptSetState')
    const AiTranslationPanel = (await import('../../src/panels/AiTranslationPanel.vue')).default

    const wrapper = mount(AiTranslationPanel, {
      props: { params: { params: {} } },
      attachTo: document.body,
    })
    await flushPromises()
    await wrapper.vm.$nextTick()

    expect(promptSetState.selectedPromptSetName.value).toBeNull()
    expect(wrapper.text()).toContain('Chưa chọn bộ nào')

    const select = wrapper.find('select.ai-prompt-select')
    expect(select.exists()).toBe(true)
    await select.setValue('Bao chi')

    // 0 vòng IPC: `setSelectedPromptSetName` (`promptSetState.ts`) không gọi `invoke` —
    // đây chính là mệnh đề "effective set changes without opening Settings" đo được, vì
    // `promptSetListMock` chỉ được gọi đúng MỘT lần (lúc `onMounted`), không thêm lần nào
    // sau lượt đổi lựa chọn.
    expect(promptSetListMock).toHaveBeenCalledTimes(1)
    expect(promptSetState.selectedPromptSetName.value).toBe('Bao chi')
    expect(wrapper.text()).toContain('Đang dùng: Bao chi')

    wrapper.unmount()
    promptSetState.resetPromptSets()
  })

  it('chưa có bộ prompt nào ⇒ mời tạo, không một `<select>` rỗng câm lặng', async () => {
    vi.resetModules()
    resetMocks()
    promptSetListMock.mockResolvedValue({ sets: [], workTierAvailable: false, error: null })

    const promptSetState = await import('../../src/promptSetState')
    const AiTranslationPanel = (await import('../../src/panels/AiTranslationPanel.vue')).default

    const wrapper = mount(AiTranslationPanel, {
      props: { params: { params: {} } },
      attachTo: document.body,
    })
    await flushPromises()
    await wrapper.vm.$nextTick()

    expect(wrapper.find('select.ai-prompt-select').exists()).toBe(false)
    expect(wrapper.text()).toContain('Chưa có bộ prompt nào')

    wrapper.unmount()
    promptSetState.resetPromptSets()
  })
})
