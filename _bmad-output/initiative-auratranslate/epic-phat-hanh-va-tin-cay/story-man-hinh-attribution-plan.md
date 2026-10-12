---
title: 'Màn hình Attribution'
type: 'feature'
ticket: '4'
created: '2026-10-12'
status: 'built'
baseline_revision: '3dff2dcd2fb470401c24f2ed5adcb92df8c105dd'
route: 'full'
route_source: 'auto'
risk: 'low'
review: 'quick'
review_source: 'pinned'
lenses_ran: ['quick']
review_loop_iteration: 0
context: []
---

<frozen-after-approval reason="human-owned intent — do not modify unless human renegotiates">

## Intent

**Problem:** Story 1.19 đã giao nửa dữ liệu của màn Attribution (FR109): bảng dựng từ các `.db` có mặt, xoá tệp thì ghi công biến mất, `license_kind` mở. Nửa hướng-bản-phát-hành mà §Quyết định #4 của 1.19 (Ice chốt 2026-08-08) giao cho 10.4 vẫn chưa có. Thiếu cụ thể: không mở được văn bản giấy phép, không sao chép được bản ghi công, chưa có `source_url`/`source_version`, chưa có thông báo "đã chuyển đổi" mà CC-BY-SA đòi, chưa có hai thẻ pháp lý. Ngoài ra màn này không mở được khi 0 nguồn nạp, vì nút mở nằm trong `v-if`.

**Approach:** Thêm một command Rust đọc `license_text` của MỘT nguồn theo `code`, gọi lúc người dùng bấm, theo §Quyết định #5 của 1.19. Mở rộng `AttributionOverlay.vue` tại chỗ, không dựng lại:
- mỗi hàng có nút mở văn bản giấy phép và nút sao chép bản ghi công, cùng `source_url`/`source_version`;
- một câu thông báo chuyển đổi, gắn theo `license_id` chứ không theo nguồn;
- hai thẻ pháp lý;
- nút mở hiện cả khi danh sách rỗng.

## Decisions (Ice, 2026-10-12)

1. Không thêm hai cột biên tập *Vai trò* / *Ghi chú xuất xứ* (sửa hợp đồng §Quyết định #4 của 1.19). Thay vào đó dẫn xuất từ trường đã có: *Vai trò* = `lang` + lớp nền/gỡ rời; *Xuất xứ* = `attribution` + `source_url` + `source_version`. Không đổi schema `.db`.
2. Thẻ "Nếu bạn là chủ sở hữu" ghi `github.com/vannamhh/AuraTranslate/issues`, dạng văn bản chọn được.
3. Nợ 381 + 1179 (thông báo tác giả HVTĐTD) giữ mở, đổi `Chủ: Ice`. Nợ 1172 (PRD §8.2, Trần Văn Chánh) để Ice chạy correct-course, đổi `Chủ: Ice`. Nợ 1487: story này thêm luật cấm `unbound()` trong `src/**/*.vue` vào `check-commands.mjs`.
4. Giữ plan nguyên cỡ (2.576 token tiktoken o200k lúc còn Open Questions).

## Boundaries & Constraints

**Always:**
- Mọi hàng dẫn xuất từ `list_dict_sources`. Webview và `vi.json` không mang `code`/tên nguồn nào, vì `dict_boundary.rs` canh điều này.
- Thông báo CC-BY-SA khoá theo `license_id`, không theo `code`.
- Rust trả dữ liệu, không trả câu hiển thị (AD-21).
- Mọi hành động qua `CommandRegistry` (AD-34).
- Giữ nguyên `trapTab`, Escape và đường trả tiêu điểm của 1.19.
- Tra một `code` không có ⇒ lỗi có tên, không phải chuỗi rỗng.

**Never:**
- Không thêm phụ thuộc mới. Sao chép dùng `navigator.clipboard.writeText`. URL hiện dạng văn bản chọn được, không `<a>` mở trình duyệt, vì không có plugin opener.
- Không gửi `license_text` trong `list_dict_sources`.
- Không thêm chế độ thứ tư (AD-24). Không dời nút mở ra titlebar (phiếu #90).
- Không sửa `tools/dict-build`, schema `.db` hay `dict-manifest.toml`.
- Không dựng các tệp giấy phép trong bản phát hành; đó là việc của 10.5 (FR110).

## I/O & Edge-Case Matrix

| Scenario | Input / State | Expected Output / Behavior | Error Handling |
|----------|--------------|---------------------------|----------------|
| Mở văn bản | Bấm nút ở hàng CC-CEDICT | Hiện đủ 43.304 ký tự trong lớp phủ, cuộn được, Escape trả về hàng | — |
| Không có văn bản | `license_text_len = 0` (`copyrighted`, `unknown`) | Không có nút mở văn bản | — |
| `code` biến mất | Tệp bị gỡ giữa lúc mở và lúc bấm | Câu lỗi có tên trong lớp phủ | Rust trả lỗi có tên, không panic |
| Sao chép | Bấm "Sao chép bản ghi công" | Clipboard nhận tên + `attribution` + `license_id` + `source_url` | Clipboard từ chối ⇒ câu lỗi hiện tại chỗ |
| CC-BY-SA | `license_id` bắt đầu bằng `CC-BY-SA` | Có câu "đã chuyển đổi định dạng, phân phối theo cùng giấy phép" | — |
| 0 nguồn | Thư mục `dict` rỗng hoặc mọi lớp bị `skipped` | Vẫn mở được, thấy trạng thái empty/unreadable | — |

</frozen-after-approval>

## Code Map

- `src-tauri/src/core/dict/layer.rs:443` -- `DictLayer::attributions`; truy vấn mới đọc `license_text` theo `code` đi cùng tệp này
- `src-tauri/src/core/dict/mod.rs:471,976` -- `SourceAttribution` (đã có `license_id`, `license_text_len`, `source_url`, `source_version`, `lang`) và `list_source_attributions`; giữ nguyên hình dạng
- `src-tauri/src/commands/dict.rs:317` -- `list_dict_sources`; command mới đặt bên cạnh. Không chạm `Store`
- `src-tauri/src/lib.rs:1225` -- đăng ký command (dây chung ⇒ full suite một lần)
- `src-tauri/tests/dict_sources.rs:3981-4240` -- khuôn fixture tạm của các ca Attribution
- `src-tauri/tests/dict_boundary.rs:891` -- cổng cấm định danh nguồn trong `src/**`/`vi.json`; không được nới
- `src/AttributionOverlay.vue` -- bề mặt của 1.19, mở rộng tại chỗ
- `src/panels/dictSourcesState.ts:255` -- `licenseKeyFor`, chưa có test đơn vị nào
- `src/panels/LookupPanel.vue:384,428` -- nút `[data-attribution-open]` nằm trong `v-if="dictSources.length > 0"`
- `src/panels/SourceHanViet.vue:473` -- `sourcesLine` nối `code` thô; đổi sang `display_name` (nợ dòng 871)
- `src/commands/index.ts:2539` -- `attribution.open`/`close`
- `src/i18n/vi.json:1050-1068` -- các khoá `attribution.*`
- `tests/frontend/attributionOverlayUnreadable.test.ts`, `e2e/specs/attribution-focus.e2e.mjs` -- khuôn test sẵn có
- Hiện trạng: dedupe `code` đã có ở `DictLayers::open → conflict_with` (`SkipReason::DuplicateSourceCode`), nhưng nợ dòng 1193 chưa đóng.

## Tasks & Acceptance

**Execution:**
- [ ] `src-tauri/src/core/dict/{layer,mod}.rs`, `commands/dict.rs`, `lib.rs` -- thêm command `read_dict_license_text(code)` → `Result<String, …>` có lỗi tên `UnknownSourceCode`, đọc lúc gọi -- §Quyết định #5 của 1.19
- [ ] `src-tauri/tests/dict_sources.rs` -- thêm ba ca: trả đúng văn bản; `code` lạ ⇒ lỗi có tên; tệp bị xoá sau khi mở ⇒ lỗi có tên
- [ ] `src/AttributionOverlay.vue` + `vi.json` -- thêm nút mở văn bản, nút sao chép, `source_url`/`source_version`, câu CC-BY-SA, hai thẻ pháp lý, cột *Vai trò* dẫn xuất từ `lang` + lớp (Quyết định 1; hai vai viwiktionary phải đọc khác nhau) -- nửa 10.4 của §Quyết định #4
- [ ] `src/panels/LookupPanel.vue` -- đưa nút mở ra khỏi `v-if`, giữ nguyên vị trí -- trạng thái empty/load_failed phải mở được
- [ ] `src/panels/SourceHanViet.vue` -- `sourcesLine` dùng `display_name`, rơi về `code` khi không có -- nợ 871
- [ ] `tests/frontend/` -- test `licenseKeyFor` (mọi nhánh + nhánh lạ), câu CC-BY-SA theo `license_id`, sao chép (mock clipboard, cả nhánh từ chối), mở văn bản và Escape quay về hàng
- [ ] `scripts/check-commands.mjs` -- luật cấm `unbound()` trong `src/**/*.vue`, chỉ đọc dòng mã (không đọc dòng chú thích); đối chứng đỏ bằng một `.vue` fixture thật gọi `unbound()` -- Quyết định 3
- [ ] `deferred-work.md` -- 1193 ✅ đóng, trỏ `conflict_with`; 871 ✅; 1487 ✅; 1132 + 1204 → `Chủ: Epic 10` (rà tay trên WKWebView + clipboard thật); 381 + 1179 + 1172 → `Chủ: Ice` (Quyết định 3)

**Acceptance Criteria:**
- Given lớp phủ đang mở, when chỉ dùng Tab/Shift+Tab/Enter/Escape, then với tới được mọi nút của mọi hàng và hai thẻ, rồi đóng về chỗ cũ.
- Given một fixture `author-grant` (giữ cơ chế AC9 của 1.19), when hiển thị, then hiện tác giả lấy từ `attribution` và câu "không thuộc GPL v3".
- Given `dict_boundary.rs`, when chạy, then vẫn xanh: không định danh nguồn nào vào `src/**`/`vi.json`.

## Implementation Notes

- Ba pha, mỗi pha một agent mới: Rust → webview → `check-commands` + sổ nợ.
- `read_dict_license_text` trả `IpcError` `dict.unknown_source_code` khi không có `code`. Lỗi đọc đi qua `err.store.read_failed` sẵn có.
- Trên macOS, một kết nối pool đang mở vẫn đọc được tệp sau `unlink`. Vì vậy "gỡ tệp giữa lúc mở và lúc bấm" trên app thật vẫn trả văn bản. Test Rust đo đường pool đã đóng.
- Tiêu đề thẻ chủ sở hữu là "Dành cho chủ sở hữu một nguồn ở đây", không phải "Nếu bạn là chủ sở hữu", vì `check:i18n` cấm "bạn".
- Thêm ba command `attribution.open_license` · `copy_credit` · `close_license`; hàng được ngắm bằng `@mousedown`/`@focusin`. `CLICK_FLOOR` 151→162 theo `ceil(0.85 × 190)`.
- Vai trò dẫn xuất từ `lang`, một tập cắt theo `,` (`tools/dict-build/src/schema.rs`). Trên dữ liệu thật, `viwiktionary`=`zh` và `viwiktionary-en`=`en`.
- Full suite sau bản vá review: mười bốn cổng và vitest xanh. Lúc đầu `npm run build` đỏ: Kiểm L truyền `p.code` (kiểu tuỳ chọn) thẳng vào `exec`, và các pha implement chỉ chạy vitest/cargo nên không thấy. Đã vá theo idiom `p.code ?? ''` sẵn có; sau đó build xanh, `cargo test` 91 target / 2.473 ca xanh.
- Còn nợ `Chủ: Epic 10` (1132, 1204): rà tay trên WKWebView, gồm cuộn văn bản giấy phép, clipboard thật và Tab.

## Review Triage Log

Lượt 1 (quick): 2 medium · 6 low · 2 false · 0 maybe-false.

- medium · patch — Nút "Đóng văn bản giấy phép" làm rơi tiêu điểm về `body`, Escape chết. Đúng vì nút nằm trong `v-if` nó tự đóng và `@keydown.esc` đặt trên scrim. Vá: trả tiêu điểm trong watch + thêm test.
- medium · patch — AC `author-grant` chưa có test render. Đúng: chỉ có nhánh `licenseKeyFor`. Vá: thêm test fixture.
- low · patch — `onEscape` gọi `closeLicenseText()` thẳng, bỏ qua registry (AD-34). Vá: `dispatch`.
- low · patch — `aimedAttributionCode` còn giữ sau khi đóng, nên phím tắt người dùng gán sẽ nhắm hàng cũ. Vá: reset trong `closeAttribution`.
- low · patch — AC bàn phím không có ca tự động. Vá: test danh sách điểm dừng của `trapTab`. Hai thẻ pháp lý không chứa điều khiển nào, nên không có điểm dừng.
- low · patch — `roleLangsFor` chưa có test; hai vai viwiktionary chưa được assert. Vá: thêm test.
- low · patch — Tên test "tệp bị xoá" đo pool đã đóng. Vá: đổi tên.
- low · patch — Story id trong doc của `read_license_text` và doc chỉ nhắc lại tên ở `dictSourcesState.ts`. Vá: xoá.
- low · reject — Tiêu đề thẻ lệch Quyết định 2 vì `check:i18n` cấm "bạn". Đã ghi ở Implementation Notes và báo Ice.
- low · reject — Chưa có test cho `creditTextFor` bỏ phần rỗng và cho `sourcesLine`. Đường chính đã có ca sao chép phủ; thêm ca là chi phí, gần như không ai gặp lỗi.
- false — "Kiểm L không đối chứng bằng `.vue` thật". Đã chạy lại: `src/ZzUnboundProbe.vue` gọi `commandRegistry.unbound()` ⇒ đúng một FAIL ở `:3:29`, exit 1; gỡ tệp ⇒ xanh. Đối chứng không commit nên không nằm trong diff.
- false — "Định dạng `lang` chưa xác minh". `schema.rs` khai tập cắt theo `,`, `insert.rs` dùng `GROUP_CONCAT(lang, ',')`.

## Verification

**Commands:**
- `npm run build && cargo test --test dict_sources --test dict_boundary` (trong `src-tauri/`) -- expected: xanh
- `npx vitest run tests/frontend/<tệp mới>` -- expected: xanh
- `npm run check:commands` -- expected: xanh; đỏ đúng ca khi đối chứng bằng fixture `unbound()`
- Full suite một lần (đã chạm `lib.rs` và `scripts/check-*`) -- expected: xanh

**Manual checks:**
- Nợ `Chủ: Epic 10`: trên app thật macOS, mở/cuộn văn bản giấy phép, sao chép vào clipboard thật, duyệt bằng Tab.
