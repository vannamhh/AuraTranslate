# Brief

Repository: the working directory. Commit: `99f3fc1c0c46326e6743b6a60e63f96889c480c4` (`99f3fc1`), branch `master`. Read code at this commit (`git show 99f3fc1:<path>` when the working tree may differ).

Budgets: at most 15 files per question; at most 10 commits of history per question. Validation level: normal.

Paths left out of every search: `_bmad-output/`, `_bmad/`.

## Questions

1. (how) What steps does a `beforeinput` or `input` event in a target-column cell of the grid go through, from the DOM listener to the state writes it causes?
2. (what touches) Which components, computeds, watchers or render functions read the reactive state that the path in question 1 writes (`editedText`, `editedShape`, `lastSavedAt`, the flush pending set, and any other state written there)?
3. (how) How does `GridBlock` decide whether to re-render, and what does a re-render of a block or a row produce (`v-memo` lists, slots, props passed down from `GridPanel`)?
4. (how) Besides state writes, what other work runs around one keystroke in the grid: caret and selection handling, rule markers, proofread repaints, the typing zone, the status bar?
5. (why) Why is the edited-text state shaped the way it is (the whole `Map` replaced per edit, `editedShape` with `triggerRef`), and why is the grid split into blocks the way it is?
6. (how) Which tests (vitest, cargo, e2e) cover typing in the grid and its render cost, and what does each assert?
