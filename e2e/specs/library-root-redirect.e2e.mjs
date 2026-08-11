/**
 * Bàn đo Story 1.22 — thư mục gốc Library đi vào thư mục TẠM, không vào Documents thật.
 *
 * ═════════════════════════════════════════════════════════════════════════════════
 * VÌ SAO SPEC NÀY TỒN TẠI — nó là phép TỰ KIỂM DƯƠNG TÍNH mà móc Library còn thiếu
 * ═════════════════════════════════════════════════════════════════════════════════
 * Móc `$APPDATA` (AC2) có một phép tự kiểm dương tính ở `onComplete`: `global.db` **phải
 * nằm** trong thư mục tạm. Móc thư mục gốc Library không có đối ứng như vậy, vì chưa bàn
 * đo nào tạo Tác phẩm — nên thư mục tạm rỗng dù móc chạy đúng hay sai, và hàng rào chiều
 * âm ở `wdio.conf.mjs` *(thư mục thật phải y nguyên)* đúng một cách **tầm thường**.
 *
 * 🔴 Một cơ chế chỉ có hàng rào âm là một cơ chế chưa ai chứng minh là có chạy. Spec này
 * đóng vế đó bằng cách **thật sự tạo một Tác phẩm** rồi hỏi nó rơi vào đâu.
 *
 * ⚠️ Nó gọi thẳng IPC chứ **không** đi qua giao diện, và đó là một lựa chọn có lý do: nút
 * tạo Tác phẩm sống ở chế độ `library` nhưng đường đi trọn vẹn của nó cần một form và một
 * lượt điều hướng — tức spec sẽ đo **giao diện nhập** thay vì đo **đường ghi**. Câu hỏi ở
 * đây là *"`.atproj` rơi vào thư mục nào"*, và IPC là bề mặt hẹp nhất trả lời đúng câu đó.
 * Ngày có fixture giao diện thật, spec này vẫn giữ nguyên giá trị: nó canh **đường ghi**.
 *
 * Bộ chạy sống trong Node, nên khẳng định đọc đĩa bằng `node:fs` — không qua webview.
 */

import { existsSync, readdirSync } from 'node:fs'
import { join } from 'node:path'
import { homedir } from 'node:os'

/** Khớp `DOCUMENTS_SUBFOLDER` ở `src-tauri/src/commands/project.rs`. */
const DOCUMENTS_SUBFOLDER = 'AuraTranslate'

/**
 * Tên Tác phẩm mang dấu của lượt chạy — nếu nó rơi nhầm vào Documents thật thì nó
 * **đọc được ra là của bộ e2e**, thay vì trông như một Tác phẩm người dùng tự tạo.
 */
const WORK_NAME = `e2e-library-root-probe-${Date.now()}`

describe('Story 1.22 · AC2 — đường ghi Tác phẩm đi vào thư mục gốc Library TẠM', () => {
  it('tạo một Tác phẩm thì `.atproj` nằm trong thư mục tạm, không trong Documents thật', async () => {
    const libraryRoot = process.env.AURATRANSLATE_E2E_LIBRARY_ROOT
    expect(libraryRoot).toBeDefined()

    // Gọi command thật của sản phẩm. Tên tham số khớp `commands::project::wire`
    // (`name` · `sourceLang` · `genre` · `text`) — cùng bộ mà `src/config/project.ts` gửi.
    const result = await browser.execute(
      async (name) => {
        const internals = window.__TAURI_INTERNALS__
        if (internals === undefined) return { ok: false, detail: 'không có cầu IPC' }
        try {
          await internals.invoke('create_work_from_text', {
            name,
            sourceLang: 'zh',
            genre: 'general',
            text: 'Một câu nguồn để bộ nhập có việc mà làm.',
          })
          return { ok: true, detail: '' }
        } catch (err) {
          return { ok: false, detail: String(err && err.code ? err.code : err) }
        }
      },
      WORK_NAME,
    )

    expect(result.ok).toBe(true)

    // ── Vế DƯƠNG: `.atproj` phải nằm trong thư mục tạm ───────────────────────────
    const inTemp = readdirSync(libraryRoot)
    expect(inTemp).toContain(`${WORK_NAME}.atproj`)

    // ── Vế ÂM: Documents THẬT không được mọc thêm gì ──────────────────────────────
    // Hỏi cả hai chiều, cùng lý do mà `shortcuts-focus` hỏi cả hai: một khẳng định chỉ
    // nói "có trong thư mục tạm" vẫn xanh nếu sản phẩm ghi vào CẢ HAI chỗ.
    const realRoot = join(homedir(), 'Documents', DOCUMENTS_SUBFOLDER)
    if (existsSync(realRoot)) {
      expect(readdirSync(realRoot)).not.toContain(`${WORK_NAME}.atproj`)
    }
  })
})
