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
 * Chạy: `VITE_SCOPE_SELFTEST=1 npm run tauri dev` — kết quả in ra console và hiện
 * trên cửa sổ. Story 1.3 tự động hoá lượt chạy này trên CI.
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
  try {
    const path = await resolveResource(IN_SCOPE_FONT)
    const url = convertFileSrc(path)
    const face = new FontFace('AuraScopeProbe', `url("${url}")`, { weight: '200 900' })
    await face.load()
    document.fonts.add(face)
    return { name, expectation: 'allowed', passed: true, detail: `loaded via ${url}` }
  } catch (err) {
    return {
      name,
      expectation: 'allowed',
      passed: false,
      detail: `unexpected rejection: ${String(err)}`,
    }
  }
}

/** Chiều ÂM: một đường dẫn ngoài scope phải bị webview từ chối nạp. */
async function checkOutOfScopeDenied(): Promise<ScopeCheckResult> {
  const target = outOfScopePath()
  const name = `out-of-scope: ${target}`
  const url = convertFileSrc(target)
  try {
    const res = await fetch(url)
    if (!res.ok) {
      return {
        name,
        expectation: 'denied',
        passed: true,
        detail: `denied with HTTP ${res.status}`,
      }
    }
    const body = await res.text()
    return {
      name,
      expectation: 'denied',
      passed: false,
      detail: `LEAK — read ${body.length} bytes through ${url}`,
    }
  } catch (err) {
    // Webview từ chối trước cả khi có response: "asset protocol not configured to
    // allow the path". Đó chính là kết quả mong đợi.
    return { name, expectation: 'denied', passed: true, detail: `denied: ${String(err)}` }
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

  // eslint-disable-next-line no-console
  console.log(text)

  // Gửi về Rust để lượt chạy thoát với mã 0/1. Chạy được bằng lệnh mới là phép kiểm;
  // một kết quả chỉ hiện trên màn hình thì không cưỡng chế được gì.
  // Rust chỉ nghe khi `AURA_SCOPE_SELFTEST=1` — bản chạy bình thường bỏ qua event này.
  const report: ScopeCheckReport = { verdict, results, text }
  await emit(SELFTEST_EVENT, report)

  return report
}
