# Handoff Pha 4 Tests -> Review (ticket 19)

## Da lam (chua commit)
- Rust test cu doi theo buoc 39: tm_contract, segment_contract (ladder, [Migration; 39] + buoc gia 40, cot/CHECK, 21 cho read_segment_history -> .versions, INSERT SQL them cot xuat xu), pinned_contract (38 buoc), review_harvest_contract, chapter_origin_contract, alignment_contract, project_contract.
- tm_contract: ca "khoi phuc ban cu roi ky khong sua" nay ky vong `self` (xuat xu cua phien ban duoc chon, AD-55), doi ten.
- Moi: src-tauri/tests/segment_version_origin_contract.rs (11 ca: trigger/CHECK, mo kho gia tri la, ky ghi cot, ba phien ban, hai hang trung chu, ban dang soan, khoi phuc other/self/NULL, doc lai sau khoi phuc, segment ve huu, diff + khong baseline_*, ham chon). Ghim dây dung serde_json cua struct `SegmentHistory` (khong qua vo wire).
- Sua san pham do naming_boundary bat: history.rs bien `origin` -> `stored`; Overlay `originKey(stored)`; vi.json `history.origin_line` dung `{label}`. schema.rs doc-comment "dich la phien ban 39".
- Webview: segmentHistory.test.ts (dây moi + nhom 6 ca), segmentHistoryOverlay.test.ts moi (6 ca, gom khong v-html), gridPanelRowErrorPriority.test.ts mock moi; e2e segment-history-restore doc `.versions`; e2e moi story-12-19-segment-history-origin.e2e.mjs (di bang invoke, KHONG CHAY DUOC TRONG PHIEN NAY).
- deferred-work.md: dong hai muc (nhan, diff) bang dong ĐÃ ĐÓNG 2026-10-16.

## Doi chung do (phep go that, da hoan nguyen)
- confirm.rs ghi hang 'self' thay confirmed_origin: 3 ca do. history.rs khoi phuc tra self thay other: 1 ca do. Overlay danh dau theo noi dung: ca hai hang trung chu do.

## Chay
Xanh: segment_version_origin_contract 11/11, segment_contract 219, tm_contract 151, review_harvest 20, project_contract 146, pinned 10, chapter_origin 17, alignment 40, ipc 48, naming_boundary 23; vitest cac tep segmentHistory* + 3 tep lien quan. Vitest toan bo (chay chung luc cargo): 4 do do timeout 5 s (librarySearchScope, settingsFrame, statusBarWorkspace) do tai may, chua chay lai.
CHUA XONG: `cargo test --no-fail-fast` toan bo dang chay nen (log trong scratchpad cargo-full.log), chua doc ket qua; vitest toan bo chua chay lai khi may ranh; e2e chua chay (se can no `Chu: Epic 12` neu khong chay duoc); `check:*` gate chua chay sau khi doi ten bien/vi.json.
