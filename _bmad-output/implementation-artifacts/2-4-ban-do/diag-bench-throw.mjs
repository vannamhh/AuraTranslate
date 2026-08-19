// ══════════════════════════════════════════════════════════════════════════════════════
// ĐỊNH VỊ CHỖ NÉM CỦA `bench.js` — KHÔNG cần dựng lại, 2026-08-19
// ══════════════════════════════════════════════════════════════════════════════════════
// Vòng chẩn đoán ③ (`diag-seam.sh`) đã khoanh được: đoạn nối thêm CHẠY và `put_config` ăn
// ngay lần đầu (`__seam__` có trong kho, n=0), nhưng `bench.js` không tới được dấu sống.
// ⇒ Thân `bench.js` ném ở đâu đó giữa dòng 26 và 297.
//
// 🔴 VÌ SAO CHẠY Ở `happy-dom` CHỨ KHÔNG DỰNG LẠI: mỗi lượt dựng ~10 phút, và luật dừng
// của Task 0 không cho tôi thêm ba lượt nữa. Chỗ ném là một mệnh đề về **JavaScript**,
// không phải về WebKit — nên một DOM giả trả lời được nó, và trả lời trong một giây.
//
// ⚠️ GIỚI HẠN THẬT, ghi ra thay vì để người sau tưởng nhầm: `happy-dom` KHÔNG phải WebKit.
// Nó chứng minh được *"đoạn này ném ở mọi engine"*; nó KHÔNG chứng minh được điều ngược
// lại. Một lượt KHÔNG ném ở đây vẫn có thể ném trên WKWebView. Kết quả dương tính ở đây
// là kết luận được; kết quả âm tính thì không.
//
// Dùng:  node diag-bench-throw.mjs
import { Window } from 'happy-dom'
import fs from 'node:fs'
import path from 'node:path'
import { fileURLToPath } from 'node:url'

const HERE = path.dirname(fileURLToPath(import.meta.url))
const SRC = fs.readFileSync(path.join(HERE, 'bench.js'), 'utf8')

function freshWindow() {
  const w = new Window({ url: 'http://localhost/' })
  // Bề mặt mà `bench.js` chạm tới. Cấp đủ, không cấp thừa — một global thừa có thể che
  // đúng chỗ đang tìm.
  w.requestAnimationFrame = () => 1
  w.cancelAnimationFrame = () => {}
  w.__TAURI_INTERNALS__ = { invoke: () => Promise.resolve() }
  return w
}

// ── ① chạy trọn tệp, bắt lỗi ────────────────────────────────────────────────────────
const w = freshWindow()
let thrown = null
try {
  w.eval(SRC)
} catch (e) {
  thrown = e
}

if (!thrown) {
  console.log('KHÔNG ném ở happy-dom. document.title =', JSON.stringify(w.document.title))
  console.log('⇒ Chỗ ném phụ thuộc ENGINE. Âm tính ở đây không kết luận được — xem §GIỚI HẠN.')
  process.exit(0)
}

console.log('🔴 NÉM:', thrown?.constructor?.name)
console.log('   message:', thrown?.message)
console.log('   stack:')
console.log(String(thrown?.stack ?? '').split('\n').slice(0, 8).map(s => '     ' + s).join('\n'))

// ── ② chia đôi theo DÒNG để chỉ đúng câu lệnh ───────────────────────────────────────
// Cắt tệp ở dòng N, đóng lại IIFE, chạy. Dòng nhỏ nhất còn ném là dòng mang lỗi.
const lines = SRC.split('\n')
function throwsUpTo(n) {
  const head = lines.slice(0, n).join('\n')
  // `bench.js` là một IIFE mở ở dòng 24. Cắt giữa chừng thì phải đóng lại cho parse được.
  const patched = head + '\n})()\n'
  const win = freshWindow()
  try {
    win.eval(patched)
    return false
  } catch (e) {
    // Lỗi cú pháp do chính lượt cắt thì KHÔNG tính — nó là tạo tác của phép dò.
    if (e instanceof SyntaxError) return null
    return true
  }
}

let firstBad = null
for (let n = 25; n <= lines.length; n++) {
  const r = throwsUpTo(n)
  if (r === true) { firstBad = n; break }
}

console.log('')
if (firstBad) {
  console.log(`⇒ DÒNG NÉM ĐẦU TIÊN: ${firstBad}`)
  const from = Math.max(0, firstBad - 6)
  lines.slice(from, firstBad).forEach((l, i) => {
    const no = from + i + 1
    console.log(`   ${no === firstBad ? '🔴' : '  '} ${String(no).padStart(4)}: ${l}`)
  })
} else {
  console.log('⇒ Không cắt được về một dòng — mọi lượt cắt đều lỗi cú pháp hoặc không ném.')
}
