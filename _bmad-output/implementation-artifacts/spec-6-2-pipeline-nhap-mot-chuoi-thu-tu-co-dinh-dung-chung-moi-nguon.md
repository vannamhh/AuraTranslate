---
title: 'Story 6.2 — Pipeline nhập một chuỗi thứ tự cố định, dùng chung mọi nguồn'
type: 'refactor'
created: '2026-09-04'
status: 'done'
review_loop_iteration: 0
baseline_commit: 'a5701b865a4a501db5d3ff797efa832facdf54a5'
context:
  - '{project-root}/AGENTS.md'
  - '{project-root}/src-tauri/AGENTS.md'
  - '{project-root}/_bmad-output/implementation-artifacts/epic-6-context.md'
---

<frozen-after-approval reason="human-owned intent — do not modify unless human renegotiates">

## Intent

**Problem:** AD-39 khai một chuỗi nhập tám bước dùng chung mọi nguồn, nhưng hôm nay thứ tự ấy chỉ sống bằng **văn xuôi** (`import.rs:14-30`) và bằng hình dạng tình cờ của một hàm 130 dòng (`commands/project.rs:228-359`) trộn lẫn filesystem, tách câu, SQL và ghi `meta.json`. Không một cấu trúc nào khai thứ tự, nên **không cổng nào đỏ** khi story sau cắm một bước vào sai chỗ. Ca hỏng mà AD-39 tồn tại để chặn là ca **im lặng**: đặt tách Chương trước giải mã bảng mã thì mẫu phân tách chạy trên byte chưa giải mã, không khớp gì, cả file 40 MB ra đúng một Chương — và không lỗi nào được ném. Mười sáu story còn lại của epic 6 sẽ cắm bước của mình vào chuỗi này.

**Approach:** Đưa **thứ tự thành dữ liệu**: một hằng khai bảy bước (1→7) và một bộ chạy tiêu thụ chính hằng đó, sống ở `core/segment/pipeline.rs`. Bước 8 (ghi `.atproj`) **ở nguyên** đường ghi hiện có tại `commands/`, và mệnh đề *"không chèn bước sau lệnh ghi"* được canh bằng cổng chứ không bằng vị trí tệp. Vì thứ tự là dữ liệu, đối chứng đỏ dựng được **hôm nay**: test dựng tay một thứ tự SAI rồi cho chạy qua **chính bộ chạy sản phẩm** và khẳng định đúng triệu chứng AD-39 mô tả — một Chương, không lỗi nào.

## Boundaries & Constraints

**Always:**
- **Hành vi sản phẩm không đổi ở story này.** Một lượt nhập vẫn ra đúng một `Work` và đúng một `Chapter` như hôm nay (`project.rs:270-275`, `ord = 1`, `title NULL`). Cái đổi là **hình dạng**, không phải kết quả. Mọi ca test đang xanh của `project_contract.rs` và `segment_contract.rs` phải còn xanh mà không sửa kỳ vọng.
- **Một đường mã duy nhất.** Sản phẩm gọi `run_import`, và `run_import` uỷ quyền cho `run_import_with_order(&PIPELINE_ORDER, …)`. Không có bản chép thứ hai của chuỗi — hai bản cài đặt là vi phạm AD-1, AD-39 nói thẳng ở spine `:498`.
- **Bộ chạy nhận thứ tự tuỳ ý phải công khai để `tests/**` gọi được, và đúng một chỗ gọi sản phẩm.** Một cổng khẳng định `run_import` là chỗ gọi sản phẩm DUY NHẤT của nó — nếu không, cái seam mở cho test sẽ thành đường tắt cho một story sau.
- **Lượt ghi giữ nguyên khuôn bốn bước** của `commands/lifecycle.rs` (ghi SQL → `WorkMeta::rebuild_from_store` → `write_atomic` → `reindex`). Thêm một đường ghi `chapter`/`segment` ngoài khuôn này làm chỉ mục Library nói dối trong im lặng và không cổng nào đỏ — đã đo 0 failed trên 34 binary khi gỡ hẳn bước 4 (`src-tauri/AGENTS.md:29`).
- **Điều kiện áp bước tách Chương khai theo HÌNH DẠNG đầu vào**, không theo danh sách đường nhập (spine `:486-491`). Đầu vào đến dạng byte chưa giải mã ⇒ có bước giải mã; đến dạng văn bản đã khai bảng mã (`.docx`) ⇒ bỏ qua.
- Chuỗi ghi bằng từ vựng cố định: `Work` · `Chapter`. Chuỗi literal trong `src-tauri/src/**` viết không dấu; chú thích tiếng Việt có dấu và chở LÝ DO kèm một phép đo (`tệp:dòng`, số, ngày).
- Mọi mệnh đề hết đúng vì story này thì **sửa tại chỗ kèm 🔵 và ngày**, không xoá — kể cả một dòng chú thích trong `Cargo.toml`.

**Ask First:**
- Bất kỳ đề xuất nào làm **đổi hành vi sản phẩm** (số Chương ra khác 1, đổi chữ ký IPC, thêm tham số lên dây) — story này là một lượt đổi hình dạng, không phải một lượt đổi tính năng.
- Bất kỳ crate mới nào. `encoding_rs =0.8.35` đã ghim và đã rà giấy phép ở Story 6.1 (`Cargo.toml:95`), dùng lại **không** mở cửa NFR15; một crate thứ tư thì có.
- Nếu bộ ca đang xanh **không thể** giữ nguyên kỳ vọng: dừng và trình số, đừng sửa kỳ vọng cho khớp mã.

**Never:**
- Không dựng bộ dò bảng mã (`chardetng`) — đó là Story 6.3. Story này chỉ **giải mã theo một bảng mã đã khai**, mặc định UTF-8, tức đúng hành vi hôm nay.
- Không dựng bóc nội dung (`dom_smoothie`, Story 6.9), luật làm sạch (6.5), chuẩn hoá khoảng trắng (6.4), mẫu phân tách người dùng cấu hình được (6.6), màn xem trước (6.5/6.9). Các bước ấy có mặt trong thứ tự nhưng thân rỗng, mỗi bước một chủ ghi tên.
- Không chạm `core/webimport/` ngoài chú thích, không một dòng mã chạm mạng (AD-41 chưa có cổng nào canh — `src-tauri/AGENTS.md:29`).
- Không đăng ký lệnh hay hợp âm nào vào `CommandRegistry`.
- Không hạ một cổng đang xanh để đi tiếp: `segment_boundary.rs` sẽ đỏ vì chỗ gọi bộ tách **dời chỗ**, và lượt sửa nó phải giữ nguyên hai mệnh đề nó đang canh, không phải nới chúng.

## I/O & Edge-Case Matrix

| Scenario | Input / State | Expected Output / Behavior | Error Handling |
|---|---|---|---|
| Thứ tự đúng, văn bản UTF-8 | Byte UTF-8, mẫu phân tách có khớp | N Chương theo mẫu, mỗi Chương có segment và cờ kết đoạn | N/A |
| **Thứ tự sai — tách Chương trước giải mã** | Byte GBK, mẫu phân tách UTF-8 | **Đúng 1 Chương**, chuỗi chạy hết, **không lỗi nào ném** — test khẳng định đúng triệu chứng này | Không có lỗi: đó CHÍNH LÀ khuyết tật |
| Đầu vào đã là văn bản (`.docx`) | Hình dạng `AlreadyText` | Bước giải mã **bỏ qua**, các bước sau chạy đủ | N/A |
| Đầu vào một đơn vị một Chương | Hình dạng "đã chia Chương" | Bước tách Chương **bỏ qua**, ra đúng số đơn vị vào | N/A |
| Byte không hợp lệ với bảng mã đã khai | Byte hỏng, khai UTF-8 | Từ chối tường minh, đúng biến thể `ImportError` đang có | `ImportError::NotUtf8` → `IpcError` qua `From` sẵn có (`import.rs:128`) |
| Không có mẫu phân tách | Đường sản phẩm hôm nay | Đúng 1 Chương — hành vi hiện tại, không đổi | N/A |
| Bước rỗng chưa có chủ thi hành | Bóc / làm sạch / chuẩn hoá / xem trước | Đi qua không đổi văn bản, và **nói được là đã đi qua** | Không nuốt im lặng: bước rỗng vẫn có mặt trong vết chạy |

</frozen-after-approval>

## Code Map

- `src-tauri/src/core/segment/import.rs:187` `ImportedChapter { source_text }` — **điểm gãy hẹp nhất**: chuỗi phải trả NHIỀU Chương, kiểu này trả một. `:197` `import_text` (chỉ gọi `strip_bom`), `:241` `import_file` (đọc đĩa, chặn phần mở rộng, trần kích thước, UTF-8 nghiêm ở `:269`), `:69` `ImportError` (5 biến thể), `:128` `From<ImportError> for IpcError`, `:228` `strip_bom`. `:27-33` khai thẳng rằng dò bảng mã, tách Chương và chuẩn hoá là **cố ý để trống** cho epic 6 — sẽ hết đúng, sửa tại chỗ kèm 🔵.
- `src-tauri/src/core/segment/split.rs:219` `split_source_text` (`:112` `SplitSegment`) — bước 7. **Gọi, đừng viết lại.**
- `src-tauri/src/core/segment/mod.rs:38-43` — chỗ khai module con; `pipeline` thêm vào đây.
- `src-tauri/src/commands/project.rs:228` `create_work` (130 dòng, trộn bốn tầng — chính là trở ngại làm thứ tự không cưỡng chế được). `:31` `use` bộ tách, `:260` chỗ gọi bộ tách, `:263-292` closure ghi một giao dịch, `:271` `INSERT INTO chapter` (`ord` cứng 1), `:305` `rebuild_from_store`, `:330` `write_atomic`. Hai vỏ thuần `:362` `create_work_from_text` · `:782` `create_work_from_file`; vỏ dây `:1960` · `:1992`.
- `src-tauri/src/commands/segment.rs:99` `insert_segments`, `:301` `split_chapter_into_segments` (chỗ gọi bộ tách thứ hai, `:334`), `:45` `use`.
- `src-tauri/src/commands/lifecycle.rs:87` `write_lifecycle_after_change` · `:253` `finish_lifecycle_write` — khuôn bốn bước dùng chung, **không được có bản chép thứ hai**.
- `src-tauri/tests/segment_boundary.rs:316` `the_splitter_has_exactly_two_product_call_sites` — ⚠️ **sẽ đỏ**. `:322` bỏ qua mọi tệp dưới `core/segment` (nên pipeline ở đó **vô hình** với cổng), nhưng `:339` đòi `commands/project.rs` PHẢI còn chứa `split_source_text` — dời lời gọi ở `project.rs:260` làm đúng dòng ấy đỏ. Đo: ngoài `core/segment/` hiện có đúng 4 dòng khớp không phải chú thích (`project.rs:31,260` · `segment.rs:45,334`). `:369` `the_splitter_stays_pure`.
- `src-tauri/tests/ai_boundary.rs:225` sàn quần thể (cây rỗng đọc thành sạch) · `:302` **kiểm chứng dương** — chạy vị từ thật trên một vi phạm dựng tay. Đây là khuôn cho cổng thứ tự.
- `src-tauri/tests/config_invariants.rs:896` — tiền lệ khẳng định một con số CHÍNH XÁC trên cây nguồn.
- `src-tauri/tests/segment_contract.rs:1-12` bốn luật thừa kế (một temp dir mỗi ca, thả `Store` trước khi xoá thư mục trên Windows, không `sleep` dài, không treo khi trượt); `:41` `temp_dir` chép tay — quy ước, không phải sơ suất.
- `src-tauri/Cargo.toml:95` `encoding_rs =0.8.35` đã ghim, đã bắc cầu sẵn nên tốn **0 byte**; `:92-94` chú thích khai chủ là `core::webimport` — **hết đúng** khi bước giải mã về `core/segment/`.
- `_bmad-output/planning-artifacts/architecture/…/ARCHITECTURE-SPINE.md:465-504` AD-39 (`:470` ca 40 MB, `:473-482` thứ tự tám bước, `:486-491` bảng hình dạng, `:498` chuỗi sống ở `core/segment/`, `:500` `.docx` bỏ qua giải mã, `:502` xem trước sau TOÀN BỘ chuỗi).
- `_bmad-output/implementation-artifacts/deferred-work.md:2941-2951` — nợ hợp âm `⌘↵`, đang ghi *"Chủ: Story 6.2"*.

## Tasks & Acceptance

**Execution:**
- [x] `src-tauri/src/core/segment/pipeline.rs` -- tạo mới: kiểu giá trị chảy trong chuỗi (byte chưa giải mã / văn bản), `Step`, hằng `PIPELINE_ORDER` bảy bước theo đúng spine `:473-482`, `run_import_with_order(order, input)` và `run_import(input)` uỷ quyền cho nó -- thứ tự thành dữ liệu là điều kiện để đối chứng đỏ dựng được mà không cần 6.3/6.6
- [x] `src-tauri/src/core/segment/mod.rs` -- khai `pipeline` cạnh sáu module con đang có -- `:38-43`
- [x] `src-tauri/src/core/segment/import.rs` -- `import_text`/`import_file` thu về đúng **bước đầu vào** (byte + hình dạng); bước giải mã và `strip_bom` chuyển vào chuỗi; `ImportedChapter` thành kết quả NHIỀU Chương; sửa tại chỗ kèm 🔵 khối `:14-33` -- đây là điểm gãy hẹp nhất, và mệnh đề "cố ý để trống" hết đúng một nửa
- [x] `src-tauri/src/commands/project.rs` -- `create_work` gọi `run_import`, ghi N Chương (`ord` 1..N) trong CÙNG giao dịch đang có, giữ nguyên khuôn bốn bước; gỡ lời gọi bộ tách ở `:260` và `use` ở `:31` -- N = 1 ở story này, nên hành vi không đổi mà đường đi tổng quát
- [x] `src-tauri/tests/segment_boundary.rs` -- sửa cổng `:316` cho địa hình mới: chỗ gọi sản phẩm ngoài `core/segment/` còn ĐÚNG MỘT (`commands/segment.rs`), và `core/segment/pipeline.rs` là chỗ gọi duy nhất trong module; giữ nguyên hai mệnh đề AC3 và AC8-vế-hai, kèm kiểm chứng dương -- cổng dời theo mã, không được nới
- [x] `src-tauri/tests/segment_pipeline_boundary.rs` -- tạo mới: `PIPELINE_ORDER` khớp đúng thứ tự AD-39; `run_import` là chỗ gọi sản phẩm DUY NHẤT của bộ chạy nhận thứ tự tuỳ ý; sàn quần thể + kiểm chứng dương theo khuôn `ai_boundary.rs:225,302` -- seam mở cho test không được thành đường tắt cho story sau
- [x] `src-tauri/tests/segment_contract.rs` -- ca hành vi: thứ tự sai ⇒ đúng 1 Chương và KHÔNG lỗi; thứ tự đúng ⇒ N Chương; đầu vào đã-là-văn-bản bỏ qua bước giải mã; đầu vào đã-chia-Chương bỏ qua bước tách -- tên hàm là câu khẳng định, không `test_foo`
- [x] `src-tauri/Cargo.toml` -- sửa tại chỗ kèm 🔵 và ngày: chủ của `encoding_rs` là `core::segment` (bước giải mã của chuỗi), `chardetng` vẫn `core::webimport` -- `:92-94`, mệnh đề hết đúng thì sửa chứ không để lặng lẽ sai
- [x] `_bmad-output/implementation-artifacts/deferred-work.md` -- nối 🔵 vào mục `:2941-2951`: chủ hợp âm `⌘↵` chuyển sang story đăng ký lệnh xác nhận nhập thật, kèm lý do đo được (6.2 đăng ký 0 lệnh); ghi nợ mới cho mọi vế story này KHÔNG nghiệm thu được, mỗi vế một chủ -- không mục nào mồ côi, không mục nào đóng khống

**Acceptance Criteria:**
- Given cây nguồn sau story, when đếm chỗ cài đặt chuỗi, then có ĐÚNG MỘT, ở `core/segment/`, và không module nguồn nào giữ bản sao của một bước dùng chung.
- Given `PIPELINE_ORDER`, when đối chiếu với AD-39 spine `:473-482`, then bảy bước khớp từng bước theo đúng thứ tự, và cổng đọc được sự lệch chứ không chỉ đọc được sự tồn tại.
- Given lượt ghi `.atproj`, when soát, then nó vẫn đi qua đúng khuôn bốn bước của `commands/lifecycle.rs`, và không bước nào của chuỗi chạy SAU nó.
- Given bộ ca Rust đang xanh trước story, when chạy sau story, then vẫn xanh **mà không sửa một kỳ vọng nào** — trừ `segment_boundary.rs:316`, nơi lượt sửa phải kèm lý do là chỗ gọi ĐÃ DỜI, không phải mệnh đề đã nới.
- Given cổng thứ tự mới, when gỡ nó ra và chạy lại bộ test CŨ, then bộ cũ **xanh** — chứng minh mệnh đề mới thật sự chưa ai canh, không phải một ca trùng.
- Given ba bước chưa có thân (bóc, làm sạch, chuẩn hoá) và bước xem trước, when đọc mã, then mỗi bước mang tên story chủ tại chỗ, và không bước nào biến mất khỏi thứ tự chỉ vì thân rỗng.

## Spec Change Log

## Design Notes

**Vì sao chuỗi dừng ở bước 7.** AD-39 `:498` nói *"chuỗi này sống ở `core/segment/`"* và liệt bước ghi là bước 8 — đọc mặt chữ thì lượt ghi cũng phải về `core/segment/`. Không chọn đường đó: mọi lượt ghi `chapter`/`segment` phải đi qua khuôn bốn bước của `commands/lifecycle.rs`, và bản chép thứ hai của khuôn ấy mở lại đúng lớp lỗi đã đo được là **0 failed trên 34 binary** khi gỡ hẳn bước 4 (`src-tauri/AGENTS.md:29`) — tức một khuyết tật mà không cổng nào thấy. Mệnh đề AD-39 thật sự cần canh là *"không đường nào chèn bước SAU lệnh ghi"*, và mệnh đề đó canh được bằng cổng, không cần lượt ghi đổi chỗ ở. Ice chốt 2026-09-04.

**Vì sao thứ tự phải là dữ liệu, chứ không phải một hàm gọi lần lượt.** Nếu thứ tự nằm cứng trong thân một hàm thì *đặt sai thứ tự* không biểu diễn được lúc chạy, và đối chứng cho AC5 chỉ còn là một phép quét chữ trên mã nguồn — kiểm CHỮ chứ không kiểm HÀNH VI. Khai thứ tự thành một hằng và cho bộ chạy tiêu thụ chính hằng đó biến "một thứ tự sai" thành một giá trị dựng được, nên test chạy được đúng bộ chạy sản phẩm trên nó. Đây là khuôn `ai_boundary.rs:302` — vị từ THẬT chạy trên một vi phạm dựng tay.

**Vì sao ca đối chứng cần byte chưa giải mã, chứ không phải một chuỗi.** UTF-8 giữ nguyên byte, nên một mẫu phân tách UTF-8 vẫn xuất hiện nguyên vẹn trong dòng byte UTF-8 — đặt tách Chương trước giải mã trên đầu vào UTF-8 thì hai thứ tự cho **cùng một kết quả**, và ca test xanh mà chẳng chứng minh gì. Triệu chứng chỉ hiện ra khi byte thuộc một bảng mã nhiều byte KHÁC UTF-8. Vì thế giá trị chảy trong chuỗi phải phân biệt được *byte chưa giải mã* với *văn bản*, và bước tách Chương gặp byte thì khớp trên byte — không khớp gì, ra một Chương, **không ném lỗi**. Đúng câu spine `:470` mô tả.

**Ba vế của AC epic KHÔNG nghiệm thu được ở story này — ghi tên ra, mỗi vế một chủ.** Năng lực chưa dựng không phải một chỗ lệch spec, nên `epics.md` giữ nguyên và ba vế dưới đi vào sổ nợ:

- *"Ba đường nhập file, URL và song ngữ khác nhau CHỈ ở bước đầu vào"* — hôm nay chỉ có đường file/dán tay. Chuỗi dựng chỗ cắm cho hai đường kia, nhưng mệnh đề "ba đường" chỉ đếm được khi đường URL tồn tại. **Chủ: Story 6.7**, đóng trọn ở **6.16**.
- *"Màn xem trước luôn hiện kết quả sau TOÀN BỘ chuỗi"* — bước xem trước có mặt trong thứ tự và đứng đúng chỗ (sau tách Chương, trước tách segment), nhưng chưa có bề mặt nào để hiện. **Chủ: Story 6.5.**
- *"Một `.docx` bỏ qua bước giải mã bảng mã"* — nghiệm thu ở story này bằng một đầu vào **hình dạng** đã-là-văn-bản, không bằng một `.docx` thật; thư viện đọc `.docx` còn là một hàng Deferred chưa đóng. **Chủ: Story 6.12** cho ca `.docx` thật.

**Bảng mã đã khai khác bộ dò bảng mã.** Story này giải mã theo một bảng mã **được khai** (mặc định UTF-8 ⇒ hành vi sản phẩm không đổi). Story 6.3 mới dựng phần **dò** bằng `chardetng` và dải năm ứng viên. Lằn ranh này cũng là lý do lệnh cấm tự sinh fixture GBK của Story 6.1 không áp vào đây: cấm ấy chặn một vòng tròn *"mã hoá rồi bảo chính bộ dò đọc lại"* khi ĐO ĐỘ CHÍNH XÁC CỦA BỘ DÒ; ở đây không có bộ dò nào tham gia, bảng mã do ca test khai.

## Verification

🔵 **SỬA (vòng rà đối kháng 2026-09-04, item 17) — số THẬT thay cho "xanh"/"không đổi" không kèm con số.** Đo lại TOÀN BỘ sau lượt vá 21 mục của vòng rà; mọi ô Execution đã `[x]`.

**Commands — số đo được (2026-09-04, sau lượt vá):**
- `npm run build && cargo test --locked --manifest-path src-tauri/Cargo.toml` -- **1023 passed / 0 failed / 11 ignored** (toàn bộ `cargo test`, gồm doctests). Thứ tự bắt buộc: thiếu `dist/` thì `cargo test` gãy ở khâu biên dịch chứ không ở một assert (`AGENTS.md:32`)
- `cargo test --locked --manifest-path src-tauri/Cargo.toml --test segment_boundary` -- **9 passed / 0 failed**, gồm cổng đếm chính xác (`the_splitter_has_exactly_one_product_call_site_outside_core_segment`, đúng 2 dòng khớp) và đối chứng dương lọc chú thích (`the_pipeline_module_actually_calls_the_splitter`)
- `cargo test --locked --manifest-path src-tauri/Cargo.toml --test segment_pipeline_boundary` -- **6 passed / 0 failed**, gồm sàn quần thể, cổng `PIPELINE_ORDER`, cổng `run_import`/`run_import_with_order` (cả hai tên), và hai ca kiểm chứng dương
- `cargo test --locked --manifest-path src-tauri/Cargo.toml --test segment_contract --test project_contract` -- **166 + 79 = 245 passed / 0 failed / 1 ignored** (ca `#[ignore]` là bàn đo 5.000 Chương có sẵn từ trước, không liên quan story này) -- không sửa kỳ vọng nào đã xanh trước vòng rà, chỉ THÊM ca và sửa CÁCH GỌI (`import_text`/`import_file` đổi kiểu trả về, xem Task 3)
- `npm run check:deps && npm run check:commands && npm run check:debt-owner` -- cả ba **exit 0**; `check:commands` xác nhận 130 command không đổi (story này đăng ký 0); `check:debt-owner` xác nhận 0/416 mục mở thiếu chủ (639 mục tổng, 71 nửa, 142 đóng)
- `cargo tree --locked --manifest-path src-tauri/Cargo.toml --prefix none --no-dedupe | wc -l` -- **246363 dòng, trước và sau đều bằng nhau** (đo bằng `git stash`/`git stash pop` đối chiếu hai bản). `encoding_rs` đã trong cây từ Story 6.1

**Manual checks — đã chạy, kết quả THẬT (không phải "phải là"):**
- Gỡ hẳn tệp `segment_pipeline_boundary.rs` (cổng thứ tự mới) rồi chạy lại toàn bộ `cargo test`: **xanh, 0 failed** trên phần còn lại — mệnh đề mới thật sự chưa ai canh trước story này (`PIPELINE_ORDER`/`Step`/`pipeline.rs` là mã hoàn toàn mới, không tồn tại trước Story 6.2 để một cổng cũ có thể trùng).
- Đảo `Step::DecodeEncoding` ↔ `Step::SplitChapters` trong `PIPELINE_ORDER` rồi chạy: `segment_pipeline_boundary` **5 passed / 1 failed** (`pipeline_order_matches_ad_39_step_by_step` đỏ, so lệch mảng đúng như kỳ vọng) VÀ `segment_contract` **165 passed / 1 failed** (`splitting_chapters_after_decoding_finds_the_pattern_and_produces_n_chapters` đỏ, `left: 1, right: 3`) -- cả hai cổng đỏ, đúng "một trong hai xanh thì phép đối chứng hỏng". Khôi phục bằng `cp` từ bản sao lưu, đối chiếu `diff` xác nhận khớp byte-for-byte, chạy lại **xanh 100%**.
- `git diff src-tauri/src/commands/project.rs`: khoảng cách từ `INSERT INTO chapter` tới `write_atomic` không chèn thêm bước nào của chuỗi -- xác nhận bằng mắt, không có cách đo tự động cho mệnh đề này.

**Vòng rà đối kháng 2026-09-04 — 21 mục `patch`, tất cả đã vá và đo lại số ở trên.** Không mục nào là `intent_gap`/`bad_spec`. Chi tiết từng mục nằm trong lịch sử commit/hội thoại của lượt vá này, không lặp lại ở đây.

## Suggested Review Order

**Thứ tự là dữ liệu — đọc trước tiên, vì đây là thứ story này thực sự giao**

- Điểm vào: bảy bước khai thành một hằng, đối chiếu được từng bước với AD-39 spine `:473-482`.
  [`pipeline.rs:104`](../../src-tauri/src/core/segment/pipeline.rs#L104)

- Bộ chạy tiêu thụ CHÍNH hằng đó — không có bản chép thứ hai của chuỗi.
  [`pipeline.rs:292`](../../src-tauri/src/core/segment/pipeline.rs#L292)

- Đường sản phẩm duy nhất, uỷ quyền thẳng; seam nhận thứ tự tuỳ ý không lộ ra `commands/`.
  [`pipeline.rs:375`](../../src-tauri/src/core/segment/pipeline.rs#L375)

- Từ chối một thứ tự không phải hoán vị — bước trùng, bước thiếu, mảng rỗng đều chặn TRƯỚC khi chạy.
  [`pipeline.rs:119`](../../src-tauri/src/core/segment/pipeline.rs#L119)

**Điều kiện áp bước khai theo HÌNH DẠNG, không theo độ dài**

- Cờ đặt MỘT LẦN từ `PipelineShape`; một danh sách một link không bị tách lại.
  [`pipeline.rs:277`](../../src-tauri/src/core/segment/pipeline.rs#L277)

- Bước tách Chương rẽ theo cờ đó, không đếm `units.len()`.
  [`pipeline.rs:477`](../../src-tauri/src/core/segment/pipeline.rs#L477)

- `.docx` bỏ qua vế transcode; UTF-8 giữ đường 0 chép thay vì copy trọn bộ đệm.
  [`pipeline.rs:404`](../../src-tauri/src/core/segment/pipeline.rs#L404)

**Bước 8 ở nguyên `commands/` — và không bước nào chạy sau nó**

- Chuỗi chạy TRƯỚC và NGOÀI giao dịch; kết quả mới đi vào lượt ghi.
  [`project.rs:269`](../../src-tauri/src/commands/project.rs#L269)

- Danh sách rỗng bị từ chối TRƯỚC khi mở giao dịch — `panic = "abort"` không có chỗ bám.
  [`project.rs:287`](../../src-tauri/src/commands/project.rs#L287)

- Ghi N Chương `ord` 1..N trong đúng giao dịch cũ, khuôn bốn bước không đổi.
  [`project.rs:326`](../../src-tauri/src/commands/project.rs#L326)

**Hai hàm nhập thu về đúng BƯỚC ĐẦU VÀO**

- Dán tay nay chỉ dựng hình dạng, không giải mã, không cắt BOM.
  [`import.rs:240`](../../src-tauri/src/core/segment/import.rs#L240)

- Đọc tệp trả byte thô kèm hình dạng; giải mã lùi vào chuỗi.
  [`import.rs:259`](../../src-tauri/src/core/segment/import.rs#L259)

**Cổng — chỗ vòng rà đối kháng bắt được lỗ hổng nặng nhất**

- ⚠️ Bản đầu chỉ bắt `run_import(`, nên `run_import_with_order(` lọt qua xanh. Nay hai vị từ.
  [`segment_pipeline_boundary.rs:123`](../../src-tauri/tests/segment_pipeline_boundary.rs#L123)

- Cổng đếm chỗ gọi sản phẩm cho CẢ HAI tên — gieo một vi phạm thật thì nó đỏ.
  [`segment_pipeline_boundary.rs:171`](../../src-tauri/tests/segment_pipeline_boundary.rs#L171)

- Kiểm chứng dương mới: nổ trên dòng vi phạm dựng tay, không nổ oan trên `use`.
  [`segment_pipeline_boundary.rs:236`](../../src-tauri/tests/segment_pipeline_boundary.rs#L236)

- So mảng từng phần tử — đảo hai bước là đỏ, đo thật 2026-09-04.
  [`segment_pipeline_boundary.rs:147`](../../src-tauri/tests/segment_pipeline_boundary.rs#L147)

- Cổng cũ dời theo mã: đếm chính xác, không nới thành kiểm thành viên.
  [`segment_boundary.rs:324`](../../src-tauri/tests/segment_boundary.rs#L324)

**Đối chứng AD-39 — ca hỏng im lặng, dựng được hôm nay**

- Thứ tự sai trên byte GBK: đúng 1 Chương, KHÔNG lỗi nào — đúng câu spine `:470` mô tả.
  [`segment_contract.rs:7982`](../../src-tauri/tests/segment_contract.rs#L7982)

- Thứ tự đúng: 3 Chương, và khẳng định VĂN BẢN từng Chương chứ không chỉ đếm.
  [`segment_contract.rs:8029`](../../src-tauri/tests/segment_contract.rs#L8029)

- Hình dạng đã chia Chương giữ nguyên số đơn vị vào.
  [`segment_contract.rs:8110`](../../src-tauri/tests/segment_contract.rs#L8110)

- Bước thân rỗng vẫn nói được là đã đi qua — trace ghi từ TRONG mỗi bước.
  [`segment_contract.rs:8229`](../../src-tauri/tests/segment_contract.rs#L8229)

**Ngoại vi**

- Chủ của `encoding_rs` chuyển sang `core::segment`, sửa tại chỗ kèm 🔵.
  [`Cargo.toml:99`](../../src-tauri/Cargo.toml#L99)
