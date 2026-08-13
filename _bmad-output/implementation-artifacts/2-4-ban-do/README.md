# Bàn đo Story 2.4 — NFR18 / NFR2

Lưu ở đây để **không phải đo lại từ đầu**. Phần khó dựng lại không phải mã, mà là **những hằng số
chỉ ra được bằng đo** — chúng nằm ở §Hằng số phải đo bên dưới.

⚠️ Đây là **tạo tác của một mũi thăm dò**, không phải công cụ của dự án. Không có gì ở đây vào
`package.json`, đúng AC16. Cùng khuôn `2-2-ban-do/` và `2-3-ban-do/`.

## Chạy

```sh
# 1. Điều kiện tiên quyết — kiểm bằng tay, KHÔNG có thì đừng chạy
#    · không instance app nào đang chạy trên dữ liệu THẬT
#    · phiên đăng nhập GUI này KHÔNG có tác nhân tự động nào khác (xem §Điều kiện tiên quyết)
#    · `cliclick` đã cài  (brew install cliclick)

# 2. Dựng bản đo (release nguyên vẹn, không bật wdio, không debug-assertions)
./build-bench.sh

# 3. Tự kiểm bộ phân loại TRƯỚC khi đốt một lượt kill nào
./classify-selftest.sh          # phải 7/7

# 4. Chạy: <mẫu-hợp-lệ-cần> <nhãn> [giây-min] [giây-max] [trần-lượt-bắn]
./kill-campaign-v2.sh 20 g4mib 20 55 30
```

Kết quả: `kill2-<nhãn>.tsv` *(một hàng một lượt)* · `wal2-<nhãn>.tsv` *(WAL hai kho theo thời
gian)* · `app2-<nhãn>-N.log` *(stderr, mang dòng chẩn đoán `store[...]`)*.

## Hằng số phải ĐO mới ra — đừng đoán lại

| Hằng | Giá trị | Đo bằng cách nào |
| --- | --- | --- |
| Chặng Tab tới ô "đường dẫn tệp" | **5** | form rỗng ⇒ nút submit `:disabled` **bị bỏ qua** trong chuỗi Tab |
| Nút "Tạo Tác phẩm từ tệp" | `(+85, +685)` so gốc cửa sổ | hiệu chuẩn từ ảnh, cửa sổ chuẩn hoá `1200×900` |
| Tab "Workspace" | `(+101, +46)` | như trên |
| Vùng gõ trong panel Bản dịch | **KHÔNG cố định** | xem §Vì sao phải tự nghiệm thu |
| Nhịp nghiệm thu con trỏ | **≥ 4,5 s** | 2 s `EDITOR_IDLE_MS` + đường ghi + biên. 2,4 s cho **âm tính giả** ở cả 16 ứng viên |
| Vị trí cửa sổ | `{200, 25}` | `x=0` để Dock *(~54 điểm)* đè lên nút |

## Bốn chỗ đã cắn, và cách chúng lộ ra

1. **`key 18` (Ctrl+Alt+Shift+1) không phải phím sản phẩm** — `bench.js:242-245` đăng ký nó. Bộ
   kill v1 vì thế phụ thuộc thẳng vào bàn đo tiêm. v1 chưa từng chạy nên chưa ai thấy.
2. **Chạy nhị phân từ shell KHÔNG đưa cửa sổ lên trước.** Phím `osascript` đi vào cửa sổ đang ở
   trước — một lần đã đi vào trình duyệt. ⇒ `front.sh`, cổng **cứng**: không lên trước được thì
   không gửi một phím nào.
3. **`System Events … click at` KHÔNG đặt được con trỏ vào `contenteditable`** *(mở được `<select>`,
   bấm được nút, nhưng không đặt được con trỏ)*. ⇒ `cliclick` phát `mousedown` thật.
4. **`osascript keystroke` CHỈ gõ được ASCII.** `⟦42⟧ Trời hôm nay…` tới kho thành
   `a42a Trai ham nay…`. ⇒ chỉ số truy vết dùng `[n]` ASCII; **tiếng Việt có dấu là nợ có chủ**,
   xem `deferred-work.md`.

## Vì sao phải TỰ NGHIỆM THU con trỏ

Lượt quét lưới tìm được `(840,190)` ăn. Lượt sau, **cùng toạ độ, cùng hình học, cùng tệp nguồn** ⇒
**không** ăn; lượt sau nữa `(860,190)` ăn còn `(840,190)` trượt.

Ice mô tả đúng hiện tượng này khi gõ tay, trước khi bàn đo gặp nó:
*"click vào gần đầu dòng thì mới có chỗ nhập, thao tác rất khó, click không chính xác thì không
hiển thị input và không gõ được."*

⇒ `focus-segment.sh`: bấm → gõ chuỗi dò → **HỎI KHO** → trượt thì thử điểm kế; trúng thì **xoá**
chuỗi dò rồi mới đo. Không tin một toạ độ nào.

## Điều kiện tiên quyết — ngang hàng với hàng rào dữ liệu thật

Phiên đo phải **độc chiếm** phiên đăng nhập GUI. Ngày 2026-08-13 một phiên agent khác có
`computer-use` + `claude-in-chrome` đã cướp tiêu điểm giữa chừng, và phím của bàn đo đi vào ứng
dụng thật. Thiếu điều kiện này thì ① số đo hỏng và ② phím đi vào chỗ không nên đi — cái thứ hai
nghiêm trọng hơn.

## Hai lỗi của chính bàn đo — giữ lại để không lặp

- `grep -c` in ra `0` **và** thoát mã 1 ⇒ `|| echo 0` thêm một dòng nữa ⇒ biến đếm thành chuỗi
  **hai dòng** ⇒ TSV đẻ hàng rác và phép đếm `busy` **vô giá trị, không phải bằng 0**.
- `instr(t,']')` lấy dấu **đầu tiên** ⇒ segment chứa `[7] … [42] …` trả `7`. Phải rút bằng
  `grep -oE '\[[0-9]+\]'` rồi lấy max.
- Bộ lọc tiêu điểm so `'AuraTranslate'` **phân biệt hoa thường** trong khi tiến trình tên
  `auratranslate` ⇒ báo đỏ **oan** 100%.

🔵 Cả ba đều là **hàng rào báo sai**, và cả ba suýt làm lượt chẩn đoán đi vá nhầm chỗ. Khi một hàng
rào báo đỏ, đọc **con số đi kèm** trước khi tin cái nhãn.

## Chưa dựng

- Ép hai luồng checkpoint **trùng pha** *(AC10 vế ③)*
- Đo tranh chấp **CPU/I-O** thật *(AC10 vế ②)* — `ps -M` hiện lấy nhầm cột
- Bơm phím Unicode ở tầng CoreGraphics — đường duy nhất cho **phím thật + dấu thật**
