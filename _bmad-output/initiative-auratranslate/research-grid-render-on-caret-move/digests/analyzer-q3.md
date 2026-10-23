Digest for Q3. All citations are `runtime-core.cjs.js` or `compiler-core.cjs.js` @ vue@3.5.40, read on disk from node_modules/@vue/{runtime-core,compiler-core}/dist as the brief allows. I did not use git. I read 2 files, and ran one compile of three sample templates through @vue/compiler-dom to see the real codegen (script: /private/tmp/claude-501/-Users-hoangnam-LocalSites-addon-AuraTranslate/38ff5937-f6f4-47cd-9033-1ffaf37e4590/scratchpad/c.cjs).

**What the compiler emits**

- LOAD-BEARING. `v-for` plus `v-memo` on an element does not use `withMemo`. It is compiled into the `renderList` callback.
  - compiler-core.cjs.js:5030-5050 (v-for transform, `if (memo)` branch) builds the loop body: `const _memo = (exp)`, then `if (_cached && _cached.el && _cached.key === <key> && isMemoSame(_cached,_memo)) return _cached`, then `const _item = <block>; _item.memo = _memo; return _item`.
  - The `renderList` call gets extra args `_cache, <slot index>` (compiler-core.cjs.js:5045-5050).
  - `withMemo` is emitted only for `v-memo` outside a `v-for` (`transformMemo`, compiler-core.cjs.js:6642).
- Compiled sample, flat `v-for` with `v-memo` (compiled output):
  ```
  _openBlock(true), _createElementBlock(_Fragment, null, _renderList(rows, (r,__,___,_cached)=>{... return _cached ...}, _cache, 0), 128 /*KEYED_FRAGMENT*/)
  ```
- The `v-for` fragment flag is `STABLE_FRAGMENT` (64) only when the source is a constant. Otherwise it is `KEYED_FRAGMENT` (128) if the item has a `:key`, else `UNKEYED_FRAGMENT` (256) (compiler-core.cjs.js:4950-4951). A dynamic source gives `openBlock(true)`, which disables tracking.
- Each memoised item is an element block (`openBlock(), createElementBlock("div",{key})`).
- A `<template v-for>` wrapper with a keyed child compiles to a `Fragment` with flag 64 containing an inner `openBlock(true)` plus `KEYED_FRAGMENT` `renderList`. Compiled sample, outer/inner nested `v-for`:
  ```
  outer renderList(groups, g => Fragment{key:g.id}[ inner KEYED_FRAGMENT renderList(g.rows, ..., _cache, 0) ], 64)
  ```

**Runtime: `renderList` and the memo cache**

- LOAD-BEARING. `renderList` reads `cached = cache && cache[index]` (3194) and passes `cached && cached[i]` to each `renderItem` call as the 4th argument (3211, 3223, 3230, 3237). After the loop it stores `cache[index] = ret` (3244). The cache is therefore one array per slot index, replaced on every call.
- LOAD-BEARING. `isMemoSame` (8620-8636) compares the memo arrays with `hasChanged` per element. If they are equal and `isBlockTreeEnabled > 0 && currentBlock`, it pushes `cached` into `currentBlock` and returns true. Inside a `v-for` fragment `currentBlock` is `null` (`openBlock(true)`), so no push happens there.
- `withMemo` (8610-8618) has the same shape: it returns `cache[index]` if `isMemoSame`, otherwise calls `render()` and stores the result.

**Case (a): inline `v-for` vnodes with `v-memo`, memo unchanged, on parent re-render**

- The parent render function body runs, because the parent's reactive effect re-runs `renderComponentRoot` (6208). The `renderList` loop runs once per item, and for each item the callback evaluates `_memo` and the key check.
- No new vnode is created for an unchanged item. The callback returns the previous vnode `_cached` (the same object) after `isMemoSame`. The item's children expressions (`_toDisplayString` and so on) do not run.
- LOAD-BEARING. The `KEYED_FRAGMENT` goes through `patchChildren` (6279-6294) to `patchKeyedChildren` (6399). Its first loop calls `cloneIfMounted(c2[i])` when `optimized` is true. `cloneIfMounted` (7880-7882) returns the vnode itself if `child.memo` is set. If `optimized` is false it calls `normalizeVNode`, which I did not read in full.
- `isSameVNodeType` is true, and `patch(n1, n2, ...)` returns at `if (n1 === n2) return;` (5440), because both are the same object. `patchElement`, props work and children work are all skipped.
- Per child, in this case:
  - the render body runs;
  - the callback runs and `isMemoSame` loops over the memo array;
  - no vnode is allocated;
  - `patch` is called once and exits at the identity check.
- Cost still linear in N: callback + memo compare + `cloneIfMounted` + `isSameVNodeType` + `patch` call per item, plus `new Array(n)` in `renderList`.
- LOAD-BEARING. `patchKeyedChildren` with an identical array, in the same-element case:
  - The head loop (6399-6420) walks every index and calls `patch` on each pair. If every pair is the same type, `i` reaches `min(l1,l2)`.
  - `i > e1` is then true. `i <= e2` is false, so nothing is mounted. The `else if (i > e2)` branch is skipped (which would unmount).
  - So the keyed-diff middle section (key map and LIS) is never reached. It is O(N) `patch` calls and no moves or DOM writes.
- If a memo changed, that item is rebuilt as a fresh block and goes through a normal `patchElement`.

**Case (b): the same vnodes wrapped in a Fragment or an outer nested `v-for`**

- Wrapper vnodes are always new. Their `renderList` callback has no memo, so a new Fragment vnode is created per outer item on each render.
- The outer fragment is flag 64 (`STABLE_FRAGMENT`) with a dynamic `renderList` source. `processFragment` (5914-5975) is taken for it.
  - If `patchFlag & 64`, `dynamicChildren` is set and `n1.dynamicChildren.length === dynamicChildren.length`, it calls `patchBlockChildren` (5854-5890). This path only walks the tracked dynamic children, not the full child array.
  - Otherwise it calls `patchChildren` on the full child list.
  - In the sample, the inner `v-for` is `openBlock(true)` and so has `EMPTY_ARR` dynamic children (`setupBlock`, 7601-7605).
- Per inner child, once the wrapper is patched, the inner list is a `KEYED_FRAGMENT` and goes through `patchKeyedChildren` as in (a), so the identity short-circuit applies per memoised item.
- LOAD-BEARING (trap). For nested `v-for`, the inner `renderList` call uses the same `_cache` slot index for every outer iteration (compiled sample: `_cache, 0` inside the outer callback).
  - `cache[index] = ret` (3244) overwrites the slot after each inner call.
  - The inner `cached[i]` for outer item j is therefore the vnode array of outer item j-1 (or the last item of the previous render pass for j=0).
  - The compiled guard `_cached.key === r.id` then fails when keys differ across groups, so the memo misses and the row is rebuilt every render.
  - This is code reading plus the compiled output. I did not execute it.
  - A flat `v-for` has one cache array per slot and does not have this problem.
- `patchBlockChildren` calls `patch(old, new, ..., optimized=true)` for each tracked child. A plain `Fragment` child passes through `hostParentNode(oldVNode.el)` first (5863-5870).

**Case (c): child components with unchanged props**

- The parent render still creates a new component vnode per child each render (`createVNode`, via `createBlock` in the compiled output for a `v-for` of `Child`).
- `patch` runs on it: `n1 !== n2`, `isSameVNodeType` is true, then it goes to `processComponent` (5977) and on to `updateComponent` (6052).
- LOAD-BEARING. `updateComponent` calls `shouldUpdateComponent(n1, n2, optimized)` (4771-4825).
  - If it returns false, only `n2.el = n1.el` and `instance.vnode = n2` run. `instance.update()` is not called, so the child render function does not run and no child subtree patch happens.
  - In optimized mode (`optimized && patchFlag >= 0`), `patchFlag & 1024` (dynamic slots) returns true. Flag 16 (`FULL_PROPS`) calls `hasPropsChanged`, and flag 8 (`PROPS`) checks only the listed `dynamicProps` via `hasPropValueChanged`. A component with static props and `patchFlag` 0 returns false.
  - In non-optimized mode, any `prevChildren || nextChildren` without `$stable` returns true, so slot content forces a child update.
  - It also returns true if `nextVNode.dirs` or `transition` is set.
- Per child, in this case:
  - the vnode is allocated (the parent render ran the `createBlock` call);
  - `patch` runs;
  - `shouldUpdateComponent` runs;
  - there is no child render and no child patch.
- The vnode `children` check matters here: any child with slot children and no `$stable` forces an update even when props are equal.

**Where the parent patch starts**

- The `componentUpdateFn` branch for an update (6205-6223): `renderComponentRoot` produces `nextTree`, then `patch(prevTree, nextTree, hostParentNode(prevTree.el), ...)` with `optimized` defaulting to `!!n2.dynamicChildren` (5439).
- That means a root block starts in optimized mode, and `patchElement` uses `patchBlockChildren` only when `dynamicChildren` exists and lengths match (5758-5800). Otherwise `patchFlag`, `optimized` and `dynamicChildren` are reset to full-diff values (~5780). Lengths can differ when `isMemoSame` pushes cached vnodes into a tracked block (`currentBlock` is non-null). I did not read that out end to end.

**Not looked for or not done**

- I did not run any of this. The compile is the only execution (a codegen check).
- I did not read `normalizeVNode` (7864) or `traverseStaticChildren` in detail, nor `mountComponent`.
- The cost of `renderList` itself with reactive arrays (`shallowReadArray`, `toReactive` wrapping at 3198-3206) was seen but not analysed. A reactive array source with a deep wrap creates a proxy per item on every call.
- Search method: `grep -n` on the two dist files plus targeted `sed -n` ranges; no git commit applies because node_modules is not tracked.
