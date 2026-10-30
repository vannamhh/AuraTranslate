# Adversarial review of AD-62 (2026-10-30)

Scope: AD-62 (architecture spine, lines 1388-1438), read against AD-34 §1, `src/commands/registry.ts` (`frozen`), `scripts/check-commands.mjs` (Kiểm A, `COMMAND_FLOOR`, Kiểm E load), `scripts/lib/commands-scan.mjs` (`scanVueAttrs`), `src/commands/index.ts` (`portMissing`, `installCommands`).

Verdict: the three-group floor fails closed (a command nobody classified is red, so it catches the plain "forgot the button" case). It does not guarantee what the title promises, "every function has a mouse point the user can reach". Three kinds of holes: (1) the mouse point is only a text match, not a reachable control; (2) the exemption and the debt ledger are cheaper to grow than to shrink; (3) several red conditions in Rule 4 have no real counter-check in Rule 6.

## Holes

### H1. Reachability is not checked: a mouse point inside a surface that only a keyboard-only command opens (HIGH)
Scenario: Story P adds `@click="dispatch('editor.merge_segments')"` inside `SegmentHistoryOverlay.vue` or any overlay. The overlay is opened by `history.open`, which is a 41-list debt id (keyboard chord only). Rule 1 and Rule 2 are satisfied to the letter: `editor.merge_segments` has a point, the gate is green, and no mouse user can ever reach it. Debt moves one hop deeper; it does not shrink. Rule 7 names this very cluster (Editor grid, URL import block tier), so it will happen.
Same family, all green on the gate:
- the control sits in a `v-if` / `v-show` branch whose condition never becomes true;
- the control is `:disabled` forever, or `pointer-events:none`/`hidden`;
- the component is never imported or never mounted (a dead `.vue` still counts, `scanVueAttrs` scans every `.vue` under `src/`);
- the `@click` is on a custom component that does not emit `click` (the attribute is on `<Foo>`, not a native element);
- a `dispatch` that is not the registry's (local function named `dispatch`) - Kiểm A does not check the import.
Smallest tightening: add to Rule 5 "closing a debt id and adding a `@click` for a new command both require one test that mounts from the app root (or e2e from cold start), clicks the real control, and asserts `registry.dispatch(<id>)` ran; the test file is named in the commit". Cost: about 41 + every new command gets a mount-and-click test; the gate itself stays static. A gate-only version (reachability graph over `v-if` and overlay-open commands) is expensive and brittle; prefer the test.
Also state in Rule 1 as a limit (like the existing ⚠️ on `reason`): "the gate proves the text exists, not that it renders".

### H2. Exemption laundering: removing a button turns into a one-line `pointer` (HIGH)
Scenario: Story Q refactors `LibraryMode.vue`, drops the "next work" button and its `dispatch('library.work_next')`. Gate goes red for that id. Rule 5 forbids new ids in the debt list, so the only green path is declaring `pointer: { kind: 'aim', reason: '...' }` on the register line. That is a one-line edit in `index.ts`, no ceiling, no count, and reviewers see a diff of 1 line. Rule 3 says the gate cannot check `reason`; `aim`/`focus`/`gesture` are exactly the unverifiable kinds (10+9+4 = 23 of 29 at baseline).
Two independent stories also disagree on kind freely: a drag-reorder fallback `chapter.move_up` can be `gesture` (replaces the drag) or `aim` (steps a list) and both are "true enough".
Prevents #3 ("each story decides on its own criterion") is therefore only partially closed: the criterion is now a closed set of words, but the decision is still per story and unverified.
Smallest tightening: per-kind exemption ceilings next to `POINTER_DEBT_CEILING` (`POINTER_EXEMPT_CEILING = { focus: 9, aim: 10, gesture: 4, auto: 4, via: 2 }`), gate red when a kind count exceeds its ceiling. A new exemption then needs a visible second edit in `scripts/check-commands.mjs` that Ice/Winston see in the diff. Cost: friction on legitimate new `focus`/`aim` commands (one number to bump, with a stated reason in the commit). It does not stop a determined lie, only a silent one.
Also add: a command that LOSES its last point while not exempt must be fixed by restoring the point or deleting the command, not by a new exemption - state it in Rule 3 (review rule; the gate cannot see history).

### H3. The debt ceiling is slack, swappable and editable in the same commit (HIGH)
- Rule 5: "mảng dài hơn trần là đỏ". Only length > ceiling is red. If Story P removes 5 ids and forgets to lower the ceiling, the gate stays green with 5 free slots. Story R later adds 5 new ids (new commands, missing buttons) and is also green. Rule 5 says "Story vá hạ trần trong cùng commit", but that is discipline, which is what AD-34 exists to replace.
- Swap: remove one fixed id and add one new unclassified id in the same commit. Length equals the ceiling, gate green. "Chỉ id đỏ ở baseline được vào sổ" has no machine form: the gate has no baseline set, so a brand-new command can enter the debt list undetected.
- Raise: `POINTER_DEBT_CEILING` and `POINTER_DEBT` live in the same file; a story that needs green raises both. Nothing in `pre-push` compares them with `HEAD`.
- Duplicates: the same id twice inflates length (harmless but allows slack).
Smallest tightening: (i) red unless `POINTER_DEBT.length === POINTER_DEBT_CEILING` and no duplicates; (ii) freeze the baseline as a second constant `POINTER_DEBT_BASELINE` (the 41 ids, never edited after the gate story) and red when `POINTER_DEBT` contains an id outside it - then adding costs a visible edit to a constant labelled "never edit". Cost: one more 41-line array, and the baseline constant can still be edited by a determined author (same weakness as any in-file list); only a `git diff` check in `pre-push` closes that, which needs a decision from Ice (it reads history, not the tree).

### H4. Two sources of truth for debt: `POINTER_DEBT` vs `deferred-work.md` (MEDIUM)
Rule 5: ownership is "một mục mỗi bề mặt" in the ledger, but `check:debt-owner` links an item to a story, never to ids in the array, and Rule 4 says the gate does not read `_bmad-output`. Scenario: Story 7.2 closes its ledger item with "✅ ĐÃ ĐÓNG" while two of its ids stay in `POINTER_DEBT`; or a surface's item names Story 7.2 and 7.2 is `done` leaving ids with no live owner. The ledger and the array disagree and both gates are green.
Smallest tightening: make the array a map `id -> '<ledger anchor>'` and have `check:debt-owner` (which already reads the ledger) assert that every anchor exists and its last `Chủ:` is not a done story. Cost: `check:debt-owner` learns to read `check-commands.mjs`, or a small JSON shared file. Alternative: each ledger item lists its ids and `check:debt-owner` greps for them in the array.

### H5. `auto` is satisfied by any `dispatch('<id>')` literal, including keyboard paths and dead code (HIGH)
Rule 3 `auto`: "ít nhất một `dispatch('<id>')` literal trong mã `src/` ngoài template và ngoài `src/commands/index.ts`". A story whose command has a keydown handler (`@keydown`/`document.addEventListener('keydown', ... dispatch('x'))`) or a leftover call in a helper that nobody calls satisfies `auto` with a keyboard-only or never-run path. In the repo, `src/commands/keys.ts` and `src/main.ts` both contain such literals. `auto` is also the only kind whose truth is a machine-checkable claim, so a weak check is a worse hole than for `focus`.
Smallest tightening: `auto` carries `at: '<path>'` (file that holds the dispatch) and the gate requires the literal in THAT file, outside comments; `at` must be one of a short closed list (`src/main.ts`, `src/App.vue`, `src/library/*`) and require the reason to name the mouse event. Cost: one extra field, still only checks presence not that the event fires. Also exclude test files and `.d.ts` from the scan (state it).

### H6. `via` has no semantic check and `reason` is free text (MEDIUM)
`via: 'settings.open'` on any command whose chord opens something under Settings passes: the target has a point, does not itself declare `pointer`, no chain. The gate does not verify "cùng kết quả". Two stories can both point `via` at `mode.library` for unrelated functions. Chains are indeed blocked (target must not declare `pointer`), but a `via` whose target point is itself inside an unreachable surface (H1) inherits H1.
Smallest tightening: `via` allowed only for ids whose label stem is the target's (`<domain>.open` -> same domain), or cap `via` count (see H2 ceilings). Cost: hand-written rule, may reject legitimate cases (shortcuts.open via settings.open is cross-domain by design); the per-kind ceiling is the cheaper answer.

### H7. Right-click, double-click and modified clicks are "mouse points" but not discoverable (MEDIUM)
Rule 1 counts `@dblclick`, `@contextmenu`, `@submit`, and `@click` with any modifier (`@click.right`, `@click.middle`, `@click.ctrl`). A function reachable only by a double-click on a row passes. Prevents #1 ("người không nhớ phím không biết chức năng có tồn tại") is the stated harm, and a hidden gesture has the same harm for a mouse-only user. Also, only `@click` is held to dispatch-only by Kiểm A; `@dblclick`/`@contextmenu` are counted only when they are the literal, so a half-converted site still exists as unclassified behaviour (Kiểm K covers some).
Smallest tightening: for the floor, count a command only if at least one point is `@click` (no `.right`/`.middle`/`.ctrl`/`.meta`/`.alt` modifier) or `@submit`; `@dblclick`/`@contextmenu` give a secondary point and do not satisfy the floor alone. Cost: some existing commands that satisfy the floor today only by `@dblclick` fall to the debt/exempt count; measure how many at baseline before adopting (this is a "two valid options, present measurements" case for Ice).

### H8. Registration the gate cannot see (LOW-MEDIUM)
Kiểm E loads `installCommands({ setMode: () => {}, isMac: true })` and reads `list()`. A `register()` placed inside `if (!isMac)`, a deps branch, or called later from a component (`mount`) is never in the list, so it is never classified. Today every command registers unconditionally (ports use `portMissing`, which logs and returns) - an unwritten convention, not a rule. Related: a button can dispatch a command whose `run` is `portMissing` because `main.ts` never injected the port; the mouse point exists and does nothing (the gate cannot see wiring; Kiểm E only counts).
Smallest tightening: add to Rule 3/AD text "register() is unconditional and only inside `installCommands`", enforced by a grep gate line in Kiểm M (count `register(` call sites outside that function = 0). Cost: tiny.

### H9. `registry.ts` `frozen()` copies fields by name (LOW, implementation trap)
`frozen()` builds a new object from `id`, `labelKey`, `run`, `keys`, `repeatable`. Adding `pointer?` to the type without adding it to `frozen()` makes every declared exemption disappear at `list()` ("trường `pointer` đọc lúc chạy"). The result is 29 reds, not a silent pass, so it is fail-closed, but the first run looks like a mass regression. Add to Binds: `frozen()` and the `Registry.list()` doc; and a unit test that `list()` returns the `pointer` that was registered.

### H10. User-rebound chords (LOW)
Not a hole in the mouse floor, but Prevents #2 says a command with no default chord is "chỉ tới được khi người dùng tự gán phím". Rule 3 `focus`/`aim`/`gesture` reasons assume the keyboard default exists; a user who unbinds `editor.next_segment` has no path if the exemption was "chỉ dời focus". That is by design (exemption means no independent function), but `auto` + user-unbound has the same property. No tightening needed; record it as a stated limit.

## Can each "Prevents" item go red?
1. Key-only function: yes. (a) is a real removal and the red names `mode.reading`.
2. No chord, no caller: yes, same path (group rule).
3. Per-story criteria: only partly; kinds `focus`/`aim`/`gesture` cannot go red (Rule 3 ⚠️). H2 ceilings give the only machine signal.
4. Exemption outlives its reason: yes for "has a point + declares" (b), `auto` loses its dispatch, `via` target loses its point; a deleted command takes its exemption with it (vacuous, fine). Not for a `focus` whose target stopped being clickable.
5. Debt as dumping ground: only for growth past the ceiling; swap and slack are green (H3).
6. Green on an empty tree: yes, but note the floor is `ceil(0.85 x live)` (about 220 of 259), so it only catches a collapse of the scan, not a loss of 39 points; those lose individually through the group rule anyway, which is the actual guard.

## Rule 6 counter-checks
- (a) real removal of a production attribute, keeps the button, same call shape; the red is for `mode.reading` having no group. Good. Verify `mode.reading` has exactly one point (grep: one `dispatch('mode.reading')`, App.vue:232) - yes at baseline.
- (b) is an ADDITION (declare `pointer` on a command that has a point), not a removal; AGENTS.md demands removal. Acceptable as a mutation but record it as such, or use the removal form: delete the `pointer` of an `auto` command's dispatch literal instead.
- (c) first half is a real removal (id from `POINTER_DEBT` without button: red names the id). Second half (add button for a still-listed id) is an addition; fine.
- (d) edits `via`; real, OK.
- (e) runs on a FIXTURE: per the repo pitfall ("a case calling the patched function directly guards itself, not the wiring"), it guards `scanVueAttrs`/the classifier, not that Kiểm M's `fail()` reaches the exit code. (a)-(d) cover wiring on the real tree, so keep both; but the list of Rule 4 red conditions has no counter-check for: ceiling exceeded, `kind` outside the set, empty `reason`, `auto` without a dispatch literal, debt id no longer registered, floor too low. Add one real-removal case each (e.g. remove the only `dispatch('library.list_works')` literal for `auto`; append a registered-nowhere id to `POINTER_DEBT`; push one extra id past the ceiling). Cost: about six small cases.

## Two-story composition (summary)
- Story A (new command `x.y`) declares `pointer: { kind: 'aim' }`; Story B (fix debt) lowers nothing. Both obey every Rule; the exemption count grows by one with zero friction (H2), the ceiling keeps slack (H3).
- Gate story G lands the floor; feature story F adds the button inside an overlay opened only by a debt command. Green; mouse user blocked (H1).
- Story fixing debt closes its ledger item, ids remain in the array (H4).

## Priority
H1, H2, H3, H5 before signing; H4, H6, H7 can be explicit limits plus a debt item; H8-H10 are one-line text additions.
