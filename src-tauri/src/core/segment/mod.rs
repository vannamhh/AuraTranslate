//! Segment: tách · gộp · tách đôi · về hưu (AD-3, AD-4, AD-5) + pipeline nhập (AD-39).
//!
//! ID segment bền và không bao giờ tái dùng (AD-3). Ranh giới tính một lần lúc nhập,
//! không bao giờ tính lại (AD-4). Gộp/tách là "về hưu + tạo mới", không sửa tại chỗ (AD-5).
//!
//! [`import`] — bước ĐẦU VÀO của chuỗi pipeline nhập (Story 1.15/6.2): dán văn bản, kéo-thả,
//! ô nhập đường dẫn đổ vào **cùng một** hình dạng ([`pipeline::PipelineShape`]). 🔵 **SỬA
//! 2026-09-04 (Story 6.2)** — module này KHÔNG còn tự tạo `segment` HAY tự giải mã; cả hai
//! đã chuyển vào [`pipeline`] (bước 1 và bước 7 của chuỗi AD-39). Nó chỉ còn đọc byte/nhận
//! văn bản và từ chối phần mở rộng/kích thước không hợp lệ TRƯỚC khi trao đi.
//!
//! [`pipeline`] — chuỗi nhập BẢY BƯỚC, thứ tự CỐ ĐỊNH, dùng CHUNG mọi nguồn (AD-39, Story
//! 6.2). Tiêu thụ một hằng [`pipeline::PIPELINE_ORDER`] khai thứ tự LÀ DỮ LIỆU — điều kiện
//! để một thứ tự SAI dựng được thành một giá trị test chạy qua ĐÚNG bộ chạy sản phẩm, thay
//! vì chỉ quét chữ trên mã nguồn. `commands::project::create_work` là chỗ gọi sản phẩm DUY
//! NHẤT của [`pipeline::run_import`].
//!
//! [`split`] — bộ tách câu cấp CÂU và cờ kết đoạn (Story 2.1, FR23, AD-37; GỌI bởi
//! [`pipeline::Step::SplitSegments`], bước 7 của chuỗi AD-39). Hàm thuần, chạy **một lần**
//! lúc nhập; kết quả ghi xuống `project.db` và không đường mã nào tính lại lúc nạp Chương
//! (AC3). Đây là chỗ DUY NHẤT trong kho biết bảng chữ cái kết câu —
//! `tests/segment_boundary.rs` cưỡng chế mệnh đề đó trên cả cây nguồn.

//! [`omit`] — chốt LỌC cho mọi đầu ra (Story 2.5c, FR133, AC5). Một câu người dùng đã cắt
//! bỏ phải biến mất khỏi Chế độ đọc và mọi bản xuất; cả hai bề mặt đó chưa tồn tại, nên
//! module này dựng sẵn cái chốt thay vì giao nghĩa vụ cho trí nhớ. Xem doc-comment của nó —
//! nghĩa vụ FR133 hôm nay chỉ phát biểu MỘT CHIỀU và không AC nào ở phía tiêu thụ canh nó.

//! [`paragraph`] — bảng ba ca biên của AD-37 áp cho một **CẶP** cờ kết đoạn (Story 2.5d,
//! FR134, AD-46, AC3). Cùng khuôn và cùng lý do với [`omit`]: nghĩa vụ có thật, bề mặt tiêu
//! thụ *(gộp/tách tường minh — Story 2.8)* thì chưa. Dựng cái chốt thay vì giao cho trí nhớ.

//! [`regroup`] — phép tính THUẦN cho hàng mới của một lượt gộp/tách tường minh (Story 2.8,
//! FR78, AD-5, AD-47 ④). Khác [`omit`] và [`paragraph`] ở đúng một điểm: bề mặt tiêu thụ
//! của nó **có thật** ngay trong story dựng nó (`commands::segment::merge_segments` ·
//! `split_segment`). Nó tách ra vì một lý do khác — bốn luật ngoài mã (AD-37 · AD-47 ④ ·
//! chữ ký #5(a) · #3(b)) không được sống trong một closure `Store::write` mà `tests/**`
//! gọi không tới.

//! [`reading`] — gom segment thuộc bản dịch thành ĐOẠN cho Chế độ đọc (Story 5.11, FR11).
//! Lượt gọi ĐẦU TIÊN của [`omit::segments_in_translation`] trên đường sản phẩm — mệnh đề
//! *"chưa bề mặt nào gọi nó"* ở doc-comment của `omit` hết đúng cho vế Chế độ đọc kể từ
//! story này (vế bản xuất vẫn mở, Epic 8). Tách đoạn cắt trước-lọc-sau/lọc-trước-cắt-sau
//! có một ca biên thật (cờ kết đoạn nằm trên chính câu bị cắt bỏ) nên nó ở lại Rust, cùng
//! lý lẽ [`paragraph`] đã dùng cho AD-37.

//! [`encoding`] — phát hiện bảng mã cho [`pipeline::Step::DecodeEncoding`] (Story 6.3,
//! FR126, AD-39). Ba trạng thái tin cậy là luật CỦA TA, không của `chardetng` — xem
//! doc-comment đầu tệp cho phép đo. `sniff_bom` → `detect` → `render_candidates` là chuỗi
//! chỉ đọc, KHÔNG bước nào trong đó thay đổi [`pipeline::PIPELINE_ORDER`].

//! [`normalize`] — thân THẬT của [`pipeline::Step::NormalizeParagraphsAndWhitespace`], bước
//! 4 của chuỗi AD-39 (Story 6.4, FR124/FR125). Hàm thuần: thống nhất xuống dòng → trim hai
//! đầu mỗi dòng → gộp dòng giữa câu + thu dòng trống về một. GỌI [`split::line_ends_a_sentence`]
//! và [`regroup::source_joiner`] cho hai bảng đã có chủ — không dựng bảng thứ hai; mệnh đề
//! *"`split` là chỗ duy nhất biết bảng kết câu"* ở đoạn trên vẫn đúng, `line_ends_a_sentence`
//! chỉ mở thêm một cửa `pub(super)` cạnh bảng đó, không lộ nó ra ngoài `core::segment`.

pub mod encoding;
pub mod import;
pub mod normalize;
pub mod omit;
pub mod paragraph;
pub mod pipeline;
pub mod reading;
pub mod regroup;
pub mod split;
