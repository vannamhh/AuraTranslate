/* eslint-disable */
/**
 * BÀN ĐO của Story 2.4 — mã DÙNG MỘT LẦN, KHÔNG vào kho.
 * Tiền lệ: `probe-typing.mjs` / `probe-paste.mjs` của Story 2.3 (scratchpad).
 *
 * Nó được tiêm vào `dist/index.html` SAU `npm run build` và TRƯỚC khi `cargo build` nhúng
 * `dist/` vào nhị phân. Nó là script CỔ ĐIỂN đặt trong <head> nên nó chạy TRƯỚC bundle
 * module của app — điều kiện để vá được `__TAURI_INTERNALS__.invoke`.
 *
 * 🔴 Nó KHÔNG đổi một dòng mã sản phẩm nào. Nó chỉ:
 *   ① đếm delta rAF (Quyết định #3 đường (a));
 *   ② đóng dấu thời gian mọi lượt `input` và mọi lượt `save_segment_targets` bay qua IPC —
 *      đó là bộ lọc bắt buộc của Quyết định #3 (*"loại delta không có input/flush nào"*);
 *   ③ đổ số ra qua `put_config` — một lệnh IPC ĐÃ CÓ, gọi ĐÚNG MỘT LẦN sau khi ngừng lấy mẫu.
 *
 * Phím điều khiển (bơm từ ngoài bằng `osascript`, tầng hệ điều hành — Quyết định #2 đường (c)):
 *   Ctrl+Alt+Shift+1  dựng Tác phẩm từ tệp cố định rồi tách segment, sau đó nạp lại trang
 *   Ctrl+Alt+Shift+2  bắt đầu lấy mẫu
 *   Ctrl+Alt+Shift+3  ngừng lấy mẫu + đổ số qua put_config
 *   Ctrl+Alt+Shift+4  chạy bộ đo ba ĐƯỜNG NÓNG (AC12)
 *   Ctrl+Alt+Shift+5  đo lượt DỰNG lại cả Chương (AC13)
 */
(function () {
  'use strict'

  var B = {
    samples: [],        // {t, dt} — chỉ giữ delta, không giữ mốc tuyệt đối cho gọn
    inputs: [],         // mốc mọi lượt `input`
    flushes: [],        // {t, ms, n} — mọi lượt `save_segment_targets`
    sampling: false,
    startedAt: 0,
    rafHandle: 0,
    sampleEveryNth: 1,  // "nhịp lấy mẫu" của AC21 — đổi nó để xem frame max có đổi theo không
    tick: 0,
    hotpath: null,
    build: null,
    note: '',
  }
  window.__B = B

  // ── ① vá IPC để đóng dấu đường flush ────────────────────────────────────────────
  //
  // 🔵 VIẾT LẠI 2026-08-19 — BẢN CŨ KHÔNG BAO GIỜ CHẠY ĐƯỢC, VÀ ĐÓ LÀ GỐC RỄ CỦA CẢ
  // STORY 2.4. Bản cũ làm `I.invoke = function …`. Nhưng `tauri-2.11.5/scripts/core.js:81`
  // khai thuộc tính đó bằng `Object.defineProperty(window.__TAURI_INTERNALS__, 'invoke',
  // { value: … })` — chỉ có `value`, nên mặc định **`writable: false` VÀ
  // `configurable: false`**. Trong `'use strict'` (dòng ngay trên), một phép gán như thế
  // **NÉM `TypeError`**. Đây là lệnh cấp cao nhất ĐẦU TIÊN của tệp ⇒ nó giết cả IIFE
  // trước `title('BENCH_LOADED')`, trước dấu sống, trước mọi thứ.
  //
  // 🔴 Vì sao chuyện đó ẩn được suốt sáu ngày: hai kênh chẩn đoán đều mù.
  //   · `document.title` KHÔNG đổi tiêu đề cửa sổ native trong Tauri, nên `osascript` đọc
  //     ra `AuraTranslate` dù mã có chạy hay không — nó không phân biệt được hai ca.
  //   · Dấu sống đi qua IPC, mà lượt ném xảy ra TRƯỚC dấu sống ⇒ vắng mặt, và "vắng" bị
  //     đọc thành "lượt tiêm trượt". Bốn giả thuyết bị bác ngày 2026-08-13 (`strings`,
  //     CSP, cache `build.rs`, `document.title`) đều bị bác ĐÚNG — nhưng không cái nào là
  //     nguyên nhân, vì nguyên nhân nằm ở chính một dòng của bàn đo.
  //   Đo 2026-08-19 (`diag-seam.sh`): một lời gọi `put_config` đặt ngay cạnh, KHÔNG vá gì,
  //   ăn ngay lần đầu (`n=0`). Cùng lệnh, cùng `kind`, cách nhau vài micro giây — cái GỌI
  //   thì sống, cái GÁN ĐÈ thì chết.
  //
  // ⇒ `Proxy` cũng KHÔNG cứu được: bất biến của Proxy buộc bẫy `get` trả về ĐÚNG giá trị
  // gốc cho một thuộc tính không-ghi-được-không-cấu-hình-được. Đường duy nhất còn lại là
  // CHÉP mọi thuộc tính sang một đối tượng mới GHI ĐƯỢC rồi gán đè biến toàn cục — và
  // biến toàn cục thì gán được, `bundle.global.js:1` đặt nó bằng một phép gán thường.
  //
  // ⚠️ GIỚI HẠN THẬT: bản chép phải chở ĐỦ mọi thuộc tính Tauri đọc lúc chạy — `ipc`
  // (invoke gốc gọi `window.__TAURI_INTERNALS__.ipc` LÚC GỌI, không giữ tham chiếu),
  // `callbacks` · `runCallback` (Rust eval gọi ngược vào), `transformCallback` ·
  // `unregisterCallback` · `postMessage` · `convertFileSrc`. Vòng chép dưới đây chép
  // TOÀN BỘ `getOwnPropertyNames`, nên nó không phụ thuộc vào danh sách bảy cái tên đó —
  // danh sách chỉ để người đọc biết cái gì sẽ vỡ nếu ai đó thay vòng chép bằng một bản
  // chép tay.
  B.patchMode = 'chưa thử'
  function patchInvoke() {
    var I = window.__TAURI_INTERNALS__
    if (!I || typeof I.invoke !== 'function') return false
    if (I.__benchPatched) { B.patchMode = 'đã vá từ trước'; return true }

    var orig = I.invoke.bind(I)
    function wrapped(cmd, args, opts) {
      if (cmd !== 'save_segment_targets') return orig(cmd, args, opts)
      var t0 = performance.now()
      var n = (args && args.edits && args.edits.length) || 0
      var p = orig(cmd, args, opts)
      return Promise.resolve(p).then(
        function (r) { B.flushes.push({ t: t0, ms: performance.now() - t0, n: n, ok: 1 }); return r },
        function (e) { B.flushes.push({ t: t0, ms: performance.now() - t0, n: n, ok: 0 }); throw e }
      )
    }

    try {
      var shim = Object.create(Object.getPrototypeOf(I))
      var names = Object.getOwnPropertyNames(I)
      for (var i = 0; i < names.length; i++) {
        var k = names[i]
        var d = Object.getOwnPropertyDescriptor(I, k)
        if (d.get || d.set) { Object.defineProperty(shim, k, d); continue }
        // Giữ nguyên GIÁ TRỊ, chỉ mở `writable`/`configurable`. `callbacks` là một đối
        // tượng Tauri còn ghi vào — chép THAM CHIẾU nên danh tính giữ nguyên, đúng ý.
        Object.defineProperty(shim, k, {
          value: d.value, writable: true, enumerable: d.enumerable, configurable: true,
        })
      }
      shim.invoke = wrapped
      shim.__benchPatched = true
      window.__TAURI_INTERNALS__ = shim
    } catch (e) {
      B.patchMode = 'TRƯỢT khi chép: ' + String(e && (e.message || e)).slice(0, 80)
      return false
    }

    // 🔴 ĐẶT RỒI ĐỌC LẠI — luật của kho, và ở đây nó bắt đúng ca mà bản cũ đã trượt:
    // một phép gán im lặng không ăn sẽ cho `B.flushes` rỗng, và một bộ lọc nửa vời sẽ bị
    // đọc thành bộ lọc đủ.
    if (window.__TAURI_INTERNALS__ !== shim || window.__TAURI_INTERNALS__.invoke !== wrapped) {
      B.patchMode = 'TRƯỢT — đọc lại không khớp'
      return false
    }
    B.patchMode = 'bản chép ghi được'
    return true
  }
  // 🔴 BỌC CHỖ GỌI: dù `patchInvoke` có hỏng kiểu gì, bàn đo phải SỐNG TIẾP. Cả story này
  // treo sáu ngày vì một lượt ném ở đúng đây, và một bàn đo chết im lặng thì không đo được
  // gì mà cũng không nói được vì sao.
  try {
    if (!patchInvoke()) {
      var tries = 0
      var iv = setInterval(function () {
        var ok = false
        try { ok = patchInvoke() } catch (e) { B.patchMode = 'NÉM trong vòng thử lại' }
        if (ok || ++tries > 200) clearInterval(iv)
      }, 25)
    }
  } catch (e) {
    B.patchMode = 'NÉM ở lượt vá đầu: ' + String(e && (e.message || e)).slice(0, 80)
  }

  // ── ①b MỐC FLUSH QUA GIAO DIỆN — đường thay thế, thêm 2026-08-19 (Ice ký) ────────
  //
  // 🔴 VÌ SAO PHẢI CÓ ĐƯỜNG NÀY: lượt vá `invoke` ở trên KHÔNG BAO GIỜ ăn được trên Tauri
  // 2.11.5, và đó là một ràng buộc của NỀN TẢNG chứ không phải một lựa chọn của dự án.
  // `tauri-2.11.5/src/manager/webview.rs:173` khai chính biến toàn cục bằng
  // `Object.defineProperty(window, '__TAURI_INTERNALS__', { value: … })` — chỉ có `value`,
  // nên `writable: false` VÀ `configurable: false`. Cộng với `core.js:81` khoá luôn thuộc
  // tính `invoke`. ⇒ Bốn đường đều đóng: gán đè `invoke` · gán đè cả đối tượng · `Proxy`
  // *(bất biến Proxy buộc trả đúng giá trị gốc)* · `defineProperty` *(không cấu hình lại được)*.
  // Đo 2026-08-19: `patchMode` trả về *"TRƯỢT khi chép: Attempted to assign to readonly
  // property"* trên WKWebView thật.
  //
  // ⇒ Lấy mốc flush từ thứ DUY NHẤT quan sát được mà không chạm mã sản phẩm: chỉ báo
  // *"Đã lưu N giây trước"* ở `StatusBar.vue:285` (`footer.status > span.saved`).
  // `secondsSinceSave` là một `computed` trên `editorLastSavedAt` (`StatusBar.vue:85-89`),
  // nên một lượt flush hạ cánh làm Vue render lại NGAY — không chờ nhịp một giây.
  //
  // ⚠️ BA GIỚI HẠN THẬT, ghi ra thay vì để người sau tưởng nhầm:
  //   ① Nó đo lúc GIAO DIỆN đổi, muộn hơn lúc WAL nhận — cộng một lượt render. ⇒ Mọi mốc ở
  //      đây là CẬN TRÊN, cùng hạng với luật đã ghi ở `grid-table.sh`.
  //   ② Nó KHÔNG đo được ĐỘ DÀI một lượt flush, chỉ đo được THỜI ĐIỂM. `ms` luôn là 0, và
  //      `flushMsMax` vì thế vô nghĩa ở chế độ này — đừng trích nó.
  //   ③ Hai lượt flush cách nhau dưới một giây cho CÙNG một chuỗi *"0 giây trước"* ⇒ không
  //      sinh mutation ⇒ lượt thứ hai MẤT. Hợp đồng AD-35 để idle 2 s nên ca đó hiếm, nhưng
  //      nó có thật và nó làm `nFlushes` thành một CẬN DƯỚI.
  B.flushSource = 'chưa gắn'
  ;(function watchSaved(tries) {
    var bar = document.querySelector('footer.status')
    if (!bar) {
      if (tries < 240) setTimeout(function () { watchSaved(tries + 1) }, 250)
      else B.flushSource = 'KHÔNG THẤY footer.status'
      return
    }
    var prev = null
    var obs = new MutationObserver(function () {
      var el = bar.querySelector('span.saved')
      if (!el) { prev = null; return }
      var m = /(\d+)/.exec(el.textContent || '')
      if (!m) return
      var n = +m[1]
      // Mốc flush = lúc bộ đếm TỤT (một lượt lưu mới đặt lại nó), hoặc lúc nó xuất hiện lần
      // đầu. Bộ đếm TĂNG là nhịp một giây trôi, không phải một lượt lưu — bỏ qua.
      if (prev === null || n < prev) {
        B.flushes.push({ t: performance.now(), ms: 0, n: null, ok: 1, via: 'statusbar' })
      }
      prev = n
    })
    obs.observe(bar, { childList: true, subtree: true, characterData: true })
    B.flushSource = 'chỉ báo "Đã lưu" (cận trên)'
  })(0)

  // ── ② mốc mọi lượt `input` ──────────────────────────────────────────────────────
  // `capture: true` — nghe TRƯỚC handler của sản phẩm, nên mốc này không bị một
  // `stopPropagation` nào che.
  document.addEventListener('input', function () { B.inputs.push(performance.now()) }, true)

  // ── ③ vòng lấy mẫu rAF ──────────────────────────────────────────────────────────
  function loop(now) {
    if (!B.sampling) return
    B.tick++
    if (B.last !== undefined && B.tick % B.sampleEveryNth === 0) {
      B.samples.push({ t: Math.round(now), dt: +(now - B.last).toFixed(2) })
    }
    B.last = now
    B.rafHandle = requestAnimationFrame(loop)
  }

  function start(note) {
    B.samples = []; B.inputs = []; B.flushes = []
    B.last = undefined; B.tick = 0
    B.note = note || ''
    B.startedAt = Date.now()
    B.sampling = true
    B.rafHandle = requestAnimationFrame(loop)
    paint('ĐANG LẤY MẪU ' + (B.note || ''))
  }

  function stop() {
    B.sampling = false
    cancelAnimationFrame(B.rafHandle)
    paint('NGỪNG · ' + B.samples.length + ' mẫu')
  }

  /**
   * Bộ lọc BẮT BUỘC của Quyết định #3(a): một delta chỉ được tính là "frame" nếu trong cửa
   * sổ của nó app THẬT SỰ có việc — có một `input` hoặc một lượt flush.
   * 🔴 Số mẫu bị loại được TRẢ VỀ, không bị nuốt: một bộ lọc bỏ 90% mẫu là một phép đo hỏng.
   */
  function summarize() {
    var kept = [], dropped = 0
    for (var i = 0; i < B.samples.length; i++) {
      var s = B.samples[i], lo = s.t - s.dt, hi = s.t, busy = false
      for (var j = 0; j < B.inputs.length; j++) { if (B.inputs[j] >= lo && B.inputs[j] <= hi) { busy = true; break } }
      if (!busy) for (var k = 0; k < B.flushes.length; k++) {
        var f = B.flushes[k]
        if (f.t + f.ms >= lo && f.t <= hi) { busy = true; break }
      }
      if (busy) kept.push(s.dt); else dropped++
    }
    kept.sort(function (a, b) { return a - b })
    var all = B.samples.map(function (s) { return s.dt }).sort(function (a, b) { return a - b })
    function q(arr, p) { return arr.length ? arr[Math.min(arr.length - 1, Math.floor(arr.length * p))] : null }
    return {
      note: B.note,
      // 🔴 `patchMode` PHẢI đi kèm mọi bảng số, và đây là lý do đo được: bộ lọc bắt buộc
      // của Quyết định #3 có HAI vế — `input` và `flush`. Vế `flush` chỉ tồn tại khi lượt
      // vá `invoke` ăn. Nếu nó trượt, `nFlushes` bằng 0 và `busy` rơi về một bộ lọc NỬA
      // VỜI — mà một bảng số nửa vời trông y hệt một bảng số đủ. Suốt sáu ngày lượt vá đó
      // ném ngay dòng đầu và không ai biết. ⇒ Chế độ vá đi THEO số, không nằm trong đầu
      // người chạy. Đọc bảng nào thiếu trường này là đọc một bảng của bản bench cũ.
      patchMode: B.patchMode,
      // Nguồn của vế `flush` trong bộ lọc. 🔴 `flushMsMax` chỉ có nghĩa khi nguồn là lượt vá
      // `invoke`; với nguồn `statusbar` nó luôn 0 và `nFlushes` là một CẬN DƯỚI. Xem §①b.
      flushSource: B.flushSource,
      startedAt: B.startedAt,
      durationMs: B.samples.length ? Math.round(B.samples[B.samples.length - 1].t - B.samples[0].t) : 0,
      sampleEveryNth: B.sampleEveryNth,
      nSamples: B.samples.length,
      nInputs: B.inputs.length,
      nFlushes: B.flushes.length,
      flushMsMax: B.flushes.reduce(function (m, f) { return Math.max(m, f.ms) }, 0),
      // "trong lúc auto-save chạy" — số NGHIỆM THU của AC1
      busy: { n: kept.length, dropped: dropped, max: q(kept, 1), p99: q(kept, 0.99), p50: q(kept, 0.5), over50: kept.filter(function (d) { return d > 50 }).length },
      // mọi mẫu, kể cả lúc app nghỉ — số đối chiếu, KHÔNG phải số nghiệm thu
      all: { n: all.length, max: q(all, 1), p99: q(all, 0.99), p50: q(all, 0.5), over50: all.filter(function (d) { return d > 50 }).length },
      over50Detail: B.samples.filter(function (s) { return s.dt > 50 }).slice(0, 40),
      build: B.build,
      hotpath: B.hotpath,
    }
  }

  // ── ④ đường ĐỔ SỐ RA — `put_config`, một lệnh IPC ĐÃ CÓ ─────────────────────────
  function dump() {
    stop()
    var payload = JSON.stringify(summarize())
    return window.__TAURI_INTERNALS__
      .invoke('put_config', { kind: 'app_config', key: '__bench__', value: payload })
      .then(function () { paint('ĐÃ ĐỔ ' + payload.length + ' byte → global.db'); return true })
      .catch(function (e) { paint('ĐỔ TRƯỢT: ' + e); throw e })
  }

  // ── ⑤ ba ĐƯỜNG NÓNG (AC12) ──────────────────────────────────────────────────────
  //
  // 🔵 VIẾT LẠI 2026-08-18 — Sprint Change Proposal 2026-08-18c, Ice ký.
  //
  // 🔴 Bản cũ hỏi `.doc` và `.sent` và đo ba đường của `EditorPanel.vue`. Lượt correct-course
  // 2026-08-14 thay bề mặt đó bằng lưới `GridPanel.vue`. Đếm ngày 2026-08-18: `.doc` và `.sent`
  // có ĐÚNG 0 chỗ sống trong `src/`, `nearestSentenceTo` có 0 chỗ, `EditorPanel.vue` không tồn
  // tại. ⇒ Bản cũ trả về "KHÔNG THẤY .doc" — nó đo RỖNG, không đo ra một số xấu. Đó đúng lớp lỗi
  // "xanh rỗng" mà AC15 của Story 2.3 đặt tên và AC21 của chính story này cấm.
  //
  // Bàn đo KHÔNG gọi hàm nội bộ của component (nó không với tới được) — nó tái lập ĐÚNG phép
  // tính mà ba chỗ đó chạy, trên ĐÚNG cây DOM thật đang hiển thị.
  function hotpaths() {
    var grid = document.querySelector('.grid')
    var colTgt = document.querySelector('.col-tgt')
    if (!grid || !colTgt) { paint('KHÔNG THẤY .grid/.col-tgt'); return null }

    function med(f, reps) {
      var xs = []
      for (var i = 0; i < reps; i++) { var t = performance.now(); f(); xs.push(performance.now() - t) }
      xs.sort(function (a, b) { return a - b })
      return { median: +xs[Math.floor(reps / 2)].toFixed(3), max: +xs[reps - 1].toFixed(3), reps: reps }
    }

    var cells = colTgt.querySelectorAll('[data-segment-id]')
    var r = {
      nCells: cells.length,
      nNodes: grid.querySelectorAll('*').length,   // 2.5b đo 49.256 ở 9.850 câu — đối chiếu

      // ① `restoreEditedText()` (`GridPanel.vue:843`, watcher `:859`) — ĐƯỜNG DUY NHẤT sống sót
      // từ AC12 bản cũ. `querySelectorAll('[data-segment-id]')` trên CẢ Chương mỗi lượt dựng lại,
      // trong khi `editedText` thường chỉ mang vài mục.
      restoreEditedText: med(function () {
        var all = colTgt.querySelectorAll('[data-segment-id]')
        for (var i = 0; i < all.length; i++) { void all[i].getAttribute('data-segment-id') }
      }, 30),

      // ② `onSelectionChange()` → `setEditorCaret()` (`GridPanel.vue:875`, đăng ký `:885`) —
      // kế thừa trực tiếp của `:data-caret`. Nó ghi vào một ref phản ứng ĐỌC TRONG HÀM RENDER,
      // nên mỗi lượt `selectionchange` bắt Vue duyệt lại `v-for` trên NĂM cột.
      // ⚠️ Tái lập vế đắt: một lượt đọc + ghi lớp trên toàn danh sách của cả năm cột.
      // 2.5b đo 24–34 ms ở 9.850 câu.
      selectionRepaint: med(function () {
        var all = grid.querySelectorAll('.cell')
        for (var i = 0; i < all.length; i++) {
          all[i].classList.toggle('__bench_probe__')
          all[i].classList.toggle('__bench_probe__')
        }
      }, 10),

      // ③ DỜI CON TRỎ — `placeCaretAtPoint()` (`:459`) + `ensureCaretNextFrame()` (`:766`).
      // 🔴 ĐÂY LÀ ĐƯỜNG 2.5b ĐO 706–770 ms Ở 9.850 CÂU, vượt trần NFR2 ~15 lần, và là đường
      // THƯỜNG NHẤT của tính năng: mỗi lần người dùng bấm sang câu khác.
      // Tái lập: đặt vùng chọn thật vào một ô khác rồi ép bố cục chạy, đúng như đường sản phẩm.
      caretMove: med(function () {
        var i = (Math.random() * cells.length) | 0
        var cell = cells[i]
        var sel = window.getSelection()
        var rg = document.createRange()
        rg.selectNodeContents(cell)
        rg.collapse(true)
        sel.removeAllRanges()
        sel.addRange(rg)
        void grid.offsetHeight            // ép bố cục, không để nó trôi sang frame sau
      }, 10),
    }
    B.hotpath = r
    paint('ĐƯỜNG NÓNG xong · ' + r.nCells + ' ô · ' + r.nNodes + ' node')
    return r
  }

  // ── ⑥ lượt DỰNG lại cả Chương (AC13) ───────────────────────────────────────────
  // Đo cửa sổ "lúc dựng Chương" — thứ Quyết định #3 tách RIÊNG khỏi số nghiệm thu AC1, và lượt
  // mở rộng AC1 ngày 2026-08-18 KHÔNG đụng vế đó.
  //
  // 🔴 Số này KHÔNG được đặt cạnh mốc cũ (300,1 ms Blink · 1.308,0 ms WebKit). Mốc đó đo "dựng
  // 9.850 <span> trong một dòng văn liên tục"; lưới dựng ~49.256 node trong năm cột subgrid.
  // `deferred-work.md:3258`: ghi hai số đó cạnh nhau như một lượt "cải thiện" LÀ NÓI DỐI.
  function measureRebuild() {
    var grid = document.querySelector('.grid')
    if (!grid) { paint('KHÔNG THẤY .grid'); return null }
    var html = grid.innerHTML
    var t0 = performance.now()
    grid.innerHTML = ''
    grid.innerHTML = html
    void grid.offsetHeight              // ép bố cục chạy, không để nó trôi sang frame sau
    var ms = performance.now() - t0
    B.build = {
      rebuildMs: +ms.toFixed(1),
      nCells: grid.querySelectorAll('.cell-tgt').length,
      nNodes: grid.querySelectorAll('*').length,
    }
    paint('DỰNG LẠI ' + B.build.rebuildMs + ' ms · ' + B.build.nNodes + ' node')
    return B.build
  }

  // ── ⑦ dựng Tác phẩm từ tệp cố định ─────────────────────────────────────────────
  var FIXED = '/tmp/at-bench-chapter.txt'
  function setup() {
    paint('đang dựng Tác phẩm…')
    var I = window.__TAURI_INTERNALS__
    return I.invoke('create_work_from_file', {
      name: 'bench-' + Date.now(), sourceLang: 'vi', genre: 'other', path: FIXED,
    })
      .then(function (w) {
        paint('Tác phẩm xong, đang tách segment…')
        return I.invoke('split_chapter_into_segments', { chapterId: w.chapterId })
      })
      .then(function (o) { paint('tách xong: ' + JSON.stringify(o) + ' — nạp lại trang…'); setTimeout(function () { location.reload() }, 800) })
      .catch(function (e) { paint('DỰNG TRƯỢT: ' + e) })
  }

  // ── ⑦b TOẠ ĐỘ THẬT CỦA MỘT Ô — thay phép ĐOÁN bằng phép ĐO, thêm 2026-08-19 ──────
  //
  // 🔴 VÌ SAO: `focus-segment.sh` dò 16 toạ độ viết cứng, và đo được 2026-08-19 nó chỉ vào
  // được **3 trên 7 lượt** — cùng script, cùng thang, cùng hình học cửa sổ. Chính đầu tệp đó
  // đã ghi *"vùng trúng không ổn định giữa các lượt"* từ 2026-08-13, nhưng cách chữa lúc ấy
  // là THÊM ứng viên, tức vẫn đoán, chỉ đoán nhiều hơn. Một phiên 30 phút × n=3 đứng trên
  // một cửa vào 50% là một lượt đo sẽ hỏng giữa chừng.
  //
  // ⇒ Bench nay sống trong bản dựng, nên nó HỎI ĐƯỢC cây DOM thật. Nó trả về hình chữ nhật
  // của một ô đích đang hiện, cộng gốc cửa sổ trên màn hình. Shell bấm đúng tâm ô đó — vẫn
  // bằng `cliclick`, tức vẫn là một `mousedown` THẬT, không phải một sự kiện tổng hợp.
  // Đây là "đo trước khi tin" áp cho toạ độ, thay cho "thử mười sáu chỗ".
  //
  // ⚠️ `window.screenX/screenY` là gốc của VÙNG NỘI DUNG webview trên màn hình, nên nó đã
  // gồm cả thanh tiêu đề — không phải cộng thêm một hằng hiệu chỉnh nào. Đó là lý do đường
  // này không cần hiệu chuẩn lại mỗi khi bề mặt đổi, khác hẳn bộ 16 ứng viên.
  //
  // ⚠️ GIỚI HẠN: chỉ chạy được trên bản dựng CÓ tiêm bench. `kill-campaign-v2.sh` chạy trên
  // bản release trần, nên bộ 16 ứng viên PHẢI ở lại làm đường lùi — đừng gỡ nó.
  // 🔴 TOẠ ĐỘ TRONG KHUNG NHÌN, KHÔNG PHẢI TOẠ ĐỘ MÀN HÌNH — sửa 2026-08-19.
  // Bản đầu dùng `window.screenX/screenY` và nó trả về một hệ toạ độ KHÁC hệ của `cliclick`:
  // đo được, tab Workspace ra `(102, 980)` trong khi cửa sổ nằm ở `(200, 25)` cỡ `1200×900`,
  // tức tab phải ở quanh `(284, 66)`. Máy này có HAI màn hình cạnh nhau *(desktop trải
  // `0,0,3456,1080`)*, và trong cấu hình đó `screenX/screenY` của WKWebView không dùng được.
  // ⇒ Bench chỉ báo thứ nó biết chắc: toạ độ trong KHUNG NHÌN, cộng kích thước khung nhìn.
  // Phép quy về màn hình do shell làm, từ gốc cửa sổ mà chính nó đã đặt-rồi-đọc-lại.
  function viewportCenter(node) {
    if (!node) return null
    var r = node.getBoundingClientRect()
    if (r.width < 4 || r.height < 4) return null
    return Math.round(r.left + r.width / 2) + ',' + Math.round(r.top + r.height / 2)
  }
  function viewport() { return window.innerWidth + ',' + window.innerHeight }

  // 🔴 TOẠ ĐỘ TAB WORKSPACE — thêm 2026-08-19, và nó là chỗ hỏng THẬT.
  // Ảnh chụp `fail-focus-keylog.png` cho thấy: app đứng ở màn **Library**, và 16 ứng viên mù
  // của `focus-segment.sh` đã gõ ký tự dò `x` thẳng vào form tạo Tác phẩm (ô "Tên Tác phẩm"
  // thành `xx`, ô "Dán văn bản" thành `xxx`). Tức cú bấm mở tab Workspace ở
  // `nfr2-session.sh` — một toạ độ viết cứng `(+101,+46)` — mới là thứ chập chờn, KHÔNG phải
  // toạ độ ô. Cả lượt "3/7 vào được" trước đây là cùng một nguyên nhân này.
  // ⇒ Đo nốt toạ độ cuối cùng còn đoán. `nav.modes > button.mode-tab`, phần tử thứ HAI
  // (`App.vue:190-197`).
  // Khuôn giá trị: `vx,vy,iw,ih` — hai số đầu là tâm phần tử trong khung nhìn, hai số sau là
  // cỡ khung nhìn. Shell quy ra màn hình: x = WIN_X + (WIN_W − iw) + vx · y = WIN_Y + (WIN_H − ih) + vy.
  function reportTabRect() {
    var tabs = document.querySelectorAll('nav.modes button.mode-tab')
    var c = viewportCenter(tabs[1])
    var v = c ? c + ',' + viewport() : 'NONE'
    return window.__TAURI_INTERNALS__.invoke('put_config', {
      kind: 'app_config', key: '__tabrect__', value: v,
    })
  }

  function reportCellRect() {
    reportTabRect()
    // 🔵 Sửa 2026-08-19, cùng ngày với lượt viết ra: bản đầu đòi ô nằm TRỌN trong khung nhìn
    // (`r.bottom <= innerHeight`) và nó trả `NONE` ngay lượt chạy thứ hai. Lý do đo được từ
    // ảnh chụp `fail-dbg.png`: ô đang gõ CAO GẦN HẾT cửa sổ, nên không ô nào thoả. Một điều
    // kiện đúng-về-lý-thuyết mà rỗng-trong-thực-tế là đúng lớp lỗi kho này gọi là "rỗng im
    // lặng" — nó không sai, nó chỉ không bao giờ trả lời.
    // ⇒ Lấy PHẦN NHÌN THẤY ĐƯỢC của ô, và bấm vào tâm phần đó.
    var cells = document.querySelectorAll('.cell-tgt')
    var pick = null
    var vw = window.innerWidth, vh = window.innerHeight
    for (var i = 0; i < cells.length; i++) {
      var r = cells[i].getBoundingClientRect()
      var top = Math.max(r.top, 0), bot = Math.min(r.bottom, vh)
      var lef = Math.max(r.left, 0), rig = Math.min(r.right, vw)
      // Đủ chỗ để một cú bấm chắc chắn rơi vào trong, kể cả khi lưới cuộn vài pixel giữa
      // lúc đo và lúc bấm.
      if (rig - lef > 16 && bot - top > 16) { pick = { cx: (lef + rig) / 2, cy: (top + bot) / 2 }; break }
    }
    if (!pick) {
      paint('KHÔNG THẤY ô .cell-tgt nào đang hiện')
      return window.__TAURI_INTERNALS__.invoke('put_config', {
        kind: 'app_config', key: '__cellrect__', value: 'NONE',
      })
    }
    // Cột thứ SÁU là số phím nóng bench đã nhận từ đầu phiên. 🔴 Nó là một CỔNG SỐNG, không
    // một con số trang trí: shell đọc nó, gửi thêm một phím, rồi chờ nó TĂNG — đó là bằng
    // chứng "app đang nhận phím" ngay lúc này. Đo được 2026-08-19: sau một phiên gõ 30 s,
    // phím đổ số phải bắn **2 lượt** ở phiên này và **5 lượt** ở phiên kế *(trần là 5 — sát
    // mép)*. Một vòng thử lại mù không phân biệt được "app chưa nhận" với "app không bao giờ
    // nhận"; cổng này thì phân biệt được, và nó thay một phép đoán bằng một phép đo.
    var v = [Math.round(pick.cx), Math.round(pick.cy), vw, vh, cells.length, B.keyLog.length].join(',')
    paint('ô đích ở màn hình: ' + v)
    return window.__TAURI_INTERNALS__.invoke('put_config', {
      kind: 'app_config', key: '__cellrect__', value: v,
    })
  }

  // ── ⑧ ô hiển thị trạng thái — để chụp màn hình xác nhận từ ngoài ────────────────
  // 🔵 2026-08-19 — ô này nay chở THÊM nhật ký phím nóng, và đây là lý do đo được.
  // Phiên 20 s: `key 23` (AC13) và `key 20` (đổ số) đều tới nơi. Phiên 30 s và 60 s: cả hai
  // KHÔNG tới — ảnh chụp lúc hỏng cho thấy ô này vẫn ghi *"ĐANG LẤY MẪU gõ"*, tức
  // `measureRebuild()` và `dump()` chưa từng chạy *(cả hai đều gọi `paint` ở nhánh đầu tiên)*.
  // Bốn phím TRƯỚC lượt gõ đều tới; hai phím SAU lượt gõ thì không. Biến phân biệt là THỜI
  // LƯỢNG GÕ, không phải mã phím.
  // ⇒ Đừng đoán tiếp: ghi ra ĐÚNG những phím nóng bench nhận được, rồi đọc bằng ảnh chụp.
  // Ô này là kênh chẩn đoán duy nhất sống sót được một lượt `dump()` hỏng.
  var el, lastMsg = ''
  B.keyLog = []
  function paint(msg) {
    if (!el) {
      el = document.createElement('div')
      el.id = '__bench__'
      el.style.cssText = 'position:fixed;z-index:999999;right:6px;bottom:6px;background:#111;color:#0f0;' +
        'font:11px ui-monospace,monospace;padding:4px 7px;border-radius:4px;pointer-events:none;max-width:46vw'
      document.body.appendChild(el)
    }
    if (msg !== null && msg !== undefined) lastMsg = msg
    el.textContent = '⟦B⟧ ' + lastMsg + '  ⌨[' + B.keyLog.join(',') + ']'
  }

  // ── ⑨ phím điều khiển ──────────────────────────────────────────────────────────
  window.addEventListener('keydown', function (e) {
    if (!(e.ctrlKey && e.altKey && e.shiftKey)) return
    var k = e.code
    // Ghi TRƯỚC lượt điều phối: một phím tới nơi rồi chết trong handler vẫn phải để lại dấu.
    B.keyLog.push(String(k).replace('Digit', ''))
    paint(null)
    if (k === 'Digit1') { e.preventDefault(); setup() }
    else if (k === 'Digit2') { e.preventDefault(); start('gõ') }
    else if (k === 'Digit3') { e.preventDefault(); dump() }
    else if (k === 'Digit4') { e.preventDefault(); hotpaths() }
    else if (k === 'Digit5') { e.preventDefault(); measureRebuild() }
    else if (k === 'Digit6') { e.preventDefault(); B.sampleEveryNth = B.sampleEveryNth === 1 ? 3 : 1; paint('nhịp lấy mẫu = mỗi ' + B.sampleEveryNth) }
    else if (k === 'Digit7') { e.preventDefault(); reportCellRect() }
  }, true)

  window.addEventListener('DOMContentLoaded', function () { paint('sẵn sàng') })

  // ── ⑩ DẤU SỐNG — cổng nghiệm thu BẰNG MÁY của lượt tiêm ────────────────────────
  // `strings` trên nhị phân là một cổng ÂM TÍNH GIẢ: `strip = true` và Tauri nén tài
  // nguyên nhúng, nên không tìm thấy chuỗi KHÔNG chứng minh được lượt tiêm trượt.
  // Dấu sống này thì chứng minh được: nếu nó có trong `global.db` thì bench ĐÃ chạy
  // trong webview thật của bản release.
  // 🔴 Kênh chẩn đoán KHÔNG đi qua IPC: `document.title`. Đọc được từ ngoài bằng
  // `osascript` (System Events → title of front window), nên nó trả lời được câu hỏi
  // "mã có chạy không" ĐỘC LẬP với câu hỏi "put_config có ghi được không".
  // Lượt trước hai câu hỏi đó bị bó làm một và `.catch(function(){})` NUỐT vế thứ hai —
  // đúng antipattern mà chính kho này cấm.
  function title(s) { try { document.title = s } catch (e) {} }
  title('BENCH_LOADED')

  ;(function alive(tries) {
    var I = window.__TAURI_INTERNALS__
    if (!I || typeof I.invoke !== 'function') {
      title('BENCH_NO_IPC_' + tries)
      if (tries < 240) setTimeout(function () { alive(tries + 1) }, 250)
      return
    }
    I.invoke('put_config', { kind: 'app_config', key: '__bench_alive__', value: String(Date.now()) })
      .then(function () { title('BENCH_OK') })
      .catch(function (e) {
        // KHÔNG nuốt: ghi nguyên văn ra title rồi THỬ LẠI. Một lượt ghi trượt lúc khởi
        // động (kho chưa được `manage`) là giả thuyết hàng đầu, và nó tự lành sau vài trăm ms.
        title('BENCH_PUT_ERR_' + tries + '_' + String(e && (e.message || e.code || e)).slice(0, 60))
        if (tries < 240) setTimeout(function () { alive(tries + 1) }, 250)
      })
  })(0)
})()
