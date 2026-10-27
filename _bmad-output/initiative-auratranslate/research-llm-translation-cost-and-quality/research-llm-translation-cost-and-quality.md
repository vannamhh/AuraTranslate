---
title: 'technical research: LLM translation cost and quality (zh→vi, en→vi)'
type: 'technical'
topic: 'LLM translation cost and quality (zh→vi, en→vi)'
decision: 'Which changes to AuraTranslate''s AI translation pipeline lower cost per chapter while keeping or raising accuracy and context fidelity'
source: 'native run (web), mapped onto research-ai-translation-pipeline (codebase @ 7bd1f90)'
status: complete
preset: 'standard'
validation: 'normal'
claims: {disputed: 1, unverified: 9, verified: 6}
created: '2026-10-27'
updated: '2026-10-27'
---

# technical research: LLM translation cost and quality (zh→vi, en→vi)

**Decision this research serves:** Which changes to AuraTranslate's AI translation pipeline lower cost per chapter while keeping or raising accuracy and context fidelity.

Câu chữ: tiếng Việt, thuật ngữ kỹ thuật giữ tiếng Anh. Mọi khẳng định về mã của app trỏ sang tài liệu mã nguồn đi kèm, `../research-ai-translation-pipeline/research-ai-translation-pipeline.md` (gọi tắt **[A]**, mốc `7bd1f90`; 17 khẳng định đã kiểm đúng, 1 bị điều chỉnh câu chữ). `[A §n]` trỏ mục n của [A]; chữ `ref n` đi sau là số khẳng định trong sổ kiểm của [A], không phải nguồn `[n]` của báo cáo này. Số `[n]` trỏ bảng nguồn ở cuối.

## Executive summary

**Nên làm gì:** đường dịch AI của app có hai chỗ hở chi phí và một chỗ hở chất lượng. Cả ba sửa được mà không cần đổi mô hình; riêng phương án gom câu (P) cần thêm quyết định kiến trúc. Lượt soát thứ hai thì nên giữ nguyên cách làm hiện tại. Việc chọn mô hình thì phải dựa vào phép đo của chính app, vì chưa có benchmark zh→vi công khai nào cho đời mô hình hiện tại.

1. **Thinking chạy mặc định và bị tính theo giá token ra.**
   - App không gửi tham số reasoning nào [A §1].
   - Gemini 3 không tắt được thinking qua endpoint tương thích, chỉ hạ được; bỏ trống thì chạy ở mức mặc định của mô hình [17]. Thinking tính theo giá token ra [15].
   - Reasoning không cải thiện bản dịch một cách đáng tin, thường làm giảm [12][13]; có một phản chứng [14].
   - Giá Gemini 3.8 Flash, preset hiện tại, tăng gấp đôi từ 2027-01-01 [15]. App hiện không hiện tiền cho mô hình này vì bảng giá chỉ có `claude-sonnet-5` [A §4].
   - ⇒ R1.
2. **Mỗi câu một request, mà prompt không có câu kề.**
   - Phần cố định khoảng 250 token bị trả đủ giá ở từng câu, vì nó ngắn hơn ngưỡng cache 1.024–4.096 token [16][21] [A §2].
   - Bằng chứng mạnh nhất về chất lượng là ngữ cảnh câu kề (3 cặp trước đã lấy phần lớn lợi ích) và bảng tên tường minh [1]. App có bảng tên (Glossary chỉ chèn mục có trong câu) nhưng không có câu kề [A §5].
   - ⇒ R2, với hai phương án trình kèm số mô hình tính: phương án C (context, thêm ngữ cảnh) tăng 64% token vào; phương án P (pack, gom 20 câu) giảm 73% token vào nhưng có rủi ro rơi câu [1].
3. **Giữ lượt soát thứ hai ở dạng phát hiện để người quyết**, như Proofreader đang làm [A §3]. Mô hình tự chẩn đoán lỗi kém và hay sửa thừa [7]. ⇒ R5.
4. **Hai việc chưa nên làm ngay:** chọn mô hình theo benchmark công khai (R3: dùng bộ đo trong app), và caching hay Batch API cho đường dịch chương (R4).

**Lưu ý lớn nhất:** mọi con số tiết kiệm ở đây là mô hình tính, rút từ số đo token của mã và từ nghiên cứu trên cặp ngôn ngữ khác. Chưa có lần gọi thật nào trong lượt này. Bước tiếp theo rẻ nhất là chạy một chương mẫu và đọc `usage` cùng hoá đơn của nhà cung cấp.

## D1. Đóng gói ngữ cảnh: dịch theo câu, theo đoạn hay theo tài liệu

- Ngữ cảnh làm bản dịch tốt hơn, nhưng phần lớn lợi ích đến từ vài câu ngay trước đó. Trong DelTA (ICLR 2025; GPT-3.5, GPT-4o-mini, Qwen2), chỉ đưa thêm 3 cặp nguồn–đích phía trước đã nâng sCOMET trung bình En→Xx từ 81,22 lên 82,95. Toàn bộ hệ bộ nhớ của DelTA nâng tiếp lên 84,36 [1]. Dịch nhiều lượt có "mồi nguồn" trên tài liệu WMT24 hơn dịch từng đoạn rời +1,34 COMET với GPT-4o-mini. Mô hình nhỏ được lợi nhiều hơn [2]. Độ tin: cao [1] (số đã kiểm lại trong bài), trung bình-cao [2].
- Độ nhất quán tên riêng đến từ một bảng tên/thuật ngữ tường minh, không đến từ ngữ cảnh. Trên Guofeng (truyện mạng Zh→En), DelTA đầy đủ nâng độ nhất quán LTCR-1 từ 37–62 lên 85–89. Trong ablation, riêng thành phần "Proper Noun Records" góp +3,95 LTCR-1, còn chỉ có ngữ cảnh gần thì LTCR-1 giảm (80,27 → 77,89) [1]. Tiếng nói người dùng Việt cũng chỉ ra đúng vấn đề này: tên nhân vật trôi, và cách chữa được nhắc tới là bảng tên dựng tay hoặc nhờ mô hình dựng. Nguồn này mỏng: một luồng voz.vn, chỉ đọc qua bản tóm tắt [36], và một thảo luận GitHub [37]. Độ tin: cao cho số [1], thấp cho tiếng nói người dùng.
- ⚠️ Gom nhiều câu vào một lượt gọi làm rơi câu. DelTA đo được: cửa sổ 30 câu rơi 8 câu, cửa sổ 50 câu rơi 10 câu. Với Doc2Doc (dịch 10 câu mỗi lượt trong cùng một cuộc hội thoại), kết quả không còn khớp từng câu [1]. Dịch nguyên đoạn văn học được người đọc chuộng hơn, nhưng bỏ sót nhiều hơn dịch theo câu (WMT23, chỉ còn giá trị lịch sử) [3]. Phần lớn mô hình mở xuống cấp mạnh ở khối 4k–8k token [4]. Chưa nghiên cứu nào đo tỉ lệ rơi câu theo số câu gom vào một request. Độ tin: cao cho số của DelTA; mối quan hệ theo kích thước lô chưa đo.
- Thuật ngữ: WMT25 nghiêng về để glossary trong ngữ cảnh một cách linh hoạt hơn là ép cứng [5]. Không tìm thấy thí nghiệm so "chỉ đưa mục có mặt trong câu" với "đưa cả glossary". Độ tin: thấp-trung bình.
- Few-shot lấy từ TM theo khớp mờ có giúp, nhưng lợi giảm dần sau vài ví dụ. Các bài còn mâu thuẫn nhau về truy hồi bằng token hay bằng embedding, và đều ở miền kỹ thuật, không phải văn chương [6]. Độ tin: trung bình-thấp.
- Không có đánh giá nào riêng cho Zh→Vi hoặc En→Vi về đại từ, xưng hô hay Hán Việt [1][36][37].

## D2. Đòn bẩy chi phí

- **Giá (trang chính thức, truy cập 2026-10-27, $/1M token, vào / vào-cache / ra):**

  | Mô hình | Vào | Vào (cache) | Ra | Nguồn |
  |---|---|---|---|---|
  | Gemini 3.8 Flash (đến hết 2026) | 0,75 | 0,075 | 3,75 | [15] |
  | Gemini 3.8 Flash (từ 2027-01-01) | 1,50 | 0,15 | 7,50 | [15][20] |
  | Gemini 3.1 Flash-Lite | 0,25 | 0,025 | 1,50 | [15] |
  | gpt-5.4-nano | 0,20 | 0,02 | 1,25 | [22] |
  | Claude Haiku 5.5 (prompt ≤100k) | 0,10 | 0,01 | 0,50 | [23] |
  | deepseek-flash (ngoài giờ cao điểm / giờ cao điểm) | 0,15 / 0,30 | 0,003 / 0,006 | 0,60 / 1,20 | [26] |
  | qwen3.8-flash (Singapore) | 0,15 | — | 0,47 | [29] |

  Giá Gemini 3.8 Flash đã kiểm với một nguồn độc lập. Việc tăng gấp đôi có blog xác nhận, nhưng các blog không thống nhất về giá vào-cache; trang của Google ghi 0,15 [15][20]. Các giá khác mới có một nguồn là trang của chính hãng. Độ tin: cao cho Gemini; trung bình-cao cho phần còn lại (mỗi giá một nguồn là trang của hãng).
- **Prompt caching** giảm giá phần vào đã cache còn khoảng 10% ở các hãng có giá cache trong bảng, DeepSeek còn khoảng 1–2%. Nhưng mỗi hãng có ngưỡng tối thiểu: Gemini 3.x Flash cần tiền tố ổn định từ 4.096 token [16], OpenAI cần 1.024 với GPT-5.6 trở lên (đời cũ hơn tuỳ thiết lập request; GPT-5.6+ còn có chế độ chỉ cache khi đặt breakpoint) [21], Haiku 5.5 cần 512 và phải đặt `cache_control` qua API gốc [24]. Lớp tương thích OpenAI của Anthropic không hỗ trợ caching [25]. DeepSeek cache tự động và báo `prompt_cache_hit_tokens` [27]. Chưa xác nhận được implicit caching của Gemini có chạy qua endpoint tương thích OpenAI hay không. Bằng chứng duy nhất là một luồng diễn đàn năm 2025 thấy không có `prompt_tokens_details` [19]. Độ tin: cao cho ngưỡng, chưa kiểm cho đường tương thích.
- **Thinking/reasoning** được tính theo giá token ra [15]. Trên Gemini 2.5, `reasoning_effort: "none"` tắt được thinking; trên Gemini 3, qua endpoint tương thích, thinking không tắt được mà chỉ hạ được bằng `reasoning_effort` (minimal/low…). Bỏ trống thì mô hình dùng mức mặc định của nó [17]; theo hướng dẫn Gemini 3 thì mức đó là high, nhưng mới thấy qua snippet, chưa đối chiếu. DeepSeek bật thinking mặc định; tắt bằng `thinking: {type: disabled}` [28]. Bật reasoning không cải thiện bản dịch một cách đáng tin, và thường làm giảm. Một bài đo XCOMET-XL trung bình theo mô hình giảm 0,3–3,8 điểm (từng cặp giảm tới khoảng 8,9) trên 4 mô hình × 9 cặp en→X, không có tiếng Việt [12]. Một nghiên cứu khác trên Qwen3 cho kết quả ngang hoặc kém hơn một chút [13]. Ngược lại, o3-mini lại tốt lên khi có reasoning [14]. Độ tin: trung bình (đã kiểm, kết quả lẫn lộn).
- 🔵 2026-10-27: lượt đo M2 với `gemini-3.5-flash-lite` cho thấy endpoint tương thích *có* báo `prompt_tokens_details.cached_tokens` (8.165/11.168 ở lượt gọi thứ hai trên cùng tiền tố), và `total_tokens` bằng đúng token vào cộng token ra (`../ad-brief-dich-ca-chuong-mot-luot/do-m2.md`).
- ⚠️ Trên endpoint tương thích, `completion_tokens` có thể không gồm token thinking. Suy luận này rút từ phép cộng trong một luồng diễn đàn, chưa kiểm [19].
- **Batch API** của Gemini giảm 50%, thời hạn mục tiêu 24 giờ, và lớp tương thích không tải tệp lên hay xuống được [18]. Trang giá của OpenAI [22], Anthropic [23] và Alibaba [29] cũng ghi batch giảm 50% (Qwen chỉ ở vùng Trung Quốc). Cách này chỉ hợp với chế độ "dịch cả cuốn qua đêm", không hợp khi người dùng đang chờ một chương. Độ tin: cao.
- **Định dạng đầu ra khi gom nhiều câu:** không có số đo nào riêng cho dịch thuật. Chỉ có bằng chứng gián tiếp: ép định dạng chặt (JSON-mode) làm giảm khả năng suy luận [31]. Độ tin: thấp.

## D3. Kiểm chất lượng và lượt soát thứ hai

- Một lượt "refine" đơn giản với mô hình đời hiện tại (GPT-5.2, GPT-4o, DeepSeek-V3, Qwen3-235B; WMT24-Literary) nâng MQM-FSP (điểm MQM theo thang FSP do giám khảo LLM chấm) khi refine cấp câu trên bản dịch tài liệu: +1,9 đến +3,6 tuỳ mô hình; toàn bài dao động khoảng +0,3 đến +7,6 tuỳ lát cắt. Lợi ích nằm ở độ trôi chảy và văn phong, không ở độ chính xác. Hai con số chấm bằng người (+5,6 với GPT-5.2; độ chính xác −2,3 đến +3,0) chưa đối chiếu được trong phần bài đã mở. Mặt trái là sửa thừa (25–40% số token bị đổi), và văn bản trôi khỏi nguồn. Prompt kiểu "tìm lỗi rồi sửa" hoặc "từng bước" cho kết quả kém hơn hoặc gây hại. Mô hình tự chẩn đoán lỗi rất kém: span F1 chỉ 0,06–0,11 [7]. Bài cũ hơn (TEaR) cũng cho lợi nhỏ, tốt nhất ở một vòng, nhiều vòng thì trôi xuống [8]. Độ tin: trung bình [7] (một bài, kiểm trích dẫn: một phần), trung bình [8].
- Mô hình bậc thang với QE: mô hình nhỏ dịch, QE xếp hạng, chuyển 30–50% câu tệ nhất (theo người chấm) hoặc 50–60% (theo chấm tự động) cho mô hình lớn thì bằng chất lượng mô hình lớn. Điểm hoà vốn là khi chuyển 75–89% số câu [9]. Thí nghiệm chỉ dùng Tower-v2 mở, không có tiếng Việt, không có mô hình API. Độ tin: trung bình-cao, nhưng khó đem sang trường hợp này.
- Thước đo không cần bản tham chiếu: XCOMET/MetricX và giám khảo LLM có tương quan với người gần ngang nhau, khoảng 0,71–0,72 ở WMT24 [11]. GEMBA-MQM V2 đứng đầu nhưng phải lấy mẫu 10 lần mỗi câu [10]. Không có kiểm định nào cho tiếng Việt, và giám khảo LLM kém ổn định hơn ở ngôn ngữ ít tài nguyên [40]. Độ tin: trung bình cho tương quan; thấp cho tiếng Việt.
- Phát hiện lỗi cấu trúc bằng tỉ lệ độ dài rất kém chính xác: một blog đếm 32 cờ đúng trên 1.258 cờ [39]. Không tìm thấy tỉ lệ rơi hay gộp câu nào được đo cho cách gom có đánh số. Độ tin: thấp.
- Hậu biên tập trên bản LLM rút ngắn thời gian so với dịch từ đầu. Đây là nghiên cứu nhỏ, 4 dịch giả, cặp tiếng giàu tài nguyên [38]. Độ tin: trung bình.

## D4. Chất lượng mô hình trên giá cho zh→vi và en→vi

- **en→vi:** dữ liệu mới nhất có số là WMT25, chỉ chấm tự động, nay đã 14 tháng tuổi. Thứ hạng AutoRank (1 là tốt nhất): Hunyuan-MT-7B 1,0 · Gemini-2.5-Pro 2,7 · CommandA-WMT 2,7 · GPT-4.1 2,8 · DeepSeek-V3 3,2 · Qwen3-235B 3,3 · Claude-4 5,0 · Gemma-3-27B 9,9 · ONLINE-G (hệ dịch trực tuyến ẩn danh) 17,4 [32]. Đây là bài "Preliminary Ranking". Nhóm đầu sát nhau. Mô hình dịch chuyên dụng 7B đứng đầu, nhưng bài báo tự cảnh báo có mô hình được tinh chỉnh theo đúng các thước đo này. Độ tin: trung bình-cao cho thứ hạng, thấp cho khả năng áp vào đời mô hình hiện nay.
- **zh→vi:** WMT25 không có cặp này [32]. WMT24++ có vi_VN nhưng số chỉ nằm trong biểu đồ [35]. Hunyuan-MT chỉ báo con số gộp [33]. DITING (benchmark truyện mạng) chỉ có zh→en; ở đó mô hình huấn luyện bằng tiếng Trung thắng mô hình ngoại lớn hơn (DeepSeek-V3 5,16 nhỉnh hơn GPT-4o 5,09 trên thang 6; Qwen3-8B hơn LLaMA3-70B) [34]. ⇒ Không có bằng chứng công khai nào xếp hạng mô hình đời hiện tại cho zh→vi. Độ tin: cao cho chuyện *thiếu* bằng chứng.
- Mô hình dịch chuyên dụng truy cập được qua API tương thích OpenAI: Qwen-MT có hỗ trợ tiếng Việt [30], nhưng chưa có điểm chất lượng cho tiếng Việt. Hunyuan-MT và Seed-X chỉ có trọng số mở; không tìm thấy endpoint hosted nào [33].
- Không tìm thấy nguồn nào so chất lượng trên chi phí cho tác vụ dịch.

## Cross-dimension insights

1. **Hai đòn bẩy chi phí lớn nhất của app chưa được dùng, và không cái nào cần đổi mô hình.**
   - Thinking: mỗi request không gửi tham số reasoning nào [A §1, ref 12]. Gemini 3 không tắt được thinking, chỉ hạ được, và bỏ trống thì chạy ở mức mặc định của mô hình [17]; thinking tính theo giá token ra [15]. Reasoning không giúp bản dịch một cách đáng tin [12][13]. Nếu `completion_tokens` qua endpoint tương thích quả thật không gồm thinking [19], app đang ghi thấp số token mà vẫn bị tính tiền. Hiện app cũng không hiện tiền cho `gemini-3.8-flash`, vì bảng giá chỉ có một hàng `claude-sonnet-5` [A §4, ref 7].
   - Phí cố định mỗi request: dịch lô gửi một request cho mỗi câu [A §2, ref 1–2]. Phần cố định khoảng 250 token mỗi request (đo bằng tiktoken `o200k_base` trên system prompt mặc định và mẫu tiên hiệp, xấp xỉ cho tokenizer của Gemini: system 85 + khung mẫu 165), thấp hơn hẳn ngưỡng cache của Gemini (4.096) và OpenAI (1.024) [16][21]. Vì vậy phần lặp lại bị tính đủ giá ở mỗi câu.
2. **Thứ app thiếu về chất lượng chính là thứ bằng chứng nói có lợi nhất: ngữ cảnh câu kề.** Prompt hiện không mang câu trước, câu sau, bản dịch trước hay tóm tắt nào [A §5, ref 16]. Theo DelTA, chỉ cần 3 cặp nguồn–đích phía trước là đã lấy được phần lớn lợi ích sCOMET [1]. Ngược lại, bảng tên tường minh (Glossary chỉ chèn mục đã xác nhận *có mặt trong câu*; chưa thấy giới hạn số mục [A §5, ref 14]) đúng là cơ chế bằng chứng chỉ ra cho nhất quán tên riêng [1].
3. **Gom câu là con dao hai lưỡi, và chính app đã có cả hai cách.** Proofreader gom 4.000 ký tự hoặc 20 câu, đánh dấu `<<<SEGMENT n`, đọc lại theo chỉ số. Cách đó đã được đo: 153 token cố định mỗi lời gọi, chương 184 câu tốn khoảng 14k token vào khi gom so với 40k khi gọi từng câu (prompt Proofreader ngắn hơn prompt dịch) [A §3, §7, ref 5, 18]. Dịch thì không gom, và không commit nào ghi lý do [A §7]. Gom giảm phí cố định và mang luôn ngữ cảnh kề, nhưng có rủi ro rơi câu [1][3]. Ngoài ra nó phá luồng `Token{segment_id}` chảy từng câu cùng quy tắc "lỗi đầu tiên dừng lô và nêu tên câu" [A §2, ref 2–3].
4. **Lượt soát thứ hai nên giữ ở dạng *phát hiện để người quyết*, đừng chuyển sang *tự sửa*.** Mô hình tự chẩn đoán kém (span F1 0,06–0,11) và hay sửa thừa [7]. Proofreader của app chỉ báo phát hiện để người duyệt, không tự sửa [A §3], đúng hướng bằng chứng. Số báo động giả nên đo bằng script có sẵn [A §6].

## Contrary evidence

Lượt red-team không chạy (tắt theo cấu hình). Những phản chứng gặp trong lúc kiểm:

- o3-mini tốt lên khi bật reasoning [14]. Vì vậy câu "tắt thinking" phải đi kèm phép đo trên chính mô hình đang dùng.
- Dịch nguyên đoạn được người đọc chuộng hơn dịch từng câu (WMT23, kết quả lịch sử) [3]. Điều này đi ngược lập luận "giữ một câu một request vì an toàn".

## Recommendations

Đây là đầu vào cho John/Sofia (vé story) và Winston (nếu phải đổi AD). Gom câu khi dịch đụng quy tắc Channel và huỷ (AD-22) cùng quy tắc dừng lô, nên cần Winston xem.

**R1. Gửi điều khiển thinking theo từng nhà cung cấp, và đo trước.** Độ tin: trung bình.
- Gemini 3: `reasoning_effort: "minimal"`. Gemini 2.5: `"none"`. DeepSeek: `thinking: {type: disabled}`. Bỏ trống với nhà cung cấp không hỗ trợ; lớp tương thích của Anthropic bỏ qua tham số này [17][28][25].
- Vì có phản chứng [14], quyết định dựa trên phép đo cùng một chương: (a) số token thinking và tiền thật ở bảng điều khiển của hãng, với mức mặc định so với minimal; (b) độ chấp nhận bản dịch của Ice.
- Đi kèm: thêm hàng giá cho preset `gemini-3.8-flash`, có ngày hiệu lực, vì giá đổi vào 2027-01-01 [15]. Kiểm xem `completion_tokens` có gồm thinking không [19], để số tiền app hiện không thấp hơn hoá đơn.

**R2. Đưa ngữ cảnh câu kề vào prompt.** Độ tin: cao cho chiều hướng [1][2], chưa đo trên zh/en→vi. Có hai phương án hợp lệ, cần Ice chọn sau khi đo (theo AGENTS.md):
- **Phương án C, giữ một câu một request và thêm ngữ cảnh.** Thêm 3 cặp nguồn–đích ngay trước câu, lấy bản đã dịch nếu có. Mô hình tính: 184 câu, phần cố định 250 token (đo), còn 35 token nguồn, 35 token đích và 40 token Glossary/TM mỗi câu là giả định. Mốc hiện nay là 184 × (250 + 35 + 40) ≈ 60k token vào mỗi chương.
  - Token vào khoảng 98k/chương (+64%).
  - Giữ nguyên luồng chảy từng câu, cách huỷ và quy tắc dừng lô.
  - Phải đổi biến prompt. Hiện chỉ có đúng ba biến [A §5]; một biến mới như `{{chapter_context}}` hiện bị giữ nguyên chữ [A §1].
- **Phương án P, gom N câu liền nhau mỗi request** theo khuôn Proofreader (đánh số, khớp theo chỉ số, đếm câu thiếu thành lỗi).
  - Với N=20, token vào khoảng 16k/chương (−73% so với hiện nay), và ngữ cảnh kề có sẵn.
  - Đổi lại: rủi ro rơi câu (8 trên 30 câu [1]); mất luồng chảy từng câu; phải đổi quy tắc dừng lô; và cần kiểm "số câu trả về = số câu gửi".
- ⚠️ Bằng chứng chính cho chiều hướng này, [1], đã quá hạn kiểm lại theo bản đồ độ cũ; cần làm mới số liệu.
- Cả hai phương án chỉ là mô hình tính. Số thật lấy bằng cách chạy một chương mẫu, đọc `usage` từ nhà cung cấp, và đếm câu bị rơi hoặc lệch.

**R3. Chọn mô hình bằng một bộ đo nhỏ trong app, không dựa vào benchmark công khai.** Độ tin: cao cho chuyện thiếu bằng chứng [32][34][35].
- Chưa có bảng xếp hạng zh→vi nào cho đời mô hình hiện tại.
- Đề xuất: lấy vài chục câu khó (tên riêng, xưng hô, thuật ngữ tu tiên) cho từng cặp zh→vi và en→vi, so các mô hình cùng một prompt.
- Ứng viên rẻ nên có: Gemini 3.1 Flash-Lite, deepseek-flash (giá ngoài giờ cao điểm), Haiku 5.5, Qwen-MT hoặc qwen3.8-flash. Gemini 3.8 Flash là mốc so sánh [15][23][26][29][30].

**R4. Chưa nên làm caching hay Batch API cho đường dịch chương.**
- Tiền tố ổn định quá ngắn (khoảng 250 token so với ngưỡng 1.024–4.096) [16][21], và Anthropic không cache qua lớp tương thích [25].
- Batch hợp với chế độ "dịch cả cuốn qua đêm" nếu sau này có [18]. Chế độ đó cần client gốc của Gemini, tức thêm một điểm mạng, đụng AD-15 (chỉ cho phép đúng ba điểm mạng) [A §7].
- Độ tin: cao.

**R5. Không thêm lượt tự sửa tự động.** Nếu muốn có bước tinh chỉnh, chỉ dùng một lượt prompt refine đơn giản, và đưa kết quả lên làm gợi ý cho người duyệt [7][8]. Độ tin: trung bình-cao.

## Open questions

- 🔵 2026-10-27: phần cache đã có câu trả lời là có, với `gemini-3.5-flash-lite` (`../ad-brief-dich-ca-chuong-mot-luot/do-m2.md`); phần thinking vẫn còn mở với `gemini-3.8-flash`.
- **Gemini qua endpoint tương thích:** implicit caching có chạy không, và `completion_tokens` có gồm thinking không [19]. Để trả lời: một lần gọi thật có prefix từ 4.096 token, đọc khung `usage` cuối.
- **Chất lượng zh→vi và en→vi của đời mô hình hiện tại:** chưa có nguồn công khai. Để trả lời: bộ đo R3.
- **Tỉ lệ rơi câu theo N** khi gom có đánh số với các mô hình API hiện tại: chưa ai đo. Để trả lời: chạy phương án P với N = 5, 10, 20 trên cùng chương.
- **Thinking mức minimal của Gemini 3.x Flash** thực tế tốn bao nhiêu token: chưa có số độc lập [17].
- **Vì sao dịch lô là một câu một request và chạy tuần tự:** lý do nằm trong spec 4.9, chưa đọc [A §Open questions].

## Source appendix

| # | Hỗ trợ khẳng định | Nhà xuất bản | Ngày | Truy cập | Độ tin |
|---|---|---|---|---|---|
| [1] | DelTA: ngữ cảnh 3 cặp, bảng tên riêng, rơi câu theo cửa sổ | [arXiv / ICLR 2025](https://arxiv.org/html/2410.08143v2) | 2024-10 | 2026-10-27 | cao |
| [2] | Dịch nhiều lượt có mồi nguồn so với từng đoạn | [arXiv](https://arxiv.org/html/2503.10494) | 2025-03 | 2026-10-27 | trung bình-cao |
| [3] | Dịch đoạn văn học, bỏ sót nhiều hơn (lịch sử) | [arXiv / WMT23](https://arxiv.org/pdf/2304.03245) | 2023-04 | 2026-10-27 | trung bình |
| [4] | SEGALE: mô hình mở xuống cấp ở khối 4k–8k | [arXiv / EMNLP 2025](https://arxiv.org/abs/2509.17249) | 2025-09 | 2026-10-27 | trung bình |
| [5] | WMT25 terminology: glossary linh hoạt hơn ép cứng | [arXiv (DuTerm)](https://arxiv.org/abs/2511.07461) | 2025-11 | 2026-10-27 | thấp-trung bình |
| [6] | Few-shot từ TM khớp mờ, lợi giảm dần | [EAMT 2026](https://aclanthology.org/2026.eamt-1.34.pdf) | 2026 | 2026-10-27 | trung bình-thấp |
| [7] | Refine với mô hình hiện tại: trôi chảy lên, sửa thừa | [arXiv](https://arxiv.org/html/2605.13368) | 2026-05 | 2026-10-27 | trung bình-cao |
| [8] | TEaR: lợi nhỏ, một vòng là tốt nhất | [arXiv / NAACL 2025 Findings](https://arxiv.org/html/2402.16379v3) | 2024-02 | 2026-10-27 | trung bình |
| [9] | Bậc thang QE: chuyển 30–50%, hoà vốn 75–89% | [arXiv / EMNLP 2025](https://arxiv.org/html/2502.12701) | 2025-02 | 2026-10-27 | trung bình-cao |
| [10] | GEMBA-MQM V2 lấy mẫu 10 lần/câu | [ACL Anthology / WMT25](https://aclanthology.org/2025.wmt-1.67/) | 2025 | 2026-10-27 | trung bình |
| [11] | WMT24 metrics: XCOMET/MetricX ≈ giám khảo LLM | [ACL Anthology / WMT24](https://aclanthology.org/2024.wmt-1.2) | 2024 | 2026-10-27 | trung bình |
| [12] | Reasoning làm XCOMET giảm 0,3–3,8 | [arXiv](https://arxiv.org/html/2602.14763) | 2026-02 | 2026-10-27 | trung bình |
| [13] | Qwen3: thinking ngang hoặc kém hơn | [arXiv](https://arxiv.org/abs/2510.11919) | 2025-10 | 2026-10-27 | thấp-trung bình (snippet) |
| [14] | o3-mini tốt lên khi reasoning (phản chứng) | [arXiv](https://arxiv.org/abs/2504.08120) | 2025-04 | 2026-10-27 | thấp-trung bình (snippet) |
| [15] | Giá Gemini, thinking tính giá ra, tăng giá 2027 | [Google](https://ai.google.dev/gemini-api/docs/pricing) | 2026-10-26 | 2026-10-27 | cao |
| [16] | Ngưỡng implicit cache 4.096 token | [Google](https://ai.google.dev/gemini-api/docs/caching) | 2026-10-23 | 2026-10-27 | cao |
| [17] | Gemini 3 không tắt được reasoning qua endpoint tương thích; Gemini 2.5 tắt bằng `none` | [Google](https://ai.google.dev/gemini-api/docs/openai) | 2026-10-26 | 2026-10-27 | cao |
| [18] | Batch 50%, 24 giờ, lớp tương thích không tải tệp | [Google](https://ai.google.dev/gemini-api/docs/batch-api) | 2026-09-17 | 2026-10-27 | cao |
| [19] | Endpoint tương thích không có `prompt_tokens_details`; người hỏi suy ra `completion_tokens` không gồm thinking (chưa ai xác nhận) | [Google AI developer forum](https://discuss.ai.google.dev/t/question-with-usage-in-openai-compatibility/98760) | 2025-08-19 | 2026-10-27 | trung bình (cũ) |
| [20] | Gemini 3.8 Flash tăng giá 2027 (nguồn thứ hai; URL bài cụ thể chưa ghi lại, chỉ có trang chủ) | [Digital Applied](https://www.digitalapplied.com/) | 2026-09-02 | 2026-10-27 | trung bình |
| [21] | OpenAI cache tự động, tiền tố 1.024 token | [OpenAI](https://developers.openai.com/api/docs/guides/prompt-caching) | trang sống | 2026-10-27 | cao |
| [22] | Giá gpt-5.4-nano | [OpenAI](https://developers.openai.com/api/docs/pricing) | trang sống | 2026-10-27 | trung bình |
| [23] | Giá Haiku 5.5 | [Anthropic](https://platform.claude.com/docs/en/about-claude/pricing) | trang sống | 2026-10-27 | cao |
| [24] | Haiku 5.5 cache tối thiểu 512 token | [Anthropic](https://platform.claude.com/docs/en/build-with-claude/prompt-caching) | trang sống | 2026-10-27 | cao |
| [25] | Lớp tương thích OpenAI của Anthropic: không caching, bỏ qua `reasoning_effort` và `response_format` | [Anthropic](https://platform.claude.com/docs/en/api/openai-sdk) | trang sống | 2026-10-27 | cao |
| [26] | Giá deepseek-flash, thinking mặc định bật | [DeepSeek](https://api-docs.deepseek.com/quick_start/pricing/) | trang sống | 2026-10-27 | cao |
| [27] | DeepSeek cache tự động | [DeepSeek](https://api-docs.deepseek.com/guides/kv_cache) | trang sống | 2026-10-27 | cao |
| [28] | Tắt thinking DeepSeek | [DeepSeek](https://api-docs.deepseek.com/guides/thinking_mode) | trang sống | 2026-10-27 | cao |
| [29] | Giá Qwen | [Alibaba Cloud](https://www.alibabacloud.com/help/en/model-studio/model-pricing) | 2026-10-27 | 2026-10-27 | trung bình |
| [30] | Qwen-MT hỗ trợ tiếng Việt, API tương thích OpenAI | [Alibaba Cloud](https://www.alibabacloud.com/help/en/model-studio/machine-translation) | 2026-10-11 | 2026-10-27 | trung bình-cao |
| [31] | Ép định dạng chặt làm giảm suy luận | [arXiv / EMNLP 2024](https://arxiv.org/pdf/2408.02442) | 2024-08 | 2026-10-27 | trung bình (snippet) |
| [32] | WMT25 en→vi AutoRank; không có zh→vi | [arXiv (WMT25 organisers)](https://arxiv.org/pdf/2508.14909) | 2025-08 | 2026-10-27 | trung bình-cao |
| [33] | Hunyuan-MT: tiếng Việt có hỗ trợ, chỉ số gộp | [arXiv (Tencent)](https://arxiv.org/pdf/2509.05209) | 2025-09 | 2026-10-27 | trung bình |
| [34] | DITING truyện mạng zh→en | [arXiv](https://arxiv.org/html/2510.09116v2) | 2025-10 | 2026-10-27 | trung bình |
| [35] | WMT24++ có vi_VN | [arXiv (Google)](https://arxiv.org/html/2502.12404) | 2025-02 | 2026-10-27 | cao (phạm vi) |
| [36] | Người dùng Việt: tên nhân vật trôi | [voz.vn](https://voz.vn/t/tool-dich-truyen-bang-gemini.1076131/) | không rõ | 2026-10-27 | thấp |
| [37] | Bảng xưng hô do Gemini dựng | [GitHub](https://github.com/hydropix/TranslateBooksWithLLMs/discussions/238) | không rõ | 2026-10-27 | thấp |
| [38] | Hậu biên tập văn chương rút ngắn thời gian | [ACL Anthology / MT Summit 2025](https://aclanthology.org/2025.mtsummit-1.40) | 2025 | 2026-10-27 | trung bình |
| [39] | Cờ theo tỉ lệ độ dài: 32 trên 1.258 cờ đúng | [Source Library blog](https://sourcelibrary.org/blog/translation-collapse) | không rõ | 2026-10-27 | thấp |
| [40] | Giám khảo LLM kém ổn định ở ngôn ngữ ít tài nguyên | [alphaXiv](https://alphaxiv.org/abs/2505.12201) | 2025 | 2026-10-27 | thấp |

## Staleness map

Tính bằng `recon_kit.py staleness`, ngày gốc 2026-10-27. Cửa sổ: giá 1 tháng · phiên bản/tương thích 1 tháng · số chất lượng và phương pháp 24 tháng · kiểu thất bại 24 tháng.

| Khẳng định | Lớp | Ngày nguồn | Kiểm lại |
|---|---|---|---|
| [15] Giá Gemini 3.8 Flash, tăng giá 2027 | pricing | 2026-10 | 2026-11-01 |
| [23] Giá Haiku 5.5 · [26] giá deepseek-flash | pricing | 2026-10 | 2026-11-01 |
| [16] ngưỡng cache Gemini · [17] reasoning Gemini 3 · [21] cache OpenAI · [25] lớp tương thích Anthropic · [28] thinking DeepSeek · [30] Qwen-MT | version | 2026-10 | 2026-11-01 |
| [19] usage qua endpoint tương thích Gemini | version | 2025-08 | **đã cũ** (2025-09-01) |
| [1] DelTA: rơi câu theo cửa sổ | failure-mode | 2024-10 | **đã cũ** (2026-10-01) |
| [32] WMT25 en→vi | quality-number | 2025-08 | 2027-08-01; xếp hạng mô hình là cảnh quan AI (cửa sổ 3 tháng), nên coi như đã cũ |
| [9] bậc thang QE · [2] mồi nguồn | quality-number | 2025-02 / 2025-03 | 2027-02-01 / 2027-03-01 |
| [12] reasoning · [7] refine | quality-number | 2026-02 / 2026-05 | 2028-02-01 / 2028-05-01 |

Hạn kiểm lại sớm nhất: 2025-09-01, khẳng định [19], đã quá hạn. Nó cần một lần gọi thật (mục Open questions). Toàn bộ nhóm giá và phiên bản đến hạn vào 2026-11-01. Lệnh Refresh của Deep Recon trên thư mục này làm việc đó.
