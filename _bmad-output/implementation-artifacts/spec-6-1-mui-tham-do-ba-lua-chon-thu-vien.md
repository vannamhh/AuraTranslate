---
title: 'Story 6.1 — Mũi thăm dò ba lựa chọn thư viện: bóc nội dung, dò bảng mã, HTTP client'
type: 'chore'
created: '2026-09-03'
status: 'review'
review_loop_iteration: 0
baseline_commit: '193ec73d17abebb2e141fc89c5d12c412be7fe62'
context:
  - '{project-root}/AGENTS.md'
  - '{project-root}/src-tauri/AGENTS.md'
  - '{project-root}/_bmad-output/implementation-artifacts/epic-6-context.md'
---

<frozen-after-approval reason="human-owned intent — do not modify unless human renegotiates">

## Intent

**Problem:** Epic 6 dựng 17 story còn lại lên ba giả định **chưa ai đo**: `dom_smoothie` bóc được nội dung chính hay không (A12), `chardetng` + `encoding_rs` dò đúng GBK/Big5 hay không (A13), và `reqwest` có đủ ba năng lực mà `Fetcher` cần hay không — chặn theo chuyển hướng để cưỡng chế allowlist, giới hạn kích thước, timeout. Ba hàng Deferred của spine (`:1038` · `:1039` · `:1040`) mở từ 2026-08-03 và chưa hàng nào đóng; giấy phép mới chỉ đọc **nhãn registry**, thứ mà NFR15 điều 1 nói thẳng là không tính.

**Approach:** Đo trên **dữ liệu thật của Ice** — bài báo `epochtimes.com` cho vế bóc, tệp `.txt` GBK/Big5 Ice tự cấp cho vế bảng mã — bằng một bộ đo `#[ignore]` sống trong `src-tauri/tests/`, kết quả ghi vào `6-1-ban-do/` theo đúng khuôn `5-14-ban-do/`. Rà giấy phép bằng cách **mở tệp trong `~/.cargo/registry/src/…`**, ghi vào bảng Stack **trước** khi sửa `Cargo.toml`, rồi ghim ba crate và đóng ba hàng Deferred.

## Boundaries & Constraints

**Always:**
- **Thứ tự NFR15 là một phần của bản giao, không phải một lời khuyên:** mở tệp giấy phép trong nguồn ĐÃ TẢI → ghi bảng Stack của spine → mới sửa `Cargo.toml`. Ghi lại **đường dẫn tệp đã mở và dòng đầu của nó**, theo khuôn `2-3-hop-dong-flush-va-trang-thai-da-luu.md:753-765`. Đánh dấu `✓` trong cột `Giấy phép` chỉ được đặt khi tệp đã mở tay (quy ước spine `:846-847`).
- Ghim bằng `=` không ngoại lệ (`src-tauri/Cargo.toml:19-22`), và mỗi crate mang một chú thích tiếng Việt nêu **module sở hữu** — đúng cách `reqwest` được khai ở `:52` khi chưa dòng mã nào gọi nó.
- **Số đo phải truy nguyên được:** `6-1-ban-do/environment.txt` mang `baseline_commit` (SHA đầy đủ), `working_diff_sha256`, ngày UTC, máy, và `rustc`/`cargo`/`node` đọc verbatim từ `--version`.
- **Đếm quần thể, đừng ước lượng nó.** `REPORT.md` ghi số mẫu THẬT của từng vế. Vế nào 0 mẫu thì ghi **0 mẫu** và một chủ ở `deferred-work.md` — không suy ra phán quyết từ vế kia.
- Ghi thẳng giới hạn: bản đo chạy trên **trang báo**, không trên truyện. `dom_smoothie` là cổng Readability, vốn tuning cho bài báo — đây là ca THUẬN của nó, nên tỉ lệ đo được **không nói gì** về nguồn khác.
- Bộ đo được commit; thứ một lượt chạy sinh ra thì không (`6-1-ban-do/.gitignore`, đúng doctrine `2-4-ban-do/.gitignore:1-9`). HTML bài báo tải về **không commit** — nội dung có bản quyền.

**Ask First:**
- Tỉ lệ bóc sai hoặc dò sai **cao** ⇒ báo số cho Ice, **không** tự đổi crate và **không** tự đổi kiến trúc. Đường sửa tay của FR123 và dải năm ứng viên của FR126 là dự phòng theo thiết kế.
- Bất kỳ crate thứ tư nào (kể cả một crate bắc cầu cần khai tường minh) ⇒ dừng, rà NFR15, hỏi Ice.

**Never:**
- Không dựng `Fetcher`/`Extractor` thật, không chạm `core/webimport/` ngoài doc-comment, không viết một dòng nào của pipeline nhập. Đó là Story 6.2/6.3/6.9.
- Không đặt mã chạm mạng lên đường sản phẩm (AD-41 — điểm ra mạng thứ ba mở ở Story 6.7, không phải ở đây).
- Không thêm `default = [...]` vào `[features]` (`Cargo.toml:102-105`), không thêm `[[bin]]`/`[[example]]`, không đụng `[profile.release]`.
- Không cấp `AD` mới: AD-39/40/41 đã ghi thẳng rằng chúng **không ràng buộc crate nào**.
- Không tự sinh tệp GBK/Big5 rồi đo — mã hoá bằng `encoding_rs` rồi bảo `chardetng` đọc lại là một vòng tròn, nó không chở BOM lẫn lộn, tệp trộn mã, hay dòng meta ASCII đầu tệp.

## I/O & Edge-Case Matrix

| Scenario | Input / State | Expected Output / Behavior | Error Handling |
|---|---|---|---|
| Bóc, ca thuận | Bài báo đã tải về `fixtures/html/` | Một hàng TSV: id, số ký tự bóc, số đoạn, tiêu đề, 80 ký tự đầu/cuối | N/A |
| Bóc, trang không phải bài | Trang mục lục / trang lỗi | Vẫn ghi một hàng, cột `note` nói rõ hình dạng — không bỏ qua im lặng | Ghi `extract_err` kèm lý do, tiếp mẫu sau |
| Dò bảng mã, có mẫu | `.txt` thật trong `fixtures/encoding/` kèm nhãn thật | Hàng TSV: nhãn thật, `chardetng` đoán, có/không khớp, độ dài | N/A |
| Dò bảng mã, **0 mẫu** | `fixtures/encoding/` rỗng hoặc vắng | Bộ đo **báo 0 mẫu và thoát khác 0** — phân biệt được với "đã đo, tỉ lệ 0%" | Lỗi hạ tầng, không phải một phép kiểm đỏ |
| `reqwest`, chuyển hướng | URL trả 30x sang host khác | Chính sách tuỳ biến **chặn được từng chặng**, ghi cả chuỗi chuyển hướng | Từ chối là kết quả ĐÚNG, ghi vào TSV |
| `reqwest`, thân quá cỡ | Phản hồi vượt trần đã đặt | Cắt theo dòng chảy, không nạp trọn vào bộ nhớ | Ghi `size_capped` |
| Mạng hỏng | Không nối được | Ghi `fetch_err` cho mẫu đó, các mẫu khác chạy tiếp | Không làm trượt cả lượt |

</frozen-after-approval>

## Code Map

- `src-tauri/Cargo.toml:29` `[dependencies]` — chỗ thêm ba crate; `:19-22` khai luật ghim `=`; `:52` là khuôn chú thích "crate đã ghim, chưa dòng mã nào gọi"; `:106-111` `[features]`, `:102-105` cấm `default = [...]`.
- `src-tauri/Cargo.lock:1122-1128` — **`encoding_rs` 0.8.35 ĐÃ có trong cây**, bắc cầu qua `reqwest` 0.13.4 và `quick-xml` 0.41.0 ⇒ khai tường minh thêm **0 byte**. `chardetng` và `dom_smoothie` vắng mặt hẳn khỏi lock.
- `src-tauri/src/core/webimport/mod.rs:1-9` — chín dòng doc-comment, không mã. `:9` đã ghi *"crate cho module này: `reqwest`, dùng chung với `core::ai`"* ⇒ vế HTTP là **xác nhận ba nhu cầu**, không phải chọn lại.
- `src-tauri/src/core/segment/import.rs:26-31` — giao đích danh việc dò bảng mã cho **Story 6.1–6.3**; hôm nay chỉ UTF-8 được nhận, khác thì **từ chối tường minh**. `:41` `SUPPORTED_EXTENSIONS`, `:56` `MAX_IMPORT_BYTES`.
- `…/ARCHITECTURE-SPINE.md:807-842` bảng Stack (34 hàng, cột `| Name | Version | Giấy phép |`); `:846-847` quy ước `✓`/`⚠️`; `:877-886` bảng rà **trước khi thêm** của lượt năm (AD-48) — khuôn cấu trúc gần nhất; `:1035-1057` bảng Deferred (21 hàng), ba hàng phải đóng là `:1038` · `:1039` · `:1040`.
- `_bmad-output/implementation-artifacts/5-14-ban-do/` — khuôn bản giao: `README.md` · `REPORT.md` · `environment.txt` (`key=value` phẳng, sinh bởi `run.sh:445-477`) · `.gitignore` · TSV thô.
- `src-tauri/tests/library_index_contract.rs:3285` · `segment_contract.rs:7759` · `dict_sources.rs:2257` — ba tiền lệ `#[ignore]` cho một phép đo sống trong `tests/`.
- `scripts/check-deps.mjs:172-177,242-243,278-279` — ba crate mới **không** chạm danh sách cấm nào; cổng này **không** canh hình dạng ghim và **không** đối chiếu bảng Stack.
- `_bmad-output/implementation-artifacts/deferred-work.md:2734` — *"cửa rà giấy phép NFR15 không có một cổng máy nào"*, chủ Ice. Cửa ở story này là cửa người.
- `.claude/skills/bmad-architecture/scripts/lint_spine.py` — trình kiểm spine, chạy sau khi sửa.

## Tasks & Acceptance

**Execution:**
- [x] `_bmad-output/implementation-artifacts/6-1-ban-do/` -- dựng thư mục: `README.md` (mục đích, cách chạy, giới hạn), `.gitignore` (`fixtures/`, `*.log`, `scratch/`), `urls.txt` (danh sách bài báo thật, commit được) -- bộ đo commit, dữ liệu chạy thì không
- [x] `src-tauri/tests/webimport_probe.rs` -- tạo mới; ba hàm `#[ignore]` tên là câu khẳng định, ghi TSV vào `6-1-ban-do/` -- ba tiền lệ `#[ignore]` đã có; `tests/**` nằm ngoài bản phát hành nên AD-41 không bị chạm
- [x] `~/.cargo/registry/src/**` -- mở và đọc tệp giấy phép của `dom_smoothie` 0.18.0, `chardetng` 1.0.0, `encoding_rs` 0.8.35; ghi đường dẫn + dòng đầu -- NFR15 điều 1: nhãn registry không tính
- [x] `…/ARCHITECTURE-SPINE.md` -- thêm ba hàng vào bảng Stack (`:842` trở xuống) + một bảng rà trước-khi-thêm theo khuôn `:877-886`; **trước** khi sửa `Cargo.toml` -- NFR15 điều 2, và ba lượt rà đầu của dự án đều là lượt "đuổi theo"
- [x] `src-tauri/Cargo.toml` -- thêm ba `dependency` ghim `=`, mỗi cái một chú thích nêu module sở hữu; ghi thẳng rằng `encoding_rs` thêm 0 byte vì đã bắc cầu -- luật ghim `:19-22`, khuôn chú thích `:52`
- [x] `6-1-ban-do/` -- chạy ba phép đo, sinh `environment.txt` + TSV thô + `REPORT.md` (§Phán quyết dạng bảng, §Phương pháp, §Giới hạn) -- khuôn `5-14-ban-do/REPORT.md`
- [x] `…/ARCHITECTURE-SPINE.md:1038-1040` -- đóng ba hàng Deferred bằng `~~gạch ngang~~` + `✅ Đã đóng 2026-09-03` kèm kết luận; hàng `:1040` đóng bằng **xác nhận** `reqwest`, không bằng một crate mới -- không xoá một hàng đã đóng
- [x] `src-tauri/src/core/webimport/mod.rs` -- nối một dòng 🔵 nêu hai crate vừa ghim và story đã ghim chúng -- mệnh đề hết đúng thì sửa tại chỗ
- [x] `_bmad-output/implementation-artifacts/deferred-work.md` -- ghi mục mới cho mọi vế KHÔNG đo được ở story này, mỗi mục một chủ -- không mục nào mồ côi

**Acceptance Criteria:**
- Given ba crate ứng viên, when rà giấy phép, then bảng rà trong spine ghi **đường dẫn tệp đã mở và dòng đầu** cho từng crate, và mọi giấy phép đều tương thích GPLv3 chiều đi vào.
- Given bảng Stack, when đọc sau story, then nó có **37 hàng** (34 + 3) và ba hàng Deferred `:1038-1040` mang dấu đóng, không hàng nào bị xoá.
- Given `git diff`, when soát thứ tự commit, then lượt sửa spine đứng **trước** lượt sửa `Cargo.toml` trong cùng cây làm việc — thứ tự NFR15 kiểm được, không chỉ khai được.
- Given `REPORT.md`, when đọc phán quyết, then mỗi vế mang **số mẫu thật** đứng cạnh tỉ lệ, và vế nào 0 mẫu thì nói *"chưa đo"* kèm chủ, không nói *"đạt"*.
- Given `cargo tree --locked` trước và sau, when so, then chênh lệch đúng bằng `dom_smoothie` + `chardetng` + cây con của chúng, và `encoding_rs` **không** làm cây dài thêm dòng nào.
- Given `npm run check:deps`, when chạy sau khi thêm, then xanh — và con số cây Rust được ghi lại để sàn `RUST_TREE_FLOOR` còn nghĩa.

## Design Notes

**Vì sao bộ đo sống ở `tests/` chứ không ở một `[[bin]]`.** `src-tauri/Cargo.toml` hôm nay **không có** `[[bin]]`, `[[example]]`, `[dev-dependencies]` — thêm mục đầu tiên của một loại là một quyết định về hình dạng manifest. Test tích hợp thấy được `[dependencies]`, nên `#[ignore]` cho đúng thứ cần mà không thêm gì. Ba tiền lệ đã tồn tại.

**Vì sao không dùng `feature` như Story 5.14.** Bộ đo 5.14 cần một **lệnh chạy trong ứng dụng đang chạy**, nên phải đi qua `[features]` + `#[cfg(...)]`. Ở đây phép đo là hàm thư viện thuần trên tệp — không cần ứng dụng, nên một `feature` chỉ thêm một bề mặt phải canh.

**Chân lý nền cho tỉ lệ bóc sai là một lượt xử của người.** Không có nhãn máy nào nói "đoạn này là nội dung chính". Bộ đo ghi số đo lượng được; `REPORT.md` chở lượt xử tay từng bài (đúng / thiếu / thừa) kèm số bài đã xử. Một tỉ lệ không kèm mẫu số là một tỉ lệ không đọc được.

## Verification

**Commands:**
- `cargo tree --locked --manifest-path src-tauri/Cargo.toml --prefix none --no-dedupe | wc -l` -- chạy TRƯỚC và SAU; ghi cả hai số vào `REPORT.md`
- `cargo test --locked --manifest-path src-tauri/Cargo.toml` -- xanh, và ba phép đo mới **không chạy** (chúng `#[ignore]`)
- `cargo test --locked --manifest-path src-tauri/Cargo.toml --test webimport_probe -- --ignored --nocapture` -- ba phép đo chạy, sinh TSV; vế thiếu mẫu thoát khác 0 kèm câu nói rõ đó là lỗi hạ tầng
- `npm run check:deps` -- xanh
- `uv run .claude/skills/bmad-architecture/scripts/lint_spine.py` -- 0 finding; ghi lại số `AD` và số hàng Stack
- `npm run build && cargo test --locked` -- theo luật `AGENTS.md:32`, build trước cargo test

**Manual checks (if no CLI):**
- Mở từng tệp giấy phép trong `~/.cargo/registry/src/**` và đối chiếu **văn bản** với nhãn — `vitest` từng khai `"MIT"` trong khi `LICENSE.md` dài 811 dòng gộp giấy phép của 27 gói.
- Soát `6-1-ban-do/.gitignore` bằng `git status`: sau một lượt chạy, không HTML bài báo và không TSV-của-lượt-chạy nào hiện ra ở danh sách chờ commit ngoài thứ đã định commit.
