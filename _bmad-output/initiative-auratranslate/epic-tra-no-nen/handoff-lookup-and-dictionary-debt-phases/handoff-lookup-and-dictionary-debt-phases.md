---
type: handoff
title: "Story 11.3 — Phase 1 (Rust) handoff, 2026-09-26"
status: done
created: 2026-09-26
skill: bmad-build
---

# Story 11.3 — Phase 1 (Rust) handoff, 2026-09-26

Working notes for the next agents (Phase 2 Webview, Phase 3 Tests/ledger). Not a spec —
see `spec-11-3-lookup-and-dictionary-debt.md` for intent/decisions. Not copied into the
ledger — Phase 3/4 writes the `deferred-work.md` `→` dispositions from here + the spec.

## What phase 1 did (all in `src-tauri/`, no commit made)

Files touched: `src/core/dict/mod.rs`, `src/core/dict/query.rs`, `src/commands/dict.rs`,
`tests/dict_sources.rs`, `tests/matching_boundary.rs`, `tests/ipc_contract.rs`.

- **L840** — `layers_loaded` at both sites (`GroupedLookup` in `mod.rs` ~L935,
  `HanVietLookup` ~L1225) now `!layers.layers().is_empty() || !layers.skipped().is_empty()`.
  Guard: `layers_loaded_is_true_when_every_layer_is_skipped_not_only_when_some_load`.
  Counter-checked BOTH sites independently (reverted one `||` at a time) — each reds on
  its own assertion.
- **L1162 (Rust half only)** — `core::dict::list_source_attributions` now returns a new
  public type `SourceAttributions { sources: Vec<SourceAttribution>, skipped:
  Vec<SkippedLayer> }` instead of a bare `Vec<SourceAttribution>`. `skipped` serializes
  through the same `serialize_skipped_as_wire_codes` helper `GroupedLookup::skipped`
  already uses (wire codes only, never `detail`/`path` — AD-21). A layer whose `dict_source`
  fails to read *at attribution time* (distinct from a layer that never opened) now lands
  in `skipped` with `SkipReason::SourcesUnreadable` instead of silently vanishing behind an
  `eprintln!`. `commands::dict::list_sources` and the `#[tauri::command] list_dict_sources`
  wire now propagate the same shape. Guard:
  `a_layer_whose_attribution_columns_go_missing_after_open_is_named_in_skipped`.
  Counter-checked (reverted to old drop-and-eprintln behavior) — reds correctly.
  **The Vue-side consumption is NOT done** — see "For phase 2" below.
- **Decision 6 (L831, option A)** — `commands::dict::LookupResponse` gained a fourth field
  `senses_failed: Vec<String>`. `lookup()`'s phase-two hydrate loop now `match`es
  `layer.senses(&entry_ids)` instead of `.unwrap_or_default()`: on `Err`, the layer name
  goes into `senses_failed` and is **absent** from `senses_by_layer` (never both). Guard:
  `a_layer_whose_hydrate_breaks_after_a_hit_is_named_failed_not_emptied`. Counter-checked
  (reverted to `unwrap_or_default()`) — reds correctly.
- **L604 + L607(a)(b)(c)(e)(f)(g)** — six new `dict_sources.rs` tests filling the missing
  `SkipReason` variants and sub-items (see file for exact names): `OpenFailed` +
  L607(f) folded into one test (`a_directory_named_dot_db_is_rejected_as_open_failed`,
  empirically confirmed via a throwaway `rusqlite` probe that opening a directory as a
  `.db` fails immediately with "disk I/O error", not a hang or panic);
  `MetaRowMissing`'s two keys in one test; `SourcesUnreadable` (table dropped after
  build); `DuplicateLayer` (two files, same `layer`, different source codes — the
  existing `two_layers_claiming_the_same_source_code_is_a_named_data_error` only covered
  `DuplicateSourceCode`); `LookupFailed` + L607(e) folded into one test (`dict_entry`
  dropped after build — `open()` never reads it, only a `lookup()` call breaks);
  L607(a) uppercase `.DB`; L607(b) `#[cfg(unix)]` permission-denied directory (safe:
  `ci.yml`'s matrix is `[macos-26, windows-2025]`, no Linux, so this simply doesn't
  compile on the Windows leg rather than being skipped at runtime); L607(c) — deleting
  the layer that "won" a `dict_source.code` conflict frees the code for the loser on the
  **next** `DictLayers::open()` call (proves the conflict lock is recomputed per-open, not
  sticky state). All six/eleven counter-checked except five (see "Not counter-checked"
  below).
  - **Correction to Task 0's L607(g) claim**: task0 said "FtsTrigram/CharIdx 2-char only
    in the `#[ignore]`d bench, no assertion at the aggregation layer" for *both* branches.
    That's only true for **FtsTrigram** — `source_counts_are_verified_not_candidate_counts`
    (pre-existing, line ~1448) already exercises the **CharIdx** 2-char verified-count path
    at the aggregation layer (`lookup_grouped`/`count_by_source`). I added
    `verified_counts_survive_the_fts_trigram_branch_too` for the genuinely-missing
    FtsTrigram half only. Flag this correction when Phase 3/4 writes the ledger `→` line
    for L607.
- **L426** — `branch_one_and_two_never_scan_the_table`, new EXPLAIN QUERY PLAN test.
  **Important finding, measured by hand before writing it**: the existing small fixtures in
  this file (≤ 20 rows, or `headword_simp` uniformly `NULL`) make SQLite's cost-based
  planner choose `SCAN dict_entry` regardless of the indexes present — verified with a
  throwaway `sqlite3`/Python probe scaling from 6 to 100,000 rows. The planner only
  switches to `MULTI-INDEX OR`/`SEARCH … USING INDEX`/`USING PRIMARY KEY` once (a) the
  table has meaningfully more than ~20-50 rows **and** (b) `headword_simp` has genuine
  per-row selectivity (not all-`NULL`, which `ANALYZE`'s `sqlite_stat1` then reports as
  "1 distinct value" and kills the OR-branch's usefulness). The new test builds its own
  300-row fixture (`build_query_plan_fixture`, ~25% of rows carry a distinct
  `headword_simp`) via raw DDL (`COPIED_DDL`, same constants as everywhere else in the
  file) — `build_layer`/`LayerSeed` can't be reused here because they require
  `&'static str` fields, which a generated fixture can't produce without leaking memory.
  Query text is a **hand copy** of `query.rs::exact`/`char_idx`'s SQL (unavoidable:
  `query::exact`/`char_idx` are `pub(super)`, and `store_boundary.rs` forbids
  `rusqlite`/`Connection::open` outside `core/store/**`, so `query.rs` can't EXPLAIN its
  own SQL in a unit test). **No automatic parity check exists for this SQL copy** — unlike
  `COPIED_DDL`, which `fixture_ddl_is_verbatim_from_dict_build_schema` guards. If `query.rs`
  ever changes the `WHERE`/`JOIN` shape of branch 1/2, this test's copy must be updated by
  hand; flag this as a small residual risk for whoever touches `query.rs` next.
  Counter-checked (removed the two `CREATE INDEX` lines from `ENTRY_INDEXES_DDL`
  temporarily, in `tests/dict_sources.rs` only — never touched
  `tools/dict-build/src/schema.rs`) — reds with "SCAN e" as the exact assertion text names.
- **L434** — new `query::ShortQuery<'a>` type replaces the `debug_assert!` on
  `char_idx`'s "≤ 2 characters" precondition. `ShortQuery::new(&str) -> Option<Self>` is
  the only constructor (private tuple field); `char_idx` now takes `ShortQuery<'_>`
  instead of `&str`. The two call sites (`mod.rs::lookup_with_branch`'s `CharIdx` arm,
  `query.rs::count_by_source`'s `CharIdx` 2-char arm) both handle `None` by returning an
  empty result (`(Vec::new(), false)` / `Ok(Vec::new())`) rather than panicking —
  panicking would kill the whole process (`panic = "abort"`), which is a worse outcome
  than the old silent-truncate-to-2-chars behavior it replaces; both are unreachable on
  the real product path (branch always agrees with query length there), so "return empty"
  is the same "least harmful" doctrine `query.rs::effective_limit` already uses. Inline
  `#[cfg(test)] mod short_query_tests` in `query.rs` (three tests). Counter-checked
  (made `ShortQuery::new` always return `Some`) — two of three inline tests red for the
  right reason.
- **L445** — `matching_boundary.rs` module doc-comment and the assert message of
  `the_dictionary_lookup_path_never_calls_the_matcher` both had the now-false clause
  ("epics.md still says `dict/` dùng nó") removed; the still-true mermaid-diagram clause
  (Winston-owned, `ARCHITECTURE-SPINE.md`) kept. Text-only, no assertion-logic change, no
  new guard (per spec, this needed no counter-check).

**Not counter-checked** (verified green, but I did not remove the specific seam and
re-run): `a_missing_meta_row_names_which_key_is_absent`,
`a_layer_whose_dict_source_table_is_gone_is_sources_unreadable`,
`a_layer_whose_entry_table_is_gone_fails_lookup_without_taking_down_the_rest`,
`a_permission_denied_scan_is_an_empty_layer_set_not_a_panic`,
`deleting_the_layer_holding_a_source_code_frees_it_for_the_loser_on_reopen`. These all
exercise pre-existing production logic (the `SkipReason` variants and `conflict_with`
already existed before this story); the tests are new, but I ran out of phase budget to
individually remove each seam. Flag for Phase 3's counter-check sweep, or do it before
`done` if time allows: for `MetaRowMissing`, temporarily delete the `let Some(schema_version)
= schema_version else { return Err(...) }` (and separately the `layer` one) in
`layer.rs::DictLayer::open`; for the two "table gone" tests, the seam is trivially real
already (dropping the table IS the seam — the guard is that `open()`/`lookup()` handle it
via `Result`, which the test's other assertions (`skipped().is_empty()` before the corrupt
call reaches it) already probe); for the permission test, temporarily change the
`ErrorKind::NotFound` match arm in `layer.rs::DictLayers::open` to catch *all* errors the
same silent way (it already does — the risk is narrow); for the delete-and-reopen test,
temporarily make `conflict_with` remember rejections across calls (there's no such state
today, so this one is closer to "impossible to fail by construction" than "unguarded").

**Verification commands run** (all green): `cargo check --tests`; `cargo test --lib`
(246 passed); `cargo test --test dict_sources --test matching_boundary --test ipc_contract
--test ipc_argument_contract --test dict_lookup --test dict_boundary --test store_boundary`
(89 + 10 + 36 + 1 + 39 + 16 + 4 passed, 0 failed). `npm run build` was already done (dist/
present) before these ran.

## New/changed wire shapes — what Phase 2 (Webview) must consume

1. **`lookup_dictionary` / `commands::dict::lookup`'s `LookupResponse`** gained
   `senses_failed: Vec<String>` (layer names). `dictSourcesState.ts`/wherever
   `senses_by_layer` is read needs a third case: a layer name in `senses_failed` means
   "this layer's meanings failed to load", distinct from "this layer has no groups" (not
   present at all in `grouped.groups`) and from "this layer hydrated to an empty list"
   (impossible now — a layer only appears in `senses_by_layer` on success). This is the
   webview half of L831/Decision 6.
2. **`list_dict_sources` (`commands::dict::list_sources`)** no longer returns a bare
   array. It now returns `{ sources: SourceAttribution[], skipped: string[] }` (the
   `skipped` array holds wire codes like `"open_failed"`, `"sources_unreadable"`, etc. —
   same catalogue as `GroupedLookup.skipped`, see `SkipReason::wire_code` in
   `core/dict/layer.rs`). `src/panels/dictSourcesState.ts` and
   `src/AttributionOverlay.vue` (~L187-201 per the spec's Code Map) need to switch from
   consuming a plain array to `{ sources, skipped }`, and `AttributionOverlay` needs the
   **third state** the spec's Disposition 1 asks for: "some layers unreadable", distinct
   from `empty` (no sources at all) and from a normal populated table. This is the
   webview half of L1162.
3. Nothing else in the IPC surface changed shape. `list_dict_sources`'s **parameters**
   didn't change (still no args), so `ipc_argument_contract.rs` stays green untouched.

## Decision 8 / L352 — measured skip ratios, proposed N (Ice must sign before landing)

**Build and population** (so this measurement can be trusted or challenged): real
`cargo run --release -- --raw raw --out-dir <scratch> --layer all` in
`tools/dict-build/`, using the raw source files already present on disk under
`tools/dict-build/raw/**` (cvdict, cc_cedict, unihan, viwiktionary, viwiktionary→en role,
en_wiktionary, en_wiktionary_vi, thieu_chuu, tran_van_chanh, vietphrase — all ten sources
that exist today). Output was NOT written anywhere inside the repo (built to a scratch
directory outside the repo tree, per AD-25 — never commit `.db`). Full stdout is not
preserved beyond this note; rerun the same command to reproduce (takes well under a
minute; `Chinese.jsonl` alone is 1.18 GB but the build is fast).

Per-source `lines_read` / `lines_skipped`, and the **raw** ratio (`lines_skipped /
lines_read`, exactly what Task 0's guard sketch and the ledger text describe):

| source | lines_read | lines_skipped | raw ratio |
|---|---|---|---|
| cvdict | 122,597 | 1 | 0.00082% |
| cc-cedict | 124,758 | 0 | 0% |
| unihan | 49,870 | 0 | 0% |
| viwiktionary | 415,115 | 413,517 | **99.61%** |
| en-wiktionary | 306,358 | 131,681 | **42.99%** |
| viwiktionary-en | 401,101 | 282,062 | **70.32%** |
| en-wiktionary-vi | 51,198 | 48,966 | **95.64%** |
| thieu-chuu | 9,898 | 1 | 0.0101% |
| vietphrase | 679,311 | 9 | 0.0013% |
| tran-van-chanh | 22,030 | 0 | 0% |

**Why the raw ratio alone is not a usable threshold, with evidence**: `viwiktionary`,
`viwiktionary-en`, and `en-wiktionary-vi` read the *same* underlying files
(`vi-extract.jsonl`, `kaikki-en-vi.jsonl`) more than once with different `lang_code`/`pos`
filters by design (`build.rs`'s own doc-comment, "SÁU nguồn nhưng NĂM thư mục"; also see
`wiktextract_common.rs`). Every one of those filter-mismatch skips is *already tagged* in
the skip reason string with the literal substring `"filtered, expected"` — a marker the
codebase already relies on elsewhere (`sources/viwiktionary_en.rs:72` and
`wiktextract_common.rs:470` both do `.contains("filtered, expected")`). Recomputing the
ratio **excluding** any skip reason containing that marker gives a very different, much
more meaningful picture:

| source | genuine skips (marker excluded) | genuine ratio |
|---|---|---|
| cvdict | 1 | 0.00082% |
| cc-cedict | 0 | 0% |
| unihan | 0 | 0% |
| viwiktionary | 1,707 ("no usable glosses on any sense") | **0.411%** |
| en-wiktionary | 131,681 ("no usable glosses on any sense", no marker) | **42.99%** (unchanged — nothing to exclude) |
| viwiktionary-en | 127 ("no usable glosses on any sense") | **0.0317%** |
| en-wiktionary-vi | 6,655 ("no han-viet-reading/nom-reading tags") | **13.00%** |
| thieu-chuu | 1 | 0.0101% |
| vietphrase | 9 | 0.0013% |
| tran-van-chanh | 0 | 0% |

**Two structural options for Ice, per AGENTS.md's "present both with measurements" rule**
(this is exactly that case — I did not pick one):

- **(A) Raw ratio** (`stats.lines_skipped as f64 / stats.lines_read as f64`, the shape
  Task 0 sketched verbatim). To stay green on today's build, `N` must be ≥ ~99.7%
  (above `viwiktionary`'s 99.61%). At that `N`, the check protects against nothing a
  regression would realistically produce short of a source going almost entirely
  unparseable — which `require_nonempty`'s existing `entries == 0` check already catches
  when it happens completely, and a ratio near 100% is barely distinguishable from that.
  Cost to implement: trivial (Task 0's one-line sketch, unchanged). Cost to safety: this
  option buys close to nothing beyond what already exists.
- **(B) Genuine-only ratio** (sum `SourceStats::skip_reasons` values whose key does
  **not** contain `"filtered, expected"`, divide by `lines_read`). Every source's
  genuine ratio today sits at ≤ 43%, with the next-highest at 13% and everything else
  under 1%. A threshold like `N = 60%` would stay comfortably green today (43% + real
  margin) while catching a regression that pushes any source's *unexplained* skip rate
  materially above its current baseline (e.g., an encoding break, a filter bug, a
  format-version drift silently dropping a majority of lines). Cost to implement: a few
  lines inside `require_nonempty` (or a small helper it calls) to filter
  `skip_reasons` by that substring before summing — not a schema or interface change,
  and the `"filtered, expected"` marker already exists in the strings, unlocked by
  option B, not invented for it.

**My recommendation, for Ice to accept or override**: option (B) with `N = 60`. It is the
only one of the two that a regression could realistically trip, and the "filtered,
expected" split it needs is already-written vocabulary in the codebase, not a new
convention. Whichever N Ice signs, the guard test the spec asks for ("a fixture where
> N% of lines are unparseable-but-1-line-succeeds must fail the build") and its
counter-check (revert the ratio check, keep only `entries == 0`, confirm that same
fixture "succeeds") are **not yet written** — that is explicitly Phase 3/4's job per the
spec's task list ("wait for Ice's signature, then land the check with a near-empty-decode
fixture"), not this phase's.

## Nothing else changed

`deferred-work.md` untouched. The spec's frozen block untouched. No commit made. No
webview file touched. `tools/dict-build/src/schema.rs` untouched, `SCHEMA_VERSION`
unbumped. No new dependency, no new gate. `pick_branch`'s length metric and the project's
Unicode-normalization behavior (L437/L7753/L789, Winston's) untouched.

## Phase 2 (Webview)

Scope actually done: Disposition 2 (L1005, L4142, L1042 ①②) + the webview halves of L1162
and L831 (Decision 6). L1042 ④ (e2e) left for Phase 3, per the coordinator's instruction.
No Rust file touched, no commit made, `deferred-work.md` untouched, spec's frozen block
untouched.

**Files touched:**
- `src/config/dict.ts` — `LookupResponse` gained `senses_failed: string[]`. `list_dict_sources`'s
  wire shape changed from a bare `SourceAttribution[]` to a new `SourceAttributions { sources,
  skipped: string[] }` type (matches the Rust `SourceAttributions` from Phase 1); `ListDictSourcesResult`
  and `listDictSources()` updated to carry `skipped` through.
- `src/panels/dictSourcesState.ts` — new `sourcesSkipped` ref + exported `dictSourcesSkipped`/
  `someDictSourceUnreadable` (computed). `loadDictSources`/`resetDictSources` updated to
  read/clear `skipped`.
- `src/AttributionOverlay.vue` — third state (Disposition 1/L1162): a `someDictSourceUnreadable`
  banner (`.attr-partial`, key `attribution.some_unreadable`) that (a) can show alongside a
  populated table, and (b) — the actual AC — suppresses `attribution.empty` ("chưa gắn lớp từ
  điển nào") when `dictSources` is empty AND some layer was skipped, since that combination means
  "layers exist but are unreadable," not "no dictionary installed."
- `src/panels/lookupPanelState.ts` — new `sensesFailedLayers` (computed Set) + `layerSensesFailed(layer)`
  pure predicate (Decision 6/L831 webview half), reading `response.senses_failed`.
- `src/panels/LookupPanel.vue` — passes `:senses-failed="layerSensesFailed(group.layer)"` into
  each `<LookupRecord>`.
- `src/panels/LookupRecord.vue` — new optional `sensesFailed` prop; when true, renders
  `.lookup-senses-failed` (key `panel.lookup.senses_failed`) once per entry cluster instead of
  the (necessarily empty) `cluster.senses` loop, so a phase-two hydrate failure no longer reads
  as "this entry has no meanings."
- `src/panels/SourceHanViet.vue` — **L1005**: `onCopy` now always rebuilds the clipboard string
  through `resolveSelection` (⇒ `resolveParallel`/`resolveSwitch`), dropping the old
  `text.includes(WORD_JOINER)` gate (that gate only ever matched in `switch` view — `parallel`
  never emits `WORD_JOINER`, so a copy in `parallel` view always fell through to
  `Selection.toString()`, which leaks Hán Việt readings on WKWebView). **L4142**: `.hv-unit` (the
  `parallel`-view Hán-word `<ruby>` span) now carries `data-src-atomic="1"`, matching phiếu quyết
  #89 (Ice, 2026-09-24) — a cut mid-Hán-word in parallel view now snaps to the word start instead
  of counting characters inside it.
- `src/panels/editorSegments.ts` — `SRC_ATOMIC` doc-comment's stale "does NOT cover `.hv-unit` in
  `parallel`" clause replaced with a note pointing at phiếu quyết #89 (the clause was the accepted
  debt item L4142 closes). Only the comment changed; `sourceCutOffsetOf`/`neoNguonCua` logic
  untouched (it already reads the attribute generically).
- `src/i18n/vi.json` — two new keys: `attribution.some_unreadable`, `panel.lookup.senses_failed`.
- `tests/frontend/hanVietCutAnchors.test.ts` — flipped the two assertions the L4142 debt item
  named (L97-110's now-renamed test, and the parametrized L162-175 comment+assertion), per the
  spec ("the fixture-only test stays… must flip").
- **New test files**: `tests/frontend/sourceHanVietCopy.test.ts` (L1005 guard),
  `tests/frontend/lookupSourceChipToggle.test.ts` (L1042 ①②), `tests/frontend/lookupSensesFailed.test.ts`
  (Decision 6/L831 webview — one case for `lookupPanelState.ts::layerSensesFailed`, two for
  `LookupRecord.vue`'s rendering), `tests/frontend/attributionOverlayUnreadable.test.ts`
  (Disposition 1/L1162 webview — three cases: all-skipped, mixed, and a regression guard for the
  unchanged old `empty` behavior).

**Design note on L1042 ①②/Decision 6/L1162 guards**: none of these three go through the real
`dispatch('lookup.toggle_source')`/`CommandRegistry` pipeline — that's only wired up in
`src/main.ts` (`installCommands(...)`), which isn't importable into a component test without
booting the whole app. Each test instead calls the **real handler function directly**
(`toggleDictSource` from `dictSourcesState.ts`, which its own doc-comment calls "Handler thật của
`lookup.toggle_source`"), or mocks `@tauri-apps/api/core`'s `invoke` to drive `loadDictSources`/
`runLookup` through their real bodies. This is the same pattern `editorAutoLookup.test.ts`
already uses (calls `attachSelectionWatcher`/`registerSelectionSurface` directly rather than
simulating a keystroke through the full command graph).

**Counter-checks — all 5 guards, seam really removed, confirmed red for the right reason, then
restored:**
1. L1005 — reverted `onCopy` to the old `text.includes(WORD_JOINER)` gate ⇒
   `sourceHanVietCopy.test.ts`'s parallel-mode case red (clipboard came back `''` instead of
   `'京都」，'`).
2. L4142 — removed `data-src-atomic="1"` from the `.hv-unit` template span ⇒ both
   `hanVietCutAnchors.test.ts` cases that assert it red (`undefined` instead of `'1'`).
3. L1042 ①② — removed the `:class="{ off: sourceIsDisabled(src.code) }"` binding from the chip
   button ⇒ `lookupSourceChipToggle.test.ts` red (`classes()` missing `'off'`).
4. Decision 6/L831 — removed the `v-if="sensesFailed"` banner from `LookupRecord.vue` (back to
   always rendering the — now silently empty — `cluster.senses` loop) ⇒
   `lookupSensesFailed.test.ts`'s rendering case red.
5. Disposition 1/L1162 — removed the `someDictSourceUnreadable` banner + the
   `&& !someDictSourceUnreadable` guard on `attribution.empty` ⇒ both non-regression cases in
   `attributionOverlayUnreadable.test.ts` red (banner missing; `empty` wrongly shown when all
   layers were skipped).

**Verification run this phase:** `npm run build` (vue-tsc ×2 + vite build) green, no type
errors. `npx vitest run tests/frontend` — full frontend tree, **104 files / 1503 tests, all
green** (run once, at the end, because the touched types — `LookupResponse`, the new
`SourceAttributions` — are shared across several modules; not a per-edit full-suite run).
`npm run check:tokens`, `npm run check:commands`, `npm run check:layout`, `npm run check:doc-refs`
all green. `npx eslint` clean on every touched/new file.

**🔴 `npm run check:i18n` is RED, but not from anything in this phase** — Kiểm A flags two
Vietnamese `assert!` messages at `src-tauri/src/core/dict/query.rs:573-574`
(`ShortQuery::new("中國人").is_none(), "ba ký tự Hán — phải bị chặn"`, and its neighbour), inside
`#[cfg(test)] mod short_query_tests` — **Phase 1's** new inline test module for L434. The
exemption list covers `src-tauri/tests/**` (a separate directory) for exactly this reason
("thông báo `assert!` — không vượt IPC"), but this module lives inside `src/core/dict/query.rs`
itself, which Kiểm A's `.rs`/`.vue` sweep does not exempt. This is a Rust file — out of scope for
this phase to fix. **Phase 3/4 must resolve this before `done`**: either translate those two
assert messages to English (cheapest — a `#[cfg(test)]` inline module has no reader who needs
Vietnamese, same reasoning the `tests/**` exemption already gives), or extend the named exemption
to cover inline `#[cfg(test)]` modules under `src/**`. Do not just add another named-exemption
line without checking whether other inline `#[cfg(test)]` modules would suddenly also need
covering.

**Not done, left for Phase 3 as instructed:** L1042 ④ (the e2e that Tabs from the chip strip to
the Attribution rows) — untouched, per the coordinator's explicit "leave it." Decision 8/L352's
Rust-side guard test + counter-check (Ice's `N` still needs signing per Phase 1's notes above) —
unaffected by this phase, still Phase 3/4's job. `deferred-work.md`'s 36 `→` dispositions — not
started this phase; Phase 3/4 writes them from this file + the spec, per the spec's own rule
("Phase handoff file = working notes for the next agent, never copied into the spec... Link,
don't copy").

## Phase 3 (Tests that move + ledger)

Scope actually done: the `tools/dict-build` ratio guard (Decision 8), the red `check:i18n`,
five missing counter-checks flagged by Phase 1, the L1042 ④ e2e attempt (and its real outcome),
Story 2.9's 🔵 pointer, and all 36 `deferred-work.md` `→` dispositions. No commit made, spec's
frozen block untouched.

- **Decision 8/L352** — `tools/dict-build/src/build.rs`: `MAX_GENUINE_SKIP_RATIO = 0.60` +
  `genuine_skip_ratio()` (sums `skip_reasons` whose key does **not** contain `"filtered,
  expected"`, divides by `lines_read`) wired into `require_nonempty` right after the existing
  `entries == 0` check. Three unit tests in a new `require_nonempty_ratio_tests` module
  (same file, same pattern as the pre-existing `distribution_table_tests`): a 90%-genuine-skip
  fixture must fail; the same 90% but entirely tagged `"filtered, expected"` must pass (ratio
  0%); a 20%-genuine fixture (under N) must pass. Built via `SourceStats::new` +
  `record_entry`/`record_skip` directly — **note for whoever touches this next**:
  `record_entry`/`record_skip` do **not** increment `lines_read` themselves (only the real
  `ingest()` loop does, once per item regardless of `Ok`/`Err`); the tests call small local
  `record_ok`/`record_bad` helpers that increment it explicitly, to avoid a silently-wrong
  ratio in the fixture itself. Counter-check: removed the `if ratio > MAX… { return Err… }`
  block entirely, reran `cargo test require_nonempty_ratio_tests` — exactly the 90%-genuine
  case reds (the filtered-expected and under-threshold cases stay green, as they must), then
  restored. Also reran the full `tools/dict-build` suite (`cargo test --locked`, 111 lib + 34
  integration/schema tests) after landing the change — the existing small fixtures (`tests/
  fixtures/raw/**`, `build_layer` fixtures used elsewhere) do not trip the new gate.
- **Red `check:i18n`** — the two Vietnamese `assert!` messages Phase 2 flagged at
  `src-tauri/src/core/dict/query.rs:573-574` (inside `#[cfg(test)] mod short_query_tests`,
  added by Phase 1 for L434) are now English. Chose translate-over-exempt: the existing
  `tests/**` exemption's own stated reason ("no reader needs Vietnamese here, only someone
  fixing the test") applies identically to an inline `#[cfg(test)]` module under `src/**`, and
  translating removes the actual violation instead of growing `EXEMPT` — `npm run check:i18n`
  is green with the exemption list unchanged (still 93 files, same three patterns).
- **Five missing counter-checks (Phase 1's list)** — all five reproduced red for the right
  reason, then restored (verified via `git diff --stat` showing zero net change to `layer.rs`/
  `mod.rs` after each):
  - `a_missing_meta_row_names_which_key_is_absent` — removed the `schema_version` `let Some
    (…) = … else { return Err(MetaRowMissing) }` and separately the `layer` one (two runs);
    each reds on its own (wrong `SkipReason` / wrong skipped count).
  - `a_layer_whose_dict_source_table_is_gone_is_sources_unreadable` — changed the `dict_source`
    read's `.map_err(SourcesUnreadable)?` to `.unwrap_or_default()`; reds (0 skipped instead
    of 1).
  - `a_layer_whose_entry_table_is_gone_fails_lookup_without_taking_down_the_rest` — removed the
    `LookupFailed` push in `mod.rs::lookup_grouped`'s `Err` arm (kept the `continue`, so the
    "other layers still answer" half stays proven); reds (0 `LookupFailed` entries instead
    of 1).
  - `a_permission_denied_scan_is_an_empty_layer_set_not_a_panic` — changed the non-`NotFound`
    `read_dir` error arm in `DictLayers::open` from silent-empty to `panic!`; reds with the
    exact panic message as the failure (this is the case Phase 1 called "closer to already
    real" — it wasn't: there was no seam that could actually panic before this counter-check
    proved one could).
  - `deleting_the_layer_holding_a_source_code_frees_it_for_the_loser_on_reopen` — Phase 1 was
    right that no persisted state exists today, so I added a temporary `static … Mutex<HashSet
    <String>>` ("sticky memory") that `conflict_with` also consults, populated from every
    accepted layer's source codes at the end of `DictLayers::open()`. This simulates the
    regression the test guards against (a conflict lock that survives past the file that
    caused it); reds on exactly the assertion this test exists for. Reverted in full — `git
    diff --stat` on `layer.rs` shows zero lines changed after restore.
- **L1042 ④ — measured, not built.** Wrote `e2e/specs/attribution-tab-order.e2e.mjs`
  (Tab from the first `.source-chip`, through the whole chip strip, expecting to land on
  `[data-attribution-open]`, then into the opened `.attr-panel`). Ran it for real on WKWebView
  via `npm run test:e2e -- --spec …` three times while debugging:
  1. First run: chip loop passed, but the final Tab landed back on a `.source-chip`, not the
     opener.
  2. Suspected a chip-count race (`$$(CHIP)` read before Vue finished rendering all N chips) —
     added a `browser.waitUntil` on chip count matching the IPC source count. Same failure,
     same line.
  3. Added a full trace (`console.log` after every single `Tab`, up to `chipCount + 2`
     presses). **Real finding**: focus never moved at all — 12 consecutive `Tab` presses all
     reported the exact same active element (`cc-cedict`, the first chip). This directly
     contradicts Task 0's stated assumption ("Tab focus movement is not the broken primitive,
     only `<button>` keyboard *activation* is") — measured here, on this driver, Tab does not
     move focus between `<button>` elements at all (plausibly the WebKit "Full Keyboard
     Access" system setting most macOS machines/CI runners have off by default, which gates
     whether buttons join the Tab sequence — not confirmed further, out of scope to chase).
  Per the spec's own escape valve ("if the driver cannot move focus by Tab, ④ goes to `Chủ:
  Epic 11`"), **removed** the spec file rather than leave a permanently-red/always-skipped
  test in `e2e/specs/`. `deferred-work.md`'s L1042 disposition records the measured finding
  (not a vague "couldn't do it").
- **Story 2.9 🔵 pointer** — `_bmad-output/implementation-artifacts/2-9-gop-bang-backspace-dau-o.md`,
  right after AC9's Given/When/Then block: a dated correction noting phiếu quyết #89 reversed
  the "chính xác từng chữ ở `parallel`" clause. Original AC9 text kept in place (not deleted),
  per "a claim that stops being true is fixed in place with 🔵 and a date."
- **`deferred-work.md`'s 36 `→` dispositions** — one per item named in the spec's Dispositions
  section (cross-checked line-for-line against `11-3-task0-2026-09-26.md`'s population). Wrote
  them with a small Python script (exact block-boundary detection: a target line's enclosing
  `- ` bullet through to the line before the next `- `/`## `), inserted bottom-up to keep line
  numbers stable, then a second pass fixed 23/36 that had landed after a pre-existing trailing
  blank line (would have read as an orphaned paragraph) by swapping them above that blank line.
  `npm run check:debt-owner` is green; independently confirmed (by walking each of the 36
  blocks' `**Chủ: …**` mentions in document order) that none of the 36 items' final owner is
  still "Story 11.3".
- **Full suite run** (triggered per AGENTS.md — `ipc_contract` and shared frontend types
  changed in Phases 1/2): all 12 `pre-push` gates green; `npm run test` (vitest) 104 files /
  1503 tests green; `npm run build` green; `cd src-tauri && cargo test --locked` — see below
  (re-run without a pipe: exit 0, 68/68 test binaries ok, 1833 passed, 0 failed); `cd tools/dict-build && cargo test --locked`
  — 111 lib + 34 integration/schema tests green, 2 pre-existing `#[ignore]`d (need real raw
  data not present here).
- **CI / nightly e2e read** (per AGENTS.md, before `done`): latest push CI (`36125438295`,
  green, both `check (macos-26)` and `check (windows-2025)`). Latest nightly e2e
  (`36189249922`, schedule, 2026-09-25T21:01:59Z, **before** this session): 24/25 spec files
  green; `editor-typing-flush.e2e.mjs` red on `expect(received).toMatch(/^Đã lưu \d+ giây
  trước$/)` receiving `"Đã lưu 0 giây trướcTra cứu"` — the Lookup button's text bleeding into
  the same status region text node, a footer-layout timing race, **not** touched by Story
  11.3 and predating this session. `attribution-focus.e2e.mjs` (the one Story-11.3-adjacent
  spec in that run) passed.
