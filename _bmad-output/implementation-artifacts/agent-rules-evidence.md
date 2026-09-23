# Agent rules — evidence archive

Moved here verbatim on 2026-09-23 when Ice had every `AGENTS.md` condensed to rules only (and the header comment of `.githooks/pre-push` shortened). Nothing here is loaded automatically. Read the section for a rule when you need to know WHY it exists or what was measured. The current rules live in the `AGENTS.md` files; where this archive and a rule disagree, the rule wins.

---

## `AGENTS.md` as of 2026-09-23 (commit 505c8bc)

<!-- Verified 2026-09-10 against 39ae75d. Managed by bmad-project-context; edits inside this block are replaced on refresh. Keep anything you want preserved outside the markers. -->

## AuraTranslate

Fully offline dictionary-lookup and translation workspace. Tauri v2 · Rust in `src-tauri/` · Vue 3 + TypeScript in `src/` · bundled SQLite. GPL-3.0-or-later. Planning lives in `_bmad-output/planning-artifacts/`, stories and the debt ledger in `_bmad-output/implementation-artifacts/`.

## Policy

- Default branch is `master`. Writing `branches: [main]` into a workflow means CI never runs and throws no error.
- Never commit `.db` files. The `*.db` line in `.gitignore` is deliberate (AD-25) — dictionary data ships through GitHub Release + `dict-manifest.toml`.
- Before adding ANY dependency (NFR15): read the licence in the DOWNLOADED source (`~/.cargo/registry/src/…`, `node_modules/…`), never the registry label; then write it into the spine's Stack table BEFORE adding. Only GPLv3-inbound-compatible licences get in. Some crates ship no `LICENSE` file — `selectors` 0.38.0 carries its MPL-2.0 only in per-file headers, so read the headers. MPL-2.0 passes, but check for an Exhibit B notice in the headers: a file carrying one cannot be combined into GPL.
- Changing an architectural invariant is a new `AD` in the spine, not a line of code.
- Allocating a new `AD` number: scan BOTH the spine AND every `ad-brief-*.md` not yet written up — the highest number in the spine is not the next free one. AD-48 is taken twice right now (the 2026-08-17 undo brief reserved it; the 2026-08-24 dialogue round measured the spine at AD-47 and issued 48 again), and `sprint-status.yaml` item B6 is still open on that number.
- Two valid options ⇒ present both with measurements for Ice to settle; never pick one and move on.
- Dirty tree before a story starts ⇒ separate commit, first, and ask Ice before committing.
- **One agent must not implement a whole story.** An agent's bill is the AREA UNDER ITS CONTEXT CURVE, not the work it does: in the worst one measured its own output was 0,04% of the cost and the other 99,96% was re-reading what it had already said. Split a story into ~4 phases along the shape §Code Map already has (plan · Rust · Webview · Tests that move), hand each phase to a FRESH agent through a FILE on disk, and keep any one agent under ~250 k context. Modelled at **2,9× cheaper** (≈11,2 bn tokens on this repo's history) for ~3,8 handoffs per story — a model reproducing real bills to a median −18,8%, not a stopwatch, and four handoffs are four chances to drop a detail. Evidence, the model's self-check, and the two wrong values this ratio had first: `_bmad-output/implementation-artifacts/agent-token-economics.md`.
- **Front-load as little as possible: a byte read early costs ~27× the same byte read late** — 69% of carry cost is created in an agent's first quarter of turns, 3% in its last. So the "let me understand the codebase first" phase is the expensive one; do that understanding in a SHORT agent that writes its findings to disk. No saving is quoted for this lever because nobody has measured how much of that reading is compulsory versus deferrable — do not invent one.
- **A source file's line count is a token bill.** 🔵 2026-09-15 (spec-ai-6 stage 1) — `src-tauri/src/commands/project.rs` (the 6 630-line figure here was already stale; measured 6 716 at the commit stage 1 branched from) is now the directory module `src-tauri/src/commands/project/`: `mod.rs` (4 556 lines, the pure-function body — stage 2, not yet done, is what would split this further by concern), `wire.rs` (1 264 lines, the `#[tauri::command]` shells), `tests.rs` (894 lines). The Rust path `commands::project` is unchanged. The **9,4%** figure below is the measurement AS TAKEN on the pre-split file and is kept as-is — the total line count under `commands/project/` did not shrink, only its shape did, and re-measuring carry cost against the new per-file read pattern is stage 2's job. Original paragraph: alone it was 9,4% of carry cost across the heavy agents: 377 Reads, ~10 per agent. Not from naive whole-file reads — 80% of Reads already pass `offset`/`limit` — but because nobody held a 6 630-line map, so every agent re-peeked through a different small window and paid a turn each time. Splitting the file is the fix, not reading less. No gate measures file size today.

## Where things are

- Architectural invariants (AD-1…AD-48, Stack table, Consistency Conventions): `_bmad-output/planning-artifacts/architecture/architecture-AuraTranslate-2026-08-02/ARCHITECTURE-SPINE.md`
- Debt ledger: `_bmad-output/implementation-artifacts/deferred-work.md`. Story status in `sprint-status.yaml`; story content in the story file itself.
- Directory-scoped rules, attached by location: `src/AGENTS.md` · `src-tauri/AGENTS.md` · `scripts/AGENTS.md` · `tests/AGENTS.md` · `e2e/AGENTS.md` · `tools/dict-build/AGENTS.md`
- `_bmad-output/project-context.md` is a deep reference frozen at 2026-08-20 (Epic 3) — read it for history, not for current rules.
- `docs/` holds raw input for `tools/dict-build`, not agent documentation — do not read it to understand the project.

## Running and verifying

- While coding a story, run the SCOPED loop, not the full suite: `npm run test:story <id>` (add `-- --list` to see the set first). It takes the union of the working-tree diff (the primary source) and the spec's §Code Map → "Tests that move" (a net for test files the story will touch but has not yet), and reports THIẾU KHAI for anything the diff hit that the spec never named. Measured 2026-09-12, same condition (`touch src-tauri/src/lib.rs`, cache settled): full suite 140,4 s · Story 6.17's real 10 targets + 2 frontend files 37,4 s. 3,7× — not the 14× a first pass claimed by dividing the full suite by a SINGLE target. This is the dev loop, never a gate: `pre-push` keeps its full scope, because running narrow does not remove neighbour-module regressions, it defers them to push. Do not trust the spec's declaration alone — spec 6.17 named 3 test files where commit `3c25e9a` touched 10.
- ⚠️ On Ice's Mac, executing a freshly built Rust binary costs **~204 ms of pure waiting** — 100 spawns of one test binary took 20,4 s for 0,95 s of CPU, against 0,33 s for 100 spawns of `/usr/bin/true`, and re-running the SAME binary never got faster, so nothing caches it. The cause is that this toolchain (Homebrew `rustc`, not rustup) emits binaries that are `not signed at all`, and macOS re-scans every unsigned exec; `codesign -s -` on the same file drops 100 spawns to 1,13 s (**18×**). It is invisible to `cargo test`, which spawns 52 processes and shows no measurable difference signed or unsigned across 5 alternating runs — but it dominates anything that spawns per TEST. Before concluding that a test runner, a script, or a benchmark is slow on this machine, count its process spawns and multiply by 204 ms.
- `cargo-nextest` was measured and **rejected** 2026-09-12 (it is not installed; do not reach for it). Unsigned it ran the suite in 331 s against `cargo test`'s 69 s — 4,8× WORSE, because 1 494 tests are 1 494 spawns. Ad-hoc-signing the binaries first brought it to 61,8 s against 69,7 s, alternating runs on one unchanged tree: a real but **11%** win, and its faster execution (38 s vs 68 s) is eaten by ~23 s of its own build-and-list phase. Ice declined it: 11% does not buy a new dependency plus an NFR15 licence review.
- Two measurement traps, both hit while building that command. `cargo test --no-run` on a genuinely still tree costs **0,58 s**; the 21 s first measured was taken while the cache was still rebuilding after the previous session's `cargo clean` — it measured the machine, not the command. And the same command run back-to-back gave 66,6 s then 21,3 s purely because the first followed another build. Two timings are comparable only from the same cache state.
- `pre-push` runs 11 gates → vitest → build → `cargo test --locked`, on **Ice's macOS**. It says nothing about the Windows half; CI runs both platforms every push, so read the CI run before concluding green.
- The e2e suite runs NIGHTLY, not on `push` (`schedule` at 18:00 UTC + `workflow_dispatch`, macOS only) — a green push says nothing about it, and the Windows half has never run. Manual: `npm run test:e2e`. How to read a red run: `e2e/AGENTS.md`.
- Because the e2e suite runs outside the push gates, nobody reads it: it was red SEVEN nights running, 10/24 specs, starting the first night after Epic 5 code landed, while 13 stories went to `done` and the `check` job stayed green on both platforms all seven runs. Before writing `done`, read the latest `schedule` run — and if it is red, write the reason down. This is the third time the same mechanism has been recorded.
- A background watcher must grep for a string the script ACTUALLY writes, and must carry a time cap. On 2026-09-12 an `until grep -q "nghiem thu xong exit=" … do sleep 5; done` spun **4h23m** against a script whose last line was `echo "=== DONE"`; the run had finished four hours earlier and had finished GREEN, so four and a half hours bought nothing. Print the sentinel from a `trap … EXIT` so it lands even when the script dies mid-step, and cap every step — macOS ships no `timeout`, use `perl -e 'alarm shift; exec @ARGV' <secs> <cmd…>` and treat exit 142 as TIMEOUT. Before blaming another session for a stuck process, walk `ps -o ppid=` up from it: that loop resolved to the CURRENT `claude` process, and it had survived a `/clear`. A watcher that cannot fail loudly is a way to hang the session politely.
- `cargo test` red on a tree that was green minutes ago, with no code changed, is a firewall blocking test binaries that bind a local port — observed 3× on 2026-09-09, once blocking `git push` itself. 🔵 2026-09-12: the cause named here and in `deferred-work.md` — the macOS Application Firewall — is WRONG. Turning ALF off left it red; turning **LuLu** off turned it green. LuLu keys on the binary, so every freshly compiled test binary is a new stranger to it, which is also why "a new binary runs 15× slower". Measured today with LuLu `disabled = 1`: `cargo test` 58,5 s, 54 binaries green, 0 hangs, in the very project directory the ledger blamed. That directory-dependence was itself a mismeasurement — rebuilding into `/private/tmp` moved TWO variables at once, because a binary at another path is another identity to LuLu. LuLu's `allowLocalHost = 1` is the setting to keep if it is ever re-enabled. Before touching a line, re-run the SAME binary twice: 1,310 pass/14 fail then 1,325 pass/0 fail means the cause is not in the code. It changes shape by layer — in `asset_contract.rs` it surfaced as the behavioural assert `images_saved left: 0, right: 1`, not as a connection error, so grepping for "error sending request" misses it. Do not reach for `--no-verify`: that drops the other 11 gates. Owner: Ice, in `deferred-work.md`.
- `check:scope` and `check:scope:bundled` sit outside `pre-push` on purpose: they build a real Tauri window and need port 1420 free, so they fail while `npm run tauri dev` is open. CI does run them; on a dev machine run them by hand.
- Adding a gate = editing THREE lists (`package.json` · `.github/workflows/ci.yml` · `.githooks/pre-push`), and `check:gates` guards all three for every `check:*` gate. `test:e2e` is the NAMED exception, guarded in only two — see `scripts/AGENTS.md`.
- Run `npm run build` BEFORE `cargo test`: without `dist/`, `cargo test` breaks at compile time, not at an assert.
- A green Rust suite says nothing about the product's dictionary path. `tauri.conf.json` `bundle.resources` declares only `fonts/` and `license/` — NOT `dict` — while tests reach the layers through `env!("CARGO_MANIFEST_DIR")`. The packaged app and the test binary read from different places, so FR19/FR20/FR21 have never run against real data in a bundle.
- Every measurement states the build and the population it ran on. A number measured on a DEBUG build, on a `.app` with 0 dictionary layers, or on a dependency nothing calls yet is not a number about the product — 12 new crates once measured as 8,102,176 → 8,102,160 bytes purely because `lto` stripped code no production line called.
- Never mark something passed by inference. Any clause that cannot be accepted at the current layer goes into `deferred-work.md` with an owner.

## Conventions that differ from defaults

- Write clean, self-documenting code. No redundant, explanatory, or trivial inline comments. Comment only for non-obvious business logic, workarounds, and critical edge cases. Comments are written in English.
- The exception is machine-read comments, which are code and carry a MANDATORY reason clause — never strip them: `aura-allow-*` (201 in `src/`, read by `check:tokens` and `check:i18n`, whose `/aura-allow-text\s*:\s*\S/` stays red on an empty reason), `dict-build:allow <token> — <reason>` (`check:dict`, em dash required), and `eslint-disable` (24 in `src/`, policed by `reportUnusedDisableDirectives: 'error'`).
- A claim that stops being true gets FIXED IN PLACE with 🔵 and a date; don't delete it, and don't let it quietly lie.
- Markers: 🔴 unbreakable rule · ⚠️ trap or limitation · ✅ closed · 🟡 half-closed · 🔵 update, an old claim expired · ⇒ conclusion. Emoji `U+26D4` is banned repo-wide — write the negation out as a word. Don't mint a new marker for a new convention; write it out.
- "Origin" names FOUR disjoint things (spine §Consistency Conventions): translation (FR117, AD-47) · Glossary entry (FR47, AD-36) · source document (FR128/FR131, AD-43) · dictionary citation (FR30). A bare `origin` identifier is banned — it cannot tell them apart, and the word is already taken on the frontend, where `origin === 'user'` means a panel activation. Read the spine for which shape each one uses; they are not uniform (`segment.translation_origin` and `glossary_entry.term_origin` suffix the entity, while the Chapter document-origin group prefixes: `chapter.origin_author` · `origin_site_name` · `origin_url` · `origin_published_at`).
- Commits: `type(scope): Vietnamese sentence`, and that sentence states WHAT WAS FOUND, not just what was changed. The subject line is a MACHINE-READ interface — write the story id in its `5-9` form, not `story-5.9`: the dotted form made `git_evidence.py` match 1 commit out of 25 for Epic 5.
- A count states the SCOPE it measured. `count_in_chapter` measured a 4 KB evidence window while its doc-comment claimed "across the whole text", so a Chapter over 4 KB reported 2 matches where the truth was 40 — and the first 15 contract cases all used text shorter than the window, so it passed `cargo test --locked` and eight gates cleanly. Same shape hit Story 6.3 (an encoding comparison run on a 12-character slice) and Story 6.5.
- Fixed vocabulary in code: Tác phẩm→`Work` · Chương→`Chapter` · Chế độ đọc→`ReadingMode` · Hán Việt→`HanViet` · base/detachable layer→`BaseLayer`/`DetachableLayer`. `Project`/`Book`/`Novel`/`Document` are banned for `Work`. Exactly eight exemptions, all naming the STORE rather than the entity: `.atproj` · `project.db` · `StoreKind::Project` · `ProjectStore` · `PROJECT_MIGRATIONS` · `commands/project/mod.rs` · `ports/project_store.rs` · `tests/project_contract.rs`. Guarded by `src-tauri/tests/naming_boundary.rs` (Story 5.1) — this list and the gate's `STORE_EXEMPT` array must match item for item.
- Debt items close in words, three ways: `→ ✅ ĐÃ ĐÓNG <date> (Story x.y)` · `→ 🟡` with the remaining gap · `→ KHÔNG LÀM <date> (Story x.y) — <reason>`, where the reason states WHAT CHANGED. Never delete a closed item.
- A capability not yet built is not a spec mismatch. Don't edit `epics.md`/`prd.md` to match written code — record an owned debt item.

## Known pitfalls

- SILENT emptiness is this project's central failure class: a query returning 0 rows in 0.01 ms throws nothing and reaches the user as *"lookup returns no results"* with no traceable cause. An empty list doesn't say why it is empty — ask the `…HasLoaded` predicate before concluding "none", and check the ARGUMENTS going into that predicate: in Story 3.9 the predicate was right but the call site fabricated `totalCount` from `filteredCount`, so a filter that swept every row on a Glossary WITH data asserted "empty". Missed FOUR times (Story 1.16 · 2.10 · 3.9 · 6.10) and no gate guards it. The fourth was `0` carrying two meanings at once — "measured, and it is zero" and "nobody measured this": the median of N−1 zeros is zero, so the screen reported "M Chapters clean" for Chapters no one had scanned. A value that can be UNKNOWN gets an `Option`/`NULL`, never a `0` or a `.unwrap_or(0)`; `library_work.status IS NULL` and `chapter_done_count IS NULL` are the shape to copy.
- Every write to `target_text` whose text does NOT come from the typing buffer must set BOTH in the same operation: the comparison baseline AND the origin column (AD-47, a closed 7-line catalogue; the single named exception is FR101 restore). Forget the origin half ⇒ the TM pair carries a wrong label, no gate goes red, and it surfaces hundreds of sentences later as *"the AI translation no longer sounds like me"*. Same rule, wider than `target_text`: a write touches EXACTLY the columns the user agreed to change — "take theirs" in Story 3.10 wrote `SET translation, note, category` unconditionally, so a two-column file wiped notes the user had written.
- Segment boundaries are computed ONCE at import and stored; no code path recomputes them at load. Merging/splitting a SEGMENT = retire + create new, but merging/splitting a CHAPTER does NOT (only `chapter_id` and `ord` change). Confusing the two destroys the entire history of finished Chapters, permanently.
- Fix the TYPE so it tells the truth; don't lower a threshold, add `eslint-disable`, or move a pair onto an exclusion list to clear a red gate — all three give exit 0 on a broken product. Every exemption must be NAMED, carry its reason in place, and be able to die.
- Don't imitate a marker you haven't understood: `grep` for its occurrence count AND its definition before reusing it. No definition ⇒ write it out in plain words and raise it with Ice with a measurement.
- A green suite does NOT prove a new seam is guarded — Epic 3 hit this FIVE times in SEVEN days: 0/12 e2e specs touched the surface Stories 3.4b and 3.5 built; 68 Rust cases passed while `work_context` degraded to always returning `None`; 412 lines of ribbon tests never mounted the component; and in Story 3.10, removing the patch made two NEW cases red while the OLD case `..._take_theirs_updates_the_existing_row` stayed green. Before declaring a clause passed, `grep` the number of cases that ACTUALLY touch that surface, and counter-check by REMOVING the seam and running the OLD suite — it must go red.
- The counter-check has to be a REMOVAL at the seam itself, and three cheap ways to fake one all showed up in Epic 6. A case that calls the patched function directly guards itself, not the wiring — Story 6.15 deleted all SIX real calls from `mod wire` and the case stayed 15/15 green. Commenting a line out is not removing it, because a source-scanning gate doing `body.contains(...)` on raw text still matches the commented line — read code lines, the way `webimport_boundary.rs` does. And an assert that holds on BOTH branches guards neither: three Epic 6 cases passed on symmetric fixtures (0 images, symmetric role sets) where the two branches could not differ.


---

## `src/AGENTS.md` as of 2026-09-23 (commit 505c8bc)

<!-- Verified 2026-09-10 against 39ae75d. Managed by bmad-project-context; edits inside this block are replaced on refresh. -->

## src/ — Vue 3 + TypeScript

The frontend only renders and holds UI state. No business rule lives in TypeScript (AD-1); the single explicit exception is the text currently being typed in the Editor.

## Conventions that differ from defaults

- `invoke()` sends parameters in **camelCase** even though the Rust function takes `snake_case` ⇒ write `sourceLang`. BUT the fields of the RETURNED struct stay `snake_case` (`meta_schema_version`, `work_id`). The two directions differ — this is the easiest thing on the wire to get wrong.
- IPC adapters in `src/config/*.ts` NEVER throw: one `invoke`, one `try/catch`, returning the three-state shape `{ <value> | null, error: IpcError | null }`. The UI layer displays errors through `tError()`, not through `try/catch`. (`shortcutsState.ts` is not an adapter — it is Vue state calling down into `bootstrap.ts`.)
- Always type-check data crossing the wire AT RUNTIME. `IpcError` on the TS side is a claim about data that already crossed IPC, not a compiler guarantee.
- `verbatimModuleSyntax` is on ⇒ `import type` must be explicit. vitest sets `globals: false` ⇒ every test file imports its own `{ describe, it, expect } from 'vitest'`.
- `@click` in a `.vue` must be EXACTLY ONE `dispatch('<id>')` call — no other function, no inline code (`check:commands` Check A). Shortcuts and Auto-Lookup emit the same `dispatch(...)`: a direct call builds a second path that Check A cannot see.
- Command ids use the same dotted grammar as i18n keys (`review.accept_change`) — a bare id will be registered twice by two phases months apart and silently overwrite.
- A function run from a keyboard chord NEVER throws — it logs a diagnostic naming the cause and returns `false`. Don't "fix" it by switching mode: that is guessing the user's intent.
- An `⌥` chord must compare `event.code`, never `event.key` — `⌥W` produces `∑`, so `event.key === 'w'` is never true (`keys.ts` already names `Alt+M` → `µ`). If a counter-check for this comes back GREEN, suspect the mock before the test: raise it rather than changing the production predicate to match `happy-dom`.
- ⚠️ `check:commands` has three MEASURED blind spots — don't read its green as coverage: Check A only watches `@click`, so an in-place edit committed through `@change` needs no registration; and seeding `keys: ['Space']` or `keys: ['Alt+w']` both pass, because the gate catches chord COLLISIONS, not whether a chord is safe. A bare `Space` as a global chord is unsafe: `isTypingZone` covers only `INPUT`/`TEXTAREA`/`SELECT`/`contenteditable`, so it would `preventDefault()` every `<button>` in the app.
- Colour AND font size come only from tokens; no drop shadows, no gradients, no floating layers. An intermediate `opacity` needs a NAMED exemption.
- A directory carrying a concept has a `README.md` — missing today in `src/config/` and `src/selftest/`.

## Known pitfalls

- 🔴 A `Ref` does NOT auto-unwrap inside a `<script>` block, only in `template`. `if (someRef)` runs on the **object** and is therefore always true, and because it is valid TypeScript, `vue-tsc` stays silent. This bug passed NINE gates out of nine and is the reason the tenth (`check:lint`, type-aware) exists.
- 🔴 Five files must load under **plain Node**, because gates `import()` them to run BEHAVIOURAL checks against production code itself: `src/i18n/resolve.ts` (this file imports nothing at all), `src/commands/{index,registry,focus}.ts`, `src/layout/writeSchedule.ts`. No VALUE imports from `vue`/`dockview`; no `enum`, `namespace`, or parameter properties (`constructor(private x)`) — all three emit code, so Node rejects them. One offending line kills three checks at once. `src/layout/dockController.ts` exists for exactly this reason: `main.ts` injects functions into it rather than importing back.
- 🔴 The startup order in `src/main.ts` is mandatory, all three clauses: `applyTheme()` before `mount()` (otherwise every `var(--color-…)` is empty on the first render ⇒ a white flash — and on a packaged build that flash is SHORTER than on a dev machine, so the bug only shows up on someone else's computer); `installCommands()` before `mount()` (`dispatch` throws on an unregistered id); `loadFonts()` starts before `await loadBootstrapConfig()`.
- Register commands in `main.ts`, NOT in `App.vue` — an HMR round rebuilding the component calls `installCommands()` a second time and `register()` throws on the duplicate id.
- dockview's `onDidLayoutChange` fires CONTINUOUSLY while a sash is dragged: writing one `putConfig` per fire turns a 3-second drag into hundreds of serialised jobs through `store::Writer`. No gate goes red for that — it surfaced in Epic 2 as *"typing stutters"*. Every write cadence goes through `src/layout/writeSchedule.ts`.
- Three write-cadence constant pairs (🔵 FIXED 2026-08-29, Story 5.7 — previously "two pairs", now a third), only ONE of which carries the AD-35 guarantee: layout uses `IDLE_MS 500`/`HARD_CAP_MS 5000` in `layout/writeSchedule.ts` (no guarantee); the Editor uses `EDITOR_IDLE_MS 2000`/`EDITOR_HARD_CAP_MS 5000` in `panels/editorFlush.ts` (guaranteed); Chapter working position uses `POSITION_IDLE_MS 500`/`POSITION_HARD_CAP_MS 5000` in `panels/positionFlush.ts` (no — losing a position write loses ONE REMINDER, not work). Same shape, different guarantees — do not merge the three pairs.
- Write-cadence functions never read `Date.now()` themselves: every timestamp arrives as a parameter, so checks are deterministic and instant instead of having to `sleep` for real.
- No second OS window (AD-24): `addPopoutGroup` is dockview's only path that calls `window.open` — banned. `check:layout` Check C is an ALLOW-LIST of every `window`/`document` member `src/**` touches; adding a name is a decision that has to be written down.
- External content is NEVER rendered as HTML: no `v-html`, no equivalent (AD-16). Rust parses it into a structured data model; Vue renders from that model.
- `src/selftest/**` deliberately stays out of the release build (`#[cfg(debug_assertions)]` on the Rust side + dynamic `import()` on the frontend) — no production code may import statically from it.


---

## `src-tauri/AGENTS.md` as of 2026-09-23 (commit 505c8bc)

<!-- Verified 2026-09-10 against 39ae75d. Managed by bmad-project-context; edits inside this block are replaced on refresh. -->

## src-tauri/ — Rust + Tauri v2

The application core: every business rule lives here (AD-1). Its own workspace, `rust-version 1.85` — deliberately different from `tools/dict-build` (1.97.1), do not "sync" the two numbers. Read `SECURITY-NOTES.md` before touching `tauri.conf.json` or `capabilities/`.

## Conventions that differ from defaults

- 🔴 `panic = "abort"` turns every `panic!` into process death: no unwind, no `Drop`, no chance to flush the WAL, and on Windows release it does not even print. `catch_unwind` is useless here. Every `unwrap()`/`expect()` under `core/store/**` is a design error. Lock mutexes with `lock().unwrap_or_else(|e| e.into_inner())`; send on reply channels with `let _ = tx.send(…)`.
- 🔴 Because of that, a dependency's panic points are an architectural decision, and NO GATE COUNTS THEM. `docx_rs` was rejected for exactly this: 140 panic points across `src/reader/`, so a truncated deflate stream kills the process instead of failing a "corrupt file" gate — which is why we read OOXML ourselves. `docx_boundary.rs::docx_module_carries_zero_panic_points_in_product_code` guards OUR code only.
- ⚠️ Cargo ALWAYS builds a test target with `unwind` (the harness needs `catch_unwind`), and `[profile.bench]` sets `panic = "unwind"` for the same reason. So no green test run anywhere proves anything about `abort` behaviour — a case that reads as "one red test" under the harness is "the whole app dies" in a release build.
- 🔴 Two-layer shape for every IPC surface: ① a pure function taking `Option<&Store>` — this is what `tests/**` can call without a webview; ② a thin `#[tauri::command]` in a nested `wire` module, taking `State` through **`try_state`**. Never `state()`: opening the store may have failed and `app.manage()` may never have run ⇒ `state()` panics ⇒ `abort` kills the process. The command name on the wire IS the function name, so the shell must live in a nested module rather than carry a suffix.
- Build IPC errors ONLY through `IpcError::new(code, message_key, params, retryable)` — the four fields are private and `new` is the only place `message_key` meets `params`. A struct literal bypassing it compiles cleanly, passes every gate, then puts a literal `{path}` on the user's screen. Do not put `#[serde(rename_all = "camelCase")]` on it: the four field names are wire, locked by `tests/ipc_contract.rs`.
- `message_key` is a CLOSED catalogue declared by `macro_rules! message_keys!` in `core/i18n/` — one declaration generates the `enum` + `ALL` + `as_str` + the required-parameter table. Do not hand-write a parallel list: the `vi.json` sync test runs OVER `ALL`, so a variant missing from `ALL` gives a falsely green test. Same shape for `scope_kinds!` in `core/scope/kinds.rs`.
- No display text in Rust, not even `impl Display` — the `Display` impls for errors are diagnostics for logs and are written WITHOUT diacritics. String literals under `src-tauri/src/**` are unaccented (`khong`, not `không`); `tests/**` carries a NAMED exemption and keeps its diacritics.
- Modules are named by DOMAIN CONCEPT, not by capability group — `C1`–`C10` are product vocabulary and never appear in a module name.
- Tests: two filename families, two roles — `*_contract.rs` (wire shape, declared tables, registered keys) and `*_boundary.rs` (which module may NOT carry another module's vocabulary). A test function name is an assertive SENTENCE, not `test_foo`: `capabilities_directory_holds_exactly_the_one_reviewed_file`.
- Pin crates with `=`: `"2.6.3"` in Cargo means `^2.6.3`, and the lock only holds the exact number until the first `cargo update`.
- `[profile.release]` is frozen (`codegen-units = 1` · `lto` · `opt-level = "s"` · `panic = "abort"` · `strip`) — changing it makes every NFR6 measurement incomparable. `[features]` has NO `default = [...]`; the empty default set is what keeps `tauri-plugin-wdio-webdriver` out of `cargo tree`, and therefore out of the release.

## Known pitfalls

- 🔴 `capabilities/` may hold EXACTLY ONE file, `main.json`. Tauri loads **every** file in that directory via the glob `{capabilities}/**/*` — any extension, recursively — so an `extra.json` with `"permissions": ["fs:default"]` grants a new IPC surface while a test reading `main.json` stays green. Enforced by `tests/config_invariants.rs::capabilities_directory_holds_exactly_the_one_reviewed_file`.
- The permission set is genuinely minimal, exactly three entries (`core:path:default` · `core:event:default` · `core:resources:default`), not the `core:default` bundle. Adding a permission is an architectural decision.
- The CSP stays as it is, never loosened: no CDN, no external fonts, no external images — this is why FR127 downloads images into `.atproj` instead of keeping links. `assetProtocol.scope` is `$RESOURCE/fonts/**` today and only that.
- 🔴 No LISTENING port in a release build (AD-45). A tool needing a server must pass through TWO layers at once: `optional = true` + a feature outside `default`, **and** `#[cfg(debug_assertions)]` at the seam. A `cfg` alone is not enough — it removes **code**, not the **dependency**.
- AI streaming goes through the Tauri Channel API, not through separate events, and there is NO self-reconnecting SSE client (AD-22): auto-reconnect creates an entirely new request, so with BYOK the user is billed twice. Every AI call must be cancellable mid-flight.
- API keys never travel over IPC — the `keyring` crate directly in Rust; the frontend only knows "configured / not configured".
- Every write goes through the serialised `store::Writer` of its store; no module opens its own write connection. `PRAGMA wal_autocheckpoint = 0` — when to checkpoint is the application's decision.
- Schemas are versioned and migrations are FORWARD-ONLY. Meeting a version newer than the application ⇒ refuse to open and say so clearly, NEVER write to it. Migrations run inside a transaction, after a backup.
- Editor flush contract (AD-35): idle 2 s · hard cap 5 s NOT reset by keystrokes · confirm · leaving a segment · closing a Work. A pure debounce never fires while the user keeps typing — unbounded work lost while still "meeting the auto-save spec". A flush is only done once it has reached the WAL. DISCRETE actions (FR94, FR58) write IMMEDIATELY, not through the typing buffer.
- Never merge dictionary sources, anywhere: results come back per source, disagreements preserved, and the `source` column is mandatory on every sense record. Do not merge `zh` with `en` either.
- No mechanism writes to the Glossary on its own — import scanning and review harvesting write to a separate pending table; only a user's approval moves an entry into the Glossary.
- The zh/en routing predicate is the SHAPE OF THE QUERY STRING, not the Work's language: selecting `API` inside a Chinese novel while filtering `lang='zh'` returns 0 rows even though the `API` entry exists. Lower-casing ADDS a key, it does not REPLACE the original — 1,635 English headwords carry meaningful capitals.
- `LIKE` is banned on the lookup hot path — measured 20–50 ms.
- SQLite's `trim()` only strips **ASCII spaces**, so `CHECK (trim(x) <> '')` does NOT block tabs, newlines, NBSP or U+3000 — measured 2026-08-19 on SQLite 3.53.4, and the empty-guard the Story 3.1 spec pre-wrote leaked seven ways. An empty-guard in DDL must list ALL 25 `White_Space` code points (`GLOSSARY_ENTRY_DDL`), exactly the set Rust's `str::trim()` strips; adding a character to one class means adding it to the other IN THE SAME PASS. No gate guards this pair.

### The import pipeline

- The step order is a VALUE, `PIPELINE_ORDER: [Step; 7]` in `core/segment/pipeline.rs`, consumed by `run_import_with_order` — so a wrong order is expressible at runtime and testable through the real runner, instead of only being greppable in source. `validate_order` runs once, before any step, and rejects a duplicated, missing, or empty order (a missing `DecodeEncoding` yields corrupt text plus 0 segments and throws NOTHING). Guarded by `segment_pipeline_boundary.rs`.
- ⚠️ The order decides what LATER steps can still see, and this bit silently three times in Epic 6. Line-joining (step 4) runs BEFORE Chapter splitting (step 5), so a `^Chuong \d+` anchor pattern — the shape the mockup teaches — matches 0 times on a source with no blank lines and the whole file comes out as exactly ONE Chapter, with no error thrown. On the `Blob` and `.docx` paths, steps 3–4 run while the pipeline still holds a single unit covering the whole document, so `joined_line_count` is `None` for every Chapter and `cleanup_report` can only attach to the first one. When adding a step, state what the steps before it have already consumed.
- Anything keyed on `segment.ord` must be maintained INSIDE the four Chapter-reorganising transactions. `commands/chapter.rs` and `commands/segment.rs` renumber `segment.ord` in three places and the word `asset` appears 0 times in either file, so a merge left orphan `asset` rows plus files in `assets/` that nothing deletes — invisible to 1,302 green cases. `chapter_position` right next to it already had the discipline. Guarded by `project_contract.rs::every_asset_anchor_never_exceeds_the_living_segment_count_of_its_own_chapter_after_any_reorganisation`.

### Derived stores: `library-index.db` and `meta.json`

- Both are DERIVED. Only `Indexer` writes `library-index.db`, and only after `.atproj` has been written. Deleting `meta.json` must always be a safe operation. `commands::project::open_work` (Story 5.7, FR12) reopens a `.atproj` from disk but only READS the derived stores.
- The orphan flag is the one piece of state NOT derivable from `.atproj`, so it does not live in the derived store: it sits in `global.db` table `library_orphan` (`core::library::orphan_store`). A reminder the user must actively dismiss is a recorded user decision, not a cache.
- 🔴 `global.db` and `library-index.db` share no transaction, so `Indexer::rebuild` writes `global.db` FIRST and only then deletes from `library-index.db`. If step two fails the row exists in both places and the next scan fixes it — the reverse order loses the reminder PERMANENTLY. Guarded by `library_index_contract.rs::orphan_write_order_is_fail_safe_write_global_before_deleting_from_index`.
- ⚠️ ONE-WAY DOOR: `global.db` is under AD-30 (forward-only migration; refuses to open a newer schema). Once its step 6 has run on a user's machine, downgrading to a build that does not know step 6 makes `global.db` REFUSE TO OPEN — losing the shared Glossary and every pinned entry, not just an index.
- `library_work` is at `LIBRARY_INDEX_MIGRATIONS` target 7 and is rewritten IN PLACE (`LIBRARY_WORK_DDL`) on every bump. `status IS NULL` and `chapter_done_count IS NULL` both mean *NOT YET KNOWN* — never "not started" or `0`. They match no filter, because `NULL IN (...)` is never true in SQL; no separate `WHERE` branch is needed.
- `Indexer::rebuild` also opens each Work's `project.db` READ-ONLY through `ReadOnlyDb` (the allow-list is `{Dict, Project}`) to harvest text into `library_segment` plus three FTS5 indexes. A Work whose `project.db` is missing, corrupt, or newer still gets its metadata upserted; only its text is skipped, counted into `RebuildOutcome::text_skipped`, and the whole `rebuild` does not fail.
- ⚠️ A NEW write path into `segment` or `chapter` that does not run `Indexer::rebuild` afterwards makes the SEARCH index lie silently — finding deleted sentences, missing new ones — and no gate goes red, because `library_segment` is only synced inside `rebuild` itself (AD-8: one write path). The same class of bug applies to `chapter_count`: all four Chapter-organising operations survive only because they run the shared four-step template in `commands/lifecycle.rs` (SQL write → `WorkMeta::rebuild_from_store` → `write_atomic` → `reindex`). Measured 2026-08-27: removing step four gave **0 failures** across 34 binaries.
- 🔴 `library_target_fts_nd` (the diacritic-LENIENT index of the translation half) NEVER runs alone (AD-27). `Indexer::search` always runs the main index (`library_target_fts`/`library_source_fts`) FIRST, every call, no exception; `_nd` runs additionally only when `mode = Lenient` or when the exact run returned 0 rows on a NON-EMPTY index (`SearchReport::widened`). Widening on an EMPTY index is explicitly banned (`indexed_segments > 0` is required) — it would report "widened to lenient" for a library with nothing in it.
- The SOURCE half (`library_source_fts`, `trigram`) has NO `_nd` index — it is always diacritic-sensitive, even in a lenient run, because trigram needs a substring verification step in Rust and Rust cannot fold diacritics yet (the `đ`/`Đ` + fold-function debt, `deferred-work.md`, owner Ice).
- Full reasoning for all of the above — why the orphan flag moved, why `remove_diacritics 1` is a trap (lenient by HALF: it misses two-mark characters like `ễ`/`ệ`), why both tokenizers are mandatory: §Design Notes of `5-3-quet-lai-thu-muc.md` · `5-4-bon-trang-thai-vong-doi.md` · `5-5-tien-do-tac-pham.md` · `5-8-to-chuc-lai-chuong-sau-khi-nhap.md` · `5-9-tim-kiem-full-text-xuyen-library.md` · `5-10-hai-che-do-dau.md`.


---

## `scripts/AGENTS.md` as of 2026-09-23 (commit 505c8bc)

<!-- Verified 2026-09-10 against 39ae75d. Managed by bmad-project-context; edits inside this block are replaced on refresh. -->

## scripts/ — the rules of a GATE

Thirteen `check:*` gates enforce declarative claims across the WHOLE TREE (*"no hard-coded colour anywhere"*) — a role no single test can carry. Adding a gate means editing THREE lists: `package.json` · `.github/workflows/ci.yml` · `.githooks/pre-push`, and `check:gates` guards all three.

⚠️ `test-story.mjs` lives here but is NOT a gate — it is the scoped dev loop (`npm run test:story <id>`), so none of the rules below apply to it and it must stay OUT of the three lists. It has no population floor and no self-check by design: it never issues a verdict, it only selects which tests to run. Nothing may depend on its exit code as a quality signal; `pre-push` remains the gate.

## Conventions that differ from defaults

- The exit code is the verdict. No gate logs and carries on.
- 🔴 An infrastructure failure is NOT a red check: if a file cannot be read ⇒ `abort()` and exit non-zero with the sentence *"this is an infrastructure failure, not a pass"*. Never report a result that does not exist.
- 🔴 No verdict may read its parameters from the very thing it is checking. The WCAG floor, the role list, the exclusion list — frozen IN the script. Measured: all three escape routes gave exit 0 while the product carried a 4.245:1 contrast pair.
- Population floor: *"an empty tree is not a clean tree"*. A gate counts its files and `abort()`s below the floor; the floor sits at ~80–85% of the real count. The floor is a LOWER bound, so surplus files never make a gate red — they only make the floor meaningless. Adding files to `src/**` means revisiting the floor.
- Plain Node, no bash — `npm run` goes through `cmd.exe` on Windows, and a gate that guards half the platforms cannot guard NFR14.
- No npm dependency for a gate. The TOML/CSS parsers in this directory are hand-written strict subsets, and syntax outside the subset ⇒ FAIL, never skip.
- A NEW gate must carry a SELF-CHECK proving it CAN go red and does not go red wrongly — a gate that has never been red is a gate nobody knows is running.
- ⚠️ A green self-check proves the gate CAN go red. It does NOT prove the real assert runs through the gate's own filter. `cleanup_boundary.rs` declared a `#[cfg(test)]` filter with two passing self-checks while its two real asserts scanned raw `code_lines`, so a call living inside a test block counted as the real call — the self-checks made the gate LOOK guarded. When you add a filter, add a case proving the REAL assert uses it.
- A gate that scans source must anchor on something both spellings share. `code.contains("run_import(")` does not match `run_import_with_order(`, and in Story 6.2 the positive-verification case then ratified that blind spot as the specification. A positive check written from the gate's own behaviour will sign off its own gaps — seed the actual violation instead.

## Known pitfalls

- ⚠️ Only 4/13 gates have a self-check today (`check-gates` Check C · `check-layout` Check D · `check-panel-refs` Check C · `check-debt-owner` Check B). The other eight have not proved they can go red — don't read a green run from them as a guarantee. Owned debt item in `deferred-work.md`.
- ⚠️ `abort()` is not universal either: `check-panel-refs.mjs:78` has the wrong shape (exit 2, different wording), and `check-scope`, `check-scope-bundled` and `check-dict-manifest` have no `abort()` at all — they call `process.exit(1)` bare, so an infrastructure failure there reads exactly like a red check.
- A gate without the `check:` prefix must appear in all three lists — Check F guards that specifically, because Checks A and D only walk `check:*` names. ⚠️ The `REQUIRED_SCRIPTS` table has EXACTLY ONE entry today (`test`). `test:e2e` is the second gate and deliberately sits OUTSIDE the table (`pre-push` excludes the e2e suite on purpose), so it is guarded in only TWO of the three lists: delete the `npm run test:e2e` step from `ci.yml` and all six checks stay green. Don't read a green `check:gates` as "the three lists agree." (No owner yet: Story 3.9 closed without changing a line of `REQUIRED_SCRIPTS`.)


---

## `tests/AGENTS.md` as of 2026-09-23 (commit 505c8bc)

<!-- Verified 2026-09-10 against 39ae75d. Managed by bmad-project-context; edits inside this block are replaced on refresh. -->

## tests/frontend/ — vitest

Role: behaviour of pure modules, DOM-touching code, and `.vue`. Four acceptance paths that do not overlap — static gates `scripts/check-*.mjs` (declarative claims across the whole tree) · `src-tauri/tests/**` (contracts, boundaries, config invariants) · vitest · `e2e/**` (real WKWebView/WebView2). Before writing a new check, ask: does this claim already have an owner on another path? Two paths guarding one claim is two sources of truth.

## Conventions that differ from defaults

- `happy-dom` is NOT WebKit. Every claim about geometry, layout, or a real engine belongs to a probe or to e2e — not to vitest.
- Tests live in `tests/frontend/**`, NOT co-located in `src/**`: four gates count the `src/**` population and a test file dropped in there inflates the denominator, plus two collisions (`check-i18n` Check A goes red on Vietnamese text, `check-tokens` Check B goes red on a hard-coded colour).
- `tsconfig.json` must `include` the test tree — an unchecked test tree is a test tree that will rot: it keeps running green while the types of what it checks change underneath it.
- Every `happy-dom` patch lives in `tests/frontend/support/setup.ts`, each entry carrying one line saying what it lacks, AND SOMEONE READS IT. That list is a measurable debt — 3 entries today, two of which no longer have a reader.
- No `vi.useFakeTimers()` when the function already takes the timestamp as a parameter: wrapping a fake clock trades a guarantee for a habit.
- `vitest.config.ts` sets `fileParallelism: false` deliberately, at a cost of 27 s → 99 s. It was measured: worker contention gave 5 red cases that were not about the code — baseline 822/822 green, 835 green + 5 red with the story, `--no-file-parallelism` 840/840 green, and the four red files run alone 42/42 green while none of them even loaded what the story touched. Before blaming a diff for a red here, re-run the file alone.

## Known pitfalls

- 🔴 The wrong path is very cheap and has to be blocked by hand: adding a `?.` to PRODUCTION CODE to clear a red. That is a branch the types say never runs — permanently dead code in the product, serving a mock. Gaps in the mock get patched in `setup.ts`; product defects get patched in `src/`.


---

## `e2e/AGENTS.md` as of 2026-09-23 (commit 505c8bc)

<!-- Verified 2026-09-10 against 39ae75d. Managed by bmad-project-context; edits inside this block are replaced on refresh. -->

## e2e/ — WebdriverIO in a real webview

One role only: behaviour in a REAL WKWebView/WebView2. This suite runs NIGHTLY on macOS (`schedule` 18:00 UTC = 01:00 Ice time, plus `workflow_dispatch`) — NOT on `push`, not in `pre-push`, and the Windows/WebView2 half has never run. Manual: `npm run test:e2e`.

## Known pitfalls

- 🔴 The driver's `.click()` is banned, use `realClick()` from `e2e/support/pointer.mjs`. The driver fires `click` BEFORE `focusin` — the reverse of a real mouse — so it both gives RED for the wrong reason and GREEN on a broken product. Enforced by `no-restricted-syntax` in `eslint.config.js`.
- Each spec opens a real window (~1.5 min) and it writes to the runner's REAL `global.db` and Library root if the two redirect env vars fail to reach the child process. `wdio.conf.mjs` carries a POSITIVE self-check — `global.db` must sit inside the temp directory — that runs before anything is deleted. Do not remove that check.
- Never assert an ABSOLUTE number against a SHARED resource — that is a claim about what the other specs did. `story-5-3-rescan` asserted `Đã lập chỉ mục 1` and the first full-suite run gave `Đã lập chỉ mục 22`; run alone it was green, so it read as an ordinary pass from the day it was written. Assert the DIFFERENCE against the moment before the action. For the same reason, a spec's scratch directory goes in its own temp dir, never the shared system temp — one crashed run's leftovers kill the next.
- Activating a `<button>` from the keyboard does not work here: `focus()` in JS plus `browser.keys(['Enter'])` leaves the handler UNRUN, while `realClick` on the same button in the same session runs it. So the "by keyboard" half of an AC has no automated acceptance path — record the debt, don't round it up to passed.
- 🔴 A red nightly does NOT mean the product regressed. The first four nights (20–23 Aug) were red twice, and both reds died at the IPC BRIDGE — `core.invoke not available after 5s` ⇒ the fixture could not create a Work ⇒ 0 rows in the grid after 30 s — while the `check` job was green on both platforms all four nights. Read the error before changing a line of product code; and don't patch it with `continue-on-error` or a retry loop, both of which turn the job into something that can never go red. (Owner: Ice since 2026-08-24, `deferred-work.md`.)
- 🔴 Each spec FILE gets a new app process and fresh `$APPDATA`/Library root — decided 2026-09-14, mechanism in `wdio.conf.mjs::onWorkerEnd` (kills the app listening on the embedded WebDriver port after every spec file, waits for the port to close, then hands the next spec file a fresh temp dir pair; `@wdio/tauri-service` respawns the app on its next health check). Before that, ONE app process served the whole run (measured 2026-09-12: one `pid` across two WebDriver sessions), and 12 of the 15 cases that failed in the full run passed when their spec ran alone. What carried forward was measured on 2026-09-14 as three layers: frontend module and `<KeepAlive>` state, in-process Rust state, and on-disk state in the shared dirs; a relaunch that kept the dirs cleared only the first two. State still carries FORWARD **within** one spec file, between cases — that part is unchanged: `openWorkspaceWithWork()` still calls `resetPanelState()` (`support/panelReset.mjs`) between Works in the same file, and only for the five panel modules it names. A red case in a spec file you did not just edit can still be a fixture/ordering issue local to that ONE file — read the case order inside the file before assuming a cross-FILE leak, since that channel is now closed. Full history and the measurements behind this decision are in `spec-e2e-cach-ly-trang-thai-giua-cac-spec.md` and `deferred-work.md` (closed G1 entry).


---

## `tools/dict-build/AGENTS.md` as of 2026-09-23 (commit 505c8bc)

<!-- Verified 2026-09-10 against 39ae75d. Managed by bmad-project-context; edits inside this block are replaced on refresh. -->

## tools/dict-build/ — the dictionary builder

An INDEPENDENT Rust workspace, not a member of `src-tauri` and with no parent workspace. `rust-version 1.97.1` diverges from `src-tauri` (1.85) deliberately — do not sync the two numbers. Raw input in `docs/dics/`.

## Conventions that differ from defaults

- 🔴 `dict-tran-van-chanh.db` must stay a SEPARATE `.db` file. Trần Văn Chánh (1999) is still in copyright — the author is alive; the digitiser's CC0 grant cannot erase copyright in the underlying work. This layer ships detached precisely because of that risk: FR112 is enforceable by deleting exactly one file. Do not merge layers, do not "consolidate for tidiness", do not fold its data into `dict-core.db`. (`src-tauri/tests/dict_sources.rs::deleting_any_detachable_layer_keeps_the_whole_lookup_suite_green` guards that removing a layer stays green — it does NOT guard against someone merging layers.)
- 🔴 A schema change ⇒ rebuild ALL FOUR `.db` files with `--layer all` ⇒ four new SHA-256s in `dict-manifest.toml` ⇒ a new release. True even when a layer's raw input has not changed by a byte. The source of truth is `dict-manifest.toml` + `src/schema.rs`, NOT `README.md` — the README still says `--layer all` builds *"exactly three"* files and `src/build.rs::run_all` still says *"two detachable layers"*, both predating the addition of `tran-van-chanh`.
- Three mandatory fields per manifest entry: `url` · `sha256` · `source_version` — `source_version` is the version of the RAW INPUT, not of the `.db` file. Never fill a placeholder value "just to have one".
- `is_han` has two DELIBERATE copies (`src/char_idx.rs` and `src-tauri/src/core/dict/mod.rs`) because the two workspaces cannot import across each other. Two gates guard it: `dict_lookup.rs::han_ranges_are_verbatim_from_dict_build_char_idx` reads this file as text and compares the CJK ranges, and `dict_boundary.rs::exactly_one_definition_of_is_han_exists_under_src_tauri`. Fix one side and forget the other and a lookup hits a `char_idx` that never indexed that character ⇒ empty, no error.
- Strings here carry a NAMED exemption from `check:i18n` Check A, so accented Vietnamese is fine.
- `// dict-build:allow <token> — <reason>` on the line above a violation is read by `check:dict`; the em dash and a non-empty reason are both required by `ALLOW_RE`. These are code, not prose — never strip them.

## Known pitfalls

- `npm run check:dict-manifest` checks SHAPE only — it has to stay green on a runner holding no dictionary bytes, so it never opens a `.db` file. It catches a dropped layer (it demands exactly 3 `[[detachable]]` entries, by name); it does NOT catch data mixed between files.


---

## `.githooks/pre-push` header comment as of 2026-09-23

```sh
# ══════════════════════════════════════════════════════════════════════════════════
#  Cổng chặn lúc PUSH — thay chỗ mà GitHub Actions bỏ trống (Ice chốt 2026-08-11).
# ══════════════════════════════════════════════════════════════════════════════════
#
# 🔴 VÌ SAO CHẶN CHỨ KHÔNG BÁO CÁO
#
# Phát hiện nặng nhất của retrospective Epic 1: CI đã chạy 12 lượt và ĐỎ cả 12, suốt
# sáu ngày, mà không một story nào biết — vì nó chỉ **báo cáo** lên một tab không ai mở.
# Một cổng chặn không có lối hỏng đó: nó đứng chắn đường push, nên nó được đọc hay
# không cũng không quan trọng.
#
# Cùng lý do, cổng này KHÔNG viết log rồi tiếp tục. Đỏ là dừng.
#
# ── Phạm vi: cái gì Ở TRONG và vì sao ────────────────────────────────────────────
#
# TRONG (đo 2026-08-11: cổng 11s · build 5s · cargo test 34s; + test frontend ~2s từ 2026-08-12):
#   **mười một** cổng đọc-tệp · `npm run test` (vitest) · `npm run build` · `cargo test --locked`
#   🔵 2026-08-19: chín → mười (`check:panel-refs`, Story 2.12) → mười một (`check:debt-owner`,
#   Story 2.13). Con số ở đây và ở dòng `echo` phải đi cùng vòng lặp `for gate in …` bên dưới.
#
# NGOÀI, có chủ ý:
#   - `check:scope` và `check:scope:bundled` — hai cổng này dựng một cửa sổ Tauri thật
#     và cần cổng 1420 trống. Chúng trượt nếu Ice đang mở `npm run tauri dev`, tức một
#     cổng chặn sẽ chặn nhầm vào đúng lúc đang làm việc. Chạy tay:
#         npm run check:scope && npm run check:scope:bundled
#   - Bộ e2e — mỗi spec mở một cửa sổ thật, tốn ~1,5 phút, VÀ nó sửa `global.db` thật
#     của người chạy (xem §Giới hạn ở `e2e/wdio.conf.mjs`). Không có việc gì trong một
#     cổng chạy mỗi lượt push. Chạy tay:  npm run test:e2e
#     🔵 2026-08-20 (lượt rà soát Story 3.3) — VÀ NÓ THÔI NẰM NGOÀI MỌI CỔNG. Câu trên
#     đúng nguyên văn về `pre-push`, nhưng lượt rà soát đo ra một vế nó KHÔNG nói: bộ e2e
#     cũng không có trong CI, tức nó chạy đúng 0 lần trong bất kỳ cổng nào và mọi mệnh đề
#     "webview thật" đi qua `pre-push` xanh + CI xanh mà chưa từng được đo. Nay job `e2e`
#     ở `.github/workflows/ci.yml` chạy trọn bộ theo NHỊP ĐÊM (`schedule`) và khi bấm tay
#     — vẫn KHÔNG ở `push`, vì bộ này còn một mục nợ chập chờn chưa chẩn đoán. Nhịp đêm
#     là chỗ đọc kết quả; `pre-push` giữ nguyên vai và nguyên lý do loại trừ.
#
# ⚠️ GIỚI HẠN THẬT, ghi ra thay vì để người sau tưởng nhầm: cổng này chạy trên **macOS
# của Ice**, nên nó KHÔNG nói được gì về nửa Windows.
#
# 🔵 CẬP NHẬT 2026-08-19 — hai mệnh đề ở đây đã hết đúng, sửa tại chỗ. Bản trước viết
# *"GitHub Actions nay chỉ chạy khi bấm tay (`workflow_dispatch`, Ice chốt 2026-08-12 —
# hạn mức để dành cho dự án khác)"* và *"trọn phần Windows đã dời về CUỐI dự án"*. Cả hai
# đã bị chính `ci.yml` lật:
#   · `ci.yml:46-49` khai `push:` + `pull_request:` + `workflow_dispatch:` — lượt tạm dừng
#     sống đúng MỘT ngày, rồi quyết định "repo công khai" của Ice làm lý do hạn mức hết
#     hiệu lực (`ci.yml:24-38`);
#   · `ci.yml:70-81` chạy `windows-2025` trong CÙNG matrix với macOS ở MỌI lượt, và
#     `ci.yml:343-508` dựng + đo ba biến thể `.msi` mỗi lượt.
#
# ⇒ Nửa Windows NAY có đường nghiệm thu: mỗi lượt push. Nhưng nó không nằm ở cổng này —
# `pre-push` xanh vẫn KHÔNG nói gì về Windows, nên vẫn phải đọc lượt CI trước khi kết
# luận. Xem action item A5 của retrospective Epic 1.
#
# Bỏ qua một lượt:  git push --no-verify
# Gỡ hẳn:           git config --unset core.hooksPath
```
