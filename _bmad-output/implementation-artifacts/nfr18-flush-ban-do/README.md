# Bàn đo Story 2.4 — NFR18 / NFR2

Lưu ở đây để **không phải đo lại từ đầu**. Phần khó dựng lại không phải mã, mà là **những hằng số
chỉ ra được bằng đo** — chúng nằm ở §Hằng số phải đo bên dưới.

⚠️ Đây là **tạo tác của một mũi thăm dò**, không phải công cụ của dự án. Không có gì ở đây vào
`package.json`, đúng AC16. Cùng khuôn `2-2-ban-do/` và `2-3-ban-do/`.

## Chạy — LƯỚI SÁU ĐIỂM (đường chính, Task 5 + AC8)

```sh
cd _bmad-output/implementation-artifacts/2-4-ban-do
./run-grid.sh
```

Chỉ một lệnh. Nó tự kiểm **năm** điều kiện tiên quyết, rồi với mỗi điểm trong
`512 KiB · 1 · 2 · 4 · 8 · 16 MiB`: đổi hằng ở đúng chỗ khai → dựng lại release → chạy tới khi
đủ **≥ 20 mẫu hợp lệ**. Cuối cùng gọi `grid-table.sh` in bảng của AC8.

🔴 **Nó ĐỘC CHIẾM bàn phím và chuột ~3,5 giờ.** Đừng chạm máy trong lúc chạy.
⚠️ Máy nên **rảnh**: đo 2026-08-18 `loadavg` nền là 6–7 trên một máy 8 nhân, và NFR18 là mệnh
đề về **đuôi phân bố** — chỗ tải nền hiện ra trước tiên.

Ba tính chất đã **tự kiểm đỏ-rồi-xanh**, không phải lời hứa:

| Tính chất | Ca đã thử |
| --- | --- |
| **Trả lại hằng số** kể cả khi Ctrl-C | `trap` cài **sau** phép kiểm sạch — cài trước là để `die()` của chính phép kiểm xoá mất thay đổi nó vừa từ chối đụng |
| **Không xoá việc của người khác** | `mod.rs` bẩn vì chuyện khác ⇒ DỪNG, và thay đổi đó **còn nguyên** |
| **Nhặt lại sau một lượt bị giết cứng** | `mod.rs` chỉ lệch ở `wal_threshold_bytes` ⇒ nhận ra là tàn dư, tự `git checkout`, **không** khuyên commit một hằng đo dở |

**Chạy lại được:** điểm nào đã đủ mẫu thì bỏ qua. Chết ở điểm thứ tư **không** phải làm lại từ đầu.

### Chạy một điểm lẻ

```sh
./classify-selftest.sh                          # phải 7/7, chạy TRƯỚC mọi lượt kill
./kill-campaign-v2.sh 20 g4mib 20 55 30         # <mẫu-hợp-lệ> <nhãn> [min] [max] [trần bắn]
WAL_EVERY=4 ./kill-campaign-v2.sh 2 doichung 20 25 4   # đối chứng AC21: đổi nhịp `stat()`
./ac21-control.sh 25                            # đối chứng AC21: chi phí bàn đo, không gõ
./calib-shot.sh                                 # hiệu chuẩn toạ độ: dựng tới Workspace rồi CHỤP
```

`build-bench.sh` **chỉ** cần cho nửa **NFR2** *(tiêm `bench.js` để lấy mẫu rAF)*. Nửa **NFR18**
không cần nó — xem §Nửa NFR2 dưới đây.

Kết quả: `kill2-<nhãn>.tsv` *(một hàng một lượt)* · `wal2-<nhãn>.tsv` *(WAL hai kho theo thời
gian)* · `app2-<nhãn>-N.log` *(stderr, mang dòng chẩn đoán `store[...]`)*.

## Hằng số phải ĐO mới ra — đừng đoán lại

| Hằng | Giá trị | Đo bằng cách nào |
| --- | --- | --- |
| Chặng Tab tới ô "đường dẫn tệp" | **5** | form rỗng ⇒ nút submit `:disabled` **bị bỏ qua** trong chuỗi Tab |
| Nút "Tạo Tác phẩm từ tệp" | `(+85, +685)` so gốc cửa sổ | hiệu chuẩn từ ảnh, cửa sổ chuẩn hoá `1200×900` |
| Tab "Workspace" | `(+101, +46)` | như trên |
| Vùng gõ trong panel Bản dịch | 🔵 **`(+372, +170)`** *(2026-08-18)* | xem §Vì sao phải tự nghiệm thu — và §Bề mặt đã đổi |
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

## 🔵 BỀ MẶT ĐÃ ĐỔI — hiệu chuẩn lại 2026-08-18

Lượt correct-course 2026-08-14 thay `EditorPanel.vue` *(một dòng văn liên tục)* bằng
`GridPanel.vue` *(lưới hai cột, chiếm **nửa trái** cửa sổ)*.

| Hằng | 13/8 | 18/8 | |
| --- | --- | --- | --- |
| Tab×5 tới ô đường dẫn · nút `(+85,+685)` | | **y nguyên** | 🟢 form Library không đổi |
| Tab Workspace `(+101,+46)` | | **y nguyên** | 🟢 |
| Ô gõ | `(+640,+165)` | **`(+372,+170)`** | 🔴 `+640` nay rơi vào panel **Tra cứu** |

Số mới đo bằng **hai đường độc lập và chúng khớp**: ① ảnh `calib-shot.sh` cho tâm cột
`[data-col="tgt"]` ≈ `+372`; ② suy từ `grid-template-columns: 3px 30px 1fr 1fr 96px`
(`GridPanel.vue:1645`) cho ≈ `+365`. `focus-segment.sh` trúng **ngay ứng viên đầu ở 6/6 lượt**.

## 🔴 Ba lỗi CỦA CHÍNH BÀN ĐO tìm ra 2026-08-18 — cả ba đã vá

Cộng vào ba lỗi ở §Hai lỗi bên trên. Tất cả cùng một hạng: **hàng rào báo sai**.

1. **`focus-segment.sh` giữ `sleep 2.4`** — trong khi chính README này ghi *"≥ 4,5 s · 2,4 s cho
   âm tính giả ở cả 16 ứng viên"*. Một số **đã được đo là hỏng** vẫn nằm trong mã: bản đã commit
   sẽ báo *"không đặt được con trỏ"* trên mọi ứng viên, và lượt chẩn đoán sau đi vá **toạ độ**
   trong khi chỗ hỏng ở **nhịp**.
2. **`grep -c … || echo 0` ở bảy biến** — `grep -c` in `0` **và** thoát 1 ⇒ biến thành chuỗi hai
   dòng ⇒ `perl` chết cú pháp ⇒ `BLURPCT` **rỗng** ⇒ 🔴 **cổng mất tiêu điểm của AC21 không chặn
   gì**. Lỗi này đã có tên ở §Hai lỗi **nhưng bản vá chưa bao giờ vào mã**.
3. **`pgrep -f 'auratranslate'`** khớp một shell vô can chỉ vì dòng lệnh của nó *chứa* đường dẫn
   nhị phân ⇒ báo *"app đang chạy"* trong khi 0 tiến trình app nào chạy. ⇒ `pgrep -x`.

🔵 Bài học chung, và nó đắt hơn cả ba: **một lỗi được GHI RA không có nghĩa là nó đã được VÁ.**
Lỗi ① và ② đều đã có tên trong chính tệp này từ 13/8; cả hai vẫn sống trong mã tới 18/8.

## Nửa NFR2 — KHÔNG chạy từ đây

Ba đầu dò DOM của `bench.js` *(`:152`, `:160`, `:199`)* hỏi `.doc` và `.sent`. Cả hai selector
**không còn tồn tại** trong `src/` sau lượt correct-course — chúng chỉ còn trong **chú thích** của
`GridPanel.vue`. ⇒ Đầu dò trả *"KHÔNG THẤY .doc"*, tức **đo rỗng**, không phải đo ra số xấu.

AC12 gọi tên **ba** đường nóng và AC13 gọi tên **một** phép so; cả bốn cái tên nay trỏ vào hư
không. Viết lại chúng là **đổi nội dung một AC** ⇒ Ice chốt 2026-08-18: nửa NFR2 đi qua một lượt
**correct-course**, không qua một lượt sửa mã của dev.

## Chưa dựng

- Ép hai luồng checkpoint **trùng pha** *(AC10 vế ③)*
- Đo tranh chấp **CPU/I-O** thật *(AC10 vế ②)* — `ps -M` hiện lấy nhầm cột
- Bơm phím Unicode ở tầng CoreGraphics — đường duy nhất cho **phím thật + dấu thật**.
  🔵 Ice chốt 2026-08-18: **giữ ASCII**, phần tiếng Việt + IME + ca *xoá lùi qua đầu câu* là
  **nợ có chủ**, và báo cáo phải nói thẳng là bàn đo không chạm hai vế đó.
