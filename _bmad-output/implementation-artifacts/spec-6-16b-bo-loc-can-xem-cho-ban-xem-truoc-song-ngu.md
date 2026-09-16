---
title: 'Story 6.16b: The "needs review" filter for the bilingual import preview'
type: 'feature' # feature | bugfix | refactor | chore
created: '2026-09-16'
status: 'done' # draft | ready-for-dev | in-progress | in-review | done
route: 'dispatch' # oneshot | dispatch
baseline_commit: '47d4a49878c8f34302d2641eca1c6c2525f29596'
review_loop_iteration: 0
context:
  - '{project-root}/AGENTS.md'
  - '{project-root}/src-tauri/AGENTS.md'
---

<frozen-after-approval reason="human-owned intent — do not modify unless human renegotiates">

## Intent

**Problem:** FR132's filter exists so nobody confirms a fifty-Chapter import blind, but Story 6.10
built it for the monolingual path only and no spec on either side asked whether it applied to the
bilingual one (retro Epic 6, finding F4). The bilingual preview shows three aggregate numbers —
rows, Chapters, pairs (`BilingualImportPreviewOverlay.vue:410-418`) — and has **no per-Chapter data
at all**: `BilingualImportEncodingPreview` has six fields, none a Chapter
(`commands/project/mod.rs:4163-4179`). Two of the three signals are already computed and then
dropped on the floor: per-cell cleanup keeps only `cleaned.text` (`pipeline.rs:929`, discarding
`matches`/`per_rule_counts`) and per-cell normalize keeps only `.text` (`:983`, `:986`, discarding
`joined_lines`), so every bilingual `ImportedChapter` is assembled with `cleanup_report: None` and
`joined_line_count: None` (`:1043`, `:1046`). The screen also never mentions the encoding
confidence it already carries on the wire (`mod.rs:4165`; UI mentions it 0 times).

**Approach:** Reuse, do not rebuild. `core/segment/review.rs::classify` is already pure, already
models *not measured* as `Option`, and already owns the single `1.5` fence constant; the
monolingual wire builder `build_chapter_split_preview_wire` already turns Chapters into a
`ChapterSplitPreviewWire`. So: stop discarding the two counts on the bilingual branch and aggregate
them per Chapter group; hang the **existing** `ChapterSplitPreviewWire` off
`BilingualEncodingCandidateWire` exactly the way `EncodingCandidateWire::chapters` already does; and
mirror the monolingual overlay's chip bar, Chapter list, cause badges and `⌥W` combinator into the
bilingual module. Rust classifies, TypeScript renders (AD-1).

### Decisions (Ice, 2026-09-16)

1. **One story, not a tier split.** Measured at 5,364 tokens (4,782 before §Implementation Notes)
   when the decision was taken, 5,544 / 4,962 once these decisions and Decision 2's two matrix rows
   were written in — against an epic baseline of 2,874–7,929 at approval time, smaller than both
   6-6b (5,527) and 6-7b (7,929), the two stories from the same correct-course batch. The pipeline,
   the wire and the screen ship together.
2. **The cleanup and joined-line signals are summed over BOTH columns** — one number per Chapter,
   because a file someone else translated is exactly where the *target* column is the suspect half.
   The screen names the Chapter and the cause, not which side. `length` stays the source column
   alone, as `pipeline.rs:1034-1039` already builds it.
3. **A row with unequal sentence counts is NOT a needs-review cause.** Story 6.17's mismatches
   already have their own workbench and already refuse confirm in Rust, so they cannot slip past;
   a second label guarding the same thing would buy nothing and would force an edit to `review.rs`.
4. **The screen mirrors the monolingual mockup** (`mockups/web-import.html:242-246`) — chip bar with
   the two counts and the `⌥W` hint. No UX pass blocks this story; the cause and filter strings are
   already import-path-neutral and are reused verbatim.

## Boundaries & Constraints

**Always:**
- 🔴 **Both column cells feed the cleanup and joined-line sums** (Decision 2). The cleanup loop at
  `pipeline.rs:923` already iterates both columns, so summing both needs **no loop split**; normalize
  already produces two separate `Normalized` values (`:983`, `:986`) whose `joined_lines` are simply
  added. `length` is not changed.
- 🔴 **`core/segment/review.rs` is not edited — 0 lines.** Its four `ReviewCause` variants, its
  `MIN_MEASURED_VALUES = 4` floor, its `IQR <= 0` exclusion and its single `TUKEY_MULTIPLIER = 1.5`
  are reused as they stand. Needing to touch it means the design drifted; STOP and say so.
- 🔴 **`ShortLength` is reused unchanged, and the length signal does apply.** The AC sentence
  *"bóc ra ngắn bất thường không áp"* is about the **extraction framing**, not about disabling the
  length fence — the same AC lists *độ dài* among the three signals that do apply, and the existing
  label is already provenance-neutral: `mode.library.preview.review_cause_short_length` =
  *"Ngắn bất thường"* (`vi.json:338`). **No new cause variant, no new cause i18n key.**
- 🔴 **"Broken link" is not a classifier cause and must not become one.** `classify` forbids feeding
  broken items in (`review.rs:189-191`); `broken_item_count` is a wire-layer integer added into
  `needs_review_count` (`mod.rs:2631`). This path has no links: pass **`0`**, and add no field.
- 🔴 **Confirm writes the same bytes.** This story changes what the preview *returns and renders*,
  never what `confirm_bilingual_import` writes. With no manual action the `.atproj` result must be
  byte-identical to before.
- 🔴 **`⌥W` compares `event.code === 'KeyW'`, never `event.key`** — `⌥W` types `∑` on macOS, so
  `event.key` is never `'w'` (`ImportPreviewOverlay.vue:753`, rationale `:740-751`).
- 🔴 **Do not relax `onMismatchKeydown`.** It rejects *every* Alt chord at
  `BilingualImportPreviewOverlay.vue:159`, and that guard is load-bearing for its seven keys. Add a
  combining scrim function in the shape of `ImportPreviewOverlay.vue:779-783`, each handler keeping
  its own predicate, bound as the single bare `@keydown` (Vue allows only one).
- **Every new module cell in `bilingualImportPreviewState.ts` gets its own `const`** and must be
  assigned inside `resetBilingualImportPreview` (`:515-535`, today 19 cells).
  `scripts/check-panel-refs.mjs` recognizes only `x = …` / `x.value = …` / `x.clear()`, and
  `const a = ref(0), b = ref(0)` is a hard FAIL.
- **Adapters never throw**, keep the three-state result shape, and check every new `Option` field as
  `x === null || <predicate>` — `undefined` is **not** valid (pattern at `config/project.ts:447-448`,
  with the reason written in place at `:452-455`).
- **Cause labels stay four separate literal keys through a shallow `switch`** (`reviewCauseMessageKey`,
  `ImportPreviewOverlay.vue:366-377`) — no interpolated keys, so `check:i18n` sees literals.
- **New commands register with `keys: undefined` plus a local DOM handler**, like all ten existing
  `command.import.preview.bilingual_*` (`commands/index.ts:1248-1380`).

**Never:**
- Never recompute the two counts, the verdicts or the two totals in TypeScript. Rust sums them,
  including on this path (AD-1).
- Never add a threshold constant beyond `1.5`. Needing one ⇒ **STOP and report**. *(The four-value
  floor is not a tunable: it is the arithmetic condition for Q1 and Q3 to fall in different halves.)*
- Never widen `ImportPreviewOverlay.vue` / `importPreviewState.ts` to serve the bilingual path.
  Story 6.16 deliberately built the bilingual module as a parallel one-tier module and recorded why
  (§Spec Change Log of `spec-6-16`); this story keeps that shape.
- Never change the `.atproj` schema, the confirm wire's behaviour, or `PIPELINE_ORDER`.

## I/O & Edge-Case Matrix

| Scenario | Input / State | Expected Output / Behavior | Error Handling |
|----------|--------------|---------------------------|----------------|
| Ten bilingual Chapters, all three signals measurable | rows grouped by pattern | chip bar shows `N cần xem` / `M sạch`; each flagged Chapter names its causes | N/A |
| One Chapter unusually short | length below `Q1 − 1.5×IQR` | that Chapter is needs-review with `short_length` | N/A |
| Cleanup count `null` for one Chapter, fence live | `cleanup_match_count: None` | that Chapter is needs-review with `not_measured`, **never** clean | N/A |
| Chapter missing both optional signals | both `None`, both fences live | `not_measured` appears exactly **once** | N/A |
| Cleanup rules hit **only the target column** | source cells untouched, target cells heavily cut | the Chapter is still flagged `high_cleanup_matches` -- Decision 2 | N/A |
| Normalize joins lines **only in the target column** | source rows already one line each | the Chapter is still flagged `high_joined_lines` -- Decision 2 | N/A |
| Fewer than four Chapters | 3 Chapters | no signal participates; `any_signal_participated: false`; the fallback line renders; `⌥W` refused | N/A |
| One signal degenerate | that signal's `IQR == 0` | only that signal is excluded; the other two still classify | N/A |
| `⌥W` pressed | `{ code: 'KeyW', key: '∑', altKey: true }` | list collapses to the needs-review group | N/A |
| `⌥W` with nothing to review | `needs_review_count == 0` | toggle refused, list unchanged | N/A |
| Encoding guessed with low confidence | `confidence === 'low'` | import-level banner renders **outside** the two counts | N/A |
| Encoding candidate switched | another candidate selected | chips and list re-read that candidate's own wire | N/A |
| Confirm with no manual action | any bilingual file | `.atproj` bytes identical to pre-story output | N/A |
| Table parse fails for a candidate | too few columns | no Chapter list for it; existing refusal path unchanged | `err.import.bilingual_*` |

</frozen-after-approval>

## Code Map

**Classifier — reuse, do not edit**
- `src-tauri/src/core/segment/review.rs:193` `classify(&[ChapterMetrics]) -> ReviewOutcome`;
  `:116-121` `ChapterMetrics { length: usize, cleanup_match_count: Option<usize>,
  joined_line_count: Option<usize> }`; `:57-69` four `ReviewCause` variants; `:125`
  `MIN_MEASURED_VALUES`; `:128` `TUKEY_MULTIPLIER`; `:161-185` `tukey_fence`. Pure, no imports
  outside `#[cfg(test)]`.

**Monolingual wire builder — reuse**
- `src-tauri/src/commands/project/mod.rs:2591-2636` `build_chapter_split_preview_wire(chapters,
  broken_item_count, origin_overrides)`; metrics built `:2596-2603`, `classify` called `:2604`,
  totals `:2627-2635`. `:2448-2498` `ChapterSplitPreviewEntryWire`; `:2555-2584`
  `ChapterSplitPreviewWire`; `:2423-2442` `ReviewCauseWire`. Precedent to copy:
  `EncodingCandidateWire::chapters: Option<ChapterSplitPreviewWire>` (`:2307`).

**Pipeline — where the two signals are discarded**
- `src-tauri/src/core/segment/pipeline.rs:920-936` per-cell cleanup, discard at `:929`
  (`*cell = cleaned.text;`); `:983`/`:986` per-cell normalize, `joined_lines` dropped — note the two
  columns are **already separate statements with different languages** there, so a per-column split
  is free; `:923` is one loop over both columns, so a per-column cleanup split means splitting that
  loop. Bilingual `ImportedChapter` assembly `:1025-1055`, the two `None`s at `:1043`/`:1046`,
  `source_text` = source column joined `:1034-1039`.
- Monolingual contrast — accumulators `Flow::cleanup_reports` (`:551`, pushed `:901-904`) and
  `Flow::joined_line_counts` (`:596`, pushed `:961`), zipped in at `:1075`/`:1078`.
- Grouping: `split_bilingual_chapters` `:1579-1623`, `BilingualChapterGroup { title, rows, segments }`
  `:634-641`, called `:1354`. Aggregation row→group is a plain sum for both signals.

**Bilingual wire + commands**
- `src-tauri/src/commands/project/mod.rs:4163-4179` `BilingualImportEncodingPreview` (six fields,
  `confidence` at `:4165`); `:4144-4160` `BilingualEncodingCandidateWire` (`chapter_count` `:4150`);
  `:4202-4215` `preview_bilingual_import`, per-candidate run `:4252-4262`, and `:4275-4298` where
  `PipelineOutput.chapters` is consumed for `len()` only (`:4289`) — every other per-Chapter field
  is dropped there. That is the seam to widen.
- `src-tauri/src/commands/project/wire.rs:979-988` / `:1039-1051` / `:1099-1114` the three Tauri
  commands; registered `lib.rs:727-730`.

**Frontend — mirror source**
- `src/ImportPreviewOverlay.vue:1465-1545` chip bar (needs-review chip `:1481-1494`, clean chip
  `:1495-1501`, `⌥W` hint `:1505`, `any_signal_participated` fallback `:1510-1512`, low-confidence
  banner `:1524-1528`); `:1588-1596` per-Chapter badge + cause chips; `:366-377`
  `reviewCauseMessageKey`; `:752-772` `onChapterFilterKeydown` (`event.code` guard `:753`);
  `:779-783` `onScrimKeydown` combinator; `:835` the single bare `@keydown`.
- `src/importPreviewState.ts:1350-1377` `toggleImportPreviewChapterFilter` (ON refused when
  `needs_review_count === 0 || !any_signal_participated`, `:1357`); `:2002-2058` `resetImportPreview`.
- `src/commands/index.ts:1479-1489` the filter command with `keys: undefined`.

**Frontend — edit target**
- `src/BilingualImportPreviewOverlay.vue` (842 lines): counts line `:410-418`, scrim `:266-272`,
  single bare `@keydown` `:271`, `onMismatchKeydown` `:158-205` with the Alt rejection at `:159`.
- `src/bilingualImportPreviewState.ts` (536 lines): 19 cells `:46-84`, computeds `:105-185`,
  `resetBilingualImportPreview` `:515-535`.
- `src/config/project.ts:765` bilingual `confidence` field; `:824-836`
  `isBilingualImportEncodingPreview`; `:804-817` `isBilingualEncodingCandidateWire`; `:841-877`
  `previewBilingualImportFromFile`. Monolingual guards to copy: `:412` `isReviewCauseWire`,
  `:440-458` entry guard, `:486-497` wire guard, `:500-515` the `null`-arm carrier pattern.
- `src/i18n/vi.json`: reuse `:337-341` (cause labels + badge) and `:331-336` (filter bar) — these are
  **path-neutral strings already written**; bilingual namespace is `:360-376`.

**Tests that move**
- `src-tauri/tests/bilingual_import_contract.rs:574` exhaustive key-set assert at three levels —
  **adding any wire field turns it RED**; that is the intended signal, update it deliberately.
- `src-tauri/tests/review_contract.rs` (5 cases) and `review.rs` `mod tests` (7 cases) — the
  classifier's own guards, expected to stay green untouched.
- `tests/frontend/importPreviewOverlayRender.test.ts:427-490` — the four `⌥W` DOM cases and the
  exact event shape to copy: `{ code: 'KeyW', key: '∑', altKey: true }`.
- ⚠️ `src/config/project.ts:824` `isBilingualImportEncodingPreview` has **no test today** —
  `tests/frontend/importPreviewBilingual.test.ts:26` mocks the whole adapter module away. The
  monolingual twin `tests/frontend/importPreviewEncodingWireShape.test.ts` has 24 unmocked cases
  including missing-field and camelCase rejection. New `Option` fields land in an unguarded checker
  unless this gap is closed in this story.

## Tasks & Acceptance

**Execution:**
- [x] `src-tauri/src/core/segment/pipeline.rs` -- keep the per-cell cleanup counts and
      `joined_lines` instead of discarding them at `:929`/`:983`/`:986`; sum row→group; fill
      `cleanup_report`/`joined_line_count` at `:1043`/`:1046` -- the two signals exist already.
- [x] `src-tauri/src/commands/project/mod.rs` -- call the existing
      `build_chapter_split_preview_wire(chapters, 0, …)` on the bilingual candidate run
      (`:4275-4298`) and carry a `ChapterSplitPreviewWire` on `BilingualEncodingCandidateWire`,
      mirroring `EncodingCandidateWire::chapters` -- reuse, no new classifier.
- [x] `src-tauri/tests/bilingual_import_contract.rs` -- update the three-level key-set assert
      (`:574`) and add one case per I/O Matrix row on the product path.
- [x] `src/config/project.ts` -- types plus runtime guards for the new wire shape, `Option` fields
      checked with the explicit `null` arm.
- [x] `tests/frontend/importPreviewEncodingWireShape.test.ts` (or a bilingual sibling) -- cover
      `isBilingualImportEncodingPreview` **unmocked**, including a missing-field and an `undefined`
      case -- closes the gap named in §Code Map.
- [x] `src/bilingualImportPreviewState.ts` -- filter + cursor cells, one declarator each, all
      assigned in `resetBilingualImportPreview`; toggle refused when nothing needs review.
- [x] `src/BilingualImportPreviewOverlay.vue` -- chip bar, Chapter list with badge and cause chips,
      low-confidence banner, and a combining scrim keydown adding the `⌥W` handler beside
      `onMismatchKeydown` without relaxing its Alt guard.
- [x] `src/commands/index.ts` -- register the bilingual filter-toggle command, `keys: undefined`.
- [x] `src/i18n/vi.json` -- only the keys that do not exist yet; reuse `:331-341` verbatim.
- [x] `tests/frontend/importPreviewBilingual.test.ts` -- mount cases for the chips, the cause chips
      and `⌥W`, using the `{ code: 'KeyW', key: '∑' }` event shape.
- [x] `_bmad-output/implementation-artifacts/deferred-work.md` -- close or re-own whatever this
      story settles; every open item keeps a real `Chủ:`.

**Acceptance Criteria:**
- 🔴 Given the row→group aggregation removed so `cleanup_report` returns to `None`, when the new
  suite runs, then a `not_measured` case is RED.
- 🔴 Given `event.code` swapped to `event.key` in the new `⌥W` handler, when the frontend suite
  runs, then the `⌥W` case is RED *(the control is genuine: the test supplies `key: '∑'`, and
  happy-dom does not backfill `key` from `code` — verified on the monolingual twin, 19/19 green)*.
- 🔴 Given the `null` arm deleted from one new `Option` guard in `config/project.ts`, when the wire-
  shape suite runs, then it is RED.
- 🔴 Given `broken_item_count` passed as anything but `0`, when the bilingual suite runs, then a
  count case is RED.
- 🔴 Given the **target** column dropped from the two sums (Decision 2 reduced to source-only), when
  the bilingual suite runs, then the two target-column-only cases of the I/O Matrix are RED. *(This
  is the control that makes Decision 2 a measured rule rather than a sentence — without it, the
  cheaper source-only implementation stays green.)*
- Given a bilingual import confirmed with no manual action, when the written `.atproj` is compared
  against the pre-story output for the same input, then the bytes are identical.
- Given the eleven `check:*` gates + `npm run test` + `npm run build` + `cargo test --locked` in
  pre-push order, when run, then 0 findings and 0 new red cases; `check:debt-owner` 0 orphans.

## Implementation Notes

**Row→group aggregation design.** `Flow::bilingual_row_cleanup_reports`/`bilingual_row_joined_line_counts`
(`pipeline.rs`) hold one measurement per row of `bilingual_rows`, filled at `Step::CleanByRules`/
`Step::NormalizeParagraphsAndWhitespace`. A row's value is `None` when it is missing **either**
of the two chosen columns (not just both) — the sum "over both columns" (Decision 2) cannot
honestly claim completeness when half of it never existed for that row. `split_bilingual_chapters`
folds these per-row values into one per-Chapter value via `merge_optional`/`aggregate_row_signal`:
`None` propagates (any unmeasured row poisons the whole Chapter's total), matching the classifier's
own "not measured, never clean" contract. This makes the `not_measured` scenario reachable on the
real product path via a genuinely ragged table (one Chapter's rows missing the target column
entirely) rather than only via `core/segment/review.rs`'s pure unit tests.

**Red controls — measured 2026-09-16, applied then reverted verbatim, suite confirmed green after each:**
1. `cleanup_report: group.cleanup_report` → hardcoded `None` (pipeline.rs, ImportedChapter assembly):
   `cargo test --test bilingual_import_contract` → 2 RED (`a_chapter_missing_the_target_column_entirely_is_not_measured_never_clean`,
   `cleanup_matches_only_in_the_target_column_still_flags_the_chapter`), 38 green.
2. `event.code !== 'KeyW'` → `event.key !== 'w'` (BilingualImportPreviewOverlay.vue,
   `onBilingualChapterFilterKeydown`): `npx vitest run tests/frontend/importPreviewBilingual.test.ts`
   → 2 RED (the two `⌥W` dispatch cases), 31 green.
3. `(v.chapters === null || isChapterSplitPreviewWire(v.chapters))` → `isChapterSplitPreviewWire(v.chapters)`
   (config/project.ts, `isBilingualEncodingCandidateWire`): `npx vitest run tests/frontend/importPreviewBilingualWireShape.test.ts`
   → 1 RED (the valid-payload case, since its first candidate has `chapters: null`), 6 green.
4. `build_chapter_split_preview_wire(&o.chapters, 0, &[])` → `…, 1, &[]` (mod.rs,
   `preview_bilingual_import`): `cargo test --test bilingual_import_contract` → 5 RED
   (`a_chapter_missing_the_target_column_entirely_is_not_measured_never_clean`,
   `broken_item_count_is_always_zero_on_the_bilingual_path`,
   `cleanup_matches_only_in_the_target_column_still_flags_the_chapter`,
   `joined_lines_only_in_the_target_column_still_flags_the_chapter`,
   `ten_bilingual_chapters_show_chip_counts_and_flag_the_short_one`), 35 green.
5. Target column dropped from the two sums — two separate mutations, run separately:
   (a) `for &col in &[bilingual_source_column, bilingual_target_column]` → `&[bilingual_source_column]`
   (CleanByRules bilingual branch): 3 RED (`a_chapter_missing_the_target_column_entirely_is_not_measured_never_clean`,
   `an_enabled_cleanup_rule_runs_on_both_cells_in_preview_and_on_disk` — a pre-existing Story 6.16
   case, `cleanup_matches_only_in_the_target_column_still_flags_the_chapter`), 37 green.
   (b) target-column block wrapped in `if false` (NormalizeParagraphsAndWhitespace bilingual
   branch): 1 RED (`joined_lines_only_in_the_target_column_still_flags_the_chapter`), 39 green.

**Full verification, 2026-09-16 (after every red control reverted):** all eleven `check:*` gates
(`deps`/`tokens`/`i18n`/`commands`/`layout`/`panel-refs`/`dict`/`dict-manifest`/`lint`/`gates`/
`debt-owner`) exit 0 · `npm run test` 1079/1079 passed (78 files) · `npm run build`
(`vue-tsc --noEmit` × 2 + `vite build`) exit 0 · `cargo test --locked` 58/58 binaries green, 0
failures. `git diff -- src-tauri/tauri.conf.json src-tauri/capabilities` empty (neither file
touched).

**⚠️ Environment note.** The Homebrew-linked `node`/`npm` on this machine are broken independent
of this story (`dyld: Library not loaded: …/libsimdutf.35.dylib`, referenced from
`libmerve.1.2.2.dylib` — a stale transitive link, reproduces on bare `node --version`). Frontend
verification ran on an `fnm`-managed Node v22.22.2 instead (`eval "$(fnm env)" && fnm use v22.22.2`),
which is unaffected. Worth fixing (`brew reinstall node` or unlinking `merve`) so `npm`/`pre-push`
work again without the `fnm` workaround — filed as a debt item below, owner Ice.

**Matrix Test Audit self-check.** I/O Matrix rows "One signal degenerate" and the two
"low confidence"/"candidate switched" rows are not re-proven by a new bilingual-specific test:
the first is `classify()`'s own unedited contract, already covered by
`core/segment/review.rs::tests::a_degenerate_iqr_excludes_only_that_signal` (reused verbatim, ran
green in this session's `cargo test --lib core::segment`); the confidence/candidate rows are
existing `BilingualImportEncodingPreview.confidence`/multi-candidate behavior this story does not
touch, covered by the new `mount` cases (`confidence "low" hiện cờ tin cậy thấp…`) and the
pre-existing candidate-switch coverage in `importPreviewBilingual.test.ts`.

---

**Coordinator's independent verification, 2026-09-16.** Judged against the diff since
`baseline_commit`, not against the implementation report.

- **Matrix row "Encoding candidate switched" was NOT covered — the self-check above is wrong on
  this point, and it was measured, not argued.** Mutating
  `bilingualImportPreviewSelectedCandidate` (`bilingualImportPreviewState.ts:122`) from
  `.find((c) => c.encoding === selectedEncoding.value)` to `candidates[0]` — i.e. the screen reads
  the wrong candidate entirely — left **all 33** cases of `importPreviewBilingual.test.ts` green,
  the "pre-existing candidate-switch coverage" among them. Those cases pin column roles, header
  and encoding *identity*, none of them a per-candidate `chapters` block. Added
  `đổi ứng viên bảng mã làm chip và danh sách Chương đọc lại khối của CHÍNH ứng viên đó` (two
  candidates carrying different `chapters` wires); under the same mutation it is the **only**
  RED (1 failed / 33 passed), and green after restore. Restores were verified byte-identical
  against a scratchpad copy, not against `HEAD` — `HEAD` does not contain this story's work.
- **Red control #5(b) reproduced independently.** Deleting the single line
  `total += normalized.joined_lines;` from the *target* branch of the bilingual normalize step
  (a removal, not an inserted flag) → exactly `joined_lines_only_in_the_target_column_still_flags_the_chapter`
  RED, 39 green; restored byte-identical, 40/40 green. Decision 2 is a measured rule, not a sentence.
- **Matrix row "One signal degenerate" is better covered than the self-check claims.** It is
  proven on the real product path, not only in `review.rs`'s pure unit tests:
  `cleanup_matches_only_in_the_target_column_still_flags_the_chapter` asserts
  `!last.review_causes.contains(&ShortLength)` precisely because `length` is constant across the
  ten Chapters (IQR = 0) while the other two signals still classify.
- **Independent full run** (Node v22.22.2 via `fnm`, the Homebrew `node` breakage above confirmed
  first-hand on bare `node -v`): eleven `check:*` gates all exit 0 · `check-debt-owner --report`
  0 open items without an owner · `npm run test` **1080/1080** (78 files — 1079 plus the case added
  here) · `npm run build` exit 0 · `cargo test --locked` **1558 passed, 0 failed**.
  `git diff` empty for `src-tauri/tauri.conf.json`, `src-tauri/capabilities`, **and
  `src-tauri/src/core/segment/review.rs`** — the spec's hardest rule (0 lines) holds.
- **Count discrepancy, not a defect.** The binary figure differs by counting method: 56 (baseline
  investigation), 58 (implementation report), 59 (`test result:` lines here, doctests included).
  Recorded because "58/58" was a self-reported number rather than a measured one; the load-bearing
  figure is **0 failures**.
- **Not re-derived independently:** red controls #1–#4 and #5(a) are taken as reported. #5(b) was
  reproduced and #5(b)'s sibling defect class is what the new matrix test now guards.

## Spec Change Log

- 2026-09-16 (implementation) — `deferred-work.md`: two open items under "Deferred from:
  6-7-nhap-tu-tren-tac-pham-co-san" and "Deferred from: 6-7b-them-chuong-vao-tac-pham-co-san" had
  named `Chủ: Story 6.16b`, on the premise that this story would add an "append to an existing
  Work" destination for the bilingual import screen. This story's frozen Intent is the "needs
  review" filter (FR132) instead — re-owned in place (🔵 SỬA 2026-09-16) rather than silently
  closed; the "append destination for bilingual import" capability remains open, owner Ice, needing
  a new planning slot (`correct-course`) before a story can claim it.

## Review Triage Log

### Review pass 1 — 2026-09-16 (blind-hunter · edge-case-hunter · verification-gap)

Diff 114 kB (spec excluded, so only the edge-case layer saw it). Blind-hunter floor 10, filed 16;
edge-case 10; verification-gap 2 + 1 other. No `intent_gap`, no `bad_spec` — no loopback.

| # | Finding (layer) | Verdict | Evidence | Route |
|---|---|---|---|---|
| VG-1 + BH-8 + BH-9 + EC-8 | The real filter toggle and the filter branch it drives are never executed by any test (verification-gap, blind, edge-case) | medium | Pre-verified. All five `⌥W` cases inject `toggleBilingualImportPreviewChapterFilter: toggleMock` and assert call counts only; the four render cases never turn the filter on. Deleting the `needs_review` filter in `bilingualChapterEntriesRendered` or emptying the toggle body leaves the suite green — the story's headline behaviour is unpinned end to end | patch |
| VG-2 | Only the `short_length` branch of the mirrored `reviewCauseMessageKey` is asserted (verification-gap) | medium | Pre-verified. The only cause-text assert is `'Ngắn bất thường'`; `high_cleanup_matches` appears in no bilingual fixture. `check:commands` Kiểm E cannot reach it — the call site is `t(reviewCauseMessageKey(cause))`, not a literal. The monolingual twin has a dedicated four-branch case. A verbatim copy is exactly the construct that drifts | patch |
| EC-9 | No case asserts `chapters == None` for a candidate whose table parse fails (edge-case) | low | Confirmed: `grep` for `chapters.is_none()`/`chapters, None` over `bilingual_import_contract.rs` returns 0 behavioural asserts; the only `chapters: None` is the struct literal in the key-set shape test. I/O Matrix row "Table parse fails for a candidate" has no owner | patch |
| BH-4 + EC-7 | `merge_cleanup_reports` doc-comment claims `matches` is emptied; the body returns `mut a` with `a.matches` intact (blind, edge-case) | low | Real, read in the diff. Holds today only because per-row reports are built with `matches: Vec::new()`. Direct correction | patch |
| EC-4 | `chapters` doc-comment states the wrong invariant — says `None` when `preview == None`, actually `None` whenever the candidate's `run_pipeline` returned `Err` (edge-case) | low | Confirmed at `mod.rs:4271-4280`: `outcome` is `None` for `Some(Err(_))` and for an unresolvable encoding, not only for an absent preview. A caller reading "preview non-null ⇒ chapters non-null" is wrong. Mirrored in `config/project.ts` | patch |
| BH-17 | `aggregate_row_signal`'s trailing `if any { acc } else { None }` is dead (blind) | low | True: after the loop `acc` is `Some` exactly when a value was consumed, so the expression always equals `acc`. Fix is a deletion | patch |
| BH-1 | `deferred-work.md` prescribes `brew reinstall node`, which cannot fix the break (blind) | low | Verified on this machine: `/usr/local/opt/simdutf/lib/` contains only `libsimdutf.36.dylib`; Cellar holds 9.1.1 and 9.2.0 but only the linked one is on `opt/`. The dangling `.35` reference lives in `libmerve`, so relinking node changes nothing | patch |
| VG-other + BH-6 + BH-7 + EC-1 + EC-2 | Filter stays ON across `refresh()`/candidate switch; switching to a candidate with `any_signal_participated === false` hides the chip bar while the filter stays active, leaving an empty list and no visible control (verification-gap, blind, edge-case) | medium | Real, but **not caused by this change** — it is a faithful mirror of the signed-off Story 6.10 screen. `ImportPreviewOverlay.vue` hides its chips under the identical `<template v-if="…any_signal_participated">`; the monolingual resets the filter only in its three *open* paths (`importPreviewState.ts:703/826/937`), never on a tier rebuild; and `importPreviewChapters.test.ts:1314` asserts on purpose that the filter **survives** a candidate switch. The hole sits in both screens | defer |
| BH-2 + BH-3 | `split_bilingual_chapters` zips three vectors (silent truncation) and indexes `bilingual_row_*[i]` directly (panic) (blind) | low | Real code shape; the invariant holds today because all three are sized together at `Step::DecodeEncoding` and only cell contents change afterwards. Not demonstrated reachable, but the failure mode — tail rows vanishing with no error — is this epic's cardinal sin, so it is recorded rather than dismissed | defer |
| BH-12 + BH-13 + EC-5 | No elision in the bilingual Chapter list, and the per-candidate Chapter payload now ships for all five candidates, cost unmeasured (blind, edge-case) | low | Real divergence: the monolingual collapses >6 entries to first-3/`⋯`/last-3. Not caused here in the sense that the monolingual's own at-scale render is itself an open Story 6.18 debt probe (`importPreviewChapters.test.ts`, 1.000-Chapter case, explicitly "bằng chứng, không phải verdict") | defer |
| BH-10 + BH-11 | No `aria-pressed` on the filter chip; the bilingual list keeps a cursor highlight without the monolingual's `role="listbox"`/`aria-activedescendant` contract (blind) | low | `aria-pressed` count is **0 in both overlays** — mirrored, not newly dropped. The cursor-without-contract half is a real divergence, but it is tied to `⌥←`/`⌥→`, which this story's AC does not carry | defer |
| EC-3 | A non-table pipeline error on the *selected* candidate vanishes silently — `is_bilingual_table_refusal` gates `selected_refusal`, so no note is shown (edge-case) | low | Real at `mod.rs:4272-4277`, and pre-existing: before this story the same branch already zeroed `chapter_count`/`pair_count` with no note. This change adds one more thing that disappears, it does not create the swallow | defer |
| EC-6 | `bilingual_source_column == bilingual_target_column` double-counts both new signals (edge-case) | low | Unreachable from the UI: `setBilingualSourceColumn:355`/`setBilingualTargetColumn:365` swap the roles instead of allowing a duplicate — Story 6.16's own EC-3 patch. The unguarded Rust entry point is pre-existing from 6.16; this story only adds a second consequence to it | defer |
| BH-14 + EC-10 | `epic-6-context.md` lost recorded constraints, and now says `docx-rs` reads `.docx` — reversing the decision that banned it (blind, edge-case) | medium | **Confirmed, and it was my own step-01, not the implementation.** The HEAD version recorded the implemented fact — a self-written OOXML reader on `zip`+`quick-xml`, chosen because `docx-rs` has hundreds of panic points — while my regenerated version restated the stale planning-time position. The cache-validity rule I followed compares mtimes only, and the HEAD file already carried the correct-course additions (6.6b/6.7b/6.16b, 6.18 moved) it was regenerated to obtain | fixed in triage — restored from `47d4a49` verbatim |
| BH-5 | `length` sums only the source column while the other two sum both (blind) | false | Not a defect: Decision 2 says so in as many words — *"`length` stays the source column alone, as `pipeline.rs:1034-1039` already builds it."* The reviewer could not see the spec by design. Implemented exactly as decided | reject |
| BH-15 | The spec every new comment cites is untracked and absent from the diff (blind) | false | The spec was deliberately excluded from `{diff_file}` so that only the edge-case layer sees it (step-04 routes it as `claims_file`). It is untracked because it is new and is committed with the story, as every prior spec in this epic was | reject |
| BH-16 | `sprint-status.yaml` says `in-progress` while the spec says `in-review` (blind) | false | Expected mid-flight: step-03 syncs the sprint file to `in-progress`, step-05 moves it on. The pair is a normal intermediate state, not a disagreement | reject |

## Design Notes

**Why a parallel render, not a shared component.** Story 6.16 split the bilingual preview into its
own overlay + state module because the prose module carries four tiers and hardcodes a five-argument
confirm; the bilingual path has one tier and an eight-argument confirm. The chip bar and Chapter list
are the first UI both screens genuinely share, so extracting a component is tempting — but the
extraction would have to straddle the two state modules that were deliberately kept apart, and it
would put Story 6.10's signed-off screen back on the table. Mirror it here; if the duplication later
earns an extraction, that is a refactor story with both screens already green.

**Why the classifier is not touched.** Everything the AC asks for already exists in `review.rs`:
three signals, the `Option` that distinguishes *not measured* from *measured and clean*, the
per-import participation flags, and one named threshold. The bilingual path is short two **inputs**,
not short an algorithm. Any design that edits `review.rs` has mistaken a plumbing gap for a logic gap.

## Verification

**Commands:**
- `npm run build && (cd src-tauri && cargo test --locked)` -- pre-push order.
- `npm run test` -- green.
- Each of the eleven `check:*` gates -- 0 violations; `node scripts/check-debt-owner.mjs --report`
  -- 0 orphans.
- Red controls -- apply each as a one-line mutation, record the failing case names and counts in
  §Implementation Notes, then revert verbatim and confirm green again.
- `git diff -- src-tauri/tauri.conf.json src-tauri/capabilities` -- empty.
- ⚠️ If an e2e run is needed, build **first** with `npm run test:e2e` (which builds
  `--features wdio`). `.githooks/pre-push:101` runs `cargo test --locked` **without** that feature
  and rebuilds the binary without the wdio plugin, so a pre-push before an e2e run breaks it in a
  way that looks like a regression in this story.

**Manual checks (if no CLI):**
- A bilingual CSV of ten Chapters with one short Chapter and one heavily cleaned Chapter: chip bar
  shows both counts, the two Chapters are flagged with named causes, `⌥W` collapses to just them,
  and pressing `⌥W` again restores the full list.
- A bilingual CSV of three Chapters: no signal participates, the fallback line says so, and `⌥W`
  does nothing.
