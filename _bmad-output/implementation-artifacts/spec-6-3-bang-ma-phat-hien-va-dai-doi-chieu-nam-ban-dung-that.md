---
title: 'Story 6.3 — Bảng mã: phát hiện và dải đối chiếu năm bản dựng thật'
type: 'feature'
created: '2026-09-04'
status: 'done'
review_loop_iteration: 2
baseline_commit: '62ddf8abd5f8af7552f1523bf32ebdbe42959f59'
context:
  - '{project-root}/AGENTS.md'
  - '{project-root}/src/AGENTS.md'
  - '{project-root}/src-tauri/AGENTS.md'
  - '{project-root}/_bmad-output/implementation-artifacts/epic-6-context.md'
---

<frozen-after-approval reason="human-owned intent — do not modify unless human renegotiates">

## Intent

**Problem:** FR126 đòi tự phát hiện bảng mã trong năm bảng và cho người dùng sửa **bằng mắt** trong một giây. Hôm nay đường sản phẩm khai cứng UTF-8 (`project.rs:277` → `PipelineInput::default_shaped`), nên một tệp GBK hoặc bị từ chối, hoặc — đúng ca AD-39 `:470` — ra một Chương mà không lỗi nào ném. Không có bề mặt nào để người dùng nhìn thấy hay sửa. Kèm theo là một món nợ đã ghi tên story này làm chủ: `ImportError::NotUtf8` trở thành **nhãn sai** đúng vào ngày GBK khai được.

**Approach:** Dựng bộ phát hiện ở `core/segment/encoding.rs` — ngửi BOM trước, rồi `chardetng` đoán, rồi ánh xạ vào **năm nhãn FR126 khai thành dữ liệu**; và dựng **dải năm bản dựng thật** bằng cách giải mã cùng một đoạn đầu bằng cả năm bảng. Ba trạng thái tin cậy là một **luật của ta**, không của thư viện — `chardetng` 1.0.0 không trả điểm tin cậy nào (§Design Notes). Đưa kết quả lên một màn xem trước ba tầng mới, chỉ **tầng 1 có thân**; tầng 2 và 3 có mặt, rỗng, mỗi tầng nói ra **vì sao nó rỗng** và ai là chủ.

## Boundaries & Constraints

**Always:**
- **Không có trạng thái lỗi cho bảng mã đoán sai.** Đọc sai ra chữ không đọc được, và đó là thứ **mắt phân xử** (`EXPERIENCE.md:140`). Đường lỗi duy nhất còn lại là byte không giải mã được với bảng mã ĐÃ CHỌN.
- **Không byte nào xuống đĩa trước khi người dùng xác nhận.** Đổi bảng mã chạy lại chuỗi **từ bước một, trong bộ nhớ**, trước khi có segment nào tồn tại (AD-39 spine `:502`). Giữ nguyên bất biến của `libraryImport.ts:276-330`.
- **Năm nhãn FR126 là DỮ LIỆU, một bảng, một chỗ** — `UTF-8 · GB18030 · GBK · Big5 · UTF-16`. Một cổng đọc được **sự lệch** của bảng đó, không chỉ đọc được sự tồn tại của nó; khuôn là `segment_pipeline_boundary.rs:147`.
- **Mẫu chữ trong dải đặt ở cỡ `read`** (`--face-read-md`/`--font-read-md`/`--leading-read-md`), không cỡ giao diện. Viết thẳng `font-size` trong component là đỏ ở `check:tokens` Kiểm B (`check-tokens.mjs:1032`).
- **Đổi `NotUtf8` là đổi CẢ HAI NỬA trong cùng một lượt** — biến thể Rust và `MessageKey`/`vi.json`. Đây là món nợ đã ghi chủ Story 6.3; đổi một nửa là nhét miễn trừ chứ không sửa nguồn.
- Chuỗi literal trong `src-tauri/src/**` viết **không dấu**; chú thích tiếng Việt có dấu và chở LÝ DO kèm một phép đo (`tệp:dòng`, số, ngày). Mệnh đề hết đúng thì **sửa tại chỗ kèm 🔵 và ngày**, không xoá.
- Mọi phím đăng ký trong `CommandRegistry`; `@click` là **đúng một** `dispatch('<id>')`.

**Ask First:**
- Bất kỳ crate mới nào. `chardetng =1.0.0` và `encoding_rs =0.8.35` đã ghim và đã rà (`Cargo.toml:91,99`) — dùng lại thêm **0 byte**, không mở cửa NFR15. Một crate thứ ba thì có.
- Đổi **luật ba trạng thái tin cậy** khỏi hình dạng ghi ở §Design Notes — nó quyết định khi nào dải mở ra, tức quyết định người dùng bị hỏi bao nhiêu lần.
- Nếu một ca đang xanh **không thể** giữ nguyên kỳ vọng: dừng và trình số, đừng sửa kỳ vọng cho khớp mã.

**Never:**
- **Không tự sinh fixture GBK/Big5 để ĐO ĐỘ CHÍNH XÁC của `chardetng`** — mã hoá bằng `encoding_rs` rồi bảo chính bộ dò đọc lại là một vòng tròn (`webimport_probe.rs:250-252`). Bàn đo `:260` giữ nguyên `#[ignore]` và giữ nguyên nhánh đỏ 0-mẫu; con số tỉ lệ ở lại sổ nợ, **chủ Ice**. Byte dựng tay vẫn dùng được để kiểm **luật của ta** và **đường dây** — tiền lệ `segment_contract.rs:7976`.
- Không dựng tầng 2 (bóc nội dung, Story 6.9) và tầng 3 (luật làm sạch, Story 6.5): hai tầng có mặt, **rỗng, có tên chủ**.
- Không dựng mẫu phân tách cấu hình được (6.6), không chạm mạng (6.7/6.8), không đọc `.docx` (6.12).
- Không thêm bước nào vào `PIPELINE_ORDER`, không đổi thứ tự bảy bước, không thêm chỗ gọi sản phẩm thứ hai của `run_import`.
- Không `v-html` cho bất kỳ mẫu chữ nào trong dải (AD-16).

## I/O & Edge-Case Matrix

| Scenario | Input / State | Expected Output / Behavior | Error Handling |
|---|---|---|---|
| Tệp có BOM | `EF BB BF` · `FF FE` · `FE FF` | Trạng thái **nguồn tự khai**, bảng mã theo BOM, **dải KHÔNG mở** | N/A |
| Văn bản dán tay | `ChapterInput::AlreadyText` | **nguồn tự khai** — đã là `String` từ lúc rời webview, không byte nào để dò | N/A |
| Tệp thuần ASCII | Không BOM, mọi byte < 0x80 | **tự đoán, tin cậy cao** — năm bản dựng cho **cùng một chuỗi**, không có gì để chọn | N/A |
| Tệp GBK chữ Hán | Không BOM, byte nhiều byte | **tự đoán, tin cậy thấp** — ≥2 bản dựng khác nhau ⇒ **dải mở**, năm ô, GBK và GB18030 hiện chữ y hệt | N/A |
| `chardetng` đoán ngoài năm bảng | Đoán `Shift_JIS`/`EUC-KR`/`windows-1252` | **tự đoán, tin cậy thấp**, ô đang chọn rơi về ứng viên giải mã được đầu tiên theo thứ tự FR126 | N/A |
| Một ứng viên không giải mã được | Byte không hợp lệ với bảng đó | Ô ấy hiện **"không ra chữ"**, vẫn chọn được; các ô khác không ảnh hưởng | N/A |
| Người dùng đổi bảng mã | Chọn một ô khác | Chuỗi chạy lại **từ bước một trong bộ nhớ**; ba tầng dựng lại; **không** phải nhập lại từ đầu | N/A |
| Xác nhận với bảng mã đã chọn | Byte không giải mã được với chính bảng ấy | Từ chối tường minh, nêu **đích danh bảng mã đã chọn** | `ImportError::UndecodableBytes { path, encoding }` |
| Tầng 2 và tầng 3 | Chưa có chủ thi hành | Có mặt, rỗng, và **nói ra vì sao rỗng** kèm tên story chủ | Không nuốt im lặng |

</frozen-after-approval>

## Code Map

**Tầng Rust — chuỗi nhập**
- `src-tauri/src/core/segment/pipeline.rs:185-200` `PipelineInput { shape, encoding: &'static encoding_rs::Encoding, chapter_pattern, source_lang }`; `:205-212` `default_shaped` khai cứng `UTF_8` — **điểm tiêm**. `:187-191` chú thích *"KHÔNG dò (Story 6.3 mới dò bằng `chardetng`)"* — **hết đúng**, sửa tại chỗ kèm 🔵.
- `pipeline.rs:404-418` `decode_unit(unit, encoding)` — dùng `decode_without_bom_handling_and_without_replacement`, KHÔNG bản `_lossy` (`:398-403`, Quyết định #6 của Story 1.15). `:437-442` `strip_bom` chỉ cắt `U+FEFF` ở đầu. `:310-318` nhánh `Step::DecodeEncoding`.
- `pipeline.rs:148-164` `ChapterInput::RawBytes { bytes, label } | AlreadyText(String)` — byte thô sẵn có ở đây, đủ để dò và dựng năm bản. `:104-112` `PIPELINE_ORDER` — **không đụng**.
- `src-tauri/src/core/segment/import.rs:70-122` `ImportError` sáu biến thể; `:83` `NotUtf8 { path }` — **đổi tên**. `:124-146` `impl Display` không dấu. `:152-216` `From<ImportError> for IpcError` đi qua `IpcError::new` (`:150-151`). `:259-291` `import_file` đọc byte thô, chưa giải mã. `:29-35` khối *"cố ý để trống"* — vế bảng mã **hết đúng**, sửa kèm 🔵.
- `src-tauri/src/core/i18n/mod.rs:147` `ImportNotUtf8 => "err.import.not_utf8" ["path"]` — danh mục **ĐÓNG** khai bằng `message_keys!`, đồng bộ `vi.json` bằng test chạy trên `ALL` (`src-tauri/AGENTS.md:13`). `:586-591` `IpcError::new` là đường dựng duy nhất.
- `src-tauri/src/commands/project.rs:236-242` `create_work(...)`; `:269-277` chỗ gọi `run_import` — **điểm tiêm duy nhất**; `:304-349` closure ghi CHỈ SQL; `:419-427` · `:839-848` hai vỏ thuần; `:2016-2077` hai vỏ dây trong `pub mod wire` (*"Không một quy tắc nào sống ở đây"*, `:1883`).
- `src-tauri/src/lib.rs:634` `generate_handler!` — chỗ đăng ký IPC duy nhất; `:626-633` khai cấm: **không** thêm mục ACL vào `capabilities/main.json`.
- `src-tauri/src/core/webimport/mod.rs:16-17` — 0 dòng mã, *"`Extractor`/`Fetcher` thật là Story 6.2/6.3/6.9"*. `Cargo.toml:89-90` khai chủ `chardetng` là `core::webimport` — **hết đúng** khi bộ dò về `core/segment/`, sửa kèm 🔵 (khuôn: Story 6.2 đã làm đúng thế cho `encoding_rs`, `Cargo.toml:94-99`).

**Bằng chứng chỉ-đọc — đo từ nguồn crate đã ghim**
- `chardetng-1.0.0/src/lib.rs` — API công khai của `EncodingDetector` chỉ có `new(Iso2022JpDetection)` · `feed(&[u8], last) -> bool` · `guess(tld, Utf8Detection) -> &'static Encoding` (`:2938,3002,3204`). **Không `guess_assess`, không điểm tin cậy.** `feed` trả bool nghĩa hẹp: *"the stream has contained at least one non-ASCII byte"* (`:2929-2931`).
- `chardetng-1.0.0/src/lib.rs` — `grep "utf_16|Utf16|UTF-16"` cho **0 dòng**; ứng viên là UTF-8 · Shift_JIS · EUC-JP · EUC-KR · GBK · Big5 · ISO-2022-JP cộng họ một-byte (`:2517-2635`). ⇒ **UTF-16 chỉ vào được bằng BOM; GB18030 không bao giờ được trả về, chỉ `GBK`.**
- `encoding_rs-0.8.35/src/lib.rs:946` — *"The decoder for this encoding is the same as the decoder for gb18030."* ⇒ ô GBK và ô GB18030 **luôn hiện chữ y hệt**; khác biệt chỉ ở chiều mã hoá, mà đường nhập không dùng.

**Cổng và test**
- `src-tauri/tests/segment_pipeline_boundary.rs:132` sàn quần thể · `:147` khuôn `assert_eq!` trên mảng thứ tự · `:171` cổng chỗ gọi duy nhất · `:219,236` **kiểm chứng dương** chạy vị từ THẬT trên vi phạm gieo tay, kèm ca ÂM · `:97-102` `code_lines` lọc chú thích · `:42` `SRC_RS_FLOOR = 50` (80,6 % số thật).
- `src-tauri/tests/segment_boundary.rs:324` cổng **đếm chính xác** (`assert_eq!(len, 2)`), không kiểm thành viên — `:345-351` ghi lý do: kiểm thành viên thuần xanh y hệt dù số chỗ khớp mọc từ 2 lên 10.
- `src-tauri/tests/segment_contract.rs:6-12` bốn luật thừa kế (một temp dir mỗi ca; thả `Store` trước khi xoá; không `sleep` dài; không treo khi trượt); `:46-57` `temp_dir`; `:7976` `ad_39_gbk_fixture_bytes()` — **tiền lệ byte GBK dựng tay trong test**; `:7985,8032` cặp đối chứng AD-39.
- `src-tauri/tests/webimport_probe.rs:260` bàn đo bảng mã, `#[ignore]`, `:273-279` assert 0-mẫu là **lỗi hạ tầng**, không phải "tỉ lệ 0 %"; `:69-73` `ban_do_dir()` → `6-1-ban-do/`; `:341-346` `normalize_label`. Thư mục `fixtures/encoding/` **chưa tồn tại**.
- `src-tauri/tests/ipc_contract.rs:771-790` — canh đăng ký bằng cách đọc `lib.rs` như VĂN BẢN; không có cổng tổng quát ⇒ command mới **phải tự thêm assert**.
- `scripts/check-debt-owner.mjs:164-193` — mục nợ mở phải mang `Chủ:` dương; `:221` dòng `→ ✅ / 🟡 / KHÔNG LÀM <ngày> (` quyết trạng thái; `:514` sàn 490 mục.

**Frontend**
- `src/modes/libraryImport.ts:116-168` `beginSubmit()` — nghẽn duy nhất (`flushEditorNow` trượt ⇒ **DỪNG**); `:170-244` `finishSubmit` reset năm panel; `:247-274` hai hàm nộp; `:276-330` `wireDragDropOnce` chỉ ĐIỀN `filePath`; `:93-98` `noticeKey` giữ **KHOÁ**, không giữ câu.
- `src/modes/LibraryMode.vue:1196-1252` form nhập; `:1264-1269` ba node `role="status"` luôn có mặt. 🔴 **Không lớp phủ nào mở từ tệp này** — cả bảy lớp phủ dựng ở `src/App.vue:347-365`, mỗi cái tự quản `v-if` bằng một ref của module state riêng.
- `src/glossaryImportState.ts:35` khuôn bốn trạng thái · `:52` vé `sequence` chống lượt cũ ghi đè lượt mới · `:54-74` export qua `readonly()` · `:107-119` `importEmptyReasonFor` — **"rỗng phải nói vì sao nó rỗng"**, khuôn cho tầng 2/3 · `:256-272` `resetGlossaryImport()` bắt buộc bởi `check:panel-refs`.
- `src/GlossaryImportOverlay.vue:43-102` khuôn lớp phủ: `useTemplateRef` + `focusReturnTargetOnOpen` + `trapTab` tự viết + `role="dialog" aria-modal="true"`.
- `src/tokens/tokens.json:348-372` — `read-md` **17.5px/1.8**; `:322-326` họ `read`. ⚠️ `SegmentHistoryOverlay.vue:409,465` trỏ `--font-read-body`, **token không tồn tại** — đừng chép.
- `src/commands/index.ts:2789` `installCommands(deps)` chỗ đăng ký **duy nhất**, gọi ở `main.ts` không ở `App.vue`; `:162+` `CommandDeps` tiêm state; `:66-70` `FOCUS_OWNERS` đối chiếu **hai chiều**. `registry.ts:125` văn phạm id; `labelKey = 'command.' + id`.
- `src/i18n/README.md:50` — 🔴 *"Đừng dựng sẵn một từ vựng khoá cho tính năng chưa tồn tại"*. `vi.json:10` `err.import.not_utf8`; `:186-199` miền `mode.library.*`. Không khoá `encoding`/`charset` nào tồn tại hôm nay.
- `tests/frontend/glossaryImportPreview.test.ts:14-31` khuôn: `vi.mock` biên IPC, `freshState()` = `vi.resetModules()` + `import()`; ⚠️ `freshState()` TRƯỚC, `mockResolvedValue` SAU. Cây test ở `tests/frontend/**`, **ngoài `src/`** có chủ ý.

**Sổ nợ kế thừa**
- `_bmad-output/implementation-artifacts/deferred-work.md` (cuối tệp, khối 6.2) — mục `ImportError::NotUtf8` **Chủ: Story 6.3**, đòi đổi **cả hai nửa trong CÙNG một lượt**. Ba mục kề: `PipelineInput.encoding` rò kiểu (Chủ 6.7) · `Step::Preview` rỗng (Chủ 6.5) · `.docx` nghiệm thu bằng hình dạng (Chủ 6.12).

## Tasks & Acceptance

**Execution:**
- [x] `src-tauri/src/core/segment/encoding.rs` -- tạo mới: bảng **năm nhãn FR126 khai thành dữ liệu**; `sniff_bom`; `detect(bytes)` → `EncodingVerdict { encoding, confidence }` ba trạng thái; `render_candidates(bytes)` → năm bản dựng đoạn đầu kèm cờ giải mã được -- thứ tự BOM-trước-rồi-đoán là bắt buộc vì `chardetng` không bao giờ trả UTF-16
- [x] `src-tauri/src/core/segment/mod.rs` -- khai `encoding` cạnh các module con đang có
- [x] `src-tauri/src/core/segment/pipeline.rs` -- constructor thứ hai mang bảng mã đã chọn, cạnh `default_shaped`; 🔵 sửa chú thích `:187-191` -- `PIPELINE_ORDER` và thứ tự bảy bước **không đụng**
- [x] `src-tauri/src/core/segment/import.rs` -- đổi `NotUtf8 { path }` → `UndecodableBytes { path, encoding }`, sửa `Display` (không dấu) và nhánh `From<ImportError>`; 🔵 sửa khối `:29-35` -- nợ kế thừa, nửa Rust
- [x] `src-tauri/src/core/i18n/mod.rs` -- `message_keys!`: thay `ImportNotUtf8 ["path"]` bằng `ImportUndecodableBytes ["path","encoding"]` -- nợ kế thừa, nửa khoá; hai nửa CÙNG một lượt
- [x] `src-tauri/src/commands/project.rs` -- hàm thuần `preview_import_encoding`; `create_work` nhận bảng mã đã chọn **dưới một định danh KHÔNG MẤT MÁT** (§Design Notes); nhãn không nhận ra ⇒ `IpcError` tường minh, KHÔNG rơi về UTF-8; **byte của nguồn đọc ĐÚNG MỘT lần** rồi dùng cho cả lượt xem trước lẫn lượt xác nhận; ba vỏ dây trong `pub mod wire` -- điểm tiêm là `:269-277`, không mở chỗ gọi `run_import` thứ hai. Nếu mang byte qua hai lượt IPC là bất khả ở story này thì đó là một mục nợ CÓ CHỦ cộng một chú thích nói ĐÚNG sự thật (đọc hai lần, có cửa sổ TOCTOU) — không phải một chú thích khai ngược lại
- [x] `src-tauri/src/lib.rs` -- thêm vỏ mới vào `generate_handler!` -- **không** thêm mục ACL nào
- [x] `src-tauri/tests/ipc_contract.rs` -- thêm assert đăng ký cho tên vỏ mới -- không có cổng tổng quát, quên dòng này là quên lặng lẽ
- [x] `src-tauri/tests/segment_encoding_boundary.rs` -- tạo mới: bảng năm nhãn khớp FR126 **theo thứ tự** (`assert_eq!` trên mảng, khuôn `:147`); `chardetng` được nêu tên ở **đúng một** tệp sản phẩm (đếm chính xác, khuôn `segment_boundary.rs:324`); sàn quần thể + **kiểm chứng dương** gieo vi phạm tay -- cổng phải đọc được sự LỆCH, không chỉ sự tồn tại
- [x] `src-tauri/tests/segment_contract.rs` -- ca hành vi cho từng hàng ma trận I/O: BOM ⇒ tự khai; ASCII ⇒ tin cậy cao; byte GBK dựng tay ⇒ tin cậy thấp và dải năm ô; GBK≡GB18030 cùng chuỗi; đoán ngoài năm bảng ⇒ tin cậy thấp; đổi bảng mã ⇒ chuỗi chạy lại, 0 byte xuống đĩa. 🔴 **Bốn ca DƯƠNG bắt buộc — thiếu ca nào thì mệnh đề tương ứng chưa ai canh** (vòng rà 1): (a) nhập với `GBK` đã chọn ⇒ `source_text` đọc lại ra ĐÚNG chữ Hán nguồn; (b) nhập một tệp **UTF-16BE có BOM** trọn đường ⇒ văn bản lưu bằng văn bản nguồn; (c) tệp GBK mở đầu 12 ký tự ASCII ⇒ **tin cậy THẤP**; (d) huỷ rồi xác nhận ⇒ **0 Tác phẩm** được tạo. 🔴 Một ca khẳng định trên một thư mục mà hàm dưới thử KHÔNG có đường ghi tới là xanh-do-cấu-tạo, không phải bằng chứng -- tên hàm là câu khẳng định; byte dựng tay kiểm LUẬT CỦA TA, không đo `chardetng`
- [x] `src-tauri/Cargo.toml` -- 🔵 chủ `chardetng` đổi `core::webimport` → `core::segment`, kèm ngày và lý do -- mệnh đề hết đúng thì sửa, không để lặng lẽ sai
- [x] `src/config/project.ts` -- adapter cho lệnh mới và tham số bảng mã -- adapter KHÔNG BAO GIỜ ném, trả `{ giá trị | null, error }`
- [x] `src/importPreviewState.ts` -- tạo mới: state ba tầng, vé `sequence`, `resetImportPreview()`, và một hàm **"vì sao tầng này rỗng"** nêu tên story chủ. 🔴 **Chọn một ứng viên khác phải CHẠY LẠI chuỗi từ bước một với bảng mã ấy** rồi dựng lại thứ ba tầng đang hiện — gán một trường state là CHƯA thi hành mệnh đề §Always. 🔴 **Huỷ phải xoá nguồn đang chờ**, để `import.preview.confirm` sau một lượt huỷ không còn gì để ghi -- khuôn `glossaryImportState.ts:52,107-119,256-272`
- [x] `src/ImportPreviewOverlay.vue` -- tạo mới: ba tầng theo thứ tự nhân quả, **không nút "Tiếp theo"**; tầng 1 chip trạng thái + dải năm ô ở cỡ `read`; bẫy Tab + focus-return -- khuôn `GlossaryImportOverlay.vue:43-112`; cỡ chữ chỉ qua token
- [x] `src/App.vue` -- treo lớp phủ cạnh bảy cái đang có -- `LibraryMode.vue` không mở lớp phủ, đó là luật của kho
- [x] `src/commands/index.ts` + `src/main.ts` -- đăng ký lệnh mở bộ chọn bảng mã (`E`), chọn ứng viên, xác nhận, huỷ; tiêm qua `CommandDeps`, nối ở `main.ts` -- `EXPERIENCE.md:182`; đăng ký ở `main.ts` không ở `App.vue`
- [x] `src/modes/libraryImport.ts` + `src/modes/LibraryMode.vue` -- nộp tệp đi qua xem trước trước khi tạo Tác phẩm; neo `data-*` cho focus-return -- giữ nguyên bất biến "không byte nào xuống đĩa trước xác nhận"
- [x] `src/i18n/vi.json` -- khoá `mode.library.preview.*`, `command.*` cho lệnh mới, và khoá lỗi mới. 🔵 **SỬA (vòng rà 1)**: KHÔNG gỡ `err.import.not_utf8` — chỉ thị gốc dựng trên tiền đề SAI rằng khoá ấy chỉ thuộc `core::segment`; thật ra `GlossaryError::ImportNotUtf8` (`core/glossary/store.rs:692`, epic 3) dùng chung nó. Khai LẠI khoá ấy trong khối glossary để nó có chủ tường minh. Khoá tra bằng nội suy chuỗi thì cổng tĩnh không thấy — dùng ánh xạ literal -- thêm khoá cùng lúc với tính năng, không dựng sẵn từ vựng
- [x] `tests/frontend/importPreviewEncoding.test.ts` -- tạo mới: dải mở đúng khi và chỉ khi tin cậy thấp; chọn ô khác gọi lại chuỗi; hai tầng rỗng nói ra lý do và tên chủ -- khuôn `glossaryImportPreview.test.ts`
- [x] `_bmad-output/implementation-artifacts/deferred-work.md` -- đóng mục `NotUtf8` bằng `→ ✅ ĐÃ ĐÓNG 2026-09-04 (Story 6.3)`; ghi nợ MỚI có chủ cho: tầng 2 (6.9), tầng 3 (6.5), số đo tỉ lệ dò đúng (**Ice**), nhánh *nguồn tự khai* của HTTP `charset` (6.7) và `.docx` (6.12) -- không mục nào mồ côi, không mục nào đóng khống

**Acceptance Criteria:**
- Given bảng năm nhãn FR126, when đối chiếu với PRD `prd.md:355`, then năm nhãn khớp **theo đúng thứ tự**, và cổng đỏ khi một nhãn bị đổi chỗ chứ không chỉ khi một nhãn biến mất.
- Given cây nguồn sau story, when đếm tệp sản phẩm nêu tên `chardetng`, then có **đúng một**, ở `core/segment/`, và `core/webimport/` vẫn 0 dòng mã.
- Given cổng mới, when gỡ nó ra và chạy lại bộ test CŨ, then bộ cũ **xanh** — chứng minh mệnh đề mới thật sự chưa ai canh.
- Given bộ ca Rust và vitest đang xanh trước story, when chạy sau story, then vẫn xanh **mà không sửa một kỳ vọng nào**, trừ những chỗ chạm trực tiếp tên `NotUtf8` — nơi lượt sửa phải kèm lý do là **đổi tên**, không phải mệnh đề đã nới.
- Given `PIPELINE_ORDER` và `segment_pipeline_boundary.rs`, when chạy sau story, then bảy bước và chỗ gọi sản phẩm duy nhất **không đổi** — story này thêm dữ liệu vào bước 1, không thêm bước.
- Given hai tầng chưa có thân, when đọc màn xem trước, then mỗi tầng hiện **lý do rỗng** kèm tên story chủ, và không tầng nào biến mất khỏi màn hình chỉ vì thân rỗng.
- Given `npm run check:debt-owner`, when chạy sau story, then **0 mục mở mồ côi**, và mục `NotUtf8` đọc là đã đóng bằng **dòng `→ ✅`** — không phải bằng một câu "đã đóng" nằm trong thân mục, vì `check-debt-owner.mjs:221` chỉ đọc dòng `→`.
- Given mọi chỗ còn nêu tên `not_utf8`/`NotUtf8` sau story, when soát từng chỗ, then mỗi chỗ **hoặc** thuộc đường glossary (chủ thật của khoá ấy từ epic 3), **hoặc** là một chú thích 🔵 có ngày giải thích chính lượt đổi tên — và không chỗ nào là đường nhập tài liệu.

## Spec Change Log

### Vòng rà 1 — 2026-09-04 (ba lớp rà đối kháng hội tụ; loại `bad_spec`)

**Phát hiện kích hoạt.** Năm khuyết tật hạng NẶNG, và cả năm nở ra từ **chỗ spec này viết
thiếu**, không từ một lượt thi hành cẩu thả:

1. `label_for_encoding` gộp `UTF_16LE`/`UTF_16BE` vào một nhãn `"UTF-16"`, `encoding_for_label`
   giải ngược ra `UTF_16LE` cứng ⇒ tệp UTF-16BE có BOM ra chữ đảo byte, **không lỗi nào ném**,
   và AD-4 đóng băng nó xuống đĩa. Trước story tệp ấy bị TỪ CHỐI; story làm nó được NHẬN SAI.
2. Phép so "cùng một chuỗi" chạy trên bản đã cắt còn 12 ký tự hiển thị, còn `chardetng` đoán
   trên cả tệp ⇒ một tệp GBK mở đầu bằng dòng ASCII 12 ký tự ra **tin cậy cao**, dải không mở,
   đúng loại tệp FR126 sinh ra để cứu.
3. `all_same` đúng-rỗng: `windows(2).all()` trên 0 hoặc 1 phần tử trả `true` ⇒ tin cậy CAO cho
   một tệp mà KHÔNG bảng nào giải mã được.
4. Mệnh đề §Always *"đổi bảng mã chạy lại chuỗi từ bước một, trong bộ nhớ"* **chưa được dựng** —
   chọn ô khác chỉ gán một biến state.
5. `cancelImportPreview()` không xoá nguồn đang chờ ⇒ huỷ rồi xác nhận vẫn ghi được, phá hàng
   ma trận *"huỷ ⇒ 0 lượt ghi"*.

**Phép đo quyết định.** Thay lượt giải nhãn→bảng mã bằng `encoding_rs::UTF_8` cứng — tức vứt bỏ
hoàn toàn lựa chọn của người dùng — rồi chạy toàn bộ: **1038 passed / 0 failed**. Tải trọng
trung tâm của story không được một ca nào quan sát.

**Đã sửa gì trong spec.** §Design Notes ghim PHẠM VI của phép so và ba ca biên của nó; §Design
Notes thêm hợp đồng *nhãn đi qua dây phải KHÔNG MẤT MÁT*; §Tasks nêu đích danh cơ chế chạy lại
chuỗi, đường huỷ, và việc đọc byte MỘT lần; §Verification đổi từ "chạy cho xanh" sang **bắt buộc
ba phép đột biến phải làm bộ test ĐỎ**; một tiêu chí `grep` dựng trên tiền đề sai đã được thay.

**Trạng thái xấu đã biết, tránh lặp lại.** Một spec tả *"mọi ứng viên giải mã được cho ra cùng
một chuỗi"* mà không nói **trên bao nhiêu byte**, và tả năm nhãn mà không nói **nhãn có phải là
định danh đủ để giải ngược không**, sẽ lại sinh ra đúng hai khuyết tật 1 và 2.

**GIỮ — những thứ đã làm tốt, phải sống sót qua lượt dựng lại:**

- Bộ giải mã **dòng chảy** trong `render_candidates` (`decode_to_string_without_replacement`,
  `last: false`): đoạn xem trước bị cắt giữa một ký tự nhiều byte KHÔNG được tính là "không ra
  chữ". Spec bản đầu không lường; đây là một cải thiện thật.
- `FR126_LABELS` khai thành **dữ liệu**, và cổng `assert_eq!` trên mảng đọc được **sự lệch** chứ
  không chỉ sự tồn tại — đã tự đối chứng: gỡ cổng thì bộ cũ xanh (1034/0), gieo hoán vị thì cổng
  đỏ (exit 101).
- Loại UTF-16 khỏi phép biểu quyết "cùng một chuỗi", kèm lý do đo được (`b"abcd"` giải mã
  UTF-16LE ra `扡摣` — hợp lệ, không lỗi, khác chuỗi ⇒ để nó dự vote thì MỌI tệp ASCII rơi xuống
  tin cậy thấp, trái một hàng ma trận đã đóng băng).
- **Khai lại** `MessageKey::ImportNotUtf8` trong khối glossary thay vì gỡ nó: khoá ấy dùng chung
  với `GlossaryError::ImportNotUtf8` từ epic 3, và việc khai lại biến một chỗ "mượn" ngầm thành
  một chỗ có chủ tường minh. Tốt hơn chỉ thị gốc của spec.
- `PipelineInput::with_encoding` đứng CẠNH `default_shaped`; `PIPELINE_ORDER` và bảy bước không
  đụng; `segment_pipeline_boundary.rs` giữ nguyên 6 ca xanh.
- Khuôn state/lớp phủ theo `glossaryImportState.ts` (vé `sequence`, export `readonly`, `reset*`),
  và mọi lượt sửa tại chỗ kèm 🔵 + ngày ở `Cargo.toml` · `pipeline.rs` · `import.rs` ·
  `scope_contract.rs` · `project_contract.rs`.

## Design Notes

**Ba trạng thái tin cậy là luật CỦA TA — thư viện không cấp.** Đo trước khi thiết kế: toàn bộ API công khai của `chardetng::EncodingDetector` là `new` · `feed` · `guess` (`chardetng-1.0.0/src/lib.rs:2938,3002,3204`); **không có `guess_assess`**, và `feed` trả bool chỉ có nghĩa *"đã thấy ít nhất một byte không phải ASCII"* (`:2929-2931`). Nên một trạng thái *"tin cậy cao"* lấy từ thư viện là một con số bịa. Luật thay thế, đo được và tất định:

- **nguồn tự khai** — có BOM (`EF BB BF` · `FF FE` · `FE FF`), hoặc đầu vào đã là `String` (`ChapterInput::AlreadyText`), hoặc đầu vào **rỗng** (không byte nào để mắt phân xử). Đây cũng là **đường duy nhất** UTF-16 vào được: `chardetng` không bao giờ trả nó.
- **tự đoán, tin cậy cao** — đoán rơi trong năm bảng FR126, **và** có **ÍT NHẤT HAI** ứng viên byte-đơn-vị giải mã được, **và** mọi ứng viên giải mã được cho ra **cùng một chuỗi** ⇒ không có gì để mắt chọn.
- **tự đoán, tin cậy thấp** — mọi ca còn lại: ≥2 chuỗi khác nhau, hoặc đoán rơi ngoài năm bảng, hoặc **dưới hai** ứng viên giải mã được (gồm ca **không bảng nào** giải mã được).

🔴 **PHẠM VI của phép so là bắt buộc, không phải chi tiết thi hành.** Phép so "cùng một chuỗi"
chạy trên **TOÀN BỘ đoạn byte đầu đã dùng để dựng ứng viên**, không trên bản đã cắt ngắn để hiển
thị. Cắt ngắn là việc của **lớp hiển thị**, và chỉ của nó. Lý do đo được (vòng rà 1): một tệp GBK
mở đầu bằng `"Chapter 01\r\n"` — đúng 12 ký tự ASCII — cho cả bốn ứng viên **cùng một chuỗi** ở
12 ký tự đầu, nên một phép so trên bản cắt ngắn kết luận **tin cậy cao** và dải không mở, trên
đúng loại tệp mà FR126 tồn tại để cứu. Hai nửa của phán quyết (`chardetng` đoán, và phép so ứng
viên) phải nhìn **cùng một cửa sổ bằng chứng**; nếu hai cửa sổ khác nhau thì phải khai ra vì sao.

🔴 **Nhãn đi qua dây phải KHÔNG MẤT MÁT.** Năm nhãn FR126 là từ vựng cho **mắt người**, không
nhất thiết là **định danh đủ để giải ngược ra một bộ giải mã**. Cụ thể: `UTF-16` gộp hai thứ tự
byte, nên một lượt quay vòng `bảng mã → nhãn → bảng mã` làm mất thứ tự byte, và một tệp UTF-16BE
sẽ ra chữ đảo byte **mà không lỗi nào ném** — rồi AD-4 đóng băng nó xuống đĩa. Hợp đồng: lượt xác
nhận phải khôi phục **đúng bảng mã** đã dò/đã chọn, không suy lại từ nhãn hiển thị; và một nhãn
**không nhận ra** là một vi phạm hợp đồng ⇒ `IpcError` tường minh, **không** âm thầm rơi về UTF-8
(rơi về mặc định làm một tệp ASCII-ish nhập "thành công" dưới một bảng mã không ai yêu cầu).

Luật này **dựng lại đúng màn hình đã thiết kế**: một tệp GBK chữ Hán cho GBK và GB18030 cùng một chuỗi nhưng Big5 một chuỗi khác ⇒ tin cậy thấp ⇒ dải mở — khớp `mockups/web-import.html:253,262-263`, nơi chip ghi *"GB18030 · tự đoán · độ tin cậy thấp"*. Một tệp ASCII cho năm bản dựng giống hệt ⇒ tin cậy cao ⇒ dải không mở, người dùng không bị hỏi vô cớ.

**Vì sao GBK và GB18030 hiện chữ y hệt, và vì sao đó không phải lỗi.** `encoding_rs` khai thẳng: *"The decoder for this encoding is the same as the decoder for gb18030."* (`encoding_rs-0.8.35/src/lib.rs:946`). Hai bảng chỉ khác ở chiều **mã hoá**, mà đường nhập không dùng. Giữ cả hai ô vì FR126 liệt năm bảng và người dùng đọc nhãn; ô thứ hai mang phán xét **"cũng ra chữ"**, đúng như mockup.

**Vì sao dải năm bản dựng KHÔNG là một bước của chuỗi.** `PIPELINE_ORDER` là bảy bước chạy **một lần trên một bảng mã đã chọn**. Dựng năm bản là chạy **một bước năm lần trên năm bảng** — một hàm song song, không một bước thứ tám. Nhét nó vào chuỗi sẽ làm `assert_eq!` của `segment_pipeline_boundary.rs:147` đỏ, và cổng ấy đỏ đúng: nó tồn tại để chặn việc cắm bước vào sai chỗ.

**Vì sao byte dựng tay dùng được ở đây mà không dùng được ở bàn đo.** Lệnh cấm (`webimport_probe.rs:250-252`) chặn một vòng tròn: mã hoá bằng `encoding_rs` rồi bảo `chardetng` đọc lại **để tuyên bố tỉ lệ dò đúng**. Test của story này khẳng định **luật của ta** (BOM ⇒ tự khai; hai chuỗi khác nhau ⇒ tin cậy thấp) và **đường dây** (đổi ô ⇒ chuỗi chạy lại) — không câu nào trong đó là một phát biểu về độ chính xác của `chardetng`. Tiền lệ trong kho: `segment_contract.rs:7976` đã dựng byte GBK tay cho cặp đối chứng AD-39. Con số tỉ lệ dò đúng vẫn **chưa có** và vẫn là nợ của Ice.

**Ba vế của AC epic KHÔNG nghiệm thu được ở story này — mỗi vế một chủ.** `epics.md` giữ nguyên; ba vế vào sổ nợ: *"kết quả ở cả ba tầng dựng lại ngay lập tức"* nghiệm thu được ở tầng 1, hai tầng kia rỗng có tên chủ (**6.9** và **6.5**) · *"file `.txt` GBK 2000 chương ra đúng số Chương"* cần mẫu phân tách cấu hình được (**Chủ: 6.6**) và một mẫu thật (**Chủ: Ice**) · nhánh *nguồn tự khai* qua `charset` của HTTP (**Chủ: 6.7**) và qua `.docx` (**Chủ: 6.12**) chưa có đường nhập nào để nghiệm thu.

## Verification

**Commands:**
- `npm run build && cargo test --locked --manifest-path src-tauri/Cargo.toml` -- 🔴 **thứ tự bắt buộc**: thiếu `dist/` thì `cargo test` gãy ở khâu biên dịch chứ không ở một assert (`AGENTS.md:32`). Kỳ vọng: ≥ 1023 passed / 0 failed (số nền của Story 6.2)
- `cargo test --locked --manifest-path src-tauri/Cargo.toml --test segment_encoding_boundary --test segment_pipeline_boundary --test segment_boundary --test ipc_contract` -- 0 failed; `segment_pipeline_boundary` giữ nguyên 6 ca xanh
- `npm run test` -- vitest, 0 failed, gồm tệp mới `importPreviewEncoding.test.ts`
- `npm run check:i18n && npm run check:commands && npm run check:tokens && npm run check:panel-refs && npm run check:debt-owner && npm run check:lint` -- cả sáu **exit 0**
- `cargo tree --locked --manifest-path src-tauri/Cargo.toml --prefix none --no-dedupe | wc -l` -- **bằng nhau trước và sau** (đo bằng `git stash`/`git stash pop`); `chardetng` đã trong cây từ Story 6.1
- `grep -rn "not_utf8\|NotUtf8" src src-tauri e2e tests` -- 🔵 **SỬA (vòng rà 1): KHÔNG kỳ vọng 0 dòng.** Tiền đề cũ sai — khoá `err.import.not_utf8` dùng chung với đường glossary từ epic 3. Soát từng dòng khớp theo tiêu chí nghiệm thu tương ứng; **0 dòng nào thuộc đường nhập tài liệu**

🔴 **Ba phép ĐỘT BIẾN bắt buộc — mỗi phép PHẢI làm bộ test ĐỎ.** Một phép cho xanh nghĩa là mệnh
đề tương ứng chưa ai canh, và ô Execution tương ứng CHƯA đóng được (vòng rà 1 đã đo: phép thứ nhất
cho **1038 passed / 0 failed**, tức tải trọng trung tâm của story không ca nào quan sát):

- Thay lượt giải bảng-mã-đã-chọn bằng `encoding_rs::UTF_8` cứng (vứt bỏ lựa chọn của người dùng) -- bộ test phải **ĐỎ**
- Đổi bảng mã trả về cho ca UTF-16BE thành `UTF_16LE` -- bộ test phải **ĐỎ**
- Cho `cancel` KHÔNG xoá nguồn đang chờ -- bộ test phải **ĐỎ**

🔴 **Ba phép ĐỘT BIẾN nữa, thêm ở vòng rà 2 — cả ba đã được ĐO là lỗ thật (mỗi phép cho 0 ca
đỏ khi đo ngày 2026-09-04), nên chúng là nợ phải đóng, không phải nghi ngờ phải kiểm:**

- Xoá dòng đăng ký trạng thái nguồn đang chờ trong `lib.rs` (`app.manage(...)`) -- bộ test phải **ĐỎ**. Đo được: xoá nó làm chức năng nhập CHẾT HOÀN TOÀN (mọi lượt xác nhận trả `no_pending_source`) mà `cargo test` vẫn xanh. Chính story này đã dựng cổng cho lớp lỗi "quên một dòng trong `lib.rs`" ở `generate_handler!`, nhưng bỏ trống nó cho dòng trạng thái mà cả ba lệnh phụ thuộc
- Đặt `#[serde(rename_all = "camelCase")]` lên payload xem trước -- bộ test phải **ĐỎ**. Đo được: tính năng chết trong bản đóng gói (guard kiểu phía TS bác payload, mọi lượt xem trước thành `UNKNOWN_IPC_ERROR`) mà cargo VÀ vitest đều xanh. Hình dạng JSON qua dây phải có một ca khẳng định tên trường, khuôn `pinned_contract.rs:419`
- Xoá lời gọi `finishImportSubmission(...)` trong `src/main.ts` -- bộ test phải **ĐỎ**. Đo được: đó là chỗ gọi sản phẩm DUY NHẤT đóng vòng nhập, và xoá nó giết bất biến reset panel (*"đọc nội dung Tác phẩm A dưới nhãn Tác phẩm B"*) mà vitest vẫn 792 xanh — hai ca tự CHÉP LẠI dây của `main.ts` thay vì đi qua nó

**Manual checks:**
- **Đối chứng đỏ bằng phép GỠ thật**: xoá hẳn `segment_encoding_boundary.rs`, chạy lại toàn bộ `cargo test` — phải **xanh**. Rồi khôi phục, đảo thứ tự hai nhãn trong bảng năm bảng mã — cổng phải **đỏ**. Một trong hai lượt sai kỳ vọng ⇒ nghi phép đối chứng trước, nghi bộ test sau.
- Chạy sản phẩm, nhập một tệp GBK: dải mở, năm ô, ô GBK và GB18030 cùng chữ; chọn ô khác ⇒ ba tầng dựng lại; **kiểm thư mục Library: không thư mục `.atproj` nào sinh ra trước khi bấm xác nhận**.
- Nhập một tệp GBK mở đầu bằng một dòng tiêu đề ASCII: dải **vẫn phải mở** (tin cậy thấp).
- Nhập một tệp UTF-16BE có BOM: chữ hiện ra đọc được, **không** phải chữ Hán đảo byte.
- Bàn đo `webimport_probe.rs:260` giữ nguyên `#[ignore]` và nhánh đỏ 0-mẫu — **không** chạy nó thành xanh bằng fixture tự sinh.

## Suggested Review Order

**Luật ba trạng thái — thứ story này thực sự quyết định**

- Năm nhãn FR126 khai thành dữ liệu; cổng đọc được sự LỆCH, không chỉ sự tồn tại.
  [`encoding.rs:36`](../../src-tauri/src/core/segment/encoding.rs#L36)

- Điểm vào: BOM trước, rồi `chardetng`, rồi luật tin cậy của TA — thư viện không cấp.
  [`encoding.rs:168`](../../src-tauri/src/core/segment/encoding.rs#L168)

- BOM là đường DUY NHẤT UTF-16 vào được; UTF-32 phải loại trước.
  [`encoding.rs:139`](../../src-tauri/src/core/segment/encoding.rs#L139)

- Năm bản dựng thật, giải mã DÒNG CHẢY để đoạn cắt giữa ký tự không bị tính là hỏng.
  [`encoding.rs:236`](../../src-tauri/src/core/segment/encoding.rs#L236)

**Định danh qua dây phải KHÔNG MẤT MÁT — gốc của khuyết tật nặng nhất vòng rà 1**

- Giải ngược theo danh sách cho phép; nhãn ngoài năm bảng bị TỪ CHỐI, không rơi về UTF-8.
  [`encoding.rs:272`](../../src-tauri/src/core/segment/encoding.rs#L272)

- Lượt xác nhận giải `wire_id` thành bảng mã thật rồi mới chạy chuỗi.
  [`project.rs:1088`](../../src-tauri/src/commands/project.rs#L1088)

**Chuỗi nhập — byte đọc một lần, ghi sau xác nhận**

- Xem trước chỉ ĐỌC: dò bảng mã, dựng năm ô, không byte nào xuống đĩa.
  [`project.rs:979`](../../src-tauri/src/commands/project.rs#L979)

- Bảng mã đã chọn tiêm vào bước 1; `PIPELINE_ORDER` bảy bước không đụng.
  [`pipeline.rs:228`](../../src-tauri/src/core/segment/pipeline.rs#L228)

- Dòng đăng ký trạng thái nguồn đang chờ — xoá nó là nhập chết, nay có cổng.
  [`lib.rs:1033`](../../src-tauri/src/lib.rs#L1033)

**Lỗi nói đúng tên bảng mã — món nợ kế thừa từ Story 6.2**

- `NotUtf8` thành `UndecodableBytes`, mang theo bảng mã đã chọn.
  [`import.rs:98`](../../src-tauri/src/core/segment/import.rs#L98)

- Nửa còn lại: khoá thông điệp; khoá cũ ở lại vì glossary mới là chủ thật của nó.
  [`i18n/mod.rs:156`](../../src-tauri/src/core/i18n/mod.rs#L156)

**Màn xem trước ba tầng — chỉ tầng 1 có thân**

- Chọn ô khác đổi lựa chọn tại chỗ; vì sao KHÔNG chạy lại chuỗi thì ghi ở sổ nợ.
  [`importPreviewState.ts:217`](../../src/importPreviewState.ts#L217)

- Xác nhận: vé `sequence` chặn lượt cũ ghi đè lượt mới.
  [`importPreviewState.ts:235`](../../src/importPreviewState.ts#L235)

- Huỷ xoá SẠCH nguồn đang chờ — điều kiện để "huỷ ⇒ 0 lượt ghi" là thật.
  [`importPreviewState.ts:300`](../../src/importPreviewState.ts#L300)

- Nộp form giờ chỉ MỞ xem trước; lượt tạo Tác phẩm dời hẳn sang xác nhận.
  [`libraryImport.ts:314`](../../src/modes/libraryImport.ts#L314)

- Đóng vòng: reset panel chỉ chạy SAU khi Rust đã tạo xong.
  [`libraryImport.ts:196`](../../src/modes/libraryImport.ts#L196)

**Cổng — mỗi cổng dưới đây đã được chứng minh là ĐỎ ĐƯỢC**

- Thứ tự năm nhãn khớp PRD; gieo hoán vị hai nhãn thì đỏ.
  [`segment_encoding_boundary.rs:109`](../../src-tauri/tests/segment_encoding_boundary.rs#L109)

- Hình dạng JSON qua dây: đặt `rename_all` vào là đỏ.
  [`segment_contract.rs:8705`](../../src-tauri/tests/segment_contract.rs#L8705)

- Ba vỏ dây và dòng đăng ký trạng thái: xoá một dòng là đỏ.
  [`ipc_contract.rs:812`](../../src-tauri/tests/ipc_contract.rs#L812)
