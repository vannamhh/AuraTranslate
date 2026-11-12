# Handoff B: vì sao một lần sửa ô bản dịch ở Song song tốn ~40 ms Layout trên Chromium

Build: app dev (vite) trong binary wdio debug `src-tauri/target/debug/auratranslate` (WKWebView, macOS, dpr 2, cửa sổ 1280x860) chỉ để CHỤP DOM;
đo bằng Chrome 157.0.8103.52 (Playwright headless, macOS, dpr 1, 1280x860). Mọi số là Layout trung vị (p90) ms của sự kiện
`Layout` trong vệt CDP, 40 lần `execCommand('insertText','ê')` + `offsetHeight` + 1 rAF sau 5 lần khởi động, lượt 1 / lượt 2.
Dữ liệu: Chương 90 hàng và 200 hàng, mỗi câu 17 từ Hán, ô dịch đã điền; rt chỉ là "·" (từ điển không có trong e2e; thay bằng âm
thật không đổi số khi KHÔNG có thanh cuộn: 1,39 so với 1,42).

## Kết luận
Culprit: ô cuộn `.grid-scroll` (`flex:1` = `1 1 0%`, `overflow:auto`, `src/panels/GridPanel.vue` ~2177) khi thanh cuộn chiếm chỗ
(`::-webkit-scrollbar{width:10px}` toàn cục ở `src/tokens/controls.css:729`) và chuỗi chiều cao theo phần trăm phía trên nó
(`.panel{height:100%}` ở `PanelFrame.vue:127`, `dv-groupview`/`dv-vue-part` `height:100%` của dockview). Thiếu một trong ba thì hết chậm.
Playwright headless thêm `--hide-scrollbars` mặc định, nên MỌI bàn đo Chrome trên Mac trước đây (D11, `do-tren-windows.html`) không có thanh cuộn
và nhanh. Bỏ cờ đó (`AURA_BENCH_SCROLLBARS=1`) là tái hiện được.
Cơ chế là SUY LUẬN, chưa đo trực tiếp: ở lượt bố cục đầu của flex item chiều cao chưa xác định, ô cuộn chưa có thanh dọc (rộng 638);
lượt cuối có thanh (628). Hai bề rộng luân phiên làm bộ đệm kết quả bố cục của cả cây ruby trượt mỗi lần, nên mỗi lần sửa là bố cục "nguội" cả cây.
Bằng chứng gián tiếp: bố cục nguội cưỡng bức trên ảnh chụp Song song = 21-56 ms, Nguyên văn = 3,4 ms (đổi `column-gap`/cỡ chữ `.hv-parallel`).

## Bảng số (Chương 90 hàng, tổng đối tượng 9.378, dirty 37, partial=false, giống vệt Windows)
| Ảnh chụp / biến thể | Layout trung vị (p90) |
|---|---|
| Song song, KHÔNG thanh cuộn (cờ mặc định Playwright) | 1,42 (1,70) / 1,67 (2,22) |
| Song song, có thanh cuộn 10px | 38,7 (69,1) / 38,9 (59,1) |
| Song song 200 hàng, có thanh cuộn | 98,6 (132,1) / 91,8 (132,3) |
| Chuyển đổi (tab Hán Việt, ruby chưa bật), có thanh | 14,6 (20,4) / 14,3 (20,8) |
| Nguyên văn, có thanh | 5,2 (5,9) / 5,5 (5,9) |
| Nguyên văn 200 hàng, có thanh | 8,2 (11,0) / 8,5 (10,3) |
| Vệt Windows thật (VM, WebView2, v1.0.2), tham chiếu | 67 / 20 / 7 |
Thứ tự và tỉ lệ ba kiểu khớp vệt Windows; số tuyệt đối trên Mac thấp hơn (suy luận: CPU máy ảo chậm hơn).

## Xác nhận hai chiều
Gỡ CHỈ culprit khỏi ảnh chụp đầy đủ (có thanh cuộn), 90 hàng, Song song, lượt 1 / lượt 2:
| Gỡ bằng | Layout trung vị |
|---|---|
| `.grid-scroll::-webkit-scrollbar{display:none}` | 1,44 / 1,47 |
| `.grid-scroll{scrollbar-gutter:stable}` | 1,56 / 1,58 (200 hàng: 99 và 92 xuống 2,67 / 2,69) |
| `.grid-scroll{overflow-y:scroll}` | 1,53 / 1,51 |
| `.grid-scroll{flex:1 1 0px}` (đổi 0% thành 0px) | 1,41 / 1,38 |
| `.grid-scroll{flex:none;height:725px}` | 1,42 / 1,43 |
| `.panel{height:781px}` (thay `100%` bằng px) | 1,42 / 1,37 |
Không chữa được: gỡ margin/padding âm của `.grid-scroll` (49,8 / 51,0), `overflow-x:hidden` (52,4 / 46,9), `.grid-scroll{flex:1 1 auto}` (48,8),
`.panel{padding:0}` (40,1), thanh cuộn 1px (43,2 / 39,6). Tách `section.panel` ra khỏi dockview vào div 640x781 cố định: 1,49 (nhanh).
THÊM vào trang tổng hợp (`do-tren-windows.html` + ô cuộn có thanh 10px; 90 hàng), `bench-synth.mjs`:
| Biến thể trang tổng hợp | Layout trung vị lượt 1 / 2 |
|---|---|
| gốc (cuộn theo viewport) | 1,11 / 1,11 |
| groupview(flex cột, 100%) > content(flex:1 1 auto) > vue-part(100%) > panel(100%) > panel-body(flex:1) > #grid(flex:1, auto, thanh 10px) | 42,8 / 41,1 (p90 64 / 68) |
| như trên, `#grid{flex:1 1 0px}` | 1,22 / 1,22 |
| như trên, `overflow:scroll` hoặc `scrollbar-gutter:stable` | 1,33 / 1,26 và 1,32 / 1,32 |
| như trên, `.vp`/panel/`.cc` đổi chiều cao phần trăm thành px | 1,21 / 1,22 |
| như trên, thanh ẩn | 1,22 / 1,31 |
Các cấu trúc tối giản hơn (chỉ ô cuộn flex, hoặc panel > panel-body > #grid mà không có chuỗi `height:100%` phía trên) KHÔNG chậm (1,1-1,3).
Trang tổng hợp 200 hàng chưa chạy lại với chuỗi đầy đủ.

## Đột biến DOM mỗi lần sửa (WKWebView, app thật, 90 hàng)
`mutations.e2e.mjs`: một lần sửa ở Song song chỉ sinh MỘT bản ghi `characterData` ở text node của ô (lớp `editing` đã có sẵn từ lúc focus);
ở Nguyên văn thêm một đổi `class` của `div.dv-scrollable` (tab dockview). Không thuộc tính/style nào của lưới bị đổi, nên JS app không gây bố cục lại.

## Chưa kiểm / ngoài phạm vi
- Chưa chạy trên WebView2 Windows. Phép sửa (một trong các dòng CSS trên) cần đo lại trên máy ảo trước khi coi là xong; WebView2 156 so với Chrome 157.
- Nhập bằng `Input.insertText`, phím thật và Backspace+gõ (kiểu EVKey) trên snapshot không thanh cuộn đều 1,3-1,4 ms/lần bố cục: đường nhập không phải nguyên nhân.
- dpr 1,25 / 1,5 / 2 không có thanh cuộn: 1,4-1,5 ms. Chưa đo dpr khác khi có thanh cuộn.
- Không chọn phép sửa nào (hai cách hợp lệ trở lên: `scrollbar-gutter:stable` đổi bề rộng khả dụng; `flex:1 1 0px`; chiều cao px) cần số đo thiết kế cho Ice.

## Cách chạy lại
Mọi lệnh cần `PW_PATH=<thư mục chứa node_modules/playwright>` và tuỳ chọn `CHROME_PATH=<tệp Chrome>`; tệp tạm ở `os.tmpdir()` (đổi bằng `AURA_BENCH_TMP`). Các ảnh chụp KHÔNG được giữ lại: chụp lại bằng `capture-snapshot.e2e.mjs`.
Ảnh chụp nằm ngoài repo (2-4 MB mỗi tệp): `scratchpad/snaps/snap-<hàng>-<plain|hv-tab|parallel|plain-ed|parallel-ed>.json`. Dựng lại, NGOÀI sandbox, từ gốc repo:
`AURA_BENCH_ROWS=90 AURA_BENCH_OUT=<thư mục> npx wdio run e2e/wdio.conf.mjs --spec <chromium-bench>/capture-snapshot.e2e.mjs`
(binary wdio: `cargo build --locked --features wdio --manifest-path src-tauri/Cargo.toml`). Đột biến: `mutations.e2e.mjs`, cùng biến môi trường.
Đo, trong `chromium-bench/`:
`AURA_BENCH_SCROLLBARS=1 node bench-snap.mjs <snap.json> 'base;gutterStable;noSbGrid' 2 [40]` (`;` tách lượt, `,` ghép biến thể; biến thể ở `variants.mjs`)
`AURA_BENCH_SCROLLBARS=1 node bench-synth.mjs 2` (trang tổng hợp). `AURA_BENCH_INPUT=exec|key|cdp|evkey`, `AURA_BENCH_DPR=1.5`.
Không đặt `AURA_BENCH_SCROLLBARS` thì Playwright ẩn thanh cuộn và mọi số về ~1,4 ms. Font trỏ thẳng `src-tauri/resources/fonts/`, không chép.
