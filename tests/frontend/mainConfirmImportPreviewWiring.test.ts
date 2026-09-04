/**
 * `src/main.ts` — chỗ gọi sản phẩm DUY NHẤT nối `confirmImportPreview()` (state) với
 * `finishImportSubmission()` (đóng vòng nộp form). Story 6.3, vòng rà đối kháng 2, mục 3.
 *
 * ─────────────────────────────────────────────────────────────────────────────
 * 🔴 VÌ SAO ĐỌC VĂN BẢN, KHÔNG `import('../../src/main')`
 * ─────────────────────────────────────────────────────────────────────────────
 * `main.ts` gọi `void boot()` KHÔNG ĐIỀU KIỆN ở cấp module — `boot()` gọi
 * `createApp(App).mount('#app')`, đăng ký command thật, gọi IPC thật. `import()` tệp này
 * trong một bài test sẽ CHẠY TRỌN bootstrap đó, đúng thứ mọi test khác trong `tests/frontend/**`
 * tránh. Đọc mã nguồn dạng VĂN BẢN — cùng khuôn `ipc_contract.rs` đọc `lib.rs`,
 * `pinned_contract.rs` đọc `config/pinned.ts` — là cách DUY NHẤT kiểm dây này mà không phải
 * viết lại kiến trúc khởi động của `main.ts`.
 *
 * ─────────────────────────────────────────────────────────────────────────────
 * 🔴 ĐO (2026-09-04, phản biện Ice) — xoá dòng `finishImportSubmission(result.created,
 * result.error)` ở `src/main.ts:429` ⇒ `vitest` 792 XANH, 0 ĐỎ.
 * ─────────────────────────────────────────────────────────────────────────────
 * `libraryImportResetsSegmentHistory.test.ts` và `libraryImportRetryAfterFailedConfirm.test.ts`
 * tự CHÉP LẠI dây đó (`const result = await confirmImportPreview(); if (...)
 * finishImportSubmission(...)`) thay vì đi QUA `main.ts`, nên xoá dòng thật ở sản phẩm không
 * làm chúng đỏ — chúng canh một BẢN SAO, không canh CHÍNH DÂY. Bất biến *"đọc nội dung Tác
 * phẩm A dưới nhãn Tác phẩm B"* (reset panel sau khi đổi Tác phẩm) chết mà không cổng nào
 * thấy.
 */
import { describe, expect, it } from 'vitest'

describe('main.ts — dây `import.preview.confirm` nối confirmImportPreview() với finishImportSubmission()', () => {
  it('CommandDeps.confirmImportPreview thật sự gọi finishImportSubmission SAU khi confirmImportPreview() (state) trả về', async () => {
    const { readFileSync } = await import('node:fs')
    const { resolve } = await import('node:path')
    const source = readFileSync(resolve(process.cwd(), 'src/main.ts'), 'utf8')

    // Cắt lấy ĐÚNG khối `confirmImportPreview: () => { … },` trong object truyền vào
    // `installCommands({...})` — không quét cả tệp: một chuỗi con `finishImportSubmission`
    // XUẤT HIỆN Ở ĐÂU ĐÓ khác trong tệp (ví dụ import statement) không chứng minh nó được
    // GỌI trong đúng handler này.
    const anchor = 'confirmImportPreview: () => {'
    const anchorIndex = source.indexOf(anchor)
    expect(anchorIndex, '`confirmImportPreview: () => {` phải có mặt trong `installCommands({...})`').toBeGreaterThan(-1)

    // Khối kết thúc ở dấu `}` đóng arrow function CỘNG `,` — tìm chỗ đóng bằng cách đếm
    // ngoặc nhọn từ điểm mở `{` liền sau `=>`.
    const bodyStart = source.indexOf('{', anchorIndex + anchor.length - 1)
    let depth = 0
    let bodyEnd = bodyStart
    for (let i = bodyStart; i < source.length; i += 1) {
      if (source[i] === '{') depth += 1
      else if (source[i] === '}') {
        depth -= 1
        if (depth === 0) {
          bodyEnd = i
          break
        }
      }
    }
    const handlerBody = source.slice(bodyStart, bodyEnd + 1)

    expect(
      handlerBody.includes('await confirmImportPreview()'),
      'handler phai goi confirmImportPreview() (tu importPreviewState.ts) va CHO ket qua',
    ).toBe(true)
    expect(
      handlerBody.includes('finishImportSubmission(result.created, result.error)'),
      'handler phai dong vong nop form bang finishImportSubmission(result.created, result.error) -- ' +
        'day la CHINH DAY ma ca test nay ton tai de canh. Xoa dong nay o main.ts phai lam ca nay DO.',
    ).toBe(true)
  })
})
