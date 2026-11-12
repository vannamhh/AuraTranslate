---
title: 'Xác nhận câu không chậm dần theo số câu đã sửa trong phiên'
type: 'bugfix'
ticket: '18'
created: '2026-11-12'
status: 'built'
baseline_revision: '22c66f0b9f3ab4baf2705107f4aa0228c94fef7b'
route: 'full'
route_source: 'auto'
risk: 'medium'
review: 'quick'
review_source: 'pinned'
lenses_ran: ['quick']
review_loop_iteration: 0
context:
  - '{project-root}/_bmad-output/initiative-auratranslate/research-go-cham-luoi-subgrid-webkit/handoff-xac-nhan-song-song.md'
---

<frozen-after-approval reason="human-owned intent — do not modify unless human renegotiates">

## Intent

**Problem:** Mỗi lần `editorSegments` đổi (kể cả lượt `Mod+Enter` chỉ đổi `status`), `restoreEditedText` gọi một `querySelector` toàn lưới cho MỖI câu trong `editorEditedText`, tập chỉ lớn lên trong phiên; ở Song song 320 câu, luồng chính bị chặn 11–27 ms ở mốc 0 lên 108–126 ms ở mốc 300.

**Approach:** Đo ba bản vá thử cùng điều kiện, trình Ice chọn, rồi làm bản Ice chọn: (a) chỉ chép lại câu mà lượt cập nhật thật sự đổi `target_text`, chép toàn bộ khi lưới dựng lại hoặc thứ tự id đổi; (b) tỉa tập "đã sửa trong phiên" sau khi lưu (đụng hợp đồng "DOM sở hữu chữ" của Story 2.3); (c) giữ ngữ nghĩa cũ, thay N lượt `querySelector` bằng một lượt lập chỉ mục id → ô mỗi lần chép lại.

**Decision:** 2026-11-12, Ice chọn (a) sau số đo Pha 3 của handoff.

## Boundaries & Constraints

**Always:** Ranh giới của vé giữ nguyên: chữ chưa lưu sống sót qua kéo panel, gộp/tách câu, mở lại Chương; AI/TM điền vào câu đang gõ dở thì ô vẫn hiện chữ đang gõ; không ghi vào ô đang giữ tiêu điểm; hợp đồng flush và auto-save giữ nguyên (AD-35, FR100); gạch chân quy ước và soát lỗi vẫn vẽ như trước (`repaintProofread()` vẫn chạy sau lượt chép lại). Số đo trước và sau cùng máy, cùng bản dựng (debug `--features wdio`), cùng Chương 320 câu, ghi tải máy.

**Never:** Không commit bộ đếm hay instrumentation đo. Không tự chọn hướng: HALT sau khi đo, trình bảng số cho Ice. Chọn (b) mà phải đổi hợp đồng của Story 2.3 thì dừng, trình Ice việc chia vé, không làm tiếp trong vé này. Không đụng lỗi gõ có dấu trên Windows (spike 17, bug 19).

## I/O & Edge-Case Matrix

| Scenario | Input / State | Expected Output / Behavior |
|----------|--------------|---------------------------|
| Xác nhận chỉ đổi trạng thái | 200 câu đã sửa, `Mod+Enter` một câu | Số lần tra ô bằng ở 20 và 200 câu đã sửa; không ô nào bị ghi |
| AI/TM điền câu đang gõ dở | `replaceEditorSegment(id, {target_text})`, id có chữ chưa flush | Ô hiện chữ đang gõ |
| Lưới dựng lại | 20 câu sửa chưa lưu, gỡ rồi gắn lại GridPanel | Cả 20 ô hiện chữ đã gõ |
| Gộp/tách câu | Thứ tự id đổi, có câu sửa chưa lưu ở khối khác | Các ô đó hiện chữ đã gõ |
| Ô đang giữ tiêu điểm | Câu đang gõ có trong tập cần chép | Không ghi, caret đứng yên |

</frozen-after-approval>

## Code Map

- `src/panels/GridPanel.vue:1250` `restoreEditedText` + `watch([editorSegments, gridRoot], …, { flush: 'post' })` ở `:1263` -- seam của vé. `targetCellById` `:661` tra theo khối khi có `rowIndex`, không thì tra cả lưới.
- `src/panels/GridBlock.vue:317-354` -- ô `tgt` render `{{ s.target_text }}` dưới `v-memo` chứa `s`; Vue chỉ ghi chữ ô khi chuỗi `target_text` của câu đổi, hoặc khi ô gắn mới (khối khoá theo `block.index`, hàng khoá theo `s.id`, nên hàng đổi khối là ô mới).
- `src/panels/editorPanelState.ts` -- `editedText` `:738` (chỉ bị thay ở `:1084`, đổi Tác phẩm; không có `delete`). Các chỗ gán `segments.value`: nạp `:266`, reset `:1061`, `replaceEditorSegment` `:378` (có thể đổi `target_text`), xác nhận `:1613` (chỉ `status`, `unlinked`), `:1721`, `:1767`, `applyRegroup` `:2906` (đổi id).
- `tests/frontend/gridConfirmWorkCost.test.ts` -- khung đếm `gridQueries` quanh một lượt xác nhận với Chương lớn; mở rộng ở đây cho ca 20/200 câu đã sửa.
- `tests/frontend/gridRestoreEditedText.test.ts` -- ca chép lại hiện có (`typeInto`, `clearByTyping`); thêm ca dựng lại lưới.
- `e2e/measure/confirm-trace.e2e.mjs` -- bàn đo AC1 (`AURA_BENCH_VIEW`, `AURA_BENCH_CHECKPOINTS=0,50,100,299`, `AURA_BENCH_FLUSH=0`); `confirm-typing-parallel.e2e.mjs` cho độ dốc mỗi câu.

## Tasks & Acceptance

**Execution:**
- [ ] Đo nền ở `baseline_revision`: `confirm-trace` kiểu `parallel` và `plain`, mốc 0/50/100/299, kèm số lần ghi DOM thật của lượt chép lại mỗi lượt xác nhận (bộ đếm tạm, không commit). Ghi vào handoff, mục Pha 3.
- [ ] Làm ba bản vá thử (a)(b)(c) trên nhánh tạm hoặc `git show` ra bản sao, đo `parallel` từng bản cùng điều kiện với số nền; khôi phục `src/` sau đo (`git diff --stat` rỗng). HALT trình Ice bảng số, độ rủi ro hành vi của từng bản.
- [ ] `src/panels/GridPanel.vue` -- làm bản Ice chọn; xoá các comment ở dòng đã sửa nếu trái luật comment của AGENTS.md.
- [ ] `tests/frontend/gridConfirmWorkCost.test.ts` -- ca: sửa 20 rồi 200 câu, xác nhận một câu chỉ đổi trạng thái, số lần tra ô bằng nhau. Đối chứng: chạy trên mã nền phải đỏ.
- [ ] `tests/frontend/gridRestoreEditedText.test.ts` -- ca dựng lại cả lưới với 20 câu chưa lưu; ca gộp/tách giữ chữ đang gõ. Đối chứng: gỡ thật vòng chép lại thì đỏ.
- [ ] Đo sau sửa: `parallel` và `plain`, cùng điều kiện; ghi số AC1 vào `## Implementation Notes` (bản dựng, tải máy, số câu, số ghi DOM trước/sau).

**Acceptance Criteria:**
- Given Chương 320 câu ở Song song, cùng bản dựng và phiên đo, when xác nhận lần lượt tới 300 câu bằng `confirm-trace`, then khoảng `Mod+Enter` tới con trỏ sang hàng kế ở mốc 50, 100, 299 không vượt mốc 0 quá 15 ms.
- Given mã ở `baseline_revision`, when chạy cùng bàn đo, then số nền có trên cùng máy, cùng cỡ Chương, cùng tải máy với số sau sửa.
- Given số nền, when mốc 299 đã đạt tiêu chí 1 trước khi sửa, then dừng, ghi số vào Notes và không sửa mã (tiêu chí 4 của vé).

## Implementation Notes

- Ice chose (a). `restoreEditedText(only)` in `GridPanel.vue` takes a set of ids; the watcher passes the ids whose `target_text` changed, or `null` (copy all) when `gridRoot` changed, the length changed, or an id differs at the same index. A status-only confirm gives an empty set: 0 lookups, 0 writes, `repaintProofread()` still runs.
- AC1 numbers (Mod+Enter to caret, ms, median of 4 / max). Build: debug `--features wdio`, WKWebView, Vite dev, 8 cores, 320 sentences, all cells mounted, load 3.5-5.8 for the baseline and 3.6-3.9 after. Baseline parallel 26 / 40 / 56 / 172 at checkpoints 0 / 50 / 100 / 299; after fix parallel 29 / 28 / 29 / 26 (max 34); baseline plain 30 / 34 / 34 / 44, after fix plain 30 / 32 / 32 / 26. AC1 (within 15 ms of checkpoint 0) holds at every checkpoint.
- Cell lookups per confirm: baseline 1 / 51 / 101 / 300 at checkpoints 0 / 50 / 100 / 299, after fix 0. DOM writes of the copy pass: 0 before and after, because the bench only confirms and never rebuilds the grid; write cases live in vitest only.
- Prototype (c) (one `querySelectorAll` into a Map) and (b) (prune after confirm) measured the same as (a) at this sample size (4 samples, one run each); (b) was rejected for the Story 2.3 contract risk.
- Block remount on scroll does not exist (`GridPanel.vue` mounts every block), so the diff needs no case for it.
- The id-at-same-index check is what guards a merge or split; the length check is redundant with it but cheap.
- Controls: `gridConfirmWorkCost` new case red on baseline `GridPanel.vue` (first case stays green). `gridRestoreEditedText`: copy loop removed gives 4 red; focus check removed gives 1 red; id check removed (with the length check) gives 1 red (the merge case); `gridRoot` comparison removed gives 1 red (the remount case). Removing only the length check stays green because the id check already catches the shift.

## Plan Change Log

## Review Triage Log

Lượt 1 (quick): high 0 · medium 0 · low 1 · false 0 · maybe-false 0 · bị bác 5.
- low · patch — vòng `restoreEditedText` vẫn duyệt cả `editedText` khi `only` rỗng (O(số câu đã sửa)); sửa: duyệt `only`.
- bác — ca tiêu điểm không kiểm caret: ca đỏ khi gỡ phép kiểm tiêu điểm nên vẫn canh đúng seam; thêm phép kiểm caret là thêm độ phức tạp cho một lỗ không gặp hằng ngày.
- bác — ô Execution chưa đánh dấu: sửa là sửa plan.
- bác — số mốc 299 ở handoff (28/45) khác Notes (26/34): hai lượt đo khác nhau (bản thử và sau sửa); sửa là sửa plan.
- bác — không có test canh "mọi khối luôn gắn": không có lỗi hiện tại, chỉ là rủi ro nếu sau này ảo hoá khối.
- bác — `gridQueries` chỉ đếm truy vấn trên `gridRoot`: mã hiện tại không tra qua khối; nới bộ đếm đổi nghĩa ca cũ (lượt dời caret tra theo khối ở `GridPanel.vue:1483`).
- defer (maybe-false, nếu thật thì medium, có từ trước) — một lượt ghi `target_text` không qua gõ phím vào ô đang giữ tiêu điểm và có chữ chưa flush thì Vue ghi đè chữ đang gõ; chưa rõ chỗ gọi `replaceEditorSegment` nào chạm được ca này.

## Design Notes

Bản (a): trong watcher, so mảng cũ với mảng mới. Độ dài khác, hoặc id ở cùng vị trí khác, hoặc `gridRoot` đổi ⇒ chép toàn bộ như cũ. Không thì chỉ chép các id có `old[i].target_text !== new[i].target_text` và có trong `editedText`. Lượt xác nhận rơi vào nhánh rỗng: 0 lần tra, 0 lần ghi.

Bản (c) giữ nguyên nghĩa: một `querySelectorAll('[data-col="tgt"]')` lập `Map<id, HTMLElement>`, rồi duyệt `editedText`. Vẫn O(số câu đã sửa) phép so `textContent`, nhưng một lượt quét DOM.

⚠️ `browser.keys()` không sinh chữ; e2e phải chạy ngoài sandbox; `cargo test` ghi đè binary wdio, dịch lại `--features wdio` trước khi đo.

## Verification

**Commands:**
- `npx vitest run tests/frontend/gridConfirmWorkCost.test.ts tests/frontend/gridRestoreEditedText.test.ts` -- expected: xanh; ca mới đỏ trên mã nền.
- `AURA_BENCH_VIEW=parallel AURA_BENCH_CHECKPOINTS=0,50,100,299 AURA_BENCH_FLUSH=0 npm run test:e2e -- --spec e2e/measure/confirm-trace.e2e.mjs` -- expected: số đạt tiêu chí 1.
- `npm run test:story 2-18 -- --list` -- expected: liệt kê các test chạm thay đổi.
