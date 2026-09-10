<!-- bmad:context -->
<!-- Verified 2026-09-10 against 39ae75d. Managed by bmad-project-context; edits inside this block are replaced on refresh. -->

## scripts/ — the rules of a GATE

Thirteen `check:*` gates enforce declarative claims across the WHOLE TREE (*"no hard-coded colour anywhere"*) — a role no single test can carry. Adding a gate means editing THREE lists: `package.json` · `.github/workflows/ci.yml` · `.githooks/pre-push`, and `check:gates` guards all three.

## Conventions that differ from defaults

- The exit code is the verdict. No gate logs and carries on.
- 🔴 An infrastructure failure is NOT a red check: if a file cannot be read ⇒ `abort()` and exit non-zero with the sentence *"this is an infrastructure failure, not a pass"*. Never report a result that does not exist.
- 🔴 No verdict may read its parameters from the very thing it is checking. The WCAG floor, the role list, the exclusion list — frozen IN the script. Measured: all three escape routes gave exit 0 while the product carried a 4.245:1 contrast pair.
- Population floor: *"an empty tree is not a clean tree"*. A gate counts its files and `abort()`s below the floor; the floor sits at ~80–85% of the real count. The floor is a LOWER bound, so surplus files never make a gate red — they only make the floor meaningless. Adding files to `src/**` means revisiting the floor.
- Plain Node, no bash — `npm run` goes through `cmd.exe` on Windows, and a gate that guards half the platforms cannot guard NFR14.
- No npm dependency for a gate. The TOML/CSS parsers in this directory are hand-written strict subsets, and syntax outside the subset ⇒ FAIL, never skip.
- A NEW gate must carry a SELF-CHECK proving it CAN go red and does not go red wrongly — a gate that has never been red is a gate nobody knows is running.
- ⚠️ A green self-check proves the gate CAN go red. It does NOT prove the real assert runs through the gate's own filter. `cleanup_boundary.rs` declared a `#[cfg(test)]` filter with two passing self-checks while its two real asserts scanned raw `code_lines`, so a call living inside a test block counted as the real call — the self-checks made the gate LOOK guarded. When you add a filter, add a case proving the REAL assert uses it.
- A gate that scans source must anchor on something both spellings share. `code.contains("run_import(")` does not match `run_import_with_order(`, and in Story 6.2 the positive-verification case then ratified that blind spot as the specification. A positive check written from the gate's own behaviour will sign off its own gaps — seed the actual violation instead.

## Known pitfalls

- ⚠️ Only 4/13 gates have a self-check today (`check-gates` Check C · `check-layout` Check D · `check-panel-refs` Check C · `check-debt-owner` Check B). The other eight have not proved they can go red — don't read a green run from them as a guarantee. Owned debt item in `deferred-work.md`.
- ⚠️ `abort()` is not universal either: `check-panel-refs.mjs:78` has the wrong shape (exit 2, different wording), and `check-scope`, `check-scope-bundled` and `check-dict-manifest` have no `abort()` at all — they call `process.exit(1)` bare, so an infrastructure failure there reads exactly like a red check.
- A gate without the `check:` prefix must appear in all three lists — Check F guards that specifically, because Checks A and D only walk `check:*` names. ⚠️ The `REQUIRED_SCRIPTS` table has EXACTLY ONE entry today (`test`). `test:e2e` is the second gate and deliberately sits OUTSIDE the table (`pre-push` excludes the e2e suite on purpose), so it is guarded in only TWO of the three lists: delete the `npm run test:e2e` step from `ci.yml` and all six checks stay green. Don't read a green `check:gates` as "the three lists agree." (No owner yet: Story 3.9 closed without changing a line of `REQUIRED_SCRIPTS`.)

<!-- /bmad:context -->
