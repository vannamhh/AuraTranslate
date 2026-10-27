# Brief

Repository commit: 7bd1f90ca2ccd15b661850f3cfd60359df6bdc66 (short: 7bd1f90), branch master.

Budgets: 15 files per analyzer question; 10 commits followed per historian question. Validation: normal.

Paths left out of every search: `_bmad-output/`, `_bmad/`, `.claude/`, `.agents/`, `.agent/`.

## Questions

1. (how) How is a request to an AI provider for translating a single segment assembled — which pieces of text go into the system and user messages, in what order — and how is the provider's response read back into a translation?
2. (how) How does translating several segments or a whole chapter with AI run — how segments are grouped into requests, how many requests run at once, how retries and cancellation behave, and how each returned translation is matched back to its segment?
3. (how) How does the Proofreader run — what it sends to the provider, how it splits work into batches, and what it does with each reply?
4. (where) Where are token counts and money cost of AI requests computed and shown, and where does the price data come from?
5. (what touches) What code contributes text to the prompt sent to the AI provider — glossary entries, translation memory, prompt sets/templates, per-Work settings, neighbouring segments, anything else?
6. (how) How is the AI translation and Proofreader path tested — which test targets exercise it, against what (fakes, recorded wire data, live providers)?
7. (why) Why are the request shape and the batching of AI translation the way they are — which commits introduced and changed them?
