/**
 * Dựng THẬT của `SettingsOverlay.vue` cho HÀNG **Khoá API** — Story 4.3 (FR65/FR67, NFR11).
 *
 * ─────────────────────────────────────────────────────────────────────────────
 * 🔴 VÌ SAO CA NÀY TỒN TẠI, KHÔNG CHỈ `aiConfigState.test.ts`
 * ─────────────────────────────────────────────────────────────────────────────
 * `aiConfigKeyStatusKey()` (ba trạng thái: đã cấu hình / chưa cấu hình / không kiểm tra
 * được) sống TRONG `<script setup>` của `SettingsOverlay.vue`, không export được — cùng tình
 * huống Story 6.3 đã gặp với `confidenceMessageKey`/`tierEmptyMessageKey`
 * (xem `importPreviewOverlayRender.test.ts`). `aiConfigState.test.ts` chỉ canh
 * `aiConfigKeyConfigured.value` (dữ liệu), không canh CHỮ nào thật sự lên màn — một lượt đổi
 * hàm ánh xạ trong `.vue` (đảo nhánh `true`/`false`/`null`, hay gõ nhầm một khoá i18n) không
 * làm ca đó đỏ. Đối chứng DUY NHẤT thấy được là dựng component thật và đọc `wrapper.text()`.
 *
 * Hai hàng I/O Matrix spec 4.3 có nửa UI chỉ đóng được ở TỆP NÀY — "No key anywhere" và
 * "Save a key" — cộng một ca bắt buộc thứ ba không nằm trong bảng theo tên riêng nhưng được
 * §Boundaries/Phase 2 đòi rõ: `key_configured: null` phải dựng một câu THỨ BA, không được
 * trùng câu "chưa cấu hình" của `false`.
 *
 * ⚠️ Cùng khuôn `importPreviewOverlayRender.test.ts::freshOverlay` — `vi.resetModules()`
 * TRƯỚC, rồi nạp ĐỘNG cả state lẫn component trong CÙNG một lượt, để `SettingsOverlay.vue`
 * và `settingsState.ts`/`aiConfigState.ts` dùng chung một thể hiện module.
 */
import { beforeEach, describe, expect, it, vi } from 'vitest'
import { flushPromises, mount } from '@vue/test-utils'

const aiConfigGetMock = vi.fn()
const aiConfigSaveFieldMock = vi.fn()
const aiConfigClearOverrideMock = vi.fn()
const aiConfigSaveKeyMock = vi.fn()
const aiConfigDeleteKeyMock = vi.fn()

vi.mock('../../src/config/aiconfig', () => ({
  aiConfigGet: (...args: unknown[]) => aiConfigGetMock(...args),
  aiConfigSaveField: (...args: unknown[]) => aiConfigSaveFieldMock(...args),
  aiConfigClearOverride: (...args: unknown[]) => aiConfigClearOverrideMock(...args),
  aiConfigSaveKey: (...args: unknown[]) => aiConfigSaveKeyMock(...args),
  aiConfigDeleteKey: (...args: unknown[]) => aiConfigDeleteKeyMock(...args),
}))

// `settingsState.ts::openSettings()` mặc định mở vào mục `privacy`, gọi `listDomainLog()`
// trước khi ca này chuyển sang `ai_and_model` — không thuộc phạm vi Story 4.3, mock rỗng
// vô hại để ca không phụ thuộc hành vi của một mục khác.
vi.mock('../../src/config/project', () => ({
  listDomainLog: async () => ({ entries: [], error: null }),
}))

function emptyFields() {
  return []
}

/** Nạp lại settingsState/aiConfigState/SettingsOverlay CÙNG một lượt — state là module-level
 * singleton, cùng lý do `freshState` của `aiConfigState.test.ts`. */
async function freshOverlay() {
  vi.resetModules()
  aiConfigGetMock.mockReset()
  aiConfigSaveFieldMock.mockReset()
  aiConfigClearOverrideMock.mockReset()
  aiConfigSaveKeyMock.mockReset()
  aiConfigDeleteKeyMock.mockReset()
  aiConfigSaveKeyMock.mockResolvedValue(null)
  aiConfigDeleteKeyMock.mockResolvedValue(null)

  const settingsState = await import('../../src/settingsState')
  const aiConfigState = await import('../../src/aiConfigState')
  const SettingsOverlay = (await import('../../src/SettingsOverlay.vue')).default
  return { settingsState, aiConfigState, SettingsOverlay }
}

/** Mở lớp phủ, chuyển sang mục `ai_and_model`, rồi CHỜ đúng một lượt `loadAiConfigSection`
 * đã dùng `keyConfigured` này định cư — `selectSettingsSection` tự bắn một lượt tải (fire-
 * and-forget), gọi lại hàm này ở đây để có một Promise CHỜ ĐƯỢC, cùng khuôn `sequence` của
 * chính module đó (lượt gọi lại không đổi kết quả, vì mock luôn trả cùng một giá trị). */
async function openToAiAndModel(
  settingsState: Awaited<ReturnType<typeof freshOverlay>>['settingsState'],
  aiConfigState: Awaited<ReturnType<typeof freshOverlay>>['aiConfigState'],
): Promise<void> {
  settingsState.openSettings()
  settingsState.selectSettingsSection('ai_and_model')
  await aiConfigState.loadAiConfigSection()
}

beforeEach(() => {
  document.body.innerHTML = ''
})

describe('SettingsOverlay.vue — hàng Khoá API, ba trạng thái đọc được (Story 4.3)', () => {
  it('I/O Matrix "No key anywhere": key_configured: false ⇒ dòng đọc đúng câu "chưa cấu hình"', async () => {
    const { settingsState, aiConfigState, SettingsOverlay } = await freshOverlay()
    aiConfigGetMock.mockResolvedValue({
      fields: emptyFields(),
      workTierAvailable: false,
      keyConfigured: false,
      error: null,
    })
    await openToAiAndModel(settingsState, aiConfigState)

    const wrapper = mount(SettingsOverlay, { attachTo: document.body })
    await wrapper.vm.$nextTick()

    const keyField = wrapper.find('.ai-key-field')
    expect(keyField.exists()).toBe(true)
    expect(keyField.text()).toContain('Chưa cấu hình — không có khoá nào trong keychain.')
    // Không được đọc như trạng thái THỨ BA — hai câu phải phân biệt được.
    expect(keyField.text()).not.toContain('Không kiểm tra được')

    wrapper.unmount()
    settingsState.resetSettings()
    aiConfigState.resetAiConfigSection()
  })

  it('I/O Matrix "Save a key": nộp một giá trị hợp lệ ⇒ dòng lật từ "chưa cấu hình" sang "đã cấu hình"', async () => {
    const { settingsState, aiConfigState, SettingsOverlay } = await freshOverlay()
    aiConfigGetMock.mockResolvedValue({
      fields: emptyFields(),
      workTierAvailable: false,
      keyConfigured: false,
      error: null,
    })
    await openToAiAndModel(settingsState, aiConfigState)

    const wrapper = mount(SettingsOverlay, { attachTo: document.body })
    await wrapper.vm.$nextTick()
    expect(wrapper.find('.ai-key-field').text()).toContain('Chưa cấu hình — không có khoá nào trong keychain.')

    // Sau lượt Lưu thành công, `loadAiConfigSection()` đọc lại — mock lượt SAU trả
    // `keyConfigured: true` để phản ánh entry vừa ghi vào keychain (hành vi Rust thật).
    aiConfigGetMock.mockResolvedValue({
      fields: emptyFields(),
      workTierAvailable: false,
      keyConfigured: true,
      error: null,
    })

    const input = wrapper.find('.ai-key-field input[type="password"]')
    expect(input.exists()).toBe(true)
    await input.setValue('sk-test-fake-key-value')
    await wrapper.find('.ai-key-field form').trigger('submit')
    await flushPromises()
    await wrapper.vm.$nextTick()

    expect(aiConfigSaveKeyMock).toHaveBeenCalledTimes(1)
    expect(aiConfigSaveKeyMock).toHaveBeenCalledWith('sk-test-fake-key-value')
    // Giá trị KHÔNG BAO GIỜ quay lại qua IPC — lời gọi thứ hai (đọc lại) không mang một tham
    // số nào chứa giá trị vừa gõ; đối chứng ở `aiConfigState.test.ts`/`config/aiconfig.ts`
    // đã canh hình dạng, ở đây canh KẾT QUẢ hiển thị.
    expect(wrapper.find('.ai-key-field').text()).toContain('Đã cấu hình — khoá đang lưu trong keychain của hệ điều hành.')
    expect(wrapper.find('.ai-key-field').text()).not.toContain('Chưa cấu hình — không có khoá nào trong keychain.')
    // Ô nhập bị vứt sau lượt Lưu thành công (`saveAiConfigKey` trong `aiConfigState.ts`) —
    // giá trị không có lý do gì để tiếp tục sống trong DOM sau khi đã vào keychain.
    expect((wrapper.find('.ai-key-field input[type="password"]').element as HTMLInputElement).value).toBe('')

    wrapper.unmount()
    settingsState.resetSettings()
    aiConfigState.resetAiConfigSection()
  })

  it('key_configured: null dựng một câu THỨ BA — không phải câu "chưa cấu hình" của `false`', async () => {
    const { settingsState, aiConfigState, SettingsOverlay } = await freshOverlay()
    aiConfigGetMock.mockResolvedValue({
      fields: emptyFields(),
      workTierAvailable: false,
      keyConfigured: null,
      error: null,
    })
    await openToAiAndModel(settingsState, aiConfigState)

    const wrapper = mount(SettingsOverlay, { attachTo: document.body })
    await wrapper.vm.$nextTick()

    const keyField = wrapper.find('.ai-key-field')
    expect(keyField.text()).toContain('Không kiểm tra được')
    expect(keyField.text()).not.toContain('Chưa cấu hình — không có khoá nào trong keychain.')
    expect(keyField.text()).not.toContain('Đã cấu hình — khoá đang lưu trong keychain của hệ điều hành.')
    // Một thăm dò trượt không nói gì về Lưu/Xoá: nút "Xoá khoá" vẫn phải hiện (không bị ẩn chỉ
    // vì probe trả None). Nút Lưu ở đây `disabled` vì một lý do KHÁC, không liên quan gì tới
    // probe — ô nhập đang rỗng (khuôn "Enter vẫn đi qua validate lại" áp dụng khi có giá trị).
    expect(keyField.find('.ai-field-clear').exists()).toBe(true)
    expect(keyField.find('.ai-field-save').attributes('disabled')).toBeDefined()

    wrapper.unmount()
    settingsState.resetSettings()
    aiConfigState.resetAiConfigSection()
  })

  it('key_configured: true ⇒ nút "Xoá khoá" hiện; key_configured: false ⇒ nút đó KHÔNG hiện', async () => {
    const { settingsState, aiConfigState, SettingsOverlay } = await freshOverlay()
    aiConfigGetMock.mockResolvedValue({
      fields: emptyFields(),
      workTierAvailable: false,
      keyConfigured: true,
      error: null,
    })
    await openToAiAndModel(settingsState, aiConfigState)

    const wrapperConfigured = mount(SettingsOverlay, { attachTo: document.body })
    await wrapperConfigured.vm.$nextTick()
    expect(wrapperConfigured.find('.ai-key-field .ai-field-clear').exists()).toBe(true)
    wrapperConfigured.unmount()

    aiConfigGetMock.mockResolvedValue({
      fields: emptyFields(),
      workTierAvailable: false,
      keyConfigured: false,
      error: null,
    })
    await aiConfigState.loadAiConfigSection()

    const wrapperNotConfigured = mount(SettingsOverlay, { attachTo: document.body })
    await wrapperNotConfigured.vm.$nextTick()
    expect(wrapperNotConfigured.find('.ai-key-field .ai-field-clear').exists()).toBe(false)

    wrapperNotConfigured.unmount()
    settingsState.resetSettings()
    aiConfigState.resetAiConfigSection()
  })
})
