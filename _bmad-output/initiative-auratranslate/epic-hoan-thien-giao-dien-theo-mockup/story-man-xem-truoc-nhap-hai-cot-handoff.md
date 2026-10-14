# Handoff: Pha Rust -> Pha Webview (man-xem-truoc-nhap-hai-cot)

Plan: `story-man-xem-truoc-nhap-hai-cot-plan.md`. Pha Rust xong, chua commit. **D3 = (a) uoc luong dung nhu quet that (Ice, 2026-10-14)**; duong (b) da go khoi Rust va TS.

## Do D3 (ket qua de Ice chon)

Build: `cargo test`, `CARGO_PROFILE_TEST_OPT_LEVEL=2` (dev opt-level=2, khong phai release), may Ice dang tai 3-10 (load average), `resources/dict` that (4 lop: core, thieu-chuu, tran-van-chanh, vietphrase). Du lieu tong hop: 2.000 Chuong x 80 cau = 160.000 cau, 5,28 trieu ky tu, tieng Trung, mau `^第\d+章`, 8 ten rieng moi Chuong dung >= 5 lan (24.518 thuat ngu duy nhat qua nguong), khong loai tru Glossary.

| | (b) chi loc tan suat | (a) dung nhu quet that |
|---|---|---|
| tong (2 luot) | 27,2 s · 25,5 s | 31,8 s · 35,3 s |
| so ~N | 24.518 | 24.474 |
| tra tu dien | 0 | 24.518 thuat ngu duy nhat (cache theo lan chay; khong cache la 63.356 lan) |

- Tra tu dien do rieng: 24.518 lan `lookup_grouped` = 11,4 s (466 us/lan). Cache giam 63.356 xuong 24.518 lan, tuc nhanh hon ~60 %.
- Phan ton nhat KHONG phai tu dien: dem n-gram theo tung Chuong (cung phep dem ma quet that chay) chiem ~25 s cua ~27 s o (b). Dung ca hai cach, 2.000 Chuong mat 25-35 s o opt-level 2; (b) chi tiet kiem ~6-10 s (nho hon 30 %).
- Ban debug (opt-level 0) cung du lieu (mau cu, 524 thuat ngu duy nhat): (b) 83-86 s, (a) 74-75 s. Do tai may nhieu hon chenh lech (a) < (b) o day.
- Tron dung mau 2.000 Chuong that (nhieu ten rieng hon, mau khac) co the doi ti le tra tu dien; con so 24.518 la do tong hop, khong phai tieng Trung that.
- (a) co the tra `Inconclusive` (lop tu dien hong) ⇒ khong uoc luong duoc; (b) khong bao gio.
- Ca hai cung can `ImportEstimateGeneration` de huy; ca hai da huy duoc (kiem moi Chuong va moi thuat ngu).

Da chon (a). Ghi chu ban dau: (b) khong "re" trong thuc te vi dem n-gram la phan chinh; con (a) cho con so dung voi quet that. Neu 25-35 s van qua dai cho dong "~N", huong giam la song song theo Chuong (can them phu thuoc, phai doc giay phep truoc) hoac cat nguong so Chuong duoc uoc luong.

## Wire moi

| Lenh | Tham so JS | Tra ve |
|---|---|---|
| `estimate_import_glossary` | `sourceLang, encoding, chapterPattern` | `GlossaryEstimateWire { outcome: 'estimated'\|'inconclusive'\|'cancelled', count: number\|null }` |
| `cancel_import_glossary_estimate` | khong | void |
| `confirm_bilingual_import` | them `importMismatchedRows: boolean` (cuoi) | khong doi |

- `estimate_import_glossary` doc nguon DANG CHO + `destination` tu `PendingImportSourceState`, `encoding`/`chapterPattern` truyen lai nhu `confirm_import_with_encoding`. Moi loi goi huy loi goi truoc (`ImportEstimateGeneration`, rieng voi `ImportScanGeneration`). Khong ghi `glossary_candidate`/`glossary_entry`. Loai tru: Glossary toan cuc; khi `destination` co: them Glossary Work + moi hang `glossary_candidate` co `candidate_origin='import_scan'` (Work dang mo thi doc qua store dang mo, khong thi `ReadOnlyDb`, khong di tru).
- `count` la `null` khi `outcome != 'estimated'`; khong bao gio `0` khi chua biet.
- Uoc luong chi den tu `estimate_import_glossary`; `ImportEncodingPreview` khong co truong uoc luong (da go).
- TS: `estimateImportGlossary(sourceLang, encoding, chapterPattern)`, `cancelImportGlossaryEstimate()`, `GlossaryEstimateWire` trong `src/config/project.ts`. `glossaryEstimate` chua duoc nhan vao `importPreviewState.ts`.
- Song ngu: KHONG co uoc luong (dung plan).

## Lui nhap hang lech

- `confirm_bilingual_import` (ham thuan Rust) giu nguyen chu ky va van tu choi; `confirm_bilingual_import_with_policy(..., MismatchedRowPolicy)` la ban co co. Vo IPC chon `ImportUnlinked` khi `importMismatchedRows == true`.
- Hang lech nhap thanh MOT segment: nguon = cac cau nguon noi lai (khong cach cho `zh`, mot cach cho `en`), dich = `target_line`; `translation_origin` = `bilingual_import` (hoac `none` khi dich rong), baseline = target, `status` = `draft`.
- Quy tac Ice: "Nhap luon, noi sau" bi KHOA kem ly do khi con bat ky hang lech nao co NGUON RONG; nguoi dung xu ly cac hang do bang Bo qua/cat truoc. Rust van tu choi (`import.bilingual_mismatched_rows`, `count` = so hang nguon rong). Dem bang DTO san co, khong doan: `mismatches.filter(m => m.source_sentences.length === 0).length` (cung vi tu Rust dung: `derive_row(...).source_sentences.is_empty()`, ke ca nguon chi co khoang trang). Khong them truong moi.

## Dau "chua noi"

- Bang `segment_unlinked (segment_id PK, created_at)`, buoc 35 cua `PROJECT_MIGRATIONS`. Dau mat trong cung giao dich khi `confirm_segment` thanh cong va khi `write_regroup` cho segment ve huu (gop, tach). Khong con duong ve huu nao khac (`split_chapter_into_segments` chi chay tren Chuong chua co segment).
- Lan 2 xac nhan tren cau da xac nhan (AC13) khong dung toi bang.
- `ChapterRow.unlinked_count: number` (so cau SONG chua noi) trong `list_chapters`/`rename_chapter`.
- `ChapterSegment.unlinked: boolean` trong `read_open_chapter_segments` va hang moi cua gop/tach. TS `ChapterSegment` o `src/config/segment.ts` CHUA co truong nay (them kieu + vao `isChapterSegmentRow` o pha Webview). `ChapterRow` TS cung chua co `unlinked_count`.

## Giu y

- `HANDLER_TABLE` / lenh phim: khong chay trong pha nay (khong lenh phim moi).
- Dem lenh trong `config_invariants`: `commands/project/wire.rs` 4 plain / 16 async / 14 blocking; toan cay 92 / 61.
- Buoc di tru 35 doi cac ca "ha ve buoc" o `tm_contract.rs` / `review_harvest_contract.rs` / `project_contract.rs` / `pinned_contract.rs`; da sua.
- Khong chay `cargo fmt` tran (dinh bon tep khong lien quan); dung `rustfmt --edition 2024 <tep>`.

# Handoff: Pha Webview -> Pha Tests

Pha Webview xong, chua commit. Gate da chay xanh: `vue-tsc`, `check:lint`, `check:commands`, `check:i18n`, `check:tokens`, `check:panel-refs`, `check:layout`, `check:doc-refs`, `cargo test --test naming_boundary` va `--test config_invariants`. Chua chay e2e, chua nhin man hinh that.

## Cau truc moi

- Ten tep GIU NGUYEN (`ImportPreviewOverlay.vue`, `BilingualImportPreviewOverlay.vue`) nhung khong con la lop phu: mount trong `src/modes/LibraryMode.vue` (KeepAlive nen doi tab khong dong man), bo khoi `App.vue`. Goc `.ip-scrim`/`.bip-scrim` -> `.ip-screen`/`.bip-screen`; `.ip-panel`/`.bip-panel` giu ten nhung bo `role="dialog"`, `aria-modal`, bay Tab (`trapTab` da xoa, mat dong HANDLER_TABLE tuong ung) va nut dong o dau (`.ip-close`/`.bip-close`); nut Huy o chan con.
- `ImportPreviewOverlay.vue`: `.ip-two` = `aside.ip-config` (dich, `dl.ip-summary`: ten Tac pham, ngon ngu, trang thai ban dau, Chuong nhan ra, `[data-ip-glossary-estimate]`; o mau phan tach Chuong) + `.ip-main` (cac tang). Duong URL them `.ip-two-stacked`. Tang chuan hoa BI BO (khoa `tier_normalized_*` va `chapters_count` con trong `vi.json`, khong dung). `ChapterOrigin` xuong tang 4 cuoi (`#ip-tier-origin-title`), chi duong URL. Nhan xac nhan = `confirm_with_count` ("Xac nhan va nhap N Chuong", nhom nghin bang NBSP qua `src/importFormat.ts`).
- Uoc luong: `importPreviewGlossaryEstimate` (`idle|running|estimated|unavailable`) trong `importPreviewState.ts`, chay bang `watch` tren `[overlayOpen, status, preview, selectedEncoding, confirming]` voi debounce 300 ms; moi lan doi huy luot cu (token + `cancel_import_glossary_estimate`). Ap cho text, file va URL; song ngu khong co.
- Song ngu: `.bip-two` (`aside.bip-config` bang cot/bang + `.bip-main` bang ma/so dem/hang lech). Chan: "Xem N cho lech roi nhap" (`import.preview.bilingual_focus_mismatch`, ban tay `focusFirstBilingualMismatch`), "Nhap luon, noi sau" (`import.preview.bilingual_confirm_unlinked`, `[data-bilingual-import-unlinked]`, gui `importMismatchedRows=true`), xac nhan thuong, Huy. "Nhap luon" khoa khi `bilingualImportPreviewEmptySourceMismatchCount > 0` (kem `bilingual_unlinked_locked_reason`) hoac khong con hang lech.
- Man "Nhap tu website": `src/modes/LibraryUrlImportScreen.vue` (`[data-library-url-import-screen]`), state `libraryUrlImportScreenIsOpen` o `libraryImportOverlayState.ts`, lenh `library.import_urls_open`/`library.import_urls_close`. `LibraryImportOverlay.vue` MAT o dan URL (con nut `[data-library-url-screen-open]`); nut nop URL nay nam trong man moi (`[data-import-preview-open]`). `LibraryImportOverlay` van mo nhung `v-show` an khi bat ky man nao trong ba (website, xem truoc, song ngu) mo; tieu diem tra ve qua `nextTick` o ca hai man xem truoc.
- `main.ts` `isBlocked` them `libraryUrlImportScreenIsOpen`, `importPreviewIsOpen`, `bilingualImportPreviewIsOpen`.
- O phai thanh tieu de: `src/titlebarImportSource.ts` -> `[data-titlebar-import-source]` trong `App.vue`.
- Dau "chua noi": `ChapterSegment.unlinked` (bat buoc) va `ChapterRow.unlinked_count` (bat buoc) trong `config/segment.ts`/`config/chapter.ts`, ca hai co type-guard. Luoi: `GridPanel.vue` them lop `cell-unlinked` o cot vach (net cham `border-right`), `data-unlinked="true"` va chu "chua noi" trong `aria-label` o bản dịch (`panel.grid.state_with_unlinked`). Danh sach Chuong: `[data-library-chapter-unlinked]` " · N cau chua noi" trong `LibraryWorkPage.vue`.
- San cong da nang: `TS_FLOOR`/`FILE_FLOOR` (check-commands, check-panel-refs) 105 -> 113, `CLICK_FLOOR` 183 -> 197, doc-refs `FILE_FLOOR` 545 -> 582 (cac gate tu bao troi duoi 80 %).

## Test da sua san / con do

- Da them `unlinked: false` va `unlinked_count: 0` vao fixture (vue-tsc va guard): cac tep `tests/frontend/{editorRegroup*,editorSegmentRule,export*,glossary*,gridPanel*,library*,reading*,segment*,tmConcordance*,chapterWireGuards,configSegmentChapterSegmentsShape}.test.ts`, `support/segmentFixture.ts`. Do cac tep nay hien xanh.
- Con do (51 ca, goc: bo chon `.ip-scrim`/`.bip-scrim`, bay Tab, tang chuan hoa, `.ip-chapters-count`, nut URL trong form): `importPreviewOverlayRender`, `importPreviewBilingual`, `importPreviewDestination` (nut URL "Tai danh sach URL" nay nam o man website), `importPreviewChapters`, `importPreviewBlocks`; chua mo: `importPreviewChapterOrigin`, `importPreviewCleanup`, `importPreviewFiles`, `importPreviewUrls`, `libraryImportOverlay`, `libraryImportBlocksResubmitWhilePreviewOpen`, va e2e (`e2e/support/importForm.mjs`, `story-5-4`, `story-5-5`, `story-12-2`, moi `story-12-6-import-preview`).
- Can case moi: moi hang ma tran I/O, uoc luong (running/estimated/unavailable, huy khi doi mau), "Nhap luon" khoa/mo theo hang nguon rong, `confirmBilingualImportPreview(true)` gui `importMismatchedRows`, tieu diem tra ve nut mo, `isBlocked` chan phim, aria-label "chua noi".

## Con ho

- O phai thanh tieu de KHONG co dung luong cho tep: wire khong mang kich thuoc (chi co ten tep; van ban dan co byte; song ngu co ten tep + so hang). Can mot truong Rust `byte_len` tren `FileImportItemWire` (cham `core/segment/import.rs::FileImportItem` va ba tep test dung struct literal) — chua lam.
- Chua nhin man hinh that (CSS hai cot, gutter so dong cua man website) va chua chay e2e; can mot luot nhin tren WKWebView.
- Ma "domain" cua man website dem bang `new URL().hostname` o TS (chi de hien thi).

## Việc còn dở giao Pha Tests (điều phối, 2026-10-14)

- AC của plan đòi ô phải thanh tiêu đề ghi tên tệp **và dung lượng**. Pha Webview chưa làm được vì dây không mang kích thước: thêm `byte_len` vào `FileImportItem`/`FileImportItemWire` ở Rust (sửa ba chỗ test dựng bằng literal), mirror TS, rồi hiện nó trong `src/titlebarImportSource.ts`. Định dạng theo mockup: `tuyet-trung-han-dao-hanh.txt · 41,2 MB`.
- Sau đó mới tới việc chính của pha: sửa 51 ca vitest đỏ và thêm case cho mọi hàng ma trận I/O, như phần ghi chú Pha Webview ở trên.
