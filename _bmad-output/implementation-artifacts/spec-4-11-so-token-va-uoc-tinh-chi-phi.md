---
title: 'Story 4.11 — Token counts and cost estimate'
type: 'feature' # feature | bugfix | refactor | chore
created: '2026-09-22'
status: 'done' # draft | ready-for-dev | in-progress | in-review | done
route: 'dispatch' # oneshot | dispatch
review_loop_iteration: 0
baseline_commit: 'afbe740e6f8b264de4bed5b5367a27e694e539a0'
context:
  - '{project-root}/_bmad-output/implementation-artifacts/epic-4-context.md'
  - '{project-root}/AGENTS.md'
  - '{project-root}/src-tauri/AGENTS.md'
  - '{project-root}/src/AGENTS.md'
---

<frozen-after-approval reason="human-owned intent — do not modify unless human renegotiates">

## Intent

**Problem:** With BYOK every call spends the translator's own money, and nothing on screen says
how much. The provider's `usage` object already reaches this tree and is thrown away — the chunk
struct's own doc-comment says so (`core/ai/client.rs:204-205`) — while no request ever asks for it
(`stream_options` and `include_usage`: zero matches in `src-tauri/src`). No symbol anywhere in
`src/` or `src-tauri/src` counts tokens or money.

**Approach:** Ask the provider for usage, carry it through the port and both channels that already
exist, and render it as a `vi.json` key plus params. The money figure comes from a dated price
table bundled with the app and keyed by model id; it is never fetched and never guessed.

## Boundaries & Constraints

**Always:**

- Every displayed sentence is a `vi.json` key plus params. Pure functions return a key and params;
  the webview never composes a Vietnamese sentence and never formats a numeral through
  `Intl.NumberFormat`. This is the shape `lookupHistoryState.ts:249-256` already chose, for NFR16,
  under gate `check-i18n.mjs` Check A2.
- Numbers are written as numbers: `"412 token · ước tính ~0,004 USD"` — UX-DR47 and
  `EXPERIENCE.md:58`. Never rounded into words, never hidden behind a click.
- **Decision (Ice, 2026-09-22): the price table is bundled, keyed by model id.** A money figure
  appears only when the model id that actually ran has a row in that table. A model with no row —
  a local Ollama/LM Studio model, or any id the table has not been taught — shows its token count
  and no money at all. This *is* the local-model rule: the app has no local/cloud discriminator
  (`provider` is free text, `mod.rs:203-212`) and FR66 deliberately gives both the same config
  path, so absence from the table is the only signal, and no `localhost` sniffing is introduced.
- When a figure is absent the line says so — never `0`, never an empty currency.
- The cost is an estimate and the screen says so; it never presents itself as the provider's
  invoice (mockup `ai-batch-translate.html:266`).
- Exactly three network egress points app-wide (AD-15, `SPEC.md:69`). Pricing data is never
  fetched, so no fourth point is opened.

**Never:**

- No new `#[tauri::command]`. The `config_invariants.rs` census row `3, 0, 0` (`:1517-1520`) and the
  tree total `(71, 28)` (`:1731`) must be unchanged by this story.
- No tokenizer, no local counting, no estimate of our own. Every token number on screen came from
  the provider or is not shown.
- No price fields in `ai_config` and no sixth `AiConfigField` variant — the table is the only
  source of prices, so the configuration surface does not grow and needs no migration.
- No persistence: no new table, no migration, no accumulation that survives a panel reset.
- Not this story: the session-cumulative status-bar figure and the per-task breakdown table the
  mockup draws (`:192`, `:261-279`); and the Prompt Inspector "sửa ngay" action the ledger assigns
  to 4.11 at `deferred-work.md:10873`, which is a separate deliverable on a different screen.

## I/O & Edge-Case Matrix

| Scenario | Input / State | Expected Output / Behavior | Error Handling |
|----------|--------------|---------------------------|----------------|
| Usage arrives, model priced | final chunk carries `usage`, model id has a table row | `"<in+out> token · ước tính ~<x> USD"` under the result | N/A |
| Usage arrives, model not in table | local model, or an id the table has not been taught | token count alone, no currency, not an error | N/A |
| Model id collides with a table row | a local or proxied model answering to a priced id | a money figure is shown for a call that cost nothing | accepted limitation of a name-keyed table; named here so it is not discovered as a surprise |
| Provider sends no usage | stream ends, no `usage` seen | the line says the provider returned no figure | never `0` |
| Batch completes | per-sentence usage summed | per-row token count plus a batch total | N/A |
| Batch partly without usage | some sentences report usage, some do not | total is shown and states it covers only the sentences that reported | never silently sum a subset as if whole |
| Cancelled mid-flight | `Cancelled` before any usage frame | no figure for that call; already-streamed text stays | no fabricated number |
| Provider error | any of the six cause families | no usage line at all | 4.10's error surface unchanged |

</frozen-after-approval>

## Code Map

**Rust — the number does not exist yet anywhere on this path**

- `core/ai/client.rs:171-182` `ChatCompletionsRequestBody`, five fields, `stream: true` hardcoded
  at `:199`; `build_request_body` `:194-202` is the single seam a test can assert without opening a
  port. `stream_options` must be added here.
- `:204-222` `ChatCompletionsChunk` / `ChunkChoice` / `ChunkDelta` — minimal shapes; the
  doc-comment at `:204-205` states `usage` is dropped by serde today. A usage-bearing final chunk
  has `choices: []`, so it reaches `parse_chunk_content` (`:363-367`), finds no `delta.content`,
  and falls through to `SseEventOutcome::Ignore` (`:356`) — silently. That fall-through is the bug
  to fix, not just a missing field.
- `:336-345` `SseEventOutcome { Ignore, Done, Token }`; `Done` is matched on the literal `"[DONE]"`
  string (`:351-352`) and carries nothing.
- `ports/translation_provider.rs:102-110` `TranslateOutcome { Done, Cancelled }` — the terminal
  value that must carry the usage. ⚠️ Declare the usage type **here**, not in `core::ai::client`:
  `ai_boundary.rs:399` `ALLOWED_AI_CLIENT_NAMES_IN_COMMAND_SEAM: [&str; 2]` breaks on a third
  `core::ai::client::` name used in the command file, and its length is in the type.
- `commands/aitranslate.rs:615`/`:737` the single-run wire is `tauri::ipc::Channel<String>` — bare
  strings, **no structured event to attach a number to**. `:699-709` `AiTranslateOutcomeWire`
  (`#[serde(tag = "state")]`, `Copy`) is the command's return value and the natural carrier for the
  single-run figure. `:514-520` `AiTranslateBatchEventWire::Done { segment_id }` is the batch
  carrier. The two paths are shaped differently; do not try to make them one.
- The price table is new and has no home yet. Put it in `core/ai/pricing.rs` as a pure module: a
  dated const table keyed by model id plus a pure `(model, usage) -> Option<estimate>`. It is
  AI-domain, so it sits inside the `core/ai/` boundary (AD-13); `ai_boundary.rs:65` `AI_FLOOR = 1`
  is a lower bound, so a second file there cannot redden it. Do **not** widen
  `ALLOWED_AI_CLIENT_NAMES_IN_COMMAND_SEAM` (`:399`) — that array governs `core::ai::client`, and
  pricing is a different module.
- The model id that must key the lookup is already carried: `resolved_field_value(..., Model)` at
  `commands/aitranslate.rs:254` (single) and `:374` (batch), into `PreparedTranslateCall.model`
  (`:191`) and on to `TranslateRequest.model` (`ports/translation_provider.rs:84`). Read it there;
  do not re-resolve config at display time.
- `core/aiconfig/` is **untouched** by this story — no sixth field, no migration (§Never).
- `core/i18n/mod.rs` `message_keys!` — new keys declare exactly the params their sentence
  interpolates; `ipc_contract.rs:325` compares both directions per key, and a missing required
  param downgrades `message_key` to `Unknown` in release, silently.

**Webview — where it lands**

- `src/config/aitranslate.ts:35-38` `AiTranslateOutcomeWire` and `:119-122`
  `AiTranslateBatchEventWire` are the TS mirrors, each with a runtime shape check (`:64-68`,
  `:130-136`) that must learn the new field. Adapters never throw.
- `src/aiTranslateState.ts:41-46` the module-level refs and `:155-163` `resetAiTranslate` — a new
  ref must be assigned inside that same reset or `check-panel-refs.mjs` fails.
  `src/aiTranslateBatchState.ts:62-66` the row shape, `:91-106` the existing computed aggregates
  (the pattern a token total follows), `:260-268` its reset.
- `src/panels/AiTranslationPanel.vue:400-403` the streamed-text block (single-run attach point) and
  `:425-434` `data-ai-translate-batch-progress` (batch attach point). Every status line carries a
  `data-*` hook; a new one must too.
- ⚠️ No number formatting exists in `src/`: `Intl.NumberFormat`, `toLocaleString` and `toFixed` are
  zero matches there (the one `toFixed` in the tree is a benchmark print,
  `tests/frontend/importPreviewChapters.test.ts:1368`), and `vi.json` contains no decimal literal
  to copy a convention from. The decimal comma is produced as a **param**, by a pure function.

**Gates that will fire**

- `check-i18n.mjs` Check A2 (no UI sentence outside `vi.json`), Check C (placeholders match the
  params actually passed), Check D (bans `bạn`/`chúng tôi`).
- `check-panel-refs.mjs` — module-level cell must be reset in the same file's `reset*()`.
- `check-commands.mjs` — only if a command id is added; `COMMAND_FLOOR = 52`, `CLICK_FLOOR = 27`,
  `DISPATCH_FLOOR = 40` are floors that must be **raised** when legitimately exceeded.
- `config_invariants.rs:1517-1520`, `:1731` — must not move (see §Never).

**Tests that move**

`src-tauri/tests/ai_translate_contract.rs` · `src-tauri/tests/ipc_contract.rs` ·
`tests/frontend/aiTranslate.test.ts` · `tests/frontend/aiTranslateBatch.test.ts`. The frontend
harnesses already drive a fake Channel: `pendingRun()` (`aiTranslate.test.ts:106-127`) and
`pendingBatchRun()` (`aiTranslateBatch.test.ts:120-137`) — extend them, do not rebuild them.

## Tasks & Acceptance

**Execution:**

- [x] `src-tauri/src/core/ai/client.rs` -- ask for usage via `stream_options`, parse it off the
      final chunk, and stop `SseEventOutcome::Ignore` from swallowing a usage-only frame -- a chunk
      with `choices: []` takes exactly that path today, so adding the struct field alone changes
      nothing observable.
- [x] `src-tauri/src/ports/translation_provider.rs` -- declare the usage type and let
      `TranslateOutcome::Done` carry `Option<...>` -- declared here, not in `core::ai::client`, so
      `ai_boundary.rs:399`'s two-name array stays as it is.
- [x] `src-tauri/src/core/ai/pricing.rs` (new) -- a dated const price table keyed by exact model
      id, and a pure `(model_id, usage) -> Option<estimate>` returning `None` for any id not in the
      table -- source each price from that provider's own published page at implementation time and
      record the date in a `PRICES_AS_OF` const beside the table; seed at least the ids this repo
      already names (`claude-sonnet-5`) and leave local ids (`qwen2.5:14b`, `llama3`) deliberately
      absent, which is what makes them show no money.
- [x] `src-tauri/src/commands/aitranslate.rs` -- carry the figure out on both wires:
      `AiTranslateOutcomeWire` for the single run, `AiTranslateBatchEventWire::Done` per sentence --
      the single-run channel is `Channel<String>` and has nowhere to put it, which is why the
      return value carries it.
- [x] `src-tauri/src/core/i18n/mod.rs` + `src/i18n/vi.json` -- one key per line the panel can show
      (figure with cost, figure without cost, no figure from provider, batch total, partial batch
      total), each declaring exactly the params its sentence uses -- `ipc_contract.rs:325` checks
      both directions per key and Check D bans `bạn`.
- [x] `src/config/aitranslate.ts` -- extend both wire types and both runtime shape checks -- an
      adapter that silently drops an unknown field would make every frontend test pass against a
      backend that sends nothing.
- [x] `src/aiTranslateState.ts` + `src/aiTranslateBatchState.ts` -- hold the single-run figure and
      derive the batch total as a computed beside the existing aggregates; reset both in the
      existing `reset*()` -- `check-panel-refs.mjs` fails on a module cell missing from its reset.
- [x] `src/panels/AiTranslationPanel.vue` -- a pure function returns key + params; the template
      renders `t(key, params)` with a `data-*` hook on each new line -- no numeral formatted in the
      webview (NFR16, and the `Intl.RelativeTimeFormat` refusal at `lookupHistoryState.ts:249-256`
      is the precedent).
- [x] `src-tauri/tests/ai_translate_contract.rs` -- one case per I/O-matrix row, including a
      usage-only final frame, a stream that ends with no usage at all, and a cancel before usage --
      then counter-check by REMOVING the usage plumbing and confirming the new cases go red for the
      right reason, not merely red.
- [x] `tests/frontend/aiTranslate.test.ts` + `tests/frontend/aiTranslateBatch.test.ts` -- assert the
      rendered string for: figure with cost, figure without a configured price, no figure from the
      provider, and a batch total that covers only the reporting sentences -- assert the text, not
      that a call happened.
- [x] `_bmad-output/implementation-artifacts/deferred-work.md` -- re-defer the Prompt Inspector
      "sửa ngay" item (`:10873`) with a named owner, and file the two mockup surfaces this story
      defers by Ice's 2026-09-22 scope decision — the status-bar cumulative figure (`:192`) and the
      per-task breakdown table (`:261-279`) -- cite §SECTION NAME, never a line number.

**Acceptance Criteria:**

- Given a completed call whose provider reported usage, when the panel updates, then the exact
  token count is on screen and no part of it is expressed in words.
- Given a provider that reported no usage, when the call completes, then the line says there is no
  figure, and no `0` and no currency symbol appear anywhere on that line.
- Given a model with no configured price, when a figure is shown, then the token count appears and
  no money is shown at all — and this is not rendered as an error state.
- Given a batch in which some sentences reported usage and some did not, when it finishes, then the
  total names how many sentences it covers rather than presenting a partial sum as complete.
- Given `cargo test --locked` on this story's baseline commit in a still tree, when the counts are
  compared after the change, then the census row `3, 0, 0` and the tree total `(71, 28)` are
  unchanged, because this story adds no `#[tauri::command]`.
- Given a model id absent from the price table, when its call completes, then the token count is
  shown and no currency appears anywhere on the line — and the panel does not treat it as an error.
- Given the price table, when it is read by a test, then every row states the date its prices were
  taken, so a stale row is visible rather than silently believed.
- Given the whole AI configuration removed, when the prior epics' features are exercised, then they
  still work in full — AD-13's boundary test still passes unchanged.

## Implementation Notes

**First pass (2026-09-22).** Implemented the whole story: `TranslateUsage` declared in
`ports/translation_provider.rs` (not `core::ai::client`, so `ai_boundary.rs`'s
`ALLOWED_AI_CLIENT_NAMES_IN_COMMAND_SEAM: [&str; 2]` never needed widening); `core/ai/client.rs`
now sends `stream_options: { include_usage: true }` and reads a usage-bearing final chunk that
previously fell into `SseEventOutcome::Ignore` silently (fixed and counter-checked by removing
the fix and watching the new test go red for that exact reason, then restoring); `core/ai/pricing.rs`
(new) holds a bundled, dated price table (`claude-sonnet-5`, sourced live from
`https://claude.com/pricing` on 2026-09-22: $2/$10 per million input/output tokens) and a pure
`estimate_cost_usd`; both wires (`AiTranslateOutcomeWire::Done`, `AiTranslateBatchEventWire::Done`)
carry `AiTranslateUsageWire`; five `vi.json`/`MessageKey` pairs render the single-run line;
`AiTranslateBatchUsageSummary`/`aiTranslateUsageLine`/`aiTranslateBatchUsageLine` on the webview
side select the key as a pure function, `formatUsdParam` produces the decimal-comma param without
`Intl.NumberFormat`. Full verification (`cargo test --locked`, `npx vitest run`, `npm run build`,
all 11 pre-push gates individually) was green.

**Correction round (2026-09-22, same day) — three gaps found against the diff, not the report:**
1. **Unmet signed AC** — `epics.md:3811-3813` signs *"hiển thị tổng token và tổng ước tính của cả
   lô"*; the batch total lines (`ai.translate.batch_usage_total`/`_partial`) shipped in the first
   pass showed tokens only, no cost. Fixed: `aiTranslateBatchUsageSummary` (`aiTranslateBatchState.ts`)
   now also returns `costUsd: number | null` — the sum of `cost_usd` across reporting rows, but
   `null` (not an understated sum) unless *every* reporting row carries a cost, since one batch
   always resolves to one model and therefore one pricing outcome for all its rows; a defensive
   `every(...)` check guards against a hypothetical mixed state rather than assuming the invariant
   holds silently. Two new keys added, `ai.translate.batch_usage_total_with_cost` and
   `..._partial_with_cost` (registered in both `vi.json` and `core/i18n/mod.rs`, same rationale as
   the original five), mirroring the single-run "one key with cost, one without" shape. New frontend
   test cases cover full+priced, full+unpriced, partial+priced, and the mixed-cost defensive fallback.
2. **Matrix row "Provider error" had no test** — added
   `tests/frontend/aiTranslate.test.ts`'s *"error: KHÔNG một dòng usage nào"*, settling with
   `STREAM_ENDED_ERROR` and asserting `[data-ai-translate-usage]` does not exist.
3. **Matrix row "Model id collides with a table row" had no test** — added
   `src-tauri/tests/ai_translate_contract.rs`'s
   `a_local_or_proxied_model_answering_to_a_seeded_cloud_id_is_priced_as_if_it_were_the_real_cloud_model_an_accepted_limitation`,
   which documents (does not endorse) that `estimate_cost_usd` cannot distinguish a real cloud call
   from a same-named local/proxied one, since it only ever sees the model id string.

Re-ran the full verification set after the fix: `cargo test --test ai_translate_contract` (58/58),
`cargo test --test ipc_contract every_message_key` (2/2, vi.json sync for the two new keys),
`npm run build`, `check:i18n` (999 keys), `check:panel-refs`, and the full `npx vitest run` /
`cargo test --locked` (results recorded in this same session's tool output).

## Spec Change Log

## Review Triage Log

### 2026-09-22 — Pass 1, three layers over a 135 kB diff (blind-hunter 11 · edge-case 1 · verification-gap 1)

| # | Finding | Verdict | Evidence | Route |
|---|---------|---------|----------|-------|
| 1 | `ChunkUsage`'s three fields are all `#[serde(default)]`, so a partial or empty `usage` object deserializes to zeros | **high** | Verified at `core/ai/client.rs:262-271`: `prompt_tokens`/`completion_tokens`/`total_tokens` are `u32` with `#[serde(default)]`, and `interpret_sse_event` (`:429-431`) returns `Usage(usage)` for any `Some`. A provider frame of `{"usage":{}}` therefore becomes `Some(TranslateUsage{0,0,0})` and the panel renders "0 token" — the exact output frozen §Always forbids ("never `0`"). A partial `{"total_tokens":412}` is worse: cost is computed from two zeros, so the line reads a real token count beside a fabricated `~0,0000 USD` | patch |
| 2 | `ai_translate_segment`'s `TranslateOutcome -> AiTranslateOutcomeWire` usage mapping is untested at every layer | **high** | Filed pre-verified by the verification-gap layer with its own searches: no Rust test invokes the command (all hits in `ai_boundary.rs`, `ipc_contract.rs`, `config_invariants.rs` are source-text checks), no `MockRuntime` is used in the AI test files, and the frontend tests hand-build the settled payload. Replacing `usage: usage.map(...)` with `usage: None` at `commands/aitranslate.rs:860` — a one-word copy of the sibling batch arm, which legitimately does exactly that — leaves `cargo test`, `vitest` and every gate green while the single-run figure never appears | patch |
| 3 | The test covering I/O Matrix row "Model id collides with a table row" is a tautology | **high** | Verified by reading the body: `a_local_or_proxied_model_answering_to_a_seeded_cloud_id_...` calls `estimate_cost_usd("claude-sonnet-5", 100, 312)` twice with identical literal arguments and asserts the two results are equal. That holds for any pure function and cannot be made red by any defect it claims to guard, so the matrix row has no real coverage. ⚠️ This also corrects this session's own Matrix Test Audit, which accepted the row on the strength of the test's NAME without reading its body | patch |
| 4 | `interpret_sse_event` checks `usage` before `choices`, so a chunk carrying both drops the text; the doc-comment states this as intended and no test pins it | **low** (data-loss half **maybe-false**) | Verified at `client.rs:429-435`: the `if let Some(usage)` arm returns before `choices` is read, so a frame with both `usage` and `delta.content` yields `Usage` and the content is discarded. The documented tie-break has no test — that half is certain and trivially fixable. Whether any provider emits such a frame is **maybe-false**: `stream_options.include_usage` is specified to add a separate final chunk with `choices: []`, and nothing in this repo can show a server that does otherwise. What would settle it: one run against a real OpenAI-compatible server that attaches usage to a content-bearing chunk | patch |
| 5 | Twelve new `///` doc-comment lines in `client.rs` carry no diacritics while the file's other doc-comments do | **low** | Verified by counting the diff: 12 of the new `///` lines are unaccented (`"LUON co mat"`, `"HANG SO CHUC NANG"`), while the same file's existing doc-comments are accented (`/// Số liệu sử dụng THÔ`, `/// Hình dạng TỐI THIỂU`) and `pricing.rs`'s new ones are fully accented. The file's convention is `///` accented and inline `//` unaccented (pre-existing `// Ket noi dong lai` at `:338`); these cross it. Fix is a direct rewrite, so the low-rejection rule does not apply | patch |
| 6 | The two manual checks the spec requires before the story closes were never performed | **medium** | True as filed: §Verification lists a real-cloud token cross-check and a local-endpoint `stream_options` check, §Design Notes calls the second one a named residual risk, and neither the Implementation Notes nor any log records them. No code fix exists — both need a human with a real endpoint | defer |
| 7 | The bundled price table ships exactly one row (`claude-sonnet-5`), so most real BYOK endpoints will show no money at all | **medium** | Verified at `core/ai/pricing.rs:47-51`: `PRICE_TABLE` has a single entry. The implementation matched the spec's "seed at least the ids this repo already names", so the spec drew the line, not the intent — which under the routing rules keeps the finding rather than dismissing it as out of scope. Routed to defer rather than to a loopback or a patch **as a judgement**: a loopback re-derives code and cannot produce verified prices, and patching means writing dollar figures this session cannot source or check. Extending the table is a one-line edit whenever prices are sourced | defer |
| 8 | `sprint-status.yaml` says `in-progress` while the spec frontmatter says `in-review` | **false** | The two are moved by the workflow at different, defined transition points — the spec advances to `in-review` at the start of review, the sprint key advances at presentation. The diff captured the interval between them; it is the designed mid-flight state, not a disagreement | reject |
| 9 | `estimate_cost_usd_computes_a_price_for_a_seeded_model_id` re-derives its expectation with production's own formula, so a scaling bug would pass | **false** | Refuted by the body: `expected = (100.0 / 1_000_000.0 * 2.0) + (312.0 / 1_000_000.0 * 10.0)` is written from literals in the test, not read from `PRICE_TABLE` or any shared constant. A wrong divisor in production gives a different number and the assertion fails; swapping the input and output rates gives 0.001624 against an expected 0.00332 and also fails. The specific bugs the finding names are caught | reject |
| 10 | Dropping the `Eq` derive may have broken an unnoticed `HashSet`/`BTreeSet` or generic bound | **false** | Grepped `src-tauri/src` and `src-tauri/tests` for `HashSet<`/`BTreeSet<` over `TranslateOutcome`, `AiTranslateOutcomeWire` and `SseEventOutcome`: zero matches. Nothing keyed a set on these types, so the compiler's silence is corroborated rather than merely trusted | reject |
| 11 | `formatUsdParam` can render `0,0000 USD` for a real, non-zero cost | **low** | Real but not reachable in ordinary use: reaching it needs a total under $0.00005, i.e. roughly fewer than 25 input tokens at the seeded rate, while every call carries a RAG-assembled prompt of hundreds. The frozen "never `0`" rule governs the absent-figure case, which has its own key and test. Rejected under the low rule — the fix adds a branch and a threshold rather than correcting or deleting a line | reject |
| 12 | The `!== undefined` half of `aiTranslateBatchUsageSummary`'s cost guard is dead code | **low** | Real: `cost_usd` is typed `number \| null` (`src/config/aitranslate.ts:39`) and `isAiTranslateUsageWire` (`:49`) admits only `null` or `number`, so `undefined` cannot survive validation. Rejected under the low rule: it is never met at runtime, and deleting it is not a clean deletion — `r.usage?.cost_usd` widens to include `undefined` under optional chaining, so removing the check risks a `vue-tsc` error, which makes the fix more than a direct correction | reject |
| 13 | The three deferred entries do not cross-reference each other | **low** | Real: the two batch-adjacent entries say they "nên đi CÙNG một story", and the Prompt Inspector re-defer does not point at them. Rejected under the low rule: `check:debt-owner` passes, every entry names an owner and cites a §SECTION NAME, and a reader reaching any one of them has the owner they need | reject |

## Design Notes

**Why the number must come from the provider.** The sixth signed acceptance criterion — *"nhà cung
cấp không trả về số token → nói rõ không có số liệu"* — only makes sense if the figure is the
provider's. A local tokenizer would make that branch unreachable, would need a new dependency past
the NFR15 licence gate, and would put a number on screen that is ours rather than the one being
billed. So `stream_options: { include_usage: true }` goes on the request, and the residual risk is
named: an OpenAI-compatible endpoint that rejects unknown request fields would fail the call
outright. Most such servers ignore unknown fields; this must be checked by hand against a real
local endpoint before the story closes, because a broken translate path is a worse regression than
a missing figure.

**Why a bundled table, and what it costs.** Ice chose the bundled table over user-entered price
fields on 2026-09-22. It shows a figure with zero setup for a known model, and it keeps the
configuration surface at five fields. Two consequences are accepted rather than solved: a published
price that changes makes a table row quietly wrong, which is why every row carries the date it was
taken; and the table is keyed by name, so a local or proxied model answering to a priced id would
be charged a price it does not cost. The second is in the I/O matrix as a named limitation — the
alternative was a `localhost` heuristic, which is wrong for a self-hosted model on the LAN and
would have introduced a discriminator FR66 deliberately avoided.

**Why the two paths carry it differently.** The batch wire is already an enum with a per-sentence
`Done` variant, so the figure rides along. The single-run wire is a bare `Channel<String>` whose
frames are token text; widening it to a tagged enum would rewrite 4.8's whole streaming contract
for one number. The command's return value is already a tagged enum, already the thing the panel
awaits, and already the place `not_configured`/`done`/`cancelled` are decided.

## Verification

**Commands:**

- `npm run build` **before** any `cargo test` — without `dist/`, `cargo test` fails at compile time.
- `cd src-tauri && cargo test --locked` — measure the baseline on
  `afbe740e6f8b264de4bed5b5367a27e694e539a0` in a still tree FIRST; two timings are comparable only
  from the same cache state.
- `npx vitest run` — 0 red. `npm run test:story 4-11` for the scoped loop; it reports THIẾU KHAI for
  any test file the diff touched that this spec never named.
- `cargo test --test ai_boundary --test config_invariants --test ipc_contract` after the Rust work,
  reading **why** each red fires rather than its colour.
- The eleven `pre-push` gates individually: `check:deps` · `tokens` · `i18n` · `commands` ·
  `layout` · `panel-refs` · `dict` · `dict-manifest` · `lint` · `gates` · `debt-owner`.
- ⚠️ The real `pre-push` order is eleven gates → vitest → `npm run build` → `cargo test`. Run all
  fourteen before calling a phase done, then read CI — `pre-push` runs only on macOS and says
  nothing about the Windows half.

**Manual checks:**

- Point the endpoint at a real cloud provider, translate one sentence, and read the line: the token
  count must match what the provider's own dashboard reports for that call. A number nobody checked
  against the source is the failure mode this whole story exists to prevent.
- Point the endpoint at a local Ollama or LM Studio model and translate one sentence: the call must
  still succeed (the `stream_options` risk above), a token count must appear, and no currency may
  appear anywhere on the line.
