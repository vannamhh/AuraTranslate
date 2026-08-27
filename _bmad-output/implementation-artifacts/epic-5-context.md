# Epic 5 Context: Library — kho tác phẩm, tìm kiếm, và đọc lại thành quả

<!-- Compiled from planning artifacts. Edit freely. Regenerate with compile-epic-context if planning docs change. -->

## Goal

Mở ứng dụng phải vào thẳng Library, không vào Workspace — đây là điểm vào, không phải một màn hình phụ. Người dịch cần nắm được mình đang có gì (lưới Tác phẩm với bìa, tiến độ, bốn trạng thái vòng đời), lọc/sắp xếp được, và tìm full-text xuyên toàn thư viện phân biệt dấu. Mở một Chương phải đưa thẳng vào Workspace đúng câu đang dở lần trước. Epic này cũng dựng Chế độ đọc — chế độ thứ ba, ngang hàng với Library và Workspace — để người dịch đọc lại thành quả như đọc sách, không như nhìn bảng dữ liệu, và đánh dấu chỗ cần sửa mà không cắt phiên đọc. Xoá chỉ mục Library rồi quét lại phải phục hồi đầy đủ, không mất một byte.

Epic này còn là nơi phải nhặt lại một món nợ kiến trúc lớn: **đường mở lại một `.atproj` đã có trên đĩa chưa tồn tại ở bất kỳ Epic nào trước đó** — ứng dụng hôm nay chỉ biết *tạo mới* một Tác phẩm. Library (menu, `Indexer`, `library-index.db`) chính là nơi quyết định hình dạng của đường mở lại đó.

## Stories

- Story 5.1: Mô hình Library hai tầng
- Story 5.2: Chỉ mục Library dẫn xuất, một đường ghi duy nhất
- Story 5.3: Quét lại thư mục
- Story 5.4: Bốn trạng thái vòng đời
- Story 5.5: Tiến độ Tác phẩm
- Story 5.6: Lưới Tác phẩm, lọc và sắp xếp
- Story 5.7: Danh sách Chương và mở Chương vào Workspace
- Story 5.8: Tổ chức lại Chương sau khi nhập
- Story 5.9: Tìm kiếm full-text xuyên Library
- Story 5.10: Hai chế độ dấu
- Story 5.11: Chế độ đọc — typography và bố cục đọc dài
- Story 5.12: Chế độ đọc chỉ đọc phần đã xong
- Story 5.13: Đánh dấu chỗ cần sửa khi đang đọc
- Story 5.14: Đo NFR3, NFR4, NFR5 và ghi lại trạng thái ba ngưỡng tạm

## Requirements & Constraints

Library mô hình hoá đúng hai tầng Tác phẩm → Chương; một tài liệu đơn lẻ là một Tác phẩm có đúng một Chương, không có thực thể thứ ba. Mỗi Tác phẩm mang tên, bìa (tuỳ chọn), ngôn ngữ nguồn (đặt lúc tạo, không đổi được sau), lĩnh vực, ngày tạo/sửa; Glossary và TM gắn ở tầng Tác phẩm. Trạng thái vòng đời có bốn giá trị (Chưa bắt đầu/Đang dịch/Tạm ngưng/Đã xong) ở cả hai tầng; trạng thái Tác phẩm suy ra tự động từ các Chương nhưng ghi đè thủ công được và phải phân biệt rõ với suy ra tự động.

Tìm kiếm full-text xuyên toàn Library, đồng thời trên nguyên văn và bản dịch, kết quả kèm Tác phẩm/Chương/đoạn khớp; đọc từ `library-index.db`. Hai chế độ dấu: chính xác trước, chỉ nới sang khoan dung không dấu khi không có kết quả hoặc người dùng chủ động chọn, và luôn nói rõ đang ở chế độ nào — khoan dung không bao giờ là mặc định.

Ba ngưỡng hiệu năng đo ở Story 5.14 đều là ngưỡng **tạm**, chưa nghiệm thu đủ điều kiện: tìm kiếm full-text p95 < 500 ms, khởi động tới lúc Library dùng được < 3 giây, bộ nhớ nhàn rỗi < 300 MB — cả ba đo trên thư viện 5.000 Chương giả định. Vì Epic 5 không có đường nào tạo ra 5.000 Chương thật (đó là FR14, thuộc Epic 6), phép đo ở đây chỉ sơ bộ; phép đo nghiệm thu đủ điều kiện là Story 6.18, có chủ riêng — không phải một lời nhắc trôi nổi. Toàn bộ thao tác Library và Chế độ đọc phải dùng được bằng bàn phím (NFR17) và đạt WCAG AA ở cả hai theme.

## Technical Decisions

Tên thực thể cố định: Tác phẩm → `Work`, Chương → `Chapter`; cấm `Project`/`Book`/`Novel`/`Document` (đuôi file `.atproj` là ngoại lệ lịch sử). `work.id` là UUID v4 lưu trong `meta.json` (vì `library-index.db` tham chiếu xuyên `.atproj` và người dùng copy `.atproj` sang máy khác); `chapter.id`/`segment.id` là số nguyên cục bộ trong `project.db`.

`library-index.db` là file riêng, tách khỏi `global.db`, chỉ component `Indexer` được ghi vào; `.atproj` luôn ghi trước, chỉ mục ghi sau. Xoá `library-index.db` phải luôn an toàn — dựng lại hoàn toàn từ các `.atproj`, không di trú (xoá và dựng lại khi nâng cấp ứng dụng). Danh sách/lọc/sắp xếp/tìm kiếm đọc từ `library-index.db`; `meta.json` chỉ là thứ `Indexer` đọc lúc quét, không phải nguồn Library đọc trực tiếp lúc chạy. `meta.json` bản thân là cache dẫn xuất từ `project.db` (tiến độ suy ra từ trạng thái Chương), ghi bởi chính `store::Writer` của Tác phẩm đó trong cùng thao tác logic — không thành phần nào khác được ghi vào nó. Indexer phải phát hiện và cảnh báo hai Tác phẩm trùng `work.id`, không âm thầm gộp/ghi đè. Lưu ý khi quét thư mục: một `.atproj` đang sống mang **năm** mục trên đĩa (`meta.json`, `project.db` cộng hai sidecar WAL `-wal`/`-shm`), không phải ba — một lượt quét giả định đúng ba tên sẽ sai.

Chỉ mục FTS chính dùng `remove_diacritics 0` (phân biệt dấu); chỉ mục xoá dấu chỉ là chỉ mục phụ cho chế độ khoan dung, không bao giờ mặc định. Phân giải hai tầng qua `ScopeResolver` chung toàn dự án.

Món nợ trung tâm phải nhặt ở Epic 5: **đường mở lại một `.atproj` đã có trên đĩa** — hôm nay `OpenWorkState` khởi tạo `None` mỗi lượt chạy, và cách duy nhất một Tác phẩm được mở là tạo mới. Đây là quyết định kiến trúc của toàn Tác phẩm (menu Library, `Indexer`, `library-index.db`), nên khi dựng đường này phải nhặt cùng lượt: bề mặt hiển thị của cơ chế từ chối `meta.json` mới hơn (`ProjectError`/`MessageKey`/khoá `vi.json` — cơ chế lõi đã có, chỉ thiếu bề mặt); Glossary/TM tầng Tác phẩm không đọc lại được sau mở lại vì `ScopeResolver::with_work` chỉ dựng lúc tạo mới; và vế "đóng app → mở lại → chữ Editor còn đó" treo cùng lý do.

Vị trí làm việc (segment đang làm + vị trí cuộn) khôi phục khi mở lại một Chương phải tham chiếu **`segment.id`, không phải toạ độ pixel** (AD-3) — phương án scrollTop từng được trình và đã bị loại. Hôm nay 0 `ScopeKind` cho vị trí đọc tồn tại; hình dạng chọn ở đây phục vụ đồng thời ba nơi tiêu thụ: mở Chương từ Library, mở lại Chương đã làm dở, và chuyển chế độ Workspace ↔ Chế độ đọc.

Chế độ đọc: hàm thuần lọc câu-đã-xoá-bỏ ở Rust cộng test hợp đồng đã dựng từ Story 2.8 (Quyết định #2); `src/modes/ReadingMode.vue` hôm nay là khung rỗng có chủ ý chờ đúng Epic 5 cắm giao diện vào chốt đó — không dựng lại cơ chế lọc.

## UX & Interaction Patterns

Ba chế độ Library/Workspace/Chế độ đọc ngang hàng, chuyển bằng `⌘1``⌘2``⌘3`, luôn giữ ngữ cảnh (đúng Chương, đúng câu, đúng vị trí cuộn). Library rỗng giải thích Tác phẩm là gì và là một thư mục mang đi được, rồi mới mời nhập; Tác phẩm không bìa dùng biểu diễn thay thế nhất quán.

Chế độ đọc có ba mức typography đo bằng `ch` không bằng `px` (Thoáng 62ch/19px/1.95 · Cân 68ch/17.5px/1.8 mặc định · Đặc 76ch/16px/1.66, giãn dòng không bao giờ dưới 1.66); mặc định chỉ hiện bản dịch, song ngữ đặt nguyên văn ở lề trái không chen dòng đọc. Không hiển thị công cụ biên tập nào (không vạch lề, không nút xác nhận, không panel). Đọc liên tục qua các Chương Đã xong, dừng ở mốc biên tường minh khi chạm Chương chưa xong kèm đường sang Workspace; câu chưa xác nhận trong Chương đã xong vẫn hiện nhưng có gạch chấm nhẹ phân biệt, không hiện như đã hoàn chỉnh. Đánh dấu chỗ cần sửa: affordance chỉ hiện khi con trỏ/focus chạm câu, phím `M` đánh dấu và đọc tiếp ngay (không chuyển màn hình), `↵` nhảy sang Workspace tại đúng segment; các chỗ đánh dấu gom một danh sách theo Tác phẩm, và một chỗ đánh dấu trỏ tới segment đã về hưu do gộp/tách vẫn ở lại kèm ghi chú.

## Cross-Story Dependencies

5.1 (mô hình hai tầng) là nền cho toàn bộ epic. 5.2 (chỉ mục dẫn xuất) là điều kiện của 5.3 (quét lại), 5.6 (lưới/lọc/sắp xếp) và 5.9 (tìm full-text) — tất cả đọc từ `library-index.db`, không đọc trực tiếp `.atproj`. 5.4 (bốn trạng thái) là điều kiện của 5.5 (tiến độ) và một phần bộ lọc của 5.6. 5.9 và 5.10 (hai chế độ dấu) dùng chung một cơ chế tìm kiếm — 5.10 là phần mở rộng chế độ khoan dung của 5.9. 5.7 (mở Chương, khôi phục vị trí) chia sẻ đúng một hạ tầng vị trí đọc với món nợ AC5 của Story 2.11 (Epic 2) và với việc chuyển chế độ Workspace ↔ Chế độ đọc — thiết kế ở đây quyết định cho cả ba nơi tiêu thụ. 5.8 (tổ chức lại Chương) là thao tác tổ chức thuần (chỉ `chapter_id`/`ord` đổi), khác biệt cố ý với gộp/tách segment ở Story 2.8; `segment.id` và mọi dữ liệu gắn theo segment giữ nguyên qua gộp/tách Chương. 5.11 → 5.12 → 5.13 dựng nối tiếp: 5.11 dựng khung typography, 5.12 thêm ranh giới "chỉ đọc phần đã xong" tái dùng hàm lọc thuần đã có từ Story 2.8, 5.13 thêm đánh dấu trên nền hai story trước. 5.14 đo sơ bộ ba ngưỡng NFR3–NFR5; phép đo đủ điều kiện là Story 6.18 sau khi Epic 6 có đường tạo 5.000 Chương thật (FR14).

Epic 5 không phụ thuộc Epic 4 (đã xác nhận bằng quét từ khoá AI ra 0 lần). Epic 6 phụ thuộc một phần vào Epic 5 ở việc sinh Chương thứ hai trở đi trong một Tác phẩm — hôm nay chưa đường sản phẩm nào tạo Chương thứ hai; FR14 (nhập hàng loạt, Epic 6) là nguồn chính, còn FR15 (gộp/tách/sắp lại Chương, Story 5.8) là nhánh Epic 5 mở thêm trên cùng cơ chế đó. Việc mở lại một `.atproj` mà Epic 5 dựng cũng là điều kiện để Epic 4's `RagInjector` thấy được Glossary/TM tầng Tác phẩm sau một lần mở lại Tác phẩm (đã ghi ở `epic-4-context.md`).
