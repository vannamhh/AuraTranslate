/* Bàn đo Story 5.14 — được nối vào ĐUÔI bundle production, không vào mã sản phẩm. */
;(function () {
  'use strict'

  var invoke = window.__TAURI_INTERNALS__ && window.__TAURI_INTERNALS__.invoke

  function delay(ms) {
    return new Promise(function (resolve) { setTimeout(resolve, ms) })
  }

  function markAndWait(key, value, after) {
    if (typeof invoke !== 'function') return Promise.reject(new Error('TAURI invoke vắng mặt'))
    return invoke('story_5_14_mark_and_wait_phase', {
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
    // `.works-block` chỉ chứng minh component đã mount; usable thật đòi grid có DỮ LIỆU.
    var usable = await waitFor('Library grid có đúng fixture', function () {
      var grid = document.querySelector('[data-library-grid]')
      var cells = grid ? grid.querySelectorAll('[data-library-work-cell]') : []
      if (cells.length !== 1) return null
      var name = cells[0].querySelector('.work-name')
      if (!name || name.textContent.trim() !== '5.14 Fixture') return null
      return { works: cells.length, name: name.textContent.trim() }
    }, 180000)

    // Sau marker này command feature giữ state machine native. Native `eval` không nhìn thấy
    // closure/global của bundle trong world riêng, nên probe dừng ở đây;
    // driver native vẫn click UI thật và chỉ ghi marker khi DOM thật thỏa điều kiện.
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
