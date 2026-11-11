# Handoff: pha Rust -> pha Webview (dọn dữ liệu dịch nhanh)

Pha Rust xong. Plan có các hộp Rust đã tick.

## Lệnh IPC (đã đăng ký ở `lib.rs`, trong `commands/aitranslate.rs` `mod wire`)

Tên khoá trên dây là snake_case; tham số lệnh là camelCase ở `invoke()`.

- `quick_translation_stats()` -> `{ count: number, bytes: number, by_model: [{ model, count }] }`. `bytes` = `.db` + `-wal`. Lỗi `IpcError` khi không có kho.
- `quick_translation_count_older({ days })` -> `number`. `days` là `i64`, `< 1` -> `IpcError` code `quick_translation.invalid_days`, key `err.quick_translation.invalid_days` (đã thêm vào `vi.json`: "Số ngày phải là số nguyên từ 1 trở lên.").
- `quick_translation_purge({ scope })`, `scope` là `{ kind: 'all' }` hoặc `{ kind: 'older_than', days: number }` -> `{ deleted, bytes_before, bytes_after, compact_error: string | null }`.
  - `compact_error` khác null: hàng đã xoá nhưng chưa thu nhỏ được (hiện `Đã xoá N câu nhưng chưa thu nhỏ được tệp.` + `tError`). `bytes_before - bytes_after` có thể âm/0 khi lỗi; không trừ khi `compact_error` có.
  - Lệnh đã tự huỷ mọi lượt dịch nhanh (cả hai bề mặt) TRƯỚC khi xoá. Webview vẫn phải tự bỏ kết quả đang giữ (số đếm lần dọn) vì lượt bị huỷ không báo lại cho UI đã đóng.
  - `(async)` vì chặn trên writer; hai lệnh kia là sync.

## Cấu hình N

- `app_config` key `quick_translation_purge_days`, Rust parse về 30 khi trống/0/âm/lẻ/rác.
- Có trên dây bootstrap: `BootstrapConfig.quick_translation_purge_days: number` (đã qua parse, không bao giờ < 1). Frontend cần: thêm vào `src/config/bootstrap.ts` (kiểu `BootstrapConfig` :86, hằng khoá `quick_translation_purge_days`, ref + readonly + nạp ở :322 theo khuôn `glossary_scan_threshold`), ghi bằng `putConfig(SCOPE_APP_CONFIG, 'quick_translation_purge_days', String(n))`. Test Rust đã canh khoá round-trip (`scope_contract.rs`).

## Trạng thái đỏ có chủ ý

- `cargo test --test ipc_argument_contract` hiện ĐỎ: ba lệnh mới đã đăng ký nhưng chưa có `invoke()` nào trong `src/**/*.ts`. Sẽ xanh khi `src/config/quickTranslationData.ts` gọi đủ ba lệnh với đúng khoá `days` và `scope`. Chạy lại test này sau adapter.
- `scripts/check-commands.mjs` / `check:i18n` chưa chạy ở pha này.

## Đã kiểm ở pha Rust

- (g1) bỏ `compact` khỏi `QuickTranslationStore::purge` -> `purging_all_rows_shrinks_db_plus_wal_on_disk` đỏ (db+wal không nhỏ đi). (g2) bỏ `runs.cancel_all()` khỏi `quick_purge_wire` -> `a_run_blocked_mid_stream_writes_no_row_after_a_purge_cancels_it` đỏ (hàng bị ghi sau dọn). Cả hai đã khôi phục.
- Đã đổi: `config_invariants.rs` census `aitranslate.rs` 16/1/1, tổng cây 132/84, thêm ca chặn `quick_translation_purge`.

## Kết quả `cargo test --no-fail-fast` đủ (pha Rust)

- Đỏ không thuộc story: `segment_boundary::the_scanned_trees_are_large_enough_to_be_real` (`WEBVIEW_FLOOR` 193 so với 243 tệp `.ts`/`.vue` thật; chạm `src/` chỉ có một dòng `vi.json`, không thêm tệp). Cần nâng sàn lên 207, thuộc debt có sẵn, không phải pha này.
- Đỏ có chủ ý: `ipc_argument_contract` (xem trên).
- Mọi binary còn lại xanh.
- 🔵 2026-11-11 (điều phối): pha Webview thêm tệp `.ts` mới nên làm `WEBVIEW_FLOOR` trôi thêm; nâng sàn trong `segment_boundary.rs` theo chính chỉ dẫn của cổng (tiền lệ 15.3 với `FRONTEND_FLOOR`/`FILE_FLOOR`) và kiểm lại `naming_boundary`, `check-layout`, rồi chạy `cargo test --test segment_boundary --test naming_boundary`.

## Pha Webview -> review

Pha Webview xong; plan có hộp Webview đã tick.

- `ipc_argument_contract` nay xanh (adapter `src/config/quickTranslationData.ts` gọi đủ ba lệnh với `days` và `scope`).
- Số đếm lần dọn: `quickPurgeCount` + `markQuickStorePurged()` trong `quickTranslateRun.ts`. `glossaryQueueState.ts` bỏ mọi tham chiếu không phải `chapter` rồi tra lại hàng hiện tại (lượt tra kho đang bay bị bỏ qua bằng mốc `quickPurgeCount`); `importPreviewState.ts` tăng `quickEpoch`, xoá `blockTranslations`, tra lại khối đang thấy.
- State Cài đặt ở `src/settingsQuickDataState.ts`; N nhớ khi `change` (rời ô/Enter) và khi Dọn, qua `putConfig`; khởi tạo từ `bootstrapQuickTranslationPurgeDays`. `markQuickStorePurged()` gọi sau MỌI lệnh dọn trả về (kể cả lỗi), vì Rust đã huỷ lượt trước khi xoá.
- `compact_error` là chuỗi thô, không phải `IpcError`: dựng `IpcError` cục bộ với khoá mới `err.quick_translation.compact_failed` ({detail}) để vẫn qua `tError`.
- Bốn lệnh mới `settings.quick_data.{purge_older,purge_all,purge_all_confirm,purge_all_cancel}`; nút Dọn tất cả/Xoá N câu?/Thôi đều `dispatch`. 0 mục `POINTER_DEBT` mới.
- Lệch plan: ca webview của hai bề mặt đặt ở `glossaryContextReference.test.ts` và `importPreviewQuickBatch.test.ts` (nơi đã có khuôn mock kho), không ở `glossaryQueue.test.ts`/`importPreviewBlocks.test.ts`; ma trận Cài đặt ở tệp mới `settingsQuickData.test.ts`.
- Sàn cổng nâng theo chỉ dẫn của chính cổng: `CLICK_FLOOR` 312->335, `COMMAND_FLOOR` 309->329 (`scripts/check-commands.mjs`), `WEBVIEW_FLOOR` 193->209 (`tests/segment_boundary.rs`).
- Chưa làm: đối chứng gỡ seam cho phía webview (chưa chạy); không chạy full `cargo test`/vitest; lượt dùng thật là nợ `deferred-work.md:13883`. Chưa kiểm bằng mắt trong app (chỉ jsdom): nên xem tab Tab/focus của hàng và `ui-btn--danger`.
