# Epic 4 Context: AI mở & Smart RAG Injector

<!-- Compiled from planning artifacts. Edit freely. Regenerate with compile-epic-context if planning docs change. -->

## Goal

The translator configures one AI provider — their own API key (BYOK) or a local model (Ollama/LM Studio) — through a single form, then calls AI to translate a segment or a batch. Results stream in, are cancellable mid-flight, and show token counts with a cost estimate. Before every call, the system automatically injects confirmed Glossary terms found in the sentence (the Smart RAG Injector), and the user can inspect the exact final prompt sent. Results land only in the AI Translation panel and never auto-flow into the Editor. The defining constraint: with AI configuration fully removed, every capability built in prior epics must keep working in full — enforced by an automated test, not developer discipline.

## Stories

- Story 4.1: Module `ai/` cô lập và test cưỡng chế ranh giới
- Story 4.2: Cấu hình nhà cung cấp AI
- Story 4.3: API key trong keychain
- Story 4.4: Bộ prompt theo thể loại
- Story 4.5: Xuất và nhập bộ prompt
- Story 4.6: Smart RAG Injector là một hàm thuần
- Story 4.7: Xem prompt cuối cùng đã gửi
- Story 4.8: Dịch một segment với kết quả chảy dần
- Story 4.9: Dịch theo lô và huỷ giữa chừng
- Story 4.10: Lỗi mạng và lỗi API
- Story 4.11: Số token và ước tính chi phí
- Story 4.12: Bố cục màn hình hẹp và hiệu chỉnh ngưỡng

## Requirements & Constraints

- Covers FR65–FR77 and FR79 (14 FRs) plus NFR9 (prompt sets are open text files), NFR11 (API keys stay in the OS keychain), and NFR12 (this is network egress point #1 of exactly three allowed app-wide, each user-triggered only).
- Success metric: the percentage of confirmed Glossary terms actually used correctly in AI suggestions — a direct measure of Smart RAG Injector effectiveness.
- AI results are read-only in their own panel; nothing about them auto-writes to the Editor — the user promotes a result explicitly (`⌘⇧↵`).
- No automatic retry on error, single or batched — with BYOK every call is the user's money. Retry is always user-initiated; in-progress Editor work must never be lost on an AI error. A streaming disconnect is an explicit error, not silently retried; tokens already received stay visible.
- SSE-parsing crates found in research (`reqwest-sse`, `sseer`) aren't yet GPL-cleared (NFR15 gate); hand-rolled frame parsing is an accepted fallback until that review runs.

## Technical Decisions

- **AD-13 (hard module boundary):** no module outside `core/ai/` may depend on it, enforced by an automated test or by `ai/` being a separate crate; the reverse direction is allowed — `ai/` may read `glossary/`, `tm/`, `segment/`. Story 4.1 built this boundary right after Epic 3, before real `ai/` code existed. Since Epic 5 and 6 now run first, the boundary test must be re-run against their finished code once Story 4.2 starts.
- **AD-14:** the Smart RAG Injector (`RagInjector`) is a pure function `(source sentence, scope, Glossary, TM) -> assembled prompt`; every AI call consumes its output as-is, no string concatenation at call sites — this is what lets the prompt inspector (FR71) reproduce the real request at 100% fidelity. FR70 splits across two epics: only the Glossary half closes here; the signature's TM parameter stays empty until Epic 7 fills it in, unchanged.
- **AD-22:** streaming goes over the Tauri Channel API, never loose events, never an auto-reconnecting SSE client — with BYOK, auto-reconnect means double-billing and duplicated output. Every AI call, single or batched, must be cancellable mid-flight.
- **AD-29:** API keys exist only in Rust via the `keyring` crate called directly — never `tauri-plugin-keyring`, which exists to expose that API to JavaScript (what NFR11 forbids). Keys never cross IPC; the frontend only ever learns "configured" / "not configured".
- **Two-tier configuration:** AI provider settings live at the Global tier and are overridable per Work, resolved through `ScopeResolver` with override semantics — the same two-tier shape `glossary/` already established. A per-Work override survives closing and reopening an `.atproj`: the reopen path rebuilds the Work tier, so a consumer sees Work-tier values in a later session, not only in the session that created them.
- **`TranslationProvider` port (AD-2) is NOT declared yet** — the ports module lists exactly three allowed ports and records this one as "not yet declared, Epic 4"; there is no trait, no implementation, no call site. Epic 4 both declares it and plugs in the first implementation. A fourth port would require a new AD, not an inline decision.
- **Two-tier vocabulary already reserves `ai_config` with Override semantics**, and the binding decision recorded alongside it is that the override is **per field** (a map of field key → value, the shape `glossary/` uses), not whole-struct replacement. The generic `config_value` table serves only global-only kinds, so an Override kind cannot be written through it — AI config needs its own table, and the migration rule is that each story owns the migration step for the table it needs, added in the same story.
- **AD-21 error shape:** errors crossing IPC carry `{ code, message_key, params, retryable }`; the displayed string resolves from `vi.json`. This is the shape a failed connection test must produce, not an ad-hoc string.
- **AD-47③ provenance:** when an AI result is promoted into the Editor via `⌘⇧↵`, its provenance is recorded as "translated by someone else", not as the user's own translation.
- **The single door onto Glossary data** is `core::glossary::entries_eligible_for_injection(resolver, global, work)` — pending-confirmation entries are never injected, and `load_tier`/`insert_entry`/`confirm_translation` are barred outside `core/glossary/**` by an existing boundary test. Its cost has not been measured: it scans and clones both Glossary tables on every call, i.e. once per sentence translated, and its return value drops the source-tier label while `id` is only unique within one `Store`. Story 4.6 must measure before wiring it into a hot path, and may need its own data shape rather than that return value.
- Dropping FR20 (sync scrolling) left a gap owned by this epic: batch translation may make the AI Translation panel carry a whole chapter, reviving the need to scroll it in sync with the grid — no current FR covers that; decide whether it's needed and raise a new FR rather than "restore" FR20.

## UX & Interaction Patterns

- AI state is always exactly one of five values: not configured · generating (streaming) · done · error · cancelled. "Not configured" is explicitly not an error — the app runs fully without AI (FR77), so the panel just invites configuration.
- Error copy never blames the user — "the provider did not respond," never "you entered the wrong key."
- The prompt inspector separates user-authored prompt text from dynamically injected content, shows which Glossary terms were inserted with their confirmed translations, and summarizes how many were injected.
- Narrow-window layout (Story 4.12) has a fixed panel-sacrifice order (AI Translation yields first, Lookup retreats to the status bar next, Original|Translation never yields) and four breakpoints; only the numbers are calibrated on real hardware, separately per layout preset since they compress differently. Values go into `[A11]` of SPEC.md, closing Q9.

## Cross-Story Dependencies

- Story 4.1 precedes everything else in this epic and already ran right after Epic 3, by the 2026-08-13 reorder decision; Stories 4.2–4.12 now run after Epic 5 and Epic 6, not in epic-number order (no FR was cut or altered).
- Story 4.4 (prompt sets) precedes Story 4.5 (export/import of prompt sets).
- Story 4.6 (`RagInjector`) is the first real caller from `ai/` into `glossary/` — the first live exercise of AD-13's allowed reverse direction. Its output feeds Story 4.7 (prompt inspector) and Stories 4.8–4.9 (translate calls).
- Epic 3 (Glossary) must precede this epic — `RagInjector` depends on confirmed Glossary entries. Epics 7 (TM, completes FR70 in the same `RagInjector` signature), 8 (Reviewer bridge), and 9 (AI Proofreader) all depend on this epic being complete.
