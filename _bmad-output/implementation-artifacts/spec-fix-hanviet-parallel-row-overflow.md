---
title: 'Fix: Hán Việt parallel view overflows its grid row and shows a per-cell scrollbar'
type: 'bugfix'
created: '2026-09-25'
status: 'done'
route: 'dispatch'
baseline_commit: 'a9464f4a63dc17d66daecf99f4707bfbd34bac96'
review_loop_iteration: 0
context:
  - '{project-root}/src/AGENTS.md'
  - '{project-root}/e2e/AGENTS.md'
---

<frozen-after-approval reason="human-owned intent — do not modify unless human renegotiates">

## Intent

**Problem:** In the grid's Hán Việt tab, parallel view (`<ruby>` readings under each word), a source cell grows its own vertical scrollbar, and its last line draws over the next row (Ice's screenshot, 2026-09-25). Measured in WKWebView on 2-segment Chinese text:
1. The last line's `rt` (`ruby-position: under`) sticks out about 4 px below the content box (`scrollHeight` 387 vs `clientHeight` 383).
2. `.hv-surface { flex: 1; min-height: 0; overflow: auto }` (`src/panels/SourceHanViet.vue:994`), left over from when this component was a standalone panel, turns those 4 px into a 15 px scrollbar inside the cell.
3. The narrower text box wraps one extra line after the row track was already sized, so the row stays 388 px while the content needs 433 px.

With that rule neutralised, 5 of 5 cycles measured 388/389. Its own doc comment (`:986-992`) already says that no branch may own a scroll box and that `.grid-scroll` is the only one.

**Approach:** Keep `subgrid` (Ice, 2026-09-25: option C over dropping subgrid). Remove the stale scroll-box rule, make the block's layout height include the last line's under-annotation, and lock both with an e2e case in the real webview.

## Boundaries & Constraints

**Always:**
- Only `.grid-scroll` scrolls. No element inside a grid cell has `overflow` `auto`/`scroll`/`hidden` in any tab or view mode.
- The annotation space is included by fixing the box that actually overflows (measure which one first). A hard-coded pixel compensation is allowed only when it is expressed through the token that sets that size.
- The `@copy` handler on `.hv-surface` and the selection-surface registration on the text paragraph (`:482-488`, `:788`) keep working as they do today.

**Never:**
- Touch the `subgrid` layout of `GridPanel.vue`, `selectionContract.ts` or `hanVietSurfaces.ts`.
- Change the reading font, line-height tokens or word spacing (`rt { padding-inline }`).
- Mistake this bug for the accepted "parallel rows are tall by design" item (`deferred-work.md:2669`, ballot #88). Height by design is fine; content taller than its row is the bug.

## I/O & Edge-Case Matrix

| Scenario | Input / State | Expected |
|---|---|---|
| Parallel view, first entry | Hán Việt tab → toggle to parallel | every source cell: height ≥ its content's lowest descendant rect; no row overlaps the next |
| Toggle cycles | parallel → switch → parallel, 3 times | same as above on every cycle |
| Always-visible scrollbars | `.grid-scroll { overflow-y: scroll }` injected | same as above |
| No inner scroll box | any view mode | computed `overflow-y` of every element inside a `[data-col]` cell is `visible` (holds even on overlay-scrollbar machines such as CI) |

</frozen-after-approval>

## Code Map

- `src/panels/SourceHanViet.vue:994-998` -- `.hv-surface` rule to remove; `:980-993` its doc comment (delete the lines that describe the removed rule); `:835` the element; `ruby`/`rt` rules at the end of `<style>`; `.hv-parallel` sets `line-height: var(--leading-source-cjk)`.
- `src/panels/GridPanel.vue:1996-2025` -- `.grid`/`.col` (`subgrid`)/`.cell` (`padding: 2px 8px`, `.para-end` bottom 14px). Read only.
- Probe that reproduced the bug and measured the fix: `/private/tmp/claude-501/-Users-hoangnam-LocalSites-addon-AuraTranslate/24a8a135-0bf6-4768-9b05-fe8e40766895/scratchpad/probe/hv-overflow.e2e.mjs` (text, `measure()`, `cycle()`); results in `result*.json` there. Reuse its fixture text and measurement.
- `e2e/support/workspace.mjs::openWorkspaceWithWork` (creates a `zh` Work), `gridWait.mjs::waitForGridRows`, `pointer.mjs::realClick`; tab ids `#grid-tab-han-viet`, toggle `.view-toggle`.

## Tasks & Acceptance

**Execution:**
- [x] `src/panels/SourceHanViet.vue` -- delete the `.hv-surface` scroll-box declarations; find the overflowing box of the last line's `rt` and make its layout height include the annotation.
- [x] `e2e/specs/hanviet-parallel-row-height.e2e.mjs` (new) -- one case per matrix row.

**Acceptance Criteria:**
- Given the new e2e file, when it runs with `.hv-surface { overflow: auto }` restored, then it goes red for the height or overflow reason; with the fix, it goes green.
- Given the app, when Ice opens a long Chinese Chapter in the parallel view, then no cell shows a scrollbar and no reading line overlaps the next row. This is a real-use check with owner `Chủ: Ice`, recorded as a debt item unless Ice runs it before `done`.

## Implementation Notes

- Measured live (WKWebView, narrow-forced `.grid` columns) that deleting `.hv-surface`'s `flex: 1; min-height: 0; overflow: auto` alone closes the gap (90 px/102 px overflow pre-fix → −13 px/−1 px margin post-fix); the "include the rt annotation in layout height" half of the task needed no separate padding/token hack — it falls out of restoring correct subgrid intrinsic sizing once the automatic-minimum-size-0 trigger (`overflow: auto`) is gone.
- The bug is transition-dependent, not just width-dependent: it only manifests at the switch→parallel toggle itself. A later, unrelated style recalc (e.g. forcing `.grid-scroll { overflow-y: scroll }` *after* the toggle) silently avoids it even on unfixed code. The new e2e spec applies its narrow-column (and scrollbar-gutter) CSS *before* every toggle into parallel; the first draft applied the scrollbar rule *after* the toggle instead, and that one case passed on both the buggy and fixed source (a guard that can't fail guards nothing).
- Counter-check ran twice each way: `.hv-surface { overflow: auto }` restored → 4/4 cases red (height/overlap ×3, `overflow-y !== 'visible'` ×1); fix restored → 4/4 green, repeated once more to rule out the flake this bug class is prone to.
- `PanelFrame.vue`'s doc comment said an earlier fix leaves `.hv-surface`'s `flex: 1` untouched. That stopped being true once the rule was removed, so the sentence was replaced, with no marker, as AGENTS.md prescribes for code.
- AC2's real-use pass (open a long Chinese Chapter, eyeball it) is unrun by a human; recorded as a debt item, `Chủ: Ice` (`deferred-work.md`, `## Deferred from: fix-hanviet-parallel-row-overflow (2026-09-25)`).

## Verification

**Commands:**
- `npm run test:e2e -- --spec e2e/specs/hanviet-parallel-row-height.e2e.mjs` -- expected: green; red against the counter-check. Ran: green ×2 (fresh build), red ×1 (counter-check, `.hv-surface{overflow:auto}` restored, 4/4 cases).
- `npm run check:tokens && npm run check:i18n` -- expected: green. Ran: both green.

## Review Triage Log

- edge: the 3× toggle case asserted before each pair of toggles, so the last switch→parallel transition went unmeasured — low, patch: one more assert after the loop.
- edge: the task claims the last line's annotation height is now included, but the diff only removes the scroll-box rule — false: re-ran the e2e file with `TOLERANCE_PX = 0` and got 4/4 green, so no overhang remains. The tolerance stays at 1 px for sub-pixel rounding.
- blind: the unedited line in the same `PanelFrame.vue` doc comment cites `GridPanel.vue:1408`, which is stale — low, patch: dropped the line number.
- blind: a 🔵 sits mid-sentence in Implementation Notes — low, patch: reworded without the marker.
- blind: the Code Map ranges and values are stale (`:980-993`, `.cell` `2px 8px`, `.para-end` 14px; lot C rounded these to 4px/16px) — rejected: the fix is a spec edit.
- blind: case 4 never injects the narrow-column CSS — false: it asserts computed `overflow-y`, which does not depend on column width.
- verification-gap: no gaps found. Other: the stale `GridPanel.vue:1408` pointer — same as the blind finding above, patched.
