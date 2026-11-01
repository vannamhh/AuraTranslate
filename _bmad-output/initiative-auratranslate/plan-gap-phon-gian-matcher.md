---
title: 'Gấp phồn/giản trong Matcher để thuật ngữ khác dạng chữ vẫn khớp'
type: 'bugfix'
ticket: ''
created: '2026-11-01'
status: 'built'
baseline_revision: 'b1f3b49bb52b2c9d6813a9e6f54ec846373882a0'
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

**Problem:** Glossary có `大纪元` (giản thể), Tác phẩm viết `大紀元` (phồn thể): nhánh `Zh` của `find_terms` tìm chuỗi nguyên văn nên không có lượt khớp nào, prompt không nhận thuật ngữ, không báo lỗi. Đo trên 10 câu thật: dạng giản 0 lượt, dạng phồn 13 lượt.

**Approach:** Ice chốt 2026-11-01: gấp phồn/giản ngay trong nhánh `Zh` của `find_terms`, cả văn bản lẫn thuật ngữ, bằng một bảng biên dịch vào binary. FR51 đổi chữ, AD mới ghi quyết định.

## Boundaries & Constraints

**Always:**
- Ranh giới token jieba tính trên văn bản GỐC. Span trả về là byte vào chuỗi gốc.
- Bảng là hàm gấp theo lớp tương đương: mỗi ký tự về đại diện của lớp, nên tự lũy đẳng.
- Mọi cặp trong bảng cùng dài 3 byte UTF-8, nên văn bản đã gấp giữ nguyên offset byte.
- Cùng một span, mục khớp nguyên văn thắng mục chỉ khớp sau gấp.
- Prompt và ledger vẫn mang `source_term` của mục Glossary.

**Never:**
- Không phụ thuộc từ điển lúc chạy, không thêm crate, không đổi `MatchLang::En`.
- Không gấp trong TM (`ngrams`, `similarity_percent`, `PreparedQuery`), khoá `source_term`, quick-add, lọc ứng viên `store.rs:456`.
- `core::matching` vẫn là lá của AD-13.

## I/O & Edge-Case Matrix

| Scenario | Input / State | Expected Output / Behavior | Error Handling |
|----------|--------------|---------------------------|----------------|
| Giản khớp phồn | text `大紀元訊`, term `大纪元` | 1 lượt, `text[span]` = `大紀元` | Không lỗi |
| Phồn khớp giản | text `大纪元`, term `大紀元` | 1 lượt | Không lỗi |
| Hai dạng cùng có mặt | terms `大纪元`, `大紀元`, text phồn | Chỉ mục `大紀元` | Không lỗi |
| Lớp nhiều chữ | `委託`/`委托`, `裡`/`里`, `週`/`周` | Khớp hai chiều | Không lỗi |
| Offset | text mở đầu bằng `𠧜`, chữ Latin xen giữa | Span đúng byte gốc | Không lỗi |
| Ranh giới | term `發`, text `发现` (jieba một token) | 0 lượt | Không lỗi |
| Thuật ngữ rỗng/dấu tách | term chỉ gồm dấu | 0 lượt như cũ | Không lỗi |
| Nhánh En | text `大紀元`, term `大纪元`, `En` | Như trước thay đổi | Không lỗi |

</frozen-after-approval>

## Code Map

- `src-tauri/src/core/matching/mod.rs:668` -- `find_terms`, nhánh `Zh` `:672`; `warm` `:148`; `HMM` `:120`. Sửa doc của nhánh Zh `:625`.
- `src-tauri/src/core/matching/zh_fold.txt` -- mới, dữ liệu sinh ra, `include_str!`.
- `scripts/gen-zh-fold.mjs` -- mới, sinh bảng từ `dict-core.db`, gọi CLI `sqlite3`.
- `src-tauri/src/core/glossary/store.rs:1534` -- `resolve_overlaps`, không đổi.
- Người gọi, không đổi: `glossary/store.rs:1592`, `:1883`, `glossary/candidate_store.rs:185`, `export/harvest.rs:159`.
- `GlossaryMark.source_term` đã chấp nhận khoá khác bề mặt, `glossaryMarksMap.ts` chỉ dùng `start`/`end`.
- `src-tauri/tauri.conf.json:42-43` -- `CC-BY-SA-4.0.txt` và `Unicode-License-v3.txt` đã vào gói qua `tools/dict-build/assets/licenses/`, không đổi.
- `tests/matching_contract.rs`, `tests/ai_rag_contract.rs`, `tests/glossary_marks_contract.rs`.
- `architecture-auratranslate.md` AD-17 `:238`; PRD FR51 `:602`; `addendum.md:114`.

## Tasks & Acceptance

**Execution:**
- [ ] `scripts/gen-zh-fold.mjs` -- Lấy cặp một ký tự `headword`≠`headword_simp` của `unihan`, `cc-cedict`, `cvdict`, giữ U+3400..U+FFFF, hợp lớp, đại diện là điểm mã nhỏ nhất; ghi sắp xếp, dòng đầu `#` nêu nguồn, giấy phép, cách sinh lại -- tái lập được.
- [ ] `src-tauri/src/core/matching/zh_fold.txt` -- Sinh bằng script trên -- dữ liệu, không sửa tay.
- [ ] `src-tauri/src/core/matching/mod.rs` -- Thêm gấp, nạp bảng trong `warm`; nhánh Zh `find` trên văn bản đã gấp, lọc ranh giới gốc, lọc mục nguyên văn thắng -- sửa lỗi.
- [ ] `src-tauri/tests/matching_contract.rs` -- Ca I/O Matrix; bất biến bảng: 3 byte, lũy đẳng, sàn số mục -- canh hành vi.
- [ ] `src-tauri/tests/ai_rag_contract.rs`, `src-tauri/tests/glossary_marks_contract.rs` -- Mục giản + câu phồn: ledger `injected` 1 mục đúng span, lưới 1 dấu -- canh đường dùng thật.
- [ ] Spine, PRD, `addendum.md` -- Winston soạn AD-63 (số trống sau khi quét spine và mọi `ad-brief-*`), ghi dòng Stack cho bảng; sửa FR51 và hàng tiếng Trung của addendum -- đổi bất biến.
- [ ] `NOTICE.txt` -- Nêu `zh_fold.txt` là dữ liệu CC BY-SA 4.0 và Unicode-3.0 nằm trong binary, vì tệp này đang nói dữ liệu từ điển chỉ ở `dict/` -- giấy phép.
- [ ] `_bmad-output/initiative-auratranslate/deferred-work.md` -- Hai mục `Chủ: Ice`: TM chưa gấp (trái `Prevents` của AD-17 cho tới khi quyết); quick-add và lọc ứng viên vẫn khoá nguyên văn -- nợ có chủ.

**Acceptance Criteria:**
- Given Glossary toàn cục có `大纪元` và Work thật ở `Verification`, when chạy probe trên 10 câu, then đọc 13 lượt.
- Given một Tác phẩm phồn thể, when mở Chương và lắp prompt, then `Đã chèn` đếm mục đó và lưới tô đúng chữ phồn.
- Given mọi test Matcher và Glossary có sẵn, when chạy lại, then không ca nào đổi kỳ vọng.
- Given gỡ thật lời gọi gấp, when chạy lại `matching_contract`, then các ca gấp đỏ vì 0 lượt.
- Given đo `find_terms` trên toàn bộ văn bản Work thật cùng bản dựng, when so trước và sau, then chênh dưới NFR2 và số đo ghi rõ bản dựng.

## Implementation Notes

- Các `TermMatch` cùng span không còn bị lọc trong `find_terms`: phân xử nằm ở `resolve_overlaps` (nguyên văn, rồi Work, rồi `term_index`), chỉ áp cho `Zh`, nên `suppressed_by_pending_overlap` vẫn thấy mục đã chốt bị che.
- AC1 đo trên 10 câu thật của `project.db`: `大纪元` đọc 13 lượt trên mã cuối, 0 lượt trên baseline.
- AC3 không đạt nguyên văn: `tm_contract` có một hàng đổi kỳ vọng (`翻译`/`翻譯`, Glossary `true`, TM `false`), hệ quả trực tiếp của quyết định không gấp TM; test đổi tên cho đúng thứ nó canh, Ice quyết ở mục nợ TM.
- AC4: gỡ thân `fold_zh` làm 5 ca `matching_contract` đỏ vì 0 lượt; gỡ tie-break tầng làm đỏ đúng ca Work; gỡ chặn `En` làm đỏ đúng ca `En`; đổi ranh giới sang văn bản đã gấp làm đỏ ca `国家`.
- AC5 đo A/B bản `test` chưa tối ưu, số ở dòng bằng chứng của AD-63.
- AD-63 do agent triển khai viết rồi sửa ở bước review, không qua Winston; hai câu sửa tay là `Binds` (C4, C6, C8) và 🔵 ở AD-17.
- `npm run build`, vitest, e2e không chạy: không đổi mã webview; `pre-push` chạy chúng khi đẩy.

## Plan Change Log

## Review Triage Log

Vòng 1, lens `quick`: high 1 · medium 4 · low 3 · false 0 · maybe-false 0 · tự soát thêm 1 (medium).

| Verdict | Route | Phát hiện | Bằng chứng / xử lý |
|---|---|---|---|
| high | patch | Mục đã chốt chỉ khớp sau gấp biến mất khi thua mục chờ chốt nguyên văn | Lọc trong `find_terms` loại nó khỏi `raw_matches` (`store.rs:1934`); chuyển phân xử sang `resolve_overlaps`, thêm ca `ai_rag_contract` |
| low | reject | AC3 bị vi phạm ở `tm_contract` | Hệ quả của `Never` đã đông băng; sửa là sửa plan, ghi ở Implementation Notes và báo Ice |
| medium | patch | Ca ranh giới không phân biệt được văn bản gốc và đã gấp | `发现` tự gấp về chính nó; thêm `国家`, đổi tokenize đọc đỏ |
| medium | patch | `target_suggestions_for_term` khớp nguyên văn, dải chốt rỗng | `core/tm/mod.rs:513`; thêm vào mục nợ TM |
| medium | patch | Mục Work thua Global cùng span | Đọc `裏`/`里`/`裡` cho Global; thêm tie-break tầng, phần nguyên văn ghi ở giới hạn AD-63 |
| low | patch | Chú thích cũ ở `warm`, `store.rs:1673` | `warm` đã sửa; `store.rs:1673` đúng lại sau khi bỏ lọc |
| medium | patch | Bảng không ghi dữ liệu nguồn | Header nay mang `source_version` ba nguồn |
| low | reject | AC1, AC4 chưa có bằng chứng | Sửa là sửa plan; đo lại và ghi ở Implementation Notes |
| medium | patch | Tự soát: thứ tự phân xử đổi kết quả `En` | Trái `Never`; chặn bằng `lang == Zh`, gỡ chặn làm đỏ ca `En` |

## Design Notes

Nguồn bảng: chỉ Unihan thì thiếu `週 祕 佈 遊 臥 託 佔 僱 兇` (829 cặp CC-CEDICT không nối được). Ba nguồn trong `dict-core.db` cho 6.835 ký tự, 3.179 lớp, lớn nhất 8 chữ, phủ cả 19 cặp đã soát, tệp Unihan gốc chỉ thêm 4 chữ hiếm. Hai giấy phép đã nằm trong gói, nên không còn lựa chọn đáng cân.

Gấp theo lớp thay cho hướng phồn→giản vì 23 khoá phồn có nhiều dạng giản, `託` sẽ về `讬` và không gặp `托`. Lớp giải cả ba: mơ hồ, chuỗi `薴→苧→苎`, lũy đẳng.

Khớp thừa là giá đã nhận: mục một chữ `后` sẽ khớp `後`. Thấy được trên lưới, khác hẳn thiếu thuật ngữ không ai biết.

## Verification

**Commands:**
- `node scripts/gen-zh-fold.mjs` -- expected: chạy lại không tạo diff.
- `npm run build` rồi `cargo test --test matching_contract --test matching_boundary --test ai_rag_contract --test glossary_marks_contract --test ai_boundary` -- expected: xanh.
- `cargo test --test glossary_contract --test glossary_scan_contract --test glossary_exchange_contract --test tm_contract` -- expected: xanh.
- Test tạm đọc 10 câu thật từ `project.db` -- expected: 13 lượt, gỡ sau khi đo.

**Manual checks (if no CLI):**
- Ice mở Tác phẩm thật, bấm dịch một câu có `大紀元`: dòng tóm tắt báo `Đã chèn 1 thuật ngữ Glossary`.
