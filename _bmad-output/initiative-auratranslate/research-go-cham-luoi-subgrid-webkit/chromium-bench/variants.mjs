// Biến thể bisection. css nối cuối <head>; html: (s)=>s; js chạy trước khi đo.
// Chạy với AURA_BENCH_SCROLLBARS=1 (bỏ --hide-scrollbars mà Playwright mặc định thêm ở headless).
export default {
  noSbGrid: { css: '.grid-scroll::-webkit-scrollbar{display:none}' },
  noSbAll: { css: '*::-webkit-scrollbar{display:none}' },
  gridOverflowScroll: { css: '.grid-scroll{overflow-y:scroll !important}' },
  gridOverflowHidden: { css: '.grid-scroll{overflow-y:hidden !important}' },
  gridNoMarginPad: { css: '.grid-scroll{margin:0 !important;padding:0 !important}' },
  gridNoFlex: { css: '.grid-scroll{flex:none !important;height:725px !important}' },
  gridOverflowXHidden: { css: '.grid-scroll{overflow-x:hidden !important}' },
  panelBodyFixed: { css: '.panel-body{flex:none !important;height:753px !important}' },
  gridBasisAuto: { css: '.grid-scroll{flex:1 1 auto !important}' },
  gridBasis0px: { css: '.grid-scroll{flex:1 1 0px !important}' },
  gridGrow0: { css: '.grid-scroll{flex:0 1 auto !important;height:725px !important}' },
  gridAbs: { css: '.panel-body{position:relative}.grid-scroll{position:absolute !important;inset:0 !important;margin:0 !important}' },
  panelPx: { css: '.panel{height:781px !important}' },
  panelNoPad: { css: '.panel{padding:0 !important}' },
  gridBasisPct100: { css: '.grid-scroll{flex:1 1 0% !important}.panel-body{flex:1 1 0px !important}' },
  isolate: { js: `{ const pn = document.querySelector('section.panel'); const w = document.createElement('div'); w.style.cssText='width:640px;height:781px'; w.append(pn); document.body.replaceChildren(w); }` },
  isolate2: { js: `{ const g = document.querySelector('.grid-scroll'); const w = document.createElement('div'); w.style.cssText='width:640px;height:781px;display:flex;flex-direction:column'; w.append(g); document.body.replaceChildren(w); }` },
  isolate3: { js: `{ const g = document.querySelector('.grid-scroll'); const w = document.createElement('div'); w.style.cssText='width:640px;height:781px'; w.append(g); g.style.height='725px'; g.style.flex='none'; document.body.replaceChildren(w); }` },
  gutterStable: { css: '.grid-scroll{scrollbar-gutter:stable !important}' },
}
