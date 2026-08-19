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

  // 🔵 **CODE REVIEW BA TẦNG 2026-08-19 — TRẦN KHÔNG ĐƯỢC CẮT TRONG IM LẶNG.**
  //
  // 🔴 Bản đầu thoát vòng vì chạm trần rồi trả về hình dạng **y hệt** một lượt duyệt trọn vẹn
  // — cùng `{ visited, bad }`, không một cờ nào. Ngày quần thể module vượt 250 *(hôm nay 58)*,
  // một module **vỡ** nằm sau điểm cắt sẽ không bao giờ được thăm, và `assertModuleGraphHealthy`
  // báo *"lành"* trên một Vite thật sự vỡ. Đó là xanh oan — đúng lớp lỗi mà cả tệp này viết ra
  // để chống, đi vào bằng cửa của chính hàng rào chống vòng lặp.
  //
  // ⚠️ Và nó **không** phải một phép kiểm đỏ: chạm trần nghĩa là **tham số của bàn đo đã cũ**,
  // không nghĩa là app hỏng. Luật *"lỗi hạ tầng KHÔNG phải một phép kiểm đỏ"* đòi một câu nói
  // đúng thứ đã xảy ra, nên `truncated` đi ra ngoài như một vế **thứ ba**, và nơi gọi dựng câu
  // chẩn đoán riêng cho nó — không trộn vào danh sách `bad`.
  // ⚠️ Vị từ là *"còn việc CHƯA LÀM trong hàng đợi"*, không phải *"hàng đợi không rỗng"* và
  // cũng không phải `seen.size >= MODULE_CEILING`. Hai vị từ sau đều SAI ở một chiều: hàng đợi
  // có thể còn những URL **đã thăm** *(chúng được `push` rồi bị `continue` bỏ qua)*, và một
  // graph có đúng 250 đỉnh duyệt **trọn vẹn** vẫn chạm `seen.size === MODULE_CEILING`. Cả hai
  // cho ĐỎ OAN, và một phép kiểm đỏ oan sẽ bị nới cho hết đỏ.
  const truncated = queue.some((u) => !seen.has(u))
  return { visited: [...seen], bad, truncated }
}

/**
 * 🔵 **Câu báo khi lượt duyệt CHẠM TRẦN — code review 2026-08-19.**
 *
 * 🔴 Nó cố ý **không** dùng chữ của [`describeBrokenGraph`]: hai thứ khác hạng nhau. *"Graph
 * vỡ"* là một phán quyết về **app**; *"chạm trần"* là một phán quyết về **bàn đo**. Trộn hai
 * câu là dạy người đọc đi sửa sai chỗ.
 *
 * @param {number} visitedCount
 */
export function describeTruncatedGraph(visitedCount) {
  return (
    `Luot duyet module graph CHAM TRAN ${MODULE_CEILING} sau ${visitedCount} module — no KHONG\n` +
    'di tron graph, nen mot module vo nam sau diem cat da KHONG duoc kiem.\n\n' +
    '🔴 Day la LOI HA TANG, KHONG mot phep kiem dat va cung KHONG mot hoi quy san pham:\n' +
    'tran la mot hang rao chong vong lap, va cham nó nghia la tham so cua ban do da cu.\n' +
    `Xet lai \`MODULE_CEILING\` o \`e2e/support/devServerHealth.mjs\` (so that 2026-08-18: 58),\n` +
    'hoac tim mot `matchAll` hong dang sinh URL vo han.'
  )
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

  // ═══════════════════════════════════════════════════════════════════════════════════
  // 🔵 CA ③ + CA ÂM ② — TRẦN, thêm ở code review 2026-08-19
  // ═══════════════════════════════════════════════════════════════════════════════════
  // 🔴 Vế `truncated` là một cơ chế mới, nên nó phải có phép tự kiểm **cả hai chiều** — đúng
  // luật đã áp cho mọi vế khác của tệp này. Không có hai ca dưới đây, `truncated` vào kho ở
  // trạng thái *"chưa ai biết nó chạy không"*, và cửa xanh-oan chỉ đổi hình dạng chứ chưa đóng.

  // Ca DƯƠNG ③: một chuỗi DÀI HƠN trần ⇒ phải báo CẮT.
  const chain = {}
  const DAI = MODULE_CEILING + 50
  for (let i = 0; i < DAI; i += 1) {
    const here = i === 0 ? ENTRY_MODULE : `/src/n${i}.ts`
    chain[here] = i === DAI - 1 ? 'export default {}' : `import "/src/n${i + 1}.ts";`
  }
  const serveChain = async (url) => {
    if (!(url in chain)) throw new Error(`tu kiem: dinh la "${url}" khong co trong chuoi gia lap`)
    return { status: 200, contentType: JS, body: chain[url] }
  }
  const cat = await crawlModuleGraph(serveChain)
  if (!cat.truncated) {
    throw new Error(
      `tu kiem devServerHealth: mot chuoi ${DAI} module (tran ${MODULE_CEILING}) KHONG bao cat. ` +
        'Nghia la mot module vo nam sau diem cat se cho mot luot XANH tren mot Vite that su vo.',
    )
  }
  if (cat.bad.length !== 0) {
    throw new Error(
      'tu kiem devServerHealth: mot luot CAT khong duoc sinh ra module `bad` nao. "Cham tran" la ' +
        'mot phan quyet ve BAN DO; "vo" la mot phan quyet ve APP. Tron hai la day nguoi doc sua sai cho.',
    )
  }

  // Ca ÂM ②: graph lành ba đỉnh **KHÔNG** được báo cắt.
  if (healthy.truncated) {
    throw new Error(
      'tu kiem devServerHealth DO OAN: mot graph lanh 3 dinh bi cham la CAT — vi tu `truncated` ' +
        'dang doc mot thu khong phai "con viec CHUA LAM".',
    )
  }

  // ── Ca ÂM ③: ĐÚNG `MODULE_CEILING` đỉnh, duyệt TRỌN VẸN, đỉnh cuối trỏ NGƯỢC lại ────
  //
  // 🔴 **Ca âm ② một mình KHÔNG phân biệt được ba vị từ** — đo bằng đột biến 2026-08-19: đổi
  // `truncated` thành `queue.length > 0` mà tự kiểm vẫn XANH. Lý do: vòng lặp `shift()` cho tới
  // khi hàng đợi rỗng, nên với một graph nhỏ thì `queue.length === 0` ở mọi lối ra. Hai vị từ
  // chỉ tách nhau ở **đúng biên**: chạm trần **và** phần còn lại trong hàng đợi toàn URL đã
  // thăm. Ca dưới đây dựng đúng biên ấy — 250 đỉnh, duyệt trọn, đỉnh cuối `import` lại đỉnh
  // đầu ⇒ ra khỏi vòng với `queue = ['/src/n1.ts']` mà `n1` **đã** trong `seen`.
  // ⇒ *"Cây rỗng không phải cây sạch"*, và một ca âm không cắn được đột biến không phải một ca âm.
  const bien = {}
  for (let i = 0; i < MODULE_CEILING; i += 1) {
    const here = i === 0 ? ENTRY_MODULE : `/src/n${i}.ts`
    bien[here] = i === MODULE_CEILING - 1 ? 'import "/src/n1.ts";' : `import "/src/n${i + 1}.ts";`
  }
  const serveBien = async (url) => {
    if (!(url in bien)) throw new Error(`tu kiem: dinh la "${url}" khong co trong graph bien`)
    return { status: 200, contentType: JS, body: bien[url] }
  }
  const tronBien = await crawlModuleGraph(serveBien)
  if (tronBien.visited.length !== MODULE_CEILING) {
    throw new Error(
      `tu kiem devServerHealth: graph bien phai cham dung ${MODULE_CEILING} dinh, cham ` +
        `${tronBien.visited.length}. Ca am nay chi co nghia khi no dung o DUNG bien.`,
    )
  }
  if (tronBien.truncated) {
    throw new Error(
      `tu kiem devServerHealth DO OAN: ${MODULE_CEILING} dinh duyet TRON VEN bi cham la CAT. ` +
        'Vi tu `truncated` dang doc "hang doi khong rong" hay "seen.size >= tran" thay vi "con ' +
        'viec CHUA LAM" — ca hai deu do oan o dung bien nay. Xem chu thich tai cho.',
    )
  }
}
