# D10 - Chi phi cua font CJK OTF/CFF lon trong Blink tren Windows (vong 1)

Ngay truy cap: 2026-11-06. Ngan sach dung: khoang 9 lan goi cong cu, nhieu trang nguon Blink (googlesource, raw.githubusercontent) tra 404/403 nen KHONG doc duoc ma nguon Blink truc tiep. Ket luan chinh: khong thiet lap duoc bang chung.

## Cac claim

1. Skia strike cache (bo nho dem glyph da raster) co gioi han mac dinh theo bien dich: SK_DEFAULT_FONT_CACHE_COUNT_LIMIT = 2048 muc, SK_DEFAULT_FONT_CACHE_LIMIT = 2 * 1024 * 1024 byte; ban cu con SK_DEFAULT_FONT_CACHE_POINT_SIZE_LIMIT 256. Nguon: https://skia.googlesource.com/skia/+/main/src/core/SkStrikeCache.h (cung ban m71: https://skia.googlesource.com/skia/+/chrome/m71/src/core/SkStrikeCache.h). Publisher: Skia/Google. Ngay: khong ro (ket qua tim kiem tom tat, chua doc nguyen trang). Truy cap 2026-11-06. Do tin: thap-trung binh (chi doc tom tat cua cong cu tim kiem; Chromium co the dat gioi han rieng luc chay). Lop: source-code. Y nghia: gioi han tinh theo so strike/bo nho glyph da raster, khong theo kich thuoc tep font; font 24.5 MB khong tu no lam tran cache, nhung so glyph khac nhau (rubi + chu CJK) co the.

2. Blink co co che don FontCache khi ap luc bo nho (ham invalidate/purge goi purgeFallbackListShaperCache, cung duong voi invalidateShapeCache). Nguon: https://codereview.chromium.org/2081333004 . Publisher: Chromium Code Reviews. Ngay: khoang 2016. Do tin: thap (cu 10 nam, chi doc tom tat). Lop: pattern. Khong co bang chung ve chinh sach eviction hien hanh cua ShapeCache.

3. Blink FontDataCache co ban va (Electron/openSUSE) them LRU tham chieu manh de font dung gan day khong bi giai phong; nghia la font co the bi huy va dung lai (tai tao) khi khong con tham chieu. Nguon: https://src.opensuse.org/rpm/nodejs-electron/raw/branch/devel/bad-font-gc000.patch (chi thay qua tom tat tim kiem, trang tai ve 403). Do tin: thap. Lop: source-code. Goi y: neu font bi giai phong/tao lai moi lan bo cuc thi chi phi co the lap lai; chua xac minh.

4. Mozilla ghi nhan font CJK Windows thuong vai MB moi mat, mot so >10 MB, nen doi font mac dinh thay doi luong du lieu doc. Nguon: https://bugzilla.mozilla.org/show_bug.cgi?id=1353639 . Publisher: Mozilla. Do tin: thap (Firefox, khong phai Blink; chi la boi canh). Lop: pattern.

5. Skia/Chromium co xu ly rieng cho DirectWrite voi font CJK (quay ve aliased khi ClearType rong; ep aliased o co nho cho font Dong A thieu bitmap nhung). Nguon: https://codereview.chromium.org/209163002 , https://codereview.chromium.org/339693002 (2014). Do tin: thap (cu, qua tom tat). Lop: pattern. Chung to DirectWrite co duong rieng cho CJK, KHONG chung minh no cham.

6. Tai lieu Microsoft: custom scheme dang ky tham gia CORS va CSP; ung dung phai dat header truy cap trong WebResourceRequested. Nguon: https://learn.microsoft.com/ru-ru/dotnet/api/microsoft.web.webview2.core.corewebview2customschemeregistration . Do tin: trung binh. Lop: spec. Khong thay bao cao nao ve tai nhieu lan hay giai ma lai font qua custom scheme.

7. Blink (2020) co thay doi "font-display: optional" khong relayout, ly do: relayout sau khi font tai lam bo cuc khong on dinh. Nguon: https://groups.google.com/a/chromium.org/d/msg/blink-dev/QsgMjIyO8qA/RWWWukoaAgAJ . Do tin: thap-trung binh. Lop: pattern. Chi noi ve relayout MOT LAN luc doi font, khong ve chi phi moi lan go.

8. Co bao cao Chromium: contenteditable tren Windows 10 Chrome bi forced layout moi lan go phim, tren Mac thi khong; workaround dat position absolute. Nguon: https://issues.chromium.org/issues/40688351 (qua tom tat tim kiem, chua doc trang; trang issue tracker thuong khong render duoc). Do tin: thap. Lop: bug-status (trang thai hien tai KHONG xac minh). Day la nguyen nhan khac, doc lap voi font.

## Huong nen theo tiep
- Doc truc tiep ma nguon Blink qua source.chromium.org (shape_cache.h, font_cache.cc, font_data_cache.cc, simple_font_data.cc) - cac duong dan googlesource/raw github deu 404 trong lan nay; can URL dung (co the nam trong third_party/blink/renderer/platform/fonts/).
- Skia SkScalerContext_DW (Windows) va SkTypeface_win_dw.cpp: kiem tra co doc lai bang font / glyph metrics moi lan khong.
- Do truc tiep: Chrome DevTools Performance trace tren Windows, xem thoi gian nam o Layout, Shape (HarfBuzzShaper), hay Paint/Raster (DirectWrite glyph). So sanh voi cung font nhung cat tap con (subset) va voi font TTF (glyf).
- Thu nghiem cach ly: bo ruby, bo font, doi sang font he thong, de phan biet chi phi font va chi phi ruby/bo cuc.
- WebView2Feedback tren GitHub: tim "font" "custom scheme" (chua tim duoc).

## Khong tim thay (la mot ket qua)
- Khong tim thay chinh sach eviction hien hanh cua ShapeCache trong Blink, hay bug Chromium ve ShapeCache thrash voi CJK.
- Khong tim thay bug Chromium ve DirectWrite cham voi font CJK lon hay CFF (chi co thay doi 2014 ve aliasing va kiem soat so font he thong).
- Khong tim thay bao cao WebView2/Chromium ve font qua custom scheme bi tai lai, cache miss, hay giai ma lai.
- Khong tim thay bang chung rang kich thuoc web font lam cham layout/paint SAU khi da tai; moi nguon chi noi ve chi phi luc tai/doi font.
- Khong tim thay bao cao go phim cham lien quan font CJK lon trong contenteditable.
- Khong so sanh duoc Windows (DirectWrite) voi macOS (CoreText) ve cache raster CFF.
