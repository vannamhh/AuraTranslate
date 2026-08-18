/**
 * Phép kiểm sức khoẻ của Vite dev — **AC1 của Story 2.12**.
 *
 * ## Vì sao tệp này tồn tại
 *
 * Bản cũ (`wdio.conf.mjs:191-198` trước lượt này) là một `fetch(DEV_URL)` rồi `return
 * res.ok`. Nó **không đọc body và không chạm một module nào**, nên nó nói *"Vite lành"*
 * về một máy chủ mà app không bao giờ khởi động nổi trên đó. Hậu quả đã ghi trong sổ nợ:
 * 11 spec đỏ vì một lý do **không liên quan**, và không câu nào chỉ đúng nguyên nhân.
 *
 * ## 🔴 Số đo dựng nên hình dạng dưới đây — không một giả thuyết nào
 *
 * Đối chứng dương 2026-08-18 (Vite 8, cổng 1421, một lượt phá thật ở `src/App.vue`):
 *
 * | Yêu cầu | Vite **lành** | Vite **hấp hối** |
 * |---|---|---|
 * | `/` | 200 · `text/html` · 355 B | **200 · `text/html` · 355 B** — *y hệt* |
 * | `/src/main.ts` | 200 · `text/javascript` · 88.494 B | **200 · `text/javascript` · 88.494 B** — *y hệt* |
 * | `/src/App.vue` | 200 · `text/javascript` · 56.203 B | **500 · `content-type` rỗng** · trang lỗi HTML 24.380 B |
 *
 * ⚠️ **Hai giả thuyết rẻ đã bị chính phép đo trên BÁC, ghi ra thay vì để người sau thử lại:**
 *
 * 1. *"Kiểm `/` kỹ hơn"* — vô vọng. `/` là `index.html` phục vụ **tĩnh**; nó không đi qua
 *    một lượt biến đổi module nào, nên nó **không thể** biết graph đã vỡ. Hai cột `/` ở
 *    trên giống nhau tới từng byte.
 * 2. *"Nạp module entry là đủ"* — **SAI, và đây là chỗ dễ dừng lại nhất.** Vite biến đổi
 *    module **LƯỜI, theo từng yêu cầu**. `main.ts` chỉ *khai* `import App from './App.vue'`;
 *    lượt biến đổi `App.vue` chỉ chạy khi có ai **hỏi** `/src/App.vue`. Nên entry vẫn 200
 *    sạch trong khi thứ nó trỏ tới đã chết.
 *
 * ⇒ Phép kiểm phải **đi theo graph**, và đó là việc của [`crawlModuleGraph`].
 *
 * ## Giá, đo chứ không ước
 *
 * - Vite **ấm**: 58 module · **270 ms** (ba lượt liên tiếp: 270 · 271 · 228 ms).
 * - Vite **nguội** (đã xoá `node_modules/.vite`): 58 module · **4.129 ms**.
 *
 * 🔴 Và giá nguội đó **không phải chi phí thêm — nó là chi phí DỜI CHỖ.** Trình duyệt trả
 * đúng khoản ấy ở lượt nạp đầu tiên dù có phép kiểm này hay không; chạy nó ở `onPrepare`
 * làm ấm sẵn Vite **trước** khi app mở. Lượt duyệt trả tiền cho chính nó.
 *
 * ## GIỚI HẠN THẬT, ghi ra thay vì để người sau tự phát hiện
 *
 * Phép duyệt đi từ entry và chỉ theo những gì **entry với tới**. Một tệp trong `src/**`
 * mà **không module nào import** sẽ không được chạm, nên một lỗi cú pháp ở đó đi qua sạch.
 * Điều đó **đúng theo cấu tạo**, không phải một chỗ chưa làm tới: một module không ai
 * import cũng không làm app chết. Cổng canh mệnh đề *"mọi tệp `src/**` phải lành"* là
 * `npm run build` (`vue-tsc`), không phải tệp này.
 */

/**
 * Đường module entry của app. Nó là **dây**: `index.html:10` khai đúng chuỗi này ở
 * `<script type="module" src="…">`. Đổi một bên mà quên bên kia thì phép duyệt bắt đầu
 * từ một URL 404 và [`crawlModuleGraph`] đỏ ngay — chứ không xanh giả.
 */
export const ENTRY_MODULE = '/src/main.ts'

/**
 * Trần số module một lượt duyệt được phép chạm.
 *
 * ⚠️ Đây là một **hàng rào chống vòng lặp**, không một mệnh đề về quần thể. Số thật hôm
 * nay là 58; trần đặt rộng gấp bốn để một lượt thêm tệp bình thường không đụng nó, nhưng
 * một `matchAll` hỏng sinh URL vô hạn thì đụng.
 */
const MODULE_CEILING = 250

/**
 * Phán quyết cho MỘT lượt trả lời — **hàm thuần, không I/O**, và đó là điều kiện để
 * [`selfCheckDevServerHealth`] gọi được **chính hàm này** thay vì một bản chép.
 *
 * 🔴 *"Một bản đồ CHÉP hàm sản phẩm sẽ đo bản chép, và bản chép cứ đi"* — bài học F4 ②
 * của retro Epic 2, đã trả giá ở Story 2.9 (bản đồ chép cho `17`/`19` y hệt lượt trước
 * **sau khi đã vá**, tức nó báo *"chưa vá"* trên một sản phẩm ĐÃ VÁ).
 *
 * @param {{ url: string, status: number, contentType: string | null }} res
 * @returns {{ ok: boolean, reason: string | null }}
 */
export function judgeModuleResponse({ url, status, contentType }) {
  if (status !== 200) {
    return { ok: false, reason: `HTTP ${status} (Vite tra loi mot trang loi, khong mot module)` }
  }
  const ct = (contentType ?? '').toLowerCase()

  // ⚠️ Đo được, và là một dương tính giả CÓ THẬT: `/src/tokens/tokens.json` và
  // `/src/i18n/vi.json` được phục vụ `application/json` trên một Vite **hoàn toàn lành**.
  // Một luật *"phải là javascript"* trần sẽ đỏ oan đúng hai chỗ đó ở mọi lượt chạy — rồi
  // bị nới cho hết đỏ, và cổng thành vô nghĩa. Nên hai đuôi tệp có luật RIÊNG, không một
  // ngoại lệ theo tên.
  if (url.endsWith('.json')) {
    return ct.includes('json')
      ? { ok: true, reason: null }
      : { ok: false, reason: `content-type "${ct || '(rong)'}" — mot .json phai la application/json` }
  }
  if (!ct.includes('javascript')) {
    return {
      ok: false,
      reason: `content-type "${ct || '(rong)'}" — Vite tra HTML/khong-JS cho mot duong module`,
    }
  }
  return { ok: true, reason: null }
}

/**
 * Rút mọi đường `/src/…` mà một module đã biến đổi trỏ tới.
 *
 * Vite viết lại specifier trần thành `/node_modules/.vite/deps/…` và specifier tương đối
 * thành `/src/…` tuyệt đối, nên một phép khớp trên tiền tố `/src/` là đủ và **không** cần
 * một parser JS. Truy vấn `?v=…`/`?t=…` bị cắt để hai lượt HMR không sinh hai đỉnh khác nhau.
 *
 * @param {string} body
 * @returns {string[]}
 */
export function extractSrcImports(body) {
  const out = []
  for (const m of body.matchAll(/["'](\/src\/[^"'?]+)(\?[^"']*)?["']/g)) out.push(m[1])
  return out
}

/**
 * Duyệt graph module theo chiều rộng từ [`ENTRY_MODULE`].
 *
 * `fetchModule` được **tiêm vào** chứ không gọi `fetch` trần: đó là thứ cho
 * [`selfCheckDevServerHealth`] dựng một Vite hấp hối **giả lập tất định** mà không phải
 * phá một tệp thật trong `src/`.
 *
 * @param {(url: string) => Promise<{ status: number, contentType: string | null, body: string }>} fetchModule
 * @returns {Promise<{ visited: string[], bad: { url: string, reason: string }[] }>}
 */
export async function crawlModuleGraph(fetchModule) {
  const seen = new Set()
  const bad = []
  const queue = [ENTRY_MODULE]

  while (queue.length > 0 && seen.size < MODULE_CEILING) {
    const url = queue.shift()
    if (seen.has(url)) continue
    seen.add(url)

    let res
    try {
      res = await fetchModule(url)
    } catch (err) {
      // Không nối được / hết giờ. Đây là một phán quyết ĐỎ, không một lỗi hạ tầng: một
      // module không lấy được là một module app không nạp được.
      bad.push({ url, reason: `khong lay duoc: ${err instanceof Error ? err.message : String(err)}` })
      continue
    }

    const verdict = judgeModuleResponse({ url, status: res.status, contentType: res.contentType })
    if (!verdict.ok) {
      // 🔴 KHÔNG đi tiếp qua một đỉnh đã hỏng. Thân của nó là một trang lỗi HTML, và moi
      // `/src/…` ra từ một trang lỗi là sinh ra những đỉnh không có thật.
      bad.push({ url, reason: verdict.reason })
      continue
    }
    if (!url.endsWith('.json')) for (const next of extractSrcImports(res.body)) queue.push(next)
  }

  return { visited: [...seen], bad }
}

/**
 * Câu báo khi Vite hấp hối — **AC1 vế *"nói ĐÚNG nguyên nhân"***.
 *
 * ⚠️ Nó nêu đích danh module đầu tiên gãy và lý do. Không có vế này thì bộ dừng bằng một
 * câu chung chung, và người đọc quay lại đúng chỗ cũ: *"11 spec đỏ, không biết vì sao"*.
 *
 * @param {{ url: string, reason: string }[]} bad
 * @param {number} visitedCount
 */
export function describeBrokenGraph(bad, visitedCount) {
  const lines = bad.map((b) => `  · ${b.url} — ${b.reason}`)
  return (
    `Vite ĐANG CHẠY nhưng module graph đã VỠ — ${bad.length} module gãy trong ${visitedCount} module đã chạm.\n\n` +
    `${lines.join('\n')}\n\n` +
    '🔴 Đây là một lỗi HẠ TẦNG của bàn đo, KHÔNG một hồi quy sản phẩm. Bộ dừng ở đây có\n' +
    'chủ ý: chạy tiếp sẽ cho 11 spec đỏ vì một lý do không liên quan gì tới thứ chúng đo.\n' +
    'Sửa module ở trên rồi chạy lại. `npm run build` cho cùng chẩn đoán đó kèm vị trí dòng.'
  )
}

/**
 * 🔴 **PHÉP TỰ KIỂM — chứng minh phán quyết ĐỎ ĐƯỢC, và không ĐỎ OAN.**
 *
 * *"Một cổng chưa bao giờ đỏ là một cổng chưa ai biết nó có chạy không"*
 * (`project-context.md`, §Luật của một CỔNG). Chạy ở `onPrepare` **trước** khi phép kiểm
 * thật được tin; nó gọi **chính** [`crawlModuleGraph`] và **chính** [`judgeModuleResponse`],
 * không một bản chép.
 *
 * Hai ca dương (phải ĐỎ) và một ca âm (KHÔNG được đỏ) tái dựng đúng ba cột của bảng số đo
 * ở đầu tệp — gồm cả cột đã bác giả thuyết *"nạp entry là đủ"*: ca hấp hối để entry **200
 * sạch** và chỉ làm vỡ đỉnh sau nó.
 *
 * @throws {Error} khi chính phép kiểm hỏng
 */
export async function selfCheckDevServerHealth() {
  const JS = 'text/javascript'
  const graph = {
    '/src/main.ts': `import App from "/src/App.vue"; import "/src/i18n/vi.json";`,
    '/src/App.vue': 'export default {}',
    '/src/i18n/vi.json': '{}',
  }
  const serve = (broken) => async (url) => {
    if (!(url in graph)) throw new Error(`tu kiem: dinh la "${url}" khong co trong graph gia lap`)
    if (url === broken) return { status: 500, contentType: null, body: '<!DOCTYPE html><title>Error</title>' }
    if (url.endsWith('.json')) return { status: 200, contentType: 'application/json', body: graph[url] }
    return { status: 200, contentType: JS, body: graph[url] }
  }

  // ── Ca ÂM: graph lành ⇒ KHÔNG được đỏ ──────────────────────────────────────────────
  const healthy = await crawlModuleGraph(serve(null))
  if (healthy.bad.length !== 0) {
    throw new Error(
      `tu kiem devServerHealth ĐỎ OAN: mot graph lanh cho ${healthy.bad.length} module gay ` +
        `(${healthy.bad.map((b) => b.url).join(', ')}). Mot phep kiem do oan se bi noi cho het do, ` +
        'va roi no khong con canh gi.',
    )
  }
  if (healthy.visited.length !== 3) {
    throw new Error(
      `tu kiem devServerHealth: cho 3 dinh, cham ${healthy.visited.length}. Phep duyet khong di ` +
        'het graph thi mot module gay o cuoi duong se lot.',
    )
  }

  // ── Ca DƯƠNG ①: đúng hình dạng đã bác giả thuyết "nạp entry là đủ" ────────────────
  const dying = await crawlModuleGraph(serve('/src/App.vue'))
  if (dying.bad.length !== 1 || dying.bad[0].url !== '/src/App.vue') {
    throw new Error(
      'tu kiem devServerHealth KHONG DO DUOC tren mot graph vo o dinh THU HAI. Day dung la ' +
        'hinh dang ma phep do 2026-08-18 da bat: entry van 200 sach trong khi thu no tro toi da chet.',
    )
  }

  // ── Ca DƯƠNG ②: entry gãy ngay ⇒ đỏ, và KHÔNG đi tiếp qua một đỉnh đã hỏng ────────
  const deadEntry = await crawlModuleGraph(serve(ENTRY_MODULE))
  if (deadEntry.bad.length !== 1 || deadEntry.visited.length !== 1) {
    throw new Error(
      'tu kiem devServerHealth: entry gay phai cho dung 1 module gay va dung 1 dinh cham. ' +
        'Di tiep qua mot dinh hong la moi `/src/…` ra tu mot trang loi HTML.',
    )
  }
}
