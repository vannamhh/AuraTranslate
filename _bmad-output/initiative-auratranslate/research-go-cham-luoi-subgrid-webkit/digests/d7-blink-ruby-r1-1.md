# D7 - Blink ruby: chi phi layout/paint va tuong tac voi CSS Grid (vong 1)

Ngay truy cap: 2026-11-06. Ngan sach da dung: 8 lan goi cong cu, 1 nguon doc day du (blink-dev). Ket qua tong the: MONG. Trang issues.chromium.org va chromestatus.com la trang JS, WebFetch chi tra ve khung rong, nen khong doc duoc trang thai bug.

## Cac phat hien

1. Claim: "Line-breakable ruby" (cho phep ngat dong ben trong `display: ruby`, moi ruby-base/ruby-text tach nhieu dong) duoc de xuat ship o Chrome 128 (desktop, Android, WebView), co Finch flag `RubyLineBreakable`; tac gia Kent Tamura (Blink>Layout>Ruby).
   - Nguon: https://groups.google.com/a/chromium.org/g/blink-dev/c/vkWGs7_yWZM
   - Publisher: blink-dev (Google Groups); ngay 2024-05-31; LGTM1 2024-06-03.
   - Truy cap: 2026-11-06. Confidence: high ve noi dung intent; class: version.
   - Luu y: day la intent ve ngat dong, KHONG phai thread "RubyNG rewrite". Cau noi "RubyNG ship quanh Chrome 128" trong de bai chi duoc xac nhan o muc phien ban ship cua tinh nang ngat dong, chua xac nhan o muc viet lai layout.

2. Claim: Intent ghi `<ruby>` xuat hien tren khoang 0,14% luot xem trang; thread (phan doc duoc) KHONG neu so do hieu nang truoc/sau nao.
   - Nguon: nhu tren. Confidence: high (la su vang mat trong phan doc duoc), nhung thread bi cat nen medium cho ket luan "khong co so do".
   - Class: performance-number (am tinh).

3. Claim: Intent tro toi design doc https://docs.google.com/document/d/1hzvrwoE_0aw08X_CaU40zV5bXbMQjY2SHQHj3Np4sDo , bug theo doi https://issues.chromium.org/issues/324111880 , chromestatus feature 5077282711666688. Ba muc nay CHUA doc duoc (tracker va chromestatus can JS/dang nhap).
   - Confidence: high ve viec cac lien ket ton tai trong intent; trang thai bug: khong biet.
   - Class: bug-status (khong xac lap).

4. Claim: Mot commit Chromium cua Koji Ishii lien quan bug 847274 go bo kiem tra RubyBase cu va dung phan tu "fabricated line-height" cho annotation ruby; day la dau vet noi bo cua viec chuyen ruby sang LayoutNG, nhung chi thay qua ket qua tim kiem, chua doc.
   - Nguon: https://github.com/chromium/chromium/commit/12434dba101b9fc2bd2347bde6d6315ee716bde8
   - Confidence: low (chi doc tom tat cua cong cu tim kiem). Class: source-code.

5. Claim: Co cac thread blink-dev khac ve ruby (ruby-align, ruby-overhang 2026-03, ruby-position unprefix 2020, display values ruby) - cho thay ruby van dang duoc phat trien tich cuc o Blink den 2026; khong cai nao la ve hieu nang.
   - Nguon: https://groups.google.com/a/chromium.org/g/blink-dev/c/_ty_-6f0u-Q , https://groups.google.com/a/chromium.org/g/blink-dev/c/EEdvr9Tv9as , https://groups.google.com/a/chromium.org/g/blink-dev/c/dKyoAe_tSQw
   - Confidence: medium (chi doc ket qua tim kiem). Class: version.

6. Claim: Ve Grid, cac bug Chromium tim thay chi la ve chi phi kich thuoc noi tai/baseline cua grid noi chung: issue "Resize performance regression" (grid track auto, do caching baseline alignment, sua o Chrome 71) va "[css-grid] Baseline Alignment may affect the grid's intrinsic size". Khong co cai nao nhac ruby.
   - Nguon: https://issues.chromium.org/issues/41407590 , https://issues.chromium.org/issues/40512278
   - Confidence: low-medium (tom tat tu ket qua tim kiem, khong doc trang). Class: bug-status; trang thai hien tai khong xac minh.

7. Claim: WebKit (khong phai Blink) co PR cho subgrid bi layout trong luc grid ngoai tinh intrinsic width; ghi chu noi thuat toan track tam thoi chay moi lan layout row subgrid co item canh baseline. Chi lien quan gian tiep, vi mac OS dung WebKit.
   - Nguon: https://github.com/WebKit/WebKit/pull/77251 . Confidence: low (chua doc). Class: pattern.

8. Claim: Tim "typing slow contenteditable" khong cho bug nao lien quan ruby; ket qua chung: DOM lon va forced reflow lam go cham trong Chrome (vi du https://issues.chromium.org/issues/40688351 , https://contenteditable.realerror.com/cases/ce-0026-performance-large-content/). Goi y tu ket qua: dat contenteditable position absolute tung giai quyet mot truong hop.
   - Confidence: low (tom tat tim kiem, cu, khong doc). Class: pattern.

## Huong nen theo tiep (leads)

- Doc bug 324111880 va tim cac bug con trong component Blink>Layout>Ruby (can trinh duyet co JS, vi du Browser pane, hoac Chromium Issue Tracker API). Loc "ruby" + "perf"/"slow"/"jank".
- Doc design doc docs.google.com/document/d/1hzvrwoE_0aw08X_CaU40zV5bXbMQjY2SHQHj3Np4sDo (can JS).
- Doc ma nguon: third_party/blink/renderer/core/layout/inline/ (ruby: `ruby_utils`, `inline_layout_algorithm`) va `ComputeMinMaxSizes` cho box ruby tren source.chromium.org de xem co cache min/max-content hay khong; va grid `grid_layout_algorithm.cc` cach cache `MinMaxSizes` cua item khi hang doi.
- Dung trace thuc tren Windows: Edge WebView2 DevTools Performance (Layout, Paint, "Update layer tree", shaping) de kiem tra thoi gian shaping chu Han (DirectWrite, font fallback) so voi layout.
- Bug 847274 (de bai tu ket qua tim kiem): doc truc tiep.

## Da tim va KHONG tim thay

- Khong tim thay thread "Intent to Ship" nao cho viec viet lai ruby LayoutNG (RubyNG) o muc layout; khong tim thay con so truoc/sau.
- Khong tim thay bug Chromium nao (mo hoac da dong) ve ruby cham/jank khi co nhieu phan tu, voi `ruby-position: under`, hay ruby trong noi dung editable/scrollable. Day la vang mat trong cac truy van toi da dung, KHONG phai bang chung rang bug khong ton tai (tracker khong tim duoc qua WebSearch).
- Khong tim thay bang chung ruby tuong tac xau voi CSS Grid / subgrid / intrinsic sizing, va khong co bang chung min/max-content cua ruby bi tinh lai thay vi cache khi anh em cung hang grid doi.
- Khong tim thay bang chung ve chi phi ruby khac nhau giua Windows va macOS trong Blink (DirectWrite vs CoreText). Chua truy van rieng ve shaping/font fallback CJK tren Windows trong vong nay (het ngan sach cho lan tim).
- Khong doc duoc trang thai hien tai cua bat ky bug nao (tracker can JS).
