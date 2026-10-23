Digest for question 5 (why the grid rows carry `v-memo` with these lists at cc60020).

**Provenance of the four lists**
- All four `v-memo` attributes were added in one commit, `43729b3 "perf(grid): 2-14 dừng ở blocked vì dời con trỏ ở 9.850 câu vẫn ~600–740 ms…"` (2026-10-23 09:22). `git log -S'v-memo' cc60020 -- src` returns only this commit, so nothing earlier in `src` had `v-memo`.
- The locator's "four columns" claim is correct. `git blame` shows 43729b3 for lines 1819, 1841, 1856-1868 and 2019-2027, except line 2025.
- Line 2025 (`editorEditedShape.get(s.id)`) is `043f3bc "perf(editor): 2-15 dừng ở blocked vì bỏ lượt dựng lại cả lưới mỗi phím…"` (2026-10-23 11:57). The locator is right here too, with one correction: 043f3bc did not add a new entry. It replaced `editorEditedText.get(s.id)`, which 43729b3 had put on that line, with `editorEditedShape.get(s.id)`.

**Entry by entry (what the diff and message give)**

| Entry (column) | Introducing commit | Reason found |
|---|---|---|
| `s` (all four) | 43729b3 | No stated reason. It is the row object, so any change to the segment re-renders the row. |
| `ruleClassForRow(s.id)` (rule column, line 1819) | 43729b3 | The same commit defines `ruleOf`, `ruleClassForRow` and `restRuleById`. `ruleOf` returns `'primary'` when `editorCaretSegmentId === id` and otherwise the entry from `restRuleById`. This replaces the old `ruleById` and `ruleClassById` computeds, which took the caret as an input and so changed for every row on every caret move. With the new split, only the old and new caret rows change their memo dependency. The code is the evidence here; the message does not spell this out. |
| `ruleOf(s.id)` (target column, line 2026) | 43729b3 | Same function, same reason. `cellLabelOf` in the same diff also switched to `ruleOf(id)`. |
| `i` (number and target columns) | 43729b3 | No reason stated. It is the row index displayed in the number column. |
| `selectedRowClassById.get(s.id)` (all four) | 43729b3 | The Map existed before; it first appears in `915f1af`. 43729b3 only uses it as a dependency because the cell templates already read it in `:class`. |
| `s.id === stripAnchorSegmentId` (all four) | 43729b3 | The variable first appears in `a92008f` (12-9). The rows already emit a `strip-pad` element after the anchor row, so the memo has to depend on whether the row is the anchor. |
| `rowErrorTextById.get(s.id)` (target column, line 2024) | 43729b3 | The Map first appears in `de7c67e` (12-7). The target cell already used it. |
| `editorEditedText.get(s.id)`, then `editorEditedShape.get(s.id)` (target column, line 2025) | 43729b3, then 043f3bc | 043f3bc's message says each `input` replaced the whole edited-text `Map`, so "mỗi phím chạy lại 4 cột × n hàng cộng `restRuleById` trên n câu". The fix mutates the Map in place and notifies the grid only when a sentence's emptiness changes ("thứ duy nhất lưới hiển thị theo chữ đang gõ"). The diff changes `empty:` and `restRuleById` to read a text-shape value instead of the text, and the memo entry to match. |
| Source column entries: `cutOffsetsOf`, `glossaryTermsOf`, `concordanceSourceHighlight` ternary, `trailingImagesOf`, `s.id === firstSegmentId ? editorChapterAssets : null`, `editorAssetsDir`, `showHanViet`, `viewMode` (lines 1860-1867) | 43729b3 as memo entries | These are the inputs the source cell's template already read. The accessors come from older commits: `cutOffsetsOf` from `a664dac`, `glossaryTermsOf` from `53035e7`, `concordanceSourceHighlight` from `3a2e03c`, and `trailingImagesOf`, `editorAssetsDir` and `firstSegmentId` from `46f068f`. The commit gives no per-entry reasoning. The one related code change is `trailingImagesOf` returning a shared constant `NO_IMAGES` instead of a fresh `[]`, so the memo comparison stays stable. |

**What the memo is meant to make cheaper**
- The only statement is in the 43729b3 subject. Six separate measurements put about 390 ms of a caret move at 9.850 sentences on the debug build on WebKit recomputing the layout of the whole `subgrid`. A `v-if` on the margin rule inserted a `div.rule` into the row that received the caret. They put about 225 ms on "`PanelFrame` dựng lại cả slot lưới".
- The message lists three fixes: the margin rule is now always present in every row (a `div.margin-slot`, with `ruleSlotClass`), `v-memo` goes on the four columns, and all rule borders are 3 px and differ only in style and colour (`confirmed` becomes a double line). It says a border-width change also forces a whole-grid relayout.
- The only number tied to a fix is "~700 → ~160 ms trên bàn biến thể CSS". That was measured on a CSS-variant test bench, so it covers the CSS change, not `v-memo`.
- The message gives no before/after figure for `v-memo` alone. It ends with "Ice ngừng sửa lưới ở đây… một phím gõ ở 9.850 câu vẫn ~700 ms", so the typing path was still slow after the commit.

**Did the commits touch how `PanelFrame` renders its slot?** (load-bearing)
- No. Neither 43729b3 nor 043f3bc changes `src/panels/PanelFrame.vue`. Their file lists are `GridPanel.vue`, the tests, `editorPanelState.ts` and `editorSegments.ts`. The only commits on `PanelFrame.vue` before cc60020 are `90d4428`, `505c8bc` and `5a23410`, none of them about this.
- The `v-memo` lists work around PanelFrame's slot re-render rather than change it. 043f3bc's message says "thân lưới là slot của `PanelFrame` nên mỗi phím chạy lại 4 cột × n hàng".
- The new test `tests/frontend/gridTypingCost.test.ts` (added in 043f3bc) uses a stub PanelFrame that counts renders (`probe.frameRenders`). It asserts `frameRenders` is 0 for a keystroke inside a written row at 20 and 200 rows. That guards the in-place `Map` write, not the memo.
- `tests/frontend/gridCaretMoveCost.test.ts` and `gridRuleStable.test.ts` (added in 43729b3) stub PanelFrame as a plain slot. They check that the margin-slot classes follow the caret. Neither contains the word "memo".

**Load-bearing claims**
1. All four `v-memo` attributes were added at once in 43729b3, and only the `editorEditedShape` entry on line 2025 is from a later commit, 043f3bc.
2. The stated purpose in 43729b3 is to avoid re-running the rows when the caret moves, given that PanelFrame re-renders the whole grid slot (about 225 ms of a 600–740 ms caret move). The message gives no measured effect of `v-memo` itself.
3. The `ruleOf`/`restRuleById` split in 43729b3 is the design that makes the memo effective for the rule and target columns, because only the previous and new caret rows change their dependency.
4. 043f3bc swapped `editorEditedText.get` for `editorEditedShape.get` because every keystroke replaced the whole `Map`, which re-ran all rows. It stopped at "blocked": 300 keys at 9.850 sentences on release still took 157–163 s, about 0,5 s per key, "chưa quy nguồn".

**Budget:** 2 commits followed of 10 (43729b3, 043f3bc), plus `git log -S` lookups for the older entry origins (not followed).
