# Brief

Repository: the working directory (git top level).
Commits read: `cc60020` (full `cc6002023bc7de1df1c4a3ef474fd628c9658802`), called tree A; and `b2874aa` (full `b2874aaaa23da5f6ec7a275cd544485bd187fcc7`), called tree B, a detached commit in the same object store whose parent is `cc60020`. Read both through git (`git show <sha>:<path>`, `git grep -n <pattern> <sha> -- <paths>`); `git diff cc60020 b2874aa` shows what B changes.
Vue runtime: not in git; read it on disk at `node_modules/@vue/runtime-core/dist/runtime-core.cjs.js` (and `node_modules/@vue/runtime-dom/`, `node_modules/@vue/compiler-core/` as needed), version 3.5.40; cite it as `path:line (symbol) @ vue@3.5.40`.
File budget: 15 files per analyzer. History budget: 10 commits per historian. Validation: normal.
Paths left out of every search and every diff: `_bmad-output/`, `_bmad/`, `.claude/`, `.agents/`.

## Questions

1. (how) When the editor caret moves from one segment to another in the comparison grid (`src/panels/GridPanel.vue`), which reactive state changes, and which render functions run again as a result? Answer for tree A and for tree B.
2. (how) How does the grid template build its rows — the `v-for` levels, `key`s, `v-memo` and its dependency list, Fragments, `subgrid` — and which reactive values does each row read while rendering? Answer for tree A and for tree B.
3. (how) In the installed Vue 3.5.40 runtime, when a parent component re-renders, what work does Vue do for children that are (a) inline `v-for` vnodes carrying `v-memo`, (b) the same vnodes wrapped in a Fragment or an outer nested `v-for`, (c) child components whose props did not change? And what does `patchKeyedChildren` do when given a new array whose elements are the same as before?
4. (what touches) Which places in `src/` read the caret and strip-anchor state (`setEditorCaret`, `stripAnchorIndex` and the state they write) and are render dependencies of `GridPanel`?
5. (why) Why do the grid's rows carry `v-memo` with the dependency list they have at `cc60020`?
6. (how) How are `GridPanel`'s rendering and caret movement tested?
