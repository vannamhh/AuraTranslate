# D2 layouts digest r1-1 (web research only; 9 tool calls used before writing)

## Claims
- WebKit subgrids spanning many rows spent most time in GridTrackSizingAlgorithm::accumulateIntrinsicSizesForTrack; fixed in 305121@main, landed 2026-01-05 (no timings given) | https://www2.webkit.org/show_bug.cgi?id=304869 | WebKit Bugzilla (bug 304869) | 2026-01-02 | accessed 2026-10-23 | medium (fetched via www2 mirror; bugs.webkit.org 503) | engine-behavior
- Nested-grid slowness in Safari (rows x nested levels; reporter: ~12 month cols, 3 nesting levels) made page unresponsive on M1 mini; Chrome/Firefox fine; bug 302529 still NEW as of 2026-01-13, comment says fix appears to be 305121@main. Reporter workaround: @supports-detect Safari, disable native subgrid, JS-resize | https://bugs.webkit.org/show_bug.cgi?id=302529 | WebKit Bugzilla | 2025-11-14 | accessed 2026-10-23 | high | engine-behavior
- Older nested-grid perf bug 232176 closed as duplicate of 304869; a comment links 276633@main as an earlier improvement (100+ item test OK on M1 Pro) | https://www2.webkit.org/show_bug.cgi?id=232176 | WebKit Bugzilla | 2021 | accessed 2026-10-23 | low (seen only in search snippet) | engine-behavior
- WebKit treats layout containment (contain: layout) as a relayout boundary so layout can start at the subtree root instead of RenderView; one engineer credited a large speedup on a profiled case. Benefit is small when a change touches most of the page. Not an inline-formatting-context exception (2025 bug) | https://bugs.webkit.org/show_bug.cgi?id=227948 (and trac.webkit.org/wiki/CSSContainment) | WebKit Bugzilla / WebKit Trac | 2013-2025 | accessed 2026-10-23 | low-medium (search snippets only, not opened) | engine-behavior
- 10,000-row x 4-col table: ~950 ms render without CSS contain vs ~500 ms with; 40,000 buttons ~11 s vs ~500 ms. M1 Pro, browser not stated, render (not keystroke) cost | https://coryrylan.com/blog/high-performance-html-tables-with-lit-and-css-contain | Cory Rylan (personal blog) | 2023-08-31 | accessed 2026-10-23 | low (single source, browser unstated) | perf-number
- content-visibility:auto: skipped content lays out as empty (zero height) so scrollbar shifts unless contain-intrinsic-size set; `contain-intrinsic-size: auto <len>` remembers last rendered size; elements with focus or selection in subtree are not skipped; off-screen content remains findable. Travel-blog example 232 ms -> 30 ms initial render | https://web.dev/articles/content-visibility | web.dev (Google) | 2020-08-05, updated 2025-09-23 | accessed 2026-10-23 | high | engine-behavior / perf-number (Blink-centric)
- CSSWG discussion confirms auto-size remembering is limited to when the element skips contents; remembered size can be stale | https://lists.w3.org/Archives/Public/public-css-archive/2021Jan/0236.html (csswg-drafts #5668/#6308) | W3C CSSWG | 2020-2022 | accessed 2026-10-23 | low (snippet) | spec
- WebKit bug 244445: attempt to confine selection to one table column via :has() + user-select worked basically but selection highlight vanished after mouse-up in Safari; no resolution seen | https://bugs.webkit.org/show_bug.cgi?id=244445 | WebKit Bugzilla | ~2022 | accessed 2026-10-23 | low (snippet only) | engine-behavior
- AG Grid offers enableCellTextSelection + ensureDomOrder to mimic table-like native selection, i.e. selection follows DOM order and libraries must keep DOM order == visual order | https://ag-grid.com/react-grid/selection-overview/ | AG Grid docs | undated | accessed 2026-10-23 | low-medium | pattern
- Confluence bug: large tables (~2000 rows x 10 cols) in an editor still slow after optimization -> editable tables need virtualization/paging | https://jira.atlassian.com/browse/CONFCLOUD-84158 | Atlassian | n/d | accessed 2026-10-23 | low | pattern

## Leads
- Bug 304869 fix (305121@main, Jan 2026) may directly cure the subgrid-spanning-many-rows cost: determine which Safari/WKWebView (macOS 26.x) first ships it; the user's WKWebView version tracks the OS, so a Tauri app cannot ship its own engine. Test on a Safari Technology Preview build with the 9,850-row grid.
- Open https://bugs.webkit.org/show_bug.cgi?id=227948, 288421, 290359, 270219 for the relayout-boundary / contain:layout details and numbers.
- WebKit blog posts on "relayout boundary" / contain support versions; check Safari version where contain:layout became a boundary.
- Chromium issue tracker for column-confined selection and contenteditable in content-visibility.
- Glide Data Grid / AG Grid / Notion / Linear engineering posts for production numbers (not reached).

## Looked for, not found
- Q1: any measured numbers comparing row-major flex/grid/table rows vs column-subgrid for a single-row content change in WebKit or Blink. Nothing found.
- Q2: evidence that chunking into 50-200-row grids bounds relayout, with numbers; only the generic containment-boundary statements above.
- Q2: content-visibility interaction with selection spanning skipped chunks and with contenteditable/IME/native undo; only "selection subtree is not skipped" (web.dev). Also WebKit's content-visibility behaviour not verified.
- Q3: table vs grid/flex performance in WebKit, and table-layout:fixed effect. Nothing authoritative.
- Q4: authoritative statement that row-major breaks column-wise selection and a working user-select technique with limits (only the WebKit 244445 anecdote and AG Grid ensureDomOrder).
- Two independent sources for any perf number: not achieved.
