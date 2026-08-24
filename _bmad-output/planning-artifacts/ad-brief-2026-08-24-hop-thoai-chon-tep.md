# Hồ sơ bàn giao cho Winston — một `AD` mới về **hộp thoại chọn tệp của hệ điều hành**

**Ngày:** 2026-08-24 · **Người bàn giao:** lượt `bmad-build` mở Story 3.10 · **Người nhận:** Winston (architect)
**Nguồn gốc:** AC1 của Story 3.10 (*"người dùng xuất ⇒ sinh ra file CSV hoặc TSV"*) đòi một đường
chọn đích ghi mà kho **cố ý không có**.
**Quyết định của Ice:** đi đường `tauri-plugin-dialog` — Ice chọn **hai lần**, lần thứ hai sau khi
đã đọc trọn số đo ở §3 của hồ sơ này.
**Trạng thái Story 3.10:** **chưa viết một dòng spec nào.** Story dừng ở bước làm rõ ý định của
`bmad-build`. Không mã, không spec, không tệp nào của story bị chạm.
**Baseline cây nguồn:** `044d7a6` (cây sạch) · `grep -c "^### AD-"` = **47** · 13 cổng trong
`package.json` · `capabilities/main.json` mang **3** quyền

⚠️ **Số `AD` kế tiếp là 48, nhưng hồ sơ này KHÔNG được đặt số.** Hai hồ sơ khác đang xếp hàng và cả
hai cũng ghi *"kế tiếp là 48"*: `ad-brief-2026-08-17-mo-hinh-hoan-tac.md` và
`ad-brief-2026-08-17-vach-le-cau-cuoi-chuong.md` — đo 2026-08-24: spine dừng ở AD-47, không mục nào
trong spine nói về mô hình hoàn tác hay vạch lề câu cuối Chương, nên **cả hai còn mở**. Số thật do
Winston gán lúc viết, theo thứ tự viết.

---

## 1. Vì sao hồ sơ này tồn tại

Story 3.10 cần một hộp thoại chọn tệp cho cả hai chiều xuất và nhập. Kho **cấm** `tauri-plugin-fs`
và `tauri-plugin-dialog` bằng một cổng có mã thoát (`scripts/check-deps.mjs` Kiểm 1), và cấm bằng
một quyết định có chữ ký của Ice ngày 2026-08-03.

`project-context.md` §Story và spec viết bằng chữ: *"🔴 Đổi một bất biến kiến trúc là một `AD` MỚI,
không phải một dòng mã."* Và chính bộ test của kho đã nói trước điều đó — thông báo `assert!` của
`src-tauri/tests/config_invariants.rs:376` viết nguyên văn:

> *"Mọi quyền `<plugin>:…` ở đây là một bề mặt IPC mới — **phải là một AD mới trước đã**"*

⇒ Story 3.10 dừng, `AD` viết trước.

---

## 2. Bảy chỗ phải sửa nếu `AD` chấp thuận — đếm được, không phải ước lượng

| # | Chỗ | Hôm nay | Phải thành |
|---|---|---|---|
| 1 | `scripts/check-deps.mjs:163` | `['tauri-plugin-fs', 'AD-1 + AD-29 — Ice chốt 2026-08-03']` | gỡ khỏi `BANNED_CRATES` |
| 2 | `scripts/check-deps.mjs:165` | `['tauri-plugin-dialog', 'cùng lý do: không phơi filesystem ra JS']` | gỡ khỏi `BANNED_CRATES` |
| 3 | `scripts/check-deps.mjs:150-156` | chú thích khai lý do cấm | viết lại kèm ngày và lý do lật |
| 4 | `ARCHITECTURE-SPINE.md:846` | hai tên trong hàng *"Không dùng, đã loại có lý do"* | dời sang bảng Stack, kèm giấy phép |
| 5 | `src-tauri/SECURITY-NOTES.md:81` | liệt `tauri-plugin-dialog` là đã loại | sửa tại chỗ kèm 🔵 và ngày |
| 6 | `src-tauri/capabilities/main.json` | 3 quyền, mô tả nói *"Không plugin filesystem — AD-1, AD-29"* | tuỳ §5 dưới đây |
| 7 | `src-tauri/tests/config_invariants.rs:347` | `assert_eq!` khoá **đúng** ba quyền | tuỳ §5 dưới đây |

Mục 6 và 7 **chỉ** phải sửa nếu `AD` chọn phơi hộp thoại ra JavaScript — xem §5, đó là chỗ rẽ thật
của hồ sơ này.

Ngoài ra: `.memlog.md:157` chở nguyên văn quyết định 2026-08-03 của Ice. Đó là **lịch sử**, không
sửa — `AD` mới nối tiếp nó, không ghi đè nó.

---

## 3. Số đo — đọc thẳng từ nguồn đã tải, 2026-08-24

### 3.1 🔴 `tauri-plugin-dialog` kéo theo `tauri-plugin-fs`, và đó là phụ thuộc CỨNG

`~/.cargo/registry/src/index.crates.io-…/tauri-plugin-dialog-2.7.2/Cargo.toml`:

```toml
[dependencies.tauri-plugin-fs]
version = "2.5.1"
```

Không `optional = true`. Không feature gate. ⇒ Chấp thuận một tên là chấp thuận **hai** tên; đó là
lý do bảng §2 có hai hàng chứ không một, và hàng thứ hai chính là hàng Ice đích thân chốt.

### 3.2 Chênh lệch cây phụ thuộc — đếm trên `src-tauri/Cargo.lock` của `044d7a6`

| Đường | Crate MỚI vào cây feature mặc định |
|---|---|
| crate `rfd` thẳng | **1** — `rfd` 0.16.0 |
| `tauri-plugin-dialog` | **≥9** trên macOS |

Chín tên đó: `tauri-plugin-dialog` · `tauri-plugin-fs` · `rfd` · `notify` · `notify-debouncer-full`
· `notify-types` · `flume` · `file-id` · `trash`; cộng `fsevent-sys` (macOS) hoặc `inotify` +
`kqueue` (Linux/BSD).

⚠️ **Phép đối chứng làm số này đáng tin:** cả **chín** phụ thuộc của `rfd` đã có sẵn trong
`Cargo.lock` — `objc2`, `objc2-app-kit`, `objc2-foundation`, `objc2-core-foundation`, `block2`,
`dispatch2`, `raw-window-handle`, `windows-sys`, `log`. Nên con số **1** không phải một ước lượng
lạc quan; nó là số đếm.

Thứ đường plugin nhét thêm vào nhị phân phát hành: một **bộ theo dõi hệ thống file** (`notify`) và
một năng lực **xoá vào thùng rác** (`trash`) — cho một tính năng chỉ cần một hộp thoại chọn tệp.

### 3.3 NFR6 — dư địa còn 3.104.634 byte

`deferred-work.md:777`: payload sản phẩm **396.895.366 / 400.000.000**. Chín crate mới ăn vào đúng
con số đó, và `prd.md:946` đã dành dư địa ấy cho **HVTĐTD** + **Cổ hán văn**, hai lớp *"gần như
CHẮC CHẮN không còn vừa"*. ⚠️ **Chưa ai đo** chín crate này thêm bao nhiêu byte — đó là một phép đo
`AD` nên đòi, không phải một con số hồ sơ này có.

### 3.4 Cửa NFR15 — đã mở một tệp, còn tám

Bước ① của NFR15 là *"mở tệp giấy phép trong nguồn ĐÃ TẢI mà đọc"*.

| Crate | Trường `license` | Tệp giấy phép đã đọc |
|---|---|---|
| `rfd` 0.16.0 | `MIT` | `LICENSE` — **21 dòng, MIT trần**, không gộp giấy phép gói nào khác. Tương thích GPLv3 chiều đi vào |
| `tauri-plugin-dialog` 2.7.2 | `Apache-2.0 OR MIT` | có `LICENSE_APACHE-2.0` · `LICENSE_MIT` · `LICENSE.spdx` — **chưa đọc** |
| tám crate còn lại | — | **chưa đọc** |

⇒ Đường plugin còn **tám** lượt rà giấy phép chưa chạy. Bài học `vitest` (khai `"MIT"` nhưng
`LICENSE.md` 811 dòng gộp 27 gói) áp thẳng ở đây: trường `license` không thay được việc mở tệp.

---

## 4. 🔴 Chỗ lý do CẤM rộng hơn vị từ CẤM — Winston cần biết, vì nó nghiêng về phía Ice

Đây là phép đo tôi thấy nghĩa vụ phải ghi ra dù nó chống lại khuyến nghị của chính tôi.

**Vị từ của cổng** là *"tên có mặt trong `cargo tree`"*. **Lý do của cổng** là *"plugin tồn tại để
phơi API ra JavaScript"*. Hai thứ đó **không trùng nhau**, và đo được:

`tauri-plugin-dialog-2.7.2/src/commands.rs:162,176,194,209,251` gọi `window.try_fs_scope()` — trả
`Option`. Nếu `tauri_plugin_fs::init()` **không** được đăng ký thì nó trả `None`, hộp thoại **vẫn
chạy**, và **không một lệnh `fs:*` nào được phơi ra JavaScript**.

⇒ Có `tauri-plugin-fs` trong cây phụ thuộc **không** tự động mở bề mặt IPC mà lệnh cấm tồn tại để
chặn. Cái nó mở là **mã trong nhị phân** — tức §3.2 và §3.3, không phải NFR11.

Thứ **thật sự** còn nguyên hiệu lực của lệnh cấm nằm ở §5.

---

## 5. Chỗ rẽ thật — `AD` phải chọn một, và hai nhánh không cùng hạng

`tauri-plugin-dialog` dùng được theo **hai** cách, và chỉ một trong hai chạm AD-1:

**Nhánh (a) — gọi từ Rust, không cấp quyền JS nào.**
`DialogExt::dialog().file()` (`src/lib.rs:83,182`) là API phía Rust. `capabilities/main.json` giữ
đúng ba quyền; `config_invariants.rs:347` **không phải sửa**; AD-1 **không bị chạm** — frontend chỉ
`dispatch` một command, Rust mở hộp thoại và ghi tệp.
🔴 **Nhưng ở nhánh này, plugin CHÍNH LÀ `rfd` cộng tám crate không ai gọi tới.** `tauri-plugin-dialog`
bọc `rfd`; bỏ lớp bọc JavaScript đi thì phần còn lại là `rfd`. Đây là chỗ `AD` phải trả lời thẳng:
*tám crate kia mua được gì?*

**Nhánh (b) — cấp `dialog:default`, gọi từ JavaScript.**
`permissions/default.toml` của plugin cấp `allow-message` · `allow-save` · `allow-open`. Đây là một
**bề mặt IPC mới thật**, và nó chạm ba chỗ cùng lúc:
- **AD-1** — *"frontend chỉ render và giữ state UI"*. Một `save()` gọi được từ JS là frontend cầm
  một năng lực hệ thống.
- `capabilities/main.json` — mô tả của chính tệp đó viết *"Không plugin filesystem — AD-1, AD-29"*.
- `config_invariants.rs:347` — `assert_eq!` khoá đúng ba quyền, và thông báo của nó **đã nói trước**
  rằng chuyện này cần một `AD`.

⇒ Nhánh (b) là thứ hồ sơ này thật sự tồn tại để hỏi. Nhánh (a) không cần lệnh cấm bị gỡ vì lý do
NFR11 — nó chỉ cần lệnh cấm bị gỡ vì **vị từ** của cổng đọc tên, không đọc cách dùng (§4).

---

## 6. Phương án đã nêu và đã bị Ice bác — ghi lại, không giấu

Tôi đã trình ba đường; Ice chọn `tauri-plugin-dialog`, giữ nguyên sau khi đọc §3.

| Đường | Crate mới | `AD` mới? | Ghi chú |
|---|---|---|---|
| crate `rfd` thẳng trong Rust | 1 | **không** | Là AD-11/AD-29 áp lần thứ ba: dùng thẳng crate nền, bỏ lớp bọc JS. Cần một hàng vào bảng Stack (NFR15 bước ②) |
| ô nhập đường dẫn đích | 0 | không | Đối xứng với đường NHẬP của Story 1.15 (`libraryImport.ts:300-312`: kéo-thả **điền vào ô**, một nút xác nhận). Người dùng phải tự biết đường dẫn |
| `tauri-plugin-dialog` | ≥9 | **có** | ← Ice chọn |

⚠️ **Vì sao ghi lại thay vì bỏ đi:** `AD` là tài liệu nói *phương án bị loại đã bị loại bằng gì*.
Một `AD` chỉ chép lựa chọn thắng cuộc là một `AD` không kiểm được. Ba hàng trên là dữ kiện cho
Winston cân, không phải một lượt nêu lại ý kiến.

---

## 7. Điều `AD` phải trả lời

1. **Nhánh (a) hay (b) của §5?** Đây là câu hỏi trung tâm; hai nhánh khác hạng, và chỉ (b) chạm AD-1.
2. **Nếu (a):** tám crate thừa mua được gì so với `rfd` thẳng? Nếu câu trả lời là *"không gì"* thì
   `AD` này biến thành một `AD` về `rfd`, và §2 rút từ bảy chỗ xuống một hàng bảng Stack.
3. **Nếu (b):** AD-1 đổi thế nào? Nó nói *"frontend chỉ render và giữ state UI"* — mệnh đề đó thu
   hẹp lại, hay mọc một ngoại lệ có tên? *(Khuôn có sẵn: `tauri-plugin-wdio-webdriver` là ngoại lệ
   DUY NHẤT của AD-45, đi qua hai lớp chặn.)*
4. **`check:deps` Kiểm 1 còn lại gì?** Gỡ hai trong sáu tên thì bốn tên còn lại có còn chung một câu
   lý do không, hay chúng phải mọc lý do riêng từng tên? Và §4 gợi ra một câu hỏi thứ hai: vị từ
   *"tên trong `cargo tree`"* có còn đúng hình dạng cho lý do *"không phơi API ra JS"* không.
5. **Ai đo NFR6?** Chín crate vào nhị phân với dư địa 3,1 MB. `AD` nên nêu đích danh chủ của phép đo
   đó, không để nó rơi vào khoảng trống.
6. **Tám lượt rà giấy phép NFR15** — chạy trước khi thêm, và ghi vào bảng Stack trước khi thêm.

---

## 8. Story 3.10 trong lúc chờ

Story ở `backlog`, **chưa có spec**. Không tệp nào của story bị chạm; `sprint-status.yaml` giữ
nguyên `3-10-…: backlog`.

Hai vế của Story 3.10 **không** phụ thuộc `AD` này như nhau:

- **Nửa định dạng** — sinh CSV/TSV, phân tích cú pháp, đối chiếu bất đồng, ca thiếu cột, ca thiếu
  bản dịch ⇒ chờ chốt, xuất xứ phân biệt được — **không** đụng hộp thoại. Nó chạy trên `&str` và
  `&Path`, nghiệm thu được trọn bằng `cargo test` không cần một pixel nào.
- **Nửa chọn tệp** — đúng hai chỗ: lấy đường dẫn nguồn khi nhập, lấy đường dẫn đích khi xuất.

⇒ Nếu Ice muốn Story 3.10 chạy song song với `AD`, đường đó có: dựng nửa định dạng trước, để nửa
chọn tệp sau một chỗ nối một hàm. **Nhưng đó là quyết định của Ice, không phải của hồ sơ này** —
tôi ghi ra để nó là một lựa chọn thấy được, không phải để đề nghị.

Một món nợ **đã có chủ là Story 3.10** và không đổi theo `AD` này: `deferred-work.md:6664` — nhánh
va `UNIQUE` của `idx_glossary_entry_source_term` phải được viết ra ở story này *"dù story này có
dựng hay không"*.
