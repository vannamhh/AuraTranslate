#!/usr/bin/env node
/**
 * Kiểm 3 của Story 1.2 — AC3: phạm vi filesystem tĩnh, cưỡng chế bởi Tauri.
 *
 * Chạy ứng dụng thật, để `src/selftest/scopeCheck.ts` thử HAI CHIỀU trong webview:
 *   - trong scope  → `$RESOURCE/fonts/**` nạp THÀNH CÔNG
 *   - ngoài scope  → `/etc/hosts` (macOS) / `C:\Windows\win.ini` (Windows) trả HTTP 403
 *
 * ⚠️ **Vì sao cần lớp bọc này thay vì chỉ `app.exit(1)` trong Rust.** Đã đo thật
 * 2026-08-03: `tauri dev` **nuốt mã thoát của ứng dụng** — app thoát 1, lệnh bọc vẫn
 * trả 0. Một phép kiểm luôn trả 0 là phép kiểm không cưỡng chế được gì, đúng thứ
 * §Testing standards của story cấm. Nên phán quyết đọc từ dòng `VERDICT:` mà chính
 * self-check in ra, và mã thoát do script này quyết.
 *
 * ⚠️ **Và một phép kiểm không bao giờ trả gì cũng vậy.** Bản bash cũ không có timeout:
 * nếu event không bao giờ tới (frontend gãy trước `onMounted`, một trong hai cờ môi
 * trường không xuống được tiến trình con) thì `app.exit()` không bao giờ được gọi và
 * job CI chạy tới hạn mức rồi bị huỷ. Nhánh "không tìm thấy VERDICT" chỉ chạy được
 * SAU KHI tiến trình đã thoát — tức đúng cái trường hợp nó không xử lý được. Nay có
 * TIMEOUT cứng, và hết giờ là exit 1.
 *
 * Vì sao là Node chứ không phải bash (Ice chốt 2026-08-03): `npm run` trên Windows
 * chạy qua `cmd.exe` — không có bash. Self-check này dò `C:\Windows\win.ini`, nên
 * chính Windows là nền tảng nó cần chạy nhất.
 *
 * Story 1.3 gắn thẳng script này vào pipeline — KHÔNG dựng pipeline thứ hai.
 *
 * Chạy:  npm run check:scope
 */
import { spawn, spawnSync } from 'node:child_process'
import { fileURLToPath } from 'node:url'
import { dirname, join } from 'node:path'

const REPO_ROOT = join(dirname(fileURLToPath(import.meta.url)), '..')
const IS_WIN = process.platform === 'win32'

/**
 * Đọc timeout từ môi trường — và TỪ CHỐI giá trị vô nghĩa thay vì ép nó thành số.
 *
 * ⚠️ `Number(process.env.X ?? 300_000)` sai ở đúng chỗ khó thấy nhất: `??` chỉ bắt
 * `undefined`/`null`. Một biến khai trong `env:` của workflow mà giá trị RỖNG cho
 * `Number('') === 0`; một `5min` cho `NaN`. `setTimeout` ép cả hai về ~1 ms ⇒ SIGKILL
 * bắn tức thì ⇒ script LUÔN exit 1 kèm *"Hết 0s/NaNs mà self-check chưa phát VERDICT —
 * nhiều khả năng webview không mở được"*. Một chẩn đoán sai vĩnh viễn, và không ai nghĩ
 * tới biến môi trường.
 */
function readTimeoutMs(raw, fallback = 300_000) {
  if (raw === undefined || raw === null || String(raw).trim() === '') return fallback
  const n = Number(raw)
  if (!Number.isFinite(n) || n <= 0) {
    console.error(
      `\n\x1b[31mAURA_SCOPE_TIMEOUT_MS = ${JSON.stringify(raw)} không phải một số mili-giây dương.\x1b[0m`,
    )
    console.error('Đây là lỗi cấu hình. Sửa biến hoặc bỏ hẳn nó để dùng mặc định 300000.')
    process.exit(1)
  }
  return n
}

const TIMEOUT_MS = readTimeoutMs(process.env.AURA_SCOPE_TIMEOUT_MS)

console.log('Kiểm 3 — phạm vi asset protocol, hai chiều (AC3)')
console.log('')

// CẢ HAI cờ, luôn luôn. `AURA_SCOPE_SELFTEST` bật listener phía Rust, `VITE_SCOPE_SELFTEST`
// bật self-check phía frontend. Thiếu một cái là lượt chạy treo, không phải chạy sai.
//
// ⚠️ `detached: true` trên POSIX KHÔNG phải trang trí — nó là điều kiện để giết được cả
// CÂY tiến trình ở phần timeout dưới. Xem `killTree()`.
const child = spawn('npx', ['tauri', 'dev'], {
  cwd: REPO_ROOT,
  env: { ...process.env, AURA_SCOPE_SELFTEST: '1', VITE_SCOPE_SELFTEST: '1' },
  stdio: ['ignore', 'pipe', 'pipe'],
  shell: IS_WIN,
  detached: !IS_WIN,
})

/**
 * Giết CẢ CÂY tiến trình, không chỉ tiến trình con trực tiếp.
 *
 * 🔴 Đây là chỗ bản trước hỏng, và nó hỏng đúng ở ca mà timeout tồn tại để xử lý.
 * `child` là `npx` (trên Windows còn thêm một tầng `cmd.exe` vì `shell: true`), còn thứ
 * thật sự chạy là `cargo` → `auratranslate` → vite dev server. `child.kill('SIGKILL')`
 * chỉ giết cái vỏ; các tiến trình CHÁU sống tiếp và vẫn giữ đầu ghi của chính ống
 * stdout/stderr này. Mà sự kiện `'close'` chỉ phát khi tiến trình đã thoát **VÀ** mọi
 * stdio đã đóng ⇒ nhánh `if (timedOut)` bên dưới có thể **KHÔNG BAO GIỜ** chạy, script
 * treo vô hạn, và job CI chạy tới `timeout-minutes: 60` (macOS ×10 hạn mức) — đúng thứ
 * doc-comment đầu tệp nói *"nay có TIMEOUT cứng"* để chặn.
 */
function killTree() {
  if (IS_WIN) {
    // `/T` = cả cây con, `/F` = cưỡng bức. Đây là cách duy nhất chạm được tới cháu khi
    // đã đi qua `cmd.exe`.
    spawnSync('taskkill', ['/PID', String(child.pid), '/T', '/F'], { stdio: 'ignore' })
    return
  }
  // `detached: true` làm `child` thành trưởng nhóm tiến trình; PID âm giết cả nhóm.
  try {
    process.kill(-child.pid, 'SIGKILL')
  } catch {
    // Nhóm đã chết rồi, hoặc chưa kịp lập — thử tiến trình đơn cho chắc.
    try {
      child.kill('SIGKILL')
    } catch {
      /* đã thoát */
    }
  }
}

let log = ''
let timedOut = false

const capture = (stream) => {
  stream.setEncoding('utf8')
  stream.on('data', (chunk) => {
    log += chunk
    process.stdout.write(chunk)
  })
}
capture(child.stdout)
capture(child.stderr)

/** Phán quyết khi hết giờ — dùng chung cho nhánh `'close'` và lưới an toàn dưới. */
const reportTimeout = () => {
  console.log('')
  console.log(
    `\x1b[31mHết ${TIMEOUT_MS / 1000}s mà self-check chưa phát VERDICT — đã giết cây tiến trình.\x1b[0m`,
  )
  console.log('Treo, không phải đạt. Đọc log ở trên.')
  process.exit(1)
}

const timer = setTimeout(() => {
  timedOut = true
  killTree()
  // ⚠️ LƯỚI AN TOÀN, và nó là phần thứ hai của cùng một lỗi. Kể cả khi `killTree()` làm
  // đúng việc, `'close'` vẫn có thể không phát nếu một tiến trình cháu nào đó sống sót
  // và giữ ống. Không có mốc thời gian thứ hai này thì script vẫn treo — chỉ là treo
  // hiếm hơn, tức khó chẩn đoán hơn. 5 giây là quá đủ để một cây đã bị SIGKILL đóng ống.
  setTimeout(reportTimeout, 5_000).unref()
}, TIMEOUT_MS)

child.on('error', (err) => {
  clearTimeout(timer)
  console.error(`\n\x1b[31mKhông chạy được \`npx tauri dev\`: ${err.message}\x1b[0m`)
  process.exit(1)
})

child.on('close', () => {
  clearTimeout(timer)
  console.log('')

  if (timedOut) reportTimeout()

  const verdicts = log.split('\n').map((l) => l.trim()).filter((l) => /^VERDICT: (PASS|FAIL)$/.test(l))
  const verdict = verdicts.at(-1)

  if (!verdict) {
    console.log('\x1b[31mKhông tìm thấy dòng VERDICT trong log.\x1b[0m')
    console.log('Self-check chưa chạy tới nơi — cửa sổ không mở được, hoặc frontend gãy trước')
    console.log("khi phát event. Đọc log ở trên; đừng coi đây là 'đạt'.")
    process.exit(1)
  }

  if (verdict === 'VERDICT: PASS') {
    console.log('\x1b[32mKiểm 3 đạt — cả chiều cho phép lẫn chiều từ chối.\x1b[0m')
    process.exit(0)
  }

  console.log('\x1b[31mKiểm 3 THẤT BẠI.\x1b[0m')
  log
    .split('\n')
    .filter((l) => /^\[(PASS|FAIL)\]/.test(l.trim()))
    .forEach((l) => console.log(`  ${l.trim()}`))
  process.exit(1)
})
