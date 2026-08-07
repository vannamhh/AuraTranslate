---
baseline_commit: b482dc174e9f2e804ae5286d6c512c6820f74ba6
---

# Story 1.6: CommandRegistry, ba chế độ, và tiêu điểm bàn phím

Status: done

> **Lượt code review 2026-08-04 đã đóng.** 2 quyết định *(Ice phân xử)* · 27 patch **đã vá và chứng minh đỏ-rồi-xanh 20 ca** · 4 mục hoãn có lý do · 3 mục loại bỏ. Sáu lệnh nghiệm thu đều exit 0 sau lượt vá.
> 🔴 **AC4 đóng ở mức ĐẠT MỘT PHẦN** theo quyết định của Ice: vế khai báo đạt trọn và đã kiểm được bằng cổng; vế *"mỗi **panel** dời focus DOM tường minh"* chưa có đường chạy được trong sản phẩm hôm nay và được giao cho **Story 1.14 / 1.21**. Chi tiết và lý do ở §Review Findings và `deferred-work.md`. Đừng đọc `done` ở đây thành "cả sáu AC đạt trọn".

**Covers:** FR22 *(nửa cấu trúc — nửa "cấu hình lại được" đóng ở Story 1.21)* · NFR17 · AD-34 §1 và §2 · AD-24 · UX-DR7, UX-DR8, UX-DR17 *(phần tiêu điểm)*, UX-DR34
**Epic:** 1 — Nền móng ứng dụng & Tra cứu ngoại tuyến tức thì

<!-- Note: Validation is optional. Run validate-create-story for quality check before dev-story. -->

---

## Story

As a **người dịch**,
I want **mọi thao tác của ứng dụng gọi được bằng bàn phím và luôn thấy rõ mình đang ở đâu**,
so that **một phiên làm việc dài không bắt tôi rời tay khỏi bàn phím**.

> **Vì sao story này đứng ở đây, trước Story 1.14 dựng panel.** NFR17 là yêu cầu cắt ngang thứ hai *(cùng NFR16)* mà PRD bắt áp **từ Giai đoạn 1**. AD-34 nói thẳng lý do: sàn khả năng tiếp cận *"chỉ lộ ra khi có người thử dùng bằng bàn phím, và tới lúc đó đã quá muộn để sửa rẻ"*. Nếu `CommandRegistry` đến sau bốn panel, thì bốn panel sẽ được viết với handler chuột tại chỗ, và story này biến thành một lượt refactor toàn bộ frontend thay vì một lượt dựng nền.
>
> Hệ quả ngược lại cũng đúng và phải nói ra: story này **dựng cơ chế khi chưa có gì để cơ chế đó điều khiển**. Nó phải chống lại hai cám dỗ đối nghịch — dựng thừa (bốn panel thật, dockview, màn hình gán phím) và dựng rỗng (một registry không ai gọi, một `unbound()` luôn trả mảng rỗng). §Ranh giới phạm vi vạch đường giữa hai bên.

---

## Acceptance Criteria

### AC1 — Mọi thao tác đăng ký ở `CommandRegistry` trước khi bind; handler chuột chỉ `dispatch`

**Given** `CommandRegistry`
**When** một thao tác được thêm vào ứng dụng
**Then** nó đăng ký ở đó **trước** khi bind vào chuột hoặc phím
**And** handler chuột chỉ được `dispatch` một command đã đăng ký, không tự cài đặt thao tác tại chỗ

> **Nghiệm thu bằng mã thoát, không bằng lời hứa.** Đây là mệnh đề trung tâm của AD-34 §1 và nó là loại quy tắc thoái hoá thành kỷ luật cá nhân qua bảy giai đoạn — đúng loại mà `check-deps.mjs`, `check-tokens.mjs`, `check-i18n.mjs` đã lần lượt đóng bằng một cổng. Story này nợ cổng thứ tư: `scripts/check-commands.mjs`.
>
> ⚠️ **"Không tự cài đặt thao tác tại chỗ" phải quy về một phép kiểm cú pháp đọc được**, nếu không nó là văn xuôi. Luật đã chốt: mọi biểu thức `@click` / `v-on:click` trong `.vue` phải là **đúng một lời gọi `dispatch('<id>')`** — không phải một hàm khác, không phải mã nội tuyến. Xem §Kiểm A của cổng.

### AC2 — Command id là khoá chấm có tiền tố miền; id trùng bị phát hiện lúc đăng ký

**Given** một command id
**When** đăng ký
**Then** dùng khoá chấm có tiền tố miền, cùng hình dạng khoá `vi.json`
**And** hai command trùng id bị phát hiện lúc đăng ký, không ghi đè im lặng

> **"Cùng hình dạng" nghĩa là cùng một biểu thức chính quy, chép đúng — không phải "trông na ná".** `check-i18n.mjs` Kiểm B đang cưỡng chế `^[a-z0-9]+(\.[a-z0-9_]+)+$` cho khoá `vi.json`. Command id dùng **đúng biểu thức đó**. Lượt review Story 1.5 đã bắt được một ca hai phép kiểm cưỡng chế *hai văn phạm khoá khác nhau* cho cùng một thứ (`ipc_contract.rs` vs Kiểm B); đừng tạo ca thứ hai.
>
> **"Không ghi đè im lặng" = ném.** Không `console.warn` rồi vẫn ghi đè: một cảnh báo trong biển log lúc khởi động là im lặng theo nghĩa thực dụng. Đăng ký trùng id là lỗi lập trình, xảy ra lúc khởi động, và **ném thì đỏ ngay ở màn hình đầu tiên** — rẻ nhất có thể. Đây cũng chính là hố mà AD-34 nêu đích danh: *"hai giai đoạn cách nhau nhiều tháng đăng ký trùng id trần sẽ ghi đè nhau âm thầm"*.

### AC3 — Ba command `mode.library` · `mode.workspace` · `mode.reading`, gọi được bằng `⌘1` `⌘2` `⌘3`

**Given** ba command `mode.library`, `mode.workspace`, `mode.reading`
**When** ứng dụng khởi động
**Then** cả ba đã đăng ký và gọi được bằng `⌘1` `⌘2` `⌘3`, kể cả khi Library và Chế độ đọc chưa có nội dung
**And** chúng là ba chế độ **ngang hàng** trong **một** cửa sổ hệ điều hành

> 🔴 **`⌘` là ký hiệu macOS của một phím *trừu tượng*, không phải một mệnh đề về `event.metaKey`.** Trên Windows phím đó là `Ctrl`. Một cài đặt chỉ đọc `metaKey` cho ra một sản phẩm **không chuyển chế độ được trên Windows** — vi phạm NFR14, và CI hai nền tảng của Story 1.3 **không bắt được** vì không có test nào chạm tầng bàn phím. Xem §Trap 1.
>
> **"Kể cả khi chưa có nội dung"** là điều kiện nghiệm thu chứ không phải một lời trấn an: ba chế độ phải chuyển được **ngay hôm nay**, với Library và Chế độ đọc là hai khung rỗng có một câu giải thích lấy từ `vi.json`.

### AC4 — Mỗi chế độ và mỗi panel dời focus DOM tường minh; focus không bao giờ rơi về `body`

**Given** mỗi chế độ và mỗi panel
**When** được kích hoạt
**Then** nó dời focus DOM tường minh tới điểm vào đã khai
**And** focus không bao giờ rơi về `body`

> **Hai vế, hai cơ chế khác nhau.** Vế đầu là **dữ liệu** — mỗi chế độ/panel *khai* một điểm vào — và dữ liệu thì máy kiểm được. Vế sau là **hành vi DOM lúc chạy**, mà dự án không có bộ chạy test frontend *(và không được thêm — xem §Không thêm phụ thuộc)*. Đường đi đã chốt: khai báo cưỡng chế bằng cổng; hành vi cưỡng chế bằng **một chốt lúc chạy tự kêu** (`console.error` khi `document.activeElement` rơi về `body` sau một lần chuyển) cộng một lượt nghiệm thu tay có ghi bảng. **Không đánh dấu đạt cho vế DOM bằng suy luận** — ghi giới hạn vào `deferred-work.md` theo đúng tiền lệ `unmeasured` của Story 1.3 và AC6 của Story 1.4.

### AC5 — Panel có tiêu điểm: vạch dọc 2px `primary` mép trái + tiêu đề `primary` in đậm

**Given** một panel có tiêu điểm
**When** quan sát
**Then** có vạch dọc 2px `primary` ở mép trái và tiêu đề chuyển `primary` in đậm
**And** không dùng viền bao quanh để báo tiêu điểm

> **"Không dùng viền bao quanh" áp cho *panel*, KHÔNG phải một lệnh xoá focus ring toàn ứng dụng.** Một `*:focus { outline: none }` là cách nhanh nhất phá NFR17 mà vẫn qua được mọi cổng hiện có. Xem §Trap 4.
>
> **Vạch không được làm bằng `box-shadow`.** `check-tokens.mjs` Kiểm F cấm `box-shadow` và `text-shadow` **không có đường miễn trừ** (AC7 Story 1.4 — không elevation). Cách đúng đã có sẵn trong chính mockup: một `::before` `position:absolute; left:0; width:2px; background: var(--color-primary)`.

### AC6 — `CommandRegistry` liệt kê được các thao tác chưa gán phím nào

**Given** `CommandRegistry`
**When** truy vấn
**Then** liệt kê được danh sách thao tác **chưa gán phím nào**

> ⚠️ **Một `unbound()` trả mảng rỗng là một AC chưa được chứng minh.** Nếu mọi command của story đều có phím thì phép truy vấn này không bao giờ chạy qua nhánh có nghĩa, và Story 1.21 sẽ phát hiện nó hỏng khi đã có 40 command. §Quyết định thiết kế #5 chốt một command **cố ý để trống phím** — `focus.next_panel` — vì lý do độc lập và đúng đắn, chứ không phải để làm cảnh cho AC này.

---

## Tasks / Subtasks

- [x] **Task 1 — Đọc trước, đo trước** (AC: tất cả)
  - [x] Đọc §Ranh giới phạm vi, §Bốn cái bẫy, §Quyết định thiết kế **trước** khi gõ dòng đầu tiên
  - [x] Chạy bốn cổng đang có trên `HEAD` và chép kết quả vào §Debug Log References — đây là đường cơ sở: `npm run check:deps` · `npm run check:tokens` · `npm run check:i18n` · `npm run build`
  - [x] Đối chiếu với §Trạng thái repo hiện tại. Lệch thì **ghi từng chỗ lệch kèm commit gây ra nó** rồi đi tiếp, đừng dừng — tiền lệ Task 1 của Story 1.5
  - [x] **Đừng "dọn" gì ở task này.** Task 1 chỉ đo

- [x] **Task 2 — `src/commands/registry.ts`: registry thuần, không `import` gì** (AC: 1, 2, 6)
  - [x] Tạo `src/commands/registry.ts` với `createRegistry()` — hình dạng API ở §Quyết định thiết kế #1
  - [x] **Tệp này KHÔNG được `import` bất cứ thứ gì** — không Vue, không `./focus`, không `@tauri-apps/api`. Đây là điều kiện để `check-commands.mjs` nạp được nó mà kiểm **hành vi**, đúng khuôn `resolve.ts` của Story 1.5 đã chạy thật *(Node ≥ 22.18 bóc kiểu TypeScript mặc định; máy Ice v22.22.2 ✅, CI `node-version: '22'` ✅)*
  - [x] Không `enum`, không `namespace`, không parameter property — ba thứ Node từ chối bóc kiểu. Dùng union type chuỗi
  - [x] `register(spec)` cưỡng chế **ba** thứ, mỗi thứ **ném** với thông báo nêu đích danh id: id trùng (AC2) · id sai văn phạm `^[a-z0-9]+(\.[a-z0-9_]+)+$` (AC2) · `labelKey` rỗng
  - [x] `dispatch(id)` với id **chưa đăng ký** ⇒ **ném**. Đây là nửa cưỡng chế lúc chạy của AC1: cổng canh cú pháp `.vue`, `dispatch` canh mọi đường còn lại
  - [x] `unbound()` trả về các command có `keys` rỗng/vắng (AC6). ⚠️ Trả **bản sao**, không trả tham chiếu vào kho nội bộ — Story 1.21 sẽ dựng màn hình gán phím trên chính hàm này
  - [x] `list()` trả thứ tự **đăng ký**, ổn định. Story 1.21 hiển thị danh sách này cho người dùng; một thứ tự đổi theo `Object.keys` là một màn hình nhảy chỗ mỗi lần mở

- [x] **Task 3 — `src/commands/keys.ts`: tầng bàn phím trung lập nền tảng** (AC: 1, 3)
  - [x] Chuỗi hợp âm viết ở dạng **trung lập**: `Mod+1`, `Mod+Shift+Enter`. `Mod` = `⌘` trên macOS, `Ctrl` ở nơi khác
  - [x] 🔴 Nhận biết nền tảng đi qua **một tham số tiêm được** (`createKeymap(registry, { isMac })`), không đọc thẳng `navigator` ở tầng module — nếu không thì cổng không lái được nó và §Trap 1 không nghiệm thu được
  - [x] **KHÔNG dùng `tauri-plugin-global-shortcut`.** Ba lý do, mỗi lý do đủ để loại: (1) một phụ thuộc mới phải rà GPLv3 và vào bảng Stack **trước khi** thêm (NFR15) — chưa ai rà; (2) nó đăng ký phím ở **tầng hệ điều hành**, tức `⌘1` bị cướp khỏi mọi ứng dụng khác trong khi AuraTranslate chạy nền; (3) *"Global Hotkeys"* của FR22 nghĩa là **toàn ứng dụng**, không phải toàn hệ điều hành — đọc danh sách thao tác nó liệt kê là thấy
  - [x] Khớp phím **chữ và số** bằng `event.code` (`Digit1`), không bằng `event.key` — bố cục bàn phím không phải US làm `event.key` trôi
  - [x] Hợp âm khớp ⇒ `preventDefault()` rồi `dispatch`. Không khớp ⇒ **không đụng vào event**
  - [x] 🔴 **Luật vùng gõ, chốt từ hôm nay dù chưa có ô nhập nào:** hợp âm **không có phím bổ trợ** thì **không** dispatch khi focus đang ở `input` / `textarea` / `[contenteditable]`. Chế độ đọc dùng `M`, `B`, `1 2 3` trần (UX-DR46) và Editor của Epic 2 là một vùng gõ tự do — không có luật này thì gõ chữ "b" trong bản dịch sẽ bật chế độ song ngữ. Rẻ hôm nay, đắt ở Epic 2
  - [x] Đăng ký listener trên `window` với `{ capture: true }`; trả về hàm gỡ. ⚠️ `noUnusedLocals` đang bật — dùng hàm gỡ hoặc `void` nó tường minh

- [x] **Task 4 — `src/commands/focus.ts`: sổ điểm vào focus + chốt chống rơi về `body`** (AC: 4)
  - [x] `declare(owner, resolve)` — `owner` là id của chế độ hoặc panel; `resolve()` trả `HTMLElement | null`
  - [x] `enter(owner)` dời focus **tường minh** (`el.focus()`), trả `false` + `console.error` nêu đích danh `owner` khi không tìm thấy phần tử
  - [x] Chốt AC4 vế sau: sau mỗi lần `enter`, kiểm ở **frame kế tiếp** (`requestAnimationFrame`) rằng `document.activeElement` không phải `document.body`; rơi về `body` ⇒ `console.error` nêu owner. **Đừng tự "sửa" bằng cách focus lại vòng lặp** — một vòng focus tự phục hồi sẽ đánh nhau với người dùng và với hộp thoại của OS; chốt này để **kêu**, không để vá
  - [x] `owners()` liệt kê các owner đã khai — đầu vào cho Kiểm C của cổng
  - [x] Phần tử đích phải nhận được focus: `tabindex="-1"` trên gốc mỗi chế độ/panel

- [x] **Task 5 — Ba chế độ + vỏ cửa sổ một cửa sổ** (AC: 3, 4)
  - [x] `src/modes/LibraryMode.vue` · `WorkspaceMode.vue` · `ReadingMode.vue` — mỗi tệp là một khung rỗng có **một** câu trạng thái lấy từ `vi.json`, gốc mang `tabindex="-1"` và khai điểm vào focus
  - [x] **Đừng dựng bốn trạng thái rỗng của UX-DR31** — chúng thuộc Story 1.14/1.15/5.x và cần nội dung thật để viết đúng. Một câu mỗi chế độ, đúng một câu
  - [x] `src/modes/modeState.ts` — chế độ đang hiện, kiểu `'library' | 'workspace' | 'reading'`. Tệp này **được phép** `import` Vue (khác `registry.ts`)
  - [x] 🔴 **Ba chế độ giữ sống, không huỷ-dựng lại:** bọc bằng `<KeepAlive>`. UX-DR34 và FR12 hứa *"chuyển chế độ luôn giữ ngữ cảnh — rời Workspace sang Chế độ đọc rồi quay lại thì vẫn đúng Chương, đúng câu, đúng vị trí cuộn"*. Hôm nay chưa có ngữ cảnh nào để mất, nên một cài đặt `v-if` sẽ **xanh mọi phép kiểm** và đúng tới ngày Epic 2 có nội dung — rồi hỏng ở một chỗ không ai nối lại được với story này
  - [x] `App.vue`: thanh tiêu đề cao `var(--space-titlebar-height)` với **ba tab chế độ**, mỗi tab `@click="dispatch('mode.…')"` — đây là chỗ AC1 có mã thật để kiểm thay vì kiểm trên hư không
  - [x] **Đừng đụng khối self-check trong `App.vue`** (`VITE_SCOPE_SELFTEST`, `fallbackReport`, import động `scopeCheck`) — đọc comment `App.vue:12-16` trước khi sửa tệp này

- [x] **Task 6 — `src/panels/PanelFrame.vue`: vỏ panel và hợp đồng thị giác tiêu điểm** (AC: 4, 5)
  - [x] Thanh tiêu đề `var(--space-head-height)`, tiêu đề `ui-md` màu `on-surface-variant` (UX-DR17)
  - [x] Có tiêu điểm ⇒ `::before` vạch dọc **2px** `var(--color-primary)` mép trái + tiêu đề đổi sang `var(--color-primary)` và `font-weight: 600`
  - [x] Không `box-shadow`, không `text-shadow` — Kiểm F của `check-tokens.mjs` cấm, **không có miễn trừ**
  - [x] Không màu viết thẳng, không cỡ chữ viết thẳng — Kiểm B và B2 của `check-tokens.mjs`
  - [x] `WorkspaceMode.vue` dựng **hai** `PanelFrame` — `panel.source` và `panel.editor`, đúng cặp mà UX-DR15 nói *"không bao giờ nhường"*. Hai chứ không bốn: một cái không đủ để nhìn thấy tương phản có/không tiêu điểm; bốn cái là dựng trước Story 1.14
  - [x] Thân panel **để trống** — nội dung là Story 1.16/1.17

- [x] **Task 7 — Đăng ký bộ command khởi động và khoá phím** (AC: 1, 2, 3, 6)
  - [x] `src/commands/index.ts` — chỗ **duy nhất** đăng ký, đúng khuôn "một chỗ chạm" của `src/i18n/index.ts`
  - [x] Bốn command: `mode.library` → `Mod+1` · `mode.workspace` → `Mod+2` · `mode.reading` → `Mod+3` · `focus.next_panel` → **không gán phím** *(lý do ở §Quyết định thiết kế #5)*
  - [x] Mỗi command mang `labelKey = 'command.' + id` — xem §Quyết định thiết kế #4 về vì sao có tiền tố
  - [x] `focus.next_panel` **có handler chạy thật** (xoay vòng focus giữa các panel đã khai), chỉ thiếu phím. Không đăng ký một command rỗng cho đủ số

- [x] **Task 8 — Chuỗi giao diện vào `vi.json`** (AC: 3, 5)
  - [x] Thêm khoá cho: ba nhãn chế độ · hai tiêu đề panel · một câu trạng thái mỗi chế độ · nhãn `focus.next_panel`
  - [x] 🔴 **Thuật ngữ đã chốt ở PRD §5.2 — không tự dịch lại:** `Library` *(PRD gạch bỏ "Thư viện" và ghi "dùng Library nhất quán")* · `Workspace` *(gạch bỏ "màn hình dịch")* · `Chế độ đọc`
  - [x] 🔴 **Hai trong ba nhãn không có dấu tiếng Việt, nên `check-i18n.mjs` Kiểm A KHÔNG bắt được nếu chúng bị viết thẳng vào `.vue`.** Lượt review Story 1.5 đã ghi đúng lỗ này: *"`<button>Xem</button>`, `<button>Save</button>`, `Dong` đều **xanh**"*. Mọi nhãn đi qua `t('…')`, kể cả `Library` và `Workspace` — cổng không giữ chỗ này, người viết giữ
  - [x] Câu trạng thái theo UX-DR47: vô nhân xưng, nói việc, không xưng *"chúng tôi"*, không gọi người dùng là *"bạn"* — Kiểm D của `check-i18n.mjs` sẽ đỏ nếu phạm
  - [x] Ghi bản cuối của từng chuỗi + lý do chọn chữ vào §Completion Notes

- [x] **Task 9 — `scripts/check-commands.mjs`: cổng thứ tư, có mã thoát** (AC: 1, 2, 4, 6)
  - [x] Node thuần `.mjs`, khuôn theo `check-i18n.mjs`: `pass()` / `fail()` / `abort()`, **ngưỡng sàn** cho số tệp quét được, in tiêu đề từng kiểm, exit 1 khi `failures !== 0`
  - [x] ⚠️ **Node chứ không bash** — `npm run` trên Windows đi qua `cmd.exe` (Ice chốt 2026-08-03, `check-deps.mjs:22-24`)
  - [x] Năm phép kiểm — chi tiết ở §Khung `check-commands.mjs`:
    - **Kiểm A (AC1)** — mọi biểu thức `@click` / `v-on:click` trong `src/**/*.vue` là **đúng một** `dispatch('<id>')`
    - **Kiểm B (AC2)** — mọi id trong `dispatch('…')` và trong `register` khớp **đúng** `^[a-z0-9]+(\.[a-z0-9_]+)+$`, và **có mặt** trong bộ đã đăng ký
    - **Kiểm C (AC1, AC2, AC6)** — nạp `src/commands/registry.ts` thật, khẳng định **hành vi**: id trùng ném · id sai văn phạm ném · `dispatch` id lạ ném · `unbound()` trả đúng tập
    - **Kiểm D (AC3)** — nạp `src/commands/keys.ts` thật với `isMac: true` rồi `isMac: false`, khẳng định **cùng một hợp âm `Mod+1` khớp `metaKey` ở ca một và `ctrlKey` ở ca hai** *(đây là phép kiểm chặn §Trap 1)*
    - **Kiểm E (AC4)** — mọi `labelKey` của command đã đăng ký **có mặt trong `vi.json`**; mọi owner focus đã khai là **duy nhất và không rỗng**
  - [x] **NGƯỠNG SÀN, bắt buộc.** 0 tệp `.vue` quét được ⇒ `abort()`, không phải "đạt". Đây là bẫy đã đâm một lần ở `check-deps.mjs:15-17` (*"cây rỗng đọc thành sạch"*) và story trước phải dựng lại nó lần nữa
  - [x] Miễn trừ — nếu có — viết **ngay trong script**, mỗi mục kèm **một câu lý do**. Không miễn trừ im lặng bằng cách thu hẹp glob

- [x] **Task 10 — Chứng minh từng cổng bằng ĐỎ trước, XANH sau** (AC: 1, 2, 3, 4, 6)
  - [x] Với **mỗi** kiểm A–E: cố ý tạo một vi phạm → chạy → phải **đỏ đúng dòng đúng lý do** → gỡ → phải **xanh**
  - [x] Vi phạm mẫu: A — `@click="mode = 'library'"` nội tuyến; B — đổi một id thành `mode_library`; C — đăng ký hai lần cùng id; D — sửa `keys.ts` chỉ đọc `metaKey`; E — xoá một khoá nhãn khỏi `vi.json`
  - [x] **Ít nhất hai ca đối chứng ÂM**: một `@click` hợp lệ và một comment chứa chữ `dispatch(` — cả hai phải **exit 0**. Story 1.5 dựng bốn ca âm và chúng là nửa quan trọng bằng nửa kia
  - [x] Ghi bảng (kiểm · vi phạm · thông báo nhận được · mã thoát) vào §Debug Log References
  - [x] **Một cổng chưa từng đỏ là một cổng chưa được chứng minh** — tiền lệ Story 1.3 §Task 11, Story 1.4 §Task 3, Story 1.5 §Task 7

- [x] **Task 11 — Nghiệm thu tay phần DOM, ghi thành bảng** (AC: 3, 4, 5)
  - [x] `npm run tauri dev`, rồi chạy đúng kịch bản này và ghi kết quả từng dòng vào §Debug Log References:
    - `⌘1` `⌘2` `⌘3` *(macOS)* đổi chế độ; sau mỗi lần, `document.activeElement` **không** phải `body` — đọc bằng console
    - Bấm ba tab chế độ bằng chuột — cùng kết quả, và console **không** có `console.error` nào từ chốt focus
    - Trong Workspace, panel có tiêu điểm hiện **vạch dọc 2px** mép trái và tiêu đề đổi màu + đậm; panel kia **không** có
    - Đổi theme sang `dark` *(gọi `applyTheme('dark')` trong console)* — vạch vẫn nhìn thấy được trên `surface` tối
  - [x] ⚠️ **Ca Windows**: nếu không có máy Windows, ghi thẳng *"chưa đo"* và mở một mục trong `deferred-work.md`. **Đừng viết "tương đương" bằng suy luận** — đó đúng là thứ NFR14 tồn tại để chặn, và Story 1.1 → 1.3 đã có tiền lệ bàn giao một phép đo sang chỗ có runner
  - [x] Chụp lại: chốt focus có **thật sự kêu** không? Ép một ca xấu (`enter()` tới một owner chưa khai) và xác nhận `console.error` xuất hiện

- [x] **Task 12 — Gắn MỘT bước vào pipeline đã có** (AC: 1)
  - [x] `package.json` → thêm `"check:commands": "node scripts/check-commands.mjs"`, đúng khuôn bốn script đã có
  - [x] `.github/workflows/ci.yml` → thêm **một** bước `npm run check:commands` trong job `check` đã có, đặt **cạnh `check:i18n` (`:119-120`)**, tức **trước** `npm run build` (`:127-128`): nó chạy trong vài giây, không cần `dist/`, không cần phiên đồ hoạ
  - [x] **Không dựng workflow thứ hai** — AC4 của Story 1.3 cấm tường minh. Khối *"CHỖ MÓC CHO EPIC SAU"* ở cuối `ci.yml` là chỗ đã chừa sẵn; thêm một dòng vào sổ đó
  - [x] **Đừng sắp xếp lại các bước đã có.** Thêm một bước, không mổ lại job
  - [x] **Đừng đặt xuống cụm cuối** nơi `check:scope` / `check:scope:bundled` đứng — hai bước đó cần webview, bước này thì không

- [x] **Task 13 — Đóng sổ: README, `deferred-work.md`, doc-comment** (AC: tất cả)
  - [x] `src/commands/README.md` — hiện chỉ ghi *"Story sở hữu nội dung: 1.6"*. Thay bằng nội dung thật: hình dạng API, văn phạm id, cách thêm một command, vì sao `registry.ts` phải thuần, lệnh chạy cổng. Giữ nguyên khối cảnh báo *"Đừng nhầm với `src-tauri/src/commands/`"*
  - [x] `src/modes/README.md` và `src/panels/README.md` — ghi phần story này đã sở hữu và phần còn lại vẫn thuộc 1.14/1.16/1.17. Đừng xoá dòng *"Story sở hữu nội dung: 1.14"*, hãy làm rõ ranh giới
  - [x] `deferred-work.md` — mở mục cho: vế DOM của AC4 chưa có test tự động · ca Windows nếu chưa đo · `focus.next_panel` chưa có phím *(nhận ở Story 1.14/1.21)*
  - [x] 🔴 `deferred-work.md:38` ghi *"Hoãn tới **Story 1.6**, khi `#[tauri::command]` thật đầu tiên cho một đường thật để quan sát"` — **story này KHÔNG tạo `#[tauri::command]` nào** (§Ranh giới phạm vi). Sửa mục đó để trỏ sang story thật sự mở đường IPC đầu tiên, **kèm một câu lý do**. Đừng đánh dấu đã đóng, và đừng dựng một command giả để "làm cho đúng lời hứa cũ"

---

## Dev Notes

### Ranh giới phạm vi — đọc trước khi gõ dòng đầu tiên

| Story này **có** làm | Story này **KHÔNG** làm |
|---|---|
| `src/commands/**` — `registry.ts` · `keys.ts` · `focus.ts` · `index.ts` · README | Màn hình gán phím, đổi phím, lưu phím xuống đĩa — **Story 1.21** |
| Ba chế độ ở dạng **khung rỗng** + vỏ chuyển chế độ | `dockview`, dock/undock/tab/preset bố cục — **Story 1.14** |
| `PanelFrame.vue` — vỏ panel + hợp đồng thị giác tiêu điểm | Bốn panel thật và nội dung của chúng — **1.14 / 1.16 / 1.17** |
| Bốn command khởi động, hai trong đó là mã thật | Bất kỳ command nghiệp vụ nào *(tra cứu, xác nhận, dịch…)* — epic sau |
| Khoá `vi.json` cho **đúng** nhãn story này dựng | Từ vựng khoá cho tính năng chưa tồn tại — mỗi story tự thêm |
| `scripts/check-commands.mjs` + **một** bước trong `ci.yml` đã có | Bốn trạng thái rỗng của UX-DR31 — cần nội dung thật mới viết đúng |
| Nghiệm thu tay có bảng cho phần DOM | Một `#[tauri::command]` nào — xem mục ngay dưới |
| Ngưỡng bố cục màn hình hẹp? **Không** — UX-DR15 đóng ở 1.14 và 4.12 | Giữ ngữ cảnh thật khi chuyển chế độ (FR12) — Epic 2/5, nhưng cơ chế phải **không cản** nó |

**Không đụng tới:** `src-tauri/**` *(story này không có phần Rust — xem ngay dưới)* · `src-tauri/tauri.conf.json` · `Cargo.toml` · `package.json` *(trừ đúng một dòng `scripts`)* · `src/selftest/**` · `src/tokens/tokens.json` · `_bmad-output/planning-artifacts/**`.

**Không thêm một phụ thuộc nào.** Không `pinia`, không `vue-router`, không `mousetrap`/`hotkeys-js`, không `tauri-plugin-global-shortcut`, không `vitest`. Mỗi phụ thuộc mới phải rà tương thích GPLv3 **bằng cách mở tệp giấy phép trong nguồn đã tải** và vào bảng Stack **trước khi** thêm (NFR15, `ARCHITECTURE-SPINE.md#Consistency Conventions`) — đó là quyết định của Ice, không phải hệ quả phụ của story này. `check-deps.mjs` sẽ đỏ.

---

### 🔴 Vì sao story này KHÔNG có phần Rust — và một lời hứa cũ phải được sửa lại

`deferred-work.md:38` viết: *"Hoãn tới **Story 1.6**, khi `#[tauri::command]` thật đầu tiên cho một đường thật để quan sát."* Câu đó được viết lúc chưa ai đọc kỹ AC của Story 1.6. Đọc rồi thì thấy: **không AC nào của story này cần Rust.**

Chuyển chế độ, tiêu điểm bàn phím, bố cục panel — cả ba là **state UI**, và AD-1 nói thẳng đó là phần frontend được phép sở hữu: *"frontend chỉ render và giữ state UI (focus, cuộn, vùng chọn, bố cục panel)"*. Một `#[tauri::command]` cho việc đổi chế độ sẽ là quy tắc nghiệp vụ giả đặt sai chỗ, cộng một vòng IPC cho một thao tác phải mượt.

**Đừng dựng một `#[tauri::command]` chỉ để đóng mục `deferred-work.md:38`.** Story 1.5 đã từ chối đúng cám dỗ này với ba lý do còn nguyên giá trị: nó là mã sản phẩm không ai gọi; chạy nó cần webview và một lượt biên dịch profile `dev` riêng *(đắt nhất trên macOS, hệ số ×10)*; và vòng chạy thật đến **miễn phí** ở story đầu tiên có nhu cầu IPC thật.

**Việc phải làm:** Task 13 sửa mục `:38` trỏ sang story đó *(ứng cử viên gần nhất: **1.8** phân giải cấu hình hai tầng, hoặc **1.9/1.11** khi đường tra cứu cần Rust)* kèm một câu lý do. Đây là loại nợ nếu để trôi thì ba story nữa sẽ không ai truy được nguồn gốc.

---

### Trạng thái repo hiện tại — số, không phải mô tả

Đọc lúc dựng story, `HEAD = b482dc1`:

| | |
|---|---|
| `src/commands/` | **rỗng** — chỉ `.gitkeep` + `README.md` giao việc cho story này |
| `src/modes/` · `src/panels/` · `src/layout/` | **rỗng** — chỉ `.gitkeep` + `README.md` |
| Số tệp `.vue` trong `src/**` | **1** — `App.vue` *(sàn Kiểm A của `check-i18n` đặt ở 1)* |
| Số tệp `.ts` trong `src/**` | **8** |
| Số tệp `.rs` trong `src-tauri/src/**` | **17** *(sàn Kiểm A của `check-i18n` đặt ở 14)* |
| `src/i18n/vi.json` | **2 khoá** — `err.unknown`, `err.io.read_failed` |
| Cổng đã có | `check:deps` · `check:tokens` · `check:i18n` · `check:scope` · `check:scope:bundled` |
| Bước CI trong job `check` | `check:deps` `:93` → `check:tokens` `:106` → `check:i18n` `:120` → `npm run build` `:128` → `cargo test` `:134` → build/đo → `check:scope:bundled` `:398` → `check:scope` `:417` |
| Node trên máy Ice / CI | **v22.22.2** / `node-version: '22'` |
| Alias `@` trong import | **KHÔNG có** — `vite.config.ts` và `tsconfig.json` đều không khai `alias`/`paths`. Dùng đường dẫn tương đối *(lượt review Story 1.5 đã bắt một README viết `@/i18n` không chạy được)* |
| `tsconfig.json` | `strict` · `noUnusedLocals` · `noUnusedParameters` · `verbatimModuleSyntax` — nhập kiểu phải viết `import type` |

**Bốn lệnh kế thừa** *(chép đúng, đừng phát minh lại)*:

```bash
npm run check:deps                                 # 13 phép kiểm — cây phụ thuộc
npm run check:tokens                               # 7 phép kiểm — màu/cỡ chữ/tương phản/elevation
npm run check:i18n                                 # 5 phép kiểm — chuỗi giao diện + hình dạng lỗi
npm run build                                      # vue-tsc ×2 + vite build
cargo test --manifest-path src-tauri/Cargo.toml    # CẦN `dist/` tồn tại
```

⚠️ `cargo test` **cần `dist/` tồn tại** — `generate_context!` nhúng frontend lúc biên dịch. Chạy `npm run build` trước.

---

### Bốn cái bẫy — ba trong bốn cho ra một lượt CI XANH với kết quả vô nghĩa

**1. 🔴 `event.metaKey` — sản phẩm không chuyển chế độ được trên Windows, CI vẫn xanh.**
`⌘1` trong AC là ký hiệu macOS của một phím **trừu tượng**. Trên Windows nó là `Ctrl+1`. Không có test nào chạm tầng bàn phím ở dự án này, và CI của Story 1.3 chỉ `cargo test` + build — nên một `if (e.metaKey && e.key === '1')` **đi qua cả hai nền tảng của CI** rồi hỏng ở tay người dùng Windows. Đây là vi phạm NFR14 hạng nặng nhất có thể lọt hôm nay. Lời giải: hợp âm trung lập (`Mod+1`), nhận biết nền tảng **tiêm được**, và **Kiểm D của cổng lái cả hai ca** — đó là toàn bộ lý do Kiểm D tồn tại.

**2. 🔴 Đăng ký trùng id ghi đè im lặng.**
`map.set(id, spec)` là dòng mã tự nhiên nhất để viết, và nó **chính xác là thứ AC2 cấm**. AD-34 gọi tên hố này: hai giai đoạn cách nhau nhiều tháng đăng ký trùng id, cái sau ghi đè cái trước, không lỗi nào được ném, và biểu hiện là *"phím tắt X bỗng làm việc Y"* — không ai lần ra được. Ném ở `register()`, và Kiểm C chứng minh nó ném.

**3. 🔴 Nhãn tiếng Anh viết thẳng trong `.vue` — cổng `check-i18n` KHÔNG bắt được.**
Hai trong ba nhãn chế độ là `Library` và `Workspace`, không dấu. `check-i18n.mjs` Kiểm A quét **ký tự có dấu tiếng Việt**; lượt review Story 1.5 đã nghiệm thu và ghi thẳng giới hạn: *"`<button>Xem</button>`, `<button>Save</button>`, `Dong`, `Trang` đều **xanh**"*, và `deferred-work.md` hạ mục đó xuống 🟡 **ĐÓNG MỘT PHẦN** vì chính lý do này. Story này là **story đầu tiên trong dự án dựng nhãn giao diện thật**, nên nó là story đầu tiên đứng đúng trên lỗ đó. Mọi nhãn qua `t('…')`, không ngoại lệ.

**4. ⚠️ `outline: none` để "tuân thủ" AC5 — phá NFR17 mà mọi cổng vẫn xanh.**
AC5 nói *"không dùng viền bao quanh để báo tiêu điểm"* — nói về **panel**. Đọc nó thành `*:focus { outline: none }` là xoá luôn chỉ báo tiêu điểm của mọi nút, ô nhập, tab về sau, tức phá đúng nửa còn lại của NFR17 (*"trạng thái focus luôn nhìn thấy rõ"*). Không cổng nào bắt được: `check-tokens.mjs` canh màu, cỡ chữ, tương phản, opacity và elevation — **không** canh focus ring. Luật đúng: `outline: none` **chỉ** áp cho phần tử `tabindex="-1"` nhận focus lập trình *(gốc chế độ, gốc panel)*, vì chúng không phải điều khiển tương tác và đã có vạch dọc làm chỉ báo. Viết lý do ngay cạnh dòng CSS đó.

---

### Quyết định thiết kế — đã chốt, không phải lựa chọn của dev

#### #1 — `registry.ts` là hàm thuần, không `import` gì

Cùng lý lẽ, cùng cơ chế, cùng bằng chứng như `src/i18n/resolve.ts` của Story 1.5 — và đường đó **đã chạy thật trong CI**, không phải một giả thuyết:

- Dự án **không có bộ chạy test frontend**, và thêm `vitest` là thêm một phụ thuộc phải rà GPLv3 trước (NFR15). Đó là quyết định của Ice.
- **Node ≥ 22.18 bóc kiểu TypeScript mặc định**, nên `check-commands.mjs` `import()` thẳng được `registry.ts`. Máy Ice v22.22.2 ✅, CI `node-version: '22'` ✅.
- Điều kiện: cú pháp **"erasable-only"** — không `enum`, không `namespace`, không parameter property. `type` / `interface` / annotation đều được.
- ⚠️ `import()` thất bại ⇒ `abort()` nêu rõ *"Kiểm C KHÔNG chạy được"* và **exit 1**. Không bỏ qua rồi exit 0 — `check-deps.mjs:60-66`: *"Lỗi hạ tầng ≠ phép kiểm đỏ. Dừng ngay, đừng báo cáo một kết quả không có thật."*

Hình dạng API *(hình dạng, không phải bản chép — dev viết bản cuối)*:

```ts
export type CommandId = string
export type CommandSpec = {
  id: CommandId          // ^[a-z0-9]+(\.[a-z0-9_]+)+$ — cùng văn phạm khoá vi.json
  labelKey: string       // 'command.' + id
  run: () => void
  keys?: readonly string[]   // hợp âm trung lập: 'Mod+1'. Vắng/rỗng ⇒ unbound()
}

export type Registry = {
  register(spec: CommandSpec): void   // ném khi trùng id / sai văn phạm / labelKey rỗng
  has(id: CommandId): boolean
  dispatch(id: CommandId): void       // ném khi id chưa đăng ký
  list(): readonly CommandSpec[]      // thứ tự ĐĂNG KÝ, ổn định
  unbound(): readonly CommandSpec[]   // AC6
}

export function createRegistry(): Registry
```

#### #2 — Hợp âm viết trung lập nền tảng, nhận biết nền tảng tiêm được

`'Mod+1'` là dữ liệu; `Mod → ⌘ | Ctrl` là một phép phân giải chạy **một lần** lúc dựng keymap. Hai hệ quả bắt buộc:

- `createKeymap(registry, { isMac })` — `isMac` là **tham số**, không phải một lời gọi `navigator.platform` ở tầng module. Không tiêm được thì Kiểm D không lái được hai ca, và §Trap 1 quay lại nguyên vẹn.
- Chỗ gọi thật ở `src/commands/index.ts` mới đọc nền tảng. Dùng `navigator.userAgentData?.platform ?? navigator.platform` và **ghi một comment** rằng cả hai đều là API đang trôi — đây là chỗ duy nhất phụ thuộc vào nó.

#### #3 — Xung đột đã phát hiện: `⌘1` `⌘2` trong mockup là **preset bố cục**, không phải chế độ

`mockups/key-screen-workspace.html:89` vẽ thanh tiêu đề với `Bố cục 2×2 nguồn–đích ⌘1 · 4 cột ⌘2`. Nhưng AC3 của story này và UX-DR34 đều nói `⌘1` `⌘2` `⌘3` là **ba chế độ**, và `EXPERIENCE.md:49` viết lại đúng như vậy.

**Phân xử: chế độ thắng.** Ba lý do: AC của epic là hợp đồng nghiệm thu, mockup là bản phác; UX-DR34 là một mục UX-DR đánh số, dòng trong mockup thì không; và ba chế độ là **cấu trúc toàn ứng dụng** (AD-24) còn preset bố cục chỉ sống trong Workspace.

**Việc phải làm:** ghi xung đột này vào §Completion Notes và mở một mục `deferred-work.md` nói rõ **Story 1.14 phải chọn phím khác cho preset bố cục (FR18)**. Đừng sửa mockup — dev không sửa tài liệu quy hoạch *(tiền lệ quyết định #3 của Ice ở Story 1.3)*.

#### #4 — `labelKey = 'command.' + id`, không dùng thẳng `id` làm khoá

AD-34 nói command id *"cùng hình dạng với khoá chuỗi i18n"* — **cùng hình dạng, không cùng danh mục**. Hai không gian tên khác nhau: một cái định danh thao tác, một cái định danh chuỗi. Dùng chung một khoá trần thì ngày `vi.json` cần **hai** chuỗi cho một command *(nhãn ngắn trên tab, mô tả dài ở màn hình gán phím của Story 1.21)*, không còn chỗ đặt cái thứ hai mà không phá quy ước.

Tiền tố `command.` giải cả hai: `command.mode.library` cho nhãn hôm nay, `command.mode.library.hint` cho mô tả sau này, và một lượt grep `"command."` trong `vi.json` liệt kê đúng bộ nhãn thao tác. Kiểm E cưỡng chế mọi `labelKey` có mặt trong `vi.json`.

#### #5 — `focus.next_panel` cố ý **không gán phím**

Ba lý do độc lập, mỗi lý do đủ đứng một mình:

1. **Bốn panel chưa tồn tại.** Phím xoay vòng panel chỉ có nghĩa khi biết vòng gồm những gì và theo thứ tự nào — đó là Story 1.14 với `dockview` và UX-DR13 (lưới 2×2).
2. **Mọi phím ứng cử đều đang có chủ hoặc sắp có chủ.** `Tab` là thứ tự tiêu điểm của trình duyệt; `⌘1..3` đã là chế độ; `⌘⇧↵` là đưa bản dịch AI sang (UX-DR35); `⌘M` `⌘/` là gộp/tách (UX-DR32). Đặt bừa một phím hôm nay là tạo một mục phải gỡ ở Story 1.21.
3. **AC6 cần một phần tử thật để chứng minh.** `unbound()` trả mảng rỗng thì nhánh có nghĩa của nó không bao giờ chạy, và Story 1.21 sẽ phát hiện nó hỏng khi đã có 40 command.

**Nhưng handler phải chạy thật** — nó xoay vòng focus giữa các panel đã khai. Một command rỗng đăng ký cho đủ số là đúng thứ story này tồn tại để chặn.

#### #6 — `<KeepAlive>` cho ba chế độ, từ hôm nay

UX-DR34 và FR12 hứa chuyển chế độ **giữ ngữ cảnh**. Hôm nay chưa có ngữ cảnh nào, nên `v-if` và `<KeepAlive>` cho **kết quả quan sát được y hệt** và mọi cổng đều xanh với cả hai. Khác biệt hiện ra ở Epic 2, khi Editor mang văn bản đang gõ và vị trí cuộn: lúc đó `v-if` sẽ huỷ component và mất chúng, và người sửa sẽ phải mổ lại vỏ chế độ mà không có gì nối được lỗi đó về story này. Chọn đúng ngay bây giờ tốn **một cặp thẻ**.

---

### Khung `check-commands.mjs` — hình dạng, không phải bản chép

Khuôn chung lấy nguyên từ `scripts/check-i18n.mjs`: `pass()` / `fail()` đếm lỗi, `abort()` cho lỗi hạ tầng, in tiêu đề từng kiểm, exit 1 nếu `failures !== 0`, in số tệp đã quét ở mỗi lượt.

**Kiểm A — `@click` chỉ được là `dispatch('<id>')`.**
Quét `src/**/*.vue`, bóc mọi thuộc tính `@click` / `v-on:click` / `@click.<modifier>`. Giá trị phải khớp `^\s*dispatch\(\s*'([a-z0-9.]+)'\s*\)\s*$` *(hoặc nháy kép)*. Mọi thứ khác — gán biến, gọi hàm khác, biểu thức nội tuyến, `$emit` — là **FAIL** kèm `đường-dẫn:dòng:cột` và trích 60 ký tự quanh chỗ đó.
⚠️ **Bóc thuộc tính phải có trạng thái**, không `replace` ngây thơ: lượt review Story 1.5 đã dựng lại được ba lỗ thủng của một bộ quét không trạng thái *(char literal chứa `"`, regex literal, `<!--` trong giá trị attribute)*. Chép cách quét của `check-i18n.mjs` thay vì viết lại từ đầu.
⚠️ Chỉ `@click` hôm nay — `@keydown`, `@input` và bạn bè **không** thuộc luật này *(chúng không phải "thao tác" theo nghĩa AD-34)*. Ghi giới hạn đó **trong script**, ngay cạnh chỗ cưỡng chế.

**Kiểm B — văn phạm id và id phải tồn tại.**
Gom mọi id từ `dispatch('…')` trong `.vue`/`.ts` và mọi id trong bộ đăng ký; khớp `^[a-z0-9]+(\.[a-z0-9_]+)+$` — **chép đúng biểu thức của `check-i18n.mjs` Kiểm B**, đừng viết lại một biến thể. Một id được `dispatch` mà không có trong bộ đăng ký ⇒ FAIL *(đây là lưới bắt lỗi gõ sai, thứ mà `dispatch` ném lúc chạy chỉ bắt được khi có người bấm đúng nút đó)*.

**Kiểm C — hành vi registry, nạp tệp thật.**
`await import('../src/commands/registry.ts')`, rồi khẳng định từng mệnh đề, mỗi cái một dòng `pass`/`fail` riêng: đăng ký trùng id **ném** · id sai văn phạm **ném** · `labelKey` rỗng **ném** · `dispatch` id lạ **ném** · `unbound()` trả **đúng** tập command thiếu `keys` · `list()` giữ **thứ tự đăng ký**.
⚠️ Mọi lời gọi đi qua một helper bắt ném thành FAIL **có tên**; `abort()` chỉ dành cho ca `import()` gãy. Đây là khiếm khuyết mà ca E của Story 1.5 tìm ra và đã sửa — đừng dựng lại nó.

**Kiểm D — hai nền tảng, cùng một hợp âm.**
`await import('../src/commands/keys.ts')`, dựng keymap hai lần với `isMac: true` và `isMac: false`, đẩy vào một object giả hình dạng `KeyboardEvent` *(`{ code, metaKey, ctrlKey, shiftKey, altKey, preventDefault(){} }`)*. Khẳng định bốn mệnh đề: `Mod+1` khớp khi `metaKey` **và** `isMac` · **không** khớp khi `ctrlKey` và `isMac` · khớp khi `ctrlKey` và **không** `isMac` · **không** khớp khi `metaKey` và không `isMac`.
🔴 **Đây là phép kiểm duy nhất trong toàn bộ dự án đứng giữa §Trap 1 và người dùng Windows.** Đừng rút gọn nó xuống một ca.

**Kiểm E — nhãn và owner focus.**
Mọi `labelKey` của command đã đăng ký **có mặt** trong `vi.json` *(nạp `vi.json` thật, không catalog giả — đây là mục mà lượt review Story 1.5 đã bắt và sửa ở Kiểm E của cổng trước)*. Mọi owner focus đã khai: không rỗng, không trùng.

**Ngưỡng sàn.** `vueFiles.length < 1` ⇒ `abort()`. Nếu đặt thêm sàn cho số command, ghi con số thật hôm nay *(4)* và một câu lý do — đừng để một cây rỗng đọc thành sạch.

---

### Testing standards

- **Không có bộ chạy test frontend** và **không được thêm** — mọi cưỡng chế frontend đi qua một cổng `.mjs` có mã thoát, đúng khuôn bốn cổng đang chạy.
- **Test Rust** đặt ở `src-tauri/tests/` *(integration, `use auratranslate_lib::…`)*. Story này **không thêm test Rust** — không có phần Rust.
- **Nghiệm thu đỏ-rồi-xanh là bắt buộc**, kèm ít nhất hai ca **đối chứng âm**. Tiền lệ: Story 1.3 §Task 11 · Story 1.4 §Task 3 (28 ca) · Story 1.5 §Task 7 (16 ca cổng + 5 ca test Rust).
- **Phần DOM nghiệm thu bằng tay, có bảng ghi lại**, và **giới hạn ghi thẳng vào `deferred-work.md`** — không đánh dấu đạt bằng suy luận.
- **Lệnh chạy trước khi báo xong:** `npm run check:commands` · `check:i18n` · `check:tokens` · `check:deps` · `npm run build` · `cargo test`. Cả sáu phải exit 0.

---

### Bàn giao từ các story trước — thứ ảnh hưởng trực tiếp tới story này

**Từ Story 1.4 (token):**
- Biến CSS đã có: `--color-<token>` · `--family-<họ>` · `--space-<token>` · `--radius-<token>` · bảy biến typography mỗi token (`--font-` `--leading-` `--weight-` `--style-` `--tracking-` `--synthesis-` `--face-`). Dùng thẳng, đừng khai lại.
- Cần cho story này: `--color-primary` · `--color-on-surface-variant` · `--space-titlebar-height` (38px) · `--space-head-height` (34px) · `--space-panel-inline` · `--radius-default` (3px) · `--font-ui-md` / `--leading-ui-md` / `--face-ui-md`.
- Kiểm F cấm `box-shadow` · `text-shadow` · `drop-shadow` · gradient **không miễn trừ**; `z-index` có miễn trừ **có tên**.
- ⚠️ `deferred-work.md` mở sẵn một mục: *"`body` chạy ở giãn dòng 1.5 và không phép kiểm nào canh được"* — lưới thật là **lượt rà soát khi Story 1.14/1.16/1.17 dựng panel**. `PanelFrame` của story này là bề mặt chữ đầu tiên sau `App.vue`; nếu thân panel về sau chở chữ chạy thành đoạn thì nó phải khai token `read-*` của chính nó.

**Từ Story 1.5 (i18n):**
- `t(key, params?)` và `tError(err, params?)` từ `./i18n` *(đường dẫn tương đối — **không có alias `@`**)*. `vi.json` là object **PHẲNG**, khoá chấm, không lồng object.
- Khoá thiếu ⇒ hiện khoá nguyên văn + `console.warn` một lần, **không sập**. Nghĩa là một `labelKey` gõ sai sẽ hiện `command.mode.libary` ra tab — Kiểm E là thứ bắt nó **trước** khi tới màn hình.
- ⚠️ `check-i18n.mjs` Kiểm A sẽ quét mọi `.vue` mới của story này. Comment tiếng Việt **không** phải vi phạm; chuỗi ở vị trí mã thì là.

**Từ Story 1.3 (CI):** job `check` là **workflow duy nhất**. Khối *"CHỖ MÓC CHO EPIC SAU"* ở cuối `ci.yml` là sổ ghi các luật gắn thêm — thêm một dòng, đừng dựng pipeline thứ hai.

**Từ Story 1.2 (scaffold):** `App.vue` mang khối self-check phạm vi asset protocol chạy sau cờ `VITE_SCOPE_SELFTEST=1`, và `scripts/check-scope*.mjs` **đọc dòng `VERDICT:`** trong `src/selftest/fallbackReport.ts`. Sửa `App.vue` mà chạm vào khối đó là làm mù hai cổng của Story 1.2/1.3.

---

### Thông tin kỹ thuật cần dùng đúng phiên bản

Bảng Stack **ghim cứng** và story này **không thêm gì**: Vue **3.5.40** · TypeScript **5.9.3** · Vite **8.2.0** · `@tauri-apps/api` **2.11.1** · `dockview-vue` **7.0.4** *(chưa dùng — Story 1.14)*. `check-deps.mjs` sẽ đỏ nếu cây phụ thuộc đổi.

Ba điểm của Vue 3.5 đáng dùng ở story này, tất cả đã có sẵn trong phiên bản đã ghim, **không cần thêm gì**:

- `useTemplateRef('name')` — lấy tham chiếu phần tử DOM sạch hơn `ref` + trùng tên biến. Đúng thứ `focus.ts` cần để `resolve()` trả về phần tử thật.
- `onWatcherCleanup()` — dọn listener bàn phím khi component tháo, không phải tự nhớ gọi hàm gỡ.
- `<KeepAlive>` — không phải API mới *(có từ Vue 2)*, nêu ở đây chỉ vì §Quyết định thiết kế #6 bắt buộc dùng nó và một lượt cài đặt vội sẽ với tay tới `v-if`.

⚠️ `verbatimModuleSyntax: true` ⇒ mọi nhập kiểu viết `import type { … }`. `noUnusedLocals` + `noUnusedParameters` ⇒ `vue-tsc` sẽ đỏ ở `npm run build` với một biến thừa; đó là hàng rào, không phải phiền nhiễu.

---

### References

- `_bmad-output/planning-artifacts/epics.md#Story 1.6` — sáu AC nguyên văn *(`:1213-1251`)*
- `_bmad-output/planning-artifacts/epics.md#Story 1.14` · `#Story 1.21` — hai ranh giới liền kề *(`:1533`, `:1835`)*
- `ARCHITECTURE-SPINE.md#AD-34` — sàn khả năng tiếp cận là cấu trúc, ba mệnh đề + luật khoá chấm
- `ARCHITECTURE-SPINE.md#AD-24` — một cửa sổ OS, ba chế độ; Review Mode là bố cục không phải cửa sổ
- `ARCHITECTURE-SPINE.md#AD-1` — frontend giữ state UI: focus, cuộn, vùng chọn, bố cục panel
- `ARCHITECTURE-SPINE.md#Consistency Conventions` — *"Thao tác giao diện: luôn đăng ký trong `CommandRegistry` rồi mới bind"*; `PascalCase.vue`; ánh xạ thuật ngữ `ReadingMode` / `ReviewMode`
- `ARCHITECTURE-SPINE.md#Cây nguồn` — `src/commands/` là nhà của `CommandRegistry`
- `DESIGN.md:139` `components.panel-focus-rule: { width: 2px, color: primary }` · `:159` *(ba việc duy nhất dùng `primary`)* · `:181` *(bảng token)* · `:354, :358` *(hình dạng vạch dọc; panel có tiêu điểm)*
- `EXPERIENCE.md:49` *(ba chế độ ngang hàng, `⌘1` `⌘2` `⌘3`, giữ ngữ cảnh)* · `:160-161` *(hai mệnh đề AD-34)*
- `epics.md` UX-DR7 `:507` · UX-DR8 `:509` · UX-DR9 `:511` · UX-DR17 `:533` · UX-DR34 `:573` · UX-DR46 `:601` · UX-DR47 `:605`
- `prd.md:202-204` — thuật ngữ chốt: `Library` *(không dịch)* · `Workspace` · `Chế độ đọc`
- `prd.md` NFR17 `epics.md:366` — sàn khả năng tiếp cận, nghiệm thu bằng một vòng dịch không chạm chuột
- `mockups/key-screen-workspace.html:31,34` — `::before` 2px + tiêu đề `primary` 600; `:89` — **xung đột phím, xem §Quyết định thiết kế #3**
- `src/commands/README.md` · `src/modes/README.md` · `src/panels/README.md` — ba README giao việc
- `deferred-work.md:38` — lời hứa cũ phải sửa lại *(§Vì sao story này KHÔNG có phần Rust)*
- Story 1.5 §Khung `check-i18n.mjs` · §Bốn thứ sẽ hỏng im lặng · §Review Findings — khuôn cổng và bốn lỗ thủng của một bộ quét không trạng thái

---

### Câu hỏi cho Ice — đã có mặc định, không chặn

1. **`src/commands/focus.ts` có phải chỗ đúng cho sổ điểm vào focus không?** Cây nguồn ở `ARCHITECTURE-SPINE.md` liệt kê sáu thư mục frontend và không thư mục nào tên `focus/`. Đặt nó cạnh `CommandRegistry` là đọc AD-34 như một khối *(§1 thao tác và §2 focus là hai mệnh đề của cùng một AD)*. Hai chỗ khác đều tệ hơn: `src/modes/` thì panel cũng khai điểm vào, `src/layout/` thì thuộc `dockview` của Story 1.14.
   → **Mặc định: `src/commands/focus.ts`**, và `src/commands/README.md` ghi rõ nó chứa cả hai nửa của AD-34. Không thêm thư mục thứ bảy vào cây nguồn.
2. **`WorkspaceMode` dựng hai `PanelFrame` — có phải dựng trước Story 1.14 không?** AC5 nói *"một panel có tiêu điểm"*, và không có panel nào thì AC5 nghiệm thu bằng suy luận.
   → **Mặc định: đúng hai**, đúng cặp `Nguyên văn | Bản dịch` mà UX-DR15 nói không bao giờ nhường, thân để trống, ghi rõ trong README rằng Story 1.14 thay chỗ chúng bằng bốn panel trong `dockview`.
3. **Xung đột `⌘1` `⌘2` giữa mockup và UX-DR34** — xem §Quyết định thiết kế #3.
   → **Mặc định: chế độ thắng**, mở mục `deferred-work.md` giao Story 1.14 chọn phím khác cho preset bố cục. Dev không sửa mockup.

---

## Dev Agent Record

### Agent Model Used

Amelia (dev agent) · Claude Opus 5 (`claude-opus-5`) · 2026-08-04

### Debug Log References

#### Task 1 — đường cơ sở trên `HEAD = b482dc1`

Trạng thái repo khớp **đúng** §Trạng thái repo hiện tại, không một chỗ lệch nào: 1 tệp `.vue` · 8 tệp `.ts` · 17 tệp `.rs` · `vi.json` 2 khoá · Node v22.22.2.

| Lệnh | Mã thoát | Ghi chú |
|---|---|---|
| `npm run check:deps` | 0 | 13 phép kiểm · 326 crate Rust, 104 gói npm |
| `npm run check:tokens` | 0 | 7 phép kiểm · 11 tệp (8 component) · 17 khai báo CSS |
| `npm run check:i18n` | 0 | 5 phép kiểm · 18 `.rs` + 1 `.vue` · 2 khoá |
| `npm run build` | 0 | vue-tsc ×2 + vite · 23 module |

#### Task 10 — nghiệm thu cổng: ĐỎ trước, XANH sau

**28 ca, 28 đúng kỳ vọng**, mỗi ca chạy trên cây nguồn thật rồi khôi phục nguyên trạng. Cột cuối là dòng chẩn đoán **đầu tiên** cổng in ra.

| | Ca | exit mong đợi | exit nhận | Chẩn đoán |
|---|---|---|---|---|
| ✅ | A1 · `@click` gán biến nội tuyến | 1 | 1 | `src/App.vue:84:19 — @click không phải một lời gọi dispatch('<id>')` |
| ✅ | A2 · `@click` gọi một hàm khác | 1 | 1 | `src/App.vue:92:19 — @click không phải một lời gọi dispatch('<id>')` |
| ✅ | A3 · `@click.prevent` cũng bị canh | 1 | 1 | `src/App.vue:100:27 — @click.prevent không phải một lời gọi dispatch('<id>')` |
| ✅ | B1 · id sai văn phạm (`mode_library`) | 1 | 1 | `id mode_library sai văn phạm, phải khớp ^[a-z0-9]+(\.[a-z0-9_]+)+$` |
| ✅ | B2 · id gõ sai, chưa đăng ký (`mode.libary`) | 1 | 1 | `dispatch('mode.libary') gọi một command CHƯA ĐĂNG KÝ` |
| ✅ | C1 · registry không còn ném khi id trùng | 1 | 1 | `id trùng ⇒ ném (AC2) — KHÔNG ném` |
| ✅ | C2 · registry không còn ném khi `dispatch` id lạ | 1 | 1 | `dispatch một id LẠ ⇒ ném — KHÔNG ném` |
| ✅ | C3 · `unbound()` bỏ sót ca `keys: []` | 1 | 1 | `unbound() trả ĐÚNG tập — nhận ``, phải là demo.keys_rong · focus.next_panel` |
| ✅ | C4 · `list()` trả tham chiếu dùng chung | 1 | 1 | `list() trả một mảng MỚI ở mỗi lời gọi — nhận false` |
| ✅ | F1 · `focus.ts` không còn ném khi owner trùng | 1 | 1 | `owner khai TRÙNG ⇒ ném — KHÔNG ném` |
| ✅ | F2 · `enter()` không gọi `el.focus()` | 1 | 1 | `phần tử đã thật sự nhận focus() — nhận 0, phải là 1` |
| ✅ | F3 · `next()` không lọc theo tiền tố | 1 | 1 | `next() quay vòng về panel đầu — nhận panel.editor` |
| ✅ | F4 · `enter()` owner lạ không kêu | 1 | 1 | `enter() owner lạ ⇒ ghi console.error nêu đích danh owner — nhận false` |
| ✅ | **D1 · `keys.ts` chỉ đọc `metaKey` (§Trap 1)** | 1 | 1 | `[Windows] Mod+1 KHỚP khi ctrlKey — nhận false, phải là true` |
| ✅ | D2 · khớp bằng `event.key` thay vì `event.code` | 1 | 1 | `[macOS] Mod+1 KHỚP khi metaKey — nhận false` |
| ✅ | D3 · luật vùng gõ bị gỡ | 1 | 1 | `hợp âm TRẦN (B) KHÔNG khớp khi focus trong vùng gõ — nhận true` |
| ✅ | E1 · xoá một khoá nhãn khỏi `vi.json` | 1 | 1 | `command.mode.reading KHÔNG có trong src/i18n/vi.json` |
| ✅ | E2 · `labelKey` không theo quy ước `command.` | 1 | 1 | `labelKey là mode.library, quy ước là command.mode.library` |
| ✅ | E3 · owner dùng trong `.vue`, thiếu ở `FOCUS_OWNERS` | 1 | 1 | `WorkspaceMode.vue:41:19 — owner panel.editor KHÔNG có trong FOCUS_OWNERS` |
| ✅ | E4 · owner khai nhưng không ai dùng | 1 | 1 | `owner panel.khong.ai.dung khai … nhưng KHÔNG chế độ/panel nào dùng` |
| ✅ | E5 · một chế độ quên `declareFocus` | 1 | 1 | `owner mode.reading được dùng nhưng KHÔNG chỗ nào gọi declareFocus()` |
| ✅ | E6 · `focus.next_panel` được gán phím | 1 | 1 | `unbound() trả MẢNG RỖNG — AC6 chưa được chứng minh` |
| ✅ | **N1** · thêm một `@click` HỢP LỆ | 0 | 0 | — |
| ✅ | **N2** · comment JS chứa `dispatch('khong.co.that')` | 0 | 0 | — |
| ✅ | **N3** · comment template chứa một `@click` vi phạm | 0 | 0 | — |
| ✅ | **N4** · regex literal `/^https?:\/\//` ngay trước một `dispatch` | 0 | 0 | — |
| ✅ | **N5** · chuỗi `'https://…'` + `//` không mở comment giả | 0 | 0 | — |
| ✅ | **N6** · attribute chứa `>` không cắt sớm thẻ | 0 | 0 | — |

> **Hai ca đầu tiên chạy đã ĐỎ đúng chỗ cổng còn yếu, và cổng được sửa — không phải ca.**
> - **C4** ban đầu cho exit 0: phép so `list().length` không phân biệt được một cài đặt trả về cùng một mảng đệm dựng lại mỗi lần. Thêm phép so **định danh** (`list() !== list()`).
> - **E5** ban đầu cho exit 0: cổng gom `declareFocus` · `enterFocus` · `releaseFocus` vào một rổ "đã dùng", nên một chế độ quên **khai** mà vẫn còn hai lời gọi kia thì xanh — đúng ca dẫn thẳng tới focus rơi về `body`. Tách thành hai rổ **khai báo** và **tham chiếu**; chiều ngược lại nay đòi `declareFocus()`.
>
> Lượt chạy đầu tiên của chính cổng cũng bắt được một lỗi thật trong sản phẩm: `isTypingZone` dùng `instanceof HTMLElement`, thứ trả `false` qua ranh giới realm (một phần tử trong `<iframe>`) và tắt luật vùng gõ ở đúng chỗ nó cần bật nhất. Đổi sang đọc hình dạng.

#### Task 11 — nghiệm thu DOM trên webview thật

🔴 **Đọc §Giới hạn ngay dưới bảng trước khi đọc bảng.**

| Kịch bản | Kết quả đo | AC |
|---|---|---|
| Khởi động | `mode = Library` · `activeElement = SECTION.mode` · `isBody = false` | AC3, AC4 |
| `⌘2` | `Workspace` · hai panel `Nguyên văn` + `Bản dịch` · `isBody = false` | AC3, AC4 |
| `⌘3` | `Chế độ đọc` · `isBody = false` | AC3, AC4 |
| `⌘1` | `Library` · `isBody = false` | AC3, AC4 |
| `⌘1/2/3` khớp | `event.defaultPrevented === true` | AC3 |
| **`Ctrl+1` trên macOS** | **không khớp**, `defaultPrevented === false` — không đụng vào event | AC3, §Trap 1 |
| Bấm **chuột** vào ba tab chế độ | đổi chế độ như phím; console **không** có `console.error` nào từ chốt focus | AC1, AC4 |
| Panel có tiêu điểm *(theme sáng)* | `::before` `width 2px` · `background rgb(47,93,99)` = `#2f5d63` = `--color-primary` · `left 0px`; tiêu đề `color #2f5d63`, `font-weight 600` | **AC5** |
| Panel **không** có tiêu điểm | không `::before` (`content: none`); tiêu đề `rgb(107,100,89)` = `#6b6459` = `on-surface-variant`, `font-weight 400` | **AC5** |
| Panel có tiêu điểm *(theme tối, `applyTheme('dark')`)* | vạch `#7fb3ba` nhìn rõ trên `surface #26241f`; tiêu đề `primary` đậm — ảnh chụp đối chiếu hai theme | **AC5** |
| Phân tách panel, sáng → tối | sáng: `border 1px #e2dccf`, khe `0px` · tối: `border 0px`, khe `2px` lộ `#201e1b`, bo `3px` | AC6 Story 1.4 |
| **Ép ca xấu** — gỡ `tabindex` khỏi một điểm vào rồi `enterFocus('panel.editor')` | `activeElement` rơi về `body`, và chốt **KÊU**: `[focus] sau khi vào 'panel.editor', focus rơi về 'body' — AC4 nói điều đó KHÔNG được xảy ra…` | **AC4** |
| Toàn phiên | đúng **một** dòng `[focus]` trong console, và nó là dòng của ca xấu ép ra ở trên | AC4 |

**⚠️ Giới hạn của lượt đo này — ghi thẳng, không suy luận thành "tương đương":**

1. **Chạy trên Blink (Chrome), KHÔNG phải WKWebView, và KHÔNG qua `npm run tauri dev`.** Lý do đo được, không phải quên: cổng `1420` mà `vite.config.ts` ghim (`strictPort: true`) đang bị **một dự án khác của Ice** (`gdrive_suite_manager`, PID 65328) chiếm lúc đo; `devUrl` trong `tauri.conf.json` trỏ cứng vào đó, và §Ranh giới phạm vi không cấm đụng tệp đó. Tiến trình của dự án kia **không** bị đụng tới. Lượt đo chạy qua `npx vite --port 1431` rồi lái bằng Chrome; máy chủ tạm đã dừng, cổng 1431 đã trả lại.
2. **Tầng phân phối phím của hệ điều hành chưa đo.** Chrome nuốt `⌘2` để chuyển tab, nên hợp âm được dựng bằng `new KeyboardEvent('keydown', { code, metaKey })` phát trên `window` — tức **tầng ứng dụng** (listener capture, phân giải hợp âm, `preventDefault`, `dispatch`) đã đo đủ; **tầng OS → webview** thì chưa.
3. **Ca Windows: CHƯA ĐO.** Không có máy Windows. Kiểm D chứng minh tầng phân giải đúng ở cả hai nhánh `Mod → ⌘ | Ctrl`, nhưng không chứng minh `Ctrl+1` tới được webview trên Windows.

Cả ba giới hạn đã mở mục trong `deferred-work.md`.

#### Lượt chạy cuối — sáu lệnh, sáu mã thoát

| Lệnh | Mã thoát |
|---|---|
| `npm run check:commands` | **0** — 5 tệp `.vue` + 13 tệp `.ts` · 3 `@click` · 3 `dispatch()` · 4 command · 5 điểm vào focus |
| `npm run check:i18n` | **0** — 18 `.rs` + 5 `.vue` · 11 khoá |
| `npm run check:tokens` | **0** — 7 phép kiểm |
| `npm run check:deps` | **0** — 13 phép kiểm |
| `npm run build` | **0** — vue-tsc ×2 + vite · 43 module |
| `cargo test --locked` | **0** — 15 + 5 test, không hồi quy |

### Completion Notes List

#### Hình dạng đã dựng, và một chỗ lệch khỏi khung của story

Bốn tệp `src/commands/**` đúng như §Ranh giới phạm vi, **không thêm tệp thứ năm**. Nhưng hình dạng của `index.ts` lệch khỏi cách đọc hiển nhiên nhất, và lý do là một ràng buộc chứ không phải sở thích:

**`installCommands({ setMode })` nhận handler phụ thuộc trạng thái qua THAM SỐ TIÊM**, thay vì `index.ts` tự `import { setMode } from '../modes/modeState'`. Nhờ vậy `index.ts` chỉ `import` ba module thuần cùng thư mục và **vẫn nạp được bằng Node thuần** — nên Kiểm E chạy trên **chính bộ command của sản phẩm**, không trên một bản chép trong script (thứ sẽ trôi khỏi sự thật trong hai story). Một dòng `import` `vue` ở đó là ba phép kiểm hành vi chết. `src/main.ts` là chỗ nối hai đầu, và nó gọi **trước `mount()`** vì `dispatch` ném với id chưa đăng ký.

Hệ quả: hướng phụ thuộc là `modes/` → `commands/`, **một chiều**. Ghi vào cả ba README.

**`FOCUS_OWNERS` là một hằng công khai ở `index.ts`** — không có trong khung của story, và nó tồn tại vì Kiểm E cần một sổ để đối chiếu **hai chiều** với mã nguồn. Chiều "khai nhưng không ai dùng" là chiều bắt được một chế độ **quên `declareFocus()`**, tức đúng nguyên nhân làm focus rơi về `body`; không có sổ thì chiều đó không tồn tại.

#### Sáu quyết định thiết kế của story — đã áp đủ

`#1` registry thuần, zero import ✓ · `#2` hợp âm trung lập + `isMac` tiêm được ✓ · `#3` chế độ thắng preset bố cục *(mục `deferred-work.md` giao Story 1.14 chọn phím khác; không mockup không bị sửa)* ✓ · `#4` `labelKey = 'command.' + id`, **Kiểm E cưỡng chế quy ước này** ✓ · `#5` `focus.next_panel` không phím nhưng **handler chạy thật** (xoay vòng trong nhóm `panel.`, đã đo) ✓ · `#6` `<KeepAlive>` ✓.

#### Chuỗi đã soạn — bản cuối và lý do chọn chữ

| Khoá | Chuỗi | Vì sao |
|---|---|---|
| `command.mode.library` | `Library` | PRD §5.2 gạch bỏ *"Thư viện"* và chốt dùng `Library` nhất quán. ⚠️ Không dấu ⇒ `check-i18n` Kiểm A **không** bắt được nếu bị viết thẳng vào `.vue` — nó vẫn đi qua `t()` vì người viết giữ luật, không vì cổng giữ (§Trap 3) |
| `command.mode.workspace` | `Workspace` | PRD §5.2 gạch bỏ *"màn hình dịch"*. Cùng lỗ hổng cổng như trên |
| `command.mode.reading` | `Chế độ đọc` | PRD §5.2 — thuật ngữ tiếng Việt đã chốt, `ReadingMode` chỉ là tên component |
| `command.focus.next_panel` | `Sang panel kế tiếp` | Nhãn thao tác cho màn hình gán phím của Story 1.21. Động từ trước, ngắn, không xưng hô |
| `mode.library.status` | `Library chưa có Tác phẩm nào.` | UX-DR47: **nói việc**. Dùng `Tác phẩm` chứ không `dự án`/`Project` (PRD §5.2). Không hứa hẹn *"hãy nhập tài liệu"* — đường nhập là Epic 6 và một lời mời tới chỗ chưa có là nói dối |
| `mode.workspace.status` | `Chưa có Chương nào được mở.` | `Chương` là thuật ngữ đã chốt (không `document`/`file`). Vô nhân xưng, nêu đúng sự thật |
| `mode.reading.status` | `Chưa có bản dịch nào để đọc lại.` | Nêu **hệ quả** chứ không chỉ sự kiện: nói rõ chế độ này đọc *bản dịch đã xong*, đúng định nghĩa PRD §5.2 |
| `panel.source.title` | `Nguyên văn` | Nguyên văn mockup `key-screen-workspace.html`; cặp *Nguyên văn | Bản dịch* của UX-DR15 |
| `panel.editor.title` | `Bản dịch` | Cùng nguồn |

Cả chín chuỗi qua Kiểm D của `check-i18n` (vô nhân xưng, không *"chúng tôi"*, không *"bạn"*).

#### Xung đột đã phân xử: `⌘1` `⌘2` — chế độ, không phải preset bố cục

`mockups/key-screen-workspace.html:89` vẽ `Bố cục 2×2 nguồn–đích ⌘1 · 4 cột ⌘2`. AC3 của story này, UX-DR34 và `EXPERIENCE.md:49` đều nói `⌘1 ⌘2 ⌘3` là **ba chế độ**. **Chế độ thắng** — AC của epic là hợp đồng nghiệm thu còn mockup là bản phác; UX-DR34 là một mục đánh số còn dòng trong mockup thì không; và ba chế độ là cấu trúc toàn ứng dụng (AD-24) còn preset bố cục chỉ sống trong Workspace. **Story 1.14 phải chọn phím khác cho preset bố cục (FR18)** — đã mở mục. Mockup **không** bị sửa: dev không sửa tài liệu quy hoạch (tiền lệ quyết định #3 của Ice ở Story 1.3).

#### Một lời hứa cũ đã sửa, KHÔNG phải đã đóng

`deferred-work.md:38` ghi *"Hoãn tới **Story 1.6**, khi `#[tauri::command]` thật đầu tiên…"*. Story này giao **0 dòng Rust**: không AC nào cần nó, và AD-1 nói thẳng chuyển chế độ/tiêu điểm/bố cục panel là state UI mà frontend được phép sở hữu. Mục đã được sửa để trỏ sang **Story 1.8** *(hoặc 1.9/1.11)* kèm lý do. Không dựng command giả, không đánh dấu đóng.

#### Bốn cái bẫy — đã đóng cái nào, bằng gì

| Bẫy | Trạng thái |
|---|---|
| **1. `event.metaKey`** ⇒ Windows không chuyển chế độ được, CI vẫn xanh | ✅ Đóng bằng **Kiểm D** — hợp âm trung lập, `isMac` tiêm được, cổng lái **cả hai** ca. Ca D1 chứng minh cổng đỏ đúng lúc |
| **2. Đăng ký trùng id ghi đè im lặng** | ✅ Đóng — `register()` **ném**, Kiểm C chứng minh (ca C1) |
| **3. Nhãn tiếng Anh viết thẳng trong `.vue`** | 🟡 **KHÔNG đóng được ở story này.** `check-i18n` Kiểm A đo **dấu**, không đo chuỗi hiển thị, nên `Library`/`Workspace` viết thẳng vẫn xanh. Story này là story đầu tiên dựng nhãn thật và nó **đứng đúng trên lỗ đó** — cả chín nhãn đi qua `t()` vì người viết giữ luật. Mục đã mở từ Story 1.5, thuộc Story 1.14 |
| **4. `outline: none` toàn ứng dụng** | 🟡 **Tránh được, không cưỡng chế được.** `outline: none` chỉ ở gốc `tabindex="-1"` của chế độ và panel, kèm lý do ngay cạnh mỗi dòng CSS (4 chỗ). Không cổng nào canh — đã mở mục |

#### Ba chỗ lệch nhỏ khỏi chữ của story, ghi ra thay vì để im

1. **`font-weight: var(--weight-read-title)`** ở `PanelFrame.vue` và `App.vue` thay vì `600` viết thẳng. Bộ token **không có** biến trọng lượng cho nhãn giao diện đậm; viết thẳng thì Kiểm B2 của `check-tokens` đỏ (đúng), và không khai một biến CSS cục bộ để lách cổng là đúng thứ AD-34 tồn tại để chặn. Mượn kèm comment ở cả hai chỗ; Story 1.14 quyết token thật.
2. **Ba tab chế độ là `<button>`, không phải `<span>`** như mockup vẽ. Một tab chế độ là thao tác, nên nó phải vào được thứ tự Tab và nhận `Enter`/`Space` — NFR17 nói *"mọi thao tác gọi được bằng bàn phím"*, và một `<span @click>` không gọi được bằng bàn phím ở bất kỳ trình duyệt nào.
3. **`focus.ts` `import` một thứ: `COMMAND_ID_RE` từ `./registry.ts`.** Owner focus dùng **chung** văn phạm với command id, và chép lại biểu thức là tạo đúng ca "hai phép kiểm cưỡng chế hai văn phạm cho cùng một thứ" mà lượt review Story 1.5 đã bắt. `registry.ts` thuần nên Node vẫn nạp được `focus.ts`.

#### Một lỗ trong NFR17 mở ra hôm nay, có chủ ý

`focus.next_panel` chưa có phím (§Quyết định thiết kế #5), nên **trạng thái tiêu điểm của AC5 hôm nay chỉ đến được bằng chuột**. AC5 vẫn nghiệm thu được và đã đo, nhưng lỗ là có thật cho tới khi Story 1.14/1.21 gán phím. Đã mở mục.

### File List

**Thêm mới**

| Tệp | Là gì |
|---|---|
| `src/commands/registry.ts` | `CommandRegistry` — thuần, **zero import**, ba phép cưỡng chế đều ném |
| `src/commands/focus.ts` | Sổ điểm vào focus + chốt chống rơi về `body` + vòng xoay panel |
| `src/commands/keys.ts` | Hợp âm trung lập nền tảng, khớp `event.code`, luật vùng gõ |
| `src/commands/index.ts` | Chỗ **duy nhất** đăng ký · `dispatch` · `declareFocus`/`enterFocus`/`releaseFocus` · `FOCUS_OWNERS` |
| `src/modes/modeState.ts` | Chế độ đang hiện, `ref` Vue, chốt lúc chạy cho giá trị lạ |
| `src/modes/LibraryMode.vue` | Khung rỗng, một câu trạng thái, điểm vào focus |
| `src/modes/WorkspaceMode.vue` | Khung rỗng + **hai** `PanelFrame` |
| `src/modes/ReadingMode.vue` | Khung rỗng, một câu trạng thái, điểm vào focus |
| `src/panels/PanelFrame.vue` | Vỏ panel + hợp đồng thị giác tiêu điểm (AC5) |
| `scripts/check-commands.mjs` | Cổng thứ tư — 5 phép kiểm, mã thoát là phán quyết |

**Sửa**

| Tệp | Sửa gì |
|---|---|
| `src/App.vue` | Thanh tiêu đề + ba tab chế độ (`@click="dispatch('mode.…')"`) · `<KeepAlive>` ba chế độ · CSS vỏ. Khối self-check (`VITE_SCOPE_SELFTEST`, `fallbackReport`, import động `scopeCheck`) **không bị chạm** |
| `src/main.ts` | `installCommands({ setMode })` + `void attachKeyboard(window)`, **trước `mount()`**. Thứ tự `applyTheme()` → `mount()` giữ nguyên |
| `src/i18n/vi.json` | +9 khoá (2 → 11) |
| `src/commands/README.md` | Nội dung thật: bốn tệp, ràng buộc "zero import", văn phạm id, cách thêm command, hợp âm, điểm vào focus, lệnh chạy cổng. Giữ nguyên khối cảnh báo `src-tauri/src/commands/` |
| `src/modes/README.md` | Bảng ranh giới sở hữu 1.6 ↔ 1.14/1.15/Epic 5/Epic 8 + ba quyết định đã chốt |
| `src/panels/README.md` | Bảng ranh giới sở hữu + hợp đồng thị giác tiêu điểm đã đo + hai cơ chế phân tách panel |
| `package.json` | +1 dòng `scripts`: `"check:commands"` |
| `.github/workflows/ci.yml` | +**một** bước trong job `check` đã có, kề `check:i18n`, **trước** `npm run build` + một dòng vào sổ *"CHỖ MÓC CHO EPIC SAU"*. Không workflow thứ hai, không sắp xếp lại bước nào |
| `_bmad-output/implementation-artifacts/deferred-work.md` | Sửa lời hứa `:38` *(không đóng)* + mở 10 mục mới cho Story 1.6 |
| `_bmad-output/implementation-artifacts/sprint-status.yaml` | `1-6-…` → `in-progress` → `review` |

**Không đụng tới** *(đã kiểm bằng `git status`)*: `src-tauri/**` · `tauri.conf.json` · `Cargo.toml` · `src/selftest/**` · `src/tokens/**` · `_bmad-output/planning-artifacts/**` · `index.html` · `vite.config.ts` · `tsconfig.json`. **Không một phụ thuộc nào được thêm** — `package.json` chỉ nhận đúng một dòng `scripts`.

### Review Findings

> Lượt review 2026-08-04 · ba lớp song song *(Blind Hunter · Edge Case Hunter · Acceptance Auditor)* + một lượt xác minh trực tiếp trên mã.
>
> **Nền đã xác nhận đạt thật:** sáu lệnh nghiệm thu chạy lại đều exit 0 *(`check:commands` · `check:i18n` · `check:tokens` · `check:deps` · `npm run build` 43 module · `cargo test --locked` 15+5)*. Bảng 28 ca đỏ-rồi-xanh **đáng tin** — ba ca A1/D1/E5 được dựng lại độc lập và cho thông báo chẩn đoán trùng nguyên văn tới từng số cột. Văn phạm id là **cùng một** biểu thức ở cả ba chỗ *(`check-i18n.mjs:781` ≡ `check-commands.mjs:581` ≡ `registry.ts:62`)*, không có văn phạm thứ hai. `registry.ts` **zero import** thật. Vùng cấm không bị chạm, khối self-check `App.vue` nguyên vẹn, đúng **một** bước CI mới, `package.json` đúng một dòng, 9/9 nhãn qua `t()`, `deferred-work.md:38` được chuyển hướng chứ không đóng.
>
> Các mục dưới đây là thứ **cổng không bắt được**, và phần lớn nằm trong chính cổng.

**Decision — đã phân xử bởi Ice, 2026-08-04**

Cả hai mục đã được quyết. Mục Kiểm E chuyển thành **patch** *(đứng đầu danh sách dưới)*; mục AC4 chuyển thành **defer** có lý do *(cuối mục Defer)*.

**Patch — sửa được, không cần hỏi**

- [x] [Review][Patch] **[Ice chốt: hướng CHẶT]** Kiểm E coi `owner="x"` trong template là một KHAI BÁO, nên panel có thể không khai gì mà cổng vẫn xanh — `scripts/check-commands.mjs:1076-1084` đẩy thuộc tính `owner=` vào **cả** `referencedOwners` **lẫn** `declaredOwners`; chiều ngược lại ở `:1097` chỉ hỏi `declaredOwners.has(owner)`, nên thuộc tính một mình đã thoả. Dựng lại được hai lần: (a) xoá hẳn `declareFocus(props.owner, …)` khỏi `PanelFrame.vue` → cổng in `OK 5 điểm vào focus … đều được declareFocus()`, trong khi thực tế **không panel nào khai**, vòng xoay rỗng, focus không bao giờ tới được panel; (b) thêm `'panel.ghost'` vào `FOCUS_OWNERS` cộng một `<div owner="panel.ghost" />` trần → `OK 6 điểm vào focus`. Đây đúng ca mà comment `:1094-1096` nói chiều này tồn tại để chặn — chặn được với **chế độ** *(literal, ca E5 đỏ đúng)* nhưng **không** với panel nhận owner qua prop. **Cách sửa đã chốt:** thuộc tính `owner=` chỉ vào `referencedOwners`; cổng nối **attribute ↔ component** và đòi mọi component nhận `owner=` phải chứa một `declareFocus(<biến>, …)`. Không khai giới hạn thay cho sửa [scripts/check-commands.mjs:1076-1107]
- [x] [Review][Patch] Kiểm A chỉ biết cách viết `@click`/`v-on:click` — `:onClick="() => {…}"`, `v-on="{ click: … }"` và `@[dyn]` đều là listener click thật trong Vue 3, đều cài thao tác tại chỗ, và đều đi qua cổng XANH *(dựng lại độc lập bởi hai lớp)*. §GIỚI HẠN `:33-36` chỉ khai `@keydown`/`@input`/`@change`/`@submit`, **không** khai các cách viết khác của click ⇒ miễn trừ im lặng [scripts/check-commands.mjs:543]
- [x] [Review][Patch] Kiểm E chỉ dựng bộ command **thật** với `isMac: true`; Kiểm D lái hai nền tảng nhưng trên `fakeRegistry`. Vì `claimed` khoá theo hợp âm **đã phân giải**, một xung đột chỉ tồn tại trên một nhánh: đăng ký thêm `keys: ['Ctrl+1']` → cổng xanh trên cả hai nền tảng CI, nhưng `installCommands({ isMac: false })` **ném** `hợp âm 'Ctrl+1' (Ctrl+Digit1) đã thuộc về 'mode.library'`. Comment CI `ci.yml:17-20` khẳng định phép kiểm này *"đứng giữa `⌘1` và người dùng Windows"* — với keymap thật thì không [scripts/check-commands.mjs:971]
- [x] [Review][Patch] `vueRegions` quét `<script>`/`<style>` trên text **thô**, không phải text đã che comment — đúng loại lỗi không-trạng-thái mà tệp tự hào đã tránh cho `<!--` ở `:348-351`. Chèn `<p>{{ '<style>' }}</p>` mở một vùng CSS giả chạy tới `</style>` thật, nuốt trọn một `<button @click="stealEverything()">` phía sau; cổng xanh và bản tóm tắt vẫn in `3 @click` [scripts/check-commands.mjs:400-417]
- [x] [Review][Patch] Không có **sàn** cho `clickAttrs.length` và `dispatched.length` — `aBad === 0` trên mảng rỗng vẫn in `pass`. Đây là chính cái bẫy *"cây rỗng đọc thành sạch"* mà `check-deps.mjs:15-17` và §Task 9 dựng sàn để chặn, chỉ là ở một tầng sâu hơn; nó là lý do lỗ `vueRegions` ở trên **im lặng** [scripts/check-commands.mjs:565,583]
- [x] [Review][Patch] `enter()` báo `true` cho phần tử đã **tách khỏi DOM** — chỉ kiểm `!el || typeof el.focus !== 'function'`, không kiểm `el.isConnected`. `<KeepAlive>` *(§Quyết định #6)* đỗ `PanelFrame` ở một container tách rời chứ không tháo, nên `panel.source`/`panel.editor` vẫn khai với root đã tách; `el.focus()` là no-op, hàm trả `true`, `last` bị ghi, `armBodyGuard` im vì `activeElement` là root chế độ chứ không phải `body`. Thông báo lỗi ngay bên dưới lại tự nhận là đã kiểm sự hiện diện trong DOM [src/commands/focus.ts:131-141]
- [x] [Review][Patch] `installCommands()` và `attachKeyboard()` chạy **không canh gác** trước `mount()`, mà `register()`/`createKeymap()`/`parseChord()` đều ném theo thiết kế; `index.html` chỉ có `<div id="app">` rỗng ⇒ người dùng nhận **cửa sổ trắng hoàn toàn**. Lý lẽ *"ném thì đỏ ngay ở màn hình đầu tiên"* lặp ở `registry.ts:102-107` và `index.ts:124-127` bị vô hiệu: không có màn hình đầu tiên nào cả. Cộng với lỗi Kiểm E ở trên, đây là dạng hỏng trên Windows [src/main.ts:49-55]
- [x] [Review][Patch] Vạch tiêu điểm panel **nói dối** sau một lượt đổi chế độ: gỡ/đỗ một phần tử đang có focus không chắc chắn phát `focusout`, và `<KeepAlive>` giữ nguyên subtree, nên `focused` kẹt ở `true`. Rời Workspace khi một panel đang có tiêu điểm rồi quay lại: vạch 2px `primary` hiện trên panel trong khi focus thật đã ở root chế độ *(do `onActivated` → `enterFocus`)*. Thuộc tính *"vạch không nói dối"* của AC5 vỡ đúng trên đường `<KeepAlive>` mà §Quyết định #6 bắt buộc; không có `onDeactivated` reset [src/panels/PanelFrame.vue:44-48]
- [x] [Review][Patch] `register()` cưỡng chế `spec`, `id`, trùng id, `labelKey`, `run` — nhưng **không** `keys`. `keys: 'Mod+1'` *(chuỗi thay vì mảng)* bị `[...(spec.keys ?? [])]` rải thành `['M','o','d','+','1']`, và lỗi mới lộ ra sau đó trong `createKeymap` với thông báo trỏ vào một hợp âm `'+'` không ai viết. `keys: [null]` ném `TypeError` trần ở `chord.split`. Cả hai trốn hợp đồng *"ném to, ngay ở cửa"* mà phần còn lại của hàm cài đặt, và cả hai đều với tới được từ Story 1.21 [src/commands/registry.ts:121-125]
- [x] [Review][Patch] Luật vùng gõ chỉ canh hợp âm **không có phím bổ trợ** (`hasNoMods(entry.mods) && isTypingZone(…)`). Một `Shift+B` khớp đúng keydown người dùng tạo ra khi gõ chữ "B" hoa trong textarea; `Alt+M` khớp Option+M *(gõ `µ` trên macOS)*. Cả hai dispatch và `preventDefault()` chạy trước, nên ký tự bị nuốt và command bắn giữa câu. Hôm nay an toàn **do tình cờ** — mọi hợp âm đã đăng ký đều có tiền tố `Mod`; UX-DR46 *(`M`, `B`, `1 2 3` trần)* và Editor Epic 2 làm nó sống [src/commands/keys.ts:246]
- [x] [Review][Patch] Không kiểm `isComposing` — `ChordEvent` không mang trường này và `handle()` không hỏi. Một IME tiếng Việt lúc commit composition phát `keydown` mang `code` vật lý; ở mọi chỗ luật vùng gõ không áp *(hợp âm có phím bổ trợ)*, phím commit bị ăn như một hợp âm và `preventDefault()` giết lượt commit. Đây là ứng dụng dịch tiếng Việt — đường này sẽ được đi hằng ngày từ Epic 2 [src/commands/keys.ts:240-255]
- [x] [Review][Patch] `focus.next()` xoay từ con trỏ **ứng dụng tự giữ** (`last`), không từ `document.activeElement`; `last` chỉ được ghi bởi một `enter()` thành công, chuột và `Tab` không bao giờ cập nhật nó. Sau `enter('mode.workspace')` rồi người dùng **bấm chuột** vào `panel.editor`, `focus.next_panel` nhảy tới `panel.source` — panel **trước** chỗ người dùng đang đứng. `PanelFrame.vue:26-31` bác thẳng đúng khuôn mẫu này cho cờ tiêu điểm *("Một cờ do ứng dụng tự giữ sẽ vẫn sáng đúng một panel trong khi focus thật đã rơi ra ngoài")*; logic xoay vòng vẫn làm y vậy [src/commands/focus.ts:150-158]
- [x] [Review][Patch] Không cổng nào kiểm khoá `t()` **ở chỗ gọi**: Kiểm E chỉ duyệt `labelKey` của command đã đăng ký, `check-i18n` chỉ kiểm hình dạng catalog và hành vi `resolve.ts`. Đổi `title-key="panel.source.title"` → `"panel.sorce.title"` và `t('mode.library.status')` → `t('mode.libary.status')`: **cả hai cổng vẫn xanh**, người dùng thấy khoá thô hiện ra màn hình *(`resolve.ts` cố ý không sập)*. 5 trong 9 khoá mới của story không có cổng nào canh. ⚠️ `t(props.titleKey)` làm chỗ này không kiểm tĩnh được kể cả khi thêm luật [src/panels/PanelFrame.vue:73]
- [x] [Review][Patch] `expectThrow` không xét kiểu lẫn thông báo, nhận **bất kỳ** lỗi nào — một hồi quy làm `register()` ném vô điều kiện sẽ biến cả bảy khẳng định *"⇒ ném"* thành xanh. Bộ kiểm chỉ đỏ sau đó vì `r.register(...)` không canh gác ở `:680` làm sập script, và nó được báo dưới dạng stack trace chứ không phải một FAIL có tên [scripts/check-commands.mjs:634-643]
- [x] [Review][Patch] `if (event.repeat === true) return false` đứng **trước** vòng khớp, nên từ keydown thứ hai trở đi hợp âm giữ phím không còn được `preventDefault()` và rơi xuống webview/OS. Ý định *"không lặp thao tác"* đạt; ý định *"hợp âm đã khớp không bao giờ tới webview"* thì không [src/commands/keys.ts:242]
- [x] [Review][Patch] `FOCUS_CALL_RE` khớp cả **định nghĩa hàm**: cổng in `5 lời gọi truyền biến`, nhưng 3 trong 5 là ba chữ ký `export function` ở `index.ts:81-93`, chỉ 2 là lời gọi `props.owner` thật ở `PanelFrame.vue`. Con số này tồn tại đúng để *"việc bỏ qua chúng không im lặng"* (`:1051-1053`) — tính như hiện tại thì nó không phục vụ được mục đích đó [scripts/check-commands.mjs:1055]
- [x] [Review][Patch] `dispatch(<biến>)` / `dispatch(\`mode.${x}\`)` vừa **vô hình** với Kiểm B vừa **không được đếm** — trái với chính kỷ luật của bộ quét owner ngay dưới, nơi `nonLiteralOwnerCalls` được đếm và in ra [scripts/check-commands.mjs:583-589]
- [x] [Review][Patch] `REGEX_PRECEDERS` có `{` nhưng thiếu `}`, nên một regex literal mở đầu câu lệnh sau một block bị đọc thành phép chia; bộ quét gặp `\/` + `/` kế đó và xoá trắng phần còn lại của dòng — đúng ca mà header `:189-191` gọi là đắt nhất. Một `dispatch(` cuối dòng đó thoát Kiểm B [scripts/check-commands.mjs:196]
- [x] [Review][Patch] `bindings()` trả mảng mới nhưng **cùng tham chiếu object**, không đóng băng — trong khi `registry.ts` đóng băng spec đúng để màn hình gán phím của Story 1.21 không sửa kho ngoài luồng. Bề mặt song song của keymap không có canh gác đó: `bindings()[0].id = 'x'` sửa thẳng keymap đang sống [src/commands/keys.ts:257]
- [x] [Review][Patch] `release()` với owner chưa khai là **no-op im lặng** (`Map.delete` trả `false` và bị bỏ) — lệch khỏi kỷ luật ném/kêu của cả tệp. Một `releaseFocus('panel.sorce')` gõ sai trong `onBeforeUnmount` để owner thật khai vĩnh viễn, rồi lượt mount sau ném *"đã khai rồi"* ở một chỗ không liên quan gì tới chỗ gõ sai [src/commands/focus.ts:82-85]
- [x] [Review][Patch] `⌘1` khi **đang ở** chính chế độ đó: `mode.value = next` không đổi giá trị ⇒ không re-render ⇒ không `onActivated` ⇒ không `enterFocus`. Nếu focus đã rơi về `body`, bấm hợp âm của chế độ hiện tại nuốt phím mà **không dời gì** — đúng thao tác một người dùng sẽ thử để tự cứu [src/commands/index.ts:142-147]
- [x] [Review][Patch] `.shell` đổi sang `display:flex; flex-direction:column; height:100vh`; `.modeport` là `flex:1` *(basis 0%)* còn `.selftest` không khai flex ⇒ `flex:0 1 auto` với `min-height:auto`, tức **không co xuống dưới chiều cao nội dung**. Một báo cáo scope nhiều dòng bóp `.modeport` về gần 0 và tràn khỏi `100vh`, không bên nào có `overflow`. `check:scope` đọc dòng `VERDICT:` từ stdout nên không cổng nào đỏ — nhưng báo cáo trên màn hình của Story 1.2/1.3 thành vô dụng đúng trong bản debug tồn tại để hiển thị nó [src/App.vue:197-205]
- [x] [Review][Patch] `PanelFrame` `declareFocus(props.owner, …)` lúc `onMounted` và `releaseFocus(props.owner)` lúc `onBeforeUnmount` — cả hai đọc prop **tại thời điểm hook**, không phải giá trị đã đăng ký. Khi `owner` thành reactive *(`:owner="…"` hoặc re-key trong `v-for` — chính là thứ `dockview` của Story 1.14 mang tới)*, owner gốc rò vĩnh viễn vào `byOwner` và mọi lượt mount lại nó sẽ ném ở `focus.ts:70`. Cổng không thấy được: lời gọi owner phi-literal bị đếm rồi bỏ qua. Bắt `const owner = props.owner` ở `setup` là đóng [src/panels/PanelFrame.vue:50-55]
- [x] [Review][Patch] `attachKeymap` không có canh gác gọi lại — `installCommands` ném ở lần gọi thứ hai nhưng `attachKeyboard` thì không: hai lần gọi cài hai listener capture trên cùng target và mọi hợp âm dispatch **hai lần**. `setMode` tình cờ idempotent nên nó ẩn; `focus.next_panel` sẽ nhảy cách một panel [src/commands/keys.ts:268-274]
- [x] [Review][Patch] Hợp âm trùng phím bổ trợ được nhận im lặng: `'Mod+Mod+1'` đặt `meta` hai lần và biên dịch ra cùng `resolved` với `'Mod+1'`; lỗi gõ chỉ lộ nếu hợp âm đúng cũng được đăng ký [src/commands/keys.ts:143-167]
- [x] [Review][Patch] Chuỗi ký tự **không** được che *(`:184-187` là quyết định cố ý)*, nên `const HINT = "dispatch('khong.co.that')"` cho một FAIL giả. Hướng an toàn, nhưng nó **không nằm** trong ba giới hạn cổng tự khai — cùng loại "hành vi không khai báo" mà Task 9 cấm ở chiều ngược lại [scripts/check-commands.mjs:184-187]
- [x] [Review][Patch] `src/modes/README.md` **xoá** dòng story không cấm xoá: `-**Story sở hữu nội dung: 1.14** (khung bốn panel). Chế độ đọc thuộc Epic 5, Review Mode thuộc Epic 8.` Task 13 ghi *"Đừng xoá dòng 'Story sở hữu nội dung: 1.14', hãy làm rõ ranh giới"*. Bảng thay thế **có** làm rõ ranh giới nên ý được giữ, nhưng chữ của ràng buộc thì không. `src/panels/README.md:18` giữ đúng [src/modes/README.md]

**Defer — có thật, chưa tới lúc**

- [x] [Review][Defer] `isTypingZone` mù với shadow DOM *(`event.target` trên listener `window` bị retarget về host, nên một `<my-editor>` bọc input đọc ra `tagName: 'MY-EDITOR'`, `isContentEditable: false` ⇒ hợp âm trần bắn khi đang gõ; `composedPath()[0]` không được hỏi)*, và chặn nhầm `input[type=checkbox/radio/button/range]` cùng input `disabled`/`readonly` *(mọi hợp âm trần chết im lặng khi focus ở một checkbox)* [src/commands/keys.ts:207-212] — deferred, chưa có shadow DOM hay input phi văn bản nào trong sản phẩm
- [x] [Review][Defer] `armBodyGuard` bắn-và-quên, không huỷ được và không có đường lui khi `rAF` không chạy: cửa sổ ẩn/thu nhỏ thì `requestAnimationFrame` không chạy nên chốt bị bỏ qua đúng trên đường khởi động nền; và một lượt blur giữa `enter()` và callback cho **cáo buộc sai** nêu đích danh một owner đã focus đúng [src/commands/focus.ts:103-113] — deferred, đây là chuông báo chứ không phải cơ chế
- [x] [Review][Defer] Bộ lọc phần mở rộng `endsWith('.ts')` bỏ qua `.tsx` · `.mts` · `.cts` — một tệp như vậy vô hình với Kiểm B và E, và cũng không tính vào `TS_FLOOR` [scripts/check-commands.mjs:122,130-131] — deferred, dự án không dùng ba phần mở rộng đó
- [x] [Review][Defer] **[Ice chốt 2026-08-04]** **AC4 vế panel chưa có đường dời focus tường minh nào chạy được trong sản phẩm hôm nay** — `src/panels/PanelFrame.vue:50-52` chỉ `declareFocus`, không `onActivated`/`enterFocus`. Đường duy nhất vào panel là `focus.next()` qua command `focus.next_panel`, mà command đó **cố ý không gán phím** *(§Quyết định #5)* **và cũng không có `@click` nào dispatch nó** — grep toàn `src/` cho đúng 3 lời gọi `dispatch()`, cả ba là `mode.*`. Handler của `focus.next_panel` là mã sống nhưng **bất khả đạt**, và vế *"mỗi panel dời focus DOM tường minh khi được kích hoạt"* của AC4 hôm nay chỉ được thoả bằng hành vi focus mặc định của trình duyệt khi bấm chuột vào `tabindex="-1"` [src/panels/PanelFrame.vue:50-52] — deferred, **giữ §Quyết định thiết kế #5 nguyên vẹn**: gán phím hôm nay sẽ làm `unbound()` trả mảng rỗng và AC6 mất bằng chứng, còn tự động focus panel là hành vi Story 1.14 có thể phải tháo ra khi có `dockview`. **AC4 hạ xuống "đạt một phần"** — vế khai báo đạt trọn, vế panel nhận ở Story 1.14/1.21

**Dismissed (3)** — hợp âm khớp không `stopPropagation` *(là lập trường thiết kế đã ghi trong tệp, không phải khiếm khuyết)* · Kiểm B không gom id từ bộ đăng ký như §Khung mô tả *(độ phủ có thật ở Kiểm E, chỉ lệch nhãn)* · `outline: none` 3/4 chỗ dùng tham chiếu chéo thay vì lý do đầy đủ *(chấp nhận được)*.

#### Nghiệm thu lượt vá — ĐỎ trước, XANH sau, 20 ca

Ice chốt **vá cả 27**. Vì 12 mục nằm trong `scripts/check-commands.mjs`, mỗi phép kiểm bị đụng đều được chứng minh lại theo đúng kỷ luật Task 10 — *"một cổng chưa từng đỏ là một cổng chưa được chứng minh"*. Mỗi ca chạy trên cây nguồn thật rồi khôi phục nguyên trạng.

| | Ca | exit mong | exit nhận | Chẩn đoán đầu tiên |
|---|---|---|---|---|
| ✅ | **A4** · `:onClick="() => {…}"` nội tuyến | 1 | 1 | `LibraryMode.vue:31:21 — :onClick là một thao tác click KHÔNG kiểm được tĩnh` |
| ✅ | **A5** · `v-on="{ click: … }"` dạng object | 1 | 1 | `LibraryMode.vue:31:17 — v-on là một thao tác click KHÔNG kiểm được tĩnh` |
| ✅ | **A6** · `@[evt]` tên sự kiện động | 1 | 1 | `LibraryMode.vue:31:19 — @[evt] là một thao tác click KHÔNG kiểm được tĩnh` |
| ✅ | **A7** · sàn `@click` — xoá cả ba tab chế độ | 1 | 1 | `abort` · thuộc tính `@click` quét được — 0 (sàn 3) |
| ✅ | **S1** · `{{ '<style>' }}` không còn nuốt `@click` sau nó | 1 | 1 | `LibraryMode.vue:32:19 — @click không phải một lời gọi dispatch('<id>')` |
| ✅ | **S2** · regex literal sau `}` không xoá trắng dòng | 1 | 1 | `dispatch('khong.co.that') gọi một command CHƯA ĐĂNG KÝ` |
| ✅ | **C5** · `register()` ném VÔ ĐIỀU KIỆN ⇒ FAIL **có tên** | 1 | 1 | `đường HỢP LỆ: register() một spec đúng ⇒ KHÔNG ném — NÉM ở đường hợp lệ` |
| ✅ | **D4** · `repeat` return trước `preventDefault()` | 1 | 1 | ``repeat: true` vẫn là một hợp âm KHỚP — nhận false, phải là true` |
| ✅ | **D5** · bỏ phép kiểm `isComposing` (IME tiếng Việt) | 1 | 1 | ``isComposing: true` ⇒ KHÔNG khớp — nhận true, phải là false` |
| ✅ | **D6** · vùng gõ quay lại `hasNoMods` *(`Shift+B` bắn khi đang gõ)* | 1 | 1 | ``Shift+B` KHÔNG khớp trong vùng gõ (thiếu bổ trợ chính) — nhận true` |
| ✅ | **D7** · bỏ canh gác phím bổ trợ lặp | 1 | 1 | `phím bổ trợ viết LẶP (Mod+Mod+1) ⇒ ném — nhận false, phải là true` |
| ✅ | **D8** · bỏ canh gác gắn keymap hai lần | 1 | 1 | `gắn keymap HAI LẦN vào một target ⇒ ném — nhận false, phải là true` |
| ✅ | **E7** · `PanelFrame` KHÔNG tự `declareFocus()` *(quyết định của Ice)* | 1 | 1 | `WorkspaceMode.vue:40:7 — <PanelFrame> nhận owner="panel.source" nhưng KHÔNG tự declareFocus()` |
| ✅ | **E8** · xung đột hợp âm **chỉ tồn tại trên Windows** | 1 | 1 | `bộ command THẬT KHÔNG dựng được keymap trên Windows/Linux: [keys] hợp âm 'Ctrl+1' …` |
| ✅ | **E9** · khoá `t()` gõ sai trong `.vue` | 1 | 1 | `LibraryMode.vue:32:26 — khoá mode.libary.status (qua t('…')) KHÔNG có trong vi.json` |
| ✅ | **E10** · `title-key` gõ sai ở chỗ gọi component | 1 | 1 | `WorkspaceMode.vue:40:51 — khoá panel.sorce.title (qua title-key) KHÔNG có trong vi.json` |
| ✅ | **N7** · thêm một `@click` HỢP LỆ | 0 | 0 | — |
| ✅ | **N8** · `{{ '<style>' }}` một mình, không vi phạm | 0 | 0 | — |
| ✅ | **N9** · `dispatch(<biến>)` — đếm, không FAIL | 0 | 0 | — |
| ✅ | **N10** · comment chứa `dispatch('khong.co.that')` | 0 | 0 | — |

> **Ca E8 là ca đáng giá nhất của lượt này.** Trước khi vá, thêm `keys: ['Ctrl+1']` vào bộ command cho cổng **XANH trên cả hai nền tảng CI** rồi ném lúc khởi động **chỉ trên Windows** — tức cửa sổ trắng, vì lượt ném xảy ra trước `mount()`. Comment ở `ci.yml:17-20` khẳng định Kiểm D *"đứng giữa `⌘1` và người dùng Windows"*; với keymap thật thì trước lượt vá này nó không đứng ở đó.

> **Hai chỗ lượt vá tự phát hiện thêm, ngoài 27 mục:**
> - Phép kiểm `repeat` của Kiểm D đang khẳng định **sai thuộc tính** — nó đo giá trị trả về của `handle()`, gộp *"không lặp thao tác"* và *"hợp âm đã khớp không rơi xuống webview"* thành một mệnh đề và ép cài đặt phải bỏ một trong hai. Đã tách thành hai khẳng định: `preventDefault()` vẫn chạy, và `fired.length` không tăng.
> - Hộp lỗi khởi động thêm vào `src/main.ts` ban đầu dùng màu và cỡ chữ viết thẳng ⇒ `check:tokens` Kiểm B **đỏ đúng**. Không dùng miễn trừ `aura-allow-literal`: `applyTheme('light')` chạy **trước** khối `try` đó nên token đã có sẵn trên `documentElement`, và hộp lỗi nay dùng `var(--color-error)` · `var(--color-background)` · `var(--font-ui-mono)`.

#### Lượt chạy cuối sau khi vá — sáu lệnh, sáu mã thoát

| Lệnh | Mã thoát |
|---|---|
| `npm run check:commands` | **0** — 5 tệp `.vue` + 13 tệp `.ts` · 3 `@click` · 3 `dispatch()` · 8 khoá `t()` ở chỗ gọi · 4 command · 5 điểm vào focus · keymap dựng được trên **cả hai** nền tảng |
| `npm run check:i18n` | **0** |
| `npm run check:tokens` | **0** |
| `npm run check:deps` | **0** |
| `npm run build` | **0** — vue-tsc ×2 + vite |
| `cargo test --locked` | **0** — 15 + 5 test, không hồi quy |

⚠️ **Vẫn chưa đo:** lượt nghiệm thu DOM trên WKWebView thật và ca Windows — cả hai đã có mục trong `deferred-work.md` từ lượt dev và **không** được lượt vá này đóng. Phép kiểm `isConnected` mới ở `focus.ts` cũng chưa có ca đỏ tự động *(nó chỉ kích hoạt trên phần tử DOM thật; đối tượng giả trong cổng không mang `isConnected`)* — ghi ra thay vì để im.

---

## Change Log

| Ngày | Ai | Gì |
|---|---|---|
| 2026-08-04 | Amelia (Dev) | Cài đặt trọn 13 task. Thêm `src/commands/**` (4 tệp, `registry.ts` zero-import), `src/modes/**` (3 chế độ + `modeState.ts`), `src/panels/PanelFrame.vue`, và cổng thứ tư `scripts/check-commands.mjs` (5 phép kiểm) gắn thành **một** bước trong job `check` đã có. +9 khoá `vi.json`. Nghiệm thu **28 ca đỏ-rồi-xanh** (22 vi phạm + 6 đối chứng âm) — hai ca đầu chạy đã lộ chỗ cổng còn yếu và **cổng được sửa**, cộng một lỗi thật trong sản phẩm (`instanceof HTMLElement` qua ranh giới realm). Nghiệm thu DOM trên webview thật có bảng, gồm cả một ca xấu ép chốt chống rơi về `body` phải kêu. Sửa lời hứa `deferred-work.md:38` *(story này giao 0 dòng Rust)* và mở 10 mục hoãn. Sáu lệnh nghiệm thu đều exit 0. |
| 2026-08-04 | Bob (SM) | Dựng story từ `epics.md#Story 1.6`, ARCHITECTURE-SPINE (AD-34, AD-24, AD-1, Consistency Conventions, Cây nguồn, Stack), DESIGN.md + EXPERIENCE.md (UX-DR7/8/17/34/46/47), PRD §5.2, trạng thái repo `HEAD = b482dc1`, và bàn giao từ Story 1.2/1.3/1.4/1.5. Ghi bốn cái bẫy, sáu quyết định thiết kế đã chốt, một xung đột mockup↔UX-DR đã phân xử, và một lời hứa cũ ở `deferred-work.md:38` phải sửa lại. |
