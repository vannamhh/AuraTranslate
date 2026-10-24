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
  var pidSeq = 0
  function blockOf(node) {
    var el = node && (node.nodeType === 1 ? node : node.parentElement)
    el = el && el.closest ? el.closest('.grid-block') : null
    if (el && !el.__pid) el.__pid = ++pidSeq
    return el ? el.__pid : 0
  }
  function profileConfirm(c) {
    var P = { h: c.h, task0: null, calls: [], seen: null, flushed: null, raf1: null, f: null, muts: 0, blocks: 0, focusin: null }
    c.prof = P
    var blocks = {}, done = false
    var origInvoke = I().invoke
    var wrapped = function (cmd, args) {
      var rec = { cmd: cmd, start: performance.now(), end: null }
      P.calls.push(rec)
      var out = origInvoke.apply(this, arguments)
      if (out && typeof out.then === 'function') out.then(function () { rec.end = performance.now() }, function () { rec.end = -performance.now() })
      return out
    }
    try { I().invoke = wrapped; P.ipcWrap = I().invoke === wrapped } catch (err) { P.ipcWrap = 'throw ' + err }
    var onFocus = function () { if (P.focusin === null) P.focusin = performance.now() }
    document.addEventListener('focusin', onFocus, true)
    setTimeout(function () { P.task0 = performance.now() }, 0)
    var mo = new MutationObserver(function (recs) {
      if (done) return
      P.muts += recs.length
      for (var i = 0; i < recs.length; i++) { var b = blockOf(recs[i].target); if (b) blocks[b] = 1 }
      if (P.seen !== null) return
      var slot = document.querySelectorAll('.cell-rule')[B.row]
      if (slot && slot.querySelector('.rule-confirmed')) {
        P.seen = performance.now()
        setTimeout(function () {
          P.flushed = performance.now()
          requestAnimationFrame(function () {
            P.raf1 = performance.now()
            requestAnimationFrame(function () {
              P.f = performance.now()
              done = true
              mo.disconnect()
              P.blocks = Object.keys(blocks).length
              if (P.ipcWrap === true) I().invoke = origInvoke
              document.removeEventListener('focusin', onFocus, true)
            })
          })
        }, 0)
      }
    })
    mo.observe(document.body, { subtree: true, childList: true, attributes: true, characterData: true })
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
      try { profileConfirm(c) } catch (err) { c.profErr = String(err) }
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
      if (k === 'KeyQ') { e.preventDefault(); create(122) }
      else if (k === 'KeyW') { e.preventDefault(); create(9850) }
      else if (k === 'KeyE') { e.preventDefault(); openAndFocus() }
      else if (k === 'KeyU') {
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
