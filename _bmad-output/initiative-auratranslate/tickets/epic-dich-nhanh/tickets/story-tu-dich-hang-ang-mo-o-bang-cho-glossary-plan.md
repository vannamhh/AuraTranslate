---
title: 'Tự dịch hàng đang mở ở Bảng chờ Glossary'
type: 'feature'
ticket: '5'
created: '2026-11-11'
status: 'built'
baseline_revision: 'd63cdae436979abf385c1701ee5c52d5313ba8a9'
route: 'full'
route_source: 'auto'
risk: 'high'
review: 'quick'
review_source: 'pinned'
lenses_ran: ['quick']
review_loop_iteration: 0
context: []
---

<frozen-after-approval reason="human-owned intent — do not modify unless human renegotiates">

## Intent

**Problem:** Queue context translation needs a click for every sentence; users need opt-in translation of the row they stop on.

**Approach:** Add Global `app_config.quick_translate_auto`, a Settings switch and debounced current-row translation after lookup. Rust enforces eligibility through a gate reusable by 15.6.

## Boundaries & Constraints

**Always:** AD-64 §1,8,9 and EXPERIENCE §Dịch nhanh. Default off; dedicated endpoint and model required. Clearing either disables the saved option. Rust observes real translation activity; quick runs never advance or cancel `AiTranslateGeneration`. Preserve manual fallback, run-id cancellation, cache precedence, plain text and usage display.

**Never:** Implement preview auto scheduling (15.6), prefetch other rows, add dependencies/migrations, change main configuration resolution, or write reference text into the Work, TM or Glossary.

## I/O & Edge-Case Matrix

| State | Automatic behavior | Manual behavior |
|---|---|---|
| Off, blank or partial dedicated config | `auto_off`; zero provider calls, including direct IPC | Existing Global fallback/validation |
| On; missing translation; settled open row | One quick run after measured wait | Existing cancellation |
| Chapter/cache hit | Display existing result; zero provider calls | `Dịch lại` retains replacement behavior |
| Real translation running | Paused status; zero automatic calls | `Dịch thử` remains usable |
| Error, empty or truncated result | Skip this row until overlay closes | Explicit retry remains usable |
| Row change, overlay close, option off | Clear timer; cancel owned run; discard late results | Other surface untouched |
| Other quick surface busy | No competing run or retry loop | Existing busy feedback |

</frozen-after-approval>

## Code Map

- `src-tauri/src/commands/aitranslate.rs`: `AiTranslateGeneration` has only `next/is_current`; four real send wires share it. Quick tickets are separate. Observe real activity through blocking completion and every exit; preserve generation semantics.
- `src-tauri/src/{commands/aiconfig.rs,core/aiconfig/store.rs,core/scope/store.rs}`: dedicated source/readiness, atomic field saves and Global config storage.
- `src/{quickAiConfigState.ts,config/quickAiConfig.ts,SettingsQuickAiSection.vue}`: 15.2 saved/draft separation and Global locking. Publish saved switch immediately after successful writes; queue watcher cancels pending/owned work.
- `src/glossaryQueueState.ts`: reuse lookup precedence, references, run tokens, watchers and reset. `src/{GlossaryQueueOverlay.vue,config/glossaryContextReference.ts,quickTranslateRun.ts}`: rendering, runtime validation and busy state.
- Coverage: `src-tauri/tests/{glossary_context_reference_contract.rs,quick_translation_contract.rs,ai_translate_wire.rs,ai_batch_wire.rs,aiconfig_contract.rs}`; `tests/frontend/{glossaryContextReference.test.ts,quickAiConfig.test.ts}`; `e2e/specs/story-15-1-quick-translation-cache.e2e.mjs` supplies provider counting/queue fixtures.

## Tasks & Acceptance

Planning is complete. The implementation lead delegates Rust, Webview and verification sequentially to a fresh agent per phase; use a sibling handoff file for working evidence. Lead integrates results and completes all tasks before reporting. Keep any further planning checkpoint with Ice limited to measured debounce alternatives.

**Execution:**
- [x] Rust: `core/scope/store.rs`, `core/aiconfig/store.rs`, `commands/aiconfig.rs` — persist opt-in, expose effective switch/eligibility, reject invalid enabling and atomically reset on field removal. Extend config IPC and register a typed Global setter in `src-tauri/src/lib.rs`.
- [x] Rust: `commands/aitranslate.rs` — track real execution lifetime; add shared gate and typed `trigger: auto|manual`; return `auto_off` or paused before preparation/network. Check Rust activity/config at admission. Emit versioned activity start/end events; expose initial snapshot. Update existing callers to manual.
- [x] Webview: config adapters/state, `SettingsQuickAiSection.vue`, commands/main/i18n — validate wire additions; register mouse/Tab switch without shortcut, locked at Work tier; show save errors and propagate saved changes.
- [x] Webview: queue state/overlay — schedule current row after lookup; clear timers/runs on lifecycle/config changes; keep session failure set across revisits/purge; map `auto_off` to missing. Hide normal button while waiting; preserve retry/paused buttons and focus/Esc. New `src/config/aiTranslateActivity.ts` subscribes before snapshot (ignore older revisions), wired in `src/main.ts`; queue watches activity end to reschedule without polling.
- [x] Verification: new `e2e/specs/story-15-5-quick-auto-queue.e2e.mjs` — measure candidate waits on actual queue/provider fixture, recording build, population, calls and latency. Present alternatives to Ice before selecting. Cover ten-row traversal/stops, off, pending disable, close and failure revisit using call deltas.
- [x] Verification: cover matrix via real IPC and activity wiring, early/error/cancel exits and stale completion. For AD-64(a), remove production guards, preserve signatures, prove targeted red for intended reason, restore.

**Acceptance Criteria:**
- Given opt-in, when rapidly traversing ten rows then stopping, then WKWebView sends only the stopped row; off sends zero.
- Given off/incomplete configuration, when real IPC receives `auto`, then `auto_off` and zero provider calls result.
- Given real translation activity, when auto attempts, then show `Lô dịch đang chạy nên tự dịch tạm dừng.`; manual still works and quick cancellation preserves real generation.
- Given a failed row, when revisiting, then no auto retry; manual retry works, reopening clears failure.

## Implementation Notes

- Ice approved 800 ms after measured 400/800 ms alternatives; production already holds the selected value.

- Real activity belongs to `AiTranslateGeneration` through a separate count/revision; its execution guard moves into the blocking closure so cancellation does not announce idle before execution exits.
- Dedicated configuration writes reset auto opt-in in their Writer transaction; successful saved settings publish cancellation revisions before refresh.
- Unknown or failed activity monitoring blocks automatic calls while leaving manual fallback available. Session failure suppression survives cache purge and row revisits.
- AD-22 remains Channel-only for streamed tokens. The prior blanket event source guard now distinguishes the typed, versioned activity-status snapshot required by this plan; other topics, payloads and listeners remain forbidden.
- Measurement, mutation and phase evidence: [phase handoff](story-tu-dich-hang-ang-mo-o-bang-cho-glossary-handoff.md).

## Plan Change Log

## Review Triage Log

- medium · patch — `glossaryQueueState.ts` paused-response race confirmed: activity-end watcher returns while the owned run is active; a later paused IPC response releases the run without rescheduling, leaving the settled row stuck. Fixed by reusing the existing scheduler after the response settles; event-before-response regression passes, and removing the production scheduling call makes it fail at the expected resume assertion.

## Verification

- `npm run build` before Rust; `npm run test:story 15.5 -- --list`, then touched Rust/frontend targets.
- Commands/i18n/lint gates; no `POINTER_DEBT` increase. Registration requires one full run.
- `npm run test:e2e -- --spec e2e/specs/story-15-5-quick-auto-queue.e2e.mjs`; record measurement and mutation evidence in handoff.
- Read CI Windows/UTC and latest scheduled e2e; record unavailable/manual evidence with Epic 15/B7 owners.
