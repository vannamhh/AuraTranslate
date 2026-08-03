---
title: "Story 1.1 — Mũi thăm dò font: dung lượng thật và rà giấy phép"
status: complete
created: 2026-08-03
updated: 2026-08-03
relates_to:
  - '_bmad-output/implementation-artifacts/1-1-mui-tham-do-font-do-dung-luong-that-va-ra-giay-phep.md'
  - '_bmad-output/planning-artifacts/architecture/architecture-AuraTranslate-2026-08-02/ARCHITECTURE-SPINE.md'
  - '_bmad-output/planning-artifacts/ux-designs/ux-AuraTranslate-2026-08-02/DESIGN.md'
  - '_bmad-output/planning-artifacts/research/phase-0-spike-results-2026-08-02.md'
---

# Story 1.1 — Mũi thăm dò font: dung lượng thật và rà giấy phép

> **`status: complete` đọc theo phạm vi AC1 ĐÃ THU HẸP.** Mũi thăm dò đã đóng trọn ba câu hỏi được giao sau khi Ice thu hẹp AC1 ngày 2026-08-03. **Hai việc vẫn mở**, cả hai đã có AC thật ở story khác: phép đo `.msi` (Story 1.3) và phép đối chiếu NFR6 trên dữ liệu thật (Story 1.9). Đừng đọc `complete` thành "không còn gì phải đo".

**Mục đích:** biến ba phỏng đoán còn lại về bộ font nhúng — dung lượng, giấy phép, biến thể vùng — thành số đo thật và quyết định đã ghi, trước khi bất kỳ story nào của Epic 1 dựng giao diện trên chúng.

**Môi trường đo:** macOS 15.7.7 (Darwin 24.6.0, build 24G720) · x86_64 (Intel) · rustc 1.97.1 · cargo 1.97.1 · Node v22.22.2 · npm 10.9.7 · `tauri-cli` 2.11.4 · crate `tauri` 2.11.5 · `tauri-build` 2.6.3 · Vue 3.5.40 · Vite 8.2.0 · `@vitejs/plugin-vue` 6.0.8 · `@tauri-apps/api` 2.11.0

**Profile release dùng cho cả hai bản build:** `lto = true` · `codegen-units = 1` · `opt-level = "s"` · `panic = "abort"` · `strip = true`. Biến môi trường `CI=true` (Tauri truyền `--skip-jenkins` xuống `bundle_dmg.sh`, bỏ bước AppleScript trang trí cửa sổ Finder — xem §Bẫy gặp thật).

---

## Tóm tắt: hai cửa đóng chặt, một cửa bàn giao, một rủi ro mới lộ ra

| # | Phép đo | Kết quả |
|---|---|---|
| 1 | Chênh lệch `.dmg` do font | 🟢 **20,300 MiB = 21,29 MB.** Tổng với database hiện tại = **151,29 MB**, lọt trần NFR6 |
| 2 | Chênh lệch `.msi` do font | 🟡 **Bàn giao sang Story 1.3** — `tauri-cli` trên macOS từ chối target `msi`. Ước **16,0–20,3 MiB** bằng phương pháp đã hiệu chuẩn |
| 3 | Giấy phép theo NFR15 | 🟢 **SIL OFL 1.1 cả ba, tương thích GPL v3** theo diện gộp gói. Đã mở từng tệp `LICENSE` mà đọc |
| 4 | Reserved Font Name | 🟢 **Ba tệp ba tình trạng** — đúng như `DESIGN.md` dự đoán, nay đã xác minh bằng tệp thật |
| 5 | Biến thể vùng TC/SC | 🟢 **Chọn TC.** Lý do có bằng chứng thị giác, và **chi phí đổi ý bằng 0** — hai tệp lệch nhau 1.176 byte |
| 6 | Dư địa còn lại dưới trần NFR6 | 🔴 **RỦI RO MỚI: chỉ còn ~47 MB** cho các nguồn từ điển còn lại **cộng toàn bộ mã sản phẩm.** Cần Ice quyết — xem §Cần Ice quyết |

> **Phỏng đoán bị bác (1):** `DESIGN.md` ước bộ font ≈ 21,6 MB trên đĩa, dựa trên phép chia zip 7 nét cho 7. Số thật là **25,99 MiB** — phần CJK là **23,41 MiB** chứ không phải ≈19 MB. Phép chia đều không đúng vì các nét không bằng nhau.
>
> **Phỏng đoán bị bác (2):** tôi giả định `.dmg` của Tauri nén bằng bzip2 (`UDBZ`). Thật ra là **`UDZO` — zlib/deflate**. Sai lầm này vô hại ở đây nhưng nó chính là thứ làm phép ước tính cho Windows trở nên kiểm chứng được — xem §Phép đo 2.

---

## Phép đo 1 — Chênh lệch `.dmg` trên macOS

Hai bản build **chỉ khác nhau đúng một biến**: cùng commit của mã thăm dò, cùng `tauri.conf.json`, cùng profile release, cùng toolchain, cùng `CI=true`. Bản thứ hai chồng thêm một tệp cấu hình khai đúng một khoá `bundle.resources`.

> ⚠️ **Đã có BA bản build, không phải hai — bản đầu của mục này không nói ra.** Hai bản A/B ở đây đóng gói **bốn** tệp font (bộ sẽ phát hành). Riêng phép so TC/SC ở §Phép đo 4 cần **cả hai** tệp CJK nên phải dựng thêm một bản thứ ba mang **năm** tệp — và **ảnh chụp đến từ bản thứ ba đó**, không phải từ bản B đã đo.
>
> Số học chứng minh chúng không thể là một: payload năm tệp là 51.796.264 byte, nhân tỉ lệ nén 0,781 rồi cộng baseline thì `.dmg` phải nặng **~39,9 MiB**. Bản B đo được **21,64 MiB**. Dựng riêng một bản để so dáng chữ là việc đúng; **không ghi ra** mới là lỗi, vì §Công thức đo trên Windows bảo Story 1.3 dựng lại theo mô tả này. **Story 1.3 đóng gói đúng bốn tệp** — thêm tệp SC vào là con số nhân đôi.

```bash
# A — baseline, không font
CI=true npm run tauri -- build --bundles dmg

# B — có font
CI=true npm run tauri -- build --bundles dmg --config src-tauri/tauri.fonts.conf.json
```

`src-tauri/tauri.fonts.conf.json` chứa đúng chừng này:

```json
{ "bundle": { "resources": { "resources/fonts/*": "fonts/" } } }
```

| Bản | Byte | MiB | MB | SHA-256 |
|---|---|---|---|---|
| **A** — `.dmg` không font | 1.402.311 | 1,337 | 1,402 | `7d4a359cf5448659d0cf36dc5de207087994ef702aafa495345af41ba2f380d7` |
| **B** — `.dmg` có font | 22.688.024 | 21,637 | 22,688 | `63774b507436bec04d5b11c41a95bb20838da043ec7ec4ffae66e282d3973208` |
| **Chênh lệch** | **21.285.713** | **20,300** | **21,286** | — |

**Dung lượng trên đĩa của payload font** (4 tệp thật sẽ phát hành): **27.253.184 byte = 25,991 MiB**. Tỉ lệ nén mà `.dmg` đạt được: **0,781**.

`hdiutil imageinfo` trên bản B: `Format: UDZO` · `Format Description: UDIF read-only compressed (zlib)`.

### Phép tính nghiệm thu AC1 — nguyên văn

> *"chênh lệch dung lượng + 130 MB database phải nằm trong trần 150–200 MB"*

> 📏 **Quy ước đơn vị — khai một lần, dùng cho mọi phép đối chiếu NFR6 về sau.** Trần NFR6 đọc theo **MB thập phân**, đúng đơn vị mà `prd.md:826` đang viết. Mọi phép đối chiếu **quy về byte trước** rồi mới đổi sang MB. **200 MB là trần**; 150 MB là mốc kỳ vọng chứ **không** phải điều kiện đạt — nhỏ hơn 150 MB là tốt, không phải trượt. *(Cách đọc "nằm trong dải" theo nghĩa đen của AC1 sẽ biến một bản cài gọn hơn thành một bản cài trượt, điều vô lý với một ngân sách dung lượng. Xem đề nghị tu chính PRD ở §Cần Ice quyết.)*

| Thành phần | byte | MB |
|---|---|---|
| Chênh lệch font (`.dmg`) | 21.285.713 | 21,29 |
| Database (Giai đoạn 0, **ba nguồn đầu tiên**) | 130.000.000 | 130,00 |
| **Tổng nghiệm thu AC1** | **151.285.713** | **151,29** |

**🟢 ĐẠT** — 151,29 MB nằm dưới trần 200 MB, còn dư **48,71 MB**.

> ⚠️ **Bản trước của mục này ghi 150,3 MB. Con số đó không tồn tại trong bất kỳ hệ đơn vị nào** — nó là kết quả cộng 20,30 **MiB** với 130 **MB** rồi dán nhãn MiB, và nó đã kịp lan sang sáu tài liệu trước khi bị bắt. Đọc đúng theo MiB thì 130 MB = 123,98 MiB và tổng là 144,28 MiB, tức vẫn đúng **151,29 MB**. Chỉ có **một** đại lượng thật, không phải hai cách đọc. **Số 150,3 đã được rút khỏi lưu thông ở cả sáu tài liệu.**

### Phép tính đầy đủ hơn

| Thành phần | byte | MB |
|---|---|---|
| Baseline app (không font, không database) | 1.402.311 | 1,40 |
| Chênh lệch font | 21.285.713 | 21,29 |
| Database (Giai đoạn 0, **ba nguồn đầu tiên**) | 130.000.000 | 130,00 |
| **Tổng** | **152.688.024** | **152,69** |

> ⚠️ **Baseline 1,40 MB là app một cửa sổ RỖNG, không phải AuraTranslate.** Nó không chứa một dòng mã sản phẩm nào — không panel, không IPC, không tầng dữ liệu. Toàn bộ phần thân ứng dụng thật vẫn còn phải nằm vừa trong dư địa còn lại. Đây là lý do §Cần Ice quyết đọc bài toán theo hướng **trừ dư địa** chứ không cộng lên trần.
>
> ⚠️ **130 MB là dung lượng SQLite trên đĩa, chưa nén.** Khi database thật xuất hiện ở Story 1.9 thì phải tính lại: `.dmg` cũng sẽ nén nó, nên con số cuối **thấp hơn** 152,69 MB. Đừng ai tưởng 152,69 MB là dung lượng tải về.

---

## Phép đo 2 — Chênh lệch `.msi` trên Windows: 🟡 BÀN GIAO SANG STORY 1.3

### Rào chặn, có bằng chứng

```
$ CI=true npm run tauri -- build --bundles msi --target x86_64-pc-windows-msvc
error: invalid value 'msi' for '--bundles [<BUNDLES>...]'
  [possible values: ios, app, dmg]
```

`tauri-cli` 2.11.4 chạy trên macOS **không nhận** `msi` làm bundle target — danh sách hợp lệ chỉ có `ios`, `app`, `dmg`. Đây không phải lỗi cấu hình mà là ranh giới của công cụ: `.msi` dựng bằng WiX v3, mà `candle`/`light` là chương trình Windows. Target `x86_64-pc-windows-msvc` **đã cài sẵn** trên máy này, nên rào chặn nằm ở tầng đóng gói chứ không ở tầng biên dịch Rust.

**AC1 đòi `.msi` (không phải NSIS), nên không có đường vòng nào ở tầng công cụ.**

> ✅ **Ice quyết 2026-08-03: bỏ phép đo này khỏi Story 1.1, chuyển sang Story 1.3.** AC1 đã thu hẹp ở `epics.md` và trong story file, bản gốc giữ dạng gạch ngang. Story 1.3 nhận một **AC mới** mô tả đúng phép đo, và ghi lại hai số ở **mỗi lần CI chạy** — nên nó còn bắt được hồi quy khi bộ font hay cấu hình WebView2 đổi, thứ mà một phép đo một lần không làm được. Công thức chạy vẫn giữ ở §Công thức đo trên Windows.
>
> **Ba lý do đo muộn là chấp nhận được:** ước 16,0–20,3 MiB lọt trần với biên rộng hơn macOS, và phương pháp ước đã tự kiểm sai số 0,1 %; rủi ro Windows thật sự nằm ở **chế độ cài WebView2** chứ không ở font, mà thứ đó chỉ CI bắt được; chi phí thêm khi CI đã chạy là gần bằng 0.

### Ước tính, bằng phương pháp đã tự hiệu chuẩn

Không đo được thì phải ước — nhưng ước có kiểm chứng chứ không đoán. Cách làm: nén chính payload font đó bằng các thuật toán tương ứng, rồi **kiểm phương pháp trên chính phép đo macOS đã có**.

| Thuật toán | Byte | MiB | Tỉ lệ |
|---|---|---|---|
| `zip -9` — deflate | 21.265.882 | 20,281 | 0,7803 |
| `bzip2 -9` | 19.312.344 | 18,418 | 0,7086 |
| `xz -9` — LZMA | 16.727.840 | 15,953 | 0,6138 |

> **Phương pháp tự kiểm:** `.dmg` hoá ra dùng **deflate** (`UDZO`), và tỉ lệ nén thật của nó là **0,7810**. Đại lượng thay thế `zip -9` cho **0,7803** — lệch **0,1 %**. Nghĩa là: nén rời cùng payload bằng cùng họ thuật toán dự đoán được chênh lệch installer với sai số dưới một phần trăm.
>
> ⚠️ **Nhưng phép tự kiểm này CHỈ hiệu chuẩn nhánh deflate.** Nó chứng minh `zip -9` dự đoán tốt cho `MSZIP` — nhánh **cận trên** — và **không nói gì** về nhánh `LZX` ở cận dưới, vốn lấy từ `xz -9`. Hai điều làm cận dưới 16,0 MiB đáng ngờ theo hướng **quá lạc quan**: `LZX` của CAB có cửa sổ tối đa **2 MiB**, còn `xz -9` dùng từ điển **64 MiB** — trên một tệp font 24,5 MB thì chênh lệch cửa sổ đó là chênh lệch lớn. Nói *"phương pháp ước đã tự kiểm sai số 0,1 %"* mà không khoanh phạm vi là mượn độ chính xác của một phép đo sang bảo chứng cho một phép ước chưa kiểm. **Cận trên đáng tin; cận dưới là phỏng đoán.**

MSI của WiX v3 nhúng một CAB. CAB có hai mức nén thường gặp — `MSZIP` (deflate) và `LZX` (họ LZ77 + Huffman, gần LZMA hơn). Chưa xác minh được Tauri đặt mức nào vì mã nguồn `tauri-bundler` chỉ tải về khi build cho Windows, nên đây là **một dải chứ không phải một số**:

| Giả thiết CAB | Chênh lệch `.msi` ước | Tổng với 130 MB database |
|---|---|---|
| `MSZIP` (deflate) — cận trên | ≈ **20,3 MiB** = 21,29 MB | ≈ **151,29 MB** |
| `LZX` mức cao — cận dưới | ≈ **16,0 MiB** = 16,73 MB | ≈ **146,73 MB** |

**Kết luận có điều kiện: Windows nhiều khả năng lọt trần NFR6 với biên rộng hơn macOS.** Phải xác nhận bằng số thật ở Story 1.3.

### Chế độ cài WebView2 đã dùng

`webviewInstallMode: { "type": "downloadBootstrapper" }` — mặc định, đã khai tường minh trong `tauri.conf.json` của app thăm dò. Chế độ này **triệt tiêu trong phép trừ** (cả hai bản build đều mang nó), nhưng nó quyết định con số **tuyệt đối**: `embedBootstrapper` hay `offlineInstaller` sẽ làm `.msi` phình thêm ~150 MB vì nhúng luôn một bản Chromium, và **một mình nó đủ đẩy tổng vượt trần NFR6** kể cả khi font bằng 0. Story 1.3 phải khai đúng `downloadBootstrapper` chứ không để mặc định ngầm.

---

## Phép đo 3 — Rà giấy phép theo NFR15

**Đã mở từng tệp `LICENSE` trong bản release đã tải mà đọc.** Không tin nhãn của GitHub, không chép kết luận từ trang web thứ ba — NFR15 đòi *rà soát tường minh*.

| Tệp giấy phép | Nguồn | SHA-256 | Kết luận |
|---|---|---|---|
| `LICENSE` (trong `09_NotoSerifCJKsc.zip` **và** `10_NotoSerifCJKtc.zip`, **giống hệt nhau**) | `notofonts/noto-cjk` tag `Serif2.003` | `6a73f9541c2de74158c0e7cf6b0a58ef774f5a780bf191f2d7ec9cc53efe2bf2` | SIL OFL 1.1 |
| `ofl/sourceserif4/OFL.txt` | `google/fonts` `main` | `5f94c3fd3a23131a417ab5a0c8452de57e70c3cfb9f604d88241f7065ebf9fd9` | SIL OFL 1.1 |
| `ofl/sourcesans3/OFL.txt` | `google/fonts` `main` | `09746787287a289323b0ec3cff4d1a4a801331b82b7207c1e186f5d26619a392` | SIL OFL 1.1 |

Cả ba đều mang nguyên văn *"SIL OPEN FONT LICENSE Version 1.1 - 26 February 2007"*.

### Kết luận tương thích GPL v3: **ĐẠT**

Quan hệ giữa font và mã là **gộp gói (aggregation)**, không phải tác phẩm phái sinh: font nằm cạnh mã trong bản cài, không liên kết vào mã. OFL-FAQ nói thẳng font OFL được đóng gói cùng phần mềm FLOSS, và FSF xếp OFL là giấy phép tự do tương thích với việc phân phối cùng GPLv2/v3/LGPL/AGPL.

**Ba ràng buộc kèm theo — không được lược khi ghi vào Stack:**

**1. Reserved Font Name — ba tệp, ba tình trạng khác nhau.** Đây là chỗ dễ suy sai nhất, nên ghi rõ từng tệp:

| Tệp | Dòng bản quyền đầu tiên | RFN? |
|---|---|---|
| `LICENSE` của Noto Serif CJK | *"This Font Software is licensed under the SIL Open Font License, Version 1.1."* — **không có dòng bản quyền nào** | ❌ **không khai** |
| `OFL.txt` của `sourceserif4` | *"Copyright 2014 The Source Serif 4 Project Authors (https://github.com/adobe-fonts/source-serif)"* | ❌ **không khai** |
| `OFL.txt` của `sourcesans3` | *"Copyright 2010-2020 Adobe (http://www.adobe.com/), **with Reserved Font Name 'Source'**. All Rights Reserved. Source is a trademark of Adobe…"* | ✅ **CÓ khai `'Source'`** |

Hệ quả có răng thật: nếu về sau subset để ghìm dung lượng, **chỉ riêng `SourceSans3[wght].ttf` bắt buộc đổi tên font nội bộ**, kéo theo chuỗi `"Source Sans 3"` trong `families.ui` của `DESIGN.md` phải đổi theo. Hai tệp kia subset thoải mái, không phải đổi tên. Đây chính là lợi thế mà kênh Google mang lại và là một trong hai lý do Ice chốt kênh Google — nay đã xác minh bằng tệp thật chứ không còn là suy đoán.

**2. Bản văn giấy phép đi kèm bản phát hành.** Miễn trừ *"font nhúng trong chương trình"* của OFL có tồn tại, nhưng dự án còn có màn hình Attribution (FR109) và ghi công trong bản phát hành (FR38). Mang theo cả ba tệp giấy phép gốc: rẻ, và bịt luôn câu hỏi ở Story 10.4/10.5. **Ba tệp** chứ không phải một — `LICENSE` của Noto Serif CJK giống hệt nhau ở cả bản TC và SC nên chỉ cần một bản sao cho phần CJK.

**3. Cấm bán riêng font.** OFL cấm bán Font Software một mình. AuraTranslate phát hành miễn phí theo GPL v3 nên ràng buộc này không chạm tới gì, nhưng nó vẫn ràng buộc mọi bản phái sinh về sau.

### Một chỗ lệch phiên bản cần biết

`Source Serif 4` trên kênh Google là **4.004**, trong khi bản Adobe mới nhất là **4.005R**. Đọc thẳng từ bảng `name` của tệp: `Version 4.004;hotconv 1.0.116`. Commit cuối chạm tệp đó trong `google/fonts` là `7b203a635ebe` ngày **2021-11-17** — kênh Google đi sau kênh Adobe một bản phát hành. `Source Sans 3` thì khớp: **3.052** hai bên như nhau.

Đây **không phải lý do đổi kênh**: 4.004 vẫn là bản ổn định, và hai lợi thế của kênh Google (không khai RFN cho Serif, font biến thiên nhẹ hơn) vẫn nguyên. Nhưng bảng Stack phải ghi **4.004**, không được chép 4.005R từ tài liệu cũ.

---

## Phép đo 4 — Chọn biến thể vùng: **TC**

### Bằng chứng

App thăm dò nạp **cả năm** tệp font qua asset protocol từ `$RESOURCE/fonts/**` và dựng cùng một nội dung hai lần, một lần với `NotoSerifCJKtc`, một lần với `NotoSerifCJKsc`.

- [`font-spike-2026-08-03/tc-vs-sc-glyphs.png`](font-spike-2026-08-03/tc-vs-sc-glyphs.png) — bảng 10 mã Hán dùng chung, hai cột
- [`font-spike-2026-08-03/zoom-glyphs-4-ma.png`](font-spike-2026-08-03/zoom-glyphs-4-ma.png) — phóng to 骨 · 直 · 房 · 令
- [`font-spike-2026-08-03/zoom-dau-cau.png`](font-spike-2026-08-03/zoom-dau-cau.png) — cùng một đoạn văn, hai biến thể
- [`font-spike-2026-08-03/tc-vs-sc-paragraph-and-latin.png`](font-spike-2026-08-03/tc-vs-sc-paragraph-and-latin.png) — đoạn văn + dải nét Latin

Khác biệt đọc được bằng mắt, ở **cùng một mã Unicode**:

| Mã | TC vẽ | SC vẽ |
|---|---|---|
| 骨 U+9AA8 | nét ngắn góc trên phải **hất sang trái**, ô rộng hơn | nét đó **hất sang phải**, ô hẹp hơn |
| 直 U+76F4 | nét ngang đáy **rời**, góc dưới trái để hở | nét dọc trái **nối liền** xuống nét ngang đáy |
| 房 U+623F | đầu là **戶** — chấm nằm ngang | đầu là **户** — chấm chếch rõ |
| 令 U+4EE4 | phần dưới có **nét sổ thẳng** | phần dưới kết bằng **chấm 丶** |

### Khác biệt quyết định lại không nằm ở dáng chữ, mà ở **vị trí dấu câu**

Xem [`zoom-dau-cau.png`](font-spike-2026-08-03/zoom-dau-cau.png): TC đặt 「，」 và 「。」 **giữa ô chữ**; SC đặt chúng ở **góc dưới bên trái**, để lại một khoảng trống thấy rõ phía trên.

**Đây mới là thứ đáng cân nhắc, vì nó xuất hiện ở mọi dòng**, không phải chỉ ở vài mã hiếm. Panel Nguyên văn là nơi mắt người dịch ở lâu nhất trong sản phẩm (`DESIGN.md` §Typography nói đúng điều này khi lập luận vì sao phải nhúng font). Một quy ước đặt dấu câu sai mạch sẽ gặm mòn suốt cả phiên làm việc.

### Chốt: **`NotoSerifCJKtc-Regular.otf`**

Lý do, theo thứ tự sức nặng:

1. **Phạm vi dự án là dịch thuật tổng quát, không phải ngách truyện mạng.** Ice đã chốt điều này ở giai đoạn brief và bác thẳng giả định ngách — *"đây là công cụ dịch thuật TỔNG QUÁT; tiên hiệp/công pháp trong PRD chỉ là ví dụ minh hoạ"*. Lập luận *"SC hợp truyện mạng đương đại"* trong `DESIGN.md` vì thế mất trọng lượng: truyện mạng không còn là trung tâm.
2. **Hai lớp từ điển của chính sản phẩm nghiêng về phồn thể.** Cổ hán văn (Tam tự kinh, Thiên tự văn, Bách gia tính) và Hán Việt Từ Điển Trích Dẫn đều là ngữ liệu cổ văn. Panel Source có hẳn một tab Hán Việt: người dùng đối chiếu âm Hán Việt với mặt chữ, và mặt chữ phồn thể là mặt chữ mà âm Hán Việt bám vào.
3. **Quy ước dấu câu của TC hợp mạch cổ văn** — xem trên.
4. **Không mất gì về phủ mã *so với SC*.** Đây là biến thể vùng **đầy đủ**, không phải bản subset theo ngôn ngữ: TC và SC phủ **cùng một** kho mã, chỉ khác dáng chữ ưu tiên. Người dùng nhập một chương giản thể vào bản cài dùng TC vẫn thấy đủ chữ. Chọn TC thay SC **không** thêm một ca ô vuông rỗng nào.

   > ⚠️ **Đừng đọc thành "không bao giờ có ô vuông rỗng" — bản đầu của mục này viết vậy và nó sai.** Noto Serif CJK phủ CJK Unified Ideographs, Extension A và một phần Extension B, nhưng **không** phủ trọn Extension B/C/D/E, cũng không phủ chữ Nôm hay các dị thể hiếm. Cổ văn là **một trong hai lớp từ điển của chính sản phẩm**, mà cổ văn là đúng chỗ những mã đó xuất hiện — nên ô vuông rỗng có thể gặp **ngay hôm nay**, chỉ là nó không liên quan gì tới lựa chọn TC/SC. Cách xử lý mã ngoài phủ chưa ai nhận; xem §Việc chưa làm được.

### Chi phí đổi ý bằng 0 — ghi rõ để Ice biết mình không bị khoá

| | TC | SC | Lệch |
|---|---|---|---|
| `NotoSerifCJK*-Regular.otf` | 24.541.904 byte | 24.543.080 byte | **1.176 byte** |

Đổi biến thể là **thay một tệp và sửa một chuỗi**, không đổi ngân sách, không đổi kiến trúc, không dựng lại gì. Nếu dùng thật một thời gian mà thấy SC hợp hơn thì đổi trong một story nhỏ.

> ⚠️ **Bẫy đặt tên đã kiểm và né đúng.** Dùng `NotoSerifCJKtc` (biến thể vùng đầy đủ, 132,36 MB zip) chứ **không** phải `NotoSerifTC` (subset theo ngôn ngữ, 45,07 MB zip — tương đương `SourceHanSerifTW` của Adobe, **không** tương đương `SourceHanSerifTC`). Dung lượng asset trên release API xác nhận đúng cặp ánh xạ mà `DESIGN.md` đã cảnh báo.

---

## Phép đo 5 — Bộ tệp thật sẽ đóng gói

| Tệp | Byte | MiB | SHA-256 |
|---|---|---|---|
| `NotoSerifCJKtc-Regular.otf` | 24.541.904 | 23,405 | `234301038e76e7c35c43113785024700c4e4fe7bdce1d1fbbc42fca7e6683798` |
| `SourceSerif4[opsz,wght].ttf` | 1.209.508 | 1,153 | `97b2d4da6e3cb494b5a1e66ae176914d852ccabef49e0c02c0df25f3e39aca0b` |
| `SourceSerif4-Italic[opsz,wght].ttf` | 855.432 | 0,816 | `15fbc7e4679489a501998c3669272637a6646388ef7e4bd77eebb5bf967a1f42` |
| `SourceSans3[wght].ttf` | 646.340 | 0,616 | `042fe2cc0b933e328410d7acbd0aa6a1873dca5aef81875f4bc214b08825c7b9` |
| **Tổng** | **27.253.184** | **25,991** | — |

Zip nguồn đã tải, để không ai lẫn tệp:

| Asset | Byte | SHA-256 |
|---|---|---|
| `09_NotoSerifCJKsc.zip` | 138.631.496 | `4bcdbff95cedfb6a4c0640403f0de8b69480d869331c24c8eff91f7bb834df04` |
| `10_NotoSerifCJKtc.zip` | 138.791.400 | `b4aa07b217532c5859b3674d53588671e7e4f340054fc30e9bf417ee3b1aa4d4` |

### Bảng `name` và trục biến thiên, đọc thẳng từ tệp

| Tệp | Family (name ID 1) | Typographic Family (ID 16) | Version |
|---|---|---|---|
| `NotoSerifCJKtc-Regular.otf` | `Noto Serif CJK TC` | — | 2.003 |
| `SourceSerif4[opsz,wght].ttf` | `Source Serif 4` | — | 4.004 |
| `SourceSerif4-Italic[opsz,wght].ttf` | `Source Serif 4` | — | 4.004 |
| `SourceSans3[wght].ttf` | **`Source Sans 3 ExtraLight`** | `Source Sans 3` | 3.052 |

| Tệp | Trục | min | mặc định | max |
|---|---|---|---|---|
| `SourceSerif4[opsz,wght].ttf` | `wght` / `opsz` | 200 / 8 | **400** / 20 | 900 / 60 |
| `SourceSerif4-Italic[opsz,wght].ttf` | `wght` / `opsz` | 200 / 8 | **400** / 20 | 900 / 60 |
| `SourceSans3[wght].ttf` | `wght` | 200 | **200** | 900 |

> **Một phỏng đoán nữa được nêu rồi bị bác ngay tại chỗ.** Thấy `SourceSans3[wght].ttf` có mặc định trục `wght` = **200** và name ID 1 ghi `Source Sans 3 **ExtraLight**`, tôi nghĩ mọi chữ giao diện không khai `font-weight` sẽ ra nét mảnh 200 — một lỗi hỏng im lặng đúng kiểu đáng sợ. **Bản dựng thật bác điều đó:** CSS luôn áp `font-weight: normal` (= 400) làm giá trị khởi đầu, nên trục được ghim ở 400 và mặc định của `fvar` không bao giờ được dùng tới. Xem dòng `Source Sans 3 · giao diện` trong ảnh chụp — nét bình thường, không mảnh.
>
> Mặc định 200 **vẫn** quan trọng với thứ đọc font ngoài đường CSS: bảng chọn font của hệ điều hành, công cụ dựng `.docx` ở Epic 8, bất kỳ nơi nào không phát biểu nét. Ghi lại để Story 8.3 không vấp.

**Nét Latin 600 và 700 là nét thật — nhưng ảnh chụp chỉ chứng minh được một nửa mệnh đề đó.**

Ảnh dựng `Source Serif 4` thành **năm dòng riêng** ở 200 / 400 / 600 / 700 / Italic, đậm nhạt khác nhau rõ — nên token **`read-title` (600, họ `read`) đã kiểm xong**. Nhưng `Source Sans 3` chỉ được dựng **một dòng duy nhất ở một nét**; chuỗi "400 / 600 / 700" trên dòng đó là **chữ viết ra**, không phải ba mẫu nét dựng thật.

> ⚠️ **Nghĩa là token `ui-label` (700, họ `ui` = `Source Sans 3`) CHƯA từng được kiểm** — và nó nằm trên đúng tệp có mặc định trục `wght = 200` và `name ID 1 = Source Sans 3 ExtraLight`, tức đúng tệp đáng ngờ nhất. **Story 1.4 phải dựng `Source Sans 3` ở 400/600/700 rồi mới coi mệnh đề này là đã kiểm.** Ngoài ra `FontFace` trong app thăm dò đăng ký font biến thiên **không khai descriptor `weight`**; với bản thật nên khai `{ weight: "200 900" }` để chắc trình duyệt mở trọn trục thay vì tự tổng hợp.

**Phần Hán vẫn chỉ có Regular — và ca nghiêng giả ĐÃ phát sinh, không phải chưa.**

> ⚠️ **Bản đầu của mục này soát sai tiêu chí.** Nó kiểm *"có token nào vừa khai họ `read-cjk` vừa đòi nét đậm hay nghiêng không"*, thấy không có, và kết luận ca này chưa phát sinh. Nhưng điều kiện sinh lỗi không phải **token khai họ nào** — mà là **ký tự CJK được dựng dưới một token nghiêng hoặc đậm bất kỳ**. Mà `families.read` có `Noto Serif CJK TC` nằm trong chuỗi dự phòng, nên chữ Hán rơi vào nó **qua fallback** dù token khai họ `read`.
>
> Hai token đang dính, cả hai `italic`, cả hai họ `read`: **`source-hanviet` (token 6)** và **`lookup-example` (token 10)**. Token 10 là *"Ví dụ và trích dẫn"* của Panel Lookup — với từ điển Trung–Việt thì ví dụ **chắc chắn** chứa chữ Hán, ở cỡ 12,5px, cỡ mà nghiêng giả xấu nhất. Phát hiện khi rà soát 2026-08-03.
>
> **Hướng xử lý thuộc Story 1.4** (story sở hữu bộ token): hoặc chấp nhận nghiêng giả cho phần Hán, hoặc khai `font-style: normal` cho ký tự CJK trong hai token đó. **Thêm một tệp nghiêng CJK không phải phương án** — đó là thêm **~23 MiB**, một phần ba ngân sách font hiện tại, chứ không phải thêm một dòng CSS.

---

## Cấu hình Tauri đã kiểm chứng — Story 1.2 chép lại đúng chỗ này

App thăm dò đi đúng đường mà AD-23 đã chốt (`$RESOURCE/fonts/**` chỉ đọc), **không** đi đường tắt qua `src/assets/`. Nghĩa là con số ở Phép đo 1 là chênh lệch ở **tầng resource**, đúng tầng mà bản thật sẽ dùng.

**`tauri.conf.json`:**

```json
"security": {
  "csp": "default-src 'self'; script-src 'self'; style-src 'self' 'unsafe-inline'; font-src 'self' asset: http://asset.localhost; img-src 'self' asset: http://asset.localhost data:",
  "assetProtocol": { "enable": true, "scope": ["$RESOURCE/fonts/**"] }
}
```

**`Cargo.toml`:** `tauri = { version = "2.11.5", features = ["protocol-asset"] }`. Feature này **bắt buộc** khi bật `assetProtocol` — `tauri-build` tự thêm vào và báo lỗi nếu thiếu.

**Frontend nạp font:**

```js
const path = await resolveResource(`fonts/${file}`);
const face = new FontFace(family, `url("${convertFileSrc(path)}")`);
await face.load();
document.fonts.add(face);
```

> **`font-src asset:` KHÔNG phải nới CSP theo nghĩa AD-15 cấm.** AD-15 cấm **origin từ xa** — CDN, font ngoài, ảnh ngoài. Asset protocol là tài nguyên cục bộ đã nằm trong bản cài; không có một byte nào ra mạng. Cả năm tệp font nạp thành công trong bản build release, thấy trên ảnh chụp.

---

## Bẫy gặp thật khi làm

**1. `bundle_dmg.sh` chết ở bước AppleScript khi không có phiên Finder tương tác.**

```
execution error: Finder got an error: Can't set Finder window id … to 128. (-10006)
Failed running AppleScript
```

Bước đó chỉ trang trí vị trí icon trong cửa sổ `.dmg`. Đặt `CI=true` là Tauri truyền `--skip-jenkins` cho `bundle_dmg.sh` và bỏ hẳn bước này. **Story 1.3 sẽ gặp đúng lỗi này trên runner GitHub Actions** — ghi lại để không mất một buổi lần mò. Cái giá: bản `.dmg` không có bố cục Finder đặt sẵn, thuần tuý thẩm mỹ.

**2. Khai `[lib]` trong `Cargo.toml` mà không có `src/lib.rs` thì `cargo metadata` gãy**, và Tauri CLI dừng trước cả khi biên dịch: *"can't find library `font_spike_lib`"*. App một tệp `main.rs` thì bỏ hẳn khối `[lib]`.

**3. Số `tauri` của crate và của CLI npm không đi cùng nhau.** Crate `tauri` có 2.11.5; `@tauri-apps/cli` mới nhất là **2.11.4**. Bảng Stack ghim 2.11.5 là ghim **crate**, đúng. Đừng cố `npm i @tauri-apps/cli@2.11.5` — không tồn tại.

**4. `tauri build` xoá `.app` sau khi đóng gói `.dmg`.** Muốn giữ cả hai để soi bên trong thì build `--bundles app` riêng một lượt.

---

## Việc chưa làm được ở mũi thăm dò này

- **Chênh lệch `.msi` trên Windows** — công cụ từ chối, xem §Phép đo 2. Đóng ở Story 1.3.
- **Mức nén CAB mà Tauri đặt cho WiX** — chưa xác minh được vì mã nguồn `tauri-bundler` chỉ tải về khi build cho Windows. Đó là lý do phép ước cho Windows là một dải chứ không phải một số.
- **Hình chữ ở cỡ nhỏ thật** — đã dựng ở 16–26px trong app thăm dò, nhưng chưa đối chiếu với bảng 14 token của `DESIGN.md` trên bố cục bốn panel thật. Việc đó thuộc Story 1.4.
- **Trục `opsz` của Source Serif 4** (8–60, mặc định 20) chưa hiệu chỉnh. `font-optical-sizing: auto` là mặc định của webview và sẽ tự bám theo `font-size`; chưa kiểm xem nó có hợp với ba mức cỡ chữ của Chế độ đọc không. Story 1.4 / 5.11.
- **Không đo trên Apple Silicon.** Máy đo là Intel x86_64. Chênh lệch font là dữ liệu thuần, gần như chắc chắn không đổi theo kiến trúc, nhưng baseline 1,34 MiB thì có.

---

## Cần Ice quyết

### 🔴 Dư địa dưới trần NFR6 chỉ còn ~47 MB — và đây là quyết định tầng PRD

> ⚠️ **Bản trước của mục này đọc sai `[A2]` và phải viết lại.** Nó hiểu *"ngân sách 150–200 MB"* thành **dự báo dung lượng database**, rồi cộng font lên trên con số đó ra 220,3 MB và gắn nhãn 🔴. Nhưng `prd.md:826` viết *"**NFR6** | Kích thước **bản cài** kèm toàn bộ từ điển | `[A2]` Ngân sách **150–200 MB**"* và `prd.md:1020` viết *"**A2** | Ngân sách 150–200 MB **đủ cho** toàn bộ nguồn từ điển"*. 150–200 MB là **trần của cả bản cài, đã bao gồm font** — nó không dự báo gì về riêng database. Cộng font vào cái trần đó là cộng font hai lần.

Phép tính đúng là **trừ dư địa**, không phải cộng lên trần:

| | MB |
|---|---|
| Trần NFR6 | **200,00** |
| − Bộ font (đã đo thật) | 21,29 |
| − Baseline app *(hôm nay là app một cửa sổ **rỗng**, **chưa có** mã sản phẩm)* | 1,40 |
| **= Dư địa cho toàn bộ dữ liệu và mã** | **177,31** |
| − Đã dùng cho **ba nguồn đầu tiên** (Giai đoạn 0) | 130,00 |
| **= CÒN LẠI** | **47,31** |

**47,31 MB đó phải chứa cả ba thứ sau:**

1. **Các nguồn từ điển còn lại.** Bao nhiêu nguồn thì hiện **chưa thống nhất** giữa các tài liệu — xem mục ngay dưới.
2. **Chỉ mục FTS phụ.** `prd.md:832` đo được **~17 MB mỗi chỉ mục**, và NFR8 đòi lập chỉ mục hai lần. Chưa xác định được 130 MB của Giai đoạn 0 đã gồm chỉ mục nào chưa.
3. **Toàn bộ mã sản phẩm AuraTranslate** — bốn panel, tầng dữ liệu, module AI, đường nhập, Library, TM. Hôm nay con số đó bằng **0** trong mọi phép tính đã làm.

**Đọc cho đúng:** bộ font **không** làm vỡ trần và không phải thủ phạm. Nó ăn 21,29 MB trong 70 MB dư địa mà `[A2]` chừa ra bên trên mức 130 MB hiện tại. Câu hỏi thật là liệu 47 MB còn lại có gánh nổi phần dữ liệu chưa nạp **cộng với** cả một ứng dụng chưa viết hay không — và hôm nay chưa ai biết.

**Đây đúng loại quyết định mà AC4 nói tới: thay đổi ở tầng PRD, không phải tối ưu ở tầng kiến trúc.** Tôi **không** tự subset font, **không** bỏ họ font nào, **không** đổi sang font hệ điều hành — cả ba đều là quyết định tầng PRD/thiết kế.

### 🔴 Ba tài liệu đang nói ba danh sách nguồn từ điển khác nhau

Phát hiện khi rà soát 2026-08-03. Mục này **không phải** việc của mũi thăm dò font, nhưng nó chặn phép tính dư địa ở trên nên phải nêu:

| Nơi | Danh sách nguồn |
|---|---|
| `prd.md:1020` — giả định `[A2]` | Unihan · Thiều Chửu · Cổ hán văn · VietPhrase |
| `epics.md` — Story 1.9 | CVDICT · Unihan · CC-CEDICT · viwiktionary · en.wiktionary |

Chỉ **Unihan** có mặt ở cả hai. *(Bản trước của báo cáo này còn tự thêm **HVTĐTD** vào danh sách `[A2]` — sai, vì theo **AD-10** HVTĐTD là lớp **gỡ rời** không thuộc GPL v3, nên nó thuộc Story 1.10 chứ không phải 1.9. Danh sách bịa đó đã được gỡ.)*

Không hợp nhất được ba danh sách thì **không tính được** dư địa 47 MB đủ hay thiếu. Hợp nhất chúng là việc tầng PRD, không phải việc của story này.

### 📏 Đề nghị tu chính PRD — một dòng, để Ice cầm sang `bmad-prd`

NFR6 hiện viết *"Ngân sách 150–200 MB"* mà **không khai đơn vị là MB hay MiB** (ở mốc 200 thì hai cách đọc lệch **7 %**, đủ để lật một phép nghiệm thu), và **không khai đó là trần hay là dải**. AC1 của Story 1.1 diễn giải thành *"phải **nằm trong** trần 150–200 MB"*, tức đọc theo nghĩa đen thì một bản cài **gọn hơn** 150 MB sẽ bị tính là **trượt**.

Đề nghị NFR6 nói rõ: **trần 200 MB thập phân (200.000.000 byte); 150 MB là mốc kỳ vọng, không phải điều kiện đạt; mọi phép đối chiếu quy về byte trước.** Story này **không tự sửa PRD** — `§Ranh giới phạm vi` cấm, và đây đúng là tầng mà AC4 dành cho chủ dự án.

### Các đòn bẩy có thật, kèm cái giá

| Đòn bẩy | Được | Mất |
|---|---|---|
| **Không làm gì với font bây giờ**, đo lại tổng thật khi database xong (Story 1.9) | Không tiêu công vào bài toán chưa chắc có; font không phải thủ phạm | Nếu vỡ thì vỡ muộn, lúc giao diện đã dựng trên bộ font này |
| Nới trần NFR6 lên 250 MB | Không mất gì kỹ thuật; 250 MB vẫn bình thường với desktop app 2026 | Đổi lời hứa sản phẩm — **đúng loại quyết định AC4 nói tới** |
| Tự subset `NotoSerifCJKtc` theo kho mã thật cần | Nhiều nhất, có thể vài chục MB | Phải dựng và bảo trì pipeline subset; mất khả năng hiện mã hiếm trong cổ văn — mà cổ văn là một trong hai lớp từ điển của chính sản phẩm. **Không phải đổi tên font** vì bản Noto không khai RFN |
| Dùng bản subset theo ngôn ngữ (`NotoSerifTC`, 45 MB zip) | ~10–12 MiB | **Tofu im lặng** khi nhập văn bản giản thể. Đã bác ở §Phép đo 4 |
| Bỏ `SourceSerif4-Italic` | 0,82 MiB trên đĩa, ~0,6 MiB sau nén | Hai token `source-hanviet` và `lookup-example` đang dùng nghiêng thật; mất tệp này là rơi sang nghiêng giả. Rẻ quá ít so với cái giá |
| Gỡ một lớp từ điển ra khỏi bản cài, tải sau | Nhiều | Phạm `[A2]` — *"không tải thêm sau khi cài"*. Đây là lời hứa sản phẩm, không phải chi tiết kỹ thuật |

**Khuyến nghị của tôi: chọn hàng đầu tiên.** Đo lại tổng thật ở Story 1.9. Lý do: hôm nay chưa biết các nguồn còn lại nặng bao nhiêu — mà chính danh sách nguồn cũng chưa thống nhất — nên mọi đòn bẩy khác đều là tối ưu mù, tốn công hoặc đổi lời hứa sản phẩm.

> ✅ **Việc cần làm ngay đã làm xong, không còn là khuyến nghị treo.** Story 1.9 nay mang **một AC mới** trong `epics.md`: cộng dung lượng font và baseline app thật vào tổng **mọi artifact dữ liệu sẽ đóng gói** — `dict-core.db` **và** bốn lớp gỡ rời của Story 1.10, chứ không chỉ `dict-core.db` — rồi đối chiếu lại trần NFR6. *(Bản trước của báo cáo chỉ **đề nghị** làm việc này rồi năm tài liệu khác chép lại thành "đối chiếu lại ở Story 1.9" như thể nó đã được lên lịch — trong khi Story 1.9 chưa hề có AC nào như vậy. Rà soát 2026-08-03 bắt được và đã ghi thành AC thật.)*

### ✅ Việc thứ hai đã đóng: `.msi` chuyển sang Story 1.3

Ice quyết ngày 2026-08-03. AC1 đã thu hẹp ở `epics.md` và trong story file; Story 1.3 nhận một AC mới. Công thức ở §Công thức đo trên Windows — **không còn là việc cần Ice làm tay**.

---

## Công thức đo trên Windows

*(Dành cho Story 1.3. Đặt ở đây chứ không chỉ trong thư mục tạm, để công thức không mất khi scratchpad bị dọn.)*

Gói mang đi **không bắt buộc** — CI dựng từ cây nguồn thật. Nhưng nếu muốn dựng lại đúng phép đo này bằng đúng bộ font đã đo trên macOS thì có sẵn `font-spike-windows.zip` (20 MB, SHA-256 `776e8d06dca6210fded432e7baec6505813571afabcf7c5435e09afda1b07af2`) trong scratchpad của phiên 2026-08-03.

**Cần có:** Rust toolchain `stable-x86_64-pc-windows-msvc` · Visual Studio Build Tools kèm workload *Desktop development with C++* · Node.js 20+ · WiX v3 (Tauri CLI tự tải lần build đầu).

```powershell
npm install

# A — baseline, KHÔNG font
$env:CI="true"; npm run tauri -- build --bundles msi

# B — CÓ font
$env:CI="true"; npm run tauri -- build --bundles msi --config src-tauri/tauri.fonts.conf.json
```

⚠️ **Đọc số của bản A trước khi chạy lệnh B** — hai lệnh ghi ra cùng một đường dẫn, B sẽ đè lên A.

⚠️ **Chiều của phép trừ sẽ ĐẢO khi chạy trên cây nguồn thật.** Công thức trên viết cho app thăm dò, nơi mặc định là **không** font và bản B *chồng thêm* `bundle.resources`. Nhưng từ Story 1.2 trở đi font **nằm sẵn** trong `tauri.conf.json` của bản thật — nên bản baseline phải dựng bằng cách **gỡ** khoá `bundle.resources` ra, không phải thêm vào. Cụ thể: giữ `tauri.conf.json` làm bản B (có font), và dùng một tệp `--config` khai `{ "bundle": { "resources": {} } }` để dựng bản A. Không đảo chiều thì Story 1.3 dựng ra hai bản **giống hệt nhau** và chênh lệch bằng 0 mà không lỗi nào được ném.

```powershell
(Get-Item "src-tauri\target\release\bundle\msi\*.msi").Length
```

**Ba điều phải giữ để số đo có nghĩa:**

1. **Không sửa gì giữa hai lệnh.** Bản B chỉ chồng thêm một tệp cấu hình chứa đúng một khoá `bundle.resources` — đó là biến duy nhất được phép khác.
2. **`CI=true` là bắt buộc**, không phải trang trí: nó giữ hành vi đóng gói khớp với bản macOS đã đo.
3. **WebView2 để nguyên `downloadBootstrapper`** (đã khai tường minh trong `tauri.conf.json`). Đổi sang `embedBootstrapper` hay `offlineInstaller` là **một mình nó đủ đẩy tổng vượt trần NFR6** kể cả khi font bằng 0.

**Cần ghi lại:** số byte bản A · số byte bản B · `rustc --version` · `npx tauri --version` · `winver`.

**Đối chiếu tại chỗ:** nếu chênh lệch rơi trong dải ước **16,0–20,3 MiB** thì phép ước ở §Phép đo 2 đúng. **Rơi ngoài dải mới là phát hiện đáng ghi** — khi đó phải xem lại mức nén CAB mà Tauri đặt cho WiX, thứ chưa xác minh được từ macOS.

---

## Mệnh đề chặn Epic 1 — đã gỡ

AC4 nói *"không story nào của Epic 1 bắt đầu trước khi kết quả này được ghi lại"*. Kết quả **đã được ghi**: tài liệu này, ba hàng font trong bảng Stack, hai hàng Deferred đã đóng, `DESIGN.md` và `EXPERIENCE.md` đã cập nhật. **Epic 1 chạy tiếp được.**

Hai việc bàn giao, **không việc nào chặn**:

1. **Story 1.3** — đo `.msi` có/không font, ghi lại ở mỗi lần phát hành, kèm chế độ cài WebView2 đang dùng. Đã thành AC trong `epics.md`. Công thức ở §Công thức đo trên Windows.
2. **Story 1.9** — cộng 20,30 MiB font vào database thật rồi đối chiếu lại NFR6.

---

## Mã nguồn của mũi thăm dò

Nằm trong thư mục scratchpad của phiên làm việc, **không đưa vào repo** — mã dùng một lần, đúng tiền lệ Giai đoạn 0. Story 1.2 dựng cây nguồn thật từ đầu và mang AC nguyên văn *"không dùng bất kỳ starter template cộng đồng nào"*; để lại app này trong repo là mời người ta vi phạm chính AC đó.

- `font-spike/app/` — app Tauri v2 một cửa sổ: `src/main.js` (nạp font qua asset protocol, dựng bảng so sánh TC/SC), `src-tauri/tauri.conf.json` (CSP + assetProtocol scope), `src-tauri/tauri.fonts.conf.json` (đúng một khoá `bundle.resources`)
- `font-spike/downloads/` — zip gốc và các tệp đã lấy ra, kèm SHA-256
- `font-spike/measurements/` — hai bản `.dmg`, ảnh chụp, số liệu thô

Ảnh chụp đã sao vào `research/font-spike-2026-08-03/` để đi cùng báo cáo.
