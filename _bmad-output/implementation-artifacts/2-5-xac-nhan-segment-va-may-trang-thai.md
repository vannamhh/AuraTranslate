---
baseline_commit: 8245a171640c2a3b9a23831fd800bd595ce81026
---
# Story 2.5: Xác nhận segment và máy trạng thái

Status: done

**Covers:** FR24 · AD-31 (máy trạng thái segment) · vế còn treo của AC3 Story 2.3 (AD-35 điểm (c))

<!-- Note: Validation is optional. Run validate-create-story for quality check before dev-story. -->

## Story

As a người dịch,
I want đánh dấu một câu là đạt chuẩn của mình và thấy nó đổi màu ngay,
so that tôi biết mình đang ở đâu trong một Chương dài.

---

## Điều kiện khởi hành

🔴 **Đọc mục này TRƯỚC Task 0.** Hai story trước còn `in-progress`, và story này dựng thẳng lên bề mặt của chúng. Ice đã chốt 2026-08-14: **cứ dựng 2.5**, ghi điều kiện khởi hành, không tự chấm đạt cho món nào của 2.3/2.4.

| Món treo | Có chặn 2.5 không | Vì sao |
| --- | --- | --- |
| **2.3 — ca gõ đầu tiên vào câu CHƯA DỊCH** (`<span>` rỗng 0px, `execCommand('insertText')` trả `false`; `test:e2e` 1 xanh/1 đỏ tại `editor-typing-flush.e2e.mjs:133`) | **KHÔNG chặn** | Task 1.0 của Story 2.4 đã **mở** món này bằng tay ngày 2026-08-13: Ice gõ, chữ hạ cánh vào câu chưa dịch (`2-4…md:1280-1294`). Cái còn đỏ là **bộ đo e2e**, không phải sản phẩm — và nguyên nhân chưa được đặt tên. ⇒ 2.5 **được đi tiếp**, nhưng **không** được viết một ca e2e mới đặt caret vào câu rỗng rồi đọc màu đỏ của nó thành *"2.5 hỏng"*. Xem Task 9. |
| **2.4 — chưa tiêm được** `bench.js` **vào webview bản release** (bốn giả thuyết đã bị bác) | **KHÔNG chặn** | 2.4 giao ra **số đo**, không giao bề mặt. 2.5 không đọc một hằng nào của 2.4. |
| **2.4 — NFR18 tại** `wal_threshold_bytes = 4 MiB` **KHÔNG ĐẠT** (1/20 vượt thật, tại 6,24 s) | **KHÔNG chặn, nhưng ràng buộc một lựa chọn** | Vì ngưỡng WAL chưa chốt, 2.5 **không được** dựng thêm bất kỳ đường ghi nào chạy theo nhịp. Lệnh xác nhận là **một lượt ghi rời rạc do người dùng bấm** — nó không thêm tải nền, nên nó trung lập với phép dò đang treo. Giữ nguyên tính chất đó. |
| **2.4 — NFR2 chưa có số nào** | **KHÔNG chặn** | 2.5 không được **tự chấm** NFR2 đạt. Nếu lượt xác nhận sinh một gai trễ, ghi số vào `deferred-work.md` với chủ là 2.4, đừng tự kết luận. |

⚠️ **Cây git phải sạch trước khi bắt đầu.** Đã kiểm 2026-08-14: sạch. Nếu lúc chạy có thay đổi lạ, commit riêng **trước**, và **hỏi Ice trước khi commit** — diff của một story phải đọc được một mình.

---

## Acceptance Criteria

### Nhóm A — nguyên văn từ `epics.md:2170-2204`

**AC1.** **Given** một segment · **When** người dùng xác nhận · **Then** trạng thái chuyển sang **đã xác nhận** và vạch lề chuyển `confirmed`.

> 🔴 **AC1 xung đột với mã đang chạy — Quyết định #1 của Task 0 phân xử nó.** `resolveSegmentRule` (`src/panels/editorSegments.ts:117-125`) cho `primary` **thắng** `confirmed`. Người dùng bấm xác nhận trên chính câu con trỏ đang đứng ⇒ vạch **vẫn** `primary`. Không được tự đổi thứ tự ưu tiên: nó là một quyết định 🔴 có lý do ghi tại chỗ.

**AC2.** **Given** một segment **chuyển sang** đã xác nhận · **When** xảy ra · **Then** tạo **đúng một** `SegmentVersion`.

**AC3.** **Given** một segment **đã xác nhận** · **When** người dùng sửa văn bản của nó · **Then** trạng thái **quay về chưa xác nhận** **And** không tạo `SegmentVersion`.

**AC4.** **Given** auto-save chạy · **When** xảy ra · **Then** trạng thái segment không đổi và không tạo `SegmentVersion`.

**AC5.** **Given** thao tác xác nhận · **When** gọi · **Then** qua một command đã đăng ký trong `CommandRegistry`, **gán phím được**.

**AC6.** **Given** người dùng tự dịch câu đó hay đang biên tập câu do người khác dịch · **When** xác nhận · **Then** ngữ nghĩa giống nhau — *"câu này đạt chuẩn của tôi"*.

### Nhóm B — suy ra từ bất biến kiến trúc và sổ nợ có chủ

Mỗi AC dưới đây trỏ về một nguồn đã tồn tại. Không AC nào là ý mới của lượt dựng story.

**AC7 — xác nhận là điểm (c) của hợp đồng flush.** Lượt xác nhận **flush văn bản đang gõ xuống Rust trước** khi ghi trạng thái, và flush chỉ coi là xong **sau khi đã vào WAL**. Xác nhận là **thao tác rời rạc ghi NGAY**, cấm định tuyến qua bộ đệm gõ 2 s/5 s.

> Nguồn: `ARCHITECTURE-SPINE.md:419-425` (AD-35, mệnh đề (c) và đoạn *"Thao tác rời rạc ghi ngay"*). Đóng vế còn treo của AC3 Story 2.3 — `deferred-work.md:2388-2391`.

**AC8 — lệnh xác nhận là một lệnh IPC MỚI,** `save_segment_targets` **không đổi một dòng.** Sau story này, `save_segment_targets` vẫn `UPDATE` **đúng hai cột** `target_text` + `updated_at` và không cột nào khác.

> Nguồn: doc-comment `src-tauri/src/commands/segment.rs:377-389`. Cổng: `src-tauri/tests/segment_contract.rs:1318 :: a_flush_touches_exactly_target_text_and_updated_at_and_nothing_else` — đọc **9 cột thô** bằng SQL; nó phải **vẫn xanh** sau khi cột `status` ra đời (tức số cột nó đọc phải được nâng có chủ ý, không phải nới lỏng phép kiểm). Nhét `status` vào câu `UPDATE` của auto-save là **phá AD-31 hàng 1** (AC4).

**AC9 — bước di trú mới đánh số 7, bằng một** `ALTER TABLE` **riêng.** Không sửa `SEGMENT_DDL` tại chỗ. Di trú **chỉ tiến**, chạy trong một giao dịch, sau khi đã sao lưu. Test cấm số 4 quay lại vẫn xanh.

> Đã kiểm bằng nguồn 2026-08-14: `PROJECT_MIGRATIONS` (`schema.rs:431-455`) hiện đích ở **6** — 1·2·3·**5**·6, số **4 cháy**. ⇒ **Số kế tiếp là 7.** ⚠️ Dòng `sprint-status.yaml:112-113` viết *"bước di trú kế tiếp phải đánh số 5"* — mệnh đề đó đúng ở thời điểm Story 2.1 và **đã hết đúng**; 5 và 6 đã tiêu. Cổng: `segment_contract.rs:472 :: the_project_migration_set_never_reuses_the_burned_number_four`. Khuôn để chép: `SEGMENT_TARGET_TEXT_DDL` (bước 6, Story 2.2). AD-30, `ARCHITECTURE-SPINE.md:362-366`.

**AC10 — nối nguồn dữ liệu thật, không dựng lại tầng hiển thị.** `segmentRuleInputOf` đọc `status` thật thay cho hằng `false`. `SEGMENT_RULE_VALUES` vẫn **đúng năm** giá trị — không thêm giá trị thứ sáu.

> Nguồn: `src/panels/editorSegments.ts:141-142` (*"Story 2.5 và Epic 7 sửa đúng hai dòng ở đây"*), `:51` (đúng năm giá trị là một mệnh đề nghiệm thu, AC12 của Story 2.2). Cổng: Kiểm I của `scripts/check-commands.mjs` đếm mảng và **đỏ ở giá trị thứ sáu**; Kiểm B của `scripts/check-tokens.mjs` đối chiếu hai chiều với các khối `.gmark.rule-*` trong `EditorPanel.vue`. Kênh thị giác kế tiếp là gạch chân lượn sóng (UX-DR22), **không** phải một màu vạch nữa.

**AC11 — mối nối cho FR117 và FR56 để lại mở, không bị chôn.** Chuyển tiếp sang đã xác nhận là **chỗ duy nhất** mà xuất xứ (FR117, Story 2.7) và cặp TM (FR56, Epic 7) sẽ được ghi. Story này **không cài** hai thứ đó, nhưng phải:
(a) giữ nguyên việc `editedText` (văn bản đang gõ) **tách rời** `segments` (bản lúc nạp) — mốc so sánh của FR117;
(b) đặt điểm ghi trạng thái ở một chỗ **gọi tên được**, kèm doc-comment chỉ đích danh hai story chủ.

> Nguồn: `ARCHITECTURE-SPINE.md:384-392` (bảng xuất xứ + hợp đồng phụ *"so văn bản đích hiện tại với bản lúc nạp segment, không dùng cờ dirty"*), `src/panels/editorPanelState.ts:161-165` (mệnh đề này đã được viết ra và phải giữ), `EXPERIENCE.md:294` (*"Cặp TM mới được ghi ngay tại chuyển tiếp đó (AD-31)"*).

**AC12 — máy trạng thái sống ở Rust.** Vue chỉ render vạch. Không quy tắc nghiệp vụ nào của AD-31 được cài lại ở TypeScript.

> Nguồn: AD-1, `ARCHITECTURE-SPINE.md:75-79` — ngoại lệ **duy nhất, tường minh** là văn bản đang gõ, không phải trạng thái.

**AC13 — xác nhận lại một segment đã xác nhận mà văn bản không đổi là VÔ HẠI.** Không tạo `SegmentVersion` thứ hai, không đổi `updated_at` của segment.

> Căn cứ văn bản: AC2 khoá vào **"chuyển sang"** đã xác nhận, và bảng AD-31 lập chỉ mục theo **sự kiện chuyển tiếp**. Một segment đã ở đích không chuyển tiếp. Nếu không có mệnh đề này, giữ phím xác nhận sẽ bơm lịch sử phiên bản đầy các bản sao y hệt và FR101 thành vô dụng — đúng hố số (1) mà AD-31 §Prevents nêu tên.

**AC14 — mọi lối từ chối phải PHÂN BIỆT ĐƯỢC, không rỗng im lặng.** Xác nhận một `segment.id` không tồn tại, hoặc một segment đã về hưu (`retired_at` khác `null`), trả một `IpcError` có `message_key` riêng. Không trả *"đã xong"* cho một lượt không ghi gì.

> Nguồn: `project-context.md` §*"Rỗng IM LẶNG bị cấm; rỗng CÓ LÝ DO thì không"*. Khuôn có sẵn: `SegmentUnknownIds => "err.segment.unknown_ids"` và ô lỗi có kiểu `BatchReject` (`commands/segment.rs`, Story 2.3 — *"không đoán lại từ chuỗi lỗi"*). ⚠️ Chưa đường nào cho segment về hưu (chủ: Story 2.8) — nhánh này là một **hàng rào viết trước**, và test của nó phải dựng trạng thái về hưu bằng SQL trực tiếp trong fixture, không bằng một đường sản phẩm chưa tồn tại.

**AC15 — chuỗi hiển thị và lỗi đi đúng đường.** Nhãn command khai `command.<id>` trong `vi.json`; `message_key` mới khai bằng `macro_rules! message_keys!`; chuỗi literal trong `src-tauri/src/**` viết** không dấu**.

> Cổng: `check:i18n` Kiểm A (chữ có dấu ở vị trí mã) · Kiểm B (`vi.json` phẳng, khoá chấm) · `ipc_contract.rs :: every_message_key_exists_in_vi_json`. Một biến thể quên thêm vào `ALL` cho một test **xanh giả** — khai bằng macro, không bằng danh sách song song.

**AC16 — nghiệm thu bằng cả bốn đường, mỗi mệnh đề đúng một chủ.** 11 cổng `npm` xanh · `npm run build` xanh · `npm run test` xanh · `cargo test --locked` xanh. Mọi phép kiểm mới phải chạy **đỏ-rồi-xanh** một lượt có ghi lại. Nếu thêm tệp vào `src/**` hoặc `src-tauri/src/**`, **xét lại sàn quần thể** của các cổng liên quan.

> Bốn đường và vai không chồng nhau: `project-context.md` §Testing Rules (AC25). Sàn hiện tại đã đo: `check-tokens.mjs` `FILE_FLOOR = 45` · `COMPONENT_FILE_FLOOR = 43`; `check-i18n.mjs` `RS_FLOOR = 36` · `VUE_FLOOR = 13`; `check-layout.mjs` `FILE_FLOOR = 43`; `check-commands.mjs` `COMMAND_FLOOR = 29` (hiện có 34 command thật ⇒ thêm một command **không** buộc nâng sàn, nhưng nhớ sàn là **cận dưới** nên tệp thừa không làm cổng đỏ, nó chỉ làm sàn vô nghĩa).

---

## Task 0 — Bảy quyết định phải chốt TRƯỚC khi viết dòng mã đầu tiên

🔴 **Khuôn bắt buộc, đã dùng ở cả bốn story trước của Epic 2:** mỗi quyết định nêu **đủ các đường**, kèm **đề xuất mặc định có lý do đo được**, rồi chờ Ice. **Ice là người chốt các quyết định mở** — nêu cả hai kèm số đo, đừng tự chọn rồi đi tiếp, và cũng đừng loại một phương án chỉ vì nó đắt.

### Quyết định #1 — 🔴 `primary` nuốt `confirmed`, nên AC1 mô tả một đổi màu KHÔNG xảy ra

**Sự kiện đo được (đọc mã, 2026-08-14).** `resolveSegmentRule` (`editorSegments.ts:117-125`) xếp: `ornament` \> `primary` \> `confirmed` \> `tm-rule` \> `none`. Lý do ghi tại chỗ (`:95-97`): *"*`primary`* thắng *`confirmed`* — *`DESIGN.md:380`* định nghĩa nó là 'đang sửa, con trỏ ở đây', tức một mệnh đề về hiện tại; trạng thái đã xác nhận là mệnh đề về quá khứ, và vạch chỉ có một chỗ để nói."*

⇒ Người dùng bấm xác nhận trên câu con trỏ đang đứng: `status` đổi trong database, nhưng **vạch không đổi màu**. AC1 (`epics.md:2182`) và KF-2 bước 2 (`EXPERIENCE.md:290` — *"bấm xác nhận. Vạch chuyển xanh ô liu."*) đều mô tả một phản hồi thị giác mà thứ tự ưu tiên hiện tại nuốt mất.

| Đường | Nội dung | Cái giá |
| --- | --- | --- |
| **(a)** ⭐ | Lượt xác nhận **dời con trỏ sang segment kế tiếp**. Câu vừa xác nhận rời caret ⇒ vạch hiện `confirmed` **ngay**. `resolveSegmentRule` đổi **0 dòng** | Thêm một hành vi mà FR24 không nói ra. ⚠️ Nó chạm địa hạt của Story 2.10 (*Điều hướng segment*) — phải ghi rõ 2.5 chỉ dựng **một** đường dời tối thiểu, và 2.10 sẽ **dùng lại** nó chứ không dựng đường thứ hai |
| **(b)** | Đổi thứ tự: `confirmed` thắng `primary` | Phá một quyết định 🔴 đã ghi lý do, **và** làm mất chỉ báo *"con trỏ đang ở đây"* trên mọi câu đã xác nhận — tức đổi nghĩa một trong năm giá trị. Đây là tầng `DESIGN.md`, cần Ice ký |
| **(c)** | Chấp nhận, và **sửa AC1 +** `EXPERIENCE.md` thành *"vạch hiện *`confirmed`* ngay khi con trỏ rời câu"* | Là một **thay đổi tầng spec**, phải có chữ ký. 🔴 Và nó **không** được làm theo kiểu sửa `epics.md` cho khớp mã — *"Năng lực chưa dựng ≠ lệch spec"* |

**Đề xuất mặc định: (a).** Ba lý do: ① nó là hành vi người dịch thật sự muốn — KF-2 (`EXPERIENCE.md:285-297`) mô tả một vòng *xác nhận → câu kế* lặp hàng trăm lần mỗi Chương; ② tiêu chí nghiệm thu NFR17 (`EXPERIENCE.md:184`) đòi dịch trọn một Chương **không chạm chuột một lần nào**, mà hôm nay *"đường duy nhất đặt được con trỏ vào một câu là một cú bấm chính xác từng pixel"* (phát hiện của Story 2.4, `2-4…md:1575-1578`, chủ: Ice) — (a) trả về một đường bàn phím thật; ③ nó **không đụng một quyết định 🔴 nào**.

### Quyết định #2 — 🔴 Hai vạch lề chồng nhau khi hai câu ngắn nằm cùng một dòng

**Sự kiện đo được (bàn đo Story 2.2, tái lập trên cả hai engine).** Vạch là `position: absolute; left: 8px`. Hai câu cùng dòng ⇒ hai vạch cùng `top` **và** cùng `left` ⇒ vạch vẽ sau che vạch vẽ trước. Fixture 5 câu vẽ **4 vạch nhưng chỉ thấy 2 vị trí**: `confirmed` bị `primary` che, `tm-rule` bị `ornament` che (`deferred-work.md:2052-2064`).

Hôm nay chưa chạm tới được vì **chỉ** `primary` **có nguồn dữ liệu** và caret chỉ có một. 🔴 **Story 2.5 là story đầu tiên làm hai vạch cùng tồn tại** — món nợ này ghi đích danh chủ là 2.5, và ghi kèm *"một lượt ký của Ice cho hình dạng lời giải"*. `DESIGN.md:380` và `EXPERIENCE.md:105-113` **không phân xử** ca này.

| Đường | Nội dung | Cái giá |
| --- | --- | --- |
| **(a)** ⭐ | Chia máng 22px thành **hai làn**: vạch của câu bắt đầu sớm hơn ở làn trong, câu sau ở làn ngoài. Chỉ áp khi phát hiện trùng `top` | Máng chật hơn; phải đo lại tương phản và bề rộng thật trên cả hai engine |
| **(b)** | Một dòng chữ ⇒ **một** vạch, lấy trạng thái của câu có **ưu tiên cao nhất** trên dòng đó | Rẻ nhất, nhưng nó **nói dối**: một dòng chứa một câu đã xác nhận và một câu chưa dịch sẽ hiện là đã xác nhận. Cùng lớp lỗi mà `prd.md:291` gọi là *"nói dối về trạng thái công việc"* |
| **(c)** | Vạch **chia đôi theo chiều dọc** của dòng — nửa trên câu A, nửa dưới câu B | Đúng thông tin, nhưng ở giãn dòng 1.95 với vạch 2px thì mỗi nửa còn rất mỏng; phải đo mới biết đọc được không |

**Đề xuất mặc định: (a), và CHỈ chốt sau một lượt đo trên bàn đo.** Lý do chọn (a) làm ứng viên đầu: nó là đường duy nhất giữ đúng mệnh đề *"vạch cao đúng bằng câu tương ứng"* (`epics.md:2067`) mà không phải đổi nghĩa của một giá trị nào. 🔴 **Không chốt bằng suy luận** — dựng fixture có ít nhất hai cặp câu ngắn cùng dòng, chụp cả hai engine, cả hai theme, rồi mới ký.

### Quyết định #3 — ⚠️ Bảng năm giá trị vạch thiếu một hàng, và 2.5 là chỗ duy nhất phân xử được

Một câu **đã dịch bằng tay · chưa xác nhận · con trỏ ở chỗ khác** không ứng với giá trị nào trong năm: `confirmed` sai (chưa ai ký), `tm-rule` sai (không phải máy điền), *không vạch* cũng sai (nó **đã có** bản dịch). Bảng `EXPERIENCE.md:105-113` đơn giản không có hàng đó. Nhánh hiện rơi về *không vạch* và điều đó **đã được ghi ra** thay vì giấu (`editorSegments.ts:104-115`).

🔴 Từ story này trở đi khe hở **chạm tới được**: người dùng gõ xong một câu rồi bấm sang câu khác mà chưa xác nhận là ca **thường nhật nhất** của tính năng, không phải ca biên.

| Đường | Nội dung | Cái giá |
| --- | --- | --- |
| **(a)** ⭐ | Giữ *không vạch*, và **sửa** `EXPERIENCE.md` để bảng nói ra hàng đó: *"đã dịch, chưa xác nhận ⇒ không vạch — vạch chỉ nói ai đã ký, không nói có chữ hay chưa"* | Sửa một tài liệu tầng nguyên tắc ⇒ cần Ice ký. Cái mất: người dùng không phân biệt được *chưa dịch* với *đã dịch chưa ký* bằng vạch lề |
| **(b)** | Dùng `tm-rule` cho hàng này | **LOẠI.** `tm-rule` mang nghĩa cố định trong toàn ứng dụng: *"máy đề xuất, chưa ai xác nhận"* (`EXPERIENCE.md:97,101`). Mượn nó là làm hỏng cả Proofreader (FR81) lẫn ranh giới bóc (đường nhập) |
| **(c)** | Một giá trị vạch **thứ sáu** | **LOẠI.** `EXPERIENCE.md:99` khai năm giá trị là *"tài nguyên hữu hạn đã tiêu hết"*; UX-DR22 đã bắt Proofreader đi đường gạch chân lượn sóng **chính vì** không còn giá trị nào. Cổng Kiểm I đỏ ở giá trị thứ sáu |

**Đề xuất mặc định: (a).** Lý do: cái mất của (a) đã có một kênh khác chở — văn bản **có chữ** là chỉ báo *"đã dịch"* rõ hơn bất kỳ vạch nào, và vạch lề theo `DESIGN.md:380` được định nghĩa là nơi đọc **trạng thái xác nhận**, không phải nơi đọc *"có chữ hay chưa"*.

### Quyết định #4 — id command và hợp âm phím

**Sự kiện.** Registry hôm nay **chưa có một command nào tiền tố** `editor.` (hiện có: `mode.*` · `layout.*` · `focus.*` · `library.*` · `source.*` · `lookup.*` · `selection.*` · `attribution.*` · `shortcuts.*`). Văn phạm id bị cưỡng chế: `COMMAND_ID_RE = /^[a-z0-9]+(\.[a-z0-9_]+)+$/`, và id dùng **cùng văn phạm khoá chấm** với khoá i18n (AD-34).

- **id đề xuất:** `editor.confirm_segment` — mở tiền tố miền `editor.` mà Story 2.8/2.9/2.10 sẽ dùng tiếp (`editor.merge_segments`, `editor.split_segment`, `editor.next_segment`…). ⚠️ Chốt tiền tố ở đây là một quyết định có tuổi thọ dài hơn story này.
- **hợp âm đề xuất:** `Mod+Enter` (`⌘↵`). `EXPERIENCE.md:169` đã dùng `⌘↵` cho *"xác nhận nhập"* ở màn xem trước — **cùng ngữ nghĩa "ký duyệt"**, khác bề mặt. Hôm nay `⌘↵` **chưa ai đăng ký**, nên nó rảnh.

⚠️ **Giới hạn thật, ghi ra thay vì để người sau phát hiện:** `check:commands` kiểm trùng hợp âm **trên toàn bộ registry**, không theo chế độ. Ngày Epic 6 đăng ký lệnh *"xác nhận nhập"* cũng bằng `⌘↵`, cổng sẽ **đỏ** và một trong hai phải nhường. Ghi món này vào `deferred-work.md` với chủ là **Story 6.2**, đừng để nó lộ ra dưới dạng một cổng đỏ không ai hiểu.
⚠️ Cổng chỉ kiểm trùng **nội bộ** bộ command — nó **không biết gì** về phím của hệ điều hành. Lưới ở đây là con người (tiền lệ: `⌘⌥H` bị macOS nuốt, Ice chốt đổi sang `Mod+Alt+J` ngày 2026-08-06).

**Cần Ice chốt:** tiền tố `editor.` và hợp âm `Mod+Enter`.

### Quyết định #5 — `segment.status` lưu dạng gì

| Đường | Nội dung | Cái giá |
| --- | --- | --- |
| **(a)** ⭐ | `TEXT NOT NULL DEFAULT 'draft'`, hai giá trị `'draft'` \| `'confirmed'`, cưỡng chế ở tầng Rust | Khuôn đã có: `chapter.status` và `config_value.kind` đều làm vậy, và doc-comment `schema.rs:284-286` nói thẳng *"cưỡng chế giá trị hợp lệ là việc của tầng Rust"*. Mở đường cho một giá trị thứ ba (ví dụ *về hưu*, *nhập từ song ngữ*) mà **không** cần một bước di trú nữa |
| **(b)** | `INTEGER NOT NULL DEFAULT 0` (0/1) | Rẻ hơn vài byte, nhưng đóng cứng thành boolean. Ngày FR117 hoặc AD-5 cần một giá trị thứ ba thì phải mở thêm một bước di trú |
| **(c)** | Không cột `status`; suy từ sự tồn tại của `SegmentVersion` mới nhất | **LOẠI bằng chính bảng AD-31.** Hàng *"sửa văn bản của segment đã xác nhận → chưa xác nhận"* **không tạo và không xoá** version nào ⇒ hai trạng thái khác nhau cho cùng một tập version. Không suy được |

**Đề xuất mặc định: (a).** Kèm một `CHECK`? **Không** — đi đúng khuôn hai cột kia; thêm `CHECK` ở một bảng mà hai bảng anh em không có là dựng hai quy ước cho cùng một việc.

### Quyết định #6 — bảng `segment_version` thuộc story nào

AC2 đòi *"tạo đúng một *`SegmentVersion`*"* ⇒ 2.5 phải **ghi** phiên bản. Story 2.6 đòi **xem lại và khôi phục**. Kho hôm nay **chưa có bảng nào** cho khái niệm này (`deferred-work.md:1961-1968`).

| Đường | Nội dung | Cái giá |
| --- | --- | --- |
| **(a)** ⭐ | 2.5 tạo bảng `segment_version` **tối giản** (`id` · `segment_id` · `target_text` · `created_at`) và ghi vào nó. 2.6 chỉ thêm đường **đọc** + khôi phục + giao diện | AC2 nghiệm thu được ngay tại story này. Rủi ro: 2.6 phát hiện thiếu một cột ⇒ một bước di trú nữa. Giảm rủi ro bằng cách đọc trước AC của Story 2.6 (`epics.md:2207-2237`) — nó đòi **thời điểm** (ISO-8601 UTC) và **lịch sử của segment đã về hưu vẫn tra được**, cả hai đã nằm trong bốn cột trên |
| **(b)** | 2.5 chỉ đổi `status`; hoãn version sang 2.6 | AC2 **không đạt được** ở story này ⇒ phải ghi một món nợ có chủ. Và nó để hở đúng hố (1) của AD-31 §Prevents trong một khoảng thời gian |

**Đề xuất mặc định: (a).
Kèm một quyết định phụ — một bước di trú hay hai? **Đề xuất** một bước duy nhất, số 7**, chứa cả `ALTER TABLE segment ADD COLUMN status …` và `CREATE TABLE segment_version …`. Lý do: cả hai là** DDL của cùng một khái niệm **(máy trạng thái AD-31), cùng tầng, cùng giao dịch — đúng thứ mà tiền lệ cho phép (bước 5 gồm `CREATE TABLE segment`** và **`CREATE INDEX`). Thứ Quyết định #4 của Story 2.1 cấm nhét vào một bước là một** quy tắc nghiệp vụ**, không phải một câu DDL thứ hai. Tách thành 7 và 8 là dựng một `user_version` trung gian mà** không **`project.db`** nào từng dừng ở đó**.

### Quyết định #7 — 🔴 xác nhận một câu CHƯA DỊCH (`target_text` rỗng): cho hay từ chối

Không tài liệu nào phân xử. Hậu quả nếu cho:

- một `SegmentVersion` mang chuỗi rỗng vào lịch sử FR101 — người dùng khôi phục về *"không có gì"*;
- và ở Epic 7, FR56 ghi **một cặp TM có vế đích rỗng**. Cặp đó sẽ được khớp 100% ở một Chương sau và **điền sẵn một bản dịch rỗng** (FR58). Đây là dữ liệu hỏng **vĩnh viễn** trong một kho dùng chung, sinh ra bởi một thao tác trông vô hại.

| Đường | Nội dung |  |
| --- | --- | --- |
| **(a)** ⭐ | **Từ chối**, trả `IpcError` với một `message_key` riêng (*"chua co ban dich de xac nhan"*). Phân biệt được với mọi lối khác |  |
| **(b)** | Cho phép — *"đạt chuẩn của tôi"* có thể nghĩa là *"câu này cố ý bỏ trống"* |  |
| **(c)** | Cho phép nhưng **không** ghi cặp TM ở Epic 7 | Đẩy một ngoại lệ sang một epic khác, nơi nó sẽ bị quên. Chỗ phân xử đúng là ở đây |

**Đề xuất mặc định: (a).** Nếu Ice thấy ca *"cố ý bỏ trống"* là thật, đường đúng là một trạng thái riêng, không phải mượn `confirmed` — và đó là một `AD` mới, không phải một dòng mã.

---

## Tasks / Subtasks

- [x] **Task 0 — Chốt bảy quyết định** (AC1, AC2, AC5, AC9, AC10, AC14)
  - [x] 0.1 Trình bảy quyết định cho Ice, mỗi cái kèm đề xuất mặc định và cái giá. **Dừng, chờ ký.**
  - [x] 0.2 Ghi nguyên văn từng lượt ký kèm **ngày** vào §Dev Agent Record của story này.
  - [x] 0.3 Với Quyết định #2: dựng fixture bàn đo **trước** khi hỏi — Ice cần nhìn ảnh, không phải nghe mô tả.
- [ ] **Task 1 — Điều kiện khởi hành, kiểm bằng TAY** *(1.2 xong; 1.1 chờ Ice)* (Điều kiện khởi hành)
  - [ ] 1.1 🔴 **CHỦ: ICE** — lượt này cần mắt và tay, dev làm không được. Mở app, mở một Chương thật, gõ vào một câu **chưa dịch**, xác nhận chữ hạ cánh. Nếu **không** gõ được: **DỪNG**, 2.5 quay về `backlog`. *(Task 1.0 của Story 2.4 đã mở món này ngày 2026-08-13 — lượt này là kiểm lại, không phải chẩn đoán lại.)*
  - [x] 1.2 Chạy `cargo test --locked` và `npm run test` lấy **mốc gốc bằng số đo, không bằng suy luận**. ⚠️ Tiền lệ: Story 2.1 ghi baseline `274` bằng phép trừ và **sai**; số thật lấy bằng `git worktree`. 🔵 **Đo 2026-08-14:** `cargo test` **324 xanh / 0 đỏ / 5 bỏ qua** · `npm run test` **41/41** *(số 40 ghi trong Dev Notes đã trôi)*.
- [x] **Task 2 — Rust: lược đồ, bước di trú 7** (AC9, Quyết định #5, #6)
  - [x] 2.1 Đọc `schema.rs:285-455` **trọn vẹn** trước khi gõ — vết sẹo số 4 và lý do *"*`ALTER TABLE`*, không sửa *`SEGMENT_DDL`*"* nằm ở đó.
  - [x] 2.2 Thêm hằng DDL mới theo khuôn `SEGMENT_TARGET_TEXT_DDL`, kèm doc-comment trả lời **vì sao số 7** và **vì sao không sửa hằng cũ**.
  - [x] 2.3 Thêm `Migration { to_version: 7, sql: … }`. Kiểm `validate_strictly_increasing` vẫn xanh.
  - [x] 2.4 Cập nhật doc-comment `SEGMENT_DDL:293-303` — cột `status` **hết** là *"cố ý không có"*. Sửa **tại chỗ** kèm dấu 🔵 và ngày, đúng cách `target_text` đã được sửa ngày 2026-08-12.
  - [x] 2.5 Test: một `project.db` ở `user_version = 6` mở lên và di trú thẳng lên 7; một `project.db` ở phiên bản **cao hơn** ứng dụng bị **từ chối mở**, không bao giờ ghi vào.
- [x] **Task 3 — Rust: máy trạng thái + lệnh IPC mới** (AC2, AC3, AC7, AC8, AC12, AC13, AC14)
  - [x] 3.1 Đọc `commands/segment.rs` **trọn vẹn** (600 dòng). Ghi vào Dev Notes: hiện `save_segment_targets` làm gì · 2.5 chạm chỗ nào · hành vi nào **phải giữ nguyên**.
  - [x] 3.2 Viết **hàm thuần** `confirm_segment(open: Option<&OpenWork>, segment_id: i64) -> Result<ConfirmOutcome, IpcError>` theo đúng khuôn hai lớp. Một giao dịch: đọc trạng thái hiện tại → phân xử → `UPDATE segment SET status` + `INSERT INTO segment_version`.
  - [x] 3.3 Vỏ mỏng `#[tauri::command]` trong `mod wire`, lấy `State` qua `try_state` (🔴 không `state()` — `panic = "abort"` giết cả tiến trình). Giữ `MutexGuard` **xuyên suốt**, không nhả sớm.
  - [x] 3.4 Cài AC13 (xác nhận lại, văn bản không đổi ⇒ vô hại) và AC14 (id lạ · đã về hưu ⇒ `IpcError` phân biệt được) **trong cùng hàm thuần**, dùng ô lỗi **có kiểu** như `BatchReject`, không đoán lại từ chuỗi.
  - [x] 3.5 Cài AC3 — sửa văn bản của segment đã xác nhận ⇒ về `'draft'`. 🔴 **Chỗ này KHÔNG được nằm trong** `save_segment_targets` (AC8). Cân nhắc và ghi rõ lựa chọn: một lệnh riêng, hay một `UPDATE` có điều kiện trong chính lượt xác nhận kế tiếp. Đề xuất: một hàm thuần thứ hai, gọi từ đường flush ở tầng **Rust**, không tầng TS (AC12).
  - [x] 3.6 Khai `message_key` mới bằng `macro_rules! message_keys!` — không viết danh sách song song.
  - [x] 3.7 Doc-comment tại điểm ghi trạng thái nêu **đích danh** Story 2.7 (xuất xứ) và Epic 7 (cặp TM) là hai thứ sẽ móc vào đúng đây (AC11).
- [x] **Task 4 — Rust: test hợp đồng** (AC2, AC3, AC4, AC8, AC13, AC14, AC16)
  - [x] 4.1 Nâng `segment_contract.rs:1318` để nó đọc **10 cột** thay vì 9 và vẫn khẳng định auto-save chạm **đúng hai cột**. ⚠️ Đây là nâng phép kiểm cho nó **nói thật về lược đồ mới**, không phải nới nó cho hết đỏ.
  - [x] 4.2 Ca: xác nhận ⇒ `status = 'confirmed'` **và đúng một** hàng `segment_version`.
  - [x] 4.3 Ca: auto-save sau khi xác nhận ⇒ `status` **không đổi**, số hàng `segment_version` **không đổi** (AC4).
  - [x] 4.4 Ca: sửa văn bản segment đã xác nhận ⇒ `status = 'draft'`, số hàng version **không đổi** (AC3).
  - [x] 4.5 Ca: xác nhận lại không sửa gì ⇒ **không** hàng version thứ hai (AC13).
  - [x] 4.6 Ca: id lạ · segment đã về hưu (dựng bằng SQL trực tiếp trong fixture) ⇒ `IpcError` đúng `message_key` (AC14).
  - [x] 4.7 Tên hàm test là một **CÂU khẳng định**, không `test_foo`. Khuôn có sẵn: `a_flush_touches_exactly_target_text_and_updated_at_and_nothing_else`.
- [x] **Task 5 — TypeScript: adapter IPC + nối** `isConfirmed` (AC10, AC12)
  - [x] 5.1 Thêm trường `status` vào `ChapterSegment` (`src/config/segment.ts:66-76`) — `snake_case`, đúng tên struct Rust trả về. ⚠️ Hai chiều khác nhau: `invoke()` **gửi** camelCase, struct **trả về** giữ `snake_case`.
  - [x] 5.2 Thêm adapter cho lệnh mới. 🔴 **Adapter ở** `src/config/*.ts` **KHÔNG BAO GIỜ ném** — một `invoke`, một `try/catch`, trả hình dạng ba trạng thái `{ <giá trị> | null, error: IpcError | null }`.
  - [x] 5.3 Sửa **đúng hai dòng** ở `editorSegments.ts:141-142`: `isConfirmed` đọc `status` thật. Cập nhật doc-comment `:73-79` kèm 🔵 và ngày.
  - [x] 5.4 Sửa doc-comment khe hở `:104-115` theo Quyết định #3 đã ký.
  - [x] 5.5 Kiểm kiểu **lúc chạy** cho dữ liệu qua dây — Rust có thể trả `null` sau một lượt đổi lược đồ, type guard là chỗ duy nhất biết.
  - [x] 5.6 ⚠️ Giữ `editorSegments.ts` là **module thuần** — không thêm một `import` giá trị nào. Một dòng `import` ở đó giết Kiểm I.
- [x] **Task 6 — Command, phím, i18n** (AC1, AC5, AC15, Quyết định #1, #4)
  - [x] 6.1 Thêm trường tiêm vào `CommandDeps` (`src/commands/index.ts:161`) — **TIÊM VÀO, không import** state Vue; `index.ts` phải nạp được bằng Node thuần (Kiểm C/D/E).
  - [x] 6.2 `target.register({ id: 'editor.confirm_segment', labelKey: 'command.editor.confirm_segment', keys: ['Mod+Enter'], run: … })` theo khuôn ở `index.ts:531-600`, kèm `portMissing(...)` khi cổng vắng — handler vắng cổng thì **KÊU**, không ném và không im.
  - [x] 6.3 Nối cổng ở `src/main.ts` (🔴 đăng ký command ở `main.ts`, **không** trong `App.vue` — một lượt HMR sẽ gọi `installCommands()` lần hai và `register()` ném vì id trùng).
  - [x] 6.4 Cài Quyết định #1 đã ký. Nếu là đường (a): đường dời con trỏ dùng lại `setEditorCaret` (`editorPanelState.ts:136-141`) — nó **đã** flush cho câu vừa rời, tức AC7 đi qua một đường đã nghiệm thu, không một đường mới.
  - [x] 6.5 Nếu bề mặt Editor bắt phím bằng `@keydown`: ⚠️ **Kiểm A của** `check:commands` **chỉ canh** `@click` (`deferred-work.md:166`). Một `@keydown` gọi thẳng hàm dựng một đường thứ hai mà **không cổng nào nhìn thấy**. Phím tắt và mọi bề mặt phải phát **cùng một** `dispatch(...)`.
  - [x] 6.6 Thêm khoá `command.editor.confirm_segment` vào `vi.json`. Khoá chấm **phẳng**, không giá trị rỗng. Giọng văn: nói việc, không nói cảm xúc; câu trạng thái viết ở dạng **vô nhân xưng**.
- [x] **Task 7 — vitest** (AC10, AC13, AC16)
  - [x] 7.1 Test cho `resolveSegmentRule` với `isConfirmed: true` — cả ca có caret (Quyết định #1) lẫn ca không caret.
  - [x] 7.2 Test cho khe hở Quyết định #3: `targetText` khác rỗng · `isConfirmed: false` · `hasCaret: false` ⇒ giá trị đã ký.
  - [x] 7.3 Tệp đặt ở `tests/frontend/**`,** không **đồng vị trí trong `src/**` (bốn cổng đếm quần thể `src/**`; một tệp test đổ vào đó thổi phồng mẫu số).
  - [x] 7.4 `import { describe, it, expect } from 'vitest'` tường minh — **không** `globals: true`.
  - [x] 7.5 🔴 **Không thêm** `?.` **vào mã sản phẩm cho hết đỏ.** Khoảng thiếu của `happy-dom` vá ở `tests/frontend/support/setup.ts`, mỗi mục kèm một dòng nói nó thiếu gì và ai đọc nó.
- [x] **Task 8 — Bàn đo: hai vạch cùng tồn tại** (AC1, AC10, Quyết định #1, #2, #3)
  - [x] 8.1 Dựng fixture có: ≥1 câu `confirmed` không caret · ≥1 câu `primary` · ≥2 câu ngắn **cùng một dòng** mang hai trạng thái khác nhau · ≥1 câu đã dịch chưa xác nhận.
  - [x] 8.2 Chụp **cả hai engine** (Blink + WebKit) × **cả hai theme**. Lưu vào `2-5-ban-do/`, theo đúng cách 2.2/2.3 đã lưu.
  - [x] 8.3 ⚠️ Kiểm fixture **không tự che** ca cần đo — tiền lệ: fixture của 2.2 đặt caret đúng vào câu duy nhất và che mất khe hở, review mới bắt được.
  - [x] 8.4 Đối chiếu tương phản của `confirmed` trên nền cả hai theme. Sàn WCAG **đóng băng trong script**, không đọc từ thứ đang bị kiểm.
- [x] **Task 9 — e2e (tuỳ điều kiện)** (AC5, AC16)
  - [x] 9.1 Chỉ viết spec e2e cho lượt xác nhận trên một câu **đã có chữ**. 🔴 **Không** dựng một ca mới đặt caret vào câu rỗng — bộ e2e có tiền sử chập chờn đúng ở điểm đó (1 xanh/7 đỏ), và một ca đỏ ở đó **không phân biệt được** *"2.5 hỏng"* với *"bộ đo hỏng"*.
  - [x] 9.2 Trong `e2e/**`:** cấm **`.click()`** của driver**, dùng `realClick()` ở `e2e/support/pointer.mjs`.
  - [x] 9.3 ⚠️ **Một dữ kiện có lợi cho story này, đo ở 2.4:** `browser.keys()` chỉ phát `keydown`, **không** `beforeinput` (`deferred-work.md:2334-2337`) — đó là lý do nó lái được **phím tắt** nhưng **không** lái được một lượt gõ chữ. ⇒ Lệnh xác nhận của 2.5 đi bằng hợp âm phím, tức nó **nằm đúng trong phần bộ e2e lái được**. Đây là điểm khác biệt với ca đỏ của 2.3, không phải cùng một hố.
  - [x] 9.4 Gặp một lượt đỏ không tái lập được: **bắt nguyên văn TRƯỚC**, đừng chẩn đoán từ trí nhớ.
- [x] **Task 10 — Cổng và nghiệm thu cuối** (AC16)
  - [x] 10.1 Mọi phép kiểm mới chạy **đỏ-rồi-xanh** một lượt, ghi lại cả hai chiều.
  - [x] 10.2 11 cổng `npm` · `npm run build` · `npm run test` · `cargo test --locked`. Ghi **số thật**, không suy ra.
  - [x] 10.3 Xét lại sàn quần thể nếu có tệp mới (`FILE_FLOOR` · `COMPONENT_FILE_FLOOR` · `RS_FLOOR` · `VUE_FLOOR` · `COMMAND_FLOOR`). Sàn đặt ở ~80–85% số thật.
  - [x] 10.4 ⚠️ `check:scope` **+** `check:scope:bundled` **nằm ngoài** `pre-push` — cần cổng 1420 trống, chạy tay.
  - [x] 10.5 Nếu thêm một cổng mới: **sửa BA danh sách** (`package.json` · `.github/workflows/ci.yml` · `.githooks/pre-push`), `check:gates` canh cả ba. *(Không có dấu hiệu story này cần cổng mới.)*
- [x] **Task 11 — Sổ nợ và tài liệu**
  - [x] 11.1 Đóng bốn món có chủ 2.5 ở `deferred-work.md`: `:2046` (`isConfirmed`) · `:2052-2064` (hai vạch chồng) · `:2066-2072` (hàng còn thiếu) · `:2388-2391` (vế xác nhận của AC3 Story 2.3). **Nối tiếp** `→ ✅ ĐÃ ĐÓNG <ngày> (Story 2.5)` kèm **cách đóng**; đóng một nửa thì ghi 🟡 và liệt kê phần còn hở. **Không bao giờ xoá** một mục.
  - [x] 11.2 Ghi món mới có chủ: xung đột `⌘↵` với lệnh *"xác nhận nhập"* của Epic 6 (chủ: **Story 6.2**).
  - [x] 11.3 Nếu Quyết định #1 hoặc #3 dẫn tới sửa `EXPERIENCE.md`/`epics.md`: sửa **tại chỗ kèm ngày và lý do**, và **chỉ sau khi Ice ký**. 🔴 Đừng sửa spec cho khớp mã đã viết.
  - [x] 11.4 Cập nhật `sprint-status.yaml` — và sửa dòng đã hết đúng ở `:112-113` (*"bước di trú kế tiếp phải đánh số 5"*) kèm 🔵 và ngày.
  - [x] 11.5 Nếu chạm một tệp mang mệnh đề đã hết đúng: sửa tại chỗ kèm 🔵 và ngày.

---

## Dev Notes

### Bản đồ tệp sẽ SỬA — hiện trạng · chỗ chạm · thứ phải giữ nguyên

🔴 Mục này là thứ ngăn lớp lỗi đắt nhất: sửa một tệp mà không biết nó đang giữ hợp đồng gì.

| Tệp | Hiện nó làm gì | 2.5 chạm chỗ nào | 🔴 Phải giữ nguyên |
| --- | --- | --- | --- |
| `src-tauri/src/core/store/schema.rs` | `PROJECT_MIGRATIONS` đích ở **6** (1·2·3·**5**·6, số 4 cháy). `SEGMENT_DDL` = bảng + `idx_segment_chapter_ord` | Thêm bước **7** | Không sửa `SEGMENT_DDL` tại chỗ — nó là DDL của một bảng **tạo mới**; một `project.db` đã ở v5 không bao giờ chạy lại nó, và sửa nó cho ra **hai lược đồ khác nhau cho cùng một số phiên bản** (đúng vết sẹo số 4) |
| `src-tauri/src/commands/segment.rs` | `split_chapter_into_segments` · `read_open_chapter_segments` · `save_segment_targets` (lô, một giao dịch, `prepare_cached`, khoá theo `segment.id`). Vỏ `mod wire` dùng `try_state` | Thêm hàm thuần + vỏ mỏng **mới** | `save_segment_targets` `UPDATE` **đúng hai cột**; khoá theo `segment.id` **không** theo `ord` (AD-3 — 2.8 sắp lại `ord` mà giữ `id`) |
| `src-tauri/src/core/i18n/mod.rs` | `IpcError::new(code, message_key, params, retryable)`, bốn trường **riêng tư**; `message_keys!` sinh `enum` + `ALL` + `as_str` + bảng tham số | Thêm `message_key` mới | Dựng lỗi **chỉ** qua `new`. Không `#[serde(rename_all = "camelCase")]` trên `IpcError` — bốn tên trường là **dây**, `tests/ipc_contract.rs` khoá lại |
| `src/config/segment.ts` | `ChapterSegment` (6 trường, `snake_case`) · ba adapter, mỗi cái trả hình dạng ba trạng thái | Thêm trường `status` + adapter mới | Adapter **không bao giờ ném**. Struct trả về giữ `snake_case`; tham số gửi đi là camelCase |
| `src/panels/editorSegments.ts` | Module **thuần**. `SEGMENT_RULE_VALUES` (5) · `resolveSegmentRule` · `segmentRuleInputOf` (hai hằng `false` có chú thích) | **Đúng hai dòng** ở `:141-142` + hai doc-comment | Không `import` giá trị nào — điều kiện để `check-commands.mjs` `import()` **hàm thật** và chạy bằng Node thuần |
| `src/panels/editorPanelState.ts` | State panel; `setEditorCaret` flush khi rời câu; `flushEditorNow(): Promise<FlushResult>`; `editedText` **tách rời** `segments` | Nối lệnh xác nhận | 🔴 `editedText` **phải** ở tách rời `segments` — `segments` giữ **bản lúc nạp**, tức mốc mà FR117 so sánh. Ghi đè lên nó là huỷ mốc đó, và nó hỏng ở Epic sau mà không gì nối được về đây |
| `src/panels/EditorPanel.vue` | Một `<span class="sent">` mang `contenteditable` tại một thời điểm; `onDocMouseDown` `setAttribute` **đồng bộ**; `ruleClassById` là Map; 4 khối `.gmark.rule-*` đọc `var(--color-…)` | Có thể chạm CSS vạch (Quyết định #2) | Bản vá `setAttribute` đồng bộ trong `mousedown` — nó là lời giải của một chẩn đoán **đã bị bác một lần**; đừng chạm. Bốn màu vạch khai trong `<style scoped>`, mỗi màu một khối cổng đọc được — **không** bind màu qua `:style` |
| `src/commands/index.ts` | `CommandDeps` (tiêm) · `registerAll` · `installCommands`; 34 command, chưa có tiền tố `editor.` | Thêm 1 command + 1 cổng tiêm | Luật **erasable-only**: không `import` giá trị của `vue`/`dockview`, không `enum`/`namespace`/parameter property. Một `import` giá trị giết **ba** phép kiểm cùng lúc |
| `src/i18n/vi.json` | Khoá chấm **phẳng**, tiền tố miền | Thêm `command.editor.*` + `err.segment.*` | Placeholder đúng dải `{ten_tham_so}`; tham số mang **DỮ LIỆU**, không mang câu |
| `src-tauri/tests/segment_contract.rs` | 1.564 dòng; `read_all_segment_rows` đọc **9 cột thô** | Nâng lên 10 cột + ~6 ca mới | Nâng phép kiểm cho nó **nói thật**, không nới nó cho hết đỏ |

### Bất biến không được phá — bảng tra nhanh

| \# | Bất biến | Nguồn | Vi phạm được mà không cổng nào đỏ? |
| --- | --- | --- | --- |
| 1 | Bảng AD-31 đúng từng hàng | `ARCHITECTURE-SPINE.md:368-392` | **Có** — đây là lý do Task 4 tồn tại |
| 2 | Xác nhận flush trước, và flush xong = **đã vào WAL** | AD-35 (c) | **Có** |
| 3 | Mọi ghi qua `store::Writer` nối tiếp; không module nào tự mở kết nối ghi | AD-11 | Không (test biên canh) |
| 4 | Máy trạng thái ở Rust, không ở TS | AD-1 | **Có** |
| 5 | Command đăng ký trước, bind phím sau | AD-34 | Nửa — Kiểm A **chỉ** canh `@click` |
| 6 | `segment.id` bất biến, không tái dùng | AD-3 | **Có** |
| 7 | Di trú chỉ tiến; phiên bản mới hơn ⇒ **từ chối mở**, không bao giờ ghi vào | AD-30 | Không |
| 8 | Đúng năm giá trị vạch lề | AC12 Story 2.2 | Không (Kiểm I) |
| 9 | So xuất xứ bằng **văn bản**, cấm cờ dirty | AD-31 hợp đồng phụ | **Có** |
| 10 | Đổi một bất biến ⇒ một `AD` **mới** (kho đang có **45**) | `project-context.md` | **Có** |

### Bài học từ story trước — thứ đã tốn tiền một lần rồi

- 🔴 **Trúng tiền đề chưa phải trúng cơ chế.** Story 2.3 chẩn đoán *"AD-34 giành tiêu điểm"* và **sai** — đo lại cho thấy **không ai giành cả**; nguyên nhân thật là một lượt đặt thuộc tính **bất đồng bộ**. Story 2.4 lặp đúng lớp lỗi đó với `AppleKeyboardUIMode`. ⇒ Gặp một triệu chứng: **đo cái cơ chế**, đừng dừng ở một tiền đề nghe hợp lý.
- 🔴 **Ba vòng chẩn đoán bị bác ⇒ DỪNG và báo Ice.** Luật này đã được viết ra ở Story 2.4 cho đúng lớp lỗi *"bộ đo chưa đứng nổi"*.
- ⚠️ **Đừng kết luận từ n=1 trên một bộ đo đã ghi là chập chờn.** Story 2.3 từng viết ra một lượt *"xanh trọn vẹn"* rồi **rút lại**.
- ⚠️ **Baseline lấy bằng đo, không bằng phép trừ.** (Story 2.1 ghi `274`, số thật `267`.)
- ✅ **Khuôn đã chạy tốt, cứ chép:** hàm thuần trước — vỏ `#[tauri::command]` sau; ô lỗi **có kiểu** thay vì đoán lại từ chuỗi; miễn trừ CSS **có tên** tham số hoá theo token; mỗi cột DDL kèm một dòng *"neo vào đâu"*; cột cố ý vắng phải ghi **chủ**.

### Số đo đã có — dùng để so, đừng đo lại

| Phép đo | Số | Ngày |
| --- | --- | --- |
| Đo + vẽ **1** vạch lề (ca thật) | 8,5 ms Blink · 5,0 ms WebKit | 2026-08-12 |
| Đo + vẽ **9.850** vạch (ca trần) | 63,1 ms · 64,0 ms | 2026-08-12 |
| Dựng DOM + bố cục Editor | 300,1 ms Blink · **1.308,0 ms WebKit** | 2026-08-12 |
| Frame max lúc lắp/tháo `contenteditable` | 17,60 ms Blink · 18,00 ms WebKit (0 vượt 50 ms) | 2026-08-12 |
| NFR18 tại `wal_threshold_bytes = 4 MiB` | trung vị 3,484 s · max 6,538 s · **1/20 vượt thật** ⇒ **KHÔNG ĐẠT** | 2026-08-13 |
| `cargo test --locked` | 324 xanh / 0 đỏ | 2026-08-13 |
| `npm run test` | 40 / 40 | 2026-08-13 |
| Bảng `segment` trên dữ liệu thật | 10.477 hàng / 21 Chương | 2026-08-12 |

⇒ Ràng buộc rút ra cho 2.5: một lượt xác nhận chạm **một** segment, nên chi phí vẽ lại vạch nằm ở cột "1 vạch" (5–8,5 ms), **không** ở cột trần. Nếu cài đặt nào bắt vẽ lại **toàn bộ** máng sau mỗi lượt xác nhận, số phải so là **63–64 ms** — vượt NFR2. Đó là một mệnh đề phải kiểm, không phải một lo lắng.

### Thư viện và phiên bản — story này KHÔNG thêm phụ thuộc nào

⇒ **Cửa rà giấy phép NFR15 không mở ở story này.** Nếu phát sinh nhu cầu thêm một gói: mở tệp giấy phép trong nguồn **đã tải** mà đọc (không tin nhãn registry), ghi vào bảng Stack của spine **TRƯỚC** khi thêm, và chỉ nhận giấy phép tương thích GPLv3 theo chiều đi vào.

Phiên bản đang ghim (đọc 2026-08-14 từ `Cargo.toml` · `package.json` — **không chép sang tệp khác**): `tauri =2.11.5` · `rusqlite =0.40.1` (feature `bundled`) · `serde =1.0.229` · `uuid =1.24.0` · `vue 3.5.40` · `typescript 5.9.3` · `vitest 4.1.10` · `@vue/test-utils 2.4.11` · `happy-dom 20.11.2` · `dockview-vue 7.0.4`.

⚠️ Crate ghim bằng `=` (`"2.6.3"` trần **nghĩa là** `^2.6.3`); npm ghim **số trần**, và CI chạy `npm ci`, không `npm install`. Nửa Rust là `cargo test --locked`.

🔵 **Một mệnh đề đã hết đúng, đừng chép lại:** *"dự án không có bộ chạy test frontend (và không được thêm — NFR15)"*. Hết đúng từ **2026-08-12** — Ice lật NFR15 và cấp phép đúng ba gói. Cửa **rà giấy phép** của NFR15 thì **vẫn đứng** cho gói tiếp theo.

### Cổng nào sẽ nhìn story này

| Cổng | Kiểm liên quan | Mệnh đề nó canh |
| --- | --- | --- |
| `check:commands` | A · B · C/D/E · **I** | `@click` chỉ `dispatch` · văn phạm id + có trong registry · hành vi thật của registry chạy bằng Node thuần · **đúng năm giá trị vạch lề** |
| `check:tokens` | A · **B** · C | đủ token đúng giá trị · màu viết thẳng bị từ chối, đối chiếu **hai chiều** với `SEGMENT_RULE_VALUES` · tương phản WCAG AA |
| `check:i18n` | **A** · A2 · B · C | không chữ tiếng Việt **có dấu** ở vị trí mã `.rs`/`.vue` · mọi text node qua `t()` · `vi.json` phẳng · placeholder khớp |
| `check:lint` | — | `@typescript-eslint/no-unnecessary-condition` có kiểu — cổng thứ mười, sinh ra vì `Ref` **không tự bóc** trong `<script>` |
| `check:gates` | A–F | ba danh sách cổng khớp nhau |
| `cargo test --locked` | `segment_contract` · `segment_boundary` · `ipc_contract` · `store_contract` | hợp đồng dây · ranh giới module · `every_message_key_exists_in_vi_json` |

🔴 **Luật của một cổng:** mã thoát là phán quyết · mỗi cổng phải có phép **tự kiểm** chứng minh nó đỏ được và không đỏ oan · **lỗi hạ tầng KHÔNG phải một phép kiểm đỏ** (`abort()` kèm câu *"đây là lỗi hạ tầng, không phải đạt"*) · **không phán quyết nào đọc tham số từ chính thứ nó đang kiểm**.

### Project Structure Notes

- Tệp mới (nếu có) đi vào cây đã có, **không** cây mới: Rust `src-tauri/src/commands/` và `src-tauri/src/core/store/`; TS `src/config/` (adapter IPC) · `src/panels/` (state + module thuần); test frontend `tests/frontend/**`; test Rust `src-tauri/tests/`.
- Module Rust đặt theo **khái niệm miền**, không theo nhóm năng lực — `C1`–`C10` không xuất hiện trong tên module.
- Đặt tên: Rust `snake_case` · Vue `PascalCase.vue` · state panel `<tênPanel>State.ts` · id command và khoá i18n **cùng văn phạm khoá chấm**.
- 🔴 Ánh xạ thuật ngữ cố định: Tác phẩm → `Work` · Chương → `Chapter` · segment → `Segment`. **Cấm** `Project`, `Book`, `Novel`, `Document` cho `Work`.
- Thư mục mang một khái niệm thì có `README.md` — `src/{commands,i18n,layout,modes,panels,tokens}` đã có; cập nhật cùng lượt nếu thêm một khái niệm.
- Ngày giờ: **ISO-8601 UTC** trong database; định dạng hiển thị **chỉ** ở frontend.

### Văn hoá viết mã của kho này

- Chú thích **tiếng Việt, dày, chở LÝ DO** — không kể mã làm gì, mà trả lời *vì sao hình dạng này chứ không phải hình dạng kia*, và **phương án bị loại đã bị loại bằng gì**.
- 🔴 Một quyết định không hiển nhiên phải kèm một **PHÉP ĐO**, không một sở thích: con số, ngày đo, `tệp:dòng` làm bằng chứng. Khuôn: *"⚠️ Đo 2026-08-14: … Hệ quả đo được: …"*
- Ghi thẳng chỗ **YẾU** thay vì giấu — mỗi module chở một mục *"GIỚI HẠN THẬT, ghi ra thay vì để người sau tự phát hiện"*.
- Ký hiệu: 🔴 luật không được phá · ⚠️ bẫy hoặc chỗ dễ đọc nhầm · ✅ đã đóng · 🟡 đóng một nửa · 🔵 **một mệnh đề cũ đã hết đúng** · ⇒ kết luận.
- 🔴 **Emoji** `U+26D4` **(biển cấm) bị CẤM** trong toàn kho và trong cả câu trả lời cho Ice. Viết `không` / `KHÔNG` thành chữ.
- 🔴 **Đừng bắt chước một ký hiệu chưa hiểu** — `grep` đếm số lần **và tìm định nghĩa** trước khi dùng lại.
- 🔴 **Sửa KIỂU cho nó nói thật; đừng nhét một cảnh báo hay một miễn trừ để cổng hết đỏ.** Mọi miễn trừ phải **có tên**, có lý do tại chỗ, và **chết được** (`reportUnusedDisableDirectives: 'error'`).
- Commit: `type(scope): câu tiếng Việt`, `scope` là `story-2.5`. Câu sau dấu hai chấm nói **ĐIỀU ĐÃ TÌM RA**, không chỉ điều đã sửa.

### Git — bối cảnh gần nhất

Nhánh mặc định là `master`, không `main` (viết cứng `branches: [main]` ⇒ CI **không bao giờ chạy** và **không lỗi nào được ném**). `core.hooksPath = .githooks`; `pre-push` chạy chín cổng → `npm run test` → `npm run build` → `cargo test --locked` (đo 2026-08-11: cổng 11 s · build 5 s · cargo test 34 s). Bỏ qua một lượt: `git push --no-verify`, và **phải viết lý do vào commit message**.

Năm commit gần nhất chạm địa hạt này: `6a9777b` (gutter + segment state — dựng `editorSegments.ts` · `editorPanelState.ts` · `editorGutter.ts`, bước di trú 6), `6a4e6b8` (cây test frontend + `StatusBar.vue` + `editorFlush.ts`), `c86c2fb` (tách Chương), `1c7658d` (đổi vai hai panel `source` → `display`), `8ac9ccb` (hai con trỏ `[Source]` trỏ vào văn bản đã đổi).

⚠️ `_bmad/` **·** `.claude/` **·** `.agent/` **·** `.agents/` **nằm ngoài index**; `_bmad-output/` thì **có** được theo dõi.

### Điều story này CỐ Ý không làm

Ghi ra để không ai làm thừa, và để không ai tưởng đã được xét:

- **FR117 (xuất xứ)** — chủ: **Story 2.7**. 2.5 chỉ để lại mối nối (AC11).
- **FR56 (ghi cặp TM khi xác nhận)** — chủ: **Epic 7**. Cùng chuyển tiếp, cùng chỗ móc.
- **FR101 (xem lại + khôi phục phiên bản)** — chủ: **Story 2.6**. 2.5 chỉ **ghi**, không đọc, không giao diện lịch sử.
- **FR58 (**`tm-rule` **— điền sẵn từ TM)** — chủ: **Epic 7**. Hằng `isTmFilled: false` ở `editorSegments.ts:144` **ở lại**.
- **AD-5 (về hưu do gộp/tách)** — chủ: **Story 2.8**. 2.5 chỉ dựng **hàng rào từ chối** (AC14).
- **Điều hướng segment (FR25)** — chủ: **Story 2.10**. Nếu Quyết định #1 chọn đường (a), 2.5 dựng **một** đường dời tối thiểu và 2.10 **dùng lại** nó — không dựng đường thứ hai.
- **Bộ đếm \*"12/34 đã xác nhận"*** ở đầu Panel Editor. ⚠️ Nó có trong mockup (*`mockups/key-screen-workspace.html:111`*) nhưng không*\* trong AC nào của Epic 2, và tiến độ là địa hạt FR5 (Epic 5). Không cài; nêu thành câu hỏi cho Ice.
- **Ngưỡng WAL và nhịp flush** — chủ: **Story 2.4**. Không đổi một hằng nào.

---

## References

- [Source: `_bmad-output/planning-artifacts/epics.md#Story-2.5`] — dòng 2170-2204, sáu AC nguyên văn
- [Source: `_bmad-output/planning-artifacts/epics.md#Epic-2-Ghi-chú-cài-đặt`] — dòng 830-843, *"bốn bất biến hội tụ"*
- [Source: `ARCHITECTURE-SPINE.md#AD-31`] — dòng 368-392, máy trạng thái + bảng xuất xứ + hợp đồng phụ
- [Source: `ARCHITECTURE-SPINE.md#AD-35`] — dòng 419-425, hợp đồng flush, mệnh đề (c) và *"thao tác rời rạc ghi ngay"*
- [Source: `ARCHITECTURE-SPINE.md#AD-1`] — dòng 75-79 · [`#AD-3`] 89-93 · [`#AD-6`] 113-117 · [`#AD-11`] 153-157 · [`#AD-18`] 238-288 · [`#AD-30`] 362-366 · [`#AD-32`] 394-398 · [`#AD-34`] 406-417 · [`#Consistency-Conventions`] 652-677
- [Source: `prds/prd-AuraTranslate-2026-08-02/prd.md#FR24`] — dòng 437-439 · [`#FR22`] 425 · [`#FR56`] 578 · [`#FR58`] 590 · [`#FR100`] 765 · [`#FR101`] 767 · [`#FR117`] 441-450 · [`#NFR2`] 821 · [`#NFR18`] 894-898 · [`#nói-dối-về-trạng-thái`] 291
- [Source: `ux-designs/…/EXPERIENCE.md#State-Patterns`] — dòng 105-115, năm giá trị vạch · [`#KF-2`] 285-297 · [`#Accessibility-Floor`] 175-186 · [`#Voice-and-Tone`] 51-61
- [Source: `ux-designs/…/DESIGN.md`] — dòng 180-204 (bảng token + sàn tương phản) · 380-382 (vạch lề segment) · 133-140 (`gutter-width 22px`, `segment-gutter-rule 2px`)
- [Source: `implementation-artifacts/deferred-work.md`] — `:2046` · `:2052-2064` · `:2066-2072` · `:2388-2391` (bốn món có chủ 2.5) · `:166` (Kiểm A chỉ canh `@click`) · `:1961-1968` (chưa có `SegmentVersion`)
- [Source: `implementation-artifacts/2-3-hop-dong-flush-va-trang-thai-da-luu.md`] — §Dev Agent Record, Quyết định #1/#2, chẩn đoán bị bác, ca đỏ `<span>` rỗng
- [Source: `implementation-artifacts/2-4-mui-tham-do-do-nfr18-va-nfr2-dong-thoi.md`] — `:1280-1294` (Task 1.0 mở), `:1580-1618` (NFR18 không đạt), `:1575-1578` (phát hiện UX có chủ là Ice)
- [Source: `src-tauri/src/core/store/schema.rs`] — `:285-345` (`SEGMENT_DDL` + ba cột cố ý vắng), `:431-455` (`PROJECT_MIGRATIONS`)
- [Source: `src-tauri/src/commands/segment.rs`] — `:377-389` (auto-save chạm đúng hai cột), `:420` (hàm thuần), `:583` (`mod wire`)
- [Source: `src-tauri/tests/segment_contract.rs`] — `:472` (cấm số 4), `:1318` (hai cột)
- [Source: `src/panels/editorSegments.ts`] — `:51` (năm giá trị), `:91-125` (thứ tự ưu tiên + khe hở), `:134-147` (hai dòng cần sửa)
- [Source: `src/panels/editorPanelState.ts`] — `:136-141` (`setEditorCaret` flush khi rời câu), `:161-165` (`editedText` tách rời `segments`), `:273` (`flushEditorNow`)
- [Source: `src/commands/index.ts`] — `:161` (`CommandDeps`), `:531-600` (khuôn `register`), `:886` (`installCommands`)
- [Source: `_bmad-output/project-context.md`] — 130 luật; §Critical Don't-Miss Rules là mục phải đọc trước dòng mã đầu tiên
- [Source: `_bmad-output/specs/spec-AuraTranslate/requirements.md`] — `:141-143` (FR24 + FR117), `:485` (KF-2 chuỗi FR)

---

## Dev Agent Record

### Agent Model Used

claude-opus-5

### Bảy chữ ký của Ice — 2026-08-14 (Task 0.2)

Nguyên văn lượt ký: *"xác nhận theo đề xuất, #2 quyết đinh a"*.

| \# | Chốt | Nội dung đã ký |
| --- | --- | --- |
| 1 | **(a)** | Lượt xác nhận **dời con trỏ sang segment kế tiếp**. `resolveSegmentRule` đổi **0 dòng**. 2.5 dựng **một** đường dời tối thiểu; Story 2.10 **dùng lại** nó. |
| 2 | **(a)** | **Chia làn**. 🔴 Ký sau khi phép đo đã bác đường này ở ca đối thoại — xem §Hệ quả bên dưới. |
| 3 | **(a)** | Giữ *không vạch* cho *"đã dịch, chưa xác nhận"*, và **sửa** `EXPERIENCE.md` để bảng nói ra hàng đó. |
| 4 | **(a)** | `editor.confirm_segment` + `Mod+Enter`. Mở tiền tố miền `editor.`. Xung đột `⌘↵` với Epic 6 ghi nợ, chủ **Story 6.2**. |
| 5 | **(a)** | `status TEXT NOT NULL DEFAULT 'draft'`, hai giá trị `'draft' \| 'confirmed'`, cưỡng chế ở Rust. **Không** `CHECK`. |
| 6 | **(a)** | 2.5 tạo bảng `segment_version` tối giản và **ghi**; 2.6 thêm đường đọc. **Một** bước di trú duy nhất, **số 7**. |
| 7 | **(a)** | **Từ chối** xác nhận một câu chưa dịch, `IpcError` có `message_key` riêng. |

#### 🔴 Hệ quả của chữ ký #2 — Ice ký (a) SAU KHI đã đọc phép đo bác nó

Tôi nêu bằng số rằng (a) **không đóng kín**: số làn nó đòi bằng số vạch chồng nhau đồng thời
nhiều nhất, và fixture đối thoại đòi **5** làn ⇒ mép phải 30px ⇒ tràn khỏi máng 22px. Ice đọc
và vẫn ký (a). ⇒ Đó là **quyết định của Ice**, và story đi tiếp với (a).

Vế mà chữ ký **không** phân xử là *"làm gì khi số làn vượt sức chứa của máng"* — tôi đã nêu nó
như một quyết định con của (a). Không quay lại hỏi lần hai; chọn đường **ít nói dối nhất** rồi
ghi ra để Ice bác được rẻ:

🔵 **Bước làn KHÔNG cố định 5px — nó CO cho vừa máng.** Máng 22px, vạch bắt đầu ở `left: 8px
`⇒ còn **14px**. Với vạch rộng 2px, `N` làn vừa khít khi `2N + (N-1)·khe ≤ 14`:

| N làn | bước | `left` từng làn | mép phải làn ngoài |
| --- | --- | --- | --- |
| 1–3 | 5px | 8 · 13 · 18 | 20px |
| 4 | 4px | 8 · 12 · 16 · 20 | 22px |
| 5 | 3px | 8 · 11 · 14 · 17 · 20 | 22px |
| 6–7 | 2px | 8 · 10 · … · 20 | 22px |

⇒ **Tới 7 làn vẫn KHÔNG một vạch nào bị che và KHÔNG tràn.** Đây là đường (a) mà Ice ký, với
bước làn làm cho vừa — không phải một đường thứ tư. Con số 5 làn của ca đối thoại nằm gọn trong
đó, tức phản đối đo được của tôi **được đường này đóng**.

⚠️ **GIỚI HẠN THẬT, ghi ra thay vì để người sau tự phát hiện:** từ **8 làn** trở lên không
lời giải nào trong máng 22px, và luật lúc đó là **dồn về làn cuối** *(tức chấp nhận che)*. Món
này ghi vào `deferred-work.md` với chủ là Ice, kèm con số: nó đòi **8 câu cùng một dòng**, mà
fixture đối thoại dày nhất mới cho 5.

### Debug Log References

#### Mốc gốc — đo 2026-08-14, KHÔNG suy ra bằng phép trừ (Task 1.2)

Đo trên `8245a17`, cây sạch trừ hai tệp tạo tác của `create-story`.

| Đường | Số thật | Ghi chú |
| --- | --- | --- |
| `cargo test --locked` | **324 xanh / 0 đỏ / 5 bỏ qua** | 18 bộ test; khớp số đã ghi ngày 2026-08-13 |
| `npm run test` (vitest) | **41 / 41**, 6 tệp | 🔵 Dev Notes của story viết **40** — số đó đã trôi. Ba mươi hai trong ghi chép cũ hơn nữa cũng đã trôi. |

#### Bàn đo Quyết định #2 — dựng và chạy 2026-08-14 (Task 0.3)

`2-5-ban-do-hai-vach.html` + `2-5-ban-do/chup.mjs`; bốn ảnh và `bao-cao.json` ở `2-5-ban-do/`.
Playwright 1.62.1 cài **ngoài** cây nguồn *(cửa NFR15 không mở cho một tạo tác bàn đo — AC16)*,
đường dẫn đi vào qua biến môi trường `AURA_PW`.

🔴 **Phép đo BÁC đề xuất mặc định (a) của story.** Số làn mà (a) đòi **không có cận trên**: nó
bằng số vạch chồng nhau đồng thời nhiều nhất. Fixture văn xuôi trộn đòi **3** làn *(vừa đủ máng
22px)*; fixture **đối thoại** đòi **5** làn, mép phải **30px** ⇒ **tràn khỏi máng**, đúng chỗ
chữ bắt đầu. Bảng số đầy đủ: `2-5-ban-do/README.md`.

⚠️ **Một vết trong chính bàn đo, ghi ra thay vì sửa im lặng.** Bản đầu gom vạch chồng nhau
**bắc cầu** rồi phát làn theo thứ tự trong nhóm; lượt chạy đầu bác nó ngay *(nhóm năm phần tử ⇒
bốn phần tử sau dồn hết vào làn 1 ⇒ vẫn một vạch bị che)*. Phép đúng là **tô màu đồ thị khoảng**.
Đúng lớp lỗi *"trúng tiền đề chưa phải trúng cơ chế"* mà §Bài học đã ghi.

### Completion Notes List

#### Bảy quyết định — cả bảy đã cài đúng chữ ký

| \# | Chốt | Cài ở đâu | Lưới |
| --- | --- | --- | --- |
| 1 | (a) dời con trỏ | `editorPanelState.ts::confirmCurrentSegment` ② + watcher `editorCaretPlacement` ở `EditorPanel.vue` | `editorConfirmSegment.test.ts` §② · e2e ⑤ |
| 2 | (a) chia làn | `editorGutter.ts::assignGutterLanes`, `left` qua `:style` | `editorGutterLanes.test.ts` · bàn đo 4 ảnh |
| 3 | (a) giữ *không vạch* | doc-comment `resolveSegmentRule` | `editorSegmentRule.test.ts` |
| 4 | `editor.confirm_segment` + `Mod+Enter` | `commands/index.ts` | `check:commands` Kiểm B/E |
| 5 | `status TEXT DEFAULT 'draft'` | `schema.rs::SEGMENT_STATUS_AND_VERSION_DDL` | `segment_contract.rs` |
| 6 | một bước **7**, kèm `segment_version` | như trên | như trên |
| 7 | từ chối câu chưa dịch | `confirm_segment` ②, `MessageKey::SegmentNothingToConfirm` | `every_refusal_of_confirm_...` |

#### 🔴 Ba thứ PHÉP ĐO bác, và cả ba đã sửa thay vì ghi nợ

**① Đề xuất mặc định của Quyết định #2 không đóng kín.** Bước làn **cố định 5px** *(đúng hình
dạng story mô tả)* chỉ chứa 3 làn; fixture đối thoại đòi **5** ⇒ mép phải 30px, **tràn** khỏi máng
22px. Vá bằng **bước co** `clamp(2, 5, ⌊12/(N-1)⌋)` ⇒ 5 làn nằm gọn ở 22px, **0 vạch bị che**.
Ice ký (a) **sau khi** đã đọc số bác nó, nên đây là quyết định của Ice và story đi tiếp với (a).

**② Thứ tự** ***hạ-trước-ghi-sau*** **nằm ở vỏ** `wire` **⇒ không cổng nào đỏ khi đảo nó.** Đo: đảo hai dòng
cho **54/54 xanh**, vì `tests/**` gọi một vỏ cần `AppHandle` không được. ⇒ Kéo xuống hàm thuần
`flush_segment_targets`, và cổng `the_flush_path_lowers_the_state_before_it_writes_the_new_text
`nay đỏ khi đảo.

**③** `assignGutterLanes` **bản đầu là** `O(n²)` **và vỡ NFR2.** Tới hết 2.3, `wanted` có nhiều nhất
**một** phần tử; 2.5 phá đúng giả định đó *(nay chứa mọi câu đã xác nhận)*. Đo 2026-08-14, Node
22.22.2, 9.850 vạch, ba lượt: `O(n²)` **482,4 / 254,5 / 261,6 ms** — vượt trần 50 ms **5–10 lần**;
quét đường **8,3 / 5,2 / 4,3 ms**, kết quả **giống hệt**.

#### 🔴 Bộ e2e bắt một khuyết tật mà CẢ BỐN đường kia bỏ lọt

`read_open_chapter_segments` **không gửi** `status` **qua dây**: kiểu TS khai có, `segmentRuleInputOf
`đọc nó, nhưng struct Rust và câu `SELECT` thiếu nó ⇒ `isConfirmed` **luôn** `false` trong app thật.
Nó đi lọt **74/74** test frontend vì fixture vitest dựng `ChapterSegment` **bằng tay** và có cấp
`status`. ⇒ Đã vá, và lưới mới là `the_load_command_carries_the_status_column_over_the_wire` — nó
đi qua **chính lệnh đọc của sản phẩm**, và nó canh cả vế *"giá trị đi theo dữ liệu thật"*, không
chỉ vế *"trường có tồn tại"*.

#### Đỏ-rồi-xanh — tám cổng mới, mỗi cổng một lượt có ghi lại (2026-08-14)

| Cổng | Đường sai đã tiêm | Kết quả |
| --- | --- | --- |
| `a_flush_touches_..._nothing_else` | nhét `status` vào `UPDATE` của auto-save | ĐỎ → hoàn nguyên → XANH |
| `confirming_an_already_confirmed_...` | gỡ nhánh no-op AC13 | ĐỎ → XANH |
| `the_flush_path_lowers_the_state_...` | đảo thứ tự trong hàm thuần | ĐỎ → XANH |
| `the_raw_column_reader_sees_every_column...` | thêm một cột thật, không sửa `SegmentRow` | ĐỎ → XANH |
| `the_load_command_carries_the_status_...` | gỡ `status` khỏi `SELECT` + dựng bằng hằng | ĐỎ → XANH |
| `editorConfirmSegment` §① | đảo `flush` / `confirm` | 2 ca ĐỎ → XANH |
| `editorSegmentRule` *(hai chỗ đọc đồng ý)* | sai chính tả `'Confirmed'` ở một chỗ | 2 ca ĐỎ → XANH |
| `editorGutterLanes` | ① đóng băng bước 5px ② gom bắc cầu | 4 ca / 5 ca ĐỎ → XANH |

#### Nghiệm thu — số THẬT, không suy ra (2026-08-14)

| Đường | Trước story | Sau story |
| --- | --- | --- |
| 11 cổng `npm` | 11 xanh | **11 xanh** *(gồm *`check:scope`* + *`check:scope:bundled`* chạy tay)* |
| `npm run build` | xanh | **xanh** |
| `npm run test` | 41/41 | **74/74** *(+33)* |
| `cargo test --locked` | 324 / 0 đỏ / 5 bỏ qua | **336 / 0 đỏ / 5 bỏ qua** *(+12)* |
| `npm run test:e2e` *(spec 2.5)* | — | **2/2 xanh** |

⚠️ **Sàn quần thể KHÔNG phải xét lại:** không tệp mới nào vào `src/**` hay `src-tauri/src/**`.
Tệp mới chỉ ở `tests/frontend/**`, `e2e/specs/**` và `_bmad-output/**`.

#### 🔴 Thứ story này KHÔNG tự chấm đạt

- **NFR2** — số ở trên đo bằng **Node**, không WKWebView, và không gồm `getClientRects()` lẫn lượt

bố cục. Vế *"một frame thật trên Chương đã dịch xong"* **chưa ai đo**. Chủ: **Story 2.4**.

- **AD-35 vế (c)** cưỡng chế được ở **một** chỗ gọi, không ở mọi chỗ gọi tương lai — chi tiết ở

`deferred-work.md`.

- **Task 1.1** *(gõ tay vào một câu chưa dịch trên app thật)* — cần mắt và tay của Ice.
- `EXPERIENCE.md:105-113` chưa có hàng của Quyết định #3: sửa tài liệu quy hoạch là lượt riêng

của Ice.

### File List

**Rust — sản phẩm**

- `src-tauri/src/core/store/schema.rs` — `SEGMENT_STATUS_AND_VERSION_DDL` (bước 7); sửa tại chỗ hai doc-comment đã hết đúng
- `src-tauri/src/core/store/mod.rs` — tái xuất hằng DDL mới
- `src-tauri/src/core/i18n/mod.rs` — ba `MessageKey` mới, khai bằng `message_keys!`
- `src-tauri/src/commands/segment.rs` — `ConfirmOutcome` · `confirm_segment` · `unconfirm_edited_segments` · `flush_segment_targets` · `ChapterSegment.status` · vỏ `wire::confirm_segment`
- `src-tauri/src/lib.rs` — đăng ký `confirm_segment` vào `invoke_handler`

**Rust — test**

- `src-tauri/tests/segment_contract.rs` — 12 ca mới; `SegmentRow` 9 → 10 cột
- `src-tauri/tests/pinned_contract.rs` — nâng hai hằng neo (5 → 6 bước, 6 → 7 phiên bản)

**TypeScript / Vue**

- `src/config/segment.ts` — `ChapterSegment.status` · `isSegmentConfirmed` · `ConfirmOutcome` · `confirmSegment`
- `src/panels/editorSegments.ts` — `isConfirmed` đọc dữ liệu thật; hai doc-comment
- `src/panels/editorGutter.ts` — `GutterRule.left` · `assignGutterLanes` (quét đường)
- `src/panels/editorPanelState.ts` — `confirmCurrentSegment` · `editorConfirmError` · `editorCaretPlacement`
- `src/panels/EditorPanel.vue` — watcher đặt caret · `left` qua `:style` · `.gmark` bỏ `left` cứng
- `src/commands/index.ts` — dep `confirmSegment` · đăng ký `editor.confirm_segment`
- `src/main.ts` — nối cổng tiêm
- `src/i18n/vi.json` — bốn khoá mới

**Test frontend / e2e**

- `tests/frontend/editorSegmentRule.test.ts` *(mới)*
- `tests/frontend/editorConfirmSegment.test.ts` *(mới)*
- `tests/frontend/editorGutterLanes.test.ts` *(mới)*
- `tests/frontend/support/segmentFixture.ts` — fixture mang `status`
- `e2e/specs/editor-confirm-segment.e2e.mjs` *(mới)*

**Bàn đo và tài liệu**

- `_bmad-output/implementation-artifacts/2-5-ban-do-hai-vach.html` *(mới)*
- `_bmad-output/implementation-artifacts/2-5-ban-do/` *(mới — *`chup.mjs`* · *`README.md`* · 4 PNG · *`bao-cao.json`*)*
- `_bmad-output/implementation-artifacts/deferred-work.md` — đóng 4 món, mở 8 món có chủ
- `_bmad-output/implementation-artifacts/sprint-status.yaml`
- `_bmad-output/implementation-artifacts/2-5-xac-nhan-segment-va-may-trang-thai.md`

### Change Log

| Ngày | Nội dung |
| --- | --- |
| 2026-08-14 | Task 0 — bảy quyết định trình cho Ice kèm bàn đo Quyết định #2; **cả bảy đã ký** |
| 2026-08-14 | Bàn đo bác đề xuất mặc định của Quyết định #2 *(bước cố định 5px tràn máng ở ca đối thoại)*; Ice ký (a), cài bằng **bước co** |
| 2026-08-14 | Bước di trú **7** — `segment.status` + bảng `segment_version` |
| 2026-08-14 | Máy trạng thái AD-31 ở Rust: `confirm_segment` · `unconfirm_edited_segments` · `flush_segment_targets` |
| 2026-08-14 | Chia làn vạch lề *(quét đường — vá một hồi quy NFR2 đo được: 254–482 ms → 4–8 ms)* |
| 2026-08-14 | e2e bắt khuyết tật *"*`status`* không qua dây"* mà cả bốn đường kia bỏ lọt; vá + dựng lưới |
| 2026-08-14 | Nghiệm thu: 11 cổng · build · 74/74 vitest · 336/0/5 cargo · 2/2 e2e |
| 2026-08-14 | **Code review ba lớp** — 7 phát hiện, 0 loại làm nhiễu. **Quyết định #8 Ice ký** *(còn dơ sau hai lượt flush ⇒ từ chối ký)*. Cả 7 đã vá; 2 vế không nghiệm thu được đi vào `deferred-work.md` kèm chủ *(Story 2.10 · Ice/UX-DR30)* |
| 2026-08-14 | Bốn lưới mới, cả bốn chạy **đỏ-rồi-xanh** có ghi lại: 2 ca Rust *(*`the_flush_path_refuses_an_unknown_id_before_it_lowers_a_single_signature`* · *`a_target_of_only_whitespace_is_refused_exactly_like_an_empty_one`*)* · 2 nhóm ca vitest *(Quyết định #8 · khoá chống-gọi-lại)* |
| 2026-08-14 | Nghiệm thu sau rà: 11 cổng · build · **78/78** vitest · **338/0/5** cargo |
| 2026-08-14 | **e2e chạy lại sau bảy bản vá.** `editor-confirm-segment.e2e.mjs` **2/2 xanh** chạy riêng, exit 0 thật. Lượt chạy CẢ BỘ cho *3 passed / 3 failed*, nhưng chạy từng tệp một thì `attribution-focus` xanh, `editor-confirm-segment` xanh 2/2, `editor-typing-flush` 1 xanh / 1 đỏ — **đúng bằng mốc Story 2.3 đã ghi**. Ca đỏ duy nhất là ca **đã có chủ**, ghi tại chỗ ở `editor-typing-flush.e2e.mjs:133-145` (`execCommand('insertText')` trả `false` trên câu chưa dịch, chủ: Story 2.3). **Không bản vá nào của lượt rà gây hồi quy.** Chênh lệch giữa hai cách chạy là một khuyết tật của bộ e2e, đã ghi nợ có chủ |

### Review Findings

*Lượt rà 2026-08-14 (*`bmad-code-review`*, ba lớp song song trên *`git diff HEAD`*, baseline *`8245a17`*).
Blind Hunter trượt lượt đầu vì treo watchdog; phóng lại với diff chẻ tư và về đủ. Không lớp nào thiếu.*

- [x] [Review][Patch] **Lượt ký dùng văn bản CŨ khi một ký tự bay xen giữa lượt flush — AD-35 vế (c) hở.** 🔴 **Quyết định #8, Ice ký 2026-08-14: đường (3) — thử lại MỘT lượt, còn dơ nữa thì TỪ CHỐI lượt ký.** Đóng ca thường (một ký tự lẻ) mà không phải đặt một trần lặp mới; giữ đường từ chối cho ca bệnh lý. Chi tiết khuyết tật: — `confirmCurrentSegment` chỉ kiểm mã kết quả của `flushEditorNow()` chứ không kiểm tập chờ có **sạch** hay không. Nhánh *originator* của `flushEditorNow` (lượt gọi đầu, tức đúng đường `Mod+Enter`) chụp `snapshot` **trước** lượt IPC và, khi lô về, chỉ `armFlushTimer(0)` rồi trả `'saved'` — nó **không** đệ quy như nhánh *joiner* ở `editorPanelState.ts:283-284`. Ký tự gõ trong lúc lô bay nằm ngoài snapshot, vẫn `isDirty()`, nhưng `'saved' !== 'failed'` nên `confirm_segment` chạy tiếp và ký **văn bản trên đĩa**, tức bản thiếu ký tự cuối. Hệ quả kép: (1) một `SegmentVersion` mang văn bản **chưa bao giờ ở trên màn hình** đi vĩnh viễn vào lịch sử FR101; (2) timer vừa lên dây flush nốt ký tự sót qua `flush_segment_targets`, mà hàm đó **hạ trạng thái trước** ⇒ câu vừa ký **tự trở về** `draft` vài mili-giây sau. Lưới không bắt được: vitest mock trả lời tức thời và không tiêm `noteEditorEdit` xen giữa; spec e2e tự khai **không đo vế (c)** vì bơm chữ thẳng qua `save_segment_targets`. Hai đường sửa đều hợp lệ và khác nhau về UX ⇒ Ice chốt. `src/panels/editorPanelState.ts:517-530` · `:273-334` · `src/panels/EditorPanel.vue:863-866`
- [x] [Review][Patch] Lô flush có id lạ: chữ ký bị hạ ở giao dịch 1 rồi lô bị từ chối ở giao dịch 2 ⇒ mất `confirmed` mà văn bản KHÔNG được ghi [src-tauri/src/commands/segment.rs:793-799]
- [x] [Review][Patch] `confirm_segment` kiểm `target_text.is_empty()` không `trim()` ⇒ câu chỉ có khoảng trắng lọt qua Quyết định #7 [src-tauri/src/commands/segment.rs:725]
- [x] [Review][Patch] `confirmCurrentSegment` không có khoá chống-gọi-lại ⇒ bấm `Mod+Enter` hai lần nhanh kéo caret về offset 0 của câu kế tiếp lần thứ hai. Watcher `editorCaretPlacement` gọi `clearEditorCaretPlacement()` ngay dòng đầu nên nó là watcher **một phát**: lượt gán thứ hai cùng giá trị **vẫn là** một lượt đổi và **vẫn bắn**. Khuôn `inFlight` đã có sẵn ngay trong tệp cho `flushEditorNow` [src/panels/editorPanelState.ts:517-557] [src/panels/EditorPanel.vue:421-431]
- [x] [Review][Patch] Khe hở AC1 ở câu cuối Chương đã tự thú trong chú thích nhưng chưa vào `deferred-work.md` kèm chủ [src/panels/editorPanelState.ts:503-504]
- [x] [Review][Patch] **Ba khoá lỗi mới KHÔNG tới được màn hình, và một chú thích khai ngược lại.** `editorConfirmError` được export nhưng **không nơi nào render**; `ConfirmResult` bị vứt ở chỗ gọi duy nhất (`main.ts:229` — `void confirmCurrentSegment()`). ⇒ Hôm nay bấm `Mod+Enter` trên một câu chưa dịch thì **không gì xảy ra trên màn hình**: cả ba khoá `err.segment.*` vừa dựng đều tới hư không. AC14 **vẫn đạt** (nó nói về hợp đồng IPC, và Rust trả đúng `IpcError` phân biệt được) — nhưng doc-comment `editorPanelState.ts:439-441` khai *"*`'refused'`* mang lỗi ra bằng *`editorConfirmError`*… chỗ gọi hiển thị bằng *`tError()`*"*, và **chỗ gọi đó không tồn tại**. Cần: một mục sổ nợ **có chủ** cho vế hiển thị, và sửa chú thích cho nó thôi nói một điều chưa đúng. [src/panels/editorPanelState.ts:439-446] [src/main.ts:229]
- [x] [Review][Patch] Ba con trỏ tài liệu, hai cái trỏ vào tệp test KHÔNG tồn tại và bịa ra thư mục con mà cây test cố ý không dùng [src/panels/editorSegments.ts:136] [src/panels/editorSegments.ts:173] [src/config/segment.ts:317]
