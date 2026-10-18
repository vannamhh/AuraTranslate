# Tests -> Review handoff (story attribution-thanh-man-ay-u, loop 2)

Uncommitted. Test files touched this phase: `tests/frontend/attributionOverlayActions.test.ts` (reworked), `e2e/specs/attribution-focus.e2e.mjs` (removed the history comment). `attributionOverlayUnreadable.test.ts` and `titlebarToolsMenu.test.ts` unchanged from the patch (plan: no delta). No src changes this phase.

## Verification run
- `npx vitest run` on the 3 Attribution files: 52 pass. `npm run test:story 12-21`: same 3 files green.
- `vue-tsc --noEmit` clean; eslint clean on the two edited files.
- `check:commands`, `check:i18n`, `check:tokens`, `check:panel-refs`, `check:layout`: exit 0.
- e2e outside sandbox: `attribution-focus` 1/1 and `glossary-quick-add` 3/3 passed (webview mounted).

## Counter-checks (each a real removal, file restored after; only the named `it` went red)
| Removal | Red `it` |
|---|---|
| add `position: fixed` to `.attr-screen` | `.attr-screen is never position: fixed` |
| add `top: var(--space-titlebar-height)` | `.attr-screen does not depend on the titlebar height` |
| `<StatusBar />` moved out of `.stage` | `the three bottom strips sit inside .stage before .attr-screen` |
| drop `deps.closeAttribution?.()` in `mode.*` | both tab cases (other tab, active tab) |
| drop `:disabled` on `[data-tools-trigger]` | `the Công cụ trigger is disabled ...` only |
| drop `\|\| attributionIsOpen` on `.layout-switch` | `the layout switch buttons are disabled ...` only |
| drop `open_license` branch of Decision 6 watch | `aiming another row with licence text ... swaps the text` (waitFor `code === 'src-b'`) |
| copy error condition back to `!== null` | `a clipboard rejection that arrives after the aim moved shows no error` |
| drop `v-show` on `ld-scrim` | `hides the drawer while the screen is open ...` |
| add `padding-left` to `tr.attr-aimed td:first-child` | `the aimed row rule adds no padding and no box-shadow` |

## I/O & Edge-Case Matrix coverage (all ran and passed)
- Mở màn: `is a screen, not a scrim` + `the title bar shows the Attribution label and three enabled mode tabs` (vitest); e2e shape block (no scrim, label, 3 tabs reachable by `elementFromPoint`).
- Chân bảng: `foot order ...` (2 buttons) + `... no row carries an action button`; e2e `footButtons=2`, `rowButtons=0`.
- Gỡ một tệp nguồn: `a source file removed from the list leaves no row and no error`; `re-aims the first row when the aimed code vanishes`.
- Không nguồn nào: `still opens when no source loaded` (cards, empty, both buttons disabled).
- Không văn bản giấy phép: `the open button follows the aimed row licence text`; `aiming a row without licence text closes the viewer ...`.
- Clipboard từ chối: `shows the error beside the foot buttons ...`; `a clipboard rejection that arrives after the aim moved ...`.
- Hàng đầu được nhắm: `aims the first row on open`; `aims the first row even when the sources load after`.
- Đổi tab chế độ: two App-mount cases (other tab, active Workspace tab) via real click -> `dispatch('mode.*')`; harness injects the real `closeAttribution`.

## Not covered / for review
- Visual layout in the real app (cover to the bottom incl. strips, `p.config-error`, open from the Lookup drawer in a narrow window) remains debt `Chủ: Epic 12`; e2e `glossary-quick-add` only confirms `.modeport` is still pushed up by the strip.
- "Focus follows the new mode" untested (triage row 18, rejected).
- The LookupDrawer case mounts the drawer + `AttributionOverlay` in a host with a stubbed `LookupPanel` carrying a `[data-attribution-open]` button.
