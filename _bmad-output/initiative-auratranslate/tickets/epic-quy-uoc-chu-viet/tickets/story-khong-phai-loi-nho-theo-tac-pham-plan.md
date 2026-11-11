---
title: '"Không phải lỗi" nhớ theo Tác phẩm'
type: 'feature'
ticket: '5'
created: '2026-11-10'
status: 'built'
baseline_revision: 'baad0af5cefadf3b9ce83f19432d77fa22a62e99'
route: 'full'
route_source: 'auto'
risk: 'high'
review: 'quick'
review_source: 'pinned'
lenses_ran: ['quick']
review_loop_iteration: 0
context:
  - '{project-root}/_bmad-output/initiative-auratranslate/architecture-auratranslate/architecture-auratranslate.md §AD-67 mục 6, 9, 12 (i), §AD-53 mục 7'
  - '{project-root}/_bmad-output/initiative-auratranslate/ux-auratranslate/EXPERIENCE.md §Quy ước chữ Việt — FR159 (thẻ Không phải lỗi, Khoá i18n mới, Interaction Primitives)'
  - '{project-root}/_bmad-output/initiative-auratranslate/ux-auratranslate/mockups/quy-uoc-chu-viet.html (hàng 4 và 4b)'
---

<frozen-after-approval reason="human-owned intent — do not modify unless human renegotiates">

## Intent

**Problem:** Quy ước bắt đúng cụm nhưng không bắt đúng chỗ: một cụm hợp lệ trong văn cảnh này (*bảo **đảm bảo** vệ*) bị đánh dấu lại ở mọi lượt quét, mọi lần mở Chương. Dải đã có nút *Bỏ qua* nhưng nó chỉ đóng dấu trong phiên; `ignored` trên dây vẫn là hằng 0 từ 16.1 tới 16.4, nên R8 chưa có đường nào.

**Approach:** Thêm thao tác thứ hai *Không phải lỗi* bên cạnh *Bỏ qua*: một bảng `convention_ignore` trong `project.db`, chữ ký (khoá luật, ngữ cảnh trước, ngữ cảnh sau) theo AD-67 mục 9, lọc ngay trong bộ quét trước khi kết quả ra dây và đếm vào `ignored`. Xem và gỡ trong thẻ **Không phải lỗi** của lớp phủ Quản lý quy ước.

## Boundaries & Constraints

**Always:**
- Chữ ký **không** mang `segment.id` và không mang vị trí (AD-67 mục 9). Khoá duy nhất là `(rule_key, context_before, context_after)`.
- Hàm chữ ký và hàm lọc là một cặp hàm thuần trong `core/convention/`, dùng chung cho lệnh ghi nhớ và cho cả ba đường quét (Chương, theo câu sau flush, gợi ý AI).
- Phép so đầu/đuôi của AD-53 mục 7 nguyên văn: phía đã lưu rỗng chỉ khớp phía hiện tại cũng rỗng; phía không rỗng khớp khi là đuôi (trước) hoặc đầu (sau) của phía hiện tại.
- `ignored` phải đếm đúng mọi chỗ bị lọc (AD-67 mục 6) — "0 chỗ khớp" không được im lặng thay cho một chỗ bị ghi nhớ.
- Lệnh ghi nhớ đi qua đúng hàm kiểm dùng chung của AD-53 mục 4 (`check_proofread_range`), như `convention_accept` đã đi.
- Phạm vi Tác phẩm là **cấu trúc** (bảng nằm trong `project.db`), không phải một cột lọc.
- `core/convention/` không import `core::ai` và ngược lại (AD-67 mục 1).

**Never:**
- Không chạm `proof_ignore`, `proof_scan`, `proof_decision`, `PROOFREAD_MEMORY_DDL`, tập loại của AD-53, hay `ALLOWED_AI_PATHS_IN_PROOFREAD_SEAM`.
- Không đổi nghĩa nút *Bỏ qua*: nó vẫn không ghi nhớ, dấu vẫn ở đó, dải sang chỗ kế (EXPERIENCE §Dải quy ước).
- Không bảng thứ hai, không cột cờ trên `convention_rule`, không ghi nhớ ở tầng Toàn cục.
- Không thêm `blocked_count` hay tỉ lệ báo động giả (đó là FR150 của Proofreader).
- Không `GLOBAL_MIGRATIONS`; không sửa một hằng DDL đã chạy.
- Ngoài story: xuất nhập (16.6).
- 🔵 2026-11-11, Ice: *"sửa luôn đi, tôi không muốn đưa vào mục nợ nào cả"* — dòng này trước đây để **e2e mới** ra ngoài story vì Done when 2 của epic đã do 16.1/16.2 phủ. Nay story mang luôn `e2e/specs/story-16-5-convention-not-an-error.e2e.mjs`, và mọi mục nợ story này mở ra phải đóng trong chính story này.

## I/O & Edge-Case Matrix

| Scenario | Input / State | Expected Output / Behavior | Error Handling |
|----------|--------------|---------------------------|----------------|
| Ghi nhớ rồi quét lại | Luật `đảm bảo`, câu `nhằm bảo đảm bảo vệ`, người dùng bấm *Không phải lỗi* ở chỗ đó | Một hàng `convention_ignore`; lượt quét kế không trả chỗ đó, `ignored` = 1 | — |
| Sống qua đóng mở Tác phẩm | Đã ghi nhớ, đóng Tác phẩm, mở lại | Vẫn không có dấu ở chỗ đó, `ignored` = 1 | — |
| Cùng cụm, câu khác | Đã ghi nhớ ở câu A; câu B chứa `đảm bảo` trong ngữ cảnh khác | Câu B vẫn có dấu | — |
| Phạm vi Tác phẩm | Đã ghi nhớ ở Tác phẩm A; Tác phẩm B có cùng câu | B vẫn có dấu, `ignored` = 0 ở B | — |
| Gộp segment | Ghi nhớ ở một câu rồi gộp nó với câu trước | Ghi nhớ còn khớp (ngữ cảnh chỉ dài ra) | — |
| Cụm sát biên segment | Chỗ khớp ở đầu câu (ngữ cảnh trước rỗng) rồi gộp | Báo lại — phía rỗng chỉ khớp phía rỗng | — |
| Ghi nhớ trên gợi ý AI | Chỗ đã ghi nhớ xuất hiện trong gợi ý ở panel AI Translation | Không có dấu trên gợi ý, `ignored` của wire gợi ý = 1 | — |
| Văn bản đã đổi | Lệnh ghi nhớ mang `expected_text` khác `target_text` | Không ghi hàng nào | `err.convention.text_changed` |
| Khoảng sai | `start`/`end` ngoài chuỗi, rỗng, hay cắt đôi cặp surrogate | Không ghi hàng nào | `err.convention.range_invalid` |
| Chưa mở Tác phẩm | Gọi ghi nhớ / liệt kê / gỡ khi không có Tác phẩm mở | Không ghi gì; thẻ nói cần mở một Tác phẩm | `err.convention.work_tier_unavailable` |
| Ghi nhớ lại đúng chỗ đã ghi nhớ | Cùng chữ ký | Vẫn một hàng (`INSERT OR IGNORE`), không lỗi | — |
| Gỡ ghi nhớ | Bấm *Bỏ ghi nhớ* trên một hàng | Hàng mất; lượt quét kế đánh dấu lại chỗ đó | Hàng đã mất ⇒ trả `false`, không lỗi |
| Xoá toàn bộ | Bấm *Xoá toàn bộ ghi nhớ* hai bước | Mọi hàng của Tác phẩm mất, trả số đã xoá | — |
| Quét không ghi | Bất kỳ lượt quét nào | `convention_ignore` không đổi hàng nào | — |

</frozen-after-approval>

## Code Map

Rust (`src-tauri/src/`):
- `core/store/schema.rs` -- `PROJECT_MIGRATIONS` :2346, bước mới nhất 44 là `CONVENTION_RULE_DDL` :893. Thêm **bước 45** với hằng riêng `CONVENTION_IGNORE_DDL` (một bảng, `UNIQUE (rule_key, context_before, context_after)`, `CHECK (avoid_phrase <> '')`); khuôn để chép là `PROOFREAD_MEMORY_DDL` :1111 (`proof_ignore`). Câu đầu khối tài liệu của `PROJECT_MIGRATIONS` ở :2176 ghi *"đích là phiên bản 44"* — đổi số và thêm khối `🔵 CẬP NHẬT`. **Không** chạm `GLOBAL_MIGRATIONS` (:768) và không chạm hằng đã chạy.
- `core/convention/ignore.rs` (**mới**) -- `CONVENTION_CONTEXT_CHARS`, `ConventionSignature`, `signature_of(text, start_utf16, end_utf16, rule_key)`, `signature_matches(stored, current)`, `filter_ignored(...)`. Khuôn nguyên văn: `core/ai/proofread.rs` :181 hằng N, :186 struct, :194 `signature_of`, :231 `signature_matches`, :252 `filter_remembered`. Khai báo trong `core/convention/mod.rs` :11-21 (đó là mặt công khai duy nhất của module).
- `core/convention/normalize.rs` :19 `normalize(input) -> Normalized { text, chars: Vec<CharOrigin{byte, utf16_start, utf16_end}> }` -- NFC + chữ thường theo cụm grapheme, kèm bản đồ UTF-16. Dùng **một lần cho mỗi câu** để lấy ngữ cảnh (xem Design Notes). `core/segment/utf16.rs` :5 `utf16_to_byte`.
- `core/convention/store.rs` -- khuôn hàm kho (`list_rules` :105, `add_rule` :135, `find_row_by_key` :199, `delete_rule` :247; `store_for_tier` :102 và biến thể `WorkTierUnavailable` của `ConventionStoreError`). Thêm `add_ignore`, `list_ignores`, `list_ignore_rows`, `remove_ignore`, `clear_ignores`, tất cả nhận `&Store` của Tác phẩm.
- `commands/convention.rs` -- `resolve_spans` :352-391 là **chỗ lọc duy nhất**, dùng chung cho cả ba đường quét: chèn phép lọc ngay sau nhánh `continue` của Glossary (:369-372) và trước `spans.push` (:378); đổi kiểu trả về để mang thêm số bị lọc. `match_sentences` :396-448 (`ignored` hằng 0 ở :443), đường gợi ý :587 và :655. Đọc tập chữ ký **một lần mỗi lượt quét** trong đúng khối `lock` đã có (:470-483 Chương, :511-524 theo câu, :572-597 gợi ý), như `work_rows(open)` :38-43, rồi mang ra ngoài khoá trong `ScanInput` :265. `ConventionScanWire` :241 và `ConventionSuggestionScanWire` :552 đã có `ignored: u32` — **không đổi dây**. `pub mod wire` :669 là khuôn vỏ lệnh (`try_state`, `lock().unwrap_or_else(PoisonError::into_inner)`, không bao giờ `state()`). Bốn lệnh mới `convention_ignore_add|list|remove|clear` nằm ở đây, **đồng bộ** (không `async`).
- `commands/segment/proofread_accept.rs` :61 `check_proofread_range` -- hàm kiểm dùng chung của AD-53 mục 4, chạy trong giao dịch ghi; `commands/segment/convention_write.rs` :139 và :370 là hai chỗ đã gọi nó, và :31-48 là chỗ ánh xạ lỗi Proofreader sang `MessageKey::Convention*`. Chép đúng phép ánh xạ đó, **không** thêm khoá lỗi mới.
- `commands/chapter.rs` `no_work_open` -- rào khi chưa mở Tác phẩm.
- `lib.rs` :1176-1184 danh sách `generate_handler!` của convention -- thêm bốn lệnh. `core/i18n/mod.rs` :944-954 đã có `err.convention.text_changed|range_invalid|work_tier_unavailable`; không khoá lỗi mới.

Webview (`src/`):
- `ProofreaderStrip.vue` :285-287 nút *Bỏ qua* (`data-convention-skip`, `dispatch('convention.skip')`) -- **giữ nguyên**; thêm nút *Không phải lỗi* ngay sau nó theo khuôn nút *Nhận thay* :276-279 (`:disabled="conventionBusy"`, `@mousedown.prevent`).
- `conventionState.ts` :600 `skipConventionSpot` -- **giữ nguyên**. Thêm `ignoreConventionSpot()`: gọi adapter, rồi bỏ **một** span khỏi `results` theo khuôn sao-chép-Map của `clearConventionFor` :187, chỉnh `opened.index` như `acceptConventionSpot` :372, rồi `stepMarks(1)` :557. `conventionCurrent` :219 cho `{segmentId, index, span, scannedText}`; `say(key, params)` cho `convention.status.ignored`.
- `config/convention.ts` :82-93 hằng `CMD_*`, :252-307 khuôn adapter "Never throws" -- thêm `CMD_IGNORE_ADD|LIST|REMOVE|CLEAR` và bốn adapter; khuôn kiểu hàng và trả về: `config/proofread.ts` :307-352 + `ProofreadIgnoreRowWire`.
- `conventionManageState.ts` (334 dòng) -- nơi giữ state của thẻ; `resetConventionManage` :184 đã được gọi từ `modes/libraryImport.ts` :488 và `modes/libraryChapters.ts` :390, nên **không thêm chỗ reset mới**. Khuôn ref và hàm: `proofreadMemoryState.ts` (128 dòng) — `rows`, `hasLoaded`, `busy`, `failure`, `selectedId`, `clearPending`, `cleared`, biến `session` chống lượt nạp cũ.
- `ConventionIgnoreCard.vue` (**mới**, ~150 dòng) + thanh thẻ trong `ConventionManageOverlay.vue` (676 dòng, hiện **chưa có thẻ nào**; khung lớp phủ ở :217, thanh công cụ :229). Khuôn hàng và nút: `ProofreadMemoryOverlay.vue` :160-202 — hàng chọn bằng `@pointerdown`/`@focus`, nút dispatch một lệnh.
- `commands/index.ts` :5388-5531 họ `convention.*` (`keys: undefined` trừ `convention.manage.open` :5432) -- thêm `convention.ignore`, `convention.manage.tab_rules`, `convention.manage.tab_ignored`, `convention.manage.ignored_forget`, `convention.manage.ignored_clear`; cổng khai ở :1203-1221, nối trong `conventionHandlers.ts` (132 dòng, rải ở `main.ts` :1133).
- `i18n/vi.json` :134+ -- khoá của EXPERIENCE §Khoá i18n mới mà story này dùng: `convention.strip.ignore`, `convention.status.ignored`, `convention.manage.ignored_empty|needs_work|context|forget|forget_done|clear|clear_confirm|clear_hint|clear_done`, và nhãn `command.convention.ignore` + bốn nhãn lệnh của thẻ.

Tests / cổng:
- `convention_contract.rs` (1462 dòng) -- khuôn đếm: :1336 `a_rescan_of_pinned_glossary_text_suppresses_the_match_and_counts_it`, :1404 vế gợi ý, :457 `a_scan_writes_nothing`, :1349 ngang giá Chương/theo câu, :961 `second_work` + :973 `scan_in`. **Chưa có `reopen`** trong file này; khuôn ở `proofread_decision_contract.rs` :118.
- `convention_boundary.rs` :11 `CONVENTION_FLOOR = 7` -- thêm `ignore.rs` ⇒ 8; :13 `SRC_RS_FLOOR = 133` có thể phải tăng.
- `project_contract.rs` :2079 `NON_ENTITY_DETAIL_TABLES: [&str; 23]` -- thêm `convention_ignore` kèm lý do, bump 23 ⇒ 24.
- `segment_contract.rs` :798 câu đầu khối tài liệu, :941 thang bậc `-> 44 (4 la so da chay)` và `vec![…, 44]`.
- `pinned_contract.rs` :269 câu văn liệt kê từng bậc (`44 them convention_rule`).
- Cốt rollback phải thêm `DROP TABLE convention_ignore` và số 45: `lifecycle_contract.rs` :963, `proofread_decision_contract.rs` :285, `prompt_set_contract.rs` :998, `prompt_set_kind_migration_contract.rs` :104 và :162, `tm_contract.rs` :778, :817, :1220, :1249.
- `config_invariants.rs` :2249-2265 hàng `src/commands/convention.rs` đang là `(file, 5, 4, 4, …)` -- bump theo bốn lệnh đồng bộ mới (đọc định nghĩa test để biết nghĩa từng cột trước khi sửa); `blocking_wire_cases()` :1449 **không** thêm hàng (bốn lệnh mới là đồng bộ).
- `ipc_contract.rs` -- khoá tên dây và tên tham số; `ipc_argument_contract.rs` :630 `NO_FRONTEND_CALLER` **không** thêm tên (mọi lệnh mới đều có `invoke` ở webview).
- `story_16_1_measure_convention.rs` (218 dòng, `#[ignore]`, `bench-release`) -- khuôn cho `story_16_5_measure_convention_ignore.rs`.
- vitest — tests that move: `tests/frontend/conventionState.test.ts` (701 dòng), `tests/frontend/conventionManage.test.ts`, `tests/frontend/conventionStripIgnore.test.ts` (mới), `tests/frontend/conventionIgnoreCard.test.ts` (mới), `tests/frontend/libraryImportResetsConventionManage.test.ts`. Khuôn để chép: `proofreadMemory{,Content}.test.ts` cho thẻ, `proofreaderStripContent.test.ts` và `proofreaderStripSlots.test.ts` cho dải. **Chưa có spec nào phủ `Bỏ qua`** — ca đầu tiên phải khẳng định *Bỏ qua* vẫn không gọi Rust.
- 🔵 2026-11-10: mọi spec mock `config/convention` **và** nạp `conventionManageState` phải khai bốn adapter mới, không thì `load()` ném — hai tệp `conventionManage` và `libraryImportResetsConventionManage` ở trên.
- 🔵 2026-11-10: hai tệp cốt mà điều tra lúc lập plan bỏ sót, cũng khẳng định phiên bản project: `glossary_unconfirm_contract.rs` và `review_harvest_contract.rs` (tệp sau đếm bằng `PROJECT_MIGRATIONS.len() - 14`).

## Tasks & Acceptance

**Execution:**
- [ ] `core/store/schema.rs` -- hằng `CONVENTION_IGNORE_DDL` + bước 45 của `PROJECT_MIGRATIONS` + câu đầu khối tài liệu (`đích là phiên bản 45`, khối `🔵 CẬP NHẬT`) -- bảng mới phải tới được Tác phẩm đang có.
- [ ] `core/convention/ignore.rs` + `mod.rs` -- `CONVENTION_CONTEXT_CHARS = 32`, chữ ký, phép khớp, phép lọc; hàm thuần, không I/O, không `core::ai` -- AD-67 mục 9 đòi một cặp hàm thuần dùng chung.
- [ ] `core/convention/store.rs` -- `add_ignore` (`INSERT OR IGNORE`), `list_ignores` (chữ ký cho bộ quét), `list_ignore_rows` (hàng cho thẻ, `ORDER BY created_at DESC, id DESC`), `remove_ignore`, `clear_ignores` -- cùng khuôn `store.read`/`store.write` của luật.
- [ ] `commands/convention.rs` -- bốn lệnh đồng bộ + vỏ `wire`; đọc tập chữ ký một lần mỗi lượt quét trong khối `lock` đã có và mang vào `ScanInput`; lọc trong `resolve_spans` và đếm vào `ignored` ở cả ba đường -- một chỗ lọc, ba đường quét.
- [ ] `lib.rs` + `config_invariants.rs` + `ipc_contract.rs` -- đăng ký và bump bảng đếm lệnh -- cổng đếm theo tệp.
- [ ] cốt rollback và thang bậc di trú -- `segment_contract.rs`, `pinned_contract.rs`, `project_contract.rs`, `lifecycle_contract.rs`, `proofread_decision_contract.rs`, `prompt_set_contract.rs`, `prompt_set_kind_migration_contract.rs`, `tm_contract.rs` -- bước 45 phải tháo lui được ở mọi cốt đang mô phỏng kho cũ.
- [ ] `src/config/convention.ts` + `src/conventionState.ts` -- bốn adapter và `ignoreConventionSpot` (bỏ một span, chỉnh `opened.index`, sang chỗ kế, `say('convention.status.ignored')`); `skipConventionSpot` không đổi một dòng.
- [ ] `src/ProofreaderStrip.vue` + `src/commands/index.ts` + `src/conventionHandlers.ts` -- nút *Không phải lỗi* và năm lệnh mới (`keys: undefined`), `@mousedown.prevent` -- WebKit không trao tiêu điểm cho nút trong lưới.
- [ ] `src/conventionManageState.ts` + `src/ConventionIgnoreCard.vue` + `src/ConventionManageOverlay.vue` -- thanh thẻ *Luật* / *Không phải lỗi {n}*, danh sách hàng `…{context_before}[{phrase}]{context_after}…`, *Bỏ ghi nhớ* mỗi hàng, *Xoá toàn bộ ghi nhớ* hai bước, trạng thái rỗng và trạng thái chưa mở Tác phẩm -- EXPERIENCE §thẻ Không phải lỗi.
- [ ] `src/i18n/vi.json` -- khoá của story này, giọng vô ngã, placeholder khớp -- `check:i18n` và `check:commands`.
- [ ] `src-tauri/tests/convention_contract.rs` -- mọi hàng của I/O Matrix, kèm `reopen` (khuôn `proofread_decision_contract.rs` :118), ngang giá ba đường quét, và *lượt quét không ghi `convention_ignore`* -- AD-67 mục 12 (i).
- [ ] `src-tauri/tests/convention_boundary.rs` -- bump `CONVENTION_FLOOR`, giữ hai chiều của mục 12 (h) -- `ignore.rs` là tệp thứ tám.
- [ ] `src-tauri/tests/story_16_5_measure_convention_ignore.rs` -- `#[ignore]`, `bench-release`: 2.000 luật, 9.850 câu, 2.000 hàng ghi nhớ; quét cả Chương p95 ≤ 250 ms -- phép lọc nằm trên đường nóng (AD-67 mục 11).
- [ ] vitest -- `Bỏ qua` không gọi Rust; `Không phải lỗi` gọi đúng một lần, bỏ đúng một span và sang chỗ kế; thẻ nạp, gỡ một hàng, xoá toàn bộ hai bước, trạng thái chưa mở Tác phẩm.
- [ ] đối chứng đỏ cho từng cổng mới -- gỡ thật phép lọc, gỡ thật phép so phía rỗng, gỡ thật số `ignored`, gỡ thật `check_proofread_range` trong lệnh ghi nhớ -- mỗi phép gỡ là dời hoặc xoá, không phải chú thích.
- [ ] `deferred-work.md` -- mục cho mọi phát hiện review bị hoãn, `Chủ: Story 16.7`.

**Acceptance Criteria:**
- Given một luật và một câu có chỗ khớp, when người dùng bấm *Không phải lỗi* rồi đóng và mở lại Tác phẩm, then lượt quét không trả chỗ đó và `ignored` bằng 1; cùng cụm trong ngữ cảnh khác vẫn được trả.
- Given một chỗ đã ghi nhớ ở Tác phẩm A, when mở Tác phẩm B có cùng câu, then B vẫn có dấu và `ignored` của B bằng 0.
- Given một chỗ đã ghi nhớ, when nó xuất hiện trong một gợi ý đã trọn ở panel AI Translation, then gợi ý không có dấu ở chỗ đó và `ignored` của wire gợi ý bằng 1.
- Given thẻ *Không phải lỗi* đang mở với ba hàng, when người dùng bấm *Bỏ ghi nhớ* trên một hàng, then hàng mất, không có lượt hỏi, và lượt quét kế đánh dấu lại chỗ đó.
- Given chưa mở Tác phẩm nào, when người dùng mở thẻ *Không phải lỗi*, then thẻ nói rõ cần mở một Tác phẩm và không lệnh nào ghi gì.
- Given nút *Bỏ qua*, when người dùng bấm nó, then không có lời gọi Rust nào và dấu vẫn ở đó.
- Given bất kỳ lượt quét nào, when nó trả về, then `convention_ignore` không đổi hàng nào.
- Given một cổng mới đã xanh, when seam của nó bị gỡ thật khỏi mã sản xuất (dời hoặc xoá, giữ chữ ký lời gọi), then đúng mục tiêu test của cổng đó đỏ, và đỏ vì lý do đang đo.

## Implementation Notes

- Hai dòng của Design Notes đánh nhau: "ngữ cảnh không hạ chữ thường" và "lấy bản đồ từ `normalize()`" (hàm đó hạ chữ thường). Giữ vế không-hạ-chữ-thường, dựng `ContextIndex` riêng trong `core/convention/ignore.rs`; dòng Design Notes đã sửa tại chỗ bằng 🔵.
- `add_ignore` nhận **giao dịch của người gọi** chứ không nhận `&Store`, để lượt ghi nhớ nằm trong đúng giao dịch đã chạy `check_proofread_range` — phép kiểm và lượt ghi không thể lệch nhau.
- `convention_ignore_add` thêm một rào không có trong plan: `rule_key(expected_phrase)` phải bằng `rule_key` webview gửi, không thì `convention.rule_unavailable`. Nó bắt webview gửi chéo khoá luật và cụm.
- Thứ tự lọc là Glossary trước, ghi nhớ sau. Một chỗ vừa nằm trong bản dịch Glossary đã chốt vừa được ghi nhớ chỉ đếm vào `glossary_suppressed`.
- Vỏ `wire` của bốn lệnh mới trả `convention.work_tier_unavailable` khi không có `OpenWorkState`, không trả `state_missing()` như các vỏ convention khác — "chưa mở Tác phẩm" là câu trả lời thật.
- `CONVENTION_FLOOR` 7 ⇒ 8 làm ngay ở cuối phase Rust, không để sang phase Tests: nó chặn mọi lượt chạy `cargo test` còn lại.
- Bẫy cốt test mà điều tra lúc lập plan không thấy: `glossary_unconfirm_contract` và `review_harvest_contract` cũng khẳng định phiên bản project (`review_harvest` dùng `PROJECT_MIGRATIONS.len() - 14`), và `prompt_set_kind_migration_contract` :104 là kho **Global** nên không được chạm. Lượt `cargo test` đủ lôi ra hai chỗ nữa mà không phép đếm nào bắt được: `chapter_origin_contract` :1152 và `segment_role_contract` :870 khoá cứng đích 44. `pinned_contract` :263 thì **không** sửa — `PROJECT_MIGRATIONS.len()` là 44 sau bước 45, vì số 4 bị bỏ trống.
- Phép lọc duyệt **toàn bộ** bộ nhớ cho mỗi chỗ khớp (493 × 2.000 phép so). Một phép chia theo khoá luật (`ORDER BY rule_key` + `partition_point` + `debug_assert!`) đã được viết rồi **gỡ lại**: `debug_assert!` thứ tự KHÔNG ca nào đỏ được (mọi `remember_second` dùng một khoá), và phép đo cho thấy nó không mua được gì — xem dòng số đo dưới. Giữ máy móc chưa đo và không canh được là trái AGENTS.md hai lần.
  - 🔵 2026-11-11: dòng trên từng ghi "giá của phép lọc +42 ms trước, +11 ms sau" — **lời khai sai, đã gỡ**. Đó là hai lượt chạy ở hai trạng thái máy khác nhau, không phải trước/sau của một phép sửa.
- Phân bố khoá của bộ nhớ **không ảnh hưởng** tới thời gian quét, và điều đó ngược với dự đoán: tôi chờ rằng rải qua nhiều khoá sẽ rẻ hơn vì `signature_matches` loại một khoá lạ ngay điều kiện đầu. Ba lượt đo cho hai phân bố bằng nhau trong nhiễu (dense p95 78,7–83,8 ms · spread p95 78,2–86,8 ms), vì chi phí chi phối là việc duyệt 2.000 hàng, không phải việc so ngữ cảnh. Nên chia theo khoá luật không giúp gì ở dân số của AD-67 mục 11 theo **cả hai** chiều. Phép đo giữ cả hai phân bố để một lần tăng số hàng ghi nhớ sau này có mốc so.
- Lệnh ghi rời không tự khai vào `mergedSinceScan`, nên một lượt quét cả Chương đang bay (đã đọc đĩa TRƯỚC lượt ghi) thay trọn `results` và dựng lại chỗ vừa ghi nhớ hoặc huỷ chữ vừa nhận thay. Mọi lượt ghi rời nay đi qua `setEntry`/`dropEntry`, hai hàm cùng gọi `markFresh`. Đối chứng: làm rỗng thân `markFresh` ⇒ đúng hai ca đua đỏ, 31 ca còn lại xanh.
- Đường ghi nhớ từng tự cộng `counts.ignored + 1`, trái chính bất biến mà comment của đường quét một phần ghi ra: `counts` là tổng cả Chương và chỉ lượt quét cả Chương được khai lại. Đã gỡ phép cộng tay; một ca mới canh rằng cú bấm không làm số nhúc nhích và lượt quét kế thì có.
- Spec e2e bắt một lỗi mà **vitest không thể**: hàng của thẻ chỉ chọn theo `@pointerdown`, nên trong WKWebView cú bấm thật không chọn hàng nào và *Bỏ ghi nhớ* im lặng không làm gì — vitest xanh vì nó gọi thẳng hàm state, không đi qua sự kiện. Thêm `@mousedown` cạnh `@pointerdown`: việc chọn một hàng không được phụ thuộc vào việc môi trường phát họ sự kiện nào. Phép thêm đó đưa handler vào `Kiểm K` của `check:commands` nên phải khai một dòng `HANDLER_TABLE`; dòng đó đã đối chứng (gỡ ra ⇒ cổng đỏ).
- Vế **bàn phím** của spec e2e không có đường tự động: `e2e/AGENTS.md` ghi sẵn rằng bàn đo này không kích hoạt được một `<button>` bằng bàn phím. Đã thử cả hai cách rồi mới tin — Tab vào dải 12 lượt không tới nút, `focus()` vào được nhưng Space không kích hoạt. Tệp đó là luật thư mục mà tôi lẽ ra phải đọc TRƯỚC khi viết spec.
- Số đo trên máy RẢNH (bản `bench-release`; 2.000 luật, 9.850 câu, 493 chỗ khớp, 2.000 chỗ ghi nhớ): bộ nhớ rỗng p95 62,2–79,2 ms; bộ nhớ đầy p95 **78,2–86,8 ms** ở cả hai phân bố khoá. Trần 250 ms của AD-67 mục 11 giữ được với dư gần ba lần; phép lọc tốn khoảng +7 ms.
  - ⚠️ Cùng mã, cùng cốt, máy BẬN cho tới p95 281,0 ms (tải 9 với một VM chiếm 321% CPU) và 613 ms ở một lượt của phase Tests. Biến thiên theo tải tới hơn ba lần, nên một lượt đơn không nói được gì — mọi con số ở trên đi kèm tải máy vì thế.
- Đường quét **không** chậm đi từ 16.1: p95 bộ nhớ rỗng 77,0 ms so với 72,5 ms mà 16.1 ghi. Mọi con số cao hơn đo được trước đó là tải máy — chính phép đo của 16.1, story này không chạm, cũng báo cao hơn bản ghi 1,7–2,4 lần ở tải 65, kể cả `matcher build` và `one sentence`.
- Hai phép đo áp trần 250 ms lên **hai đại lượng khác nhau**: 16.1 áp lên lượt khớp cả Chương, 16.5 áp lên `convention_scan_chapter` end-to-end. AD-67 mục 11 nói "quét cả Chương", nên cách đọc của 16.5 là cách chặt hơn — và nó qua, nên không còn lựa chọn nào phải chốt.

## Plan Change Log

## Review Triage Log

Pass 1 (quick): 0 high · 3 medium · 3 low · 4 false · 0 maybe-false.
- Số đo của plan và của handoff đánh nhau — medium · patch: con số 157–207/275/613 ms của handoff là ghi chép phase Tests TRƯỚC khi sửa, ở tải 150–250; plan thì mang một lượt đơn và một câu khai hiệu quả "+42 → +11 ms" vô căn cứ. Đã sửa tại chỗ bằng 🔵, đo lại năm lượt cùng một binary, và đánh dấu số của handoff là đã bị thay.
- `partition_point` chưa từng được đo — medium · patch: `story_16_5_measure_convention_ignore.rs` :200 gán cả 2.000 hàng ghi nhớ cùng `rule_key("đảm bảo")`, nên đoạn chia trả về trọn bộ nhớ; và mọi `remember_second` cũng một khoá nên `debug_assert!` thứ tự không ca nào đỏ được. Đã gỡ cả phép tối ưu, bất biến thứ tự và `debug_assert`; mở mục nợ `Chủ: Story 16.7`.
- Mục Verification chưa thoả mà plan đã `in-review` — false: `cargo test` đủ đã chạy xanh (110 target / 3005 ca) và cả 14 cổng xanh, sau khi handoff được viết, nên handoff nói khác là vì nó cũ. Hai con số 7 và 8 ca vitest là hai lượt chạy khác nhau, không phải mâu thuẫn. CI thì đúng là chưa đọc — đó là cổng trước `done`, không phải trước review.
- Rào `rule_key` thiếu đối chứng — low · patch: đã gỡ thật rào đó ⇒ đúng một ca `a_phrase_that_is_not_the_rules_key_is_refused` đỏ, 72 ca còn lại xanh, rồi khôi phục và `cmp` xác nhận byte-identical.
- `CONVENTION_FLOOR`, ca đăng ký `ipc_contract`, bảng đếm `config_invariants` thiếu đối chứng — false: đó là cổng ĐÃ CÓ mà số đếm dịch đi, không phải seam mới của thay đổi này; "gỡ" một lượt bump số chỉ là trả về số cũ, mà đó đúng là trạng thái cổng đang đỏ ở.
- Khối 🔵 trong `schema.rs` mang ngày và lịch sử story — low · patch: đúng theo AGENTS.md ("không bao giờ trong mã: story id, ngày, lịch sử sự cố"; trong mã thì thay thẳng). Đã gỡ; câu đã sửa ở trên tự đứng được. Cùng cách tôi đã gỡ hai khối y hệt khỏi `chapter_origin_contract.rs` và `segment_role_contract.rs` trong chính thay đổi này.
- Hai doc comment chỉ nhắc lại chữ ký — false: `// AD-67 rule 9: …` là trích dẫn AD theo id, đúng điều AGENTS.md đòi, và cùng khuôn với comment của bước 44 ngay trên nó; còn doc của `convention_ignore_remove`/`clear` nói `bool` và `u32` NGHĨA LÀ GÌ, điều chữ ký không nói.
- Khối chưa chỉnh thụt dòng trong `ConventionManageOverlay.vue` — low · bác: repo KHÔNG có formatter nào cho `.vue` (không prettier trong `package.json`), `check:lint` và `check:layout` xanh, nên không có luật nào bị vi phạm; phase Webview đã ghi vào sổ nợ có chủ, và phép sửa được đề nghị (tách thân thẻ Luật ra component) lớn hơn một phép sửa trực tiếp.
- `stepPast` để lời báo thành `no_next` — false: `ignoreConventionSpot` gọi `say('convention.status.ignored')` **sau** `stepPast` và không điều kiện, nên lời báo ghi nhớ luôn thắng. Nửa còn lại — dải hiện một chỗ trước đó của cùng câu — là một chỗ còn dấu thật, đúng hành vi, không phải lỗi.
- `counts.ignored + 1` lệch khỏi số của Rust — low · bác: lệch là thật trên đường quét một phần, nhưng bề mặt duy nhất hiện `ignored` là màn thay cả Chương, mà màn đó lưu rồi quét lại trước khi vẽ (EXPERIENCE §Màn xác nhận thay cả Chương), nên số người dùng đọc luôn là số của lượt quét.
- Lượt quét cả Chương có thể dựng lại chỗ vừa bỏ qua — medium · defer: lệnh ghi nhớ không bump thế hệ quét và không thêm gì vào `mergedSinceScan`, nên một lượt quét mà `read_scan_input` chạy trước lúc hàng được ghi sẽ trả lại chỗ đó và thay trọn `results`. Đường nhận thay mang đúng hình dạng ấy từ 16.1, nên lỗi là chung chứ không mới; sửa một đường mà bỏ đường kia để lại hai hành vi lệch nhau.

## Design Notes

- **N = 32, chép của Proofreader.** `PROOFREAD_CONTEXT_CHARS = 32` (`core/ai/proofread.rs` :181) là tiền lệ duy nhất đang chạy cho đúng phép so đầu/đuôi mà AD-67 mục 9 trỏ tới. AD-67 cho phép **tăng** N sau này, không cho giảm, nên lấy đúng con số đã có là lựa chọn rẻ nhất mà không đóng cửa nào.
- **Ngữ cảnh NFC, không hạ chữ thường.** AD-67 mục 9 nói "tối đa N ký tự NFC mỗi bên", đúng chữ của AD-53 mục 7; cả hai không nhắc chữ thường. Khoá luật đã mang vế không-phân-biệt-hoa-thường, nên ngữ cảnh giữ nguyên dạng chữ của văn bản.
- **Một lượt NFC cho mỗi câu, không một lượt cho mỗi chỗ khớp.** Chép thẳng `signature_of` của Proofreader nghĩa là NFC lại `text[..start]` và `text[end..]` cho **từng** chỗ khớp — O(k·n) trên một câu có k chỗ. Proofreader sống được vì k nhỏ; bộ quét quy ước chạy 9.850 câu mỗi lượt. Nên: dựng một bản đồ (ký tự NFC → offset UTF-16 của cụm nó sinh ra) một lần cho mỗi câu CÓ chỗ khớp, rồi cắt ngữ cảnh trên bản đồ đó. Và bỏ hẳn bước này khi tập chữ ký rỗng — đó là trạng thái thường gặp.
  - 🔵 2026-11-10: câu trước đây nói lấy bản đồ từ `normalize()` — **sai**, `normalize()` hạ chữ thường, trái chính dòng "ngữ cảnh NFC, không hạ chữ thường" ở trên. `core/convention/ignore.rs` dựng bản đồ riêng (`ContextIndex`), cùng một lượt duyệt, giữ nguyên dạng chữ.
- **Cột hiển thị nằm ngoài khoá.** Thẻ cần `{phrase}` để vẽ `…trước [cụm] sau…`, nhưng chữ ký của AD-67 chỉ có khoá luật. Nên bảng mang thêm `avoid_phrase` — văn bản đã khớp, sau `trim` + NFC — **không** nằm trong `UNIQUE`: hai lần ghi nhớ `Đảm bảo` và `đảm bảo` ở cùng ngữ cảnh vẫn là một hàng, đúng ý "khoá luật". Lý do phải là một `.rs` doc comment ở hằng DDL, vì nó là một bất biến mà vi phạm vẫn dịch được.
- **Lọc một chỗ, ba đường.** `resolve_spans` là hàm duy nhất cả ba lệnh quét đi qua, nên phép lọc nằm ở đó và không đường nào lọt. Phép lọc **không** đi vào `Matcher`: `ConventionMatcherState` cache theo Tác phẩm và chỉ `invalidate()` khi danh sách luật đổi, còn tập ghi nhớ đổi theo lệnh khác, nên nó phải được đọc tươi mỗi lượt quét.
- **Nhãn lệnh theo `check:commands`, không theo EXPERIENCE.** `command.convention.ignore` và `command.convention.manage.tab_rules|tab_ignored|ignored_forget|ignored_clear`; EXPERIENCE không liệt kê bốn khoá lệnh của thẻ, nên chúng theo đúng khuôn id lệnh như 16.1 đã làm với `manage.open`.
- **Thẻ là component con, lớp phủ không phình.** `ConventionManageOverlay.vue` đã 676 dòng; thanh thẻ ở lớp phủ, nội dung thẻ ở `ConventionIgnoreCard.vue`, state ở `conventionManageState.ts` để `resetConventionManage` đã có sẵn lo luôn lượt đổi Tác phẩm (`check:panel-refs`).
- Phases (AGENTS.md): Rust (di trú + cốt rollback + core + lệnh + lọc) · Webview (dải + adapter + state + thẻ) · Tests that move (ca canh + đối chứng gỡ + phép đo), mỗi phase một agent mới, bàn giao qua một tệp notes bên cạnh plan này.

## Verification

**Commands:**
- `npm run build` rồi `cargo test --manifest-path src-tauri/Cargo.toml --test convention_contract --test convention_boundary` -- expected: xanh.
- `cargo test --locked --manifest-path src-tauri/Cargo.toml` **đủ bộ** + `npm test` một lượt (bước di trú và danh sách `lib.rs` đổi, chín tệp test mang cốt rollback) -- expected: xanh, và đọc cả dòng `Errors`/`Unhandled` cùng mã thoát của vitest.
- `npm run check:commands` và `npm run check:i18n` và `npm run check:panel-refs` -- expected: xanh, `POINTER_DEBT` không tăng.
- `cargo test --profile bench-release --locked --manifest-path src-tauri/Cargo.toml --test story_16_5_measure_convention_ignore -- --ignored --nocapture --test-threads=1` -- expected: p95 trong ngân sách AD-67 mục 11, nêu rõ bản dựng và dân số.
- `npm run test:story 16-5 -- --list` -- expected: liệt kê được mục tiêu test đọc ra từ plan này, không còn cảnh báo `THIẾU KHAI`.
- `npm run test:e2e -- --spec e2e/specs/story-16-5-convention-not-an-error.e2e.mjs` (ngoài sandbox) -- expected: xanh trên WKWebView; nó phủ đúng phần vitest không thể — chuột thật, tiêu điểm sau cú bấm nút trong dải, hàng chọn bằng `pointerdown`, và `onKeydown` của bảng luật đã tắt ở thẻ.
- CI: đọc lượt chạy, cả nửa Windows và lượt múi giờ UTC, trước khi đặt `done`.
