# Review đối kháng AD-68 — 2026-11-04

Phạm vi: chỉ đọc mục AD-68 (spine dòng 1743–1789) và mã `core/promptset/store.rs` (`rename`, `delete`, `read_chosen_name`), `exchange_io.rs` (`export_file_name`), `schema.rs` (`proof_scan`, `proof_decision`). Lựa chọn của Ice không bị tấn công; chỉ tấn công các Rule. Mã được đọc ở working tree hiện tại, chưa có thay đổi AD-68 nào trong mã.

Kết luận: các Rule giữ được hợp đồng trả lời và thứ tự lắp, nhưng để hở vòng đời của TÊN được chọn (đổi tên, xoá, bóng che, `.atproj` mang sang máy khác) và để lượt quét "thành công rỗng" khi chỉ dẫn mâu thuẫn định dạng, không có chỗ nào ghi lại chỉ dẫn nào đã sinh ra phát hiện.

## H1 — critical/high: đổi tên và xoá bộ không biết có ai đang chọn nó

- Đơn vị A: `promptset::rename` / `delete` (mã hiện tại: `UPDATE prompt_set SET name = ?1 WHERE id = ?2` và `DELETE ... WHERE id = ?1`, không đọc bảng chọn). Đơn vị B: Rule 3 lưu lựa chọn là TÊN (`work.chosen_proofread_prompt_set_name`, `config_value.proofread_prompt_set`).
- Mỗi bên tuân đủ Rule: Rule 1 chỉ bảo mọi đường nhận `kind`; Rule 3 chỉ bảo lỗi khi tên không còn. Không Rule nào bảo ai cập nhật lựa chọn khi đổi tên.
- Kịch bản 1 (Global): Global chọn "Chặt chẽ", người dùng đổi tên thành "Chặt chẽ v2". Lượt quét kế tiếp ở MỌI Tác phẩm dừng với `proofread.set_missing` (Rule 3). Không mất dữ liệu nhưng cả thư viện Tác phẩm hỏng một lượt, và thông báo nêu tên cũ mà người dùng không còn thấy bộ nào mang tên đó.
- Kịch bản 2 (Work, `.atproj` KHÔNG mở): Tác phẩm X chọn "A" ở `project.db` của nó. Ở Cài đặt, người dùng đổi tên "A" tầng Global (đang có Tác phẩm Y mở). Lựa chọn của X nằm ở tệp khác không mở, nên không có đường cập nhật; lần sau mở X, quét dừng. Cách "sửa lựa chọn khi đổi tên" không thể làm đầy đủ trong một giao dịch vì cột nằm ở kho khác (AD-8: nhiều `.atproj`).
- Kịch bản 3 (rơi im lặng, nguy hiểm nhất): Work chọn "A", tầng Global cũng có "A" (Rule 1: Work che Global cùng tên cùng loại). Người dùng xoá "A" tầng Work (đúng như doc của `delete`: "bộ Global trở lại hiệu lực ngay"). Tên vẫn phân giải được, nên KHÔNG có lỗi, nhưng chỉ dẫn đổi sang thân khác hẳn. Rule 3 "báo khi tên mất" không kích hoạt vì tên chưa mất. Đây đúng là loại "silent emptiness / đổi hành vi không báo" mà AGENTS.md gọi tên.
- Hậu quả: lượt quét chạy bằng chỉ dẫn khác với cái người dùng nghĩ; `proof_scan` không ghi lại (xem H5).
- Sửa AD nhỏ nhất: thêm vào Rule 3 hai câu. (i) "Đổi tên hoặc xoá bộ `proofread` ở tầng T: nếu tên cũ đang được chọn ở tầng cùng kho (hàng `config_value` cho Global; cột `work` của Tác phẩm đang mở cho Work), thao tác bị từ chối với lỗi nêu nơi đang chọn, hoặc người dùng xác nhận 'bỏ chọn'; KHÔNG tự đổi theo." (ii) "Lựa chọn ở kho không mở không bao giờ bị sửa từ xa; cái lưới an toàn là lỗi `set_missing` ở Rule 3, và lỗi đó phải nêu TẦNG đã chọn." Cộng thêm: xoá bộ Work che bộ Global cùng tên khi tên đó đang được chọn phải hỏi xác nhận (nêu rõ chỉ dẫn sẽ chuyển sang bộ Global). Thêm ca test (i) vào Rule 7.

## H2 — high: `.atproj` mang sang máy khác, global.db thiếu bộ Global

- Đơn vị A: tầng Work lưu TÊN trong `.atproj` (FR97 — đi theo tệp). Đơn vị B: bộ chỉ dẫn nằm ở `global.db` của từng máy. Rule 3 nói tên mất thì dừng.
- Kịch bản: máy 1 chọn "Chặt chẽ" (bộ Global, tầng Work không có bản). Sao chép `.atproj` sang máy 2. Máy 2 quét: dừng với lỗi `set_missing`. Về an toàn thì đúng, nhưng: (a) người dùng không có cách lấy thân bộ từ `.atproj` (thân không nằm trong tệp), (b) hậu quả với cộng tác viên là Tác phẩm "không quét được" cho đến khi họ tự làm lại bộ trùng tên, (c) nếu họ làm bộ trùng tên nhưng thân khác, kết quả quét khác nhau giữa hai máy mà không ai biết (cùng tên, hai thân).
- Rule hiện tại mâu thuẫn với mục đích của cột "đi theo `.atproj`": cột đi theo nhưng thứ nó trỏ tới không đi theo.
- Sửa AD nhỏ nhất: chọn một trong hai (đây là hai lựa chọn hợp lệ, cần Ice, không được tự chọn): (A) thêm một câu "Thân bộ không nằm trong `.atproj`; lỗi `set_missing` có nút nhập tệp `.prompt.md` mà vẫn giữ nguyên tên đã chọn"; hoặc (B) khi chọn bộ ở tầng Work, sao bộ ra tầng Work (đọc: ghi `prompt_set` vào `project.db`) để nó đi theo tệp. Mức tối thiểu đề xuất cho văn bản AD: ghi nhận (A) kèm một dòng "chống trôi": lượt quét ghi mã băm thân (xem H5), nên hai máy cùng tên khác thân phân biệt được.

## H3 — high: chỉ dẫn mâu thuẫn định dạng, lượt quét "xanh rỗng"

- Đơn vị A: Rule 4 đặt chỉ dẫn TRƯỚC hợp đồng, kỳ vọng hợp đồng thắng. Đơn vị B: `locate_batch_findings` (Rule 6 "không đổi") và dải tóm tắt (commit `fb652559`) đọc trả lời.
- Kịch bản: chỉ dẫn "Hãy trả lời bằng văn xuôi tiếng Việt, giải thích từng câu." Hợp đồng đứng sau, nhưng model vẫn có thể thắng bằng chỉ dẫn (đặc biệt model nhỏ chạy cục bộ). Trả lời không phải JSON: bộ đọc không định vị được ⇒ 0 phát hiện, lượt quét báo "không thấy lỗi trong N câu" (xem `fb652559`). Đó chính là kiểu hỏng AD-68 Prevents 1 tuyên bố ngăn, nhưng Rule 4 chỉ có BIỆN PHÁP (thứ tự), không có PHÁT HIỆN. Phương án bị loại cuối cùng ("đặt sau hợp đồng… biến mất mà không báo lỗi") tự thừa nhận rủi ro này tồn tại ở thứ tự ngược lại mà không đòi dò nó ở thứ tự đã chọn.
- Sửa AD nhỏ nhất: thêm Rule: "Lượt quét có chỉ dẫn mà một lô trả về không phân tích được thành cấu trúc JSON hợp lệ (khác với JSON hợp lệ có 0 phát hiện) được tính là lô lỗi `reply_unparseable`, tách khỏi 'không thấy lỗi', và lỗi nêu tên bộ đang dùng. Test (i): chỉ dẫn mâu thuẫn + máy trả văn xuôi ⇒ lỗi chứ không phải kết quả rỗng; đối chứng: bỏ phân biệt hai trường hợp." Nếu bộ đọc hiện nay đã phân biệt, câu này chỉ cần trỏ vào; nếu chưa, đây là việc mới cần ghi vào AD vì Rule 6 đang nói `locate_batch_findings` không đổi.

## H4 — medium: tên trùng, khác loại ⇒ va chạm tên tệp xuất, và danh sách lẫn loại

- `export_file_name(name)` chỉ nhận TÊN. Hai bộ "Chặt chẽ" loại `translate` và `proofread` cùng cho `Chặt chẽ.prompt.md`. Rule 1 nói tên duy nhất theo `(kind, name)` nên hai bộ này hợp lệ; Rule 5 nói xuất mang `kind`. Không Rule nào nói tên tệp. Xuất hàng loạt hoặc thư mục đích dùng chung ghi đè một tệp bằng tệp kia (khác nội dung, bốn dòng vs ba dòng; nhập lại ra loại này, mất loại kia, không lỗi).
- Cặp đơn vị: story làm đường xuất (Rust) và story làm giao diện chọn thư mục. Mỗi bên đều tuân Rule.
- Sửa: Rule 5 thêm "Tên tệp xuất của loại `proofread` là `<tên>.proofread.prompt.md`; `translate` giữ `<tên>.prompt.md`; nhập chấp nhận cả hai vì loại do dòng `kind:` quyết định, không do đuôi." Test: xuất cả hai bộ cùng tên ra một thư mục ⇒ hai tệp.
- Cùng họ: "Mọi đường đọc… nhận loại làm tham số" là câu chung; một đường list quên tham số (mặc định `translate` vì cột có DEFAULT) vẫn biên dịch và qua test. Phải gài ở kiểu: tham số `kind` không có giá trị mặc định, và test (e) mở rộng đếm số đường truy vấn đọc `prompt_set` không đi qua một hàm chọn loại. Thêm câu: "Không câu SQL nào đọc `prompt_set` mà thiếu `kind` trong điều kiện; một cổng nguồn quét `FROM prompt_set` canh điều đó."

## H5 — medium: `proof_scan`/`proof_decision` không biết chỉ dẫn nào sinh ra phát hiện

- AD-56 đếm quyết định để ra tỷ lệ "báo nhầm". Rule 6 giữ nguyên "cách đếm quyết định" và Rule 3 đọc chỉ dẫn "một lần mỗi lượt", nhưng `proof_scan` (một hàng mỗi lượt quét, mẫu số) không có cột nào mang tên bộ, tầng hay mã băm thân (đọc `schema.rs:1070–1133`).
- Hậu quả: người dùng đổi chỉ dẫn để giảm báo nhầm (chính là ca thúc đẩy của AD) rồi không thể đo xem có giảm không; tỷ lệ chấp nhận trộn lẫn mọi chỉ dẫn, kể cả không chỉ dẫn. Cũng không phân biệt được sau H1 kịch bản 3.
- Sửa nhỏ nhất: Rule 3 thêm "Mỗi hàng `proof_scan` ghi tên bộ, tầng, và băm của thân đã dùng (NULL nếu không có chỉ dẫn)". Sửa Rule 6: "cách đếm quyết định (AD-56) không đổi; mẫu số được cắt theo cột mới". Nếu Ice muốn hoãn, ghi thành nợ có `Chủ:` thay vì im lặng.

## H6 — medium: chi phí token vô hình

- Rule 4 đã cảnh báo chỉ dẫn gửi lại ở mỗi lô nhưng không đặt giới hạn nào, không hiện cho người dùng; `plan_batches` "vẫn chỉ đếm nguồn cộng bản dịch" nên chỉ dẫn dài làm mỗi lô vượt ngưỡng cửa sổ ngữ cảnh mà bộ chia không biết (đây là một đơn vị lập kế hoạch lô và một đơn vị lắp prompt, mỗi bên đúng Rule, và kết quả là lô bị cắt hoặc bị từ chối ở nhà cung cấp).
- Sửa: thêm "Độ dài thân bộ `proofread` bị chặn (số đề xuất do Ice chốt, nêu bằng ký tự) tại lúc tạo/sửa/nhập, và `plan_batches` trừ độ dài chỉ dẫn khỏi ngân sách mỗi lô. Màn soạn hiện số ước lượng token." Hai chỗ này đổi Rule 6 ("không đổi cách chia lô"), nên cần ghi rõ là ngoại lệ.

## H7 — low: bộ rỗng làm "bỏ chỉ dẫn" và hai tầng

- Rule 3: Work muốn bỏ chỉ dẫn Global thì tạo bộ Work thân rỗng. Nhưng bộ đó vẫn phải được CHỌN tên; nếu tên bộ rỗng trùng một bộ Global đang được chọn ở Global, bộ Work che nó (đúng ý) và việc xoá nó ở H1 kịch bản 3 làm chỉ dẫn Global bật trở lại không ai hay. Ngoài ra thân rỗng sau khi cắt khoảng trắng giữ nguyên khác biệt giữa "không chọn" và "chọn bộ rỗng" ở phía Rust nhưng UI có thể gộp hai trạng thái. Sửa: Rule 3 thêm "UI phân biệt 'không chọn' với 'đã chọn bộ rỗng' bằng nhãn riêng"; hoặc cho phép giá trị đặc biệt có tên cố định thay vì mẹo bộ rỗng (chọn cách nào là việc của Ice).

## H8 — low: nhập — luồng xung đột

- Rule 5: "Va chạm tên chỉ xét trong cùng loại". Luồng xung đột nhập hiện có (ghi đè / đổi tên / bỏ qua) phải mang `kind` trong khoá so; nếu ghi đè một bộ `proofread` đang được chọn thì chọn vẫn giữ (an toàn). Nhưng "ghi đè" khi tệp đến từ tay người khác đổi chỉ dẫn đang được chọn bằng một thân lạ mà màn xác nhận không cho xem sự khác biệt. Sửa: màn xung đột nêu "bộ này đang được chọn ở Global/Tác phẩm X" khi nó là bộ đang dùng.

## Không tìm thấy lỗ trong

- Thứ tự lắp prompt, byte-identical khi không chỉ dẫn, chèn U+200B (Rule 4 và test (a)(b)(g)): khớp AD-60 mục 4, các ca (a)–(h) đều có phép gỡ thật.
- Bộ `translate` bị nhập nhầm vào lượt quét: Rule 5 từ chối `kind:` lạ và bản cũ từ chối tệp bốn dòng (`MissingClosingDelimiter`) nên không lẫn.

## Xếp hạng

| Hạng | Lỗ |
|---|---|
| critical/high | H1 (đổi tên/xoá bộ đang được chọn, kể cả bóng che bị gỡ im lặng) |
| high | H2 (`.atproj` sang máy khác), H3 (chỉ dẫn mâu thuẫn định dạng ⇒ kết quả rỗng) |
| medium | H4 (tên tệp xuất đụng nhau, đường đọc quên `kind`), H5 (`proof_scan` không biết chỉ dẫn), H6 (token vô hình, `plan_batches`) |
| low | H7 (bộ rỗng mơ hồ), H8 (màn xung đột nhập) |

Hai lựa chọn hợp lệ cần Ice (không tự chọn): H2 (A) nhập lại bộ vs (B) sao bộ vào `project.db`; H7 bộ rỗng vs giá trị "không chỉ dẫn" riêng.
