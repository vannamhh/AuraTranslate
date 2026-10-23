/* eslint-disable */
;(function () {
  'use strict'
  var B = { prof: [], quiet: null, layout: null, mouse: [], keys: [], focus: [], targets: [], nCells: 0 }
  window.__B216 = B
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
    return I().invoke('put_config', { kind: 'shortcut', key: 'editor.next_segment', value: 'Mod+Alt+Y' })
      .then(function () {
        return I().invoke('create_work_from_text', { name: 'b216-' + n, sourceLang: 'zh', genre: 'general', text: text(n) })
      })
      .then(function () { setTimeout(function () { location.reload() }, 500) })
      .catch(function (e) { put('__b216_err__', 'create ' + e) })
  }
  function twoFrames(cb) { requestAnimationFrame(function () { requestAnimationFrame(cb) }) }
  function activeId() {
    var a = document.activeElement
    return a && a.getAttribute ? a.getAttribute('data-segment-id') : null
  }
  function openAndFocus() {
    var tabs = document.querySelectorAll('nav.modes button.mode-tab')
    var L = { h0: performance.now(), cell: null, f: null }
    B.layout = L
    if (tabs[1]) tabs[1].click()
    ;(function poll() {
      var cell = document.querySelector('.cell-tgt')
      if (cell) {
        L.cell = performance.now()
        twoFrames(function () {
          L.f = performance.now()
          B.nCells = document.querySelectorAll('.cell-tgt').length
          settle()
        })
        return
      }
      if (performance.now() - L.h0 > 150000) { return put('__b216_ready__', 'NOCELLS') }
      requestAnimationFrame(poll)
    })()
    function settle() {
      var good = 0, last = performance.now(), L2 = { t0: last }
      B.quiet = L2
      ;(function frame() {
        var now = performance.now()
        good = now - last < 40 ? good + 1 : 0
        last = now
        if (good >= 60) {
          L2.q = now
          var s0 = performance.now()
          void document.body.offsetHeight
          L2.forced_layout_ms = performance.now() - s0
          return finish()
        }
        if (now - L2.t0 > 120000) return put('__b216_ready__', 'NOQUIET')
        requestAnimationFrame(frame)
      })()
    }
    function finish() {
      var c = document.querySelector('.cell-tgt')
      c.focus()
      var r = document.createRange(); r.selectNodeContents(c); r.collapse(false)
      var s = getSelection(); s.removeAllRanges(); s.addRange(r)
      put('__b216_ready__', B.nCells + ',' + (document.activeElement === c) + ',' + document.querySelectorAll('.cell-rule').length)
    }
  }
  function visTargets() {
    var cells = Array.prototype.slice.call(document.querySelectorAll('.cell-tgt'))
    return cells.filter(function (c) {
      var r = c.getBoundingClientRect()
      return r.top >= 40 && r.bottom <= window.innerHeight - 10 && r.width > 20
    })
  }
  function sig(vis) {
    return vis.map(function (c) { var r = c.getBoundingClientRect(); return Math.round(r.left) + ':' + Math.round(r.top) + ':' + Math.round(r.height) }).join('|')
  }
  function pickTargets(tries) {
    tries = tries || 0
    var v0 = visTargets(), s0 = sig(v0)
    setTimeout(function () {
      var vis = visTargets()
      if (sig(vis) !== s0) return tries < 8 ? pickTargets(tries + 1) : put('__b216_target__', 'v0')
      var out = []
      B.targets = []
      for (var k = 1; k < vis.length && out.length < 3; k += 2) {
        var r = vis[k].getBoundingClientRect()
        out.push([Math.round(r.left + Math.min(40, r.width / 2)), Math.round(r.top + r.height / 2), window.innerWidth, window.innerHeight].join(','))
        B.targets.push(vis[k].getAttribute('data-segment-id'))
      }
      put('__b216_target__', out.join('_') + 'v' + vis.length).catch(function (e) { put('__b216_err__', 'target ' + e) })
    }, 1000)
  }
  function profile(rec, kind) {
    var host = document.querySelector('.grid')
    var P = { kind: kind, h: rec.h, bubble: null, calls: [], task: null, style: null, layout: null, raf1: null, f: null, muts: 0, blocks: 0 }
    rec.prof = P
    B.prof.push(P)
    CUR = P
    moBuf = []
    var onBubble = function () { P.bubble = performance.now(); document.removeEventListener(kind === 'mouse' ? 'mousedown' : 'keydown', onBubble, false) }
    document.addEventListener(kind === 'mouse' ? 'mousedown' : 'keydown', onBubble, false)
    setTimeout(function () {
      P.task = performance.now()
      CUR = null
      var m = moBuf.concat(MO.takeRecords()), seen = {}
      P.muts = m.length
      m.forEach(function (r) {
        var b = r.target && (r.target.nodeType === 1 ? r.target : r.target.parentElement)
        b = b && b.closest ? b.closest('.grid-block') : null
        if (b) { if (!b.__pid) b.__pid = ++pidSeq; seen[b.__pid] = 1 }
      })
      P.blocks = Object.keys(seen).length
      void getComputedStyle(document.body).color
      void document.body.getBoundingClientRect()
      P.style = performance.now()
      void document.body.offsetHeight
      P.layout = performance.now()
      requestAnimationFrame(function () { P.raf1 = performance.now() })
    }, 0)
  }
  var pidSeq = 0
  var moBuf = []
  var MO = new MutationObserver(function (recs) { for (var i = 0; i < recs.length; i++) moBuf.push(recs[i]) })
  var CUR = null
  function wrap(proto, name) {
    var orig = proto[name]
    proto[name] = function () {
      var t = performance.now()
      try { return orig.apply(this, arguments) } finally {
        if (CUR) { CUR.calls.push([name, Math.round((performance.now() - t) * 10) / 10]) }
      }
    }
  }
  ;['focus', 'getBoundingClientRect', 'scrollIntoView'].forEach(function (n) { wrap(Element.prototype, n) })
  ;['setPosition', 'addRange', 'removeAllRanges', 'collapse', 'selectAllChildren', 'extend'].forEach(function (n) { wrap(Selection.prototype, n) })
  ;['getBoundingClientRect', 'getClientRects'].forEach(function (n) { wrap(Range.prototype, n) })
  function armMO() {
    var host = document.querySelector('.grid')
    if (host) MO.observe(host, { subtree: true, childList: true, attributes: true, characterData: true })
  }
  document.addEventListener('focusin', function (e) {
    if (!B.armed) return
    var rec = { h: performance.now(), f: null, id: e.target && e.target.getAttribute ? e.target.getAttribute('data-segment-id') : null }
    B.focus.push(rec)
    twoFrames(function () { rec.f = performance.now() })
  }, true)
  function mouseEv(kind) {
    return function (e) {
      if (!B.armed) return
      var rec = { kind: kind, ts: e.timeStamp, h: performance.now(), f: null, id: null }
      B.mouse.push(rec)
      if (kind === 'down') profile(rec, 'mouse')
      twoFrames(function () { rec.f = performance.now(); rec.id = activeId(); if (rec.prof) rec.prof.f = rec.f })
    }
  }
  document.addEventListener('mousedown', mouseEv('down'), true)
  document.addEventListener('mouseup', mouseEv('up'), true)
  document.addEventListener('keydown', function (e) {
    if (B.armed && e.metaKey && e.altKey && e.code === 'KeyY') {
      var rec = { ts: e.timeStamp, h: performance.now(), f: null, before: activeId(), after: null, later: null }
      B.keys.push(rec)
      profile(rec, 'key')
      twoFrames(function () { rec.f = performance.now(); rec.after = activeId(); if (rec.prof) rec.prof.f = rec.f })
      setTimeout(function () { rec.later = activeId() }, 3000)
    }
    if (e.ctrlKey && e.altKey && e.shiftKey) {
      var k = e.code
      if (k === 'KeyQ') { e.preventDefault(); create(122) }
      else if (k === 'KeyW') { e.preventDefault(); create(9850) }
      else if (k === 'KeyE') { e.preventDefault(); openAndFocus() }
      else if (k === 'KeyR') { e.preventDefault(); try { pickTargets() } catch (x) { put('__b216_err__', 'pick ' + x) } }
      else if (k === 'KeyT') { e.preventDefault(); armMO(); B.armed = true }
      else if (k === 'KeyU') {
        e.preventDefault()
        put('__b216__', JSON.stringify({ ua: navigator.userAgent, nCells: B.nCells, layout: B.layout, mouse: B.mouse, keys: B.keys, focus: B.focus, targets: B.targets, prof: B.prof, quiet: B.quiet, origin: performance.timeOrigin }))
      }
    }
  }, true)
  ;(function alive(n) {
    if (!I() || typeof I().invoke !== 'function') { if (n < 240) setTimeout(function () { alive(n + 1) }, 250); return }
    put('__bench_alive__', Date.now()).catch(function () { if (n < 240) setTimeout(function () { alive(n + 1) }, 250) })
  })(0)
})()
