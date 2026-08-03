/**
 * Kiểm 3 của Story 1.2 — AC3: "một thử nghiệm đọc file ngoài scope bị Tauri từ chối".
 * Mở rộng ở Story 1.3 — AC8: cùng phép kiểm đó, chạy NGOÀI chế độ dev, nơi CSP có hiệu lực.
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
 * Chạy:
 *   `npm run check:scope`          — chế độ dev, qua `tauri dev`
 *   `npm run check:scope:bundled`  — chế độ bundled, qua `tauri build --debug` (Story 1.3)
 * Cả hai đặt CẢ HAI cờ — `VITE_SCOPE_SELFTEST=1` (frontend chạy self-check) và
 * `AURA_SCOPE_SELFTEST=1` (Rust nghe kết quả và quyết mã thoát). ⛔ Chỉ bật một cờ thì
 * lượt chạy treo: frontend phát event vào hư không, hoặc Rust chờ một event không bao giờ tới.
 *
 * ─────────────────────────────────────────────────────────────────────────────────
 * ⚠️ HAI CHẾ ĐỘ, VÀ CHÚNG ĐO ĐƯỢC HAI THỨ KHÁC NHAU — Story 1.3 đo ra 2026-08-03
 * ─────────────────────────────────────────────────────────────────────────────────
 *
 * **dev**: Tauri KHÔNG áp CSP — webview nạp HTML từ Vite qua `devUrl`, còn Tauri chỉ
 * chèn header CSP cho HTML nó tự phục vụ. `fetch` đi thẳng tới asset protocol và trả về
 * **mã HTTP thật**, nên đo được CẢ HAI chiều, và chiều âm phân biệt được 403 (scope
 * chặn) với 404 (thiếu tệp).
 *
 * **bundled**: CÓ CSP. Và `connect-src` của dự án là `'self' ipc: http://ipc.localhost`
 * — **không có `asset:`**. Nên `fetch`/`XHR` tới asset protocol bị chặn TRƯỚC KHI chạm
 * hàng rào scope. Đo thật trên bản `.app` debug 2026-08-03, bốn sự kiện
 * `securitypolicyviolation` nêu đích danh `connect-src`:
 *
 *     fetch IN : THROW TypeError: Load failed      ← CSP, không phải scope
 *     fetch OUT: THROW TypeError: Load failed      ← CSP, không phải scope
 *     font IN  : LOADED                            ← `font-src asset:` cho qua ✅
 *     font OUT : THROW NetworkError
 *
 * ⇒ Ở chế độ bundled, chiều DƯƠNG chuyển sang `FontFace` — đúng đường mà Story 1.4 sẽ
 * dùng thật, và do `font-src` quyết chứ không do `connect-src`.
 *
 * ⛔ Còn chiều ÂM thì **KHÔNG đo được** ở chế độ này, và ta ghi thẳng như vậy thay vì
 * lấy một lỗi bất kỳ làm bằng chứng. Lý do đã đo: `FontFace` trả **cùng một**
 * `NetworkError` cho cả ba ca khác hẳn nhau —
 *
 *     ngoài scope (403)                       → NetworkError
 *     trong scope, có thật, KHÔNG phải font   → NetworkError   ← đo thật, `OFL-sourcesans3.txt`
 *     trong scope, không tồn tại (404)        → NetworkError
 *
 * Ca thứ hai chính là thứ giết phép kiểm: nếu `scope` mở toang thì `/etc/hosts` được
 * phục vụ 200 rồi FontFace vẫn ném NetworkError vì nó không phải font — **y hệt** khi
 * bị chặn. Một phép kiểm cho cùng kết quả dù hàng rào còn hay mất thì không kiểm gì cả.
 * Đúng thứ doc-comment này tồn tại để chặn.
 *
 * Chiều âm vẫn có bằng chứng, chỉ là từ chỗ khác: Story 1.2 đã đo **403** ở chế độ dev,
 * và hàng rào scope là **cùng một mã Rust** ở cả hai chế độ — CSP chỉ chồng thêm một
 * lớp lên trên. Xem `deferred-work.md`, mục `connect-src` thiếu `asset:`.
 */
import { convertFileSrc } from '@tauri-apps/api/core'
import { emit } from '@tauri-apps/api/event'
import { resolveResource } from '@tauri-apps/api/path'

/** Khớp với `SCOPE_SELFTEST_EVENT` ở `src-tauri/src/lib.rs`. */
const SELFTEST_EVENT = 'selftest:scope-check'

/**
 * `unmeasured` KHÔNG phải `passed`. Nó tồn tại để một thứ không đo được không bao giờ
 * lặng lẽ đội lốt một thứ đã đạt — nhưng cũng không làm đỏ một pipeline vì một lý do
 * đã biết và đã ghi thành chữ. Cả hai cách đọc sai đều đắt.
 */
export type ScopeCheckStatus = 'pass' | 'fail' | 'unmeasured'

export interface ScopeCheckResult {
  name: string
  expectation: 'allowed' | 'denied'
  status: ScopeCheckStatus
  /** Giữ cho tương thích. ⚠️ `unmeasured` KHÔNG tính là `true`. */
  passed: boolean
  detail: string
}

export interface ScopeCheckReport {
  verdict: 'PASS' | 'FAIL'
  mode: 'dev-no-csp' | 'bundled-csp'
  results: ScopeCheckResult[]
  text: string
}

const IN_SCOPE_FONT = 'fonts/SourceSans3[wght].ttf'

function isWindows(): boolean {
  return navigator.userAgent.includes('Windows')
}

function outOfScopePath(): string {
  return isWindows() ? 'C:\\Windows\\win.ini' : '/etc/hosts'
}

function result(
  name: string,
  expectation: 'allowed' | 'denied',
  status: ScopeCheckStatus,
  detail: string,
): ScopeCheckResult {
  return { name, expectation, status, passed: status === 'pass', detail }
}

// ── Chế độ dev — `fetch` chạm thẳng asset protocol, đọc được mã HTTP ──────────────

/** Chiều DƯƠNG: một tài nguyên trong scope phải nạp được, qua đúng đường Story 1.4 sẽ dùng. */
async function checkInScopeLoads(url: string): Promise<ScopeCheckResult> {
  const name = `in-scope: $RESOURCE/${IN_SCOPE_FONT}`
  try {
    // Phân biệt "scope chặn" với "thiếu tệp" TRƯỚC khi thử FontFace: nếu tệp bị đổi
    // tên hay chưa vào thư mục resource thì asset protocol trả 404, và đổ lỗi cho
    // hàng rào scope là chẩn đoán sai — người vận hành sẽ đi sửa nhầm chỗ.
    const probe = await fetch(url)
    if (probe.status === 404) {
      return result(
        name,
        'allowed',
        'fail',
        `HTTP 404 — tệp không có ở ${url}. Đây là LỖI TÀI NGUYÊN, không phải scope chặn.`,
      )
    }
    if (!probe.ok) {
      return result(
        name,
        'allowed',
        'fail',
        `scope đã chặn một tài nguyên LẼ RA được phép: HTTP ${probe.status} ở ${url}`,
      )
    }

    const face = new FontFace('AuraScopeProbe', `url("${url}")`, { weight: '200 900' })
    await face.load()
    document.fonts.add(face)
    return result(name, 'allowed', 'pass', `loaded via ${url}`)
  } catch (err) {
    return result(name, 'allowed', 'fail', `unexpected rejection at ${url}: ${String(err)}`)
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
async function checkOutOfScopeDenied(target: string, url: string): Promise<ScopeCheckResult> {
  const name = `out-of-scope: ${target}`
  try {
    const res = await fetch(url)
    if (res.status === 403) {
      return result(name, 'denied', 'pass', 'denied with HTTP 403')
    }
    if (res.ok) {
      const body = await res.text()
      return result(name, 'denied', 'fail', `LEAK — read ${body.length} bytes through ${url}`)
    }
    return result(
      name,
      'denied',
      'fail',
      `HTTP ${res.status}, không phải 403 — không chứng minh được scope đã chặn (404 = thiếu tệp)`,
    )
  } catch (err) {
    // Không có response nào: có thể là scope chặn ở tầng protocol, nhưng cũng có thể
    // là CSP, là `convertFileSrc` gãy, hay webview hỏng. Không phân biệt được ⇒ không
    // được tính là đạt.
    return result(
      name,
      'denied',
      'fail',
      `không có response, không phân biệt được nguyên nhân: ${String(err)}`,
    )
  }
}

// ── Chế độ bundled — CSP chặn `connect-src`, chiều dương chuyển sang `font-src` ───

/**
 * Chiều DƯƠNG dưới CSP, qua `FontFace` — đường THẬT của Story 1.4.
 *
 * `LOADED` ở đây là bằng chứng mạnh và không nhập nhằng: asset protocol đã phục vụ tệp
 * **và** CSP đã cho qua. Chỉ chiều âm mới mất khả năng phân biệt, không phải chiều này.
 */
async function checkInScopeLoadsViaFontSrc(url: string): Promise<ScopeCheckResult> {
  const name = `in-scope qua font-src: $RESOURCE/${IN_SCOPE_FONT}`
  try {
    const face = new FontFace('AuraScopeProbe', `url("${url}")`, { weight: '200 900' })
    await face.load()
    document.fonts.add(face)
    return result(
      name,
      'allowed',
      'pass',
      `loaded via ${url} — asset protocol phục vụ được VÀ CSP cho qua`,
    )
  } catch (err) {
    return result(
      name,
      'allowed',
      'fail',
      `KHÔNG nạp được: ${String(err)}\n        ` +
        'Ba nguyên nhân có thể, và FontFace không phân biệt được: scope chặn · tệp thiếu ' +
        'trong bundle · `font-src` không còn cho `asset:`.\n        Kiểm `bundle.resources` trước.',
    )
  }
}

function unmeasurableOutOfScope(target: string, blocked: string[]): ScopeCheckResult {
  return result(
    `out-of-scope: ${target}`,
    'denied',
    'unmeasured',
    'KHÔNG đo được ở chế độ bundled — và điều đó được ghi ra, không được đoán.\n        ' +
      `CSP đã chặn kênh duy nhất đọc được mã HTTP: ${blocked.join(' · ') || 'connect-src'}.\n        ` +
      'Kênh còn lại (`font-src`) trả CÙNG một NetworkError cho "403 scope chặn", cho "tệp ' +
      'có thật nhưng không phải font",\n        và cho "404" — nên nó không phân biệt được ' +
      'hàng rào còn hay mất.\n        Chiều này có bằng chứng 403 từ chế độ dev (Story 1.2), ' +
      'trên CÙNG mã Rust cưỡng chế scope.',
  )
}

// ── Điều phối ────────────────────────────────────────────────────────────────────

export async function runScopeCheck(): Promise<ScopeCheckReport> {
  const path = await resolveResource(IN_SCOPE_FONT)
  const inUrl = convertFileSrc(path)
  const outTarget = outOfScopePath()
  const outUrl = convertFileSrc(outTarget)

  // Ghi lại vi phạm CSP để phân biệt "CSP chặn" với "asset protocol từ chối" — hai thứ
  // trông giống hệt nhau từ phía `fetch`, và đọc nhầm cái này thành cái kia là chẩn đoán
  // sai đúng nghĩa: một cái là hàng rào ta muốn có, một cái là hàng rào ta không biết
  // mình đang đâm vào.
  const cspBlocked: string[] = []
  const onViolation = (e: SecurityPolicyViolationEvent) => {
    cspBlocked.push(e.effectiveDirective || e.violatedDirective)
  }
  document.addEventListener('securitypolicyviolation', onViolation)

  // Một `fetch` thăm dò quyết chế độ. ⚠️ Không đoán từ `import.meta.env`: cờ lúc build
  // nói ta ĐỊNH chạy ở đâu, còn thứ cần biết là CSP có THẬT SỰ đang áp hay không.
  let cspApplies = false
  try {
    await fetch(inUrl)
  } catch {
    // Cho sự kiện violation kịp phát — nó bất đồng bộ với promise của `fetch`.
    await new Promise((r) => setTimeout(r, 100))
    cspApplies = cspBlocked.length > 0
  }

  const mode: ScopeCheckReport['mode'] = cspApplies ? 'bundled-csp' : 'dev-no-csp'
  const results: ScopeCheckResult[] = cspApplies
    ? [
        await checkInScopeLoadsViaFontSrc(inUrl),
        unmeasurableOutOfScope(outTarget, [...new Set(cspBlocked)]),
      ]
    : [await checkInScopeLoads(inUrl), await checkOutOfScopeDenied(outTarget, outUrl)]

  document.removeEventListener('securitypolicyviolation', onViolation)

  // ⚠️ `unmeasured` không làm đỏ, nhưng cũng KHÔNG được đếm là đạt: chỉ `fail` mới quyết
  // verdict, và mọi mục `unmeasured` đều in kèm lý do ngay dưới nó.
  const verdict = results.some((r) => r.status === 'fail') ? 'FAIL' : 'PASS'

  const label = (s: ScopeCheckStatus) => (s === 'pass' ? 'PASS' : s === 'fail' ? 'FAIL' : '----')
  const modeNote = cspApplies
    ? `  (CSP đang áp — chỉ thị đã chặn: ${[...new Set(cspBlocked)].join(', ')})`
    : '  (Tauri không áp CSP ở chế độ dev)'

  const lines = [
    'AuraTranslate — asset protocol scope self-check (Story 1.2 AC3 · Story 1.3 AC8)',
    `platform: ${isWindows() ? 'windows' : 'unix'}`,
    `mode:     ${mode}${modeNote}`,
    '',
    ...results.map(
      (r) => `[${label(r.status)}] ${r.name}\n        expect=${r.expectation}  ${r.detail}`,
    ),
    '',
    `VERDICT: ${verdict}`,
  ]
  const text = lines.join('\n')

  console.log(text)

  // Gửi về Rust để lượt chạy thoát với mã 0/1. Chạy được bằng lệnh mới là phép kiểm;
  // một kết quả chỉ hiện trên màn hình thì không cưỡng chế được gì.
  // Rust chỉ nghe khi `AURA_SCOPE_SELFTEST=1` — bản chạy bình thường bỏ qua event này.
  const report: ScopeCheckReport = { verdict, mode, results, text }
  await emit(SELFTEST_EVENT, report)

  return report
}
