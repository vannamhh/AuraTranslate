Digest (all seven checked against the on-disk compiled RA.js/RB.js and vue@3.5.40; I did not regenerate the compiled files).

[7] verified. src/panels/GridPanel.vue:1839-1844 (grid) @ b2874aa: block v-for over gridBlockViews, key block.index. RB.js has the four column loops inside that outer callback, over block.segments. Their renderList calls end `}, _cache, 7)` (RB.js line 167, rule), `_cache, 9` (line 189, number), `_cache, 13` (line 302, source), `_cache, 15` (line 362, target). The slot numbers are literals, so every block gets the same four slots. The outer renderList (RB.js line 140) has no `_cache` argument and no memo, and the block `<div>` has no v-memo.

[8] verified. RA.js has one loop per column over _ctx.editorSegments, with distinct slots: 7 at line 160 (rule), 9 at line 182 (number), 13 at line 294 (source), 15 at line 353 (target). No two loops share a slot. All four are memoized (`const _memo` at RA.js lines 142, 165, 190, 311). The analyzer's "unread" marks on number 7 and source 13 are settled: the slots are 7 and 13 as in the ledger line, with number 9.

[9] verified, line range off. compiler-core.cjs.js line 5029-5052 (transformFor) @ vue@3.5.40: the memo branch builds `const _memo = (exp)` and `if (_cached && _cached.el [&& _cached.key === keyExp] && isMemoSame(_cached, _memo)) return _cached`. It then builds `_item`, sets `_item.memo = _memo` and returns it. The loop gets `_cached` as a 4th parameter, and renderExp receives `_cache` and `String(context.cached.length)` as the slot index. The slot number is allocated per v-for at compile time. The compiled RB.js matches (for example line 150).

[10] verified. runtime-core.cjs.js line 3192-3246 (renderList) @ vue@3.5.40: `cached = cache && cache[index]` at 3194. The callback gets `cached && cached[i]` as its 4th argument (3211-3237). `cache[index] = ret` is stored after the loop (about line 3244). So a slot holds one array, replaced on every renderList call.

[11] verified for tree B. Per runtime-core.cjs.js line 3192-3246 (renderList) @ vue@3.5.40 and compiler-core.cjs.js line 5029-5052 (transformFor) @ vue@3.5.40:
- The inner loops use literal slots 7/9/13/15 at RB.js lines 167/189/302/362, with `_cache` being the component-level cache.
- For block k, `cached = cache[slot]` is the array stored by block k-1's inner call in the same render. For the first block (k=0) it is the last block's array from the previous render.
- So `_cached` for item i is block k-1's vnode at index i, not this block's own previous vnode. Segment ids differ across blocks, so `_cached.key === s.id` fails (RB.js lines 150, 173, 211, 329) and every inner item rebuilds its vnode instead of reusing the memo.
- Nothing in RB.js keeps the cache per block: the outer renderList has no `_cache`, no memo, no per-iteration cache, and `_cache` is not indexed by block. The block div has no v-memo and the outer fragment is only `128 KEYED_FRAGMENT`. The only `_cache` uses in RB.js are the literal slots and the `_cache[n]` event-handler cache.
- The memo can still hit when there is a single block, because then the previous array is the same block's. For two or more blocks with distinct segment ids it never hits.

[12] verified. runtime-core.cjs.js line 5439-5441 (patch) @ vue@3.5.40: `if (n1 === n2) return`. Line 7880-7882 (cloneIfMounted): `child.el === null && child.patchFlag !== -1 || child.memo ? child : cloneVNode(child)`, so a memo vnode is returned as is. Line 6399-6420 (patchKeyedChildren): the head loop does `c2[i] = optimized ? cloneIfMounted(c2[i]) : normalizeVNode(...)`, then `isSameVNodeType`, then `patch`. When every pair is the same object, i reaches e1+1 and e2+1, so the later branches (mount/unmount and the key-map/LIS section) are not entered. I read the head loop directly and inferred the rest from the loop condition and the standard shape of the following branches; I did not read those branches.

[13] verified. runtime-core.cjs.js line 4771-4825 (shouldUpdateComponent) and line 6052-6070 (updateComponent) @ vue@3.5.40:
- When shouldUpdateComponent returns false, updateComponent only runs `n2.el = n1.el; instance.vnode = n2`. The child render is not run.
- Optimized mode (`optimized && patchFlag >= 0`): flag 1024 (DYNAMIC_SLOTS) returns true. Flag 16 (FULL_PROPS) runs hasPropsChanged and flag 8 (PROPS) checks dynamicProps. With patchFlag 0 and no dynamic props it falls to `return false`.
- Non-optimized mode: `(prevChildren || nextChildren) && !nextChildren?.$stable` returns true.
- An extra condition not in the claim: `nextVNode.dirs || nextVNode.transition` returns true first.
