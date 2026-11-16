# Hướng chỉnh giao diện website

Ice thích chất thị giác của bản đầu, nhưng bản chỉnh nội dung đã biến trang thành nhiều cột văn bản phẳng. Lượt này giữ nội dung tính năng đã đối chiếu và xây lại phân cấp thị giác quanh ảnh app thật.

## Hệ màu và chữ

- Sáng: nền giấy `#f7f5ef`, mặt nổi `#fffdfa`, chữ `#262b2b`, teal `#315f62`, vàng trầm `#a98750`.
- Tối: nền than `#121619`, mặt nổi `#1b2226`, chữ `#f1f1eb`, teal `#9dc9c4`.
- Source Serif 4 cho tiêu đề chính và tiêu đề section; Source Sans 3 cho điều hướng, nội dung, thao tác. Tăng phân biệt giữa tiêu đề, đoạn dẫn và chữ phụ; giới hạn chiều dài dòng trong nội dung.

## Bố cục

```text
Điều hướng / thương hiệu                         Theme / Tải về

                  Hero một màn hình
                  Một nút tải về

             Tab chọn màn hình app
             Ảnh app thật trong khung

    Bốn khối tra cứu, biên tập, thư viện, đọc lại

      Glossary                 Bộ nhớ dịch

      Ngữ cảnh AI nổi bật      Cấu hình và prompt

      Proofreader              Quy ước chữ Việt

    Bốn bước quy trình / nhận biên tập / định dạng thu gọn

    Dữ liệu / hai bản cài / FAQ / footer
```

Hero và ảnh căn giữa; nội dung bên trong các khối căn trái. Dùng biểu tượng nét để nhận diện nhóm tính năng. Phần ngữ cảnh AI có mặt teal đậm, các phần còn lại yên hơn để ảnh sản phẩm và cơ chế AI là hai điểm nhấn chính.

## Tự kiểm trước khi xây

Màu ấm, chữ serif và bo góc xuất phát từ bản Ice đã thích. Giảm nguy cơ thành bộ card SaaS lặp bằng cách dùng các kiểu bố cục khác nhau cho glossary/TM, ngữ cảnh AI, quy trình có thứ tự và FAQ. Chỉ đánh số quy trình. Không thêm huy hiệu thống kê hay nội dung minh họa giả. Giữ tab trên ảnh, không khôi phục nút xem ảnh lớn hoặc CTA phụ đã bị bỏ.

## Kiểm tra

Xem bằng ảnh chụp trình duyệt ở 1440 px, tablet 768 px và mobile 375 px; cả hai theme. Kiểm tra hero và section ảnh tách rời, không tràn ngang, tab đổi đúng ảnh, bảng định dạng mở bằng bàn phím, các nút tải vẫn mở trang release.


## Kết quả kiểm tra

Đã xem ảnh chụp hero, gallery và thẻ tính năng sáng trên desktop; phần AI tối; hero và gallery mobile. Browser đo desktop 1440/1425 px, tablet 768/753 px, mobile 375/375 px (viewport/scrollWidth). Hero kết thúc đúng 1000 px trên màn cao 1000 và 812 px trên màn cao 812; gallery nằm ở section kế tiếp. Tab mobile cao 89 px, nằm trên ảnh. Enter mở bảng định dạng sáu hàng và chọn được ảnh Glossary; không có SVG use bị thiếu đích. Năm tình huống theme/storage/gallery/FAQ với script hiện tại qua. Các nội dung tính năng và ảnh chụp app đã xác minh vẫn được giữ.
