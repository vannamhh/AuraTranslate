---
title: 'Bảng chờ Glossary: bỏ ứng viên là số, lớp phủ rộng hơn, một thanh cuộn, chữ gọn, câu gốc và câu dịch đặt cạnh nhau'
type: 'feature'
ticket: ''
created: '2026-11-16'
status: 'built'
baseline_revision: '52d78217ba1e0de7a9c2a1750c2d20b45a28a875'
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

**Problem:** Ở lớp phủ Duyệt bảng chờ Glossary, Ice phải lướt qua nhiều ứng viên là số (`20`, `02`, `24`…) không bao giờ là thuật ngữ. Lớp phủ hẹp (721px), danh sách có thanh cuộn riêng lồng trong thanh cuộn của lớp phủ, chữ giải thích dài, và câu Ngữ cảnh nằm trên câu dịch nên phải đọc dọc để đối chiếu.

**Approach:** Lượt quét không sinh ứng viên số nữa, và bảng chờ không hiện ứng viên số đã nằm sẵn trong `project.db`. Lớp phủ rộng hơn, chỉ còn thanh cuộn ngoài, chữ giải thích rút gọn, câu gốc và câu dịch tham khảo đặt thành hai cột cạnh nhau.

## Boundaries & Constraints

**Always:** Ứng viên số đã có trong DB chỉ bị ẩn khi đọc bảng chờ, không xoá, không đổi `resolution`. Lượt quét lúc nhập và ước lượng số ứng viên (`estimate.rs`) dùng chung một vị từ nên con số ước lượng khớp bảng chờ. Thanh nút Nhận/Bỏ/Để lại/Lên/Xuống luôn thấy được khi danh sách dài (dính đáy lớp phủ). Phím `N`/`B`/`Space`/`↑`/`↓`/`Esc` giữ nguyên. Kích thước theo lưới bội số 7px như các số đang có (721, 98, 133).

**Never:** Không migration, không thêm cột. Không đổi luật quét tiếng Anh (token số đã không qua `is_capitalized`). Không động vào khối Dịch thử/Dịch lại/Huỷ ngoài việc đặt lại bố cục. Không thêm cấu hình bật/tắt lọc số.

**Decisions (Ice, 2026-11-16):** Một ứng viên chứa ít nhất một ký tự số (`char::is_numeric`) là ứng viên số, chỉ áp cho Tác phẩm `zh` — đo trên Tác phẩm ở ảnh chụp: bỏ 28/90 hàng (16 toàn số, 12 lẫn chữ Hán như `年1`, `在2`, `第1`). Chấp nhận mất thuật ngữ Latin kèm số trong văn bản Hán (`F35`). Giữ một plan dù vượt 1.600 token.

## I/O & Edge-Case Matrix

| Scenario | Input / State | Expected Output / Behavior | Error Handling |
|----------|--------------|---------------------------|----------------|
| Quét chữ Hán có số | Chương lặp `20` 47 lần | Không có ứng viên `20` | Không lỗi |
| Hàng số đã có trong DB | `glossary_candidate` có `20`, `resolution IS NULL` | Bảng chờ không hiện, đếm "Còn lại" không tính | Hàng vẫn nằm nguyên trong DB |
| Chữ số toàn chiều rộng | `２０` | Xử lý như `20` | — |
| Số lẫn chữ Hán | `年1`, `第1`, `4年` | Không thành ứng viên, hàng cũ bị ẩn | — |
| Tác phẩm tiếng Anh | Hàng `F35` đã có | Vẫn hiện | — |
| Danh sách dài | 90 hàng | Một thanh cuộn duy nhất, thanh nút vẫn thấy | — |
| Hàng đang mở có bản dịch | Câu Ngữ cảnh + câu dịch | Hai cột cạnh nhau: gốc trái, dịch phải | Cửa sổ hẹp thì xếp dọc như cũ |

</frozen-after-approval>

## Code Map

- `src-tauri/src/core/glossary/scan.rs` -- `count_zh_candidates` (bộ lọc `is_alphanumeric` ở vòng n-gram) là chỗ chặn khi quét; cả `scan_candidates_controlled` lẫn `terms_passing_threshold` (dùng bởi `estimate.rs:51`) đi qua nó. Thêm vị từ thuần `pub fn has_numeric_char(&str) -> bool` dùng `char::is_numeric` (phủ cả `０-９`).
- `src-tauri/src/commands/glossary.rs` -- `read_pending_snapshot` (:415) có `lang`; `retain` bỏ hàng `has_numeric_char` khi `lang == MatchLang::Zh`, trước khi đọc `chapter_source_texts`. Mọi đường đọc bảng chờ (kể cả :1385) đi qua nó; `core::pending_candidates` giữ nguyên vì không biết ngôn ngữ.
- `src-tauri/tests/glossary_scan_contract.rs` -- ca quét; `src-tauri/tests/glossary_commands_contract.rs` -- ca đọc bảng chờ qua lệnh.
- `src/GlossaryQueueOverlay.vue` -- `.gq-panel` (`max-width: 721px`, `overflow: auto`) là thanh cuộn ngoài; `.gq-list` (`max-height: 60vh; overflow: auto`) là thanh cuộn lồng cần bỏ. `scrollIntoView({ block: 'nearest' })` (:105) vẫn đúng với thanh cuộn ngoài. Khối `.gq-ctx` (:484) chứa `.gq-ctx-quote` và `.gq-ref`. Chữ giải thích: `.gq-subtitle` (:336), `.gq-hint` phân loại (:482), `.gq-notrans` (:589), `.gq-stop` + `.gq-footer` (:621).
- `src/i18n/vi.json` (:1963–1994) -- các khoá `glossary.queue.*` cần rút gọn.
- `tests/frontend/glossaryQueue.test.ts` -- chỉ `:694` chạm `.gq-list`; không ca nào khẳng định chữ giải thích.

## Tasks & Acceptance

**Execution:**
- [x] `src-tauri/src/core/glossary/scan.rs` -- thêm `has_numeric_char`, bỏ n-gram thoả nó trong `count_zh_candidates` -- một vị từ cho cả quét và ước lượng.
- [x] `src-tauri/src/commands/glossary.rs` -- lọc `has_numeric_char` trong `read_pending_snapshot` khi `zh` -- ẩn hàng cũ không ghi DB.
- [x] `src-tauri/tests/glossary_scan_contract.rs` -- ca quét có số, số lẫn chữ Hán, số toàn chiều rộng. Ca đọc bảng chờ đặt ở `src-tauri/tests/glossary_commands_contract.rs` (đã gọi `glossary_pending_candidates` trên Work thật): `zh` ẩn hàng số đã chèn sẵn và hàng vẫn còn trong bảng; `en` vẫn hiện `F35`.
- [x] `src/GlossaryQueueOverlay.vue` -- `max-width` 1120px; bỏ `max-height`/`overflow` của `.gq-list`; `.gq-actions` `position: sticky; bottom: 0` có nền; `.gq-ctx` thành lưới hai cột (gốc | dịch) và xếp dọc dưới ~700px; bỏ đoạn `.gq-hint` phân loại; gộp `.gq-stop` và `.gq-footer` thành một dòng.
- [x] `src/i18n/vi.json` -- rút gọn chữ (xem Design Notes); xoá khoá không còn dùng.
- [x] `tests/frontend/glossaryQueue.test.ts` -- ca: `.gq-list` không còn là vùng cuộn riêng; câu gốc và câu dịch nằm trong cùng hàng lưới `.gq-ctx`.

**Acceptance Criteria:**
- Given Tác phẩm ở ảnh chụp (90 hàng chờ), when mở bảng chờ, then không còn hàng nào chứa chữ số (còn 62 hàng), và `SELECT count(*) FROM glossary_candidate` không đổi.
- Given 90 hàng chờ, when cuộn tới cuối, then chỉ có một thanh cuộn và thanh nút vẫn hiện.
- Given hàng đang mở có câu dịch, when cửa sổ rộng 1280px, then câu gốc bên trái, câu dịch bên phải cùng một hàng.

## Review Triage Log

Lượt 1 (quick): high 0 · medium 0 · low 1 · false 7 · maybe-false 0.
- low → patch: `.gq-actions` sticky dừng cách đáy 13px (padding panel), hàng lộ qua khe — đo WebKit Playwright: gap 13px.
- false: cột phải trống khi `.gq-ref` ẩn — chỉ ở trạng thái `looking`/chờ tự động thoáng qua; nút Dịch thử nằm trong `.gq-ref`.
- false: `char::is_numeric` không bắt `一`/`三` — đúng Decision Ice chốt (ký tự số), không phải lệch mã.
- false: mọi hàng là số ⇒ "Bảng chờ đã sạch" — người dùng quả thật không còn gì để duyệt; lọc là chủ ý.
- false: đường đọc khác bỏ qua bộ lọc — `pending_candidates(` trong `src/` chỉ gọi ở `read_pending_snapshot`; `:1385` đi qua nó; `estimate.rs` dùng `count_zh_candidates` đã lọc.
- false: test bố cục chỉ đọc nguồn CSS — jsdom không áp CSS scoped; assert đỏ được khi gỡ luật; bố cục thật là kiểm tay của Ice.
- false: ô Execution chưa tick — sổ sách plan, không phải lỗi mã.
- false: bỏ `margin-top` của `.gq-ref` — `gap` lưới thay thế, người rà tự xác nhận.

## Design Notes

Chữ đề xuất (Ice sửa được ở checkpoint):
- `subtitle`: "Chuỗi lặp từ {threshold} lần, chưa có trong từ điển." · `subtitle_no_threshold`: "Chuỗi lặp nhiều lần, chưa có trong từ điển."
- `category_not_persisted_hint`: bỏ.
- `no_translation_title` + `no_translation_body` → một câu: "Nhận được ngay, bản dịch chốt khi gặp lần đầu lúc dịch."
- `context_source_ai`: "AI dịch tham khảo".
- `stop_note` + `footer_rule` + `footer_origin` → "Esc để dừng, lần sau mở lại đúng chỗ này. Không mục nào tự vào Glossary."

Hai cột: `grid-template-columns: 1fr 1fr` cho `.gq-ctx-quote` và `.gq-ref`; nhãn "Ngữ cảnh" bỏ vì cột gốc tự rõ. Khi `.gq-ref` chưa hiện (chưa dịch), câu gốc chiếm cột trái, cột phải là nút Dịch thử.

## Verification

**Commands:**
- `cargo test --test glossary_scan_contract --test glossary_commands_contract` (sau `npm run build`) -- expected: xanh, ca mới chạy.
- `npx vitest run tests/frontend/glossaryQueue.test.ts tests/frontend/glossaryContextReference.test.ts` -- expected: xanh, thoát 0.
- `npm run check:i18n && npm run check:tokens` -- expected: xanh.

**Manual checks:**
- Ice mở Tác phẩm ở ảnh chụp trong `npm run tauri dev`: không còn ứng viên số, một thanh cuộn, hai cột.
