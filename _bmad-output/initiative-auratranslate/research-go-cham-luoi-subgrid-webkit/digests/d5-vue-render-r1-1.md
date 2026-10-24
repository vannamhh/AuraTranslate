# D5 Vue 3.5 render cost, r1-1 (web only, accessed 2026-10-24; 8 fetches + 5 searches; thin on benchmarks)

## Claims
- Vue 3.5 reactivity refactored to version counting + doubly-linked-list dep graph, "inspired by" Preact signals; PR #10397 merged 2024-02-25, shipped 3.5.0 | https://github.com/vuejs/core/pull/10397 | vuejs/core GitHub | 2024-02-25 | 2026-10-24 | high | framework-behavior
- 3.5 blog: memory −56% with no behavior change; array tracking "up to 10x faster in some cases" | https://blog.vuejs.org/posts/vue-3-5 | Vue blog | 2024-09 | 2026-10-24 | high | perf-number
- PR #10397 micro-benchmarks: "1 ref invoking 1000 effects" ~4,574 → 13,068 ops/s; one ref invoking several effects +118% to +185%; "1000 refs invoke" ~11.3 → 9.6 ops/s; effect-only, no components/DOM | https://github.com/vuejs/core/pull/10397 | vuejs/core GitHub | 2024-02 | 2026-10-24 | medium (summarized) | perf-number
- scheduler.ts (main): queueJob dedupes via QUEUED flag; queue sorted by job id (parents before children); one flush via `resolvedPromise.then(flushJobs)`; synchronous triggers join one microtask flush; post-flush callbacks deduped and sorted | https://raw.githubusercontent.com/vuejs/core/main/packages/runtime-core/src/scheduler.ts | vuejs/core source | main | 2026-10-24 | medium | framework-behavior
- INFERENCE (not retrieved): `triggerRef` on a shallowRef read by N component renders re-runs all N renders in one flush, id order; component's own render effect not gated by `shouldUpdateComponent`; no benchmark | (inference) | - | - | 2026-10-24 | low | framework-behavior
- `shouldUpdateComponent` gates only parent-forced updates (skip when compiled props unchanged; re-render on changed dynamic prop, dynamic slots, directives/transition); does not gate a child's own reactive trigger | https://raw.githubusercontent.com/vuejs/core/main/packages/runtime-core/src/componentRenderUtils.ts | vuejs/core source | main | 2026-10-24 | medium | framework-behavior
- Docs: child updates only when a received prop changes; keep props stable; example `:active="item.id === activeId"` (primitive) instead of `activeId` so only changed items update | https://vuejs.org/guide/best-practices/performance.html | vuejs.org | undated | 2026-10-24 | high | pattern
- `v-memo`: fixed-length array; all unchanged ⇒ whole sub-tree skipped incl. vnode creation; `v-memo="[]"` ≡ `v-once`; "solely for micro optimizations", most useful for `v-for` > 1000; must be on same element as `v-for`; with `:key` there, `item.id` inferred; usable on components when child update check de-optimized; wrong arrays skip needed updates | https://vuejs.org/api/built-in-directives.html#v-memo | vuejs.org | undated | 2026-10-24 | high | framework-behavior
- INFERENCE: memo array compared on each re-render of owning component; hit cost = array build + compare, no vnode/diff; no benchmark | https://vuejs.org/api/built-in-directives.html#v-memo | vuejs.org | - | 2026-10-24 | low | framework-behavior
- Docs: deep reactivity matters only in unusual cases ("100,000+ properties"); `shallowRef`/`shallowReactive` escape hatches, nested data immutable, replace root | https://vuejs.org/guide/best-practices/performance.html | vuejs.org | undated | 2026-10-24 | high | pattern
- `shallowRef` reactive only on `.value` access/assignment; `triggerRef` shown only in signal example | https://vuejs.org/guide/extras/reactivity-in-depth.html | vuejs.org | undated | 2026-10-24 | high | framework-behavior
- Latest stable 3.5.44 (13 Oct); prerelease 3.6.0-rc.13 (23 Oct); 3.6 rc.1 notes: @vue/reactivity refactored on alien-signals, no figures | https://github.com/vuejs/core/releases ; https://newreleases.io/project/github/vuejs/core/release/v3.6.0-rc.1 | vuejs/core GitHub; newreleases.io | 2026-10 | 2026-10-24 | medium | version-compat
- Vapor mode not stable; feature-complete in 3.6 RCs; one blog claiming stable contradicts release page | https://github.com/vuejs/core/releases ; https://www.itechguides.com/vue-3-6-vapor-mode-what-developers-need-to-know/ | vuejs/core GitHub; itechguides | 2026-10 | 2026-10-24 | medium | version-compat

## Leads
- renderer.ts `setupRenderEffect` (queueJob(update)); reactivity dep.ts `Dep.trigger/notify`.
- 3.6 CHANGELOG / alien-signals PR numbers; classic-mode gains.
- No 1-ref → N-components fan-out benchmark exists; measure in WKWebView.
- vuejs/core issues #6593/#6606 (v-memo with v-if).
- js-framework-benchmark Vue entries Chrome-only.
- Hypothesis (unsourced): per-row shallowRefs or shallowReactive Map keyed by row id, primitives as props; closest primary support is the docs' primitive-prop example.

## Looked for, not found
- triggerRef fan-out benchmarks; v-memo hit-cost benchmarks or at 50 × 200 rows; official statement on when v-memo array is evaluated; official scheduler/batching docs; official effectScope/markRaw per-row scoping docs; production retrospectives on editable grids in Vue 3.5; 3.5 vs 3.6 reactivity figures.
