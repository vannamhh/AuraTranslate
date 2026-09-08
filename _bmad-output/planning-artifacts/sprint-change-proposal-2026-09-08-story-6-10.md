# Sprint Change Proposal — 2026-09-08 · Story 6.10

**Người soạn:** `bmad-correct-course` (chế độ Batch) · **Người duyệt:** Ice
**Baseline:** `24d07c4` (master, đã đẩy) · `grep -c "^### AD-"` = **48**
**Hồ sơ điều tra kèm phép đo:** `_bmad-output/implementation-artifacts/ho-so-dieu-tra-6-10-bo-loc-can-xem-2026-09-08.md`

---

## 1. Vấn đề

Story 6.10 (*Bộ lọc "cần xem"*, phủ FR132) chưa bắt đầu. Lượt điều tra ngày 2026-09-08, chạy ở bước 2 của
`bmad-build` trước khi soạn spec, đo được rằng **AC của nó đứng trên một năng lực chưa dựng**: màn hình xem
trước hôm nay chưa bao giờ đa-Chương thật.

**Phép đo dẫn tới mục này.** `preview_import_encoding` gặp `PipelineShape::Chapters` thì chốt từ đơn vị ĐẦU
(`src-tauri/src/commands/project.rs:1891-1893`), rồi `encoding_candidate_wire` gói lại thành
`PipelineShape::Blob(ChapterInput::RawBytes { bytes: full_bytes, label })` (`:1749-1752`). ⇒ Trên **đường
URL**, cả bốn tầng xem trước được tính từ byte của **đúng link đầu tiên**. Một lượt dán 50 link cho ra một
màn xem trước của **một** Chương.

Đối chiếu bốn nguyên nhân AC đòi với thứ thật sự tồn tại:

| Nguyên nhân | Đo được | Neo |
|---|---|---|
| ① bảng mã tin cậy thấp | Có, nhưng **cấp lượt nhập**, không cấp Chương — Ice chốt 2026-09-06 *"một bảng mã cho cả danh sách"* | `core/segment/encoding.rs:122` |
| ② bóc ra ngắn bất thường so với **trung vị** | Độ dài per-Chương **có**; trung vị **0 dòng mã** trong cả kho | `project.rs:1189` |
| ③ luật làm sạch xoá quá nhiều | Span xoá **có**; per-Chương chỉ tồn tại trên đường `Chapters`, mà đường đó bị gói về `Blob` | `core/cleanup/mod.rs:140` · `project.rs:1749` |
| ④ link hỏng | 8 lý do **có**, nhưng còn một mục hỏng thì `chapters_shape_if_all_ok` trả `None` ⇒ **không màn xem trước nào cả** | `core/webimport/mod.rs:66-84` · `project.rs:2362-2371` |

**Loại vấn đề:** *technical limitation discovered during implementation* — không phải yêu cầu mới, không phải
hiểu sai spec ban đầu.

🔴 **Đây KHÔNG phải một chỗ lệch spec.** `AGENTS.md` §Conventions: *"Năng lực chưa dựng ≠ lệch spec. Đừng
sửa `epics.md`/`prd.md` cho khớp mã đã viết."* AC mô tả đích đến và nó đúng; `epics.md:4919` đã nói trước
rằng bộ lọc là *"điều kiện để FR123, FR124 và FR126 còn tác dụng ở quy mô năm mươi Chương"*. Thứ đo được là
**phân bố khối lượng**: phần lớn công việc của Story 6.10 không phải bộ lọc, mà là dựng đường xem trước chạy
cho cả N đơn vị.

Khoảng trống này **đã có tên từ trước** — hai mục nợ mang `Chủ: Story 6.10` mô tả đúng nó từ hai hướng:
`deferred-work.md:9865` (tầng luật làm sạch ghim vĩnh viễn vào Chương ĐẦU, ghi ở vòng rà Story 6.6) và
`:10000` (tầng ranh giới bóc chỉ hiện Chương ĐẦU, ghi ở vòng rà Story 6.7).

---

## 2. Phân tích tác động

### 2.1 Tác động Epic

**Epic 6 vẫn hoàn tất được như quy hoạch — nó chỉ cần thêm một story.** Không epic nào bị vô hiệu, không epic
nào cần đổi thứ tự, không epic tương lai nào bị chạm.

**Đo tác động xuôi dòng:** `grep -n "Story 6\.10" epics.md` cho **đúng một** kết quả ngoài chính mục
`### Story 6.10` — dòng `:4918`, nằm trong AC của chính nó. ⇒ **Không story nào phụ thuộc 6.10.** Chiều
ngược lại thì có: `epic-6-context.md:61` ghi *"Story 6.9, 6.6, 6.3, 6.5 nuôi trực tiếp vào 6.10"*, và cả bốn
đã `review`.

Đây là **cùng hình dạng** với hai lượt tách trước: Story 3.4b (2026-08-21) và Story 3.10b (2026-08-25). Và
nó cùng **lý do** với lượt 3.4b — tách theo **TẦNG, không theo mục tiêu**: đây vẫn là một mục tiêu người
dùng duy nhất, story dưới giao đường dữ liệu, story trên giao thứ người dùng nhìn thấy.
*(Khác lượt 3.10b, vốn tách vì một cửa chặn kiến trúc cần một `AD` mới. Ở đây **không cần `AD` mới** — xem §2.3.)*

### 2.2 Tác động PRD

**PRD không cần sửa, và MVP không bị chạm.** `prd.md:333` phát biểu FR132 bằng đúng hai vế — *hai con số* và
*một thao tác lọc* — và không liệt kê nguyên nhân nào. Phép tách này giao trọn cả hai vế, chỉ qua hai story
thay vì một. Phạm vi MVP không đổi, không mục tiêu nào bị hạ.

⚠️ **Nhưng lượt quét tìm ra một mâu thuẫn NỘI BỘ trong `epics.md`, cần đóng cùng lượt này.** Nguyên nhân
**thứ tư** được viết khác nhau ở hai chỗ:

- `epics.md:92` (bản diễn giải FR132): *"số Chương tách ra không khớp số đơn vị đầu vào (FR14)"*
- `epics.md:4934` (AC của Story 6.10): *"link hỏng"*
- `EXPERIENCE.md:142` (bảng trạng thái): *"link hỏng"*

Hai trên ba nói *link hỏng*, và Ice đã chốt hôm nay 2026-09-08 theo đúng nghĩa đó (quyết định #2, §3). ⇒ Đề
xuất **sửa `:92` tại chỗ kèm 🔵 và ngày**, không xoá. Đây không phải sửa spec cho khớp mã — không dòng mã nào
liên quan; đây là hai câu trong cùng một tài liệu nói khác nhau, và một trong hai phải nhường.

### 2.3 Tác động Kiến trúc

**Không cần một `AD` mới.** Hai chỗ có thể trông như cần, cả hai đều không:

1. `project.rs:1194-1200` ghi *"§Never spec 6.6 cấm mọi ngưỡng/cờ đáng ngờ"*. Đọc `deferred-work.md:9825`
   thì đó là một lượt **HOÃN**, không phải lệnh cấm vĩnh viễn, và lý do hoãn là *"cả hai vế đòi một hằng số
   ngưỡng chưa đo được"*. Phép so **trung vị** trả lời đúng lý do ấy — nó TƯƠNG ĐỐI, cần N Chương chứ không
   cần một hằng số. ⇒ Lệnh cấm hết hiệu lực bằng chính điều kiện nó đặt ra; doc-comment phải được sửa tại
   chỗ kèm 🔵 và ngày khi lượt build đóng mục này.
2. AD-40 (`Fetcher`/`Extractor` tách rời) và AD-41 (allowlist mạng hai tầng) **không bị chạm** — chạy chuỗi
   cho N đơn vị thay vì một là cùng một chuỗi chạy nhiều lần, không một cổng mới, không một điểm ra mạng
   mới. AD-16 (xem trước bắt buộc trước khi ghi) được **củng cố**, không bị nới.

⚠️ Một bất biến **phải giữ nguyên và rất dễ trộn**: Story 6.7 chốt *"còn mục hỏng ⇒ nút xác nhận KHOÁ"*
(`chapters_shape_if_all_ok` doc-comment). Quyết định #2 hôm nay nới vế **XEM ĐƯỢC**, không nới vế **GHI
ĐƯỢC**. Hai mệnh đề khác nhau; lượt build phải nghiệm thu cả hai bằng hai ca riêng.

### 2.4 Tác động UX

`EXPERIENCE.md:142` **không cần sửa** — nó mô tả trạng thái một Chương, và cả hai story cộng lại giao đúng
mô tả đó. Mockup `web-import.html` cũng không cần sửa.

⚠️ **Hai chỗ mockup ngụ ý dữ liệu chưa có, ghi ra để lượt build không tưởng đã có:** `web-import.html:238-240`
vẽ `⌥← · Chương 3 / 50 · ⌥→` — con trỏ *Chương đang chọn* chưa tồn tại ở cả Rust lẫn `importPreviewState.ts`;
và mockup **không vẽ** nhãn *sạch*/*cần xem* ở cấp từng Chương, cũng **không vẽ** bốn nhãn nguyên nhân, nên
lượt build phải lấy từ `EXPERIENCE.md:142` chứ không từ mockup (`EXPERIENCE.md:423`: bản dựng là minh hoạ,
tài liệu thắng khi mâu thuẫn).

⚠️ `epic-6-context.md:53` mô tả phím `R` là *"bật/tắt luật khớp"*, trong khi §Spec Change Log của Story 6.9
ghi `R` đã đúc lại thành *"nhảy sang tầng 3"*. Không thuộc phạm vi lượt này; ghi ra để lượt biên soạn lại
epic context sau không chép tiếp mệnh đề đã hết đúng.

### 2.5 Tác động kỹ thuật khác

- **Lược đồ CSDL: không đổi.** `schema_version()` giữ **19**, 0 bước di trú. Cả hai story chỉ đổi thứ đi qua
  dây và thứ hiện lên màn hình.
- **Cổng: không thêm cổng mới.** 11 cổng của `pre-push` giữ nguyên; sàn quần thể phải rà lại nếu thêm tệp
  `.ts`/`.rs` (`check-panel-refs.mjs:555` FILE_FLOOR=39 · `webimport_boundary.rs:41` SRC_RS_FLOOR=50 và các
  sàn khác liệt ở hồ sơ điều tra §7).
- **Sổ nợ:** 16 mục mang `Chủ: Story 6.10` phải chia lại chủ giữa hai story — chi tiết ở §4.4.

---

## 3. Bốn quyết định Ice đã chốt 2026-09-08

Ghi lại nguyên vẹn để hai lượt build sau không phải hỏi lại:

1. **Phạm vi — TÁCH ĐÔI.** Một story cho đường dữ liệu theo từng Chương, một story cho bộ lọc.
2. **Nguyên nhân ④ — NỚI.** Link hỏng thành một Chương *"cần xem"*; xem trước dựng được với N−1 mục OK cộng
   một mục giữ chỗ. 🔴 Bất biến *"còn mục hỏng ⇒ nút xác nhận KHOÁ"* giữ nguyên.
3. **Nguyên nhân ① — CỜ CẤP LƯỢT NHẬP.** Tin cậy thấp báo một dòng cho cả lượt nhập, **không** nhân thành N
   Chương *"cần xem"*. Lý do: gắn cờ cả N làm hai số thành 50/0, tức bộ lọc mất tác dụng đúng lúc cần nhất.
4. **Ngưỡng ③ — CÙNG PHÉP SO TRUNG VỊ như ②.** Tỉ lệ xoá của một Chương so với trung vị tỉ lệ xoá của các
   Chương khác. Một cơ chế cho hai tín hiệu, không hằng số phù thuỷ nào — đúng lệnh cấm Ice đặt 2026-09-05.

---

## 4. Đề xuất sửa cụ thể

### 4.0 🔴 MỘT QUYẾT ĐỊNH CÒN MỞ — cách đặt tên hai story

Ice nói *"6.10a / 6.10b"*. Một phép đo làm tôi phải trình lại trước khi thi hành:

> `grep -rn "Story 6\.10"` cho **30** tham chiếu: **20** trong `deferred-work.md`, **8** trong bốn spec đã
> `done` (`spec-6-4`, `spec-6-5`, `spec-6-6`, `spec-6-7` — nội dung **frozen**, không được sửa), và **2**
> trong chú thích mã Rust (`core/segment/encoding.rs:238`, `tests/webimport_contract.rs:536`).

Đổi `6-10` thành `6-10b` làm mồ côi cả 30, chạm bốn tệp frozen, và va luật `sprint-status.yaml:4`
*"không đổi id story"*.

⇒ **Đề xuất (a) — GIỮ `6-10`, thêm `6-10a`:**

- **Story 6.10a** *(MỚI)* — Xem trước theo từng Chương và điều hướng Chương
- **Story 6.10** *(giữ nguyên id và tên)* — Bộ lọc "cần xem"

Cả 30 tham chiếu cũ vẫn đúng: chúng nói về bộ lọc, và bộ lọc vẫn là 6.10. Thứ tự thi hành 6.10a → 6.10 đọc
ngược so với tên, nhưng kho này đã khai sẵn *"thứ tự ĐỌC không phản ánh thứ tự THỰC THI"* (`sprint-status.yaml:7`).

⇒ **Phương án (b) — theo đúng lời Ice:** đổi `6-10` → `6-10b`, thêm `6-10a`. Rõ về thứ tự, nhưng phải sửa 30
chỗ trong đó 8 chỗ nằm trong tệp frozen.

✅ **Ice chốt (a) ngày 2026-09-08.** Giữ `6-10` cho bộ lọc, thêm `6-10a` cho nửa nền. Cả 30 tham chiếu cũ
đứng nguyên, 0 tệp frozen bị chạm.

### 4.1 `epics.md` — thêm mục Story 6.10a, chèn TRƯỚC `### Story 6.10` (dòng 4905)

```markdown
### Story 6.10a: Xem trước theo từng Chương và điều hướng Chương

**Covers:** FR132 *(nửa nền — đường dữ liệu theo từng Chương; nửa bộ lọc ở Story 6.10)*
**Nghiệm thu lại ở quy mô N Chương:** FR123, FR124

> 🔵 *(Thêm 2026-09-08 qua `correct-course` — `sprint-change-proposal-2026-09-08-story-6-10.md`.
> Điều tra 2026-09-08 đo được rằng màn xem trước tính cả bốn tầng từ byte của riêng đơn vị ĐẦU
> (`project.rs:1891-1893` → `:1749-1752`), nên hai con số của FR132 chưa có dữ liệu để bám. Phép tách
> theo **TẦNG, không theo mục tiêu** — cùng khuôn lượt 3.4b, và vẫn là một mục tiêu người dùng duy
> nhất. Story này đóng hai mục nợ đã có tên từ trước: `deferred-work.md:9865` và `:10000`.)*

As a người dịch dán 50 link cùng lúc,
I want soát được từng Chương một, không chỉ Chương đầu,
So that thứ tôi duyệt là thứ sắp ghi xuống, chứ không phải một mẫu đại diện.

**Acceptance Criteria:**

**Given** một lượt nhập nhiều Chương
**When** màn xem trước dựng
**Then** mỗi Chương mang dữ liệu của **chính nó** ở cả tầng ranh giới bóc lẫn tầng luật làm sạch
**And** không Chương nào mượn số của Chương khác

**Given** màn xem trước đa-Chương
**When** bấm `⌥←` hoặc `⌥→`
**Then** con trỏ *Chương đang chọn* dời một bước trong cùng lượt nhập
**And** tầng ranh giới bóc và tầng luật làm sạch hiện dữ liệu của **đúng Chương đó**

**Given** con trỏ đang ở Chương đầu hoặc Chương cuối
**When** bấm `⌥←` hoặc `⌥→` theo chiều đi ra
**Then** nó **dừng**, không cuộn vòng — cuộn vòng ở một danh sách 50 Chương làm người dùng mất chỗ đứng

**Given** một danh sách N link trong đó có link hỏng
**When** dựng xem trước
**Then** N−1 Chương tải được vẫn hiện đầy đủ, và mục hỏng **giữ chỗ tại đúng vị trí** kèm một trong tám lý do
🔴 **And** nút xác nhận vẫn **KHOÁ** — bất biến Story 6.7 không đổi. *Xem được* và *ghi được* là hai mệnh đề khác nhau, và lượt nghiệm thu phải có hai ca riêng cho chúng.

**Given** một lượt nhập và **không thao tác tay nào**
**When** xác nhận
**Then** byte ghi xuống `.atproj` **trùng đúng** kết quả trước story này — story này đổi thứ **xem được**, không đổi thứ **ghi xuống**

**Given** lớp phủ xem trước đã đóng
**When** bấm `⌥←`/`⌥→`
**Then** không thao tác nào của màn nhập xảy ra
⚠️ `⌥` **không phải** phím bổ trợ chính (`keys.ts:415` `lacksPrimaryMod = !m.meta && !m.ctrl`) ⇒ ba hợp âm này rơi đúng nhánh mà Story 6.9 đo là không an toàn cho hợp âm trần; theo khuôn `keys: undefined` + handler DOM cục bộ.
```

### 4.2 `epics.md` — sửa mục Story 6.10 hiện có (dòng 4905-4941)

**Đổi ba chỗ, giữ nguyên phần còn lại:**

| Chỗ | Cũ | Mới |
|---|---|---|
| `:4907` `**Covers:**` | `FR132` | `FR132` *(vế bộ lọc — nửa nền ở Story 6.10a)* |
| `:4936-4938` cặp AC `⌥←`/`⌥→` | nằm ở đây | **chuyển sang Story 6.10a**, thay bằng một dòng 🔵 trỏ sang |
| Sau `**Covers:**` | — | thêm khối 🔵 ghi bốn quyết định của Ice ở §3 |

**Khối 🔵 thêm vào:**

```markdown
> 🔵 *(Thu hẹp 2026-09-08 qua `correct-course` — nửa **nền** tách sang **Story 6.10a**; cặp phím
> `⌥←`/`⌥→` theo sang đó vì nó là điều hướng, không phải lọc. Bốn quyết định Ice chốt cùng ngày:
> ① bảng mã tin cậy thấp là **cờ cấp lượt nhập**, ngoài hai con số — gắn cờ cả N làm hai số thành
> 50/0, tức bộ lọc mất tác dụng đúng lúc cần nhất; ② *"luật làm sạch xoá quá nhiều"* dùng **cùng
> phép so trung vị** như vế *"bóc ra ngắn bất thường"*, một cơ chế cho hai tín hiệu và không hằng
> số phù thuỷ nào; ③ link hỏng thành một Chương *"cần xem"*, dựng được nhờ Story 6.10a; ④ story
> giữ nguyên id `6-10` vì `Story 6.10` đang được tham chiếu **30 lần**, trong đó 8 lần ở bốn spec
> đã `done` và frozen.)*
```

⚠️ **Còn mở, không chặn:** AC đòi `⌘↵` xác nhận toàn bộ, nhưng `Mod+Enter` đã thuộc `editor.confirm_segment`
(`src/commands/index.ts:2157`) và `createKeymap` **ném** khi hai command giành một hợp âm (`keys.ts:472-478`,
`check:commands` canh ở `check-commands.mjs:1670-1685` trên cả hai nhánh `isMac`). Hôm nay
`import.preview.confirm` né bằng `Mod+Alt+Enter` (`index.ts:1131`). ⇒ Năng lực *"xác nhận toàn bộ"* **đã có**;
chỉ hợp âm khác AC. Ghi thành nợ **có chủ: Story 6.10**, quyết định sản phẩm thuộc Ice.

### 4.3 `epics.md:92` — đóng mâu thuẫn nguyên nhân thứ tư

```
CŨ:  … hoặc số Chương tách ra không khớp số đơn vị đầu vào (FR14).
MỚI: … hoặc link hỏng (FR122). 🔵 *(Sửa 2026-09-08: bản trước viết "số Chương tách ra không
     khớp số đơn vị đầu vào (FR14)", lệch với AC của chính Story 6.10 (`:4934`) và với
     `EXPERIENCE.md:142` — cả hai viết "link hỏng". Hai trên ba, và Ice chốt theo nghĩa đó
     2026-09-08. Vế "N link ≠ N Chương" KHÔNG mất: nó đã có chủ riêng ở AC4 của Story 6.7.)*
```

### 4.4 `deferred-work.md` — chia lại chủ cho 16 mục

Sửa **tại chỗ kèm 🔵 và ngày**, không xoá mục nào.

| Chuyển sang **Chủ: Story 6.10a** | Vì sao |
|---|---|
| `:9865` tầng luật làm sạch ghim Chương đầu | Chính là thân story 6.10a |
| `:10000` tầng ranh giới bóc ghim Chương đầu | Chính là thân story 6.10a |
| `:10007` sửa luật khi màn URL mở thì không dựng lại | Cùng đường dựng lại xem trước |

🔵 *(Sửa 2026-09-08 lúc thi hành: bản đầu của §4.4 xếp `:9491` — "đếm trên TOÀN Chương" — vào nhóm chuyển
sang 6.10a. Đọc kỹ thân mục thì nó tự nói phép đếm đầy đủ *"hợp lý hơn để làm SAU khi Chương đã có trong
`project.db`"*, tức ở cấp Thư viện chứ không ở đường xem trước mà 6.10a mở. ⇒ **`:9491` ở lại
`Chủ: Story 6.10`**, không đổi. Ba mục chuyển, không phải bốn.)*

**Giữ `Chủ: Story 6.10`** (3 mục lõi): `:9825` cờ "đáng ngờ" + nút lọc · `:9844` ba hợp âm *(vế `⌥←`/`⌥→`
chuyển sang 6.10a, vế `⌥W` ở lại — đóng 🟡, ghi rõ phần còn hở)* · `:9948` phép so trung vị.

**Mục MỚI phải thêm** *(có chủ, không mồ côi)*: hợp âm `⌘↵` va `editor.confirm_segment` — **Chủ: Ice**
(quyết định sản phẩm: đổi phím của Epic 2, giữ `⌥⌘↵`, hay bắt cục bộ trong lớp phủ).

**Giữ nguyên `Chủ: Story 6.10`, không đụng** (9 mục thừa kế vì quy mô N): `:9478` · `:9503` 🟡 · `:9761` ·
`:9834` · `:9853` · `:9918` · `:9927` · `:9980` · `:10117`.
⚠️ `:9478` đòi `joined_lines` làm nguyên nhân **thứ năm** — mâu thuẫn với AC *"bốn nguyên nhân"*. Không xử
trong lượt này; nó cần một quyết định của Ice, không phải một dòng dev tự thêm.

### 4.5 `sprint-status.yaml` — thêm một khoá

```yaml
  6-9-bóc-nội-dung-chính-và-sửa-ranh-giới-bằng-bàn-phím: review
  6-10a-xem-trước-theo-từng-chương-và-điều-hướng-chương: backlog   # ← THÊM
  6-10-bộ-lọc-cần-xem: backlog                                      # ← giữ nguyên
```

⚠️ Thứ tự đọc `6-10a` trước `6-10` là **cố ý** và khớp thứ tự thi hành; nó không mâu thuẫn ghi chú
`sprint-status.yaml:7`, chỉ là một chỗ hiếm hoi hai thứ tự trùng nhau.

---

## 5. Đường đã chọn, và hai đường đã loại

**Đã chọn — Direct Adjustment (thêm một story vào cấu trúc epic hiện có).**
Effort: **Medium** · Risk: **Low** · Timeline: Epic 6 dài thêm một story, không epic nào khác bị chạm.

**Loại — Potential Rollback.** Không có gì để rollback: Story 6.10 chưa bắt đầu, và bốn story nuôi vào nó
(6.3, 6.5, 6.6, 6.9) đều đã `review` và đúng. Việc ghim `chapters.first()` không phải một lỗi cần gỡ — nó
đúng vào thời điểm nó ra đời, khi N luôn bằng 1, và cả hai mục nợ đã ghi ra điều đó tại chỗ.

**Loại — PRD MVP Review.** MVP không bị đe doạ. FR132 giao được trọn vẹn, chỉ qua hai story. Không mục tiêu
nào phải hạ, không thứ gì đẩy sang hậu-MVP.

**Vì sao KHÔNG giữ một story.** Đo trên tiền lệ: lượt 3.4b tách vì spec một mảnh đo **17.408 ký tự ≈
5.000–5.800 token** so với trần **1.600** của `bmad-build`, vượt 3,1–3,6× (`epics.md:2957`). Story 6.10 gộp
cả hai nửa chạm: pipeline Rust (`pipeline.rs`, `project.rs`), kiểu dây IPC, lớp phủ 1.975 dòng, module state
1.276 dòng, bảng command, cộng bốn tín hiệu phân loại mới và một phép trung vị chưa có dòng mã nào. Nó sẽ
vượt trần ít nhất bằng lượt 3.4 đã vượt. ⚠️ Con số 3,1–3,6× là **chép từ lượt 3.4b**, không phải số tôi đo
cho story này — nêu ra làm tiền lệ về hình dạng, không làm bằng chứng về kích thước.

---

## 6. Bàn giao

**Phân loại phạm vi: Moderate** — sắp xếp lại backlog, không phải một lượt replan.

| Vai | Việc |
|---|---|
| **Ice** | Chốt §4.0 (cách đặt tên); duyệt trọn đề xuất; nhận mục nợ `⌘↵` |
| **Developer agent** (`bmad-build`) | Thi hành §4.1–4.5, rồi build Story 6.10a từ hồ sơ điều tra 2026-09-08 |
| **Winston (Architect)** | Không cần — §2.3 đo được là **không cần `AD` mới** |

**Tiêu chí thành công của lượt sửa quy hoạch này:**

- `epics.md` có mục Story 6.10a đầy đủ; mục Story 6.10 thu hẹp kèm khối 🔵; `:92` hết mâu thuẫn với `:4934`
- `sprint-status.yaml` có khoá `6-10a-…: backlog`
- `deferred-work.md`: 4 mục đổi chủ sang 6.10a, `:9844` đóng 🟡 đúng nửa, 1 mục mới `Chủ: Ice` — và
  `npm run check:debt-owner` **xanh**, tức không mục nào mồ côi
- **0** tệp spec đã `done` bị sửa

**Tiêu chí thành công của Story 6.10a** *(cho lượt build sau)*: một lượt dán N link cho ra N Chương soát được
từng cái bằng `⌥←`/`⌥→`, tầng 2 và tầng 3 theo con trỏ; một link hỏng không giết cả màn xem trước nhưng vẫn
khoá nút xác nhận; và byte ghi xuống không đổi một chút nào so với trước story.
