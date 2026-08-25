/**
 * Dải "Thêm thuật ngữ" — **vế TEMPLATE**, Story 3.3 · FR48 (lượt rà soát 2026-08-20).
 *
 * ─────────────────────────────────────────────────────────────────────────────
 * 🔴 VÌ SAO TỆP NÀY TỒN TẠI TÁCH KHỎI `glossaryQuickAdd.test.ts`
 * ─────────────────────────────────────────────────────────────────────────────
 * `glossaryQuickAdd.test.ts` (412 dòng) kiểm rất kỹ **module state** — đua `sequence`, hai
 * chốt tái nhập, trả lại tiêu điểm và vùng chọn — nhưng nó **không mount `GlossaryQuickAdd.vue`
 * một lần nào**. Hệ quả đo được ở lượt rà soát: gỡ `:disabled` của hai nút tầng, đảo nhánh
 * `v-if` của `quickAddSaveError`, hay xoá hẳn đoạn lý do tầng Tác phẩm — **cả `cargo test`
 * lẫn `vitest` đều xanh**. Các AC giao diện của story không có một lớp bảo vệ tự động nào.
 *
 * Khuôn chép từ `statusBar.test.ts`: giả ở **biên IPC** (`config/glossary.ts`), `vi.resetModules()`
 * rồi `import` ĐỘNG cả state lẫn component trong CÙNG một lượt để hai bên chia chung một thể
 * hiện module (state của dải là singleton cấp module — nạp lệch nhau là kiểm hai state khác
 * nhau mà tưởng một).
 *
 * ⚠️ PHẠM VI dừng ở thứ `happy-dom` trả lời được: nhánh nào được render, thuộc tính nào bật.
 * Vế THỊ GIÁC (dải ĐẨY `.modeport` lên chứ không PHỦ) và vế vùng chọn trên engine thật thuộc
 * `e2e/specs/glossary-quick-add.e2e.mjs` và bàn đo tay — `happy-dom` không có bố cục.
 */
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'
import { mount } from '@vue/test-utils'
import type { IpcError } from '../../src/i18n'

/** Bản giả của `config/glossary.ts` — biên IPC, cùng khuôn `glossaryQuickAdd.test.ts`. */
const lookupMock = vi.fn()
const addMock = vi.fn()
const updateMock = vi.fn()

vi.mock('../../src/config/glossary', () => ({
  lookupGlossaryTerm: (term: string) => lookupMock(term),
  addGlossaryTerm: (...args: unknown[]) => addMock(...args),
  updateGlossaryTerm: (...args: unknown[]) => updateMock(...args),
}))

/**
 * Nạp lại state VÀ component trong cùng một lượt — xem doc-comment đầu tệp về vì sao hai
 * lượt `import` phải nằm sau CÙNG một `resetModules()`.
 */
async function freshStrip() {
  vi.resetModules()
  lookupMock.mockReset()
  addMock.mockReset()
  updateMock.mockReset()
  const state = await import('../../src/glossaryQuickAddState')
  const i18n = await import('../../src/i18n')
  const GlossaryQuickAdd = (await import('../../src/GlossaryQuickAdd.vue')).default
  return { state, i18n, GlossaryQuickAdd }
}

/** Hai nhịp microtask cho lượt tra về, rồi một nhịp render — khuôn `glossaryQuickAdd.test.ts`. */
async function settle(wrapper: { vm: { $nextTick: () => Promise<void> } }): Promise<void> {
  await Promise.resolve()
  await Promise.resolve()
  await wrapper.vm.$nextTick()
}

/** Một mục Glossary trên dây — hình dạng `snake_case` đúng như `QuickAddTerm` phía Rust. */
function entryWire(overrides: Record<string, unknown> = {}) {
  return {
    tier: 'global',
    id: 7,
    source_term: '慕容',
    translation: 'Mộ Dung',
    note: 'ghi chu',
    category: 'person',
    term_origin: 'manual',
    created_at: '2026-08-20T00:00:00.000Z',
    ...overrides,
  }
}

const SCOPE_ERROR: IpcError = {
  code: 'glossary.scope_error',
  message_key: 'err.glossary.scope_error',
  params: {},
  retryable: false,
}

const WORK_TIER_ERROR: IpcError = {
  code: 'glossary.work_tier_unavailable',
  message_key: 'err.glossary.work_tier_unavailable',
  params: {},
  retryable: false,
}

beforeEach(() => {
  document.body.innerHTML = ''
})

afterEach(() => {
  vi.restoreAllMocks()
})

describe('dải "Thêm thuật ngữ" — lượt tra TRƯỢT phải nói LÝ DO, không đứng chờ vĩnh viễn', () => {
  it('lượt tra trả một IpcError ⇒ dải hiện đúng câu lỗi đó, KHÔNG "đang kiểm tra…"', async () => {
    const { state, i18n, GlossaryQuickAdd } = await freshStrip()
    lookupMock.mockResolvedValue({ found: 'unknown', error: SCOPE_ERROR })

    const wrapper = mount(GlossaryQuickAdd)
    state.openGlossaryQuickAdd('慕容')
    await settle(wrapper)

    // Vế trung tâm: câu lỗi PHẢI ra tới màn hình. Trước lượt sửa 2026-08-20, `IpcError` này
    // nằm chết trong `lookup.value` và dải hiện "đang kiểm tra…" mãi mãi — đúng lớp
    // "rỗng im lặng" mà story viện dẫn ba lần.
    const alert = wrapper.find('.gqa-status.gqa-error')
    expect(alert.exists()).toBe(true)
    expect(alert.text()).toBe(i18n.tError(SCOPE_ERROR))
    expect(alert.attributes('role')).toBe('alert')

    // Và dòng "đang kiểm tra" KHÔNG được đứng cùng lúc — một lượt tra đã trượt thì nó
    // không còn cơ hội nào để về.
    expect(wrapper.text()).not.toContain(i18n.t('glossary.quick_add.checking'))

    wrapper.unmount()
  })

  it('lượt tra trượt NGOÀI Tauri (`error: null`) ⇒ vẫn là "đang kiểm tra", KHÔNG một câu lỗi bịa', async () => {
    const { state, i18n, GlossaryQuickAdd } = await freshStrip()
    lookupMock.mockResolvedValue({ found: 'unknown', error: null })

    const wrapper = mount(GlossaryQuickAdd)
    state.openGlossaryQuickAdd('慕容')
    await settle(wrapper)

    // `config/glossary.ts` cố ý phân biệt "chạy ngoài Tauri" với một lỗi THẬT; nhánh mới
    // không được xoá ranh giới đó bằng cách dựng một lỗi từ hư không.
    expect(wrapper.find('.gqa-status.gqa-error').exists()).toBe(false)
    expect(wrapper.text()).toContain(i18n.t('glossary.quick_add.checking'))

    wrapper.unmount()
  })

  it('lỗi lượt GHI đứng TRÊN lỗi lượt TRA — tin mới hơn thắng', async () => {
    const { state, i18n, GlossaryQuickAdd } = await freshStrip()
    lookupMock.mockResolvedValue({ found: 'none', workTierAvailable: false })
    addMock.mockResolvedValue({ value: null, error: WORK_TIER_ERROR })

    const wrapper = mount(GlossaryQuickAdd)
    state.openGlossaryQuickAdd('慕容')
    await settle(wrapper)

    state.quickAddTierChoice.value = 'work'
    await state.saveGlossaryQuickAdd()
    await wrapper.vm.$nextTick()

    // Dải ở lại MỞ kèm lỗi (AC: lỗi phải đọc được, không rỗng im lặng) — và câu hiện ra là
    // câu của lượt ghi vừa trượt.
    expect(state.quickAddIsOpen.value).toBe(true)
    expect(wrapper.find('.gqa-status.gqa-error').text()).toBe(i18n.tError(WORK_TIER_ERROR))

    wrapper.unmount()
  })
})

describe('dải "Thêm thuật ngữ" — nút Lưu chỉ bật khi ĐÃ biết THÊM hay SỬA', () => {
  it('lượt tra chưa về ⇒ nút Lưu TẮT và dải nói "đang kiểm tra…"', async () => {
    const { state, i18n, GlossaryQuickAdd } = await freshStrip()
    // Một lượt tra không bao giờ về — đúng cửa sổ mà `mode === 'unknown'` mô tả.
    lookupMock.mockReturnValue(new Promise(() => {}))

    const wrapper = mount(GlossaryQuickAdd)
    state.openGlossaryQuickAdd('慕容')
    await wrapper.vm.$nextTick()

    expect(wrapper.find('.gqa-act-primary').attributes('disabled')).toBeDefined()
    expect(wrapper.text()).toContain(i18n.t('glossary.quick_add.checking'))

    wrapper.unmount()
  })

  it('lượt tra về "none" với ô nguồn có chữ ⇒ nút Lưu BẬT, không còn dòng trạng thái nào', async () => {
    const { state, i18n, GlossaryQuickAdd } = await freshStrip()
    lookupMock.mockResolvedValue({ found: 'none', workTierAvailable: true })

    const wrapper = mount(GlossaryQuickAdd)
    state.openGlossaryQuickAdd('慕容')
    await settle(wrapper)

    expect(wrapper.find('.gqa-act-primary').attributes('disabled')).toBeUndefined()
    expect(wrapper.find('.gqa-status').exists()).toBe(false)
    expect(wrapper.text()).toContain(i18n.t('glossary.quick_add.title_add'))

    wrapper.unmount()
  })

  it('ô nguồn RỖNG (không có vùng chọn) ⇒ nút Lưu TẮT dù lượt tra đã về', async () => {
    const { state, GlossaryQuickAdd } = await freshStrip()
    lookupMock.mockResolvedValue({ found: 'none', workTierAvailable: true })

    const wrapper = mount(GlossaryQuickAdd)
    state.openGlossaryQuickAdd('')
    await settle(wrapper)

    expect(wrapper.find('.gqa-act-primary').attributes('disabled')).toBeDefined()

    wrapper.unmount()
  })
})

describe('dải "Thêm thuật ngữ" — tầng GHIM ở chế độ SỬA, tự chọn ở chế độ THÊM', () => {
  it('chế độ SỬA ⇒ CẢ HAI nút tầng bị tắt, và nút được chọn là tầng của mục tìm được', async () => {
    const { state, i18n, GlossaryQuickAdd } = await freshStrip()
    lookupMock.mockResolvedValue({
      found: 'entry',
      workTierAvailable: true,
      entry: entryWire({ tier: 'work' }),
    })

    const wrapper = mount(GlossaryQuickAdd)
    state.openGlossaryQuickAdd('慕容')
    await settle(wrapper)

    expect(wrapper.text()).toContain(i18n.t('glossary.quick_add.title_edit'))

    const tiers = wrapper.findAll('input[name="gqa-tier"]')
    expect(tiers).toHaveLength(2)
    // `id` chỉ duy nhất TRONG một `Store` (§Design Notes): tầng KHÔNG được đổi ở chế độ SỬA,
    // nếu không lượt `UPDATE` bay vào nhầm kho, im lặng và không cổng nào đỏ.
    for (const tier of tiers) {
      expect(tier.attributes('disabled')).toBeDefined()
    }
    // Và đúng tầng của mục tìm được đang được chọn — `find<HTMLInputElement>` để `.checked`
    // là một thuộc tính có kiểu, không một phép ép kiểu trần.
    const workTier = wrapper.find<HTMLInputElement>('input[name="gqa-tier"][value="work"]')
    expect(workTier.element.checked).toBe(true)

    wrapper.unmount()
  })

  it('chế độ THÊM ⇒ hai nút tầng KHÔNG bị tắt (người dùng tự chọn)', async () => {
    const { state, GlossaryQuickAdd } = await freshStrip()
    lookupMock.mockResolvedValue({ found: 'none', workTierAvailable: true })

    const wrapper = mount(GlossaryQuickAdd)
    state.openGlossaryQuickAdd('慕容')
    await settle(wrapper)

    for (const tier of wrapper.findAll('input[name="gqa-tier"]')) {
      expect(tier.attributes('disabled')).toBeUndefined()
    }

    wrapper.unmount()
  })
})

describe('dải "Thêm thuật ngữ" — tầng Tác phẩm chưa dùng được thì hiện KÈM LÝ DO', () => {
  it('work_tier_available === false ⇒ lý do hiện ra, và lựa chọn KHÔNG biến mất', async () => {
    const { state, i18n, GlossaryQuickAdd } = await freshStrip()
    lookupMock.mockResolvedValue({ found: 'none', workTierAvailable: false })

    const wrapper = mount(GlossaryQuickAdd)
    state.openGlossaryQuickAdd('慕容')
    await settle(wrapper)

    // AC: "rỗng có lý do, không rỗng im lặng" — lựa chọn tầng Tác phẩm vẫn còn trên màn hình.
    expect(wrapper.findAll('input[name="gqa-tier"]')).toHaveLength(2)
    const reason = wrapper.find('.gqa-tier-reason')
    expect(reason.exists()).toBe(true)
    expect(reason.text()).toBe(i18n.t('glossary.quick_add.tier_work_unavailable_reason'))

    wrapper.unmount()
  })

  it('lượt tra CHƯA về ⇒ KHÔNG hiện lý do — đừng nói sai khi còn chưa biết', async () => {
    const { state, GlossaryQuickAdd } = await freshStrip()
    lookupMock.mockReturnValue(new Promise(() => {}))

    const wrapper = mount(GlossaryQuickAdd)
    state.openGlossaryQuickAdd('慕容')
    await wrapper.vm.$nextTick()

    // `quickAddWorkTierAvailable` là `null` ở đây, không `false`: hiện "chưa mở Tác phẩm nào"
    // lúc này là một câu có thể SAI ngay khi lượt tra về.
    expect(wrapper.find('.gqa-tier-reason').exists()).toBe(false)

    wrapper.unmount()
  })

  it('work_tier_available === true ⇒ KHÔNG hiện lý do', async () => {
    const { state, GlossaryQuickAdd } = await freshStrip()
    lookupMock.mockResolvedValue({ found: 'none', workTierAvailable: true })

    const wrapper = mount(GlossaryQuickAdd)
    state.openGlossaryQuickAdd('慕容')
    await settle(wrapper)

    expect(wrapper.find('.gqa-tier-reason').exists()).toBe(false)

    wrapper.unmount()
  })
})

/**
 * 🔵 **THÊM 2026-08-25 — vòng rà Epic 3, lăng kính adversarial.**
 *
 * `onCategoryKeydown` khớp `opt.digit === event.key` mà **không** lọc phím bổ trợ, trong khi
 * anh em ruột của nó ở `GlossaryQueueOverlay.vue::onKeydown` đã có đúng bản vá đó kèm lý do
 * viết sẵn. Dải này **không** nuốt bàn phím — đó là một quyết định Ice ký 2026-08-20
 * (`glossaryQuickAddState.ts::openGlossaryQuickAdd`) — nên `Mod+1` vẫn bắn lệnh toàn cục
 * `mode.library`, và bản trước của hàm này *cũng* khớp nó vì `event.key` của `⌘1` vẫn là `'1'`.
 * ⇒ MỘT hợp âm làm HAI việc, và việc thứ hai (ghi đè phân loại của thuật ngữ đang gõ dở) im
 * lặng hoàn toàn.
 *
 * ⚠️ Đây là đường DOM **cục bộ**, song song với `CommandRegistry` và không đi qua `isBlocked`
 * — đúng lý do không cổng nào hiện có nhìn thấy nó, và đúng lý do phép kiểm phải mount thật
 * rồi bắn một `KeyboardEvent` có `metaKey`.
 *
 * Đối chứng gỡ chỗ nối: gỡ dòng `if (event.ctrlKey || event.metaKey || event.altKey) return`
 * khỏi `GlossaryQuickAdd.vue` ⇒ ca ① và ② phải ĐỎ.
 */
describe('dải "Thêm thuật ngữ" — hợp âm Mod+số KHÔNG được lặng lẽ đổi phân loại', () => {
  async function stripDaMo() {
    const { state, GlossaryQuickAdd } = await freshStrip()
    lookupMock.mockResolvedValue({ found: 'none', workTierAvailable: true })
    const wrapper = mount(GlossaryQuickAdd)
    state.openGlossaryQuickAdd('慕容')
    await settle(wrapper)
    // Mặc định của một lượt mở là `'person'` — mốc để mọi ca dưới đây so.
    expect(state.quickAddCategory.value).toBe('person')
    return { state, wrapper }
  }

  it('① `⌘2` (metaKey) ⇒ phân loại GIỮ NGUYÊN — lệnh toàn cục sở hữu hợp âm này', async () => {
    const { state, wrapper } = await stripDaMo()

    await wrapper.get('fieldset.gqa-group').trigger('keydown', { key: '2', metaKey: true })

    expect(state.quickAddCategory.value).toBe('person')
    wrapper.unmount()
  })

  it('② `Ctrl+3` và `Alt+4` ⇒ cùng luật, phân loại GIỮ NGUYÊN', async () => {
    const { state, wrapper } = await stripDaMo()
    const nhom = wrapper.get('fieldset.gqa-group')

    await nhom.trigger('keydown', { key: '3', ctrlKey: true })
    expect(state.quickAddCategory.value).toBe('person')

    await nhom.trigger('keydown', { key: '4', altKey: true })
    expect(state.quickAddCategory.value).toBe('person')

    wrapper.unmount()
  })

  it('③ phím số TRẦN vẫn đổi phân loại — bản vá không được lấy mất tính năng', async () => {
    const { state, wrapper } = await stripDaMo()

    await wrapper.get('fieldset.gqa-group').trigger('keydown', { key: '2' })

    expect(state.quickAddCategory.value).toBe('place')
    wrapper.unmount()
  })

  it('④ `Shift+1` ⇒ không khớp gì cả, và đó là CỐ Ý (bố cục US cho `event.key === "!"`)', async () => {
    const { state, wrapper } = await stripDaMo()

    // `Shift` KHÔNG nằm trong danh sách lọc; nó không cần nằm đó, vì bàn phím đã đổi chính
    // `event.key`. Ca này khoá lời giải thích đó lại — nếu ai đó "dọn cho nhất quán" bằng
    // cách thêm `shiftKey` vào vệ, ca ④ vẫn xanh nhưng lý do đã mất, nên nó khẳng định thêm
    // rằng một `Shift+<digit>` TRẦN (bố cục giữ nguyên digit) VẪN đổi được.
    await wrapper.get('fieldset.gqa-group').trigger('keydown', { key: '!', shiftKey: true })
    expect(state.quickAddCategory.value).toBe('person')

    await wrapper.get('fieldset.gqa-group').trigger('keydown', { key: '3', shiftKey: true })
    expect(state.quickAddCategory.value).toBe('domain_term')

    wrapper.unmount()
  })
})
