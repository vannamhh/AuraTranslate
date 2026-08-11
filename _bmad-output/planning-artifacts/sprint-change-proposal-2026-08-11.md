# Sprint Change Proposal — rà soát tài liệu vs mã nguồn

**Ngày:** 2026-08-11 · **Chủ dự án:** Ice · **Chủ trì:** Amelia (Developer)
**Chế độ:** Incremental — mỗi đề xuất trình riêng, Ice duyệt từng cái
**Baseline:** `8a9992b` *(master)* · cây làm việc sạch lúc bắt đầu

---

## 1. Vấn đề

Ice yêu cầu đối chiếu tài liệu với mã đã triển khai, và nói thẳng rằng **không nắm cụ thể
chỗ nào đã trôi**. Nên trigger của lượt này không phải một sự cố — nó là một nghi ngờ có
căn cứ, và việc đầu tiên là **đo để tìm ra trigger thật**, không phải bàn phương án.

**Loại vấn đề:** lệch giữa tạo tác quy hoạch và mã — cụ thể là *năng lực dựng ra trong lúc
code mà không đi qua tầng quy hoạch*.

**Phương pháp:** không lấy một khẳng định nào của story file làm đúng sẵn. Đọc trọn
`ARCHITECTURE-SPINE.md` *(857 dòng, 44 AD lúc bắt đầu)*, quét `epics.md` *(6.409 dòng)*,
chạy lại chín cổng, đối chiếu `package.json` · `Cargo.toml` · `capabilities/` ·
`tauri.conf.json` · `ci.yml` · `.githooks/pre-push` với thứ tài liệu khai.

### Nền đo được — thứ KHÔNG trôi

Ghi trước, vì nó là điều kiện để phần còn lại của báo cáo này đáng tin.

| Phép đo | Kết quả |
|---|---|
| Chín cổng bằng máy | **9/9 XANH** |
| Cây git lúc bắt đầu | **sạch** |
| `capabilities/` | đúng **một** tệp `main.json` · 3 permission · **0** permission plugin |
| `Cargo.toml` | **không** có `default = [...]` — `wdio` là feature tắt |
| Ba story `in-progress` | story file **khớp** `sprint-status.yaml` |
| Bao phủ FR Epic 1 | **27/27** *(xác nhận lại từ retrospective)* |

Không tìm được một khai sai nào trong các bảng số của story file.

### Một đính chính của Ice đã đổi khung phân tích

Bốn tạo tác đang chép quyết định ngày 2026-08-11 thành *"Ice chốt **BỎ QUA** GitHub
Actions"*. Ice đính chính trong lượt này: đó là **TẠM DỪNG**, lý do là **không có máy
Windows để đối chiếu kết quả runner**.

Khác biệt không phải chữ nghĩa. §10 của retrospective đang giao cho Epic 2 một điều kiện
khởi hành mang chữ *"nay KHÔNG có đường nghiệm thu nào"* như một **trạng thái đã chốt**,
trong khi đúng ra nó là một khoảng mù **có điều kiện mở lại** — và điều kiện đó chính là
món nợ **A5** đang chờ chủ. Hai cách đọc dẫn tới hai hành động khác nhau ở Story 2.4, nơi
NFR14 là một mệnh đề hai nền tảng.

---

## 2. Phân tích tác động

### 2.1 Tác động Epic

| Epic | Tác động |
|---|---|
| **Epic 1** | Story 1.3 nhận **hai AC mới**; **Story 1.22** thêm vào cuối epic. Epic vẫn hoàn thành được theo kế hoạch gốc — không AC nào bị gỡ, không phạm vi nào bị thu hẹp |
| **Epic 2** | **Không đổi kế hoạch.** Điều kiện khởi hành §10 của retrospective được phát biểu lại cho đúng *(khoảng mù có điều kiện mở lại, không phải vĩnh viễn)* |
| **Epic 10** | Story 10.1 nhận một **điều kiện khởi hành**, không đụng AC. FR107 **không sửa** — xem §3 |
| Epic 3–9 | **Không tác động.** Không phát hiện nào của lượt này chạm phạm vi của chúng |

### 2.2 Tác động Story

| Story | Thay đổi |
|---|---|
| **1.3** | +2 AC: ba danh sách cổng phải khai cùng bộ · đường cưỡng chế lúc GitHub Actions vắng mặt |
| **1.22** *(mới)* | Bộ chạy e2e trong webview thật — `in-progress`, ba giới hạn còn mở |
| **10.1** | +1 điều kiện khởi hành |
| 1.20 · 1.21 | Không đụng. Nợ 28 hàng bàn đo vẫn thuộc **A4** |

### 2.3 Xung đột tạo tác

| Tạo tác | Phát hiện | Xử |
|---|---|---|
| `ARCHITECTURE-SPINE.md` | **8 chỗ** lỗi thời so với mã | Đã đồng bộ, `lint_spine.py` **0 findings** |
| `epics.md` | **0** dòng nhắc e2e / eslint / wdio, trong khi bốn năng lực đã sống trong mã | Story 1.3 +2 AC · Story 1.22 mới |
| `sprint-status.yaml` | A2 chép sai quyết định · thiếu story 1.22 | Đã sửa · đã thêm |
| `epic-1-retro-2026-08-11.md` | §9 A2 và §10 mục 2 chép sai quyết định | Đã sửa |
| `.githooks/pre-push` | §Giới hạn chép sai quyết định | Đã sửa |
| `prd.md` | FR107 dựng trên GitHub Actions | **KHÔNG sửa** — ghi nợ, xem §3 |
| `ux-designs/` | Không xung đột | — |

### 2.4 Tác động kỹ thuật

Một thay đổi mã duy nhất: `scripts/check-gates.mjs`. Không chạm mã sản phẩm, không chạm
lược đồ, không thêm phụ thuộc.

---

## 3. Hướng đi được chọn

**Option 1 — Direct Adjustment.** Effort **thấp** · Risk **thấp**.

Rollback không xét tới: không có công việc nào cần hoàn nguyên — mã hiện tại đúng, thứ
thiếu là **tạo tác quy hoạch nhận nó**. MVP Review không xét tới: không FR nào rơi, không
phạm vi nào phải thu hẹp; lượt này **thêm** trách nhiệm chứ không bớt năng lực.

**Lý do chọn, và chỗ nó suýt không được chọn:** phần lớn công việc là **chép lại thứ đã
chạy thật** vào đúng chỗ tài liệu. Đúng một mục đi xa hơn thế — **AD-45** thêm một bất
biến mới — và nó được chọn vì cơ chế đã có trong mã *(hai lớp chặn)* và đã có cổng canh
*(`check-deps.mjs` Kiểm 1b)*: AD-45 đặt tên cho một luật đang chạy, không đặt việc mới.

### Ba quyết định có thể gây tranh cãi, ghi thẳng lý do

**① KHÔNG sửa FR107 hôm nay.** Ice chốt: ghi nợ, hoãn tới Epic 10. Tạm dừng không phải
bỏ, và Epic 10 còn cách **chín epic**. Sửa một FR dựa trên một ràng buộc có thể đã biến
mất trước lúc Epic 10 tới là đổi tài liệu bằng **phỏng đoán** — đúng thứ doctrine *"đo
trước khi tin"* cấm. Thay vào đó Story 10.1 mang một điều kiện khởi hành với hai câu hỏi
phải trả lời trước khi dựng.

**② KHÔNG dựng bốn story cho bốn năng lực.** Ba trong bốn nằm gọn trong hiến chương sẵn
có của Story 1.3 — AC4 của nó viết bằng chữ *"các luật cưỡng chế bằng test sinh ra ở epic
sau… gắn vào **chính pipeline này**"*. Hai cổng mới là hai **thể hiện** của luật đó. Thứ
AC4 chưa phủ là **số lượng danh sách cổng** và **đường cưỡng chế lúc CI vắng mặt** — đó là
nội dung hai AC mới. Chỉ bộ e2e khác bản chất *(một năng lực nghiệm thu, phục vụ nợ xuyên
chín story, và chưa xong)* nên nó thành story riêng.

**③ Story 1.22 dựng SAU khi mã đã viết — ngược chiều, và không giả vờ là bình thường.**
Lý do vẫn dựng: bộ e2e mang **ba khuyết tật có tên** mà trước lượt này không tạo tác nào
chịu trách nhiệm. Bỏ qua thì chúng sống trong một tệp `.md` đề xuất, và Epic 2 sẽ dựng
Panel Editor — bề mặt thị giác lớn nhất dự án — lên trên một nền như vậy.

---

## 4. Các thay đổi cụ thể

### Đề xuất 1 — đính chính hồ sơ về GitHub Actions ✅ đã áp

Bốn chỗ: `sprint-status.yaml` (A2) · `epic-1-retro-2026-08-11.md` (§9 hàng A2, §10 mục 2)
· `.githooks/pre-push` (§Giới hạn). *"BỎ QUA"* → *"TẠM DỪNG, lý do: không có máy Windows
để đối chiếu"*, kèm mệnh đề **có điều kiện mở lại**.

### Đề xuất 2 — cổng thứ mười một canh HAI trong BA danh sách ✅ đã áp

**Phép đo:** `package.json` khai **11** script `check:*`, `.githooks/pre-push` chạy **9**.
Chênh 2 có lý do thật, nhưng lý do chỉ nằm trong một khối chú thích và **không phép kiểm
nào buộc nó đúng**. Ngày mai thêm cổng thứ mười hai mà quên hook là lặp lại nguyên vẹn sự
cố `check:lint`, chỉ đổi tệp bị quên.

**Vá `scripts/check-gates.mjs`:** thêm **Kiểm D** *(cổng thiếu trong hook)* và **Kiểm E**
*(hook gọi cổng không tồn tại)*, đối xứng cặp A/B sẵn có, cộng `PREPUSH_EXEMPT` — mỗi miễn
trừ kèm lý do chép từ chính §Phạm vi của hook.

🔴 **Chi tiết đắt nhất:** bộ đọc trả `null` khi không phân giải nổi vòng lặp
`for gate in … ;`, **không** trả tập rỗng. Một bộ đọc trả rỗng làm Kiểm D xanh trong khi
nó chẳng kiểm gì — đúng lớp lỗi *"rỗng im lặng"* mà AD-26 và AD-44 ④ tồn tại để cấm.
`null` buộc `abort`, tức một lỗi hạ tầng tường minh.

**Nghiệm thu đỏ-rồi-xanh — bốn ca chạy thật trên bản sao ngoài kho:**

| Ca | Kết quả |
|---|---|
| Thêm cổng thứ 12 vào `package.json` + `ci.yml`, quên hook | **A và B XANH, D ĐỎ** — đúng hình dạng sự cố đã xảy ra |
| `for gate in` → `for g in` | **abort, exit 1** — không xanh oan |
| Hook gọi `check:da-bi-xoa` | **E ĐỎ** |
| Khôi phục cả ba tệp | **exit 0** |

Sau bản vá: chín cổng vẫn **9/9 XANH**.

⚠️ **Chỗ căng đã ghi vào chính tệp thay vì giấu:** dòng kết của `check-gates.mjs` in ra
*"AC4 của Story 1.3 — MỘT pipeline duy nhất"*, và hook `pre-push` **LÀ** một đường cưỡng
chế thứ hai. AC4 cấm bằng chữ một **tệp workflow** thứ hai nên hook không phạm chữ; nhưng
tinh thần AC4 *(một danh sách, không dựa trí nhớ)* chỉ còn đúng **KHI có Kiểm D**. Bản vá
không xin ngoại lệ khỏi AC4 — nó là điều kiện để AC4 tiếp tục đúng dưới ba danh sách.

### Đề xuất 3 — đồng bộ spine với mã, 8 sửa ✅ đã áp

| # | Sửa |
|---|---|
| 1 | `updated: '2026-08-05'` → `'2026-08-11'` |
| 2 | Bảng Stack **+10 phụ thuộc** |
| 3 | Đoạn *Rà NFR15 lượt ba — 2026-08-11* |
| 4 | **AD-45 mới** — bản phát hành không mở một cổng LẮNG NGHE nào |
| 5 | §*"Không dùng, đã loại có lý do"* — kho không còn **0** plugin Tauri |
| 6 | `scripts/check-deps.sh` → `.mjs` |
| 7 | Cây nguồn **+5 nhánh**: `src/config/` · `src/selftest/` · `scripts/` · `e2e/` · `.githooks/` |
| 8 | Hàng *Cổng lắng nghe* trong bảng Consistency Conventions |

**Về bảng Stack:** chính spine đặt luật *"mỗi phụ thuộc mới phải rà GPLv3 và **ghi vào
bảng Stack**"*. Bảy trong mười hàng mới sinh ra rồi mới được ghi — `uuid` từ Story 1.15,
ba hàng ESLint từ cổng thứ mười, năm gói WebdriverIO cùng plugin từ bộ lái e2e. Quy ước đó
bị bỏ lỡ **ba lần liên tiếp**.

Rà lượt ba theo đúng phương pháp hai lượt trước — **mở tệp `LICENSE` trong nguồn đã tải mà
đọc**, không tin nhãn registry: **10/10 mang ✓**, thân tệp đều có mệnh đề *"Permission is
hereby granted, free of charge"*; `uuid` là MIT OR Apache-2.0.

⚠️ **Hai mục phải nói thẳng, cả hai ở phần BẮC CẦU chứ không phải hàng Stack:** cây npm đi
**194 → 530** gói; `@promptbook/utils` mang **CC-BY-4.0** *(đòi ghi công)* và
`css-value@0.0.1` **không khai giấy phép**. Cả hai chỉ devDependency, không vào sản phẩm —
nhưng chúng là hai mục duy nhất trong 530 gói không thuộc nhóm dễ dãi.

**Về AD-45:** AD-15 đếm điểm **RA** mạng và không nói một chữ nào về chiều ngược lại, nên
một máy chủ nghe trên `localhost` đi vào bản người dùng cài mà **không phạm AD-15**. Cái
bẫy phản trực giác: một `#[cfg(debug_assertions)]` đơn độc **không đủ** — nó loại **mã**,
không loại **phụ thuộc**. AD-45 đòi **hai** lớp chặn cùng lúc và trỏ vào cổng đã canh sẵn.

**Nghiệm thu:** `lint_spine.py` → **0 findings** · 45 AD · bảng Stack 31 hàng.

### Đề xuất 4 — bốn năng lực không story ✅ đã áp

`grep -ni "e2e|webdriver|eslint|wdio" epics.md` → **0 kết quả**, trong khi bốn thứ sau
sống trong mã: cổng thứ mười `check:lint` (`01be1c2`) · cổng thứ mười một `check:gates`
(`b53002f`) · bộ lái e2e (`3a54628`, `7127f5f`) · hook `pre-push` (`8a9992b`).

**4a — Story 1.3 nhận hai AC mới:** ba danh sách cổng phải khai cùng một bộ *(mỗi miễn trừ
kèm lý do đọc được tại chỗ; bộ đọc không phân giải nổi thì dừng bằng lỗi hạ tầng)* · đường
cưỡng chế lúc GitHub Actions tạm dừng là `pre-push`, và nó **CHẶN** chứ không báo cáo.

**4b — Story 1.22 mới:** *Bộ chạy e2e trong webview thật.* Sáu AC, gồm ba giới hạn còn mở
và một AC khoá phạm vi *(dựng **đường**, không viết trọn 28 hàng)*.

### Đề xuất 5 — ghi nợ và đóng sổ ✅ đã áp

- `epics.md` Story 10.1 — điều kiện khởi hành, hai câu hỏi phải trả lời trước khi dựng
- `deferred-work.md` — mục *correct-course 2026-08-11*: đã đóng · còn mở · ngoài phạm vi
- `sprint-status.yaml` — story `1-22`, 5 action item mới, A2 sửa lại

---

## 5. Bàn giao

**Phân hạng: MODERATE.** Nó chạm ba tạo tác quy hoạch *(epics · architecture · sprint
status)* và thêm một story, nhưng **không** chạm PRD, **không** đổi phạm vi MVP, và
**không** đòi replan.

### Việc đã xong trong chính lượt này

| Việc | Nghiệm thu |
|---|---|
| Đính chính hồ sơ GitHub Actions, 4 chỗ | đọc lại 4 tệp |
| `check-gates.mjs` Kiểm D + E | 4 ca đỏ-rồi-xanh · 9/9 cổng xanh |
| Đồng bộ spine, 8 sửa | `lint_spine.py` 0 findings |
| Story 1.3 +2 AC · Story 1.22 mới | `epics.md` |
| Ghi nợ FR107 · sổ nợ · sprint status | YAML hợp lệ · 14 action item, 7 open |

### Việc bàn giao đi

| # | Việc | Chủ | Điều kiện xong |
|---|---|---|---|
| **C1** | 🔴 Chỉ `$APPDATA` của app con sang thư mục tạm mỗi lượt e2e — **trước khi dựng thêm hàng bàn đo nào** | Dev | AC 2 của Story 1.22 xanh |
| **C2** | Tương tác có thứ tự đi Actions API, không `element.click()` | Dev | AC 3 của Story 1.22 |
| **C3** | Cổng theo worker, hoặc ghi giới hạn một-spec-một-phiên ra chỗ đọc được | Dev | AC 4 của Story 1.22 |
| **C4** | FR107 — kiểm lại ở Epic 10, hai câu hỏi ở điều kiện khởi hành Story 10.1 | Ice | Story 10.1 dựng ra đã trả lời cả hai |
| **C5** | AD-23 lệch `tauri.conf.json` | Ice | AD-23 khớp cấu hình thật |
| **C6** | Sơ đồ AD-13 còn cạnh `dict --> matching` · bảng Porter AD-44 ③ chưa thay bằng số đo Story 1.12 | Winston | spine khớp thân Rule |

### Ngoài phạm vi lượt này, ghi thẳng

Lượt này đối chiếu **tài liệu quy hoạch với hình dạng mã** — bảng Stack, bất biến kiến
trúc, cây nguồn, danh sách cổng, bao phủ story. Nó **KHÔNG** đối chiếu từng AC của 25
story với hành vi thật của mã; phép đó cần chạy lại 28 hàng bàn đo thị giác và một máy
Windows, tức đúng hai món nợ **A4** và **A5** đang chờ chủ. Không lượt đọc tài liệu nào
thay được hai món đó, và lượt này không giả vờ thay.
