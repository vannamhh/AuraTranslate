/* eslint-disable -- bench appended to the built bundle, never part of the product */
;(function () {
  'use strict'
  var B = { ready: null, sweeps: [], err: null }
  window.__B1413 = B
  var I = function () { return window.__TAURI_INTERNALS__ }
  function put(key, value) {
    return I().invoke('put_config', { kind: 'app_config', key: key, value: String(value) })
  }
  function create() {
    return I().invoke('create_work_from_text', { name: 'b1413', sourceLang: 'zh', genre: 'general', text: '句子。'.repeat(9850) })
      .then(function () { setTimeout(function () { location.reload() }, 500) })
      .catch(function (e) { put('__b1413_err__', 'create ' + e) })
  }
  function frame() { return new Promise(function (r) { requestAnimationFrame(function () { r() }) }) }
  function openAndSettle() {
    var tabs = document.querySelectorAll('nav.modes button.mode-tab')
    var t0 = performance.now()
    if (tabs[1]) tabs[1].click()
    ;(function poll() {
      var n = document.querySelectorAll('[data-num-segment-id]').length
      if (n >= 9850) return settle(n)
      if (performance.now() - t0 > 150000) return put('__b1413_ready__', 'NOROWS ' + n)
      requestAnimationFrame(poll)
    })()
    function settle(n) {
      var good = 0, last = performance.now(), s0 = last
      ;(function tick() {
        var now = performance.now()
        good = now - last < 40 ? good + 1 : 0
        last = now
        if (good >= 60) { B.ready = n; return put('__b1413_ready__', n) }
        if (now - s0 > 120000) return put('__b1413_ready__', 'NOQUIET')
        requestAnimationFrame(tick)
      })()
    }
  }
  function stats(a) {
    var s = a.slice().sort(function (x, y) { return x - y })
    return { median: s[Math.floor(s.length / 2)], max: s[s.length - 1], over50: s.filter(function (v) { return v > 50 }).length, n: s.length }
  }
  function sweep(shape) {
    var rows = Array.prototype.slice.call(document.querySelectorAll('[data-num-segment-id]'))
    var opts = { buttons: 1, bubbles: true }
    var steps = shape === 'adjacent' ? 60 : 59
    var rec = { shape: shape, rows: rows.length, total: [], js: [], flush: [], selected: null }
    return new Promise(function (resolve) {
      rows[0].dispatchEvent(new MouseEvent('mousedown', { buttons: 1, bubbles: true, button: 0, cancelable: true }))
      frame().then(async function () {
        for (var i = 1; i <= steps; i++) {
          var j = shape === 'adjacent' ? i : Math.min(i * 167, rows.length - 1)
          var t0 = performance.now()
          rows[j].dispatchEvent(new MouseEvent('mouseover', opts))
          var tj = performance.now()
          await Promise.resolve()
          await Promise.resolve()
          var tf = performance.now()
          await frame()
          rec.total.push(performance.now() - t0)
          rec.js.push(tj - t0)
          rec.flush.push(tf - t0)
        }
        rec.selected = document.querySelectorAll('.row-selected').length
        document.dispatchEvent(new MouseEvent('mouseup', { button: 0, buttons: 0, bubbles: true }))
        rec.stat = { total: stats(rec.total), js: stats(rec.js), flush: stats(rec.flush) }
        B.sweeps.push(rec)
        resolve()
      })
    })
  }
  async function runAll() {
    try {
      await sweep('adjacent')
      await frame(); await frame()
      await sweep('step167')
      await frame(); await frame()
      await sweep('adjacent')
      await frame(); await frame()
      await sweep('step167')
      put('__b1413_done__', '1')
    } catch (e) { put('__b1413_err__', 'sweep ' + e) }
  }
  document.addEventListener('keydown', function (e) {
    if (e.ctrlKey && e.altKey && e.shiftKey) {
      var k = e.code
      if (k === 'KeyW') { e.preventDefault(); create() }
      else if (k === 'KeyE') { e.preventDefault(); openAndSettle() }
      else if (k === 'KeyD') { e.preventDefault(); runAll() }
      else if (k === 'KeyU') { e.preventDefault(); put('__b1413__', JSON.stringify({ ua: navigator.userAgent, origin: performance.timeOrigin, ready: B.ready, sweeps: B.sweeps })) }
    }
  }, true)
  ;(function alive(n) {
    if (!I() || typeof I().invoke !== 'function') { if (n < 240) setTimeout(function () { alive(n + 1) }, 250); return }
    put('__bench_alive__', Date.now()).catch(function () { if (n < 240) setTimeout(function () { alive(n + 1) }, 250) })
  })(0)
})()
