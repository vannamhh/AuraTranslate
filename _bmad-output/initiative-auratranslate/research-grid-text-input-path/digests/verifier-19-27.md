# Verifier digest — refs 19–27 @ 99f3fc1 (history)

- [19] verified — 6a4e6b8 added `editedText = ref(new Map<number,string>())`; `noteEditorEdit` copies with `new Map(editedText.value)` and reassigns.
- [20] verified — 043f3bc: `shallowRef(new Map())` in place, `editedShape`, `textShapeOf`, `triggerRef` only on shape change; message: 4 columns × n rows + `restRuleById`, ~20% at 9.850 câu, release 157–163 s from 188–193 s.
- [21] verified — block "DOM SỞ HỮU VĂN BẢN BẢN DỊCH. VUE KHÔNG." first in src/panels/EditorPanel.vue at 6a4e6b8; ca33072 deleted it there and added a shorter `//` form to GridPanel.vue (moved and condensed, not byte for byte); at 99f3fc1 at GridPanel.vue line 1145.
- [22] verified — 43729b3: always-present `div.margin-slot` replacing `v-if` on `div.rule`, four `v-memo`, 3px rules, `confirmed` double; ~390 ms, ~700 → ~160 ms.
- [23] verified (message only; commit touches excluded paths) — 468 / 998 / 172 ms whole subgrid, 10 / 15 / 3 ms 200-row blocks; OS WKWebView, WebKit 627 (`305121@main`), Chromium; synthetic 9,850 rows, no Vue.
- [24] verified (message only) — Deferred row "chứ không thành AD", replaces 2-5b decision #1.
- [25] verified — 742096c added gridBlocks.ts (`GRID_BLOCK_ROWS = 200`), GridBlock.vue, gridBlockContext.ts; `useSelectionSurface(gridRoot,'source',…,'[data-col="src"]')`, `useSelectionSurface(gridRoot,'display',undefined,true,'[data-col="tgt"]')`; `scope` parameter in selectionContract.ts.
- [26] verified — f62012f: `block.rules` via restRulesOf per block; `gridBlockViews` `computed((previous)=>…)` keeps previous object when segments identical and `sameRules`; `restRuleById` dropped; 50 → 1/2 block renders (1,050-row vitest).
- [27] verified with a gap — `GRID_BLOCK_ROWS = 200` only in 742096c, never changed; no non-excluded diff or message compares another size; bench/prototype results live under excluded `_bmad-output` paths, so a comparison there cannot be ruled out.
