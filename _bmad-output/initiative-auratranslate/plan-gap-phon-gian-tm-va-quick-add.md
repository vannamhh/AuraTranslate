---
title: 'Đóng nợ gấp phồn/giản: TM, quick-add và lọc ứng viên'
type: 'bugfix'
ticket: ''
created: '2026-11-01'
status: 'done'
baseline_revision: 'afdc4205e29c0719a679b2dc1244f64eff974d57'
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

**Problem:** Sau AD-63, Glossary khớp xuyên dạng chữ nhưng TM không tìm được câu khác dạng chữ, dải chốt không có gợi ý TM, và quick-add cùng quét ứng viên vẫn đề xuất `大紀元` khi Glossary đã có `大纪元`.

**Approach:** Ice chốt 2026-11-01: gấp TM luôn; quick-add và lọc ứng viên so sánh sau gấp, giữ nguyên khoá `source_term`, không migration; hai từ khác nhau như `发`/`髮` vẫn thêm được mục riêng.

## Boundaries & Constraints

**Always:**
- Mọi phép so sánh mới dùng đúng một hàm gấp của `core::matching` (AD-17).
- Điểm khớp mờ của câu chỉ khác dạng chữ vẫn tối đa 99.
- Quick-add chỉ báo mục khác dạng chữ, không chặn lưu và không chuyển dải sang chế độ sửa.
- Chữ đang lưu trong TM, Glossary và ứng viên giữ nguyên, không ghi dạng đã gấp xuống đĩa.

**Never:**
- Không gấp `ngrams`: đầu ra của nó thành `source_term` ứng viên qua `glossary/scan.rs:257`.
- Không gấp `pairs_for_source`: FR58 điền sẵn chỉ cho câu y hệt.
- Không gấp `similarity_percent`, `diff_spans`, các khoá ghi `live_copies`, `update_copies_target`.
- Không đổi kết quả `MatchLang::En`, không migration, không thêm crate, không thêm command.

## I/O & Edge-Case Matrix

| Scenario | Input / State | Expected Output / Behavior | Error Handling |
|----------|--------------|---------------------------|----------------|
| Khớp mờ xuyên dạng | TM giản, câu mở phồn, cùng chữ | Hiện ở danh sách khớp mờ, 99% | Không lỗi |
| Điền sẵn | TM `大纪元…`, câu `大紀元…` | Không điền sẵn | Không lỗi |
| Gợi ý dải chốt | Mục `大纪元`, TM nguồn `大紀元` | Có gợi ý, đếm đúng số cặp | Không lỗi |
| Concordance | Tìm `大紀元`, TM nguồn `大纪元` | Có kết quả, kể cả ô tìm của màn quản lý TM | Không lỗi |
| Quick-add khác dạng | Glossary `大纪元`, bôi `大紀元` | Chế độ thêm, một dòng báo mục kia, Lưu được | Không lỗi |
| Quick-add trùng nguyên văn | Glossary `大纪元`, bôi `大纪元` | Chế độ sửa như cũ, không dòng báo | Không lỗi |
| Hai từ khác nhau | Glossary `发`, thêm `髮` | Lưu tạo mục riêng | Không lỗi |
| Lọc ứng viên | Glossary `大纪元`, quét ra `大紀元` | Bị lọc, tính vào `skipped` | Không lỗi |
| Tiếng Anh | Mọi đường trên với `En` | Như trước thay đổi | Không lỗi |

</frozen-after-approval>

## Code Map

- `src-tauri/src/core/matching/mod.rs` -- `ZH_FOLD` `:153`, `fold_zh` `:166`, `SimilarityScorer::units` nhánh Zh. `ngrams` không đổi.
- `src-tauri/src/core/tm/mod.rs` -- `target_suggestions_for_term` `:515`, `concordance_key` `:604` dùng cho `rank_concordance` và `rank_manage_listing`, `load_all_pair_rows` `:163`, `merge_tiers`, `police`.
- `src-tauri/src/core/glossary/store.rs` -- `resolve_term_for_quick_add` `:979` giữ nguyên, `filter_import_scan_candidates_by_scope` `:448`, `resolved_source_terms`.
- `src-tauri/src/commands/glossary.rs:125` -- `QuickAddLookup`, `glossary_lookup_term` `:142`.
- `src/config/glossary.ts:70` -- `GlossaryLookupResult`, `isQuickAddLookupWire`; `src/glossaryQuickAddState.ts`; `src/GlossaryQuickAdd.vue:216`; `src/i18n/vi.json:1672`.
- Canh hình dạng: `tests/ipc_contract.rs`, `tests/glossary_boundary.rs:181` (danh sách bề mặt Glossary), `tests/tm_boundary.rs`, `tests/frontend/glossaryQuickAdd.test.ts`, `tests/frontend/glossaryQuickAddStrip.test.ts`.
- `src-tauri/tests/tm_contract.rs:2226` -- test đối chứng, baseline đúng ở `b1f3b49bb52b2c9d6813a9e6f54ec846373882a0`.

## Tasks & Acceptance

**Execution:**
- [x] `src-tauri/src/core/matching/mod.rs` -- Đổi `ZH_FOLD` sang bảng chỉ số trực tiếp theo điểm mã BMP, thêm `fold_char`, phơi `fold_zh`; `units` nhánh Zh gấp từng ký tự -- đường khớp mờ quét cả TM cho từng câu nên không dùng băm.
- [x] `src-tauri/src/core/tm/mod.rs` -- `target_suggestions_for_term` đọc mọi hàng hai tầng, lọc `fold_zh(source) == fold_zh(term)`, giữ `merge_tiers` và `police`; `concordance_key` gấp sau bước hạ chữ -- đóng nợ TM.
- [x] `src-tauri/src/core/glossary/store.rs` -- Thêm `variant_term_for_quick_add` trả `source_term` đầu tiên có dạng gấp trùng mà không trùng nguyên văn; lọc ứng viên so trên tập đã gấp -- đóng nợ quick-add.
- [x] `src-tauri/src/commands/glossary.rs` -- `QuickAddLookup` thêm `variant_term: Option<String>`, chỉ có giá trị khi `entry` là `None` -- bề mặt cho dải.
- [x] `src/config/glossary.ts`, `src/glossaryQuickAddState.ts`, `src/GlossaryQuickAdd.vue`, `src/i18n/vi.json` -- Nhánh `none` mang `variantTerm`, guard kiểm kiểu, một dòng `role="status"` khoá `glossary.quick_add.variant_exists`: `Đã có dạng chữ khác: {term}. Lưu sẽ tạo thêm một mục riêng.` -- báo trước khi lưu.
- [x] `src-tauri/tests/tm_contract.rs` -- Trả test đối chứng về bản baseline, hàng `翻译`/`翻譯` là `true` -- AD-17 hết ngoại lệ.
- [x] `src-tauri/tests/tm_contract.rs`, `src-tauri/tests/glossary_contract.rs`, `src-tauri/tests/glossary_commands_contract.rs`, `src-tauri/tests/glossary_scan_contract.rs`, `tests/frontend/glossaryQuickAdd.test.ts`, `tests/frontend/glossaryQuickAddStrip.test.ts` -- Một ca cho mỗi hàng Matrix -- canh hành vi.
- [x] Spine -- `Binds` AD-63 thêm C5, hàng C5 thêm AD-63, Rule 5 ghi đường đã gấp và đường cố ý nguyên văn, bỏ ngoại lệ ở 🔵 AD-17 -- bất biến.
- [x] `_bmad-output/initiative-auratranslate/deferred-work.md` -- Đóng hai mục nợ của `plan-gap-phon-gian-matcher.md` bằng `→ ✅ ĐÃ ĐÓNG`; sửa bằng 🔵 câu nêu `ngrams`, `similarity_percent`, `PreparedQuery` -- ghi sai đường TM.

**Acceptance Criteria:**
- Given Glossary và TM cùng cặp biến thể, when chạy test đối chứng, then cả hai cùng bắt `翻译`/`翻譯`.
- Given gỡ thật bước gấp ở `units`, `concordance_key`, `target_suggestions_for_term` và lọc ứng viên, when chạy test của từng đường, then đúng ca của đường đó đỏ.
- Given TM 20.000 hàng tiếng Trung, when đo `fuzzy_pairs_in_candidates` trước và sau cùng bản dựng, then ghi số và mức chênh vào Implementation Notes.

## Implementation Notes

- `ZH_FOLD` nay là bảng 65.536 ô theo điểm mã BMP; `SimilarityScorer::units` qua `fold_char`, các đường chuỗi qua `fold_zh`.
- AC3 đo trên một bản `test` chưa tối ưu, 20.000 hàng tiếng Trung tổng hợp, 5 truy vấn: `fuzzy_pairs_in_candidates` ~368–404 ms mỗi truy vấn khi tắt gấp, ~395–400 ms khi bật, khoảng +6%, lượt lạnh đầu 474 ms.
- `target_suggestions_for_term` ban đầu đọc mọi hàng trên luồng chính dưới khoá `OpenWorkState`; nay `(async)` và SQL chỉ lấy hàng cùng độ dài byte, vì gấp giữ nguyên byte. Đo 20.000 hàng, tầng Global, bản `test`: 2,2–2,3 ms với thuật ngữ 3 chữ, 7,1 ms với 8 chữ, không có lượt trúng.
- AC2: gỡ thật ở `units`, `concordance_key`, `target_suggestions_for_term`, lọc ứng viên, đúng ca từng đường đỏ. Lượt gỡ `target_suggestions_for_term` chạy trên bản đọc mọi hàng, trước khi đổi loader; bộ lọc gấp không đổi, ca của nó xanh trên mã cuối.
- Ngoài danh sách plan: đổi `chinese_matches_raw` trong `tm_contract` sang kỳ vọng gấp, tách `翻译`/`翻譯` khỏi bộ so `Dice` với `ngrams` trong `matching_contract` vì `ngrams` cố ý không gấp, thêm ca `skipped` ở `commands/project/tests.rs`.
- Hàng Matrix `Tiếng Anh` do các ca `En` có sẵn canh, không thêm ca mới.
- Không chạy full suite và e2e: không đổi wiring dùng chung, `pre-push` chạy khi đẩy.

## Plan Change Log

## Review Triage Log

Vòng 1, lens `quick`: high 1 · medium 3 · low 5 · false 0 · maybe-false 0.

| Verdict | Route | Phát hiện | Bằng chứng / xử lý |
|---|---|---|---|
| high | patch | Quét cả TM trên luồng chính, giữ khoá | `glossary_tm_suggestions` là lệnh thường, hai lệnh TM cùng loại là `(async)`; đổi `(async)`, thêm `blocking_wire_cases`, lọc độ dài ở SQL |
| low | reject | AC3 chưa ghi số, ô công việc chưa tích | Sửa là sửa plan; số đã có và ghi ở Implementation Notes |
| medium | patch | Không ca nào chạm ô tìm quản lý TM | Thêm `師父` vào ca `rank_manage_listing` |
| medium | patch | Ca lọc ứng viên không qua chỗ đếm `skipped` | Thêm ca ở `commands/project/tests.rs`, đọc `(1, 1)` |
| medium | patch | Ca `发`/`髮` xanh trên mã cũ, không ca nào lưu từ trạng thái báo | Viết lại ca, thêm ca dải gọi add không gọi update |
| low | patch | Trường wire chưa khoá ở Rust | Guard TS từ chối thiếu trường; thêm `ipc_contract`, `QUICK_ADD_SURFACE` lên 13 |
| low | patch | Ba chú thích cũ | Thay ở `commands/glossary.rs`, `core/tm/mod.rs`, `core/matching/mod.rs` |
| low | patch | Rule 5 nêu sai hàm | AD-63 nêu cả `fold_zh` và `fold_char` |
| low | patch | Lượt dùng thật chưa có mục sổ nợ | Thêm mục `Chủ: Ice` |

## Design Notes

Đường thật của TM khác với sổ nợ cũ. `PreparedQuery` không tồn tại, đường khớp mờ là `SimilarityScorer`. `ngrams` nhánh Zh chỉ phục vụ quét ứng viên; gấp nó sẽ biến `家` thành `傢` trong `source_term`. `similarity_percent` chỉ gọi với `En` trên văn bản đích. Đại diện lớp là điểm mã nhỏ nhất nên không dạng đã gấp nào được hiện ra hay ghi xuống.

Câu chỉ khác dạng chữ dừng ở khớp mờ 99%, không điền sẵn: `diff_spans` vẫn tô `紀`/`纪` là chỗ khác, đúng với lý do nó không phải 100%, và người dùng tự nhận.

Lọc ứng viên theo dạng gấp là nhất quán: chữ của ứng viên bị loại đã được mục sẵn có tô và chèn. Thêm `髮` tay vẫn được, và mục đó thắng trên chữ `髮` nhờ luật nguyên văn của AD-63.

## Verification

**Commands:**
- `cargo test --test matching_contract --test tm_contract --test tm_boundary --test glossary_contract --test glossary_boundary --test glossary_commands_contract --test glossary_scan_contract --test ipc_contract --test ai_rag_contract` -- expected: xanh.
- `npx vitest run tests/frontend/glossaryQuickAdd.test.ts tests/frontend/glossaryQuickAddStrip.test.ts` -- expected: xanh.
- `npm run check:i18n` và `npm run check:tokens` -- expected: xanh.

**Manual checks (if no CLI):**
- Ice bôi `大紀元` trong Tác phẩm thật khi Glossary có `大纪元`: dải báo dòng dạng chữ khác và vẫn Lưu được.
