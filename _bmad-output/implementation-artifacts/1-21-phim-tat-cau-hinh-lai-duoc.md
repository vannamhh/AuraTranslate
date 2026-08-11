---
baseline_commit: fe952de6ce4ac87b91ab1adef0b431b9f4920536
---

# Story 1.21: Phím tắt cấu hình lại được

Status: review

**Covers:** FR22 (`prd.md:419`) · NFR17 (`prd.md:887`) · AD-34 (`ARCHITECTURE-SPINE.md:406-417`)
**Epic:** 1 — Nền móng ứng dụng & Tra cứu ngoại tuyến tức thì · **story CUỐI của epic**
**Nguồn:** `epics.md:1881-1914` · mockup `settings.html:235-298` · AD-18 · AD-21 · AD-24 · AD-34 · AD-44 ④
**Nợ đóng ở đây:** `deferred-work.md:241` *(mã hoá hợp âm trên đĩa là TẠM)* · `:243` *(xung đột chỉ được sống sót, chưa được giải quyết)*
**Nợ ĐI QUA đây mà KHÔNG đóng:** `:149` · `:184` · `:485` · `:491` · `:656` · `:1135` — xem Quyết định #7

---

## 🔴 Điều kiện khởi hành — ĐỌC TRƯỚC KHI GÕ MỘT DÒNG

**Cây làm việc lúc dựng story này (2026-08-11) KHÔNG SẠCH.** `git status --porcelain` trả **23 dòng**: toàn bộ phần cài đặt của **Story 1.20 chưa được commit**, và Story 1.20 đang ở `in-progress` với Task 7 còn **9 trên 18 hàng bàn đo chạy tay** chưa chạy.

Sáu trong số tệp bẩn đó là **đúng những tệp story này phải sửa**: `src/commands/index.ts` · `src/main.ts` · `src/App.vue` · `src/i18n/vi.json` · `scripts/check-commands.mjs` · `scripts/check-i18n.mjs`.

⇒ **Không bắt đầu Task 1 trước khi 1.20 được commit.** Bắt đầu sớm là trộn hai diff vào nhau, và Ice đã chốt bằng chữ rằng diff của một story phải đọc được một mình. Trình tự bắt buộc:

1. Story 1.20 đóng Task 7 (hoặc Ice ký chấp nhận phần còn treo) → commit riêng.
2. Điền `baseline_commit` của story này bằng SHA vừa commit.
3. Xác nhận `git status --porcelain` trả **0 dòng**, rồi mới gõ dòng mã đầu tiên.

*(Không có trạng thái `blocked` trong lược đồ `sprint-status.yaml`; mệnh đề chặn sống ở đây — cùng tiền lệ Story 1.16 bị 1.10c chặn.)*

---

## Story

As a người dịch,
I want đổi mọi phím tắt theo thói quen của mình,
So that công cụ chạy theo tay tôi chứ không ngược lại.

---

## Acceptance Criteria

Nguyên văn từ `epics.md:1889-1914`, đánh số để tham chiếu:

**AC1** — **Given** danh sách command đã đăng ký · **When** mở màn hình phím tắt · **Then** **mọi** command hiện ra kèm phím đang gán

**AC2** — **Given** một command · **When** người dùng gán phím khác · **Then** thay đổi có hiệu lực **ngay** và lưu ở tầng Global

**AC3** — **Given** hai command được gán cùng một phím · **When** xảy ra · **Then** xung đột hiện ra cho người dùng giải quyết · **And** không im lặng ghi đè

**AC4** — **Given** phím tắt đã đổi · **When** mở lại ứng dụng · **Then** giữ nguyên

**AC5** — **Given** `CommandRegistry` · **When** truy vấn từ màn hình phím tắt · **Then** liệt kê được các command **chưa gán phím nào**

**AC6** — **Given** một vòng thao tác trong phạm vi epic này — mở Tác phẩm, chuyển panel, bôi đen tra cứu, bật tắt nguồn, ghim một mục, chuyển chế độ · **When** thực hiện · **Then** làm được **hoàn toàn bằng bàn phím, không chạm chuột một lần nào**

### AC bổ sung — dẫn xuất từ mockup, kiến trúc và đo đạc mã nguồn

Sáu AC trên không nói hết thứ phải đúng để tính năng chạy được trong hệ thống đang có. Tám AC dưới đây **cùng hạng ràng buộc**, mỗi cái neo vào một nguồn kiểm chứng được:

**AC7 — `unbound()` trên bộ command THẬT phải còn phần tử.** `scripts/check-commands.mjs:1398-1404` **đỏ** khi `commandRegistry.unbound()` trả mảng rỗng: *"AC6 chưa được chứng minh trên bộ command thật"*. ⇒ story này **không** được "sửa nốt NFR17" bằng cách gán hợp âm mặc định cho mọi command. Màn hình cho **người dùng** gán; bộ mặc định của sản phẩm giữ nguyên số command chưa gán. Nghiệm thu: chạy `check:commands` sau story, dòng `unbound()` vẫn liệt kê tên thật.

**AC8 — *"bỏ gán"* và *"trả về mặc định"* là HAI trạng thái khác nhau, và cả hai phải tới được từ màn hình.** `src/main.ts:85-91` đã khoá hợp đồng này bằng chữ: giá trị rỗng trên đĩa nghĩa là *"thao tác này cố ý không có phím"*, còn *"chưa ai đặt gì"* là **khoá vắng mặt**; `chordsFor` (`index.ts:309-315`) dùng `?? fallback` chứ không `|| fallback` chính vì thế. Một màn hình chỉ có "bỏ gán" mà không có "trả về mặc định" là một cửa **một chiều**: người dùng gỡ phím của `mode.library` rồi không còn đường nào lấy lại `Mod+1`. Nghiệm thu: bỏ gán → mở lại app → vẫn không phím; trả về mặc định → mở lại app → `Mod+1` trở lại.

**AC9 — xung đột so trên hợp âm ĐÃ PHÂN GIẢI của nền tảng đang chạy, không trên chuỗi hợp âm.** `keys.ts:261-264` dựng `resolved` (`'Meta+KeyD'`), và `claimed` (`:256`, `:267`) khoá theo **`resolved`**. Trên macOS, `Mod+D` và `Meta+D` là **cùng một phím** nhưng là **hai chuỗi khác nhau**. Một phép so trên chuỗi hợp âm để lọt đúng ca đó và cho ra hai command giành một phím mà màn hình nói *"không xung đột"*. Nghiệm thu: gán `Meta+D` cho một command trong khi `lookup.toggle_pin` đang giữ `Mod+D`, trên macOS ⇒ màn hình **phải** báo xung đột.

**AC10 — trong lúc BẮT hợp âm, không command toàn cục nào được chạy.** `attachKeymap` nghe ở pha `capture` trên `window` (`keys.ts:366`). Không có cửa chặn thì gõ `Mod+1` để gán phím sẽ **đổi chế độ** giữa lúc gán. Cửa đã có sẵn: `KeymapGate.isBlocked` (`keys.ts:323-334`), hôm nay nối vào `attributionIsOpen` ở `main.ts:250`. Nghiệm thu: mở màn hình, bấm bắt hợp âm, gõ `Mod+1` ⇒ ô phím nhận `Mod+1`, chế độ **không** đổi.

**AC11 — một phím không biểu diễn được bị TỪ CHỐI kèm lý do, không nuốt im lặng.** `keys.ts:125-137` ném với một tên phím ngoài `NAMED_CODES`, và thông điệp của nó nói đúng việc phải làm. Màn hình bắt được `F1`, `NumpadEnter`, `IntlBackslash`… — không cái nào có trong bảng. Rơi im lặng ở đây là *"bấm mà không có gì xảy ra"*, đúng lớp lỗi AD-44 ④ cấm (`ARCHITECTURE-SPINE.md:622`). Nghiệm thu: bắt `F1` ⇒ một câu có lý do hiện ra, ô phím giữ nguyên giá trị cũ.

**AC12 — sau một lượt gán, *"phím đang gán"* và *"chưa gán phím nào"* đọc từ MỘT nguồn lúc chạy.** `CommandSpec.keys` bị `frozen()` đóng băng lúc `register()` (`registry.ts:72-78`), nên `spec.keys` và `registry.unbound()` mãi mãi trả lời **thời điểm cài đặt**. Sau lượt gán đầu tiên chúng **cũ**. Màn hình phải đọc hợp âm đang có hiệu lực từ `keymap.bindings()` (`keys.ts:303-311` — doc-comment của chính nó ghi *"Story 1.21 dựng màn hình gán phím trên bề mặt này"*), và *"chưa gán"* = `list()` trừ đi các id có trong `bindings()`. Nghiệm thu: gán một phím mới, **không** đóng màn hình, đối chiếu lại danh sách ⇒ hàng đó rời khỏi nhóm "chưa gán" ngay.

**AC13 — hợp âm trên đĩa bị từ chối phải NÓI RA trên màn hình.** `bindingsAreUsable` (`index.ts:766-785`) đã bắt ca *"một `global.db` sửa tay làm `createKeymap` ném"* và **rơi về hợp âm mặc định** — nhưng chẩn đoán chỉ đi ra `console.error`, tức im lặng theo nghĩa thực dụng. `deferred-work.md:243` ghi nguyên văn: *"người dùng chỉ biết nếu họ mở console, và lựa chọn của họ im lặng không được áp. **Màn giải quyết xung đột là Story 1.21**"*. AC3 cấm giải xung đột im lặng, nên ca này thuộc AC3. Nghiệm thu: sửa tay `global.db` cho hai command cùng một hợp âm → mở app → màn hình phím tắt hiện một câu nói **lựa chọn đang không được áp** và vì sao.

**AC14 — mọi sàn `*_FLOOR` bị vượt được nâng theo SỐ THẬT.** `VUE_FLOOR` · `TS_FLOOR` · `COMMAND_FLOOR` · `CLICK_FLOOR` · `DISPATCH_FLOOR` · `SELECTION_SURFACE_FLOOR` trong `scripts/check-commands.mjs`, và `VUE_FLOOR`/`RS_FLOOR` trong `scripts/check-i18n.mjs` nếu chạm. Số thật đo được ghi vào §Completion Notes, không ước.

---

## Task 0 — BẢY QUYẾT ĐỊNH, chốt TRƯỚC dòng mã đầu tiên

Khuôn cố định của mọi story lớn trong dự án (1.17 §161, 1.18 §142, 1.19 §201, 1.20 §62). Mỗi quyết định có **đề xuất mặc định kèm lý lẽ đo được**. Dev đọc, xác nhận hoặc phản biện **bằng số** — không im lặng thi hành, và không tự đổi sau khi đã gõ mã.

### Quyết định #1 — Cơ chế "có hiệu lực NGAY" (AC2): keymap dựng lại được bằng cách nào?

Ba sự thật đo được chặn hết các đường hiển nhiên:

| Sự thật | Nguồn | Hệ quả |
|---|---|---|
| `register()` **ném** với id trùng | `registry.ts:109-114` | Không thể đăng ký lại bộ command lên registry đang sống |
| `CommandSpec` bị **đóng băng** lúc đăng ký | `registry.ts:72-78` | Không thể sửa `spec.keys` tại chỗ |
| `attachKeymap` **ném** ở lần gắn thứ hai vào cùng target | `keys.ts:353-358` | Không thể gắn một keymap thứ hai lên `window` |

⇒ Đường duy nhất là **dựng một `Keymap` mới rồi thay nó vào chỗ listener đang đọc**.

**(a) Thêm tham số `overrides` tuỳ chọn cho `createKeymap` — ĐỀ XUẤT.**
`createKeymap(registry, platform, overrides?)`: hợp âm hiệu lực của mỗi spec = `overrides?.[spec.id] ?? spec.keys`. `applyBindings()` ở `index.ts` dựng keymap mới rồi gán vào biến module `keymap`.
- Tương thích ngược đo được: **7** lời gọi `createKeymap` trong `check-commands.mjs` (`:1088` `:1089` `:1160` `:1182` `:1217` `:1235` `:1298`) đều **hai tham số** ⇒ tham số thứ ba tuỳ chọn không làm Kiểm D đỏ.
- **Một** registry sống, một đường `dispatch`.

**(b) Registry nháp + một `Keymap` proxy ổn định.**
Dùng lại `registerAll(scratch, deps, next)` (`index.ts:321`) rồi `createKeymap(scratch, …)`; `attachKeyboard` truyền một object `{ handle, bindings }` uỷ quyền sang biến `keymap` hiện tại.
- Ưu: **0 dòng** sửa trong `keys.ts` và `registry.ts`.
- Nhược: hai registry cùng sống, và cái **dispatch** thật lại là cái nháp. Một người đọc mã sau này sẽ mất nửa giờ để tin rằng điều đó đúng.

**Cả hai đường đều cần:** `installCommands` **giữ lại `deps`** trong một biến module (hôm nay nó không giữ — `index.ts:794-807`), vì lượt dựng lại cần đúng bộ handler đó.

**Ràng buộc chung, không thương lượng:** một lượt dựng lại **trượt** (xung đột) **không được** để lại keymap hỏng. Dựng xong mới thay — biên dịch vào một biến tạm, `try/catch`, chỉ gán khi thành công.

### Quyết định #2 — Bề mặt: một LỚP PHỦ, không một chế độ thứ tư

`AD-24` khai **ba** chế độ ngang hàng và `MODE_IDS` (`index.ts:37`) là một hằng ba phần tử; `Mod+4` là phím của Story 8.11. Story 1.19 đã giải đúng bài này cho Attribution và ghi lý lẽ vào `AttributionOverlay.vue:5-10`.

**ĐỀ XUẤT: `src/ShortcutsOverlay.vue`, cạnh `AttributionOverlay.vue`, dựng ở `App.vue` cùng tầng lớp phủ Attribution (`App.vue:202-203`).** Chép khuôn: `role="dialog"` + `aria-modal="true"` · `tabindex="-1"` để nhận `focus()` · `@keydown.esc` **DOM thường** (không command — `AttributionOverlay.vue:151-159` viết sẵn lý do) · `trapTab` · trả tiêu điểm về chỗ cũ theo UX-DR17 · nút đóng **đi qua command**.

**Đường vào:** một nút ở `header.titlebar` của `App.vue:146-173` — chỗ duy nhất luôn hiện ở cả ba chế độ. Nút mang một thuộc tính `data-` để `returnFocusTo` có đường lui (`AttributionOverlay.vue:82-88`).

⚠️ **Đây là màn *Cài đặt › Phím tắt*, không phải màn Cài đặt.** Chín mục còn lại của `settings.html:251-262` thuộc Epic 4/5/6/10. Dựng một khung điều hướng trái cho chín mục chưa tồn tại là trỏ tới năng lực chưa có — thứ §KHÔNG-LÀM của Story 1.17 đã cấm và Story 1.20 Quyết định #4 vừa áp lại.

### Quyết định #3 — Lưu ở đâu: **0 dòng Rust, 0 bước di trú, 0 `MessageKey`**

Đường đã có sẵn từ Story 1.8 và chưa ai đi:

| Mảnh | Đã có | Nguồn |
|---|---|---|
| `ScopeKind::Shortcut` = `"shortcut"`, ngữ nghĩa `GlobalOnly` | ✅ | `kinds.rs:200-204` |
| Đường ĐỌC: `bootstrap_config.shortcuts: BTreeMap<String,String>` | ✅ | `commands/config.rs:62-63`, `scope/store.rs:171-174` |
| Đường GHI: `save_value` nhận mọi `kind` **GlobalOnly** | ✅ | `scope/store.rs:286-304` |
| Adapter webview: `putConfig(kind, key, value)` | ✅ | `config/bootstrap.ts:203-219` |
| Test vòng ghi–đọc cho `shortcut` | ✅ | `tests/scope_contract.rs:567,595,612` |

⇒ **Ghi bằng `putConfig('shortcut', <command_id>, <chuỗi hợp âm>)`.** Không `#[tauri::command]` mới, không `PINNED_ENTRY_DDL` thứ hai, không bước di trú.

**Và nó ĐÓNG `deferred-work.md:241` bằng một phép đo, không bằng một mô hình mới.** Mục nợ đó lo rằng mã hoá *"ngăn nhau bằng dấu phẩy, không escape"* sẽ vỡ với **một hợp âm chứa dấu phẩy**. Đo trên `keys.ts:112`: phím dấu phẩy viết là **`Comma`**, một tên chữ cái — hợp âm của nó là `'Mod+Comma'`, **không** `'Mod+,'`. `keyToCode` (`:125-137`) chỉ nhận `[0-9]`, `[A-Za-z]` và các khoá của `NAMED_CODES`, và **không khoá nào chứa `,`**. ⇒ không hợp âm hợp lệ nào chứa dấu phẩy, và mã hoá hiện tại **an toàn theo cấu trúc**.
🔴 Biến phép đo đó thành **cơ chế**: một khẳng định ở Kiểm D rằng không khoá nào của `NAMED_CODES` chứa `','`. Ba dòng, và nó là khác biệt giữa *"đúng do tình cờ"* và *"đúng có lưới"*.

### Quyết định #4 — State sống ở đâu

`src/commands/*.ts` **bị cấm** cho state này: Kiểm C/D/E nạp `registry.ts` · `focus.ts` · `keys.ts` · `index.ts` bằng **Node thuần** (`check-commands.mjs:799,962,1040,1257`), và bốn tệp đó chỉ `import` lẫn nhau. Một module dùng `ref` của Vue hoặc `@tauri-apps/api` mà bị `index.ts` `import` là **ba phép kiểm hành vi chết cùng lúc**.

**ĐỀ XUẤT: `src/config/shortcutsState.ts`** — cùng thư mục `bootstrap.ts`, vì đây **là** cấu hình ứng dụng và nó gọi thẳng `putConfig`. Nó chứa: bản đồ hợp âm từ đĩa, trạng thái mở/đóng lớp phủ, trạng thái đang-bắt-hợp-âm, hàng đang nhắm, và câu lỗi gần nhất.

⚠️ **Đừng chép ca `dictSourcesState.ts`** — nó sống ở `src/panels/` trong khi giữ state của một lớp phủ **toàn ứng dụng** (`attributionOpen`, `:47`). Đó là một vết, không một tiền lệ.

Handler đi vào `installCommands` bằng **tiêm** qua `CommandDeps`, nối ở `src/main.ts` — cùng cửa `selectLookupTab`/`toggleDictSource` đã đi qua.

### Quyết định #5 — Bộ command mới: **năm**, và tất cả TĨNH

Kiểm A (`check-commands.mjs:649`) đòi **mọi** `@click` là **đúng một** `dispatch('<id>')` với id **literal**. Một bảng 29+ hàng, mỗi hàng ba nút, **không** dựng được bằng một command cho mỗi hàng — đó là §KHÔNG-LÀM ⑤ của Story 1.19/1.20 viết thành chữ ký (`index.ts:238-246`, `:261-269`).

⇒ Cùng khuôn `toggleDictSource`: **id tĩnh, handler đọc hàng đang nhắm tại thời điểm chạy.**

| id | Nhãn (`command.<id>`) | Cổng `CommandDeps` | Hợp âm mặc định |
|---|---|---|---|
| `shortcuts.open` | Mở màn hình phím tắt | `openShortcuts` | **`Mod+Comma`** |
| `shortcuts.close` | Đóng màn hình phím tắt | `closeShortcuts` | `undefined` |
| `shortcuts.capture` | Bắt hợp âm mới cho thao tác đang nhắm | `captureShortcut` | `undefined` |
| `shortcuts.unassign` | Bỏ gán phím của thao tác đang nhắm | `unassignShortcut` | `undefined` |
| `shortcuts.reset` | Trả phím của thao tác đang nhắm về mặc định | `resetShortcut` | `undefined` |

**Ba phép đo đứng sau `Mod+Comma`:**
1. **Trống** — hợp âm đang chiếm: `Mod+1/2/3` · `Mod+Alt+1/2` · `Mod+Alt+O/J/V/L/S` · `Mod+Alt+←/→` · `Shift+Arrow…` · `Alt+Shift+Arrow…` · `Mod+D`. `Comma` chưa ai dùng.
2. **`Comma` có trong `NAMED_CODES`** (`keys.ts:112`) ⇒ `parseChord` phân giải được, không phải thêm tên phím mới.
3. **Mockup gọi đúng phím đó** — `settings.html:134,240` vẽ `⌘,` ở thanh tiêu đề của màn Cài đặt. `⌘,` là quy ước Preferences của macOS; app không khai menu Tauri nên webview nhận được nó.

⚠️ **Bốn command còn lại 0 phím mặc định**, theo tiền lệ 1.19/1.20: họ `Mod+Alt+…` đã kín chỗ có nghĩa, và cả bốn tới được bằng Tab + Enter/Space bên trong lớp phủ. Chúng cũng là **nhiên liệu cho AC7** — `unbound()` giữ được phần tử thật.

### Quyết định #6 — Cử chỉ gán phím, và cái bẫy `⌫`

`settings.html:294` nguyên văn: *"Bấm vào ô phím rồi gõ tổ hợp mới · `⌫` để bỏ gán"*.

**ĐỀ XUẤT — bốn trạng thái của một hàng, mỗi trạng thái một câu:**
1. **nghỉ** — hiện hợp âm đang có hiệu lực, hoặc câu *"chưa gán"*;
2. **đang bắt** — `Enter`/`Space` trên ô phím (hoặc `shortcuts.capture`) vào trạng thái này; ô nói ra rằng nó đang chờ;
3. **đã bắt, có xung đột** — hàng và hàng đối thủ cùng hiện màu, kèm tên thao tác đang giữ phím (`settings.html:286-288`: *"Lệnh gán sau **bị chặn** chứ không cướp phím của lệnh gán trước"*);
4. **đã bắt, sạch** — áp ngay (AC2) và ghi đĩa ngay.

🔴 **`⌫` KHÔNG được là phím "bỏ gán" trong lúc đang bắt.** `Backspace` có trong `NAMED_CODES` (`keys.ts:102`), tức `Backspace` trần và `Mod+Backspace` đều là **hợp âm hợp lệ gán được**. Nếu trạng thái ③ đọc `⌫` thành "bỏ gán" thì không ai gán được phím đó nữa. ⇒ `⌫` bỏ gán ở trạng thái **①** (ô đang có tiêu điểm, chưa bắt); trong trạng thái ② nó là một hợp âm như mọi hợp âm khác. `Escape` ở trạng thái ② là **huỷ lượt bắt**, và nó phải huỷ lượt bắt **trước** khi tới được `@keydown.esc` đóng lớp phủ.

⚠️ Ô phím là một **`<button>`**, không một `<input>`. Một `<input>` là *vùng gõ* theo `isTypingZone` (`keys.ts:246-251`), và điều đó đổi hành vi của luật vùng gõ ngay giữa lúc ta đang cố bắt phím trần.

### Quyết định #7 — BA món nợ mang tên "Story 1.21" mà story này **KHÔNG** nhận

`deferred-work.md` gán ba mục cho story này. Cả ba **không có một AC nào** ở `epics.md:1889-1914`. Đề xuất: **trả lại, kèm chủ mới**, và Ice ký.

| Nợ | Nguồn | Vì sao KHÔNG nhận | Chủ đề xuất |
|---|---|---|---|
| Cờ `repeatable` trên `CommandSpec` (giữ phím không lặp thao tác) | `:656` | Chạm `registry.ts` + `keys.ts` + **mọi** command đang có, để phục vụ đúng 4 command `selection.extend_*`. 0 AC. Một thay đổi diện rộng không AC là đúng thứ mọi story của dự án này từ chối | Story đầu tiên có một AC đòi giữ phím (ứng viên: Epic 2 — điều hướng segment) |
| Màn quản lý **preset bố cục do người dùng đặt tên** | `:491` | `epics.md:1579-1581` giao FR17/FR18 cho **Story 1.14**; `epics.md:1883` giao story này **đúng FR22**. Dựng một bề mặt cho `ScopeKind::LayoutPreset` ở đây là thêm năng lực không AC | Chưa gán — nêu ở retrospective Epic 1 |
| Token thứ tự giữa nạp và ghi bộ ghim | `:1135-1145` | Điều kiện kích hoạt là *"story đầu tiên thêm một lượt `loadPinnedEntries()` thứ hai"*. Story này **không** thêm lượt nào | Giữ nguyên điều kiện, chủ vẫn treo |

🔴 Còn `:149` · `:184` · `:485` — ba lỗ NFR17 *(vào panel bằng bàn phím · `PanelFrame` không `enterFocus` · bốn `layout.toggle_*` không phím)* — story này **đóng chúng theo nghĩa của FR22**: từ nay người dùng **gán được** phím cho cả bốn `layout.toggle_*`. Nó **không** đóng chúng theo nghĩa *"bộ mặc định của sản phẩm có phím"* — và không được đóng, vì AC7. Ghi lại mệnh đề đó vào `deferred-work.md` bằng chữ, đừng gạch mục.

---

## Tasks / Subtasks

- [x] **Task 0 — chốt bảy quyết định** *(AC: mọi AC)*
  - [x] Xác nhận Story 1.20 đã commit (`fe952de`); `git status --porcelain` trả **1 dòng** và dòng đó là chính story file 1.21 *(tệp chưa theo dõi, 0 tệp mã bẩn)*; `baseline_commit` đã điền
  - [x] Đọc bảy quyết định; **phản biện #1 bằng số** *(hai lỗ, hai vá)* và Ice ký; sáu quyết định còn lại xác nhận bằng phép đo tại chỗ — xem §Debug Log References
  - [x] Ghi lại số thật hiện tại của mọi sàn `*_FLOOR` sẽ chạm (bảng ở §Debug Log References)
  - [x] Ice ký Quyết định #7 — **lật một phần**: NHẬN `:656` (cờ `repeatable`), TRẢ `:491` và `:1135`. Dev Notes ⑦ ⇒ Ice chọn **(a)** `delete_config`

- [x] **Task 1 — tầng bàn phím: hai hàm thuần mới, cùng bảng phím** *(AC2, AC9, AC11)*
  - [x] `chordFromEvent(event, platform): string` trong `keys.ts` — nghịch đảo của `parseChord`. Bỏ qua keydown chỉ có phím bổ trợ; `meta && isMac` **hoặc** `ctrl && !isMac` ⇒ `Mod`; thứ tự chuẩn `Mod` → `Alt` → `Shift` → phím *(khớp cách các hợp âm đang có được viết: `Mod+Alt+ArrowRight`, `Alt+Shift+ArrowLeft`)*
  - [x] `formatChord(chord, platform): string` — chuỗi HIỂN THỊ (`⌘⌥⇧⌃` trên macOS, `Ctrl+Alt+Shift+` nơi khác). **Hàm thuần, ở `keys.ts`**, vì `NAMED_CODES` không được export và không nên export
  - [x] Cả hai **không** `import` gì — `keys.ts` phải nạp được bằng Node thuần (Kiểm D)
  - [x] Phím ngoài bảng ⇒ trả `null` (không ném): chỗ gọi là một cử chỉ người dùng, không một lỗi lập trình. Ném chỉ đúng ở `parseChord`, nơi đầu vào là **dữ liệu đã lưu**
  - [x] `createKeymap` nhận tham số thứ ba `overrides` **tuỳ chọn** *(nếu Quyết định #1a)* — giữ nguyên hành vi khi vắng mặt

- [x] **Task 2 — dựng lại keymap lúc chạy** *(AC2, AC3, AC12, AC13)*
  - [x] `installCommands` giữ `deps` và `isMac` vào biến module
  - [x] `applyBindings(next)` trong `index.ts`: dựng keymap mới trên một bản nháp → **chỉ khi thành công** mới thay vào biến `keymap`; trượt ⇒ trả về mô tả xung đột có cấu trúc, **không** ném và **không** đụng keymap đang sống
  - [x] `conflictFor(chord, id, isMac)`: phân giải hợp âm rồi tra trong `keymap.bindings()` — so trên **`resolved`** (AC9), không trên chuỗi hợp âm
  - [x] `effectiveBindings()` / `effectiveUnbound()` export ra cho màn hình — **một** nguồn lúc chạy (AC12)
  - [x] Thêm doc-comment vào `registry.ts::unbound()` và `CommandSpec.keys`: chúng trả lời **thời điểm cài đặt**; nguồn lúc chạy là `keymap.bindings()`
  - [x] `bindingsAreUsable` (`index.ts:766`) ghi lý do trượt vào một chỗ **màn hình đọc được**, không chỉ `console.error` (AC13)

- [x] **Task 3 — state + đường ghi đĩa** *(AC2, AC4, AC8)*
  - [x] `src/config/shortcutsState.ts`: bản đồ hợp âm từ đĩa, hàng đang nhắm, trạng thái bắt, câu lỗi gần nhất
  - [x] Nạp từ `config.shortcuts` **đã `await`** ở `main.ts:142` — không thêm một vòng IPC thứ hai
  - [x] Ghi bằng `putConfig('shortcut', id, chords.join(','))`; **bỏ gán** ⇒ ghi chuỗi **rỗng**; **trả về mặc định** ⇒ **xoá khoá**. Xoá khoá chưa có đường IPC — xem §Dev Notes ⑦ trước khi tự chế
  - [x] `putConfig` trả `IpcError | null` và **không ném**: một lượt lưu trượt phải hiện ra (AD-21), không chỉ vào log
  - [x] **KHÔNG** `import` module này vào `src/commands/index.ts` — tiêm qua `CommandDeps`, nối ở `src/main.ts`

- [x] **Task 4 — lớp phủ** *(AC1, AC3, AC5, AC10, AC11, AC12)*
  - [x] `src/ShortcutsOverlay.vue` theo khuôn `AttributionOverlay.vue`: `role="dialog"` · `aria-modal` · `tabindex="-1"` · `trapTab` · trả tiêu điểm (UX-DR17) · `@keydown.esc` DOM thường
  - [x] Bảng: **mọi** command của `commandRegistry.list()` (AC1), nhãn qua `t(spec.labelKey)`, hợp âm qua `formatChord`, nhóm *"chưa gán phím"* đọc từ `effectiveUnbound()` (AC5, AC12)
  - [x] Nhắm hàng: `@mousedown` uỷ quyền ở vùng chứa **cộng** `document.activeElement.closest('[data-command-id]')`, **aimed trước, focused sau** — `dictSourcesState.ts:324-341` ghi lý do bằng một ca đo được trên WKWebView
  - [x] Mỗi `@click` là **đúng một** `dispatch('<id>')` với id literal (Kiểm A)
  - [x] Đăng ký `useSelectionSurface(panel, 'display')` — bảng này chứa chữ thật; vai `'display'`, **không** `'source'` (Bẫy 1 của Story 1.18)
  - [x] Mọi text node qua `t()`; hợp âm và mã lệnh dùng `<!-- aura-allow-text: … -->` kèm lý do
  - [x] Màu chỉ từ token — `check:tokens` xanh

- [x] **Task 5 — command, cửa nuốt hợp âm, i18n** *(AC6, AC10)*
  - [x] Năm command trong `registerAll()`, id theo Quyết định #5
  - [x] Năm cổng mới trong `CommandDeps`, nối handler thật ở `src/main.ts`
  - [x] `main.ts:250` — cửa nuốt hợp âm nay hỏi **hai** vị từ: `attributionIsOpen || shortcutsCaptureIsArmed`. ⚠️ **Chỉ nuốt khi ĐANG BẮT**, không nuốt suốt thời gian lớp phủ mở: mở màn hình rồi bấm `Mod+Alt+←` để đi lại giữa panel vẫn phải chạy — nó không phá gì cả
  - [x] Khoá mới trong `src/i18n/vi.json` — object phẳng, khoá chấm, giọng **vô nhân xưng** (Kiểm D của `check-i18n.mjs:1152`). Chuỗi chính xác ở §Dev Notes ⑧
  - [x] Nút mở ở `header.titlebar` của `App.vue`, mang `data-shortcuts-open`

- [x] **Task 6 — nâng sàn và chạy đủ chín cổng** *(AC7, AC14)*
  - [x] `npm run check:deps && check:tokens && check:i18n && check:commands && check:layout && check:scope && check:scope:bundled && check:dict && check:dict-manifest`
  - [x] `npm run build` (hai lượt `vue-tsc --noEmit` + `vite build`)
  - [x] `cd src-tauri && cargo test` — **kỳ vọng 261 xanh, 0 đỏ, 5 ignored, không đổi**. Story này không sửa Rust; một con số khác nghĩa là có gì đó ngoài dự tính
  - [x] Kiểm D: thêm khẳng định `NAMED_CODES` không chứa `','` (Quyết định #3) và vòng khứ hồi `parseChord(chordFromEvent(e, isMac), isMac)` khớp `e` trên **cả hai** nền tảng (NFR14)
  - [x] Đọc dòng `unbound()` trong log của `check:commands` và **ghi tên** các command còn chưa gán vào §Completion Notes (AC7)
  - [x] Nâng **mọi** sàn bị vượt theo số thật

- [ ] **Task 7 — nghiệm thu chạy tay, có đối chứng âm** *(mọi AC)*
  - [x] **8 trên 20 hàng ĐÃ CHẠY và ĐẠT**, đo **bằng máy** trên đường sản phẩm thật — 24 phép đo, tất cả đạt; bộ đo và log đầy đủ ở §Debug Log References. Hàng **2** (AC1) · **3** (AC5) · **4**+**5** (AC2 + AC12, có đối chứng `registry.unbound()` vẫn nói cũ) · **7** (AC3) · **8** (AC9, có đối chứng âm) · **13** (AC8, hai chiều) · **20** (Bẫy 9)
  - [ ] **12 hàng CÒN LẠI cần app Tauri thật** — không đo bằng máy được, và ghi rõ vì sao: hàng **1**/**19** (thị giác) · **6** (đóng rồi mở lại **tiến trình**) · **9**/**18** (cửa nuốt hợp âm — đòi một listener `window` sống) · **10** (AC11) · **11** (Bẫy 4) · **12** (Bẫy 5) · **14** (AC13, sửa tay `global.db`) · **15**/**16** (AC6 trên **cả** macOS lẫn Windows, NFR14) · **17** (UX-DR17). Cộng **ảnh chụp màn hình thật** cho mỗi AC thị giác
  - [x] Ghi mọi lệch mockup và mọi món nợ mới vào `deferred-work.md` — **6 mục mới** + **6 mục cũ** cập nhật *(3 đóng · 1 đóng một nửa · 2 đổi chủ)*

---

## Dev Notes

### ① Bốn tệp phải nạp được bằng NODE THUẦN — đọc trước khi thêm một dòng `import`

`check-commands.mjs` `import()` thẳng bốn tệp: `registry.ts` (`:799`) · `focus.ts` (`:962`) · `keys.ts` (`:1040`) · `index.ts` (`:1257`). Node ≥ 22.18 chỉ **bóc kiểu**; nó không hiểu `.vue`, không phân giải `vi.json` theo luật Vite, không có `@tauri-apps/api`.

⇒ Trong bốn tệp đó: **không** `import` giá trị ngoài ba module cùng thư mục; **không** `enum`, **không** `namespace`, **không** parameter property. Ba thứ sau **sinh mã** chứ không chỉ mang chú thích, và Node từ chối bóc chúng.

Một dòng `import { ref } from 'vue'` ở `index.ts` là **Kiểm C, D và E chết cùng lúc** — tức ba AC của Story 1.6 quay về nghiệm thu bằng mắt.

### ② `chordFromEvent` — nghịch đảo của `parseChord`, và bốn ca dễ sai

`parseChord` (`keys.ts:146-192`) đi từ `'Mod+Shift+Enter'` → `{ mods, code }`. Ta cần chiều ngược lại, và bốn ca dưới đây là nơi bản đầu sẽ sai:

1. **Keydown chỉ có phím bổ trợ.** Nhấn `⌘` phát một `keydown` với `code === 'MetaLeft'`. Đó **không** phải một hợp âm — bỏ qua và **tiếp tục chờ**, đừng chốt.
2. **`Mod` phụ thuộc nền tảng.** `metaKey && isMac` ⇒ `Mod`; `ctrlKey && !isMac` ⇒ `Mod`. Nhưng `ctrlKey` **trên** macOS ⇒ `Ctrl` viết tường minh, và `metaKey` ngoài macOS ⇒ `Meta`. Cả hai đều biểu diễn được (`parseChord:172-177`) và cả hai đều có nghĩa thật.
3. **`code` → tên phím.** `Digit7` → `'7'`; `KeyD` → `'D'`; còn lại phải là **một khoá của `NAMED_CODES`** (`:97-123`). `F1`, `NumpadEnter`, `IntlBackslash`, `CapsLock` **không** có ⇒ trả `null`, và màn hình nói ra (AC11).
4. **Thứ tự phần tử.** `parseChord` chấp nhận mọi thứ tự, nhưng chuỗi lưu xuống đĩa và chuỗi hiện trên màn hình phải **ổn định**, nếu không hai lần gán cùng một phím cho ra hai chuỗi khác nhau và mọi phép so bằng chuỗi đều lệch. Chuẩn: `Mod` → `Meta` → `Ctrl` → `Alt` → `Shift` → phím.

⚠️ **Đừng đọc `event.key`.** `keys.ts:85-95` viết sẵn lý do: trên AZERTY và trên mọi bố cục không phải US, `event.key` của phím vật lý `1` là `'&'`. Toàn bộ tầng này khớp bằng **`event.code`**.

### ③ Xung đột — so trên `resolved`, và bảng đối thủ đã có sẵn

`createKeymap` dựng `resolved` ở `keys.ts:261-264`:

```ts
const resolved =
  [mods.meta && 'Meta', mods.ctrl && 'Ctrl', mods.alt && 'Alt', mods.shift && 'Shift']
    .filter((p): p is string => typeof p === 'string')
    .join('+') + (hasNoMods(mods) ? code : `+${code}`)
```

và `claimed: Map<string, CommandId>` (`:256`) khoá theo chính chuỗi đó. `Keymap.bindings()` (`:309`) trả về `{ chord, resolved, id }` cho **mọi** hợp âm đang sống.

⇒ Phép kiểm xung đột là **một lượt tra trên `resolved`**, không một vòng lặp mới:

```
resolved(hợp âm vừa bắt) === resolved(một binding đang có) && id khác  ⇒  XUNG ĐỘT
```

Trên macOS, `Mod+D` và `Meta+D` cùng cho `Meta+KeyD` ⇒ xung đột thật, dù hai chuỗi hợp âm khác nhau (AC9). Ngoài macOS chúng khác nhau ⇒ **không** xung đột. Cùng một mã, hai câu trả lời đúng — đó là lý do phép so phải đi qua `parseChord` chứ không so chuỗi.

### ④ *"Có hiệu lực NGAY"* — vì sao không có đường tắt

Ba chốt chặn đường sửa tại chỗ, và cả ba là **hành vi đúng** chứ không phải chỗ cần nới:

```ts
// registry.ts:109 — đăng ký trùng id
if (byId.has(id)) throw new Error(…)
// registry.ts:72 — spec bị đóng băng, `keys` cũng bị đóng băng
const frozen = (spec) => Object.freeze({ …, keys: Object.freeze([...(spec.keys ?? [])]) })
// keys.ts:353 — gắn keymap lần thứ hai vào cùng target
if (attached.has(target)) throw new Error(…)
```

`registry.ts:64-70` nói thẳng rằng chốt thứ hai tồn tại **vì story này**: *"Story 1.21 dựng màn hình gán phím trên `unbound()`; một lượt 'sửa tại chỗ' ở đó sẽ đổi kho mà không đi qua `register()`, tức trốn hết ba phép cưỡng chế."*

⇒ Đường đúng: **dựng một `Keymap` mới, thay vào biến `keymap` mà listener đang đọc.** Listener gắn ở `keys.ts:360-366` gọi `keymap.handle(event)` trên **đối tượng đã truyền vào** — nên hoặc `createKeymap` nhận `overrides` (Quyết định #1a), hoặc `attachKeyboard` truyền một proxy ổn định (#1b). Không có đường thứ ba mà không nới một phép cưỡng chế đang đúng.

### ⑤ Sau story này, `spec.keys` là một câu trả lời CŨ

Đây là hệ quả kiến trúc quan trọng nhất của story, và nó không tự hiện ra trong mã:

| Câu hỏi | Trước 1.21 | Sau 1.21 |
|---|---|---|
| Hợp âm của command X? | `spec.keys` — đúng | `keymap.bindings()` — `spec.keys` là giá trị **lúc cài đặt** |
| Command nào chưa gán phím? | `registry.unbound()` — đúng | `list()` trừ `bindings()` — `unbound()` là câu trả lời **lúc cài đặt** |

Cả hai vẫn **đúng cho mục đích của chúng**: `check-commands.mjs:1399` đọc `unbound()` để chứng minh AC6 của Story 1.6 trên **bộ mặc định của sản phẩm**, và bộ đó không đổi lúc chạy. Nhưng **màn hình** thì không được đọc chúng.

⇒ Viết mệnh đề này thành **doc-comment ngay trên `unbound()` và `CommandSpec.keys`**. Một hàm đúng-trong-một-nghĩa mà không nói ra nghĩa nào là cách story sau đọc nhầm nó.

### ⑥ Cửa nuốt hợp âm — nuốt lúc BẮT, không nuốt lúc MỞ

`main.ts:250` hôm nay:

```ts
void attachKeyboard(window, { isBlocked: () => attributionIsOpen.value })
```

`KeymapGate` (`keys.ts:323-334`) nuốt bằng **thoát sớm**, **không** `preventDefault()` — đó là điều kiện để `@keydown.esc` của lớp phủ vẫn nhận được sự kiện. Giữ nguyên cơ chế đó.

🔴 **Vị từ mới hỏi *đang bắt hợp âm*, không hỏi *lớp phủ đang mở*.** Attribution chặn suốt thời gian mở vì nó là `aria-modal` và một lượt đổi preset bố cục phía sau nó gọi `api.clear()` — có hậu quả thật. Màn phím tắt thì khác: người dùng đang **đọc bảng phím** và có mọi lý do để thử `Mod+Alt+←`. Chỉ trạng thái **đang chờ một hợp âm** mới cần độc quyền bàn phím.

⚠️ Nhưng lớp phủ vẫn khai `aria-modal="true"` và vẫn `trapTab` — hai thứ đó nói về **tiêu điểm**, không về hợp âm. Đừng gộp.

### ⑦ *"Trả về mặc định"* cần XOÁ một khoá, và hôm nay chưa có đường

`chordsFor` (`index.ts:309-315`) phân biệt ba trạng thái bằng **sự có mặt của khoá**:

| Trên đĩa | `bindings[id]` | Kết quả |
|---|---|---|
| khoá vắng mặt | `undefined` | dùng hợp âm **mặc định** của `registerAll` |
| khoá có, giá trị `""` | `[]` | **cố ý không có phím** |
| khoá có, giá trị `"Mod+K"` | `['Mod+K']` | hợp âm của người dùng |

`putConfig` chỉ **ghi** (`scope/store.rs:311-321` là một `INSERT … ON CONFLICT DO UPDATE`). **Không có đường xoá một hàng `config_value`.**

⇒ *"Trả về mặc định"* (AC8) cần một trong ba, và Task 0 phải chọn:
- **(a)** một `#[tauri::command]` `delete_config(kind, key)` — sạch nhất, nhưng phá mệnh đề *"0 dòng Rust"* của Quyết định #3;
- **(b)** ghi chính hợp âm mặc định của command đó vào đĩa — **SAI**, và sai theo cách sẽ nổ về sau: hàng đó thành một giá trị **đóng băng**, nên khi một story sau đổi hợp âm mặc định, người dùng đã bấm "trả về mặc định" một lần sẽ mắc kẹt ở giá trị cũ **mãi mãi**, không dấu hiệu nào;
- **(c)** không dựng "trả về mặc định", chỉ có "bỏ gán" — **vi phạm AC8** (cửa một chiều).

**ĐỀ XUẤT: (a).** Nó là ~25 dòng Rust theo đúng khuôn hai lớp của `commands/config.rs:165-192`, đi qua `From<StoreError>` sẵn có, **0 `MessageKey` mới**, và nó là năng lực mà mọi màn cấu hình sau này (Epic 4 §Cấu hình AI) đều sẽ cần.

### ⑧ Chuỗi giao diện — khoá và ràng buộc

Object **phẳng**, khoá chấm có tiền tố miền (`check-i18n.mjs:1010`). Giọng **vô nhân xưng** — Kiểm D (`:1152`) từ chối *"bạn"*/*"chúng tôi"*. Nội suy `{ten_tham_so}`, tên khớp `[a-z_][a-z0-9_]*` (`resolve.ts:41-48`).

Năm nhãn command (`command.<id>` — bắt buộc, Kiểm E của `check-commands.mjs:1246` đối chiếu từng `labelKey` với `vi.json`):

```
"command.shortcuts.open":     "Mở màn hình phím tắt",
"command.shortcuts.close":    "Đóng màn hình phím tắt",
"command.shortcuts.capture":  "Bắt hợp âm mới cho thao tác đang nhắm",
"command.shortcuts.unassign": "Bỏ gán phím của thao tác đang nhắm",
"command.shortcuts.reset":    "Trả phím của thao tác đang nhắm về mặc định",
```

Chuỗi màn hình — mỗi câu **nói ra lý do**, không một nhãn trần (AD-44 ④, UX-DR27):

```
"shortcuts.title":         "Phím tắt",
"shortcuts.intro":         "Mọi thao tác của ứng dụng đi qua một sổ đăng ký duy nhất, nên câu hỏi \"thao tác nào chưa gán được phím\" là câu hỏi liệt kê được, không phải câu hỏi kiểm bằng mắt.",
"shortcuts.scope_note":    "Phím tắt chỉ tồn tại ở tầng Toàn cục — một thao tác không nên đổi phím theo từng Tác phẩm.",
"shortcuts.col_command":   "Thao tác",
"shortcuts.col_id":        "Mã lệnh",
"shortcuts.col_key":       "Phím",
"shortcuts.col_note":      "Ghi chú",
"shortcuts.unassigned":    "chưa gán",
"shortcuts.capturing":     "Đang chờ một tổ hợp phím — Escape để huỷ.",
"shortcuts.conflict":      "Trùng với \"{other}\" — một trong hai phải đổi, và thao tác gán sau đang bị chặn.",
"shortcuts.key_unknown":   "Phím này chưa nằm trong bảng phím của ứng dụng nên chưa gán được — phím đang gán giữ nguyên.",
"shortcuts.save_failed":   "Chưa lưu được lựa chọn phím — phím vẫn đang có hiệu lực trong phiên này, nhưng lần mở sau sẽ trở về giá trị cũ.",
"shortcuts.disk_rejected": "Bộ phím tắt đã lưu có hai thao tác giành cùng một phím nên chưa được áp — ứng dụng đang chạy bằng phím mặc định. Lựa chọn cũ chưa bị xoá.",
```

⚠️ `shortcuts.scope_note` là **nguyên văn** `settings.html:246`. Nó **không** phải một chú thích: `kinds.rs:29-37` trích đúng câu đó làm lý do khai `Shortcut` là `GlobalOnly`. Câu này ở màn hình là chỗ người dùng đọc được quyết định kiến trúc đó — xem Bẫy 1.

⚠️ **Hợp âm và mã lệnh KHÔNG đi qua `vi.json`.** `⌘D` và `lookup.toggle_pin` là **dữ liệu**, không một câu giao diện — cùng hạng `src.display_name` ở `AttributionOverlay.vue:215`. Dùng `<!-- aura-allow-text: … -->` kèm lý do (Kiểm A2, `check-i18n.mjs:892`).

### ⑨ Nhắm hàng — `@mousedown` TRƯỚC `activeElement`, và thứ tự đó là một ca đo được

Story 1.19 đã trả giá cho bài này. `dictSourcesState.ts:290-341`:

> *"**Vì sao không chỉ đọc `document.activeElement`:** WebKit (WKWebView — engine Tauri trên macOS) không đặt tiêu điểm cho `<button>` khi bấm chuột […]"*
> *"**`aimedCode` ĐỨNG TRƯỚC `focusedChipCode()`, và thứ tự đó là cả điểm** […] Tab tới chip A (giờ `activeElement` là A), rồi **bấm chuột** chip B. `mousedown` đặt `aimedCode = 'B'` đúng, nhưng `activeElement` vẫn là A"*

⇒ Chép đúng khuôn: `aimedId ?? focusedRowId()`, với `focusedRowId()` đọc `document.activeElement.closest('[data-command-id]')`. Cả hai đường phải chạy — đường chuột **và** đường bàn phím (AC6).

Không xác định được hàng ⇒ **không im lặng**: một câu có lý do, cùng doctrine `panel.lookup.pin_no_target` của Story 1.20.

### ⑩ Kiểm A chỉ canh `@click` — và đó là thứ định hình cả bảng

`check-commands.mjs:33` nguyên văn: *"**Chỉ `@click`.** `@keydown`, `@input`, `@change`, `@submit` KHÔNG thuộc luật Kiểm A"*, và luật đó là: mỗi `@click` là **đúng một** `dispatch('<id>')` với id **literal**.

⇒ Hệ quả cứng cho bảng 29+ hàng:
- `@click` trên nút của hàng ⇒ `dispatch('shortcuts.capture')` — **một** id tĩnh cho mọi hàng;
- hàng nào đang được nhắm thì đi bằng `@mousedown` (⑨), **không** bằng tham số của `dispatch`;
- `@keydown` xử lý `Enter`/`Space`/`Escape`/`Backspace` tự do — nó ngoài luật Kiểm A.

Một `@click="dispatch('shortcuts.capture', spec.id)"` là **đỏ ở cổng**, và một `@click="captureFor(spec.id)"` cũng vậy.

### ⑪ Cây nguồn — file sẽ chạm

| File | NEW/UPDATE | Việc |
|---|---|---|
| `src/commands/keys.ts` | UPDATE | `chordFromEvent` · `formatChord` · tham số `overrides` của `createKeymap` |
| `src/commands/registry.ts` | UPDATE | **chỉ doc-comment** trên `unbound()` và `CommandSpec.keys` (§Dev Notes ⑤) |
| `src/commands/index.ts` | UPDATE | năm command · năm cổng `CommandDeps` · `applyBindings` · `conflictFor` · `effectiveBindings`/`effectiveUnbound` · giữ `deps` |
| `src/config/shortcutsState.ts` | NEW | state + đường ghi đĩa |
| `src/ShortcutsOverlay.vue` | NEW | lớp phủ, khuôn `AttributionOverlay.vue` |
| `src/App.vue` | UPDATE | nút mở ở `titlebar` + dựng lớp phủ |
| `src/main.ts` | UPDATE | nối năm handler · cửa nuốt hợp âm hai vị từ · nạp bản đồ hợp âm từ `config.shortcuts` |
| `src/i18n/vi.json` | UPDATE | ~18 khoá mới |
| `src-tauri/src/commands/config.rs` | UPDATE | `delete_config` **nếu** Quyết định ⑦(a) |
| `src-tauri/src/core/scope/store.rs` | UPDATE | `delete_value` **nếu** Quyết định ⑦(a) |
| `src-tauri/src/lib.rs` | UPDATE | đăng ký command mới **nếu** ⑦(a) |
| `src-tauri/tests/scope_contract.rs` | UPDATE | ca xoá khoá **nếu** ⑦(a) |
| `scripts/check-commands.mjs` | UPDATE | nâng sàn (AC14) · hai khẳng định mới ở Kiểm D |
| `scripts/check-i18n.mjs` | UPDATE | nâng sàn **nếu** chạm |

Cây nguồn đã chốt: `ARCHITECTURE-SPINE.md:781-815`. Đặt tên: Rust `snake_case`; Vue component `PascalCase.vue`; khoá `vi.json` phẳng theo khoá chấm.

### ⑫ Bảng Stack — phiên bản ghim (`ARCHITECTURE-SPINE.md:664-689`)

Rust edition 2024 · `tauri` 2.11.5 · `@tauri-apps/api` 2.11.1 · `@tauri-apps/cli` 2.11.4 · `rusqlite` 0.40.1 · `serde` 1.0.229 · Vue 3.5.40 · TypeScript 5.9.3 · Vite 8.2.0 · `dockview-vue` 7.0.4 · `vue-tsc` 3.3.9.

**0 phụ thuộc mới.** Mọi thứ cần đã có: `KeyboardEvent.code` là API nền tảng, `CommandRegistry` là của Story 1.6, đường ghi cấu hình là của Story 1.8.

🔴 **`tauri-plugin-global-shortcut` bị BÁC, và lý do đã ghi ở `keys.ts:20-24`** — ba lý do, mỗi lý do đủ để loại: (1) một phụ thuộc mới phải rà GPLv3 và vào bảng Stack trước (NFR15); (2) nó đăng ký phím ở tầng **hệ điều hành**, tức cướp phím khỏi mọi ứng dụng khác khi AuraTranslate chạy nền; (3) *"Global Hotkeys"* của FR22 nghĩa là **toàn ứng dụng**, không phải toàn hệ điều hành. Nếu thấy cần một thư viện bắt phím, đó là dấu hiệu đi sai đường — dừng và hỏi.

`tauri-plugin-sql`, `tauri-plugin-fs`, `tauri-plugin-dialog` **bị cấm tường minh** (`ARCHITECTURE-SPINE.md:710`); `check-deps` chạy `cargo tree -i` cho từng tên.

**Không cần tra cứu web cho story này** — không API ngoài, không thư viện mới, không bề mặt mạng.

### ⑬ Ranh giới — §KHÔNG-LÀM

1. **Không** dựng màn Cài đặt đầy đủ. Chín mục còn lại của `settings.html:251-262` thuộc Epic 4/5/6/10 — trỏ tới năng lực chưa tồn tại là thứ §KHÔNG-LÀM của 1.17 cấm.
2. **Không** thanh chuyển phạm vi Toàn cục/Tác phẩm. Xem Bẫy 1 — `kinds.rs:36` gọi đích danh story này trong lời cấm.
3. **Không** Xuất/Nhập bộ phím tắt (`settings.html:291-292`). 0 AC, và một định dạng trao đổi là một hợp đồng phải bảo trì.
4. **Không** ô tìm kiếm / tra ngược hợp âm (`settings.html:269`). 0 AC.
5. **Không** cờ `repeatable`, **không** màn preset bố cục — Quyết định #7.
6. **Không** gán hợp âm mặc định cho các command đang chưa gán. AC7 nói vì sao, và cổng sẽ đỏ.
7. **Không** đụng `core/store/{writer,reader,checkpoint,pragmas}.rs` và **không** thêm bước di trú. Nếu Quyết định ⑦(a) được chọn thì chỉ thêm một câu `DELETE` trong `core/scope/store.rs`.
8. **Không** thêm `vitest`/Playwright — quyết định giữ nguyên từ Story 1.5 qua mười một story; nghiệm thu DOM bằng bàn đo chạy tay.
9. **Không** sửa `epics.md`, `prd.md`, hay bất kỳ mockup nào. Lệch thì **ghi ra** (Quyết định #3 của Story 1.3).

### References

Mọi mệnh đề kỹ thuật ở trên neo vào một trong các nguồn sau. Danh sách này để tra ngược, không thay cho trích dẫn tại chỗ.

- **Yêu cầu:** `epics.md:1881-1914` *(story + 6 AC)* · `epics.md:1579-1581` *(Story 1.14 giữ FR17/FR18)* · `prd.md:419` *(FR22)* · `prd.md:767` *(FR103)* · `prd.md:887` *(NFR17)*
- **Kiến trúc:** `ARCHITECTURE-SPINE.md:406-417` *(AD-34)* · `:238` *(AD-18)* · `:302` *(AD-21)* · `:622` *(AD-44 ④)* · `:664-689` *(bảng Stack)* · `:710` *(plugin bị cấm)* · `:781-815` *(cây nguồn)*
- **UX:** `mockups/settings.html:235-298` *(màn phím tắt)* · `:246` *(một tầng)* · `:272-284` *(bảng)* · `:286-288` *(luật xung đột)* · `:294` *(cử chỉ gán)* · `EXPERIENCE.md:340` *(bản đồ màn hình)* · `EXPERIENCE.md:511` *(UX-DR9)*
- **Tầng bàn phím:** `src/commands/keys.ts:5-29` *(vì sao `Mod`)* · `:20-24` *(bác `tauri-plugin-global-shortcut`)* · `:85-95` *(khớp bằng `code`)* · `:97-123` *(`NAMED_CODES`)* · `:125-137` *(`keyToCode`)* · `:146-192` *(`parseChord`)* · `:246-251` *(`isTypingZone`)* · `:253-312` *(`createKeymap`)* · `:323-334` *(`KeymapGate`)* · `:344-371` *(`attachKeymap`)*
- **Sổ đăng ký:** `src/commands/registry.ts:48-51` *(`list`/`unbound`)* · `:64-78` *(`frozen`)* · `:109-114` *(id trùng ném)* · `:115-120` *(`labelKey` bắt buộc)*
- **Chỗ đăng ký:** `src/commands/index.ts:302-315` *(`chordsFor`)* · `:321-746` *(`registerAll`)* · `:378-400` *(lỗ NFR17 có chủ)* · `:766-785` *(`bindingsAreUsable`)* · `:794-822` *(`installCommands`/`attachKeyboard`)*
- **Cấu hình:** `src/config/bootstrap.ts:203-219` *(`putConfig`)* · `src/main.ts:82-104` *(`toBindings`)* · `:142` *(lượt `await` duy nhất)* · `:250` *(cửa nuốt hợp âm)* · `src-tauri/src/core/scope/kinds.rs:29-37,200-204` · `src-tauri/src/core/scope/store.rs:171-174,286-326` · `src-tauri/src/commands/config.rs:56-93,145-192`
- **Tiền lệ lớp phủ:** `src/AttributionOverlay.vue` *(toàn tệp)* · `src/App.vue:146-173,202-203` · `src/panels/dictSourcesState.ts:290-341` *(nhắm mục tiêu)*
- **Cổng:** `scripts/check-commands.mjs:33` *(phạm vi Kiểm A)* · `:211-238` *(sàn)* · `:649` *(Kiểm A)* · `:1029` *(Kiểm D, hai nền tảng)* · `:1246` *(Kiểm E)* · `:1398-1404` *(`unbound()` phải khác rỗng)* · `:1607-1648` *(Kiểm F, `SELECTION_SURFACE_FLOOR`)* · `scripts/check-i18n.mjs:276-281` *(sàn)* · `:831` *(Kiểm A)* · `:892` *(Kiểm A2)* · `:1010` *(Kiểm B)* · `:1152` *(Kiểm D giọng văn)*
- **Nợ:** `deferred-work.md:149` · `:184` · `:241` · `:243` · `:485` · `:491` · `:656` · `:836-846` · `:1135-1145`
- **Test Rust đã có:** `src-tauri/tests/scope_contract.rs:567,595,612` *(vòng ghi–đọc một hàng `shortcut`)* · `src-tauri/tests/ipc_contract.rs:162,194` *(hình dạng dây của `shortcuts`)*

---

## Bẫy đã biết

### Bẫy 1 — thanh chuyển phạm vi của mockup là một cái bẫy CÓ TÊN *(nghiêm trọng nhất)*

`settings.html:243-248` vẽ màn phím tắt **có** thanh phạm vi hai nút — `Toàn cục` (đang chọn) · `Tác phẩm`. Phản xạ tự nhiên là dựng nó cho giống mockup.

**`kinds.rs:29-37` cấm bằng chữ, và nó gọi đích danh story này:**

> *"Khai chúng là `Override` là **sai im lặng**: nó mở một tầng Tác phẩm mà UX đã cấm, và **Story 1.14/1.21 sẽ dựng thanh chuyển phạm vi cho một thứ không nên có**."*

`Shortcut` là `Semantics::GlobalOnly`, và `save_value` (`scope/store.rs:294-304`) **từ chối** mọi loại không phải `GlobalOnly` với `store.write_failed`. Một nút *"Tác phẩm"* bấm được sẽ ghi trượt, hoặc tệ hơn, không ghi gì và trông như đã ghi.

⇒ **Không dựng thanh phạm vi.** Thay nó bằng đúng một câu — `shortcuts.scope_note`, nguyên văn `settings.html:246` — nói ra rằng phím tắt chỉ có một tầng. Ghi lệch mockup này vào Change Log.

### Bẫy 2 — `unbound()` rỗng làm cổng đỏ, và nó rỗng RẤT dễ

`check-commands.mjs:1398-1404`:

```js
const unboundIds = indexMod.commandRegistry.unbound().map((s) => s.id)
if (unboundIds.length === 0) {
  eFail('`unbound()` trả MẢNG RỖNG — AC6 chưa được chứng minh trên bộ command thật')
}
```

Story này là story **đầu tiên** có động cơ thật để "dọn nốt" các command chưa gán phím — bốn `layout.toggle_*`, hai `library.import_*`, ba của 1.19, ba của 1.20. Gán phím cho tất cả **là cổng đỏ**, và lý lẽ đã ghi ở `index.ts:378-399` và `deferred-work.md:485`: *"Một lỗ có tên tốt hơn một bằng chứng bị xoá."*

⇒ Bộ **mặc định** không đổi. Màn hình là để **người dùng** gán.

### Bẫy 3 — bắt hợp âm mà không chặn cửa ⇒ đang gán thì đổi chế độ

`attachKeymap` nghe pha `capture` trên `window` (`keys.ts:366`) và `preventDefault()` **trước** khi dispatch (`:290`). Gõ `Mod+1` trong lúc đang bắt, không có cửa chặn ⇒ ứng dụng nhảy sang Library, lớp phủ vẫn mở, ô phím không nhận gì.

Cửa đã có: `KeymapGate.isBlocked`. **Nuốt bằng thoát sớm, KHÔNG `preventDefault()`** (`keys.ts:329-333`) — nếu không, `@keydown.esc` của lớp phủ không tới được handler và người dùng bị nhốt trong đúng thứ vừa mở.

### Bẫy 4 — `Escape` phải huỷ lượt BẮT trước khi đóng lớp phủ

Lớp phủ có `@keydown.esc` đóng chính nó (khuôn `AttributionOverlay.vue:167`). Trong trạng thái *đang bắt*, `Escape` phải huỷ lượt bắt và **dừng ở đó** — không đóng lớp phủ. Hai nghĩa của một phím trong hai trạng thái là bình thường; **quên tách chúng** thì người dùng huỷ một lượt gán và mất luôn màn hình.

### Bẫy 5 — `⌫` vừa là "bỏ gán" vừa là một hợp âm gán được

`Backspace` có trong `NAMED_CODES` (`keys.ts:102`). `settings.html:294` dạy `⌫` = bỏ gán. Nếu trạng thái *đang bắt* đọc `⌫` thành "bỏ gán" thì **không ai gán được `Backspace`** nữa, và không dấu hiệu nào nói vì sao.

⇒ `⌫` bỏ gán ở trạng thái **nghỉ**; trong trạng thái *đang bắt* nó là một hợp âm như mọi hợp âm khác. Xem Quyết định #6.

### Bẫy 6 — ô phím là `<button>`, không `<input>`

`isTypingZone` (`keys.ts:246-251`) đọc `tagName === 'INPUT' | 'TEXTAREA' | 'SELECT'` hoặc `isContentEditable`. Một `<input>` làm ô bắt phím trở thành **vùng gõ**, và luật vùng gõ (`:287`) sẽ bỏ qua mọi hợp âm không mang phím bổ trợ chính — tức chính lúc ta muốn bắt `B` trần thì tầng dưới cư xử khác. Dùng `<button>` và `@keydown` trên nó.

### Bẫy 7 — chuỗi Rust phải viết KHÔNG DẤU *(chỉ áp nếu Quyết định ⑦(a))*

Kiểm A của `check-i18n.mjs:831` quét `.rs` bằng máy trạng thái: chuỗi/`char` literal ở **vị trí mã** không được mang dấu tiếng Việt; doc-comment thì được. Miễn trừ chỉ ba chỗ có tên: `src-tauri/tests/**`, `src/selftest/**`, `tools/**`. `src-tauri/src/**` **không** trong danh sách. Viết `khong`, `xoa khoa`, `nothing was deleted`.

### Bẫy 8 — chú thích `.vue` nhắc tên thẻ `style`/`template` kèm ngoặc nhọn

`check-i18n.mjs` cắt tệp `.vue` thành ba khối bằng chính hai thẻ đó. Nhắc tên chúng trong một chú thích làm cổng cắt nhầm và **mọi** miễn trừ `aura-allow-text` phía dưới mất hiệu lực cùng lúc (bắt lúc chạy cổng, 2026-08-07). Cùng bẫy đã ghi trong `LookupPanel.vue`.

### Bẫy 9 — dựng lại keymap TRƯỢT mà vẫn thay vào chỗ đang sống

Một lượt gán gây xung đột làm `createKeymap` **ném** (`keys.ts:269-273`). Nếu mã thay biến `keymap` **trước** khi biết lượt dựng có thành công không, hoặc bắt lỗi rồi để `keymap` thành `null`, thì **toàn bộ bàn phím ứng dụng chết** sau một lượt gán sai — và người dùng mất luôn đường bàn phím để sửa nó.

⇒ Dựng vào một biến tạm, `try/catch`, **chỉ gán khi thành công**. Cùng doctrine `bindingsAreUsable` (`index.ts:766-785`) đã dựng cho lượt khởi động: *"Ở đây chỉ có 'đừng chết'."*

### Bẫy 10 — `installCommands` không giữ `deps`, và lượt dựng lại cần đúng bộ đó

`installCommands(deps)` (`index.ts:794-807`) dùng `deps` rồi **bỏ**. Một lượt dựng lại keymap cần chạy `registerAll` (hoặc đọc lại `spec.keys`) với **đúng** bộ handler đã tiêm — nếu không, các cổng `applyPreset`/`togglePanel`/`toggleLookupPin`… vắng mặt và mọi command mới dựng chỉ `console.error` *"cổng chưa được tiêm"*. Triệu chứng: gán một phím xong thì **mọi** phím ngừng làm gì, mà không lỗi nào ném.

### Bẫy 11 — nút mở lớp phủ nằm ở `titlebar`, và tiêu điểm phải quay về nó

`AttributionOverlay.vue:82-88` dùng `document.querySelector('[data-attribution-open]')` làm đường lui cho tiêu điểm, và `:72-80` giải thích vì sao phải kiểm `isConnected` trước khi `focus()`: *"Một node đã rời DOM vẫn nhận được lời gọi `focus()` mà **không** ném và **không** có tác dụng — tiêu điểm rơi về `body`, đúng thứ UX-DR17 cấm."* Chép cả hai vế; đừng chép mỗi vế đầu.

---

## Testing

**Không có bộ chạy test frontend** — quyết định giữ nguyên từ Story 1.5 qua mười một story (§KHÔNG-LÀM ⑧). Vế DOM nghiệm thu bằng **bàn đo chạy tay**; món nợ này nối dài `deferred-work.md:836-846`.

### Cổng bằng máy — chạy đủ, không chọn lọc

```
npm run check:deps && npm run check:tokens && npm run check:i18n && \
npm run check:commands && npm run check:layout && npm run check:scope && \
npm run check:scope:bundled && npm run check:dict && npm run check:dict-manifest
npm run build
cd src-tauri && cargo test
```

### Hai khẳng định mới ở Kiểm D — **đây là lưới NFR14 duy nhất của story này**

`check-commands.mjs:1029` là chỗ duy nhất trong dự án lái **cả hai** nền tảng bằng một object giả. Story này thêm hai hàm phụ thuộc nền tảng, nên chúng phải vào đó:

1. **Vòng khứ hồi:** với một tập sự kiện giả *(`{code:'KeyD', metaKey:true}`, `{code:'Digit1', ctrlKey:true, altKey:true}`, `{code:'Comma', metaKey:true}`, `{code:'ArrowLeft', shiftKey:true, altKey:true}`)*, `parseChord(chordFromEvent(e, p), p)` cho **đúng** `{mods, code}` của `e`, ở **cả** `isMac: true` lẫn `false`.
2. **Mã hoá trên đĩa an toàn:** không khoá nào của `NAMED_CODES` chứa `','` — ba dòng, và nó đóng `deferred-work.md:241` bằng cơ chế thay vì bằng văn xuôi.

Cộng: `chordFromEvent` trả `null` cho `{code:'F1'}` và cho `{code:'MetaLeft', metaKey:true}` (AC11 + ca "chỉ phím bổ trợ").

### Test Rust *(chỉ nếu Quyết định ⑦(a))*

Quy ước: integration test ở `src-tauri/tests/`, hai họ `*_contract.rs` / `*_boundary.rs`; tên hàm là **câu tiếng Anh mô tả mệnh đề đang kiểm**, snake_case dài. Không `test_xxx`.

- xoá một khoá `shortcut` rồi đọc lại ⇒ khoá **vắng mặt**, không phải một hàng giá trị rỗng *(đối chứng âm: phân biệt được với lượt ghi `""`)*;
- xoá một khoá **không tồn tại** ⇒ **không lỗi**, không hàng nào bị đụng;
- xoá một `kind` không phải `GlobalOnly` ⇒ `store.write_failed`, cùng luật `save_value`.

### Bàn đo chạy tay — mỗi hàng một ảnh chụp màn hình thật

| # | AC | Thao tác | Kết quả phải thấy |
|---|---|---|---|
| 1 | AC1 | Mở màn hình phím tắt | **Mọi** command của `list()` có mặt — đếm khớp số `check:commands` in ra; mỗi hàng có nhãn tiếng Việt, mã lệnh, phím |
| 2 | AC1 | Đối chiếu vài hàng với mã | `mode.library` = `⌘1` (macOS) · `lookup.toggle_pin` = `⌘D` · `layout.toggle_source` = *"chưa gán"* |
| 3 | AC5 | Xem nhóm *"chưa gán phím"* | Đúng tập mà `check:commands` in ở dòng `unbound()` |
| 4 | AC2 | Gán `Mod+K` cho `layout.toggle_source`, **không** đóng màn hình, bấm `⌘K` | Panel Source ẩn/hiện **ngay** |
| 5 | AC12 | (cùng lượt #4) nhìn lại nhóm "chưa gán" | Hàng đó **đã rời** nhóm, không cần mở lại màn hình |
| 6 | AC4 | Đóng app, mở lại, bấm `⌘K` | Vẫn ẩn/hiện panel; màn hình vẫn hiện `⌘K` |
| 7 | AC3 | Gán `Mod+D` cho một command khác *(`lookup.toggle_pin` đang giữ)* | Hàng hiện **xung đột** kèm tên thao tác đang giữ; **phím cũ không đổi**; đối chứng âm: bấm `⌘D` vẫn ghim, không chạy thao tác mới |
| 8 | AC9 | Trên **macOS**, gán `Meta+D` cho một command khác | Vẫn báo xung đột — hai chuỗi khác nhau, cùng một `resolved` |
| 9 | AC10 | Bấm bắt hợp âm, gõ `Mod+1` | Ô nhận `Mod+1`; chế độ **không** đổi; đối chứng âm: đóng màn hình rồi gõ `⌘1` ⇒ đổi chế độ bình thường |
| 10 | AC11 | Bấm bắt hợp âm, gõ `F1` | Câu `shortcuts.key_unknown` hiện; phím cũ **giữ nguyên** |
| 11 | Bẫy 4 | Bấm bắt hợp âm, gõ `Escape` | Huỷ lượt bắt; lớp phủ **vẫn mở**. Gõ `Escape` lần hai ⇒ lớp phủ đóng |
| 12 | Bẫy 5 | Bấm bắt hợp âm, gõ `⌫` | `Backspace` được gán như một hợp âm — **không** bị đọc thành "bỏ gán" |
| 13 | AC8 | Bỏ gán `mode.library` → đóng/mở app → trả về mặc định → đóng/mở app | Sau bước 2: **không** phím. Sau bước 4: `⌘1` **trở lại**. Hai trạng thái phân biệt được |
| 14 | AC13 | Sửa tay `global.db`: hai command cùng `Mod+1` → mở app → mở màn hình | Câu `shortcuts.disk_rejected` hiện; ứng dụng chạy bằng phím mặc định; **không** cửa sổ trắng |
| 15 | AC6 | **Vòng đầy đủ, không chạm chuột:** mở Tác phẩm ở Library → `⌘2` sang Workspace → `Mod+Alt+→` đi giữa panel → `Mod+Alt+S` + `Shift+→` bôi đen → `Mod+Alt+L` tra → Tab tới chip nguồn + Enter bật/tắt → `⌘D` ghim → `⌘1`/`⌘3` đổi chế độ | Mỗi bước có hiệu lực; **không một lần chạm chuột**; ghi lại bước nào phải đi bằng Tab thay vì hợp âm |
| 16 | AC6 · NFR14 | Chạy hàng 15 trên **cả** macOS lẫn Windows | `Mod+…` phân giải thành `⌘…` và `Ctrl+…`; hành vi tương đương |
| 17 | UX-DR17 | Mở màn hình từ nút titlebar, đóng bằng `Escape` | Tiêu điểm quay **đúng** về nút đã mở, không rơi về `body` |
| 18 | AC10 | Mở màn hình *(chưa bắt hợp âm)*, gõ `Mod+Alt+←` | Vòng xoay panel **vẫn chạy** — cửa chỉ nuốt lúc **đang bắt**, không nuốt lúc mở |
| 19 | Bẫy 1 | Nhìn kỹ đầu màn hình | **Không** thanh chuyển phạm vi; đúng một câu `shortcuts.scope_note` |
| 20 | Bẫy 9 | Gây một xung đột rồi thử mọi phím tắt cũ | Mọi phím cũ **vẫn chạy** — lượt dựng lại trượt không được để lại keymap hỏng |

---

## Dev Agent Record

### Agent Model Used

*(điền lúc thực thi)*

### Debug Log References

#### §Task 0 — kết quả chốt (2026-08-11), bảy quyết định

**Điều kiện khởi hành — ĐẠT.** Story 1.20 đã commit riêng ở `fe952de` *(23 tệp, 3.408 dòng thêm)*; `git status --porcelain` trả **1 dòng** và dòng đó là chính story file 1.21 *(tệp chưa theo dõi, không phải mã)*. Sáu tệp mà story này phải sửa — `commands/index.ts` · `main.ts` · `App.vue` · `vi.json` · `check-commands.mjs` · `check-i18n.mjs` — đều **sạch**. Lý do chặn *("không trộn hai diff")* hết hiệu lực. 🔴 **Ice ký chấp nhận phần còn treo của 1.20** *(9/18 hàng bàn đo chạy tay, cần app Tauri thật)* — 1.20 giữ `in-progress`, và việc đó không đụng một dòng nào của 1.21. `baseline_commit` = `fe952de6ce4ac87b91ab1adef0b431b9f4920536`.

**Quyết định #1 — `#1a` KÈM HAI VÁ. Story viết `#1a` chưa đủ, và mình phản biện bằng số trước khi gõ:**

| Lỗ | Phép đo | Vá |
|---|---|---|
| `overrides?.[id] ?? spec.keys` **không** trả về mặc định sản phẩm | `main.ts:198` truyền `bindings: toBindings(config.shortcuts)` vào `installCommands`, và `registerAll` nướng nó vào `spec.keys` qua `chordsFor` (`index.ts:302-315`). ⇒ `spec.keys` = *"đĩa-hoặc-mặc-định lúc khởi động"*, không phải mặc định. Nút "Trả về mặc định" của AC8 sẽ trả **giá trị đĩa cũ** mà 0 cổng đỏ | Chụp một bản đồ **hợp âm mặc định sản phẩm** lúc cài đặt bằng chính `registerAll(scratch, …, undefined)` đã có sẵn |
| Gán biến module `keymap` **không** tới được listener | `attachKeymap(keymap, target, gate)` (`keys.ts:346`) đóng gói **tham số** `keymap`; listener `:364` gọi `keymap.handle` trên đúng đối tượng đó. Thay biến module ở `index.ts` là thay một tham chiếu mà không ai đọc | `attachKeyboard` truyền một **proxy ổn định** `{ handle, bindings }` uỷ quyền sang keymap hiện tại |

Tương thích ngược đã đo lại: **7** lời gọi `createKeymap` ở `check-commands.mjs` (`:1088` `:1089` `:1160` `:1182` `:1217` `:1235` `:1298`) đều **hai tham số** ⇒ tham số thứ ba tuỳ chọn không làm Kiểm D đỏ.

**Quyết định #2 — GIỮ NGUYÊN.** Lớp phủ `ShortcutsOverlay.vue`, không chế độ thứ tư. `MODE_IDS` (`index.ts:37`) đo được là hằng **ba** phần tử.

**Quyết định #3 — GIỮ NGUYÊN, nhưng mệnh đề *"0 dòng Rust"* bị Quyết định ⑦ lật.** Đường ĐỌC/GHI xác nhận có sẵn: `ScopeKind::Shortcut => "shortcut" : GlobalOnly` (`kinds.rs:204`) · `save_value` từ chối mọi loại không `GlobalOnly` (`store.rs:294-304`) · `putConfig(kind,key,value)` trả `IpcError | null` và **không ném** (`bootstrap.ts:196-219`). **0 bước di trú, 0 `MessageKey` mới** vẫn đúng. Phép đo đóng `deferred-work.md:241` cũng xác nhận: `NAMED_CODES` (`keys.ts:97-123`) có **25** khoá, `Comma`/`Period`/`Slash`… đều là **tên chữ cái**, và `keyToCode` (`:125-137`) chỉ nhận `[0-9]`, `[A-Za-z]`, khoá của `NAMED_CODES` ⇒ không hợp âm hợp lệ nào chứa `','`.

**Quyết định #4 — GIỮ NGUYÊN.** `src/config/shortcutsState.ts`. Xác nhận lời cấm: `check-commands.mjs` `import()` bốn tệp bằng Node thuần ở `:799` `:962` `:1040` `:1257`.

**Quyết định #5 — GIỮ NGUYÊN.** Năm command tĩnh, `shortcuts.open` mang `Mod+Comma`, bốn cái còn lại `undefined`.

**Quyết định #6 — GIỮ NGUYÊN.** Bốn trạng thái một hàng; `⌫` bỏ gán ở trạng thái **nghỉ**, là hợp âm thường ở trạng thái **đang bắt**; ô phím là `<button>`.

**Quyết định #7 — 🔴 ICE LẬT MỘT PHẦN, 2026-08-11.** Story đề xuất trả cả ba món nợ; **Ice chốt: NHẬN `deferred-work.md:656` (cờ `repeatable`), TRẢ hai món còn lại** (`:491` preset đặt tên · `:1135` token thứ tự). Mình đã nói trước cái giá — `repeatable` chạm `registry.ts` + `keys.ts` + mọi command đang có, cho một thay đổi **0 AC** ở `epics.md` — và Ice ký nhận. ⇒ story này mang thêm một việc ngoài AC, ghi ra ở đây bằng chữ để lượt review sau không đọc nó thành phạm vi trôi.

**Dev Notes ⑦ — Ice chọn (a).** Thêm `delete_config` Rust. Đo lại lý do (b) bị bác: `save_value` (`store.rs:311-321`) là `INSERT … ON CONFLICT DO UPDATE`, **không** có đường xoá; ghi hợp âm mặc định xuống đĩa biến hàng đó thành giá trị đóng băng.

#### Sàn `*_FLOOR` — số THẬT lúc mở story (mốc so cho AC14)

| Sàn | Tệp | Giá trị sàn | Số thật ghi trong chú thích (2026-08-10, sau 1.20) |
|---|---|---|---|
| `VUE_FLOOR` | `check-commands.mjs:211` | 12 | 14 tệp `.vue` |
| `TS_FLOOR` | `check-commands.mjs:212` | 24 | 30 tệp `.ts` |
| `COMMAND_FLOOR` | `check-commands.mjs:219` | 24 | 29 command |
| `CLICK_FLOOR` | `check-commands.mjs:237` | 13 | 16 thuộc tính `@click` |
| `DISPATCH_FLOOR` | `check-commands.mjs:238` | 20 | 25 lời gọi `dispatch()` |
| `SELECTION_SURFACE_FLOOR` | `check-commands.mjs:1648` | 6 | — |
| `RS_FLOOR` | `check-i18n.mjs:276` | 35 | 41 tệp `.rs` |
| `VUE_FLOOR` | `check-i18n.mjs:281` | 12 | 14 tệp `.vue` |

#### Bộ đo bằng máy — 24 phép đo trên đường sản phẩm THẬT

Nạp chính `src/commands/index.ts` của sản phẩm bằng **Node thuần** *(cùng cửa mà Kiểm C/D/E của cổng đã đi qua)*, `installCommands({ setMode, isMac: true })` với **0 hợp âm nào từ đĩa**, rồi lái từng hàng của §Testing không cần một cửa sổ Tauri. Kết quả: **24/24 đạt**.

| Hàng | AC | Phép đo | Kết quả |
|---|---|---|---|
| 2 | AC1 | `mode.library` · `lookup.toggle_pin` · `shortcuts.open` · `layout.toggle_source` | `⌘1` · `⌘D` · `⌘,` · *chưa gán* |
| 3 | AC5 | nhóm chưa gán | **16** thao tác, khớp dòng `unbound()` của `check:commands` |
| 4 | AC2 | gán `Mod+K` cho `layout.toggle_source` | có hiệu lực **ngay** trên nguồn lúc chạy ⇒ `⌘K` |
| 5 | AC12 | (cùng lượt) nhìn lại nhóm chưa gán | hàng đó **rời** nhóm ngay; **đối chứng**: `registry.unbound()` **vẫn** nói cũ — đúng như doc-comment khai |
| 7 | AC3 | `conflictFor('Mod+D', 'layout.toggle_lookup')` | báo xung đột, chủ = `lookup.toggle_pin`; `⌘D` **không đổi**; hàng đối thủ **không** được gán |
| 8 | AC9 | `conflictFor('Meta+D', …)` trên macOS | **vẫn** xung đột, `resolved` chung = `Meta+KeyD`; **đối chứng âm**: một hợp âm trống ⇒ `null` |
| 13 | AC8 | bỏ gán → về mặc định | bỏ gán ⇒ không phím **và** khoá có mặt mang `[]`; về mặc định ⇒ `⌘1` **trở lại** và lớp override im lặng |
| 20 | Bẫy 9 | gán một hợp âm gây xung đột | lượt gán **bị từ chối**; số binding không đổi; `⌘1` · `⌘D` · `⌘K` **đều vẫn chạy** |
| — | AC7 | `registry.unbound()` trên bộ mặc định | khác rỗng |

⚠️ **Giới hạn, ghi thẳng thay vì để người sau tự phát hiện:** bộ đo này không chạm DOM, không có `window`, không một tiến trình khởi động lại nào. Nên nó **không** đo được cửa nuốt hợp âm, hai nghĩa của `Escape`, `⌫` gán được, tiêu điểm quay về, hay bất cứ hàng thị giác nào — 12 hàng đó ở lại Task 7 và có mục riêng ở `deferred-work.md`.

### Completion Notes List

**Story giao trọn 14 AC ở tầng cơ chế; 12 trên 20 hàng bàn đo chạy tay còn treo (Task 7).**

**Chín trên chín cổng XANH** · `npm run build` **XANH** · `cargo test` **264 xanh, 0 đỏ, 5 ignored** *(261 → 264, ba ca mới cho đường xoá khoá)* · bộ đo bằng máy **24/24 đạt**.

**AC7 — số thật, đọc từ log của `check:commands`.** `unbound()` trên bộ command thật giữ **16** phần tử: `layout.toggle_source` · `layout.toggle_lookup` · `layout.toggle_ai_translation` · `layout.toggle_editor` · `library.import_text` · `library.import_file` · `lookup.toggle_source` · `attribution.open` · `attribution.close` · `lookup.select_tab_record` · `lookup.select_tab_history` · `lookup.clear_history` · `shortcuts.close` · `shortcuts.capture` · `shortcuts.unassign` · `shortcuts.reset`. Bốn cái cuối là của chính story này — bộ mặc định của sản phẩm **không** đổi, và màn hình là để **người dùng** gán.

**AC14 — mọi sàn nâng theo SỐ THẬT, không ước.**

| Sàn | Trước | Sau | Số thật | Tỷ lệ |
|---|---|---|---|---|
| `VUE_FLOOR` (`check-commands`) | 12 | **13** | 15 tệp `.vue` | 86,7% |
| `TS_FLOOR` | 24 | **26** | 31 tệp `.ts` | 83,9% |
| `COMMAND_FLOOR` | 24 | **29** | 34 command | 85,3% |
| `CLICK_FLOOR` | 13 | **17** | 21 `@click` | 81,0% |
| `DISPATCH_FLOOR` | 20 | **23** | 28 `dispatch()` | 82,1% |
| `SELECTION_SURFACE_FLOOR` | 6 | **7** | 7 bề mặt | = số thật, theo doctrine của chính sàn đó |
| `VUE_FLOOR` (`check-i18n`) | 12 | **13** | 15 tệp `.vue` | 86,7% |
| `RS_FLOOR` (`check-i18n`) | 35 | **35** | 41 tệp `.rs` | **không nâng** — story sửa hai tệp `.rs` đã có, **0** tệp mới |

**Ba cơ chế mới, mỗi cái một lưới ở Kiểm D** *(chúng không có phép kiểm nào trước story này, và một cơ chế không lưới là một lời hứa)*:
1. **`chordFromEvent`/`formatChord`** — vòng khứ hồi trên **5 sự kiện × 2 nền tảng**, cộng một khẳng định rằng cùng một sự kiện cho **hai** hợp âm khác nhau giữa hai nền tảng *(không có vế này thì vòng khứ hồi vẫn xanh với một bản bỏ qua `isMac` hoàn toàn)*, cộng ba ca `null` *(phím ngoài bảng · chỉ phím bổ trợ · lượt commit của bộ gõ)*, cộng hai ca `formatChord`.
2. **`repeatable`** — nhánh dương **và** đối chứng âm trên **cùng một keymap**.
3. **`overrides`** — cả **bốn** trạng thái: vắng mặt *(tương thích ngược)* · khoá vắng mặt · khoá có mảng rỗng · khoá có phần tử.

**Quyết định #1 — phản biện bằng số, và nó đổi hình dạng lời giải.** Story đề xuất `#1a` *(thêm `overrides` cho `createKeymap`)*. Đo được **hai lỗ** mà `#1a` như viết không lấp:
- `registerAll` nướng hợp âm **từ đĩa** vào `spec.keys` lúc đăng ký ⇒ `overrides?.[id] ?? spec.keys` với khoá bỏ đi trả về *"giá trị đĩa lúc khởi động"*, **không** mặc định sản phẩm. Nút "Trả về mặc định" của AC8 sẽ trả sai giá trị mà **0 cổng đỏ**. ⇒ **Vá:** `registerAll` **thôi nhận** `bindings`; nó đăng ký bộ mặc định sản phẩm, **luôn luôn**, và lớp của đĩa sống trọn trong `overrides`. Sau story này `spec.keys` có đúng **một** nghĩa ở mọi chỗ đọc nó. `chordsFor` bị xoá — hợp đồng ba trạng thái của nó chuyển sang doc-comment của `ChordOverrides`.
- `attachKeymap` đóng gói **đối tượng** keymap đã truyền vào ⇒ gán biến module `keymap` mới **không tới được listener**; triệu chứng là *"gán phím xong, phím mới không chạy"* và **không lỗi nào ném**. ⇒ **Vá:** `attachKeyboard` truyền một **proxy ổn định** uỷ quyền sang keymap hiện tại.

**Hai thứ BIẾN MẤT, và cả hai là đơn giản hoá chứ không đánh đổi:**
- **Registry nháp** của `bindingsAreUsable` — `createKeymap` **chỉ đọc** registry, nên lượt thử chạy thẳng trên registry thật. Một biến thể ít hơn để hai đường trôi khỏi nhau.
- **Bẫy 10** *(`installCommands` không giữ `deps`)* — mất luôn điều kiện kích hoạt: lượt dựng lại **không** `registerAll` lần nữa, nên nó không cần bộ handler nào. Story ghi *"cả hai đường đều cần giữ `deps`"*; điều đó đúng cho `#1b`, không đúng cho `#1a` sau khi vá.

**Quyết định #7 — Ice lật một phần.** NHẬN `deferred-work.md:656` *(cờ `repeatable`)*, TRẢ `:491` và `:1135`. Cái giá đã nói trước và Ice ký nhận: `repeatable` chạm `registry.ts` + `keys.ts` + mọi command đang có cho một thay đổi **0 AC**. Ghi ra đây bằng chữ để lượt review sau không đọc nó thành phạm vi trôi.

**Dev Notes ⑦ — Ice chọn (a).** `delete_config` Rust: **+~30 dòng** trên hai tệp đã có (`commands/config.rs` · `core/scope/store.rs`), đi qua `From<StoreError>` sẵn có, **0 `MessageKey` mới**, **0 bước di trú**, **0 tệp `.rs` mới**. Ba ca test, và ca đầu mang **đối chứng âm** — nó chạy *bỏ gán* và *trả về mặc định* cạnh nhau rồi khẳng định chúng cho ra hai trạng thái phân biệt được trên dây. Không có vế đó thì một bản cài đặt kiểu *"xoá = ghi chuỗi rỗng"* vẫn xanh và AC8 mất đúng thứ nó nói.

**Một lượt vá tự bắt ở cổng:** bản đầu đánh dấu hàng đang nhắm bằng `box-shadow` — Kiểm F của `check:tokens` **đỏ** *(không bóng đổ, không lớp nổi)*. Vá bằng `border-left` trong suốt khai ở **mọi** hàng *(nếu chỉ khai ở hàng được chọn thì cả bảng nhảy 2px mỗi lần đổi hàng)*, **cộng** một nhãn chữ `shortcuts.state_aimed` — UX-DR27 đòi trạng thái nói ra bằng **chữ**, không bằng màu một mình, và ba nút thao tác đều áp vào hàng này nên nó là một trạng thái người dùng **phải** biết.

**Nợ đóng ở story này:** `deferred-work.md:241` *(mã hoá hợp âm — đóng bằng một phép đo rồi một cơ chế ở Kiểm D, không bằng một mô hình mới)* · `:243` *(xung đột từ đĩa nay NÓI RA, AC13)* · `:656` *(cờ `repeatable`)*. **Đóng một nửa:** `:485` — người dùng nay **gán được** phím cho bốn `layout.toggle_*`, nhưng bộ **mặc định** không đổi và không được đổi (AC7). **Đổi chủ:** `:491` · `:1135`.

### File List

**NEW** *(2)*

- `src/ShortcutsOverlay.vue` — lớp phủ phím tắt, khuôn `AttributionOverlay.vue`
- `src/config/shortcutsState.ts` — state + đường ghi đĩa + năm handler

**UPDATE** *(15)*

- `src/commands/keys.ts` — `chordFromEvent` · `formatChord` · `resolveChord` · `resolvedOf` · `MODIFIER_CODES` · `KEY_GLYPHS` · `MAC_KEY_GLYPHS` · `codeToKey` · tham số thứ ba `overrides` của `createKeymap` · nhánh `repeatable` trong `handle`
- `src/commands/registry.ts` — `CommandSpec.repeatable` · doc-comment §Dev Notes ⑤ trên `keys` và `unbound()` · `frozen()` chở cờ mới
- `src/commands/index.ts` — năm command · năm cổng `CommandDeps` · `applyBindings` · `conflictFor` · `effectiveBindings` · `effectiveUnbound` · `defaultChordsFor` · `overrideFor` · `currentOverrides` · `shortcutsDiskRejection` · proxy ổn định ở `attachKeyboard` · **xoá** `chordsFor` và `bindingsAreUsable` · `registerAll` thôi nhận `bindings`
- `src/config/bootstrap.ts` — `deleteConfig` · `SCOPE_SHORTCUT` · `CMD_DELETE`
- `src/main.ts` — nối năm handler · cửa nuốt hợp âm **hai vị từ**
- `src/App.vue` — nút `data-shortcuts-open` ở `titlebar` · dựng `ShortcutsOverlay`
- `src/i18n/vi.json` — **22** khoá mới *(5 nhãn command + 17 chuỗi màn hình)*
- `src-tauri/src/commands/config.rs` — `delete_config` + vỏ `wire::delete_config`
- `src-tauri/src/core/scope/store.rs` — `delete_value`
- `src-tauri/src/core/scope/mod.rs` — tái xuất `delete_value`
- `src-tauri/src/lib.rs` — đăng ký `delete_config` vào `generate_handler!`
- `src-tauri/tests/scope_contract.rs` — **3** ca mới cho đường xoá khoá
- `scripts/check-commands.mjs` — 6 sàn nâng · lưới `chordFromEvent`/`formatChord` hai nền tảng · lưới `repeatable` · lưới `overrides` bốn trạng thái · khẳng định `NAMED_CODES` không chứa `,`
- `scripts/check-i18n.mjs` — `VUE_FLOOR` nâng; `RS_FLOOR` giữ nguyên kèm lý do
- `_bmad-output/implementation-artifacts/deferred-work.md` — 6 mục mới, 6 mục cũ cập nhật

---

## Change Log

| Ngày | Việc |
|---|---|
| 2026-08-11 | Story dựng. Bảy quyết định để mở chờ Task 0; §Điều kiện khởi hành ghi mệnh đề chặn theo trạng thái cây làm việc lúc dựng |
| 2026-08-11 | **Task 0 — bảy quyết định CHỐT, Ice ký bốn câu hỏi.** Xem §Debug Log References |
| 2026-08-11 | **Task 1–6 giao trọn.** 14 AC ở tầng cơ chế; chín trên chín cổng XANH · `build` XANH · `cargo test` 264/0/5 · bộ đo bằng máy 24/24 |
| 2026-08-11 | 🔴 **LỆCH MOCKUP, bốn chỗ — ghi ra, KHÔNG dựng theo và KHÔNG sửa mockup** *(Quyết định #3 của Story 1.3)*: ① thanh chuyển phạm vi `Toàn cục`/`Tác phẩm` (`settings.html:243-248`) — `kinds.rs:29-37` cấm bằng chữ và gọi đích danh story này; ② khung điều hướng chín mục Cài đặt (`:251-262`) — thuộc Epic 4/5/6/10; ③ Xuất/Nhập bộ phím tắt (`:291-292`) — 0 AC; ④ ô tìm kiếm / tra ngược hợp âm (`:269`) — 0 AC |
| 2026-08-11 | ⚠️ **Task 7 CÒN TREO — story ở `review`, KHÔNG `done`.** 8/20 hàng bàn đo đã chạy **bằng máy**; **12** hàng còn lại cần một cửa sổ Tauri thật, và hàng 16 cần một máy Windows. Danh sách đầy đủ kèm lý do ở `deferred-work.md` |
