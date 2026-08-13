# Sprint Change Proposal — Auto-Lookup thu về Panel Source, và ràng buộc QuickTranslator được phát biểu lại

**Ngày:** 2026-08-13 · **Chủ dự án:** Ice · **Chủ trì:** John (Product Manager)
**Chế độ:** Batch — cả gói trình một lần
**Baseline:** `3215e68` *(master)* · cây làm việc **sạch** lúc bắt đầu

> ✅ **ICE DUYỆT 2026-08-13** — trọn gói, **giữ §4.9** *(bổ sung `prd.md:56`)*. Bàn giao cho
> Amelia (Developer) theo thứ tự §5.3.
>
> **Hai thay đổi, và cái thứ hai lớn hơn cái thứ nhất.**
> **①** FR21 thu hẹp — Panel AI Translation và Panel Editor thôi là nguồn tra cứu *(§1, §4.1–4.7)*.
> **②** Ràng buộc QuickTranslator được phát biểu lại: *"tham khảo và nâng cấp, không bê nguyên
> si"* thay cho *"không được thiết kế lại cho khác đi"* *(§2.5, §4.8–4.9)* — Ice chỉ đạo
> 2026-08-13. Thay đổi ② là **nguyên tắc**, nên nó sống lâu hơn lượt sửa này; thay đổi ① trở
> thành **ca áp dụng đầu tiên** của nó.

---

## 1. Vấn đề

### 1.1 Trigger

Ice dùng thật và báo:

> *"hiện tại phần biên dịch cũng tra từ điển, như vậy là dư thừa, vì là tiếng việt đã dịch
> ra, khi chọn thì không tra cứu gì nữa, cần bỏ luồng chức năng này đi"*

**Loại vấn đề:** *Misunderstanding of original requirements*. FR21 được viết lúc Panel Editor
và Panel AI Translation **chưa có một chữ nào**, nên câu hỏi *"bôi đen tiếng Việt thì tra
vào đâu?"* chưa ai đặt ra. Epic 2 đổ nội dung thật vào Editor, và câu hỏi đó nay có câu trả
lời: **không vào đâu cả.**

### 1.2 Đây KHÔNG phải một chỗ sót — nó lật một kết quả đã đo

Phải nói thẳng chỗ này, vì nó quyết định lượt sửa đi đường nào.

`2-3-hop-dong-flush-va-trang-thai-da-luu.md:219` — **AC23**, nguyên văn:

> *"đo xem Auto-Lookup còn chạy trên bề mặt Editor sau khi nó thành vùng gõ… ⇒ **Một phép
> đo, hai kết quả đều hợp lệ:** còn chạy *(hành vi native của `contenteditable` đủ)* ⇒ ghi
> số và đóng; không còn chạy ⇒ đó là một **khuyết tật sản phẩm mới lộ ra**"*

Phép đo đã chạy (Task 6.2), kết quả là **còn chạy**, và nó đã được đóng theo đúng nhánh đó.
Bằng chứng còn nguyên: `tests/frontend/editorAutoLookup.test.ts`, **năm ca**, ba trong số đó
khẳng định *"đường chuột CÒN CHẠY"* · *"đường BÀN PHÍM còn chạy"*.

⇒ AC23 hỏi **"còn chạy không?"** và trả lời đúng. Nhưng nó **không bao giờ hỏi "có NÊN chạy
không?"** — và đó là câu hỏi Ice vừa đặt. Hai câu hỏi khác nhau; phép đo cũ không sai, nó
chỉ không phủ câu hỏi này.

🟢 **Story 2.3 đang `in-progress`.** Nên AC23 sửa được **tại chỗ**, không phải một lượt lật
ngược story đã đóng. Đây là lý do chính khiến lượt sửa này rẻ.

### 1.3 Bằng chứng — và nó mạnh hơn chữ "dư thừa"

Ice nói *"dư thừa"*. Đo ra thì nó **đang phá**, và phá đúng lớp lỗi mà dự án đặt ở trung tâm.

| Bước | Chuyện xảy ra hôm nay |
|---|---|
| 1 | Bôi đen `走廊` ở Panel Source → Panel Lookup hiện ba nguồn |
| 2 | Đọc xong, chuyển sang Editor sửa câu tiếng Việt |
| 3 | Bôi đen `hành lang` để sửa chữ | 
| 4 | `mouseup` → `dispatch('lookup.lookup_selection')` → tra `hành lang` trong từ điển **zh/en** |
| 5 | **0 hàng · 0 lỗi · 0 ms** → Panel Lookup **thay** nội dung Ice đang đọc bằng trạng thái rỗng |

`project-context.md` §*"Rỗng IM LẶNG bị cấm"* gọi đây là **lớp lỗi trung tâm của cả dự án**:

> *"Một truy vấn trả 0 hàng trong 0,01 ms **không ném lỗi nào** và biểu hiện thành 'tra từ
> không ra kết quả' — không ai lần được nguyên nhân."*

Và lý lẽ để chặn nó **đã tồn tại sẵn trong kho**, chỉ chưa ai áp cho hai panel này.
`src/panels/selectionContract.ts:11-17`, viết cho Panel Lookup:

> *"Panel Lookup **tự nó chứa chữ**… Một listener `document` không lọc nguồn dựng một **vòng
> tự thay thế**: bôi đen một nghĩa để đọc kỹ ⇒ một lượt tra mới thay chính đoạn đang đọc,
> cộng một hiệu ứng, cộng một lượt cuộn về đầu. Người dùng mất chỗ và không hiểu vì sao —
> **không test nào bắt được, không cổng nào nhìn thấy**."*

Vòng đó đang mở ở Editor. Cùng cơ chế, cùng hậu quả, khác đúng một chỗ: ở Panel Lookup nó
thay bằng **kết quả khác**, ở Editor nó thay bằng **rỗng** — tệ hơn.

### 1.4 Một mệnh đề của Ice cần được đọc chính xác

Ice viết *"khi chọn thì không tra cứu gì nữa"*. Mệnh đề đó có **hai** cách đọc, và chúng dẫn
tới hai lượt sửa rất khác nhau:

| Cách đọc | Hệ quả |
|---|---|
| ❌ *"vùng chọn ở hai panel đó không còn được đọc"* | Chặn **FR48** (Epic 3) và **FR60** (Epic 7) |
| ✅ *"vùng chọn ở hai panel đó không còn phát lượt tra TỪ ĐIỂN"* | Không chặn gì |

Proposal này cài **cách đọc thứ hai**, và §2.3 giải thích vì sao cách thứ nhất là một cái bẫy.

---

## 2. Phân tích tác động

### 2.1 Tác động Epic

| Epic | Tác động |
|---|---|
| **Epic 1** | Story 1.18 nhận **một AC mới** + **một mệnh đề AC2 được phát biểu lại**. Epic vẫn hoàn thành theo kế hoạch gốc — không AC nào bị gỡ |
| **Epic 2** | Story 2.3 (`in-progress`) — **AC23 đổi từ một phép ĐO thành một mệnh đề**. Story 2.4 (`in-progress`) **không đụng**, xem §2.4 |
| **Epic 3** | **Không đổi.** FR48 / Story 3.3 vẫn cần vùng chọn ở hai panel này — và vẫn có nó |
| **Epic 4** | **Không đổi kế hoạch, nhưng ràng buộc rõ hơn**: khi Epic 4 đổ nội dung vào AI Translation, nội dung đó **không** trở thành nguồn tra cứu. Hôm nay điều đó được ghi thành cổng, không thành một chú thích |
| **Epic 7** | **Không đổi.** FR60 / Story 7.7 (Concordance) — xem §2.3 |

**Không epic nào bị vô hiệu. Không epic mới nào cần thêm. Thứ tự epic không đổi.**

### 2.2 Tác động Story

| Story | Trạng thái | Tác động |
|---|---|---|
| **1.18** Auto-Lookup | `done` | AC2 mệnh đề hai được phát biểu lại; **một AC mới** cho hai panel không-nguồn. Mã đã dựng **không phải sửa cấu trúc** — hợp đồng đã chừa sẵn vai `'display'` |
| **2.3** Hợp đồng flush | `in-progress` | **AC23 viết lại**: từ *"đo, hai kết quả đều hợp lệ"* → *"Editor KHÔNG phát lượt tra"*. `tests/frontend/editorAutoLookup.test.ts` đảo mệnh đề |
| **3.3** Thêm nhanh thuật ngữ | `backlog` | **Không đổi AC.** Nhận một **điều kiện khởi hành** (§4.7) |
| **7.7** Concordance | `backlog` | **Không đổi AC.** Nhận cùng điều kiện khởi hành |

### 2.3 🔴 Ràng buộc phải viết ra: vai `'display'` ≠ *"không lấy được chữ"*

Đây là chỗ dễ hỏng nhất của lượt sửa này, và nó hỏng **ở hai epic sau**, tức đúng khoảng cách
mà không ai còn nhớ proposal này tồn tại.

Hai FR còn sống dựa vào vùng chọn ở chính hai panel này:

| FR | Story | Đòi gì |
|---|---|---|
| **FR48** | 3.3 · Epic 3 | `epics.md:2553` ghi **đích danh**: *"bôi đen một cụm từ ở Panel Source, **Panel Lookup**, **Panel AI Translation** hoặc **Panel Editor**"* → thêm nhanh vào Glossary |
| **FR60** | 7.7 · Epic 7 | Concordance *"cụm này trước đây tôi dịch thế nào?"* — tra **ngược trên bản dịch**, tức nguồn truy vấn tự nhiên chính là tiếng Việt trong Editor |

Chú ý: FR48 liệt kê cả **Panel Lookup** — một panel đã mang vai `'display'` từ Story 1.18.
⇒ Bản thân `epics.md` đã khẳng định sẵn rằng `'display'` **không** loại một bề mặt khỏi
FR48. Proposal này không mở một ngoại lệ mới; nó chỉ làm mệnh đề đó đúng cho hai panel nữa.

**Vì sao không có xung đột:** cả FR48 lẫn FR60 đều là **lệnh người dùng gọi**
*(`epics.md:2554` "gọi lệnh thêm thuật ngữ" · `epics.md:5034` "người dùng gọi lệnh
Concordance")*, không phải phản xạ theo vùng chọn. Chúng sẽ đọc `Selection` bằng đường của
riêng chúng. Thứ vai `'display'` tắt là **đúng một** thứ: `currentSelectionText()`, tức đường
tra **từ điển** *(`selectionContract.ts:203` — trả `''` cho mọi vai khác `'source'`)*.

⇒ **`registerSelectionSurface` phải ở nguyên chỗ trên cả hai panel.** Gỡ nó là chặn hai epic
sau, và `SELECTION_SURFACE_FLOOR = 7` sẽ đỏ ngay — AC23 đã dặn đích danh *"đừng gỡ nó"*.

### 2.4 Tác động kỹ thuật

**Cổng — tin tốt, đã đọc mã cổng để xác nhận, không suy luận.**
`scripts/check-commands.mjs` Kiểm F có ba mệnh đề: ① mỗi panel trong sổ đúng **một** lời gọi
với vai **literal**; ② **chỉ** `LookupPanel.vue` bị ghim vai `'display'`; ③ sàn nội dung 7.

⇒ Đổi `'source'` → `'display'` ở hai panel **đi qua Kiểm F mà không sửa một dòng cổng nào**.
Số lời gọi giữ nguyên **7**, sàn không đụng. Vai là chuỗi literal nên cổng vẫn đọc được.

⚠️ **Và đó chính là chỗ hở.** Sau lượt sửa này, lật ngược về `'source'` là **một từ**, đi qua
sạch mười một cổng, và triệu chứng chỉ lộ khi Epic 4 đổ nội dung thật vào AI Translation —
**hai epic sau**. Đúng tiêu chí vào mục *Critical Don't-Miss* của `project-context.md`:
*"vi phạm được mà không cổng nào đỏ"*. ⇒ **Ice đã chốt: ghim cả hai vào Kiểm F** (§4.6).

**Test — một cái bẫy phải chặn bằng tay.** Đảo năm ca của `editorAutoLookup.test.ts` sang
`not.toHaveBeenCalled()` biến **cả tệp** thành năm mệnh đề *"không có gì xảy ra"*. Một tệp
như vậy **xanh kể cả khi toàn bộ `attachSelectionWatcher` chết**. `project-context.md` cấm
đúng hình dạng đó: *"Một cổng chưa bao giờ đỏ là một cổng chưa ai biết nó có chạy không."*
⇒ Tệp bắt buộc mang một **đối chứng dương** (§4.5).

**NFR2 / NFR18 — Story 2.4 không phải đo lại.** Số của 2.4 được đo **trong khi** Auto-Lookup
còn chạy trên Editor, tức trong khi mỗi vùng chọn còn tốn một vòng IPC. Gỡ nó chỉ **bớt**
việc khỏi đường nóng ⇒ số cũ trở thành **cận trên bảo thủ**, vẫn hợp lệ. Không lượt đo nào
phải chạy lại.

**Kiến trúc — không `AD` mới.** Không bất biến nào đổi. Hợp đồng vùng chọn giữ nguyên hình
dạng; chỉ **dữ liệu khai lúc đăng ký** đổi. `ARCHITECTURE-SPINE.md` **không phải sửa**.

### 2.5 🔴 Một câu cấm phải SỬA, không phải vượt — và nó đã trôi khỏi nguồn của chính nó

**Ice đính chính 2026-08-13:** *"không được thiết kế cho khác đi"* ⇒ **tham khảo và nâng cấp
tối ưu hơn, không phải bê nguyên si.**

Đính chính này đổi bản chất lượt sửa. Mình soạn bản đầu như một **lượt Ice ký đè một câu
cấm**. Đọc lại toàn kho thì thấy: **không có câu cấm nào để vượt.** Câu ở `EXPERIENCE.md:131`
đã **trôi chặt hơn nguồn của nó**, và Ice chỉ đang kéo nó về đúng chỗ.

**Ba bằng chứng, không một lý lẽ nào:**

**① Nguồn của ràng buộc mềm hơn hẳn.** `prd.md:56` — chỗ ràng buộc này thật sự sống:

> *"cộng đồng đã quen với mô hình tra cứu của QuickTranslator. Đây **vừa là lợi thế** (không
> phải dạy lại) **vừa là ràng buộc** — **lệch quá xa** khỏi mô hình đó sẽ bị từ chối."*

*"Lệch **quá xa**"* là một **ranh giới**. *"Không được thiết kế lại cho khác đi"* là một
**lệnh sao chép**. `EXPERIENCE.md` đã biến cái thứ nhất thành cái thứ hai — một lượt siết
không ai quyết.

**② `.memlog.md:11` của chính UX chép đúng bản mềm:** *"mô hình tra cứu KHÔNG được lệch xa
QuickTranslator"*. Nên bản chặt ở `:131` là một lượt trôi **trong cùng một tài liệu**, không
phải một quyết định.

**③ 🔴 Và nó mâu thuẫn với một quyết định Ice đã ký.** `.memlog.md:15-16` — ngày 2026-08-02,
ba hướng thị giác được dựng, trong đó hướng **C — *"Kế thừa QuickTranslator"*** *(khung viền
rõ, tương phản cao, *"không cố tỏ ra hiện đại — cố tỏ ra quen tay"*)*. **Ice LOẠI hướng C**
và chọn hướng B — Bàn viết.

⇒ Ice đã bác *"bê nguyên si QuickTranslator"* **từ ngày thứ nhất của UX**. `DESIGN.md:149`
ghi lại đúng lập trường đó: *"kế thừa sự quen tay của QuickTranslator ở mô hình tra cứu và
bố cục panel, **nhưng thay ngôn ngữ thị giác tiện ích bằng ngôn ngữ của trang sách**"*.

**Kết luận:** `EXPERIENCE.md:131` là **tệp duy nhất trong kho mang bản chặt**, và nó đứng một
mình chống lại PRD, memlog của chính nó, `DESIGN.md`, và lựa chọn hướng của Ice. ⇒ Sửa nó
**không cần một lượt ký đè** — nó là một lượt **dọn một mệnh đề đã trôi**.

**Hệ quả lên lượt thu hẹp FR21:** nó thôi là một ngoại lệ phải xin phép. Nó trở thành **ca
áp dụng đầu tiên** của nguyên tắc vừa được phát biểu cho đúng — giữ **thao tác** đã quen
tay, và sửa **cài đặt** ở chỗ nó tệ hơn.

---

## 3. Phương án và phán quyết

| # | Phương án | Đánh giá |
|---|---|---|
| **1** | **Direct Adjustment** — sửa AC tại chỗ, đổi vai ở hai panel, ghim bằng cổng | ✅ **Khả thi** · công **Thấp** · rủi ro **Thấp** |
| **2** | **Rollback** — lật ngược Story 1.18 / 2.3 | ❌ **Không khả thi và không cần**. Hợp đồng vùng chọn **đúng**; chỉ **hai đối số** sai. Lật ngược để đổi hai từ là phá một tài sản đang tốt |
| **3** | **PRD MVP Review** — xét lại phạm vi MVP | ❌ **Không cần**. Không mục tiêu MVP nào lung lay; đây là thu hẹp một FR cho đúng thực tế, không cắt năng lực |

**Chọn: Phương án 1 — Direct Adjustment.**

**Lý lẽ:** hợp đồng vùng chọn của Story 1.18 **đã chừa sẵn đúng ô này** — `SelectionRole` có
hai giá trị và `LookupPanel` đã dùng `'display'` với **cùng lý do** *(Bẫy 1, vòng tự thay
thế)*. Lượt sửa không dựng cơ chế mới, không thêm nhánh, không thêm phụ thuộc; nó **áp một
lý lẽ đã được chứng minh trong chính kho này cho hai bề mặt bị bỏ sót**. Mã sản phẩm đổi
**đúng hai từ**; phần còn lại là tài liệu, một mệnh đề cổng, và một tệp test đảo chiều.

**Tác động MVP:** **không có.** FR21 hẹp lại nhưng **không mất năng lực nào** — bề mặt bị gỡ
chưa từng trả về một kết quả có ích nào.

---

## 4. Đề xuất sửa chi tiết

> Mười một mục. §4.1–4.6 bắt buộc · §4.7 cổng *(Ice chốt)* · **§4.8 sửa NGUYÊN TẮC** *(Ice
> chỉ đạo — bán kính lớn nhất)* · §4.9 hệ quả gốc của §4.8, **bỏ được** · §4.10–4.11 sổ sách.

### 4.1 `prd.md:417` — FR21

**CŨ**
```
**FR21.** **Auto-Lookup:** bôi đen một cụm từ ở Source, AI Translation hoặc Editor → kết quả
tra cứu hiện **ngay** ở panel Lookup. Không copy, không paste, không chuyển cửa sổ.
```

**MỚI**
```
**FR21.** **Auto-Lookup:** bôi đen một cụm từ ở **Panel Source** — nguyên văn hoặc tab Hán
Việt — → kết quả tra cứu hiện **ngay** ở panel Lookup. Không copy, không paste, không chuyển
cửa sổ.

🔵 **Thu hẹp 2026-08-13 (Sprint Change Proposal), Ice ký.** Mệnh đề cũ liệt kê ba bề mặt:
*"Source, AI Translation hoặc Editor"*. **Panel AI Translation và Panel Editor KHÔNG phải
nguồn tra cứu** — chúng chứa **tiếng Việt đã dịch**, và từ điển nhúng là zh→vi / en→vi. Một
lượt tra ở đó trả **0 hàng, 0 lỗi, 0 ms** rồi **thay mất** kết quả người dùng đang đọc ở
Panel Lookup — đúng vòng tự thay thế mà `selectionContract.ts:11-17` đã bác cho Panel Lookup.
Hai panel đó **vẫn là bề mặt vùng chọn đã đăng ký** *(vai `display`)*: FR48 và FR60 đọc vùng
chọn ở đó qua lệnh của riêng chúng.
```

**Lý do:** FR21 viết lúc hai panel chưa có chữ. Câu mới chở cả **quyết định** lẫn **bằng
chứng**, và ghi rõ thứ **không** bị gỡ — nếu không, Epic 3 sẽ đọc nó thành *"Editor không có
vùng chọn"*.

---

### 4.2 `epics.md:106` — FR21

**CŨ**
```
FR21: **Auto-Lookup:** bôi đen một cụm từ ở Source, AI Translation hoặc Editor → kết quả tra
cứu hiện **ngay** ở Panel Lookup. Không copy, không paste, không chuyển cửa sổ.
```

**MỚI**
```
FR21: **Auto-Lookup:** bôi đen một cụm từ ở **Panel Source** — nguyên văn hoặc tab Hán Việt —
→ kết quả tra cứu hiện **ngay** ở Panel Lookup. Không copy, không paste, không chuyển cửa sổ.
🔵 **Thu hẹp 2026-08-13** *(Sprint Change Proposal, Ice ký)*: Panel AI Translation và Panel
Editor **không** phải nguồn tra cứu — tiếng Việt đã dịch tra vào từ điển zh/en cho **0 hàng**
và **thay mất** kết quả đang hiện. Chúng giữ vai `display` trong hợp đồng vùng chọn, nên
FR48/FR60 không bị chạm.
```

**Lý do:** hai bản FR21 phải khớp từng chữ — một bản trôi là hai nguồn sự thật.

---

### 4.3 `epics.md:1780-1784` — Story 1.18, AC2

**CŨ**
```
**Given** cơ chế Auto-Lookup
**When** đăng ký
**Then** nó gắn vào một **hợp đồng vùng chọn dùng chung cho mọi panel văn bản**
**And** Panel AI Translation và Editor nhận được cùng hành vi khi chúng có nội dung ở các
epic sau, không cần cài lại
```

**MỚI**
```
**Given** cơ chế Auto-Lookup
**When** đăng ký
**Then** nó gắn vào một **hợp đồng vùng chọn dùng chung cho mọi panel văn bản**
**And** **mọi** panel văn bản khai **vai** của mình trong hợp đồng — `source` hoặc `display`
— nên không bề mặt chữ nào đứng ngoài sổ, và một bề mặt mới thêm ở story sau phải khai vai
chứ không được im lặng

🔵 **Sửa 2026-08-13 (Sprint Change Proposal, Ice ký).** Mệnh đề cũ đọc: *"Panel AI Translation
và Editor nhận được cùng hành vi khi chúng có nội dung ở các epic sau, không cần cài lại."*
Nó **hết đúng**: hai panel đó nay mang vai `display` *(FR21 thu hẹp)*. Thứ AC này thật sự
mua thì **vẫn đúng và mạnh hơn** — **hợp đồng không phải sửa một dòng nào** khi nội dung đổ
vào; chỉ **vai khai lúc đăng ký** quyết định hành vi. Chính vì hợp đồng đã chừa sẵn hai vai,
lượt thu hẹp FR21 tốn **đúng hai từ** trong mã sản phẩm.
```

**Lý do:** không xoá mệnh đề cũ — **sửa tại chỗ kèm 🔵 và ngày**, đúng khuôn `project-context.md`
§*"Khi một mệnh đề hết đúng, SỬA TẠI CHỖ thay vì để nó lặng lẽ sai"*. Lịch sử của một quyết
định là bằng chứng cho quyết định kế tiếp.

---

### 4.4 `epics.md` — Story 1.18, **AC mới** *(chèn ngay sau AC2 vừa sửa)*

**THÊM**
```
**Given** người dùng bôi đen một cụm từ ở **Panel AI Translation** hoặc **Panel Editor**
**When** thả chuột, kết thúc vùng chọn bằng bàn phím, **hoặc** bấm phím tra thủ công
**Then** **không lượt tra từ điển nào được phát**
**And** nội dung đang hiện ở Panel Lookup **giữ nguyên** — không bị thay bằng trạng thái rỗng

**Given** chính hai panel đó
**When** một story sau cần vùng chọn ở đó *(FR48 · FR60)*
**Then** vùng chọn **vẫn đọc được** — vai `display` tắt đường tra **từ điển**, không tắt
việc bề mặt được đăng ký
```

**Lý do:** không có AC này thì mệnh đề mới chỉ sống trong một câu 🔵 của FR21, và
`epics.md` là thứ `bmad-check-implementation-readiness` đối chiếu. Mệnh đề thứ hai tồn tại
**riêng** để chặn cách đọc sai ở §1.4 — nó là chỗ Epic 3 sẽ đọc.

---

### 4.5 Mã sản phẩm — hai từ, cộng chú thích chở lý do

**`src/panels/EditorPanel.vue:81`**

```diff
- useSelectionSurface(surface, 'source')
+ // 🔵 2026-08-13 (Sprint Change Proposal, Ice ký) — vai đổi `'source'` → `'display'`.
+ // Bề mặt này chứa **tiếng Việt đã dịch**; từ điển nhúng là zh→vi / en→vi. Một lượt tra ở
+ // đây trả **0 hàng, 0 lỗi, 0 ms** rồi THAY MẤT kết quả người dùng vừa tra từ Panel Source
+ // — đúng vòng tự thay thế mà `selectionContract.ts:11-17` đã bác cho Panel Lookup, chỉ tệ
+ // hơn một bậc vì thứ thay vào là **rỗng**.
+ // 🔴 ĐỪNG gỡ lời gọi này. FR48 (Story 3.3) và FR60 (Story 7.7) đọc vùng chọn ở đây bằng
+ // lệnh của riêng chúng; `'display'` tắt đường tra TỪ ĐIỂN, không tắt việc đăng ký.
+ // Ghim bằng máy: `check-commands.mjs` Kiểm F ③.
+ useSelectionSurface(surface, 'display')
```

**`src/panels/AiTranslationPanel.vue:35`** — cùng lượt sửa. Và khối chú thích `:17-25` mang
một mệnh đề nay **hết đúng**, phải sửa tại chỗ:

```diff
  // 🔴 STORY 1.18 · AC2 — ĐĂNG KÝ HỢP ĐỒNG VÙNG CHỌN, KHÔNG NỘI DUNG
  ...
- // ở các epic sau, **không cần cài lại**"*.
+ // ở các epic sau, **không cần cài lại**"*.
+ //
+ // 🔵 **2026-08-13 — mệnh đề trên đã ĐƯỢC THU HẸP** (Sprint Change Proposal, Ice ký). Panel
+ // này sẽ mang **bản dịch AI tiếng Việt**, nên nó KHÔNG phải nguồn tra cứu: vai nay là
+ // `'display'`. Phần còn đúng của AC2 — và là phần đắt nhất — vẫn nguyên: hợp đồng KHÔNG
+ // phải sửa một dòng nào khi Epic 4 đổ nội dung vào. Chỉ **vai** quyết định hành vi.
- useSelectionSurface(surface, 'source')
+ useSelectionSurface(surface, 'display')
```

**Lý do:** hai tệp này đang mang chú thích khẳng định hành vi cũ. `project-context.md`
§*Bẫy tài liệu* vừa ghi tên đúng lớp lỗi đó *(hai tệp còn viết "dự án không có bộ chạy test
frontend")*. Sửa vai mà để chú thích cũ là dựng cái bẫy thứ ba.

---

### 4.6 `tests/frontend/editorAutoLookup.test.ts` — đảo mệnh đề, **và thêm đối chứng dương**

Năm ca hiện tại đăng ký bề mặt giả bằng `registerSelectionSurface(doc, 'source')` *(dòng 51)*
để mô phỏng `EditorPanel.vue`. Đổi thành `'display'` và đảo ba khẳng định:

| Ca | Cũ | Mới |
|---|---|---|
| 1 · `currentSelectionText()` | `toBe('thổi t')` | `toBe('')` |
| 2 · `mouseup` | `toHaveBeenCalledTimes(1)` | `not.toHaveBeenCalled()` |
| 3 · `keyup` Shift | `toHaveBeenCalledTimes(1)` | `not.toHaveBeenCalled()` |
| 4 · `keyup` không-Shift | `not.toHaveBeenCalled()` | **giữ nguyên** |
| 5 · caret thu gọn | `not.toHaveBeenCalled()` | **giữ nguyên** |

🔴 **CA THỨ SÁU BẮT BUỘC — đối chứng dương.** Sau lượt đảo, cả năm ca đều khẳng định *"không
có gì xảy ra"*, nên **tệp này xanh kể cả khi `attachSelectionWatcher` chết hoàn toàn** — ca 4
và ca 5 trở thành xanh rỗng. Thêm:

```
it('cùng bộ theo dõi VẪN phát trên một bề mặt vai `source` — đối chứng dương', () => {
  // 🔴 Không có ca này, năm ca trên là năm mệnh đề "không có gì xảy ra" và tệp xanh kể cả
  // khi bộ theo dõi chết. `project-context.md` §Luật của một CỔNG: "Một cổng chưa bao giờ
  // đỏ là một cổng chưa ai biết nó có chạy không." Ca này chứng minh **cơ chế còn sống**,
  // nên bốn ca âm tính ở trên nói về **vai**, không về một bộ theo dõi hỏng.
})
```

Ca này đăng ký một bề mặt `'source'` *(mô phỏng `SourcePanel`)*, bôi đen, bắn `mouseup`, và
đòi `toHaveBeenCalledTimes(1)`.

**Doc-comment đầu tệp** *(dòng 1-21)* phải viết lại: nó đang mô tả AC23 như một **phép đo**
với *"hai kết quả đều hợp lệ"*, và mở đầu bằng câu hỏi *"Auto-Lookup còn chạy trên bề mặt
Editor… không?"*. Nay đó là một **mệnh đề**, không một câu hỏi.

---

### 4.7 `scripts/check-commands.mjs` Kiểm F — ghim hai panel *(Ice chốt)*

Chèn sau mệnh đề ② hiện tại *(`LookupPanel` = `'display'`)*, và **đổi số** mệnh đề sàn hiện
tại từ ③ → ④:

```
// ③ Panel AI Translation và Panel Editor phải mang vai `display`, không `source`.
//    🔴 Sprint Change Proposal 2026-08-13 (Ice ký) — FR21 thu hẹp. Hai panel này chứa TIẾNG
//    VIỆT ĐÃ DỊCH; một lượt tra ở đó trả 0 hàng rồi thay mất kết quả đang hiện ở Panel
//    Lookup. Cùng Bẫy 1 mà ② canh cho Panel Lookup, chỉ tệ hơn một bậc vì thứ thay vào là
//    RỖNG.
//
//    ⚠️ VÌ SAO MỆNH ĐỀ NÀY CẦN MỘT CỔNG: lật ngược về `'source'` là ĐÚNG MỘT TỪ, đi qua sạch
//    mười một cổng, và Panel AI Translation hôm nay KHÔNG CÓ CHỮ — nên triệu chứng chỉ lộ ở
//    Epic 4, tức hai epic sau. Đúng tiêu chí §Critical Don't-Miss: "vi phạm được mà không
//    cổng nào đỏ". Cùng khuôn ②, cùng lý lẽ, chỉ khác danh sách tệp.
const DISPLAY_ONLY_FILES = [
  'src/panels/AiTranslationPanel.vue',
  'src/panels/EditorPanel.vue',
]
```

+ vòng lặp `fail()` cùng khuôn ②. **Sàn 7 và `SELECTION_PANEL_FILES` không đụng** — số lời
gọi không đổi, chỉ vai đổi.

**Nghiệm thu bắt buộc — đỏ-rồi-xanh:** lật một trong hai về `'source'` ⇒ Kiểm F **ĐỎ** →
khôi phục ⇒ **XANH**. Ghi vào §Debug Log References của story thi hành. *(`project-context.md`:
"Mỗi cổng phải có phép TỰ KIỂM chứng minh nó ĐỎ ĐƯỢC — và không đỏ oan.")*

---

### 4.8 `EXPERIENCE.md:131` — sửa NGUYÊN TẮC, không chỉ sửa danh sách bề mặt

🔴 **Đây là mục có bán kính lớn nhất trong proposal.** Nó không sửa một đặc tả — nó sửa một
**mệnh đề nguyên tắc** mà mọi quyết định UX sau này sẽ đọc. Ice chỉ đạo trực tiếp 2026-08-13.

**CŨ**
```
**Auto-Lookup** — bôi đen ở bất kỳ panel nào (Nguyên văn, Đề xuất AI, hoặc Bản dịch) thì kết
quả hiện ngay ở Panel Lookup. Không copy, không paste, không hộp thoại. Đây là tương tác lặp
nhiều nhất trong sản phẩm và là thứ cộng đồng đã quen ở QuickTranslator — **không được thiết
kế lại cho khác đi**.
```

**MỚI**
```
**Auto-Lookup** — bôi đen ở **Nguyên văn** (kể cả tab Hán Việt) thì kết quả hiện ngay ở Panel
Lookup. Không copy, không paste, không hộp thoại. Đây là tương tác lặp nhiều nhất trong sản
phẩm, và là thao tác cộng đồng đã quen tay ở QuickTranslator — **thao tác đó không được bắt
học lại**.

🔵 **Sửa 2026-08-13 — Ice, và đây là một lượt SỬA NGUYÊN TẮC, không một lượt ngoại lệ.**
Bản cũ đóng lại bằng *"**không được thiết kế lại cho khác đi**"*. Mệnh đề đó **chặt hơn nguồn
của chính nó** và **mâu thuẫn với một quyết định Ice đã ký**:

- `prd.md:56` — chỗ ràng buộc này thật sự sống — chỉ nói *"**lệch quá xa** khỏi mô hình đó sẽ
  bị từ chối"*. Đó là một **ranh giới**, không một lệnh sao chép.
- `.memlog.md:11` của chính tài liệu này chép đúng bản mềm: *"KHÔNG được lệch xa"*.
- 🔴 Ngày **2026-08-02**, ba hướng thị giác được dựng và hướng **C — *"Kế thừa
  QuickTranslator"*** *(khung viền rõ, tương phản cao, "không cố tỏ ra hiện đại — cố tỏ ra
  quen tay")* **bị Ice LOẠI**; Ice chọn hướng B — Bàn viết *(`.memlog.md:15-16`)*.
  `DESIGN.md:149` ghi lại nguyên văn lập trường đó.

⇒ **Lập trường đúng, và nó áp cho toàn sản phẩm: QuickTranslator là mốc tham khảo để VƯỢT
QUA, không phải bản mẫu để bê nguyên si.** Hai vế, và ranh giới giữa chúng là thứ phải đọc
được:

| | |
|---|---|
| **BẤT BIẾN — *thao tác*** | Bôi đen là ra kết quả, **tức thì**, không copy/paste, không hộp thoại. Đây là thứ cộng đồng đã quen tay và **không được bắt học lại** — đúng cái `prd.md:56` gọi là *"không phải dạy lại"* |
| **MỞ — *cài đặt*** | Bề mặt nào là nguồn · kết quả trình bày ra sao · nguồn nào hiện · ngôn ngữ thị giác · bố cục. Ở đâu AuraTranslate làm **tốt hơn** được thì **phải làm tốt hơn** |

🔴 **Và vế thứ hai có chiều bắt buộc, không chỉ chiều cho phép:** giữ một khuyết điểm của
QuickTranslator chỉ vì *"cộng đồng đã quen"* là **chép cả cái dở**. Một mệnh đề *"QT làm thế"*
**không bao giờ đủ** để đóng một lựa chọn thiết kế — nó là dữ kiện, không phải lý lẽ.

**Ca áp dụng đầu tiên, ngay trong lượt sửa này:** bản cũ liệt kê *"bất kỳ panel nào (Nguyên
văn, Đề xuất AI, hoặc Bản dịch)"*. Hai bề mặt sau chứa **tiếng Việt đã dịch**; tra chúng
trong một từ điển zh→vi / en→vi cho **0 hàng** và **thay mất** thứ người dùng đang đọc *(FR21,
§4.1)*. **Thao tác** giữ nguyên; **cài đặt** sửa ở đúng chỗ nó tệ hơn.
```

**Lý do:** một mệnh đề nguyên tắc trôi chặt hơn nguồn của nó là thứ sẽ **bác oan** những
quyết định đúng ở các epic sau — và nó sẽ bác chúng bằng một câu trích dẫn nghe rất dứt
khoát. Sửa tại chỗ kèm 🔵 và ngày, giữ nguyên bản cũ trong câu giải thích: lịch sử của một
mệnh đề là bằng chứng cho mệnh đề kế tiếp.

---

### 4.9 `prd.md:56` — bổ sung vế còn thiếu của chính ràng buộc *(MỚI, do §4.8 kéo theo)*

`prd.md:56` là **nguồn** của ràng buộc QuickTranslator, và nó chỉ phát biểu **một** chiều —
chiều **rủi ro** *(lệch quá xa sẽ bị từ chối)*. Nó chưa bao giờ nói chiều còn lại: rằng
QuickTranslator cũng là một **sàn phải vượt**. Chính khoảng trống đó là chỗ
`EXPERIENCE.md:131` trôi vào.

**THÊM** một câu vào cuối khối trích dẫn ở `prd.md:56`:

```
🔵 **Bổ sung 2026-08-13 (Ice):** ràng buộc này là một **ranh giới**, không một lệnh sao chép.
QuickTranslator là mốc **tham khảo để vượt qua** — bất biến là **thao tác** người dùng đã
quen tay *(bôi đen là ra kết quả, tức thì, không copy/paste)*; **cài đặt** thì mở, và ở đâu
làm tốt hơn được thì **phải làm tốt hơn**. Giữ một khuyết điểm của QuickTranslator vì
*"cộng đồng đã quen"* là chép cả cái dở. Xem `EXPERIENCE.md` §Interaction Primitives.
```

**Lý do:** sửa `EXPERIENCE.md` mà để PRD thiếu vế thứ hai là dọn triệu chứng rồi để nguyên
nguyên nhân — mệnh đề sẽ trôi chặt lại ở tài liệu tiếp theo. ⚠️ **Đây là mục duy nhất trong
proposal vượt ra ngoài phạm vi Ice nêu** *(Ice nói về câu ở `EXPERIENCE.md`)*. Mình đề xuất
vì nó là nguyên nhân gốc, nhưng nó **bỏ được** mà chín mục kia vẫn đứng vững — Ice cắt thì
mình cắt.

---

### 4.10 `_bmad-output/specs/spec-AuraTranslate/requirements.md:133` — **BẢN FR21 THỨ BA**

🔴 **Chỗ này suýt lọt.** Kho có **ba** bản FR21 sống *(không phải hai)*: PRD · `epics.md` ·
và **SPEC**. Bản thứ ba là *"hợp đồng máy chính tắc cho công việc xuôi dòng"* — để nó trôi là
dựng đúng thứ nguồn-sự-thật-thứ-hai mà cả dự án được tổ chức để chống.

**CŨ** *(giống hệt bản PRD)*
```
- **FR21.** **Auto-Lookup:** bôi đen một cụm từ ở Source, AI Translation hoặc Editor → kết quả
tra cứu hiện **ngay** ở panel Lookup. Không copy, không paste, không chuyển cửa sổ.
```

**MỚI:** chép **nguyên văn** thân bài của §4.1, giữ tiền tố `- **FR21.**` của tệp này.

**Lý do:** ba bản phải khớp **từng chữ**. Đây cũng là lý do tiêu chí §5.4 dùng một lệnh `grep`
trên **toàn `_bmad-output/`** thay vì một danh sách tệp — danh sách tệp là thứ vừa suýt sót.

**⚠️ HAI tệp KHÔNG được sửa** — cả hai là ảnh chụp có ngày, sửa chúng là viết lại lịch sử:

| Tệp | Vì sao để nguyên |
|---|---|
| `implementation-readiness-report-2026-08-03.md:117` | Báo cáo có ngày — nó ghi lại điều đã đúng **ngày 2026-08-03** |
| `brief.md:36` | **Đã kiểm: KHÔNG sai.** Câu này viết *"Auto-Lookup đưa kết quả tra cứu ra panel Lookup ngay khi bôi đen"* — nó **không liệt kê panel nào**, nên không mệnh đề nào hết đúng. Không cần một chữ nào |

---

### 4.11 `deferred-work.md` — mục mới

```
## Deferred from: Sprint Change Proposal 2026-08-13 (FR21 thu hẹp)

- 🔵 **AC23 của Story 2.3 ĐỔI TỪ MỘT PHÉP ĐO THÀNH MỘT MỆNH ĐỀ.** AC23 hỏi *"Auto-Lookup còn
  chạy trên bề mặt Editor không?"* và đo ra **còn chạy**, đóng theo nhánh hợp lệ. Nó **không
  bao giờ hỏi "có NÊN chạy không?"** — Ice đặt câu hỏi đó ngày 2026-08-13 và trả lời:
  **không**. Phép đo cũ **không sai**; nó không phủ câu hỏi này. ⇒ AC23 nay đọc: *"Editor
  KHÔNG phát lượt tra từ điển"*, nghiệm thu bằng `editorAutoLookup.test.ts` (đã đảo) + Kiểm F
  ③. **Không món nợ nào mở ra từ mục này** — ghi để lượt retro Epic 2 thấy được vì sao một AC
  đã đóng lại đổi nghĩa.

- ⚠️ **CHỦ: Story 3.3 (FR48) và Story 7.7 (FR60) — điều kiện khởi hành.** Vai `'display'` của
  `AiTranslationPanel.vue` / `EditorPanel.vue` tắt **đúng một** đường: `currentSelectionText()`,
  tức tra từ điển. Nó **KHÔNG** tắt việc bề mặt được đăng ký. Hai story trên đọc vùng chọn ở
  hai panel đó bằng đường của **riêng chúng** *(cả hai là lệnh người dùng gọi)*, **không** qua
  `currentSelectionText()`. 🔴 Đọc nhầm `'display'` thành *"không lấy được chữ"* sẽ dẫn tới một
  lượt "sửa" gỡ đăng ký hoặc lật vai — và `epics.md:2553` đã liệt kê **Panel Lookup** (vai
  `display` từ 1.18) trong chính danh sách FR48, nên tiền lệ đã có sẵn.
```

**Lý do:** `project-context.md` — *"Mọi thứ không nghiệm thu được ở story hiện tại đi vào
đây, KÈM MỘT CHỦ. Không có mục nào mồ côi."*

---

## 5. Bàn giao

### 5.1 Phân loại phạm vi: **MINOR** — với một chú thích phải đọc

**Về thi hành: Minor.** Mã sản phẩm đổi **hai từ**. Không `AD` mới, không lược đồ, không phụ
thuộc, không cổng mới *(một mệnh đề thêm vào cổng đã có)*. Không epic nào bị vô hiệu, không
epic mới, thứ tự không đổi. Story duy nhất đang mở bị chạm — **2.3** — nhận một AC viết lại,
không một task mới. Không cần PM/Architect vào lại; Amelia làm trọn được.

⚠️ **Nhưng §4.8 KHÔNG minor về ảnh hưởng.** Nó sửa một mệnh đề **nguyên tắc**, và mệnh đề
nguyên tắc thì được trích dẫn ở các epic sau để **đóng** những lựa chọn thiết kế. Bản cũ —
*"không được thiết kế lại cho khác đi"* — sẽ bác oan mọi lượt cải tiến C2/C3 bằng một câu
nghe rất dứt khoát; bản mới mở đúng cánh cửa đó và **đóng lại một cánh khác** *(một mệnh đề
"QT làm thế" thôi không còn đủ để đóng một lựa chọn)*.

⇒ Phân loại **Minor** ở đây nói về **đường thi hành**, không nói về **tầm**. Đề nghị nêu §4.8
thành một mục riêng ở **retro Epic 2** — nó là loại thay đổi mà sáu tháng nữa sẽ có người hỏi
*"ai đổi câu này, và dựa vào gì?"*.

### 5.2 Người nhận

| Vai | Việc |
|---|---|
| **Amelia (Developer)** | Toàn bộ §4.1–4.11 |
| **Ice** | **Không còn chỗ nào phải ký đè** — §4.8 nay là chỉ đạo của chính Ice, không một ngoại lệ xin phép. Còn lại **một** quyết định: giữ hay cắt §4.9 *(bổ sung `prd.md:56`)*, và nghiệm thu bằng mắt trên `tauri dev` thật sau khi vá |

### 5.3 Thứ tự thi hành — có phụ thuộc, đừng đảo

1. **§4.5 mã** *(hai từ + chú thích)* → chạy `npm run check:commands` ⇒ phải **XANH** *(chứng
   minh Kiểm F hiện tại không chặn lượt đổi vai — nếu đỏ thì phân tích §2.4 sai, **dừng lại
   và báo**, đừng vá cổng cho hết đỏ)*
2. **§4.6 test** → `npm run test` ⇒ xanh, **kèm ca đối chứng dương**
3. **§4.7 cổng** → nghiệm thu **đỏ-rồi-xanh**
4. **§4.1–4.4, §4.8–4.11 tài liệu** — làm **§4.8 trước**: nó phát biểu nguyên tắc mà §4.1–4.4
   là ca áp dụng, nên viết ngược thứ tự sẽ ra một FR21 không ai đọc được lý do
5. `npm run test` · 11 cổng · `npm run build` · `cargo test --locked` ⇒ tất cả xanh

⚠️ **Cây làm việc sạch lúc bắt đầu** *(`3215e68`)* — lượt sửa này commit riêng, diff đọc được
một mình. Commit message theo khuôn: `fix(scope): câu nói ĐIỀU ĐÃ TÌM RA`.

### 5.4 Tiêu chí thành công

- [ ] Bôi đen tiếng Việt trong Editor ⇒ Panel Lookup **giữ nguyên** nội dung *(mắt Ice, `tauri dev` thật)*
- [ ] `Mod+Alt+L` trong Editor ⇒ **không** lượt tra nào *(Ice chốt 2026-08-13)*
- [ ] Bôi đen ở Panel Source ⇒ tra bình thường, **không hồi quy**
- [ ] Kiểm F ③ chứng minh **ĐỎ ĐƯỢC** *(lật một panel về `'source'`)*
- [ ] `editorAutoLookup.test.ts` mang **đối chứng dương** — không phải sáu mệnh đề *"không có gì xảy ra"*
- [ ] 11 cổng · vitest · build · `cargo test --locked` xanh
- [ ] `grep -rn "không được thiết kế lại cho khác đi" _bmad-output/` ⇒ **chỉ còn trong câu
      giải thích của §4.8 và trong proposal này** — không chỗ nào còn phát biểu nó như một
      mệnh đề đang hiệu lực
- [ ] `grep -rn "AI Translation hoặc Editor" _bmad-output/` ⇒ **đúng 1 kết quả**, và nó là
      `implementation-readiness-report-2026-08-03.md` *(ảnh chụp có ngày, cố ý để nguyên)*.
      Ba bản FR21 sống — PRD · `epics.md` · SPEC — **không bản nào còn trôi**

---

## Phụ lục — Change Navigation Checklist

| Mục | Trạng thái | Ghi chú |
|---|---|---|
| 1.1 Story kích hoạt | [x] | Không phải một story — Ice dùng thật. Lật kết quả AC23 của Story 2.3 (`in-progress`) |
| 1.2 Định nghĩa vấn đề | [x] | *Misunderstanding of original requirements* — FR21 viết khi hai panel chưa có chữ |
| 1.3 Bằng chứng | [x] | `selectionContract.ts:203` không lọc ngôn ngữ ⇒ 0 hàng, 0 lỗi, panel bị thay bằng rỗng |
| 2.1 Epic hiện tại | [x] | Epic 2 hoàn thành được như kế hoạch; chỉ AC23 của 2.3 đổi nghĩa |
| 2.2 Đổi cấp epic | [N/A] | Không thêm/bớt/định nghĩa lại epic nào |
| 2.3 Epic còn lại | [x] | Epic 3 (FR48) và Epic 7 (FR60) rà kỹ — **không bị chặn**, xem §2.3 |
| 2.4 Epic bị vô hiệu / cần mới | [N/A] | Không |
| 2.5 Thứ tự / ưu tiên epic | [N/A] | Không đổi |
| 3.1 PRD | [!] | **Hai chỗ:** FR21 (`prd.md:417`) — §4.1 · ràng buộc QuickTranslator (`prd.md:56`) — §4.9. MVP **không** bị chạm |
| 3.2 Architecture | [x] | **Không xung đột.** Không `AD` nào đổi; hợp đồng giữ nguyên hình dạng |
| 3.3 UI/UX | [!] | 🔴 **Bán kính lớn nhất của lượt này.** `EXPERIENCE.md:131` mang một mệnh đề **nguyên tắc đã trôi chặt hơn nguồn của nó** *(và mâu thuẫn với lượt Ice loại hướng C ngày 2026-08-02)* — §2.5 · §4.8. Không phải một lượt ký đè: một lượt **dọn** |
| 3.4 Tạo tác khác | [!] | **SPEC `requirements.md:133` — bản FR21 thứ ba, suýt sót** · cổng `check-commands.mjs` Kiểm F · `editorAutoLookup.test.ts` · `deferred-work.md`. Hai ảnh chụp có ngày *(`implementation-readiness-report`, `brief.md`)* **để nguyên** — §4.9 |
| 4.1 Direct Adjustment | [x] Khả thi | Công **Thấp** · rủi ro **Thấp** — **ĐÃ CHỌN** |
| 4.2 Rollback | [x] Không khả thi | Hợp đồng đúng; chỉ hai đối số sai |
| 4.3 PRD MVP Review | [x] Không cần | Không năng lực nào bị cắt |
| 4.4 Chọn đường | [x] | Phương án 1 — §3 |
| 5.1–5.5 Thành phần proposal | [x] | §1–§5 |
| 6.4 `sprint-status.yaml` | [!] | Không epic/story nào thêm/bớt ⇒ **chỉ thêm dòng chú** ghi lượt sửa AC23 của 2.3 |

Sau đính chính của Ice, các mục `[!]` **không còn chỗ nào cần ký đè**. Còn đúng **một** quyết
định của Ice: giữ hay cắt **§4.9** *(bổ sung `prd.md:56`)* — mục duy nhất mình đề xuất vượt ra
ngoài phạm vi Ice nêu.
