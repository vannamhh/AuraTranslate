# Hồ sơ điều tra — Story 6.10 "Bộ lọc cần xem" (2026-09-08)

**Người soạn:** `bmad-build` bước 2 (điều tra), dừng trước khi soạn spec · **Baseline:** `24d07c4` (master, cây sạch)
**Vì sao có tệp này:** lượt điều tra tìm ra rằng AC của Story 6.10 đứng trên một năng lực **chưa dựng**, nên
story phải tách đôi; tách đôi là thêm một mục quy hoạch, và kho này định tuyến việc đó qua `correct-course`
(tiền lệ Story 3.4b 2026-08-21, Story 3.10b 2026-08-25). Tệp này chở **phép đo** để lượt `correct-course`
và lượt build sau đó không phải đo lại.

⇒ Mọi số trong tệp này đo ngày **2026-09-08** trên `24d07c4`. Số đo không truy nguyên được thì không phải số đo.

---

## 1. Phát hiện trung tâm — màn xem trước chưa bao giờ đa-Chương thật

`preview_import_encoding` gặp `PipelineShape::Chapters` thì **chốt từ đơn vị ĐẦU**:

- `src-tauri/src/commands/project.rs:1891-1893` — `PipelineShape::Chapters(chapters) => match chapters.first()`
- `src-tauri/src/commands/project.rs:1749-1752` — `encoding_candidate_wire` gói lại thành
  `PipelineShape::Blob(ChapterInput::RawBytes { bytes: full_bytes, label })`

⇒ Trên **đường URL**, cả bốn tầng xem trước (bảng mã · chuẩn hoá · ranh giới bóc · tách Chương) được tính từ
byte của **đúng link đầu tiên**. Đây không phải một khuyết tật mới phát hiện — nó đã được ghi làm nợ từ hai
hướng khác nhau, và cả hai mục đều mang `Chủ: Story 6.10`:

- `deferred-work.md:9865` — tầng 3 (luật làm sạch) ghim vĩnh viễn vào Chương ĐẦU
- `deferred-work.md:10000` — tầng 2 (ranh giới bóc) chỉ hiện văn bản đã bóc của Chương ĐẦU

⚠️ Điều này KHÔNG làm AC sai. *"Năng lực chưa dựng ≠ lệch spec"* — AC mô tả đích đến, và `epics.md:4919`
đã nói trước rằng bộ lọc là *"điều kiện để FR123, FR124 và FR126 còn tác dụng ở quy mô năm mươi Chương"*.
Thứ đo được là: phần lớn khối lượng của Story 6.10 **không phải bộ lọc**, mà là dựng đường xem trước
chạy cho cả N đơn vị.

## 2. Bốn nguyên nhân "cần xem" — thứ thật sự tồn tại hôm nay

| Nguyên nhân (AC) | Trạng thái đo được | Neo |
|---|---|---|
| ① bảng mã tin cậy thấp | Có, nhưng **cấp lượt nhập**, không cấp Chương | `core/segment/encoding.rs:122` `Confidence` ba nhánh · `pipeline.rs` doc-comment `PipelineInput::encoding` |
| ② bóc ra ngắn bất thường so với **trung vị** | Độ dài per-Chương **có**; trung vị **0 dòng mã** trong cả kho | `project.rs:1189` `length` · `grep median` chỉ trúng một doc-comment ở `core/matching/mod.rs:156` |
| ③ luật làm sạch xoá quá nhiều | Span xoá **có** (`start`/`end`), nhưng per-Chương chỉ tồn tại trên đường `Chapters`, mà đường đó bị gói về `Blob` | `core/cleanup/mod.rs:140` `CleanupMatch` · `project.rs:1749` |
| ④ link hỏng | 8 lý do **có**, nhưng còn một mục hỏng thì **không có màn xem trước nào cả** | `core/webimport/mod.rs:66-84` · `project.rs:2362-2371` |

**Chi tiết ①.** `Confidence` là phán quyết ba nhánh (`SelfDeclared` / `HighGuess` / `LowGuess`), không phải
một con số. Dò chạy **một lần** cho cả lượt nhập, trên đơn vị đầu — Ice chốt 2026-09-06, *"một bảng mã cho
cả danh sách"*, và ca *"link A GBK, link B UTF-8"* đã được ghi là lỗ còn hở tại chính doc-comment đó.

**Chi tiết ④.** `chapters_shape_if_all_ok` (`project.rs:2362`) trả `None` khi **bất kỳ** mục nào mang
`error.is_some()`. Doc-comment của nó nói rõ trả `None` là *"lựa chọn AN TOÀN duy nhất"* vì hàm không được
lặng lẽ dựng một Chương RỖNG. ⇒ Một Chương "cần xem vì link hỏng" hôm nay **không tồn tại được**; muốn có
phải nới hàm này mà **giữ nguyên** bất biến 6.7 *"còn mục hỏng ⇒ nút xác nhận KHOÁ"* — hai mệnh đề khác
nhau, rất dễ trộn.

## 3. Bàn phím — bốn hợp âm, ba vấn đề khác nhau

🔴 **`⌥` KHÔNG phải phím bổ trợ chính.** `src/commands/keys.ts:415`:

```ts
const lacksPrimaryMod = (m: Mods): boolean => !m.meta && !m.ctrl
```

dùng ở `keys.ts:510`. ⇒ `⌥W`, `⌥←`, `⌥→` rơi **đúng nhánh** mà Story 6.9 đã đo là không an toàn cho hợp âm
trần, vì `isTypingZone` (`keys.ts:434`) không phủ `<button>`.

⚠️ Sắc thái phải giữ: rủi ro cụ thể của `Space` là cướp phím **kích hoạt native** của `<button>`;
`⌥W`/`⌥←`/`⌥→` không có hành vi native đó nên rủi ro ấy biến mất. Rủi ro **còn lại và có thật**:
`preventDefault()` chạy trước mọi thứ (`keys.ts:512`), nên một hợp âm toàn cục vẫn nuốt phím **khi lớp phủ
đã đóng**. ⇒ Vẫn theo khuôn `keys: undefined` + handler DOM cục bộ của Story 6.9.

⚠️ **`onTier2Keydown` hiện `return` sớm với `altKey`/`metaKey`** — `src/ImportPreviewOverlay.vue:517`, dòng
đầu tiên của handler. Đường DOM cục bộ hôm nay **không thể** chở `⌥`/`⌘`; cần một handler thứ hai trên scrim,
không phải nới vị từ này (nới nó sẽ làm `⌥`+`j` rơi vào nhánh `j`).

🔴 **`⌘↵` đã có chủ.** `src/commands/index.ts:2157` `keys: ['Mod+Enter']` thuộc `editor.confirm_segment`.
`createKeymap` **ném** khi hai command giành một hợp âm (`keys.ts:472-478`), và `check:commands` dựng keymap
từ bộ command THẬT trên **cả hai** nhánh `isMac` (`check-commands.mjs:1670-1685`) ⇒ va chạm là một cổng ĐỎ,
không phải một lỗi runtime âm thầm. `import.preview.confirm` hôm nay né bằng `Mod+Alt+Enter`
(`index.ts:1131`, lý do ghi tại `:1127-1130`). ⇒ AC đòi `⌘↵` là một **quyết định sản phẩm còn mở**, không
phải một dòng mã.

**Hợp âm còn trống** (đo `grep -oE "Mod\+Alt\+[A-Za-z0-9]+"` và `'Alt+` trên `index.ts`): `Alt+W`,
`Alt+ArrowLeft`, `Alt+ArrowRight` trần **chưa ai chiếm**. Đã chiếm: `Mod+Alt+W` = `library.list_works`
(`:1313`), `Mod+Alt+ArrowLeft/Right` = `panelRing` (`:1044`), `Alt+Shift+ArrowLeft/Right` = mở rộng chọn từ
(`:2095-2096`).

## 4. Bốn quyết định Ice đã chốt 2026-09-08

Ghi lại nguyên vẹn để lượt sau không phải hỏi lại:

1. **Phạm vi — TÁCH ĐÔI.** `6.10a` = đường N Chương (gỡ hai chỗ ghim `chapters.first()`, con trỏ *Chương đang
   chọn*, `⌥←`/`⌥→`; đóng nợ `:9865` + `:10000`). `6.10b` = bộ lọc (phân loại, hai số, `⌥W`).
2. **Nguyên nhân ④ — NỚI.** Link hỏng thành một Chương *"cần xem"*, xem trước dựng được với N−1 mục OK cộng
   một mục giữ chỗ. 🔴 Giữ nguyên bất biến 6.7 *"còn mục hỏng ⇒ nút xác nhận KHOÁ"*.
3. **Nguyên nhân ① — CỜ CẤP LƯỢT NHẬP.** Tin cậy thấp báo một dòng cho cả lượt nhập, **không** nhân thành N
   Chương *"cần xem"*. Lý do: gắn cờ cả N làm hai số thành 50/0, tức bộ lọc mất tác dụng đúng lúc cần nhất.
4. **Ngưỡng ③ — CÙNG PHÉP SO TRUNG VỊ như ②.** Tỉ lệ xoá của một Chương so với trung vị tỉ lệ xoá của các
   Chương khác. Một cơ chế cho hai tín hiệu, và không hằng số phù thuỷ nào — đúng lệnh cấm Ice đặt 2026-09-05.

⚠️ **Còn mở, chưa hỏi:** hợp âm `⌘↵` (mục 3 ở trên). Nó thuộc bề mặt bàn phím của `6.10b`.

## 5. Sổ nợ — 16 mục, không phải 14

`grep` theo **dòng** đếm 14; số thật là **16** vì hai dấu chủ bị gãy dòng (`deferred-work.md:9486→9487` và
`:9846→9849`). ⇒ Đếm nợ phải quét theo **mục**, không theo dòng.

**Lõi AC (3 mục):** `:9825` cờ "đáng ngờ" + nút lọc (Ice hoãn từ 6.6) · `:9844` ba hợp âm `⌥W`/`⌥←`/`⌥→` ·
`:9948` phép so trung vị.

**Điều kiện tiên quyết kỹ thuật (4 mục):** `:9865` tầng 3 ghim Chương đầu · `:10000` tầng 2 ghim Chương đầu ·
`:10007` sửa luật khi màn URL mở thì không dựng lại · `:9491` đếm trên TOÀN Chương.

**Thừa kế vì quy mô N (9 mục):** `:9478` `joined_lines` là nguyên nhân **thứ năm** — mâu thuẫn với AC "bốn
nguyên nhân", cần Ice quyết chứ không phải dev tự thêm · `:9503` 🟡 nửa đóng · `:9761` bản chép thứ hai trong
renderer · `:9834` + `:9927` `spawn_import_scan` chỉ quét Chương đầu · `:9853` trần số Chương một mẫu sinh ra ·
`:9918` đo lại tỉ lệ bóc đúng trên trang đọc truyện · `:9980` phản hồi tiến độ khi tải N link · `:10117` chưa
có phép đo hiệu năng nào trên đường xem trước URL.

**Không mục 6.10 nào đã ĐÃ ĐÓNG hoặc KHÔNG LÀM.** Duy nhất `:9503` là 🟡, vế còn lại vẫn thuộc 6.10.

🔵 **Vì sao lệnh cấm cũ KHÔNG chặn story này.** `project.rs:1194-1200` ghi *"§Never spec 6.6 cấm mọi ngưỡng/cờ
đáng ngờ"*. Đọc `deferred-work.md:9825` thì thấy đó là một lượt **HOÃN**, không phải một lệnh cấm vĩnh viễn,
và lý do hoãn là *"cả hai vế đòi một hằng số ngưỡng chưa đo được"*. Phép so **trung vị** của AC trả lời đúng
lý do ấy — nó là phép so TƯƠNG ĐỐI, cần N Chương chứ không cần một hằng số. ⇒ Không cần một `AD` mới; nhưng
doc-comment `project.rs:1194-1200` phải được **sửa tại chỗ kèm 🔵 và ngày** khi lượt build đóng mục này.

## 6. Hai con số tôi đang mang SAI trước lượt đo này

🔵 **`spec-6-9` viết "tám cổng tĩnh" — hết đúng.** `.githooks/pre-push:81` chạy **11** cổng:
`deps tokens i18n commands layout panel-refs dict dict-manifest lint gates debt-owner`. Có **12** tệp
`check-*.mjs`, hai trong số đó (`check:scope`, `check:scope:bundled`) cố ý ngoài pre-push.

🔵 **`check:commands` CÓ canh va chạm hợp âm** — `check-commands.mjs:1670-1685` dựng keymap từ bộ command thật
trên cả hai nhánh `isMac`. Thứ nó **không** canh là hợp âm *có an toàn không* (gieo `keys: ['Space']` vẫn
XANH, `spec-6-9…md:168`). Đừng đọc lượt xanh của nó thành "hợp âm đã được duyệt".

## 7. Ràng buộc cho lượt build sau — thứ đã có chủ, đừng dựng nguồn sự thật thứ hai

- 8 lý do **link hỏng** + giữ vị trí mục hỏng ⇒ `webimport_contract.rs:489-734`, `:307`
- *"Xem trước = xác nhận từng byte"*, kể cả khi luật xoá sạch Chương ⇒ `cleanup_contract.rs:468`, `:539`,
  `:747`, `:1203`
- Hình dạng dây `snake_case` ⇒ `segment_contract.rs:9074`. ⚠️ Đối chứng `rename_all = "camelCase"` **sẽ đỏ
  thật** cho trường mới của 6.10 (`needs_review`, `review_cause` — không phải từ đơn), khác ghi chú
  `spec-6-9…md:167` nơi mọi trường đều là từ đơn nên phép biến đổi là **rỗng**.
- Vỏ IPC mới phải vào `generate_handler!` + `app.manage` ⇒ `ipc_contract.rs:898`, `:955`
- Ô nhớ mới trong `importPreviewState.ts` phải được **GÁN** trong `resetImportPreview` (`:1237`, 38 ô hôm nay)
  ⇒ `check-panel-refs.mjs:531`; một lượt **đọc** không tính. ⚠️ Khai **một ô một dòng** —
  `const a = ref(0), b = ref(0)` rơi ra ngoài tập con của cổng và cho FAIL.
- Nhãn trạng thái phân biệt bằng **sắc độ + viền**, không bóng đổ ⇒ `check-tokens.mjs:1459`, không lối miễn trừ.
- Bốn nhãn nguyên nhân phải là **bốn khoá literal riêng** qua một `switch` cạn, không nội suy khoá — để
  `check:i18n` thấy literal.

## 8. Số nền để đối chiếu (đo trên `24d07c4`, chưa chạy lại trong lượt này)

`spec-6-9…md:161-162` ghi **1230 ca Rust / 44 binary** và **909 ca vitest / 69 tệp**, `fileParallelism: false`
⇒ một lượt vitest ~98 s. ⚠️ Đây là số **chép lại**, không phải số tôi đo — lượt build sau phải chạy và đo lại,
đừng chép tiếp.

---

## 9. Bốn quyết định Ice chốt ở lượt `bmad-build` 2026-09-08 (bước 2, dừng trước khi soạn spec)

Ghi nguyên vẹn để lượt `correct-course` và lượt build sau không phải hỏi lại. Baseline: `b42be3b`,
cây sạch. Ba lượt điều tra đi trước bốn câu hỏi này; số đo của chúng nằm ở §10.

1. 🔴 **Quy tắc ngưỡng — HÀNG RÀO TUKEY, không phải một tỉ lệ so trung vị.** *Ngắn bất thường* =
   dưới `Q1 − 1,5 × IQR`; *xoá quá nhiều* = trên `Q3 + 1,5 × IQR`. Một cơ chế cho hai tín hiệu,
   đúng AC. Lý do chọn: hằng `1,5` là quy ước thống kê **có tên** (Tukey 1977), không phải một số
   tự đúc — nên nó đi qua được lệnh cấm hằng số phù thuỷ 2026-09-05 theo đúng lý do lệnh cấm ấy
   tồn tại; và hàng rào **tự thích nghi theo độ tản**, nên một lượt nhập N Chương dài đều nhau
   không bị cờ oan.
   ⚠️ **Giá phải trả, phải xử lý tường minh:** tứ phân vị vô nghĩa khi N nhỏ. `N = 1` thì
   *"các Chương khác"* không tồn tại; N nhỏ thì `IQR = 0` là chuyện thường. Cả hai ca phải khai
   **"không đo được"** và phân biệt được với *"sạch"* — chứ không rơi vào nhánh sạch.

2. **Hợp âm `⌘↵` — GIỮ `Mod+Alt+Enter`, nợ `deferred-work.md:9943` đứng nguyên.** Không đụng bàn
   phím Epic 2 trong story này. AC `⌘↵` mô tả đích đến và **không sai** vì đường đi chưa tới —
   đúng luật *"năng lực chưa dựng ≠ lệch spec"*. Không sửa `epics.md` cho khớp mã.

3. 🔴 **Nguyên nhân thứ năm `joined_lines` — MỞ `correct-course` TRƯỚC.** Lượt `bmad-build`
   2026-09-08 **dừng ở bước 2** vì lý do này. Thêm dấu hiệu thứ năm là sửa FR132 (`prd.md:335`)
   và AC của Story 6.10 trong `epics.md` — một mục quy hoạch, và kho định tuyến việc đó qua
   `correct-course` (tiền lệ 3.4b 2026-08-21, 3.10b 2026-08-25, và chính lượt tách 6.10a
   2026-09-08). ⇒ Bộ lọc ra đời đã đủ **năm** tín hiệu thay vì phải nới lần hai.
   Mục nợ nguồn: `deferred-work.md:9478` (kèm `:9498` — `joined_lines` hôm nay chỉ đếm trên
   **cửa sổ bằng chứng**, không trên toàn Chương; hai vế phải đi cùng nhau).

4. 🔴 **`cleanup_match_count` đổi thành `Option<usize>`.** `null` = *không đo được cho Chương này*;
   `Some(0)` = *luật thật sự không khớp gì*. Đây là phép **sửa NGUỒN cho nó nói thật**, không phải
   một cờ vá bên ngoài.
   **Vì sao bắt buộc:** trên đường `Blob` + mẫu phân tách, chỉ Chương `ord == 1` có `cleanup_report`
   (`project.rs:1196-1199`, giới hạn pipeline đã ký ở 6.10a). Mọi Chương `ord >= 2` cho `0`, và trên
   dây con số ấy không phân biệt được với một phép đo thật. ⇒ Trung vị của N−1 số `0` là `0`, và bộ
   lọc sẽ khai *"M Chương sạch"* cho những Chương **chưa ai đo** — đúng biến thể Story 3.9 mà
   `AGENTS.md` §Known pitfalls dẫn ra.
   **Hai chỗ phải sửa tại chỗ kèm 🔵 và ngày:** `src-tauri/tests/segment_contract.rs:9333` (ca khoá
   hình dạng dây, fixture đang đặt `cleanup_match_count: 0` làm *"số thật"*) và vị từ kiểm kiểu
   `src/config/project.ts:357`.

## 10. Số đo mới của lượt 2026-09-08 (bmad-build bước 2) — thứ lật một mệnh đề cũ

- 🔵 **Cổng: MƯỜI MỘT, không phải "tám".** `check-gates.mjs` tự in *"11 cổng trong .githooks/pre-push"*.
  Cụm *"tám cổng"* ở commit `24d07c4` và `deferred-work.md:10290` là số **tường thuật đã cũ**, không
  phải số đo. Có 12 tệp `check-*.mjs`; 11 cổng = 10 tệp cộng `check:lint` (là `eslint`, không phải
  một `.mjs`), và `check:scope`/`check:scope:bundled` cố ý ngoài pre-push.
- 🔵 **40 tệp test Rust, không phải "44 binary".** `spec-6-10a:154` chép lại con số "44" đúng vào
  lượt mà `:168` của chính nó cảnh báo *"đừng chép tiếp một con số không phải mình đo"*. Số nền ca
  test (`1.255` Rust / `928` vitest) cũng là số **đã ghi**, chưa đo lại — lượt build sau phải tự chạy.
- 🔴 **Nguyên nhân ④ chưa có chỗ đứng trong danh sách Chương.** `chapters_shape_for_view`
  (`project.rs:2565-2569`) **lọc bỏ** mục hỏng bằng `filter_map`, và `ord` được đánh lại `1..M` liên
  tục (`:1227`). Doc-comment `:2554-2556` chỉ khai bất biến vị trí cho danh sách `items`, **không**
  cho danh sách Chương. ⇒ Hai danh sách song song lệch chỉ số, không trường nào trỏ ngược. Quyết định
  #2 của §4 (*"link hỏng thành một Chương cần xem"*) vì thế đòi một phép hợp nhất mà Rust chưa dựng,
  và mọi ánh xạ *"hàng thứ k trên màn" → `chapter_index`* phải đi qua phép lọc đó.
- **Nguyên nhân ① đã sẵn, không cần trường mới.** `ConfidenceWire` (`project.rs:1415`, ba nhánh) ra
  tới frontend ở cấp lượt nhập qua `ImportEncodingPreview::confidence` (`:1436`), đọc ở
  `importPreviewState.ts:364`. Đúng hình dạng quyết định #3 của §4 đòi.
- **Trung vị phía Rust: 0 dòng.** Bản duy nhất trong kho là `percentile()` ở
  `src/panels/lookupTiming.ts:90` — TypeScript, thuộc panel đo tra cứu. Lượt build phải viết mới và
  khai tường minh nó khớp hay cố ý lệch quy ước của bản kia, đừng để hai định nghĩa âm thầm khác nhau.
- ⚠️ **`⌥W` trên macOS gõ ra `∑`** — `event.key` sẽ không bao giờ là `'w'`. Phải so
  `event.code === 'KeyW'`, đúng thứ `keys.ts:509` làm. `keys.ts:406` đã ghi đích danh một khuyết tật
  cùng lớp (`Alt+M` gõ ra `µ`). Cặp `⌥←`/`⌥→` của 6.10a né được vì phím mũi tên không bị `⌥` biến đổi,
  nên khuôn của nó **không** chép thẳng sang được.
- ⚠️ **Phép co gọn tầng 4 va thẳng vào một chiều lọc.** `chapterEntriesDefaultWindow`
  (`ImportPreviewOverlay.vue:359-361`) cắt `slice(0,3)`/`slice(-3)` trên mảng **gốc**, và
  `currentChapterDomId` (`:381-387`) dựng lại danh sách rendered từ hai nhánh. Một chế độ lọc thứ ba
  làm biểu thức đó trả một `id` cho hàng **đã bị lọc khỏi DOM** ⇒ `aria-activedescendant` treo. Thứ tự
  *lọc trước hay cắt trước* là một quyết định phải viết ra. Cộng thêm: ba nhánh `v-for` (`:1210`,
  `:1234`, `:1257`) lặp cùng một khối `<li>`, nên thêm một chiều mà không hợp nhất là ba chỗ phải sửa.
- 🔴 **`check:commands` KHÔNG canh hợp âm có an toàn không** — `grep lacksPrimaryMod` trên
  `scripts/check-commands.mjs` cho **0** kết quả; Kiểm E (`:1673-1679`) chỉ dựng keymap thật và bắt
  **va chạm**. ⇒ Một `Alt+W` trần đăng ký toàn cục sẽ đi qua cổng **XANH**. Kỷ luật `keys: undefined`
  cộng handler DOM cục bộ phải tự giữ, không trông cậy cổng.
- 🔵 **`deferred-work.md:9903` hết đúng.** Nó khai phép vá `NEGATIVE_OWNER_RE` *"thêm `chưa ai`,
  `chưa story`, `chưa xác định`"*, nhưng `check-debt-owner.mjs:192-193` hôm nay chỉ có bảy cụm và
  **không** có `chưa story`, `chưa xác định` — lượt rà sau đó đã gỡ hai cụm tự đúc. Sửa câu ở sổ nợ.
- ⚠️ **`FILE_FLOOR = 39` của `check-panel-refs.mjs:555` đã lỗi thời.** `find src -name '*.ts'` đếm
  **63** tệp hôm nay ⇒ 39/63 = **61,9 %**, tụt xa dưới dải 80–85 % mà chính doc-comment `:551` đặt ra,
  và `:552` tự cảnh báo *"một sàn cũ là một sàn vô nghĩa"*. Món nợ này chưa có chủ.
