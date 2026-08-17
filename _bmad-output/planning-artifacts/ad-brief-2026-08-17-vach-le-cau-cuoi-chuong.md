# Hồ sơ bàn giao cho Winston — một `AD` mới về **trạng thái đọc được của câu đang có con trỏ**

**Ngày:** 2026-08-17 · **Người bàn giao:** lượt code review ba tầng của Story 2.10 · **Người nhận:** Winston (architect)
**Nguồn gốc:** nửa còn hở của món nợ *"đi đâu khi hết Chương"* (`deferred-work.md:2837-2878`), đóng
🟡 ở Story 2.10 theo Quyết định #6 đường (c) — Ice ký 2026-08-18
**Trạng thái Story 2.10:** chín AC đều có mã cài đúng chữ; cửa chặn này **không** chặn AC nào của
2.10. Nó chặn **vế thị giác của AC1 Story 2.5**, vốn đã `done` — xem §7.
**Baseline cây nguồn:** `ccffa23` + cây làm việc của 2.10 · `cargo test --locked` **401/0/5** ·
vitest **224/224, 20 tệp** · 9 cổng đọc-tệp xanh · `COMMAND_FLOOR` **41** / 49 command ·
`grep -c "^### AD-"` = **47**

⚠️ **Số `AD` kế tiếp là 48, nhưng hồ sơ này KHÔNG được đặt số.** Hồ sơ
`ad-brief-2026-08-17-mo-hinh-hoan-tac.md` cũng đang xếp hàng và cũng ghi *"kế tiếp là 48"*. Hai hồ
sơ, một con số — số thật do Winston gán lúc viết, theo thứ tự viết. Đừng ai chép số 48 từ một trong
hai tệp.

---

## 1. Vì sao hồ sơ này tồn tại

**AC1 của Story 2.5** đòi hai vế: *"trạng thái chuyển sang đã xác nhận **và vạch lề chuyển
`confirmed`**"*. Vế thứ nhất đạt; vế thứ hai **hụt ở đúng câu cuối mỗi Chương**, và Story 2.10 —
chủ được giao — đóng nó **một nửa**, bằng thông tin thay vì bằng màu.

Cơ chế, đo từ nguồn:

| Phép đo | Kết quả |
|---|---|
| `resolveSegmentRule` (`src/panels/editorSegments.ts:162-176`) | `if (input.hasCaret) return 'primary'` đứng **trước** `if (input.isConfirmed) return 'confirmed'` |
| Thứ tự đó là gì | 🔴 Một **quyết định có chữ ký** kèm lý do tại chỗ (`:124-126`): *"`primary` thắng `confirmed` — nó là mệnh đề về **hiện tại**, trạng thái đã xác nhận là mệnh đề về quá khứ, và vạch chỉ có một chỗ để nói"* |
| Đường thoát của lượt xác nhận thường | Dời con trỏ sang câu kế ⇒ `hasCaret` thành `false` ⇒ vạch tự chuyển `confirmed` |
| Ở câu **cuối** Chương | `next.at(index + 1)` là `undefined` (`editorPanelState.ts:921`) ⇒ không dời được ⇒ `hasCaret` **ở lại `true`** ⇒ vạch **ở lại `primary`** |
| Tần suất | **Đúng một lần mỗi Chương**, ở đúng câu cuối. Không phải ca hiếm |
| `segment.status` trên đĩa | **Đúng** — `'confirmed'`. Chỉ vế đọc được ở giao diện hụt |

`project-context.md` §Story và spec viết bằng chữ: *"Đổi một bất biến kiến trúc là một `AD` MỚI,
không phải một dòng mã."* ⇒ Story 2.10 dừng, `AD` viết trước.

---

## 2. 🔴 Khoảng hở RỘNG HƠN sổ nợ mô tả — một phép đo của lượt code review

Sổ nợ (`deferred-work.md:2854-2856`) gọi phần còn hở là *"vạch lề **vẫn `primary`**, không
`confirmed`… Đóng bằng **thông tin**, không bằng **màu**."* **Vế "bằng màu" đó không đầy đủ, và
khác biệt này đổi hẳn cân nhắc.**

Đo 2026-08-17 trên `GridPanel.vue:244-251`:

```ts
const STATE_LABEL_KEYS: Readonly<Record<SegmentRuleValue, string>> = {
  confirmed: 'panel.grid.state_confirmed',   // "đã xác nhận"
  primary:   'panel.grid.state_editing',     // "đang sửa"
  …
}
```

Cột nhãn trạng thái **khoá theo cùng một `SegmentRuleValue`** mà `resolveSegmentRule` trả về. ⇒ Ở
câu cuối Chương sau một lượt `⌘Enter` thành công, cột ⑤ đọc ra **"đang sửa"**, không *"đã xác
nhận"* — cùng một phép ưu tiên, nhân ra **hai** kênh.

🔴 **Và cột ⑤ chính là kênh khả năng tiếp cận.** Doc-comment của nó (`:240-242`) viết thẳng:

> *"Đây là cột thứ năm, và nó là **kênh đọc được** cho đúng thứ vạch lề nói bằng màu. Một người
> dùng đọc bằng bàn phím hoặc bằng trình đọc màn hình không có vạch. **Cột này là lý do vạch được
> phép `aria-hidden`.**"*

Và vạch **thật sự** mang `aria-hidden="true"` (`:1368`).

⇒ Hệ quả phải nói ra: với một người dùng trình đọc màn hình, hàng cuối Chương **không có kênh nào**
nói *"đã xác nhận"*. Vạch bị `aria-hidden`; cột ⑤ nói *"đang sửa"*. Câu duy nhất còn nói đúng là
thông báo `'confirmed-last'` trên thanh trạng thái — một `role="status"` **thoáng qua**, không phải
một thuộc tính của hàng, và nó bị lượt thao tác kế tiếp dọn đi.

⚠️ Điều này đưa cửa chặn từ *"một sắc màu lệch"* sang **AD-34 §2 + NFR17** (`prd.md:903`, *"mọi
thao tác hoàn toàn bằng bàn phím"*). Đó là hạng khác, và nó là lý do hồ sơ này không nên đóng bằng
một lượt chọn token.

---

## 3. Vì sao nó không giải được bằng một lượt "tiện tay"

### 3.1 — Đảo thứ tự hai nhánh là lật một quyết định có chữ ký, và nó có nạn nhân

`resolveSegmentRule:124-126` khai lý do bằng chữ. Đảo thành `confirmed` thắng `primary` sửa được ca
câu cuối, nhưng nó làm **mọi** câu đã xác nhận mất vạch `primary` khi con trỏ vào đó ⇒ mâu thuẫn
`EXPERIENCE.md:198` mục 3: *"**Tiêu điểm luôn nhìn thấy** — vạch dọc `primary` ở mép trái panel"*.
Và ở cột ⑤ nó còn nặng hơn: một câu đang gõ sẽ đọc ra *"đã xác nhận"*.

⇒ Đây không phải *"chọn ưu tiên nào đúng hơn"*. **Cả hai thứ tự đều nói dối ở một ca**, vì một kênh
đang chở **hai** mệnh đề độc lập.

### 3.2 — Bảng giá trị vạch là một tài nguyên ĐÃ TIÊU HẾT, cưỡng chế bằng cổng

`SEGMENT_RULE_VALUES` (`editorSegments.ts:69-76`) có **đúng sáu** giá trị, và
`scripts/check-commands.mjs` Kiểm I **đếm mảng này và đỏ ở giá trị thứ bảy**. Cổng còn đối chiếu
**hai chiều** với các khối `.rule-<giá trị>` trong `GridPanel.vue`.

`EXPERIENCE.md:99` gọi bộ giá trị vạch là *"tài nguyên hữu hạn đã tiêu hết"*, và lượt xin giá trị
thứ **sáu** (Story 2.5b) chỉ được cấp vì nó **lấp một hàng vốn đã thiếu**, không phải một trạng thái
mới xin chỗ *(doc-comment `:158-162`)*.

⇒ Một giá trị thứ bảy kiểu `'primary-confirmed'` là một lượt xin **kênh thị giác mới**, đúng thứ hai
tiền lệ đã từ chối. Nó cũng làm cổng đỏ theo thiết kế, tức nó **phải** đi qua một `AD`.

### 3.3 — Nhả caret đã được đo và bị loại, nhưng KHÔNG vì lý do story dự đoán

Story 2.10 lo `setEditorCaret(null)` *"bỏ rơi bộ đệm gõ"*. Vế đó **bị bác bằng phép đo**: bước ① của
`confirmCurrentSegmentUnguarded` đã `flushEditorBeforeDiscreteWrite()` trước lượt IPC, nên tập chờ
sạch **theo cấu tạo** tại điểm gọi.

Cái chặn thật lộ ra khi đọc `onSelectionChange` (`GridPanel.vue:875-882`): đường đó dựng trạng thái
`caretSegmentId === null` **trong khi DOM focus vẫn nằm trong ô** — hai nguồn sự thật về *"người
dùng đang ở đâu"* nói ngược nhau, **không cổng nào canh câu đó**, và `onSelectionChange` đặt lại
`id` ở lượt dịch caret kế tiếp nên hiệu lực thị giác chỉ **tạm**.

⇒ Nó mua một vế thị giác **tạm thời** bằng một trạng thái lệch **thường trực**. Và nó đụng AD-34 §2
*(focus không được rơi về `body`)* mà chưa ai trả lời caret rơi về đâu.

---

## 4. Câu hỏi thứ nhất — một kênh chở được mấy mệnh đề?

Đây là câu hỏi gốc, và bốn đường dưới đây khác nhau ở **kiến trúc**, không ở token.

| | **(A) Tách hai khái niệm caret** | **(B) Kênh thứ hai cho hàng** | **(C) Ưu tiên có điều kiện** | **(D) Nhận vĩnh viễn** |
|---|---|---|---|---|
| Việc làm | `hasCaret` tách thành *"DOM focus đang ở đây"* và *"đây là câu người dùng đang làm việc"*; lượt ký câu cuối hạ vế thứ hai | Vạch giữ `primary`; thêm **một** kênh khác cho vế *"đã ký"* *(ví dụ cột ⑤ đọc hai mệnh đề, hoặc một dấu ở cột số câu)* | `resolveSegmentRule` nhận thêm dữ kiện *"vừa được ký trong lượt này"* và cho `confirmed` thắng **chỉ** ở ca đó | Vế thị giác của AC1 khai là **không đạt được** ở câu cuối; thanh trạng thái là câu trả lời cuối cùng |
| `SEGMENT_RULE_VALUES` | Không đổi — vẫn sáu | Không đổi | Không đổi | Không đổi |
| Cổng Kiểm I | Xanh | Xanh | Xanh | Xanh |
| Cột ⑤ *(kênh a11y)* | **Sửa được** — nó khoá theo cùng giá trị | **Sửa được** nếu kênh mới là chữ | **Sửa được** | 🔴 **Vẫn nói *"đang sửa"*** |
| Hàm thuần còn thuần? | Có — thêm một trường vào `SegmentRuleInput` | Có | ⚠️ **Không hoàn toàn**: nó phải biết *"lượt này"*, tức một mệnh đề về **thời gian**, không về hàng | Có |
| Cái mất | Một khái niệm mới phải định nghĩa ở đâu cũng đọc được; hai cờ dễ lệch nhau | Một kênh thị giác/chữ mới — đúng thứ `EXPERIENCE.md:99` gọi là đã tiêu hết | 🔴 Một hàm phân giải **thuần** nay phụ thuộc lịch sử ⇒ `check:commands` `import()` nó bằng Node trần để chạy phép kiểm **hành vi**; một dữ kiện thời gian làm phép kiểm ấy phải dựng cả một dòng thời gian | NFR17 hụt ở một hàng mỗi Chương, **vĩnh viễn**, và AC1 của một story `done` ở lại sai |

### 🔴 Điểm căng, và nó không giải được bằng cách chọn "đường an toàn"

**Đường (A) là đường sổ nợ đã gợi**, và nó đụng đúng chỗ đau: hôm nay `hasCaret` **là** DOM focus
*(`caretSegmentId === segment.id`)*, và cả `onSelectionChange` lẫn watcher đường lệnh đều ghi vào
một biến duy nhất. Tách nó thành hai nghĩa là dựng **hai** nguồn sự thật về vị trí người dùng — đúng
thứ §3.3 vừa loại đường nhả caret vì nó gây ra. Winston phải phán định: hai cờ có **một** chủ ghi
duy nhất được không, hay đây là cùng cái bẫy mang tên khác?

**Đường (D) là đường đang chạy hôm nay**, và nó không trung tính: `project-context.md` §Story viết
*"Năng lực chưa dựng ≠ lệch spec — đừng sửa `epics.md` cho khớp mã đã viết"*. Chọn (D) **là** một
lượt hạ AC, nên nó phải là một quyết định viết ra, không một món nợ để lâu thành mặc định.

⚠️ **Cả bốn đường đều làm một lời hứa nào đó kém đúng đi.** Đó là lý do nó là một `AD`.

---

## 5. Câu hỏi thứ hai — mệnh đề nào thuộc HÀNG, mệnh đề nào thuộc THANH TRẠNG THÁI?

Story 2.10 đóng nửa món nợ bằng một câu trên thanh trạng thái. Câu đó là một `role="status"`
**thoáng qua**: nó bị lượt thao tác kế tiếp dọn *(và lượt code review này vừa tìm ra rằng một lượt
điều hướng thành công **không** dọn nó — một patch riêng)*.

⇒ Câu hỏi kiến trúc: *"câu này đã được ký"* là một **thuộc tính của hàng** hay một **sự kiện vừa
xảy ra**?

- Nếu là **thuộc tính**: nó phải đọc được từ hàng ở mọi lúc, kể cả sau khi người dùng đi vòng rồi
  quay lại — và (D) không đủ.
- Nếu là **sự kiện**: thanh trạng thái là đúng chỗ, nhưng khi ấy phải khai bằng chữ rằng cột ⑤
  **không** chở mệnh đề đó ở ca câu cuối, và ghi nó vào bảng khả năng tiếp cận thay vì để nó là một
  khoảng hở không ai khai.

🔴 Không mức nào tự trả lời câu này, và nó quyết định cả bốn đường ở §4.

---

## 6. Ràng buộc cứng — `AD` phải đứng vừa trong bộ này

| Ràng buộc | Nguồn | Nó cấm gì |
|---|---|---|
| `primary` thắng `confirmed`, sáu nhánh theo thứ tự đã ký | `editorSegments.ts:120-133` | Một lượt đảo thứ tự không kèm `AD` |
| Đúng **sáu** giá trị vạch, cổng đỏ ở giá trị thứ bảy | `SEGMENT_RULE_VALUES` + `check-commands.mjs` Kiểm I | Một giá trị `'primary-confirmed'` |
| Bộ giá trị vạch là *"tài nguyên hữu hạn đã tiêu hết"* | `EXPERIENCE.md:99` | Một kênh thị giác mới xin chỗ mà không phải lấp một hàng thiếu |
| Vạch mang `aria-hidden`; cột ⑤ **là** kênh đọc được | `GridPanel.vue:240-242` · `:1368` | Một lời giải chỉ sửa màu mà để cột ⑤ nói sai |
| Tiêu điểm **luôn nhìn thấy** | `EXPERIENCE.md:198` mục 3 · **AD-34 §2** | `confirmed` thắng `primary` vô điều kiện |
| Focus không rơi về `body`; mỗi panel khai điểm vào focus | **AD-34 §2** `SPINE:412` | Nhả caret mà không khai caret đi đâu |
| Mọi thao tác hoàn toàn bằng bàn phím | **NFR17** `prd.md:903` | Nhận vĩnh viễn một hàng không có kênh a11y nào nói *"đã ký"* |
| `editorSegments.ts` là **module thuần**, `import()` được bằng Node trần | `editorSegments.ts:26-31` | Một dữ kiện thời gian/lịch sử đi vào `resolveSegmentRule` *(đường (C))* |
| `segment.id` bất biến; dữ liệu gắn theo `id` không theo vị trí | **AD-3** `SPINE:89-93` | Một lời giải dựa vào *"hàng cuối"* như một danh tính |
| Ranh giới segment đóng băng lúc nhập | **AD-4** | Một lượt "thêm một hàng trống ở cuối Chương" cho caret dời tới |

⚠️ **Một cái bẫy dễ bỏ sót:** đường (D) *(nhận vĩnh viễn)* trông như *"không đổi gì"* nên nó dễ được
chọn bằng cách **không chọn**. Nhưng nó là đường duy nhất trong bốn đường để cột ⑤ — kênh khả năng
tiếp cận — nói sai, nên nó là đường **đắt nhất** theo NFR17, không rẻ nhất.

---

## 7. Phạm vi — cửa chặn này chặn CÁI GÌ, và không chặn cái gì

| Thứ | Phụ thuộc `AD` này? |
|---|---|
| Chín AC của **Story 2.10** | **Không.** Cả chín có mã cài đúng chữ; AC7 *("hành vi ở biên rõ ràng và không sập")* đạt bằng Quyết định #5(a) |
| Vế **`segment.status`** của AC1 Story 2.5 | **Không** — đúng trên đĩa từ 2.5 |
| Vế **vạch lề `confirmed`** của AC1 Story 2.5 | 🔴 **CÓ** |
| Vế **cột ⑤ nói *"đã xác nhận"*** ở câu cuối *(chưa ai khai là một vế)* | 🔴 **CÓ** — và §2 là chỗ nó được khai lần đầu |
| Bộ e2e / bàn đo của 2.10 | **Không** |

⚠️ **Ca test cố ý KHÔNG kiểm vạch lề** ở `tests/frontend/editorNavNotice.test.ts` là một lựa chọn
đúng và phải giữ: vạch **vẫn** `primary`, và một ca khẳng định ngược lại sẽ là một lời hứa sai nằm
trong chính bộ test. `AD` này đóng xong thì ca đó mới được đổi.

⚠️ **Năng lực chưa dựng ≠ lệch spec.** AC1 của Story 2.5 **không sai** vì đường đi chưa tới nó.
**Không sửa `epics.md`/`prd.md`** cho khớp mã đã viết.

---

## 8. Điều kiện nghiệm thu của `AD` này

1. Khuôn **Binds / Prevents / Rule** như 47 `AD` kia; `lint_spine.py` chạy sạch.
2. Trả lời **cả hai** câu hỏi: đường (A)/(B)/(C)/(D) ở §4, **và** *"thuộc tính của hàng hay sự
   kiện vừa xảy ra"* ở §5.
3. Nói rõ thứ tự sáu nhánh của `resolveSegmentRule` **đổi gì** — hoặc khai bằng chữ rằng nó
   **không đổi một chữ** *(khuôn AD-47 đã dùng với AD-31/AD-5)*.
4. Trả lời vế **cột ⑤** tường minh. Một `AD` chỉ nói về màu vạch là một `AD` bỏ sót đúng kênh mà
   NFR17 dựa vào.
5. Nếu đường ký đòi một giá trị vạch thứ bảy ⇒ nói rõ nó lấp **hàng thiếu** nào, và ai sửa
   `EXPERIENCE.md:99` cùng cổng Kiểm I.
6. Nếu đường ký tách `hasCaret` thành hai khái niệm ⇒ nói rõ **một chủ ghi duy nhất** của cặp cờ đó
   là ai, vì §3.3 vừa loại một đường khác vì đúng lỗi hai-nguồn-sự-thật.
7. **Không** đòi một bước di trú: `segment.status` đã đủ. Bước di trú kế tiếp là **12** và `AD` này
   **không** được tiêu nó.

---

## 9. Thứ hồ sơ này **KHÔNG** làm

Không đề xuất một đường. Bốn đường ở §4 được trình **cùng cái giá của chúng**, và cái giá nằm ở một
lời hứa của spine — đó là loại quyết định `project-context.md` giao cho Ice và cho một `AD`, không
cho một lượt dev và cũng không cho một lượt code review.

⚠️ Hồ sơ này cũng **không** soạn `AD`. Luật kho: *"`AD` mới giao Winston, dev không tự soạn."*
Thứ mới ở đây so với sổ nợ là **§2** — một phép đo cho thấy khoảng hở chạm vào kênh khả năng tiếp
cận, không chỉ vào màu. Phần còn lại là gom về một chỗ những gì đã rải trong `deferred-work.md`,
`editorSegments.ts` và tệp story.
