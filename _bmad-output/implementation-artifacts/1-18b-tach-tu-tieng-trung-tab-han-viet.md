---
baseline_commit: 8c54441
---

# Story 1.18b: Tách từ tiếng Trung cho tab Hán Việt — double-click chọn CỤM TỪ

Status: done

> 🔴 **STORY NÀY SINH RA TỪ MỘT LƯỢT NGHIỆM THU TAY, KHÔNG TỪ `epics.md`.** Không FR nào
> đặc tả nó *(đã rà: FR40 là hình thái **tiếng Anh** · FR51/FR61 nói khớp thuật ngữ, và FR61
> còn cố ý **không** tách từ · FR64 là TMX · FR113 là đề xuất Glossary)*. Nó là một khoản nợ
> Ice bắt bằng mắt 2026-08-07 và chốt thành story riêng. ⇒ **Toàn bộ AC dưới đây phải tự
> đứng được**, không có mệnh đề epic nào đỡ lưng.
>
> 🔴 **PHÂN TÍCH ĐẦU TIÊN CỦA DEV SAI, VÀ ICE LẬT NÓ — đừng đi lại vào hố đó.** Dev kết luận
> *"phải tự xây bộ tách từ, cần `jieba-rs` ở Rust, một story lớn"*. **Sai.** Bộ tách từ **đã
> có sẵn trong engine** — nó chính là thứ làm double-click hoạt động đúng ở tab nguyên văn
> tiếng Trung *(Ice: "ở phần văn bản gốc thì double click sẽ chọn được cả cụm từ, vậy tại sao
> khi chuyển đổi sang phần hán việt lại không chọn được")*. Truy được qua **`Intl.Segmenter`**
> ⇒ **0 phụ thuộc mới · 0 dòng Rust**.
>
> 🔴 **BA MỆNH ĐỀ SỐNG CỦA HAI STORY TRƯỚC ĐI QUA ĐÚNG BỀ MẶT NÀY**, và story này phá cấu
> trúc mà cả ba đứng lên: **AC6 của 1.16** *(vùng chọn ở kiểu song song = đúng ký tự nguồn)* ·
> **AC12 của 1.18** *(truy vấn Auto-Lookup không lẫn âm)* · **AC11 của 1.18** *(bôi đen bằng
> bàn phím)*. ⇒ **ĐO LẠI CẢ BA**, không suy từ số đo cũ. Xem AC5·AC6·AC7.
>
> ⚠️ **Cây làm việc không SẠCH lúc tạo story** — 4 tệp chưa commit *(lượt vá `<ruby>` +
> cỡ chữ/nghiêng, cùng ngày)*. Đọc §Bối cảnh git trước khi gõ dòng đầu tiên.

## Change Log

| Ngày | Thay đổi |
|---|---|
| 2026-08-07 | **Task 0 — bốn quyết định đã chốt, KÈM LÝ DO.** **#1 (a)** — `Intl.Segmenter` ở webview **không** vi phạm AD-1: `ARCHITECTURE-SPINE.md:75-79` liệt kê **"vùng chọn"** là thứ frontend được giữ, và `EXPERIENCE.md:23` chỉ khoá ba thứ ở Rust *(tách **câu**, khớp ngôn ngữ, phân giải scope)* — tách **từ để chọn vùng** không nằm trong đó. Mệnh đề này **đã viết vào mã** *(doc-comment đầu `wordBoundary.ts`, kèm cả hai trích dẫn + `review-ad-44-2026-08-05.md:50`)*, đúng như Quyết định #1 đòi, vì nó là một **suy luận** chứ không một câu chốt tường minh. **#2 (a)** — `Segment.han` mang **một TỪ** *(`chars[]` + `readings[]`, cùng độ dài)*: đây là hình dạng duy nhất giữ được cả ba mệnh đề *(một `<ruby>` mỗi từ · âm vẫn tra theo **ký tự** qua `hanVietByChar` không sửa · ánh xạ ngược cắt được đúng biên)*. Hệ quả bắt buộc **đã cài**: cả hai đường phân giải nay CẮT theo range. **#3 (a)** — `resolveSelection` **đọc DOM trực tiếp**, cùng khuôn `resolveParallel`; ba bảng phẳng `text`/`map`/`starts` bị **xoá hẳn**. **#4 (a)** — ký tự thiếu âm giữ `READING_PLACEHOLDER` ở **đúng âm tiết đó**. |
| 2026-08-07 | **Task 0 — cây làm việc: Ice chốt COMMIT RIÊNG.** Bốn tệp vá `<ruby>`/token 2026-08-07 đi thành commit `51132cb` *(«fix: âm Hán Việt đi bằng `<ruby>`, gỡ hai token vá triệu chứng»)* **trước** dòng mã đầu tiên của story. Lý do Ice chọn: diff của 1.18b đọc được một mình, và `git revert` lật được story mà **không** lật lượt vá. ⚠️ `baseline_commit` trong frontmatter **giữ nguyên `8c54441`** theo luật workflow *(không ghi đè giá trị đã có)*; baseline **thật** mà story đứng lên là **`51132cb`**. |
| 2026-08-07 | **LỆCH CÓ CHỦ Ý so với chữ của Task 3 — `<rt>` ở kiểu song song nối âm bằng DẤU CÁCH, không `U+2060`.** Giữ nguyên **ý** của Task 3, bác **chữ** của nó, vì ba lý do: ① `U+2060` tồn tại để một cụm âm **là một từ với ICU**, tức để double-click trúng nó — mà ở kiểu song song thứ người dùng double-click là **chữ Hán** (base của `<ruby>`), không phải `<rt>`; ② `<rt>` là **một** text node nên CSS **không** vẽ được khe giữa hai âm trong nó ⇒ `U+2060` cho ra `thailoan` dính liền, đúng thứ Ice bác nguyên văn ở 1.16 *(`phảnđốitrungcộngkhoác…`)*; ③ `user-select: none` **không** ràng buộc `Selection.modify()` trên WebKit *(số đo AC12/1.18)* nên `<rt>` rò được vào `toString()` — rò một **dấu cách** còn lần ra được, rò một ký tự **vô hình** thì không. |
| 2026-08-07 | 🔴 **LỆCH THỨ HAI, do SỐ ĐO ép: `.hv-unit` chuyển `display: inline-block` → `display: inline`.** Bản đầu giữ `inline-block` *(và doc-comment còn khen nó "chặn ICU với qua biên ô")*. Lượt đo AC7 lật điều đó: `Selection.modify('extend','right','word')` **KẸT HẲN** ở ô đầu tiên với `inline-block` *(1→12 lần bấm đều cho `台湾`)*, trong khi cấu trúc **CŨ** đi được tới `台湾地方议会` — tức một **hồi quy thẳng** của AC11/1.18, đúng thứ AC7 cấm. Với `inline`: 3 lần bấm ⇒ `台湾地方`, 6 ⇒ `台湾地方议会`, **nhanh hơn cả cấu trúc cũ** *(một lần bấm = một TỪ, không một ký tự)*. Và không đánh đổi gì: double-click **26/26** vẫn khớp *(ranh giới từ do **ICU** quyết, không do biên node)*, `toString()` cả đoạn trả đúng chuỗi nguồn — **0** `\n` chèn thêm, **0** ký tự Latin rò từ `<rt>`. |
| 2026-08-07 | 🔴 **MỘT LỖI THẬT DO BÀN ĐO BẮT, KHÔNG DO SUY LUẬN: `intersectsNode` một mình thừa MỘT ký tự.** Chọn `đài` rồi thả đúng ở **đầu** `loan` trả về `台湾` thay vì `台` — vỡ AC6 *("không thừa một ký tự nào")*. Nguyên nhân: `selectNode(el)` đặt ranh giới **NGOÀI** phần tử, nên một điểm cuối nằm **TRONG** nó ở offset 0 vẫn đứng *sau* ranh giới đó dù phủ **không một ký tự nào**; `selectNodeContents()` **không** sửa được *(`(span,0)` vẫn đứng trước `(textNode,0)`)*. ⇒ [`overlapsRange`] đo ranh giới ở **mức TEXT NODE** khi node có đúng một text node con. Đo lại sau khi sửa: `台` ✅. |
| 2026-08-07 | **AC9 — trần render GIỮ NGUYÊN 50.000, theo số, không theo cảm tính.** Đo 2026-08-07 *(Chromium, cùng một bàn đo cho cả hai cấu trúc)*: số node **0,642** trên mỗi lần xuất hiện ký tự Hán *(3.502/5.000 · 34.714/50.000 — cùng tỉ lệ ở hai mức)*, chi phí dựng DOM **giảm ~31 %** *(10,7 ms vs 15,2 ms ở 5.000 · 92,6 ms vs 133,7 ms ở 50.000)*, cộng **18,7 ms** một lượt tách từ ở mức 50.000 — chạy **một lần** mỗi Chương. ⚠️ **KHÔNG nâng trần**, vì bàn đo này **khác** bàn đo Playwright đã đặt ra con số 1.408,5 ms của 1.16 ⇒ chỉ **tỉ lệ** so được, **giá trị tuyệt đối thì không**. Đúng mệnh đề AC9: *"nếu không đo được thì ghi ra, giữ nguyên 50.000, không nâng"*. |
| 2026-08-07 | Tạo story. Baseline `8c54441` **+ 4 tệp chưa commit**. Baseline đã ĐO, không chép: `cargo test` **232 xanh** · **7 cổng `check:*` xanh** · `npm run build` xanh. Nguồn số đo: `deferred-work.md` §*Deferred from: nghiệm thu tay tab Hán Việt (Ice, 2026-08-07)* — **bảng đo đã có sẵn, không đo lại từ đầu**. Phân tích: `ARCHITECTURE-SPINE.md` *(AD-1 · AD-4 · AD-15 · AD-16 · AD-17 · AD-25 · AD-34 · AD-44)* · `epics.md` *(§Story 1.12 · 3.4 · 7.7 · UX-DR5/6/8/17)* · `prd.md` *(FR40/51/61/64/113 · NFR1/13/14/15/16/17)* · `EXPERIENCE.md:131` *(FR21 — "không được thiết kế lại cho khác đi")* · `DESIGN.md` *(§Bảng token · §Motion)* · mã thật `src/panels/**`. **Phát hiện quyết định phạm vi:** ① **AD-1 KHÔNG cấm** — nó liệt kê **"vùng chọn"** là thứ frontend được sở hữu *(nguyên văn `ARCHITECTURE-SPINE.md:75-79`)*; ② `review-ad-44-2026-08-05.md:50` đã phân xử sẵn đúng ranh giới này bằng chữ: *"hàng **cụm từ nhiều chữ** là câu hỏi của **Auto-Lookup (chọn gì để tra)**, không phải của **đường tra cứu (tra thế nào)**"* ⇒ story này **không** là một Matcher thứ hai *(AD-17/Story 1.12 giữ nguyên độc quyền khớp ngôn ngữ ở Rust)*; ③ **AD-4 không áp** — nó khoá ranh giới **SEGMENT (câu)**, story này tách **TỪ**, không lưu xuống đĩa, không mang `id`. |

**Epic:** 1 — Nền móng ứng dụng & Tra cứu ngoại tuyến tức thì
**Story key:** `1-18b-tach-tu-tieng-trung-tab-han-viet`
**Vì sao mang số `18b`:** cùng tiền lệ `1-10b`/`1-10c` — nó thuộc **họ bề mặt Hán Việt** *(1.16 dựng, 1.18 chạm)* và phải chạy **trước** mọi story sau chạm cùng bề mặt *(3.4 Glossary highlight)*, nhưng không phải một năng lực mới của epic.
**Nhận nợ từ:** `deferred-work.md` §*nghiệm thu tay tab Hán Việt (Ice, 2026-08-07)* — **mục duy nhất**, đã mang trọn bảng đo.
**Governed by:** **AD-1** *(🔴 **đọc kỹ**: "vùng chọn" NẰM TRONG danh sách frontend được giữ — xem Quyết định #1)* · **AD-17** *(Matcher là **đúng một** cài đặt khớp ngôn ngữ ở Rust — story này **không đụng**, xem §KHÔNG-LÀM ①)* · **AD-34** *(mọi thao tác qua `CommandRegistry`; handler chuột chỉ `dispatch`)* · **AD-16** *(nội dung nhập từ ngoài **không bao giờ** render thành HTML — không `v-html`)* · **AD-4** *(ranh giới **segment** tính một lần lúc nhập — story này tách **TỪ**, khác đơn vị, xem Quyết định #2)*
**UX phải tôn trọng:** **UX-DR5** *(`ornament`/`tm-rule` là màu của **nét**, không bao giờ của chữ)* · **UX-DR6** *(không `opacity` làm mờ chữ ở trạng thái nghỉ)* · **UX-DR8/UX-DR17** *(hợp đồng tiêu điểm — story này **giữ nguyên** `tabindex="0"` mà 1.18 đặt, không thêm điểm dừng `Tab` mới)* · `DESIGN.md` §Giãn dòng *(sàn **1.66** cho họ `read`)*
**Ràng buộc xuôi dòng phải để lại chỗ đứng:** **Story 3.4** *(đánh dấu thuật ngữ Glossary **trong Panel Source** — cùng bề mặt, và nó sẽ cần biết ranh giới TỪ)* · **Story 1.20** *(lịch sử tra cứu — đường tra vẫn phải đúng MỘT điểm nghẽn)* · **Story 7.7** *(Concordance)* · **Story 1.21** *(gán lại phím)*
**NFR:** **NFR1** *(p95 < 100 ms đầu-cuối — Auto-Lookup chạm bề mặt này hàng trăm lần mỗi Chương)* · **NFR13** *(ngoại tuyến — `Intl.Segmenter` chạy cục bộ, **0** điểm ra mạng)* · **NFR14** *(hai nền tảng — 🔴 `Intl.Segmenter` trên **WKWebView** CHƯA đo)* · **NFR15** *(**0** phụ thuộc mới)* · NFR16 *(chuỗi ở `vi.json`)* · **NFR17** *(bàn phím — đường `Selection.modify()` của AC11/1.18 **không được hồi quy**)*
**Ngày tạo:** 2026-08-07

---

## 🔴 ĐỌC TRƯỚC TIÊN — BỐN VIỆC STORY NÀY KHÔNG LÀM

### ① KHÔNG dựng một `Matcher` thứ hai — AD-17 giữ nguyên độc quyền

`AD-17` *(`ARCHITECTURE-SPINE.md:230-236`)* nói **đúng một** cài đặt khớp ngôn ngữ, dùng chung
cho FR40/FR51/FR61, tách từ qua **`jieba-rs` ở Rust**. Story này **không chạm một dòng nào**
của `core/matching/**`, và **không** cấp `Intl.Segmenter` cho bất kỳ đường **khớp/tra cứu**
nào.

🔴 **Ranh giới đã được phân xử SẴN, bằng chữ** — `reviews/review-ad-44-2026-08-05.md:50`:

> *"hàng **cụm từ nhiều chữ** là câu hỏi của **Auto-Lookup (chọn gì để tra)**, không phải của
> **đường tra cứu (tra thế nào)**"*

⇒ `Intl.Segmenter` ở đây trả lời **"chọn gì"** *(ranh giới vùng chọn trên màn hình)*.
`jieba-rs`/`Matcher` trả lời **"tra thế nào"** *(khớp dữ liệu)*. Hai câu hỏi, hai tầng, hai
cổng. **Ghi mệnh đề này vào doc-comment tại chỗ** — người rà soát sau sẽ hỏi đúng câu đó.

⚠️ Và nhớ lý do `mockups/tm-fuzzy-match.html:267-269` **từ chối** tách từ cho TM: *"tách từ
tiếng Trung… sai ở một tỷ lệ nhất định, và mỗi lần sai sẽ làm điểm khớp lệch theo cách không
giải thích được"*. Lý lẽ đó **đúng cho khớp** và **không áp cho chọn vùng**: một lượt tách sai
ở đây chỉ khiến người dùng kéo chọn lại, không làm hỏng một điểm số nào.

### ② KHÔNG đụng tầng dữ liệu tra cứu, không đụng `runLookup`

`core/dict/**` · `commands/dict.rs` · `src/config/dict.ts` · `lookupPanelState.ts::runLookup` /
`resetLookupPanel` — **giữ nguyên chữ ký, giữ nguyên bộ đếm `sequence`**. Story này chỉ đổi
**chuỗi truy vấn đi vào** đường đó, không đổi đường.

### ③ KHÔNG thêm một điểm dừng `Tab` nào, không đụng hợp đồng tiêu điểm

Story 1.18 đã trả giá một lần cho `tabindex="0"` trên `.hv-switch`/`.hv-parallel` *(Ice chốt
chấp nhận 2026-08-07)*. Cấu trúc mới **giữ nguyên đúng hai `tabindex` đó** — mọi `<span>`/
`<ruby>` mới sinh ra **không** được mang `tabindex`.

### ④ KHÔNG dựng bảng tra âm→chữ, không đoán nghĩa

Cùng lý do Story 1.18 §Quyết định #2 đã bác: ánh xạ âm→chữ là **đa trị** *(`"lương"` → 良·涼·
糧·量·粱)* và giải nó cần ngữ cảnh — một **quy tắc nghiệp vụ mới ở webview**, đúng thứ AD-1
cấm, và là phần việc của **FR113/Story 3.7**. Ánh xạ ngược ở đây đi bằng **VỊ TRÍ**, y hệt
bản đang chạy.

---

## Story

As a người dịch,
I want double-click ở tab Hán Việt chọn được **cả cụm từ** đúng như ở tab nguyên văn,
So that tôi tra một từ ghép bằng **hai cú bấm** thay vì phải kéo chọn thủ công từng lần.

---

## Ranh giới phạm vi — ĐỌC TRƯỚC KHI GÕ DÒNG ĐẦU TIÊN

| Trong phạm vi | Ngoài phạm vi (và ai sở hữu) |
|---|---|
| Gom ký tự Hán liên tiếp thành **TỪ** bằng `Intl.Segmenter('zh')` | Tách từ cho **khớp/tra cứu** *(AD-17 · Story 1.12 · `jieba-rs` ở Rust)* |
| Kiểu **song song**: một `<ruby>` mỗi **TỪ** | Đổi cơ chế `<ruby>`/`ruby-position` *(vừa chốt 2026-08-07)* |
| Kiểu **chuyển đổi**: âm trong cùng từ nối bằng `U+2060`, khe hở vẽ bằng **CSS** | Đổi token `source-hanviet` *(vừa chốt: 14,5px, không nghiêng)* |
| Viết lại `buildSegments` · `resolveParallel` · `resolveSelection` | Sửa `runLookup`/`resetLookupPanel`/`config/dict.ts` *(§KHÔNG-LÀM ②)* |
| **ĐO LẠI** AC6 *(1.16)* · AC11 · AC12 *(1.18)* trên cấu trúc mới | Tin số đo cũ *(cấm tường minh — xem AC5·AC6·AC7)* |
| Đo lại `PARALLEL_VIEW_RENDER_CEILING` nếu số node đổi | Nới trần mà không đo |
| **0 phụ thuộc mới** *(NFR15)* | Một thư viện tách từ ở npm |
| | **Glossary highlight** *(3.4)* · **lịch sử** *(1.20)* · **Concordance** *(7.7)* |
| | Nội dung Editor/AI Translation *(Epic 2/4)* |

---

## 🔴 BỐN QUYẾT ĐỊNH — CHỐT Ở TASK 0, TRƯỚC DÒNG MÃ ĐẦU TIÊN

> Mỗi quyết định có **mặc định đề xuất kèm lý do**. Chốt theo mặc định ⇒ một dòng Change Log.
> Chốt ngược ⇒ ghi **lý do**, không chỉ ghi lựa chọn.

### 🔴 Quyết định #1 — `Intl.Segmenter` ở webview có vi phạm AD-1 không? *(CHẶN THẬT)*

**(a) — MẶC ĐỊNH ĐỀ XUẤT: KHÔNG vi phạm, và lý do phải ghi vào mã.**

`ARCHITECTURE-SPINE.md:75-79` nguyên văn:

> *"frontend chỉ render và giữ state UI (**focus, cuộn, vùng chọn, bố cục panel**). Không cài
> đặt lại bất kỳ quy tắc nghiệp vụ nào ở TypeScript."*

**"Vùng chọn" nằm trong danh sách được phép, viết thẳng.** Story này dùng `Intl.Segmenter` để
quyết định **ranh giới của một vùng chọn** — không để khớp, không để dịch, không để chấm điểm.
Củng cố bằng `EXPERIENCE.md:23`: ba thứ phải ở Rust là *"tách **câu**, khớp ngôn ngữ, phân giải
scope"* — tách **từ để chọn vùng** không nằm trong ba thứ đó.

⚠️ **Nhưng đây là một SUY LUẬN, không một câu chốt tường minh trong tài liệu.** ⇒ story phải
**viết ra mệnh đề này tại chỗ** *(doc-comment đầu module tách từ)*, kèm cả hai trích dẫn, để
người rà soát sau không đọc nhầm thành "một Matcher thứ hai ở webview".

**(b) — đưa tách từ về Rust qua một `#[tauri::command]` mới.** **Bác.** Nó thêm một vòng IPC
vào **đúng đường nóng NFR1** cho một thứ engine làm sẵn miễn phí và đồng bộ; nó tạo một cài
đặt tách từ **thứ hai** cạnh `jieba-rs` *(đúng thứ AD-17 cấm)*; và nó vẫn không trả lời được
câu hỏi thật là *"trình duyệt sẽ chọn tới đâu khi người dùng double-click"* — thứ chỉ engine
biết.

### 🔴 Quyết định #2 — Hình dạng `Segment` mới *(CHẶN — mọi task sau đứng trên nó)*

Hôm nay: `{ kind:'han'; char: string; reading: string|null }` — **một ký tự**.

**(a) — MẶC ĐỊNH ĐỀ XUẤT: một segment `han` mang MỘT TỪ.**

```ts
| { kind: 'han'; chars: string[]; readings: (string | null)[] }   // chars.length === readings.length
```

**Vì sao:** nó là hình dạng **duy nhất** giữ được cả ba: ① một `<ruby>` mỗi từ *(AC1)*;
② âm đọc của **từng ký tự** vẫn tra qua `hanVietByChar` **không đổi** *(bảng khoá theo ký tự,
`sourcePanelState.ts:166` — story này **không** chạm nó)*; ③ ánh xạ ngược vị trí→ký tự nguồn
vẫn cắt được đúng biên.

🔴 **HỆ QUẢ BẮT BUỘC PHẢI CÀI, KHÔNG ĐƯỢC QUÊN** *(đây là chỗ dễ vỡ AC nhất của story)*: hôm
nay `resolveParallel` *(`SourceHanViet.vue:330-334`)* và nhánh `han` của `resolveSelection`
*(`:386`)* lấy **TRỌN** ký tự nguồn **không cắt theo range** — hợp lệ vì một segment = một ký
tự nên *"chọn nửa"* là bất khả. **Segment = một TỪ làm điều đó KHẢ THI ngay lập tức.** ⇒ cả
hai chỗ phải thêm phép cắt theo `range.startOffset`/`endOffset`, y hệt nhánh `text` đã có
*(`:341-349` và `:392-395`)*. Không cắt ⇒ bôi đen nửa từ trả về cả từ ⇒ **vỡ AC12**.

**(b) — giữ một ký tự một segment, gom từ ở tầng render.** **Bác.** Nó để `switchView`/
`resolveSelection` tiếp tục nghĩ theo ký tự trong khi DOM nghĩ theo từ — hai mô hình lệch nhau
trong cùng một tệp, đúng lớp lỗi mà lượt review 1.18 đã bắt hai lần *(rò ký tự ngoài vùng chọn)*.

### 🔴 Quyết định #3 — Kiểu **chuyển đổi**: `resolveSelection` viết lại theo đường nào *(ĐẮT NHẤT)*

Hôm nay `.hv-switch` là **một text node thuần** và `resolveSelection` **đòi đúng điều đó**
*(`SourceHanViet.vue:366-371`: `range.startContainer !== node ⇒ null`)*. Cấu trúc span mới
**phá thẳng** bất biến này ⇒ không sửa thì hàm **luôn trả `null`**, tức **Auto-Lookup CHẾT
HẲN** ở kiểu chuyển đổi. Đây là hồi quy nặng nhất story có thể gây ra.

**(a) — MẶC ĐỊNH ĐỀ XUẤT: đọc DOM trực tiếp, cùng khuôn `resolveParallel`.**

Duyệt `host.children`, `intersectsNode`, cắt theo offset ở phần tử biên — bỏ hẳn đường
`text`/`map`/`starts` phẳng cho kiểu chuyển đổi.

**Vì sao:** ① nó **miễn nhiễm** với việc engine chèn `\n` hay `U+2060` vào `toString()` — bài
học AC12 nói thẳng `user-select:none` **không** ràng buộc `Selection.modify()` trên WebKit;
② hai kiểu xem dùng **cùng một khuôn đọc**, nên lỗi sửa một lần ăn cả hai; ③ nó bỏ được ba
bảng phải giữ đồng bộ tay.

**(b) — giữ `map`/`starts`, tính lại offset qua cấu trúc span lồng.** **Bác** *(mặc định)*:
phải dịch giữa "offset trong chuỗi phẳng" và "offset trong node cụ thể" ở mọi biên — đúng lớp
lỗi off-by-one mà lượt review 1.18 đã bắt, nhân lên vì nay có thêm `U+2060` vô hình trong chuỗi.

⚠️ **Nếu chốt (b)**: `U+2060` **có mặt trong `text`** nên `map`/`starts` phải đếm nó, và mọi
phép `slice` phải loại nó khỏi chuỗi trả về.

### Quyết định #4 — Từ có ký tự KHÔNG có âm thì hiện thế nào

Hôm nay: mỗi ký tự thiếu âm hiện `READING_PLACEHOLDER` = `·` *(một ký tự, Ice chốt ở review
1.16)*. Khi gom từ, một từ có thể **có âm một phần** *(vd `台` có âm, `湾` không)*.

**(a) — MẶC ĐỊNH ĐỀ XUẤT: giữ placeholder ở đúng âm tiết thiếu**, vẫn nối `U+2060` như mọi
âm khác ⇒ `台湾` → `thai⁠·`. Cụm vẫn là một `<ruby>`, vẫn double-click được, và chỗ thiếu vẫn
**nhìn thấy được** — đúng tinh thần AD-44 ④ *(rỗng có lý do không được trông giống rỗng im
lặng)*.

**(b) — coi cả từ là không xác định** *(`台湾` → `·`)*. **Bác:** nó **giấu** thông tin đã có
*(âm của `台`)*, và làm người đọc tưởng cả cụm không tra được.

---

## Acceptance Criteria

### AC1 — Double-click ở tab Hán Việt chọn CẢ CỤM TỪ, cả hai kiểu xem

**Given** kiểu xem **song song**, con trỏ trên một ký tự thuộc một từ nhiều ký tự *(vd `台湾`)*
**When** người dùng **double-click**
**Then** vùng chọn phủ **trọn từ**, và Auto-Lookup phát truy vấn **`台湾`** — không phải `台`

**Given** kiểu xem **chuyển đổi**, con trỏ trên một âm thuộc một từ nhiều âm *(`thai loan`)*
**When** người dùng **double-click**
**Then** vùng chọn phủ **trọn cụm âm**, và truy vấn gửi đi là **`台湾`** *(ký tự nguồn, không
chuỗi âm Latin)*

**Given** một ký tự Hán **đứng một mình** thành một từ *(vd `跨` trong `跨境`→ nếu ICU tách rời)*
**When** double-click
**Then** chọn đúng ký tự đó — không gom lấn sang ký tự bên cạnh

### AC2 — 🔴 Hành vi ĐỐI CHIẾU ĐƯỢC với tab nguyên văn

**Given** cùng một đoạn văn bản
**When** double-click cùng một vị trí ở tab **Trung** và ở tab **Hán Việt**
**Then** **cùng một cụm** được chọn — đây là mệnh đề Ice nêu bằng chữ *(*"nó phải nên được xử
lý theo phần văn bản gốc chứ"*)*
**And** nghiệm thu bằng một **bảng đối chiếu ≥ 20 vị trí**, ghi vào §Debug Log References —
không bằng *"đã kiểm bằng mắt"*

### AC3 — Tách từ đi bằng `Intl.Segmenter`, KHÔNG một phụ thuộc nào

**Given** cơ chế tách từ
**When** rà
**Then** nó là **`Intl.Segmenter('zh', { granularity: 'word' })`** — **0** phụ thuộc npm mới,
**0** dòng Rust, **0** điểm ra mạng *(NFR13/NFR15/AD-15)*
**And** `package.json` và `Cargo.toml` **không đổi một dòng**
**And** có **một đường lui đo được** khi engine thiếu API: `typeof Intl.Segmenter !== 'function'`
⇒ rơi về **một ký tự một từ** *(hành vi hôm nay)* kèm `console.error` **nêu đích danh** —
không im lặng, không ném

### AC4 — 🔴 Ranh giới AD-17 ghi thành CHỮ, không để suy luận

**Given** module tách từ mới
**When** đọc doc-comment của nó
**Then** nó nói rõ: đây là **"chọn gì để tra"** *(vùng chọn — AD-1 cho phép frontend)*, **không**
là **"tra thế nào"** *(khớp ngôn ngữ — AD-17, `jieba-rs`, Rust, Story 1.12)*
**And** trích **cả hai** nguồn: `ARCHITECTURE-SPINE.md:75-79` và `review-ad-44-2026-08-05.md:50`
**And** `core/matching/**` **không đổi một dòng**

### AC5 — 🔴 AC6 của Story 1.16 ĐO LẠI, không suy từ số cũ

**Given** kiểu xem **song song** trên cấu trúc `<ruby>`-mỗi-TỪ mới
**When** bôi đen bằng **chuột** một đoạn nhiều từ
**Then** `window.getSelection().toString()` cho ra **đúng chuỗi ký tự nguồn** — không lẫn âm
Hán Việt, không lẫn khoảng trắng chèn thêm, **không lẫn `U+2060`**
**And** phép kiểm chạy **LẠI** trên cấu trúc mới, số đo cũ **không được dùng lại**
*(nguyên văn ràng buộc của chính AC6: *"Nghiệm thu bằng một phép kiểm thật trên
`window.getSelection().toString()`, không bằng lời hứa"*)*

**Given** người dùng **copy** một đoạn đã bôi đen *(⌘C)*
**When** dán ra ngoài
**Then** **không một ký tự `U+2060` vô hình nào** đi theo — 🔴 **đây là hàng rào MỚI story này
tự tạo ra nhu cầu**, chưa có cơ chế nào hôm nay xử lý ký tự đó

### AC6 — 🔴 AC12 của Story 1.18 ĐO LẠI, cả hai kiểu xem

**Given** Auto-Lookup phát từ tab Hán Việt
**When** bôi đen **trọn một từ**, và **một phần một từ**, ở **cả hai** kiểu xem
**Then** truy vấn gửi đi **đúng bằng phần ký tự nguồn đã chọn** — không thừa một ký tự nào
**And** ca *"chọn nửa từ"* phải có **ít nhất một phép kiểm riêng** — nó **không tồn tại** trước
story này *(một segment = một ký tự)* và **trở nên khả thi** từ story này *(Quyết định #2,
§hệ quả bắt buộc)*

**Given** kiểu **chuyển đổi**
**When** Auto-Lookup phát
**Then** nó **vẫn hoạt động** — 🔴 đây là hồi quy nặng nhất story có thể gây ra
*(`resolveSelection` trả `null` nếu không viết lại — Quyết định #3)*

### AC7 — AC11 của Story 1.18 không hồi quy *(NFR17)*

**Given** bốn command `selection.extend_*` và `selection.focus_source`
**When** dùng **chỉ bàn phím** trên cả hai kiểu xem
**Then** đặt được caret **và** mở rộng vùng chọn theo ký tự/theo từ **y như trước story này**
**And** truy vấn phát ra từ đường bàn phím cũng **đúng ký tự nguồn** *(cùng phép kiểm AC6)*
**And** **không** thêm một điểm dừng `Tab` nào *(§KHÔNG-LÀM ③)*

### AC8 — Thị giác: âm đọc tách bạch, chữ Hán không dính

**Given** kiểu **chuyển đổi**
**When** hiển thị
**Then** âm trong **cùng một từ** có khe hở **nhìn thấy được** *(vẽ bằng CSS, không bằng ký tự
`U+2060` rộng 0)*, và hai **từ** khác nhau tách xa hơn khe trong từ — đọc được ranh giới từ
**bằng mắt**
**And** không một giá trị khoảng cách nào viết thẳng — token *(Kiểm B2 của `check:tokens`)*

**Given** kiểu **song song**
**When** một từ có âm dài *(vd `khoác cảnh`)*
**Then** ô ruby nới theo `<rt>` như cơ chế hiện tại, **không** thêm `min-width` tay
*(đã gỡ 2026-08-07 — đừng dựng lại)*

### AC9 — Trần render đo LẠI, không chép số cũ

**Given** `PARALLEL_VIEW_RENDER_CEILING = 50_000` *(`sourcePanelState.ts:105`)*
**When** cấu trúc đổi từ một `<ruby>`/ký-tự sang một `<ruby>`/từ *(số node **giảm**)*
**Then** đo lại chi phí render và **ghi số**; giữ hoặc nâng trần **theo số đo**, không theo cảm
tính
**And** nếu không đo được thì **ghi ra**, giữ nguyên 50.000, **không** nâng

### AC10 — Mọi cổng xanh, sàn nâng theo số THẬT

**Given** bộ DoD **chín lệnh**
**When** chạy
**Then** cả chín **exit 0**, `cargo test` không tụt dưới **232**
**And** mọi hằng `*_FLOOR` bị vượt được nâng **theo số thật**
**And** ba ranh giới không chạm đếm lại: `matchMedia` **0** · `window.innerWidth` **0** · phụ
thuộc mới **0** *(npm lẫn crate)*
**And** mỗi mục mới vào `ALLOWED_GLOBAL_MEMBERS` *(hôm nay **11**)* kèm **một dòng nói nó phục
vụ AC nào**

### AC11 — Nói THẬT hai thứ không nghiệm thu được

**Given** `Intl.Segmenter` trên **WKWebView** *(macOS — NFR14)*
**When** không dựng được bản Tauri thật để đo
**Then** **ghi thẳng** vào Completion Notes, **không** đánh dấu đạt *(cùng kỷ luật món nợ hai
nền tảng mà 1.6/1.14/1.16/1.17/1.18 đã giữ)*

**Given** chất lượng tách từ trên văn bản **TIỂU THUYẾT** thật *(tên riêng, từ cổ, thành ngữ)*
**When** mẫu đã đo chỉ là văn bản **tin tức**
**Then** chạy thêm trên **ít nhất một Chương tiểu thuyết thật**, ghi các ca ICU cắt **sai** vào
Completion Notes — một danh sách ca sai **có thật** đáng giá hơn một lời khẳng định *"tách tốt"*

---

## Tasks / Subtasks

### Task 0 — Chốt bốn quyết định, dọn cây làm việc *(AC toàn bộ)*

- [x] 🔴 **Xử lý 4 tệp chưa commit TRƯỚC** *(xem §Bối cảnh git)*: commit riêng hay cuốn vào
      story — quyết định và ghi ra. Đừng bắt đầu trên một cây bẩn.
- [x] Xác nhận baseline **đã đo**: `cargo test` **232** · **7** cổng `check:*` · `npm run build`.
- [x] Chốt **#1** *(AD-1 — CHẶN)*, **#2** *(hình dạng `Segment` — CHẶN)*, **#3** *(đường viết
      lại `resolveSelection` — ĐẮT NHẤT)*, **#4**.
- [x] Ghi cả bốn vào Change Log **kèm lý do**, không chỉ kèm lựa chọn.

### Task 1 — Module tách từ *(AC3 · AC4 · Quyết định #1)*

- [x] Module mới ở `src/panels/` *(cùng cửa `selectionContract.ts` — **KHÔNG** ở `src/commands/**`,
      xem §Bẫy 7)*: nhận văn bản nguồn, trả ranh giới từ.
- [x] **Đường lui khi thiếu API** *(AC3)*: `typeof Intl.Segmenter !== 'function'` ⇒ một ký tự
      một từ + `console.error` nêu đích danh. Không im lặng, không ném.
- [x] 🔴 **Doc-comment mang mệnh đề AD-17** *(AC4)* — trích cả `ARCHITECTURE-SPINE.md:75-79`
      lẫn `review-ad-44-2026-08-05.md:50`.
- [x] ⚠️ Cân nhắc **cache** theo `sourceText`: `Intl.Segmenter` chạy trên toàn Chương và
      `segments` là một `computed` — đo trước khi tối ưu, nhưng đừng để nó chạy lại mỗi lần
      render *(NFR1)*.

### Task 2 — `buildSegments` gom theo TỪ *(AC1 · Quyết định #2 · #4)*

- [x] Đổi kiểu `Segment`: nhánh `han` mang **một từ** *(`chars[]` + `readings[]`, cùng độ dài)*.
- [x] Âm từng ký tự **vẫn** tra qua `hanVietByChar` *(`sourcePanelState.ts:166` — **không sửa**)*.
- [x] Ký tự thiếu âm ⇒ `READING_PLACEHOLDER` ở **đúng âm tiết đó** *(Quyết định #4a)*.
- [x] ⚠️ Ranh giới từ chỉ gom **ký tự Hán liên tiếp** — mẩu không-Hán *(dấu câu, số, Latin)*
      vẫn là segment `text` như cũ.

### Task 3 — Kiểu song song: một `<ruby>` mỗi TỪ *(AC1 · AC5 · AC8)*

- [x] Template: `.hv-unit` bọc **một** `<ruby>` mang **cả từ**; `<rt>` mang các âm nối `U+2060`.
- [x] 🔴 **Giữ lối viết dính liền `><`** — một khoảng trắng thật lọt vào `v-for` là một ký tự
      chèn thêm vào vùng chọn *(bất biến của AC6, `SourceHanViet.vue:21-24`)*.
- [x] 🔴 **`resolveParallel` THÊM PHÉP CẮT theo range** *(Quyết định #2, §hệ quả)* — `:330-334`
      hôm nay lấy trọn `textContent`; nay chọn nửa từ là **khả thi**.
- [x] Giữ `<rt> user-select: none` và cả năm biến `--*-source-hanviet` *(AC7 của 1.16)*.

### Task 4 — 🔴 Kiểu chuyển đổi: viết lại `resolveSelection` *(AC1 · AC6 · Quyết định #3 — ĐẮT NHẤT)*

- [x] Cấu trúc mới cho `.hv-switch`: âm trong cùng từ nối `U+2060`, khe hở **vẽ bằng CSS**.
- [x] 🔴 **Viết lại `resolveSelection` nhánh `switch`** — bất biến *"đúng MỘT text node"*
      *(`:366-371`)* **bị phá có chủ đích**. Không sửa ⇒ hàm luôn trả `null` ⇒ **Auto-Lookup
      chết hẳn** ở kiểu này.
- [x] 🔴 **`U+2060` KHÔNG được lọt vào chuỗi truy vấn lẫn clipboard** *(AC5, AC6)* — chưa có
      hàng rào nào hôm nay biết tới ký tự này.
- [x] Luật chèn khoảng trắng: khoảng trắng **thật** chỉ giữa hai **TỪ**; trong từ dùng `U+2060`.
      `selfSpacing()` *(chữ số/Latin nửa rộng vs dấu câu toàn rộng)* **giữ nguyên logic**, chỉ
      đổi chỗ áp.

### Task 5 — Đo lại ba mệnh đề sống *(AC5 · AC6 · AC7)*

- [x] **AC6/1.16**: `Selection.toString()` bằng **kéo chuột thật** trên cấu trúc mới — ghi
      chuỗi vào/ra.
- [x] **Copy/paste**: dán ra ngoài, xác nhận **0** ký tự `U+2060`.
- [x] **AC12/1.18**: truy vấn Auto-Lookup — trọn từ **và nửa từ**, **cả hai** kiểu xem.
- [x] **AC11/1.18**: đường bàn phím `Selection.modify()` — caret + mở rộng ký tự/từ.
- [x] Bảng đối chiếu **≥ 20 vị trí** tab Trung ↔ tab Hán Việt *(AC2)*.

### Task 6 — Trần render, cổng, sàn *(AC9 · AC10)*

- [x] Đo lại chi phí render kiểu song song; giữ/nâng `PARALLEL_VIEW_RENDER_CEILING` **theo số**.
- [x] 🔴 **Sửa hai doc-comment thành sai ở `sourcePanelState.ts:84` và `:102`** *(Bẫy 10)* —
      mệnh đề *"mỗi lần xuất hiện sinh một `.hv-unit` riêng"* không còn đúng sau khi gom từ.
      Logic `hanCharOccurrenceCount` **giữ nguyên** *(vẫn là proxy hợp lý)*, chỉ sửa chữ.
- [x] 🔴 Chạy `npm run check:i18n` **ngay sau** lượt đổi template *(Bẫy 9 — năm dấu
      `aura-allow-text` sẽ xê dịch)*, đừng dồn tới cuối.
- [x] Nâng mọi `*_FLOOR` bị vượt — số thật vào Completion Notes.
- [x] Đếm lại: `matchMedia` **0** · `window.innerWidth` **0** · phụ thuộc mới **0**.
- [x] Mục mới vào `ALLOWED_GLOBAL_MEMBERS` *(nếu có)* kèm dòng lý do AC.

### Task 7 — Bàn giao và nói thật *(AC11)*

- [x] Chạy **chín** lệnh DoD lần cuối cùng lượt.
- [x] Chạy trên **một Chương tiểu thuyết thật**; ghi các ca ICU cắt **sai** *(danh sách có thật)*.
- [x] Ghi thẳng: `Intl.Segmenter` trên **WKWebView** đo được hay không.
- [x] 🔴 **Viết lại banner đầu `SourceHanViet.vue`** *(`:4-24`)* — nó đang tuyên bố *"MỘT NODE
      CHO MỖI KÝ TỰ"* là điều kiện tiên quyết của 1.18/3.4, và story này **phá tuyên bố đó có
      chủ đích**.
- [x] Rà **Story 3.4** *(Glossary highlight trong Panel Source)*: nó có phụ thuộc cấu trúc
      một-ký-tự-một-node không? Ghi kết luận — `selectionContract.ts:181` đã gọi tên nó.
- [x] `deferred-work.md`: đóng mục §*nghiệm thu tay tab Hán Việt*; ghi món nợ **mới** nếu có.
- [x] `src/panels/README.md`: hàng 1.18b + đoạn mô tả cơ chế tách từ. 🔴 **Và sửa dòng `:95`** —
      nó đang nói *"đọc node `.hv-char`"*, một lớp **đã không còn tồn tại** *(gỡ ở lượt vá
      `<ruby>` 2026-08-07)*. Đây là một chú thích đã sai **từ trước story này**; đừng để nó
      sai thêm một vòng nữa.

### Review Findings *(code review 2026-08-08 — ba lớp song song + số đo của người rà soát)*

- [x] **[Review][Patch] Kiểu CHUYỂN ĐỔI đi từ MỘT text node sang O(ký tự) phần tử, và nó là đường lui KHÔNG có trần** — 🔴 **Ice chốt 2026-08-08: CHẤP NHẬN, ghi số vào `sourcePanelState.ts` và mở một món nợ CÓ SỐ ĐO.** Không đặt trần cho kiểu chuyển đổi *(Chương lớn sẽ không còn đường xem nào)*, không dựng `.hv-syl` theo yêu cầu *(một cơ chế mới, đắt hơn khoản nó vá)*. — Trước story này `.hv-switch` là `<p>{{ switchText }}</p>`, **đúng một** text node; đó chính là lý do bảng đo của `PARALLEL_VIEW_RENDER_CEILING` ghi kiểu chuyển đổi **222,4 ms ở 500.000 ký tự** và gọi nó *"rẻ, tuyến tính"*. Nay mỗi từ sinh một `.hv-word` và **mỗi ký tự** một `.hv-syl`. Đo lại 2026-08-08 *(cùng bàn đo tái lập được số cũ: 23,6 ms vs 24,2 ms của bảng gốc ở 50.000 ⇒ so sánh được)*: ở **50.000 ký tự Hán**, kiểu chuyển đổi đi từ **23,6 ms · 1 node** lên **532,9 ms · 136.864 node** — **gấp 23 lần**; ở 100.000 là **1.038,5 ms**, tuyến tính ⇒ ngoại suy **~5 s ở 500.000**. Và `canUseParallelView` chỉ khoá **kiểu song song**: trên 50.000 người dùng bị **ép vào đúng bề mặt vừa bị làm nặng lên**, không trần nào che. Bảng AC9 của story chỉ đo **song song mới vs song song cũ**; nhánh chuyển đổi **chưa được đo một lần nào**. *(Ghi chú phụ: cột "node" của bảng AC9 đếm **con trực tiếp** của host, không phải node DOM — đo cùng bàn: song song mới ở 50.000 là **142.128** node DOM, không phải 34.714.)* ⇒ Cần Ice quyết: đặt trần cho cả kiểu chuyển đổi · hay chỉ dựng `.hv-syl` khi thật cần · hay chấp nhận và ghi số vào `sourcePanelState.ts`.
- [x] **[Review][Patch] `resolveSwitch` bỏ chốt "vùng chọn phải nằm TRỌN trong bề mặt", không ghi lý do** [src/panels/SourceHanViet.vue:505-518] — 🔴 **Ice chốt 2026-08-08: GIỮ lượt bỏ chốt, và GHI LÝ DO vào doc-comment** *(nó đồng bộ hai kiểu xem về cùng một khuôn)*. — Bản trước có chốt tường minh kèm lý do bằng chữ *(`range.startContainer !== node ⇒ null`; chú thích: "kéo qua dòng trạng thái, qua dòng nguồn thì không ánh xạ được, và `null` nghĩa là không phát lượt tra")*. Bản mới chỉ còn `if (range.collapsed) return null`. Kéo chọn **từ trong** `.hv-switch` ra ngoài qua `.hv-notice`/`.hv-sources` nay **phát** một lượt tra cho phần ký tự Hán đã phủ, thay vì im lặng. Rủi ro thấp *(đường tra đọc DOM nên không bao giờ tra được chuỗi âm Latin — đúng thứ chốt cũ sinh ra để chặn, nay bất khả về cấu trúc)*, và `resolveParallel` **vốn đã** hành xử như vậy từ 1.18 ⇒ đây là một lượt **đồng bộ hai kiểu xem**. Nhưng nó là một đổi hành vi có chủ đích mà story không ghi ở đâu cả. ⇒ Ice quyết: giữ *(và ghi lý do vào doc-comment)* hay dựng lại chốt.
- [x] **[Review][Patch] Doc-comment nói `<br>` "không có nội dung để chạm hụt" — số đo bác** [src/panels/SourceHanViet.vue:355-359] — Đo trên DOM thật 2026-08-08: vùng chọn từ **cuối** dòng trên tới **đầu** dòng dưới cho `overlapsRange(range, br) === true`, tức `<br>` **có** bị đánh dấu phủ dù không phủ nội dung nào. Hệ quả thực tế bằng **không**: chuỗi trả về là `"\n"`, và `runLookup` *(`lookupPanelState.ts:253-254`)* có `if (trimmed === '') return` nuốt trọn nó; `\n` ở **giữa** vùng chọn thì đúng bằng thứ người dùng đã kéo qua, và mã **trước** story này cũng làm y hệt. ⇒ Không phải hồi quy, không vỡ AC6 — nhưng **câu chú thích đang nói sai sự thật**, đúng lớp nợ mà Bẫy 10 của chính story bắt phải dọn. Sửa chữ, không sửa logic.
- [x] **[Review][Patch] `new Intl.Segmenter(...)` không bọc `try/catch`, phá cam kết "không ném" của chính nó** [src/panels/wordBoundary.ts:79] — Doc-comment và AC3 hứa *"không im lặng, không ném"*, nhưng chốt chặn chỉ phủ ca **vắng API** *(`typeof … !== 'function'`)*. Một engine có API mà constructor ném *(bản polyfill hỏng, ICU bị cắt)* sẽ ném xuyên qua `wordStartOffsets` → `buildSegments` → `computed(segments)` ⇒ **sập cả bề mặt Hán Việt**, cả hai kiểu xem. Xác suất thấp *(`'zh'`/`'word'` đều hợp lệ theo ECMA-402, thiếu dữ liệu locale thì rơi về mặc định chứ không ném)*, giá sửa gần bằng không: bọc `try/catch` và rơi về **cùng** đường lui đã có.
- [x] **[Review][Patch] §Debug Log ghi "đo lại AC5·AC6·AC7" nhưng mẫu đo KHÔNG có một `\n` nào** — Cả bảng đối chiếu 26 vị trí, bảng "bôi đen nửa từ", lẫn bảng bàn phím đều chạy trên **một câu một dòng** *(`台湾地方议会…8月5日`)*. Nhánh `kind:'break'`/`<br>` — thứ **Bẫy 4** gọi đích danh là rủi ro riêng của `.hv-switch` — chưa lượt nào đi qua. Task 5 vẫn tích `[x]`. Sửa **lời khai** cho đúng phạm vi đã đo *(hoặc đo bổ sung một mẫu nhiều dòng)*; kết luận không đổi vì `runLookup` đã chặn, nhưng một checkbox nói quá là thứ lượt review sau phải trả tiền lại.
- [x] **[Review][Defer] `onCopy` không chặn rò âm `<rt>` qua `Selection.modify()` trên WKWebView ở kiểu song song** [src/panels/SourceHanViet.vue:571-582] — deferred, pre-existing. `onCopy` chỉ vào cuộc khi chuỗi chứa `WORD_JOINER`; kiểu song song không sinh ký tự đó nên nó thoát sớm, để lượt copy đi đường mặc định — và chính doc-comment `:409-412` đã đo rằng đường bàn phím trên WKWebView cho ra `他tha打đả開khai…`. AC5 chỉ đòi **`U+2060`**, và lớp lỗi này có **trước** 1.18b *(`<rt>` đã tồn tại từ lượt vá `51132cb`)* ⇒ ngoài phạm vi story.
- [x] **[Review][Defer] Ranh giới ICU của `Intl.Segmenter('zh')` trên WKWebView có trùng ranh giới double-click của WebKit không** [src/panels/wordBoundary.ts:48-52] — deferred, pre-existing. Doc-comment tuyên bố AC2 đúng *"theo cấu trúc"* vì hai bên dùng **cùng** bộ tách từ ICU; đó là một chi tiết **cài đặt của engine**, không một hợp đồng chuẩn hoá, và toàn bộ số đo chạy trên Chromium. Story **đã khai thật** ở AC11 ① và đã ghi nợ ở `deferred-work.md` §1-18b ⇒ giữ nguyên ở đó, không mở nợ trùng.

---

## Dev Notes

### Trạng thái repo hôm nay — SỐ, không mô tả *(đo 2026-08-07)*

| | Số thật |
|---|---|
| `cargo test` | **232 xanh** |
| Cổng `check:*` xanh | **7/7** *(+`npm run build` + `cargo test` = **9** lệnh DoD)* |
| Tệp `.vue` / `.ts` trong `src/**` | **13** / **26** |
| Khoá `vi.json` | **80** |
| Token typography | **17** · `deviations` | **8** |
| `ALLOWED_GLOBAL_MEMBERS` | **11** mục |
| `SELECTION_SURFACE_FLOOR` *(Kiểm F)* | **5** — 🔴 story này **không được** hạ |
| Lời gọi `Intl.Segmenter` trong `src/**` | **0** — story này là **người đầu tiên** |
| `matchMedia` / `window.innerWidth` | **0** / **0** — 🔴 giữ nguyên |
| `PARALLEL_VIEW_RENDER_CEILING` | **50.000** *(`sourcePanelState.ts:105`)* |
| Cây làm việc | 🔴 **KHÔNG SẠCH — 4 tệp** *(xem §Bối cảnh git)* |

### 📐 SỐ ĐO ĐÃ CÓ — ĐỪNG ĐO LẠI TỪ ĐẦU *(Chromium, 2026-08-07)*

| Đo | Kết quả |
|---|---|
| `Intl.Segmenter('zh',{granularity:'word'})` có mặt | ✅ |
| Tách câu mẫu | `台湾/地方/议会/接连/通过/提案/，/反对/中共/跨/境/镇压/。/北市/议会/8/月/5/日` — ĐÚNG |
| Song song: một `<ruby>` mỗi **ký tự** *(hiện tại)* | double-click ⇒ `""` — **hỏng hoàn toàn** |
| Song song: một `<ruby>` mỗi **TỪ** | double-click ⇒ `台湾` ✅ |
| Chuyển đổi: âm cách nhau bằng **dấu cách** *(hiện tại)* | ⇒ `thai` — một âm |
| Chuyển đổi: nối `U+2060` + khe vẽ bằng **CSS margin** | ⇒ `thai⁠loan` ✅ |
| `U+00A0` · `U+2009` làm chất nối | ❌ vẫn cắt ở dấu cách |
| Tab nguyên văn *(đối chứng)* | ⇒ `台湾` ✅ — **đây là hành vi đích của AC2** |

⚠️ **Bẫy đã biết:** `U+2060` rộng **bằng 0** ⇒ dùng nó **thay** dấu cách làm chữ dính
*(`thailoan`)*. Đường chạy được là **tách hai vai**: `U+2060` giữ *tính liền từ* cho engine,
*khoảng cách nhìn thấy* do **CSS** vẽ.

### API thật — chép từ MÃ, không từ trí nhớ

```ts
// src/panels/SourceHanViet.vue — BA hàm story này VIẾT LẠI
type Segment = {kind:'break'} | {kind:'text';text:string} | {kind:'han';char:string;reading:string|null}
function buildSegments(text: string): Segment[]                      // :80  — gom theo TỪ
function resolveParallel(selection: Selection): string | null        // :305 — THÊM phép cắt
function resolveSelection(selection: Selection): string | null       // :360 — VIẾT LẠI nhánh switch
const switchView: ComputedRef<{text:string;map:number[];starts:number[]}>  // :186

// src/panels/sourcePanelState.ts — KHÔNG SỬA
hanVietByChar: ComputedRef<ReadonlyMap<string, CharacterReading>>     // :166 — khoá theo KÝ TỰ, vẫn dùng được
isHanChar(char: string): boolean                                     // :52
canUseParallelView: ComputedRef<boolean>                             // :164
PARALLEL_VIEW_RENDER_CEILING = 50_000                                // :105

// src/panels/selectionContract.ts — KHÔNG SỬA
type SelectionResolver = (selection: Selection) => string | null     // :63  — `null` ⇒ không phát tra
useSelectionSurface(elRef, role, resolve?): void                     // :120 — `role` LITERAL (Kiểm F)
```

### ⚠️ TÁM CÁI BẪY — sáu trong tám cho ra CI **XANH** với kết quả **VÔ NGHĨA**

1. 🔴 **`resolveSelection` không viết lại ⇒ Auto-Lookup CHẾT ở kiểu chuyển đổi.** Điều kiện
   `range.startContainer !== node` *(`:371`)* trả `null` cho **mọi** vùng chọn khi `.hv-switch`
   không còn một text node duy nhất. Không lỗi, không cổng nào đỏ — chỉ là tra cứu **im lặng
   thôi chạy**. Nguy hiểm nhất của story.
2. 🔴 **Quên thêm phép CẮT cho segment `han`.** `:330-334` và `:386` lấy **trọn** ký tự nguồn.
   Hợp lệ khi segment = một ký tự; **vỡ AC12 ngay** khi segment = một từ. Lượt review 1.18 đã
   bắt **đúng lớp lỗi này hai lần** ở nhánh `text` — đừng để nó tái phát ở nhánh `han`.
3. 🔴 **`U+2060` rò vào clipboard.** Người dùng copy nguyên văn và dán ra Word, dính ký tự vô
   hình. **Không hàng rào nào hôm nay biết tới nó.**
4. 🔴 **Cấu trúc span mới trong `.hv-switch` làm engine chèn `\n` vào `toString()`.** Bài học
   này *(`:538-572`)* trước nay chỉ áp cho `.hv-parallel`; `.hv-switch` **chưa từng có node
   nào** nên rủi ro **quay lại từ đầu** cho riêng nó. Phòng bằng cách đọc DOM *(Quyết định #3a)*,
   **chủ động**, không phải "quên".
5. **Tin `Selection.toString()` trên WKWebView.** `user-select:none` **không** ràng buộc
   `Selection.modify()` trên WebKit *(số đo AC12 của 1.18)* — đường bàn phím vẫn rò âm.
6. **Đo tách từ chỉ trên văn bản tin tức.** Tiểu thuyết mang tên riêng, từ cổ, thành ngữ; ICU
   cắt khác. Một danh sách ca sai **có thật** đáng giá hơn *"tách tốt"*.
7. **Đặt module tách từ vào `src/commands/**`.** Tệp đó phải nạp được bằng **Node thuần** để
   Kiểm C/D/E của `check:commands` chạy trên chính bộ command sản phẩm. Một import DOM ở đó
   giết ba phép kiểm cùng lúc.
8. **Nâng `PARALLEL_VIEW_RENDER_CEILING` vì "chắc là nhanh hơn".** Số node giảm, nhưng mỗi
   `<ruby>` nay nặng hơn *(base nhiều ký tự, `<rt>` nhiều âm)*. **Đo, đừng suy.**
9. 🔴 **Làm lệch dấu `aura-allow-text` khi dựng lại template ⇒ `check:i18n` ĐỎ.** Kiểm A của
   `check-i18n.mjs` đòi dấu miễn trừ nằm **NGAY TRÊN** khai báo nó miễn trừ. Template hiện có
   **năm** dấu như vậy *(`SourceHanViet.vue:409,414,426,433,435`)*, và lượt tái cấu trúc sẽ
   xê dịch cả năm. ⚠️ **Đây là một tai nạn ĐÃ XẢY RA, không một lo xa**: `SourcePanel.vue:156-158`
   ghi nguyên văn *"chen vào giữa là vô hiệu hoá nó (bắt lúc chạy cổng, 2026-08-07)"*. Chạy
   `npm run check:i18n` **ngay sau** khi đổi template, đừng để tới cuối.
10. 🔴 **Để lại hai doc-comment nay SAI SỰ THẬT ở `sourcePanelState.ts`.** `:84` nói phép đo
    Playwright chạy trên *"DOM y hệt `.hv-unit`"*, `:102` nói *"mỗi lần xuất hiện sinh một
    `.hv-unit` riêng"* — mệnh đề thứ hai **thành sai** ngay khi gom từ *(một `.hv-unit` mỗi
    **TỪ**, không mỗi lần xuất hiện)*. `hanCharOccurrenceCount` vẫn là một proxy hợp lý nên
    **logic không cần đổi**, nhưng **hai câu đó phải sửa** — một chú thích nói dối tốn đúng
    một lượt code review, và story 1.16/1.17/1.18 đã trả giá đó ba lần.

### 🧠 Trí tuệ từ story trước — thứ đắt tiền, đừng học lại bằng tiền

- **1.18 · code review:** lấy trọn `textContent` khi vùng chọn chỉ chạm **một phần** — bắt được
  **hai lần** *(song song và chuyển đổi)*. ⇒ Bẫy 2 là **tái phát có báo trước**.
- **1.18 · lượt vá 2026-08-07:** ba lượt vá liên tiếp *(line-height 4.8 → 3.2 → `<ruby>`)* vì
  hai lượt đầu **vá triệu chứng**, không vá gốc *(âm đọc không chiếm chỗ trong layout)*.
  ⇒ ở đây: hỏi *"gốc rễ là gì"* trước khi chỉnh một con số.
- **1.18 · Ice lật kết luận:** dev bảo phải xây bộ tách từ ở Rust; Ice chỉ ra engine đã có sẵn.
  ⇒ **hỏi "thứ này đã tồn tại chưa" trước khi ước lượng một story lớn.**
- **1.16 · AC6:** `display: inline-flex` làm Chromium chèn `\n` vào `toString()`. ⇒ Bẫy 4.
- **1.17 · code review:** một `export` không ai `import` là lỗi **im lặng hoàn toàn** — tái phát
  nguyên văn từ 1.16. ⇒ mỗi hàm mới phải có chỗ tiêu thụ **nhìn thấy được**.

### Testing standards

Bộ DoD **chín lệnh** — **mã thoát là phán quyết**, không đầu ra:

```
cargo test --manifest-path src-tauri/Cargo.toml
npm run build            npm run check:tokens     npm run check:i18n
npm run check:commands   npm run check:layout     npm run check:deps
npm run check:dict-manifest                       npm run check:scope
```

- 🔴 **không có bộ chạy test frontend, và không được thêm** *(NFR15 — Ice chốt ở 1.5, giữ qua
  **bảy** story)*. ⇒ **toàn bộ** vế DOM *(vùng chọn, double-click, clipboard, bàn phím)* nghiệm
  thu bằng **bảng chạy tay CÓ SỐ** trong §Debug Log References.
- ⚠️ Story này **không có bề mặt Rust** ⇒ `cargo test` không canh được gì cho nó. **Lưới tự động
  duy nhất là các cổng `check:*`.**
- **Đỏ-rồi-xanh cho mọi cổng bị đụng**: mỗi mệnh đề mới phải có **ít nhất một ca làm cổng ĐỎ**
  cộng **một đối chứng âm**. Con số vào Completion Notes.

### Project Structure Notes

```
src/
  panels/<module tách từ>.ts    NEW     🔴 KHÔNG import vào src/commands/** (Bẫy 7)
                                        doc-comment mang mệnh đề AD-17 (AC4)
  panels/SourceHanViet.vue      UPDATE  🔴 buildSegments · resolveParallel · resolveSelection
                                        + template hai kiểu xem + CSS khe hở
                                        + VIẾT LẠI banner :4-24 (Task 7)
  panels/sourcePanelState.ts    UPDATE  ⚠️ CHỈ nếu trần render đổi theo số đo (AC9)
  panels/README.md              UPDATE  hàng 1.18b + đoạn cơ chế tách từ
scripts/check-*.mjs             UPDATE  ⚠️ CHỈ nâng sàn theo số thật (AC10)
_bmad-output/implementation-artifacts/deferred-work.md   UPDATE  đóng mục §nghiệm thu tay
```

⚠️ **không tệp Rust nào trong danh sách, và đó là một DỮ KIỆN của story** *(Quyết định #1)*.
Nếu bản cài của bạn chạm `src-tauri/**`, **dừng lại** và đọc lại §KHÔNG-LÀM ① — gần như chắc
chắn bạn đang dựng một Matcher thứ hai.

### 📌 Bối cảnh git

`8c54441` *(HEAD — refactor test messages)* · `09d9c87` *(1.18 · nạp lại Chương)* ·
`4136f3f` *(1.17)* · `cb03974` *(1.10c/1.16)*.

🔴 **Cây làm việc KHÔNG SẠCH — 4 tệp chưa commit**, tất cả từ lượt vá cùng ngày 2026-08-07:
`src/panels/SourceHanViet.vue` *(`<ruby>` thay `position:absolute`)* · `src/tokens/tokens.json`
*(gỡ `source-cjk-parallel`; `source-hanviet` 14,5px + bỏ nghiêng — hai `deviations` mới)* ·
`scripts/check-tokens.mjs` *(đếm 17, tám deviation)* · `deferred-work.md`.
⇒ **Quyết định ở Task 0**: commit riêng trước khi bắt đầu, hay cuốn vào story. **Đừng bỏ qua** —
chúng nằm ở **đúng tệp** story này sẽ mổ.

**Đọc gì trước khi gõ:**
`src/panels/SourceHanViet.vue` *(trọn — banner `:4-24`, `buildSegments` `:80`, `switchView`
`:186`, `resolveParallel` `:305`, `resolveSelection` `:360`, template `:404-439`, CSS
`:484-599`)* · `src/panels/sourcePanelState.ts` *(`:41-173`)* · `src/panels/selectionContract.ts`
*(`:63`, `:120-138`, `:201-212`)* · `deferred-work.md` §*nghiệm thu tay tab Hán Việt*.

### References

- [Source: `_bmad-output/implementation-artifacts/deferred-work.md` §Deferred from: nghiệm thu tay tab Hán Việt (Ice, 2026-08-07)] — **nguồn gốc + toàn bộ bảng đo**
- [Source: `ARCHITECTURE-SPINE.md:75-79`] — AD-1, danh sách frontend được giữ *(gồm **"vùng chọn"**)*
- [Source: `ARCHITECTURE-SPINE.md:230-236`] — AD-17, Matcher là đúng một cài đặt *(`jieba-rs`, Rust)*
- [Source: `ARCHITECTURE-SPINE.md:95-101`] — AD-4, ranh giới **segment** *(khác đơn vị với **từ**)*
- [Source: `ARCHITECTURE-SPINE.md:406-417`] — AD-34, handler chuột chỉ `dispatch`
- [Source: `ARCHITECTURE-SPINE.md:218`] — AD-16, không render nội dung ngoài thành HTML
- [Source: `reviews/review-ad-44-2026-08-05.md:50`] — 🔴 phân xử *"chọn gì để tra"* vs *"tra thế nào"*
- [Source: `_bmad-output/implementation-artifacts/1-16-panel-source-va-tab-han-viet.md:511-522`] — AC6 nguyên văn
- [Source: `_bmad-output/implementation-artifacts/1-18-auto-lookup.md:495-521`] — AC11 · AC12 nguyên văn
- [Source: `epics.md:1497-1526`] — Story 1.12, Matcher dùng chung
- [Source: `epics.md:503-505`] — UX-DR5 · UX-DR6
- [Source: `EXPERIENCE.md:131`] — FR21 Auto-Lookup *"không được thiết kế lại cho khác đi"*
- [Source: `EXPERIENCE.md:23`] — ba thứ ở Rust: tách **câu**, khớp ngôn ngữ, phân giải scope
- [Source: `DESIGN.md:287-291`] — sàn giãn dòng 1.66 cho họ `read`
- [Source: `prd.md:814,866,875,876,877,887`] — NFR1 · NFR13 · NFR14 · NFR15 · NFR16 · NFR17
- [Source: `mockups/tm-fuzzy-match.html:258,267-269`] — vì sao **không** tách từ cho **khớp** TM

---

## Dev Agent Record

### Agent Model Used

`claude-opus-5` (Claude Code / bmad-dev-story).

### Debug Log References

**Bàn đo:** một trang HTML chạy tay trong trình duyệt, **chép** DOM + CSS của
`SourceHanViet.vue` và **chép** logic của `wordBoundary.ts`/`buildSegments`/`resolveParallel`/
`resolveSwitch`. Nó **KHÔNG mount component thật** — component cần cầu IPC Tauri cho
`hanVietByChar`, và dự án **cố ý không có bộ chạy test frontend** (NFR15). Giới hạn này là món
nợ mới ở `deferred-work.md` §1-18b. Trang **không** được commit: `Project Structure Notes` của
story không có chỗ cho nó và không cổng nào canh nó.

Mọi double-click dưới đây là **double-click THẬT** (bơm chuột vào engine), không sự kiện tổng
hợp — `dispatchEvent('dblclick')` **không** tạo vùng chọn, chỉ đầu vào tin cậy mới tạo.

🔴 **PHẠM VI THẬT CỦA MỌI BẢNG DƯỚI ĐÂY — đính chính ở lượt code review 2026-08-08.** Cả ba
bảng *(đối chiếu 26 vị trí · "bôi đen nửa từ" · bàn phím)* chạy trên **một câu MỘT DÒNG**
(`台湾地方议会…8月5日`) — **không một `\n` nào**. Nghĩa là nhánh `kind:'break'`/`<br>`, thứ mà
**Bẫy 4** gọi đích danh là rủi ro riêng của `.hv-switch`, **chưa lượt nào đi qua**, dù Task 5
tích `[x]` cho *"đo lại ba mệnh đề sống"*. Ca thử trên Chương tiểu thuyết ở AC11 cũng chỉ chạy
`Intl.Segmenter` trên chuỗi, không kéo chọn tương tác qua ranh giới dòng.
⇒ Lượt review đã **đo bù** đúng nhánh đó: `overlapsRange(range, br)` trả `true` cho vùng chọn
từ cuối dòng trên tới đầu dòng dưới, nhưng `runLookup` *(`lookupPanelState.ts:253-254`)* có
`if (trimmed === '') return` nuốt trọn hệ quả, và mã **trước** story cũng làm y hệt ⇒ **không
hồi quy, không vỡ AC6**. Kết luận giữ nguyên; **lời khai** thì phải đúng phạm vi.

⚠️ **Hai cái bẫy của chính bàn đo, ghi lại để lượt sau không mất thời gian:** ① double-click ở
Chromium bắt theo **vị trí caret**, nên nhắm vào **tâm** một ký tự Hán làm nó chọn **từ kế
tiếp** — phải nhắm **1/4 ô** tính từ mép trái; ② log dài ra làm thanh cuộn xuất hiện ⇒ viewport
hẹp lại ⇒ văn bản **reflow giữa lúc đo toạ độ và lúc bấm**, cho ra một bảng sai hoàn toàn im
lặng. Khoá bằng `html { overflow-y: scroll }`.

#### AC2 · AC1 — bảng đối chiếu 26 vị trí, tab Trung ↔ tab Hán Việt

Mẫu: `台湾地方议会接连通过提案，反对中共跨境镇压。北市议会8月5日` — double-click lên **từng**
ký tự Hán (26 ký tự), ở **ba** bề mặt.

Ranh giới ICU đo được (khớp **đúng** bảng đã có trong `deferred-work.md`):
`台湾/地方/议会/接连/通过/提案/，/反对/中共/跨/境/镇压/。/北市/议会/8/月/5/日`

| # | ký tự | tab NGUYÊN VĂN (đối chứng) | song song — truy vấn | chuyển đổi — truy vấn |
|---|---|---|---|---|
| 0·1 | 台 · 湾 | `台湾` | `台湾` ✅ | `台湾` ✅ |
| 2·3 | 地 · 方 | `地方` | `地方` ✅ | `地方` ✅ |
| 4·5 | 议 · 会 | `议会` | `议会` ✅ | `议会` ✅ |
| 6·7 | 接 · 连 | `接连` | `接连` ✅ | `接连` ✅ |
| 8·9 | 通 · 过 | `通过` | `通过` ✅ | `通过` ✅ |
| 10·11 | 提 · 案 | `提案` | `提案` ✅ | `提案` ✅ |
| 12·13 | 反 · 对 | `反对` | `反对` ✅ | `反对` ✅ |
| 14·15 | 中 · 共 | `中共` | `中共` ✅ | `中共` ✅ |
| 16 | 跨 | `跨` *(một ký tự)* | `跨` ✅ | `跨` ✅ |
| 17 | 境 | `境` *(một ký tự)* | `境` ✅ | `境` ✅ |
| 18·19 | 镇 · 压 | `镇压` | `镇压` ✅ | `镇压` ✅ |
| 20·21 | 北 · 市 | `北市` | `北市` ✅ | `北市` ✅ |
| 22·23 | 议 · 会 | `议会` | `议会` ✅ | `议会` ✅ |
| 24 | 月 | `月` | `月` ✅ | `月` ✅ |
| 25 | 日 | `日` | `日` ✅ | `日` ✅ |

**26/26 khớp ở CẢ HAI kiểu xem. 0 lệch.** Ca *"một ký tự Hán đứng một mình thành một từ"*
(mệnh đề ba của AC1) có mặt thật trong mẫu: `跨` và `境` — ICU tách rời, và cả hai bề mặt chọn
đúng **một** ký tự, không gom lấn.

**Đối chứng ÂM — cấu trúc TRƯỚC story này** (một `<ruby>` mỗi KÝ TỰ, dựng cùng bàn đo):
số node **30** cho cùng đoạn văn, so với **19** của cấu trúc mới.

#### AC6 · AC12/1.18 — "bôi đen NỬA TỪ" (ca không tồn tại trước story này)

| ca | truy vấn | đạt |
|---|---|---|
| SW — chỉ âm `đài` của `台湾` | `台` | ✅ |
| SW — chỉ âm `loan` của `台湾` | `湾` | ✅ |
| SW — `loan` **gồm cả** `U+2060` đứng trước | `湾` | ✅ |
| SW — trọn từ `台湾` | `台湾` | ✅ |
| SW — bắc cầu `loan`..`địa` *(nửa từ này + nửa từ kia)* | `湾地` | ✅ |
| SW — chọn `hô` (một phần âm `thông`) | `通` | ✅ *(âm tiết là ATOM với ký tự nguồn — không có "nửa ký tự Hán")* |
| SW — ba từ liền | `台湾地方议会` | ✅ |
| SW — 🔴 `đài` → thả đúng **đầu** `loan` *(chạm biên)* | `台` | ✅ **sau khi sửa** — trước sửa: `台湾` (thừa 1 ký tự) |
| PAR — chỉ `台` của `台湾` | `台` | ✅ |
| PAR — chỉ `湾` của `台湾` | `湾` | ✅ |
| PAR — bắc cầu `湾`..`地` | `湾地` | ✅ |
| PAR — `台湾` → thả đúng **đầu** `地方` *(chạm biên)* | `台湾` | ✅ |

#### AC5 — `Selection.toString()` và CLIPBOARD

| đo | kết quả |
|---|---|
| SONG SONG · `toString()` **cả đoạn** | `台湾地方议会接连通过提案，反对中共跨境镇压。北市议会8月5日` — **bằng đúng** chuỗi nguồn |
| ↳ có `\n` chèn thêm không | **0** |
| ↳ có ký tự Latin (âm Hán Việt) rò từ `<rt>` không | **0** |
| ↳ có `U+2060` không | **0** *(kiểu song song không sinh ký tự này)* |
| CHUYỂN ĐỔI · `toString()` của một lượt double-click | `"đài⁠loan"` — **CÓ** `U+2060` (22/26 lượt) |
| ↳ **truy vấn** phát đi có `U+2060` không | **0/26** — đường truy vấn đọc DOM, không đọc `toString()` |
| ↳ **CLIPBOARD** sau `onCopy` | `"đài loan"` — **0/26** còn `U+2060` |

🔴 Clipboard đổi `U+2060` thành **một dấu cách**, **không xoá trắng**: xoá trắng cho ra
`"đàiloan"` — đúng thứ `U+2060` sinh ra để tránh trên màn hình, chỉ dời sang clipboard.

#### AC7 · AC11/1.18 — đường BÀN PHÍM (`Selection.modify`), KHÔNG hồi quy

Caret đặt ở đầu bề mặt, rồi `modify('extend','right', …)` n lần. Số **ký tự nguồn** thu được:

| n | 1 | 2 | 3 | 4 | 6 | 8 | 10 | 12 |
|---|---|---|---|---|---|---|---|---|
| `word` · **cấu trúc CŨ** (ô = ký tự) | 台 | 台 | 台湾 | 台湾 | 台湾地 | 台湾地方 | …议 | …议会 |
| `word` · mới, `inline-block` | 台湾 | 台湾 | 台湾 | 台湾 | 台湾 | 台湾 | 台湾 | 台湾 |
| `word` · mới, **`inline`** ⇐ đã giao | 台湾 | 台湾 | 台湾地方 | 台湾地方 | 台湾地方议会 | …接连 | | |
| `char` · **cấu trúc CŨ** | 台 | 台 | 台湾 | 台湾 | 台湾地 | 台湾地方 | …议 | …议会 |
| `char` · mới, `inline` | 台 | 台湾 | 台湾 | 台湾地 | 台湾地方 | …议会 | …接 | …接连 |

⇒ `inline-block` là một **hồi quy** *(kẹt hẳn)*; `inline` **nhanh hơn cấu trúc cũ** ở cả hai
mức chi tiết. Kiểu **chuyển đổi**: `word` ×1/×2/×3 ⇒ `台湾` / `台湾地方` / `台湾地方议会`;
`char` ×1/×3/×5 ⇒ `台` / `台` / `台湾`. **Mọi** truy vấn phát ra từ đường bàn phím đều là ký
tự nguồn, **0** lượt rò `U+2060`.

**Điểm dừng `Tab`:** vẫn **đúng hai** (`.hv-switch` và `.hv-parallel`) — không `<span>`/
`<ruby>` mới nào mang `tabindex` (§KHÔNG-LÀM ③).

#### AC3 — đường lui khi engine thiếu `Intl.Segmenter`

| đo | kết quả |
|---|---|
| `wordStartOffsets('台湾地方')` khi thiếu API | `{0,1,2,3}` — **mọi ký tự là một từ** ✅ |
| `console.error` nêu đích danh | **1** lần ✅ |
| gọi lần thứ hai | **không** kêu lại *(instance cache)* ✅ |
| có ném không | **không** ✅ |

#### AC9 — trần render, đo lại

| ký tự Hán | tách từ | dựng DOM · **mới** (một ruby/TỪ) | node | dựng DOM · **cũ** (một ruby/ký tự) | node | tỉ lệ node |
|---|---|---|---|---|---|---|
| 5.000 | 3,8 ms | **10,7 ms** | 3.502 | 15,2 ms | 5.459 | 0,642 |
| 50.000 | 18,7 ms | **92,6 ms** | 34.714 | 133,7 ms | 54.113 | 0,642 |

⇒ node **−36 %**, chi phí dựng **−31 %**, tách từ **+18,7 ms** một lần mỗi Chương ở mức trần.
**Giữ 50.000** — xem Change Log để biết vì sao không nâng.

#### AC11 — ICU trên văn xuôi TIỂU THUYẾT thật (danh sách ca SAI, không một lời khen)

Bốn đoạn mở đầu của bốn bộ tiểu thuyết cổ điển *(công hữu)*. Bảng đầy đủ 10 ca sai ở
`deferred-work.md` §1-18b; trích lớp sai lớn nhất — **tên riêng bị xé**: `傲/來/國` *(nước Ngạo
Lai)* · `姑/蘇` · `閶/門` · `大/宋`. Lớp thứ hai — **từ hiện đại đè nghĩa cổ**: `周末` *(ICU đọc
là "cuối tuần"; ở đây là 周 + 末 = cuối đời Chu)* · `正當`. Lớp thứ ba — **ghép sai qua ranh
giới**: `有一` · `中有` · `二等`.

#### Cổng — chín lệnh DoD

| lệnh | trước story | sau story |
|---|---|---|
| `cargo test` | 232 xanh | **232 xanh** |
| `npm run build` | exit 0 | **exit 0** |
| `check:tokens` · `check:i18n` · `check:commands` · `check:layout` · `check:deps` · `check:dict-manifest` · `check:scope` | 7/7 exit 0 | **7/7 exit 0** |

**Đỏ-rồi-xanh, hai cổng bị đụng** *(mã thoát là phán quyết)*:

| ca | cổng | exit |
|---|---|---|
| gỡ dấu `aura-allow-text` của `WORD_SEPARATOR` | `check:i18n` | **1** — `SourceHanViet.vue:620:10 — {{WORD_SEPARATOR}} không phải t()/tError()` |
| ↳ đối chứng âm: đặt lại | `check:i18n` | **0** |
| thêm `window.requestIdleCallback` vào `wordBoundary.ts` | `check:layout` | **1** — `wordBoundary.ts:79 — window.requestIdleCallback KHÔNG có trong danh sách cho phép` |
| ↳ đối chứng âm: gỡ ra | `check:layout` | **0** |

Ca thứ hai cũng là **bằng chứng tệp mới NẰM TRONG tầm quét** của `check:layout` — nếu không,
nó đã im lặng xanh.

### Completion Notes List

**Đã làm:** double-click ở tab Hán Việt nay chọn **cả cụm từ**, giống hệt tab nguyên văn, ở
**cả hai** kiểu xem. Ranh giới từ đi qua `Intl.Segmenter('zh')` — **cùng bộ tách từ ICU** mà
trình duyệt dùng cho double-click trên văn bản thuần, nên AC2 đúng **theo cấu trúc** chứ không
nhờ trùng hợp.

**Ba ranh giới không chạm, đếm lại:** `matchMedia` **0** · `window.innerWidth` **0** *(cả hai
chỉ còn trong chú thích — cổng `check:layout` che chú thích, và nó xanh)* · phụ thuộc mới
**0** *(`package.json` · `package-lock.json` · `Cargo.toml` · `Cargo.lock` — **không một dòng
đổi**)*. `core/matching/**` **không một dòng đổi** (AC4). `src/commands/**` **không** chạm
`wordBoundary.ts` lẫn `Intl.Segmenter` (Bẫy 7).

**`ALLOWED_GLOBAL_MEMBERS` — thêm 0 mục.** `Intl.Segmenter` không phải thành viên của
`window`/`document` nên không rơi vào tầm của Kiểm C; `document.createRange` mà
[`overlapsRange`] dùng **đã có sẵn** từ AC11/1.18. ⚠️ **Đính chính một số trong §Dev Notes của
story:** bảng đó ghi `ALLOWED_GLOBAL_MEMBERS` = **11**; số thật đếm lại là **12**
*(`check-layout.mjs:394`)*. Con số đã sai **từ trước** story này; story không thêm mục nào.

**Sàn:** không hằng `*_FLOOR` nào bị vượt. `src/**` đi từ 42 → **43** tệp *(`FILE_FLOOR` 34,
`check-layout.mjs` 32)*; quần thể component 39 → **40** *(`COMPONENT_FILE_FLOOR` 32)*; số tệp
`.vue` **không đổi** (13, `VUE_FLOOR` 11); `SELECTION_SURFACE_FLOOR` **5** giữ nguyên và
`check:commands` xanh. Không sàn nào cần nâng.

**Hai lệch có chủ ý so với chữ của story** — cả hai giữ nguyên **ý**, và cả hai có lý do ghi
trong Change Log + doc-comment tại chỗ: ① `<rt>` ở kiểu song song nối âm bằng **dấu cách**,
không `U+2060`; ② `.hv-unit` dùng `display: **inline**`, không `inline-block` *(số đo AC7 ép —
`inline-block` làm mở rộng theo TỪ bằng bàn phím **kẹt hẳn**)*.

**Một lỗi bàn đo bắt được, không suy luận nào bắt được:** `intersectsNode` một mình cho ra
**thừa một ký tự** khi vùng chọn dừng đúng ở biên node kế tiếp. Sửa bằng cách đo ranh giới ở
**mức text node**. Đây đúng lớp lỗi mà lượt review 1.18 đã bắt hai lần ở nhánh `text` (Bẫy 2) —
lần này nó tái phát ở một chỗ **mới** mà story không dự đoán: phép thử *giao nhau*, không phép
*cắt*.

---

#### 🔴 HAI THỨ **KHÔNG** NGHIỆM THU ĐƯỢC — nói thẳng, không đánh dấu đạt (AC11)

**① `Intl.Segmenter` trên WKWebView: CHƯA ĐO.** Toàn bộ số đo của story chạy trên **Chromium**.
Không dựng được bản Tauri thật trong phiên này. Ba thứ còn treo: API có mặt hay không · ranh
giới ICU của WebKit có **trùng** Chromium không *(nếu lệch, hai nền tảng chọn hai cụm khác nhau
cho cùng một cú double-click, và **không cổng nào bắt**)* · `Selection.modify(...,'word')` trên
`.hv-unit` `display: inline` có chạy như đo được ở Chromium không. Ghi vào `deferred-work.md`
§1-18b, cùng món nợ hai nền tảng mà 1.6/1.14/1.16/1.17/1.18 đang giữ.

**② Bàn đo CHÉP DOM, không MOUNT component thật.** Nó không thể mount — component cần cầu IPC
Tauri cho `hanVietByChar`, và dự án **cố ý** không có bộ chạy test frontend (NFR15, Ice chốt ở
1.5). Nghĩa là một lượt sửa template sau này có thể làm bàn đo và sản phẩm **lệch nhau mà không
cổng nào đỏ**. Đã ghi thành món nợ ở `deferred-work.md` §1-18b. Trang bàn đo **không** được
commit — nói ra ở đây để lượt review biết nó tồn tại và biết nó không nằm trong repo.

**③ Chất lượng tách từ trên tiểu thuyết: ĐO RỒI, và nó SAI ở một tỉ lệ có thật.** 10 ca sai cụ
thể đã liệt kê *(xem §Debug Log và `deferred-work.md`)*. **Không** sửa ở story này, có lý do
ghi thành chữ: một lượt cắt sai ở đây chỉ khiến người dùng **kéo chọn lại**, không làm lệch một
điểm khớp nào — khác hẳn lý do `mockups/tm-fuzzy-match.html:267-269` từ chối tách từ cho **TM**.

#### Ghi cho story xuôi dòng

**Story 3.4 KHÔNG bị chặn**, nhưng nó phải **tự cắt `.hv-unit`**: ranh giới thuật ngữ do
`Matcher` (Rust) trả về **không nhất thiết trùng** ranh giới TỪ của ICU — một thuật ngữ có thể
phủ *một phần* một `.hv-unit` hoặc *bắc cầu* hai cái. Mệnh đề để lại: **ranh giới của `Matcher`
thắng ranh giới của ICU**; ICU chỉ quyết *"double-click phủ tới đâu"*. Và khi 3.4 tách node, nó
phải giữ bất biến mà `resolveSwitch()` đứng lên: `host.children[i]` ứng **một-một** với
`segments.value[i]`. Chi tiết ở `deferred-work.md` §1-18b.

**Hai chú thích đã sửa vì chúng THÀNH SAI** *(Bẫy 10)*: `sourcePanelState.ts:84` *(phép đo
Playwright chạy trên DOM của 1.16 — một `.hv-unit` mỗi **ký tự**)* và `:102` *(mệnh đề "mỗi lần
xuất hiện sinh một `.hv-unit` riêng" — nay một `.hv-unit` mang cả **TỪ**)*.
`hanCharOccurrenceCount` **giữ nguyên logic**: nó *quá* ước lượng chi phí (0,642 node/lần xuất
hiện), không *dưới* ước lượng, nên trần vẫn giữ đúng vai chặn — đổi nó thành phép đếm TỪ sẽ bắt
chạy `Intl.Segmenter` trên toàn Chương chỉ để quyết định có cho phép kiểu song song hay không.

**Một chú thích đã sai TỪ TRƯỚC story này, cũng sửa luôn:** `src/panels/README.md` nói
`resolveParallel` *"đọc node `.hv-char`"* — lớp đó đã bị gỡ ở lượt vá `<ruby>` 2026-08-07.

**Banner đầu `SourceHanViet.vue` đã VIẾT LẠI**: tuyên bố cũ *"MỘT NODE CHO MỖI KÝ TỰ là điều
kiện tiên quyết của 1.18/3.4"* bị story này **phá có chủ đích**, và banner nay nói thẳng điều
đó cùng hệ quả *(nửa từ trở nên khả thi ⇒ mọi đường phân giải phải CẮT)*.

### File List

| tệp | trạng thái |
|---|---|
| `src/panels/wordBoundary.ts` | **NEW** — `wordStartOffsets()` · `WORD_JOINER`; doc-comment mang mệnh đề AD-17 (AC4) |
| `src/panels/SourceHanViet.vue` | UPDATE — `Segment.han` theo TỪ · `buildSegments` · `switchLeads` *(thay `switchView`)* · `readingLine` · `overlapsRange` · `sliceTextNode` · `resolveParallel` · `resolveSwitch` *(thay nhánh `switch` của `resolveSelection`)* · `onCopy` · template hai kiểu xem · CSS khe hở + `.hv-unit` `display: inline` · banner viết lại |
| `src/panels/sourcePanelState.ts` | UPDATE — **chỉ hai doc-comment** (`:84`, `:102`); `PARALLEL_VIEW_RENDER_CEILING` **giữ 50.000**, logic không đổi |
| `src/panels/README.md` | UPDATE — hàng 1.18b · §Tách từ tiếng Trung · sửa chú thích `.hv-char` đã sai từ trước |
| `_bmad-output/implementation-artifacts/deferred-work.md` | UPDATE — đóng §nghiệm thu tay tab Hán Việt · mở §1-18b với **bốn** món nợ mới |
| `_bmad-output/implementation-artifacts/sprint-status.yaml` | UPDATE — `1-18b…: ready-for-dev → review` |
| `_bmad-output/implementation-artifacts/1-18b-tach-tu-tieng-trung-tab-han-viet.md` | UPDATE — Change Log · Tasks · Dev Agent Record · File List · Status |

**KHÔNG đổi một dòng nào** *(kiểm bằng `git status`)*: `package.json` · `package-lock.json` ·
`src-tauri/Cargo.toml` · `src-tauri/Cargo.lock` · `src-tauri/src/core/matching/**` ·
`src/panels/selectionContract.ts` · `src/panels/lookupPanelState.ts` · `src/config/dict.ts` ·
`src/commands/**` · `scripts/*.mjs` *(không sàn nào bị vượt)* · `src/tokens/tokens.json`
*(không token mới — khe hở đi bằng `--space-unit` sẵn có)*.
