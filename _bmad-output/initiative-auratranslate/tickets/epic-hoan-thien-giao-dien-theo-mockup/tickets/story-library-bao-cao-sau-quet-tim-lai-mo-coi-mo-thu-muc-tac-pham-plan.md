---
title: 'Library: báo cáo sau quét, tìm lại mồ côi, mở thư mục Tác phẩm'
type: 'feature'
ticket: '3'
created: '2026-10-13'
status: done
baseline_revision: '26975a928321069fa4daa57a3812b2511e547a74'
route: 'full'
route_source: 'auto'
risk: 'high'
review: 'quick'
review_source: 'pinned'
lenses_ran: ['quick']
review_loop_iteration: 1
context: []
---

<frozen-after-approval reason="human-owned intent — do not modify unless human renegotiates">

## Intent

**Problem:** Quét lại chỉ trả bốn con số và một danh sách mồ côi; app không nói tệp nào mới, tệp nào đã dời, mục mồ côi nào mới sinh, không có `Tìm lại…`, không có dựng lại toàn bộ, không có mốc "dựng lần cuối", và không mở được thư mục của một Tác phẩm (FR140, FR142; a:140, a:141, a:144; A10, A13, A15, A19).

**Approach:** `Indexer::rebuild` phân loại mới · di chuyển · mồ côi mới trong chính giao dịch nó đã đọc hàng cũ; `RescanReport` chở ba danh sách đó cùng mốc dựng. Ba lệnh Rust mới: tìm lại mục mồ côi (hộp thoại chọn thư mục gọi từ Rust, đối chiếu `work_id` với `meta.json`), dựng lại toàn bộ chỉ mục, mở thư mục Tác phẩm theo `workId`.

## Boundaries & Constraints

**Always:**
- Lệnh nhận `workId`, không nhận đường dẫn từ webview; không cấp quyền plugin nào cho JS (AD-48, AD-23; `capabilities/main.json` giữ ba quyền).
- Cờ mồ côi ở `global.db`; mọi đường ghi theo thứ tự `Indexer::rebuild` (ghi `global.db` trước, xoá khỏi chỉ mục sau).
- Mốc dựng chưa biết là `NULL`/`Option`, không `0`, không `Invalid Date`; báo cáo rỗng chỉ nói "không đổi" sau khi `rescanResultHasLoaded`.
- Chữ chỉ qua `vi.json`; khoá lỗi mới qua `message_keys!`.

**Never:**
- Không sửa nội dung `.atproj`, không viết `meta.json` từ các lệnh này.
- Không nút "Thêm vào Library" / "Bỏ qua" / "Cập nhật đường dẫn" của mockup (ngoài FR140); đường dẫn dời trong gốc đã tự cập nhật.
- Không thêm thư mục theo dõi thứ hai, không migration `PROJECT_MIGRATIONS`/`GLOBAL_MIGRATIONS`.

## I/O & Edge-Case Matrix

| Scenario | Input / State | Expected Output / Behavior | Error Handling |
|----------|--------------|---------------------------|----------------|
| Dời trong gốc + xoá | một `.atproj` đổi tên/dời nhưng vẫn nằm ngay dưới gốc, một bị xoá | báo cáo: `moved` 1 (từ → tới), `newly_orphaned` 1, `appeared` 0 | — |
| Tệp mới | `.atproj` có `work_id` chưa từng thấy | `appeared` 1 | — |
| Không đổi | quét lại ngay | ba danh sách rỗng, dòng "không có thay đổi" | — |
| Tìm lại đúng | mục mồ côi, chọn thư mục chứa `meta.json` cùng `work_id` trong gốc | hết mồ côi, hàng có trong chỉ mục, `library_orphan` mất hàng | — |
| Tìm lại sai | thư mục khác `work_id`, không `meta.json`, hoặc ngoài gốc | mục vẫn mồ côi, không ghi gì | `library.relocate_mismatch` / `library.relocate_outside_root` |
| Huỷ hộp thoại | người dùng huỷ | `Ok(None)`, không đổi gì | không lỗi |
| Dựng lại toàn bộ | `meta.json` của một Tác phẩm còn trên đĩa nhưng hỏng | hàng cũ rời chỉ mục, đếm vào `skipped`, không thành mồ côi | — |
| Mở thư mục | `workId` có trong chỉ mục / không có | đường dẫn đúng chuyển cho bộ mở / `library.work_not_found` | `library.open_folder_failed` khi hệ điều hành từ chối |

## Decisions (Ice, 2026-10-13)

- A19: mở thư mục bằng `tauri-plugin-opener` 2.5.4 (Apache-2.0 OR MIT; `open` 5.4.0 MIT; không Exhibit B). Trước khi sửa `Cargo.toml`: ghi hàng Stack của spine (NFR15) và thêm vào danh sách cho phép của `scripts/check-deps.mjs`. Đăng ký bằng `tauri_plugin_opener::Builder::new().open_js_links_on_click(false).build()`, không thêm quyền `opener:*`; gọi từ Rust qua `workId`.
- A10: mốc dựng là bảng một hàng `library_index_state(rebuilt_at, duration_ms)` ghi trong giao dịch `rebuild`; `LIBRARY_INDEX_MIGRATIONS` 8 → 9 theo luật viết lại tại chỗ. Không dùng mtime của `library-index.db`: WAL với `wal_autocheckpoint = 0` làm mtime tệp chính đi theo checkpoint, không theo lượt dựng.
- `Tìm lại…` từ chối thư mục ngoài thư mục gốc bằng `library.relocate_outside_root`.
- Giữ trọn kế hoạch dù vượt 1600 token.
- Quét giữ phẳng như Epic 5: "di chuyển" là `.atproj` đổi tên/dời nhưng vẫn nằm ngay dưới gốc, hoặc mục mồ côi quay lại; dời vào thư mục con thì thành mồ côi.

</frozen-after-approval>

## Code Map

- `src-tauri/src/core/library/indexer.rs` -- `rebuild` 222–505: giao dịch ở 324–471 UPSERT `kept` rồi đọc `existing` (hàng đã bị UPSERT ghi đè `atproj_path`). Phân loại cần đọc `(work_id, atproj_path)` TRƯỚC UPSERT và `orphan_store::list` trước giao dịch: mới = không có ở cả hai; di chuyển = có, đường dẫn khác, hoặc đang ở `library_orphan`; mồ côi mới = `to_orphan`. `RebuildOutcome` 2008 (`orphans: usize` là số mới chuyển). `mark_all_orphaned_for_missing_root` 521: cũng điền `newly_orphaned`. Thêm `rebuild_all` dùng chung thân với cờ xoá `library_work`/`library_segment` trước khi nạp (đọc `existing` trước khi xoá).
- `src-tauri/src/core/library/orphan_store.rs` -- `list/upsert_many/remove_many/forget` có sẵn; dùng lại cho `relocate`.
- `src-tauri/src/core/library/meta.rs` -- `WorkMeta::read(dir)` (181) để đối chiếu `work_id`; không gọi `write_atomic`.
- `src-tauri/src/core/store/schema.rs` -- `LIBRARY_WORK_DDL`, `LIBRARY_INDEX_MIGRATIONS` 2658.
- `src-tauri/src/commands/library.rs` (784 dòng) -- `RescanReport` 120, `OrphanEntry` 49, `rescan` 195, `forget_orphan` 263 (mẫu hàm thuần + `Option<&…>`), `apply_chosen_root` 321 (mẫu huỷ hộp thoại), `wire::library_choose_root` 701 (mẫu `blocking_pick_folder`, kiểm `Store`/`Indexer` trước hộp thoại). `wire` mới: `library_relocate_orphan`, `library_rebuild_index`, `library_open_work_folder`, đều `#[tauri::command(async)]`.
- `src-tauri/src/lib.rs` -- `generate_handler!` 1168–1174 thêm ba lệnh; 1097 là chỗ `.plugin(tauri_plugin_dialog::init())`.
- `src-tauri/src/core/i18n/` -- `message_keys!` (mẫu `LibraryNotOrphaned` 489); khoá mới: `library.relocate_mismatch`, `library.relocate_outside_root`, `library.work_not_found`, `library.open_folder_failed`; khoá `err.*` tương ứng ở `src/i18n/vi.json`.
- `src/config/library.ts` -- `RescanReport` 48, `isRescanReport` 141 (từ chối hình dạng cũ: phải thêm trường mới và sửa fixture), `CMD_*` 307, `callRescan` 334; thêm `relocateLibraryOrphan`, `rebuildLibraryIndex`, `openWorkFolder` theo luật adapter không ném.
- `src/modes/libraryRescan.ts` (277 dòng) -- `applyReport`, `rescanResultHasLoaded`, `resetLibraryRescan`; thêm state danh sách thay đổi, `rebuiltAt`, `relocateCurrentLibraryOrphan`, `rebuildLibraryIndexFolder`.
- `src/modes/LibraryMode.vue` -- khối Thư mục gốc/Quét lại/mồ côi 845–960 (`.status` 861, `.conflict-warning` 880, `.orphan-actions` 925–960), nút `library.open_work` 761 (đặt `Mở thư mục` cạnh nó). `src/commands/index.ts` 356–367, 1829–1870: mẫu đăng ký lệnh `library.*`; thêm `library.relocate_orphan`, `library.rebuild_index`, `library.open_work_folder` + `command.*` ở `vi.json` + dây `src/main.ts`.
- **Tests that move:** `src-tauri/tests/ipc_argument_contract.rs` (`REGISTERED_COMMAND_FLOOR`), `tests/frontend/libraryRescanReport.test.ts` (mới), `e2e/specs/story-12-3-library-rescan-report.e2e.mjs` (mới), cùng các tệp dưới đây.
- Tests hiện có phải đổi: `src-tauri/tests/library_commands_contract.rs` (fixture `write_atproj` 80, ca 168/907 đọc `RescanReport`), `library_index_contract.rs`, `ipc_contract.rs` (hình `RescanReport` đóng băng), `config_invariants.rs` 1509 (chữ ký vỏ `(async)`), `tests/frontend/libraryRescan.test.ts` (fixture `RESCAN_REPORT_*` thiếu trường mới sẽ bị `isRescanReport` từ chối), `e2e/specs/story-5-3-rescan.e2e.mjs` (đọc dòng `.status`).
- Cổng chạm: `check:i18n`, `check:commands`, `check:tokens`, `check:panel-refs`, `check:deps`, `config_invariants`.

## Tasks & Acceptance

**Execution (pha Rust):**
- [x] `src-tauri/src/core/library/indexer.rs` -- phân loại `appeared`/`moved`/`newly_orphaned` trong `rebuild`; `RebuildOutcome` mang chúng; thêm `rebuild_all` -- FR140
- [x] `src-tauri/src/core/store/schema.rs` + `indexer.rs` -- bảng `library_index_state`, ghi `rebuilt_at` (UTC ISO-8601) và `duration_ms` trong giao dịch `rebuild`, đọc ra `Option` -- A10
- [x] `src-tauri/src/commands/library.rs` -- `RescanReport` thêm `appeared`, `moved`, `newly_orphaned`, `rebuilt_at: Option<String>`; hàm thuần `relocate_orphan(indexer, global, work_id, picked: Option<&Path>, root)`, `rebuild_index`, `open_work_folder(indexer, work_id, opener: &dyn Fn(&Path) -> io::Result<()>)`; vỏ `wire` -- FR140, FR142
- [x] `src-tauri/src/lib.rs`, `src-tauri/src/core/i18n/`, `src/i18n/vi.json` -- đăng ký ba lệnh, bốn khoá lỗi
- [x] `src-tauri/src/commands/library.rs` + `Cargo.toml`, hàng Stack của spine, `scripts/check-deps.mjs`, `lib.rs` -- bộ mở thư mục qua `tauri-plugin-opener` (Stack + allow-list trước `Cargo.toml`)

**Execution (pha Webview):**
- [x] `src/config/library.ts`, `src/modes/libraryRescan.ts` -- trường báo cáo mới và ba adapter/hành động; huỷ hộp thoại giữ nguyên state
- [x] `src/commands/index.ts`, `src/main.ts`, `src/i18n/vi.json` -- ba lệnh `library.*`, nhãn
- [x] `src/modes/LibraryMode.vue` -- dải kết quả quét (mới · di chuyển · mồ côi, mỗi mục tên và đường dẫn), `Tìm lại…` cạnh `Gỡ khỏi chỉ mục`, `Dựng lại toàn bộ chỉ mục` có câu "mất dữ liệu không xảy ra", chân "Chỉ mục dựng lần cuối …" bỏ vế khi `rebuilt_at` là `null`, nút `Mở thư mục`; mỗi `@click` đúng một `dispatch`

**Execution (vòng sửa sau review):**
- [x] `src-tauri/src/commands/library.rs`, `src-tauri/src/lib.rs`, `src/config/library.ts`, `src/modes/libraryRescan.ts`, `src/modes/LibraryMode.vue` -- lệnh đọc `library_index_rebuilt_at` và lượt đọc ở `onActivated` (R11)
- [x] các bản sửa R1, R3, R4, R5, R9, R12 trong `## Review Triage Log`; sửa helper `changeNames` của `e2e/specs/story-12-3-library-rescan-report.e2e.mjs` (`TypeError: object is not iterable` ở dòng 56, hai ca đỏ khi chạy thật)

**Execution (pha Tests):**
- [x] `src-tauri/tests/library_commands_contract.rs`, `library_index_contract.rs`, `ipc_contract.rs`, `config_invariants.rs` -- ca theo I/O Matrix (dời + xoá đúng hai mục, mới, không đổi, tìm lại đúng/sai/ngoài gốc/huỷ, dựng lại với `meta.json` hỏng, mở thư mục nhận đúng đường dẫn qua bộ mở giả, `workId` lạ); sửa hình `RescanReport`
- [x] `tests/frontend/libraryRescan.test.ts` (+ `libraryRescanReport.test.ts` mới) -- fixture trường mới, danh sách rỗng chỉ "không đổi" sau lượt quét thật, huỷ hộp thoại, `rebuilt_at: null`
- [x] `e2e/specs/story-12-3-library-rescan-report.e2e.mjs` (mới), `story-5-3-rescan.e2e.mjs` -- dời một `.atproj` trong gốc và xoá một, quét lại, đúng hai mục; sửa các đọc `.status` cũ
- [x] Kiểm tay (không tự động hoá được): mở thư mục thật trên macOS (`Chủ: Epic 12`) và Windows (`B7`), hộp thoại `Tìm lại…` trên bản đóng gói -- ghi vào `deferred-work.md` lúc đóng, không bây giờ

**Acceptance Criteria:**
- Given thư mục gốc có hai Tác phẩm, when một `.atproj` đổi tên/dời nhưng vẫn nằm ngay dưới gốc và một bị xoá rồi bấm Quét lại, then báo cáo liệt kê đúng một di chuyển (đường dẫn cũ → mới) và một mồ côi mới, không mục thứ ba.
- Given một mục mồ côi, when chọn `Tìm lại…` và trỏ tới thư mục trong gốc chứa `meta.json` cùng `work_id`, then mục rời danh sách mồ côi và hiện trong lưới; trỏ sai thì còn nguyên và có lỗi nêu tên.
- Given Library đã quét, when bấm `Dựng lại toàn bộ chỉ mục`, then lưới và mồ côi khớp đĩa, mốc dựng cập nhật, và không mục mồ côi nào biến mất im lặng.
- Given một Tác phẩm đang chọn, when bấm `Mở thư mục`, then Rust nhận đúng `atproj_path` của `workId` đó; chưa tải chỉ mục thì nút không gọi lệnh.
- Given chưa quét lần nào, when mở Library, then không có câu "không có thay đổi" hay mốc dựng giả.

## Implementation Notes

- Phân loại đọc `library_work` cũ TRƯỚC UPSERT trong cùng giao dịch và `library_orphan` trước giao dịch; một mục rời danh sách mồ côi luôn vào `moved`, kể cả khi đường dẫn không đổi.
- Quét giữ phẳng (Ice): `Tìm lại…` chỉ nhận một `.atproj` con trực tiếp của gốc, nên lệnh thực chất là kiểm `work_id` rồi chạy một lượt `rebuild`; báo cáo của nó có thể kèm thay đổi khác.
- `rebuild_all` = thân `rebuild` + `DELETE FROM library_work` sau khi đã đọc hàng cũ; hàng `meta.json` hỏng rời chỉ mục vào `skipped`, thư mục biến mất vẫn thành mồ côi.
- App dựng lại chỉ mục mỗi lần khởi động, nên mốc "dựng lần cuối" thường là giờ mở app; lệnh đọc `library_index_rebuilt_at` gọi ở `onActivated` (R11).
- `tauri-plugin-opener` 2.5.4 ghim `=`; `open` bị cargo kéo lên 5.4.4, ghim lại 5.4.0 bằng `cargo update -p open --precise`. Hàng Stack của `is-wsl`/`is-docker` (chỉ Linux, MIT) ghi sau khi chúng đã vào `Cargo.lock`, trái thứ tự NFR15; giấy phép đã đọc trong nguồn tải.
- Đối chứng đỏ: xoá khối `moved.push` thì ca dời + xoá đỏ ở `report.moved.len()`; xoá lượt đọc mốc ở `onActivated` thì e2e ca đầu đỏ.
- `DISPATCH_FLOOR` 220 → 236 do `judgeFloor` đòi khi ba `dispatch` mới vào.
- Kiểm tay mở thư mục và hộp thoại `Tìm lại…` thật: hai mục nợ `Chủ: Epic 12` (macOS) và `Chủ: B7` (Windows).

## Plan Change Log

- 2026-10-13 (review, vòng 1) — R11: chân "Chỉ mục dựng lần cuối …" chỉ có mốc sau một cú bấm nên luôn ghi giờ của chính cú bấm. Thêm (ngoài khối frozen): lệnh đọc thuần `library_index_rebuilt_at` trả `Option<String>` từ `Indexer::index_state`, webview gọi ở `onActivated` cạnh `loadLibraryOrphans` và đặt `libraryRebuiltAt`; `null` thì bỏ vế. Ice chọn sửa trên mã hiện có thay cho revert-và-dựng-lại (lệch luật loopback của quy trình, Ice duyệt). Trạng thái xấu tránh: chân trang chỉ phản chiếu cú bấm vừa xong. KEEP: phân loại trong `rebuild_inner` đọc hàng cũ trước UPSERT, `rebuild_all` dùng chung thân, bộ mở giả trong `open_work_folder`, `open_js_links_on_click(false)` không quyền `opener:*`, mọi ca hợp đồng của ma trận. Ghi chú: app dựng lại chỉ mục mỗi lần khởi động (`lib.rs` lượt `startup`), nên mốc thường là giờ mở app.

## Review Triage Log

- R1 (quick) `libraryRescan.ts`: doc comment của `forgetCurrentLibraryOrphan` bị đẩy lên `relocateCurrentLibraryOrphan` — low, đúng như mô tả; patch.
- R2 (quick) `relocate_orphan`: `not_orphaned(work_id, work_id)` hiện mã hai lần — low; chỉ gặp khi mục rời danh sách mồ côi giữa lúc liệt kê và lúc bấm, sửa cần nhánh tên dự phòng; reject.
- R3 (quick) chú thích mới trái luật AGENTS.md (mã story `12-3` ở `lib.rs`, hai e2e, `libraryRescanReport.test.ts`; khối tiếng Việt nhiều dòng ở `schema.rs`, `indexer.rs`, `commands/library.rs`) — low; patch.
- R4 (quick) `indexer.rs`: rustfmt dàn lại dòng không liên quan (FTS `rebuild`, `text_skipped.push`, `fold_diacritics…`) — low, làm nhiễu diff giao dịch risk high; patch.
- R5 (quick) `indexer.rs`: câu UPSERT `library_index_state` chép hai nơi — low, hai bản có thể lệch; patch.
- R6 (quick) nhánh gốc vắng vẫn đóng mốc dựng — false: nhánh đó thật sự ghi lại chỉ mục (xoá `library_segment`, chuyển mọi hàng sang mồ côi), mốc nói đúng.
- R7 (quick) mốc ghi ở giao dịch 1 trước bước 2 (`global.db`); bước 2 trượt thì mất `moved`/`appeared` — low, cần `global.db` ghi trượt, sửa đòi tách giao dịch; reject.
- R8 (quick) `watch(libraryIndexRevision)` tải lưới hai lần khi kích hoạt — false: `onActivated` không gọi `applyReport` (chỉ `loadWorks` + đọc mồ côi), và `loadLibraryWorks` có `worksSequence` chặn đua.
- R9 (quick) nhánh mồ côi quay lại (`was_orphan`) không ca nào canh `moved` — medium, gỡ nhánh đó không làm đỏ ca nào; patch (ca tìm lại khẳng định `moved`).
- R10 (quick) `DISPATCH_FLOOR` 220 → 236 — false: `judgeFloor` đòi nâng khi số sống vượt, `check:commands` xanh.
- R11 (orchestrator) chân "Chỉ mục dựng lần cuối …" chỉ hiện sau một lượt Quét lại/Dựng lại/Tìm lại, nên luôn ghi giờ của chính cú bấm vừa xong; không đường đọc nào đưa `rebuilt_at` lúc vào Library — medium; bad_plan (kế hoạch không có đường đọc mốc khi kích hoạt).
- R12 (orchestrator) `Indexer::index_state` đọc `duration_ms` bằng `u64::try_from(..).unwrap_or(0)` — low, trái luật "giá trị có thể không biết là Option, không 0"; patch.

## Design Notes

- Báo cáo là "đổi kể từ lượt dựng trước": `lib.rs:1779` dựng lại mỗi lần khởi động và `reindex_library` chạy sau mỗi lượt tạo, nên tệp dời trước khi mở app đã được hấp thụ (tự cập nhật đường dẫn) và không còn trong báo cáo Quét lại; mục mồ côi thì vẫn ở `library_orphan`. Không lưu báo cáo qua khởi động (ngoài FR140).
- `Indexer::rebuild` đã nạp lại `library_segment` và FTS mỗi lượt; `rebuild_all` khác ở chỗ xoá `library_work` của mọi hàng rồi nạp từ đĩa, nên hàng "hỏng nhưng còn" (bị giữ cũ ở quét thường) rời chỉ mục và đếm vào `skipped`.
- Rủi ro nâng từ medium lên high: đổi hình `RescanReport` đã đóng băng (`ipc_contract`, `isRescanReport`), ba lệnh IPC mới, và sửa giao dịch hai kho trong `Indexer::rebuild` mà ngoại lệ thứ tự ghi không có test nào canh ngoài `library_index_contract`.
- Phần mockup `Thêm vào Library`/`Bỏ qua`/`Cập nhật đường dẫn` và "2 thư mục theo dõi khác" là chức năng ngoài FR140; `data-integrity.html` §1 là nguồn bố cục.

## Verification

**Commands:**
- `cargo test --test library_commands_contract --test library_index_contract --test ipc_contract --test config_invariants` -- expected: xanh; đối chứng: gỡ phân loại `moved` khỏi `rebuild` thì ca dời + xoá đỏ.
- `npm run test:story 12-3 -- --list` rồi `npm run test:story 12-3` -- expected: chọn và xanh.
- `npm run check:i18n && npm run check:commands && npm run check:tokens && npm run check:panel-refs` && npm run check:deps -- expected: xanh.
- `npm run build`, rồi `story-12-3-library-rescan-report` và `story-5-3-rescan` e2e -- expected: xanh.

**Manual checks (if no CLI):**
- Mở thư mục Tác phẩm thật trên macOS và Windows; `Tìm lại…` qua hộp thoại thật.
