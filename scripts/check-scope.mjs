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
import { spawn } from 'node:child_process'
import { fileURLToPath } from 'node:url'
import { dirname, join } from 'node:path'

const REPO_ROOT = join(dirname(fileURLToPath(import.meta.url)), '..')
const TIMEOUT_MS = Number(process.env.AURA_SCOPE_TIMEOUT_MS ?? 300_000)

console.log('Kiểm 3 — phạm vi asset protocol, hai chiều (AC3)')
console.log('')

// CẢ HAI cờ, luôn luôn. `AURA_SCOPE_SELFTEST` bật listener phía Rust, `VITE_SCOPE_SELFTEST`
// bật self-check phía frontend. Thiếu một cái là lượt chạy treo, không phải chạy sai.
const child = spawn('npx', ['tauri', 'dev'], {
  cwd: REPO_ROOT,
  env: { ...process.env, AURA_SCOPE_SELFTEST: '1', VITE_SCOPE_SELFTEST: '1' },
  stdio: ['ignore', 'pipe', 'pipe'],
  shell: process.platform === 'win32',
})

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

const timer = setTimeout(() => {
  timedOut = true
  child.kill('SIGKILL')
}, TIMEOUT_MS)

child.on('error', (err) => {
  clearTimeout(timer)
  console.error(`\n\x1b[31mKhông chạy được \`npx tauri dev\`: ${err.message}\x1b[0m`)
  process.exit(1)
})

child.on('close', () => {
  clearTimeout(timer)
  console.log('')

  if (timedOut) {
    console.log(
      `\x1b[31mHết ${TIMEOUT_MS / 1000}s mà self-check chưa phát VERDICT — đã giết tiến trình.\x1b[0m`,
    )
    console.log('Treo, không phải đạt. Đọc log ở trên.')
    process.exit(1)
  }

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
