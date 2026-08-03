/**
 * Kiểm 3 của Story 1.2 — AC3: "một thử nghiệm đọc file ngoài scope bị Tauri từ chối".
 *
 * Phép kiểm này PHẢI có cả hai chiều. Chỉ kiểm chiều từ chối thì một cấu hình chặn
 * sạch mọi thứ vẫn "qua", và ta sẽ tin vào một hàng rào không tồn tại:
 *
 *   - trong scope  → `$RESOURCE/fonts/**` nạp THÀNH CÔNG
 *   - ngoài scope  → `/etc/hosts` (macOS) / `C:\Windows\win.ini` (Windows) BỊ TỪ CHỐI
 *
 * Vì sao đây là mã frontend chứ không phải `cargo test`: `assetProtocol.scope` là hàng
 * rào của **webview**. Mã Rust gọi `std::fs` không đi qua nó. Chỉ chạy trong webview
 * thật mới chứng minh được hàng rào.
 *
 * Chạy: `npm run check:scope`. Nó đặt CẢ HAI cờ — `VITE_SCOPE_SELFTEST=1` (frontend
 * chạy self-check) và `AURA_SCOPE_SELFTEST=1` (Rust nghe kết quả và quyết mã thoát).
 * ⛔ Chỉ bật một cờ thì lượt chạy treo: frontend phát event vào hư không, hoặc Rust
 * chờ một event không bao giờ tới. Story 1.3 gắn thẳng `npm run check:scope` vào CI.
 *
 * ⚠️ Phép kiểm này chỉ chạy ở chế độ **dev**, nơi Tauri KHÔNG áp CSP (webview nạp
 * HTML từ Vite, còn Tauri chỉ chèn header CSP cho HTML nó tự phục vụ qua asset
 * protocol). Đó là lý do `fetch` dưới đây đo đúng hàng rào `assetProtocol.scope`
 * chứ không đo CSP. Tổ hợp CSP + asset protocol của bản **release** bàn giao sang
 * Story 1.3 — xem §Review Findings của story.
 */
import { convertFileSrc } from '@tauri-apps/api/core'
import { emit } from '@tauri-apps/api/event'
import { resolveResource } from '@tauri-apps/api/path'

/** Khớp với `SCOPE_SELFTEST_EVENT` ở `src-tauri/src/lib.rs`. */
const SELFTEST_EVENT = 'selftest:scope-check'

export interface ScopeCheckResult {
  name: string
  expectation: 'allowed' | 'denied'
  passed: boolean
  detail: string
}

export interface ScopeCheckReport {
  verdict: 'PASS' | 'FAIL'
  results: ScopeCheckResult[]
  text: string
}

const IN_SCOPE_FONT = 'fonts/SourceSans3[wght].ttf'

function outOfScopePath(): string {
  return navigator.userAgent.includes('Windows') ? 'C:\\Windows\\win.ini' : '/etc/hosts'
}

/** Chiều DƯƠNG: một tài nguyên trong scope phải nạp được, qua đúng đường Story 1.4 sẽ dùng. */
async function checkInScopeLoads(): Promise<ScopeCheckResult> {
  const name = `in-scope: $RESOURCE/${IN_SCOPE_FONT}`
  let url = '<not resolved>'
  try {
    const path = await resolveResource(IN_SCOPE_FONT)
    url = convertFileSrc(path)

    // Phân biệt "scope chặn" với "thiếu tệp" TRƯỚC khi thử FontFace: nếu tệp bị đổi
    // tên hay chưa vào thư mục resource thì asset protocol trả 404, và đổ lỗi cho
    // hàng rào scope là chẩn đoán sai — người vận hành sẽ đi sửa nhầm chỗ.
    const probe = await fetch(url)
    if (probe.status === 404) {
      return {
        name,
        expectation: 'allowed',
        passed: false,
        detail: `HTTP 404 — tệp không có ở ${url}. Đây là LỖI TÀI NGUYÊN, không phải scope chặn.`,
      }
    }
    if (!probe.ok) {
      return {
        name,
        expectation: 'allowed',
        passed: false,
        detail: `scope đã chặn một tài nguyên LẼ RA được phép: HTTP ${probe.status} ở ${url}`,
      }
    }

    const face = new FontFace('AuraScopeProbe', `url("${url}")`, { weight: '200 900' })
    await face.load()
    document.fonts.add(face)
    return { name, expectation: 'allowed', passed: true, detail: `loaded via ${url}` }
  } catch (err) {
    return {
      name,
      expectation: 'allowed',
      passed: false,
      detail: `unexpected rejection at ${url}: ${String(err)}`,
    }
  }
}

/**
 * Chiều ÂM: một đường dẫn ngoài scope phải bị asset protocol từ chối.
 *
 * ⛔ Chỉ **HTTP 403** mới tính là đạt. Một `catch` bắt tất cả, hay một `!res.ok`
 * bắt tất cả, sẽ nuốt luôn 404 (tệp không tồn tại) và mọi lỗi không liên quan —
 * lúc đó phép kiểm xanh kể cả khi `scope` mở toang, tức là ta tin vào một hàng rào
 * không tồn tại. Đúng thứ doc-comment đầu tệp này tồn tại để chặn.
 */
async function checkOutOfScopeDenied(): Promise<ScopeCheckResult> {
  const target = outOfScopePath()
  const name = `out-of-scope: ${target}`
  const url = convertFileSrc(target)
  try {
    const res = await fetch(url)
    if (res.status === 403) {
      return { name, expectation: 'denied', passed: true, detail: 'denied with HTTP 403' }
    }
    if (res.ok) {
      const body = await res.text()
      return {
        name,
        expectation: 'denied',
        passed: false,
        detail: `LEAK — read ${body.length} bytes through ${url}`,
      }
    }
    return {
      name,
      expectation: 'denied',
      passed: false,
      detail: `HTTP ${res.status}, không phải 403 — không chứng minh được scope đã chặn (404 = thiếu tệp)`,
    }
  } catch (err) {
    // Không có response nào: có thể là scope chặn ở tầng protocol, nhưng cũng có thể
    // là CSP, là `convertFileSrc` gãy, hay webview hỏng. Không phân biệt được ⇒ không
    // được tính là đạt.
    return {
      name,
      expectation: 'denied',
      passed: false,
      detail: `không có response, không phân biệt được nguyên nhân: ${String(err)}`,
    }
  }
}

export async function runScopeCheck(): Promise<ScopeCheckReport> {
  const results = [await checkInScopeLoads(), await checkOutOfScopeDenied()]
  const verdict = results.every((r) => r.passed) ? 'PASS' : 'FAIL'

  const lines = [
    'AuraTranslate — asset protocol scope self-check (Story 1.2, AC3)',
    `platform: ${navigator.userAgent.includes('Windows') ? 'windows' : 'unix'}`,
    '',
    ...results.map((r) => `[${r.passed ? 'PASS' : 'FAIL'}] ${r.name}\n        expect=${r.expectation}  ${r.detail}`),
    '',
    `VERDICT: ${verdict}`,
  ]
  const text = lines.join('\n')

  console.log(text)

  // Gửi về Rust để lượt chạy thoát với mã 0/1. Chạy được bằng lệnh mới là phép kiểm;
  // một kết quả chỉ hiện trên màn hình thì không cưỡng chế được gì.
  // Rust chỉ nghe khi `AURA_SCOPE_SELFTEST=1` — bản chạy bình thường bỏ qua event này.
  const report: ScopeCheckReport = { verdict, results, text }
  await emit(SELFTEST_EVENT, report)

  return report
}
