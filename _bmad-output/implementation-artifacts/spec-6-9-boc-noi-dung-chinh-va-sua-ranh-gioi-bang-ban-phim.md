---
title: 'Story 6.9: Bóc nội dung chính và sửa ranh giới bằng bàn phím'
type: 'feature'
created: '2026-09-07'
status: 'done'
baseline_commit: '7f7ee1d9aa9f10702a45b7aa50379c2ca731a6b4'
review_loop_iteration: 1
context:
  - '{project-root}/AGENTS.md'
  - '{project-root}/src-tauri/AGENTS.md'
  - '{project-root}/src/AGENTS.md'
  - '{project-root}/tests/AGENTS.md'
  - '{project-root}/scripts/AGENTS.md'
---

<frozen-after-approval reason="human-owned intent — do not modify unless human renegotiates">

## Intent

**Problem:** Nửa *thuật toán* của FR123 **đã hạ cánh ở Story 6.7** — `extractor.rs:47` có thân thật, `Step::ExtractMainContent` nối vào pipeline (`pipeline.rs:491-517`), và `webimport_contract.rs:237` đã canh *"văn bản bóc ra không chứa `<` của thẻ"*. Thứ còn hở là **đúng cái AC cuối gọi là điều kiện nghiệm thu**: `extract` trả một `String` **phẳng**, tầng 2 render nó thành **một đoạn văn liền** (`ImportPreviewOverlay.vue:637-641`), và **0 dòng mã** nào có khối, có ba trạng thái vạch lề, hay có một phím. `epics.md:4901` nói thẳng: *"một bản chỉ có thuật toán mà không có đường sửa tay ⇒ chưa đạt"*.

**Approach:** `Extractor` trả một **mô hình khối có thứ tự cho CẢ TRANG** thay cho `String` — mỗi khối mang thân (đoạn văn · ảnh · caption) và một trạng thái giữ/loại. Khối bị thuật toán loại **có mặt trong mô hình**, vì `Space` và `[`/`]` chỉ có nghĩa khi bóc **THIẾU** cũng sửa được, không riêng bóc **THỪA**. Tầng 2 render dãy đó thành khối đọc ở vạch lề; sáu phím đi qua handler DOM cục bộ của lớp phủ và `dispatch` xuống command đã đăng ký. Rust vẫn là nơi duy nhất ghép văn bản cuối.

## Boundaries & Constraints

**Always:**
- **AD-16 §2 — luật nằm trong KIỂU.** Mô hình không có nhánh nào mang chuỗi đánh dấu, và không có trường nào để một giai đoạn sau nhét HTML vào. `Article::content` **không được đọc** trong `extractor.rs` — cổng `webimport_boundary.rs:320` quét literal `.content` trong chính tệp đó, nên chỉ `text_content` được dùng.
- **AD-40 — `Extractor` KHÔNG trait hoá**, vẫn đúng một cài đặt dùng chung, và **0 dòng chạm mạng** (`webimport_boundary.rs:223` quét `reqwest`/`TcpStream`/literal `http://`).
- **AD-1 — Rust ghép văn bản cuối, không phải TypeScript.** Trạng thái giữ/loại người dùng đặt phải đi **xuống Rust**, và bản xem trước dựng lại từ đó. Khuôn có sẵn: tầng 3 (`CMD_CLEANUP_SET_ENABLED` `config/project.ts:466` → vỏ `wire` → chạy lại chuỗi thật). TypeScript **không được tự cắt ghép** văn bản sẽ ghi xuống đĩa.
- **Mọi phím là command đăng ký (AD-34), nhưng `keys: undefined`.** Đo 2026-09-07: `keys.ts:510-513` chỉ nuốt hợp âm không-`Mod` khi tiêu điểm ở vùng gõ, mà `isTypingZone` (`keys.ts:434`) **không phủ `<button>`** ⇒ một `Space` trần toàn cục `preventDefault()` mọi lượt bấm nút của **cả ứng dụng**. Đường đúng là handler DOM cục bộ trên scrim lớp phủ rồi `dispatch('<id>')` — khuôn `GlossaryQueueOverlay.vue:150-182`.
- **Phân biệt bằng ĐỘ LÙI, không bằng màu nhấn thứ hai** (`EXPERIENCE.md:95`). Khối đã loại: `surface-sunken` + `on-surface-variant` + đổi **thang chữ** từ chữ đọc sang chữ giao diện. Token đúng là **`ui-md-wrap`** (13px/1.66, `wraps: true`, `tokens.json:469`) — **không** `ui-label` (11px, `wraps: false`, sai vai cho văn bản chạy nhiều dòng).
- **`primary` giữ đúng ba việc cũ** (`DESIGN.md:165`); tiêu điểm bàn phím là một trong ba nên khối đang chọn được dùng nó — nhưng **không qua `box-shadow`**.
- **Tầng 2 LUÔN HIỆN và luôn nói vì sao nếu rỗng** (`deferred-work.md:9130-9169`). Đường tệp/dán tay không bóc gì nên nó rỗng **có lý do**, không phải một khuyết tật.
- **NFR15 — ghi hàng `dom_query` vào bảng Stack của spine TRƯỚC khi thêm vào `Cargo.toml`**, ghim bằng `=`. Đã mở `LICENSE` trong nguồn đã tải mà đọc (MIT, `~/.cargo/registry/src/…/dom_query-0.28.0/LICENSE`); spine `:909` đã rà nó một lượt ở diện bắc cầu.

**Ask First:**
- Phép đối chiếu giữ/loại **không phân biệt được** trên bất kỳ mẫu nào trong bảy mẫu cache của bàn đo 6.1 nếu thiếu một hằng ngưỡng ⇒ **DỪNG**. Một hằng phù thuỷ ở đây là đúng lớp lỗi §Design Notes `extractor.rs:48-51` đã từ chối một lần.
- Cần một bất biến kiến trúc mới (một `AD`) ⇒ **DỪNG**, không tự cấp số — và nếu cấp thì quét cả spine lẫn mọi `ad-brief-*.md` (AGENTS.md:14).
- Món nợ hợp âm `⌘↵` (`deferred-work.md:2971`, chủ chuyển sang story này nếu 6.5 không đăng ký) đòi **đổi** hợp âm `import.preview.confirm` đang chạy (`Mod+Alt+Enter`, `commands/index.ts:1102`) ⇒ **DỪNG**: đó là đổi một phím người dùng đã học.

**Never:**
- **Không bộ đọc riêng theo site** — v1 tuyên bố không làm (FR123, AD-40).
- **Không sửa ranh giới CÂU ở màn xem trước** (AD-39). "Ranh giới" của story này là ranh giới **bóc** (khối nào giữ). Mở một đường sửa ranh giới câu ở đây là phá AD-4.
- **Không dùng `is_probably_readable()`** làm điều kiện phân loại hay từ chối — âm tính giả đã quan sát trên mẫu `a04` (`deferred-work.md:9035-9045`, luật đã ghi ở `extractor.rs:32-36`).
- **Không chép CSS mockup.** `web-import.html:92` dùng `box-shadow` (Kiểm F cấm **tuyệt đối, không miễn trừ**, `check-tokens.mjs:1471`) và `:95` viết thẳng `13.5px/1.7` (Kiểm B2 đỏ).
- **Không nhãn "vì sao khối bị loại".** `Article` của `dom_smoothie` không có trường nào nói ra điều đó; bảng thẩm quyền `EXPERIENCE.md:89-92` chỉ có ba trạng thái, không cột lý do. Nhãn `.brule` trong mockup là **minh hoạ** (`EXPERIENCE.md:429`).
- **Không đăng ký hợp âm trần toàn cục**, và không đụng tầng 1 / tầng chuẩn hoá / tầng 3 / tầng 4.
- **Không mở rộng tầng 2 ra ngoài Chương ĐẦU TIÊN.** Giới hạn `vi.json:245` (`tier2_url_first_note`) **giữ nguyên**: dây chưa mang văn bản riêng cho từng Chương (`ChapterSplitPreviewEntryWire` chỉ có `ord`/`title`/`length`). Sửa nó là một story khác — ghi nợ, không lặng lẽ nới.

## I/O & Edge-Case Matrix

| Scenario | Input / State | Expected Output / Behavior | Error Handling |
|---|---|---|---|
| Trang báo ca thuận | HTML có breadcrumb + bài + bài liên quan + bình luận | Dãy khối cả trang theo đúng thứ tự tài liệu; khối thân bài `tm-rule`, bốn khối ngoài `ornament` | N/A |
| Bóc rỗng | `text_content` rỗng sau trim | `ExtractionEmpty` như hôm nay — **không** rơi về HTML thô | Lý do đã có, `webimport_contract.rs:249` |
| Trang 0 khối | HTML không có phần tử khối nào | Mô hình **rỗng có lý do**, phân biệt được với "chưa nạp". 🔵 **2026-09-24 (Ice chốt):** lưới an toàn cuối `build_blocks` luôn dựng ít nhất một khối khi `text_content` khác rỗng, và `text_content` rỗng đã bị `ExtractionEmpty` chặn trước, nên hàng này là **đường chết có chủ ý**; nhánh UI `tier2_empty_blocks` giữ làm hàng rào nếu lưới an toàn đổi | Không khẳng định "không có nội dung" khi chưa biết |
| Người dùng bấm `Space` trên khối `ornament` | Khối máy đã loại | Khối thành `confirmed`, văn bản sẽ ghi **dài ra**; hai số ở đầu tầng đổi theo | N/A |
| `[` rồi `]` | Chọn khối 3, `[`; chọn khối 9, `]` | Khối 3–9 `confirmed`, mọi khối ngoài dải `ornament`, **một lượt** | `]` trước `[` ⇒ kêu, không ném |
| Đường tệp/dán tay | `extract_main_content == false` | Tầng 2 rỗng **kèm câu nói vì sao** (khoá `tier_empty_story_6_9` viết lại cho đúng lý do mới) | N/A |
| Đổi bảng mã khi đang có khối | Bấm `E`, chọn ứng viên khác | Dãy khối dựng lại theo ứng viên mới; **lựa chọn tay của người dùng không được âm thầm áp sang một văn bản khác** | Nợ D2 `deferred-work.md:9255-9315` |
| Ảnh và caption trong HTML | `<figure><img alt><figcaption>` | Mô hình chở đúng nhánh `Image`/`Caption`; **0 chỗ gọi sản phẩm** cho tới 6.11/6.13 | Nợ có tên, không sửa `epics.md` |

</frozen-after-approval>

## Code Map

**Rust — điểm tiêm**
- `src-tauri/src/core/webimport/extractor.rs:47` `extract(html, url) -> Result<String, ExtractError>` — **chữ ký đổi**. `:52` `Config { text_mode: TextMode::Formatted, .. }` giữ nguyên (phép đo Task 1 ở doc-comment `:16-25` vẫn đứng). `:62` là chỗ `text_content` ra; `:32-36` là luật cấm `is_probably_readable()`. 🔴 `.content` **không được xuất hiện** trong tệp này.
- `src-tauri/src/core/webimport/mod.rs:51` `pub mod extractor;` · `:54-57` bốn `pub use` — kiểu mới phải re-export ở đây. ⚠️ Story 6.7 đã dính một lần: thiếu dòng `mod` thì trình liên kết loại trọn module.
- `src-tauri/src/core/segment/pipeline.rs:502` — **chỗ gọi `extract` DUY NHẤT của sản phẩm** (`grep` 2026-09-07: 1 lời gọi). `:491-517` nhánh `Step::ExtractMainContent`; `:492` cổng `if !extract_main_content`; `:496-497` `units`/`labels` song song theo index. ⚠️ Doc-comment `:87` còn ghi *"THÂN RỖNG (Story 6.9)"* — **mệnh đề đã hết đúng từ 6.7**, sửa tại chỗ kèm 🔵 và ngày.
- `src-tauri/src/commands/project.rs:1543` `encoding_candidate_wire(…)` — nơi khối mới chảy vào dây, **theo từng ứng viên bảng mã**. `:1620` `preview_import_encoding` · `:1633-1668` closure `verdict_and_candidates` (6.7 đã luồn `label`/`extract_main_content` xuống đây). `:1033` `EncodingCandidateWire` (6 trường) — trường khối thêm cạnh `:1054` `chapters`.
- `project.rs:235` `run_pipeline` — chỗ gọi `run_import` duy nhất từ `commands/**`; `:1387` `cleanup_and_chapters_preview_for`. Lệnh đặt trạng thái khối đi cùng khuôn `:3594` (vỏ `wire`, `try_state`, **0 quy tắc trong vỏ**).
- `src-tauri/src/core/i18n/mod.rs:63` `message_keys!` — chỉ thêm khoá nếu có **lỗi** mới; nhãn trạng thái khối **không phải lỗi** ⇒ đi qua chuỗi định danh máy + hàm ánh xạ thuần phía TS, đúng quyết định #3 của spec 6.8.
- `src-tauri/Cargo.toml:92` `dom_smoothie = "=0.18.0"` — hàng `dom_query = "=0.28.0"` đặt cạnh, **sau** khi spine có hàng Stack.

**Rust — 🔴 ĐƯỜNG GHI, chỗ vòng 1 đã hụt**
- `src-tauri/src/commands/project.rs:358-362` `create_work` dựng `PipelineInput` — **đường DUY NHẤT ghi Chương xuống `.atproj`**. 🔴 Vòng 1 dựng `.with_cleanup_rules().with_chapter_pattern().with_extract_main_content()` mà **thiếu `.with_block_overrides()`**, nên mọi lượt sửa bàn phím chỉ đổi bản xem trước còn đĩa nhận phán đoán máy — và **1230 ca Rust + 909 ca vitest + tám cổng vẫn xanh**. Override phải đi vào ĐÂY, đọc từ `Tier2BlockOverridesState` ở vỏ `wire::confirm_import_with_encoding` (`:3755`), nơi vòng 1 chỉ đọc nó để **reset**.
- `src-tauri/tests/cleanup_contract.rs:463` `preview_and_confirm_agree_byte_for_byte_on_the_same_input_and_the_same_rules` · `:994` bản N Chương — **khuôn bắt buộc phải nối sang đường URL**. Hai ca hiện có dùng `Blob(AlreadyText)` nên `extract_main_content == false` và **không khối nào tồn tại**; chúng không thể đỏ vì lỗi này.
- `src-tauri/tests/segment_contract.rs:9074` `the_import_encoding_preview_wire_shape_keeps_snake_case_field_names` — ca này ra đời **đúng cho lớp lỗi này** (chú thích của nó ghi phép đo: thêm `rename_all = "camelCase"` ⇒ tính năng chết sạch mà mọi cổng vẫn xanh). Vòng 1 chỉ nối thêm `blocks: None`, tức **chưa serialize một `BlockWire` nào**.
- `src-tauri/tests/ipc_contract.rs:896-950` — cổng đăng ký lệnh + `app.manage(...)` mà Story 6.7 viết cho ba lệnh của nó. Hai lệnh mới của story này phải vào đây (6.8 đã bỏ sót `list_domain_log` — trôi hai story, đóng luôn).

**Rust — cổng**
- `src-tauri/tests/webimport_boundary.rs:35` `SRC_RS_FLOOR = 50` (**chỉ tăng, không hạ**) · `:223` mệnh đề "0 dòng chạm mạng" · `:320` mệnh đề "`.content` không rời module" — cả hai phải **vẫn đỏ được** sau khi `extractor.rs` phình ra; mỗi mệnh đề đã có sẵn một ca kiểm-chứng-vị-từ (dương + âm) làm khuôn.
- `src-tauri/tests/webimport_contract.rs:237` `extracted_text_never_contains_an_angle_bracket_from_the_source_markup` — **mệnh đề đổi nghĩa** khi `extract` trả khối: viết lại cho nó nói *"không nhánh nào của mô hình chứa `<` của thẻ"*, và phủ **cả** nhánh `ornament` (khối bị loại cũng đi qua dây). `:249` ca bóc rỗng · `:38` `spawn_once`.
- `src-tauri/tests/segment_contract.rs` — `schema_version() == 19` **không đổi**, 0 bước di trú.

**Frontend**
- `src/ImportPreviewOverlay.vue` (1640 dòng) — `:618-629` khối comment mốc tầng 2 (mang một **giới hạn tự khai** phải cập nhật), `:630-649` `<section>` tầng 2 **là chỗ thay trọn**. `:380-389` `focusableWithin` · `:392-407` `trapTab` · `:446-449` scrim đã có `@keydown.esc`/`@keydown.tab` — **chưa có `@keydown` chung**, đó là chỗ gắn handler điều hướng. `:1050` `.ip-tier` · `:1153` `.ip-normalized-text` · `:1494` `.ip-tier-empty-reason`.
- Khuôn điều hướng để chép: `src/GlossaryManageOverlay.vue:298-338` `onKeydown` — lọc `ctrl/meta/alt` **trước mọi nhánh** (`:299`), lọc target là form field **kể cả `HTMLButtonElement`** (`:308-313`), `if (event.repeat) return` **chỉ ở nhánh phá huỷ** (`:336`); `aria-activedescendant` `:461`; hai watcher giữ tiêu điểm khỏi rơi ra ngoài scrim khi danh sách 0↔N (`:140-145`, `:167-172`).
- `src/importPreviewState.ts` (1056 dòng) — `:361` `importPreviewEmptyReasonForTier` (chữ ký nay chỉ nhận `2`) · `:1024-1074` `resetImportPreview` **30 ô**: 🔴 mọi ô mới của story này phải được **GÁN** ở đây (`check:panel-refs` Kiểm A; một lượt **đọc** không tính là dọn). `:291`/`:329` computed theo ứng viên là khuôn cho computed khối.
- `src/config/project.ts:1033`→`:223` `EncodingCandidateWire` · `:340` `isEncodingCandidateWire` · `:364` `isImportEncodingPreview` — 🔴 kiểu dây mới **phải** có vị từ kiểm kiểu lúc chạy theo khuôn này, và trường tuỳ chọn kiểm bằng `x === null || isXxx(x)` (`undefined` là **không hợp lệ**, doc `:362-363`). `:466` `CMD_CLEANUP_SET_ENABLED` là khuôn hằng + adapter ba trạng thái (`:382-404`).
- `src/commands/index.ts:1084-1120` khối Story 6.3 — sáu command mới đặt cạnh, cùng tiền tố `import.preview.*`; `:162` `CommandDeps` (dep mới **tuỳ chọn**, thiếu thì `portMissing`, không ném). `:38` `MODE_IDS` và `:66-73` `FOCUS_OWNERS` — **không đụng** (lớp phủ không khai `FOCUS_OWNERS`, luật đã ghi ở `SegmentHistoryOverlay.vue:20`).
- `src/main.ts:790` danh sách `installCommands({…})` · `:818-831` `isBlocked` — 🔴 **không** thêm lớp phủ này vào `isBlocked`: `E` và `⌘⌥↵` phải chạy được **bên trong** nó.
- `src/tokens/tokens.json:20`/`:39` `surface-sunken` · `:24`/`:43` `on-surface-variant` (cặp đã có trong `contrast.pairs` `:142-145`, **không cần cặp mới**) · `:469` `ui-md-wrap` · `:378` `source-cjk` (thang chữ đọc đang dùng).
- `src/i18n/vi.json:244` `tier2_title` = `Ranh giới nội dung` · `:245` `tier2_url_first_note` · `:247` `tier_empty_story_6_9` — **viết lại**, lý do rỗng nay là "nguồn này không bóc gì", không phải "chưa dựng".

**Chuỗi UX đã chốt — dùng nguyên văn**
`web-import.html:272` `Ranh giới nội dung` · `:273` khuôn hai số `— 6 khối giữ · 3 khối loại` · `:279` `Đã loại` · `:289` `Giữ · máy đoán` · `:297` `Giữ` · `:284`/`:310` `Đầu vùng giữ`/`Cuối vùng giữ` + `máy đặt — chưa ai xác nhận` · `:385-390` gợi ý phím `J K` `khối` · `Space` `giữ / loại` · `[ ]` `đặt biên` · `R` `luật`. Cả bảy nhãn **vô nhân xưng** ⇒ sạch với `check:i18n` Kiểm D.

## Tasks & Acceptance

**Execution:**
- [x] `ARCHITECTURE-SPINE.md` §Stack -- thêm hàng `dom_query 0.28.0` (MIT, `LICENSE` đã mở đọc, 0 gói mới vào `Cargo.lock`) **TRƯỚC** mọi lượt sửa `Cargo.toml` -- NFR15 đã ba lần là lượt "đuổi theo"; cửa này đi trước, không đi sau
- [x] `src-tauri/Cargo.toml` -- `dom_query = "=0.28.0"` -- ghim `=`; `"0.28.0"` nghĩa là `^0.28.0`
- [x] `src-tauri/src/core/webimport/extractor.rs` -- mô hình khối (thân đoạn văn · **ảnh** · **caption**, cộng trạng thái giữ/loại) + duyệt HTML gốc lấy dãy khối cả trang + đối chiếu với `text_content` để đánh dấu -- luật *"không nhánh nào mang HTML"* sống trong **kiểu** (AD-16 §2), không trong một `if` ở chỗ gọi
- [x] `src-tauri/src/core/webimport/mod.rs` -- re-export kiểu mới -- thiếu dòng `mod`/`use` thì trình liên kết loại trọn module (đã dính ở 6.7)
- [x] `src-tauri/src/core/segment/pipeline.rs` -- nhận mô hình khối ở `:502`, ghép văn bản từ khối **đang giữ**; 🔵 sửa doc-comment `:87` (*"THÂN RỖNG (Story 6.9)"* hết đúng từ 6.7) -- một mệnh đề hết đúng thì sửa tại chỗ kèm ngày
- [x] `src-tauri/src/commands/project.rs` -- kiểu dây khối vào `EncodingCandidateWire`; lệnh đặt trạng thái khối + dựng lại xem trước -- Rust ghép văn bản cuối, **không** TypeScript (AD-1)
- [x] 🔴 `src-tauri/src/commands/project.rs:358-362` -- `create_work` phải nhận và áp `block_overrides`; vỏ `wire::confirm_import_with_encoding` đọc `Tier2BlockOverridesState` rồi **truyền xuống**, reset SAU khi ghi xong -- vòng 1 chỉ reset mà không truyền, nên đĩa nhận phán đoán máy trong khi màn hình khai đã sửa
- [x] 🔴 `src-tauri/src/core/webimport/extractor.rs` -- bộ chọn khối phải phủ **mọi phần tử khối mang chữ** Readability giữ lại (`h1`–`h6`, `li`, `blockquote`, `pre`, `div` lá mang chữ), không riêng `p, img, figcaption` -- đo 2026-09-07: bảy mẫu bàn đo 6.1 có 4–34 `<p>` nhưng 4–10 tiêu đề và 37–118 `<li>`, và `a07` có **0** thẻ `<p>` ⇒ mô hình rỗng ⇒ Chương ghi xuống **rỗng**
- [x] 🔴 `src-tauri/src/commands/project.rs` -- tách luật `(start, end, total) -> Vec<Option<bool>>` ra **hàm thuần** cạnh `read_tier2_block_overrides`, vỏ `#[tauri::command]` chỉ gọi xuống -- `src-tauri/AGENTS.md:11` cấm quy tắc trong vỏ, và `tests/**` không có harness Tauri nên một luật trong vỏ là một luật **không ca nào chạm được**
- [x] `src-tauri/tests/cleanup_contract.rs` -- nối bất biến "xem trước và xác nhận trùng từng byte" sang hình dạng `Chapters(RawBytes)` **có override khác rỗng** -- hai ca hiện có dùng `Blob(AlreadyText)` nên không khối nào tồn tại; chúng **không thể đỏ** vì lỗi đường ghi
- [x] `src-tauri/tests/segment_contract.rs:9074` -- serialize **một `blocks` có dữ liệu thật** (đủ ba nhánh thân, đủ ba vạch lề) và khẳng định tên khoá + giá trị tag -- vòng 1 chỉ nối `blocks: None`, tức chưa một `BlockWire` nào đi qua serde
- [x] `src-tauri/tests/ipc_contract.rs:896-950` -- hai lệnh mới vào cổng đăng ký + `app.manage(Tier2BlockOverridesState…)`; đóng luôn `list_domain_log` mà 6.8 bỏ sót -- thiếu nó thì đổi tên tham số cho lỗi **chỉ lúc chạy**, mọi cổng vẫn xanh
- [x] `src-tauri/tests/webimport_boundary.rs` -- mệnh đề mới canh tệp `extractor.rs` đã phình; giữ `SRC_RS_FLOOR` (chỉ tăng); 🔴 mệnh đề phải quét **THÂN MÃ**, không phải doc-comment -- vòng 1 dùng token `dom_query`/`Vec<Block>` mà cả hai có mặt trong chính doc-comment, nên gỡ sạch thân vẫn xanh (cùng lớp "kiểm chứng dương khoá chỗ mù")
- [x] `src-tauri/tests/webimport_contract.rs` -- viết lại `:237` cho nói đúng mệnh đề mới, phủ **cả nhánh `ornament`**; ca dãy khối cả trang trên mẫu cache bàn đo 6.1 -- mệnh đề đổi, không chỉ mã đổi
- [x] `src/config/project.ts` -- kiểu dây khối + vị từ kiểm kiểu lúc chạy + hằng `CMD_*` + adapter ba trạng thái -- adapter không bao giờ ném; `undefined` không hợp lệ
- [x] `src/importPreviewState.ts` -- ô khối + khối đang chọn + mốc `[`; **gọi tên trong `resetImportPreview`** -- `check:panel-refs` đòi một phép **GÁN**, một lượt đọc không tính
- [x] `src/ImportPreviewOverlay.vue` -- thay trọn `:630-649`: dãy khối, vạch lề ba trạng thái, hai số ở đầu tầng, `@keydown` trên scrim -- khối đã loại đổi **thang chữ** sang `ui-md-wrap`; dấu khối đang chọn **không** `box-shadow` (Kiểm F, không miễn trừ)
- [x] `src/commands/index.ts` + `src/main.ts` -- sáu command (`J`/`K` gộp làm next/prev, `Space`, `[`, `]`, **`R`** — nợ thừa kế của 6.5) với `keys: undefined`, tiêm dep -- hợp âm trần toàn cục là phương án đã chết (đo `keys.ts:510-513`)
- [x] `src/i18n/vi.json` -- chuỗi mới, giọng **vô nhân xưng**; viết lại `:247` -- Kiểm D cấm "bạn"/"chúng tôi", `VOICE_EXCEPTIONS` rỗng
- [x] `tests/frontend/` -- ca render dãy khối, ba trạng thái, và **điều hướng bàn phím** -- khuôn nạp động state + component trong cùng một lượt
- [x] `src-tauri/AGENTS.md` -- 🔵 sửa `:36` (*"AD-41 … bộ test riêng chưa tồn tại — `core/webimport/` cũng là stub"* hết đúng từ 6.7/6.8) -- luật kho bắt sửa tại chỗ kèm ngày, không để một mệnh đề lặng lẽ sai trong tệp luật bắt buộc
- [x] `_bmad-output/implementation-artifacts/deferred-work.md` -- đóng `:9615-9626` (phím `R`) và nửa bước 2 của `:9525-9559`; nợ **MỚI có chủ**: nhánh ảnh/caption chưa có chỗ gọi sản phẩm (**Chủ: 6.11 / 6.13**), nợ D2 `:9255` cho tầng 2 -- đóng bằng chữ, không xoá

**Acceptance Criteria:**
- Given một trang có breadcrumb, thân bài, bài liên quan và bình luận, when bóc, then dãy khối chở **cả bốn** theo đúng thứ tự tài liệu, và khối ngoài thân bài mang `ornament`
- Given `extract` trả mô hình khối, when **GỠ** phép đánh dấu giữ/loại, then bộ test **CŨ** phải **đỏ** — đối chứng là một phép **GỠ**, không phải một phép chèn
- 🔴 Given một trang bất kỳ và **không override nào**, when chạy bước 2, then văn bản ghép ra **trùng đúng** đầu ra `text_content` của Story 6.7 — không mất một tiêu đề, một mục danh sách, hay một đoạn nào
- 🔴 Given người dùng đã bấm `Space`/`[`/`]` rồi xác nhận, when đọc `source_text` **trên đĩa**, then nó **trùng từng byte** với văn bản màn xem trước vừa hiện — đây là ca vòng 1 đã hụt, và nó phải đỏ nếu ai gỡ `.with_block_overrides()` khỏi `create_work`
- Given một trang **0 thẻ `<p>`** (mẫu `a07` của bàn đo 6.1), when nhập, then **không** ghi một Chương rỗng trong im lặng
- Given một khối `ornament`, when bấm `Space`, then khối thành `confirmed` và văn bản sẽ ghi **dài ra** — tức bóc THIẾU sửa được, không riêng bóc THỪA
- Given tiêu điểm đang ở một `<button>` bất kỳ **ngoài** lớp phủ, when bấm `Space`, then nút đó hoạt động bình thường — không phím nào của story này chiếm hợp âm toàn cục
- Given `cargo test`, when chạy, then `schema_version() == 19` **không đổi** và **0** bước di trú mới
- Given tám cổng `check:*` và `cargo test --locked` và `npm run test`, when chạy trọn, then **0** finding và **0** ca đỏ

## Spec Change Log

- 2026-09-07 (thi công) — **Phím `R` đúc lại thành "nhảy sang tầng 3", không "khớp luật với khối đang chọn".** §Design Notes spec 6.5 (nợ thừa kế) đề xuất "bật/tắt MỘT luật làm sạch cho ĐÚNG khối đang chọn" — dựng đúng nghĩa đó đòi `core::cleanup::apply` nhận thêm PHẠM VI KÝ TỰ (một luật chỉ áp trong vùng của một khối), một thay đổi kiến trúc nằm NGOÀI AC của spec này (AC chỉ đòi sửa RANH GIỚI BÓC, không đòi sửa PHẠM VI LUẬT LÀM SẠCH). Đường đã chọn: `R` cuộn/đặt tiêu điểm sang tầng 3 (luật làm sạch) — một điều hướng thật, đăng ký đúng `check:commands`, không giả bộ có một cơ chế khớp-theo-khối chưa tồn tại. Vế "khớp theo khối" ghi lại thành nợ mới, chưa có chủ (`deferred-work.md`, mục "Deferred from: 6-5-…").
- 2026-09-07 (thi công) — **`Tier2BlockOverridesState` dùng CHUNG một vector override cho cả năm ứng viên bảng mã** (áp theo INDEX khối, không tách theo ứng viên). Đây là một giản lược khả thi vì cấu trúc DOM/số khối bất biến qua năm bảng mã — chỉ nội dung chữ khác nhau; rủi ro hẹp (ứng viên giải mã sai làm hỏng cấu trúc thẻ) ghi thành nợ mới, chưa có chủ.
- 2026-09-07 (thi công) — **Boundary marker "máy đặt — chưa ai xác nhận" của mockup KHÔNG dựng** (chỉ dựng mốc `[` do người dùng đặt tay). Mockup vẽ một chỉ báo trực quan cho vị trí biên MÁY ĐOÁN trước khi người dùng chạm — không AC/I-O-Matrix nào của spec này đòi hỏi nó tường minh, và dựng nó đòi phân biệt "chưa ai chạm" khỏi "đã chạm" trên TỪNG cặp khối liền kề mà không thêm dữ liệu mới. Giản lược có chủ đích, ghi ra thay vì lặng lẽ bỏ qua.

- 2026-09-07 (vòng rà 1 — `bad_spec`, quay vòng) — **Finding kích hoạt, HAI lỗi cao, cả hai đã tự đo lại chứ không nhận theo lời khai của reviewer.** ① **Đường GHI bỏ qua override:** `grep -rn "with_block_overrides" src-tauri/src/` cho đúng **hai** kết quả — định nghĩa builder (`pipeline.rs:351`) và đường **xem trước** (`project.rs:1495`); `create_work` (`project.rs:358-362`, đường DUY NHẤT ghi Chương) không gọi nó, nên mọi lượt `Space`/`[`/`]` chỉ đổi bản xem trước còn đĩa nhận phán đoán máy. AC *"văn bản sẽ ghi dài ra"* **sai trên đường sản phẩm**. ② **Mô hình khối chỉ phủ `p, img, figcaption`** (`extractor.rs:185`) trong khi bước 2 nay ghép văn bản **từ khối** thay vì từ `text_content` ⇒ mọi tiêu đề/`<li>`/`<blockquote>` Readability ĐÃ GIỮ rơi khỏi văn bản ghi xuống. Đo trên bảy mẫu bàn đo 6.1: 4–34 `<p>` so với 4–10 tiêu đề và 37–118 `<li>`; mẫu `a07` có **0** thẻ `<p>` ⇒ 0 khối ⇒ Chương **rỗng**.
  **Trạng thái xấu đã tránh được:** cả hai lỗi đi qua **1230 ca Rust + 909 ca vitest + tám cổng tĩnh, tất cả xanh** — nếu nhận lượt này là đạt thì kho có một tính năng khai là xong mà đường sản phẩm không chạy, cộng một đường mất nội dung im lặng ở đúng bước ghi dữ liệu người dùng. Đúng lớp lỗi `AGENTS.md:52` đã đếm được **năm lần trong bảy ngày** ở Epic 3.
  **Đã sửa gì trong spec (mọi mục NGOÀI khối `<frozen>`):** thêm §Code Map "ĐƯỜNG GHI, chỗ vòng 1 đã hụt" trỏ `create_work` + ba cổng khuôn (`cleanup_contract.rs:463`, `segment_contract.rs:9074`, `ipc_contract.rs:896-950`); thêm bảy task bắt buộc (áp override ở đường ghi · nới bộ chọn khối · tách luật `[`/`]` ra hàm thuần vì `tests/**` không có harness Tauri · bốn ca cổng); thêm ba AC đối chứng, trong đó ca *"đọc `source_text` TRÊN ĐĨA trùng từng byte với bản xem trước"* là ca vòng 1 đã hụt.
  **KEEP — thứ vòng 1 làm ĐÚNG và phải sống sót qua lượt dựng lại:** ① `BlockState { kept, confirmed }` hai cờ trực giao suy ra đủ ba vạch lề — gọn hơn một enum ba nhánh và không đẻ trạng thái thứ tư; ② phép đối chiếu giữ/loại bằng **so khớp chính xác sau chuẩn hoá khoảng trắng**, KHÔNG một hằng ngưỡng — đúng §Ask First; ③ lý do hai lượt phân tích HTML (Readability sửa trực tiếp cây của nó nên khối bị loại không còn để duyệt) — doc-comment này đúng và đắt, giữ nguyên; ④ đặt tên trường thân là `body` chứ không `content` để không chạm cổng quét literal `.content`; ⑤ sáu command `keys: undefined` + handler DOM cục bộ trên scrim; ⑥ chọn `border-left` thay `box-shadow` và `ui-md-wrap` cho thân khối đã loại; ⑦ thói quen ghi ra chỗ giản lược thay vì giấu — ba mục §Spec Change Log ở trên giữ nguyên, **nhưng ba món nợ mới phải có CHỦ**, `Chủ: chưa có` là mồ côi và trái `AGENTS.md:33`.

## Design Notes

**Vì sao mô hình phải chở khối ĐÃ BỊ LOẠI, và vì sao điều đó buộc thêm một phụ thuộc.** `dom_smoothie::Article` phơi ra `content` (HTML phần **giữ lại**) và `text_content` — **không trường nào** liệt kê khối bị loại, cũng không có lý do loại. Một mô hình chỉ chở phần giữ lại thì **mọi khối đều đang giữ**, và cả `Space` (*bật/tắt giữ*) lẫn `[`/`]` (*đặt đầu và cuối vùng giữ*) mất sạch đối tượng — đường sửa tay của FR123 chết, mà `epics.md:4901` nói thẳng đó là điều kiện nghiệm thu. Ice chốt 2026-09-07: nâng `dom_query 0.28.0` thành phụ thuộc trực tiếp. Phép đo đỡ quyết định: nó **đã** nằm trong cây (phụ thuộc bắc cầu của `dom_smoothie`, `cargo tree` 2026-09-07), giấy phép **MIT** đã mở tệp trong nguồn đã tải mà đọc, và spine `:909` đã rà nó một lượt ở diện bắc cầu ⇒ **0 gói mới** vào `Cargo.lock`. Cái phải trả là một hàng Stack và một trách nhiệm mới cho `extractor.rs` — **không** phải một cổng thứ tư: AD-40 nói `Extractor` là *"điểm mở rộng đã đặt tên"*, và duyệt DOM vẫn nằm gọn trong vai *"byte → mô hình nội dung có cấu trúc"*.

**Vì sao `Space` không được là hợp âm trần toàn cục — một phép đo, không một sở thích.** `keys.ts:510-513`: hợp âm không mang `⌘`/`Ctrl` chỉ bị nuốt khi `isTypingZone(event.target)`, mà `isTypingZone` (`:434`) phủ `INPUT`/`TEXTAREA`/`SELECT`/`contenteditable` và **không phủ `<button>`**. Đây đúng khuyết tật `main.ts:623-643` đã đo cho `Escape` trần: *"Tab tới nút Lưu/Đóng của một dải đang mở rồi bấm `Escape` ⇒ dải đóng VÀ tập điểm cắt bị xoá, im lặng"*. Một `Space` trần toàn cục còn rộng hơn — nó `preventDefault()` **mọi** nút của cả ứng dụng. Và hai bản vá hiển nhiên đều sai theo đúng lý lẽ `main.ts:637-643` đã viết: thêm lớp phủ vào `isBlocked` sẽ giết `E` và `⌘⌥↵` **bên trong** chính nó; `.stop` trên `@keydown` tới **quá muộn** vì `attachKeymap` gắn ở pha capture. ⇒ Command vẫn đăng ký (thoả AC *"mọi phím là command"*) nhưng `keys: undefined`, và phím đi qua handler DOM cục bộ — khuôn `glossary.queue.next`/`prev` đã chạy trong kho.

**Ba thứ của mockup KHÔNG được chép, mỗi thứ một lý do khác nhau.** ① `web-import.html:92` đánh dấu khối đang chọn bằng `box-shadow` — Kiểm F cấm **tuyệt đối, không có đường miễn trừ** (`check-tokens.mjs:1471`), nên dấu chọn phải đúc lại bằng thứ khác. ② `:95` viết thẳng `13.5px/1.7` — Kiểm B2 đỏ; token đúng là `ui-md-wrap` (13px/1.66), và nó là token họ `ui` **duy nhất** khai `wraps: true`, tức token duy nhất hợp vai cho một thân khối chạy nhiều dòng. ③ Nhãn `.brule` (`khung điều hướng` · `khối bài liên quan` · `khung bình luận`) giả định thuật toán **nói ra được vì sao nó loại một khối** — `Article` không có trường đó, và bảng thẩm quyền `EXPERIENCE.md:89-92` chỉ có ba trạng thái, **không** cột lý do. `EXPERIENCE.md:429` đã khai bản dựng là minh hoạ và tài liệu thắng khi mâu thuẫn.

**Hai mệnh đề trong cây đã hết đúng, sửa tại chỗ trong lượt này.** `pipeline.rs:87` còn ghi `Step::ExtractMainContent` *"THÂN RỖNG (Story 6.9)"* — hết đúng từ Story 6.7. `src-tauri/AGENTS.md:36` còn ghi *"AD-41 … bộ test riêng của nó chưa tồn tại — `core/webimport/` cũng là stub"* — hết đúng từ 6.7/6.8 (`webimport_contract.rs:874-945` là bốn ca AD-41). Cả hai sửa kèm 🔵 và ngày, không xoá.

## Verification

**Commands:**
- `npm run build && cargo test --locked` -- expected: 0 đỏ; `dist/` phải có **TRƯỚC** `cargo test`. Số nền vòng 1: 1230 ca / 44 binary — đo lại, đừng chép.
- `npm run test` -- expected: 0 đỏ. Số nền vòng 1: 909 ca / 69 tệp. ⚠️ `fileParallelism: false` ⇒ một lượt ~98 s
- `npm run check:deps && npm run check:i18n && npm run check:tokens && npm run check:commands && npm run check:layout && npm run check:panel-refs && npm run check:gates && npm run check:debt-owner` -- expected: 0 finding mỗi cổng, chạy TỪNG cổng
- 🔴 **Đối chứng đỏ ① — đường GHI (ca vòng 1 đã hụt).** **GỠ** `.with_block_overrides(...)` khỏi `create_work` (`project.rs:358-362`) rồi chạy `cleanup_contract.rs` -- expected: ca *"xem trước và xác nhận trùng từng byte trên hình dạng `Chapters(RawBytes)` có override"* **ĐỎ**. Trả lại -- **xanh**. ⚠️ Đây là một phép **GỠ THẬT phải biên dịch và chạy** — một "đối chứng logic" không tính, và vòng 1 đã tự khai làm đúng thế cho đối chứng ②.
- 🔴 **Đối chứng đỏ ② — phủ khối.** Chạy `extract()` trên **cả bảy** mẫu `_bmad-output/implementation-artifacts/6-1-ban-do/fixtures/html/a0*.html` -- expected: mỗi mẫu cho văn bản ghép **không rỗng** và **không ngắn hơn đáng kể** `text_content` của cùng mẫu. ⚠️ `a07.html` có **0** thẻ `<p>` — nó là ca quyết định, không phải ca rìa.
- **Đối chứng đỏ ③ — mệnh đề ranh giới phải quét THÂN MÃ.** Gỡ sạch thân lượt duyệt DOM thứ hai nhưng **giữ nguyên doc-comment** rồi chạy `webimport_boundary.rs` -- expected: **ĐỎ**. Vòng 1 xanh ở ca này vì token nằm trong chính doc-comment.
- **Đối chứng ④ — dây lồng.** 🔵 **SỬA 2026-09-08 (vòng vá bước 4) — mệnh đề tôi viết ở lượt vá vòng 1 SAI, sửa tại chỗ thay vì để nó lặng lẽ sai.** Bản trước đòi *"thêm `#[serde(rename_all = "camelCase")]` lên `ChapterBlocksPreviewWire` ⇒ **ĐỎ**"*. Đo 2026-09-08: **XANH**, và lý do là một dữ kiện về HÌNH DẠNG chứ không phải một chỗ hở — mọi trường của họ kiểu này (`blocks` · `body` · `kept` · `confirmed` · `text` · `src` · `alt` · `kind`) đều là **từ đơn**, nên `camelCase` là một phép biến đổi **rỗng** ở đây. Đối chứng đúng cho lớp lỗi này là một phép **đổi tên tag/variant** (`#[serde(rename = …)]` trên một nhánh `BlockBodyWire`), không phải `rename_all`. ⚠️ Giữ nguyên bài học gốc: ca `segment_contract.rs:9074` ra đời vì một lượt `camelCase` từng giết sạch tính năng mà mọi cổng vẫn xanh — lớp lỗi đó **thật**, chỉ là phép gieo tôi chọn không chạm được nó trên hình dạng hôm nay.
- **Đối chứng ⑤ — giới hạn đã biết, kỳ vọng XANH.** Gieo `keys: ['Space']` vào một lệnh tầng 2 rồi `npm run check:commands` -- expected: **XANH**. Cổng KHÔNG canh việc này ⇒ ghi thành nợ **có chủ** ở `deferred-work.md`, đừng đọc lượt xanh thành "đã canh".
- `cargo test --locked --test segment_contract` -- expected: `schema_version() == 19`, mệnh đề **không đổi**

**Manual checks (if no CLI):**
- Dán 3 link, bấm tải: tầng 2 hiện dãy khối; breadcrumb và bình luận ở `ornament`, thân bài ở `tm-rule`; chụp màn ở **cả hai theme**.
- `J`/`K` đi giữa khối, `Space` đảo trạng thái, hai số ở đầu tầng đổi theo **ngay**; giữ `Space` không bắn một tràng IPC.
- Bấm `Space` đổi vài khối rồi **xác nhận thật**, mở lại Chương vừa tạo: văn bản trên đĩa **đúng thứ màn xem trước đã hiện**.
- `Tab` xoay vòng trong lớp phủ, không thoát ra; `Esc` đóng và trả tiêu điểm về nút đã mở nó.
- Tab tới một nút **ngoài** lớp phủ rồi bấm `Space`: nút hoạt động bình thường.
- Nhập từ một tệp `.txt`: tầng 2 **vẫn hiện** và nói rõ nguồn này không bóc gì — không phải một khung trắng.

## Suggested Review Order

**Mô hình khối — bắt đầu ở đây**

- Chữ ký đổi từ `String` phẳng sang dãy khối cả trang; đọc doc-comment đầu tệp trước.
  [`extractor.rs:150`](../../src-tauri/src/core/webimport/extractor.rs#L150)

- Ba nhánh thân; luật "không nhánh nào mang HTML" sống trong KIỂU, không trong một `if`.
  [`extractor.rs:112`](../../src-tauri/src/core/webimport/extractor.rs#L112)

- Lượt phân tích HTML thứ hai — cây của Readability đã xoá khối bị loại, nên phải parse lại.
  [`extractor.rs:191`](../../src-tauri/src/core/webimport/extractor.rs#L191)

- Chỗ tinh vi nhất: khớp tham lam để nav 2 ký tự cướp mất thân bài, mất ~1900 ký tự trên `a03`.
  [`extractor.rs:499`](../../src-tauri/src/core/webimport/extractor.rs#L499)

**Đường ghi — chỗ vòng 1 đã hụt, đọc kỹ nhất ở đây**

- Một dòng này là toàn bộ khác biệt giữa "sửa được" và "màn hình nói dối".
  [`project.rs:372`](../../src-tauri/src/commands/project.rs#L372)

- Ghép văn bản từ khối đang giữ; không override nào ⇒ trùng đúng đầu ra Story 6.7.
  [`pipeline.rs:827`](../../src-tauri/src/core/segment/pipeline.rs#L827)

- Nơi override của người dùng gặp phán đoán của máy.
  [`pipeline.rs:799`](../../src-tauri/src/core/segment/pipeline.rs#L799)

**Luật bàn phím — hàm thuần, không nằm trong vỏ `wire`**

- Luật của phím `]`; tách ra hàm thuần vì `tests/**` không có harness Tauri.
  [`project.rs:1322`](../../src-tauri/src/commands/project.rs#L1322)

- Kiểm biên theo số khối THẬT, không tin số từ IPC.
  [`project.rs:1353`](../../src-tauri/src/commands/project.rs#L1353)

**Bề mặt tầng 2**

- Sáu phím đi qua handler DOM cục bộ — hợp âm trần toàn cục là phương án đã chết.
  [`ImportPreviewOverlay.vue:516`](../../src/ImportPreviewOverlay.vue#L516)

- Ba vạch lề suy từ hai cờ trực giao `kept`/`confirmed`.
  [`ImportPreviewOverlay.vue:468`](../../src/ImportPreviewOverlay.vue#L468)

- Kiểu dây khối cộng vị từ kiểm kiểu lúc chạy; adapter không bao giờ ném.
  [`config/project.ts:371`](../../src/config/project.ts#L371)

**Phép kiểm — đọc ba ca này là đủ hiểu vì sao story quay vòng một lần**

- Gỡ `.with_block_overrides()` khỏi `create_work` thì ca này đỏ; hai ca byte-for-byte cũ thì không.
  [`cleanup_contract.rs:539`](../../src-tauri/tests/cleanup_contract.rs#L539)

- Bảy mẫu thật; a01–a06 so ĐẲNG THỨC từng ký tự, không một sàn phần trăm.
  [`webimport_contract.rs:1087`](../../src-tauri/tests/webimport_contract.rs#L1087)

- Tám ca cho luật `[`/`]` và vòng đời override — đặt ở `tests/**`, không nội tuyến.
  [`project_contract.rs:27`](../../src-tauri/tests/project_contract.rs#L27)
