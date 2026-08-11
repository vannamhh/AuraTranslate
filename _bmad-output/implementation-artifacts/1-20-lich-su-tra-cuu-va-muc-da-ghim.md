---
baseline_commit: 19ea24ccc89618d95ea0da43f5cb06a6b772fec9
---

# Story 1.20: Lịch sử tra cứu và mục đã ghim

Status: in-progress

**Covers:** FR41 (`prd.md:504`) · Giả định **A9** (`prd.md:1080`)
**Epic:** 1 — Nền móng ứng dụng & Tra cứu ngoại tuyến tức thì
**Nguồn:** `epics.md:1845-1878` · mockup `lookup-history-pins.html` · `ARCHITECTURE-SPINE.md` AD-1 · AD-7 · AD-11 · AD-21 · AD-30 · AD-34 · AD-44 ④
**baseline_commit:** `19ea24c` (cây làm việc SẠCH lúc dựng story — `git status --porcelain` trả 0 dòng, không cần commit vá riêng)

---

## Story

As a người dịch,
I want xem lại những gì mình vừa tra và ghim những mục hay dùng,
So that tôi không phải tra lại cùng một chữ năm lần trong một Chương.

---

## Acceptance Criteria

Nguyên văn từ `epics.md:1853-1878`, đánh số để tham chiếu:

**AC1** — **Given** người dùng tra nhiều lần trong một phiên · **When** mở tab lịch sử · **Then** thấy các lần tra theo thứ tự gần nhất trước

**AC2** — **Given** một mục từ trong kết quả tra cứu · **When** người dùng ghim · **Then** nó xuất hiện trong danh sách đã ghim

**AC3** — **Given** một mục đã ghim · **When** đóng rồi mở lại ứng dụng · **Then** nó vẫn còn

**AC4** — **Given** lịch sử tra cứu · **When** đóng ứng dụng · **Then** lịch sử của phiên kết thúc — đây là lịch sử **trong phiên**, khác với mục ghim

**AC5** — **Given** lịch sử và mục ghim · **When** hiển thị · **Then** là **tab thứ ba** của Panel Lookup, không phải một cửa sổ riêng

**AC6** — **Given** mọi thao tác của tính năng này · **When** gọi · **Then** có command đăng ký trong `CommandRegistry` và gán được phím

### AC bổ sung — dẫn xuất từ mockup, kiến trúc và đo đạc mã nguồn

Sáu AC trên không nói hết thứ phải đúng để tính năng chạy được trong hệ thống đang có. Bảy AC dưới đây **cùng hạng ràng buộc**, mỗi cái neo vào một nguồn kiểm chứng được:

**AC7 — dedupe, không dòng trùng.** Tra lại một mục đã có trong lịch sử ⇒ **đẩy hàng đó lên đầu** và tăng số đếm, **không** thêm một hàng thứ hai. *(`lookup-history-pins.html:171-172` nguyên văn: "Tra lại một mục đã có **đẩy nó lên đầu** và tăng số đếm, không tạo dòng trùng — nếu không, một Chương sẽ sinh ra hàng trăm dòng giống nhau.")*

**AC8 — hai trạng thái rỗng KHÁC NHAU, nói khác nhau.** Lịch sử rỗng và danh sách ghim rỗng là hai câu riêng, không một khung trắng chung. *(AD-44 ④, `ARCHITECTURE-SPINE.md:622`: "rỗng im lặng bị cấm; rỗng có lý do thì không." · `lookup-history-pins.html:182`: "Hai trạng thái rỗng — khác nhau, nên nói khác nhau.")* Chuỗi chính xác ở §Dev Notes ⑦.

**AC9 — đường vào Attribution KHÔNG được biến mất khi đổi tab.** Nút *"Nguồn dữ liệu"* (`data-attribution-open`, `LookupPanel.vue`) là **đường chuột duy nhất** vào màn hình Attribution — AC11 của Story 1.19. Nghiệm thu: ở **mọi** tab của Panel Lookup, nút đó vẫn bấm được bằng chuột. Xem §Bẫy 1 — đây là regression đắt nhất của story này.

**AC10 — `.lookup-head` giữ nguyên 76px.** Dải tab **không** được nhồi vào `.lookup-head` (`--lookup-head-height: 76px; overflow: hidden`). Nghiệm thu: `--lookup-head-height` vẫn là `76px` sau story, và đo chiều cao thật của `.lookup-head` ở cả hai tab cho cùng một số. Xem §Bẫy 2.

**AC11 — ghi lịch sử ở ĐÚNG MỘT chỗ.** Điểm ghi duy nhất là cuối `lookupPanelState.ts::runLookup`, **sau** guard `mine !== sequence`. Nghiệm thu: `grep` toàn `src/` cho hàm ghi lịch sử trả về **đúng một** chỗ gọi. *(Story 1.18 Quyết định #4a dựng điểm nghẽn này CHO story 1.20 — `main.ts:309-313` và `dictSourcesState.ts:171` đều ghi thành chữ.)*

**AC12 — đổi Tác phẩm vứt lịch sử, và nạp lại đúng bộ ghim.** `resetLookupPanel()` phải vứt luôn lịch sử trong phiên; nếu ghim theo phạm vi Tác phẩm (Quyết định #1) thì danh sách ghim phải **nạp lại** theo Tác phẩm mới, không giữ bộ của Tác phẩm cũ. Đối chứng âm bắt buộc: tạo Tác phẩm B sau khi ghim ở A ⇒ tab ghim của B **không** chứa mục của A.

**AC14 — câu trạng thái của `PanelFrame` không rò sang tab Lịch sử.** Ở tab Lịch sử, panel **không** được hiện câu *"Chọn một từ trong Nguyên văn để tra cứu."* — câu đó sai ngữ cảnh ở đó. Nghiệm thu: mở app sạch (chưa tra lượt nào), chuyển sang tab Lịch sử ⇒ chỉ thấy nội dung tab, không thấy câu dạy thao tác. Xem §Bẫy 10.

**AC13 — mọi sàn `*_FLOOR` bị vượt được nâng theo SỐ THẬT.** `COMMAND_FLOOR` · `CLICK_FLOOR` · `DISPATCH_FLOOR` · `TS_FLOOR` · `VUE_FLOOR` trong `scripts/check-commands.mjs`, và `RS_FLOOR`/`VUE_FLOOR` trong `scripts/check-i18n.mjs` nếu chạm. Số thật đo được ghi vào §Completion Notes, không ước.

---

## Task 0 — BẢY QUYẾT ĐỊNH, chốt TRƯỚC dòng mã đầu tiên

Khuôn cố định của mọi story lớn trong dự án (1.17 §161, 1.18 §142, 1.19 §201). Mỗi quyết định dưới đây có **đề xuất mặc định kèm lý lẽ đo được**. Dev đọc, xác nhận hoặc phản biện bằng số — không im lặng thi hành, và không tự đổi sau khi đã gõ mã.

### Quyết định #1 — Phạm vi của mục ghim: `project.db` hay `global.db`?

| | (a) `project.db` — theo Tác phẩm | (b) `global.db` — toàn ứng dụng |
|---|---|---|
| Mockup nói gì | **Ủng hộ.** `lookup-history-pins.html:158`: *"Mục ghim sống qua các phiên và **theo Tác phẩm**"*; `:205`: *"theo bạn qua các phiên và các Chương của **Tác phẩm này**"* | Không nguồn nào ủng hộ |
| AC epics nói gì | Trung lập — AC3 chỉ đòi *"đóng rồi mở lại ứng dụng, nó vẫn còn"* | Trung lập, cùng câu |
| Tiền lệ | `WORK_DDL`/`CHAPTER_DDL` (Story 1.15/1.16) đã ở `project.db` | Story 1.19 `KEY_DICT_DISABLED` ở `global.db` |
| Di trú cần thêm | `PROJECT_MIGRATIONS` bước **4** (target hiện tại = 3) | `GLOBAL_MIGRATIONS` bước **3** (target hiện tại = 2) |
| Ngữ nghĩa | Ghim 聽潮閣 của một bộ tiên hiệp không có nghĩa ở một tiểu thuyết khác | Một người dịch tắt nguồn X thì tắt ở mọi Tác phẩm — logic đó **không** chuyển sang ghim |
| Giá phải trả | Chưa mở Tác phẩm ⇒ **không có** `project.db` ⇒ tab ghim phải nói một câu có lý do (AD-44 ④), không khung trắng | Ghim của mọi Tác phẩm trộn vào một danh sách |

**ICE CHỐT 2026-08-10: (a) `project.db`.** Mockup nói bằng chữ ở hai chỗ, và ngữ nghĩa đúng — ghim 聽潮閣 của một bộ tiên hiệp không có nghĩa ở một tiểu thuyết khác. AD-18 **không** liệt kê "mục ghim" trong bảng ngữ nghĩa hai tầng (`ARCHITECTURE-SPINE.md:244-254`) nên không có ràng buộc kiến trúc ngược lại — đây là một khoảng trống, không một điều cấm.

**Ba hệ quả bắt buộc:**
1. Bước di trú mới vào **`PROJECT_MIGRATIONS`**, `to_version: 4` (target hiện tại = 3). `GLOBAL_MIGRATIONS` **không** đụng.
2. Khi chưa mở Tác phẩm nào, tab nói `panel.lookup.pinned_no_work` — **không** hiện trạng thái rỗng *"Chưa ghim mục nào"*. Hai câu trả lời hai câu hỏi khác nhau; gộp chúng là đúng bẫy `??` mà Story 1.17 đã bắt (§Bẫy 4).
3. Đổi Tác phẩm phải **nạp lại** bộ ghim, không giữ bộ cũ (AC12, có đối chứng âm ở bàn đo hàng 10).

### Quyết định #2 — Bảng SQL riêng, không một khoá `config_value`

Story 1.19 lấy tiền lệ *"thêm một khoá của `ScopeKind::AppConfig`, 0 `ScopeKind` mới"*. **Tiền lệ đó KHÔNG áp được ở đây**, và lý do là hình dạng dữ liệu:

- `KEY_DICT_DISABLED` chở **một tập mã ngắn** (`"cvdict,thieuchuu"`) — một chuỗi phẳng.
- Một mục ghim chở **nhiều trường có cấu trúc**: nguồn (`source_code`), `entry_id`, đầu mục (`headword`), nghĩa rút gọn để hiện lại mà không phải tra lại, thời điểm ghim, số lần tra.
- `CONFIG_VALUE_DDL` (`schema.rs:98-105`) là `(kind, key) → TEXT`. Nhồi một danh sách bản ghi vào đó là dựng lại đúng lược đồ EAV mà doc-comment `schema.rs:75-97` cấm bằng chữ: *"Một bảng, không phải ba — và không phải một bảng cho MỌI loại"*.
- Nếu Quyết định #1 chọn (a) thì `config_value` **không tồn tại** trong `project.db` — câu hỏi tự đóng.

**ĐỀ XUẤT: một bảng `pinned_entry` mới, một bước di trú mới, 0 `ScopeKind` mới.** DDL đề xuất ở §Dev Notes ④.

### Quyết định #3 — Lịch sử sống ở RAM, không đi qua `core::store`

AC4 đòi lịch sử **kết thúc khi đóng ứng dụng**. Cài đặt đúng nhất là state module-level Vue thuần — nó thoả AC4 **về cơ học**, không cần một dòng mã xoá nào (một lượt ghi rồi xoá lúc thoát là hai chỗ để sai). AD-1 (`ARCHITECTURE-SPINE.md:79`) cho phép: frontend giữ *"state UI (focus, cuộn, vùng chọn, bố cục panel)"* — lịch sử tra là danh sách thao tác trong phiên, không một quy tắc nghiệp vụ.

`localStorage` **bị cấm tường minh** (Story 1.19 Quyết định #1c, AD-1 + FR103).

**ĐỀ XUẤT: state module-level, cùng khuôn `lookupPanelState.ts`.** 0 IPC command mới cho lịch sử.

### Quyết định #4 — HAI tab, không ba. Mockup đếm cả một tab chưa tồn tại.

`lookup-history-pins.html:103` vẽ ba tab: `Từ điển` · `Concordance` · `Lịch sử`.

**Đo trên mã thật (2026-08-10):** `grep -rn "Concordance" src/ src-tauri/src/` trả **0 lần trong `src/`**, và đúng 2 doc-comment ở `src-tauri/src/commands/dict.rs:119,173` — cả hai nói Concordance là **FR64, Story 7.7**, một năng lực khác chưa dựng. Story 1.17 §KHÔNG-LÀM đã cấm trỏ tới năng lực chưa tồn tại, và AD-34 áp cùng tinh thần.

⇒ Panel Lookup hôm nay có **0 tab nội bộ** (đo: `grep "tablist" src/panels/LookupPanel.vue` trả rỗng). Story này dựng dải tab đầu tiên, với **hai** tab: `Từ điển` và `Lịch sử`.

**Chữ "tab thứ ba" trong AC5 là ngôn ngữ của mockup, không một phép đếm phải khớp.** Mệnh đề thật của AC5 là *"trong Panel Lookup, không phải một cửa sổ riêng"* — và điều đó được thoả. Ghi lệch này vào Change Log; **không sửa mockup** (Quyết định #3 của Story 1.3: dev không sửa tài liệu quy hoạch).

### Quyết định #5 — LOẠI BỎ thanh bộ lọc ba chip của mockup

`lookup-history-pins.html:106-111` vẽ ba chip: `Phiên này` (chọn) · `Chương 47` · `Cả Tác phẩm`.

**Hai chip sau MÂU THUẪN trực tiếp với AC4.** *"Cả Tác phẩm"* chỉ có nghĩa nếu lịch sử sống xuyên phiên và được khoá theo Tác phẩm — tức chính xác thứ AC4 nói là **không**: *"lịch sử của phiên kết thúc — đây là lịch sử trong phiên"*. Một bộ lọc phạm vi trên một tập dữ liệu chỉ có đúng một phạm vi là một hứa hẹn rỗng.

**ĐỀ XUẤT: bỏ cả ba chip. Giữ nút *"Xoá lịch sử phiên"*** (nó có nghĩa thật, và mockup gán nó phím riêng). Ghi lệch này vào Change Log.

Nếu về sau A9 được xác nhận và người dùng thật đòi lịch sử xuyên phiên, đó là một story mới với một AC mới — không một bộ lọc dựng sẵn cho một dữ liệu chưa tồn tại.

### Quyết định #6 — Phím mặc định: theo tiền lệ 1.19, không theo mockup

Mockup ghi `⌘D` (ghim) và `⌘⌫` (xoá lịch sử). Nhưng `src/commands/index.ts:603-608` đã chốt bằng chữ, cho chính ba command của Story 1.19:

> *"**Cố ý KHÔNG gán phím mặc định cho cả ba** [...]: họ `Mod+Alt+…` đã kín chỗ có nghĩa (`1` `2` `O` `J` `V` `L` `S` `←` `→`), và ba thao tác này đều tới được bằng bàn phím qua Tab + Enter/Space — chuẩn HTML gốc. Đây là một **lỗ NFR17 có tên và có chủ**: màn hình gán phím là Story 1.21, và `chordsFor(id, bindings, undefined)` nghĩa là một hợp âm người dùng tự đặt trong `global.db` ĐƯỢC dùng ngay hôm nay."*

Đo hợp âm đang chiếm: `Mod+1/2/3` · `Mod+Alt+1/2` · `Mod+Alt+O/J/V/L/S` · `Mod+Alt+ArrowLeft/Right` · `Shift+Arrow…` · `Alt+Shift+Arrow…`. `Mod+D` và `Mod+Backspace` đều **trống**.

**ICE CHỐT 2026-08-10 — đúng MỘT ngoại lệ so với tiền lệ 1.19:**

| id | Hợp âm mặc định |
|---|---|
| `lookup.toggle_pin` | **`Mod+D`** — theo mockup (`⌘D`), thao tác tần suất cao nhất của story |
| `lookup.select_tab_record` · `lookup.select_tab_history` · `lookup.clear_history` | `chordsFor(id, bindings, undefined)` — 0 phím mặc định, theo tiền lệ 1.19 |

Viết `chordsFor('lookup.toggle_pin', bindings, ['Mod+D'])` — **`Mod`**, không `Cmd`/`Ctrl`: `keys.ts` là tầng trung lập nền tảng, và một hợp âm viết cứng theo một hệ điều hành là đúng thứ Kiểm D (NFR14) tồn tại để chặn.

**Ba phép đo đứng sau `Mod+D` (2026-08-10):**
1. **Trống hoàn toàn** — `grep -rn "Mod+D\|KeyD" src/ scripts/` trả 0 lần; hợp âm đang chiếm là `Mod+1/2/3` · `Mod+Alt+1/2` · `Mod+Alt+O/J/V/L/S` · `Mod+Alt+←/→` · `Shift+Arrow…` · `Alt+Shift+Arrow…`. Không có menu Tauri nào (`tauri.conf.json` không khai menu) nên không phím hệ thống nào tranh.
2. **Luật vùng gõ không chặn nó** — `keys.ts:227,287`: `lacksPrimaryMod(m) = !m.meta && !m.ctrl`, và luật vùng gõ chỉ áp khi vị từ đó `true`. `Mod+D` mang phím bổ trợ chính ⇒ ghim được **kể cả khi caret đang trong một ô nhập**, đúng lời hứa NFR17 ghi ngay tại `keys.ts:237-238`.
3. **`createKeymap` không ném** — không command nào đang giành hợp âm này.

**Không dùng `⌘⌫`** cho `lookup.clear_history`. `Mod+Backspace` trong một vùng gõ là *"xoá tới đầu dòng"* trên macOS, và theo phép đo #2 ở trên, một hợp âm mang `Mod` **không** được luật vùng gõ chặn — tức nó sẽ cướp phím ngay giữa lúc người dùng đang gõ, để xoá lịch sử. Đó là một thao tác phá hoại đi bằng một phím quen tay làm việc khác. Xoá lịch sử tới được bằng Tab + Enter, và gán lại được ở Story 1.21.

⚠️ **Rủi ro đã biết, hôm nay bằng 0:** nếu một `global.db` đã gán `Mod+D` cho một command khác thì `createKeymap` ném và **ứng dụng không mở được** — `installCommands()` chạy trước `mount()` (Bẫy 5 của Story 1.19, `src/commands/index.ts:646-650`). Hôm nay chưa có đường nào để người dùng gán phím (màn hình gán phím là Story 1.21), nên rủi ro chỉ thành thật khi 1.21 tồn tại — và 1.21 phải xử xung đột chứ không im lặng ghi đè, đúng AC3 của chính nó.

### Quyết định #7 — `settings.html` KHÔNG phải nguồn cho command id

`settings.html:282` ghi mã lệnh `lookup.entry.pin`. Nhưng cùng bảng đó ghi `lookup.query.selection` cho Auto-Lookup — mã thật là **`lookup.lookup_selection`** — và ghi `editor.segment.nextUntranslated`, thứ **vi phạm văn phạm id** của chính registry: `COMMAND_ID_RE = /^[a-z0-9]+(\.[a-z0-9_]+)+$/` (`src/commands/registry.ts:62`) không cho chữ hoa. `register()` sẽ **ném**.

⇒ Cả bảng mã lệnh của `settings.html` là phác thảo trước khi văn phạm được chốt ở Story 1.6. **Đặt id theo mã thật đang chạy**, không theo mockup.

**ĐỀ XUẤT id (hai đoạn, cùng khuôn `source.select_tab_original` / `lookup.toggle_source`):**

| id | Nhãn (`command.<id>` trong `vi.json`) | Handler port trong `CommandDeps` | Hợp âm (QĐ #6) |
|---|---|---|---|
| `lookup.select_tab_record` | Chuyển sang tab Từ điển | `selectLookupTab` | `undefined` |
| `lookup.select_tab_history` | Chuyển sang tab Lịch sử | `selectLookupTab` | `undefined` |
| `lookup.toggle_pin` | Ghim hoặc bỏ ghim mục từ đang xem | `toggleLookupPin` | **`Mod+D`** |
| `lookup.clear_history` | Xoá lịch sử tra cứu của phiên | `clearLookupHistory` | `undefined` |

Ghi lệch mockup này vào Change Log.

---

## Tasks / Subtasks

- [x] **Task 0 — chốt năm quyết định còn lại** *(AC: mọi AC)*
  - [x] ~~**#1 Ice ký 2026-08-10** — ghim ở `project.db`~~ ⇒ **ICE KÝ LẠI 2026-08-11: `global.db`, phạm vi TOÀN ỨNG DỤNG.** Lý do đo được ở §Change Log: không tồn tại đường mở lại một `.atproj`, nên AC3 **không nghiệm thu được** ở phạm vi Tác phẩm. Bước di trú vào `GLOBAL_MIGRATIONS` `to_version: 3`; bước 4 của `PROJECT_MIGRATIONS` bị **gỡ**
  - [x] **#6 Ice ký 2026-08-10 — không mở lại**: `lookup.toggle_pin` mang `Mod+D`, ba command còn lại 0 phím mặc định
  - [x] Đọc năm quyết định còn lại (#2 #3 #4 #5 #7); xác nhận hoặc phản biện **bằng số**, ghi kết quả vào §Change Log trước khi gõ dòng mã đầu tiên
  - [x] Xác nhận `git status --porcelain` sạch; nếu bẩn, đề xuất commit riêng phần vá cũ và **hỏi Ice trước khi commit**
  - [x] Ghi lại số thật hiện tại của mọi sàn `*_FLOOR` sẽ chạm (để AC13 có mốc so)

- [x] **Task 1 — tầng dữ liệu: bảng ghim + di trú** *(AC2, AC3)*
  - [x] `PINNED_ENTRY_DDL` + `Migration { to_version: 3, … }` vào **`GLOBAL_MIGRATIONS`** (`schema.rs`). `PROJECT_MIGRATIONS` về **đúng ba bước** như trước story — bước 4 của bản đầu bị gỡ, và số 4 thành một số **đã cháy** (bước kế tiếp của `project.db` phải đánh số 5)
  - [x] Test Rust: `a_fresh_database_migrates_up_to_target_and_logs_it` **ĐỎ THẬT** sau lượt ký lại — nó chạy trên `GLOBAL_MIGRATIONS`, và nay bảng ghim ở đó. Cập nhật target 2→3, sổ di trú 2→3 bản ghi, thêm assert `pinned_entry` có mặt. Fixture `TWO_STEP`/`BROKEN_STEP_TWO` **không** đụng. *(Bản đầu đặt bảng ở `project.db` nên ca này KHÔNG đỏ — tiền đề của story sai ở lượt đó, đúng ở lượt này.)*
  - [x] Test Rust: một `global.db` ở phiên bản 2 di trú lên 3 và **giữ nguyên `config_value`** — ca thật của mọi người dùng đã chạy bản 1.8–1.19
  - [x] Test Rust: `the_pin_table_lives_in_the_global_store_not_the_project_one` — lưới của lượt đổi phạm vi, đỏ ở **cả hai** hướng đi sai
  - [x] Test Rust: `pins_survive_closing_and_reopening_the_store` — **AC3**, vòng ghi → đóng → mở → đọc

- [x] **Task 2 — IPC: đọc/ghi/xoá mục ghim** *(AC2, AC3, AC12)*
  - [x] Ba hàm **thuần** nhận `Option<&Store>` + ba `#[tauri::command]` mỏng trong `mod wire` (khuôn `commands/config.rs`, `try_state` không `state()`)
  - [x] Đăng ký trong `generate_handler![…]` ở `src-tauri/src/lib.rs`
  - [x] Lỗi đi bằng `IpcError` qua `From<StoreError>` sẵn có — **0** `MessageKey` mới; mọi nhánh thuộc từ vựng **kho**, cùng ba nhánh của `commands::config`
  - [x] Mọi chuỗi literal trong `src-tauri/src/**` viết **không dấu** — `check:i18n` Kiểm A xanh
  - [x] Adapter TypeScript `src/config/pinned.ts`, khuôn `src/config/chapter.ts`

- [x] **Task 3 — state: lịch sử trong phiên + bộ ghim đã nạp** *(AC1, AC4, AC7, AC11, AC12)*
  - [x] Module state mới `src/panels/lookupHistoryState.ts`, cùng khuôn `lookupPanelState.ts`
  - [x] **KHÔNG** `import` module này vào `src/commands/index.ts` — tiêm qua `CommandDeps`, nối ở `src/main.ts` (Kiểm C/D/E xanh)
  - [x] Ghi lịch sử tại **đúng một** chỗ: cuối `runLookup`, sau guard `mine !== sequence` — `grep -rn "recordLookup" src/` trả **đúng một** chỗ gọi (AC11)
  - [x] Dedupe theo AC7: khớp ⇒ đẩy lên đầu + tăng đếm, không thêm hàng
  - [x] `resetLookupPanel()` vứt lịch sử (AC12 vế thứ nhất). **Vế thứ hai tự rụng** sau lượt ký lại: ghim không thuộc Tác phẩm nào nên không có gì để nạp lại

- [x] **Task 4 — giao diện: dải tab + tab Lịch sử** *(AC1, AC2, AC5, AC8, AC9, AC10)*
  - [x] Dải `role="tablist"` theo khuôn `SourcePanel.vue`: `tabindex` roving, mũi tên trái/phải đổi tab, `aria-selected`/`aria-controls`, mỗi `@click` là **đúng một** `dispatch('<id>')`
  - [x] Dải tab là hàng **RIÊNG** (`.lookup-tabs`, `flex: none`), không nhồi vào `.lookup-head`; `--lookup-head-height` vẫn `76px` (AC10, §Bẫy 2)
  - [x] Nút *"Nguồn dữ liệu"* vẫn bấm được ở **mọi** tab — §Bẫy 1 đường (b): dải `.lookup-sources` **không** bọc `v-if` theo tab, nên nút không đi đâu cả (AC9)
  - [x] Hai trạng thái rỗng riêng + **hai** ca mockup không vẽ *(ca `pinned_no_work` bị gỡ cùng phạm vi Tác phẩm; Bẫy 4 từ bốn trạng thái xuống ba)*
  - [x] `showFrameStatus = neverLookedUp && lookupTab === 'record'` — AC14, §Bẫy 10
  - [x] Mọi text node qua `t()` — Kiểm A2 xanh; dữ liệu dùng `<!-- aura-allow-text: … -->` kèm lý do
  - [x] Màu chỉ từ token — `check:tokens` xanh

- [x] **Task 5 — command + i18n** *(AC6)*
  - [x] Bốn command đăng ký trong `registerAll()`, id theo Quyết định #7
  - [x] Ba field mới trong `CommandDeps` (`selectLookupTab` phục vụ **hai** command — cùng khuôn `selectSourceTab`), nối handler thật ở `src/main.ts`
  - [x] **24** khoá mới trong `src/i18n/vi.json` *(105 → 129, đếm lại ở lượt code review 2026-08-11)* — object phẳng, khoá chấm; Kiểm B/D xanh

- [x] **Task 6 — nâng sàn và chạy đủ chín cổng** *(AC13)*
  - [x] **Chín trên chín cổng XANH.** `check:scope` ban đầu trượt vì một phiên `tauri dev` của Ice giữ cổng 1420; Ice đóng phiên đó lúc 21:08 và lượt chạy lại cho `VERDICT: PASS`
  - [x] `npm run build` (hai lượt `vue-tsc --noEmit` + `vite build`) — xanh
  - [x] `cargo test` trong `src-tauri/` — **259 ca xanh, 0 đỏ** (5 ignored), gồm 8 ca mới
  - [x] Nâng **mọi** sàn bị vượt theo số thật, ghi số vào §Completion Notes

- [ ] **Task 7 — nghiệm thu chạy tay, có đối chứng âm** *(mọi AC)*
  - [x] **9 trên 18 hàng ĐÃ CHẠY và ĐẠT**, đo bằng máy trên đường dữ liệu thật — xem §Debug Log References về bộ đo. Hàng **1** (AC1) · **2** (AC7, có đối chứng âm) · **3** (AC2, có đối chứng âm) · **7** (AC9, cả hai tab) · **8** (AC10, cả hai tab) · **13** (AC14, hai chiều) · **15** (`pinned_load_failed`) · **16** (`pinned_no_work`) · một phần **6** (AC8)
  - [ ] **9 hàng CÒN LẠI cần app Tauri thật** — không đo bằng máy được, và ghi rõ vì sao: hàng **4**/**5** (AC3/AC4) đòi **đóng rồi mở lại tiến trình**; **10** (AC12) đòi hai Tác phẩm trên đĩa; **11** (AC5) đòi đổi preset bố cục thật; **12** đòi `prefers-reduced-motion` ở tầng hệ điều hành; **14** đòi bấm phím ghim khi chưa nhắm mục; **17**/**18** đòi `Mod+D` trên **cả macOS lẫn Windows** (NFR14); **9** vế Tab+Enter đầy đủ. Cộng **ảnh chụp màn hình thật** cho mỗi AC thị giác
  - [x] Ghi mọi lệch mockup và mọi món nợ mới vào `deferred-work.md` — 9 mục mới

### Review Findings

*Lượt code review 2026-08-11 — ba lớp song song (Blind Hunter · Edge Case Hunter · Acceptance Auditor), mọi phát hiện đã đọc lại mã tại chỗ và chạy lại phép đo trước khi chấm mức.*

🔴 **MỘT việc còn treo — story ở `in-progress`, KHÔNG `done`:**
1. **9 hàng bàn đo chạy tay** (§Testing) — Decision 1 chỉ thu hẹp vế máy làm được. Hàng **4**/**5** (AC3/AC4, đóng-mở tiến trình thật) · **10** (AC12) · **11** (AC5) · **12** (`prefers-reduced-motion`) · **14** · **17**/**18** (`Mod+D` trên **cả** macOS lẫn Windows) · **9** vế Tab+Enter đầy đủ. Riêng hàng **9** nay có một mảnh mới cần đo: mũi tên trái/phải phải **dời được tiêu điểm** qua lại giữa hai tab, không chỉ đổi tab.

*(Việc thứ hai — một lượt `bmad-correct-course` cho `epics.md` — đã **HUỶ** ngày 2026-08-11 sau khi mở `epics.md` ra đo. Xem Decision 2 bên dưới.)*

**Sau lượt vá, đo lại 2026-08-11:** chín trên chín cổng XANH · `npm run build` XANH · `cargo test` **261 xanh, 0 đỏ, 5 ignored** *(259 → 261, hai ca mới)*.

- [x] [Review][Decision] **9/18 hàng bàn đo chưa chạy — AC3 · AC4 · AC5 chưa có một bằng chứng runtime nào** *(mức: cao)* — AC3 là lời hứa đầu bảng của story (*"đóng rồi mở lại, mục ghim vẫn còn"*) và hôm nay nó chỉ có bằng chứng ở tầng kho (`pins_survive_closing_and_reopening_the_store` — vòng `write → close → open → read` trên `Store`), **không** một lượt nào đi qua mối nối thật: tiến trình Tauri khởi động lại → `main.ts:325 void loadPinnedEntries()` → `LookupPanel`. Bộ đo CDP của §Debug Log References tự khai đúng giới hạn này (*"không đo được thứ cần một tiến trình khởi động lại"*). ⇒ **ICE CHỐT 2026-08-11: dựng thêm lưới bằng máy.** Thu hẹp khoảng trống AC3 tới mức máy làm được, rồi Ice chỉ chạy tay phần thật sự đòi hai nền tảng (hàng 17/18) và phần thị giác. Thành mục `[Review][Patch]` bên dưới.
- [x] [Review][Decision] ~~**AC12 đảo chiều nhưng `epics.md` chưa biết**~~ ⇒ **PHÁT HIỆN BỊ RÚT, 2026-08-11. `epics.md` KHÔNG cần sửa một chữ.**

  🔴 **Phát hiện gốc SAI, và nó sai ở một phép đo không ai chạy.** Lượt review đọc AC12 trong story file rồi **suy ra** nó có nguồn ở `epics.md`, không mở `epics.md` ra kiểm. Đo lại: Story 1.20 ở `epics.md:1845-1878` mang **đúng sáu** AC (AC1–AC6). **AC7–AC14 — gồm AC12 — không tồn tại ở đó**; chúng do người dựng story tự dẫn xuất, và chính story ghi rõ điều đó ở mục *"AC bổ sung — dẫn xuất từ mockup, kiến trúc và đo đạc mã nguồn"*. ⇒ không có xung đột nào giữa tài liệu quy hoạch và mã để mà `correct-course`. Đã kiểm cả `prd.md`: FR41 (`:504`) và giả định A9 (`:1080`) **không nói gì** về phạm vi ghim.

  ⚠️ **Ứng viên thứ hai cũng bị bác, và bác bằng lý lẽ của Ice.** Lượt rà tiếp theo nêu `epics.md:1873` (*"là **tab thứ ba** của Panel Lookup"*) như một lệch thật, vì mã dựng **hai** tab. **Ice bác 2026-08-11: đó không phải một lệch.** Tab thứ ba là **Concordance** (FR64, Story 7.7) — một tab **chưa được triển khai**, không một tab bị bỏ. AC5 mô tả **trạng thái cuối** và nó **đúng**: hôm nay tab Lịch sử tạm đứng thứ hai, và nó **thành** thứ ba đúng lúc Story 7.7 chèn Concordance vào giữa. Một AC mô tả đích đến không lệch chỉ vì đường đi chưa tới nơi.

  🔴 **Hệ quả có tên cho Story 7.7:** Concordance phải vào **GIỮA** `Từ điển` và `Lịch sử`, không nối vào đuôi. Chèn sai chỗ làm AC5 của story này vĩnh viễn không thoả trong khi mọi cổng vẫn xanh. Ghi vào `deferred-work.md`.

  ⇒ **`correct-course` HUỶ.** 0 tệp quy hoạch bị chạm — đúng Quyết định #3 của Story 1.3.
- [x] [Review][Patch] `toggleLookupPin` tính hướng ghim/bỏ-ghim từ `pinnedRaw` chưa cập nhật — hai lượt bấm nhanh cho hai lượt GHIM thay vì ghim-rồi-bỏ [src/panels/lookupHistoryState.ts:464]
- [x] [Review][Patch] Mũi tên trái/phải trên dải tab đổi tab nhưng **không** dời tiêu điểm DOM — hợp đồng `tabindex` roving khai một nửa, và người dùng bàn phím kẹt ở tab thứ hai [src/panels/LookupPanel.vue:395]
- [x] [Review][Patch] Ba chú thích còn sót lại phạm vi CŨ sau lượt ký lại 2026-08-11 — một trong số đó mô tả sai chính hàm ngay dưới nó [src-tauri/src/lib.rs:88 · src/panels/lookupPanelState.ts:330 · src-tauri/src/core/store/schema.rs:256]
- [x] [Review][Patch] Ba con số trong §Completion Notes / §Task 5 / §File List không khớp số đo lại — vi phạm chính AC13 [_bmad-output/implementation-artifacts/1-20-lich-su-tra-cuu-va-muc-da-ghim.md:759]
- [x] [Review][Patch] `resetLookupHistory()` không rào lượt ghi ghim đang bay — một lỗi ghi về muộn dựng lại banner của Tác phẩm đã rời [src/panels/lookupHistoryState.ts:344]
- [x] [Review][Patch] *(từ Decision 1)* Dựng thêm lưới bằng máy thu hẹp khoảng trống AC3 — hôm nay `pins_survive_closing_and_reopening_the_store` dừng ở tầng `Store`, không chạm mối nối `commands::pinned` [src-tauri/tests/pinned_contract.rs]
- [x] [Review][Defer] Không token thứ tự giữa phản hồi `loadPinnedEntries()` và phản hồi `pinWriteQueue` [src/panels/lookupHistoryState.ts:365] — deferred, hôm nay chỉ có một lượt nạp và không đường nào tới được
- [x] [Review][Defer] `sessionLookupCount` về 0 khi hàng lịch sử bị `HISTORY_CEILING` đẩy ra [src/panels/lookupHistoryState.ts:227] — deferred, đã có mục nợ về `HISTORY_CEILING`
- [x] [Review][Defer] *"Số 4 đã cháy"* của `PROJECT_MIGRATIONS` chỉ được cưỡng chế bằng văn xuôi [src-tauri/src/core/store/schema.rs:284] — deferred, hành vi hôm nay an toàn (`store.schema_too_new`)
- [x] [Review][Defer] `SourcePanel.vue` mang **cùng** khuyết tật tiêu điểm dải tab, có từ Story 1.18 [src/panels/SourcePanel.vue:113] — deferred, pre-existing

---

## Dev Notes

### ① Điểm cắm lịch sử — MỘT chỗ, đã dựng sẵn cho story này

`src/panels/lookupPanelState.ts:266-292`. Trạng thái hiện tại của hàm:

```ts
export async function runLookup(rawQuery: string): Promise<void> {
  const trimmed = rawQuery.trim()
  if (trimmed === '') return

  const mine = ++sequence

  query.value = trimmed
  pending.value = true
  const { response: result, error: err } = await lookupDictionary(trimmed)

  // Lượt này đã bị một lượt mới hơn — hoặc bởi `resetLookupPanel()` — vượt mặt.
  if (mine !== sequence) return

  pending.value = false
  resolvedQuery.value = err === null ? trimmed : null
  response.value = result
  error.value = err
}
```

**Chèn lời ghi lịch sử sau dòng `error.value = err`, KHÔNG trước guard `mine !== sequence`.** Ghi trước guard làm một lượt tra đã bị vượt mặt (hoặc đã bị `resetLookupPanel()` huỷ) vẫn để lại một dòng lịch sử — tức lịch sử của Tác phẩm A rò sang Tác phẩm B, đúng thứ `sequence` tồn tại để chặn.

**Ghi khi nào:** `err === null`, **kể cả** khi `groups` rỗng. Mockup có một hàng lịch sử ghi *"không nguồn nào có mục từ này"* (`lookup-history-pins.html`, hàng 鐵鏽) — *"đã tra mà không thấy"* là một sự kiện thật đáng nhớ. **Không ghi** khi `err !== null`: *"không tra được"* khác *"tra mà không thấy"*, và trộn hai thứ là đúng bẫy `??` mà Story 1.17 đã bắt (§Bẫy 5).

Ba nguồn độc lập xác nhận đây là điểm nghẽn duy nhất, và cả ba đã ghi tên story này bằng chữ:
- `src/main.ts:309-313` — *"Story 1.20 (lịch sử tra cứu) chỉ có một chỗ để cắm vào"*
- `src/panels/dictSourcesState.ts:171` — *"đi **qua** điểm nghẽn của Story 1.17 chứ không **quanh** nó, nên Story 1.20 (lịch sử) vẫn chỉ có một chỗ để cắm vào"*
- `1-18-auto-lookup.md:83-91` — *"1.20 chỉ được phép **thêm một dòng ghi** vào chỗ đó, không phải đi tìm hai đường gọi rải rác"*

### ② Cấu trúc hiện tại của `LookupPanel.vue` — đọc trước khi sửa

Thân panel hôm nay, theo thứ tự trong `<template>`:

```
<PanelFrame owner="panel.lookup" status-key="panel.lookup.status" :show-status="showFrameStatus">
  <div ref="body" class="lookup-body">              ← useSelectionSurface(body, 'display') ở dòng 95
    <div v-if="dictSources.length > 0" class="lookup-sources">   ← Story 1.19, flex: none
      <span class="lookup-sources-label">Nguồn</span>
      <div class="lookup-sources-chips"> …chip mỗi nguồn… </div>  ← cuộn riêng, max-height 52px
      <button data-attribution-open @click="dispatch('attribution.open')">Nguồn dữ liệu</button>
    </div>
    <div class="lookup-head">                        ← --lookup-head-height: 76px; overflow: hidden
      <p class="lookup-headword">{{ currentQuery }}</p>
      <div class="lookup-spine"> …thanh nhịp… </div>
      <div v-if="showProgress" class="lookup-progress"></div>    ← position: absolute, đáy
    </div>
    <div ref="scroller" class="lookup-scroll">        ← flex: 1; overflow: auto — VÙNG CUỘN
      …bốn trạng thái rỗng · hai banner · LookupRecord cho mỗi nguồn…
    </div>
  </div>
</PanelFrame>
```

**Cái gì đổi:** thêm một dải `role="tablist"` và một nhánh nội dung thứ hai cho `.lookup-scroll`.
**Cái gì PHẢI giữ nguyên:** `--lookup-head-height: 76px` · `overflow: hidden` của `.lookup-head` · `position: absolute` của `.lookup-progress` · nút `data-attribution-open` bấm được ở mọi tab · `useSelectionSurface(body, 'display')` trên `body`.

### ③ Bề mặt vùng chọn — KHÔNG cần đăng ký mới, và đây là một phép đo

`LookupPanel.vue:95` gọi `useSelectionSurface(body, 'display')` trên `body`, tức trên **`.lookup-body`** — hộp ngoài cùng bao trọn cả `.lookup-sources`, `.lookup-head` và `.lookup-scroll`. Nội dung tab Lịch sử nằm trong `.lookup-scroll`, tức **đã** nằm trong bề mặt đã đăng ký, với đúng vai `'display'`.

⇒ **`SELECTION_SURFACE_FLOOR` không cần nâng** (hiện = 6, `check-commands.mjs:1648`), và Kiểm F không đòi gì thêm.

Vai `'display'` là bắt buộc, không `'source'`: bôi đen một đầu mục trong lịch sử để đọc kỹ mà phát ra một lượt tra mới sẽ **thay chính đoạn đang đọc dưới tay người đọc** — Bẫy 1 của Story 1.18, đã bắt lại lần hai ở Story 1.19 với bảng Attribution.

### ④ Bảng ghim — DDL đề xuất

Theo khuôn `WORK_DDL`/`CHAPTER_DDL` (`schema.rs:144-188`): `TEXT` cho thời gian, `INTEGER PRIMARY KEY AUTOINCREMENT` cho khoá thay thế, không `DEFAULT` ngầm.

```sql
CREATE TABLE pinned_entry (
  id          INTEGER PRIMARY KEY AUTOINCREMENT,
  source_code TEXT NOT NULL,
  entry_id    INTEGER NOT NULL,
  headword    TEXT NOT NULL,
  gloss       TEXT,
  pinned_at   TEXT NOT NULL,
  UNIQUE (source_code, entry_id)
);
```

`UNIQUE (source_code, entry_id)` là hợp đồng "ghim hai lần cùng một mục không sinh hai hàng" ở tầng lược đồ, không ở tầng ứng dụng — cùng doctrine `CHECK (id = 1)` của `WORK_DDL`.

`gloss` để `NULL` được: một mục ghim từ một lượt tra không có nghĩa nào lấy về vẫn ghim được. `headword` và `gloss` là **ảnh chụp** để hiện lại hàng mà không phải tra lại — chấp nhận rằng chúng cũ đi nếu tệp `.db` nguồn được thay ở một bản phát hành sau. Ghi giới hạn này vào `deferred-work.md`.

**Ghi ngay, không debounce.** AD-35 (`ARCHITECTURE-SPINE.md:425`): thao tác rời rạc dứt khoát của người dùng **không** được định tuyến qua bộ đệm gõ — *"một thao tác đã hoàn tất nằm chờ tới 5 giây và biến mất nếu app sập, dù người dùng thấy nó đã xong trên màn hình"*. Ghim là đúng loại thao tác đó. Đi thẳng qua `Store::write` (writer nối tiếp, AD-11).

### ⑤ Khuôn dải tab — sao chép từ `SourcePanel.vue:102-137`

```vue
<div class="tabs" role="tablist">
  <button
    id="source-tab-original"
    type="button"
    class="tab"
    role="tab"
    aria-controls="source-tabpanel"
    :aria-selected="activeTab === 'original'"
    :tabindex="activeTab === 'original' ? 0 : -1"
    :class="{ active: activeTab === 'original' }"
    @click="dispatch('source.select_tab_original')"
    @keydown.right.prevent="dispatch('source.select_tab_han_viet')"
    @keydown.left.prevent="dispatch('source.select_tab_han_viet')"
  >{{ t('panel.source.tab_original') }}</button>
  …
```

Năm thuộc tính là bắt buộc, không trang trí: `role="tab"` · `aria-selected` · `aria-controls` · `tabindex` roving (đúng **một** tab trong vòng Tab) · mũi tên trái/phải đổi tab. `SourcePanel.vue:100` ghi lý do bằng chữ: *"hợp đồng `tablist` khai một nửa còn tệ hơn không khai, vì nó hứa một mô hình [tương tác mà nó không giữ]"*.

`@click` là **đúng một** `dispatch('<id>')` — Kiểm A của `check:commands` quét tĩnh và sẽ đỏ nếu có bất kỳ thứ gì khác.

### ⑥ State tab phải sống ở module, không trong `<script setup>`

`activeTab` **không** được là một `ref` cục bộ của `LookupPanel.vue`. Lý do đã ghi ở `lookupPanelState.ts:6-10` và lặp lại ở `sourcePanelState.ts`: đổi preset bố cục gọi `api.clear()` rồi dựng lại cả bốn panel, và **chỉ** state module-level (singleton của tiến trình) sống sót qua lượt tháo/dựng đó. Một `ref` cục bộ làm tab tự nhảy về mặc định mỗi lần người dùng đổi bố cục.

### ⑦ Chuỗi giao diện — khoá và văn bản chính xác

Object phẳng trong `src/i18n/vi.json`, khoá chấm tiền tố miền (`ARCHITECTURE-SPINE.md:641`). Giọng **vô nhân xưng**, không xưng "bạn"/"chúng tôi" — Kiểm D của `check-i18n.mjs`.

Bốn nhãn command (`command.<id>` — bắt buộc, `check-commands.mjs` Kiểm E đối chiếu từng `labelKey` với `vi.json`):

```
"command.lookup.select_tab_record": "Chuyển sang tab Từ điển",
"command.lookup.select_tab_history": "Chuyển sang tab Lịch sử",
"command.lookup.toggle_pin": "Ghim hoặc bỏ ghim mục từ đang xem",
"command.lookup.clear_history": "Xoá lịch sử tra cứu của phiên",
```

Nhãn tab và tiêu đề mục (nhãn tab lấy từ `lookup-history-pins.html:103`):

```
"panel.lookup.tab_record": "Từ điển",
"panel.lookup.tab_history": "Lịch sử",
"panel.lookup.section_pinned": "Đã ghim",
"panel.lookup.section_recent": "Tra gần đây",
"panel.lookup.clear_history": "Xoá lịch sử phiên",
"panel.lookup.unpin": "Bỏ ghim",
```

Hai trạng thái rỗng — **văn bản nguyên văn từ mockup** (`lookup-history-pins.html:189-206`), giữ đúng từng chữ:

```
"panel.lookup.history_empty_title": "Chưa tra gì trong phiên này",
"panel.lookup.history_empty_body": "Bôi đen một cụm ở bất kỳ panel nào — mỗi lượt tra tự vào đây, không cần lưu tay.",
"panel.lookup.history_empty_note": "Lịch sử xoá khi đóng ứng dụng",
"panel.lookup.pinned_empty_title": "Chưa ghim mục nào",
"panel.lookup.pinned_empty_body": "Ghim một mục từ đang xem để nó theo bạn qua các phiên và các Chương của Tác phẩm này.",
"panel.lookup.pinned_empty_note": "khi đang xem một mục từ",
```

⚠️ `pinned_empty_body` chứa chữ *"bạn"* — Kiểm D của `check-i18n.mjs` cưỡng chế giọng vô nhân xưng và có thể **từ chối** câu này. Chạy cổng trước khi tin; nếu đỏ, viết lại giữ nguyên nghĩa (ví dụ *"Ghim một mục từ đang xem để nó ở lại qua các phiên và các Chương của Tác phẩm này."*) và ghi lệch vào Change Log — **không** tắt Kiểm D.

Ba câu **không có trong mockup**, dẫn xuất từ AD-44 ④ (*"rỗng im lặng bị cấm; rỗng có lý do thì không"*) — mockup chỉ vẽ đường đi thuận, còn ba ca dưới đây là ca thật của hệ thống đang có:

```
"panel.lookup.pinned_no_work": "Chưa mở Tác phẩm nào — mục ghim thuộc về một Tác phẩm cụ thể.",
"panel.lookup.pinned_load_failed": "Không đọc được danh sách đã ghim — đây là một lỗi của ứng dụng, không phải một danh sách rỗng.",
"panel.lookup.pin_no_target": "Chưa nhắm được mục từ nào để ghim — chạm vào một mục trong kết quả tra cứu trước.",
```

Dòng gợi ý cuối tab, nguyên văn `lookup-history-pins.html:157-158` — nó là **quy tắc hành vi viết thành chữ cho người dùng**, không trang trí:

```
"panel.lookup.history_hint": "Ghim là của người, lịch sử là của máy. Mục ghim sống qua các phiên và theo Tác phẩm; lịch sử tra chỉ sống trong phiên và xoá được bất cứ lúc nào.",
```

### ⑧ "Mục từ đang xem" — command KHÔNG mang tham số, handler đọc trạng thái quanh nó

`lookup.toggle_pin` không nhận `entry_id`. Lý do là §KHÔNG-LÀM ⑤ viết thành chữ ký, và Story 1.19 đã giải đúng bài toán này cho `lookup.toggle_source`:

> *"danh sách nguồn **dẫn xuất lúc chạy**, còn `CommandRegistry` là một danh sách TĨNH mà `check-commands.mjs` đếm bằng máy (`COMMAND_FLOOR`). Một command cho mỗi nguồn phá chính cơ chế cưỡng chế của AD-34, và một id không tồn tại lúc dựng màn hình phím thì Story 1.21 không gán lại được."* — doc-comment của `toggleDictSource`, `src/commands/index.ts`

⇒ Cùng khuôn: **mục tiêu đi bằng `@mousedown` uỷ quyền ở vùng chứa**, handler đọc nó tại thời điểm chạy. Story 1.19 phải dùng `@mousedown` chứ không `document.activeElement`, và lý do đã đo trên WKWebView — đọc `src/panels/dictSourcesState.ts` trước khi tự chế cách khác.

**Ba nguồn "mục đang xem", theo thứ tự ưu tiên — chốt ở Task 0 nếu chưa rõ:**
1. Một `LookupRecord` vừa được `@mousedown` chạm ở tab Từ điển.
2. Nếu chưa chạm cái nào và kết quả chỉ có **đúng một** mục — lấy mục đó.
3. Không xác định được ⇒ **không làm gì im lặng**. Nói ra bằng một câu có lý do (UX-DR27, AD-44 ④), ví dụ `panel.lookup.pin_no_target`.

Ca 3 là bắt buộc, không tuỳ chọn: một phím bấm không có hiệu lực và không giải thích là đúng thứ *"rỗng im lặng"* mà `ARCHITECTURE-SPINE.md:622` cấm.

### ⑨ Số lần tra trên hàng ghim — thuộc phiên, không bền vững

Mockup hiện `12×` trên mỗi hàng ghim (`lookup-history-pins.html:116-131`). AC7 nói số đếm tăng khi tra lại.

**Đề xuất: số đếm sống trong lịch sử phiên, không trong bảng `pinned_entry`.** Ba lý do:
- Một số đếm bền vững cần một lượt ghi đĩa **mỗi lượt tra** — tức mỗi lần bôi đen chữ. Điều đó đưa một lượt `Store::write` vào đường nóng của Auto-Lookup và cạnh tranh hàng đợi ghi nối tiếp với auto-save Editor (NFR2, AD-11/AD-12).
- Không AC nào đòi số đếm sống qua phiên. AC3 chỉ đòi **mục ghim** còn.
- Ghim và lịch sử khớp nhau qua `(source_code, entry_id)`, nên hàng ghim tra được số đếm của phiên mà không cần cột riêng.

⇒ Hàng ghim hiện số đếm **của phiên này**; mở app mới thì đếm về 0 và hàng ghim vẫn còn. Nếu Ice muốn số đếm bền vững, đó là một cột `lookup_count` + một luật ghi có tiết chế — **đo chi phí ghi trước**, đừng thêm cột rồi mới đo. Ghi hướng này vào `deferred-work.md`.

### ⑩ Hình dạng hàng — ràng buộc thị giác từ mockup

**Hàng ghim** (`.row.pin`), trái sang phải: đầu mục gốc (15.5px, họ `read`, `min-width: 74px`) · âm đọc (11px, `on-surface-variant`, `min-width: 88px`) · nghĩa rút gọn **một dòng, `text-overflow: ellipsis`, `white-space: nowrap`** (13px, họ `read`, `flex: 1`) · nhãn nguồn (10px, hoa, đậm, `letter-spacing: .09em`, màu `primary`) · số lần tra (10.5px, `tm-text`, căn phải, `min-width: 34px`). Vạch dọc trái 2px màu **`tm`**.

**Hàng lịch sử** (`.row`): cùng bốn field đầu, nhưng **không** nhãn nguồn và **không** số đếm — thay bằng thời gian tương đối (`min-width: 56px`, căn phải). Hàng đang xem (`.cur`) có nền `surface-accent` + vạch trái **`primary`**.

Hai màu vạch khác nhau là ngữ nghĩa, không thẩm mỹ: `primary` được `DESIGN.md` §Do's dành cho đúng ba việc (thuật ngữ Glossary, nhãn nguồn từ điển, tiêu điểm bàn phím), nên vạch ghim dùng `tm`.

Hình dạng chủ đạo là **vạch dọc**, không hộp bo tròn (`DESIGN.md:354`). Giãn dòng hàng danh sách `1.66` — sàn của chữ họ `read` (`DESIGN.md:297`).

Chuyển động khi một hàng mới vào: **90 ms · `opacity` 0.4 → 1 · `ease-out`**, không `translate`, không `scale`; `prefers-reduced-motion` bỏ hẳn (`DESIGN.md:341-348`). `.lookup-scroll` đã có `.lookup-fade` với đúng thông số này — dùng lại, đừng viết một keyframes thứ hai.

**Dữ liệu trong mockup là MINH HOẠ, không chép cứng:** 聽潮閣/氣機/春秋/徐驍/走廊/鐵鏽/少年/推開/回頭, các số `12×`/`31×`/`218 lượt`, tên `Chương 47`, các mốc `vừa xong`/`2 ph`. Story 1.19 đã bị đúng lớp lỗi này với `sources-attribution.html` (mockup liệt kê nguồn không tồn tại) — bảng phải **dẫn xuất từ dữ liệu thật**, không từ bảng của mockup.

### ⑪ Khuôn IPC command — hai lớp, không một

Mọi command trong dự án đi theo đúng khuôn này (`commands/config.rs:165-192`): một hàm **thuần** nhận `Option<&Store>` (đường sản phẩm thật, test gọi trực tiếp không cần webview), rồi một `#[tauri::command]` **mỏng** trong `mod wire` chỉ lấy `State` qua **`try_state`** (không `state()` — `panic = "abort"` giết tiến trình nếu state chưa từng được `app.manage`) rồi gọi xuống.

```rust
pub mod wire {
    #[tauri::command]
    pub fn put_config(app: tauri::AppHandle, kind: String, key: String, value: String) -> Result<(), IpcError> {
        use tauri::Manager as _;
        let managed = app.try_state::<Store>();
        super::put_config(managed.as_deref(), &kind, &key, &value)
    }
}
```

Hình dạng lỗi cố định (AD-21, `core/i18n/mod.rs:216-239`): `{ code, message_key, params, retryable }`, serialize **snake_case** nguyên văn (không `rename_all`), `params` luôn `BTreeMap<String, String>` — số cũng đi dạng chuỗi. Khoá lại bởi `tests/ipc_contract.rs`. `From<StoreError> for IpcError` (`core/store/mod.rs:473-496`) là **chỗ duy nhất** một `StoreError` thành `IpcError`.

### ⑫ Bảng Stack — phiên bản ghim (`ARCHITECTURE-SPINE.md:664-689`)

Rust edition 2024 · `tauri` 2.11.5 · `@tauri-apps/api` 2.11.1 · `@tauri-apps/cli` 2.11.4 · `rusqlite` 0.40.1 (feature `bundled`) · `libsqlite3-sys` 0.38.1 · `serde` 1.0.229 · `serde_json` 1.0.151 · Vue 3.5.40 · TypeScript 5.9.3 · Vite 8.2.0 · `dockview-vue` 7.0.4 · `vue-tsc` 3.3.9.

**0 phụ thuộc mới cho story này.** Mọi thứ cần đã có: `rusqlite` cho bảng ghim, Vue `ref` cho lịch sử, `CommandRegistry` cho command. Nếu thấy cần một thư viện, đó là dấu hiệu đi sai đường — dừng và hỏi.

`tauri-plugin-sql`, `tauri-plugin-fs`, `tauri-plugin-dialog` **bị cấm tường minh** (`ARCHITECTURE-SPINE.md:710`); `check-deps` chạy `cargo tree -i` cho từng tên và trả mã thoát khác 0.

**Không cần tra cứu web cho story này** — không API ngoài, không thư viện mới, không bề mặt mạng. Mọi phiên bản đã ghim và đã đóng bởi `check:deps`.

### ⑬ Cây nguồn — file sẽ chạm

| File | NEW/UPDATE | Việc |
|---|---|---|
| `src-tauri/src/core/store/schema.rs` | UPDATE | `PINNED_ENTRY_DDL` + một `Migration` mới |
| `src-tauri/src/commands/dict.rs` *(hoặc file mới)* | UPDATE/NEW | ba command ghim, khuôn hai lớp |
| `src-tauri/src/lib.rs` | UPDATE | đăng ký command mới trong `generate_handler![…]` (dòng 79-88) |
| `src-tauri/tests/store_contract.rs` | UPDATE | cập nhật target version; thêm test di trú |
| `src/config/dict.ts` *(hoặc file mới)* | UPDATE/NEW | adapter IPC + type mục ghim |
| `src/panels/lookupPanelState.ts` | UPDATE | một dòng ghi lịch sử trong `runLookup`; `resetLookupPanel` vứt lịch sử |
| `src/panels/lookupHistoryState.ts` | NEW | state lịch sử + bộ ghim + `activeTab` |
| `src/panels/LookupPanel.vue` | UPDATE | dải tab + nhánh nội dung tab Lịch sử |
| `src/commands/index.ts` | UPDATE | bốn `CommandDeps` mới + bốn command trong `registerAll()` |
| `src/main.ts` | UPDATE | nối bốn handler thật |
| `src/i18n/vi.json` | UPDATE | ~16 khoá mới |
| `scripts/check-commands.mjs` | UPDATE | nâng sàn theo số thật (AC13) |
| `scripts/check-i18n.mjs` | UPDATE | nâng sàn **nếu** chạm |

Cây nguồn đã chốt: `ARCHITECTURE-SPINE.md:781-815`. Đặt tên: Rust `snake_case`; Vue component `PascalCase.vue`; khoá `vi.json` phẳng theo khoá chấm (`ARCHITECTURE-SPINE.md:641`). Panel Lookup gọi là `LookupPanel` trong mã (`:639`).

### ⑭ Ranh giới — §KHÔNG-LÀM

1. **Không** Concordance — FR64, Story 7.7. Không tab, không nút, không chỗ trống dành sẵn (Quyết định #4).
2. **Không** bộ lọc phạm vi lịch sử — mâu thuẫn AC4 (Quyết định #5).
3. **Không** đụng `src/commands/registry.ts` · `focus.ts` · `keys.ts` — cơ chế của Story 1.6 đã ổn định; chỉ sửa `index.ts`.
4. **Không** đụng `core/store/{writer,reader,checkpoint,pragmas}.rs` — tầng của Story 1.7 đã ổn định; chỉ thêm một bước di trú ở `schema.rs`.
5. **Không** một command cho mỗi mục ghim. `CommandRegistry` là danh sách **tĩnh** mà `check-commands.mjs` đếm bằng máy, và một id không tồn tại lúc dựng màn hình phím thì Story 1.21 không gán lại được. Handler đọc mục tiêu từ trạng thái quanh nó, đúng khuôn `deps.currentSelection` của `lookup.lookup_selection` và `deps.toggleDictSource` của Story 1.19 (`src/commands/index.ts` doc-comment của `toggleDictSource` viết lý lẽ này thành chữ ký).
6. **Không** thêm `vitest`/Playwright vào pipeline chính thức — quyết định giữ nguyên từ Story 1.5 qua mười story; nghiệm thu DOM bằng bàn đo chạy tay (§Testing).
7. **Không** sửa `epics.md`, `prd.md`, hay bất kỳ mockup nào — Quyết định #3 của Story 1.3. Lệch thì **ghi ra**, không vá tài liệu quy hoạch.

---

## Bẫy đã biết

### Bẫy 1 — đổi tab làm biến mất đường vào Attribution *(nghiêm trọng nhất)*

Nút *"Nguồn dữ liệu"* (`data-attribution-open`) sống **bên trong** `.lookup-sources`. Phản xạ tự nhiên khi thêm tab là bọc cả dải chip nguồn trong `v-if="activeTab === 'record'"` — dải chip chỉ có nghĩa ở tab Từ điển, đúng vậy. **Nhưng nút Attribution đi theo**, và với nó là đường chuột **duy nhất** vào màn hình ghi công (AC11 của Story 1.19).

Đây không phải rủi ro lý thuyết: `LookupPanel.vue` mang một khối chú thích dài ghi lại rằng ngày 2026-08-10 Ice **chạy app thật với mười nguồn** và phát hiện đúng nút này là thứ đầu tiên biến mất khi dải chip tràn — phải vá tay bằng cách tách nút ra `flex: none` ngoài vùng cuộn.

**Hai đường ra, chọn một và ghi lý do:**
- (a) Ẩn `.lookup-sources-chips`, **giữ** nhãn và nút Attribution ở mọi tab.
- (b) Giữ nguyên cả dải ở mọi tab — đơn giản nhất, và dải chip ở tab Lịch sử tuy thừa nhưng vô hại.

Nghiệm thu AC9 bằng **chuột thật ở cả hai tab**, không bằng đọc mã.

### Bẫy 2 — `.lookup-head` 76px đã vỡ HAI lần

`--lookup-head-height: 76px; overflow: hidden`. Đo ghi trong chính `LookupPanel.vue`: *"đầu mục 24px/1.3 ≈ 31px + `margin-top` 7px + thanh nhịp ≈ 15px + `padding-bottom` ⇒ vùng 76px **đã đầy**"*. Story 1.17 vỡ ở đây với thanh nhịp; Story 1.18 vỡ lần hai với vạch tiến trình; Story 1.19 phải tách dải chip **ra ngoài** thành hàng riêng.

Dải tab là thứ **thứ tư** muốn chỗ trong đó. Nó là một hàng **RIÊNG**, `flex: none`, ngoài `.lookup-head`. Nếu nó không vừa bố cục, **nói ra và đo** — đừng nới hằng trong im lặng.

### Bẫy 3 — `sequence` và lượt tra đang bay

`runLookup` tăng `sequence` mỗi lượt; `resetLookupPanel()` cũng tăng. Guard `if (mine !== sequence) return` là thứ giữ cho một lượt cũ về muộn không ghi đè lượt mới (bắt ở code review 2026-08-07, và Story 1.18 biến lỗ này thành thường trực).

Nếu bấm một hàng lịch sử để tra lại, **phải đi qua `runLookup`**, không tự chế một đường gọi song song. Một đường thứ hai bỏ qua `sequence` là dựng lại đúng lỗi đã vá.

### Bẫy 4 — `??` sập nhiều trạng thái làm một

Story 1.17 bắt được một `?? true` biến *"chưa tra được"* thành *"đã tra mà không có"*. Story này có **bốn** trạng thái dễ sập vào nhau ở tab Lịch sử:

| Trạng thái | Câu phải nói |
|---|---|
| Chưa mở Tác phẩm | `panel.lookup.pinned_no_work` |
| Đã mở, chưa ghim gì | `panel.lookup.pinned_empty_*` |
| Đang nạp bộ ghim từ đĩa | không nói gì — **không** nháy sang "chưa ghim mục nào" |
| Nạp bộ ghim trượt (lỗi IPC) | một câu lỗi riêng, không im lặng |

Mỗi trạng thái một vị từ riêng, cùng doctrine `lookupResolved` / `lookupDisplayable` mà `lookupPanelState.ts:154-174` dựng — hai vị từ trả lời hai câu hỏi khác nhau, và gộp chúng là bẫy đã bắt ở 1.16 lẫn 1.17.

### Bẫy 5 — hàm `export` mà không ai `import`

Bài học code review 1.16, lặp lại ở 1.17: một vị từ trạng thái được export mà không bề mặt nào tiêu thụ là **lỗi im lặng tuyệt đối** — `lookupError` từng là đúng thứ đó, và một lỗi IPC cho ra thân panel trắng câm. Ở Task cuối, kiểm bằng mắt rằng **mỗi** vị từ mới có một nơi tiêu thụ nhìn thấy được.

### Bẫy 6 — chú thích `.vue` nhắc tên thẻ `style`/`template` kèm ngoặc nhọn

`check-i18n.mjs` cắt tệp `.vue` thành ba khối bằng chính hai thẻ đó. Nhắc tên chúng trong một chú thích làm cổng cắt nhầm và **mọi** miễn trừ `aura-allow-text` phía dưới mất hiệu lực cùng lúc (bắt lúc chạy cổng, 2026-08-07). Ghi ngay trong `LookupPanel.vue` — đừng để bị lần nữa.

### Bẫy 7 — chuỗi Rust phải viết KHÔNG DẤU

Kiểm A của `check-i18n.mjs` quét `.rs` bằng máy trạng thái: chuỗi/`char` literal ở vị trí mã **không** được mang dấu tiếng Việt; doc-comment thì được. Miễn trừ chỉ ba chỗ có tên: `src-tauri/tests/**`, `src/selftest/**`, `tools/**`. `src-tauri/src/**` **không** trong danh sách. Viết `khong`, `du lieu`, `loi ghi`.

### Bẫy 8 — `event.repeat` bị chặn mặc định

`keys.ts:295` chặn `event.repeat === true` (`deferred-work.md:656`). Nếu thiết kế UX cho một phím giữ được (ví dụ giữ để duyệt danh sách), nó **không** chạy hôm nay, và nới nó cần một cờ `repeatable` trên `CommandSpec` — chạm `registry.ts` + `keys.ts` + mọi command đang có. **Chủ: Story 1.21.** Không mở trong story này.

### Bẫy 10 — câu trạng thái của `PanelFrame` rò sang tab Lịch sử *(đo được, không suy đoán)*

`PanelFrame.vue` render câu trạng thái **phía trên `<slot />`**, tức phía trên toàn bộ thân panel:

```vue
<p v-if="props.showStatus" class="status">{{ t(props.statusKey) }}</p>
<div class="panel-body"><slot /></div>
```

Và `LookupPanel.vue:59` truyền `:show-status="showFrameStatus"` với `showFrameStatus = computed(() => neverLookedUp.value)`.

⇒ **Mở app sạch rồi chuyển thẳng sang tab Lịch sử** — đúng ca nghiệm thu AC3 (*"đóng rồi mở lại, mục ghim vẫn còn"*) — cho `neverLookedUp === true`, nên panel hiện *"Chọn một từ trong Nguyên văn để tra cứu."* **đè trên danh sách ghim**. Câu đó sai ngữ cảnh: người dùng đang xem mục đã ghim, không đang chờ tra cứu.

**Vá:** `showFrameStatus` phải hỏi thêm tab đang chọn — `neverLookedUp && activeTab === 'record'`. Một dòng, nhưng **không** tìm ra được nếu chỉ đọc `LookupPanel.vue`: nguyên nhân nằm ở `PanelFrame.vue`, một file story này không định sửa.

### Bẫy 9 — đổi target version làm một test Rust đỏ, và đó là ĐÚNG

`schema.rs:110-116` nói bằng chữ: thêm một bước di trú làm `tests/store_contract.rs::a_fresh_database_migrates_up_to_target_and_logs_it` đỏ, *"và đó là hành vi đúng: số phiên bản đổi phải là một quyết định có người ký, chứ không phải một hiệu ứng phụ."* Cập nhật số trong test đó. **Đừng** "sửa cho nhất quán" các fixture `TWO_STEP`/`BROKEN_STEP_TWO` — chúng cục bộ và không phụ thuộc hằng này.

---

## Testing

**Không có bộ chạy test frontend** — quyết định giữ nguyên từ Story 1.5 qua mười story (§KHÔNG-LÀM ⑥). Vế DOM nghiệm thu bằng **bàn đo chạy tay**, và món nợ này nối dài `deferred-work.md:836-846`.

### Cổng bằng máy — chạy đủ, không chọn lọc

```
npm run check:deps && npm run check:tokens && npm run check:i18n && \
npm run check:commands && npm run check:layout && npm run check:scope && \
npm run check:scope:bundled && npm run check:dict && npm run check:dict-manifest
npm run build
cd src-tauri && cargo test
```

### Test Rust — quy ước

Integration test ở `src-tauri/tests/`, hai họ: `*_contract.rs` (hành vi/AC) và `*_boundary.rs` (ranh giới kiến trúc, quét mã nguồn tìm token cấm). Tên hàm là **câu tiếng Anh mô tả mệnh đề đang kiểm**, snake_case dài — `writes_are_serialized`, `every_store_error_converts_to_a_complete_ipc_error`. Không `test_xxx`.

Test bắt buộc cho story này:
- một bảng ghim mới di trú lên được từ phiên bản trước và **giữ nguyên dữ liệu cũ**
- ghim hai lần cùng một mục cho **đúng một** hàng (`UNIQUE` thật sự cưỡng chế)
- bỏ ghim rồi ghim lại cho trạng thái **giống hệt** trước khi bỏ *(đối chứng âm)*
- ghim ở Tác phẩm A rồi mở Tác phẩm B ⇒ bộ ghim của B **không** chứa mục của A *(nếu Quyết định #1a)*

### Bàn đo chạy tay — mỗi hàng một ảnh chụp màn hình thật

| # | AC | Thao tác | Kết quả phải thấy |
|---|---|---|---|
| 1 | AC1 | Tra 5 cụm khác nhau, mở tab Lịch sử | 5 hàng, **gần nhất trên cùng** |
| 2 | AC7 | Tra lại cụm thứ 3 | Hàng đó **lên đầu**, đếm +1, **vẫn 5 hàng** (đối chứng âm: không thành 6) |
| 3 | AC2 | Ghim một mục từ kết quả | Hàng xuất hiện ở mục "Đã ghim", đếm section +1 |
| 4 | AC3 | Đóng app, mở lại, mở tab Lịch sử | Mục ghim **còn**; đối chứng âm: lịch sử **rỗng** |
| 5 | AC4 | (cùng lượt #4) | Trạng thái rỗng lịch sử hiện đúng câu, không khung trắng |
| 6 | AC8 | App sạch, chưa tra, chưa ghim | **Hai** câu khác nhau, đúng chữ ở §Dev Notes ⑦ |
| 7 | AC9 | Đổi sang tab Lịch sử, bấm chuột nút "Nguồn dữ liệu" | Màn hình Attribution mở — **ở cả hai tab** |
| 8 | AC10 | Đo `.lookup-head` bằng DevTools ở cả hai tab | Cùng một số, và `--lookup-head-height` vẫn `76px` |
| 9 | AC6 | Tab + Enter/Space trên mọi nút mới; mũi tên trái/phải trên dải tab | Mọi thao tác tới được, **không chạm chuột** |
| 10 | AC12 | Ghim ở Tác phẩm A, tạo Tác phẩm B | Lịch sử rỗng, bộ ghim của B **không** có mục của A |
| 11 | AC5 | Đổi preset bố cục (`Mod+Alt+1` ↔ `Mod+Alt+2`) | Tab đang chọn **sống sót** qua lượt tháo/dựng panel |
| 12 | — | Bật `prefers-reduced-motion` ở hệ điều hành | Hàng mới hiện **tức thì**, không fade |
| 13 | AC14 | Mở app sạch (chưa tra lượt nào), chuyển thẳng sang tab Lịch sử | **Không** thấy câu "Chọn một từ trong Nguyên văn để tra cứu"; đối chứng dương: về tab Từ điển ⇒ câu đó **có** hiện |
| 14 | Dev Notes ⑧ ca 3 | Bấm phím ghim khi chưa nhắm mục nào | Câu `pin_no_target` hiện — **không** im lặng không hiệu lực |
| 15 | Bẫy 4 | Ngắt `project.db` (đổi tên file) rồi mở tab Lịch sử | Câu `pinned_load_failed` hiện — **không** đọc thành "chưa ghim mục nào" |
| 16 | QĐ #1 | Chưa mở Tác phẩm nào, mở tab Lịch sử | Câu `pinned_no_work` — **không** đọc thành "chưa ghim mục nào" |
| 17 | QĐ #6 | Đặt caret vào ô nhập tên Tác phẩm ở Library, bấm `Mod+D` | Lệnh ghim **chạy** (luật vùng gõ không áp cho hợp âm mang `Mod`); đối chứng âm: không ký tự nào lọt vào ô nhập |
| 18 | QĐ #6 · NFR14 | Chạy trên cả macOS và Windows | `Mod+D` phân giải thành `⌘D` và `Ctrl+D` — hợp âm viết `Mod+D`, không viết cứng theo nền tảng |

### Luật nghiệm thu của dự án

- **Số ĐO THẬT, không suy luận.** Mọi con số vào §Completion Notes là số đọc được từ một lượt chạy, không một ước lượng. Nếu một lượt đo ra số bất thường, **ghi thẳng** kèm ngữ cảnh — không làm tròn xuống, không kết luận trên một lượt đo *(tiền lệ: `deferred-work.md:886-891`)*.
- **Đối chứng âm bắt buộc.** Mỗi AC khẳng định một thứ xuất hiện phải kèm một phép đo rằng nó **không** xuất hiện ở điều kiện ngược lại.
- **Một câu văn xuôi giải thích một quyết định không thay được một lượt render** *(`deferred-work.md:940-943`, Ice tự tay bác một triage dựa trên đọc chú thích CSS thay vì chạy thật)*. Mọi thay đổi thị giác nghiệm thu bằng app chạy thật.

---

## Project Structure Notes

- Cây nguồn khớp Structural Seed (`ARCHITECTURE-SPINE.md:781-815`); story này không mở thư mục mới.
- **Một cạnh phụ thuộc mới:** Panel Lookup (C3) → `core/store/` để ghi mục ghim. Bảng Capability Map (`ARCHITECTURE-SPINE.md:823`) liệt kê C3 là `core/dict/`, `ports/DictionarySource`, `resources/dict/` — **không** có `core/store/`. Đây là một cạnh chưa có trong bản đồ, cùng loại với cách AD-36 phải thêm cạnh `glossary/ → dict/` (`:435`). Ghi vào `deferred-work.md` để lượt cập nhật kiến trúc sau nhặt.
- **AD-18 không phủ mục ghim.** Bảng ngữ nghĩa hai tầng (`:244-254`) không liệt kê lịch sử tra cứu hay mục ghim. Quyết định #1/#2 lấp khoảng trống này bằng một bảng SQL riêng, không một `ScopeKind` mới — nên bảng AD-18 **không** đổi và `tests/scope_contract.rs::the_semantics_table_matches_ad_18_row_by_row` **không** phải chạm.

---

## References

- `_bmad-output/planning-artifacts/epics.md:1845-1878` — Story 1.20, sáu AC nguyên văn
- `_bmad-output/planning-artifacts/prds/prd-AuraTranslate-2026-08-02/prd.md:504` — FR41 · `:1080` — giả định A9 · `:877,890` — NFR16, NFR17
- `.../architecture/architecture-AuraTranslate-2026-08-02/ARCHITECTURE-SPINE.md` — AD-1 `:75-79` · AD-7 `:119-131` · AD-11 `:153-157` · AD-21 `:302-306` · AD-30 `:362-366` · AD-34 `:406-417` · AD-35 `:425` · AD-44 ④ `:622` · Conventions `:639-652` · Stack `:664-712` · Structural Seed `:781-815` · Capability Map `:823`
- `.../ux-designs/ux-AuraTranslate-2026-08-02/mockups/lookup-history-pins.html` — mockup chính; `:103` dải tab · `:106-111` bộ lọc (bị loại, QĐ #5) · `:114-131` hàng ghim · `:133-155` hàng lịch sử · `:157-158` quy tắc ghim-vs-lịch-sử · `:171-172` luật dedupe · `:189-206` hai trạng thái rỗng
- `.../ux-designs/.../DESIGN.md:169-186` token màu · `:260-297` typography · `:341-348` motion · `:354-360` hình dạng và component "Bản ghi từ điển" · `:377` cấm màu viết thẳng
- `.../ux-designs/.../EXPERIENCE.md:312` mockup không phải nguồn sự thật · `:339` neo FR41
- `src/panels/lookupPanelState.ts:266-292` điểm cắm lịch sử · `:302-313` `resetLookupPanel` · `:154-174` doctrine hai vị từ
- `src/panels/LookupPanel.vue` cấu trúc panel, `.lookup-head` 76px, dải chip nguồn, nút Attribution
- `src/panels/SourcePanel.vue:102-137` khuôn dải tab đầy đủ
- `src/commands/index.ts:465-475` khuôn command tab · `:511-524` `lookup.lookup_selection` · `:603-625` tiền lệ "0 phím mặc định"
- `src/commands/registry.ts:31-62` API `register`, văn phạm id
- `src-tauri/src/core/store/schema.rs:52-70,98-126,144-220` migration, DDL hiện có · `:110-116` luật "đổi target làm test đỏ"
- `src-tauri/src/commands/config.rs:104-192` khuôn IPC hai lớp
- `src-tauri/src/core/i18n/mod.rs:216-297` `IpcError`, `MessageKey`
- `scripts/check-commands.mjs:211-238` các sàn · `:1648` `SELECTION_SURFACE_FLOOR`
- `_bmad-output/implementation-artifacts/deferred-work.md:656` `event.repeat` · `:672` `SELECTION_PANEL_FILES` chép tay · `:836-846` nợ "không bộ chạy test frontend" · `:886-891` luật ghi số bất thường · `:940-943` luật "chạy thật, không đọc chú thích"
- `_bmad-output/implementation-artifacts/1-18-auto-lookup.md:83-91,256` ràng buộc một điểm nghẽn để lại cho 1.20
- `_bmad-output/implementation-artifacts/1-19-bat-tat-nguon-tu-dien-va-ghi-cong.md:154-155` phân định tab thứ ba · `:206-229` tiền lệ persistence · `:607-633` Bẫy 4 và Bẫy 8

---

## Change Log

| Ngày | Việc |
|---|---|
| 2026-08-10 | Story dựng. Baseline `19ea24c`, cây làm việc sạch. |
| 2026-08-10 | **Lệch mockup #1** — `lookup-history-pins.html:103` vẽ ba tab gồm `Concordance`; đo được 0 lần trong `src/`, Concordance là FR64/Story 7.7 chưa dựng. Story này dựng **hai** tab (QĐ #4). Mockup **không** sửa. |
| 2026-08-10 | **Lệch mockup #2** — thanh bộ lọc ba chip (`:106-111`) mâu thuẫn AC4 (*"Cả Tác phẩm"* hàm ý lịch sử xuyên phiên). Loại bỏ, giữ nút xoá lịch sử (QĐ #5). |
| 2026-08-10 | **Lệch mockup #3** — bảng mã lệnh của `settings.html` dùng văn phạm chưa tồn tại; `editor.segment.nextUntranslated` vi phạm `COMMAND_ID_RE` và sẽ làm `register()` ném. Id đặt theo mã thật (QĐ #7). |
| 2026-08-10 | **Đo, không suy** — `SELECTION_SURFACE_FLOOR` **không** cần nâng: `LookupPanel.vue:95` đã đăng ký `useSelectionSurface(body, 'display')` trên `.lookup-body`, bao trọn tab mới. |
| 2026-08-10 | **Regression tìm được lúc dựng story (Bẫy 10)** — `PanelFrame.vue` render câu trạng thái phía trên `<slot />`, và `showFrameStatus = neverLookedUp`, nên tab Lịch sử ở app vừa mở bị đè câu *"Chọn một từ trong Nguyên văn để tra cứu"*. Thành **AC14**. |
| 2026-08-10 | **Ba ca mockup không vẽ** — chưa mở Tác phẩm · nạp bộ ghim trượt · ghim mà chưa nhắm mục nào. Cả ba là *"rỗng có lý do"* theo AD-44 ④; ba khoá chuỗi mới ở §Dev Notes ⑦. |
| 2026-08-10 | **Ice ký Quyết định #1** — mục ghim ở **`project.db`**, phạm vi Tác phẩm. Bước di trú mới vào `PROJECT_MIGRATIONS`, `to_version: 4`; `GLOBAL_MIGRATIONS` không đụng. Ba hệ quả ở QĐ #1. |
| 2026-08-10 | **Ice ký Quyết định #6** — `lookup.toggle_pin` mang **`Mod+D`** theo mockup, phá tiền lệ *"0 phím mặc định"* của Story 1.19 **có chủ ý** vì đây là thao tác tần suất cao nhất của story. Ba command còn lại giữ `undefined`. Đo trước khi ký: `Mod+D` trống trên toàn kho, không menu Tauri nào tranh, và `keys.ts:227,287` cho thấy luật vùng gõ **không** áp cho hợp âm mang phím bổ trợ chính. |
| 2026-08-10 | **`⌘⌫` của mockup bị bác** — cùng phép đo `keys.ts:227,287`: một hợp âm mang `Mod` không bị luật vùng gõ chặn, nên `Mod+Backspace` sẽ cướp *"xoá tới đầu dòng"* giữa lúc người dùng đang gõ, để chạy một thao tác phá hoại. `lookup.clear_history` giữ 0 phím mặc định. |
| 2026-08-10 | **Số lần tra thuộc PHIÊN, không bền vững** (§Dev Notes ⑨) — một số đếm bền vững đòi một lượt `Store::write` mỗi lần bôi đen chữ, tức đưa ghi đĩa vào đường nóng Auto-Lookup. Không AC nào đòi nó sống qua phiên. |
| 2026-08-10 | **Task 0 — cây làm việc.** `git status --porcelain` trả **2** dòng: `sprint-status.yaml` (M) và chính story file này (??). Cả hai là **tạo tác của lượt dựng story 1.20**, không phần vá cũ nào ⇒ **không cần commit riêng**, và không hỏi Ice. Baseline `19ea24c` khớp `git rev-parse HEAD`. |
| 2026-08-10 | **Task 0 — Quyết định #2 XÁC NHẬN bằng số.** `grep -n "config_value" src-tauri/src/core/store/schema.rs`: `CONFIG_VALUE_DDL` chỉ xuất hiện trong `GLOBAL_MIGRATIONS` (bước 2). `PROJECT_MIGRATIONS` **không có** bảng đó ⇒ với Quyết định #1a, câu hỏi *"một khoá `config_value` hay một bảng riêng"* **tự đóng**: khoá đó không tồn tại trong `project.db`. Một bảng `pinned_entry` mới, **0** `ScopeKind` mới. |
| 2026-08-10 | **Task 0 — Quyết định #3 XÁC NHẬN.** 0 IPC command nào cho lịch sử; `HISTORY_CEILING = 200` là hằng **duy nhất** thêm vào, và nó có tên vì AC7 chỉ chặn dòng TRÙNG chứ không chặn dòng khác nhau. Ghi vào `deferred-work.md`. |
| 2026-08-10 | **Task 0 — Quyết định #4 XÁC NHẬN bằng số.** `grep -rn "Concordance" src/ src-tauri/src/` trả **0** lần trong `src/` và đúng **2** doc-comment ở `commands/dict.rs:119,173`, cả hai nói FR64/Story 7.7. `grep "tablist" src/panels/LookupPanel.vue` trả **rỗng** ⇒ Panel Lookup có **0** tab nội bộ trước story này. Dựng **HAI** tab. |
| 2026-08-10 | **Task 0 — Quyết định #5 và #7 XÁC NHẬN**, không phản biện: `settings.html:282` dùng một văn phạm id mà `COMMAND_ID_RE` từ chối, và ba chip lọc mâu thuẫn AC4. Id đặt theo mã thật. |
| 2026-08-10 | **Task 0 — `Mod+D` đo lại trước khi gõ.** `grep -rn "Mod+D\|KeyD" src/ scripts/` trả **0** lần. `check:commands` Kiểm D sau cài đặt: bộ command THẬT dựng được keymap trên **cả** macOS lẫn Windows/Linux, không hợp âm nào giành nhau. |
| 2026-08-10 | 🔴 **PHẢN BIỆN BẰNG SỐ — Task 1 tiền đề SAI.** Story viết *"`a_fresh_database_migrates_up_to_target_and_logs_it` sẽ **đỏ** khi target đổi"*. Đo: `store_contract.rs:764-771` nói bằng chữ *"ca DUY NHẤT ở tệp này chạy trên `GLOBAL_MIGRATIONS` THẬT"*, và nó khẳng định `schema_version() == 2` của **`global.db`**. Thêm một bước vào `PROJECT_MIGRATIONS` ⇒ ca đó **KHÔNG** đỏ. **Hệ quả nặng hơn một chi tiết test:** trước story này `project.db` **không có** lưới nào đóng băng số phiên bản đích của nó — tức đúng thứ *"một hiệu ứng phụ không ai ký"* mà doc-comment của `GLOBAL_MIGRATIONS` tuyên bố là không được phép. Lỗ đó đóng ở `tests/pinned_contract.rs::a_fresh_project_database_ends_at_the_pinned_entry_step`. |
| 2026-08-10 | **§Bẫy 1 — chọn đường (b), và ghi lý do.** Dải `.lookup-sources` giữ nguyên ở **mọi** tab thay vì ẩn phần chip. Đường (a) *(ẩn chip, giữ nhãn + nút)* cũng thoả AC9 nhưng thêm một điều kiện `v-if` **quanh chính vùng chứa** của nút Attribution — tức thêm một chỗ để lượt sửa sau vô tình bọc cả nút vào. Đường (b) không có chỗ đó: nút đứng ngoài mọi nhánh tab. Dải chip ở tab Lịch sử là thừa, không hại. |
| 2026-08-10 | **Lệch mockup #4 — cột ÂM ĐỌC của hàng ghim/lịch sử không dựng.** Mockup vẽ *"Thính Triều Các"*, *"khí cơ"* cạnh mỗi đầu mục. Âm Hán Việt là một lượt tra **RIÊNG** (`read_han_viet`, Story 1.16) và `pinned_entry` không lưu nó ⇒ dựng cột đó là một vòng IPC thứ hai cho **mỗi hàng**, mỗi lượt render tab. Không AC nào đòi. Ghi vào `deferred-work.md`. |
| 2026-08-10 | **Lệch mockup #5 — `pinned_empty_note` viết lại.** Mockup ghi `⌘D khi đang xem một mục từ`; một hợp âm viết cứng theo một hệ điều hành trong `vi.json` là đúng thứ Kiểm D/NFR14 tồn tại để chặn (`Mod+D` phân giải thành `Ctrl+D` ngoài macOS). Câu mới: *"Lệnh ghim chỉ có hiệu lực khi đang xem một mục từ."* |
| 2026-08-10 | **Kiểm D CHẠM `pinned_empty_body` đúng như story lường trước.** Câu mockup chứa chữ *"bạn"*, và `check-i18n.mjs:1159` cấm nó bằng biên tiếng. Viết lại giữ nguyên nghĩa: *"…để nó **ở lại** qua các phiên và các Chương của Tác phẩm này."* Kiểm D **không** bị tắt. |
| 2026-08-10 | **`SELECTION_SURFACE_FLOOR` KHÔNG nâng — xác nhận lại sau cài đặt.** `check:commands` Kiểm F đo **6** bề mặt trên 4 panel (sàn 6): nội dung tab Lịch sử nằm trong `.lookup-scroll`, tức đã trong `useSelectionSurface(body, 'display')` đăng ký ở `LookupPanel.vue:95`. **0** lời gọi đăng ký mới. |
| 2026-08-10 | 🔴 **Ice bắt bằng mắt ở bàn đo hàng 7 (AC9): thanh kéo chia bốn panel vẽ ĐÈ lên lớp phủ Attribution.** Ba phép đo: `.dv-sash` mang `z-index: 99` (`dockview.css:2940-2942`) · `.attr-scrim` mang `z-index: 10` (`AttributionOverlay.vue:266-269`) · `.modeport` khai `flex: 1; min-height: 0`, **không** thuộc tính nào tạo ngữ cảnh xếp lớp ⇒ hai số tranh nhau trong cùng ngữ cảnh gốc và sash thắng; nó mang `--dv-sash-color: var(--color-background)` nên vẽ ra đúng hai vệt màu nền. **Vá bằng `isolation: isolate` trên `.modeport`**, KHÔNG bằng nâng `z-index` của lớp phủ: dockview đang dùng 99 · 999 · 1000 · 9999, nên một số lớn hơn là bước vào cuộc đua sẽ thua im lặng ở lần nâng thư viện kế tiếp. ⚠️ **Khuyết tật của Story 1.19, không của story này** — nó có từ ngày lớp phủ được dựng; bàn đo của 1.20 chỉ là chỗ nó bị nhìn thấy. |
| 2026-08-10 | 🔴 **Ice bảo *"tự đo test đi"* — và lượt đo bác chính ước lượng của tôi.** Tôi đoán *"cắt 17,7px"* từ số học trên hộp **nội dung** 63px. Sai: `overflow: hidden` cắt ở **mép padding**, tức 75px, không 63px. Số thật đo trên app: nội dung cần **77,44px**, cắt **~2,4px** cuối dòng nhịp thứ hai cộng trọn 12px khoảng thở. Ảnh của Ice đúng (hai dòng nhịp *trông* đủ), ước lượng của tôi sai, và khuyết tật **vẫn có thật** — chỉ nhỏ hơn tôi nói. Ghi vào `deferred-work.md` kèm ba đường vá; **không** nới `--lookup-head-height` ở story không sở hữu nó (AC10 cấm bằng chữ). |
| 2026-08-10 | **AC10 ĐẠT, đo bằng máy:** `offsetHeight` = **76** ở tab Từ điển, **76** ở tab Lịch sử, **76** sau khi quay lại; `--lookup-head-height` vẫn `76px`. Dải tab của story này **không** lấy một pixel nào của khối đó. |
| 2026-08-10 | **AC9 và AC14 ĐẠT, có đối chứng hai chiều** — xem bảng bàn đo ở §Completion Notes. AC14 là regression đắt nhất bắt được lúc dựng story, và nay nó có bằng chứng chạy được chứ không chỉ một dòng mã. |
| 2026-08-10 | **`check:scope` chạy được sau khi Ice đóng phiên `tauri dev`** — `VERDICT: PASS`, cả chiều cho phép lẫn chiều từ chối. **Chín trên chín cổng xanh.** |
| 2026-08-11 | 🔴 **ICE KÝ LẠI QUYẾT ĐỊNH #1 — ghim chuyển từ `project.db` sang `global.db`, phạm vi TOÀN ỨNG DỤNG.** Ice nêu ý muốn; một phép đo biến nó từ sở thích thành bắt buộc: `grep` toàn bộ bề mặt IPC cho **11 command, KHÔNG cái nào mở lại một `.atproj` từ đĩa** — `OpenWorkState` khởi động `None` và chỉ `create_work_*` đặt được giá trị (`commands/chapter.rs` ghi mệnh đề này bằng chữ). ⇒ đóng app rồi mở lại là **không Tác phẩm nào đang mở**, nên bộ ghim trong `project.db` **không có đường nào để đọc tới**: **AC3 đúng trên đĩa mà không bao giờ đúng trên màn hình**, cho tới Epic 5. Bảng so sánh của Quyết định #1 **không có hàng nào** cho phép đo này — đó là chỗ nó sai. |
| 2026-08-11 | **Hệ quả lược đồ, Ice ký cách xử lý.** `PINNED_ENTRY_DDL` vào `GLOBAL_MIGRATIONS` `to_version: 3`; bước 4 của `PROJECT_MIGRATIONS` **bị gỡ**, bộ đó về đúng ba bước như trước story. Đo trước khi gỡ: `~/Documents/AuraTranslate/` có **27** `.atproj` — **21 ở `user_version=3`** (không việc gì) và **6 ở `user_version=4`** (tạo tác nghiệm thu của chính lượt trước). Ice chốt xoá 6 cái đó; đã xoá sau khi kiểm lại từng tệp, còn **21**, tất cả mở được. ⚠️ Số **4** nay là một số **ĐÃ CHÁY**: bước di trú kế tiếp của `project.db` phải đánh số **5**, và doc-comment của `PROJECT_MIGRATIONS` ghi mệnh đề đó thành một vết sẹo có tên. |
| 2026-08-11 | **Ca test mà story hẹn từ đầu nay ĐỎ THẬT.** `store_contract.rs::a_fresh_database_migrates_up_to_target_and_logs_it` chạy trên `GLOBAL_MIGRATIONS`; bảng ghim nay ở đó, nên nó đỏ **đúng như cơ chế được thiết kế để đỏ**. Cập nhật target 2→3, sổ di trú 2→3 bản ghi, thêm assert `pinned_entry` có mặt. Ở lượt trước tiền đề của story sai; ở lượt này nó đúng. |
| 2026-08-11 | **Một ca khác đỏ, và nó chỉ ra một test viết quá chặt.** `scope_contract.rs::a_row_written_straight_into_global_db_resolves_back_through_the_scope_path` khẳng định `schema_version() == 2`. Ca đó quan tâm *"bước 2 đã chạy chưa"*, **không** quan tâm target — nên nó đổi thành `>= 2`. Một ca về `config_value` không có việc gì phải sửa mỗi lần một story khác thêm một bảng khác vào cùng kho. |
| 2026-08-11 | 🔴 **Lượt đổi phạm vi làm HAI chuỗi giao diện nói dối — bắt được khi rà lại `vi.json`, không phải khi chạy.** `pinned_empty_body` hứa *"…các Chương của **Tác phẩm này**"* và `history_hint` hứa *"Mục ghim sống qua các phiên và **theo Tác phẩm**"*. Cả hai **sai** với phạm vi mới, và cả chín cổng vẫn xanh với chúng — cổng đọc khoá và giọng văn, không đọc **nghĩa**. Sửa thành *"…qua các phiên và **mọi Tác phẩm**"* / *"…đi theo **mọi Tác phẩm**"*. |
| 2026-08-11 | **Bẫy 4 từ BỐN trạng thái xuống BA.** *"Chưa mở Tác phẩm nào"* biến mất cùng phạm vi Tác phẩm — câu đó **sai** ở kho toàn cục. Vị từ `pinnedNoWork` và khoá `panel.lookup.pinned_no_work` **gỡ hẳn**, không để lại làm khoá chết. `commands/chapter.rs::no_work_open` trả về `fn` riêng tư như trước. Đo lại cả ba ô cộng ô *"chưa có câu trả lời"*: vẫn tách sạch, ô cuối vẫn **im hoàn toàn**. |
| 2026-08-11 | **AC12 mất một nửa, và đó là hệ quả đúng.** Vế *"đổi Tác phẩm vứt lịch sử"* giữ nguyên. Vế *"nạp lại bộ ghim theo Tác phẩm mới"* **tự rụng** — mệnh đề đó viết có điều kiện trong AC (*"nếu ghim theo phạm vi Tác phẩm"*), và điều kiện nay sai. ⚠️ **Đối chứng âm của AC12 ĐẢO CHIỀU**: *"tab ghim của B không chứa mục của A"* nay phải đọc là **có chứa**. Ghi ra thay vì sửa AC — sửa tài liệu quy hoạch là một lượt riêng của Ice (Quyết định #3 của Story 1.3). |
| 2026-08-11 | **Ice xác nhận lịch sử giữ nguyên trong phiên (AC4).** *"Lịch sử cũng không còn"* sau khi mở lại app là **đúng thiết kế**, không một lỗi — Quyết định #3 và AC4 nói bằng chữ. Cho nó sống qua phiên là một AC mới, đi qua `correct-course`. |
| 2026-08-11 | **`check-i18n.mjs::VUE_FLOOR` KHÔNG nâng** — số thật vẫn **14**: story này thêm một tệp `.rs` và hai tệp `.ts`, **0** component `.vue` mới (dải tab và tab Lịch sử sống trong `LookupPanel.vue` đã có). Một sàn nâng khi số thật không đổi là nâng theo cảm giác. *(Dòng này trước ghi 2026-08-10 và nằm sau cả loạt dòng 08-11 — sửa ngày ở lượt code review.)* |
| 2026-08-11 | 🔴 **CODE REVIEW — hai `decision-needed`, sáu `patch`, bốn `defer`, một bị loại.** Ba lớp song song; chín cổng và `cargo test` chạy lại độc lập và **không** bác một khẳng định nào về cổng. Sáu mục đã vá: ① `toggleLookupPin` tính hướng ghim ngoài hàng đợi ⇒ hai cú bấm nhanh cho hai lệnh GHIM *(bước lùi so với tiền lệ `dictSourcesState.ts:191-194` của Story 1.19)*; ② mũi tên trên dải tab không dời tiêu điểm DOM ⇒ người dùng bàn phím kẹt ở tab thứ hai; ③ ba chú thích còn sót phạm vi `project.db` sau lượt ký lại — một cái mô tả sai chính hàm ngay dưới nó; ④ ba con số của story không khớp số đo lại; ⑤ `resetLookupHistory()` không rào lượt ghi đang bay; ⑥ hai ca Rust mới thu hẹp mối nối AC3. |
| 2026-08-11 | 🔴 **`correct-course` MỞ RỒI HUỶ — phát hiện của lượt review sai ở một phép đo không ai chạy.** Lượt review báo *"`epics.md` vẫn mang một AC12 mà mã cố ý không thoả"*; nó đọc AC12 trong story file rồi **suy ra** nguồn ở `epics.md` thay vì mở tệp ra kiểm. Đo lại: `epics.md:1845-1878` mang **đúng sáu** AC, và AC7–AC14 do chính story dẫn xuất — **không có xung đột nào để sửa**. Ứng viên thay thế (`epics.md:1873`, *"tab thứ ba"*) cũng bị **Ice bác**: tab thứ ba là **Concordance**, một năng lực **chưa triển khai** (FR64/Story 7.7), không một tab bị bỏ — AC5 mô tả trạng thái cuối và nó **đúng**. ⇒ **0 tệp quy hoạch bị chạm**, đúng Quyết định #3 của Story 1.3. Ràng buộc thứ tự tab ghi lại cho Story 7.7 ở `deferred-work.md`. ⚠️ Lượt này cũng làm hỏng chính khuôn *"đo trước khi tin"* mà story dùng ở mọi chỗ khác — một phát hiện review cũng là một khẳng định, và nó phải có phép đo đứng sau như mọi khẳng định khác. |
| 2026-08-11 | 🔴 **`ipc_contract.rs` KHÔNG khoá hình dạng dây của `PinnedEntry`** — cả hai phía chỉ khai bằng chú thích (*"Đổi một tên ở đây mà không đổi ở kia cho ra `undefined` mà TypeScript không hề biết"*). Một lượt đổi tên phía Rust đi qua trọn `cargo test` **và** trọn chín cổng, rồi vỡ đúng ở màn hình sau khi mở lại app — tức vỡ **đúng AC3**. Đóng bằng `the_pinned_wire_shape_matches_what_the_frontend_reads`, ca **đọc mã TypeScript thật** chứ không một bảng tên chép tay. Đối chứng âm đã chạy: đổi `id` → `pinId` trong `config/pinned.ts` ⇒ ca **ĐỎ** đúng dòng, khôi phục ⇒ xanh lại. |

---

## Dev Agent Record

### Agent Model Used

`claude-opus-5` (Amelia / bmad-dev-story), 2026-08-10.

### Debug Log References

- `cargo test` toàn bộ — **261 xanh, 0 đỏ, 5 ignored** (16 tệp test), đo lại 2026-08-11 sau
  lượt code review. Ca mới: `tests/pinned_contract.rs`, **10 ca xanh** *(8 của lượt dev, cộng
  **2** dựng ở lượt review để thu hẹp mối nối AC3 — xem §Review Findings)*.
- Một lượt đỏ **có thật** trên đường đi, và nó là một cổng làm đúng việc:
  `store_boundary.rs::only_core_store_may_name_rusqlite` bắt `commands/pinned.rs` gõ
  `rusqlite::params!` / `rusqlite::Result`. Vá bằng cách đi qua ba tên **tái xuất từ
  `core::store`** (`Row`, `SqlResult`, `Transaction`) và tham số dạng tuple — đúng khuôn
  `commands/project.rs` đã dùng. AD-11 không bị nới một chữ.
- `check:scope` ban đầu trượt vì cổng 1420 bận; sau khi Ice đóng phiên `tauri dev`, lượt
  chạy lại cho `VERDICT: PASS`. **Chín trên chín cổng xanh.**

**Bộ đo bàn nghiệm thu — CDP, 0 phụ thuộc mới, 0 tệp tải về.** Ice yêu cầu tôi tự đo
(2026-08-10). Extension Chrome không kết nối được và Playwright thì §KHÔNG-LÀM ⑥ cấm đưa
vào pipeline, nên đường đi là: Chrome đã có sẵn trên máy + `WebSocket` toàn cục của Node 22
+ giao thức CDP. Bốn script ở thư mục nháp của phiên: `measure-head.mjs` · `measure-tabs.mjs`
· `measure-ac.mjs` · `bench.mjs` (+ `boot-mock.js`, `bench-page.js`) · `trap4.mjs`.

Hai điều làm phép đo này **hợp lệ**, ghi ra thay vì để người đọc tự tin:
1. **Cùng một component.** Thuộc tính scope đọc được là `data-v-317d915d` — **khớp y hệt**
   ảnh DevTools Ice gửi từ app Tauri thật. Cùng CSS, cùng token, cùng cây DOM.
2. **Đường dữ liệu THẬT, không DOM chế.** Cầu IPC giả cắm vào
   `window.__TAURI_INTERNALS__.invoke` **trước** khi bundle chạy (`Page.addScriptToEvaluate
   OnNewDocument`), nên `config/pinned.ts` → `lookupHistoryState.ts` → `LookupPanel.vue`
   chạy nguyên vẹn. Lượt tra phát bằng một `Range` thật trên `.original` cộng một `mouseup`
   thật — tức đi qua **chính** hợp đồng vùng chọn của Story 1.18, không gọi tắt `runLookup`.

⚠️ **Hai giới hạn của bộ đo, không giấu:** ① font thật (`$RESOURCE/fonts/`) không nạp được
ngoài Tauri nên chữ rộng khác, làm thanh nhịp xuống dòng ở ngưỡng khác — **không** ảnh
hưởng chiều cao hộp dòng vì cả ba token khai `lineHeight` là **hệ số không đơn vị**;
② không đo được thứ cần một tiến trình khởi động lại (AC3/AC4) hay hai nền tảng (NFR14).

### Completion Notes List

**Số ĐO THẬT sau story (AC13).** Sàn cũ → sàn mới · *(số thật)*:

| Hằng | Tệp | Cũ | Mới | Số THẬT |
|---|---|---|---|---|
| `VUE_FLOOR` | `check-commands.mjs` | 11 | **12** | 14 tệp `.vue` |
| `TS_FLOOR` | `check-commands.mjs` | 23 | **24** | 30 tệp `.ts` |
| `COMMAND_FLOOR` | `check-commands.mjs` | 21 | **24** | 29 command |
| `CLICK_FLOOR` | `check-commands.mjs` | 9 | **13** | 16 `@click` |
| `DISPATCH_FLOOR` | `check-commands.mjs` | 13 | **20** | 25 `dispatch()` |
| `SELECTION_SURFACE_FLOOR` | `check-commands.mjs` | 6 | **6** *(không đổi)* | 6 bề mặt / 4 panel |
| `RS_FLOOR` | `check-i18n.mjs` | 34 | **35** | 41 tệp `.rs` |
| `VUE_FLOOR` | `check-i18n.mjs` | 12 | **12** *(không đổi)* | 14 tệp `.vue` |

Sàn mới giữ đúng tỷ lệ ~80–86% của số thật mà mọi lượt nâng trước tuân theo (`COMMAND` 83%,
`CLICK` 81%, `DISPATCH` 80%, `TS` 80%, `VUE` 86%, `RS` 85%). Hai hằng **không** đổi vì số
thật của chúng **không** đổi — nâng chúng là nâng theo cảm giác, đúng thứ AC13 cấm.

**Các số khác đo được:** `vi.json` **129** khoá *(**105** → 129, **+24**)* · **114** text node
qua `t()` / `tError()` trên 14 tệp `.vue`, 44 miễn trừ có tên · `GLOBAL_MIGRATIONS` **3** bước
*(target `global.db` 2 → 3)* · `PROJECT_MIGRATIONS` **3** bước, **về nguyên trạng** ·
`generate_handler!` **11** command IPC *(8 → 11)* · **0** phụ thuộc mới, **0** `ScopeKind`
mới, **0** `MessageKey` mới.

**Bàn đo — 9 hàng đã chạy, số thật.**

| # | AC | Đo được | Kết |
|---|---|---|---|
| 1 | AC1 | tra 5 cụm ⇒ lịch sử `[加, 尔, 佐, 书, 秘]` — **gần nhất trước** | ĐẠT |
| 2 | AC7 | tra lại cụm thứ 3 ⇒ `佐` **lên đầu**; **5 hàng trước, 5 hàng sau** (đối chứng âm: KHÔNG thành 6); mọi truy vấn duy nhất | ĐẠT |
| 3 | AC2 | `data-entry-key="cvdict:1"`; nhãn `Ghim` → `Bỏ ghim`, `aria-pressed="true"`; hàng hiện ở mục *Đã ghim* (`佐` · CVDICT · 6 lượt). Đối chứng âm: bỏ ghim ⇒ **0** hàng | ĐẠT |
| 4 | AC3 | `pins_survive_closing_and_reopening_the_store` — ghi → `close()` → mở lại → **mục ghim còn nguyên** kèm `gloss`. Ca này **không tồn tại được** ở phạm vi Tác phẩm | ĐẠT |
| 6 | AC8 | hai trạng thái rỗng **khác nhau, cùng lúc**: *"Chưa ghim mục nào"* và *"Chưa tra gì trong phiên này"*, mỗi cái ba dòng riêng | ĐẠT (một phần) |
| 7 | AC9 | `[data-attribution-open]` **hiện và có kích thước** (75,8px) ở **cả** tab Lịch sử **lẫn** tab Từ điển | ĐẠT |
| 8 | AC10 | `offsetHeight` = **76** ở cả hai tab và sau khi quay lại; `--lookup-head-height` vẫn `76px` | ĐẠT |
| 13 | AC14 | tab Từ điển ⇒ *"Chọn một từ trong Nguyên văn để tra cứu."*; tab Lịch sử ⇒ **`null`**; quay lại ⇒ câu đó **hiện lại**. Hai chiều | ĐẠT |
| 15 | Bẫy 4 | `list_pinned_entries` trả `store.read_failed` ⇒ *"Không đọc được danh sách đã ghim…"*, và **KHÔNG** *"Chưa ghim mục nào"* | ĐẠT |
| 16 | QĐ #1 | ~~*"Chưa mở Tác phẩm nào"*~~ — **hàng này CHẾT** sau lượt ký lại 2026-08-11: ghim ở `global.db` nên trạng thái đó không tồn tại. Đo lại: một lỗi mang `code = project.no_work_open` nay rơi đúng vào nhánh `pinned_load_failed` | KHÔNG CÒN ÁP DỤNG |

**Bẫy 4 — BA trạng thái tách nhau sạch, đo lại sau lượt ký lại:** nạp trượt · đã hỏi-chưa
ghim · **chưa có câu trả lời** (lượt IPC treo) ⇒ ô cuối **IM HOÀN TOÀN** ở mục ghim, không
nháy sang *"chưa ghim mục nào"*. Đây là ô mà `??` hay nuốt, và nó là ô duy nhất không có
câu nào để đọc — nên nó chỉ nghiệm thu được bằng vắng mặt. *(Ô thứ tư — "chưa mở Tác phẩm"
— biến mất cùng phạm vi Tác phẩm; xem §Change Log 2026-08-11.)*

**AC11 nghiệm thu bằng `grep`, không bằng đọc:** `grep -rn "recordLookup" src/` ngoài chính
module state trả về **đúng hai** dòng — một `import` và **một** lời gọi
(`lookupPanelState.ts:307`), nằm **sau** guard `mine !== sequence`.

**Các AC đã có bằng chứng bằng máy:** AC2 · AC3 · AC7 *(vế dedupe ở tầng lược đồ)* qua 10 ca
Rust; AC6 qua Kiểm B/D/E của `check:commands` *(29 command dựng được keymap trên **cả hai**
nền tảng, `unbound()` liệt kê đúng ba command mới không phím)*; AC13 qua bảng trên.

⚠️ **Đính chính ở lượt code review 2026-08-11:** dòng cũ ghi *"AC12 qua 7 ca Rust"* — **sai**.
Không ca Rust nào lái qua `resetLookupHistory()` hay bất kỳ mã TypeScript nào; vế còn sống của
AC12 (*"đổi Tác phẩm vứt lịch sử"*) sống trọn ở tầng frontend và **chưa** có bằng chứng bằng
máy. Cùng lượt, AC3 được thu hẹp thêm hai bậc — tên trường trên dây và thứ tự sau lượt mở lại
— nhưng khoảng còn hở của nó *(một tiến trình Tauri khởi động lại thật)* **không** đóng được
bằng Rust.

**Các AC có mặt thị giác** — AC1 · AC4 · AC5 · AC8 · AC9 · AC10 · AC14 — và chúng **CHƯA**
được nghiệm thu đầy đủ: xem Task 7.

🔴 **Story CHƯA hoàn tất.** Hai việc còn treo, cả hai cần Ice:
1. **`check:scope`** — cần đóng phiên `tauri dev` đang giữ cổng 1420 rồi chạy lại.
2. **Bàn đo chạy tay 18 hàng** (§Testing) — không bộ chạy test frontend nào thay được
   (§KHÔNG-LÀM ⑥), và §Luật nghiệm thu của chính story nói: *"một câu văn xuôi giải thích
   một quyết định không thay được một lượt render"*. Đánh dấu `review` trước lượt đó là đúng
   thứ luật ấy tồn tại để chặn.

### File List

**NEW**

- `src-tauri/src/commands/pinned.rs`
- `src-tauri/tests/pinned_contract.rs`
- `src/config/pinned.ts`
- `src/panels/lookupHistoryState.ts`

**UPDATE**

- `src-tauri/src/core/store/schema.rs` — `PINNED_ENTRY_DDL` + bước di trú `GLOBAL_MIGRATIONS` `to_version: 3`; `PROJECT_MIGRATIONS` về ba bước
- `src-tauri/src/core/store/mod.rs` — tái xuất `PINNED_ENTRY_DDL`
- `src-tauri/src/commands/mod.rs` — `pub mod pinned`
- `src-tauri/src/commands/chapter.rs` — `no_work_open` nâng `pub(crate)` rồi **trả về riêng tư** sau lượt ký lại
- `src-tauri/src/lib.rs` — ba command mới trong `generate_handler![…]`
- `src/panels/lookupPanelState.ts` — điểm ghi lịch sử trong `runLookup`; `resetLookupPanel` gọi `resetLookupHistory`
- `src/panels/LookupPanel.vue` — dải tab, nhánh nội dung tab Lịch sử, `showFrameStatus` (AC14), `@mousedown` uỷ quyền
- `src/panels/LookupRecord.vue` — `data-entry-key` + nút ghim cho mỗi đầu mục
- `src/commands/index.ts` — ba cổng mới trong `CommandDeps`, bốn command trong `registerAll()`
- `src/main.ts` — nối ba handler thật, nạp bộ ghim trước `mount()`
- `src/App.vue` — `isolation: isolate` trên `.modeport`; vá lỗi sash vẽ đè lớp phủ Attribution (khuyết tật Story 1.19, Ice bắt ở bàn đo hàng 7)
- `src/i18n/vi.json` — **24** khoá mới *(105 → 129)*
- `scripts/check-commands.mjs` — nâng năm sàn
- `scripts/check-i18n.mjs` — nâng `RS_FLOOR`
- `src-tauri/tests/store_contract.rs` — target `global.db` 2 → 3, sổ di trú 3 bản ghi, assert bảng mới
- `src-tauri/tests/scope_contract.rs` — `== 2` nới thành `>= 2` (ca đó không sở hữu target)
- `_bmad-output/implementation-artifacts/deferred-work.md` — 10 mục nợ mới
- `_bmad-output/implementation-artifacts/sprint-status.yaml` — trạng thái story
