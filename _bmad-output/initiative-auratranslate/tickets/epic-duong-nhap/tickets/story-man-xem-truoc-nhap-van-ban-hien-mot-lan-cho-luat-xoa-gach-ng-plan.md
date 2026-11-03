---
title: 'Story 6.24 — Màn xem trước nhập: văn bản hiện một lần, chỗ luật xoá gạch ngang tại chỗ'
type: 'feature'
ticket: 24
created: '2026-11-02'
status: 'done'
baseline_revision: 'd3ccc54bb4e770e108982f748b14fd7562fdeda9'
route: 'full'
route_source: 'auto'
risk: 'medium'
review: 'quick'
review_source: 'pinned'
lenses_ran: ['quick']
review_loop_iteration: 0
context:
  - '{project-root}/src/AGENTS.md'
  - '{project-root}/src-tauri/AGENTS.md'
  - '{project-root}/tests/AGENTS.md'
  - '{project-root}/e2e/AGENTS.md'
---

<frozen-after-approval reason="human-owned intent — do not modify unless human renegotiates">

## Intent

**Problem:** Tầng luật làm sạch ở cột cấu hình in lại tới ~4 KiB văn bản để gạch ngang chỗ sắp xoá. Văn bản đó đã có trong cột văn bản, nên người dùng đọc hai lần, và chỗ gạch nằm xa chỗ mắt đang đọc. Ở đường tệp và dán tay, bản in lại này là chỗ duy nhất văn bản Chương hiện ra.

**Approach:** Ở đường URL, Rust gắn vào mỗi `BlockWire` các chỗ khớp của luật đang bật, đo bằng điểm mã trong văn bản của chính khối. Webview gạch ngang ngay trong khối. Ở đường tệp và dán tay, đoạn văn bản của tầng luật chuyển nguyên sang cột văn bản. Tầng luật chỉ còn danh sách luật.

## Boundaries & Constraints

**Always:** Chỗ khớp trong khối đo trên toàn Chương, không trên cửa sổ 4 096 byte. Chỉ luật đang bật. Một chỗ khớp vắt qua hai khối được Rust cắt thành một mảnh cho mỗi khối. Vị trí khối trong văn bản đã ghép lấy từ đúng phép ghép mà bước 2 dùng, không có bản thứ hai của luật ghép (AD-1). Văn bản Chương là dữ liệu, không markup (AD-16). Cùng đầu vào thì Tác phẩm tạo ra giống hệt từng byte như trước.

**Decisions (Ice, 2026-11-02, ghi ở story):** cách A, chỗ khớp theo từng khối. Nhãn luật hiện khi rê chuột hoặc có tiêu điểm, không có chip luôn hiện. Đường tệp và dán tay giữ nguyên đoạn và giới hạn hôm nay.

**Never:** Webview không tự tính vị trí khối trong văn bản đã ghép. Không đổi luật xoá gì, không đổi hai số đếm, không đổi `CleanupPreviewWire`. Không đụng màn song ngữ (`bip-*`). Không đổi dải khối, phím và chân dính của story 23.

## I/O & Edge-Case Matrix

| Scenario | Input / State | Expected |
|---|---|---|
| Khớp trong một khối | URL, luật bật khớp 3 chỗ trong 2 khối giữ | 3 mảnh gạch, mỗi mảnh trong khối của nó |
| Sau cửa sổ | Chỗ khớp ở khối nằm sau 4 096 byte đầu | Vẫn gạch |
| Vắt qua hai khối | Regex khớp từ cuối khối i tới đầu khối i+1 | Hai mảnh, mỗi khối một mảnh; `count_in_chapter` vẫn +1 |
| Khối loại | Khối loại chứa chuỗi khớp | 0 mảnh; giữ khối đó → mảnh hiện, số đếm tăng |
| Khối ảnh | Alt-text chứa chuỗi khớp | 0 mảnh |
| Luật tắt | Luật khớp nhưng `enabled = false` | 0 mảnh; bật lại → mảnh hiện |
| Hai luật chồng nhau | Hai luật phủ cùng đoạn | Một đoạn gạch, nhãn nêu cả hai luật |
| Tệp/dán tay | `.txt` > 4 096 byte, luật khớp | Cột văn bản: phần đầu có gạch và câu "Chỉ xem trước phần đầu Chương…"; cột cấu hình không có văn bản |

</frozen-after-approval>

## Code Map

- `src-tauri/src/core/segment/pipeline.rs:1538-1575` `join_kept_blocks`. Đưa thân hàm sang `join_kept_blocks_with_ranges(blocks, effective_kept) -> (String, Vec<Option<(usize, usize)>>)`: mỗi chỉ số khối một mục, `[start, end)` theo điểm mã. Khối không vào văn bản ghép thì `None`; phần ngăn cách không thuộc khối nào. `join_kept_blocks` gọi lại hàm mới và lấy `.0`. Hàm này còn nhiều chỗ gọi khác (encoding.rs, docx, anchor.rs) và chúng giữ nguyên.
- `src-tauri/src/core/cleanup/mod.rs:140,282`: `CleanupMatch{rule_tier, rule_id, start, end}`. Đơn vị là điểm mã trên toàn văn bản. `apply` ghi chỗ khớp của mọi luật, kể cả luật tắt.
- `src-tauri/src/commands/project/mod.rs`:
  - `:1343` `BlockWire`: thêm `cleanup_spans: Vec<CleanupSpanWire>`.
  - `:1375` `build_chapter_blocks_preview_wire` là chỗ duy nhất dựng `BlockWire`. Nó nhận thêm `report.matches` và tập luật bật.
  - `:1913-1917`: tập luật bật trong `build_cleanup_preview_wire`. Tách ra một hàm để cả hai nơi dùng chung.
  - `:1838-1862` `cleanup_and_chapters_preview_for`: có cả `detail_chapter.blocks` lẫn `cleanup_report`. Đây là chỗ nối. Mọi đường (eager, lazy, `tier2_block_set_kept`, `confirm_range`) đều đi qua nó.
- `src-tauri/tests/segment_contract.rs:10839-10911`: 5 literal `BlockWire` và một ca ghim bộ khoá. Thêm khoá mới vào.
- `src/config/project.ts`: `BlockWire :268` và `isBlockWire :498`. Thêm trường và kiểm `cleanup_spans.every(isCleanupSpanWire)`.
- Fixture TS dựng `BlockWire`: `tests/frontend/importPreviewBlocks.test.ts`, `importPreviewEncodingWireShape.test.ts`, `importPreviewFooterLayout.test.ts`.
- `src/ImportPreviewOverlay.vue`:
  - `cleanupPiecesOf :168` dùng lại cho thân khối. Mỗi mảnh cần mang danh sách `(tier, id)` đang phủ nó để dựng nhãn từ `importPreviewSelectedCleanup.rules`.
  - `blockPlainText :672` và chỗ render `:1773-1776`.
  - Văn bản tầng luật nằm ở `:1333-1348`, trong `aside.ip-config :1004`. Ở đường tệp và dán tay, nó chuyển vào tầng 2 của `.ip-main :1648`, thay câu `ip-tier-empty-reason :1789`.
  - CSS `.ip-cleanup-text :2524` và `.ip-cleanup-struck :2537`.
- `src/importPreviewState.ts`: mọi thay đổi luật đều gọi `reloadImportPreviewAfterRuleChange :1976`. Ở đường URL hàm này gọi tiếp `reloadUrlImportChapterOnScreen :1681`, nên `chapterDetailBlocks` được tải lại cùng luật.
  - ⚠️ `importPreviewSelectedBlocks :813` đọc `chapterDetailBlocks` khi `urlChapterDetailEagerStale` bật. Lần đổi luật đầu tiên ở Chương 0 thì giá trị này còn `null` trong lúc chờ, nên danh sách khối biến mất và cột nhảy vị trí cuộn (AC7). Phải giữ khối cũ cho tới khi khối mới về.
- Test đọc văn bản in lại: chỉ `tests/frontend/importPreviewCleanup.test.ts:413-416`. Không spec e2e nào đọc nó.
- Tests that move: `tests/frontend/importPreviewBlocks.test.ts`, `tests/frontend/importPreviewCleanup.test.ts`, `tests/frontend/importPreviewCleanupStrikeInPlace.test.ts`, `tests/frontend/importPreviewEncodingWireShape.test.ts`, `tests/frontend/importPreviewFooterLayout.test.ts`, `src-tauri/tests/cleanup_contract.rs`, `src-tauri/tests/segment_contract.rs`, `e2e/specs/story-6-24-cleanup-strike-in-place.e2e.mjs`.

## Tasks & Acceptance

Bốn pha, mỗi pha một agent mới, bàn giao qua tệp trong scratchpad: Rust · Webview · Tests (vitest) · e2e.

**Execution:**
- [ ] `pipeline.rs` -- thêm `join_kept_blocks_with_ranges`; `join_kept_blocks` dựa trên nó -- một luật ghép (AD-1).
- [ ] `commands/project/mod.rs` -- `BlockWire.cleanup_spans`; hàm thuần ánh xạ `(matches, ranges, enabled) -> Vec<Vec<CleanupSpanWire>>`: cắt chỗ khớp vắt qua khối và đổi sang tọa độ của khối; dùng chung hàm tập luật bật -- criterion 2–4.
- [ ] `src-tauri/tests/` (`cleanup_contract.rs` hoặc `webimport_contract.rs`, `segment_contract.rs`) -- ca theo từng dòng ma trận I/O trên đường sản phẩm thật (`url_import_batch_wire` hoặc `preview_chapter_detail`), cộng ca của seam: `joined[start..end] == văn bản khối` với mọi khối.
- [ ] `src/config/project.ts` -- type và guard.
- [ ] `src/ImportPreviewOverlay.vue` -- khối giữ vẽ theo mảnh, có gạch; đường tệp và dán tay đưa đoạn văn bản, câu cắt và câu rỗng sang cột văn bản dưới một tiêu đề mới; tầng luật bỏ phần văn bản; nhãn luật theo Design Notes.
- [ ] `src/importPreviewState.ts` -- giữ khối hiện tại trong lúc tải lại sau khi đổi luật.
- [ ] `src/i18n/vi.json` -- tiêu đề đoạn văn bản và mẫu nhãn luật; xoá khoá không còn dùng.
- [ ] vitest: ba tệp fixture ở trên, `importPreviewCleanup.test.ts`, và một tệp mới `tests/frontend/importPreviewCleanupStrikeInPlace.test.ts` (mảnh trong khối, nhãn hai luật, đường tệp không còn văn bản ở cột cấu hình, khối không biến mất khi tải lại).
- [ ] `e2e/specs/story-6-24-cleanup-strike-in-place.e2e.mjs` -- máy chủ cục bộ phục vụ một trang có luật khớp 3 chỗ (một chỗ sau 4 096 byte) và một `.txt`; kiểm số mảnh gạch, văn bản hiện một lần, tắt luật thì mảnh mất, `scrollTop` giữ nguyên.

**Acceptance Criteria:** criteria 1–7 của story. Thêm:
- Given phần ánh xạ ở Rust bị gỡ thật (luôn trả `cleanup_spans` rỗng), when chạy spec e2e mới, then ca criterion 2 đỏ vì có 0 mảnh gạch.

## Implementation Notes

- Rust: `join_kept_blocks_with_ranges` là thân duy nhất của phép ghép; `cleanup_spans_by_block` (thuần, riêng tư) cắt chỗ khớp theo khối bằng `partition_point` trên các khoảng đã đặt; `enabled_cleanup_keys` dùng chung với `build_cleanup_preview_wire`.
- `BlockWire.cleanup_spans` lấy `matches` từ `chapter.cleanup_report` của chính Chương chi tiết, nên chỉ cần thêm tham số `cleanup_rules` cho `build_chapter_blocks_preview_wire`; không đổi chữ ký pub nào.
- Ca Rust mới nằm ở `cleanup_contract.rs` (qua `chapter_detail_for_index`); đối chứng gỡ vòng lặp ánh xạ làm 5 ca đỏ. HTML thử cần xuống dòng giữa các thẻ, nếu không cửa sổ 4 096 byte của đường lazy trả `None`.
- Webview: mảnh gạch của khối dùng lại `cleanupPiecesOf` (mảnh mang danh sách luật phủ); nhãn là `span.ip-cleanup-hint` ẩn kiểu chỉ-dành-cho-trình-đọc, hiện ở `:hover`, `:focus-visible` và trong `.ip-block-focused`; mảnh ở đường tệp có `tabindex="0"`.
- Đường tệp và dán tay: văn bản chuyển sang tầng 2 dưới tiêu đề `tier2_text_title`; tầng 3 chỉ còn danh sách luật.
- AC7: `reloadUrlImportChapterOnScreen` gieo `chapterDetailBlocks`/`chapterDetailCleanup` từ giá trị đang chọn trước khi bật `urlChapterDetailEagerStale`.
- Vitest: `importPreviewCleanupStrikeInPlace.test.ts` (10 ca) phủ hàng ma trận mà happy-dom chứng minh được; hiện nhãn khi rê chuột/tiêu điểm (CSS), `scrollTop` và chỗ khớp sau 4 096 byte qua Rust thật để e2e.
- Mỗi ca mới có đối chứng gỡ thật (mảnh khối, gieo AC7, mẫu văn bản tầng 2, guard `cleanup_spans`, nhãn nhiều luật, `tabindex`, văn bản lạc về tầng 3) và đều đỏ đúng lý do.
- e2e: `story-6-24-cleanup-strike-in-place.e2e.mjs` (2 ca) phủ gạch sau 4 096 byte qua Rust thật, nhãn theo `:focus-visible` và con trỏ J/K, văn bản hiện một lần, `scrollTop` giữ qua tắt/bật luật, đường `.txt`; đối chứng trả `cleanup_spans` rỗng làm ca criterion 2 đỏ "0 mảnh", khôi phục thì xanh.
- e2e: bộ lái WKWebView không dựng `:hover` từ Actions, nên nhãn khi rê chuột chỉ được khoá bằng luật CSS trong CSSOM (không bằng con trỏ); và pointer `origin: element` lên ô tick làm cuộn cột 53 px, nên ca bấm ô tick bằng toạ độ viewport.

## Plan Change Log

## Review Triage Log

Vòng 1 (lens `quick`, cộng 1 phát hiện của chính build). Kết quả: high 0 · medium 1 · low 6 · false 2 · maybe-false 1.

- low · patch · Chú thích tiếng Việt mới ở `mod.rs` (`BlockWire.cleanup_spans`, `enabled_cleanup_keys`, `cleanup_spans_by_block`), `pipeline.rs` (`join_kept_blocks_with_ranges`) và đầu tệp vitest mới, trái luật chú thích của AGENTS.md. Đã sửa: còn một hoặc hai dòng tiếng Anh nêu bất biến, phần còn lại xoá.
- low · patch · Tên story trong `describe` của tệp vitest mới và spec e2e mới. Đã bỏ.
- low · patch · `.ip-cleanup-struck` có hai luật CSS rời. Đã gộp làm một.
- low · patch · `cleanupPiecesOf` tạo `[...text]` cho mọi khối kể cả khi không có chỗ khớp. Đã chuyển lệnh trả sớm lên trước.
- low · reject · Nhãn rỗng khi không tìm thấy luật theo khoá. Khối và luật luôn tới từ cùng một lượt Rust, và lúc gieo AC7 cũng lấy cả hai từ cùng trạng thái, nên khó gặp; sửa thì phải thêm một nhánh bảo vệ.
- low · reject · Nhãn hiện ở khối đang trỏ có thể đè nhau khi một khối có từ hai chỗ gạch cùng dòng, và nhãn dài chạy quá cột vì `nowrap`. Watermark thường chỉ khớp một chỗ mỗi khối; Ice kiểm bằng mắt ở lượt dùng thật.
- maybe-false · reject · Khối sau vẽ đè lên nhãn vì nhãn không có `z-index`. Không tổ tiên nào cắt `overflow`; muốn biết có bị đè không thì phải xem trong WebKit. Nếu đúng thì chỉ là `low`. Để lượt dùng thật của Ice.
- false · Thứ tự nhãn không xác định vì khoá sắp xếp thiếu tầng: `sort_by_key` ổn định và `apply` sinh chỗ khớp theo thứ tự cố định, nên thứ tự luôn như nhau.
- false · Ô đánh dấu của plan chưa tick, hai log còn trống: sửa thì chỉ là sửa plan, mà bước rà không nhận phát hiện kiểu đó.
- medium · defer · Lệnh lazy `preview_chapter_detail` trả `None` khi 4 096 byte HTML thô đầu tiên không có dấu xuống dòng, và webview xoá cả khối lẫn tầng luật. Lỗi có từ 6.10a; ở trang nén một dòng, mỗi lần đổi luật sẽ mất khối. Đã ghi vào `deferred-work.md`, `Chủ: Ice`.

## Design Notes

- Nhãn luật:
  - Mỗi mảnh gạch có một nhãn ẩn, viết "luật · mẫu · tầng" cho mọi luật phủ nó. Rê chuột hoặc đặt tiêu điểm thì nhãn hiện thành chú thích bằng CSS `:hover` và `:focus-visible`. Ngoài lúc đó nhãn được ẩn kiểu chỉ dành cho trình đọc màn hình, nên trình đọc vẫn đọc được.
  - Đường URL: tiêu điểm là con trỏ khối `J`/`K`. Khối đang được trỏ hiện nhãn của mọi mảnh trong nó. Mảnh không nhận `Tab`, để giữ mô hình listbox.
  - Đường tệp: mảnh có `tabindex="0"`. Đoạn văn chỉ dài 4 096 byte nên số điểm dừng ít.
- Đường URL thôi vẽ `CleanupPreviewWire.text`. Cửa sổ này vốn không khớp với văn bản mà `spans` trỏ vào: eager là văn bản ghép không có override, lazy là đầu HTML thô. Lỗi đó tự hết vì không còn ai vẽ nó; story này không sửa trường đó.

## Verification

**Commands:**
- `npm run build`, rồi `cargo test --test segment_contract` và `cargo test` cho tệp test có ca mới -- expected: xanh.
- `npm run test:story 6-24` -- expected: thoát 0, không có dòng `Errors`.
- `npm run test:e2e` cho spec mới cùng `import-preview-mouse` và `story-6-23-import-preview-footer`. Chạy ngoài sandbox, sau khi dịch lại `--features wdio` -- expected: xanh. Đối chứng gỡ ánh xạ thì ca criterion 2 đỏ.

**Manual checks:**
- Ice thêm một luật khớp vào bài epochtimes.com trong app thật, thấy gạch ngay trong khối, và nhãn hiện khi rê chuột.
