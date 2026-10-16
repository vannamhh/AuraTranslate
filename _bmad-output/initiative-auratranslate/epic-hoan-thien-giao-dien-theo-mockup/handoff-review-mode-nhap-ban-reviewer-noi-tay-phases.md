# Handoff, Review Mode nhap ban reviewer noi tay (plan: story-review-mode-nhap-ban-reviewer-noi-tay-plan.md)

## Pha 1 (Rust) -- xong

Wire moi, snake_case (khong rename):
- `reviewer_import_confirm` tra them `chapters: ReviewerImportChapterMatchWire[]`, sap theo `chapter_ord`:
  `{ chapter_id, chapter_ord, matched_rows, total_rows, unresolved_count }`.
  - `matched_rows` = hang reviewer nam trong nhom co ca hai phia; `total_rows` = so hang reviewer;
  - `unresolved_count` = hang + cau chua xu ly, bang `unmatched_row_ids.len() + unmatched_segment_ids.len()` cua `alignment_open` (ca Rust dang canh).
  - Nhan "Da khop x/y hang . k muc can noi tay": x = tong `matched_rows`, y = tong `total_rows`, k = tong `unresolved_count` (cac Chuong nhap).
  - Chuong can Noi tay = `chapters.filter(unresolved_count > 0)`, da sap theo `chapter_ord`.
- Lenh moi `reviewer_harvest_refresh` (khong tham so) -> `{ harvest_candidate_count: number | null, harvest_error: IpcError | null }`. Khong co Tac pham mo -> loi `work.none_open`; thu hoach loi KHONG la loi cua lenh (nam trong `harvest_error`). Vo `(async)`.
- Core: `ChapterAlignment::match_counts()` + `MatchCounts` (export tu `core::export`).
- Loi doc alignment luc xac nhan nhap (hiem) tra loi `alignment_error` du import da ghi; webview xu ly nhu loi xac nhan thuong.

## Viec dang do cho pha Webview

- `ipc_argument_contract` (`every_registered_commands_business_parameters_match_camel_case_across_the_wire`) DO cho den khi `src/**/*.ts` co lan `invoke('reviewer_harvest_refresh')` (them `const CMD_REFRESH_HARVEST` trong `src/config/reviewerImport.ts`). `REGISTERED_COMMAND_FLOOR` da nang 136 -> 137.
- `scripts/check-commands.mjs` `HANDLER_TABLE`: chua kiem, pha Webview/Tests chay `npm run check:commands` (hoac gate tuong ung) sau khi them adapter.
- `isSummary` trong `src/config/reviewerImport.ts` :~143 can nhan `chapters` (bo gac kieu).
- `src/reviewerImportState.ts`: `afterClosedWithHarvest` doi `openGlossaryQueue` sang `openGlossaryHarvest`.

## Da xanh (Rust)

alignment_contract, reviewer_import_contract, review_harvest_contract (them 2 ca lam moi), ipc_contract (them hinh wire), config_invariants (census export.rs 7/8/8, tong 100/63), harvest_boundary. Khong chay full suite.
Ca moi: `the_import_summary_counts_each_chapter_by_the_same_numbers_alignment_open_reports`, `a_group_joined_by_hand_after_the_import_gives_candidates_when_the_harvest_is_refreshed`, `refreshing_the_harvest_without_a_global_store_reports_the_error_and_without_a_work_fails`.
Chua doi chung do (xoa seam) cho ca moi.

## Pha 2 (Webview: luong nhap + Noi tay) -- xong, chua commit

Da lam:
- `config/reviewerImport.ts`: `chapters` vao `ReviewerImportSummary` + bo gac; `reviewerHarvestRefresh()` (`CMD_REFRESH_HARVEST`) -> `{refresh:{harvest_candidate_count,harvest_error}|null, error}`.
- `alignmentState.ts`: `openAlignmentOverlay(chapterId, flow?)`, `AlignmentFlow {chapterIndex,chapterCount,matchedRows,totalRows,unresolved}`, `alignmentFlow`, `endAlignmentChain()` (Esc), `onAlignmentClosed(listener)` -> unsubscribe (event `{chapterId, flow, chainEnded}`; chi phat khi dong that, `closeAlignmentOverlay` thoat som neu chua mo). PHA 3 (Review Mode doc lai diff + an link) dang ky `onAlignmentClosed` va loc theo `chapterId`, KHONG can sua alignmentState nua.
- `reviewerImportState.ts`: `afterConfirmed` -> con Chuong lech: dong man nhap, mo Noi tay tung Chuong (sau `nextTick`); dong bang nut -> Chuong ke, Esc -> ket thuc chuoi; het chuoi -> `reviewerHarvestRefresh` (BO QUA neu import da co `harvest_error`, theo ma tran: dung o man "Da nhap" co dong loi) -> count>0 mo `afterClosedWithHarvest`, nguoc lai mo lai man "Da nhap". Khop het + thu hoach>0 -> dong man nhap, mo thang thu hoach. Export `reviewerImportMatchTotals`, `reviewerImportAlignmentFlowRan`.
- `main.ts`: hook `afterClosedWithHarvest` -> `openGlossaryHarvest`.
- `ReviewerImportOverlay.vue`: dong "Da khop x/y hang · k muc can noi tay" (sau luong: "lúc nhap"); BO nut Noi tay khoi man done (luong tu mo; lenh `export.alignment.open` van o Cong cu).
- `AlignmentOverlay.vue`: luoi hai cot, hang `?`/`→`, nut "Noi n doan da chon" / "Bo qua doan nay" / "Xem n cho con lai" (dispatch `export.alignment.next`, khong lenh moi) / dong "i / k · ↑ ↓ chuyen · ↵ noi"; dau man ghi file + so khop + "Chuong i/n"; nut dong doi nhan "Sang Chuong ke" khi con Chuong; `onEscape` moi (`HANDLER_TABLE` da them).
- `vi.json`: khoa moi `reviewer.import.matched*`, `alignment.flow_*`, `col_*`, `no_counterpart`, `join_*`, `skip_this`, `see_remaining`, `position_hint`, `next_chapter`; xoa 4 khoa het dung (pending_heading, groups_heading, side_segment, source_missing).

Lech so voi Design Notes (co chu y): thu tu hang van la `alignmentEntries` (chua xu ly truoc, nhom sau), KHONG xep theo `segment.ord`, de giu nguyen phim/con tro va ca `alignment.test.ts` "lists unprocessed ... then groups". Doi thu tu la doi `alignmentEntries` -> can Ice/pha test quyet.

Xanh: vue-tsc, eslint (cac tep dong), check:i18n, check:commands, check:tokens, `alignment.test.ts`.
DO (pha 4 phai chuyen): `reviewerImport.test.ts` (6 ca), `reviewerImportFocus.test.ts` (1), `reviewerImportHooksWiring.test.ts` (2) -- mock summary thieu `chapters` (TypeError `.filter`), wiring doi `openGlossaryQueue` -> `openGlossaryHarvest`. Chua viet vitest cho ma tran moi (luong lan luot Chuong, Esc, refresh, loi thu hoach) va e2e. `ipc_argument_contract` het do (da co invoke refresh). Chua chay `npm run check:commands` HANDLER_TABLE ngoai muc tren: da xanh.
Pha 3 con: `ReviewDock`/panels nhan doan, link Noi tay (dung `data-alignment-open`), the Tac pham + `data-reviewer-import-open` (chua dong vao).

## Pha 3 (Webview: Review Mode + the Tac pham) -- xong, chua commit

Da lam:
- `reviewModeState.ts`: `ReviewPairView.position` (1-based) + `labelKey` (`review.pair_accepted|skipped|changed|same`); export `reviewModeUnresolvedCount` (= unmatched segments + rows); dang ky `onAlignmentClosed` o cap module: Noi tay dong tren dung Chuong dang mo -> `reload()` (loi -> `diff_failed`/`error` san co). Review Mode khong mo lai.
- `ReviewMinePanel.vue`: nhan `Doan N · ...` (span `data-review-pair-label`, `data-review-label` = `accepted|skipped|changed|same`); nhan accepted/skipped cu o cuoi dong BO (khoa `review.label_accepted/skipped` xoa); nhan `manual` giu. `ReviewCopyPanel.vue`: `Doan N` (`review.pair_index`).
- `ReviewDock.vue`: nut `Noi tay (k muc chua xu ly)` (`data-review-align-link` + `data-alignment-open`, dispatch `export.alignment.open`) chi khi k>0; chu thich chan `review.footnote` (`data-review-footnote`).
- The Tac pham: lenh MOI `library.open_work_reviewer_import` (check:commands buoc `@click` chi duoc `dispatch`), dep `openCurrentLibraryWorkThenReviewerImport` (libraryChapters.ts, main.ts, commands/index.ts); nut chi o o dang chon (cung khuon `library.open_work`), `data-library-work-reviewer-import` + `data-reviewer-import-open`; mo Tac pham that bai/bi chan (workPageWork null) -> khong mo hop chon tep; truoc khi mo, tieu diem dat vao nut `data-library-chapters-reviewer-import` cua trang Tac pham de man nhap tra tieu diem ve do. Nut thanh hanh dong `LibraryWorkPage.vue` them `data-reviewer-import-open`.
- `scripts/check-commands.mjs`: `CLICK_FLOOR` 214 -> 228 (gate tu do vi so @click 268; nang theo loi nhan cua gate).
- vi.json: `review.pair_*`, `review.align_link`, `review.footnote`, `command.library.open_work_reviewer_import`.

Xanh: vue-tsc, eslint (tep dong), check:i18n, check:commands, check:tokens, check:gates, commandsRegistry*, libraryChapters/libraryWorks.
DO can chuyen (pha 4): `reviewMode.test.ts` 2 ca (`open: two read-only panels...` va `identical pair carries no ins or del`) vi text panel nay co them nhan `Doan N · ...`. Chua viet vitest moi: nhan doan, link Noi tay hien/an, doc lai sau `onAlignmentClosed`, nut the Tac pham (mo that bai -> khong dispatch), tieu diem tro ve; chua viet e2e.

## Pha 4 (Tests that move) -- xong, chua commit

- Sua: `reviewerImport.test.ts` (them `chapters: []`, mock `alignmentState`/`reviewerHarvestRefresh`), `reviewerImportHooksWiring.test.ts` (-> `openGlossaryHarvest`), `reviewerImportFocus.test.ts` (-> man thu hoach `.gh-panel`), `reviewMode.test.ts` (2 ca nhan `Doan N`).
- Them ca: luong nhap -> Noi tay tung Chuong (11 ca ma tran, trong `reviewerImport.test.ts`), `alignment.test.ts` (hai cot, nhan nut noi, dau man luong, Esc ket thuc chuoi), `reviewMode.test.ts` (nhan doan, link Noi tay, doc lai khi dong, Chuong khac, loi doc lai), `libraryChapters.test.ts` (`openCurrentLibraryWorkThenReviewerImport` 3 ca + nut thanh hanh dong), `libraryWorks.test.ts` (nut tren o dang chon).
- e2e `e2e/specs/story-12-16-reviewer-import-hand-join.e2e.mjs`: chi mo Noi tay tu Cong cu (hop chon tep goc khong lai duoc); CHUA CHAY.
- Bay: `reviewMode.test.ts` co `vi.resetModules()` giua tep -> dung import tinh `alignmentState`, khong `await import`.
- Doi chung: go khoi `onAlignmentClosed` -> `reload` trong `reviewModeState.ts` lam ca "reads the diff again" do; da khoi phuc.
- `npm run test:story 12-16`: 7 tep frontend 189 ca + 6 target Rust xanh. Code Map cua plan them 3 tep test bi bao thieu khai.
