/* eslint-disable */
;(function () {
  'use strict'
  var B = { layout: null, mouse: [], keys: [], focus: [], targets: [], nCells: 0 }
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
      setTimeout(function () {
        var c = document.querySelector('.cell-tgt')
        c.focus()
        var r = document.createRange(); r.selectNodeContents(c); r.collapse(false)
        var s = getSelection(); s.removeAllRanges(); s.addRange(r)
        put('__b216_ready__', B.nCells + ',' + (document.activeElement === c) + ',' + document.querySelectorAll('.cell-rule').length)
      }, 1500)
    }
  }
  function pickTargets() {
    var cells = Array.prototype.slice.call(document.querySelectorAll('.cell-tgt'))
    var vis = cells.filter(function (c) {
      var r = c.getBoundingClientRect()
      return r.top >= 40 && r.bottom <= window.innerHeight - 10 && r.width > 20
    })
    var out = []
    B.targets = []
    for (var k = 1; k < vis.length && out.length < 3; k += 2) {
      var r = vis[k].getBoundingClientRect()
      out.push([Math.round(r.left + Math.min(40, r.width / 2)), Math.round(r.top + r.height / 2), window.innerWidth, window.innerHeight].join(','))
      B.targets.push(vis[k].getAttribute('data-segment-id'))
    }
    put('__b216_target__', out.join('_') + 'v' + vis.length).catch(function (e) { put('__b216_err__', 'target ' + e) })
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
      twoFrames(function () { rec.f = performance.now(); rec.id = activeId() })
    }
  }
  document.addEventListener('mousedown', mouseEv('down'), true)
  document.addEventListener('mouseup', mouseEv('up'), true)
  document.addEventListener('keydown', function (e) {
    if (B.armed && e.metaKey && e.altKey && e.code === 'KeyY') {
      var rec = { ts: e.timeStamp, h: performance.now(), f: null, before: activeId(), after: null, later: null }
      B.keys.push(rec)
      twoFrames(function () { rec.f = performance.now(); rec.after = activeId() })
      setTimeout(function () { rec.later = activeId() }, 3000)
    }
    if (e.ctrlKey && e.altKey && e.shiftKey) {
      var k = e.code
      if (k === 'KeyQ') { e.preventDefault(); create(122) }
      else if (k === 'KeyW') { e.preventDefault(); create(9850) }
      else if (k === 'KeyE') { e.preventDefault(); openAndFocus() }
      else if (k === 'KeyR') { e.preventDefault(); try { pickTargets() } catch (x) { put('__b216_err__', 'pick ' + x) } }
      else if (k === 'KeyT') { e.preventDefault(); B.armed = true }
      else if (k === 'KeyU') {
        e.preventDefault()
        put('__b216__', JSON.stringify({ ua: navigator.userAgent, nCells: B.nCells, layout: B.layout, mouse: B.mouse, keys: B.keys, focus: B.focus, targets: B.targets, origin: performance.timeOrigin }))
      }
    }
  }, true)
  ;(function alive(n) {
    if (!I() || typeof I().invoke !== 'function') { if (n < 240) setTimeout(function () { alive(n + 1) }, 250); return }
    put('__bench_alive__', Date.now()).catch(function () { if (n < 240) setTimeout(function () { alive(n + 1) }, 250) })
  })(0)
})()
