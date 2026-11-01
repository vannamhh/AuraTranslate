# Adversarial review of AD-64 (2026-11-01)

Scope read: AD-64 whole (Binds, Prevents 1-6, Rule 1-11, Chưa chốt), AD-11, AD-15 and its 🔵 line, AD-22, AD-60 header and Rule 1. AD-47 and the AD-60 marker rules were not re-read in full; where a finding leans on them it says so.

Verdict: AD-64 fixes the store, the gate and the write-only-complete rule, but it never defines the cache KEY, who OWNS a run, or which side checks the option, so two units can obey all 11 items and still ship incompatible code. Eight holes below; F1-F4 should block the first story.

## Pair A: Glossary queue story x URL block-tier story (both obey every item)

Story G (Glossary): `glossary_context_translation` takes the candidate's `context_example` (stored raw, cut at 200 characters, no ellipsis) and calls the provider with it. Cache read/write key: `context_example` as stored. It trims nothing because the string came from the DB.

Story U (URL block tier): takes a visible block's full text from the parsed page, trims it, NFC-normalises it (it already does this for the preview), and uses that as `source_text`. Batches up to `QUICK_BATCH_MAX_BLOCKS`.

Both satisfy Rule 3 ("một câu nguồn một hàng, `source_text` khoá chính"): Rule 3 never says what string `source_text` is.

Resulting breakage:

1. Same sentence, two rows. A 150-character sentence with a trailing space or a different Unicode form gets one row from G and one from U. The user pays twice and Rule 10(b) ("trúng bộ đệm thì 0 lời gọi, ở cả hai bề mặt") is green in each story's own test because each test seeds its own key shape.
2. Cut key hits the wrong row. A 600-character block seen by U is stored under its full text. The same sentence reached through G is stored under its first 200 characters. A different block whose first 200 characters are the same string (common on a site with a repeated boilerplate lead, or a quote that opens many paragraphs) is never matched by U; but if G's cut text is later equal to a short real block, U serves a translation of a HALF sentence as the full block's translation. This is exactly Prevents 5, entered through the input side instead of the output side. Rule 4 only guards the output (`maybe_truncated`), not an input that was cut before sending.
3. Rule 5's first step ("bản dịch có sẵn trong Tác phẩm") inherits the existing `substr(source_text,1,200)` equality match. Any segment sharing a 200-character prefix returns its `target_text` as the "translation" of the candidate's context. AD-64 re-blesses the lookup without stating the match rule.
4. Different prompts under one key. G sends a single sentence with the fixed single-sentence prompt; U sends a block inside a batch prompt (AD-60 markers). The `model` column is not in the key, and neither is "prompt kind". Row quality depends on which surface wrote first.

Tighten Rule 3: define `quick_key(text)` as ONE pure function in `core/quick_translation/` (trim, NFC, collapse nothing else), the only producer of `source_text`; both surfaces call it, `quick_translation` read and write take the already-keyed string and never the raw one. Add: a source longer than `QUICK_SOURCE_MAX_CHARS` is NOT sent and NOT cached (shown as "câu quá dài, không dịch nhanh") rather than being cut; G must read the full sentence from the segment, or refuse when only the 200-character `context_example` exists and it is exactly 200 long. Rule 5 step 1 must name its match: full-string equality of `quick_key(source_text)`, never a prefix. Add a column `lang` (source language) to the key if `quick_translation` can ever hold zh and en with equal text, and say whether the target language is fixed.
Add test 10(f): a fixture pair differing only by trailing whitespace and one 250-character sentence; assert one row and a refusal for the cut input.

## Pair B: Settings story x Cleanup story

Settings story owns the "tự dịch" option and the quick config; Cleanup story owns `Dọn`. Both satisfy Rule 8 and Rule 2.

5. Where the option lives is not stated (Rule 1 says "tuỳ chọn", Prevents 2 says two surfaces must not diverge). Settings stores it in `global.db` and exposes it to the webview; G and U each read it from the frontend store (test 10(a) is a frontend test: "mở hàng không gọi lệnh dịch"). Nobody checks it in Rust. Result: the command cannot tell an auto call from a `Dịch thử` click, so either it refuses the click when the option is off (breaks Rule 2 fallback) or it accepts any call (the option is a UI politeness, not a gate, and AD-15's "chỉ khi đủ hai điều kiện" is not enforced where the money leaves).
   Tighten Rule 1: the option and the config-present test are read by the Rust command from `global.db` at call time; the command takes an explicit `trigger: auto | manual`; `auto` with option off or quick config incomplete returns a typed refusal and makes no provider call. Test (a) must go through the real command, with the frontend flag forced stale-on.
6. Cleanup vs stale reply (Rule 8, test 10(e)). "Dọn huỷ lượt đang chạy trước" is cooperative cancel, but a frame or a final write already queued behind the single writer (AD-11) lands after `VACUUM` finishes. Test (e) says "không ghi hàng sau khi dọn" without a mechanism. Two stories will pick two: Cleanup bumps the generation, Settings/U check a token nobody told them about.
   Tighten Rule 8: every cache write carries the run's `generation` and the writer rejects it inside the serial queue when `generation != current`; `Dọn` bumps the generation as the first queued job, before the DELETE, so a stale write queued earlier is either already committed (and then deleted) or rejected. The test must interleave: write enqueued, Dọn enqueued, reply arrives afterwards.
   Also `Dọn câu cũ hơn N ngày` uses `created_at`: say whether a cache hit touches it (it does not, so the most-used sentence is first to be deleted), and whether `Dịch lại` resets it. Pick one (recommend: `Dịch lại` resets, hits do not, and say it, or add `last_used_at` and show both measurements to Ice per AGENTS.md).

## Run ownership, concurrency, `Dịch lại`

7. "Một lượt chạy tại một thời điểm" (Rule 7) has no semantics. When a block lot is in flight and G starts a call: reject, queue, or supersede? Story G will cancel-and-replace (it is the active overlay); story U will reject (its lot is expensive). Both are "one at a time". Worse, `QuickTranslateGeneration` is one counter shared by both surfaces, so closing the Glossary overlay ("đóng lớp phủ thì huỷ") cancels a URL-preview lot that the user is still watching, and vice versa; and test (d) only proves quick cancel does not touch `AiTranslateGeneration`.
   Tighten Rule 7: name the policy (recommend: a new call from the SAME surface supersedes; a call from a DIFFERENT surface is refused with `busy` and the surface shows the retry affordance; auto never supersedes manual). Cancel takes `(surface, run_id)`, not the bare counter. Add 10(g): two surfaces open, cancel from A leaves B's run alive.
8. `Dịch lại` racing a run. Row R is in a lot in flight; the user clicks `Dịch lại` on R. If it supersedes the lot, the lot's cancel discards 99 paid-for blocks (Rule 4 stores nothing from a cancelled run) for one row. If it waits, it is not "ghi đè đúng một hàng" at click time. If it runs concurrently, the lot's later write for R overwrites the manual result (last writer wins, stale content). Rule 6's "khối đang nằm trong một lượt chạy không được gửi lại" would also block the `Dịch lại` itself.
   Tighten Rule 3/6: `Dịch lại` is `manual`, bypasses the in-flight dedupe for its own key only, and its write is the only `REPLACE`; auto/lot writes are `INSERT OR IGNORE` so they can never overwrite a row. Add a monotonic `rev` or compare `created_at` so a lot that started before the click cannot clobber it. Write the answer to the 99-blocks case: complete the lot for blocks that already closed validly instead of discarding (Rule 4 allows "khối đóng đúng"; the cancel path should keep those).
9. TOCTOU double spend. Both surfaces read the cache (miss) before either call has written; with "one run" undefined (item 7), two calls for the same sentence both go out. Add an in-flight set keyed by `quick_key` shared across surfaces, not per run (Rule 6 covers only "khối trong một lượt").

## Cost leaks that obey the letter

10. Cancelled spend. Rule 4 stores nothing on cancel; Rule 7 cancels on every scroll/cursor move that follows a 300 ms stop. A user who scrolls with pauses of 400 ms launches a 100-block lot per pause, cancels it, and pays for every streamed token with no row kept. This is the cost Prevents 1 was written against, now at flash-lite prices. Tighten: keep every block that closed validly before cancel (Rule 4 already allows it by AD-60), cap the lot to the blocks currently visible, add a per-overlay-session ceiling `QUICK_AUTO_CALLS_PER_SESSION` with a visible counter, and put the debounce on "no change for 300 ms AND previous lot settled".
11. Failure retried by revisit. Rule 6 says a missing block shows a retry button and is not re-sent; nothing says the same for a Glossary row that errored, was refused (AD-59) or truncated. Moving off and back onto the row is a new "hàng đang mở", so auto fires again every time: a silent retry loop, against AD-22 (retry is the user's act). Tighten Rule 7: a failed or empty `source_text` keyed attempt sets a per-session "failed" mark that auto respects; only a click clears it.
12. Local provider with auto. "Cấu hình đã khai endpoint cùng model" admits an Ollama/LM Studio endpoint with no key. Cost is not money but the same server often runs the user's main batch (AD-60); `QuickTranslateGeneration` is deliberately independent of `AiTranslateGeneration` (Rule 7, test d), so auto debounced calls interleave with a 100-sentence batch and starve it or time it out. Tighten Rule 1: while `AiTranslateGeneration` is active and the quick endpoint host equals the main endpoint host, auto is suppressed (manual still allowed). Also say "local = no key required, empty keychain slot is valid", otherwise story S demands a key and story G does not.
13. Key fallback. Rule 2: quick key is its own keychain account, and a blank quick config falls back to the main config on `Dịch thử`. Not stated: if the quick slot has an endpoint but no key, does the code fall back to the MAIN key for the quick endpoint? That sends the main provider's secret to a host the user typed. Tighten: no cross-slot key fallback, ever; the fallback is all-or-nothing on the whole config (endpoint, model, key), and a half-filled quick config is "incomplete", not "blank".

## Config resolution with no Work open

14. Rule 2 says "tầng Global" for the fallback, but `prepare_context_reference_call` today resolves through `ScopeResolver` with an open Work (AD-18, Work overrides). With the Glossary queue open under a Work whose Work-layer AI config differs from Global, "Global" means a different model than the user's reading panel uses; with no Work open (URL tier) there is no Work layer at all. One story calls `ScopeResolver` (Work wins), the other reads Global directly; both read "Global" literally only if told so. Also: a manual fallback result with the BIG model is written to the cache (Rule 3 writes any complete answer; `model` stored but not in key) and then served as the "quick" answer for auto on both surfaces. Tighten Rule 2 and Rule 5: fallback reads `GlobalAiConfig` directly, never through `ScopeResolver`; state whether fallback results are cached (recommend yes, with `model` shown in the source label so the user sees it, since Rule 5 already says a label is shown); and the `model` label must be the one actually used, read from the response, not from config at display time.

## Option turned off during the 300 ms wait

15. Rule 7 cancels runs on "tắt tuỳ chọn", but during the wait there is no run: the debounce timer is in the webview. If story G cancels via `QuickTranslateGeneration` only, the timer survives and fires after the option is off. Combined with finding 5 (option not re-checked in Rust) the call goes out after the user disabled auto. Tighten: the debounce timer is part of the surface state and is cleared on option change; the Rust-side `auto` gate (finding 5) is the backstop; add test 10(h) that flips the option at 150 ms of a fake 300 ms timer and asserts 0 provider calls.

## Smaller points

- Rule 5 "tầng khối: bộ đệm, rồi gọi mạng" vs Rule 6 "gom khối chưa có trong bộ đệm": the cache read and the lot assembly must be one snapshot; a block cached by G between read and send is sent anyway (covered by finding 9).
- Source text in a fetched page can contain the AD-60 block marker; Rule 6 says "dùng dấu khối của AD-60" but not that the marker is expanded against the page text (AD-60 Prevents 6 names this). State it.
- Auto sends text of a page the user only fetched, to a third-party quick endpoint, without a click per page. AD-15's 🔵 and Rule 1 accept this; PRD §Kết nối mạng is deferred to John. Suggest the Settings option be per-surface or at least list both surfaces by name on the toggle.
- A finish_reason absent on some endpoints (AD-60 measured this with gemini-3.5-flash-lite) must count as `maybe_truncated` for single-sentence calls too; Rule 4 mentions the flag but does not define it for the Rule-5 single path.
- Test 10(c) "ảnh chụp không đổi sau một lượt chạy" proves no write to `segment`/`tm_unit`/`glossary_entry`, not that the translation text never reaches an editor path; fine as is, but add a grep guard that `core/quick_translation` imports nothing from `segment`/`tm`/`glossary` (AD-13 style).
- Missing from Rule 10: nothing counts the cases that actually reach the Rule 7 supersede/refuse branch; AGENTS.md "count the cases that reach it" applies.

## Items to tighten, summary

| AD-64 item | Change |
|---|---|
| 1 | Option and config check live in Rust; `trigger: auto\|manual`; local provider rules; suppress auto when main batch runs on the same host |
| 2 | Fallback reads Global directly, all-or-nothing, no cross-slot key use; define "incomplete" |
| 3 | `quick_key()` single producer, max length refuse-not-cut, `lang`, REPLACE only for `Dịch lại`, `created_at` semantics |
| 4 | Define `maybe_truncated` for the single path; keep blocks that closed validly on cancel |
| 5 | Step 1 match is full-string equality; label shows the model actually used |
| 6 | Cross-surface in-flight set; marker expansion against page text |
| 7 | Supersede/refuse policy, `(surface, run_id)` cancel, debounce timer cleared, failed-mark respected by auto, per-session ceiling |
| 8 | Generation checked inside the writer queue; Dọn bumps it first; hit/refresh effect on `created_at` |
| 10 | Add (f) key-shape, (g) cross-surface cancel, (h) option flip in the wait, interleaved cleanup, and the count of cases reaching each branch |
