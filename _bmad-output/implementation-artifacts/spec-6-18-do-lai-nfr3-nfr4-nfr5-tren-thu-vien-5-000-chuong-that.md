---
title: 'Story 6.18 — Re-measure NFR3, NFR4, NFR5 on a 5,000-Chapter library built through the product import path'
type: 'chore' # feature | bugfix | refactor | chore
created: '2026-09-14'
status: 'in-progress' # draft | ready-for-dev | in-progress | in-review | done
route: 'dispatch' # oneshot | dispatch
baseline_commit: '7863afbd7ea9a23c3db77b8cf0f1959172eb880e'
review_loop_iteration: 0
context:
  - '{project-root}/AGENTS.md'
  - '{project-root}/src-tauri/AGENTS.md'
---

<frozen-after-approval reason="human-owned intent — do not modify unless human renegotiates">

## Intent

**Problem:** A6, A7, A8 and Q4 are still provisional. Story 5.14's numbers came from a synthetic
fixture written with raw SQL, and its NFR4/NFR5 numbers from an `.app` with 0 dictionary layers
(Epic 5 retro F5; `AGENTS.md`: such a number "is not a number about the product"). Eleven debt
entries also wait on a real large library under the owner "Story 6.18".

**Approach:** Build a 5,000-Chapter library in a scratch library root by calling the product's own
import and lifecycle code, then re-run the 5.14 measurement shapes against it on a release app that
loads the dictionary layers. Measure the eleven debt entries in the same effort. Give A6/A7/A8
exactly one verdict each, write the verdicts into the PRD and the canonical SPEC, mark 5.14 as
preliminary, and close Q4.

### Decisions (Ice, 2026-09-14)

- **macOS only; Windows becomes owned debt.** Q4 closes on the macOS verdicts. Every place a verdict
  is written says "macOS", and one `deferred-work.md` entry carries the Windows re-measurement, owner
  Ice, pointing at the end-of-project Windows acceptance (2026-08-12 decision, retro item B7). This
  knowingly narrows the epic AC "trên cả macOS lẫn Windows".
- **Shape: 50 Works × 100 Chapters × 10 segments** (5,000 Chapters, 50,000 segments), one product
  import per Work.
- **State: bilingual import (Story 6.16) with a chapter pattern**, so every segment has source and
  target text (`draft`). Two library states, both reached through product lifecycle code: *frontier*
  (the opened Work's first Chapter not `done`) and *full* (all 5,000 Chapters `done`).
- **All eleven "Chủ: Story 6.18" debt entries are in scope**, including the four on other surfaces:
  wall-clock `webimport_contract` cases · one `reqwest` client per link · preview re-classification
  on `⌥←/⌥→` · the filter rendering the full list. Each ends measured, with a verdict; a fix, if
  warranted, goes to a named new owner.
- **The bench command becomes one neutral feature `nfr-bench`**, replacing `story-5-14-bench`, with
  the limits Ice set on 2026-09-02 unchanged: only that feature compiles the command, `default`
  lacks it, no dependency/network/CSP/ATS, never a product path, no file I/O outside the scratch
  HOME. The 5-14 harness is updated to the new names.

## Boundaries & Constraints

**Always:**
- The library is created only by product functions: the pure `preview_bilingual_import` then
  `confirm_bilingual_import` (reaching `create_work(documents_root, …)`) with `documents_root` =
  scratch root; Chapter status only through `lifecycle::set_chapter_status`. Before any timing, the
  harness reads the library back and checks Works 50 / Chapters 5,000 / segments 50,000 / bytes /
  statuses.
- App for NFR4/NFR5 and every UI probe built `--release --features nfr-bench`; Rust benches
  `--profile bench-release`; no debug, no `wdio`. The run is invalid unless the probe reads a
  loaded-layer count > 0 before the first sample.
- Keep 5.14's fail-closed rules: complete raw matrix or no report; one p95 per NFR3 query kind;
  NFR4 ends when the Library grid holds this library; NFR5 = app PID + every spawned WebKit PID,
  reported in byte, MB and MiB.
- Every number carries commit, date, machine/OS, toolchain, profile, load average and sample size.
  Before timing: tree unchanged since build, no other cargo/npm/app process of this session.
- Each of A6/A7/A8 gets exactly one verdict: keep · change to the measured number · far over, so a
  PRD-level decision for Ice. Writing a changed number or a PRD-level verdict needs Ice's sign-off
  on the presented numbers. If NFR5 lands between 300,000,000 and 314,572,800 bytes, stop for Ice to
  rule MB vs MiB.
- Verdicts are written to `prd.md` (NFR3–5 rows, A6–A8, Q4) and to the canonical SPEC; 5.14's
  records stay, marked preliminary and superseded by 6.18.
- `check:debt-owner` 0 orphans.

**Never:**
- No raw SQL write into `project.db` / `library-index.db` on the library-building path.
- No product optimization, no threshold edited to change a verdict, no new crate, no public network,
  no `tauri.conf.json` / `capabilities/**` / CSP / `[profile.release]` change.
- No commit of `.db`, scratch HOME/library, `.app`, `dist/`, `target/`, dictionary files.
- No touch of real user data (real HOME, Documents); scratch paths are marker-guarded.
- No Windows number, and no wording that implies one.
- No edit to `epics.md` to match the result.

## I/O & Edge-Case Matrix

| Scenario | Input / State | Expected Output / Behavior | Error Handling |
|---|---|---|---|
| Build library | 50 bilingual imports | Read-back equals declaration; build time + peak RSS recorded | Any mismatch ⇒ run invalid, no timing |
| NFR3 | Target, source, auto-widen, lenient | 10 warmup + ≥200 samples/case, ≥3 sessions; p50/p95/p99, `truncated` | Wrong hit count / mode / unexpected truncation ⇒ invalid |
| NFR4 | ≥10 sessions, cold and warm | ms to a grid holding the library; per sample, median, max, spread | Layers 0 or grid empty ⇒ `unknown` |
| NFR5 | frontier and full × Library, Reading, back-Library | byte/MB/MiB, app + WebKit set, 10 samples/phase | Missing PID or phase ⇒ sample kept as error, verdict `unknown` |
| Debt probes | One probe per entry, sizes recorded | A number and a verdict per entry | Probe not run ⇒ entry stays open |
| Verdict | Numbers vs A6–A8 | One verdict each; Q4 closed only when all three are recorded | 300.0–314.6 MB ⇒ stop for Ice |
| Red controls | Layers stripped / population off by one | Harness refuses before sampling | Named refusal message |

</frozen-after-approval>

## Code Map

Line numbers drift — re-locate by symbol.

**Harness and benches**
- `_bmad-output/implementation-artifacts/5-14-ban-do/` -- `build.sh` (`npx tauri build --bundles app
  --features story-5-14-bench`), `run.sh` (`mktemp` HOME, `trap cleanup`, `set_phase`,
  `launch_to_usable`, `measure_memory_session`, `sample_phase` via `/usr/bin/footprint` + `ps` WebKit
  set), `probe.js` (usable marker), `summarize.mjs` `requireMatrix`. Reuse the shapes; rename to
  `nfr-bench` in place.
- `src-tauri/tests/library_index_contract.rs` `bench_p95_of_a_library_search_over_five_thousand_chapters`,
  `NFR3_CASE`; `src-tauri/tests/segment_contract.rs` `bench_reading_run_over_five_thousand_chapters`,
  `AURA_5_14_EXPORT_LIBRARY_ROOT` -- raw-SQL fixtures (`write_atproj_with_real_project_db`); 5.14
  history, not a 6.18 library source.
- `src-tauri/src/lib.rs` `mod story_5_14_bench` (`SCRATCH_PREFIX`, `PHASE_FILE`, `MARKER_PREFIX`,
  HOME guard) + `src-tauri/Cargo.toml` feature `story-5-14-bench`, `[profile.bench-release]` -- the
  rename (git grep: `lib.rs` 14, `Cargo.toml` 2, `segment_contract.rs` 2, four 5-14-ban-do files;
  `environment.txt`, `spec-5-14…md`, `deferred-work.md` mentions are history, left as written). `open_dict_layers` reads `resource_dir()/dict` (`DICT_RESOURCE_DIR`); `setup()` rebuilds
  the index at startup.
- `src-tauri/src/commands/dict.rs` `list_sources(layers)` -- the loaded-layer signal for the guard.
- **Second session (task 5), added:** `_bmad-output/implementation-artifacts/6-18-ban-do/`
  (`build.sh`/`run.sh`/`probe.js`/`summarize.mjs`/`README.md`/`.gitignore`) — adapted from
  `5-14-ban-do`, not a fork of it: `run.sh` toggles full/frontier via
  `src-tauri/tests/story_6_18_bench_transition.rs` (new file, `#[ignore]`, product-lifecycle-only
  toggle, no raw SQL) instead of a `sqlite3 UPDATE`. `src-tauri/src/lib.rs::nfr_bench` gained three
  env vars (`AURA_NFR_BENCH_WORK_NAME`/`_WORKS`/`_READING_FULL_SEGMENTS`, all defaulting to Story
  5.14's own values) replacing the two hardcoded literals (`"5.14 Fixture"`, `50_000`/`0`) the first
  session's handoff flagged as the real gap; the `"usable"` marker now also carries a server-side
  `layers` field (`DictLayers::layers().len()`) that `run.sh` reads to refuse before the first
  sample.
- `src-tauri/tauri.conf.json` `bundle.resources` -- fonts/license only, do not edit;
  `src-tauri/resources/dict/*.db` exist locally, git-ignored. Epic 5 retro AI-1 stays Ice's.

**Product code the library goes through**
- `src-tauri/src/commands/project.rs` -- `preview_bilingual_import` / `confirm_bilingual_import`
  (take `chapter_pattern`), `create_work`. Test-callable precedent: `tests/project_contract.rs`,
  `tests/bilingual_import_contract.rs`.
- `src-tauri/src/commands/lifecycle.rs` -- pure `set_chapter_status(open, chapter_id, status)`: one
  SQL `UPDATE`, no reindex, needs an `OpenWork`; `set_chapter_status_indexed` and the IPC wire add a
  full `Indexer::rebuild` per call.
- `src-tauri/src/core/library/indexer.rs` `Indexer::rebuild(root, global)`; `commands/segment.rs`
  `read_reading_run` reads the `Done` Chapter prefix of the open Work only.

**Debt probes** (entries in `deferred-work.md`, find by the quoted symbol)
- Search: `search_source_text` / `search_candidate_ceiling` (trigram ceiling); `rebuild` opening every
  `project.db`; NFR3 preliminary; NFR4 spread.
- Import chain: `core/segment/normalize.rs` `normalize` (two `.replace()` passes);
  `core/segment/chapterpattern.rs` `ChapterPattern::match_starts` (regex recompiled); peak RSS of the
  whole chain.
- URL: `core/webimport/fetcher.rs` `reqwest::blocking::Client::builder()` per link;
  `tests/webimport_contract.rs` `perf_probe_twenty_links_end_to_end_fetch_plus_extract_plus_pipeline`,
  `spawn_once` loopback server, wall-clock cases `a_response_that_is_not_html_…`,
  `an_oversized_body_…`, `a_response_advertising_far_more…`, `a_blocked_cross_host_redirect…`.
- Preview: `commands/project.rs` `wire::preview_chapter_detail` → `cleanup_and_chapters_preview_for`
  → `build_chapter_split_preview_wire` → `classify`; `src/ImportPreviewOverlay.vue` `chaptersShowAll`;
  eager precedent `tests/cleanup_contract.rs` `perf_probe_chapter_split_preview_on_two_thousand_chapters`.
- Ice-owned inputs: NFR5 routes entry and `char_idx` third-branch entry get the numbers appended only.

**Records**
- `_bmad-output/planning-artifacts/prds/prd-AuraTranslate-2026-08-02/prd.md` (NFR3–5, A6–A8, Q4);
  `_bmad-output/specs/spec-AuraTranslate/SPEC.md` (A6–A8, Q4), `requirements.md` (NFR3–5).

## Tasks & Acceptance

**Execution:**
- [x] `src-tauri/Cargo.toml`, `src-tauri/src/lib.rs`, `src-tauri/tests/segment_contract.rs`,
  `5-14-ban-do/{build.sh,run.sh,probe.js,README.md}` -- rename to `nfr-bench` (feature, module,
  scratch prefix, phase file, marker) -- Decision 5. Verified: `npm run build && cargo test --locked`
  green; `git grep` for the old `story-5-14-bench`/`story_5_14_bench` tokens across
  `src-tauri/**`/`scripts/**`/`.github/**` returns 0 (the fixture name `"5.14 Fixture"` and
  `AURA_5_14_EXPORT_LIBRARY_ROOT` deliberately kept, history per Code Map).
- [x] `src-tauri/tests/config_invariants.rs` -- one source-scan case: `nfr-bench` absent from
  `default`, and the bench command registered only under `cfg(feature = "nfr-bench")` -- no gate
  guards this today. Verified both halves RED under injected defects (`default = ["nfr-bench"]`;
  `cfg` stripped from the registration line, both alone and combined) and GREEN restored.
- [x] `src-tauri/tests/story_6_18_library.rs` -- `#[ignore]` builder: deterministic bilingual files
  (Chinese source, Vietnamese target, carrying the NFR3 query vocabulary), 50 product imports into a
  marker-guarded root, both states via `set_chapter_status` then one `Indexer::rebuild`, read-back,
  export root -- the only library source. Verified at small scale (3 Works) AND run twice at the real
  50×100×10 scale under `--profile bench-release --ignored` — both green, 50/5,000/50,000 exact, both
  states — see §Implementation Notes for the printed population lines.
- [x] NFR3 bench reading the exported root, one case per query kind — added
  `library_index_contract.rs::bench_p95_of_a_library_search_over_the_story_6_18_library`, run against
  the real 50×100×10 export — all five cases green, `worst_case_p95_ms≈80.9` — see §Implementation
  Notes for the full numbers and why they are evidence, not a verdict (one session, and this
  session's own load/idle state at capture time was not recorded, not "≥3 sessions"/"an idle
  machine" — machine identity itself is not in question, corrected 2026-09-14 P3 audit: it is
  MacBookPro16,1, the same machine Story 5.14 measured on, not a separate "container").
  🔵 **the trigram ceiling probe is DONE (third session)** — see task 6 below and §Implementation
  Notes, third session.
- [x] `_bmad-output/implementation-artifacts/6-18-ban-do/` -- `build.sh` (dictionary layers through a
  build-time `--config` merge), `run.sh`, `probe.js`, `summarize.mjs`, README; layer and population
  guards -- adapted from 5-14-ban-do, generalizing the shared `nfr-bench` command instead of forking
  it. See §Implementation Notes, second session, for what was verified and what was NOT (the shell
  orchestration itself — spawning the real `.app`, sampling memory across ≥10 sessions — has never
  run end to end; that is task 7).
  🔵 **CORRECTED 2026-09-14 (audit P3, fourth session)** — three bugs found by an external audit
  against the frozen spec, all fixed and re-verified with real runs, see §Implementation Notes,
  fourth session, for full detail: (1) the dirty-tree guard rejected four of this story's own new
  test files, which would have killed task 7 at line one — fixed in both `6-18-ban-do/run.sh` and
  `5-14-ban-do/run.sh`; (2) matrix row "Build library" required build time + peak RSS, neither was
  recorded — `story_6_18_library.rs` now samples and prints both
  (`STORY_6_18_BUILD_STATS`), `run.sh` copies them into `fixture.txt`, verified with a real
  50×100×10 run (`build_wall_ms=204570`, `peak_rss_mb=45.6`); (3) matrix row "Red controls" had
  never been exercised — both now run for real: a population-off-by-one copy makes the harness's
  own read-back check die with a named message, and a release app built without the dict `--config`
  merge reports `layers:0` in its real `usable` marker and trips run.sh's own refusal line.
- [x] Debt probes: import-chain peak RSS, `normalize`, `match_starts`, `rebuild` per open; loopback
  link lists of 100 and 1,000 for the client-per-link cost; `webimport_contract` under a loaded
  machine vs idle, ≥5 runs each; preview re-classification per cursor move and `chaptersShowAll` row
  count + render time on a 1,000-Chapter preview. All measured — see §Implementation Notes, third
  session, for every number and verdict. 🟡 **One sub-piece stays open**: `chaptersShowAll`'s render
  time was measured in `happy-dom` (vitest), not the real release `.app`/WKWebView the bullet asks
  for — `vitest.config.ts` itself documents `happy-dom` is not WKWebView; the row-count half (1,000
  `<li>`, no virtualization) IS the real DOM shape, confirmed. The WKWebView number is task 7's to
  get once the release app exists.
  🔵 **CORRECTED 2026-09-14 (audit P3, fourth session)** — five of these probes had been measured
  on the wrong profile (`dev`/debug for the plain `#[test]`/`#[ignore]` cases, `--release` for the
  client-per-link cases; §Always requires `--profile bench-release`). Re-measured all five on
  `bench-release`, see §Implementation Notes, fourth session, for raw output/exact commands. Four
  verdicts held (same order of magnitude, sometimes a moved percentage); **one verdict FLIPPED**:
  `chapter_detail_for_index` cursor-move cost dropped from ~35.65 ms/move (debug, called "real,
  user-perceptible") to 6.15 ms/move (release) — the "user-perceptible" conclusion does not survive
  the correct profile.
- [ ] Runs on an idle machine; `REPORT.md`, `environment.txt`, raw TSV committed without binaries.
  **NOT STARTED** — the harness (task 5) now exists and is syntax-/logic-checked, but has never been
  executed for real (no `.app` has ever been launched by `6-18-ban-do/run.sh`).
- [ ] `prd.md`, `SPEC.md`, `requirements.md` -- macOS verdicts after Ice sign-off where required; Q4
  closed; 5.14 text marked preliminary. **NOT STARTED** — blocked on tasks 6/7's numbers and on Ice's
  sign-off per §Always ("Writing a changed number or a PRD-level verdict needs Ice's sign-off").
- [🟡] `deferred-work.md`, `sprint-status.yaml` -- a closing line on all eleven entries; the Windows
  entry; numbers appended to the two Ice-owned entries. **9 of 11 entries closed with a measured
  verdict** (third session) — the two left untouched are "NFR3 sơ bộ" and "NFR4 vượt trần" (the two
  entries whose OWN closing condition is the real macOS run + Ice sign-off of tasks 7/8, not a
  probe task 6 can run standalone). The Windows entry is added (new "Deferred from: spec-6-18…"
  section, Chủ Ice, pointing at B7). Of the two Ice-owned entries, the `char_idx` third-branch entry
  got a baseline number appended (today's `rebuild` cost with no third table, for Ice to compare
  against); the "NFR5 vượt trần" entry did NOT get a new number — this session produced no new NFR5
  evidence (that needs task 7's real memory-sampling run). `sprint-status.yaml` needed no edit (the
  story's status line was already `in-progress`, still correct).

**Acceptance Criteria:**
- Given the library-building code, when grepped, then no `INSERT`/`UPDATE` SQL targets `project.db`
  or `library-index.db`, and the read-back reports 50 / 5,000 / 50,000.
- Given the dictionary layers stripped from the bench build, when the harness runs, then it refuses
  before the first sample; given one Chapter missing, then it refuses too.
- Given `nfr-bench` added to `default`, or the command's `cfg` removed, when `cargo test --locked
  --test config_invariants` runs, then the new case is RED; restored, GREEN.
- Given the finished run, then `prd.md` and `SPEC.md` show one macOS verdict for each of A6/A7/A8 with
  commit, date and machine, no "ngưỡng tạm" label on NFR3–5, and Q4 closed with a pointer to
  `6-18-ban-do/REPORT.md` and to the Windows debt entry.
- Given the pre-push gates, `npm run test`, `npm run build`, `cargo test --locked`, then all green
  and `git diff -- src-tauri/tauri.conf.json src-tauri/capabilities src-tauri/Cargo.lock` is empty.

## Implementation Notes

**Scope actually closed this session: tasks 1, 2, 3, and half of 4.** Tasks 5–8 are untouched — see
the handoff at the end of this section. This split follows `AGENTS.md`'s own rule ("one agent must
not implement a whole story"; split along the Code Map's shape) rather than rushing a 50-work
release-build-plus-multi-hour-benchmark story through a single pass.

**Task 1 (rename).** Mechanical, guided by `git grep` for `story-5-14-bench` / `story_5_14_bench` /
`AURA_5_14_PHASE_BUDGET_SECS` / `.auratranslate-5-14-phase` / `auratranslate-5-14-` / `__5_14_` across
`lib.rs`, `Cargo.toml`, `segment_contract.rs`, and the four `5-14-ban-do` files. Left as history (not
renamed): the fixture name `"5.14 Fixture"` itself, and `AURA_5_14_EXPORT_LIBRARY_ROOT` (the
`bench_reading_run_over_five_thousand_chapters` env var) — neither is a `nfr-bench` identifier, both
describe Story 5.14's own still-standing bench, which the rename does not touch beyond the shared
scratch-prefix string it validates against.

**Task 2 (config_invariants case).** Two halves in one test:
`nfr_bench_is_absent_from_default_and_its_command_is_cfg_gated`. Vế 1 scans `Cargo.toml` for a
`default = [...]` array containing `nfr-bench` (none exists today, matching `[features]` having no
`default` key at all). Vế 2 scans `lib.rs` for the nearest non-blank/non-comment line above the
`nfr_bench::nfr_bench_mark_and_wait_phase,` registration and requires it to read exactly
`#[cfg(feature = "nfr-bench")]`. Measured both defects independently and combined (see commit diff of
this session for the exact mutations tried) — vế 1 alone: RED via the panic message; vế 2 alone
(`--features nfr-bench`, `cfg` stripped only from the registration line): RED via the panic message;
vế 2 alone under the DEFAULT feature set (no `nfr-bench`): RED via a compile error instead
(`E0433: cannot find module nfr_bench`), because `mod nfr_bench` is *also* `cfg`-gated — still a
valid RED for the AC ("`cargo test --locked --test config_invariants` runs... RED"), just at the
compiler rather than the assertion, and the doc-comment says so explicitly so a future reader isn't
surprised. All four scenarios restored to GREEN.

**Task 3 (`story_6_18_library.rs`) — three things learned empirically, not assumed:**
1. A throwaway probe test (written, run, then deleted — its exact defect is described in the
   `push_row` doc-comment) showed that an UNQUOTED CSV cell containing a literal comma (my first
   draft's source/target text, which both carry `", "`) gets cut into extra columns by the bilingual
   parser, silently shifting the "target" column read out to whatever landed in column 1 — not a
   crash, a wrong-but-plausible-looking value. Fixed by RFC4180-quoting every cell
   (`push_row`).
2. The row that matches `chapter_pattern` is **not** excluded from segments — it becomes the
   Chapter's title *and* its first bilingual segment. The builder folds the `HOI {c}` pattern marker
   into segment `s == 0`'s own source cell (rather than a separate title-only row) so that "10 rows
   per Chapter" gives exactly "10 segments per Chapter" as the spec's Decision 3 shape requires,
   with every one of the 10 carrying the real NFR3 vocabulary (needed so `source_latin_unique`-style
   probes stay meaningful for whichever `(gc, s)` pair a downstream bench picks).
3. "Both states via `set_chapter_status`" is read literally: the builder transitions all 5,000
   Chapters to `done` (full), rebuilds, reads back; then transitions exactly one Work's 100 Chapters
   back to `not_started` (frontier — "the opened Work's first Chapter not done"), rebuilds, reads
   back a second population (still 50/5,000/50,000; status counts split 4,900/100); then transitions
   that same Work back to `done` and rebuilds a third time so the artifact left on disk is the clean
   "full" state — the more foundational default (NFR3 needs `indexed_segments == 50,000` regardless
   of status; NFR4/NFR5 toggle status at *measurement* time, Story 6.18 task 5's concern, not the
   builder's). Reopening the Work for the frontier/restore transitions goes through the product's own
   pure `commands::project::open_work(work_id, indexed)` — no hand-assembled `OpenWork` struct.

Population math: `WORKS(50) × CHAPTERS_PER_WORK(100) = TOTAL_CHAPTERS(5,000)`;
`TOTAL_CHAPTERS × SEGMENTS_PER_CHAPTER(10) = TOTAL_SEGMENTS(50,000)` — read back independently of
`Indexer` (opening each `project.db` directly) precisely so a bug in `Indexer::rebuild` itself
couldn't mask a wrong population with its own wrong count.

*Small-scale (3 Works × 4 Chapters × 3 segments) dry run, done this session:* population, frontier
transition (4 not_started of 12), and restore-to-full all matched declared numbers exactly; full
output logged in this session's transcript.

*Full-scale (50 × 100 × 10) run, done this session — ran TWICE, both green:*

```
cargo test --profile bench-release --locked --manifest-path src-tauri/Cargo.toml \
  --test story_6_18_library -- --ignored --nocapture
```

Build: 7m48s cold (`bench-release` is a from-scratch profile, first use). Test body: 213.80s /
214.32s. Both runs:

```
STORY_6_18_POPULATION  state=full      works=50  chapters=5000  segments=50000  done_chapters=5000                          bytes=21726250  verdict=matches_declaration
STORY_6_18_POPULATION  state=frontier  works=50  chapters=5000  segments=50000  done_chapters=4900  not_started_chapters=100 verdict=matches_declaration
STORY_6_18_POPULATION  state=final_export  works=50  chapters=5000  segments=50000  documents_root=<root>  index_path=<root>/../library-index.db
test builds_the_6_18_library_through_product_import_and_lifecycle_code ... ok
```

50/5,000/50,000 exactly, both states, both runs — no raw SQL write anywhere on the path (`grep -n
"tx.execute\|conn.execute" src-tauri/tests/story_6_18_library.rs` returns 0 matches; the file's only
`conn.prepare`/`query_row` calls are `SELECT`s in the read-back helpers). AC1's population half is
met on a real run, not an assumption.

**Task 4 (NFR3 bench on the real library) — the five-case half is done and RUN, the trigram-ceiling
half is not started.** Added
`library_index_contract.rs::bench_p95_of_a_library_search_over_the_story_6_18_library`: same five
`BenchCase`s as the Story 5.14 bench above it (reusing its exact vocabulary shape — the builder was
deliberately written to match), reading `AURA_6_18_LIBRARY_ROOT` (the `documents_root` the builder
prints) instead of building a fixture. `source_latin_unique` targets `(gc, s) = (2500, 5)`, chosen
mid-population rather than at an edge. **Run against the real 50×100×10 export from the run above**
(second run, with `AURA_6_18_EXPORT_LIBRARY_ROOT` set so the export survived past the test process):

```
NFR3_CASE  target_exact_truncated              query=má của tôi              warmups=10 samples=200 p50_ms=32.127039 p95_ms=34.853705 p99_ms=37.255260 worst_ms=38.480029
NFR3_CASE  source_han_truncated                query=分久必合                 warmups=10 samples=200 p50_ms=78.388675 p95_ms=80.868243 p99_ms=83.447296 worst_ms=86.520882
NFR3_CASE  source_latin_unique                 query=brown fox number 2500-5 warmups=10 samples=200 p50_ms=5.150953  p95_ms=5.875094  p99_ms=6.057078  worst_ms=6.173046
NFR3_CASE  target_auto_widen_truncated         query=ma cua toi              warmups=10 samples=200 p50_ms=32.748000 p95_ms=34.403134 p99_ms=35.145847 worst_ms=35.841484
NFR3_CASE  target_explicit_lenient_truncated   query=ma cua toi              warmups=10 samples=200 p50_ms=33.162477 p95_ms=34.484575 p99_ms=37.061181 worst_ms=37.963032
NFR3_SUMMARY  works=50 chapters=5000 segments=50000 worst_case_p95_ms=80.868243 threshold_ms=500 profile=bench-release verdict=story_6_18_final supersedes=story_5_14_preliminary
test bench_p95_of_a_library_search_over_the_story_6_18_library ... ok
```

Machine/toolchain for both runs above: MacBookPro16,1, macOS 15.7.9, Intel Core i9-9980HK, 16
logical CPUs — the same physical machine Story 5.14 measured on (§Always requires "commit, date,
machine/OS, toolchain, profile, load average and sample size" on every number that goes into a
verdict; corrected 2026-09-14 P3 audit — an earlier draft of this note wrongly called this "a
sandboxed container," distinct from "Ice's Mac"; it is not). **These numbers are real and
reproducible, but they are evidence for the next session, not a verdict** — only ONE session (this
bench run, not NFR4/NFR5, not the debt probes, not "≥3 sessions" per §Always), and this run's own
load average/idle state at capture time was not recorded (§Always: "Runs on an idle machine"; this
run shared the box with its own `bench-release` compile the first time). Treat `worst_case_p95_ms ≈
80.9 ms` against the informal `threshold_ms=500` printed in `NFR3_SUMMARY` as a strong preliminary
signal (comfortably under, all five cases), not as the ≥3-session evidence A6/A7/A8 need before a
verdict is written.

**The trigram ceiling probe (this task's other half, a debt-probe-shaped addition for
`search_candidate_ceiling`) is not started at all** — it belongs with task 6's debt probes as much
as with task 4's bench, and neither is done.

**Handoff — tasks 5, 6, 7, 8 (not started), as this section stood after the first session.**

---

## Second session (2026-09-14) — task 5 closed, tasks 6/7/8/9 still open

**Scope closed this pass: task 5 (the `6-18-ban-do/` harness) only** — same discipline as the
first session, one phase at a time. The four remaining bullets are 6 (debt probes), 7 (the real
multi-hour run + `REPORT.md`), 8 (PRD/SPEC/requirements verdicts, Ice-gated), 9 (`deferred-work.md`
closing lines) — the first session's handoff paragraph above numbered these 5/6/7/8, one short of
the actual five remaining checklist bullets (it silently folded "runs on an idle machine" into
"task 5"); this section uses the checklist's own numbering (5=harness, 6=debt probes, 7=the real
run, 8=verdicts, 9=deferred-work) so a future handoff doesn't inherit the off-by-one.

**What "generalized the command" meant, concretely.** `lib.rs::nfr_bench` no longer hardcodes
`"5.14 Fixture"` / `50_000`/`0`. Three env vars, all optional and all defaulting to Story 5.14's own
values (so `5-14-ban-do/run.sh` runs unmodified, unaware anything changed):
`AURA_NFR_BENCH_WORK_NAME` (target Work the command opens through the product's own `open_work`),
`AURA_NFR_BENCH_WORKS` (how many Work cells the `back-library` DOM poll requires before accepting
the marker), `AURA_NFR_BENCH_READING_FULL_SEGMENTS` (expected segment count for the `content`
Reading status; the `frontier-only`/`0` shape is structural, not fixture-specific, so it stays a
literal). `6-18-ban-do/run.sh` sets all three (`NFR Story 6.18 Work 00`, `50`, `1000`) when it
spawns the app. One more thing the shared command now does that Story 5.14's never needed: the
`"usable"` marker gets a `layers` field appended **server-side** (parses the JS-supplied JSON,
inserts the count from `DictLayers::layers().len()`, re-serializes) — this is the §Always signal
("invalid unless the probe reads a loaded-layer count > 0 before the first sample"). It is
deliberately NOT a hard `Err` refusal inside the Rust command: §I/O Matrix's Red Controls row says
*"Harness refuses before sampling"*, and 5.14's own harness still runs on 0 layers (history) — a
command-level refusal would silently break it. `6-18-ban-do/run.sh::launch_to_usable` is the thing
that reads `marker.layers` and `die`s if it is not `> 0`.

**Why a new test file instead of reusing `5-14-ban-do`'s raw-SQL toggle.** The old
`run.sh::set_fixture_status` did `sqlite3 "$PROJECT_DB" "UPDATE chapter SET status=..."` directly —
correct for Story 5.14 (not governed by this spec) but a straight violation of this spec's own
§Always ("Chapter status only through `lifecycle::set_chapter_status`") if carried over verbatim,
since that clause is not scoped to the *build* step alone. `src-tauri/tests/story_6_18_bench_transition.rs`
is a new `#[ignore]` test: opens the alphabetically-first indexed Work (same `min_by(name)` pick
`story_6_18_library.rs` already uses for its own frontier target, so no `work_id` needs to travel
between processes), reads its Chapter ids, calls `lifecycle::set_chapter_status` once per Chapter,
then `Indexer::rebuild`, then re-opens that Work's `project.db` read-only to verify the count
changed. `6-18-ban-do/run.sh::set_target_work_status` shells out to it (same "capture the binary
path on session 1, re-invoke directly after" pattern the NFR3 loop already uses), between the
full/frontier measurement blocks — no `sqlite3 UPDATE` anywhere in the new harness.

**Real verification performed this session (not just "it compiles"):**
- `cargo check --locked --features nfr-bench --tests` and `cargo check --locked --profile
  bench-release --test story_6_18_bench_transition` — both green, confirming the generalized
  `lib.rs::nfr_bench` and the new test file compile under the same profile the harness uses.
- **Ran the real 50×100×10 builder AGAIN this session** (`AURA_6_18_EXPORT_LIBRARY_ROOT` pointed at
  a fresh `/tmp/auratranslate-nfr-bench-*` scratch dir, kept alive past the test process), then ran
  `story_6_18_bench_transition` against that REAL export TWICE — once `not_started` (100 Chapters,
  5.17 s), once `done` (100 Chapters, 5.22 s) — both printed `verdict=matches_target`, and an
  INDEPENDENT read-only sweep of all 50 `project.db` files afterward confirmed the library ended
  exactly where the builder had left it: `chapters=5000 done=5000 not_started=0 works=50`. This is
  the actual product-lifecycle toggle the real harness will call between full/frontier blocks,
  proven end to end — not assumed from the code reading correct. Scratch dir removed after.
- `node summarize.mjs` was run against a **synthetic** full-matrix TSV set (15 NFR3 rows, 30 startup
  rows, 600 memory rows shaped exactly like a real 10-session run) — green, and `REPORT.md` rendered
  sensibly. Then a counter-check per `AGENTS.md`'s own "Known pitfalls" rule (remove the seam, prove
  the OLD gate goes red): deleted ONE of the 600 memory rows and re-ran — `requireMatrix` threw
  `"NFR5 cần 600 mẫu ok, có 599"` as designed. Synthetic files deleted afterward; nothing under
  `6-18-ban-do/` was committed by this check.
- `zsh -n` on `build.sh`/`run.sh`, `node --check` on `probe.js`/`summarize.mjs` — all clean.
- Full-suite verification per spec's own §Verification: `npm run build && (cd src-tauri && cargo
  test --locked)` — green, `0` `FAILED`, `56` `test result: ok` blocks (up from the first session's
  55, the new test binary's own summary line). `node scripts/check-debt-owner.mjs --report` — `mở
  KHÔNG có Chủ: 0`, unchanged. `git diff -- src-tauri/tauri.conf.json src-tauri/capabilities
  src-tauri/Cargo.lock` — empty.
- `cargo test --locked --test config_invariants` re-run in isolation — `nfr_bench_is_absent_from_default_and_its_command_is_cfg_gated`
  (task 2's own gate) still green after this session's `lib.rs` edits.

**What was explicitly NOT attempted, and why — this is the honest boundary of task 5.**
`6-18-ban-do/build.sh`/`run.sh` have never been RUN. Building the release `.app` bundles the four
real `resources/dict/*.db` files (~372 MB combined) through the new `--config` merge, and the full
matrix is 10 sessions × (NFR4 cold+warm launches, NFR5 memory across full/frontier × three phases ×
10 samples) — the first session's own estimate ("50-work release-build-plus-multi-hour-benchmark")
undersells 6.18's real harness, which is ~3.3× larger than 5.14's already (10 sessions vs 3). That
run needs a genuinely idle machine (§Always: "no other cargo/npm/app process of this session") and
is task 7, not task 5 — attempting it inside this same turn would be exactly the mistake `AGENTS.md`
warns against (one agent swallowing a whole multi-hour story instead of handing off a phase). Two
concrete risks task 7 should watch for, spotted while writing the harness but not testable without
a real run: (1) the `--config` JSON merge for `bundle.resources` is asserted to be a Tauri CLI deep
merge (matching how 5.14's `build:beforeBuildCommand` override behaves today) rather than a full
replace of the `resources` object — `build.sh` DOES verify the resulting `.app` actually contains
`.db` files under `Contents/Resources/dict/` before declaring success, so a wrong-merge failure mode
is caught, just not exercised; (2) bundling ~372 MB adds real time/disk to `npx tauri build`, not
measured here.

**Handoff — tasks 6, 7, 8, 9 (still not started):**
- **Task 6** (debt probes, 11 `Chủ: Story 6.18` entries — 10 found by `grep -n "Chủ: Story 6.18"
  deferred-work.md` per the first session, the spec says 11; the 11th was not located in either
  session) needs a probe per entry and is naturally sequenced after task 5's release app exists,
  which it now does (unrun).
- **Task 7** (the real run: `6-18-ban-do/run.sh` on an idle machine, `REPORT.md`/`environment.txt`/
  raw TSV committed) is the harness this session built, actually executed. Budget real wall-clock
  time for it (see the size estimate above) and confirm the machine is idle first (`ps aux | grep
  -E 'cargo|npm|node|tauri'` empty) — §Always requires this explicitly, and the project's own
  memory ledger ("Hai lượt đo phải cùng tải máy") is exactly about this failure mode.
- **Task 8** (`prd.md`/`SPEC.md`/`requirements.md` verdicts) is explicitly gated by this spec's own
  §Always: *"Writing a changed number or a PRD-level verdict needs Ice's sign-off on the presented
  numbers"* — this cannot be done by an unattended agent regardless of how much of tasks 6–7 get
  finished first. If NFR5 lands in the 300.0–314.6 MB band, §Always also says stop for Ice to rule
  MB vs MiB.
- **Task 9** (`deferred-work.md`/`sprint-status.yaml` closing lines) follows task 6's numbers.

**Verification run this session (final, after both ignored builder/bench runs above):** `npm run
build && (cd src-tauri && cargo test --locked)` — green, `0` `FAILED`, `55` `test result: ok` blocks
(checked programmatically: `grep -c "test result: FAILED"` / `grep -c "test result: ok"` on the full
log, not by eyeballing a tail). `node scripts/check-debt-owner.mjs --report` — `mở KHÔNG có Chủ: 0`
(unchanged; this session added no debt entries and closed none). `git diff -- src-tauri/tauri.conf.json
src-tauri/capabilities src-tauri/Cargo.lock` — empty (checked; this session never touched any of the
three).

**One flake hit and cleared, recorded per `AGENTS.md`'s own convention (don't let a cleared claim
quietly lie).** A `cargo test --locked` run taken WHILE the 50×100×10 builder's `bench-release`
compile was hammering the CPU in the background showed
`store_contract.rs::the_wal_stops_growing_once_it_crosses_the_threshold` RED — a WAL-checkpoint
threshold test, inherently timing-sensitive (its own panic message: *"ĐỪNG nới trần theo phản
xạ"*). `store_contract.rs` is untouched by this session's diff (`git diff --stat | grep store` is
empty), so this could not be a regression from anything here. Re-ran it alone three times, and the
full suite once more, on a quiet machine (`ps aux | grep rustc` empty first) — GREEN every time,
`0` bytes of unexplained WAL growth on all three isolated runs. Matches this repo's own recorded
memory item "Hai lượt đo phải cùng tải máy" exactly: two measurements only compare under the same
machine load, and this session's own concurrent compile was that load. Not fixed, not touched — it
needed nothing fixed.

## Third session (2026-09-14) — task 6 closed (plus the trigram-ceiling half of task 4), tasks 7/8/9 partially open

**Scope closed this pass: task 6 (debt probes)** — every Rust/frontend-measurable probe the task
names, plus the trigram-ceiling half of task 4 that the second session's handoff flagged as the
real gap. Same discipline as the first two sessions: one phase, measured, written down, handed off
— no attempt at task 7 (the real multi-hour idle-machine run) or task 8 (Ice-gated PRD/SPEC/
requirements verdicts), both of which need conditions this session cannot manufacture (a
genuinely idle machine confirmed by Ice, and Ice's own sign-off on presented numbers).

**Why task 6 did not need task 5's release app for most of it.** Re-reading the Code Map's own
split: NFR4/NFR5 and "every UI probe" need the release `--features nfr-bench` app; the debt probes
this task names are Rust perf-probes in the `cleanup_contract.rs::perf_probe_*` mould (§Always:
"Rust benches `--profile bench-release`"), which run as plain `#[test]` functions, not through the
packaged `.app`. Only `chaptersShowAll`'s *render time* genuinely needs a real WKWebView — see below.

**Trigram-ceiling probe (task 4's open half).** `library_index_contract.rs::true_matches_piling_up_at_an_early_work_id_starve_a_later_work_ids_real_match_before_it_is_ever_read`
— tried several hand-crafted false-positive decoys first (both `sqlite3` CLI and Python's bundled
3.53.4) and found FTS5's `trigram` tokenizer's phrase-match is offset-strict (adjacent query
trigrams must land at adjacent document offsets), so none of the decoys produced a false MATCH —
recorded in the test's own doc-comment so a future reader doesn't retry the same guess. Fell back
to the honest lower bound: 2,600 *genuinely verified* matches in an early `work_id` (exceeding
`search_candidate_ceiling(51) = 2,550` at the real `limit = 50`) plus one genuine match in a later
`work_id` — the later Work's real hit is **never read at all** by the SQL `LIMIT <ceiling>`, not
merely outranked. `report.truncated == true` is the only signal a caller gets today. Verdict: the
architecture flaw is real and measured (not "chưa đo được" any more); a fix is an architecture
decision (search by Work, or rank candidates by likelihood instead of storage order) — Chủ: Ice.

**The other eight debt probes — one line each, full numbers in `deferred-work.md`'s own closing
notes (search each entry for "CẬP NHẬT 2026-09-14"); five were re-measured 2026-09-14 (audit P3)
under `cargo test --profile bench-release --locked --manifest-path src-tauri/Cargo.toml`, the
profile §Always actually requires — the first pass had wrongly used the `dev` (debug) profile for
plain `#[test]`/`#[ignore]` cases, or `--release` (order-dependent per the 5.14 spec's own record)
for the client-per-link cases. All five re-runs: machine MacBookPro16,1/macOS 15.7.9/16 logical
CPUs, commit `7863afbd7ea9a23c3db77b8cf0f1959172eb880e`, rustc 1.98.1, 2026-09-14:**
- `normalize()`'s two-pass `.replace()`: **62.0%** of `normalize()`'s own cost on a 10 MB `\r`-free
  chapter (debug had said 72.3%); a `contains('\r')` guard costs **95.5% less** (debug: 99.3%).
  Verdict unchanged — worth fixing, confirmed at both profiles, same order of magnitude.
- Peak RSS of the whole seven-step chain on a real 100 MB file: **979.1 MB** under `bench-release`
  (debug had said ~1,007.2 MB) — **~9.79×** the source size (debug: ~10.07×), same 990,233
  segments, but **9.153 s** wall time vs debug's 33.0 s (~3.6× faster). Verdict unchanged — peak RSS
  barely moved even though wall time dropped sharply (the ~990k live segments held in memory at
  once don't shrink under speed optimization); still ~10× source size at the 100 MB cap is a real
  number for A6/A7/A8's memory budget, now on the required profile.
- `ChapterPattern::match_starts` recompiling per call: **~55.0%** overhead across 7 repeated calls
  on 2,000 Chapters under `bench-release` (debug had said ~27.5%) — **the percentage roughly
  doubled**, not dropped, even though the absolute total for 7 calls fell from 46.26 ms (debug) to
  3.53 ms (release, ~13× faster): the "scan against an already-compiled Regex" baseline shrank
  faster than the recompile cost itself under release optimization, so the *share* attributable to
  recompilation grew. Still significant by the original >10% bar, though the absolute user-facing
  cost is now sub-4-ms per reload at this Chapter count.
- `Indexer::rebuild` across 50 real `.atproj` (the real 6.18 library, not a fixture): **1.285 s
  total**, 25.71 ms/`.atproj` (already `bench-release` in the first pass, unchanged) — cheaper in
  total than the old single-`.atproj` 50k-segment measurement (2,180.9 ms). Verdict: not a problem
  at 50-Work scale; "hundreds" still unmeasured.
- `reqwest::blocking::Client` per link at N=100 and N=1,000 (loopback, `bench-release`, idle
  machine): **0.53 ms/link** and **0.45 ms/link** (the first pass's `--release` run had said 0.49
  and 0.34 ms/link) — still both under 1 ms/link, still not increasing with N; the small shift
  between `--release` and `bench-release` is within run-to-run noise, not a reversed trend. Verdict
  unchanged — negligible at these sizes; the old N=20 alarm was machine load, not the mechanism
  (matches that test's own documented caveat).
- `chapter_detail_for_index` re-running the full preview pipeline per cursor move, on 2,000
  Chapters (`PipelineShape::Chapters`, the real URL-path shape — `Blob` only supports index 0,
  learned by first getting a `None` and re-reading `display_window_for_chapter`): under
  `bench-release`, **7.23 ms** for one cursor move, **43.05 ms** for 7 consecutive `⌥→` (average
  **6.15 ms/move**) — the debug pass had said ~36.53 ms / ~249.52 ms / ~35.65 ms average, roughly
  **5.8× slower** than the real number. **Verdict FLIPS**: the debug number's "real,
  user-perceptible" (>30 ms/keystroke) conclusion does not hold at the release number (6.15 ms
  average is well under most perceptibility thresholds). The cost is still O(Chapters) — it grows
  with library size, unmeasured yet at 5,000+ Chapters — so this is cheap-not-zero, not cleared.
- `webimport_contract`'s four wall-clock-flaky cases: **10/10 green** across 5 idle + 5
  concurrently-compiling runs on this session's machine (MacBookPro16,1, 16 logical CPUs — the
  same physical machine the original red report ran on; corrected 2026-09-14 P3 audit, an earlier
  draft wrongly called this "a container") — NOT reproduced, but NOT closed either: the load
  achieved this session (~5, same 16 cores) never reached the original report's load (19-30, same
  16 cores), and a miss at lower load doesn't clear a cause (`AGENTS.md`: "kiểm điều kiện đo trước
  khi lật quyết định"). Still needs a real CI-runner measurement. (Not re-run this pass — already a
  `--release`/idle-vs-loaded wall-clock comparison, not a profile-mismatch case.)
- `chaptersShowAll` at 1,000 Chapters, filter on: confirmed **1,000** real `<li>` rows on the DOM,
  zero virtualization, exactly as designed. Mount+`$nextTick` in `happy-dom` (vitest,
  `tests/frontend/importPreviewChapters.test.ts`) = **390.3 ms** — evidence, explicitly NOT the
  release-app/WKWebView number the task asks for (the harness's own doc-comment says `happy-dom` is
  not WKWebView; this is a vitest/happy-dom number, `--profile bench-release` does not apply to it).

**Owners still unnamed (Decision 4 requires "a named new owner"; none invented here — Ice names
them after seeing the re-measured verdicts above):** a `contains('\r')` guard before `normalize()`'s
two-pass `.replace()`; a peak-RSS budget response to the ~9.8×-source import-chain finding (if any);
a `Regex` cache for `ChapterPattern::match_starts`; and, if still wanted despite the flipped
verdict, cutting `chapter_detail_for_index`'s per-cursor-move `shape` down to the requested Chapter.

**Real verification performed this session:** every new test compiled and RUN (not just checked) —
`story_6_18_debt_probes.rs` (2 fast + 1 `#[ignore]` heavy, all three actually executed),
`library_index_contract.rs` (2 new cases, both `#[ignore]`-heavy ones run against the real 50×100×10
library rebuilt fresh for this session in `bench-release`), `webimport_contract.rs` (2 new
`#[ignore]` cases run in `--release`, plus the 5+5 idle/loaded flake probe run by hand),
`cleanup_contract.rs` (1 new case), `tests/frontend/importPreviewChapters.test.ts` (1 new case, run
both isolated and inside the full `npm run test`). Full-suite verification per spec's own
§Verification: `npm run build && (cd src-tauri && cargo test --locked)` — green, `0` `FAILED`, `57`
`test result: ok` blocks (up from the second session's 56 — one new test binary). `npm run test`
(vitest) — 74 files / 1,010 tests, all green. `node scripts/check-debt-owner.mjs --report` — `mở
KHÔNG có Chủ: 0` (every new/edited entry carries a `Chủ:`). `git diff -- src-tauri/tauri.conf.json
src-tauri/capabilities src-tauri/Cargo.lock` — empty. The full local `pre-push` (all fourteen
lines: eleven read-file gates, `npm run test`, `npm run build`, `cargo test --locked`) — all green,
154 s. (Per this repo's own memory ledger, a green local `pre-push` is not proof of CI — no push
was made this session, and CI has not run this diff.)

**deferred-work.md / sprint-status.yaml (task 9, partial — see the task-list bullet above for the
precise split).** Nine of the eleven `Chủ: Story 6.18` entries now carry a `🔵 CẬP NHẬT 2026-09-14`
closing note with the number, the verdict, and (where a fix looks warranted) an explicit "owner:
new, not this story" — this story's own §Never bars a product optimization, so no entry was closed
as `✅ ĐÃ ĐÓNG`; a debt entry moving from "chưa đo được" to "đo được, đây là con số" is the honest
shape here, not a fix. The two entries this session did NOT touch ("NFR3 sơ bộ" and "NFR4 vượt
trần vs biên độ phiên") stay exactly as the second session left them — their own closing condition
is Q4/A6-A8's real verdict (tasks 7/8), not a task-6 probe. A new "Deferred from: spec-6-18…"
section carries the Windows re-measurement entry (Chủ: Ice, points at B7). The `char_idx`
third-branch entry (Chủ: Ice) got today's baseline `rebuild` cost appended (no third table built —
out of this story's scope — just the number Ice needs to compare against). The "NFR5 vượt trần"
entry got nothing new — this session produced no NFR5 evidence.

**Handoff — tasks 7, 8, 9(remainder) still open:**
- **Task 7** unchanged from the second session's handoff: the real `6-18-ban-do/run.sh` matrix on
  a genuinely idle machine, multi-hour, needs the release `.app` built with real dictionary layers
  bundled (~372 MB). Not attempted here for the same reason the second session gave — this is a
  distinct phase, not something to fold into a debt-probe pass.
- **Task 8** stays gated on Ice's sign-off per §Always, unchanged.
- **Task 9's remainder** (the two NFR3/NFR4-general entries, and the NFR5-routes Ice-owned number)
  follows task 7's numbers — there is nothing further task 6's kind of probe can add to them.

## Fourth session (2026-09-14) — audit P3: six gaps found against the frozen spec, fixed and
## re-verified with real runs; tasks 7/8 still untouched (same gates as before)

An external step-3 audit read the diff and the machine directly (not the third session's own
report) and found six gaps. All six addressed this session, each with a real command run and raw
output captured below — no claim in this section is asserted without a paste. Machine for
everything in this section: **MacBookPro16,1, macOS 15.7.9, Intel Core i9-9980HK, 16 logical
CPUs** (`sysctl -n hw.model` / `sw_vers -productVersion` / `sysctl -n hw.logicalcpu` /
`sysctl -n machdep.cpu.brand_string`, run directly), commit
`7863afbd7ea9a23c3db77b8cf0f1959172eb880e` (unchanged — HEAD, tree dirty with this story's own
files only), rustc 1.98.1, cargo 1.98.1.

**1. Dirty-tree guard gap (would have killed task 7 at line one).** `6-18-ban-do/run.sh`'s
allowlist rejected four files this story's own task 6 added/modified:
`src-tauri/tests/cleanup_contract.rs`, `src-tauri/tests/story_6_18_debt_probes.rs`,
`src-tauri/tests/webimport_contract.rs`, `tests/frontend/importPreviewChapters.test.ts`. Added all
four to both `6-18-ban-do/run.sh`'s allowlist and `5-14-ban-do/run.sh`'s (the latter would hit the
same gap if run on this tree — same reasoning the earlier rename session used for
`config_invariants.rs`/`story_6_18_library.rs`). `zsh -n` on both files: clean.

**2. Five debt probes were measured on the wrong profile.** §Always: *"Rust benches `--profile
bench-release`; no debug."* The third session ran `normalize`, `match_starts`, and the
import-chain peak-RSS probe on plain `cargo test` (`dev`/debug — they are not
`--ignored`-gated-for-profile, just plain or `#[ignore]`-for-cost `#[test]`s, so no profile flag
meant debug), and the two client-per-link probes on `--release` (which the 5.14 spec's own record
says produces build-order-dependent results, distinct from `bench-release`). All five re-run this
session under the required profile; commands and raw stdout below, verbatim.

```
$ cd src-tauri && cargo test --profile bench-release --locked --manifest-path Cargo.toml \
    --test story_6_18_debt_probes -- --ignored --nocapture peak_rss
[peak_rss] tep nguon 104853588 byte tai /var/folders/xs/.../mot-chuong-100mb.txt
[peak_rss_of_the_whole_seven_step_import_chain] nguon=104853588B (~100.0 MB) chapters=1 \
  segments=990233 thoi_gian=9.153169698s dinh_RSS_tien_trinh=1002620 KB (~979.1 MB) \
  ti_le_dinh_RSS/nguon=9.79x
test peak_rss_of_the_whole_seven_step_import_chain_on_a_max_import_bytes_chapter ... ok
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 2 filtered out; finished in 14.11s
```

```
$ cargo test --profile bench-release --locked --manifest-path Cargo.toml --test cleanup_contract \
    -- --nocapture perf_probe_chapter_detail_for_index_repeated_across_seven_cursor_moves_on_two_thousand_chapters
[perf_probe_chapter_detail_for_index_cursor_moves] nguon 120890 byte, 2000 Chuong \
  (PipelineShape::Chapters, duong URL that), 1 luat literal — mot_lan_doi_con_tro=7.225508ms \
  7_lan_doi_con_tro_lien_tiep=43.054512ms trung_binh_moi_lan=6.150644ms
test perf_probe_chapter_detail_for_index_repeated_across_seven_cursor_moves_on_two_thousand_chapters ... ok
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 29 filtered out; finished in 0.06s
```

```
$ cargo test --profile bench-release --locked --manifest-path Cargo.toml --test story_6_18_debt_probes \
    -- --nocapture perf_probe_normalize_two_pass_replace_cost_on_a_ten_megabyte_crlf_free_chapter \
       perf_probe_match_starts_recompiles_the_regex_every_call
[perf_probe_match_starts_recompile] 2000 Chuong, 7 lan goi/luot tai. mot_lan_goi=993.162µs \
  bay_lan_goi_that=3.53358ms bay_lan_QUET_neu_da_cache_regex=1.588552ms \
  chi_phi_bien_dich_lai_uoc_tinh=1.945028ms (55.0% cua bay lan goi that)
test perf_probe_match_starts_recompiles_the_regex_every_call ... ok
[perf_probe_normalize_two_pass] nguon=10485781B (0 \r) two_pass_replace=23.368134ms \
  guard_check_contains_cr=1.053398ms whole_normalize=37.661692ms \
  replace_ti_trong_trong_normalize=62.0% guard_re_hon_bao_nhieu=95.5%
test perf_probe_normalize_two_pass_replace_cost_on_a_ten_megabyte_crlf_free_chapter ... ok
test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 1 filtered out; finished in 0.08s
```

```
$ cargo test --profile bench-release --locked --manifest-path Cargo.toml --test webimport_contract \
    -- --ignored --nocapture perf_probe_client_per_link_cost_on_one_hundred_links \
       perf_probe_client_per_link_cost_on_one_thousand_links
PERF_PROBE_CLIENT_PER_LINK	n=100	ok=100	total_ms=53.3	per_link_ms=0.53
test perf_probe_client_per_link_cost_on_one_hundred_links ... ok
PERF_PROBE_CLIENT_PER_LINK	n=1000	ok=1000	total_ms=453.4	per_link_ms=0.45
test perf_probe_client_per_link_cost_on_one_thousand_links ... ok
test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 34 filtered out; finished in 0.72s
```

Verdicts (full prose in `deferred-work.md`'s per-entry "SỬA LẠI 2026-09-14 (audit P3)" notes):
`normalize` (62.0% vs 72.3% debug, same conclusion), `rebuild` (already `bench-release`, untouched),
`client-per-link` (0.53/0.45 ms vs 0.49/0.34 ms `--release`, same conclusion) all **held**.
`match_starts`'s *percentage* moved the *wrong* way for a "release should look better" intuition —
55.0% vs 27.5% debug — because the cached-scan baseline shrank faster than the recompile cost under
optimization; still "significant" by the >10% bar, now on an absolute base under 4 ms total.
**`chapter_detail_for_index` verdict FLIPPED**: 6.15 ms/move average (release) is not
"user-perceptible" the way the debug number (35.65 ms/move) was.

**3. Machine identity was wrong in `deferred-work.md` and this spec.** Several notes called this
session's own machine "a sandboxed container" distinct from "Ice's Mac," and one called it "a
machine of unknown core count." Both are false: this session runs directly on the same physical
MacBookPro16,1 Story 5.14 measured on, confirmed by `sysctl`/`sw_vers` above. Every such sentence
corrected in place (`deferred-work.md`: the `webimport_contract` flake note, the trigram-ceiling
Windows-debt entry, the CI-runner-vs-local-machine line; this spec: the NFR3 bench machine/
toolchain line, the task-4 checklist bullet, the debt-probes summary intro and its
`webimport_contract` bullet) — none of these corrections change what machine *should* eventually
produce the A6/A7/A8 verdicts (still task 7, still an idle-machine confirmation Ice makes), they
only fix what machine *did* produce these evidence numbers.

**4. Matrix row "Build library" required build time + peak RSS; neither was recorded.**
`story_6_18_library.rs` now wraps the import-then-rebuild-full span (not the frontier/restore
exploration below it, a separate concern already covered by `transition-raw.tsv`) with a wall-clock
timer and a `ps -o rss=`-every-20ms sampler (same tool the peak-RSS debt probe and 5.14's own
WebKit sampling already use — no new crate), and prints one new line,
`STORY_6_18_BUILD_STATS\tbuild_wall_ms=…\tpeak_rss_kb=…\tpeak_rss_mb=…`. `6-18-ban-do/run.sh` now
parses that line (`die`s if absent) and copies all three fields into `fixture.txt`. Verified with
a real full-scale run, not just a compile check:

```
$ AURA_6_18_EXPORT_LIBRARY_ROOT=/tmp/auratranslate-nfr-bench-redctl-src/home/Documents/AuraTranslate \
    cargo test --profile bench-release --locked --manifest-path src-tauri/Cargo.toml \
    --test story_6_18_library -- --ignored --nocapture
STORY_6_18_BUILD	phase=import	works=50
STORY_6_18_BUILD	phase=rebuild_full
STORY_6_18_POPULATION	state=full	works=50	chapters=5000	segments=50000	done_chapters=5000	bytes=21726250	verdict=matches_declaration
STORY_6_18_BUILD_STATS	build_wall_ms=204570	peak_rss_kb=46660	peak_rss_mb=45.6	mau_moi_20ms_qua_ps_khong_phai_footprint
STORY_6_18_BUILD	phase=frontier_transition	work=…/NFR Story 6.18 Work 00.atproj
STORY_6_18_POPULATION	state=frontier	works=50	chapters=5000	segments=50000	done_chapters=4900	not_started_chapters=100	verdict=matches_declaration
STORY_6_18_POPULATION	state=final_export	works=50	chapters=5000	segments=50000	…
test builds_the_6_18_library_through_product_import_and_lifecycle_code ... ok
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 215.51s
```

204.57 s wall / 45.6 MB peak RSS to import+`done`-transition+rebuild the real 50×100×10 library
end to end, on this machine, this session (not a claim about `run.sh`'s own eventual full-matrix
numbers, which also run NFR3/NFR4/NFR5 afterward under task 7).

**5. Matrix row "Red controls" had never been run.** Two real, separate runs, neither through
`run.sh` itself (task 7 stays off limits this pass) — each is `run.sh`'s own logic, extracted and
run standalone against a deliberately broken input, exactly as §I/O Matrix names it.

*Population off-by-one.* Copied the real export above, deleted one Chapter's rows (`DELETE FROM
segment WHERE chapter_id=1; DELETE FROM chapter WHERE id=1;` on the COPY only — SQL permitted here
because this is a red control, not the library-building path §Always governs), then ran
`run.sh`'s own read-back block (lines "đọc lại quần thể ĐỘC LẬP với builder" verbatim) against the
copy:

```
== đọc lại quần thể ĐỘC LẬP với builder — §Always: kiểm trước bất kỳ lượt đo nào ==
LỖI: quần thể đọc lại sai: works=50 chapters=4999 segments=49990 (kỳ vọng 50/5000/50000)
exit=1
```

Green control (same block, uncorrupted original): `quần thể khớp -- không có gì để từ chối`,
`exit=0` — the check does not always fail.

*Layers stripped.* Built the release app with feature `nfr-bench` (Story 6.18's probe injected)
but **without** the dict `--config` merge (`npx tauri build --bundles app --features nfr-bench
--config '{"build":{"beforeBuildCommand":""}}'`, i.e. the 5.14-style build, no
`bundle.resources` merge) — confirmed `find Contents/Resources -iname '*.db'` empty and
`Contents/Resources/dict` absent, i.e. genuinely 0 layers, not `src-tauri/resources/dict/` touched
in any way. Launched it (`HOME` pointed at a scratch copy of the real 50-Work library so the
probe's grid check can reach `usable`) and read the real marker from `global.db`:

```
== marker usable (RAW) ==
{"epoch_ms":1789403171704,"layers":0,"performance_epoch_ms":1789403171704,"work_name":"NFR Story 6.18 Work 00","works":50}
== §Always spec 6.18 refusal check -- literal node -e line từ 6-18-ban-do/run.sh ==
die "0 lớp từ điển đã nạp trước mẫu đầu tiên …" (node exit=2, run.sh sẽ thoát khác 0 ở đây)
```

`layers:0` is server-side (`DictLayers::layers().len()`, not client JS), and `run.sh`'s own
`node -e` refusal line, run verbatim against this real marker, exits 2 — `run.sh` would `die` here
exactly as §I/O Matrix requires. All scratch homes/copies removed after
(`/tmp/auratranslate-nfr-bench-redctl-*`); `src-tauri/resources/dict/` was never touched. The
release `.app` this left at `target/release/bundle/macos/AuraTranslate.app` has no dict layers —
task 7's own first step (`6-18-ban-do/build.sh`) rebuilds it properly (with the dict merge) before
running anything, so this is not a lasting state, only worth noting for whoever runs task 7 next.

**6. Owners not invented.** Several closing notes said "owner: MỚI, chưa có" — per Decision 4 that
is not a named owner, so none was invented. Existing `Chủ:` lines on every entry are untouched.
**Entries still needing Ice to name an owner, if a fix is wanted:** (a) a `contains('\r')` guard
ahead of `normalize()`'s two-pass `.replace()`; (b) any response to the ~9.8×-source peak-RSS
import-chain finding; (c) a `Regex` cache for `ChapterPattern::match_starts`; (d) cutting
`chapter_detail_for_index`'s per-cursor-move `shape`, if still wanted despite the flipped verdict.

**Verification this session:** `sh .githooks/pre-push` (the real fourteen-gate hook, not a
reimplementation) run directly, no other cargo/npm/app process running first (checked via `ps aux`
immediately before): all fourteen gates green, 202 s total (eleven `check:*` gates, `npm run test`,
`npm run build`, `cargo test --locked`). `node scripts/check-debt-owner.mjs --report` — AC5's four
numbers unchanged (761 total / 503 open / 87 half / 161 closed / 10 KHÔNG LÀM / **0 mở KHÔNG có
Chủ:**), since this session only appended to existing entries, never added a new bare one.
`git diff -- src-tauri/tauri.conf.json src-tauri/capabilities src-tauri/Cargo.lock` — empty
(unchanged this session; this pass never touched any of the three).

**What is still NOT done, unchanged from the third session's handoff:** **Task 7** — the real
`6-18-ban-do/run.sh` full matrix on an idle machine, multi-hour — still not started; this session's
`.app` builds (with and without dict) were narrow, single-purpose verification runs for items 4/5
above, not the harness's own orchestrated run. **Task 8** — `prd.md`/`SPEC.md`/`requirements.md`
verdicts — still gated on Ice's sign-off per §Always and untouched. Neither hard limit given for
this pass was crossed: `6-18-ban-do/run.sh` itself was never executed, and no commit was made.

## Spec Change Log

## Review Triage Log

## Design Notes

**Why command functions count as the product path.** The IPC shell adds only serde around the pure
`preview_*` / `confirm_*` functions; what lands on disk is identical. Driving 50 imports through
WebDriver would measure the driver and add nothing to the library.

**Lifecycle cost.** The public `set_chapter_status` wire rebuilds the whole index per call: 5,000
rebuilds over 50 `project.db`. Pure `set_chapter_status` per Chapter, then one `Indexer::rebuild`,
both timed.

**What the shape changes.** Reading runs over the open Work only, so the *full* Reading phase holds
at most 100 Chapters / 1,000 segments, against 50,000 in 5.14. The NFR5 Reading number is therefore
not comparable with 5.14's 894,570,496 bytes; the report states this next to the number.

**Dictionary layers.** The bench build bundles `resources/dict/*.db` into `dict/` through a
build-time config merge, the directory `open_dict_layers` reads. This does not decide release
bundling (AI-1, Story 10.1).

## Verification

**Commands:**
- `npm run build && (cd src-tauri && cargo test --locked)` -- exit 0; benches stay `#[ignore]`.
- `cargo test --profile bench-release --locked --manifest-path src-tauri/Cargo.toml --test story_6_18_library -- --ignored --nocapture` -- population printed, equals declaration.
- `_bmad-output/implementation-artifacts/6-18-ban-do/run.sh` -- complete matrix, `REPORT.md` generated.
- `node scripts/check-debt-owner.mjs --report` -- 0 open items without `Chủ:`.
