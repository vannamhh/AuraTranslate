---
title: '`Dịch thử` ở Bảng chờ Glossary nhớ kết quả qua kho riêng'
type: 'feature'
ticket: '1'
created: '2026-11-11'
status: 'done'
baseline_revision: 'a9019def8aa1e5781ccc188c0c43a208677904a3'
route: 'full'
route_source: 'auto'
risk: 'high'
review: 'quick'
review_source: 'pinned'
lenses_ran: ['quick']
review_loop_iteration: 0
context: ['{project-root}/_bmad-output/initiative-auratranslate/architecture-auratranslate/architecture-auratranslate.md §AD-64', '{project-root}/_bmad-output/initiative-auratranslate/ux-auratranslate/EXPERIENCE.md §Một khuôn cho mọi câu dịch']
---

<frozen-after-approval reason="human-owned intent — do not modify unless human renegotiates">

## Intent

**Problem:** `Dịch thử` ở khối `Ngữ cảnh` của Bảng chờ Glossary giữ câu AI dịch trong một `Map` của webview, nên đóng Tác phẩm là mất và cùng một câu ở Tác phẩm khác phải trả tiền lần nữa. Lượt chạy không có `run_id`, kết quả không nói mô hình nào dịch, bỏ qua token, chi phí và `maybe_truncated`.

**Approach:** Tracer bullet của Epic 15. Dựng `quick-translation.db` (AD-64 mục 3–4) và `core/quick_translation`. `Dịch thử` đọc bản dịch trong Tác phẩm, rồi kho, rồi mới gọi mạng (mục 6), chỉ lưu kết quả trọn vẹn (mục 5). `QuickTranslateGeneration` có sổ `run_id` thay `ContextReferenceGeneration` (mục 8). Webview hiện đủ các hàng của bảng §Một khuôn cho mọi câu dịch mà bề mặt này có. Cấu hình vẫn là cấu hình chính như hôm nay; cấu hình nhanh là Story 15.2.

## Boundaries & Constraints

**Always:**
- Đúng một hàm mở kho. Lệch schema hoặc tệp hỏng thì xoá rồi dựng lại; mở thất bại thì ghi log và app chạy tiếp.
- `quick_key` (NFC rồi `trim`) là nơi duy nhất sinh khoá. `source_hash` là sha256 của khoá, `UNIQUE`.
- Ghi chỉ khi `state` là `done` và `maybe_truncated` là `false`. Ghi thường là `INSERT OR IGNORE`, `Dịch lại` là `REPLACE`.
- Phép kiểm `generation` nằm TRONG job của `Store::write`.
- Huỷ chỉ có tác dụng khi `run_id` khớp lượt đang chạy.
- Đối chứng (d) phần một câu, (e), (f) phần `run_id`, (h) là phép gỡ thật (AGENTS.md §Tests).

**Never:**
- Đụng `AiTranslateGeneration`, `resolve_call_config`, lô dịch thật.
- Ghi `target_text`, `tm_unit`, `glossary_entry`, `AiCallRecords`, `LastAssembledPromptState`.
- Để `core/quick_translation` phụ thuộc `core::ai` (AD-13).
- Thêm trạng thái `busy` khác bề mặt (15.3), cấu hình nhanh (15.2), Dọn (15.4) hay tự dịch (15.5).
- Mở cổng lắng nghe trong app (AD-45).

## I/O & Edge-Case Matrix

| Scenario | Input / State | Expected Output / Behavior | Error Handling |
|----------|--------------|---------------------------|----------------|
| Trúng Tác phẩm | Câu đã có `target_text` trong Tác phẩm đang mở | `Bản dịch trong Chương` | Không đọc kho, không gọi mạng |
| Trúng kho | Tác phẩm không có bản dịch, kho có hàng | Câu dịch · nhãn AI · `<mô hình> · Dịch lại` | 0 lời gọi provider, kể cả khi lệnh dịch bị gọi không có `retranslate` |
| Trượt, dịch xong | Bấm `Dịch thử` | Chữ chảy, rồi câu dịch + mô hình + token/chi phí; có một hàng trong kho | — |
| Có thể bị cắt | `maybe_truncated: true` | Phần đã nhận · `Câu dịch có thể bị cắt, chưa lưu.` · `Dịch lại` | Không có hàng |
| Huỷ, lỗi | `Huỷ`/`Esc` hoặc lỗi mạng | Về `Chưa có`, hoặc hiện câu lỗi kèm `Dịch thử` | Không có hàng; huỷ trước câu ghi thì job thấy `generation` cũ và bỏ |
| Huỷ sai `run_id` | Lệnh huỷ mang `run_id` cũ | Lượt đang chạy chạy tiếp, vẫn lưu | — |
| Kho không mở được | Tệp kho hỏng mà xoá không được, hoặc thư mục không ghi được | `Không mở được kho dịch nhanh.` · nút tắt | Không gọi mạng |

</frozen-after-approval>

## Code Map

- `src-tauri/src/core/store/mod.rs:179` -- `StoreKind`: thêm `QuickTranslation`, nhánh `as_str` (`:210`) là `"quick-translation"`. Thêm `StoreSpec::quick_translation` bên cạnh `library_index` (`:343`). Không đưa vào `check_glossary_lifecycle` (`:745`). `delete_if_schema_version_differs` (`:1002`) dùng nguyên, nhánh peek lỗi đã xoá tệp hỏng.
- `src-tauri/src/core/store/schema.rs:2901` -- khuôn `LIBRARY_INDEX_MIGRATIONS`: một `Migration` mang DDL, đổi thì tăng số; viết `QUICK_TRANSLATION_MIGRATIONS` theo đúng khuôn này.
- `src-tauri/src/core/library/indexer.rs:164` -- khuôn hàm mở duy nhất (`Indexer::open`); `core/quick_translation` chép khuôn, bọc `Store`.
- `src-tauri/src/lib.rs` -- khuôn `open_library_index` (`:1823`), `data_dir_override` (`:130`), đóng kho khi `RunEvent::Exit` (`:1525`, `:1963`), `ContextReferenceGeneration` được manage ở `:1753`, lệnh đăng ký ở `:1443-1445`.
- `src-tauri/src/commands/aitranslate.rs` -- `ContextReferenceGeneration` (`:808`), `prepare_context_reference_call` (`:825`, giữ nguyên đường cấu hình), `wire::ai_context_reference_translate` (`:2103`), `wire::ai_context_reference_cancel` (`:2141`), `send_with_cancel` (`:1374`), `run_plain_call` (`:716`), `AiTranslateOutcomeWire` (`:1714`, dùng chung với lô thật nên KHÔNG thêm trường), `AiTranslateUsageWire` (`:1080`).
- `src-tauri/src/commands/glossary.rs:623` -- `glossary_context_translation` giữ nguyên; hình dạng của nó bị `tests/config_invariants.rs:1538` ghim.
- `src-tauri/tests/glossary_context_reference_contract.rs` -- `Fake: TranslationProvider` (`:215`), ca wire dùng `tauri::test::mock_builder()` với endpoint `127.0.0.1:1` (`:251`), ca huỷ (`:301`). `tests/config_invariants.rs:2314` ghim dạng async của lệnh dịch.
- `src-tauri/tests/library_index_boundary.rs` -- khuôn cho `quick_translation_boundary.rs` (`EXEMPT_FILES`, `FORBIDDEN`, đối chứng dương/âm, tập tệp chứa needle = `EXEMPT_FILES`). `tests/ai_boundary.rs` đã cấm mọi tệp ngoài `core/ai` gọi `crate::core::ai`.
- `src-tauri/tests/store_contract.rs:1676` -- ca ghim tên `as_str`. Chép helper `temp_dir`/`cleanup` từ `library_index_contract.rs:55-95` (không có `tempfile`; drop store trước khi xoá thư mục).
- `src-tauri/Cargo.toml` -- `unicode-normalization` đã có (`:68`). `sha2` chỉ là phụ thuộc bắc cầu (0.10.9 trong `Cargo.lock`); spine Stack table đã ghi giấy phép.
- `src/glossaryQueueState.ts:106-215` -- `ContextReference` (7 kind), `aiReplies` (`:117`, bỏ), `referenceCall` (`:119`), `abortContextReference` (`:136`), `loadContextReference` (`:150`), `translateGlossaryContextReference` (`:178`), `closeGlossaryQueue` (`:388`), `resetGlossaryQueue` (`:533`).
- `src/config/glossaryContextReference.ts` -- adapter IPC không ném; `src/config/aitranslate.ts:63` là kiểu outcome hiện tại.
- `src/GlossaryQueueOverlay.vue` -- `:106-120` computed, template `:428-485` (`data-gq-context-*`), Esc ở `.gq-scrim` (`:268`). Khuôn "Esc đầu huỷ" là `GlossaryManageOverlay.vue:347` `onEscape`.
- `src/aiUsageLine.ts:11` -- `aiUsageLine(usage)`. `src/i18n/vi.json:1958-1969` -- chuỗi `glossary.queue.context_*`. `src/commands/index.ts:1064, 4557, 4568` và `src/main.ts:1037` -- lệnh `glossary.context.*`.
- `e2e/specs/glossary-context-reference.e2e.mjs` -- khuôn dựng Tác phẩm + Bảng chờ. `e2e/specs/import-preview-mouse.e2e.mjs:99` -- khuôn `node:http` trên 127.0.0.1. `story-16-4-…e2e.mjs:42-75` -- khuôn đếm lời gọi.

## Tasks & Acceptance

Bốn pha theo AGENTS.md, mỗi pha một agent mới; giao lại qua `…-handoff.md` cạnh plan.

**Execution (pha Rust):**
- [x] `src-tauri/Cargo.toml` -- thêm `sha2 = "=0.10.9"` -- AD-64 mục 13; trùng bản đã có trong lock.
- [x] `src-tauri/src/core/store/{mod.rs,schema.rs}` -- `StoreKind::QuickTranslation`, `StoreSpec::quick_translation`, `QUICK_TRANSLATION_MIGRATIONS` với bảng `quick_translation` (`id`, `source_hash BLOB UNIQUE`, `source_text`, `translated_text`, `model`, `created_at`) -- mục 4.
- [x] `src-tauri/src/core/quick_translation/mod.rs` (mới) -- `QuickTranslationStore::open(path)` (hàm mở duy nhất), `quick_key`, `lookup(text) -> Option<{translated_text, model}>`, `save(text, translation, model, replace, still_current: impl Fn() -> bool + Send)`; phép kiểm `still_current` chạy trong job -- mục 3–5.
- [x] `src-tauri/src/commands/aitranslate.rs` -- `QuickTranslateGeneration` thay `ContextReferenceGeneration`: bộ đếm cộng `run_id` của lượt đang chạy; huỷ chỉ khi `run_id` khớp. Lệnh dịch nhận `run_id` và `retranslate`. Thứ tự: thiếu kho thì `store_unavailable`; không `retranslate` mà kho có thì trả `cached`, không gửi; còn lại gửi rồi lưu theo mục 5. Trả wire riêng `QuickTranslateOutcomeWire` (`not_configured | store_unavailable | cached{text, model} | done{model, usage, maybe_truncated} | cancelled`). Thêm lệnh `quick_translation_lookup(text)`. Phần thân gửi-và-lưu là hàm nhận `&dyn TranslationProvider` để test chạy được với `Fake` -- mục 5, 6, 8.
- [x] `src-tauri/src/lib.rs` -- `open_quick_translation` theo khuôn `open_library_index`, tên tệp là hằng `QUICK_TRANSLATION_DB_FILE`, đóng khi `Exit`; đăng ký lệnh mới.
- [x] `src-tauri/tests/quick_translation_contract.rs` (mới), `quick_translation_boundary.rs` (mới), `glossary_context_reference_contract.rs`, `store_contract.rs`, `config_invariants.rs` -- ma trận I/O cộng các đối chứng (d) một câu (ảnh chụp `segment`/`tm_unit`/`glossary_entry` không đổi qua lệnh thật), (e) qua lệnh wire (trúng kho với endpoint `127.0.0.1:1` trả `cached`, trượt thì lỗi kết nối), (f) `run_id` (sai `run_id` không dừng lượt; `AiTranslateGeneration` không đổi), (h) đúng một chỗ mở kho; tệp rác được dựng lại; khoá NFC/`trim` gộp hai cách viết.

**Execution (pha Webview):**
- [x] `src/config/glossaryContextReference.ts`, `src/glossaryQueueState.ts` -- bỏ `aiReplies`; tra Tác phẩm rồi `quick_translation_lookup`; `run_id` tăng dần cấp ở webview; kind mới: `cached{text, model}`, `truncated{text, model}`, `store_unavailable`; giữ `usage` của lượt vừa chạy; `Dịch lại` gọi với `retranslate: true`.
- [x] `src/GlossaryQueueOverlay.vue`, `src/commands/index.ts`, `src/main.ts`, `src/i18n/vi.json` -- dòng nguồn `role="status"` với `<mô hình> · Dịch lại` và `aiUsageLine`; chuỗi bị cắt và kho hỏng; tiêu điểm chuyển sang `Huỷ` rồi trả về nút mới; Esc đầu huỷ lượt theo khuôn `onEscape`; lệnh `glossary.context.retranslate`.
- [x] `tests/frontend/glossaryQueue.test.ts`, `glossaryContextReference.test.ts` -- từng kind, Esc hai bước, tiêu điểm, `run_id` đi theo lệnh huỷ.

**Execution (pha Tests that move):**
- [x] `e2e/specs/story-15-1-quick-translation-cache.e2e.mjs` (mới) -- máy chủ giả tương thích OpenAI chạy `node:http` trên 127.0.0.1 trong tiến trình spec, có đếm lời gọi; cấu hình chính tầng Global là `ollama` (không cần khoá) trỏ vào máy chủ đó. Tác phẩm A: bấm `Dịch thử`, đóng lớp phủ. Tác phẩm B có cùng câu, chưa dịch: mở lại thì câu dịch hiện và bộ đếm không tăng.
- [x] `_bmad-output/initiative-auratranslate/plan-cau-dich-tham-khao-bang-cho-glossary.md:32` -- 🔵 2026-11-11 trỏ AD-64 mục 13: lời hứa giữ trong bộ nhớ hết hiệu lực.

**Acceptance Criteria:**
- Given một câu đã dịch ở Tác phẩm A, when mở Bảng chờ ở Tác phẩm B có cùng câu, then câu dịch và mô hình hiện mà provider giả nhận 0 lời gọi.
- Given một lượt đang chạy, when nhấn `Esc`, then lượt bị huỷ và lớp phủ còn mở; when nhấn `Esc` lần nữa, then lớp phủ đóng.
- Given mỗi đối chứng (d)(e)(f)(h), when gỡ seam tương ứng trong mã sản phẩm, then đúng test canh của nó đỏ, và đỏ đúng lý do.
- Given `quick-translation.db` là tệp rác, when app khởi động, then kho được dựng lại rỗng và `Dịch thử` chạy được; given kho không mở được, then app vẫn chạy và Bảng chờ hiện `Không mở được kho dịch nhanh.`

## Implementation Notes

- Trúng kho trả `cached` trước khi đọc cấu hình, nên câu đã lưu vẫn hiện khi AI chưa cấu hình; `Dịch lại` khi đó bị tắt.
- Lưu thất bại sau một lượt đã trả tiền thì ghi log và vẫn trả `done`; đọc kho lỗi thì `store_unavailable`, không gọi mạng.
- `run_plain_call` và `ReplyShape::Plain` bị xoá vì không còn ai gọi.
- e2e: bật provider `ollama` (không cần khoá) trỏ vào máy chủ `node:http` giả trong tiến trình spec. Trên máy Ice, bản e2e chưa ký vẫn hỏi mục keychain thật `ai_provider_api_key` khi mở Bảng chờ (`ai_config_get` giữ khoá `OpenWorkState` trong lúc chờ hộp thoại), nên lần chạy đầu treo cho tới khi Ice trả lời hộp thoại.
- Nút `Dịch thử` được Vue dựng lại khi hàng đi qua các bước tra Tác phẩm → kho; spec phải chờ `data-kind="none"` rồi mới lấy nút, nếu không WebKit báo `Couldn't find element for "pointerMove"`.
- Đối chứng (d) qua lệnh thật nằm ở e2e: wire Rust tạo `OpenAiChatClient` thật nên đường thành công chỉ chạy được khi có máy chủ.

## Plan Change Log

## Review Triage Log

Lượt 1 (lens `quick`): high 0 · medium 2 · low 5 · false 4 · maybe-false 0.

| # | Phát hiện | Verdict | Route | Bằng chứng / hành động |
|---|---|---|---|---|
| 1 | Lệnh huỷ tới Rust trước `begin` thì mất | low | rejected | Bộ đếm cũ có đúng cửa sổ đó (`next()` sau lệnh huỷ cũng ra thế hệ mới); `begin` chạy trước mọi `await`, còn người dùng chỉ bấm được `Huỷ` sau khi nút đã đổi; sửa cần thêm trạng thái mộ bia. |
| 2 | Đối chứng (d) gọi thẳng `quick_translate`, không qua lệnh thật | medium | patch | Spec e2e chụp `read_open_chapter_segments`, `tm_count_pairs`, `glossary_list_entries` trước `Dịch thử` và sau khi lưu, so bằng nhau. |
| 3 | `Dịch lại` khi chưa cấu hình xoá câu đã lưu khỏi màn | medium | patch | Nút `Dịch lại` có `:disabled="contextReferenceUnconfigured"` như `Dịch thử` (EXPERIENCE: chưa cấu hình ⇒ nút tắt); thêm ca vitest. |
| 4 | `Dịch lại` lỗi bỏ câu cũ | low | rejected | Bảng EXPERIENCE: lỗi ⇒ câu lỗi kèm `Dịch thử`; bấm `Dịch thử` trả `cached` với 0 lời gọi mạng nên câu cũ quay lại. |
| 5 | Doc comment tiếng Việt trên `StoreKind::QuickTranslation`, `StoreSpec::quick_translation` | low | patch | Xoá, tên đã mang nghĩa (AGENTS.md §Code comments). Ý về chú thích giấy phép cho `sha2`: false, `unicode-normalization` bên cạnh cũng không có, giấy phép ở spine Stack table. |
| 6 | Ghi chú census ở `config_invariants.rs` mô tả lệnh huỷ cũ | low | patch | Sửa câu: huỷ chỉ bơm `AtomicU64` khi `run_id` khớp. |
| 7 | Nhánh `store_unavailable` trong `contextReferenceSource` không bao giờ chạy | low | patch | Xoá dòng. |
| 8 | Hàng `truncated` không hiện mô hình | false | rejected | Bảng EXPERIENCE cho "Câu có thể bị cắt" chỉ có phần đã nhận, câu cảnh báo và `Dịch lại`. |
| 9 | Task chưa khớp diff: `glossaryQueue.test.ts` không đổi, ô chưa đánh dấu, Implementation Notes trống | false | rejected | Ca test nằm ở `glossaryContextReference.test.ts`; ô và ghi chú được điền ở bước trình bày, không phải lỗi mã. |
| 10 | Chưa có canh cho dây khởi động; boundary không đòi `open_quick_translation` là chỗ gọi duy nhất | low | rejected | Cùng khuôn với `library_index_boundary.rs` mà spine chọn cho (h); đường `store_unavailable` có ca wire. |
| 11 | e2e không có ca Esc hai bước; `server.close` có thể treo | false | rejected | AC Esc không đòi e2e và có ca vitest; spec chạy xong trong 3 s, không treo. |

## Design Notes

Wire riêng thay vì thêm `model` vào `AiTranslateOutcomeWire`, vì kiểu đó chung với lô thật: thêm trường sẽ đổi hình dạng của đường mà epic không được đụng. Lệnh dịch tự tra kho lần nữa (không chỉ webview) để (e) đứng ở Rust: webview hỏng hay hai lượt chạy chồng nhau vẫn không trả tiền hai lần. `quick_translation_lookup` là lệnh riêng, không gộp vào `glossary_context_translation`, để tầng khối ở 15.3, nơi chưa có Tác phẩm, dùng lại được. Bề mặt và `busy` để 15.3 thêm vào sổ `run_id`.

## Verification

**Commands:**
- `npm run build && cd src-tauri && cargo test --test quick_translation_contract --test quick_translation_boundary --test glossary_context_reference_contract --test ai_boundary --test store_contract --test config_invariants` -- expected: xanh.
- `npx vitest run tests/frontend/glossaryQueue.test.ts tests/frontend/glossaryContextReference.test.ts` -- expected: xanh.
- `npm run check:commands && npm run check:i18n` -- expected: xanh, `POINTER_DEBT` không tăng.
- `npm run test:e2e -- --spec e2e/specs/story-15-1-quick-translation-cache.e2e.mjs` (ngoài sandbox, binary `--features wdio`) -- expected: xanh.
- Thêm `sha2` chạm `Cargo.toml`/`Cargo.lock` và `lib.rs` đăng ký lệnh ⇒ chạy đủ `cargo test` một lần trước khi báo xong (AGENTS.md §Tests).
