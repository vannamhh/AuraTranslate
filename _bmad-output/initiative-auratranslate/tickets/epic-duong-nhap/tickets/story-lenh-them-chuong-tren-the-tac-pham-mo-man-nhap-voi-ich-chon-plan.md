---
title: 'Story 6.27: Lệnh Thêm Chương… trên thẻ Tác phẩm mở màn nhập với đích chọn sẵn'
type: 'feature'
ticket: '27'
created: '2026-11-15'
status: 'built'
baseline_revision: 'a962efd4ce91db61da61c8a9284bd1daa82b5ce0'
route: 'full'
route_source: 'auto'
risk: 'low'
review: 'quick'
review_source: 'pinned'
lenses_ran: ['quick']
review_loop_iteration: 0
context:
  - '{project-root}/src/AGENTS.md'
  - '{project-root}/e2e/AGENTS.md'
---

<frozen-after-approval reason="human-owned intent — do not modify unless human renegotiates">

## Intent

**Problem:** Adding Chapters to an existing Work means opening the import overlay and picking the Work again in *Đưa vào*, although the user is already looking at that Work's card or Chapter list (EXPERIENCE.md §Đường nhập — màn nhập, "Tác phẩm đã có áp cho mọi tab").

**Approach:** One command `library.add_chapters` (label *Thêm Chương…*) opens the Story 25 overlay with *Đưa vào* = Tác phẩm đã có and that Work picked. Mouse points: the Work card's `⋯` menu, the Chapter list bar of the Work page, and `Công cụ ▾` (AD-62).

## Boundaries & Constraints

**Always:**
- The command only sets the destination and opens the overlay. Locking Tên · Ngôn ngữ nguồn · Thể loại to the Work is Story 25's existing behaviour (`pickedDestinationWork`), reused as is.
- Target Work: in Library, the open Work page's Work, else the Work under the grid cursor; outside Library, the Work open in the app (`openWorkId`). With no target, the overlay opens on Tác phẩm đã có with the picker empty, so the existing `destinationPending` lock and picker show; never a silent no-op.
- A Work created earlier in the session is pickable: when the target id is not in `destinationWorks`, reload the list before the overlay opens, so the fields never show the unlocked values of a stale list.
- The overlay opens on the last-used tab, as Mod+N does.
- Decision (Ice, 2026-11-15): a destination set by Thêm Chương… lasts for that opening only. The next plain open (Mod+N, *Nhập tài liệu*, Library invites) resets to Tác phẩm mới. A pick the user makes by hand in *Đưa vào* stays sticky across opens, as in Story 25.
- Each mouse point is `@click="dispatch('library.add_chapters')"`; the grid card is aimed by its existing `@mousedown`.

**Never:**
- No new Rust command, no change to the import pipeline or to `submit*` functions.
- No bilingual import into an existing Work (Story 28); the Story 25 notice stays.
- No `localStorage`, no new dependency.

## I/O & Edge-Case Matrix

| Scenario | Input / State | Expected | Error handling |
|---|---|---|---|
| Card | `⋯` → Thêm Chương… on Kiếm Lai | overlay open, Đưa vào = Kiếm Lai, three fields show Kiếm Lai's values, disabled | — |
| Work page | Kiếm Lai page, bar button | same as Card | — |
| Tools, Workspace | Kiếm Lai open in Workspace | moves to Library, same as Card | — |
| Tools, nothing to target | Library grid empty, no Work open | overlay on Tác phẩm đã có, picker empty, primary locked | existing `destinationPending` sentence |
| Fresh Work | Work created this session, list loaded before | Kiếm Lai picked after reload | load error: existing `destinationWorksError` alert |
| Plain open after | Thêm Chương… on Kiếm Lai, close, Mod+N | Đưa vào = Tác phẩm mới | — |
| Manual pick after | Thêm Chương… on Kiếm Lai, user picks Tuyết Trung by hand, close, Mod+N | Tuyết Trung stays picked | — |
| Bilingual | Thêm Chương… then a `.csv` | Story 25 notice + switch to Tác phẩm mới | — |

</frozen-after-approval>

## Code Map

- `src/modes/libraryImport.ts` -- `destinationMode` :139, `destinationWorkId` :145, `destinationWorks` :152, `pickedDestinationWork` :164, `loadDestinationWorks` :211, `setDestinationMode` :227 (clears the id and loads the list only when not yet loaded), `setDestinationWorkId` :241. Add one exported `presetImportDestination(workId: string | null)` and a module flag marking the destination as preset; `setDestinationMode`/`setDestinationWorkId` (the user's own picks) clear it, and a plain open resets to `new` while it is set.
- `src/modes/libraryImportFocus.ts` -- `openLibraryImportForm` (sets mode `library`, opens overlay; `library.import.open` and, via `openLibraryImportOnTab`, the invites). The plain-open reset goes on the plain path only. Add `openLibraryImportToAddChapters()` here: resolve the target, preset, then open without passing through that reset.
- Target sources: `libraryWorkPageWork` (`src/modes/libraryChapters.ts:196`), `currentLibraryWork` (`src/modes/libraryWorks.ts:103`), `openWorkId` (`src/openWorkIdentity.ts:8`), `currentMode` (`src/modes/modeState`).
- `src/commands/index.ts` -- register beside `library.import.open` (:1695), dep `openLibraryImportToAddChapters?: () => void` beside :221, same `portMissing` shape. Wire the dep in `src/main.ts:~619`.
- `src/modes/LibraryMode.vue:621-649` -- card `⋯` `DropdownMenu`; add a menu item before *Mở thư mục* (`data-library-work-add-chapters`, `data-menu-close`).
- `src/modes/LibraryWorkPage.vue:449-544` -- `.wp-bar` (search · filter · sort). Add a button at the end (`data-library-chapters-add`). Not the chapter panel (:758+): it renders only with a selection, so an empty Work would have no entry.
- `src/TitlebarToolsMenu.vue:~205` -- copy the `export.reviewer_import.open` `<li>` shape.
- `src/i18n/vi.json` -- `command.library.add_chapters`: *Thêm Chương…*.
- Tests: `tests/frontend/libraryImportEntryMatrix.test.ts` (Story 25 matrix), `titlebarToolsMenu.test.ts`. e2e: `e2e/support/importForm.mjs` (`createWorkThroughForm` :111), model spec `e2e/specs/story-6-26-name-autofill.e2e.mjs`.

## Tasks & Acceptance

**Execution:**
- [ ] `src/modes/libraryImport.ts`, `src/modes/libraryImportFocus.ts` -- preset + target resolution + reload-if-missing.
- [ ] `src/commands/index.ts`, `src/main.ts` -- `library.add_chapters` and its dep.
- [ ] `src/modes/LibraryMode.vue`, `src/modes/LibraryWorkPage.vue`, `src/TitlebarToolsMenu.vue`, `src/i18n/vi.json` -- three mouse points, one label.
- [ ] `tests/frontend/libraryAddChapters.test.ts` (new) -- each matrix row on the resolver, preset and plain-open reset, including a target absent from a loaded list; extend `titlebarToolsMenu.test.ts` for the new item.
- [ ] `e2e/specs/story-6-27-add-chapters.e2e.mjs` (new) -- the ticket's verify: create Kiếm Lai with a set language and genre, click Thêm Chương… in the card menu, assert Đưa vào and the three disabled fields.

**Acceptance Criteria:**
- Given Kiếm Lai exists, when the user picks Thêm Chương… in its card menu, then the overlay shows Tác phẩm đã có = Kiếm Lai and its three fields carry Kiếm Lai's values.
- Given the Kiếm Lai page is open, when the user clicks Thêm Chương… in the Chapter list bar, then the same holds.
- Given `check:commands`, when it runs, then `library.add_chapters` has a mouse point and is not in the pointer debt.

## Implementation Notes

- The preset is a plain module `let destinationIsPreset` beside the destination refs; `setDestinationMode`/`setDestinationWorkId` (the user's own picks) clear it. `openLibraryImportForm` calls `resetPresetImportDestination()` first, so Mod+N, *Nhập tài liệu* and both invites (via `openLibraryImportOnTab`) reset; `openLibraryImportToAddChapters` opens without passing through it.
- The Work list reloads before the overlay opens when it never loaded, last failed, or lacks the target.
- Tools item sits before *Nhập bản reviewer*; card item before *Mở thư mục*; Work page button closes `.wp-bar`.
- The Work page unit case mocks `libraryChapters` with a ref: opening a page for real runs `openWorkById` (flush + IPC). The Bilingual matrix row rests on Story 25's matrix case, which drives the same destination refs by hand.
- e2e: the paste helper creates a Work with no genre, so the spec picks a genre first (the overlay keeps it across a close) and asserts the locked genre equals it; an empty genre had made the first run red.
- Counter-checks: deleting the `presetImportDestination(target)` call turns 6 of 10 unit cases red; deleting the plain-open reset turns the 2 reset cases red. Not done for e2e.
- Ran: the 3 unit files (12 + 23 + 11), `check:commands`, `check:i18n`, e2e `story-6-27` (card menu and Work page bar, language and genre pinned to picked labels) and `story-6-25` on one `--features wdio` build.

## Plan Change Log

## Review Triage Log

Pass 1 (quick, loop 0): high 0 · medium 1 · low 3 · false 0 · maybe-false 0 · rejected 1.

- low · patch — e2e language assert is only "non-empty": the fixture Work keeps `auto` on `'x'`, so the case passes for any Work's language; pin a picked chip's label as with genre.
- medium · patch — AC2 Work page bar button: the only Work page case mocks `libraryChapters`, and the e2e covers the card menu only, so no case reaches `[data-library-chapters-add]`; add the page path to the e2e.
- low · patch — unit "nothing to target" runs in `workspace`, so the Library branch with an empty grid and no page never runs; add a `library` mode case.
- low · patch — matrix row Bilingual has no case: nothing shows `switchBilingualImportToNewWork` after a preset ends at `new` and stays `new` on a plain open; add one.
- low · reject — plan Execution boxes unticked: the fix is an edit of this build's plan (Implementation Notes was filled after the diff was staged).

## Design Notes

One command for three places keeps one label and one shortcut row; the resolver mirrors where the user is looking (page, cursor, open Work). Destination state already persists across overlay opens in module refs (Story 25), so the preset is the only new write.

## Verification

**Commands:**
- `npx vitest run tests/frontend/libraryAddChapters.test.ts tests/frontend/titlebarToolsMenu.test.ts tests/frontend/libraryImportEntryMatrix.test.ts` -- expected: pass
- `npm run check:commands && npm run check:i18n` -- expected: green
- `npm run test:e2e -- --spec e2e/specs/story-6-27-add-chapters.e2e.mjs` (outside the sandbox) -- expected: pass
