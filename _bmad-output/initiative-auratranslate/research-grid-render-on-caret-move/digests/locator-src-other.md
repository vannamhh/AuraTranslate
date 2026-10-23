Digest for the area src/commands, src/config, src/i18n, src/layout, src/modes, src/selftest and src/tokens at cc60020. Tree B changes nothing here: `git diff --stat cc60020 b2874aa` over these paths is empty, so everything below holds for both trees.

**Headline:** nothing in this area reads `editorCaretSegmentId`, `editorLastCaretSegmentId` or `stripAnchorIndex`, and nothing here calls `setEditorCaret` directly. Code in this area moves the caret only indirectly, through `editorPanelState` functions. The components that host GridPanel have no caret state among their render dependencies.

## Q1 / Q4 — what writes the caret (indirectly, from this area)

**Keyboard path (commands registry).** Handlers are injected as dependencies; the real functions are wired outside this area.
- `src/commands/index.ts:171 (CommandDeps) @ cc60020` defines the dependency ports, including `goToNextUntranslated` (:580) and `goToNextSegment`/`goToPrevSegment` (:594/:596).
- `src/commands/index.ts:3310 (registerAll) @ cc60020`: `editor.next_untranslated`, default key `Mod+Alt+ArrowDown`, calls `deps.goToNextUntranslated()`.
- `src/commands/index.ts:3361 (registerAll) @ cc60020`: `editor.next_segment`, no default key, calls `deps.goToNextSegment()`.
- `src/commands/index.ts:3373 (registerAll) @ cc60020`: `editor.prev_segment`, no default key, calls `deps.goToPrevSegment()`.
- `src/commands/index.ts:3428 / :3441 (registerAll) @ cc60020`: `editor.next_chapter` / `editor.prev_chapter`, call `deps.goToNextChapter` / `deps.goToPrevChapter`.
- `src/commands/index.ts:3244 (registerAll) @ cc60020`: `editor.confirm_segment`, key `Mod+Enter`, calls `deps.confirmSegment`.
- `src/commands/index.ts:3181 (registerAll) @ cc60020`: `segment.selection.extend_down` / `extend_up` (`Mod+Shift+ArrowDown/Up`). These change the multi-segment selection, not the caret.
- `src/commands/keys.ts:587 (attachKeymap) @ cc60020` adds a capture-phase `keydown` listener.
- `src/commands/keys.ts:439 (isTypingZone) @ cc60020` and `:417 (lacksPrimaryMod)` gate chords that lack a primary modifier inside typing zones. Bare-arrow caret movement inside the grid is therefore not a registry command here.
- The bodies these ports point to are in `src/panels/editorPanelState.ts` (outside this area). `goToNextSegment` at :1787 and `goToPrevSegment` at :1796 call `doiConTroToi`, and `moveEditorCaretTo` is at :1778. `setEditorCaret` (:281) writes `caretSegmentId` and `lastCaretSegmentId`, calls `flushEditorNow()` when the caret moves from segment A to segment B, and calls `positionFlush.markMoved`.

**Command / navigation paths in `src/modes`.** These go through `openChapterById(chapterId, targetSegmentId?)`, which moves the caret to the target segment via `doiConTroToi` (doc comment at `editorPanelState.ts` ~:2180).
- `src/modes/librarySearch.ts:574 (openCurrentLibrarySearchHit) @ cc60020`: `openChapterById(hit.chapter_id, hit.segment_id)`.
- `src/modes/readingState.ts:255 (openAimedReadingSegment) @ cc60020`: `openChapterById(found.chapterId, segmentId)`, then `requestCurrentEditorCaretPlacement()` at :262.
- `src/modes/readingState.ts:267 (openSearchHitInReading) @ cc60020`: `openChapterById` with a segment.
- `src/modes/readingState.ts:389 (openWalkMark) @ cc60020` and `:447 (openCurrentReadingMark)`: `openChapterById(mark.chapter_id, mark.navigation_segment_id)`, plus `requestCurrentEditorCaretPlacement()` at :396 and :454.
- `src/modes/readingState.ts:430 (openReadingWorkStart)`, `:496 (openFrontierInWorkspace)`, `:576 (openCurrentTocChapter)` and `src/modes/libraryChapters.ts:465 (openCurrentChapter) @ cc60020`: `openChapterById` without a segment id, so the caret comes from Rust's saved position.
- `src/config/segment.ts:201 (ChapterSegments) @ cc60020`: the `caret_segment_id` wire field that seeds the caret on chapter load; validated at :264 (`isChapterSegments`).
- Other imports from `editorPanelState` here read only `editorChapterId`, not the caret: `src/modes/libraryChapters.ts:48`, `src/modes/libraryWorks.ts:33`, `src/modes/libraryImport.ts:45`.

**Mouse path.** None in this area. Mouse-driven caret setting lives in `src/panels`.

## Q1 / Q4 — what hosts GridPanel and its render dependencies

**`src/modes/WorkspaceMode.vue:137 (template) @ cc60020`** renders `<WorkspaceDock :saved-layout="bootstrapLayout">`. Its template reads only:
- `currentTier`, a local `shallowRef` (:79, :116);
- `reviewNotice` / `reviewNoticeOffersImport`, computed from `reviewModeStatus` (:85, :103);
- `reviewModeIsOpen` (:136, :139);
- `bootstrapLayout`.

None of these is caret state. The Workspace dock is hidden with `visibility`, not `v-if` (:171–188). `onActivated` (:46) suggests the mode is kept alive by its parent, which is outside this area.

**`src/layout/WorkspaceDock.vue:103 (components) @ cc60020`** maps `grid: GridPanel`. Its template (:1144–1157) reads only `currentTier` (:519), `currentPresetId` (:531) and `tierMerged` (:535), all `shallowRef`s. It passes the static `auraTheme`, `components` and `tabComponents` to `<DockviewVue>`. `addPanel` (:213) passes `params: { titleKey }` only, so no caret value travels through panel params.

**How dockview-vue 7.0.4 mounts GridPanel** (`node_modules/dockview-vue/dist/dockview-vue.es.js`, read for context, outside this area). Panels are mounted through `VueRendererRegistry` (:56): a `shallowReactive` `entries` array, with each entry holding `props` in a `shallowRef`. The `dockviewPortals` component (:304–320) renders each entry as `<Teleport :to="entry.target">` around `resolveDynamicComponent(entry.component)` with `entry.props.value`. That component is a child of the `dockview` component's Fragment (:645–656). So GridPanel stays inside WorkspaceDock's Vue component tree. It re-renders from above only when `entries` or `entry.props.value` change, which happens through `VueRenderer.update` (:126–138) on dockview param updates. There is also a fallback `mountVueComponent` (:31), a detached `render()` root, used only when no registry exists.

**Other layout pieces.** `src/layout/panelProps.ts:27 (DockviewPanelProps) @ cc60020` defines the single `params` prop shape GridPanel receives. `LookupDrawer.vue` and `ReviewDock.vue` do not host GridPanel.

## Q2 / Q3 / Q5

Nothing in this area. There is no `v-for`, `v-memo` or `subgrid` relating to the grid. `src/tokens/reset.css:80 @ cc60020` holds only the shared `.cut-mark` shape that GridPanel uses (CSS, not a reactive read).

## Q6

There are no test files in this area at cc60020: `git ls-tree` shows no `*.test.*` or `*.spec.*` under these paths.

## Seen in passing outside this area (other locators own these)
- `src/chapterSplitState.ts:80 (confirmChapterSplit) @ cc60020` calls `setEditorCaret(pickedId.value)`.
- Caret readers in `src/` root: `src/StatusBar.vue:156`, `src/inlineStripEligibility.ts:34/57/66/69` (`inlineStripAnchorSegmentId` and others), `src/GlossaryConfirmStrip.vue:61`, `src/ProofreaderStrip.vue:40`, `src/TmFuzzyStrip.vue:30`, `src/AiPromptInspectorOverlay.vue:112`, `src/aiTranslateHandlers.ts:45/77`.
- `stripAnchorIndex` exists only in `src/panels/GridPanel.vue` (:601 computed; read at :1808, :2062, :2063).

## Searched, nothing found (cc60020, this area)
`git grep -E 'setEditorCaret|editorCaret|stripAnchor|caretSegment|LastCaret|moveEditorCaretTo|goTo(Next|Prev)|inlineStrip'` over src/commands, src/config, src/i18n, src/layout, src/modes, src/selftest and src/tokens. The only hits were the `deps.goTo*` ports in `src/commands/index.ts` and the `editorPanelState` imports listed above. There are no `v-memo`, `subgrid` or caret references in src/i18n, src/selftest or src/tokens.
