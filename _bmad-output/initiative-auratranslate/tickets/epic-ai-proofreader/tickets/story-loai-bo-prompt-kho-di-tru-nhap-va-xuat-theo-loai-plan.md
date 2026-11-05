---
title: 'Loại bộ prompt: kho, di trú, nhập và xuất theo loại'
type: 'feature'
ticket: '13'
created: '2026-11-05'
status: 'built'
baseline_revision: '7afbf4a9b08c3d835aee3741f38af633bd701755'
route: 'full'
route_source: 'auto'
risk: 'high'
review: 'quick'
review_source: 'pinned'
lenses_ran: ['quick']
review_loop_iteration: 0
context:
  - '{project-root}/_bmad-output/initiative-auratranslate/architecture-auratranslate/architecture-auratranslate.md §AD-68 mục 1, 8, 9, 11 (chỉ đọc mục AD-68)'
---

<frozen-after-approval reason="human-owned intent — do not modify unless human renegotiates">

## Intent

**Problem:** Thư viện prompt chỉ có một loại bộ, nên chỉ dẫn Proofreader (FR160) không có chỗ ở mà không lẫn vào prompt dịch; tệp `.prompt.md` (FR79) cũng không mang loại.

**Approach:** Thêm loại `translate`/`proofread` vào `prompt_set` ở hai kho và cho loại đi qua mọi đường thư viện, nhập và xuất (AD-68 mục 1, 8, 9); di trú thêm luôn bốn cột của 9.14. Đường dịch giữ nguyên hành vi.

## Boundaries & Constraints

**Always:**
- Loại là tham số bắt buộc ở mọi hàm `core/promptset/` và mọi lệnh thư viện (đọc, tạo, đổi tên, sửa thân, xoá, xuất), không mặc định ở Rust. Lệnh theo `id` lọc `WHERE id = ? AND kind = ?`; lệch loại ⇒ lỗi không-tìm-thấy hiện có.
- Nhập lấy loại từ tệp, không từ màn; va chạm tên và ghi đè tầng (AD-18) chỉ xét trong cùng loại.
- Đường dịch (`aiprompt.rs`, `aitranslate.rs`, `prompt_set_choice_get/set`) chỉ thấy `translate`; chữ ký IPC của các lệnh này không đổi.
- Bộ `translate` xuất ra trùng từng byte với hôm nay, cùng tên tệp.
- Webview truyền `'translate'` ở mọi chỗ gọi hiện có; hành vi màn thư viện không đổi.
- Ca canh (e) phần thư viện, (f), (k) có đối chứng bằng phép GỠ thật.

**Never:**
- Không thêm lệnh IPC mới, không đọc bốn cột mới (lựa chọn và vết là 9.14), không thêm giao diện cho bộ `proofread` (9.15).
- Không chạm prompt nào của đường dịch hay của lượt quét.
- Không viết `kind: translate` khi xuất.

## I/O & Edge-Case Matrix

| Scenario | Input / State | Expected Output / Behavior | Error Handling |
|---|---|---|---|
| Kho cũ di trú | `global.db` v15 / `project.db` v42 có bộ "A" | v16 / v43; "A" đọc được với `translate`; tạo `proofread` "A" được; `work` có `off = 0`, tên `NULL`; `proof_scan` hai cột `NULL` | — |
| Trùng tên cùng loại | tạo `translate` "A" lần hai | như hôm nay | lỗi trùng tên hiện có |
| Che tầng | Work `proofread` "A", Global `translate` "A" | danh sách `translate` thấy bộ Global, không bị che | — |
| Đường dịch | chỉ có `proofread` "A", dịch với tên "A" | như khi "A" không tồn tại | lỗi hiện có |
| Tệp ba dòng | `---`/`name: A`/`---`/thân | `translate` "A" | — |
| Tệp bốn dòng | `---`/`name: A`/`kind: proofread`/`---`/thân | `proofread` "A" | — |
| `kind:` lạ | dòng 3 `kind: x` (kể cả `kind: translate`) | từ chối cả tệp | `ParseIssue` mới, `MessageKey` mới, chuỗi trong `vi.json` |
| Thiếu dấu đóng | dòng 3 `kind: proofread`, dòng 4 khác `---` | từ chối | `MissingClosingDelimiter` |
| Xuất | `proofread` "A" | `A.proofread.prompt.md`; trùng ⇒ `A-2.proofread.prompt.md` | — |
| Hai loại cùng tên | xuất cả hai | `A.prompt.md` và `A.proofread.prompt.md` | — |

</frozen-after-approval>

## Code Map

Đường dẫn dưới `src-tauri/` trừ khi ghi khác; số dòng ở `7afbf4a9`.

- `src/core/store/schema.rs` -- `PROMPT_SET_DDL` :1373, chỉ mục `idx_prompt_set_name` :1387; `GLOBAL_MIGRATIONS` :800 (cuối 15), `PROJECT_MIGRATIONS` :2306 (cuối 42); khuôn ALTER: `WORK_TM_ONLY_MINE_DDL` :1111, `PROOF_IGNORE_BLOCKED_COUNT_DDL` :1106; nhiều câu: `CHAPTER_ORIGIN_DDL` :2121. `migrate` :2954 bọc mỗi bước một giao dịch. `proof_scan` ở `PROOFREAD_MEMORY_DDL` :1071.
- `src/core/promptset/mod.rs` -- `PromptSetTier` :83 (`as_str`/`from_wire`) là khuôn cho `PromptSetKind`.
- `src/core/promptset/store.rs` -- `load_prompt_set_tier` :63, `resolve_two_tiers` :115 (qua `apply_override`, `ScopeKind::Prompt`: lọc loại TRƯỚC khi phân giải), `create` :194, `rename` :251, `update_body` :308, `delete` :337, `load_one` :388, `import_into_tier` :451 (va chạm unique ⇒ stale :485; UPDATE lạc quan :499). `read/write_chosen_name` :356/:368 là của đường dịch.
- `src/core/promptset/exchange.rs` -- `render` :56, `ParseIssue` :64 + map `IpcError` :115-158, `parse` :184 (đọc cứng 3 dòng), `classify` :260 theo tên.
- `src/core/promptset/exchange_io.rs` -- `export_file_name` :65, `unique_export_path` :85 (chỉ bóc `.prompt.md`; phải chèn `-N` trước `.proofread.prompt.md`).
- `src/commands/promptset.rs` -- wire từ :683: chín lệnh thường, ba async; `PendingPromptImport` :405 (trường `global_kind`/`work_kind` là kế hoạch nhập, đặt tên loại bộ khác đi, vd `set_kind`). Đăng ký `lib.rs:1375-1392`; `lib.rs:839` gọi `wire::prompt_set_create`.
- `src/commands/aiprompt.rs` -- `resolve_two_tiers` :533 và :758: truyền `Translate`. (`resolve_two_tiers` ở `aitranslate.rs` là của aiconfig, không chạm.)
- `src/core/i18n/mod.rs:785-833` + `src/i18n/vi.json` -- khoá lỗi prompt set.
- Webview: `src/config/promptset.ts` (wrapper :111-478, `PromptSetWire` :41); gọi ở `src/promptSetState.ts`, `src/promptSetImportState.ts`, overlay thư viện/nhập; không có bindings sinh tự động.
- Test phải sửa cùng lượt (không đỏ vì lý do mới):
  - Ca hạ phiên bản phải gỡ cả thay đổi bước 16/43 trước khi mở lại (ALTER ADD COLUMN không lặp được): `prompt_set_contract.rs:880-895`, `proofread_decision_contract.rs:275-286`, `lifecycle_contract.rs:963`, `tm_contract.rs:778,817,1220,1246-1250`.
  - Số phiên bản 15→16: `pinned_contract.rs:92,101,698`, `store_contract.rs:1202,1318`, `tm_contract.rs:1187,1208`, `glossary_contract.rs:3445`, `glossary_unconfirm_contract.rs:308`. 42→43: `pinned_contract.rs:263,273`, `segment_contract.rs` (1017…6453, và `STEP_THIRTYFOUR` :2200/:2243 thành 43 phần tử, bước giả 44), `chapter_origin_contract.rs:1152`, `segment_role_contract.rs:870`, `tm_contract.rs:786,823,1234`, `review_harvest_contract.rs:688,701`, `prompt_set_contract.rs:895`.
  - Chữ ký lệnh: `config_invariants.rs:1383-1389,1744-1754`, `ipc_contract.rs:2371,2560,2589`, `ai_boundary.rs:1955`; bảng đếm `config_invariants.rs:2280` giữ 9/3.
  - Gọi `prompt_set_create`: `ai_prompt_contract.rs`, `ai_translate_contract.rs`, `ai_batch_wire.rs`, `ai_translate_wire.rs`, `ipc_contract.rs`; khoảng 17 tệp `tests/frontend/` và `e2e/specs/story-12-14-prompt-library.e2e.mjs`.

## Tasks & Acceptance

Bốn pha, mỗi pha một agent mới, bàn giao qua `epic-ai-proofreader/handoff-9-13-phases/`.

**Execution:**
- [x] `src/core/store/schema.rs` -- global bước 16, project bước 43 (project gồm cột `work` và `proof_scan` của AD-68 mục 3, 7) -- AD-68 mục 9.
- [x] `src/core/promptset/{mod,store}.rs` -- `PromptSetKind` + loại ở mọi hàm và truy vấn; phân giải hai tầng theo loại.
- [x] `src/core/promptset/{exchange,exchange_io}.rs` + `core/i18n/mod.rs` + `src/i18n/vi.json` -- render/parse bốn dòng, `ParseIssue` loại lạ, tên tệp và hậu tố `-N`.
- [x] `src/commands/promptset.rs`, `src/commands/aiprompt.rs`, `src/lib.rs` -- tham số `kind` cho lệnh thư viện (cả `PromptSetExportRef`); bản xem trước nhập trả `kind`; đường dịch truyền `Translate`.
- [x] `src/config/promptset.ts` + nơi gọi -- truyền `'translate'`; kiểu wire có `kind`.
- [x] `tests/prompt_set_contract.rs`, `tests/prompt_set_exchange_contract.rs`, một tệp di trú -- ca (e) thư viện + lắp dịch, (f), (k) theo bảng trên; cập nhật các test ở Code Map.

**Acceptance Criteria:**
- Given gỡ thật nhánh đọc dòng `kind:`, when chạy test (f), then ca bốn dòng và ca khứ hồi `proofread` đỏ vì loại/lỗi phân tích, không vì lý do khác.
- Given gỡ thật điều kiện loại trong truy vấn đọc của `store.rs`, when chạy test (e), then danh sách hoặc lắp prompt dịch thấy bộ `proofread` và ca đỏ.
- Given gỡ thật bước di trú mới, when chạy test (k), then ca đỏ.
- Given tệp `.prompt.md` xuất từ bộ `translate` trước thay đổi, when xuất lại sau, then trùng từng byte.

## Implementation Notes

- Loại là `PromptSetKind` bắt buộc ở mọi hàm `core/promptset/` và lệnh thư viện; lệnh theo `id` lọc `AND kind = ?`, lệch loại trả `PromptSetNotFound` (test `by_id_commands_with_the_wrong_kind...`).
- Di trú: global bước 16, project bước 43 (chỉ mục `(kind, name)` thay `idx_prompt_set_name`; thêm bốn cột của 9.14, chưa ai đọc). Các ca hạ phiên bản trong tests phải gỡ cả chỉ mục, cột `kind` và bốn cột trước khi mở lại.
- Tệp `.prompt.md`: ba dòng = `translate`, bốn dòng `kind: proofread` = `proofread`; mọi dòng 3 mở đầu `kind:` khác đúng `kind: proofread` (kể cả `kind: translate`) bị từ chối bằng `ParseIssue::UnknownKind`. Bộ `translate` xuất trùng byte với trước.
- Tên tệp xuất `A.proofread.prompt.md`; `unique_export_path` nhận loại và chèn `-N` trước hậu tố của loại đó, nên bộ `translate` tên `A.proofread` vẫn ra `A.proofread-2.prompt.md` như trước. Tệp `proofread` thiếu `---` dòng 4 báo dòng 4 (`MissingProofreadClosingDelimiter`), vì báo dòng 3 dẫn người dùng sửa tệp thành bộ `translate`.
- Ca (e) lắp dịch nằm ở `ai_prompt_contract.rs`, (e) thư viện ở `prompt_set_contract.rs`, (f) ở `prompt_set_exchange_contract.rs`, (k) ở tệp mới `prompt_set_kind_migration_contract.rs`.
- `PromptSetExportFileWire` mang `kind`, bản xem trước nhập mang `set_kind` (tránh trùng tên với `global.kind`/`work.kind` của kế hoạch nhập); guard webview đòi cả hai và có ca canh.

## Plan Change Log

## Review Triage Log

Lượt 1 (quick, 2026-11-05): high 0 · medium 3 · low 4 · false 1 · maybe-false 0. Mọi mục còn lại route patch.

- false — test exchange không dịch: bản diff giao review cũ hơn cây; cây đã so `PromptSetKind::…`, target chạy xanh 28/28.
- medium · patch — tệp `proofread` thiếu `---` dòng 4 báo "dòng 3"; làm theo lời báo thì tệp thành bộ `translate` (AD-68 Prevents 2).
- medium · patch — lọc loại của `load_one` không ca nào canh; gỡ `AND kind = ?2` vẫn xanh.
- medium · patch — guard `kind` của webview và đối số `'translate'` không ca nào chạm; fixture xem trước giả thiếu trường.
- low · patch — `unique_export_path` đoán hậu tố từ chuỗi; bộ `translate` tên `A.proofread` khi trùng đổi tên tệp, trái ràng buộc frozen.
- low · patch — `kind` của bộ nằm cạnh `global.kind` (kế hoạch nhập) trên cùng wire; đổi thành `set_kind`.
- low · patch — thông điệp assert và doc comment cạnh dòng sửa còn ghi 15/42, `idx_prompt_set_name`, "thân sau dòng ba".
- low · patch — chú thích mới tiếng Việt/nhắc lại dòng kế, trái luật chú thích AGENTS.md.

## Design Notes

Lệnh theo `id` vẫn nhận loại vì AD-68 mục 1 đòi tham số bắt buộc ở mọi đường; lọc `AND kind = ?` làm loại thành phép kiểm chứ không thừa. Tệp `proofread` nhập từ màn dịch trước 9.15 vào kho mà chưa màn nào hiện; chấp nhận được vì trước 9.15 không có đường tạo bộ `proofread`, và bản xem trước đã mang `kind` cho 9.15.

## Verification

**Commands:**
- `npm run build` rồi `cd src-tauri && cargo test --test prompt_set_contract --test prompt_set_exchange_contract` -- xanh trong vòng sửa.
- Một lượt đủ (`npm run build`, `cargo test`, vitest, các cổng) ở cuối pha Tests -- bắt buộc vì đổi di trú (AGENTS.md).
- `npx vitest run tests/frontend/promptSetState.test.ts` -- xanh sau pha Webview.
- Một spec e2e thư viện prompt (`story-12-14-prompt-library`) trước khi push -- bắt TDZ/vòng import mà vitest bỏ lọt.
