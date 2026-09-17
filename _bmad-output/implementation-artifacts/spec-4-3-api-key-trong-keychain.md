---
title: 'Story 4.3 — API key trong keychain'
type: 'feature'
created: '2026-09-16'
status: 'done'
route: 'dispatch'
review_loop_iteration: 0
baseline_commit: '93fe113dc2fc0b71012388d7f043c2527b66a5c5'
context:
  - '{project-root}/_bmad-output/implementation-artifacts/epic-4-context.md'
  - '{project-root}/AGENTS.md'
  - '{project-root}/src-tauri/AGENTS.md'
  - '{project-root}/src/AGENTS.md'
---

<frozen-after-approval reason="human-owned intent — do not modify unless human renegotiates">

## Intent

**Problem:** FR67/NFR11 require the AI provider's API key to live in the OS keychain,
never in a config file, a project file or a log, and never to cross the IPC boundary —
the frontend may learn only "configured" / "not configured". Today nothing in the repo
touches a keychain: `core/ai/mod.rs` is a 10-line doc stub, there is no secret type, no
redaction helper, and `core/aiconfig/` has five plaintext fields and deliberately no key
field. FR65 (BYOK key entry) was moved here by Story 4.2 so the field and its storage
mechanism land in one story rather than two.

**Approach:** Add a keychain-backed secret store to the existing `core/aiconfig/` domain —
outside `core/ai/`, for the same AD-13 reason Story 4.2 recorded — exposing set, delete and
a `configured: bool` probe, plus an internal read reserved for Story 4.8's provider call.
The key gets its own wire shape carrying only status, never a value; the settings section
gains a key row built on the field rows already there. No new table and no migration: the
keychain itself is the source of truth for whether a key exists.

**Decisions taken with Ice, 2026-09-17 (each closes a gap this spec would otherwise guess at):**

- **The key is Global-only — one keychain entry for the whole app, no per-Work override.**
  FR68 names only provider, model and generation parameters as two-tier; FR67 and this story's
  ACs say nothing about tiers. The mockup draws a tier marker on the key row, and this decision
  departs from it: a per-Work key would mean one keychain entry per Work keyed on a stable Work
  identity, plus an orphan question — nothing in the app can enumerate keychain entries, so a
  deleted Work would leave a credential behind with no way to find it. The consequence to build
  deliberately: the other five rows write the Work tier while a Work is open, and the key row
  must not follow them — it says plainly that one key serves every Work. `epics.md` is not edited.
- **Production keeps `keyring`; `keyring-core` is declared as a dev-dependency so tests can
  install a mock store.** Measured in the downloaded source: `keyring-4.1.6/src/v1.rs:108-121`
  installs the real platform store through a private `LazyLock` on the first `Entry::new`, and
  `keyring-core-1.0.0/src/lib.rs:65-71` shows `set_default_store` overwrites unconditionally —
  so a mock installed before first use is silently replaced. The order that works is: force
  initialization once, then swap in `keyring_core::mock`. AD-29's wording is untouched and the
  production path never names `keyring-core`. This is the project's first `[dev-dependencies]`
  section; it adds no `[[package]]` to `Cargo.lock`, only one name to the `auratranslate`
  dependency list.

## Boundaries & Constraints

**Always:**

- The key crosses no IPC boundary in either direction, except inbound on save. No
  `#[tauri::command]` return type, and no type reachable from one, may carry the key value.
  `AiConfigFieldWire.value: String` must not gain a sixth field for it.
- The secret is held in a newtype whose `Debug` and `Display` render a fixed placeholder,
  never the value. Deriving `Debug` on it, or any struct holding it, is forbidden.
- The raw-value accessor is reachable only from inside `core/aiconfig/**`, enforced by a
  source-scanning boundary test in the shape the repo's other `*_boundary.rs` files use —
  with a counter-check that removing the seam turns an old case red.
- "Not configured" is a state, not an error (the epic invariant), and so is a keychain that
  refuses to answer — the latter surfaces as an `IpcError` on the action, not as a
  permanently broken settings screen.
- Errors crossing IPC use the AD-21 shape with a new `message_keys!` entry; no error message,
  parameter or log line interpolates the key or any prefix of it.
- Adding or changing a `#[tauri::command]` in `commands/aiconfig.rs` requires updating its
  `COMMAND_FILE_CENSUS` row in `config_invariants.rs`, which today reads 3 plain / 0 async.
- The key is stored and read at the Global tier only, whatever tier the screen is operating in.
  A Work-tier key request is refused at the command layer, not merely hidden in the UI.
- A test that means to use the mock store must prove it got it: assert the real store
  initialized (`Entry::store_status()` is `Ok`) and fail loudly if not. A `NoDefaultStore`
  error silently makes every later keychain case exercise nothing.

**Never:**

- No `tauri-plugin-keyring` (AD-29), and no new `[[package]]` in `Cargo.lock` — the whole
  keyring family is already pinned and licence-cleared; measured below. The production path
  never names `keyring-core`; that name belongs to `[dev-dependencies]` only.
- No new table, no migration step. Bumping a migration target would ripple into 6 files /
  16 hard-coded version sites for a story that needs no schema.
- No `TranslationProvider`, no outbound network call, no file under `core/ai/`, no edit to
  `ai_boundary.rs`'s exemption predicate or `FORBIDDEN_BARE_TOKENS`.
- No key value written to `global.db`, `project.db`, `.atproj`, or any file on disk.
- No "test connection" affordance — it is owned by Story 4.8.

## I/O & Edge-Case Matrix

| Scenario | Input / State | Expected Output / Behavior | Error Handling |
|----------|--------------|---------------------------|----------------|
| No key anywhere | Keychain has no entry | Key row reads "not configured"; every other section behaves unchanged (FR77) | N/A |
| Save a key | Non-empty value, keychain writable | Entry created; row flips to "configured"; the value is never returned | N/A |
| Save an empty or whitespace-only key | `""`, `"   "` | Rejected before any keychain call; nothing written | `IpcError`, invalid-value key |
| Overwrite an existing key | Entry exists, new value saved | Entry replaced; status stays "configured"; no second entry created | N/A |
| Delete the key | Entry exists, user deletes | Entry gone from the keychain; status returns to "not configured" | N/A |
| Delete when none exists | No entry | Treated as success — the post-state is the requested one | N/A |
| Save a key while a Work is open | Work open, other five rows would write the Work tier | The key still writes Global, and the row says one key serves every Work | N/A |
| Work-tier key request reaches the command layer | Any caller asks for the key at the Work tier | Refused; nothing written and nothing read | `IpcError`, key-is-global key |
| Keychain refuses (locked, denied, no store) | Store returns an error on any call | The action fails and says so; the section stays usable and other settings still save | `IpcError`, keychain-unavailable key |
| Error raised while the keychain holds a key | Any failing call | Neither the message, its params, nor any log line contains the key or a prefix of it | N/A |

</frozen-after-approval>

## Code Map

- `src-tauri/Cargo.toml:61-67` — `keyring = "=4.1.6"` is already pinned with a comment naming
  this story. Measured 2026-09-16: `cargo tree -p keyring` on macOS adds exactly three crates
  nothing else pulls (`keyring` 4.1.6 · `apple-native-keyring-store` 1.0.1 · `keyring-core`
  1.0.0); `log` and `security-framework` arrive anyway via `tauri` and `reqwest`. All five
  keyring-family crates in `Cargo.lock` (including the Windows and Secret-Service stores) are
  `MIT OR Apache-2.0` with both licence bodies read in `~/.cargo/registry/src/…`. A new
  `[dev-dependencies]` section declares `keyring-core = "=1.0.0"`; measured, `Cargo.lock`'s
  `auratranslate` entry lists `"keyring"` but not `"keyring-core"` today, so the lock gains
  that one name and **no** `[[package]]`.
- `src-tauri/src/core/aiconfig/mod.rs:36-40` — the doc-comment that says there is no key field
  "because that is Story 4.3", and `AiConfigField` (5 variants), whose shape the key must
  **not** join. `validate_field` at `:200` is the validator pattern to copy.
- `src-tauri/src/core/aiconfig/store.rs:27,58,91,114,128` — `ResolvedField`, `resolve_two_tiers`,
  `write_field`, `clear_field`, and `AiConfigStoreError` with its two `From … for IpcError`
  impls at `:163`/`:183`. The new error variants join this enum.
- `src-tauri/src/core/aiconfig/keychain.rs` — **new.** Secret newtype, account naming, set /
  delete / `configured` probe, and the read reserved for Story 4.8.
- `src-tauri/src/commands/aiconfig.rs:31,110,125,149,168,181` — `store_for_tier`,
  `AiConfigGetWire`, the three pure fns and `mod wire`. This is the two-layer shape to follow;
  the adapters must keep not judging arguments.
- `src-tauri/src/core/i18n/mod.rs:62,646-660` — `message_keys!` and Story 4.2's three aiconfig
  keys with their params. New keys go in the same block.
- `src-tauri/src/core/ai/mod.rs:8-10` — **read-only except one correction.** It claims `keyring`
  is a crate for `core/ai/`; after this story the keychain lives in `core/aiconfig/`, so that
  line gets a 🔵 in-place fix per `AGENTS.md:51`, not a deletion.
- `src-tauri/tests/aiconfig_contract.rs:30,47,54,150` — `temp_dir`, `open_global`,
  `open_work_real`, and the two-tier example case. New contract cases go here.
- `src-tauri/tests/config_invariants.rs:1385` — `COMMAND_FILE_CENSUS`; the `commands/aiconfig.rs`
  row must match the new command count.
- `src-tauri/tests/ai_boundary.rs:65,89,99,211` — **read-only.** `AI_FLOOR` stays 1 and
  `SRC_RS_FLOOR` (68) may need a re-measure if the file count changes; the predicate and token
  list must not be touched.
- `src/config/aiconfig.ts:81-119` — the per-command `invoke` wrappers and their `IpcError`
  mapping. New commands get siblings here.
- `src/aiConfigState.ts:56,94-123,174,202,227,250` — field list, accessors, the pure validator,
  load / save / clear, and `resetAiConfigSection`. The key needs its own state, not a sixth
  entry in `AI_CONFIG_FIELDS`.
- `src/SettingsOverlay.vue:242-300` — the per-field row: one `<form @submit.prevent>` per
  action (they cannot nest), `:value` + `@input`, status line, and the separate
  clear-override form. The key row copies this, with a status-only display instead of a value.
- `src/i18n/vi.json:54-56,790-807` — the aiconfig error and section keys; new flat keys join
  them, impersonal voice per UX-DR47.
- `scripts/check-deps.mjs:174` — **read-only.** `tauri-plugin-keyring` is already banned here;
  no edit needed, and the gate must stay green untouched.
- `_bmad-output/planning-artifacts/architecture/…/ARCHITECTURE-SPINE.md:827` — the `keyring`
  Stack row, dated 2026-08-02 from crates.io rather than from downloaded source. The NFR15
  round record goes below, in the shape of rounds five through eight.
- `_bmad-output/implementation-artifacts/deferred-work.md` — append at EOF only; the FR65 entry
  owned by this story closes in words, never by deletion.

## Tasks & Acceptance

**Execution:**

- [x] `ARCHITECTURE-SPINE.md` — record NFR15 round nine with the measured subtree and the five
      licence bodies read in the downloaded source, add the `keyring-core` row, and correct the
      `keyring` Stack row's provenance — rationale: the existing ✓ came from a registry label,
      which `AGENTS.md:12` forbids as evidence, and the row predates the crate ever being called.
      *(Phase 1, 2026-09-17.)*
- [x] `src-tauri/Cargo.toml` — add `[dev-dependencies]` with `keyring-core = "=1.0.0"` and a
      comment naming why it is test-only — rationale: it is the project's first such section, so
      the reason must sit where the next reader will look. *(Phase 1, 2026-09-17.)*
- [x] `src-tauri/src/core/aiconfig/keychain.rs` — secret newtype with redacting `Debug`/`Display`,
      account naming, set / delete / `configured`, and the Story 4.8 read — rationale: one
      module owning the raw value is what makes the boundary test able to say anything.
      *(Phase 1, 2026-09-17. `set`/`delete`/`configured`/`read` are `pub(crate)`; the raw-value
      accessor `ApiKeySecret::expose_secret` and `read` are documented as restricted to
      `core/aiconfig/**` today, but that restriction still needs Phase 3's scanning test — Rust
      visibility alone does not enforce it against another same-crate module.)*
- [x] `src-tauri/src/core/aiconfig/store.rs` + `mod.rs` — new error variants and their `IpcError`
      conversions, and the key's own validator — rationale: rejection stays in the domain, and
      `IpcError::new` is the only constructor. *(Phase 1, 2026-09-17: `AiConfigKeyError` — a new
      enum, not new variants on `AiConfigStoreError` — plus `InvalidKeyValue`/`validate_key`.)*
- [x] `src-tauri/src/core/i18n/mod.rs` + `src/i18n/vi.json` — the three new error keys
      (invalid key, keychain unavailable, key-is-global) with their params, plus the section
      strings: the two status readings, save, delete, and the sentence saying one key serves
      every Work — rationale: the catalogue is closed, so an error cannot be reported without an
      entry, and the Work sentence is what stops the key row from reading like the five rows
      above it that do write the Work tier. *(Phase 1, 2026-09-17: the `core/i18n/mod.rs` half —
      three `message_keys!` entries, no params. The orchestrator closed the `vi.json` error-key
      half before Phase 2 started (the three error strings), bringing `ipc_contract` to 29/29.
      Phase 4, 2026-09-17, closed the remaining section-strings half: eight new
      `settings.ai_config.key_*` keys — label, the "one key serves every Work" sentence, three
      status readings (configured/not configured/unknown), invalid-value message, save, delete.)*
- [x] `src-tauri/src/commands/aiconfig.rs` + `src-tauri/tests/config_invariants.rs` — save / delete
      commands and the status field on `AiConfigGetWire`, plus the census row — rationale: a new
      command file row that does not match its census fails the gate. *(Phase 2, 2026-09-17:
      `ai_config_save_key`/`ai_config_delete_key` (Global-only refusal at the command layer) plus
      `AiConfigGetWire.key_configured: Option<bool>`, and the `COMMAND_FILE_CENSUS` row updated to
      5 plain / 0 async. Checkbox left unticked by Phase 2's own handoff and flagged by Phase 3 as
      an oversight to fix downstream — ticked now, no behavior change.)*
- [x] `src-tauri/src/core/ai/mod.rs` — 🔵 in-place correction of the line placing `keyring` in
      `core/ai/` — rationale: a claim that stops being true gets fixed, not deleted.
      *(Phase 1, 2026-09-17.)*
- [x] `src-tauri/tests/` — a boundary test for the raw-value accessor plus contract cases for
      every I/O matrix row, including the keychain-refuses row via an injected failure, and a
      guard asserting the mock store actually took effect — rationale: a pure-layer test alone
      would prove the rules and not the wiring, and a silently-ignored mock would exercise
      nothing while staying green. *(Phase 3, 2026-09-17: `tests/aiconfig_keychain_boundary.rs`
      new (6 cases) for the raw-value accessor boundary; `tests/aiconfig_contract.rs` gained 12
      cases for the I/O Matrix rows plus the mock-swap guard shared by the whole binary. See
      `4-3-phases-2026-09-17.md` Phase 3 for the full account.)*
- [x] `src/config/aiconfig.ts` + `src/aiConfigState.ts` + `src/SettingsOverlay.vue` — key status
      state, save and delete, and the key row — rationale: the save path re-validates because
      Enter bypasses a disabled button. *(Phase 4, 2026-09-17: `aiConfigSaveKey(value)`/
      `aiConfigDeleteKey()` in `config/aiconfig.ts` take no `tier` param — Global-only is baked
      into the adapter, Rust's own refusal stays as defense-in-depth. `aiConfigState.ts` gained
      the tri-state `aiConfigKeyConfigured` (`boolean | null`), draft/busy/error refs, and
      `saveAiConfigKey`/`deleteAiConfigKey`. `SettingsOverlay.vue` gained a sixth row
      (`.ai-key-field`, not part of the five-field `v-for`) with three status readings and a
      Delete button gated on `!== false` (offered on both `true` and unknown `null`, per Phase
      2's note). See `4-3-phases-2026-09-17.md` Phase 4 for the full account.)*
- [x] `tests/frontend/aiConfigState.test.ts` — cases proving the key never appears in any
      response type the state module reads — rationale: this is the one claim a type change
      could silently break. *(Phase 4, 2026-09-17: 16 new cases in `aiConfigState.test.ts`
      (validator, tri-state, save/delete, reset, and a decoy-response regression proving no
      accessor exposes a `key`/`value`/`api_key`-shaped field even if one arrived), plus a new
      file `tests/frontend/settingsOverlayAiConfigKeyRender.test.ts` (4 cases) mounting
      `SettingsOverlay.vue` for real to close the two UI-only I/O Matrix rows — "No key
      anywhere" and "Save a key" — and to prove `key_configured: null` renders a third,
      distinct sentence rather than falling back to "not configured".)*
- [x] `deferred-work.md` — close the FR65 entry in words, and record what the two decisions in
      §Intent leave open: the mockup's tier marker on the key row, which Global-only departs
      from, and the process-global mock swap shared by the whole test binary — rationale: the
      ledger is the evidence for the next decision, and a departure recorded nowhere reads later
      as an oversight.

**Acceptance Criteria:**

- Given the whole repository after this story, when every `#[tauri::command]` signature and
  every type reachable from one is read, then none can carry the key value — and removing the
  boundary seam turns an existing case red.
- Given a key saved through the settings section, when `global.db`, `project.db`, the `.atproj`
  directory and captured stdout/stderr are searched for the key, then it appears in none of them.
- Given `ai_boundary.rs` on the tree as it stands after this story, when it runs, then it is
  green **without** its exemption predicate or forbidden-token list being edited, and `AI_FLOOR`
  is still 1.
- Given `Cargo.lock` before and after this story, when the `[[package]]` entries are compared,
  then the set is identical — the only change is `"keyring-core"` joining the `auratranslate`
  dependency list.
- Given a Work open and a key saved from the settings section, when the Global and Work tiers are
  inspected, then exactly one keychain entry exists and it is the Global one.
- Given the gates and suites after this story, then `cargo test --locked` and `npx vitest run`
  are green with the new cases counted, and the eleven `pre-push` gates exit 0 — including
  `check:deps` with no edit to its banned list.

## Implementation Notes

Built in four phases through `4-3-phases-2026-09-17.md`, per `AGENTS.md:17`. What follows is
what the orchestrator verified at the source, not what the phase reports claimed.

**Shape as built.** `core/aiconfig/keychain.rs` holds `ApiKeySecret` (hand-written
`Debug`/`Display`, both fixed to `<redacted>`) plus `set`/`delete`/`configured`/`read`, none of
which takes a tier — the key is one entry for the whole app, and the Work-tier refusal lives in
`commands/aiconfig.rs` so a request reaching the command layer is rejected rather than merely
un-offered by the UI. Every keychain call maps its error with `map_err(|_| KeychainUnavailable)`,
discarding the source: there is no path by which the underlying error's `Debug` could carry the
value. Measured: `expose_secret` appears exactly three times in `src-tauri/src/**` — the
doc-comment, the definition, and one unit test — all inside `core/aiconfig/keychain.rs`.

**The one defect that had to go back.** Phase 2 first wrote
`keychain::configured().map_err(..)?` inside `ai_config_get`, so a keychain that refused to
answer sank the whole call. Combined with `src/SettingsOverlay.vue:234`, whose
`v-if="aiConfigLoadError !== null"` swallows the section body, that made one refusal collapse
the whole `ai_and_model` section — the five plaintext fields unreadable and unsaveable. That is
the second half of the frozen matrix row ("the section stays usable and other settings still
save") being false. Corrected to `.ok()` with `key_configured: Option<bool>`, the shape
`AGENTS.md:62` mandates for a value that can be UNKNOWN: `None` renders as its own third
sentence (`settings.ai_config.key_status_unknown`), never as "not configured", which this epic
treats as a normal state and would therefore have told the user everything was fine.

**A hazard found by a phase agent falsifying its own claim, and worth more than the fix.** Phase
2 asserted the suite touched the real macOS keychain without raising a prompt; asked how it knew,
it answered that it had only inferred this from the run not hanging, then tested it properly:
with a real entry present, a freshly compiled binary reading it back raised `SecurityAgent` and
the run **hung**. A hang, not a red case — no gate reports it. The consequence neither agent
named: `commands/aiconfig.rs:182` probes the keychain on **every** `ai_config_get`, so the
tripping call is a read, and the ten pre-existing Story 4.2 cases in `aiconfig_contract.rs`
already walk that path. On a machine where the developer had ever saved a real key, today's
suite could hang with no code change at all. That is why the mock store is installed once per
binary before any test runs, not merely before the first `set`. Verified afterwards that no
`SecurityAgent` process survived and that the probe entry was gone from the real keychain.

**Verification the orchestrator ran itself**, not read from a report: `npm run build` clean ·
`npx vitest run` alone, 1128/1128 across 80 files · `cargo test --locked` full suite, exit 0
with zero `FAILED` · all eleven `pre-push` gates exit 0 · `ai_boundary.rs` untouched by the diff ·
`Cargo.lock` changed by exactly one line, `+ "keyring-core",`, with no `[[package]]` added ·
every one of the ten I/O-matrix rows mapped to a named case that ran and passed.

⚠️ **Two measurement mistakes the orchestrator made and corrected, recorded because both are
recurring shapes in this repo.** Checking for a stuck `SecurityAgent` with
`ps aux | grep -i "[S]ecurityAgent"` matched the orchestrator's own command line, which contains
that string — redone with `pgrep -x`. And the first full-suite tally piped through `head -20`,
so the count was taken over a truncated list; the number it produced was meaningless and the
run was re-measured without the cut. Both are the same error: a measuring command that changes
what it measures.

## Spec Change Log

- **Code Map said "The new error variants join this enum" (`AiConfigStoreError`); the
  implementation created a separate `AiConfigKeyError` instead** (`core/aiconfig/store.rs:202`,
  verified at the declaration site rather than taken from the phase report). The reason holds:
  the key never touches the `ai_config` table, so none of `AiConfigStoreError`'s variants apply
  to it, and it carries a `TierNotGlobal` state the five plaintext fields can never reach.
  Keeping them apart also keeps `params` structurally empty on every key-error branch, which is
  what the frozen §Boundaries line about errors never interpolating the key relies on. The Code
  Map line is the part that was wrong, not the code.
- 🔴 **The phase split left the tree red between phases, and that was the orchestrator's defect,
  not the phase agent's — recorded because a red measurement window is exactly the trap this
  repo has been bitten by before.** `4-3-phases-2026-09-17.md` drew Phase 1's boundary by
  directory ("leave all of `src/**` untouched") while Task 5 of this spec bundles
  `core/i18n/mod.rs` and `src/i18n/vi.json` under one checkbox. Phase 1 therefore added three
  `message_keys!` entries with no `vi.json` counterpart and stopped, leaving
  `tests/ipc_contract.rs` at 27/29 — measured, exactly the two catalogue-sync cases and nothing
  else. Had Phase 3 run next, it would have measured its own new tests against an already-red
  suite. Closed by adding the three error strings before handing off, after which
  `ipc_contract` is 29/29; the phase file's boundary was corrected to match Task 5's grouping.

## Review Triage Log

Pass 1 — 2026-09-17, three layers (blind-hunter, edge-case-hunter, verification-gap) over a
227 kB diff. 15 distinct findings after grouping the one both blind-hunter and edge-case-hunter
filed. No `intent_gap` and no `bad_spec`, so no loopback; `review_loop_iteration` stays 0. Every
claim below was re-verified at its cited location by the orchestrator before a verdict was
rendered.

| # | Finding | Verdict | Route | Evidence |
|---|---|---|---|---|
| 1 | The one keychain case that never calls `open_global()` may run before the mock is installed | high | patch | Filed by two layers independently. `install_mock_keychain_store_once()` has exactly one call site, `open_global` at `aiconfig_contract.rs:175`; `a_work_tier_key_request_is_refused_...` at `:704` calls neither it nor `open_global`, only `reset_key_to_not_configured()` (which drives `keychain::delete`) and `inject_one_shot_keychain_error()` (whose `downcast_ref::<mock::Cred>` carries `.expect("mock chua duoc cai")`). Whether the `Once` has fired depends on another test reaching `:175` first. The real-store branch is the hang this story measured, so the bad outcome is a panic or a hang, not a red case. |
| 2 | Registering the two new wires in `generate_handler!` is guarded by nothing | medium | patch | Pre-verified by the verification-gap layer with a removal demonstration: delete `lib.rs:911-913` and both `cargo test --locked` and `npx vitest run` stay fully green, because `aiconfig_contract.rs` imports the pure functions directly and never goes through `wire::`, and every frontend test mocks `src/config/aiconfig` at the module boundary. `ipc_contract.rs` already carries this guard for other domains by name. |
| 3 | A brace or glob `use` of `keychain::read` evades the boundary gate | medium | patch | `FORBIDDEN_RAW_VALUE_TOKENS` is `["expose_secret", "keychain::read"]`; `use ...::keychain::{read, configured};` yields the text `keychain::{read` and the call site yields `read(` — neither matches. Distinct from the rename-with-`as` hole the file already discloses. Zero live callers today, so nothing is leaking now; the gate's claim is what is false. |
| 4 | A comment contradicts the assertion two lines under it | low | patch | `settingsOverlayAiConfigKeyRender.test.ts:175` says both buttons stay clickable; `:177` asserts Save's `disabled` is defined. A direct correction, and this repo's rule is that a claim which stops being true gets fixed rather than left to lie. |
| 5 | `err.ai_config.keychain_unavailable` prescribes a macOS-only remedy | low | patch | The string ends "thử lại sau khi mở khoá keychain". `Cargo.lock` carries the Windows and Secret-Service stores, and `keychain.rs`'s error path is platform-agnostic, so the same sentence reaches a user whose Secret Service daemon is simply absent. Copy fix, no logic. |
| 6 | `err.ai_config.key_is_global` says "để ghi vào" on the delete path too | low | patch | One string is returned by both `ai_config_save_key` and `ai_config_delete_key`; nothing was going to be written on the delete refusal. Copy fix. |
| 7 | The shared `keyBusy` guard is tested on one branch only | low | patch | One flag at `aiConfigState.ts:102` guards both actions (`:294` save, `:319` delete). The tests cover save-blocks-save only. An assert exercising one branch of a shared guard says nothing about the other — the same shape this repo has been bitten by before. The fix adds a test, not complexity. |
| 8 | `.atproj` and process output are not covered by any automated case | medium | defer | True and already visible: `saving_a_key_leaves_no_trace_on_disk_in_global_db_or_project_db` reads only the two SQLite files, and §Verification marks the manual check 🟡. The reviewer's point that stands is the second half — it had no ledger entry of its own, so it would vanish when the phase file is archived. Recorded with an owner rather than closed by an end-to-end run that would write a real credential and risk the measured hang. |
| 9 | Every keychain error discards its source, leaving nothing to diagnose with | medium | defer | Verified: all four functions use `map_err(\|_\| KeychainUnavailable)`. Named harm, developer-side: the first real report of "keychain unavailable" in Story 4.8 cannot be told apart from a declined OS prompt, an absent platform store, or anything else. Not a direct correction — what is safe to log from a `keyring::Error` without touching the secret is a decision, and it belongs with the story that first meets a real failure. |
| 10 | `ApiKeySecret` does not zeroize on drop | medium | defer | Real hardening gap: the key sits in process memory for the value's lifetime, exposed to core dumps and swap. The fix needs a new crate, which means the NFR15 licence gate and a Stack-table row — not a change this story can absorb, and the spec never asked for it. |
| 11 | Editing the draft after a failed save leaves the stale error showing | low | defer | Verified real: `setAiConfigKeyDraft` (`aiConfigState.ts:144`) does not clear `keyError`, and the overlay's `v-else-if` keeps rendering it. But `setAiConfigDraft` at `:134` behaves identically for the five plaintext fields and pre-dates this story — patching only the key row would leave six rows with two behaviours. Recorded as one item covering all six. |
| 12 | No maximum length on the key before `keychain::set` | maybe-false | rejected | The claim that an OS keychain caps secret size was asserted, not measured, by either layer, and nothing shows the limit is reachable. If true it would be `low` — a multi-megabyte paste into an API-key box is not everyday use — and the fix adds a guard on state no one has demonstrated. Rejected with the note rather than deferred, per the rule for a `maybe-false` that would only be `low`. What would settle it: write a key of increasing size through `keychain::set` on macOS and Windows and find where it starts failing. |
| 13 | `type="password"` may trigger an OS password-manager prompt | maybe-false | rejected | Stated as something that happens "on some platforms" with no path shown on any platform this app ships to, and `autocomplete="off"` is already set. Would be `low` if real. What would settle it: open the built app on macOS and Windows and see whether a save prompt appears. |
| 14 | A `/* … */` block comment mentioning a forbidden token is flagged as a violation | low | rejected | The behaviour is real — `code_lines` at `aiconfig_keychain_boundary.rs:130` strips only lines beginning `//`. But it fails loudly in the safe direction and matches `ai_boundary.rs`, and `AGENTS.md:68` deliberately wants a source-scanning gate to read code lines rather than trust that a mention is inert. Stripping block-comment spans is added complexity for a case that costs a developer one edit. |
| 15 | Nothing stops a future second keychain test binary repeating the mock blind spot | low | rejected | The concern is real but is already carried, with an owner: the ledger entry recording that the mock store is process-global says in terms that a second binary must install its own. A regression test cannot be written against a binary that does not exist. |

## Design Notes

**Why no table and no migration.** The obvious symmetry — five fields in `ai_config`, so a
sixth row for the key — is exactly what NFR11 forbids, and Story 4.2's ledger entry already
recorded that `ai_config` is "đúng hình dạng SAI cho một bí mật". The second instinct, a
plaintext marker row saying "a key exists", is also unnecessary: the keychain answers that
question directly, and a marker row could disagree with the keychain after the user deletes
the credential outside the app. Not adding a migration step is worth stating as a decision
rather than an omission, because the last five stories that touched the schema each rippled
into the same 6 files and 16 hard-coded version assertions.

**Why the key does not join `AiConfigField`.** Every one of the five variants flows through
`AiConfigFieldWire { value: String }`, a type that crosses IPC with the value in it. Adding
a sixth variant would put the key on that wire by construction, and the code that would need
to special-case it is spread across the resolver, the wire conversion, and the template loop.
A separate status-only shape makes the invariant structural instead of a rule someone must
remember at three call sites.

**The mock-store swap depends on an order, and the order is the whole trick.** Setting
`keyring_core::mock` first does not work: the first `keyring::Entry::new` forces a `LazyLock`
that calls `set_default_store` with the real platform store and overwrites the mock, and both
halves of that were read in the downloaded source rather than inferred. The order that works is
the reverse — touch the real store once so the `LazyLock` is already resolved, then install the
mock, after which no further `Entry::new` re-runs the initializer. Two consequences follow and
neither is optional. The store is process-global, so the swap is shared by every test in the
binary; and if the real store fails to initialize, `Entry::new` returns `NoDefaultStore` forever
and every keychain case afterwards exercises nothing while reporting green — which is why the
guard asserting the swap took effect is a task and not a nicety.

## Verification

**Commands:**

- `cd src-tauri && cargo test --locked --test ai_boundary` — expected: green with no edit to
  the exemption predicate.
- `cd src-tauri && cargo test --locked` — expected: 0 red; record the before and after case
  counts and name the new cases.
  🔵 **Ran 2026-09-17 after the review patches: 1595 green · 0 red · 20 ignored · 60 test
  binaries · 0 `FAILED`** (1593 before the patches; +2 = the brace/glob self-test in
  `aiconfig_keychain_boundary.rs` and `the_aiconfig_key_wires_are_registered` in
  `ipc_contract.rs`). The
  **before** count was not measured and no derived figure is written in its place. What that
  count exists to catch is answered directly instead: `git diff` against the baseline shows
  **zero** removed test functions across `src-tauri/tests/**` and `tests/**` — every change to
  a test file is an addition. New cases: 12 in `aiconfig_contract.rs` (one per I/O-matrix row
  plus the on-disk leak check), 6 in the new `aiconfig_keychain_boundary.rs`, 2 unit tests in
  `core::aiconfig`.
  ⚠️ Two earlier tallies of this same command were wrong and are named so the method is not
  repeated: one piped through `head -20` and summed a truncated list (457); one was read from
  the output file while the run was still in progress (995). A tally is only valid from a
  finished run, read whole.
- `git diff src-tauri/Cargo.lock | grep '^[+-]\[\[package\]\]\|^[+-]name = '` — expected: empty,
  i.e. no `[[package]]` added or removed. The lock's only legitimate change is `"keyring-core"`
  joining the `auratranslate` dependency list.
- `npm run build` then `npx vitest run` — expected: 0 red. Run vitest alone; a concurrent
  `cargo test` has already caused flake.
  🔵 **Ran 2026-09-17 after the review patches: build clean; vitest 1130/1130 across 80 files,
  run alone** (1128 before; +2 cross-action `keyBusy` cases). ⚠️ The first post-patch vitest
  run was accidentally started while `cargo test` was still running — the very contention this
  line warns about. It came back green in 122 s against 54 s alone; the green stands, because
  contention produces spurious reds rather than false greens, but it was re-run alone anyway
  and that is the number recorded.
- `npm run check:deps && npm run check:i18n && npm run check:commands && npm run check:panel-refs && npm run check:tokens && npm run check:debt-owner && npm run check:gates` — expected: exit 0 each.
  🔵 **Ran 2026-09-17: all eleven `pre-push` gates exit 0**, each captured on its own so no
  `exit=$?` was read after a pipe — that mistake has reported a false 0 on a red gate before.

**Manual checks:**

- After saving a key, grep the key string across `global.db`, `project.db`, the `.atproj`
  directory and the captured process output — expected: zero hits in all four.
  🟡 **Half-closed 2026-09-17, and the half that is open is named rather than inferred.**
  Two of the four are closed by an automated case:
  `aiconfig_contract::saving_a_key_leaves_no_trace_on_disk_in_global_db_or_project_db` reads the
  raw bytes of `global.db` and `project.db` after a real save. The other two were **not** run
  through the product, because doing so writes a real credential into the developer's keychain
  and, on an unsigned dev binary, risks the authorization dialog this story measured as a
  silent hang. What stands in their place is structural, not a pass by inference: `keychain::set`
  is the only writer of the value and its sole call is `Entry::set_password`, so no file-writing
  path ever receives it; and `expose_secret` has zero call sites outside
  `core/aiconfig/keychain.rs` (measured, and now gated), while every keychain error is mapped
  with `map_err(|_| KeychainUnavailable)`, discarding the source before anything could print it.
  The remaining gap is an end-to-end run of the real app, and it belongs to the debt entry that
  already owns the dev-binary dialog question.
