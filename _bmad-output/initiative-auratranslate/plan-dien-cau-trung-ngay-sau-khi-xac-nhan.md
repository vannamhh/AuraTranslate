---
title: 'Điền câu trùng nguồn trong Chương ngay sau khi xác nhận'
type: 'feature'
ticket: ''
created: '2026-11-03'
status: 'done'
baseline_revision: '33209d4b168dbb1c3787201e352a41454e13fd0e'
route: 'full'
route_source: 'auto'
risk: 'medium'
review: 'quick'
review_source: 'pinned'
lenses_ran: ['quick']
review_loop_iteration: 0
context:
  - '{project-root}/src-tauri/AGENTS.md'
  - '{project-root}/src/AGENTS.md'
---

<frozen-after-approval reason="human-owned intent — do not modify unless human renegotiates">

## Intent

**Problem:** Điền sẵn khớp tuyệt đối (FR58) chỉ chạy lúc nạp Chương (`fill_exact_tm_matches`). Xác nhận một câu ghi cặp TM mới, nhưng các câu khác cùng nguồn trong Chương đang mở vẫn trống tới lần nạp sau.

**Approach:** Sau mỗi lượt xác nhận có chuyển tiếp thật (`version_created = true`), Rust chạy lại đúng hàm điền sẵn của lượt nạp, thu hẹp về các câu cùng `source_text` với câu vừa ký trong Chương đang mở, rồi trả các câu đã điền trong `ConfirmOutcome` để webview vá ảnh chụp và gắn dấu TM.

## Boundaries & Constraints

**Decision (Ice 2026-11-03):** Lượt điền sau ký `skipped` ⇒ `console.error` + một thông báo mới ("Đã xác nhận; chưa điền được các câu trùng từ TM. Văn bản đang có không bị đổi."), khóa i18n mới cạnh `panel.grid.nav_tm_prefill_skipped`. `ran { hidden_count > 0 }` ⇒ chỉ ghi console, không thông báo.

**Decision (Ice 2026-11-03):** Giữ một plan dù vượt 1600 token; không tách phần webview.

**Always:**
- FR58 giữ nguyên: chỉ điền hàng `draft`, `trim(target_text) = ''`, chưa về hưu, còn trong bản dịch; hàng điền vẫn `draft`, không sinh `segment_version`, không ghi cặp TM.
- Ghi qua `write_non_user_target` (AD-47/AD-50 rule 3: văn bản, hai cột baseline và xuất xứ trong một câu lệnh).
- Chọn cặp như lượt nạp: cặp đầu tiên của `pairs_for_source` theo thứ tự AD-18, qua `TmSidePolicy::from_work` (công tắc "chỉ cặp của tôi"). Kết quả điền phải trùng với thứ một lượt nạp lại sẽ điền.
- Lượt xác nhận đã commit thì không bao giờ bị lượt điền làm hỏng: điền trượt ⇒ xác nhận vẫn `Ok`, trạng thái điền báo `Skipped { code }`.
- Webview: "chữ đang gõ chưa flush vẫn thắng" (`replaceEditorSegment` đã giữ luật này).

**Never:**
- Không điền khi xác nhận lại vô hại (`version_created = false`, AC13: không ghi một byte).
- Không điền sang Chương khác, không điền mờ (FR59+), không đổi trạng thái Chương (FR5: điền TM không tính).
- Không đổi chữ ký `commands::segment::confirm_segment(open, id)` mà 80+ ca test đang gọi.
- Không chạy lượt điền trong giao dịch ghi của lượt xác nhận (đọc kho toàn cục và đọc lại `project.db` trong closure ghi nối tiếp).

## I/O & Edge-Case Matrix

| Scenario | Input / State | Expected Output / Behavior | Error Handling |
|----------|--------------|---------------------------|----------------|
| Trùng trong Chương | 3 câu cùng nguồn, cả 3 trống; ký câu 1 | Câu 2, 3 điền văn bản vừa ký, `draft`, xuất xứ của cặp; `tm_filled` liệt 2 id | — |
| Hàng đã có chữ | Câu 2 có bản nháp | Câu 2 không đổi | — |
| Ký lại | Câu đã `confirmed`, ký lần nữa | `tm_filled = []`, `tm_prefill = not_asked` | — |
| "Chỉ của tôi" bật, cặp vừa ký là của người khác | Không cặp nào qua policy | Không điền, `ran { hidden_count > 0 }` | Chỉ ghi console |
| Kho toàn cục chưa quản lý / hàng TM xuất xứ lạ | — | Xác nhận `Ok`, không điền | `skipped { code }`, không ghi gì |
| Chữ chưa flush | Webview đang giữ chữ gõ ở câu 2 | Ảnh chụp giữ chữ đang gõ | Rust chỉ điền hàng trống trên đĩa |

</frozen-after-approval>

## Code Map

- `src-tauri/src/commands/segment/chapter_read.rs:521` `fill_exact_tm_matches` -- thêm tham số thu hẹp (ví dụ `Option<i64>` id câu vừa ký ⇒ thêm `AND source_text = (SELECT source_text FROM segment WHERE id = ?)` vào truy vấn ứng viên); để `pub(super)`. Giữ tên và tệp: `tests/tm_boundary.rs:119` đòi hàm này gọi `TmSidePolicy::from_work`.
- `chapter_read.rs:129` `TmPrefillStatus` -- tái dùng nguyên hình dạng cho `ConfirmOutcome`.
- `src-tauri/src/commands/segment/confirm.rs:13,178` -- `ConfirmOutcome` thêm `tm_filled: Vec<TmFilledSegment { segment_id, target_text, translation_origin }>` và `tm_prefill: TmPrefillStatus`; hàm mới `confirm_segment_and_fill(global, open, id)` gọi `confirm_segment` rồi điền. `confirm_segment` cũ trả `tm_filled: []`, `not_asked`.
- `src-tauri/src/commands/segment/wire.rs:212` -- vỏ lấy `try_state::<Store>()` như `read_open_chapter_segments` (`wire.rs:36-51`) và gọi hàm mới.
- `src/config/segment.ts:331,351` -- kiểu và phép kiểm `ConfirmOutcome`; tái dùng `isTmPrefillStatus` (`:217`).
- `src/panels/editorPanelState.ts:1546` -- sau `dropTmFilled(id)`: `replaceEditorSegment` cho từng câu điền, thêm id vào `tmFilledSegmentIds`; `tm_prefill.kind === 'skipped'` ⇒ `datThongBao` với khóa mới (khuôn `:250-252`; nhãn `nav` khai ở `src/StatusBar.vue:471`, chuỗi ở `src/i18n/vi.json:1457`, tệp ngôn ngữ duy nhất). `dropTmFilled` ở `:158`.
- `src-tauri/tests/tm_contract.rs` -- fixture `wired(...)` (`~:1290`) đi qua vỏ thật; khuôn ca điền ở `:1343`, `:1623`.
- `tests/frontend/editorTmFilled.test.ts` -- mock `confirmSegment` (`:24`); 9 tệp test frontend dựng `ConfirmOutcome` cần thêm hai trường mới.

## Tasks & Acceptance

**Execution:**
- [x] `src-tauri/src/commands/segment/chapter_read.rs` -- thu hẹp `fill_exact_tm_matches` theo câu vừa ký -- một luật điền cho cả hai đường.
- [x] `src-tauri/src/commands/segment/confirm.rs` + `wire.rs` -- trường mới, hàm `confirm_segment_and_fill`, vỏ gọi hàm mới -- điền sau commit, trượt thì báo `skipped`.
- [x] `src-tauri/tests/tm_contract.rs` -- ca theo từng hàng Ma trận qua `wire::confirm_segment`, cộng ca "điền sau ký = điền khi nạp lại" (ký trên Tác phẩm A, nạp lại Tác phẩm B dựng giống hệt, so `target_text`/baseline/xuất xứ từng hàng).
- [x] `src/config/segment.ts` + `src/panels/editorPanelState.ts` -- kiểu, phép kiểm, vá ảnh chụp và dấu TM.
- [x] `tests/frontend/editorTmFilled.test.ts` + 9 tệp mock -- ca vá ảnh chụp, dấu TM, chữ chưa flush thắng; bổ sung trường mới vào mock.

**Acceptance Criteria:**
- Given Chương có hai câu trống cùng nguồn, when người dùng ký câu thứ nhất, then câu thứ hai hiện ngay bản dịch đó, mang dấu TM điền sẵn, vẫn chưa xác nhận, không cần nạp lại Chương.
- Given cùng dữ liệu, when ký câu thứ nhất rồi nạp lại, then lượt nạp không điền thêm câu nào (`tm_filled_segment_ids = []`).
- Given seam bị gỡ thật (vỏ quay về `super::confirm_segment`), when chạy `cargo test --test tm_contract`, then các ca mới đỏ vì câu thứ hai còn trống.

## Implementation Notes

- `fill_exact_tm_matches` nhận `only_like: Option<i64>` và trả `TmFilledSegment`; lượt nạp truyền `None`, nên hành vi lúc nạp giữ nguyên. Lượt điền sau ký chạy sau commit, ngoài giao dịch ký; vỏ IPC vẫn giữ mutex Tác phẩm đang mở như vỏ nạp Chương.
- Webview bỏ qua hàng đang có chữ gõ chưa flush: không vá, không gắn dấu TM. Trên đĩa hàng đó tạm mang văn bản TM cho tới khi lượt flush kế tiếp ghi đè.
- Thông báo `tm-prefill-skipped-after-confirm` thắng `confirmed-last` khi cả hai cùng xảy ra (câu cuối Chương + kho TM hỏng); chưa thứ tự nào đúng rõ ràng, xem Review Triage Log.
- Đối chứng đỏ: vỏ quay về `super::confirm_segment` ⇒ 6/7 ca mới ở `tm_contract` đỏ vì câu cùng nguồn còn trống; ca "ký lại không điền" xanh ở cả hai nhánh.
- Rà soát bắt 5 ca vitest đỏ do mock `ConfirmOutcome` cũ ở 3 tệp mà Code Map không liệt; đã sửa. 8 ca reset-khi-nhập đỏ có sẵn từ 6-23, đã ghi vào `deferred-work.md` (Chủ: Story 6.23).
- Chưa chạy: kiểm trên app thật (ghi nợ, Chủ: Ice) và e2e.

## Plan Change Log

## Review Triage Log

Lượt 1 (quick): high 0 · medium 1 · low 2 · false 2 · maybe-false 0.

- medium · patch — mock `confirmSegment` ở `editorNavNotice.test.ts`, `editorConfirmInFlightBlocksNav.test.ts` và tải dây ở `libraryWorks.test.ts` thiếu `tm_filled`/`tm_prefill`; chạy lại thấy 5 ca đỏ (`TypeError … reading 'kind'`, adapter trả `outcome: null`). Gộp hai phát hiện cùng gốc: mock theo hình dạng cũ. Đã giao sửa mock.
- low · patch — `confirm_segment_and_fill` có nhánh `let Some(open) = open else { return Ok(outcome) }` không tới được, vì `confirm_segment(None, …)` lỗi trước; đọc như một đường không-điền im lặng. Đã giao: giải `open` trước, bỏ nhánh chết.
- low · reject — câu cuối Chương + điền trượt ⇒ `tm-prefill-skipped-after-confirm` thắng `confirmed-last` (`editorPanelState.ts:1490`). Có thật, nhưng cần kho TM toàn cục hỏng đúng ở câu cuối, và chưa thứ tự ưu tiên nào đúng rõ ràng ⇒ báo Ice ở phần tổng kết.
- false — `eprintln!` không dấu: luật comment của AGENTS.md không áp cho chuỗi log; cùng khuôn `tm prefill bi bo qua` của đường nạp (`chapter_read.rs:507`).
- false — cờ `kyDienTmTruotSauKy` cũ có thể rò sang lượt ký sau: không, vì cờ được đặt lại ở đầu `confirmCurrentSegmentUnguarded` và trong `resetEditorPanel` (chính reviewer cũng kết luận vậy).

## Design Notes

Lượt điền chạy sau khi giao dịch xác nhận commit, trong hai bước đọc/ghi riêng của `fill_exact_tm_matches`; giữa hai bước, phép kiểm `still_eligible` trong giao dịch ghi (`chapter_read.rs:577`) đã chặn ghi đè hàng vừa có chữ.

Tỉ lệ câu trùng (đo 2026-11-03, `~/Documents/AuraTranslate`, 49 `.atproj`, chủ yếu Tác phẩm thử): 29 Chương, 416 câu chưa về hưu nguồn không rỗng; 3 câu (0,72 %) trùng nguồn trong cùng Chương, cả 3 ở một Chương. Quần thể quá nhỏ để ước lượng cho truyện thật; không dùng con số này làm căn cứ giá trị. Chi phí mỗi lượt ký thêm: một `TmSidePolicy::from_work`, một truy vấn ứng viên theo `source_text` trong Chương, một `pairs_for_source` (chỉ mục nguồn ở cả hai kho, ca `:1650`), một giao dịch ghi khi có hàng điền.

## Verification

**Commands:**
- `npm run build && cargo test --manifest-path src-tauri/Cargo.toml --test tm_contract` -- expected: xanh, các ca mới có mặt.
- `cargo test --manifest-path src-tauri/Cargo.toml --test tm_boundary --test segment_contract --test ipc_contract` -- expected: xanh.
- `npx vitest run tests/frontend/editorTmFilled.test.ts tests/frontend/editorConfirmSegment.test.ts` -- expected: xanh.
- Đối chứng đỏ: gỡ thật lời gọi điền ở vỏ, chạy lại `--test tm_contract` -- expected: ca mới đỏ vì câu thứ hai trống.

**Manual checks:**
- App thật: Chương có hai câu trùng nguồn, ký câu đầu, câu sau hiện bản dịch có dấu TM, vẫn chưa xác nhận.
