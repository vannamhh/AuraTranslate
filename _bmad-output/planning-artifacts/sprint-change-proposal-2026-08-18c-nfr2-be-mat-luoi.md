# Sprint Change Proposal — 2026-08-18c

**Nửa NFR2 của Story 2.4 đo một bề mặt đã bị xoá — và số thay thế đã nằm sẵn trong sổ nợ**

| | |
| --- | --- |
| **Ngày** | 2026-08-18 |
| **Người soạn** | Dev *(lượt thi hành Story 2.4)* |
| **Người ký** | Ice |
| **Story kích hoạt** | `2-4-mui-tham-do-do-nfr18-va-nfr2-dong-thoi` |
| **Mốc gốc rà** | `c097eb3` *(story neo ở `6a4e6b8`, cách 47 commit)* |
| **Phạm vi Ice ký** | rà **trọn 22 AC** của Story 2.4 · chế độ **trọn gói** |
| **Hạng thay đổi** | **Moderate** — viết lại AC, không đổi một ngưỡng NFR nào |

---

## 1. Tóm tắt vấn đề

Story 2.4 là mũi thăm dò đo **NFR2** *(không frame nào vượt 50 ms trong lúc auto-save chạy)* và
**NFR18** *(mất ≤ 5 giây công việc)* **đồng thời**. Nó được soạn ngày 2026-08-13 trên bề mặt Editor
lúc đó: `EditorPanel.vue`, **một dòng văn liên tục** gồm các `<span class="sent">` trong một `.doc`.

Lượt **correct-course 2026-08-14** thay trọn bề mặt đó bằng **lưới hai cột** `GridPanel.vue`
*(Story 2.5b, `Supersedes:` 4/8 AC của Story 2.2)*. Chẩn đoán gốc trong `epics.md` rất trung thực:

> *"Không có lỗi cài đặt nào ở gốc: **ba quyết định ĐÚNG chồng lên nhau thành một bề mặt SAI**.
> Nguyên nhân gốc nằm ở HÌNH DẠNG — một dòng văn liên tục là hình dạng để **ĐỌC** một bản dịch
> đã xong."*

**Vấn đề:** Story 2.4 không được rà lại sau lượt đó. Nửa **NFR18** của nó không hề hấn gì — nó đo
đường ghi Rust, độc lập với hình dạng DOM. Nhưng nửa **NFR2** neo vào bốn cái tên cụ thể, và cả bốn
nay trỏ vào hư không.

### Bằng chứng — đếm trên cây nguồn hôm nay, không suy luận

| Cái tên AC gọi | Chỗ AC trỏ tới | Đếm được hôm nay |
| --- | --- | --- |
| `nearestSentenceTo()` | `EditorPanel.vue:565`, `:602`, `:636` | **0** chỗ trong toàn `src/` |
| `:data-caret` | `EditorPanel.vue:892` | chỉ còn trong **một chú thích** (`editorPanelState.ts:67`) |
| `restoreEditedText()` | `EditorPanel.vue:294`, `:300` | 🟢 **còn sống** — `GridPanel.vue:843`, watcher `:859` |
| `.doc` · `.sent` *(bench.js hỏi)* | `bench.js:152`, `:160`, `:199` | **0** chỗ sống; chỉ còn trong chú thích của `GridPanel.vue:839`, `:1879` |
| `EditorPanel.vue` | — | **tệp không tồn tại** |

⇒ Ba đầu dò của `bench.js` mở đầu bằng `document.querySelector('.doc')` rồi thoát với
*"KHÔNG THẤY .doc"*. Chúng **đo rỗng** — không phải đo ra một số xấu. Một bảng số đọc ra từ đó là
đúng lớp lỗi *"xanh rỗng"* mà AC15 của Story 2.3 đã đặt tên và mà chính AC21 của Story 2.4 cấm.

### Và số thay thế ĐÃ CÓ — nó chỉ chưa được nối vào AC

`deferred-work.md:3245-3262` *(Story 2.5b, 2026-08-15, WKWebView 605.1.15, bản dựng thật)*:

> 🔴 **TASK 8 ĐÃ ĐO — SỐ GIAO CHO STORY 2.4. MỘT ĐƯỜNG VƯỢT TRẦN NFR2 15 LẦN.**

| Phép đo | 2.000 câu | **9.850 câu** |
| --- | --- | --- |
| node DOM trong lưới | 10.005 | **49.256** *(5 node/câu — đúng năm cột)* |
| một lượt `selectionchange` + 2 frame | 12 / 34 / 34 ms | 24 / 33 / 33 ms |
| một lượt **DỜI CON TRỎ** | 226 / 173 / 195 / 189 / 161 ms | 🔴 **770 / 706 / 767 ms** |

Cùng mục đó đã tự viết ra vì sao mốc cũ không dùng được nữa:

> ⚠️ **Mốc cũ mất hiệu lực THEO CẤU TRÚC, không bị đóng** — `:2113-2129` đo *"dựng 9.850 `<span>`"*
> (300,1 ms Blink · 1.308,0 ms WebKit). Lưới không dựng `<span>` nào; nó dựng **49.256 node** trong
> năm cột `subgrid`. Hai con số **không so được với nhau**, và ghi chúng cạnh nhau như một lượt
> *"cải thiện"* là **nói dối**.

🔴 **Đây là chỗ nặng nhất của cả đề xuất:** một vi phạm NFR2 **~15 lần trần** đang sống, đã đo, đã
ghi *"Chủ: Story 2.4"* ở **ba** chỗ *(`:3579` từ 2.5c · `:3680` · `:4707` từ 2.11)* — trong khi
người chủ của nó bị đóng băng vì AC của nó trỏ vào một bề mặt không còn. Retro Epic 2 §F6 gọi đúng
tên: *"con số lớn dần trong khi người chủ của nó bị đóng băng. Đó không phải giao nợ, đó là **xếp
nợ vào một cái hộp khoá**."*

---

## 2. Phân tích tác động

### 2.1 Tác động Epic

| Mục | Phán quyết |
| --- | --- |
| Epic 2 hoàn thành được như quy hoạch? | 🟡 **Được, nhưng cửa chặn B1 của retro vẫn đứng** — Epic 3 không mở trước khi 2.4 ra một con số NFR2 |
| Cần thêm/bỏ/định nghĩa lại Epic nào? | **Không.** Đây là một lượt sửa AC trong một story, không phải một lượt đổi Epic |
| Epic sau bị ảnh hưởng? | **Epic 3** — retro đã ghi: *"Epic 3 trang trí thêm lên đúng đường nóng đó"*. Thứ tự `2-4 → 2-12 → Epic 3` Ice ký 2026-08-18 **không đổi** |
| Đổi thứ tự hay ưu tiên? | **Không** |

### 2.2 Tác động tạo tác

| Tạo tác | Có xung đột không |
| --- | --- |
| **PRD** | 🟢 **Không.** NFR2 và NFR18 giữ nguyên từng chữ. Thứ đổi là *đo trên bề mặt nào*, không phải *ngưỡng là bao nhiêu* |
| **`epics.md`** | 🟡 **Một chỗ** — `:2204`, AC4 gốc của Story 2.4, còn ghi *"thư viện editor cho **Panel Editor**"*. Tên panel chết. Cùng lớp với action item **B4** của retro *(đã đóng cho Epic 3 + Epic 6, sót Epic 2)* |
| **`ARCHITECTURE-SPINE.md`** | 🟡 **Không xung đột nội dung**, nhưng **ba số dòng đã trôi**: story trỏ `:894/:897/:899`, thực tế `:990/:993/:995` |
| **UX** | 🟢 **Không.** Lượt 2026-08-14 đã xử vế UX; đây là vế hiệu năng |
| **Tệp story 2.4** | 🔴 **Chỗ chính** — AC12, AC13 phải viết lại; AC4, AC14, AC15, AC17 phải sửa dữ kiện |
| **Mã sản phẩm** | 🟢 **Không đổi một dòng** trong lượt này |
| **Bàn đo `2-4-ban-do/`** | 🟡 `bench.js` phải đổi ba đầu dò DOM sang bề mặt lưới — **việc của dev**, sau khi AC được ký |

---

## 3. Rà TRỌN 22 AC — Ice ký phạm vi này

Ký hiệu: 🟢 giữ nguyên · 🟡 sửa dữ kiện *(số, tên, đường dẫn)* · 🔴 viết lại nội dung.

| AC | Nói gì | Phán quyết | Vì sao |
| --- | --- | --- | --- |
| **AC1** | phiên gõ ≥ 30 phút, không frame > 50 ms **trong lúc auto-save chạy** | 🟡 | Tiền đề *"Chương thật"* Ice đã xử 2026-08-13 *(thang nhân tạo)*. **Nhưng xem §3.1 — AC1 không phủ đường đang vi phạm** |
| **AC2** | ≥ 20 lượt kill, mất ≤ 5 s, dung sai đã ghim | 🟢 | Đo đường ghi Rust, độc lập hình dạng DOM |
| **AC3** | chọn giá trị ngưỡng WAL đạt cả hai, ghi vào hàng Deferred | 🟡 | Nội dung đúng; số dòng `:894` → **`:990`** |
| **AC4** | thư viện editor cho **Panel Editor**, tuân AD-31 | 🟡 | Tên panel chết. Và câu hỏi nay **sống hơn trước**: ô lưới là `contenteditable` |
| **AC5** | hai ngưỡng không đạt đồng thời ⇒ đổi tầng PRD | 🟢 | Cơ chế leo thang giữ nguyên. Xem §3.1 về điều kiện kích hoạt |
| **AC6** | báo cáo theo khuôn mũi thăm dò, bảy mục | 🟢 | Khuôn và đường dẫn không phụ thuộc bề mặt |
| **AC7** | sáu số `Tuning` đều có phán quyết | 🟢 | Tầng Rust |
| **AC8** | NFR18 ⟷ ngưỡng WAL đo tại mọi điểm lưới | 🟢 | Bộ chạy `run-grid.sh` đã giao 2026-08-18 |
| **AC9** | kill là `SIGKILL`, phân ba loại | 🟢 | Đã tự kiểm 7/7 |
| **AC10** | kịch bản hai kho, ba vế | 🟢 | Tầng Rust |
| **AC11** | định nghĩa đo được cho *"frame"* và *"mất mấy giây"* | 🟡 | Định nghĩa giữ nguyên; **cửa sổ đo** phải thêm đường dời con trỏ *(§3.1)* |
| **AC12** | **ba đường nóng** `:data-caret` · `restoreEditedText()` · `nearestSentenceTo()` | 🔴 | Hai trong ba không còn tồn tại |
| **AC13** | trần 9.850 span, so với 300,1 / 1.308,0 ms | 🔴 | Lưới không dựng span; phép so **mất hiệu lực theo cấu trúc** |
| **AC14** | đổi hằng ở đúng chỗ khai; sửa `SPINE:883` trong `editorFlush.ts` | 🟡 | Đích đúng nay là **`:990`**, không phải `:894` như AC đang ghi |
| **AC15** | lưới hiện có xanh lại: `npm run test` ≥ 32 · `cargo` ≥ 319 | 🟡 | Sàn trôi: thật hôm nay **249** và **409** |
| **AC16** | 0 dependency runtime mới, ba gói ghim | 🟢 | Đã kiểm: `@tauri-apps/api 2.11.1` · `dockview-vue 7.0.4` · `vue 3.5.40` — **khớp từng số** |
| **AC17** | chụp lại 3 ảnh bàn đo 2.2 · sửa lời khai NFR15 | 🔴 | Bề mặt của 2.2 đã bị thay; chụp lại ảnh của một bề mặt chết là việc vô nghĩa. Vế NFR15 thì **vẫn đúng** |
| **AC18** | đỉnh RSS lượt nhập 100 MB | 🟢 | Tách rời khỏi cặp NFR2/NFR18 |
| **AC19** | mọi số lặp lại được, n=3 cho AC1, cấm gộp mẫu | 🟢 | |
| **AC20** | khoảng mù Windows nói bằng chữ | 🟢 | `deferred-work.md:1954-1957` đã kiểm, còn đúng |
| **AC21** | chi phí bàn đo đo được hoặc nói ra | 🟡 | Bốn nguồn giữ nguyên; nguồn *"vòng rAF"* nay lấy mẫu trên lưới |
| **AC22** | trạng thái máy lúc đo | 🟢 | Đã chạy 2026-08-18 |

**Tổng: 🟢 12 · 🟡 7 · 🔴 3.**

### 3.1 🔴 Phát hiện của lượt rà: AC1 KHÔNG phủ đường đang vi phạm

Đây là chỗ lượt rà trọn-22-AC trả lại giá trị lớn nhất, và nó **không** nằm trong ba AC bị đánh 🔴.

NFR2 nguyên văn: *"không frame nào vượt 50 ms **trong lúc auto-save chạy**"*. AC1 chép đúng vế đó,
và Quyết định #3 của story còn ghim thêm ranh giới: lượt **dựng Chương** *(300,1 / 1.308,0 ms)*
**không** nằm trong lúc auto-save chạy, nên nó là số giao cho hàng Deferred ảo hoá, không phải số
nghiệm thu AC1.

**Nhưng đường đang vượt trần 15 lần là đường DỜI CON TRỎ**, và nó cũng không nằm trong lúc auto-save
chạy. ⇒ Theo đúng câu chữ, Story 2.4 có thể cho **AC1 xanh** trong khi một vi phạm NFR2 **~15 lần**
sống nguyên vẹn trên bề mặt chính của sản phẩm.

⚠️ Đây **không** phải một lỗi soạn AC. Lúc AC1 được viết, đường dời con trỏ chưa tồn tại — bề mặt cũ
đặt `contenteditable` lên **đúng một** `<span>` tại một thời điểm. Lưới đặt nó lên **mọi** ô. Cơ chế
mới đẻ ra một đường nóng mới, và AC được viết trước nó.

🔴 **Đây là hàng của §Cần Ice quyết, không phải chỗ dev tự nới** — vì mở rộng cửa sổ đo của AC1 là
mở rộng thứ Story 2.4 phải chịu trách nhiệm, và điều đó đổi cả điều kiện kích hoạt của **AC5**
*(leo thang lên tầng PRD)*. Xem §5, mục ⑶.

---

## 4. Đề xuất sửa cụ thể

### ⑴ 🔴 AC12 — viết lại trên bề mặt lưới

**CŨ:**
> **AC12 — ba đường nóng của §Điều kiện khởi hành mục 6 đều có SỐ.** `:data-caret` ·
> `restoreEditedText()` · `nearestSentenceTo()`. Mỗi cái: chi phí một lượt, ở Chương **thật** và ở
> Chương **9.850 câu**.

**MỚI:**
> **AC12 — ba đường nóng của bề mặt LƯỚI đều có SỐ.** Mỗi cái: chi phí một lượt, ở thang nhân tạo
> và ở Chương **9.850 câu**.
>
> | Đường | Chỗ | Vì sao nó đắt |
> | --- | --- | --- |
> | **dời con trỏ** — `placeCaretAtPoint()` + `ensureCaretNextFrame()` | `GridPanel.vue:459`, `:766` | 🔴 **đã đo 706–770 ms ở 9.850 câu** *(2.5b, `deferred-work.md:3252`)*, vượt trần NFR2 ~15×. Đây là đường **thường nhất** của tính năng |
> | `onSelectionChange()` → `setEditorCaret()` | `GridPanel.vue:875`, đăng ký `:885` | kế thừa trực tiếp của `:data-caret`: nó ghi vào một ref phản ứng **đọc trong render**, nên mỗi lượt `selectionchange` chạy lại `v-for` trên **năm** cột. Đã đo 24–34 ms ở 9.850 câu |
> | `restoreEditedText()` | `GridPanel.vue:843`, watcher `:859` | **đường duy nhất sống sót từ AC12 bản cũ** — `querySelectorAll('[data-segment-id]')` trên cả Chương mỗi lượt dựng lại |
>
> 🔴 **Vá chỉ khi số nói cần**, và mỗi bản vá kèm **đỏ-rồi-xanh** cộng một dòng nói vì sao nó không
> phải thứ hàng Deferred *"ảo hoá danh sách dài"* sẽ làm lại từ đầu ở Giai đoạn 3.
>
> ⚠️ **Ba biểu thức `:class` boolean mà Story 2.5c thêm vào bốn trong năm `v-for`** *(39.400 phép
> đọc thuộc tính mỗi lượt dời con trỏ ở 9.850 câu — `deferred-work.md:3572-3583`)* phải được đo
> **trên cây sau 2.5c**, và so với **706–770 ms**, không so với một mốc trước lưới.

**Lý do:** hai trong ba cái tên cũ đếm được **0** chỗ trong `src/`. Ba đường mới là ba đường nóng
thật của bề mặt hiện tại, cả ba đã có `tệp:dòng` kiểm được, và một trong ba **đã có số đo**.

### ⑵ 🔴 AC13 — đổi mốc so, và đổi cả ĐƯỜNG được đo

**CŨ:** đo lại trần dựng **9.850 `<span>`** với chữ thật, **trên đúng bàn Playwright hai engine cũ**,
đặt cạnh *300,1 ms Blink · 1.308,0 ms WebKit*; ngưỡng đòi mở ảo hoá sớm: `< 1,4 s` giữ · `1,4–2,0 s`
ghi điều kiện · `> 2,0 s` hoặc tăng > 50 % thì đòi mở.

**MỚI:**
> **AC13 — trần của bề mặt LƯỚI được đo lại, và story này KHÔNG dựng ảo hoá.**
>
> 🔴 **Mốc cũ *(300,1 / 1.308,0 ms)* KHÔNG được đặt cạnh số mới.** Nó đo *"dựng 9.850 `<span>` trong
> một dòng văn liên tục"*; lưới dựng **49.256 node** trong năm cột `subgrid`. `deferred-work.md:3258`
> đã ghi bằng chữ: ghi hai số đó cạnh nhau như một lượt cải thiện **là nói dối**. Mốc cũ được khai
> là **bản ghi lịch sử**, gạch ngang, không xoá.
>
> **Mốc so mới là số của 2.5b** *(`deferred-work.md:3252`, WKWebView thật)*:
>
> | Phép đo | 2.000 câu | 9.850 câu |
> | --- | --- | --- |
> | node DOM | 10.005 | 49.256 |
> | `selectionchange` + 2 frame | 12 / 34 / 34 ms | 24 / 33 / 33 ms |
> | **dời con trỏ** | 226 / 173 / 195 / 189 / 161 ms | **770 / 706 / 767 ms** |
>
> 📏 **Ngưỡng đòi mở ảo hoá sớm, tính lại trên đường DỜI CON TRỎ** *(không phải đường dựng)*:
>
> | Số mới ở 9.850 câu | Phán quyết hàng Deferred `:995` |
> | --- | --- |
> | ≤ 50 ms | 🟢 đạt NFR2 — giữ Giai đoạn 3, không mở |
> | 50 → 200 ms | ghi **điều kiện mở lại** kèm số; Ice quyết, dev **không** tự mở |
> | > 200 ms **hoặc** không giảm so với 706–770 ms | 🔴 **đòi mở sớm** — vào *"Cần Ice quyết"* với bảng Được/Mất |
>
> 🔴 Story này **không** mở hàng `:995` *(Giai đoạn 3)*; nó ghi số, ghi hệ quả, ghi điều kiện mở lại.

**Lý do:** phép so cũ không còn hợp lệ, và ngưỡng cũ *(1,4 s / 2,0 s)* được lấy từ trần **một lượt
dựng chạy một lần mỗi Chương** — mốc 1,4 s của Story 1.16. Đường dời con trỏ chạy **mỗi lần người
dùng bấm sang câu khác**, nên nó phải được cân theo trần NFR2 *(50 ms)*, không theo trần của một
thao tác chạy một lần.

### ⑶ 🟡 AC4 + `epics.md:2204` — tên panel chết

| Chỗ | CŨ | MỚI |
| --- | --- | --- |
| `epics.md:2204` | **Given** thư viện editor cho **Panel Editor** | **Given** thư viện editor cho **cột bản dịch của lưới** *(`panel.grid`)* 🔵 *(sửa 2026-08-18 theo lượt lật sang lưới; câu hỏi và AD-31 không đổi một chữ)* |
| Story 2.4 AC4 | *"thư viện editor cho Panel Editor"* | như trên |
| `ARCHITECTURE-SPINE.md:993` | *"Thư viện editor cho panel Editor"* | *"Thư viện editor cho **cột bản dịch của lưới**"* |

⚠️ **Nội dung câu hỏi không thu hẹp** — nó còn **nặng hơn**: bề mặt cũ đặt `contenteditable` lên
đúng một `<span>` tại một thời điểm; lưới đặt nó lên **mọi ô**. Doctrine *"DOM sở hữu văn bản bản
dịch, Vue không"* (`deferred-work.md:2261`) vì thế áp cho một bề mặt rộng hơn hẳn.

⇒ Cùng lớp với action item **B4** của retro Epic 2 *(đã đóng cho Epic 3 và Epic 6, sót Epic 2)*.

### ⑷ 🔴 AC17 — một vế chết, một vế sống

| Vế | CŨ | MỚI |
| --- | --- | --- |
| ① ba ảnh bàn đo 2.2 | *"chụp lại sau khi fixture thêm câu thứ sáu"* | 🔴 **BỎ.** Ba ảnh *(`ban-do-blink-light.png` · `ban-do-webkit-dark.png` · `ban-do-webkit-light.png`)* chụp một bề mặt đã bị `Supersedes` xoá 4/8 AC. Chụp lại là sản xuất bằng chứng cho một thứ không còn. Khai là **bản ghi lịch sử**, giữ nguyên tệp, ghi một dòng vào `deferred-work.md` |
| ② lời khai NFR15 sai | `2-2-ban-do-editor.html:11` — *"Dự án CỐ Ý không có bộ chạy test frontend"* | 🟢 **GIỮ.** Đã kiểm: dòng 11 còn nguyên lời khai sai đó. Hết đúng từ 2026-08-12. **Sửa**, đừng xoá trắng |

### ⑸ 🟡 AC14 · AC15 · AC3 — dữ kiện đã trôi

| AC | CŨ | MỚI | Đã kiểm |
| --- | --- | --- | --- |
| AC14 | grep `ARCHITECTURE-SPINE.md:883` phải trả 0; đích là `:894` | đích là **`:990`** | `editorFlush.ts:35`, `:62` còn trỏ `:883`; hàng Deferred thật ở `:990` |
| AC15 | `npm run test` ≥ **32/32** · `cargo test` ≥ **319/0** | ≥ **249/249** · ≥ **409/0/5** | đo 2026-08-18 |
| AC3 · §References | hàng Deferred `:894` · `:897` · `:899` | **`:990`** · **`:993`** · **`:995`** | đọc lại `ARCHITECTURE-SPINE.md` |

⚠️ Sàn AC15 cũ thấp hơn thật **~7,8×** ở vế vitest. Dùng nó để nghiệm thu là để một lượt mất **hàng
trăm** ca đi qua cổng mà không ai thấy.

### ⑹ 🔴 Câu hỏi giao cho Ice — cửa sổ đo của AC1 *(§3.1)*

Ba đường, và mỗi đường đổi **thứ Story 2.4 chịu trách nhiệm**:

| Đường | Được | Mất |
| --- | --- | --- |
| **(a) Giữ AC1 hẹp** *(chỉ trong lúc auto-save)* | trung thành từng chữ với NFR2; 2.4 đóng được sớm hơn | một vi phạm **~15× trần** sống tiếp mà **không AC nào bắt**; nó cần một story khác nhận, nếu không lại vào *"hộp khoá"* lần hai |
| **(b) Mở AC1 phủ cả đường dời con trỏ** | 2.4 bắt đúng cái đang hỏng; cửa chặn **B1** của retro đóng được đúng nghĩa | AC1 rộng hơn NFR2 nguyên văn; và nếu đường đó không vá nổi trong phạm vi hằng số, **AC5 kích hoạt** ⇒ Epic 2 dừng chờ Ice |
| **(c) Giữ AC1 hẹp, mở một story riêng cho đường dời con trỏ** | mỗi story một mệnh đề; 2.4 không phình | thêm một story vào Epic 2 đang dở; và nợ 2.5b/2.5c/2.11 phải chuyển chủ lần nữa |

**Đề xuất của dev: (b).** Lý lẽ đo được: cửa chặn **B1** mà Ice ký nguyên văn đòi *"ra một con số
NFR2 sau khi vá"*, và con số đang hỏng là **706–770 ms trên đường dời con trỏ** — không phải một
con số trong cửa sổ auto-save. Chọn (a) là đóng 2.4 bằng một con số **không phải** con số B1 hỏi.

> ✅ **ICE KÝ 2026-08-18 — đường (b).** AC1 mở rộng phủ **cả** đường dời con trỏ.
>
> **Hệ quả phải ghi ra, không nuốt:**
> 1. AC1 nay **rộng hơn NFR2 nguyên văn**. Đó là một lượt mở rộng **có chủ ý và có chữ ký**, không
>    một lượt đọc lỏng — báo cáo phải nói rõ hai cửa sổ đo và cửa nào nghiệm thu mệnh đề nào.
> 2. **Điều kiện kích hoạt AC5 rộng theo.** Nếu đường dời con trỏ không hạ được xuống 50 ms trong
>    phạm vi hằng số mà story được phép chạm, đó là ca *"một ngưỡng trượt một mình"* của AC5 ⇒
>    dừng, báo Ice theo khuôn Task 10, **và Epic 2 dừng theo**.
> 3. Ba mục `deferred-work.md` ghi *"Chủ: Story 2.4"* *(`:3579` · `:3680` · `:4707`)* nay trỏ được
>    vào một AC đang hoạt động — hết ca *"hộp khoá"* của retro Epic 2 §F6.

---

## 5. Đường đi được chọn

**Option 1 — Direct Adjustment.** Sửa AC trong story hiện có, không rollback, không đụng MVP.

| | |
| --- | --- |
| Effort | **Thấp** — 3 AC viết lại, 7 AC sửa dữ kiện, 1 dòng `epics.md`, 1 dòng SPINE |
| Risk | **Thấp** cho lượt sửa AC · **Trung bình** cho việc sau nó *(đường dời con trỏ có thể không vá nổi trong phạm vi hằng số ⇒ AC5)* |
| MVP | 🟢 **Không đổi.** NFR2 và NFR18 giữ nguyên từng chữ |
| Timeline | Không đổi thứ tự `2-4 → 2-12 → Epic 3` |

**Vì sao KHÔNG Option 2 (rollback):** không có gì để lùi — lượt lật sang lưới là một quyết định đã
ký và đã có bảy story xây lên trên.
**Vì sao KHÔNG Option 3 (MVP review):** chưa tới lúc. AC5 là cửa leo thang lên tầng PRD và nó chỉ
kích hoạt **sau** khi lưới sáu điểm và đường dời con trỏ đã có số. Gọi nó bây giờ là leo thang trước
khi đo — đúng cái mà chính AC5 cấm *(*"chưa hết lưới ⇒ chưa được báo Ice"*)*.

---

## 6. Bàn giao

| Hạng | Việc | Chủ |
| --- | --- | --- |
| **Ice ký** | ⑹ cửa sổ đo của AC1 — ba đường (a)/(b)/(c) | **Ice** |
| **Moderate** | Sửa AC12 · AC13 · AC17 và bảy AC 🟡 trong tệp story 2.4 | **Dev**, sau khi Ice ký ⑹ |
| **Moderate** | `epics.md:2204` tên panel chết, kèm 🔵 + ngày + lý do | **Dev** *(ngoại lệ B4 đã có tiền lệ)* |
| **Minor** | `ARCHITECTURE-SPINE.md:993` tên panel; `lint_spine.py` phải trả **0 findings** | **Dev** |
| **Minor** | `bench.js` đổi ba đầu dò DOM sang bề mặt lưới | **Dev**, sau khi AC12 được ký |
| **Chưa đụng** | Lưới sáu điểm NFR18 — `run-grid.sh` đã giao 2026-08-18 | **Ice** *(cần máy rảnh ~3,5 h)* |

### Tiêu chí xong

1. `grep -rn "nearestSentenceTo\|EditorPanel.vue" ` trong tệp story 2.4 trả **0** dòng còn dùng
   chúng như một đích đo *(chỗ nhắc lịch sử thì được, và phải gạch ngang)*.
2. `epics.md` và `ARCHITECTURE-SPINE.md` **0** chỗ còn gọi *"Panel Editor"* trong một mệnh đề đang
   hoạt động; `lint_spine.py` = **0 findings**.
3. AC13 mang bảng số của 2.5b làm mốc, và mốc cũ được gạch ngang chứ không xoá.
4. AC15 mang sàn **249** và **409**.
5. Ba mục `deferred-work.md` ghi *"Chủ: Story 2.4"* *(`:3579` · `:3680` · `:4707`)* trỏ được vào một
   AC **đang tồn tại** — hôm nay chúng trỏ vào AC12/AC13 bản chết.

---

## 7. Ghi chú phương pháp

🔵 **Lượt rà trọn-22-AC trả lại giá trị ở chỗ không ai đặt hàng.** Ba AC bị đánh 🔴 là thứ đã biết
trước khi mở workflow. Thứ **không** biết trước là §3.1 — AC1 không phủ đường đang vi phạm — và nó
chỉ lộ ra khi soát **từng** AC đối chiếu cây nguồn, thay vì soát đúng hai AC đã bị nghi.

⚠️ Cùng bài học với §9.2 của retro Epic 2: *"Bộ đo là một sản phẩm"*. Ở đây: **AC cũng là một sản
phẩm**, và một AC neo vào `tệp:dòng` sẽ mục đúng như một bản đồ chép tay mục — chỉ chậm hơn, và
không cổng nào đỏ.

---

## 8. Đã thi hành — 2026-08-18

Ice duyệt *"thi hành luôn"* cùng ngày. Toàn bộ nằm trong tạo tác quy hoạch và tạo tác mũi thăm dò;
🔴 **0 dòng mã sản phẩm bị chạm**.

| Tệp | Việc |
| --- | --- |
| `2-4-…-nfr18-va-nfr2-dong-thoi.md` | AC1 mở cửa sổ đo · AC12 · AC13 · AC17 viết lại · AC3/AC4/AC14/AC15 sửa dữ kiện · §mục 6 · §References · §Project Structure · bảng nợ đầu tệp · Task 3 |
| `epics.md:2204` | *"Panel Editor"* → *"cột bản dịch của lưới"* kèm 🔵 + ngày + lý do |
| `ARCHITECTURE-SPINE.md` | hàng Deferred *"Thư viện editor"* đổi tên kèm 🔵 · `lint_spine.py` = **0 findings** |
| `deferred-work.md` | ba mục *"Chủ: Story 2.4"* nay trỏ vào AC sống · ba ảnh 2.2 khai là bản ghi lịch sử · hai tên chết ngoài phạm vi giao **B5** |
| `2-4-ban-do/bench.js` | ba đầu dò DOM viết lại cho lưới *(`.grid` · `.col-tgt` · `.cell`)*; bản cũ hỏi `.doc`/`.sent` = **đo rỗng** |
| `sprint-status.yaml` | ghi kết quả và phán quyết ⑹ |

**Nghiệm thu:** `lint_spine.py` **0** · `npm run test` **249/249** · 7/7 cổng tĩnh chạy được **🟢** ·
`node --check bench.js` **hợp lệ** · `git status` **0** tệp mã sản phẩm.

### Ba thứ lượt thi hành tìm thêm, ngoài 22 AC

1. **Sàn `SELECTION_SURFACE_FLOOR` đã trôi** — story ghi **7** ở `check-commands.mjs:1908`; thật là
   **6** ở `:2025`. Cả sàn lẫn số dòng.
2. **Ba số dòng sổ nợ của AC12 đã trôi** — `:2167-2168` → **`:2225`** · `:2441` → **`:2511`** ·
   `:2449` → ~~`:2518`~~ *(mục `nearestSentenceTo`, nay mất hiệu lực)*.
3. **Bảng nợ đầu tệp story** vẫn liệt `nearestSentenceTo()` như một món **phải đóng** — đã gạch
   ngang và thay bằng đường dời con trỏ.

⚠️ Cả ba cùng một hạng với phát hiện chính: **một tham chiếu `tệp:dòng` là một tạo tác sẽ mục**, và
không cổng nào đỏ khi nó mục. Đây là ca thứ hai trong một ngày — xem §7.
