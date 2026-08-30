/**
 * `src/tokens/themeState.ts` — đường GHI đầu tiên cho khoá `theme` (Story 5.11, phím `D`).
 *
 * 🔴 **Vì sao tệp này tồn tại, và nó là một lỗ hổng đã ĐO ĐƯỢC chứ không một phép kiểm cho
 * đủ bộ.** Lượt rà 2026-08-30 đếm: `grep -rln themeState tests/ e2e/` cho **0** — không một
 * ca nào chạm module này. Bàn đo e2e của story chỉ khẳng định
 * `document.documentElement.dataset.theme` đổi sau khi bấm `D`, mà thuộc tính đó do
 * `applyTheme()` đặt ĐỒNG BỘ — nó xanh y hệt kể cả khi lượt `putConfig` mang sai `kind`, sai
 * `key`, hay biến mất hẳn. Chính doc-comment của `KEY_THEME` gọi tên rủi ro đó: *"một lỗi gõ
 * ở đây không có kiểu nào bắt được — lượt lưu chỉ im lặng biến mất"*.
 *
 * Khuôn giả lập chép từ `glossarySettings.test.ts` — cùng hình dạng cho cùng hạng bề mặt.
 */
import { beforeEach, describe, expect, it, vi } from 'vitest'

const putConfigMock = vi.fn()

vi.mock('../../src/config/bootstrap', () => ({
  KEY_THEME: 'theme',
  SCOPE_APP_CONFIG: 'app_config',
  putConfig: (...args: unknown[]) => putConfigMock(...args),
}))

async function freshThemeState() {
  vi.resetModules()
  return import('../../src/tokens/themeState')
}

beforeEach(() => {
  putConfigMock.mockReset()
  putConfigMock.mockResolvedValue(null)
  document.documentElement.removeAttribute('data-theme')
})

describe('tokens/themeState — lượt GHI theo thao tác người dùng', () => {
  it('`setTheme("dark")` lưu ĐÚNG một lần với đúng bộ ba (kind, key, value)', async () => {
    const state = await freshThemeState()
    state.setTheme('dark')

    expect(putConfigMock).toHaveBeenCalledTimes(1)
    expect(putConfigMock).toHaveBeenCalledWith('app_config', 'theme', 'dark')
    expect(state.currentTheme.value).toBe('dark')
    expect(document.documentElement.dataset.theme).toBe('dark')
  })

  it('`toggleTheme()` đảo sáng ↔ tối và lưu giá trị MỚI, không giá trị cũ', async () => {
    const state = await freshThemeState()
    state.toggleTheme()

    expect(state.currentTheme.value).toBe('dark')
    expect(putConfigMock).toHaveBeenLastCalledWith('app_config', 'theme', 'dark')

    state.toggleTheme()
    expect(state.currentTheme.value).toBe('light')
    expect(putConfigMock).toHaveBeenLastCalledWith('app_config', 'theme', 'light')
  })

  it('một giá trị KHÔNG hợp lệ không lưu gì và giữ nguyên theme đang áp', async () => {
    const state = await freshThemeState()
    // @ts-expect-error — cố ý đẩy một giá trị ngoài kiểu: dữ liệu này có thể tới từ đĩa.
    state.setTheme('mauve')

    expect(putConfigMock).not.toHaveBeenCalled()
    expect(state.currentTheme.value).toBe('light')
  })
})

describe('tokens/themeState — đường KHỞI ĐỘNG không được ghi', () => {
  /**
   * 🔴 Mệnh đề trung tâm của tệp này. `config/bootstrap.ts` §`deleteConfig` khai luật của cửa
   * `app_config`: *"đường đọc phân biệt ba trạng thái bằng SỰ CÓ MẶT CỦA KHOÁ, không bằng giá
   * trị"*. Một lượt ghi mặc định lúc khởi động biến *"chưa ai chọn"* thành một giá trị đã lưu
   * tường minh — nó XOÁ một trạng thái khỏi mô hình, im lặng và vĩnh viễn.
   */
  it('`initTheme()` áp theme mà KHÔNG ghi xuống đĩa', async () => {
    const state = await freshThemeState()
    state.initTheme('dark')

    expect(state.currentTheme.value).toBe('dark')
    expect(document.documentElement.dataset.theme).toBe('dark')
    expect(putConfigMock).not.toHaveBeenCalled()
  })

  it('`initTheme()` với một giá trị lạ rơi về mặc định, vẫn KHÔNG ghi', async () => {
    const state = await freshThemeState()
    // @ts-expect-error — cùng lý do ca trên: giá trị này tới từ `global.db`.
    state.initTheme(undefined)

    expect(state.currentTheme.value).toBe('light')
    expect(putConfigMock).not.toHaveBeenCalled()
  })
})
