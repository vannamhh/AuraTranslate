# Review (rubric) — AD-58, 2026-10-25

Scope: spine lines 1139-1162 (AD-58), line 775 (AD-48 amendment), Capability map rows C1/C2; brief `ad-brief-keo-cua-so-thanh-tieu-de`.

Verdict: PASS with 5 fixes before handoff (no blocker; items 1, 2, 5, 6 have gaps).

## 1. Divergence points one level down — FAIL (minor)
Covered: window block in `tauri.conf.json`, `capabilities/main.json`, `<header class="titlebar">`, `config_invariants.rs`, `SECURITY-NOTES.md`, platform probe `detectIsMac`.
Missed, found by grep:
- `capabilities/main.json` field `description` says "Ba tập quyền … TỐI THIỂU"; stays stale at five. A builder editing only `permissions` leaves a lie in the file the AD binds.
- `SECURITY-NOTES.md` has the heading "ba tập quyền" (line 157) and "đúng BA quyền" (line 134); Rule 2 says only "thêm hai hàng".
- Rule 4 names no seam for the margin token (which file/token), so two units can pick different places.

## 2. Rules enforceable / Prevents covered — FAIL (partial)
- Prevents 1: PASS. `assert_eq!` on five strings in `config_invariants.rs`.
- Prevents 2: mostly PASS. Frontend test counts the attribute (=1) and bans `@tauri-apps/api/window`. Gap: nothing bans the raw string `plugin:window|` or `invoke('plugin:window…')`, though Rule 3 forbids it. Without the permission it would fail at runtime, but the test does not say so. Also Rule 3 says "`tabindex` ≥ 0"; `drag.js` blocks any `tabindex` other than `-1`. State the real rule.
- Prevents 3: FAIL. Rule 1 (`decorations: true`, `titleBarStyle`) has no test; no existing test in `src-tauri/tests` reads `titleBarStyle` or `decorations`. Prose only. Add an assertion on the window block in `config_invariants.rs` (named in the Rule).
- Prevents 4: FAIL. Rule 4 ("token, no number in component, macOS only") names no guard. Say whether `check:tokens` already catches a literal px, or name a test (platform-gated margin; the Windows case must be a case that can fail).

## 3. Brief requirements and choices — PASS
Landed: branch (a), double-click zoom, exactly two permissions (3 -> 5), `deep`, Windows unchanged, `decorations` kept, first-click limit (#4316) and `acceptFirstMouse` false, fullscreen margin kept, double-click ignores the macOS system setting, SECURITY-NOTES and assert message updates, AD-24 unchanged. Rejected options match brief §3 and A/B. No contradiction found. Not carried (acceptable, rationale): 28pt gain (§5).

## 4. No weakening of AD-1/24/29/48 — PASS
- AD-1: drag is a UI gesture, no business rule moves. AD-29 untouched.
- AD-24: the claim that `label` cannot reach another window holds only while there is one window; Rule 6 states this dependency. Good.
- AD-48: line 775 uses the `🔵 date: *old* thành *new* theo AD` shape of the other in-place edits, sits at sentence start inside Rule 2, keeps the Rust-only branch and the ban on `dialog:*`/`fs:*`. The heading text "ĐÚNG ba" is left as is, which is the convention. PASS.
- Capability map: AD-58 added to C1 and C2. PASS.

## 5. Decisions not prose; markers — FAIL (nit)
- Rule 6 writes "nhận một dòng 🔵" mid-sentence: the marker is used as a word. Reword to "nhận một dòng ghi chú cập nhật" (AGENTS.md: markers only at the start of a sentence).
- No placeholders; no new markers. Rules 2 and 3 are long but each clause is a decision; the Rule 5 reason clause for `acceptFirstMouse` is the only rationale and is short. OK.

## 6. Left to the implementing story — FAIL (gap)
- Rule 1 says `trafficLightPosition` is "đo trên máy thật" but the AD does not say it is a debt item, who owns it, or that Ice runs one real-app pass (brief §4: debt `Chủ:` of the implementing story). Add one sentence: story records the measured x,y and opens an owned debt item; Ice checks once in the real app.
- The first-click behaviour (#4316) is recorded as accepted but the brief says "chưa đo"; the AD states it as fact. Add "chưa đo" or put it in the same debt item.
- The new frontend test (Rule 3) has no stated file/target, so the builder must invent one; name it.

## Fix list (priority)
1. (medium) Rule 1: name the test that locks `decorations`/`titleBarStyle`/`hiddenTitle` in the window block.
2. (medium) Rule 4: name the guard for the macOS-only token margin.
3. (medium) Rule 1/5: state the trafficLightPosition debt item with owner, and the unmeasured first-click note.
4. (low) Rule 2: add `main.json` `description` and the SECURITY-NOTES heading and line 134 to the update list.
5. (low) Rule 6: remove mid-sentence 🔵; Rule 3: replace "tabindex >= 0" with "tabindex khác -1".
