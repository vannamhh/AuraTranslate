/* eslint-disable */
;(function () {
  'use strict'
  var B = { keys: [], confirm: null, row: -1, nCells: 0 }
  window.__B215 = B
  var I = function () { return window.__TAURI_INTERNALS__ }
  function put(key, value) {
    return I().invoke('put_config', { kind: 'app_config', key: key, value: String(value) })
  }
  function text(n) {
    var p = []
    for (var i = 0; i < n; i++) p.push('第' + (i + 1) + '句话，用来量渲染成本。')
    return p.join('')
  }
  function create(n) {
    return I().invoke('create_work_from_text', { name: 'b215-' + n, sourceLang: 'zh', genre: 'general', text: text(n) })
      .then(function () { setTimeout(function () { location.reload() }, 500) })
      .catch(function (e) { put('__b215_err__', 'create ' + e) })
  }
  function openAndFocus() {
    var tabs = document.querySelectorAll('nav.modes button.mode-tab')
    if (tabs[1]) tabs[1].click()
    var tries = 0
    ;(function wait() {
      var cells = document.querySelectorAll('.cell-tgt')
      if (!cells.length) { if (++tries < 600) return setTimeout(wait, 250); return put('__b215_ready__', 'NOCELLS') }
      setTimeout(function () {
        var c = cells[0]
        c.focus()
        var r = document.createRange(); r.selectNodeContents(c); r.collapse(false)
        var s = getSelection(); s.removeAllRanges(); s.addRange(r)
        B.nCells = cells.length
        B.row = 0
        put('__b215_ready__', cells.length + ',' + (document.activeElement === c) + ',' + (document.querySelectorAll('.cell-rule').length))
      }, 1500)
    })()
  }
  function dumpNow() {
    put('__b215__', JSON.stringify({ ua: navigator.userAgent, nCells: B.nCells, keys: B.keys, confirm: B.confirm, origin: performance.timeOrigin }))
  }
  document.addEventListener('input', function (e) {
    var rec = { ts: e.timeStamp, h: performance.now(), f: null }
    B.keys.push(rec)
    requestAnimationFrame(function () { requestAnimationFrame(function () { rec.f = performance.now() }) })
  }, true)
  document.addEventListener('keydown', function (e) {
    if (e.metaKey && e.key === 'Enter' && !B.confirm) {
      var c = { ts: e.timeStamp, h: performance.now(), f: null }
      B.confirm = c
      var t0 = performance.now()
      setTimeout(dumpNow, 20000)
      ;(function poll() {
        var slot = document.querySelectorAll('.cell-rule')[B.row]
        var ok = slot && slot.querySelector('.rule-confirmed')
        if (ok) { requestAnimationFrame(function () { c.f = performance.now() }); return }
        if (performance.now() - t0 > 5000) { c.f = -1; return }
        requestAnimationFrame(poll)
      })()
    }
    if (e.ctrlKey && e.altKey && e.shiftKey) {
      var k = e.code
      if (k === 'Digit1') { e.preventDefault(); create(122) }
      else if (k === 'Digit9') { e.preventDefault(); create(9850) }
      else if (k === 'Digit2') { e.preventDefault(); openAndFocus() }
      else if (k === 'Digit3') {
        e.preventDefault()
        dumpNow()
      }
    }
  }, true)
  ;(function alive(n) {
    if (!I() || typeof I().invoke !== 'function') { if (n < 240) setTimeout(function () { alive(n + 1) }, 250); return }
    put('__bench_alive__', Date.now()).catch(function () { if (n < 240) setTimeout(function () { alive(n + 1) }, 250) })
  })(0)
})()
