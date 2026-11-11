---
title: 'Dọn dữ liệu dịch nhanh ở Cài đặt → Dữ liệu'
type: 'feature'
ticket: '4'
created: '2026-11-11'
status: 'done'
baseline_revision: '45e3dd48c6fb7a72d37f0df4633ae01f640d5695'
route: 'full'
route_source: 'auto'
risk: 'medium'
review: 'quick'
review_source: 'pinned'
lenses_ran: ['quick']
review_loop_iteration: 0
context: ['{project-root}/_bmad-output/initiative-auratranslate/architecture-auratranslate/architecture-auratranslate.md §AD-64', '{project-root}/_bmad-output/initiative-auratranslate/ux-auratranslate/EXPERIENCE.md §Cài đặt › Dữ liệu']
---

<frozen-after-approval reason="human-owned intent — do not modify unless human renegotiates">

## Intent

**Problem:** `quick-translation.db` chỉ lớn dần: người dùng không thấy kho đang giữ bao nhiêu câu, chiếm bao nhiêu chỗ, và không có cách nào xoá. Story 15.5 sắp cho app tự sinh dữ liệu vào kho, nên đường dọn phải có trước.

**Approach:** `Store` có thêm một đường không transaction chạy `VACUUM` rồi `wal_checkpoint(TRUNCATE)` trên writer. Kho có thêm đếm, thống kê và dọn. Lệnh dọn huỷ mọi lượt dịch nhanh đang chạy, xoá hàng, thu nhỏ tệp rồi trả số câu đã xoá và số byte lấy lại. Hàng `Dịch nhanh` trong `SettingsDataSection.vue` vẽ theo EXPERIENCE §Cài đặt › Dữ liệu. Dọn xong, bề mặt nào đang mở thì về `Chưa có`.

## Boundaries & Constraints

**Always:** AD-64 mục 5 và mục 10, cùng đối chứng (g). Thứ tự là hợp đồng: tăng `generation` của `QuickTranslateGeneration` (huỷ mọi `run_id`) TRƯỚC khi xếp job xoá, để mọi `save` xếp sau bị `still_current` chặn, còn `save` xếp trước thì bị job xoá dọn đi. Mốc N tính theo `created_at` và so trong SQL với `strftime('%Y-%m-%dT%H:%M:%fZ','now', …)`, đúng định dạng `save` đang ghi. N là số nguyên ≥ 1, mặc định 30, lưu ở `app_config` của `global.db` (không dùng `localStorage`, AD-1). Dung lượng là `.db` cộng `-wal`, đo bằng metadata tệp. Số lượng và số byte đi trên dây dạng số; định dạng `12.480` và `8,4 MB` làm ở frontend. Kho không mở được thì hàng hiện `tError` và cả hai nút tắt.

**Never:** Xoá hay đổi tên tệp (Windows). Tự dọn hoặc trần dung lượng. Đổi ngữ nghĩa `Store::write`, `Writer::enqueue` hay `write_ticket`. Mở đường cho kết nối hay transaction ra ngoài `core/store`. Đụng `AiTranslateGeneration`. Hộp thoại xác nhận cho `Dọn tất cả`. Thêm phụ thuộc.

## I/O & Edge-Case Matrix

| Scenario | Input / State | Expected Output / Behavior | Error Handling |
|---|---|---|---|
| Thống kê | 3 hàng, 2 mô hình | `3 câu · <x> KB`, dưới đó mỗi mô hình một dòng số câu | — |
| Kho rỗng | 0 hàng | `Chưa có câu nào.`, hai nút tắt | — |
| Đếm trước | N=30, 2 hàng cũ hơn 30 ngày | `Sẽ xoá 2 câu.` đếm lại khi đổi số | N trống, 0, âm, lẻ ⇒ ô báo lỗi, `Dọn` tắt |
| Dọn quá N | Bấm `Dọn` | Xoá 2 hàng, thu nhỏ tệp, `Đã xoá 2 câu, lấy lại <y>. Lần sau dịch lại sẽ gọi AI.`, số liệu cập nhật | — |
| Dọn tất cả | Bấm `Dọn tất cả` | Nút thành `Xoá 3 câu?` cạnh `Thôi`; xác nhận thì xoá hết; `Thôi` thì về nút cũ | — |
| Đang dọn | Lệnh chưa trả | `Đang dọn…`, hai nút tắt | — |
| Lượt đang chạy | Bảng chờ hoặc tầng khối đang dịch khi bấm Dọn | Lượt bị huỷ, không ghi hàng nào sau khi dọn; bề mặt về `Chưa có` | — |
| Không thu nhỏ được | `VACUUM` hoặc checkpoint lỗi | `Đã xoá 2 câu nhưng chưa thu nhỏ được tệp.` kèm `tError`; hàng đã xoá vẫn xoá | Không thử lại |
| Kho hỏng | Không có `QuickTranslationStore` | `tError`, hai nút tắt | Lệnh trả lỗi, không panic |

</frozen-after-approval>

## Code Map

- `src-tauri/src/core/store/writer.rs` -- `Task` (:41, trả `bool` committed để `touch_write`), `enqueue` (:259, bọc job trong `conn.transaction()` :283), kiểm gọi lồng `ON_WRITER_THREAD`. Thêm một job riêng không transaction, xếp vào CÙNG hàng đợi, không mở `Connection` ra ngoài; các bất biến không-panic ở đầu module vẫn áp dụng.
- `src-tauri/src/core/store/mod.rs` -- `Store::write` (:790), `path()` (:852). Thêm một hàm hẹp kiểu `Store::compact() -> Result<(), StoreError>` (`VACUUM` rồi `PRAGMA wal_checkpoint(TRUNCATE)`; `busy != 0` là lỗi), không nhận closure. `tests/store_boundary.rs` cấm `rusqlite` ngoài `core/store`.
- `src-tauri/src/core/quick_translation/mod.rs` -- `QuickTranslationStore` (:77), `save` (:142, `created_at` dạng `%Y-%m-%dT%H:%M:%fZ`), `count` (:176). Thêm: thống kê (số câu, số câu theo `model`, số byte), đếm cũ hơn N ngày, xoá tất cả hoặc cũ hơn N ngày, rồi `compact`. Kết quả dọn mang `deleted` và byte trước/sau hoặc lỗi thu nhỏ.
- `src-tauri/src/commands/aitranslate.rs` -- `QuickTranslateGeneration` (:784, `begin` :818, `cancel(run_id)` :842, `is_current` :858). Thêm cách huỷ không cần `run_id` (tăng `generation`, xoá `active_run`). Lệnh mới đặt trong `mod wire` cạnh `quick_translation_lookup` (:2758), lấy kho bằng `app.try_state::<QuickTranslationStore>()`. Ghim số lệnh của tệp: `tests/config_invariants.rs:2320` (hiện 14).
- `src-tauri/src/lib.rs` -- đăng ký lệnh ở :1450-1455. Ghim tên lệnh: `tests/ipc_contract.rs:3078`; tham số camelCase: `tests/ipc_argument_contract.rs`.
- `src-tauri/src/core/scope/store.rs` -- khuôn khoá `app_config` dạng số: `KEY_GLOSSARY_SCAN_THRESHOLD` (:130), `parse_glossary_scan_threshold` (:233). Đi tiếp qua `commands/config.rs:103,165` và `src/config/bootstrap.ts:86,213,322`. Ghi bằng `putConfig(SCOPE_APP_CONFIG, …)` như `src/glossarySettingsState.ts:129`.
- `src-tauri/tests/quick_translation_contract.rs` -- khuôn `Fake` và các ca huỷ (:390-431, :521-592). Test được dùng `rusqlite` trực tiếp để cài hàng có `created_at` cũ (tiền lệ `tests/export_contract.rs`).
- `src/SettingsDataSection.vue` (110 dòng) -- thêm hàng; khuôn lớp `sd-row`/`sd-label`/`sd-note`/`sd-alert`. Test có sẵn: `tests/frontend/settingsReadingDataBodies.test.ts:72`.
- `src/quickTranslateRun.ts` -- module dùng chung của hai bề mặt. Thêm một số đếm lần dọn để `glossaryQueueState.ts` (`references` :124, `loadContextReference` :163) và `importPreviewState.ts` (`blockTranslations` :860) bỏ kết quả lấy từ kho hay từ lượt chạy rồi tra lại. Câu lấy từ Tác phẩm giữ nguyên. Không import chéo hai state vào module dọn (bẫy TDZ vòng import).
- `src/commands/index.ts` -- khuôn handler Cài đặt `runQuickAiConfigTest` (:1153); `scripts/check-commands.mjs` `HANDLER_TABLE`. `src/i18n/vi.json` -- `settings.data.*` (:2383-2386). Định dạng số: `toLocaleString('vi-VN')` như `src/modes/libraryRescan.ts:106`.
- Sổ nợ đã có: `deferred-work.md:13883` (lượt dùng thật của Ice, gồm `Dọn` thu nhỏ tệp, `Chủ: Epic 15`) và `:13884` (Windows `VACUUM`). Không thêm mục mới cho hai việc này.

## Tasks & Acceptance

Ba pha thực thi theo AGENTS.md, mỗi pha một agent mới; giao lại qua `story-don-du-lieu-dich-nhanh-o-cai-at-du-lieu-handoff.md` cạnh plan.

**Execution (pha Rust):**
- [x] `src-tauri/src/core/store/{writer.rs,mod.rs}` -- job không transaction và `Store::compact` -- mục 10: `Writer` bọc mọi job trong transaction, còn `VACUUM` không chạy được trong transaction.
- [x] `src-tauri/src/core/quick_translation/mod.rs` -- thống kê, đếm cũ hơn N ngày, xoá tất cả hoặc cũ hơn N ngày, rồi `compact`; `compact` lỗi thì hàng đã xoá vẫn xoá, kết quả mang lỗi -- mục 10.
- [x] `src-tauri/src/commands/aitranslate.rs`, `src-tauri/src/lib.rs` -- huỷ tất cả trên `QuickTranslateGeneration`; lệnh `quick_translation_stats`, `quick_translation_count_older(days)`, `quick_translation_purge(scope)` với `scope` là `all` hoặc `older_than{days}`. Lệnh dọn huỷ tất cả TRƯỚC khi xếp job xoá; `days < 1` trả lỗi -- mục 8, 10.
- [x] `src-tauri/src/core/scope/store.rs`, `src-tauri/src/commands/config.rs` -- khoá `quick_translation_purge_days`, parse về 30 khi trống hoặc không hợp lệ, đi qua bootstrap -- nhớ N.
- [x] `src-tauri/tests/quick_translation_contract.rs`, `store_contract.rs`, `config_invariants.rs`, `ipc_contract.rs`, `ipc_argument_contract.rs` -- ma trận I/O phần Rust và đối chứng (g): (g1) chèn nhiều hàng, dọn tất cả, kích thước `.db`+`-wal` giảm thật; (g2) một lượt bị chặn giữa luồng bằng `Fake`, dọn chạy, lượt đi tiếp, kho 0 hàng. Kèm các ca: `compact` chạy không transaction; dọn quá N chỉ xoá hàng cũ.

**Execution (pha Webview):**
- [x] `src/config/quickTranslationData.ts` (mới) -- adapter IPC không ném cho ba lệnh.
- [x] `src/quickTranslateRun.ts`, `src/glossaryQueueState.ts`, `src/importPreviewState.ts` -- số đếm lần dọn; hai bề mặt bỏ kết quả lấy từ kho hay từ lượt chạy, rồi tra lại.
- [x] `src/SettingsDataSection.vue`, `src/settingsQuickDataState.ts` (mới), `src/config/bootstrap.ts`, `src/commands/index.ts`, `src/main.ts`, `src/i18n/vi.json` -- hàng `Dịch nhanh` theo ma trận I/O. Ô N là `ui-field--short`. `Dọn tất cả` là nút Nguy hiểm, xác nhận tại chỗ. Câu kết quả dùng `role="status"`. Mọi nút vào được bằng `Tab`.
- [x] `tests/frontend/settingsReadingDataBodies.test.ts` (hoặc tệp mới cạnh nó), `glossaryQueue.test.ts`, `importPreviewBlocks.test.ts` -- từng hàng của ma trận phần webview; dọn xong thì hai bề mặt về `Chưa có` còn câu lấy từ Tác phẩm vẫn còn.

**Acceptance Criteria:**
- Given kho có hàng và một lượt dịch nhanh đang chạy ở bất kỳ bề mặt nào, when người dùng dọn, then lượt đó không ghi được hàng nào và kho đếm ra đúng số hàng còn lại.
- Given dọn tất cả trên một kho đã có nhiều hàng, when lệnh trả, then tổng kích thước `.db` và `-wal` trên đĩa nhỏ hơn trước, và câu kết quả nêu số byte lấy lại.
- Given mỗi đối chứng (g1) và (g2), when gỡ seam tương ứng trong mã sản phẩm (bỏ `compact` khỏi đường dọn; bỏ bước huỷ tất cả trước khi xoá), then đúng test canh của nó đỏ, và đỏ đúng lý do.
- Given người dùng nhập N = 7 rồi đóng và mở lại app, when mở Cài đặt → Dữ liệu, then ô vẫn là 7.

## Implementation Notes

- `Store::compact` là đường không-transaction duy nhất của `Writer`: không nhận closure, xếp cùng hàng đợi, checkpoint `busy != 0` trả lỗi. Một reader giữ snapshot đủ làm checkpoint bị chặn mà `VACUUM` vẫn qua.
- Lệnh dọn kiểm kho có mặt trước rồi mới `cancel_all`; kho thiếu thì không huỷ lượt nào.
- `compact_error` đi trên dây dạng chuỗi; webview dựng `IpcError` với khoá `err.quick_translation.compact_failed` (chỉ ở `vi.json`) để đi qua `tError`.
- Ca webview nằm ở `settingsQuickData.test.ts` (mới), `glossaryContextReference.test.ts` và `importPreviewQuickBatch.test.ts`, không ở ba tệp mà Tasks/Verification kể, vì mock kho dịch nhanh đã sẵn ở đó.
- Ba sàn cổng nâng theo công thức của chính cổng: `COMMAND_FLOOR` 329, `CLICK_FLOOR` 335 (`check-commands.mjs`), `WEBVIEW_FLOOR` 209 (`segment_boundary.rs`, đã đỏ sẵn ở baseline: 193 so với 243 tệp).
- Đối chứng gỡ thật, mỗi ca đỏ đúng lý do: bỏ `compact` khỏi `purge` (g1); bỏ `cancel_all` (g2); bỏ nhánh `busy != 0` (ca reader chặn checkpoint); bỏ hai `watch(quickPurgeCount)` (5 ca bề mặt).

## Plan Change Log

## Review Triage Log

Lượt 1 (quick): 0 high · 2 medium · 6 low · 6 false.
- medium · patch — `commitQuickDataDays` bỏ lỗi `putConfig`, N không được nhớ mà không ai biết.
- medium · patch — `v-if`/`v-else` của `Dọn tất cả`/`Xoá N câu?`/`Thôi` tháo nút đang có tiêu điểm, tiêu điểm rơi về `body`.
- low · patch — câu báo N không hợp lệ thiếu `role="alert"`.
- low · patch — `loadQuickDataRow` giữ `result`/`purgeError` của lần mở Cài đặt trước.
- low · patch — doc comment tiếng Việt, năm dòng, kể lại mã ở `Store::compact`/`Writer::compact` (AGENTS.md §Code comments).
- low · patch — ca `a_shrink_blocked…` có thông điệp assert nói ngược; ca `the_store_purge_reports_scope_all…` trùng ca (g1), xoá.
- low · reject — `file_bytes` `.unwrap_or(0)`: `-wal` vắng thật sự là 0 byte; `.db` của kho đang mở không đọc được metadata không gặp khi dùng thường, còn sửa phải đổi wire sang `Option`.
- low · reject — `{detail}` của `compact_failed` là chữ SQLite tiếng Anh; chỉ lộ khi checkpoint bị chặn, sửa phải thêm khoá lỗi có cấu trúc trên dây.
- false — `runPurge` đánh dấu đã dọn cả khi lỗi: kho thiếu thì Rust trả lỗi trước `cancel_all`; lỗi sau đó thì lượt đã bị huỷ thật nên bề mặt về `Chưa có` là đúng; nhánh `value`/`error` cùng `null` chỉ có ngoài Tauri.
- false — `eslint-disable` ở `bootstrap.ts`: cùng khuôn, cùng lý do với tám dòng sẵn có trong tệp cho trường IPC mà bản Rust cũ thiếu; "không gửi số dưới 1" và "bản cũ thiếu trường" không mâu thuẫn.
- false — `ipc_argument_contract.rs` không cần sửa: nó tự đọc lệnh từ `lib.rs` và đối với mọi `invoke()`; xanh sau khi có adapter.
- false — tệp test khác plan làm `settingsReadingDataBodies.test.ts` vỡ: chạy lại 4/4 xanh, không `console.error`.
- false — sàn cổng nâng vì thay đổi lạ trong cây: cây sạch ở baseline; số mới theo công thức `ceil(0.85 × live)` của cổng.
- false — watcher dọn gọi tra kho khi lớp phủ đóng: `loadContextReference` và `lookupVisibleBlocks` đều thoát sớm khi `!overlayOpen.value`.

## Design Notes

`compact` là một hàm hẹp trên `Store`, không phải một `write_raw(closure)`. Một closure nhận `&Connection` sẽ mở cửa cho mọi job bỏ transaction, mà AD-11 dựa trên chuyện mỗi job là một giao dịch. Chạy trên writer, không trên kết nối riêng, vì `VACUUM` cần kết nối ghi duy nhất và không được có transaction đọc nào đang mở. Checkpoint `busy` coi là chưa thu nhỏ được, không coi là xong.

Đối chứng (g2) cần một lượt đang chạy thật: `Fake` dừng ở một rào giữa luồng, test gọi đường dọn, rồi thả rào. Gỡ seam là bỏ bước huỷ tất cả. Khi đó `still_current` vẫn đúng và hàng được ghi sau khi dọn.

## Verification

**Commands:**
- `npm run build && cd src-tauri && cargo test --test quick_translation_contract --test store_contract --test store_boundary --test config_invariants --test ipc_contract --test ipc_argument_contract` -- expected: xanh, có ca (g1)(g2).
- `npx vitest run tests/frontend/settingsReadingDataBodies.test.ts tests/frontend/glossaryQueue.test.ts tests/frontend/importPreviewBlocks.test.ts` -- expected: xanh, mã thoát 0.
- `npm run check:commands && npm run check:i18n && npm run check:tokens` -- expected: xanh, `POINTER_DEBT` không tăng.
- Đăng ký lệnh trong `lib.rs` chạm wiring dùng chung ⇒ chạy đủ `cargo test` một lần trước khi báo xong (AGENTS.md §Tests).

**Manual checks:**
- Lượt dùng thật thu nhỏ tệp thuộc mục nợ `deferred-work.md:13883` của Epic 15; Windows thuộc `:13884`.
