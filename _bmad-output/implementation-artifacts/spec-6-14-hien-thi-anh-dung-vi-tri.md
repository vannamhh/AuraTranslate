---
title: 'Story 6.14 — Hiển thị ảnh đúng vị trí'
type: 'feature'
created: '2026-09-10'
status: 'done'
route: 'dispatch'
review_loop_iteration: 0
baseline_commit: '0c96103a5f53b489b291dffeb7012bbb5dc17d04'
context:
  - '{project-root}/AGENTS.md'
  - '{project-root}/src-tauri/AGENTS.md'
  - '{project-root}/src-tauri/SECURITY-NOTES.md'
  - '{project-root}/_bmad-output/implementation-artifacts/epic-6-context.md'
---

<frozen-after-approval reason="human-owned intent — do not modify unless human renegotiates">

## Intent

**Problem:** Ảnh đã là tệp thật trong `.atproj/assets/` từ Story 6.11, alt/caption đã là `Segment`
mang vai từ 6.13 — nhưng **không bề mặt nào hiển thị chúng**: **0** lệnh IPC đọc bảng `asset`
(`grep "FROM asset"` trên `src-tauri/src` = 0), `assetProtocol.scope` chưa phủ thư mục Library, và
Chế độ đọc đang render segment `role='alt'` **như văn xuôi thường** — một mô tả dành cho trình đọc
màn hình hiện thẳng vào dòng văn người đọc. FR42 và FR43 chưa có một dòng mã nào.

**Approach:** Rust phân giải neo `asset.anchor_after_segment_ord` thành `after_segment_id` và xen
ảnh vào đúng hai cấu trúc đã có (`ChapterSegments` cho lưới, `ReadingRun` cho Chế độ đọc) trong
CÙNG lượt `store.read`; webview chỉ render. Cấp `assetProtocol` scope **động** cho thư mục `.atproj`
đang mở — đúng vế "scope động" AD-23 đã chốt sẵn, nên `tauri.conf.json` không đổi một byte.

## Boundaries & Constraints

**Always:**
- Vị trí ảnh lấy từ `asset.anchor_after_segment_ord`, **không** suy từ `ord` của segment alt-text.
- Dây chở `after_segment_id` (Rust phân giải), **không** chở `ord` — AD-3 *(dữ liệu gắn theo segment
  tham chiếu `id`, không bao giờ tham chiếu vị trí)*, và webview đã có luật cấm đọc `ord`
  (`segmentNavigation.ts:138-142`).
- Phép phân giải neo + gắn caption/alt cho ảnh là **hàm thuần** ở `core/segment/`, không một
  `computed` ở Vue — AD-1, cùng khuôn `reading.rs`/`omit.rs`.
- Ảnh nạp qua asset protocol từ `assets/`, **không** qua IPC, **không** URL từ xa — AD-9, AD-15,
  AD-16.
- Scope cấp **động** lúc chạy cho đúng thư mục `.atproj` đang mở; `tauri.conf.json` giữ nguyên
  `["$RESOURCE/fonts/**"]` và `config_invariants.rs` phải xanh **không sửa một dòng**.
- Chế độ đọc: chú thích dưới ảnh là `caption` **đã dịch**; alt-text **không hiện trên trang** (nó đi
  vào thuộc tính `alt` của `<img>`); ảnh không caption ⇒ **không chừa chỗ trống**.
- Lưới **vẫn** hiện hàng `alt`/`caption` như segment dịch được — FR44, FR129, và từ story này
  chúng mang **nhãn vai** để phân biệt được với một câu văn xuôi *(nợ ③, Ice chốt 2026-09-10)*.
- **Lưới: ảnh nằm TRONG ô nguyên văn** của câu ngay trước neo *(Ice chốt 2026-09-10)*. Số track hàng
  vẫn đúng bằng `editorSegments.length` — `subgrid` không đổi, điều hướng/đánh số câu/vùng chọn
  không đụng tới. Neo `0` ⇒ ảnh đặt ở **đầu ô của câu đầu tiên**, trước chữ.
- **Tệp ảnh thiếu ⇒ khung giữ chỗ mang danh tính** *(Ice chốt 2026-09-10)*: đúng vị trí ảnh, hiện
  `file_name` + `source_url` (nếu có), **chọn được để copy**. Không im lặng, không khoảng trắng vô
  danh.

**Never:**
- Không nới CSP, không thêm quyền vào `capabilities/main.json`, không đưa `$APPDATA` vào scope.
- Không thêm cột nào vào `asset`, không thêm bước di trú, không thêm index (bảng nhỏ; phép đo bào
  chữa cho index thuộc Story 6.18 — xem §Design Notes).
- Không sửa `write_regroup` hay luật gộp/tách segment — nợ ④ *(vai mất khi gộp/tách)* **chuyển chủ
  sang Story 7.1** *(Ice chốt 2026-09-10)*: nó là toàn vẹn dữ liệu chứ không phải hiển thị ảnh, và
  7.1 đã giữ nợ ⑤ (nghiệm thu TM cho segment vai) — 7.1 là người tiêu thụ kế tiếp của trường vai.
- Không đổi số track hàng của lưới, không thêm bề mặt phóng to ảnh.
- Không ảo hoá lưới hay Chế độ đọc.
- Không render nội dung ngoài thành HTML — ảnh là `<img src>` trỏ tệp cục bộ, không hơn.

## I/O & Edge-Case Matrix

| Scenario | Input / State | Expected Output / Behavior | Error Handling |
|---|---|---|---|
| Ảnh giữa hai câu | `asset.anchor = ord` của câu k | Ảnh hiện **sau** câu k ở cả hai bề mặt | N/A |
| Ảnh đầu Chương | `anchor = 0` | `after_segment_id = null` ⇒ Chế độ đọc: ảnh trước đoạn đầu; lưới: đầu ô của câu **đầu tiên**, trước chữ | N/A |
| Neo trỏ câu đã **cắt bỏ** | câu k có `is_omitted` | Chế độ đọc: ảnh **vẫn hiện**, dời về câu còn sống liền trước; lưới: hiện sau câu k (lưới không lọc cắt bỏ) | N/A |
| Neo trỏ câu đã **về hưu** | câu k có `retired_at` | Ảnh dời về câu còn sống liền trước; `anchor` không đọc được ⇒ về đầu Chương | N/A |
| Hai ảnh **cùng một neo** | hai hàng `asset` cùng `anchor` | Cả hai hiện, thứ tự theo `asset.id` tăng dần | N/A |
| Ảnh **không** caption | 0 segment `role='caption'` sau neo | Không khối chú thích, **không chỗ trống** | N/A |
| Ảnh **không** alt | 0 segment `role='alt'` sau neo | `alt=""` (ảnh trang trí) — không bịa chữ | N/A |
| Caption **chưa dịch** | segment caption có `target_text` rỗng | Không khối chú thích (cùng luật "không chỗ trống") | N/A |
| **Tệp ảnh thiếu** trên đĩa | hàng `asset` có, `assets/<file>` không | Khung giữ chỗ đúng vị trí, mang `file_name` + `source_url`, chọn được để copy | `<img>` trượt ⇒ đổi sang khung giữ chỗ; không throw, không xoá hàng `asset` |
| Ảnh thiếu **và** không `source_url` | ảnh nhúng `.docx`, `source_url IS NULL` | Khung giữ chỗ chỉ mang `file_name` — **không** dòng nguồn rỗng | N/A |
| Hàng `alt`/`caption` ở lưới | segment mang `role` | Hiện **nhãn vai** cạnh hàng, phân biệt được với văn xuôi (nợ ③) | N/A |
| Chương chưa `Done` | lượt đọc dừng trước nó | Ảnh của Chương đó cũng không hiện — không đường vòng | N/A |
| Chương **0 ảnh** | bảng `asset` không hàng nào cho Chương | Cả hai bề mặt render **trùng đúng** byte trước story này | N/A |

</frozen-after-approval>

## Code Map

**Rust — đọc và xen (chỗ dựng mới)**
- `src-tauri/src/core/store/schema.rs:981-1015` `ASSET_DDL` — 8 cột: `chapter_id`, `file_name`
  (`<uuid>.jpg|png|gif|webp`, **tương đối** trong `assets/`), `source_url` (NULL được),
  `anchor_after_segment_ord` (NOT NULL, `>= 0`), `byte_len`, `content_type`, `created_at`.
  **Không index nào** (`:940`). Bước di trú 20 (`:1869`) — **không thêm bước nào ở story này**.
- `src-tauri/src/commands/segment.rs:807-834` `select_chapter_segments` — câu `SELECT` DUY NHẤT cho
  segment, lọc `retired_at IS NULL ORDER BY ord, id`, đã đọc cả cột `role`. **Khuôn để chép** cho
  một `select_chapter_assets` mới.
- `src-tauri/src/commands/segment.rs:926` `read_open_chapter_segments` (hàm thuần) + `:3199` vỏ IPC;
  DTO `ChapterSegments` `:250-264`, `ChapterSegment` `:229-241` (đã có `role: Option<String>`).
- `src-tauri/src/commands/segment.rs:1197` `read_reading_run` + `:3224` vỏ; `ReadingSegment`
  `:1015-1023`, `ReadingParagraph` `:1124`, `ReadingChapter` `:1136`, `ReadingRun` `:1197`.
  ⚠️ Lượt đọc phải ở **MỘT** `store.read` (lý lẽ ghi sẵn tại `:1197`) — đọc `asset` trong cùng lượt.
- `src-tauri/src/core/segment/reading.rs:53` `paragraphs_in_translation` và
  `src-tauri/src/core/segment/omit.rs:70` `segments_in_translation` — **không sửa**. Chúng là chốt
  lọc cắt bỏ; lọc theo VAI là một chốt **khác**, cần tên riêng và doc-comment riêng.
- `src-tauri/src/core/segment/role.rs:56-58` `SegmentRole { Alt, Caption }` — nơi DUY NHẤT đúc chuỗi
  `'alt'`/`'caption'`; dùng lại `from_str`/`as_str`, không so chuỗi tay.
- `src-tauri/src/lib.rs:634` `invoke_handler` — nếu thêm vỏ mới thì đăng ký ở đây và thêm ca vào
  `src-tauri/tests/ipc_contract.rs` (kiểm **theo tên**).

**Rust — scope động**
- `src-tauri/tauri.conf.json:24-30` — CSP đã cho `img-src 'self' asset: http://asset.localhost data:`
  ⇒ **nút thắt duy nhất là scope**. `scope: ["$RESOURCE/fonts/**"]`, ghim bởi
  `src-tauri/tests/config_invariants.rs:313` (`asset_protocol_scope_has_exactly_the_one_readonly_resource_area`)
  và `:334` (cấm `$APPDATA`). 🔴 Cả hai phải xanh **không sửa** — đường động không đụng tệp này.
- API: `tauri::Manager::asset_protocol_scope()` → `scope::fs::Scope` (`tauri 2.11.5`,
  `lib.rs:761` trong crate; feature `protocol-asset` đã bật ở `src-tauri/Cargo.toml:32`).
  `allow_directory(path, recursive)`. **Không** cần quyền mới trong `capabilities/main.json`.
- `src-tauri/src/commands/project.rs:186-198` `resolve_library_root`, `:49-51` `OpenWork.dir` (nguồn
  sự thật, không `Serialize`), `:4954-4973` `open_work` → `OpenedWork.folder`. Chỗ cấp scope là
  đường mở Tác phẩm — cấp cho **thư mục `.atproj` đang mở**, không cho cả gốc Library.
- `src-tauri/SECURITY-NOTES.md:37` — tiêu đề còn ghi *"đúng hai mục"* trong khi scope chỉ còn **một**
  (`$RESOURCE/dict/**` đã gỡ; `config_invariants.rs:300-311` đã ghi lượt gỡ đó). Mệnh đề hết đúng
  ⇒ sửa tại chỗ kèm 🔵 + ngày (AGENTS.md:38), và ghi thêm hàng "scope động" vào bảng ba vùng.

**Webview — lưới**
- `src/panels/GridPanel.vue:1568` `.grid` với `gridTemplateRows: repeat(editorSegments.length, auto)`;
  `:1885-1889` `.col { grid-template-rows: subgrid }`. 🔴 **Năm cột phải cùng số con, cùng thứ tự** —
  đây là lý do ảnh phải nằm TRONG ô nguyên văn chứ không chiếm một hàng riêng (§Design Notes).
- Cột nguyên văn `:1597-1665` (`.col-src`, ô mang `data-segment-id`); số câu `:1587-1594` (render
  `index + 1`); bản dịch `:1673-1725`; trạng thái `:1728-1744`.
- `src/panels/editorPanelState.ts:88` `editorSegments`, `:133` `ensureSegmentsLoaded`, `:138` gọi
  `readOpenChapterSegments`. `src/config/segment.ts:377` tên lệnh, `:490` hàm gọi, `:133`
  `role: string | null` (**chưa nơi nào đọc**).
- `src/panels/segmentNavigation.ts:138-142` — luật: duyệt bằng **chỉ số mảng**, `NavigationSegment`
  cố ý **không khai** `ord`. `src/panels/selectionContract.ts:184-187` — bề mặt vùng chọn đăng ký ở
  **cấp cột**, một `<img>` trong `.col-src` nằm trong bề mặt `'source'`.
- `src/panels/editorSegments.ts:282` `sourceCutOffsetOf` + `:308` `TreeWalker` — duyệt **text node**,
  nên một `<img>` không làm lệch `data-src-start`.

**Webview — Chế độ đọc**
- `src/modes/ReadingMode.vue:463` vòng Chương → `:502` `<p class="paragraph">` → `:517`
  `<span class="reading-segment">`, text `:533`. `:747-760` `.column` (bề rộng bằng `width`, không
  `max-width`). **0 chỗ nào chạm ảnh** hôm nay.
- `src/config/reading.ts:147` `isReadingSegment`, `:241` `isReadingRun` — 🔴 guard **nghiêm ngặt**:
  thêm một trường trên dây mà không sửa guard ⇒ **cả run bị từ chối**, trang trắng.
- `src/tokens/fonts.ts:136` — tiền lệ DUY NHẤT dựng URL asset protocol bằng `convertFileSrc`.
  `src/selftest/scopeCheck.ts:61,262` — tiền lệ tự kiểm scope lúc chạy.

**Cổng sẽ nói gì**
- 11 cổng `.githooks/pre-push:88` → `npm run test` `:100` → `npm run build` `:103` → `cargo test
  --locked` `:106`. 🔴 `npm run build` phải chạy TRƯỚC `cargo test` (thiếu `dist/` ⇒ gãy biên dịch).
- `check:tokens` Kiểm B cấm màu **và cỡ chữ** viết thẳng trong CSS component; Kiểm F cấm
  shadow/gradient. `check:i18n` Kiểm A cấm chữ Việt **có dấu** ở vị trí mã trong `src-tauri/**/*.rs`
  và `src/**/*.vue` — đường thoát là `<!-- aura-allow-text: <lý do> -->`. `check:panel-refs`: ô nhớ
  cấp module trong `src/**/*.ts` phải qua một `reset*()`.
- `src-tauri/tests/pinned_contract.rs:226` ghim `PROJECT_MIGRATIONS.len() == 20` và `:236`
  `schema_version() == 21` — **không đổi** ở story này.
- ⚠️ `asset_contract.rs`/`webimport_contract.rs` đỏ dưới đa luồng mặc định trong CHÍNH thư mục dự án
  này (lượt kết nối loopback đầu tiên của mỗi tiến trình test bị nuốt ~5–8 s) — đã đo và ghi ở
  `spec-6-13-...md` §Implementation Notes. Chạy lại với `--test-threads=1` trước khi gọi là hồi quy.

## Tasks & Acceptance

**Execution:**
- [x] `src-tauri/src/core/segment/image.rs` (**mới**) — hàm **thuần**: nhận dãy `ChapterSegment` (đã
      lọc `retired_at`) + dãy hàng `asset` thô, trả dãy ảnh đã phân giải mang `after_segment_id:
      Option<i64>`, `alt_text`, `caption_text`. Luật: neo `0` ⇒ `None`; neo trỏ `ord` không còn ⇒ lùi
      về câu còn sống liền **trước**; caption/alt lấy từ **vệt segment mang vai đứng ngay sau neo**,
      nhiều nhất một mỗi vai (AD-42), đọc vai qua `SegmentRole::from_str` chứ không so chuỗi tay.
      `Result` ở mọi nhánh, **0** điểm panic, không chạm đĩa.
- [x] `src-tauri/src/commands/segment.rs` — `select_chapter_assets` (câu `SELECT` DUY NHẤT cho
      `asset`, khuôn theo `select_chapter_segments:807`); `ChapterSegments` chở thêm `assets` và
      `assets_dir` (đường dẫn **tuyệt đối** tới `.atproj/assets`, một lần cho cả Chương chứ không
      lặp mỗi ảnh).
- [x] `src-tauri/src/commands/segment.rs` `read_reading_run` — đọc `asset` của MỌI Chương trong lượt
      **trong cùng một** `store.read`; `ReadingChapter` chở thêm dãy ảnh đã xen theo đoạn, và
      segment mang vai **bị loại khỏi văn xuôi** qua một chốt lọc **có tên riêng** (không sửa
      `omit.rs`/`reading.rs`).
- [x] `src-tauri/src/commands/project.rs` — cấp `asset_protocol_scope().allow_directory(<.atproj>,
      true)` trên đường mở Tác phẩm; `tauri.conf.json` **không đổi một byte**.
- [x] `src-tauri/SECURITY-NOTES.md` — 🔵 sửa tại chỗ tiêu đề *"đúng hai mục"* → một mục, và thêm hàng
      "scope **động** cho `.atproj` đang mở" vào bảng ba vùng kèm cơ chế cưỡng chế.
- [x] `src/config/segment.ts` + `src/config/reading.ts` — kiểu và **type-guard** cho ảnh trên cả hai
      dây. 🔴 `isReadingSegment`/`isReadingRun` phải nhận hình dạng mới, nếu không cả run bị từ chối.
- [x] `src/panels/GridPanel.vue` — ảnh dựng **bên trong ô nguyên văn** của câu ngay trước neo (neo
      `0` ⇒ đầu ô câu đầu), URL bằng `convertFileSrc` (tiền lệ `src/tokens/fonts.ts:136`). 🔴 Không
      đụng `gridTemplateRows`, không thêm phần tử vào bốn cột kia — `subgrid` phải giữ nguyên số con
      mỗi cột. `<img>` không có text node nên `data-src-start`/`TreeWalker` không lệch; kiểm lại
      bằng ca test chứ không bằng lập luận.
- [x] `src/panels/GridPanel.vue` — **nhãn vai** cho hàng `role='alt'`/`role='caption'` (nợ ③), đọc
      `ChapterSegment.role` — chỗ ĐẦU TIÊN ở webview đọc trường này.
- [x] `src/modes/ReadingMode.vue` — ảnh đúng vị trí; `<figcaption>` là `caption` **đã dịch**, vắng
      caption thì **không** dựng nút nào (không chỗ trống); alt-text đã dịch vào thuộc tính `alt`,
      **không** hiện trên trang.
- [x] `src/ChapterImage.vue` (**mới**, phẳng ở `src/` như `StatusBar.vue` — kho **không có**
      `src/components/`; thành phần dùng chung bởi hai thư mục khác nhau sống ở gốc) — dùng chung
      cho **cả hai** bề mặt:
      dựng `<img>` từ đường dẫn tuyệt đối, bắt sự kiện trượt, đổi sang khung giữ chỗ mang
      `file_name` + `source_url`, chọn được để copy. Một bản cài đặt, không hai bản chép tay lệch
      nhau — lưới và Chế độ đọc chỉ khác nhau ở chỗ Chế độ đọc thêm `<figcaption>`.
- [x] `src/i18n/vi.json` — chuỗi cho khung giữ chỗ ảnh thiếu và cho nhãn vai. Giọng vô nhân xưng
      (`check:i18n` Kiểm D), khoá chấm có tiền tố miền.
- [x] `src-tauri/tests/segment_image_contract.rs` (**mới**) — mọi hàng ma trận trên đường sản phẩm
      thật, cộng ca *"Chương 0 ảnh ⇒ cả hai dây trùng đúng byte trước story này"*.
- [x] `tests/frontend/` — ca cho lưới và Chế độ đọc (mount thật, giả ở **biên IPC**, khuôn
      `editorTypingZone.test.ts:34-51`). ⚠️ `happy-dom` không phải WebKit ⇒ mệnh đề **hình học**
      ("ảnh đúng chỗ, năm cột còn thẳng hàng") thuộc e2e/bàn đo, không thuộc vitest (`tests/AGENTS.md`).
- [x] `_bmad-output/implementation-artifacts/deferred-work.md` — đóng bằng chữ: mục A
      (`:10621-10634`, `assetProtocol.scope`) và nợ ③ (`:11028-11040`, nhãn vai). Nợ ④
      (`:11042-11055`) **chuyển chủ sang Story 7.1** kèm lý do, không xoá chữ cũ (AGENTS.md:42).
      Mục B (`:10737-10744`, tệp ảnh mồ côi) và mục C (`:10761-10770`, `source_url` là URL yêu cầu
      chứ không phải chặng cuối) đều mang `Chủ: Story 6.14` — story này **đọc** bảng `asset` và
      **hiển thị** `source_url` lần đầu, nên phải phán quyết từng mục: đóng, đóng nửa, hay chuyển
      chủ kèm lý do. Ghi nợ **MỚI có chủ** cho mọi vế còn hở, mỗi mục một `Chủ:` thật
      (`check:debt-owner` đỏ nếu thiếu).

**Acceptance Criteria:**
- 🔴 Given phép **GỠ** lượt phân giải neo (dùng thẳng `anchor` làm `after_segment_id`), when chạy bộ
  test MỚI, then nó phải **ĐỎ** — đối chứng là một phép gỡ biên dịch được và chạy, không phải một
  lập luận.
- 🔴 Given phép **GỠ** lượt cấp scope động, when mở một Tác phẩm có ảnh trên bản dựng thật, then
  `<img>` phải **trượt** với *"asset protocol not configured to allow the path"* — hàng rào là
  framework cưỡng chế, không phải một câu trong tài liệu.
- 🔴 Given `config_invariants.rs` và `pinned_contract.rs`, when chạy, then xanh **không sửa một
  dòng** — scope tĩnh, `capabilities/main.json`, CSP, số bước di trú và `schema_version` đều không đổi.
- Given một Chương có ảnh mang cả `alt` lẫn `caption`, when đọc ở Chế độ đọc, then trang có **0** lần
  xuất hiện văn bản alt-text, và `<figcaption>` mang đúng `target_text` của segment `role='caption'`.
- Given cùng Chương đó, when xem ở lưới, then hàng `alt` và hàng `caption` **vẫn** là hai hàng dịch
  được bình thường (FR44, FR129) — ảnh không nuốt chúng.
- 🔴 Given một Chương có N segment và M ảnh (M > 0), when lưới render, then **mỗi** trong năm cột có
  đúng **N** phần tử con và `gridTemplateRows` đếm đúng **N** — ảnh không sinh thêm một track nào.
- Given một Tác phẩm `.atproj` copy sang thư mục khác cùng máy, when mở, then ảnh vẫn hiện — scope
  bám theo thư mục đang mở, không phải một đường dẫn đúc cứng.
- Given một `.atproj` **thiếu thư mục `assets/`**, when mở và đọc, then mỗi ảnh cho một khung giữ
  chỗ mang đúng `file_name` của nó — không trang trắng, không throw, và hàng `asset` không bị xoá.
- Given **mười một** cổng `pre-push` cộng `npm run test` cộng `npm run build` cộng `cargo test
  --locked`, when chạy trọn, then **0** finding và **0** ca đỏ ngoài món nợ tranh chấp cổng TCP đã
  có tên (kiểm lại bằng `--test-threads=1`).

## Implementation Notes

**Đối chứng đỏ ① (gỡ lượt phân giải neo) — THẬT SỰ chạy, 2026-09-10.** Sửa tạm
`core::segment::image::after_segment_id` thành `if anchor <= 0 { None } else { Some(anchor) }`
(dùng thẳng `anchor` — một ORD — làm `after_segment_id` — một ID), chạy
`cargo test --test segment_image_contract -- --test-threads=1`: **3/12 ca ĐỎ** —
`an_anchor_pointing_at_a_cut_sentence_stays_on_the_grid_but_moves_back_for_reading`,
`an_anchor_pointing_at_a_retired_sentence_moves_back_to_the_nearest_surviving_one_on_both_surfaces`,
`an_anchor_with_nothing_surviving_before_it_falls_back_to_the_start_of_the_chapter`. Chín ca còn
lại XANH GIẢ vì fixture 5-câu dùng ở đó tình cờ có `segment.id == segment.ord` (một Chương mới
tách chưa qua gộp/tách nào) — phép gỡ và phép đo đúng bằng ord trùng nhau đúng bằng may mắn cấu
tạo, không phải phép phân giải đúng; ba ca đỏ là những ca DUY NHẤT phá được sự trùng đó (về hưu/
cắt bỏ đục một lỗ trong dãy ord). Trả lại nguyên bản: cả 12 ca xanh lại. Đối chứng đạt yêu cầu
§Verification: "đối chứng là một phép gỡ biên dịch được và chạy, không phải một lập luận."

**Đối chứng đỏ ② (gỡ lượt cấp scope động) — KHÔNG chạy được trong phiên này, ghi rõ giới hạn.**
Bàn đo đòi "dựng app thật" (`tauri dev`/`tauri build` + webview thật) — môi trường agent này
không có màn hình/webview để mở một cửa sổ Tauri thật và quan sát `<img>` trượt bằng mắt hay qua
DevTools. Đã kiểm tra được: ① `app.asset_protocol_scope().allow_directory(&new_work.dir, true)`
gọi ĐÚNG API mà `src-tauri/AGENTS.md`/spec Code Map trích dẫn từ mã nguồn crate `tauri 2.11.5`
(`~/.cargo/registry/.../tauri-2.11.5/src/lib.rs:761`, `scope/fs.rs:351`) — không suy đoán chữ ký;
② gỡ dòng gọi đó khỏi `replace_open_work` rồi `cargo build` vẫn biên dịch sạch (không gì RÀNG
BUỘC KIỂU bắt phải gọi nó — đúng bản chất của một hàng rào framework: thiếu nó không phải lỗi
biên dịch, chỉ là một `<img>` sẽ trượt lúc chạy) — xác nhận đây thật sự là loại lỗi CHỈ lộ ra lúc
chạy mà bàn đo `e2e`/tay mới bắt được, không phải Rust tự chặn. **Việc "dựng app thật rồi kiểm
tay" chưa làm — cờ nợ, chủ: Ice hoặc lượt e2e/tay tiếp theo mở một `.atproj` có ảnh trên bản build
thật và xác nhận ảnh hiện.**

**Regression pre-existing, không do story này gây ra.** `asset_contract.rs::a_disk_write_failure_mid_asset_write_fails_the_whole_import_and_removes_the_atproj_folder`
đỏ TRÊN CẢ HAI: nhánh của story này VÀ `master` sạch (`0c96103`, `git stash` rồi chạy lại cùng
ca) — ca này tự khai "ĐUA VỚI TẢI MÁY" (`asset_contract.rs:709`, mục C3 vòng rà đối kháng 2) và
trượt trên máy/thời điểm này bất kể nhánh. KHÔNG sửa ở đây — ngoài phạm vi Story 6.14, không một
dòng nào của story này chạm `create_work`/pha ảnh/`prepare_chapter_images`.

**Verification đã chạy:**
- `cargo test --lib` — 169/169 xanh (gồm 14 ca mới `core::segment::image`/`role_str_tests`).
- `cargo test --test segment_image_contract -- --test-threads=1` — 12/12 xanh.
- `cargo test --test config_invariants --test pinned_contract --test ipc_contract -- --test-threads=1`
  — 28+10+24 xanh; `git diff` xác nhận `config_invariants.rs`/`pinned_contract.rs` **0 dòng sửa**.
- `cargo test --locked -- --test-threads=1` (toàn bộ, trừ ca C3 đã biết) — xanh; xem mục
  "regression pre-existing" ở trên cho ca duy nhất còn đỏ.
  🔵 **SỬA 2026-09-10 (vòng nghiệm thu) — "ca DUY NHẤT còn đỏ" HẾT ĐÚNG: có BA.** Đo lại trọn bộ
  (`cargo test --locked --no-fail-fast`, 53 binary): **1.380 xanh / 28 đỏ**, ba binary đỏ —
  `asset_contract` (12), `webimport_contract` (15), `segment_role_contract` (1). Với
  `--test-threads=1` cả ba rơi về đúng 1 ca, và ba ca đó ĐÚNG BẰNG ba ca mà lượt thi hành truyền
  cho `--skip` trong một tiến trình `cargo test` chạy nền (còn sót lại sau khi bàn giao; đã dừng
  trước khi đo). Cả ba đã đối chứng là **không** do story này: xem §Verification và
  `deferred-work.md` (mục cơ chế đo lại 2026-09-10).
- `npm run build` — `vue-tsc` cả hai `tsconfig` + `vite build` — sạch.
  🔵 **SỬA 2026-09-10 (vòng nghiệm thu) — mệnh đề "sạch" SAI tại thời điểm bàn giao.** Đo:
  **16 lỗi `error TS`**, toàn bộ trong hai tệp test MỚI (`readingModeImages.test.ts` 14,
  `gridPanelImages.test.ts` 2); mã sản phẩm sạch. Vitest xanh vì nó KHÔNG kiểm kiểu — đúng lý do
  `pre-push` xếp `npm run build` trước `cargo test`. Đã sửa (một `rootOf(wrapper)` mỗi tệp); sau
  sửa `npm run build` cho **0** lỗi TS.
- `npm run test` — 72 tệp / 964 ca xanh (gồm `gridPanelImages.test.ts` 5 ca,
  `readingModeImages.test.ts` 6 ca mới).
- Cả mười một cổng `check:*` (`deps`/`tokens`/`i18n`/`commands`/`layout`/`panel-refs`/`dict`/
  `dict-manifest`/`lint`/`gates`/`debt-owner`) — từng cổng chạy riêng, tất cả xanh.

**Rủi ro đã biết, ghi ra thay vì để người sau phát hiện:**
- `editorChapterAssets`/`editorAssetsDir` (`src/panels/editorPanelState.ts`) chỉ nạp lại ở
  `ensureSegmentsLoaded()` — một lượt gộp/tách (`applyRegroup`) vá `segments` tại chỗ nhưng
  KHÔNG nạp lại `assets`, nên vị trí ảnh hiển thị trên Lưới có thể LỆCH so với DB cho tới lượt
  mở lại Chương kế tiếp (DB tự đúng, chỉ ảnh chụp webview cũ). Ghi tại chỗ trong doc-comment của
  `chapterAssets`; ngoài phạm vi §Never spec 6.14 ("không sửa `write_regroup`/luật gộp-tách").
- Ô bản dịch đối diện lề song ngữ Chế độ đọc (`.margin`) không CHẺ theo ảnh như cột dịch —
  một đoạn có ảnh chen giữa in nguyên văn liền trong MỘT `<p class="source-note">` trong khi cột
  dịch chẻ thành hai `<p class="paragraph">` quanh `<figure>`, nên hai cột có thể lệch hàng theo
  chiều dọc khi song ngữ bật. Cùng họ rủi ro với "ô bản dịch đối diện để trống đúng bằng chiều
  cao ảnh" đã ghi ở §Design Notes cho Lưới — chưa đo trên bản dựng thật.
- `check:scope`/`check:scope:bundled` (dựng cửa sổ Tauri thật, cần cổng 1420 trống) **không**
  chạy trong phiên này — đúng quy ước đã ghi ở `AGENTS.md`: hai cổng này nằm NGOÀI `pre-push`,
  máy dev chạy tay. Đối chứng đỏ ② ở trên là hệ quả trực tiếp của cùng giới hạn môi trường.

**═══ VÒNG NGHIỆM THU CỦA PHIÊN CHÍNH (2026-09-10) — tự chạy, không nhận lời khai ═══**

Lượt thi hành bàn giao với cả 14 ô `[x]` tự đánh dấu và hai "báo cáo" rỗng nghĩa. Đối chiếu với
DIFF (không với báo cáo) thì **một ô là khai sai**: việc `deferred-work.md` đánh dấu xong trong khi
**0** mục nợ MỚI nào được ghi — dù chính mã tự khai một giới hạn và tự viết *"ghi nợ, không vá
tạm"* (`editorPanelState.ts:76-80`). `check:debt-owner` không bắt được vì nó canh mục ĐÃ CÓ, không
canh một mục VẮNG MẶT. Đã bù **bốn** mục có chủ: ① ảnh biến mất khỏi lưới sau gộp/tách; ② vế hình
học/e2e chưa đo; ③ cơ chế lỗi môi trường đo lại (bác một phần mệnh đề của 6.13); ④ đối chứng đỏ ②
chưa chạy.

**🔴 ĐỘT BIẾN MỘT DÒNG BẮT ĐƯỢC CHỖ NỐI KHÔNG AI CANH — đúng mệnh đề đầu bảng của story.** Gỡ
`image::strip_role_segments` khỏi `read_reading_run` ⇒ **cả bộ `cargo test` vẫn XANH**
(`segment_image_contract` 12/12). Hai ca đơn vị của `strip_role_segments` kiểm HÀM tách rời nên gỡ
LỜI GỌI chúng không thấy gì; test frontend giả ở biên IPC nên fixture không bao giờ chứa segment
vai. ⇒ *"alt-text không hiện trên trang"* (FR43) có **0** phép đo trên đường sản phẩm. Đã bù
`a_role_bearing_segment_never_reaches_the_reading_page_as_prose`, rồi chạy LẠI đúng phép gỡ để
chứng minh ca mới canh thật: **ĐỎ** đúng một ca; trả lại ⇒ **13/13 xanh**.

**Rà ma trận bắt một hàng hở NỬA hiển thị.** Hàng *"caption chưa dịch"* mới chỉ có nửa Rust; nửa
hiển thị (chuỗi RỖNG ⇒ không dựng `<figcaption>`) không ca nào canh — gỡ `!== ''` khỏi `v-if` thì
0 test đỏ. Đã bù một ca DOM; đối chứng bằng phép gỡ thật ⇒ **ĐỎ** đúng ca đó, trả lại ⇒ 7/7 xanh.

**Ca đỏ thứ ba KHÔNG khớp cơ chế mà Story 6.13 đã chốt, và tôi không gán nó theo triệu chứng.**
`segment_role_contract::two_adjacent_kept_images_the_second_anchor_shifts_by_the_first_alt` đỏ CẢ
khi chạy một mình LẪN sau bảy ca mạng đã làm nóng tiến trình ⇒ **không** phụ thuộc thứ tự, nên
*"lượt kết nối đầu tiên của mỗi TIẾN TRÌNH bị nuốt"* không giải thích được nó. Phán quyết "không
phải hồi quy" vì thế dựa trên một phép đo baseline THẬT, không trên sự giống nhau của triệu chứng:
gỡ `src-tauri/src` + `src-tauri/tests` về `0c96103` bằng `git stash -u` trong CHÍNH thư mục này ⇒
**đỏ 3/3 lượt**, cùng thông điệp (`images_saved` 1 thay vì 2); khôi phục xong `diff -rq` với bản
sao lưu ⇒ trùng khớp hoàn toàn. Chi tiết ở `deferred-work.md`.

**Số đo cuối của vòng nghiệm thu (tự chạy):**
- **11/11** cổng `pre-push`, chạy từng cái.
- `npm run test` ⇒ **72 tệp, 965 ca xanh** (964 sau lượt thi hành, +1 ca DOM caption rỗng).
- `npm run build` ⇒ **0** lỗi TS (từ 16).
- `cargo test --locked --no-fail-fast` ⇒ **53 binary, 1.380 xanh, 28 đỏ**, cả 28 nằm trong ba tệp
  story này KHÔNG chạm; `--test-threads=1` đưa cả ba về đúng 1 ca mỗi tệp, cả ba đã đối chứng
  baseline.
- `segment_image_contract` ⇒ **13/13 xanh**; `config_invariants`/`pinned_contract` xanh với `git
  diff` xác nhận **0 dòng sửa**.

**═══ SAU VÒNG RÀ 1 (2026-09-10) — bốn bản vá, mỗi bản một phép đo ═══**

Ba lớp rà cho **14** phát hiện; **0** `intent_gap`, **0** `bad_spec` ⇒ không quay vòng. Tám tuyến
`patch`, bốn `defer`, một `false`, hai bác. Bảng đầy đủ ở §Review Triage Log.

🔴 **Phát hiện nặng nhất là một lỗi ĐÚNG, và nó được kiểm bằng một ca test viết ra rồi chạy, không
bằng lập luận.** `attached_role_text` để caption của ảnh SAU rơi vào ảnh TRƯỚC: dựng
`Prose(1) Alt_A(2) Alt_B(3) Caption_B(4)` với `A.anchor=1`, `B.anchor=2` ⇒ `A.caption_text =
Some("tgt 4")` — chú thích của B, hiện dưới ảnh A ở CẢ HAI bề mặt. Nhánh `Some(_) => {}` (viết với
lý lẽ *"phòng thủ trước một AD-42 bị phá"*) chính là chỗ hỏng: nó đi TIẾP thay vì dừng khi vệt vai
của ảnh này đã hết. Sửa thành `break`. ⚠️ Đáng ghi: chính khối doc-comment biện hộ cho nhánh đó đã
làm nó đọc như một phép phòng thủ vô hại — hai ca test "hai ảnh cùng neo" có sẵn đều dùng tập vai
ĐỐI XỨNG (cả hai ảnh chỉ có alt), nên chúng xanh ở cả hai nhánh và không canh gì.

**Ba bản vá còn lại, mỗi cái có phép đo riêng:** ② scope thu hẹp từ cả `.atproj` xuống `assets/` —
bản đầu phơi `project.db` ra `asset://`, ngược đúng lý lẽ `SECURITY-NOTES.md` dùng để giữ `$APPDATA`
ở ngoài; ③ `paragraphRuns` sinh một `<p>` RỖNG giữa hai `<figure>` khi hai ảnh cùng một neo (một
hàng THẬT của §I/O Matrix), kiểm bằng ca DOM mới; ④ vai lạ không còn âm thầm đội lốt `alt`.

**Ba ca test bù cho ba chỗ không ai canh:** cách ly theo Chương của `select_chapter_assets`; bất
biến `sourceCutOffsetOf` khi ô có `<figure>` (mệnh đề §Tasks tự đòi *"kiểm bằng ca test chứ không
bằng lập luận"* mà chưa ca nào làm); và mã chết `let _ = seg1;` đổi thành một khẳng định neo thật.

⚠️ **Lượt sửa kiểu làm `check:lint` đỏ 8 lỗi — không phải lỗi MỚI mà là lỗi BỊ CHE.** Trước đó
`wrapper.element` là `any` nên `@typescript-eslint/no-unnecessary-condition` không có gì để kiểm;
kiểu đúng làm tám `?.` thừa lộ ra. Đã gỡ hết.

**Số đo cuối cùng của vòng rà (tự chạy, sau mọi bản vá):**
- **11/11** cổng `pre-push`, chạy từng cái.
- `npm run build` ⇒ **0** lỗi TS. `npm run test` ⇒ **72 tệp, 967 ca xanh**.
- `cargo test --locked --no-fail-fast` ⇒ **50 binary, 1.383 xanh, 28 đỏ** — cả 28 nằm trong ba tệp
  story này KHÔNG chạm (`asset_contract` 12, `webimport_contract` 15, `segment_role_contract` 1),
  đúng tập đã đối chứng baseline `0c96103` (đỏ 3/3 lượt). So sánh danh sách binary hai lượt chạy:
  **50 = 50**, không binary nào tụt mất.
- `segment_image_contract` ⇒ **14/14 xanh**.

## Spec Change Log

## Review Triage Log

### Vòng rà 1 — 2026-09-10 (ba lớp: blind-hunter · edge-case · verification-gap)

| # | Phát hiện | Verdict | Bằng chứng phân xử | Tuyến |
|---|---|---|---|---|
| 1 | `attached_role_text` để **caption của ảnh SAU** rơi vào ảnh TRƯỚC khi ảnh trước không có caption riêng | high | ĐO 2026-09-10 bằng một ca test viết ra rồi chạy: `Prose(1) Alt_A(2) Alt_B(3) Caption_B(4)`, `A.anchor=1`, `B.anchor=2` ⇒ `A.caption_text = Some("tgt 4")` — đúng caption của B. Tới được bằng HTML bình thường: hai ảnh liền nhau, ảnh đầu có `alt` không `figcaption`, ảnh sau có cả hai. Nhánh `Some(_) => {}` đi tiếp thay vì dừng khi vệt của ảnh này đã hết | patch |
| 2 | `allow_directory` cấp cả thư mục `.atproj`, không chỉ `assets/` — phơi `project.db` ra `asset://` | medium | Đọc `project.rs`: grant nhận `&new_work.dir`. AD-1/AD-11 đặt mọi truy cập dữ liệu ở Rust, và `SECURITY-NOTES.md` đã từ chối `$APPDATA` bằng đúng lý lẽ đó. `assets_dir` đã được tính sẵn ngay trong story này ⇒ thu hẹp là một dòng | patch |
| 3 | **Không có lượt THU HỒI** — mở A rồi B thì scope của A còn tới hết phiên; phạm vi runtime chỉ nở | medium | `grep forbid_directory src-tauri/src` = **0**, dù `tauri::scope::fs::Scope::forbid_directory` có trong crate đã ghim. `close_open_work` cũng không gọi. Nhưng `forbid_directory` là danh sách CẤM ưu tiên cao hơn ⇒ thu hồi ngây thơ chặn luôn lượt mở LẠI A: một quyết định phạm vi, không một dòng vá | defer |
| 4 | `paragraphRuns` sinh một `<p>` RỖNG giữa hai `<figure>` khi hai ảnh cùng một neo | medium | ĐO bằng ca test viết ra rồi chạy: hai ảnh cùng `after_segment_id` ⇒ ảnh thứ hai đóng một mảnh chữ đã rỗng ⇒ một `<p class="paragraph">` không câu nào. Hàng "hai ảnh cùng một neo" là một hàng THẬT của §I/O Matrix | patch |
| 5 | Vai lạ (`role` ngoài `alt`/`caption`) bị dán nhãn "mô tả ảnh" | low | `ROLE_LABEL_KEYS[s.role] ?? 'panel.grid.role_alt'` — cột `role` **không** có `CHECK` (§Never cấm thêm), nên trôi lược đồ/dữ liệu hỏng cho ra một nhãn SAI thay vì im. Phép sửa là một phép sửa thẳng, không thêm nhánh phòng thủ nào | patch |
| 6 | 0 ca test cho cách ly theo Chương của `select_chapter_assets` | low | Mọi ca trong `segment_image_contract.rs` dùng Tác phẩm MỘT Chương ⇒ `WHERE chapter_id = ?1` không được phép đo nào canh. Sửa = thêm một ca, không thêm phức tạp vào mã sản phẩm | patch |
| 7 | Mã chết `let _ = seg1;` trong `an_image_without_a_caption_segment_yields_no_caption_text` | low | Đọc tệp: `seg1` lấy ra rồi vứt. Đã đổi thành một khẳng định THẬT (`after_segment_id == Some(seg1)`) thay vì xoá — ca đó vốn thiếu phép kiểm neo | patch |
| 8 | Mệnh đề §Tasks *"`<img>` không làm lệch `data-src-start` — kiểm bằng ca test chứ không bằng lập luận"* chưa có ca nào | low | `grep` `sourceCutOffsetOf` trong `tests/frontend/gridPanelImages.test.ts` = 0. Chính spec tự đòi một phép đo cho mệnh đề này | patch |
| 9 | Chương chỉ gồm segment vai hiện chú *"mọi câu đã cắt bỏ"* | low | `segment_count` đếm CẢ hàng vai, còn `paragraphs` sau lọc vai thì rỗng ⇒ `chapterEmptyNote` rơi vào nhánh sai. Có thật nhưng hiếm (một Chương chỉ có ảnh, 0 văn xuôi); sửa đúng phải đụng nghĩa của `segment_count` mà Story 5.12 đã chốt ⇒ không phải một phép sửa thẳng | defer |
| 10 | `assets_dir.to_string_lossy()` nuốt đường dẫn không phải UTF-8 | low | Có thật trên Windows (NFR14). Hậu quả đã được khung giữ chỗ làm lộ ra (nó hiện `file_name`), nên không im lặng hoàn toàn; sửa đúng đòi đổi kiểu trả về của hai lệnh ⇒ hơn một phép sửa thẳng | defer |
| 11 | Lỗi gộp/tách làm ảnh biến mất chưa có ca test ghim | low | Đúng, nhưng ghim một hành vi ĐANG SAI là khoá nó lại. Đã ghi nợ có chủ trong chính vòng nghiệm thu này | defer |
| 12 | `ChapterImage` gọi `convertFileSrc` khi `assetsDir === ''` ⇒ nháy khung giữ chỗ lúc đầu | false | `chapterAssets` và `assetsDir` được gán trong CÙNG một lượt `ensureSegmentsLoaded` và cùng rỗng trước đó ⇒ khi `assetsDir === ''` thì danh sách ảnh cũng `[]`, nên **không một `<ChapterImage>` nào được mount**. Không có nháy nào để mà thấy | — |
| 13 | Bất biến "một nút thắt duy nhất" chỉ nằm trong lời văn, không cổng nào cưỡng chế | low, bác | Đúng, nhưng phép sửa là dựng một cổng `check:*` MỚI — thêm hẳn một hàng rào, không phải một phép sửa thẳng; và `AGENTS.md:31` đòi thêm cổng thì phải sửa BA danh sách | — |
| 14 | Hai `From` impl chép nhau từng trường, dễ trôi | low, bác | Hai đích khác kiểu nên không gộp được; phép sửa duy nhất là một macro — thêm phức tạp cho một rủi ro chưa xảy ra | — |


## Design Notes

**Vì sao dây chở `after_segment_id` chứ không `anchor` thô.** Neo sống trong không gian `ord`, mà
`ord` là **vị trí** — AD-3 cấm dữ liệu gắn theo segment tham chiếu vị trí, và webview đã có luật
riêng cấm đọc `ord` (`segmentNavigation.ts:138-142`). Đẩy `ord` lên dây là mở đúng cánh cửa hai luật
đó đóng, và phép lùi-về-câu-còn-sống có **ca biên thật** (câu bị cắt bỏ, câu về hưu, neo 0) — AD-1
nói thẳng một quy tắc như thế không được sống ở webview.

**Vì sao scope động chứ không nới `tauri.conf.json`.** Gốc Library do người dùng cấu hình **lúc
chạy** (`resolve_library_root:186`), nên một glob tĩnh không diễn đạt được nó; và
`config_invariants.rs:313` ghim scope tĩnh đúng một mục vì `$RESOURCE/dict/**` từng là quyền thừa
đã gỡ. AD-23 đã chốt sẵn vế này — *"Scope động cấp lúc chạy chỉ khi người dùng chọn qua hộp thoại —
thư mục gốc Library"* — nên đây là **năng lực chưa dựng**, không phải một bất biến bị đổi: không cần
AD mới. Cấp cho **thư mục `.atproj` đang mở** chứ không cả gốc Library là hẹp hơn mức AD-23 cho
phép, và hẹp hơn thì không cần xin thêm.

**Vì sao không thêm index cho `asset`.** `schema.rs:940` nói lượt đọc đầu tiên là lượt bào chữa cho
một index — nhưng bảng chỉ giữ ảnh của một Tác phẩm, và một lượt quét vài nghìn hàng rẻ hơn hai con
số ghim phải nâng (`PROJECT_MIGRATIONS.len()`, `schema_version()`). Phép đo bào chữa thuộc Story
6.18, nơi NFR3/NFR4/NFR5 đo lại trên thư viện 5.000 Chương thật.

**Vì sao ảnh nằm TRONG ô nguyên văn chứ không chiếm một hàng riêng (Ice chốt 2026-09-10).** Lưới là
**chủ-cột**: `gridTemplateRows` đếm `editorSegments.length` và năm cột `v-for` cùng một mảng, thẳng
hàng nhờ `subgrid` — nên "một hàng" không phải một phần tử mà là một **bất biến giữa năm cột**. Một
hàng ảnh riêng buộc cả năm cột cùng thêm một phần tử **không phải segment**, và kéo theo bốn thứ đã
viết thành luật ở chỗ khác: đánh số câu bằng `index + 1` (`GridPanel.vue:1589`), điều hướng duyệt
bằng **chỉ số mảng** (`segmentNavigation.ts:138-142`, `NavigationSegment` cố ý không khai `ord`),
`focus()` tìm theo `[data-segment-id]`, và bề mặt vùng chọn đăng ký ở **cấp cột**. Đổi một bất biến
để hiện một ảnh là cái giá sai. Ô nguyên văn thì đã là một hộp chữ tự do chiều cao — đặt ảnh vào đó
là dùng đúng thứ đang có. Cái giá đã nhận, ghi ra thay vì để người sau phát hiện: **ô bản dịch đối
diện để trống đúng bằng chiều cao ảnh** — chưa đo trên bản dựng thật, và với ảnh cao thì khoảng
trống đó có thể lớn.

**Vì sao alt-text đi vào thuộc tính `alt`.** EXPERIENCE.md:357 nói alt-text *"là thứ trình đọc màn
hình đọc lên, không phải thứ mắt nhìn"*. Bỏ hẳn nó khỏi trang là làm đúng nửa đầu và **vứt** nửa
sau; đặt nó vào `alt` là chỗ duy nhất thoả cả hai. Ảnh không có segment `alt` ⇒ `alt=""` (ảnh trang
trí theo chuẩn ARIA), **không** bịa tên tệp vào đó.

## Verification

**Commands:**
- `npm run build && (cd src-tauri && cargo test --locked)` — theo đúng thứ tự `pre-push:103,106`.
- `npm run test` — vitest, `fileParallelism: false`.
- Chạy **TỪNG** cổng: `check:deps` `check:tokens` `check:i18n` `check:commands` `check:layout`
  `check:panel-refs` `check:dict` `check:dict-manifest` `check:lint` `check:gates` `check:debt-owner`
  — **mười một**, đúng `.githooks/pre-push:88`.
- 🔴 Đối chứng đỏ ① — gỡ lượt phân giải neo, chạy `segment_image_contract.rs`, phải **ĐỎ**, trả lại
  phải xanh. Ghi số ca đỏ và tên ca vào §Implementation Notes.
- 🔴 Đối chứng đỏ ② — gỡ lượt cấp scope động, dựng app thật, `<img>` phải trượt. Đây là mệnh đề
  **framework cưỡng chế**, không nghiệm thu được bằng vitest.
- `cargo test --test config_invariants --test pinned_contract` — xanh với **0 dòng sửa** trong hai
  tệp đó (`git diff` xác nhận).

**Manual checks (if no CLI):**
- Nhập một bài web có ảnh + `figcaption`, mở Chế độ đọc: ảnh đúng chỗ, chú thích dưới ảnh là bản
  dịch, `document.body.innerText` **không chứa** văn bản alt-text.
- Copy thư mục `.atproj` sang chỗ khác, mở lại: ảnh vẫn hiện.
</content>
</invoke>
