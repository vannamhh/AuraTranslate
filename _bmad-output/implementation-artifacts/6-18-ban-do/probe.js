/* Bàn đo Story 6.18 — __nfr_bench_alive_6_18__ — nối vào ĐUÔI bundle production, không vào
 * mã sản phẩm. Cùng khuôn 5-14-ban-do/probe.js, chỉ đổi hình dạng "usable": grid phải mang
 * ĐỦ 50 Tác phẩm (không phải 1), và Tác phẩm mà `nfr_bench` server-side sẽ mở qua
 * `AURA_NFR_BENCH_WORK_NAME` là "NFR Story 6.18 Work 00" (Tác phẩm tên nhỏ nhất theo thứ tự
 * từ điển — cùng phép chọn `min_by(name)` mà `story_6_18_library.rs` và
 * `story_6_18_bench_transition.rs` dùng cho hình dạng frontier). */
;(function () {
  'use strict'

  var invoke = window.__TAURI_INTERNALS__ && window.__TAURI_INTERNALS__.invoke
  var EXPECTED_WORKS = 50
  var TARGET_WORK_NAME = 'NFR Story 6.18 Work 00'

  function delay(ms) {
    return new Promise(function (resolve) { setTimeout(resolve, ms) })
  }

  function markAndWait(key, value, after) {
    if (typeof invoke !== 'function') return Promise.reject(new Error('TAURI invoke vắng mặt'))
    return invoke('nfr_bench_mark_and_wait_phase', {
      marker: key,
      value: JSON.stringify(value),
      after: after,
    }).then(function (phase) {
      return { phase: phase }
    })
  }

  async function waitFor(label, read, timeoutMs) {
    var started = performance.now()
    while (performance.now() - started < timeoutMs) {
      var value = read()
      if (value) return value
      await delay(25)
    }
    throw new Error('timeout ' + label + ' sau ' + timeoutMs + ' ms')
  }

  async function main() {
    // `.works-block` chỉ chứng minh component đã mount; usable thật đòi grid có ĐỦ 50 Tác
    // phẩm THẬT, kể cả Tác phẩm đích mà pha Reading sẽ mở.
    var usable = await waitFor('Library grid có đủ 50 Tác phẩm Story 6.18', function () {
      var grid = document.querySelector('[data-library-grid]')
      var cells = grid ? grid.querySelectorAll('[data-library-work-cell]') : []
      if (cells.length !== EXPECTED_WORKS) return null
      var hasTarget = false
      for (var i = 0; i < cells.length; i++) {
        var name = cells[i].querySelector('.work-name')
        if (name && name.textContent.trim() === TARGET_WORK_NAME) { hasTarget = true; break }
      }
      if (!hasTarget) return null
      return { works: cells.length, name: TARGET_WORK_NAME }
    }, 180000)

    // Sau marker này command feature giữ state machine native. Native `eval` không nhìn thấy
    // closure/global của bundle trong world riêng, nên probe dừng ở đây;
    // driver native vẫn click UI thật và chỉ ghi marker khi DOM thật thỏa điều kiện. Số lớp
    // từ điển đã nạp (§Always spec 6.18) được server-side thêm vào marker này, không phải
    // probe — xem `lib.rs::nfr_bench`, nhánh `"usable"`.
    await markAndWait('usable', {
      epoch_ms: Date.now(),
      performance_epoch_ms: performance.timeOrigin + performance.now(),
      works: usable.works,
      work_name: usable.name,
    }, 'library')
  }

  main().catch(function (error) {
    var detail = String(error) + '\n' + String(error && error.stack ? error.stack : '')
    markAndWait('invalid', { epoch_ms: Date.now(), detail: detail }, 'done').catch(function () {})
  })
})()
