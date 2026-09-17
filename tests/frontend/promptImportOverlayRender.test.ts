/**
 * Dựng THẬT của `PromptImportOverlay.vue` VÀ `promptSetImportState.ts` — Story 4.5
 * (FR79/NFR9, AD-48).
 *
 * ⚠️ `commands` (`vi.mock`) — `dispatch()` NÉM với một id chưa đăng ký (AC1 của
 * `CommandRegistry`), và tệp này không dựng `installCommands()`; thay bằng một spy để đối
 * chứng mỗi `@click`/`@keydown.esc` phát ĐÚNG id, cùng khuôn
 * `glossaryConfirmStripTemplate.test.ts`. Mỗi `@click` trong `PromptImportOverlay.vue` là
 * đúng MỘT `dispatch('<id>')` — Enter/Space trên một `<button>` focus được phát cùng sự kiện
 * DOM `click` mà chuột phát, nên đối chứng qua `.trigger('click')` chứng minh được cả đường
 * bàn phím.
 */
import { beforeEach, describe, expect, it, vi } from 'vitest'
import { mount } from '@vue/test-utils'

const openPreviewMock = vi.fn()
const confirmImportMock = vi.fn()
const cancelImportMock = vi.fn()
const dispatchMock = vi.fn()

vi.mock('../../src/config/promptset', () => ({
  promptSetOpenImportPreview: (...args: unknown[]) => openPreviewMock(...args),
  promptSetConfirmImport: (...args: unknown[]) => confirmImportMock(...args),
  promptSetCancelImport: (...args: unknown[]) => cancelImportMock(...args),
}))

vi.mock('../../src/commands', () => ({
  dispatch: (...args: unknown[]) => dispatchMock(...args),
}))

type TierPreview = { kind: 'new' | 'identical' | 'conflict'; existing_body: string | null }
type Preview = {
  file_name: string
  name: string
  body: string
  warnings: { unknown_markers: string[]; glossary_terms_missing: boolean }
  global: TierPreview
  work: TierPreview | null
}

function preview(over: Partial<Preview> = {}): Preview {
  return {
    file_name: 'xianxia.prompt.md',
    name: 'Xianxia',
    body: 'Dich: {{source_segment}}',
    warnings: { unknown_markers: [], glossary_terms_missing: false },
    global: { kind: 'new', existing_body: null },
    work: null,
    ...over,
  }
}

async function freshState() {
  vi.resetModules()
  openPreviewMock.mockReset()
  confirmImportMock.mockReset()
  cancelImportMock.mockReset()
  dispatchMock.mockReset()
  return import('../../src/promptSetImportState')
}

beforeEach(() => {
  document.body.innerHTML = ''
})

describe('openPromptImportPreviewOverlay — nhịp một', () => {
  it('huỷ hộp thoại (outcome cancelled) không mở lớp phủ', async () => {
    const state = await freshState()
    openPreviewMock.mockResolvedValue({ outcome: 'cancelled' })

    await state.openPromptImportPreviewOverlay()

    expect(state.promptImportOverlayIsOpen.value).toBe(false)
  })

  it('lượt tải xong ⇒ mở lớp phủ, mặc định chọn tầng Work khi preview mang phân loại Work', async () => {
    const state = await freshState()
    openPreviewMock.mockResolvedValue({ outcome: 'loaded', preview: preview({ work: { kind: 'new', existing_body: null } }) })

    await state.openPromptImportPreviewOverlay()

    expect(state.promptImportOverlayIsOpen.value).toBe(true)
    expect(state.promptImportStatus.value).toBe('loaded')
    expect(state.promptImportSelectedTier.value).toBe('work')
  })

  it('không có Tác phẩm nào mở (work: null) ⇒ mặc định chọn tầng Global', async () => {
    const state = await freshState()
    openPreviewMock.mockResolvedValue({ outcome: 'loaded', preview: preview({ work: null }) })

    await state.openPromptImportPreviewOverlay()

    expect(state.promptImportSelectedTier.value).toBe('global')
  })
})

describe('PromptImportOverlay.vue — vẽ từ mô hình, chọn tầng và quyết định va chạm qua radiogroup', () => {
  it('hiện tên tệp/tên bộ/thân, và tầng Work bị vô hiệu hoá khi preview không mang phân loại Work', async () => {
    const state = await freshState()
    openPreviewMock.mockResolvedValue({ outcome: 'loaded', preview: preview({ work: null }) })
    await state.openPromptImportPreviewOverlay()

    const PromptImportOverlay = (await import('../../src/PromptImportOverlay.vue')).default
    const wrapper = mount(PromptImportOverlay, { attachTo: document.body })
    await wrapper.vm.$nextTick()

    expect(wrapper.find('.pi-scrim').exists()).toBe(true)
    expect(wrapper.text()).toContain('xianxia.prompt.md')
    expect(wrapper.text()).toContain('Xianxia')

    const radios = wrapper.findAll('input[type="radio"][name="pi-tier"]')
    expect(radios).toHaveLength(2)
    expect((radios[0]?.element as HTMLInputElement).checked).toBe(true) // global
    expect((radios[1]?.element as HTMLInputElement).disabled).toBe(true) // work vắng mặt

    wrapper.unmount()
  })

  it('chọn tầng Work khi preview mang phân loại Work, đổi qua radio', async () => {
    const state = await freshState()
    openPreviewMock.mockResolvedValue({
      outcome: 'loaded',
      preview: preview({ work: { kind: 'new', existing_body: null } }),
    })
    await state.openPromptImportPreviewOverlay()

    const PromptImportOverlay = (await import('../../src/PromptImportOverlay.vue')).default
    const wrapper = mount(PromptImportOverlay, { attachTo: document.body })
    await wrapper.vm.$nextTick()

    const radios = wrapper.findAll('input[type="radio"][name="pi-tier"]')
    expect((radios[1]?.element as HTMLInputElement).disabled).toBe(false)
    await radios[0]?.setValue(true)
    expect(state.promptImportSelectedTier.value).toBe('global')
    await radios[1]?.setValue(true)
    expect(state.promptImportSelectedTier.value).toBe('work')

    wrapper.unmount()
  })

  it('va chạm tên (conflict) hiện cả hai thân và một radiogroup quyết định, mặc định giữ của tôi', async () => {
    const state = await freshState()
    openPreviewMock.mockResolvedValue({
      outcome: 'loaded',
      preview: preview({ global: { kind: 'conflict', existing_body: 'than dang co' } }),
    })
    await state.openPromptImportPreviewOverlay()

    const PromptImportOverlay = (await import('../../src/PromptImportOverlay.vue')).default
    const wrapper = mount(PromptImportOverlay, { attachTo: document.body })
    await wrapper.vm.$nextTick()

    expect(wrapper.text()).toContain('than dang co')
    const decisionRadios = wrapper.findAll('input[type="radio"][name="pi-decision"]')
    expect(decisionRadios).toHaveLength(2)
    expect((decisionRadios[0]?.element as HTMLInputElement).checked).toBe(true) // keep_mine mac dinh

    await decisionRadios[1]?.setValue(true)
    expect(state.promptImportDecision.value).toBe('take_theirs')

    wrapper.unmount()
  })

  it('cảnh báo dấu ngoặc của preview hiện ra khi có', async () => {
    const state = await freshState()
    openPreviewMock.mockResolvedValue({
      outcome: 'loaded',
      preview: preview({ warnings: { unknown_markers: ['{{glosary_terms}}'], glossary_terms_missing: true } }),
    })
    await state.openPromptImportPreviewOverlay()

    const PromptImportOverlay = (await import('../../src/PromptImportOverlay.vue')).default
    const wrapper = mount(PromptImportOverlay, { attachTo: document.body })
    await wrapper.vm.$nextTick()

    expect(wrapper.text()).toContain('{{glosary_terms}}')
    expect(wrapper.find('.pi-warn').exists()).toBe(true)

    wrapper.unmount()
  })

  it('mỗi @click là đúng một dispatch(id) — nút Xác nhận và Đóng/Huỷ, đường Enter/Space bàn phím', async () => {
    const state = await freshState()
    openPreviewMock.mockResolvedValue({ outcome: 'loaded', preview: preview() })
    await state.openPromptImportPreviewOverlay()

    const PromptImportOverlay = (await import('../../src/PromptImportOverlay.vue')).default
    const wrapper = mount(PromptImportOverlay, { attachTo: document.body })
    await wrapper.vm.$nextTick()

    await wrapper.find('.pi-close').trigger('click')
    expect(dispatchMock).toHaveBeenCalledWith('prompt.import.cancel')

    await wrapper.find('.pi-act-primary').trigger('click')
    expect(dispatchMock).toHaveBeenCalledWith('prompt.import.confirm')

    const cancelButtons = wrapper.findAll('.pi-act').filter((b) => !b.classes('pi-act-primary'))
    await cancelButtons[0]?.trigger('click')
    expect(dispatchMock).toHaveBeenCalledWith('prompt.import.cancel')

    wrapper.unmount()
  })
})

describe('confirmPromptImportPreview — nhịp hai', () => {
  it('tầng New ⇒ gửi decision null (không mang một quyết định vô nghĩa), đóng lớp phủ khi thành công', async () => {
    const state = await freshState()
    openPreviewMock.mockResolvedValue({ outcome: 'loaded', preview: preview() })
    await state.openPromptImportPreviewOverlay()
    confirmImportMock.mockResolvedValue({ outcome: 'inserted', error: null })

    await state.confirmPromptImportPreview()

    expect(confirmImportMock).toHaveBeenCalledWith('global', null)
    expect(state.promptImportOverlayIsOpen.value).toBe(false)
    expect(state.promptImportConfirmedOutcome.value).toBe('inserted')
  })

  it('tầng Conflict ⇒ gửi decision hiện tại, lượt trượt GIỮ lớp phủ mở và hiện lỗi', async () => {
    const state = await freshState()
    openPreviewMock.mockResolvedValue({
      outcome: 'loaded',
      preview: preview({ global: { kind: 'conflict', existing_body: 'x' } }),
    })
    await state.openPromptImportPreviewOverlay()
    state.setPromptImportDecision('take_theirs')
    const err = { code: 'prompt_set.import_stale_conflict', message_key: 'err.prompt_set.import_stale_conflict', params: {}, retryable: false }
    confirmImportMock.mockResolvedValue({ outcome: null, error: err })

    await state.confirmPromptImportPreview()

    expect(confirmImportMock).toHaveBeenCalledWith('global', 'take_theirs')
    expect(state.promptImportOverlayIsOpen.value).toBe(true)
    expect(state.promptImportConfirmError.value).toEqual(err)
  })
})

describe('cancelPromptImportPreview', () => {
  it('đóng lớp phủ NGAY, không đợi lượt gọi Rust trả về', async () => {
    const state = await freshState()
    openPreviewMock.mockResolvedValue({ outcome: 'loaded', preview: preview() })
    await state.openPromptImportPreviewOverlay()
    let resolveCancel: (value: null) => void = () => {}
    cancelImportMock.mockReturnValue(new Promise((resolve) => (resolveCancel = resolve)))

    const promise = state.cancelPromptImportPreview()
    expect(state.promptImportOverlayIsOpen.value).toBe(false)

    resolveCancel(null)
    await promise
  })
})

describe('resetPromptImport', () => {
  it('vứt toàn bộ state về mặc định', async () => {
    const state = await freshState()
    openPreviewMock.mockResolvedValue({ outcome: 'loaded', preview: preview() })
    await state.openPromptImportPreviewOverlay()
    state.setPromptImportDecision('take_theirs')

    state.resetPromptImport()

    expect(state.promptImportOverlayIsOpen.value).toBe(false)
    expect(state.promptImportStatus.value).toBe('unknown')
    expect(state.promptImportPreview.value).toBeNull()
    expect(state.promptImportSelectedTier.value).toBe('global')
    expect(state.promptImportDecision.value).toBe('keep_mine')
  })
})
