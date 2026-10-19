# 12-22 phase 2 (Rust): ghi chu ban giao cho phase Webview / ledger

Khong commit. Chua chay full suite.

## Do (build: `bench-release` = release + panic unwind, Homebrew-free rustup stable x86_64-apple-darwin; may Ice, may rong, 1 luong test)

Dung `--profile bench-release`, KHONG `--release` nhu plan ghi: `--release` xung dot ten tep lib (panic=abort) nhu Cargo.toml da canh bao. Lenh:
`cargo test --profile bench-release --locked --manifest-path src-tauri/Cargo.toml --test story_22_measure_export_tm -- --ignored --nocapture --test-threads=1`
(bien dich bin LTO mat ~25 phut lan dau.)

`export_preview`, 1 Tac pham, 1 Chuong 2.000 cau (nguon CJK ~35 ky tu, ban dich ~70 ky tu, tat ca `draft`), `ExportContent` mac dinh, 200 lan/dinh dang, goi ham core truc tiep (khong qua IPC):
| dinh dang | rows | p50 ms | p95 ms | max ms |
| plain | 6 | 10.040 | 15.626 | 19.753 |
| markdown | 6 | 9.016 | 13.869 | 24.147 |
| docx_two_column | 6 | 6.977 | 9.572 | 12.414 |
| docx_one_block | 1 | 14.267 | 23.342 | 88.675 |
=> p95 lon nhat 23.3 ms <= 50 ms: KHONG vuot nguong, khong can Ice chon. Luu y: `docx_one_block` co 1 mau ngoai le 88.7 ms trong 200 lan (p95 van 23 ms).

`tm_export_counts` (goi ca `all` + `mine`, tuc 2 lan doc tron tang), moi tang 200.000 cap (Work store va Global deu 200k; nua `self`, nua `other` => all=200000, mine=100000), 7 lan:
| tang | p50 ms | max ms |
| work | 2314.6 | 3169.6 |
| global | 2560.9 | 2838.0 |
Day cung la thoi gian giu `OpenWorkState` cua lenh (ham chay duoi khoa; wire la `(async)` o commands/tm.rs:989 nen khong chan luong chinh). Chi ghi so, khong sua. Luu y quan the: plan noi "200.000 cap"; tep do 200k MOI tang (xau nhat), builder cu story_e7_r4_population la 100k/tang. Ghi dung "200.000 cap/tang" khi dong L13238.

## Da doi
- Tep moi `src-tauri/tests/story_22_measure_export_tm.rs` (2 ca `#[ignore]`, in `S22_EXPORT_PREVIEW` / `S22_TM_EXPORT_COUNTS`, khong assert nguong).
- `src-tauri/src/commands/chapter.rs`: go `merge_chapter_into_previous` + `_indexed`. `MergeStep::Merged { into }` thanh `Merged` (truong `into` het ai doc, rustc canh bao dead_code). `chapter_at_first`, `NoPrevious` van con (move_chapter, merge_into_previous_tx dung).
- Doc-comment doi ten sang `merge_chapters`/`merge_chapters_indexed`: `core/store/schema.rs` (3), `commands/lifecycle.rs`, `commands/project/mod.rs` (2).
- Test chuyen sang `merge_chapters(.., &[truoc, sau])`: `project_contract.rs` (18 cho goi + none_open `&[1, 2]` + `_indexed` + 2 ca not_found dung `&[a_id, khong_ton_tai]`), `alignment_contract.rs` (3), `reviewer_import_contract.rs` (3), `review_harvest_contract.rs` (1). Go ca `merging_the_first_chapter_touches_no_row_and_is_a_named_error` (at_first, het duong toi). rustfmt chay tren project_contract/alignment_contract/tep do (goc sach fmt).
- Ca `moving_or_merging_an_unknown_chapter_id_never_renumbers_a_sparse_ord_sequence`: them `let a_id = opened.chapter_id;` (merge_chapters kiem NotFound truoc moi cau ghi nen van canh duoc).

## Kiem chung
`cargo test` (debug) 5 tep: project_contract 40, alignment_contract 12, reviewer_import_contract 145 (dong thu 3 ghi la ten test, ket qua 145), review_harvest_contract 20, export_content_options_contract 27, tat ca xanh, 0 canh bao. `grep -rn merge_chapter_into_previous src-tauri --exclude-dir=target` rong. (Luu y: thu tu in trong log khong chac khop thu tu tep; tong 5 ket qua xanh.)

## Chua lam / phase sau
- Webview `reset*`, ledger `deferred-work.md` (L13238 -> `Chu: Story 12.22` roi dong bang so tren; L13261 dong bang so tren; muc dong cho ro `reset*`), dong 🔵 khung-app: khong dung.
- Chua kiem tra 4 so ca `project_contract` con ghi chu ten `merge_chapters` trong doc-comment test (da doi ten bang tim-thay-thay-the; doc o 5039 va 5318 doc duoc).
- Khong chay full suite (se chay mot lan o cuoi).
