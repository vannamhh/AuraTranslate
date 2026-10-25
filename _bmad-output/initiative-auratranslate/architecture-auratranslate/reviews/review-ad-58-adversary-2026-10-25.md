# AD-58 adversarial review (2026-10-25)

Scope: spine lines 1139-1162 (AD-58), AD-48 Rule 2, AD-24, `config_invariants.rs` ~360-440, `SECURITY-NOTES.md` capabilities table, `<header class="titlebar">` in `src/App.vue`, Tauri `drag.js` and `plugin.rs`. Nothing edited.

Verdict: the permission half (Rule 2) is tight. The drag-region half (Rules 3, 4) is under-specified: two units can follow every Rule and still diverge or regress, and two of the prescribed guards cannot fail for the reason they exist.

## Facts that drive the findings

- `drag.js` is injected on every platform. On Windows `<header class="titlebar">` becomes draggable, with double-click maximize on `mousedown`, inside the webview and below the native OS bar. Rule 1 and Rule 4 scope the overlay and margin to macOS; Rule 3 does not.
- On a drag-region hit, `drag.js` calls `preventDefault()` and `stopImmediatePropagation()` on `mousedown` at `document`, registered before app code.
- `isDragRegion` only treats these as blocking: tags A/BUTTON/INPUT/SELECT/TEXTAREA/LABEL/SUMMARY, contenteditable, `tabindex` other than -1, and a fixed role list. A `div @click` with no tabindex, or with `tabindex="-1"`, is a drag region under `deep`.
- All 22 overlay/drawer files use `position: fixed; inset: 0` (SettingsOverlay:739, LookupDrawer:143, AttributionOverlay:386) with an own stacking context, so they cover `.titlebar`.
- `TitlebarToolsMenu.vue:69` closes its popover with a `document` `mousedown` listener.
- `detectIsMac()` is `/mac/i` on `userAgentData.platform ?? navigator.platform`. There is no test that ties it to the `titleBarStyle` the OS actually applied.

## Findings

### F1 (high) Swallowed mousedown on the header: the tools menu will not close
- Unit A obeys Rule 3: it puts `data-tauri-drag-region="deep"` on the header. Unit B obeys Rule 3: it never touches the attribute, and adds a non-button clickable (a `<div @click>` or `<span @click>` status chip) inside the header.
- Pair, case 1: with the tools popover open, the user presses blank header space. `drag.js` calls `stopImmediatePropagation` at the document level before Vue's `onDocumentMouseDown` runs. The menu stays open while the window starts dragging. Focus also never moves, because `preventDefault` blocks the blur.
- Pair, case 2: the new `div @click` has no tag, role or tabindex, so `isDragRegion` is true for it. Its click is converted into a window drag and the click handler never fires.
- Rule 3 says clickables "block drag by themselves" but only lists button, `role="tab"` and `tabindex >= 0`. It omits `tabindex="-1"` (the roving-tabindex menu items) and any non-semantic clickable.
- Fix: add to Rule 3 that every interactive element in the header is a native `<button>`/`<a>`/`<input>`, or carries a role in drag.js's `INTERACTIVE_ROLES`. Add that any other `document`-level `mousedown` listener the header depends on must use the capture phase. Make the test iterate the header's `@click` bindings.

### F2 (high) The drag-region test can pass while the region sits on the wrong element
- The prescribed test counts `data-tauri-drag-region` in `src/` and expects 1. The count is global, so Unit A moves the attribute to `.titlebar-right`, or to a new full-window wrapper (`.shell`), and the count is still 1.
- A wrapper with `deep` covers the whole window, so any non-clickable area becomes a drag handle, including text in panels. Unit B writes `data-tauri-drag-region="false"` on a child (Rule 3 only forbids it for clickables) and the count of 1 is satisfied by that "false" instead of the real region.
- The counter-check (remove the attribute from the real header) does catch removal, but not displacement or the "false" substitution.
- Fix: the test must parse `App.vue`, require exactly one occurrence with value exactly `"deep"` (reject `:data-tauri-drag-region` bindings and `"false"`) and require the element to be `<header class="titlebar">`. The counter-check must also run a move-to-wrapper variant.

### F3 (high) Full-screen overlays cover the title bar: no drag, content under the traffic lights
- Unit A builds a new overlay with `position: fixed; inset: 0`, as the 22 existing ones do. Unit B adds a drag region to the overlay header, which would break Rule 3 and the count, so B follows the Rule and adds nothing.
- Result: while any overlay is open the title bar is covered. Dragging and double-click maximize do nothing, and the window cannot be moved for the length of an import or export. Overlay content, such as a close button or title at top-left, is painted under the native traffic lights (`trafficLightPosition`) on macOS only, where the lights are drawn over the webview.
- The AD is silent on both z-order and the safe area. The Rule 4 margin applies only to the titlebar.
- Fix: add a Rule that overlays either start below `--space-titlebar-height`, or reserve the same traffic-light inset token. State that the drag region is not required while an overlay is open, or require the overlay top band to be inert (`pointer-events` pass-through).

### F4 (medium) Windows: a second, unspecified drag surface
- Windows keeps the OS title bar (Rule 1). `drag.js` still makes the webview header draggable, with `mousedown` maximize on double-click. Windows users get two draggable bars.
- Unit A (Windows e2e or manual) sees the header act as a drag region. Unit B treats the OS bar as the only region and puts selectable or double-click-sensitive content in the header (for example, a double-click-to-edit chapter title).
- Rule 3 never says whether this is accepted. NFR14 is left to the Rule 4 detector, which scopes only the margin.
- Fix: state "the region is active on every platform; Windows double-click on the header maximizes too" as an accepted behavior in Rule 5, or gate the attribute by platform. Record it as a `B7` real-use check.

### F5 (medium) Text selection and focus inside the header
- `preventDefault()` on `mousedown` blocks text selection and also focus change. Unit A makes status text (`titlebar-status`, with `role="status"`) copyable. Unit B adds an `<input>` search box to the header, which is safe because inputs block drag. Neither violates a Rule, but the `deep` region makes plain status text unselectable and non-focusable. Pressing the header does not blur a focused editor in the work pane, so Esc and shortcut routing target the wrong place.
- Fix: add to Rule 3 "text in the header is not selectable by design; an editor focus is retained on a header press" so the next unit does not treat it as a bug.

### F6 (medium) Rules 1 and 4 have no guard (not enforceable as written)
- Rule 1 (`titleBarStyle`, `hiddenTitle`, `decorations: true`, `trafficLightPosition`) has no test. Unit A changes `--space-titlebar-height` and leaves `trafficLightPosition` fixed, and the lights misalign. Unit B switches `decorations` to `false` (rejected option 3) with all five permissions untouched, and every gate stays green.
- Rule 4 ("a spacing token, no number in a component") is covered by `check:tokens` only if the token usage is real. The macOS-only condition rests on `detectIsMac()` (`navigator`), so a WebView that reports `Mac` on a Windows-hosted test passes both branches.
- Fix: add one `config_invariants` test pinning the window block (`decorations == true`, `titleBarStyle`, `hiddenTitle`, and `trafficLightPosition.y` derived from or asserted against the token value). Add a Vitest case that drives `detectIsMac` through its injection seam for both platform strings and asserts the margin class differs.

### F7 (medium) The capability test passes while the surface widens
- The assert fixes strings, but `core:window:allow-start-dragging` and `allow-internal-toggle-maximize` take a `label` argument (`get_window(window, label)`). The ACL grants them for every window label the capability reaches. `windows: ["main"]` and AD-24 hold today, but the AD relies on the single-window assumption without a test that no second window is created (for example a Rust `WebviewWindowBuilder` for a future export preview).
- The test also checks `windows == ["main"]` for the capability file, but not that `tauri.conf.json` lists only `main`. A new window plus the existing capability would not match, but a window labelled `main` created programmatically would pass.
- The frontend ban is a source scan of `@tauri-apps/api/window` only. `@tauri-apps/api/webviewWindow` re-exports `getCurrentWindow`, and `window.__TAURI_INTERNALS__.invoke('plugin:window|start_dragging')` needs no import at all. A second caller passes the scan.
- Fix: extend the Rule 3 scan to `@tauri-apps/api/(window|webviewWindow|webview)` and the strings `plugin:window|` and `__TAURI_INTERNALS__`. Add a `config_invariants` assertion that `app.windows` has exactly one entry with label `main`.

### F8 (low) Rule 2 wording and doc drift
- The text says "đúng năm" but SECURITY-NOTES.md §`capabilities/main.json` is titled "ba tập quyền" and its table lists three rows. Rule 2 only adds two rows.
- The file description in `capabilities/main.json` says "Ba tập quyền … TỐI THIỂU thật". Both are false after the change.
- The title of AD-58 says "đúng hai quyền" while the body says "đúng năm quyền". The intent is "two new, five total", but a reader can build either count.
- Fix: list the five strings verbatim in Rule 2, and require the SECURITY-NOTES heading and the `description` field to be updated in the same commit.

### F9 (low) Rule 4 fullscreen: the margin stays while the lights are hidden
- Accepted in the AD, with the cost named. Unit A pins the margin and the UI shifts 0 px in fullscreen. Unit B, on a laptop with a notch or in a Spaces fullscreen, shows an empty gap.
- Fix: record it as a `B7` real-use check rather than leaving it unowned.

## Guards that could pass on both branches

- Rule 3 attribute-count test: passes with the attribute on the wrong element (F2).
- Rule 3 import scan: passes for `webviewWindow` and for raw `invoke` strings (F7).
- Rule 4: no guard that distinguishes mac from non-mac output (F6).
- Rule 1: no guard at all (F6).
- Rule 2 test is sound: it fails when a string is added or removed, and `capabilities_directory_holds_exactly_the_one_reviewed_file` closes the second-file route. Residual: it does not check object-form permissions with `allow`/`deny` scopes, but `.expect("quyền dạng chuỗi")` already rejects them.
