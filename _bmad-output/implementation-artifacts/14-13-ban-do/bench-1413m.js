/* eslint-disable -- bench appended to the built bundle, never part of the product */
;(function () {
  'use strict'
  var B = { ready: null, sweeps: [], err: null }
  window.__B1413M = B
  var I = function () { return window.__TAURI_INTERNALS__ }
  function put(key, value) {
    return I().invoke('put_config', { kind: 'app_config', key: key, value: String(value) })
  }
  function create() {
    return I().invoke('create_work_from_text', { name: 'b1413', sourceLang: 'zh', genre: 'general', text: '句子。'.repeat(9850) })
      .then(function () { setTimeout(function () { location.reload() }, 500) })
      .catch(function (e) { put('__b1413m_err__', 'create ' + e) })
  }
  function frame() { return new Promise(function (r) { requestAnimationFrame(function () { r() }) }) }
  function openAndSettle() {
    var tabs = document.querySelectorAll('nav.modes button.mode-tab')
    var t0 = performance.now()
    if (tabs[1]) tabs[1].click()
    ;(function poll() {
      var n = document.querySelectorAll('[data-num-segment-id]').length
      if (n >= 9850) return settle(n)
      if (performance.now() - t0 > 150000) return put('__b1413m_ready__', 'NOROWS ' + n)
      requestAnimationFrame(poll)
    })()
    function settle(n) {
      var good = 0, last = performance.now(), s0 = last
      ;(function tick() {
        var now = performance.now()
        good = now - last < 40 ? good + 1 : 0
        last = now
        if (good >= 60) { B.ready = n; return put('__b1413m_ready__', n) }
        if (now - s0 > 120000) return put('__b1413m_ready__', 'NOQUIET')
        requestAnimationFrame(tick)
      })()
    }
  }
  var P = { downs: 0, last: null, moves: 0 }
  window.addEventListener('mousedown', function (e) { P.downs++; P.last = [e.clientX, e.clientY, e.shiftKey, e.target && e.target.closest && (e.target.closest('[data-num-segment-id]') || {getAttribute: function () { return null }}).getAttribute('data-num-segment-id')] }, true)
  function report() {
    var nums = Array.prototype.slice.call(document.querySelectorAll('[data-num-segment-id]'))
    var rows = []
    for (var i = 0; i < nums.length; i++) {
      var r = nums[i].getBoundingClientRect()
      if (r.bottom > 0 && r.top < window.innerHeight) rows.push({ i: i, id: nums[i].getAttribute('data-num-segment-id'), x: r.x + Math.min(r.width / 2, 24), y: r.y + r.height / 2 })
    }
    var sel = []
    for (var k = 0; k < nums.length; k++) if (nums[k].classList.contains('row-selected')) sel.push(k)
    var cells = document.querySelectorAll('.row-selected').length
    put('__b1413m__', JSON.stringify({ win: { x: window.screenX, y: window.screenY, oh: window.outerHeight, ih: window.innerHeight }, downs: P.downs, last: P.last, total: nums.length, selectedIdx: sel, selectedCells: cells, visible: rows }))
  }
  document.addEventListener('keydown', function (e) {
    if (e.ctrlKey && e.altKey && e.shiftKey) {
      var k = e.code
      if (k === 'KeyW') { e.preventDefault(); create() }
      else if (k === 'KeyE') { e.preventDefault(); openAndSettle() }
      else if (k === 'KeyM') { e.preventDefault(); report() }
    }
  }, true)
  ;(function alive(n) {
    if (!I() || typeof I().invoke !== 'function') { if (n < 240) setTimeout(function () { alive(n + 1) }, 250); return }
    put('__bench_alive__', Date.now()).catch(function () { if (n < 240) setTimeout(function () { alive(n + 1) }, 250) })
  })(0)
})()
