# Sprint Change Proposal — Bề mặt nhập lật sang lưới hai cột đối chiếu

**Ngày:** 2026-08-14 · **Chủ dự án:** Ice · **Chủ trì:** Correct Course
**Chế độ:** Incremental — bốn cụm đề xuất, Ice duyệt từng cụm
**Baseline:** `74f8825` *(master, đã push)* · cây làm việc **sạch** lúc bắt đầu soạn

> ✅ **ICE DUYỆT TRỌN GÓI 2026-08-14** — bốn cụm đề xuất, cộng ba quyết định kèm theo: **ba
> panel** (`panel.grid` · `panel.lookup` · `panel.ai_translation`) · Story 1.14 **AC
> superseded** · món nợ *"thư viện editor"* **chuyển chủ** từ Story 2.4 sang **2.5b**.
> **Đã thi công bước 1–5 của §5.2** *(sửa artifact; không viết mã)*.
>
> 🔵 **HAI THỨ PHÁT HIỆN TRONG LÚC THI CÔNG, ghi lại thay vì để trôi:**
>
> **① UX-DR32 cũng bị đụng, và nó KHÔNG có trong bảng tác động §2.3 ban đầu.** Nó khai cử chỉ
> gộp là *"gõ đè lên đúng vị trí ranh giới"* — tiền đề không còn tồn tại trong lưới. Đã sửa
> thành *"`Backspace` ở đầu ô bản dịch"*; **ngữ nghĩa AD-5 giữ nguyên một chữ**.
>
> **② Nghiệm thu §5.3 bắt được CHÍN chỗ sót** mà lượt soạn đề xuất không liệt kê: `epics.md`
> bảng FR→Epic *(FR16)*, tiêu đề + user story + AC preset của Story 1.14, hai AC của Story
> 4.12, ba chỗ chép lại *"vạch lề đã dùng hết **năm** giá trị"* *(nay **sáu**)*, và bốn chỗ
> trong `prd.md` *(bảng C2, bảng từ vựng, mô tả Q9)*. **Tất cả đã vá.** Đây đúng là thứ
> §5.3 sinh ra để bắt — một mệnh đề cũ sống sót qua một lượt lật hình dạng.
>
> ⚠️ **Giới hạn của phép nghiệm thu, ghi ra:** nó là `grep` **theo dòng**, nên một khối 🔵
> nằm ở dòng kế **không được nó nhận**. Bảy dòng còn lại sau lượt vá đã được **duyệt thủ
> công** — tất cả đều nằm trong khối đã đánh dấu, hoặc thuộc miền khác *("Trạng thái AI năm
> giá trị" — không liên quan tới bảng trạng thái segment)*.

> **Một câu tóm tắt:** hình dạng của bề mặt nhập đổi từ **một dòng văn liên tục** sang
> **lưới hai cột đối chiếu, mỗi câu một hàng**. Spine UX đã viết và đã ký trước lượt này;
> tài liệu đây **lan thay đổi đó xuống** `epics.md`, `prd.md`, tầng story, sổ trạng thái
> và sổ nợ.
>
> **Ba phát hiện của lượt rà làm phạm vi rộng hơn mô tả ban đầu**, và cả ba đều là chỗ
> spine vừa ký **nói lạc quan hơn cây nguồn**:
> **Ⓐ** *"không phá AD-37"* là **sai** — đó là xung đột trực diện, kéo theo FR121 ở Epic 8.
> **Ⓑ** *"FR20 thành thừa"* **thiếu một cặp panel** — panel Đề xuất AI vẫn cuộn độc lập.
> **Ⓒ** Lưới nuốt hai panel ⇒ **FR16 và danh tính `PanelId` bị đụng**, kéo theo Story 1.14.

---

## 1. Vấn đề

### 1.1 Trigger

Ice dùng thật và báo, nguyên văn ghi ở `.memlog.md`:

> *"nhập biên dịch vô cùng tệ hại… nếu không app gần như vứt bỏ, vô dụng."*

**Loại vấn đề:** *Failed approach requiring different solution*. Không có lỗi cài đặt nào ở
gốc — có **ba quyết định, mỗi cái hợp lý một mình**, chồng lên nhau thành một bề mặt sai.

### 1.2 Ba quyết định chồng nhau

| # | Quyết định | Hệ quả một mình | Hệ quả khi chồng |
|---|---|---|---|
| ① | Bảng trạng thái UX-DR19 khai **năm** giá trị, gom *"đã dịch tay, chưa xác nhận"* chung ô **không vạch** với *"chưa dịch"* | `resolveSegmentRule` cho cả hai ra `'none'`, và một test khoá mệnh đề đó lại | Người dịch **không phân biệt được** câu mình đã làm với câu chưa động tới |
| ② | Thu hẹp UX-DR20 *(Ice ký 2026-08-13, đo thật: mỗi câu rỗng đẩy chữ **9,05 px**, bố cục **nhảy trong lúc làm việc**)* | `.sent:empty::after{content:none}` ⇒ câu chưa dịch rộng **0 px** | Câu chưa dịch **vô hình** và **không bấm trúng được bằng chuột** |
| ③ | Ba khoá lỗi `err.segment.*` không có đường ra màn hình *(`main.ts` vứt `ConfirmResult`)* | Không component nào đọc `editorConfirmError` | Bấm xác nhận trên câu chưa dịch ⇒ **không một pixel nào đổi** |

⇒ Người dịch **không thấy** câu chưa dịch, **không bấm được** vào nó, và khi thao tác bị từ
chối thì **không được báo**. Ba lần im lặng liên tiếp.

### 1.3 Nguyên nhân gốc nằm ở HÌNH DẠNG, không ở ba mục trên

Một dòng văn liên tục là hình dạng để **ĐỌC** một bản dịch đã xong. Suốt thời gian dịch, bề
mặt ấy **gần như rỗng** — và một dòng văn liên tục không có cách nào diễn đạt *"ở đây sẽ có
một câu, nhưng chưa"* mà không hoặc **chiếm chỗ** *(bố cục nhảy — đã đo 9,05 px)* hoặc
**tàng hình** *(không bấm được)*. Hai vế đó đã được đo và chúng **xung khắc**.

Sửa lần lượt ba mục trên **không** giải được: vá ① và ③ vẫn để lại một câu chưa dịch rộng
0 px; gỡ vá ② thì bố cục nhảy trở lại.

### 1.4 Bằng chứng bổ sung Ice nêu trong cùng phiên

- **Khuyết tật *"sập hố"*:** xoá lui tới khi câu rỗng thì con trỏ **thấp xuống** và
  `Backspace` **chết**. Cơ chế: span rỗng 0 px co lại nên caret lấy chiều cao từ một hộp
  rỗng; và `contenteditable` chỉ đặt trên **đúng một** span nên `Backspace` ở offset 0
  không có chỗ nào để xoá lui vào — ngõ cụt.
- **Liên kết bấm-để-đối-chiếu** *(bấm câu gốc ⇒ câu dịch sáng lên)*: Ice hỏi, và lượt rà
  `epics.md` cho thấy **không FR nào** chứa nhu cầu đó. FR20 chỉ là Sync Scrolling; FR25 là
  điều hướng segment. Đây là **một khoảng trống thật** trong spec.
- **Câu hỏi chưa có lời đáp trong spine:** *"nếu cần BỎ QUA một đoạn, một câu trong bản dịch
  thì sao?"* — hôm nay *"cố ý bỏ trống"* và *"chưa dịch"* đều là `target_text` rỗng, không
  phân biệt được, và nó làm hỏng chính ý tưởng điều hướng *"câu chưa dịch kế tiếp"*.

---

## 2. Phân tích tác động

### 2.1 Ba phát hiện của lượt rà

#### Ⓐ Xung đột AD-37 — `.memlog.md` ghi *"khong pha AD-37"*, và điều đó **sai**

`ARCHITECTURE-SPINE.md:443` khai bằng chữ:

> **"Một cờ duy nhất dùng chung cho cả nguyên văn và bản dịch.** Đây là thứ làm cho lời hứa
> của FR121 đúng **theo định nghĩa** thay vì nhờ hai đường mã tình cờ đồng ý với nhau."

`epics.md:400` chép lại y hệt vào bảng bất biến. FR121 *(`epics.md:274`, Epic 8)* hứa
**"hai ô giữ đúng số lần xuống đoạn như nhau"**.

⇒ Cho phép bản dịch ngắt đoạn khác bản gốc **phá cả hai**, và phá **im lặng**: bản `.docx`
một khối vẫn xuất ra, vẫn là bảng hai cột hợp lệ, chỉ hai ô thôi không còn đối xứng — và
**không đường nghiệm thu nào của Epic 2 nhìn thấy điều đó**. Nó lộ ở Epic 8, cách sáu epic.

**Ice chốt:** một **AD-46 mới**; AD-37 giữ nguyên văn.

#### Ⓑ FR20 thừa **hai phần ba**, không phải hoàn toàn

`EXPERIENCE.md` viết *"FR20 thành thừa — không còn hai thứ để đồng bộ"*. FR20 khai **ba**
panel: `Source · AI Translation · Editor`. Lưới nuốt **hai**. Panel Đề xuất AI **vẫn là một
cột riêng** ở cả Ⓑ-1 lẫn Ⓑ-2, và Epic 4 cho gọi AI **theo lô** *(`epics.md:868`)* ⇒ panel
đó có nội dung cả Chương và **vẫn cuộn được lệch với lưới**.

**Ice chốt:** rút hẳn FR20, xoá Story 2.12 — sau khi đã đọc rủi ro. Món nợ có chủ **Epic 4**.

#### Ⓒ `PanelId` là union bốn giá trị chạy xuyên bảy chỗ

```
workspaceLayout.ts:32   PanelId = 'panel.source' | 'panel.lookup' | 'panel.ai_translation' | 'panel.editor'
        :41 PANEL_IDS   :54 PANEL_TITLE_KEYS   :62 PANEL_COMPONENTS
        :104 GRID_2X2   :124 LAYOUT_PRESETS
        :152 SACRIFICE_ORDER   :153 NEVER_SACRIFICED = ['panel.source','panel.editor']
focus.ts                xoay vòng focus theo tiền tố 'panel.'
```

`workspaceLayout.ts:153` mang nguyên văn mệnh đề *"Hai tập này rời nhau và **hợp lại đúng
bốn panel** (AC7)"* — **vỡ** khi còn ba. Kéo theo **FR16**, **FR19**, **FR21**, và
**Story 1.14** đã `done` với AC *"bốn slot panel… tồn tại trong một cửa sổ"* `(epics.md:1612)`.

**Ice chốt:** **ba panel** — `panel.grid` · `panel.lookup` · `panel.ai_translation`.

### 2.2 Hai tin làm NHẸ phạm vi *(đã kiểm, không phải suy luận)*

| Tin | Bằng chứng | Hệ quả |
|---|---|---|
| Hàng `draft` **không cần di trú lược đồ** | `schema.rs:398` — `status TEXT NOT NULL DEFAULT 'draft'`, hai giá trị `'draft' \| 'confirmed'`, cưỡng chế tầng Rust *(Story 2.5)* | Cần **một nhánh** trong `resolveSegmentRule` + một khối CSS `.rule-draft` |
| Hợp đồng vùng chọn **không phải sửa một dòng** | `selectionContract.ts` đăng ký theo **phần tử DOM** + vai `'source' \| 'display'`, duyệt bằng `el.contains(anchor)` — nó không biết panel nào tồn tại | Ràng buộc kèm theo: lưới đăng ký theo **CỘT**, không theo từng ô *(cổng đếm tĩnh ở `:112`)* |

### 2.3 Bảng tác động đầy đủ

| # | Artifact | Kiểu | Chi tiết |
|---|---|---|---|
| 1 | **AD-46** *(mới)* | ➕ | Cấu trúc đoạn của bản dịch là dữ liệu riêng |
| 2 | **AD-37** | ✅ Không đụng | Giữ nguyên văn — đây là điều kiện Ice ký |
| 3 | **FR121** `:274` | ✍️ | Đổi lời hứa, **không** đổi nghiệm thu |
| 4 | **FR16** `:96` | ✍️ | Bốn panel → **ba** |
| 5 | **FR19** `:102` | ✍️ | Chủ thể → *cột nguyên văn của lưới*; bỏ chữ *"tab"* |
| 6 | **FR21** `:106` | ✍️ | Hai chỗ đổi tên; khối 🔵 thu hẹp 2026-08-13 giữ nguyên văn |
| 7 | **FR20** `:104` + `prd.md:419` | ✂️ Rút | Kèm bia mộ + món nợ chủ Epic 4 |
| 8 | **FR133** *(mới)* | ➕ | Cắt bỏ câu khỏi bản dịch |
| 9 | **FR134** *(mới)* | ➕ | Bản dịch ngắt đoạn khác bản gốc |
| 10 | **UX-DR13** `:523` | ✍️ Viết lại | Lưới + Ⓑ-1/Ⓑ-2; preset *4 cột* rút |
| 11 | **UX-DR15** `:527` | ✍️ Viết lại mô tả | **Số và thứ tự hy sinh giữ nguyên** |
| 12 | **UX-DR19** `:537` | ✍️ Viết lại | Năm → **sáu** giá trị; bỏ *"không ô, không bảng"* |
| 13 | **UX-DR20** `:539` | ✂️ Rút | Hàng LÀ ranh giới |
| 13b | **UX-DR32** `:595` | ✍️ Sửa cử chỉ | **Thêm lúc thi công** — cử chỉ gộp *"gõ đè lên ranh giới"* → *"`Backspace` ở đầu ô"*; ngữ nghĩa AD-5 giữ nguyên |
| 13c | **UX-DR22** + 2 chỗ chép lại | ✍️ Sửa số | **Thêm lúc thi công** — *"vạch lề đã dùng hết **năm** giá trị"* → **sáu**; lý do *(tài nguyên đã tiêu hết)* không đổi |
| 14 | **Story 1.14** | 🔵 AC superseded | Story **không** mở lại |
| 15 | **Story 2.2** | 🔵 4/8 AC superseded | Bốn AC còn lại sống nguyên |
| 16 | **Story 2.5b** *(mới)* | ➕ | Lưới hai cột đối chiếu |
| 17 | **Story 2.5c** *(mới)* | ➕ | Cắt bỏ — bước di trú **8** |
| 18 | **Story 2.5d** *(mới)* | ➕ | Ngắt đoạn bản dịch — bước di trú **9** |
| 19 | **Story 2.9** | ✍️ Đổi tên + tiền đề | Ngữ nghĩa năm AC giữ nguyên |
| 20 | **Story 2.10** | ➕ Hai AC | Bỏ qua câu cắt bỏ; định nghĩa lại *"chưa dịch"* |
| 21 | **Story 2.12** | ⚰️ Xoá | Theo FR20 |
| 22 | **`sprint-status.yaml`** | ✍️ | Ba khoá mới, một đổi tên, một xoá |
| 23 | **`EXPERIENCE.md`** | ✍️ | Sửa **lý do** của câu về FR20, giữ kết luận |
| 24 | **`.memlog.md`** | ➕ | Nối mục `(correction)` — **không** sửa mục cũ |
| 25 | **`deferred-work.md`** | ➕ | Bốn món, mỗi món một chủ |
| 26 | **Cổng `check:commands` Kiểm I** | 🔧 | Đối chiếu hai chiều `SEGMENT_RULE_VALUES` ↔ `.rule-*` |

### 2.4 PRD — MVP **không** đổi phạm vi

Không FR nào bị cắt khỏi MVP. C2 vẫn giao đúng lời hứa: *"một vòng dịch tay hoàn chỉnh"*.
FR20 rời bảng, FR133 và FR134 vào — và cả hai đều nằm trong C2, không mở một năng lực mới
nào ngoài phạm vi đã ký.

---

## 3. Đường đi được chọn

### 3.1 Ba đường đã cân

| Đường | Phán quyết | Công | Rủi ro |
|---|---|---|---|
| **① Direct Adjustment** | ✅ **Chọn** | Cao | Trung bình |
| **② Rollback** | ❌ Không khả thi | Cao | Cao |
| **③ MVP Review** | ❌ N/A | — | — |

### 3.2 ② bị loại BẰNG SỐ, không bằng cảm tính

Phép đo 2026-08-14 *(đếm thô bằng `wc`/`grep` — không phải phân tích AST)*:

| Tầng | Dòng | Số phận |
|---|---|---|
| `segment.rs` + `config/segment.ts` + `editorPanelState.ts` | **2.103** | **Giữ nguyên** — flush AD-35, máy trạng thái, hợp đồng IPC không biết gì về hình dạng |
| `EditorPanel.vue` template + style | **361** | Viết lại |
| `editorGutter.ts` *(31 chỗ nhắc "làn")* | **273** | Gần như bỏ hẳn |

⇒ Rollback Story 2.2–2.5 vứt luôn **2.103 dòng vẫn đúng nguyên vẹn** để dựng lại đúng
chúng. Nặng hơn: Story 2.3 *(hợp đồng flush)* và 2.4 *(mũi thăm dò NFR18/NFR2)* mang **số
đo truy nguyên được**; rollback làm chúng hết so sánh được — đúng thứ
`project-context.md:337` cấm.

### 3.3 Vì sao ① đủ

Hình dạng cũ chỉ ràng buộc **tầng render**. Ba thứ đắt nhất của Epic 2 — hợp đồng flush
AD-35, máy trạng thái AD-31, và bề mặt IPC hai lớp — **không** biết văn bản được vẽ thành
dòng liền hay thành hàng. Đó là lý do một lượt lật hình dạng lớn tới mức này vẫn **không**
phải một lượt replan.

**Phân loại phạm vi: Moderate** — tổ chức lại backlog cộng một bất biến kiến trúc mới.
Không phải Minor *(có AD mới, có FR mới, có story mới)*; không phải Major *(mục tiêu sản
phẩm, MVP và kiến trúc nền không đổi)*.

---

## 4. Đề xuất sửa chi tiết

> Bốn cụm dưới đây Ice đã duyệt từng cụm trong phiên. Lời văn đầy đủ của từng mục nằm
> trong bản ghi phiên; mục này giữ **quyết định** và **chỗ sửa**, đủ để thi công.

### 4.1 Cụm #1 — `ARCHITECTURE-SPINE.md`: AD-46 *(duyệt)*

**Chèn sau AD-45.**

> ### AD-46 — Cấu trúc đoạn của bản dịch là dữ liệu riêng của bản dịch
>
> - **Binds:** C2, C8, C9
> - **Prevents:** AD-37 khai *"một cờ duy nhất dùng chung cho cả nguyên văn và bản dịch"*,
>   và mệnh đề đó **cấm** một đoạn Trung dài tách thành hai đoạn Việt — một quyền người dịch
>   có thật và dùng thường xuyên. Mở quyền đó bằng cách **nới AD-37** thì FR121 mất tiền đề
>   ở Epic 8, cách đây sáu epic, và mất **im lặng**.
> - **Rule:** `SEGMENT` mang **cờ kết đoạn thứ hai**, thuộc bản dịch. AD-37 **không sửa một
>   chữ** và tiếp tục sở hữu cờ của nguyên văn.
>
>   Cấu trúc đoạn của bản dịch được chở bởi **hai** thứ — **cờ đích** cho ranh giới **giữa**
>   segment, và **ký tự xuống dòng trong `target_text`** cho ranh giới **trong** một segment.
>   Đường xuất đọc **cả hai**; không đường nào suy ra từ nội dung nguồn.
>
>   Cờ đích **mặc định bằng cờ nguồn** lúc nhập. Ba ca biên của AD-37 *(gộp → câu cuối ·
>   tách → mảnh cuối, các mảnh trước tắt · segment cuối Chương → tắt, luôn luôn)* áp **y
>   nguyên** cho cờ thứ hai.
>
>   **FR121 đổi lời hứa, không đổi nghiệm thu.** Vế *"hai ô giữ đúng số lần xuống đoạn như
>   nhau"* thay bằng *"mỗi cột theo cờ của chính nó"*. Nghiệm thu thật — bôi đen **cột phải**
>   dán sang trình soạn thảo website ra văn bản liền mạch — **chỉ đọc cột phải**.
>
>   ⚠️ **Cái mất, ghi ra thay vì để người sau tự phát hiện:** đối xứng thị giác của bản
>   `.docx` một khối. Hai cột lệch số đoạn thì càng xuống càng lệch xa, và **không có gì
>   sai** — đó là bản dịch đúng ý người dịch. Đường đối chiếu thật là lưới của Workspace,
>   không phải file xuất.

**Kèm theo:** FR121 `:274` đổi lời hứa · `epics.md:400` thêm dòng AD-46 · ghi chú Epic 8
`SPINE §955` viết lại *(lời hứa "đúng theo định nghĩa" đã rút)*.

### 4.2 Cụm #2 — `epics.md` tầng yêu cầu *(duyệt)*

| Mục | Sửa |
|---|---|
| FR16 | Bốn panel → **ba**: *Lưới đối chiếu · Lookup · AI Translation*, kèm khối 🔵 |
| FR19 | *Panel Source* → **cột nguyên văn của lưới**; bỏ *"tab"*; thêm *"người dùng tự bật tắt"* + lời từ chối hai mặc định thông minh |
| FR21 | *Panel Source* → **cột nguyên văn**; *Panel Editor* → **cột bản dịch**; khối 🔵 2026-08-13 giữ nguyên văn |
| FR20 | Rút, kèm bia mộ nêu rõ cặp `lưới ↔ AI Translation` là thứ bị bỏ và nợ có chủ Epic 4 |
| FR133 | Mới — cắt bỏ, trục độc lập kiểu `translate="no"` của XLIFF 2.0 |
| FR134 | Mới — ngắt đoạn bản dịch, trỏ AD-46 |
| UX-DR13 | Viết lại: hàng · ô trống có chiều cao thật · khoảng thở từ `is_paragraph_end` · Ⓑ-1/Ⓑ-2; **Review Mode (FR92) không đụng** |
| UX-DR15 | **Bốn ngưỡng và thứ tự hy sinh GIỮ NGUYÊN** *(số → Story 4.12; thứ tự → quyết định)*; chỉ đổi cách diễn đạt sang panel thay vì vị trí |
| UX-DR19 | Viết lại: **sáu** giá trị, đọc ở cột trạng thái; rút *"không ô, không bảng"* và ngôn ngữ *"hai vạch, bốn vạch"* |
| UX-DR20 | Rút — phép đo 9,05 px **không mất giá trị**, nó là bằng chứng vì sao hình dạng cũ không thể thoả cả hai vế |

### 4.3 Cụm #3 — `epics.md` tầng story *(duyệt)*

**Superseded:** Story 1.14 *(AC bốn slot)* · Story 2.2 *(4/8 AC)*. Cả hai **giữ nguyên văn
AC cũ**, thêm khối 🔵 trỏ sang 2.5b và nói rõ vì sao lật. Không story nào mở lại.

**Story 2.5b — Lưới hai cột đối chiếu.** Covers UX-DR13 · UX-DR15 · UX-DR19 · FR16 · FR19 ·
FR21 · AD-1 · AD-34. **14 AC**, trong đó bốn cái đáng gọi tên:

- AC7 — hợp đồng vùng chọn đăng ký theo **CỘT**; `selectionContract.ts` **không sửa một dòng**
- AC9 — `NEVER_SACRIFICED` còn **đúng một** phần tử; hai tập hợp lại đúng **ba** panel
- AC11 — 🔴 `Enter` **trơn không bao giờ xác nhận**. Bằng chứng: OmegaT có tuỳ chọn *"Use TAB
  to Advance"* đặt ra **chính vì** `Enter` va chạm IME; người dùng sản phẩm này gõ tiếng Việt
  bằng bộ gõ. **Không đường nghiệm thu nào của dự án bắt được lớp lỗi này** — nó chỉ lộ ở tay
  người dùng
- AC14 — ba khoá lỗi `err.segment.*` **có đường ra màn hình**

**Story 2.5c — Cắt bỏ.** Covers FR133 · bước di trú **8**. 7 AC.
**Story 2.5d — Ngắt đoạn bản dịch.** Covers FR134 · AD-46 · bước di trú **9**. 6 AC, gồm gỡ
hai lớp chặn `Enter` *(`EditorPanel.vue:769` + `:842`)* **chỉ ở ô bản dịch**.

**Story 2.9** đổi tên → *"Gộp bằng `Backspace` ở đầu ô"*; năm AC giữ **nguyên ngữ nghĩa**,
chỉ đổi cử chỉ kích hoạt; Covers FR78 không đổi.
**Story 2.10** thêm hai AC. **Story 2.12** xoá.

### 4.4 Cụm #4 — sổ trạng thái, PRD, spine UX, sổ nợ *(duyệt)*

`sprint-status.yaml`: thêm `2-5b-luoi-hai-cot-doi-chieu` · `2-5c-cat-bo-cau-khoi-ban-dich` ·
`2-5d-ngat-doan-ban-dich` *(đều `backlog`)*; đổi khoá 2.9; xoá 2.12. Comment viết **không
dấu** theo quy ước tại chỗ. **Không** sửa dòng *"So ke tiep la 8"* — chính tệp đó dặn nguồn
sự thật là `PROJECT_MIGRATIONS`.

`prd.md`: rút FR20 `:419`; thêm FR133/FR134; đối chiếu FR16/FR19/FR21 lúc thi công.

`EXPERIENCE.md`: sửa **lý do** *(không phải kết luận)* của câu về FR20 — bản đầu bỏ quên
panel Đề xuất AI. `.memlog.md`: **nối** một mục `(correction)`, không sửa mục cũ.

`deferred-work.md` — bốn món:

| # | Món | Chủ |
|---|---|---|
| 1 | FR20 rút; nếu Epic 4 dựng dịch **theo lô** thì nhu cầu cuộn cùng nhau quay lại và không FR nào chứa nó | **Epic 4** |
| 2 | Chiều cao hàng khi bật Hán Việt *song song* ở cột hẹp Ⓑ-2 — **ước lượng hình học, CHƯA ĐO** | **Story 2.5b** |
| 3 | `editorGutter.ts` + `editorGutterLanes.test.ts` mất lý do tồn tại; **không xoá im lặng** | **Story 2.5b** |
| 4 | Ngưỡng bố cục nay phải hiệu chỉnh cho **hai** bố cục, không phải một | **Story 4.12** |

⚠️ **Một món nêu mà không tự xếp:** hàng Deferred *"thư viện editor"* có chủ **Story 2.4**.
Lưới đổi bài toán đó — `contenteditable` trên **một ô mỗi hàng** khác hẳn trên **một dòng văn
liên tục**, và khuyết tật *"sập hố"* đến thẳng từ hình dạng cũ. Nên **đọc lại** ở 2.5b thay
vì mặc nhiên giữ kết luận cũ. Đổi chủ là quyết định của Ice.

---

## 5. Bàn giao

### 5.1 Phân loại: **Moderate**

Tổ chức lại backlog + một bất biến kiến trúc mới. Cần phối hợp Product Owner ↔ Developer;
**không** cần replan với Product Manager / Architect.

### 5.2 Thứ tự thi công

| Bước | Việc | Vì sao đứng đây |
|---|---|---|
| 1 | AD-46 vào `ARCHITECTURE-SPINE.md` | FR134 và Story 2.5d **treo vào nó** |
| 2 | `epics.md` tầng yêu cầu *(cụm #2)* | Story mới trỏ vào FR/UX-DR đã sửa |
| 3 | `epics.md` tầng story *(cụm #3)* | — |
| 4 | `prd.md` · `EXPERIENCE.md` · `.memlog.md` · `deferred-work.md` | — |
| 5 | `sprint-status.yaml` | Sổ trạng thái đi **sau cùng** — nó phản ánh, không dẫn dắt |
| 6 | **Story 2.5b** dựng lưới | 2.9 và 2.10 dựa vào hình dạng |
| 7 | **2.5c** → **2.5d** | Bước di trú 8 rồi 9, đúng thứ tự |
| 8 | 2.6 → 2.11 theo thứ tự cũ | — |

### 5.3 Nghiệm thu của chính lượt correct-course này

- `epics.md` **không còn** một chỗ nào nói *"bốn panel"*, *"lưới 2×2"*, *"năm giá trị"*,
  *"không ô, không bảng"* mà **không** kèm dấu superseded hoặc khối 🔵.
- `grep -n "FR20" epics.md prd.md` chỉ ra **dòng bia mộ**, không ra một yêu cầu còn sống.
- `AD-37` **không đổi một ký tự**.
- Mỗi món trong `deferred-work.md` **có một chủ**; không mục nào mồ côi.

### 5.4 Việc KHÔNG thuộc lượt này

Viết mã. Lượt correct-course chỉ sửa artifact quy hoạch. Story 2.5b/2.5c/2.5d đi qua
`bmad-create-story` rồi `bmad-dev-story` như mọi story khác.

---

## 6. Ghi chú về baseline

Commit `74f8825` *(đã push lên `origin/master`)* gom **cả hai lớp làm một**: Story 2.5 **và**
bốn tệp spine UX của phiên bàn tròn hôm nay *(`DESIGN.md`, `EXPERIENCE.md`, `.memlog.md`,
`.working/editor-grid-two-column.html`)*. Diff của Story 2.5 vì thế **không đọc được một
mình**, ngược với luật ở `project-context.md:426`. Commit đã ở trên remote ⇒ tách bằng
rewrite **không đáng giá**; ghi lại ở đây thay vì để nó trôi. *(Message cũng đi tiếng Anh
thay vì khuôn `type(scope): câu tiếng Việt`.)*

**Từ đây trở đi lượt correct-course là một commit sạch, riêng.**
