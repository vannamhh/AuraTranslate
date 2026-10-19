# Handoff phase 1 (Rust) -> phase 2 (Webview) -- tim-kiem-library-pham-vi-bo-loc

Working notes only. Plan: `../story-tim-kiem-library-pham-vi-bo-loc-mo-o-che-o-oc-plan.md`.
Not committed. `lib.rs` untouched (no new IPC command).

## Invoke args of `library_search` (camelCase on the JS side)

Existing: `query`, `limit`, `mode`. New, all optional (omit or `undefined` = default):

| JS key | Rust | Values | Default |
|---|---|---|---|
| `scope` | `Option<String>` | `both` / `source` / `target` | `both` |
| `status` | `Option<Vec<String>>` | any of `not_started` `in_progress` `paused` `done`; `[]` = no filter | no filter |
| `sourceLang` | `Option<String>` | `zh` / `en` only | no filter |
| `sort` | `Option<String>` | `updated_desc` / `name_asc` | work_id order (legacy) |
| `workId` | `Option<String>` | one work id ("Xem cả N kết quả") | all works |

`sort` omitted keeps the old `work_id, chapter_ord, segment_ord` order. Send `sort` explicitly from the webview so groups follow the chosen sort. Inside a work, hits are chapter then segment. Target-side hits come first, then source-side hits (each side is limited to `limit` separately, as before).

## Error keys (all refused before any SQL runs)

- `library.unknown_search_scope` `{scope}` -- NEW, `MessageKey::LibraryUnknownSearchScope`. I added `err.library.unknown_search_scope` to `src/i18n/vi.json` (one line, needed by `ipc_contract::every_message_key_exists_in_vi_json`). This is the only `src/` file touched.
- bad status -> `lifecycle.unknown_status` `{status}` (existing).
- bad sort -> `library.unknown_sort` `{sort}` (existing).
- `sourceLang` not `zh`/`en` -> `import.unsupported_source_lang` `{source_lang}` (existing key; its vi text talks about "không có gì được ghi lại", slightly off for search). The webview can only send `zh`/`en` from chips, so this should never show.

## Wire shape additions (snake_case)

`SearchHit` new fields:
- `segment_ord: number | null` -- null iff `segment_id` is null (chapter-level hit).
- `segment_status: string | null` -- `segment.status` AS OF THE LAST INDEX BUILD (`draft`, `confirmed`, ...; the list is the segment status set, not re-checked by Rust). Null on chapter-level hits. Confirming a segment does NOT refresh the index, so this value can be stale until rescan/import/status change.
- `work_status: string | null` -- `library_work.status`, null when the work has none.
- `source_lang: string` -- `zh` / `en`.

`SearchReport` new fields:
- `scope: string` -- echo of the scope that ran.
- `chapters_total: number` -- `SUM(chapter_count)` over every indexed work.
- `chapters_filtered_out: number` -- `chapter_count` of works failing the status/sourceLang filter (the `workId` filter is NOT counted). 0 when no status/sourceLang filter. Use for "X trong Y Chương không được tìm tới".
- `work_counts: { work_id, target_count, source_count, source_count_is_lower_bound }[]` -- real per-work counts for the branch that actually ran (`effective_mode`), only works with at least one result. `target_count` is a SQL `GROUP BY` count; `source_count` counts trigram candidates the Rust verification accepted. A side excluded by `scope` counts 0.
- `source_counts_capped: boolean` -- the source verification pass hit its candidate ceiling (25,000 candidates). Then only the LAST work in sort order carries `source_count_is_lower_bound: true` (show `N+`), and works sorted after the cut may be missing from `work_counts` entirely. Works before the cut are exact.

Semantics to keep in mind:
- `short_query` is now reported only when the scope includes the source side (scope `target` + 2-char query => `short_query: false`, the query runs).
- `total` is still `hits.length`, `truncated` still "some branch hit `limit`". The header "N kết quả" must come from `work_counts` (sum of `target_count + source_count`, with `+` if any lower bound or `source_counts_capped`), not from `total`.
- A hit works-id always has an entry in `work_counts` (hits query uses a prefix of the same ordered candidates).
- Widening (`widened`) stays inside scope + filters: exact empty under the filter -> lenient inside the same filter.
- `hit.match_kind` unchanged.

## Index

`LIBRARY_INDEX_MIGRATIONS` `to_version` 9 -> 10, new column `library_segment.segment_status` (NULL on chapter-level rows). `Indexer::open` deletes and rebuilds on first open; no migration step. Existing tests that asserted version 9 now assert 10.

## Rust API (for reference)

`Indexer::search` unchanged (wrapper, default filter). New `Indexer::search_filtered(query, limit, mode, SearchFilter)`; `SearchFilter { scope, status, source_lang, sort, work_id }`, `SearchScope`. Command layer: `search_library` unchanged (4 args) plus `search_library_scoped(.., &SearchArgs)`.

## Measurements (p95 task)

Probe was a throwaway test (deleted); Indexer::search_filtered, limit 50, 20 runs per case, in-process (no IPC), counts included.

Real Library (`~/Documents/AuraTranslate`, rebuilt into a temp index): 47 works, 12 with text, 90 segments, 47 chapters. Every query 0.1-1.6 ms. Too small to say anything about p95 at scale.

Synthetic population (20 works, 40,000 segments, 2,000 chapters; every source row contains `the quick brown fox`, every target row `không`/`và`/`nguoi`). Build: `cargo test` dev profile with `opt-level=2`, no debug info (not the shipped `opt-level="s"` + LTO). p95 ms, Exact / Lenient:
- `the` (both; matches all 40,000 source rows, hits the 25,000 candidate ceiling): 189 / 569  <- over NFR3 (500 ms) in Lenient
- `không` (both): 58 / 93
- `và` (target): 79 / 69
- `nguoi` (both): 54 / 80
- 1-2 char CJK queries are below the trigram floor, so they only measure the target side: 2-5.
Same cases in the plain dev profile (opt-level 0): `the` 621 / 1305, `không` 221 / 313, `và` 260 / 405, `nguoi` 209 / 291.

Cause of the 569: the count pass verifies up to 25,000 trigram candidates in Rust, and the lenient verifier (`fold_for_tolerant_verification`, NFD per char) is several times dearer than the exact `to_lowercase().contains`. Only a source-side term matching more than ~25,000 rows reaches it. Needs Ice (plan: over NFR3 goes to Ice before going on). Two options: lower the count ceiling (earlier `N+`), or keep it and accept a rare slow lenient case.

## Verified / not verified

Verified (cargo, debug build): `library_index_contract` 102 ok, `library_commands_contract` 32 ok, `ipc_contract` 43 ok. New cases cover: target-only string per scope, filter before `LIMIT`, widening inside filter, blocked chapter counts, real per-work target counts above `limit`, trigram false positive excluded from the source count (counter-checked: dropping the verification in `count_per_work` turns it red 2 != 1), ceiling lower bound, chapter-level hit fields, sort in SQL, unknown scope/status/sort/sourceLang refused.

Not verified: any webview/e2e behaviour; the real packaged app; Windows.

## Ceiling measurement for Ice

One build (dev test profile, opt-level=2, no debug info, not shipped `opt-level="s"`+LTO), one process, run back-to-back; 20 works / 40,000 segments / 2,000 chapters (source `the quick brown fox number w-c-s 天下大势`, target `không và nguoi hiền ở câu w-c-s`); `search_filtered`, limit 50, 20 runs after 2 warm-ups. Cells: median / p95 ms, pass 1 | pass 2.

| Case | `the` Exact (Both) | `the` Lenient (Both) | `không` Lenient Both | `không` Lenient Source |
|---|---|---|---|---|
| Ceiling 25,000 (current) | 163/199 \| 107/157 | 292/359 \| 313/497 | 51/56 \| 49/65 | 2/4 \| 3/9 |
| Ceiling 10,000 | 68/73 \| 93/134 | 154/163 \| 200/323 | 44/50 \| 54/64 | 2/2 \| 2/2 |
| Ceiling 5,000 | 53/78 \| 60/88 | 119/139 \| 122/173 | 50/64 \| 44/47 | 2/2 \| 2/4 |
| Ceiling 2,500 | 55/75 \| 52/63 | 109/127 \| 118/232 | 46/58 \| 54/59 | 2/3 \| 3/5 |
| No count pass (pre-story approx) | 32/35 \| 43/47 | 75/91 \| 87/98 | 30/53 \| 33/37 | 2/3 \| 2/3 |

Variance: current-ceiling `the` Lenient p95 differs 359 vs 497 (>20%), also 10k 163 vs 323; medians agree within ~10% for 25k (292/313) but 154 vs 200 at 10k, so the tail is machine-load noisy; the earlier 569 was the same class of tail. Even 5,000 is under 300 ms p95 lenient in both passes (139/173); `không` is unaffected by the ceiling (it never reaches it).

## Ice decision on the ceiling

Ice chose 5,000 (option a). `SOURCE_COUNT_CEILING: i64 = 5_000` is now a `pub const` in `indexer.rs`, used by `count_per_work`; the lower-bound test reads the constant. Webview: a source count flagged `source_count_is_lower_bound` renders as `N+`.

# Webview phase -> Tests phase

Not committed. `src-tauri/` untouched (handoff was right). Plan boxes ticked: config/state, component + LibraryMode, readingState/commands/main/i18n. Still open for Tests phase: `tests/frontend/*` new coverage, deferred-work item, e2e story-12-5 + move 5-9/5-10.

## Design choices that differ from a literal reading of the plan

- Filters (status, source_lang, sort) are NOT a second set of controls: the search reuses the Library top-bar state from `libraryWorks.ts` (`libraryStatusFilter`, `librarySourceLangFilter`, `librarySortKey`). `LibraryMode.vue` watches those three and calls `rerunLibrarySearch()` (only when a search has already loaded). The results component shows them as read-only chips. The genre filter is NOT sent (Rust has no genre filter); `library.filter_clear` ("Bo loc") also clears genre.
- Scope is the only new control: three native radios (`@change`, outside check:commands Kiem A), in the top bar next to the diacritic segmented control.
- Group order = order of first appearance in `hits` (Rust returns target-side hits first, then source-side), so a source-only work can sort after target works regardless of sort key. `work_counts` is BTreeMap-by-work_id, so it cannot supply order.
- Header "N Chuong · M Tac pham" is result-derived: distinct (work, chapter) among RETURNED hits, and number of groups. It is a lower bound when truncated; the counter "N ket qua · X ms" is the real count (sum of work_counts, `N+` if lower bound or `source_counts_capped`).
- Hit rows carry two buttons each (open, open in Reading). Each button does `@pointerdown`/`@focus` -> `setLibrarySearchCursor(i)` before its `dispatch`, because commands read the cursor (WKWebView does not focus a button on click, hence pointerdown). Same trick for group buttons via `selectLibrarySearchGroup(workId)`.
- Mode note has no `⌘⇧D` / `⌘↵` hint (no such keys). Key hint line shows the `library.search` chord only.

## New command ids (all `keys: undefined`)

- `library.open_search_hit_reading` -> `openCurrentLibrarySearchHitInReading` (librarySearch.ts) -> `openSearchHitInReading(chapterId, segmentId|null)` (readingState.ts: `openChapterById` -> `anchorSegmentId` -> `resetReading({preserveAnchor:true})` -> `resetReadingToc()` -> `setMode('reading')`; returns false and stays if the chapter did not open). Same `openWorkById` + error/notice guard as `openCurrentLibrarySearchHit`.
- `library.search_group_toggle` (cursor's group expand/collapse), `library.search_group_see_all` (re-run with `workId` of cursor hit). i18n labels `command.library.*` added.
- Existing reused by the empty state: `library.filter_clear`, `library.rescan`; "Tim lai tren moi Tac pham" button dispatches `library.search` (which clears the work filter).
- `CommandDeps`: `openCurrentLibrarySearchHitInReading`, `seeAllLibrarySearchGroup`, `toggleCurrentLibrarySearchGroup`; wired in `main.ts`.

## librarySearch.ts exports (new)

State (readonly refs): `librarySearchScope`, `librarySearchReportScope`, `librarySearchWorkId`, `librarySearchElapsedMs`, `librarySearchChaptersTotal`, `librarySearchChaptersFilteredOut`, `librarySearchSourceCountsCapped`, `librarySearchExpandedWorks`. Computed: `librarySearchGroups`, `librarySearchVisibleHits` (cursor walks THIS, not `librarySearchHits`; `currentLibrarySearchHit` reads it), `librarySearchResultCount` `{count, lowerBound}`, `librarySearchChapterCount`, `librarySearchFilterActive`. Pure: `splitSnippet`, `groupSearchHits(hits, counts, expanded)`, const `SEARCH_GROUP_COLLAPSED_HITS = 3`. Actions: `setLibrarySearchScope`, `rerunLibrarySearch`, `toggleLibrarySearchGroup(workId)`, `toggleCurrentLibrarySearchGroup`, `selectLibrarySearchGroup`, `setLibrarySearchCursor`, `seeAllLibrarySearchGroup`, `clearLibrarySearchWorkFilter`, `openCurrentLibrarySearchHitInReading`.
Semantics: `runLibrarySearch()` (Enter / `library.search`) clears `workId`; mode/scope/filter changes go through the internal `executeLibrarySearch` and keep it. Every successful run resets `expandedWorks`. `resetLibrarySearch()` resets all new state (scope back to `both`). `ms` = `performance.now()` delta around `searchLibrary`, rounded.
`config/library.ts`: `searchLibrary(query, limit?, mode?, options?)` sends `scope/status/sourceLang/sort/workId` (null when unset; `status: []` when no status filter). Types `SearchScope`, `SearchLibraryOptions`, `WorkSearchCount`; guards reject a report/hit missing any new field (so every existing mock of `library_search` needed the new fields).

## DOM hooks (LibrarySearchResults.vue unless noted)

- `LibraryMode.vue`: `[data-library-search-scope="both|source|target"]` radios; `[data-library-search-input]`, `[data-library-search-mode]` unchanged. Inactive state (no search loaded/busy/error) shows works grid plus a `p[data-library-search-status]` with the not-typed hint; active state replaces `.works-block` entirely (`searchActive`).
- Root `[data-library-search-results-block]`; `[data-library-search-status]` (role=status; result states show `N ket qua · X ms`); `[data-library-search-extent]`; `[data-library-search-chips]`; `[data-library-search-mode-note]` (class `mode-note--relax` when lenient/widened); `[data-library-search-coverage-gap]`; `[data-library-search-truncated]`; `[data-library-search-prev|next]` + `.grid-nav-position`.
- Groups: `ul[data-library-search-results]` > `li[data-library-search-group][data-work-id]` with `[data-library-search-group-name]`, `[data-library-search-group-count]` (text `N ket qua` / `N+ ket qua`); buttons `[data-library-search-group-expand]`, `[data-library-search-group-collapse]`, `[data-library-search-group-see-all]`, `[data-library-search-work-clear]`.
- Rows: `li[data-library-search-hit][data-field="target|source"]` (class `hitrow--current` + `aria-current` on the cursor row), `[data-library-search-mark]` (`data-match-kind`, class `mark--loose` when lenient), `[data-library-search-hit-lenient]` text tag, buttons `[data-library-search-open]`, `[data-library-search-open-reading]` (every row, not only the cursor row; old e2e that expected one `data-library-search-open` now sees one per visible row).
- Empty: `[data-library-search-empty]`, `-empty-scope`, `-empty-filtered`, `[data-library-search-clear-filters]`, `[data-library-search-rescan]`. Key hint `[data-library-search-key-hint]`.
- Margin rule colours: target = `--color-primary`, source = `--color-on-surface-variant`; mark = primary on surface-accent, `.mark--loose` = tm-text on surface-tm.

## Surprises

- `check:tokens` `FILE_FLOOR` (scripts/check-tokens.mjs) went 148 -> 159: one new `.vue` file tripped the 80% rule (186 files). Mark padding `0 4px` (grid rule, mockup has 3px).
- `check:i18n` requires an `aura-allow-text` comment before every non-`t()` mustache (helper-built strings, snippet parts), and UX-DR47 rejects "ban" in a vi string.
- Old test assertions on the status line for widened/truncated moved to `[data-library-search-mode-note]` / `[data-library-search-truncated]` (the latter only renders when real count > shown hits, so the mock must provide `work_counts`).

## Verified / not verified

Verified: `npx vue-tsc --noEmit` clean; `npm run check:tokens|i18n|commands|panel-refs|layout|doc-refs|lint` green; full `npx vitest run` 178 files / 2481 tests green after fixing fixtures in `tests/frontend/librarySearch.test.ts` (new required fields on hits and reports; 3 assertions retargeted as above).
Not verified: anything in a real webview (no e2e run, no visual check of layout, radios styling inside `.segmented`, 160px row grid); `openSearchHitInReading` has no test (anchor set to the right segment, chapter-level hit -> null anchor, chapter-not-opened -> stays in Library); in-progress chapter -> `frontier-only` view and scroll-to-anchor in Reading unverified; `cargo test` not run (Rust untouched); no counter-check of the guards.

## Fix: group order + extent

- Rust: `count_per_work` now returns `work_counts` ordered by `SearchFilter::work_order_sql()` (no sort = `work_id`; `updated_desc`; `name_asc`), via one query over `library_work`; counted ids missing from it are appended. Test `work_counts_follow_the_filter_sort_not_the_work_id_or_which_side_hit` (source-only Work first under `updated_desc`, last under `name_asc`).
- Webview: `groupSearchHits` orders groups by `work_counts` order, Works without a count entry follow in first-seen order. Test in `tests/frontend/librarySearch.test.ts`.
- Header extent reads `librarySearchChaptersTotal` and `librarySearchWorksTotal` (index population); `librarySearchChapterCount` removed. Mount test checks `1847 Chương · 12 Tác phẩm`.
- Counter-check: Rust sorted by `work_id` after the query, red at the `name_asc` assertion; webview first-seen order and `groups.length` extent, both new vitest cases red. All restored.

# Tests phase

Not committed. No product code changed (readingState.ts was edited and restored byte-for-byte for a counter-check).

## Files

- NEW `tests/frontend/librarySearchScope.test.ts` (40 cases): scope/status/sourceLang/sort/workId on the wire, ms, grouping and real counts, `N+`, DOM (grid replaced, group header, counter, see-all, mark class, margin side, hit meta, chapter-level row, open-in-reading button, key hint, empty scope, filter-blocked, widened note), wire guards for every new field, `openSearchHitInReading`.
- NEW `e2e/specs/story-12-5-library-search-scope.e2e.mjs` (6 cases).
- `e2e/specs/story-5-9-library-search.e2e.mjs`: 2nd case waited for `.works-block` after a search left results on screen (results replace the grid); now waits for the search field.
- `deferred-work.md`: one item, `Chủ: Epic 12`, key for `library.open_search_hit_reading`. `check:debt-owner` green.
- Plan: the three remaining Execution boxes ticked.

## Ran

- `npx vitest run tests/frontend/librarySearch.test.ts librarySearchScope.test.ts libraryTopBar.test.ts`: 82 pass.
- `cargo test --test library_index_contract --test library_commands_contract --test ipc_contract` (debug): 103 (3 ignored, pre-existing) / 32 / 43 pass.
- e2e on WKWebView, debug build with `--features wdio`, built from the current `dist/`: 12-5 6/6, 5-9 2/2, 5-10 2/2, 12-2 4/4, 5-6 1/1, 5-11 1/1, each file alone.
- check:lint, commands, i18n, tokens, panel-refs, doc-refs, debt-owner green. Full suite NOT run (no shared wiring touched this phase).
- Counter-check: deleting `anchorSegmentId.value = segmentId` in `openSearchHitInReading` turns "hit ở câu" red (1 failed, 39 pass). Restored.

## Surprises

- Lenient `match_kind` is only set for rows that do NOT match exactly (rowid set difference), so a lenient hit needs a differently-accented query. e2e types `khoáng<tag>tq` into the translation and searches `khoang<tag>tq`.
- 12-5 relies on `set_chapter_status` rebuilding the index to make typed target text searchable; it worked.
- Not hit-tested in e2e: `Xem cả N kết quả` (needs more hits than returned), `N+` and the 5.000 ceiling (Rust test + vitest only).

## Not verified

Packaged app, Windows, nightly CI run, keyboard path of the new buttons (driver cannot), real-use look of layout/colours (only computed-style difference of mark background is measured), real Library at scale.
