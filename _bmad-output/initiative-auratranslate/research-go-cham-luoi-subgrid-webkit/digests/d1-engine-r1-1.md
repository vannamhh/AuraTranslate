# D1 engine mechanics, r1-1 (web only, accessed 2026-10-23; 8 searches/fetches, source code NOT read)

## Claims
- WebKit bug 304869 "[Grid] Lots of time spent in GridTrackSizingAlgorithm::accumulateIntrinsicSizesForTrack with subgrids that span lots of rows": RESOLVED FIXED, commit 305121@main, landed 2026-01-05 (so a WebKit build before ~Jan 2026 lacks it; WebKit 605.1.15 on macOS 15 almost surely does) | https://www2.webkit.org/show_bug.cgi?id=304869 | WebKit Bugzilla | 2026-01-02 | accessed 2026-10-23 | high (fix content not described on page) | bug-status
- WebKit bug 302529 "Nested grids performance degradation": subgrids for rows x 12 columns nested ~3 deep made Safari unresponsive on M1 (worse on Intel); Chrome/Firefox fine; workaround = disable native subgrid via @supports + JS sizing; a commenter says it looks fixed by 305121@main | https://bugs.webkit.org/show_bug.cgi?id=302529 | WebKit Bugzilla | 2025-11-14 (mod 2026-01-13) | 2026-10-23 | medium (comment by a third party; no timing numbers) | bug-status / engine-behavior
- Bug 232176 "Improve performance of nested grids" marked duplicate of 304869; a search snippet says users still saw extreme lag with many nested grids in Safari 17 | https://www2.webkit.org/show_bug.cgi?id=232176 (fetch 503, snippet only) and 304869 page | WebKit Bugzilla | 2021 / dup 2026-01-20 | 2026-10-23 | low-medium | bug-status
- Track sizing in WebKit hot spot for subgrids spanning many rows is intrinsic-size accumulation per track (function name in bug title) ⇒ consistent with, but does not prove, "full track sizing re-run over all items" | bug 304869 title | WebKit Bugzilla | 2026-01-02 | 2026-10-23 | medium | engine-behavior
- Layout containment helps when a small part of the page changes, little when most is affected; relayout cost of getBoundingClientRect barely changes with containment in the large-grid test (bug 233628 discussion) | https://bugs.webkit.org/show_bug.cgi?id=233628 | WebKit Bugzilla | 2021 | 2026-10-23 | low-medium (secondhand summary, old) | engine-behavior
- Bug 275394: commenter claims sites using containment aggressively suffer a (layout) issue, with offsetHeight benchmark on a large grid; details not read | https://bugs.webkit.org/show_bug.cgi?id=275394 | WebKit Bugzilla | ~2024 | 2026-10-23 | low | bug-status
- Spec: css-grid-2 says if a grid container must establish an independent formatting context (e.g. layout containment), `subgrid` computes to `none` ⇒ `contain: layout` on a subgrid makes it a normal grid, not a subgrid | via csswg-drafts #7091 summary | https://github.com/w3c/csswg-drafts/issues/7091 | W3C CSSWG | opened 2022-02-25 | 2026-10-23 | medium (fetched via summarizer, spec text not read directly) | spec
- Spec: size containment on a subgrid lays it out as if empty; spec text still "Needs Edits" to say grid items don't contribute to parent track sizing (fantasai 2022-10-03; label moved 2024-08-26) | same URL | W3C CSSWG | 2022-2024 | 2026-10-23 | medium | spec
- WebKit bug 232673: typing in contenteditable slow with default focus outline, lag grows with document size, DevTools shows time as "Other"; workaround `outline: none`; NEW, no numbers | https://bugs.webkit.org/show_bug.cgi?id=232673 | WebKit Bugzilla | 2021-11-03 | 2026-10-23 | medium (Safari 15; may be stale) | bug-status
- Blink GridNG (Chrome 93, 2021) was built on LayoutNG with improved caching and better performance in key scenarios | https://blogs.windows.com/msedgedev/?p=24831 | Microsoft Edge blog | 2021 | 2026-10-23 | medium (snippet; says nothing specific on subgrid) | engine-behavior
- ProseMirror forum: Chrome/macOS very low frame rate with many DOM nodes in a focused contenteditable; workaround `content-visibility: auto` on wrapper elements; one commenter said Safari fine | https://discuss.prosemirror.net/t/performance-issues-with-prosemirror-and-chrome/2498 | ProseMirror forum | ~2020 | 2026-10-23 | low (forum, old, Chromium) | perf-number-free

## Leads
- Read the diff of 305121@main (github.com/WebKit/WebKit PR #56024; commit 425f9293f8d4) and test case attached to bug 304869: shows what was O(n^2) and whether a fixed WebKit nightly (Safari Technology Preview / WebKit nightly) removes the 0.45 s relayout. Highest value: rerun the static-DOM copy in STP; also check whether the system WKWebView version on the target macOS has the fix.
- Bug 232176 comments (retry; 503) and 275394 and 233628 for numbers.
- RenderGrid.cpp / GridLayoutFunctions for whether a subgrid child marks the parent grid for full relayout (dirty-propagation: subgrid items force the outer grid to re-run track sizing).
- csswg-drafts #7091 / css-grid-2 text direct; Chromium subgrid (shipped Chrome 117) perf bugs on crbug not searched.
- Workaround direction from bug 302529: avoid native subgrid (row-major, explicit row heights) on WebKit.

## Looked for, not found
- Any source stating WebKit TypingCommand/insertText forces synchronous layout per keystroke (not searched in source; no blog/bug found).
- Any independent source with numbers for "one grid-item change re-lays out the whole grid" (the two-source rule is NOT met).
- Blink subgrid-specific layout-cache behavior; crbug subgrid perf reports.
- content-visibility: auto behavior on subgrid items / grid items.
- Changing border width causing grid relayout: no source.
