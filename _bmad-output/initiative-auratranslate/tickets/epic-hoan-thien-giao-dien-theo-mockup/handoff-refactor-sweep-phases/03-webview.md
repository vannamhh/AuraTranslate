# 12-22 phase 3 (Webview): ghi chu ban giao cho phase ledger

Khong commit. Chua chay full suite.

## Da doi
- `src/modes/libraryImport.ts` va `src/modes/libraryChapters.ts`: them import + `resetAlignment()` / `resetReviewerImport()` ngay sau `resetGlossaryQueue()` (hai cho doi Tac pham).
- Tep moi `tests/frontend/alignmentReviewerImportResetWiring.test.ts` (2 ca, hanh vi that, khuon `glossaryQueueResetWiring.test.ts`): lam ban state bang `openAlignmentOverlay(null)` + `openReviewerImportPreviewOverlay()` (mock `reviewerImportOpenPreview` -> `ipc_unavailable`), roi nhap Tac pham moi / `openWorkById`, assert overlay dong va status `'unknown'`.

## Doi chung do (go that tung loi goi, chay lai tep canh)
4 phep go (2 ham x 2 tep): ca tuong ung do, ca con lai xanh, assert `expected true to be false`. Da khoi phuc ca 4.

## Kiem chung
vitest: reviewMode, reviewerImport*, alignmentReviewerImportResetWiring, glossaryQueueResetWiring, libraryChapters -> 10 tep, 160 ca xanh. eslint 3 tep sach.

## Phase 3 con lai
Ledger `deferred-work.md` (L13238, L13261, muc dong cho ro `reset*`), dong 🔵 khung-app, full suite mot lan.
