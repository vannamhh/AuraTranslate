/**
 * Bàn đo Story 5.3 — "Quét lại thư mục" (FR99) trong **WKWebView thật**.
 *
 * ═════════════════════════════════════════════════════════════════════════════════
 * VÌ SAO SPEC NÀY TỒN TẠI — §Manual checks của story chưa ai chạy, và ba vỏ IPC chưa ai chạm
 * ═════════════════════════════════════════════════════════════════════════════════
 * `library_index_contract.rs`/`library_commands_contract.rs` gọi hàm THUẦN; `tests/frontend/
 * libraryRescan.test.ts` chạy trên `happy-dom` với `invoke` GIẢ. Ba vỏ
 * `#[tauri::command(async)]` — `library_rescan` · `library_forget_orphan` (và
 * `library_choose_root`, xem §Giới hạn) — cho tới trước spec này chỉ bị **so chuỗi chữ ký**
 * ở `config_invariants.rs`. Nghĩa là: đăng ký thiếu ở `generate_handler!`, tên tham số lệch
 * `camelCase`, hay một `dispatch('library.*')` chưa nối port đều đi qua **toàn bộ** cổng
 * hiện có mà không ai đỏ. Spec này đi trọn đường: nút thật → `dispatch` thật → registry thật
 * → `invoke` thật → `Indexer` thật → DOM thật.
 *
 * ═════════════════════════════════════════════════════════════════════════════════
 * §Giới hạn — ghi ra thay vì để người sau tưởng đã phủ
 * ═════════════════════════════════════════════════════════════════════════════════
 * 1. **`library.choose_root` KHÔNG đo được ở đây.** Nó mở `blocking_pick_folder()` — một
 *    hộp thoại NATIVE của hệ điều hành, nằm ngoài webview, nên WebDriver không chạm tới
 *    được. Bấm nút đó trong một lượt tự động sẽ TREO cửa sổ chờ người thật. ⇒ Vế "đổi thư
 *    mục gốc" ở lại §Manual checks, và spec này **không bấm** nút đó.
 * 2. **AC6 ("quét lại không chặn thao tác") không đo ở đây.** Nó cần một thư viện đủ lớn để
 *    lượt quét kéo dài đo được; dựng một thư viện như thế trong bàn đo là dựng một phép đo
 *    chập chờn theo tốc độ đĩa của người chạy. Vế cấu trúc (`(async)`) đã có cổng
 *    `config_invariants.rs`; vế cảm nhận ở lại §Manual checks.
 * 3. Spec này **ghi vào thư mục gốc Library TẠM** do `wdio.conf.mjs` cấp qua
 *    `AURATRANSLATE_E2E_LIBRARY_ROOT`. Nó `expect` biến đó có mặt TRƯỚC khi chạm đĩa —
 *    thiếu nó thì dừng ngay, không đoán một đường dẫn.
 */

import { cpSync, existsSync, mkdirSync, renameSync, rmSync } from 'node:fs'
import { join, dirname } from 'node:path'

import { realClick } from '../support/pointer.mjs'

/** Tên mang dấu lượt chạy — đọc ra là của bộ e2e nếu nó rơi nhầm chỗ. */
const WORK_NAME = `e2e-rescan-probe-${Date.now()}`

/** Nút "Quét lại" là nút THỨ HAI trong `.root-actions` (nút đầu là "Đổi thư mục gốc"). */
const RESCAN_BTN = '.root-block .root-actions .btn:nth-of-type(2)'
/** Trong `.orphan-actions`: `‹` · (span) · `›` · "Gỡ khỏi chỉ mục" — nút thứ BA mang class. */
const FORGET_BTN = '.orphan-actions .btn:nth-of-type(3)'

const ROOT_VALUE = '.root-block .root-value'
const ROOT_MISSING = '.root-block .root-missing'
const ORPHAN_NAME = '.orphan-row .orphan-name'
const ORPHAN_PATH = '.orphan-row .orphan-path'
/** Node cảnh báo xung đột `work_id` — phán quyết Ice #3 (2026-08-27), `LibraryMode.vue`. */
const CONFLICT_WARNING = '.root-block .conflict-warning'

/** Bấm "Quét lại" rồi chờ lượt quét xong (nút hết `disabled`). */
async function rescanAndWait() {
  await realClick(await $(RESCAN_BTN))
  // `rescanBusy` bật ĐỒNG BỘ ngay trong `dispatch` (trước `await`), và `busy = false` được
  // đặt trong CÙNG một tick với `applyReport` — nên Vue gộp chúng vào một lượt render: lúc
  // nút hiện lại là bấm được thì báo cáo ĐÃ vào DOM. Không cần một `pause()` đoán mò.
  await browser.waitUntil(async () => (await $(RESCAN_BTN)).isEnabled(), {
    timeout: 20_000,
    timeoutMsg: 'nút Quét lại không trở lại trạng thái bấm được sau 20 giây',
  })
}

/** Câu ba-con-số của `.root-block > .status` (node đầu tiên mang class `status`). */
async function resultLine() {
  const nodes = await $$('.root-block .status')
  return (await nodes[0].getText()).trim()
}

/**
 * Ba con số của dòng kết quả, tách ra để khẳng định bằng **DELTA**.
 *
 * ═════════════════════════════════════════════════════════════════════════════════
 * 🔴 VÌ SAO DELTA CHỨ KHÔNG PHẢI CON SỐ TUYỆT ĐỐI — MỘT LƯỢT ĐỎ ĐÃ ĐO
 * ═════════════════════════════════════════════════════════════════════════════════
 * ⚠️ **Đo 2026-08-28, lượt chạy TRỌN BỘ đầu tiên: 5 trên 7 ca của spec này ĐỎ.** Bản trước
 * viết `expect(resultLine()).toContain('Đã lập chỉ mục 1')`; nó nhận
 * *"Đã lập chỉ mục 22 · trùng work_id 0 · bỏ qua 0"*.
 *
 * Nguyên nhân KHÔNG phải hồi quy sản phẩm — chạy RIÊNG spec này thì nó xanh. `wdio.conf.mjs`
 * §onPrepare cấp **MỘT** thư mục gốc Library tạm cho **cả lượt chạy** (`maxInstances: 1`, một
 * quyết định có chủ ý và có lý do viết sẵn ở đó), nên mọi spec tạo Tác phẩm đều để lại
 * `.atproj` trong đúng thư mục này. Một con số tuyệt đối ở đây là một khẳng định về **toàn bộ
 * tài nguyên dùng chung**, tức về việc những spec khác đã làm gì — thứ spec này không biết và
 * không được phép biết.
 *
 * ⇒ Mọi khẳng định về ba con số đổi sang **hiệu so với ngay trước thao tác**. Delta đúng dù
 * mô hình cách ly có đổi hay không, nên bản vá này không nợ gì một lượt đổi hạ tầng về sau.
 *
 * Trả `null` khi chưa quét lần nào (dòng còn rỗng) — chỗ gọi phải xử lý, không được đoán 0.
 */
async function resultCounts() {
  const line = await resultLine()
  const matched = /Đã lập chỉ mục (\d+) · trùng work_id (\d+) · bỏ qua (\d+)/.exec(line)
  if (matched === null) return null
  return { indexed: Number(matched[1]), conflicts: Number(matched[2]), skipped: Number(matched[3]) }
}

/** Câu trạng thái mục mồ côi (node `.status` THỨ HAI của `.root-block`). */
async function orphanLine() {
  const nodes = await $$('.root-block .status')
  return (await nodes[1].getText()).trim()
}

describe('Story 5.3 · FR99 — quét lại thư mục trong cửa sổ thật', () => {
  const libraryRoot = process.env.AURATRANSLATE_E2E_LIBRARY_ROOT
  /** Nơi cất `.atproj` khi mô phỏng "người dùng chuyển nó ra ngoài thư viện". */
  let parkingLot
  let workDir

  before(async () => {
    expect(libraryRoot).toBeDefined()
    parkingLot = join(dirname(libraryRoot), 'e2e-parking-lot')
    mkdirSync(parkingLot, { recursive: true })
    workDir = join(libraryRoot, `${WORK_NAME}.atproj`)
  })

  it('trước lượt quét đầu, màn hình nói CHƯA BIẾT chứ không nói "không có"', async () => {
    // 🔴 Đây là vế `…HasLoaded` của `AGENTS.md::Known pitfalls`: một danh sách rỗng TRƯỚC
    // lượt quét đầu tiên là "chưa biết", không phải "không có mục mồ côi nào".
    expect(await orphanLine()).toContain('Bấm Quét lại')
    expect(await (await $(ROOT_VALUE)).getText()).toContain('Chưa quét lần nào')
    expect(await resultLine()).toBe('')
  })

  it('một `.atproj` mới trong thư mục gốc xuất hiện sau đúng một lượt Quét lại', async () => {
    // Mốc so sánh: một lượt quét TRƯỚC khi tạo gì. Thư mục gốc dùng chung cả lượt chạy nên
    // con số nền là bao nhiêu không ai biết trước — chỉ HIỆU của nó mới thuộc về ca này.
    await rescanAndWait()
    const before = await resultCounts()
    expect(before).not.toBe(null)

    const created = await browser.execute(async (name) => {
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
    }, WORK_NAME)
    expect(created.ok).toBe(true)
    expect(existsSync(workDir)).toBe(true)

    await rescanAndWait()

    // Đường dẫn THẬT chỉ đến từ `RescanReport.root` do Rust phân giải — nếu bộ phân giải
    // ba tầng hỏng, câu này vẫn là "Chưa quét lần nào" và ca đỏ ở đây.
    expect(await (await $(ROOT_VALUE)).getText()).toBe(libraryRoot)
    expect((await resultCounts()).indexed).toBe(before.indexed + 1)
    expect(await orphanLine()).toContain('Không có mục mồ côi nào')
    expect(await (await $(ROOT_MISSING)).getText()).toBe('')
  })

  it('chuyển `.atproj` ra NGOÀI thư mục gốc ⇒ mục mồ côi, kèm đường dẫn CŨ', async () => {
    const before = await resultCounts()
    renameSync(workDir, join(parkingLot, `${WORK_NAME}.atproj`))

    await rescanAndWait()

    expect((await resultCounts()).indexed).toBe(before.indexed - 1)
    expect(await (await $(ORPHAN_NAME)).getText()).toBe(WORK_NAME)
    // AC3 nguyên văn: "nêu rõ nó trỏ tới đâu" — đường dẫn CŨ, không rỗng, không bị xoá.
    expect(await (await $(ORPHAN_PATH)).getText()).toContain(workDir)
  })

  it('mục mồ côi QUAY LẠI thì hết mồ côi, và KHÔNG sinh hàng thứ hai', async () => {
    const before = await resultCounts()
    renameSync(join(parkingLot, `${WORK_NAME}.atproj`), workDir)

    await rescanAndWait()

    expect((await resultCounts()).indexed).toBe(before.indexed + 1)
    expect(await orphanLine()).toContain('Không có mục mồ côi nào')
    expect(await (await $('.orphan-row')).isExisting()).toBe(false)
  })

  it('nút "Gỡ khỏi chỉ mục" xoá đúng mục mồ côi đang chọn', async () => {
    renameSync(workDir, join(parkingLot, `${WORK_NAME}.atproj`))
    await rescanAndWait()
    expect(await (await $(ORPHAN_NAME)).getText()).toBe(WORK_NAME)

    await realClick(await $(FORGET_BTN))

    await browser.waitUntil(async () => (await orphanLine()).includes('Không có mục mồ côi'), {
      timeout: 20_000,
      timeoutMsg: 'gỡ mục mồ côi rồi mà danh sách vẫn không nói "không có mục mồ côi nào"',
    })
    expect(await (await $('.orphan-row')).isExisting()).toBe(false)
  })

  it('thư mục gốc BIẾN MẤT nói ra lý do, không im lặng thành "đã quét, rỗng"', async () => {
    // 🔴 Đây là vế P1 của vòng rà: `indexed: 0` một mình KHÔNG phân biệt được "gốc không
    // còn ở đó" với "gốc rỗng thật". Câu `.root-missing` là chỗ duy nhất nói ra khác biệt.
    const before = await resultCounts()
    const stashed = `${libraryRoot}-stashed`
    renameSync(libraryRoot, stashed)
    try {
      await rescanAndWait()
      const missing = await (await $(ROOT_MISSING)).getText()
      expect(missing).toContain('không còn tồn tại trên đĩa')
      expect(missing).toContain(libraryRoot)
    } finally {
      // Khôi phục TRƯỚC khi `onComplete` của wdio chạy phép tự kiểm và dọn thư mục tạm.
      renameSync(stashed, libraryRoot)
    }

    // Gốc trở lại ⇒ câu lý do phải TẮT, và ba con số quay về đúng mốc trước khi cất gốc đi.
    // 🔵 SỬA 2026-08-28: bản trước khẳng định `Đã lập chỉ mục 0` kèm chú thích "gốc RỖNG
    // THẬT" — mệnh đề đó chỉ đúng khi spec này chạy MỘT MÌNH. Vế thật của ca là "`.root-missing`
    // tắt khi gốc có mặt", và nó không cần gốc phải rỗng.
    await rescanAndWait()
    expect(await (await $(ROOT_MISSING)).getText()).toBe('')
    expect((await resultCounts()).indexed).toBe(before.indexed)
  })

  /**
   * Phán quyết Ice #3 (2026-08-27) — AC4 nói "phát hiện VÀ CẢNH BÁO", và ca này là ca DUY
   * NHẤT trong cả ba tầng (Rust/frontend/e2e) đi TRỌN đường nút thật → `dispatch` →
   * `invoke` → `Indexer::rebuild` → `RescanReport.conflicts` (nay có cấu trúc) → DOM cho
   * đúng kịch bản "hai Tác phẩm cùng `work_id`".
   *
   * `create_work_from_text` luôn sinh một UUID MỚI mỗi lần gọi (không có đường IPC nào tạo
   * hai Tác phẩm cùng `work_id`), nên fixture phải dựng THẲNG trên đĩa — đúng cách
   * `library_index_contract.rs::a_duplicate_work_id_keeps_the_first_entry_and_reports_the_conflict_with_both_paths`
   * đã làm ở tầng Rust: COPY một `.atproj` đã có `meta.json` hợp lệ (từ `parkingLot`, còn
   * nguyên sau ca "Gỡ khỏi chỉ mục" ở trên) vào HAI thư mục MỚI cùng gốc, giữ NGUYÊN
   * `work_id` bên trong — chỉ tên thư mục khác nhau.
   */
  it('hai `.atproj` cùng work_id ⇒ node cảnh báo nêu đúng CẢ HAI đường dẫn', async () => {
    const parkedSource = join(parkingLot, `${WORK_NAME}.atproj`)
    expect(existsSync(parkedSource)).toBe(true)

    // Tên đặt để thứ tự SẮP XẾP (Indexer quét theo tên đã sắp) tất định: "-A" trước "-B".
    const keptDir = join(libraryRoot, `${WORK_NAME}-Conflict-A.atproj`)
    const duplicateDir = join(libraryRoot, `${WORK_NAME}-Conflict-B.atproj`)
    const before = await resultCounts()
    cpSync(parkedSource, keptDir, { recursive: true })
    cpSync(parkedSource, duplicateDir, { recursive: true })

    try {
      await rescanAndWait()

      // Hai thư mục vào, nhưng CHỈ MỘT được lập chỉ mục — bản sao thứ hai thành xung đột.
      const after = await resultCounts()
      expect(after.indexed).toBe(before.indexed + 1)
      expect(after.conflicts).toBe(1)

      const warning = await (await $(CONFLICT_WARNING)).getText()
      expect(warning).toContain(keptDir)
      expect(warning).toContain(duplicateDir)
    } finally {
      // Dọn TRƯỚC khi `onComplete` của wdio chạy phép tự kiểm chữ ký thư mục Library thật.
      rmSync(keptDir, { recursive: true, force: true })
      rmSync(duplicateDir, { recursive: true, force: true })
    }
  })

  after(() => {
    if (parkingLot !== undefined && existsSync(parkingLot)) {
      rmSync(parkingLot, { recursive: true, force: true })
    }
  })
})
