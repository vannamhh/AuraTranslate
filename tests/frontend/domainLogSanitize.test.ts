/**
 * `config/project.ts::listDomainLog` — hạ một `outcome` LẠ về `null` cho RIÊNG bản ghi đó,
 * không loại bỏ CẢ MẢNG.
 *
 * ─────────────────────────────────────────────────────────────────────────────
 * 🔴 VÌ SAO TỆP NÀY TỒN TẠI — vòng rà đối kháng 3 (lớp 3, mục R3)
 * ─────────────────────────────────────────────────────────────────────────────
 * Trước sửa này, `listDomainLog` gọi `entries.every(isDomainLogEntryWire)` — MỘT bản ghi
 * mang một biến thể `outcome` mà bản TS chưa biết (ví dụ Rust thêm biến thể thứ chín mà bản
 * build frontend chưa cập nhật `DOMAIN_LOG_OUTCOMES`) làm `every()` trả `false`, và
 * `listDomainLog` vứt TOÀN BỘ mảng (`entries: null`) — cả màn Quyền riêng tư trống trơn vì
 * đúng MỘT trường của đúng MỘT bản ghi. Khuôn `bootstrap.test.ts`/`glossaryMarksRefresh.test.ts`:
 * mock đúng biên IPC (`@tauri-apps/api/core`), không mock trọn adapter — bộ phân giải THẬT
 * phải chạy qua test này.
 */
import { beforeEach, describe, expect, it, vi } from 'vitest'

const mockInvoke = vi.fn()
vi.mock('@tauri-apps/api/core', () => ({ invoke: (...args: unknown[]) => mockInvoke(...args) }))

function rawEntry(overrides: Record<string, unknown> = {}): Record<string, unknown> {
  return {
    at_epoch_ms: 1,
    domain: 'example.test',
    kind: 'image',
    allowed: true,
    tier: 'tier2',
    outcome: 'fetched',
    ...overrides,
  }
}

beforeEach(() => {
  vi.resetModules()
  mockInvoke.mockReset()
})

describe('listDomainLog — một outcome LẠ chỉ hạ RIÊNG bản ghi đó, không vứt cả mảng', () => {
  it('một biến thể outcome CHƯA BIẾT trên MỘT bản ghi: hạ về null, các hàng khác giữ nguyên', async () => {
    const { listDomainLog } = await import('../../src/config/project')
    mockInvoke.mockResolvedValue([
      rawEntry({ domain: 'a.example', outcome: 'fetched' }),
      rawEntry({ domain: 'b.example', outcome: 'mot_bien_the_tuong_lai_chua_biet' }),
      rawEntry({ domain: 'c.example', outcome: null }),
    ])

    const result = await listDomainLog()
    expect(result.error).toBeNull()
    expect(result.entries).not.toBeNull()
    expect(result.entries).toHaveLength(3)
    expect(result.entries?.[0]).toMatchObject({ domain: 'a.example', outcome: 'fetched' })
    expect(result.entries?.[1]).toMatchObject({ domain: 'b.example', outcome: null })
    expect(result.entries?.[2]).toMatchObject({ domain: 'c.example', outcome: null })
  })

  it('mọi outcome hợp lệ (tám biến thể + null): không hàng nào bị hạ oan', async () => {
    const { listDomainLog } = await import('../../src/config/project')
    const outcomes = [
      'fetched',
      'redirected',
      'http_status',
      'timeout',
      'connect_failed',
      'too_large',
      'mime_rejected',
      'other',
      null,
    ]
    mockInvoke.mockResolvedValue(outcomes.map((outcome) => rawEntry({ outcome })))

    const result = await listDomainLog()
    expect(result.entries).toHaveLength(outcomes.length)
    expect(result.entries?.map((e) => e.outcome)).toEqual(outcomes)
  })

  it('hình dạng CỐT LÕI hỏng (không phải chỉ outcome) vẫn từ chối TOÀN BỘ mảng', async () => {
    const { listDomainLog } = await import('../../src/config/project')
    mockInvoke.mockResolvedValue([
      rawEntry({ domain: 'a.example' }),
      rawEntry({ domain: 'b.example', tier: 'mot_tang_khong_ton_tai' }),
    ])

    const result = await listDomainLog()
    expect(result.entries).toBeNull()
    expect(result.error).not.toBeNull()
  })
})
