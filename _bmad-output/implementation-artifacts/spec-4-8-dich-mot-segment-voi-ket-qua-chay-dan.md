---
title: 'Story 4.8 — Translate one segment with a streamed result'
type: 'feature'
created: '2026-09-21'
status: 'done'
route: 'dispatch'
review_loop_iteration: 0
baseline_commit: '5a23410790642ea43ae0ca3169a49dbef425d517'
context:
  - '{project-root}/_bmad-output/implementation-artifacts/epic-4-context.md'
  - '{project-root}/AGENTS.md'
  - '{project-root}/src-tauri/AGENTS.md'
  - '{project-root}/src/AGENTS.md'
---

<frozen-after-approval reason="human-owned intent — do not modify unless human renegotiates">

## Intent

**Problem:** Everything Epic 4 built so far stops one step short of the network. Story 4.2 stores
the provider config, 4.3 parked the API key in the OS keychain behind a `read()` that carries
`#[allow(dead_code)] // Story 4.8 la cho goi dau tien`, and 4.7 records an assembled prompt that has
never been sent — its own Decision 2 leaves AC1's word *"đã gửi"* unfulfilled on purpose. Measured on
this tree: `TranslationProvider` is still `chưa khai` in `ports/mod.rs:10`, there is no
`core/ai/client.rs`, and `tauri::ipc::Channel` appears **0** times in the whole repository. FR72 and
FR74 are the reason the epic exists, and nothing can yet call a provider.

**Approach:** Declare the third and last port, put one OpenAI-compatible streaming client behind it,
and carry its tokens to the panel over a Tauri Channel. The send path reuses 4.7's producer instead
of assembling again — it records, then sends **the recorded string**, which is what makes FR71's
*"matches 100%"* structural rather than a promise. The result lands in the AI Translation panel and
stays there; a separate, explicit `⌘⇧↵` is the only thing that moves it into the Editor, and that
write declares its provenance as **someone else's translation** (AD-47③).

## Boundaries & Constraints

**Always:**
- The bytes that go out are the bytes 4.7 recorded. The send path calls the existing producer, keeps
  its record, and transmits that exact `prompt` string — never a second assembly, never a
  concatenation at the call site (AD-14).
- Streaming crosses IPC through **one** `tauri::ipc::Channel<T>`, never loose events (AD-22). The
  CSP's `connect-src` keeps `ipc: http://ipc.localhost`; dropping it degrades the Channel's data
  path to postMessage silently (`SECURITY-NOTES.md:132-142`).
- No self-reconnecting SSE client and no automatic retry of any kind, at any layer. With BYOK a
  retried call is the user's money spent twice. A dropped stream ends the call; whatever tokens
  arrived stay on screen.
- Every call is cancellable mid-flight, and a cancelled call sends nothing further on its Channel.
- AI state is exactly one of five values at all times — `not_configured` · `generating` · `done` ·
  `error` · `cancelled`. `not_configured` is **not** an error: the panel invites configuration
  (FR77, `EXPERIENCE.md:136`).
- No AI result reaches `target_text` except through the explicit promote action. That write is a
  non-user write under AD-47①, so it does **both** halves in one logical operation: it resets the
  segment's comparison baseline to the text just written **and** sets `translation_origin` to
  `TRANSLATION_ORIGIN_OTHER`. Forgetting the second half is the silent poisoning AD-47 exists to
  prevent, and no gate goes red for it.
- The API key never crosses IPC, never enters a log line, an error message, an `IpcError` param, or
  a `Debug` output. `ApiKeySecret` carries no `derive(Debug)` and neither may anything holding it.
- A segment carrying `is_omitted` is never sent to a provider, checked before the request is built.

**Never:**
- No new crate and no new Cargo feature. Measured on the pinned source: `Response::chunk()`
  (`reqwest-0.13.4/src/async_impl/response.rs:310`) is **not** feature-gated; only `bytes_stream()`
  (`:351`) sits behind `stream`. The SSE framing is parsed by hand — the spine's CAP-4 row still
  records `reqwest-sse`/`sseer` as un-reviewed under NFR15, and hand-rolled framing is the named
  fallback.
- No batch translation, no batch progress, no batch cancel — Story 4.9.
- No error-copy catalogue, no retry button, no "tokens already received" error wording — Story 4.10.
  This story produces the `error` state and an `IpcError`-shaped failure; 4.10 owns what the reader
  sees.
- No token counts and no cost estimate — Story 4.11.
- No "test connection" button and no probe endpoint — Story 4.2's deferred AC stays deferred
  (Decision 4).
- No second prompt record, no second assembly function, no re-derivation of the prompt pieces.
- No local HTTP server anywhere in the test suite: AD-45 bans a listening port, and a test binding a
  port is the known-false-red this machine has already paid for twice. The port trait is the seam
  tests substitute at.
- Do not touch `assemble_prompt` / `gather_glossary_context` signatures (AD-14, frozen at 4.6).

## I/O & Edge-Case Matrix

| Scenario | Input / State | Expected Output / Behavior | Error Handling |
|----------|--------------|---------------------------|----------------|
| Translate a segment | Work open, caret on a segment, provider config resolved, key present | Record written, request sent, tokens arrive on the Channel and render as they land; state `generating` → `done` | N/A |
| Provider not configured | `endpoint` or `model` empty, or `key_configured == Some(false)` | State `not_configured`; **zero** network calls; panel invites configuration | Not an error — no `IpcError`, no alert |
| Keychain refuses to answer | `keychain::read()` returns `KeychainUnavailable` | State `error`, existing `AiConfigKeychainUnavailable` key | `IpcError`; key value never in `params` |
| Segment cut from the translation | `ChapterSegment.is_omitted == true` | Refused before the request is built; named message key | `IpcError`, `retryable: false` |
| No Work open / segment not in chapter | Caret id absent from the loaded chapter | Refused; reuses `WorkNoneOpen` / `AiPromptSegmentNotInChapter` | `IpcError` |
| Cancel mid-stream | State `generating`, user cancels | Stream stops, state `cancelled`, received tokens stay visible, no further Channel message arrives | N/A |
| Stream ends without `[DONE]` | Connection drops mid-generation | State `error`; received tokens stay visible; **no** reconnect, **no** retry | `IpcError`, `retryable: true` |
| Provider returns a non-2xx | HTTP 401/429/500 | State `error`; no retry | `IpcError` carrying the status |
| Promote the result | State `done` or `cancelled`, panel text non-empty, `⌘⇧↵` | `target_text` written, `translation_origin = "other"`, baseline reset to that text, grid mirrors it | N/A |
| Promote while generating | State `generating` | Refused, no write | Diagnostic only; a chord never throws |
| Caret moves during generation | Another segment focused mid-stream | The call keeps running and lands against the segment it started on; the panel marks the result stale, the way 4.7 already marks a stale record | N/A |

## Decisions

**1. Cancel is built here, not deferred to 4.9.** Ice settled 2026-09-21. This story's ACs name
`đã huỷ` as one of the five states but no AC says *cancel*, so the question was real; what closes it
is that the other three sources agree — AD-22 (*"mọi lời gọi AI huỷ được giữa chừng"*), Story 4.9's
own AC (*"dù đơn lẻ hay theo lô"*), and the mockup, which draws `Esc huỷ` on the single-segment panel
(`key-screen-workspace.html:176`). Deferring it would ship a five-value state machine with one value
nothing can reach, and leave the product's first real AI call uninterruptible for a story's length.
4.9 inherits the mechanism and adds only the batch loop. Cost accepted: one more command, one
generation flag, and the matrix's cancel row.

**2. Both producers stay.** Ice settled 2026-09-21, overriding the reading of spec 4.7's Decision 2
that *"Story 4.8's translate action takes that producer's place"* implies deletion. The measurement
that makes keeping it worth the cost: assembling and inspecting a prompt needs **no** API key and
touches **no** network, so with the button removed, a translator who has not yet configured a
provider loses FR71 entirely — the one diagnostic the epic exists to provide, gone exactly for the
people still setting the thing up. So `ai.prompt.assemble` and its button survive unchanged, and
translate becomes a second producer of the same record. 🔴 The condition that makes two producers
safe rather than two truths: both call **one** recording function — the translate path adds the sent
facts after the send, and adds nothing else. A second code path that builds the record itself is the
drift this record was created to prevent, and a contract case asserts the two paths produce an
identical record for identical inputs.

**3. This story absorbs four debt items; the other eight are re-owned.** Ice settled 2026-09-21,
on the count taken from this tree: **12** open items name Story 4.8 as owner and **0** are closed.
Absorbed, because each sits on a line this story must write anyway: the CRLF split in
`expand_prompt_body` (`deferred-work.md:10653`), the sent-state on `AssembledPromptRecord` (`:10666`),
the `is_omitted` guard before the send (`:10789`), and the keychain gate's blindness to a renamed
import (`:10441`). Re-owned with a named owner rather than carried: the per-sentence Glossary reload
measurement (`:4540`), the second mock-keychain binary (`:10485`), the two still-manual "no key on
disk" checks (`:10497`), what is safe to surface from a `keyring::Error` (`:10513`), the Global-tier
prompt-set write on the hot path (`:10552`), the real per-call query counter (`:10642`), the dead
`lib.rs` half of 4.7's exemption (`:10840`), and the "test connection" AC (`:10240`, see Decision 4).
Reason for the line: two of the eight are pure **measurement** work with no user-visible result, and
absorbing all twelve roughly doubles a story that is already the epic's largest.

**4. No "test connection" here.** Ice settled 2026-09-21. `deferred-work.md:10240` parked Story 4.2's
deferred AC on *"the first story that can make a real call out of `core/ai/**`"*, which is literally
this one — but the button lives on the Settings screen, not in the Workspace, and it is reviewable,
testable and mergeable on its own. It is therefore a separate deliverable, and taking it in would
make this story multi-goal by the project's own definition. It stays a debt item and gets a new owner
in Phase 4. `epics.md` is not edited: a capability not yet built is not a spec mismatch.

**5. The full spec stands at 9.432 tokens** (tiktoken `o200k_base`, measured on the approved text;
the figure shown to Ice at the gate was 9.074, before Decisions 1–4 were written in). Far above the
1.600 mark. Measured on one scale — every Epic 4 spec's current body with the three append-only logs
stripped — the neighbours are 4.2 = 5.597 · 4.3 = 6.645 · 4.4 = 5.303 · 4.5 = 8.098 · 4.6 = 6.414 ·
4.7 = 11.920, so this sits inside their band rather than outside it. ⚠️ Those are **not** the figures
spec 4.7 §Decision 4 quotes for the same files; that entry measured 4.4 and 4.5 before their review
loops grew them, and its own 7.677 was taken before 4.7's last two loops. Ice chose **keep full
spec** 2026-09-21. It is ONE user-facing goal across four layers, and after Decision 4 removed the
Settings-screen probe, the only remaining cut would be the promote half — which would ship a panel
whose result cannot be used and push this story's own sixth AC into debt. The context-rot risk is
real and is paid the way root `AGENTS.md` requires: four phases, each handed to a fresh agent through
this file, no agent reading the whole story.

</frozen-after-approval>

## Code Map

**The producer this story sends from — reuse, do not duplicate**

- `src-tauri/src/commands/aiprompt.rs:307` `pub type LastAssembledPromptState =
  std::sync::Mutex<Option<AssembledPromptRecord>>`; `:294-300` the record —
  `{ prompt, ledger, segment_id, chapter_id, prompt_set_name, prompt_set_tier }`. **No timestamp, no
  model name, no sent flag**: `deferred-work.md:10666` asks this story to widen *this* record rather
  than open a second one. `:355` `assemble_and_record_prompt(...)` is the producer to call; `:415`
  `read_last_assembled_prompt`; `:435` `clear_last_assembled_prompt_on_work_close`.
- `src-tauri/src/core/ai/rag.rs:482` `assemble_prompt(body, sentence, glossary, tm) -> (String,
  InjectionLedger)` and `:200` `gather_glossary_context(...)` — **frozen signatures** (AD-14).
  `:158-161` `PromptPiece { kind, text }`, `:138-150` `PromptPieceKind::{Authored, Glossary,
  SourceSegment, Tm}`; the concatenation-equals-`prompt` assert at `:504-508` must keep holding.
- ⚠️ `core/ai/rag.rs` `expand_prompt_body` splits on `'\n'` only, so a CRLF-authored body imported by
  Story 4.5 leaves an orphan `\r` when a marker line is removed (`deferred-work.md:10653`). This
  story is the first to put those bytes on a wire; fix it where it is, do not normalise at the send.

**Where the client may live — the gate already says so**

- `src-tauri/tests/webimport_boundary.rs:265` `path_is_allowed_for_reqwest(rel)` = `core/webimport/`
  **or `core/ai/`**, and `:303` is a positive control naming `core/ai/client.rs` by hand. So the
  client's home is already legal and already guarded; no gate edit is needed for `reqwest` itself.
- `src-tauri/src/ports/mod.rs:10` `| `TranslationProvider` | chưa khai | Epic 4 *(module AI)* |` —
  declare it here. AD-2 allows exactly three ports and this is the third; a **fourth** would need a
  new AD. Existing shapes to copy: `dict_source.rs`, `project_store.rs`.
- `src-tauri/Cargo.toml:76` `reqwest = { version = "=0.13.4", features = ["blocking"] }`, and
  `:68` already records it as *"core::ai (điểm ra mạng 1) … AD-15"*. 🔴 The async client, not the
  blocking one: `read_timeout` exists only on the async builder
  (`reqwest-0.13.4/src/async_impl/client.rs:1456`) and appears **0** times in `src/blocking/client.rs`.
  A blocking client can only cap the *whole* request, which cannot tell a long answer from a stalled
  one. Do **not** copy `core/webimport/fetcher.rs`'s client pool, its 20 s `REQUEST_TIMEOUT`, its
  redirect loop or its allowlist — AD-41 is web-import's rule, and the AI endpoint is one host the
  user typed in themselves.

**Config, key, and the segment being translated**

- `src-tauri/src/core/aiconfig/store.rs:58` `resolve_two_tiers(resolver, global, work)`; the field set
  is closed at five — `core/aiconfig/mod.rs:48-56` `AiConfigField::{Provider, Endpoint, Model,
  Temperature, MaxTokens}`. Read through `ALL`; do not re-spell the names.
- `src-tauri/src/core/aiconfig/keychain.rs:140` `read() -> Result<Option<ApiKeySecret>,
  KeychainUnavailable>`, `#[allow(dead_code)]` with the comment naming this story. `:52`
  `ApiKeySecret` — **no `derive(Debug)`**, and nothing wrapping it may derive one either. Removing
  the `#[allow]` is part of the work; leaving it is a lie about the call graph.
- 🔴 `src-tauri/tests/aiconfig_keychain_boundary.rs:81` `FORBIDDEN_RAW_VALUE_TOKENS =
  ["expose_secret", "keychain::read"]`, enforced by `:243`
  `no_file_outside_core_aiconfig_calls_the_raw_value_accessor_of_the_api_key`. The file's own
  doc-comment says widening this exemption is **this story's** decision. `deferred-work.md:10441`
  adds that the scan misses `use …keychain::read as fetch;` — close that in the same pass, or the
  exemption is guarded by a gate that a rename walks straight through.
- `src-tauri/src/commands/segment.rs:1050` `read_open_chapter_segments(open)`; the row carries
  `is_omitted` (`:282-292`), which is the flag the pre-send guard reads — the row is already in hand,
  so the guard costs one field access, not a query.
- `src-tauri/src/commands/segment.rs:2022-2033` the closed origin catalogue:
  `TRANSLATION_ORIGIN_NONE = ""`, `_SELF = "self"`, **`_OTHER = "other"`**, `_BILINGUAL_IMPORT`.
  AD-47③ assigns *"Đưa đề xuất AI sang Editor"* → **người khác dịch** → `_OTHER`. ⚠️ No existing
  `target_text` writer sets this column: `save_segment_targets:1892`, `flush_segment_targets:2613`
  and `restore_segment_version:733` all leave it alone, and only `confirm_segment:2383` writes it.
  The promote path therefore needs its **own** write, not a reuse of `save_segment_targets`.

**Streaming transport and cancellation — no local precedent, two structural ones**

- `tauri::ipc::Channel` is used **0** times in `src-tauri/**`, `src/**` and `tests/**`. Tauri is
  pinned at `=2.11.5` with `@tauri-apps/api` 2.11.1, so it is the v2 shape (`new Channel()` on the TS
  side, `Channel<T>` + `.send()` on the Rust side). `src-tauri/capabilities/main.json` already grants
  `core:event:default` and `core:resources:default`, annotated in place as being *for* Channel
  (AD-22) — nothing to add to the permission set, and adding one would be an architectural decision.
- `src-tauri/src/commands/project/wire.rs:459-520` — the `#[tauri::command(async)]` mechanism written
  out: the macro wraps the sync body and hands it to `async_runtime::spawn`, i.e. a tokio worker
  thread. There is **no** literal `async fn` command in the tree today.
- `src-tauri/src/commands/project/mod.rs:1785` `ImportScanGeneration(Arc<AtomicU64>)` with `next()`
  and `is_current(generation)` — the cooperative supersede pattern to copy for cancel: the streaming
  loop re-checks between frames, and a newer generation makes the older one stop writing.
- 🔴 Lock mutexes only as `lock().unwrap_or_else(std::sync::PoisonError::into_inner)`; `panic = "abort"`
  makes an `unwrap()` process death with no WAL flush.

**The IPC seam through AD-13**

- `src-tauri/tests/ai_boundary.rs:99` `FORBIDDEN_BARE_TOKENS = ["crate::core::ai", "super::ai"]`;
  `:111` `AI_PROMPT_SEAM_COMMAND_FILE = "commands/aiprompt.rs"` (exact path, not prefix); `:163`
  `AI_PROMPT_SEAM_COMMAND_FILE_MARKER = "crate::core::ai::rag::"`; `:190-200`
  `ALLOWED_AI_RAG_NAMES_IN_COMMAND_SEAM` — nine names, scanned by `:262`. ⚠️ The marker admits the
  **`rag`** path only, so a call to `core::ai::client` does not pass through today's exemption
  whichever file makes it. Widen by name, never by file: `:172-189` records that a whole-file
  exemption was already tried and closed (finding V4). Floors: `:65` `AI_FLOOR = 1`, `:89`
  `SRC_RS_FLOOR = 68`, both `>=`.
- `src-tauri/tests/ai_boundary.rs` also still carries 4.7's `lib.rs` half of the exemption
  (`AI_PROMPT_SEAM_LIB_RS_MARKER` / `_FILE`), which `deferred-work.md:10840` reports as never having
  fired. Whether this story writes `crate::core::ai` into `lib.rs` is the measurement that closes
  that item either way — state which happened.

**Gates that count things, and will be wrong the moment a file moves**

- `src-tauri/tests/config_invariants.rs:1481` `COMMAND_FILE_CENSUS` (14 rows) and `:1697-1699`
  `assert_eq!((tree_plain, tree_async), (67, 28))`. A new command file fails the `unclassified`
  assert at `:1685` first. **Re-count both numbers; do not adjust them until they match.**
- `src-tauri/tests/ipc_contract.rs:1895`
  `the_ai_prompt_wires_are_registered_and_keep_their_parameter_names` — the per-feature registration
  test to copy; there is no single global one. `:232` `every_message_key_exists_in_vi_json` runs over
  `MessageKey::ALL`, so a new variant is falsely green until `vi.json` carries its key.
- `src-tauri/src/core/i18n/mod.rs:100` `message_keys!` — the closed catalogue, 101 variants, eight
  beginning `Ai` and none about a provider call. Declare required params inline; never a parallel list.
- `src-tauri/src/lib.rs:712` `generate_handler!`, Epic 4 entries at `:908-935`; managed state at
  `:1174-1206`.

**Webview**

- `src/panels/AiTranslationPanel.vue` — `.ai-surface` (the empty `<div ref="surface">`) is where the
  streamed text belongs; `.ai-inspector-bar` holds 4.7's summary line, the stale notice, and the two
  buttons. The panel already imports `editorCaretSegmentId` directly
  (`src/panels/editorPanelState.ts:121`) — that is the established way a non-Editor panel reads the
  focused segment; do not add an indirection.
- `src/aiPromptInspectorState.ts` — the module shape to copy: readonly refs, one `{value, error}`
  adapter call per action, and `resetAiPromptInspector()` at `:246`, wired into the two Work-change
  reset clusters at `src/modes/libraryChapters.ts:291` and `src/modes/libraryImport.ts:423`. A new
  state module must join **both** clusters; `scripts/check-panel-refs.mjs` fails a module-level ref in
  `src/**/*.ts` that no `reset*()` touches.
- `src/config/aiprompt.ts:234-235` the command-name constants and the `{value, error}` adapter idiom,
  including the doc-comment at `:270-284` explaining why "nothing recorded yet" and "a real failure"
  must stay distinguishable. The new adapter also has to construct the TS `Channel` and hand it to
  `invoke` — the only part of the idiom with no precedent in this repo.
- `src/commands/index.ts:3298-3412` the Epic 4 registrations, all `keys: undefined`.
  `src/commands/keys.ts:92-95` reserves `⌘⇧↵` **by name** for UX-DR35 — *"`⌘⇧↵` đưa bản dịch AI sang"* —
  and `:390-395` plus `index.ts:1070`/`:2182-2184` record that `Mod+Shift+…` was kept free for it.
  This story is its first consumer. Chord grammar is platform-neutral: **`'Mod+Shift+Enter'`**, never
  `Meta`/`Cmd`. 🔵 **CORRECTED 2026-09-21 — the earlier line here read the mockup's `⌘↵` as
  `'Mod+Enter'` for translate, and that is wrong.** Measured on this file before the change:
  `'Mod+Enter'` is `editor.confirm_segment`'s (`:2476`, Story 2.5) and `'Escape'` is
  `editor.clear_source_cuts`'s (`:2827`, Story 2.9). `createKeymap` **throws** on a collision at
  `installCommands()` time, so following the old line would have crashed the app at startup rather
  than turning a gate red. Translate and cancel therefore ship `keys: undefined` (Decision 7).
- The promote write follows `src/panels/segmentHistoryState.ts:273` `restoreVersion`: Rust performs
  the write, then the frontend mirrors it with `replaceEditorSegment`
  (`src/panels/editorPanelState.ts:248`). That mirror is not cosmetic — the confirm baseline is read
  from the loaded snapshot at `:1150` (`confirmSegment(id, loaded.target_text)`), so mirroring the
  promoted text **is** AD-47①(a).
- `@click` must be exactly one `dispatch('<id>')` (`check:commands` Kiểm A); every template text node
  goes through `t()` (`check:i18n` Kiểm A2); `vi.json` stays flat.

**The Channel is testable — measured on the pinned source, added after Phase 1**

- `tauri-2.11.5/src/ipc/channel.rs:213` `pub fn new<F: Fn(InvokeResponseBody) -> crate::Result<()> +
  Send + Sync + 'static>(on_message: F) -> Self` sits on `impl<TSend> Channel<TSend>` — **no
  `Runtime`, no `AppHandle`**, and `:292` `send()` calls that closure directly. So a test constructs
  a real `Channel` from a closure that collects what it is sent, and every matrix row — including
  *"cancel: no further Channel message arrives"* — is checkable with no webview and no socket.
  `:62` it is `Clone`, so it moves into a spawned task; `:300` it implements `CommandArg`, so the
  `mod wire` shell takes it as a parameter straight from the JS side.
- Consequently the port stays ignorant of Tauri: `ports/translation_provider.rs` takes
  `on_token: &mut dyn FnMut(&str)`, and the command layer is the only place that owns a `Channel`.

**Where the API key is read — settled during Phase 1, and one stale claim it leaves**

- The key is read in `commands/aitranslate.rs` and passed into the port as a bare `&str`; `core/ai/`
  names neither `keyring` nor `core::aiconfig`. Reason, measured: AD-13's rule names exactly three
  legal reverse edges — *"`ai/` đọc `glossary/`, `tm/`, `segment/` để phục vụ FR70"* — and
  `aiconfig` is not among them, because it did not exist when AD-13 was written. Letting
  `core/ai/client.rs` read the key would add a fourth reverse edge, which root `AGENTS.md` says is a
  new AD in the spine, not a line of code. The gate that gets checked here covers `glossary::` only
  (`ai_boundary.rs:1008`), so the compiler and the suite would both have stayed green — the
  invariant, not a test, is what rules this out.
- 🔴 `core/ai/mod.rs:14-16` predicts the opposite — *"`core/ai/` (Story 4.8) sẽ ĐỌC khoá qua
  `core::aiconfig`"*. Its load-bearing half (*"không tự mở keychain"*) is satisfied even more
  strictly than it asked, but the sentence as written is now false. Fix it in place with 🔵 and the
  date; do not delete it.
- ⚠️ `TranslateRequest` carries the exposed key and therefore has **no** `derive(Debug)`, by the
  same rule that bars one on `ApiKeySecret`. Nothing that holds it may derive one either.

**Tests that move**

- New `src-tauri/tests/ai_translate_contract.rs` (SSE frame parsing as a pure function, the matrix
  rows, the wire shape) · `src-tauri/tests/ai_boundary.rs` · `src-tauri/tests/aiconfig_keychain_boundary.rs` ·
  `src-tauri/tests/config_invariants.rs` · `src-tauri/tests/ipc_contract.rs` · new
  `tests/frontend/aiTranslate*.test.ts`. `tests/frontend/aiPromptInspector.test.ts` is the example
  that mounts the real component and dispatches through the real registry;
  `tests/frontend/glossaryIpcBridge.test.ts:42` is the example that mocks `@tauri-apps/api/core`
  rather than the adapter module.

## Tasks & Acceptance

Four phases, each handed to a **fresh agent** through this file (root `AGENTS.md`: one agent must not
implement a whole story).

**Execution:**

*Phase 1 — the port and the three gates that must move before any code*
- [x] `src-tauri/src/ports/mod.rs` — declare `TranslationProvider` (streaming translate + cancel),
      replacing the `chưa khai` row with the real trait and keeping the table honest — rationale:
      AD-2's third and last port; a client written before the port would invert the dependency the
      port exists to state.
- [x] `src-tauri/tests/ai_boundary.rs` — widen the seam by **name**: the client-facing identifiers the
      command layer may name, plus a marker admitting `crate::core::ai::client::`, plus the negative
      control that a neighbouring path does not match and the positive control that deleting
      `core/ai/` with its seams leaves the rest compiling — rationale: today's marker admits `rag`
      only, and a whole-file exemption was already tried and closed (V4).
- [x] `src-tauri/tests/aiconfig_keychain_boundary.rs` — add the named exemption for the one new caller
      of `keychain::read`, **and** close the renamed-import hole (`deferred-work.md:10441`) with a
      seeded violation proving the new predicate fires — rationale: an exemption whose gate a rename
      walks through is not a boundary.

*Phase 2 — Rust: the client, the send path, the Channel*
- [x] `src-tauri/src/core/ai/client.rs` (new) — the `TranslationProvider` implementation: build the
      OpenAI-compatible chat-completions request with `stream: true`, drive the async `reqwest`
      client with `read_timeout` between frames, and parse SSE framing **as a pure function** over a
      byte buffer returning `(events, remaining)` — rationale: the pure split is what lets the matrix
      be tested without a socket, which AD-45 and this machine's firewall both require.
- [x] `src-tauri/src/commands/aitranslate.rs` (new) — the two-layer shape: a pure function taking
      `Option<&Store>`/`Option<&OpenWork>` that resolves config, reads the key, refuses an omitted
      segment, calls 4.7's producer, and streams through a `Channel<T>`; plus `mod wire` shells using
      `try_state`, never `state()` — rationale: `tests/**` must reach every branch without a webview.
      🔴 It calls 4.7's recording function; it does not build a record of its own (Decision 2).
- [x] `src-tauri/src/commands/aitranslate.rs` + cancel state — a generation counter in the
      `ImportScanGeneration` shape, checked between frames, so cancel stops the loop and nothing more
      is sent on the Channel — rationale: AD-22, and the `cancelled` state has to be reachable.
- [x] `src-tauri/src/commands/aiprompt.rs` — widen `AssembledPromptRecord` with the sent facts (when,
      and which model), set only by the send path — rationale: `deferred-work.md:10666` asks for this
      record to gain them, explicitly not a second record.
- [x] `src-tauri/src/core/ai/rag.rs` — fix `expand_prompt_body`'s `'\n'`-only split so a CRLF body
      leaves no orphan `\r` — rationale: `deferred-work.md:10653`; this is the first story that sends
      those bytes anywhere.
- [x] `src-tauri/src/core/aiconfig/keychain.rs` — drop `#[allow(dead_code)]` from `read` now that it
      has a caller — rationale: the attribute is a claim about the call graph.
- [x] `src-tauri/src/core/i18n/mod.rs` — add the `message_keys!` variants this story actually needs
      (omitted segment; provider call failed), reusing `WorkNoneOpen`, `AiPromptSegmentNotInChapter`
      and `AiConfigKeychainUnavailable` where the fact already has a key — rationale: one key per
      fact; a variant outside `ALL` is falsely green.
- [x] `src-tauri/src/lib.rs` — manage the cancel/generation state and register the new wires; state in
      §Implementation Notes whether `crate::core::ai` now appears here, closing
      `deferred-work.md:10840` in one direction or the other — rationale: that item is waiting on
      exactly this measurement.
- [x] `src-tauri/src/commands/segment.rs` — a promote write that sets `target_text` **and**
      `translation_origin = TRANSLATION_ORIGIN_OTHER` in one operation, beside the existing origin
      catalogue rather than as a flag on `save_segment_targets` — rationale: AD-47①'s two halves;
      the three existing `target_text` writers all leave the column alone, so reusing one of them
      would stamp the translator's name on the model's sentence.
- [x] `src-tauri/src/core/ai/client.rs` — mark the `Authorization` header value **sensitive**
      (`HeaderValue::set_sensitive(true)`) instead of passing a plain `format!("Bearer {…}")`
      string — rationale: §Always bars the key from any `Debug` output, and an unmarked
      `HeaderValue` prints its contents; `Sensitive` is what makes that structural rather than a
      promise that nothing downstream ever debug-prints a request.
- [x] `src-tauri/tests/config_invariants.rs` — add the `COMMAND_FILE_CENSUS` row and **re-count**
      `(tree_plain, tree_async)` — rationale: the numbers are declarations, not adjustments.
- [x] `src-tauri/tests/ai_boundary.rs` — **carried from Phase 1, which could not do it:** the third
      seam is admitted today by its `crate::core::ai::client::` prefix alone. Now that
      `core/ai/client.rs` exists, freeze the identifiers the command layer may name from it, in the
      `ALLOWED_AI_RAG_NAMES_IN_COMMAND_SEAM` shape, and extend the `#[ignore]`d compile probe to
      strip `commands/aitranslate.rs` and its `commands/mod.rs` declaration — rationale: a
      prefix-only exemption is the V4 mistake in a second shape, and Phase 1 deliberately did not
      freeze invented names against a file that did not exist.
- [x] `src-tauri/src/core/ai/mod.rs` — fix lines 14-16 in place with 🔵 and the date: the key is read
      in the command layer, not through `core::aiconfig` from inside `core/ai/` — rationale: root
      `AGENTS.md`, a claim that stops being true gets fixed in place, never left to lie quietly.

- [x] `src-tauri/src/core/ai/client.rs` + `src-tauri/src/commands/aitranslate.rs` — make
      `temperature`/`max_tokens` `Option` and omit them from the request body when unset, so a blank
      generation parameter is not a reason to call the provider unconfigured (Decision 6) —
      rationale: the matrix's `not_configured` row names `endpoint`/`model`/key and nothing else.

*Phase 3 — Webview: the panel, the state, the three commands*
- [x] `src/config/aitranslate.ts` (new) — the adapter: construct the TS `Channel`, hand it to
      `invoke`, keep the `{value, error}` shape, never throw — rationale: the Channel construction is
      the one part of the adapter idiom with no precedent in this repo.
- [x] `src/aiTranslateState.ts` (new) — the five-value state machine, the accumulating text, the
      stale marker when the caret leaves mid-stream, and `resetAiTranslate()` — rationale: five states
      as one value makes an impossible pair unrepresentable instead of merely untested.
- [x] `src/modes/libraryChapters.ts` + `src/modes/libraryImport.ts` — join `resetAiTranslate()` to
      both existing reset clusters — rationale: a result labelled with a Work the app has closed is
      the known-bad state 4.7 already hit once.
- [x] `src/commands/index.ts` + `src/i18n/vi.json` — register translate (`'Mod+Enter'`), cancel, and
      promote (`'Mod+Shift+Enter'`), with their labels — rationale: `⌘⇧↵` has been reserved by name
      for this story since `keys.ts:92`. ⚠️ **Measured and changed for two of the three — see
      Implementation Notes: `'Mod+Enter'`/`'Escape'` are both already claimed elsewhere in this
      registry, and `createKeymap` throws on a collision.**
- [x] `src/panels/AiTranslationPanel.vue` — render the streamed text in `.ai-surface`, show the state,
      and offer cancel and promote as single-`dispatch` `@click`s — rationale: `check:commands` Kiểm A
      sees `@click` only, so a second path would be invisible to it.
- [x] `src/config/segment.ts` + `src/panels/editorPanelState.ts` — call Phase 2's promote write and
      mirror the result with `replaceEditorSegment`, following `restoreVersion` — rationale: the
      mirror is not cosmetic, it **is** AD-47①(a): the confirm baseline is read from that snapshot.

*Phase 4 — Tests that move*
- [x] `src-tauri/tests/ai_translate_contract.rs` (new) — one named case per matrix row; the SSE parser
      driven with split frames, a frame arriving in two chunks, a `[DONE]` terminator, and a stream
      that ends without one — rationale: the split-frame case is the one a happy-path fixture cannot
      fail on.
- [x] `src-tauri/tests/ipc_contract.rs` — the registration test for the new wires, in the
      `the_ai_prompt_wires_are_registered…` shape — rationale: there is no global registration test.
- [x] `tests/frontend/aiTranslate*.test.ts` (new) — mount the real panel, dispatch through the real
      registry, drive a fake Channel through each of the five states, and assert promote is refused
      while generating — rationale: a case that passes in two states guards neither.
- [x] `src-tauri/tests/ai_translate_contract.rs` — the two-producer identity case required by
      Decision 2: assemble-only and translate must write an **identical** record for identical
      inputs, differing in the sent facts alone — rationale: two producers are safe only while there
      is one recording function, and only a case can keep that true.
- [x] `src-tauri/tests/ai_translate_contract.rs` — a case asserting the serialized request body
      **omits** `temperature`/`max_tokens` when they are `None` and includes them when set, and a
      case that a blank generation parameter does **not** yield `NotConfigured` (Decision 6) —
      rationale: `skip_serializing_if` is invisible to the type checker, and Decision 6 exists
      because the previous behaviour locked a correctly-configured user out with no explanation.
- [x] `src-tauri/tests/ai_rag_contract.rs` — a permanent CRLF regression case for `pop_piece`, and
      with it the case that `pieces` still concatenates to `prompt` byte for byte on a CRLF body —
      rationale: Phase 2a verified the fix with scratch tests and then deleted them, so the absorbed
      debt item `deferred-work.md:10653` has **no** standing guard; and `pop_piece` pops `'\r'` from
      the `pieces` mirror based on what `out` ends with, not on what that piece ends with, so the two
      can disagree if a `\r` and its `\n` ever land in different pieces.
- [x] `_bmad-output/implementation-artifacts/deferred-work.md` — close the four absorbed items in
      words (`:10653` · `:10666` · `:10789` · `:10441`), and re-own the other eight, each with a
      named owner: `:4540` · `:10485` · `:10497` · `:10513` · `:10552` · `:10642` · `:10840`, plus
      `:10240` (the "test connection" AC, Decision 4) — rationale: a debt item closes in words or
      not at all, and an item whose owner has passed through without it is an item with no owner.

**Acceptance Criteria:**
- [x] Given a selected segment and a configured provider, when the translate command runs, then the
      request carries the string `assemble_and_record_prompt` recorded, byte for byte.
- [x] Given the model is generating, when tokens arrive, then they render in the AI Translation panel
      as they land, and every one crossed a `Channel`; `grep` finds **0** `emit`/`listen` on this path.
- [x] Given a completed AI result, when nothing else is done, then `target_text` is unchanged — under
      every state, including error and cancelled.
- [x] Given a result and `⌘⇧↵`, when promoted, then the Editor shows it at the selected segment, its
      `translation_origin` reads `other`, and confirming it without an edit keeps `other` — the
      counter-check being that removing the origin write makes a named case go red.
- [x] Given the AI state at any moment, when read, then it is exactly one of the five values.
- [x] Given the translate, cancel and promote actions, when invoked, then each is a registered command
      dispatched by id, and each works when rebound through the shortcut screen.
- [ ] Given `core/ai/` and its approved seams are deleted, when the tree is compiled, then every other
      capability still compiles and its tests still pass (FR77).
- [ ] Given the eleven `pre-push` gates run individually, then all pass, and the CI run for the push is
      read on **both** platforms before this spec is written `done`.

## Implementation Notes

### 2026-09-21 — Phase 1 (the port and the three gates) — done, and one inherited red measured

Delivered: `ports/translation_provider.rs` (new) declaring AD-2's third port —
`TranslateRequest<'_>` with **no** `derive(Debug)`, `TranslateOutcome::{Done, Cancelled}` so a
cancel is an `Ok` and not an error, and `translate(&self, request, on_token: &mut dyn FnMut(&str),
should_cancel: &dyn Fn() -> bool)`. The port names no Tauri type, so the `Channel` lives entirely in
the command layer. `ports/mod.rs`'s table row flipped to `✅ đã khai`. `ai_boundary.rs` gained the
third seam plus two negative controls; `aiconfig_keychain_boundary.rs` gained the named exemption and
closed `deferred-work.md:10441` — where the measured hole turned out to be the **brace** form
(`use …keychain::{read as alias}`), not the bare form the debt item quotes, which was already caught.

Carried to Phase 2, deliberately and not silently: the allowed-**names** gate for the third seam and
the compile-probe extension, both of which need `core/ai/client.rs` to exist first. Two tasks added
to Phase 2 for them.

⚠️ **Inherited red on this machine, measured rather than attributed.** `cargo test --locked` stops at
`asset_contract.rs`: **7 passed / 12 failed**, twice in a row with identical counts — so it is
deterministic here, not the varying shape root `AGENTS.md` describes. Evidence gathered before
blaming anything:
- The failing cases are exactly the ones that bind a local server —
  `asset_contract.rs:85`/`:111` `TcpListener::bind("127.0.0.1:0")` — and fetch from it; the assert
  that fires is `images_saved left: 0, right: 1`, the signature root `AGENTS.md` names for this class.
- **Removal test:** `ports/mod.rs` reverted and `translation_provider.rs` moved out of the tree →
  still exactly 7 passed / 12 failed. Phase 1's diff is **not** the cause. Files restored and verified
  against `git status` plus a byte-for-byte `diff` with the backup.
- CI at this story's `baseline_commit` (run `35436773193`, commit `5a23410`) was **green on both
  platforms**, so the failure does not exist at the baseline anywhere but on this machine.
- 🔵 LuLu is running (pid 3156) with `disabled = 0` **and `allowLocalHost = 1`, `passiveMode = 0`**.
  That contradicts the remedy the debt ledger records: `allowLocalHost = 1` is not, on this machine
  today, enough to make these cases green. Whether LuLu is the blocker was **not** isolated — that
  needs disabling a security tool, which is Ice's call, not a story's.

⇒ Consequence for the rest of this story: `pre-push` runs `cargo test --locked`, so it cannot go
green on this machine until that is resolved. Later phases must not "fix" `asset_contract.rs` — it is
untouched by this story and green on CI.

### 2026-09-21 — Phase 2a (the client, the SSE parser, the CRLF fix) — done, two gaps it exposed

Delivered: `core/ai/client.rs` (new) — `OpenAiChatClient` behind the port, async `reqwest` with
`read_timeout`, `chunk()`, and a pure `split_sse_frames(&[u8]) -> (Vec<String>, Vec<u8>)` handling
`\n\n` and `\r\n\r\n`, multi-line `data:`, and a frame split across two reads. `should_cancel` is
asked both before each read and before emitting each parsed event; a cancel returns
`Ok(Cancelled)`, never `Err`. No new crate and no new feature — `serde_json::to_string` builds the
body, so reqwest's `json` feature stays off. `rag.rs::pop_piece` now removes a whole `"\r\n"`, on
both `out` and the `pieces` mirror. `core/ai/mod.rs:14-16` fixed in place with 🔵 and the date.

Two gaps this phase exposed — neither is a defect in what it built, both are holes in the spec as
written, and both now have tasks:
1. **The CRLF fix has no standing test.** The phase verified it with a real revert-and-confirm-red
   counter-check, then deleted the scratch cases, correctly noting that no task named a permanent
   home for them. An absorbed debt item that closes with no guard is a debt item that reopens
   silently. → Phase 4 task added against `ai_rag_contract.rs`.
2. **`pop_piece` decides the mirror pop from the wrong string.** It pops `'\r'` from
   `pieces.last_mut()` when **`out`** ends with `'\r'`, not when that piece does. Reachability was
   not demonstrated either way; the `debug_assert_eq!` in `assemble_prompt` would catch a divergence
   in a debug build, but only if some case exercises a CRLF body — and none does today. The Phase 4
   case covers both halves.

Also hardened into a task rather than left implicit: the `Authorization` header is built as a plain
`format!("Bearer {…}")` and is not marked sensitive, so a `HeaderValue` debug-print would show the
key. §Always bars exactly that.

### 2026-09-21 — Phase 2b (the send path, cancel state, the promote write) — done, one Send problem
found and solved

Delivered exactly the Phase 2 tasks not already closed by Phase 2a: `commands/aitranslate.rs` (new),
its cancel state, `commands/aiprompt.rs` widened, `core/aiconfig/keychain.rs`'s `#[allow(dead_code)]`
dropped, two new `message_keys!` variants (plus `vi.json`), `lib.rs` wiring, `commands/segment.rs`'s
promote write, `core/ai/client.rs`'s `set_sensitive(true)`, and `config_invariants.rs`'s census.
`tests/ai_boundary.rs`'s two carried Phase 1 tasks (allowed-names freeze, compile-probe extension)
were **not** touched — left for the agent that owns them, per this file's routing.

⚠️ **A real `Future: Send` problem, not anticipated in the task list, found while wiring the send
path — solved by construction, not by weakening anything frozen.** `ports::TranslationProvider::
translate`'s `on_token: &mut dyn FnMut(&str)` / `should_cancel: &dyn Fn() -> bool` (Phase 1, frozen)
carry no `+ Send` bound, so `OpenAiChatClient::translate`'s `Future` is not `Send` — but
`ai_translate_segment` is the tree's first literal `async fn` command, and Tauri's macro dispatches
every async command (literal or `(async)`-annotated alike) through `async_runtime::spawn` →
`tokio::spawn`, which requires `Send`. `.await`-ing the port directly inside the command does not
compile. Fix: `commands/aitranslate.rs` splits into three layers instead of two — `prepare_translate_
call` (sync, borrows `Store`/`OpenWork`, returns an owned `PreparedTranslateCall`), `run_translate_
call` (the actual testable seam — generic over `P: TranslationProvider`, no `Store`, no `spawn_
blocking`, this is what a fake provider substitutes at), and `send_prepared_translate_call` (wire-
only: hardcodes `OpenAiChatClient`, runs `run_translate_call` inside `tauri::async_runtime::
spawn_blocking` + `handle().block_on(...)` on a dedicated blocking-pool thread, so the non-`Send`
`Future` never crosses an `.await` that itself needs to be `Send`). Verified by `cargo check --tests`
compiling clean and `cargo test --locked` running the whole suite with no new failure — see below.

Also engineered, beyond the letter of the task list but load-bearing for correctness: `mod wire`
locks `OpenWorkState` only for the synchronous `prepare_translate_call` call and drops the guard
**before** the network `.await` — holding it across a call that can run tens of seconds would block
every other Chapter/segment command for that whole window, the exact class of debt
`config_invariants.rs` already carries for five *different*, deliberately-synchronous-body wires.
Here it is not debt: the split makes it structurally impossible.

**`deferred-work.md:10840` measurement, as this task asked for:** `grep -n "crate::core::ai" src/
lib.rs` → **0 lines**, same as at the 2026-09-18 measurement. `lib.rs` only ever names `crate::
commands::aitranslate::{...}` (the `AiTranslateGeneration` state, the two `wire::` fns) — the actual
`core::ai::client` reference lives inside `commands/aitranslate.rs`, one layer down. So the answer is
the ledger's "still doesn't" branch: *"XOÁ nửa `lib.rs` của miễn trừ cùng ba hằng số/vị từ chỉ phục
vụ nó"* (`AI_PROMPT_SEAM_LIB_RS_MARKER`/`_FILE` and the predicates that read them in
`tests/ai_boundary.rs`). That edit was **not** made here — it touches `ai_boundary.rs`, which this
phase was told to leave for the agent that owns the two carried Phase 1 tasks there. Recorded here so
that agent (or Phase 4, which owns closing `deferred-work.md:10840` in words) has the measurement
without re-deriving it.

**Two companion edits outside the Phase 2 file list, made because Phase 2's own changes broke them
mechanically:** `tests/ai_prompt_contract.rs`'s
`the_wire_returned_by_assemble_and_record_prompt_serializes_with_the_exact_tag_strings_and_field_names`
asserted the *exact* top-level key set of `AssembledPromptWire`; widening the wire with `sent_at`/
`sent_model` (the very task this phase was assigned) makes that assertion false unless it is updated
to name the two new keys — updated, both asserted `null` (no send has happened yet at that point in
the test). `commands::aiprompt::segment_not_in_chapter` was widened from `fn` to `pub(crate) fn` so
`commands/aitranslate.rs` could reuse it instead of minting a second key for the same fact — no
behaviour change, a visibility change only.

**Verification run, this machine, this story's baseline tree plus this phase's diff:**
- `cargo check` / `cargo check --tests` — clean, one pre-existing warning (`async_fn_in_trait`,
  acknowledged and accepted at Phase 1).
- `cargo test --test config_invariants --test aiconfig_keychain_boundary --test ai_prompt_contract
  --test ai_boundary --test ipc_contract --test segment_contract --test segment_boundary
  --test ai_rag_contract` — all green (22+19+9+31+33+209+9+32 = 364 passed, 0 failed).
- `cargo test --locked --no-fail-fast` — every test binary green **except** `asset_contract.rs`
  (7 passed / 12 failed), byte-for-byte the same signature Phase 1 already measured and cleared as
  pre-existing and unrelated (`TcpListener::bind` cases, LuLu on this machine) — re-measured here,
  not re-attributed. No other binary regressed.
- `npm run check:i18n` — green (965 keys, including the two new ones, placeholders match).
- `npx vitest run` — 87 files / 1261 tests, green (untouched by this phase, run as a regression
  check since `vi.json` changed).
- `npm run check:gates` / `check:commands` / `check:deps` / `check:debt-owner` — all green,
  unaffected.
- Not run this phase: the eleven `pre-push` gates as a single sequential run, `npm run build`'s
  freshness against the current `src/` (a pre-existing `dist/` was reused), and the nightly `schedule`
  e2e run — all three are explicitly Phase 4's/the story-closing agent's verification, not Phase 2's.

### 2026-09-21 — Phase 2b (the command layer, the Channel, cancel, the promote write) — done

Delivered: `commands/aitranslate.rs` (new) in three layers rather than two, because the port's
`on_token`/`should_cancel` are bare `&mut dyn FnMut`/`&dyn Fn` with no `+ Send`, so the client's
future is not `Send` and Tauri dispatches every async command through `tokio::spawn`, which requires
it — resolved by running the port call inside `spawn_blocking` + `Handle::block_on`. The middle
layer `run_translate_call` is generic over `P: TranslationProvider` and touches no `Store`, which is
the seam Phase 4 substitutes a fake provider at. `OpenWorkState`'s mutex is released **before** the
network `.await`, so a multi-second call does not freeze every other segment command.
`AssembledPromptRecord` gained `sent_at`/`sent_model` (both `None` until a send succeeds);
`promote_ai_translation` writes `target_text` and `translation_origin = TRANSLATION_ORIGIN_OTHER` in
one `UPDATE`; the `Authorization` header is now `set_sensitive(true)`; `keychain::read`'s
`#[allow(dead_code)]` is gone.

Verified by me against the diff, not the report: the promote `UPDATE` does set both columns in one
statement; the edit to `ai_prompt_contract.rs` is purely **additive** (two keys added to the expected
set, plus two new asserts that both are `null` on an assemble-only call) — no assertion was removed
or loosened; the census documents, in place, that `count_command_attrs_in` reads the attribute line
and not the signature's asyncness, so the repo's first literal `async fn` command lands in the
`plain` column — a limit declared rather than a number fudged.

`deferred-work.md:10840` measurement recorded: `crate::core::ai` appears **0** times in `lib.rs`,
unchanged from 2026-09-18. By the ledger's own instruction that means the dead `lib.rs` half of
Decision 1's exemption should now be deleted — the agent that owns the `ai_boundary.rs` tasks does it.

**Decision 6 — `temperature`/`max_tokens` are omitted when unset.** Ice settled 2026-09-21, after
Phase 2b flagged that it had widened the matrix on its own. Measured: `ChatCompletionsRequestBody`
declared both fields non-`Option`, so the client had to send something; `aiConfigState.ts:69` starts
a config as `{provider:'', endpoint:'', model:'', temperature:'', max_tokens:''}` with no seeded
defaults and no required-field marking. The shipped behaviour was therefore that a user with a
correct endpoint, model and key but a blank `temperature` sees *"not configured"* forever with no
hint which field is missing — the silent-emptiness class root `AGENTS.md` names as central. Both
fields become `Option` and are left out of the body when unset; an OpenAI-compatible endpoint treats
them as optional and applies its own defaults. Rejected: seeding defaults (writes a number to the
user's disk that they never chose, and is Story 4.2's territory) and refusing with a field-level
message (a new state and new strings for a case that should not be an error at all). The I/O matrix
needs no change — this is what makes its `not_configured` row literally true.

### 2026-09-21 — Phase 3 (the panel, the state, the three commands) — done, one chord decision measured and changed

Delivered exactly the six Phase 3 tasks: `src/config/aitranslate.ts` (new adapter — `Channel<string>`
built per call, `{value, error}` shape, never throws); `src/aiTranslateState.ts` (new — the five-value
`AiTranslateState`, `accumulatedText`, `runSegmentId` carrying the result's own identity the way
`aiPromptRecord.segment_id` does, `isAiTranslateResultStale` as a pure function mirroring
`aiPromptRecordIsStale`, `resetAiTranslate()`); joined into both reset clusters
(`modes/libraryChapters.ts`, `modes/libraryImport.ts`); three commands registered
(`ai.translate.run`/`.cancel`/`.promote`) with labels in `vi.json`; `AiTranslationPanel.vue` renders
the streamed text inside the existing `.ai-surface` div (kept the same `ref="surface"` element per
its own doc-comment — content was added as children, not a replacement) plus Dịch/Huỷ/Đưa-sang-bản-dịch
buttons, each a single `dispatch(...)` `@click`; `config/segment.ts` gained the `promoteAiTranslation`
adapter and `editorPanelState.ts` gained `promoteAiTranslationToEditor` (mirrors via
`replaceEditorSegment`, following `restoreVersion`'s shape) plus its own `editorPromoteAiTranslationError`
joined into `resetEditorPanel()`. `main.ts` wires all three command deps (not itself a named Phase 3
task, but load-bearing for the AC "each is a registered command dispatched by id... and each works" —
without it every dispatch would hit `portMissing`, the same shape every earlier Epic 4 command's deps
wiring already required and got).

⚠️ **Measured, not followed literally: `ai.translate.run` and `ai.translate.cancel` keep
`keys: undefined`, not `'Mod+Enter'`/`'Escape'` as §Code Map's reading of the mockup names.** Before
writing the registration, `grep` on `src/commands/index.ts` as it stood found `'Mod+Enter'` already
claimed by `editor.confirm_segment` (Story 2.5, line ~2464) and `'Escape'` already claimed by
`editor.clear_source_cuts` (Story 2.9, no `Mod`, line ~2815) — both single, unambiguous prior claims.
`src/commands/index.ts`'s own doc-comment at the `editor.confirm_segment` registration states the
mechanism measured here: `createKeymap` **throws** when two commands claim the same chord, and that
throw happens at `installCommands()` time, i.e. app startup — not a gate that reads red, a runtime
crash. Binding either chord again for translate/cancel would have made the counter-check
("bộ command THẬT dựng được keymap... không hợp âm nào giành nhau", `check:commands` Kiểm E) fail
immediately; it did not, because both were left `undefined`. `'Mod+Shift+Enter'` for
`ai.translate.promote` was checked the same way and found free (`grep` = 0 hits before this change),
matching §Code Map's claim that `keys.ts:92-95` reserved it by name — that one was bound as specified.
This is the same conflict class `deferred-work.md`/Story 6.2 already carries for a different `⌘↵` pair
(`index.ts:2441-2444`); it does not resolve itself by reading the mockup correctly, and choosing which
of the three commands gets `'Mod+Enter'`/`'Escape'` is Ice's call, not a default an agent should
silently pick. Both commands remain reachable by button and by the shortcut-rebinding screen (Story
1.21, FR22) in the meantime — the AC that "each works when rebound through the shortcut screen" holds
for all three regardless of default binding.

**A second judgment call, recorded because the spec left it open:** the pre-first-call rest value of
`AiTranslateState` is `'not_configured'`, matching the value `commands/aitranslate.rs::AiTranslateOutcomeWire`
itself returns when config is genuinely missing. This is the harmless default the matrix's own
"not_configured is not an error" language points at — but it is technically incorrect for a properly
configured user who has not yet translated anything (the panel would, if it rendered text for that
state, invite configuration to someone who does not need it). The panel only renders that invite text
when `aiTranslateRunSegmentId !== null` (i.e. a translate attempt has actually happened and come back
`not_configured`), so a configured-but-idle user sees nothing extra — the pre-attempt gap is closed by
omission in the template, not by a sixth state value, keeping the five-value union exactly as §Always
requires.

**A third judgment call:** `promoteAiTranslationToEditor(segmentId, text)` writes to the segment the
AI result was generated for (`aiTranslateRunSegmentId`), not necessarily to whichever segment the caret
currently sits on. The matrix's own "Caret moves during generation" row requires the call to "land
against the segment it started on"; writing to the current caret instead would silently put one
sentence's AI translation into a different sentence's `target_text` whenever the caret moved mid-stream
or after completion — exactly the silent-corruption shape AD-47/AGENTS.md's Known Pitfalls warn about.
`main.ts`'s `promoteAiTranslate` dep gates on this explicitly (state is `done`/`cancelled`, text
non-empty, `runSegmentId !== null`) before calling it; a refusal is a `console.warn`, never a throw.

**One CSS bug caught and fixed during self-verification, not by a gate:** the first draft of
`.ai-translate-text` used `var(--face-read)`/`var(--font-read)`/`var(--leading-read)`. `--face-read` is
a real generated var (font-family only), but `--font-read`/`--leading-read` are not — every other file
in the tree pairs `--face-read` with a sized token (`ui-sm`, `ui-md`, `read-title`, …), never bare.
`check:tokens` did not catch this (it validates declared token names, not that a `var()` reference
resolves to something CSS actually defines) and `vue-tsc`/`eslint` cannot see inside a CSS string
either — this would have shipped as invisible text (empty computed font-size/line-height) with every
other gate green. Fixed to the `editor` token (`--face-editor`/`--font-editor`/`--leading-editor`,
15px/1.95, DESIGN.md's own "Bản dịch trong Editor" role) — this text **is** a translation, just not
yet promoted, so the token whose declared `use` already names that role is the correct reuse, not an
invented one.

**Verification run, this machine, this phase's diff on top of Phase 2b's tree:**
- `npx vue-tsc --noEmit -p tsconfig.json` — clean, no errors.
- `npm run check:i18n` / `check:commands` / `check:tokens` / `check:layout` / `check:panel-refs` /
  `check:lint` / `check:gates` / `check:deps` / `check:debt-owner` / `check:dict` / `check:dict-manifest`
  — all green. `check:commands` Kiểm E's real-registry counter-check ("bộ command THẬT dựng được
  keymap... không hợp âm nào giành nhau") is what confirms the `'Mod+Enter'`/`'Escape'` decision above
  did not silently reintroduce the collision.
- `npx vitest run` — 87 files / 1261 tests, green, same count as Phase 2b recorded (no frontend test
  exists yet for this phase's own surface — that is Phase 4's job; this run is a regression check).
- `npm run build` — clean (`vue-tsc` both configs + `vite build`); the one Rollup warning
  (`INEFFECTIVE_DYNAMIC_IMPORT` on `@tauri-apps/api/core`/`event`) is pre-existing, naming
  `editorPanelState.ts`'s already-there dynamic import, not something this phase added.
- Not run this phase: `cargo test` (no Rust file touched — Phase 2a/2b already measured it, and
  AGENTS.md's routing keeps re-deriving a neighbouring phase's measurement out of scope here), the
  eleven `pre-push` gates as a single sequential run, and the nightly `schedule` e2e run — all three
  are the story-closing agent's verification, not this phase's.

**Left for Phase 4, not started here:** no test file was added or touched (`ai_translate_contract.rs`,
`ipc_contract.rs`'s registration case, `tests/frontend/aiTranslate*.test.ts`, the two-producer identity
case, the temperature/max_tokens omission case, the CRLF regression case, and the `deferred-work.md`
closures are all still open Phase 4 tasks — none of them were touched by this phase).

### 2026-09-21 — Phase 3 (the panel, the state, the three commands) — done

Delivered: `src/config/aitranslate.ts` (builds the TS `Channel<string>` and hands it to `invoke`),
`src/aiTranslateState.ts` (the five-value state, the accumulating text, `runSegmentId` carrying the
result's own identity, and `resetAiTranslate()` joined to both Work-change reset clusters), the three
commands, the panel rendering into the pre-existing `.ai-surface`, and the promote path mirroring
through `replaceEditorSegment` exactly as `restoreVersion` does.

**Decision 7 — translate and cancel ship with no default chord.** Ice settled 2026-09-21. This spec's
§Code Map told Phase 3 to bind `'Mod+Enter'`; measured, that chord belongs to
`editor.confirm_segment` (Story 2.5) and `'Escape'` to `editor.clear_source_cuts` (Story 2.9), and
`createKeymap` throws on a collision at `installCommands()` — so the instruction as written was an
app-startup crash, not a gate failure. No AC of this story asks for a chord on either command, and
all eight existing Epic 4 commands already ship `keys: undefined`. Rejected: taking `⌘↵` back from
confirm-segment, the most-used action in the app and in users' fingers since Story 2.5. `⌘⇧↵` for
promote was free and is bound as specified, which is what `keys.ts:92-95` reserved it for. §Code Map
corrected in place above rather than left to mislead the next reader.

Also worth keeping: Phase 3 caught, during its own verification and not from any gate, that its first
CSS draft used `var(--font-read)`/`var(--leading-read)`, which do not exist — only suffixed forms like
`--font-read-title`/`--font-read-body` do. That would have shipped as invisible AI-translated text
with every gate green, which is this repo's central failure class wearing a stylesheet.

### 2026-09-21 — Phase 4a + 4b (tests that move, the ledger) — done, then verified against the diff

Delivered by 4a: `ai_translate_contract.rs` (new, **26** cases — the report said 27, the measured
count is 26), the `ipc_contract.rs` registration case, the CRLF regression pair in
`ai_rag_contract.rs`, the allowed-names freeze for the third seam
(`ALLOWED_AI_CLIENT_NAMES_IN_COMMAND_SEAM`), the compile probe widened to three seams and run by
hand, and the deletion of the dead `lib.rs` half of Decision 1's exemption. Delivered by 4b:
`tests/frontend/aiTranslate.test.ts` (11 cases) and the ledger pass.

**Verified by the orchestrating session against the diff, not the reports:**
- 33 files in the story diff.
- 4a made one production-code change outside its task list: extracting `build_request_body` as
  `pub` so Decision 6's `skip_serializing_if` is observable without a socket. Checked the thing that
  makes such an extraction honest or worthless — `translate()` at `client.rs:202` really does call it
  at `:213`, so the case guards the production path and not a parallel copy.
- 4a claimed it deleted `line_is_the_approved_ai_prompt_seam_in_lib_rs`; `grep` finds the name 3
  times. Measured further: `grep -c "fn line_is_..."` → **0**. The definition is gone and the three
  hits are historical doc-comments, which is the repo's fix-in-place convention. Claim holds.
- The ledger pass is 130 insertions, **0** deletions — a closed item is never deleted.
- Two of 4b's re-own notes had gone stale between writing and reading, exactly as 4b warned:
  `ai_translate_contract.rs` now exists and touches the keychain 33 times (so the second-binary
  condition fired, and separate test binaries are separate processes, which is the answer), and 4a
  did run the deletion. Both closed with the measurement rather than left pointing at a future that
  had already happened.

**Matrix Test Audit — all 11 rows covered by a named case that RAN and PASSED.** Nine rows map to
cases in `ai_translate_contract.rs` (26 passed); the two frontend-only rows — *"Promote while
generating"* and *"Caret moves during generation"* — map to named cases in `aiTranslate.test.ts`
(11 passed). No row is covered by a case that exists but was filtered out, ignored or skipped.

**AC4's counter-check, run by me rather than asserted.** The AC requires that removing the origin
write turns a named case red. Removed `translation_origin = ?2` from the promote `UPDATE` and
adjusted the params so the statement still **compiles** — a signature break would have produced a
false red. Result: exactly **1** of 26 red,
`promote_writes_target_text_and_origin_other_in_one_operation_and_confirm_without_an_edit_keeps_it`,
failing on `left: "" right: "other"` — the right reason, and specific rather than a blanket break.
Restored byte-identical to the backup, re-ran 26/26 green, and confirmed the restoration with
`git status`/`--stat` rather than `git diff`.

### 2026-09-21 — Verification pass: a CI gate the rename had silently unhooked

**Found and fixed: `.github/workflows/ci.yml` still filtered on the old test name.** Phase 4a renamed
the FR77 compile probe `..._two_approved_seams_...` → `..._three_approved_seams_...` (honest — the
seam count really did go from two to three). The dedicated FR77 CI step passes that name to
`cargo test -- --ignored --exact`, and nothing in the eleven `pre-push` gates or `check:gates` covers
a raw workflow step, so nothing local went red. Measured before fixing: `grep -c` for the old name in
`ci.yml` → 2 (a comment and the live filter), and `grep -c` for it in `ai_boundary.rs` → 0.

What would have happened is worth recording, because it is the one case where a guard built in an
earlier story earned its keep: Story 4.7 loop 2 added `if ! grep -q '^running 1 test$'` with the
message *"bộ lọc --exact khớp 0 ca … cổng đối chứng dương AD-13 đang bị vô hiệu hoá trong im lặng"*.
So this would have surfaced as a loud red on the next push rather than a step that quietly tested
nothing. That is exactly the defect it was written for, hit for the first time.

Fixed the filter and the step's own name (*"hai seam"* → *"ba seam"*), then **verified by running the
real CI command**, not by reading it: `running 1 test` … `1 passed; 0 failed; 25 filtered out` in
16 s. The `^running 1 test$` string the guard greps for is present.

**AC7 is discharged on its first clause only, and is left unchecked for the second.** The clause
*"every other capability still compiles"* now has real evidence — a `cargo check` on a copied tree
with `core/ai/` and all three seams removed. The clause I also wrote, *"and its tests still pass"*,
does **not** have evidence: the probe runs `cargo check`, never `cargo test`, and as written the
clause is not even well-defined, because the AI test binaries themselves name `core::ai` and cannot
survive the deletion they are supposed to run after. That is a defect in the acceptance criterion I
wrote, not in what was built. Per root `AGENTS.md` — *"Never mark something passed by inference. Any
clause that cannot be accepted at the current layer goes into `deferred-work.md` with an owner"* — it
is recorded there rather than ticked.

Also noted while sweeping for other seams the rename could have cut: every remaining occurrence of
the old name lives in historical records (`spec-4-7…md`, which is `done`, and debt entries recording
4.7's findings), which the repo's convention says to leave as written. One closed item's body names
`the_two_approved_seams_are_matched_narrowly_and_neighbours_are_not`, a test 4a renamed — a dangling
pointer inside an already-closed item, left alone deliberately rather than rewritten.

### 2026-09-21 — AC6 closed by a follow-up pass, after the verification found it uncovered

The rebind half of AC6 had **no** coverage: `aiTranslate.test.ts` contained zero references to
`applyBindings`/`effectiveBindings`/`SCOPE_SHORTCUT`, so only dispatch-by-id was tested — while
Story 4.7 had already set the pattern for the other half at `aiPromptInspector.test.ts:666-700`.
Four cases added on that pattern: each of the three commands is rebound through the real shortcut
layer to a chord confirmed free in the product registry, a real `KeyboardEvent` is fired, and each is
asserted to reach the same place the click path reaches — including promote landing on the run's own
segment after the caret has moved. Counter-checked by removal: `applyBindings` was made a global
no-op, all four new cases went red for the right reason (the mocks were never called) while the
original eleven stayed green — so the new cases guard the keyboard path specifically and do not
merely re-cover dispatch-by-id. Suite: 88 files / **1276** tests green.

⚠️ **The dispatch I wrote for that pass contained a false statement, and the agent measured instead
of obeying it.** I said all three commands ship `keys: undefined`; measured, `ai.translate.promote`
registers `keys: ['Mod+Shift+Enter']` (`src/commands/index.ts:3478-3480`) — Decision 7 left only
`.run` and `.cancel` unbound, which is what Decision 7's own text says. The spec was right and my
instruction was wrong; the case still drives promote through an explicit rebind to a *different*
chord, so it exercises the rebind path rather than coincidentally matching the shipped default.

**The eleven `pre-push` gates, run individually: all eleven PASS** (`deps` · `tokens` · `i18n` ·
`commands` · `layout` · `panel-refs` · `dict` · `dict-manifest` · `lint` · `gates` · `debt-owner`).

### 2026-09-21 — Final local measurement on a still tree

`cargo test --locked --no-fail-fast`: **65 binaries · 1 768 passed · 0 failed · 21 ignored**, exit 0.
The arithmetic closes exactly against the pre-story figure: 1 736 + 26 (`ai_translate_contract.rs`,
the new binary) + 2 (the CRLF pair) + 1 (the registration case) + 3 (the third seam's controls)
= 1 768. `npx vitest run`: 88 files · **1 276** passed. The eleven `pre-push` gates individually: all
pass.

`asset_contract.rs` was **green** in this run (19/19), as it also was in Phase 4a's. Combined with the
removal test from Phase 1 — diff pulled out of the tree, still 7/12 red — the inherited red is now
measured from both directions: it is not caused by this story, and it is not permanent either. It
varies across longer intervals while staying deterministic within a few minutes, which is what a
per-binary environment gate looks like and is not what a defect in this diff would look like.

**AC8 is discharged on everything local and is left unchecked for its last clause.** The gates pass
and both suites are green here; *"the CI run for the push is read on both platforms"* cannot be
discharged without a push, which this workflow forbids and which is Ice's call.

## Spec Change Log

### 2026-09-21 — Patch round: a patch agent overwrote a test file and destroyed 13 cases

Finding #1's fix in `src/aiTranslateState.ts` is correct and is in place. But the same agent, asked
to *add* two cases to `tests/frontend/aiTranslate.test.ts`, **replaced the file**: 629 lines and 15
cases became 64 lines and 2. Destroyed with it were both frontend-only I/O-matrix rows (*"Promote
while generating"*, *"Caret moves during generation"*) and all four rebind cases that had just closed
AC6 — the patch round would have silently re-opened a criterion the verification round had closed,
and `npx vitest run` would still have reported every file passing.

What caught it was the agent's **own report**, not a gate: it described the file as "(new)" with two
cases, for a file that already held fifteen. A report that disagrees with the tree is worth more
attention than one that merely sounds wrong.

Recovery, and why it was possible: the file is untracked, so `git checkout` could not restore it. The
review diff written before the patch round carried the whole file as added lines; it was extracted
from there (629 lines, 15 `it(` — matching exactly), restored, and the agent's two genuinely new
cases were re-applied by hand onto the restored file using its existing `freshPanel()`/`pendingRun()`
helpers rather than the agent's own parallel mock setup. The file now holds **17** cases.

Counter-checked afterwards, because a restored file that no longer guards anything is worse than a
missing one: removing `if (state.value === 'generating') void cancelAiTranslateCall()` turns exactly
**1 of 17** red — the new generating case — while the negative control (reset with no call in flight
sends nothing) stays green, so the pair guards the branch and not merely the function. Restored
byte-identical, re-ran 17/17.

### 2026-09-21 — Patch round verified, and a red `npm run build` that several green reports had hidden

All three patches verified against the tree, not the reports: `mark_prompt_as_sent` now tests
`r.segment_id == segment_id && r.prompt == sent_prompt` (no new field — it reuses the string the
record already carries); `row.is_omitted` at `aitranslate.rs:227` now precedes `keychain::read()` at
`:231`; `interpret_sse_event` returns `Ignore` for an empty payload **before** parsing, with an
explicit negative control keeping the `: ping` path at zero events; and the 1 MiB cap is checked on
the genuinely unconsumed remainder rather than on each gross read. The patch agent that added a new
error variant also added the arm its exhaustive `From` match required *and* the seventh row of the
table-locking test, because leaving that test alone would have quietly falsified its own doc-comment.
No file shrank this round.

Suites after the patches: Rust **65 binaries · 1 772 passed · 0 failed**, the arithmetic closing
exactly as 1 768 + 1 + 2 + 1. vitest **88 files · 1 278**, which is 1 276 + the two reset-cancel
cases. Eleven gates: all pass.

⚠️ **`npm run build` was red, and had been for several rounds.** `tests/frontend/aiTranslate.test.ts`
contained `mount(AiTranslationPanel as never, …)`, which makes the wrapper `never` and every
`wrapper.vm.$nextTick()` a type error — about 20 of them. It survived because nothing that ran
covered it: `npx vitest` is green (a type error does not stop the runtime), `check:lint` is green,
and **the eleven-gate loop I had been reporting as "11/11 PASS" does not include `build`** — real
`pre-push` runs gates → vitest → **build** → `cargo test`, and I had been running the first two and
the last while checking the third only now. Fixed by deleting the cast and typing the helper
`Component`, which is what the two files that already mount this same panel
(`aiPromptInspector.test.ts:859`, `promptLibraryOverlayRender.test.ts:497`) do — they pass no cast at
all. `vue-tsc` clean, build passes, 17/17 still green.

The lesson is the session's own recurring one, aimed at me this time: a set of gates is only evidence
for what it runs, and I described a subset as though it were the whole.

## Review Triage Log

### 2026-09-21 — Note on the Implementation Notes' own shape (review finding #7)

§Implementation Notes carries **two** "Phase 2b" entries and **two** "Phase 3" entries, describing
overlapping but not identical work. Cause, named rather than tidied away: the Phase 2b and Phase 3
implementation agents each wrote their own entry, and the orchestrating session wrote its own
verification entry for the same phase on top — only Phase 4b was told not to touch the spec, and that
instruction came too late for the earlier phases. The entries are **kept** rather than merged,
because that section is append-only; read each pair as *the agent's account* followed by *the
orchestrator's verification of it*, which is also why the second of each pair is the one carrying
measurements taken against the diff. The same slip produced a duplicate `## Review Triage Log`
heading while this very finding was being written up; that one was a structural duplicate rather than
a record, so it was removed.

Pass 1 — 2026-09-21, three layers over a 405,8 kB diff (blind-hunter 10 · edge-case-hunter 4 ·
verification-gap 1 + 1). No `intent_gap` and no `bad_spec`, so no loopback.

| # | Finding | Verdict | Evidence | Route |
|---|---|---|---|---|
| 1 | Switching Work / importing while `generating` never cancels the backend call | **high** | Verified: `resetAiTranslate()` (`aiTranslateState.ts:145-151`) bumps `sequence` and clears state; `cancelAiTranslateCall` is referenced only at `:136`. The provider keeps streaming and billing for a result nobody can see — against AD-22 and the epic's BYOK money rule | patch |
| 2 | I/O matrix has no row for "Work changes while generating" | **high** (same root cause as #1) | Same defect: the scenario was never considered. The matrix is frozen, so the behaviour is fixed instead | patch (grouped with #1) |
| 3 | `mark_prompt_as_sent` stamps a record it did not send | **high** | Verified: it compares only `r.segment_id == segment_id` (`aiprompt.rs:462-469`). The assemble button is disabled by `!canAssemble \|\| aiPromptAssembleBusy` (`AiTranslationPanel.vue:216`), **not** by `generating` — so translate, then re-assemble the same segment, and the finishing call stamps `sent_at`/`sent_model` onto a prompt that was never sent. The one diagnostic FR71 exists to provide would be lying | patch |
| 4 | An empty `data:` payload aborts the whole call | **medium** | Verified: `extract_data_payload` returns `Some("")` for a `data:` line with an empty value (`:345-355`), and `parse_chunk_content("")` fails `serde_json::from_str` → `MalformedEvent` → `?` ends the call (`:290-293`). The reviewer's other half is **refuted**: a comment-only frame (`: ping`) yields `None` and is skipped | patch |
| 5 | No cap on the SSE accumulation buffer | **medium** | Verified: `buf.extend_from_slice` grows without bound while no `\n\n` ever arrives (`client.rs:267`). `core/webimport/fetcher.rs` caps its own reads with `MAX_RESPONSE_BYTES`, so the project's own convention is to cap | patch |
| 6 | Keychain is read before the `is_omitted` guard | **low** | Verified: `keychain::read()` at `aitranslate.rs:216`, `row.is_omitted` at `:228`. An already-cut segment costs a real OS keychain access before being refused. Fix is a reordering, not added complexity | patch |
| 7 | Implementation Notes carry two "Phase 2b" and two "Phase 3" entries | **low** | Verified: headers at lines 546/619 and 657/750. Two phase agents wrote their own entries and the orchestrator wrote its own on top. §Implementation Notes is append-only, so the entries stay and a note explains the overlap | patch |
| 8 | Idle `aiTranslateStateValue` is `'not_configured'` for a configured user | **low** | Verified: `const state = ref<AiTranslateState>('not_configured')` (`:41`), papered over in the template. **Rejected**: the cleaner fix is a sixth value, and the AC fixes the set at exactly five. The modelling smell is real but the constraint that causes it is frozen | rejected |
| 9 | AC7's second clause is undischarged | — | **Rejected** — already recorded as owned debt with the measurement; its only fix would edit this build's spec | rejected |
| 10 | Decision 5's token basis (9.074 vs 9.432) is unreconciled | — | **Rejected** — both numbers are stated with their basis in the decision itself (the gate figure and the post-decision figure); the fix would edit this build's spec | rejected |
| 11 | Phase 4a's self-reported case count was wrong (27 vs 26) | — | **Rejected** — caught and corrected during verification and already recorded; not a defect in the change | rejected |
| 12 | `PreparedTranslateCall::api_key` widens the plaintext key's lifetime | **medium**, pre-existing | Real, and it extends an already-recorded `ApiKeySecret`-never-zeroizes debt rather than creating it | defer |
| 13 | The FR77 CI step couples to a test name by bare string | **medium**, pre-existing | Real and just cost a break; the coupling predates this story and its fix (shared constant or generated step) is a CI-design change | defer |
| 14 | `pop_piece` may diverge from the `pieces` mirror at a CRLF piece boundary | **maybe-false** | Two independent traces (Phase 4a by hand, edge-case-hunter by path tracing) found no construction that reaches it; neither proved it unreachable. Would be `medium` if real. Settled by enumerating what `expand_prompt_body` can put at a piece boundary, or by a property test over CRLF bodies | defer |
| 15 | The wire's `Done → mark_prompt_as_sent` branch is never exercised | **medium** | Pre-verified by the gap layer: every test calls `mark_prompt_as_sent` directly; no test builds an `AppHandle` to invoke the command. Deleting the call would leave `sent_at` permanently `None` with the suite green | defer |
| 16 | A malformed-header failure is labelled `retryable: true` | **low** | Real but Story 4.10 owns retry policy and copy, and the exhaustive variant test pins today's mapping | defer |

## Design Notes

**Why the async client, measured rather than preferred.** The obvious move is to copy
`core/webimport/fetcher.rs`, which is blocking and has a year of hardening behind it. It cannot work
here: `read_timeout` — the gap *between* chunks — exists only on the async builder
(`async_impl/client.rs:1456`) and appears zero times in `src/blocking/client.rs`. A blocking client
offers only a whole-request timeout, so any value short enough to notice a stalled provider also
kills a long, healthy generation, and any value long enough to allow a long generation makes a stall
indistinguishable from work. The async path also makes cancel immediate instead of "after the current
read returns".

**Why no new dependency, stated because the opposite is the natural assumption.** Streaming with
`reqwest` is usually written with `bytes_stream()`, which is behind the `stream` feature — and the
spine's CAP-4 row leaves every SSE crate un-reviewed under NFR15, so reaching for one would open a
licence gate mid-story. Measured on the pinned source, `Response::chunk()` is not feature-gated at
all (`async_impl/response.rs:310` against the `#[cfg(feature = "stream")]` at `:349`). So this story
adds zero crates and zero features, and NFR15 has nothing to review.

**Why the SSE parser is a pure function.** The suite cannot bind a local port: AD-45 bans a listening
port in the product, and on this machine a test binary that opens one has produced false reds twice.
A parser that takes a byte slice and returns `(events, remaining)` lets every interesting case — a
frame split across two chunks, a `[DONE]` terminator, a stream that stops without one — be a plain
unit test, and leaves the network layer thin enough that the port trait covers it.

**Why promote needs its own write.** Three functions write `target_text` today and none touches
`translation_origin`; only `confirm_segment` does, and only to decide self-versus-unchanged. AD-47①
requires a non-user write to reset the baseline *and* stamp the origin in one logical operation, and
AD-47③ fixes the value for this mechanism at *người khác dịch*. Reusing `save_segment_targets` would
write the text and silently leave the origin saying the translator typed it — the failure AD-47 was
written for, which surfaces hundreds of sentences later as *"the AI translation no longer sounds like
me"* and turns no gate red.

## Verification

**Commands:**
- `npm run build` before any `cargo test` — without `dist/`, `cargo test` fails at compile time.
- `cd src-tauri && cargo test --locked` — measure the baseline on this story's own starting commit in
  a still tree first; do not inherit a neighbouring spec's counts.
- `npx vitest run` — expected 0 red; the case count rises by the new frontend files only.
- The eleven `pre-push` gates individually: `check:deps` · `tokens` · `i18n` · `commands` · `layout` ·
  `panel-refs` · `dict` · `dict-manifest` · `lint` · `gates` · `debt-owner`.
- `cargo test --test ai_boundary --test aiconfig_keychain_boundary --test config_invariants` after
  Phase 1 and again after Phase 2 — and read **why** each red fires, not its colour.
- Counter-check the seam by removal: delete the approved exemption and confirm `ai_boundary` goes red
  naming the new file; delete the origin write and confirm a named case goes red naming the column;
  delete the `is_omitted` guard and confirm a matrix case goes red. A compile error is not a discharge.
- Counter-check cancel by measuring, not by asserting a flag: after cancelling, assert that **no
  further** Channel message arrives, not merely that the state changed.

**Manual checks:**
- ⚠️ Baseline on entry, measured rather than inherited: the nightly run of 2026-09-20
  (`35534874121`) is **red** at `check (windows-2025)` →
  `store_contract::the_wal_stops_growing_once_it_crosses_the_threshold` (`store_contract.rs:890`),
  while `e2e (macos-26)` was green. Do **not** record it as a Windows item: the same case failed on
  `check (macos-26)` on 2026-09-16 (`35148735115`) and the nights of 09-17 and 09-19 were fully
  green, so it changes platform and it changes night — an intermittent failure of the
  *tolerance-0* clause Ice tightened on 2026-09-13, not a platform fault.
  `spec-ca-wal-do-tren-windows.md` is already `status: done` and names this same case, so the
  closure did not hold; that is an open fact this story inherits and must not silently absorb.
  Re-read the run at the end rather than assuming the same red.
- Read the latest nightly `schedule` run before writing `done`; if it is red, write the reason down.
