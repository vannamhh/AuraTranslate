/**
 * `src/i18n/index.ts::hasMessageKey` — Story 5.12 (lượt rà 2026-08-30, Bản vá 7).
 *
 * Đóng mắt xích `vi.json → TypeScript` cho bộ ba `LifecycleStatus` (Rust khai → `vi.json`
 * qua `label_key()`, canh bởi `lifecycle_contract.rs::every_lifecycle_status_label_key_exists_in_vi_json`
 * → phía dùng nó). Trước hàm này, `ReadingMode.vue::frontierStatusLabel` tự chép tay bốn
 * giá trị vào một `Set` — một bản chép THỨ BA không cổng nào nối. `hasMessageKey` đọc THẲNG
 * `vi.json`, nên không còn bản chép nào để mà lệch.
 */
import { describe, expect, it } from 'vitest'
import { hasMessageKey } from '../../src/i18n'

describe('src/i18n/index.ts::hasMessageKey', () => {
  it('cả bốn khoá `lifecycle.*` thật (khớp `LifecycleStatus::ALL` phía Rust) đều có mặt', () => {
    for (const status of ['not_started', 'in_progress', 'paused', 'done']) {
      expect(hasMessageKey(`lifecycle.${status}`)).toBe(true)
    }
  })

  it('một giá trị NGOÀI bốn giá trị (dữ liệu lạ trên đĩa) trả `false`, không ném', () => {
    expect(hasMessageKey('lifecycle.finished')).toBe(false)
    expect(hasMessageKey('lifecycle.')).toBe(false)
    expect(hasMessageKey('khong_ton_tai')).toBe(false)
  })
})
