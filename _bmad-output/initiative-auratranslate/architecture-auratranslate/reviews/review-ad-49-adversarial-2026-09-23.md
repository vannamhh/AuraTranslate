# Lens đối kháng — AD-49 (không ngăn xếp hoàn tác)

**Phương pháp:** dựng hai đơn vị ở tầng dưới (hai story/feature), mỗi đơn vị tuân AD-49 đúng từng chữ, rồi tìm chỗ chúng vẫn dựng ra thứ không khớp nhau — hai cách phân lớp cho cùng một lượt ghi, hai chủ cho cùng một thực thể, hai đường mutate xung khắc.

Không re-litigate: không hoàn tác cho gộp/tách; phạm vi toàn sản phẩm; siêu dữ liệu Chương bị loại khỏi "nội dung người dùng"; binding `Mod+Z` phải nhường vùng gõ. Bốn quyết định này được giữ nguyên; các lỗ dưới đây nằm ở chỗ AD-49 CHƯA nói, hoặc tự mâu thuẫn.

**Verdict: 4 lỗ, 3 nghiêm trọng.**

---

## 🔴 F1 — Ví dụ lớp (i) "gộp ↔ tách Chương" mượn bảo chứng của AD-32, nhưng AD-32 chỉ nói về segment

**Hai đơn vị:** Đơn vị A dựng chính thao tác gộp/tách Chương. Theo Rule 2(i), cặp này được liệt sẵn làm ví dụ lớp (i) — "đường lui là gọi lệnh kia", **không** hỏi xác nhận. Đơn vị B là bất kỳ story sau này gắn trạng thái riêng lên **hàng `CHAPTER`** — tiến độ đọc theo Chương, ghi chú ghim, asset cấp Chương (AD-42 đã mở tiền lệ media gắn ở tầng Chương) — mỗi thứ đó là cột trên `CHAPTER`, không phải trên `SEGMENT`.

**Va nhau ở đâu:** bảo chứng cấu trúc duy nhất cho "gộp/tách Chương an toàn" là AD-32, và AD-32 chỉ cam kết **"chỉ đổi `chapter_id` và `ord` của các segment liên quan"** — nó không nói một chữ về các cột riêng của chính hàng `CHAPTER` bị về hưu (gộp) hay sinh ra (tách). Đơn vị A đúng luật AD-49 (đã có ví dụ khai sẵn, không cần hỏi). Đơn vị B đúng luật của chính nó. Nhưng khi gộp hai Chương, cột trạng thái riêng của Chương bị bỏ sẽ **biến mất không hỏi**, và tách lại không phục hồi nó — đúng thứ Prevents(3) của AD-49 định chặn ("ghi đè hoặc xoá văn bản người dùng không còn bản sao, không hỏi"), xảy ra **qua chính miễn trừ lớp (i)** thay vì bất chấp nó.

**Hậu quả:** mất dữ liệu người dùng đặt vào Chương, im lặng, không cổng nào đỏ — vì lớp (i) đã tự loại bỏ nhu cầu xác nhận trước khi Đơn vị B kịp tồn tại.

**Rule line cho qua:** Rule 2, ví dụ đã khai sẵn "gộp ↔ tách Chương" ở lớp (i), phạm vi rộng hơn bảo chứng thật của AD-32.

**Vá (đề xuất):** *"Ví dụ 'gộp ↔ tách Chương' ở lớp (i) chỉ phủ đúng phạm vi AD-32 bảo chứng: `chapter_id`, `ord` của segment liên quan, và các cột `CHAPTER` đã có tại 2026-08-02. Một story sau này thêm cột mới trực tiếp trên `CHAPTER` phải tự khai lớp cho lượt gộp/tách xoá cột đó — không thừa hưởng miễn trừ (i) của thao tác gộp/tách Chương gốc."*

---

## 🔴 F2 — `prompt_set` vừa là nội dung người dùng lớp (iii), vừa là giá trị ghi đè cấu hình được loại trừ — cùng một câu

**Hai đơn vị:** Rule 3 liệt `prompt_set` vào danh sách "hàng người dùng soạn trọn một thực thể" → lớp (iii), yêu cầu xác nhận hai lượt kèm *"không hoàn tác được"*. Cùng câu đó loại trừ **"giá trị ghi đè cấu hình (xoá = trả về kế thừa)"**. Nhưng AD-18 đã xếp **Prompt = ghi đè** trong bảng ngữ nghĩa hai tầng (dòng "Prompt | ghi đè | FR69") — nghĩa là một `prompt_set` ở tầng Tác phẩm CHÍNH LÀ giá trị ghi đè, xoá nó = trả về `prompt_set` toàn cục kế thừa.

**Va nhau ở đâu:** Đơn vị A dựng "xoá bộ prompt tuỳ chỉnh của Tác phẩm, quay về mặc định toàn cục" — đọc theo AD-18, đây là xoá một ghi đè, thuộc nhóm loại trừ thứ hai của Rule 3 → không hỏi. Đơn vị B dựng một màn quản lý `prompt_set` nói chung — đọc đúng chữ đầu của Rule 3, `prompt_set` được **liệt tên tường minh** ở nhóm (iii) → bắt buộc xác nhận hai lượt. Cả hai đều trích đúng Rule 3, cho **cùng một bảng, có thể cùng một hàng**, hai hành vi UX và an toàn dữ liệu trái ngược nhau.

**Hậu quả:** một đường xoá dữ liệu thật (mất bộ prompt Tác phẩm đã tinh chỉnh, không có bản sao nào khác theo Rule 3) đi qua mà không hỏi gì, vì story đó tự đọc theo nhánh loại trừ.

**Rule line cho qua:** Rule 3, danh sách nội dung người dùng (`prompt_set`) và nhóm loại trừ "giá trị ghi đè cấu hình" trong cùng một câu, không phân biệt trường hợp một thực thể vừa khớp cả hai.

**Vá (đề xuất):** *"`prompt_set` là nội dung người dùng lớp (iii) chỉ khi nó không có tầng kế thừa để rơi về (vd. `prompt_set` toàn cục gốc). Một `prompt_set` mà AD-18 xếp là ghi đè ở tầng Tác phẩm thì xoá nó thuộc nhóm loại trừ thứ hai (trả về kế thừa), không thuộc lớp (iii), bất kể tên bảng."*

---

## 🔴 F3 — "Nhường vùng gõ" không có một phép kiểm dùng chung; mỗi binding tự định nghĩa "vùng gõ" theo cách của nó

**Hai đơn vị:** Đơn vị A (Editor chính) coi vùng gõ là "trong editing host contenteditable". Đơn vị B (một overlay nổi trong dockview — ô đổi tên, ô tìm trong panel Lookup, ô sửa nhanh một mục Glossary) coi vùng gõ là `tagName === 'INPUT' || 'TEXTAREA'`. Cả hai bind `Mod+Z`/`Mod+Shift+Z` cho lý do riêng (theo tinh thần AD-34, mọi thao tác qua `CommandRegistry`) và cả hai tự tin đã "nhường vùng gõ" theo đúng chữ Rule 1.

**Va nhau ở đâu:** Rule 1 chỉ nói **kết quả** phải đạt ("phải nhường vùng gõ — không `preventDefault` ở đó"), không đặt tên một component/predicate DÙNG CHUNG để trả lời "đây có phải vùng gõ không" — khác hẳn cách AD-17 ("một component Matcher dùng chung") và AD-18 ("một ScopeResolver") đã xử lý đúng dạng câu hỏi này ở nơi khác trong spine. Một `<input>` bên trong overlay nổi của dockview, đang giữa một chuỗi gõ IME (tiếng Trung/Việt có dấu), là vùng gõ thật mà phép kiểm `contentEditable`-only của Đơn vị A bỏ sót; ngược lại editing host contenteditable của Đơn vị A không phải `INPUT`/`TEXTAREA` nên phép kiểm tagName-only của Đơn vị B cũng bỏ sót nó. Bất kỳ ai ra sau sẽ gọi `preventDefault` đúng vào vùng gõ của người kia.

**Hậu quả:** đúng lỗ Prevents(2) đã nêu tên — command `Mod+Z` "bắn cả trong ô bản dịch […] mà vẫn biên dịch và vẫn xanh" — vì test của mỗi story chỉ phủ hình dạng input của chính nó.

**Rule line cho qua:** Rule 1, "phải nhường vùng gõ" không kèm tên một phép kiểm/công cụ dùng chung.

**Vá (đề xuất):** *"Một `isTypingZone()` dùng chung, sống cạnh `CommandRegistry` (AD-34), là điểm kiểm DUY NHẤT mọi binding `Mod+Z`/`Mod+Shift+Z` phải gọi trước khi cân nhắc `preventDefault`. Không component nào tự cài lại phép kiểm này. Định nghĩa của nó phải phủ: contentEditable, `input`/`textarea` kể cả bên trong overlay/panel nổi của dockview, và trạng thái đang gõ IME (`compositionstart`…`compositionend`)."*

---

## 🟡 F4 — Lớp (i) là tự khai, không có gì cưỡng chế "lệnh nghịch đảo" là nghịch đảo THẬT ngoài segment/Chương

**Hai đơn vị:** Đơn vị A dựng "gộp hai mục Glossary trùng lặp" (dedupe), tự khai lớp (i) theo tương tự "gộp ↔ tách Chương" — không hỏi xác nhận, vì spec của nó nói có lệnh "tách lại". Đơn vị B là chính con đường sửa/xoá trực tiếp một `glossary_entry` mà Rule 3 đã liệt tên ở lớp (iii) — bắt buộc xác nhận hai lượt vì "không còn bản sao nào".

**Va nhau ở đâu:** với segment và Chương, lớp (i)/(ii) có bảo chứng cấu trúc thật — AD-3 (id bền), AD-5 (về hưu, không xoá), AD-31 (SegmentVersion), AD-32 (segment giữ nguyên qua gộp/tách Chương). Với mọi thực thể khác, Rule 2 chỉ đòi "khai trong spec của story dựng nó" — không đòi chứng minh lệnh nghịch đảo thật sự phục hồi được byte đã mất, không đòi một cơ chế giữ bản cũ tương đương SegmentVersion. Đơn vị A có thể hợp lệ về mặt chữ nghĩa (đã khai lớp trong spec) mà không có gì canh cả — hàng thua trong lượt gộp bị xoá thật, "tách lại" chỉ tạo hàng mới trống, không phải khôi phục hàng cũ. Trong khi đó Đơn vị B, thao tác cùng bảng `glossary_entry`, đòi hỏi xác nhận nghiêm ngặt cho đúng loại mất mát đó.

**Hậu quả:** hai đường ghi vào cùng một bảng, một đường được canh chặt, một đường thoát canh vì tự nhận là lớp (i) — mất dữ liệu Glossary người dùng gõ tay, không hỏi, không có bản sao.

**Rule line cho qua:** Rule 2, mệnh đề mở đầu "khai trong spec của story dựng nó" — tự khai, không kèm yêu cầu bảo chứng cấu trúc hay test cho những thực thể ngoài segment/Chương.

**Vá (đề xuất):** *"Lớp (i) chỉ được khai khi lệnh nghịch đảo có bảo chứng cấu trúc từ một AD khác (như AD-3/AD-5/AD-31 cho segment, AD-32 cho Chương). Với thực thể chưa có AD nào bảo chứng như vậy, một thao tác 'gộp' phải khai lớp (iii) trừ khi story đó đồng thời dựng cơ chế giữ bản cũ (bảng lịch sử, hàng về hưu…) VÀ một test cưỡng chế nó."*

---

## Bằng chứng đã đọc

AD-49 (dòng 775–789), AD-1 (75–79), AD-3 (89–93), AD-5 (103–111), AD-18 (238–288, đặc biệt dòng 247 "Prompt | ghi đè"), AD-31 (368–392), AD-32 (394–398), AD-34 (406–417), AD-35 (419–425), AD-47 (675–742), Consistency Conventions liên quan (791–817). Không đọc `prd.md`/`epics.md`; các suy luận về `prompt_set`, tiến độ đọc theo Chương, và dedupe Glossary là **cấu trúc lỗ giả định hợp lý từ chữ đã ký trong spine**, không phải trích dẫn từ tài liệu khác — cần Winston xác nhận trước khi tính là đóng hay mở.
