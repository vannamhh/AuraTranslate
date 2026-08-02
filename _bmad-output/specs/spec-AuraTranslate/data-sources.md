# Nguồn dữ liệu từ điển & giấy phép — AuraTranslate

> Companion của `SPEC.md`. Đây là mối quan tâm lớn nhất của dự án **ngoài phạm vi kỹ thuật**, và là lý do một nguyên tắc sản phẩm (hiển thị nguồn) trở thành yêu cầu bắt buộc (FR31, FR32).

## Giấy phép dự án: GPL v3

**GPL v3 là lựa chọn chủ động, không phải hệ quả kỹ thuật.** Quyết định này từng bị ép bởi việc dùng FVDP (GPL v2+, có tính lan truyền). Sau khi loại FVDP, **toàn bộ dữ liệu còn lại mang CC-BY-SA, phạm vi công cộng hoặc Unicode License — không có gì buộc dự án phải theo GPL nữa.** Chủ dự án vẫn giữ GPL, nay là một lập trường.

Chọn **v3 chứ không phải v2** vì v3 tương thích với crate Apache-2.0 — phủ gần trọn hệ sinh thái Rust mà không phải kiểm tra từng gói.

## Bộ nguồn từ điển

| Lớp | Nguồn | Giấy phép | Vai trò | Trạng thái |
|---|---|---|---|---|
| **Nền** | **CVDICT** | CC-BY-SA 4.0 | Từ và cụm từ ZH→VI, >122.000 mục | ✅ Sạch |
| **Nền** | **Unihan** | Unicode License | Âm Hán Việt, nền tab Hán Việt | ✅ Sạch |
| **Nền** | **CC-CEDICT** | CC-BY-SA 4.0 | Đối chiếu chéo — ý kiến thứ ba khi các nguồn Việt bất đồng | ✅ Sạch |
| **Nền** | **kaikki.org / Wiktextract** *(viwiktionary)* | CC-BY-SA + GFDL | **Từ loại + nghĩa + ví dụ cho tiếng Anh** — 133.319 mục, 100% có từ loại | ✅ Sạch |
| **Nền** | **en.wiktionary** *(qua Wiktextract)* | CC-BY-SA + GFDL | **Khung từ loại + câu ví dụ cho tiếng Trung**, ghép nghĩa tiếng Việt từ CVDICT | ✅ Sạch |
| **Gỡ rời** | **Thiều Chửu** (1942) | Phạm vi công cộng *(bản số hoá **không xác minh**)* | Tự điển ký tự chuẩn mực | 🟡 Rủi ro đã chấp nhận |
| **Gỡ rời** | **Cổ hán văn** — Tam tự kinh, Thiên tự văn, Bách gia tính | Văn bản gốc phạm vi công cộng *(bản chú giải **không xác minh**)* | Trích dẫn minh hoạ cách dùng cổ văn | 🟡 Rủi ro đã chấp nhận |
| **Gỡ rời** | **VietPhrase** | ❓ Không xác định được tác giả | Cách cộng đồng dịch giả **thực sự** dịch, tích luỹ hơn một thập kỷ | 🟡 Đóng gói tách rời, có chính sách gỡ (FR112) |
| **Gỡ rời** | **Hán Việt Từ Điển Trích Dẫn** (HVTĐTD) | © Đặng Thế Kiệt — **đã được tác giả cho phép bằng văn bản, 2026-08-02** | Nguồn duy nhất có từ loại + ví dụ + trích dẫn **bằng tiếng Việt** cho Hán Việt | ✅ Được phép — lớp gỡ rời cao cấp |
| — | ~~Trần Văn Chánh (1999)~~ | Còn bản quyền | — | ⛔ Đã loại |

## Lớp từ loại tiếng Trung — quyết định "B rồi C", nay đã có cả hai

Giai đoạn 0 đo được kaikki.org chỉ phủ **2,76%** đầu mục tiếng Trung của CVDICT, và chỉ **0,067%** có kèm ví dụ. Lựa chọn kaikki.org làm lớp từ loại **đúng cho tiếng Anh nhưng sai cho tiếng Trung**. Quyết định khi đó là **chạy song song hai đường**: xin phép HVTĐTD ngay, đồng thời dựng lớp C làm nền bắt buộc để **tiến độ dự án không phụ thuộc vào một lời đồng ý**.

**Cả hai đường nay đều về đích.**

### Lớp C — nền bắt buộc, không đổi

en.wiktionary cho khung từ loại và câu ví dụ, CVDICT cho nghĩa tiếng Việt. **Vẫn là lớp nền bắt buộc.** Việc HVTĐTD được đồng ý **không** làm lớp C thành tuỳ chọn — HVTĐTD là lớp gỡ rời, nên sản phẩm phải đầy đủ chức năng khi không có nó.

Hệ quả cho FR35 giữ nguyên: ở v1, nhãn từ loại và bản dịch ví dụ **bằng tiếng Anh được chấp nhận** cho mục từ tiếng Trung, và phải được đánh dấu rõ là nhãn ngoại ngữ.

### HVTĐTD — đã được phép, làm lớp cao cấp gỡ rời

Tác giả **Đặng Thế Kiệt** hồi âm ngày **2026-08-02**, xác nhận cho phép sử dụng data trong Từ điển Hán Việt Trích dẫn, và **đề nghị được thông báo khi công cụ hoàn thành**.

HVTĐTD chồng lên nền theo đúng mô hình đã dùng cho VietPhrase — **không đổi kiến trúc**, chỉ thêm một file `.db` lớp gỡ rời (AD-10).

> **Ràng buộc giấy phép:** phần dữ liệu này **không thuộc GPL v3** mà dùng theo phép riêng tác giả cấp — GPL không thể áp lên phần dự án không sở hữu. Phải ghi rõ trong `LICENSE`/`NOTICE` và trong màn hình Attribution (FR109). Vì đây là **phép sử dụng chứ không phải giấy phép mở**, lớp này giữ nguyên hình dạng gỡ rời: phép có thể được rút lại, và FR36 + FR112 đã bao trường hợp đó.

**Test nghiệm thu FR36 mà lớp này cho không:** bật lớp → mục từ Hán Việt có từ loại · ví dụ · trích dẫn **bằng tiếng Việt**; xoá file `.db` đó → rơi về nhãn tiếng Anh của lớp nền, và toàn bộ bộ test tra cứu vẫn xanh.

**Phạm vi phân phối lại — quyết định 2026-08-02 (đóng Q8):** thư đồng ý nói *"sử dụng data"*, không nói rõ có bao gồm việc đóng gói và phân phối lại file dữ liệu kèm bản cài công khai trên GitHub Releases. Chủ dự án chọn **mặc định cho phép: đóng gói vào bản phát hành, gỡ khi tác giả yêu cầu.** Không hỏi lại trước khi đóng gói.

> **Cùng tư thế phản ứng như mục dưới đây**, và có cùng đường lui: lớp gỡ rời (FR36) + chính sách gỡ bỏ (FR112). Khác biệt đáng ghi — với Thiều Chửu và Cổ hán văn, chủ sở hữu không xác định được; ở đây tác giả **đã biết mặt, đã hồi âm và đang giữ liên lạc**, nên bất đồng về phạm vi (nếu có) sẽ đến dưới dạng một yêu cầu trực tiếp chứ không phải im lặng kéo dài. Đó cũng chính là lý do biện pháp phản ứng ở đây rẻ hơn hẳn hai nguồn kia.

## Nguyên tắc kiến trúc: nền sạch + lớp gỡ rời

Cả VietPhrase, HVTĐTD, Thiều Chửu và Cổ hán văn đều dùng **chung một khuôn mẫu** (FR36): gỡ bất kỳ lớp gỡ rời nào cũng không làm hỏng chức năng tra cứu. Đây là thứ biến một rủi ro pháp lý thành một quyết định đóng gói.

Cưỡng chế bởi `ARCHITECTURE-SPINE.md` AD-10: mỗi lớp gỡ rời là **một file `.db` độc lập**, runtime không có mã riêng cho từng nguồn, gỡ một lớp = xoá một file. Mỗi file `.db` tự mang metadata giấy phép và ghi công của chính nó, nên gỡ một lớp cũng gỡ luôn ghi công của nó — không để lại ghi công mồ côi khi thực thi FR112.

## Rủi ro xuất xứ đã chấp nhận có ý thức

**Chủ dự án quyết định (2026-08-02): không xác minh xuất xứ trước khi phát hành.** Hai việc dưới đây từng nằm trong danh sách bắt buộc và **đã được chủ động bỏ**:

| Việc đã bỏ | Rủi ro còn lại |
|---|---|
| Xác minh bản quyền **bản Thiều Chửu số hoá cụ thể** **[A3]** | Bản gốc 1942 nhiều khả năng đã thuộc phạm vi công cộng, nhưng **bản số hoá có thể kèm tuyên bố quyền riêng** |
| Chọn **bản Cổ hán văn không có chú giải hiện đại** | Văn bản gốc đã rất cổ, nhưng **phần chú giải của người biên soạn hiện đại thì còn bản quyền** |

**Vì sao quyết định này có đường lui:** kiến trúc lớp nền + lớp gỡ rời (FR36) và chính sách gỡ bỏ (FR112) vốn thiết kế cho VietPhrase, và **áp dụng được nguyên vẹn cho Thiều Chửu và Cổ hán văn**.

**Cái giá phải trả:** rủi ro chuyển từ **chủ động** (biết trước, xử lý trước) sang **phản ứng** (xử lý khi có khiếu nại). Với một bản phát hành công khai mang tên thật, đây là đánh đổi thật chứ không phải hình thức.

> **Hệ quả bắt buộc:** vì không xác minh trước, **Thiều Chửu và Cổ hán văn phải được đóng gói như lớp gỡ rời** (FR36), **không phải lớp nền** — khác với giả định ban đầu ở bảng nguồn phía trên. Kiến trúc phải phản ánh điều này.

> Phần bản quyền trong nghiên cứu nền là **suy luận từ dữ kiện đã xác minh, không phải ý kiến pháp lý**.

## Việc phải làm trước khi phát hành

1. **Ghi công đầy đủ từng nguồn** theo yêu cầu CC-BY-SA, và giữ share-alike cho phần dữ liệu phái sinh.
2. **Rà soát tương thích GPL v3** của toàn bộ crate Rust và thư viện frontend (NFR15).
3. **Ghi phép dùng HVTĐTD vào `LICENSE`/`NOTICE`** — nêu rõ phần dữ liệu này thuộc © Đặng Thế Kiệt, dùng theo phép riêng tác giả cấp, không thuộc GPL v3.

## Việc phải làm sau khi hoàn thành

- **Thông báo cho tác giả Đặng Thế Kiệt khi công cụ hoàn thành** — đề nghị tường minh của tác giả trong thư đồng ý. Đây là nghĩa vụ dự án, không phải một yêu cầu chức năng, nên không mang số FR; nhưng nó là điều kiện của phép sử dụng nên không được rơi.
