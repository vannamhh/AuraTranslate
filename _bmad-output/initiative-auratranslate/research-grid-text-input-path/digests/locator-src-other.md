# Locator digest — area `src/` minus `src/panels/` @ 99f3fc1

`git grep`/`git show` at 99f3fc1, excluded `_bmad-output/`, `_bmad/`. Read `src/panels/editorPanelState.ts` only to learn what `noteEditorEdit` writes: `editedText` (`shallowRef` Map mutated in place via `.set`, no trigger), `editedShape` (in place, `triggerRef` only when row `TextShape` changes), `tmFilledSegmentIds` via `dropTmFilled` (new Set), `flush.markChanged` + `armFlushTimer`, `datThongBao({})` nulling `confirmNotice`, `regroupNotice`, `navNotice`; `flushEditorNow` later writes `flushError`, `lastSavedAt` (`src/panels/editorPanelState.ts:682-698 / :758 noteEditorEdit / :835 flushEditorNow @ 99f3fc1`). Map not replaced per edit at this commit (comment :677-681); brief premise does not match.

## Q1
- `src/commands/keys.ts:607 (attachKeymap) @ 99f3fc1` — single `keydown` listener on `window`, `{capture:true}`, runs before grid handlers on every key.
- `src/commands/keys.ts:439 (isTypingZone) @ 99f3fc1` — `isContentEditable === true` ⇒ typing zone.
- `src/commands/keys.ts:510-530 (createKeymap handle) @ 99f3fc1` — early return on `isComposing`; `Mod+Z` yields to contenteditable; no-modifier chords skipped in typing zone; else `preventDefault` + `dispatch`.
- `src/main.ts:1118 (boot) @ 99f3fc1` — `attachKeyboard(window, { isBlocked })`, gate is OR over ~20 overlay-open refs.
- `src/commands/index.ts:5442 (attachKeyboard) @ 99f3fc1` — proxy wrapping `attachKeymap`.
- `src/config/segment.ts:757 (saveSegmentTargets) @ 99f3fc1` — IPC adapter, one `invoke` per batch; doc :740-755 cites 57–64 ms parse for 9,850 rows. `src/config/segment.ts:280 (SegmentTargetEdit)` wire type.

## Q2
- `editorEditedText`, `editorEditedShape`, `editorTmFilledSegmentIds`, `editorFlushError`: no reader in area.
- `editorLastSavedAt`: `src/StatusBar.vue:85 (now)`, `:88-93 (onMounted)` 1 s `setInterval` writing `now` only after `lastSavedAt` non-null; `src/StatusBar.vue:279 (secondsSinceSave)` computed; template last `v-else-if` in `role="status"` ~`:610`.
- Notices: `src/StatusBar.vue:308 (confirmNoticeKey)`, `:361 (regroupNoticeText)`, `:459 (navNoticeKey) @ 99f3fc1` computeds; `v-if`/`v-else-if` chain ~`:532-610`; comment `:540-555` Backspace merge bypasses `noteEditorEdit`.
- Pending set (imperative via `flush.isDirty`): `src/main.ts:1242 (boot)` `wireExitFlush()`; `src/modes/libraryImport.ts:407 (beginSubmit)` `flushEditorNow()`; `src/modes/libraryChapters.ts:313 (openWorkById)`, `:484 (beginChapterReorg)` `flushEditorBeforeDiscreteWrite()`; `src/proofreadState.ts:182 (runProofread)`, `:442 (acceptProofreadFinding)`, `:497 (revertProofreadFinding)`, `:588 (ignoreProofreadFinding)`; `src/reviewModeState.ts:370 (runAccept)`.
- `editorSegments` readers (load snapshot, not written by typing): `src/StatusBar.vue:154 (positionInChapter)`, `:179 (confirmedCounterText)` via `toRaw`; `src/proofreadState.ts:325 (editorTextOf)`, `:350 (proofreadCurrent)`; `src/GlossaryConfirmStrip.vue:60 (watch)` `[caret, editorSegments, glossaryMarks]`; `src/tmConcordanceCommandDeps.ts:26 (watch)`.

## Q3
- `GridBlock`, `v-memo`: no hits in area. Mount: `src/layout/WorkspaceDock.vue:103 (components) @ 99f3fc1` `grid: GridPanel` via dockview; `src/layout/panelProps.ts (DockviewPanelProps)` one prop `params`; `src/modes/WorkspaceMode.vue:171 @ 99f3fc1` dock hidden with `visibility`, never `v-if`.

## Q4
- Typing zone: `keys.ts:439`, `:510`; `src/commands/README.md:77 @ 99f3fc1` rule; `src/commands/index.ts:3163 (registerAll) @ 99f3fc1` "ô bản dịch của lưới LUÔN `contenteditable`".
- Status bar: notice clear + 1 s ticker `src/StatusBar.vue:76-93 @ 99f3fc1`; comment weighs one-text-node update vs `v-for` over 9,850 `<span>`s.
- Caret readers (re-evaluate on `editorCaretSegmentId` change; `setEditorCaret` from `GridPanel.vue:1239 onSelectionChange`): `src/TmFuzzyStrip.vue:30` (`syncTmFuzzyStrip`), `src/ProofreaderStrip.vue:40` (`syncProofreadStripHidden`), `src/GlossaryConfirmStrip.vue:60`, `src/inlineStripEligibility.ts:29 (eligibleInlineStrips)`, `:53 (proofreaderStripPlace)`, `:61 (inlineStripAnchorSegmentId)`, `src/proofreadScope.ts:17 (proofreadScopeIds)`, `src/proofreadState.ts:276 (proofreadStripHidden)`, `:350`, `:375 (proofreadRevertible)`, `:380 (proofreadActionError)`, `src/AiPromptInspectorOverlay.vue:112 (isStale)`, `src/StatusBar.vue:154`.
- Proofread/rule markers: none in area; `src/proofreadState.ts:394 (shiftProofreadFindings)` only after accept/revert.

## Q5
- `src/AGENTS.md:6 @ 99f3fc1` — no business rule in TS (AD-1) except text being typed in the Editor.
- `src/AGENTS.md:29 @ 99f3fc1` — Editor flush cadence 2000/5000, not merged with others.
- `src/StatusBar.vue:76-84` render-cost reasoning; `src/config/segment.ts:740-755` one IPC per batch (AC13).
- Nothing in area on block split.

## Q6
- No test files in area; `src/selftest/` nothing on typing.

## Searched, nothing found
- Reads of `editedText|editedShape|lastSavedAt|pendingFlush|flushPending|pendingSet|markDirty` and exported names; `GridBlock|v-memo`; `addEventListener('beforeinput'|'input'|'selectionchange')` — only `keys.ts:607` keydown capture and `TitlebarToolsMenu.vue:69` mousedown; `repaint|CSS.highlights|ruleMarker`; test files.
