/* __nfr_bench_alive_e7_r4__ -- appended to the tail of the production bundle by build.sh. */
;(function () {
  'use strict'

  var invoke = window.__TAURI_INTERNALS__ && window.__TAURI_INTERNALS__.invoke
  var WORK_NAME = 'E7 R4 Work'

  function delay(ms) {
    return new Promise(function (resolve) { setTimeout(resolve, ms) })
  }

  function call(name, args) {
    if (typeof invoke !== 'function') return Promise.reject(new Error('TAURI invoke absent'))
    return invoke(name, args || {})
  }

  async function waitFor(label, read, timeoutMs) {
    var started = performance.now()
    while (performance.now() - started < timeoutMs) {
      var value = read()
      if (value) return value
      await delay(25)
    }
    throw new Error('timeout ' + label + ' after ' + timeoutMs + ' ms')
  }

  // The measurement itself runs in Rust (nfr_bench_e7_start) through the real wire shells: driving
  // 54+ IPC round trips from the webview stalled mid-run with both sides idle.
  async function main() {
    await waitFor('Library grid with the one E7 R4 Work', function () {
      var grid = document.querySelector('[data-library-grid]')
      var cells = grid ? grid.querySelectorAll('[data-library-work-cell]') : []
      if (cells.length !== 1) return null
      var name = cells[0].querySelector('.work-name')
      return name && name.textContent.trim() === WORK_NAME ? true : null
    }, 180000)

    await call('nfr_bench_mark_and_wait_phase', {
      marker: 'usable',
      value: JSON.stringify({ epoch_ms: Date.now(), works: 1, work_name: WORK_NAME }),
      after: 'library',
    })
    await call('nfr_bench_e7_start')
  }

  main().catch(function (error) {
    var detail = String(error) + '\n' + String(error && error.stack ? error.stack : '')
    call('nfr_bench_e7_put_marker', {
      marker: 'e7_probe_error',
      value: JSON.stringify({ detail: detail.slice(0, 4000) }),
    }).catch(function () {})
  })
})()
