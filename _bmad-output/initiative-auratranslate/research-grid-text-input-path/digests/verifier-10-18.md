# Verifier digest — refs 10–18 @ 99f3fc1

- [10] src/panels/GridBlock.vue:311-319 (editorEditedShape) @ 99f3fc1 — verified (memo array as claimed).
- [11] src/panels/GridBlock.vue:317 (editorEditedShape) @ 99f3fc1 — verified. `editorEditedShape` is module-level shallowRef (editorPanelState.ts:686-688) imported at GridBlock.vue:9; memo evaluated for every row of `v-for="(s, j) in block.segments"`; `:328` reads again on miss; no per-block guard (target column `:271`, root `:99` no `v-if`/`v-memo`; GridPanel.vue:1918-1926 bare `v-for`). Trigger only on shape change (`:793`).
- [12] src/panels/GridPanel.vue:1615-1620 (onEditKeydown) @ 99f3fc1 — verified; `:1614` early return on `isComposing`, then no key filter.
- [13] src/panels/editorPanelState.ts:292-295 (setEditorCaret) @ 99f3fc1 — verified.
- [14] src/panels/proofreadHighlight.ts:117-128 (repaintProofreadHost) @ 99f3fc1 — disputed. Wrap half: loops result entries; skips when no `[data-col="tgt"][data-segment-id=id]` cell (`:125`); `wrapProofread` returns when `textContent !== scannedText` or no marks (`:85`). Unwrap half wrong: `unwrapAllProofread(host)` (src/panels/proofreadHighlight.ts:102-109 (unwrapAllProofread) @ 99f3fc1) runs `querySelectorAll` over proofread spans (`PROOFREAD_SPAN_SELECTOR` = `.proofread-error, .proofread-meaning`), collects each span's `closest('[data-segment-id]')`, unwraps only cells that contain spans.
- [15] src/panels/GridPanel.vue:1325-1332 (onSelectionChange) @ 99f3fc1 — verified; `onMounted` adds, `onBeforeUnmount` removes both.
- [16] tests/frontend/gridTypingCost.test.ts:58-67, :108-112 (type) @ 99f3fc1 — verified.
- [17] tests/frontend/gridBlockRenderCost.test.ts:74-82, :99-165 (updated) @ 99f3fc1 — verified; only GridBlock render counter under tests/ and e2e/ (`git grep` `updated(`, `mixins`, `renderTracked`, `onRenderTriggered`, `Renders`).
- [18] e2e/specs/editor-typing-flush.e2e.mjs:38-55 (execCommand) @ 99f3fc1 — verified; uses at `:164`, `:275-277`; comment `:48` says `browser.keys()` synthesizes keydown/keyup while table `:44` shows only keydown observed.
