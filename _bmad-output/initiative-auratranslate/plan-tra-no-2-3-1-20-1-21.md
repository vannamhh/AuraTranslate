---
title: 'Trả 12 mục nợ có chủ Story 2.3 · 1.20 · 1.21'
type: 'bugfix'
ticket: ''
created: '2026-10-20'
status: 'built'
baseline_revision: 'eb97bbf52861de3feaa9638ec61c53c2203b5c8c'
route: 'full'
route_source: 'auto'
risk: 'medium'
review: 'quick'
review_source: 'pinned'
lenses_ran: ['quick']
review_loop_iteration: 0
context: []
---

<frozen-after-approval reason="human-owned intent — do not modify unless human renegotiates">

## Intent

**Problem:** 12 mục mở/🟡 trong `deferred-work.md` có `Chủ:` cuối là Story 2.3, 1.20 hoặc 1.21 (`:232 :270 :275 :2233 :2603 :4660 :1282 :1356 :331 :337 :339 :677`). Một mục là lỗi người dùng chạm tới được hôm nay: tiêu điểm trên thanh trượt `range` của ReadingMode làm các hợp âm trần chết im lặng.

**Approach:** Mỗi mục kết thúc bằng một trong ba dạng đóng (`→ ✅ ĐÃ ĐÓNG` · `→ 🟡` · `→ KHÔNG LÀM`), nối ngay trong mục. Phần chỉ người kiểm được trong app thật chuyển sang `Chủ: Epic 1` hoặc `Chủ: Epic 2`.

**Decisions (Ice 2026-10-20; giữ cả 12 mục trong một plan):**
- :232: hợp âm in được chữ chạy trên control không phải văn bản; mũi tên, Space, Home/End, PgUp/PgDn để lại cho control.
- :270: `.db` không đổi byte; `-wal`/`-shm` được phép tồn tại.
- :275: thêm `StoreError::CheckpointFailed`, dùng lại `MessageKey::StoreWriteFailed`, không key i18n mới; lúc mở (`backup_before_migration`) vẫn là `OpenFailed`.
- :331: lỗi ngữ nghĩa trong `resolve_one` thành `StoreError::OpenFailed` có detail, trả bằng `?`.
- :339: trần key 256 B, value 64 KiB; vượt trần ⇒ `StoreError::WriteFailed` có detail, 0 hàng ghi.
- :1282: `source_version` lấy phía Rust từ `DictLayers` (frontend không gửi); ghim cũ NULL không cảnh báo; chỉ so `source_version`, không so `built_at`.
- :677: KHÔNG LÀM.
- :1356: `.lookup-spine-count` một dòng (`nowrap` + `ellipsis`); không nới `--lookup-head-height`.

## Boundaries & Constraints

**Always:** Mỗi seam mới có một ca đỏ được khi GỠ thật. Bump `global.db` theo AD-30. Từ `vi.json` mới phải qua `check:i18n`. Sửa sổ chỉ bằng dòng `→` nối thêm, không xoá mục.

**Never:** Không thêm `MessageKey` mới. Không nới `--lookup-head-height`. Không chạm nợ của chủ khác. Không đánh dấu story 1.20/1.21/2.3 là `done`.

## I/O & Edge-Case Matrix

| Scenario | Input / State | Expected | Error Handling |
|---|---|---|---|
| Hợp âm trần trên input không phải văn bản | focus `range`/`checkbox`, gõ `D` | lệnh chạy | phím mũi tên/Space vẫn về control |
| Hợp âm trên input văn bản | focus `<input type=text>` | không dispatch | — |
| Lược đồ mới hơn, WAL còn frame | `.db` của bản sau, `-wal` chưa checkpoint | `SchemaTooNew`, `.db` không đổi byte | `-wal`/`-shm` được phép tồn tại |
| Ghi config quá trần | `value` > trần | từ chối, 0 hàng ghi | `StoreError::WriteFailed` có detail |
| Đọc config lỗi thật | bảng `config_value` hỏng | `bootstrap_config` trả `store.read_failed` | — |
| Ghim từ dữ liệu cũ | `source_version` ghim ≠ hiện tại | hiện một dòng cảnh báo | NULL ⇒ không cảnh báo |

</frozen-after-approval>

## Code Map

- `src/commands/keys.ts:436-441` `isTypingZone`, listener capture ở `:601`. Không có shadow DOM trong sản phẩm (nửa (1) của :232 không chạm tới được). `ReadingMode.vue:579,591` có hai `range`. Hợp âm trần ở `src/commands/index.ts:2578-2708`.
- `tests/frontend/keysModZYieldsToNativeEditor.test.ts`: đã lái target giả, dùng cho :232.
- `src/panels/editorPanelState.ts:935-940` `flushEditorBeforeDiscreteWrite`, `switchChapter` ~1980. Mẫu cuộc đua thật: `tests/frontend/editorConfirmSegment.test.ts:68-77,193-207` (`midFlightHooks`). Đích: `tests/frontend/chapterPosition.test.ts` (`recordSave`, flush thật).
- `src/panels/LookupPanel.vue:942-958` (`.lookup-head`: 76px, `overflow:hidden`), `:990-996` `.lookup-spine` `flex-wrap:wrap`. Spine chỉ còn số đếm (`4daf368`). e2e: `e2e/specs/story-12-8-lookup-sources-popup.e2e.mjs`.
- `src-tauri/src/core/store/mod.rs:662` `Store::open`, bước 3 `drop(conn)` trước `SchemaTooNew`. `rusqlite` 0.40.1 có `DbConfig::SQLITE_DBCONFIG_NO_CKPT_ON_CLOSE`. Test: `tests/store_contract.rs:1430`, helpers `temp_dir` `db_path` `sidecar` `spec_with` `quiet_tuning`.
- `src-tauri/src/core/store/pragmas.rs:351-367` `wal_checkpoint`. Gọi từ `checkpoint.rs:333,375` và `schema.rs:2833` (lúc mở; giữ `OpenFailed`). Các variant `StoreError` ở `mod.rs:378`. Test: `store_contract.rs:1502`.
- `src-tauri/src/core/scope/store.rs:383-388` `resolve_one` (`debug_assert!` + `unwrap_or_default`). `:444` `save_value`. `src/commands/config.rs:148` `bootstrap_config`. Test: `tests/scope_contract.rs` (`:53-82` helpers, `:1104` vocabulary). Cách làm hỏng việc đọc: `DROP TABLE config_value` (`review_harvest_contract.rs:576`).
- `src-tauri/src/core/store/schema.rs:156` `PINNED_ENTRY_DDL`; `GLOBAL_MIGRATIONS` `:757` có 14 bước. `src/commands/pinned.rs:139` `pin_entry`. `DictSource.source_version` có sẵn ở Rust (`core/dict/mod.rs:508`) lẫn frontend (`src/config/dict.ts:326`). Assert cần sửa: `pinned_contract.rs:99`, `store_contract.rs:1290`, `pinned_contract.rs:485`. Frontend: `src/config/pinned.ts`, `lookupHistoryState.ts:545`.
- `EditorPanel.vue` đã bị xoá ở `ca33072`. Các handler kế thừa nằm ở `GridPanel.vue`. Story 2.8 đã vá `caretPositionFromPoint` (`GridPanel.vue:690-713`).

## Tasks & Acceptance

**Execution:**
- [ ] `src/commands/keys.ts` -- input không phải văn bản không còn là vùng gõ cho hợp âm in được chữ -- vá :232 (2)
- [ ] `tests/frontend/keysModZYieldsToNativeEditor.test.ts` -- ca range/checkbox/text -- lưới cho :232
- [ ] `tests/frontend/chapterPosition.test.ts` -- gõ trong lúc lô đầu đang bay ⇒ `chapter-still-dirty`, `openChapter` không được gọi -- đóng :4660
- [ ] `src/panels/LookupPanel.vue` + e2e 12-8 -- `.lookup-spine-count` một dòng; assert `scrollHeight <= clientHeight` ở panel hẹp -- :1356
- [ ] `core/store/mod.rs` + `store_contract.rs` -- `NO_CKPT_ON_CLOSE` trước `drop(conn)`; fixture WAL có frame treo -- :270
- [ ] `core/store/{mod,pragmas,schema}.rs` + `store_contract.rs` -- `CheckpointFailed` -- :275
- [ ] `core/scope/store.rs` + `scope_contract.rs` -- `resolve_one` trả `OpenFailed`; ca read_failed qua `bootstrap_config`; trần 256 B / 64 KiB -- :331 :337 :339
- [ ] migration bước 15 + `pinned.rs` + `pinned.ts` + Lookup UI + `vi.json` + test -- `source_version` từ `DictLayers` -- :1282
- [ ] `deferred-work.md` -- nối `→` cho cả 12 mục. :2233 và :2603 → KHÔNG LÀM (`ca33072`); phần gõ tay bằng IME còn lại cho ô rỗng → `Chủ: Epic 2`. :677 → KHÔNG LÀM.

**Acceptance Criteria:**
- Given HEAD sau story, when `node scripts/check-debt-owner.mjs --list-status` chạy, then không còn mục nào có `Chủ:` cuối là Story 2.3, 1.20 hoặc 1.21 ở trạng thái `open`.
- Given mỗi seam mới, when gỡ thật seam đó, then đúng ca của nó đỏ, và đỏ vì đúng lý do.
- Given app mở `global.db` v14, when nâng lên v15, then các ghim cũ còn nguyên.


## Implementation Notes

- :4660: nhánh `chapter-still-dirty` của `openChapterById` không tới được qua API thật (`noteEditorEdit` bỏ phím khi `dangChuyenChuong` bật, và khoá bật trước flush); ca "gõ trong lúc lô đầu bay" của plan không thể assert. Viết hai cuộc đua tới được trong `chapterPosition.test.ts`; sổ đóng KHÔNG LÀM cho nhánh chết.
- :1356: CSS xong nhưng e2e 12-8 SKIP cả hai ca (app e2e có 0 nguồn từ điển), nên không có số đo; sổ ghi 🟡 `Chủ: Epic 1` cho lượt đo thật trong app, không đóng theo suy luận.
- :232: chỉ nửa (2) làm; nửa (1) shadow DOM đóng KHÔNG LÀM vì `src` không có `attachShadow`/`customElements.define`.
- :2233 và :2603 đóng KHÔNG LÀM theo `ca33072`; phần gõ tay IME vào ô rỗng chuyển `Chủ: Epic 2`.
- :1282: `source_version` đọc phía Rust từ `DictLayers`, `pin_entry` IPC không đổi; `PinnedEntry` thêm trường thứ 7 nên `pinned_contract` wire-shape đỏ cho tới khi `pinned.ts` có trường. Migration bước 15 kéo theo sửa assert v14 sang v15 ở `pinned_contract`, `store_contract`, `tm_contract`, `glossary_contract`, doc `segment_contract`.
- :270 dùng `NO_CKPT_ON_CLOSE` (rusqlite 0.40.1); `-wal`/`-shm` vẫn được phép tồn tại.
- :275 `CheckpointFailed` chỉ ở đường chạy nền; `backup_before_migration` map về `OpenFailed` nên nhãn lúc mở không đổi.
- Mọi seam mới có đối chứng gỡ thật đỏ đúng lý do (chi tiết trong commit). Dòng cảnh báo ghim cũ chưa được nhìn trong app thật.
- Lượt chạy cả bộ bắt một hồi quy ở :232: bản đầu cho mọi phím trừ phím điều hướng lọt qua input không phải văn bản, nên `Escape` trên radio phân loại xoá vết cắt (`editorClearSourceCuts.test.ts` ⑫); nay chỉ hợp âm in được chữ lọt, đúng quyết định đã ký.

## Plan Change Log

## Review Triage Log

Lượt 1 (quick): high 0 · medium 0 · low 6 · false 2 · maybe-false 0. Vá trực tiếp, không quay vòng plan.
- low · patch: `scope_contract` ceiling assert đọc `mode()` nên giữ trên cả hai nhánh → nay đếm `COUNT(*) FROM config_value` = 0.
- low · patch: ca :331 gọi thẳng `resolve_global_kind`, không canh dây nối (không tới được bằng loại thật) → sửa câu sổ cho đúng phạm vi ca đo, không thêm seam.
- low · patch: `set_db_config` trượt thay `SchemaTooNew` bằng `OpenFailed` → bỏ `?`, phán quyết `SchemaTooNew` luôn giữ.
- low · patch: chú thích mới tiếng Việt/nhắc lại tên (`pinned.rs`, `pinned.ts`, `pinIsStale`, `MAX_*_BYTES`, doc `load_global_config`) → tiếng Anh hoặc xoá.
- low · patch: e2e 12-8 khôi phục `head.style.width` chỉ ở đường thành công → `try/finally`. Vế "ca skip không canh gì" đã ghi 🟡 `Chủ: Epic 1`; vế sửa dòng Verification là sửa plan nên bác.
- low · reject: `pin_entry` dựng cả danh sách attribution mỗi lượt ghim; ghim là thao tác hiếm, sửa cần hàm mới.
- false: :4660 đóng KHÔNG LÀM "chưa ai ký" — Intent cho phép ba dạng đóng cho mỗi mục; đã kiểm `flush.markChanged` chỉ có một nơi gọi (`noteEditorEdit`, bị khoá `dangChuyenChuong` chặn) nên nhánh không tới được; nhánh giữ lại vì `flushEditorBeforeDiscreteWrite` vẫn trả `'still-dirty'` theo hợp đồng.
- false: "ghim lúc nguồn chưa nạp mãi NULL" — mục ghim lấy từ `targetsOf(LookupResponse)`, tức chỉ từ nguồn đã nạp.
- (tự bắt, ngoài lens) `check:panel-refs` đỏ ở `NON_TEXT_INPUT_TYPES` (Set cấp module) và regex hai dòng → hai regex một dòng.

## Verification

**Commands:**
- `npx vitest run tests/frontend/keysModZYieldsToNativeEditor.test.ts tests/frontend/chapterPosition.test.ts` -- xanh
- `npm run build && cd src-tauri && cargo test --test store_contract --test scope_contract --test pinned_contract` -- xanh
- Có migration (shared wiring) ⇒ chạy cả bộ một lần trước khi báo; `npm run check:debt-owner` và `check:i18n` -- xanh
- `npm run test:e2e -- --spec e2e/specs/story-12-8-lookup-sources-popup.e2e.mjs` (ngoài sandbox) -- xanh
