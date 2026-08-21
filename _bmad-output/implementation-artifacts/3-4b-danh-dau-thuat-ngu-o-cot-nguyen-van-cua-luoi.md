---
title: 'Story 3.4b — Đánh dấu thuật ngữ ở cột nguyên văn của lưới'
type: 'feature'
created: '2026-08-21'
status: 'done'
baseline_commit: '5c65256dece0e5b059381e57b58ab8b501242326'
review_loop_iteration: 0
context:
  - '{project-root}/_bmad-output/implementation-artifacts/epic-3-context.md'
  - '{project-root}/_bmad-output/implementation-artifacts/3-4-khop-thuat-ngu-theo-ngon-ngu-qua-matcher-dung-chung.md'
  - '{project-root}/AGENTS.md'
  - '{project-root}/src/AGENTS.md'
---

<frozen-after-approval reason="human-owned intent — do not modify unless human renegotiates">

## Intent

**Problem:** Story 3.4 dựng `glossary_marks_for_chapter` và nó có **0 người tiêu thụ** — người dịch vẫn không thấy câu đang dịch chứa thuật ngữ nào đã chốt. FR50 vì thế chưa đóng.

**Approach:** Tiêu thụ bề mặt đó **một lượt mỗi lần mở Chương**; cắt tại biên thuật ngữ ở **tầng dữ liệu** trên cả hai đường render của cột nguyên văn; hiện bản dịch của thuật ngữ ở một nhánh mới của `StatusBar`.

## Boundaries & Constraints

**Always:**
- 🔴 **Cắt ở TẦNG DỮ LIỆU, không chèn node.** `buildSegments` nhận thêm một tập biên; `sourcePiecesOf` gộp thêm biên. Template giữ nguyên khuôn `v-for` **một Segment ↔ một phần tử con** ⇒ bất biến `host.children[i] ↔ segments.value[i]` đứng **theo cấu tạo**. Thứ `editorSegments.ts:329-340` cấm là chẻ DOM **ngoài** tầm `segments.value`, không phải cắt mịn hơn ở tầng dữ liệu.
- 🔴 **HAI tập điểm, không một.** `props.cuts` vẽ `cut-here`; biên thuật ngữ **chỉ cắt**. `cut-here` gắn bằng `cutSet.has(seg.srcStart)` (`SourceHanViet.vue:789`), nên trộn hai tập ⇒ **mỗi thuật ngữ vẽ ra một dấu ngắt đoạn giả**.
- 🔴 **Độ hạt click-để-cắt của Story 2.9 KHÔNG được đổi.** Mảnh sinh ra bởi biên thuật ngữ giữ `data-src-atomic` và báo `data-src-start` của **đầu nhóm nguyên tử cũ** — Ice đã ký *"từ chối offset giữa mảnh"* (`editorSegments.ts:335-337`); node mịn hơn mà không xử sẽ **âm thầm mở ra** những vị trí cắt đoạn chưa ai duyệt.
- **Đúng MỘT lượt IPC mỗi lần mở Chương**, cộng một lượt làm mới sau gộp/tách và sau thêm nhanh 3.3. **Không một lượt nào trên đường gõ** (Ice ký 2026-08-21 — đây là thứ giữ 214 ms ngoài trần NFR2 50 ms).
- **Offset trên dây là ĐIỂM MÃ**, Rust đã quy đổi từ byte đúng một chỗ; `data-src-start` cũng là điểm mã. **Không quy đổi lại ở TS.**
- **Trường về JS giữ `snake_case`** (`is_confirmed`) — `GlossaryMarkWire` cố ý không có `rename_all`.
- **Adapter `src/config/glossary.ts` không bao giờ ném**: một `invoke`, một `try/catch`, hình dạng ba trạng thái; **type guard lúc chạy** cho mảng mark.
- Đã chốt tô `var(--color-primary)`; chờ chốt phân biệt bằng **kiểu gạch chân**. Tuyệt đối **không** `opacity`, không bóng đổ, không lớp nổi, **0** miễn trừ `z-index`.
- **Thứ tự ưu tiên `StatusBar`**: lỗi xác nhận > gộp/tách > điều hướng > **bản dịch thuật ngữ** > "Đã lưu N giây trước". Nhánh mới nối vào đúng chuỗi `v-if/v-else-if` đang có (`StatusBar.vue:235-288`), không dựng dòng thứ hai — thanh cao 34px chỉ chở một mệnh đề.
- Không quy tắc nghiệp vụ nào ở TypeScript (AD-1).

**Ask First:**
- Nếu phép đo cho thấy **một dấu bắc cầu qua chất nối** ⇒ hình dạng dây phải đổi (thêm `segment_id`, hoặc nhận một lát chuỗi) ⇒ chạm lại `src-tauri/`, ngoài phạm vi đang giả định.
- Nếu phải **hạ** một sàn quần thể thay vì nâng.
- Nếu phải thêm bất kỳ phụ thuộc nào (cửa NFR15).
- Nếu cặp số mở Chương cho thấy độ trễ đủ lớn để mở Chương thành một thao tác chậm thấy được.

**Never:**
- Gọi `glossary_marks_for_chapter` **mỗi segment một lượt** — `marks_for_source_text` nạp cả hai tầng ở **mỗi** lượt; Chương 9.850 câu ⇒ 9.850 lần nạp.
- Thêm cache hay chỉ mục ngược. Thêm `id`/`source_term` vào hình dạng dây để làm *"tô sáng các dấu anh em"* — đó là một quyết định thiết kế tương tác chưa ai mở.
- Sửa `core/matching/**`, cài lại phép khớp, hay chạm `candidate_store` · 3.5 → 3.10.
- Sửa `epics.md`/`prd.md` cho khớp mã.

## I/O & Edge-Case Matrix

| Scenario | Input / State | Expected Output / Behavior | Error Handling |
|---|---|---|---|
| Đã chốt, đường chữ trần | `.src-piece`, thuật ngữ giữa câu | Mảnh phủ thuật ngữ tô `primary`; số mảnh tăng, **0** dấu `cut-here` mới | — |
| Đã chốt, đường Hán Việt | tab Hán Việt, Chương tiếng Trung | Dấu hiện trên `.hv-unit`/`.hv-word`; một Segment vẫn một node | — |
| Thuật ngữ phủ **một phần** một `.hv-unit` | `文` trong unit `文化` | Unit tách thành hai Segment tại biên; `host.children.length === segments.value.length` | — |
| Thuật ngữ **bắc cầu** hai `.hv-unit` | biên thuật ngữ cắt ngang hai từ ICU | Cả hai unit tách; dấu liền mạch thị giác; ranh giới `Matcher` **thắng** ICU | — |
| Chờ chốt | `is_confirmed=false`, `translation=null` | Gạch chân, **không** `opacity`; rê chuột ⇒ thanh nói *"chưa chốt bản dịch"* | — |
| Rê chuột / tiêu điểm lên dấu đã chốt | `translation="Ngạo Lai quốc"` | Thanh hiện bản dịch **của thuật ngữ**, xướng qua `role="status"` | — |
| Rời chuột / mất tiêu điểm | — | Thanh trả về người ở trọ ưu tiên kế tiếp | — |
| Đang có câu lỗi xác nhận | `confirmNoticeKey !== null` | Bản dịch **không** đè lên nó | — |
| Chương không có thuật ngữ nào | `marks = []` | **0** mảnh bị cắt thêm; DOM y hệt trước story | — |
| IPC trả lỗi | kho đóng giữa chừng | **Không** đánh dấu gì; lỗi hiện qua `tError()`; **không** coi như rỗng | `IpcError` |
| Gộp/tách segment | `applyRegroup` chạy | Dấu làm mới; **không** dấu nào trỏ vào segment đã về hưu | — |
| Thêm nhanh một thuật ngữ (3.3) | lưu thành công | Dấu xuất hiện **không cần** mở lại Chương | — |
| Chương tiếng Anh | `source_lang !== 'zh'` | Chỉ đường chữ trần; khớp qua Porter2 | — |

</frozen-after-approval>

## Code Map

**Bề mặt IPC đã có (Story 3.4) — chỉ tiêu thụ, không sửa**
- `src-tauri/src/commands/glossary.rs:385` — vỏ `wire::glossary_marks_for_chapter(app, text: String, source_lang: String)`. Tên trên dây: **`text`** · **`sourceLang`**. Đã đăng ký ở `src-tauri/src/lib.rs:375`.
- `src-tauri/src/commands/glossary.rs:201-215` — `GlossaryMarkWire { start, end, tier: String("global"|"work"), is_confirmed, translation: Option<String> }`. **Không** `rename_all` (dòng 201 nói rõ) ⇒ về JS là `is_confirmed`. `start`/`end` là **điểm mã**.
- `src-tauri/src/core/glossary/store.rs:679` — `warm_jieba_for_source_lang`, gọi **đồng bộ** ở `commands/chapter.rs:117` (`read_open_chapter`) và `:236` (`open_adjacent_chapter`). Chi phí 179–329 ms rơi vào lần **đầu** của phiên.

**Cột nguyên văn — hai đường render, loại trừ lẫn nhau**
- `src/panels/GridPanel.vue:1408-1438` — `v-if="showHanViet"` ⇒ `<SourceHanViet>`; `v-else` ⇒ vòng `sourcePiecesOf(...)` dựng `span.src-piece[data-src-start]`. `showHanViet` ở `:271`. **Không bao giờ cả hai cùng lúc.**
- `src/panels/GridPanel.vue:349-362` `sourcePiecesOf(segmentId, text)` · `:339-347` `sourcePieceStartsOf` — cắt theo `pendingCuts` (2.8/2.9), đơn vị **điểm mã**, hai hàm là **bản sao có chủ ý phải giữ đồng bộ** (`:335-337`).
- `src/panels/GridPanel.vue:292-296` — lý do dấu cắt vẽ bằng `::before` chứ không chèn `<span>`: chèn node làm lệch bất biến.

**Hán Việt — chỗ biên thuật ngữ phải đi vào**
- `src/panels/SourceHanViet.vue:164` — 🔴 `buildSegments(text: string): Segment[]`, **một tham số, không nhận cắt ngoài**. Cắt 100% theo `wordStarts.has(index)` (ICU) ở `:212` — **đây là điểm chèn**. Nhánh `text` (`:222-225`) hôm nay không flush theo biên nào, cần điểm chèn tương tự.
- `src/panels/SourceHanViet.vue:75-117` — prop hiện có: `sourceText` · `viewMode` · `cuts?: readonly number[]` (`:86`) · `surfaceRole`. **Prop mark mới chép đúng khuôn `cuts`.**
- `src/panels/SourceHanViet.vue:773-826` — kiểu `switch` dựng `.hv-word` (chứa `.hv-syl`, hiện **âm**, không hiện chữ Hán); kiểu `parallel` dựng `.hv-unit` bọc `<ruby>`. ⚠️ Banner `:8-16` còn viết *"một node mỗi ký tự"* — **hết đúng từ 1.18b**, nay là một **TỪ** một node; sửa tại chỗ kèm 🔵 + ngày khi chạm tệp.
- `src/panels/SourceHanViet.vue:628-681` `resolveSwitch()` + doc-comment `:596-627` — nơi bất biến sống.
- `src/panels/editorSegments.ts:282-317` `sourceCutOffsetOf` · `:322-347` `SRC_ATOMIC` · `:360-367` `neoNguonCua` — đường ánh xạ DOM → offset. `data-src-atomic` = *"chỉ cắt được ở ĐẦU phần tử này"*, đặt trên `.hv-word` và `.hv-text`, **không** trên `.hv-unit`.

**Nạp Chương và làm mới**
- `src/config/chapter.ts:110-132` `read_open_chapter` → `OpenChapter { chapter_id, source_text, source_lang }` — **toàn văn Chương**, rơi vào `sourcePanelState.ts:59`.
- `src/config/segment.ts:66-120` `ChapterSegment { id, ord, source_text, target_text, is_paragraph_end, retired_at, status, is_omitted, is_target_paragraph_end }` — 🔴 **không trường offset cấp Chương nào**.
- `src/panels/editorPanelState.ts:1536-1537` — funnel **duy nhất**: `switchChapter()` gọi lại `ensureSegmentsLoaded()` + `ensureChapterLoaded()` sau khi reset. `GridPanel.vue:115-123` gọi cùng hai hàm lúc mount. ⇒ **một chỗ gắn, phủ cả hai đường mở Chương.**
- `src/panels/editorPanelState.ts:1953-1975` `applyRegroup` — chỗ làm mới sau gộp/tách.
- `src/glossaryQuickAddState.ts:301-310` — ⚠️ **không phát tín hiệu nào** sau khi thêm thành công; state singleton cấp module, không có hook đăng ký. Vế "làm mới khi Glossary đổi" phải tự dựng đường gọi.
- `src/config/glossary.ts:113` `lookupGlossaryTerm` — **khuôn ba trạng thái để chép**; helper dùng lại: `hasIpcBridge`, `isIpcError`, `UNKNOWN_IPC_ERROR`. **Chưa có** hàm mark, chưa có kiểu, chưa có type guard.

**StatusBar · token · cổng**
- `src/StatusBar.vue:235-288` — `<footer role="status">` với **bốn nhánh `v-if/v-else-if` loại trừ lẫn nhau**; `:307-315` cao 34px một hàng. Chú thích `:104-106,244-246` nói rõ *"chỉ đủ MỘT mệnh đề"* là cố ý.
- `src/tokens/tokens.json:29` `primary #2f5d63` (sáng) · `:48` `#7fb3ba` (tối). `DESIGN.md:165,187` — ba việc duy nhất, thuật ngữ Glossary đã chốt là việc thứ nhất.
- ⚠️ **Chưa có token/lớp gạch chân** trong `src/**`; chỉ có `line-through` (`GridPanel.vue:2048`) cho mục đích khác.
- `scripts/check-tokens.mjs:833` Kiểm B · `~1010` B2 · `1345` D (`opacity` cần `/* aura-allow-opacity: … */`) · `1443` F (bóng/gradient **không miễn trừ được**; `z-index` miễn trừ được, hiện đúng **3** mục, `src/tokens/README.md:55`).
- **Sàn quần thể phải xét lại nếu thêm tệp:** `check-tokens.mjs:91` `FILE_FLOOR=50` · `:92` `COMPONENT_FILE_FLOOR=47` · `check-layout.mjs:110` `FILE_FLOOR=47` · `check-panel-refs.mjs:517` `FILE_FLOOR=34` (`.ts`) · `check-commands.mjs:211` `VUE_FLOOR=14` · `:233` `TS_FLOOR=34` · `check-i18n.mjs:306` `VUE_FLOOR=14`.
- `scripts/check-panel-refs.mjs:2-4` — mọi ô nhớ cấp module trong `src/**/*.ts` phải đi qua một hàm reset. ⇒ state mark mới **bắt buộc** có reset.
- `scripts/check-commands.mjs:33-35` Kiểm A chỉ canh `@click`; `@mouseenter`/`@focus` **không cổng nào chạm** — chú thích tại chỗ cảnh báo đừng nới regex lặng lẽ.

**Phép đo phải chạy lại, và cách chúng từng được đo**
- AC6/1.16 (`1-16-…md:511-524`) · AC11+AC12/1.18 (`1-18-…md:495-521`) — vùng chọn cho đúng chuỗi nguồn ở kiểu song song, và `Selection.modify()` trên **cả** WKWebView lẫn Chromium. ⚠️ **Cả ba đo bằng script chạy tay** (Playwright headless / bộ đo Swift), **không** có cổng tự động nào lặp lại.
- `tests/frontend/hanVietCutAnchors.test.ts` — cổng vitest duy nhất canh `data-src-start`/`data-src-atomic`/`cut-here`.
- `src/panels/selectionContract.ts:358-361` — ⚠️ `Selection.modify()` đi **xuyên qua** `user-select: none` trên WKWebView (đo 2026-08-07).

**Sổ nợ chủ Story 3.4b — đọc trước khi viết dòng đầu**
- `deferred-work.md:243` (`ScopeResolver` chưa cache) · `:492-500` (điều kiện khởi hành đã đóng, **món nợ ĐO còn mở**) · `:914` (ICU cắt sai tên riêng, bảng 10 ca) · `:926` (tự cắt `.hv-unit`) · `:5362` (story rủi ro nhất, đường nóng) · `:5781` (chuột kéo thật chưa nghiệm thu được trong WKWebView) · `:5925` (cặp số mở Chương) · `:5935` (`GlossaryMark` không mang `id`).

## Tasks & Acceptance

**Execution:**
- [x] `src/config/glossary.ts` -- thêm adapter thứ tư `glossaryMarksForChapter(text, sourceLang)` + kiểu `GlossaryMark` + **type guard lúc chạy** cho mảng; chép khuôn ba trạng thái của `lookupGlossaryTerm` -- Rust có thể trả `null` cho `translation`, và type guard là chỗ duy nhất biết.
- [x] `src/panels/glossaryMarksState.ts` (mới) -- giữ mảng mark theo Chương, phơi `ensureGlossaryMarksLoaded()` · `refreshGlossaryMarks()` · `resetGlossaryMarks()` · một vị từ `glossaryMarksHaveLoaded` -- ô nhớ cấp module **bắt buộc** có reset (`check:panel-refs`), và một danh sách rỗng phải phân biệt được với *"chưa nạp"*.
- [x] `src/panels/glossaryMarksMap.ts` (mới) -- hàm **thuần**: nối `segment.source_text` bằng `\n`, tính offset cộng dồn, chia mảng mark tuyệt đối về **từng segment**; trả cả tập biên cắt lẫn nhãn (đã chốt / chờ chốt) -- hàm thuần là thứ vitest kiểm được tất định, và đây là chỗ **duy nhất** phép cộng dồn tồn tại.
- [x] `src/panels/editorPanelState.ts` -- gọi `ensureGlossaryMarksLoaded()` cạnh `ensureChapterLoaded()` trong `switchChapter()`, và `refreshGlossaryMarks()` trong `applyRegroup` -- 🔵 xem Spec Change Log: funnel THẬT hoá ra không thể là hai lời gọi tường minh không thôi (đường mount đầu tiên gọi `ensureSegmentsLoaded()`/`ensureChapterLoaded()` KHÔNG `await`), nên `GridPanel.vue` mang một `watch` bổ sung — cả hai cùng idempotent, an toàn khi trùng.
- [x] `src/glossaryQuickAddState.ts` -- gọi `refreshGlossaryMarks()` trên đường lưu thành công -- không có event bus; một lời gọi tường minh đọc được, một đường ngầm thì không.
- [x] `src/panels/GridPanel.vue` -- gộp biên thuật ngữ vào `sourcePiecesOf`/`sourcePieceStartsOf` **thành một tập RIÊNG** với `pendingCuts`; gắn class dấu lên mảnh; truyền prop mark xuống `SourceHanViet` -- trộn hai tập là vẽ ra một dấu ngắt đoạn giả cho mỗi thuật ngữ. 🔵 **Bổ sung 2026-08-21 (Ice bác lượt hạ vế tiêu điểm thành nợ):** `onSourceSelectionChange()` nghe `selectionchange` ở `document` (cùng khuôn `onSelectionChange` đã có cho cột bản dịch), ánh xạ caret về offset qua `sourceCutOffsetOf`, ghi vào ĐÚNG state mà `@mouseenter` ghi -- đạt vế "tiêu điểm" của I/O Matrix với **0** `tabindex` mới, tái dùng `Selection.modify()`/AC11 đã có từ Story 1.18.
- [x] `src/panels/SourceHanViet.vue` -- `buildSegments(text, termBoundaries)` flush thêm tại biên thuật ngữ; mảnh mới giữ `data-src-atomic` và báo `data-src-start` của **đầu nhóm nguyên tử cũ**. Banner `:8-16` đã đọc lại: 1.18b (TRƯỚC story này) đã sửa nó đúng — không còn "một node mỗi ký tự" nào để sửa; ghi lại trong Spec Change Log thay vì tạo một lượt sửa không cần thiết.
- [x] `src/StatusBar.vue` -- nhánh `v-else-if` thứ năm, đặt **trên** nhánh "Đã lưu" và **dưới** ba thông báo -- thanh 34px chở đúng một mệnh đề; không câu báo lỗi nào được một cú rê chuột nuốt mất.
- [x] `src/tokens/tokens.json` + `src/panels/GridPanel.vue` -- 🔵 xem Spec Change Log: KHÔNG thêm token mới vào `tokens.json` -- `--color-primary` đã tồn tại và đã qua Kiểm C; kiểu gạch chân dùng `text-decoration-*` (KHÔNG phải một token màu/cỡ chữ, ngoài phạm vi Kiểm B/B2, cùng lý lẽ đã ghi cho `.cell.omitted{text-decoration:line-through}`).
- [x] `src/i18n/vi.json` -- **một** khoá mới `glossary.mark.pending_translation`; ca đã chốt TÁI DÙNG `glossary.quick_add.translation_label` đã có (xem Spec Change Log) -- không nhân đôi nguồn sự thật cho cùng một nhãn.
- [x] `tests/frontend/glossaryMarksMap.test.ts` (mới) -- mọi hàng I/O Matrix kiểm được không cần DOM: cộng dồn offset, chia về segment, phủ một phần, bắc cầu, rỗng, chờ chốt -- tên hàm là một **CÂU** khẳng định. 12 ca, xanh.
- [x] `tests/frontend/hanVietCutAnchors.test.ts` -- mở rộng: mảnh sinh bởi biên thuật ngữ vẫn báo `data-src-start` của đầu nhóm nguyên tử, đối chứng số phần tử trước/sau cắt (`segments.value` không phơi ra ngoài component nên đối chứng qua số con DOM thay vì so trực tiếp với state nội bộ) -- 15 ca mới (28 tổng trong tệp), xanh.
- [x] `tests/frontend/glossaryHoverSelection.test.ts` (mới, bổ sung 2026-08-21) -- mount `GridPanel.vue` THẬT, dựng `Selection`/`Range` thật, canh vế TIÊU ĐIỂM: caret vào mảnh mang dấu → state khớp bản dịch; caret rời (cùng segment hoặc khác segment) → về `null`; chuột và bàn phím ghi vào ĐÚNG một state; **0** `.src-piece` nào mang `tabindex` -- 6 ca, xanh.
- [x] `src-tauri/tests/glossary_marks_contract.rs` (bổ sung 2026-08-21, theo yêu cầu Ice) -- hai test THƯỜNG TRỰC cho AC "0 dấu bắc cầu qua chất nối `\n`": nhánh `Zh` (`萧炎`, ví dụ đã có trong doc-comment `find_terms`) và nhánh `En` (`fire dragon`), mỗi ca có đối chứng dương (liền nhau thì khớp) trước khi khẳng định đối chứng âm (bị `\n` chen thì không khớp) -- 16 ca trong tệp (14 cũ + 2 mới), xanh.
- [x] `_bmad-output/implementation-artifacts/3-4b-ban-do-danh-dau.html` (mới) -- vế thị giác (hai theme, ba bề mặt render) và vế đo số (tương phản WCAG của cả hai kiểu dấu) THẬT SỰ chạy được (đã mở bằng Chromium headless, `window.__benchReport()` in bảng số, 0 lỗi JS). 🔵 xem Spec Change Log cho lý do cặp số mở Chương KHÔNG nằm trong tệp HTML này.
- [x] `scripts/check-*.mjs` -- `check-panel-refs.mjs` `FILE_FLOOR` 34→36, `check-commands.mjs` `TS_FLOOR` 34→36, `check-tokens.mjs` `FILE_FLOOR` 50→53 + `COMPONENT_FILE_FLOOR` 47→50, `check-layout.mjs` `FILE_FLOOR` 47→50 -- số THẬT đo bằng `find`, ghi kèm ngày tại chỗ. `check-i18n.mjs` `RS_FLOOR`/`VUE_FLOOR` KHÔNG đổi (0 tệp `.rs`/`.vue` mới).
- [x] `_bmad-output/implementation-artifacts/deferred-work.md` -- đóng `:243` (✅, câu hỏi tần suất giờ có câu trả lời SẢN PHẨM), `:492-500` (🟡, đo được cặp lạnh/ấm ở tầng Rust, còn hở vế webview thật), `:926` (✅, `.hv-unit` tự cắt xong), `:5935` (✅, quyết định KHÔNG tô sáng anh em); mở mục mới `## Deferred from: 3-4b-…` với BỐN mục có chủ: cặp số mở Chương trên webview thật, khoảng cách `StatusBar` ↔ thuật ngữ, bàn phím chưa tới được dấu, chuột kéo thật WKWebView chưa nghiệm thu. Không xoá mục nào đã có.

**Acceptance Criteria:**
- Given một Chương tiếng Trung mở ở tab Hán Việt, when đếm, then `host.children.length === segments.value.length` **sau** khi cắt tại biên thuật ngữ, và **0** node nào được chèn ngoài `segments.value`.
- Given đường click-để-cắt của Story 2.9, when bấm vào một mảnh sinh bởi biên thuật ngữ, then offset cắt giải ra **y hệt** trước story — độ hạt không đổi.
- Given `glossary_marks_for_chapter`, when đếm số lượt gọi trong một phiên mở một Chương rồi gõ 200 phím, then đúng **1** lượt, và **0** lượt trên đường gõ.
- Given đường ánh xạ offset nối bằng `\n`, when đo trên **cả hai** nhánh `Zh` và `En` với thuật ngữ đặt sát biên segment, then **0** dấu bắc cầu qua chất nối — số ghi thẳng vào story, không suy luận.
- Given cùng một Chương tiếng Trung, when đo cặp số mở Chương trước/sau story với lượt **lạnh** tách khỏi lượt ấm, then cả hai số ghi vào story kèm toolchain + ngày.
- Given AC6/1.16 và AC11+AC12/1.18, when chạy lại phép đo vùng chọn trên **cả** WKWebView lẫn Chromium, then không mệnh đề nào hồi quy — **đo lại**, không suy từ số cũ.
- Given `.githooks/pre-push`, when chạy, then mười một cổng + vitest + build + `cargo test --locked` xanh.
- Given lượt CI sau khi push, when đọc, then **cả** macOS lẫn Windows xanh; và vì bề mặt này là bề mặt **DOM**, một lượt **e2e chạy tay** đỡ lưng mệnh đề "webview thật" — *"CI xanh" không có nghĩa e2e đã chạy*.

## Spec Change Log

### 2026-08-21 — thực thi

**Quyết định thực thi không tường minh trong spec, ghi lại để lượt sau đọc được lý do:**

- **`glossaryMarksState.ts` KHÔNG tự đọc `editorSegments`/`editorChapterId`/`sourceChapter` —
  mọi hàm nhận `chapterId`/`segments`/`sourceLang` làm THAM SỐ.** Lý do là một vòng phụ thuộc:
  task đòi `editorPanelState.ts` GỌI `ensureGlossaryMarksLoaded`/`refreshGlossaryMarks` (chiều
  `editorPanelState.ts → glossaryMarksState.ts`). Nếu `glossaryMarksState.ts` quay lại `import`
  state của `editorPanelState.ts` để tự đọc segment/chapter, đó là một VÒNG — đúng luật
  `editorPanelState.ts:27` đã tự đặt cho `sourcePanelState.ts` ("Không vòng"). Giải: cùng khuôn
  `glossary_marks_for_chapter` phía Rust (nhận `text`/`source_lang` làm tham số thay vì tự đọc
  Chương từ đĩa) — chỗ gọi luôn có sẵn dữ liệu cần trong tay.
- **Funnel "một chỗ gắn, phủ cả hai đường mở Chương" hoá ra KHÔNG THỂ chỉ là lời gọi tường
  minh ở `editorPanelState.ts::switchChapter`.** Đường mount ĐẦU TIÊN
  (`GridPanel.vue::onMounted`) gọi `ensureSegmentsLoaded()`/`ensureChapterLoaded()` KHÔNG
  `await` (fire-and-forget) — gọi `ensureGlossaryMarksLoaded()` ngay sau đó với
  `editorChapterId.value`/`sourceChapter.value` sẽ đọc `null` vì hai lượt IPC kia còn đang bay
  (một lượt `await ensureChapterLoaded()` THỨ HAI không chờ được lượt ĐẦU: hàm đó
  idempotent-early-return, không phải một promise dùng lại). ⇒ `GridPanel.vue` mang một
  `watch([editorChapterId, sourceChapter], …, {immediate:true})` — phản ứng đúng lúc CẢ HAI
  nguồn sẵn sàng, bất kể ai/khi nào gọi khiến chúng đổi. Lời gọi tường minh trong
  `switchChapter()` (task đòi) **vẫn giữ**, an toàn vì `ensureGlossaryMarksLoaded` idempotent
  theo `chapterId` — hai đường trùng nhau vô hại, đường nào tới trước thắng.
- **Chuỗi GỬI cho `glossaryMarksForChapter` là `segments.map(s => s.source_text).join('\n')`,
  KHÔNG PHẢI `chapter.source_text` thô.** Đây là hệ quả bắt buộc của Design Notes "chất nối
  `\n`, chưa phải kết luận": chỉ chuỗi nối-từ-segment mới có phép cộng dồn NGHỊCH được về
  `(segmentId, offset cục bộ)` mà `glossaryMarksMap.ts` cần — `chapter.source_text` thô đã bị
  `trim()`/`skip_gap` biến dạng nên không tái tạo lại được ranh giới segment.
- **KHÔNG thêm token màu/kiểu mới vào `tokens.json`.** `--color-primary` đã tồn tại, đã qua
  Kiểm C (WCAG). Kiểu gạch chân của mục chờ chốt dùng thẳng `text-decoration-line/-color/
  -thickness/-offset` — các thuộc tính này KHÔNG phải màu/cỡ chữ nên nằm ngoài phạm vi Kiểm
  B/B2 (`check-tokens.mjs`), đúng tiền lệ `.cell.omitted{text-decoration:line-through}` đã có
  từ trước ở `GridPanel.vue`. Hai kiểu (đã chốt = tô CHỮ primary, chờ chốt = gạch chân primary,
  chữ giữ màu thường) là hai kênh KHÔNG chồng nhau — không cần một biến thể `opacity` nào.
- **`rt { text-decoration-line: none }` là một hàng rào MỚI, thêm ngoài phạm vi tường minh của
  spec.** `.hv-word`/`.hv-unit` (cha) mang `text-decoration` cho ca chờ chốt; theo mặc định CSS,
  giá trị đó THỪA KẾ xuống mọi hộp con, kể cả `<rt>` (âm Hán Việt, kiểu song song) — tức gạch
  chân sẽ "bleed" vào dòng âm đọc bên dưới. Tắt tường minh ở `<rt>` vì gạch chân là dấu cho CHỮ
  HÁN (base của `<ruby>`), không cho âm đọc (đã có màu `on-surface-variant` riêng). Đo được
  bằng `getComputedStyle` trong `3-4b-ban-do-danh-dau.html`.
- ~~**HAI kênh tương tác trong I/O Matrix ("rê chuột / đưa tiêu điểm") chỉ cài được MỘT: chuột
  (`@mouseenter`/`@mouseleave`).** KHÔNG `tabindex="0"`/`@focus`/`@blur` trên từng mảnh mang
  dấu. Lý do: gắn tab-stop lên hàng trăm mảnh trong một Chương là một thay đổi vào đúng bề mặt
  mà hợp đồng vùng chọn (AC6/AC11/AC12 của Story 1.16/1.18) đã đo và ký RẤT cẩn thận trên cấu
  trúc DOM hôm nay — mở rộng phạm vi đo được của story này ra một rủi ro chưa cân xứng. Ghi nợ
  có chủ ở `deferred-work.md` (mục `## Deferred from: 3-4b-…`), không lặng lẽ bỏ qua.~~
  🔵 **HẾT ĐÚNG 2026-08-21, cùng phiên — mệnh đề trên bị bác ngay trong lượt nghiệm thu.** Hàng
  I/O Matrix đó nằm **trong** khối `<frozen-after-approval>`, tức thuộc về Ice; một agent cài đặt
  không hạ nó xuống thành nợ được. **Tiền đề của lý do thì vẫn đúng và được giữ:** không thêm
  một `tabindex` nào. Cái sai là kết luận *"vậy thì không cài được"* — `.hv-switch`/`.hv-parallel`
  đã mang `tabindex="0"` từ Story 1.18 (AC11) và `Selection.modify()` di chuyển caret **không**
  đòi phần tử tự focus được, nên vế tiêu điểm đạt được qua `selectionchange` với **0** tab-stop
  mới và **0** đổi cấu trúc DOM. Cài đặt: `GridPanel.vue::onSourceSelectionChange()`; kiểm chứng:
  `tests/frontend/glossaryHoverSelection.test.ts` (6 ca, gồm một ca khẳng định **không**
  `.src-piece` nào mang `tabindex`). Mục sổ nợ tương ứng đã đóng bằng `→ ✅ ĐÃ ĐÓNG`, kèm một
  biên hiếm còn hở ghi ra tại chỗ.
  ⚠️ **Bài học của lượt này, giữ lại vì nó lặp được:** một ràng buộc thật (*"đừng phá hợp đồng
  vùng chọn"*) đã được dùng làm lý do cho một kết luận rộng hơn nó (*"nên bỏ vế tiêu điểm"*), và
  cả hai cùng đi vào sổ nợ như một khối. Kiểm được: ràng buộc đứng, kết luận đổ.
  🔵 **SỬA TẠI CHỖ 2026-08-21 (rà ba lớp, P8) — câu "đạt vế tiêu điểm với 0 tab-stop mới" ở
  trên ĐÚNG nhưng bị đọc RỘNG HƠN nó khai.** Nó chỉ được ĐO cho đường **Hán Việt**
  (`.hv-switch`/`.hv-parallel` có `tabindex="0"` sẵn từ 1.18, và
  `glossaryHoverSelection.test.ts` dựng `Selection`/`Range` thật trên chính hai phần tử đó).
  Đường **CHỮ TRẦN** (`.src-piece`, `.col.col-src`) hoàn toàn KHÔNG `tabindex` — chính
  `GridPanel.vue:673`/`:1033` đã ghi thế, TỪ TRƯỚC story này. Đường bàn phím DUY NHẤT vào cột
  đó là lệnh CÓ SẴN `selection.focus_source` (`⌘⌥S`, `selectionContract.ts::focusSelectionSource`,
  Story 1.16/1.18) — nó đặt một `Range` thu gọn ở đầu `colSrc` (không cần `.focus()` thành
  công, `Range`/`Selection` không đòi phần tử tự focus được), rồi `Shift+←/→`
  (`selectionCommands.extendLeft/Right`) di chuyển tiếp qua `Selection.modify()`. VỀ MẶT KIẾN
  TRÚC, `Selection.modify()` bắn `selectionchange` NGUYÊN SINH nên `onSourceSelectionChange()`
  NÊN vẫn phản ứng đúng — nhưng chuỗi ĐẦY ĐỦ *"`⌘⌥S` → `Shift+→` lặp lại → bản dịch thuật ngữ
  hiện trên `StatusBar`"* trên đường chữ trần **CHƯA được đo trên một webview thật**, chỉ có
  lập luận kiến trúc. Xem mục sổ nợ MỚI, có chủ, đã mở ở `deferred-work.md`
  (`## Deferred from: 3-4b-…`).
- **Một khoá i18n MỚI, không hai.** Ca "đã chốt" tái dùng `glossary.quick_add.translation_label`
  (đã có từ Story 3.3, "Bản dịch") làm tiền tố ghép với `translation` (DỮ LIỆU) — cùng khuôn
  `panel.source.han_viet_sources_prefix` + dữ liệu ghép sau ở `SourceHanViet.vue`. Chỉ ca "chờ
  chốt" cần một câu hoàn chỉnh mới: `glossary.mark.pending_translation`.
- **Cặp số "mở Chương" (lạnh/ấm) đo Ở TẦNG RUST bằng một tệp bench TẠM
  (`src-tauri/tests/zzz_scratch_bench_chapter_open.rs`, dựng · chạy · XOÁ ngay trong lượt này —
  cùng tiền lệ `zzz_scratch_bench_marks.rs` của Story 3.4), KHÔNG nằm trong
  `3-4b-ban-do-danh-dau.html`.** Một trang HTML tĩnh mở bằng trình duyệt không có cầu IPC tới
  Rust nên không thể tự đo `warm_jieba_for_source_lang`/`marks_for_source_text` — yêu cầu gốc
  của task "vế đo số nằm trong bàn đo" không thực thi được ĐÚNG NGHĨA ĐEN cho vế này; đo tách
  ra và ghi số + toolchain + ngày vào `deferred-work.md` (mục `:492-500` nối tiếp). Bàn đo HTML
  giữ đúng vai còn lại của nó: tương phản + hình học của DẤU, thứ nó ĐO ĐƯỢC thật.
  ⚠️ **Số đo này CHƯA đóng được AC "cặp số mở Chương ghi vào story"** theo đúng nghĩa "một lượt
  mở Chương cảm nhận được" — nó là chi phí THUẦN RUST của lượt khớp, không gồm hai lệnh IPC đọc
  Chương/segment đã có từ trước, chi phí serialize, hay lượt render lại DOM. Xem `deferred-work.md`
  cho số liệu đầy đủ và giới hạn đã ghi.
- **`tests/frontend/hanVietCutAnchors.test.ts` đối chứng "host.children.length ===
  segments.value.length" GIÁN TIẾP, không trực tiếp.** `segments` là một `computed` NỘI BỘ của
  `<script setup>`, không export — không có đường nào từ bên ngoài đọc `segments.value` để so
  trực tiếp với `host.children.length`. Cách đối chứng thật: đếm số phần tử DOM khớp đúng số
  Segment SUY RA được từ input đã biết (số biên ICU + số biên thuật ngữ), cho cả ca "không
  thuật ngữ" (đối chứng) và "có thuật ngữ, phủ một phần/bắc cầu" (mệnh đề chính) — cùng năng
  lực xác nhận, khác cách viết.
- **`StatusBar.vue::glossaryHoverText` được BỔ SUNG một nhánh lỗi — một hành vi THẬT, không chỉ
  một ca test** (Matrix Test Audit, Ice 2026-08-21). Trước bản vá này, `glossaryMarksState.ts`
  giữ đúng bất biến "lỗi phân biệt được với rỗng" Ở TẦNG STATE (`glossaryMarksHaveLoaded()`),
  nhưng KHÔNG chỗ nào trên màn hình hiện lỗi đó ra — I/O Matrix đòi "lỗi hiện qua `tError()`"
  và trước lượt vá không đường nào thi hành đúng chữ đó. Vá: khi không có gì đang hover
  (`hoveredGlossaryTerm.value === null`) VÀ `glossaryMarksError.value !== null`, nhánh thứ năm
  của `StatusBar` hiện `tError(err)` thay vì trống rỗng — cùng VỊ TRÍ ưu tiên đã có (dưới ba
  thông báo khẩn hơn, trên "Đã lưu"), không một nhánh `v-else-if` thứ sáu nào được thêm vào
  chuỗi đã đóng của template.

### 2026-08-21 — hai lượt sửa của chính vòng nghiệm thu (điều phối viên, không phải agent cài đặt)

- 🔴 **Nửa thứ hai của hàng Matrix *"số mảnh tăng, 0 dấu ngắt đoạn mới"* không có chủ cho tới
  lượt rà cuối.** Mã sản phẩm đã đúng từ đầu (`GridPanel.vue:1614`,
  `v-if="i > 0 && piece.isPendingCut"`, kèm chú thích 🔴 nêu lý do) — nhưng **không test nào
  canh nó**. Gỡ vế `&& piece.isPendingCut` thì 316 ca, `cargo test`, và cả 9 cổng **vẫn xanh**
  trong khi mỗi thuật ngữ vẽ ra một dấu ngắt đoạn GIẢ trên màn hình. Đúng ràng buộc *"HAI tập
  điểm, không một"* mà `§Boundaries` ghi 🔴, và đúng lớp *"vi phạm được mà không cổng nào đỏ"*.
  Thêm một ca ở `glossaryHoverSelection.test.ts`, kèm **đối chứng dương** (2 `.src-piece` —
  không có vế này thì "0 `.cut-mark`" cũng xanh khi phép cắt hỏng hoàn toàn).
  Nghiệm thu bằng **chu trình đỏ→xanh thật**: gỡ vế bảo vệ ⇒ **đúng 1 ca đỏ**; khôi phục ⇒ 7/7
  xanh. *(Một cổng chưa bao giờ đỏ là một cổng chưa ai biết nó có chạy không.)*
- **`check:lint` ĐỎ ở `glossaryMarksRefresh.test.ts` sau lượt sửa Matrix** —
  `@typescript-eslint/no-unnecessary-condition` trên một `??`. Gốc là **kiểu nói dối**:
  `tsconfig` không bật `noUncheckedIndexedAccess` nên phép tra chỉ số khai là luôn có giá trị,
  trong khi hàng đợi rỗng cho `Math.min(n, -1)` = **-1** ⇒ `undefined` thật.
  ⚠️ **Lượt sửa ĐẦU thất bại và nó dạy một điều đáng ghi:** khai `const ke: T | undefined = …`
  **không** đủ — TypeScript **thu hẹp `const` theo giá trị gán**, nên nó narrow ngược về `T` và
  cổng vẫn đỏ. Lối ra đúng là nêu điều kiện rỗng thành một **nhánh có tên** (`if (length === 0)
  return …`) — không gỡ `??` (gỡ là mở một ca sập) và không `eslint-disable` (một miễn trừ cho
  một mệnh đề đang hỏng).

## Design Notes

**Chất nối `\n`, và vì sao nó là ứng viên chứ chưa phải kết luận.** `ChapterSegment` không mang offset cấp Chương nào, nên đường nối duy nhất còn lại là nối `segment.source_text` rồi cộng dồn. `\n` được chọn vì **cả hai** nhánh khớp đã có lý do từ chối bắc cầu qua nó: nhánh `En` từ chối tường minh dãy bắc cầu qua `\n`; nhánh `Zh` phụ thuộc jieba cắt `\n` thành token riêng.

🔵 **ĐO XONG 2026-08-21 — cả hai nhánh, qua chính `marks_for_source_text`, hai test THƯỜNG TRỰC ở `src-tauri/tests/glossary_marks_contract.rs`.** Câu "vế thứ hai chưa ai đo" ở trên đã HẾT ĐÚNG:
- `a_chinese_term_placed_right_across_the_newline_joiner_produces_no_mark` — thuật ngữ `萧炎` (rơi ra từng ký tự vì HMM=false, đúng ví dụ đã có sẵn trong doc-comment của `find_terms`) khớp đúng 1 dấu khi liền nhau (`萧炎和林动`, đối chứng dương), **0** dấu khi bị `\n` chen giữa (`萧\n炎和林动`). Cơ chế: `find_terms` nhánh `Zh` khớp bằng `text.find(term)` (chuỗi con byte) — `\n` xen vào GIỮA hai ký tự của thuật ngữ làm chuỗi con đó **không còn tồn tại** trong văn bản, độc lập với việc jieba tách token thế nào. Bảo đảm này đứng **theo cấu tạo** của thuật toán khớp con, không chỉ nhờ luật ranh giới token.
- `an_english_multi_word_term_placed_right_across_the_newline_joiner_produces_no_mark` — thuật ngữ `fire dragon` khớp đúng 1 dấu khi cách nhau một dấu cách (đối chứng dương), **0** dấu khi bị `\n` chen giữa (`a fire\ndragon roars`). Cơ chế: `crosses_sentence_boundary` (vá lúc code review 2026-08-05 cho ranh giới CÂU, `.`/`!`/`?`/`\n`) — Story 3.4b không thêm luật mới, nó **tiêu thụ** một luật đã có sẵn cho một mục đích khác (ranh giới câu) và luật đó tình cờ đúng cho ranh giới segment.

⇒ Kết luận: chất nối `\n` an toàn cho CẢ HAI nhánh, và an toàn **theo cấu tạo** (không phải một điều kiện chạy đua có thể vỡ khi dữ liệu đổi) — Zh vì bản chất phép khớp chuỗi con, En vì một luật đã tồn tại từ trước cho mục đích khác.

⚠️ Nối bằng `chapter.source_text` thì **không** cho một phép cộng dồn: `push_segment` `trim()` mỗi câu và bỏ câu rỗng, `skip_gap` nuốt trọn khe trắng, và sau gộp/tách segment không còn dẫn xuất được từ văn bản Chương.

**Ba tập điểm trên cùng một trục, đừng gộp:**

```
props.cuts        -> điểm ngắt đoạn của người dùng   -> CẮT + vẽ `cut-here`
termBoundaries    -> biên thuật ngữ Glossary          -> CẮT, không vẽ gì
srcStart của node -> neo ánh xạ ngược về source_text  -> KHÔNG cắt, chỉ báo cáo
```

Gộp hai tập đầu là hỏng thấy được ngay (dấu ngắt đoạn giả). Gộp tập thứ ba vào hai tập kia là hỏng **im lặng**: `sourceCutOffsetOf` sẽ trả những offset mà Ice đã ký từ chối.

### Rà ba lớp LẦN HAI (Ice, 2026-08-21) — 15 phát hiện, xếp theo hậu quả

Ice phân loại cả 15 phát hiện là `patch` (không cái nào chạm khối đông cứng, không cái nào đòi
dựng lại mã) và bác hai đề xuất khác sau khi tự kiểm (chồng lấn mark — đã phân xử ở
`store.rs:722`/`:810`, dấu lên dây không bao giờ chồng nhau; Global hiện ở Chương khác đang mở
song song — workspace chỉ mở một Chương một lúc). Cả 15 mục **ĐÃ VÁ**, không mục nào bị hạ
xuống nợ:

- **P1 (Cao)** — `switchChapter()` gọi `ensureGlossaryMarksLoaded` thiếu guard khớp Chương mà
  `GridPanel.vue`'s watcher đã có; hai lượt chuyển Chương liên tiếp nhanh có thể lệch cặp
  `chapterId`/`source_lang`. Thêm đúng guard `chapterId.value === sourceChapter.value.chapter_id`.
- **P2 (Cao)** — `applyRegroup` gán `segments.value = next` ĐỒNG BỘ trong khi `refreshGlossaryMarks`
  BẤT ĐỒNG BỘ, mở một cửa sổ render "segment mới + mark cũ". Vá: `resetGlossaryMarks()` ĐỒNG BỘ
  ngay trước khi đổi `segments.value` — không còn cửa sổ nào để lộ ra.
- **P3 (Cao, test)** — đường chuyển Chương kề chưa ca nào lái qua đường dấu Glossary. Thêm ca ở
  `glossaryMarksRefresh.test.ts` (mock `openAdjacentChapter`, hai bộ dấu cho hai Chương), đối
  chứng đỏ-xanh THẬT (gỡ tạm `ensureGlossaryMarksLoaded` ⇒ cả 5 ca liên quan đỏ, không riêng
  ca này — funnel kép của story khiến việc gỡ RIÊNG lời gọi ở `switchChapter` không đủ để một
  mình ca này đỏ, vì watcher của `GridPanel.vue` cũng độc lập tải đúng dấu; ghi ra thay vì giấu).
- **P4 (Cao, test)** — dây `GridPanel → SourceHanViet` qua `:glossary-terms` chưa từng được lái
  qua một lượt mount `GridPanel` thật. Thêm ca chuyển `activeTab` sang `'han_viet'` bằng
  `selectSourceTab()` thật, khẳng định `.hv-word` mang đúng lớp — đối chứng đỏ-xanh THẬT đã
  chạy (gỡ prop `:glossary-terms` ⇒ đúng một ca này đỏ).
- **P5 (Vừa)** — thuật ngữ đang hover bị sửa/xoá qua một lượt refresh, `StatusBar` giữ bản dịch
  CŨ. Vá: `clearHoveredGlossaryTerm()` ở mọi lượt `marks.value` đổi (`loadMarksFor`,
  `resetGlossaryMarks`).
- **P6 (Vừa)** — mark `is_confirmed=true` mà `translation=null` (vi phạm hợp đồng) làm
  `StatusBar` hiện "Bản dịch: " rỗng. Siết type guard: bất biến CHÉO trường
  (`is_confirmed ? translation string : translation null`) — mark hỏng bị TỪ CHỐI nguyên khối,
  không lọt qua như dữ liệu hợp lệ.
- **P7 (Vừa)** — lỗi tải dấu chiếm nhánh thứ năm CỦA `StatusBar` SUỐT PHIÊN, che "Đã lưu N giây
  trước" vĩnh viễn. Vá: `GLOSSARY_MARKS_ERROR_DISPLAY_MS` (8s), một `setTimeout` tự tắt câu lỗi,
  `{immediate:true}` để một lỗi đã tồn tại TRƯỚC khi `StatusBar` mount vẫn hiện ngay.
- **P8 (Vừa, tài liệu)** — mệnh đề "đạt vế bàn phím với 0 tab-stop mới" bị đọc RỘNG hơn số đo:
  chỉ ĐÚNG cho đường Hán Việt. Đường chữ trần không `tabindex`, phụ thuộc lệnh CÓ SẴN
  `selection.focus_source` (`⌘⌥S`) — sửa mệnh đề tại chỗ kèm 🔵, mở mục sổ nợ MỚI có chủ (cần
  nghiệm thu tay trên webview thật, đường chữ trần).
- **P9 (Vừa, test)** — AC "mở Chương rồi gõ 200 phím ⇒ đúng 1 lượt" chỉ đứng bằng lập luận đọc
  mã. Thêm ca đếm THẬT: 200 lượt `noteEditorEdit` sau khi mở Chương, khẳng định
  `soLuotGoiMarks` không đổi.
- **P10 (Thấp)** — type guard chấp nhận `start`/`end` là `NaN`/âm/phân số/`start > end`. Siết:
  `Number.isInteger` + `>= 0` + `end > start` nghiêm ngặt.
- **P11 (Thấp, test)** — nhánh guard "Chương chưa mở" của `refreshGlossaryMarksAfterSave` chưa
  ca nào canh. Thêm ca: lưu thành công mà KHÔNG có Chương nào mở ⇒ lưu vẫn `true`, KHÔNG lượt
  `glossaryMarksForChapter` nào phát.
- **P12 (Thấp, test)** — điểm cắt người dùng rơi GIỮA một span thuật ngữ (không trùng biên ICU
  lẫn biên thuật ngữ) chưa ca nào đo. Thêm ca: mark phủ TRỌN một segment 2 ký tự (không tự sinh
  biên), `pendingCut` ở giữa ⇒ hai mảnh, CẢ HAI vẫn mang lớp dấu, `cut-mark` đúng vị trí. Không
  cần sửa mã — cơ chế hợp hai tập điểm đã đúng sẵn, chỉ thiếu bằng chứng đo được.
- **P13 (Thấp → thực ra Cao, phát hiện lúc vá)** — `onSourceSelectionChange` dọn hover VÔ ĐIỀU
  KIỆN khi vùng chọn ở NGOÀI cột nguồn, tức MỌI phím gõ ở cột bản dịch xoá một hover chuột đang
  hiện — đường THƯỜNG NGÀY, không phải biên hiếm như doc-comment cũ ngụ ý. Vá:
  `if (cell === null) return` TRƯỚC bước dọn — chỉ dọn khi vùng chọn THẬT SỰ trong cột nguồn mà
  không trúng dấu. Đối chứng đỏ-xanh THẬT đã chạy.
- **P14 (Thấp, tài liệu)** — mục sổ nợ "Bàn phím KHÔNG tới được dấu" dựng khung rủi ro dài
  TRƯỚC dòng `→ ✅ ĐÃ ĐÓNG`, dễ đọc nhầm là đang MỞ. Thêm một dòng dẫn ngay sau tiêu đề.
- **P15 (Thấp, tài liệu)** — `.find()` ở ba nơi (`glossarySpanAt`/`glossarySpanAtPoint`,
  `glossarySpanFor`) dựa vào bảo đảm KHÔNG CHỒNG LẤN của Rust (`store.rs:722`,
  `resolve_overlaps`) mà không tài liệu nào nói ra. Thêm một chú thích tại nguồn của mảng
  `spans` (`glossaryMarksMap.ts`), giải thích hệ quả nếu bảo đảm đó bị phá.

⚠️ **`npm run check:lint` ĐỎ THẬT một lần trong vòng này** (5 lỗi `no-unnecessary-condition` ở
ca P12 mới thêm — `?.` thừa trên `NodeList` đã thu hẹp kiểu) — sửa bằng cách bỏ `?.` thừa, đúng
khuôn lỗi tương tự đã gặp ở vòng review trước. Ghi ra vì đây là lượt `check:lint` đỏ THẬT đã
được đọc và sửa, không phải một khẳng định "sạch" chưa chạy lại.

### Rà ba lớp LẦN BA (Ice, 2026-08-21) — bản vá P7 tự nó không có ca nào canh

Sau khi vòng LẦN HAI đóng, Ice đối chứng độc lập rồi chỉ ra: bản vá của P7
(`StatusBar.vue::GLOSSARY_MARKS_ERROR_DISPLAY_MS` + `watch` + `setTimeout` tự tắt câu lỗi sau
8s) không có một ca nào canh — gỡ `setTimeout` ra thì đúng khuyết tật P7 tái diễn (câu lỗi che
"Đã lưu N giây trước" vĩnh viễn) mà toàn bộ 323 ca vẫn xanh, đúng lớp *"vi phạm được mà không
cổng nào đỏ"*.

Thêm một ca ở `glossaryMarksRefresh.test.ts` (nhà đúng — `glossaryMarksError` là `readonly`
nên phải lái qua một lượt IPC trượt thật, không đặt thẳng được từ `statusBar.test.ts`), canh
ĐÚNG HAI mệnh đề (không chỉ "câu lỗi biến mất" — vế đó một mình bỏ sót đúng cái hại mà P7 tồn
tại để chống):
1. lỗi tải dấu ⇒ câu lỗi hiện trên `StatusBar`, che "Đã lưu…";
2. sau `GLOSSARY_MARKS_ERROR_DISPLAY_MS` ⇒ câu lỗi TẮT và "Đã lưu N giây trước" TRỞ LẠI.

Harness: `vi.useFakeTimers()` bật SAU khi `mountGrid()` + một lượt `flushEditorNow()` THẬT đã
xong (cả hai cần `setTimeout` thật để tự nhường vòng sự kiện), và TRƯỚC lượt đặt lỗi (chính lượt
dựng `setTimeout` mà ca này cần điều khiển) — đóng lại bằng `vi.useRealTimers()` trong `finally`
để không rò sang ca sau. `vi.advanceTimersByTimeAsync(7999)` xác nhận câu lỗi CÒN trước mốc,
`+2` xác nhận nó tắt ĐÚNG lúc — không phải một lượt xoá tức thời trùng hợp.

**Đối chứng đỏ-xanh THẬT đã chạy**: gỡ tạm khối `setTimeout` khỏi `StatusBar.vue` ⇒ đúng MỘT ca
(ca mới này) đỏ, 9 ca còn lại của tệp vẫn xanh; khôi phục ⇒ xanh lại. `glossaryMarksRefresh.test.ts`
lên **10** ca; tổng cây test lên **28 tệp, 324 ca**.

## Verification

**Commands — chạy thật 2026-08-21, kết quả dưới đây:**
- `npm run test` -- ✅ xanh. 🔵 **Số CUỐI sau SÁU lượt sửa của vòng nghiệm thu: 28 tệp, 324 ca**
  *(304 → 310 → 316 → 317 → 323 → **324**, mỗi lượt hết đúng, sửa tại chỗ 2026-08-21 — xem
  §"Rà ba lớp LẦN HAI (P1–P15)" và §"Rà ba lớp LẦN BA (P7 tự canh)" ngay dưới)*. Gồm
  `glossaryMarksMap.test.ts` (12 ca) · `hanVietCutAnchors.test.ts` (28 ca) ·
  `glossaryHoverSelection.test.ts` (8 ca — vế tiêu điểm + P13) · `glossaryMarksRefresh.test.ts`
  (**10** ca — Matrix Test Audit + P3/P4/P9/P11/P12 + P7 tự canh) · `statusBar.test.ts`
  (12 ca — +2 ưu tiên câu báo).
- `npm run build` -- ✅ xanh (`vue-tsc --noEmit` × 2 + `vite build`, 0 lỗi kiểu).
- `npm run check:tokens` -- ✅ xanh. "Tầm quét: 64 tệp (61 component)" — khớp đúng số đã tính
  khi nâng `FILE_FLOOR`/`COMPONENT_FILE_FLOOR`.
- `npm run check:panel-refs` -- ✅ xanh — "44 tep `.ts`, 109 o nho cap module, 25 mien tru co
  ten".
- `npm run check:commands` -- ✅ xanh — "17 tệp `.vue` + 44 tệp `.ts` · 26 `@click` · 37 lời gọi
  `dispatch()`". `@mouseenter`/`@mouseleave` mới thêm không đi qua Kiểm A (chỉ canh `@click`),
  đúng như spec ghi.
- `npm run check:layout` -- ✅ xanh — "Tầm quét: 61 tệp dưới `src/**`".
- `npm run check:i18n` -- ✅ xanh — "252 khoá" (251 cũ + 1 mới), 178 text node qua `t()`/`tError()`.
- `npm run check:debt-owner` -- ✅ xanh — "0/332 mục mở thiếu Chủ".
- `npm run check:lint` -- ✅ xanh (2 lỗi ESLint ban đầu ở test mới — `?.` thừa trên giá trị
  không thể `null`/`undefined` — đã sửa).
- `npm run check:deps` · `npm run check:gates` · `npm run check:dict` ·
  `npm run check:dict-manifest` -- ✅ xanh, không đổi (story không chạm phụ thuộc/từ điển).
- **Cả mười một cổng của `.githooks/pre-push` đã chạy riêng lẻ, đều xanh** — không có cửa sổ
  Tauri đang mở cổng 1420 trong môi trường triển khai nên không chạy `check:scope`/
  `check:scope:bundled` (đúng ngoại lệ đã khai của chính `pre-push`).
- `cd src-tauri && cargo test --locked` -- ✅ xanh, **0 thất bại** trên toàn bộ cây test (23 bộ).
  Story không đổi một dòng mã Rust **sản phẩm** nào (cửa ASK-FIRST về hình dạng dây KHÔNG kích
  hoạt) — chỉ `tests/glossary_marks_contract.rs` nhận **hai** ca mới, cộng một tệp bench tạm đã
  chạy và XOÁ, xem Spec Change Log.
- 🔴 **Hai ca chất nối chạy ĐÍCH DANH, không suy từ "bộ test xanh"** — một test tồn tại mà bị lọc
  ra thì tính là thiếu:
  `cargo test --locked --test glossary_marks_contract newline_joiner` ⇒ **2 passed, 0 failed,
  14 filtered out**, cả hai xanh:
  `a_chinese_term_placed_right_across_the_newline_joiner_produces_no_mark` ·
  `an_english_multi_word_term_placed_right_across_the_newline_joiner_produces_no_mark`.

🔴 **Matrix Test Audit (Ice, 2026-08-21) — bốn hàng I/O Matrix KHÔNG có ca nào canh, đóng
trong cùng lượt:**
- **"Đang có câu lỗi xác nhận ⇒ bản dịch KHÔNG đè lên nó"** -- `statusBar.test.ts` +2 ca: một
  khẳng định câu `no-caret` đứng dù `hoveredGlossaryTerm` đang mang giá trị thật (canh đúng
  THỨ TỰ `v-if`/`v-else-if`, thứ không cổng tĩnh nào bắt được nếu ai đảo hai nhánh), một đối
  chứng dương (không câu báo nào treo ⇒ bản dịch hiện bình thường).
- **"IPC trả lỗi ⇒ KHÔNG đánh dấu gì; lỗi qua `tError()`; KHÔNG coi như rỗng"** -- phát hiện
  một **khoảng trống THẬT**, không chỉ một khoảng trống của test: `glossaryMarksState.ts` giữ
  đúng bất biến phân biệt (`glossaryMarksHaveLoaded()` `false` khi lỗi) nhưng KHÔNG chỗ nào
  HIỂN THỊ lỗi đó — một lượt IPC trượt trông giống hệt "Chương sạch thuật ngữ". Đã vá:
  `StatusBar.vue::glossaryHoverText` nay đọc `glossaryMarksError` (import mới từ
  `glossaryMarksState.ts`) và trả `tError(err)` khi không có gì đang hover — cùng vị trí ưu
  tiên (nhánh thứ năm) mà câu hover thuật ngữ đã đứng, không một nhánh mới. `glossaryMarksRefresh.test.ts`
  +2 ca: `glossaryMarksHaveLoaded() === false` + `0` phần tử `.glossary-*` trên DOM, và câu lỗi
  thật sự lên `StatusBar`.
- **"Gộp/tách segment ⇒ dấu làm mới, không dấu nào trỏ vào segment đã về hưu"** VÀ **"thêm
  nhanh 3.3 ⇒ dấu xuất hiện không cần mở lại Chương"** -- `glossaryMarksRefresh.test.ts` +2 ca,
  thiết kế theo đúng ràng buộc *"kiểm hành vi, không kiểm một spy"*: lượt `glossaryMarksForChapter`
  ĐẦU luôn trả `marks: []` (0 dấu trước thao tác đang kiểm); chỉ lượt THỨ HAI (do
  `refreshGlossaryMarks()` phát ra) trả một dấu thật. Mệnh đề quan sát được: dấu chỉ xuất hiện
  trên DOM **SAU** gộp/lưu, và với gộp, dấu đứng đúng trên hàng MỚI trong khi hai `data-segment-id`
  đã về hưu **biến mất khỏi DOM**. **Đối chứng đỏ-xanh THẬT đã chạy**: gỡ tạm hai lời gọi
  `refreshGlossaryMarks()` (khỏi `applyRegroup` và `glossaryQuickAddState.ts`) ⇒ đúng hai ca
  này ĐỎ ngay (`expected null not to be null`), 316 ca còn lại vẫn xanh — xác nhận hai ca này
  THẬT SỰ canh đúng lời gọi, không xanh giả. Đã khôi phục nguyên trạng và chạy lại — xanh.

**Manual checks — thực hiện được TRONG lượt này:**
- **Bàn đo `3-4b-ban-do-danh-dau.html`** -- ✅ chạy thật bằng Chromium headless (0 lỗi JS):
  tương phản CHỮ đã chốt = **7,014:1** (sáng) / **6,688:1** (tối), cả hai ĐẠT AA (≥4,5:1);
  gạch chân chờ chốt cùng tỉ lệ, ĐẠT ngưỡng non-text (≥3:1, WCAG 1.4.11); `<rt>` xác nhận
  `text-decoration-line: none` ở cả hai theme (gạch chân KHÔNG bleed vào âm Hán Việt). Ảnh
  chụp đối chứng bằng mắt: ba bề mặt (chữ trần · Hán Việt chuyển đổi · Hán Việt song song),
  hai theme, dấu đã chốt/chờ chốt phân biệt rõ, `cut-here` không lẫn với gạch chân chờ chốt.
- **Cặp số mở Chương, lạnh/ấm** -- 🟡 **MỘT PHẦN**: đo được ở tầng Rust
  (`warm_jieba_for_source_lang` + `marks_for_source_text`, Glossary 5.000 mục, Chương 48.640
  ký tự, `cargo test --release`, `rustc 1.97.1`, Intel i9-9980HK, macOS/darwin 24.6.0,
  2026-08-21, 4 tiến trình riêng): **LẠNH 351–436 ms (trung vị ~402 ms) · ẤM 169–218 ms (trung
  vị ~189 ms)**. **CHƯA đo được** cặp số "cảm nhận được" trên webview thật (gồm IPC đọc
  Chương/segment + serialize + render DOM). ⚠️ **Không phải vì môi trường KHÔNG dựng được cửa
  sổ** — `npm run check:scope` CHẠY THẬT và ĐẠT trong lượt này (dựng cửa sổ Tauri, tự đóng khi
  xong) — mà vì đo con số "cảm nhận được" đòi một `.atproj` mang Glossary 5.000 mục + Chương
  48.640 ký tự thật, chưa có bộ dựng fixture đó, và đường gần nhất tới nó (`npm run test:e2e`)
  sửa `global.db` thật của người chạy (`e2e/wdio.conf.mjs` §Giới hạn) — một cái giá không nên
  trả ngoài một phiên Ice chủ động yêu cầu. Ghi đầy đủ + giới hạn ở `deferred-work.md`.

**Manual checks — CHƯA thực hiện được trong lượt này (quyết định phạm vi, KHÔNG phải cửa sổ
Tauri không dựng được — xem chú thích ⚠️ ngay trên):**
- **Vùng chọn** ở kiểu song song sau khi cấu trúc node đổi — chạy lại đúng phép kiểm của
  AC6/1.16 trên WKWebView **và** Chromium. **CHƯA CHẠY.** Cấu trúc DOM mà `buildSegments` sinh
  ra (nhiều Segment/node hơn khi có thuật ngữ khớp) về nguyên tắc KHÔNG động tới cơ chế
  `resolveParallel()`/`resolveSwitch()` (chúng đọc `host.children`/`segments.value` theo CHỈ
  SỐ, không theo số lượng cố định), nhưng đây là một LẬP LUẬN, không một PHÉP ĐO — đúng luật
  "đo lại, không suy từ số cũ" mà chính AC này đặt ra. Ghi nợ ở `deferred-work.md`.
- ~~**Một lượt e2e chạy tay** cho vế "webview thật" — **CHƯA CHẠY**~~
  🟡 **ĐÃ CHẠY 2026-08-21, Ice yêu cầu tường minh — và nó chứng minh MỘT nửa, không phải cả hai.**
  `npm run test:e2e` ⇒ **12/12 spec xanh, 8 phút 29 giây, `webkit 605.1.15 macos`** (WKWebView
  thật). Hàng rào dữ liệu đã kiểm trước khi chạy: `onPrepare` chuyển hướng **cả hai** bề mặt
  (`$APPDATA` qua `AURATRANSLATE_E2E_DATA_DIR`, thư mục gốc Library qua biến riêng) sang thư mục
  tạm, và `onComplete` tự kiểm **dương tính** rằng `global.db` NẰM TRONG thư mục tạm trước khi
  xoá.
  ✅ **Vế ĐƯỢC chứng minh — KHÔNG HỒI QUY trên engine thật, và nó không rỗng:** story đổi cấu
  trúc node của cột nguyên văn, và ba spec đi thẳng qua đúng chỗ đó đều xanh —
  `segment-merge-split` (⌘/ tách câu tại điểm bấm, tức `sourceCutOffsetOf` và **độ hạt
  click-để-cắt**), `segment-backspace-merge`, `segment-navigation`. Đây là bằng chứng **trên
  engine thật** cho ràng buộc 🔴 *"độ hạt click-để-cắt của Story 2.9 KHÔNG được đổi"* — thứ
  trước lượt này chỉ có `happy-dom` đỡ lưng.
  🔴 **Vế KHÔNG được chứng minh, ghi ra thay vì để đọc nhầm:** **0/12** spec chạm bề mặt đánh
  dấu (`grep 'glossary-confirmed|glossary-pending|glossaryMarks' e2e/specs/` = rỗng). ⇒ *"e2e
  xanh"* ở đây **KHÔNG** có nghĩa dấu thuật ngữ đã được nghiệm thu trên WKWebView — đúng cùng
  hình dạng với cái bẫy *"CI xanh không có nghĩa e2e đã chạy"* mà Story 3.4 đã trả giá để biết,
  chỉ đổi tầng. Một spec e2e cho bề mặt đánh dấu là món nợ **có chủ** ở `deferred-work.md`.
  `deferred-work.md:5781` (chuột kéo thật WKWebView) ở lại **MỞ**, không đóng bởi story này.
- **Lượt CI sau khi push** -- **CHƯA CÓ**, vì story chưa được push. Đọc lượt CI (cả macOS lẫn
  Windows) là bước của Ice sau khi nhận bàn giao, đúng luật đã ghi ở mọi story trước.

## Suggested Review Order

**Đường ánh xạ offset — chỗ cả story đứng hoặc đổ**

- Điểm vào: chia dấu tuyệt đối về từng segment, phép cộng dồn sống ĐÚNG một chỗ.
  [`glossaryMarksMap.ts:98`](../../src/panels/glossaryMarksMap.ts#L98)

- Chất nối `\n` — mắt xích mà `ChapterSegment` không mang offset nào buộc phải có.
  [`glossaryMarksMap.ts:152`](../../src/panels/glossaryMarksMap.ts#L152)

- Bảo đảm KHÔNG chồng lấn đến từ Rust, không từ đây — kiểm bằng hai ca thường trực.
  [`glossary_marks_contract.rs:564`](../../src-tauri/tests/glossary_marks_contract.rs#L564)

**Cắt ở tầng dữ liệu, không chèn node — bất biến đứng theo cấu tạo**

- `buildSegments` nay nhận tập biên; ranh giới `Matcher` thắng ranh giới ICU.
  [`SourceHanViet.vue:199`](../../src/panels/SourceHanViet.vue#L199)

- 🔴 HAI tập điểm gộp làm một tập CẮT, nhưng dấu ngắt đoạn chỉ vẽ theo `pendingCuts`.
  [`GridPanel.vue:1638`](../../src/panels/GridPanel.vue#L1638)

- Dây `GridPanel → SourceHanViet`: span đã chia sẵn theo segment, không offset Chương.
  [`GridPanel.vue:1620`](../../src/panels/GridPanel.vue#L1620)

**Tần suất gọi — thứ giữ 214 ms ngoài trần NFR2**

- Guard khớp Chương ở `switchChapter`; thiếu nó là dấu Chương này gán cho Chương kia.
  [`editorPanelState.ts:1575`](../../src/panels/editorPanelState.ts#L1575)

- Dọn dấu ĐỒNG BỘ trước khi thay `segments` — đóng cửa sổ dấu cũ trên bố cục mới.
  [`editorPanelState.ts:2021`](../../src/panels/editorPanelState.ts#L2021)

- Làm mới sau thêm nhanh: một lời gọi tường minh, không một event bus ngầm.
  [`glossaryQuickAddState.ts:314`](../../src/glossaryQuickAddState.ts#L314)

**Một cơ chế, hai đường vào — chuột và caret**

- `selectionchange` đạt vế tiêu điểm với 0 tab-stop mới, tái dùng `sourceCutOffsetOf`.
  [`GridPanel.vue:1054`](../../src/panels/GridPanel.vue#L1054)

- Nhánh thứ năm của thanh: dưới ba câu báo, trên "Đã lưu" — thứ tự là hợp đồng.
  [`StatusBar.vue:388`](../../src/StatusBar.vue#L388)

- Câu lỗi tự tắt sau một khoảng, thay vì che "Đã lưu" suốt phiên.
  [`StatusBar.vue:87`](../../src/StatusBar.vue#L87)

**Biên IPC**

- Adapter thứ tư, ba trạng thái, không bao giờ ném.
  [`glossary.ts:288`](../../src/config/glossary.ts#L288)

- Type guard lúc chạy: dữ liệu qua dây là một LỜI KHAI, không một bảo đảm.
  [`glossary.ts:237`](../../src/config/glossary.ts#L237)

**Phụ trợ**

- Ba hàng Matrix từng hở, mỗi ca kèm đối chứng "gỡ lời gọi thì ĐỎ".
  [`glossaryMarksRefresh.test.ts:1`](../../tests/frontend/glossaryMarksRefresh.test.ts#L1)

- Vế caret trên component mount thật, `Selection`/`Range` thật.
  [`glossaryHoverSelection.test.ts:121`](../../tests/frontend/glossaryHoverSelection.test.ts#L121)

- Sàn quần thể đã nâng theo ba tệp `.ts` mới, số đo bằng `find` kèm ngày.
  [`check-panel-refs.mjs:517`](../../scripts/check-panel-refs.mjs#L517)
