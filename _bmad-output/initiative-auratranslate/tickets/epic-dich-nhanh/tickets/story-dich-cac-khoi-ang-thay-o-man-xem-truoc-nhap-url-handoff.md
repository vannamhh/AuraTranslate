# Handoff pha Rust -> pha Webview

Rust xong, test xanh. Chưa commit.

## Số đo hai hằng (quần thể: bảy trang tin epochtimes tiếng Trung `6-1-ban-do/fixtures/html/a01-a07.html`, qua `webimport::extractor::extract`, Debug build)
- Khối chữ mỗi trang 172-464 (đa số là menu/chân trang, trung vị 4-19 ký tự); tối đa một khối 246 ký tự.
- 40 khối chữ liền nhau: tối đa 3 827 ký tự (a05), 3 214 (a03), các trang còn lại dưới 1 100.
- Chốt `QUICK_BATCH_MAX_BLOCKS = 40`, `QUICK_BATCH_MAX_CHARS = 4000` (ở `core/quick_translation/mod.rs`). Số khối "đang thấy" thật phụ thuộc cửa sổ, chưa đo ở app thật.

## Hợp đồng IPC (camelCase ở webview)
- `quick_translate_blocks({ texts: string[], runId: number, channel })` -> outcome tag `state`:
  `not_configured` | `store_unavailable` | `busy` | `cancelled` | `done { model: string|null, usage, missing: [{index, reason}], deferred: number }`. Lỗi mạng/keychain ra `Err(IpcError)`.
- Kênh JSON tag `kind`, `index` là vị trí trong `texts` mà webview gửi (chữ trùng nhận sự kiện ở mọi vị trí):
  `cached {index,text,model}` | `started {indices}` | `token {index,text}` | `received {index,text,model,saved}`.
  Không có sự kiện kết thúc; `missing`/`usage` nằm ở giá trị trả về. `reason` là `missing|unclosed|no_finish|duplicate_index|empty|interrupted`.
- `quick_translation_lookup_many({ texts })` -> `QuickLookupWire[]` cùng thứ tự (`hit {text,model}` | `miss` | `store_unavailable`).
- Huỷ dùng lại `ai_context_reference_cancel({ runId })`. Bề mặt cố định ở Rust: Bảng chờ = `ai_context_reference_translate`, màn xem trước = `quick_translate_blocks`.
- `ai_context_reference_translate` nay có thể trả `{ state: 'busy' }` (thêm vào `QuickTranslateOutcomeWire`).

## Hành vi cần biết
- Rust tự tra kho trước khi gửi và phát `cached` cho khối đã có; webview vẫn nên `lookup_many` khi khối hiện để không đợi bấm nút.
- Một lượt nhận tối đa 40 khối chưa có / 4000 ký tự theo thứ tự trang; phần thừa là `deferred` (đếm vị trí). Gửi lại danh sách đầy đủ lần sau, kho tự loại khối đã có.
- Khối thiếu/bị cắt KHÔNG lưu và Rust không tự gửi lại. Khối trùng chỉ số đóng hai lần: lưu lần đóng đầu và KHÔNG có trong `missing` (Rust bỏ khối đã lưu khỏi `missing`; ca `a_block_closed_twice_keeps_its_first_text_and_is_not_reported_missing`). Một `index` không bao giờ vừa `received` vừa `missing`.
- `received.saved=false` khi kho từ chối ghi: vẫn hiện chữ.
- Khe chạy: sau khi lượt kết thúc tự nhiên, `cancel(runId)` không còn tác dụng (không đổi hành vi hiển thị, chỉ test cũ của Bảng chờ phải biết). Hai bề mặt khác nhau: bề mặt sau nhận `busy` tới khi bề mặt trước xong hoặc bị huỷ.

## Còn đỏ có chủ đích
- `cargo test --test ipc_argument_contract` đỏ tới khi pha Webview thêm `invoke('quick_translate_blocks', { texts, runId, channel })` và `invoke('quick_translation_lookup_many', { texts })` ở `src/**/*.ts` (adapter `src/config/quickBatch.ts`). Đây là ghim "lệnh đăng ký phải có invoke".
- Pha cuối chạy toàn bộ một lần (đổi đăng ký lệnh `lib.rs`).

## Đối chứng đã làm (gỡ seam thật, đỏ đúng chỗ, đã khôi phục)
Gỡ vòng lưu-khi-đóng -> 2 ca đỏ (huỷ giữa lô, lỗi mạng); gỡ cổng bề mặt trong `begin` -> ca khe + ca wire `busy` đỏ; bỏ tra kho trước khi gửi -> 5 ca đỏ (kể cả hai bề mặt một hàng); `cancel` không kiểm `run_id` -> ca "huỷ sai run_id" đỏ; gỡ gom theo khoá -> ca trùng chữ đỏ.

## Tệp đổi
`src-tauri/src/core/ai/{batch,rag}.rs`, `core/quick_translation/mod.rs`, `commands/{aitranslate,aiprompt}.rs`, `lib.rs`; test mới `tests/quick_batch_contract.rs`; sửa `quick_translation_contract`, `glossary_context_reference_contract`, `ai_boundary`, `ipc_contract`, `config_invariants`.

# Handoff pha Webview -> pha Tests that move

Webview xong; vitest xanh. Chưa commit.

## Đã làm
- `src/quickTranslateRun.ts` (mới): bộ cấp `run_id` + lượt đang chạy `{surface, runId}`; `beginQuickRun/endQuickRun/quickRunBusyFor`. Bảng chờ dùng nó (`kind: 'busy'`, nút tắt + `Đang dịch ở màn khác.`).
- `src/config/quickBatch.ts` (mới): `translateQuickBlocks`, `quickTranslationLookupMany`; huỷ dùng lại `cancelContextReference`.
- `src/importPreviewState.ts`: câu dịch gắn theo chữ khối (`quickBlockKey` = NFC+trim), `setImportPreviewVisibleBlocks` (overlay báo, tra kho khi hiện), `translateImportPreviewVisibleBlocks`, `translateImportPreviewFocusedBlock`, `cancelImportPreviewQuickBatch`; huỷ khi đổi Chương/đóng màn.
- `ImportPreviewOverlay.vue`: dòng đầu tầng `.ip-quick`, câu dịch `[data-ip-block-translation]`, nút `[data-ip-quick-translate]` / `[data-ip-quick-cancel]` / `[data-ip-block-translation-retry]`; IntersectionObserver trên `li.ip-block[data-ip-block-index]`; bù cuộn khối con trỏ (đo trước/sau DOM, chỉ khi khối nằm trong khung nhìn); `Esc` đầu huỷ lượt.
- Ba command mới `import.preview.quick_translate_{visible,block,cancel}` (keyless, nút bấm).
- Đã đóng mục nợ `lastRunId` trong `deferred-work.md`.

## Cho pha e2e
- Selector: `[data-ip-quick-translate]`, `[data-ip-quick-running]`, `[data-ip-quick-cancel]`, `[data-ip-block-translation-text]`, `[data-ip-block-translation-line]`, `#ip-block-<i>`.
- Số khối "đang thấy" thật do IntersectionObserver (root = viewport); jsdom không có nó nên vitest gọi `setImportPreviewVisibleBlocks` trực tiếp. Phép đo AC "lệch ≤ 1 px" của bù cuộn CHƯA đo ở WKWebView; vitest không phủ.
- Khối loại nằm trong dải đóng không được vẽ nên không tính là đang thấy.

## Chưa làm / lệch plan
- 🔵 2026-11-11: `Dịch lại` đã có. `quick_translate_blocks` nhận `retranslate: bool` (bỏ tra kho, lưu bằng `REPLACE`); nút `[data-ip-block-translation-retranslate]` ở dòng nguồn khối `done`, lệnh `import.preview.quick_retranslate_block` gửi riêng khối con trỏ. Huỷ hoặc thiếu kết quả khi dịch lại thì câu cũ trở lại (kho vẫn giữ nó).
- Không có chỉ báo riêng khi `deferred > 0`; nút vẫn đó cho phần còn lại.
- `check:*` đã chạy: panel-refs, commands, i18n, tokens xanh; vue-tsc, eslint xanh; `ipc_argument_contract` xanh.

# Ghi chú pha Tests that move

- Thêm `e2e/specs/story-15-3-quick-batch-preview.e2e.mjs` (một ca): máy chủ `node:http` phục vụ trang 30 đoạn và `chat/completions` SSE theo dấu `<<<SEGMENT n`; giữ luồng ở điểm bắt đầu (để đo vị trí khối con trỏ sau cú bấm nút, vì `realClick` tự cuộn) rồi giữ lại sau 2 khối đóng để huỷ giữa chừng.
- Xanh trên WKWebView (`npx wdio run e2e/wdio.conf.mjs --spec …`, ngoài sandbox, ~6 s): 2 khối đóng giữ nguyên sau `Huỷ`, đóng và mở lại màn xem trước hiện lại 2 khối từ kho, `calls` vẫn 1, `library_list_works` không đổi, khối con trỏ lệch ≤ 1 px.
- Đối chứng: đổi `scroller.scrollTop += delta` (ImportPreviewOverlay.vue ~:1076) thành `void delta` ⇒ ca đỏ đúng chỗ (`lệch 168 px`); đã khôi phục và dựng lại `dist/`.
- Chưa chạy: toàn bộ suite một lần (Cargo.toml/lib.rs đổi đăng ký lệnh) và đọc CI/nightly; chưa đo AC "bị huỷ khi Esc" trong e2e (vitest phủ).
