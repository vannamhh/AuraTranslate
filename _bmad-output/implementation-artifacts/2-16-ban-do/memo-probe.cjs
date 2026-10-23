const { createRequire } = require('module')
const req = createRequire(require('path').resolve(__dirname, '../../../package.json'))
const { Window } = req('happy-dom')
const win = new Window(); global.window = win; global.document = win.document
for (const k of ['Node','Element','HTMLElement','SVGElement','Text','Comment','DocumentFragment','MutationObserver']) global[k] = win[k]
const V = req('vue/dist/vue.cjs.prod.js')
const { createApp, ref, shallowRef, computed, nextTick, defineComponent } = V

const N = +process.argv[2] || 9850, R = 200
const segs = Array.from({ length: N }, (_, i) => ({ id: i + 1 }))
const blocks = []; for (let s = 0; s < N; s += R) blocks.push({ index: blocks.length, start: s, segments: segs.slice(s, s + R) })
const blockOf = id => Math.floor((id - 1) / R)

function variant(name, tpl, extra = {}) { return { name, tpl, extra } }
const ROW = `<div class="cell" :class="{ e: caret === s.id }">{{ body(s) }}</div>`
const variants = [
  variant('A flat (bố cục cũ)', `<div><template v-for="s in segs" :key="s.id" v-memo="[s, caret === s.id]">${ROW}</template></div>`),
  variant('B nested (bản thử)', `<div><div v-for="b in blocks" :key="b.index" class="blk"><template v-for="s in b.segments" :key="s.id" v-memo="[s, caret === s.id]">${ROW}</template></div></div>`),
  variant('C1 component/khối, đọc caret chung', `<div><Blk v-for="b in blocks" :key="b.index" :b="b" /></div>`, { child: 'global' }),
  variant('C2 component/khối, prop caretHere', `<div><Blk v-for="b in blocks" :key="b.index" :b="b" :caret-here="blockOf(caret) === b.index ? caret : null" /></div>`, { child: 'prop' }),
  variant('D v-memo ở cấp khối', `<div><div v-for="b in blocks" :key="b.index" class="blk" v-memo="[b, blockOf(caret) === b.index ? caret : null]"><template v-for="s in b.segments" :key="s.id" v-memo="[s, caret === s.id]">${ROW}</template></div></div>`),
]

async function run(v) {
  const caretRef = ref(1); let bodies = 0
  const body = s => { bodies++; return s.id }
  const comps = {}
  if (v.extra.child === 'global') comps.Blk = defineComponent({ props: ['b'], setup: () => ({ get caret() { return caretRef.value }, body }),
    template: `<div class="blk"><template v-for="s in b.segments" :key="s.id" v-memo="[s, caret === s.id]">${ROW}</template></div>` })
  if (v.extra.child === 'prop') comps.Blk = defineComponent({ props: ['b', 'caretHere'], setup: (p) => ({ get caret() { return p.caretHere }, body }),
    template: `<div class="blk"><template v-for="s in b.segments" :key="s.id" v-memo="[s, caret === s.id]">${ROW}</template></div>` })
  const app = createApp({ components: comps, setup: () => ({ segs, blocks, blockOf, body, get caret() { return caretRef.value } }), template: v.tpl })
  const el = document.createElement('div'); document.body.appendChild(el)
  app.mount(el)
  const moves = [[5000, 5001], [5199, 5200], [5001, 5002], [5200, 5201], [3000, 3001], [3001, 3002], [7000, 7001], [7001, 7002]]
  const out = []
  for (const [a, b] of moves) {
    caretRef.value = a; await nextTick()
    bodies = 0; const t = performance.now(); caretRef.value = b; await nextTick(); const ms = performance.now() - t
    out.push({ move: `${a}->${b}`, bodies, ms: +ms.toFixed(1) })
  }
  app.unmount(); el.remove()
  return out
}
;(async () => {
  for (const v of variants) {
    const r = await run(v); const ms = r.map(x => x.ms).sort((a, b) => a - b)
    console.log(`${v.name.padEnd(38)} bodies/move: ${[...new Set(r.map(x => x.bodies))].join(',')}  ms median ${ms[ms.length >> 1]} (min ${ms[0]}, max ${ms[ms.length - 1]})`)
  }
})()
