/**
 * Pure floor judge — the JS counterpart to `boundary_scan::assert_population_floor`
 * (`src-tauri/tests/support/boundary_scan.rs`). Same two checks, same reasoning:
 *
 *   1. `live < floor` — the tree is too small to be real (an empty tree must not read
 *      as a pass; AD-26/AD-44 ④).
 *   2. `floor < 0.8 × live` — the floor has drifted below 80% of the real population and
 *      has stopped guarding anything.
 *
 * Pure — no file I/O, no `console`, no `process.exit`. Each gate passes ITS OWN live
 * count (there is no single population root shared across gates) and decides how to report a bad verdict (`fail(...)`, `abort(...)`, …).
 */

/**
 * @param {number} floor
 * @param {number} live
 * @param {string} label - the `*_FLOOR` constant's own name, printed verbatim.
 * @param {string} population - what was counted, printed verbatim (Vietnamese, matches
 *   the gate's own wording for the population).
 * @returns {{ok: true} | {ok: false, message: string}}
 */
export function judgeFloor(floor, live, label, population) {
  if (live < floor) {
    return {
      ok: false,
      message:
        `chỉ tìm thấy ${live} ${population} (sàn \`${label}\` = ${floor}). Cây quá nhỏ để ` +
        'là thật — một danh sách rỗng làm phép đếm này im lặng bằng 0.',
    }
  }
  if (floor * 5 < live * 4) {
    const ceil85 = Math.ceil((live * 85) / 100)
    return {
      ok: false,
      message:
        `sàn \`${label}\` = ${floor} đã trôi dưới 80% số thật ${live} ${population} — nâng ` +
        `lên ceil(0.85 × live) = ${ceil85}.`,
    }
  }
  return { ok: true }
}
