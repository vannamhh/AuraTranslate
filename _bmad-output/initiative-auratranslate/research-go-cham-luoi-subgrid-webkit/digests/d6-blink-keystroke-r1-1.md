# D6 - Chi phi moi phim trong contenteditable (Blink / WebView2) - digest r1-1

Ngay truy cap: 2026-11-06. Gioi han: 12 nguon, 15 lenh goi; da dung khoang 11 lenh goi. Luu y quan trong: WebFetch tra ve ban tom tat do mo hinh nho viet, khong phai nguyen van; cac "trich dan" ham duoi day la ten ham / dong goi do WebFetch bao lai tu ban base64 giai ma, nen do tin cay la medium cho den khi doi chieu nguyen van tren source.chromium.org.

## Cac khang dinh

1. Claim: `CompositeEditCommand::Apply()` goi `GetDocument().UpdateStyleAndLayout(DocumentUpdateReason::kEditing)` truoc `DoApply(&editing_state)` (kem comment "Changes to the document may have been made since the last editing operation"), sau do goi `AppliedEditing()` tru khi la typing command. Day la tuong duong Blink cua WebKit `updateLayoutIgnorePendingStylesheets()`.
   - URL: https://chromium.googlesource.com/chromium/src/+/main/third_party/blink/renderer/core/editing/commands/composite_edit_command.cc?format=TEXT
   - Publisher: Chromium (googlesource). Ngay: nhanh main, doc 2026-11-06. Accessed 2026-11-06.
   - Confidence: medium (qua tom tat WebFetch cua ban base64; 100000/137888 ky tu dau duoc doc). Class: source-code.

2. Claim: Cung file con goi `UpdateStyleAndLayout` trong nhieu ham phu: `EndingVisibleSelection()`, `InsertNodeBefore()`, `InsertNodeAt()`, `AppendBlockPlaceholder()`, `AddBlockPlaceholderIfNeeded()`, `PositionOutsideTabSpan()`, `CanRebalance()`, `DeleteInsignificantText()`, `PrepareWhitespaceAtPositionForSplit()`, `RebalanceWhitespace()`, `CleanupAfterDeletion()`. Moi lan goi la mot diem co the ep style/layout neu DOM dang ban. Khong thay `UpdateStyleAndLayoutIgnorePendingStylesheets` trong phan da doc.
   - URL: nhu tren. Confidence: medium. Class: source-code.

3. Claim: Trong `typing_command.cc`, `UpdateStyleAndLayout(DocumentUpdateReason::kEditing)` xuat hien 4 cho: `TypingCommand::InsertText` (overload nhan `passed_selection_for_insertion`, goi `document.UpdateStyleAndLayout`), `TypingCommand::AdjustSelectionAfterIncrementalInsertion`, `TypingCommand::DeleteKeyPressed`, `TypingCommand::ForwardDeleteKeyPressed`. Nhu vay mot ky tu go (InsertText) co it nhat mot lan ep cap nhat style+layout truoc khi ap lenh, va them mot lan sau chen tang dan (`AdjustSelectionAfterIncrementalInsertion`). Cac ham lien quan: `InsertTextInternal`, `DoApply`, `TypingAddedToOpenCommand`.
   - URL: https://chromium.googlesource.com/chromium/src/+/main/third_party/blink/renderer/core/editing/commands/typing_command.cc?format=TEXT
   - Publisher: Chromium. Accessed 2026-11-06. Confidence: medium (tom tat WebFetch). Class: source-code.

4. Claim: `UpdateStyleAndLayout` cua Blink la thao tac cap Document: no chi lam viec neu style/layout dang ban (dirty), nhung khi bi DOM mutation lam ban thi chi phi style recalc phu thuoc vao so phan tu bi invalid va cac selector toan cuc (vd `:has()`, selector hau due), khong chi vung duoc sua. Bang chung gian tiep o claim 6. Day la suy luan tu cac nguon, khong phai doan trich nguon Blink noi thang.
   - Confidence: low-medium. Class: pattern.

5. Claim: Co bao cao Chromium (Windows Chrome) rang go trong contenteditable cham con vai khung/giay, "Forced Layout is happening on every keystroke"; dat `overflow-y: hidden` lam het layout, dat div tuyet doi (absolute) tranh reflow. Firefox on dinh. Chua doc duoc trang goc (can dang nhap), chi nhan qua ket qua tim kiem; trang thai hien tai KHONG xac dinh duoc.
   - URL: https://issues.chromium.org/issues/40688351 ("Continues typing in contenteditable is slow in certain cases")
   - Publisher: Chromium issue tracker. Ngay bai: khong ro. Accessed 2026-11-06. Confidence: low (khong doc duoc nguyen van; WebFetch chi thay trang dang nhap). Class: bug-status.

6. Claim: Bao cao Chromium thu hai https://issues.chromium.org/41237496 duoc mo ta (qua search) la go moi chu mat ~100 ms trong contenteditable, chi phi chinh la layout va frame. Khong doc duoc noi dung. Confidence: low. Class: bug-status.

7. Claim: Trong ung dung WebView2 tren Windows, go cham trong khung chat dai duoc truy ra la selector CSS `:has()` gay style recalc tren toan chat moi phim; median keydown-to-second-frame giam 160.1 ms -> 5.0 ms sau khi sua selector (chat tong hop 600 tin nhan). Tren headless Chromium: style recalc moi phim 22.7-26.4 ms -> 0.9-2.4 ms; bisect 200-turn: 619 ms -> thap sau sua; chi phi tang theo do dai chat (23.7 ms/phim o 30 luot -> 100.3 ms). Day la bang chung rang trong Blink, chi phi moi phim co the ti le voi kich thuoc DOM toan tai lieu qua style recalc do selector, khong qua layout toan tai lieu. Day la nguon mot (PR duy nhat, tu bao cao cua tac gia, chua doc doc lap), khong phai WebView2-specific: cung hanh vi tren headless Chromium.
   - URL: https://github.com/unslothai/unsloth/pull/16656
   - Publisher: unslothai (GitHub). Ngay: mo 2026-11-04, review 2026-11-05. Accessed 2026-11-06.
   - Confidence: medium (so do cua tac gia, mot nguon; tinh nhat quan voi claim 3). Class: performance-number.

8. Claim: Cung mot DOM, WebView2 va headless Chromium cho cung kieu hanh vi (chi phi style theo kich thuoc DOM) trong PR tren; khong thay bang chung rang WebView2 them chi phi rieng moi phim ben tren Blink. Bao cao WebView2 lien quan toi tre go khac: #1564 openclaw-windows-node (cua so WebView2 4K phong to, khoang cach giua commit khung va presentation; chu so huu chua ro; ~35 ngay tuoi, chua dong) - day la tre trinh chieu, khong phai chi phi DOM.
   - URL: https://github.com/openclaw/openclaw-windows-node/issues/1564 (chi doc qua ket qua search)
   - Confidence: low. Class: bug-status.

9. Claim: WebView2Feedback #3003 / WindowsAppSDK #3266 (go nhanh lam chuot khong nhan, WinAppSDK 1.2) va #4325 (SetWindowDisplayAffinity, runtime 121.0.2277.83, sua o 121.0.2277.98) la loi khac, khong phai go cham trong contenteditable lon.
   - URLs: https://github.com/MicrosoftEdge/WebView2Feedback/issues/3003 , https://github.com/MicrosoftEdge/WebView2Feedback/issues/4325 , https://github.com/microsoft/WindowsAppSDK/issues/3266
   - Confidence: low-medium (chi snippet tim kiem). Class: bug-status.

10. Claim: WebKit bug 232673: outline focus mac dinh tren contenteditable lam go cham, tang theo do dai tai lieu (Safari 15, Apple Silicon); trang thai NEW, khong nhac Blink. Lien quan do bieu hien doi ngau tren macOS WKWebView, nhung khong bao gio dung de giai thich Windows.
   - URL: https://bugs.webkit.org/show_bug.cgi?id=232673 . Publisher: WebKit Bugzilla. Radar 2021-11-10. Confidence: medium. Class: bug-status.

11. Claim (so cap): Blink 2016 Chromium Code Review them "layout update after TypingCommand::willAddTypingToOpenCommand" roi bi revert trong dot refactor (crbug 673789). Khong xac nhan duoc co duoc dua lai hay khong; claim 3 cho thay lan goi `UpdateStyleAndLayout` van ton tai o ban hien tai.
   - URL: https://codereview.chromium.org/2634133002 . Confidence: low (chi snippet). Class: source-code.

## Tra loi cho cac cau hoi

- Q1: Co - Blink goi `UpdateStyleAndLayout(DocumentUpdateReason::kEditing)` truoc `DoApply` trong `CompositeEditCommand::Apply` va trong `TypingCommand::InsertText` / `AdjustSelectionAfterIncrementalInsertion`. Day la goi cap Document, nhung chi ton chi phi khi style/layout dang dirty.
- Q2: Khong xac lap duoc bang nguon Blink rang layout toan tai lieu bi ep moi phim. Co bang chung gian tiep (claim 7, cac bai viet contenteditable.realerror.com chi la tom tat thu cap, khong dung lam bang chung) rang style recalc co the ti le theo kich thuoc DOM khi co selector rong.
- Q3: Khong tim thay bao cao WebView2Feedback ve contenteditable cham dac thu WebView2.
- Q4: Khong tim thay hoi quy Blink 150-157 ve editing / input latency / forced layout.

## Huong dang theo duoi

- Doc nguyen van tren source.chromium.org: `CompositeEditCommand::Apply`, `TypingCommand::InsertText`, `AdjustSelectionAfterIncrementalInsertion` (WebFetch chi cho ban tom tat).
- Chay DevTools/ETW trace tren WebView2 that: do `UpdateStyleAndLayout` (Recalculate Style vs Layout) moi phim; kiem tra selector `:has()`, hau due, `content-visibility`, outline focus (WebKit 232673).
- Dang nhap de doc Chromium issue 40688351 va 41237496.
- Kiem tra khac biet GPU/compositor Windows (frame presentation) nhu openclaw #1564.

## Da tim nhung KHONG thay

- Tai lieu thiet ke Chromium ve chi phi go theo kich thuoc tai lieu: khong thay.
- Mot bao cao WebView2Feedback ve contenteditable / typing lag lien quan Blink: khong thay (chi ket qua ve loi khac).
- Ghi chu phat hanh WebView2 / Edge 150-157 nhac hoi quy editing: khong thay (khong doc release notes truc tiep trong ngan sach).
- Hoi quy Blink phien ban 150-157: khong thay; vang mat bang chung khong chung minh khong ton tai.
- Trang thai hien tai cua hai Chromium issue: khong xac dinh duoc.
