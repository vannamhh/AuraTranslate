# Review AD-62 (rubric) — 2026-10-30

Phạm vi: AD-62 (spine dòng 1388-1438), AD-34 (416-432), ad-brief-san-chuot §5, đối chiếu mã. Chỉ đọc; không sửa mã.

## Verdict
Sát brief và phần lớn khớp mã (mọi citation file:line đúng chỗ), nhưng có 1 lỗi sự kiện (ví dụ `auto` tự đỏ), 2 lỗ hổng cưỡng chế (trần nợ nâng được; `auto` chứng minh sai thứ) và vài điểm mơ hồ gây phân kỳ. Cần sửa AD trước khi giao story cổng.

## Citation (đã kiểm)
- `registry.ts:45` = `export type CommandSpec`; `index.ts:5391` = `installCommands`; `check-commands.mjs:238` = `COMMAND_FLOOR`; `:449` Kiểm A mở đầu (kết thúc ~533); `:1341` mở Kiểm E, lời gọi `installCommands` ở `:1371` (AD ghi `:1370` — lệch 1, chấp nhận được, nhưng đúng hơn là `:1371`); `commands-scan.mjs:362` `scanVueAttrs`; `main.ts:1302` `dispatch('lookup.lookup_selection')`; `GridPanel.vue:962-975` là `onCellMouseDown`/ghi chú `@mouseup` đúng ý; `TmManageOverlay.vue:723`, `PromptLibraryOverlay.vue:763` đúng. `maskFile` đúng là chưa export (`check-commands.mjs:358`, ghi chú `:275`).

## Findings

### F1 — HIGH: ví dụ `auto` của `library.list_works` tự đỏ dưới chính luật `auto`
- AD Rule 3 `auto` nêu `library.list_works` "khi vào Thư viện" và đòi ít nhất một `dispatch('<id>')` literal trong mã `src/` ngoài template và ngoài `index.ts`.
- Thực tế: `grep dispatch('library.list_works')` trong `src/` ra 0. `LibraryMode.vue:154` gọi `void loadWorks()` trực tiếp (hàm, không dispatch); id chỉ xuất hiện ở `index.ts:2103` và `vi.json:435`.
- Hệ quả: phân loại 29 (`auto` 4) trong Rule 7 không đạt được như viết; story cổng hoặc phải dựng lại phân loại, hoặc nới điều kiện `auto` để gỡ đỏ (đúng điều AGENTS.md cấm). Brief §2 cũng trích `LibraryMode.vue:154` như bằng chứng "tự chạy" mà thực ra không đi qua registry.
- Fix: (a) bắt buộc đổi `LibraryMode.vue:154` thành `dispatch('library.list_works')` (cùng tinh thần AD-34 §1) và ghi vào AD là việc của story vá; hoặc (b) chuyển `library.list_works` sang sổ nợ/`focus`-style khác. Rà cả 3 command `auto` còn lại (AD chỉ nêu `lookup.lookup_selection`; ids còn lại không được liệt kê) bằng grep trước khi ghi con số 4.

### F2 — HIGH: "sổ nợ chỉ co lại" không được cưỡng chế bằng máy
- Rule 5: `POINTER_DEBT_CEILING` = độ dài mảng lúc hạ cổng; mảng dài hơn trần là đỏ. Hai lối thoát:
  1. Thêm id mới và nâng trần cùng commit (cả hai hằng đều trong `scripts/check-commands.mjs`).
  2. Đổi chỗ: gỡ một id, thêm một id khác — độ dài không đổi, xanh.
- "Chỉ id đỏ ở baseline được vào sổ" và "Command mới không bao giờ vào sổ" (Prevents #5) không có phép kiểm nào; đúng kiểu "mở rộng danh sách loại trừ để gỡ đỏ".
- Fix: thêm hằng đông cứng `POINTER_DEBT_BASELINE` (41 id, không bao giờ sửa trừ khi sửa AD) và Kiểm M đòi `POINTER_DEBT ⊆ POINTER_DEBT_BASELINE`; hoặc ít nhất ghim trần = 41 bằng chữ trong AD ("trần chỉ hạ, không nâng") và thêm self-check ca "id lạ vào sổ". Bỏ `POINTER_DEBT_CEILING` thành thừa nếu dùng tập con.

### F3 — MEDIUM: điều kiện `auto` chứng minh sai thứ
- "ít nhất một `dispatch('<id>')` literal trong `src/` ngoài template và index.ts" thoả cả khi lời gọi đó nằm trong handler `@keydown`/hàm bàn phím (ví dụ `ImportPreviewOverlay.vue:711` dispatch theo phím). Đó là đường phím, không phải sự kiện chuột/lúc mở bề mặt.
- Ghi chú ⚠️ của AD chỉ nói cổng không kiểm `reason` của `focus`/`aim`; `auto` được ngầm coi là "có kiểm" trong khi chỉ kiểm sự hiện diện của một dispatch bất kỳ. Dispatch trong comment: cần chắc bộ đọc dùng bản đã che (`p.masked`/`code`), AD không nói.
- Fix: nói rõ trong AD rằng `auto` cũng chỉ kiểm được "có call site", phần "đó là sự kiện chuột/mở bề mặt" thuộc review; yêu cầu quét trên văn bản đã che comment/chuỗi; cân nhắc đòi `auto.site` (`tệp::hàm`) khai kèm để review đối chiếu.

### F4 — MEDIUM: rewrite `@submit` làm vỡ bảng đông cứng Kiểm K, AD không nhắc
- `check-commands.mjs:2496` (`PromptLibraryOverlay.vue::onOpenImport`) và `:2580` (`TmManageOverlay.vue::onRowEdit`) là mục `HANDLER_TABLE`. Khi viết lại thành `@submit.prevent="dispatch('...')"`, các mục này thành "mục TREO" (`danglingHandlerKeys`) và Kiểm K đỏ cho đến khi xoá (AGENTS: không nhét miễn trừ). Cũng đụng `HANDLER_ATTR_FLOOR = 157` (`:2814` vùng) nếu số thuộc tính giảm.
- Fix: thêm vào Rule 1 hoặc Rule 8: story vá `@submit` xoá đúng hai mục bảng K và hạ sàn nếu cần trong cùng commit; ghi Kiểm K là nơi canh phần `@submit` không-dispatch.

### F5 — MEDIUM: `@dblclick`/`@contextmenu` không có cổng nào canh hình dạng
- Rule 1 viết "cùng hình dạng Kiểm A cưỡng chế cho `@click`" nhưng Kiểm A chỉ khớp `@click` (`CLICK_ATTR_RE`, `check-commands.mjs:451`); Kiểm K không có `dblclick`/`contextmenu`. Hiện có `@dblclick` thật (`LibraryMode.vue:602`, `GlossaryManageOverlay.vue:636`, `ReadingMode.vue:781`); một `@dblclick="fn()"` mới đi qua im lặng, vi phạm AD-34 §1 mà không đỏ. AD-62 làm chúng thành "điểm chuột" hợp lệ mà không đóng chiều "chỉ dispatch" cho chúng.
- Fix: ghi thành giới hạn đã biết trong AD (và `src/AGENTS.md` dòng luật), hoặc cho Kiểm M thêm phép "mọi `@dblclick`/`@contextmenu` là `dispatch` literal hay `.stop` trần"; nếu không làm thì ghi nợ có chủ.

### F6 — MEDIUM: số phận 9 command "không ai gọi" và quan hệ `kind: 'via'`/trường `via` mơ hồ
- Prevents #2 nêu 9 command mồ côi; Rule 3 nói "Command không ai cần thì xoá", Rule 7 nói 41 vào sổ nợ. Không rõ 9 ca này xoá, vào sổ, hay `via` (chỉ 2 `via`). Story phân loại sẽ chia khác nhau.
- `pointer: { kind, reason, via? }` và `via` cũng là một trong năm "loại": không nói `kind === 'via'` ⇔ `via` có mặt; `via` có mặt với `kind` khác thì sao.
- Fix: một bảng 70 → nhóm đề xuất đính kèm brief/AD (hoặc ghi rõ 9 ca đi đâu); câu: "`kind:'via'` bắt buộc có `via`; `via` chỉ hợp lệ khi `kind:'via'`", thêm vào Kiểm M và self-check (e).

### F7 — LOW: liên kết `POINTER_DEBT` ↔ `deferred-work.md` chỉ bằng kỷ luật
- Rule 5 nói `check:debt-owner` "canh được" chủ, nhưng nó chỉ canh chủ của mục, không biết id nào trong mảng thuộc mục nào. Đóng mục nợ mà quên gỡ id (hoặc ngược lại) không đỏ. Cổng không đọc `_bmad-output` (đúng AGENTS), nên phần này là thủ tục.
- Fix: yêu cầu mỗi mục nợ liệt kê id bằng văn bản và đưa vào DoD của story vá; ghi rõ đây là kiểm tay.

### F8 — LOW: các điểm nhỏ
- Điểm chuột đếm cả `.vue` không mount hoặc nút luôn `disabled`/`v-if=false`; AD không nói (chấp nhận được, nên ghi vào ⚠️).
- `pointer` là object lồng trong spec bị `frozen()` (`registry.ts:135`): không rõ có đông cứng sâu; nhỏ nhưng "chết cùng command" phụ thuộc vào đó. Nên nói `readonly`.
- Rule 8 "Đổi" liệt kê sửa `src/commands/README.md` và `src/AGENTS.md` nhưng không nói ai/lúc nào (AD-time hay story cổng). Hiện `src/AGENTS.md` chưa có dòng luật; AD-34 🔵 (dòng 422) đã có.
- Sàn quần thể: `ceil(0.85×259)=221` cho phép mất 38 điểm chuột mà sàn không đỏ — an toàn vì kiểm theo từng command, nhưng nên ghi sàn chỉ canh cây rỗng, không canh sự giảm.
- `:1370` nên là `:1371`.

## Đối chiếu brief §5
- Q1 (b có giới hạn, tiêu chí máy đọc được, xếp nhóm §2): đạt (Rule 3, 7), trừ F6.
- Q2 (a literal 4 thuộc tính): đạt (Rule 1); `@mousedown`/hàm bị loại có lý do.
- Q3/Q4 (Winston tự đề xuất, có số đo): đạt; trường trên `CommandSpec` + Kiểm M, 1,00 s vs 0,08 s. Khớp ràng buộc erasable-only (union chuỗi, không `enum`); không cổng thứ 15, ba danh sách không đổi — khớp AGENTS.
- Q5 (a: sổ nợ có chủ, rút dần): đạt về ý, nhưng cưỡng chế yếu (F2, F7).
- Dòng 🔵 ở AD-34 và câu cho README/AGENTS: có/đã khai (AD-34:422). AD-62 mở rộng AD-34 một chiều, không làm yếu §1/§2/§3; không mâu thuẫn Kiểm A/K, chỉ chồng lên (F4, F5).
- Không vi phạm "cổng không đọc `_bmad-output`" (Rule 5 nói rõ), không "mở rộng danh sách loại trừ" về nguyên tắc, nhưng F2 là cửa sau.
