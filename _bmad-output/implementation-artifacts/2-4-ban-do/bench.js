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
  // Vá SỚM: bundle của app import `invoke` từ @tauri-apps/api, mà hàm đó gọi
  // `window.__TAURI_INTERNALS__.invoke` LÚC CHẠY. Nên vá cái global là đủ và không cần
  // chạm một dòng mã sản phẩm nào.
  function patchInvoke() {
    var I = window.__TAURI_INTERNALS__
    if (!I || typeof I.invoke !== 'function' || I.__benchPatched) return false
    var orig = I.invoke.bind(I)
    I.invoke = function (cmd, args, opts) {
      if (cmd !== 'save_segment_targets') return orig(cmd, args, opts)
      var t0 = performance.now()
      var n = (args && args.edits && args.edits.length) || 0
      var p = orig(cmd, args, opts)
      return Promise.resolve(p).then(
        function (r) { B.flushes.push({ t: t0, ms: performance.now() - t0, n: n, ok: 1 }); return r },
        function (e) { B.flushes.push({ t: t0, ms: performance.now() - t0, n: n, ok: 0 }); throw e }
      )
    }
    I.__benchPatched = true
    return true
  }
  if (!patchInvoke()) {
    var tries = 0
    var iv = setInterval(function () { if (patchInvoke() || ++tries > 200) clearInterval(iv) }, 25)
  }

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
  // Đo CHI PHÍ MỘT LƯỢT của đúng ba chỗ §Điều kiện khởi hành mục 6 gọi tên. Bàn đo KHÔNG
  // gọi hàm nội bộ của component (nó không với tới được) — nó tái lập ĐÚNG phép tính mà
  // ba chỗ đó chạy, trên ĐÚNG cây DOM thật đang hiển thị.
  function hotpaths() {
    var doc = document.querySelector('.doc')
    if (!doc) { paint('KHÔNG THẤY .doc'); return null }
    function med(f, reps) {
      var xs = []
      for (var i = 0; i < reps; i++) { var t = performance.now(); f(); xs.push(performance.now() - t) }
      xs.sort(function (a, b) { return a - b })
      return { median: +xs[Math.floor(reps / 2)].toFixed(3), max: +xs[reps - 1].toFixed(3), reps: reps }
    }
    var spans = doc.querySelectorAll('[data-segment-id]')
    var r = {
      nSpans: spans.length,
      // `restoreEditedText()` — querySelectorAll trên CẢ Chương mỗi lượt dựng lại
      restoreEditedText: med(function () {
        var all = doc.querySelectorAll('[data-segment-id]')
        for (var i = 0; i < all.length; i++) { void all[i].getAttribute('data-segment-id') }
      }, 30),
      // `nearestSentenceTo()` — duyệt TỪNG câu, gọi getClientRects()/getBoundingClientRect()
      nearestSentenceTo: med(function () {
        var all = doc.querySelectorAll('.sent')
        var best = null, bd = Infinity
        for (var i = 0; i < all.length; i++) {
          var rects = all[i].getClientRects()
          for (var j = 0; j < rects.length; j++) {
            var d = Math.abs(rects[j].top - 300) + Math.abs(rects[j].left - 300)
            if (d < bd) { bd = d; best = all[i] }
          }
        }
        return best
      }, 10),
      // `:data-caret` — mỗi `selectionchange` bắt Vue duyệt lại v-for trên tới N span.
      // Tái lập vế ĐẮT: một lượt đọc thuộc tính + ghi lại trên toàn danh sách.
      dataCaretRepaint: med(function () {
        var all = doc.querySelectorAll('.sent')
        for (var i = 0; i < all.length; i++) {
          var v = all[i].getAttribute('data-caret')
          all[i].setAttribute('data-caret', v === 'true' ? 'false' : 'false')
        }
      }, 10),
    }
    B.hotpath = r
    paint('ĐƯỜNG NÓNG xong · ' + r.nSpans + ' span')
    return r
  }

  // ── ⑥ lượt DỰNG lại cả Chương (AC13) ───────────────────────────────────────────
  // Đo cửa sổ "lúc dựng Chương" — thứ Quyết định #3 tách RIÊNG khỏi số nghiệm thu AC1.
  function measureRebuild() {
    var doc = document.querySelector('.doc')
    if (!doc) { paint('KHÔNG THẤY .doc'); return null }
    var html = doc.innerHTML
    var t0 = performance.now()
    doc.innerHTML = ''
    doc.innerHTML = html
    void doc.offsetHeight               // ép bố cục chạy, không để nó trôi sang frame sau
    var ms = performance.now() - t0
    B.build = { rebuildMs: +ms.toFixed(1), nSpans: doc.querySelectorAll('.sent').length }
    paint('DỰNG LẠI ' + B.build.rebuildMs + ' ms · ' + B.build.nSpans + ' span')
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

  // ── ⑧ ô hiển thị trạng thái — để chụp màn hình xác nhận từ ngoài ────────────────
  var el
  function paint(msg) {
    if (!el) {
      el = document.createElement('div')
      el.id = '__bench__'
      el.style.cssText = 'position:fixed;z-index:999999;right:6px;bottom:6px;background:#111;color:#0f0;' +
        'font:11px ui-monospace,monospace;padding:4px 7px;border-radius:4px;pointer-events:none;max-width:46vw'
      document.body.appendChild(el)
    }
    el.textContent = '⟦B⟧ ' + msg
  }

  // ── ⑨ phím điều khiển ──────────────────────────────────────────────────────────
  window.addEventListener('keydown', function (e) {
    if (!(e.ctrlKey && e.altKey && e.shiftKey)) return
    var k = e.code
    if (k === 'Digit1') { e.preventDefault(); setup() }
    else if (k === 'Digit2') { e.preventDefault(); start('gõ') }
    else if (k === 'Digit3') { e.preventDefault(); dump() }
    else if (k === 'Digit4') { e.preventDefault(); hotpaths() }
    else if (k === 'Digit5') { e.preventDefault(); measureRebuild() }
    else if (k === 'Digit6') { e.preventDefault(); B.sampleEveryNth = B.sampleEveryNth === 1 ? 3 : 1; paint('nhịp lấy mẫu = mỗi ' + B.sampleEveryNth) }
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
