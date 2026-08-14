# Bàn đo Story 2.5 — hai vạch lề cùng tồn tại (Quyết định #2)

Lưu ở đây để **không phải đo lại từ đầu**. Món nợ gốc: `deferred-work.md:2052-2064`, chủ Story 2.5.

⚠️ Đây là **tạo tác của một lượt đo**, không phải công cụ của dự án. Không có gì ở đây vào
`package.json` — đúng AC16, và đúng khuôn `2-2-ban-do/` · `2-3-ban-do/` · `2-4-ban-do/`.

## Chạy

```sh
# Playwright CỐ Ý nằm ngoài cây nguồn — cửa NFR15 không mở cho một tạo tác bàn đo.
PW=/tmp/pw-aura; mkdir -p $PW && (cd $PW && npm init -y && npm i playwright@1.62.1)
npx --yes playwright@1.62.1 install chromium webkit     # một lần

AURA_PW=$PW/node_modules/playwright/index.mjs \
  node _bmad-output/implementation-artifacts/2-5-ban-do/chup.mjs
```

Ra: `2-5-hai-vach-{blink,webkit}-{light,dark}.png` · `bao-cao.json`.

## Số đo — 2026-08-14, Playwright 1.62.1 (Chromium 1234 · WebKit 2336), macOS 15.6

Hai engine × hai theme cho **cùng một** số (WebKit lệch ±0,04 px do làm tròn hình học).
Bảng dưới lấy nhánh `blink-light`; ba nhánh kia ở `bao-cao.json`.

### Fixture ① — văn xuôi trộn (9 câu, 5 dòng có vạch)

| Đường | vẽ ra | vị trí phân biệt | **bị che** | làn cần | bước làn | mép phải | tràn máng 22px | khúc thấp nhất | dòng nói dối |
|---|---|---|---|---|---|---|---|---|---|
| hiện trạng | 7 | 6 | **1** | 1 | — | 10px | không | 17,00px | 0 |
| **(a) chia làn** ✅ Ice ký | 7 | 7 | **0** | 3 | 5px | 20px | không | 17,00px | 0 |
| (b) một vạch | 5 | 5 | 0 | 1 | — | 10px | không | 17,00px | **4/5** |
| (c) chia dọc | 10 | 10 | 0 | 1 | — | 10px | không | **5,67px** | 0 |

### Fixture ② — đoạn ĐỐI THOẠI (12 câu ngắn, 3 dòng có vạch, dòng đông nhất **5 câu**)

| Đường | vẽ ra | vị trí phân biệt | **bị che** | làn cần | bước làn | mép phải | tràn máng 22px | khúc thấp nhất | dòng nói dối |
|---|---|---|---|---|---|---|---|---|---|
| hiện trạng | 11 | 5 | **6** *(55 %)* | 1 | — | 10px | không | 17,00px | 0 |
| **(a) chia làn** ✅ Ice ký | 11 | 11 | **0** | **5** | **3px** | 22px | không | 17,00px | 0 |
| (b) một vạch | 3 | 3 | 0 | 1 | — | 10px | không | 17,00px | **3/3** |
| (c) chia dọc | 13 | 13 | 0 | 1 | — | 10px | không | **3,40px** | 0 |

## 🔴 Kết luận đo được — hai lượt, và lượt đầu BÁC đề xuất ban đầu

**Lượt đo ①** chạy đường (a) với **bước làn cố định 5px** — đúng như §Quyết định #2 của story
mô tả nó *("làn trong 8px, làn ngoài 13px")*. Kết quả: fixture ① cần **3** làn *(vừa khít)*,
fixture ② cần **5** làn ⇒ mép phải **30px** ⇒ **tràn khỏi máng**, và 30px đúng bằng chỗ chữ bắt
đầu (`22 + padding-left 8`). ⇒ **(a) với bước cố định KHÔNG đóng kín.**

Ice đọc số đó và **vẫn ký (a)** ngày 2026-08-14.

**Lượt đo ②** chạy đường (a) với **bước làn CO cho vừa máng** — `buoc = clamp(2, 5, ⌊12/(N-1)⌋)`.
Fixture ② khi đó dùng bước **3px**, làn ngoài cùng ở `left: 20px`, mép phải **22px**, `tràn =
false`, **0 vạch bị che**. ⇒ Phản đối đo được của lượt ① **được đóng**, và nó đóng **trong** đường
Ice đã ký chứ không bằng một đường thứ tư.

- Đối thoại **không** phải ca biên bịa ra. Nó là hình dạng văn bản thường nhật của thể loại
  AuraTranslate nhắm tới, và nó là ca **tệ nhất** cho hiện trạng: **6/11 vạch bị che**.
- ⚠️ **GIỚI HẠN THẬT của lời giải đã ký:** từ **8 làn** trở lên không lời giải nào trong máng
  22px *(bước tối thiểu 2px = bề rộng vạch)*, và luật lúc đó là dồn về làn cuối, tức chấp nhận
  che. Nó đòi **8 câu cùng một dòng**; fixture đối thoại dày nhất mới cho **5**. Món nợ có chủ
  trong `deferred-work.md`.
- ⚠️ **Một vết trong chính bàn đo này, ghi ra thay vì sửa im lặng:** bản đầu gom vạch chồng
  nhau **bắc cầu** rồi phát làn theo thứ tự trong nhóm. Lượt chạy đầu bác nó ngay — một nhóm
  năm phần tử dồn cả bốn phần tử sau vào làn 1, nên (a) vẫn còn một vạch bị che. Phép đúng là
  **tô màu đồ thị khoảng** *(mỗi vạch nhận làn nhỏ nhất chưa bị một vạch chồng nào chiếm)*.
  Đây là đúng lớp lỗi *"trúng tiền đề chưa phải trúng cơ chế"* mà §Bài học của story ghi.

## Hằng phải ĐO mới ra — đừng đoán lại

| Hằng | Giá trị | Ra bằng cách nào |
|---|---|---|
| Một dòng Editor cao | **17,00px** Blink · **17,04px** WebKit | `getClientRects()`; ⚠️ **không** phải 15 × 1,95 = 29,25px — bàn đo chạy font `serif` hệ thống, không ba font nhúng của UX-DR4 |
| Bước làn tối đa | **5px** | 2px vạch + 3px khe |
| Bước làn tối thiểu | **2px** | bằng bề rộng vạch — hai vạch chạm nhau |
| Số làn máng 22px chứa được, bước **cố định** 5px | **3** | làn k ở `left = 8 + 5k`; làn 2 hết ở 20px, làn 3 ở 25px ⇒ tràn |
| Số làn máng 22px chứa được, bước **co** | **7** | `buoc = ⌊12/(N-1)⌋`; N=7 ⇒ bước 2, làn 6 ở 20px, mép phải 22px |
| Mép trái chữ | **30px** | `gutter-width 22` + `.doc { padding-left: 8px }` |

## Giới hạn thật — ghi ra thay vì để người sau tự phát hiện

1. Tệp bàn đo **CHÉP** CSS/DOM của `EditorPanel.vue` và thuật toán của `editorGutter.ts`;
   nó **không mount** component thật. Lệch nhau được mà **không cổng nào đỏ**.
2. Ba font nhúng của UX-DR4 **vắng mặt** ⇒ chỗ một dòng ngắt khác sản phẩm, nên **câu nào
   nằm cùng dòng với câu nào** ở đây không phải câu trả lời của sản phẩm. Thứ **không** phụ
   thuộc font: hai câu cùng một dòng thì hai vạch trùng `top` **và** trùng `left`.
3. Engine là **WebKit của Playwright**, không phải **WKWebView của Tauri** — cùng món nợ
   `deferred-work.md:2127-2134`.
4. Bàn đo **không** đo tương phản. Sàn WCAG của `confirmed` trên hai theme thuộc Task 8.4.
5. Bàn đo **không** trả lời *"ca này thường gặp tới đâu trên dữ liệu thật"*. Đó là một phép
   đo khác, trên 21 Chương thật, và **không** thuộc story này.
