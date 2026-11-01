# Review UX: mục `## Dịch nhanh` (EXPERIENCE.md) so với AD-64, FR153–FR158 và mã hiện tại

Phạm vi đọc: EXPERIENCE.md `## Dịch nhanh` (dòng 381–443), `## Voice and Tone`, `## Accessibility Floor`; DESIGN.md `Câu dịch tham khảo` (dòng 533); AD-64 (kiến trúc dòng 1498); PRD `#### Dịch nhanh` (FR153–FR158) cùng NFR17, NFR20; mã `GlossaryQueueOverlay.vue` (`.gq-ref`), `ImportPreviewOverlay.vue` (`ip-block`, `onTier2Keydown`), `SettingsOverlay.vue` (mục AI, thanh phạm vi), `SettingsDataSection.vue`, `settingsState.ts`, `aiConfigState.ts`. Tất cả là đọc tĩnh, chưa chạy app.

Kết luận: mục đúng hướng, nhưng thiếu 6 trạng thái AD-64 đòi, có 3 chỗ mô tả không khớp mã, và 1 lỗ phím tắt thật (J/K chết khi nút mới giữ tiêu điểm). Chưa nên ký.

## A. Mâu thuẫn và tuyên bố không khớp mã (nặng nhất trước)

**A1. `J` `K` sẽ chết khi nút mới giữ tiêu điểm; `Space` đổi nghĩa.** Mục nói "`J` `K` và `Space` giữ nguyên hành vi. Câu dịch không nhận tiêu điểm riêng". Nhưng `onTier2Keydown` (`ImportPreviewOverlay.vue` ~dòng 662–668) `return` sớm khi `event.target` là `HTMLButtonElement`. Hai nút Giữ/Loại tránh được vì có `@mousedown.prevent`, nên chuột không bao giờ dời tiêu điểm vào đó. Nút mới `Dịch các khối đang thấy`, `Đang dịch N khối`, `Huỷ`, `Dịch lại` không có bảo đảm đó. Bấm chuột hoặc `Tab` vào là tiêu điểm nằm trên nút: `J` `K` không làm gì, `Space` bấm lại nút (dịch lại, tốn tiền) thay vì đảo Giữ/Loại. Mục cần chốt: nút mới dùng `@mousedown.prevent` như hai nút cũ, hoặc tiêu điểm trả về `.ip-blocks` sau khi bấm, và nói rõ `Tab` tới nút thì `J` `K` tạm nhường nút. Khi chưa chốt, câu "giữ nguyên" là sai.

**A2. "Hàng nút nằm ở đầu khối, phía trên phần mọc" đúng nhưng chỉ đủ cho khối đang chọn.** Mã: `.ip-block-head` (nhãn trạng thái + `.ip-block-actions`) đứng trước `.ip-block-text`, và `v-if="index === importPreviewBlockFocusedIndex"` chỉ vẽ hàng nút ở khối con trỏ. Câu dịch của chính khối đó mọc xuống dưới thì nút không dời. Nhưng câu dịch của khối đứng TRÊN nó, vẫn trong vùng nhìn, mọc ra thì đẩy cả khối con trỏ (và nút dưới chuột) xuống. Mục chỉ xử lý "khối ngoài vùng nhìn". Cần: neo cuộn theo khối con trỏ, hoặc chỉ cho chiều cao mọc khi khối ở dưới con trỏ, hoặc chừa sẵn chiều cao. Ngoài ra `j`/`k` đổi `v-if` của hàng nút nên chính nó đã làm nhảy; không dùng làm tiền lệ.

**A3. Khối đã loại: câu dịch và độ lùi mâu thuẫn.** Mục: "Khối đã loại vẫn dịch ... Độ lùi của khối giữ nguyên". Mã: `.ip-block-dropped` nền `surface-sunken`, chữ nguồn rút về `on-surface-variant` (ui-md-wrap). DESIGN.md `Câu dịch tham khảo` buộc câu dịch `read-sm` màu `on-surface`. Kết quả: dưới một khối đã loại, câu dịch đậm hơn chữ gốc, ngược nguyên tắc "phân biệt bằng độ lùi" (§Always 6.9). Cần chốt màu câu dịch trong khối loại (đề xuất `on-surface-variant`) và ghi vào DESIGN.md.

**A4. Mô tả bộ chọn tầng ở Cài đặt không đúng cấu trúc thật.** Mục: "Khối này không theo bộ chọn tầng của mục". Mã: bộ chọn tầng là một thanh phạm vi (`.set-scope`, `SettingsOverlay.vue` 354–392) nằm TRÊN lưới, chung cho cả cửa sổ, bật khi `settingsSectionHasWorkTier` (chỉ `ai_and_model`). Khi đang ở tầng Tác phẩm thanh này viết `settings.scope.msg_work`: "Đang sửa Tác phẩm đang mở — thay đổi ở đây không chạm tới Tác phẩm khác", kèm bộ đếm ghi đè và chord `clear_all_overrides`. Ba hệ quả mục chưa nói:
- Khối `Dịch nhanh` nằm dưới thông điệp đó mà không theo nó. Người đọc hiểu nhầm là sửa theo Tác phẩm. Cần dòng phụ `Dùng chung mọi Tác phẩm` đủ nổi, hoặc ẩn thông điệp ở khối này.
- `Xoá mọi ghi đè` và bộ đếm ghi đè không được tính cấu hình nhanh.
- `aiConfigState` giữ nháp, lỗi lưu, kết quả `Thử kết nối` theo từng tầng và chỉ MỘT `testState`. Khối nhanh cần nháp, lỗi lưu và kết quả thử riêng, và `Thử kết nối` của nó không được chia sẻ trạng thái chạy với nút chính (hiện `selectAiConfigViewTier` còn gọi `resetAiConfigTest()`).

**A5. "Để trống thì bấm tay dùng cấu hình chính" mơ hồ khi đang ở tầng Tác phẩm.** AD-64 mục 2 và FR153: để trống thì dùng trọn cấu hình chính ở tầng TOÀN CỤC, không theo ghi đè Tác phẩm. Màn Cài đặt đang ở tầng Tác phẩm lại hiện các ô chính với giá trị ghi đè. Người dùng đọc "cấu hình chính" là thứ họ thấy trên màn, mà thực tế là thứ khác (nhà cung cấp, endpoint, model, khoá đều khác). Dòng phụ nên nêu "cấu hình chính ở tầng Toàn cục", và khi có ghi đè Tác phẩm thì nói thẳng chúng không áp cho dịch nhanh.

**A6. Mất lời nói "tham khảo" ở Bảng chờ Glossary.** Mục 385: "Mọi bề mặt phải nói rõ điều đó"; FR154: "ghi rõ là tham khảo". Mã hiện có `glossary.queue.context_source_ai`: "AI dịch để tham khảo, chưa vào văn bản". Bảng khuôn mới thay dòng nguồn bằng `<mô hình> · Dịch lại`, và tầng khối có dòng đầu tầng, còn Bảng chờ không có chỗ nào mang chữ "tham khảo". Cần giữ nhãn hoặc thêm vào dòng nguồn (`<mô hình> · tham khảo · Dịch lại`) hoặc lên nhãn khối `Ngữ cảnh`. Màu: `gq-ref-text` đã `on-surface`, khớp DESIGN.

**A7. Mã nguồn đang có `Dịch thử` khi tự dịch bật ở trạng thái chờ.** Mục: tự dịch bật thì `Dịch thử` "không xuất hiện". Nhưng giữa lúc con trỏ dừng và lúc tự chạy bắn (AD-64 mục 8 có thời gian chờ), bảng không hiện gì (trạng thái `Đang tra` "không hiện gì"). Nếu cổng bị chặn (xem B2, B3) thì khoảng trống đó thành vĩnh viễn: đúng lớp lỗi "rỗng im lặng" ở AGENTS.md. Xem B2.

## B. Trạng thái và luật AD-64/FR153–158 chưa có chỗ trong mục

| # | Luật nguồn | Thiếu gì | Đề xuất |
|---|---|---|---|
| B1 | AD-64 mục 1 `auto_off`, FR156 | Tự dịch bật nhưng Rust trả `auto_off` (tuỳ chọn tắt ở cửa sổ khác, cấu hình bị xoá) thì hàng hiện gì? Bảng chỉ có trạng thái "tự dịch tắt" và "tự dịch bật" | Về trạng thái `Dịch thử` như tự dịch tắt; không báo lỗi |
| B2 | FR156 "không bắt đầu khi một lô dịch thật đang chạy" | Tự dịch bị chặn vì lô chính. Người dùng thấy hàng trống, không có `Dịch thử`, không có lý do | Hiện `Đang chờ lô dịch xong.` hoặc hiện lại `Dịch thử` kèm lý do; chọn một, ghi cả việc bấm tay có được phép khi lô chính chạy không (AD-64 chỉ chặn tự chạy) |
| B3 | AD-64 mục 8 chờ con trỏ dừng | Trạng thái "đã vào hàng, đang chờ bắn" không có. Có thể trùng B2 | Một trạng thái chờ im lặng được, nhưng phải có thời hạn trên và lối ra là `Dịch thử` |
| B4 | AD-64 mục 5 `maybe_truncated` | Bản dịch cắt ngang hiển thị thế nào? Nó KHÔNG được lưu, nên mở lại hàng là gọi lại và tính tiền lại. Bảng chỉ có "xong" và "lỗi" | Thêm hàng: câu hiện, `Có thể bị cắt, chưa lưu` và nút `Dịch thử`; tự dịch coi như đã thử (như B8) |
| B5 | AD-64 mục 5, mục 7; FR157 | Lô khối thiếu một phần: một vài khối đóng đủ và lưu, các khối còn lại thiếu hoặc lỗi. Bảng gom vào "Lỗi hoặc thiếu kết quả" nhưng bề mặt là một dòng đầu tầng, không phải từng khối | Quy định: dòng đầu tầng nêu `Đã dịch N/M khối`, khối thiếu hiện `Dịch thử` riêng; huỷ giữa chừng thì khối đã đóng giữ chữ trên màn |
| B6 | AD-64 mục 10, FR158 "dung lượng giảm thật" | `VACUUM` lỗi: hàng đã xoá vẫn xoá, nhưng câu mẫu luôn nói "lấy lại 2,1 MB" | Thêm câu: `Đã xoá 1.204 câu; chưa lấy lại được dung lượng.` kèm `tError`. Số liệu nên đọc lại từ kho, không tính từ số hàng |
| B7 | AD-64 mục 3 mở kho thất bại | Mục chỉ có "Kho lỗi thì hiện `tError` và hai nút tắt" ở Cài đặt. Ở hai bề mặt dịch, mọi `Dịch thử` đều lỗi. AD nói "dịch nhanh báo lỗi", nhưng nút vẫn bấm được | Bề mặt: tắt `Dịch thử` và `Dịch các khối` kèm lý do; Cài đặt: công tắc `Tự dịch` cũng tắt |
| B8 | FR156, AD-64 mục 9 | "Tự dịch bỏ qua chỗ này trong phiên" không có dấu hiệu nào cho người dùng, cũng không nói dấu có hết khi đóng màn hay đổi tuỳ chọn | Ghi vòng đời: hết khi đóng màn; đổi `Tự dịch` tắt rồi bật không xoá |
| B9 | AD-64 mục 8 `busy`, huỷ theo `run_id` | Khi nào hai bề mặt cùng sống? Đóng lớp phủ đã huỷ lượt, nên `busy` chỉ lộ trong khoảng huỷ chưa kịp, hoặc khi Cài đặt mở trên một bề mặt. Hàng `Đang bận` không nói nút chờ bao lâu, và "tự bật lại khi lượt kia xong" cần ai nghe sự kiện. Cũng thiếu: lượt bị thay cùng bề mặt (đổi hàng) thì chữ đang chảy biến mất thế nào | Chốt kịch bản thật của `busy` hoặc bỏ hàng; mô tả chuyển hàng khi đang chạy: chữ cũ dừng, hàng mới tự chạy |
| B10 | AD-64 mục 10 | Dọn huỷ lượt đang chạy ở bề mặt phía dưới. Bề mặt đó hiện gì sau huỷ do Dọn (không phải do người dùng bấm `Huỷ`)? Mục không nói | Về `Dịch thử` không báo lỗi; hoặc một dòng `Đã huỷ vì dọn dữ liệu` |
| B11 | FR153: khai một trong hai thì báo chưa cấu hình | Ở bề mặt dịch, "Chưa cấu hình" có hai nguyên nhân (trống hoàn toàn rồi cấu hình chính cũng chưa có; hoặc khai nửa). Cả hai dùng một câu `Chưa cấu hình AI...` và một nút `Mở Cài đặt AI`. Nửa cấu hình cần nói `Cấu hình dịch nhanh thiếu endpoint hoặc mô hình.` | Tách hai câu |
| B12 | AD-64 mục 4 "cắt ở 200 ký tự là chuỗi riêng" | Ví dụ ngữ cảnh bị cắt: câu dịch của chuỗi cắt hiện dưới `Ngữ cảnh` như dịch cả câu. Người đọc không biết đó là dịch phần đã cắt. Có thể gây nhầm khi quyết nhận/bỏ | Nêu trong dòng nguồn khi `context_example` đã bị cắt (có `…`), hoặc chấp nhận và ghi vào spec |
| B13 | FR157 "ghi rõ mô hình" | Mô hình nằm trong dòng nguồn nhưng khi mô hình trong cấu hình đã đổi, nhãn cũ vẫn hiện với `Dịch lại` kề; chưa có chỉ dẫn `Dịch lại` sẽ dùng mô hình hiện tại | Một tooltip hoặc hậu tố `→ <mô hình hiện tại>` nếu khác |
| B14 | FR76 và cài đặt hiện có | `aiUsageVisible` (Cài đặt AI, `data-ai-usage-toggle`) cho người dùng tắt dòng token/chi phí. Mục "Xong lượt vừa chạy" ép hiện số token và chi phí mà không nhắc cài đặt này | Ghi: theo `aiUsageVisible`; hoặc nói rõ dịch nhanh bỏ qua |
| B15 | FR158 "dọn quá một số ngày" | Ô số: không nói giá trị không hợp lệ (0, âm, rỗng), nút `Dọn` khi ô sai, và khi `Sẽ xoá 0 câu.`. Nút `Dọn câu đã dịch hơn [30] ngày` không có bước xác nhận như `Dọn tất cả` dù cũng không hoàn tác được | Quy tắc ô số; xác nhận hai bước cho cả hai nút, hoặc giải thích vì sao không cần |
| B16 | FR156 "tắt tuỳ chọn thì dừng ngay" | Công tắc ở Cài đặt, nhưng bề mặt dịch đang mở bên dưới: dòng đang chạy dừng thế nào, giữ chữ đã chảy hay xoá? FR157 chỉ cứu khối đã đóng đủ | Chốt: chữ dở bị bỏ, hàng về `Dịch thử` |
| B17 | FR155 | Màn xem trước khi `kept`/khối ảnh: "Khối ảnh không dịch" nhưng chú thích ảnh (caption) thì có. Chưa nói khối `alt` | Ghi rõ alt không dịch |

## C. Tiếp cận (NFR17, NFR20, Accessibility Floor)

- **C1. Đường phím chưa đủ cho NFR17.** Mục cuối: "Không có phím tắt cho `Dịch thử` và `Tự dịch`... vào bằng `Tab` theo NFR17". NFR17 buộc mọi thao tác làm được hoàn toàn bằng phím và NFR20 buộc mọi chức năng độc lập có đường chuột. `Tab` đạt NFR17 về lý thuyết, nhưng trong Bảng chờ Glossary hàng đang mở có phím riêng (`ArrowDown`..., `1`–`4`), nên tới nút `Dịch thử` bằng `Tab` đi qua bao nhiêu điểm dừng? Cần đếm. Và Accessibility Floor mục 1: mọi thao tác phải qua `CommandRegistry`. Mã đã có `glossary.context.translate` và `glossary.context.cancel`; mục cần liệt kê command id cho `Dịch các khối`, `Huỷ`, `Dịch lại`, `Dọn`, `Tự dịch` (AD-34 và `check:commands` Kiểm A canh `@click`).
- **C2. `Esc` đang đóng cả màn.** Bảng chờ Glossary: `@keydown.esc` → `glossary.queue.close`. Màn xem trước: `onEscapeCancel`. Không có đường phím để huỷ một lượt dịch mà không đóng màn (và đóng màn thì huỷ lượt, AD-64 mục 8, nên người dùng mất cả công việc nhập URL). Mục cần chốt `Huỷ` bằng phím nào, hoặc `Esc` lần đầu huỷ lượt rồi mới đóng. Đây là trường hợp NFR17 thật.
- **C3. Vùng sống cho chữ chảy.** Accessibility Floor để ngoài phạm vi v1 "hỗ trợ trình đọc màn hình (ARIA đầy đủ)", nên không có luật buộc `aria-live`. Nhưng mã hiện có đã dùng `role="status"` (`gq-status`, `ip-status`) và `role="alert"` (lỗi `gq-ref`). Đề xuất để mục quyết một lần: KHÔNG đặt `aria-live` lên chữ đang chảy (đọc lại từng token là hỏng), chỉ đặt `role="status"` lên dòng nguồn đổi `Đang dịch…` → `Xong` / số token. Hiện `.gq-ref-source` không có `role`. Nếu muốn nhất quán với `role="alert"` ở lỗi, ghi vào mục.
- **C4. Tiêu điểm khi nút biến mất.** `Dịch thử` bị thay bằng `Huỷ` rồi bằng dòng nguồn + `Dịch lại` trong cùng chỗ. Nếu tiêu điểm đang ở `Dịch thử` rồi nút bị gỡ khỏi DOM, tiêu điểm rơi về `body`, vi phạm Floor mục 2 ("không chế độ nào để focus rơi về body"). Mục cần quy định tiêu điểm đi đâu (đề xuất: nút thay thế cùng vị trí, hoặc trả về hàng/khối).
- **C5. Nút `Huỷ` khi đang chạy cần là phần tử được `Tab`.** `Huỷ` chỉ xuất hiện trong lúc chạy; nếu lượt xong trước khi người dùng tới nó thì mất.
- **C6. Công tắc.** DESIGN.md định nghĩa `role="switch"` (`ui-switch`). Cài đặt hiện dùng nút `aria-pressed` cho `aiUsageVisible`. Mục nói "công tắc" mà không chỉ rõ dùng loại nào; theo DESIGN thì `ui-switch` với chữ trạng thái cạnh bên. Công tắc `Tự dịch` đang tắt cưỡng bức (khi cấu hình nhanh trống) cần chữ lý do cạnh bên, không chỉ `disabled` (bảng đã có, tốt) và cần `aria-describedby` nếu làm.
- **C7. Xác nhận hai bước của `Dọn tất cả`.** Nút đổi nhãn tại chỗ: tiêu điểm phải ở lại nút, và `Thôi` phải với tới bằng `Tab`; `Esc` nên là `Thôi` nhưng `Esc` ở Cài đặt có thể đóng cả cửa sổ. Chưa quy định.

## D. Giọng văn (so với `## Voice and Tone`)

Phần lớn đạt: câu vô nhân xưng, nêu hệ quả, số liệu thật (`Đã xoá 1.204 câu, lấy lại 2,1 MB. Lần sau dịch lại sẽ gọi AI.` là mẫu tốt).

- `Cần cả endpoint và mô hình.` nói thiếu gì nhưng không nêu hệ quả. Nên nối: `...; chưa khai đủ thì Tự dịch tắt.` (chỉ khi đúng thực tế).
- `Để trống thì bấm tay dùng cấu hình chính. Tự dịch cần cấu hình riêng.` Đúng ý, nhưng "bấm tay" là từ lóng so với phần còn lại; đã dùng ở EXPERIENCE.md chỗ khác? Nếu chưa thì đổi `Dịch thử dùng cấu hình chính`. Và xem A5.
- `Đang bận` / `Đang dịch ở màn khác.`: ổn, nhưng không nêu hệ quả (chờ hay bấm được). Thêm `Tự chạy khi xong.`
- `Xoá 12.480 câu?`: nút `Dọn tất cả` đổi thành nhãn bắt đầu bằng `Xoá`. Theo DESIGN bậc Nguy hiểm dành cho "chữ **Xoá** làm mất thứ không lấy lại được", nên thống nhất: nút đầu nên đã ở bậc Nguy hiểm hoặc đổi dáng khi xác nhận. Nhãn `Dọn tất cả` và `Dọn câu đã dịch hơn` lệch với từ `Xoá` ở bước xác nhận; chọn một động từ.
- `Dịch thử` (nhãn nút, theo PRD) và `Dịch lại` và `Dịch các khối đang thấy`: ba động từ khác nhau cho gần cùng việc là chấp nhận được, nhưng `Dịch thử` mang nghĩa "thử" nên ở tầng khối nó đổi thành `Dịch các khối đang thấy`, lệch thuật ngữ PRD §5.2 (`Dịch nhanh` / `Dịch thử`). Ghi vào bảng thuật ngữ nếu giữ.
- Chuỗi mới phải là khoá `vi.json` (NFR16, AD-21); mục không liệt kê khoá. `Xong lượt vừa chạy` dựa `aiUsageLine` (AD-64 mục 11) nên khoá đã có.

## E. Điểm nhất quán nhỏ với AD-64/PRD

- EXPERIENCE nói "Chưa có, tự dịch tắt: nút `Dịch thử`" nhưng bảng không có hàng "Chưa có, tự dịch bật, chưa bắn" (A7/B3).
- "Cả ứng dụng chỉ một lượt dịch nhanh" (FR156) đúng; mục `Đang bận` khớp, nhưng cần chốt B9.
- Ghi chú "Ngoài giao diện này": "lệnh đã đăng ký (AD-34)" đúng với `glossary.context.translate`; chưa có lệnh cho các nút mới (C1).
- Mục không mâu thuẫn FR71/FR72: không có nút đưa sang Editor, đúng.
- Kho trống: `Chưa có câu nào.` + hai nút tắt khớp. Không nêu `Dọn quá N ngày` khi tuổi không có hàng nào (B15).

## F. Việc đề nghị trước khi ký

1. Chốt A1 (tiêu điểm nút mới) và C2 (Huỷ bằng phím): hai thứ này làm hỏng bàn phím thật.
2. Thêm hàng bảng B1, B2/B3, B4, B5, B7 và câu B6.
3. Sửa A4/A5: mô tả đúng thanh phạm vi và nói "Toàn cục" trong dòng phụ.
4. Sửa A3 (màu trong khối loại) và A6 (giữ chữ "tham khảo" ở Bảng chờ).
5. Quy định tiêu điểm khi nút thay thế (C4) và `role="status"` ở dòng nguồn (C3).
