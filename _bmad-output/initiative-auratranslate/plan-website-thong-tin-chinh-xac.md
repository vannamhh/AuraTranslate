---
title: 'Website giới thiệu đúng khả năng AuraTranslate'
type: 'bugfix'
ticket: ''
created: '2026-11-16'
status: 'built'
baseline_revision: 'dbf63880f05fe8009faf1982a735f129417cc86b'
route: 'full'
route_source: 'auto'
risk: 'low'
review: 'quick'
review_source: 'pinned'
lenses_ran: [quick]
review_loop_iteration: 0
context: []
---

<frozen-after-approval reason="human-owned intent — do not modify unless human renegotiates">

## Intent

**Problem:** The standalone Vietnamese landing page mixes implemented capabilities with unsupported performance numbers, installation formats and privacy promises. Broken links, misleading simulation labels and narrow-screen layout problems prevent visitors from understanding and trying the app confidently.

**Approach:** Match public claims to the implementation, add the missing features and workflow guidance, replace fabricated app visuals with actual screenshots, and repair interactions within the existing design. Ice directed autonomous execution and added the requirement to create a separate demonstration Work before photography; capture only original sample content in isolated app data and Library.

## Boundaries & Constraints

**Always:** Keep the website a standalone static page. Preserve its warm dark/light palette and bilingual examples. Explain that dictionary lookup, editing and storage work offline, while optional online AI sends content to the configured provider. Label all sample lookup/translation content as prepared demonstrations. Link to the repository's installation instructions and actual release listing without assuming a particular public release exists.

**Never:** Capture or publish Ice's existing Works, translations, terminology or other test data. Change desktop app behavior, architectural invariants, dependencies or release pipelines. Invent benchmark numbers, competitor prices, minimum OS versions, Linux/Portable availability, direct QuickTranslator data import or automatic AI learning. Do not publish or push.

## I/O & Edge-Case Matrix

| Scenario | Input / State | Expected Output / Behavior | Error Handling |
|----------|---------------|----------------------------|----------------|
| Theme preference | Valid, invalid or inaccessible localStorage | A usable initial theme and working toggle | Storage failure cannot abort other interactions |
| Demonstration | Chinese or English preset | Matching prepared source, translation and terminology labels | No real AI or dictionary request |
| Keyboard navigation | Tab, Enter/Space and Escape | Screenshot gallery and FAQ are operable; open/closed state is exposed | Focus remains usable after closing |
| Narrow screen | 375px viewport | Navigation and demo remain usable without page overflow | Wide content wraps or scrolls in its own container |
| Downloads | macOS, Windows or other platform | Correct repository release links and accurate package descriptions | Generic release entry remains available without detection |

</frozen-after-approval>

## Code Map

- `website/index.html`: replace the fabricated hero app and reviewer visual with actual screenshots; retain the visual palette while rewriting unsupported claims and missing links. Replace competitor comparisons with a factual workflow/format overview; no competitor research needed.
- `website/app.js`: theme storage currently unguarded; lookup triggers are pointer-only; English terms are labelled HanViet; FAQ state lacks accessible relationships.
- `website/style.css`: header has no narrow-screen adaptation; playground inline grid overrides the responsive workspace rule. Reuse existing tokens and card styles.
- `.github/workflows/release.yml`, `.github/release-notes.md`, `INSTALL.md`: universal DMG, MSI with WebView2, bundled dictionaries, unsigned packages and checksum instructions.
- `src/aiConfigState.ts`, `src-tauri/src/core/ai/client.rs`: optional configured online/local AI providers; local models require separate setup.
- `src-tauri/src/core/export/options.rs`, `reviewer_copy.rs`: DOCX one-block/two-column, Markdown, text and supported reviewer reimport.
- `src-tauri/src/core/glossary/exchange.rs`, `src-tauri/src/commands/tm.rs`: CSV/TSV glossary exchange and TMX translation memory.
- `src-tauri/src/commands/segment/history.rs`, `src/i18n/vi.json`, `dict-manifest.toml`: confirmed-version history, actual statuses and dictionary layers. No desktop edits are needed.
- `website/assets/screenshots/`: parent supplies actual app captures of a new Work, using original Chinese/Vietnamese prose and isolated E2E app data/Library. Use workspace.png, glossary.png, translation-memory.png, reading.png and export.png only after they exist. Preserve pixel contents; display accurate captions and a sample-data label. Retain sample-content.json as capture provenance.
- `src/PromptLibraryOverlay.vue`, `src/AiPromptInspectorOverlay.vue`, `src/proofreadHandlers.ts`, `src/ConventionManageOverlay.vue`, `src/modes/ReadingMode.vue`, `src/panels/concordanceState.ts`, `src/config/quickBatch.ts`: implemented prompt library, prompt inspector, optional AI proofreader, Vietnamese conventions, reading bookmarks, Concordance and quick-reference AI. Quick translation exists only in Glossary queue and URL import preview; it does not automatically translate all Chapters.

## Tasks & Acceptance

**Execution:**
- [x] `website/index.html` — correct claims and links; replace fake visuals with supplied screenshots and a keyboard-operable gallery; add concrete descriptions of the features in Code Map, import/export formats, installation and first-use steps. Keep user-facing copy Vietnamese, concrete and concise, without developer implementation jargon.
- [x] `website/app.js` — make theme storage resilient, synchronize accessible interaction states, support screenshot selection and correct preset terminology and simulation labels. Remove hero lookup logic when replacing the mockup. A simple gallery that changes the displayed image is sufficient; no modal is required.
- [x] `website/style.css` — repair header, playground, mockup and downloads across small screens; add focus styles and respect reduced motion where existing effects need it.
- [x] Verify the I/O matrix with focused browser checks, including blocked storage, both presets, keyboard and narrow-screen behavior. Use an existing browser tool and dependencies; add no new test framework.

**Acceptance Criteria:**
- Given a visitor reading the page, when they inspect AI/privacy text, then core offline work and optional provider requests are clearly distinguished.
- Given the download section, when a visitor follows its links, then they reach `vannamhh/AuraTranslate` releases or the existing installation guide, with universal DMG/MSI descriptions and unsigned-package information.
- Given feature descriptions, when compared with the code map, then import/export, glossary, TMX, reviewer workflow and history claims match supported behavior and no unsupported benchmark or package remains.
- Given every internal link, when followed, then its target exists; no community or guide link is a placeholder.
- Given product images, when opened, then they are actual app captures containing only the new demonstration Work; no previous private capture is referenced or retained under website/.

## Implementation Notes

- Ice authorized the baseline commit, then expanded the task to missing features and actual photography. Previous captures of existing test data were removed from website/ and the temporary capture files; replacement captures use an isolated E2E app and a new original Work.

- Live release inspection found `/releases/latest` points to dictionary data; app links now use verified v1.1.0 assets.
- Five new demonstration screenshots and focused browser/interaction checks passed; evidence and existing Windows CI failure: [verification](website-verification.md).

## Plan Change Log

## Review Triage Log

- Quick: no findings. Parent finding: medium → patch — actual browser mixed updated HTML with cached old CSS/JS, overflowing the gallery; versioned asset URLs corrected it and browser/interaction checks passed again.

## Verification

- `node --check website/app.js` and `git diff --check` must pass.
- Check every fragment/local path and repository URL; verify accessible state relationships in the rendered DOM.
- Browser checks at 375px, 768px and 1440px, dark/light themes, both presets, FAQ and keyboard lookup; capture representative screenshots.
- Inspect current CI and latest scheduled e2e status before reporting completion; record unavailable or red evidence accurately. Desktop suites are not required for these isolated static website changes.
