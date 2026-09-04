/**
 * `src/**` — không một dòng SẢN PHẨM nào gọi thẳng `create_work_from_text`/
 * `create_work_from_file` (bỏ qua màn xem trước bảng mã). Story 6.3, vòng rà đối kháng 2,
 * mục 5.
 *
 * ─────────────────────────────────────────────────────────────────────────────
 * 🔴 VÌ SAO CA NÀY TỒN TẠI
 * ─────────────────────────────────────────────────────────────────────────────
 * §Always spec 6.3: *"không byte nào xuống đĩa trước khi người dùng xác nhận"*. Trước ca
 * này, mệnh đề đó chỉ đúng vì `createWorkFromText`/`createWorkFromFile` (adapter TS) đơn
 * giản KHÔNG CÓ chỗ gọi nào — một QUY ƯỚC quan sát được, không một CỔNG. Adapter đó đã bị
 * xoá (cùng lượt vá này), thu hẹp bề mặt xuống còn "gọi thẳng `invoke('create_work_from_text',
 * …)` bằng tay" — ca này khoá luôn đường đó lại: `grep` toàn `src/**` cho CHUỖI TÊN COMMAND,
 * không riêng tên hàm adapter.
 *
 * ⚠️ Vỏ Rust (`wire::create_work_from_text`/`_from_file`) và tên command đó VẪN tồn tại có
 * chủ ý — `e2e/specs/**` gọi thẳng qua `internals.invoke(...)` để dựng fixture nhanh, cố ý
 * đi ĐƯỜNG IPC trực tiếp bỏ qua UI. Ca này chỉ canh `src/**` (mã SẢN PHẨM), không canh
 * `e2e/**` (hạ tầng test, được PHÉP đi tắt).
 */
import { describe, expect, it } from 'vitest'

async function walk(dir: string, out: string[]): Promise<void> {
  const { readdir } = await import('node:fs/promises')
  const { join } = await import('node:path')
  const entries = await readdir(dir, { withFileTypes: true })
  for (const entry of entries) {
    const full = join(dir, entry.name)
    if (entry.isDirectory()) {
      await walk(full, out)
    } else if (entry.isFile() && (entry.name.endsWith('.ts') || entry.name.endsWith('.vue'))) {
      out.push(full)
    }
  }
}

describe('src/** — không dòng sản phẩm nào gọi thẳng create_work_from_text/_file', () => {
  it('0 chỗ gọi bằng chuỗi tên command, ngoại trừ chú thích lịch sử đã biết', async () => {
    const { readFile } = await import('node:fs/promises')
    const { resolve } = await import('node:path')

    const files: string[] = []
    await walk(resolve(process.cwd(), 'src'), files)
    expect(files.length).toBeGreaterThan(50) // sàn quần thể -- cùng bậc với các test khác

    const offenders: string[] = []
    for (const file of files) {
      const text = await readFile(file, 'utf8')
      const lines = text.split('\n')
      lines.forEach((line, index) => {
        const trimmed = line.trim()
        // Chỉ bắt LỜI GỌI thật (`invoke('create_work_from_text'` hoặc tương đương) -- không
        // bắt oan một dòng comment nhắc TÊN nó để giải thích lịch sử (đã biết, có chủ ý:
        // `libraryImport.ts`, và `libraryImportResetsSegmentHistory.test.ts` không nằm
        // trong `src/`).
        if (trimmed.startsWith('//') || trimmed.startsWith('*') || trimmed.startsWith('/*')) return
        if (line.includes("'create_work_from_text'") || line.includes("'create_work_from_file'")) {
          offenders.push(`${file}:${index + 1}  ${trimmed}`)
        }
      })
    }

    expect(
      offenders,
      `một dòng SẢN PHẨM trong src/** gọi thẳng create_work_from_text/_file, bỏ qua màn xem ` +
        `trước bảng mã:\n${offenders.join('\n')}`,
    ).toEqual([])
  })
})
