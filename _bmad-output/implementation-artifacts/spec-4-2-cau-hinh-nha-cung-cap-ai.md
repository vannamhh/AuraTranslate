---
title: 'Story 4.2 — Cấu hình nhà cung cấp AI'
type: 'feature'
created: '2026-09-16'
status: 'done'
route: 'dispatch'
review_loop_iteration: 0
baseline_commit: '334a3d0f3f3d42f7a29337086c0ab981cdbb714f'
context:
  - '{project-root}/_bmad-output/implementation-artifacts/epic-4-context.md'
  - '{project-root}/AGENTS.md'
  - '{project-root}/src-tauri/AGENTS.md'
  - '{project-root}/src/AGENTS.md'
---

<frozen-after-approval reason="human-owned intent — do not modify unless human renegotiates">

## Intent

**Problem:** FR68 requires AI provider settings (provider, endpoint, model, generation
parameters) to live at the Global tier and be overridable per Work. Today the Settings
overlay's very first nav section, `ai_and_model`, renders the "no body yet, owned by Epic 4"
placeholder, and `ScopeKind::AiConfig` is reserved with Override semantics but has no table
behind it — `save_value` rejects it, because the generic `config_value` table serves only
global-only kinds. Nothing in the app can store or resolve an AI configuration.

**Approach:** Add a two-tier AI-config domain module outside `core/ai/`, following the
`core/cleanup/` precedent exactly: its own table in the Global and Project schemas, a
`resolve_two_tiers` reader driven by `ScopeResolver::apply_override`, a command module in the
`commands/cleanup.rs` two-layer shape, and a real body for the `ai_and_model` settings section
built on the `GlossarySettingsOverlay` form pattern. The override is **per field**, not
whole-struct — Ice signed that shape 2026-08-04 and the mockup shows one configuration
carrying overridden and inherited fields side by side.

**Decisions taken with Ice, 2026-09-16 (each closes a gap this spec would otherwise guess at):**

- **No connection test in this story, and no code in `core/ai/`.** The "test connection" AC
  needs an outbound call from `core/ai/`, and the AD-13 gate forbids every file outside
  `core/ai/**` from naming it, with no exemption for `lib.rs` or `commands/`. Rather than
  widen that gate one story after Story 4.1 built it, the AC is deferred with a named owner.
  `epics.md` is not edited. This also falsifies an earlier prediction in the debt ledger that
  Story 4.2 would be the first story to put real code into `core/ai/` — it puts none.
- **No API key field in this story.** FR65 is listed under this story, but FR67/NFR11 put the
  key in the OS keychain and that mechanism is Story 4.3. A key entered here would have
  nowhere to go but a plaintext config table. FR65 moves to 4.3 with the keychain, as one
  piece of work.
- **Population floors: bump only this story's own two.** The assigned debt asks whether
  `AI_FLOOR`/`SRC_RS_FLOOR` need raising; measured, `SRC_RS_FLOOR` is 44 against a real
  population of 82. The other 22 floor constants across 16 `*_boundary.rs` files have drifted
  the same way and are recorded as one owned debt rather than fixed here.

## Boundaries & Constraints

**Always:**

- The override resolves **per field**, like Glossary: a `BTreeMap<field key, value>` through
  `ScopeResolver::apply_override("ai_config", global, work)`. A Work row for one field must
  not hide Global values of the other fields. Add a test that seeds a Work override for
  exactly one field and asserts the rest still resolve from Global, with `shadowed` present.
- Each story owns the migration step for the table it needs, added in the same story. This
  story adds one step to the Global schema and one to the Project schema, and nothing else.
- Values are stored as strings, matching every existing config surface. Parsing and range
  checks live in one pure function per field, callable from tests without a Store.
- An invalid value is rejected before any write, and the rejection crosses IPC as an
  `IpcError` whose `message_key` is a new entry in the closed `message_keys!` catalogue.
- Settings-overlay nav buttons are `<form @submit.prevent>` + `type="submit"`, never bare
  `@click`. ~~Every action is a registered command with a literal id.~~
  🔵 **Narrowed 2026-09-16 on Ice's approval — the struck clause was unsatisfiable, and this
  is the only place in the repo that claimed it.** Every action **reachable without an
  argument** is a registered command with a literal id; a per-item action that needs a
  parameter follows the established `ImportPreviewOverlay.vue::onDeleteCleanupRule` shape —
  a form submit calling a local handler. Reason, verified at the declaration site:
  `src/commands/registry.ts:51` declares `run: () => void`, so a registered command cannot
  carry the `field` argument these actions need. What the narrowing gives up is a bindable
  shortcut and a command-palette entry, not keyboard access — the submit buttons are still
  reachable by Tab + Enter. No new gate: `check:commands` Check A watches `@click` only (a
  blind spot `src/AGENTS.md:18` already documents), and a gate policing `@submit` would flag
  every correct use of the established shape. The original wording is struck rather than
  deleted, per this repo's rule about keeping the wrong text visible.
- UI strings are new flat keys in `vi.json` under an `ai_config.`/`settings.` prefix, voiced
  as UX-DR47 requires (no `chúng tôi`, no `bạn`).

**Ask First:**

- If bumping `SRC_RS_FLOOR` turns any existing `*_boundary.rs` case red, stop and show the
  failure instead of lowering the new value.
- If the per-field override needs a change to any `ScopeResolver` signature, stop — three
  signatures are frozen by `scope_boundary.rs` and changing them is not this story's call.

**Never:**

- No file in this story names `crate::core::ai` or `super::ai`, and no file is added under
  `core/ai/`. The AD-13 gate must stay green without being edited for exemptions.
- No outbound network call, no `reqwest`, no `keyring`. Both crates keep their current call
  counts in `core/ai/`: zero.
- No API key field, no key storage of any kind, in any tier.
- No `TranslationProvider` trait — declaring the port belongs with its first implementation,
  and AD-2 keeps the port count at three by procedure, not by inference here.
- No new design tokens and no change to `EXPECTED_COUNTS` in `check-tokens`.
- No edit to `epics.md` for the two deferred ACs.

## I/O & Edge-Case Matrix

| Scenario | Input / State | Expected Output / Behavior | Error Handling |
|----------|--------------|---------------------------|----------------|
| No configuration anywhere | Global table empty, no Work open | Form renders with empty fields; AI panel status sentence unchanged | N/A |
| Global only, no Work open | Global has all fields | Every field resolves from Global, marked inherited | N/A |
| Work overrides one field | Global has all fields; Work has `endpoint` only | `endpoint` resolves from Work marked overridden with the Global value present as shadowed; every other field still resolves from Global | N/A |
| Save a Work override with no Work open | User edits a field at Work tier, `OpenWorkState` is `None` | Write refused, nothing persisted | `IpcError`, store-missing code |
| Temperature out of range | `-1`, `3`, `abc` | Rejected before write; field marked invalid inline | `IpcError` with the invalid-value key |
| Max tokens not a positive integer | `0`, `-5`, `1.5`, `abc` | Rejected before write | `IpcError` with the invalid-value key |
| Endpoint is not a valid absolute URL | `localhost:11434`, empty string | Rejected before write | `IpcError` with the invalid-value key |
| Clear a Work override | User clears an overridden field | Work row deleted; the field resolves from Global again, no longer marked overridden | N/A |
| Reopen a saved `.atproj` | Work-tier rows written in an earlier session | Work tier resolves in the new session — `open_work` rebuilds `ScopeResolver::with_work` | N/A |

</frozen-after-approval>

## Code Map

- `src-tauri/src/core/store/schema.rs` — add one `Migration` to `GLOBAL_MIGRATIONS` (target
  7 → 8) and one to `PROJECT_MIGRATIONS` (target 21 → 22), each creating the AI-config table.
  Read the migration rule and the anti-EAV clause in the `CONFIG_VALUE_DDL` doc-comment first.
  Bumping a target makes `tests/store_contract.rs::a_fresh_database_migrates_up_to_target_and_logs_it`
  red until its expected version is updated — that is the intended signal, not a surprise.
- `src-tauri/src/core/cleanup/store.rs` — **read-only, the closest template.**
  `resolve_two_tiers(resolver, global, work)` is the shape to copy for the reader.
- `src-tauri/src/core/glossary/store.rs` — **read-only.** The per-field `apply_override`
  call with two loaded tiers, including how `shadowed` reaches the caller.
- `src-tauri/src/core/aiconfig/` — **new module.** Field definitions, one pure
  parse/validate function per field, a two-tier reader, and write/delete against the new
  table. Name follows the single-word `webimport`/`cleanup` convention. It must not be placed
  under `core/ai/`, and must not name it.
- `src-tauri/src/core/scope/kinds.rs:175` — **read-only.** `AiConfig => "ai_config" : Override`
  already exists with the signed per-field decision; do not add a kind, do not change semantics.
- `src-tauri/src/core/scope/mod.rs` — **read-only.** `apply_override` is the entry point;
  public method names are `apply_*` while internals are `resolve_*`, and `scope_boundary.rs`
  bans the internal names outside `core/scope/**`.
- `src-tauri/src/core/scope/store.rs` — **read-only.** `save_value` rejects non-global-only
  kinds; this is why `ai_config` needs its own table rather than `config_value`.
- `src-tauri/src/commands/cleanup.rs` — **read-only, the command template.** Two-layer shape:
  pure fns taking `(global: Option<&Store>, open: Option<&OpenWork>, …)`, a `pub mod wire`
  whose `#[tauri::command]` fns share the pure fns' names, tier routing helper, snake_case
  wire DTO, and adapters that do not judge arguments.
- `src-tauri/src/commands/aiconfig.rs` — **new.** Same shape; declare it in `commands/mod.rs`.
- `src-tauri/src/core/i18n/mod.rs` — add the new error keys to the `message_keys!` catalogue
  with their required params. `IpcError::new` is the only constructor; conversion is a `From`
  impl on the module's error type, never a struct literal.
- `src-tauri/src/lib.rs` — register the new commands inside `generate_handler!` alongside the
  cleanup commands. Do **not** add ACL entries to `capabilities/main.json`; three permissions
  are locked by `tests/config_invariants.rs`.
- `src/settingsState.ts` — `settingsSectionHasBody` must return `true` for `ai_and_model`;
  once it does, that section stops rendering the owner placeholder.
- `src/SettingsOverlay.vue` — add the section body. Nav buttons stay `<form>` + submit.
- `src/aiConfigState.ts` — **new.** State module in the `glossarySettingsState.ts` shape:
  `readonly` refs out, one mutable ref per input, exported pure validators, a save path that
  re-validates rather than trusting the template, and `IpcError` held for `tError()`.
  `check:panel-refs` requires module-level cells to be cleared by a `reset*()` function or to
  carry a named EXEMPT with a reason — `glossarySettingsState.ts` documents that choice.
- `src/GlossarySettingsOverlay.vue` — **read-only, the form template.** `@submit.prevent`
  dispatching a command, `<fieldset :disabled>`, `:value` + `@input` rather than `v-model`,
  inline invalid message gated on the same exported validator.
- `src/commands/index.ts` — register the save/clear-override commands through the injected
  `deps` port object; `src/commands/registry.ts` is read-only and must not gain imports.
- `src/i18n/vi.json` — new flat keys. Two `panel.ai_translation.*` keys exist and are not
  touched by this story.
- `src-tauri/tests/ai_boundary.rs` — bump `SRC_RS_FLOOR` 44 → 65 (~80% of the measured 82).
  `AI_FLOOR` stays 1: this story adds no file to `core/ai/`. Do not touch the exemption
  predicate or `FORBIDDEN_BARE_TOKENS`.
- `_bmad-output/implementation-artifacts/deferred-work.md` — append at EOF only; never edit
  or delete existing entries; every open entry needs a real `Chủ:`.

## Tasks & Acceptance

**Execution:**

- [x] `src-tauri/src/core/store/schema.rs` — add the two migration steps and bump both
      targets — rationale: an Override kind cannot ride the global-only `config_value` table,
      and the repo's rule is that the story needing the table owns its migration step.
- [x] `src-tauri/src/core/aiconfig/` — field set (provider, endpoint, model, temperature, max
      tokens), one pure validator per field, two-tier reader via `apply_override`, write and
      delete — rationale: pure validators are the only way the range and URL cases in the I/O
      matrix can be tested without a Store.
- [x] `src-tauri/src/core/i18n/mod.rs` — add the invalid-value and store-missing message keys
      with their params — rationale: the catalogue is closed, so an error cannot be reported
      without an entry.
- [x] `src-tauri/src/commands/aiconfig.rs` + `commands/mod.rs` + `lib.rs` — the two-layer
      command module and its registration — rationale: the adapter layer must not judge
      arguments; rejection stays in the domain module.
- [x] `src/aiConfigState.ts` + `src/settingsState.ts` — state module, exported validators, and
      flipping `settingsSectionHasBody` for `ai_and_model` — rationale: the section currently
      renders the owner placeholder, and that is the single switch that retires it.
- [x] `src/SettingsOverlay.vue` + `src/commands/index.ts` + `src/i18n/vi.json` — section body,
      command registrations, new strings — rationale: every action must be a registered
      command with a literal id for `check:commands` to see it.
- [x] `src-tauri/tests/` — contract tests for the two-tier resolution, covering every row of
      the I/O matrix, including the one-field-override row and the reopen row — rationale: the
      per-field shape is the one thing a whole-struct implementation would silently get wrong.
- [x] `tests/frontend/` — validator and save-path tests in the `glossarySettings.test.ts`
      shape — rationale: the save path re-validates because Enter bypasses a disabled button.
- [x] `src-tauri/tests/ai_boundary.rs` — bump `SRC_RS_FLOOR` to 65 — rationale: closes the
      measured half of the assigned debt; 44 against a real 82 means the floor stopped being a
      tripwire.
- [x] `_bmad-output/implementation-artifacts/deferred-work.md` — append the deferred
      connection test (owner: the first story that calls a provider), FR65 + the key field
      (owner: Story 4.3), the 22 drifted floors with the measured table (owner: unassigned —
      flag for Ice), and the re-export gate decision; mark the boundary-rerun debt closed with
      the measured result — rationale: the ledger is the evidence for the next decision, and
      an entry that is merely predicted must not be written as if measured.

**Acceptance Criteria:**

- Given a Global configuration and an open Work whose only override is `endpoint`, when the
  configuration is resolved, then `endpoint` comes from the Work tier with the Global value
  present as shadowed and every other field comes from Global — a whole-struct implementation
  fails this.
- Given a Work-tier configuration written in one session, when the same `.atproj` is closed
  and reopened, then the Work tier resolves in the new session.
- Given the `ai_and_model` settings section, when opened, then it shows the real form instead
  of the "no body yet" placeholder, and each field shows whether it is inherited or overridden.
- Given any field rejected by its validator, when saved, then nothing is written and the
  failure surfaces as an `IpcError` whose `message_key` exists in the catalogue.
- Given the whole repository, when the gates and suites run after this story, then
  `cargo test --locked` and `npx vitest run` are green with the new cases counted, and the
  eleven `pre-push` gates exit 0 — including `check:debt-owner` with zero open entries
  lacking an owner.
- Given `ai_boundary.rs` on the tree as it stands after this story, when it runs, then it is
  green **without** its exemption predicate or forbidden-token list being edited, and
  `AI_FLOOR` is still 1 because no file was added to `core/ai/`.

## Implementation Notes

- `core/aiconfig/` (new): `AiConfigField` (5 variants) · `AiConfigTier` · `validate_field`
  dispatching to one pure validator per field · `ResolvedField`/`resolve_two_tiers` (wraps
  `ScopeResolver::apply_override`, converting `core::scope::Tier`/`Resolved` to domain-local
  types so nothing outside `core/scope/**` needs them) · `write_field`/`clear_field`.
- Schema: one `AI_CONFIG_DDL` constant (`ai_config(key, value, updated_at)`, no `tier`
  column, no `CHECK` on `key` — same rationale as `CONFIG_VALUE_DDL`), used at
  `GLOBAL_MIGRATIONS` step 8 and `PROJECT_MIGRATIONS` step 23 (see §Spec Change Log for why
  23, not the 22 the Code Map named).
- `commands/aiconfig.rs`: three pure functions (`ai_config_get`/`ai_config_save_field`/
  `ai_config_clear_override`) plus a `mod wire` shell, same two-layer shape as
  `commands/cleanup.rs`. `ai_config_clear_override` always targets the Work tier — clearing
  the Global tier has no lower tier to fall back to, so no `tier` parameter exists for it.
- `endpoint` validation is hand-rolled (scheme prefix + non-empty, whitespace-free host),
  **not** `reqwest::Url::parse` — `tests/webimport_boundary.rs::
  reqwest_is_named_only_inside_core_webimport_or_core_ai` forbids the token `reqwest`
  anywhere outside `core/webimport/`/`core/ai/`, with no exemption for "just parsing, no
  network call." Caught this by running the full `cargo test --locked` suite, not by reading
  the gate's source first.
- `temperature` range `[0.0, 2.0]`: not stated by any FR/mockup found; chosen as the
  OpenAI-compatible convention (the mockup's endpoints are explicitly "OpenAI-compatible").
  Not an §Ask First item per the spec's own list, so decided rather than escalated — flagged
  here for Ice to override if wrong.
- Frontend: `src/config/aiconfig.ts` (IPC adapter) · `src/aiConfigState.ts` (state module,
  `glossarySettingsState.ts` shape — exported pure validators mirroring the Rust ones,
  `resetAiConfigSection()` for `check:panel-refs`) · `SettingsOverlay.vue` gained the
  `ai_and_model` section body, five per-field forms (`@submit.prevent` calling a local
  handler with the field as a parameter — `check:commands` Check A only watches `@click`,
  and `ImportPreviewOverlay.vue`'s cleanup-rule rows already establish this pattern for a
  per-item action list, so no new command ids were registered for the five save/five clear
  actions). `settingsState.ts` now triggers `loadAiConfigSection()` on entering/opening the
  section, same shape as its existing domain-log load for `privacy`.
- Tests: `src-tauri/tests/aiconfig_contract.rs` (9 cases, one per I/O-matrix row, including
  the one-field-override case and a reopen simulation via a fresh `Store::open` +
  `ScopeResolver::with_work`) · `tests/frontend/aiConfigState.test.ts` (18 cases: validators +
  save/clear paths, `glossarySettings.test.ts` shape).
- `SRC_RS_FLOOR` in `tests/ai_boundary.rs`: 44 → 68 (80% of 85, the population measured
  AFTER this story's own three files — an earlier pass used 65, ~80% of 82 measured BEFORE
  those files landed; corrected per Review Triage Log #6). `AI_FLOOR` unchanged at 1 — no
  file was added to `core/ai/`.

## Spec Change Log

- **Code Map said "add one step to `PROJECT_MIGRATIONS` (target 21 → 22)"; the real target
  measured at implementation time was already 22** (Story 6.15's `CHAPTER_ORIGIN_DDL`, landed
  between spec authoring and implementation — `epic-4-context.md` itself documents that
  Stories 4.2–4.12 run after Epic 5 and Epic 6, so Epic 6 stories including 6.15 were expected
  to have already landed). Measured via `PROJECT_MIGRATIONS`'s own array before editing it,
  per `AGENTS.md`: "đọc chính danh sách `Migration` để biết bước kế tiếp, đừng đếm bằng mắt."
  Used the measured value: the new step is `to_version: 23`, not 22. `GLOBAL_MIGRATIONS`'s
  "target 7 → 8" in the Code Map was already correct as measured (still 7 at implementation
  time), so that half needed no correction.
- Six other test files asserted the OLD final `PROJECT_MIGRATIONS`/`GLOBAL_MIGRATIONS`
  version as a "fresh database lands here" invariant (`segment_contract.rs` ×8 sites,
  `chapter_origin_contract.rs`, `segment_role_contract.rs`, `pinned_contract.rs` ×2,
  `glossary_contract.rs`, `config_invariants.rs`'s command-file census). All bumped to match,
  following the exact precedent each of those sites already documents for the prior five
  times this same ripple happened (Stories 6.5/6.11/6.13/6.15).
- No `TranslationProvider`-adjacent UI (provider-name pills, a closed provider list) built
  into the form despite the mockup showing `Anthropic`/`OpenAI` pill buttons for the
  `provider` field — §Never bars declaring the port in this story (AD-2), and the field is
  validated as free text only. The mockup's pills are a Story 4.8+ concern once a real
  provider list exists to validate against.
- 🔴 **A frozen `Always` line is over-broad, and the implementation was right to depart from
  it — added at the orchestrator's acceptance pass, because the departure was recorded only in
  a `.vue` comment and would otherwise have left this log claiming full compliance.** The line
  reads *"Every action is a registered command with a literal id."* The per-field save and
  clear-override actions cannot satisfy it: `src/commands/registry.ts:51` declares
  `run: () => void`, so a registered command takes no argument, and these actions need the
  field as a parameter. The repo already has a same-context precedent —
  `ImportPreviewOverlay.vue:247` `onDeleteCleanupRule(rule: CleanupRuleReportWire)`, invoked at
  `:1376` through `<form @submit.prevent="onDeleteCleanupRule(rule)">` — a parameterized local
  handler behind a form submit, not a dispatch. The precedent was checked at its declaration
  site, not taken on the implementation's word. Consequence: `src/commands/index.ts` is
  untouched, which is correct here but means the Tasks line naming it is only partly
  applicable. No gate catches this either way — `check:commands` Kiểm A polices `@click` only,
  so the frozen line was never machine-enforced. **The line sits inside
  `<frozen-after-approval>` and has NOT been edited: narrowing it to "every action reachable
  without a parameter" is Ice's call, not this pass's.**

## Review Triage Log

Pass 1 — 2026-09-16, three layers (blind-hunter, edge-case-hunter, verification-gap). 14 distinct
findings after grouping. No `intent_gap` and no `bad_spec`, so no loopback;
`review_loop_iteration` stays 0. Every claim below was re-verified at its cited location by the
orchestrator before a verdict was rendered.

| # | Finding | Verdict | Route | Evidence |
|---|---|---|---|---|
| 1 | `Chủ: chưa phân` is invisible to the debt-owner gate, so three genuinely ownerless entries pass as owned | high | patch | `scripts/check-debt-owner.mjs:192` `NEGATIVE_OWNER_RE` is a CLOSED list (`chưa gán\|chưa có\|chưa cần\|chưa ai\|không ai\|chưa chốt\|trống`) and omits `chưa phân`. Ran the gate: it prints all-clear while three entries are unassigned. The gate exists precisely to stop this. |
| 2 | The drifted-floor table scores `SRC_TAURI_RS_FLOOR` against the wrong population | medium | patch | The table row reads `61 \| 85 \| 71,8%`, but that constant counts `src-tauri/{src,tests}/**`, measured 140 — so the true ratio is 43,6%, making it the WORST row, not a middling one. `dict_boundary.rs:313` warns in its own doc-comment that this population differs from the four `src/**` floors. The table is evidence for an Ice decision, and it understates the problem. |
| 3 | TS temperature check rejects values Rust accepts | medium | patch | `src/aiConfigState.ts:111` uses `/^-?\d+(\.\d+)?$/`; `core/aiconfig/mod.rs:136` uses `trimmed.parse::<f64>()`. Rust's float grammar accepts `.5`, `5.`, `1e-1`, `+0.5`; the regex rejects all four ⇒ a backend-valid value is unreachable from the UI, and the two hand-copied rules have already diverged. |
| 4 | The invalid-value message renders on a never-touched empty form | medium | patch | `isAiConfigValueValid` returns false for every empty draft (`aiConfigState.ts:108` for provider/model, and the numeric/URL cases likewise), and `SettingsOverlay.vue:243` gates the alert on that alone. First open with nothing configured shows five red messages — colliding with the epic invariant that "chưa cấu hình" is explicitly NOT an error state. |
| 5 | The status line models two states where three exist | medium | patch | `vi.json:803` interpolates `{value}` and `SettingsOverlay.vue:236` passes `shadowed ?? ''`; `commands/aiconfig.rs:88` sets `shadowed: None` when a field exists only at the Work tier, so the sentence renders ending in "đang là " with nothing after. Reachable on a likely path: configure inside a Work while Global is empty and every field hits it. Separately, a field with no value anywhere still reads "Kế thừa Toàn cục". |
| 6 | `SRC_RS_FLOOR = 65` sits below the band its own comment cites | low | patch | 65 was derived as ~80% of 82, the population measured BEFORE this story added three files. Real population is now 85, so 65 is 76,5% — under the 80–85% mold the comment invokes, and the same margin the story catalogues as drift in other constants. |
| 7 | Unrecognized tier string is silently relabelled Global | low | patch | `core/aiconfig/store.rs:72` `AiConfigTier::from_wire(...).unwrap_or(AiConfigTier::Global)`. Unreachable today — `core::scope::Tier::as_str()` yields only `global`/`work` — so the harm is latent, but the fix is a direct correction and `AiConfigStoreError::Scope` already exists to carry it. |
| 8 | Work-tier routing duplicated instead of reused | low | patch | `commands/aiconfig.rs:142` re-writes `store_for_tier`'s Work branch verbatim rather than calling it; a later change to the tier rule would land in one of the two. |
| 9 | Nothing keeps the Rust and TS validators in sync | medium | defer | Finding 3 is this defect already realized. A cross-language contract check needs a shared fixture, which is more than a direct correction — recorded with an owner instead of improvised here. |
| 10 | Work-tier availability derived from UI mode, not from authority | medium | defer | Independently verified: `OpenWorkState` is cleared only in `lib.rs::close_open_work`, reached only from the `RunEvent::Exit` branch, so it stays `Some` all session while `currentMode` returns to `library`. The verification-gap layer added what the orchestrator's own pass had missed — the repo ALREADY solves this exact shape with `work_tier_available: bool` computed from `OpenWorkState` inside the IPC response and consumed by three frontend modules. That precedent is what makes option (a) of the existing debt entry the lower-risk one. |
| 11 | sprint-status says `in-progress`, not `review` | false | rejected | `in-progress` is what step-03 prescribes; `review` is set at step-05. The bad outcome does not occur. |
| 12 | Endpoint accepts a host with no dot (`http://x`) undocumented | false | rejected | A dotless host is required, not tolerated: `http://localhost:11434/v1` is the primary FR66 case. Accepting it is correct behavior, so there is no defect to document. |
| 13 | `AiConfigStoreError::Scope` produces no log line | low | rejected | Developer-only, and only on a path shown to be unreachable. Finding 7's fix makes the same condition speak through the error type, which is the direct correction; adding a logging call on top would be added complexity for a case no one has been shown to reach. |
| 14 | A frozen `Always` line is unsatisfiable, and the implementation was right to depart from it | medium | rejected for action; escalated | Verified at the declaration site, not taken on the implementation's word: `src/commands/registry.ts:51` declares `run: () => void`, so a registered command cannot carry the `field` argument these actions need, and `ImportPreviewOverlay.vue:247`/`:1376` is the same-context precedent. The only remaining fix edits this build's spec — and the root cause sits inside `<frozen-after-approval>`, which only Ice may narrow. Recorded in §Spec Change Log and raised in the final summary rather than resolved here. |

## Design Notes

**Why the AI-config domain lives outside `core/ai/`.** The instinct is that anything named
"AI" belongs in `core/ai/`. It cannot: AD-13's gate forbids every file outside `core/ai/**`
from naming that module, with no exemption for `lib.rs` or `commands/`, so a command module
reaching into `core/ai/` would turn the gate red on this story's first line. The measured
contrast is that `core/dict` is named from ten files outside itself — `ai/` is the only module
under this rule, which is exactly why no precedent exists for reaching into it. Configuration
is not AI logic: it is a two-tier settings domain like `cleanup/` and `glossary/`, and the
vocabulary already agrees — `ScopeKind::AiConfig` lives in `core/scope/kinds.rs`, not in
`core/ai/`. Keeping it outside leaves the question of how anything ever calls into `core/ai/`
open for the story that first needs to, which is the honest place for it.

**The per-field override is the load-bearing detail.** Ice signed it on 2026-08-04 after the
mockup showed one configuration carrying an overridden `endpoint` next to an inherited model
and temperature. A whole-struct override passes every obvious test and fails exactly one
scenario: a Work that overrides a single field and expects the others to keep tracking Global.
That scenario is in the I/O matrix and in the acceptance criteria for that reason.

**Why the write tier follows "is a Work open," with no explicit Global/Work switcher in the
UI.** The mockup shows a per-field "ghi đè cho Tác phẩm này" / "trả về kế thừa" affordance but
does not show how the user picks which tier a field's *input* writes to when both a Global
default and a Work override could plausibly be edited from the same screen. Rather than invent
a scope-switcher control the spec's Boundaries never asked for (and that `GlossarySettingsOverlay.vue`'s
own precedent explicitly warns against building for a field that doesn't need it), the save
target is the tier the screen is already operating in: no Work open ⇒ every save writes Global
(there is nowhere else to write); a Work open ⇒ every save writes a Work-tier override, and a
"Trả về kế thừa" button (shown only when a field is currently overridden) clears it back to
inherited. The consequence, named rather than left for someone to discover: editing the Global
default *while* a Work is open has no path in this screen — close the Work, edit, reopen. This
satisfies every literal AC and I/O-matrix row (including the Rust-level "no Work open ⇒ Work-tier
save refused" case, which the UI simply never triggers, but the command layer still enforces
and the contract test still covers). Whether editing Global-with-a-Work-open needs a dedicated
control is left as a UX question, not resolved here.

🔵 **Correction, 2026-09-16, orchestrator acceptance pass — the paragraph above is right about
the design and wrong about two facts. Original text kept; the correction is appended, not
substituted.**

The write tier is not read from the authority. `settingsState.ts` computes it as
`currentMode.value !== 'library'` — a **proxy** for "a Work is open", where the authority is
`OpenWorkState` in Rust. Measured, the two are not equivalent:

- `OpenWorkState` is cleared in exactly one function, and that function runs only on
  `RunEvent::Exit`. No IPC command closes a Work. Every other `*guard = None` in the project
  command module clears import state, not the open Work.
- Therefore, once any Work has been opened, `OpenWorkState` stays `Some` for the rest of the
  session, Library mode included.

Two consequences follow, and both contradict the sentence above:

1. **"Editing the Global default while a Work is open has no path in this screen" is false.**
   Returning to Library leaves the Work open in Rust while the screen flips to Global — so the
   path exists, and it is reached without any deliberate act.
2. **"Close the Work, edit, reopen" describes an action the user cannot perform.** There is no
   close-Work affordance; the only close is quitting the application.

Nothing here corrupts data and no acceptance criterion is affected — the resolution semantics
are unchanged, and a mismatched tier request is refused at the command layer rather than
written. What is affected is trust in the record, and a coupling nothing tests: if a later
story adds a close-Work command, or makes Library reachable on a different signal, the proxy
drifts from the authority with no case going red. The honest reading is that this screen writes
to "the tier the *mode* implies", not "the tier the backend is actually in", and that those are
the same today only by accident. Recorded as an owned debt.

## Verification

**Commands:**

- `cd src-tauri && cargo test --locked --test ai_boundary` — expected: all cases green with no
  edit to the exemption predicate.
- `cd src-tauri && cargo test --locked` — expected: 0 red, and the new contract cases present
  by name. Record the before and after case counts.
- `npx vitest run` — expected: 0 red. Run alone; `fileParallelism` is off and a concurrent
  `cargo test` has already caused flake, with a debt entry for it.
- `npm run check:i18n && npm run check:commands && npm run check:panel-refs && npm run check:tokens && npm run check:debt-owner && npm run check:gates` — expected: exit 0 each.
- `npm run build` — expected: success.

**Removal counter-evidence (record the case names and the real red count):**

- Change the resolution to whole-struct override — expected: the one-field-override case goes
  red, and it alone. If it stays green, the test is not testing the signed shape.
- Point a migration step at the wrong schema target — expected: the store-contract migration
  case goes red.
- Lower `SRC_RS_FLOOR` below the real population — expected: nothing goes red, which is the
  point of raising it; instead raise it above 82 and confirm the population case fires.

**Manual checks:**

- `npm run tauri dev` → Settings → "AI và mô hình": the form renders instead of the
  placeholder, inherited and overridden markers read correctly with a Work open and with none,
  and no field is tinted with an `error` color in its resting state.
