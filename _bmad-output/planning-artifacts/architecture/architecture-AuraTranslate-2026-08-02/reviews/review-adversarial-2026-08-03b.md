# Lens đối kháng — vòng cập nhật FR122–FR131

**Phương pháp:** dựng hai đơn vị ở tầng dưới, mỗi đơn vị tuân **mọi** AD đúng từng chữ, rồi tìm chỗ chúng vẫn dựng ra thứ không khớp nhau.

**Verdict: 6 lỗ, 4 nghiêm trọng.** Tất cả nằm trong vùng vừa thêm; AD-1..AD-38 không sinh lỗ mới.

---

## 🔴 F1 — Allowlist của AD-41 chặn luôn ảnh của FR127

**Hai đơn vị:** Giai đoạn dựng `Fetcher` đọc AD-41: *"allowlist dựng từ danh sách link người dùng vừa dán"*. Giai đoạn dựng đường tải ảnh đọc FR127: *"ảnh trong nội dung tải từ web được tải về"*.

**Va nhau ở đâu:** bài ở `example.com` nhưng ảnh nằm ở `cdn.example.net`, `i.imgur.com`, `images.wp.com`. Host của ảnh **không có trong danh sách link người dùng dán**. Đơn vị thứ nhất từ chối đúng luật; đơn vị thứ hai không tải được gì.

**Hậu quả:** FR127 hỏng hoàn toàn trên phần lớn website thật. Và nó hỏng **im lặng theo hướng xấu nhất** — Chương nhập vào trông bình thường, chỉ thiếu ảnh, đúng thứ người dùng không đếm lúc nhập.

**Đây không phải hai đơn vị hiểu sai — cả hai đều đúng.** AD-41 và FR127 mâu thuẫn ở tầng spine.

**Vá:** allowlist **hai tầng** — host của link người dùng dán, cộng host của tài nguyên **được tham chiếu từ trang đã tải trong cùng lần nhập**, tầng sau **chỉ cho ảnh**, không bao giờ cho tài liệu. Cả hai tầng vào nhật ký domain của NFR19, **phân biệt được với nhau**.

---

## 🔴 F2 — Ảnh không có alt-text thì mất neo vị trí

**Hai đơn vị:** cả hai đọc quy ước *"alt-text mang `ord` đúng vị trí ảnh"* (AD-42, quy ước cũ FR42–FR44).

**Va nhau ở đâu:** ảnh trên web **thường không có thuộc tính `alt`**. AD-42 lại cấm sinh segment rỗng. Vậy khi một ảnh không có alt lẫn caption, **cái gì giữ vị trí của nó trong `ord`?** Đơn vị A sinh một segment giữ chỗ; đơn vị B lưu vị trí trên `ASSET`. Hai mô hình khác nhau cho cùng một dữ kiện.

**Hậu quả:** FR42 và FR43 (*ảnh hiển thị đúng vị trí*) hỏng ở một trong hai đơn vị, và Chế độ đọc với panel Source hiện ảnh ở hai chỗ khác nhau.

**Vì sao lỗ này MỚI:** trước FR127 mọi ảnh đến từ `.docx` của người dùng và giả định *"ảnh nào cũng có alt-text"* còn đứng được. Nhập từ web phá giả định đó.

**Vá:** `ASSET` mang **neo vị trí của chính nó** trong Chương, độc lập với việc có hay không có segment alt/caption. Segment alt và caption **treo vào neo đó**, không phải ngược lại.

---

## 🔴 F3 — AD-39 nói "một chuỗi duy nhất" nhưng không nói chuỗi đó sống ở đâu

**Hai đơn vị:** đơn vị dựng đường nhập `.docx` đọc AD-38 → cổng vào nằm ở `core/export/`. Đơn vị dựng đường nhập URL đọc AD-40 → `core/webimport/`.

**Va nhau ở đâu:** cả hai cần các bước **dùng chung** của AD-39 — giải mã bảng mã, làm sạch, chuẩn hoá, tách segment. AD-39 không chỉ định module sở hữu. Mỗi đơn vị cài một bản trong module của mình.

**Hậu quả:** đúng thứ AD-39 tồn tại để cấm — hai thứ tự, hai kết quả chuẩn hoá, trên cùng một văn bản. Và nó là **hai bản cài đặt của một quy tắc nghiệp vụ**, vi phạm luôn tinh thần AD-1.

**Vá:** đặt tên module sở hữu chuỗi, và nói rõ các module nguồn chỉ **nạp vào** chứ không sở hữu.

---

## 🔴 F4 — Luật làm sạch (FR124) là loại dữ liệu hai tầng chưa có trong bảng AD-18

**Hai đơn vị:** AD-18 khai ngữ nghĩa hai tầng cho **bốn** loại: Glossary (ghi đè), Prompt (ghi đè), Cấu hình AI (ghi đè), TM (hợp nhất). Luật làm sạch của FR124 **không có trong bảng**.

**Va nhau ở đâu:** đơn vị A đặt luật làm sạch ở tầng toàn cục (rác web giống nhau ở mọi Tác phẩm). Đơn vị B đặt ở tầng Tác phẩm (mỗi site một kiểu watermark). Đơn vị C làm cả hai với ngữ nghĩa **ghi đè**, đơn vị D làm cả hai với ngữ nghĩa **hợp nhất**.

**Hậu quả:** bốn hành vi khác nhau, và ba trong bốn sẽ **xoá nhầm hoặc bỏ sót** so với thứ người dùng nghĩ mình đã cấu hình. AD-18 tồn tại chính xác để chặn chuyện này — nó chỉ chưa biết loại dữ liệu mới.

**Cùng lỗ, ca thứ hai:** **tên người dịch** ở AD-43 được nói là *"cấu hình toàn cục, qua `ScopeResolver` theo AD-18"* — nhưng nó cũng không có hàng trong bảng AD-18.

**Vá:** thêm hai hàng vào bảng AD-18 với ngữ nghĩa khai tường minh.

---

## 🟡 F5 — Bước giải mã bảng mã áp cho cả nguồn không có gì để giải mã

**Va nhau ở đâu:** AD-39 đặt *giải mã bảng mã* là bước 1 cho **mọi** nguồn. Nhưng `.docx` là zip chứa XML **đã khai encoding**; chạy bộ dò thống kê trên byte zip cho ra rác. Đơn vị A chạy bộ dò trên mọi thứ; đơn vị B bỏ qua với `.docx`.

**Hậu quả:** nhẹ hơn F1–F4 vì đơn vị A sẽ hỏng ồn ào chứ không im lặng. Nhưng nó vẫn là hai hành vi từ cùng một quy tắc.

**Vá:** khai rõ bước 1 áp cho nguồn **không mang khai báo bảng mã** (`.txt`, `.md`), và cho phản hồi HTTP (nơi khai báo có thể sai hoặc vắng); `.docx` bỏ qua bước này.

---

## 🟡 F6 — "Không tải lại ảnh đã có" (NFR19) chưa có khoá định danh

**Va nhau ở đâu:** AD-15 nhắc ràng buộc *không tải lại ảnh đã có*, nhưng không nói **đã có** so theo cái gì. Đơn vị A so theo `source_url`; đơn vị B so theo băm nội dung ảnh.

**Hậu quả:** nhập lại cùng một Chương cho ra số lời gọi mạng khác nhau giữa hai bản — nên **bộ test của AD-41 không xác định được**, mà AD-41 lại nói chính nó phải có test riêng.

**Vá:** khoá là `source_url` trong phạm vi cùng một Tác phẩm. Băm nội dung là tối ưu lưu trữ, không phải điều kiện tải.
