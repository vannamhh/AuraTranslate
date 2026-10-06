---
type: handoff
title: "Story 11.2 — Task 0 re-read (HEAD 717196d, 2026-09-25)"
status: done
created: 2026-09-25
skill: bmad-build
---

# Story 11.2 — Task 0 re-read (HEAD 717196d, 2026-09-25)

Working notes for `spec-11-2-e2e-and-test-runner-debt.md` (Ice chose one spec, no lots). Not a spec. Line numbers are at `717196d`. Population: the 35 items whose last `Chủ:` is `Story 11.2`, as `check-debt-owner.mjs::parseItems` reports them (all open or 🟡).

Nightly `e2e (macos-26)` job: green 09-16→09-22, red on 09-23 (run 35919646999: `editor-typing-flush:218`, `story-3-5-review:143`) and on 09-24 (run 36059125872: `editor-typing-flush:218`). The `:218` cause is `505c8bc`, which added the lookup button inside `footer.status`. Job time is 4–6 min.

## Group A — nightly verdict and e2e harness
L10530, L4492, L4571, L5338, L5414, L10504, L7341, L4371, L4602, L4671, L4682, L10521, L10471, L10489, L4618, L4149 — dispositions are in the spec's Decisions.

## Group B — new WKWebView coverage
- L947 | valid: no e2e covers `Intl.Segmenter` on WKWebView | ✅ via new `e2e/specs/hanviet-segmenter-webkit.e2e.mjs` (3 propositions) | M-L
- L988 | changed shape: `hanVietCutAnchors.test.ts` mounts the real component. Double-click, drag and clipboard exist only in the stale HTML bench. B10 is resolved | fold into L947's spec
- L10212 | valid: no e2e for the five-column image alignment (FR42/43). Design settled by ticket #68 | ✅ new spec that asserts per-row top offsets of all `data-col` cells | M
- L5665 | valid: 0 e2e hits for glossary marks | ✅ new spec (mark render on `.hv-unit`, real `Selection.modify`, `<rt>` underline), or `Chủ: Epic 11` real-use pass | M
- L6202 | valid: no keyboard e2e for the Glossary import/export overlays | could share a spec with L5665 | S-M
- L7502 | valid: e2e cannot activate a `<button>` by keyboard. `e2e/AGENTS.md` records this as accepted | time-boxed spike on the WebDriver Actions key chain after `realClick`, or `KHÔNG LÀM` with the keyboard half moved to the `Epic 11` real-use pass | S-M
- L7612 | same root as L7502 (split_chapter shortcut) | same disposition as L7502
- L3847 | valid: `segment-merge-split` still fires a synthetic `MouseEvent` on the source column. `realClick` now exists (`e2e/support/pointer.mjs:49`), but nobody has retried it on the source column | first try `realClick`, otherwise `Chủ: Epic 11` real-use pass

## Group C — vitest/cargo runner and guards
- L10035 | valid: `asset_contract.rs` has 8 `TcpListener::bind` sites and no serialisation. It measured 12/19 red at the default thread count | Ice picks one: (A) move the net cases into an `#[ignore]` probe target (like `webimport_probe.rs`) — no flake, no continuous coverage; (B) `--test-threads=1` for this binary — 18/19, not fully fixed | M
- L10231 | same flake class (`segment_role_contract.rs`, `webimport_contract.rs`) | closes with L10035's fix, checked against those two files | S
- L8654 | open half valid: 4 files still wait on a real `setTimeout(0)` (`editorClearSourceCuts`, `editorTypingZone`, `glossaryHoverSelection`, `glossaryMarksRefresh`) | measure these 4 under load, then Ice decides between fake timers and accepting the risk | M
- L6610 | self-closed: `fileParallelism: false` (`b69a345`, 2026-09-05) | ✅ with a pointer to L8654 | S
- L7962 | self-closed by the same commit | ✅ with a pointer to L8654 | S
- L3493 | valid: `segmentHistoryTime.test.ts:139` `offsetMin === 0` has no TZ set | ✅ set `process.env.TZ` at the top of the file, before the import | S
- L2216 | self-satisfying: `tests/frontend/setup.ts:22-52` already meets the item's own bar | ✅ | S
- L2190 | valid but unbounded (retroactive coverage audit) | `KHÔNG LÀM`, reopen on a named regression | S
- L7900 | valid: the `nfr-bench` deadlock fix is unguarded, and CI never builds the feature | ✅ with a boundary-style source scan in `src-tauri/tests/` (runs always) that asserts the `"usable"` arm calls `spawn_phase_controller(`, or `KHÔNG LÀM` | S
- L3698 | valid: no gate on the `textAtLoad` ↔ `text_at_load` wire shape | Ice picks one: (a) regex script (no dependency); (b) `syn`-based Rust test (new dependency, NFR15) | L
- L5362 | valid: a WebKit limit that its own text ties to Story 3.4b, which is `done` | `KHÔNG LÀM` with the reopen condition stated | S
