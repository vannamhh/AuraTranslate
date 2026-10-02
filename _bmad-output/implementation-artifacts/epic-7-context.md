# Epic 7 Context: Translation Memory

<!-- Compiled from planning artifacts. Edit freely. Regenerate with compile-epic-context if planning docs change. -->

## Goal

Every segment the translator confirms silently becomes a (source, target) pair in a Translation Memory, with no manual step. Afterwards identical sentences are pre-filled (still unconfirmed), similar ones show a match percentage and diff, and Concordance answers "how did I translate this phrase before?" inside Panel Lookup. TM exports as TMX. Because the owner both translates and edits others' work, each pair carries an origin, and the Smart RAG Injector prefers the user's own pairs so the AI learns their style. Epic 7 is cut candidate #3 if R1 blows; the AD-6 boundary keeps it cleanly separable.

## Stories

- Story 7.1: Auto-write TM keyed by text pair
- Story 7.2: Origin on each TM pair
- Story 7.3: Dual-scope TM and two-key sort order
- Story 7.4: Exact 100% match
- Story 7.5: Fuzzy match
- Story 7.6: Language-specific matching algorithm
- Story 7.7: Concordance
- Story 7.8: Multiple translations for one source sentence
- Story 7.9: TM management
- Story 7.10: TMX export and import
- Story 7.11: Smart RAG prefers the user's own pairs

## Requirements & Constraints

- FR56 auto-write on confirm, FR57 dual scope, FR58 exact match, FR59 fuzzy, FR60 Concordance, FR61 language-aware matching, FR62 manage/filter, FR63 multiple translations, FR64 TMX, FR118 pair origin, FR70 (TM half closes here), plus FR44/FR129 (alt-text and caption pairs; Story 7.1 is their TM acceptance point, 6.13 covered only structure). NFR9 covers TMX.
- Confirming writes NO manual action and writes only to the Work-level TM; global TM receives data only via explicit action (TMX import or pushing a pair up).
- An empty target (`''`) writes no pair.
- Hard principle: the system never treats a segment as done on its own (pre-fill stays unconfirmed).

## Technical Decisions

- AD-6: a TM entry is `(source text, target text) + metadata`, fully independent of `segment.id`. Merge/split/re-split of segments leaves written pairs untouched. Editing a confirmed segment's target and re-confirming ADDS a new pair; never edits the old one. Local integer ids; retired ids never reused.
- AD-31 state machine: the pair is written exactly at the transition to confirmed, nowhere else. Editing a confirmed segment moves it back to unconfirmed (this is what triggers a new write on next confirm). Confirm creates one SegmentVersion; auto-save, TM pre-fill and Review Mode accept create none.
- Origin (FR117, three values plus `''`): written in the same operation as the pair. Target text differs from the text at segment load => "I translated"; identical => keep loaded origin (other-translated or bilingual-import). Compare text to text, never a dirty flag. 100% pre-fill takes the origin of the source TM pair. Value set is closed at three; any new value must declare which side of the binary axis (mine / other's) it falls on. Column naming: `glossary_entry.term_origin`-style; here `segment.translation_origin`; bare `origin` identifier is banned.
- AD-50 (built in 7.2): the origin comparison baseline is stored on `segment` (`baseline_target_text`, `baseline_translation_origin`), set only by non-user writes, never sent over IPC; `confirm_segment(segment_id)` arbitrates through one pure function in `core/segment/`. TM pair origin is the closed `core::tm::PairOrigin` with an exhaustive mine/others projection (AD-47 ⑥); `''` is unrepresentable in `tm_unit`.
- AD-18 sort: primary key = origin (mine first), secondary = scope (Work before Global). Applies to pre-fill choice (7.4), multi-translation listing (7.8, then date) and RAG.
- Scope resolution: TM query is a UNION of both tiers via `ScopeResolver`. Scope storage: Work tier in `project.db` (inside `.atproj`), Global in `global.db`; schema migrations are forward-only (never touch `.db` in git, AD-25).
- AD-17: TM matching uses the single shared `Matcher` (Chinese: exact + character n-gram, jieba-rs when needed; English: stemming then token n-gram). Same implementation as `dict/` and `glossary/`; no second copy.
- AD-51 (built in 7.5): fuzzy-match source diff is one pure fn `diff_spans(old, new, lang) -> Vec<DiffSpan>` in `core/matching` (`similar` =3.1.1; `diff_chars` for Chinese, `diff_words` + ≤2-char equal merge for English), text not offsets, trim+NFC both sides; shared with Epic 8 Diff Viewer. Accepting a fuzzy suggestion writes origin `other` (AD-47 ③ row).
- Module `tm/` depends on `matching`/`store`; `ai/` may read `tm/` (one direction, enforced by test). RAG injector is a pure function (source, scope, Glossary, TM) -> assembled prompt; Story 7.11 must not change the signature fixed in Story 4.6; non-user-origin pairs are inserted only after user pairs and marked in the prompt as reference style; deterministic output.
- AD-31 discrete actions (100% pre-fill, Review Mode accept) write immediately, bypassing the typing buffer, and create no SegmentVersion.
- Image alt-text and caption are ordinary `Segment`s with a role field (`alt`|`caption`); at most one per role per image; no empty segment is created. They join TM like any segment.
- Guard thinking from AGENTS.md: silent emptiness is the central failure class; a guard must be able to fail (counter-check by removing the seam).

## UX & Interaction Patterns

- 100% pre-fill: margin bar uses `tm-rule` (#b99a5e, a stroke color never a text color; use `tm-text` for text), surface `surface-tm`; labelled as suggestion; segment remains unconfirmed.
- Fuzzy match: a slide-up strip below the editing sentence pushes text down (does not overlay) and collapses when done; shows match %, diff between old and current source; accepting moves text into the Editor as unconfirmed; fully keyboard operable.
- Concordance lives in Panel Lookup beside dictionary results; shows source, target and origin per hit; registered command with bindable key. The dictionary empty state gains a pointer to Concordance (pays the promise deferred in Story 1.17); the Concordance empty state explains TM fills itself on confirm and there is no "add to TM" button; empty differs from "nothing looked up yet".
- TM management mockup: `ux-designs/ux-AuraTranslate-2026-08-02/mockups/tm-manage.html` (origin, multiple translations, TMX, TM health strip); filter by origin and by tier; edit/delete entries; keyboard accessible.
- TMX import: invalid file errors clearly and writes nothing partial; imported pairs get an origin distinguishable from confirm-generated ones; duplicates follow 7.8 (keep both).

## Cross-Story Dependencies

- 7.1 is the foundation: 7.2 (origin), 7.3 (scopes/sort), 7.4 (pre-fill), 7.8 (multi-translation) all build on the pair table; 7.5 and 7.7 depend on 7.6 (shared Matcher); 7.9 and 7.10 depend on origin and scope; 7.11 depends on 7.2/7.3 and on Epic 4's `RagInjector` (Story 4.6).
- Depends on Epic 2 confirm flow (AD-31, FR117 origin columns), Epic 6 Story 6.13 (alt/caption segments) and Epic 1 shared Matcher / Panel Lookup (Story 1.17).
- Closes the TM half of FR70 with Epic 4 (Glossary half already in Epic 3/4).
