---
title: 'Xuất và nhập danh sách quy ước'
type: 'feature'
ticket: '6'
created: '2026-11-11'
status: 'built'
baseline_revision: '275c1b5c1c7d829c30509e4e8016c69dbb1efb54'
route: 'full'
route_source: 'auto'
risk: 'medium'
review: 'quick'
review_source: 'pinned'
lenses_ran: ['quick']
review_loop_iteration: 0
context:
  - '{project-root}/_bmad-output/initiative-auratranslate/architecture-auratranslate/architecture-auratranslate.md §AD-67 mục 10 và gạch "Nhập có bước xem trước" của 12b, §AD-48'
  - '{project-root}/_bmad-output/initiative-auratranslate/ux-auratranslate/EXPERIENCE.md §Quy ước chữ Việt — FR159 (gạch Xuất/Nhập, Tab trong lớp phủ Quản lý, Khoá i18n mới `convention.manage.export`…`import_cancel`)'
  - '{project-root}/_bmad-output/initiative-auratranslate/ux-auratranslate/mockups/quy-uoc-chu-viet.html (hàng 4)'
---

<frozen-after-approval reason="human-owned intent — do not modify unless human renegotiates">

## Intent

**Problem:** Danh sách quy ước chỉ sống trong `global.db` và `project.db`; người dịch không mang được nó sang máy khác, không chia cho người khác, không giữ bản sao ở định dạng mở (R9, NFR9). Cả epic chưa có đường xuất nhập nào.

**Approach:** Xuất một tầng ra CSV/TSV hai cột `avoid,replacement` và nhập lại qua bước xem trước theo khuôn Glossary (AD-67 mục 10, 12b; AD-48): Rust mở hộp thoại, đọc, phân loại và giữ kế hoạch; webview chỉ hiện số đếm và quyết định từng chỗ bất đồng; ghi một giao dịch sau khi xác nhận. Bộ tách dòng, tách trường, gỡ BOM của `core/glossary/exchange.rs` dời sang một module dùng chung; định dạng sáu cột của Glossary không đổi.

## Boundaries & Constraints

**Always:**
- Tầng xuất/nhập = tầng của thanh lọc; *Cả hai* nghĩa là Tác phẩm khi có Tác phẩm mở, Toàn cục khi không (mockup hàng 4: lọc *Cả hai* ⇒ *Tầng xuất/nhập: Tác phẩm*). Nhãn `convention.manage.exchange_tier_label` luôn nói tầng đó.
- Định dạng: hàng tiêu đề bắt buộc `avoid` và `replacement`; dấu phân cách khi xuất theo đuôi tệp (`.tsv` ⇒ tab, còn lại phẩy), khi nhập dò từ hàng tiêu đề; trích dẫn RFC 4180, rào công thức và gỡ BOM như Glossary; cột hai trống ⇔ `replacement_phrase = NULL`.
- Tệp có lỗi (thiếu cột, ô trống ở `avoid`, hai hàng cùng `rule_key`, trích dẫn không đóng, không UTF-8, quá trần) bị từ chối trọn: 0 hàng ghi, lỗi đầu tiên ra dây.
- Chỉ trộn, không bao giờ thay cả tầng. Giống hệt = cùng `rule_key`, cùng `avoid_phrase` và cùng `replacement_phrase` sau khi cắt khoảng trắng; khác bất kỳ phần nào là bất đồng, mặc định *Giữ của tôi*.
- Tầng Tác phẩm: hàng có cờ tắt (đánh dấu "tắt luật Toàn cục ở Tác phẩm này") không được liệt kê, không được xuất, và khi nhập được coi như chưa có. Nhập hàng cùng khoá thì dùng lại hàng đó và bật nó lên, đúng như lượt thêm (Decision của Ice ở 16.3, 2026-11-10).
- Mỗi lượt nhập thành công vô hiệu bộ khớp; webview nạp lại danh sách rồi quét lại Chương đang mở theo đường `afterWrite` có sẵn.
- Mọi thao tác làm được bằng bàn phím và bằng chuột (R11); `POINTER_DEBT` vẫn rỗng.
- Decision 2026-11-11, Ice: nút **Xuất CSV/TSV** và **Nhập CSV/TSV** nằm ở đầu lớp phủ, ngay trước Đóng, theo mockup hàng 4. Chúng là điểm dừng Tab 1–2. Nhãn *Tầng xuất/nhập: …* nằm ở một chân lớp phủ mới.

**Never:**
- Không đổi định dạng, test hay mã lỗi của xuất nhập Glossary và nhập song ngữ; chỉ đường import của chúng đổi.
- Không đưa nội dung tệp qua dây tới webview; không thêm dependency.
- Không cho webview truyền đường dẫn tệp.

## I/O & Edge-Case Matrix

| Scenario | Input / State | Expected Output / Behavior | Error Handling |
|----------|--------------|---------------------------|----------------|
| Khứ hồi | Xuất tầng có `đảm bảo→bảo đảm` và `một cách→(trống)`, nhập vào tầng trống | Đúng hai luật đó, luật thứ hai `replacement_phrase` NULL | — |
| Rào công thức | Luật `-ạ→(trống)`, xuất CSV và TSV rồi nhập | Cho lại đúng `-ạ` ở cả hai | — |
| Phân loại | Tầng có A, B; tệp có A y hệt, B khác cụm thay, C mới | 1 mới · 1 giống hệt · 1 bất đồng; xác nhận mặc định chỉ thêm C | — |
| Lấy của file | Như trên, chọn *Lấy của file* cho B | B mang cụm thay của tệp | — |
| Hàng tắt ở Tác phẩm | Toàn cục có `thập kỷ`, đã tắt ở Tác phẩm; tệp tầng Tác phẩm có `thập kỷ→thập niên` | Tính là mới; sau nhập hàng đó bật, luật Tác phẩm che luật Toàn cục | — |
| Trùng trong tệp | `Đảm bảo` và `đảm bảo` cùng tệp | 0 ghi | `err.convention.import_duplicate_avoid` |
| Danh sách đổi giữa chừng | Một luật bất đồng bị sửa sau lúc xem trước | 0 ghi | `err.convention.import_stale` |
| Huỷ hộp thoại | Bấm Huỷ ở hộp thoại lưu hoặc chọn | Không đổi gì, không báo lỗi | — |
| Tác phẩm đóng khi đang xem trước | Kế hoạch tầng Tác phẩm còn chờ | Kế hoạch bị xoá; xác nhận nhận `no_pending_import` | lỗi trong lớp phủ xem trước |

</frozen-after-approval>

## Code Map

Dùng chung (Rust):
- `core/glossary/exchange.rs`: `Delimiter` :36, `field_needs_quoting` :163, `quote_field` :168, rào công thức :120–148, `strip_zero_width` :557, `split_first_logical_line` :621, `logical_lines` :704, `split_fields` :731; `ParseIssue` :240 (`logical_lines` chỉ trả `UnterminatedQuotedField`); `parse` :839 dò dấu phân cách ở :865–871. Dời các mục này sang module mới `core/delimited_text.rs` với lỗi riêng; `glossary::exchange` `pub use` lại tên cũ để `core/segment/bilingual.rs:19-21`, `segment/import.rs:1065`, `segment/pipeline.rs:215,555` biên dịch không đổi. `core/promptset/exchange.rs` là bản sao có chủ ý (🔴 ở :5), không đụng.
- `core/glossary/exchange_io.rs`: `read_import_file` :62 (trần qua `take` :29, UTF-8, gỡ BOM :35) và `write_export_file` :120 (tệp tạm, fsync, rename) trả `GlossaryError`. Dời lõi sang `delimited_text` (trần là tham số, lỗi trung tính); hai hàm Glossary thành vỏ đổi lỗi để mã lỗi Glossary không đổi.

Quy ước (Rust):
- `core/convention/store.rs`: `RuleRow` :12, `ConventionStoreError` :32 (mã IPC :65/:70), `list_rules` :106, `list_rules_in(tx)` :115, `add_rule` :136 (dùng lại hàng tắt :162–168, từ chối hàng bật cùng khoá :161), `normalized` :183, `find_row_by_key` :200, `edit_rule` :214. `normalize.rs:66` `rule_key`. `rule.rs:2` `RuleTier`.
- Mới `core/convention/exchange.rs`: render, parse, classify, áp kế hoạch trong một `store.write`. Nâng `CONVENTION_FLOOR` 8→9 ở `tests/convention_boundary.rs:11`.
- `commands/convention.rs`: `store_for_tier` ~:104 (Work mà chưa mở ⇒ `WorkTierUnavailable`), `invalidate` :48, `pub mod wire` :782 (mọi lệnh luật hiện là `pub fn` đồng bộ). Khuôn chép: `commands/glossary.rs` `glossary_export_tier` :887, `_after_dialog` :905, `PendingImportState` :844/:855, `glossary_open_import_preview` :1013, `ImportPreviewWire` :930, `first_issue_or_unknown` :964, `confirm` :1118, `cancel` :1147; dây :1661 (hộp thoại lưu một bộ lọc, tên tệp mặc định :1691–1696), :1750 (`blocking_pick_file` :1771–1774), :1817, :1841. Khoá `OpenWorkState` chỉ lấy SAU khi hộp thoại đóng.
- `lib.rs`: đăng ký lệnh quy ước :1176–1188, của Glossary :1367–1370, `app.manage(PendingImportState)` :1722, xoá kế hoạch chờ khi đóng Tác phẩm :1886.

Webview:
- `src/conventionManageState.ts`: `tierFilter` :34 (mặc định `'both'`, ép về `'both'` khi mất tầng Work :178), `load` :142, `afterWrite` :287–301, `guarded` :303. `src/config/convention.ts`: adapter ở :267–322, không bao giờ ném.
- Khuôn chép: `src/glossaryImportState.ts` (:137 mở, :208 xác nhận, :240 huỷ), `src/GlossaryImportOverlay.vue` (z-index 11 trên lớp phủ quản lý, xác nhận :237, huỷ :122/:247), `src/glossaryManageState.ts:656` `exportGlossaryManageTier` (thanh trạng thái `export_done`, huỷ hộp thoại thì im). Cờ bận `src/glossaryExchangeGate.ts` là của riêng Glossary; quy ước giữ cờ riêng trong state của mình.
- `src/ConventionManageOverlay.vue`: header :222 (chỉ có Đóng), `focusableWithin` :187, `trapTab` :195; chưa có chân lớp phủ. `src/App.vue:380` gắn `GlossaryImportOverlay`.
- `src/commands/index.ts`: họ `convention.manage.*` ở :5448–5621; lệnh Glossary tương ứng ở :4778–4811. Bản đồ handler ở `src/conventionHandlers.ts`.

Cổng:
- `tests/config_invariants.rs`: `blocking_wire_cases` :1449 (mọi dây hộp thoại mang `(async)`), thứ tự khoá :1309/:1315–1321, `COMMAND_FILE_CENSUS` hàng `convention.rs` :2257.
- `tests/ipc_argument_contract.rs:630` `NO_FRONTEND_CALLER`: mỗi lệnh mới phải có người gọi thật ở `src/`.
- `scripts/check-commands.mjs:235–252`: các trần đếm; mỗi `@click` là đúng một `dispatch('<id>')`.

## Tasks & Acceptance

**Execution:**
- [ ] `src-tauri/src/core/delimited_text.rs` (mới), `core/mod.rs`, `core/glossary/exchange.rs`, `core/glossary/exchange_io.rs`: dời bộ tách, gỡ BOM, trích dẫn, rào công thức và đọc/ghi tệp sang module dùng chung, Glossary `pub use` lại. Lý do: AD-67 mục 10; giữ nguyên định dạng và mã lỗi Glossary.
- [ ] `src-tauri/src/core/convention/exchange.rs` (mới), `core/convention/mod.rs`, `store.rs`: `render_tier`, `parse`, `classify` (mới · giống hệt · bất đồng, xét theo hàng tầng như danh sách thấy), `apply_import` trong một giao dịch, kiểm lại kế hoạch trong giao dịch (cũ ⇒ `ImportStale`); thêm lỗi nhập và xuất vào `ConventionStoreError` cùng mã IPC.
- [ ] `src-tauri/src/commands/convention.rs`: `ConventionPendingImportState`; bốn hàm thuần cùng bốn dây `convention_export_tier`, `convention_open_import_preview`, `convention_confirm_import`, `convention_cancel_import`; ba dây có hộp thoại hoặc ghi là `(async)`; tên tệp mặc định `convention_global.csv` hoặc `convention_work.csv`; trần đọc 1 MiB; xác nhận gọi `invalidate`.
- [ ] `src-tauri/src/lib.rs`, `core/i18n/mod.rs`, `src/i18n/vi.json`: đăng ký bốn lệnh, `app.manage` kế hoạch chờ, xoá nó khi đóng Tác phẩm; khoá `err.convention.import_*`/`export_write_failed`; mười khoá `convention.manage.*` của EXPERIENCE, `convention.manage.export_done`, nhãn `command.convention.*`.
- [ ] `src/config/convention.ts`: bốn adapter kiểm kiểu lúc chạy.
- [ ] `src/conventionManageState.ts`, `src/conventionImportState.ts` (mới): tầng xuất/nhập dẫn xuất từ `tierFilter`; xuất với thanh trạng thái `export_done`; mở, quyết định từng chỗ bất đồng, xác nhận rồi `afterWrite`, huỷ; cờ bận chặn lượt thứ hai khi hộp thoại đang mở.
- [ ] `src/ConventionManageOverlay.vue`, `src/ConventionImportOverlay.vue` (mới), `src/App.vue`: nút Xuất/Nhập ở header trước Đóng; chân lớp phủ mới mang nhãn tầng; lớp phủ xem trước xếp trên lớp phủ quản lý (z-index 11) với dòng `import_summary`, danh sách bất đồng có *Giữ của tôi* / *Lấy của file*, nút `import_apply` và `import_cancel`; Esc huỷ xem trước rồi trả tiêu điểm về nút Nhập.
- [ ] `src/commands/index.ts`, `src/conventionHandlers.ts`, `scripts/check-commands.mjs`: lệnh `convention.manage.export`, `.import`, `convention.import.confirm`, `.cancel`, `.keep_mine`, `.take_file`, kèm khai báo `pointer`; nâng trần chỉ khi cổng đòi.
- [ ] `src-tauri/tests/convention_exchange_contract.rs` (mới): mỗi hàng của bảng I/O, khứ hồi cả hai dấu phân cách, BOM; ca "danh sách đổi giữa chừng" đối chứng bằng phép gỡ thật phép kiểm lại trong giao dịch.
- [ ] `src-tauri/tests/config_invariants.rs`, `ipc_argument_contract.rs`, `convention_boundary.rs`: hàng dây chặn, thứ tự khoá, điểm danh `convention.rs`, trần tệp.
- [ ] `tests/frontend/conventionImport.test.ts` (mới): tầng dẫn xuất ở ba giá trị lọc, chọn *Lấy của file* rồi xác nhận gửi đúng quyết định, huỷ hộp thoại thì im, xác nhận gọi đúng một lượt quét lại, Xuất và Nhập đều là điểm dừng Tab trong lớp phủ.

**Acceptance Criteria:**
- Given một tầng có luật có và không có cụm thay, when người dùng xuất rồi nhập tệp đó vào một tầng trống, then tầng trống có đúng các luật ấy, kể cả luật không có cụm thay (verify của vé).
- Given `glossary_exchange_contract`, `glossary_import_dialog_contract`, `bilingual_import_contract` và `prompt_set_exchange_contract`, when module dùng chung đã thay chỗ, then cả bốn target xanh mà không test nào bị sửa.
- Given lớp phủ xem trước có một luật bất đồng, when người dùng xác nhận mà không đổi gì, then luật đang có giữ nguyên và chỉ các luật mới được thêm.
- Given lớp phủ Quản lý quy ước, when người dùng xuất, nhập, chọn một luật bất đồng và xác nhận chỉ bằng bàn phím, rồi làm lại chỉ bằng chuột, then cả hai lượt hoàn tất (R11).
- Given một Chương đang mở, when một lượt nhập thêm luật khớp chữ trong Chương, then dấu của luật đó hiện mà không cần mở lại Chương.

## Implementation Notes

- Bộ tách/trích dẫn/gỡ BOM/rào công thức và đọc-ghi tệp nằm ở `core/delimited_text.rs`; Glossary giữ `logical_lines` như vỏ đổi lỗi để `bilingual.rs` biên dịch không đổi, và hai hàm `exchange_io` là vỏ đổi lỗi nên mã lỗi Glossary không đổi.
- Rust không bao giờ thấy `both`: webview suy tầng (Both ⇒ Work khi có Tác phẩm mở, ngược lại Global); hàng cờ tắt của tầng Tác phẩm bị loại ở `visible_rows` cho cả xuất lẫn phân loại, nên nhập hàng cùng khoá là "mới" và dùng lại hàng cờ, bật nó lên.
- `apply_import` phân loại lại từng hàng của kế hoạch TRONG giao dịch; lệch ⇒ `import_stale`, 0 ghi. Đối chứng bằng phép gỡ thật: bỏ kiểm tra ấy thì đúng hai ca (`a_rule_edited_after_the_preview...`, `the_in_transaction_recheck...`) đỏ.
- `import_stale` là vĩnh viễn với kế hoạch đó; lớp phủ khoá Áp dụng (`conventionImportBlocked`) và chỉ còn Huỷ. Nhãn `import_apply` đếm `new_count` + chỗ bất đồng chọn Lấy của file (khuôn Glossary), lệch so với handoff Rust.
- Cả bốn dây là `(async)` kể cả `convention_cancel_import` (khoá cùng ô mà confirm giữ suốt lượt ghi); `OpenWorkState` chỉ khoá sau hộp thoại, giờ có ca canh ở `config_invariants.rs` cho hai dây hộp thoại của quy ước.
- Thay Tác phẩm (`replace_open_work`) cũng xoá kế hoạch Tác phẩm đang chờ, không chỉ đóng Tác phẩm.
- Các trần đếm đã trôi dưới 80% trước story này (`REGISTERED_COMMAND_FLOOR`, `TS_FLOOR`, `DISPATCH_FLOOR`, `FILE_FLOOR`, `COMPONENT_FILE_FLOOR`); đã nâng theo số đếm thật, không nới ngưỡng.
- Hộp thoại lưu chỉ có bộ lọc CSV (khuôn Glossary): `.tsv` chỉ ra khi người dùng gõ đuôi; nhập nhận cả hai. Ca khứ hồi TSV đi qua `convention_export_tier` với đường dẫn `.tsv`.
- Test frontend mock `../../src/commands` (chỉ `dispatch`) để mount được `ConventionManageOverlay`; thứ tự Tab Xuất, Nhập, Đóng được kiểm bằng `keydown` Tab thật qua `trapTab`.
- Đợt chạy đủ vitest song song với `cargo test` đủ làm tám tệp quá hạn (hook 10 s/ca 5 s); chạy lại riêng thì xanh. Không tin lượt song song.

## Plan Change Log

## Review Triage Log

Pass 1 (quick + kiểm ma trận của người điều phối): 0 high · 0 medium · 5 low · 4 false · 1 maybe-false.
- Comment trong `core/delimited_text.rs` mang story id, ngày, băng ═══, 🔵, và câu đã sai sau lần dời ("Bản CHÉP, không `use`…") — low · patch: tệp mới là dòng đang đổi, nên phải gỡ theo AGENTS.md §Code comments.
- `work_tier_is_open` khoá `OpenWorkState` trước hộp thoại, trái doc comment của hai dây — low · patch: khoá nhả ngay trong câu lệnh, không giữ qua hộp thoại, nên không có treo; câu doc sai, sửa câu.
- Mã lỗi trùng trong tệp lệch ma trận — false: hàng "Trùng trong tệp" của ma trận ghi đúng `err.convention.import_duplicate_avoid`, trùng với `vi.json:150` và mã IPC.
- R11 và Esc chưa có ca canh — low · patch: Esc huỷ xem trước và trả tiêu điểm, ↑/↓ dời con trỏ và tiêu điểm, Tab bị nhốt trong lớp phủ xem trước đều chưa có ca vitest nào chạm.
- Tiêu điểm trả về nút Nhập đang khoá sau khi xác nhận — false: `reloadConventionManageAfterImport` gọi `afterWrite` → `load`, và cả hai không đặt `busy`, `editing` hay `exchangeBusy`, nên nút không khoá lúc watcher trả tiêu điểm.
- Lời báo xuất nằm ở lớp phủ thay vì thanh trạng thái — false: Glossary (`GlossaryManageOverlay.vue:871`) và TM (`TmManageOverlay.vue:974`) cũng hiện `export_done` trong lớp phủ; chữ "thanh trạng thái" trong plan là cách gọi sai, không phải ý định khác.
- Lời báo xuất còn lại sau khi đổi tầng lọc — low · patch: đổi tầng thì xoá `exportedPath`, một dòng.
- Xuất TSV chỉ tới được bằng cách tự gõ đuôi, và hộp thoại macOS có thể ép `.csv` — maybe-false · defer (medium nếu đúng): cùng một bộ lọc với Glossary (`commands/glossary.rs:1680`, cố ý); cần một lượt thật gõ `x.tsv` vào hộp thoại lưu trên macOS và Windows.
- Ô task chưa đánh dấu, Implementation Notes kể lượt chạy test, Plan Change Log rỗng — false/bác: sửa chính plan của lượt build, không phải lỗi mã; Triage Log rỗng là vì review đang chạy.
- (Người điều phối) Không cổng nào canh lời gọi `clear_pending_convention_import_for_work` trong `replace_open_work`/`close_open_work`; ca test gọi thẳng hàm nên chỉ canh chính nó — low · patch: thêm vào phép quét nguồn có sẵn ở `project_contract.rs` cạnh hai người láng giềng, đối chứng bằng phép gỡ thật.

## Design Notes

- **Hộp thoại gốc nằm ngoài tầm e2e.** Rust mở hộp thoại bằng `blocking_save_file`/`blocking_pick_file`, nên WebDriver không lái được nó. Đường hộp thoại được canh ở Rust bằng hàm thuần `_after_dialog`, và một lượt dùng thật thuộc mục nợ `Chủ: Epic 16` đã có. Story này không viết spec e2e.
- **Phases** (AGENTS.md): Rust · Webview · Tests that move, mỗi phase một agent mới, giao nhau qua tệp handoff cạnh plan này. `lib.rs` đổi ⇒ phase cuối chạy đủ `cargo test` một lần.

## Verification

**Commands:**
- `npm run build && cd src-tauri && cargo test --test convention_exchange_contract --test glossary_exchange_contract --test glossary_import_dialog_contract --test prompt_set_exchange_contract --test bilingual_import_contract --test convention_boundary --test config_invariants --test ipc_argument_contract` -- expected: xanh
- `npx vitest run tests/frontend/conventionImport.test.ts tests/frontend/conventionManage.test.ts` -- expected: xanh, thoát 0
- `npm run check:commands && npm run check:i18n` -- expected: xanh
- Một lượt `cargo test` đủ ở phase cuối (đổi `lib.rs`) -- expected: xanh
