# Verifier digest — refs 1–9 @ 99f3fc1

All nine verified; none re-anchored.
- [1] src/commands/keys.ts:510-545 (handle) @ 99f3fc1 — verified. `:527` `if (lacksPrimaryMod(entry.mods) && isTypingZone(event.target, event.code)) return false` before `preventDefault()` `:530`; no match ⇒ `:544` false. Listener `addEventListener('keydown', listener, { capture: true })` `:607`; `main.ts:1118` `attachKeyboard(window, …)`. Gate `isBlocked` `:604` can return early without preventDefault.
- [2] src/panels/GridPanel.vue:1497-1498 (onBeforeInput) @ 99f3fc1 — verified. `FROM_OUTSIDE = ['insertFromPaste','insertFromDrop','insertReplacementText']`, `if (!FROM_OUTSIDE.includes(event.inputType)) return`; earlier branches only `insertParagraph` (1461), `insertLineBreak` (1494).
- [3] src/panels/GridPanel.vue:1699-1704 (reportEdit) @ 99f3fc1 — verified. `noteEditorEdit(` appears once in GridPanel.vue (1702); reportEdit called at 1560 and 1714.
- [4] src/panels/editorPanelState.ts:786-793 (noteEditorEdit) @ 99f3fc1 — verified. `:791` `editedText.value.set(...)`; `editedText` `shallowRef(new Map())` `:682`; `:793` `if (shape !== before) triggerRef(editedShape)`; `editedShape` shallowRef `:686`.
- [5] src/panels/editorFlush.ts:129 (createEditorFlush) @ 99f3fc1 — verified. `const dirty = new Map<number, string>()`; one import `createWriteSchedule` from `../layout/writeSchedule` (`:30`); writeSchedule.ts imports not checked.
- [6] src/panels/GridPanel.vue:1168-1181 (restoreEditedText) @ 99f3fc1 — verified.
- [7] src/StatusBar.vue:279-283 (secondsSinceSave) @ 99f3fc1 — verified.
- [8] src/panels/GridPanel.vue:1918-1926 (gridBlockViews) @ 99f3fc1 — verified; exactly five props.
- [9] src/panels/GridPanel.vue:615-643 (gridBlockViews) @ 99f3fc1 — verified; second keep path `:635-637` via `sameRules`.
