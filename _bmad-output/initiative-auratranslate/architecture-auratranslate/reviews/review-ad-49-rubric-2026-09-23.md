# Rubric review — AD-49 (Không có ngăn xếp hoàn tác)

Reviewed: `ARCHITECTURE-SPINE.md:775-789` (AD-49), against `ad-brief-2026-08-17-mo-hinh-hoan-tac.md` §11,
`sprint-change-proposal-2026-08-18b-mo-hinh-hoan-tac.md`, and the referenced ADs (AD-1, AD-3, AD-5,
AD-31, AD-34, AD-35, AD-43, AD-47). `lint_spine.py --workspace <dir>` → `total_findings: 0`.

**Verdict: mostly sound and evidence-backed — path (C), scope, and the excluded-content list are
correctly ratified — but Rule ① is unenforced discipline in a codebase whose own AD-34 was written to
ban exactly that pattern, and the sign-off trail (`sprint-status.yaml` B6) has not caught up with the
spine.**

## Findings

### 1. HIGH — Rule ①'s "Mod+Z must yield the typing zone" has zero structural enforcement, in tension with AD-34's own charter
`keys.ts:510` — `if (lacksPrimaryMod(entry.mods) && isTypingZone(event.target)) return false` — is the
**only** typing-zone yield path in `handle()`, and it explicitly excludes chords that carry a primary
mod. So AD-49's own stated failure mode (Prevents ②: "a Mod+Z command registered for another reason
fires inside the translation box and its `preventDefault()` kills the browser's native `⌘Z`") is real
and not caught by the general dispatch mechanism. Today there is genuinely 0 risk (`grep -rn "KeyZ" src/`
= 0 hits, confirmed), but Rule ① gives future authors of Epic-3+ discrete-write commands (batch
glossary review, TM autofill, AI suggestions — the exact stated reason AD-49 must exist) a plain-English
obligation with **no test, no lint, no scanning gate** behind it (`scripts/check-commands.mjs` has no
KeyZ/primary-mod check; `grep -rln "CommandRegistry" scripts/` = only `check-commands.mjs`, which does
not cover this). This is precisely the shape AD-34 was written to eliminate — its own title is *"Sàn
khả năng tiếp cận là cấu trúc, không phải kỷ luật"* (a floor is structure, not discipline), and its
Prevents clause names this exact failure class ("chỉ lộ ra khi có người thử... và tới lúc đó đã quá
muộn để sửa rẻ"). AD-49 Binds AD-34 but its Rule ① does not inherit AD-34's structural guarantee — it
reverts to discipline for the one case (`Mod+Z`) where discipline is least trustworthy (a compiling,
green, silently-wrong change).
**Fix:** either (a) hard-code the carve-out structurally in `keys.ts::handle()` — treat `code === 'KeyZ'`
(with `Mod`/`Mod+Shift`) as always-yield-typing-zone regardless of `lacksPrimaryMod`, so no future
registration can violate it by omission — or (b) add a source-scanning case to
`scripts/check-commands.mjs` that fails if any registered binding maps `KeyZ`+primary-mod to a handler
that does not explicitly yield, and cite that gate in AD-49's Rule the way AD-45 cites `check-deps.mjs`
Kiểm 1b in the Consistency Conventions table.

### 2. MEDIUM — `sprint-status.yaml` B6 is stale against the spine as of the same date
`sprint-status.yaml:520-530` (epic 2, action "B6") still reads *"`AD-49` (mo hinh hoan tac) chua duoc
viet"* with `status: open`, and states its own completion condition: *"AD-49 co mat trong
ARCHITECTURE-SPINE.md, lint_spine.py sach."* Both conditions are now true (AD-49 is written; lint is
clean). `deferred-work.md` (same date, 2026-09-23) already closed the matching ledger item with
`→ ✅ ĐÃ ĐÓNG 2026-09-23 (AD-49)`, so two trackers dated the same day disagree about whether this work
is done. This is a "one fact, one place" drift, not a defect in the AD text itself.
**Fix:** flip B6 to `status: done` (or `review`, pending Ice's sign-off read) now that its own stated
exit condition is met; don't leave the spine and the sprint tracker telling different stories.

### 3. LOW — Rule ②(ii)'s inclusion of "xác nhận" (confirm) has no stated way-back, unlike its neighbor
Rule ②(ii) lists "gộp/tách segment theo AD-5" and "xác nhận theo AD-31" together as class (ii) (no byte
destroyed, no automatic inverse). For merge/split the "way back" is spelled out (re-invoke the command).
For confirm, no way-back is stated in AD-49's own text — it is only recoverable because AD-31's state
table already says *"Sửa văn bản của segment đã xác nhận → chưa xác nhận"* (editing a confirmed
segment's text auto-reverts it to unconfirmed; nothing needs an explicit "unconfirm" command). That
reasoning is correct and verified (`ARCHITECTURE-SPINE.md:378`), but AD-49 doesn't say it — a reader of
AD-49 alone, drafting a new Epic-3 discrete write and trying to classify it via this Rule, has to
already know AD-31's table to see why "confirm" doesn't need a Rule-①-style pair or a Rule-②(iii)-style
confirmation gate.
**Fix:** add a half-clause to ②(ii), e.g. "...xác nhận theo AD-31 (sửa văn bản sau xác nhận tự lùi
trạng thái — không cần lệnh bỏ-xác-nhận riêng)."

### 4. LOW (confirmatory, not a defect) — code-level claims in AD-49 check out
Spot-checked and correct: `editor.omit_segment`/`editor.restore_segment` (`Mod+Alt+X`/`Mod+Alt+R`,
`src/commands/index.ts:2850-2851`), `editor.end_target_paragraph`/`editor.join_target_paragraph`
(`Mod+Alt+P`/`Mod+Alt+U`, `:2883-2884`), `merge_chapter_into_previous`/`split_chapter_at_segment`/
`move_chapter` (`src/config/chapter.ts:293-295`, `src-tauri/src/commands/chapter.rs:642`) all exist as
named. `restore_segment_version` (`src-tauri/src/commands/segment.rs:733-880,3726-3757`) genuinely has
the `needs_confirmation`/`force` shape AD-49 cites for Rule ②(iii). The excluded-from-"user content"
list (chapter `title`/`origin_*` — AD-43) matches Ice's signed exclusion. Scope language ("tất cả")
matches the signed product-wide scope. AD-3/AD-5/AD-31 are correctly asserted unchanged — nothing in
AD-49's Rule touches their tables.

## Not flagged as findings, but noted while reading
- The brief/SCP call this decision "`AD-48`" throughout (that number was later given to the file-dialog
  AD, per `deferred-work.md`'s 2026-09-23 close note); the spine correctly uses AD-49. This is a
  historical-document naming artifact, not a spine defect — the evidence pointer in AD-49 still
  resolves to the right files.
- The genuinely open "⌘Z does nothing, silently" gap (`deferred-work.md`, "Deferred from: SCP
  2026-08-18b") is correctly left open with owner Ice and correctly *not* silently closed by AD-49 —
  AD-49 only codifies the constraint that would apply *if* Ice later picks the StatusBar-message option.
  That is the right scope boundary for an AD to hold.
