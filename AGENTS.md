<!-- bmad:context -->
<!-- Condensed 2026-09-23 (Ice): rules only. The history and measurements behind each rule moved verbatim to `_bmad-output/implementation-artifacts/agent-rules-evidence.md`. Managed by bmad-project-context; edits inside this block are replaced on refresh. -->

## AuraTranslate

Fully offline dictionary-lookup and translation workspace. Tauri v2 · Rust in `src-tauri/` · Vue 3 + TypeScript in `src/` · bundled SQLite. GPL-3.0-or-later.

## Where things are

- Architectural invariants (ADs, Stack table, Consistency Conventions): `_bmad-output/planning-artifacts/architecture/architecture-AuraTranslate-2026-08-02/ARCHITECTURE-SPINE.md`
- Story specs, `sprint-status.yaml`, debt ledger `deferred-work.md`: `_bmad-output/implementation-artifacts/`
- Directory rules: `AGENTS.md` in `src/` · `src-tauri/` · `scripts/` · `tests/` · `e2e/` · `tools/dict-build/`.
- Why a rule exists: `agent-rules-evidence.md` (read on demand only). `project-context.md` is frozen history (2026-08-20). `docs/` is raw input for `tools/dict-build`, not documentation.

## Policy

- Default branch is `master`; a workflow on `main` never runs and throws no error.
- Never commit `.db` files (AD-25). Dictionary data ships through GitHub Release + `dict-manifest.toml`.
- New dependency (NFR15): read the licence in the DOWNLOADED source (`~/.cargo/registry/src/…`, `node_modules/…`), per-file headers when there is no `LICENSE`, and record it in the spine's Stack table BEFORE adding. GPLv3-compatible only; MPL-2.0 passes unless a file carries an Exhibit B notice.
- Changing an architectural invariant is a new `AD` in the spine, drafted by Winston, not a line of code. Next free number: scan the spine AND every unwritten `ad-brief-*.md`.
- Two valid options ⇒ present both with measurements for Ice; never pick one and move on.
- Dirty tree before a story ⇒ ask Ice, then commit it separately first.
- Agent cost is the area under the context curve, and a byte read early costs far more than one read late (evidence: `agent-token-economics.md`). So: split a story into ~4 phases (plan · Rust · Webview · Tests that move), one FRESH agent per phase, handed off through a file on disk, each under ~250 k context (a guideline, not a gate). Keep up-front reading short. Read large files with `offset`/`limit`; a file too large to hold as a map should be split, not re-peeked.

## Tests: run what the change touches

- Dev loop: run only the tests for what changed — `npm run test:story <id>` (`-- --list` to preview), one `cargo test --test <name>`, or one vitest file. It selects tests; it is never a verdict.
- Do NOT run the full suite (all of `cargo test`, all of vitest, all gates) after each edit, each phase, or before each report. `pre-push` runs it once per push, and that is the full run.
- Run the full suite by hand only when: the change touches shared wiring (`Cargo.toml`/`Cargo.lock`, migrations, command registration in `lib.rs`, `tauri.conf.json`, `vitest.config.ts`, `scripts/check-*`); a failure looks order- or cross-module-dependent (then only a full run is evidence, never a subset); or Ice asks.
- `pre-push` always runs the 11 gates; it skips vitest, build and `cargo test` when every pushed file is documentation (`*.md` except the root `AGENTS.md`, which `naming_boundary.rs` reads; `_bmad-output/**` except `*-ban-do/`). Never `--no-verify`.
- Counter-check a new guard by REMOVING the seam in production code and re-running that guard's test target (not the whole suite). The removal must be real — moved or deleted, not commented out, not duplicated elsewhere — must keep the call's signature, and the red must be for the reason being measured.
- `npm run build` before `cargo test`: without `dist/` it fails at compile time.
- Before `done`: read the CI run (the Windows half and the UTC timezone run only there) and the latest nightly e2e run (`schedule`, macOS only; manual: `npm run test:e2e`). Red ⇒ write down why.
- `check:scope`/`check:scope:bundled` are outside `pre-push` (they need port 1420 free); CI runs them.
- Adding a gate = three lists (`package.json` · `.github/workflows/ci.yml` · `.githooks/pre-push`), guarded by `check:gates`; `test:e2e` is the named two-list exception.
- A green suite does not prove a new seam is guarded: count the cases that actually reach it.

## This machine (Ice's Mac)

- Tests red with no code change ⇒ suspect the environment first: LuLu blocks freshly built test binaries that bind a local port. Re-run the SAME binary twice before touching code.
- A freshly built, unsigned Rust binary costs ~204 ms per exec (Homebrew `rustc`; `codesign -s -` removes it). Count process spawns before calling a tool slow. `cargo-nextest` was measured and rejected — don't install it.
- Compare two timings only from the same cache state and machine load.
- A background watcher greps a sentinel the script really prints (emit it from `trap … EXIT`) and caps every step: `perl -e 'alarm shift; exec @ARGV' <secs> <cmd…>` (exit 142 = timeout). macOS has no `timeout`.
- `tauri.conf.json` bundles only `fonts/` and `license/`, not `dict`; tests read dictionaries via `CARGO_MANIFEST_DIR`. A green Rust suite says nothing about dictionaries in a packaged app.
- A measurement states its build and population. Never mark something passed by inference; what cannot be accepted now goes to `deferred-work.md` with an owner.

## Code comments

- Default: no comment. Names, types and test names carry the meaning.
- Allowed only, one or two lines, in English: a workaround for an external bug or platform quirk (what and why); a non-obvious invariant whose violation still compiles and passes tests.
- Machine-read comments are code and are never stripped; each carries a mandatory reason: `aura-allow-*` (`check:tokens`, `check:i18n`), `// dict-build:allow <token> — <reason>` (`check:dict`, em dash required), `eslint-disable` (`reportUnusedDisableDirectives: 'error'`).
- Never in code: story ids, dates, measurements, incident history, review findings, 🔵 corrections, banners, or a restatement of the next line. Those go in the commit message or the spec.
- Doc comments on public items only when the contract is not evident from the signature.
- No mass cleanup of existing comments. In lines you are already changing, delete comments that break these rules.

## Specs, handoffs, ledger

- One fact, one place. Spec = intent, decisions, acceptance. Phase handoff file = working notes for the next agent, never copied into the spec. Commit message = what was found. Link, don't copy.
- Don't restate AGENTS.md, the spine or the PRD in a spec; cite by id (`AD-35`, `FR101`).
- `## Implementation Notes`: at most ~10 bullets for the whole story — decisions and surprises a later reader needs. No per-phase narration, no command logs, no intermediate test counts. Review Triage Log: one line per finding.
- `deferred-work.md` is over 1 MB: never read it whole, `grep` it. A new item is at most 5 lines: what, why it matters, owner, pointer. `Chủ:` names Ice, a persona, `Story X.Y`, `Epic N` or a sprint-status item id (`B7`); `check:debt-owner` rejects a vague owner such as "a later story touching X". Close items in words — `→ ✅ ĐÃ ĐÓNG <date> (Story x.y)` · `→ 🟡 <remaining gap>` · `→ KHÔNG LÀM <date> (Story x.y) — <what changed>` — and never delete one.
- In specs and the ledger, a claim that stops being true is fixed in place with 🔵 and a date. In AGENTS.md and in code, just replace it; git keeps the old text.
- A capability not yet built is not a spec mismatch: record an owned debt item; don't edit `epics.md`/`prd.md` to match code.

## Conventions

- Commits: `type(scope): Vietnamese sentence` stating what was found, not only what changed. Story id in `5-9` form, never `story-5.9` (machine-read by `git_evidence.py`).
- Markers, only at the start of a sentence: 🔴 unbreakable · ⚠️ trap · ✅ closed · 🟡 half-closed · 🔵 update · ⇒ conclusion. No new markers. `U+26D4` is banned repo-wide.
- "Origin" names four disjoint things (spine §Consistency Conventions): translation (AD-47) · Glossary entry (AD-36) · source document (AD-43) · dictionary citation (FR30). A bare `origin` identifier is banned (frontend `origin === 'user'` means a panel activation). Shapes differ: `segment.translation_origin`, `glossary_entry.term_origin`, `chapter.origin_author` · `origin_site_name` · `origin_url` · `origin_published_at`.
- Fixed vocabulary: Tác phẩm→`Work` · Chương→`Chapter` · Chế độ đọc→`ReadingMode` · Hán Việt→`HanViet` · `BaseLayer`/`DetachableLayer`; `Project`/`Book`/`Novel`/`Document` are banned for `Work`. Exactly eight exemptions, all naming the STORE rather than the entity: `.atproj` · `project.db` · `StoreKind::Project` · `ProjectStore` · `PROJECT_MIGRATIONS` · `commands/project/mod.rs` · `ports/project_store.rs` · `tests/project_contract.rs`. `src-tauri/tests/naming_boundary.rs` reads this line and matches it against `STORE_EXEMPT` item for item.
- A count states the scope it measured.

## Known pitfalls

- Silent emptiness is the central failure class: 0 rows, no error, "no results" for the user. Ask the `…HasLoaded` predicate and check the arguments you pass it. A value that can be unknown is `Option`/`NULL`, never `0` or `.unwrap_or(0)` (`library_work.status IS NULL` is the shape).
- Every write to `target_text` not coming from the typing buffer sets BOTH the comparison baseline and the origin column in the same operation (AD-47; sole exception FR101 restore). Generally, a write touches exactly the columns the user agreed to change.
- Segment boundaries are computed once at import and stored. Merging/splitting a SEGMENT = retire + create; merging/splitting a CHAPTER changes only `chapter_id` and `ord`.
- Fix the type so it tells the truth; don't lower a threshold, add `eslint-disable`, or extend an exclusion list to clear a red. Every exemption is named, carries its reason, and can die.
- Don't reuse a marker or pattern you haven't understood: grep its definition and its count first.
- A guard must be able to fail: a case calling the patched function directly guards itself, not the wiring; a source-scanning gate must read code lines, not commented ones; an assert that holds on both branches (symmetric fixtures) guards neither.

<!-- /bmad:context -->
