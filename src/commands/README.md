`CommandRegistry` của **giao diện** — MỌI thao tác người dùng đăng ký ở đây rồi mới bind vào chuột/phím (AD-34, FR22). Không thao tác nào chỉ tồn tại trong một handler chuột.

Thư mục này chứa **cả hai nửa của AD-34**: §1 thao tác (`registry.ts` · `keys.ts`) và §2 điểm vào focus (`focus.ts`). Chúng ở chung một chỗ vì chúng là hai mệnh đề của cùng một AD, không phải hai tính năng tình cờ liên quan.

**Story sở hữu nội dung: 1.6.** Màn hình gán phím, đổi phím, lưu phím xuống đĩa là **Story 1.21**.

---

## Bốn tệp

| Tệp | Là gì | `import` được gì |
|---|---|---|
| `registry.ts` | Kho command: `register` · `has` · `dispatch` · `list` · `unbound` | **KHÔNG GÌ CẢ** |
| `focus.ts` | Sổ điểm vào focus + chốt chống rơi về `body` | chỉ `./registry.ts` |
| `keys.ts` | Hợp âm trung lập nền tảng → `dispatch` | chỉ **kiểu** từ `./registry.ts` |
| `index.ts` | Chỗ **DUY NHẤT** đăng ký + ba singleton của ứng dụng | chỉ ba tệp trên |

### Vì sao cột bên phải là một ràng buộc, không phải một sở thích

Dự án **không có bộ chạy test frontend**, và thêm một (`vitest`) là thêm một phụ thuộc phải rà tương thích GPLv3 bằng cách mở tệp giấy phép trong nguồn đã tải, rồi vào bảng Stack **trước khi** thêm (NFR15). Đó là quyết định của Ice.

Đường không tốn gì — và nó **đã chạy thật trong CI từ Story 1.5** với `src/i18n/resolve.ts`: Node ≥ 22.18 bóc kiểu TypeScript mặc định, nên `scripts/check-commands.mjs` `import()` thẳng bốn tệp này và khẳng định **hành vi**, không chỉ đọc văn bản. Một dòng `import` giá trị từ `vue`, từ `../modes/**` hay từ một tệp `.json` là ba phép kiểm hành vi (C, D, E) chết ngay hôm đó.

Điều kiện kèm theo, cú pháp **"erasable-only"**: không `enum` · không `namespace` · không parameter property (`constructor(private x)`). `type` / `interface` / annotation đều được. Ba thứ bị cấm SINH MÃ chứ không chỉ mang chú thích, nên Node từ chối bóc.

⚠️ Import trong thư mục này viết **kèm đuôi `.ts`** (`from './registry.ts'`) — Node cần nó để phân giải. Nơi khác trong dự án viết theo kiểu thường (`from '../commands'`). Dự án **không có alias `@`**: `vite.config.ts` và `tsconfig.json` đều không khai `alias`/`paths`.

---

## Văn phạm id

```
^[a-z0-9]+(\.[a-z0-9_]+)+$
```

Khoá chấm có tiền tố miền: `mode.library` · `focus.next_panel` · `lookup.search_selection`. **Chép đúng** biểu thức mà `scripts/check-i18n.mjs` Kiểm B đang cưỡng chế cho khoá `vi.json` — AD-34 nói command id *"cùng hình dạng khoá `vi.json`"*, và "cùng hình dạng" nghĩa là cùng một biểu thức, không phải "trông na ná".

**Owner của điểm vào focus dùng chung văn phạm đó**: `mode.workspace` · `panel.source`.

⚠️ `labelKey` là **`'command.' + id`**, không dùng thẳng id làm khoá chuỗi. Hai không gian tên khác nhau: một cái định danh thao tác, một cái định danh chuỗi. Tiền tố chừa chỗ cho `command.<id>.hint` — mô tả dài ở màn hình gán phím của Story 1.21 — mà không phá quy ước, và một lượt grep `"command."` trong `vi.json` liệt kê đúng bộ nhãn thao tác.

---

## Thêm một command

1. Thêm chuỗi nhãn vào `src/i18n/vi.json` với khoá `command.<id>`.
2. Đăng ký trong `installCommands()` ở `index.ts` — **chỉ ở đó**:

```ts
registry.register({
  id: 'lookup.search_selection',
  labelKey: 'command.lookup.search_selection',
  keys: ['Mod+L'],          // TRUNG LẬP: `Mod` = ⌘ trên macOS, Ctrl ở nơi khác
  run: () => { /* thao tác thật */ },
})
```

3. Gọi từ giao diện bằng **đúng một** lời gọi `dispatch('<id>')`:

```vue
<button type="button" @click="dispatch('lookup.search_selection')">…</button>
```

`@click="doSomething()"` · `@click="mode = 'library'"` · `@click="$emit('x')"` đều là **FAIL** ở Kiểm A. Handler chuột chỉ được `dispatch` một command đã đăng ký — đó là mệnh đề trung tâm của AD-34 §1, và nó là thứ khiến câu hỏi *"thao tác nào chưa gán được phím"* trả lời được bằng máy (`unbound()`).

### Hợp âm

Viết ở dạng trung lập: `Mod+1` · `Mod+Shift+Enter` · `B`. 🔴 **Đừng viết `event.metaKey`.** `⌘` là ký hiệu macOS của một phím **trừu tượng**; trên Windows nó là `Ctrl`. Một cài đặt chỉ đọc `metaKey` đi qua **cả hai nền tảng của CI** rồi hỏng ở tay người dùng Windows — vi phạm NFR14 nặng nhất còn lọt được. Kiểm D của cổng lái cả hai ca `isMac` và là lưới duy nhất chặn nó.

⚠️ Khớp bằng `event.code` (`Digit1`), **không** bằng `event.key`: trên bố cục không phải US, `event.key` trôi theo bố cục.

⚠️ **Luật vùng gõ:** hợp âm **không có phím bổ trợ** không dispatch khi focus đang ở `input` / `textarea` / `[contenteditable]`. Chế độ đọc dùng `M`, `B`, `1 2 3` trần (UX-DR46) và Editor của Epic 2 là một vùng gõ tự do.

---

## Điểm vào focus

Mỗi chế độ và mỗi panel **khai** một điểm vào (AD-34 §2, UX-DR7), và tên nó phải có trong `FOCUS_OWNERS` ở `index.ts`:

```ts
onMounted(() => { declareFocus('mode.library', () => root.value) })
onBeforeUnmount(() => { releaseFocus('mode.library') })
onActivated(() => { void enterFocus('mode.library') })     // ba chế độ sống trong <KeepAlive>
```

Phần tử đích phải mang `tabindex="-1"` để nhận được focus lập trình.

Cổng đối chiếu `FOCUS_OWNERS` với mã nguồn **hai chiều**: mọi owner dùng trong `.vue` phải có ở đó, và **mọi mục ở đó phải được `declareFocus()`** — chiều thứ hai là chiều bắt được một chế độ quên khai điểm vào, tức đúng nguyên nhân làm focus rơi về `body`.

⚠️ Chốt chống rơi về `body` ở `focus.ts` để **KÊU**, không để **VÁ**: nó `console.error` ở frame kế tiếp nếu `document.activeElement` là `body`. Đừng "sửa" bằng một vòng focus tự phục hồi — nó sẽ đánh nhau với người dùng đang Tab và với hộp thoại của hệ điều hành.

⚠️ `outline: none` **chỉ** áp cho gốc `tabindex="-1"` của chế độ và panel. Một `*:focus { outline: none }` phá NFR17 (*"trạng thái focus luôn nhìn thấy rõ"*) mà **không cổng nào bắt được**.

---

## Chạy cổng

```bash
npm run check:commands     # 5 phép kiểm — A cú pháp `@click` · B văn phạm+tồn tại id
                           #             · C hành vi registry+focus · D hai nền tảng
                           #             · E nhãn trong vi.json + sổ focus hai chiều
```

Nó chạy trong CI ở job `check`, kề `check:i18n` và **trước** `npm run build`.

---

⚠️ **Đừng nhầm với `src-tauri/src/commands/`.** Hai thư mục cùng tên, hai thứ hoàn toàn khác nhau:

| Đường dẫn | Là gì | AD |
|---|---|---|
| `src-tauri/src/commands/` | **Bề mặt IPC** — hàm `#[tauri::command]` mà frontend gọi qua. Adapter thuần, không chứa quy tắc nghiệp vụ | AD-1 |
| `src/commands/` *(thư mục này)* | **`CommandRegistry` của giao diện** — nơi đăng ký thao tác người dùng | AD-34, FR22 |

Hai thứ này **không** ánh xạ một-một và **không** được gộp. Chuyển chế độ, tiêu điểm bàn phím và bố cục panel là **state UI**, và AD-1 nói thẳng đó là phần frontend được phép sở hữu — không `#[tauri::command]` nào cho chúng.

## Năm command bôi đen bằng bàn phím (Story 1.18)

`deferred-work.md:608` — *"Ice chốt 2026-08-06: **ghi nợ cho 1.18**"*. AC1 của epic đòi
*"thả chuột **hoặc kết thúc vùng chọn bằng bàn phím**"*, và vế thứ hai trước story này không
**thực hiện được**: một `<div>`/`<p>` không sửa được **không nhận** `Shift+Mũi tên`.

| Command | Hợp âm mặc định |
|---|---|
| `selection.focus_source` | `Mod+Alt+S` |
| `selection.extend_left` / `extend_right` | `Shift+←` / `Shift+→` |
| `selection.extend_word_left` / `extend_word_right` | `Alt+Shift+←` / `Alt+Shift+→` |

🔴 **Vì sao chúng là COMMAND, không một `@keydown` trên bề mặt chữ:** AD-34 §1 — sàn khả năng
tiếp cận là **CẤU TRÚC**, không kỷ luật. Một handler gắn thẳng vào phần tử không gán lại phím được,
không liệt kê được ở màn hình gán phím của **Story 1.21**, và không đi qua ba phép cưỡng chế của
`register()`.

⚠️ **`Mod+Shift+Mũi tên` bị BÁC** dù story đề xuất nó: `⌘⇧…` là không gian **UX-DR35** giữ.
`Mod+Alt+Mũi tên` đã thuộc `focus.next_panel`; `⌥←`/`⌥→` **trần** thuộc *Chương trước/sau*
(Story 2.11).

🔴 **Chúng không giết bôi đen trong ô nhập của Library** vì cả hai hợp âm **không mang `Meta`/`Ctrl`**,
nên luật vùng gõ của `keys.ts:287` (`lacksPrimaryMod && isTypingZone`) bỏ qua chúng khi tiêu
điểm ở `<input>`/`<textarea>`. Luật đó có từ Story 1.6; story này là **người tiêu thụ đầu
tiên** của nó với một hợp âm thật.

⚠️ **Giới hạn đã biết:** `keys.ts:295` trả sớm khi `event.repeat` ⇒ **giữ** phím không mở rộng
liên tục. Xem `deferred-work.md` §1.18 — chủ: Story 1.21.
