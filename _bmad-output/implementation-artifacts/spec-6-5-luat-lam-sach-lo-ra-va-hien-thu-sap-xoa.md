---
title: 'Story 6.5 — Luật làm sạch lộ ra và hiện thứ sắp xoá'
type: 'feature'
created: '2026-09-05'
status: 'done'
review_loop_iteration: 1
baseline_commit: '42117e5f0e5b430eefb2217908650bc6ab4306cb'
context:
  - '{project-root}/AGENTS.md'
  - '{project-root}/src/AGENTS.md'
  - '{project-root}/src-tauri/AGENTS.md'
  - '{project-root}/_bmad-output/implementation-artifacts/epic-6-context.md'
---

<frozen-after-approval reason="human-owned intent — do not modify unless human renegotiates">

## Intent

**Problem:** Bước 3 của chuỗi AD-39 (`pipeline.rs:348-351` `Step::CleanByRules`) là **no-op thuần** — `trace.push(step); flow`. FR124 gọi làm sạch là *loại duy nhất có thể xoá nhầm nội dung thật* (`prd.md:349`), nhưng hôm nay không có bảng luật, không có bề mặt xem/sửa/tắt, và tầng 3 của xem trước là một dòng chữ `tier_empty_story_6_5` (`ImportPreviewOverlay.vue:302-307`). Kèm theo: nợ `deferred-work.md:9359` (chủ **Story 6.5 / 6.9**) đo được rằng xem trước **không** chạy qua chuỗi — `preview_import_encoding` nhảy thẳng bước 1 → bước 4 trong `encoding.rs`, nên ngày bước 3 có thân, AC6 *"hiện đúng thứ sẽ được ghi"* sai trong im lặng và không cổng nào đỏ.

**Approach:** Một bảng luật ở **cả hai tầng** (cùng một hằng DDL, khuôn `GLOSSARY_ENTRY_DDL`), hợp nhất qua `ScopeResolver::apply_merge("import_cleanup_rule", …)` — hàng `Merge` đã có sẵn ở `kinds.rs:198`, story này là consumer sản phẩm **đầu tiên** của `Merge`. Cho thân vào bước 3 bằng module thuần mới, và **chuyển xem trước sang chạy chính chuỗi** thay vì dựng lại một bản thứ hai — đóng nợ `:9359` bằng cấu trúc chứ không bằng lời hứa. Ice chốt 2026-09-05: xuất xưởng **0 luật**, người dùng tự soạn (đủ ba động từ xem/sửa/tắt của FR124, và không luật nhà máy nào xoá nhầm chữ thật).

## Boundaries & Constraints

**Always:**
- 🔴 **Danh tính một luật trên dây là cặp `(tầng, id)`.** `glossary_entry` dùng `id INTEGER PRIMARY KEY AUTOINCREMENT` (`schema.rs:302`) và bảng luật theo cùng khuôn ⇒ hai tầng đánh số **độc lập**, nên luật Toàn cục #1 và luật Tác phẩm #1 cùng tồn tại. Một `rule_id` trần trên dây là hai luật đội lốt một.
- 🔴 **Chuỗi AD-39 chỉ có MỘT bản cài đặt** (spine `:498`). Xem trước phải gọi `run_import*`, **không** được dựng lại thứ tự bước trong `encoding.rs` hay `commands/project.rs`. `PIPELINE_ORDER` và bảy bước **KHÔNG đổi** — story này cho thân vào bước 3, không thêm bước.
- **Áp cả hai tầng, hợp nhất** (AD-18 spine `:250`), phân giải **chỉ** qua `ScopeResolver::apply_merge`. Không tự viết một phép trộn thứ hai. `scope_boundary.rs:71` cấm mã ngoài `core/scope/**` gõ `ScopeKind`/`Semantics` ⇒ truyền chuỗi literal `"import_cleanup_rule"`.
- **Số đếm phải nói đúng phạm vi nó đã đo.** Hai con số của mỗi luật là *trong Chương này* và *trong cả lần nhập*; cả hai đo trên **toàn** văn bản, không trên cửa sổ hiển thị. Bản dựng hiển thị **được phép** cắt, và khi cắt thì màn hình **nói ra**. (Đây là siết chặt, không nới, luật cửa sổ của Story 6.4 §Design Notes: cái bị cấm là một số đo trên cửa sổ mà **khai** là số của cả Chương.)
- **Luật đã TẮT vẫn được đếm và vẫn báo cáo chỗ khớp** — mockup ghi *"Đã tắt · nếu bật sẽ xoá thêm 12 chỗ"* (`web-import.html:355`). Tắt đổi việc **xoá**, không đổi việc **đo**.
- 🔴 **`ornament` là màu của NÉT, không bao giờ là màu của chữ** (`DESIGN.md:213`; `check:tokens` `neverTextTokens`, đo 2,44/2,64 — trượt AA). Gạch ngang dùng `text-decoration-color: var(--color-ornament)`, đúng như mockup `:103`.
- **Đổi ứng viên bảng mã vẫn phải là 0 lời gọi IPC** — ba ca đang canh (`importPreviewEncoding.test.ts:192-211` và hai ca kề). ⇒ Khối làm sạch của **cả năm** ứng viên đi kèm sẵn trên dây, **và** đường tự khai (0 ứng viên) phải có khối của riêng nó — đúng lỗ mà vòng rà 1 của Story 6.4 đã bắt được.
- **Bật/tắt và mọi lượt soạn là một lượt ghi THẬT** vào bảng luật rồi dựng lại xem trước; không trạng thái bật/tắt chỉ sống trong bộ nhớ frontend. Không quy tắc nghiệp vụ nào ở TypeScript (AD-1) — frontend chỉ render mô hình span mà Rust trả về (AD-16, không `v-html`).
- Mẫu regex biên dịch **một lần** cho mỗi lượt chạy; mẫu hỏng là một `IpcError` có `message_key` trong danh mục đóng `message_keys!`, không một `panic!` (`panic = "abort"` giết cả tiến trình).
- Ô nhớ cấp module mới phải nằm trong `resetImportPreview()` (`importPreviewState.ts:339-354`) — `check:panel-refs` Kiểm A.

**Ask First:**
- Muốn xuất xưởng bất kỳ luật mặc định nào (Ice đã chốt **0**).
- Muốn luật áp lên **cột đích** của đường song ngữ, hay lên tiêu đề Chương.
- Muốn thêm phụ thuộc nào **ngoài** `regex` — cửa NFR15 ba bước, mở tệp giấy phép trong nguồn đã tải mà đọc.
- Muốn áp luật lên dữ liệu **đã có** trên đĩa (một lượt di trú).

**Never:**
- Không nhét luật vào `config_value` — `schema.rs:82-85` và `core/scope/store.rs:15-18` cấm bằng chữ và nêu đích danh Story 6.5.
- Không port mới trong `ports/` — AD-2 khai đúng ba, cổng thứ tư phải là một AD mới. Khuôn đúng là `core/glossary/store.rs`.
- Không tách Chương (FR14, **Story 6.6**); không bóc nội dung chính, không bề mặt khối (FR123, **Story 6.9**); không bộ lọc "cần xem" (FR132, **Story 6.10**).
- Không sửa `epics.md`/`prd.md` cho khớp thứ dựng được — năng lực chưa dựng thì ghi nợ có chủ.
- Không command IPC nào bỏ qua xem trước; không đường gọi `run_import` thứ hai.

## I/O & Edge-Case Matrix

| Scenario | Input / State | Expected Output / Behavior | Error Handling |
|---|---|---|---|
| Không luật nào (mặc định) | 0 hàng ở cả hai tầng | Tầng 3 hiện văn bản **nguyên trạng**, 0 gạch ngang, lời mời soạn luật; `source_text` ghi xuống không đổi một byte | N/A |
| Mẫu chuỗi trần | luật `literal` `"求收藏"`, Chương có 3 chỗ | 3 chỗ gạch ngang; hai số đếm `3` / `3`; xác nhận ⇒ ba chỗ **biến mất** khỏi `source_text` | N/A |
| Mẫu regex | luật `regex` `^本章由.*整理$` | khớp theo dòng, gạch ngang đúng dải | N/A |
| Regex hỏng | người dùng lưu `[unclosed` | Lưu **bị từ chối**, bảng không đổi một hàng, thông báo qua `message_key` | `IpcError`, không `panic!` |
| Hai tầng cùng khớp | luật Toàn cục và luật Tác phẩm khớp **cùng** một chỗ | Chỗ đó xoá **một lần**; hai luật **đều** đếm nó — hợp nhất, không khử trùng lặp (`scope_contract.rs:327`) | N/A |
| Trùng id giữa hai tầng | Toàn cục #1 và Tác phẩm #1 | Hai hàng **riêng biệt** trong danh sách, hai nhãn tầng khác nhau, bật/tắt độc lập | N/A |
| Tắt một luật | người dùng bỏ tick | Chỗ vừa gạch ngang **trở về nguyên trạng** ngay; số đếm của luật ở lại; nhãn đổi sang "nếu bật sẽ xoá thêm N chỗ" | N/A |
| Đổi ứng viên bảng mã | chọn ô khác | Khối làm sạch (văn bản + span + hai số) đổi ngay, **0 lời gọi IPC** | N/A |
| Dán văn bản tay | 0 ứng viên bảng mã | Tầng 3 **vẫn** đầy đủ, dựng từ chính văn bản tự khai | N/A |
| Luật xoá sạch Chương | mẫu khớp toàn bộ văn bản | Xem trước hiện rõ **mọi thứ** bị gạch ngang; xác nhận ⇒ Chương có `source_text` **rỗng**, **0 segment**, **không lỗi** | N/A — xem 🔵 dưới bảng |
| Văn bản dài hơn cửa sổ | Chương lớn | Bản dựng cắt ở ranh giới dòng và **nói ra**; hai số đếm vẫn của **toàn** Chương | N/A |
| Mẫu rỗng / chỉ khoảng trắng | `"\u{3000}  "` | Lưu bị từ chối ở tầng DDL (`CHECK trim(...)` 25 điểm mã) **và** ở tầng lệnh | `IpcError` |

🔵 **SỬA 2026-09-05 (phán quyết Ice, khối đóng băng được mở đúng hàng này).** Hàng *"Luật xoá
sạch Chương"* trước đó khai `ImportError::EmptyImport`. Biến thể ấy **KHÔNG TỒN TẠI**: `grep -rn
"EmptyImport" src-tauri/src/` cho **0** kết quả, và `core::segment::import::ImportError` có đúng
tám biến thể (`UnsupportedFormat` · `UndecodableBytes` · `ReadFailed` · `MissingExtension` ·
`TooLarge` · `InvalidPipelineOrder` · `UnrecognizedEncoding` · `InvalidCleanupPattern`).
`create_work` chỉ từ chối khi `chapters.is_empty()` (0 Chương), **không** khi một Chương đơn có
`source_text` rỗng — hành vi đã có từ trước Story 6.5 (dán một chuỗi rỗng đi đúng đường này).
⇒ Mã ĐÚNG, hàng ma trận SAI. Cái tên ma này bị chép sang đây từ §I/O Matrix của spec 6.4
(`spec-6-4-…:59`, cũng đã sửa tại chỗ cùng ngày) — một bước máy móc bảo toàn cả cái sai, nên nó
bị diệt ở **cả hai** chỗ thay vì một.

</frozen-after-approval>

## Code Map

**Rust — điểm tiêm và chỗ nối**
- `src-tauri/src/core/segment/pipeline.rs:348-351` — nhánh `Step::CleanByRules`, hôm nay `trace.push(step); flow`. **ĐIỂM TIÊM DUY NHẤT.** 🔴 `trace.push` phải ở lại **trong** nhánh (AC6 spec 6.2). `:107-115` `PIPELINE_ORDER` · `:122-141` `validate_order` (chỉ kiểm **hoán vị đủ bảy bước** — một tiền tố bốn bước sẽ bị TỪ CHỐI, đọc trước khi thiết kế đường xem trước) · `:317-320` `run_import_with_order` · `:426-428` `run_import` — **không đổi thứ tự**.
- `pipeline.rs:189-211` `PipelineInput` (`encoding` `:203`, `chapter_pattern: Option<String>` `:207` — **khuôn cho một cấu hình per-run mới**, `source_lang` `:210`); `:216` `default_shaped`; `:258-264` `PipelineOutput` (`chapters`, `trace`) — nơi báo cáo làm sạch phải đi ra; `:272-275` `Unit`; `:287-303` `Flow`.
- `src-tauri/src/core/segment/normalize.rs:58-68` `Normalized` · `:78` `normalize` · `:165` `normalize_window` — **khuôn thân bước** (văn bản + số đếm trong chính struct kết quả). Story này lặp khuôn, không sửa tệp.
- `src-tauri/src/core/scope/kinds.rs:192-198` — hàng `ImportCleanupRule => "import_cleanup_rule" : Merge` **đã tồn tại**, doc-comment nêu đích danh FR124. Không thêm `ScopeKind` mới.
- `src-tauri/src/core/scope/mod.rs:230` `ScopeResolver` · `:265` `with_work` · `:330` `apply_merge(kind, global, work, primary)` → `Vec<Tiered<V>>`; `resolve.rs:87` `Tiered` (`tier`/`value`); `mod.rs:81` `Tier`. ⚠️ `apply_merge` **chưa có chỗ gọi sản phẩm nào** — story này là consumer đầu tiên.
- `src-tauri/src/core/glossary/store.rs:262` `load_tier` · `:326` chỗ gọi `apply_override` — **khuôn kho hai tầng**; `core/glossary/entry.rs:158` `GlossaryTier` — **khuôn kiểu nhãn tầng riêng** (không tái dùng `Tier`, nó bị cấm ngoài `core/scope/**`).
- `src-tauri/src/core/store/schema.rs:300-331` `GLOSSARY_ENTRY_DDL` — khuôn DDL, gồm rào rỗng **25 điểm mã** (`trim()` của SQLite chỉ cắt dấu cách ASCII — `src-tauri/AGENTS.md:37`); `:649` `GLOBAL_MIGRATIONS` (6 bước ⇒ **bước mới `to_version: 7`**) · `:1474` `PROJECT_MIGRATIONS` (đích 18 ⇒ **bước mới `to_version: 19`**, số 4 là số cháy) · `:1858` `validate_strictly_increasing`.
- `src-tauri/src/commands/project.rs:286` chỗ gọi `run_import` **duy nhất** · `:351-354` INSERT `source_text` · `:1032` `preview_import_encoding` (hàm thuần) · `:1086` nhánh tự khai · `:1090` dựng `ImportEncodingPreview` · `:913-949` `NormalizedPreviewWire`/`EncodingCandidateWire` · `:1170` `confirm_import_with_encoding` · `:425`/`:1356` chỗ dựng `ScopeResolver::with_work` — **chỗ store gặp resolver**.
- `src-tauri/src/core/segment/encoding.rs:272` `render_candidates` · `:305` `normalized_candidate` · `:339` `normalized_self_declared` · `:64` `EVIDENCE_WINDOW_BYTES` (private, 4096) — đường xem trước hôm nay **bỏ qua** chuỗi; đây là chỗ nợ `:9359` sống.
- `src-tauri/src/core/i18n/` `macro_rules! message_keys!` — danh mục **đóng**; một biến thể quên thêm vào `ALL` cho một test xanh giả (`src-tauri/AGENTS.md:13`).
- `src-tauri/src/core/segment/import.rs:35` — mệnh đề *"Luật làm sạch (Story 6.5) vẫn mở"*, **hết đúng** sau story, sửa tại chỗ kèm 🔵.

**Cổng — cái nào đỏ, cái nào mù**
- `src-tauri/tests/segment_pipeline_boundary.rs:147` khoá `PIPELINE_ORDER` theo thứ tự (`Step::CleanByRules` liệt ở `:153`) · `:171` đếm chỗ gọi `run_import` đúng 1 · `:132` sàn quần thể · `:219`/`:236` khuôn kiểm chứng dương. ⚠️ **Không cổng nào đỏ khi thân một bước phình ra.**
- `src-tauri/tests/segment_normalize_boundary.rs:137` `the_pipeline_module_actually_calls_the_normalize_module` · `:274` đếm chỗ gọi có số nêu rõ · `:98` `text_before_first_cfg_test_line` (neo theo **đầu dòng**, không `text.find` chuỗi trần) · `:231`/`:244` tự-kiểm phép neo — **khuôn gần nhất** cho cổng mới.
- `src-tauri/tests/scope_boundary.rs:71-76` `FORBIDDEN_OUTSIDE_SCOPE`; `scope_contract.rs:327` `a_merge_keeps_both_tiers_without_deduplicating` · `:365` tier là khoá phụ.
- `src-tauri/tests/pinned_contract.rs:84-89` assert `GLOBAL_MIGRATIONS.len()` · `:214-216` assert `PROJECT_MIGRATIONS.len()` — **cả hai đỏ** khi thêm bước, sửa kèm lý do là thêm bảng. `store_contract.rs:894`/`:1138` di trú + từ chối lược đồ mới hơn. `project_contract.rs` giữ danh sách bảng `project.db` — bảng mới phải thêm vào đó.
- `src-tauri/tests/ipc_contract.rs` khoá tên command + bốn trường `IpcError`.

**Frontend — dây và bề mặt**
- `src/config/project.ts:142-147` `NormalizedPreviewWire` · `:150-155` `EncodingCandidateWire` · `:158-165` `ImportEncodingPreview` · `:177-220` **kiểm kiểu lúc chạy** (mỗi trường mới = một vế `typeof`; thiếu ⇒ `undefined` lọt lên `.vue`) · `:173-175` hằng lệnh · `:269-276` `confirmImportWithEncoding`.
- `src/importPreviewState.ts:56` ô `preview` · `:130-135` `importPreviewSelectedCandidate` · `:152-158` `importPreviewSelectedNormalized` (khuôn "đọc ứng viên khi có, rơi về bản tự khai khi không", **0 IPC**) · `:164-166` `importPreviewEmptyReasonForTier` — nhánh `3` **phải chết** · `:245-256` `selectImportPreviewCandidate` · `:339-354` `resetImportPreview()`.
- `src/ImportPreviewOverlay.vue:255-291` tầng chuẩn hoá (**khuôn một tầng có thân**) · `:302-307` **tầng 3, chỗ thay** · `:62-69` `tierEmptyMessageKey` (`switch` cạn, TS bắt thiếu nhánh) · `:280` `.ip-normalized-text` (`white-space: pre-wrap`, cỡ `read`).
- **Khuôn đánh dấu inline — tái dụng chính:** `src/panels/glossaryMarksMap.ts:46-83` `SegmentTermSpan` (`start`/`end` là **ĐIỂM MÃ, nửa-mở `[start,end)`**, không chồng lấn — Rust đã `resolve_overlaps`); `src/panels/GridPanel.vue:397-421` `sourcePieceStartsOf`/`sourcePiecesOf` (cắt bằng `[...text]`), `:452-475` `SourcePieceInfo`, `:1634-1663` template `v-for` mảnh `<span>` **viết liền không khoảng trắng** (nếu không sẽ thêm ký tự vào `Selection`). Tiền lệ gạch ngang: `GridPanel.vue:2302` `.cell.omitted { text-decoration: line-through }`.
- Token `ornament`: `src/tokens/tokens.json:27`; dùng làm nét ở `GridPanel.vue:1986-1987`, `ReadingMode.vue:778`. Mockup tầng 3: `mockups/web-import.html:103` (`.strike`), `:332-366` (danh sách luật, nhãn tầng, hai số, ô "＋ thêm luật mới").
- `src/commands/index.ts:1064-1090` ba lệnh xem trước (`E`, `Mod+Alt+Enter`) — **chỗ đăng ký duy nhất**; `:1077-1081` ghi rõ hợp âm phải duy nhất **toàn cục**. Phím `R` trần **chưa ai chiếm**.
- `src/i18n/vi.json:206-228` khối `mode.library.preview.*`; `:226` `tier_empty_story_6_5` — **khoá chết sau story**.
- `tests/frontend/importPreviewNormalized.test.ts:29` `freshState()` · `:97` ca "đổi ứng viên, 0 IPC" · `:167` nhánh tự khai — khuôn cho tệp test mới. `importPreviewOverlayRender.test.ts:63` `freshOverlay()` (nạp **động** để cùng một thể hiện module). `importPreviewEncodingWireShape.test.ts:63` — trường wire mới phải có ca ở đây.

**Nợ liên quan** — `deferred-work.md:9359-9372` (🔴 **chủ story này**, xem trước lệch bản sản phẩm) · `:9066-9090` (tầng 3 còn rỗng, chủ 6.5) · `:9172-9195` (`selectImportPreviewCandidate` không chạy lại chuỗi; chủ *"6.9 hoặc 6.5 nếu 6.5 đi trước"* — **6.5 đi trước**) · `:9293-9302` (overlay thiếu test DOM: bẫy Tab, `aria-modal`, `:disabled`, ba nhánh lỗi; chủ 6.5) · `:2971-2980` (va hợp âm `⌘↵`, chủ mới 6.5) · `:3818-3846` (🔴 `ornament` cấm làm màu chữ) · `:9283-9302` · `:2204-2206` (hàng bảng Markdown bị cắt; chủ 6.5 **hoặc** 6.12).

## Tasks & Acceptance

**Execution:**
- [x] `_bmad-output/planning-artifacts/architecture/architecture-AuraTranslate-2026-08-02/ARCHITECTURE-SPINE.md` -- 🔴 **LÀM TRƯỚC MỌI THỨ**: cửa NFR15 ba bước cho `regex` — mở tệp giấy phép trong `~/.cargo/registry/src/**/regex-1.13.1/` mà **đọc** (không tin nhãn registry), ghi một hàng vào bảng §Stack kèm giấy phép và lý do, **rồi mới** thêm vào `Cargo.toml`. Ghim bằng `=` (`regex = "=1.13.1"`) -- lock chỉ giữ số đúng tới lần `cargo update` đầu tiên
- [x] `src-tauri/src/core/store/schema.rs` -- hằng `IMPORT_CLEANUP_RULE_DDL` mới (`id` AUTOINCREMENT · `pattern` · `kind CHECK IN ('literal','regex')` · `enabled` · `ord` · `created_at`), rào rỗng liệt **trọn 25 điểm mã** như `GLOSSARY_ENTRY_DDL:305-323`; **bước song sinh** dùng **cùng một hằng** vào `GLOBAL_MIGRATIONS` (`to_version: 7`) và `PROJECT_MIGRATIONS` (`to_version: 19`) -- một bảng chép hai lần là hai nguồn sự thật
- [x] `src-tauri/tests/pinned_contract.rs` + `src-tauri/tests/project_contract.rs` -- cập nhật hai assert độ dài bộ di trú và danh sách bảng `project.db`; mỗi lượt sửa kèm lý do là **thêm bảng**, không phải một kỳ vọng đã nới
- [x] `src-tauri/src/core/cleanup/mod.rs` -- tạo mới, **THUẦN**: `CleanupRule { id, tier, pattern, kind, enabled }`, `CleanupMatch { rule_key, start, end }` (**điểm mã, nửa-mở `[start,end)`** — cùng quy ước `SegmentTermSpan`), `apply(text, &[CleanupRule]) -> Cleaned { text, matches, per_rule_counts }`. Biên dịch regex **một lần**/lượt; mẫu hỏng trả `Err`, không `panic!`. Đếm cho **mọi** luật kể cả luật đã tắt; **xoá** chỉ luật đang bật. Chỗ khớp chồng nhau ⇒ xoá một lần, nhưng **mỗi** luật vẫn đếm -- không khử trùng lặp (AD-18 hợp nhất)
- [x] `src-tauri/src/core/cleanup/store.rs` -- nạp một tầng từ một `&Store` (khuôn `glossary/store.rs:262`), rồi `resolver.apply_merge("import_cleanup_rule", &global, work, primary)`; CRUD bốn phép (thêm · sửa · xoá · bật/tắt) qua `store::Writer` nối tiếp. Kiểu nhãn tầng **riêng** (khuôn `GlossaryTier`), **không** tái dùng `Tier` -- `scope_boundary.rs:71` cấm
- [x] `src-tauri/src/core/segment/pipeline.rs` -- thêm trường `cleanup_rules: Vec<CleanupRule>` vào `PipelineInput` (khuôn `chapter_pattern`, mặc định rỗng ở `default_shaped`); nhánh `:348-351` **GỌI** `cleanup::apply` (đừng viết lại nội tuyến), `trace.push` ở lại trong nhánh; `PipelineOutput` chở báo cáo làm sạch theo từng Chương -- xem trước phải đọc số của **chuỗi**, không của một bản dựng thứ hai
- [x] `src-tauri/src/commands/project.rs` -- 🔴 **chuyển `preview_import_encoding` sang chạy chuỗi thật** cho **mỗi** ứng viên **và** cho đường tự khai: nạp luật hai tầng, gọi `run_import*`, đọc báo cáo làm sạch từ `PipelineOutput`. Hai số đếm đo trên **toàn** văn bản; bản dựng hiển thị được phép cắt và phải mang cờ đã-cắt. `confirm_import_with_encoding` truyền **cùng** tập luật -- đây là phép đóng nợ `:9359`, và là chỗ duy nhất store gặp resolver
- [x] `src-tauri/src/commands/cleanup.rs` + `src-tauri/src/core/i18n/` -- năm command (liệt · thêm · sửa · xoá · bật/tắt) theo **khuôn hai lớp** (hàm thuần nhận `Option<&Store>` + vỏ `#[tauri::command]` trong module lồng `wire`, lấy `State` qua **`try_state`**); `message_key` mới cho *mẫu regex hỏng* và *mẫu rỗng* khai trong `message_keys!` -- một danh mục đóng, đừng viết danh sách song song
- [x] `src-tauri/tests/cleanup_boundary.rs` -- tạo mới: sàn quần thể; `pipeline.rs` **có gọi** `cleanup::` (khuôn `segment_normalize_boundary.rs:137`); `core/cleanup/**` mang **0** dòng gõ `ScopeKind`/`Semantics`/`Tier`; đếm chỗ gọi sản phẩm kèm số nêu rõ; neo `#[cfg(test)]` theo **đầu dòng** (`:98`); **kiểm chứng dương** ca dương + ca âm cho **mỗi** vị từ -- cổng phải đọc được sự LỆCH, không chỉ sự tồn tại
- [x] `src-tauri/tests/cleanup_contract.rs` + `src-tauri/tests/segment_contract.rs` -- ca cho từng hàng ma trận I/O ở **tầng lệnh/pipeline**, không ở tầng hàm thuần. 🔴 Ba ca DƯƠNG bắt buộc: (a) `run_import` với một luật bật ⇒ `source_text` đọc lại từ `project.db` **không còn** chuỗi bị khớp; (b) luật Toàn cục #1 và Tác phẩm #1 cùng tồn tại, bật/tắt **độc lập**; (c) `preview_import_encoding` và `confirm_import_with_encoding` cho **cùng** văn bản trên cùng đầu vào -- mệnh đề đóng nợ `:9359` phải có người quan sát
- [x] `src/config/project.ts` -- kiểu wire khối làm sạch (`rules` mang `tier`+`id`+hai số đếm, `spans`, `window_truncated`) trên **mỗi** `EncodingCandidateWire` **và** một trường tự khai; thêm vế kiểm kiểu lúc chạy cho **từng** trường mới; năm hằng lệnh + adapter ba trạng thái -- thiếu vế thứ hai thì `undefined` lọt thẳng lên `.vue`
- [x] `src/importPreviewState.ts` -- computed dẫn xuất khối làm sạch (khuôn `:152-158`: đọc ứng viên khi có, rơi về bản tự khai khi không); hành động bật/tắt và soạn gọi IPC rồi nạp lại xem trước; bỏ nhánh `3` của `importPreviewEmptyReasonForTier`; mọi ô mới vào `resetImportPreview()`
- [x] `src/ImportPreviewOverlay.vue` -- thay `:302-307` bằng tầng 3 có thân: văn bản mang gạch ngang tại chỗ (cắt theo mảnh **điểm mã**, khuôn `GridPanel.vue:408-421`+`:1634-1663`, `<span>` viết liền) + danh sách luật (mẫu · nhãn tầng · hai số · tick bật/tắt · sửa/xoá · ô thêm luật mới). `text-decoration-color: var(--color-ornament)`, **không** `color` -- `check:tokens` đỏ nếu ngược lại. Khoá i18n qua `switch` cạn; AD-16: dữ liệu, không markup, không `v-html`

**Vá vòng rà 1 (2026-09-05) — xem §Spec Change Log:**
- [x] `src/ImportPreviewOverlay.vue` + `src-tauri/src/commands/cleanup.rs` -- 🔴 **bề mặt soạn chỉ tạo luật tầng TOÀN CỤC**: bỏ ô chọn tầng khỏi form thêm/sửa. Luật tầng Tác phẩm VẪN được đọc và hợp nhất bình thường khi có Tác phẩm mở (AD-18 nguyên vẹn, `apply_merge` vẫn là đường phân giải duy nhất) — chỉ là chưa SOẠN được từ màn này. Lý do đo được: `commands/project.rs:2608-2615` phân giải tầng Tác phẩm từ `OpenWorkState`, tức Tác phẩm **đang mở** — mà lượt nhập Library đang TẠO một Tác phẩm chưa tồn tại, nên ô chọn ấy hoặc trượt, hoặc đính luật vào `project.db` của một Tác phẩm **khác**
- [x] `src-tauri/src/commands/project.rs` -- span khớp **vắt qua** biên cửa sổ (`m.start < visible_chars <= m.end`) phải được **cắt về** biên rồi VẪN gạch ngang, không bị `:1232` loại bỏ -- chữ đang hiện trên màn mà sẽ bị xoá thì phải mang dấu; đây đúng lời hứa cốt lõi của FR124
- [x] `src-tauri/src/core/cleanup/mod.rs` -- mẫu regex khớp **độ dài 0** (`x*`) không được đếm và không được sinh span -- một chỗ khớp xoá 0 ký tự làm số đếm phồng lên và tạo một dấu vô hình
- [x] `src-tauri/tests/cleanup_boundary.rs` -- 🔴 hai assert thật (`the_pipeline_module_actually_calls_the_cleanup_module`, `the_cleanup_apply_function_has_exactly_one_named_product_call_site`) phải **GỌI** `text_before_first_cfg_test_line`, đúng như khuôn gốc `segment_normalize_boundary.rs:194` -- hôm nay hàm ấy chỉ được hai ca tự-kiểm của chính nó gọi, nên cổng quét cả thân `#[cfg(test)]`: một lời gọi nằm trong khối test cũng làm cổng xanh
- [x] `tests/frontend/importPreviewCleanup.test.ts` -- ca cho nhánh **giữ ứng viên người dùng đã chọn** của `reloadImportPreviewAfterRuleChange`: chọn tay một ứng viên khác mặc định, rồi CRUD một luật, rồi khẳng định lựa chọn ấy còn nguyên -- gỡ nhánh đó đi thì mọi ca hiện có vẫn xanh, mà hậu quả là ghi **sai bảng mã** xuống đĩa
- [x] `src/ImportPreviewOverlay.vue` + `src/importPreviewState.ts` -- cờ "đang gửi" riêng cho bốn hành động CRUD (khuôn `GlossaryQuickAdd.vue` bọc `<fieldset>`), **không** mượn `importPreviewConfirming`; xoá một luật đi qua bước xác nhận hai nhịp (khuôn `GlossaryManageOverlay.vue` `manageDeletePending`); tick bật/tắt phải có `<label>`/`aria-label` nêu mẫu luật
- [x] `src-tauri/src/core/cleanup/mod.rs` + `src/config/project.ts` -- gỡ `CleanupReport::source_before_removal` (ghi mỗi lượt chạy, **không ai đọc** — một bản chép toàn văn bản trên mọi `ImportedChapter`) và adapter `cleanupListRules` (không chỗ gọi nào trong `src/**`), hoặc nếu giữ thì viết ra ai sẽ đọc
- [x] `src-tauri/tests/cleanup_contract.rs` -- đánh lại số hàng ma trận cho nhất quán (hiện có **hai** khối cùng nhãn `// Hàng 11`)
- [x] **ĐO, đừng khai** -- xem trước nay chạy trọn chuỗi tới **sáu** lượt trên **toàn** văn bản (năm ứng viên + đường tự khai). Chú thích hiện khẳng định *"CPU của regex/normalize rẻ"* mà không có số. Đo trên một Chương lớn thật, ghi con số vào chú thích; nếu chậm thì ghi nợ có chủ -- một mệnh đề hiệu năng không kèm phép đo là đúng thứ kho này cấm
- [x] `src/commands/index.ts` + `src/i18n/vi.json` -- lệnh bật/tắt luật khớp **khối đang chọn** hôm nay chưa có bề mặt khối (tầng 2 là Story 6.9) ⇒ **không** đăng ký phím `R` trong story này, ghi nợ có chủ thay vì một hợp âm không có đối tượng; khoá `mode.library.preview.tier3_*` mới, xoá `tier_empty_story_6_5` (`:226`) -- khoá phải là literal trong `t('…')` để `check:i18n` thấy
- [x] `tests/frontend/importPreviewCleanup.test.ts` (mới) + `importPreviewEncodingWireShape.test.ts` + `importPreviewOverlayRender.test.ts` -- tầng 3 dựng đúng span của ứng viên đang chọn; đổi ô ⇒ khối làm sạch đổi mà **0 lời gọi IPC**; đường tự khai có khối riêng; hình dạng dây mới có ca. 🔴 giữ nguyên ba ca "đổi ứng viên = 0 IPC" (`importPreviewEncoding.test.ts:192-211`) **không sửa kỳ vọng**
- [x] `_bmad-output/implementation-artifacts/deferred-work.md` -- nối dòng `→` cho `:9359` (✅ đóng), `:9066` (🟡 tầng 3 xong, tầng 2 còn mở), `:9172` (✅ xem trước nay chạy chuỗi thật); ghi nợ **MỚI có chủ** cho: phím `R` + luật khớp khối chọn (**Story 6.9**) · số *"cả lần nhập"* mới bằng số *"Chương này"* khi lần nhập có đúng một Chương (**Story 6.6/6.7**) · va hợp âm `⌘↵` `:2971` (chuyển chủ nếu story này không đăng ký) · test DOM overlay `:9293` nếu chưa đóng hết -- `check:debt-owner` đọc **dòng `→`**, một câu trong thân mục thì cổng không thấy

**Acceptance Criteria:**
- Given một lượt nhập bất kỳ, when so `source_text` mà `confirm_import_with_encoding` ghi xuống với văn bản mà `preview_import_encoding` vừa hiện, then **giống nhau từng byte** — và ca kiểm phải chạy trên đường lệnh thật, không trên hai hàm thuần đặt cạnh nhau.
- Given `PIPELINE_ORDER` và `segment_pipeline_boundary.rs`, when chạy sau story, then bảy bước và chỗ gọi `run_import` duy nhất **không đổi** — story này thêm thân, không thêm bước.
- Given cổng mới `cleanup_boundary.rs`, when **gỡ** nó ra và chạy lại bộ test **CŨ**, then bộ cũ **xanh** — chứng minh mệnh đề mới thật sự chưa ai canh.
- Given `core/cleanup/**` sau story, when `grep` đếm dòng gõ `ScopeKind`/`Semantics`/`Tier`, then **0**, và `scope_boundary.rs` xanh **mà không sửa một kỳ vọng nào**.
- Given `npm run check:tokens`, when chạy sau story, then 0 findings — `ornament` xuất hiện ở `text-decoration-color`, không ở `color`.
- Given bộ Rust và vitest đang xanh trước story, when chạy sau story, then vẫn xanh mà không nới một kỳ vọng nào — trừ hai assert độ dài bộ di trú và hình dạng dây ứng viên, nơi lượt sửa phải kèm lý do là **thêm bảng / thêm trường**.
- Given `npm run check:debt-owner`, when chạy sau story, then **0 mục mở mồ côi**, và `:9359` đọc là đóng bằng một dòng `→`.
- Given mọi chỗ còn khai bước 3 là "thân rỗng" hay tầng 3 là "chờ Story 6.5" sau story, when soát từng chỗ, then mỗi chỗ là một chú thích 🔵 có ngày — không chỗ nào còn khai ngược sự thật.

## Spec Change Log

### Vòng rà 1 — 2026-09-05 (ba lớp rà đối kháng; `bad_spec`, Ice chốt vá tại chỗ)

**Phát hiện kích hoạt.** Ô chọn tầng trong form soạn luật cho người dùng chọn *"Tác phẩm"*, nhưng
`commands/project.rs:2608-2615` phân giải tầng ấy từ `OpenWorkState` — **Tác phẩm đang mở**. Một
lượt nhập Library thì đang TẠO một Tác phẩm chưa tồn tại, nên chọn "Tác phẩm" hoặc trượt với
`cleanup.work_tier_unavailable`, hoặc **đính luật vào `project.db` của một Tác phẩm khác** đang
mở — im lặng, không cổng nào đỏ.

**Vì sao đây là lỗi của SPEC.** §Tasks viết *"chọn tầng Toàn cục hay Tác phẩm"* mà không nói
**Tác phẩm nào** ở một màn hình mà Tác phẩm đích chưa tồn tại. Bản thi hành làm đúng chữ ấy;
chữ ấy có hai cách đọc và spec không chọn cách nào. Ice chốt 2026-09-05: bề mặt soạn chỉ tạo
luật **Toàn cục**; nửa đọc/hợp nhất của tầng Tác phẩm giữ nguyên.

🔴 **Trạng thái xấu mà lượt sửa này tránh.** Một luật người dùng viết "cho lần nhập này" nằm im
trong `project.db` của một Tác phẩm không liên quan, rồi áp cho **mọi** lượt nhập sau của Tác
phẩm đó. Đúng lớp "luật ẩn xoá nhầm" mà FR124 tồn tại để chặn — và tệ hơn, do chính bề mặt
dựng ra để chặn nó đẻ ra.

**KEEP — đã nghiệm thu, phải sống sót mọi lượt dựng lại:**
1. Thân bước 3 gọi xuống module thuần; `PIPELINE_ORDER` và bảy bước không đổi;
   `run_import` vẫn đúng một chỗ gọi sản phẩm (qua `run_pipeline`).
2. `ScopeResolver::apply_merge("import_cleanup_rule", …)` là đường phân giải **duy nhất** —
   đột biến `Merge` → `Override` làm nhiều ca đỏ, đã chạy thật 2026-09-05.
3. Bảng luật một hằng DDL dùng chung hai tầng, di trú song sinh `to_version` 7 và 19, rào rỗng
   trọn 25 điểm mã.
4. Xem trước chạy **chính** chuỗi (`run_pipeline`), đóng nợ `deferred-work.md:9359`; ca
   `preview_and_confirm_agree_byte_for_byte_on_the_same_input_and_the_same_rules` là phép đo của nó.
5. Số đếm đo trên **toàn** văn bản, bản dựng hiển thị được cắt — đã có ca
   `counts_cover_the_whole_chapter_even_when_the_rendered_window_is_truncated`, và ca ấy đã được
   chứng minh **đỏ** trước khi mã được sửa.
6. Cổng `cleanup_boundary.rs` gỡ ra thì bộ test CŨ vẫn xanh (đo: 1120 xanh / 0 đỏ).

**Đã sửa trong lượt này:** §Tasks thêm chín mục vá (bề mặt soạn Toàn cục-only · span vắt biên
cửa sổ · regex khớp rỗng · cổng quét thân test · ca giữ bảng mã đã chọn · cờ đang-gửi + xác nhận
xoá + nhãn trợ năng · gỡ dữ liệu chết · đánh lại số hàng · ĐO mệnh đề hiệu năng).

## Design Notes

**Vì sao kho luật KHÔNG sống trong `core/segment/`.** Bước 3 khác bước 4 ở đúng một điểm đo được: `normalize` là hàm thuần của văn bản, còn làm sạch phụ thuộc **trạng thái ngoài chuỗi đã giải mã** — chính câu mà §Design Notes của Story 6.4 đã viết để từ chối suy rộng lý lẽ "dựng sẵn năm bản" sang tầng 3. `core/segment/` phải ở lại thuần (`segment_boundary.rs:397` `the_splitter_stays_pure`), nên bảng luật + hai tầng + CRUD sống ở `core/cleanup/`, đúng khuôn `core/glossary/`. `pipeline.rs` **gọi xuống**, và `PipelineInput` chở luật **đã phân giải** vào — nên `core/segment/` vẫn không biết gì về `Store`. Chuỗi AD-39 không vì thế mà có bản cài đặt thứ hai: thứ tự bảy bước vẫn nằm đúng một chỗ.

**Vì sao xem trước phải chạy chuỗi, không dựng lại.** Nợ `:9359` cho hai lối thoát: (a) chạy lại trọn chuỗi, hoặc (b) đo và ghi rõ vì sao hai đường trùng. Lối (b) **chết** ở story này — bước 3 có thân, và thân ấy phụ thuộc bảng luật, nên hai đường phân kỳ theo định nghĩa. Còn lối (a) không chỉ là cách rẻ hơn: dựng một bản thứ hai trong `encoding.rs` sẽ **chép thứ tự bước** sang một chỗ thứ hai, đúng thứ AD-39 (`spine:498`) cấm bằng chữ. ⚠️ `validate_order` (`pipeline.rs:122-141`) hôm nay chỉ nhận **hoán vị đủ bảy bước**, nên "chạy một tiền tố bốn bước" không phải một tham số có sẵn — đọc nó trước khi chọn hình dạng, và nếu phải mở một lối vào mới thì lối ấy vẫn phải đi qua `PIPELINE_ORDER`, không qua một mảng viết tay.

**Cửa sổ: siết chặt luật 6.4, không nới.** Story 6.4 cấm một số đếm **đo trên cửa sổ** mà **khai** là số của cả Chương. Ở đây hai số đếm đo trên **toàn** văn bản — chúng nói đúng phạm vi chúng đo, nên hợp lệ; thứ được phép cắt là **bản dựng hiển thị**, và cắt thì màn hình nói ra. Đây là hai mệnh đề khác nhau, đừng đọc gộp: cấm là *"khai sai phạm vi"*, không phải *"cắt"*.

**Danh tính luật là cặp, không phải số.** `id INTEGER PRIMARY KEY AUTOINCREMENT` chạy độc lập ở hai tệp `.db`, nên `id = 1` tồn tại ở **cả hai** tầng ngay khi người dùng soạn luật đầu tiên ở mỗi tầng. Một `rule_id` trần trên dây làm hai luật đội lốt một: bật/tắt luật Tác phẩm sẽ tắt nhầm luật Toàn cục, và không cổng nào đỏ vì cả hai đều là số nguyên hợp lệ. ⇒ Khoá trên dây và trong `CleanupMatch` là cặp `(tier, id)`.

**Gạch ngang render ở tầng 3, không ở tầng 2 — và vì sao đó không phải lệch UX.** Mockup đặt `.strike` trong thân **khối** của tầng 2 (`web-import.html:103` dùng ở khối bóc nội dung), vì bản thiết kế giả định tầng 2 đã có. Hôm nay tầng 2 rỗng (chủ Story 6.9), nên nơi duy nhất còn lại để *"hiện thứ sắp xoá"* là tầng 3 — cùng khuôn tầng chuẩn hoá của 6.4, vốn cũng tự render văn bản của nó (`ImportPreviewOverlay.vue:280`). ⇒ Tầng 3 chở **hai nửa**: văn bản đã đánh dấu và danh sách luật. Khi 6.9 dựng bề mặt khối, phần đánh dấu **chuyển** sang đó và tầng 3 co lại còn danh sách — đó là một lượt dời có chủ, không phải một lượt vá; ghi nợ để lượt ấy không phải suy lại từ đầu.

**Cái story này KHÔNG cứu được, ghi ra thay vì làm nhẹ đi.** ⚠️ Phím `R` *"bật/tắt luật đang khớp khối đang chọn"* (AC cuối của `epics.md:4688`) đòi một **bề mặt khối** mà hôm nay chưa tồn tại — tầng 2 vẫn rỗng, chủ Story 6.9 (`importPreviewState.ts:164` trả `'story_6_9'`). Đăng ký một hợp âm không có đối tượng là dựng một đường chết và chiếm trước một phím trong một registry mà hợp âm phải duy nhất **toàn cục**. ⇒ Ghi nợ có chủ **Story 6.9**, không sửa `epics.md` — năng lực chưa dựng không phải lệch spec. ⚠️ Cùng loại: số *"trong cả lần nhập"* hôm nay bằng số *"trong Chương này"*, vì một lượt xem trước hiện đúng một Chương (`PipelineShape::Blob`); nó chỉ khác đi khi 6.6/6.7 dựng lần nhập nhiều Chương. Con số **không sai** — nó đúng với một lần nhập một Chương — nhưng phải ghi nợ để không ai đọc một lượt xanh thành "đã nghiệm thu ở quy mô thật".

## Verification

**Commands:**
- `npm run build && cargo test --locked --manifest-path src-tauri/Cargo.toml` -- 🔴 **thứ tự bắt buộc**: thiếu `dist/` thì `cargo test` gãy ở khâu biên dịch chứ không ở một assert (`AGENTS.md:32`). Kỳ vọng: ≥ số nền của Story 6.4, 0 failed
- `cargo test --locked --manifest-path src-tauri/Cargo.toml --test cleanup_boundary --test cleanup_contract --test segment_pipeline_boundary --test scope_boundary --test scope_contract --test store_contract --test pinned_contract --test project_contract --test ipc_contract` -- 0 failed; `segment_pipeline_boundary` giữ nguyên số ca xanh
- `npm run test` -- vitest 0 failed; ba ca "đổi ứng viên = 0 IPC" xanh **không sửa kỳ vọng**
- `npm run check:i18n && npm run check:panel-refs && npm run check:commands && npm run check:lint && npm run check:tokens && npm run check:deps` -- 0 findings
- `npm run check:debt-owner` -- **0 mục mở mồ côi**
- `npm run check:gates` -- 0 findings (đỏ nếu một cổng mới quên một trong ba danh sách)

**Manual checks (if no CLI):**
- Gỡ hẳn `cleanup_boundary.rs` rồi chạy lại **bộ test CŨ** — phải **xanh**. Một lượt đỏ nghĩa là cổng mới đang canh thứ đã có chủ.
- Đột biến một dòng: đổi `apply_merge` thành `apply_override` ở `core/cleanup/store.rs` — ít nhất một ca phải đỏ. Xanh nghĩa là ngữ nghĩa hợp nhất chưa ai canh.
- Đột biến một dòng: bỏ vế `tier` khỏi khoá luật trên dây — ca "trùng id giữa hai tầng" phải đỏ.
- Mở tệp giấy phép `regex` trong `~/.cargo/registry/src/**/regex-1.13.1/` và đối chiếu với hàng vừa ghi vào bảng §Stack — nhãn registry **không** phải bằng chứng.

## Suggested Review Order

**Điểm tiêm — đọc trước nhất**

- Thân bước 3 gọi xuống module thuần; `trace.push` ở lại trong nhánh.
  [`pipeline.rs:393`](../../src-tauri/src/core/segment/pipeline.rs#L393)

- Luật đã tắt vẫn ĐẾM, chỉ không XOÁ; chỗ khớp chồng nhau xoá một lần.
  [`cleanup/mod.rs:275`](../../src-tauri/src/core/cleanup/mod.rs#L275)

**Hai tầng — consumer sản phẩm đầu tiên của `apply_merge`**

- Hợp nhất, không ghi đè; đột biến sang `apply_override` làm nhiều ca đỏ.
  [`cleanup/store.rs:109`](../../src-tauri/src/core/cleanup/store.rs#L109)

- Một hằng DDL, hai bước di trú song sinh (7 và 19), rào rỗng 25 điểm mã.
  [`schema.rs:852`](../../src-tauri/src/core/store/schema.rs#L852)

**Xem trước chạy CHÍNH chuỗi — chỗ đóng nợ `:9359`**

- Chạy `run_pipeline` trên TOÀN văn bản; không dựng lại thứ tự bước lần hai.
  [`project.rs:1191`](../../src-tauri/src/commands/project.rs#L1191)

- Span vắt qua biên cửa sổ được CẮT về biên, không bị loại bỏ.
  [`project.rs:1258`](../../src-tauri/src/commands/project.rs#L1258)

**Chỗ hở đã vá ở vòng rà — đọc kỹ nhất**

- Tầng Tác phẩm phân giải từ Tác phẩm ĐANG MỞ; vì thế bề mặt soạn chỉ tạo luật Toàn cục.
  [`project.rs:2630`](../../src-tauri/src/commands/project.rs#L2630)

- Tải lại sau CRUD phải GIỮ ứng viên người dùng đã chọn, nếu không ghi sai bảng mã.
  [`importPreviewState.ts:319`](../../src/importPreviewState.ts#L319)

**Bề mặt — dữ liệu, không markup**

- Gạch ngang là màu NÉT (`text-decoration-color`), không bao giờ màu chữ.
  [`ImportPreviewOverlay.vue:492`](../../src/ImportPreviewOverlay.vue#L492)

- Tick bật/tắt mang `aria-label` nêu đích danh mẫu luật.
  [`ImportPreviewOverlay.vue:556`](../../src/ImportPreviewOverlay.vue#L556)

**Cổng — và chỗ mù đã vá**

- Cổng thân-bước nay lọc `#[cfg(test)]` trước khi quét; trước đó thân test làm nó xanh giả.
  [`cleanup_boundary.rs:133`](../../src-tauri/tests/cleanup_boundary.rs#L133)

- Số đếm của TOÀN Chương dù bản dựng bị cắt — ca này đã chứng minh ĐỎ trước khi mã được sửa.
  [`cleanup_contract.rs:656`](../../src-tauri/tests/cleanup_contract.rs#L656)

- Chỗ khớp vắt biên bị cắt chứ không mất dấu — đối chứng đỏ đã chạy.
  [`cleanup_contract.rs:709`](../../src-tauri/tests/cleanup_contract.rs#L709)

- Mệnh đề hiệu năng có SỐ, không có lời khai: sáu lượt chuỗi trên Chương 440 KB.
  [`cleanup_contract.rs:809`](../../src-tauri/tests/cleanup_contract.rs#L809)
