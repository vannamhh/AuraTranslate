# Duyệt FR160 và FR79 (PRD) theo AD-68, 2026-11-04

Phạm vi: FR160 (PRD dòng 833-847), FR79 (dòng 602), dòng bảng C7 (192), dòng Tổng (197). Đối chiếu AD-68 (kiến trúc, `### AD-68`), FR69, FR81, FR83-FR86, FR97, FR103, FR150. Không sửa PRD.

## Kết luận

FR160 đúng tầng năng lực (không tên bảng, cột, hàm), số FR và dòng Tổng đúng, nhưng có một câu nói quá so với AD-68 (đổi tên) và bỏ sót vài hành vi người dùng thấy (cách tầng Work che và chọn bộ, marker không mở rộng, chi phí token, FR97 tự chứa). Chưa nên coi là đóng cho đến khi sửa mục Cao và Trung bình.

## Kiểm đếm cơ học (đã chạy)

- Đếm `^\*\*FR[0-9]+\.\*\*` = 159, không trùng số; FR20 là `~~**FR20.**~~` nên không tính. FR1-FR160 trừ FR20 = 159. Dòng Tổng ĐÚNG.
- Dòng bảng C7 "FR80-FR86, FR150, FR160": đúng, FR150 nằm ở §6.7. FR79 nằm ở hàng C4 (không cần đổi; xem T4).
- Đánh số toàn cục ổn định: FR160 mang số cuối dãy, đặt trong §6.7 sau FR150 theo tiền lệ FR121 và FR150. Đạt.
- Ký hiệu: FR160 thân không có 🔴 ⚠️ ✅ 🟡 🔵 nào. Không ký hiệu mới. Vi phạm vị trí: xem T5.

## Đối chiếu hai chiều

AD-68 cam kết, FR160 và FR79 CÓ nêu: loại bộ tách biệt; hai tầng Global/Tác phẩm; ba trạng thái (theo Global / không dùng / bộ cụ thể); lựa chọn theo `.atproj`; hợp đồng trả lời không sửa được; trả sai định dạng thành chưa quét; tên bộ mất thì dừng trước mạng và nêu tên; đổi tên đi theo; xoá cần xác nhận; ghi vết mỗi lượt quét; không chọn thì như trước; tệp mang loại, nhập không tạo bộ dịch.

AD-68 cam kết hành vi người dùng thấy, FR160/FR79 THIẾU:
1. Bộ Work che bộ Global cùng tên (trong cùng loại); Tác phẩm chọn tên chỉ có ở Global vẫn dùng được.
2. Tên bộ chỉ duy nhất trong cùng loại: một bộ dịch và một bộ Proofreader được trùng tên.
3. Thân bộ Proofreader là chữ thuần: `{{...}}` gõ vào được gửi nguyên văn, màn soạn không cảnh báo marker; thân rỗng nghĩa là không có chỉ dẫn.
4. Chỉ dẫn gửi lại ở mỗi lô, không trần độ dài, token tính vào mức dùng của lượt (FR76).
5. Ô chọn hiện tên đã mất kèm dấu thiếu, không hiện ô trống; lỗi nêu tầng đặt lựa chọn.
6. Xoá không tự bỏ chọn; xác nhận nói một trong hai hậu quả (báo thiếu, hoặc bộ Global cùng tên thay vào).
7. Nhập tệp: va chạm tên chỉ xét trong cùng loại; tệp mang loại lạ bị từ chối; hai bộ cùng tên khác loại xuất ra hai tệp khác nhau.
8. Lượt quét đọc lựa chọn và thân bộ một lần ở đầu lượt.
9. Tỉ lệ FR150 chưa đổi cách đếm (FR160 chỉ nói "về sau tách được").

FR160 nói điều AD-68 không xác nhận hoặc mâu thuẫn:
- "Đổi tên bộ thì lựa chọn đi theo tên mới" (mâu thuẫn một phần, T1).
- "Xoá một bộ đang được chọn cần xác nhận" (AD-68 chỉ bắt buộc khi lựa chọn của Tác phẩm ĐANG MỞ hoặc Global phân giải tới bộ đó; Tác phẩm đóng không được cảnh báo).
- Nghiệm thu ca 1 "quét ở hai Tác phẩm ⇒ cả hai quét bằng bộ đó" chỉ đúng khi hai Tác phẩm đang "theo Global" và không có bộ Work cùng tên che (T3).

## Phát hiện

### T1. CAO: "Đổi tên ... lựa chọn đi theo tên mới" nói quá so với AD-68 mục 4

AD-68: chỉ ghi theo cho Tác phẩm đang mở (nếu bằng tên cũ) và cho lựa chọn Global khi đổi ở tầng Global; Tác phẩm đang đóng giữ tên cũ và gặp lỗi thiếu bộ ở lượt quét kế. Người đọc FR160 hiểu mọi lựa chọn tự đi theo, rồi gặp báo thiếu bộ ở Tác phẩm cũ và coi là lỗi.

Thay dòng "Đổi tên và xoá" bằng:

> - **Đổi tên và xoá.** Đổi tên một bộ thì lựa chọn của Tác phẩm đang mở và lựa chọn Global đi theo tên mới (lựa chọn Global chỉ đi theo khi bộ được đổi tên ở tầng Global). Tác tác phẩm đang đóng giữ tên cũ và gặp báo thiếu bộ ở lượt quét kế tiếp. Xoá một bộ mà lựa chọn của Tác phẩm đang mở hoặc lựa chọn Global đang trỏ tới thì cần xác nhận, và màn xác nhận nói đúng một hậu quả: lượt quét sau sẽ báo thiếu bộ, hoặc bộ Global cùng tên sẽ thay vào. Xoá không tự bỏ chọn.

(Sửa lỗi gõ: "Tác tác phẩm" thành "Tác phẩm" khi áp dụng.)

### T2. CAO: FR160 và FR97 ("tự chứa") chưa được ghi cùng nhau

Tác phẩm chọn một bộ chỉ có ở Global thì `.atproj` mang tên, không mang thân; chép sang máy khác là lượt quét dừng. FR97 nói "copy sang máy khác phải mở được nguyên vẹn" và NFR9 nói "`.atproj` tự chứa". AD-68 chủ ý loại phương án chép thân vào Work. PRD phải nói đây là ngoại lệ có chủ ý, kẻo bị đọc là vi phạm FR97.

Thêm vào FR160, sau dòng "Không rơi im lặng":

> - **Ranh giới với FR97.** `.atproj` mở được nguyên vẹn trên máy khác; cái mất chỉ là thân của bộ Global mà Tác phẩm đã chọn, nên lượt quét dừng và báo tên bộ thiếu (xem trên), không quét sai. Chỉ dẫn muốn đi theo Tác phẩm thì lưu ở tầng Tác phẩm. Sửa bộ Global sau đó vẫn ăn vào mọi Tác phẩm đã chọn nó.

Thêm một câu 🔵 vào FR97 (đứng đầu câu): `🔵 2026-11-04: thân bộ chỉ dẫn Proofreader chọn từ tầng Global không nằm trong tệp (FR160).`

### T3. TRUNG BÌNH: nghiệm thu mơ hồ và thiếu ca

- Ca 1 "Chọn một bộ ở tầng Global, quét ở hai Tác phẩm" không nói hai Tác phẩm ở trạng thái nào. Ca 3 "Xoá bộ đang được chọn" không nói tầng nào, trong khi xoá bộ Work che bộ Global cùng tên thì bộ Global thay vào, không báo thiếu.
- Không có ca cho: trả sai định dạng ⇒ chưa quét; không chọn ⇒ như trước; đổi tên; ghi vết; hai bộ cùng tên khác loại; tệp nhập.

Thay khối "Nghiệm thu" bằng:

> - **Nghiệm thu:**
>   - Global chọn bộ B, hai Tác phẩm đều ở "theo Global" và không có bộ Tác phẩm cùng tên B: quét ở cả hai thì cả hai dùng chỉ dẫn của B.
>   - Global chọn B, Tác phẩm chọn bộ A: Tác phẩm đó dùng A. Tác phẩm chọn "không dùng chỉ dẫn": quét đúng như khi chưa có tính năng này, dù Global có chọn.
>   - Một bộ Tác phẩm trùng tên với bộ Global và được chọn: lượt quét dùng thân bộ Tác phẩm.
>   - Xoá bộ Global mà một Tác phẩm đang mở chọn rồi quét (không còn bộ cùng tên ở tầng Tác phẩm): lượt quét báo thiếu bộ, nêu tên, và không có lời gọi AI nào.
>   - Chỉ dẫn khiến AI trả văn xuôi thay vì kết quả đọc được: các câu của lô được tính vào số chưa quét, không câu nào được báo là không có lỗi.
>   - Không chọn chỉ dẫn nào ở cả hai tầng: kết quả quét giống hệt bản trước tính năng này.
>   - Một bộ dịch và một bộ Proofreader cùng tên tồn tại song song; lượt quét chỉ thấy bộ Proofreader, lượt dịch chỉ thấy bộ dịch.
>   - Sau mỗi lượt quét, bộ chỉ dẫn đã dùng (hoặc "không có") được ghi lại.

### T4. TRUNG BÌNH: hành vi người dùng thấy còn thiếu (mục 1, 3, 4, 5, 7 ở danh sách thiếu)

Thêm vào FR160, sau "Hai tầng":

> - **Bộ chỉ dẫn và bộ prompt dịch tách nhau hoàn toàn.** Hai loại cùng nằm trong thư viện prompt nhưng không bao giờ lẫn: lượt quét chỉ thấy bộ chỉ dẫn, lượt dịch chỉ thấy bộ prompt dịch, và hai bộ khác loại được trùng tên. Trong cùng loại, bộ tầng Tác phẩm che bộ Global cùng tên (FR69). Tác phẩm chọn tên chỉ có ở Global vẫn dùng được bộ đó.
> - **Chữ thuần.** Thân bộ chỉ dẫn được gửi nguyên văn: các biến của prompt dịch không có nghĩa ở đây và không được mở rộng, màn soạn không cảnh báo về chúng. Thân rỗng nghĩa là không có chỉ dẫn.
> - **Chi phí.** Chỉ dẫn được gửi lại ở mỗi lô của lượt quét và không có trần độ dài; token của nó nằm trong mức dùng của lượt (FR76).

Thay câu "Không rơi im lặng" (thêm hai chi tiết giao diện):

> - **Không rơi im lặng.** Bộ đã chọn không còn, vì bị xoá, bị đổi tên ở Tác phẩm đang đóng, hay vì `.atproj` được chép sang máy không có bộ đó: lượt quét dừng trước khi gọi AI và báo **tên bộ bị thiếu** cùng tầng đặt lựa chọn đó. Ô chọn hiện tên đã mất kèm dấu thiếu, không để trống. Ứng dụng không tự quét bằng chỉ dẫn gốc.

Thay dòng ghi vết (nói rõ FR150 chưa đổi):

> - **Mỗi lượt quét ghi lại bộ chỉ dẫn đã dùng** (hoặc "không có"), để về sau tỉ lệ ở FR150 tách được theo chỉ dẫn. Cách đếm FR150 hiện tại không đổi; lượt quét có trước tính năng này không có thông tin chỉ dẫn.

### T5. TRUNG BÌNH: FR79 thiếu hai hành vi nhập/xuất, và FR69/FR103 chưa được 🔵

FR79 hiện: "nhập một tệp chỉ dẫn Proofreader không bao giờ tạo ra một bộ prompt dịch". Thiếu va chạm tên theo loại, tệp loại lạ, tên tệp khác nhau. Thay câu cuối FR79 (sau chữ "(FR160).") bằng:

> Tệp mang theo loại của nó, nên nhập một tệp chỉ dẫn Proofreader không bao giờ tạo ra một bộ prompt dịch, và ngược lại; tệp được nhập vào đúng loại ghi trong tệp, không theo màn đang mở. Va chạm tên chỉ xét trong cùng loại. Tệp mang loại lạ bị từ chối, và hai bộ khác loại cùng tên xuất ra hai tệp khác tên (AD-68).

FR69 chỉ nói "custom prompt theo thể loại", không nêu thư viện có hai loại; "bộ" và "thư viện prompt" không có mục Glossary. Thêm dưới FR69 (đứng đầu câu): `🔵 2026-11-04: thư viện prompt có hai loại bộ, prompt dịch (FR69) và chỉ dẫn Proofreader (FR160); luật "tầng Tác phẩm thắng" áp riêng trong từng loại.` FR103 bảng: ô Global thêm "bộ chỉ dẫn Proofreader mặc định (FR160)"; ô Tác phẩm thêm "lựa chọn chỉ dẫn Proofreader: theo Global, không dùng, hoặc một bộ (FR160)".

### T6. THẤP: ký hiệu `⇒` giữa câu

Ba gạch nghiệm thu dùng `⇒` giữa câu ("... quét ở hai Tác phẩm ⇒ cả hai ..."). Quy ước: ký hiệu chỉ đứng đầu câu. Khối nghiệm thu ở T3 đã thay bằng dấu hai chấm. Nếu giữ khối cũ, thay `⇒` bằng "thì". Cũng thấp: `*(🔵 2026-11-04: ...)*` ở hàng C7 mở ngoặc đơn trước ký hiệu; cùng kiểu đã có ở FR103 nên chỉ ghi chú, đề nghị để nguyên theo tiền lệ.

### T7. THẤP: FR71 (xem prompt cuối cùng đã gửi) chưa nói có áp cho lượt quét

Không bên nào (PRD, AD-68) nói. Nếu có chủ ý thì thêm vào FR160: "Người dùng xem được prompt cuối cùng của lượt quét, gồm chỉ dẫn, như FR71" hoặc ghi vào Open Questions là chưa làm. Đề nghị chủ dự án quyết; không tự thêm cam kết.

### T8. THẤP: dòng C7 không nêu FR79 và C6/C9

AD-68 gắn C6, C7, C9. Hàng bảng chỉ có FR160 ở C7. Chấp nhận được vì mỗi FR thuộc một hàng; nếu muốn, thêm FR160 vào ghi chú hàng C6 `(thư viện prompt)`. Không bắt buộc.

## Rubric (một FR thêm)

- Năng lực, không triển khai: đạt. Có `.atproj` và "Global" là từ vựng sản phẩm (FR97, FR103). Không tên bảng, cột, khoá cấu hình, hàm, hằng.
- Kiểm thử được: các ý chính đều quy được thành ca đo, nhưng nghiệm thu hiện thiếu ca và có chỗ mơ hồ (T3).
- Nhất quán: FR69 (cần 🔵, T5), FR97 (T2), FR103 (T5), FR81/FR83-FR86 (không xung đột: chỉ dẫn không đổi năm loại lỗi, không cho phép tự sửa, ghi nhớ "không phải lỗi" không đổi; có thể thêm một câu "không đổi" nếu muốn), FR150 (T4).
- Không phải trụ cột tương tác Open Questions: không phát sinh câu hỏi mở mới ngoài T7.
