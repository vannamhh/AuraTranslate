---
title: 'Story 6.4 — Chuẩn hoá xuống dòng và khoảng trắng'
type: 'feature'
created: '2026-09-04'
status: 'done'
review_loop_iteration: 1
baseline_commit: 'd20fe679132f81b31e2b8607189264d01745071c'
context:
  - '{project-root}/AGENTS.md'
  - '{project-root}/src/AGENTS.md'
  - '{project-root}/src-tauri/AGENTS.md'
  - '{project-root}/_bmad-output/implementation-artifacts/epic-6-context.md'
---

<frozen-after-approval reason="human-owned intent — do not modify unless human renegotiates">

## Intent

**Problem:** Bước 4 của chuỗi AD-39 là **no-op thuần** — `pipeline.rs:349-352` chỉ `trace.push(step); flow`, không đổi một byte. Hệ quả đo được, không suy đoán: `split.rs:243-252` coi `\n` **và** `\r` là ranh giới **CỨNG**, nên một dòng bị ngắt giữa câu ra **đúng hai segment**, và AD-4 (spine `:95,:101`) đóng băng ranh giới đó xuống `.atproj` **vĩnh viễn** — không đường mã nào tính lại. Sổ nợ đã ghi đích danh từ 2026-08-06 (`deferred-work.md:744,751`): mọi Chương nhập từ tệp Windows đang mang `\r\n` trong `chapter.source_text`, và bộ tách chỉ **tự phòng thủ**, không chuẩn hoá.

**Approach:** Cho thân thật vào **đúng** bước 4 bằng một module thuần mới `core/segment/normalize.rs`. Vì `source_text` ghi xuống chính là chuỗi ra khỏi chuỗi bảy bước (`commands/project.rs:348-353`), tiêm ở bước 4 làm AC4 và AC5 thành **hệ quả cấu trúc**, không cần đụng đường ghi. Thêm một tầng xem trước hiện văn bản đã chuẩn hoá kèm **số đếm phép biến đổi**, để thiệt hại đếm được **trước** khi xác nhận.

## Boundaries & Constraints

**Always:**
- Chuẩn hoá là hàm **THUẦN** của văn bản đã giải mã: cùng đầu vào ⇒ cùng đầu ra. Không đọc đồng hồ, kho, cấu hình, hay `source_lang` nào ngoài tham số.
- **Nối dòng CHỈ KHI** dòng kết thúc mà **không** có dấu kết câu (đã bỏ qua dấu đóng ngoặc/ngoặc kép ở đuôi), **và** hai dòng nằm trong **cùng một đoạn**. Một dòng trống là **ranh giới đoạn** — không bao giờ nối qua nó.
- 🔴 **Bảng dấu kết câu không được sao chép.** `mod.rs:19-23` khai `split` là chỗ **DUY NHẤT** trong kho biết bảng ấy và `tests/segment_boundary.rs` cưỡng chế trên cả cây nguồn. Vị từ "dòng này kết thúc một câu chưa" phải sống **trong** `split.rs` cạnh bảng, và `normalize.rs` **gọi** nó. Cùng luật cho bảng "ngôn ngữ nối không dấu cách": `regroup::source_joiner` (`:100-107`) là chủ duy nhất.
- **Khoảng trắng:** trim **hai đầu** mỗi dòng bằng `str::trim()` — đúng tập 25 điểm mã `White_Space`, cùng tập mà `schema.rs:305-323` đã khai triển tay. **Không** đụng khoảng trắng **giữa** dòng.
- `PIPELINE_ORDER` và bảy bước **KHÔNG đổi**. Story này cho **thân** vào bước 4, không thêm bước.
- **Đổi ứng viên bảng mã vẫn phải là 0 lời gọi IPC** — ba ca đang canh (`importPreviewEncoding.test.ts:123,161,192`). ⇒ Bản dựng chuẩn hoá và số đếm của **cả năm** ứng viên đi kèm sẵn trên dây.
- 🔴 **Số đếm và bản dựng phải nhìn CÙNG một cửa sổ bằng chứng, và cửa sổ ấy phải nói ra trên màn hình.** Đây là đúng cái bẫy vòng rà 1 của Story 6.3 đã bắt được (§Design Notes 6.3: phép so trên bản cắt ngắn kết luận sai). Cửa sổ cắt ở **ranh giới dòng** và **bỏ dòng cuối** — quyết định nối của dòng cuối phụ thuộc byte nằm ngoài cửa sổ.
- Ô nhớ cấp module mới (nếu có) phải nằm trong `resetImportPreview()` (`importPreviewState.ts:311-326`) — `check:panel-refs` Kiểm A.

**Ask First:**
- Muốn nối **qua** một dòng trống, hoặc đụng khoảng trắng **giữa** dòng.
- Muốn chuẩn hoá lại dữ liệu **đã có** trên đĩa (một lượt di trú).
- Muốn thêm bất kỳ phụ thuộc mới nào — cửa NFR15 ba bước, mở tệp giấy phép trong nguồn đã tải mà đọc.

**Never:**
- Không chuẩn hoá Unicode NFC/NFD — đòi crate mới, nợ của **Ice** (`deferred-work.md:4344`).
- Không xoá chữ theo luật mẫu — FR124, **Story 6.5**. Không tách Chương — FR14, **Story 6.6**.
- Không lớp hiển thị đắp lên bản gốc (AC4; AD-39 spine `:475`, AD-4 spine `:101`).
- Không di trú `chapter.source_text` đã có trên đĩa.
- Không command IPC mới, không mục ACL mới, không đường gọi `run_import` thứ hai.

## I/O & Edge-Case Matrix

| Scenario | Input / State | Expected Output / Behavior | Error Handling |
|---|---|---|---|
| Ngắt giữa câu | `"Hắn nhìn về phía\nngọn núi xa."` (`en`) | `"Hắn nhìn về phía ngọn núi xa."` — nối bằng **một dấu cách** | N/A |
| Ngắt giữa câu, zh | `"他转过头看向\n远处的山。"` (`zh`) | nối bằng **chuỗi rỗng**, không dấu cách | N/A |
| Dòng đã trọn câu | `"他转过头。\n“谁？”"` | **KHÔNG nối** — hai dòng ở lại | N/A |
| Dấu kết + dấu đóng | `"「走吧。」\n第二天。"` | **KHÔNG nối** — đuôi đóng được bỏ qua trước khi xét | N/A |
| Dòng trống thừa | `"A。\n\n\n\n\nB。"` | `"A。\n\nB。"` — đúng **một** dòng trống | N/A |
| Không nối qua dòng trống | `"Hắn nhìn về phía\n\nngọn núi xa."` | giữ nguyên ranh giới đoạn, **không** nối | N/A |
| CRLF / CR trần | `"A。\r\nB。"` · `"A。\rB。"` | cả hai thành `"A。\nB。"` | N/A |
| Trim hai đầu dòng | `"\u{3000}\u{3000}他走了。   \n"` | `"他走了。"` — U+3000 và dấu cách đuôi biến mất | N/A |
| Chỉ khoảng trắng | `"\u{3000}\n \t \r\n "` | `""` ⇒ Chương **0 segment**, chỗ gọi đã chịu được | `ImportError::EmptyImport` như hôm nay |
| Bất động | văn bản đã chuẩn hoá | chạy lần hai cho **cùng chuỗi** (idempotent) | N/A |
| Cửa sổ xem trước | văn bản dài hơn cửa sổ | bản dựng cắt ở ranh giới dòng, **bỏ dòng cuối**; màn hình nói ra phạm vi | N/A |
| Đổi ứng viên | người dùng chọn ô khác | bản dựng + số đếm đổi ngay, **0 lời gọi IPC** | N/A |

</frozen-after-approval>

## Code Map

**Rust — điểm tiêm và chủ sở hữu bảng**
- `src-tauri/src/core/segment/pipeline.rs:349-352` — nhánh `Step::NormalizeParagraphsAndWhitespace`, hôm nay `trace.push(step); flow`. **ĐIỂM TIÊM DUY NHẤT.** 🔴 `trace.push` phải ở lại **bên trong** nhánh (AC6 spec 6.2, lý do ở `:44-49`).
- `pipeline.rs:104-112` `PIPELINE_ORDER` · `:119-138` `validate_order` · `:314-397` `run_import_with_order` · `:402-404` `run_import` — **không đụng**.
- `pipeline.rs:41-43` doc-comment "BỐN BƯỚC THÂN RỖNG" liệt bước 4; `pipeline.rs:545-556` `split_on_literal` cố ý **bỏ** `.map(str::trim)` và trỏ sang bước 4; `pipeline.rs:467-469` `strip_bom` khai CRLF để dành FR124/125 — **cả ba hết đúng**, sửa tại chỗ kèm 🔵.
- `pipeline.rs:269-272` `Unit::{Undecoded,Decoded}` · `:284-300` `Flow` — chuẩn hoá áp trên `Unit::Decoded`; `Undecoded` ở bước này là bất khả (bước 1 đứng trước).
- `src-tauri/src/core/segment/split.rs:51,57,65` `ZH_TERMINATORS`/`EN_TERMINATORS`/`TRAILING_CLOSERS` — **private, phải ở lại private**. `:303` `is_terminator` · `:328` `absorb_trailing_closers` · `:48` `pub const LANG_CHINESE`. `:243-252` ranh giới cứng `\n`/`\r`. `:213-216` doc-comment khai chuẩn hoá là Epic 6 — **hết đúng một nửa**, sửa kèm 🔵.
- `src-tauri/src/core/segment/regroup.rs:100-107` `source_joiner` — chủ duy nhất của bảng nối theo ngôn ngữ, kèm mệnh đề cấm nguồn sự thật thứ hai (`:92-99`). Hiện **private** — mở `pub(super)`.
- `src-tauri/src/core/segment/mod.rs:52-59` khai module; `:19-23` mệnh đề "split là chỗ duy nhất biết bảng kết câu" — **giữ đúng**, thêm dòng vai trò cho `normalize`.
- `src-tauri/src/core/segment/encoding.rs:101` `render_candidates` dựng năm bản — chỗ nối để bản dựng chuyển sang **đã chuẩn hoá**.
- `src-tauri/src/core/store/schema.rs:305-323` bảng 25 điểm mã `White_Space` + lý lẽ `:200-232` — **tham chiếu**, không chép sang Rust; `str::trim()` cắt đúng tập ấy.
- `src-tauri/src/commands/project.rs:284-288` gọi `run_import` · `:348-353` INSERT `source_text` — **không đụng**; AC4/AC5 là hệ quả của điểm tiêm.
- `src-tauri/src/commands/segment.rs:301-334` `split_chapter_into_segments` — chỗ **thứ hai** đọc `source_text` từ đĩa rồi tách; tự chặn bằng `already_split` (`:329-331`). Đọc để hiểu vì sao chuẩn hoá phải nằm ở `source_text`, không ở lúc tách.

**Cổng hiện có — cái nào đỏ, cái nào mù**
- `tests/segment_pipeline_boundary.rs:147` khoá `PIPELINE_ORDER` theo thứ tự · `:171` đếm chỗ gọi `run_import` **đúng 1** · `:132` sàn quần thể. ⚠️ **Không cổng nào đỏ khi thân một bước phình ra** — toàn bộ nội dung story này hôm nay **không ai canh**.
- `tests/segment_boundary.rs:373` `the_pipeline_module_actually_calls_the_splitter` — **khuôn** cho cổng thân-bước phải thêm; `:324` khuôn đếm chỗ gọi.
- `tests/segment_encoding_boundary.rs:109` khuôn `assert_eq!` bảng theo thứ tự · `:124-181` khuôn đếm tệp · `:182-192` khuôn **kiểm chứng dương** (ca dương + ca âm trên chính vị từ thật).
- `tests/segment_contract.rs:439,457,499,536,571,585` — mọi ca `\r\n`/dòng trống hiện có, **đều ở tầng `split_source_text`**, không ở tầng pipeline. ⚠️ **Không ca nào về gộp dòng.** `:8236` `every_step_of_pipeline_order_appears_in_the_trace_even_the_empty_bodied_ones` — thông báo lỗi `:8241` gọi bước 4 là "thân rỗng", **hết đúng**, sửa văn bản.

**Frontend — dây và bề mặt**
- `src/config/project.ts:139-143` `EncodingCandidateWire` · `:146-150` `ImportEncodingPreview` · `:162-186` kiểm kiểu **lúc chạy** (trường mới phải thêm mệnh đề, nếu không `undefined` lọt lên `.vue`) · `:224-231` `confirmImportWithEncoding` — **đường ghi duy nhất** (adapter cũ đã xoá, `:108-127`).
- `src/importPreviewState.ts:111-130` computed dải/ứng viên đang chọn · `:217-228` `selectImportPreviewCandidate` (chủ ý **không** gọi Rust) · `:136-138` `importPreviewEmptyReasonForTier(2|3)` — kiểu union hẹp, **không** mở rộng cho 6.4.
- `src/ImportPreviewOverlay.vue:181-232` tầng 1 · `:232-234` **chỗ chèn tầng mới** · `:235-240` khuôn một tầng · `:50-68` ánh xạ khoá bằng `switch` cạn (không ghép chuỗi) · `:436-442` mẫu chữ cỡ `read` qua token.
- `src/i18n/vi.json:206-221` khối `mode.library.preview.*` — chưa khoá nào cho tầng chuẩn hoá.
- `tests/frontend/importPreviewEncoding.test.ts:18-36` khuôn `freshState()`; `:41-71` ba fixture builder (đổi wire ⇒ TS bắt lỗi ở đây); `:352` ca `resetImportPreview()` quét mọi ô. `importPreviewEncodingWireShape.test.ts` khoá hình dạng dây.

**Nợ liên quan** — `deferred-work.md:744,751` (D1, chủ `Story 2.1 + Story 6.4/6.5`, vế Epic 6 còn mở) · `:9052-9067` (D3, không tầng nào cho FR125) · `:9135-9170` (D2, đổi ứng viên không chạy lại chuỗi) · `:8514-8537` (D4, chủ **Ice**: `chapter.source_text` còn là bản lưu trữ thô không) · `:1580-1586` (🔴 fixture mang CRLF có chủ ý; **cấm** `text=auto eol=lf` trong `.gitattributes`).

## Tasks & Acceptance

**Execution:**
- [x] `src-tauri/src/core/segment/split.rs` -- thêm `pub(super) fn line_ends_a_sentence(line: &str, chinese: bool) -> bool` **cạnh bảng**, dựng trên `absorb_trailing_closers` + `is_terminator` đang có; 🔵 sửa doc-comment `:213-216` -- bảng kết câu phải ở lại một chủ, nếu không `segment_boundary.rs` đỏ đúng
- [x] `src-tauri/src/core/segment/regroup.rs` -- mở `source_joiner` thành `pub(super)`, không chép -- `:92-99` cấm nguồn sự thật thứ hai bằng chữ
- [x] `src-tauri/src/core/segment/normalize.rs` -- tạo mới: `normalize(text, source_lang) -> Normalized { text, joined_lines, blank_lines_removed }`; ba phép theo thứ tự cố định (thống nhất xuống dòng → trim hai đầu mỗi dòng → gộp dòng giữa câu + thu dòng trống về một); `normalize_window(text, lang, max_bytes)` cắt ở ranh giới dòng và **bỏ dòng cuối** -- hàm thuần, không bảng riêng
- [x] `src-tauri/src/core/segment/mod.rs` -- khai `normalize`, thêm đoạn vai trò; giữ nguyên mệnh đề `:19-23`
- [x] `src-tauri/src/core/segment/pipeline.rs` -- nhánh `:349-352` **GỌI** `normalize::normalize` (đừng viết lại nội tuyến), `trace.push` ở lại trong nhánh; 🔵 sửa `:41-43`, `:467-469`, `:545-556` -- ba mệnh đề hết đúng, sửa tại chỗ kèm ngày
- [x] `src-tauri/src/core/segment/encoding.rs` + `src-tauri/src/commands/project.rs` -- bản dựng của **mỗi** ứng viên chuyển sang văn bản **đã chuẩn hoá** kèm hai số đếm, tính qua `normalize_window`; **không** command mới, **không** mục ACL -- giữ bất biến "đổi ứng viên = 0 IPC"
- [x] `src-tauri/tests/segment_normalize_boundary.rs` -- tạo mới: sàn quần thể; `pipeline.rs` **có gọi** `normalize::` (khuôn `segment_boundary.rs:373`); `normalize.rs` **0 dòng** mang ký tự của bảng kết câu hay chuỗi nối (chủ vẫn là `split`/`regroup`); đếm chỗ gọi sản phẩm; **kiểm chứng dương** ca dương + ca âm cho **mỗi** vị từ -- cổng phải đọc được sự LỆCH, không chỉ sự tồn tại
- [x] `src-tauri/tests/segment_contract.rs` -- ca hành vi ở **tầng pipeline** cho từng hàng ma trận I/O. 🔴 Ba ca DƯƠNG bắt buộc: (a) `run_import` trên byte CRLF ⇒ `source_text` đọc lại từ `project.db` **0 ký tự `\r`**; (b) văn bản ngắt giữa câu ⇒ **một** segment, không hai; (c) chạy `normalize` hai lần ⇒ cùng chuỗi. 🔵 sửa văn bản thông báo lỗi `:8241` -- ca ở tầng `split_source_text` không chứng minh gì cho tầng pipeline
- [x] `src/config/project.ts` -- thêm trường mới vào `EncodingCandidateWire` **và** mệnh đề kiểm kiểu lúc chạy ở `:162-170` -- thiếu vế thứ hai thì `undefined` lọt thẳng lên `.vue`
- [x] `src/importPreviewState.ts` -- computed dẫn xuất từ ứng viên đang chọn cho văn bản đã chuẩn hoá + hai số đếm; **không** ô nhớ mới nếu tránh được, có thì phải vào `resetImportPreview()`
- [x] `src/ImportPreviewOverlay.vue` -- tầng mới chèn ở `:232-234`, khuôn `:235-240`; hiện văn bản đã chuẩn hoá ở cỡ `read` qua token, hai số đếm, và **phạm vi cửa sổ** nói bằng chữ; khoá i18n qua `switch` cạn -- AD-16: dữ liệu, không markup, không `v-html`
- [x] `src/i18n/vi.json` -- khoá `mode.library.preview.tier_normalized_*` (tiêu đề · hai số đếm · nhãn phạm vi cửa sổ) -- khoá phải là literal trong `t('…')` để `check:i18n` thấy
- [x] `tests/frontend/importPreviewEncoding.test.ts` + `importPreviewEncodingWireShape.test.ts` -- cập nhật ba fixture builder và hình dạng dây; 🔴 giữ nguyên ba ca canh "đổi ứng viên = 0 IPC" (`:123,161,192`) **không sửa kỳ vọng**
- [x] `tests/frontend/importPreviewNormalized.test.ts` -- tạo mới: tầng hiện đúng bản dựng của ứng viên đang chọn; đổi ô ⇒ bản dựng và số đếm đổi mà **0 lời gọi IPC**; phạm vi cửa sổ hiện ra -- khuôn `importPreviewEncoding.test.ts:18-36`
- [x] `_bmad-output/implementation-artifacts/deferred-work.md` -- nối `→ 🟡` cho D1 (`:744`, vế FR125 đóng cho đường nhập TỚI, dữ liệu cũ trên đĩa vẫn mở) và cho D3 (`:9052`); ghi nợ MỚI có chủ cho: số đếm trên TOÀN Chương (**Story 6.10**, FR132 không liệt FR125 làm dấu hiệu "cần xem"), tiêu đề không dấu chấm bị nối oan (**Story 6.6**), di trú dữ liệu cũ (**Ice**) -- không mục nào mồ côi, không mục nào đóng khống

**Vá vòng rà 1 (2026-09-04) — xem §Spec Change Log:**
- [x] `src-tauri/src/commands/project.rs` + `src-tauri/src/core/segment/encoding.rs` -- 🔴 **đường tự khai phải có bản dựng chuẩn hoá**: ca `AlreadyText` (và mọi ca 0 ứng viên) vẫn phải chở một bản chuẩn hoá + hai số đếm trên dây, dựng từ chính văn bản tự khai. Cơ chế theo-ứng-viên KHÔNG phủ ca này -- không có nó, luật gộp dòng chạy mà người dùng không thấy gì
- [x] `src/importPreviewState.ts` + `src/ImportPreviewOverlay.vue` + `src/i18n/vi.json` -- tầng hiện bản chuẩn hoá của đường tự khai; nhãn số đếm đọc là **số CHỖ NỐI**, không phải "số dòng" (giá trị là K-1 cho một run K dòng); `normalized.text` rỗng phải có lời giải thích riêng, không để đoạn trắng
- [x] `src-tauri/src/core/segment/encoding.rs` -- thu `EVIDENCE_WINDOW_BYTES` về private và sửa chú thích khai `commands::project` cần nó -- không chỗ nào ngoài tệp này đọc nó; một chú thích sai còn tệ hơn một hằng private
- [x] `src-tauri/tests/segment_contract.rs` -- ca quan sát `.normalized` đi ra từ `render_candidates` THẬT: nguồn dài hơn `EVIDENCE_WINDOW_BYTES` ⇒ `window_truncated == true`; cùng byte ngắt giữa câu chạy `"en"` và `"zh"` ⇒ hai `.normalized.text` khác nhau đúng dấu nối -- dây nối là bề mặt hở nhất, hôm nay đột biến sống
- [x] `src-tauri/tests/segment_normalize_boundary.rs` -- neo `#[cfg(test)]` theo ĐẦU DÒNG (không `text.find` chuỗi trần) và cho `code_lines()` lọc cả `/* */` -- một lần nhắc chuỗi ấy trong chú thích làm cổng mù đúng mệnh đề nó tuyên bố canh
- [x] `src-tauri/src/core/segment/split.rs` -- ca unit riêng cho `line_ends_a_sentence` (dấu kết trần · dấu kết + đóng ngoặc · không dấu kết · dòng rỗng), cả hai nhánh ngôn ngữ -- seam mới phải tự canh, không mượn ca của module khác
- [x] `src/ImportPreviewOverlay.vue` -- gộp hai luật CSS `.ip-normalized-counts` / `.ip-normalized-window-note` trùng nguyên văn
- [x] `_bmad-output/implementation-artifacts/deferred-work.md` -- ghi ba nợ MỚI có chủ: xem trước sẽ lệch bản sản phẩm khi bước 2/3 có thân (**Story 6.5 / 6.9**) · `normalize()` chép toàn buffer hai lượt kể cả khi không có `\r` (**Story 6.18**, cùng lượt đo quy mô thật) · ghi chú peak-RSS 100 MB chưa tính lượt quét mới (**Story 6.18**)

**Acceptance Criteria:**
- Given một lượt nhập **dán văn bản tay** (0 ứng viên bảng mã), when mở màn xem trước, then tầng chuẩn hoá hiện **văn bản đã chuẩn hoá cùng hai số đếm** — AC6 của `epics.md` áp cho **mọi** đường đi qua xem trước, không riêng đường có ứng viên.
- Given `render_candidates`, when đột biến `window_truncated` hoặc tham số `source_lang` của nó, then ít nhất một ca **đỏ** — dây nối phải có người quan sát, không chỉ hàm thuần ở hai đầu.
- Given `PIPELINE_ORDER` và `segment_pipeline_boundary.rs`, when chạy sau story, then bảy bước và chỗ gọi sản phẩm duy nhất **không đổi** — story này thêm thân, không thêm bước.
- Given cổng mới ở `segment_normalize_boundary.rs`, when **gỡ** nó ra và chạy lại bộ test **CŨ**, then bộ cũ **xanh** — chứng minh mệnh đề mới thật sự chưa ai canh.
- Given cây nguồn sau story, when đếm tệp `src-tauri/src/**` mang ký tự bảng kết câu, then vẫn **đúng một** (`split.rs`), và `segment_boundary.rs` xanh **mà không sửa một kỳ vọng nào**.
- Given bộ Rust và vitest đang xanh trước story, when chạy sau story, then vẫn xanh mà không nới một kỳ vọng nào — trừ những chỗ chạm trực tiếp hình dạng dây ứng viên, nơi lượt sửa phải kèm lý do là **thêm trường**, không phải một mệnh đề đã nới.
- Given `npm run check:debt-owner`, when chạy sau story, then **0 mục mở mồ côi**, và D1/D3 đọc là đóng một nửa bằng **dòng `→`** — `check-debt-owner.mjs:221` chỉ đọc dòng `→`, một câu "đã đóng" trong thân mục thì cổng không thấy.
- Given mọi chỗ còn khai bước 4 là "thân rỗng" sau story, when soát từng chỗ, then mỗi chỗ là một chú thích 🔵 có ngày giải thích chính lượt này — không chỗ nào còn khai ngược sự thật.

## Spec Change Log

### Vòng rà 1 — 2026-09-04 (ba lớp rà đối kháng; `bad_spec`, Ice chốt vá tại chỗ)

**Phát hiện kích hoạt.** Tầng "Văn bản đã chuẩn hoá" **rỗng** trên đường **dán văn bản tay**.
Kiểm bằng mã, không bằng lời khai: `commands/project.rs` trả `Vec::new()` cho
`PipelineShape::Blob(ChapterInput::AlreadyText(_))` ⇒ 0 ứng viên ⇒
`importPreviewSelectedCandidate === null` ⇒ tầng hiện *"Chưa chọn được ứng viên bảng mã nào"*.

**Vì sao đây là lỗi của SPEC, không của bản thi hành.** Bản thi hành làm **đúng** chữ của
§Always (*"bản dựng của **cả năm** ứng viên đi kèm sẵn trên dây"*) — một cơ chế định nghĩa
theo ỨNG VIÊN, mà đường tự khai thì không có ứng viên nào. Lỗi thật: §Tasks và danh sách
§Acceptance Criteria **không mang AC6 của `epics.md` sang** (*"màn xem trước hiện văn bản đã
chuẩn hoá — đúng thứ sẽ được ghi"*), nên không mệnh đề nào buộc tầng ấy phủ ca 0 ứng viên.

🔴 **Trạng thái xấu mà lượt sửa này tránh.** §Design Notes viện **chính số đếm trên xem
trước** làm lý do chấp nhận luật gộp dòng rủi ro (*"thiệt hại nhìn thấy được trước khi xác
nhận"*). Trên đường dán tay, lý do đó **không tồn tại**: văn bản bị nối, AD-4 đóng băng, và
người dùng không thấy gì. Một lượt vá chỉ-sửa-mã sẽ để spec tái sinh đúng lỗ này ở lần dựng
lại sau — nên §Tasks và §AC được sửa cùng lượt, không chỉ mã.

**KEEP — bốn thứ đã nghiệm thu, phải sống sót mọi lượt dựng lại:**
1. Điểm tiêm ở **thân bước 4**, không thêm bước — `PIPELINE_ORDER` và `segment_pipeline_boundary.rs`
   (6 ca) không đổi, `run_import` vẫn đúng một chỗ gọi sản phẩm.
2. **Một chủ cho mỗi bảng**: `line_ends_a_sentence` sống trong `split.rs` cạnh bảng kết câu;
   `source_joiner` mở `pub(super)`, không chép. Cổng mới khẳng định `normalize.rs` mang **0**
   ký tự kết câu.
3. **Ba phép đúng thứ tự** (thống nhất xuống dòng → trim hai đầu mỗi dòng → gộp giữa câu +
   thu dòng trống về một), và `normalize_window` **bỏ dòng cuối** — đã có 13 ca unit phủ và
   một phép gỡ thật chứng minh cổng không rỗng (gỡ vế `\r` trần ⇒ `crlf_and_bare_cr_both_become_lf` đỏ).
4. Ba lượt sửa 🔵 có ngày cho các mệnh đề hết đúng (`project_contract.rs` CRLF, hai đối chứng
   AD-39, thông báo vết chạy) — **siết chặt**, không nới.

**Đã sửa trong lượt này:** §Tasks thêm đường dựng bản chuẩn hoá cho ca **tự khai** cộng tám
mục vá; §Acceptance Criteria thêm hai mệnh đề (AC6 phủ **mọi** đường qua xem trước; dây nối
`render_candidates` phải có ca quan sát).

## Design Notes

**Vì sao AC4 và AC5 không cần một dòng mã nào ở đường ghi.** Đo trước khi thiết kế: `commands/project.rs:348-353` INSERT `chapter.source_text` lấy thẳng từ `PipelineOutput`, và `pipeline.rs:610-620` tính ranh giới segment trên **cùng** chuỗi ấy ở bước 7. Bước 4 đứng trước bước 7 trong `PIPELINE_ORDER`. ⇒ Tiêm ở bước 4 làm *"kết quả chuẩn hoá là thứ được lưu"* và *"ranh giới tính trên văn bản đã chuẩn hoá"* thành **hệ quả cấu trúc**, không phải hai nghĩa vụ phải nhớ. Đây cũng là lý do phương án *"chuẩn hoá lúc tách segment"* bị loại: nó để `source_text` trên đĩa lệch khỏi chuỗi đã tính ranh giới, và `commands/segment.rs:334` — chỗ **thứ hai** đọc `source_text` từ đĩa rồi tách — sẽ cho kết quả khác. Chính lớp lỗi AD-4 tồn tại để cấm.

**Luật gộp dòng, và cái nó KHÔNG cứu được.** Ice chốt 2026-09-04: nối **chỉ khi** dòng kết thúc mà không có dấu kết câu — bám đúng chữ *"ngắt tuỳ tiện **giữa câu**"* của AC1 và đúng hại đã đo (`split.rs:243-252` cắt cứng ở `\n`). ⚠️ **Giới hạn phải ghi ra thay vì làm nhẹ đi:** một **tiêu đề không dấu chấm** đứng riêng một dòng mà **không** có dòng trống ở sau **vẫn bị nối** vào câu kế. Hai phương án chặn đều bị loại có lý do: một ngưỡng độ dài dòng là một con số phù thuỷ chưa đo, và một luật "dòng ngắn thì đừng nối" bác oan thoại ngắn. ⇒ Ba việc thay cho một phép đoán: (1) tầng xem trước **đếm** số dòng đã nối, nên thiệt hại **nhìn thấy được trước** khi xác nhận; (2) Story 6.6 bóc tiêu đề Chương ra khỏi thân trước khi chuyện này xảy ra; (3) tiêu đề **trong** Chương vào sổ nợ, chủ **Story 6.6**. Không cổng nào canh vế (3) hôm nay — đó là một sự vắng mặt đã đặt tên, không phải một cổng đã có.

**Vì sao bản dựng đi kèm sẵn KHÔNG phải là né tránh nợ D2.** `deferred-work.md:9135-9170` ghi rằng *"chọn ứng viên khác phải chạy lại chuỗi từ bước một"* nhưng bản thi hành 6.3 đọc `preview.candidates` dựng sẵn, và cảnh báo story **đầu tiên** dựng thân thật sẽ tính trên bảng mã cũ. Story 6.4 **là** story đó — nên tiền đề phải kiểm, không nhận. Kiểm được: chuẩn hoá là hàm thuần của **chuỗi đã giải mã** và **không gì khác**. ⇒ Với riêng bước 4, "dựng sẵn năm bản" và "chạy lại chuỗi cho ứng viên đã chọn" cho **cùng một kết quả** — quan sát không phân biệt được hai đường. Nợ D2 **ở lại mở**: tầng 2 (6.9) và tầng 3 (6.5) phụ thuộc trạng thái ngoài chuỗi đã giải mã, nên lý lẽ này **không** suy rộng sang chúng, và đóng D2 ở đây sẽ là đóng khống.

**Cửa sổ bằng chứng — cùng cái bẫy Story 6.3 đã bắt.** Vòng rà 1 của 6.3 đo được: một tệp GBK mở đầu 12 ký tự ASCII cho bốn ứng viên **cùng một chuỗi** ở bản cắt ngắn, nên một phép so trên bản hiển thị kết luận *tin cậy cao* đúng trên loại tệp FR126 tồn tại để cứu. Ở đây cùng hình dạng: một số đếm tính trên cửa sổ mà **khai** là số của cả Chương là một câu đúng hình dạng, sai sự thật. ⇒ Hai nửa (bản dựng và số đếm) nhìn **cùng** cửa sổ, cửa sổ **nói ra** trên màn hình, và số của **toàn** Chương vào sổ nợ với chủ **Story 6.10** — nơi FR132 đã có bề mặt đếm *"N Chương cần xem"*. ⚠️ PRD `prd.md:335` **không** liệt FR125 vào danh sách dấu hiệu "cần xem"; đó là một khoảng trống quy hoạch, ghi nợ chứ không tự sửa PRD.

**Cắt cửa sổ ở đâu.** Chuẩn hoá **không** bất động theo tiền tố: dòng cuối của một cửa sổ có nối hay không phụ thuộc dòng **sau** nó, tức byte ngoài cửa sổ. ⇒ `normalize_window` cắt ở ranh giới dòng rồi **bỏ dòng cuối**. Bỏ một dòng làm bản dựng **ngắn hơn** sự thật; giữ lại làm nó **sai**. Ngắn thì đọc được, sai thì không.

**Thứ tự ba phép là bắt buộc.** Thống nhất xuống dòng **trước** (nếu không, `"A。\r\n\r\nB。"` đọc thành một dòng trống có `\r`, và phép đếm dòng trống lệch); trim **trước** khi xét nối (nếu không, một dòng kết bằng `"。   "` không khớp bảng kết câu và bị nối oan — đúng ca `\u{3000}\u{3000}他走了。   ` trong ma trận I/O).

## Verification

**Commands:**
- `npm run build && cargo test --locked --manifest-path src-tauri/Cargo.toml` -- 🔴 **thứ tự bắt buộc**: thiếu `dist/` thì `cargo test` gãy ở khâu biên dịch chứ không ở một assert (`AGENTS.md:32`). Kỳ vọng: ≥ số nền của Story 6.3, 0 failed
- `cargo test --locked --manifest-path src-tauri/Cargo.toml --test segment_normalize_boundary --test segment_pipeline_boundary --test segment_boundary --test segment_contract --test ipc_contract` -- 0 failed; `segment_pipeline_boundary` giữ nguyên **6 ca** xanh, `segment_boundary` xanh **không sửa kỳ vọng**
- `npm run test` -- vitest 0 failed; ba ca "đổi ứng viên = 0 IPC" xanh không sửa kỳ vọng
- `npm run check:i18n && npm run check:panel-refs && npm run check:commands && npm run check:lint && npm run check:tokens` -- 0 findings
- `npm run check:debt-owner` -- **0 mục mở mồ côi**
- `npm run check:gates` -- 0 findings (đỏ nếu một cổng mới quên một trong ba danh sách)

**Manual checks (if no CLI):**
- Gỡ hẳn `segment_normalize_boundary.rs` rồi chạy lại **bộ test CŨ** — phải **xanh**. Một lượt đỏ nghĩa là cổng mới không canh mệnh đề mới mà đang canh thứ đã có chủ.
- Đột biến một dòng: đổi chuỗi nối của nhánh `zh` từ `""` sang `" "` trong `regroup::source_joiner` — ít nhất một ca ở `segment_contract.rs` phải đỏ. Xanh nghĩa là dây nối chưa ai canh.

## Suggested Review Order

**Điểm tiêm — đọc trước nhất**

- Thân bước 4 gọi xuống module thuần; `trace.push` ở lại trong nhánh.
  [`pipeline.rs:357`](../../src-tauri/src/core/segment/pipeline.rs#L357)

- Ba phép đúng thứ tự; nhóm theo đoạn rồi mới nối trong từng đoạn.
  [`normalize.rs:78`](../../src-tauri/src/core/segment/normalize.rs#L78)

**Một chủ cho mỗi bảng**

- Vị từ kết câu sống cạnh bảng, nên `normalize.rs` mang 0 ký tự kết câu.
  [`split.rs:362`](../../src-tauri/src/core/segment/split.rs#L362)

- Dấu nối theo ngôn ngữ chỉ mở tầm nhìn, không sao chép.
  [`regroup.rs:106`](../../src-tauri/src/core/segment/regroup.rs#L106)

**Cửa sổ bằng chứng — chỗ dễ nói dối nhất**

- Bỏ dòng cuối vì quyết định nối của nó nằm ngoài cửa sổ.
  [`normalize.rs:165`](../../src-tauri/src/core/segment/normalize.rs#L165)

- Ép nhánh cắt cho ứng viên; số đếm và bản dựng cùng một cửa sổ.
  [`encoding.rs:305`](../../src-tauri/src/core/segment/encoding.rs#L305)

**Vá vòng rà 1 — đường tự khai**

- Ca 0 ứng viên vẫn phải có bản dựng; cơ chế theo-ứng-viên không phủ nó.
  [`project.rs:1085`](../../src-tauri/src/commands/project.rs#L1085)

- Bản dựng tự khai, chuẩn hoá trên văn bản thật thay vì cửa sổ byte.
  [`encoding.rs:339`](../../src-tauri/src/core/segment/encoding.rs#L339)

- Đọc ứng viên khi có, rơi về bản tự khai khi không — giữ 0 lời gọi IPC.
  [`importPreviewState.ts:152`](../../src/importPreviewState.ts#L152)

- Tầng mới đặt ngay sau bảng mã, đúng thứ tự nhân quả của xem trước.
  [`ImportPreviewOverlay.vue:255`](../../src/ImportPreviewOverlay.vue#L255)

**Cổng — và chỗ mù đã vá**

- Cổng thân-bước: pipeline phải GỌI module, không viết lại nội tuyến.
  [`segment_normalize_boundary.rs:137`](../../src-tauri/tests/segment_normalize_boundary.rs#L137)

- Neo theo đầu dòng; một chú thích nhắc chuỗi ấy từng làm cổng mù.
  [`segment_normalize_boundary.rs:98`](../../src-tauri/tests/segment_normalize_boundary.rs#L98)

- Kiểm chứng dương cho chính phép neo trên — cổng đọc được sự lệch.
  [`segment_normalize_boundary.rs:231`](../../src-tauri/tests/segment_normalize_boundary.rs#L231)
