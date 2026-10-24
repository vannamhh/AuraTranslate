# D5 WebKit keystroke cost + measurement, r1-1 (web only, accessed 2026-10-24; 5 searches + 3 fetches; 3 pages read)

## Claims
- Safari 26.2 added the Event Timing API (post does not mention interactionId) | https://webkit.org/blog/17640/webkit-features-for-safari-26-2/ | WebKit blog | 2025-12-12 | 2026-10-24 | high (one primary source) | version-compat
- interactionId is a spec field (Event Timing WD 2026-02-23) grouping entries into one interaction; MDN lists Baseline 2025 "newly available" since Dec 2025; Safari-specific interactionId support not confirmed from compat table | https://developer.mozilla.org/en-US/docs/Web/API/PerformanceEventTiming/interactionId | MDN | n/d | 2026-10-24 | medium | spec
- WebKit bug 300102 "Event Timing: implement InteractionId logic" (Aug 2025) committed before 26.2 — search summary only | https://bugs.webkit.org/show_bug.cgi?id=300102 | WebKit Bugzilla | 2025-08 | 2026-10-24 | low | version-compat
- Bug 297361 (keydown entries getting duration assigned when queued) fixed 2025-10-24 — search summary only | https://bugs.webkit.org/show_bug.cgi?id=297361 | WebKit Bugzilla | 2025-10-24 | 2026-10-24 | low | bug-status
- Bug 232673: typing in contenteditable slower as document grows with default focus outline; `outline: none` removes lag; time shown as "Other"; Safari 15 / macOS 11, NEW, unassigned (2021-11-10) | https://bugs.webkit.org/show_bug.cgi?id=232673 | WebKit Bugzilla | 2021-11-03 | 2026-10-24 | medium | bug-status
- Bug 263303: spellchecking in WebKit Editor makes navigation slow in very long textarea/contenteditable; VoiceOver case fixed by disabling spellcheck under VoiceOver, workaround skips eager caching > 5,000 chars — search summary only | https://bugs.webkit.org/show_bug.cgi?id=263303 | WebKit Bugzilla | 2023 | 2026-10-24 | low | bug-status
- Resolved contenteditable slowness bugs (summaries only): 254858 (2023), 256762 (2023), 150692 (2015) — not current | https://bugs.webkit.org/show_bug.cgi?id=254858 | WebKit Bugzilla | 2023 | 2026-10-24 | low | bug-status
- `selectionchange` fires asynchronously in WebKit (Ryosuke Niwa, 2011); bug 213025 (Safari 13, 2020, NEW): typing fires `selectionchange` before MutationObserver callback, unlike Chromium/Firefox; does not settle one-per-character | https://bugs.webkit.org/show_bug.cgi?id=213025 | WebKit Bugzilla | 2020 | 2026-10-24 | low | engine-behavior
- `event.timeStamp` in WebKit comes from the platform (OS) event since bug 117179 (2013), not DOM Event creation ⇒ marks OS input receipt, not handler start; bug 211101 (2020) ~31-year offset under WebDriver only; not verified against current source | https://lists.webkit.org/pipermail/webkit-dev/2013-June/025023.html | webkit-dev | 2013-06 | 2026-10-24 | medium (primary, old) | engine-behavior
- Event Timing semantics: startTime = browser received event; processingStart = listeners began; processingEnd = listeners finished; duration = startTime → next paint, rounded (8 ms in Chrome) | https://w3c.github.io/event-timing/ | W3C | 2026-02-23 WD | 2026-10-24 | medium (spec text not read) | spec

## Leads
- Bug 300102, 26.2 notes, wpt.fyi event-timing results to confirm interactionId and keydown duration in target macOS.
- Bug 232673 A/B: `outline: none` on editable cells.
- WebKit source / Bugzilla HTML Editing for normalize / setBaseAndExtent synchronous layout.
- Unsourced suggestions: Web Inspector Timelines on WKWebView, `performance.now()` around handlers, rAF after event.

## Looked for, not found
- Documentation/bug showing DOM mutation, `normalize()`, `setBaseAndExtent`/`addRange` in keydown/beforeinput/input forces synchronous style/layout.
- Cost of selection restoration or many sibling nodes/ranges in WebKit contenteditable.
- Confirmation `selectionchange` fires once per typed character in current WebKit.
- Bug about typing latency growing with node count beyond 232673 and 47047 (WORKSFORME, old).
- WKWebView-specific Event Timing account (Safari ⇒ WKWebView is inference).
- Production latency numbers for contenteditable typing in WebKit.
- Budget: 8 of 15 tool calls; 3 pages read.
