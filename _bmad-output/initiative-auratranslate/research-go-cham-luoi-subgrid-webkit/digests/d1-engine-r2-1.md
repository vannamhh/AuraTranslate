# d1-engine-r2-1 — WebKit subgrid fix 305121@main vs local WebKit (accessed 2026-10-23)

## Claims
claim | URL | publisher | pub_date | accessed | confidence | class
- Bug 304869 "Grid performance with subgrids spanning many rows": RESOLVED FIXED, filed 2026-01-02, committed 2026-01-05 as 305121@main (425f9293f8d4); 232176 (nested grids) closed as its duplicate 2026-01-20 | https://bugs.webkit.org/show_bug.cgi?id=304869 | WebKit Bugzilla | 2026-01-02..20 | 2026-10-23 | high | primary
- Bug 302529 "Nested grids performance degradation" (Safari 26, filed 2025-11-14) is still NEW, no resolution as of last mod 2026-01-13; a commenter (2026-01-13) says it "seems to have been fixed in" 305121@main but bug not closed; reporter: page unresponsive on M1 with native subgrid, Chrome/Firefox fine; no @supports to detect fix | https://bugs.webkit.org/show_bug.cgi?id=302529 | WebKit Bugzilla | 2025-11-14..2026-01-13 | 2026-10-23 | high (status as fetched; page may have changed since) | primary
- Fix = rewrite of GridTrackSizingAlgorithm::accumulateIntrinsicSizesForTrack: recursion that re-walked a subgrid once per spanned row replaced by one iterative pass over grid items (SizingData stack); benchmark PerformanceTests/Layout/subgrids-spanning-many-rows.html = 12 subgrid items x 20000 rows, toggling body width; M4 Max ~0.20 -> ~50 runs/s; author says other engines still faster, "starting point". Touches 4 files, +82/-28 | https://github.com/WebKit/WebKit/commit/425f9293f8d4f3cd2423eabd9dd099fe0dce82b2 (via commits.webkit.org/305121@main) | WebKit (GitHub) | 2026-01-05 | 2026-10-23 | high (commit message paraphrased by fetch tool) | primary
- Scope of fix = intrinsic track sizing during grid layout; benchmark triggers full relayout by width change. Fetched text says nothing on contenteditable / per-item content change; whether a text edit in one cell re-runs track sizing for the whole grid is NOT shown | same | | | medium | inference
- WebKit build 20621.3.11.11.3 = Safari 18.6 on macOS Sequoia 15.6 (also seen with macOS 15.7.2 in network-process strings) => a mid-2025 branch, i.e. built before 2026-01-05 | https://mrmacintosh.com/macos-sequoia-15-6-update-everything-you-need-to-know/ | Mr. Macintosh (blog) | 2025-07 (approx, unverified) | 2026-10-23 | medium (secondary) | secondary
- Wikipedia lists Safari 27.0 (2026-09-14) available on macOS 15; page flagged as possibly unverified | https://en.wikipedia.org/wiki/Safari_version_history | Wikipedia | n/a | 2026-10-23 | low | secondary
- Safari 26.x security updates (26.1, 26.5, 26.5.2, 26.6.1 of 2026-08-18) ship for Sonoma and Sequoia | https://9to5mac.com/2026/08/18/safari-26-6-1-patches-multiple-webkit-security-flaws-on-macos-sonoma-and-sequoia/ | 9to5Mac | 2026-08-18 | 2026-10-23 | medium | secondary
- Blink: GridNG (Chrome/Edge M93) rewrote grid on LayoutNG; MS claims better perf via "improved caching" and fixes exponential nested grids; no cache design or subgrid scale numbers found | https://thewindowsupdate.com/2021/08/10/compat2021-improving-css-grid-compatibility-with-gridng/ | Microsoft Edge blog (mirror) | 2021-08-10 | 2026-10-23 | medium | secondary
- Other WebKit bugs: 276609 (2024, fewer grid item invalidations when stretching on column axis); 232673 (2021, contenteditable typing slows with document size only when focus outline visible - not grid) | https://bugs.webkit.org/show_bug.cgi?id=276609 ; https://bugs.webkit.org/show_bug.cgi?id=232673 | WebKit Bugzilla | 2024 / 2021 | 2026-10-23 | low (search snippets only) | secondary

## Leads
- Safari.app 27.2 vs /System WebKit.framework 20621.x on macOS 15.8.2 look inconsistent; Tauri's WKWebView uses the system framework, which may not be what Safari.app runs. Check what WebKit the Tauri app actually loads (navigator.userAgent AppleWebKit/ token, or the process's loaded WebKit.framework path / com.apple.WebKit.WebContent version) rather than Safari.app version.
- Test the committed benchmark (subgrids-spanning-many-rows.html) and a content-edit variant in the app's WKWebView to see if the 305121 speedup is present (0.2 vs ~50 runs/s on M4 Max; scale for hardware).
- Read the diff on GitHub for whether track sizing is skipped on single-item content change.
- Safari 26.2-27.x release notes on webkit.org/blog for 304869 mention (not found this run).

## Looked for, not found
- Which Safari release first contains 305121@main (release notes cite no bug number found); branch cut date of 621.x/622.x/7624 tags.
- Any text confirming 20621 -> "AppleWebKit/621" UA token.
- Chromium primary docs on subgrid scale or per-item relayout cost.
- Any WebKit bug specifically on contenteditable typing in a large (sub)grid.
