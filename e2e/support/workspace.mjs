/**
 * Fixture — đưa app vào chế độ `workspace` với một Tác phẩm đang mở.
 *
 * ═════════════════════════════════════════════════════════════════════════════════
 * VÌ SAO CẦN FIXTURE NÀY
 * ═════════════════════════════════════════════════════════════════════════════════
 * App khởi động ở chế độ `library` (`modes/modeState.ts`), và **phần lớn bề mặt của Epic
 * 1 sống trong `workspace`**: ba panel *(🔵 bốn → ba ở Story 2.5b)*, lưới đối chiếu, Panel Lookup, và nút mở màn hình
 * Attribution. Mọi hàng bàn đo chạm tới chúng đều bắt đầu bằng đúng hai bước dưới đây.
 *
 * 🔴 Fixture này chỉ dựng được SAU khi hai bề mặt dữ liệu thật đã bị chuyển hướng —
 * `$APPDATA` **và** thư mục gốc Library. Trước đó, một fixture tạo Tác phẩm sẽ ghi vào
 * `~/Documents/AuraTranslate/` của người chạy mỗi lượt. Thứ tự đó không phải tình cờ: bề
 * mặt thứ hai được tìm ra **trong lúc chuẩn bị chính fixture này**.
 *
 * ═════════════════════════════════════════════════════════════════════════════════
 * HAI LỰA CHỌN CÓ CHỦ Ý
 * ═════════════════════════════════════════════════════════════════════════════════
 * ① **Tạo Tác phẩm bằng IPC, không bằng form Library.** Form đó không có một mối nối
 *    `data-` nào (`LibraryMode.vue` — `v-model` trên `<input>` trần), nên một fixture đi
 *    qua nó phải chọn phần tử theo cấu trúc DOM và sẽ vỡ ở lượt đổi bố cục đầu tiên. Quan
 *    trọng hơn: mọi hàng dùng fixture này đo **một thứ khác** — Panel Lookup, tiêu điểm,
 *    Attribution — nên một fixture giòn sẽ làm chúng đỏ vì lý do không liên quan.
 *    ⚠️ Đánh đổi phải nói ra: fixture này **không** đo đường nhập của người dùng thật.
 *    Ngày có một hàng bàn đo cho chính form Library, nó phải đi qua giao diện, không qua
 *    đây.
 *
 * ② **Đổi chế độ bằng BÀN PHÍM (`Mod+2`), không bằng cách bấm tab.** Ba tab chế độ ở
 *    `App.vue` không mang mối nối `data-` nào, và thêm một mối nối vào mã sản phẩm **chỉ
 *    để test chọn được** là mở một tiền lệ mà kho này cố ý chưa mở: `data-shortcuts-open`
 *    và `data-attribution-open` tồn tại vì **sản phẩm** cần chúng (đường lui của tiêu
 *    điểm), không vì bàn đo. Bàn phím tránh hẳn câu hỏi đó và đi đúng đường NFR17 hứa.
 */

/** Khớp `DOCUMENTS_SUBFOLDER` ở `src-tauri/src/commands/project.rs`. */
export const WORK_SUFFIX = '.atproj'

/**
 * Tạo một Tác phẩm rồi vào `workspace`, đợi tới khi Panel Lookup thật sự có mặt.
 *
 * @param {string} name tên Tác phẩm — nên mang dấu của lượt chạy để đọc ra được nếu nó
 *   rơi nhầm vào thư mục thật
 * @returns {Promise<void>}
 */
export async function openWorkspaceWithWork(name) {
  const created = await browser.execute(async (workName) => {
    const internals = window.__TAURI_INTERNALS__
    if (internals === undefined) return { ok: false, detail: 'không có cầu IPC' }
    try {
      await internals.invoke('create_work_from_text', {
        name: workName,
        sourceLang: 'zh',
        genre: 'general',
        text: 'Một câu nguồn để bộ nhập có việc mà làm.',
      })
      return { ok: true, detail: '' }
    } catch (err) {
      return { ok: false, detail: String(err && err.code ? err.code : err) }
    }
  }, name)

  if (!created.ok) {
    throw new Error(
      `Fixture không tạo được Tác phẩm "${name}": ${created.detail}\n\n` +
        'Đây là lỗi HẠ TẦNG của bàn đo, không một hồi quy giao diện — đừng đọc một ca đỏ\n' +
        'phía sau nó thành một khuyết tật sản phẩm.',
    )
  }

  // `Mod+2` — `mode.workspace`. `Mod` phân giải thành `Meta` trên macOS.
  await browser.keys(['Meta', '2'])

  // Đợi Panel Lookup có mặt THẬT, không chỉ đợi chế độ đổi: dải chip nguồn chỉ render khi
  // `dictSources.length > 0`, tức sau một lượt IPC nữa. Một ca không đợi vế đó sẽ đỏ bằng
  // "không tìm thấy phần tử" ở đúng lượt chạy đầu tiên trên một máy chậm.
  await $('[data-attribution-open]').waitForExist({
    timeout: 30_000,
    timeoutMsg:
      'Vào `workspace` rồi mà không thấy `[data-attribution-open]` sau 30 giây.\n' +
      'Nút đó nằm trong dải chip nguồn của Panel Lookup, và dải đó chỉ render khi\n' +
      '`dictSources.length > 0` — kiểm `src-tauri/target/debug/dict/*.db` có mặt chưa.',
  })
}
