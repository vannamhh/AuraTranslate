import type { GenreId } from './modes/libraryWorkCard'

const GUIDANCE: Record<GenreId, string> = {
  news: `Đây là tin tức thời sự. Văn phong báo chí, câu ngắn, trung tính, không thêm bình luận của người dịch.
Tên người và địa danh Trung Quốc theo Hán Việt, tên phương Tây giữ nguyên dạng gốc.
Chức danh, cơ quan, tổ chức dịch theo cách báo Việt Nam vẫn dùng.
Số liệu, ngày tháng, đơn vị tiền tệ viết theo kiểu Việt Nam.`,
  commentary: `Đây là bình luận, chính luận. Giữ giọng lập luận của tác giả, không làm nhẹ hay làm gắt hơn nguyên văn.
Thuật ngữ chính trị, kinh tế dùng cách gọi đã quen trong tiếng Việt; khẩu hiệu và thành ngữ có thể dịch thoáng nhưng phải giữ ý.
Tên người và địa danh Trung Quốc theo Hán Việt.
Giữ nguyên các câu hỏi tu từ và phép đối nếu tiếng Việt diễn đạt được.`,
  tcm: `Đây là văn bản Đông y. Thuật ngữ y học cổ truyền giữ Hán Việt: kinh lạc, khí huyết, âm dương, ngũ hành, tạng phủ.
Tên vị thuốc, bài thuốc, huyệt vị dùng tên Hán Việt quen thuộc, không dịch nghĩa đen.
Liều lượng và đơn vị đo cổ (lượng, tiền, thăng) giữ nguyên, không quy đổi.
Văn phong rõ ràng, chính xác, tránh diễn đạt có thể bị hiểu như lời khuyên chữa bệnh mà nguyên văn không có.`,
  health: `Đây là văn bản sức khỏe, dưỡng sinh. Văn phong gần gũi, dễ hiểu, như lời khuyên cho người đọc phổ thông.
Thuật ngữ y học hiện đại dùng tên tiếng Việt chuẩn; khái niệm dưỡng sinh cổ truyền giữ Hán Việt (khí, tinh, thần).
Giữ nguyên mọi con số: liều lượng, thời gian, tần suất.
Không thêm lời cảnh báo hay khẳng định mà nguyên văn không có.`,
  cuisine: `Đây là văn bản ẩm thực. Tên món ăn Trung Hoa dịch nghĩa hoặc giữ Hán Việt theo cách phổ biến (há cảo, sủi cảo, lẩu); món không có tên quen thì dịch nghĩa kèm giữ tên gốc.
Tên nguyên liệu, gia vị, kỹ thuật nấu (xào, hầm, hấp, om) dùng đúng từ bếp Việt.
Định lượng và nhiệt độ giữ đúng số trong nguyên văn.
Văn phong sinh động, gợi hương vị nhưng không thêm chi tiết mới.`,
  family: `Đây là văn bản giáo dục, gia đình. Văn phong ấm áp, thân tình, dễ đọc.
Xưng hô theo quan hệ trong nguyên văn: cha mẹ với con, thầy với trò, vợ với chồng; chọn cách gọi tự nhiên của gia đình Việt.
Thuật ngữ giáo dục và tâm lý dùng cách gọi chuẩn trong tiếng Việt.
Tên cấp học, kỳ thi, trường lớp dịch theo tương đương gần nhất ở Việt Nam, lần đầu có thể kèm tên gốc.`,
  travel: `Đây là văn bản du lịch. Địa danh Trung Quốc theo Hán Việt, địa danh nước ngoài dùng tên tiếng Việt quen thuộc hoặc giữ nguyên.
Tên di tích, chùa, núi, sông dịch theo Hán Việt (Vạn Lý Trường Thành, Tây Hồ).
Khoảng cách, giá vé, giờ mở cửa giữ đúng số trong nguyên văn.
Văn phong sinh động, gợi cảnh nhưng không thêm chi tiết nguyên văn không có.`,
  culture: `Đây là văn bản về văn hóa truyền thống. Giữ Hán Việt cho khái niệm, lễ nghi, tiết khí, điển tích (Nho, Đạo, Thiền, lễ, nhạc).
Thành ngữ và điển cố giữ lối diễn đạt cổ, khi cần thì chú nghĩa ngắn gọn trong ngoặc.
Tên người, sách, triều đại theo Hán Việt.
Văn phong trang trọng, điềm đạm, tránh từ lóng hiện đại.`,
  history_figures: `Đây là văn bản về nhân vật lịch sử. Tên người, tự, hiệu, thụy hiệu, niên hiệu, địa danh theo Hán Việt.
Chức quan và danh xưng dùng đúng thuật ngữ đã quen (thừa tướng, thái thú, tiết độ sứ).
Năm tháng âm lịch và niên hiệu giữ nguyên cách ghi, không tự quy đổi sang dương lịch.
Văn phong nghiêm cẩn như sử sách, giữ khoảng cách khách quan với nhân vật.`,
  spiritual: `Đây là văn bản tâm linh, kỳ văn. Thuật ngữ tôn giáo, huyền học giữ Hán Việt (nhân quả, luân hồi, âm đức, thần thức, pháp sự).
Tên thần, phật, quỷ, thánh địa theo Hán Việt.
Giữ không khí huyền bí của nguyên văn, không giải thích hay phủ nhận điều được kể.
Lời thoại của người trong cuộc giữ giọng kể chân thật, không bóng bẩy thêm.`,
  arts: `Đây là văn bản nghệ thuật. Thuật ngữ hội họa, thư pháp, âm nhạc, kiến trúc, sân khấu dùng từ chuyên ngành tiếng Việt; trường phái và kỹ thuật cổ giữ Hán Việt.
Tên tác phẩm dịch nghĩa, lần đầu có thể kèm tên gốc trong ngoặc.
Tên nghệ sĩ Trung Quốc theo Hán Việt, tên nước ngoài giữ nguyên.
Văn phong giàu cảm thụ nhưng chính xác về kỹ thuật.`,
  business: `Đây là văn bản kinh tế, kinh doanh. Thuật ngữ tài chính, kế toán, quản trị dùng cách gọi chuẩn trong tiếng Việt.
Tên công ty, thương hiệu, sản phẩm giữ nguyên dạng gốc; tên công ty Trung Quốc có thể kèm Hán Việt lần đầu.
Số liệu, tỷ lệ, đơn vị tiền tệ giữ chính xác, không làm tròn.
Văn phong rõ ràng, chuyên nghiệp, câu gọn.`,
  science: `Đây là văn bản khoa học, kỹ thuật. Thuật ngữ chuyên ngành dùng đúng tên chuẩn trong tiếng Việt; chưa có tên chuẩn thì giữ thuật ngữ gốc kèm nghĩa lần đầu.
Ký hiệu, công thức, đơn vị đo, tên riêng của định luật hay phương pháp giữ nguyên.
Văn phong khách quan, chính xác, ưu tiên câu rõ ràng hơn câu hay.
Không diễn giải thêm điều nguyên văn không nói.`,
  legal: `Đây là văn bản pháp lý. Dịch sát nghĩa, thuật ngữ pháp lý dùng đúng từ chuẩn của luật Việt Nam (hợp đồng, bên A, điều khoản, chế tài).
Giữ nguyên cấu trúc điều, khoản, điểm và số thứ tự.
Một từ thuật ngữ chỉ dịch một cách xuyên suốt, không đổi từ đồng nghĩa.
Không làm mềm hay đơn giản hóa nghĩa vụ, quyền lợi, điều kiện; giữ nguyên các từ chỉ mức độ như phải, có thể, không được.`,
  nonfiction: `Đây là văn bản phi hư cấu. Văn phong mạch lạc, thuyết phục, đúng giọng tác giả (tự sự, phân tích hay phổ biến kiến thức).
Khái niệm then chốt dịch nhất quán từ đầu đến cuối.
Tên người, tổ chức, tác phẩm trích dẫn giữ đúng dạng đã quen, tên Trung Quốc theo Hán Việt.
Số liệu và trích dẫn giữ nguyên, không diễn giải lại.`,
  xianxia: `Đây là truyện tiên hiệp. Giữ Hán Việt cho cảnh giới, công pháp, pháp bảo, tông môn (Trúc Cơ, Kim Đan, Nguyên Anh, Độ Kiếp).
Xưng hô cổ phong: ta, ngươi, tại hạ, sư huynh, sư muội, tiền bối, đạo hữu.
Văn phong trang trọng, hào hùng khi chiến đấu, tránh từ hiện đại.
Tên chiêu thức và địa danh theo Hán Việt, ngắt nhịp gọn.`,
  xuanhuan: `Đây là truyện huyền huyễn. Giữ Hán Việt cho hệ thống tu luyện, cấp bậc, thần thú, vũ kỹ và địa danh; tên chiêu thức ngắn gọn, mạnh.
Xưng hô theo thân phận và thế lực: ta, ngươi, các hạ, đại nhân, thiếu chủ.
Văn phong nhanh, giàu khí thế, nhất là cảnh chiến đấu và đột phá.
Cấp bậc và danh hiệu dịch nhất quán, không đổi cách gọi giữa chừng.`,
  wuxia: `Đây là truyện kiếm hiệp. Giữ Hán Việt cho môn phái, võ công, binh khí, nội công, chiêu thức (Thiếu Lâm, Võ Đang, Cửu Âm Chân Kinh).
Xưng hô giang hồ: tại hạ, các hạ, huynh đài, cô nương, đại hiệp, sư phụ.
Văn phong cổ điển, súc tích, nhịp câu theo cảnh đấu; lời thoại mang khí chất hiệp khách.
Tránh từ hiện đại và từ lóng.`,
  urban: `Đây là truyện đô thị hiện đại. Văn phong đời thường, tự nhiên, ít Hán Việt, ưu tiên từ thuần Việt.
Xưng hô theo tuổi và quan hệ trong bối cảnh hiện đại: anh, chị, em, cậu, tớ, sếp, đồng nghiệp.
Lời thoại ngắn, khẩu ngữ, giữ sắc thái hài hước hay mỉa mai của nguyên văn.
Tên người Trung Quốc theo Hán Việt, tên thương hiệu và địa điểm hiện đại giữ nguyên.`,
  romance: `Đây là truyện ngôn tình. Văn phong mềm mại, giàu cảm xúc, chú ý sắc thái yêu, giận, ghen, ngượng trong lời thoại.
Xưng hô thay đổi theo mức thân mật: anh, em, chàng, nàng, tên gọi thân mật; chọn đúng lúc, nhất quán trong cả đoạn.
Hán Việt vừa phải, chỉ dùng khi bối cảnh cổ trang.
Giữ nguyên nhịp nội tâm, không làm lời thoại sến hơn nguyên văn.`,
  historical: `Đây là truyện lịch sử, cổ đại. Tên người, chức quan, địa danh, niên hiệu theo Hán Việt.
Xưng hô theo phẩm cấp: bệ hạ, thần, vi thần, điện hạ, hoàng hậu, bản cung; không dùng từ hiện đại.
Văn phong trang trọng, gần cổ văn nhưng vẫn dễ đọc.
Giữ nguyên cách ghi năm tháng theo niên hiệu, không quy đổi.`,
  scifi: `Đây là truyện khoa huyễn. Thuật ngữ khoa học viễn tưởng dùng tên chuẩn hoặc quen thuộc trong tiếng Việt; khái niệm bịa ra thì dịch nhất quán một lần rồi giữ.
Tên công nghệ, tàu, tổ chức dịch nghĩa gọn, tên riêng giữ nguyên.
Văn phong rõ ràng, chính xác ở đoạn kỹ thuật, nhịp nhanh ở đoạn hành động.
Đơn vị, số liệu giữ nguyên.`,
  game: `Đây là truyện võng du, game. Thuật ngữ game giữ cách gọi cộng đồng người chơi Việt (boss, cấp, kỹ năng, nhiệm vụ, hồi chiêu, vật phẩm); tên kỹ năng và trang bị có thể dịch nghĩa hoặc giữ Hán Việt cho gọn.
Thông báo hệ thống, thuộc tính, chỉ số giữ đúng số và định dạng.
Lời thoại người chơi mang khẩu ngữ, xưng hô như bạn bè trong game: tớ, cậu, ông, tôi.
Văn phong nhanh, đậm không khí chiến đấu.`,
  horror: `Đây là truyện linh dị, kinh dị. Giữ không khí rợn người: câu ngắn đúng lúc căng, câu dài khi tả nỗi sợ âm ỉ.
Thuật ngữ tâm linh giữ Hán Việt (oan hồn, âm khí, quỷ vực, trấn yểm).
Không giải thích hay làm dịu cảnh đáng sợ; giữ nguyên chi tiết gợi hình.
Lời thoại nhân vật hoảng hốt giữ nhịp đứt quãng của nguyên văn.`,
  mystery: `Đây là truyện trinh thám. Dịch chính xác từng manh mối, mốc thời gian, địa điểm, tên người; không đổi cách gọi một nhân vật hay vật chứng giữa chừng.
Thuật ngữ điều tra, pháp y, pháp luật dùng từ chuẩn tiếng Việt.
Không để lộ hay gợi ý thêm điều nguyên văn chưa nói.
Văn phong gọn, lạnh, đủ căng cho phần suy luận.`,
  fanfic: `Đây là truyện đồng nhân. Tên nhân vật, địa danh, kỹ năng, tổ chức của tác phẩm gốc dùng đúng cách gọi đã quen với người hâm mộ ở Việt Nam; chưa có thì theo Hán Việt hoặc giữ nguyên tên gốc.
Giữ giọng điệu và cách xưng hô đặc trưng của từng nhân vật như trong tác phẩm gốc.
Một nhân vật chỉ có một cách gọi xuyên suốt.
Chú thích hoặc nhắc tới tác phẩm gốc trong nguyên văn thì giữ nguyên.`,
  literary: `Đây là văn học. Ưu tiên giọng văn, nhịp câu và hình ảnh của tác giả hơn việc dịch từng chữ.
Ẩn dụ, biểu tượng, câu nhiều tầng nghĩa giữ nguyên tính mơ hồ, không giải thích cho rõ.
Hán Việt dùng vừa phải, theo không khí tác phẩm.
Lời thoại hợp với tuổi, nghề, hoàn cảnh nhân vật; tên người Trung Quốc theo Hán Việt.`,
  children: `Đây là văn bản cho thiếu nhi. Dùng từ đơn giản, câu ngắn, giọng vui tươi, dễ nghe khi đọc thành tiếng.
Tránh Hán Việt khó và từ trừu tượng; có từ thuần Việt thì dùng từ đó.
Tên nhân vật dễ đọc; tên động vật, đồ vật dùng cách gọi quen thuộc với trẻ em Việt Nam.
Giữ vần, điệp từ, tiếng kêu và từ tượng thanh nếu tiếng Việt diễn đạt được.`,
  other: `Chưa rõ thể loại cụ thể, nên giữ đúng giọng và mức trang trọng của nguyên văn.
Tên người và địa danh Trung Quốc theo Hán Việt, tên nước ngoài giữ nguyên dạng gốc.
Thuật ngữ chuyên ngành dịch nhất quán từ đầu đến cuối.
Không thêm bớt ý, không giải thích điều nguyên văn không nói.`,
}

function frame(guidance: string): string {
  return `${guidance}

Thuật ngữ bắt buộc (trống thì bỏ qua):
{{glossary_terms}}

Câu tương tự đã dịch, tham khảo văn phong (trống thì bỏ qua):
{{tm_similar_segments}}

Đoạn nguồn (tiếng Trung hoặc tiếng Anh):
{{source_segment}}`
}

export const PROMPT_TEMPLATES: Record<GenreId, string> = Object.fromEntries(
  (Object.entries(GUIDANCE) as [GenreId, string][]).map(([id, text]) => [id, frame(text)]),
) as Record<GenreId, string>
