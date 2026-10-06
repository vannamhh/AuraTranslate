---
id: 1
type: bug
title: "Hán Việt parallel view overflows its grid row and shows a per-cell scrollbar"
parent: none
covers: []
after: []
assignee: ""
refined: true
hitl: false
risk: low
severity: P2
---

# Hán Việt parallel view overflows its grid row and shows a per-cell scrollbar

## Description

In the grid's Hán Việt tab, parallel view (`<ruby>` readings under each word), a source cell grows its own vertical scrollbar, and its last line draws over the next row (Ice's screenshot, 2026-09-25). Height by design is fine ("parallel rows are tall by design", `deferred-work.md` ballot #88); content taller than its row is the bug.

## Reproduction

Measured in WKWebView on 2-segment Chinese text, Hán Việt tab, toggle to parallel view:

1. The last line's `rt` (`ruby-position: under`) sticks out about 4 px below the content box (`scrollHeight` 387 vs `clientHeight` 383).
2. `.hv-surface { flex: 1; min-height: 0; overflow: auto }` (`src/panels/SourceHanViet.vue:994`), left over from when this component was a standalone panel, turns those 4 px into a 15 px scrollbar inside the cell.
3. The narrower text box wraps one extra line after the row track was already sized, so the row stays 388 px while the content needs 433 px.

The bug shows at the switch-to-parallel toggle itself, not on a later style recalculation.

## Cause Hypothesis

The stale scroll-box rule on `.hv-surface` gives the cell an inner scroll box, and the last line's under-annotation is not part of the block's layout height. With that rule neutralised, 5 of 5 cycles measured 388/389. The component's own doc comment already says no branch may own a scroll box and `.grid-scroll` is the only one.

## Acceptance Criteria

1. **No cell overflows its row**
   **Given** the Hán Việt tab in parallel view on Chinese text
   **When** the view is toggled in, three times in a row, and again with `.grid-scroll { overflow-y: scroll }` applied before the toggle
   **Then** every source cell is at least as tall as its content's lowest descendant, no row overlaps the next, and the computed `overflow-y` of every element inside a `[data-col]` cell is `visible`
2. **Only `.grid-scroll` scrolls**
   **Given** any tab or view mode
   **When** the grid renders
   **Then** no element inside a grid cell has `overflow` `auto`, `scroll` or `hidden`
3. **The e2e case can fail**
   **Given** the new e2e file `hanviet-parallel-row-height.e2e.mjs`
   **When** it runs with `.hv-surface { overflow: auto }` restored
   **Then** it goes red for the height or overflow reason, and green with the fix
4. **Real-use check**
   **Given** the app
   **When** Ice opens a long Chinese Chapter in the parallel view
   **Then** no cell shows a scrollbar and no reading line overlaps the next row (owner `Chủ: Ice`)

## Boundaries

- Must not change: the `subgrid` layout of `GridPanel.vue`, `selectionContract.ts`, `hanVietSurfaces.ts`, the reading font, line-height tokens or word spacing; the `@copy` handler and selection-surface registration on `.hv-surface` keep working.

## References

- parent — none
- plan — bug-fix-hanviet-parallel-row-overflow-plan.md
- Ice, 2026-09-25: keep `subgrid` (option C over dropping it)
