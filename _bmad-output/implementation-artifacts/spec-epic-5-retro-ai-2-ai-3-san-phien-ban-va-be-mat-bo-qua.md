---
title: 'Retro Epic 5 · AI-2 + AI-3 — sàn phiên bản cho lượt thu hoạch, và bề mặt cho phần bị bỏ qua'
type: 'bugfix'
created: '2026-09-03'
status: 'done'
baseline_commit: '9cf891454fb59986f65b0fbcd53a4835ac232d49'
review_loop_iteration: 0
context:
  - '{project-root}/src-tauri/AGENTS.md'
  - '{project-root}/src/AGENTS.md'
  - '{project-root}/_bmad-output/implementation-artifacts/epic-5-retro-2026-09-03.md'
---

<frozen-after-approval reason="human-owned intent — do not modify unless human renegotiates">

## Intent

**Problem:** `harvest_work_text` chỉ chặn `project.db` **mới hơn** đích, không có sàn dưới — nên một tệp **cũ hơn** lọt cửa rồi gãy ở câu SQL kế tiếp bằng một chuỗi SQLite thô (`no such table: segment` · `no such column: is_omitted`). Đo trên thư viện thật của Ice 2026-09-03: **35/47 Tác phẩm (74,5%)** không có một dòng nào trong chỉ mục tìm kiếm, ranh giới sạch ở **v ≤ 7 trượt / v ≥ 11 xanh**. Con số bị bỏ qua **có** được đếm (`RebuildOutcome::text_skipped`) nhưng `grep -rn "text_skipped\|textSkipped" src/` = **0** — nó chỉ ra `eprintln!`. Người dùng gõ một câu, nhận *"Không tìm thấy"*, và không một pixel nào nói rằng ba phần tư thư viện chưa vào chỉ mục.

**Approach:** Cài **sàn phiên bản** cho lượt thu hoạch đúng khuôn `core/dict/layer.rs::MINIMUM_SCHEMA_VERSION` (Story 1.19), và đổi `TextHarvestSkipped::reason` từ `String` sang một **enum có tên** — *"Rỗng im lặng bị cấm; rỗng có lý do thì không"* (AD-44 ④). Rồi cho con số ấy một bề mặt: `SearchReport` chở **độ phủ cấp Tác phẩm** đếm tại lúc truy vấn, và `RescanReport` thôi vứt `text_skipped`.

## Boundaries & Constraints

**Always:**
- Sàn = **8**, và con số đó phải **suy được từ cột mà SQL gõ đích danh**, không từ số đo: câu `SELECT … is_omitted … FROM segment` cần `SEGMENT_OMITTED_DDL` — **bước 8** của `PROJECT_MIGRATIONS`. Mọi cột khác đã có từ v3 (`chapter`) · v5 (`segment.retired_at`) · v6 (`target_text`). Nâng phiên bản mà đường đọc gõ thêm cột mới ⇒ **nâng cả sàn**, ghi ngay tại hằng.
- Độ phủ trên `SearchReport` đếm từ **chính chỉ mục** (`library_work` so với `DISTINCT work_id FROM library_segment`), trong **cùng** `read` closure với `indexed_segments` — KHÔNG đọc `text_skipped`, thứ chỉ sống trong bộ nhớ của lượt `rebuild` gần nhất và biến mất sau khi khởi động lại.
- Dòng độ phủ là một bề mặt **ĐỘC LẬP** với `librarySearchStatusKey`: nó phải hiện **cả khi có kết quả**. Một lượt trả "3 kết quả" trong lúc 35 Tác phẩm vô hình là đúng thứ lỗi này sinh ra.
- `LibrarySearchStatus` giữ nguyên **tám** giá trị. Không thêm giá trị thứ chín.
- Chẩn đoán trong `src-tauri/src/**` viết **KHÔNG DẤU** (Kiểm A của `check:i18n`); văn bản hiển thị **chỉ** ở `vi.json`.
- Struct qua dây giữ `snake_case`, không `rename_all` (AD-21), và đóng băng ở `tests/ipc_contract.rs`.

**Ask First:**
- Nếu số đo cho thấy hai truy vấn `COUNT` mới làm một lượt `library.search` vượt **NFR3 (trần 500 ms; nền đo 2026-09-02 là 57,981 ms)** — dừng và trình số, đừng tự đổi hình dạng.
- Nếu phát hiện một cột thứ hai mà SQL thu hoạch gõ đích danh và nó ra đời **sau** bước 8 — dừng, vì sàn phải đổi và con số 8 trong spec này sai.

**Never:**
- KHÔNG `Store::open` trên `project.db` của một Tác phẩm khác trong lượt quét (§Always của Story 5.9: `Store::open` chạy bộ di trú, tức **ghi** vào một Tác phẩm mà lượt quét không sở hữu). ⇒ Spec này **không** đưa 35 Tác phẩm kia vào chỉ mục — nó làm chỗ hở **nói ra được**. Vế FR8 còn lại đi vào `deferred-work.md` kèm chủ.
- KHÔNG hạ sàn, KHÔNG thêm nhánh `CASE WHEN` dò cột để "cứu" tệp cũ — đó là đổi lỗi này lấy một lược đồ thứ hai cho cùng một số phiên bản.
- KHÔNG chạm AI-4 (làm tươi chỉ mục), AI-5 (e2e), AI-6, AI-7 — đã tách và có chủ.

## I/O & Edge-Case Matrix

| Scenario | Input / State | Expected Output / Behavior | Error Handling |
|---|---|---|---|
| Tệp đúng dải | `project.db` `user_version` ∈ [8, 18] | Thu hoạch bình thường, vào `library_segment` | N/A |
| Tệp CŨ hơn sàn | `user_version` = 7 (hoặc 3/5/6) | `text_skipped` một mục, `reason = SchemaTooOld { found, minimum: 8 }`; `library_work` vẫn UPSERT; `rebuild` KHÔNG trượt | Không SQL nào chạy sau phép kiểm |
| Tệp MỚI hơn đích | `user_version` = 19 | `SchemaTooNew { found, target }` — hành vi cũ giữ nguyên | như trên |
| `project.db` vắng mặt | tệp bị xoá | `ProjectDbMissing` | như trên |
| Tìm trên chỉ mục thủng | 47 hàng `library_work`, 12 `work_id` trong `library_segment`, truy vấn khớp 3 hàng | `works_total = 47`, `works_with_text = 12`; màn hình hiện **cả** "3 kết quả" **và** dòng "35/47 Tác phẩm chưa vào chỉ mục" | N/A |
| Tìm trên chỉ mục đủ | 47/47 có văn bản | `works_total == works_with_text` ⇒ dòng độ phủ **vắng mặt** | N/A |
| Chỉ mục rỗng hẳn | `library_segment` = 0 hàng, `library_work` = 0 | `index_empty` như cũ; dòng độ phủ vắng mặt (0/0 không phải một chỗ thủng) | N/A |
| Quét lại | người dùng bấm "Quét lại", 35 Tác phẩm trượt | `RescanReport.text_skipped` mang 35 mục (`work_id` + mã lý do), màn hình quét hiện số đó | N/A |

</frozen-after-approval>

## Code Map

- `src-tauri/src/core/library/indexer.rs:1286-1300` — `harvest_work_text`, phép kiểm **một chiều** cần sửa (`if found > target`). `:1263-1272` `HarvestedRow`. `:1307` đọc `chapter`; `:1328-1332` đọc `segment.is_omitted` — hai câu SQL quyết định con số sàn.
- `src-tauri/src/core/library/indexer.rs:1520-1568` — `RebuildOutcome`, trường `text_skipped`. `:1569-1575` `TextHarvestSkipped { work_id, reason: String }` — kiểu `reason` đổi ở đây. `:407-414` chỗ đẩy vào; `:1626-1636` dòng `eprintln!` tổng hợp trong `log_if_notable`.
- `src-tauri/src/core/dict/layer.rs:76-93` `MINIMUM_SCHEMA_VERSION` + `:104-120` `SkipReason::{SchemaTooNew, SchemaTooOld}` — **khuôn phải chép**, kèm lý lẽ đã viết sẵn ở `:60-73`.
- `src-tauri/src/core/store/schema.rs` — `PROJECT_MIGRATIONS` (đích 18); `SEGMENT_OMITTED_DDL` là **bước 8**, `SEGMENT_DDL` bước 5, `SEGMENT_TARGET_TEXT_DDL` bước 6, `CHAPTER_DDL` bước 3. Chỉ ĐỌC — không thêm bước di trú nào.
- `src-tauri/src/core/library/indexer.rs:791-812` — `Indexer::search`, chỗ đếm `indexed_segments` trong `read` closure; `:1072-1104` `core::SearchReport`.
- `src-tauri/src/commands/library.rs:95-117` `RescanReport` (trường `skipped: usize`, `text_skipped` đang bị **vứt** ở `:172-181`); `:464-482` IPC `SearchReport`; `:485-497` `From<CoreSearchReport>`.
- `src/modes/librarySearch.ts:61-105` refs + export; `:123-171` `librarySearchStatus` (hàm thuần, **tám** giá trị — giữ nguyên); `:246-253` chỗ chép `report.*` vào ref.
- `src/modes/LibraryMode.vue:523-545` khối `role="status"` tám nhánh — dòng độ phủ là một `<p>` **mới**, không sửa cây ba ngôi này.
- `src/i18n/vi.json:269-291` — cụm khoá `mode.library.search_*`.
- `src-tauri/tests/library_index_contract.rs:2489-2534` — `a_project_db_at_a_newer_schema_version_…`, bản đối xứng cho chiều CŨ dựng theo đúng khuôn này. `:2536+` ca `project.db` vắng mặt.
- `src-tauri/tests/ipc_contract.rs:395-430` (RescanReport) và `:541-585` (SearchReport) — hai danh sách khoá dây phải cập nhật.
- `tests/frontend/` — cây test vitest (KHÔNG đồng vị trí trong `src/`).

## Tasks & Acceptance

**Execution:**
- [x] `src-tauri/src/core/library/indexer.rs` — khai `MINIMUM_HARVEST_SCHEMA_VERSION: u32 = 8` kèm doc-comment nêu **cột nào** buộc con số đó (`segment.is_omitted`, bước 8) và luật "gõ thêm cột mới ⇒ nâng sàn"; thêm nhánh `found < MINIMUM` **trước** `ReadOnlyDb::open`.
- [x] `src-tauri/src/core/library/indexer.rs` — đổi `TextHarvestSkipped::reason` sang enum `HarvestSkipReason` (`ProjectDbMissing` · `VersionUnreadable{detail}` · `SchemaTooNew{found,target}` · `SchemaTooOld{found,minimum}` · `OpenFailed{detail}` · `ReadFailed{detail}`) với `code() -> &'static str` (mã máy đọc, ổn định) và `diagnostic() -> String` (KHÔNG DẤU, cho `eprintln!`). Sửa `harvest_work_text` trả `Result<_, HarvestSkipReason>` và `log_if_notable` dùng `diagnostic()`.
- [x] `src-tauri/src/core/library/indexer.rs` — `SearchReport` thêm `works_total` + `works_with_text`, đếm trong **cùng** `read` closure với `indexed_segments`; điền cả ở nhánh truy vấn rỗng.
- [x] `src-tauri/src/commands/library.rs` — IPC `SearchReport` chở hai trường mới (`From` chép thẳng); `RescanReport` thêm `text_skipped: Vec<TextSkippedEntry { work_id, reason }>` thay vì vứt.
- [x] `src-tauri/tests/library_index_contract.rs` — ca đối xứng với `:2490` cho chiều CŨ: hạ `user_version` xuống 7 **và** `ALTER TABLE segment DROP COLUMN is_omitted`, khẳng định `text_skipped` đúng một mục mang `SchemaTooOld`, `indexed == 1`, `rebuild` không trượt. Thêm ca độ phủ: 2 Tác phẩm, 1 trượt ⇒ `works_total == 2 && works_with_text == 1`.
- [x] `src-tauri/tests/ipc_contract.rs` — cập nhật hai danh sách khoá dây (`SearchReport`, `RescanReport`).
- [x] `src/modes/librarySearch.ts` — hai ref + export mới, và hàm **thuần** `librarySearchCoverageGap(worksTotal, worksWithText): { missing, total } | null` (trả `null` khi `worksTotal === 0` hoặc không thủng).
- [x] `src/modes/LibraryMode.vue` + `src/i18n/vi.json` — `<p role="status">` mới, `v-if` theo hàm trên, khoá `mode.library.search_coverage_gap` `{missing}`/`{total}` nói cả **cách sửa** ("mở Tác phẩm một lần để nâng cấp, rồi quét lại"); và hiện số `text_skipped` ở khối kết quả quét lại.
- [x] `tests/frontend/` — ca vitest cho `librarySearchCoverageGap` (thủng · không thủng · `worksTotal = 0`).
- [x] `_bmad-output/implementation-artifacts/deferred-work.md` — món nợ **có chủ** cho vế FR8 còn lại: 35 Tác phẩm ở v ≤ 7 vẫn nằm ngoài chỉ mục cho tới khi có một đường nâng cấp; chủ: Ice (quyết trước Epic 6).

**Acceptance Criteria:**
- Given một `project.db` ở `user_version` 7, when `Indexer::rebuild` chạy, then `text_skipped` mang đúng một mục `SchemaTooOld { found: 7, minimum: 8 }` và **không** một chuỗi lỗi SQLite thô nào xuất hiện.
- Given ca test trên, when GỠ nhánh sàn khỏi `harvest_work_text` rồi chạy lại **bộ test cũ**, then ca mới ĐỎ bằng một lỗi SQL thô — đối chứng phải là một phép **gỡ** thật, không một ca chèn thêm.
- Given `library_work` có 47 hàng và `library_segment` chỉ chứa 12 `work_id`, when người dùng tìm một câu khớp, then màn hình hiện **đồng thời** số kết quả và dòng "35/47 Tác phẩm chưa vào chỉ mục".
- Given `works_total == works_with_text`, when tìm, then dòng độ phủ không được render (một `v-if`, không một chuỗi rỗng).
- Given cả `npm run build` rồi `cargo test --locked`, when chạy `npm run check:i18n` và `npm run check:tokens`, then xanh — chẩn đoán Rust không dấu, mọi văn bản mới qua `t()`.

## Design Notes

Vì sao độ phủ đếm ở **lúc tìm** chứ không chở `text_skipped` xuống: `text_skipped` là kết quả của lượt `rebuild` **gần nhất trong tiến trình này**. Sau một lần khởi động lại mà chưa quét lại, nó rỗng — trong khi chỉ mục vẫn thủng y nguyên. Một bề mặt dựa vào nó sẽ **im lặng đúng vào ca thường gặp nhất**, tức tái lập chính lỗi đang sửa. Hai `COUNT` trên `library_work`/`library_segment` thì luôn nói đúng trạng thái trên đĩa.

Khuôn cần chép nguyên (`core/dict/layer.rs:60-73`) đã viết sẵn cả chẩn đoán lẫn thuốc:

```
"CẢNH BÁO — phép kiểm dưới đây KHÔNG chặn tệp CŨ, chỉ chặn tệp MỚI. […] rồi mới gãy ở
 DictLayer::attributions bằng `no such column: lang`. […] Đường bịt thật là một sàn phiên bản."
```

⚠️ **Chỗ chưa đo, ghi ra thay vì giấu:** thư viện của Ice **không có** Tác phẩm nào ở v8/v9/v10 (phân bố đo được: v3×21 · v5×1 · v6×10 · v7×3 · v11–18×12). Nên dải [8, 10] được biện minh bằng **cột SQL gõ đích danh**, không bằng một mẫu quan sát — và đó là lý do doc-comment của hằng phải nêu cột, không nêu số đo.

## Verification

**Commands:**
- `cargo test --locked --test library_index_contract` — xanh; hai ca mới có mặt.
- `cargo test --locked --test ipc_contract` — xanh; khoá dây mới đã đóng băng.
- `npm run test` — vitest xanh, gồm ca `librarySearchCoverageGap`.
- `npm run build` **trước** `cargo test --locked` — thiếu `dist/` thì `cargo test` gãy ở khâu biên dịch.
- `npm run check:i18n && npm run check:tokens && npm run check:commands` — xanh.
- `node scripts/check-debt-owner.mjs --report` — `mở KHÔNG có Chủ:` vẫn **0**.

**Manual checks (if no CLI):**
- Chạy `AURA_SCOPE_SELFTEST=1 ./src-tauri/target/debug/auratranslate` trên thư viện thật của Ice: dòng `library[index:startup]` phải nêu `schema_too_old` thay vì `no such table: segment`, và màn hình tìm kiếm phải hiện dòng "35/47".

## Suggested Review Order

**Sàn phiên bản — vì sao con số là 8**

- Điểm vào: hằng kèm dẫn xuất theo CỘT SQL, không theo mẫu quan sát.
  [`indexer.rs:1309`](../../src-tauri/src/core/library/indexer.rs#L1309)

- Nhánh chặn chiều DƯỚI, đặt TRƯỚC `ReadOnlyDb::open` nên 0 câu SQL chạy.
  [`indexer.rs:1440`](../../src-tauri/src/core/library/indexer.rs#L1440)

- Lý do thành GIÁ TRỊ có tên: sáu biến thể, `code()` cho dây, `diagnostic()` cho log.
  [`indexer.rs:1319`](../../src-tauri/src/core/library/indexer.rs#L1319)

**Độ phủ — đếm ở lúc TRUY VẤN, không đọc `text_skipped`**

- Hai `COUNT` trong cùng closure với `indexed_segments`; xem chú thích cho lý do.
  [`indexer.rs:809`](../../src-tauri/src/core/library/indexer.rs#L809)

- Hàm THUẦN quyết khi nào có chỗ thủng — `null` khi `worksTotal === 0`.
  [`librarySearch.ts:204`](../../src/modes/librarySearch.ts#L204)

- Bề mặt ĐỘC LẬP với khối tám nhánh: hiện cả khi lượt tìm CÓ kết quả.
  [`LibraryMode.vue:567`](../../src/modes/LibraryMode.vue#L567)

- Câu không khẳng định MỘT nguyên nhân — phán quyết Ice (B), 2026-09-03.
  [`vi.json:282`](../../src/i18n/vi.json#L282)

**Biên IPC — chỗ nối trước đây không ai canh**

- `text_skipped` thôi bị vứt; `reason` mang mã ổn định, không chuỗi SQLite thô.
  [`library.rs:108`](../../src-tauri/src/commands/library.rs#L108)

- Hai trường độ phủ chép qua `From` — hoán vị chúng từng đi qua sạch cả bộ test.
  [`library.rs:537`](../../src-tauri/src/commands/library.rs#L537)

**Test — đọc phần này như bằng chứng, không như phụ lục**

- Ca bịt đúng chỗ hở trên: gọi `rescan`/`search_library` THẬT, đỏ khi hoán vị.
  [`library_commands_contract.rs:774`](../../src-tauri/tests/library_commands_contract.rs#L774)

- Chiều CŨ, đối xứng với ca chiều MỚI; gỡ sàn ra thì đỏ bằng `no such column`.
  [`library_index_contract.rs:2584`](../../src-tauri/tests/library_index_contract.rs#L2584)

- Ca biên đúng tại sàn — thứ phân xử `<` với `<=`.
  [`library_index_contract.rs:2645`](../../src-tauri/tests/library_index_contract.rs#L2645)

- Độ phủ ở tầng lõi: 2 Tác phẩm, 1 trượt.
  [`library_index_contract.rs:2693`](../../src-tauri/tests/library_index_contract.rs#L2693)

**Ngoại vi**

- Guard kiểm MỌI phần tử, không chỉ `value[0]` — chú thích nói vì sao không chép người anh em yếu hơn.
  [`library.ts:137`](../../src/config/library.ts#L137)
