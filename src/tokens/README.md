Token màu và chữ — **đã kiểm tương phản WCAG AA ở cả hai theme** (AD-34). Không giá trị màu nào nằm trong component.

**Story sở hữu nội dung: 1.4.**

---

## Bốn tệp, một nguồn sự thật

| Tệp | Vai |
|---|---|
| `tokens.json` | **Nguồn sự thật duy nhất.** 16 màu × 2 theme · 14 typography · 4 họ · spacing · rounded · danh sách cặp tương phản · sổ deviation |
| `index.ts` | `applyTheme('light' \| 'dark')` — ghi CSS custom properties lên `document.documentElement`. Gọi **trước `mount()`** |
| `fonts.ts` | `loadFonts()` — đăng ký bốn `FontFace` từ `$RESOURCE/fonts/**` |
| `reset.css` | Reset toàn cục. Import trong `main.ts` |

**Không tạo một tệp `.css` viết tay mang cùng giá trị.** `tsconfig.json` đã bật `resolveJsonModule` nên TS đọc `tokens.json` trực tiếp, và `scripts/check-tokens.mjs` đọc bằng `JSON.parse` — cùng một tệp, hai người tiêu thụ. Hai bản chép sẽ lệch nhau ở lần sửa thứ ba.

## Quy ước tên biến CSS — chốt một lần, 127 story sau dùng lại

```
--color-<token>       16 màu của theme đang áp      var(--color-on-surface)
--family-<họ>         4 họ chữ                      var(--family-read)
--space-<token>       khoảng cách + thước đọc       var(--space-panel-inline)
--radius-<token>      bo góc (DEFAULT → default)    var(--radius-default)
```

Mỗi token typography phát **bảy** biến, mỗi biến đúng một nghĩa:

```
--font-<t>       font-size        --leading-<t>    line-height
--weight-<t>     font-weight      --style-<t>      font-style
--tracking-<t>   letter-spacing   --synthesis-<t>  font-synthesis
--face-<t>       font-family, trỏ về --family-<họ> của chính token
```

Dùng `--face-<t>` thay vì `--family-<họ>` khi áp một token: một lần đổi họ chữ của token đi theo mà không phải sửa ở nơi tiêu thụ.

## Cổng cưỡng chế

`npm run check:tokens` — bảy phép kiểm, mã thoát là phán quyết, đã gắn vào `ci.yml` trong job `check`. Cả bảy đã nghiệm thu **đỏ-rồi-xanh** (52 ca sau lượt rà soát 2026-08-03).

Tầm quét là `src/**` cộng `index.html` ở gốc repo, trên tám đuôi tệp: `.css` `.scss` · `.vue` `.html` `.svg` · `.ts` `.tsx` `.js` `.jsx` `.mjs` `.cjs`. Symlink bị **bỏ qua** và ghi tên ra.

Điều đáng biết nhất về cổng này: Kiểm C không kiểm những cặp màu *tình cờ tồn tại trong mã* mà kiểm **danh sách đã khai** ở `tokens.json`, và nó cưỡng chế **tính đầy đủ** — mọi tổ hợp (chữ × nền) phải nằm ở `contrast.pairs` hoặc ở `contrast.excluded`. Khi bạn dựng component mới với một cặp màu mới, **thêm cặp đó vào `pairs`**; im lặng bỏ qua là FAIL.

**Không một phán quyết nào của cổng đọc tham số từ `tokens.json`.** Sàn WCAG (4,5 / 3,0), danh sách vai, danh sách cặp loại trừ, danh sách màu đã loại — tất cả đóng băng trong chính `check-tokens.mjs`. `tokens.json` được phép *nhắc lại* chúng cho người đọc và cổng đối chiếu hai bản, nhưng nó không phải nơi phán quyết đọc ngưỡng của mình ra. Lượt rà soát 2026-08-03 chạy thật ba đường thoát và cả ba cho exit 0 trong khi sản phẩm mang một cặp 4,245:1: hạ `contrast.floors` · **chuyển** cặp trượt sang `excluded` với một chuỗi lý do bất kỳ · thêm một mục `deviations` không có lý do. Cả ba nay đều đỏ.

### Ba đường miễn trừ CÓ TÊN — và không có đường thứ tư

Khi cổng đỏ oan, đường ra là **viết một câu giải thích**, không phải nới quy tắc:

| Comment | Cho phép gì | Vì sao có |
|---|---|---|
| `/* aura-allow-opacity: <lý do> */` | một `opacity` trung gian | Kiểm D đỏ với **mọi** `opacity` khác 0 và khác 1 trong `src/**` (Ice chốt 2026-08-03) — kể cả trên thẻ bọc, kể cả `var()`/`calc()` không tĩnh. Nét và nền thật thì khai ra. |
| `/* aura-allow-z-index: <lý do> */` | một `z-index` | Ngữ cảnh xếp lớp là nhu cầu **cơ học** (dropdown, tooltip, dockview), khác với bóng đổ là quyết định thị giác mà AC7 cấm thẳng. `box-shadow`/`text-shadow` **không** có đường này. |
| `/* aura-allow-literal: <lý do> */` | một chuỗi hình dạng hex trong mã | `#dad`, `#decade`, `href="#face"` là hex hợp lệ mà không phải màu. Đừng nới regex — một cổng chỉ đường sai sẽ bị người sau thêm ngoại lệ cho tới khi nó không bắt được gì. |

Miễn trừ phải nằm trong phạm vi **một dòng** của khai báo, và phần `<lý do>` không được rỗng.

## Ba thứ sẽ hỏng im lặng nếu bạn không biết

### 1. Font KHÔNG nằm trong `src/` — đừng `@font-face { src: url('./fonts/…') }`

Bốn tệp sống ở `src-tauri/resources/fonts/` và tới webview qua **asset protocol**. Một `url()` tương đối sẽ được Vite giải thành asset của bundle, tức font bị **nhân bản vào `dist/`**: cộng thẳng ~26 MiB trong khi NFR6 chỉ còn ~47 MB dư địa. Nó *chạy được* trên máy dev, nên bẫy chỉ lộ ở phép đo dung lượng của Story 1.9.

`fetch()` tới asset protocol cũng **gãy ở bản đóng gói**: CSP là `connect-src 'self' ipc: http://ipc.localhost` — không có `asset:`, trong khi `font-src` thì có.

⇒ Đường duy nhất: `resolveResource()` → `convertFileSrc()` → `new FontFace(...)`.

### 2. `Source Sans 3` có mặc định trục `wght = 200` — descriptor `{ weight: '200 900' }` là bắt buộc

**Đã đo thẳng từ tệp (Story 1.4, đọc bảng `fvar`/`name`):**

```
SourceSans3[wght].ttf        name ID 1 = "Source Sans 3 ExtraLight"
                             fvar wght: min 200 · MẶC ĐỊNH 200 · max 900
SourceSerif4[opsz,wght].ttf  fvar wght: min 200 · mặc định 400 · max 900
                             fvar opsz: min 8 · mặc định 20 · max 60
NotoSerifCJKtc-Regular.otf   KHÔNG có fvar — tệp tĩnh, một nét duy nhất
```

**Nét 600 và 700 của `Source Sans 3` nay đã được dựng thật và là nét THẬT** (ảnh chụp Story 1.4 Task 4, bốn nét 200/400/600/700 phân biệt rõ trên chuỗi dày dấu tiếng Việt). Mệnh đề này trước đó chỉ mới được chứng minh cho `Source Serif 4`.

⚠️ **Một sắc thái đo được, ghi thẳng vì nó ngược với dự đoán:** đối chứng *thiếu* descriptor **vẫn ra nét đúng trên Blink** (Chrome 2x, macOS) — Blink đọc `fvar` và nội suy trục dù `@font-face` không khai dải nét. Nên descriptor vẫn **bắt buộc** (nó là thứ đặc tả dựa vào, và `scopeCheck.ts:165,223` đã dùng), nhưng mệnh đề *"thiếu nó thì chắc chắn ra chữ mảnh"* **chưa đúng cho mọi engine**. WKWebView chưa đo.

### 3. Chữ Hán nghiêng giả ở `source-hanviet` và `lookup-example`

`families.read` có `Noto Serif CJK TC` trong chuỗi dự phòng, nên chữ Hán rơi vào tệp CJK qua fallback — mà tệp đó chỉ có Regular. Hai token trên đều `italic`, nên trình duyệt **tổng hợp nghiêng giả** cho phần Hán.

**Lời giải đã chọn: `fontSynthesis: 'none'` khai ở chính hai token đó** (`tokens.json`), phát ra `--synthesis-<token>`.

Vì sao đường này chứ không phải hai đường kia:

| Đường | Phán quyết |
|---|---|
| Thêm tệp nghiêng CJK | ~23 MiB, một phần ba ngân sách font, dư địa NFR6 chỉ còn ~47 MB |
| Chấp nhận nghiêng giả | `lookup-example` là ví dụ từ điển Trung–Việt ở 12,5px — đúng cỡ mà nghiêng giả xấu nhất |
| `unicode-range` + `@font-face` riêng | Làm được, nhưng phải bảo trì một dải mã CJK viết tay ở tầng token, và nó không phủ được ký tự nằm ngoài dải mình nghĩ ra |
| **`font-synthesis: none`** | ✅ Một thuộc tính, **0 byte**, 0 bộ nhớ thêm, không dải mã nào phải bảo trì |

**Đã dựng thật và chụp lại (Story 1.4 Task 9):** ở `auto`, 橫看成嶺側成峰 nghiêng rõ; ở `none`, chữ Hán đứng thẳng **trong khi phần Latin vẫn nghiêng thật** — vì `Source Serif 4` có tệp Italic riêng nên không cần tổng hợp, và `font-synthesis: none` chỉ tắt phần *tổng hợp*.

⚠️ Nó cũng tắt tổng hợp **nét đậm** cho hai token đó. Đây là tác dụng phụ *mong muốn*: nếu descriptor nét sai ở đâu đó, ta muốn thấy chữ mảnh (lỗi hiện ra) hơn là nét đậm giả (lỗi ẩn đi).

## Ba deviation khỏi bảng `DESIGN.md` đang chờ Ice phê chuẩn

Xem khối `deviations` trong `tokens.json` — mỗi mục có số đo và lý do, và `check-tokens.mjs` cưỡng chế rằng **không có chỗ lệch nào khác**. Không sửa `DESIGN.md`: đó là quyết định của Ice, không phải hệ quả phụ của một lượt cài đặt.
