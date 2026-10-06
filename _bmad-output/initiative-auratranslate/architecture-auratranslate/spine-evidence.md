---
name: 'Spine evidence — phần cắt khỏi ARCHITECTURE-SPINE.md'
base_commit: 'd861aab'
---

# Spine evidence

Tệp này giữ nguyên văn lịch sử, số đo, 🔵 và trỏ nợ đã cắt khỏi `ARCHITECTURE-SPINE.md` ở lượt gọn 2026-09-23. Spine là nguồn luật; tệp này là bằng chứng — không đọc như một luật thứ hai. Quyết định phạm vi cắt: `../spine-condense-plan-2026-09-23.md` §6 (không chép lại nội dung §6 ở đây).

## AD-10

L150 (Prevents, nguyên câu, base `d861aab`):

- **Prevents:** FR112 (chính sách gỡ bỏ) biến thành thay đổi mã nguồn + dựng lại toàn bộ payload (nay 343.991.430 byte — NFR6 sửa lần hai 2026-08-05); FR36 chỉ nghiệm thu được bằng suy luận thay vì bằng test thật.

## AD-12

L162 (Prevents, nguyên câu):

- **Prevents:** auto-checkpoint mặc định (1000 trang) rơi đúng lúc người dùng đang gõ → vi phạm NFR2. *(WAL2 mà báo cáo technical research đề xuất **không tồn tại** như tính năng đã phát hành — xem `.memlog.md`.)*

## AD-18

L256–276 (khối blockquote nguyên văn):

  > **Ngữ nghĩa thứ ba — *chỉ toàn cục* — thêm ở Story 1.8, Ice phê chuẩn 2026-08-04.**
  > FR103 đặt phím tắt và preset bố cục ở tầng Global **và không cho chúng đối ứng ở tầng
  > Tác phẩm**; `mockups/settings.html:246` nói thẳng: *"Phím tắt chỉ tồn tại ở tầng Toàn
  > cục — một thao tác không nên đổi phím theo từng Tác phẩm."* Khai chúng bằng một trong
  > hai ngữ nghĩa cũ đều sai, và sai **im lặng**: *ghi đè* mở một tầng Tác phẩm mà UX đã
  > cấm, nên Story 1.14/1.21 sẽ dựng thanh chuyển phạm vi cho một thứ không nên có; *hợp
  > nhất* thì vô nghĩa. Một tầng Tác phẩm cho các loại này trả lỗi, không bị bỏ qua im
  > lặng — bỏ qua im lặng là cách một tầng bị cấm vẫn được ghi xuống đĩa rồi không bao giờ
  > có tác dụng.
  >
  > **Cấu hình AI ghi đè theo từng *trường*, không theo cả struct** *(làm rõ ở Story 1.8,
  > cùng lượt ký)*. Bảng này trước đây chỉ ghi *"ghi đè"*, và Story 4.2 cũng chỉ nói *"ghi
  > đè được theo Tác phẩm đó"*. Chỉ `mockups/settings.html` lộ ra rằng trong **cùng một**
  > cấu hình có trường `ghi đè` và trường `kế thừa` cùng lúc (`:172`, `:188`, `:200`) — tức
  > Epic 4 phân giải nó như một map `khoá trường → giá trị`, y hệt Glossary.
  >
  > **`ngôn ngữ nguồn` KHÔNG phải một loại ở bảng này.** FR103 liệt kê nó ở tầng Tác
  > phẩm, nhưng Story 5.1 định nghĩa nó là trường **bất biến** trong `meta.json` — đặt lúc
  > tạo, không đổi được (`prd.md:765-774`: *"cố định, đặt lúc tạo"*, mệnh đề mà
  > `epics.md:296` làm rơi mất) — và nó **không có đối ứng ở tầng Global**, nên không có gì
  > để ghi đè. Nó là thuộc tính của `Work`, không phải cấu hình hai tầng.

## AD-25

L331 (Prevents, nguyên câu):

- **Prevents:** nguồn thô 1,13 GB hoặc `.db` (nay 320,3 MB cho cả ba tệp) lọt vào git; đồng thời giữ FR107 (build công khai, kiểm chứng được).

## AD-26

L338 (Rule, nguyên câu):

- **Rule:** tra chính xác đầu mục → chỉ mục B-tree (đường nóng Auto-Lookup). Chuỗi con 1–2 ký tự → bảng đảo ngược `char_idx`. Chuỗi con 3+ ký tự → FTS5 `trigram`. `LIKE` **cấm** trên đường nóng (đo được 20–50 ms).

L340 (đoạn 🔴, nguyên văn):

  🔴 **Phạm vi là TIẾNG TRUNG, và mệnh đề này thuộc Rule chứ không chỉ thuộc tiêu đề.** Cả ba nhánh là cơ chế cho chữ Hán; nhánh `char_idx` **không** áp được cho tiếng Anh *(đo: **9** cặp trên **119.039** đầu mục)*. Đường tiếng Anh và vị từ điều phối giữa hai đường: **AD-44**.

L342 (đoạn ⚠️, nguyên văn — toàn bộ):

  ⚠️ **Dải hiệu năng mà bản đầu của AD này công bố — 0,02 ms · 0,15–4,5 ms · 0,13–0,19 ms — nay LỖI THỜI, đừng trích tiếp như số hiện hành.** Số đó đo ở Giai đoạn 0 trên một database **ba** nguồn. Đo lại 2026-08-05 trên `dict-core.db` **sáu** nguồn, bản release, p95: nhánh 1 **0,083 ms** · nhánh 2 một ký tự **7,324 ms** · nhánh 2 hai ký tự **1,039 ms** · nhánh 3 **0,448 ms**. Nhánh 2 với truy vấn **một ký tự** vượt **1,6×** cận trên của dải cũ — chi phí nằm ở **số hàng** *(`char_idx` của `山` đi từ ~2.576 lên **3.177**)*, không ở chỉ mục, nên nó **không** sửa được bằng một chỉ mục mới. Mỗi nguồn từ điển thêm vào sẽ làm nó dày thêm.

## AD-39

L494 (blockquote nguyên văn):

  > **Đường song ngữ (FR115) rơi vào hàng trên theo đúng tiêu chí hình dạng** — một file `.docx` hai cột chứa cả bộ truyện cũng đến thành một dòng chưa chia Chương. **Mẫu phân tách áp lên cột nguồn** *(PRD chốt 2026-08-03)*: đầu Chương ở cột gốc mang dạng ổn định máy khớp được, còn cột đích do người khác dịch nên có thể ghi khác đi hoặc bỏ hẳn dòng tiêu đề.

## AD-44

L575 (mục ① của Prevents, nguyên câu):

  1. **AD-26 bị đọc như thể áp cho mọi ngôn ngữ.** Tiêu đề nó nói *"tiếng Trung"*, thân Rule thì không — nên một giai đoạn sẽ cho tiếng Anh đi qua `char_idx`. Đo thật: lớp tiếng Anh sinh **9** cặp `char_idx` trên **119.039** đầu mục *(0,0076%)*. Truy vấn 1–2 ký tự trả rỗng trong 0,01 ms, không lỗi nào được ném — **đúng lớp lỗi AD-26 ra đời để chặn, tái sinh ở ngôn ngữ khác**.

L608 (đoạn 🔴, nguyên câu):

  🔴 Hạ chữ thường là **THÊM** một khoá, **không phải THAY** khoá gốc — **1.635** đầu mục tiếng Anh mang chữ hoa có nghĩa (`API` · `Wikipedia` · `English`), và **184** nhóm đầu mục chỉ phân biệt nhau bằng chữ hoa. Phép hạ chữ thường **không phụ thuộc locale** *(`I` luôn ra `i`)*: một phép fold theo locale làm cùng một truy vấn cho hai kết quả trên hai máy cài ngôn ngữ hệ điều hành khác nhau.

L610, L612, L614, L616 (khối "Dữ kiện MẠNH" / "Dữ kiện YẾU HƠN" / blockquote ⚠️, nguyên văn liền mạch — giữ câu mở "🔴 Stemming KHÔNG nằm..." để không cụt ngữ cảnh):

  🔴 **Stemming KHÔNG nằm trên đường nóng tra từ điển** — ghi ra để một giai đoạn sau không "vá" nó vào. Hai dữ kiện, và chúng **không cùng độ chắc** — đừng trích cái yếu như cái mạnh:

  **Dữ kiện MẠNH, đo trên `dict-core.db` thật — đây là thứ quyết định đứng lên:** corpus đã có sẵn mọi dạng biến thể làm **đầu mục riêng**. Mẫu thử **16/16** có mặt, **gồm cả bất quy tắc** `went` · `gone` · `children` · `happiest` — thứ stemming về nguyên tắc **không bao giờ** làm được. Quy mô: **7.656** đầu mục `-ing` · **8.855** `-ed` · **19.616** `-s` · **228** `-est` trên **119.039**. ⇒ Nhánh tra chính xác một mình đã phủ FR40 **rộng hơn** thứ stemming phủ được.

  **Dữ kiện YẾU HƠN, corroborating:** đầu ra của một stemmer là một *stem*, không phải một *lemma*, nên nó không cần là một đầu mục. Ba dạng stem Porter kinh điển tra vào `dict-core.db` cho **0** hàng — `dictionari` · `studi` · `happi` — trong khi `run` cho **1**.

  > ⚠️ **Chỗ yếu nói thẳng:** *số hàng* ở trên là **đo thật**, nhưng *ba chuỗi stem đó* lấy từ hành vi kinh điển của Porter chứ **chưa chạy qua stemmer mà sản phẩm sẽ dùng**. Chuỗi thật có thể khác *(`tantivy-stemmers` phơi nhiều biến thể; Porter và Porter2/English không cho cùng kết quả)*. Quyết định **không** đứng trên dữ kiện này — nó đứng trên dữ kiện mạnh — nhưng ai muốn **mở lại** câu hỏi stemming thì việc đầu tiên là chạy stemmer thật và thay bảng này bằng số đo, đừng chép tiếp.

## AD-45

L638 (Prevents, nguyên câu):

- **Prevents:** AD-15 đếm **điểm RA** mạng và không nói một chữ nào về chiều ngược lại. Nên một máy chủ nghe trên `localhost` đi thẳng vào bản người dùng cài mà **không phạm AD-15** — một bề mặt mới không có luật, đúng lớp lỗi kho này tồn tại để săn. Bộ lái e2e (Ice chốt 2026-08-11) là ca đầu tiên: `tauri-plugin-wdio-webdriver` kéo `axum` + `tokio` và mở cổng **4445**.

L646 (đoạn "Cưỡng chế bằng lệnh", nguyên câu):

  **Cưỡng chế bằng lệnh, không bằng kỷ luật:** `scripts/check-deps.mjs` **Kiểm 1b** khẳng định `tauri-plugin-wdio-webdriver` và `axum` **vắng mặt** khỏi `cargo tree` của bộ feature mặc định. Số đo 2026-08-11: cây mặc định **831** dòng · `--features wdio` **948**.

L648 (đoạn ⚠️, nguyên câu):

  ⚠️ Danh sách canh **không** gồm `tokio` — đo được nó đã nằm trong cây mặc định từ trước qua `tauri` (`tokio 1.53.1`). Canh nó là dựng một cổng đỏ oan ngay ở lượt chạy đầu.

## AD-46

L657 (đoạn ⚠️, nguyên văn — toàn bộ):

  ⚠️ Ghi ra vì nó đã suýt xảy ra: bản ghi phiên thiết kế 2026-08-14 kết luận *"không phá AD-37, AD-37 nói về cấu trúc đoạn của NGUYÊN VĂN"*. Cây nguồn nói ngược — `AD-37 §Rule` khai bằng chữ *"dùng chung cho cả nguyên văn và bản dịch"*, và `epics.md` chép lại y hệt vào bảng bất biến. Lượt rà correct-course bắt được chỗ này; nếu không, một `ALTER TABLE` sẽ đi qua **mọi cổng** rồi làm FR121 hỏng ở Epic 8.

## AD-47

L678 (Prevents, nguyên câu — chứa trích `prd.md:452`):

- **Prevents:** AD-31 phân xử xuất xứ bằng cách so văn bản đích hiện tại với **bản lúc nạp segment**. Mốc đó là một **thế thân** cho câu hỏi thật — *"người dùng có gõ chữ này không"* (`prd.md:452`) — và nó chỉ đúng chừng nào **lượt nạp là cơ chế duy nhất** đặt văn bản vào một segment. Ba cơ chế đã có đặc tả phá thế thân đó, mỗi cái ghi `target_text` mà người dùng **không gõ một ký tự nào**:

L680–684 (bảng nguyên gốc, đủ cột — cột "Cơ chế" mang tên Epic bị cắt trong spine sau lượt gọn):

  | Cơ chế | Đọc AD-31 theo đúng chữ hôm nay |
  |---|---|
  | Chấp nhận thay đổi từ Review Mode (FR94, Epic 8) | khác bản lúc nạp ⇒ **tôi dịch** cho chữ của reviewer |
  | Điền sẵn từ TM khớp 100% (FR58, Epic 7) | khác bản lúc nạp ⇒ **tôi dịch** cho chữ lấy từ kho |
  | Đưa đề xuất AI sang Editor (Epic 4) | khác bản lúc nạp ⇒ **tôi dịch** cho chữ của máy |

L694 (① Định nghĩa mốc, nguyên câu — chứa trích `commands/segment.rs:1737-1741`):

  **① Định nghĩa mốc.** Một **lượt ghi không-phải-người-dùng** là lượt ghi `target_text` mà văn bản **không đến từ bộ đệm gõ của Editor**. Flush theo AD-35 **không** thuộc loại này — nó chở đúng bộ đệm gõ, và một mốc chạy theo từng lượt flush phá đúng ca *gõ rồi hoàn tác* (đo ở Quyết định #2 của Story 2.7: `commands/segment.rs:1737-1741` so với **đĩa tại lượt flush**, nên `AB` → `A` bật cờ dù văn bản cuối y nguyên).

L705–713 (bảng ③ nguyên gốc, đủ cột — cột "Chủ" bị cắt trong spine sau lượt gọn):

  | Lượt ghi không-phải-người-dùng | Xuất xứ nó đặt | Chủ |
  |---|---|---|
  | Nạp Chương từ đĩa | giá trị đang có, **không ghi lại** | Story 2.7 |
  | Nhập song ngữ (FR115) | **nhập từ tài liệu song ngữ** | Epic 6 |
  | Chấp nhận thay đổi từ Review Mode (FR94) | **người khác dịch** | Epic 8 |
  | Điền sẵn từ TM khớp 100% (FR58) | xuất xứ của **cặp TM nguồn** | Story 7.4 |
  | Đưa đề xuất AI sang Editor | **người khác dịch** | Epic 4 |
  | Gộp/tách segment (AD-5) | xem ④ | Story 2.8 |
  | Khôi phục phiên bản (FR101) | 🔴 **KHÔNG đặt** — ngoại lệ có tên, xem ⑤ | Story 2.6 |

L719 (⑤ Khôi phục, nguyên câu — chứa mốc "ngày 2026-08-16"):

  **⑤ Khôi phục (FR101) làm (a) mà KHÔNG làm (b).** Đây là **hệ quả bắt buộc** của chữ ký #1(a) ngày 2026-08-16: `segment_version` không mang xuất xứ, nên **không có gì để trả về**. `replaceEditorSegment` vốn đã định nghĩa lại mốc giữa phiên — ⑤ chỉ khai điều đó bằng chữ.

L721 (đoạn ⚠️, nguyên văn — toàn bộ, chứa trích `deferred-work.md:3685-3697`):

  ⚠️ **Chỗ yếu, ghi ra thay vì để người sau tự phát hiện:** khôi phục văn bản của một phiên bản cũ rồi xác nhận mà không sửa ⇒ giữ nguyên xuất xứ **hiện tại**, thứ có thể thuộc về một phiên bản khác. Món nợ này **cùng gốc** với món nợ bốn nhãn của Story 2.6 (`deferred-work.md:3685-3697`) và đóng cùng lúc với nó — chủ: story nào cho `segment_version` một cột xuất xứ. Nó **không** được tự chấm đạt ở Story 2.7.

L734 (đoạn ⚠️, nguyên câu — chứa trích `epics.md:5355`):

  ⚠️ **Cái mất của quyết định giữ ba giá trị, ghi ra:** FR62 lọc TM **không phân biệt được** *từ AI* với *từ reviewer* với *từ người dịch trước* — cả ba là *người khác dịch*. FR62 khai mục đích của bộ lọc là *"rà lại hoặc dọn sạch phần không phải văn phong của mình"* (`epics.md:5355`), tức đúng trục nhị phân, nên nó **không hụt gì**. Ai muốn phân biệt ba nguồn đó phải mở một `AD` mới **và** một bước di trú.

L738 (bullet AD-31 §bảng máy trạng thái, nguyên câu — chứa "Story 2.5 đã cài đúng nó, 372 ca Rust canh"):

  - **AD-31 §bảng máy trạng thái** (sáu hàng) — **không sửa một chữ**. Story 2.5 đã cài đúng nó, 372 ca Rust canh.

L742 (bullet AD-18/AD-14/AD-6, nguyên câu — chứa mốc "từ 2026-08-02"):

  - **AD-18, AD-14, AD-6** — không sửa. ⑥ chỉ khai bằng chữ phép chiếu mà AD-18 §thứ tự hai khoá **đã giả định** từ 2026-08-02.

L744 (⑧ Story 7.4, nguyên câu — chứa trích `epics.md:5169-5170`):

  **⑧ Story 7.4 hết là một giả định.** AC *"xác nhận một segment điền sẵn từ TM mà không sửa ⇒ giữ nguyên xuất xứ của cặp TM nguồn"* (`epics.md:5169-5170`) thêm ở Epic 7 **trước khi** có `AD` nào chốt hình dạng. Nó nay là **hệ quả** của ③ cộng ②, không phải một luật rời.

## AD-48

L751 (① Lớp hỏng của người dùng, nguyên câu — chứa "Story 3.10 dựng trọn nửa định dạng CSV/TSV..."):

  **① Lớp hỏng của người dùng.** Trước `AD` này, kho **không có đường nào** cho người dùng chọn một tệp để ghi ra. Story 3.10 dựng trọn nửa định dạng CSV/TSV rồi dừng ở đúng chỗ đó — mã chạy được, nghiệm thu được bằng `cargo test`, và **không lối vào nào**. FR49/NFR9 hứa *"chia sẻ không cần server hay tài khoản"*; lời hứa đó không thực hiện được bằng một hàm không ai gọi tới.

L771 (đoạn ⚠️ vị từ `check-deps.mjs` Kiểm 1, nguyên văn — toàn bộ, chứa mốc "Ice chốt 2026-08-25"):

- ⚠️ **Vị từ của `check-deps.mjs` Kiểm 1 rộng hơn lý do nó tự khai — sửa LÝ DO, giữ VỊ TỪ** *(Ice chốt 2026-08-25)*. Cổng đó canh *"tên có mặt trong `cargo tree`"*, trong khi chú thích khai lý do là *"plugin tồn tại để phơi API ra JavaScript"*. Hai mệnh đề đó **không trùng nhau** — §Rule ③ ngay trên là bằng chứng: `tauri-plugin-fs` ở trong cây mà **không** phơi một lệnh nào. Vị từ **ở lại** vì nó cưỡng chế được bằng máy; chú thích phải nói thật rằng nó canh **mã trong nhị phân** (NFR6 + bề mặt tấn công), còn **bề mặt IPC** (NFR11) do `config_invariants.rs` canh. Hai cổng, hai mệnh đề, không cái nào thay được cái kia.

## Stack

### Bảng chính — 11 hàng rút gọn, nguyên văn trước khi rút (đủ ba cột)

| Name | Version | Giấy phép |
|---|---|---|
| `keyring` *(provenance sửa lại — Rà NFR15 lượt chín, 2026-09-17, Story 4.3; xem ngay dưới)* | 4.1.6 | MIT OR Apache-2.0 ✓ |
| `keyring-core` *(`[dev-dependencies]` — Story 4.3, cài `keyring_core::mock::Store` cho test; production không khai tên nó, xem `Cargo.toml`)* | 1.0.0 | MIT OR Apache-2.0 ✓ |
| `reqwest` *(feature `blocking` bật ở Story 6.1 cho bàn đo — 0 crate mới)* | **0.13.4** | MIT OR Apache-2.0 ✓ |
| `dom_smoothie` *(core::webimport::Extractor — Story 6.1)* | 0.18.0 | MIT ✓ |
| `chardetng` *(core::webimport — dò bảng mã, Story 6.1)* | 1.0.0 | Apache-2.0 OR MIT ✓ |
| `encoding_rs` *(core::webimport — giải mã theo bảng mã đã dò, Story 6.1; đã bắc cầu qua `reqwest`/`quick-xml` trước story này, khai tường minh thêm 0 byte)* | 0.8.35 | (Apache-2.0 OR MIT) AND BSD-3-Clause ✓ |
| `regex` *(core::cleanup — luật làm sạch dạng biểu thức chính quy, Story 6.5; đã có sẵn trong `Cargo.lock` từ trước — bắc cầu qua `jieba-rs`/`tantivy-stemmers` — khai tường minh thêm 0 byte)* | =1.13.1 | MIT OR Apache-2.0 ✓ |
| `dom_query` *(core::webimport::Extractor — Story 6.9, nâng từ bắc cầu của `dom_smoothie` (dòng `:909`) thành phụ thuộc TRỰC TIẾP: `extractor.rs` cần duyệt lại HTML GỐC bằng CSS selector để dựng mô hình khối giữ/loại — `Article::content` của `dom_smoothie` không phơi ra đủ, xem §Design Notes spec 6.9. Đã có sẵn trong `Cargo.lock` từ Story 6.1, LICENSE đã mở đọc trong nguồn đã tải (`~/.cargo/registry/src/…/dom_query-0.28.0/LICENSE`) — 0 gói MỚI vào `Cargo.lock`, chỉ đổi từ bắc cầu sang khai tường minh)* | =0.28.0 | MIT ✓ |
| `zip` *(core::docx — Story 6.12, đọc kho zip của `.docx`. Đã có sẵn trong `Cargo.lock` từ trước qua `docx-rs` (bắc cầu, `cargo tree -i zip` xác nhận 2026-09-09) — khai tường minh ở đây thêm 0 gói MỚI, chỉ đổi từ bắc cầu sang trực tiếp. LICENSE (MIT) đã mở đọc trong nguồn đã tải, xem Rà NFR15 lượt tám ngay dưới)* | =8.6.0 | MIT ✓ |
| `quick-xml` *(core::docx — Story 6.12, phân tích `word/document.xml`/rels. `Cargo.lock` mang HAI phiên bản của crate này (0.38.4 qua `tauri`→`plist`, 0.41.0 qua `docx-rs`) — khai đúng **0.41.0** để không thêm một phiên bản thứ ba; 0 gói MỚI, LICENSE (MIT) đã mở đọc trong nguồn đã tải, xem Rà NFR15 lượt tám ngay dưới)* | =0.41.0 | MIT ✓ |
| `vitest` *(bộ chạy test frontend — Story 2.3)* | 4.1.10 | MIT ✓ |

### Prose dưới bảng — đoạn cắt nguyên khối, nguyên văn

L874–883 (khối "Ba hàng ⚠️ và một hàng suýt bị chấm sai" + bảng + câu Vite `LICENSE.md`):

Ba hàng **⚠️** và một hàng suýt bị chấm sai, đọc thẳng từ tệp:

| Hàng | Điều thật sự tìm thấy | Kết luận |
|---|---|---|
| `jieba-rs` 0.10.3 | Bản `.crate` **không kèm tệp `LICENSE`** — `Cargo.toml` khai `license.workspace = true` và tệp thật nằm ở gốc workspace, không được đóng gói. `README.md` **trong chính nguồn đã tải** ghi nguyên văn *"This work is released under the MIT license."* | MIT — xác nhận bằng văn bản trong nguồn, không bằng nhãn |
| `docx-rs` 0.4.22 | **Không tệp `LICENSE`, README không có mục giấy phép.** Bằng chứng duy nhất: `license = "MIT"` trong `Cargo.toml.orig` + một dòng header `// MIT license (LICENSE-MIT or …)` trong `src/xml_json/mod.rs` | MIT — **bằng chứng yếu nhất trong cả bảng**. Nếu Giai đoạn 5 đổi sang `docx-reader`/`rdocx` (hàng Deferred đã nêu), rà lại từ đầu |
| `dockview-vue` 7.0.4 | Gói npm **không kèm tệp giấy phép**; `package.json` khai `MIT`. Gói chạy kèm `dockview-core` 7.0.4 mang banner `@license MIT` **nhúng trong chính bundle đã phát hành** | MIT |
| `tantivy-stemmers` 0.4.0 | **Suýt chấm sai**: tệp `LICENSE` dùng gạch đầu dòng thay vì điều khoản đánh số, nên bộ nhận dạng tự động đọc thành BSD-2. Đọc bằng mắt thì mệnh đề *"Neither the name … may be used to endorse"* **có mặt** → đúng **BSD-3-Clause**. *(Tệp còn để sót placeholder `{{ project }}` chưa thay — lỗi hình thức của thượng nguồn, không đổi bản chất giấy phép.)* | BSD-3-Clause, nhãn đúng |

> `LICENSE.md` của Vite gộp giấy phép của các gói nó vendor vào (MIT + BSD-2-Clause + ISC) — đó là hình dạng bình thường của một bundler, bản thân Vite là MIT.

L885 (lượt ba correct-course, nguyên văn — toàn bộ):

**Rà NFR15 lượt ba — 2026-08-11, lượt correct-course**, cùng phương pháp hai lượt trước: **mở tệp `LICENSE` trong nguồn ĐÃ TẢI mà đọc**, không tin nhãn của registry. Mười hàng mới ở trên đều mang **✓**: cả mười tệp có mặt và thân tệp mang đúng mệnh đề *"Permission is hereby granted, free of charge"* của MIT. Bảy hàng trong số đó **sinh ra rồi mới được ghi vào bảng** — `uuid` từ Story 1.15, ba hàng ESLint từ cổng thứ mười, năm gói WebdriverIO cùng plugin từ bộ lái e2e — nên lượt này là một lượt **đuổi theo**, không phải một lượt rà trước khi thêm. Đó chính là quy ước ở hàng *Giấy phép* của bảng Consistency Conventions, và nó đã bị bỏ lỡ ba lần liên tiếp.

L891 (nguyên văn — toàn bộ):

Khác ba lượt trước ở đúng một chỗ, và đó là chỗ đáng ghi: đây là lượt rà **TRƯỚC khi thêm**, không phải một lượt đuổi theo. Cả ba tệp giấy phép được mở trong `node_modules/` **sau lượt `npm install` và trước dòng mã test đầu tiên**; đường dẫn và dòng đầu ghi ở §Completion Notes của story `2-3-hop-dong-flush-va-trang-thai-da-luu.md`.

L893 (đoạn ⚠️ `vitest`, nguyên văn — toàn bộ):

⚠️ **`vitest` mang một bảng giấy phép GỘP, và đó là lý do đọc tệp thật hơn đọc trường `license`.** `node_modules/vitest/LICENSE.md` dài **811** dòng: phần đầu là MIT của chính Vitest, phần sau khai giấy phép của **27** gói nó vendor — **24 MIT · 2 BSD-3-Clause · 1 ISC**. Trường `license` trong `package.json` của gói chỉ nói `"MIT"` và không nói một chữ nào về 27 gói kia. Cả ba nhóm đều thuộc nhóm dễ dãi và tương thích GPL v3 theo chiều đi vào.

L895 (đoạn ⚠️ cây npm 530→656, nguyên văn — toàn bộ):

⚠️ **Cây npm đi từ 530 lên 656 gói** (số `npm ls --all` đếm được, gồm cả node trùng tên ở nhiều độ sâu; số **gói đã cài** mà `check-deps.mjs` đếm là **522**). Lượt cài này làm lộ ra một **khuyết tật của chính cổng phụ thuộc**, đã vá cùng lượt: `vitest` khai `@opentelemetry/api` làm **peer tuỳ chọn chưa cài**, và `npm ls --all --json` xếp một node **rỗng** cho nó vào `dependencies` — bản trước của `check-deps.mjs` đếm node đó là thành viên cây rồi báo *"cây npm có thư viện thu thập dữ liệu"*, trong khi **không một byte** của gói đó có trên đĩa. Nay cổng chỉ đếm node **có `version`**, và in ra số node chỉ-lời-khai đã bỏ (**82**). Xem `scripts/check-deps.mjs` §④.

L899 (đoạn 🔵 "con số CHÍN sai; thật ra là BA", nguyên văn — toàn bộ, TRÙNG với AD-48 L765, giữ một bản duy nhất ở AD-48 trong spine):

🔵 **2026-08-25 (muộn hơn, Story 3.10b) — con số CHÍN sai; thật ra là BA.** Đo trên cây sau lượt thêm thật: `Cargo.lock` nhận đúng **`tauri-plugin-dialog`** · **`tauri-plugin-fs`** · **`rfd`** (cộng mười mục `windows-*` chỉ dành cho đích Windows, không biên dịch trên macOS). **Sáu** tên còn lại — `notify` · `notify-debouncer-full` · `notify-types` · `flume` · `file-id` · `fsevent-sys` — **chưa bao giờ vào cây**, và `grep` trong `Cargo.lock` cho **0** cho cả sáu. Nguyên nhân đọc thẳng từ nguồn đã tải: `tauri-plugin-fs-2.5.1/Cargo.toml:51-54` gate `notify` + `notify-debouncer-full` sau feature **`watch`**, mà `watch` **không** nằm trong bộ mặc định và `tauri-plugin-dialog` không bật nó. Sáu hàng ấy **ở lại bảng** — lượt rà giấy phép đã chạy thật và là bằng chứng cho quyết định kế tiếp nếu một ngày kho cần feature `watch` — nhưng chúng được đánh dấu **KHÔNG TRONG CÂY** để không ai đọc bảng này thành một bản kê cây phụ thuộc.

L915 (đoạn ⚠️ `trash`, nguyên văn — toàn bộ):

⚠️ **`trash` KHÔNG vào cây phát hành, và bản ghi đầu tiên nói sai chỗ này.** Hồ sơ bàn giao `ad-brief-2026-08-24-hop-thoai-chon-tep.md` §3.2 liệt `trash` vào danh sách crate mới. Đo lại: `notify-8.2.0/Cargo.toml:93` khai nó là **`dev-dependencies`, chỉ cho Windows** — nó không bao giờ chạm nhị phân phát hành. Sửa tại chỗ thay vì để con số truyền tiếp.

L917 (đoạn 🔵 số byte payload đã đo, nguyên văn — toàn bộ, TRÙNG với AD-48 L765's "156.392 byte", giữ một bản duy nhất ở AD-48):

🔵 **2026-08-25 (Story 3.10b) — số byte payload ĐÃ ĐO; mệnh đề *"chưa ai đo"* hết đúng.** Hai bản dựng `--release` chỉ khác nhau **đúng một phụ thuộc**, cùng một `dist/` không dựng lại giữa hai lượt (Tauri nhúng `dist/` vào nhị phân, nên đo ở cuối story sẽ gộp cả phần frontend mới vào): baseline ở `ce5d276` = **7.555.496 byte** → sau khi thêm = **7.711.888 byte**. **Delta = 156.392 byte (≈152,7 KiB)**, tức **15%** ngưỡng xét lại 1 MB — không kích điều kiện quay lui `rfd` thẳng. macOS, `rustc 1.97.1 (8bab26f4f)` · `cargo 1.97.1`. ⚠️ Dư địa NFR6 tổng (**3.104.634 byte**) vẫn thuộc **Story 10.1**; delta này ăn 5% dư địa đó.

L919 (chỉ câu mở đầu bị cắt, nguyên văn):

**Rà NFR15 lượt sáu — 2026-09-03, Story 6.1 (mũi thăm dò ba lựa chọn thư viện đường nhập).**

L941 (chỉ câu mở đầu bị cắt, nguyên văn):

**Rà NFR15 lượt bảy — 2026-09-05, Story 6.5 (luật làm sạch dạng regex, FR124).**

L943–945 (bảng chi tiết `regex`, nguyên văn — trùng hàng chính đã có ở bảng Stack):

| Crate | Trường `license` | Tệp đã mở | Dòng đầu |
|---|---|---|---|
| `regex` 1.13.1 | `MIT OR Apache-2.0` | `LICENSE-MIT` · `LICENSE-APACHE` | `Copyright (c) 2014 The Rust Project Developers` (MIT) · `Apache License` (APACHE) |

L949 (chỉ câu mở đầu bị cắt, nguyên văn):

**Rà NFR15 lượt tám — 2026-09-09, Story 6.12 (đọc `.docx`, AD-38).**

L951–954 (bảng chi tiết `zip`/`quick-xml`, nguyên văn — trùng hai hàng chính đã có ở bảng Stack):

| Crate | Trường `license` | Tệp đã mở | Dòng đầu |
|---|---|---|---|
| `zip` 8.6.0 | `MIT` | `LICENSE` | `The MIT License (MIT)` |
| `quick-xml` 0.41.0 | `MIT` | `LICENSE-MIT.md` | `The MIT License (MIT)` |

L996 (đoạn 🔵 "rời danh sách này (AD-48)", nguyên văn — toàn bộ):

🔵 **2026-08-25 — `tauri-plugin-dialog` và `tauri-plugin-fs` RỜI danh sách này (AD-48).** Hai tên đó nằm đây từ 2026-08-03 với lý do *"plugin tồn tại để phơi API ra JavaScript"*. Lý do ấy **vẫn đúng** — thứ đổi là kho nay có một luật nói *cách dùng* chứ không chỉ *có hay không*: AD-48 khoá hộp thoại vào API phía Rust và giữ `capabilities/main.json` ở đúng ba quyền, nên plugin vào cây mà **không** một lệnh nào ra JavaScript. Ba tên còn lại giữ nguyên hiệu lực, và **`tauri-plugin-fs` không được `init()`** — có mặt trong cây khác hẳn có mặt trên dây (AD-48 §Rule ③).

### Prose dưới bảng — cắt một phần, phần CẮT chép nguyên văn (phần GIỮ ở lại trong spine, không chép ở đây)

L868 (bốn mảnh cắt, theo thứ tự xuất hiện):

Ba họ font rà theo NFR15 ngày 2026-08-03 ở Story 1.1, bằng cách **mở tệp `LICENSE` trong bản release đã tải mà đọc** chứ không tin nhãn của GitHub.

Phiên bản và tên họ ghi ở đây đọc từ bảng `name` của chính tệp: `Source Serif 4` trên kênh Google là **4.004**, đi sau bản Adobe 4.005R một phát hành.

*(Tên họ lấy ở **name ID 16** với `Source Sans 3` — ID 1 của tệp đó là `Source Sans 3 ExtraLight` vì nó là font biến thiên có mặc định trục `wght = 200`; hai tệp kia lấy ở **ID 1**.)*

Số đo và lý do đầy đủ: [`research/font-spike-results-2026-08-03.md`](../../research/font-spike-results-2026-08-03.md).

L870 (ba mảnh cắt, theo thứ tự xuất hiện):

**Rà NFR15 lượt hai — 2026-08-03, Story 1.2**, cùng phương pháp Story 1.1: **mở tệp giấy phép trong nguồn ĐÃ TẢI mà đọc** (`~/.cargo/registry/src/…`, `node_modules/…`), không tin nhãn của registry.

(**16/19** hàng phần mềm)

(**3/19**: `dockview-vue` · `jieba-rs` · `docx-rs`)

L872 (mảnh mở đầu bị cắt, câu cuối "Mọi giấy phép trong bảng..." vẫn ở lại spine):

> **Bốn hàng phải phân xử bằng mắt ≠ ba hàng mang ⚠️.** Bốn hàng cần đọc kỹ là `tantivy-stemmers` · `jieba-rs` · `docx-rs` · `dockview-vue`; trong đó **`tantivy-stemmers` phân xử xong thành ✓** (bộ nhận dạng tự động đọc nhầm ra BSD-2 vì tệp dùng gạch đầu dòng thay vì điều khoản đánh số, nhưng mệnh đề *"Neither the name … may be used to endorse"* có mặt → đúng BSD-3-Clause). Ba hàng còn lại giữ ⚠️. Sửa 2026-08-03 sau rà soát mã: prose cũ ghi 15/19 trong khi bảng đếm được 16 ✓.

L887:

Cây npm đi từ **194** lên **530** gói khi bộ lái e2e vào.

L998 (ba mảnh cắt, theo thứ tự xuất hiện; câu "**Bốn** tên `tauri-*` trên được **cưỡng chế bằng lệnh**..." vẫn ở lại spine):

(Story 1.2)

Story 1.3 gắn script này vào CI.

🔵 *(2026-08-25: sáu → bốn, AD-48.)*

L1002 (mảnh cắt):

từ 2026-08-11

## Deferred

L1135–1154 (toàn bộ ô giữa của hàng "Thư viện bóc nội dung chính" FR123 — đóng 2026-09-03 Story 6.1 — cộng đoạn "Đo bổ sung 2026-09-03" và đoạn 🔵 "SỬA 2026-09-06 (Story 6.7)", kể cả hai bảng byte nhị phân; nội dung này trùng với mục "Chi phí byte NFR6 THẬT của `dom_smoothie`" đã có ở `deferred-work.md`, xác nhận bằng chuỗi `→ ✅ ĐÃ ĐÓNG 2026-09-06` trong chính đoạn cuối):

| ~~**Thư viện bóc nội dung chính** (FR123)~~ | ✅ **Đã đóng 2026-09-03 (Story 6.1)** — `dom_smoothie` 0.18.0 GHIM. Đo trên **7 mẫu thật** (`6-1-ban-do/extraction-raw.tsv`): 6 bài báo epochtimes.com + 1 trang không phải bài (trang chủ ấn bản Phồn thể). Cả 7 fetch/extract không lỗi. 6/6 bài: tiêu đề đúng, 80 ký tự đầu khớp đúng phần mở bài (không lẫn menu/quảng cáo); đối chiếu tay với vùng `#post_content` của chính HTML cho độ đầy đủ nội dung **72%–99%** (dom_smoothie luôn hụt một chút ở cuối bài, không bao giờ lẫn rác). Cờ `is_probably_readable`: đúng ✅ cho trang không phải bài (false) và cho 5/6 bài (true); **1 bài thật bị chấm false** (âm tính giả trên bài dạng tóm tắt video ngắn) — ghi ra như một giới hạn của heuristic, không phải một lỗi bóc. ⚠️ Cả 7 mẫu cùng MỘT site — xem giới hạn ở `README.md`/`REPORT.md` của `6-1-ban-do/`; không suy rộng sang site khác. Tỉ lệ này không chặn: đường sửa tay của FR123 vẫn là phương án dự phòng theo thiết kế | —

⚠️ **Đo bổ sung 2026-09-03 (lượt rà đối kháng, theo đúng khuôn Story 3.10b `:896`) — byte nhị phân `--release`, KHÔNG dựng lại `dist/` giữa hai lượt.** Hai bản dựng `cargo build --release --locked` (`dist/` đối chiếu **giống hệt** giữa hai cây bằng `diff -rq`, đúng cảnh báo phương pháp của Story 3.10b — story này không đổi frontend): baseline `193ec73` (worktree riêng, `/private/tmp/aura-6-1-p5/baseline/`) = **8.102.176 byte** → cây hiện tại (`src-tauri/target/release/auratranslate`) = **8.102.160 byte**.

1. **Delta = −16 byte trên 8,1 MB — đây là "không đo được khác biệt", KHÔNG phải "rẻ".** So với tiền lệ Story 3.10b (+156.392 byte cho ĐÚNG MỘT phụ thuộc) thì đây là một hạng độ lớn khác hẳn — dưới cả sai số căn chỉnh của trình liên kết — nên điều đáng ghi là VÌ SAO chứ không phải chính con số.
2. **Lý do là CƠ CHẾ, không phải may mắn — xác nhận bằng `nm`/`strings`:** `dom_smoothie`/`chardetng` và toàn bộ cây con 10 gói bắc cầu biên dịch thành `.rlib` trong `target/release/deps/` nhưng **0 ký hiệu** của chúng có mặt trong nhị phân cuối. Trình liên kết loại bỏ trọn vì **chưa một dòng mã SẢN PHẨM nào gọi tới** (`core/webimport/mod.rs` vẫn chỉ có doc-comment) — chỉ nhị phân TEST (`webimport_probe.rs`) mới liên kết thật.
3. 🔴 **PHẠM VI HẸP — mệnh đề dễ bị trích sai nhất.** Con số này đo *"đã ghim nhưng chưa gọi"*, KHÔNG đo *"đường nhập tốn bao nhiêu"*. Nó HẾT ĐÚNG ngay khi Story 6.9 gọi `dom_smoothie` thật — lúc đó `html5ever`/`markup5ever`/`selectors`/`cssparser` mới thật sự vào nhị phân. Đo lại là nợ có chủ **Story 6.9**, ghi ở `deferred-work.md`; đừng đọc hàng Deferred này rồi kết luận dư địa NFR6 đã an toàn.
4. ⚠️ **Giới hạn phương pháp phải ghi ra: đường dẫn tuyệt đối hai bên lệch độ dài** (`/private/tmp/aura-6-1-p5/baseline` = 33 ký tự so với đường dẫn thật của repo = 46 ký tự), mà `OUT_DIR`/`file!()` bị nhúng vào nhị phân (thông điệp panic, debug info) — nên có một sai lệch HỆ THỐNG cỡ vài chục byte không đến từ phụ thuộc nào cả. Ở mức delta này nó cùng bậc độ lớn với chính −16 byte, nên phép đo chỉ đủ sức nói **"không có khác biệt đáng kể"**, KHÔNG đủ sức khẳng định đúng con số −16.

Ăn **~0%** dư địa NFR6 còn lại (3.104.634 byte, chủ Story 10.1) — nhưng xem mệnh đề 3: đây là 0% của một năng lực CHƯA nối dây, không phải phán quyết cuối của FR123. macOS, `rustc 1.97.1 (8bab26f4f)` · `cargo 1.97.1`. Dữ liệu đầy đủ: `6-1-ban-do/environment.txt` + `REPORT.md`. | — |

🔵 **SỬA 2026-09-06 (Story 6.7) — mệnh đề 3 ngay trên ("HẾT ĐÚNG ngay khi Story 6.9 gọi `dom_smoothie` thật") đã ĐÚNG NHƯ DỰ ĐOÁN, chỉ lệch TÊN STORY.** `Extractor` thật hạ cánh ở **Story 6.7**, không phải 6.9 (§Design Notes spec 6.7 "Vì sao thuật toán bóc vào 6.7 dù `epics.md` giao nó cho 6.9" — AD-16 buộc kéo thuật toán bóc lên trước màn xem trước bắt buộc). Đo lại ĐÚNG khuôn trên, cùng máy: hai bản dựng `--release`, CÙNG một `dist/` (`diff -rq` giống hệt), baseline dựng trong `git worktree` tại `d990e1c4f11d86facbc00468e47b4ed1b0ef9ece` (commit ngay trước lượt code 6.7).

| Bản dựng | Byte |
| --- | ---: |
| baseline `d990e1c4` (`Extractor`/`Fetcher` vẫn stub) | **9.370.872** |
| cây hiện tại (Story 6.7 — `Fetcher`+`Extractor` thật) | **13.739.088** |
| **Delta** | **+4.368.216 byte (≈4,166 MiB)** |

Xác nhận CƠ CHẾ bằng `strings` (nhị phân release đã `strip`, `nm` không còn ký hiệu — khác điều kiện đo Story 6.1): `"dom_smoothie"` xuất hiện **5** lần ở nhị phân hiện tại, **0** ở baseline; `"html5ever"` **7** lần so **0**. 🔴 **Delta này ĂN HẾT VÀ VƯỢT dư địa NFR6 đã ghi nhận ở Story 6.1 (3.104.634 byte) — khoảng 140% của con số đó.** Dư địa NFR6 (chủ Story 10.1) PHẢI được tính lại toàn bộ trước khi đóng bất kỳ quyết định ngân sách byte nào tiếp theo — số 3.104.634 không còn phản ánh thực tế từ dòng này. ⚠️ Máy đo có `load average` 15 phút > 100/16 lõi lúc dựng (kéo dài THỜI GIAN dựng tới ~14-15 phút/lượt, không ảnh hưởng KÍCH THƯỚC nhị phân đầu ra). Chi tiết đầy đủ: `deferred-work.md` (mục "Chi phí byte NFR6 THẬT của `dom_smoothie`", dòng `→ ✅ ĐÃ ĐÓNG 2026-09-06`). | — |

L1156 (hàng "HTTP client cho `Fetcher`" — đóng 2026-09-03 Story 6.1):

| ~~**HTTP client cho `Fetcher`**~~ | ✅ **Đã đóng 2026-09-03 (Story 6.1) — bằng XÁC NHẬN `reqwest`, không bằng một crate mới.** Đo trực tiếp cả ba năng lực trên server `127.0.0.1` tự dựng (`6-1-ban-do/reqwest-raw.tsv`): (1) `redirect::Policy::custom` chặn đúng một chặng chuyển hướng sang cổng khác (đứng cho host khác), server bị chặn nhận **0** kết nối, chuỗi chuyển hướng ghi lại được; (2) đọc qua `Read` (không `.bytes()`) dừng ở **1.048.576/20.971.520 byte** quảng cáo — cắt theo dòng chảy, không nạp trọn; (3) cổng không ai lắng nghe ⇒ lỗi được nhận diện đúng là lỗi kết nối (`is_connect()`/`is_timeout()`). Bật thêm feature `blocking` trên dòng `reqwest` đã có sẵn trong `Cargo.toml` — đo `cargo tree` xác nhận **0** crate mới, chỉ bật lại `tokio/sync` cộng hai crate đã có sẵn trong `Cargo.lock` | — |

L1157 (hàng "Ranh giới Chương ở đường nhập song ngữ" FR115 — đóng 2026-08-03, kèm bài học quy trình):

| ~~**Ranh giới Chương ở đường nhập song ngữ** (FR115)~~ | ✅ **Đã đóng 2026-08-03** — PRD chốt **mẫu phân tách áp lên cột nguồn**. Đúng như dự liệu: AD-39 đã cố định phần bất biến (bước tách Chương nằm sau chuẩn hoá, trước xem trước) nên câu trả lời **không đổi kiến trúc**, chỉ điền vào một hàm. Hàng này vốn **rộng hơn thực tế** — nó ghi ba lựa chọn để ngỏ, trong khi tầng thiết kế đã chọn cột nguồn từ hôm trước và ghi kèm biểu thức thật trong `bilingual-import.html`; PRD chỉ phê chuẩn thành chữ. Bài học: trước khi ghi một hàng Deferred, **soát xem tầng dưới đã trả lời chưa** | — |

L1158 (hàng "Hành vi khi một link trong danh sách hỏng" — đóng 2026-09-06 Story 6.7):

| ~~**Hành vi khi một link trong danh sách hỏng** (404, timeout, tường chặn)~~ | ✅ **Đã đóng 2026-09-06 (Story 6.7).** Ice chốt 2026-09-06: **giữ chỗ, đánh dấu** — một link hỏng thành đúng một mục trong danh sách, GIỮ NGUYÊN vị trí, mang lý do phân biệt được (8 lý do: URL không hợp lệ · lỗi HTTP · timeout · không kết nối · chuyển hướng bị chặn · vượt trần kích thước · không phải HTML · bóc ra rỗng). Hai con số (N link · N Chương) vẫn bằng nhau kể cả khi có mục hỏng; xác nhận **khoá** cho tới khi người dùng bỏ mục đó (hai số cùng giảm) hoặc tải lại riêng nó (đúng 1 lời gọi mạng). AD-39 "mọi thứ trước bước ghi" giữ nguyên — 0 hàng ghi xuống khi còn 1 mục hỏng, cưỡng chế bằng `commands::project::chapters_shape_if_all_ok` (trả `None` khi còn lỗi) đồng bộ với `PendingImportSourceState` phía Rust. | — |

L1163 (hàng "HVTĐTD" Q3 — đóng 2026-08-02):

| ~~**HVTĐTD** (Q3)~~ | ✅ **Đã đóng 2026-08-02** — tác giả đồng ý bằng văn bản. Đúng như dự liệu: chỉ thêm một file lớp gỡ rời, AD-10 đã bao, **không đổi kiến trúc**. Lớp này vào Giai đoạn 1. Ràng buộc kèm theo đã ghi vào Rule của AD-10: dữ liệu dùng theo **phép riêng của tác giả, không thuộc GPL v3** | — |

L1164 (mảnh 🔵 bị cắt khỏi tiêu đề hàng "Thư viện editor cho cột bản dịch của lưới" — hàng này vẫn MỞ, không đóng):

🔵 *(đổi tên 2026-08-18: "panel Editor" là tên đã chết sau correct-course 2026-08-14; câu hỏi và AD-31 không đổi)*

L1165 (mảnh bị cắt khỏi cột "Điều kiện mở lại" của hàng "Cách phân tích khung SSE" — hàng này vẫn MỞ):

*(Sửa 2026-08-13: bản trước ghi "Giai đoạn 2". CAP-4 dời sang Giai đoạn 2c và 2c nay chạy SAU Giai đoạn 3b — xem `build-sequence.md` cột "Thứ tự". Mỏ neo cũ để lại sẽ bị đọc thành "phải rà giấy phép SSE trước khi làm Editor", sai cả hai vế. Cửa rà NFR15 **không đổi**, chỉ mở muộn hơn.)*

L1167 (hàng "Dung lượng và giấy phép font nhúng" — đóng 2026-08-03 Story 1.1):

| ~~**Dung lượng và giấy phép font nhúng**~~ | ✅ **Đã đóng 2026-08-03** (Story 1.1) — đo thật: chênh lệch `.dmg` do font là **20,300 MiB = 21,29 MB**, tổng với database 130 MB hiện tại là **151,29 MB**, **dưới trần NFR6**. Giấy phép: **SIL OFL 1.1** cả ba, tương thích GPL v3 theo diện gộp gói, đã ghi ba hàng vào bảng Stack. Ước 30–50 MB của bản trước **quá cao**; nhưng ước 21,6 MB sau đó lại **quá thấp** vì phần CJK là 23,41 MiB chứ không phải 19 MB. **Còn nợ hai việc, cả hai đã có AC thật, không chặn:** `.msi` chưa đo được (`tauri-cli` trên macOS từ chối target `msi`) → **AC mới của Story 1.3**; và dư địa dưới trần chỉ còn **~47 MB** cho các nguồn từ điển còn lại **cộng toàn bộ mã sản phẩm chưa viết** → **AC mới của Story 1.9** 🔄 **CẬP NHẬT 2026-08-05 — dòng "dư địa ~47 MB" là BẢN GHI tại thời điểm 2026-08-03, không còn là ràng buộc đang sống.** NFR6 sửa lần hai: trần nâng **150–200 MB → 400.000.000 byte**; payload đo thật với BẢY nguồn = **343.991.430 byte**, ĐẠT, dư **56.008.570**. Xem `prd.md` §7.2.. *(Rà soát 2026-08-03: bản đầu của hàng này ghi "200 MB database + 20,30 MiB font = 220 MB vượt trần" — **đọc sai `[A2]`**, vì 150–200 MB là trần của cả bản cài **đã bao gồm font**. Phép tính đúng là trừ dư địa, không phải cộng lên trần.)* | — |

L1168 (hàng "Biến thể vùng cho Source Han Serif" — đóng 2026-08-03 Story 1.1):

| ~~**Biến thể vùng cho Source Han Serif**~~ | ✅ **Đã đóng 2026-08-03** (Story 1.1) — chốt **TC** (`NotoSerifCJKtc-Regular.otf`). Lý do: phạm vi dự án là dịch thuật **tổng quát** chứ không phải ngách truyện mạng (Ice chốt ở giai đoạn brief), và hai lớp từ điển của chính sản phẩm — Cổ hán văn và HVTĐTD — đều là ngữ liệu cổ văn. Khác biệt nặng nhất hoá ra **không** phải dáng chữ mà là **vị trí dấu câu**: TC đặt 「，。」 giữa ô chữ, SC đặt góc dưới trái — xuất hiện ở mọi dòng, không chỉ vài mã hiếm. **Chi phí đổi ý bằng 0**: hai tệp lệch nhau 1.176 byte | — |
