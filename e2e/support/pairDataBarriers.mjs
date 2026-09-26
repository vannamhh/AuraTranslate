import { join } from 'node:path'
import { existsSync, readFileSync } from 'node:fs'

/**
 * Ba hàng rào dữ liệu thật của MỘT cặp thư mục app con đã dùng — tách khỏi
 * `wdio.conf.mjs::onComplete` để một ca vitest lái được bằng cặp GIẢ LẬP, không cần dựng
 * app thật (`deferred-work.md`, "cách ly trạng thái giữa các spec"). Không xoá thư mục — vòng
 * gọi vẫn làm việc đó, sau khi đã gom hết thất bại của MỌI cặp (Review Triage Log #7).
 *
 * @param {{dataDir: string, libraryDir: string, realLibraryBefore: string, realLibraryAfter: string, exitCode: number | undefined}} pair
 * @param {{realLibraryPathValue: string, globalDbFile: string, libraryIndexDbFile: string, dataDirEnv: string, libraryRootEnv: string}} context
 * @returns {string[]} thông điệp lỗi — rỗng nếu cặp qua cả ba hàng rào.
 */
export function checkPairDataBarriers(pair, context) {
  const { realLibraryPathValue, globalDbFile, libraryIndexDbFile, dataDirEnv, libraryRootEnv } = context
  const failures = []

  // ── Hàng rào chiều ÂM: thư mục Library THẬT phải y nguyên ────────────────────
  if (pair.realLibraryBefore !== pair.realLibraryAfter) {
    failures.push(
      `Thư mục Library THẬT của bạn đã ĐỔI trong lượt e2e này, khi app chạy trên cặp\n` +
        `  ${pair.dataDir}\n  ${pair.libraryDir}\n` +
        `Đường dẫn thật: ${realLibraryPathValue}\n` +
        `  trước: ${pair.realLibraryBefore}\n  sau:   ${pair.realLibraryAfter}\n\n` +
        'Bộ e2e không được chạm vào đó. Nguyên nhân hay gặp:\n' +
        `  1. nhị phân dựng thiếu \`--features wdio\` ⇒ \`${libraryRootEnv}\` không được đọc;\n` +
        '  2. tên biến ở `src-tauri/src/lib.rs` đã đổi mà tệp này chưa đổi theo;\n' +
        '  3. một đường ghi mới không đi qua `default_library_root()` — đó là một bề\n' +
        '     mặt THỨ BA, và nó cần một móc riêng chứ không một ngoại lệ ở đây.\n\n' +
        'Nếu bạn vừa mở ứng dụng thật song song với lượt chạy này thì đây là báo động\n' +
        'giả — chạy lại khi app đã đóng, đừng gỡ phép kiểm.',
    )
  }

  // ── Hàng rào chiều ĐỌC: `library-index.db` không được nhắc đường dẫn Library THẬT ──
  const indexPath = join(pair.dataDir, libraryIndexDbFile)
  let indexBytes = null
  try {
    indexBytes = readFileSync(indexPath)
  } catch (err) {
    if (err.code === 'ENOENT') {
      // Chưa từng mở/lập chỉ mục trên cặp này -- KHÔNG phải lỗi, bỏ qua êm.
      indexBytes = null
    } else {
      // Lỗi HẠ TẦNG (quyền, đĩa hỏng, …) -- KHÔNG phải một phép kiểm ĐỎ.
      console.warn(
        `[e2e] không đọc được ${indexPath} để kiểm hàng rào chiều ĐỌC (${err.code}) -- ` +
          'bỏ qua phép kiểm này, đây là lỗi HẠ TẦNG, không phải một phát hiện.',
      )
      indexBytes = null
    }
  }
  if (indexBytes !== null) {
    const needle = Buffer.from(realLibraryPathValue, 'utf8')
    if (indexBytes.includes(needle)) {
      failures.push(
        `${indexPath} (${libraryIndexDbFile}) chứa đường dẫn Library THẬT của bạn:\n` +
          `  ${realLibraryPathValue}\n\n` +
          'Nghĩa là ứng dụng đã ĐỌC và lập chỉ mục thư viện thật trong lượt e2e này, dù\n' +
          'không byte nào bị GHI vào thư mục đó (hàng rào chữ ký ở trên không bắt được\n' +
          'chiều này). Đây chính là dấu vết của "Một lượt e2e ĐỎ chưa chẩn đoán được"\n' +
          '(`deferred-work.md`) — đọc mục đó trước khi sửa bất cứ dòng nào.',
      )
    }
  }

  // ── Hàng rào chiều DƯƠNG: `global.db` phải NẰM trong `$APPDATA` tạm ───────────
  if (pair.exitCode === 0) {
    const storePath = join(pair.dataDir, globalDbFile)
    if (!existsSync(storePath)) {
      failures.push(
        `Tệp spec chạy trên cặp ${pair.dataDir} xanh nhưng KHÔNG thấy ${globalDbFile}\n` +
          'trong đó.\n\n' +
          'Phần lớn nguyên nhân nghĩa là app con đã ghi vào `$APPDATA` THẬT của bạn, không\n' +
          'vào thư mục tạm — một lượt xanh ở đây là một lượt xanh giả. Theo thứ tự hay gặp:\n' +
          `  1. nhị phân dựng THIẾU \`--features wdio\` ⇒ \`${dataDirEnv}\` không được đọc\n` +
          '     (`npm run test:e2e` truyền sẵn; một lượt `cargo build` tay thì không);\n' +
          '  2. tên biến ở `src-tauri/src/lib.rs` đã đổi mà tệp này chưa đổi theo;\n' +
          '  3. `open_global_store` thôi không đi qua `data_dir_override()` nữa;\n' +
          '  4. `onWorkerEnd` không kịp giết app cũ trước khi ghi biến môi trường mới,\n' +
          '     nên tệp spec này chạy trên `$APPDATA` của cặp TRƯỚC;\n' +
          '  5. KHÔNG kho nào được mở cả — `open_global_store` (`src-tauri/src/lib.rs:899-\n' +
          '     940`) chỉ `eprintln!` rồi `return` khi `create_dir_all` trên thư mục tạm\n' +
          '     hay `Store::open` thất bại, nên app tiếp tục chạy KHÔNG store, không panic,\n' +
          '     không rơi vào nhánh dữ liệu thật; xem log stderr của app cho dòng\n' +
          '     `store[global] …`.\n\n' +
          'Đừng bỏ phép kiểm này để cho xanh — nó là thứ duy nhất đứng giữa bộ đo và\n' +
          'cấu hình thật của bạn.',
      )
    }
  }

  return failures
}
